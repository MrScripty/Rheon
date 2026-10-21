//! Serial periodic, cell-centered ideal-gas Euler with atomic publication.
use crate::{Axis, GridGeometry};
use std::{fmt, mem::size_of, time::Instant};

/// Conservative cell averages: rho, mx, my, mz, total energy density (SI).
pub type ConservativeState = [f64; 5];
pub const COMPRESSIBLE_METHOD: &str = "llf-euler-periodic-v1";
const MAX_RETRIES: u8 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressibleStage {
    Initialization,
    Preflight,
    Cfl,
    Candidate,
    Diagnostics,
    Publication,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressibleCause {
    Nonfinite,
    NonpositiveDensity,
    NonpositiveInternalEnergy,
    InvalidThermodynamics,
    Arithmetic,
}

/// Context of an EOS, candidate or reduction failure. Components use U order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompressibleFailure {
    pub stage: CompressibleStage,
    pub cell: Option<usize>,
    pub component: Option<usize>,
    pub cause: CompressibleCause,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompressibleError {
    InvalidParameter {
        name: &'static str,
    },
    StateLength {
        expected: usize,
        actual: usize,
    },
    InvalidState(CompressibleFailure),
    CapacityOverflow,
    BufferLimit {
        required: usize,
        limit: usize,
    },
    Allocation,
    Arithmetic(CompressibleFailure),
    Cancelled(CompressibleCheckpoint),
    NonadvancingTime {
        time: f64,
        dt: f64,
    },
    GenerationOverflow,
    RetryExhausted {
        attempts: u8,
        last_dt: f64,
        last: CompressibleFailure,
    },
}

impl fmt::Display for CompressibleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "compressible Euler: {self:?}")
    }
}
impl std::error::Error for CompressibleError {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompressiblePrimitive {
    pub density: f64,
    pub velocity: [f64; 3],
    pub pressure: f64,
    pub internal_energy_density: f64,
    pub temperature: f64,
    pub sound_speed: f64,
}

/// Constant calorically perfect gas. No floors or pressure gauge.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IdealGas {
    gamma: f64,
    gas_constant: f64,
    cv: f64,
}

fn failure(
    stage: CompressibleStage,
    cell: Option<usize>,
    component: Option<usize>,
    cause: CompressibleCause,
) -> CompressibleFailure {
    CompressibleFailure {
        stage,
        cell,
        component,
        cause,
    }
}

impl IdealGas {
    pub fn new(gamma: f64, gas_constant: f64) -> Result<Self, CompressibleError> {
        if !gamma.is_finite() || gamma <= 1.0 {
            return Err(CompressibleError::InvalidParameter { name: "gamma" });
        }
        if !gas_constant.is_finite() || gas_constant <= 0.0 {
            return Err(CompressibleError::InvalidParameter {
                name: "gas_constant",
            });
        }
        let cv = gas_constant / (gamma - 1.0);
        if !cv.is_finite() || cv <= 0.0 {
            return Err(CompressibleError::InvalidParameter { name: "cv" });
        }
        Ok(Self {
            gamma,
            gas_constant,
            cv,
        })
    }
    pub fn gamma(self) -> f64 {
        self.gamma
    }
    pub fn gas_constant(self) -> f64 {
        self.gas_constant
    }
    pub fn cv(self) -> f64 {
        self.cv
    }

    pub fn primitive(
        self,
        u: ConservativeState,
    ) -> Result<CompressiblePrimitive, CompressibleError> {
        self.derive(u, CompressibleStage::Initialization, None)
            .map_err(CompressibleError::InvalidState)
    }

