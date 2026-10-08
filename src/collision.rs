//! Immutable, two-sided triangle surfaces. Queries do not infer solid volume,
//! change fluid topology, or implement moving-wall response.
use std::{fmt, mem::size_of};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceStamp {
    /// Caller-owned object identity; uniqueness is the caller's responsibility.
    pub id: u64,
    pub version: u64,
}
#[derive(Debug, Clone, Copy)]
pub struct SurfaceSettings {
    /// Dimensionless conditioning/ambiguity threshold, in [32*EPSILON, 1e-3].
    pub relative_tolerance: f64,
    /// Retained Vec-capacity payload only, not RSS or constructor input allocation.
    pub memory_limit: usize,
}
impl Default for SurfaceSettings {
    fn default() -> Self {
        Self {
            relative_tolerance: 1e-12,
            memory_limit: 16 * 1024 * 1024,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum SurfaceError {
    InvalidSettings,
    EmptySurface,
    NonFiniteVertex { vertex: usize },
    InvalidTriangleIndex { triangle: usize, vertex: usize },
    DegenerateTriangle { triangle: usize },
    UnrepresentableGeometry { triangle: usize },
    BufferLimit { required: usize, limit: usize },
    CapacityOverflow,
    InvalidSegment,
    ArithmeticFailure,
    AmbiguousIntersection { triangle: usize },
    Cancelled { triangle: usize },
}
impl fmt::Display for SurfaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "surface query rejected operation: {self:?}")
    }
}
impl std::error::Error for SurfaceError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitFacing {
    /// Segment travels against the winding-derived normal. Not a solid-entry claim.
    Front,
    Back,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceHit {
    pub surface: SurfaceStamp,
    pub triangle: usize,
    /// Parameter on start + t*(end-start), in [0,1].
    pub parameter: f64,
    pub position: [f64; 3],
    /// Weights for the three indexed vertices; finite and nonnegative.
    pub barycentric: [f64; 3],
    /// Unit geometric normal from triangle winding, not a rendering normal.
    pub normal: [f64; 3],
    pub facing: HitFacing,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClippedSegment {
    pub end: [f64; 3],
    pub hit: Option<SurfaceHit>,
}

/// Owned world-space indexed triangles, immutable after admission. Open,
/// disconnected, duplicate and intersecting surfaces are allowed: there is
/// deliberately no watertightness, manifoldness or inside/outside assertion.
#[derive(Debug)]
pub struct TriangleSurface {
    stamp: SurfaceStamp,
    vertices: Vec<[f64; 3]>,
    triangles: Vec<[usize; 3]>,
    tolerance: f64,
    allocated_bytes: usize,
}
impl TriangleSurface {
    /// Consumes the caller's buffers even on rejection; does not allocate or
    /// mutate any previously admitted surface or simulation state.
    pub fn new(
        stamp: SurfaceStamp,
        vertices: Vec<[f64; 3]>,
        triangles: Vec<[usize; 3]>,
        settings: SurfaceSettings,
    ) -> Result<Self, SurfaceError> {
        let tolerance = settings.relative_tolerance;
        if !tolerance.is_finite() || !(32.0 * f64::EPSILON..=1e-3).contains(&tolerance) {
            return Err(SurfaceError::InvalidSettings);
        }
        let allocated_bytes = vertices
            .capacity()
            .checked_mul(size_of::<[f64; 3]>())
            .and_then(|n| {
                triangles
                    .capacity()
                    .checked_mul(size_of::<[usize; 3]>())
                    .and_then(|t| n.checked_add(t))
            })
            .ok_or(SurfaceError::CapacityOverflow)?;
        if allocated_bytes > settings.memory_limit {
            return Err(SurfaceError::BufferLimit {
                required: allocated_bytes,
                limit: settings.memory_limit,
            });
        }
        if vertices.is_empty() || triangles.is_empty() {
            return Err(SurfaceError::EmptySurface);
        }
        for (vertex, p) in vertices.iter().enumerate() {
            if !finite(*p) {
                return Err(SurfaceError::NonFiniteVertex { vertex });
            }
        }
        for (triangle, indices) in triangles.iter().enumerate() {
            for &vertex in indices {
                if vertex >= vertices.len() {
                    return Err(SurfaceError::InvalidTriangleIndex { triangle, vertex });
                }
            }
            facet(&vertices, *indices, tolerance, triangle)?;
        }
        Ok(Self {
            stamp,
            vertices,
            triangles,
            tolerance,
            allocated_bytes,
        })
    }
    pub fn stamp(&self) -> SurfaceStamp {
        self.stamp
    }
    pub fn vertices(&self) -> &[[f64; 3]] {
        &self.vertices
    }
    pub fn triangles(&self) -> &[[usize; 3]] {
        &self.triangles
    }
    pub fn allocated_bytes(&self) -> usize {
        self.allocated_bytes
    }
    /// Exact conditioning threshold retained at admission; pose regeneration
    /// must reuse this value rather than silently restoring a default.
    pub fn relative_tolerance(&self) -> f64 {
        self.tolerance
    }

    /// Scan all facets, without heap allocation. Exact parameter ties retain
    /// the lowest triangle index. An ambiguous candidate rejects the query,
    /// even if another facet has a definite hit: earliest contact is unresolved.
    /// Cancellation polls once per facet and exposes no partial hit result.
    pub fn first_hit(
        &self,
        start: [f64; 3],
        end: [f64; 3],
        mut cancelled: impl FnMut(usize) -> bool,
    ) -> Result<Option<SurfaceHit>, SurfaceError> {
        let direction = sub(end, start);
        if !finite(start) || !finite(end) || !finite(direction) || scale(direction) == 0.0 {
            return Err(SurfaceError::InvalidSegment);
        }
        let mut best: Option<SurfaceHit> = None;
        for (triangle, &indices) in self.triangles.iter().enumerate() {
            if cancelled(triangle) {
                return Err(SurfaceError::Cancelled { triangle });
            }
            let f = facet(&self.vertices, indices, self.tolerance, triangle)?;
            if !segment_overlaps(start, end, f.lower, f.upper) {
                continue;
            }
            if let Some(hit) = intersect(
                start,
                end,
                direction,
                &f,
                self.tolerance,
                triangle,
                self.stamp,
            )? && best.is_none_or(|old| hit.parameter < old.parameter)
            {
                best = Some(hit);
            }
        }
        Ok(best)
    }
    /// End a nonzero segment at its first surface hit, or preserve its end on
    /// a definite miss. This is geometric clipping, not a boundary response.
    pub fn clip_segment(
        &self,
        start: [f64; 3],
        end: [f64; 3],
        cancelled: impl FnMut(usize) -> bool,
    ) -> Result<ClippedSegment, SurfaceError> {
        let hit = self.first_hit(start, end, cancelled)?;
        Ok(ClippedSegment {
            end: hit.map_or(end, |h| h.position),
            hit,
        })
    }
}

struct Facet {
    origin: [f64; 3],
    e1: [f64; 3],
    e2: [f64; 3],
    scale: f64,
    normal: [f64; 3],
    lower: [f64; 3],
    upper: [f64; 3],
}
fn facet(
    vertices: &[[f64; 3]],
    indices: [usize; 3],
    tolerance: f64,
    triangle: usize,
) -> Result<Facet, SurfaceError> {
    let [a, b, c] = indices.map(|i| vertices[i]);
    let (e1, e2) = (sub(b, a), sub(c, a));
    let length = scale(e1).max(scale(e2));
    if !finite(e1) || !finite(e2) || !length.is_finite() {
        return Err(SurfaceError::UnrepresentableGeometry { triangle });
    }
    if length == 0.0 {
        return Err(SurfaceError::DegenerateTriangle { triangle });
    }
    let (e1, e2) = (div(e1, length), div(e2, length));
    let n = cross(e1, e2);
    let area = norm(n);
    if area <= tolerance {
        return Err(SurfaceError::DegenerateTriangle { triangle });
    }
    Ok(Facet {
        origin: a,
        e1,
        e2,
        scale: length,
        normal: div(n, area),
        lower: std::array::from_fn(|d| a[d].min(b[d]).min(c[d])),
        upper: std::array::from_fn(|d| a[d].max(b[d]).max(c[d])),
    })
}
fn intersect(
    start: [f64; 3],
    end: [f64; 3],
    direction: [f64; 3],
    f: &Facet,
    tolerance: f64,
    triangle: usize,
    stamp: SurfaceStamp,
) -> Result<Option<SurfaceHit>, SurfaceError> {
    let offset = sub(f.origin, start);
    if !finite(offset) {
        return Err(SurfaceError::ArithmeticFailure);
    }
    let length = scale(direction).max(scale(offset));
    let d = div(direction, length);
    let denominator = dot(f.normal, d);
    if denominator.abs() <= tolerance * norm(d) {
        return Err(SurfaceError::AmbiguousIntersection { triangle });
    }
    let t = dot(f.normal, div(offset, length)) / denominator;
    if !t.is_finite() {
        return Err(SurfaceError::ArithmeticFailure);
    }
    if t < -tolerance || t > 1.0 + tolerance {
        return Ok(None);
    }
    if uncertain_unit(t, tolerance) {
        return Err(SurfaceError::AmbiguousIntersection { triangle });
    }
    let position = std::array::from_fn(|i| (1.0 - t) * start[i] + t * end[i]);
    if !finite(position) {
        return Err(SurfaceError::ArithmeticFailure);
    }
    let mut drop = 0;
    for i in 1..3 {
        if f.normal[i].abs() > f.normal[drop].abs() {
            drop = i;
        }
    }
    for (i, &coordinate) in position.iter().enumerate() {
        if i == drop {
            continue;
        }
        let outside = (f.lower[i] - coordinate).max(coordinate - f.upper[i]);
        if outside > tolerance * f.scale {
            return Ok(None);
        }
        if outside > 0.0 {
            return Err(SurfaceError::AmbiguousIntersection { triangle });
        }
    }
    let q = div(sub(position, f.origin), f.scale);
    if !finite(q) {
        return Err(SurfaceError::ArithmeticFailure);
    }
    // Drop the dominant normal axis: the 2D determinant is the largest
    // component of the already conditioned cross product, avoiding Gram loss.
    let (i, j) = ((drop + 1) % 3, (drop + 2) % 3);
    let area = f.e1[i] * f.e2[j] - f.e1[j] * f.e2[i];
    let u = (q[i] * f.e2[j] - q[j] * f.e2[i]) / area;
    let v = (f.e1[i] * q[j] - f.e1[j] * q[i]) / area;
    let w = 1.0 - (u + v);
    if !u.is_finite() || !v.is_finite() || !w.is_finite() {
        return Err(SurfaceError::ArithmeticFailure);
    }
    if u < -tolerance || v < -tolerance || w < -tolerance {
        return Ok(None);
    }
    if [u, v, w].into_iter().any(|x| uncertain_unit(x, tolerance)) {
        return Err(SurfaceError::AmbiguousIntersection { triangle });
    }
    // Projected barycentrics do not validate the dropped axis. Large segment
    // interpolation can lose the plane offset; reject an inconsistent full
    // reconstruction in the same dimensionless facet tolerance.
    let reconstructed = std::array::from_fn(|i| u * f.e1[i] + v * f.e2[i]);
    if scale(sub(q, reconstructed)) > tolerance {
        return Err(SurfaceError::AmbiguousIntersection { triangle });
    }
    Ok(Some(SurfaceHit {
        surface: stamp,
        triangle,
        parameter: t,
        position,
        barycentric: [w, u, v],
        normal: f.normal,
        facing: if denominator < 0.0 {
            HitFacing::Front
        } else {
            HitFacing::Back
        },
    }))
}
fn uncertain_unit(x: f64, tolerance: f64) -> bool {
    !(0.0..=1.0).contains(&x) || (x > 0.0 && x < tolerance) || (x < 1.0 && x > 1.0 - tolerance)
}
fn segment_overlaps(a: [f64; 3], b: [f64; 3], lower: [f64; 3], upper: [f64; 3]) -> bool {
    (0..3).all(|d| a[d].min(b[d]) <= upper[d] && a[d].max(b[d]) >= lower[d])
}
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|d| a[d] - b[d])
}
fn div(a: [f64; 3], b: f64) -> [f64; 3] {
    a.map(|v| v / b)
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn scale(a: [f64; 3]) -> f64 {
    a.into_iter().fold(0.0_f64, |m, v| m.max(v.abs()))
}
fn norm(a: [f64; 3]) -> f64 {
    a[0].hypot(a[1]).hypot(a[2])
}
fn finite(a: [f64; 3]) -> bool {
    a.into_iter().all(f64::is_finite)
}
