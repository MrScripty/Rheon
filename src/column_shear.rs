//! Fixed flat-column shear quadrature with periodic lateral boundaries.
//! update uses zero endpoint traction; update_with_walls adds finite physical
//! wall friction; update_with_boundaries also supports exact endpoint no-slip.
//! None composes with the sealed carrier pressure pipeline.
use crate::{Axis, ColumnSurfaceView, GridGeometry, VolumeStamp};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnShearStage {
    BeforeGeometry,
    GeometryNode,
    BeforeAssembly,
    ShearEdge,
    BeforeUpdate,
    UpdateNode,
    BeforeAcceptance,
    WallTraction,
    WallConstraint,
}
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnShearError {
    GeometryMismatch,
    VaryingHeight,
    UnsupportedVelocity,
    InvalidDensity,
    InvalidCoefficient,
    InvalidTimeStep,
    IncompatibleNoSlip,
    OverlappingNoSlip,
    ArithmeticFailure,
    StabilityLimit { actual: f64, limit: f64 },
    AcceptanceFailure,
    Cancelled { stage: ColumnShearStage },
    AllocationFailed,
    BufferLimit { required: usize, limit: usize },
}
impl fmt::Display for ColumnShearError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "flat-column shear prerequisite rejected operation: {self:?}"
        )
    }
}
impl std::error::Error for ColumnShearError {}

#[derive(Clone, Copy)]
pub struct ColumnShearInputs<'a> {
    pub surface: ColumnSurfaceView<'a>,
    /// Periodic lateral fields, uniform on each normal layer. Normal velocity
    /// and dry-layer velocities must be zero. No air inertia or extension band.
    pub velocity: [&'a [f32]; 3],
    /// kg/m³, constant. Dynamic viscosity is Pa s; dt is seconds.
    pub density: f64,
    pub dynamic_viscosity: f64,
    pub dt: f64,
}

/// Prescribed tangential velocity (m/s) and finite Navier coefficient
/// (Pa s/m) of a flat stationary-normal wall. Zero friction is free slip.
/// This is mechanical wall traction, not adhesion or a wetting law.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColumnShearWall {
    pub velocity: [f64; 3],
    pub friction: f64,
}

