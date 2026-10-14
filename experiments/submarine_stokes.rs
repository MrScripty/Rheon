//! Bounded, opt-in steady creeping-Stokes laboratory on an aligned padded cuboid.
//!
//! This module is included by its example and contract tests, not `src/lib.rs`.
//! All six outer planes prescribe the full velocity. The solid is stationary.
//! There is no convection, physical clock, moving geometry or turbulence model.
//! Construction checks DOFs before allocating; solve reuses every retained Vec.
//! Cancellation and every refusal preserve the accepted fields, report and ID.
//! Payload is actual Vec capacity, excluding borrowed geometry, stack, allocator
//! metadata, callback storage and RSS. The dense backend has a hard 4096 DOF cap.
#[path = "outward.rs"]
mod outward;
use rheon::{Axis, GridGeometry, StaticObstacleGeometry};
use std::{fmt, mem::size_of};

pub const MAX_DOFS: usize = 4096;
pub const PIVOT_FLOOR: f64 = 1e-12;
pub const RESIDUAL_FACTOR: f64 = 4096.0 * f64::EPSILON;
/// At most four correction solves after the initial solve, reusing all buffers.
pub const MAX_REFINEMENT_PASSES: usize = 4;
pub const CORNER_RULE: &str = "rheon-i1-sector-half-cell-v1";
const NONE: usize = usize::MAX;
const AXES: [Axis; 3] = [Axis::X, Axis::Y, Axis::Z];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cuboid {
    pub lower: [f64; 3],
    pub upper: [f64; 3],
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Config {
    pub counts: [usize; 3],
    pub origin: [f64; 3],
    pub spacing: [f64; 3],
    pub body: Option<Cuboid>,
    pub density: f64,
    pub viscosity: f64,
    pub u_ref: f64,
    pub l_ref: f64,
    pub dof_limit: usize,
    pub payload_limit: usize,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnalyticField {
    Uniform([f64; 3]),
    /// Body-centered psi=A product((x_d^2-a_d^2)^2), u=(psi_y,-psi_x,0).
    PolynomialMms {
        amplitude: f64,
        pressure_linear: [f64; 3],
        pressure_quadratic: [f64; 3],
    },
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Request {
    pub boundary: AnalyticField,
    pub pressure_offset: f64,
    pub moment_reference: [f64; 3],
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Input,
    Boundary,
    Assembly,
    Factorization,
    BackSubstitution,
    Residual,
    Reconstruction,
    Loads,
    Commit,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    InvalidParameter,
    UnsupportedGeometry,
    GeometryMismatch,
    ArithmeticFailure,
    CapacityOverflow,
    AllocationFailure,
    DofLimit {
        required: usize,
        limit: usize,
    },
    BufferLimit {
        required: usize,
        limit: usize,
    },
    Cancelled {
        stage: Stage,
        index: usize,
    },
    IncompatibleFlux {
        defect: f64,
        budget: f64,
    },
    Conditioning {
        column: usize,
        scaled_pivot: f64,
    },
    Residual {
        row: usize,
        defect: f64,
        budget: f64,
    },
    Enclosure(outward::Error),
    SolveIdOverflow,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "submarine Stokes refused: {self:?}")
    }
}
impl std::error::Error for Error {}
impl From<outward::Error> for Error {
    fn from(e: outward::Error) -> Self {
        Self::Enclosure(e)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Owner {
    Body,
    Outer { axis: usize, side: i8 },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableKey {
    Face {
        axis: usize,
        coordinates: [usize; 3],
    },
    Ghost {
        axes: [usize; 2],
        coordinates: [usize; 3],
        component: usize,
    },
    Corner {
        axes: [usize; 2],
        coordinates: [usize; 3],
        component: usize,
    },
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrescribedVariable {
    pub variable: usize,
    pub key: VariableKey,
    pub owner: Owner,
    pub component: usize,
    pub point: [f64; 3],
    pub value: f64,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundaryTrace {
    pub key: VariableKey,
    pub owner: Owner,
    pub point: [f64; 3],
    pub velocity: [f64; 3],
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WallPatch {
    pub cell: usize,
    pub axis: usize,
    pub side: i8,
    pub point: [f64; 3],
    pub normal: [f64; 3],
    pub area: f64,
    pub velocity: [f64; 3],
    pub pressure: f64,
    pub gradient: [[f64; 3]; 3],
}
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Loads {
    pub pressure_force: [f64; 3],
    pub shear_force: [f64; 3],
    pub pressure_moment: [f64; 3],
    pub shear_moment: [f64; 3],
    /// Fluid reaction on the body: minus the functional boundary derivative.
    pub body_force: [f64; 3],
    /// Force exerted on the fluid by the outer boundary: functional derivative.
    pub outer_force: [f64; 3],
    /// Fluid reaction moment on the body about the requested reference.
    pub body_moment: [f64; 3],
    /// Outer boundary moment on the fluid about the requested reference.
    pub outer_moment: [f64; 3],
    pub wall_work: f64,
    pub outer_work: f64,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PatchLoad {
    pub pressure_force: [f64; 3],
    pub shear_force: [f64; 3],
    pub pressure_moment: [f64; 3],
    pub shear_moment: [f64; 3],
    pub work: f64,
}
/// Original Newtonian constitutive/load path used by solves and analytic controls.
pub fn integrate_patch(
    p: &WallPatch,
    viscosity: f64,
    reference: [f64; 3],
) -> Result<PatchLoad, Error> {
    positive(viscosity)?;
    positive(p.area)?;
    let mut pressure_force = [0.0; 3];
    let mut shear_force = [0.0; 3];
    let mut arm = [0.0; 3];
    for (a, _) in AXES.iter().enumerate() {
        arm[a] = checked(p.point[a] - reference[a])?;
        pressure_force[a] = mul(mul(-p.pressure, p.normal[a])?, p.area)?;
        let mut shear = 0.0;
        for b in 0..3 {
            shear = add(
                shear,
                mul(add(p.gradient[a][b], p.gradient[b][a])?, p.normal[b])?,
            )?;
        }
        shear_force[a] = mul(mul(viscosity, shear)?, p.area)?;
    }
    let pressure_moment = cross(arm, pressure_force)?;
    let shear_moment = cross(arm, shear_force)?;
    let mut work = 0.0;
    for (a, _) in AXES.iter().enumerate() {
        work = add(
            work,
            mul(add(pressure_force[a], shear_force[a])?, p.velocity[a])?,
        )?;
    }
    Ok(PatchLoad {
        pressure_force,
        shear_force,
        pressure_moment,
        shear_moment,
        work,
    })
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Report {
    pub solve_id: u64,
    pub actual_dofs: usize,
    pub active_velocity_dofs: usize,
    pub nongauge_pressure_dofs: usize,
    pub min_scaled_pivot: f64,
    pub retained_payload_bytes: usize,
    pub payload_limit_bytes: usize,
    pub max_momentum_defect: f64,
    pub max_divergence_defect: f64,
    pub max_residual_ratio: f64,
    pub dissipation: f64,
    pub body_work: f64,
    pub outer_work: f64,
    pub forcing_work: f64,
    pub work_defect: f64,
    pub work_budget: f64,
}
#[derive(Debug)]
pub struct Solution {
    pub config: Config,
    pub request: Request,
    pub face_fields: [Vec<f64>; 3],
    pub volume_forcing: [Vec<f64>; 3],
    pub pressure: Vec<f64>,
    pub variable_values: Vec<f64>,
    pub prescribed: Vec<PrescribedVariable>,
    pub outer_traces: Vec<BoundaryTrace>,
    pub body_traces: Vec<BoundaryTrace>,
    pub wall_patches: Vec<WallPatch>,
    /// Functional derivative K_all v-B_all^T p-F_all, in prescribed order.
    pub reactions: Vec<f64>,
    pub loads: Loads,
    pub report: Report,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Term {
    pub variable: usize,
    pub coefficient: f64,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrainRow {
    pub axes: [usize; 2],
    pub coordinates: [usize; 3],
    pub quadrant: u8,
    /// Hessian scalar: 2 mu V for normal rows, mu V/4 for shear sectors.
    pub hessian_weight: f64,
    pub terms: [Term; 4],
    pub term_count: usize,
}
impl StrainRow {
    fn new(axes: [usize; 2], coordinates: [usize; 3], quadrant: u8, hessian_weight: f64) -> Self {
        Self {
            axes,
            coordinates,
            quadrant,
            hessian_weight,
            terms: [Term {
                variable: NONE,
                coefficient: 0.0,
            }; 4],
            term_count: 0,
        }
    }
    fn term(&mut self, variable: usize, coefficient: f64) -> Result<(), Error> {
        if variable == NONE || self.term_count == 4 {
            return Err(Error::UnsupportedGeometry);
        }
        self.terms[self.term_count] = Term {
            variable,
            coefficient: checked(coefficient)?,
        };
        self.term_count += 1;
        Ok(())
    }
    pub fn terms(&self) -> &[Term] {
        &self.terms[..self.term_count]
    }
}
/// One immutable supported geometry and one atomic accepted solution owner.
pub struct Solver<'a> {
    config: Config,
    grid: GridGeometry,
    geometry: Option<&'a StaticObstacleGeometry>,
    body_planes: Option<([usize; 3], [usize; 3])>,
    face_masks: [Vec<u8>; 3],
    face_dual_volumes: [Vec<f64>; 3],
    face_variables: [Vec<usize>; 3],
    pressure_unknowns: Vec<usize>,
    rows: Vec<StrainRow>,
    nv: usize,
    nd: usize,
    nvars: usize,
    gauge: usize,
    matrix: Vec<f64>,
    rhs: Vec<f64>,
    column_scales: Vec<f64>,
    unknowns: Vec<f64>,
    gradients: Vec<f64>,
    magnitudes: Vec<f64>,
    candidate: Solution,
    accepted: Solution,
    has_accepted: bool,
    payload: usize,
}

fn checked(x: f64) -> Result<f64, Error> {
    if !x.is_finite() || (x != 0.0 && !x.is_normal()) {
        Err(Error::ArithmeticFailure)
    } else {
        Ok(x)
    }
}
fn positive(x: f64) -> Result<f64, Error> {
    checked(x)?;
    if x > 0.0 {
        Ok(x)
    } else {
        Err(Error::ArithmeticFailure)
    }
}
fn add(a: f64, b: f64) -> Result<f64, Error> {
    outward::Interval::point(a)?.add(outward::Interval::point(b)?)?;
    checked(a + b)
}
fn mul(a: f64, b: f64) -> Result<f64, Error> {
    outward::Interval::point(a)?.mul(outward::Interval::point(b)?)?;
    checked(a * b)
}
fn div(a: f64, b: f64) -> Result<f64, Error> {
    outward::Interval::point(a)?.div(outward::Interval::point(b)?)?;
    checked(a / b)
}
fn cross(a: [f64; 3], b: [f64; 3]) -> Result<[f64; 3], Error> {
    Ok([
        add(mul(a[1], b[2])?, -mul(a[2], b[1])?)?,
        add(mul(a[2], b[0])?, -mul(a[0], b[2])?)?,
        add(mul(a[0], b[1])?, -mul(a[1], b[0])?)?,
    ])
}
fn product(v: [usize; 3]) -> Result<usize, Error> {
    v[0].checked_mul(v[1])
        .and_then(|x| x.checked_mul(v[2]))
        .ok_or(Error::CapacityOverflow)
}
fn sum(a: usize, b: usize) -> Result<usize, Error> {
    a.checked_add(b).ok_or(Error::CapacityOverflow)
}
fn index(n: [usize; 3], p: [usize; 3]) -> usize {
    p[0] + n[0] * (p[1] + n[1] * p[2])
}
fn coordinate(n: [usize; 3], i: usize) -> [usize; 3] {
    [i % n[0], (i / n[0]) % n[1], i / (n[0] * n[1])]
}
fn checkpoint(
    c: &mut impl FnMut(Stage, usize) -> bool,
    stage: Stage,
    i: usize,
) -> Result<(), Error> {
    if c(stage, i) {
        Err(Error::Cancelled { stage, index: i })
    } else {
        Ok(())
    }
}
fn gate(required: usize, limit: usize) -> Result<(), Error> {
    if required > limit {
        Err(Error::BufferLimit { required, limit })
    } else {
        Ok(())
    }
}
fn alloc<T: Clone>(n: usize, value: T, used: &mut usize, limit: usize) -> Result<Vec<T>, Error> {
    let planned = sum(
        *used,
        n.checked_mul(size_of::<T>())
            .ok_or(Error::CapacityOverflow)?,
    )?;
    gate(planned, limit)?;
    let mut result = Vec::new();
    result
        .try_reserve_exact(n)
        .map_err(|_| Error::AllocationFailure)?;
    *used = sum(
        *used,
        result
            .capacity()
            .checked_mul(size_of::<T>())
            .ok_or(Error::CapacityOverflow)?,
    )?;
    gate(*used, limit)?;
    result.resize(n, value);
    Ok(result)
}
fn reserve<T>(n: usize, used: &mut usize, limit: usize) -> Result<Vec<T>, Error> {
    let planned = sum(
        *used,
        n.checked_mul(size_of::<T>())
            .ok_or(Error::CapacityOverflow)?,
    )?;
    gate(planned, limit)?;
    let mut result = Vec::new();
    result
        .try_reserve_exact(n)
        .map_err(|_| Error::AllocationFailure)?;
    *used = sum(
        *used,
        result
            .capacity()
            .checked_mul(size_of::<T>())
            .ok_or(Error::CapacityOverflow)?,
    )?;
    gate(*used, limit)?;
    Ok(result)
}

fn fluid(n: [usize; 3], planes: Option<([usize; 3], [usize; 3])>, p: [usize; 3]) -> bool {
    if (0..3).any(|d| p[d] >= n[d]) {
        return false;
    }
    match planes {
        None => true,
        Some((lo, hi)) => !(0..3).all(|d| lo[d] <= p[d] && p[d] < hi[d]),
    }
}
fn face_mask(
    n: [usize; 3],
    planes: Option<([usize; 3], [usize; 3])>,
    a: usize,
    p: [usize; 3],
) -> u8 {
    let upper = fluid(n, planes, p);
    let lower = if p[a] > 0 {
        let mut q = p;
        q[a] -= 1;
        fluid(n, planes, q)
    } else {
        false
    };
    match (lower, upper) {
        (true, true) => 1,
        (false, false) => 0,
        _ => 2,
    }
}
fn each_edge(
    n: [usize; 3],
    planes: Option<([usize; 3], [usize; 3])>,
    mut f: impl FnMut(usize, usize, [usize; 3], [bool; 4]) -> Result<(), Error>,
) -> Result<(), Error> {
    for (a, b) in [(0, 1), (0, 2), (1, 2)] {
        let mut shape = n;
        shape[a] += 1;
        shape[b] += 1;
        for edge in 0..product(shape)? {
            let p = coordinate(shape, edge);
            let sectors = std::array::from_fn(|q| {
                let mut cell = p;
                for (d, bit) in [(a, 1), (b, 2)] {
                    if q & bit == 0 {
                        if cell[d] == 0 {
                            return false;
                        }
                        cell[d] -= 1;
                    }
                }
                fluid(n, planes, cell)
            });
            if sectors.iter().any(|&x| x) {
                f(a, b, p, sectors)?;
            }
        }
    }
    Ok(())
}
fn point(c: Config, p: [usize; 3], planes: [bool; 3]) -> [f64; 3] {
    std::array::from_fn(|d| {
        c.origin[d] + (p[d] as f64 + if planes[d] { 0.0 } else { 0.5 }) * c.spacing[d]
    })
}
fn face_point(c: Config, a: usize, p: [usize; 3]) -> [f64; 3] {
    point(c, p, std::array::from_fn(|d| d == a))
}
fn edge_point(c: Config, a: usize, b: usize, p: [usize; 3]) -> [f64; 3] {
    point(c, p, std::array::from_fn(|d| d == a || d == b))
}
fn face_owner(c: Config, a: usize, p: [usize; 3]) -> Owner {
    if p[a] == 0 {
        Owner::Outer { axis: a, side: -1 }
    } else if p[a] == c.counts[a] {
        Owner::Outer { axis: a, side: 1 }
    } else {
        Owner::Body
    }
}
fn ghost_owner(c: Config, across: usize, p: [usize; 3]) -> Owner {
    if p[across] == 0 {
        Owner::Outer {
            axis: across,
            side: -1,
        }
    } else if p[across] == c.counts[across] {
        Owner::Outer {
            axis: across,
            side: 1,
        }
    } else {
        Owner::Body
    }
}
fn empty_report(c: Config) -> Report {
    Report {
        solve_id: 0,
        actual_dofs: 0,
        active_velocity_dofs: 0,
        nongauge_pressure_dofs: 0,
        min_scaled_pivot: 0.0,
        retained_payload_bytes: 0,
        payload_limit_bytes: c.payload_limit,
        max_momentum_defect: 0.0,
        max_divergence_defect: 0.0,
        max_residual_ratio: 0.0,
        dissipation: 0.0,
        body_work: 0.0,
        outer_work: 0.0,
        forcing_work: 0.0,
        work_defect: 0.0,
        work_budget: 0.0,
    }
}
#[derive(Clone, Copy)]
struct SolutionShape {
    nvars: usize,
    npres: usize,
    nouter: usize,
    nbody: usize,
    npatch: usize,
}
fn solution(
    c: Config,
    g: &GridGeometry,
    shape: SolutionShape,
    used: &mut usize,
) -> Result<Solution, Error> {
    let SolutionShape {
        nvars,
        npres,
        nouter,
        nbody,
        npatch,
    } = shape;
    let mut face_fields = [Vec::new(), Vec::new(), Vec::new()];
    let mut volume_forcing = [Vec::new(), Vec::new(), Vec::new()];
    for (a, _) in AXES.iter().enumerate() {
        face_fields[a] = alloc(g.face_len(AXES[a]), 0.0, used, c.payload_limit)?;
        volume_forcing[a] = alloc(g.face_len(AXES[a]), 0.0, used, c.payload_limit)?;
    }
    Ok(Solution {
        config: c,
        request: Request {
            boundary: AnalyticField::Uniform([0.0; 3]),
            pressure_offset: 0.0,
            moment_reference: [0.0; 3],
        },
        face_fields,
        volume_forcing,
        pressure: alloc(g.cell_len(), 0.0, used, c.payload_limit)?,
        variable_values: alloc(nvars, 0.0, used, c.payload_limit)?,
        prescribed: reserve(npres, used, c.payload_limit)?,
        outer_traces: reserve(nouter, used, c.payload_limit)?,
        body_traces: reserve(nbody, used, c.payload_limit)?,
        wall_patches: reserve(npatch, used, c.payload_limit)?,
        reactions: alloc(npres, 0.0, used, c.payload_limit)?,
        loads: Loads::default(),
        report: empty_report(c),
    })
}
impl<'a> Solver<'a> {
    /// Checks supported aligned/padded geometry, total unknowns and complete
    /// planned payload before allocating. Body geometry remains borrowed.
    pub fn new(
        config: Config,
        geometry: Option<&'a StaticObstacleGeometry>,
    ) -> Result<Self, Error> {
        outward::ensure_supported()?;
        if config.counts.iter().any(|&n| n < 2) || config.dof_limit == 0 {
            return Err(Error::InvalidParameter);
        }
        for value in [config.density, config.viscosity, config.u_ref, config.l_ref] {
            positive(value).map_err(|_| Error::InvalidParameter)?;
        }
        let counts = std::array::from_fn(|d| u64::try_from(config.counts[d]).unwrap_or(u64::MAX));
        let grid = GridGeometry::new(counts, config.spacing, config.origin)
            .map_err(|_| Error::InvalidParameter)?;
        let body_planes = if let Some(body) = config.body {
            let geom = geometry.ok_or(Error::GeometryMismatch)?;
            if geom.grid() != &grid
                || geom.box_bounds() != (body.lower, body.upper)
                || geom.component_count() != 1
            {
                return Err(Error::GeometryMismatch);
            }
            let mut lo = [0; 3];
            let mut hi = [0; 3];
            for d in 0..3 {
                if !body.lower[d].is_finite()
                    || !body.upper[d].is_finite()
                    || body.lower[d] >= body.upper[d]
                {
                    return Err(Error::InvalidParameter);
                }
                let l = (body.lower[d] - config.origin[d]) / config.spacing[d];
                let h = (body.upper[d] - config.origin[d]) / config.spacing[d];
                if l < 2.0
                    || h > config.counts[d] as f64 - 2.0
                    || l.fract() != 0.0
                    || h.fract() != 0.0
                {
                    return Err(Error::UnsupportedGeometry);
                }
                lo[d] = l as usize;
                hi[d] = h as usize;
                if point(config, lo, [true; 3])[d] != body.lower[d]
                    || point(config, hi, [true; 3])[d] != body.upper[d]
                {
                    return Err(Error::UnsupportedGeometry);
                }
            }
            Some((lo, hi))
        } else {
            if geometry.is_some() {
                return Err(Error::GeometryMismatch);
            }
            None
        };
        let solid = if let Some((lo, hi)) = body_planes {
            product(std::array::from_fn(|d| hi[d] - lo[d]))?
        } else {
            0
        };
        let nf = grid
            .cell_len()
            .checked_sub(solid)
            .ok_or(Error::CapacityOverflow)?;
        let limit = config.dof_limit.min(MAX_DOFS);
        if nf - 1 > limit {
            return Err(Error::DofLimit {
                required: nf - 1,
                limit,
            });
        }
        let mut nv = 0;
        let mut nfacepres = 0;
        let mut nouter = 0;
        let mut nbody = 0;
        let mut npatch = 0;
        for (a, _) in AXES.iter().enumerate() {
            for face in 0..grid.face_len(AXES[a]) {
                let p = coordinate(grid.face_counts(AXES[a]), face);
                match face_mask(config.counts, body_planes, a, p) {
                    1 => nv = sum(nv, 1)?,
                    2 => {
                        nfacepres = sum(nfacepres, 1)?;
                        match face_owner(config, a, p) {
                            Owner::Body => {
                                nbody = sum(nbody, 1)?;
                                npatch = sum(npatch, 1)?;
                            }
                            Owner::Outer { .. } => nouter = sum(nouter, 1)?,
                        }
                    }
                    _ => {}
                }
            }
        }
        let nd = sum(nv, nf - 1)?;
        if nd > limit {
            return Err(Error::DofLimit {
                required: nd,
                limit,
            });
        }
        let mut nrows = nf.checked_mul(3).ok_or(Error::CapacityOverflow)?;
        let mut nghost = 0;
        each_edge(config.counts, body_planes, |a, b, p, sectors| {
            let count = sectors.iter().filter(|&&x| x).count();
            nrows = sum(nrows, if count == 3 { 3 } else { 1 })?;
            for (component, across) in [(a, b), (b, a)] {
                let mut present = [false; 2];
                if p[across] > 0 {
                    let mut q = p;
                    q[across] -= 1;
                    present[0] = face_mask(config.counts, body_planes, component, q) != 0;
                }
                if p[across] < config.counts[across] {
                    present[1] = face_mask(config.counts, body_planes, component, p) != 0;
                }
                if count == 3 || present[0] != present[1] {
                    nghost = sum(nghost, 1)?;
                    match ghost_owner(config, across, p) {
                        Owner::Body => nbody = sum(nbody, 1)?,
                        Owner::Outer { .. } => nouter = sum(nouter, 1)?,
                    }
                } else if !present[0] {
                    return Err(Error::UnsupportedGeometry);
                }
            }
            Ok(())
        })?;
        let npres = sum(nfacepres, nghost)?;
        let nvars = sum(nv, npres)?;
        // Complete planned retained payload, before the first reserve. Actual
        // allocator capacities are separately gated after each fallible reserve.
        let nfaces = (0..3).try_fold(0, |s, a| sum(s, grid.face_len(AXES[a])))?;
        let matrix_len = nd.checked_mul(nd).ok_or(Error::CapacityOverflow)?;
        let mut planned = 0usize;
        for (n, size) in [
            (
                nfaces,
                size_of::<u8>() + size_of::<f64>() + size_of::<usize>(),
            ),
            (grid.cell_len(), size_of::<usize>()),
            (nrows, size_of::<StrainRow>()),
            (matrix_len, size_of::<f64>()),
            (nd, 3 * size_of::<f64>()),
            (nvars, 2 * size_of::<f64>()),
            (2 * nfaces, 2 * size_of::<f64>()),
            (2 * grid.cell_len(), size_of::<f64>()),
            (2 * nvars, size_of::<f64>()),
            (
                2 * npres,
                size_of::<PrescribedVariable>() + size_of::<f64>(),
            ),
            (2 * (nouter + nbody), size_of::<BoundaryTrace>()),
            (2 * npatch, size_of::<WallPatch>()),
        ] {
            planned = sum(planned, n.checked_mul(size).ok_or(Error::CapacityOverflow)?)?;
        }
        gate(planned, config.payload_limit)?;
        let mut used = 0;
        let mut face_masks = [Vec::new(), Vec::new(), Vec::new()];
        let mut face_dual_volumes = [Vec::new(), Vec::new(), Vec::new()];
        let mut face_variables = [Vec::new(), Vec::new(), Vec::new()];
        for (a, _) in AXES.iter().enumerate() {
            let n = grid.face_len(AXES[a]);
            face_masks[a] = alloc(n, 0u8, &mut used, config.payload_limit)?;
            face_dual_volumes[a] = alloc(n, 0.0, &mut used, config.payload_limit)?;
            face_variables[a] = alloc(n, NONE, &mut used, config.payload_limit)?;
        }
        let mut pressure_unknowns = alloc(grid.cell_len(), NONE, &mut used, config.payload_limit)?;
        let mut gauge = NONE;
        let mut pi = nv;
        for (cell, mapped) in pressure_unknowns.iter_mut().enumerate() {
            let is_fluid = fluid(config.counts, body_planes, coordinate(config.counts, cell));
            if let Some(geom) = geometry
                && geom.fluid_volumes()[cell] != if is_fluid { grid.cell_volume() } else { 0.0 }
            {
                return Err(Error::GeometryMismatch);
            }
            if is_fluid {
                if gauge == NONE {
                    gauge = cell;
                } else {
                    *mapped = pi;
                    pi += 1;
                }
            }
        }
        let candidate = solution(
            config,
            &grid,
            SolutionShape {
                nvars,
                npres,
                nouter,
                nbody,
                npatch,
            },
            &mut used,
        )?;
        let accepted = solution(
            config,
            &grid,
            SolutionShape {
                nvars,
                npres,
                nouter,
                nbody,
                npatch,
            },
            &mut used,
        )?;
        let mut result = Self {
            config,
            grid,
            geometry,
            body_planes,
            face_masks,
            face_dual_volumes,
            face_variables,
            pressure_unknowns,
            rows: reserve(nrows, &mut used, config.payload_limit)?,
            nv,
            nd,
            nvars,
            gauge,
            matrix: alloc(matrix_len, 0.0, &mut used, config.payload_limit)?,
            rhs: alloc(nd, 0.0, &mut used, config.payload_limit)?,
            column_scales: alloc(nd, 0.0, &mut used, config.payload_limit)?,
            unknowns: alloc(nd, 0.0, &mut used, config.payload_limit)?,
            gradients: alloc(nvars, 0.0, &mut used, config.payload_limit)?,
            magnitudes: alloc(nvars, 0.0, &mut used, config.payload_limit)?,
            candidate,
            accepted,
            has_accepted: false,
            payload: used,
        };
        result.build_rows()?;
        if result.rows.len() != nrows
            || result.candidate.prescribed.len() != npres
            || result.candidate.wall_patches.len() != npatch
        {
            return Err(Error::ArithmeticFailure);
        }
        for p in &result.candidate.prescribed {
            result.accepted.prescribed.push(*p);
        }
        for p in &result.candidate.outer_traces {
            result.accepted.outer_traces.push(*p);
        }
        for p in &result.candidate.body_traces {
            result.accepted.body_traces.push(*p);
        }
        for p in &result.candidate.wall_patches {
            result.accepted.wall_patches.push(*p);
        }
        Ok(result)
    }
    pub fn config(&self) -> Config {
        self.config
    }
    pub fn grid(&self) -> &GridGeometry {
        &self.grid
    }
    pub fn geometry(&self) -> Option<&'a StaticObstacleGeometry> {
        self.geometry
    }
    pub fn face_masks(&self, axis: usize) -> &[u8] {
        &self.face_masks[axis]
    }
    pub fn face_dual_volumes(&self, axis: usize) -> &[f64] {
        &self.face_dual_volumes[axis]
    }
    pub fn is_fluid(&self, cell: usize) -> bool {
        cell < self.grid.cell_len()
            && fluid(
                self.config.counts,
                self.body_planes,
                coordinate(self.config.counts, cell),
            )
    }
    pub fn gauge_cell(&self) -> usize {
        self.gauge
    }
    pub fn active_velocity_dofs(&self) -> usize {
        self.nv
    }
    pub fn nongauge_pressure_dofs(&self) -> usize {
        self.nd - self.nv
    }
    pub fn actual_dofs(&self) -> usize {
        self.nd
    }
    pub fn allocated_bytes(&self) -> usize {
        self.payload
    }
    pub fn rows(&self) -> &[StrainRow] {
        &self.rows
    }
    pub fn strain_rows(&self) -> &[StrainRow] {
        &self.rows
    }
    pub fn face_variable(&self, axis: usize, p: [usize; 3]) -> Option<usize> {
        if axis >= 3 || (0..3).any(|d| p[d] >= self.grid.face_counts(AXES[axis])[d]) {
            return None;
        }
        let v = self.face_variables[axis][index(self.grid.face_counts(AXES[axis]), p)];
        if v == NONE { None } else { Some(v) }
    }
    pub fn accepted(&self) -> Option<&Solution> {
        if self.has_accepted {
            Some(&self.accepted)
        } else {
            None
        }
    }
    fn prescribe(
        &mut self,
        key: VariableKey,
        owner: Owner,
        component: usize,
        point: [f64; 3],
    ) -> usize {
        let variable = self.nv + self.candidate.prescribed.len();
        self.candidate.prescribed.push(PrescribedVariable {
            variable,
            key,
            owner,
            component,
            point,
            value: 0.0,
        });
        let trace = BoundaryTrace {
            key,
            owner,
            point,
            velocity: [0.0; 3],
        };
        match owner {
            Owner::Body => self.candidate.body_traces.push(trace),
            Owner::Outer { .. } => self.candidate.outer_traces.push(trace),
        }
        variable
    }
    fn build_rows(&mut self) -> Result<(), Error> {
        let c = self.config;
        let v = self.grid.cell_volume();
        let mut active = 0;
        for (a, _) in AXES.iter().enumerate() {
            for face in 0..self.face_masks[a].len() {
                let p = coordinate(self.grid.face_counts(AXES[a]), face);
                let mask = face_mask(c.counts, self.body_planes, a, p);
                self.face_masks[a][face] = mask;
                self.face_dual_volumes[a][face] = match mask {
                    1 => v,
                    2 => v / 2.0,
                    _ => 0.0,
                };
                if let Some(geom) = self.geometry {
                    let expected = if mask == 1
                        || (mask != 0 && !matches!(face_owner(c, a, p), Owner::Body))
                    {
                        self.face_area(a)?
                    } else {
                        0.0
                    };
                    if geom.open_areas(AXES[a])[face] != expected {
                        return Err(Error::GeometryMismatch);
                    }
                }
                if mask == 1 {
                    self.face_variables[a][face] = active;
                    active += 1;
                }
            }
        }
        for cell in 0..self.grid.cell_len() {
            if !self.is_fluid(cell) {
                continue;
            }
            let p = coordinate(c.counts, cell);
            for (a, _) in AXES.iter().enumerate() {
                for side in [-1i8, 1] {
                    let mut fp = p;
                    if side == 1 {
                        fp[a] += 1;
                    }
                    if self.face_masks[a][index(self.grid.face_counts(AXES[a]), fp)] == 2
                        && matches!(face_owner(c, a, fp), Owner::Body)
                    {
                        let mut normal = [0.0; 3];
                        normal[a] = -(side as f64);
                        self.candidate.wall_patches.push(WallPatch {
                            cell,
                            axis: a,
                            side: -(side),
                            point: face_point(c, a, fp),
                            normal,
                            area: self.face_area(a)?,
                            velocity: [0.0; 3],
                            pressure: 0.0,
                            gradient: [[0.0; 3]; 3],
                        });
                    }
                }
            }
        }
        for (a, _) in AXES.iter().enumerate() {
            for face in 0..self.face_masks[a].len() {
                if self.face_masks[a][face] != 2 {
                    continue;
                }
                let p = coordinate(self.grid.face_counts(AXES[a]), face);
                let variable = self.prescribe(
                    VariableKey::Face {
                        axis: a,
                        coordinates: p,
                    },
                    face_owner(c, a, p),
                    a,
                    face_point(c, a, p),
                );
                self.face_variables[a][face] = variable;
            }
        }
        for cell in 0..self.grid.cell_len() {
            if !self.is_fluid(cell) {
                continue;
            }
            let p = coordinate(c.counts, cell);
            for (a, _) in AXES.iter().enumerate() {
                let mut hi = p;
                hi[a] += 1;
                let mut row = StrainRow::new([a, a], p, 0, mul(mul(2.0, c.viscosity)?, v)?);
                row.term(
                    self.face_variables[a][index(self.grid.face_counts(AXES[a]), p)],
                    -div(1.0, c.spacing[a])?,
                )?;
                row.term(
                    self.face_variables[a][index(self.grid.face_counts(AXES[a]), hi)],
                    div(1.0, c.spacing[a])?,
                )?;
                self.rows.push(row);
            }
        }
        each_edge(c.counts, self.body_planes, |a, b, p, sectors| {
            let count = sectors.iter().filter(|&&x| x).count();
            let mut samples = [[NONE; 2]; 2];
            let mut ghosts = [NONE; 2];
            for (r, (component, across)) in [(a, b), (b, a)].into_iter().enumerate() {
                if p[across] > 0 {
                    let mut q = p;
                    q[across] -= 1;
                    samples[r][0] = self.face_variables[component]
                        [index(self.grid.face_counts(AXES[component]), q)];
                }
                if p[across] < c.counts[across] {
                    samples[r][1] = self.face_variables[component]
                        [index(self.grid.face_counts(AXES[component]), p)];
                }
                if count == 3 || (samples[r][0] == NONE) != (samples[r][1] == NONE) {
                    ghosts[r] = self.prescribe(
                        if count == 3 {
                            VariableKey::Corner {
                                axes: [a, b],
                                coordinates: p,
                                component,
                            }
                        } else {
                            VariableKey::Ghost {
                                axes: [a, b],
                                coordinates: p,
                                component,
                            }
                        },
                        ghost_owner(c, across, p),
                        component,
                        edge_point(c, a, b, p),
                    );
                }
            }
            for (q, &present) in sectors.iter().enumerate() {
                if !present {
                    continue;
                }
                let weight = if count == 3 {
                    mul(c.viscosity, v / 4.0)?
                } else {
                    mul(c.viscosity, mul(v / 4.0, count as f64)?)?
                };
                let mut row = StrainRow::new([a, b], p, q as u8, weight);
                for (r, (_, across, bit)) in [(a, b, 2), (b, a, 1)].into_iter().enumerate() {
                    if count == 3 {
                        // Separate V/4 rows for every fluid quadrant. Prescribed
                        // normal-face columns stay present even when zero.
                        let plus = q & bit != 0;
                        let sign = if plus { 1.0 } else { -1.0 };
                        let coefficient = div(2.0 * sign, c.spacing[across])?;
                        row.term(samples[r][usize::from(plus)], coefficient)?;
                        row.term(ghosts[r], -coefficient)?;
                    } else if samples[r][0] != NONE && samples[r][1] != NONE {
                        let coefficient = div(1.0, c.spacing[across])?;
                        row.term(samples[r][0], -coefficient)?;
                        row.term(samples[r][1], coefficient)?;
                    } else {
                        let plus = samples[r][1] != NONE;
                        let sign = if plus { 1.0 } else { -1.0 };
                        let coefficient = div(2.0 * sign, c.spacing[across])?;
                        row.term(samples[r][usize::from(plus)], coefficient)?;
                        row.term(ghosts[r], -coefficient)?;
                    }
                }
                self.rows.push(row);
                // The centered/intervening-boundary stencil is identical in
                // every incident noncorner sector. Retain one summed row;
                // the three-fluid corner keeps its three distinct V/4 rows.
                if count != 3 {
                    break;
                }
            }
            Ok(())
        })?;
        Ok(())
    }
    fn face_area(&self, a: usize) -> Result<f64, Error> {
        mul(
            self.config.spacing[(a + 1) % 3],
            self.config.spacing[(a + 2) % 3],
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnalyticSample {
    pub velocity: [f64; 3],
    pub pressure: f64,
    pub forcing: [f64; 3],
}
/// Analytic boundary and point-forcing evaluation. These values prescribe data;
/// they never replace the native solved unknown velocity or pressure.
pub fn analytic_sample(
    config: Config,
    request: Request,
    position: [f64; 3],
) -> Result<AnalyticSample, Error> {
    checked(request.pressure_offset)?;
    match request.boundary {
        AnalyticField::Uniform(velocity) => {
            for x in velocity {
                checked(x)?;
            }
            Ok(AnalyticSample {
                velocity,
                pressure: request.pressure_offset,
                forcing: [0.0; 3],
            })
        }
        AnalyticField::PolynomialMms {
            amplitude,
            pressure_linear,
            pressure_quadratic,
        } => {
            let body = config.body.ok_or(Error::UnsupportedGeometry)?;
            let x = std::array::from_fn::<_, 3, _>(|d| {
                position[d] - (body.lower[d] + body.upper[d]) / 2.0
            });
            let a = std::array::from_fn::<_, 3, _>(|d| (body.upper[d] - body.lower[d]) / 2.0);
            let mut q = [[0.0; 4]; 3];
            let mut pressure = request.pressure_offset;
            let mut forcing = [0.0; 3];
            for d in 0..3 {
                let x2 = mul(x[d], x[d])?;
                let a2 = mul(a[d], a[d])?;
                let t = add(x2, -a2)?;
                q[d] = [
                    mul(t, t)?,
                    mul(mul(4.0, x[d])?, t)?,
                    add(mul(12.0, x2)?, -mul(4.0, a2)?)?,
                    mul(24.0, x[d])?,
                ];
                pressure = add(
                    pressure,
                    add(
                        mul(pressure_linear[d], x[d])?,
                        mul(pressure_quadratic[d], x2)?,
                    )?,
                )?;
                forcing[d] = add(
                    pressure_linear[d],
                    mul(2.0, mul(pressure_quadratic[d], x[d])?)?,
                )?;
            }
            let term = |dx: usize, dy: usize, dz: usize| {
                mul(amplitude, mul(mul(q[0][dx], q[1][dy])?, q[2][dz])?)
            };
            let velocity = [term(0, 1, 0)?, -term(1, 0, 0)?, 0.0];
            let lap0 = add(add(term(2, 1, 0)?, term(0, 3, 0)?)?, term(0, 1, 2)?)?;
            let lap1 = -add(add(term(3, 0, 0)?, term(1, 2, 0)?)?, term(1, 0, 2)?)?;
            forcing[0] = add(forcing[0], -mul(config.viscosity, lap0)?)?;
            forcing[1] = add(forcing[1], -mul(config.viscosity, lap1)?)?;
            Ok(AnalyticSample {
                velocity,
                pressure,
                forcing,
            })
        }
    }
}

impl Solver<'_> {
    /// Synchronous bounded attempt. The callback is observed at each stage and
    /// during row/pivot work, including rest solves. Only the final swap publishes.
    pub fn solve(
        &mut self,
        request: Request,
        mut cancel: impl FnMut(Stage, usize) -> bool,
    ) -> Result<&Solution, Error> {
        checkpoint(&mut cancel, Stage::Input, 0)?;
        for x in request.moment_reference {
            checked(x).map_err(|_| Error::InvalidParameter)?;
        }
        checked(request.pressure_offset).map_err(|_| Error::InvalidParameter)?;
        match request.boundary {
            AnalyticField::Uniform(v) => {
                for x in v {
                    checked(x).map_err(|_| Error::InvalidParameter)?;
                }
            }
            AnalyticField::PolynomialMms {
                amplitude,
                pressure_linear,
                pressure_quadratic,
            } => {
                if self.config.body.is_none() {
                    return Err(Error::UnsupportedGeometry);
                }
                for x in [
                    amplitude,
                    pressure_linear[0],
                    pressure_linear[1],
                    pressure_linear[2],
                    pressure_quadratic[0],
                    pressure_quadratic[1],
                    pressure_quadratic[2],
                ] {
                    checked(x).map_err(|_| Error::InvalidParameter)?;
                }
            }
        }
        let solve_id = if self.has_accepted {
            self.accepted
                .report
                .solve_id
                .checked_add(1)
                .ok_or(Error::SolveIdOverflow)?
        } else {
            1
        };
        self.candidate.request = request;
        self.candidate.loads = Loads::default();
        self.candidate.report = empty_report(self.config);
        self.candidate.variable_values.fill(0.0);
        self.candidate.pressure.fill(0.0);
        self.unknowns.fill(0.0);
        checkpoint(&mut cancel, Stage::Boundary, 0)?;
        for i in 0..self.candidate.prescribed.len() {
            checkpoint(&mut cancel, Stage::Boundary, i)?;
            let p = &mut self.candidate.prescribed[i];
            let value = match p.owner {
                Owner::Body => 0.0,
                Owner::Outer { .. } => {
                    analytic_sample(self.config, request, p.point)?.velocity[p.component]
                }
            };
            p.value = value;
            self.candidate.variable_values[p.variable] = value;
        }
        for trace in &mut self.candidate.outer_traces {
            trace.velocity = analytic_sample(self.config, request, trace.point)?.velocity;
        }
        for trace in &mut self.candidate.body_traces {
            trace.velocity = [0.0; 3];
        }
        for (a, _) in AXES.iter().enumerate() {
            for face in 0..self.face_masks[a].len() {
                let variable = self.face_variables[a][face];
                self.candidate.face_fields[a][face] = if variable == NONE {
                    0.0
                } else {
                    self.candidate.variable_values[variable]
                };
                self.candidate.volume_forcing[a][face] = if variable == NONE {
                    0.0
                } else {
                    analytic_sample(
                        self.config,
                        request,
                        face_point(
                            self.config,
                            a,
                            coordinate(self.grid.face_counts(AXES[a]), face),
                        ),
                    )?
                    .forcing[a]
                };
            }
        }
        let gauge_position = point(
            self.config,
            coordinate(self.config.counts, self.gauge),
            [false; 3],
        );
        let gauge_pressure = analytic_sample(self.config, request, gauge_position)?.pressure;
        self.candidate.pressure[self.gauge] = gauge_pressure;
        // A connected component's prescribed flux must balance before dropping
        // its redundant divergence row. Count the physical boundary terms only.
        let mut flux = 0.0;
        let mut flux_scale = 0.0;
        for (a, _) in AXES.iter().enumerate() {
            let area = self.face_area(a)?;
            for face in 0..self.face_masks[a].len() {
                if self.face_masks[a][face] != 2 {
                    continue;
                }
                let p = coordinate(self.grid.face_counts(AXES[a]), face);
                let sign = if p[a] == 0 {
                    -1.0
                } else if p[a] == self.config.counts[a] {
                    1.0
                } else {
                    let mut lower = p;
                    lower[a] -= 1;
                    if self.is_fluid(index(self.config.counts, lower)) {
                        1.0
                    } else {
                        -1.0
                    }
                };
                let t = mul(sign, mul(area, self.candidate.face_fields[a][face])?)?;
                flux = add(flux, t)?;
                flux_scale = add(flux_scale, t.abs())?;
            }
        }
        let flux_budget = mul(RESIDUAL_FACTOR, flux_scale)?;
        if flux.abs() > flux_budget {
            return Err(Error::IncompatibleFlux {
                defect: flux,
                budget: flux_budget,
            });
        }
        checkpoint(&mut cancel, Stage::Assembly, 0)?;
        self.assemble(&mut cancel)?;
        let mut min_scaled_pivot = self.factor(&mut cancel)?;
        checkpoint(&mut cancel, Stage::Reconstruction, 0)?;
        self.install_unknowns(false)?;
        // Residuals use the physical sparse functional and every divergence row,
        // independently of the overwritten dense elimination matrix.
        checkpoint(&mut cancel, Stage::Residual, 0)?;
        for pass in 0..=MAX_REFINEMENT_PASSES {
            match self.residuals(&mut cancel) {
                Ok(()) => break,
                Err(Error::Residual { .. }) if pass < MAX_REFINEMENT_PASSES => {
                    // Restore the original physical mixed equations, then solve
                    // A delta = b-A x. Strong pressure/forcing cancellation can
                    // leave tiny velocities with insufficient local continuity
                    // precision after one factorization. No acceptance budget
                    // changes: every corrected candidate passes the same sparse
                    // physical checks, including the eliminated gauge row.
                    self.assemble(&mut cancel)?;
                    for row in 0..self.nd {
                        checkpoint(&mut cancel, Stage::Assembly, row)?;
                        let mut residual = self.rhs[row];
                        for col in 0..self.nd {
                            let coefficient = self.matrix[row * self.nd + col];
                            if coefficient != 0.0 {
                                residual = add(residual, -mul(coefficient, self.unknowns[col])?)?;
                            }
                        }
                        self.rhs[row] = residual;
                    }
                    min_scaled_pivot = min_scaled_pivot.min(self.factor(&mut cancel)?);
                    checkpoint(&mut cancel, Stage::Reconstruction, pass + 1)?;
                    self.install_unknowns(true)?;
                }
                Err(error) => return Err(error),
            }
        }
        self.reconstruct_walls(&mut cancel)?;
        checkpoint(&mut cancel, Stage::Loads, 0)?;
        self.loads(&mut cancel)?;
        self.candidate.report.solve_id = solve_id;
        self.candidate.report.actual_dofs = self.nd;
        self.candidate.report.active_velocity_dofs = self.nv;
        self.candidate.report.nongauge_pressure_dofs = self.nd - self.nv;
        self.candidate.report.min_scaled_pivot = min_scaled_pivot;
        self.candidate.report.retained_payload_bytes = self.payload;
        checkpoint(&mut cancel, Stage::Commit, 0)?;
        std::mem::swap(&mut self.candidate, &mut self.accepted);
        self.has_accepted = true;
        Ok(&self.accepted)
    }
    fn install_unknowns(&mut self, additive: bool) -> Result<(), Error> {
        for (a, _) in AXES.iter().enumerate() {
            for face in 0..self.face_masks[a].len() {
                let variable = self.face_variables[a][face];
                if variable < self.nv {
                    let value = if additive {
                        add(
                            self.candidate.variable_values[variable],
                            self.unknowns[variable],
                        )?
                    } else {
                        self.unknowns[variable]
                    };
                    self.candidate.variable_values[variable] = value;
                    self.candidate.face_fields[a][face] = value;
                    self.unknowns[variable] = value;
                }
            }
        }
        for cell in 0..self.grid.cell_len() {
            let pi = self.pressure_unknowns[cell];
            if pi != NONE {
                let value = if additive {
                    add(self.candidate.pressure[cell], self.unknowns[pi])?
                } else {
                    self.unknowns[pi]
                };
                self.candidate.pressure[cell] = value;
                self.unknowns[pi] = value;
            }
        }
        Ok(())
    }
    fn assemble(&mut self, cancel: &mut impl FnMut(Stage, usize) -> bool) -> Result<(), Error> {
        let n = self.nd;
        self.matrix.fill(0.0);
        self.rhs.fill(0.0);
        for (ri, row) in self.rows.iter().enumerate() {
            checkpoint(cancel, Stage::Assembly, ri)?;
            for ti in row.terms() {
                if ti.variable >= self.nv {
                    continue;
                }
                for tj in row.terms() {
                    let coefficient =
                        mul(row.hessian_weight, mul(ti.coefficient, tj.coefficient)?)?;
                    if tj.variable < self.nv {
                        let entry = ti.variable * n + tj.variable;
                        self.matrix[entry] = add(self.matrix[entry], coefficient)?;
                    } else {
                        self.rhs[ti.variable] = add(
                            self.rhs[ti.variable],
                            -mul(coefficient, self.candidate.variable_values[tj.variable])?,
                        )?;
                    }
                }
            }
        }
        for cell in 0..self.grid.cell_len() {
            if !self.is_fluid(cell) {
                continue;
            }
            let p = coordinate(self.config.counts, cell);
            let pi = self.pressure_unknowns[cell];
            for (a, _) in AXES.iter().enumerate() {
                let area = self.face_area(a)?;
                for (sign, fp) in [
                    (-1.0, p),
                    (1.0, {
                        let mut q = p;
                        q[a] += 1;
                        q
                    }),
                ] {
                    let variable =
                        self.face_variables[a][index(self.grid.face_counts(AXES[a]), fp)];
                    let b = mul(sign, area)?;
                    if variable < self.nv {
                        if pi != NONE {
                            self.matrix[variable * n + pi] =
                                add(self.matrix[variable * n + pi], -b)?;
                            self.matrix[pi * n + variable] =
                                add(self.matrix[pi * n + variable], b)?;
                        } else {
                            self.rhs[variable] = add(
                                self.rhs[variable],
                                mul(b, self.candidate.pressure[self.gauge])?,
                            )?;
                        }
                    } else if pi != NONE {
                        self.rhs[pi] = add(
                            self.rhs[pi],
                            -mul(b, self.candidate.variable_values[variable])?,
                        )?;
                    }
                }
            }
        }
        for (a, _) in AXES.iter().enumerate() {
            for face in 0..self.face_masks[a].len() {
                let variable = self.face_variables[a][face];
                if variable < self.nv {
                    self.rhs[variable] = add(
                        self.rhs[variable],
                        mul(
                            self.face_dual_volumes[a][face],
                            self.candidate.volume_forcing[a][face],
                        )?,
                    )?;
                }
            }
        }
        Ok(())
    }
    fn factor(&mut self, cancel: &mut impl FnMut(Stage, usize) -> bool) -> Result<f64, Error> {
        checkpoint(cancel, Stage::Factorization, 0)?;
        let n = self.nd;
        let pressure_scale = div(
            mul(self.config.viscosity, self.config.u_ref)?,
            self.config.l_ref,
        )?;
        positive(pressure_scale)?;
        for col in 0..n {
            self.column_scales[col] = if col < self.nv {
                self.config.u_ref
            } else {
                pressure_scale
            };
        }
        for row in 0..n {
            let mut scale = 0.0f64;
            for col in 0..n {
                let e = row * n + col;
                self.matrix[e] = mul(self.matrix[e], self.column_scales[col])?;
                scale = scale.max(self.matrix[e].abs());
            }
            if scale == 0.0 {
                return Err(Error::Conditioning {
                    column: row,
                    scaled_pivot: 0.0,
                });
            }
            for col in 0..n {
                let e = row * n + col;
                self.matrix[e] = div(self.matrix[e], scale)?;
            }
            self.rhs[row] = div(self.rhs[row], scale)?;
        }
        let mut minimum = f64::INFINITY;
        for k in 0..n {
            checkpoint(cancel, Stage::Factorization, k)?;
            let mut pivot = k;
            for row in k + 1..n {
                if self.matrix[row * n + k].abs() > self.matrix[pivot * n + k].abs() {
                    pivot = row;
                }
            }
            let magnitude = self.matrix[pivot * n + k].abs();
            if magnitude < PIVOT_FLOOR {
                return Err(Error::Conditioning {
                    column: k,
                    scaled_pivot: magnitude,
                });
            }
            minimum = minimum.min(magnitude);
            if pivot != k {
                for col in 0..n {
                    self.matrix.swap(k * n + col, pivot * n + col);
                }
                self.rhs.swap(k, pivot);
            }
            let diagonal = self.matrix[k * n + k];
            for row in k + 1..n {
                let factor = div(self.matrix[row * n + k], diagonal)?;
                self.matrix[row * n + k] = 0.0;
                if factor == 0.0 {
                    continue;
                }
                for col in k + 1..n {
                    let e = row * n + col;
                    self.matrix[e] = add(self.matrix[e], -mul(factor, self.matrix[k * n + col])?)?;
                }
                self.rhs[row] = add(self.rhs[row], -mul(factor, self.rhs[k])?)?;
            }
        }
        checkpoint(cancel, Stage::BackSubstitution, 0)?;
        for row in (0..n).rev() {
            checkpoint(cancel, Stage::BackSubstitution, row)?;
            let mut value = self.rhs[row];
            for col in row + 1..n {
                value = add(value, -mul(self.matrix[row * n + col], self.unknowns[col])?)?;
            }
            self.unknowns[row] = div(value, self.matrix[row * n + row])?;
        }
        for col in 0..n {
            self.unknowns[col] = mul(self.unknowns[col], self.column_scales[col])?;
        }
        Ok(minimum)
    }
}

impl Solver<'_> {
    fn residuals(&mut self, cancel: &mut impl FnMut(Stage, usize) -> bool) -> Result<(), Error> {
        self.gradients.fill(0.0);
        self.magnitudes.fill(0.0);
        let mut dissipation = 0.0;
        for (ri, row) in self.rows.iter().enumerate() {
            checkpoint(cancel, Stage::Residual, ri)?;
            let mut strain = 0.0;
            for ti in row.terms() {
                strain = add(
                    strain,
                    mul(ti.coefficient, self.candidate.variable_values[ti.variable])?,
                )?;
            }
            dissipation = add(dissipation, mul(row.hessian_weight, mul(strain, strain)?)?)?;
            for ti in row.terms() {
                for tj in row.terms() {
                    let t = mul(
                        mul(row.hessian_weight, mul(ti.coefficient, tj.coefficient)?)?,
                        self.candidate.variable_values[tj.variable],
                    )?;
                    self.gradients[ti.variable] = add(self.gradients[ti.variable], t)?;
                    self.magnitudes[ti.variable] = add(self.magnitudes[ti.variable], t.abs())?;
                }
            }
        }
        let mut max_divergence = 0.0f64;
        let mut max_ratio = 0.0f64;
        for cell in 0..self.grid.cell_len() {
            if !self.is_fluid(cell) {
                continue;
            }
            checkpoint(cancel, Stage::Residual, cell)?;
            let p = coordinate(self.config.counts, cell);
            let pressure = self.candidate.pressure[cell];
            let mut divergence = 0.0;
            let mut divergence_scale = 0.0;
            for (a, _) in AXES.iter().enumerate() {
                let area = self.face_area(a)?;
                for (sign, fp) in [
                    (-1.0, p),
                    (1.0, {
                        let mut q = p;
                        q[a] += 1;
                        q
                    }),
                ] {
                    let variable =
                        self.face_variables[a][index(self.grid.face_counts(AXES[a]), fp)];
                    let b = mul(sign, area)?;
                    let t = mul(b, self.candidate.variable_values[variable])?;
                    divergence = add(divergence, t)?;
                    divergence_scale = add(divergence_scale, t.abs())?;
                    let pt = -mul(b, pressure)?;
                    self.gradients[variable] = add(self.gradients[variable], pt)?;
                    self.magnitudes[variable] = add(self.magnitudes[variable], pt.abs())?;
                }
            }
            let budget = mul(RESIDUAL_FACTOR, divergence_scale)?;
            if divergence.abs() > budget {
                return Err(Error::Residual {
                    row: self.nv + cell,
                    defect: divergence,
                    budget,
                });
            }
            max_divergence = max_divergence.max(divergence.abs());
            if budget > 0.0 {
                max_ratio = max_ratio.max(divergence.abs() / budget);
            }
        }
        for (a, _) in AXES.iter().enumerate() {
            for face in 0..self.face_masks[a].len() {
                let variable = self.face_variables[a][face];
                if variable == NONE {
                    continue;
                }
                let forcing = mul(
                    self.face_dual_volumes[a][face],
                    self.candidate.volume_forcing[a][face],
                )?;
                self.gradients[variable] = add(self.gradients[variable], -forcing)?;
                self.magnitudes[variable] = add(self.magnitudes[variable], forcing.abs())?;
            }
        }
        let mut max_momentum = 0.0f64;
        for i in 0..self.nv {
            let budget = mul(RESIDUAL_FACTOR, self.magnitudes[i])?;
            let defect = self.gradients[i];
            if defect.abs() > budget {
                return Err(Error::Residual {
                    row: i,
                    defect,
                    budget,
                });
            }
            max_momentum = max_momentum.max(defect.abs());
            if budget > 0.0 {
                max_ratio = max_ratio.max(defect.abs() / budget);
            }
        }
        for (i, p) in self.candidate.prescribed.iter().enumerate() {
            self.candidate.reactions[i] = self.gradients[p.variable];
        }
        self.candidate.report.dissipation = dissipation;
        self.candidate.report.max_momentum_defect = max_momentum;
        self.candidate.report.max_divergence_defect = max_divergence;
        self.candidate.report.max_residual_ratio = max_ratio;
        Ok(())
    }
    fn reconstruct_walls(
        &mut self,
        cancel: &mut impl FnMut(Stage, usize) -> bool,
    ) -> Result<(), Error> {
        for i in 0..self.candidate.wall_patches.len() {
            checkpoint(cancel, Stage::Reconstruction, i)?;
            let mut patch = self.candidate.wall_patches[i];
            let a = patch.axis;
            let side = patch.side;
            let near = coordinate(self.config.counts, patch.cell);
            let mut next = near;
            next[a] = next[a]
                .checked_add_signed(side as isize)
                .ok_or(Error::UnsupportedGeometry)?;
            if !fluid(self.config.counts, self.body_planes, next) {
                return Err(Error::UnsupportedGeometry);
            }
            patch.pressure = div(
                add(
                    mul(3.0, self.candidate.pressure[patch.cell])?,
                    -self.candidate.pressure[index(self.config.counts, next)],
                )?,
                2.0,
            )?;
            patch.gradient = [[0.0; 3]; 3];
            for component in 0..3 {
                let (sample1, sample2) = if component == a {
                    let mut boundary = near;
                    if side < 0 {
                        boundary[a] += 1;
                    }
                    let mut p1 = boundary;
                    let mut p2 = boundary;
                    p1[a] = p1[a]
                        .checked_add_signed(side as isize)
                        .ok_or(Error::UnsupportedGeometry)?;
                    p2[a] = p2[a]
                        .checked_add_signed(2 * side as isize)
                        .ok_or(Error::UnsupportedGeometry)?;
                    let shape = self.grid.face_counts(AXES[component]);
                    if p1[a] >= shape[a] || p2[a] >= shape[a] {
                        return Err(Error::UnsupportedGeometry);
                    }
                    (
                        self.candidate.face_fields[component][index(shape, p1)],
                        self.candidate.face_fields[component][index(shape, p2)],
                    )
                } else {
                    let centered = |cell: [usize; 3]| -> Result<f64, Error> {
                        let mut upper = cell;
                        upper[component] += 1;
                        let shape = self.grid.face_counts(AXES[component]);
                        div(
                            add(
                                self.candidate.face_fields[component][index(shape, cell)],
                                self.candidate.face_fields[component][index(shape, upper)],
                            )?,
                            2.0,
                        )
                    };
                    (centered(near)?, centered(next)?)
                };
                let derivative = if component == a {
                    div(
                        add(
                            add(-mul(3.0, patch.velocity[component])?, mul(4.0, sample1)?)?,
                            -sample2,
                        )?,
                        mul(2.0, self.config.spacing[a])?,
                    )?
                } else {
                    div(
                        add(
                            add(mul(9.0, sample1)?, -sample2)?,
                            -mul(8.0, patch.velocity[component])?,
                        )?,
                        mul(3.0, self.config.spacing[a])?,
                    )?
                };
                patch.gradient[component][a] = mul(side as f64, derivative)?;
            }
            self.candidate.wall_patches[i] = patch;
        }
        Ok(())
    }
    fn loads(&mut self, cancel: &mut impl FnMut(Stage, usize) -> bool) -> Result<(), Error> {
        let mut loads = Loads::default();
        let reference = self.candidate.request.moment_reference;
        for (i, p) in self.candidate.wall_patches.iter().enumerate() {
            checkpoint(cancel, Stage::Loads, i)?;
            let l = integrate_patch(p, self.config.viscosity, reference)?;
            for (a, _) in AXES.iter().enumerate() {
                loads.pressure_force[a] = add(loads.pressure_force[a], l.pressure_force[a])?;
                loads.shear_force[a] = add(loads.shear_force[a], l.shear_force[a])?;
                loads.pressure_moment[a] = add(loads.pressure_moment[a], l.pressure_moment[a])?;
                loads.shear_moment[a] = add(loads.shear_moment[a], l.shear_moment[a])?;
            }
            loads.wall_work = add(loads.wall_work, l.work)?;
        }
        let mut boundary_work = 0.0;
        let mut work_scale = self.candidate.report.dissipation;
        let mut body_work = 0.0;
        let mut outer_work = 0.0;
        for (i, p) in self.candidate.prescribed.iter().enumerate() {
            checkpoint(cancel, Stage::Loads, i)?;
            let reaction = self.candidate.reactions[i];
            let mut force = [0.0; 3];
            force[p.component] = match p.owner {
                Owner::Body => -reaction,
                Owner::Outer { .. } => reaction,
            };
            let arm = std::array::from_fn(|a| p.point[a] - reference[a]);
            let moment = cross(arm, force)?;
            let work = mul(reaction, p.value)?;
            boundary_work = add(boundary_work, work)?;
            work_scale = add(work_scale, mul(self.magnitudes[p.variable], p.value.abs())?)?;
            match p.owner {
                Owner::Body => {
                    for (a, _) in AXES.iter().enumerate() {
                        loads.body_force[a] = add(loads.body_force[a], force[a])?;
                        loads.body_moment[a] = add(loads.body_moment[a], moment[a])?;
                    }
                    body_work = add(body_work, work)?;
                }
                Owner::Outer { .. } => {
                    for (a, _) in AXES.iter().enumerate() {
                        loads.outer_force[a] = add(loads.outer_force[a], force[a])?;
                        loads.outer_moment[a] = add(loads.outer_moment[a], moment[a])?;
                    }
                    outer_work = add(outer_work, work)?;
                }
            }
        }
        loads.outer_work = outer_work;
        let mut forcing_work = 0.0;
        for (a, _) in AXES.iter().enumerate() {
            for face in 0..self.face_masks[a].len() {
                if self.face_masks[a][face] == 0 {
                    continue;
                }
                let term = mul(
                    mul(
                        self.face_dual_volumes[a][face],
                        self.candidate.volume_forcing[a][face],
                    )?,
                    self.candidate.face_fields[a][face],
                )?;
                forcing_work = add(forcing_work, term)?;
                work_scale = add(work_scale, term.abs())?;
            }
        }
        // v^T(Kv-B^Tp-F)=2Phi-Fwork after every physical divergence row
        // (including the eliminated gauge row) has passed its residual gate.
        let defect = add(
            add(self.candidate.report.dissipation, -forcing_work)?,
            -boundary_work,
        )?;
        let budget = mul(RESIDUAL_FACTOR, work_scale)?;
        if defect.abs() > budget {
            return Err(Error::Residual {
                row: self.nvars,
                defect,
                budget,
            });
        }
        self.candidate.report.body_work = body_work;
        self.candidate.report.outer_work = outer_work;
        self.candidate.report.forcing_work = forcing_work;
        self.candidate.report.work_defect = defect;
        self.candidate.report.work_budget = budget;
        self.candidate.loads = loads;
        Ok(())
    }
}