    fn derive(
        self,
        u: ConservativeState,
        stage: CompressibleStage,
        cell: Option<usize>,
    ) -> Result<CompressiblePrimitive, CompressibleFailure> {
        for (component, value) in u.iter().enumerate() {
            if !value.is_finite() {
                return Err(failure(
                    stage,
                    cell,
                    Some(component),
                    CompressibleCause::Nonfinite,
                ));
            }
        }
        if u[0] <= 0.0 {
            return Err(failure(
                stage,
                cell,
                Some(0),
                CompressibleCause::NonpositiveDensity,
            ));
        }
        let velocity: [f64; 3] = std::array::from_fn(|d| u[d + 1] / u[0]);
        let kinetic = 0.5 * (u[1] * velocity[0] + u[2] * velocity[1] + u[3] * velocity[2]);
        let internal_energy_density = u[4] - kinetic;
        if !kinetic.is_finite()
            || !internal_energy_density.is_finite()
            || velocity.iter().any(|v| !v.is_finite())
        {
            return Err(failure(stage, cell, Some(4), CompressibleCause::Arithmetic));
        }
        if internal_energy_density <= 0.0 {
            return Err(failure(
                stage,
                cell,
                Some(4),
                CompressibleCause::NonpositiveInternalEnergy,
            ));
        }
        let pressure = (self.gamma - 1.0) * internal_energy_density;
        let temperature = pressure / (u[0] * self.gas_constant);
        let sound_squared = self.gamma * pressure / u[0];
        let sound_speed = sound_squared.sqrt();
        if [pressure, temperature, sound_squared, sound_speed]
            .iter()
            .any(|v| !v.is_finite() || *v <= 0.0)
            || !(u[0] * self.gas_constant).is_finite()
        {
            return Err(failure(
                stage,
                cell,
                Some(4),
                CompressibleCause::InvalidThermodynamics,
            ));
        }
        Ok(CompressiblePrimitive {
            density: u[0],
            velocity,
            pressure,
            internal_energy_density,
            temperature,
            sound_speed,
        })
    }

    pub fn conservative(
        self,
        density: f64,
        velocity: [f64; 3],
        pressure: f64,
    ) -> Result<ConservativeState, CompressibleError> {
        if !pressure.is_finite() || pressure <= 0.0 {
            return Err(CompressibleError::InvalidParameter { name: "pressure" });
        }
        if velocity.iter().any(|v| !v.is_finite()) {
            return Err(CompressibleError::InvalidParameter { name: "velocity" });
        }
        let momentum: [f64; 3] = velocity.map(|v| density * v);
        let energy = pressure / (self.gamma - 1.0)
            + 0.5
                * (momentum[0] * velocity[0]
                    + momentum[1] * velocity[1]
                    + momentum[2] * velocity[2]);
        let u = [density, momentum[0], momentum[1], momentum[2], energy];
        self.primitive(u)?;
        Ok(u)
    }

    /// Flux through a unit Cartesian normal. `positive=false` reverses it.
    pub fn physical_flux(
        self,
        u: ConservativeState,
        axis: Axis,
        positive: bool,
    ) -> Result<ConservativeState, CompressibleError> {
        let p = self.primitive(u)?;
        let mut flux = flux(u, p, axis.index());
        if flux.iter().any(|v| !v.is_finite()) {
            return Err(CompressibleError::Arithmetic(failure(
                CompressibleStage::Candidate,
                None,
                None,
                CompressibleCause::Arithmetic,
            )));
        }
        if !positive {
            flux = flux.map(|v| -v);
        }
        Ok(flux)
    }
}

fn flux(u: ConservativeState, p: CompressiblePrimitive, axis: usize) -> ConservativeState {
    let normal = p.velocity[axis];
    let mut f = u.map(|v| v * normal);
    f[axis + 1] += p.pressure;
    f[4] += p.pressure * normal;
    f
}

/// Deterministic cooperative checkpoints; no hard latency bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompressibleCheckpoint {
    pub stage: CompressibleStage,
    /// Zero for preflight; candidate attempts are numbered 1..=9.
    pub attempt: u8,
    pub cell: Option<usize>,
}

