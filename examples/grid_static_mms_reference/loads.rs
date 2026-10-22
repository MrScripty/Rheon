//! New CV3-A load integration, leaving the old constant-stress trace rule intact.
use super::super::grid_cut_variational_reference::{
    assembly::{Matrix, Operators, zero},
    basis::{GAUSS_WEIGHTS, gauss_nodes},
    trace::Traces,
};
use super::field;
pub struct BoundaryPoint {
    pub record: usize,
    pub point: [f64; 2],
    pub normal_weight: [f64; 2],
    pub traction_weight: [f64; 2],
    pub basis: Vec<(usize, f64)>,
}
pub struct Loads {
    pub body: Vec<f64>,
    pub boundary: Vec<f64>,
    pub body_resultant: [f64; 3],
    pub boundary_resultant: [f64; 3],
    pub h1: Matrix,
    pub points: Vec<BoundaryPoint>,
}
impl Loads {
    pub fn new(op: &Operators, tr: &Traces) -> Result<Self, String> {
        let n = op.active.len();
        let mut out = Self {
            body: vec![0.0; n],
            boundary: vec![0.0; n],
            body_resultant: [0.0; 3],
            boundary_resultant: [0.0; 3],
            h1: zero(n, n),
            points: Vec::new(),
        };
        for q in &op.quadrature {
            let f = field::body(q.point);
            for (r, v) in out.body_resultant.iter_mut().zip([
                f[0],
                f[1],
                q.point[0] * f[1] - q.point[1] * f[0],
            ]) {
                *r += q.weight * field::DEPTH * v;
            }
            for a in &q.values {
                let i = op.map[a.global];
                out.body[i] += q.weight * field::DEPTH * a.phi * f[a.axis];
                for b in &q.values {
                    if a.axis == b.axis {
                        out.h1[i][op.map[b.global]] += q.weight
                            * field::DEPTH
                            * (a.dx * b.dx
                                + a.dy * b.dy
                                + a.phi * b.phi / (field::LENGTH * field::LENGTH));
                    }
                }
            }
        }
        for (record, r) in tr
            .records
            .iter()
            .enumerate()
            .filter(|(_, r)| r.kind == "global")
        {
            for (&t, &w) in gauss_nodes().iter().zip(&GAUSS_WEIGHTS) {
                let point =
                    std::array::from_fn(|d| r.segment.a[d] + t * (r.segment.b[d] - r.segment.a[d]));
                if point == r.segment.a || point == r.segment.b {
                    return Err("CV3 positive load integration point collapsed".into());
                }
                let normal_weight = r.segment.normal_measure().map(|v| v * w * field::DEPTH);
                let traction_weight = field::traction(point, normal_weight);
                for (t, v) in out.boundary_resultant.iter_mut().zip([
                    traction_weight[0],
                    traction_weight[1],
                    point[0] * traction_weight[1] - point[1] * traction_weight[0],
                ]) {
                    *t += v;
                }
                let mut basis = Vec::new();
                for (i, node) in op.active.iter().enumerate() {
                    let phi = op.layout.trace(node, point);
                    if phi != 0.0 {
                        basis.push((i, phi));
                    }
                    out.boundary[i] += phi * traction_weight[node.axis];
                }
                out.points.push(BoundaryPoint {
                    record,
                    point,
                    normal_weight,
                    traction_weight,
                    basis,
                });
            }
        }
        if out
            .body
            .iter()
            .chain(&out.boundary)
            .chain(&out.body_resultant)
            .chain(&out.boundary_resultant)
            .chain(out.h1.iter().flatten())
            .chain(out.points.iter().flat_map(|p| {
                p.point
                    .iter()
                    .chain(&p.normal_weight)
                    .chain(&p.traction_weight)
            }))
            .any(|x| !x.is_finite())
        {
            return Err("CV3 nonfinite physical loads or H1 map".into());
        }
        Ok(out)
    }
}
