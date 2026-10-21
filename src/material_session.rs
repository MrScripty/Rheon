//! Synchronous session ownership for the existing SPH preview model.
use crate::{
    FluidParticle, GridGeometry, ParticleDiagnostic, ParticleFluidConfig, ParticleFluidError,
    ParticleFluidSimulation, ParticleStepReport, RotatingCylinderEmitter, SessionIdentity,
    SnapshotStage,
};
use std::{fmt, time::Instant};

/// Caller-declared source identity within a run; parameters are the actual inputs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaterialEmitterControl {
    pub source_id: u64,
    pub source_version: u64,
    pub emitter: RotatingCylinderEmitter,
}

/// Per-call controls. The accepted report records the actual (possibly shorter) dt.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaterialStepControls {
    pub requested_dt: f64,
    pub emitter: Option<MaterialEmitterControl>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleEvaluationStage {
    /// Includes emitted particles and pair/gravity acceleration, excludes later
    /// integration and wall velocity/temperature response.
    PostEmissionPreIntegration,
}

#[derive(Debug, Clone, Copy)]
pub struct MaterialSessionCapabilities {
    pub api_version: u32,
    pub emitter_controls_version: u32,
    pub crate_version: &'static str,
    pub particle_diagnostics: bool,
    /// This SPH model does not call the grid pressure provider.
    pub grid_pressure_provider: bool,
    pub cooperative_cancellation: bool,
    pub hard_realtime: bool,
    pub units: &'static [(&'static str, &'static str)],
}

pub const MATERIAL_SESSION_CAPABILITIES: MaterialSessionCapabilities =
    MaterialSessionCapabilities {
        api_version: 2,
        emitter_controls_version: crate::EMITTER_CONTROLS_VERSION,
        crate_version: env!("CARGO_PKG_VERSION"),
        particle_diagnostics: true,
        grid_pressure_provider: false,
        cooperative_cancellation: true,
        hard_realtime: false,
        units: &[
            ("position", "m"),
            ("time", "s"),
            ("velocity", "m/s"),
            ("mass", "kg"),
            ("temperature", "K"),
            ("density", "kg/m^3"),
            ("pressure", "Pa"),
            ("acceleration", "m/s^2"),
        ],
    };

/// Borrowed immutable publication. Particle identity is (run_id, index); indices
/// are stable only within this append-only simulation's lifetime.
/// Inspection is not replay: no source history or archived snapshots are owned.
pub struct MaterialSnapshot<'a> {
    identity: &'a SessionIdentity,
    simulation: &'a ParticleFluidSimulation,
    report: Option<&'a ParticleStepReport>,
    controls: Option<MaterialStepControls>,
}
impl<'a> MaterialSnapshot<'a> {
    pub fn identity(&self) -> &'a SessionIdentity {
        self.identity
    }
    pub fn grid(&self) -> &'a GridGeometry {
        self.simulation.grid()
    }
    pub fn config(&self) -> ParticleFluidConfig {
        self.simulation.config()
    }
    pub fn particles(&self) -> &'a [FluidParticle] {
        self.simulation.particles()
    }
    pub fn time(&self) -> f64 {
        self.simulation.time()
    }
    pub fn generation(&self) -> u64 {
        self.simulation.generation()
    }
    pub fn stage(&self) -> SnapshotStage {
        if self.report.is_some() {
            SnapshotStage::Accepted
        } else {
            SnapshotStage::Constructor
        }
    }
    pub fn report(&self) -> Option<&'a ParticleStepReport> {
        self.report
    }
    /// Controls of the last accepted call; absent at construction. Rejected
    /// requests never relabel prior state. Source identity is a caller declaration.
    pub fn accepted_controls(&self) -> Option<MaterialStepControls> {
        self.controls
    }
    /// Reuses the solver's canonical capture. Returned time is evaluation time,
    /// not accepted position time. None before the first accepted step.
    pub fn particle_diagnostics(&self) -> Option<(f64, &'a [ParticleDiagnostic])> {
        self.simulation.particle_diagnostics()
    }
    pub fn diagnostic_stage(&self) -> Option<ParticleEvaluationStage> {
        self.simulation
            .particle_diagnostics()
            .map(|_| ParticleEvaluationStage::PostEmissionPreIntegration)
    }
}

#[derive(Debug)]
pub enum MaterialSessionError {
    InvalidIdentity,
    Stopped,
    StepLimit,
    Deadline,
    Physics(ParticleFluidError),
}
impl fmt::Display for MaterialSessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "material session: {self:?}")
    }
}
impl std::error::Error for MaterialSessionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Physics(error) => Some(error),
            _ => None,
        }
    }
}

