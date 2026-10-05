//! Restricted 2.5D flow u=(a,0,w), atmospheric relative pressure zero.
//! A material translating frame makes relative xy transport exactly zero;
//! w evolves by a bounded weak Neumann viscosity solve on the same fitted mesh.
use crate::{
    FittedHeightError, FittedHeightGeometry, FittedHeightPlan, FittedHeightSettings,
    FittedHeightWorkspace,
};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslatedViscousStage {
    BeforeSolve,
    Iteration,
    BeforeAcceptance,
    BeforePublish,
}
#[derive(Debug, Clone, PartialEq)]
pub enum TranslatedViscousError {
    Fitted(FittedHeightError),
    InvalidSettings,
    ShapeMismatch,
    UnsupportedVelocity,
    ArithmeticFailure,
    AllocationFailed,
    BufferLimit { required: usize, limit: usize },
    IterationLimit,
    AcceptanceFailure,
    TimeResolution,
    CoordinateResolution,
    VersionOverflow,
    Cancelled { stage: TranslatedViscousStage },
}
impl fmt::Display for TranslatedViscousError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "translated viscous flow rejected: {self:?}")
    }
}
impl std::error::Error for TranslatedViscousError {}
impl From<FittedHeightError> for TranslatedViscousError {
    fn from(e: FittedHeightError) -> Self {
        Self::Fitted(e)
    }
}
pub(crate) fn check(x: f64) -> Result<f64, TranslatedViscousError> {
    if x == 0.0 || x.is_normal() {
        Ok(x)
    } else {
        Err(TranslatedViscousError::ArithmeticFailure)
    }
}
pub(crate) fn add(a: f64, b: f64) -> Result<f64, TranslatedViscousError> {
    check(a + b)
}
pub(crate) fn mul(a: f64, b: f64) -> Result<f64, TranslatedViscousError> {
    let x = check(a * b)?;
    if a != 0.0 && b != 0.0 && x == 0.0 {
        Err(TranslatedViscousError::ArithmeticFailure)
    } else {
        Ok(x)
    }
}
pub(crate) fn div(a: f64, b: f64) -> Result<f64, TranslatedViscousError> {
    if !b.is_normal() {
        return Err(TranslatedViscousError::ArithmeticFailure);
    }
    let x = check(a / b)?;
    if a != 0.0 && x == 0.0 {
        Err(TranslatedViscousError::ArithmeticFailure)
    } else {
        Ok(x)
    }
}
pub(crate) fn dot(a: &[f64], b: &[f64]) -> Result<f64, TranslatedViscousError> {
    let mut s = 0.0;
    for (&x, &y) in a.iter().zip(b) {
        s = add(s, mul(x, y)?)?;
    }
    Ok(s)
}
pub(crate) fn norm(a: &[f64]) -> Result<f64, TranslatedViscousError> {
    check(dot(a, a)?.sqrt())
}
fn checkpoint(
    cancel: &mut impl FnMut(TranslatedViscousStage) -> bool,
    s: TranslatedViscousStage,
) -> Result<(), TranslatedViscousError> {
    if cancel(s) {
        Err(TranslatedViscousError::Cancelled { stage: s })
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TranslatedViscousSettings {
    pub memory_limit: usize,
    pub max_iterations: usize,
    pub relative_residual: f64,
    pub absolute_residual: f64,
    pub momentum_limit: f64,
    pub divergence_limit: f64,
}
impl Default for TranslatedViscousSettings {
    fn default() -> Self {
        Self {
            memory_limit: 64 << 20,
            max_iterations: 512,
            relative_residual: 1e-12,
            absolute_residual: 1e-13,
            momentum_limit: 1e-11,
            divergence_limit: 1e-11,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TranslatedViscousStamp {
    pub id: u64,
    pub version: u64,
}
#[derive(Debug, Clone, Copy)]
pub struct TranslatedViscousReport {
    pub before: TranslatedViscousStamp,
    pub after: TranslatedViscousStamp,
    pub time_before: f64,
    pub time_after: f64,
    pub offset_before: f64,
    pub offset_after: f64,
    pub liquid_mass: f64,
    pub momentum_before: [f64; 3],
    pub momentum_after: [f64; 3],
    pub energy_before: f64,
    pub energy_after: f64,
    pub increment_energy: f64,
    pub strain_power: f64,
    pub true_residual: f64,
    pub residual_work: f64,
    pub work_identity_error: f64,
    pub divergence_max: f64,
    pub iterations: usize,
}
/// One geometry authority: immutable co-moving template plus accepted offset.
/// Pressure is analytically atmospheric; this type is not a general pressure solve.
pub struct TranslatedViscousState<'a> {
    pub template: &'a FittedHeightWorkspace,
    pub velocity: &'a [[f64; 3]],
    pub time: f64,
    pub offset: f64,
    pub stamp: TranslatedViscousStamp,
    pub density: f64,
}
impl TranslatedViscousState<'_> {
    pub fn physical_node(&self, index: usize) -> Option<[f64; 2]> {
        let p = self.template.nodes().get(index)?.position;
        Some([p[0] + self.offset, p[1]])
    }
    pub fn nodal_liquid_volume(&self, index: usize) -> Option<f64> {
        self.template
            .nodal_mass()
            .get(index)
            .map(|m| m / self.density)
    }
    /// Actual fluid-minus-mesh flux is zero on every material xy face.
    pub fn relative_xy_flux(&self) -> f64 {
        0.0
    }
    pub fn relative_pressure(&self) -> f64 {
        0.0
    }
}
/// Dedicated opt-in owner for this invariant family. It contains no legacy MAC
/// state, independent phase owner or trajectory history. General stepping refusal
/// in LiquidTransportSimulation is unchanged.
pub struct TranslatedViscousFlow {
    frame: FittedHeightWorkspace,
    density: f64,
    settings: TranslatedViscousSettings,
    accepted: Vec<[f64; 3]>,
    candidate: Vec<[f64; 3]>,
    solution: Vec<f64>,
    rhs: Vec<f64>,
    residual: Vec<f64>,
    direction: Vec<f64>,
    product: Vec<f64>,
    preconditioned: Vec<f64>,
    diagonal_mass: Vec<f64>,
    diagonal_stiffness: Vec<f64>,
    nodal: Vec<f64>,
    force: Vec<f64>,
    free_nodes: Vec<usize>,
    time: f64,
    offset: f64,
    stamp: TranslatedViscousStamp,
    allocated_bytes: usize,
}
pub(crate) struct TranslatedViscousScratch<'a> {
    pub frame: &'a FittedHeightWorkspace,
    pub accepted: &'a [[f64; 3]],
    pub candidate: &'a mut [[f64; 3]],
    pub solution: &'a mut [f64],
    pub rhs: &'a mut [f64],
    pub residual: &'a mut [f64],
    pub direction: &'a mut [f64],
    pub product: &'a mut [f64],
    pub preconditioned: &'a mut [f64],
    pub nodal: &'a mut [f64],
    pub force: &'a mut [f64],
    pub free_nodes: &'a [usize],
}
struct Budget {
    used: usize,
    limit: usize,
}
impl Budget {
    fn vector<T>(&mut self, n: usize) -> Result<Vec<T>, TranslatedViscousError> {
        let bytes = n
            .checked_mul(std::mem::size_of::<T>())
            .and_then(|b| self.used.checked_add(b))
            .ok_or(TranslatedViscousError::AllocationFailed)?;
        if bytes > self.limit {
            return Err(TranslatedViscousError::BufferLimit {
                required: bytes,
                limit: self.limit,
            });
        }
        let mut v = Vec::new();
        v.try_reserve_exact(n)
            .map_err(|_| TranslatedViscousError::AllocationFailed)?;
        self.used = self
            .used
            .checked_add(
                v.capacity()
                    .checked_mul(std::mem::size_of::<T>())
                    .ok_or(TranslatedViscousError::AllocationFailed)?,
            )
            .ok_or(TranslatedViscousError::AllocationFailed)?;
        if self.used > self.limit {
            return Err(TranslatedViscousError::BufferLimit {
                required: self.used,
                limit: self.limit,
            });
        }
        Ok(v)
    }
    fn scalar(&mut self, n: usize) -> Result<Vec<f64>, TranslatedViscousError> {
        let mut v = self.vector(n)?;
        v.resize(n, 0.0);
        Ok(v)
    }
}
impl TranslatedViscousFlow {
    pub fn nominal_bytes(columns: usize) -> Result<usize, TranslatedViscousError> {
        FittedHeightPlan::new(columns)?
            .nominal_bytes
            .checked_add(
                columns
                    .checked_mul(944)
                    .ok_or(TranslatedViscousError::AllocationFailed)?,
            )
            .ok_or(TranslatedViscousError::AllocationFailed)
    }
    pub fn new(
        geometry: FittedHeightGeometry<'_>,
        initial_velocity: &[[f64; 3]],
        mut fitted: FittedHeightSettings,
        settings: TranslatedViscousSettings,
        id: u64,
    ) -> Result<Self, TranslatedViscousError> {
        let c = geometry
            .cap
            .len()
            .checked_sub(1)
            .ok_or(TranslatedViscousError::ShapeMismatch)?;
        let plan = FittedHeightPlan::new(c)?;
        if initial_velocity.len() != plan.periodic_nodes {
            return Err(TranslatedViscousError::ShapeMismatch);
        }
        if settings.max_iterations == 0 {
            return Err(TranslatedViscousError::InvalidSettings);
        }
        for x in [
            settings.relative_residual,
            settings.absolute_residual,
            settings.momentum_limit,
            settings.divergence_limit,
        ] {
            if !x.is_normal() || x <= 0.0 {
                return Err(TranslatedViscousError::InvalidSettings);
            }
        }
        let required = Self::nominal_bytes(c)?;
        if required > settings.memory_limit {
            return Err(TranslatedViscousError::BufferLimit {
                required,
                limit: settings.memory_limit,
            });
        }
        let speed = check(initial_velocity[0][0])?;
        for v in initial_velocity {
            for &x in v {
                check(x)?;
            }
            if v[0].to_bits() != speed.to_bits() || v[1] != 0.0 {
                return Err(TranslatedViscousError::UnsupportedVelocity);
            }
        }
        let extra = required - plan.nominal_bytes;
        fitted.memory_limit = fitted.memory_limit.min(settings.memory_limit - extra);
        let density = geometry.density;
        let frame = FittedHeightWorkspace::new(geometry, fitted, |_| false)?;
        for &m in frame.nodal_mass() {
            div(m, density)?;
        }
        let mut budget = Budget {
            used: frame.allocated_bytes(),
            limit: settings.memory_limit,
        };
        let n = plan.periodic_nodes;
        let nr = 6 * c;
        let mut accepted = budget.vector(n)?;
        accepted.extend_from_slice(initial_velocity);
        let mut candidate = budget.vector(n)?;
        candidate.resize(n, [0.0; 3]);
        let solution = budget.scalar(nr)?;
        let rhs = budget.scalar(nr)?;
        let residual = budget.scalar(nr)?;
        let direction = budget.scalar(nr)?;
        let product = budget.scalar(nr)?;
        let preconditioned = budget.scalar(nr)?;
        let diagonal_mass = budget.scalar(nr)?;
        let diagonal_stiffness = budget.scalar(nr)?;
        let nodal = budget.scalar(n)?;
        let force = budget.scalar(n)?;
        let mut free_nodes = budget.vector(nr)?;
        free_nodes.resize(nr, usize::MAX);
        let mut flow = Self {
            frame,
            density,
            settings,
            accepted,
            candidate,
            solution,
            rhs,
            residual,
            direction,
            product,
            preconditioned,
            diagonal_mass,
            diagonal_stiffness,
            nodal,
            force,
            free_nodes,
            time: 0.0,
            offset: 0.0,
            stamp: TranslatedViscousStamp { id, version: 0 },
            allocated_bytes: budget.used,
        };
        for i in 0..n {
            let row = flow.frame.velocity_embedding(i, 2).unwrap();
            if row.weights[0] == 1.0
                && let Some(col) = row.columns[0]
            {
                flow.free_nodes[col - 11 * c] = i;
            }
            for (col, w) in row.columns.into_iter().zip(row.weights) {
                if let Some(col) = col {
                    let j = col - 11 * c;
                    flow.diagonal_mass[j] = add(
                        flow.diagonal_mass[j],
                        mul(flow.frame.nodal_mass()[i], mul(w, w)?)?,
                    )?;
                }
            }
        }
        for (j, &node) in flow.free_nodes.iter().enumerate() {
            if node == usize::MAX {
                return Err(TranslatedViscousError::UnsupportedVelocity);
            }
            flow.solution[j] = flow.accepted[node][2];
        }
        embed(&flow.frame, &flow.solution, &mut flow.nodal)?;
        for (i, &z) in flow.nodal.iter().enumerate() {
            if z.to_bits() != flow.accepted[i][2].to_bits()
                && !(z == 0.0 && flow.accepted[i][2] == 0.0)
            {
                return Err(TranslatedViscousError::UnsupportedVelocity);
            }
        }
        for j in 0..nr {
            flow.solution.fill(0.0);
            flow.solution[j] = 1.0;
            embed(&flow.frame, &flow.solution, &mut flow.nodal)?;
            stiffness(&flow.frame, &flow.nodal, &mut flow.force)?;
            flow.product.fill(0.0);
            for i in 0..n {
                scatter(&flow.frame, i, flow.force[i], &mut flow.product)?;
            }
            flow.diagonal_stiffness[j] = flow.product[j];
            if flow.diagonal_stiffness[j] < 0.0 || flow.diagonal_mass[j] <= 0.0 {
                return Err(TranslatedViscousError::ArithmeticFailure);
            }
        }
        quantities(&flow.frame, &flow.accepted)?;
        Ok(flow)
    }
    pub fn state(&self) -> TranslatedViscousState<'_> {
        TranslatedViscousState {
            template: &self.frame,
            velocity: &self.accepted,
            time: self.time,
            offset: self.offset,
            stamp: self.stamp,
            density: self.density,
        }
    }
    pub(crate) fn settings(&self) -> TranslatedViscousSettings {
        self.settings
    }
    pub(crate) fn ale_scratch(&mut self) -> TranslatedViscousScratch<'_> {
        TranslatedViscousScratch {
            frame: &self.frame,
            accepted: &self.accepted,
            candidate: &mut self.candidate,
            solution: &mut self.solution,
            rhs: &mut self.rhs,
            residual: &mut self.residual,
            direction: &mut self.direction,
            product: &mut self.product,
            preconditioned: &mut self.preconditioned,
            nodal: &mut self.nodal,
            force: &mut self.force,
            free_nodes: &self.free_nodes,
        }
    }
    pub(crate) fn accept_ale(
        &mut self,
        geometry: &mut FittedHeightWorkspace,
        time: f64,
        offset: f64,
        stamp: TranslatedViscousStamp,
    ) {
        std::mem::swap(&mut self.frame, geometry);
        std::mem::swap(&mut self.accepted, &mut self.candidate);
        self.time = time;
        self.offset = offset;
        self.stamp = stamp;
    }
    pub fn allocated_bytes(&self) -> usize {
        self.allocated_bytes
    }
    /// Only this translating u=(a,0,w), zero-pressure family is advanced.
    pub fn step(
        &mut self,
        dt: f64,
        mut cancel: impl FnMut(TranslatedViscousStage) -> bool,
    ) -> Result<TranslatedViscousReport, TranslatedViscousError> {
        if !dt.is_normal() || dt <= 0.0 {
            return Err(TranslatedViscousError::InvalidSettings);
        }
        let next_time = add(self.time, dt)?;
        if next_time <= self.time {
            return Err(TranslatedViscousError::TimeResolution);
        }
        let speed = self.accepted[0][0];
        let displacement = mul(speed, dt)?;
        let next_offset = add(self.offset, displacement)?;
        if displacement != 0.0 && next_offset == self.offset {
            return Err(TranslatedViscousError::CoordinateResolution);
        }
        for n in self.frame.nodes() {
            let old_x = add(n.position[0], self.offset)?;
            let new_x = add(n.position[0], next_offset)?;
            if displacement != 0.0 && new_x == old_x {
                return Err(TranslatedViscousError::CoordinateResolution);
            }
        }
        // Converting the exact template-plus-offset representation to physical
        // f64 nodes must retain every element edge, rather than collapse it at
        // a large offset. This guard uses local geometry scales, not a floor.
        for tri in self.frame.triangles() {
            for edge in 0..3 {
                let a = self.frame.nodes()[tri.nodes[edge]].position;
                let b = self.frame.nodes()[tri.nodes[(edge + 1) % 3]].position;
                let dx = check(b[0] - a[0])?;
                let dy = check(b[1] - a[1])?;
                let physical_dx = check(add(b[0], next_offset)? - add(a[0], next_offset)?)?;
                let tolerance = mul(64.0 * f64::EPSILON, add(dx.abs(), dy.abs())?)?;
                if check(physical_dx - dx)?.abs() > tolerance {
                    return Err(TranslatedViscousError::CoordinateResolution);
                }
            }
        }
        let after = TranslatedViscousStamp {
            id: self.stamp.id,
            version: self
                .stamp
                .version
                .checked_add(1)
                .ok_or(TranslatedViscousError::VersionOverflow)?,
        };
        checkpoint(&mut cancel, TranslatedViscousStage::BeforeSolve)?;
        self.rhs.fill(0.0);
        for (i, v) in self.accepted.iter().enumerate() {
            scatter(
                &self.frame,
                i,
                mul(self.frame.nodal_mass()[i], v[2])?,
                &mut self.rhs,
            )?;
        }
        for (j, &i) in self.free_nodes.iter().enumerate() {
            self.solution[j] = self.accepted[i][2];
        }
        apply(
            &self.frame,
            &mut self.nodal,
            &mut self.force,
            &self.solution,
            &mut self.product,
            dt,
        )?;
        for ((r, &b), &a) in self.residual.iter_mut().zip(&self.rhs).zip(&self.product) {
            *r = check(b - a)?;
        }
        let target = self
            .settings
            .absolute_residual
            .max(mul(self.settings.relative_residual, norm(&self.rhs)?)?);
        let mut iterations = 0;
        precondition(
            &self.residual,
            &mut self.preconditioned,
            &self.diagonal_mass,
            &self.diagonal_stiffness,
            dt,
        )?;
        self.direction.copy_from_slice(&self.preconditioned);
        let mut rz = dot(&self.residual, &self.preconditioned)?;
        while norm(&self.residual)? > target {
            if iterations == self.settings.max_iterations {
                return Err(TranslatedViscousError::IterationLimit);
            }
            checkpoint(&mut cancel, TranslatedViscousStage::Iteration)?;
            apply(
                &self.frame,
                &mut self.nodal,
                &mut self.force,
                &self.direction,
                &mut self.product,
                dt,
            )?;
            let denominator = dot(&self.direction, &self.product)?;
            if denominator <= 0.0 || rz <= 0.0 {
                return Err(TranslatedViscousError::ArithmeticFailure);
            }
            let alpha = div(rz, denominator)?;
            for j in 0..self.solution.len() {
                self.solution[j] = add(self.solution[j], mul(alpha, self.direction[j])?)?;
                self.residual[j] = add(self.residual[j], -mul(alpha, self.product[j])?)?;
            }
            iterations += 1;
            let refresh = norm(&self.residual)? <= target;
            if refresh {
                apply(
                    &self.frame,
                    &mut self.nodal,
                    &mut self.force,
                    &self.solution,
                    &mut self.product,
                    dt,
                )?;
                for ((r, &b), &a) in self.residual.iter_mut().zip(&self.rhs).zip(&self.product) {
                    *r = check(b - a)?;
                }
                if norm(&self.residual)? <= target {
                    break;
                }
            }
            precondition(
                &self.residual,
                &mut self.preconditioned,
                &self.diagonal_mass,
                &self.diagonal_stiffness,
                dt,
            )?;
            let next_rz = dot(&self.residual, &self.preconditioned)?;
            if refresh {
                self.direction.copy_from_slice(&self.preconditioned);
            } else {
                let beta = div(next_rz, rz)?;
                for (p, &z) in self.direction.iter_mut().zip(&self.preconditioned) {
                    *p = add(z, mul(beta, *p)?)?;
                }
            }
            rz = next_rz;
        }
        apply(
            &self.frame,
            &mut self.nodal,
            &mut self.force,
            &self.solution,
            &mut self.product,
            dt,
        )?;
        for ((r, &a), &b) in self.residual.iter_mut().zip(&self.product).zip(&self.rhs) {
            *r = check(a - b)?;
        }
        let true_residual = norm(&self.residual)?;
        if true_residual > target {
            return Err(TranslatedViscousError::AcceptanceFailure);
        }
        let residual_work = dot(&self.solution, &self.residual)?;
        embed(&self.frame, &self.solution, &mut self.nodal)?;
        for (v, &z) in self.candidate.iter_mut().zip(&self.nodal) {
            *v = [speed, 0.0, z];
        }
        checkpoint(&mut cancel, TranslatedViscousStage::BeforeAcceptance)?;
        let (energy_before, momentum_before) = quantities(&self.frame, &self.accepted)?;
        let (energy_after, momentum_after) = quantities(&self.frame, &self.candidate)?;
        let mut increment_energy = 0.0;
        for (i, (new, old)) in self.candidate.iter().zip(&self.accepted).enumerate() {
            for (&a, &b) in new.iter().zip(old) {
                let d = check(a - b)?;
                increment_energy = add(
                    increment_energy,
                    mul(0.5, mul(self.frame.nodal_mass()[i], mul(d, d)?)?)?,
                )?;
            }
        }
        let mut divergence_max = 0.0_f64;
        for tri in self.frame.triangles() {
            let mut grad = [[0.0; 2]; 3];
            for j in 0..3 {
                let v = self.candidate[self.frame.nodes()[tri.nodes[j]].periodic_index];
                for (d, row) in grad.iter_mut().enumerate() {
                    for (e, x) in row.iter_mut().enumerate() {
                        *x = add(*x, mul(v[d], tri.gradients[j][e])?)?;
                    }
                }
            }
            divergence_max = divergence_max.max(add(grad[0][0], grad[1][1])?.abs());
        }
        // Local differences preserve the constant nullspace without cancellation
        // of a large absolute value against itself in an assembled row sum.
        let strain_power = stiffness(&self.frame, &self.nodal, &mut self.force)?;
        let delta = check(energy_after - energy_before)?;
        let work_identity_error =
            check(add(add(delta, increment_energy)?, mul(dt, strain_power)?)? - residual_work)?;
        let allowance = mul(
            64.0 * f64::EPSILON,
            add(
                add(1.0, energy_before.abs())?,
                add(energy_after.abs(), mul(dt, strain_power.abs())?)?,
            )?,
        )?;
        if strain_power < 0.0
            || divergence_max > self.settings.divergence_limit
            || work_identity_error.abs() > allowance
            || energy_after > add(energy_before, add(residual_work, allowance)?)?
        {
            return Err(TranslatedViscousError::AcceptanceFailure);
        }
        for d in 0..3 {
            if check(momentum_after[d] - momentum_before[d])?.abs() > self.settings.momentum_limit {
                return Err(TranslatedViscousError::AcceptanceFailure);
            }
        }
        let mut liquid_mass = 0.0;
        for &m in self.frame.nodal_mass() {
            liquid_mass = add(liquid_mass, m)?;
        }
        let report = TranslatedViscousReport {
            before: self.stamp,
            after,
            time_before: self.time,
            time_after: next_time,
            offset_before: self.offset,
            offset_after: next_offset,
            liquid_mass,
            momentum_before,
            momentum_after,
            energy_before,
            energy_after,
            increment_energy,
            strain_power,
            true_residual,
            residual_work,
            work_identity_error,
            divergence_max,
            iterations,
        };
        checkpoint(&mut cancel, TranslatedViscousStage::BeforePublish)?;
        std::mem::swap(&mut self.accepted, &mut self.candidate);
        self.time = next_time;
        self.offset = next_offset;
        self.stamp = after;
        Ok(report)
    }
}
pub(crate) fn scatter(
    frame: &FittedHeightWorkspace,
    node: usize,
    value: f64,
    out: &mut [f64],
) -> Result<(), TranslatedViscousError> {
    let row = frame.velocity_embedding(node, 2).unwrap();
    let shift = 11 * frame.plan().columns;
    for (col, w) in row.columns.into_iter().zip(row.weights) {
        if let Some(col) = col {
            let j = col - shift;
            out[j] = add(out[j], mul(w, value)?)?;
        }
    }
    Ok(())
}
pub(crate) fn embed(
    frame: &FittedHeightWorkspace,
    v: &[f64],
    out: &mut [f64],
) -> Result<(), TranslatedViscousError> {
    let shift = 11 * frame.plan().columns;
    for (i, x) in out.iter_mut().enumerate() {
        *x = 0.0;
        let row = frame.velocity_embedding(i, 2).unwrap();
        for (col, w) in row.columns.into_iter().zip(row.weights) {
            if let Some(col) = col {
                *x = add(*x, mul(w, v[col - shift])?)?;
            }
        }
    }
    Ok(())
}
pub(crate) fn stiffness(
    frame: &FittedHeightWorkspace,
    input: &[f64],
    out: &mut [f64],
) -> Result<f64, TranslatedViscousError> {
    out.fill(0.0);
    let mut power = 0.0;
    for (t, tri) in frame.triangles().iter().enumerate() {
        let k = frame.triangle_stiffness(t)?;
        let ids = tri.nodes.map(|i| frame.nodes()[i].periodic_index);
        let d = [
            check(input[ids[1]] - input[ids[0]])?,
            check(input[ids[2]] - input[ids[0]])?,
        ];
        let f1 = add(mul(k[5][5], d[0])?, mul(k[5][8], d[1])?)?;
        let f2 = add(mul(k[8][5], d[0])?, mul(k[8][8], d[1])?)?;
        out[ids[0]] = add(out[ids[0]], -add(f1, f2)?)?;
        out[ids[1]] = add(out[ids[1]], f1)?;
        out[ids[2]] = add(out[ids[2]], f2)?;
        power = add(power, add(mul(d[0], f1)?, mul(d[1], f2)?)?)?;
    }
    Ok(power)
}

