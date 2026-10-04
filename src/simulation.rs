//! Owned named fields and transactional publication for the fixed-box smoke
//! milestone. Export buffers and application/GUI state are outside this module.
use crate::{
    AdvectionError, Axis, BufferPlan, GeometryError, GridGeometry, OperatorError, PressureError,
    PressureImplementation, PressureOperator, PressureReport, PressureSettings, PressureWorkspace,
    advect_tracer, advect_velocity,
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
    BeforePressure,
    PressureIteration,
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
    InvalidConfig,
    InvalidSource,
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
fn allocate(n: usize, remaining: &mut usize) -> Result<Vec<f32>, SimulationError> {
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
        mut cancel: impl FnMut(StepStage) -> bool,
    ) -> Result<StepReport, SimulationError> {
        if self.paused {
            return Err(SimulationError::Paused);
        }
        if !requested_dt.is_finite() || requested_dt <= 0.0 {
            return Err(SimulationError::InvalidTimeStep);
        }
        if let Some(s) = source {
            validate_source(&self.grid, s)?;
        }
        let next_generation = self
            .generation
            .checked_add(1)
            .ok_or(SimulationError::GenerationOverflow)?;
        let h = self.grid.spacing();
        let rate = velocity_rate(self.accepted.velocity(), h)?;
        let acceleration = source.map_or(0.0, |s| s.vertical_acceleration.abs() / h[1]);
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
        if let Some(s) = source {
            apply_force(&self.grid, &mut self.candidate.y, s, dt)?;
        }
        checkpoint(&mut cancel, StepStage::BeforePressure)?;
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
        advect_tracer(
            &self.grid,
            &self.accepted.tracer,
            [&self.candidate.x, &self.candidate.y, &self.candidate.z],
            dt,
            &mut self.candidate.tracer,
            || cancel(StepStage::TracerSlice),
        )?;
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
        let kinetic_energy = 0.5 * self.config.density * self.grid.cell_volume() * energy_sum;
        if !tracer_integral.is_finite() || !kinetic_energy.is_finite() {
            return Err(SimulationError::ArithmeticFailure);
        }
        checkpoint(&mut cancel, StepStage::BeforeCommit)?;
        std::mem::swap(&mut self.accepted, &mut self.candidate);
        self.time = next_time;
        self.generation = next_generation;
        Ok(StepReport {
            dt,
            time: self.time,
            generation: self.generation,
            pressure,
            actual_divergence_max,
            courant,
            tracer_integral,
            kinetic_energy,
        })
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
