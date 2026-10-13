//! Reduced Newtonian shear coupled to prescribed or finite-mass sliding plates.
//! Tangentially periodic/invariant geometry; no normal motion, pressure or torque.
#[path = "outward.rs"]
mod outward;
pub use outward::Interval;
use rheon::{Axis, StaticObstacleShear};
use std::{fmt, mem::size_of};

/// Mass is for the represented periodic cross-section, in kg.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Plate {
    Prescribed,
    Dynamic { mass_kg: f64 },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Input,
    Assembly,
    Solve,
    Acceptance,
    Publication,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    InvalidParameter,
    ShapeMismatch,
    CapacityOverflow,
    AllocationFailure,
    BufferLimit { required: usize, limit: usize },
    Arithmetic,
    Enclosure(outward::Error),
    Gate { stage: Stage, index: usize },
    Cancelled { stage: Stage, index: usize },
}
impl From<outward::Error> for Error {
    fn from(e: outward::Error) -> Self {
        Self::Enclosure(e)
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "moving-plate step refused: {self:?}")
    }
}
impl std::error::Error for Error {}

/// Intervals concern the actual stored coefficients and accepted velocities.
/// No continuum-error bound or floating-point environment proof is implied.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Report {
    pub dt_s: f64,
    pub tangent: Axis,
    pub fluid_impulse_n_s: [Interval; 2],
    pub kinetic_before_j: Interval,
    pub kinetic_after_j: Interval,
    pub kinetic_change_j: Interval,
    pub increment_energy_j: Interval,
    pub dissipation_j: Interval,
    pub prescribed_work_j: Interval,
    pub energy_defect_j: Interval,
    pub energy_budget_j: Interval,
    pub momentum_change_n_s: Interval,
    pub prescribed_impulse_n_s: Interval,
    pub momentum_defect_n_s: Interval,
    pub momentum_budget_n_s: Interval,
    pub equation_defect_max_n_s: f64,
}

const RELATIVE_LIMIT: f64 = 8192.0 * f64::EPSILON;
fn scalar(x: f64) -> Result<f64, Error> {
    if x == 0.0 || x.is_normal() {
        Ok(x)
    } else {
        Err(Error::Arithmetic)
    }
}
fn product(a: f64, b: f64) -> Result<f64, Error> {
    let x = scalar(a * b)?;
    if a != 0.0 && b != 0.0 && x == 0.0 {
        Err(Error::Arithmetic)
    } else {
        Ok(x)
    }
}
fn quotient(a: f64, b: f64) -> Result<f64, Error> {
    if b <= 0.0 || !b.is_normal() {
        return Err(Error::Arithmetic);
    }
    let x = scalar(a / b)?;
    if a != 0.0 && x == 0.0 {
        Err(Error::Arithmetic)
    } else {
        Ok(x)
    }
}
fn point(x: f64) -> Result<Interval, Error> {
    Ok(Interval::point(x)?)
}
fn check(
    cancel: &mut impl FnMut(Stage, usize) -> bool,
    stage: Stage,
    index: usize,
) -> Result<(), Error> {
    if cancel(stage, index) {
        Err(Error::Cancelled { stage, index })
    } else {
        Ok(())
    }
}
fn buffer(n: usize, used: &mut usize, limit: usize) -> Result<Vec<f64>, Error> {
    let planned = n
        .checked_mul(size_of::<f64>())
        .and_then(|x| x.checked_add(*used))
        .ok_or(Error::CapacityOverflow)?;
    if planned > limit {
        return Err(Error::BufferLimit {
            required: planned,
            limit,
        });
    }
    let mut v = Vec::new();
    v.try_reserve_exact(n)
        .map_err(|_| Error::AllocationFailure)?;
    *used = v
        .capacity()
        .checked_mul(size_of::<f64>())
        .and_then(|x| x.checked_add(*used))
        .ok_or(Error::CapacityOverflow)?;
    if *used > limit {
        return Err(Error::BufferLimit {
            required: *used,
            limit,
        });
    }
    v.resize(n, 0.0);
    Ok(v)
}
fn accept(defect: Interval, scale: Interval, index: usize) -> Result<Interval, Error> {
    let budget = point(RELATIVE_LIMIT)?.mul(scale)?;
    if defect.abs()?.hi > budget.lo {
        return Err(Error::Gate {
            stage: Stage::Acceptance,
            index,
        });
    }
    Ok(budget)
}