fn checkpoint(
    cancel: &mut impl FnMut(CompressibleCheckpoint) -> bool,
    stage: CompressibleStage,
    attempt: u8,
    cell: Option<usize>,
) -> Result<(), CompressibleError> {
    let point = CompressibleCheckpoint {
        stage,
        attempt,
        cell,
    };
    if cancel(point) {
        Err(CompressibleError::Cancelled(point))
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompressibleDiagnostics {
    /// Compensated volume-integrated U, in the declared SI unit system.
    pub totals: ConservativeState,
    pub min_density: f64,
    pub min_pressure: f64,
    pub min_internal_energy_density: f64,
    pub min_temperature: f64,
    pub max_speed: f64,
    pub max_mach: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompressibleStepReport {
    pub requested_dt: f64,
    pub accepted_dt: f64,
    pub time: f64,
    pub generation: u64,
    /// max over cells of sum of incident alpha / spacing (two faces per axis).
    pub incident_acoustic_rate: f64,
    pub cfl_bound: f64,
    pub controlling_cell: usize,
    pub diagnostics: CompressibleDiagnostics,
    /// Candidate totals minus previous accepted totals; periodic boundary flux is zero.
    pub ledger_residual: ConservativeState,
    pub initial_total_drift: ConservativeState,
    pub attempts: u8,
    pub rejections: u8,
    pub last_rejection: Option<CompressibleFailure>,
    pub flux_fallbacks: u64,
    pub method: &'static str,
    pub eos: IdealGas,
    pub counts: [usize; 3],
    pub spacing: [f64; 3],
    pub origin: [f64; 3],
    pub managed_capacity_bytes: usize,
    /// Host monotonic elapsed observation; distinct from simulated time.
    pub host_step_seconds: f64,
    /// Accumulated candidate flux/update time, including rejected attempts.
    pub host_kernel_seconds: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct CompressibleView<'a> {
    pub geometry: &'a GridGeometry,
    pub eos: IdealGas,
    pub conservative: &'a [ConservativeState],
    pub time: f64,
    pub generation: u64,
    pub diagnostics: CompressibleDiagnostics,
    pub last_report: Option<&'a CompressibleStepReport>,
}

/// Owns accepted state and bounded reusable scratch. Failed calls publish nothing.
pub struct CompressibleSimulation {
    grid: GridGeometry,
    eos: IdealGas,
    cfl: f64,
    accepted: Vec<ConservativeState>,
    candidate: Vec<ConservativeState>,
    change: Vec<ConservativeState>,
    rates: Vec<f64>,
    bytes: usize,
    time: f64,
    generation: u64,
    diagnostics: CompressibleDiagnostics,
    initial_totals: ConservativeState,
    last_report: Option<CompressibleStepReport>,
}

fn reserve<T: Clone>(n: usize, value: T) -> Result<Vec<T>, CompressibleError> {
    let mut v = Vec::new();
    v.try_reserve_exact(n)
        .map_err(|_| CompressibleError::Allocation)?;
    v.resize(n, value);
    Ok(v)
}

impl CompressibleSimulation {
    /// Construct a uniform accepted state at t=0. Payload cap covers three U
    /// buffers and one rate buffer; excludes caller data, stack, allocator/RSS.
    pub fn new(
        grid: GridGeometry,
        eos: IdealGas,
        cfl: f64,
        payload_limit: usize,
        uniform: ConservativeState,
    ) -> Result<Self, CompressibleError> {
        eos.primitive(uniform)?;
        Self::from_fn(grid, eos, cfl, payload_limit, |_, _| uniform)
    }

    /// Construct nonuniform conservative cell averages at t=0/generation=0.
    /// No owner escapes until all fields and diagnostics are admissible.
    pub fn from_fn(
        grid: GridGeometry,
        eos: IdealGas,
        cfl: f64,
        payload_limit: usize,
        mut initial: impl FnMut([usize; 3], [f64; 3]) -> ConservativeState,
    ) -> Result<Self, CompressibleError> {
        if !cfl.is_finite() || cfl <= 0.0 || cfl > 0.5 {
            return Err(CompressibleError::InvalidParameter { name: "cfl" });
        }
        let n = grid.cell_len();
        let required = n
            .checked_mul(3 * size_of::<ConservativeState>() + size_of::<f64>())
            .ok_or(CompressibleError::CapacityOverflow)?;
        if required > payload_limit {
            return Err(CompressibleError::BufferLimit {
                required,
                limit: payload_limit,
            });
        }
        let mut accepted = reserve(n, [0.0; 5])?;
        let candidate = reserve(n, [0.0; 5])?;
        let change = reserve(n, [0.0; 5])?;
        let rates = reserve(n, 0.0)?;
        let bytes = [accepted.capacity(), candidate.capacity(), change.capacity()]
            .into_iter()
            .try_fold(0usize, |sum, count| {
                count
                    .checked_mul(size_of::<ConservativeState>())
                    .and_then(|v| sum.checked_add(v))
            })
            .and_then(|sum| {
                rates
                    .capacity()
                    .checked_mul(size_of::<f64>())
                    .and_then(|v| sum.checked_add(v))
            })
            .ok_or(CompressibleError::CapacityOverflow)?;
        if bytes > payload_limit {
            return Err(CompressibleError::BufferLimit {
                required: bytes,
                limit: payload_limit,
            });
        }
        let [nx, ny, nz] = grid.counts();
        for z in 0..nz {
            for y in 0..ny {
                for x in 0..nx {
                    let coordinate = [x, y, z];
                    accepted[grid.cell_unchecked(coordinate)] = initial(
                        coordinate,
                        grid.cell_position(coordinate)
                            .expect("validated loop coordinates"),
                    );
                }
            }
        }
        let diagnostics = diagnose(
            &accepted,
            eos,
            grid.cell_volume(),
            CompressibleStage::Initialization,
            0,
            &mut |_| false,
        )?;
        Ok(Self {
            grid,
            eos,
            cfl,
            accepted,
            candidate,
            change,
            rates,
            bytes,
            time: 0.0,
            generation: 0,
            diagnostics,
            initial_totals: diagnostics.totals,
            last_report: None,
        })
    }
    pub fn allocated_bytes(&self) -> usize {
        self.bytes
    }
    pub fn cfl(&self) -> f64 {
        self.cfl
    }
    pub fn view(&self) -> CompressibleView<'_> {
        CompressibleView {
            geometry: &self.grid,
            eos: self.eos,
            conservative: &self.accepted,
            time: self.time,
            generation: self.generation,
            diagnostics: self.diagnostics,
            last_report: self.last_report.as_ref(),
        }
    }

    /// Replace the complete state and initial budget at a finite nonnegative
    /// clock. Success increments generation and clears the last step report.
    pub fn initialize(
        &mut self,
        state: &[ConservativeState],
        time: f64,
    ) -> Result<(), CompressibleError> {
        if state.len() != self.accepted.len() {
            return Err(CompressibleError::StateLength {
                expected: self.accepted.len(),
                actual: state.len(),
            });
        }
        let [nx, ny, _] = self.grid.counts();
        self.initialize_with(time, |p, _| state[p[0] + nx * (p[1] + ny * p[2])])
    }

    /// Callback supplies conservative cell averages; no allocation during calls.
    pub fn initialize_with(
        &mut self,
        time: f64,
        mut state: impl FnMut([usize; 3], [f64; 3]) -> ConservativeState,
    ) -> Result<(), CompressibleError> {
        if !time.is_finite() || time < 0.0 {
            return Err(CompressibleError::InvalidParameter { name: "time" });
        }
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(CompressibleError::GenerationOverflow)?;
        let [nx, ny, nz] = self.grid.counts();
        for z in 0..nz {
            for y in 0..ny {
                for x in 0..nx {
                    let p = [x, y, z];
                    let i = self.grid.cell_unchecked(p);
                    let u = state(
                        p,
                        self.grid
                            .cell_position(p)
                            .expect("validated loop coordinates"),
                    );
                    self.eos
                        .derive(u, CompressibleStage::Initialization, Some(i))
                        .map_err(CompressibleError::InvalidState)?;
                    self.candidate[i] = u;
                }
            }
        }
        let diagnostics = diagnose(
            &self.candidate,
            self.eos,
            self.grid.cell_volume(),
            CompressibleStage::Initialization,
            0,
            &mut |_| false,
        )?;
        std::mem::swap(&mut self.accepted, &mut self.candidate);
        self.time = time;
        self.generation = generation;
        self.diagnostics = diagnostics;
        self.initial_totals = diagnostics.totals;
        self.last_report = None;
        Ok(())
    }

    /// dt is capped by actual incident-face CFL and halved on candidate failure.
    /// At most eight halvings (nine attempts), always from the accepted state.
    pub fn step(
        &mut self,
        requested_dt: f64,
        mut cancel: impl FnMut(CompressibleCheckpoint) -> bool,
    ) -> Result<CompressibleStepReport, CompressibleError> {
        let started = Instant::now();
        if !requested_dt.is_finite() || requested_dt <= 0.0 {
            return Err(CompressibleError::InvalidParameter {
                name: "requested_dt",
            });
        }
        checkpoint(&mut cancel, CompressibleStage::Preflight, 0, None)?;
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(CompressibleError::GenerationOverflow)?;
        let (incident_acoustic_rate, controlling_cell) = self.incident_rates(&mut cancel)?;
        let cfl_bound = self.cfl / incident_acoustic_rate;
        if !cfl_bound.is_finite() || cfl_bound <= 0.0 {
            return Err(CompressibleError::Arithmetic(failure(
                CompressibleStage::Cfl,
                Some(controlling_cell),
                None,
                CompressibleCause::Arithmetic,
            )));
        }
        let mut dt = requested_dt.min(cfl_bound);
        let mut last_rejection = None;
        let mut host_kernel_seconds = 0.0;
        for attempt in 1..=MAX_RETRIES + 1 {
            checkpoint(&mut cancel, CompressibleStage::Candidate, attempt, None)?;
            let time = self.time + dt;
            if !time.is_finite() || time <= self.time {
                return Err(CompressibleError::NonadvancingTime {
                    time: self.time,
                    dt,
                });
            }
            let kernel_started = Instant::now();
            let updated = self.update(dt, attempt, &mut cancel);
            host_kernel_seconds += kernel_started.elapsed().as_secs_f64();
            let candidate = updated.and_then(|()| {
                diagnose(
                    &self.candidate,
                    self.eos,
                    self.grid.cell_volume(),
                    CompressibleStage::Diagnostics,
                    attempt,
                    &mut cancel,
                )
            });
            let diagnostics = match candidate {
                Ok(d) => d,
                // Flux/divergence arithmetic does not depend on dt. Halving
                // cannot repair it, so it is a terminal operator failure.
                Err(CompressibleError::Arithmetic(cause))
                    if cause.stage == CompressibleStage::Candidate =>
                {
                    return Err(CompressibleError::Arithmetic(cause));
                }
                Err(
                    CompressibleError::InvalidState(cause) | CompressibleError::Arithmetic(cause),
                ) => {
                    if attempt == MAX_RETRIES + 1 {
                        return Err(CompressibleError::RetryExhausted {
                            attempts: attempt,
                            last_dt: dt,
                            last: cause,
                        });
                    }
                    last_rejection = Some(cause);
                    dt *= 0.5;
                    continue;
                }
                Err(e) => return Err(e),
            };
            let ledger_residual =
                std::array::from_fn(|d| diagnostics.totals[d] - self.diagnostics.totals[d]);
            let initial_total_drift =
                std::array::from_fn(|d| diagnostics.totals[d] - self.initial_totals[d]);
            if ledger_residual
                .iter()
                .chain(initial_total_drift.iter())
                .any(|v| !v.is_finite())
            {
                return Err(CompressibleError::Arithmetic(failure(
                    CompressibleStage::Diagnostics,
                    None,
                    None,
                    CompressibleCause::Arithmetic,
                )));
            }
            checkpoint(&mut cancel, CompressibleStage::Publication, attempt, None)?;
            let report = CompressibleStepReport {
                requested_dt,
                accepted_dt: dt,
                time,
                generation,
                incident_acoustic_rate,
                cfl_bound,
                controlling_cell,
                diagnostics,
                ledger_residual,
                initial_total_drift,
                attempts: attempt,
                rejections: attempt - 1,
                last_rejection,
                flux_fallbacks: 0,
                method: COMPRESSIBLE_METHOD,
                eos: self.eos,
                counts: self.grid.counts(),
                spacing: self.grid.spacing(),
                origin: self.grid.origin(),
                managed_capacity_bytes: self.bytes,
                host_step_seconds: started.elapsed().as_secs_f64(),
                host_kernel_seconds,
            };
            std::mem::swap(&mut self.accepted, &mut self.candidate);
            self.time = time;
            self.generation = generation;
            self.diagnostics = diagnostics;
            self.last_report = Some(report.clone());
            return Ok(report);
        }
        unreachable!("bounded attempts return on final rejection")
    }

    fn incident_rates(
        &mut self,
        cancel: &mut impl FnMut(CompressibleCheckpoint) -> bool,
    ) -> Result<(f64, usize), CompressibleError> {
        self.rates.fill(0.0);
        let counts = self.grid.counts();
        let spacing = self.grid.spacing();
        for (axis, dx) in spacing.into_iter().enumerate() {
            for i in 0..self.accepted.len() {
                checkpoint(cancel, CompressibleStage::Cfl, 0, Some(i))?;
                let j = neighbor(i, axis, counts);
                let left = self
                    .eos
                    .derive(self.accepted[i], CompressibleStage::Cfl, Some(i))
                    .map_err(CompressibleError::InvalidState)?;
                let right = self
                    .eos
                    .derive(self.accepted[j], CompressibleStage::Cfl, Some(j))
                    .map_err(CompressibleError::InvalidState)?;
                let alpha = (left.velocity[axis].abs() + left.sound_speed)
                    .max(right.velocity[axis].abs() + right.sound_speed);
                let rate = alpha / dx;
                self.rates[i] += rate;
                self.rates[j] += rate;
                if !rate.is_finite() || !self.rates[i].is_finite() || !self.rates[j].is_finite() {
                    return Err(CompressibleError::Arithmetic(failure(
                        CompressibleStage::Cfl,
                        Some(i),
                        None,
                        CompressibleCause::Arithmetic,
                    )));
                }
            }
        }
        let mut control = 0;
        for i in 1..self.rates.len() {
            if self.rates[i] > self.rates[control] {
                control = i;
            }
        }
        Ok((self.rates[control], control))
    }

    fn update(
        &mut self,
        dt: f64,
        attempt: u8,
        cancel: &mut impl FnMut(CompressibleCheckpoint) -> bool,
    ) -> Result<(), CompressibleError> {
        self.change.fill([0.0; 5]);
        let counts = self.grid.counts();
        for (axis, dx) in self.grid.spacing().into_iter().enumerate() {
            for i in 0..self.accepted.len() {
                checkpoint(cancel, CompressibleStage::Candidate, attempt, Some(i))?;
                let j = neighbor(i, axis, counts);
                if i == j {
                    continue;
                } // Self-periodic face has exactly zero divergence.
                let ul = self.accepted[i];
                let ur = self.accepted[j];
                let pl = self
                    .eos
                    .derive(ul, CompressibleStage::Candidate, Some(i))
                    .map_err(CompressibleError::InvalidState)?;
                let pr = self
                    .eos
                    .derive(ur, CompressibleStage::Candidate, Some(j))
                    .map_err(CompressibleError::InvalidState)?;
                let alpha = (pl.velocity[axis].abs() + pl.sound_speed)
                    .max(pr.velocity[axis].abs() + pr.sound_speed);
                let fl = flux(ul, pl, axis);
                let fr = flux(ur, pr, axis);
                for d in 0..5 {
                    let shared = 0.5 * fl[d] + 0.5 * fr[d] - (0.5 * alpha) * (ur[d] - ul[d]);
                    let rate = shared / dx;
                    self.change[i][d] -= rate;
                    self.change[j][d] += rate;
                    if !shared.is_finite()
                        || !rate.is_finite()
                        || !self.change[i][d].is_finite()
                        || !self.change[j][d].is_finite()
                    {
                        return Err(CompressibleError::Arithmetic(failure(
                            CompressibleStage::Candidate,
                            Some(i),
                            Some(d),
                            CompressibleCause::Arithmetic,
                        )));
                    }
                }
            }
        }
        for i in 0..self.accepted.len() {
            checkpoint(cancel, CompressibleStage::Candidate, attempt, Some(i))?;
            self.candidate[i] =
                std::array::from_fn(|d| self.accepted[i][d] + dt * self.change[i][d]);
            self.eos
                .derive(self.candidate[i], CompressibleStage::Candidate, Some(i))
                .map_err(CompressibleError::InvalidState)?;
        }
        Ok(())
    }
}

fn neighbor(i: usize, axis: usize, counts: [usize; 3]) -> usize {
    let stride = [1, counts[0], counts[0] * counts[1]][axis];
    let coordinate = (i / stride) % counts[axis];
    if coordinate + 1 == counts[axis] {
        i - coordinate * stride
    } else {
        i + stride
    }
}

#[derive(Clone, Copy, Default)]
struct Compensated {
    sum: f64,
    correction: f64,
}
impl Compensated {
    fn add(&mut self, value: f64) {
        let next = self.sum + value;
        self.correction += if self.sum.abs() >= value.abs() {
            (self.sum - next) + value
        } else {
            (value - next) + self.sum
        };
        self.sum = next;
    }
    fn total(self) -> f64 {
        self.sum + self.correction
    }
}

fn diagnose(
    state: &[ConservativeState],
    eos: IdealGas,
    volume: f64,
    stage: CompressibleStage,
    attempt: u8,
    cancel: &mut impl FnMut(CompressibleCheckpoint) -> bool,
) -> Result<CompressibleDiagnostics, CompressibleError> {
    let mut totals = [Compensated::default(); 5];
    let mut result = CompressibleDiagnostics {
        totals: [0.0; 5],
        min_density: f64::INFINITY,
        min_pressure: f64::INFINITY,
        min_internal_energy_density: f64::INFINITY,
        min_temperature: f64::INFINITY,
        max_speed: 0.0,
        max_mach: 0.0,
    };
    for (i, &u) in state.iter().enumerate() {
        checkpoint(cancel, stage, attempt, Some(i))?;
        let p = eos
            .derive(u, stage, Some(i))
            .map_err(CompressibleError::InvalidState)?;
        let speed = p.velocity[0].hypot(p.velocity[1]).hypot(p.velocity[2]);
        let mach = speed / p.sound_speed;
        if !speed.is_finite() || !mach.is_finite() {
            return Err(CompressibleError::Arithmetic(failure(
                stage,
                Some(i),
                None,
                CompressibleCause::Arithmetic,
            )));
        }
        result.min_density = result.min_density.min(p.density);
        result.min_pressure = result.min_pressure.min(p.pressure);
        result.min_internal_energy_density = result
            .min_internal_energy_density
            .min(p.internal_energy_density);
        result.min_temperature = result.min_temperature.min(p.temperature);
        result.max_speed = result.max_speed.max(speed);
        result.max_mach = result.max_mach.max(mach);
        for d in 0..5 {
            let weighted = u[d] * volume;
            totals[d].add(weighted);
            if !weighted.is_finite() || !totals[d].total().is_finite() {
                return Err(CompressibleError::Arithmetic(failure(
                    stage,
                    Some(i),
                    Some(d),
                    CompressibleCause::Arithmetic,
                )));
            }
        }
    }
    result.totals = totals.map(Compensated::total);
    Ok(result)
}
