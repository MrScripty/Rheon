//! Owned named fields and transactional publication for the fixed-box smoke
//! milestone. Export buffers and application/GUI state are outside this module.
use crate::{
    AdvectionError, Axis, BodyForce, BufferPlan, ForcedStepReport, GeometryError, GridGeometry,
    OperatorError, PressureError, PressureImplementation, PressureOperator, PressureReport,
    PressureSettings, PressureWorkspace, advect_tracer, advect_velocity,
};
use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct SimulationConfig {
    pub density: f64,
    /// Retained simulation Vec-capacity payload, not process RSS or allocator overhead.
    pub memory_limit: usize,
    pub pressure: PressureSettings,
    pub actual_divergence_limit: f64,
    pub max_courant: f64,
}
#[derive(Debug, Clone, Copy)]
pub struct SmokeSource {
    pub lower: [f64; 3],
    pub upper: [f64; 3],
    /// Concentration units per second, saturated at one.
    pub tracer_rate: f64,
    /// A localized prescribed Y acceleration, not a temperature/buoyancy model.
    pub vertical_acceleration: f64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStage {
    BeforeAdvection,
    VelocitySlice,
    BeforeForces,
    ForceSlice,
    BeforePressure,
    PressureIteration,
    ProjectionSlice,
    BeforeProjectionAcceptance,
    BeforeTracer,
    TracerSlice,
    BeforeCommit,
}
#[derive(Debug, Clone, Copy)]
pub struct StepReport {
    pub dt: f64,
    pub time: f64,
    pub generation: u64,
    pub pressure: PressureReport,
    pub actual_divergence_max: f64,
    pub courant: f64,
    pub tracer_integral: f64,
    pub kinetic_energy: f64,
}
#[derive(Debug, Clone, PartialEq)]
pub enum SimulationError {
    Geometry(GeometryError),
    Operator(OperatorError),
    Pressure(PressureError),
    Advection(AdvectionError),
    TracerBarrier(crate::TracerBarrierError),
    BoxFlux(crate::BoxFluxError),
    BoundaryWorkspaceMismatch,
    InvalidConfig,
    InvalidSource,
    InvalidForce,
    InvalidTimeStep,
    TimeResolution,
    GenerationOverflow,
    AllocationFailed,
    BufferLimit { required: usize, limit: usize },
    Paused,
    Cancelled { stage: StepStage },
    DivergenceLimit { actual: f64, limit: f64 },
    CourantLimit { actual: f64, limit: f64 },
    ArithmeticFailure,
}
impl fmt::Display for SimulationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "simulation rejected operation: {self:?}")
    }
}
impl std::error::Error for SimulationError {}
impl From<GeometryError> for SimulationError {
    fn from(e: GeometryError) -> Self {
        Self::Geometry(e)
    }
}
impl From<OperatorError> for SimulationError {
    fn from(e: OperatorError) -> Self {
        Self::Operator(e)
    }
}
impl From<PressureError> for SimulationError {
    fn from(e: PressureError) -> Self {
        Self::Pressure(e)
    }
}
impl From<AdvectionError> for SimulationError {
    fn from(e: AdvectionError) -> Self {
        Self::Advection(e)
    }
}

impl From<crate::TracerBarrierError> for SimulationError {
    fn from(e: crate::TracerBarrierError) -> Self {
        match e {
            crate::TracerBarrierError::Cancelled => Self::Cancelled {
                stage: StepStage::TracerSlice,
            },
            other => Self::TracerBarrier(other),
        }
    }
}

