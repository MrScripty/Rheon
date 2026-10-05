//! Conservative overlap remap of lumped tangential column profiles. Geometry
//! changes exchange explicit cap parcels. No MAC-face, pressure or phase step
//! is published; this is a prerequisite for a future coupled momentum owner.
use crate::{Axis, ColumnHeight, ColumnSurfaceView, GridGeometry, VolumeStamp};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnMomentumStage {
    BeforeRemap,
    SourceColumn,
    TargetNode,
    BeforeAcceptance,
    BeforeCommit,
}
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnMomentumError {
    GeometryMismatch,
    StampMismatch,
    ShapeMismatch,
    MissingAddedVelocity,
    InvalidDensity,
    InvalidVelocity,
    ArithmeticFailure,
    AcceptanceFailure,
    Cancelled { stage: ColumnMomentumStage },
    AllocationFailed,
    BufferLimit { required: usize, limit: usize },
}
impl fmt::Display for ColumnMomentumError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "column momentum remap rejected operation: {self:?}")
    }
}
impl std::error::Error for ColumnMomentumError {}
#[derive(Clone, Copy)]
pub struct ColumnMomentumInputs<'a> {
    pub before: ColumnSurfaceView<'a>,
    pub after: ColumnSurfaceView<'a>,
    pub density: f64,
    /// Two tangential lumped profiles, stored on the cell lattice. Components
    /// follow ascending tangential axis order. These are NOT MAC face fields.
    pub velocity: [&'a [f32]; 2],
    /// One constant velocity per growing column's admitted cap parcel, m/s.
    /// The full column slice is borrowed and every value is validated.
    pub added_velocity: Option<&'a [[f32; 2]]>,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColumnMomentumReport {
    pub before: VolumeStamp,
    pub after: VolumeStamp,
    pub mass_before: f64,
    pub mass_after: f64,
    pub mass_added: f64,
    pub mass_removed: f64,
    pub mass_error: f64,
    pub mass_budget: f64,
    pub momentum_before: [f64; 2],
    pub momentum_after: [f64; 2],
    pub momentum_added: [f64; 2],
    pub momentum_removed: [f64; 2],
    pub rounding_momentum: [f64; 2],
    pub momentum_error: [f64; 2],
    pub momentum_budget: [f64; 2],
    pub kinetic_before: f64,
    pub kinetic_after: f64,
    pub kinetic_added: f64,
    pub kinetic_removed: f64,
    pub mixing_loss: f64,
    pub rounding_work: f64,
    pub energy_error: f64,
    pub energy_budget: f64,
    pub activated_nodes: usize,
    pub deactivated_nodes: usize,
    pub workspace_bytes: usize,
}
pub struct ColumnMomentumWorkspace {
    grid: GridGeometry,
    axis: Axis,
    mass: Vec<f64>,
    candidate: [Vec<f64>; 2],
    allocated_bytes: usize,
}
fn checked(x: f64) -> Result<f64, ColumnMomentumError> {
    if x == 0.0 || x.is_normal() {
        Ok(x)
    } else {
        Err(ColumnMomentumError::ArithmeticFailure)
    }
}
fn mul(a: f64, b: f64) -> Result<f64, ColumnMomentumError> {
    let x = checked(a * b)?;
    if a != 0.0 && b != 0.0 && x == 0.0 {
        Err(ColumnMomentumError::ArithmeticFailure)
    } else {
        Ok(x)
    }
}
fn div(a: f64, b: f64) -> Result<f64, ColumnMomentumError> {
    if !b.is_normal() || b == 0.0 {
        return Err(ColumnMomentumError::ArithmeticFailure);
    }
    let x = checked(a / b)?;
    if a != 0.0 && x == 0.0 {
        Err(ColumnMomentumError::ArithmeticFailure)
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
    fn add(&mut self, x: f64) -> Result<(), ColumnMomentumError> {
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
    fn finish(self) -> Result<f64, ColumnMomentumError> {
        checked(self.value + self.correction)
    }
}
fn checkpoint(
    cancel: &mut impl FnMut(ColumnMomentumStage) -> bool,
    stage: ColumnMomentumStage,
) -> Result<(), ColumnMomentumError> {
    if cancel(stage) {
        Err(ColumnMomentumError::Cancelled { stage })
    } else {
        Ok(())
    }
}
#[derive(Clone, Copy)]
struct Dual {
    height: f64,
    wet: usize,
    h: f64,
}
impl Dual {
    fn new(height: ColumnHeight, h: f64) -> Result<Self, ColumnMomentumError> {
        Ok(Self {
            height: mul(
                checked(height.full_layers() as f64 + height.top_fraction())?,
                h,
            )?,
            wet: height.full_layers() + usize::from(height.top_fraction() > 0.5),
            h,
        })
    }
    fn interval(self, j: usize) -> Result<[f64; 2], ColumnMomentumError> {
        let low = mul(j as f64, self.h)?;
        let high = if j + 1 == self.wet {
            self.height
        } else {
            mul((j + 1) as f64, self.h)?
        };
        if high <= low {
            return Err(ColumnMomentumError::GeometryMismatch);
        }
        Ok([low, high])
    }
}
fn overlap(a: [f64; 2], b: [f64; 2]) -> Result<f64, ColumnMomentumError> {
    checked((a[1].min(b[1]) - a[0].max(b[0])).max(0.0))
}
impl ColumnMomentumWorkspace {
    /// Three cell-count f64 vectors: candidate mass and two staged profiles.
    /// Actual Vec capacities are charged; no arrays are allocated during remap.
    pub fn new(grid: GridGeometry, axis: Axis, limit: usize) -> Result<Self, ColumnMomentumError> {
        let n = grid.cell_len();
        let required = n
            .checked_mul(24)
            .ok_or(ColumnMomentumError::AllocationFailed)?;
        if required > limit {
            return Err(ColumnMomentumError::BufferLimit { required, limit });
        }
        let mut remaining = limit;
        let mut allocate = || -> Result<Vec<f64>, ColumnMomentumError> {
            let mut v = Vec::new();
            v.try_reserve_exact(n)
                .map_err(|_| ColumnMomentumError::AllocationFailed)?;
            let bytes = v
                .capacity()
                .checked_mul(8)
                .ok_or(ColumnMomentumError::AllocationFailed)?;
            if bytes > remaining {
                return Err(ColumnMomentumError::BufferLimit {
                    required: bytes,
                    limit: remaining,
                });
            }
            remaining -= bytes;
            v.resize(n, 0.0);
            Ok(v)
        };
        let mass = allocate()?;
        let candidate = [allocate()?, allocate()?];
        Ok(Self {
            grid,
            axis,
            mass,
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
    /// Working scratch from the last attempted remap, potentially incomplete
    /// after rejection. It is not accepted mass or a geometry authority.
    pub fn mass_scratch(&self) -> &[f64] {
        &self.mass
    }
    fn cell(&self, column: usize, layer: usize) -> usize {
        let mut p = [0; 3];
        let mut remainder = column;
        for (d, value) in p.iter_mut().enumerate() {
            if d != self.axis.index() {
                *value = remainder % self.grid.counts()[d];
                remainder /= self.grid.counts()[d];
            }
        }
        p[self.axis.index()] = layer;
        self.grid.cell_index(p).unwrap()
    }
    pub fn remap(
        &mut self,
        inputs: ColumnMomentumInputs<'_>,
        output: [&mut [f32]; 2],
        mut cancel: impl FnMut(ColumnMomentumStage) -> bool,
    ) -> Result<ColumnMomentumReport, ColumnMomentumError> {
        let ColumnMomentumInputs {
            before,
            after,
            density,
            velocity,
            added_velocity,
        } = inputs;
        if before.grid() != &self.grid
            || after.grid() != &self.grid
            || before.axis() != self.axis
            || after.axis() != self.axis
        {
            return Err(ColumnMomentumError::GeometryMismatch);
        }
        if before.stamp().id != after.stamp().id
            || before.stamp().version.checked_add(1) != Some(after.stamp().version)
        {
            return Err(ColumnMomentumError::StampMismatch);
        }
        if !density.is_normal() || density <= 0.0 {
            return Err(ColumnMomentumError::InvalidDensity);
        }
        let columns = before.heights().len();
        let layers = self.grid.counts()[self.axis.index()];
        let h = self.grid.spacing()[self.axis.index()];
        if velocity
            .iter()
            .zip(&output)
            .any(|(a, b)| a.len() != self.grid.cell_len() || b.len() != self.grid.cell_len())
            || added_velocity.is_some_and(|v| v.len() != columns)
        {
            return Err(ColumnMomentumError::ShapeMismatch);
        }
        for value in velocity
            .into_iter()
            .flatten()
            .chain(added_velocity.into_iter().flatten().flatten())
        {
            if !(*value == 0.0 || value.is_normal()) {
                return Err(ColumnMomentumError::InvalidVelocity);
            }
        }
        let mut area = 1.0;
        for d in 0..3 {
            if d != self.axis.index() {
                area = mul(area, self.grid.spacing()[d])?;
            }
        }
        let rho_area = mul(density, area)?;
        for column in 0..columns {
            let old = Dual::new(before.heights()[column], h)?;
            let new = Dual::new(after.heights()[column], h)?;
            if new.height > old.height && added_velocity.is_none() {
                return Err(ColumnMomentumError::MissingAddedVelocity);
            }
            for j in old.wet..layers {
                let cell = self.cell(column, j);
                if velocity.iter().any(|v| v[cell] != 0.0) {
                    return Err(ColumnMomentumError::InvalidVelocity);
                }
            }
        }
        checkpoint(&mut cancel, ColumnMomentumStage::BeforeRemap)?;
        self.mass.fill(0.0);
        for v in &mut self.candidate {
            v.fill(0.0);
        }
        let mut mass_before = Sum::default();
        let mut mass_after = Sum::default();
        let mut mass_added = Sum::default();
        let mut mass_removed = Sum::default();
        let mut p_before: [Sum; 2] = std::array::from_fn(|_| Sum::default());
        let mut p_after: [Sum; 2] = std::array::from_fn(|_| Sum::default());
        let mut p_added: [Sum; 2] = std::array::from_fn(|_| Sum::default());
        let mut p_removed: [Sum; 2] = std::array::from_fn(|_| Sum::default());
        let mut p_round: [Sum; 2] = std::array::from_fn(|_| Sum::default());
        let mut p_absolute: [Sum; 2] = std::array::from_fn(|_| Sum::default());
        let mut e_before = Sum::default();
        let mut e_after = Sum::default();
        let mut e_added = Sum::default();
        let mut e_removed = Sum::default();
        let mut mixing = Sum::default();
        let mut work = Sum::default();
        let mut absolute_work = Sum::default();
        let mut activated = 0;
        let mut deactivated = 0;
        for column in 0..columns {
            checkpoint(&mut cancel, ColumnMomentumStage::SourceColumn)?;
            let old = Dual::new(before.heights()[column], h)?;
            let new = Dual::new(after.heights()[column], h)?;
            activated += new.wet.saturating_sub(old.wet);
            deactivated += old.wet.saturating_sub(new.wet);
            for j in 0..old.wet {
                let interval = old.interval(j)?;
                let mass = mul(rho_area, checked(interval[1] - interval[0])?)?;
                let removed = mul(
                    rho_area,
                    overlap(interval, [new.height, old.height.max(new.height)])?,
                )?;
                mass_before.add(mass)?;
                mass_removed.add(removed)?;
                let cell = self.cell(column, j);
                for t in 0..2 {
                    let u = f64::from(velocity[t][cell]);
                    let po = mul(mass, u)?;
                    let pr = mul(removed, u)?;
                    p_before[t].add(po)?;
                    p_removed[t].add(pr)?;
                    p_absolute[t].add(checked(po.abs() + pr.abs())?)?;
                    e_before.add(mul(0.5, mul(mass, mul(u, u)?)?)?)?;
                    e_removed.add(mul(0.5, mul(removed, mul(u, u)?)?)?)?;
                }
            }
            for j in 0..new.wet {
                checkpoint(&mut cancel, ColumnMomentumStage::TargetNode)?;
                let interval = new.interval(j)?;
                let target_mass = mul(rho_area, checked(interval[1] - interval[0])?)?;
                let imported = mul(
                    rho_area,
                    overlap(interval, [old.height, new.height.max(old.height)])?,
                )?;
                let cell = self.cell(column, j);
                self.mass[cell] = target_mass;
                mass_after.add(target_mass)?;
                mass_added.add(imported)?;
                // A last dual slab is at most 1.5h wide. At most two old slabs
                // overlap each target. No dense transfer matrix is retained.
                let first = j.min(old.wet - 1);
                let end = (first + 2).min(old.wet);
                let mut weights = [0.0; 2];
                let mut count = 0;
                let mut row_mass = Sum::default();
                for i in first..end {
                    let weight = mul(rho_area, overlap(interval, old.interval(i)?)?)?;
                    weights[i - first] = weight;
                    if weight > 0.0 {
                        count += 1;
                    }
                    row_mass.add(weight)?;
                }
                row_mass.add(imported)?;
                let partition_error = checked(row_mass.finish()? - target_mass)?;
                let partition_budget = mul(64.0 * f64::EPSILON, mul(2.0, target_mass)?)?;
                if partition_error.abs() > partition_budget {
                    return Err(ColumnMomentumError::AcceptanceFailure);
                }
                for t in 0..2 {
                    let mut momentum = Sum::default();
                    let mut lower = f64::INFINITY;
                    let mut upper = f64::NEG_INFINITY;
                    let mut sole = 0.0;
                    for i in first..end {
                        let weight = weights[i - first];
                        if weight > 0.0 {
                            let u = f64::from(velocity[t][self.cell(column, i)]);
                            momentum.add(mul(weight, u)?)?;
                            lower = lower.min(u);
                            upper = upper.max(u);
                            sole = u;
                        }
                    }
                    if imported > 0.0 {
                        let u = f64::from(added_velocity.unwrap()[column][t]);
                        momentum.add(mul(imported, u)?)?;
                        lower = lower.min(u);
                        upper = upper.max(u);
                        sole = u;
                        let p = mul(imported, u)?;
                        p_added[t].add(p)?;
                        p_absolute[t].add(p.abs())?;
                        e_added.add(mul(0.5, mul(imported, mul(u, u)?)?)?)?;
                    }
                    let proposed = if count + usize::from(imported > 0.0) == 1 {
                        sole
                    } else {
                        div(momentum.finish()?, target_mass)?
                    };
                    let stored = proposed as f32;
                    if !(stored == 0.0 || stored.is_normal()) || (stored == 0.0 && proposed != 0.0)
                    {
                        return Err(ColumnMomentumError::ArithmeticFailure);
                    }
                    let v = f64::from(stored);
                    if proposed < lower || proposed > upper || v < lower || v > upper {
                        return Err(ColumnMomentumError::AcceptanceFailure);
                    }
                    self.candidate[t][cell] = v;
                    let rounding = checked(v - proposed)?;
                    let pn = mul(target_mass, v)?;
                    let pr = mul(target_mass, rounding)?;
                    p_after[t].add(pn)?;
                    p_round[t].add(pr)?;
                    p_absolute[t].add(checked(pn.abs() + pr.abs())?)?;
                    e_after.add(mul(0.5, mul(target_mass, mul(v, v)?)?)?)?;
                    let wr = mul(
                        target_mass,
                        mul(rounding, checked(proposed + mul(0.5, rounding)?)?)?,
                    )?;
                    work.add(wr)?;
                    absolute_work.add(wr.abs())?;
                    for i in first..end {
                        let weight = weights[i - first];
                        if weight > 0.0 {
                            let difference =
                                checked(f64::from(velocity[t][self.cell(column, i)]) - proposed)?;
                            mixing.add(mul(0.5, mul(weight, mul(difference, difference)?)?)?)?;
                        }
                    }
                    if imported > 0.0 {
                        let difference =
                            checked(f64::from(added_velocity.unwrap()[column][t]) - proposed)?;
                        mixing.add(mul(0.5, mul(imported, mul(difference, difference)?)?)?)?;
                    }
                }
            }
        }
        checkpoint(&mut cancel, ColumnMomentumStage::BeforeAcceptance)?;
        let mass_before = mass_before.finish()?;
        let mass_after = mass_after.finish()?;
        let mass_added = mass_added.finish()?;
        let mass_removed = mass_removed.finish()?;
        let mass_error = checked(mass_after - mass_before - mass_added + mass_removed)?;
        let mass_budget = mul(
            64.0 * f64::EPSILON,
            checked(mass_after + mass_before + mass_added + mass_removed)?,
        )?;
        if mass_error.abs() > mass_budget {
            return Err(ColumnMomentumError::AcceptanceFailure);
        }
        let mut momentum_before = [0.0; 2];
        let mut momentum_after = [0.0; 2];
        let mut momentum_added = [0.0; 2];
        let mut momentum_removed = [0.0; 2];
        let mut rounding_momentum = [0.0; 2];
        let mut momentum_error = [0.0; 2];
        let mut momentum_budget = [0.0; 2];
        for t in 0..2 {
            momentum_before[t] = std::mem::take(&mut p_before[t]).finish()?;
            momentum_after[t] = std::mem::take(&mut p_after[t]).finish()?;
            momentum_added[t] = std::mem::take(&mut p_added[t]).finish()?;
            momentum_removed[t] = std::mem::take(&mut p_removed[t]).finish()?;
            rounding_momentum[t] = std::mem::take(&mut p_round[t]).finish()?;
            momentum_error[t] = checked(
                momentum_after[t] - momentum_before[t] - momentum_added[t] + momentum_removed[t]
                    - rounding_momentum[t],
            )?;
            momentum_budget[t] = mul(
                64.0 * f64::EPSILON,
                std::mem::take(&mut p_absolute[t]).finish()?,
            )?;
            if momentum_error[t].abs() > momentum_budget[t] {
                return Err(ColumnMomentumError::AcceptanceFailure);
            }
        }
        let kinetic_before = e_before.finish()?;
        let kinetic_after = e_after.finish()?;
        let kinetic_added = e_added.finish()?;
        let kinetic_removed = e_removed.finish()?;
        let mixing_loss = mixing.finish()?;
        let rounding_work = work.finish()?;
        let absolute_work = absolute_work.finish()?;
        let energy_error = checked(
            kinetic_after - kinetic_before - kinetic_added + kinetic_removed + mixing_loss
                - rounding_work,
        )?;
        let floating_budget = mul(
            64.0 * f64::EPSILON,
            checked(
                kinetic_before
                    + kinetic_after
                    + kinetic_added
                    + kinetic_removed
                    + mixing_loss
                    + absolute_work,
            )?,
        )?;
        let energy_budget = checked(absolute_work + floating_budget)?;
        if energy_error.abs() > floating_budget
            || kinetic_after - kinetic_before - kinetic_added + kinetic_removed > energy_budget
        {
            return Err(ColumnMomentumError::AcceptanceFailure);
        }
        checkpoint(&mut cancel, ColumnMomentumStage::BeforeCommit)?;
        for (out, candidate) in output.into_iter().zip(&self.candidate) {
            for (value, &u) in out.iter_mut().zip(candidate) {
                *value = u as f32;
            }
        }
        Ok(ColumnMomentumReport {
            before: before.stamp(),
            after: after.stamp(),
            mass_before,
            mass_after,
            mass_added,
            mass_removed,
            mass_error,
            mass_budget,
            momentum_before,
            momentum_after,
            momentum_added,
            momentum_removed,
            rounding_momentum,
            momentum_error,
            momentum_budget,
            kinetic_before,
            kinetic_after,
            kinetic_added,
            kinetic_removed,
            mixing_loss,
            rounding_work,
            energy_error,
            energy_budget,
            activated_nodes: activated,
            deactivated_nodes: deactivated,
            workspace_bytes: self.allocated_bytes,
        })
    }
}
