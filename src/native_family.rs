//! Borrowed dispatch to the existing MAC and particle-preview state owners.
//! The facade checks the requested family, caller-declared source and accepted
//! generation; the selected owner still owns all numerical validation, clocks,
//! quotas, cancellation and publication. It stores no state or snapshot history.
//!
//! Native snapshot borrows prevent stepping while their fields remain in use:
//! ```compile_fail
//! use rheon::native_family::{NativeSession, NativeStepRequest};
//! fn advance_with_live_snapshot(mut session: NativeSession<'_>, request: NativeStepRequest<'_>) {
//!     let snapshot = session.snapshot();
//!     let _ = session.step(request, |_| false);
//!     drop(snapshot);
//! }
//! ```

use crate::{
    LiquidStepInputs, LiquidStepReport, LiquidStepStage, MaterialSession, MaterialSessionError,
    MaterialSnapshot, MaterialStepControls, ParticleStepReport, ProviderStepWorkspace,
    SessionError, SessionIdentity, SessionSnapshot, SimulationSession,
    solver_provider::ProviderReport,
};
use std::{fmt, time::Instant};

/// Native representations remain distinct. Particle support is the existing
/// preview, not a qualified interchangeable pressure or whole-state candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeFamilyKind {
    Mac,
    ParticlePreview,
}

/// A caller retains its constructor identity separately from the borrowed owner.
/// Equality detects a wrong declared source; it proves neither authenticity nor
/// a cryptographic identity of every native input field.
#[derive(Debug, Clone, Copy)]
pub struct NativeInputExpectation<'a> {
    pub identity: &'a SessionIdentity,
    pub generation: u64,
}

pub struct MacRequest<'a> {
    pub expected: NativeInputExpectation<'a>,
    pub inputs: LiquidStepInputs<'a>,
    pub workspace: &'a mut ProviderStepWorkspace,
    pub deadline: Option<Instant>,
}

pub struct ParticleRequest<'a> {
    pub expected: NativeInputExpectation<'a>,
    pub controls: MaterialStepControls,
    pub deadline: Option<Instant>,
}

/// Family-specific requests retain their native controls and storage contracts.
pub enum NativeStepRequest<'a> {
    Mac(MacRequest<'a>),
    ParticlePreview(ParticleRequest<'a>),
}

/// Real owner cancellation checkpoints. The particle owner supplies a poll, not
/// a named numerical stage; the facade does not infer one from callback order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeCancellationPoint {
    Mac(LiquidStepStage),
    ParticlePoll,
}

/// Original numerical reports, including actual shortened dt and native
/// pressure selection. Elapsed timings retain their original owner and scope.
#[derive(Debug)]
#[allow(
    clippy::large_enum_variant,
    reason = "Native reports stay by value; boxing would allocate after the owner's publication"
)]
pub enum NativeStepReport {
    Mac {
        step: LiquidStepReport,
        pressure: ProviderReport,
    },
    ParticlePreview(ParticleStepReport),
}

impl NativeStepReport {
    pub fn actual_dt(&self) -> f64 {
        match self {
            Self::Mac { step, .. } => step.carrier.step.dt,
            Self::ParticlePreview(report) => report.dt,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeRefusal {
    WrongFamily {
        owner: NativeFamilyKind,
        requested: NativeFamilyKind,
    },
    WrongSource {
        family: NativeFamilyKind,
        expected_run_id: u64,
        actual_run_id: u64,
        source_revision_mismatch: bool,
    },
    StaleGeneration {
        family: NativeFamilyKind,
        expected: u64,
        actual: u64,
    },
}

#[derive(Debug)]
pub enum NativeStepError {
    Refused(NativeRefusal),
    Mac(SessionError),
    ParticlePreview(MaterialSessionError),
}

impl fmt::Display for NativeStepError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Refused(reason) => write!(f, "native family request refused: {reason:?}"),
            Self::Mac(error) => write!(f, "MAC native owner: {error}"),
            Self::ParticlePreview(error) => write!(f, "particle preview native owner: {error}"),
        }
    }
}

impl std::error::Error for NativeStepError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Refused(_) => None,
            Self::Mac(error) => Some(error),
            Self::ParticlePreview(error) => Some(error),
        }
    }
}

