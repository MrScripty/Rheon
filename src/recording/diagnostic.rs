//! Bounded diagnostic attempts, never accepted simulation publications.
//!
//! See `docs/api/diagnostic-recording.md`. Captured lineage and qualification
//! are producer observations. Integrity checks do not authenticate their author
//! or establish the physical correctness of the upstream step composition.
use super::{RecordingError, RecordingLimits, WriterState, bounded_text, connect, valid_text};
use crate::{
    GridGeometry,
    solver_provider::context::{
        PressureContext, PressureContextInput, PressureContextMode, PressureHistory,
        PressureHistoryStatus, STAGE_CONTRACT,
    },
};
use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Separate SQLite family; accepted archive version 1 is not this format.
pub const APPLICATION_ID: i64 = 0x52484450; // RHDP
pub const VERSION: i64 = 1;
const MAX_CELLS: usize = 65_536;
pub const MAX_NATIVE_PREFIX_STEPS: usize = 512;
const SCHEMA: &str = "
PRAGMA application_id=1380467792;
PRAGMA user_version=1;
CREATE TABLE diagnostic_run(id INTEGER PRIMARY KEY CHECK(id=1),run_id TEXT NOT NULL,
 revision TEXT NOT NULL,geometry BLOB NOT NULL,density BLOB NOT NULL,viscosity BLOB NOT NULL,
 dt BLOB NOT NULL,mode INTEGER NOT NULL,binding_id TEXT NOT NULL,binding_version TEXT NOT NULL,
 manifest BLOB NOT NULL,checkpoint BLOB,training_run TEXT,provenance TEXT NOT NULL,
 seed_generation BLOB,seed_end BLOB,seed_status INTEGER,seed_pressure BLOB,
 max_database_bytes INTEGER NOT NULL,max_record_bytes INTEGER NOT NULL,max_records INTEGER NOT NULL,
 stage TEXT NOT NULL,retention TEXT NOT NULL,digest BLOB NOT NULL,native_prefix_steps INTEGER NOT NULL);
CREATE TABLE diagnostic_lifecycle(id INTEGER PRIMARY KEY CHECK(id=1),outcome INTEGER NOT NULL,
 detail TEXT NOT NULL,produced_attempts INTEGER,advanced_generation BLOB,digest BLOB NOT NULL);
CREATE TABLE attempts(attempt_index INTEGER PRIMARY KEY,generation BLOB NOT NULL,
 start BLOB NOT NULL,dt BLOB NOT NULL,rhs BLOB NOT NULL,residual BLOB NOT NULL,
 prior_generation BLOB,prior_end BLOB,prior_status INTEGER,prior_pressure BLOB,
 dispatched INTEGER NOT NULL,outcome INTEGER NOT NULL,reason INTEGER,detail TEXT,
 invalid_payload_digest BLOB,transition INTEGER NOT NULL,request_digest BLOB NOT NULL,
 payload_bytes INTEGER NOT NULL,digest BLOB NOT NULL);
