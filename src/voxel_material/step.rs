use super::transport::{TransportWorkspace, phase_volumes};
use super::viscosity::MaterialViscosityWorkspace;
use super::{
    MaterialPair, MaterialProjectionWorkspace, ProjectionReport, TransportReport, ViscosityReport,
    VoxelMaterialError, VoxelMaterialState, check_limit, checked, coordinate, divide, face_mass,
    multiply, payload, upper,
};
use crate::{Axis, GridGeometry, PressureSettings};

const MAX_STEP_ATTEMPTS: usize = 8;
#[derive(Debug, Clone, Copy)]
pub struct VoxelMaterialConfig {
    pub pressure: PressureSettings,
    pub gravity: [f64; 3],
    /// Aggregate owned array payload, excluding the input and native adapters.
    pub memory_limit: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoxelMaterialStage {
    BeforeTransport,
    Transport,
    Viscosity,
    Gravity,
    Projection,
    BeforeCommit,
}

#[derive(Debug, Clone, Copy)]
pub struct VoxelMaterialStepReport {
    /// Original accepted-state clock, preserved before the interval update.
    pub interval_start: f64,
    pub generation: u64,
    pub time: f64,
    pub dt: f64,
    pub attempts: usize,
    pub transport: TransportReport,
    pub viscosity: ViscosityReport,
    pub projection: ProjectionReport,
    pub phase_masses: [f64; 2],
    pub cumulative_phase_volume_budgets: [f64; 2],
}

/// Serial, filled-box two-material numerical owner. Native exposure is gated
/// separately; this constructor is not a particle or carrier fallback.
pub struct VoxelMaterialSimulation {
    state: VoxelMaterialState,
    candidate: VoxelMaterialState,
    projection: MaterialProjectionWorkspace,
    transport: TransportWorkspace,
    viscosity: MaterialViscosityWorkspace,
    config: VoxelMaterialConfig,
    reference_volumes: [f64; 2],
    accumulated_budgets: [f64; 2],
    report: Option<VoxelMaterialStepReport>,
    allocated_bytes: usize,
}
impl VoxelMaterialSimulation {
    pub fn new(
        grid: GridGeometry,
        materials: MaterialPair,
        fraction: &[f64],
        config: VoxelMaterialConfig,
    ) -> Result<Self, VoxelMaterialError> {
        if grid.counts().iter().any(|n| !(4..=64).contains(n)) {
            return Err(VoxelMaterialError::UnsupportedCounts);
        }
        if config.gravity.iter().any(|v| !v.is_finite())
            || [
                config.pressure.relative_residual,
                config.pressure.absolute_residual,
                config.pressure.divergence_limit,
            ]
            .iter()
            .any(|v| !v.is_finite() || *v < 0.0)
        {
            return Err(VoxelMaterialError::InvalidSettings);
        }
        // Two state buffers + projection(6 PCG cells included) + transport + stress.
        check_limit(payload(14, 9, &grid)?, config.memory_limit)?;
        let state =
            VoxelMaterialState::new(grid.clone(), materials, fraction, config.memory_limit)?;
        let mut used = state.allocated_bytes();
        let candidate = VoxelMaterialState::new(
            grid.clone(),
            materials,
            fraction,
            config.memory_limit - used,
        )?;
        let projection = MaterialProjectionWorkspace::new(&candidate, config.memory_limit - used)?;
        used = used
            .checked_add(candidate.allocated_bytes())
            .and_then(|n| n.checked_add(projection.allocated_bytes()))
            .ok_or(VoxelMaterialError::CapacityOverflow)?;
        let transport = TransportWorkspace::new(&grid, config.memory_limit - used)?;
        used = used
            .checked_add(transport.allocated_bytes())
            .ok_or(VoxelMaterialError::CapacityOverflow)?;
        let viscosity = MaterialViscosityWorkspace::new(&grid, config.memory_limit - used)?;
        used = used
            .checked_add(viscosity.allocated_bytes())
            .ok_or(VoxelMaterialError::CapacityOverflow)?;
        check_limit(used, config.memory_limit)?;
        let reference_volumes = phase_volumes(&grid, fraction)?;
        Ok(Self {
            state,
            candidate,
            projection,
            transport,
            viscosity,
            config,
            reference_volumes,
            accumulated_budgets: [0.0; 2],
            report: None,
            allocated_bytes: used,
        })
    }
    pub fn state(&self) -> &VoxelMaterialState {
        &self.state
    }
    pub fn allocated_bytes(&self) -> usize {
        self.allocated_bytes
    }
    pub fn last_report(&self) -> Option<VoxelMaterialStepReport> {
        self.report
    }
    pub fn config(&self) -> VoxelMaterialConfig {
        self.config
    }
    fn gravity(&mut self, dt: f64) -> Result<(), VoxelMaterialError> {
        let n = self.candidate.grid.counts();
        for axis in Axis::ALL {
            let d = axis.index();
            let dims = self.candidate.grid.face_counts(axis);
            for k in 0..dims[2] {
                for j in 0..dims[1] {
                    for i in 0..dims[0] {
                        let p = [i, j, k];
                        if p[d] == 0 || p[d] == n[d] {
                            continue;
                        }
                        let index = self.candidate.grid.face_unchecked(axis, p);
                        let mass = face_mass(
                            &self.candidate.grid,
                            self.candidate.materials,
                            &self.candidate.liquid_fraction,
                            axis,
                            p,
                        )?;
                        let impulse = multiply(dt, multiply(mass, self.config.gravity[d])?)?;
                        self.candidate.velocity[d][index] =
                            checked(self.candidate.velocity[d][index] + divide(impulse, mass)?)?;
                    }
                }
            }
        }
        Ok(())
    }
    fn final_courant(&self, dt: f64) -> Result<(), VoxelMaterialError> {
        let grid = &self.candidate.grid;
        for cell in 0..grid.cell_len() {
            let p = coordinate(grid, cell);
            let mut incoming = 0.0;
            let mut outgoing = 0.0;
            for axis in Axis::ALL {
                let d = axis.index();
                let mut hi = p;
                hi[d] += 1;
                let lo = self.candidate.velocity[d][grid.face_unchecked(axis, p)];
                let hi = self.candidate.velocity[d][grid.face_unchecked(axis, hi)];
                incoming =
                    checked(incoming + divide(lo.max(0.0) + (-hi).max(0.0), grid.spacing()[d])?)?;
                outgoing =
                    checked(outgoing + divide(hi.max(0.0) + (-lo).max(0.0), grid.spacing()[d])?)?;
            }
            let actual = multiply(dt, incoming.max(outgoing))?;
            if actual > 0.5 {
                return Err(VoxelMaterialError::Courant { actual, limit: 0.5 });
            }
        }
        Ok(())
    }
    fn attempt(
        &mut self,
        dt: f64,
        attempts: usize,
        cancelled: &mut impl FnMut(VoxelMaterialStage) -> bool,
    ) -> Result<VoxelMaterialStepReport, VoxelMaterialError> {
        if cancelled(VoxelMaterialStage::BeforeTransport) {
            return Err(VoxelMaterialError::Cancelled);
        }
        let transport = self.transport.advect(
            &self.state,
            dt,
            self.config.pressure.divergence_limit,
            &mut self.candidate.liquid_fraction,
            &mut self.candidate.velocity,
            || cancelled(VoxelMaterialStage::Transport),
        )?;
        let viscosity = self.viscosity.apply(&mut self.candidate, dt, || {
            cancelled(VoxelMaterialStage::Viscosity)
        })?;
        if cancelled(VoxelMaterialStage::Gravity) {
            return Err(VoxelMaterialError::Cancelled);
        }
        self.gravity(dt)?;
        let projected = self.projection.project(
            &self.candidate,
            self.candidate.velocity.each_ref().map(|v| v.as_slice()),
            dt,
            self.config.pressure,
            || cancelled(VoxelMaterialStage::Projection),
        )?;
        let projection = projected.report();
        for axis in Axis::ALL {
            self.candidate.velocity[axis.index()].copy_from_slice(projected.velocity(axis));
        }
        self.candidate
            .pressure
            .copy_from_slice(projected.pressure());
        self.final_courant(dt)?;
        let mut accumulated = [0.0; 2];
        for (phase, accumulated_phase) in accumulated.iter_mut().enumerate() {
            *accumulated_phase =
                upper(self.accumulated_budgets[phase] + transport.phase_budgets[phase])?;
            let actual = (transport.phase_volumes[phase] - self.reference_volumes[phase]).abs();
            if actual > *accumulated_phase {
                return Err(VoxelMaterialError::Balance {
                    stage: "cumulative phase volume",
                    actual,
                    budget: *accumulated_phase,
                });
            }
        }
        let generation = self
            .state
            .generation
            .checked_add(1)
            .ok_or(VoxelMaterialError::ClockOverflow)?;
        let time = checked(self.state.time + dt)?;
        if time <= self.state.time {
            return Err(VoxelMaterialError::ClockOverflow);
        }
        let phase_masses = [
            multiply(
                transport.phase_volumes[0],
                self.state.materials.liquid().density(),
            )?,
            multiply(
                transport.phase_volumes[1],
                self.state.materials.gas().density(),
            )?,
        ];
        let report = VoxelMaterialStepReport {
            interval_start: self.state.time,
            generation,
            time,
            dt,
            attempts,
            transport,
            viscosity,
            projection,
            phase_masses,
            cumulative_phase_volume_budgets: accumulated,
        };
        self.candidate.generation = generation;
        self.candidate.time = time;
        self.candidate.pressure_available = true;
        if cancelled(VoxelMaterialStage::BeforeCommit) {
            return Err(VoxelMaterialError::Cancelled);
        }
        // No allocation, callback or other fallible operation follows commit.
        std::mem::swap(&mut self.state, &mut self.candidate);
        self.accumulated_budgets = accumulated;
        self.report = Some(report);
        Ok(report)
    }
    /// At most eight disposable attempts may shorten dt for the explicit
    /// volume/mass Courant or stress bound. All retries start from accepted
    /// fields; failure never advances time, pressure, fraction or generation.
    pub fn step(
        &mut self,
        requested_dt: f64,
        mut cancelled: impl FnMut(VoxelMaterialStage) -> bool,
    ) -> Result<VoxelMaterialStepReport, VoxelMaterialError> {
        if !requested_dt.is_finite() || requested_dt <= 0.0 {
            return Err(VoxelMaterialError::InvalidTimeStep);
        }
        let mut dt = requested_dt;
        for attempt in 1..=MAX_STEP_ATTEMPTS {
            match self.attempt(dt, attempt, &mut cancelled) {
                Ok(report) => return Ok(report),
                Err(error) => {
                    let ratio = match error {
                        VoxelMaterialError::Courant { actual, limit } => Some(limit / actual),
                        VoxelMaterialError::ViscousStability { number } => Some(1.0 / number),
                        _ => None,
                    };
                    if attempt == MAX_STEP_ATTEMPTS || ratio.is_none() {
                        return Err(error);
                    }
                    dt = multiply(dt, ratio.ok_or(VoxelMaterialError::ArithmeticFailure)?)?
                        .next_down();
                    if !dt.is_normal() || dt <= 0.0 {
                        return Err(VoxelMaterialError::InvalidTimeStep);
                    }
                }
            }
        }
        Err(VoxelMaterialError::ArithmeticFailure)
    }
}