/// Independent tangential laws at the lower and upper fixed slab walls.
/// NoSlip constrains the retained endpoint trace, not an interpolated physical
/// wall node. Its f32 velocity must match the incoming endpoint value exactly;
/// changing prescribed velocities or projecting an incompatible state is out
/// of scope. Positive viscosity and zero normal wall motion are required.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColumnShearBoundary {
    Navier(ColumnShearWall),
    NoSlip { velocity: [f32; 3] },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColumnBoundaryShearReport {
    /// Includes external wall forces and their work in the balance checks.
    pub shear: ColumnShearReport,
    pub boundaries: [ColumnShearBoundary; 2],
    /// Force on liquid (N); at NoSlip this is the constraint reaction.
    pub wall_force: [[f64; 3]; 2],
    /// dt times each total wall force (N s), including finite Navier traction.
    pub wall_impulse: [[f64; 3]; 2],
    /// NoSlip reaction only (N s); zero at Navier walls.
    pub reaction_impulse: [[f64; 3]; 2],
    pub bulk_dissipation_before: f64,
    pub wall_dissipation_before: f64,
    /// Sum of prescribed wall velocity dot wall impulse (J), with either sign.
    pub actuator_work: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColumnWallShearReport {
    /// dissipation_before includes both bulk and relative wall dissipation.
    /// force_sum includes the external wall force; momentum_error subtracts it.
    pub shear: ColumnShearReport,
    pub walls: [ColumnShearWall; 2],
    pub wall_force: [[f64; 3]; 2],
    pub bulk_dissipation_before: f64,
    pub wall_dissipation_before: f64,
    /// dt * sum(wall_velocity dot force_on_fluid), with either sign.
    pub actuator_work: f64,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColumnShearGeometry {
    pub axis: Axis,
    pub stamp: VolumeStamp,
    pub wet_nodes: usize,
    pub cell_height: f64,
    pub height: f64,
    pub area: f64,
    pub liquid_volume: f64,
}
impl ColumnShearGeometry {
    /// Liquid-only row-sum mass length. The last node owns all liquid from
    /// its lower dual boundary to the surface, including an air-centered cap.
    pub fn dual_length(self, layer: usize) -> Option<f64> {
        if layer >= self.wet_nodes {
            None
        } else if layer + 1 == self.wet_nodes {
            Some(self.height - layer as f64 * self.cell_height)
        } else {
            Some(self.cell_height)
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColumnShearReport {
    pub geometry: ColumnShearGeometry,
    pub density: f64,
    pub dynamic_viscosity: f64,
    pub dt: f64,
    /// dt * max_i(sum adjacent stiffnesses / mass_i) <= 1.
    pub stability_number: f64,
    pub kinetic_before: f64,
    pub kinetic_after: f64,
    pub dissipation_before: f64,
    pub update_energy: f64,
    pub rounding_work: f64,
    pub identity_error: f64,
    pub energy_budget: f64,
    pub force_sum: [f64; 3],
    pub force_budget: [f64; 3],
    pub momentum_before: [f64; 3],
    pub momentum_after: [f64; 3],
    pub rounding_momentum: [f64; 3],
    pub momentum_error: [f64; 3],
    pub momentum_budget: [f64; 3],
    pub mass_volume_error: f64,
    pub mass_volume_budget: f64,
    pub workspace_bytes: usize,
}

/// Five normal-count f64 arrays (40 N payload bytes at nominal capacities).
/// Mass, two forces and two staged profiles are scratch, not accepted owners.
/// Geometry is borrowed from the existing column owner on each operation.
pub struct ColumnShearWorkspace {
    grid: GridGeometry,
    axis: Axis,
    mass: Vec<f64>,
    force: [Vec<f64>; 2],
    candidate: [Vec<f64>; 2],
    allocated_bytes: usize,
}
fn checked(x: f64) -> Result<f64, ColumnShearError> {
    if x == 0.0 || x.is_normal() {
        Ok(x)
    } else {
        Err(ColumnShearError::ArithmeticFailure)
    }
}
fn mul(a: f64, b: f64) -> Result<f64, ColumnShearError> {
    let x = checked(a * b)?;
    if a != 0.0 && b != 0.0 && x == 0.0 {
        Err(ColumnShearError::ArithmeticFailure)
    } else {
        Ok(x)
    }
}
fn div(a: f64, b: f64) -> Result<f64, ColumnShearError> {
    if !b.is_normal() || b == 0.0 {
        return Err(ColumnShearError::ArithmeticFailure);
    }
    let x = checked(a / b)?;
    if a != 0.0 && x == 0.0 {
        Err(ColumnShearError::ArithmeticFailure)
    } else {
        Ok(x)
    }
}
#[derive(Default)]
struct Sum {
    value: f64,
    correction: f64,
}
impl Sum {
    fn add(&mut self, x: f64) -> Result<(), ColumnShearError> {
        let next = checked(self.value + x)?;
        let error = if self.value.abs() >= x.abs() {
            checked(checked(self.value - next)? + x)?
        } else {
            checked(checked(x - next)? + self.value)?
        };
        self.correction = checked(self.correction + error)?;
        self.value = next;
        Ok(())
    }
    fn finish(self) -> Result<f64, ColumnShearError> {
        checked(self.value + self.correction)
    }
}
fn checkpoint(
    cancel: &mut impl FnMut(ColumnShearStage) -> bool,
    stage: ColumnShearStage,
) -> Result<(), ColumnShearError> {
    if cancel(stage) {
        Err(ColumnShearError::Cancelled { stage })
    } else {
        Ok(())
    }
}
impl ColumnShearWorkspace {
    pub fn new(grid: GridGeometry, axis: Axis, limit: usize) -> Result<Self, ColumnShearError> {
        let n = grid.counts()[axis.index()];
        let required = n
            .checked_mul(40)
            .ok_or(ColumnShearError::AllocationFailed)?;
        if required > limit {
            return Err(ColumnShearError::BufferLimit { required, limit });
        }
        let mut remaining = limit;
        let mut allocate = || -> Result<Vec<f64>, ColumnShearError> {
            let mut v = Vec::new();
            v.try_reserve_exact(n)
                .map_err(|_| ColumnShearError::AllocationFailed)?;
            let actual = v
                .capacity()
                .checked_mul(8)
                .ok_or(ColumnShearError::AllocationFailed)?;
            if actual > remaining {
                return Err(ColumnShearError::BufferLimit {
                    required: actual,
                    limit: remaining,
                });
            }
            remaining -= actual;
            v.resize(n, 0.0);
            Ok(v)
        };
        let mass = allocate()?;
        let force = [allocate()?, allocate()?];
        let candidate = [allocate()?, allocate()?];
        Ok(Self {
            grid,
            axis,
            mass,
            force,
            candidate,
            allocated_bytes: limit - remaining,
        })
    }
    pub fn allocated_bytes(&self) -> usize {
        self.allocated_bytes
    }
    pub fn grid(&self) -> &GridGeometry {
        &self.grid
    }
    /// Working scratch from the last attempted operation. May be incomplete
    /// after rejection; these arrays are not accepted or published state.
    pub fn mass_scratch(&self) -> &[f64] {
        &self.mass
    }
    /// Tangential components in ascending axis order, including zero dry tails.
    /// Working scratch only, with the same rejection caveat as mass_scratch.
    pub fn force_scratch(&self) -> [&[f64]; 2] {
        [&self.force[0], &self.force[1]]
    }
    fn geometry(
        &self,
        surface: ColumnSurfaceView<'_>,
    ) -> Result<ColumnShearGeometry, ColumnShearError> {
        if surface.grid() != &self.grid || surface.axis() != self.axis {
            return Err(ColumnShearError::GeometryMismatch);
        }
        let first = surface.heights()[0];
        if surface.heights().iter().any(|&h| h != first) {
            return Err(ColumnShearError::VaryingHeight);
        }
        let h = self.grid.spacing()[self.axis.index()];
        let height = mul(
            checked(first.full_layers() as f64 + first.top_fraction())?,
            h,
        )?;
        let wet_nodes = first.full_layers() + usize::from(first.top_fraction() > 0.5);
        if wet_nodes == 0 || wet_nodes >= self.grid.counts()[self.axis.index()] {
            return Err(ColumnShearError::GeometryMismatch);
        }
        let mut area = 1.0;
        for d in 0..3 {
            if d != self.axis.index() {
                area = mul(
                    area,
                    mul(self.grid.spacing()[d], self.grid.counts()[d] as f64)?,
                )?;
            }
        }
        let liquid_volume = mul(area, height)?;
        Ok(ColumnShearGeometry {
            axis: self.axis,
            stamp: surface.stamp(),
            wet_nodes,
            cell_height: h,
            height,
            area,
            liquid_volume,
        })
    }
    fn profile(&self, velocity: [&[f32]; 3], component: usize, layer: usize) -> f64 {
        let mut p = [0; 3];
        p[self.axis.index()] = layer;
        f64::from(velocity[component][self.grid.face_index(Axis::ALL[component], p).unwrap()])
    }
    /// Isolated fixed-flat-surface shear update. Publication to output is
    /// infallible after all gates and the last callback. Lateral boundaries are
    /// periodic; output is NOT admissible as a general sealed carrier state.
    pub fn update(
        &mut self,
        inputs: ColumnShearInputs<'_>,
        output: [&mut [f32]; 3],
        cancel: impl FnMut(ColumnShearStage) -> bool,
    ) -> Result<ColumnShearReport, ColumnShearError> {
        Ok(self.update_impl(inputs, output, None, cancel)?.shear)
    }
    /// Fixed flat slab with prescribed tangential lower/upper wall motion.
    /// Endpoint traces use the existing constant-end-extension basis. The cap
    /// is confined in this operation; it is not an evolving free surface.
    /// Finite beta is supported. Use update_with_boundaries for exact no-slip.
    pub fn update_with_walls(
        &mut self,
        inputs: ColumnShearInputs<'_>,
        walls: [ColumnShearWall; 2],
        output: [&mut [f32]; 3],
        cancel: impl FnMut(ColumnShearStage) -> bool,
    ) -> Result<ColumnWallShearReport, ColumnShearError> {
        let r = self.update_impl(
            inputs,
            output,
            Some(walls.map(ColumnShearBoundary::Navier)),
            cancel,
        )?;
        Ok(ColumnWallShearReport {
            shear: r.shear,
            walls,
            wall_force: r.wall_force,
            bulk_dissipation_before: r.bulk_dissipation_before,
            wall_dissipation_before: r.wall_dissipation_before,
            actuator_work: r.actuator_work,
        })
    }
    /// Exact compatible endpoint constraints, optionally mixed with Navier slip.
    /// Reaction cancels the unconstrained endpoint force, so its increment is
    /// zero. Two NoSlip walls on one wet node are refused because their separate
    /// reaction impulses would be nonunique, even for equal prescribed speeds.
    pub fn update_with_boundaries(
        &mut self,
        inputs: ColumnShearInputs<'_>,
        boundaries: [ColumnShearBoundary; 2],
        output: [&mut [f32]; 3],
        cancel: impl FnMut(ColumnShearStage) -> bool,
    ) -> Result<ColumnBoundaryShearReport, ColumnShearError> {
        self.update_impl(inputs, output, Some(boundaries), cancel)
    }
    fn update_impl(
        &mut self,
        inputs: ColumnShearInputs<'_>,
        output: [&mut [f32]; 3],
        boundaries: Option<[ColumnShearBoundary; 2]>,
        mut cancel: impl FnMut(ColumnShearStage) -> bool,
    ) -> Result<ColumnBoundaryShearReport, ColumnShearError> {
        let ColumnShearInputs {
            surface,
            velocity,
            density,
            dynamic_viscosity: mu,
            dt,
        } = inputs;
        if !density.is_normal() || density <= 0.0 {
            return Err(ColumnShearError::InvalidDensity);
        }
        if mu < 0.0 || !(mu == 0.0 || mu.is_normal()) {
            return Err(ColumnShearError::InvalidCoefficient);
        }
        if !dt.is_normal() || dt <= 0.0 {
            return Err(ColumnShearError::InvalidTimeStep);
        }
        let geom = self.geometry(surface)?;
        let normal = self.axis.index();
        let no_slip = boundaries.map_or([false; 2], |b| {
            b.map(|wall| matches!(wall, ColumnShearBoundary::NoSlip { .. }))
        });
        if no_slip == [true; 2] && geom.wet_nodes == 1 {
            return Err(ColumnShearError::OverlappingNoSlip);
        }
        if no_slip.contains(&true) && mu == 0.0 {
            return Err(ColumnShearError::InvalidCoefficient);
        }
        let walls = boundaries.map(|b| {
            b.map(|wall| match wall {
                ColumnShearBoundary::Navier(wall) => wall,
                ColumnShearBoundary::NoSlip { velocity } => ColumnShearWall {
                    velocity: velocity.map(f64::from),
                    friction: 0.0,
                },
            })
        });
        if let Some(boundaries) = boundaries {
            for boundary in boundaries {
                if let ColumnShearBoundary::NoSlip { velocity } = boundary
                    && velocity.iter().any(|v| !(*v == 0.0 || v.is_normal()))
                {
                    return Err(ColumnShearError::UnsupportedVelocity);
                }
            }
        }
        if let Some(walls) = walls {
            for wall in walls {
                if wall.friction < 0.0
                    || !(wall.friction == 0.0 || wall.friction.is_normal())
                    || (wall.friction > 0.0 && mu == 0.0)
                {
                    return Err(ColumnShearError::InvalidCoefficient);
                }
                if wall.velocity[normal] != 0.0
                    || wall.velocity.iter().any(|v| !(*v == 0.0 || v.is_normal()))
                {
                    return Err(ColumnShearError::UnsupportedVelocity);
                }
            }
        }
        let tangents = match normal {
            0 => [1, 2],
            1 => [0, 2],
            _ => [0, 1],
        };
        for (d, field) in velocity.iter().enumerate() {
            if field.len() != self.grid.face_len(Axis::ALL[d]) || output[d].len() != field.len() {
                return Err(ColumnShearError::GeometryMismatch);
            }
            let m = self.grid.face_counts(Axis::ALL[d]);
            for k in 0..m[2] {
                for j in 0..m[1] {
                    for i in 0..m[0] {
                        let p = [i, j, k];
                        let value = field[self.grid.face_index(Axis::ALL[d], p).unwrap()];
                        if !(value == 0.0 || value.is_normal()) {
                            return Err(ColumnShearError::ArithmeticFailure);
                        }
                        let expected = if d == normal || p[normal] >= geom.wet_nodes {
                            0.0
                        } else {
                            self.profile(velocity, d, p[normal])
                        };
                        if f64::from(value) != expected {
                            return Err(ColumnShearError::UnsupportedVelocity);
                        }
                    }
                }
            }
        }
        if let Some(walls) = walls {
            for (side, layer) in [0, geom.wet_nodes - 1].into_iter().enumerate() {
                if no_slip[side]
                    && tangents
                        .iter()
                        .any(|&d| self.profile(velocity, d, layer) != walls[side].velocity[d])
                {
                    return Err(ColumnShearError::IncompatibleNoSlip);
                }
            }
        }
        checkpoint(&mut cancel, ColumnShearStage::BeforeGeometry)?;
        self.mass.fill(0.0);
        for f in &mut self.force {
            f.fill(0.0);
        }
        let coefficient = div(mul(mu, geom.area)?, geom.cell_height)?;
        let mut volume = Sum::default();
        let mut stability = 0.0_f64;
        for layer in 0..geom.wet_nodes {
            checkpoint(&mut cancel, ColumnShearStage::GeometryNode)?;
            let length = checked(geom.dual_length(layer).unwrap())?;
            if length <= 0.0 {
                return Err(ColumnShearError::GeometryMismatch);
            }
            volume.add(mul(geom.area, length)?)?;
            self.mass[layer] = mul(density, mul(geom.area, length)?)?;
            let degree = usize::from(layer > 0) + usize::from(layer + 1 < geom.wet_nodes);
            let mut row_weight = mul(coefficient, degree as f64)?;
            if let Some(walls) = walls {
                for (wall, end) in walls.iter().zip([0, geom.wet_nodes - 1]) {
                    if layer == end {
                        row_weight = checked(row_weight + mul(wall.friction, geom.area)?)?;
                    }
                }
            }
            let constrained =
                (layer == 0 && no_slip[0]) || (layer + 1 == geom.wet_nodes && no_slip[1]);
            if !constrained {
                stability = stability.max(div(mul(dt, row_weight)?, self.mass[layer])?);
            }
        }
        if stability > 1.0 {
            return Err(ColumnShearError::StabilityLimit {
                actual: stability,
                limit: 1.0,
            });
        }
        let mass_volume_error = checked(volume.finish()? - geom.liquid_volume)?;
        let mass_volume_budget = mul(64.0 * f64::EPSILON, mul(2.0, geom.liquid_volume)?)?;
        if mass_volume_error.abs() > mass_volume_budget {
            return Err(ColumnShearError::AcceptanceFailure);
        }
        checkpoint(&mut cancel, ColumnShearStage::BeforeAssembly)?;
        let mut dissipation = Sum::default();
        for (t, &d) in tangents.iter().enumerate() {
            for edge in 0..geom.wet_nodes - 1 {
                checkpoint(&mut cancel, ColumnShearStage::ShearEdge)?;
                let difference =
                    checked(self.profile(velocity, d, edge + 1) - self.profile(velocity, d, edge))?;
                let transfer = mul(coefficient, difference)?;
                self.force[t][edge] = checked(self.force[t][edge] + transfer)?;
                self.force[t][edge + 1] = checked(self.force[t][edge + 1] - transfer)?;
                dissipation.add(mul(coefficient, mul(difference, difference)?)?)?;
            }
        }
        let bulk_dissipation_before = dissipation.finish()?;
        let mut wall_dissipation = Sum::default();
        let mut wall_power = Sum::default();
        let mut wall_force = [[0.0; 3]; 2];
        if let Some(walls) = walls {
            for (side, (wall, layer)) in walls.iter().zip([0, geom.wet_nodes - 1]).enumerate() {
                let weight = mul(wall.friction, geom.area)?;
                for (t, &d) in tangents.iter().enumerate() {
                    checkpoint(&mut cancel, ColumnShearStage::WallTraction)?;
                    let relative = checked(self.profile(velocity, d, layer) - wall.velocity[d])?;
                    let force = -mul(weight, relative)?;
                    wall_force[side][d] = force;
                    self.force[t][layer] = checked(self.force[t][layer] + force)?;
                    wall_dissipation.add(mul(weight, mul(relative, relative)?)?)?;
                    wall_power.add(mul(wall.velocity[d], force)?)?;
                }
            }
        }
        let mut reaction_impulse = [[0.0; 3]; 2];
        if let Some(walls) = walls {
            for (side, layer) in [0, geom.wet_nodes - 1].into_iter().enumerate() {
                if no_slip[side] {
                    for (t, &d) in tangents.iter().enumerate() {
                        checkpoint(&mut cancel, ColumnShearStage::WallConstraint)?;
                        // The incoming constrained trace equals its fixed wall
                        // speed. Eliminating this row requires zero increment.
                        // Include any Navier force on a shared one-node endpoint.
                        let reaction = -self.force[t][layer];
                        wall_force[side][d] = reaction;
                        reaction_impulse[side][d] = mul(dt, reaction)?;
                        wall_power.add(mul(walls[side].velocity[d], reaction)?)?;
                        self.force[t][layer] = 0.0;
                    }
                }
            }
        }
        let mut wall_impulse = [[0.0; 3]; 2];
        for side in 0..2 {
            for d in 0..3 {
                wall_impulse[side][d] = mul(dt, wall_force[side][d])?;
            }
        }
        let wall_dissipation_before = wall_dissipation.finish()?;
        let actuator_work = mul(dt, wall_power.finish()?)?;
        let wall_net = std::array::from_fn::<_, 3, _>(|d| wall_force[0][d] + wall_force[1][d]);
        let mut force_sum = [0.0; 3];
        let mut force_budget = [0.0; 3];
        for (t, &d) in tangents.iter().enumerate() {
            let mut sum = Sum::default();
            let mut absolute = Sum::default();
            for &force in &self.force[t][..geom.wet_nodes] {
                sum.add(force)?;
                absolute.add(force.abs())?;
            }
            force_sum[d] = sum.finish()?;
            force_budget[d] = mul(
                64.0 * f64::EPSILON,
                checked(absolute.finish()? + wall_force[0][d].abs() + wall_force[1][d].abs())?,
            )?;
            if checked(force_sum[d] - wall_net[d])?.abs() > force_budget[d] {
                return Err(ColumnShearError::AcceptanceFailure);
            }
        }
        checkpoint(&mut cancel, ColumnShearStage::BeforeUpdate)?;
        let mut before = Sum::default();
        let mut after = Sum::default();
        let mut update = Sum::default();
        let mut work = Sum::default();
        let mut absolute_work = Sum::default();
        let mut momentum_before = [0.0; 3];
        let mut momentum_after = [0.0; 3];
        let mut rounding_momentum = [0.0; 3];
        let mut momentum_error = [0.0; 3];
        let mut momentum_budget = [0.0; 3];
        for (t, &d) in tangents.iter().enumerate() {
            let mut old_p = Sum::default();
            let mut new_p = Sum::default();
            let mut round_p = Sum::default();
            let mut absolute_p = Sum::default();
            let mut lower = f64::INFINITY;
            let mut upper = f64::NEG_INFINITY;
            for layer in 0..geom.wet_nodes {
                let u = self.profile(velocity, d, layer);
                lower = lower.min(u);
                upper = upper.max(u);
            }
            if let Some(walls) = walls {
                for wall in walls {
                    if wall.friction > 0.0 {
                        lower = lower.min(wall.velocity[d]);
                        upper = upper.max(wall.velocity[d]);
                    }
                }
            }
            for layer in 0..geom.wet_nodes {
                checkpoint(&mut cancel, ColumnShearStage::UpdateNode)?;
                let old = self.profile(velocity, d, layer);
                let mass = self.mass[layer];
                let delta = div(mul(dt, self.force[t][layer])?, mass)?;
                let proposed = if layer == 0 && no_slip[0] {
                    walls.unwrap()[0].velocity[d]
                } else if layer + 1 == geom.wet_nodes && no_slip[1] {
                    walls.unwrap()[1].velocity[d]
                } else {
                    checked(old + delta)?
                };
                let stored = proposed as f32;
                if !(stored == 0.0 || stored.is_normal()) || (stored == 0.0 && proposed != 0.0) {
                    return Err(ColumnShearError::ArithmeticFailure);
                }
                let new = f64::from(stored);
                let rounding = checked(new - proposed)?;
                if proposed < lower || proposed > upper || new < lower || new > upper {
                    return Err(ColumnShearError::AcceptanceFailure);
                }
                self.candidate[t][layer] = new;
                before.add(mul(0.5, mul(mass, mul(old, old)?)?)?)?;
                after.add(mul(0.5, mul(mass, mul(new, new)?)?)?)?;
                update.add(mul(0.5, mul(mass, mul(delta, delta)?)?)?)?;
                let rounding_work = mul(
                    mass,
                    mul(rounding, checked(proposed + mul(0.5, rounding)?)?)?,
                )?;
                work.add(rounding_work)?;
                absolute_work.add(rounding_work.abs())?;
                let po = mul(mass, old)?;
                let pn = mul(mass, new)?;
                let pr = mul(mass, rounding)?;
                old_p.add(po)?;
                new_p.add(pn)?;
                round_p.add(pr)?;
                absolute_p.add(checked(po.abs() + pn.abs() + pr.abs())?)?;
            }
            momentum_before[d] = old_p.finish()?;
            momentum_after[d] = new_p.finish()?;
            rounding_momentum[d] = round_p.finish()?;
            momentum_error[d] = checked(
                momentum_after[d]
                    - momentum_before[d]
                    - rounding_momentum[d]
                    - mul(dt, wall_net[d])?,
            )?;
            momentum_budget[d] = mul(
                64.0 * f64::EPSILON,
                checked(absolute_p.finish()? + mul(dt, wall_net[d])?.abs())?,
            )?;
            if momentum_error[d].abs() > momentum_budget[d] {
                return Err(ColumnShearError::AcceptanceFailure);
            }
        }
        let kinetic_before = before.finish()?;
        let kinetic_after = after.finish()?;
        let update_energy = update.finish()?;
        let rounding_work = work.finish()?;
        let absolute_work = absolute_work.finish()?;
        let dissipation_before = checked(bulk_dissipation_before + wall_dissipation_before)?;
        let loss = mul(dt, dissipation_before)?;
        let identity_error = checked(
            kinetic_after - kinetic_before + loss - actuator_work - update_energy - rounding_work,
        )?;
        let floating_budget = mul(
            64.0 * f64::EPSILON,
            checked(
                kinetic_before
                    + kinetic_after
                    + loss
                    + update_energy
                    + absolute_work
                    + actuator_work.abs(),
            )?,
        )?;
        let energy_budget = checked(absolute_work + floating_budget)?;
        if identity_error.abs() > floating_budget
            || kinetic_after - kinetic_before > checked(actuator_work + energy_budget)?
        {
            return Err(ColumnShearError::AcceptanceFailure);
        }
        checkpoint(&mut cancel, ColumnShearStage::BeforeAcceptance)?;
        // Every face output shape and candidate value was checked above. There
        // are no callbacks or fallible operations after publication begins.
        for (d, field) in output.into_iter().enumerate() {
            let m = self.grid.face_counts(Axis::ALL[d]);
            for k in 0..m[2] {
                for j in 0..m[1] {
                    for i in 0..m[0] {
                        let p = [i, j, k];
                        let value = if d == normal || p[normal] >= geom.wet_nodes {
                            0.0
                        } else {
                            self.candidate[usize::from(d == tangents[1])][p[normal]] as f32
                        };
                        field[self.grid.face_index(Axis::ALL[d], p).unwrap()] = value;
                    }
                }
            }
        }
        Ok(ColumnBoundaryShearReport {
            shear: ColumnShearReport {
                geometry: geom,
                density,
                dynamic_viscosity: mu,
                dt,
                stability_number: stability,
                kinetic_before,
                kinetic_after,
                dissipation_before,
                update_energy,
                rounding_work,
                identity_error,
                energy_budget,
                force_sum,
                force_budget,
                momentum_before,
                momentum_after,
                rounding_momentum,
                momentum_error,
                momentum_budget,
                mass_volume_error,
                mass_volume_budget,
                workspace_bytes: self.allocated_bytes,
            },
            boundaries: boundaries.unwrap_or(
                [ColumnShearBoundary::Navier(ColumnShearWall {
                    velocity: [0.0; 3],
                    friction: 0.0,
                }); 2],
            ),
            wall_force,
            wall_impulse,
            reaction_impulse,
            bulk_dissipation_before,
            wall_dissipation_before,
            actuator_work,
        })
    }
}
