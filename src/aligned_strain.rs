//! Immutable strain rows on exactly aligned, padded stationary box geometry.
//! Sealed outer walls are free slip; the obstacle is no slip. This diagnostic
//! owner provides no time integration or pressure/velocity composition. Shear
//! rows enumerate strictly internal grid edges: outer shear strains vanish
//! identically under sealed normal and free-slip tangential traces and are
//! analytically eliminated before quadrature. Every included fluid sector,
//! including an identically zero row, is retained.
use crate::obstacle_pressure::{
    Sum, allocate, checked, checkpoint, coordinate, div, gate, mul, positive,
};
use crate::{Axis, ObstacleFlowError, ObstacleFlowStage, StaticObstacleGeometry};
use std::{fmt, mem::size_of};

#[derive(Debug, Clone, PartialEq)]
pub enum AlignedStrainError {
    UnsupportedGeometry,
    Flow(ObstacleFlowError),
}
impl From<ObstacleFlowError> for AlignedStrainError {
    fn from(value: ObstacleFlowError) -> Self {
        Self::Flow(value)
    }
}
impl fmt::Display for AlignedStrainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "aligned strain refused: {self:?}")
    }
}
impl std::error::Error for AlignedStrainError {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlignedStrainFace {
    pub axis: Axis,
    pub face: usize,
    pub coordinates: [usize; 3],
    pub position: [f64; 3],
    pub negative_cell: usize,
    pub positive_cell: usize,
    pub area: f64,
    pub distance: f64,
    /// Exactly the pressure owner's declared rho A d mass.
    pub mass: f64,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlignedStrainTerm {
    pub active: usize,
    pub coefficient: f64,
}
const EMPTY_TERM: AlignedStrainTerm = AlignedStrainTerm {
    active: 0,
    coefficient: 0.0,
};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignedStrainBoundary {
    Interior,
    ObstacleFlat,
    ObstacleCorner,
    OuterFreeSlip,
    Normal,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlignedStrainRow {
    /// Equal axes mean a cell-normal row; unequal axes mean an edge shear row.
    pub axes: [Axis; 2],
    pub coordinates: [usize; 3],
    /// Shear quadrant bits: bit 0 positive first axis, bit 1 positive second.
    pub quadrant: u8,
    pub boundary: AlignedStrainBoundary,
    /// Normal: 2 V; shear: actual represented fluid quadrant volume.
    pub weight: f64,
    terms: [AlignedStrainTerm; 4],
    count: usize,
}
impl AlignedStrainRow {
    pub fn terms(&self) -> &[AlignedStrainTerm] {
        &self.terms[..self.count]
    }
    fn add(&mut self, active: usize, coefficient: f64) -> Result<(), ObstacleFlowError> {
        if active == usize::MAX {
            return Ok(());
        }
        checked(coefficient)?;
        if coefficient != 0.0 {
            self.terms[self.count] = AlignedStrainTerm {
                active,
                coefficient,
            };
            self.count += 1;
        }
        Ok(())
    }
    fn evaluate(&self, u: &[f64]) -> Result<f64, ObstacleFlowError> {
        let mut sum = Sum::default();
        for term in self.terms() {
            sum.add(mul(term.coefficient, u[term.active])?)?;
        }
        sum.finish()
    }
}
fn blank_row() -> AlignedStrainRow {
    AlignedStrainRow {
        axes: [Axis::X; 2],
        coordinates: [0; 3],
        quadrant: 0,
        boundary: AlignedStrainBoundary::Normal,
        weight: 0.0,
        terms: [EMPTY_TERM; 4],
        count: 0,
    }
}
fn plane(g: &crate::GridGeometry, d: usize, i: usize) -> f64 {
    g.origin()[d] + i as f64 * g.spacing()[d]
}
fn center(g: &crate::GridGeometry, d: usize, i: usize) -> f64 {
    g.origin()[d] + (i as f64 + 0.5) * g.spacing()[d]
}
fn admission(
    geometry: &StaticObstacleGeometry,
) -> Result<([usize; 3], [usize; 3]), AlignedStrainError> {
    let g = geometry.grid();
    let (lower, upper) = geometry.box_bounds();
    let mut lo = [0; 3];
    let mut hi = [0; 3];
    for d in 0..3 {
        lo[d] = (1..g.counts()[d])
            .find(|&i| plane(g, d, i) == lower[d])
            .ok_or(AlignedStrainError::UnsupportedGeometry)?;
        hi[d] = (1..g.counts()[d])
            .find(|&i| plane(g, d, i) == upper[d])
            .ok_or(AlignedStrainError::UnsupportedGeometry)?;
        if lo[d] >= hi[d] {
            return Err(AlignedStrainError::UnsupportedGeometry);
        }
    }
    Ok((lo, hi))
}
fn active_face(geometry: &StaticObstacleGeometry, axis: Axis, face: usize) -> bool {
    let g = geometry.grid();
    let p = coordinate(g.face_counts(axis), face);
    p[axis.index()] > 0
        && p[axis.index()] < g.counts()[axis.index()]
        && geometry.open_areas(axis)[face] > 0.0
}
fn sectors(
    geometry: &StaticObstacleGeometry,
    a: usize,
    b: usize,
    p: [usize; 3],
) -> ([Option<usize>; 4], bool) {
    let g = geometry.grid();
    let mut cells = [None; 4];
    let mut outer = false;
    for (q, cell) in cells.iter_mut().enumerate() {
        let mut s = p;
        let mut inside = true;
        for (d, bit) in [(a, 1), (b, 2)] {
            if q & bit == 0 {
                if p[d] == 0 {
                    inside = false;
                } else {
                    s[d] -= 1;
                }
            } else if p[d] == g.counts()[d] {
                inside = false;
            }
        }
        if !inside {
            outer = true;
            continue;
        }
        let index = g.cell_unchecked(s);
        if geometry.fluid_volumes()[index] > 0.0 {
            *cell = Some(index);
        }
    }
    (cells, outer)
}
fn each_edge(
    geometry: &StaticObstacleGeometry,
    mut f: impl FnMut(
        usize,
        usize,
        [usize; 3],
        [Option<usize>; 4],
        bool,
    ) -> Result<(), ObstacleFlowError>,
) -> Result<(), ObstacleFlowError> {
    let n = geometry.grid().counts();
    for (a, b) in [(0, 1), (0, 2), (1, 2)] {
        let mut shape = n;
        shape[a] += 1;
        shape[b] += 1;
        for k in 0..shape[2] {
            for j in 0..shape[1] {
                for i in 0..shape[0] {
                    if [i, j, k][a] == 0
                        || [i, j, k][a] == n[a]
                        || [i, j, k][b] == 0
                        || [i, j, k][b] == n[b]
                    {
                        continue;
                    }
                    let p = [i, j, k];
                    let (cells, outer) = sectors(geometry, a, b, p);
                    f(a, b, p, cells, outer)?;
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlignedStrainLedger {
    /// mu u^T E^T W E u, W.
    pub dissipation: f64,
    /// u dot (-mu E^T W E u), W.
    pub force_work: f64,
    pub identity_error: f64,
    /// Heuristic diagnostic tolerance, not an IEEE error enclosure or an
    /// acceptance gate. It does not authorize integration or pressure coupling.
    pub rounding_budget: f64,
    /// Nearest-rounded coefficient estimate, explicitly UNENCLOSED. No step
    /// size or integration may be authorized from this diagnostic quantity.
    pub unenclosed_b_estimate: f64,
}
/// Bounded retained payload is active faces, sparse fixed-width rows, and three
/// full face-to-active maps plus one coefficient estimate per active face. The cap excludes borrowed geometry, caller fields,
/// stack, allocator metadata and process RSS. Actions allocate no heap memory.
/// Caller output may be partially written if arithmetic or cancellation fails.
pub struct AlignedStrain<'a> {
    geometry: &'a StaticObstacleGeometry,
    density: f64,
    viscosity: f64,
    lo: [usize; 3],
    hi: [usize; 3],
    active: Vec<AlignedStrainFace>,
    map: [Vec<usize>; 3],
    rows: Vec<AlignedStrainRow>,
    allocated_bytes: usize,
    unenclosed_b: f64,
    unenclosed_face_estimates: Vec<f64>,
}
impl<'a> AlignedStrain<'a> {
    pub fn new(
        geometry: &'a StaticObstacleGeometry,
        density: f64,
        viscosity: f64,
        limit: usize,
        mut cancel: impl FnMut(ObstacleFlowStage, usize) -> bool,
    ) -> Result<Self, AlignedStrainError> {
        positive(density)?;
        positive(viscosity)?;
        let (lo, hi) = admission(geometry)?;
        let g = geometry.grid();
        let mut nf = 0usize;
        let mut face_total = 0usize;
        for axis in Axis::ALL {
            face_total = face_total
                .checked_add(g.face_len(axis))
                .ok_or(ObstacleFlowError::CapacityOverflow)?;
            for face in 0..g.face_len(axis) {
                checkpoint(&mut cancel, ObstacleFlowStage::Assembly, face)?;
                if active_face(geometry, axis, face) {
                    nf = nf
                        .checked_add(1)
                        .ok_or(ObstacleFlowError::CapacityOverflow)?;
                }
            }
        }
        let mut nr = geometry
            .fluid_volumes()
            .iter()
            .filter(|&&v| v > 0.0)
            .count()
            .checked_mul(3)
            .ok_or(ObstacleFlowError::CapacityOverflow)?;
        each_edge(geometry, |_, _, _, cells, _| {
            nr = nr
                .checked_add(cells.iter().flatten().count())
                .ok_or(ObstacleFlowError::CapacityOverflow)?;
            checkpoint(&mut cancel, ObstacleFlowStage::Assembly, nr)
        })?;
        let planned = nf
            .checked_mul(size_of::<AlignedStrainFace>() + size_of::<f64>())
            .and_then(|x| {
                nr.checked_mul(size_of::<AlignedStrainRow>())
                    .and_then(|y| x.checked_add(y))
            })
            .and_then(|x| {
                face_total
                    .checked_mul(size_of::<usize>())
                    .and_then(|y| x.checked_add(y))
            })
            .ok_or(ObstacleFlowError::CapacityOverflow)?;
        gate(planned, limit)?;
        let mut used = 0;
        let mut map = [Vec::new(), Vec::new(), Vec::new()];
        for axis in Axis::ALL {
            map[axis.index()] = allocate(g.face_len(axis), usize::MAX, &mut used, limit)?;
        }
        let empty_face = AlignedStrainFace {
            axis: Axis::X,
            face: 0,
            coordinates: [0; 3],
            position: [0.0; 3],
            negative_cell: 0,
            positive_cell: 0,
            area: 0.0,
            distance: 0.0,
            mass: 0.0,
        };
        let mut active = allocate(nf, empty_face, &mut used, limit)?;
        let mut rows = allocate(nr, blank_row(), &mut used, limit)?;
        let mut cursor = 0;
        for axis in Axis::ALL {
            let d = axis.index();
            for (face, mapped) in map[d].iter_mut().enumerate() {
                checkpoint(&mut cancel, ObstacleFlowStage::Assembly, face)?;
                if !active_face(geometry, axis, face) {
                    continue;
                }
                let p = coordinate(g.face_counts(axis), face);
                let mut q = p;
                q[d] -= 1;
                let tail = g.cell_unchecked(q);
                let head = g.cell_unchecked(p);
                if geometry.fluid_volumes()[tail] <= 0.0
                    || geometry.fluid_volumes()[head] <= 0.0
                    || geometry.component_labels()[tail] != geometry.component_labels()[head]
                {
                    return Err(ObstacleFlowError::InvalidParameter.into());
                }
                let distance = positive(center(g, d, p[d]) - center(g, d, p[d] - 1))?;
                let area = geometry.open_areas(axis)[face];
                let mass = positive(mul(mul(density, area)?, distance)?)?;
                active[cursor] = AlignedStrainFace {
                    axis,
                    face,
                    coordinates: p,
                    position: std::array::from_fn(|a| {
                        if a == d {
                            plane(g, a, p[a])
                        } else {
                            center(g, a, p[a])
                        }
                    }),
                    negative_cell: tail,
                    positive_cell: head,
                    area,
                    distance,
                    mass,
                };
                *mapped = cursor;
                cursor += 1;
            }
        }
        cursor = 0;
        for (cell, &volume) in geometry.fluid_volumes().iter().enumerate() {
            checkpoint(&mut cancel, ObstacleFlowStage::Assembly, cell)?;
            if volume == 0.0 {
                continue;
            }
            let p = coordinate(g.counts(), cell);
            for axis in Axis::ALL {
                let d = axis.index();
                let mut row = blank_row();
                row.axes = [axis; 2];
                row.coordinates = p;
                row.weight = positive(mul(2.0, volume)?)?;
                let inverse = div(1.0, positive(plane(g, d, p[d] + 1) - plane(g, d, p[d]))?)?;
                row.add(map[d][g.face_unchecked(axis, p)], -inverse)?;
                let mut upper = p;
                upper[d] += 1;
                row.add(map[d][g.face_unchecked(axis, upper)], inverse)?;
                rows[cursor] = row;
                cursor += 1;
            }
        }
        each_edge(geometry, |a, b, p, cells, outer| {
            let count = cells.iter().flatten().count();
            if count == 0 {
                return Ok(());
            }
            let mut prototype = blank_row();
            prototype.axes = [Axis::ALL[a], Axis::ALL[b]];
            prototype.coordinates = p;
            if outer {
                prototype.boundary = AlignedStrainBoundary::OuterFreeSlip;
            } else if count == 4 {
                prototype.boundary = AlignedStrainBoundary::Interior;
                for (component, across) in [(a, b), (b, a)] {
                    let inv = div(
                        1.0,
                        positive(center(g, across, p[across]) - center(g, across, p[across] - 1))?,
                    )?;
                    let mut lower = p;
                    lower[across] -= 1;
                    prototype.add(
                        map[component][g.face_unchecked(Axis::ALL[component], lower)],
                        -inv,
                    )?;
                    prototype.add(
                        map[component][g.face_unchecked(Axis::ALL[component], p)],
                        inv,
                    )?;
                }
            } else if count == 2 {
                prototype.boundary = AlignedStrainBoundary::ObstacleFlat;
                let q0 = cells.iter().position(Option::is_some).unwrap();
                let q1 = cells.iter().rposition(Option::is_some).unwrap();
                let varying = q0 ^ q1;
                let (component, across, bit) = if varying == 1 {
                    (a, b, 2)
                } else if varying == 2 {
                    (b, a, 1)
                } else {
                    return Err(ObstacleFlowError::UnsupportedShearGeometry);
                };
                let plus = q0 & bit != 0;
                let mut sample = p;
                if !plus {
                    sample[across] -= 1;
                }
                let delta = positive(
                    (center(g, across, sample[across]) - plane(g, across, p[across])).abs(),
                )?;
                prototype.add(
                    map[component][g.face_unchecked(Axis::ALL[component], sample)],
                    div(if plus { 1.0 } else { -1.0 }, delta)?,
                )?;
            } else if count != 3 {
                return Err(ObstacleFlowError::UnsupportedShearGeometry);
            }
            for (quadrant, cell) in cells.into_iter().enumerate() {
                let Some(cell) = cell else {
                    continue;
                };
                checkpoint(&mut cancel, ObstacleFlowStage::Assembly, cursor)?;
                let mut row = prototype;
                row.quadrant = quadrant as u8;
                let sector_cell = coordinate(g.counts(), cell);
                let c = 3 - a - b;
                let width_a = positive((center(g, a, sector_cell[a]) - plane(g, a, p[a])).abs())?;
                let width_b = positive((center(g, b, sector_cell[b]) - plane(g, b, p[b])).abs())?;
                let width_c = positive(plane(g, c, p[c] + 1) - plane(g, c, p[c]))?;
                row.weight = positive(mul(mul(width_a, width_b)?, width_c)?)?;
                if !outer && count == 3 {
                    row.boundary = AlignedStrainBoundary::ObstacleCorner;
                    let missing = cells.iter().position(Option::is_none).unwrap();
                    for (component, across, bit) in [(a, b, 2), (b, a, 1)] {
                        let plus = missing & bit == 0;
                        if (quadrant & bit != 0) != plus {
                            continue;
                        }
                        let mut sample = p;
                        if !plus {
                            sample[across] -= 1;
                        }
                        let delta = positive(
                            (center(g, across, sample[across]) - plane(g, across, p[across])).abs(),
                        )?;
                        row.add(
                            map[component][g.face_unchecked(Axis::ALL[component], sample)],
                            div(if plus { 1.0 } else { -1.0 }, delta)?,
                        )?;
                    }
                }
                rows[cursor] = row;
                cursor += 1;
            }
            Ok(())
        })?;
        let mut unenclosed_face_estimates = allocate(nf, 0.0, &mut used, limit)?;
        for (i, row) in rows.iter().enumerate() {
            checkpoint(&mut cancel, ObstacleFlowStage::Assembly, i)?;
            let mut abs_sum = Sum::default();
            for term in row.terms() {
                abs_sum.add(term.coefficient.abs())?;
            }
            let scale = mul(row.weight, abs_sum.finish()?)?;
            for term in row.terms() {
                unenclosed_face_estimates[term.active] = checked(
                    unenclosed_face_estimates[term.active] + mul(scale, term.coefficient.abs())?,
                )?;
            }
        }
        let mut result = Self {
            geometry,
            density,
            viscosity,
            lo,
            hi,
            active,
            map,
            rows,
            allocated_bytes: used,
            unenclosed_b: 0.0,
            unenclosed_face_estimates,
        };
        // No enclosing-rounding claim: ordinary binary64, including stored rows.
        for (i, face) in result.active.iter().enumerate() {
            checkpoint(&mut cancel, ObstacleFlowStage::Assembly, i)?;
            result.unenclosed_face_estimates[i] =
                div(result.unenclosed_face_estimates[i], face.mass)?;
            result.unenclosed_b = result.unenclosed_b.max(result.unenclosed_face_estimates[i]);
        }
        Ok(result)
    }
    pub fn geometry(&self) -> &'a StaticObstacleGeometry {
        self.geometry
    }
    pub fn density(&self) -> f64 {
        self.density
    }
    pub fn viscosity(&self) -> f64 {
        self.viscosity
    }
    pub fn box_plane_indices(&self) -> ([usize; 3], [usize; 3]) {
        (self.lo, self.hi)
    }
    pub fn active_faces(&self) -> &[AlignedStrainFace] {
        &self.active
    }
    pub fn active_index(&self, axis: Axis, face: usize) -> Option<usize> {
        self.map[axis.index()]
            .get(face)
            .copied()
            .filter(|&i| i != usize::MAX)
    }
    pub fn rows(&self) -> &[AlignedStrainRow] {
        &self.rows
    }
    pub fn allocated_bytes(&self) -> usize {
        self.allocated_bytes
    }
    /// UNENCLOSED and independent of viscosity. Never authorizes a timestep.
    pub fn unenclosed_b_estimate(&self) -> f64 {
        self.unenclosed_b
    }
    /// UNENCLOSED nearest-rounded coefficient sums divided by each stored mass.
    pub fn unenclosed_face_estimates(&self) -> &[f64] {
        &self.unenclosed_face_estimates
    }
    fn validate(&self, u: &[f64], out: &[f64]) -> Result<(), ObstacleFlowError> {
        if u.len() != self.active.len() || out.len() != u.len() {
            return Err(ObstacleFlowError::ShapeMismatch);
        }
        if u.iter().any(|v| !v.is_finite()) {
            return Err(ObstacleFlowError::NonFiniteInput);
        }
        Ok(())
    }
    /// Apply the base stiffness E^T W E; viscosity is not included.
    pub fn apply(
        &self,
        u: &[f64],
        out: &mut [f64],
        mut cancel: impl FnMut(ObstacleFlowStage, usize) -> bool,
    ) -> Result<(), AlignedStrainError> {
        self.validate(u, out)?;
        out.fill(0.0);
        for (i, row) in self.rows.iter().enumerate() {
            checkpoint(&mut cancel, ObstacleFlowStage::Correction, i)?;
            let x = mul(row.weight, row.evaluate(u)?)?;
            for term in row.terms() {
                out[term.active] = checked(out[term.active] + mul(term.coefficient, x)?)?;
            }
        }
        Ok(())
    }
    /// Stationary viscous force -mu E^T W E u and force/work identity diagnostic.
    pub fn diagnose(
        &self,
        u: &[f64],
        force: &mut [f64],
        mut cancel: impl FnMut(ObstacleFlowStage, usize) -> bool,
    ) -> Result<AlignedStrainLedger, AlignedStrainError> {
        self.apply(u, force, &mut cancel)?;
        let mut work = Sum::default();
        let mut work_abs = Sum::default();
        for (i, (&u, force)) in u.iter().zip(force.iter_mut()).enumerate() {
            checkpoint(&mut cancel, ObstacleFlowStage::Acceptance, i)?;
            *force = -mul(self.viscosity, *force)?;
            let term = mul(u, *force)?;
            work.add(term)?;
            work_abs.add(term.abs())?;
        }
        let mut dissipation = Sum::default();
        for (i, row) in self.rows.iter().enumerate() {
            checkpoint(&mut cancel, ObstacleFlowStage::Acceptance, i)?;
            let strain = row.evaluate(u)?;
            dissipation.add(mul(mul(self.viscosity, row.weight)?, mul(strain, strain)?)?)?;
        }
        let dissipation = dissipation.finish()?;
        let force_work = work.finish()?;
        let identity_error = checked(force_work + dissipation)?;
        let rounding_budget = mul(
            256.0 * f64::EPSILON,
            checked(work_abs.finish()? + dissipation)?,
        )?;
        Ok(AlignedStrainLedger {
            dissipation,
            force_work,
            identity_error,
            rounding_budget,
            unenclosed_b_estimate: self.unenclosed_b,
        })
    }
}