pub enum NativeSnapshot<'a> {
    Mac(SessionSnapshot<'a>),
    ParticlePreview(MaterialSnapshot<'a>),
}

impl NativeSnapshot<'_> {
    pub fn family(&self) -> NativeFamilyKind {
        match self {
            Self::Mac(_) => NativeFamilyKind::Mac,
            Self::ParticlePreview(_) => NativeFamilyKind::ParticlePreview,
        }
    }

    pub fn identity(&self) -> &SessionIdentity {
        match self {
            Self::Mac(snapshot) => snapshot.identity(),
            Self::ParticlePreview(snapshot) => snapshot.identity(),
        }
    }

    pub fn generation(&self) -> u64 {
        match self {
            Self::Mac(snapshot) => snapshot.state().carrier.generation,
            Self::ParticlePreview(snapshot) => snapshot.generation(),
        }
    }
}

/// Exclusive borrows of real owners, not replacement candidates or a second
/// scheduler. Requests with a different family are refused without conversion.
pub enum NativeSession<'a> {
    Mac(&'a mut SimulationSession),
    ParticlePreview(&'a mut MaterialSession),
}

impl NativeSession<'_> {
    pub fn snapshot(&self) -> NativeSnapshot<'_> {
        match self {
            Self::Mac(owner) => NativeSnapshot::Mac(owner.snapshot()),
            Self::ParticlePreview(owner) => NativeSnapshot::ParticlePreview(owner.snapshot()),
        }
    }

    pub fn step(
        &mut self,
        request: NativeStepRequest<'_>,
        mut cancel: impl FnMut(NativeCancellationPoint) -> bool,
    ) -> Result<NativeStepReport, NativeStepError> {
        match (self, request) {
            (Self::Mac(owner), NativeStepRequest::Mac(request)) => {
                let snapshot = owner.snapshot();
                check_expectation(
                    NativeFamilyKind::Mac,
                    request.expected,
                    snapshot.identity(),
                    snapshot.state().carrier.generation,
                )
                .map_err(NativeStepError::Refused)?;
                owner
                    .step_with_pressure_provider(
                        request.inputs,
                        request.workspace,
                        None,
                        request.deadline,
                        |stage| cancel(NativeCancellationPoint::Mac(stage)),
                    )
                    .map(|(step, pressure)| NativeStepReport::Mac { step, pressure })
                    .map_err(NativeStepError::Mac)
            }
            (Self::ParticlePreview(owner), NativeStepRequest::ParticlePreview(request)) => {
                let snapshot = owner.snapshot();
                check_expectation(
                    NativeFamilyKind::ParticlePreview,
                    request.expected,
                    snapshot.identity(),
                    snapshot.generation(),
                )
                .map_err(NativeStepError::Refused)?;
                owner
                    .step(request.controls, request.deadline, || {
                        cancel(NativeCancellationPoint::ParticlePoll)
                    })
                    .map(NativeStepReport::ParticlePreview)
                    .map_err(NativeStepError::ParticlePreview)
            }
            (Self::Mac(_), NativeStepRequest::ParticlePreview(_)) => {
                Err(NativeStepError::Refused(NativeRefusal::WrongFamily {
                    owner: NativeFamilyKind::Mac,
                    requested: NativeFamilyKind::ParticlePreview,
                }))
            }
            (Self::ParticlePreview(_), NativeStepRequest::Mac(_)) => {
                Err(NativeStepError::Refused(NativeRefusal::WrongFamily {
                    owner: NativeFamilyKind::ParticlePreview,
                    requested: NativeFamilyKind::Mac,
                }))
            }
        }
    }
}

fn check_expectation(
    family: NativeFamilyKind,
    expected: NativeInputExpectation<'_>,
    actual_identity: &SessionIdentity,
    actual_generation: u64,
) -> Result<(), NativeRefusal> {
    if expected.identity != actual_identity {
        return Err(NativeRefusal::WrongSource {
            family,
            expected_run_id: expected.identity.run_id,
            actual_run_id: actual_identity.run_id,
            source_revision_mismatch: expected.identity.source_revision
                != actual_identity.source_revision,
        });
    }
    if expected.generation != actual_generation {
        return Err(NativeRefusal::StaleGeneration {
            family,
            expected: expected.generation,
            actual: actual_generation,
        });
    }
    Ok(())
}

/// Named future transfer contracts. Neither direction is admitted by C1-A.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferOperator {
    MacToParticleV1,
    ParticleToMacV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransferUnavailable {
    pub operator: TransferOperator,
}

impl fmt::Display for TransferUnavailable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "native transfer {:?} is not admitted", self.operator)
    }
}

impl std::error::Error for TransferUnavailable {}

pub fn require_transfer(operator: TransferOperator) -> Result<(), TransferUnavailable> {
    Err(TransferUnavailable { operator })
}

#[cfg(test)]
mod tests;
