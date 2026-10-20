//! Opt-in closed projection execution through the existing full interval owner.
use super::*;
use crate::projection_step::*;
use std::{
    cell::{Cell, RefCell},
    sync::Arc,
};

impl<'a> SessionSnapshot<'a> {
    pub fn projection_intent(&self) -> Option<ProjectionIntent> {
        self.projection_intent
    }
    /// Accepted receipt only. A later failed attempt never replaces this Arc.
    pub fn projection_attempt(&self) -> Option<&'a Arc<ProjectionAttempt>> {
        self.projection_attempt
    }
}
impl SimulationSession {
    /// Construct the sealed, stationary constant-density builtin projection
    /// path. Intent is immutable; legacy Session step entrypoints then refuse.
    /// There is no caller provider, optional viscosity, or external runtime.
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_projection(
        identity: SessionIdentity,
        grid: GridGeometry,
        config: LiquidTransportConfig,
        implementation: PressureImplementation,
        fraction: Vec<f64>,
        workspace_limit: usize,
        max_steps: usize,
        intent: ProjectionIntent,
    ) -> Result<Self, SessionError> {
        let configuration = SessionConfiguration {
            transport: config,
            implementation,
            boundary_workspace_limit: workspace_limit,
            max_steps,
        };
        intent.validate_configuration(&configuration).map_err(|_| {
            SessionError::PressureBinding(crate::solver_provider::ProviderRejection::Unsupported(
                "invalid native projection configuration",
            ))
        })?;
        let mut session = Self::new(
            identity,
            grid,
            config,
            implementation,
            fraction,
            workspace_limit,
            max_steps,
        )?;
        session.projection_intent = Some(intent);
        Ok(session)
    }
    /// Produce and qualify actual provisional native fields, then publish only
    /// after the existing complete carrier/volume gates. Failure returns a
    /// separate attempt and consumes no accepted quota. Deadlines are cooperative.
    /// The additive recording profile supports cold library inspection; the
    /// existing desktop workflow graph deliberately does not inspect this profile.
    pub fn step_with_native_projection(
        &mut self,
        inputs: LiquidStepInputs<'_>,
        workspace: &mut ProjectionStepWorkspace,
        deadline: Option<Instant>,
        cancel: impl FnMut(ProjectionStepStage) -> bool,
    ) -> Result<ProjectionStepAccepted, ProjectionStepFailure> {
        self.projection_step_with_clock(inputs, workspace, deadline, cancel, Instant::now)
    }
    fn projection_step_with_clock(
        &mut self,
        inputs: LiquidStepInputs<'_>,
        workspace: &mut ProjectionStepWorkspace,
        deadline: Option<Instant>,
        mut cancel: impl FnMut(ProjectionStepStage) -> bool,
        mut now: impl FnMut() -> Instant,
    ) -> Result<ProjectionStepAccepted, ProjectionStepFailure> {
        let Some(attempt_id) = self.projection_attempt_counter.checked_add(1) else {
            return Err(ProjectionStepFailure {
                error: ProjectionStepError::AttemptExhausted,
                attempt: None,
            });
        };
        self.projection_attempt_counter = attempt_id;
        let state = self.simulation.state();
        let accepted_generation = state.carrier.generation;
        let interval_start = state.carrier.time;
        let mut capture = ProjectionCapture {
            stage: ProjectionAttemptStage::SessionGuard,
            attempt: Arc::new(ProjectionAttempt {
                version: 1,
                intent: ProjectionIntent::BuiltinNativeV1,
                source: self.identity.clone(),
                attempt_id,
                accepted_generation,
                attempted_generation: accepted_generation.checked_add(1),
                interval_start,
                requested_dt: inputs.requested_dt,
                actual_dt: None,
                input_sha256: crate::workflow::input_digest(state),
                controls_sha256: crate::workflow::request_digest(inputs),
                provisional_input_id: None,
                producer: None,
                qualification: None,
                selected_native: None,
                outcome: ProjectionAttemptOutcome::Failed {
                    stage: ProjectionAttemptStage::SessionGuard,
                    code: ProjectionFailureCode::InvalidInput,
                },
                output_sha256: None,
                diagnostic: String::new(),
            }),
        };
        #[cfg(test)]
        let _alias = workspace
            .alias_before_work()
            .then(|| capture.attempt.clone());
        let expired = Cell::new(false);
        let result = (|| {
            capture
                .update(|_| {})
                .map_err(|_| ProjectionStepError::ReceiptAliased)?;
            if self.projection_intent != Some(ProjectionIntent::BuiltinNativeV1) {
                return Err(ProjectionStepError::IntentMismatch);
            }
            if self.stopped {
                return Err(ProjectionStepError::Session(SessionError::Stopped));
            }
            if self.steps >= self.max_steps {
                return Err(ProjectionStepError::Session(SessionError::StepLimit));
            }
            if capture.attempt.attempted_generation.is_none() {
                return Err(ProjectionStepError::Session(SessionError::Step(
                    LiquidStepError::Carrier(crate::SimulationError::GenerationOverflow),
                )));
            }
            if !workspace.matches(
                self.simulation.grid(),
                self.configuration.transport.carrier.density,
                self.configuration.implementation,
            ) {
                return Err(ProjectionStepError::WorkspaceMismatch);
            }
            if deadline.is_some_and(|d| now() >= d) {
                return Err(ProjectionStepError::Session(SessionError::Deadline));
            }
            let mut guarded = |stage| {
                if deadline.is_some_and(|d| now() >= d) {
                    expired.set(true);
                    return true;
                }
                let cancelled = cancel(stage);
                if deadline.is_some_and(|d| now() >= d) {
                    expired.set(true);
                    return true;
                }
                cancelled
            };
            let control = ProjectionControl {
                callback: RefCell::new(&mut guarded),
            };
            let (report, native) = self
                .simulation
                .step_with_native_projection(
                    inputs,
                    ProjectionStepInput {
                        workspace,
                        capture: &mut capture,
                        source: &self.identity,
                        configuration: &self.configuration,
                        control: &control,
                    },
                    |stage| {
                        crate::projection_step::ProjectionCancellation::poll(
                            &control,
                            ProjectionStepStage::Liquid(stage),
                        )
                    },
                )
                .map_err(|error| projection_step_error(error, expired.get()))?;
            // Successful receipt was fully validated/sealed before Liquid's
            // infallible pressure copy and owner swaps. Adoption adds no callback.
            self.interval_start = Some(interval_start);
            self.steps += 1;
            self.report = Some(report);
            self.provider = Some(native.clone());
            self.configured = Some(
                crate::solver_provider::configured::ConfiguredProviderReport {
                    workflow: None,
                    dynamic_viscosity: 0.0,
                    binding: None,
                    context: None,
                    prediction_called: false,
                    pressure: native,
                },
            );
            self.pressure_history = None;
            self.projection_attempt = Some(capture.attempt.clone());
            Ok(report)
        })();
        match result {
            Ok(report) => Ok(ProjectionStepAccepted {
                report,
                attempt: capture.attempt,
            }),
            Err(error) => {
                let (code, cancelled, refused) = failure_facts(&error, capture.stage);
                let diagnostic = format!("{error}").chars().take(240).collect::<String>();
                let outcome = if cancelled {
                    ProjectionAttemptOutcome::Cancelled {
                        stage: capture.stage,
                    }
                } else if refused {
                    ProjectionAttemptOutcome::Refused {
                        stage: capture.stage,
                        code,
                    }
                } else {
                    ProjectionAttemptOutcome::Failed {
                        stage: capture.stage,
                        code,
                    }
                };
                // Aliased diagnostic scratch is not mutated. A separate bounded
                // failure receipt is allocated before any accepted publication.
                if Arc::get_mut(&mut capture.attempt).is_none() {
                    let mut failure = (*capture.attempt).clone();
                    failure.outcome = outcome;
                    failure.output_sha256 = None;
                    failure.diagnostic = diagnostic;
                    capture.attempt = Arc::new(failure);
                } else {
                    let failure = Arc::get_mut(&mut capture.attempt).expect("exclusivity checked");
                    failure.outcome = outcome;
                    failure.output_sha256 = None;
                    failure.diagnostic = diagnostic;
                }
                Err(ProjectionStepFailure {
                    error,
                    attempt: Some(capture.attempt),
                })
            }
        }
    }
    pub(super) fn require_legacy_intent(&self) -> Result<(), SessionError> {
        if self.projection_intent.is_some() {
            return Err(SessionError::PressureBinding(
                crate::solver_provider::ProviderRejection::Unsupported(
                    "projection Session requires its immutable native projection path",
                ),
            ));
        }
        Ok(())
    }
}
fn projection_step_error(error: LiquidStepError, expired: bool) -> ProjectionStepError {
    if expired {
        return ProjectionStepError::Session(SessionError::Deadline);
    }
    match error {
        LiquidStepError::Carrier(crate::SimulationError::ProjectionMixedQualificationFallback) => {
            ProjectionStepError::UnrepresentableMixedQualificationFallback
        }
        LiquidStepError::Carrier(crate::SimulationError::ProjectionReceiptAliased) => {
            ProjectionStepError::ReceiptAliased
        }
        LiquidStepError::Carrier(crate::SimulationError::ProjectionReceiptInvalid(reason)) => {
            ProjectionStepError::InvalidReceipt(reason)
        }
        other => ProjectionStepError::Session(SessionError::Step(other)),
    }
}
fn failure_facts(
    error: &ProjectionStepError,
    stage: ProjectionAttemptStage,
) -> (ProjectionFailureCode, bool, bool) {
    use ProjectionFailureCode as C;
    let code = match error {
        ProjectionStepError::IntentMismatch => C::IntentMismatch,
        ProjectionStepError::WorkspaceMismatch => C::WorkspaceMismatch,
        ProjectionStepError::ReceiptAliased => C::ReceiptAliased,
        ProjectionStepError::InvalidReceipt(_) => C::InvalidInput,
        ProjectionStepError::UnrepresentableMixedQualificationFallback => {
            C::UnrepresentableMixedQualificationFallback
        }
        ProjectionStepError::AttemptExhausted => C::InvalidInput,
        ProjectionStepError::Session(SessionError::Stopped) => C::Stopped,
        ProjectionStepError::Session(SessionError::StepLimit) => C::StepLimit,
        ProjectionStepError::Session(SessionError::Deadline) => return (C::Deadline, true, false),
        ProjectionStepError::Session(SessionError::Step(LiquidStepError::Carrier(
            crate::SimulationError::GenerationOverflow,
        ))) => C::GenerationExhausted,
        ProjectionStepError::Session(SessionError::Step(
            LiquidStepError::Cancelled
            | LiquidStepError::Carrier(
                crate::SimulationError::Cancelled { .. }
                | crate::SimulationError::Pressure(crate::PressureError::Cancelled { .. })
                | crate::SimulationError::Projection(
                    crate::solver_provider::composition::projection::ProjectionError::Cancelled
                    | crate::solver_provider::composition::projection::ProjectionError::Pressure(
                        crate::PressureError::Cancelled { .. },
                    ),
                ),
            )
            | LiquidStepError::Volume(crate::LiquidVolumeError::Cancelled { .. }),
        )) => return (C::InvalidInput, true, false),
        _ => match stage {
            ProjectionAttemptStage::NativeProducer => C::NativePressure,
            ProjectionAttemptStage::HostQualification => C::HostQualification,
            ProjectionAttemptStage::VolumeTransport => C::Volume,
            _ => C::Carrier,
        },
    };
    (code, false, stage == ProjectionAttemptStage::SessionGuard)
}

#[cfg(test)]
#[path = "projection_tests.rs"]
mod tests;
