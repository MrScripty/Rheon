//! Explicitly opted-in, unforced aligned viscous update and pressure proposal.
//! Included only by the experimental example/tests, never by the production lib.
//! The caller's velocities and all accepted witnesses survive every refusal.
//! The workspace and reused operator/pressure paths allocate no heap storage
//! during a step. Cancellation callbacks are caller code and are outside this
//! storage audit, as are caller buffers and whole-process resident memory.
#[path = "outward.rs"]
mod outward;
pub use outward::{Error as EnclosureError, Interval};
use rheon::{
    AlignedStrain, AlignedStrainError, Axis, ObstacleFlowError, ObstacleFlowStage,
    ObstaclePressureReport, PressureSettings, StaticObstaclePressure,
};
use std::{fmt, mem::size_of};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Construction,
    Input,
    InitialDivergence,
    Bound,
    Action,
    ViscousUpdate,
    ViscousEquation,
    ViscousEnergy,
    Pressure(ObstacleFlowStage),
    PressureEquation,
    PressureEnergy,
    FinalDivergence,
    Commit,
}
#[derive(Debug)]
pub enum Error {
    InvalidParameter,
    ShapeMismatch,
    NonFiniteInput,
    ArithmeticFailure,
    NonzeroWallSpeed {
        axis: Axis,
        face: usize,
    },
    Enclosure(EnclosureError),
    Strain(AlignedStrainError),
    Pressure(ObstacleFlowError),
    CapacityOverflow,
    AllocationFailure,
    BufferLimit {
        required: usize,
        limit: usize,
    },
    Cancelled {
        stage: Stage,
        index: usize,
    },
    StepBound {
        product: Interval,
    },
    Divergence {
        stage: Stage,
        cell: usize,
        bound: Interval,
        limit: f64,
    },
    Energy {
        stage: Stage,
        change: Interval,
    },
    Equation {
        stage: Stage,
        active: usize,
        defect: Interval,
        scale: Interval,
        allowance: Interval,
    },
}
impl From<EnclosureError> for Error {
    fn from(e: EnclosureError) -> Self {
        Self::Enclosure(e)
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "experimental aligned Stokes step refused: {self:?}")
    }
}
impl std::error::Error for Error {}
fn pressure_error(e: ObstacleFlowError) -> Error {
    match e {
        ObstacleFlowError::Cancelled { stage, index } => Error::Cancelled {
            stage: Stage::Pressure(stage),
            index,
        },
        other => Error::Pressure(other),
    }
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
fn scalar(x: f64) -> Result<f64, Error> {
    if x == 0.0 || x.is_normal() {
        Ok(x)
    } else {
        Err(Error::ArithmeticFailure)
    }
}
fn multiply(a: f64, b: f64) -> Result<f64, Error> {
    let x = scalar(a * b)?;
    if a != 0.0 && b != 0.0 && x == 0.0 {
        Err(Error::ArithmeticFailure)
    } else {
        Ok(x)
    }
}
fn divide(a: f64, b: f64) -> Result<f64, Error> {
    if !b.is_normal() || b <= 0.0 {
        return Err(Error::ArithmeticFailure);
    }
    let x = scalar(a / b)?;
    if a != 0.0 && x == 0.0 {
        Err(Error::ArithmeticFailure)
    } else {
        Ok(x)
    }
}
fn axis(a: Axis) -> usize {
    match a {
        Axis::X => 0,
        Axis::Y => 1,
        Axis::Z => 2,
    }
}
fn gate(required: usize, limit: usize) -> Result<(), Error> {
    if required > limit {
        Err(Error::BufferLimit { required, limit })
    } else {
        Ok(())
    }
}
fn allocate<T: Clone>(n: usize, value: T, used: &mut usize, limit: usize) -> Result<Vec<T>, Error> {
    let planned = n
        .checked_mul(size_of::<T>())
        .and_then(|b| used.checked_add(b))
        .ok_or(Error::CapacityOverflow)?;
    gate(planned, limit)?;
    let mut v = Vec::new();
    v.try_reserve_exact(n)
        .map_err(|_| Error::AllocationFailure)?;
    *used = v
        .capacity()
        .checked_mul(size_of::<T>())
        .and_then(|b| used.checked_add(b))
        .ok_or(Error::CapacityOverflow)?;
    gate(*used, limit)?;
    v.resize(n, value);
    Ok(v)
}
fn bytes(n: usize, width: usize) -> Result<usize, Error> {
    n.checked_mul(width).ok_or(Error::CapacityOverflow)
}
fn plus(a: usize, b: usize) -> Result<usize, Error> {
    a.checked_add(b).ok_or(Error::CapacityOverflow)
}
fn coordinate(n: [usize; 3], i: usize) -> [usize; 3] {
    [i % n[0], i / n[0] % n[1], i / (n[0] * n[1])]
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub pressure: PressureSettings,
    pub divergence_limit: f64,
    pub relative_update_limit: f64,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            pressure: PressureSettings {
                relative_residual: 1e-12,
                absolute_residual: 1e-12,
                divergence_limit: 1e-10,
                max_iterations: 300,
            },
            divergence_limit: 1e-10,
            relative_update_limit: 4096.0 * f64::EPSILON,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComponentCertificate {
    pub defect: Interval,
    pub scale: Interval,
    pub allowance: Interval,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DivergenceCertificate {
    pub bound: Interval,
    pub upper_cell: usize,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Report {
    pub dt: f64,
    pub density: f64,
    pub viscosity: f64,
    pub bound: Interval,
    pub step_product: Interval,
    pub viscous_energy_delta: Interval,
    pub pressure_energy_delta: Interval,
    pub initial_divergence: DivergenceCertificate,
    pub final_divergence: DivergenceCertificate,
    /// Existing nearest-rounded proposal report. The independent certificates
    /// above and the component certificates are the acceptance authority.
    pub pressure_proposal: ObstaclePressureReport,
}

/// The cap includes borrowed strain payload, owned pressure payload, all attempt
/// buffers and accepted witness caches, measured by actual Vec capacity. It
/// excludes geometry, caller fields, stack, allocator metadata and process RSS.
/// Every step allocates nothing. No advection, forcing or simulation clock.
pub struct Workspace<'op, 'g> {
    op: &'op AlignedStrain<'g>,
    config: Config,
    pressure: StaticObstaclePressure<'g>,
    candidate: [Vec<f64>; 3],
    old: Vec<f64>,
    viscous: Vec<f64>,
    action: Vec<f64>,
    action_interval: Vec<Interval>,
    absolute_contributions: Vec<Interval>,
    update: Vec<ComponentCertificate>,
    momentum: Vec<ComponentCertificate>,
    accepted_initial: Vec<f64>,
    accepted_viscous: Vec<f64>,
    accepted_final: Vec<f64>,
    accepted_pressure: Vec<f64>,
    accepted_update: Vec<ComponentCertificate>,
    accepted_momentum: Vec<ComponentCertificate>,
    last_report: Option<Report>,
    bound: Interval,
    owned_bytes: usize,
    combined_bytes: usize,
}
impl<'op, 'g> Workspace<'op, 'g> {
    pub fn new(
        op: &'op AlignedStrain<'g>,
        config: Config,
        limit: usize,
        mut cancel: impl FnMut(Stage, usize) -> bool,
    ) -> Result<Self, Error> {
        outward::ensure_supported()?;
        if !config.divergence_limit.is_finite()
            || config.divergence_limit < 0.0
            || config.divergence_limit.is_subnormal()
            || !config.relative_update_limit.is_finite()
            || config.relative_update_limit < 0.0
            // The nearest 1e-8 literal lies above the exact decimal contract cap.
            || config.relative_update_limit > 1e-8_f64.next_down()
            || config.relative_update_limit.is_subnormal()
            || [
                config.pressure.relative_residual,
                config.pressure.absolute_residual,
                config.pressure.divergence_limit,
            ]
            .into_iter()
            .any(|x| !x.is_finite() || x < 0.0 || x.is_subnormal())
        {
            return Err(Error::InvalidParameter);
        }
        check(&mut cancel, Stage::Construction, 0)?;
        let g = op.geometry().grid();
        let n = op.active_faces().len();
        let cells = g.cell_len();
        let faces = [Axis::X, Axis::Y, Axis::Z]
            .into_iter()
            .try_fold(0, |s, a| plus(s, g.face_len(a)))?;
        let scratch = plus(
            plus(
                bytes(faces, 8)?,
                bytes(
                    n,
                    6 * 8 + 2 * size_of::<Interval>() + 4 * size_of::<ComponentCertificate>(),
                )?,
            )?,
            bytes(cells, 8)?,
        )?;
        let pressure_plan = plus(
            bytes(plus(bytes(cells, 7)?, faces)?, 8)?,
            bytes(op.geometry().component_count(), size_of::<usize>())?,
        )?;
        let planned = plus(op.allocated_bytes(), plus(scratch, pressure_plan)?)?;
        gate(planned, limit)?;
        let pressure_limit = limit
            .checked_sub(op.allocated_bytes())
            .and_then(|b| b.checked_sub(scratch))
            .ok_or(Error::CapacityOverflow)?;
        let pressure = StaticObstaclePressure::new(
            op.geometry(),
            op.density(),
            pressure_limit,
            |stage, index| cancel(Stage::Pressure(stage), index),
        )
        .map_err(pressure_error)?;
        let mut used = plus(op.allocated_bytes(), pressure.allocated_bytes())?;
        let zero = Interval::point(0.0)?;
        let blank = ComponentCertificate {
            defect: zero,
            scale: zero,
            allowance: zero,
        };
        let mut candidate = [Vec::new(), Vec::new(), Vec::new()];
        for a in [Axis::X, Axis::Y, Axis::Z] {
            candidate[axis(a)] = allocate(g.face_len(a), 0.0, &mut used, limit)?;
        }
        let old = allocate(n, 0.0, &mut used, limit)?;
        let viscous = allocate(n, 0.0, &mut used, limit)?;
        let action = allocate(n, 0.0, &mut used, limit)?;
        let action_interval = allocate(n, zero, &mut used, limit)?;
        let absolute_contributions = allocate(n, zero, &mut used, limit)?;
        let update = allocate(n, blank, &mut used, limit)?;
        let momentum = allocate(n, blank, &mut used, limit)?;
        let accepted_initial = allocate(n, 0.0, &mut used, limit)?;
        let accepted_viscous = allocate(n, 0.0, &mut used, limit)?;
        let accepted_final = allocate(n, 0.0, &mut used, limit)?;
        let accepted_pressure = allocate(cells, 0.0, &mut used, limit)?;
        let accepted_update = allocate(n, blank, &mut used, limit)?;
        let accepted_momentum = allocate(n, blank, &mut used, limit)?;
        let mut w = Self {
            op,
            config,
            pressure,
            candidate,
            old,
            viscous,
            action,
            action_interval,
            absolute_contributions,
            update,
            momentum,
            accepted_initial,
            accepted_viscous,
            accepted_final,
            accepted_pressure,
            accepted_update,
            accepted_momentum,
            last_report: None,
            bound: zero,
            owned_bytes: used - op.allocated_bytes(),
            combined_bytes: used,
        };
        w.bound = w.build_bound(&mut cancel)?;
        Ok(w)
    }
    fn build_bound(
        &mut self,
        cancel: &mut impl FnMut(Stage, usize) -> bool,
    ) -> Result<Interval, Error> {
        self.action_interval.fill(Interval::point(0.0)?);
        for (i, row) in self.op.rows().iter().enumerate() {
            check(cancel, Stage::Bound, i)?;
            let mut absolute = Interval::point(0.0)?;
            for t in row.terms() {
                absolute = absolute.add(Interval::point(t.coefficient.abs())?)?;
            }
            let weighted = Interval::point(row.weight)?.mul(absolute)?;
            for t in row.terms() {
                self.action_interval[t.active] = self.action_interval[t.active]
                    .add(weighted.mul(Interval::point(t.coefficient.abs())?)?)?;
            }
        }
        let mut lo = 0.0f64;
        let mut hi = 0.0f64;
        for (i, f) in self.op.active_faces().iter().enumerate() {
            check(cancel, Stage::Bound, i)?;
            let b = self.action_interval[i].div(Interval::point(f.mass)?)?;
            lo = lo.max(b.lo);
            hi = hi.max(b.hi);
        }
        Ok(Interval::new(lo, hi)?)
    }
    fn validate(
        &self,
        velocity: [&[f64]; 3],
        cancel: &mut impl FnMut(Stage, usize) -> bool,
    ) -> Result<(), Error> {
        for a in [Axis::X, Axis::Y, Axis::Z] {
            let d = axis(a);
            if velocity[d].len() != self.op.geometry().grid().face_len(a) {
                return Err(Error::ShapeMismatch);
            }
            for (face, &v) in velocity[d].iter().enumerate() {
                check(cancel, Stage::Input, face)?;
                if !v.is_finite() {
                    return Err(Error::NonFiniteInput);
                }
                scalar(v)?;
                if self.op.active_index(a, face).is_none() && v != 0.0 {
                    return Err(Error::NonzeroWallSpeed { axis: a, face });
                }
            }
        }
        Ok(())
    }
    fn divergence(
        &self,
        velocity: [&[f64]; 3],
        stage: Stage,
        cancel: &mut impl FnMut(Stage, usize) -> bool,
    ) -> Result<DivergenceCertificate, Error> {
        let geometry = self.op.geometry();
        let g = geometry.grid();
        let mut maximum = Interval::point(0.0)?;
        let mut upper_cell = 0;
        for (cell, &volume) in geometry.fluid_volumes().iter().enumerate() {
            check(cancel, stage, cell)?;
            if volume == 0.0 {
                continue;
            }
            let p = coordinate(g.counts(), cell);
            let mut flux = Interval::point(0.0)?;
            for a in [Axis::X, Axis::Y, Axis::Z] {
                let d = axis(a);
                let low = g.face_index(a, p).ok_or(Error::ShapeMismatch)?;
                let mut q = p;
                q[d] += 1;
                let high = g.face_index(a, q).ok_or(Error::ShapeMismatch)?;
                let area = geometry.open_areas(a);
                let high_flux =
                    Interval::point(area[high])?.mul(Interval::point(velocity[d][high])?)?;
                let low_flux =
                    Interval::point(area[low])?.mul(Interval::point(velocity[d][low])?)?;
                flux = flux.add(high_flux.sub(low_flux)?)?;
            }
            let bound = flux.abs()?.div(Interval::point(volume)?)?;
            if bound.hi > self.config.divergence_limit {
                return Err(Error::Divergence {
                    stage,
                    cell,
                    bound,
                    limit: self.config.divergence_limit,
                });
            }
            if bound.hi > maximum.hi {
                upper_cell = cell;
            }
            maximum = Interval::new(maximum.lo.max(bound.lo), maximum.hi.max(bound.hi))?;
        }
        Ok(DivergenceCertificate {
            bound: maximum,
            upper_cell,
        })
    }
    fn enclose_action(
        &mut self,
        cancel: &mut impl FnMut(Stage, usize) -> bool,
    ) -> Result<(), Error> {
        self.action_interval.fill(Interval::point(0.0)?);
        self.absolute_contributions.fill(Interval::point(0.0)?);
        for (i, row) in self.op.rows().iter().enumerate() {
            check(cancel, Stage::ViscousEquation, i)?;
            let mut strain = Interval::point(0.0)?;
            for t in row.terms() {
                strain = strain.add(
                    Interval::point(t.coefficient)?.mul(Interval::point(self.old[t.active])?)?,
                )?;
            }
            let weighted = Interval::point(row.weight)?.mul(strain)?;
            for t in row.terms() {
                let contribution = Interval::point(t.coefficient)?.mul(weighted)?;
                self.action_interval[t.active] =
                    self.action_interval[t.active].add(contribution)?;
                self.absolute_contributions[t.active] =
                    self.absolute_contributions[t.active].add(contribution.abs()?)?;
            }
        }
        Ok(())
    }
    fn component(
        &self,
        defect: Interval,
        scale: Interval,
        stage: Stage,
        active: usize,
    ) -> Result<ComponentCertificate, Error> {
        let allowance = Interval::point(self.config.relative_update_limit)?.mul(scale)?;
        if defect.abs_upper()? > allowance.lo {
            return Err(Error::Equation {
                stage,
                active,
                defect,
                scale,
                allowance,
            });
        }
        Ok(ComponentCertificate {
            defect,
            scale,
            allowance,
        })
    }
    fn energy(
        &self,
        before: &[f64],
        after: &[f64],
        stage: Stage,
        cancel: &mut impl FnMut(Stage, usize) -> bool,
    ) -> Result<Interval, Error> {
        let mut total = Interval::point(0.0)?;
        for (i, f) in self.op.active_faces().iter().enumerate() {
            check(cancel, stage, i)?;
            let old = Interval::point(before[i])?;
            let new = Interval::point(after[i])?;
            let term = Interval::point(0.5)?
                .mul(Interval::point(f.mass)?)?
                .mul(new.sub(old)?)?
                .mul(new.add(old)?)?;
            total = total.add(term)?;
        }
        if total.hi > 0.0 {
            return Err(Error::Energy {
                stage,
                change: total,
            });
        }
        Ok(total)
    }
    fn pressure_energy(
        &self,
        cancel: &mut impl FnMut(Stage, usize) -> bool,
    ) -> Result<Interval, Error> {
        let mut total = Interval::point(0.0)?;
        for (i, f) in self.op.active_faces().iter().enumerate() {
            check(cancel, Stage::PressureEnergy, i)?;
            let old = Interval::point(self.viscous[i])?;
            let new = Interval::point(self.candidate[axis(f.axis)][f.face])?;
            total = total.add(
                Interval::point(0.5)?
                    .mul(Interval::point(f.mass)?)?
                    .mul(new.sub(old)?)?
                    .mul(new.add(old)?)?,
            )?;
        }
        if total.hi > 0.0 {
            return Err(Error::Energy {
                stage: Stage::PressureEnergy,
                change: total,
            });
        }
        Ok(total)
    }
    pub fn step(
        &mut self,
        velocity: [&mut [f64]; 3],
        dt: f64,
        mut cancel: impl FnMut(Stage, usize) -> bool,
    ) -> Result<Report, Error> {
        outward::ensure_supported()?;
        if !dt.is_normal() || dt <= 0.0 {
            return Err(Error::InvalidParameter);
        }
        self.validate(velocity.each_ref().map(|v| v.as_ref()), &mut cancel)?;
        let initial_divergence = self.divergence(
            velocity.each_ref().map(|v| v.as_ref()),
            Stage::InitialDivergence,
            &mut cancel,
        )?;
        let t = Interval::point(dt)?.mul(Interval::point(self.op.viscosity())?)?;
        let step_product = t.mul(Interval::point(self.bound.hi)?)?;
        check(&mut cancel, Stage::Bound, 0)?;
        if step_product.hi > 2.0 {
            return Err(Error::StepBound {
                product: step_product,
            });
        }
        for (i, f) in self.op.active_faces().iter().enumerate() {
            self.old[i] = velocity[axis(f.axis)][f.face];
        }
        self.op
            .apply(&self.old, &mut self.action, |_, index| {
                cancel(Stage::Action, index)
            })
            .map_err(|e| match e {
                AlignedStrainError::Flow(ObstacleFlowError::Cancelled { index, .. }) => {
                    Error::Cancelled {
                        stage: Stage::Action,
                        index,
                    }
                }
                other => Error::Strain(other),
            })?;
        let rounded_t = multiply(dt, self.op.viscosity())?;
        for (d, field) in velocity.iter().enumerate() {
            self.candidate[d].copy_from_slice(field);
        }
        for (i, f) in self.op.active_faces().iter().enumerate() {
            check(&mut cancel, Stage::ViscousUpdate, i)?;
            let change = divide(multiply(rounded_t, self.action[i])?, f.mass)?;
            let v = scalar(self.old[i] - change)?;
            self.viscous[i] = v;
            self.candidate[axis(f.axis)][f.face] = v;
        }
        self.enclose_action(&mut cancel)?;
        for (i, f) in self.op.active_faces().iter().enumerate() {
            check(&mut cancel, Stage::ViscousEquation, i)?;
            let inertia = Interval::point(f.mass)?
                .mul(Interval::point(self.viscous[i])?.sub(Interval::point(self.old[i])?)?)?;
            let defect = inertia.add(t.mul(self.action_interval[i])?)?;
            let scale = inertia.abs()?.add(t.mul(self.absolute_contributions[i])?)?;
            self.update[i] = self.component(defect, scale, Stage::ViscousEquation, i)?;
        }
        let viscous_energy_delta =
            self.energy(&self.old, &self.viscous, Stage::ViscousEnergy, &mut cancel)?;
        let [x, y, z] = &mut self.candidate;
        let pressure_proposal = self
            .pressure
            .project([x, y, z], dt, self.config.pressure, |stage, index| {
                cancel(Stage::Pressure(stage), index)
            })
            .map_err(pressure_error)?;
        let pressure = self.pressure.pressure().ok_or(Error::ArithmeticFailure)?;
        for (i, f) in self.op.active_faces().iter().enumerate() {
            check(&mut cancel, Stage::PressureEquation, i)?;
            let jump = Interval::point(pressure[f.positive_cell])?
                .sub(Interval::point(pressure[f.negative_cell])?)?;
            let correction = Interval::point(dt)?
                .mul(Interval::point(f.area)?)?
                .mul(jump)?;
            let inertia = Interval::point(f.mass)?.mul(
                Interval::point(self.candidate[axis(f.axis)][f.face])?
                    .sub(Interval::point(self.viscous[i])?)?,
            )?;
            self.momentum[i] = self.component(
                inertia.add(correction)?,
                inertia.abs()?.add(correction.abs()?)?,
                Stage::PressureEquation,
                i,
            )?;
        }
        let pressure_energy_delta = self.pressure_energy(&mut cancel)?;
        self.validate(self.candidate.each_ref().map(|v| v.as_slice()), &mut cancel)?;
        let final_divergence = self.divergence(
            self.candidate.each_ref().map(|v| v.as_slice()),
            Stage::FinalDivergence,
            &mut cancel,
        )?;
        let report = Report {
            dt,
            density: self.op.density(),
            viscosity: self.op.viscosity(),
            bound: self.bound,
            step_product,
            viscous_energy_delta,
            pressure_energy_delta,
            initial_divergence,
            final_divergence,
            pressure_proposal,
        };
        check(&mut cancel, Stage::Commit, 0)?;
        self.accepted_initial.copy_from_slice(&self.old);
        self.accepted_viscous.copy_from_slice(&self.viscous);
        for (i, f) in self.op.active_faces().iter().enumerate() {
            self.accepted_final[i] = self.candidate[axis(f.axis)][f.face];
        }
        self.accepted_pressure.copy_from_slice(pressure);
        self.accepted_update.copy_from_slice(&self.update);
        self.accepted_momentum.copy_from_slice(&self.momentum);
        for (d, field) in velocity.into_iter().enumerate() {
            field.copy_from_slice(&self.candidate[d]);
        }
        self.last_report = Some(report);
        Ok(report)
    }
    pub fn operator(&self) -> &'op AlignedStrain<'g> {
        self.op
    }
    pub fn gauge_cells(&self) -> &[usize] {
        self.pressure.gauge_cells()
    }
    pub fn config(&self) -> Config {
        self.config
    }
    pub fn allocated_bytes(&self) -> usize {
        self.owned_bytes
    }
    pub fn combined_bytes(&self) -> usize {
        self.combined_bytes
    }
    pub fn bound(&self) -> Interval {
        self.bound
    }
    pub fn last_report(&self) -> Option<&Report> {
        self.last_report.as_ref()
    }
    pub fn initial_active(&self) -> Option<&[f64]> {
        self.last_report.map(|_| self.accepted_initial.as_slice())
    }
    pub fn viscous_active(&self) -> Option<&[f64]> {
        self.last_report.map(|_| self.accepted_viscous.as_slice())
    }
    pub fn final_active(&self) -> Option<&[f64]> {
        self.last_report.map(|_| self.accepted_final.as_slice())
    }
    pub fn pressure(&self) -> Option<&[f64]> {
        self.last_report.map(|_| self.accepted_pressure.as_slice())
    }
    pub fn update_certificates(&self) -> Option<&[ComponentCertificate]> {
        self.last_report.map(|_| self.accepted_update.as_slice())
    }
    pub fn momentum_certificates(&self) -> Option<&[ComponentCertificate]> {
        self.last_report.map(|_| self.accepted_momentum.as_slice())
    }
}
