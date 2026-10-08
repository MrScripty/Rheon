//! Sealed, stationary pressure projection on the retained static obstacle owner.
//! The face mass rho A d is the declared face-area approximation, not the exact
//! clipped dual volume. All stored fields and arithmetic are binary64.
use crate::{Axis, NO_FLUID_COMPONENT, PressureSettings, StaticObstacleGeometry};
use std::{fmt, mem::size_of};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObstacleFlowStage {
    Assembly,
    Solve,
    Correction,
    Acceptance,
}
#[derive(Debug, Clone, PartialEq)]
pub enum ObstacleFlowError {
    InvalidParameter,
    ShapeMismatch,
    NonFiniteInput,
    ArithmeticFailure,
    NonzeroWallSpeed {
        axis: Axis,
        face: usize,
    },
    NonzeroDryRhs {
        cell: usize,
    },
    IncompatibleComponent {
        component: usize,
        sum: f64,
        rounding_budget: f64,
    },
    UnsupportedShearGeometry,
    IterationLimit {
        iterations: usize,
        divergence: f64,
    },
    DivergenceLimit {
        actual: f64,
        limit: f64,
    },
    EnergyLimit,
    Cancelled {
        stage: ObstacleFlowStage,
        index: usize,
    },
    AllocationFailure,
    CapacityOverflow,
    BufferLimit {
        required: usize,
        limit: usize,
    },
}
impl fmt::Display for ObstacleFlowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "static obstacle flow refused: {self:?}")
    }
}
impl std::error::Error for ObstacleFlowError {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObstaclePressureReport {
    pub iterations: usize,
    /// ||b - Lp||_2, m³/s²; includes gauge rows.
    pub residual_l2: f64,
    pub residual_max: f64,
    /// max_i dt |b_i - (Lp)_i| / V_i, 1/s; excludes dry cells only.
    pub predicted_divergence_max: f64,
    /// Set by project; solve_rhs has no corrected field and returns None.
    pub corrected: Option<ObstacleProjectionLedger>,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObstacleProjectionLedger {
    pub actual_divergence_max: f64,
    /// Kinetic energies use the declared mass rho A_f d_f, J.
    pub kinetic_before: f64,
    pub kinetic_after: f64,
    pub correction_energy: f64,
    /// dt sum_i p_i Q_i(after), J. This is numerical residual work,
    /// not work by the stationary wall.
    pub residual_work: f64,
    /// E_after - E_before + E_correction - residual_work, J.
    pub identity_error: f64,
    pub rounding_budget: f64,
}

pub(crate) fn checked(x: f64) -> Result<f64, ObstacleFlowError> {
    if x == 0.0 || x.is_normal() {
        Ok(x)
    } else {
        Err(ObstacleFlowError::ArithmeticFailure)
    }
}
pub(crate) fn mul(a: f64, b: f64) -> Result<f64, ObstacleFlowError> {
    let x = checked(a * b)?;
    if a != 0.0 && b != 0.0 && x == 0.0 {
        Err(ObstacleFlowError::ArithmeticFailure)
    } else {
        Ok(x)
    }
}
pub(crate) fn div(a: f64, b: f64) -> Result<f64, ObstacleFlowError> {
    if !b.is_normal() {
        return Err(ObstacleFlowError::ArithmeticFailure);
    }
    let x = checked(a / b)?;
    if a != 0.0 && x == 0.0 {
        Err(ObstacleFlowError::ArithmeticFailure)
    } else {
        Ok(x)
    }
}
pub(crate) fn positive(x: f64) -> Result<f64, ObstacleFlowError> {
    if x.is_normal() && x > 0.0 {
        Ok(x)
    } else {
        Err(ObstacleFlowError::InvalidParameter)
    }
}
pub(crate) fn checkpoint(
    cancel: &mut impl FnMut(ObstacleFlowStage, usize) -> bool,
    stage: ObstacleFlowStage,
    index: usize,
) -> Result<(), ObstacleFlowError> {
    if cancel(stage, index) {
        Err(ObstacleFlowError::Cancelled { stage, index })
    } else {
        Ok(())
    }
}
pub(crate) fn gate(required: usize, limit: usize) -> Result<(), ObstacleFlowError> {
    if required > limit {
        Err(ObstacleFlowError::BufferLimit { required, limit })
    } else {
        Ok(())
    }
}
pub(crate) fn allocate<T: Clone>(
    n: usize,
    value: T,
    used: &mut usize,
    limit: usize,
) -> Result<Vec<T>, ObstacleFlowError> {
    let bytes = n
        .checked_mul(size_of::<T>())
        .and_then(|v| v.checked_add(*used))
        .ok_or(ObstacleFlowError::CapacityOverflow)?;
    gate(bytes, limit)?;
    let mut v = Vec::new();
    v.try_reserve_exact(n)
        .map_err(|_| ObstacleFlowError::AllocationFailure)?;
    *used = v
        .capacity()
        .checked_mul(size_of::<T>())
        .and_then(|v| v.checked_add(*used))
        .ok_or(ObstacleFlowError::CapacityOverflow)?;
    gate(*used, limit)?;
    v.resize(n, value);
    Ok(v)
}
pub(crate) fn coordinate(dims: [usize; 3], i: usize) -> [usize; 3] {
    [i % dims[0], i / dims[0] % dims[1], i / (dims[0] * dims[1])]
}
#[derive(Default)]
pub(crate) struct Sum {
    value: f64,
    correction: f64,
}
impl Sum {
    pub(crate) fn add(&mut self, x: f64) -> Result<(), ObstacleFlowError> {
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
    pub(crate) fn finish(self) -> Result<f64, ObstacleFlowError> {
        checked(self.value + self.correction)
    }
}
fn dot(a: &[f64], b: &[f64]) -> Result<f64, ObstacleFlowError> {
    let mut s = Sum::default();
    for (&a, &b) in a.iter().zip(b) {
        s.add(mul(a, b)?)?;
    }
    s.finish()
}

/// Borrows one immutable mesh/volume/area/component owner. Seven cell arrays,
/// three candidate face arrays, and one gauge index per component are retained.
/// Solves allocate nothing. The cap covers their actual Vec capacity payload;
/// it excludes the borrowed owner, stack, allocator metadata and process RSS.
/// Pressure scratch is observable only after a successful operation. A refused
/// projection never modifies the caller's velocity. No transport or fluid state
/// composition with the older filled-box carrier is implied.
pub struct StaticObstaclePressure<'a> {
    geometry: &'a StaticObstacleGeometry,
    density: f64,
    gauges: Vec<usize>,
    diagonal: Vec<f64>,
    pressure: Vec<f64>,
    rhs: Vec<f64>,
    residual: Vec<f64>,
    direction: Vec<f64>,
    product: Vec<f64>,
    preconditioned: Vec<f64>,
    candidate: [Vec<f64>; 3],
    allocated_bytes: usize,
    qualified: bool,
}
#[derive(Clone, Copy)]
struct Edge {
    axis: usize,
    face: usize,
    tail: usize,
    head: usize,
    distance: f64,
    weight: f64,
    mass: f64,
}
fn edges(
    geometry: &StaticObstacleGeometry,
    density: f64,
    mut f: impl FnMut(Edge) -> Result<(), ObstacleFlowError>,
) -> Result<(), ObstacleFlowError> {
    let g = geometry.grid();
    for axis in Axis::ALL {
        let d = axis.index();
        for face in 0..g.face_len(axis) {
            let p = coordinate(g.face_counts(axis), face);
            if p[d] == 0 || p[d] == g.counts()[d] {
                continue;
            }
            let area = geometry.open_areas(axis)[face];
            if area == 0.0 {
                continue;
            }
            let mut q = p;
            q[d] -= 1;
            let tail = g.cell_unchecked(q);
            let head = g.cell_unchecked(p);
            if geometry.fluid_volumes()[tail] <= 0.0
                || geometry.fluid_volumes()[head] <= 0.0
                || geometry.component_labels()[tail] != geometry.component_labels()[head]
            {
                return Err(ObstacleFlowError::InvalidParameter);
            }
            // Represented grid centers. No center-distance or cut-volume clamp.
            let origin = g.origin()[d];
            let h = g.spacing()[d];
            let distance =
                positive((origin + (p[d] as f64 + 0.5) * h) - (origin + (p[d] as f64 - 0.5) * h))?;
            let weight = positive(div(area, mul(density, distance)?)?)?;
            let mass = positive(mul(mul(density, area)?, distance)?)?;
            f(Edge {
                axis: d,
                face,
                tail,
                head,
                distance,
                weight,
                mass,
            })?;
        }
    }
    Ok(())
}
fn apply(
    geometry: &StaticObstacleGeometry,
    density: f64,
    p: &[f64],
    out: &mut [f64],
) -> Result<(), ObstacleFlowError> {
    out.fill(0.0);
    edges(geometry, density, |e| {
        let x = mul(e.weight, checked(p[e.tail] - p[e.head])?)?;
        out[e.tail] = checked(out[e.tail] + x)?;
        out[e.head] = checked(out[e.head] - x)?;
        Ok(())
    })
}
impl<'a> StaticObstaclePressure<'a> {
    pub fn new(
        geometry: &'a StaticObstacleGeometry,
        density: f64,
        limit: usize,
        mut cancel: impl FnMut(ObstacleFlowStage, usize) -> bool,
    ) -> Result<Self, ObstacleFlowError> {
        positive(density)?;
        let n = geometry.grid().cell_len();
        let c = geometry.component_count();
        let faces = Axis::ALL
            .into_iter()
            .try_fold(0usize, |s, a| s.checked_add(geometry.grid().face_len(a)))
            .ok_or(ObstacleFlowError::CapacityOverflow)?;
        let planned = n
            .checked_mul(7)
            .and_then(|v| v.checked_add(faces))
            .and_then(|v| v.checked_mul(8))
            .and_then(|v| {
                c.checked_mul(size_of::<usize>())
                    .and_then(|c| v.checked_add(c))
            })
            .ok_or(ObstacleFlowError::CapacityOverflow)?;
        gate(planned, limit)?;
        let mut used = 0;
        let mut gauges = allocate(c, usize::MAX, &mut used, limit)?;
        let mut diagonal = allocate(n, 0.0, &mut used, limit)?;
        for (i, &label) in geometry.component_labels().iter().enumerate() {
            checkpoint(&mut cancel, ObstacleFlowStage::Assembly, i)?;
            if label != NO_FLUID_COMPONENT && gauges[label] == usize::MAX {
                gauges[label] = i;
            }
        }
        edges(geometry, density, |e| {
            checkpoint(&mut cancel, ObstacleFlowStage::Assembly, e.face)?;
            diagonal[e.tail] = checked(diagonal[e.tail] + e.weight)?;
            diagonal[e.head] = checked(diagonal[e.head] + e.weight)?;
            Ok(())
        })?;
        let pressure = allocate(n, 0.0, &mut used, limit)?;
        let rhs = allocate(n, 0.0, &mut used, limit)?;
        let residual = allocate(n, 0.0, &mut used, limit)?;
        let direction = allocate(n, 0.0, &mut used, limit)?;
        let product = allocate(n, 0.0, &mut used, limit)?;
        let preconditioned = allocate(n, 0.0, &mut used, limit)?;
        let mut candidate = [Vec::new(), Vec::new(), Vec::new()];
        for axis in Axis::ALL {
            candidate[axis.index()] =
                allocate(geometry.grid().face_len(axis), 0.0, &mut used, limit)?;
        }
        Ok(Self {
            geometry,
            density,
            gauges,
            diagonal,
            pressure,
            rhs,
            residual,
            direction,
            product,
            preconditioned,
            candidate,
            allocated_bytes: used,
            qualified: false,
        })
    }
    pub fn geometry(&self) -> &'a StaticObstacleGeometry {
        self.geometry
    }
    pub fn density(&self) -> f64 {
        self.density
    }
    pub fn allocated_bytes(&self) -> usize {
        self.allocated_bytes
    }
    pub fn gauge_cells(&self) -> &[usize] {
        &self.gauges
    }
    pub fn pressure(&self) -> Option<&[f64]> {
        self.qualified.then_some(&self.pressure)
    }
    /// Full unpinned graph operator, with zero dry rows. Gauge elimination is
    /// confined to the solver; the residual always includes every wet row.
    pub fn apply(&self, p: &[f64], out: &mut [f64]) -> Result<(), ObstacleFlowError> {
        if p.len() != self.pressure.len() || out.len() != p.len() {
            return Err(ObstacleFlowError::ShapeMismatch);
        }
        if p.iter().any(|v| !v.is_finite()) {
            return Err(ObstacleFlowError::NonFiniteInput);
        }
        apply(self.geometry, self.density, p, out)
    }
    fn active(&self, i: usize) -> bool {
        let label = self.geometry.component_labels()[i];
        label != NO_FLUID_COMPONENT && self.gauges[label] != i
    }
    fn report(
        &mut self,
        dt: f64,
        iterations: usize,
    ) -> Result<ObstaclePressureReport, ObstacleFlowError> {
        apply(
            self.geometry,
            self.density,
            &self.pressure,
            &mut self.product,
        )?;
        let mut maximum = 0.0f64;
        let mut divergence = 0.0f64;
        for i in 0..self.pressure.len() {
            self.product[i] = checked(self.rhs[i] - self.product[i])?;
            maximum = maximum.max(self.product[i].abs());
            let volume = self.geometry.fluid_volumes()[i];
            if volume > 0.0 {
                divergence = divergence.max(div(mul(dt, self.product[i].abs())?, volume)?);
            }
        }
        Ok(ObstaclePressureReport {
            iterations,
            residual_l2: checked(dot(&self.product, &self.product)?.sqrt())?,
            residual_max: maximum,
            predicted_divergence_max: divergence,
            corrected: None,
        })
    }
    fn precondition(&mut self) -> Result<f64, ObstacleFlowError> {
        for i in 0..self.pressure.len() {
            if self.active(i) {
                self.preconditioned[i] = div(self.residual[i], self.diagonal[i])?;
            } else {
                self.residual[i] = 0.0;
                self.preconditioned[i] = 0.0;
            }
        }
        dot(&self.residual, &self.preconditioned)
    }
    fn solve_loaded(
        &mut self,
        dt: f64,
        settings: PressureSettings,
        cancel: &mut impl FnMut(ObstacleFlowStage, usize) -> bool,
    ) -> Result<ObstaclePressureReport, ObstacleFlowError> {
        positive(dt)?;
        if [
            settings.relative_residual,
            settings.absolute_residual,
            settings.divergence_limit,
        ]
        .into_iter()
        .any(|v| !v.is_finite() || v < 0.0)
        {
            return Err(ObstacleFlowError::InvalidParameter);
        }
        checkpoint(cancel, ObstacleFlowStage::Solve, 0)?;
        for i in 0..self.rhs.len() {
            if !self.rhs[i].is_finite() {
                return Err(ObstacleFlowError::NonFiniteInput);
            }
            if self.geometry.fluid_volumes()[i] == 0.0 && self.rhs[i] != 0.0 {
                return Err(ObstacleFlowError::NonzeroDryRhs { cell: i });
            }
        }
        for component in 0..self.gauges.len() {
            let mut sum = Sum::default();
            let mut absolute = Sum::default();
            for (i, &label) in self.geometry.component_labels().iter().enumerate() {
                if label == component {
                    sum.add(self.rhs[i])?;
                    absolute.add(self.rhs[i].abs())?;
                }
            }
            let sum = sum.finish()?;
            let rounding_budget = mul(64.0 * f64::EPSILON, absolute.finish()?)?;
            if sum.abs() > rounding_budget {
                return Err(ObstacleFlowError::IncompatibleComponent {
                    component,
                    sum,
                    rounding_budget,
                });
            }
        }
        self.pressure.fill(0.0);
        let threshold = settings.absolute_residual.max(mul(
            settings.relative_residual,
            checked(dot(&self.rhs, &self.rhs)?.sqrt())?,
        )?);
        let accepted = |r: &ObstaclePressureReport| {
            r.residual_l2 <= threshold && r.predicted_divergence_max <= settings.divergence_limit
        };
        let mut report = self.report(dt, 0)?;
        if accepted(&report) {
            return Ok(report);
        }
        self.residual.copy_from_slice(&self.product);
        let mut rz = self.precondition()?;
        self.direction.copy_from_slice(&self.preconditioned);
        for iteration in 1..=settings.max_iterations {
            checkpoint(cancel, ObstacleFlowStage::Solve, iteration)?;
            apply(
                self.geometry,
                self.density,
                &self.direction,
                &mut self.product,
            )?;
            let denominator = dot(&self.direction, &self.product)?;
            positive(denominator)?;
            positive(rz)?;
            let alpha = div(rz, denominator)?;
            for i in 0..self.pressure.len() {
                if self.active(i) {
                    self.pressure[i] = checked(self.pressure[i] + mul(alpha, self.direction[i])?)?;
                    self.residual[i] = checked(self.residual[i] - mul(alpha, self.product[i])?)?;
                }
            }
            report = self.report(dt, iteration)?;
            if accepted(&report) {
                return Ok(report);
            }
            let next = self.precondition()?;
            let beta = div(next, rz)?;
            for i in 0..self.pressure.len() {
                self.direction[i] =
                    checked(self.preconditioned[i] + mul(beta, self.direction[i])?)?;
            }
            rz = next;
        }
        Err(ObstacleFlowError::IterationLimit {
            iterations: settings.max_iterations,
            divergence: report.predicted_divergence_max,
        })
    }
    /// b=-Q*/dt in m³/s². Each sealed component must have sum b=0 up to
    /// the declared summation allowance; incompatible data are never shifted.
    pub fn solve_rhs(
        &mut self,
        rhs: &[f64],
        dt: f64,
        settings: PressureSettings,
        mut cancel: impl FnMut(ObstacleFlowStage, usize) -> bool,
    ) -> Result<ObstaclePressureReport, ObstacleFlowError> {
        self.qualified = false;
        if rhs.len() != self.rhs.len() {
            return Err(ObstacleFlowError::ShapeMismatch);
        }
        self.rhs.copy_from_slice(rhs);
        let report = self.solve_loaded(dt, settings, &mut cancel)?;
        checkpoint(&mut cancel, ObstacleFlowStage::Acceptance, 0)?;
        self.qualified = true;
        Ok(report)
    }
    /// Corrects open interior face speeds by -dt (p_head-p_tail)/(rho d).
    /// All outer faces and fully blocked faces must be exactly stationary.
    /// The caller's field is copied only after residual, actual divergence,
    /// energy identity and cancellation gates succeed.
    pub fn project(
        &mut self,
        velocity: [&mut [f64]; 3],
        dt: f64,
        settings: PressureSettings,
        mut cancel: impl FnMut(ObstacleFlowStage, usize) -> bool,
    ) -> Result<ObstaclePressureReport, ObstacleFlowError> {
        self.qualified = false;
        positive(dt)?;
        let g = self.geometry.grid();
        for axis in Axis::ALL {
            let d = axis.index();
            if velocity[d].len() != g.face_len(axis) {
                return Err(ObstacleFlowError::ShapeMismatch);
            }
            for (face, &speed) in velocity[d].iter().enumerate() {
                if !speed.is_finite() {
                    return Err(ObstacleFlowError::NonFiniteInput);
                }
                let p = coordinate(g.face_counts(axis), face);
                if (p[d] == 0
                    || p[d] == g.counts()[d]
                    || self.geometry.open_areas(axis)[face] == 0.0)
                    && speed != 0.0
                {
                    return Err(ObstacleFlowError::NonzeroWallSpeed { axis, face });
                }
            }
        }
        for cell in 0..g.cell_len() {
            let p = coordinate(g.counts(), cell);
            let q = self
                .geometry
                .outward_flux(p, [velocity[0], velocity[1], velocity[2]])
                .map_err(|_| ObstacleFlowError::ArithmeticFailure)?;
            self.rhs[cell] = div(-q, dt)?;
        }
        let mut report = self.solve_loaded(dt, settings, &mut cancel)?;
        for (d, v) in velocity.iter().enumerate() {
            self.candidate[d].copy_from_slice(v);
        }
        let mut before = Sum::default();
        let mut after = Sum::default();
        let mut correction = Sum::default();
        edges(self.geometry, self.density, |e| {
            checkpoint(&mut cancel, ObstacleFlowStage::Correction, e.face)?;
            let change = div(
                mul(dt, checked(self.pressure[e.head] - self.pressure[e.tail])?)?,
                mul(self.density, e.distance)?,
            )?;
            let old = velocity[e.axis][e.face];
            let new = checked(old - change)?;
            self.candidate[e.axis][e.face] = new;
            before.add(mul(0.5, mul(e.mass, mul(old, old)?)?)?)?;
            after.add(mul(0.5, mul(e.mass, mul(new, new)?)?)?)?;
            correction.add(mul(0.5, mul(e.mass, mul(change, change)?)?)?)?;
            Ok(())
        })?;
        let mut actual = 0.0f64;
        let mut residual_work = Sum::default();
        for cell in 0..g.cell_len() {
            checkpoint(&mut cancel, ObstacleFlowStage::Acceptance, cell)?;
            let v = self.geometry.fluid_volumes()[cell];
            if v == 0.0 {
                continue;
            }
            let q = self
                .geometry
                .outward_flux(
                    coordinate(g.counts(), cell),
                    self.candidate.each_ref().map(|v| v.as_slice()),
                )
                .map_err(|_| ObstacleFlowError::ArithmeticFailure)?;
            actual = actual.max(div(q.abs(), v)?);
            residual_work.add(mul(dt, mul(self.pressure[cell], q)?)?)?;
        }
        if actual > settings.divergence_limit {
            return Err(ObstacleFlowError::DivergenceLimit {
                actual,
                limit: settings.divergence_limit,
            });
        }
        let kinetic_before = before.finish()?;
        let kinetic_after = after.finish()?;
        let correction_energy = correction.finish()?;
        let residual_work = residual_work.finish()?;
        let identity_error = checked(
            checked(checked(kinetic_after - kinetic_before)? + correction_energy)? - residual_work,
        )?;
        let rounding_budget = mul(
            1024.0 * f64::EPSILON,
            checked(
                checked(kinetic_before + kinetic_after)?
                    + checked(correction_energy + residual_work.abs())?,
            )?,
        )?;
        if identity_error.abs() > rounding_budget
            || kinetic_after
                > checked(kinetic_before + checked(residual_work.abs() + rounding_budget)?)?
        {
            return Err(ObstacleFlowError::EnergyLimit);
        }
        checkpoint(&mut cancel, ObstacleFlowStage::Acceptance, g.cell_len())?;
        for (d, v) in velocity.into_iter().enumerate() {
            v.copy_from_slice(&self.candidate[d]);
        }
        report.corrected = Some(ObstacleProjectionLedger {
            actual_divergence_max: actual,
            kinetic_before,
            kinetic_after,
            correction_energy,
            residual_work,
            identity_error,
            rounding_budget,
        });
        self.qualified = true;
        Ok(report)
    }
}
