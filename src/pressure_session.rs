//! Publication ownership for qualified sealed-box pressure stages, not full steps.
use crate::solver_provider::{
    PressureProvider, PressureRequest, ProviderReport, ProviderWorkspace,
};
use crate::{
    GridGeometry, PressureError, PressureImplementation, PressureSettings, SessionIdentity,
};
use std::{fmt, time::Instant};

/// Caller identifies the RHS source; this declaration does not retain or replay it.
#[derive(Debug, Clone, Copy)]
pub struct PressureSolveControls {
    pub source_id: u64,
    pub source_version: u64,
    /// Constant carrier density, kg/m³.
    pub density: f64,
    /// Pressure interval duration, seconds; not an accumulated simulation clock.
    pub dt: f64,
    pub settings: PressureSettings,
}

/// Qualified pressure only. A full simulation must still correct velocity and
/// accept its actual f32 divergence before publishing a simulation state.
pub struct QualifiedPressureSnapshot<'a> {
    identity: &'a SessionIdentity,
    grid: &'a GridGeometry,
    generation: usize,
    pressure: &'a [f64],
    controls: PressureSolveControls,
    report: &'a ProviderReport,
}
impl<'a> QualifiedPressureSnapshot<'a> {
    pub fn identity(&self) -> &'a SessionIdentity {
        self.identity
    }
    pub fn grid(&self) -> &'a GridGeometry {
        self.grid
    }
    pub fn generation(&self) -> usize {
        self.generation
    }
    pub fn pressure_pa(&self) -> &'a [f64] {
        self.pressure
    }
    pub fn controls(&self) -> PressureSolveControls {
        self.controls
    }
    pub fn report(&self) -> &'a ProviderReport {
        self.report
    }
    pub fn stage(&self) -> &'static str {
        "qualified-sealed-box-pressure"
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PressureSessionCapabilities {
    pub api_version: u32,
    pub crate_version: &'static str,
    pub stage: &'static str,
    pub layout: &'static str,
    pub pressure_units: &'static str,
    pub rhs_units: &'static str,
    pub full_step_acceptance: bool,
    pub hard_realtime: bool,
}
pub const PRESSURE_SESSION_CAPABILITIES: PressureSessionCapabilities =
    PressureSessionCapabilities {
        api_version: 1,
        crate_version: env!("CARGO_PKG_VERSION"),
        stage: "qualified-sealed-box-pressure",
        layout: "x-fastest cell-centered f64",
        pressure_units: "Pa",
        rhs_units: "m^3/s^2",
        full_step_acceptance: false,
        hard_realtime: false,
    };

#[derive(Debug)]
pub enum PressureSessionError {
    InvalidIdentity,
    Stopped,
    SolveLimit,
    Deadline,
    Pressure(PressureError),
}
impl fmt::Display for PressureSessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pressure session: {self:?}")
    }
}
impl std::error::Error for PressureSessionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Pressure(e) => Some(e),
            _ => None,
        }
    }
}