CREATE TABLE pressure_samples(attempt_index INTEGER NOT NULL REFERENCES attempts(attempt_index),
 role INTEGER NOT NULL,request_digest BLOB NOT NULL,pressure BLOB NOT NULL,policy INTEGER NOT NULL,
 residual_l2 BLOB NOT NULL,relative_residual BLOB,predicted_divergence BLOB NOT NULL,
 actual_divergence BLOB NOT NULL,courant BLOB NOT NULL,net_flux BLOB NOT NULL,
 relative_limit BLOB NOT NULL,absolute_limit BLOB NOT NULL,predicted_limit BLOB NOT NULL,
 actual_limit BLOB NOT NULL,courant_limit BLOB NOT NULL,flux_limit BLOB NOT NULL,
 qualified INTEGER NOT NULL,hard_stop INTEGER NOT NULL,history_status INTEGER,PRIMARY KEY(attempt_index,role));";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticMode {
    TeacherStage,
    IndependentStopOnReject,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BindingIdentity {
    pub id: String,
    pub version: String,
    pub manifest_sha256: [u8; 32],
    pub checkpoint_sha256: Option<[u8; 32]>,
    pub training_run_id: Option<String>,
}
/// An observed seed, not proof that preceding warmup steps were captured.
#[derive(Clone, Debug, PartialEq)]
pub struct ObservedHistory {
    pub generation: u64,
    pub interval_end: f64,
    pub status: PressureHistoryStatus,
    pub pressure: Vec<f64>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct DiagnosticRun {
    pub run_id: String,
    pub source_revision: String,
    pub grid: GridGeometry,
    pub density: f64,
    pub viscosity: f64,
    pub dt: f64,
    pub mode: DiagnosticMode,
    /// Caller-observed prefix length, not reconstructed from the seed generation.
    pub native_prefix_steps: usize,
    pub binding: BindingIdentity,
    pub provenance: String,
    pub seed: Option<ObservedHistory>,
}

/// Immutable physical request captured before a prediction consumes its input.
#[derive(Clone, Debug, PartialEq)]
pub struct DiagnosticRequest {
    run_id: String,
    grid: GridGeometry,
    density: f64,
    viscosity: f64,
    generation: u64,
    interval_start: f64,
    dt: f64,
    rhs: Vec<f64>,
    prior: Option<ObservedHistory>,
    initial_residual: Vec<f64>,
}
impl DiagnosticRequest {
    /// Captures only a diagnostic context. The byte cap is checked before copies
    /// or residual allocation; the recorder subsequently checks the whole attempt.
    pub fn capture(
        context: &PressureContext<'_>,
        viscosity: f64,
        payload_limit: usize,
    ) -> Result<Self, RecordingError> {
        let n = context.grid().cell_len();
        if context.mode() != PressureContextMode::DiagnosticCandidate
            || n > MAX_CELLS
            || !viscosity.is_finite()
            || viscosity < 0.0
        {
            return Err(RecordingError::Invalid("diagnostic request domain"));
        }
        let bytes = n
            .checked_mul(if context.prior().is_some() { 24 } else { 16 })
            .and_then(|v| v.checked_add(context.run_id().len() + 256))
            .ok_or(RecordingError::Limit("request bytes"))?;
        if bytes > payload_limit {
            return Err(RecordingError::Limit("request payload"));
        }
        let mut residual = vec![0.0; n];
        context
            .initial_residual(&mut residual)
            .map_err(|_| RecordingError::Invalid("initial residual"))?;
        Ok(Self {
            run_id: context.run_id().into(),
            grid: context.grid().clone(),
            density: context.density(),
            viscosity,
            generation: context.generation(),
            interval_start: context.interval_start(),
            dt: context.dt(),
            rhs: context.rhs().to_vec(),
            prior: context.prior().map(|p| ObservedHistory {
                generation: p.generation,
                interval_end: p.interval_end,
                status: p.status,
                pressure: p.values.to_vec(),
            }),
            initial_residual: residual,
        })
    }
    pub fn run_id(&self) -> &str {
        &self.run_id
    }
    pub fn grid(&self) -> &GridGeometry {
        &self.grid
    }
    pub fn density(&self) -> f64 {
        self.density
    }
    pub fn viscosity(&self) -> f64 {
        self.viscosity
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn interval_start(&self) -> f64 {
        self.interval_start
    }
    pub fn interval_end(&self) -> f64 {
        self.interval_start + self.dt
    }
    pub fn dt(&self) -> f64 {
        self.dt
    }
    pub fn rhs(&self) -> &[f64] {
        &self.rhs
    }
    pub fn prior(&self) -> Option<&ObservedHistory> {
        self.prior.as_ref()
    }
    pub fn initial_residual(&self) -> &[f64] {
        &self.initial_residual
    }
    pub fn digest(&self) -> [u8; 32] {
        let mut h = Hash::new(b"rheon-diagnostic-request-v1\0");
        self.hash(&mut h);
        h.finish()
    }
    fn context(&self) -> Result<PressureContext<'_>, RecordingError> {
        PressureContext::new(PressureContextInput {
            grid: &self.grid,
            density: self.density,
            dt: self.dt,
            interval_start: self.interval_start,
            rhs: &self.rhs,
            prior: self.prior.as_ref().map(|p| PressureHistory {
                values: &p.pressure,
                grid: &self.grid,
                density: self.density,
                run_id: &self.run_id,
                generation: p.generation,
                interval_end: p.interval_end,
                status: p.status,
            }),
            run_id: &self.run_id,
            generation: self.generation,
            mode: PressureContextMode::DiagnosticCandidate,
        })
        .map_err(|_| RecordingError::Invalid("request context"))
    }
    fn validate(&self, run: &DiagnosticRun) -> Result<(), RecordingError> {
        if self.run_id != run.run_id
            || !same_grid_bits(&self.grid, &run.grid)
            || self.density.to_bits() != run.density.to_bits()
            || self.viscosity.to_bits() != run.viscosity.to_bits()
            || self.dt.to_bits() != run.dt.to_bits()
        {
            return Err(RecordingError::Invalid("request differs from run domain"));
        }
        let context = self.context()?;
        let mut residual = vec![0.0; self.grid.cell_len()];
        context
            .initial_residual(&mut residual)
            .map_err(|_| RecordingError::Invalid("request residual"))?;
        if !same_values(&residual, &self.initial_residual) {
            return Err(RecordingError::Integrity("initial residual"));
        }
        Ok(())
    }
    fn hash(&self, h: &mut Hash) {
        h.text(STAGE_CONTRACT);
        h.text(&self.run_id);
        hash_grid(h, &self.grid);
        h.f(self.density);
        h.f(self.viscosity);
        h.u(self.generation);
        h.f(self.interval_start);
        h.f(self.dt);
        h.values(&self.rhs);
        hash_history(h, self.prior.as_ref());
        h.values(&self.initial_residual);
    }
}

/// Explicit host policy and observations, not a recording-owned numerical gate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QualificationPolicy {
    HostPressureGateV1,
}
#[derive(Clone, Debug, PartialEq)]
pub struct QualificationObservation {
    pub policy: QualificationPolicy,
    pub residual_l2: f64,
    pub relative_residual: Option<f64>,
    pub predicted_divergence: f64,
    pub actual_divergence: f64,
    pub courant: f64,
    pub net_flux: f64,
    pub relative_limit: f64,
    pub absolute_limit: f64,
    pub predicted_limit: f64,
    pub actual_limit: f64,
    pub courant_limit: f64,
    pub flux_limit: f64,
    pub qualified: bool,
    pub hard_stop: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PressureSample {
    pub pressure: Vec<f64>,
    pub observation: QualificationObservation,
}
#[derive(Clone, Debug, PartialEq)]
pub struct NativeReference {
    pub request_digest: [u8; 32],
    pub sample: PressureSample,
    /// The native owner's observed history status; recording does not grant it.
    pub history_status: PressureHistoryStatus,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoCandidateReason {
    Unsupported,
    UnsupportedContext,
    Cancelled,
    RuntimeFailure,
    InvalidOutput,
}
#[derive(Clone, Debug, PartialEq)]
pub enum PredictionOutcome {
    Candidate(PressureSample),
    NoCandidate {
        reason: NoCandidateReason,
        detail: String,
        invalid_payload_sha256: Option<[u8; 32]>,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoryTransition {
    NotAdvanced,
    DiagnosticAdvanced,
    NativeTeacherAdvanced,
}
#[derive(Clone, Debug, PartialEq)]
pub struct DiagnosticAttempt {
    /// A contiguous diagnostic index, unrelated to an accepted ordinal.
    pub attempt_index: usize,
    pub request: DiagnosticRequest,
    pub dispatched: bool,
    pub outcome: PredictionOutcome,
    pub native_reference: Option<NativeReference>,
    pub transition: HistoryTransition,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalOutcome {
    Completed,
    StoppedOnRejection,
    Cancelled,
    Failed,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticTerminal {
    pub outcome: TerminalOutcome,
    pub detail: String,
    /// Producer attempts can exceed the durable prefix after a recording error.
    pub produced_attempts: usize,
    pub last_advanced_generation: Option<u64>,
}

impl DiagnosticRun {
    fn validate(&self, limits: RecordingLimits) -> Result<(), RecordingError> {
        limits.validate()?;
        for text in [
            &self.run_id,
            &self.source_revision,
            &self.binding.id,
            &self.binding.version,
            &self.provenance,
        ] {
            valid_text(text)?;
        }
        if self.run_id.len() > 4096
            || self.run_id.chars().any(char::is_control)
            || !self.density.is_finite()
            || self.density <= 0.0
            || !self.viscosity.is_finite()
            || self.viscosity < 0.0
            || !self.dt.is_finite()
            || self.dt <= 0.0
            || self.grid.cell_len() > MAX_CELLS
            || self.native_prefix_steps > MAX_NATIVE_PREFIX_STEPS
            || (self.native_prefix_steps > 0 && self.seed.is_none())
            || self
                .seed
                .as_ref()
                .is_some_and(|seed| self.native_prefix_steps as u64 > seed.generation)
        {
            return Err(RecordingError::Invalid("diagnostic run"));
        }
        if let Some(training) = &self.binding.training_run_id {
            valid_text(training)?;
        }
        if let Some(seed) = &self.seed {
            validate_history(seed, self.grid.cell_len())?;
            if matches!(
                seed.status,
                PressureHistoryStatus::Diagnostic { qualified: false }
            ) || (self.mode == DiagnosticMode::TeacherStage
                && seed.status != PressureHistoryStatus::Accepted)
            {
                return Err(RecordingError::Invalid(
                    "unsupported diagnostic seed history",
                ));
            }
        }
        if self.bytes()? > limits.max_record_bytes {
            return Err(RecordingError::Limit("run payload"));
        }
        Ok(())
    }
    fn bytes(&self) -> Result<usize, RecordingError> {
        let base = 512
            + self.run_id.len()
            + self.source_revision.len()
            + self.binding.id.len()
            + self.binding.version.len()
            + self.provenance.len()
            + self.binding.training_run_id.as_ref().map_or(0, String::len);
        add_bytes(base, self.seed.as_ref().map_or(0, |s| s.pressure.len()))
    }
    fn digest(&self, limits: RecordingLimits) -> [u8; 32] {
        let mut h = Hash::new(b"rheon-diagnostic-run-v1\0");
        h.text(STAGE_CONTRACT);
        h.text(&self.run_id);
        h.text(&self.source_revision);
        hash_grid(&mut h, &self.grid);
        h.f(self.density);
        h.f(self.viscosity);
        h.f(self.dt);
        h.u(mode_id(self.mode) as u64);
        h.u(self.native_prefix_steps as u64);
        h.text(&self.binding.id);
        h.text(&self.binding.version);
        h.bytes(&self.binding.manifest_sha256);
        h.optional_bytes(
            self.binding
                .checkpoint_sha256
                .as_ref()
                .map(|x| x.as_slice()),
        );
        h.optional_bytes(self.binding.training_run_id.as_ref().map(|x| x.as_bytes()));
        h.text(&self.provenance);
        hash_history(&mut h, self.seed.as_ref());
        h.u(limits.max_database_bytes);
        h.u(limits.max_record_bytes as u64);
        h.u(limits.max_records as u64);
        h.text("keep_all_stop");
        h.finish()
    }
}
fn validate_history(p: &ObservedHistory, n: usize) -> Result<(), RecordingError> {
    validate_pressure(&p.pressure, n)?;
    if p.generation == 0 || !p.interval_end.is_finite() || p.interval_end <= 0.0 {
        return Err(RecordingError::Invalid("observed history"));
    }
    Ok(())
}
fn validate_pressure(p: &[f64], n: usize) -> Result<(), RecordingError> {
    if p.len() != n || p.first() != Some(&0.0) || p.iter().any(|v| !v.is_finite()) {
        return Err(RecordingError::Invalid(
            "pressure shape, finiteness or cell-zero gauge",
        ));
    }
    Ok(())
}
fn same_values(a: &[f64], b: &[f64]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(a, b)| a.to_bits() == b.to_bits())
}
fn same_grid_bits(a: &GridGeometry, b: &GridGeometry) -> bool {
    a.counts() == b.counts()
        && a.spacing()
            .into_iter()
            .chain(a.origin())
            .zip(b.spacing().into_iter().chain(b.origin()))
            .all(|(a, b)| a.to_bits() == b.to_bits())
}
fn same_history(a: Option<&ObservedHistory>, b: Option<&ObservedHistory>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => {
            a.generation == b.generation
                && a.interval_end.to_bits() == b.interval_end.to_bits()
                && a.status == b.status
                && same_values(&a.pressure, &b.pressure)
        }
        _ => false,
    }
}
impl PressureSample {
    fn validate(&self, n: usize) -> Result<(), RecordingError> {
        validate_pressure(&self.pressure, n)?;
        let q = &self.observation;
        if [
            q.residual_l2,
            q.predicted_divergence,
            q.actual_divergence,
            q.courant,
            q.relative_limit,
            q.absolute_limit,
            q.predicted_limit,
            q.actual_limit,
            q.courant_limit,
            q.flux_limit,
        ]
        .iter()
        .any(|v| !v.is_finite() || *v < 0.0)
            || !q.net_flux.is_finite()
            || q.relative_residual
                .is_some_and(|v| !v.is_finite() || v < 0.0)
            || (q.qualified && q.hard_stop)
        {
            return Err(RecordingError::Invalid("qualification observation"));
        }
        Ok(())
    }
    fn hash(&self, h: &mut Hash) {
        h.values(&self.pressure);
        let q = &self.observation;
        h.u(1);
        h.f(q.residual_l2);
        h.optional_f(q.relative_residual);
        for v in [
            q.predicted_divergence,
            q.actual_divergence,
            q.courant,
            q.net_flux,
            q.relative_limit,
            q.absolute_limit,
            q.predicted_limit,
            q.actual_limit,
            q.courant_limit,
            q.flux_limit,
        ] {
            h.f(v);
        }
        h.u(u64::from(q.qualified));
        h.u(u64::from(q.hard_stop));
    }
    fn bytes(&self) -> Result<usize, RecordingError> {
        add_bytes(256, self.pressure.len())
    }
}
impl DiagnosticAttempt {
    fn validate(
        &self,
        run: &DiagnosticRun,
        expected: Option<&ObservedHistory>,
    ) -> Result<(), RecordingError> {
        self.request.validate(run)?;
        if !same_history(self.request.prior.as_ref(), expected) {
            return Err(RecordingError::Invalid("diagnostic history continuity"));
        }
        let n = run.grid.cell_len();
        match &self.outcome {
            PredictionOutcome::Candidate(sample) => {
                sample.validate(n)?;
                if !self.dispatched {
                    return Err(RecordingError::Invalid("candidate without dispatch"));
                }
            }
            PredictionOutcome::NoCandidate { reason, detail, .. } => {
                valid_text(detail)?;
                if !self.dispatched && *reason == NoCandidateReason::InvalidOutput {
                    return Err(RecordingError::Invalid("output failure without dispatch"));
                }
            }
        }
        if let Some(reference) = &self.native_reference {
            reference.sample.validate(n)?;
            if reference.request_digest != self.request.digest() {
                return Err(RecordingError::Invalid("native reference request differs"));
            }
        }
        match (run.mode, self.transition) {
            (DiagnosticMode::IndependentStopOnReject, HistoryTransition::DiagnosticAdvanced) => {
                if !matches!(&self.outcome,PredictionOutcome::Candidate(sample) if sample.observation.qualified)
                {
                    return Err(RecordingError::Invalid(
                        "advance requires qualified candidate",
                    ));
                }
            }
            (DiagnosticMode::IndependentStopOnReject, HistoryTransition::NotAdvanced) => {
                if matches!(&self.outcome,PredictionOutcome::Candidate(sample) if sample.observation.qualified)
                {
                    return Err(RecordingError::Invalid(
                        "qualified independent candidate must advance",
                    ));
                }
            }
            (DiagnosticMode::TeacherStage, HistoryTransition::NativeTeacherAdvanced) => {
                if !self.native_reference.as_ref().is_some_and(|r| {
                    r.sample.observation.qualified
                        && !matches!(
                            r.history_status,
                            PressureHistoryStatus::Diagnostic { qualified: false }
                        )
                }) {
                    return Err(RecordingError::Invalid(
                        "teacher advance requires qualified native reference",
                    ));
                }
            }
            (DiagnosticMode::TeacherStage, HistoryTransition::NotAdvanced) => {}
            _ => {
                return Err(RecordingError::Invalid(
                    "transition contradicts diagnostic mode",
                ));
            }
        }
        Ok(())
    }
    fn next_history(&self) -> Option<ObservedHistory> {
        let (sample, status) = match self.transition {
            HistoryTransition::DiagnosticAdvanced => match &self.outcome {
                PredictionOutcome::Candidate(s) => Some((
                    s,
                    PressureHistoryStatus::Diagnostic {
                        qualified: s.observation.qualified,
                    },
                )),
                _ => None,
            },
            HistoryTransition::NativeTeacherAdvanced => self
                .native_reference
                .as_ref()
                .map(|r| (&r.sample, r.history_status)),
            HistoryTransition::NotAdvanced => None,
        }?;
        Some(ObservedHistory {
            generation: self.request.generation,
            interval_end: self.request.interval_end(),
            status,
            pressure: sample.pressure.clone(),
        })
    }
    fn bytes(&self) -> Result<usize, RecordingError> {
        let mut total = 512 + self.request.run_id.len();
        for v in [&self.request.rhs, &self.request.initial_residual] {
            total = add_bytes(total, v.len())?;
        }
        if let Some(p) = &self.request.prior {
            total = add_bytes(total, p.pressure.len())?;
        }
        total = total
            .checked_add(match &self.outcome {
                PredictionOutcome::Candidate(s) => s.bytes()?,
                PredictionOutcome::NoCandidate { detail, .. } => 256 + detail.len(),
            })
            .ok_or(RecordingError::Limit("attempt bytes"))?;
        if let Some(r) = &self.native_reference {
            total = total
                .checked_add(r.sample.bytes()?)
                .ok_or(RecordingError::Limit("attempt bytes"))?;
        }
        Ok(total)
    }
    fn digest(&self, run_digest: &[u8; 32]) -> [u8; 32] {
        let mut h = Hash::new(b"rheon-diagnostic-attempt-v1\0");
        h.bytes(run_digest);
        h.u(self.attempt_index as u64);
        self.request.hash(&mut h);
        h.u(u64::from(self.dispatched));
        match &self.outcome {
            PredictionOutcome::Candidate(s) => {
                h.u(1);
                s.hash(&mut h);
            }
            PredictionOutcome::NoCandidate {
                reason,
                detail,
                invalid_payload_sha256,
            } => {
                h.u(0);
                h.u(reason_id(*reason) as u64);
                h.text(detail);
                h.optional_bytes(invalid_payload_sha256.as_ref().map(|x| x.as_slice()));
            }
        }
        h.u(u64::from(self.native_reference.is_some()));
        if let Some(r) = &self.native_reference {
            h.bytes(&r.request_digest);
            h.u(history_id(r.history_status) as u64);
            r.sample.hash(&mut h);
        }
        h.u(transition_id(self.transition) as u64);
        h.finish()
    }
}
fn add_bytes(base: usize, values: usize) -> Result<usize, RecordingError> {
    values
        .checked_mul(8)
        .and_then(|n| base.checked_add(n))
        .ok_or(RecordingError::Limit("payload arithmetic"))
}

struct Hash(Sha256);
impl Hash {
    fn new(domain: &[u8]) -> Self {
        let mut h = Sha256::new();
        h.update(domain);
        Self(h)
    }
    fn u(&mut self, v: u64) {
        self.0.update(v.to_le_bytes());
    }
    fn f(&mut self, v: f64) {
        self.0.update(v.to_le_bytes());
    }
    fn bytes(&mut self, v: &[u8]) {
        self.u(v.len() as u64);
        self.0.update(v);
    }
    fn text(&mut self, v: &str) {
        self.bytes(v.as_bytes());
    }
    fn optional_bytes(&mut self, v: Option<&[u8]>) {
        self.u(u64::from(v.is_some()));
        if let Some(v) = v {
            self.bytes(v);
        }
    }
    fn optional_f(&mut self, v: Option<f64>) {
        self.u(u64::from(v.is_some()));
        if let Some(v) = v {
            self.f(v);
        }
    }
    fn values(&mut self, v: &[f64]) {
        self.u(v.len() as u64);
        for v in v {
            self.f(*v);
        }
    }
    fn finish(self) -> [u8; 32] {
        self.0.finalize().into()
    }
}
fn hash_grid(h: &mut Hash, g: &GridGeometry) {
    for n in g.counts() {
        h.u(n as u64);
    }
    for v in g.spacing().into_iter().chain(g.origin()) {
        h.f(v);
    }
}
fn hash_history(h: &mut Hash, p: Option<&ObservedHistory>) {
    h.u(u64::from(p.is_some()));
    if let Some(p) = p {
        h.u(p.generation);
        h.f(p.interval_end);
        h.u(history_id(p.status) as u64);
        h.values(&p.pressure);
    }
}
fn mode_id(v: DiagnosticMode) -> i64 {
    match v {
        DiagnosticMode::TeacherStage => 0,
        DiagnosticMode::IndependentStopOnReject => 1,
    }
}
fn history_id(v: PressureHistoryStatus) -> i64 {
    match v {
        PressureHistoryStatus::Accepted => 0,
        PressureHistoryStatus::Diagnostic { qualified: true } => 1,
        PressureHistoryStatus::Diagnostic { qualified: false } => 2,
    }
}
fn transition_id(v: HistoryTransition) -> i64 {
    match v {
        HistoryTransition::NotAdvanced => 0,
        HistoryTransition::DiagnosticAdvanced => 1,
        HistoryTransition::NativeTeacherAdvanced => 2,
    }
}
fn reason_id(v: NoCandidateReason) -> i64 {
    match v {
        NoCandidateReason::Unsupported => 0,
        NoCandidateReason::UnsupportedContext => 1,
        NoCandidateReason::Cancelled => 2,
        NoCandidateReason::RuntimeFailure => 3,
        NoCandidateReason::InvalidOutput => 4,
    }
}
fn terminal_id(v: TerminalOutcome) -> i64 {
    match v {
        TerminalOutcome::Completed => 1,
        TerminalOutcome::StoppedOnRejection => 2,
        TerminalOutcome::Cancelled => 3,
        TerminalOutcome::Failed => 4,
    }
}

/// One synchronous writer in the existing recording owner. No resume, queue,
/// model loading, automatic deletion or destructor completion is supported.
pub struct DiagnosticRecorder {
    connection: Connection,
    path: PathBuf,
    run: DiagnosticRun,
    limits: RecordingLimits,
    count: usize,
    progress: Progress,
    state: WriterState,
}
#[derive(Clone)]
struct Progress {
    history: Option<ObservedHistory>,
    advanced: Option<u64>,
    stopped: bool,
    rejected: bool,
}
impl Progress {
    fn new(run: &DiagnosticRun) -> Self {
        Self {
            history: run.seed.clone(),
            advanced: None,
            stopped: false,
            rejected: false,
        }
    }
    fn observe(&mut self, a: &DiagnosticAttempt) {
        if let Some(history) = a.next_history() {
            self.advanced = Some(history.generation);
            self.history = Some(history);
        }
        self.stopped = a.transition == HistoryTransition::NotAdvanced;
        self.rejected =
            matches!(&a.outcome,PredictionOutcome::Candidate(s) if !s.observation.qualified);
    }
}
impl DiagnosticRecorder {
    pub fn create(
        path: &Path,
        run: DiagnosticRun,
        limits: RecordingLimits,
    ) -> Result<Self, RecordingError> {
        run.validate(limits)?;
        let file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        file.sync_all()?;
        let mut connection = connect(path, limits)?;
        connection.pragma_update(None, "page_size", super::PAGE_BYTES)?;
        let pages = limits.max_database_bytes / super::PAGE_BYTES;
        connection.pragma_update(None, "max_page_count", pages)?;
        if connection.pragma_query_value::<u64, _>(None, "max_page_count", |r| r.get(0))? != pages {
            return Err(RecordingError::Limit("diagnostic database pages"));
        }
        connection.execute_batch(
            "PRAGMA journal_mode=DELETE; PRAGMA synchronous=FULL; PRAGMA foreign_keys=ON;",
        )?;
        let tx = connection.transaction()?;
        tx.execute_batch(SCHEMA)?;
        let seed = run.seed.as_ref();
        let seed_data = seed.map(|s| encode_values(&s.pressure));
        tx.execute("INSERT INTO diagnostic_run VALUES(1,?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,'keep_all_stop',?22,?23)",
            params![run.run_id,run.source_revision,encode_grid(&run.grid),run.density.to_le_bytes().as_slice(),run.viscosity.to_le_bytes().as_slice(),
                run.dt.to_le_bytes().as_slice(),mode_id(run.mode),run.binding.id,run.binding.version,run.binding.manifest_sha256.as_slice(),
                run.binding.checkpoint_sha256.as_ref().map(|x|x.as_slice()),run.binding.training_run_id,run.provenance,
                seed.map(|s|s.generation.to_le_bytes()).as_ref().map(|b|b.as_slice()),seed.map(|s|s.interval_end.to_le_bytes()).as_ref().map(|b|b.as_slice()),
                seed.map(|s|history_id(s.status)),seed_data,limits.max_database_bytes as i64,limits.max_record_bytes as i64,limits.max_records as i64,STAGE_CONTRACT,
                run.digest(limits).as_slice(),run.native_prefix_steps as i64])?;
        tx.execute(
            "INSERT INTO diagnostic_lifecycle VALUES(1,0,?1,NULL,NULL,?2)",
            params![
                "incomplete; producer has not declared a terminal outcome",
                terminal_digest(&run.digest(limits), None).as_slice()
            ],
        )?;
        tx.commit()?;
        let progress = Progress::new(&run);
        Ok(Self {
            connection,
            path: path.into(),
            run,
            limits,
            count: 0,
            progress,
            state: WriterState::Open,
        })
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn run(&self) -> &DiagnosticRun {
        &self.run
    }
    /// Acknowledged commits only; an append error requires cold reconciliation.
    pub fn recorded_count(&self) -> usize {
        self.count
    }
    pub fn append(&mut self, attempt: &DiagnosticAttempt) -> Result<(), RecordingError> {
        if self.state != WriterState::Open {
            return Err(RecordingError::Closed);
        }
        let result = self.append_inner(attempt);
        if result.is_err() {
            self.state = WriterState::Poisoned;
        }
        result
    }
    fn append_inner(&mut self, a: &DiagnosticAttempt) -> Result<(), RecordingError> {
        if self.progress.stopped {
            return Err(RecordingError::Closed);
        }
        if self.count >= self.limits.max_records {
            return Err(RecordingError::Limit("diagnostic attempt count"));
        }
        if a.attempt_index != self.count {
            return Err(RecordingError::Invalid("diagnostic attempt index"));
        }
        // Check all supplied allocation sizes before recomputing residuals or encoding.
        let bytes = a.bytes()?;
        if bytes > self.limits.max_record_bytes {
            return Err(RecordingError::Limit("diagnostic attempt payload"));
        }
        a.validate(&self.run, self.progress.history.as_ref())?;
        let prior = a.request.prior.as_ref();
        let prior_data = prior.map(|p| encode_values(&p.pressure));
        let (outcome, reason, detail, invalid_digest) = match &a.outcome {
            PredictionOutcome::Candidate(_) => (1, None, None, None),
            PredictionOutcome::NoCandidate {
                reason,
                detail,
                invalid_payload_sha256,
            } => (
                0,
                Some(reason_id(*reason)),
                Some(detail.as_str()),
                invalid_payload_sha256.as_ref().map(|d| d.as_slice()),
            ),
        };
        let tx = self.connection.transaction()?;
        tx.execute("INSERT INTO attempts VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)",
            params![a.attempt_index as i64,a.request.generation.to_le_bytes().as_slice(),a.request.interval_start.to_le_bytes().as_slice(),
                a.request.dt.to_le_bytes().as_slice(),encode_values(&a.request.rhs),encode_values(&a.request.initial_residual),
                prior.map(|p|p.generation.to_le_bytes()).as_ref().map(|b|b.as_slice()),prior.map(|p|p.interval_end.to_le_bytes()).as_ref().map(|b|b.as_slice()),
                prior.map(|p|history_id(p.status)),prior_data,i64::from(a.dispatched),outcome,reason,detail,invalid_digest,
                transition_id(a.transition),a.request.digest().as_slice(),bytes as i64,a.digest(&self.run.digest(self.limits)).as_slice()])?;
        if let PredictionOutcome::Candidate(sample) = &a.outcome {
            write_sample(&tx, a.attempt_index, 0, &a.request.digest(), sample, None)?;
        }
        if let Some(reference) = &a.native_reference {
            write_sample(
                &tx,
                a.attempt_index,
                1,
                &reference.request_digest,
                &reference.sample,
                Some(reference.history_status),
            )?;
        }
        tx.commit()?;
        self.count += 1;
        self.progress.observe(a);
        Ok(())
    }
    /// Closes admission before validation or writing. Failures, including an
    /// uncertain terminal commit, cannot be retried on this writer.
    pub fn finish(&mut self, terminal: DiagnosticTerminal) -> Result<(), RecordingError> {
        if self.state == WriterState::TerminalAttempted {
            return Err(RecordingError::Closed);
        }
        let was_open = self.state == WriterState::Open;
        self.state = WriterState::TerminalAttempted;
        valid_text(&terminal.detail)?;
        if matches!(
            terminal.outcome,
            TerminalOutcome::Completed | TerminalOutcome::StoppedOnRejection
        ) && !was_open
        {
            return Err(RecordingError::Invalid("poisoned diagnostic completion"));
        }
        // A failed append might have committed: reconcile the actual prefix,
        // rather than treating recorded_count() as authoritative after error.
        let reader = DiagnosticReader::open(&self.path, self.limits)?;
        let (count, progress) = reader.audit()?;
        validate_terminal(&terminal, count, &progress, &self.run)?;
        let changed=self.connection.execute("UPDATE diagnostic_lifecycle SET outcome=?1,detail=?2,produced_attempts=?3,advanced_generation=?4,digest=?5 WHERE id=1 AND outcome=0",
            params![terminal_id(terminal.outcome),terminal.detail,i64::try_from(terminal.produced_attempts).map_err(|_|RecordingError::Limit("produced attempts"))?,
                terminal.last_advanced_generation.map(u64::to_le_bytes).as_ref().map(|b|b.as_slice()),terminal_digest(&self.run.digest(self.limits),Some(&terminal)).as_slice()])?;
        if changed != 1 {
            return Err(RecordingError::Closed);
        }
        Ok(())
    }
}
fn write_sample(
    c: &Connection,
    index: usize,
    role: i64,
    request: &[u8; 32],
    sample: &PressureSample,
    history_status: Option<PressureHistoryStatus>,
) -> Result<(), RecordingError> {
    let q = &sample.observation;
    c.execute("INSERT INTO pressure_samples VALUES(?1,?2,?3,?4,1,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)",
        params![index as i64,role,request.as_slice(),encode_values(&sample.pressure),q.residual_l2.to_le_bytes().as_slice(),
            q.relative_residual.map(f64::to_le_bytes).as_ref().map(|b|b.as_slice()),q.predicted_divergence.to_le_bytes().as_slice(),
            q.actual_divergence.to_le_bytes().as_slice(),q.courant.to_le_bytes().as_slice(),q.net_flux.to_le_bytes().as_slice(),
            q.relative_limit.to_le_bytes().as_slice(),q.absolute_limit.to_le_bytes().as_slice(),q.predicted_limit.to_le_bytes().as_slice(),
            q.actual_limit.to_le_bytes().as_slice(),q.courant_limit.to_le_bytes().as_slice(),q.flux_limit.to_le_bytes().as_slice(),
            i64::from(q.qualified),i64::from(q.hard_stop),history_status.map(history_id)])?;
    Ok(())
}

/// Indexed cold reader: metadata plus one selected attempt, no model imports or
/// whole-run field cache. Full lineage and terminal claims require `verify()`.
pub struct DiagnosticReader {
    connection: Connection,
    run: DiagnosticRun,
    limits: RecordingLimits,
    recorded_limits: RecordingLimits,
}
impl DiagnosticReader {
    pub fn open(path: &Path, limits: RecordingLimits) -> Result<Self, RecordingError> {
        limits.validate()?;
        let metadata = fs::symlink_metadata(path)?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err(RecordingError::Invalid(
                "regular diagnostic database required",
            ));
        }
        if metadata.len() > limits.max_database_bytes {
            return Err(RecordingError::Limit("diagnostic database bytes"));
        }
        let connection = connect(path, limits)?;
        check_kind(&connection)?;
        connection.execute_batch("PRAGMA query_only=ON; PRAGMA trusted_schema=OFF;")?;
        if connection.query_row::<String, _, _>("PRAGMA quick_check", [], |r| r.get(0))? != "ok" {
            return Err(RecordingError::Integrity("diagnostic SQLite quick_check"));
        }
        let (run, recorded_limits) = read_header(&connection, limits)?;
        if metadata.len() > recorded_limits.max_database_bytes {
            return Err(RecordingError::Limit("recorded diagnostic database bytes"));
        }
        let reader = Self {
            connection,
            run,
            limits,
            recorded_limits,
        };
        reader.count()?;
        reader.status()?;
        Ok(reader)
    }
    pub fn run(&self) -> &DiagnosticRun {
        &self.run
    }
    pub fn recorded_limits(&self) -> RecordingLimits {
        self.recorded_limits
    }
    pub fn count(&self) -> Result<usize, RecordingError> {
        let (n, min, max): (i64, Option<i64>, Option<i64>) = self.connection.query_row(
            "SELECT count(*),min(attempt_index),max(attempt_index) FROM attempts",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;
        let count = usize::try_from(n).map_err(|_| RecordingError::Integrity("attempt count"))?;
        if count
            > self
                .limits
                .max_records
                .min(self.recorded_limits.max_records)
        {
            return Err(RecordingError::Limit("diagnostic attempt count"));
        }
        if (count == 0 && (min.is_some() || max.is_some()))
            || (count > 0 && (min != Some(0) || max != Some(n - 1)))
        {
            return Err(RecordingError::Integrity("diagnostic index prefix"));
        }
        Ok(count)
    }
    /// None is explicitly active/incomplete, including after producer Drop.
    pub fn status(&self) -> Result<Option<DiagnosticTerminal>, RecordingError> {
        let mut statement=self.connection.prepare("SELECT outcome,detail,produced_attempts,advanced_generation,digest FROM diagnostic_lifecycle WHERE id=1")?;
        let mut rows = statement.query([])?;
        let r = rows
            .next()?
            .ok_or(RecordingError::Integrity("diagnostic lifecycle missing"))?;
        let outcome: i64 = r.get(0)?;
        let detail = bounded_text(r, 1)?;
        valid_text(&detail)?;
        let terminal = if outcome == 0 {
            if !is_null(r, 2)?
                || !is_null(r, 3)?
                || detail != "incomplete; producer has not declared a terminal outcome"
            {
                return Err(RecordingError::Integrity("active diagnostic lifecycle"));
            }
            None
        } else {
            Some(DiagnosticTerminal {
                outcome: parse_terminal(outcome)?,
                detail,
                produced_attempts: usize::try_from(r.get::<_, i64>(2)?)
                    .map_err(|_| RecordingError::Integrity("produced attempts"))?,
                last_advanced_generation: optional_u(r, 3)?,
            })
        };
        if blob(r, 4)? != terminal_digest(&self.run.digest(self.recorded_limits), terminal.as_ref())
        {
            return Err(RecordingError::Integrity("diagnostic terminal digest"));
        }
        Ok(terminal)
    }
    pub fn select(&self, attempt_index: usize) -> Result<DiagnosticAttempt, RecordingError> {
        let _snapshot = self.connection.unchecked_transaction()?;
        self.verify_header()?;
        self.count()?;
        self.read_attempt(attempt_index)
    }
    pub fn verify(&self) -> Result<(), RecordingError> {
        self.audit().map(|_| ())
    }
    fn audit(&self) -> Result<(usize, Progress), RecordingError> {
        let _snapshot = self.connection.unchecked_transaction()?;
        self.verify_header()?;
        if self
            .connection
            .query_row::<String, _, _>("PRAGMA integrity_check", [], |r| r.get(0))?
            != "ok"
        {
            return Err(RecordingError::Integrity(
                "diagnostic SQLite integrity_check",
            ));
        }
        let orphans:bool=self.connection.query_row("SELECT EXISTS(SELECT 1 FROM pressure_samples s LEFT JOIN attempts a ON a.attempt_index=s.attempt_index WHERE a.attempt_index IS NULL)",[],|r|r.get(0))?;
        if orphans {
            return Err(RecordingError::Integrity("orphan diagnostic samples"));
        }
        let count = self.count()?;
        let mut progress = Progress::new(&self.run);
        for index in 0..count {
            if progress.stopped {
                return Err(RecordingError::Integrity("attempt after diagnostic stop"));
            }
            let a = self.read_attempt(index)?;
            a.validate(&self.run, progress.history.as_ref())?;
            progress.observe(&a);
        }
        if let Some(terminal) = self.status()? {
            validate_terminal(&terminal, count, &progress, &self.run)?;
        }
        Ok((count, progress))
    }
    fn verify_header(&self) -> Result<(), RecordingError> {
        check_kind(&self.connection)?;
        let (run, limits) = read_header(&self.connection, self.limits)?;
        if run.digest(limits) != self.run.digest(self.recorded_limits)
            || limits != self.recorded_limits
        {
            return Err(RecordingError::Integrity("diagnostic header changed"));
        }
        Ok(())
    }
    fn read_attempt(&self, index: usize) -> Result<DiagnosticAttempt, RecordingError> {
        let i = i64::try_from(index).map_err(|_| RecordingError::Missing(index))?;
        let cap = self
            .limits
            .max_record_bytes
            .min(self.recorded_limits.max_record_bytes);
        // Preflight total BLOB/text bytes in SQL before allocating any field.
        let measured:Option<i64>=self.connection.query_row("SELECT length(rhs)+length(residual)+coalesce(length(prior_pressure),0)+coalesce(length(CAST(detail AS BLOB)),0)+coalesce((SELECT sum(length(pressure)) FROM pressure_samples WHERE attempt_index=?1),0) FROM attempts WHERE attempt_index=?1",[i],|r|r.get(0)).optional()?;
        let measured = measured.ok_or(RecordingError::Missing(index))?;
        if measured < 0
            || usize::try_from(measured).map_err(|_| RecordingError::Limit("attempt bytes"))? > cap
        {
            return Err(RecordingError::Limit("diagnostic selected payload"));
        }
        let mut statement=self.connection.prepare("SELECT generation,start,dt,rhs,residual,prior_generation,prior_end,prior_status,prior_pressure,dispatched,outcome,reason,detail,invalid_payload_digest,transition,request_digest,payload_bytes,digest FROM attempts WHERE attempt_index=?1")?;
        let mut rows = statement.query([i])?;
        let r = rows.next()?.ok_or(RecordingError::Missing(index))?;
        let request = DiagnosticRequest {
            run_id: self.run.run_id.clone(),
            grid: self.run.grid.clone(),
            density: self.run.density,
            viscosity: self.run.viscosity,
            generation: read_u(r, 0)?,
            interval_start: read_f(r, 1)?,
            dt: read_f(r, 2)?,
            rhs: read_values(r, 3, self.run.grid.cell_len(), cap)?,
            initial_residual: read_values(r, 4, self.run.grid.cell_len(), cap)?,
            prior: read_history(r, 5, self.run.grid.cell_len(), cap)?,
        };
        let dispatched = read_bool(r, 9)?;
        let outcome_id: i64 = r.get(10)?;
        let samples = self.read_samples(index, cap, &request.digest())?;
        let (candidate, native_reference) = samples;
        let outcome = match outcome_id {
            1 => {
                if !is_null(r, 11)? || !is_null(r, 12)? || !is_null(r, 13)? {
                    return Err(RecordingError::Integrity("candidate has failure metadata"));
                }
                PredictionOutcome::Candidate(
                    candidate.ok_or(RecordingError::Integrity("candidate sample missing"))?,
                )
            }
            0 => {
                if candidate.is_some() {
                    return Err(RecordingError::Integrity("no-candidate has sample"));
                }
                PredictionOutcome::NoCandidate {
                    reason: parse_reason(r.get(11)?)?,
                    detail: bounded_text(r, 12)?,
                    invalid_payload_sha256: optional_digest(r, 13)?,
                }
            }
            _ => return Err(RecordingError::Unsupported("diagnostic outcome")),
        };
        let attempt = DiagnosticAttempt {
            attempt_index: index,
            request,
            dispatched,
            outcome,
            native_reference,
            transition: parse_transition(r.get(14)?)?,
        };
        let stored_bytes = usize::try_from(r.get::<_, i64>(16)?)
            .map_err(|_| RecordingError::Integrity("attempt byte count"))?;
        if stored_bytes > cap {
            return Err(RecordingError::Limit("diagnostic selected payload"));
        }
        if stored_bytes != attempt.bytes()?
            || blob(r, 15)? != attempt.request.digest()
            || blob(r, 17)? != attempt.digest(&self.run.digest(self.recorded_limits))
        {
            return Err(RecordingError::Integrity(
                "diagnostic attempt digest or size",
            ));
        }
        // A selection validates its own context/output/transition. Full chain
        // validation against preceding records belongs to verify().
        attempt.validate(&self.run, attempt.request.prior.as_ref())?;
        Ok(attempt)
    }
    fn read_samples(
        &self,
        index: usize,
        cap: usize,
        expected_request: &[u8; 32],
    ) -> Result<(Option<PressureSample>, Option<NativeReference>), RecordingError> {
        let mut statement=self.connection.prepare("SELECT role,request_digest,pressure,policy,residual_l2,relative_residual,predicted_divergence,actual_divergence,courant,net_flux,relative_limit,absolute_limit,predicted_limit,actual_limit,courant_limit,flux_limit,qualified,hard_stop,history_status FROM pressure_samples WHERE attempt_index=?1 ORDER BY role")?;
        let mut rows = statement.query([index as i64])?;
        let mut candidate = None;
        let mut native = None;
        let mut count = 0;
        while let Some(r) = rows.next()? {
            count += 1;
            if count > 2 {
                return Err(RecordingError::Limit("diagnostic sample count"));
            }
            if r.get::<_, i64>(3)? != 1 {
                return Err(RecordingError::Unsupported("qualification policy"));
            }
            let sample = PressureSample {
                pressure: read_values(r, 2, self.run.grid.cell_len(), cap)?,
                observation: QualificationObservation {
                    policy: QualificationPolicy::HostPressureGateV1,
                    residual_l2: read_f(r, 4)?,
                    relative_residual: optional_f(r, 5)?,
                    predicted_divergence: read_f(r, 6)?,
                    actual_divergence: read_f(r, 7)?,
                    courant: read_f(r, 8)?,
                    net_flux: read_f(r, 9)?,
                    relative_limit: read_f(r, 10)?,
                    absolute_limit: read_f(r, 11)?,
                    predicted_limit: read_f(r, 12)?,
                    actual_limit: read_f(r, 13)?,
                    courant_limit: read_f(r, 14)?,
                    flux_limit: read_f(r, 15)?,
                    qualified: read_bool(r, 16)?,
                    hard_stop: read_bool(r, 17)?,
                },
            };
            let digest = read_digest(r, 1)?;
            if &digest != expected_request {
                return Err(RecordingError::Integrity("sample request digest"));
            }
            match r.get::<_, i64>(0)? {
                0 => {
                    if !is_null(r, 18)? {
                        return Err(RecordingError::Integrity(
                            "candidate cannot claim native history status",
                        ));
                    }
                    candidate = Some(sample);
                }
                1 => {
                    native = Some(NativeReference {
                        request_digest: digest,
                        sample,
                        history_status: parse_history(r.get(18)?)?,
                    });
                }
                _ => return Err(RecordingError::Unsupported("diagnostic sample role")),
            }
        }
        Ok((candidate, native))
    }
}
fn validate_terminal(
    t: &DiagnosticTerminal,
    count: usize,
    p: &Progress,
    run: &DiagnosticRun,
) -> Result<(), RecordingError> {
    valid_text(&t.detail)?;
    if t.last_advanced_generation == Some(0)
        || t.produced_attempts < count
        || (p.advanced.is_some() && t.last_advanced_generation < p.advanced)
        || (t.produced_attempts == count && t.last_advanced_generation != p.advanced)
    {
        return Err(RecordingError::Invalid(
            "terminal does not cover diagnostic prefix",
        ));
    }
    let seed_generation = run.seed.as_ref().map_or(0, |s| s.generation);
    let produced = u64::try_from(t.produced_attempts)
        .map_err(|_| RecordingError::Invalid("terminal chronology overflow"))?;
    let possible_tail = seed_generation
        .checked_add(produced)
        .ok_or(RecordingError::Invalid("terminal chronology overflow"))?;
    let missing = u64::try_from(t.produced_attempts - count)
        .map_err(|_| RecordingError::Invalid("terminal chronology overflow"))?;
    let durable_baseline = p.history.as_ref().map_or(0, |h| h.generation);
    let possible_unrecorded_tail = durable_baseline
        .checked_add(missing)
        .ok_or(RecordingError::Invalid("terminal chronology overflow"))?;
    if t.last_advanced_generation
        .is_some_and(|g| g <= seed_generation || g > possible_tail || g > possible_unrecorded_tail)
        || (p.stopped && t.last_advanced_generation != p.advanced)
    {
        return Err(RecordingError::Invalid(
            "terminal advanced generation chronology",
        ));
    }
    match t.outcome {
        TerminalOutcome::Completed
            if count == 0
                || p.stopped
                || t.produced_attempts != count
                || t.last_advanced_generation != p.advanced =>
        {
            Err(RecordingError::Invalid("incomplete diagnostic completion"))
        }
        TerminalOutcome::StoppedOnRejection
            if run.mode != DiagnosticMode::IndependentStopOnReject
                || !p.stopped
                || !p.rejected
                || t.produced_attempts != count
                || t.last_advanced_generation != p.advanced =>
        {
            Err(RecordingError::Invalid("diagnostic rejection terminal"))
        }
        _ => Ok(()),
    }
}
fn terminal_digest(run: &[u8; 32], terminal: Option<&DiagnosticTerminal>) -> [u8; 32] {
    let mut h = Hash::new(b"rheon-diagnostic-terminal-v1\0");
    h.bytes(run);
    if let Some(t) = terminal {
        h.u(terminal_id(t.outcome) as u64);
        h.text(&t.detail);
        h.u(t.produced_attempts as u64);
        h.u(u64::from(t.last_advanced_generation.is_some()));
        if let Some(g) = t.last_advanced_generation {
            h.u(g);
        }
    } else {
        h.u(0);
        h.text("incomplete; producer has not declared a terminal outcome");
    }
    h.finish()
}
fn check_kind(c: &Connection) -> Result<(), RecordingError> {
    let application: i64 = c.pragma_query_value(None, "application_id", |r| r.get(0))?;
    let version: i64 = c.pragma_query_value(None, "user_version", |r| r.get(0))?;
    if application != APPLICATION_ID || version != VERSION {
        return Err(RecordingError::Unsupported(
            "diagnostic archive kind/version",
        ));
    }
    Ok(())
}
fn read_header(
    c: &Connection,
    limits: RecordingLimits,
) -> Result<(DiagnosticRun, RecordingLimits), RecordingError> {
    let bytes:i64=c.query_row("SELECT 512+length(CAST(run_id AS BLOB))+length(CAST(revision AS BLOB))+length(CAST(binding_id AS BLOB))+length(CAST(binding_version AS BLOB))+length(CAST(provenance AS BLOB))+coalesce(length(CAST(training_run AS BLOB)),0)+coalesce(length(seed_pressure),0) FROM diagnostic_run WHERE id=1",[],|r|r.get(0))?;
    if bytes < 0
        || usize::try_from(bytes).map_err(|_| RecordingError::Limit("diagnostic header"))?
            > limits.max_record_bytes
    {
        return Err(RecordingError::Limit("diagnostic header payload"));
    }
    let mut statement=c.prepare("SELECT run_id,revision,geometry,density,viscosity,dt,mode,binding_id,binding_version,manifest,checkpoint,training_run,provenance,seed_generation,seed_end,seed_status,seed_pressure,max_database_bytes,max_record_bytes,max_records,stage,retention,digest,native_prefix_steps FROM diagnostic_run WHERE id=1")?;
    let mut rows = statement.query([])?;
    let r = rows
        .next()?
        .ok_or(RecordingError::Integrity("diagnostic header missing"))?;
    let recorded = RecordingLimits {
        max_database_bytes: u64::try_from(r.get::<_, i64>(17)?)
            .map_err(|_| RecordingError::Integrity("diagnostic database cap"))?,
        max_record_bytes: usize::try_from(r.get::<_, i64>(18)?)
            .map_err(|_| RecordingError::Integrity("diagnostic record cap"))?,
        max_records: usize::try_from(r.get::<_, i64>(19)?)
            .map_err(|_| RecordingError::Integrity("diagnostic record quota"))?,
    };
    recorded.validate()?;
    let cap = limits.max_record_bytes.min(recorded.max_record_bytes);
    let grid = decode_grid(blob(r, 2)?)?;
    let run = DiagnosticRun {
        run_id: bounded_text(r, 0)?,
        source_revision: bounded_text(r, 1)?,
        grid: grid.clone(),
        density: read_f(r, 3)?,
        viscosity: read_f(r, 4)?,
        dt: read_f(r, 5)?,
        mode: parse_mode(r.get(6)?)?,
        native_prefix_steps: usize::try_from(r.get::<_, i64>(23)?)
            .map_err(|_| RecordingError::Invalid("native prefix steps"))?,
        binding: BindingIdentity {
            id: bounded_text(r, 7)?,
            version: bounded_text(r, 8)?,
            manifest_sha256: read_digest(r, 9)?,
            checkpoint_sha256: optional_digest(r, 10)?,
            training_run_id: if is_null(r, 11)? {
                None
            } else {
                Some(bounded_text(r, 11)?)
            },
        },
        provenance: bounded_text(r, 12)?,
        seed: read_history(r, 13, grid.cell_len(), cap)?,
    };
    run.validate(recorded)?;
    if run.bytes()? > cap {
        return Err(RecordingError::Limit("diagnostic header payload"));
    }
    if bounded_text(r, 20)? != STAGE_CONTRACT || bounded_text(r, 21)? != "keep_all_stop" {
        return Err(RecordingError::Unsupported("diagnostic stage/retention"));
    }
    if blob(r, 22)? != run.digest(recorded) {
        return Err(RecordingError::Integrity("diagnostic run digest"));
    }
    Ok((run, recorded))
}
fn encode_grid(g: &GridGeometry) -> Vec<u8> {
    g.counts()
        .into_iter()
        .flat_map(|v| (v as u64).to_le_bytes())
        .chain(
            g.spacing()
                .into_iter()
                .chain(g.origin())
                .flat_map(f64::to_le_bytes),
        )
        .collect()
}
fn decode_grid(raw: &[u8]) -> Result<GridGeometry, RecordingError> {
    if raw.len() != 72 {
        return Err(RecordingError::Integrity("diagnostic geometry width"));
    }
    let counts =
        std::array::from_fn(|i| u64::from_le_bytes(raw[i * 8..i * 8 + 8].try_into().unwrap()));
    let spacing = std::array::from_fn(|i| {
        f64::from_le_bytes(raw[24 + i * 8..32 + i * 8].try_into().unwrap())
    });
    let origin = std::array::from_fn(|i| {
        f64::from_le_bytes(raw[48 + i * 8..56 + i * 8].try_into().unwrap())
    });
    let grid = GridGeometry::new(counts, spacing, origin)
        .map_err(|_| RecordingError::Invalid("diagnostic geometry"))?;
    if grid.cell_len() > MAX_CELLS {
        return Err(RecordingError::Limit("diagnostic cells"));
    }
    Ok(grid)
}
fn encode_values(values: &[f64]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_le_bytes()).collect()
}
fn blob<'a>(r: &'a rusqlite::Row<'_>, i: usize) -> Result<&'a [u8], RecordingError> {
    r.get_ref(i)?
        .as_blob()
        .map_err(|_| RecordingError::Integrity("diagnostic blob type"))
}
fn is_null(r: &rusqlite::Row<'_>, i: usize) -> Result<bool, RecordingError> {
    Ok(matches!(r.get_ref(i)?, rusqlite::types::ValueRef::Null))
}
fn read_u(r: &rusqlite::Row<'_>, i: usize) -> Result<u64, RecordingError> {
    Ok(u64::from_le_bytes(super::number(blob(r, i)?)?))
}
fn read_f(r: &rusqlite::Row<'_>, i: usize) -> Result<f64, RecordingError> {
    Ok(f64::from_le_bytes(super::number(blob(r, i)?)?))
}
fn optional_u(r: &rusqlite::Row<'_>, i: usize) -> Result<Option<u64>, RecordingError> {
    if is_null(r, i)? {
        Ok(None)
    } else {
        read_u(r, i).map(Some)
    }
}
fn optional_f(r: &rusqlite::Row<'_>, i: usize) -> Result<Option<f64>, RecordingError> {
    if is_null(r, i)? {
        Ok(None)
    } else {
        read_f(r, i).map(Some)
    }
}
fn read_digest(r: &rusqlite::Row<'_>, i: usize) -> Result<[u8; 32], RecordingError> {
    blob(r, i)?
        .try_into()
        .map_err(|_| RecordingError::Integrity("diagnostic digest width"))
}
fn optional_digest(r: &rusqlite::Row<'_>, i: usize) -> Result<Option<[u8; 32]>, RecordingError> {
    if is_null(r, i)? {
        Ok(None)
    } else {
        read_digest(r, i).map(Some)
    }
}
fn read_bool(r: &rusqlite::Row<'_>, i: usize) -> Result<bool, RecordingError> {
    match r.get::<_, i64>(i)? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(RecordingError::Integrity("diagnostic boolean")),
    }
}
fn read_values(
    r: &rusqlite::Row<'_>,
    i: usize,
    n: usize,
    cap: usize,
) -> Result<Vec<f64>, RecordingError> {
    let raw = blob(r, i)?;
    if raw.len() > cap {
        return Err(RecordingError::Limit("diagnostic field bytes"));
    }
    if n.checked_mul(8) != Some(raw.len()) {
        return Err(RecordingError::Integrity("diagnostic field shape"));
    }
    Ok(raw
        .chunks_exact(8)
        .map(|b| f64::from_le_bytes(b.try_into().unwrap()))
        .collect())
}
fn read_history(
    r: &rusqlite::Row<'_>,
    start: usize,
    n: usize,
    cap: usize,
) -> Result<Option<ObservedHistory>, RecordingError> {
    if is_null(r, start)? {
        if !(start + 1..start + 4)
            .map(|i| is_null(r, i))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .all(|v| v)
        {
            return Err(RecordingError::Integrity("partial absent history"));
        }
        return Ok(None);
    }
    Ok(Some(ObservedHistory {
        generation: read_u(r, start)?,
        interval_end: read_f(r, start + 1)?,
        status: parse_history(r.get(start + 2)?)?,
        pressure: read_values(r, start + 3, n, cap)?,
    }))
}
fn parse_mode(v: i64) -> Result<DiagnosticMode, RecordingError> {
    match v {
        0 => Ok(DiagnosticMode::TeacherStage),
        1 => Ok(DiagnosticMode::IndependentStopOnReject),
        _ => Err(RecordingError::Unsupported("diagnostic mode")),
    }
}
fn parse_history(v: i64) -> Result<PressureHistoryStatus, RecordingError> {
    match v {
        0 => Ok(PressureHistoryStatus::Accepted),
        1 => Ok(PressureHistoryStatus::Diagnostic { qualified: true }),
        2 => Ok(PressureHistoryStatus::Diagnostic { qualified: false }),
        _ => Err(RecordingError::Unsupported("diagnostic history status")),
    }
}
fn parse_transition(v: i64) -> Result<HistoryTransition, RecordingError> {
    match v {
        0 => Ok(HistoryTransition::NotAdvanced),
        1 => Ok(HistoryTransition::DiagnosticAdvanced),
        2 => Ok(HistoryTransition::NativeTeacherAdvanced),
        _ => Err(RecordingError::Unsupported("diagnostic transition")),
    }
}
fn parse_reason(v: i64) -> Result<NoCandidateReason, RecordingError> {
    match v {
        0 => Ok(NoCandidateReason::Unsupported),
        1 => Ok(NoCandidateReason::UnsupportedContext),
        2 => Ok(NoCandidateReason::Cancelled),
        3 => Ok(NoCandidateReason::RuntimeFailure),
        4 => Ok(NoCandidateReason::InvalidOutput),
        _ => Err(RecordingError::Unsupported("no-candidate reason")),
    }
}
fn parse_terminal(v: i64) -> Result<TerminalOutcome, RecordingError> {
    match v {
        1 => Ok(TerminalOutcome::Completed),
        2 => Ok(TerminalOutcome::StoppedOnRejection),
        3 => Ok(TerminalOutcome::Cancelled),
        4 => Ok(TerminalOutcome::Failed),
        _ => Err(RecordingError::Unsupported("diagnostic terminal outcome")),
    }
}