/// Owns a fixed material configuration and bounded accepted-call quota. Emitter
/// controls may change each call. Recreate with a new run identity to change
/// configuration. Drop releases resources; stop preserves inspection but is terminal.
pub struct MaterialSession {
    identity: SessionIdentity,
    simulation: ParticleFluidSimulation,
    report: Option<ParticleStepReport>,
    controls: Option<MaterialStepControls>,
    steps: usize,
    max_steps: usize,
    stopped: bool,
}
impl MaterialSession {
    pub fn new(
        identity: SessionIdentity,
        grid: GridGeometry,
        config: ParticleFluidConfig,
        max_steps: usize,
    ) -> Result<Self, MaterialSessionError> {
        if !identity.is_valid() {
            return Err(MaterialSessionError::InvalidIdentity);
        }
        let simulation =
            ParticleFluidSimulation::new(grid, config).map_err(MaterialSessionError::Physics)?;
        Ok(Self {
            identity,
            simulation,
            report: None,
            controls: None,
            steps: 0,
            max_steps,
            stopped: false,
        })
    }
    pub fn capabilities(&self) -> MaterialSessionCapabilities {
        MATERIAL_SESSION_CAPABILITIES
    }
    pub fn snapshot(&self) -> MaterialSnapshot<'_> {
        MaterialSnapshot {
            identity: &self.identity,
            simulation: &self.simulation,
            report: self.report.as_ref(),
            controls: self.controls,
        }
    }
    pub fn stop(&mut self) {
        self.stopped = true;
    }
    pub fn is_stopped(&self) -> bool {
        self.stopped
    }
    /// Solver-owned Vec-capacity payload, including diagnostic capture. Excludes
    /// caller allocations, identity, stack, allocator overhead and process RSS.
    pub fn allocated_array_bytes(&self) -> usize {
        self.simulation.allocated_bytes()
    }
    /// One existing bounded SPH step (up to its 16 stability retries). Successful
    /// dt may be shorter than requested. Deadline checks surround synchronous
    /// cancellation callbacks; kernels cannot be preempted. No wall-time guarantee.
    pub fn step(
        &mut self,
        controls: MaterialStepControls,
        deadline: Option<Instant>,
        cancel: impl FnMut() -> bool,
    ) -> Result<ParticleStepReport, MaterialSessionError> {
        self.step_with_clock(controls, deadline, cancel, Instant::now)
    }
    fn step_with_clock(
        &mut self,
        controls: MaterialStepControls,
        deadline: Option<Instant>,
        mut cancel: impl FnMut() -> bool,
        mut now: impl FnMut() -> Instant,
    ) -> Result<ParticleStepReport, MaterialSessionError> {
        if self.stopped {
            return Err(MaterialSessionError::Stopped);
        }
        if self.steps >= self.max_steps {
            return Err(MaterialSessionError::StepLimit);
        }
        if deadline.is_some_and(|d| now() >= d) {
            return Err(MaterialSessionError::Deadline);
        }
        let mut expired = false;
        let report = self
            .simulation
            .step(
                controls.requested_dt,
                controls.emitter.map(|c| c.emitter),
                || {
                    expired = deadline.is_some_and(|d| now() >= d);
                    if expired {
                        return true;
                    }
                    let cancelled = cancel();
                    expired = deadline.is_some_and(|d| now() >= d);
                    expired || cancelled
                },
            )
            .map_err(|e| {
                if expired {
                    MaterialSessionError::Deadline
                } else {
                    MaterialSessionError::Physics(e)
                }
            })?;
        self.steps += 1;
        self.report = Some(report);
        self.controls = Some(controls);
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::Cell, time::Duration};
    #[test]
    fn final_callback_deadline_retains_particles_diagnostics_and_controls() {
        let grid = GridGeometry::new([4; 3], [0.25; 3], [0.0; 3]).unwrap();
        let config = ParticleFluidConfig {
            samples_per_axis: 4,
            max_particles: 128,
            memory_limit: 1024 * 1024,
            fill: crate::DomainFill::EmittedFluid,
            ..Default::default()
        };
        let mut session = MaterialSession::new(
            SessionIdentity {
                run_id: 1,
                source_revision: "authored-clock-test".into(),
            },
            grid,
            config,
            4,
        )
        .unwrap();
        let controls = MaterialStepControls {
            requested_dt: 0.001,
            emitter: None,
        };
        let mut checkpoints = 0;
        session
            .step(controls, None, || {
                checkpoints += 1;
                false
            })
            .unwrap();
        let particles = session.snapshot().particles().to_vec();
        let (time, diagnostics) = session.snapshot().particle_diagnostics().unwrap();
        let diagnostics = diagnostics.to_vec();
        let start = Instant::now();
        let deadline = start + Duration::from_secs(1);
        let clock = Cell::new(start);
        let mut visits = 0;
        let result = session.step_with_clock(
            controls,
            Some(deadline),
            || {
                visits += 1;
                if visits == checkpoints {
                    clock.set(deadline);
                }
                false
            },
            || clock.get(),
        );
        assert_eq!(visits, checkpoints);
        assert!(matches!(result, Err(MaterialSessionError::Deadline)));
        let snapshot = session.snapshot();
        assert_eq!(snapshot.particles(), particles);
        assert_eq!(
            snapshot.particle_diagnostics(),
            Some((time, diagnostics.as_slice()))
        );
        assert_eq!(snapshot.accepted_controls(), Some(controls));
        assert_eq!(snapshot.generation(), 1);
        session.step(controls, None, || false).unwrap();
        assert_eq!(session.snapshot().generation(), 2);
    }
}
