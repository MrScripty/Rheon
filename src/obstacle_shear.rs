//! Reduced fully developed Newtonian shear above the same retained box obstacle.
//! Tangential directions are periodic/invariant; this is NOT arbitrary 3D
//! cut-cell tensor viscosity or the sealed pressure projection's boundary model.
use crate::obstacle_pressure::{
    Sum, allocate, checked, checkpoint, coordinate, div, gate, mul, positive,
};
use crate::{Axis, ObstacleFlowError, ObstacleFlowStage, StaticObstacleGeometry};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ObstacleShearWall {
    NoSlip,
    /// beta in Pa s/m. Zero is free slip. This is friction, not adhesion.
    Navier {
        beta: f64,
    },
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ObstacleShearForce {
    /// N/m³. Density enters inertia only.
    Density(f64),
    /// m/s². Converted to rho*a in N/m³.
    Acceleration(f64),
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObstacleShearReport {
    pub dt: f64,
    pub force_density: f64,
    pub kinetic_before: f64,
    pub kinetic_after: f64,
    pub increment_energy: f64,
    pub body_work: f64,
    /// dt v^T K v, J, including the stationary wall closure.
    pub dissipation: f64,
    /// Part of dissipation in the effective lower/upper boundary conductances.
    /// A Navier lower wall combines fluid half-layer and wall-friction loss.
    pub boundary_dissipation: f64,
    pub energy_identity_error: f64,
    pub energy_rounding_budget: f64,
    pub momentum_before: f64,
    pub momentum_after: f64,
    pub body_impulse: f64,
    pub wall_impulse: f64,
    pub momentum_identity_error: f64,
    pub momentum_rounding_budget: f64,
    /// Maximum true backward-Euler momentum equation residual, N.
    pub residual_max: f64,
}

/// One tangential velocity per fluid layer, sampled at its fluid centroid.
/// A single box must cover the two tangential directions and extend from below
/// the container to one stationary interior wall. Fluid above that wall is a
/// connected extruded channel. The upper wall is stationary no-slip. Both
/// tangential directions are invariant/periodic, so no end-wall model is hidden.
///
/// Masses use sums of the owner's actual cell volumes; interlayer traction uses
/// sums of its shared open areas. Lower wall location comes from the retained
/// mesh's admitted bounds. No second collision mesh/voxel mask is reconstructed.
/// Six layer-sized Vec payloads are capped; the borrowed owner, allocator, stack,
/// caller field and RSS are excluded. Steps allocate no heap buffers and only
/// publish the caller's velocity after every gate succeeds.
pub struct StaticObstacleShear<'a> {
    geometry: &'a StaticObstacleGeometry,
    normal: Axis,
    first: usize,
    density: f64,
    viscosity: f64,
    lower_wall: ObstacleShearWall,
    volumes: Vec<f64>,
    centers: Vec<f64>,
    conductance: Vec<f64>,
    candidate: Vec<f64>,
    diagonal: Vec<f64>,
    rhs: Vec<f64>,
    lower_conductance: f64,
    upper_conductance: f64,
    allocated_bytes: usize,
}
impl<'a> StaticObstacleShear<'a> {
    pub fn new(
        geometry: &'a StaticObstacleGeometry,
        normal: Axis,
        density: f64,
        viscosity: f64,
        lower_wall: ObstacleShearWall,
        limit: usize,
        mut cancel: impl FnMut(ObstacleFlowStage, usize) -> bool,
    ) -> Result<Self, ObstacleFlowError> {
        positive(density)?;
        positive(viscosity)?;
        if let ObstacleShearWall::Navier { beta } = lower_wall
            && (!beta.is_finite() || beta < 0.0 || beta.is_subnormal())
        {
            return Err(ObstacleFlowError::InvalidParameter);
        }
        let g = geometry.grid();
        let d = normal.index();
        let (lo, hi) = geometry.box_bounds();
        if lo[d] > g.origin()[d]
            || hi[d] < g.origin()[d]
            || hi[d] >= g.upper()[d]
            || (0..3).any(|a| a != d && (lo[a] > g.origin()[a] || hi[a] < g.upper()[a]))
        {
            return Err(ObstacleFlowError::UnsupportedShearGeometry);
        }
        let wall = hi[d];
        let mut first = 0;
        while first < g.counts()[d] && g.origin()[d] + (first + 1) as f64 * g.spacing()[d] <= wall {
            first += 1;
        }
        let n = g.counts()[d] - first;
        if n == 0 {
            return Err(ObstacleFlowError::UnsupportedShearGeometry);
        }
        let planned = n
            .checked_mul(6 * 8)
            .ok_or(ObstacleFlowError::CapacityOverflow)?;
        gate(planned, limit)?;
        let mut used = 0;
        let mut volumes = allocate(n, 0.0, &mut used, limit)?;
        let mut centers = allocate(n, 0.0, &mut used, limit)?;
        // Entry j is the edge from layer j to j+1; the last entry is zero.
        let mut conductance = allocate(n, 0.0, &mut used, limit)?;
        let candidate = allocate(n, 0.0, &mut used, limit)?;
        let diagonal = allocate(n, 0.0, &mut used, limit)?;
        let rhs = allocate(n, 0.0, &mut used, limit)?;
        for cell in 0..g.cell_len() {
            checkpoint(&mut cancel, ObstacleFlowStage::Assembly, cell)?;
            let p = coordinate(g.counts(), cell);
            let v = geometry.fluid_volumes()[cell];
            if p[d] < first {
                if v != 0.0 {
                    return Err(ObstacleFlowError::UnsupportedShearGeometry);
                }
            } else {
                if v <= 0.0 {
                    return Err(ObstacleFlowError::UnsupportedShearGeometry);
                }
                volumes[p[d] - first] = checked(volumes[p[d] - first] + v)?;
            }
        }
        for (j, center) in centers.iter_mut().enumerate() {
            let low = (g.origin()[d] + (j + first) as f64 * g.spacing()[d]).max(wall);
            let high = g.origin()[d] + (j + first + 1) as f64 * g.spacing()[d];
            *center = checked(low + mul(0.5, positive(high - low)?)?)?;
            positive(volumes[j])?;
            positive(mul(density, volumes[j])?)?;
        }
        let area_at = |layer: usize| -> Result<f64, ObstacleFlowError> {
            let mut sum = Sum::default();
            for face in 0..g.face_len(normal) {
                let p = coordinate(g.face_counts(normal), face);
                if p[d] == layer {
                    sum.add(geometry.open_areas(normal)[face])?;
                }
            }
            positive(sum.finish()?)
        };
        for j in 0..n - 1 {
            conductance[j] = positive(div(
                mul(viscosity, area_at(j + first + 1)?)?,
                positive(centers[j + 1] - centers[j])?,
            )?)?;
        }
        // The admitted box covers the entire cross-section; its planar wall has
        // the same area as the fully open top section supplied by this owner.
        let area = area_at(g.counts()[d])?;
        let lower_distance = positive(centers[0] - wall)?;
        let lower_conductance = match lower_wall {
            ObstacleShearWall::NoSlip => positive(div(mul(viscosity, area)?, lower_distance)?)?,
            ObstacleShearWall::Navier { beta } => {
                if beta == 0.0 {
                    0.0
                } else {
                    positive(div(
                        mul(mul(area, viscosity)?, beta)?,
                        checked(viscosity + mul(beta, lower_distance)?)?,
                    )?)?
                }
            }
        };
        let upper_conductance = positive(div(
            mul(viscosity, area)?,
            positive(g.upper()[d] - centers[n - 1])?,
        )?)?;
        Ok(Self {
            geometry,
            normal,
            first,
            density,
            viscosity,
            lower_wall,
            volumes,
            centers,
            conductance,
            candidate,
            diagonal,
            rhs,
            lower_conductance,
            upper_conductance,
            allocated_bytes: used,
        })
    }
    pub fn geometry(&self) -> &'a StaticObstacleGeometry {
        self.geometry
    }
    pub fn normal(&self) -> Axis {
        self.normal
    }
    pub fn first_fluid_layer(&self) -> usize {
        self.first
    }
    pub fn density(&self) -> f64 {
        self.density
    }
    pub fn dynamic_viscosity(&self) -> f64 {
        self.viscosity
    }
    pub fn lower_wall(&self) -> ObstacleShearWall {
        self.lower_wall
    }
    pub fn layer_volumes(&self) -> &[f64] {
        &self.volumes
    }
    pub fn layer_centers(&self) -> &[f64] {
        &self.centers
    }
    pub fn interior_conductances(&self) -> &[f64] {
        &self.conductance[..self.conductance.len() - 1]
    }
    pub fn wall_conductances(&self) -> [f64; 2] {
        [self.lower_conductance, self.upper_conductance]
    }
    pub fn allocated_bytes(&self) -> usize {
        self.allocated_bytes
    }
    fn stiffness(&self, j: usize, u: &[f64]) -> Result<f64, ObstacleFlowError> {
        let n = u.len();
        let mut k = 0.0;
        if j > 0 {
            k = checked(k + mul(self.conductance[j - 1], checked(u[j] - u[j - 1])?)?)?;
        }
        if j + 1 < n {
            k = checked(k + mul(self.conductance[j], checked(u[j] - u[j + 1])?)?)?;
        }
        if j == 0 {
            k = checked(k + mul(self.lower_conductance, u[j])?)?;
        }
        if j + 1 == n {
            k = checked(k + mul(self.upper_conductance, u[j])?)?;
        }
        Ok(k)
    }
    /// One backward-Euler update (M + dt K)v = M u + dt q V.
    /// Time is seconds, mu is Pa s, rho is kg/m³, q is N/m³, speed is m/s.
    /// There is no hidden substep and no convergence/time-accuracy claim.
    pub fn step(
        &mut self,
        velocity: &mut [f64],
        dt: f64,
        force: ObstacleShearForce,
        mut cancel: impl FnMut(ObstacleFlowStage, usize) -> bool,
    ) -> Result<ObstacleShearReport, ObstacleFlowError> {
        positive(dt)?;
        let n = self.volumes.len();
        if velocity.len() != n {
            return Err(ObstacleFlowError::ShapeMismatch);
        }
        if velocity.iter().any(|v| !v.is_finite()) {
            return Err(ObstacleFlowError::NonFiniteInput);
        }
        let q = match force {
            ObstacleShearForce::Density(q) => checked(q)?,
            ObstacleShearForce::Acceleration(a) => mul(self.density, checked(a)?)?,
        };
        for (j, &old) in velocity.iter().enumerate() {
            checkpoint(&mut cancel, ObstacleFlowStage::Assembly, j)?;
            let mass = mul(self.density, self.volumes[j])?;
            let mut k = 0.0;
            if j > 0 {
                k = checked(k + self.conductance[j - 1])?;
            }
            if j + 1 < n {
                k = checked(k + self.conductance[j])?;
            }
            if j == 0 {
                k = checked(k + self.lower_conductance)?;
            }
            if j + 1 == n {
                k = checked(k + self.upper_conductance)?;
            }
            self.diagonal[j] = positive(checked(mass + mul(dt, k)?)?)?;
            self.rhs[j] = checked(mul(mass, old)? + mul(dt, mul(q, self.volumes[j])?)?)?;
        }
        // Thomas elimination on the strictly diagonally dominant M+dt K.
        for j in 1..n {
            checkpoint(&mut cancel, ObstacleFlowStage::Solve, j)?;
            let off = -mul(dt, self.conductance[j - 1])?;
            let ratio = div(off, self.diagonal[j - 1])?;
            self.diagonal[j] = positive(checked(self.diagonal[j] - mul(ratio, off)?)?)?;
            self.rhs[j] = checked(self.rhs[j] - mul(ratio, self.rhs[j - 1])?)?;
        }
        self.candidate[n - 1] = div(self.rhs[n - 1], self.diagonal[n - 1])?;
        for j in (0..n - 1).rev() {
            checkpoint(&mut cancel, ObstacleFlowStage::Correction, j)?;
            self.candidate[j] = div(
                checked(self.rhs[j] + mul(mul(dt, self.conductance[j])?, self.candidate[j + 1])?)?,
                self.diagonal[j],
            )?;
        }
        let mut before = Sum::default();
        let mut after = Sum::default();
        let mut increment = Sum::default();
        let mut work = Sum::default();
        let mut loss = Sum::default();
        let mut momentum_before = Sum::default();
        let mut momentum_after = Sum::default();
        let mut body = Sum::default();
        let mut residual_max = 0.0f64;
        let mut residual_scale = 0.0f64;
        for (j, &old) in velocity.iter().enumerate() {
            checkpoint(&mut cancel, ObstacleFlowStage::Acceptance, j)?;
            let new = self.candidate[j];
            let difference = checked(new - old)?;
            let mass = mul(self.density, self.volumes[j])?;
            let load = mul(q, self.volumes[j])?;
            before.add(mul(0.5, mul(mass, mul(old, old)?)?)?)?;
            after.add(mul(0.5, mul(mass, mul(new, new)?)?)?)?;
            increment.add(mul(0.5, mul(mass, mul(difference, difference)?)?)?)?;
            work.add(mul(dt, mul(load, new)?)?)?;
            momentum_before.add(mul(mass, old)?)?;
            momentum_after.add(mul(mass, new)?)?;
            body.add(mul(dt, load)?)?;
            if j + 1 < n {
                let jump = checked(self.candidate[j + 1] - new)?;
                loss.add(mul(dt, mul(self.conductance[j], mul(jump, jump)?)?)?)?;
            }
            let inertia = div(mul(mass, difference)?, dt)?;
            let stress = self.stiffness(j, &self.candidate)?;
            let residual = checked(checked(inertia + stress)? - load)?;
            residual_max = residual_max.max(residual.abs());
            residual_scale = residual_scale.max(checked(
                checked(inertia.abs() + stress.abs())? + load.abs(),
            )?);
        }
        let low = mul(self.lower_conductance, self.candidate[0])?;
        let high = mul(self.upper_conductance, self.candidate[n - 1])?;
        let boundary_dissipation = mul(
            dt,
            checked(mul(low, self.candidate[0])? + mul(high, self.candidate[n - 1])?)?,
        )?;
        loss.add(boundary_dissipation)?;
        let kinetic_before = before.finish()?;
        let kinetic_after = after.finish()?;
        let increment_energy = increment.finish()?;
        let body_work = work.finish()?;
        let dissipation = loss.finish()?;
        let energy_identity_error = checked(
            checked(
                checked(kinetic_after - kinetic_before)? + checked(increment_energy + dissipation)?,
            )? - body_work,
        )?;
        let energy_rounding_budget = mul(
            2048.0 * f64::EPSILON,
            checked(
                checked(kinetic_before + kinetic_after)?
                    + checked(checked(increment_energy + dissipation)? + body_work.abs())?,
            )?,
        )?;
        let momentum_before = momentum_before.finish()?;
        let momentum_after = momentum_after.finish()?;
        let body_impulse = body.finish()?;
        let wall_impulse = -mul(dt, checked(low + high)?)?;
        let momentum_identity_error = checked(
            checked(momentum_after - momentum_before)? - checked(body_impulse + wall_impulse)?,
        )?;
        let momentum_rounding_budget = mul(
            2048.0 * f64::EPSILON,
            checked(
                checked(momentum_before.abs() + momentum_after.abs())?
                    + checked(body_impulse.abs() + wall_impulse.abs())?,
            )?,
        )?;
        if energy_identity_error.abs() > energy_rounding_budget
            || momentum_identity_error.abs() > momentum_rounding_budget
            || residual_max > mul(4096.0 * f64::EPSILON, residual_scale)?
        {
            return Err(ObstacleFlowError::EnergyLimit);
        }
        if q == 0.0 && kinetic_after > checked(kinetic_before + energy_rounding_budget)? {
            return Err(ObstacleFlowError::EnergyLimit);
        }
        checkpoint(&mut cancel, ObstacleFlowStage::Acceptance, n)?;
        velocity.copy_from_slice(&self.candidate);
        Ok(ObstacleShearReport {
            dt,
            force_density: q,
            kinetic_before,
            kinetic_after,
            increment_energy,
            body_work,
            dissipation,
            boundary_dissipation,
            energy_identity_error,
            energy_rounding_budget,
            momentum_before,
            momentum_after,
            body_impulse,
            wall_impulse,
            momentum_identity_error,
            momentum_rounding_budget,
            residual_max,
        })
    }
}
