//! Immutable application intent and bounded manifests over existing numerical archives.
//! Native sealed-box carrier v1 only. This module never runs a worker or model,
//! parses Debug provenance, appends numerical data, or restores integrator state.
//! Opening inherits RecordingReader's SQLite hot-journal recovery contract.
#[path = "run_sets/configured.rs"]
pub mod configured;
#[path = "run_sets/source.rs"]
pub mod source;

use rheon::{
    GridGeometry, PressureImplementation, PressureSettings, SessionIdentity, Simulation,
    SimulationConfig, SmokeSource, SnapshotStage,
    recording::{
        RecordedCarrier, RecordingError, RecordingLimits, RecordingReader, RecordingRun,
        RecordingStatus, carrier_run,
    },
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::{
    fmt,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Component, Path},
    sync::atomic::{AtomicU64, Ordering},
};

const MANIFEST_VERSION: u32 = 1;
const FORCE_SPEC_VERSION: u32 = 2;
const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;
const MAX_MEMBERS: usize = 64;
const MAX_TEXT_BYTES: usize = 1000;

#[derive(Debug)]
pub enum RunSetError {
    Io(std::io::Error),
    Codec(serde_json::Error),
    Invalid(&'static str),
    Unsupported(&'static str),
    Integrity(&'static str),
    Recording(RecordingError),
    /// Publication may already be visible. Preserve it; reopen to reconcile.
    PublicationUncertain(std::io::Error),
}
impl fmt::Display for RunSetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "saved run set I/O: {error}"),
            Self::Codec(error) => write!(f, "saved run set codec: {error}"),
            Self::Invalid(reason) => write!(f, "invalid saved run set: {reason}"),
            Self::Unsupported(reason) => write!(f, "unsupported saved run set: {reason}"),
            Self::Integrity(reason) => write!(f, "saved run set integrity: {reason}"),
            Self::Recording(error) => write!(f, "saved run set archive: {error}"),
            Self::PublicationUncertain(error) => {
                write!(f, "saved run set publication uncertain: {error}")
            }
        }
    }
}
impl std::error::Error for RunSetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) | Self::PublicationUncertain(error) => Some(error),
            Self::Codec(error) => Some(error),
            Self::Recording(error) => Some(error),
            _ => None,
        }
    }
}
impl From<std::io::Error> for RunSetError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
impl From<serde_json::Error> for RunSetError {
    fn from(value: serde_json::Error) -> Self {
        Self::Codec(value)
    }
}
impl From<RecordingError> for RunSetError {
    fn from(value: RecordingError) -> Self {
        Self::Recording(value)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NativePressureMethod {
    JacobiPcgV1,
    SymmetricGaussSeidelPcgV1,
}
impl NativePressureMethod {
    pub fn implementation(self) -> PressureImplementation {
        match self {
            Self::JacobiPcgV1 => PressureImplementation::JacobiPcgV1,
            Self::SymmetricGaussSeidelPcgV1 => PressureImplementation::SymmetricGaussSeidelPcgV1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SourceSpec {
    pub lower_m: [f64; 3],
    pub upper_m: [f64; 3],
    pub tracer_rate_per_s: f64,
    pub vertical_acceleration_m_s2: f64,
    /// Source is applied for exactly this many requested intervals from zero.
    pub active_steps: usize,
}
impl SourceSpec {
    fn source(&self) -> SmokeSource {
        SmokeSource {
            lower: self.lower_m,
            upper: self.upper_m,
            tracer_rate: self.tracer_rate_per_s,
            vertical_acceleration: self.vertical_acceleration_m_s2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NativeTolerances {
    pub relative_residual: f64,
    pub absolute_residual_m3_s2: f64,
    pub predicted_divergence_per_s: f64,
    pub actual_divergence_per_s: f64,
    pub max_courant: f64,
    pub max_pressure_iterations: usize,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ArchiveBudget {
    pub max_database_bytes: u64,
    pub max_record_bytes: usize,
    pub max_records: usize,
}
impl ArchiveBudget {
    pub fn limits(self) -> RecordingLimits {
        RecordingLimits {
            max_database_bytes: self.max_database_bytes,
            max_record_bytes: self.max_record_bytes,
            max_records: self.max_records,
        }
    }
}

/// Runnable constructor-zero, sealed-box native carrier intent. No external
/// initial assets, viscosity, obstacle, adaptive dt, emitter or provider implied.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NativeCarrierSpec {
    pub counts_xyz: [u64; 3],
    pub spacing_m: [f64; 3],
    pub origin_m: [f64; 3],
    pub density_kg_m3: f64,
    pub pressure_method: NativePressureMethod,
    pub tolerances: NativeTolerances,
    pub requested_dt_s: f64,
    pub requested_steps: usize,
    pub source: Option<SourceSpec>,
    pub simulation_memory_limit_bytes: usize,
    pub recording: ArchiveBudget,
}
impl NativeCarrierSpec {
    pub fn validate(&self) -> Result<(), RunSetError> {
        let grid = GridGeometry::new(self.counts_xyz, self.spacing_m, self.origin_m)
            .map_err(|_| RunSetError::Invalid("spec geometry"))?;
        self.recording.limits().validate()?;
        let t = &self.tolerances;
        if !self.density_kg_m3.is_finite()
            || self.density_kg_m3 <= 0.0
            || !self.requested_dt_s.is_finite()
            || self.requested_dt_s <= 0.0
            || self.requested_steps == 0
            || self.requested_steps > 10000
            || !(self.requested_dt_s * self.requested_steps as f64).is_finite()
            || self.simulation_memory_limit_bytes == 0
            || [
                t.relative_residual,
                t.absolute_residual_m3_s2,
                t.predicted_divergence_per_s,
                t.actual_divergence_per_s,
            ]
            .iter()
            .any(|x| !x.is_finite() || *x < 0.0)
            || !t.max_courant.is_finite()
            || t.max_courant <= 0.0
            || t.max_courant > 1.0
            || t.max_pressure_iterations == 0
        {
            return Err(RunSetError::Invalid("spec controls"));
        }
        if let Some(s) = &self.source
            && (s.active_steps > self.requested_steps
                || !s.tracer_rate_per_s.is_finite()
                || s.tracer_rate_per_s < 0.0
                || !s.vertical_acceleration_m_s2.is_finite()
                || (0..3).any(|i| {
                    !s.lower_m[i].is_finite()
                        || !s.upper_m[i].is_finite()
                        || s.lower_m[i] >= s.upper_m[i]
                        || s.lower_m[i] < grid.origin()[i]
                        || s.upper_m[i] > grid.upper()[i]
                }))
        {
            return Err(RunSetError::Invalid("spec source"));
        }
        Ok(())
    }
    pub fn simulation_config(&self) -> SimulationConfig {
        let t = &self.tolerances;
        SimulationConfig {
            density: self.density_kg_m3,
            memory_limit: self.simulation_memory_limit_bytes,
            pressure: PressureSettings {
                relative_residual: t.relative_residual,
                absolute_residual: t.absolute_residual_m3_s2,
                divergence_limit: t.predicted_divergence_per_s,
                max_iterations: t.max_pressure_iterations,
            },
            actual_divergence_limit: t.actual_divergence_per_s,
            max_courant: t.max_courant,
        }
    }
    /// Existing numerical owners make final allocation and physics admission.
    pub fn create_simulation(&self) -> Result<Simulation, RunSetError> {
        self.validate()?;
        let grid = GridGeometry::new(self.counts_xyz, self.spacing_m, self.origin_m)
            .map_err(|_| RunSetError::Invalid("spec geometry"))?;
        Simulation::with_implementation(
            grid,
            self.simulation_config(),
            self.pressure_method.implementation(),
        )
        .map_err(|_| RunSetError::Invalid("numerical constructor rejected spec"))
    }
    /// Zero-based requested interval index; history is owned by Simulation.
    pub fn source_at(&self, interval: usize) -> Option<SmokeSource> {
        self.source
            .as_ref()
            .filter(|s| interval < s.active_steps)
            .map(SourceSpec::source)
    }
}

/// Digest binds a validated typed payload, never a mutable path or Debug text.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FrozenSpec {
    pub schema_version: u32,
    pub sha256: [u8; 32],
    pub spec: NativeCarrierSpec,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_force_scene"
    )]
    pub force_fields: Option<rheon::ForceScene>,
}
impl FrozenSpec {
    /// Freeze the existing native producer's actual constructor configuration.
    /// The worker retains its loop and supplies only its explicit future time
    /// and source schedule; accepted state is never interpreted as a checkpoint.
    pub fn capture_constructor(
        simulation: &Simulation,
        requested_dt_s: f64,
        requested_steps: usize,
        source: Option<SourceSpec>,
        recording: RecordingLimits,
    ) -> Result<Self, RunSetError> {
        if simulation.state().generation != 0 || simulation.state().time != 0.0 {
            return Err(RunSetError::Invalid("native intent requires constructor"));
        }
        let actual = simulation.config();
        let pressure_method = match simulation.implementation() {
            PressureImplementation::JacobiPcgV1 => NativePressureMethod::JacobiPcgV1,
            PressureImplementation::SymmetricGaussSeidelPcgV1 => {
                NativePressureMethod::SymmetricGaussSeidelPcgV1
            }
        };
        Self::new(NativeCarrierSpec {
            counts_xyz: simulation.grid().counts().map(|value| value as u64),
            spacing_m: simulation.grid().spacing(),
            origin_m: simulation.grid().origin(),
            density_kg_m3: actual.density,
            pressure_method,
            tolerances: NativeTolerances {
                relative_residual: actual.pressure.relative_residual,
                absolute_residual_m3_s2: actual.pressure.absolute_residual,
                predicted_divergence_per_s: actual.pressure.divergence_limit,
                actual_divergence_per_s: actual.actual_divergence_limit,
                max_courant: actual.max_courant,
                max_pressure_iterations: actual.pressure.max_iterations,
            },
            requested_dt_s,
            requested_steps,
            source,
            simulation_memory_limit_bytes: actual.memory_limit,
            recording: ArchiveBudget {
                max_database_bytes: recording.max_database_bytes,
                max_record_bytes: recording.max_record_bytes,
                max_records: recording.max_records,
            },
        })
    }
    pub fn new(spec: NativeCarrierSpec) -> Result<Self, RunSetError> {
        spec.validate()?;
        Ok(Self {
            schema_version: MANIFEST_VERSION,
            sha256: digest(&spec)?,
            spec,
            force_fields: None,
        })
    }
    pub fn with_force_fields(mut self, fields: &rheon::ForceSnapshot) -> Result<Self, RunSetError> {
        self.validate()?;
        if self.force_fields.is_some() {
            return Err(RunSetError::Invalid("force intent already bound"));
        }
        self.schema_version = FORCE_SPEC_VERSION;
        self.force_fields = Some(fields.scene().clone());
        self.sha256 = force_spec_digest(&self.spec, fields)?;
        Ok(self)
    }
    pub fn force_snapshot(&self) -> Result<Option<rheon::ForceSnapshot>, RunSetError> {
        self.force_fields
            .as_ref()
            .map(|scene| {
                scene
                    .freeze()
                    .map_err(|_| RunSetError::Invalid("frozen force document"))
            })
            .transpose()
    }
    pub fn validate(&self) -> Result<(), RunSetError> {
        self.spec.validate()?;
        let expected = match (self.schema_version, self.force_fields.as_ref()) {
            (MANIFEST_VERSION, None) => digest(&self.spec)?,
            (FORCE_SPEC_VERSION, Some(_)) => force_spec_digest(
                &self.spec,
                &self
                    .force_snapshot()?
                    .ok_or(RunSetError::Invalid("force intent missing"))?,
            )?,
            (MANIFEST_VERSION | FORCE_SPEC_VERSION, _) => {
                return Err(RunSetError::Invalid("frozen intent version/payload"));
            }
            _ => return Err(RunSetError::Unsupported("frozen native intent version")),
        };
        if self.sha256 != expected {
            return Err(RunSetError::Integrity("spec digest"));
        }
        Ok(())
    }
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, RunSetError> {
        if bytes.len() as u64 > MAX_MANIFEST_BYTES {
            return Err(RunSetError::Invalid("manifest byte limit"));
        }
        let result: Self = serde_json::from_slice(bytes)?;
        result.validate()?;
        Ok(result)
    }
    pub fn save_new(&self, path: &Path) -> Result<(), RunSetError> {
        self.validate()?;
        publish_new(path, &serde_json::to_vec(self)?, MAX_MANIFEST_BYTES)
    }
    pub fn open(path: &Path) -> Result<Self, RunSetError> {
        Self::from_bytes(&read_document(path, MAX_MANIFEST_BYTES)?)
    }
}

fn deserialize_force_scene<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<rheon::ForceScene>, D::Error> {
    rheon::ForceScene::deserialize(deserializer).map(Some)
}
fn force_spec_digest(
    spec: &NativeCarrierSpec,
    fields: &rheon::ForceSnapshot,
) -> Result<[u8; 32], RunSetError> {
    let spec = serde_json::to_vec(spec)?;
    let fields = rheon::recording::encode_force_metadata(fields)?;
    let mut hash = Sha256::new();
    hash.update(b"rheon-desktop-native-force-spec-v2\0");
    hash.update((spec.len() as u64).to_le_bytes());
    hash.update(spec);
    hash.update((fields.len() as u64).to_le_bytes());
    hash.update(fields);
    Ok(hash.finalize().into())
}
fn force_bytes(spec: &FrozenSpec) -> Result<Option<Vec<u8>>, RunSetError> {
    spec.force_snapshot()?
        .as_ref()
        .map(rheon::recording::encode_force_metadata)
        .transpose()
        .map_err(Into::into)
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeIntentDeclaration {
    schema_version: u32,
    spec_sha256: [u8; 32],
    build_provenance: String,
}
/// Attach the complete immutable intent identity to the existing archive's
/// provenance, after checking actual constructor configuration. This is a
/// producer declaration, not authentication or proof of an external execution.
/// The composer must drive dt and source schedules from this same frozen spec.
pub fn bound_native_run(
    spec: &FrozenSpec,
    simulation: &Simulation,
    identity: SessionIdentity,
    build_provenance: &str,
) -> Result<RecordingRun, RunSetError> {
    spec.validate()?;
    text(build_provenance)?;
    let actual = simulation.config();
    let mut observed = spec.spec.clone();
    observed.counts_xyz = simulation.grid().counts().map(|n| n as u64);
    observed.spacing_m = simulation.grid().spacing();
    observed.origin_m = simulation.grid().origin();
    observed.density_kg_m3 = actual.density;
    observed.simulation_memory_limit_bytes = actual.memory_limit;
    observed.tolerances = NativeTolerances {
        relative_residual: actual.pressure.relative_residual,
        absolute_residual_m3_s2: actual.pressure.absolute_residual,
        predicted_divergence_per_s: actual.pressure.divergence_limit,
        actual_divergence_per_s: actual.actual_divergence_limit,
        max_courant: actual.max_courant,
        max_pressure_iterations: actual.pressure.max_iterations,
    };
    if observed != spec.spec
        || simulation.implementation() != spec.spec.pressure_method.implementation()
        || simulation.state().generation != 0
        || simulation.state().time != 0.0
    {
        return Err(RunSetError::Invalid("native constructor/spec binding"));
    }
    let provenance = serde_json::to_string(&NativeIntentDeclaration {
        schema_version: spec.schema_version,
        spec_sha256: spec.sha256,
        build_provenance: build_provenance.into(),
    })?;
    let run = carrier_run(identity, simulation, &provenance)?;
    Ok(match spec.force_snapshot()? {
        Some(fields) => run.with_force_fields(&fields)?,
        None => run,
    })
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TerminalOutcome {
    Completed,
    Cancelled,
    Failed,
}
impl TerminalOutcome {
    fn from_status(status: &str) -> Result<Self, RunSetError> {
        match status {
            "completed" => Ok(Self::Completed),
            "cancelled" => Ok(Self::Cancelled),
            "failed" => Ok(Self::Failed),
            _ => Err(RunSetError::Invalid("archive is not terminal")),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FinalizedArchive {
    pub sha256: [u8; 32],
    pub recorded_count: usize,
    pub outcome: TerminalOutcome,
    pub accepted_generation: Option<u64>,
}
/// A retry is a new attempt ID and fresh archive. A reference with no final pin
/// is inspectable as missing/incomplete/unpinned, never silently promoted ready.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RunAttempt {
    pub attempt_id: String,
    pub archive_relative_path: String,
    pub expected_run_id: u64,
    pub expected_source_revision: String,
    pub finalized: Option<FinalizedArchive>,
}
impl RunAttempt {
    fn validate(&self) -> Result<(), RunSetError> {
        text(&self.attempt_id)?;
        text(&self.expected_source_revision)?;
        relative_path(&self.archive_relative_path)?;
        Ok(())
    }
    /// Call only after closing the producer/writer. Local protected paths and
    /// immutable terminal artifacts are required throughout capture and reopen.
    pub fn pin_closed(&mut self, root: &Path, spec: &FrozenSpec) -> Result<(), RunSetError> {
        self.validate()?;
        spec.validate()?;
        if self.finalized.is_some() {
            return Err(RunSetError::Invalid("attempt already pinned"));
        }
        let path = archive_path(root, &self.archive_relative_path)?;
        let reader = RecordingReader::open(&path, spec.spec.recording.limits())?;
        validate_binding(&reader, self, spec)?;
        let status = reader.status()?;
        let outcome = TerminalOutcome::from_status(&status.outcome)?;
        reader.verify()?;
        validate_native_publications(&reader, &spec.spec, outcome)?;
        let recorded_count = reader.count()?;
        drop(reader);
        self.finalized = Some(FinalizedArchive {
            sha256: file_digest(&path, spec.spec.recording.max_database_bytes)?,
            recorded_count,
            outcome,
            accepted_generation: status.accepted_generation,
        });
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpecDifference {
    Geometry,
    Density,
    PressureMethod,
    Tolerances,
    TimePolicy,
    Source,
    MemoryBudget,
    RecordingBudget,
    ForceMetadata,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RunSetMember {
    pub member_id: String,
    pub spec: FrozenSpec,
    pub differences_from_baseline: Vec<SpecDifference>,
    pub attempt: Option<RunAttempt>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RunSetRevision {
    pub schema_version: u32,
    pub set_id: String,
    /// Optional exact parent content identity; there is no mutable latest alias.
    pub parent_sha256: Option<[u8; 32]>,
    pub baseline_member: String,
    pub members: Vec<RunSetMember>,
}
impl RunSetRevision {
    pub fn validate(&self) -> Result<(), RunSetError> {
        version(self.schema_version)?;
        text(&self.set_id)?;
        text(&self.baseline_member)?;
        if self.members.is_empty() || self.members.len() > MAX_MEMBERS {
            return Err(RunSetError::Invalid("member count"));
        }
        let baseline = self
            .members
            .iter()
            .find(|m| m.member_id == self.baseline_member)
            .ok_or(RunSetError::Invalid("baseline missing"))?;
        for (i, m) in self.members.iter().enumerate() {
            text(&m.member_id)?;
            m.spec.validate()?;
            if self.members[..i]
                .iter()
                .any(|other| other.member_id == m.member_id)
            {
                return Err(RunSetError::Invalid("duplicate member"));
            }
            let mut declared = differences(&baseline.spec.spec, &m.spec.spec);
            if force_bytes(&baseline.spec)? != force_bytes(&m.spec)? {
                declared.push(SpecDifference::ForceMetadata);
            }
            if declared != m.differences_from_baseline {
                return Err(RunSetError::Invalid("declared differences"));
            }
            if let Some(a) = &m.attempt {
                a.validate()?;
                if self.members[..i]
                    .iter()
                    .filter_map(|m| m.attempt.as_ref())
                    .any(|other| {
                        other.attempt_id == a.attempt_id
                            || other.archive_relative_path == a.archive_relative_path
                    })
                {
                    return Err(RunSetError::Invalid("duplicate attempt or archive"));
                }
            }
        }
        Ok(())
    }
    pub fn sha256(&self) -> Result<[u8; 32], RunSetError> {
        self.validate()?;
        digest(self)
    }
    pub fn save_new(&self, path: &Path) -> Result<(), RunSetError> {
        self.validate()?;
        let envelope = Manifest {
            schema_version: MANIFEST_VERSION,
            sha256: digest(self)?,
            revision: self.clone(),
        };
        publish_new(path, &serde_json::to_vec(&envelope)?, MAX_MANIFEST_BYTES)
    }
    pub fn open(path: &Path) -> Result<Self, RunSetError> {
        let envelope: Manifest = read_versioned(path, MANIFEST_VERSION, MAX_MANIFEST_BYTES)?;
        envelope.revision.validate()?;
        if envelope.sha256 != digest(&envelope.revision)? {
            return Err(RunSetError::Integrity("run-set digest"));
        }
        Ok(envelope.revision)
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema_version: u32,
    sha256: [u8; 32],
    revision: RunSetRevision,
}

/// No numerical data is returned for a missing/unpinned/incomplete member.
pub enum MemberState {
    NotRun,
    Missing,
    Incomplete {
        recorded_count: usize,
        status: RecordingStatus,
    },
    Unpinned {
        recorded_count: usize,
        status: RecordingStatus,
    },
    Unsupported(&'static str),
    Invalid(RunSetError),
    Ready(Box<RecordingReader>),
}
/// Explicit resolution from this immutable member only. No live/latest lookup,
/// model loading, missing-field substitution, migration, deletion or execution.
pub fn open_member(root: &Path, member: &RunSetMember) -> MemberState {
    match open_member_inner(root, member) {
        Ok(state) => state,
        Err(RunSetError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => MemberState::Missing,
        Err(RunSetError::Recording(RecordingError::Io(e)))
            if e.kind() == std::io::ErrorKind::NotFound =>
        {
            MemberState::Missing
        }
        Err(
            RunSetError::Unsupported(reason)
            | RunSetError::Recording(RecordingError::Unsupported(reason)),
        ) => MemberState::Unsupported(reason),
        Err(error) => MemberState::Invalid(error),
    }
}
fn open_member_inner(root: &Path, m: &RunSetMember) -> Result<MemberState, RunSetError> {
    m.spec.validate()?;
    let Some(a) = &m.attempt else {
        return Ok(MemberState::NotRun);
    };
    a.validate()?;
    let path = archive_path(root, &a.archive_relative_path)?;
    let reader = RecordingReader::open(&path, m.spec.spec.recording.limits())?;
    validate_binding(&reader, a, &m.spec)?;
    let status = reader.status()?;
    let count = reader.count()?;
    let Some(pin) = &a.finalized else {
        return Ok(if status.outcome == "active" {
            MemberState::Incomplete {
                recorded_count: count,
                status,
            }
        } else {
            MemberState::Unpinned {
                recorded_count: count,
                status,
            }
        });
    };
    if status.outcome == "active" {
        return Err(RunSetError::Integrity("pinned archive became active"));
    }
    if pin.recorded_count != count
        || pin.outcome != TerminalOutcome::from_status(&status.outcome)?
        || pin.accepted_generation != status.accepted_generation
        || pin.sha256 != file_digest(&path, m.spec.spec.recording.max_database_bytes)?
    {
        return Err(RunSetError::Integrity("finalized archive changed"));
    }
    reader.verify()?;
    validate_native_publications(&reader, &m.spec.spec, pin.outcome)?;
    Ok(MemberState::Ready(Box::new(reader)))
}
fn validate_binding(
    reader: &RecordingReader,
    a: &RunAttempt,
    spec: &FrozenSpec,
) -> Result<(), RunSetError> {
    let s = &spec.spec;
    let run = reader.run();
    let actual_limits = reader.recorded_limits();
    let intended_limits = s.recording.limits();
    if actual_limits != intended_limits {
        return Err(RunSetError::Integrity("archive recording budget"));
    }
    if !run.solver.starts_with("rheon-carrier-full-box-v1/") {
        return Err(RunSetError::Unsupported("saved native carrier profile"));
    }
    let declaration: NativeIntentDeclaration = serde_json::from_str(&run.provenance)?;
    if declaration.schema_version != spec.schema_version {
        return Err(RunSetError::Integrity("archive intent version"));
    }
    let expected_force = force_bytes(spec)?.map(|bytes| <[u8; 32]>::from(Sha256::digest(bytes)));
    if run.force_fields_digest()? != expected_force {
        return Err(RunSetError::Integrity("archive force intent"));
    }
    text(&declaration.build_provenance)?;
    if run.identity.run_id != a.expected_run_id
        || run.identity.source_revision != a.expected_source_revision
        || run.counts != s.counts_xyz
        || run.spacing.map(f64::to_bits) != s.spacing_m.map(f64::to_bits)
        || run.origin.map(f64::to_bits) != s.origin_m.map(f64::to_bits)
        || run.solver.rsplit('/').next() != Some(s.pressure_method.implementation().id())
        || declaration.spec_sha256 != spec.sha256
    {
        return Err(RunSetError::Integrity("archive/spec binding"));
    }
    Ok(())
}
fn validate_native_publications(
    reader: &RecordingReader,
    spec: &NativeCarrierSpec,
    outcome: TerminalOutcome,
) -> Result<(), RunSetError> {
    let count = reader.count()?;
    if count > spec.requested_steps + 1
        || reader
            .status()?
            .accepted_generation
            .is_some_and(|generation| generation > spec.requested_steps as u64)
    {
        return Err(RunSetError::Integrity("native publication horizon"));
    }
    for ordinal in 0..count {
        let step = reader.select(ordinal)?;
        RecordedCarrier::decode(reader.run(), &step)?;
        if ordinal > 0 && step.dt.map(f64::to_bits) != Some(spec.requested_dt_s.to_bits()) {
            return Err(RunSetError::Unsupported(
                "actual dt differs from fixed native intent; inspect retained archive directly",
            ));
        }
        if step.generation != ordinal as u64
            || (ordinal == 0 && step.stage != SnapshotStage::Constructor)
            || (ordinal > 0 && step.stage != SnapshotStage::Accepted)
        {
            return Err(RunSetError::Integrity("native publication schedule"));
        }
    }
    if outcome == TerminalOutcome::Completed && count != spec.requested_steps + 1 {
        return Err(RunSetError::Integrity("completed native horizon"));
    }
    Ok(())
}
fn differences(a: &NativeCarrierSpec, b: &NativeCarrierSpec) -> Vec<SpecDifference> {
    use SpecDifference::*;
    let mut result = Vec::new();
    for (different, kind) in [
        (
            a.counts_xyz != b.counts_xyz || a.spacing_m != b.spacing_m || a.origin_m != b.origin_m,
            Geometry,
        ),
        (a.density_kg_m3 != b.density_kg_m3, Density),
        (a.pressure_method != b.pressure_method, PressureMethod),
        (a.tolerances != b.tolerances, Tolerances),
        (
            a.requested_dt_s != b.requested_dt_s || a.requested_steps != b.requested_steps,
            TimePolicy,
        ),
        (a.source != b.source, Source),
        (
            a.simulation_memory_limit_bytes != b.simulation_memory_limit_bytes,
            MemoryBudget,
        ),
        (a.recording != b.recording, RecordingBudget),
    ] {
        if different {
            result.push(kind);
        }
    }
    result
}
fn version(value: u32) -> Result<(), RunSetError> {
    if value != MANIFEST_VERSION {
        Err(RunSetError::Unsupported("manifest version"))
    } else {
        Ok(())
    }
}
fn text(value: &str) -> Result<(), RunSetError> {
    if value.is_empty() || value.len() > MAX_TEXT_BYTES || value.chars().any(char::is_control) {
        Err(RunSetError::Invalid("bounded text"))
    } else {
        Ok(())
    }
}
pub(crate) fn relative_path(value: &str) -> Result<(), RunSetError> {
    text(value)?;
    if value
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
        || value.contains('\\')
        || value.contains(':')
        || !Path::new(value)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
    {
        return Err(RunSetError::Invalid("archive relative path"));
    }
    Ok(())
}
pub(crate) fn archive_path(root: &Path, relative: &str) -> Result<std::path::PathBuf, RunSetError> {
    relative_path(relative)?;
    let mut path = root.to_owned();
    for component in Path::new(relative).components() {
        path.push(component);
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            return Err(RunSetError::Invalid("symlink archive component"));
        }
    }
    Ok(path)
}
fn digest<T: Serialize>(value: &T) -> Result<[u8; 32], RunSetError> {
    Ok(Sha256::digest(serde_json::to_vec(value)?).into())
}
pub(crate) fn file_digest(path: &Path, max_bytes: u64) -> Result<[u8; 32], RunSetError> {
    if max_bytes == 0 || max_bytes == u64::MAX {
        return Err(RunSetError::Invalid("document byte limit"));
    }
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(RunSetError::Invalid("regular file required"));
    }
    if metadata.len() > max_bytes {
        return Err(RunSetError::Invalid("file byte limit"));
    }
    let mut input = File::open(path)?.take(max_bytes + 1);
    let mut buffer = [0; 64 * 1024];
    let mut n = 0u64;
    let mut hash = Sha256::new();
    loop {
        let read = input.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        n += read as u64;
        if n > max_bytes {
            return Err(RunSetError::Invalid("file byte limit"));
        }
        hash.update(&buffer[..read]);
    }
    Ok(hash.finalize().into())
}
pub(crate) fn read_versioned<T: DeserializeOwned>(
    path: &Path,
    expected_version: u32,
    max_bytes: u64,
) -> Result<T, RunSetError> {
    let bytes = read_document(path, max_bytes)?;
    #[derive(Deserialize)]
    struct Header {
        schema_version: u64,
    }
    let header: Header = serde_json::from_slice(&bytes)?;
    if header.schema_version != u64::from(expected_version) {
        return Err(RunSetError::Unsupported("manifest version"));
    }
    // Decode the original bytes so duplicate required keys are not erased by a
    // generic JSON object representation before strict typed validation.
    Ok(serde_json::from_slice(&bytes)?)
}
fn read_document(path: &Path, max_bytes: u64) -> Result<Vec<u8>, RunSetError> {
    if max_bytes == 0 || max_bytes == u64::MAX {
        return Err(RunSetError::Invalid("document byte limit"));
    }
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(RunSetError::Invalid("regular manifest required"));
    }
    if metadata.len() > max_bytes {
        return Err(RunSetError::Invalid("manifest byte limit"));
    }
    let mut bytes = Vec::new();
    File::open(path)?
        .take(max_bytes + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max_bytes {
        return Err(RunSetError::Invalid("manifest byte limit"));
    }
    Ok(bytes)
}
/// Atomic no-replace publication on caller-owned local filesystems supporting
/// hard links and directory sync. Protect every path component from concurrent
/// replacement throughout use. Existing data and abandoned temporary files are
/// never overwritten as recovery. Interrupted staging is not authoritative.
pub(crate) fn publish_new(path: &Path, bytes: &[u8], max_bytes: u64) -> Result<(), RunSetError> {
    if max_bytes == 0 || max_bytes == u64::MAX {
        return Err(RunSetError::Invalid("document byte limit"));
    }
    if bytes.len() as u64 > max_bytes {
        return Err(RunSetError::Invalid("manifest byte limit"));
    }
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let directory = File::open(parent)?;
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let temporary = parent.join(format!(
        ".rheon-manifest-{}-{}.tmp",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let staged = file.write_all(bytes).and_then(|()| file.sync_all());
    drop(file);
    if let Err(error) = staged {
        return Err(RunSetError::Io(error));
    }
    fs::hard_link(&temporary, path)?;
    // Once linked, errors mean the durable publication outcome needs inspection.
    directory
        .sync_all()
        .map_err(RunSetError::PublicationUncertain)?;
    fs::remove_file(&temporary).map_err(RunSetError::PublicationUncertain)?;
    directory
        .sync_all()
        .map_err(RunSetError::PublicationUncertain)?;
    Ok(())
}
