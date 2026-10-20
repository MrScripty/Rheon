use super::super::{MU, RHO};
use super::{
    basis::{GAUSS_WEIGHTS, Layout, Node, Value, gauss_nodes},
    clip::{self, QPoint, Source, Triangle},
    geometry::{Point, Polygon},
};
use num_traits::ToPrimitive;
pub type Matrix = Vec<Vec<f64>>;
pub fn zero(n: usize, m: usize) -> Matrix {
    vec![vec![0.0; m]; n]
}
pub fn action(a: &Matrix, x: &[f64]) -> Vec<f64> {
    a.iter()
        .map(|r| r.iter().zip(x).map(|(v, x)| v * x).sum())
        .collect()
}
pub fn dot(x: &[f64], y: &[f64]) -> f64 {
    x.iter().zip(y).map(|(x, y)| x * y).sum()
}
pub struct Patch {
    pub i: usize,
    pub j: usize,
    pub cell: usize,
    pub lo: Point,
    pub hi: Point,
    pub polygon: Vec<QPoint>,
    pub triangles: Vec<Triangle>,
}
pub struct Quadrature {
    pub patch: usize,
    pub cell: usize,
    pub point: Point,
    pub weight: f64,
    pub values: Vec<Value>,
}
pub struct Operators {
    pub layout: Layout,
    pub source: Polygon,
    pub exact: Source,
    pub patches: Vec<Patch>,
    pub quadrature: Vec<Quadrature>,
    pub active: Vec<Node>,
    pub map: Vec<usize>,
    pub cells: Vec<usize>,
    pub cell_map: Vec<usize>,
    pub volume: Vec<f64>,
    pub old_dual: Vec<f64>,
    pub mass: Matrix,
    pub divergence: Matrix,
    pub stiffness: Matrix,
    pub gradient: Vec<[f64; 3]>,
    pub moment: Vec<[f64; 3]>,
}
pub fn strain(v: Value) -> [f64; 3] {
    if v.axis == 0 {
        [v.dx, 0.0, v.dy]
    } else {
        [0.0, v.dy, v.dx]
    }
}
pub fn integrate_triangle(t: &Triangle) -> Result<Vec<(Point, f64)>, String> {
    let nodes = gauss_nodes();
    let [a, b, c] = t.points;
    let mut out = Vec::with_capacity(9);
    for (i, &r) in nodes.iter().enumerate() {
        for (j, &s) in nodes.iter().enumerate() {
            let p =
                std::array::from_fn(|d| a[d] + r * ((1.0 - s) * (b[d] - a[d]) + s * (c[d] - a[d])));
            let w = t.cross * r * GAUSS_WEIGHTS[i] * GAUSS_WEIGHTS[j];
            if !w.is_finite() || w <= 0.0 || p.iter().any(|x| !x.is_finite()) {
                return Err("invalid positive Duffy quadrature".into());
            }
            out.push((p, w));
        }
    }
    Ok(out)
}
impl Operators {
    pub fn build(n: usize, origin: Point, extent: f64, source: Polygon) -> Result<Self, String> {
        let layout = Layout::new(n, origin, extent, &source)?;
        let exact = Source::new(&source)?;
        let mut patches = Vec::new();
        let mut quadrature = Vec::new();
        let mut active_flags = vec![false; layout.nodes.len()];
        let mut volume = vec![0.0; n * n];
        for j in 0..2 * n {
            for i in 0..2 * n {
                let (lo, hi) = layout.quarter_bounds(i, j);
                let polygon = exact.clip(lo, hi)?;
                let triangles = clip::triangles(&polygon)
                    .map_err(|e| format!("halfcell patch ({i},{j}) bounds {lo:?}..{hi:?}: {e}"))?;
                let cell = i / 2 + n * (j / 2);
                let id = patches.len();
                for t in &triangles {
                    for (point, weight) in integrate_triangle(t)? {
                        if (0..2).any(|d| point[d] <= lo[d] || point[d] >= hi[d]) {
                            return Err(format!(
                                "patch({i},{j}) positive integration point collapsed to knot boundary"
                            ));
                        }
                        let values = layout.local(i, j, point)?;
                        for v in &values {
                            active_flags[v.global] = true;
                        }
                        volume[cell] += weight;
                        quadrature.push(Quadrature {
                            patch: id,
                            cell,
                            point,
                            weight,
                            values,
                        });
                    }
                }
                patches.push(Patch {
                    i,
                    j,
                    cell,
                    lo,
                    hi,
                    polygon,
                    triangles,
                });
            }
        }
        let active: Vec<_> = layout
            .nodes
            .iter()
            .filter(|p| active_flags[p.global])
            .cloned()
            .collect();
        if active.is_empty() || active.len() > 544 {
            return Err("bounded positive active basis n1..544 required".into());
        }
        let mut map = vec![usize::MAX; layout.nodes.len()];
        for (i, p) in active.iter().enumerate() {
            map[p.global] = i;
        }
        let cells: Vec<_> = volume
            .iter()
            .enumerate()
            .filter_map(|(i, v)| (*v > 0.0).then_some(i))
            .collect();
        let mut cell_map = vec![usize::MAX; n * n];
        for (i, &c) in cells.iter().enumerate() {
            cell_map[c] = i;
        }
        let nv = active.len();
        nv.checked_mul(nv)
            .filter(|&entries| entries <= 544 * 544)
            .ok_or("private dense allocation ceiling")?;
        if cells.len() > 256 {
            return Err("bounded pressure rows n1..256 required".into());
        }
        let mut mass = zero(nv, nv);
        let mut divergence = zero(cells.len(), nv);
        let mut stiffness = zero(nv, nv);
        let mut gradient = vec![[0.0; 3]; nv];
        let mut moment = vec![[0.0; 3]; nv];
        for q in &quadrature {
            let pressure = cell_map[q.cell];
            for &a in &q.values {
                let i = map[a.global];
                divergence[pressure][i] += q.weight * if a.axis == 0 { a.dx } else { a.dy };
                if a.axis == 0 {
                    gradient[i][0] += q.weight * a.dx;
                    gradient[i][1] += q.weight * a.dy;
                    moment[i][0] += RHO * q.weight * a.phi;
                    moment[i][2] -= RHO * q.weight * q.point[1] * a.phi;
                } else {
                    gradient[i][1] += q.weight * a.dx;
                    gradient[i][2] += q.weight * a.dy;
                    moment[i][1] += RHO * q.weight * a.phi;
                    moment[i][2] += RHO * q.weight * q.point[0] * a.phi;
                }
                let sa = strain(a);
                for &b in &q.values {
                    let j = map[b.global];
                    let sb = strain(b);
                    if a.axis == b.axis {
                        mass[i][j] += RHO * q.weight * a.phi * b.phi;
                    }
                    stiffness[i][j] +=
                        MU * q.weight * (2.0 * sa[0] * sb[0] + 2.0 * sa[1] * sb[1] + sa[2] * sb[2]);
                }
            }
        }
        if mass
            .iter()
            .enumerate()
            .any(|(i, r)| !r[i].is_finite() || r[i] <= 0.0)
            || mass
                .iter()
                .chain(&divergence)
                .chain(&stiffness)
                .flatten()
                .any(|x| !x.is_finite())
        {
            return Err("invalid full mass or bulk operator; no floor".into());
        }
        if volume
            .iter()
            .chain(gradient.iter().flatten())
            .chain(moment.iter().flatten())
            .any(|x| !x.is_finite())
        {
            return Err("nonfinite volume or moment map".into());
        }
        let old_dual = active
            .iter()
            .map(|node| {
                let lo = node.point.map(|x| x - layout.h / 2.0);
                let hi = node.point.map(|x| x + layout.h / 2.0);
                clip::area(&exact.clip(lo, hi)?)
                    .to_f64()
                    .ok_or_else(|| "old dual observation unrepresentable".to_string())
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(Self {
            layout,
            source,
            exact,
            patches,
            quadrature,
            active,
            map,
            cells,
            cell_map,
            volume,
            old_dual,
            mass,
            divergence,
            stiffness,
            gradient,
            moment,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::super::{clip::QPoint, exact::rational};
    use super::*;
    #[test]
    fn positive_triangle_rule_reproduces_polynomial_moments() {
        let v: Vec<QPoint> = vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]
            .into_iter()
            .map(|p| p.map(|x| rational(x).unwrap()))
            .collect();
        let triangles = clip::triangles(&v).unwrap();
        let q = integrate_triangle(&triangles[0]).unwrap();
        assert!(q.iter().all(|(_, w)| *w > 0.0));
        for (a, b, expected) in [
            (0, 0, 0.5),
            (1, 0, 1.0 / 6.0),
            (2, 2, 1.0 / 180.0),
            (4, 0, 1.0 / 30.0),
        ] {
            let actual: f64 = q.iter().map(|(p, w)| w * p[0].powi(a) * p[1].powi(b)).sum();
            assert!((actual - expected).abs() < 1e-14);
        }
    }
}