impl From<crate::BoxFluxError> for SimulationError {
    fn from(error: crate::BoxFluxError) -> Self {
        match error {
            crate::BoxFluxError::Pressure(error) => Self::Pressure(error),
            crate::BoxFluxError::Operator(error) => Self::Operator(error),
            crate::BoxFluxError::DivergenceLimit { actual, limit } => {
                Self::DivergenceLimit { actual, limit }
            }
            crate::BoxFluxError::Cancelled { stage } => Self::Cancelled {
                stage: match stage {
                    crate::BoxFluxStage::BeforeSolve => StepStage::BeforePressure,
                    crate::BoxFluxStage::PressureIteration => StepStage::PressureIteration,
                    crate::BoxFluxStage::CorrectionSlice => StepStage::ProjectionSlice,
                    crate::BoxFluxStage::BeforeAcceptance => StepStage::BeforeProjectionAcceptance,
                },
            },
            other => Self::BoxFlux(other),
        }
    }
}
struct BoundaryStepInput<'a> {
    workspace: &'a mut crate::BoxFluxStepWorkspace,
    boundary: crate::BoxFluxStepBoundary,
}
pub(crate) struct StepOutcome {
    pub(crate) legacy: crate::BarrierStepReport,
    pub(crate) boundary: Option<crate::BoxFluxStepReport>,
}

struct Fields {
    x: Vec<f32>,
    y: Vec<f32>,
    z: Vec<f32>,
    tracer: Vec<f32>,
}
impl Fields {
    fn new(g: &GridGeometry, remaining: &mut usize) -> Result<Self, SimulationError> {
        Ok(Self {
            x: allocate(g.face_len(Axis::X), remaining)?,
            y: allocate(g.face_len(Axis::Y), remaining)?,
            z: allocate(g.face_len(Axis::Z), remaining)?,
            tracer: allocate(g.cell_len(), remaining)?,
        })
    }
    fn velocity(&self) -> [&[f32]; 3] {
        [&self.x, &self.y, &self.z]
    }
    fn velocity_mut(&mut self) -> [&mut [f32]; 3] {
        [&mut self.x, &mut self.y, &mut self.z]
    }
    fn clear(&mut self) {
        self.x.fill(0.0);
        self.y.fill(0.0);
        self.z.fill(0.0);
        self.tracer.fill(0.0);
    }
}
pub(crate) fn allocate(n: usize, remaining: &mut usize) -> Result<Vec<f32>, SimulationError> {
    let required = n.checked_mul(4).ok_or(SimulationError::AllocationFailed)?;
    if required > *remaining {
        return Err(SimulationError::BufferLimit {
            required,
            limit: *remaining,
        });
    }
    let mut v = Vec::new();
    v.try_reserve_exact(n)
        .map_err(|_| SimulationError::AllocationFailed)?;
    let bytes = v
        .capacity()
        .checked_mul(4)
        .ok_or(SimulationError::AllocationFailed)?;
    if bytes > *remaining {
        return Err(SimulationError::BufferLimit {
            required: bytes,
            limit: *remaining,
        });
    }
    v.resize(n, 0.0);
    *remaining -= bytes;
    Ok(v)
}

/// Immutable named-axis view. Callers cannot mutate accepted state or replace
/// equally sized axis arrays. Low-level positional views are assembled here.
#[derive(Clone, Copy)]
pub struct StateView<'a> {
    pub x: &'a [f32],
    pub y: &'a [f32],
    pub z: &'a [f32],
    pub tracer: &'a [f32],
    pub time: f64,
    pub generation: u64,
}

pub struct Simulation {
    implementation: PressureImplementation,
    grid: GridGeometry,
    config: SimulationConfig,
    accepted: Fields,
    candidate: Fields,
    workspace: PressureWorkspace,
    allocated_bytes: usize,
    time: f64,
    generation: u64,
    paused: bool,
}
impl Simulation {
    pub fn new(grid: GridGeometry, config: SimulationConfig) -> Result<Self, SimulationError> {
        Self::with_implementation(grid, config, PressureImplementation::default())
    }
    /// Select once per simulation. Create a fresh simulation to compare methods
    /// from identical initial conditions; accepted state cannot change methods.
    pub fn with_implementation(
        grid: GridGeometry,
        config: SimulationConfig,
        implementation: PressureImplementation,
    ) -> Result<Self, SimulationError> {
        if !config.actual_divergence_limit.is_finite()
            || config.actual_divergence_limit < 0.0
            || !config.max_courant.is_finite()
            || config.max_courant <= 0.0
            || config.max_courant > 1.0
            || [
                config.pressure.relative_residual,
                config.pressure.absolute_residual,
                config.pressure.divergence_limit,
            ]
            .iter()
            .any(|x| !x.is_finite() || *x < 0.0)
        {
            return Err(SimulationError::InvalidConfig);
        }
        PressureOperator::new(&grid, config.density)?;
        BufferPlan::for_grid(&grid, config.memory_limit)?;
        let mut remaining = config.memory_limit;
        let accepted = Fields::new(&grid, &mut remaining)?;
        let candidate = Fields::new(&grid, &mut remaining)?;
        let workspace = PressureWorkspace::with_implementation(&grid, remaining, implementation)?;
        remaining -= workspace.allocated_bytes();
        Ok(Self {
            implementation,
            grid,
            config,
            accepted,
            candidate,
            workspace,
            allocated_bytes: config.memory_limit - remaining,
            time: 0.0,
            generation: 0,
            paused: false,
        })
    }
    pub fn implementation(&self) -> PressureImplementation {
        self.implementation
    }