/// Owns workspace plus one published pressure copy, bounded by a combined Vec
/// payload limit. The preferred provider is caller-owned and borrowed per call.
/// Its model memory and side effects are outside this owner's transaction/budget.
pub struct PressureSession {
    identity: SessionIdentity,
    grid: GridGeometry,
    workspace: ProviderWorkspace,
    pressure: Vec<f64>,
    accepted: Option<(PressureSolveControls, ProviderReport)>,
    solves: usize,
    max_solves: usize,
    stopped: bool,
}
impl PressureSession {
    pub fn new(
        identity: SessionIdentity,
        grid: GridGeometry,
        memory_limit: usize,
        fallback: PressureImplementation,
        max_solves: usize,
    ) -> Result<Self, PressureSessionError> {
        if !identity.is_valid() {
            return Err(PressureSessionError::InvalidIdentity);
        }
        let bytes = grid
            .cell_len()
            .checked_mul(std::mem::size_of::<f64>())
            .ok_or(PressureSessionError::Pressure(
                PressureError::AllocationFailed,
            ))?;
        let limit_error = |required| {
            PressureSessionError::Pressure(PressureError::BufferLimit {
                required,
                limit: memory_limit,
            })
        };
        if bytes > memory_limit {
            return Err(limit_error(bytes));
        }
        let mut pressure = Vec::new();
        pressure
            .try_reserve_exact(grid.cell_len())
            .map_err(|_| PressureSessionError::Pressure(PressureError::AllocationFailed))?;
        let actual = pressure
            .capacity()
            .checked_mul(8)
            .ok_or(PressureSessionError::Pressure(
                PressureError::AllocationFailed,
            ))?;
        if actual > memory_limit {
            return Err(limit_error(actual));
        }
        pressure.resize(grid.cell_len(), 0.0);
        let workspace = ProviderWorkspace::new(&grid, memory_limit - actual, fallback)
            .map_err(PressureSessionError::Pressure)?;
        Ok(Self {
            identity,
            grid,
            workspace,
            pressure,
            accepted: None,
            solves: 0,
            max_solves,
            stopped: false,
        })
    }
    pub fn capabilities(&self) -> PressureSessionCapabilities {
        PRESSURE_SESSION_CAPABILITIES
    }
    /// No qualified state exists at construction. Rejected solves preserve this
    /// publication even though the underlying workspace's candidate is invalidated.
    pub fn snapshot(&self) -> Option<QualifiedPressureSnapshot<'_>> {
        self.accepted
            .as_ref()
            .map(|(controls, report)| QualifiedPressureSnapshot {
                identity: &self.identity,
                grid: &self.grid,
                generation: self.solves,
                pressure: &self.pressure,
                controls: *controls,
                report,
            })
    }
    pub fn stop(&mut self) {
        self.stopped = true;
    }
    pub fn is_stopped(&self) -> bool {
        self.stopped
    }
    pub fn allocated_array_bytes(&self) -> usize {
        self.workspace.allocated_bytes() + self.pressure.capacity() * 8
    }
    /// None selects the configured numerical reference. Some explicitly requests
    /// the provider with that reference as fallback; the report preserves attempted
    /// metadata/provenance, rejection, selected implementation and stage timings.
    /// Cancellation aborts, never falls back. Panics/hung providers are not isolated.
    pub fn solve(
        &mut self,
        rhs: &[f64],
        controls: PressureSolveControls,
        preferred: Option<&mut dyn PressureProvider>,
        deadline: Option<Instant>,
        cancel: impl FnMut() -> bool,
    ) -> Result<ProviderReport, PressureSessionError> {
        self.solve_with_clock(rhs, controls, preferred, deadline, cancel, Instant::now)
    }
    fn solve_with_clock(
        &mut self,
        rhs: &[f64],
        controls: PressureSolveControls,
        preferred: Option<&mut dyn PressureProvider>,
        deadline: Option<Instant>,
        mut cancel: impl FnMut() -> bool,
        mut now: impl FnMut() -> Instant,
    ) -> Result<ProviderReport, PressureSessionError> {
        if self.stopped {
            return Err(PressureSessionError::Stopped);
        }
        if self.solves >= self.max_solves {
            return Err(PressureSessionError::SolveLimit);
        }
        if deadline.is_some_and(|d| now() >= d) {
            return Err(PressureSessionError::Deadline);
        }
        let mut expired = false;
        let result = self.workspace.solve(
            PressureRequest {
                grid: &self.grid,
                density: controls.density,
                dt: controls.dt,
                rhs,
            },
            controls.settings,
            preferred,
            || {
                expired = deadline.is_some_and(|d| now() >= d);
                if expired {
                    return true;
                }
                let cancelled = cancel();
                expired = deadline.is_some_and(|d| now() >= d);
                expired || cancelled
            },
        );
        let report = result.map_err(|e| {
            if expired {
                PressureSessionError::Deadline
            } else {
                PressureSessionError::Pressure(e)
            }
        })?;
        // A numerical fallback can finish after its last inner callback. This
        // final checkpoint prevents the session from publishing an observed expiry.
        if deadline.is_some_and(|d| now() >= d) {
            return Err(PressureSessionError::Deadline);
        }
        let cancelled = cancel();
        if deadline.is_some_and(|d| now() >= d) {
            return Err(PressureSessionError::Deadline);
        }
        if cancelled {
            return Err(PressureSessionError::Pressure(PressureError::Cancelled {
                iterations: report.accepted.iterations,
            }));
        }
        self.pressure.copy_from_slice(self.workspace.pressure());
        self.solves += 1;
        self.accepted = Some((controls, report.clone()));
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::Cell, time::Duration};
    #[test]
    fn deadline_during_final_callback_preserves_qualified_pressure_and_retry() {
        let grid = GridGeometry::new([2; 3], [0.5; 3], [0.0; 3]).unwrap();
        let mut session = PressureSession::new(
            SessionIdentity {
                run_id: 1,
                source_revision: "authored-clock-test".into(),
            },
            grid,
            1024 * 1024,
            PressureImplementation::JacobiPcgV1,
            3,
        )
        .unwrap();
        let controls = PressureSolveControls {
            source_id: 1,
            source_version: 0,
            density: 1.25,
            dt: 0.01,
            settings: PressureSettings {
                relative_residual: 1e-8,
                absolute_residual: 1e-12,
                divergence_limit: 1e-7,
                max_iterations: 100,
            },
        };
        let rhs = [0.0; 8];
        let mut checkpoints = 0;
        session
            .solve(&rhs, controls, None, None, || {
                checkpoints += 1;
                false
            })
            .unwrap();
        let prior = session.snapshot().unwrap().pressure_pa().to_vec();
        let start = Instant::now();
        let deadline = start + Duration::from_secs(1);
        let clock = Cell::new(start);
        let mut visits = 0;
        let result = session.solve_with_clock(
            &rhs,
            controls,
            None,
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
        assert!(matches!(result, Err(PressureSessionError::Deadline)));
        assert_eq!(session.snapshot().unwrap().pressure_pa(), prior);
        assert_eq!(session.snapshot().unwrap().generation(), 1);
        session.solve(&rhs, controls, None, None, || false).unwrap();
        assert_eq!(session.snapshot().unwrap().generation(), 2);
    }
}
