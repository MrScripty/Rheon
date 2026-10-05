//! Nonzero physical relative transport for the existing u=(a,0,w), pi=0 family.
//! Bottom tangential mesh coordinates are fixed; cap follows accepted velocity.
//! This does not enable the general pressure-coupled liquid stepping API.
use crate::translated_viscous::{
    TranslatedViscousScratch, add, check, div, dot, embed, mul, norm, quantities, scatter,
    stiffness,
};
use crate::*;
use std::fmt;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixedBottomAleStage {
    BeforeGeometry,
    Quadrature,
    BeforeSolve,
    Iteration,
    BeforeAcceptance,
    BeforePublish,
}
#[derive(Debug, Clone, PartialEq)]
pub enum FixedBottomAleError {
    Flow(TranslatedViscousError),
    GeometryPathLimit,
    TopologyChanged,
    QuadratureFailure,
    GeometricConservationFailure,
    AcceptanceFailure,
    IterationLimit,
    Cancelled { stage: FixedBottomAleStage },
}
impl From<TranslatedViscousError> for FixedBottomAleError {
    fn from(e: TranslatedViscousError) -> Self {
        Self::Flow(e)
    }
}
impl From<FittedHeightError> for FixedBottomAleError {
    fn from(e: FittedHeightError) -> Self {
        Self::Flow(e.into())
    }
}
impl fmt::Display for FixedBottomAleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "fixed-bottom ALE flow rejected: {self:?}")
    }
}
impl std::error::Error for FixedBottomAleError {}
#[derive(Debug, Clone, Copy, Default)]
pub struct FixedBottomAleTransfer {
    pub nodes: [usize; 2],
    pub positive: f64,
    pub negative: f64,
    /// Independent lower quadrature order, never used to fit accepted masses.
    pub coarse_positive: f64,
    pub coarse_negative: f64,
}
#[derive(Debug, Clone, Copy)]
pub struct FixedBottomAleReport {
    pub before: TranslatedViscousStamp,
    pub after: TranslatedViscousStamp,
    pub dt: f64,
    pub time_after: f64,
    pub cap_offset_after: f64,
    pub mass_before: f64,
    pub mass_after: f64,
    pub momentum_before: [f64; 3],
    pub momentum_after: [f64; 3],
    pub energy_before: f64,
    pub energy_after: f64,
    pub increment_loss: f64,
    pub advection_loss: f64,
    pub strain_power: f64,
    pub true_residual: f64,
    pub residual_work: f64,
    pub gcl_work: f64,
    pub work_error: f64,
    pub gcl_max: f64,
    pub quadrature_error_max: f64,
    pub relative_transfer_l1: f64,
    pub divergence_max: f64,
    pub iterations: usize,
}
/// The sole accepted geometry is the current fitted frame, with fixed bottom
/// coordinates and the cap at the accepted offset. No legacy phase/MAC owner.
pub struct FixedBottomAleState<'a> {
    pub geometry: &'a FittedHeightWorkspace,
    pub velocity: &'a [[f64; 3]],
    pub density: f64,
    pub time: f64,
    pub cap_offset: f64,
    pub stamp: TranslatedViscousStamp,
}
impl FixedBottomAleState<'_> {
    pub fn physical_node(&self, i: usize) -> Option<[f64; 2]> {
        self.geometry.nodes().get(i).map(|n| n.position)
    }
    pub fn nodal_liquid_volume(&self, i: usize) -> Option<f64> {
        self.geometry.nodal_mass().get(i).map(|m| m / self.density)
    }
    pub fn relative_pressure(&self) -> f64 {
        0.0
    }
}
struct Work {
    geometry: FittedHeightWorkspace,
    cap: Vec<[f64; 2]>,
    bottom: Vec<f64>,
    velocity: Vec<[f64; 3]>,
    diagnostic: Vec<FittedHeightNodeDiagnostic>,
    pressure: Vec<f64>,
    faces: Vec<FixedBottomAleTransfer>,
    gcl: Vec<f64>,
    basis: Vec<f64>,
    hessenberg: Vec<f64>,
    cosine: Vec<f64>,
    sine: Vec<f64>,
    least_squares: Vec<f64>,
    coefficients: Vec<f64>,
}
/// Reuses the preceding full accepted-state owner and its candidate/solver
/// vectors. One additional frame is strictly candidate scratch. No step allocates
/// heap storage, snapshots accepted state or materializes a dense transfer map.
pub struct FixedBottomAleFlow {
    flow: TranslatedViscousFlow,
    work: Work,
    allocated_bytes: usize,
}
struct Budget {
    used: usize,
    limit: usize,
}
impl Budget {
    fn vector<T: Default + Clone>(&mut self, n: usize) -> Result<Vec<T>, FixedBottomAleError> {
        let requested = n
            .checked_mul(std::mem::size_of::<T>())
            .and_then(|x| self.used.checked_add(x))
            .ok_or(TranslatedViscousError::AllocationFailed)?;
        if requested > self.limit {
            return Err(TranslatedViscousError::BufferLimit {
                required: requested,
                limit: self.limit,
            }
            .into());
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
            }
            .into());
        }
        v.resize(n, T::default());
        Ok(v)
    }
}
impl FixedBottomAleFlow {
    pub fn nominal_bytes(columns: usize) -> Result<usize, FixedBottomAleError> {
        let plan = FittedHeightPlan::new(columns)?;
        let n = plan.periodic_nodes;
        let nr = columns
            .checked_mul(6)
            .ok_or(TranslatedViscousError::AllocationFailed)?;
        let cap = columns
            .checked_add(1)
            .ok_or(TranslatedViscousError::AllocationFailed)?;
        let krylov = nr
            .checked_add(1)
            .and_then(|x| x.checked_mul(nr))
            .ok_or(TranslatedViscousError::AllocationFailed)?;
        let mut total = TranslatedViscousFlow::nominal_bytes(columns)?
            .checked_add(plan.nominal_bytes)
            .ok_or(TranslatedViscousError::AllocationFailed)?;
        for (count, size) in [
            (cap, std::mem::size_of::<[f64; 2]>()),
            (cap, std::mem::size_of::<f64>()),
            (n, std::mem::size_of::<[f64; 3]>()),
            (n, std::mem::size_of::<FittedHeightNodeDiagnostic>()),
            (n, std::mem::size_of::<f64>()),
            (36 * columns, std::mem::size_of::<FixedBottomAleTransfer>()),
            (n, std::mem::size_of::<f64>()),
            (krylov, std::mem::size_of::<f64>()),
            (krylov, std::mem::size_of::<f64>()),
            (nr, std::mem::size_of::<f64>()),
            (nr, std::mem::size_of::<f64>()),
            (nr + 1, std::mem::size_of::<f64>()),
            (nr, std::mem::size_of::<f64>()),
        ] {
            total = total
                .checked_add(
                    count
                        .checked_mul(size)
                        .ok_or(TranslatedViscousError::AllocationFailed)?,
                )
                .ok_or(TranslatedViscousError::AllocationFailed)?;
        }
        Ok(total)
    }

