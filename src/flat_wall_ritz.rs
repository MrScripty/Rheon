//! Opt-in reduced steady Ritz research provider. No pressure or time integration.
//! Only the [0,3]^3 / [1,2]^3 lower-Y trial patch is admitted. A small projected
//! residual, outward arithmetic intervals, or a wall observation does not qualify
//! physical convergence. The physical campaign is separately authorized.
use crate::obstacle_pressure::{Sum, checked, checkpoint, div, mul, positive};
use crate::*;
use std::{fmt, mem::size_of};

const MAX_Q: usize = 36;
const MAX_TERMS: usize = 1024;
const MAX_DEGREE: u8 = 16;
/// This implementation lowers the reviewed envelope; it never raises the parent cap.
pub const FLAT_WALL_RITZ_ENVELOPE: usize = 10_000_000;

#[derive(Debug)]
pub enum FlatWallRitzError {
    InvalidBudget,
    UnsupportedGeometry,
    InvalidForce,
    InvalidProvenance,
    CoverageMismatch,
    ShapeMismatch,
    NonPositivePivot { index: usize },
    NotSolved,
    AlreadySolved,
    Flow(ObstacleFlowError),
    State(ObstacleStateError),
    Gradient(ObstacleGradientError),
    Viscous(ObstacleViscousError),
}
impl From<ObstacleFlowError> for FlatWallRitzError {
    fn from(e: ObstacleFlowError) -> Self {
        Self::Flow(e)
    }
}
impl From<ObstacleStateError> for FlatWallRitzError {
    fn from(e: ObstacleStateError) -> Self {
        Self::State(e)
    }
}
impl From<ObstacleGradientError> for FlatWallRitzError {
    fn from(e: ObstacleGradientError) -> Self {
        Self::Gradient(e)
    }
}
impl From<ObstacleViscousError> for FlatWallRitzError {
    fn from(e: ObstacleViscousError) -> Self {
        Self::Viscous(e)
    }
}
impl fmt::Display for FlatWallRitzError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "flat-wall Ritz refused: {self:?}")
    }
}
impl std::error::Error for FlatWallRitzError {}
type Result<T> = std::result::Result<T, FlatWallRitzError>;
fn budget(limit: usize) -> Result<()> {
    if limit == 0 || limit > FLAT_WALL_RITZ_ENVELOPE {
        return Err(FlatWallRitzError::InvalidBudget);
    }
    Ok(())
}
fn gate(bytes: usize, limit: usize) -> Result<()> {
    if bytes > limit {
        return Err(ObstacleFlowError::BufferLimit {
            required: bytes,
            limit,
        }
        .into());
    }
    Ok(())
}
fn add(a: usize, b: usize) -> Result<usize> {
    a.checked_add(b)
        .ok_or(ObstacleFlowError::CapacityOverflow.into())
}
fn bytes<T>(n: usize) -> Result<usize> {
    n.checked_mul(size_of::<T>())
        .ok_or(ObstacleFlowError::CapacityOverflow.into())
}
fn reserve<T>(n: usize) -> Result<Vec<T>> {
    let mut v = Vec::new();
    v.try_reserve_exact(n)
        .map_err(|_| ObstacleFlowError::AllocationFailure)?;
    Ok(v)
}
fn zeros(n: usize) -> Result<Vec<f64>> {
    let mut v = reserve(n)?;
    v.resize(n, 0.);
    Ok(v)
}

/// Outward-rounded arithmetic interval for exact stored f64 inputs. This is an
/// engineering error diagnostic, not a formal IEEE theorem or spatial-error bound.
/// Nonfinite endpoints/operations refuse. Subnormal interval endpoints are allowed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlatWallInterval {
    pub lower: f64,
    pub upper: f64,
}
impl FlatWallInterval {
    fn point(v: f64) -> Result<Self> {
        if !v.is_finite() {
            return Err(ObstacleFlowError::ArithmeticFailure.into());
        }
        Ok(Self { lower: v, upper: v })
    }
    fn rounded(lo: f64, hi: f64) -> Result<Self> {
        let lower = lo.next_down();
        let upper = hi.next_up();
        if !lower.is_finite() || !upper.is_finite() {
            return Err(ObstacleFlowError::ArithmeticFailure.into());
        }
        Ok(Self { lower, upper })
    }
    fn plus(self, r: Self) -> Result<Self> {
        Self::rounded(self.lower + r.lower, self.upper + r.upper)
    }
    fn minus(self, r: Self) -> Result<Self> {
        Self::rounded(self.lower - r.upper, self.upper - r.lower)
    }
    fn times(self, r: Self) -> Result<Self> {
        let v = [
            self.lower * r.lower,
            self.lower * r.upper,
            self.upper * r.lower,
            self.upper * r.upper,
        ];
        if v.iter().any(|x| !x.is_finite()) {
            return Err(ObstacleFlowError::ArithmeticFailure.into());
        }
        Self::rounded(
            v.into_iter().fold(f64::INFINITY, f64::min),
            v.into_iter().fold(f64::NEG_INFINITY, f64::max),
        )
    }
    fn divide_positive(self, d: f64) -> Result<Self> {
        positive(d)?;
        Self::rounded(self.lower / d, self.upper / d)
    }
    fn quotient(self, d: Self) -> Result<Self> {
        if d.lower <= 0. {
            return Err(ObstacleFlowError::ArithmeticFailure.into());
        }
        self.times(Self::rounded(1. / d.upper, 1. / d.lower)?)
    }
    fn power(self, n: usize) -> Result<Self> {
        let mut p = Self::point(1.)?;
        for _ in 0..n {
            p = p.times(self)?;
        }
        Ok(p)
    }
    pub fn contains(self, v: f64) -> bool {
        v >= self.lower && v <= self.upper
    }
}