    pub fn grid(&self) -> &GridGeometry {
        &self.grid
    }
    pub fn allocated_bytes(&self) -> usize {
        self.allocated_bytes
    }
    pub fn state(&self) -> StateView<'_> {
        StateView {
            x: &self.accepted.x,
            y: &self.accepted.y,
            z: &self.accepted.z,
            tracer: &self.accepted.tracer,
            time: self.time,
            generation: self.generation,
        }
    }
    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }
    pub fn reset(&mut self) -> Result<(), SimulationError> {
        let next = self
            .generation
            .checked_add(1)
            .ok_or(SimulationError::GenerationOverflow)?;
        self.accepted.clear();
        self.candidate.clear();
        self.time = 0.0;
        self.generation = next;
        Ok(())
    }
    /// Any error/cancellation preserves accepted fields, time and generation.
    /// Scratch may be modified; the next call overwrites every candidate field.
    pub fn step(
        &mut self,
        requested_dt: f64,
        source: Option<SmokeSource>,
        cancel: impl FnMut(StepStage) -> bool,
    ) -> Result<StepReport, SimulationError> {
        self.step_with_forces(requested_dt, source, &[], cancel)
            .map(|report| report.step)
    }
    /// Advance with caller-owned external forces. The legacy smoke force is
    /// applied first; external vectors add on their own face lattices before
    /// projection. Empty forces preserve the original stage sequence/results.
    /// Any error/cancellation preserves accepted fields, time and generation.
    pub fn step_with_forces(
        &mut self,
        requested_dt: f64,
        source: Option<SmokeSource>,
        forces: &[BodyForce],
        cancel: impl FnMut(StepStage) -> bool,
    ) -> Result<ForcedStepReport, SimulationError> {
        self.step_impl(requested_dt, source, forces, None, None, cancel)
            .map(|report| report.legacy.step)
    }
    /// Opt-in static passive-tracer barrier; velocity/pressure remain the fixed
    /// box model. None preserves the legacy kernel and exact callback sequence.
    /// Surface geometry is borrowed for this call and its stamp is reported only
    /// on accepted publication. Failure preserves every accepted state bit.
    pub fn step_with_tracer_barrier(
        &mut self,
        requested_dt: f64,
        source: Option<SmokeSource>,
        forces: &[BodyForce],
        barrier: Option<&crate::TriangleSurface>,
        cancel: impl FnMut(StepStage) -> bool,
    ) -> Result<crate::BarrierStepReport, SimulationError> {
        self.step_impl(requested_dt, source, forces, barrier, None, cancel)
            .map(|report| report.legacy)
    }
    /// Opt-in fixed-volume inlet/outlet step. Both state and boundary work report
    /// publish only after pressure, stored divergence, Courant and tracer gates.
    /// Boundary projection uses an independently capped caller-owned workspace;
    /// default steps and their allocation/callback sequence remain unchanged.
    pub fn step_with_box_flux(
        &mut self,
        requested_dt: f64,
        source: Option<SmokeSource>,
        forces: &[BodyForce],
        workspace: &mut crate::BoxFluxStepWorkspace,
        boundary: crate::BoxFluxStepBoundary,
        cancel: impl FnMut(StepStage) -> bool,
    ) -> Result<crate::BoxFluxStepReport, SimulationError> {
        let outcome = self.step_impl(
            requested_dt,
            source,
            forces,
            None,
            Some(BoundaryStepInput {
                workspace,
                boundary,
            }),
            cancel,
        )?;
        Ok(outcome
            .boundary
            .expect("requested boundary has accepted diagnostics"))
    }
    fn step_impl(
        &mut self,
        requested_dt: f64,
        source: Option<SmokeSource>,
        forces: &[BodyForce],
        barrier: Option<&crate::TriangleSurface>,
        boundary: Option<BoundaryStepInput<'_>>,
        cancel: impl FnMut(StepStage) -> bool,
    ) -> Result<StepOutcome, SimulationError> {
        let outcome = self.prepare_impl(requested_dt, source, forces, barrier, boundary, cancel)?;
        self.commit_prepared(&outcome);
        Ok(outcome)
    }
    pub(crate) fn prepare_liquid_carrier(
        &mut self,
        requested_dt: f64,
        source: Option<SmokeSource>,
        forces: &[BodyForce],
        boundary: Option<(&mut crate::BoxFluxStepWorkspace, crate::BoxFluxStepBoundary)>,
        cancel: impl FnMut(StepStage) -> bool,
    ) -> Result<StepOutcome, SimulationError> {
        self.prepare_impl(
            requested_dt,
            source,
            forces,
            None,
            boundary.map(|(workspace, boundary)| BoundaryStepInput {
                workspace,
                boundary,
            }),
            cancel,
        )
    }
    pub(crate) fn candidate_velocity(&self) -> [&[f32]; 3] {
        self.candidate.velocity()
    }
    pub(crate) fn candidate_pressure(&self) -> &[f64] {
        self.workspace.pressure()
    }
    pub(crate) fn commit_prepared(&mut self, outcome: &StepOutcome) {
        std::mem::swap(&mut self.accepted, &mut self.candidate);
        self.time = outcome.legacy.step.step.time;
        self.generation = outcome.legacy.step.step.generation;
    }
    fn prepare_impl(
        &mut self,
        requested_dt: f64,
        source: Option<SmokeSource>,
        forces: &[BodyForce],
        barrier: Option<&crate::TriangleSurface>,
        mut boundary: Option<BoundaryStepInput<'_>>,
        mut cancel: impl FnMut(StepStage) -> bool,
    ) -> Result<StepOutcome, SimulationError> {
        if self.paused {
            return Err(SimulationError::Paused);
        }
        if !requested_dt.is_finite() || requested_dt <= 0.0 {
            return Err(SimulationError::InvalidTimeStep);
        }
        if let Some(s) = source {
            validate_source(&self.grid, s)?;
        }
        let external_rate = crate::forces::validate(&self.grid, self.config.density, forces)?;
        if let Some(input) = &boundary
            && !input
                .workspace
                .matches(&self.grid, self.config.density, self.implementation)
        {
            return Err(SimulationError::BoundaryWorkspaceMismatch);
        }
        let mut boundary_work = if boundary.is_some() {
            Some(crate::BoxFluxStepWork {
                kinetic_old: crate::box_flux_step::interior_energy(
                    &self.grid,
                    self.config.density,
                    self.accepted.velocity(),
                )?,
                kinetic_after_advection: 0.0,
                kinetic_after_smoke_force: 0.0,
                kinetic_before_projection: 0.0,
                kinetic_accepted: 0.0,
                advection_change: 0.0,
                smoke_force_work: 0.0,
                external_force_work: 0.0,
                budget_error: 0.0,
                boundary_volume_imbalance: 0.0,
                tracer_old_integral: self
                    .accepted
                    .tracer
                    .iter()
                    .map(|&v| f64::from(v))
                    .sum::<f64>()
                    * self.grid.cell_volume(),
                tracer_transport_change: 0.0,
                tracer_source_change: 0.0,
            })
        } else {
            None
        };
        let next_generation = self
            .generation
            .checked_add(1)
            .ok_or(SimulationError::GenerationOverflow)?;
        let h = self.grid.spacing();
        let rate = if let Some(input) = &boundary {
            let speeds = input.boundary.flux.outward_speeds();
            let rate = (0..3)
                .map(|d| {
                    let old = self.accepted.velocity()[d]
                        .iter()
                        .fold(0.0_f64, |m, &v| m.max(f64::from(v).abs()));
                    old.max(f64::from(speeds[d][0]).abs())
                        .max(f64::from(speeds[d][1]).abs())
                        / h[d]
                })
                .sum::<f64>();
            if !rate.is_finite() {
                return Err(SimulationError::ArithmeticFailure);
            }
            rate
        } else {
            velocity_rate(self.accepted.velocity(), h)?
        };
        let acceleration =
            source.map_or(0.0, |s| s.vertical_acceleration.abs() / h[1]) + external_rate;
        if !acceleration.is_finite() {
            return Err(SimulationError::ArithmeticFailure);
        }
        // Split the displacement budget equally between old speed and forcing.
        // A smaller dt is a declared policy, not a proof of solver stability.
        let mut dt = requested_dt;
        if rate > 0.0 {
            dt = dt.min(0.5 * self.config.max_courant / rate);
        }
        if acceleration > 0.0 {
            dt = dt.min((0.5 * self.config.max_courant / acceleration).sqrt());
        }
        let next_time = self.time + dt;
        if dt <= 0.0 || !next_time.is_finite() || next_time <= self.time {
            return Err(SimulationError::TimeResolution);
        }
        checkpoint(&mut cancel, StepStage::BeforeAdvection)?;
        advect_velocity(
            &self.grid,
            self.accepted.velocity(),
            dt,
            self.candidate.velocity_mut(),
            || cancel(StepStage::VelocitySlice),
        )?;
        if let Some(work) = &mut boundary_work {
            work.kinetic_after_advection = crate::box_flux_step::interior_energy(
                &self.grid,
                self.config.density,
                self.candidate.velocity(),
            )?;
            work.advection_change = work.kinetic_after_advection - work.kinetic_old;
        }
        if let Some(s) = source {
            apply_force(&self.grid, &mut self.candidate.y, s, dt)?;
        }
        if let Some(work) = &mut boundary_work {
            work.kinetic_after_smoke_force = crate::box_flux_step::interior_energy(
                &self.grid,
                self.config.density,
                self.candidate.velocity(),
            )?;
            work.smoke_force_work = work.kinetic_after_smoke_force - work.kinetic_after_advection;
        }
        let forces = if forces.is_empty() {
            None
        } else {
            checkpoint(&mut cancel, StepStage::BeforeForces)?;
            Some(crate::forces::apply(
                &self.grid,
                self.config.density,
                forces,
                dt,
                self.candidate.velocity_mut(),
                &mut cancel,
            )?)
        };
        checkpoint(&mut cancel, StepStage::BeforePressure)?;
        let (pressure, actual_divergence_max, projection) = if let Some(input) = &mut boundary {
            for (scratch, velocity) in input
                .workspace
                .provisional
                .iter_mut()
                .zip(self.candidate.velocity())
            {
                scratch.copy_from_slice(velocity);
            }
            let provisional = &input.workspace.provisional;
            let report = input.workspace.projection.project(
                [&provisional[0], &provisional[1], &provisional[2]],
                dt,
                input.boundary.flux,
                crate::BoxFluxSettings {
                    pressure: self.config.pressure,
                    actual_divergence_limit: self.config.actual_divergence_limit,
                },
                self.candidate.velocity_mut(),
                |stage| match stage {
                    // The ordinary BeforePressure checkpoint above already ran.
                    crate::BoxFluxStage::BeforeSolve => false,
                    crate::BoxFluxStage::PressureIteration => cancel(StepStage::PressureIteration),
                    crate::BoxFluxStage::CorrectionSlice => cancel(StepStage::ProjectionSlice),
                    crate::BoxFluxStage::BeforeAcceptance => {
                        cancel(StepStage::BeforeProjectionAcceptance)
                    }
                },
            )?;
            (report.pressure, report.actual_divergence_max, Some(report))
        } else {
            let operator = PressureOperator::new(&self.grid, self.config.density)?;
            let pressure = self.workspace.solve_velocity(
                &operator,
                self.candidate.velocity(),
                dt,
                self.config.pressure,
                || cancel(StepStage::PressureIteration),
            )?;
            operator.correct_candidate_in_place(
                self.workspace.pressure(),
                dt,
                self.candidate.velocity_mut(),
            )?;
            let actual_divergence_max = self
                .workspace
                .actual_divergence_max(&operator, self.candidate.velocity())?;
            if actual_divergence_max > self.config.actual_divergence_limit {
                return Err(SimulationError::DivergenceLimit {
                    actual: actual_divergence_max,
                    limit: self.config.actual_divergence_limit,
                });
            }
            (pressure, actual_divergence_max, None)
        };
        if let (Some(work), Some(report)) = (&mut boundary_work, projection) {
            work.kinetic_before_projection = report.work.kinetic_before;
            work.kinetic_accepted = report.work.kinetic_after;
            work.external_force_work = forces.map_or(0.0, |f| f.applied_work);
            work.boundary_volume_imbalance = dt * report.net_outward_flux;
            work.budget_error = work.kinetic_accepted - work.kinetic_old
                + report.work.correction_energy
                - (work.advection_change
                    + work.smoke_force_work
                    + work.external_force_work
                    + report.work.boundary_pressure_work
                    + report.work.divergence_residual_work
                    + report.work.correction_residual_work);
        }
        let courant = dt * velocity_rate(self.candidate.velocity(), h)?;
        if !courant.is_finite() {
            return Err(SimulationError::ArithmeticFailure);
        }
        if courant > self.config.max_courant {
            return Err(SimulationError::CourantLimit {
                actual: courant,
                limit: self.config.max_courant,
            });
        }
        checkpoint(&mut cancel, StepStage::BeforeTracer)?;
        let tracer_barrier = if let Some(surface) = barrier {
            Some(crate::advect_tracer_with_barrier(
                &self.grid,
                &self.accepted.tracer,
                [&self.candidate.x, &self.candidate.y, &self.candidate.z],
                dt,
                surface,
                &mut self.candidate.tracer,
                || cancel(StepStage::TracerSlice),
            )?)
        } else {
            advect_tracer(
                &self.grid,
                &self.accepted.tracer,
                [&self.candidate.x, &self.candidate.y, &self.candidate.z],
                dt,
                &mut self.candidate.tracer,
                || cancel(StepStage::TracerSlice),
            )?;
            None
        };
        let transported_integral = if boundary.is_some() {
            self.candidate
                .tracer
                .iter()
                .map(|&v| f64::from(v))
                .sum::<f64>()
                * self.grid.cell_volume()
        } else {
            0.0
        };
        if let Some(s) = source {
            apply_source(&self.grid, &mut self.candidate.tracer, s, dt)?;
        }
        let tracer_sum = self
            .candidate
            .tracer
            .iter()
            .try_fold(0.0_f64, |sum, &value| {
                if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                    Err(SimulationError::ArithmeticFailure)
                } else {
                    Ok(sum + f64::from(value))
                }
            })?;
        let energy_sum = self
            .candidate
            .velocity()
            .into_iter()
            .flatten()
            .map(|&v| f64::from(v).powi(2))
            .sum::<f64>();
        let tracer_integral = tracer_sum * self.grid.cell_volume();
        let kinetic_energy = boundary_work.map_or_else(
            || 0.5 * self.config.density * self.grid.cell_volume() * energy_sum,
            |work| work.kinetic_accepted,
        );
        if let Some(work) = &mut boundary_work {
            work.tracer_transport_change = transported_integral - work.tracer_old_integral;
            work.tracer_source_change = tracer_integral - transported_integral;
            if [
                work.budget_error,
                work.boundary_volume_imbalance,
                work.tracer_old_integral,
                work.tracer_transport_change,
                work.tracer_source_change,
            ]
            .iter()
            .any(|v| !v.is_finite())
            {
                return Err(SimulationError::ArithmeticFailure);
            }
        }
        if !tracer_integral.is_finite() || !kinetic_energy.is_finite() {
            return Err(SimulationError::ArithmeticFailure);
        }
        checkpoint(&mut cancel, StepStage::BeforeCommit)?;
        let legacy = crate::BarrierStepReport {
            step: ForcedStepReport {
                step: StepReport {
                    dt,
                    time: next_time,
                    generation: next_generation,
                    pressure,
                    actual_divergence_max,
                    courant,
                    tracer_integral,
                    kinetic_energy,
                },
                forces,
            },
            tracer_barrier,
        };
        let boundary = match (boundary, projection, boundary_work) {
            (Some(input), Some(projection), Some(work)) => Some(crate::BoxFluxStepReport {
                step: legacy.step,
                projection,
                work,
                tracer_policy: input.boundary.tracer,
                simulation_array_bytes: self.allocated_bytes,
                boundary_workspace_array_bytes: input.workspace.allocated_bytes(),
            }),
            _ => None,
        };
        Ok(StepOutcome { legacy, boundary })
    }
}
fn checkpoint(
    cancel: &mut impl FnMut(StepStage) -> bool,
    stage: StepStage,
) -> Result<(), SimulationError> {
    if cancel(stage) {
        Err(SimulationError::Cancelled { stage })
    } else {
        Ok(())
    }
}
fn velocity_rate(velocity: [&[f32]; 3], h: [f64; 3]) -> Result<f64, SimulationError> {
    let rate = (0..3)
        .map(|d| {
            velocity[d]
                .iter()
                .fold(0.0_f64, |m, v| m.max(f64::from(*v).abs()))
                / h[d]
        })
        .sum::<f64>();
    if rate.is_finite() {
        Ok(rate)
    } else {
        Err(SimulationError::ArithmeticFailure)
    }
}
fn validate_source(g: &GridGeometry, s: SmokeSource) -> Result<(), SimulationError> {
    if !s.tracer_rate.is_finite() || s.tracer_rate < 0.0 || !s.vertical_acceleration.is_finite() {
        return Err(SimulationError::InvalidSource);
    }
    for d in 0..3 {
        if !s.lower[d].is_finite()
            || !s.upper[d].is_finite()
            || s.lower[d] >= s.upper[d]
            || s.lower[d] < g.origin()[d]
            || s.upper[d] > g.upper()[d]
        {
            return Err(SimulationError::InvalidSource);
        }
    }
    Ok(())
}
fn inside(p: [f64; 3], s: SmokeSource) -> bool {
    (0..3).all(|d| p[d] >= s.lower[d] && p[d] < s.upper[d])
}
fn apply_force(
    g: &GridGeometry,
    y: &mut [f32],
    s: SmokeSource,
    dt: f64,
) -> Result<(), SimulationError> {
    let [nx, ny, nz] = g.counts();
    for k in 0..nz {
        for j in 1..ny {
            for i in 0..nx {
                let p = [i, j, k];
                if inside(g.face_position(Axis::Y, p).expect("bounded face"), s) {
                    let n = g.face_unchecked(Axis::Y, p);
                    let v = f64::from(y[n]) + dt * s.vertical_acceleration;
                    if !v.is_finite() || v.abs() > f64::from(f32::MAX) {
                        return Err(SimulationError::ArithmeticFailure);
                    }
                    y[n] = v as f32;
                }
            }
        }
    }
    Ok(())
}
fn apply_source(
    g: &GridGeometry,
    tracer: &mut [f32],
    s: SmokeSource,
    dt: f64,
) -> Result<(), SimulationError> {
    let amount = dt * s.tracer_rate;
    if !amount.is_finite() {
        return Err(SimulationError::ArithmeticFailure);
    }
    let [nx, ny, nz] = g.counts();
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let p = [i, j, k];
                if inside(g.cell_position(p).expect("bounded cell"), s) {
                    let n = g.cell_unchecked(p);
                    tracer[n] = (f64::from(tracer[n]) + amount).min(1.0) as f32;
                }
            }
        }
    }
    Ok(())
}
