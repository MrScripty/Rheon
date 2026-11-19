//! In-process full-box transport API. No worker threads or network services.
use crate::{
    BoxFluxStepBoundary, BoxFluxStepWorkspace, Dense3dError, GridGeometry, LiquidStepError,
    LiquidStepInputs, LiquidStepReport, LiquidStepStage, LiquidTransportConfig,
    LiquidTransportSimulation, LiquidTransportView, PressureImplementation, write_dense3d_frame,
};
use std::{fmt, io::Write, time::Instant};
mod configured;
mod projection;

/// Caller-assigned run and source identity, not proof of provenance or independence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionIdentity {
    pub run_id: u64,
    pub source_revision: String,
}

impl SessionIdentity {
    pub(crate) fn is_valid(&self) -> bool {
        !self.source_revision.is_empty()
            && self.source_revision.len() <= 1000
            && !self.source_revision.chars().any(char::is_control)
    }
}

/// Versioned API capabilities; dense3D wire versions remain independently versioned.
#[derive(Debug, Clone, Copy)]
pub struct SessionCapabilities {
    pub api_version: u32,
    pub crate_version: &'static str,
    pub transport_only: bool,
    /// Opt-in full-step provider selection only for the sealed full-box carrier.
    pub sealed_box_pressure_provider: bool,
    /// AcceptedPreviousV1 on the native sealed liquid route; no portable history.
    pub accepted_previous_pressure_v1: bool,
    pub cooperative_cancellation: bool,
    pub hard_realtime: bool,
    pub units: &'static [(&'static str, &'static str)],
}

pub const SESSION_CAPABILITIES: SessionCapabilities = SessionCapabilities {
    api_version: 3,
    crate_version: env!("CARGO_PKG_VERSION"),
    transport_only: true,
    sealed_box_pressure_provider: true,
    accepted_previous_pressure_v1: cfg!(feature = "accepted-pressure-warm-start"),
    cooperative_cancellation: true,
    hard_realtime: false,
    units: &[
        ("position", "m"),
        ("time", "s"),
        ("velocity", "m/s"),
        ("pressure", "Pa"),
        ("fraction", "dimensionless"),
        ("tracer", "dimensionless appearance"),
    ],
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotStage {
    Constructor,
    Accepted,
}

/// Immutable constructor controls supplied to the Session's actual numerical
/// and boundary-workspace owners. Per-call inputs and provider bindings are
/// separate; this value does not infer a future experiment recipe.
#[derive(Debug, Clone, Copy)]
pub struct SessionConfiguration {
    pub transport: LiquidTransportConfig,
    pub implementation: PressureImplementation,
    pub boundary_workspace_limit: usize,
    pub max_steps: usize,
}

/// Immutable borrow of one publication. Rust prevents stepping while it is used.
/// Pressure is the last interval projection pressure, not end-surface pressure.
/// Copy fields explicitly if they must outlive the next step; that storage belongs
/// to the consumer. There is no hidden snapshot history or unbounded queue.
pub struct SessionSnapshot<'a> {
    identity: &'a SessionIdentity,
    configuration: &'a SessionConfiguration,
    interval_start: Option<f64>,
    grid: &'a GridGeometry,
    state: LiquidTransportView<'a>,
    report: Option<&'a LiquidStepReport>,
    provider: Option<&'a crate::solver_provider::ProviderReport>,
    configured: Option<&'a crate::solver_provider::configured::ConfiguredProviderReport>,
    frame: usize,
    projection_intent: Option<crate::projection_step::ProjectionIntent>,
    projection_attempt: Option<&'a std::sync::Arc<crate::projection_step::ProjectionAttempt>>,
}
impl<'a> SessionSnapshot<'a> {
    pub fn identity(&self) -> &'a SessionIdentity {
        self.identity
    }
    pub fn configuration(&self) -> &'a SessionConfiguration {
        self.configuration
    }
    /// Exact carrier time before the last completely accepted interval. None
    /// at construction; never reconstructed by subtracting dt from its end.
    pub fn accepted_interval_start(&self) -> Option<f64> {
        self.interval_start
    }
    pub fn grid(&self) -> &'a GridGeometry {
        self.grid
    }
    pub fn state(&self) -> LiquidTransportView<'a> {
        self.state
    }
    pub fn report(&self) -> Option<&'a LiquidStepReport> {
        self.report
    }
    /// Pressure selection for this accepted full step only. None on constructor
    /// or a successful legacy step. The contained provider stage is pressure;
    /// this snapshot's Accepted stage covers the complete transport interval.
    pub fn provider_report(&self) -> Option<&'a crate::solver_provider::ProviderReport> {
        self.provider
    }
    pub fn frame(&self) -> usize {
        self.frame
    }
    pub fn stage(&self) -> SnapshotStage {
        if self.report.is_some() {
            SnapshotStage::Accepted
        } else {
            SnapshotStage::Constructor
        }
    }
    /// Existing frame encoding only. This does not produce a completed dataset
    /// manifest or v2 controls, and does not encode warm-start history/provenance.
    /// Writer failures do not modify simulation state.
    pub fn write_dense3d<W: Write>(&self, out: &mut W) -> Result<(), Dense3dError> {
        write_dense3d_frame(out, self.grid, self.state, self.frame, self.report)
    }
}