/// Body-force density term: coefficient*(x-1)^px*y^py*(z-1)^pz N/m^3,
/// supported only on [1,2]x[0,1]x[1,2]. No velocity or reference load field exists.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlatWallForceTerm {
    pub component: Axis,
    pub coefficient: f64,
    pub powers: [u8; 3],
}
pub struct FlatWallPolynomialForce {
    terms: Vec<FlatWallForceTerm>,
    evidence: ObstacleEvidenceRef,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlatWallForceIntegral {
    /// Nearest analytic polynomial integral over the declared clipped dual, N.
    pub force: f64,
    pub arithmetic_interval: FlatWallInterval,
    pub sum_absolute_terms: f64,
    pub clipped_geometric_volume: f64,
    /// Existing pressure-face volume convention A*d, unchanged by source integration.
    pub stored_face_volume: f64,
}
impl FlatWallPolynomialForce {
    pub fn new(terms: Vec<FlatWallForceTerm>, evidence: ObstacleEvidenceRef) -> Result<Self> {
        if terms.len() > MAX_TERMS
            || terms.capacity() > MAX_TERMS
            || terms.iter().any(|t| {
                !(t.coefficient == 0. || t.coefficient.is_normal())
                    || t.powers.iter().any(|&p| p > MAX_DEGREE)
            })
        {
            return Err(FlatWallRitzError::InvalidForce);
        }
        Ok(Self { terms, evidence })
    }
    pub fn terms(&self) -> &[FlatWallForceTerm] {
        &self.terms
    }
    pub fn evidence(&self) -> ObstacleEvidenceRef {
        self.evidence
    }
    pub fn owned_payload_bytes(&self) -> usize {
        size_of::<Self>() + self.terms.capacity() * size_of::<FlatWallForceTerm>()
    }
    /// Analytic integration, not sampling. Actual represented centers/planes are
    /// intersected with the fixed source support; geometry and rho*A*d are untouched.
    pub fn integrate_face(
        &self,
        g: &StaticObstacleGeometry,
        axis: Axis,
        face: usize,
    ) -> Result<FlatWallForceIntegral> {
        validate_geometry(g)?;
        let grid = g.grid();
        let shape = grid.face_counts(axis);
        if face >= grid.face_len(axis) {
            return Err(FlatWallRitzError::ShapeMismatch);
        }
        let p = crate::obstacle_pressure::coordinate(shape, face);
        let a = axis.index();
        let support_lo = [1., 0., 1.];
        let support_hi = [2., 1., 2.];
        let mut lo = [0.; 3];
        let mut hi = [0.; 3];
        for d in 0..3 {
            let left = if d == a {
                center(g, d, p[d].saturating_sub(1))
            } else {
                plane(g, d, p[d])
            };
            let right = if d == a {
                center(g, d, p[d])
            } else {
                plane(g, d, p[d] + 1)
            };
            lo[d] = left.max(support_lo[d]);
            hi[d] = right.min(support_hi[d]);
        }
        let volume = if (0..3).any(|d| hi[d] <= lo[d]) {
            0.
        } else {
            mul(mul(hi[0] - lo[0], hi[1] - lo[1])?, hi[2] - lo[2])?
        };
        let stored_face_volume =
            if p[a] == 0 || p[a] == grid.counts()[a] || g.open_areas(axis)[face] == 0. {
                0.
            } else {
                mul(
                    g.open_areas(axis)[face],
                    positive(center(g, a, p[a]) - center(g, a, p[a] - 1))?,
                )?
            };
        if volume == 0. || stored_face_volume == 0. {
            return Ok(FlatWallForceIntegral {
                force: 0.,
                arithmetic_interval: FlatWallInterval::point(0.)?,
                sum_absolute_terms: 0.,
                clipped_geometric_volume: 0.,
                stored_face_volume,
            });
        }
        let shift = [1., 0., 1.];
        let mut force = Sum::default();
        let mut absolute = Sum::default();
        let mut interval = FlatWallInterval::point(0.)?;
        for t in self.terms.iter().filter(|t| t.component == axis) {
            let mut value = t.coefficient;
            let mut enclosure = FlatWallInterval::point(value)?;
            for d in 0..3 {
                let l = checked(lo[d] - shift[d])?;
                let u = checked(hi[d] - shift[d])?;
                // b^p-a^p=(b-a)*sum b^(p-1-k)a^k avoids cancellation
                // inside each nonnegative monomial integral. Term cancellation
                // is separately enclosed and counted, not hidden by tolerance.
                let n = t.powers[d] as usize;
                let mut sum = Sum::default();
                let mut isum = FlatWallInterval::point(0.)?;
                let li =
                    FlatWallInterval::point(lo[d])?.minus(FlatWallInterval::point(shift[d])?)?;
                let ui =
                    FlatWallInterval::point(hi[d])?.minus(FlatWallInterval::point(shift[d])?)?;
                for k in 0..=n {
                    let mut v = 1.;
                    for _ in 0..n - k {
                        v = mul(v, u)?;
                    }
                    for _ in 0..k {
                        v = mul(v, l)?;
                    }
                    sum.add(v)?;
                    isum = isum.plus(ui.power(n - k)?.times(li.power(k)?)?)?;
                }
                value = mul(value, div(mul(u - l, sum.finish()?)?, (n + 1) as f64)?)?;
                enclosure =
                    enclosure.times(ui.minus(li)?.times(isum)?.divide_positive((n + 1) as f64)?)?;
            }
            force.add(value)?;
            absolute.add(value.abs())?;
            interval = interval.plus(enclosure)?;
        }
        Ok(FlatWallForceIntegral {
            force: force.finish()?,
            arithmetic_interval: interval,
            sum_absolute_terms: absolute.finish()?,
            clipped_geometric_volume: volume,
            stored_face_volume,
        })
    }
}
fn plane(g: &StaticObstacleGeometry, d: usize, i: usize) -> f64 {
    g.grid().origin()[d] + i as f64 * g.grid().spacing()[d]
}
fn center(g: &StaticObstacleGeometry, d: usize, i: usize) -> f64 {
    g.grid().origin()[d] + (i as f64 + 0.5) * g.grid().spacing()[d]
}
fn validate_geometry(g: &StaticObstacleGeometry) -> Result<usize> {
    let n = g.grid().counts();
    if ![6, 9, 12].contains(&n[0])
        || n != [n[0]; 3]
        || g.grid().origin() != [0.; 3]
        || g.grid().spacing() != [3. / n[0] as f64; 3]
        || g.box_bounds() != ([1.; 3], [2.; 3])
    {
        return Err(FlatWallRitzError::UnsupportedGeometry);
    }
    crate::aligned_strain::admission(g).map_err(|_| FlatWallRitzError::UnsupportedGeometry)?;
    Ok(n[0] / 3)
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlatWallFluxTerm {
    pub component: Axis,
    pub face: usize,
    pub coefficient: f64,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlatWallFluxColumn {
    pub node: [usize; 3],
    pub terms: [FlatWallFluxTerm; 4],
}
/// q is integrated flux potential, m^3/s. Each column is four oriented face
/// flux incidences divided by actual stored area. No nominal h-curl is used.
pub struct FlatWallRitzPlan<'g> {
    geometry: &'g StaticObstacleGeometry,
    columns: Vec<FlatWallFluxColumn>,
    sites: Vec<ObstacleGradientSite>,
    m: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlatWallCoverage {
    pub selected_rows: usize,
    pub selected_blocks: usize,
    pub zero_corner_directed_sectors: usize,
    pub outer_even_reflection: bool,
}
impl<'g> FlatWallRitzPlan<'g> {
    pub fn new(geometry: &'g StaticObstacleGeometry, limit: usize) -> Result<Self> {
        budget(limit)?;
        let m = validate_geometry(geometry)?;
        let q = (m - 1) * (m - 1) * m;
        let rows = 27 * m * m * m + 4 * m * m - 28 * m;
        gate(
            add(
                geometry.allocation().retained_bytes,
                add(
                    size_of::<Self>(),
                    add(
                        bytes::<FlatWallFluxColumn>(q)?,
                        bytes::<ObstacleGradientSite>(rows)?,
                    )?,
                )?,
            )?,
            limit,
        )?;
        let mut columns = reserve(q)?;
        for z in m..2 * m {
            for y in 1..m {
                for x in m + 1..2 * m {
                    let node = [x, y, z];
                    let mut points = [node; 4];
                    points[0][1] -= 1;
                    points[2][0] -= 1;
                    let axes = [Axis::X, Axis::X, Axis::Y, Axis::Y];
                    let signs = [1., -1., -1., 1.];
                    let mut terms = [FlatWallFluxTerm {
                        component: Axis::X,
                        face: 0,
                        coefficient: 0.,
                    }; 4];
                    for j in 0..4 {
                        let face = geometry
                            .grid()
                            .face_index(axes[j], points[j])
                            .ok_or(FlatWallRitzError::CoverageMismatch)?;
                        terms[j] = FlatWallFluxTerm {
                            component: axes[j],
                            face,
                            coefficient: div(
                                signs[j],
                                positive(geometry.open_areas(axes[j])[face])?,
                            )?,
                        };
                    }
                    columns.push(FlatWallFluxColumn { node, terms });
                }
            }
        }
        let mut sites = reserve(rows)?;
        for z in m..2 * m {
            for y in 0..m {
                for x in m..2 * m {
                    for axis in Axis::ALL {
                        sites.push(ObstacleGradientSite::Normal {
                            axis,
                            cell: [x, y, z],
                        });
                    }
                }
            }
        }
        fn pair(
            sites: &mut Vec<ObstacleGradientSite>,
            a: Axis,
            b: Axis,
            edge: [usize; 3],
            quadrant: u8,
        ) {
            sites.push(ObstacleGradientSite::Cross {
                component: a,
                derivative: b,
                edge,
                quadrant,
            });
            sites.push(ObstacleGradientSite::Cross {
                component: b,
                derivative: a,
                edge,
                quadrant,
            });
        }
        for z in m..2 * m {
            for y in 1..m {
                for x in m..=2 * m {
                    for q in 0..4 {
                        pair(&mut sites, Axis::X, Axis::Y, [x, y, z], q);
                    }
                }
            }
        }
        for z in m..2 * m {
            for x in m + 1..2 * m {
                for q in 0..2 {
                    pair(&mut sites, Axis::X, Axis::Y, [x, m, z], q);
                }
            }
        }
        for z in m..=2 * m {
            for y in 0..m {
                for x in m + 1..2 * m {
                    for q in 0..4 {
                        pair(&mut sites, Axis::X, Axis::Z, [x, y, z], q);
                    }
                }
            }
        }
        for z in m..=2 * m {
            for y in 1..m {
                for x in m..2 * m {
                    for q in 0..4 {
                        pair(&mut sites, Axis::Y, Axis::Z, [x, y, z], q);
                    }
                }
            }
        }
        if sites.len() != rows || columns.len() != q {
            return Err(FlatWallRitzError::CoverageMismatch);
        }
        let p = Self {
            geometry,
            columns,
            sites,
            m,
        };
        gate(
            add(
                geometry.allocation().retained_bytes,
                p.owned_payload_bytes(),
            )?,
            limit,
        )?;
        Ok(p)
    }
    pub fn geometry(&self) -> &'g StaticObstacleGeometry {
        self.geometry
    }
    pub fn columns(&self) -> &[FlatWallFluxColumn] {
        &self.columns
    }
    pub fn sites(&self) -> &[ObstacleGradientSite] {
        &self.sites
    }
    pub fn expected_blocks(&self) -> usize {
        (self.sites.len() + 3 * self.m * self.m * self.m) / 2
    }
    pub fn owned_payload_bytes(&self) -> usize {
        size_of::<Self>()
            + self.columns.capacity() * size_of::<FlatWallFluxColumn>()
            + self.sites.capacity() * size_of::<ObstacleGradientSite>()
    }
    fn coefficient(&self, col: usize, component: Axis, face: usize) -> f64 {
        self.columns[col]
            .terms
            .iter()
            .find(|t| t.component == component && t.face == face)
            .map_or(0., |t| t.coefficient)
    }
    fn row_column(&self, row: &ObstacleGradientRow, col: usize) -> Result<f64> {
        let mut sum = Sum::default();
        for e in row.endpoints {
            if let ObstacleGradientSource::VelocityFace { face } = e.source {
                sum.add(mul(
                    e.coefficient,
                    self.coefficient(col, row.component, face),
                )?)?;
            }
        }
        Ok(sum.finish()?)
    }
    /// Caller scratch only; zeros inactive faces and retains all actual C rounding.
    pub fn apply_flux_curl(&self, q: &[f64], out: [&mut [f64]; 3]) -> Result<()> {
        if q.len() != self.columns.len()
            || (0..3).any(|d| out[d].len() != self.geometry.grid().face_len(Axis::ALL[d]))
        {
            return Err(FlatWallRitzError::ShapeMismatch);
        }
        for &v in q {
            checked(v)?;
        }
        let [x, y, z] = out;
        let mut out = [x, y, z];
        for v in &mut out {
            v.fill(0.);
        }
        for (column, &value) in self.columns.iter().zip(q) {
            for t in column.terms {
                let target = &mut out[t.component.index()][t.face];
                *target = checked(*target + mul(t.coefficient, value)?)?;
            }
        }
        Ok(())
    }
    /// Every global interior sector is inspected for every basis. Corners are
    /// admitted only when ALL four surrounding component-face samples are absent
    /// from C; no unsupported nonzero corner is silently dropped. Outer shears
    /// use sealed-normal/free-slip even continuation, a declared boundary model.
    pub fn verify_coverage(
        &self,
        state: &ObstacleFlowState<'_>,
        mut cancel: impl FnMut(ObstacleFlowStage, usize) -> bool,
    ) -> Result<FlatWallCoverage> {
        if !std::ptr::eq(state.geometry(), self.geometry) {
            return Err(FlatWallRitzError::UnsupportedGeometry);
        }
        let n = self.geometry.grid().counts();
        let mut count = 0;
        let mut zero_corner = 0;
        let check = |row: ObstacleGradientRow| -> Result<()> {
            let mut nonzero = false;
            for col in 0..self.columns.len() {
                if self.row_column(&row, col)? != 0. {
                    nonzero = true;
                }
            }
            if nonzero && !self.sites.contains(&row.site) {
                return Err(FlatWallRitzError::CoverageMismatch);
            }
            Ok(())
        };
        for z in 0..n[2] {
            for y in 0..n[1] {
                for x in 0..n[0] {
                    if self.geometry.fluid_volumes()
                        [self.geometry.grid().cell_index([x, y, z]).unwrap()]
                        == 0.
                    {
                        continue;
                    }
                    for axis in Axis::ALL {
                        checkpoint(&mut cancel, ObstacleFlowStage::Assembly, count)?;
                        count += 1;
                        check(crate::obstacle_gradient::make_row(
                            state,
                            ObstacleGradientSite::Normal {
                                axis,
                                cell: [x, y, z],
                            },
                        )?)?;
                    }
                }
            }
        }
        for a in 0..3 {
            for b in a + 1..3 {
                let c = 3 - a - b;
                for k in 0..n[c] {
                    for j in 1..n[b] {
                        for i in 1..n[a] {
                            let mut edge = [0; 3];
                            edge[a] = i;
                            edge[b] = j;
                            edge[c] = k;
                            for quadrant in 0..4 {
                                for (component, derivative) in
                                    [(Axis::ALL[a], Axis::ALL[b]), (Axis::ALL[b], Axis::ALL[a])]
                                {
                                    checkpoint(&mut cancel, ObstacleFlowStage::Assembly, count)?;
                                    count += 1;
                                    let site = ObstacleGradientSite::Cross {
                                        component,
                                        derivative,
                                        edge,
                                        quadrant,
                                    };
                                    match crate::obstacle_gradient::make_row(state, site) {
                                        Ok(row) => check(row)?,
                                        Err(ObstacleGradientError::UnsupportedCorner) => {
                                            for axis in [Axis::ALL[a], Axis::ALL[b]] {
                                                for side in 0..2 {
                                                    let mut p = edge;
                                                    if side == 0 {
                                                        p[if axis.index() == a { b } else { a }] -=
                                                            1;
                                                    }
                                                    let face = self
                                                        .geometry
                                                        .grid()
                                                        .face_index(axis, p)
                                                        .ok_or(
                                                            FlatWallRitzError::CoverageMismatch,
                                                        )?;
                                                    if (0..self.columns.len()).any(|col| {
                                                        self.coefficient(col, axis, face) != 0.
                                                    }) {
                                                        return Err(
                                                            FlatWallRitzError::CoverageMismatch,
                                                        );
                                                    }
                                                }
                                            }
                                            zero_corner += 1;
                                        }
                                        Err(ObstacleGradientError::InvalidSite) => {}
                                        Err(e) => return Err(e.into()),
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(FlatWallCoverage {
            selected_rows: self.sites.len(),
            selected_blocks: self.expected_blocks(),
            zero_corner_directed_sectors: zero_corner,
            outer_even_reflection: true,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FlatWallRitzAllocation {
    /// Geometry once plus provider, source, requests, C and ALL owned Vec capacities.
    pub retained_managed_bytes: usize,
    /// Includes rest owner, gradient, stress blocks and temporary sort indices.
    pub assembly_peak_managed_bytes: usize,
    /// Same capacity accounting for a copied acquired owner and immutable actions.
    pub acquisition_peak_managed_bytes: usize,
    pub limit: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FlatWallRitzSolveReport {
    pub projected_residual_max: f64,
    pub minimum_pivot: f64,
    pub integrated_divergence_max: f64,
    /// Outward arithmetic bound for integrated D on the actual rounded Cq values.
    pub integrated_divergence_arithmetic_bound: f64,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlatWallRitzOwnedDiagnostics {
    pub work: ObstacleViscousWork,
    /// ||b_force + F_viscous||_inf on every stored active face, N.
    /// This is neither a projected residual nor a strong continuum PDE residual.
    pub full_active_face_residual_max: f64,
    pub projected_action_residual_max: f64,
    pub matrix_action_discrepancy_max: f64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    Assembled,
    SolveStarted,
    Solved,
}
/// Owns bounded assembly/solve work. Its explicit opt-in solve method is NOT a
/// timestep, pressure solve or production capability. No physical solve occurs
/// in construction; the three physical source campaigns remain separately held.
pub struct FlatWallRitzProvider<'g> {
    plan: FlatWallRitzPlan<'g>,
    source: FlatWallPolynomialForce,
    inputs: ObstaclePhysicalInputs,
    matrix: Vec<f64>,
    factor: Vec<f64>,
    rhs: Vec<f64>,
    q: Vec<f64>,
    velocity: [Vec<f64>; 3],
    body_force: [Vec<f64>; 3],
    action_force: [Vec<f64>; 3],
    stress: Vec<f64>,
    allocation: FlatWallRitzAllocation,
    coverage: FlatWallCoverage,
    source_interval_width_max: f64,
    source_absolute_term_sum_max: f64,
    status: Status,
    limit: usize,
}
fn frame() -> ObstacleStateFrame {
    ObstacleStateFrame {
        time: 0.,
        generation: 0,
        origin: ObstacleStateOrigin::InitialData,
        errors: ObstacleVelocityErrors::unknown(),
    }
}
fn parent_peak(state_bytes: usize, rows: usize, blocks: usize) -> Result<usize> {
    let gradient = add(
        size_of::<ObstacleVelocityGradient<'_, '_>>(),
        bytes::<ObstacleGradientRow>(rows)?,
    )?;
    let stress = add(
        size_of::<ObstacleViscousStress<'_, '_, '_>>(),
        bytes::<ObstacleViscousBlock>(blocks)?,
    )?;
    add(
        add(add(state_bytes, gradient)?, stress)?,
        bytes::<usize>(rows)?,
    )
}
fn pack(g: &StaticObstacleGeometry) -> Result<[Vec<f64>; 3]> {
    Ok([
        zeros(g.grid().face_len(Axis::X))?,
        zeros(g.grid().face_len(Axis::Y))?,
        zeros(g.grid().face_len(Axis::Z))?,
    ])
}
impl<'g> FlatWallRitzProvider<'g> {
    pub fn new(
        geometry: &'g StaticObstacleGeometry,
        source: FlatWallPolynomialForce,
        inputs: ObstaclePhysicalInputs,
        limit: usize,
        mut cancel: impl FnMut(ObstacleFlowStage, usize) -> bool,
    ) -> Result<Self> {
        budget(limit)?;
        if inputs.model != ObstacleStateModel::TransientStokes
            || inputs.material.density != 1.
            || inputs.material.dynamic_viscosity != 1.
            || inputs.initial_time != 0.
            || inputs.forcing.kind != ObstacleForcingKind::DeclaredExternalForcing
            || inputs.forcing.evidence != source.evidence()
            || inputs.forcing.start_time != 0.
            || inputs.forcing.end_time != 0.
            || !matches!(inputs.initial, ObstacleInitialData::KnownRest(_))
        {
            return Err(FlatWallRitzError::InvalidProvenance);
        }
        let plan = FlatWallRitzPlan::new(geometry, limit)?;
        let nq = plan.columns.len();
        let nr = plan.sites.len();
        let face_count = Axis::ALL
            .into_iter()
            .map(|a| geometry.grid().face_len(a))
            .sum::<usize>();
        let dynamic = add(
            plan.owned_payload_bytes() - size_of::<FlatWallRitzPlan<'_>>(),
            source.owned_payload_bytes() - size_of::<FlatWallPolynomialForce>(),
        )?;
        let vector_elements = 3 * face_count + nr + 2 * nq * nq + 2 * nq;
        let predicted_provider = add(
            add(size_of::<Self>(), dynamic)?,
            bytes::<f64>(vector_elements)?,
        )?;
        let state_combined = ObstacleFlowState::planned_payload_bytes(geometry)?;
        gate(
            add(
                predicted_provider,
                parent_peak(state_combined, nr, plan.expected_blocks())?,
            )?,
            limit,
        )?;
        checkpoint(&mut cancel, ObstacleFlowStage::Assembly, 0)?;
        let mut p = Self {
            plan,
            source,
            inputs,
            matrix: zeros(nq * nq)?,
            factor: zeros(nq * nq)?,
            rhs: zeros(nq)?,
            q: zeros(nq)?,
            velocity: pack(geometry)?,
            body_force: pack(geometry)?,
            action_force: pack(geometry)?,
            stress: zeros(nr)?,
            allocation: FlatWallRitzAllocation::default(),
            coverage: FlatWallCoverage {
                selected_rows: nr,
                selected_blocks: 0,
                zero_corner_directed_sectors: 0,
                outer_even_reflection: false,
            },
            source_interval_width_max: 0.,
            source_absolute_term_sum_max: 0.,
            status: Status::Assembled,
            limit,
        };
        let provider = p.owned_payload_bytes();
        let predicted = add(
            provider,
            parent_peak(state_combined, nr, p.plan.expected_blocks())?,
        )?;
        gate(predicted, limit)?;
        // The only rest owner is local to this block. Its gradient/stress are
        // dropped before acquisition, never retained once per trial basis.
        {
            let state = ObstacleFlowState::new(
                geometry,
                inputs,
                frame(),
                [&p.velocity[0], &p.velocity[1], &p.velocity[2]],
                limit - provider,
            )?;
            p.coverage = p.plan.verify_coverage(&state, &mut cancel)?;
            let gradient = ObstacleVelocityGradient::new(
                &state,
                p.plan.sites(),
                limit - provider,
                &mut cancel,
            )?;
            let op = ObstacleViscousStress::new(&gradient, limit - provider, &mut cancel)?;
            if op.blocks().len() != p.plan.expected_blocks() {
                return Err(FlatWallRitzError::CoverageMismatch);
            }
            let actual_peak = add(provider, op.constructor_peak_payload_bytes())?;
            gate(actual_peak, limit)?;
            p.allocation = FlatWallRitzAllocation {
                retained_managed_bytes: add(provider, geometry.allocation().retained_bytes)?,
                assembly_peak_managed_bytes: actual_peak,
                acquisition_peak_managed_bytes: actual_peak,
                limit,
            };
            let mut g = [0.; MAX_Q];
            for (step, block) in op.blocks().iter().enumerate() {
                checkpoint(&mut cancel, ObstacleFlowStage::Assembly, step)?;
                let weight = match *block {
                    ObstacleViscousBlock::Normal { row } => {
                        for (j, v) in g.iter_mut().enumerate().take(nq) {
                            *v = p.plan.row_column(&gradient.rows()[row], j)?;
                        }
                        mul(
                            2.,
                            mul(
                                inputs.material.dynamic_viscosity,
                                gradient.rows()[row].weight,
                            )?,
                        )?
                    }
                    ObstacleViscousBlock::Shear { first, second } => {
                        for (j, v) in g.iter_mut().enumerate().take(nq) {
                            *v = checked(
                                p.plan.row_column(&gradient.rows()[first], j)?
                                    + p.plan.row_column(&gradient.rows()[second], j)?,
                            )?;
                        }
                        mul(
                            inputs.material.dynamic_viscosity,
                            gradient.rows()[first].weight,
                        )?
                    }
                };
                for i in 0..nq {
                    for j in 0..nq {
                        let at = i * nq + j;
                        p.matrix[at] = checked(p.matrix[at] + mul(weight, mul(g[i], g[j])?)?)?;
                    }
                }
            }
        }
        for a in Axis::ALL {
            for face in 0..geometry.grid().face_len(a) {
                checkpoint(&mut cancel, ObstacleFlowStage::Assembly, face)?;
                let integral = p.source.integrate_face(geometry, a, face)?;
                p.body_force[a.index()][face] = integral.force;
                p.source_interval_width_max = p.source_interval_width_max.max(checked(
                    integral.arithmetic_interval.upper - integral.arithmetic_interval.lower,
                )?);
                p.source_absolute_term_sum_max = p
                    .source_absolute_term_sum_max
                    .max(integral.sum_absolute_terms);
            }
        }
        for (i, column) in p.plan.columns.iter().enumerate() {
            let mut sum = Sum::default();
            for t in column.terms {
                sum.add(mul(
                    t.coefficient,
                    p.body_force[t.component.index()][t.face],
                )?)?;
            }
            p.rhs[i] = sum.finish()?;
        }
        Ok(p)
    }
    fn owned_payload_bytes(&self) -> usize {
        size_of::<Self>() + self.plan.owned_payload_bytes() - size_of::<FlatWallRitzPlan<'_>>()
            + self.source.owned_payload_bytes()
            - size_of::<FlatWallPolynomialForce>()
            + [&self.matrix, &self.factor, &self.rhs, &self.q, &self.stress]
                .into_iter()
                .map(|v| v.capacity() * 8)
                .sum::<usize>()
            + [&self.velocity, &self.body_force, &self.action_force]
                .into_iter()
                .flat_map(|p| p.iter())
                .map(|v| v.capacity() * 8)
                .sum::<usize>()
    }
    pub fn plan(&self) -> &FlatWallRitzPlan<'g> {
        &self.plan
    }
    pub fn source(&self) -> &FlatWallPolynomialForce {
        &self.source
    }
    pub fn matrix(&self) -> &[f64] {
        &self.matrix
    }
    pub fn rhs(&self) -> &[f64] {
        &self.rhs
    }
    pub fn body_force(&self) -> [&[f64]; 3] {
        [
            &self.body_force[0],
            &self.body_force[1],
            &self.body_force[2],
        ]
    }
    pub fn allocation(&self) -> FlatWallRitzAllocation {
        self.allocation
    }
    pub fn coverage(&self) -> FlatWallCoverage {
        self.coverage
    }
    pub fn source_arithmetic_width_max(&self) -> f64 {
        self.source_interval_width_max
    }
    pub fn source_absolute_term_sum_max(&self) -> f64 {
        self.source_absolute_term_sum_max
    }
    /// Exactly one primal LDL^T attempt, no regularization/symmetrization/retries.
    /// Calling this method is a steady numerical experiment and needs separate
    /// campaign authorization; source/unit qualification never calls it for the
    /// physical manufactured source on the N6/N9/N12 roster.
    pub fn solve(
        &mut self,
        mut cancel: impl FnMut(ObstacleFlowStage, usize) -> bool,
    ) -> Result<FlatWallRitzSolveReport> {
        if self.status != Status::Assembled {
            return Err(FlatWallRitzError::AlreadySolved);
        }
        self.status = Status::SolveStarted;
        let pivot = dense_ldlt(
            &self.matrix,
            &self.rhs,
            &mut self.factor,
            &mut self.q,
            &mut cancel,
        )?;
        let [x, y, z] = &mut self.velocity;
        self.plan.apply_flux_curl(&self.q, [x, y, z])?;
        let nq = self.q.len();
        let mut residual: f64 = 0.;
        for i in 0..nq {
            let mut sum = Sum::default();
            for j in 0..nq {
                sum.add(mul(self.matrix[i * nq + j], self.q[j])?)?;
            }
            residual = residual.max(checked(sum.finish()? - self.rhs[i])?.abs());
        }
        let divergence = divergence_report(
            self.plan.geometry(),
            [&self.velocity[0], &self.velocity[1], &self.velocity[2]],
        )?;
        self.status = Status::Solved;
        Ok(FlatWallRitzSolveReport {
            projected_residual_max: residual,
            minimum_pivot: pivot,
            integrated_divergence_max: divergence.0,
            integrated_divergence_arithmetic_bound: divergence.1,
        })
    }
    /// Available only after the numerical solve, for source-bound journal hashing.
    pub fn solved_coefficients(&self) -> Result<&[f64]> {
        if self.status != Status::Solved {
            return Err(FlatWallRitzError::NotSolved);
        }
        Ok(&self.q)
    }
    pub fn solved_velocity(&self) -> Result<[&[f64]; 3]> {
        if self.status != Status::Solved {
            return Err(FlatWallRitzError::NotSolved);
        }
        Ok([&self.velocity[0], &self.velocity[1], &self.velocity[2]])
    }
    /// Caller first hashes the actual numerical acquisition journal/field, then
    /// supplies that reference. The reference is declared, not authenticated here.
    /// Stored as supplied initial data acquired from this reduced solve, NEVER as
    /// evolved state/full Stokes. All errors Unknown and pressure unavailable.
    pub fn acquire_initial_state(
        &self,
        acquisition: ObstacleEvidenceRef,
    ) -> Result<ObstacleFlowState<'g>> {
        let v = self.solved_velocity()?;
        let mut inputs = self.inputs;
        inputs.initial = ObstacleInitialData::Supplied(acquisition);
        Ok(ObstacleFlowState::new(
            self.plan.geometry(),
            inputs,
            frame(),
            v,
            self.limit - self.owned_payload_bytes(),
        )?)
    }
    /// Actions use the actual immutable acquired owner. A different geometry,
    /// source/material/frame or changed velocity refuses; no raw gather escape.
    pub fn diagnose_owned(
        &mut self,
        state: &ObstacleFlowState<'g>,
        mut cancel: impl FnMut(ObstacleFlowStage, usize) -> bool,
    ) -> Result<FlatWallRitzOwnedDiagnostics> {
        let solved = self.solved_velocity()?;
        if !std::ptr::eq(state.geometry(), self.plan.geometry())
            || state.velocity() != solved
            || state.physical_inputs().forcing.evidence != self.source.evidence()
            || state.physical_inputs().material != self.inputs.material
            || state.frame() != &frame()
            || !matches!(
                state.physical_inputs().initial,
                ObstacleInitialData::Supplied(_)
            )
        {
            return Err(FlatWallRitzError::InvalidProvenance);
        }
        let provider = self.owned_payload_bytes();
        let peak = add(
            provider,
            parent_peak(
                state.combined_payload_bytes(),
                self.plan.sites.len(),
                self.plan.expected_blocks(),
            )?,
        )?;
        gate(peak, self.limit)?;
        let gradient = ObstacleVelocityGradient::new(
            state,
            self.plan.sites(),
            self.limit - provider,
            &mut cancel,
        )?;
        let op = ObstacleViscousStress::new(&gradient, self.limit - provider, &mut cancel)?;
        gate(
            add(provider, op.constructor_peak_payload_bytes())?,
            self.limit,
        )?;
        let [x, y, z] = &mut self.action_force;
        let work = op.diagnose(&mut self.stress, [x, y, z], &mut cancel)?;
        let mut full: f64 = 0.;
        for a in 0..3 {
            for (&f, &b) in self.action_force[a].iter().zip(&self.body_force[a]) {
                full = full.max(checked(f + b)?.abs());
            }
        }
        let mut projected: f64 = 0.;
        let mut discrepancy: f64 = 0.;
        let nq = self.q.len();
        for (i, col) in self.plan.columns.iter().enumerate() {
            let mut action = Sum::default();
            for t in col.terms {
                action.add(mul(
                    t.coefficient,
                    self.action_force[t.component.index()][t.face],
                )?)?;
            }
            let action = action.finish()?;
            projected = projected.max(checked(action + self.rhs[i])?.abs());
            let mut matrix = Sum::default();
            for j in 0..nq {
                matrix.add(mul(self.matrix[i * nq + j], self.q[j])?)?;
            }
            discrepancy = discrepancy.max(checked(matrix.finish()? + action)?.abs());
        }
        Ok(FlatWallRitzOwnedDiagnostics {
            work,
            full_active_face_residual_max: full,
            projected_action_residual_max: projected,
            matrix_action_discrepancy_max: discrepancy,
        })
    }
}
/// Bounded algebraic kernel. Independent unit fixtures exercise this kernel on
/// unrelated small synthetic matrices; they do not solve the physical source.
fn dense_ldlt(
    matrix: &[f64],
    rhs: &[f64],
    factor: &mut [f64],
    q: &mut [f64],
    cancel: &mut impl FnMut(ObstacleFlowStage, usize) -> bool,
) -> Result<f64> {
    let n = rhs.len();
    if n == 0 || n > MAX_Q || matrix.len() != n * n || factor.len() != n * n || q.len() != n {
        return Err(FlatWallRitzError::ShapeMismatch);
    }
    for i in 0..n {
        checked(rhs[i])?;
        for j in 0..n {
            checked(matrix[i * n + j])?;
            if matrix[i * n + j] != matrix[j * n + i] {
                return Err(FlatWallRitzError::CoverageMismatch);
            }
        }
    }
    factor.fill(0.);
    let mut minimum = f64::INFINITY;
    for i in 0..n {
        checkpoint(cancel, ObstacleFlowStage::Correction, i)?;
        for j in 0..=i {
            let mut s = Sum::default();
            s.add(matrix[i * n + j])?;
            for k in 0..j {
                s.add(-mul(
                    mul(factor[i * n + k], factor[j * n + k])?,
                    factor[k * n + k],
                )?)?;
            }
            let value = s.finish()?;
            if i == j {
                if value <= 0. {
                    return Err(FlatWallRitzError::NonPositivePivot { index: i });
                }
                factor[i * n + i] = positive(value)?;
                minimum = minimum.min(value);
            } else {
                factor[i * n + j] = div(value, factor[j * n + j])?;
            }
        }
    }
    for i in 0..n {
        checkpoint(cancel, ObstacleFlowStage::Correction, n + i)?;
        let mut s = Sum::default();
        s.add(rhs[i])?;
        for j in 0..i {
            s.add(-mul(factor[i * n + j], q[j])?)?;
        }
        q[i] = s.finish()?;
    }
    for i in 0..n {
        q[i] = div(q[i], factor[i * n + i])?;
    }
    for i in (0..n).rev() {
        checkpoint(cancel, ObstacleFlowStage::Correction, 2 * n + i)?;
        let mut s = Sum::default();
        s.add(q[i])?;
        for j in i + 1..n {
            s.add(-mul(factor[j * n + i], q[j])?)?;
        }
        q[i] = s.finish()?;
    }
    Ok(minimum)
}
fn divergence_report(g: &StaticObstacleGeometry, v: [&[f64]; 3]) -> Result<(f64, f64)> {
    if (0..3).any(|a| v[a].len() != g.grid().face_len(Axis::ALL[a])) {
        return Err(FlatWallRitzError::ShapeMismatch);
    }
    let n = g.grid().counts();
    let mut observed: f64 = 0.;
    let mut enclosure: f64 = 0.;
    for z in 0..n[2] {
        for y in 0..n[1] {
            for x in 0..n[0] {
                let cell = [x, y, z];
                if g.fluid_volumes()[g.grid().cell_index(cell).unwrap()] == 0. {
                    continue;
                }
                let mut s = Sum::default();
                let mut r = FlatWallInterval::point(0.)?;
                for a in Axis::ALL {
                    let mut upper = cell;
                    upper[a.index()] += 1;
                    for (p, sign) in [(cell, -1.), (upper, 1.)] {
                        let face = g.grid().face_index(a, p).unwrap();
                        let area = g.open_areas(a)[face];
                        let value = v[a.index()][face];
                        checked(value)?;
                        s.add(mul(sign, mul(area, value)?)?)?;
                        r = r.plus(
                            FlatWallInterval::point(sign)?.times(
                                FlatWallInterval::point(area)?
                                    .times(FlatWallInterval::point(value)?)?,
                            )?,
                        )?;
                    }
                }
                observed = observed.max(s.finish()?.abs());
                enclosure = enclosure.max(r.lower.abs().max(r.upper.abs()));
            }
        }
    }
    Ok((observed, enclosure))
}
/// Geometry-weighted integrated divergence of actual rounded caller values,
/// with arithmetic-only enclosure. This never authorizes projection/stepping.
pub fn flat_wall_divergence(g: &StaticObstacleGeometry, v: [&[f64]; 3]) -> Result<(f64, f64)> {
    validate_geometry(g)?;
    divergence_report(g, v)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlatWallNormalTraction {
    FirstRowP1,
    TwoPlaneP2,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlatWallOwnedWrench {
    pub force: [f64; 3],
    pub torque: [f64; 3],
    /// Arithmetic only for traction from these actual stored velocity values.
    pub arithmetic_force: [FlatWallInterval; 3],
    pub arithmetic_torque: [FlatWallInterval; 3],
    pub geometric_wall_area: f64,
    /// Active interior P1 hat area; endpoint hats have exactly zero traces.
    pub tangential_basis_area: f64,
    /// Sum of included flat sector volumes divided by their actual wall delta.
    pub energy_sector_effective_area: f64,
}
fn validate_trial_support(state: &ObstacleFlowState<'_>, m: usize) -> Result<()> {
    let g = state.geometry();
    for a in Axis::ALL {
        for (face, &v) in state.velocity()[a.index()].iter().enumerate() {
            let p = crate::obstacle_pressure::coordinate(g.grid().face_counts(a), face);
            let in_patch = match a {
                Axis::X => p[0] > m && p[0] < 2 * m && p[1] < m && p[2] >= m && p[2] < 2 * m,
                Axis::Y => {
                    p[0] >= m && p[0] < 2 * m && p[1] > 0 && p[1] < m && p[2] >= m && p[2] < 2 * m
                }
                Axis::Z => false,
            };
            if !in_patch && v != 0. {
                return Err(FlatWallRitzError::CoverageMismatch);
            }
        }
    }
    Ok(())
}
/// Physical fluid-on-solid viscous surface wrench about (1.5,1.5,1.5), lower-Y
/// face only. Uses immutable OWNED samples, actual distances and exact physical
/// P1 hat masses/first moments (nearest evaluated + outward arithmetic intervals).
/// Endpoint/corner/other-face zeros require complete trial-support validation.
/// This is not the generalized transpose wrench and does not change energy rows.
pub fn flat_wall_owned_traction(
    state: &ObstacleFlowState<'_>,
    normal: FlatWallNormalTraction,
    mut cancel: impl FnMut(ObstacleFlowStage, usize) -> bool,
) -> Result<FlatWallOwnedWrench> {
    let g = state.geometry();
    let m = validate_geometry(g)?;
    validate_trial_support(state, m)?;
    let mu = state.physical_inputs().material.dynamic_viscosity;
    let wall = plane(g, 1, m);
    let reference = [1.5; 3];
    let zero = FlatWallInterval::point(0.)?;
    let mut force = std::array::from_fn::<_, 3, _>(|_| Sum::default());
    let mut torque = std::array::from_fn::<_, 3, _>(|_| Sum::default());
    let mut fi = [zero; 3];
    let mut ti = [zero; 3];
    let mut basis_area = Sum::default();
    let mut effective_area = Sum::default();
    let mut step = 0;
    let mut accumulate = |component: usize,
                          traction: f64,
                          t: FlatWallInterval,
                          area: f64,
                          ai: FlatWallInterval,
                          position: [f64; 3],
                          pi: [FlatWallInterval; 3]|
     -> Result<()> {
        let contribution = mul(traction, area)?;
        force[component].add(contribution)?;
        let f = t.times(ai)?;
        fi[component] = fi[component].plus(f)?;
        let r = [
            position[0] - reference[0],
            position[1] - reference[1],
            position[2] - reference[2],
        ];
        let ri = [
            pi[0].minus(FlatWallInterval::point(reference[0])?)?,
            pi[1].minus(FlatWallInterval::point(reference[1])?)?,
            pi[2].minus(FlatWallInterval::point(reference[2])?)?,
        ];
        let terms = match component {
            0 => [(1, 2, 1.), (2, 1, -1.)],
            1 => [(0, 2, -1.), (2, 0, 1.)],
            _ => [(0, 1, 1.), (1, 0, -1.)],
        };
        for (a, d, sign) in terms {
            torque[a].add(mul(sign, mul(r[d], contribution)?)?)?;
            ti[a] = ti[a].plus(FlatWallInterval::point(sign)?.times(ri[d].times(f)?)?)?;
        }
        Ok(())
    };
    // x shear P1 hat basis; exact hat centroid, not nominal node/midpoint.
    for z in m..2 * m {
        for x in m + 1..2 * m {
            checkpoint(&mut cancel, ObstacleFlowStage::Correction, step)?;
            step += 1;
            let face = g.grid().face_index(Axis::X, [x, m - 1, z]).unwrap();
            let value = state.velocity()[0][face];
            let delta = positive(wall - center(g, 1, m - 1))?;
            let tau = div(mul(mu, value)?, delta)?;
            let taui = FlatWallInterval::point(mu)?
                .times(FlatWallInterval::point(value)?)?
                .quotient(
                    FlatWallInterval::point(wall)?.minus(FlatWallInterval::point(center(
                        g,
                        1,
                        m - 1,
                    ))?)?,
                )?;
            let hl = positive(plane(g, 0, x) - plane(g, 0, x - 1))?;
            let hr = positive(plane(g, 0, x + 1) - plane(g, 0, x))?;
            let hli = FlatWallInterval::point(plane(g, 0, x))?
                .minus(FlatWallInterval::point(plane(g, 0, x - 1))?)?;
            let hri = FlatWallInterval::point(plane(g, 0, x + 1))?
                .minus(FlatWallInterval::point(plane(g, 0, x))?)?;
            let hat = mul(0.5, checked(hl + hr)?)?;
            let hati = hli.plus(hri)?.times(FlatWallInterval::point(0.5)?)?;
            let dz = positive(plane(g, 2, z + 1) - plane(g, 2, z))?;
            let dzi = FlatWallInterval::point(plane(g, 2, z + 1))?
                .minus(FlatWallInterval::point(plane(g, 2, z))?)?;
            let area = mul(hat, dz)?;
            basis_area.add(area)?;
            let cx = checked(plane(g, 0, x) + div(hr - hl, 3.)?)?;
            let cxi = FlatWallInterval::point(plane(g, 0, x))?
                .plus(hri.minus(hli)?.divide_positive(3.)?)?;
            let cz = mul(0.5, checked(plane(g, 2, z) + plane(g, 2, z + 1))?)?;
            let czi = FlatWallInterval::point(plane(g, 2, z))?
                .plus(FlatWallInterval::point(plane(g, 2, z + 1))?)?
                .times(FlatWallInterval::point(0.5)?)?;
            accumulate(
                0,
                tau,
                taui,
                area,
                hati.times(dzi)?,
                [cx, wall, cz],
                [cxi, FlatWallInterval::point(wall)?, czi],
            )?;
            for quadrant in 0..2 {
                let row = crate::obstacle_gradient::make_row(
                    state,
                    ObstacleGradientSite::Cross {
                        component: Axis::X,
                        derivative: Axis::Y,
                        edge: [x, m, z],
                        quadrant,
                    },
                )?;
                effective_area.add(div(row.weight, delta)?)?;
            }
        }
    }
    // normal traction: ALL cell areas/first moments, never suppressed using p*=0.
    for z in m..2 * m {
        for x in m..2 * m {
            checkpoint(&mut cancel, ObstacleFlowStage::Correction, step)?;
            step += 1;
            let f1 = g.grid().face_index(Axis::Y, [x, m - 1, z]).unwrap();
            let v1 = state.velocity()[1][f1];
            let d1 = positive(wall - plane(g, 1, m - 1))?;
            let d1i = FlatWallInterval::point(wall)?.minus(FlatWallInterval::point(plane(
                g,
                1,
                m - 1,
            ))?)?;
            let (derivative, di) = match normal {
                FlatWallNormalTraction::FirstRowP1 => {
                    (div(-v1, d1)?, FlatWallInterval::point(-v1)?.quotient(d1i)?)
                }
                FlatWallNormalTraction::TwoPlaneP2 => {
                    let f2 = g.grid().face_index(Axis::Y, [x, m - 2, z]).unwrap();
                    let v2 = state.velocity()[1][f2];
                    let d2 = positive(wall - plane(g, 1, m - 2))?;
                    let gap = positive(d2 - d1)?;
                    let d2i = FlatWallInterval::point(wall)?
                        .minus(FlatWallInterval::point(plane(g, 1, m - 2))?)?;
                    let gapi = d2i.minus(d1i)?;
                    let c1 = div(-d2, mul(d1, gap)?)?;
                    let c2 = div(d1, mul(d2, gap)?)?;
                    let derivative = checked(mul(c1, v1)? + mul(c2, v2)?)?;
                    let di = FlatWallInterval::point(-1.)?
                        .times(d2i)?
                        .quotient(d1i.times(gapi)?)?
                        .times(FlatWallInterval::point(v1)?)?
                        .plus(
                            d1i.quotient(d2i.times(gapi)?)?
                                .times(FlatWallInterval::point(v2)?)?,
                        )?;
                    (derivative, di)
                }
            };
            let tau = mul(-2. * mu, derivative)?;
            let taui = FlatWallInterval::point(-2.)?
                .times(FlatWallInterval::point(mu)?)?
                .times(di)?;
            let dx = positive(plane(g, 0, x + 1) - plane(g, 0, x))?;
            let dz = positive(plane(g, 2, z + 1) - plane(g, 2, z))?;
            let dxi = FlatWallInterval::point(plane(g, 0, x + 1))?
                .minus(FlatWallInterval::point(plane(g, 0, x))?)?;
            let dzi = FlatWallInterval::point(plane(g, 2, z + 1))?
                .minus(FlatWallInterval::point(plane(g, 2, z))?)?;
            let cx = mul(0.5, plane(g, 0, x) + plane(g, 0, x + 1))?;
            let cz = mul(0.5, plane(g, 2, z) + plane(g, 2, z + 1))?;
            let cxi = FlatWallInterval::point(plane(g, 0, x))?
                .plus(FlatWallInterval::point(plane(g, 0, x + 1))?)?
                .times(FlatWallInterval::point(0.5)?)?;
            let czi = FlatWallInterval::point(plane(g, 2, z))?
                .plus(FlatWallInterval::point(plane(g, 2, z + 1))?)?
                .times(FlatWallInterval::point(0.5)?)?;
            accumulate(
                1,
                tau,
                taui,
                mul(dx, dz)?,
                dxi.times(dzi)?,
                [cx, wall, cz],
                [cxi, FlatWallInterval::point(wall)?, czi],
            )?;
        }
    }
    // Uz and the normal wall trace vanish structurally on the admitted trial
    // footprint, so z shear is exactly zero. Other solid faces likewise vanish.
    let mut f = [0.; 3];
    let mut t = [0.; 3];
    for (i, s) in force.into_iter().enumerate() {
        f[i] = s.finish()?;
    }
    for (i, s) in torque.into_iter().enumerate() {
        t[i] = s.finish()?;
    }
    Ok(FlatWallOwnedWrench {
        force: f,
        torque: t,
        arithmetic_force: fi,
        arithmetic_torque: ti,
        geometric_wall_area: mul(
            plane(g, 0, 2 * m) - plane(g, 0, m),
            plane(g, 2, 2 * m) - plane(g, 2, m),
        )?,
        tangential_basis_area: basis_area.finish()?,
        energy_sector_effective_area: effective_area.finish()?,
    })
}

#[cfg(test)]
mod algebra_tests {
    use super::*;
    #[test]
    fn unrelated_synthetic_spd_ldlt_and_refusals() {
        let mut factor = [0.; 4];
        let mut q = [0.; 2];
        let pivot = dense_ldlt(
            &[2., 1., 1., 2.],
            &[3., 0.],
            &mut factor,
            &mut q,
            &mut |_, _| false,
        )
        .unwrap();
        assert_eq!(pivot, 1.5);
        assert_eq!(q, [2., -1.]);
        assert!(matches!(
            dense_ldlt(
                &[1., 1., 1., 1.],
                &[0.; 2],
                &mut factor,
                &mut q,
                &mut |_, _| false
            ),
            Err(FlatWallRitzError::NonPositivePivot { index: 1 })
        ));
        assert!(
            dense_ldlt(
                &[2., 1., 0., 2.],
                &[0.; 2],
                &mut factor,
                &mut q,
                &mut |_, _| false
            )
            .is_err()
        );
        assert!(
            dense_ldlt(
                &[2., 1., 1., 2.],
                &[0.; 2],
                &mut factor,
                &mut q,
                &mut |_, _| true
            )
            .is_err()
        );
    }
}
