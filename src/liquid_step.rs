//! Transactional fixed-box carrier and represented-volume transport. Fractions
//! do not classify pressure unknowns or feed back into carrier inertia.
use crate::{
    Axis, BodyForce, BoxFluxStepBoundary, BoxFluxStepReport, BoxFluxStepWorkspace, BufferPlan,
    ForcedStepReport, GridGeometry, LiquidFlowInterval, LiquidInlet, LiquidVolumeError,
    LiquidVolumeReport, LiquidVolumeSettings, LiquidVolumeSource, LiquidVolumeState,
    LiquidVolumeView, PressureImplementation, Simulation, SimulationConfig, SimulationError,
    SmokeSource, StateView, StepStage, VolumeStage, VolumeStamp,
};
use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct LiquidTransportConfig {
    /// memory_limit caps all facade-owned arrays, including accepted pressure
    /// and transferred initial fraction capacity. Boundary scratch is separate.
    pub carrier: SimulationConfig,
    /// Independent constant represented-liquid density; no two-phase inertia.
    pub represented_density: f64,
    pub volume_stamp: VolumeStamp,
    /// Caller-owned identity for this carrier's published generations.
    pub carrier_id: u64,
}

#[derive(Clone, Copy)]
pub struct LiquidStepInputs<'a> {
    pub requested_dt: f64,
    pub smoke_source: Option<SmokeSource>,
    pub forces: &'a [BodyForce],
    pub inlet: LiquidInlet,
    pub source: Option<LiquidVolumeSource<'a>>,
    pub volume: LiquidVolumeSettings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiquidStepStage {
    Carrier(StepStage),
    Volume(VolumeStage),
    /// Last callback; neither owner has published yet. No callbacks or fallible
    /// operations run after this checkpoint accepts the candidate pair.
    BeforeCommit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiquidStepError {
    Carrier(SimulationError),
    Volume(LiquidVolumeError),
    Cancelled,
    AllocationFailed,
    BufferLimit { required: usize, limit: usize },
}
impl fmt::Display for LiquidStepError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "coupled liquid transport rejected operation: {self:?}")
    }
}
impl std::error::Error for LiquidStepError {}
impl From<SimulationError> for LiquidStepError {
    fn from(e: SimulationError) -> Self {
        Self::Carrier(e)
    }
}
impl From<LiquidVolumeError> for LiquidStepError {
    fn from(e: LiquidVolumeError) -> Self {
        Self::Volume(e)
    }
}

#[derive(Clone, Copy)]
pub struct LiquidTransportView<'a> {
    pub carrier: StateView<'a>,
    pub liquid: LiquidVolumeView<'a>,
    /// Gauge-fixed pressure of the published carrier projection, in Pa. Initial
    /// rest state uses zero pressure. Failed preparation cannot change this view.
    pub pressure: &'a [f64],
    pub carrier_stamp: VolumeStamp,
}

#[derive(Debug, Clone, Copy)]
pub struct LiquidStepReport {
    pub carrier: ForcedStepReport,
    pub boundary: Option<BoxFluxStepReport>,
    pub liquid: LiquidVolumeReport,
    pub owned_array_bytes: usize,
    pub boundary_workspace_array_bytes: usize,
    pub total_array_bytes: usize,
}