    pub fn new(
        geometry: FittedHeightGeometry<'_>,
        velocity: &[[f64; 3]],
        fitted: FittedHeightSettings,
        settings: TranslatedViscousSettings,
        id: u64,
    ) -> Result<Self, FixedBottomAleError> {
        let columns = geometry
            .cap
            .len()
            .checked_sub(1)
            .ok_or(TranslatedViscousError::ShapeMismatch)?;
        let required = Self::nominal_bytes(columns)?;
        if required > settings.memory_limit {
            return Err(TranslatedViscousError::BufferLimit {
                required,
                limit: settings.memory_limit,
            }
            .into());
        }
        if velocity.first().is_some_and(|v| v[0] < 0.) {
            return Err(TranslatedViscousError::UnsupportedVelocity.into());
        }
        let flow = TranslatedViscousFlow::new(geometry, velocity, fitted, settings, id)?;
        let candidate = FittedHeightWorkspace::new(geometry, fitted, |_| false)?;
        let mut b = Budget {
            used: flow
                .allocated_bytes()
                .checked_add(candidate.allocated_bytes())
                .ok_or(TranslatedViscousError::AllocationFailed)?,
            limit: settings.memory_limit,
        };
        let n = 8 * columns;
        let nr = 6 * columns;
        let work = Work {
            geometry: candidate,
            cap: b.vector(columns + 1)?,
            bottom: b.vector(columns + 1)?,
            velocity: b.vector(n)?,
            diagnostic: b.vector(n)?,
            pressure: b.vector(n)?,
            faces: b.vector(36 * columns)?,
            gcl: b.vector(n)?,
            basis: b.vector((nr + 1) * nr)?,
            hessenberg: b.vector((nr + 1) * nr)?,
            cosine: b.vector(nr)?,
            sine: b.vector(nr)?,
            least_squares: b.vector(nr + 1)?,
            coefficients: b.vector(nr)?,
        };
        let mut owner = Self {
            flow,
            work,
            allocated_bytes: b.used,
        };
        let state = owner.flow.state();
        for (i, f) in state.template.shared_flux_scratch().iter().enumerate() {
            owner.work.faces[i].nodes = f.nodes;
        }
        // Retain reserved capacity but expose only actual topology rows.
        owner
            .work
            .faces
            .truncate(state.template.shared_flux_scratch().len());
        owner.work.bottom.copy_from_slice(geometry.bottom_x);
        Ok(owner)
    }
    pub fn state(&self) -> FixedBottomAleState<'_> {
        let s = self.flow.state();
        FixedBottomAleState {
            geometry: s.template,
            velocity: s.velocity,
            density: s.density,
            time: s.time,
            cap_offset: s.offset,
            stamp: s.stamp,
        }
    }
    pub fn allocated_bytes(&self) -> usize {
        self.allocated_bytes
    }
    /// Last working quadrature transfers; may be incomplete after rejection.
    /// They are diagnostics, not a separately publishable accepted state.
    pub fn transfer_scratch(&self) -> &[FixedBottomAleTransfer] {
        &self.work.faces
    }
    pub fn step(
        &mut self,
        dt: f64,
        mut cancel: impl FnMut(FixedBottomAleStage) -> bool,
    ) -> Result<FixedBottomAleReport, FixedBottomAleError> {
        if !dt.is_normal() || dt <= 0. {
            return Err(TranslatedViscousError::InvalidSettings.into());
        }
        let state = self.flow.state();
        let density = state.density;
        let old_time = state.time;
        let old_offset = state.offset;
        let before = state.stamp;
        let speed = state.velocity[0][0];
        let time = add(old_time, dt)?;
        if time <= old_time {
            return Err(TranslatedViscousError::TimeResolution.into());
        }
        let displacement = mul(speed, dt)?;
        let offset = add(old_offset, displacement)?;
        if displacement != 0. && offset == old_offset {
            return Err(TranslatedViscousError::CoordinateResolution.into());
        }
        let period = check(self.work.bottom[self.work.bottom.len() - 1] - self.work.bottom[0])?;
        if offset < 0. || offset > mul(period, 0.125)? {
            return Err(FixedBottomAleError::GeometryPathLimit);
        }
        let after = TranslatedViscousStamp {
            id: before.id,
            version: before
                .version
                .checked_add(1)
                .ok_or(TranslatedViscousError::VersionOverflow)?,
        };
        let settings = self.flow.settings();
        checkpoint(&mut cancel, FixedBottomAleStage::BeforeGeometry)?;
        let mut s = self.flow.ale_scratch();
        let w = &mut self.work;
        for f in &mut w.faces {
            f.positive = 0.;
            f.negative = 0.;
            f.coarse_positive = 0.;
            f.coarse_negative = 0.;
        }
        w.velocity.fill([speed, 0., 0.]);
        // Independent physical quadrature orders. Positive and negative face
        // transfers are integrated separately, even if a sampled face reverses.
        integrate(&s, w, dt, displacement, &GAUSS8, true, &mut cancel)?;
        integrate(&s, w, dt, displacement, &GAUSS16, false, &mut cancel)?;
        let mut quadrature_error_max = 0.0_f64;
        let mut relative_transfer_l1 = 0.;
        for f in &w.faces {
            let error = check(f.positive - f.coarse_positive)?
                .abs()
                .max(check(f.negative - f.coarse_negative)?.abs());
            quadrature_error_max = quadrature_error_max.max(error);
            let scale = add(
                s.frame.nodal_mass()[f.nodes[0]],
                s.frame.nodal_mass()[f.nodes[1]],
            )?;
            if error > mul(64. * f64::EPSILON, scale)? {
                return Err(FixedBottomAleError::QuadratureFailure);
            }
            relative_transfer_l1 = add(relative_transfer_l1, add(f.positive, f.negative)?)?;
        }
        reconstruct(s.frame, w, displacement)?;
        for i in 0..s.accepted.len() {
            let old = s.frame.velocity_embedding(i, 2).unwrap();
            let new = w.geometry.velocity_embedding(i, 2).unwrap();
            if old.columns != new.columns
                || old.weights.map(f64::to_bits) != new.weights.map(f64::to_bits)
            {
                return Err(FixedBottomAleError::TopologyChanged);
            }
        }
        let mut mass_before = 0.;
        let mut mass_after = 0.;
        for (i, g) in w.gcl.iter_mut().enumerate() {
            let old = s.frame.nodal_mass()[i];
            let new = w.geometry.nodal_mass()[i];
            mass_before = add(mass_before, old)?;
            mass_after = add(mass_after, new)?;
            *g = check(new - old)?;
        }
        for f in &w.faces {
            let net = check(f.positive - f.negative)?;
            w.gcl[f.nodes[0]] = add(w.gcl[f.nodes[0]], net)?;
            w.gcl[f.nodes[1]] = add(w.gcl[f.nodes[1]], -net)?;
        }
        let mut gcl_max = 0.0_f64;
        for (i, &g) in w.gcl.iter().enumerate() {
            gcl_max = gcl_max.max(g.abs());
            let scale = add(s.frame.nodal_mass()[i], w.geometry.nodal_mass()[i])?;
            if g.abs() > mul(128. * f64::EPSILON, scale)? {
                return Err(FixedBottomAleError::GeometricConservationFailure);
            }
        }
        if check(mass_after - mass_before)?.abs()
            > mul(128. * f64::EPSILON, add(mass_before, mass_after)?)?
        {
            return Err(FixedBottomAleError::GeometricConservationFailure);
        }
        s.rhs.fill(0.);
        for (i, v) in s.accepted.iter().enumerate() {
            scatter(&w.geometry, i, mul(s.frame.nodal_mass()[i], v[2])?, s.rhs)?;
        }
        for (j, &i) in s.free_nodes.iter().enumerate() {
            s.solution[j] = s.accepted[i][2];
        }
        let target = settings
            .absolute_residual
            .max(mul(settings.relative_residual, norm(s.rhs)?)?);
        checkpoint(&mut cancel, FixedBottomAleStage::BeforeSolve)?;
        let iterations = gmres(&mut s, w, dt, target, settings.max_iterations, &mut cancel)?;
        apply(
            &w.geometry,
            &w.faces,
            s.nodal,
            s.force,
            s.solution,
            s.product,
            dt,
        )?;
        for ((r, &a), &b) in s
            .residual
            .iter_mut()
            .zip(s.product.iter())
            .zip(s.rhs.iter())
        {
            *r = check(a - b)?;
        }
        let rz = norm(s.residual)?;
        let mut residual_work = dot(s.solution, s.residual)?;
        s.direction.fill(0.);
        for (i, &g) in w.gcl.iter().enumerate() {
            scatter(&w.geometry, i, mul(speed, g)?, s.direction)?;
        }
        let rx = norm(s.direction)?;
        for &r in s.direction.iter() {
            residual_work = add(residual_work, mul(speed, r)?)?;
        }
        let true_residual = check(add(mul(rz, rz)?, mul(rx, rx)?)?.sqrt())?;
        if true_residual > target {
            return Err(FixedBottomAleError::AcceptanceFailure);
        }
        embed(&w.geometry, s.solution, s.nodal)?;
        for (v, &z) in s.candidate.iter_mut().zip(s.nodal.iter()) {
            *v = [speed, 0., z];
        }
        checkpoint(&mut cancel, FixedBottomAleStage::BeforeAcceptance)?;
        let (energy_before, momentum_before) = quantities(s.frame, s.accepted)?;
        let (energy_after, momentum_after) = quantities(&w.geometry, s.candidate)?;
        let mut increment_loss = 0.;
        let mut gcl_work = 0.;
        for (i, (v, u)) in s.candidate.iter().zip(s.accepted).enumerate() {
            let mut v2 = 0.;
            for (&a, &b) in v.iter().zip(u) {
                let delta = check(a - b)?;
                increment_loss = add(
                    increment_loss,
                    mul(0.5, mul(s.frame.nodal_mass()[i], mul(delta, delta)?)?)?,
                )?;
                v2 = add(v2, mul(a, a)?)?;
            }
            gcl_work = add(gcl_work, mul(0.5, mul(w.gcl[i], v2)?)?)?;
        }
        let mut advection_loss = 0.;
        for f in &w.faces {
            for d in 0..3 {
                let delta = check(s.candidate[f.nodes[0]][d] - s.candidate[f.nodes[1]][d])?;
                advection_loss = add(
                    advection_loss,
                    mul(0.5, mul(add(f.positive, f.negative)?, mul(delta, delta)?)?)?,
                )?;
            }
        }
        let strain_power = stiffness(&w.geometry, s.nodal, s.force)?;
        let work_error = add(
            check(
                add(
                    add(check(energy_after - energy_before)?, increment_loss)?,
                    add(advection_loss, mul(dt, strain_power)?)?,
                )? - residual_work,
            )?,
            gcl_work,
        )?;
        let scale = add(
            add(energy_before.abs(), energy_after.abs())?,
            add(
                add(increment_loss, advection_loss)?,
                add(
                    mul(dt, strain_power.abs())?,
                    add(residual_work.abs(), gcl_work.abs())?,
                )?,
            )?,
        )?;
        let allowance = mul(128. * f64::EPSILON, scale)?;
        let mut divergence_max = 0.0_f64;
        for t in w.geometry.triangles() {
            let mut divergence = 0.;
            for j in 0..3 {
                let v = s.candidate[w.geometry.nodes()[t.nodes[j]].periodic_index];
                divergence = add(
                    divergence,
                    add(mul(v[0], t.gradients[j][0])?, mul(v[1], t.gradients[j][1])?)?,
                )?;
            }
            divergence_max = divergence_max.max(divergence.abs());
        }
        if work_error.abs() > allowance
            || strain_power < 0.
            || divergence_max > settings.divergence_limit
            || energy_after
                > add(
                    energy_before,
                    add(check(residual_work - gcl_work)?, allowance)?,
                )?
        {
            return Err(FixedBottomAleError::AcceptanceFailure);
        }
        for d in 0..3 {
            if check(momentum_after[d] - momentum_before[d])?.abs() > settings.momentum_limit {
                return Err(FixedBottomAleError::AcceptanceFailure);
            }
        }
        for &m in w.geometry.nodal_mass() {
            div(m, density)?;
        }
        let report = FixedBottomAleReport {
            before,
            after,
            dt,
            time_after: time,
            cap_offset_after: offset,
            mass_before,
            mass_after,
            momentum_before,
            momentum_after,
            energy_before,
            energy_after,
            increment_loss,
            advection_loss,
            strain_power,
            true_residual,
            residual_work,
            gcl_work,
            work_error,
            gcl_max,
            quadrature_error_max,
            relative_transfer_l1,
            divergence_max,
            iterations,
        };
        checkpoint(&mut cancel, FixedBottomAleStage::BeforePublish)?;
        self.flow.accept_ale(&mut w.geometry, time, offset, after);
        Ok(report)
    }
}
fn checkpoint(
    cancel: &mut impl FnMut(FixedBottomAleStage) -> bool,
    stage: FixedBottomAleStage,
) -> Result<(), FixedBottomAleError> {
    if cancel(stage) {
        Err(FixedBottomAleError::Cancelled { stage })
    } else {
        Ok(())
    }
}
fn reconstruct(
    old: &FittedHeightWorkspace,
    w: &mut Work,
    displacement: f64,
) -> Result<(), FixedBottomAleError> {
    let c = old.plan().columns;
    for i in 0..=c {
        let p = old.nodes()[c + 1 + i].position;
        let x = add(p[0], displacement)?;
        if displacement != 0. && x == p[0] {
            return Err(TranslatedViscousError::CoordinateResolution.into());
        }
        w.cap[i] = [x, p[1]];
    }
    let period = check(w.bottom[c] - w.bottom[0])?;
    w.cap[c][0] = add(w.cap[0][0], period)?;
    w.geometry.reassemble(&w.cap, &w.bottom)?;
    Ok(())
}
fn integrate(
    s: &TranslatedViscousScratch<'_>,
    w: &mut Work,
    dt: f64,
    displacement: f64,
    rule: &[(f64, f64)],
    coarse: bool,
    cancel: &mut impl FnMut(FixedBottomAleStage) -> bool,
) -> Result<(), FixedBottomAleError> {
    for &(x, weight) in rule {
        checkpoint(cancel, FixedBottomAleStage::Quadrature)?;
        let theta = mul(0.5, add(1., x)?)?;
        reconstruct(s.frame, w, mul(displacement, theta)?)?;
        w.geometry.inspect(
            FittedHeightInputs {
                velocity: &w.velocity,
                pressure_coefficients: &w.pressure,
            },
            &mut w.diagnostic,
            |_| false,
        )?;
        let scale = mul(mul(dt, 0.5)?, weight)?;
        if w.geometry.shared_flux_scratch().len() != w.faces.len() {
            return Err(FixedBottomAleError::TopologyChanged);
        }
        for (f, actual) in w.faces.iter_mut().zip(w.geometry.shared_flux_scratch()) {
            if f.nodes != actual.nodes {
                return Err(FixedBottomAleError::TopologyChanged);
            }
            let pos = mul(scale, actual.flux.max(0.))?;
            let neg = mul(scale, (-actual.flux).max(0.))?;
            if coarse {
                f.coarse_positive = add(f.coarse_positive, pos)?;
                f.coarse_negative = add(f.coarse_negative, neg)?;
            } else {
                f.positive = add(f.positive, pos)?;
                f.negative = add(f.negative, neg)?;
            }
        }
    }
    Ok(())
}
fn apply(
    frame: &FittedHeightWorkspace,
    faces: &[FixedBottomAleTransfer],
    nodal: &mut [f64],
    force: &mut [f64],
    z: &[f64],
    out: &mut [f64],
    dt: f64,
) -> Result<(), FixedBottomAleError> {
    embed(frame, z, nodal)?;
    stiffness(frame, nodal, force)?;
    for f in force.iter_mut() {
        *f = mul(dt, *f)?;
    }
    for f in faces {
        let [i, j] = f.nodes;
        let momentum = check(mul(f.positive, nodal[i])? - mul(f.negative, nodal[j])?)?;
        force[i] = add(force[i], momentum)?;
        force[j] = add(force[j], -momentum)?;
    }
    out.fill(0.);
    for i in 0..nodal.len() {
        scatter(
            frame,
            i,
            add(mul(frame.nodal_mass()[i], nodal[i])?, force[i])?,
            out,
        )?;
    }
    Ok(())
}
fn gmres(
    s: &mut TranslatedViscousScratch<'_>,
    w: &mut Work,
    dt: f64,
    target: f64,
    max_iterations: usize,
    cancel: &mut impl FnMut(FixedBottomAleStage) -> bool,
) -> Result<usize, FixedBottomAleError> {
    let n = s.solution.len();
    apply(
        &w.geometry,
        &w.faces,
        s.nodal,
        s.force,
        s.solution,
        s.product,
        dt,
    )?;
    for ((r, &b), &a) in s
        .residual
        .iter_mut()
        .zip(s.rhs.iter())
        .zip(s.product.iter())
    {
        *r = check(b - a)?;
    }
    let beta = norm(s.residual)?;
    if beta <= target {
        return Ok(0);
    }
    s.preconditioned.copy_from_slice(s.solution);
    w.hessenberg.fill(0.);
    w.least_squares.fill(0.);
    w.least_squares[0] = beta;
    for (v, &r) in w.basis[..n].iter_mut().zip(s.residual.iter()) {
        *v = div(r, beta)?;
    }
    let mut used = 0;
    for j in 0..n.min(max_iterations) {
        checkpoint(cancel, FixedBottomAleStage::Iteration)?;
        apply(
            &w.geometry,
            &w.faces,
            s.nodal,
            s.force,
            &w.basis[j * n..(j + 1) * n],
            s.direction,
            dt,
        )?;
        // Two-pass modified Gram–Schmidt; all Krylov/Hessenberg storage is
        // preallocated and charged, with no restart history or dense map.
        for _ in 0..2 {
            for i in 0..=j {
                let vi = &w.basis[i * n..(i + 1) * n];
                let h = dot(s.direction, vi)?;
                let index = i * n + j;
                w.hessenberg[index] = add(w.hessenberg[index], h)?;
                for (a, &v) in s.direction.iter_mut().zip(vi) {
                    *a = check(*a - mul(h, v)?)?;
                }
            }
        }
        let next = norm(s.direction)?;
        w.hessenberg[(j + 1) * n + j] = next;
        if next > 0. {
            for (v, &r) in w.basis[(j + 1) * n..(j + 2) * n]
                .iter_mut()
                .zip(s.direction.iter())
            {
                *v = div(r, next)?;
            }
        }
        for i in 0..j {
            let a = w.hessenberg[i * n + j];
            let b = w.hessenberg[(i + 1) * n + j];
            w.hessenberg[i * n + j] = add(mul(w.cosine[i], a)?, mul(w.sine[i], b)?)?;
            w.hessenberg[(i + 1) * n + j] = check(mul(w.cosine[i], b)? - mul(w.sine[i], a)?)?;
        }
        let a = w.hessenberg[j * n + j];
        let b = w.hessenberg[(j + 1) * n + j];
        let radius = check(add(mul(a, a)?, mul(b, b)?)?.sqrt())?;
        w.cosine[j] = div(a, radius)?;
        w.sine[j] = div(b, radius)?;
        w.hessenberg[j * n + j] = radius;
        w.hessenberg[(j + 1) * n + j] = 0.;
        w.least_squares[j + 1] = -mul(w.sine[j], w.least_squares[j])?;
        w.least_squares[j] = mul(w.cosine[j], w.least_squares[j])?;
        used = j + 1;
        if w.least_squares[j + 1].abs() <= target || next == 0. {
            break;
        }
    }
    for i in (0..used).rev() {
        let mut rhs = w.least_squares[i];
        for j in i + 1..used {
            rhs = check(rhs - mul(w.hessenberg[i * n + j], w.coefficients[j])?)?;
        }
        w.coefficients[i] = div(rhs, w.hessenberg[i * n + i])?;
    }
    s.solution.copy_from_slice(s.preconditioned);
    for j in 0..used {
        for (x, &v) in s.solution.iter_mut().zip(&w.basis[j * n..(j + 1) * n]) {
            *x = add(*x, mul(v, w.coefficients[j])?)?;
        }
    }
    if used == max_iterations && w.least_squares[used].abs() > target {
        return Err(FixedBottomAleError::IterationLimit);
    }
    Ok(used)
}
const GAUSS8: [(f64, f64); 8] = [
    (-0.9602898564975363, 0.1012285362903763),
    (-0.7966664774136267, 0.2223810344533745),
    (-0.525532409916329, 0.3137066458778873),
    (-0.1834346424956498, 0.362683783378362),
    (0.1834346424956498, 0.362683783378362),
    (0.525532409916329, 0.3137066458778873),
    (0.7966664774136267, 0.2223810344533745),
    (0.9602898564975363, 0.1012285362903763),
];
const GAUSS16: [(f64, f64); 16] = [
    (-0.9894009349916499, 0.0271524594117541),
    (-0.9445750230732326, 0.0622535239386479),
    (-0.8656312023878318, 0.0951585116824928),
    (-0.755404408355003, 0.1246289712555339),
    (-0.6178762444026438, 0.1495959888165767),
    (-0.4580167776572274, 0.1691565193950025),
    (-0.2816035507792589, 0.1826034150449236),
    (-0.0950125098376374, 0.1894506104550685),
    (0.0950125098376374, 0.1894506104550685),
    (0.2816035507792589, 0.1826034150449236),
    (0.4580167776572274, 0.1691565193950025),
    (0.6178762444026438, 0.1495959888165767),
    (0.755404408355003, 0.1246289712555339),
    (0.8656312023878318, 0.0951585116824928),
    (0.9445750230732326, 0.0622535239386479),
    (0.9894009349916499, 0.0271524594117541),
];
