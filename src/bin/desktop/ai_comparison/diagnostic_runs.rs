//! Immutable runnable diagnostic intent and exact cold archive custody. Numerical
//! payloads stay in recording::diagnostic; publication reuses the run-set owner.
use super::{
    engine::ExecutionPolicy,
    model_process::ProviderIdentity,
    scene::{DT, MU, RHO, Scene},
};
use crate::run_sets::{
    ArchiveBudget, archive_path, file_digest, publish_new, read_versioned, relative_path,
};
use rheon::recording::diagnostic::{
    BindingIdentity, DiagnosticReader, DiagnosticRun, PredictionOutcome, TerminalOutcome,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
const MAX_DOCUMENT: u64 = 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderPin {
    pub id: String,
    pub version: String,
    pub manifest_sha256: [u8; 32],
    pub checkpoint_sha256: Option<[u8; 32]>,
    pub training_run_id: Option<String>,
}
fn text(s: &str, max: usize) -> Result<(), String> {
    if s.is_empty() || s.len() > max || s.chars().any(char::is_control) {
        Err("Invalid bounded diagnostic text".into())
    } else {
        Ok(())
    }
}
fn hash_text(s: &str) -> Result<[u8; 32], String> {
    if s.len() != 64
        || !s
            .bytes()
            .all(|v| v.is_ascii_hexdigit() && !v.is_ascii_uppercase())
    {
        return Err("Invalid measured SHA256".into());
    }
    let mut out = [0; 32];
    for (i, v) in out.iter_mut().enumerate() {
        *v = u8::from_str_radix(&s[2 * i..2 * i + 2], 16).map_err(|_| "Invalid SHA256")?;
    }
    Ok(out)
}
impl ProviderPin {
    pub fn capture(p: &ProviderIdentity) -> Result<Self, String> {
        Ok(Self {
            id: p.id.clone(),
            version: p.version.clone(),
            manifest_sha256: hash_text(&p.manifest_sha256)?,
            checkpoint_sha256: p.checkpoint_sha256.as_deref().map(hash_text).transpose()?,
            training_run_id: p.training_run_id.clone(),
        })
    }
    pub fn identity(&self) -> ProviderIdentity {
        fn hex(v: &[u8; 32]) -> String {
            v.iter().map(|b| format!("{b:02x}")).collect()
        }
        ProviderIdentity {
            id: self.id.clone(),
            version: self.version.clone(),
            manifest_sha256: hex(&self.manifest_sha256),
            checkpoint_sha256: self.checkpoint_sha256.as_ref().map(hex),
            training_run_id: self.training_run_id.clone(),
        }
    }
    pub fn binding(&self) -> BindingIdentity {
        BindingIdentity {
            id: self.id.clone(),
            version: self.version.clone(),
            manifest_sha256: self.manifest_sha256,
            checkpoint_sha256: self.checkpoint_sha256,
            training_run_id: self.training_run_id.clone(),
        }
    }
    fn validate(&self) -> Result<(), String> {
        text(&self.id, 4096)?;
        text(&self.version, 4096)?;
        if let Some(v) = &self.training_run_id {
            text(v, 4096)?;
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenDiagnosticSpec {
    pub scene: Scene,
    pub policy: ExecutionPolicy,
    pub deployment: String,
    pub provider: ProviderPin,
    pub source_revision: String,
    pub recording: ArchiveBudget,
}
impl FrozenDiagnosticSpec {
    pub fn capture(
        scene: Scene,
        policy: ExecutionPolicy,
        deployment: &Path,
        identity: &ProviderIdentity,
    ) -> Result<Self, String> {
        scene.validate()?;
        let deployment = deployment
            .canonicalize()
            .map_err(|e| e.to_string())?
            .to_str()
            .ok_or("Deployment path must be UTF-8")?
            .to_owned();
        let spec = Self {
            recording: ArchiveBudget {
                max_database_bytes: 1024 * 1024 * 1024,
                max_record_bytes: 16 * 1024 * 1024,
                max_records: scene.steps - scene.warmup_steps,
            },
            scene,
            policy,
            deployment,
            provider: ProviderPin::capture(identity)?,
            source_revision: format!(
                "rheon-diagnostic-engine/{}; {}",
                env!("CARGO_PKG_VERSION"),
                option_env!("RHEON_SOURCE_REVISION").unwrap_or("build revision unspecified")
            ),
        };
        spec.validate()?;
        Ok(spec)
    }
    pub fn validate(&self) -> Result<(), String> {
        self.scene.validate()?;
        if self.policy.diagnostic_mode().is_none() {
            return Err("Legacy raw continuation has no typed saved diagnostic spec".into());
        }
        text(&self.deployment, 4096)?;
        if !Path::new(&self.deployment).is_absolute() {
            return Err("Saved deployment path must be absolute".into());
        }
        text(&self.source_revision, 1000)?;
        self.provider.validate()?;
        self.recording
            .limits()
            .validate()
            .map_err(|e| e.to_string())?;
        if self.recording.max_records > 512 {
            return Err("Diagnostic attempt budget exceeds supported horizon".into());
        }
        Ok(())
    }
    pub fn digest(&self) -> Result<[u8; 32], String> {
        self.validate()?;
        Ok(Sha256::digest(serde_json::to_vec(self).map_err(|e| e.to_string())?).into())
    }
    pub fn provenance(&self) -> Result<String, String> {
        let digest: String = self.digest()?.iter().map(|v| format!("{v:02x}")).collect();
        Ok(format!("rheon-frozen-diagnostic-spec-v1 sha256={digest}"))
    }
    pub fn bind_header(&self, run: &mut DiagnosticRun) -> Result<(), String> {
        self.validate()?;
        run.source_revision = self.source_revision.clone();
        run.provenance = self.provenance()?;
        Ok(())
    }
    fn verify_header(&self, run: &DiagnosticRun) -> Result<(), String> {
        let h = self.scene.h();
        if run.binding != self.provider.binding()
            || run.mode
                != self
                    .policy
                    .diagnostic_mode()
                    .ok_or("Unsupported diagnostic mode")?
            || run.grid.counts() != self.scene.counts
            || run.grid.spacing().map(f64::to_bits) != [h.to_bits(); 3]
            || run.grid.origin().map(f64::to_bits) != [0.0_f64.to_bits(); 3]
            || run.density.to_bits() != RHO.to_bits()
            || run.viscosity.to_bits() != MU.to_bits()
            || run.dt.to_bits() != DT.to_bits()
            || run.native_prefix_steps > self.scene.warmup_steps
            || run.source_revision != self.source_revision
            || run.provenance != self.provenance()?
        {
            return Err("Diagnostic archive differs from captured intent".into());
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunTerminal {
    Completed,
    StoppedOnRejection,
    Cancelled,
    Failed,
}
impl RunTerminal {
    pub fn recording(self) -> TerminalOutcome {
        match self {
            Self::Completed => TerminalOutcome::Completed,
            Self::StoppedOnRejection => TerminalOutcome::StoppedOnRejection,
            Self::Cancelled => TerminalOutcome::Cancelled,
            Self::Failed => TerminalOutcome::Failed,
        }
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunStatistics {
    pub native_prefix_steps: usize,
    pub produced_attempts: usize,
    pub dispatched_calls: usize,
    pub qualified: usize,
    pub rejected: usize,
    pub unavailable: usize,
    pub recorded_attempts: usize,
    pub last_advanced_generation: Option<u64>,
    pub model_start_seconds: f64,
    pub model_prediction_seconds: f64,
    pub run_wall_seconds: f64,
}
impl RunStatistics {
    fn validate(&self) -> Result<(), String> {
        if self.produced_attempts > 512
            || self.native_prefix_steps > 512
            || self.recorded_attempts > self.produced_attempts
            || self.dispatched_calls > self.produced_attempts
            || self.dispatched_calls < self.qualified.saturating_add(self.rejected)
            || self
                .qualified
                .checked_add(self.rejected)
                .and_then(|v| v.checked_add(self.unavailable))
                != Some(self.produced_attempts)
            || [
                self.model_start_seconds,
                self.model_prediction_seconds,
                self.run_wall_seconds,
            ]
            .iter()
            .any(|v| !v.is_finite() || *v < 0.0)
        {
            Err("Invalid diagnostic statistics".into())
        } else {
            Ok(())
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchivePin {
    pub sha256: [u8; 32],
    pub recorded_count: usize,
    pub terminal: RunTerminal,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticMember {
    pub member_id: String,
    pub run_id: String,
    pub spec: FrozenDiagnosticSpec,
    pub archive_file: String,
    pub finalized: Option<ArchivePin>,
    pub observed: Option<RunObservation>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunObservation {
    pub terminal: RunTerminal,
    pub detail: String,
    pub statistics: RunStatistics,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticRunSet {
    pub set_id: String,
    pub parent_sha256: Option<[u8; 32]>,
    pub baseline_member: String,
    pub members: Vec<DiagnosticMember>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    schema_version: u32,
    kind: ArtifactKind,
    sha256: [u8; 32],
    run_set: DiagnosticRunSet,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ArtifactKind {
    DiagnosticRunSet,
}
pub enum MemberState {
    Ready(DiagnosticReader),
    Incomplete(DiagnosticReader),
    Unpinned(DiagnosticReader),
    NotRun,
    Missing,
    Invalid(String),
}
impl DiagnosticRunSet {
    pub fn single(id: String, spec: FrozenDiagnosticSpec, parent_sha256: Option<[u8; 32]>) -> Self {
        Self {
            set_id: id.clone(),
            parent_sha256,
            baseline_member: "candidate".into(),
            members: vec![DiagnosticMember {
                member_id: "candidate".into(),
                run_id: id,
                spec,
                archive_file: "attempts.sqlite".into(),
                finalized: None,
                observed: None,
            }],
        }
    }
    fn validate(&self) -> Result<(), String> {
        text(&self.set_id, 4096)?;
        text(&self.baseline_member, 4096)?;
        if self.members.is_empty() || self.members.len() > 16 {
            return Err("Choose1–16 diagnostic members".into());
        }
        let mut ids = std::collections::BTreeSet::new();
        let mut runs = std::collections::BTreeSet::new();
        let mut files = std::collections::BTreeSet::new();
        for m in &self.members {
            text(&m.member_id, 4096)?;
            text(&m.run_id, 4096)?;
            m.spec.validate()?;
            relative_path(&m.archive_file).map_err(|e| e.to_string())?;
            if !ids.insert(&m.member_id)
                || !runs.insert(&m.run_id)
                || !files.insert(&m.archive_file)
            {
                return Err("Duplicate diagnostic member/run/archive identity".into());
            }
            if let Some(o) = &m.observed {
                text(&o.detail, 4096)?;
                o.statistics.validate()?;
            }
            if let Some(p) = &m.finalized {
                let o = m
                    .observed
                    .as_ref()
                    .ok_or("Pinned diagnostic member has no observed terminal")?;
                if p.recorded_count != o.statistics.recorded_attempts || p.terminal != o.terminal {
                    return Err("Diagnostic pin and observed terminal differ".into());
                }
            }
        }
        if !ids.contains(&self.baseline_member) {
            return Err("Diagnostic baseline member absent".into());
        }
        Ok(())
    }
    pub fn digest(&self) -> Result<[u8; 32], String> {
        self.validate()?;
        Ok(Sha256::digest(serde_json::to_vec(self).map_err(|e| e.to_string())?).into())
    }
    pub fn save_new(&self, path: &Path) -> Result<(), String> {
        let envelope = Envelope {
            schema_version: 1,
            kind: ArtifactKind::DiagnosticRunSet,
            sha256: self.digest()?,
            run_set: self.clone(),
        };
        publish_new(
            path,
            &serde_json::to_vec(&envelope).map_err(|e| e.to_string())?,
            MAX_DOCUMENT,
        )
        .map_err(|e| e.to_string())
    }
    pub fn open(path: &Path) -> Result<Self, String> {
        let e: Envelope = read_versioned(path, 1, MAX_DOCUMENT).map_err(|e| e.to_string())?;
        if e.sha256 != e.run_set.digest()? {
            return Err("Diagnostic run-set digest differs".into());
        }
        Ok(e.run_set)
    }
    pub fn open_member(&self, root: &Path, index: usize) -> MemberState {
        let result = (|| -> Result<MemberState, String> {
            self.validate()?;
            let m = self.members.get(index).ok_or("Diagnostic member index")?;
            let path = match archive_path(root, &m.archive_file) {
                Ok(v) => v,
                Err(crate::run_sets::RunSetError::Io(e))
                    if e.kind() == std::io::ErrorKind::NotFound =>
                {
                    return Ok(if m.observed.is_none() {
                        MemberState::NotRun
                    } else {
                        MemberState::Missing
                    });
                }
                Err(e) => return Err(e.to_string()),
            };
            if let Some(pin) = &m.finalized
                && file_digest(&path, m.spec.recording.max_database_bytes)
                    .map_err(|e| e.to_string())?
                    != pin.sha256
            {
                return Err("Pinned diagnostic artifact changed".into());
            }
            let reader = DiagnosticReader::open(&path, m.spec.recording.limits())
                .map_err(|e| e.to_string())?;
            reader.verify().map_err(|e| e.to_string())?;
            m.spec.verify_header(reader.run())?;
            if reader.run().run_id != m.run_id
                || reader.recorded_limits() != m.spec.recording.limits()
            {
                return Err("Diagnostic run identity/budgets differ".into());
            }
            let count = reader.count().map_err(|e| e.to_string())?;
            let status = reader.status().map_err(|e| e.to_string())?;
            if let Some(pin) = &m.finalized {
                let status = status.ok_or("Pinned diagnostic archive is incomplete")?;
                if count != pin.recorded_count || status.outcome != pin.terminal.recording() {
                    return Err("Diagnostic terminal pin differs".into());
                }
                let observed = m
                    .observed
                    .as_ref()
                    .ok_or("Diagnostic observations absent")?;
                let stats = &observed.statistics;
                if status.produced_attempts != stats.produced_attempts
                    || status.last_advanced_generation != stats.last_advanced_generation
                    || status.detail != observed.detail
                    || reader.run().native_prefix_steps != stats.native_prefix_steps
                {
                    return Err("Cold diagnostic observations differ from archive".into());
                }
                let (mut dispatched, mut qualified, mut rejected, mut unavailable) = (0, 0, 0, 0);
                for index in 0..count {
                    let attempt = reader.select(index).map_err(|e| e.to_string())?;
                    dispatched += usize::from(attempt.dispatched);
                    match attempt.outcome {
                        PredictionOutcome::Candidate(sample) if sample.observation.qualified => {
                            qualified += 1
                        }
                        PredictionOutcome::Candidate(_) => rejected += 1,
                        PredictionOutcome::NoCandidate { .. } => unavailable += 1,
                    }
                }
                let tail = stats.produced_attempts - count;
                if [
                    (dispatched, stats.dispatched_calls),
                    (qualified, stats.qualified),
                    (rejected, stats.rejected),
                    (unavailable, stats.unavailable),
                ]
                .iter()
                .any(|(recorded, observed)| observed < recorded || observed - recorded > tail)
                {
                    return Err("Diagnostic category counts differ from committed attempts".into());
                }
                if status.outcome == TerminalOutcome::Completed
                    && (reader.run().native_prefix_steps != m.spec.scene.warmup_steps
                        || count != m.spec.scene.steps - m.spec.scene.warmup_steps
                        || status.last_advanced_generation != Some(m.spec.scene.steps as u64))
                {
                    return Err("Completed diagnostic horizon differs from intent".into());
                }
                if file_digest(&path, m.spec.recording.max_database_bytes)
                    .map_err(|e| e.to_string())?
                    != pin.sha256
                {
                    return Err("Diagnostic source changed during cold verification".into());
                }
                Ok(MemberState::Ready(reader))
            } else if status.is_some() {
                Ok(MemberState::Unpinned(reader))
            } else {
                Ok(MemberState::Incomplete(reader))
            }
        })();
        result.unwrap_or_else(MemberState::Invalid)
    }
    pub fn pin_closed(
        &mut self,
        root: &Path,
        index: usize,
        observed: RunObservation,
    ) -> Result<(), String> {
        observed.statistics.validate()?;
        let m = self
            .members
            .get_mut(index)
            .ok_or("Diagnostic member index")?;
        m.observed = Some(observed);
        let path = archive_path(root, &m.archive_file).map_err(|e| e.to_string())?;
        let reader =
            DiagnosticReader::open(&path, m.spec.recording.limits()).map_err(|e| e.to_string())?;
        reader.verify().map_err(|e| e.to_string())?;
        m.spec.verify_header(reader.run())?;
        let status = reader
            .status()
            .map_err(|e| e.to_string())?
            .ok_or("Diagnostic terminal is unconfirmed")?;
        let o = m.observed.as_ref().unwrap();
        if status.outcome != o.terminal.recording()
            || status.produced_attempts != o.statistics.produced_attempts
            || status.last_advanced_generation != o.statistics.last_advanced_generation
            || reader.count().map_err(|e| e.to_string())? != o.statistics.recorded_attempts
        {
            return Err("Diagnostic observations differ from closed archive".into());
        }
        drop(reader);
        m.finalized = Some(ArchivePin {
            sha256: file_digest(&path, m.spec.recording.max_database_bytes)
                .map_err(|e| e.to_string())?,
            recorded_count: o.statistics.recorded_attempts,
            terminal: o.terminal,
        });
        match self.open_member(root, index) {
            MemberState::Ready(_) => Ok(()),
            _ => {
                self.members[index].finalized = None;
                Err("Diagnostic pin failed cold verification".into())
            }
        }
    }
}
