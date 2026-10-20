//! Private planar common-geometry reference, version 3. No production DOF policy.
use rheon::GridGeometry;
pub type Point = [f64; 2];
#[derive(Clone, Copy, Debug, Default)]
pub struct Moments {
    pub area: f64,
    pub first: Point,
    pub second: [f64; 3], // xx, xy, yy
}
#[derive(Clone, Copy, Debug)]
pub struct Segment {
    pub a: Point,
    pub b: Point,
}
impl Segment {
    pub fn normal_measure(self) -> Point {
        [self.b[1] - self.a[1], self.a[0] - self.b[0]]
    }
    pub fn middle(self) -> Point {
        [(self.a[0] + self.b[0]) * 0.5, (self.a[1] + self.b[1]) * 0.5]
    }
    pub fn integrate(self, f: impl Fn(Point) -> Point) -> f64 {
        let mid = self.middle();
        let offset =
            std::array::from_fn::<_, 2, _>(|d| (self.b[d] - self.a[d]) / (2.0 * 3.0_f64.sqrt()));
        let left = f(std::array::from_fn(|d| mid[d] - offset[d]));
        let right = f(std::array::from_fn(|d| mid[d] + offset[d]));
        let normal = self.normal_measure();
        (0..2).map(|d| normal[d] * (left[d] + right[d]) * 0.5).sum()
    }
}
fn cross(a: Point, b: Point) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}
#[derive(Clone)]
pub struct Polygon {
    vertices: Vec<Point>,
    exact: super::exact::ExactPolygon,
}
impl Polygon {
    pub fn vertices(&self) -> &[Point] {
        &self.vertices
    }
    pub fn line(&self, axis: usize, position: f64) -> Result<super::exact::LineSection, String> {
        self.exact.line(axis, position)
    }
    pub fn new(vertices: Vec<Point>) -> Result<Self, String> {
        if vertices.len() < 3 || vertices.iter().flatten().any(|x| !x.is_finite()) {
            return Err("finite convex CCW polygon with at least three vertices required".into());
        }
        for (i, p) in vertices.iter().enumerate() {
            if vertices[..i].contains(p) {
                return Err("repeated source vertex or repeated winding unsupported".into());
            }
        }
        for (a, b) in edges(&vertices) {
            if a == b
                || vertices
                    .iter()
                    .any(|p| cross([b[0] - a[0], b[1] - a[1]], [p[0] - a[0], p[1] - a[1]]) < 0.0)
            {
                return Err("polygon must be convex CCW without repeated edges".into());
            }
        }
        if moments(&vertices)?.area <= 0.0 {
            return Err("positive polygon area required".into());
        }
        let exact = super::exact::ExactPolygon::new(&vertices)?;
        Ok(Self { vertices, exact })
    }
    pub fn clip(&self, lo: Point, hi: Point) -> Result<Vec<Point>, String> {
        if (0..2).any(|d| !lo[d].is_finite() || !hi[d].is_finite() || lo[d] >= hi[d]) {
            return Err("invalid clipping box".into());
        }
        let mut v = self.vertices.clone();
        for d in 0..2 {
            for (plane, sign) in [(lo[d], 1.0), (hi[d], -1.0)] {
                let mut next = Vec::new();
                for (a, b) in edges(&v) {
                    let inside_a = sign * (a[d] - plane) >= 0.0;
                    let inside_b = sign * (b[d] - plane) >= 0.0;
                    if inside_a != inside_b {
                        let t = (plane - a[d]) / (b[d] - a[d]);
                        let mut p = std::array::from_fn(|k| a[k] + t * (b[k] - a[k]));
                        p[d] = plane;
                        if next.last() != Some(&p) {
                            next.push(p);
                        }
                    }
                    if inside_b && next.last() != Some(&b) {
                        next.push(b);
                    }
                }
                if next.len() > 1 && next.first() == next.last() {
                    next.pop();
                }
                v = next;
            }
        }
        Ok(v)
    }
}
pub fn edges(v: &[Point]) -> impl Iterator<Item = (Point, Point)> + '_ {
    v.iter()
        .copied()
        .zip(v.iter().copied().cycle().skip(1))
        .take(v.len())
}
pub fn moments(v: &[Point]) -> Result<Moments, String> {
    if v.len() < 3 {
        return Ok(Moments::default());
    }
    // Accumulate about a polygon vertex. This avoids cancellation of large
    // global shoelace terms for small positive cut regions.
    let anchor = v[0];
    let local: Vec<Point> = v
        .iter()
        .map(|p| [p[0] - anchor[0], p[1] - anchor[1]])
        .collect();
    let mut m = Moments::default();
    for (a, b) in edges(&local) {
        let c = cross(a, b);
        if c < 0.0 {
            return Err("unresolved cut orientation; no sliver deletion permitted".into());
        }
        m.area += c / 2.0;
        for d in 0..2 {
            m.first[d] += (a[d] + b[d]) * c / 6.0;
        }
        m.second[0] += (a[0] * a[0] + a[0] * b[0] + b[0] * b[0]) * c / 12.0;
        m.second[1] +=
            (2.0 * a[0] * a[1] + a[0] * b[1] + b[0] * a[1] + 2.0 * b[0] * b[1]) * c / 24.0;
        m.second[2] += (a[1] * a[1] + a[1] * b[1] + b[1] * b[1]) * c / 12.0;
    }
    if m.area == 0.0 {
        // Exact coordinate equality certifies zero measure for represented
        // horizontal/vertical tangent polygons. No positive area tolerance.
        if (0..2).any(|d| v.iter().all(|p| p[d] == v[0][d])) {
            return Ok(Moments::default());
        }
        return Err("unresolved zero-area cut with three or more represented vertices".into());
    }
    m.second[0] += 2.0 * anchor[0] * m.first[0] + anchor[0] * anchor[0] * m.area;
    m.second[1] += anchor[0] * m.first[1] + anchor[1] * m.first[0] + anchor[0] * anchor[1] * m.area;
    m.second[2] += 2.0 * anchor[1] * m.first[1] + anchor[1] * anchor[1] * m.area;
    for (d, a) in anchor.into_iter().enumerate() {
        m.first[d] += a * m.area;
    }
    if !m.area.is_finite()
        || m.area < 0.0
        || m.first.iter().chain(&m.second).any(|x| !x.is_finite())
    {
        return Err("invalid polygon moments; no sliver floor permitted".into());
    }
    Ok(m)
}
fn clip_segment(s: Segment, lo: Point, hi: Point) -> Option<Segment> {
    let mut lower: f64 = 0.0;
    let mut upper: f64 = 1.0;
    for d in 0..2 {
        let delta = s.b[d] - s.a[d];
        if delta == 0.0 {
            if s.a[d] < lo[d] || s.a[d] > hi[d] {
                return None;
            }
        } else {
            let t0 = (lo[d] - s.a[d]) / delta;
            let t1 = (hi[d] - s.a[d]) / delta;
            lower = lower.max(t0.min(t1));
            upper = upper.min(t0.max(t1));
        }
    }
    if upper <= lower {
        return None;
    }
    let a = std::array::from_fn(|d| s.a[d] + lower * (s.b[d] - s.a[d]));
    let b = std::array::from_fn(|d| s.a[d] + upper * (s.b[d] - s.a[d]));
    Some(Segment { a, b })
}
pub struct BoundaryPiece {
    pub edge: usize,
    pub segment: Segment,
}
pub struct Region {
    pub moments: Moments,
    pub lo: Point,
    pub hi: Point,
}
impl Region {
    fn clip(source: &Polygon, lo: Point, hi: Point) -> Result<Self, String> {
        Ok(Self {
            moments: moments(&source.clip(lo, hi)?)?,
            lo,
            hi,
        })
    }
}
pub struct Cell {
    pub region: Region,
    pub physical: Vec<BoundaryPiece>,
    pub clipped: Vec<Point>,
}
pub struct Face {
    pub segment: Option<Segment>,
    pub dual: Region,
    pub exact: super::exact::ExactFace,
}
pub struct Geometry {
    pub grid: GridGeometry,
    pub source: Polygon,
    pub cells: Vec<Cell>,
    pub faces: [Vec<Face>; 2],
    pub shear: Vec<Region>,
}
impl Geometry {
    pub fn new(n: usize, origin: Point, extent: f64, source: Polygon) -> Result<Self, String> {
        if !(1..=128).contains(&n) || !extent.is_finite() || extent <= 0.0 {
            return Err("bounded reference grid n1..128 required".into());
        }
        let h = extent / n as f64;
        let grid = GridGeometry::new(
            [n as u64, n as u64, 1],
            [h, h, 1.0],
            [origin[0], origin[1], 0.0],
        )
        .map_err(|e| format!("{e:?}"))?;
        if source
            .vertices
            .iter()
            .any(|p| (0..2).any(|d| p[d] < origin[d] || p[d] > origin[d] + extent))
        {
            return Err("source must be contained by reference grid".into());
        }
        // A finite source edge on a primal grid plane needs an explicit ownership
        // policy. This bounded reference refuses it, rather than counting it twice.
        for (a, b) in edges(&source.vertices) {
            for d in 0..2 {
                if a[d] == b[d] && (0..=n).any(|k| a[d] == origin[d] + k as f64 * h) {
                    return Err(
                        "source edge coincident with primal grid plane unsupported in geometry reference"
                            .into(),
                    );
                }
            }
        }
        let mut cells = Vec::with_capacity(n * n);
        for j in 0..n {
            for i in 0..n {
                let lo = [origin[0] + i as f64 * h, origin[1] + j as f64 * h];
                let hi = [
                    origin[0] + (i + 1) as f64 * h,
                    origin[1] + (j + 1) as f64 * h,
                ];
                let clipped = source
                    .clip(lo, hi)
                    .map_err(|e| format!("cell ({i},{j}) clip: {e}"))?;
                let m = moments(&clipped).map_err(|e| format!("cell ({i},{j}) moments: {e}"))?;
                let physical = edges(&source.vertices)
                    .enumerate()
                    .filter_map(|(edge, (a, b))| {
                        clip_segment(Segment { a, b }, lo, hi)
                            .map(|segment| BoundaryPiece { edge, segment })
                    })
                    .collect();
                cells.push(Cell {
                    region: Region { moments: m, lo, hi },
                    physical,
                    clipped,
                });
            }
        }
        let mut faces: [Vec<Face>; 2] = std::array::from_fn(|_| Vec::with_capacity(n * (n + 1)));
        for (axis, collection) in faces.iter_mut().enumerate() {
            let lines: Vec<_> = (0..=n)
                .map(|i| source.line(axis, origin[axis] + i as f64 * h))
                .collect::<Result<_, _>>()?;
            let counts = grid.face_counts(if axis == 0 {
                rheon::Axis::X
            } else {
                rheon::Axis::Y
            });
            for y in 0..counts[1] {
                for x in 0..counts[0] {
                    let (i, j) = if axis == 0 { (x, y) } else { (y, x) };
                    let position = origin[axis] + i as f64 * h;
                    let lo = origin[1 - axis] + j as f64 * h;
                    let hi = origin[1 - axis] + (j + 1) as f64 * h;
                    let exact = lines[i].clip(lo, hi)?;
                    let segment = exact
                        .segment(axis, position)
                        .map_err(|e| format!("face axis{axis} ({x},{y}): {e}"))?;
                    let center = if axis == 0 {
                        [position, (lo + hi) / 2.0]
                    } else {
                        [(lo + hi) / 2.0, position]
                    };
                    let dual = Region::clip(
                        &source,
                        center.map(|x| x - h / 2.0),
                        center.map(|x| x + h / 2.0),
                    )
                    .map_err(|e| format!("face axis{axis} ({x},{y}) dual moments: {e}"))?;
                    collection.push(Face {
                        segment,
                        dual,
                        exact,
                    });
                }
            }
        }
        let mut shear = Vec::with_capacity((n + 1) * (n + 1));
        for j in 0..=n {
            for i in 0..=n {
                let c = [origin[0] + i as f64 * h, origin[1] + j as f64 * h];
                shear.push(
                    Region::clip(&source, c.map(|x| x - h / 2.0), c.map(|x| x + h / 2.0))
                        .map_err(|e| format!("shear ({i},{j}) moments: {e}"))?,
                );
            }
        }
        Ok(Self {
            grid,
            source,
            cells,
            faces,
            shear,
        })
    }
    pub fn cell_faces(&self, i: usize, j: usize) -> [(usize, usize, f64); 4] {
        let n = self.grid.counts()[0];
        [
            (0, i + (n + 1) * j, -1.0),
            (0, i + 1 + (n + 1) * j, 1.0),
            (1, i + n * j, -1.0),
            (1, i + n * (j + 1), 1.0),
        ]
    }
}