/// Owns the existing Simulation and LiquidVolumeState and one explicitly
/// counted accepted-pressure array. No mutable access to either accepted owner
/// is exposed. Stepping reuses candidates and scratch; no rollback snapshots.
/// The optional borrowed boundary workspace retains its own capacity cap.
pub struct LiquidTransportSimulation {
    carrier: Simulation,
    liquid: LiquidVolumeState,
    pressure: Vec<f64>,
    carrier_id: u64,
    allocated_bytes: usize,
}
impl LiquidTransportSimulation {
    pub fn new(
        grid: GridGeometry,
        config: LiquidTransportConfig,
        implementation: PressureImplementation,
        fraction: Vec<f64>,
    ) -> Result<Self, LiquidStepError> {
        let limit = config.carrier.memory_limit;
        let faces = Axis::ALL.into_iter().try_fold(0_usize, |n, a| {
            n.checked_add(grid.face_len(a))
                .ok_or(LiquidStepError::AllocationFailed)
        })?;
        let volume_min = fraction
            .capacity()
            .checked_add(grid.cell_len())
            .and_then(|n| n.checked_add(faces))
            .and_then(|n| n.checked_mul(8))
            .ok_or(LiquidStepError::AllocationFailed)?;
        let pressure_min = grid
            .cell_len()
            .checked_mul(8)
            .ok_or(LiquidStepError::AllocationFailed)?;
        let carrier_min = BufferPlan::for_grid(&grid, usize::MAX)
            .map_err(SimulationError::from)?
            .total_bytes;
        let required = carrier_min
            .checked_add(volume_min)
            .and_then(|n| n.checked_add(pressure_min))
            .ok_or(LiquidStepError::AllocationFailed)?;
        if required > limit {
            return Err(LiquidStepError::BufferLimit { required, limit });
        }
        let mut carrier_config = config.carrier;
        carrier_config.memory_limit = limit - volume_min - pressure_min;
        let carrier =
            Simulation::with_implementation(grid.clone(), carrier_config, implementation)?;
        let liquid = LiquidVolumeState::new(
            grid,
            config.represented_density,
            config.volume_stamp,
            fraction,
            limit - carrier.allocated_bytes() - pressure_min,
        )?;
        let mut remaining = limit - carrier.allocated_bytes() - liquid.allocated_bytes();
        let pressure = crate::liquid_volume::allocate(carrier.grid().cell_len(), &mut remaining)?;
        Ok(Self {
            carrier,
            liquid,
            pressure,
            carrier_id: config.carrier_id,
            allocated_bytes: limit - remaining,
        })
    }
    pub fn grid(&self) -> &GridGeometry {
        self.carrier.grid()
    }
    pub fn implementation(&self) -> PressureImplementation {
        self.carrier.implementation()
    }
    pub fn allocated_bytes(&self) -> usize {
        self.allocated_bytes
    }
    pub fn state(&self) -> LiquidTransportView<'_> {
        let carrier = self.carrier.state();
        LiquidTransportView {
            carrier,
            liquid: self.liquid.state(),
            pressure: &self.pressure,
            carrier_stamp: VolumeStamp {
                id: self.carrier_id,
                version: carrier.generation,
            },
        }
    }
    pub fn set_paused(&mut self, paused: bool) {
        self.carrier.set_paused(paused);
    }
    /// Closed carrier box. Inlet fractions have no effect on zero outer flux.
    pub fn step(
        &mut self,
        inputs: LiquidStepInputs<'_>,
        cancel: impl FnMut(LiquidStepStage) -> bool,
    ) -> Result<LiquidStepReport, LiquidStepError> {
        self.step_impl(inputs, None, cancel)
    }
    /// End-of-step prescribed carrier box flux and explicit liquid inflow
    /// donors. Appearance tracer follows the existing selected boundary policy.
    pub fn step_with_box_flux(
        &mut self,
        inputs: LiquidStepInputs<'_>,
        workspace: &mut BoxFluxStepWorkspace,
        boundary: BoxFluxStepBoundary,
        cancel: impl FnMut(LiquidStepStage) -> bool,
    ) -> Result<LiquidStepReport, LiquidStepError> {
        self.step_impl(inputs, Some((workspace, boundary)), cancel)
    }
    fn step_impl(
        &mut self,
        inputs: LiquidStepInputs<'_>,
        mut boundary: Option<(&mut BoxFluxStepWorkspace, BoxFluxStepBoundary)>,
        mut cancel: impl FnMut(LiquidStepStage) -> bool,
    ) -> Result<LiquidStepReport, LiquidStepError> {
        self.liquid.validate_advance(inputs.source, inputs.volume)?;
        let boundary_bytes = boundary.as_ref().map_or(0, |(w, _)| w.allocated_bytes());
        let total_bytes = self
            .allocated_bytes
            .checked_add(boundary_bytes)
            .ok_or(LiquidStepError::AllocationFailed)?;
        let candidate = self.carrier.prepare_liquid_carrier(
            inputs.requested_dt,
            inputs.smoke_source,
            inputs.forces,
            boundary.as_mut().map(|(w, b)| (&mut **w, *b)),
            |stage| cancel(LiquidStepStage::Carrier(stage)),
        )?;
        let step = candidate.legacy.step.step;
        let flow = LiquidFlowInterval::new(
            self.carrier.grid(),
            VolumeStamp {
                id: self.carrier_id,
                version: step.generation,
            },
            self.carrier.candidate_velocity(),
            self.carrier.state().time,
            step.dt,
        )?;
        let liquid = self.liquid.prepare_advance(
            flow,
            inputs.inlet,
            inputs.source,
            inputs.volume,
            |stage| cancel(LiquidStepStage::Volume(stage)),
        )?;
        if cancel(LiquidStepStage::BeforeCommit) {
            return Err(LiquidStepError::Cancelled);
        }
        // All sizes, times, versions and numeric gates have already passed.
        // Pressure copy and owner swaps are infallible and invoke no callbacks.
        let pressure = match &boundary {
            Some((w, _)) => w.projection.pressure(),
            None => self.carrier.candidate_pressure(),
        };
        self.pressure.copy_from_slice(pressure);
        self.carrier.commit_prepared(&candidate);
        self.liquid.commit_prepared(&liquid);
        Ok(LiquidStepReport {
            carrier: candidate.legacy.step,
            boundary: candidate.boundary,
            liquid,
            owned_array_bytes: self.allocated_bytes,
            boundary_workspace_array_bytes: boundary_bytes,
            total_array_bytes: total_bytes,
        })
    }
}
