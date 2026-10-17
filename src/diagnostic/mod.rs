//! Sealed-water diagnostic sessions using Rheon-owned advancement and history.
//!
//! `IndependentStopOnReject` retains the raw same-request attempt and stops before
//! advancing a rejected candidate. It never publishes accepted Session state or
//! performs native fallback/refinement. `TeacherStage` advances native history.
//! The legacy raw-continuation mode remains an explicitly selected experimental
//! trajectory policy; callers must inspect every returned quality observation.
mod engine;
mod scene;
mod wire;
use crate::recording::diagnostic::NoCandidateReason;
pub use engine::{
    AdvanceResult, AdvanceStop, DiagnosticSession, ExecutionPolicy, Frame, FramePhase, Input,
    PRESSURE_ITERATION_LIMIT, Quality,
};
pub use scene::{DT, MU, RHO, Scene};
pub use wire::{AdapterErrorCode, AdapterReply, InputSemantic, OutputRole};

/// Provider failure with exact dispatch and optional invalid-wire evidence.
#[derive(Debug)]
pub struct PredictionFailure {
    pub reason: NoCandidateReason,
    pub detail: String,
    pub dispatched: bool,
    /// Digest of the complete invalid packet, never a truncated prefix.
    pub invalid_payload_sha256: Option<[u8; 32]>,
}
