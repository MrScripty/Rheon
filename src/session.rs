//! In-process full-box transport API. No worker threads or network services.
use crate::{
    BoxFluxStepBoundary, BoxFluxStepWorkspace, Dense3dError, GridGeometry, LiquidStepError,
    LiquidStepInputs, LiquidStepReport, LiquidStepStage, LiquidTransportConfig,
    LiquidTransportSimulation, LiquidTransportView, PressureImplementation, write_dense3d_frame,
};
use std::{fmt, io::Write, time::Instant};

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
    pub cooperative_cancellation: bool,
    pub hard_realtime: bool,
    pub units: &'static [(&'static str, &'static str)],
}

pub const SESSION_CAPABILITIES: SessionCapabilities = SessionCapabilities {
    api_version: 1,
    crate_version: env!("CARGO_PKG_VERSION"),
    transport_only: true,
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

/// Immutable borrow of one publication. Rust prevents stepping while it is used.
/// Pressure is the last interval projection pressure, not end-surface pressure.
/// Copy fields explicitly if they must outlive the next step; that storage belongs
/// to the consumer. There is no hidden snapshot history or unbounded queue.
pub struct SessionSnapshot<'a> {
    identity: &'a SessionIdentity,
    grid: &'a GridGeometry,
    state: LiquidTransportView<'a>,
    report: Option<&'a LiquidStepReport>,
    frame: usize,
}
impl<'a> SessionSnapshot<'a> {
    pub fn identity(&self) -> &'a SessionIdentity {
        self.identity
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
    /// manifest or v2 controls. Writer failures do not modify simulation state.
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
    simulation: LiquidTransportSimulation,
    workspace: BoxFluxStepWorkspace,
    report: Option<LiquidStepReport>,
    steps: usize,
    max_steps: usize,
    stopped: bool,
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
            simulation,
            workspace,
            report: None,
            steps: 0,
            max_steps,
            stopped: false,
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
            grid: self.simulation.grid(),
            state: self.simulation.state(),
            report: self.report.as_ref(),
            frame: self.steps,
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

    fn step_with_clock<F: FnMut(LiquidStepStage) -> bool, N: FnMut() -> Instant>(
        &mut self,
        inputs: LiquidStepInputs<'_>,
        boundary: BoxFluxStepBoundary,
        deadline: Option<Instant>,
        mut cancel: F,
        mut now: N,
    ) -> Result<LiquidStepReport, SessionError> {
        if self.stopped {
            return Err(SessionError::Stopped);
        }
        if self.steps >= self.max_steps {
            return Err(SessionError::StepLimit);
        }
        if deadline.is_some_and(|d| now() >= d) {
            return Err(SessionError::Deadline);
        }
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
        self.steps += 1;
        self.report = Some(report);
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
    }
}