#[derive(Debug)]
pub enum SessionError {
    InvalidIdentity,
    Stopped,
    StepLimit,
    Deadline,
    Step(LiquidStepError),
    Workspace(crate::SimulationError),
    PressureBinding(crate::solver_provider::ProviderRejection),
}
impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "simulation session: {self:?}")
    }
}
impl std::error::Error for SessionError {}

/// Exclusive owner. Drop releases resources; stop is terminal but preserves the
/// last accepted snapshot. Rejected steps are retryable and consume no step quota.
/// Physics memory and boundary scratch have separate explicit constructor limits.
pub struct SimulationSession {
    identity: SessionIdentity,
    configuration: SessionConfiguration,
    interval_start: Option<f64>,
    simulation: LiquidTransportSimulation,
    workspace: BoxFluxStepWorkspace,
    report: Option<LiquidStepReport>,
    provider: Option<crate::solver_provider::ProviderReport>,
    configured: Option<crate::solver_provider::configured::ConfiguredProviderReport>,
    pressure_history: Option<crate::solver_provider::configured::AcceptedHistoryStamp>,
    steps: usize,
    max_steps: usize,
    stopped: bool,
    projection_intent: Option<crate::projection_step::ProjectionIntent>,
    projection_attempt: Option<std::sync::Arc<crate::projection_step::ProjectionAttempt>>,
    projection_attempt_counter: u64,
}
impl SimulationSession {
    pub fn new(
        identity: SessionIdentity,
        grid: GridGeometry,
        config: LiquidTransportConfig,
        implementation: PressureImplementation,
        fraction: Vec<f64>,
        workspace_limit: usize,
        max_steps: usize,
    ) -> Result<Self, SessionError> {
        if !identity.is_valid() {
            return Err(SessionError::InvalidIdentity);
        }
        let workspace = BoxFluxStepWorkspace::new(
            grid.clone(),
            config.carrier.density,
            workspace_limit,
            implementation,
        )
        .map_err(SessionError::Workspace)?;
        let simulation = LiquidTransportSimulation::new(grid, config, implementation, fraction)
            .map_err(SessionError::Step)?;
        Ok(Self {
            identity,
            configuration: SessionConfiguration {
                transport: config,
                implementation,
                boundary_workspace_limit: workspace_limit,
                max_steps,
            },
            interval_start: None,
            simulation,
            workspace,
            report: None,
            provider: None,
            configured: None,
            pressure_history: None,
            steps: 0,
            max_steps,
            stopped: false,
            projection_intent: None,
            projection_attempt: None,
            projection_attempt_counter: 0,
        })
    }
    pub fn capabilities(&self) -> SessionCapabilities {
        SESSION_CAPABILITIES
    }
    pub fn stop(&mut self) {
        self.stopped = true;
    }
    pub fn is_stopped(&self) -> bool {
        self.stopped
    }
    pub fn snapshot(&self) -> SessionSnapshot<'_> {
        SessionSnapshot {
            identity: &self.identity,
            configuration: &self.configuration,
            interval_start: self.interval_start,
            grid: self.simulation.grid(),
            state: self.simulation.state(),
            report: self.report.as_ref(),
            provider: self.provider.as_ref(),
            configured: self.configured.as_ref(),
            frame: self.steps,
            projection_intent: self.projection_intent,
            projection_attempt: self.projection_attempt.as_ref(),
        }
    }
    /// Vec-capacity payload only, excluding caller allocations, identity, object
    /// stack storage and allocator overhead; this is not process RSS.
    pub fn allocated_array_bytes(&self) -> usize {
        self.simulation.allocated_bytes() + self.workspace.allocated_bytes()
    }
    /// One transactional interval using the existing controls. Requested dt may
    /// be shortened: consumers must advance clocks using report.carrier.step.dt.
    /// Deadline is checked before work and at existing cancellation checkpoints;
    /// kernels and user callbacks cannot be preempted. No wall-time bound is promised.
    pub fn step<F: FnMut(LiquidStepStage) -> bool>(
        &mut self,
        inputs: LiquidStepInputs<'_>,
        boundary: BoxFluxStepBoundary,
        deadline: Option<Instant>,
        cancel: F,
    ) -> Result<LiquidStepReport, SessionError> {
        self.step_with_clock(inputs, boundary, deadline, cancel, Instant::now)
    }

    /// Native sealed-box interval with explicit numerical initialization.
    /// AcceptedPreviousV1 borrows only the preceding accepted liquid pressure.
    /// Provider and prescribed-boundary entry points remain cold. Existing dense3D
    /// snapshots do not serialize the history needed to resume this policy.
    #[cfg(feature = "accepted-pressure-warm-start")]
    pub fn step_with_initialization<F: FnMut(LiquidStepStage) -> bool>(
        &mut self,
        inputs: LiquidStepInputs<'_>,
        policy: crate::PressureInitializationPolicy,
        deadline: Option<Instant>,
        cancel: F,
    ) -> Result<LiquidStepReport, SessionError> {
        self.initialized_step_with_clock(inputs, policy, deadline, cancel, Instant::now)
    }

    #[cfg(feature = "accepted-pressure-warm-start")]
    fn initialized_step_with_clock<F: FnMut(LiquidStepStage) -> bool, N: FnMut() -> Instant>(
        &mut self,
        inputs: LiquidStepInputs<'_>,
        policy: crate::PressureInitializationPolicy,
        deadline: Option<Instant>,
        mut cancel: F,
        mut now: N,
    ) -> Result<LiquidStepReport, SessionError> {
        self.require_legacy_intent()?;
        if self.stopped {
            return Err(SessionError::Stopped);
        }
        if self.steps >= self.max_steps {
            return Err(SessionError::StepLimit);
        }
        if deadline.is_some_and(|d| now() >= d) {
            return Err(SessionError::Deadline);
        }
        let interval_start = self.simulation.state().carrier.time;
        let mut expired = false;
        let result = self
            .simulation
            .step_with_initialization(inputs, policy, |stage| {
                expired = deadline.is_some_and(|d| now() >= d);
                if expired {
                    return true;
                }
                let cancelled = cancel(stage);
                expired = deadline.is_some_and(|d| now() >= d);
                expired || cancelled
            });
        let report = result.map_err(|e| {
            if expired {
                SessionError::Deadline
            } else {
                SessionError::Step(e)
            }
        })?;
        self.interval_start = Some(interval_start);
        self.steps += 1;
        self.report = Some(report);
        self.provider = None;
        self.configured = None;
        self.pressure_history = None;
        Ok(report)
    }

    /// Full sealed-box interval with explicit preferred-provider/reference
    /// selection. Caller owns separately capped scratch and model; deadlines are
    /// cooperative through the same final publication checkpoint as legacy steps.
    /// None selects the numerical reference, Some permits its explicit fallback.
    pub fn step_with_pressure_provider<F: FnMut(LiquidStepStage) -> bool>(
        &mut self,
        inputs: LiquidStepInputs<'_>,
        workspace: &mut crate::ProviderStepWorkspace,
        preferred: Option<&mut dyn crate::solver_provider::PressureProvider>,
        deadline: Option<Instant>,
        cancel: F,
    ) -> Result<(LiquidStepReport, crate::solver_provider::ProviderReport), SessionError> {
        self.provider_step_with_clock(inputs, workspace, preferred, deadline, cancel, Instant::now)
    }

    fn provider_step_with_clock<F: FnMut(LiquidStepStage) -> bool, N: FnMut() -> Instant>(
        &mut self,
        inputs: LiquidStepInputs<'_>,
        workspace: &mut crate::ProviderStepWorkspace,
        preferred: Option<&mut dyn crate::solver_provider::PressureProvider>,
        deadline: Option<Instant>,
        mut cancel: F,
        mut now: N,
    ) -> Result<(LiquidStepReport, crate::solver_provider::ProviderReport), SessionError> {
        self.require_legacy_intent()?;
        if self.stopped {
            return Err(SessionError::Stopped);
        }
        if self.steps >= self.max_steps {
            return Err(SessionError::StepLimit);
        }
        if deadline.is_some_and(|d| now() >= d) {
            return Err(SessionError::Deadline);
        }
        let interval_start = self.simulation.state().carrier.time;
        let mut expired = false;
        let result =
            self.simulation
                .step_with_pressure_provider(inputs, workspace, preferred, |stage| {
                    expired = deadline.is_some_and(|d| now() >= d);
                    if expired {
                        return true;
                    }
                    let cancelled = cancel(stage);
                    expired = deadline.is_some_and(|d| now() >= d);
                    expired || cancelled
                });
        let (report, provider) = result.map_err(|e| {
            if expired {
                SessionError::Deadline
            } else {
                SessionError::Step(e)
            }
        })?;
        self.interval_start = Some(interval_start);
        self.steps += 1;
        self.report = Some(report);
        self.provider = Some(provider.clone());
        self.configured = None;
        self.accept_pressure_history(report, &provider, 0.0);
        Ok((report, provider))
    }

    fn step_with_clock<F: FnMut(LiquidStepStage) -> bool, N: FnMut() -> Instant>(
        &mut self,
        inputs: LiquidStepInputs<'_>,
        boundary: BoxFluxStepBoundary,
        deadline: Option<Instant>,
        mut cancel: F,
        mut now: N,
    ) -> Result<LiquidStepReport, SessionError> {
        self.require_legacy_intent()?;
        if self.stopped {
            return Err(SessionError::Stopped);
        }
        if self.steps >= self.max_steps {
            return Err(SessionError::StepLimit);
        }
        if deadline.is_some_and(|d| now() >= d) {
            return Err(SessionError::Deadline);
        }
        let interval_start = self.simulation.state().carrier.time;
        let mut expired = false;
        let result =
            self.simulation
                .step_with_box_flux(inputs, &mut self.workspace, boundary, |stage| {
                    expired = deadline.is_some_and(|d| now() >= d);
                    if expired {
                        return true;
                    }
                    let cancelled = cancel(stage);
                    expired = deadline.is_some_and(|d| now() >= d);
                    expired || cancelled
                });
        let report = result.map_err(|e| {
            if expired {
                SessionError::Deadline
            } else {
                SessionError::Step(e)
            }
        })?;
        self.interval_start = Some(interval_start);
        self.steps += 1;
        self.report = Some(report);
        self.provider = None;
        self.configured = None;
        self.pressure_history = None;
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use std::{cell::Cell, time::Duration};

    #[test]
    fn deadline_crossed_in_final_callback_preserves_publication_and_retries() {
        let grid = GridGeometry::new([2, 2, 2], [0.5; 3], [0.0; 3]).unwrap();
        let config = LiquidTransportConfig {
            carrier: SimulationConfig {
                density: 1000.0,
                memory_limit: 1024 * 1024,
                pressure: PressureSettings {
                    relative_residual: 1e-12,
                    absolute_residual: 1e-12,
                    divergence_limit: 1e-9,
                    max_iterations: 1000,
                },
                actual_divergence_limit: 1e-5,
                max_courant: 1.0,
            },
            represented_density: 800.0,
            volume_stamp: VolumeStamp { id: 1, version: 0 },
            carrier_id: 2,
        };
        let mut session = SimulationSession::new(
            SessionIdentity {
                run_id: 1,
                source_revision: "authored-clock-test".into(),
            },
            grid,
            config,
            PressureImplementation::JacobiPcgV1,
            vec![0.5; 8],
            1024 * 1024,
            8,
        )
        .unwrap();
        let inputs = LiquidStepInputs {
            requested_dt: 0.0625,
            smoke_source: None,
            forces: &[],
            source: None,
            inlet: LiquidInlet::new(VolumeStamp { id: 3, version: 0 }, [[0.0; 2]; 3]).unwrap(),
            volume: LiquidVolumeSettings {
                max_outward_courant: 1.0,
                actual_divergence_limit: 1e-5,
            },
        };
        let boundary = BoxFluxStepBoundary {
            flux: PrescribedBoxFlux::new(BoxFluxStamp { id: 4, version: 0 }, [[0.0; 2]; 3])
                .unwrap(),
            tracer: BoxFluxTracerPolicy::ClampedAppearance,
        };
        session.step(inputs, boundary, None, |_| false).unwrap();
        let mut before = Vec::new();
        session.snapshot().write_dense3d(&mut before).unwrap();
        let start = Instant::now();
        let deadline = start + Duration::from_secs(1);
        let clock = Cell::new(start);
        let reached = Cell::new(false);
        let result = session.step_with_clock(
            inputs,
            boundary,
            Some(deadline),
            |stage| {
                if stage == LiquidStepStage::BeforeCommit {
                    reached.set(true);
                    clock.set(deadline);
                }
                false
            },
            || clock.get(),
        );
        assert!(reached.get());
        assert!(matches!(result, Err(SessionError::Deadline)));
        let mut after = Vec::new();
        session.snapshot().write_dense3d(&mut after).unwrap();
        assert_eq!(before, after);
        session.step(inputs, boundary, None, |_| false).unwrap();
        assert_eq!(session.snapshot().frame(), 2);

        let mut workspace = crate::ProviderStepWorkspace::new(
            session.snapshot().grid(),
            1000.0,
            1024 * 1024,
            PressureImplementation::JacobiPcgV1,
        )
        .unwrap();
        session
            .step_with_pressure_provider(inputs, &mut workspace, None, None, |_| false)
            .unwrap();
        let previous_provider = session.snapshot().provider_report().unwrap().total;
        before.clear();
        session.snapshot().write_dense3d(&mut before).unwrap();
        clock.set(start);
        reached.set(false);
        let result = session.provider_step_with_clock(
            inputs,
            &mut workspace,
            None,
            Some(deadline),
            |stage| {
                if stage == LiquidStepStage::BeforeCommit {
                    reached.set(true);
                    clock.set(deadline);
                }
                false
            },
            || clock.get(),
        );
        assert!(reached.get());
        assert!(matches!(result, Err(SessionError::Deadline)));
        after.clear();
        session.snapshot().write_dense3d(&mut after).unwrap();
        assert_eq!(before, after);
        assert_eq!(
            session.snapshot().provider_report().unwrap().total,
            previous_provider
        );
        session
            .step_with_pressure_provider(inputs, &mut workspace, None, None, |_| false)
            .unwrap();
        assert_eq!(session.snapshot().frame(), 4);

        #[cfg(feature = "accepted-pressure-warm-start")]
        {
            let policy = PressureInitializationPolicy::AcceptedPreviousV1;
            session
                .step_with_initialization(inputs, policy, None, |_| false)
                .unwrap();
            let seed = session.simulation.pressure_seed();
            let initialization = session.snapshot().report().unwrap().pressure_initialization;
            before.clear();
            session.snapshot().write_dense3d(&mut before).unwrap();
            clock.set(start);
            reached.set(false);
            let result = session.initialized_step_with_clock(
                inputs,
                policy,
                Some(deadline),
                |stage| {
                    if stage == LiquidStepStage::BeforeCommit {
                        reached.set(true);
                        clock.set(deadline);
                    }
                    false
                },
                || clock.get(),
            );
            assert!(reached.get());
            assert!(matches!(result, Err(SessionError::Deadline)));
            after.clear();
            session.snapshot().write_dense3d(&mut after).unwrap();
            assert_eq!(before, after);
            assert_eq!(session.simulation.pressure_seed(), seed);
            assert_eq!(
                session.snapshot().report().unwrap().pressure_initialization,
                initialization
            );
            assert!(session.snapshot().provider_report().is_none());
            let report = session
                .step_with_initialization(inputs, policy, None, |_| false)
                .unwrap();
            assert_eq!(report.pressure_initialization.seed, seed);
            assert_eq!(session.snapshot().frame(), 6);
        }
    }
}
