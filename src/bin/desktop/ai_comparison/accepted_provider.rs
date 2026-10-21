//! Accepted Session adapter over the existing bounded CPU process owner.
//! Package pins are verified locally; runtime/source strings remain caller
//! declarations. Session retains qualification, history, and commit authority.
use super::{
    adapter::{AdapterError, AdapterPackage},
    engine::Input,
    model_process::{ChildSlot, ModelProcess, PredictionFailure, ProviderIdentity, StartupFailure},
};
use rheon::{
    recording::diagnostic::NoCandidateReason,
    recording::provider::runtime::{RuntimeObservation, RuntimeOutcome, RuntimeRequestStamp},
    solver_provider::{
        ProviderRejection,
        configured::{ContextPressureProvider, ContextPressureRequest, ContextProviderIdentity},
        context::{PressureContext, PressureContextInput, PressureContextMode},
    },
};
use std::{
    path::Path,
    sync::{Arc, atomic::AtomicBool},
};

pub(super) struct AcceptedCpuProvider {
    package: AdapterPackage,
    process: ModelProcess,
    cancelled: Arc<AtomicBool>,
    identity: Arc<ContextProviderIdentity>,
    guess: Vec<f64>,
    residual: Vec<f64>,
    last_failure: Option<PredictionFailure>,
    last_observation: Option<RuntimeObservation>,
}
fn hex(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn rejection(reason: NoCandidateReason) -> ProviderRejection {
    match reason {
        NoCandidateReason::Cancelled => ProviderRejection::Cancelled,
        NoCandidateReason::Unsupported | NoCandidateReason::UnsupportedContext => {
            ProviderRejection::Unsupported("CPU package does not support accepted context")
        }
        NoCandidateReason::RuntimeFailure | NoCandidateReason::InvalidOutput => {
            ProviderRejection::Execution("CPU runtime failed to produce a valid candidate")
        }
    }
}
impl AcceptedCpuProvider {
    /// Scratch cap covers two host f64 arrays at the package's maximum cell
    /// count. Existing process protocol/output and child budgets remain separate.
    /// Trusted-local package files must not be concurrently mutated during use.
    pub(super) fn start(
        path: &Path,
        identity: Arc<ContextProviderIdentity>,
        slot: ChildSlot,
        cancelled: Arc<AtomicBool>,
        scratch_limit_bytes: usize,
        progress: impl FnMut(&str) -> Result<(), String>,
    ) -> Result<Self, StartupFailure> {
        identity
            .validate()
            .map_err(|_| "Invalid CPU provider identity")?;
        let package = AdapterPackage::load(path).map_err(|e| e.to_string())?;
        let expected = ProviderIdentity {
            id: identity.id.clone(),
            version: identity.version.clone(),
            manifest_sha256: hex(&identity.manifest_sha256),
            checkpoint_sha256: identity.checkpoint_sha256.as_ref().map(hex),
            training_run_id: identity.training_run_id.clone(),
        };
        // Adapter v1 has no checkpoint/training identity fields. Do not attach
        // a caller's unverified checkpoint claim to an accepted binding.
        if expected.id != package.id()
            || expected.version != package.version()
            || expected.manifest_sha256 != package.manifest_sha256()
            || expected.checkpoint_sha256.is_some()
            || expected.training_run_id.is_some()
        {
            return Err("CPU adapter differs from declared package pin".into());
        }
        let count = package.support().max_cells;
        let bytes = count
            .checked_mul(2 * size_of::<f64>())
            .ok_or("CPU adapter scratch size overflow")?;
        if bytes > scratch_limit_bytes {
            return Err("CPU adapter scratch exceeds caller byte limit".into());
        }
        let mut guess = Vec::new();
        let mut residual = Vec::new();
        guess
            .try_reserve_exact(count)
            .map_err(|_| "CPU guess scratch allocation failed")?;
        residual
            .try_reserve_exact(count)
            .map_err(|_| "CPU residual scratch allocation failed")?;
        guess.resize(count, 0.0);
        residual.resize(count, 0.0);
        let process =
            ModelProcess::start_pinned_typed(path, slot, &cancelled, &expected, progress)?;
        Ok(Self {
            package,
            process,
            cancelled,
            identity,
            guess,
            residual,
            last_failure: None,
            last_observation: None,
        })
    }
    /// Clear bounded observations before a new owned interval, including one
    /// that support declines. This never resets or revives the process owner.
    pub(super) fn begin_interval_observation(&mut self) {
        self.last_failure = None;
        self.last_observation = None;
    }
    pub(super) fn last_observation(&self) -> Option<&RuntimeObservation> {
        self.last_observation.as_ref()
    }
    /// Most recent prediction failure, including actual process dispatch facts.
    /// A successful prediction clears it. Inert supports calls do not change it.
    pub(super) fn last_failure(&self) -> Option<&PredictionFailure> {
        self.last_failure.as_ref()
    }
    pub(super) fn scratch_bytes(&self) -> usize {
        (self.guess.len() + self.residual.len()) * size_of::<f64>()
    }
    fn fail(&mut self, reason: NoCandidateReason, detail: &'static str) -> ProviderRejection {
        self.last_failure = Some(PredictionFailure {
            reason,
            detail: detail.into(),
            dispatched: false,
            invalid_payload_sha256: None,
        });
        if let Some(observation) = &mut self.last_observation {
            observation.outcome = RuntimeOutcome::NoCandidate {
                reason,
                detail: detail.into(),
                dispatched: false,
                invalid_payload_sha256: None,
            };
        }
        rejection(reason)
    }
}
impl ContextPressureProvider for AcceptedCpuProvider {
    fn identity(&self) -> Arc<ContextProviderIdentity> {
        Arc::clone(&self.identity)
    }
    fn supports(&self, request: &ContextPressureRequest<'_>) -> Result<(), ProviderRejection> {
        let c = &request.context;
        if c.mode() != PressureContextMode::AcceptedCandidate
            || c.grid().cell_len() > self.guess.len()
        {
            return Err(ProviderRejection::Unsupported(
                "CPU bridge requires bounded accepted context",
            ));
        }
        self.package
            .validate_support(
                c.grid(),
                c.density(),
                request.dynamic_viscosity,
                c.dt(),
                request.native_prefix_steps,
                c.prior().is_some(),
            )
            .map_err(|e| match e {
                AdapterError::Unsupported(_) => {
                    ProviderRejection::Unsupported("CPU package physical/context support declined")
                }
                AdapterError::Invalid(_) | AdapterError::Unavailable(_) => {
                    ProviderRejection::Execution("CPU package support unavailable")
                }
            })
    }
    fn predict(
        &mut self,
        request: &ContextPressureRequest<'_>,
        pressure: &mut [f64],
        poll: &mut dyn FnMut() -> bool,
    ) -> Result<(), ProviderRejection> {
        self.last_failure = None;
        self.last_observation = None;
        let stamp = match RuntimeRequestStamp::capture(request, &self.identity) {
            Ok(stamp) => stamp,
            Err(_) => {
                return Err(self.fail(
                    NoCandidateReason::UnsupportedContext,
                    "Cannot retain accepted runtime request association",
                ));
            }
        };
        self.last_observation = Some(RuntimeObservation {
            request: stamp,
            outcome: RuntimeOutcome::NoCandidate {
                reason: NoCandidateReason::UnsupportedContext,
                detail: "CPU request validation incomplete".into(),
                dispatched: false,
                invalid_payload_sha256: None,
            },
        });
        if self.supports(request).is_err() {
            return Err(self.fail(
                NoCandidateReason::UnsupportedContext,
                "Unsupported accepted CPU context",
            ));
        }
        let c = &request.context;
        let n = c.grid().cell_len();
        if pressure.len() != n {
            return Err(self.fail(
                NoCandidateReason::UnsupportedContext,
                "CPU output shape differs from context",
            ));
        }
        if let Some(prior) = c.prior() {
            self.guess[..n].copy_from_slice(prior.values);
        } else {
            self.guess[..n].fill(0.0);
        }
        if c.initial_residual(&mut self.residual[..n]).is_err() {
            return Err(self.fail(
                NoCandidateReason::UnsupportedContext,
                "Cannot form finite authoritative initial residual",
            ));
        }
        let context = match PressureContext::new(PressureContextInput {
            grid: c.grid(),
            density: c.density(),
            dt: c.dt(),
            interval_start: c.interval_start(),
            rhs: c.rhs(),
            prior: c.prior(),
            run_id: c.run_id(),
            generation: c.generation(),
            mode: c.mode(),
        }) {
            Ok(context) => context,
            Err(_) => {
                return Err(self.fail(
                    NoCandidateReason::UnsupportedContext,
                    "Invalid accepted CPU context",
                ));
            }
        };
        match self.process.predict_typed_with_poll(
            Input {
                context,
                initial_guess: &self.guess[..n],
                residual: &self.residual[..n],
                viscosity: request.dynamic_viscosity,
            },
            &self.cancelled,
            poll,
        ) {
            Ok(values) => {
                // ModelProcess validates finite shape and canonical gauge.
                pressure.copy_from_slice(&values);
                if let Some(observation) = &mut self.last_observation {
                    observation.outcome = RuntimeOutcome::CandidateReturned;
                }
                Ok(())
            }
            Err(failure) => {
                let coarse = rejection(failure.reason);
                if let Some(observation) = &mut self.last_observation {
                    observation.outcome = RuntimeOutcome::NoCandidate {
                        reason: failure.reason,
                        detail: super::recording_detail(&failure.detail),
                        dispatched: failure.dispatched,
                        invalid_payload_sha256: failure.invalid_payload_sha256,
                    };
                }
                self.last_failure = Some(failure);
                Err(coarse)
            }
        }
    }
}

#[cfg(test)]
#[path = "accepted_provider_tests.rs"]
mod tests;
