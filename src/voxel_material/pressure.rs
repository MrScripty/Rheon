use super::{
    VoxelMaterialError, VoxelMaterialState, allocate, capacity, check_limit, faces, payload,
};
use crate::{
    Axis, GridGeometry, PressureOperator, PressureReport, PressureSettings, PressureWorkspace,
};

#[derive(Debug, Clone, Copy)]
pub struct ProjectionReport {
    pub pressure: PressureReport,
    pub actual_divergence_max: f64,
}

/// Success-only borrowed projection candidate. It is not an accepted timestep.
pub struct ProjectedVelocity<'a> {
    velocity: &'a [Vec<f64>; 3],
    pressure: &'a [f64],
    report: ProjectionReport,
}
impl ProjectedVelocity<'_> {
    pub fn velocity(&self, axis: Axis) -> &[f64] {
        &self.velocity[axis.index()]
    }
    pub fn pressure(&self) -> &[f64] {
        self.pressure
    }
    pub fn report(&self) -> ProjectionReport {
        self.report
    }
}

/// Jacobi-PCG scratch for a filled box. Payload admission includes the supplied
/// authoritative state and all owned actual Vec capacities, including PCG.
pub struct MaterialProjectionWorkspace {
    grid: GridGeometry,
    velocity: [Vec<f64>; 3],
    face_density: [Vec<f64>; 3],
    rhs: Vec<f64>,
    divergence: Vec<f64>,
    pressure: PressureWorkspace,
    allocated_bytes: usize,
}
impl MaterialProjectionWorkspace {
    pub fn new(state: &VoxelMaterialState, limit: usize) -> Result<Self, VoxelMaterialError> {
        let grid = state.geometry();
        let own_minimum = payload(8, 2, grid)?; // two cell scratch + six PCG; two face fields
        let aggregate = own_minimum
            .checked_add(state.allocated_bytes())
            .ok_or(VoxelMaterialError::CapacityOverflow)?;
        check_limit(aggregate, limit)?;
        let velocity = faces(grid)?;
        let face_density = faces(grid)?;
        let rhs = allocate(grid.cell_len())?;
        let divergence = allocate(grid.cell_len())?;
        let own = capacity(
            velocity
                .iter()
                .chain(face_density.iter())
                .chain([&rhs, &divergence]),
        )?;
        let remaining = limit
            .checked_sub(state.allocated_bytes())
            .and_then(|n| n.checked_sub(own))
            .ok_or(VoxelMaterialError::BufferLimit {
                required: aggregate,
                limit,
            })?;
        let pressure = PressureWorkspace::new(grid, remaining)?;
        let allocated_bytes = own
            .checked_add(pressure.allocated_bytes())
            .ok_or(VoxelMaterialError::CapacityOverflow)?;
        check_limit(
            state
                .allocated_bytes()
                .checked_add(allocated_bytes)
                .ok_or(VoxelMaterialError::CapacityOverflow)?,
            limit,
        )?;
        Ok(Self {
            grid: grid.clone(),
            velocity,
            face_density,
            rhs,
            divergence,
            pressure,
            allocated_bytes,
        })
    }
    pub fn allocated_bytes(&self) -> usize {
        self.allocated_bytes
    }
    /// Does not mutate the authoritative state. Failed/cancelled scratch is not
    /// returned as a success value; pressure and actual divergence both gate it.
    pub fn project<'a>(
        &'a mut self,
        state: &VoxelMaterialState,
        predictor: [&[f64]; 3],
        dt: f64,
        settings: PressureSettings,
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<ProjectedVelocity<'a>, VoxelMaterialError> {
        if state.geometry() != &self.grid {
            return Err(VoxelMaterialError::GeometryMismatch);
        }
        if cancelled() {
            return Err(VoxelMaterialError::Cancelled);
        }
        let counts = self.grid.counts();
        for axis in Axis::ALL {
            let d = axis.index();
            let [nx, ny, nz] = self.grid.face_counts(axis);
            for k in 0..nz {
                for j in 0..ny {
                    for i in 0..nx {
                        let p = [i, j, k];
                        let mut lo = p;
                        let mut hi = p;
                        lo[d] = p[d].saturating_sub(1);
                        hi[d] = p[d].min(counts[d] - 1);
                        let a = state
                            .materials
                            .density(state.liquid_fraction[self.grid.cell_unchecked(lo)]);
                        let b = state
                            .materials
                            .density(state.liquid_fraction[self.grid.cell_unchecked(hi)]);
                        let rho = 0.5 * a + 0.5 * b;
                        if !rho.is_normal() || !(rho * self.grid.cell_volume()).is_normal() {
                            return Err(VoxelMaterialError::ArithmeticFailure);
                        }
                        self.face_density[d][self.grid.face_unchecked(axis, p)] = rho;
                    }
                }
            }
        }
        let operator = PressureOperator::with_face_densities(
            &self.grid,
            self.face_density.each_ref().map(|v| v.as_slice()),
        )?;
        operator.build_rhs_f64(predictor, dt, &mut self.rhs)?;
        for (out, input) in self.velocity.iter_mut().zip(predictor) {
            out.copy_from_slice(input);
        }
        let pressure =
            self.pressure
                .solve_rhs(&operator, &self.rhs, dt, settings, &mut cancelled)?;
        let [x, y, z] = &mut self.velocity;
        operator.correct_candidate_f64(self.pressure.pressure(), dt, [x, y, z])?;
        operator.divergence_f64(
            self.velocity.each_ref().map(|v| v.as_slice()),
            &mut self.divergence,
        )?;
        let actual_divergence_max = self.divergence.iter().fold(0.0_f64, |m, v| m.max(v.abs()));
        if actual_divergence_max > settings.divergence_limit {
            return Err(VoxelMaterialError::Divergence {
                actual: actual_divergence_max,
                limit: settings.divergence_limit,
            });
        }
        if cancelled() {
            return Err(VoxelMaterialError::Cancelled);
        }
        Ok(ProjectedVelocity {
            velocity: &self.velocity,
            pressure: self.pressure.pressure(),
            report: ProjectionReport {
                pressure,
                actual_divergence_max,
            },
        })
    }
}