/// Borrows existing closure coefficients; owns solve scratch and last report.
/// Caller owns fluid/plate speeds and time. A step allocates no heap storage;
/// cancellation callbacks are caller code. The combined payload cap includes
/// borrowed shear storage, excludes geometry/caller arrays/allocator/stack/RSS.
pub struct Workspace<'op, 'g> {
    op: &'op StaticObstacleShear<'g>,
    tangent: Axis,
    plates: [Plate; 2],
    masses: Vec<f64>,
    diagonal: Vec<f64>,
    rhs: Vec<f64>,
    candidate: Vec<f64>,
    combined_bytes: usize,
    last: Option<Report>,
}
impl<'op, 'g> Workspace<'op, 'g> {
    pub fn new(
        op: &'op StaticObstacleShear<'g>,
        tangent: Axis,
        plates: [Plate; 2],
        limit: usize,
    ) -> Result<Self, Error> {
        outward::ensure_supported()?;
        if tangent == op.normal() {
            return Err(Error::InvalidParameter);
        }
        for plate in plates {
            if let Plate::Dynamic { mass_kg } = plate
                && (!mass_kg.is_normal() || mass_kg <= 0.0)
            {
                return Err(Error::InvalidParameter);
            }
        }
        let n = op
            .layer_volumes()
            .len()
            .checked_add(2)
            .ok_or(Error::CapacityOverflow)?;
        let mut used = op.allocated_bytes();
        let mut masses = buffer(n, &mut used, limit)?;
        let diagonal = buffer(n, &mut used, limit)?;
        let rhs = buffer(n, &mut used, limit)?;
        let candidate = buffer(n, &mut used, limit)?;
        for (i, v) in op.layer_volumes().iter().enumerate() {
            masses[i + 1] = product(op.density(), *v)?;
        }
        for (end, i) in [0, n - 1].into_iter().enumerate() {
            if let Plate::Dynamic { mass_kg } = plates[end] {
                masses[i] = mass_kg;
            }
        }
        Ok(Self {
            op,
            tangent,
            plates,
            masses,
            diagonal,
            rhs,
            candidate,
            combined_bytes: used,
            last: None,
        })
    }
    pub fn combined_bytes(&self) -> usize {
        self.combined_bytes
    }
    pub fn last_report(&self) -> Option<&Report> {
        self.last.as_ref()
    }
    pub fn operator(&self) -> &'op StaticObstacleShear<'g> {
        self.op
    }
    pub fn plates(&self) -> [Plate; 2] {
        self.plates
    }
    fn conductance(&self, edge: usize) -> f64 {
        let n = self.masses.len();
        if edge == 0 {
            self.op.wall_conductances()[0]
        } else if edge == n - 2 {
            self.op.wall_conductances()[1]
        } else {
            self.op.interior_conductances()[edge - 1]
        }
    }
    fn prescribed(&self, node: usize) -> bool {
        let n = self.masses.len();
        (node == 0 && self.plates[0] == Plate::Prescribed)
            || (node == n - 1 && self.plates[1] == Plate::Prescribed)
    }
    fn old(&self, node: usize, fluid: &[f64], walls: &[f64; 2]) -> f64 {
        if node == 0 {
            walls[0]
        } else if node == self.masses.len() - 1 {
            walls[1]
        } else {
            fluid[node - 1]
        }
    }
    // Eliminated prescribed endpoints have no off-diagonal entry.
    fn off(&self, edge: usize, dt: f64) -> Result<f64, Error> {
        if self.prescribed(edge) || self.prescribed(edge + 1) {
            Ok(0.0)
        } else {
            Ok(-product(dt, self.conductance(edge))?)
        }
    }
    /// No hidden substeps; every failure preserves both caller arrays and last report.
    pub fn step(
        &mut self,
        fluid: &mut [f64],
        walls: &mut [f64; 2],
        dt: f64,
        mut cancel: impl FnMut(Stage, usize) -> bool,
    ) -> Result<Report, Error> {
        outward::ensure_supported()?;
        let n = self.masses.len();
        if fluid.len() != n - 2 {
            return Err(Error::ShapeMismatch);
        }
        if !dt.is_normal() || dt <= 0.0 {
            return Err(Error::InvalidParameter);
        }
        for i in 0..n {
            check(&mut cancel, Stage::Input, i)?;
            scalar(self.old(i, fluid, walls))?;
        }
        for i in 0..n {
            check(&mut cancel, Stage::Assembly, i)?;
            let old = self.old(i, fluid, walls);
            if self.prescribed(i) {
                self.diagonal[i] = 1.0;
                self.rhs[i] = old;
                continue;
            }
            let mut diag = self.masses[i];
            let mut rhs = product(diag, old)?;
            for (edge, other) in [
                (i.checked_sub(1), i.checked_sub(1)),
                (
                    if i + 1 < n { Some(i) } else { None },
                    if i + 1 < n { Some(i + 1) } else { None },
                ),
            ] {
                if let (Some(edge), Some(other)) = (edge, other) {
                    let c = product(dt, self.conductance(edge))?;
                    diag = scalar(diag + c)?;
                    if self.prescribed(other) {
                        rhs = scalar(rhs + product(c, self.old(other, fluid, walls))?)?;
                    }
                }
            }
            self.diagonal[i] = diag;
            self.rhs[i] = rhs;
        }
        for i in 1..n {
            check(&mut cancel, Stage::Solve, i)?;
            let off = self.off(i - 1, dt)?;
            let ratio = quotient(off, self.diagonal[i - 1])?;
            self.diagonal[i] = scalar(self.diagonal[i] - product(ratio, off)?)?;
            self.rhs[i] = scalar(self.rhs[i] - product(ratio, self.rhs[i - 1])?)?;
        }
        for i in (0..n).rev() {
            check(&mut cancel, Stage::Solve, n + i)?;
            let rhs = if i + 1 < n {
                scalar(self.rhs[i] - product(self.off(i, dt)?, self.candidate[i + 1])?)?
            } else {
                self.rhs[i]
            };
            self.candidate[i] = quotient(rhs, self.diagonal[i])?;
        }
        // Each connected chain component at common speed is an exact zero-
        // relative-strain identity. Zero Navier conductance may isolate a plate.
        // Preserve that nullspace without replacing any nonuniform proposal.
        let mut start = 0;
        for end in 0..n {
            if end + 1 == n || self.conductance(end) == 0.0 {
                let speed = self.old(start, fluid, walls);
                if (start..=end).all(|i| self.old(i, fluid, walls) == speed) {
                    self.candidate[start..=end].fill(speed);
                }
                start = end + 1;
            }
        }
        let zero = point(0.0)?;
        let half = point(0.5)?;
        let dti = point(dt)?;
        let mut before = zero;
        let mut after = zero;
        let mut change = zero;
        let mut increment = zero;
        let mut momentum = zero;
        let mut momentum_scale = zero;
        let mut loss = zero;
        let mut work = zero;
        let mut external = zero;
        let mut impulses = [zero; 2];
        let mut equation_max = 0.0_f64;
        for i in 0..n {
            check(&mut cancel, Stage::Acceptance, i)?;
            let v = point(self.candidate[i])?;
            if !self.prescribed(i) {
                let u = point(self.old(i, fluid, walls))?;
                let m = point(self.masses[i])?;
                let diff = v.sub(u)?;
                let inertia = m.mul(diff)?;
                let mut defect = inertia;
                let mut scale = inertia.abs()?;
                for (edge, other) in [
                    (i.checked_sub(1), i.checked_sub(1)),
                    (
                        if i + 1 < n { Some(i) } else { None },
                        if i + 1 < n { Some(i + 1) } else { None },
                    ),
                ] {
                    if let (Some(edge), Some(other)) = (edge, other) {
                        let traction = dti
                            .mul(point(self.conductance(edge))?)?
                            .mul(v.sub(point(self.candidate[other])?)?)?;
                        defect = defect.add(traction)?;
                        scale = scale.add(traction.abs()?)?;
                    }
                }
                accept(defect, scale, i)?;
                equation_max = equation_max.max(defect.abs()?.hi);
                before = before.add(half.mul(m)?.mul(u)?.mul(u)?)?;
                after = after.add(half.mul(m)?.mul(v)?.mul(v)?)?;
                change = change.add(half.mul(m)?.mul(diff)?.mul(v.add(u)?)?)?;
                increment = increment.add(half.mul(m)?.mul(diff)?.mul(diff)?)?;
                momentum = momentum.add(inertia)?;
                momentum_scale = momentum_scale.add(m.mul(u)?.abs()?.add(m.mul(v)?.abs()?)?)?;
            }
            if i + 1 < n {
                let relative = point(self.candidate[i + 1])?.sub(v)?;
                loss = loss.add(
                    dti.mul(point(self.conductance(i))?)?
                        .mul(relative)?
                        .mul(relative)?,
                )?;
            }
        }
        for (end, (wall, cell, edge)) in [(0, 1, 0), (n - 1, n - 2, n - 2)].into_iter().enumerate()
        {
            let w = point(self.candidate[wall])?;
            let j = dti
                .mul(point(self.conductance(edge))?)?
                .mul(w.sub(point(self.candidate[cell])?)?)?;
            impulses[end] = j;
            if self.prescribed(wall) {
                work = work.add(j.mul(w)?)?;
                external = external.add(j)?;
                momentum_scale = momentum_scale.add(j.abs()?)?;
            }
        }
        let energy_defect = change.add(increment)?.add(loss)?.sub(work)?;
        let energy_scale = before
            .add(after)?
            .add(increment)?
            .add(loss)?
            .add(work.abs()?)?;
        let energy_budget = accept(energy_defect, energy_scale, n)?;
        let momentum_defect = momentum.sub(external)?;
        let momentum_budget = accept(momentum_defect, momentum_scale, n + 1)?;
        // Unforced closed dynamic system cannot gain kinetic energy. Prescribed
        // stationary plates also supply exactly zero work.
        if work.lo == 0.0 && work.hi == 0.0 && change.hi > 0.0 {
            return Err(Error::Gate {
                stage: Stage::Acceptance,
                index: n + 2,
            });
        }
        let report = Report {
            dt_s: dt,
            tangent: self.tangent,
            fluid_impulse_n_s: impulses,
            kinetic_before_j: before,
            kinetic_after_j: after,
            kinetic_change_j: change,
            increment_energy_j: increment,
            dissipation_j: loss,
            prescribed_work_j: work,
            energy_defect_j: energy_defect,
            energy_budget_j: energy_budget,
            momentum_change_n_s: momentum,
            prescribed_impulse_n_s: external,
            momentum_defect_n_s: momentum_defect,
            momentum_budget_n_s: momentum_budget,
            equation_defect_max_n_s: equation_max,
        };
        check(&mut cancel, Stage::Publication, 0)?;
        fluid.copy_from_slice(&self.candidate[1..n - 1]);
        *walls = [self.candidate[0], self.candidate[n - 1]];
        self.last = Some(report);
        Ok(report)
    }
}