fn apply(
    frame: &FittedHeightWorkspace,
    nodal: &mut [f64],
    force: &mut [f64],
    v: &[f64],
    out: &mut [f64],
    dt: f64,
) -> Result<(), TranslatedViscousError> {
    embed(frame, v, nodal)?;
    stiffness(frame, nodal, force)?;
    out.fill(0.0);
    for i in 0..nodal.len() {
        scatter(
            frame,
            i,
            add(mul(frame.nodal_mass()[i], nodal[i])?, mul(dt, force[i])?)?,
            out,
        )?;
    }
    Ok(())
}
fn precondition(
    r: &[f64],
    out: &mut [f64],
    mass: &[f64],
    stiffness: &[f64],
    dt: f64,
) -> Result<(), TranslatedViscousError> {
    for (((x, &r), &m), &k) in out.iter_mut().zip(r).zip(mass).zip(stiffness) {
        let d = add(m, mul(dt, k)?)?;
        if d <= 0.0 {
            return Err(TranslatedViscousError::ArithmeticFailure);
        }
        *x = div(r, d)?;
    }
    Ok(())
}
pub(crate) fn quantities(
    frame: &FittedHeightWorkspace,
    u: &[[f64; 3]],
) -> Result<(f64, [f64; 3]), TranslatedViscousError> {
    let mut e = 0.0;
    let mut p = [0.0; 3];
    for (&m, v) in frame.nodal_mass().iter().zip(u) {
        for (d, &x) in v.iter().enumerate() {
            p[d] = add(p[d], mul(m, x)?)?;
            e = add(e, mul(0.5, mul(m, mul(x, x)?)?)?)?;
        }
    }
    Ok((e, p))
}
