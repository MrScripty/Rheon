use super::{
    assembly::{Matrix, Operators, zero},
    clip::{self, QPoint},
    exact::{ExactFace, rational},
    geometry::{Point, Segment},
};
#[derive(Clone)]
pub struct Record {
    pub kind: &'static str,
    pub owner: usize,
    pub axis: usize,
    pub edge: usize,
    pub segment: Segment,
}
pub struct TracePoint {
    pub record: usize,
    pub point: Point,
    pub normal_weight: Point,
    pub values: Vec<(usize, f64)>,
}
pub struct Traces {
    pub records: Vec<Record>,
    pub points: Vec<TracePoint>,
    pub gradient: Vec<[f64; 3]>,
    pub grid: Matrix,
    pub physical: Matrix,
}
impl Traces {
    pub fn build(op: &Operators) -> Result<Self, String> {
        let n = op.layout.n;
        let h = op.layout.h;
        let origin = op.layout.origin;
        let nv = op.active.len();
        let planes: [Vec<_>; 2] = std::array::from_fn(|d| {
            (0..=2 * n)
                .map(|i| rational(origin[d] + i as f64 * h / 2.0).unwrap())
                .collect()
        });
        let mut out = Self {
            records: Vec::new(),
            points: Vec::new(),
            gradient: vec![[0.0; 3]; nv],
            grid: zero(op.cells.len(), nv),
            physical: zero(op.cells.len(), nv),
        };
        for (edge, (a, b)) in op
            .exact
            .points
            .iter()
            .zip(op.exact.points.iter().cycle().skip(1))
            .take(op.exact.points.len())
            .enumerate()
        {
            for s in clip::split_segment(&[a.clone(), b.clone()], &planes)? {
                let (_, grad) = out.add(
                    op,
                    Record {
                        kind: "global",
                        owner: edge,
                        axis: 2,
                        edge,
                        segment: s,
                    },
                )?;
                for (t, g) in out.gradient.iter_mut().zip(grad) {
                    for d in 0..3 {
                        t[d] += g[d];
                    }
                }
            }
        }
        let mut faces: [Matrix; 2] = std::array::from_fn(|_| zero(n * (n + 1), nv));
        for (axis, rows) in faces.iter_mut().enumerate() {
            let lines = (0..=n)
                .map(|i| op.source.line(axis, origin[axis] + i as f64 * h))
                .collect::<Result<Vec<_>, _>>()?;
            for (id, row) in rows.iter_mut().enumerate() {
                let (normal, tangent) = if axis == 0 {
                    (id % (n + 1), id / (n + 1))
                } else {
                    (id / n, id % n)
                };
                let position = origin[axis] + normal as f64 * h;
                if let ExactFace::Positive { lower, upper } = lines[normal].clip(
                    origin[1 - axis] + tangent as f64 * h,
                    origin[1 - axis] + (tangent + 1) as f64 * h,
                )? {
                    let p = rational(position)?;
                    let mut a: QPoint = if axis == 0 {
                        [p.clone(), lower.clone()]
                    } else {
                        [lower.clone(), p.clone()]
                    };
                    let mut b = if axis == 0 { [p, upper] } else { [upper, p] };
                    if axis == 1 {
                        std::mem::swap(&mut a, &mut b);
                    }
                    for s in clip::split_segment(&[a, b], &planes)? {
                        let (flux, _) = out.add(
                            op,
                            Record {
                                kind: "grid",
                                owner: id,
                                axis,
                                edge: usize::MAX,
                                segment: s,
                            },
                        )?;
                        for (t, v) in row.iter_mut().zip(flux) {
                            *t += v;
                        }
                    }
                }
            }
        }
        for (c, &cell) in op.cells.iter().enumerate() {
            let i = cell % n;
            let j = cell / n;
            for (axis, face, sign) in [
                (0, i + (n + 1) * j, -1.0),
                (0, i + 1 + (n + 1) * j, 1.0),
                (1, i + n * j, -1.0),
                (1, i + n * (j + 1), 1.0),
            ] {
                for (t, v) in out.grid[c].iter_mut().zip(&faces[axis][face]) {
                    *t += sign * v;
                }
            }
            let lo = [origin[0] + i as f64 * h, origin[1] + j as f64 * h];
            let hi = [
                origin[0] + (i + 1) as f64 * h,
                origin[1] + (j + 1) as f64 * h,
            ];
            for (edge, (a, b)) in op
                .exact
                .points
                .iter()
                .zip(op.exact.points.iter().cycle().skip(1))
                .take(op.exact.points.len())
                .enumerate()
            {
                if let Some(segment) = clip::clipped_segment(a, b, lo, hi)? {
                    for s in clip::split_segment(&segment, &planes)? {
                        let (flux, _) = out.add(
                            op,
                            Record {
                                kind: "cell",
                                owner: cell,
                                axis: 2,
                                edge,
                                segment: s,
                            },
                        )?;
                        for (t, v) in out.physical[c].iter_mut().zip(flux) {
                            *t += v;
                        }
                    }
                }
            }
        }
        Ok(out)
    }
    fn add(&mut self, op: &Operators, record: Record) -> Result<(Vec<f64>, Vec<[f64; 3]>), String> {
        let id = self.records.len();
        let s = record.segment;
        let mid = s.middle();
        let offset: Point = std::array::from_fn(|d| (s.b[d] - s.a[d]) / (2.0 * 3.0_f64.sqrt()));
        let normal_weight = s.normal_measure().map(|x| x / 2.0);
        let mut flux = vec![0.0; op.active.len()];
        let mut gradient = vec![[0.0; 3]; op.active.len()];
        for sign in [-1.0, 1.0] {
            let point: Point = std::array::from_fn(|d| mid[d] + sign * offset[d]);
            if point.iter().chain(&normal_weight).any(|x| !x.is_finite())
                || point == s.a
                || point == s.b
            {
                return Err("positive trace integration point collapsed or nonfinite".into());
            }
            let mut values = Vec::new();
            for (i, node) in op.active.iter().enumerate() {
                let phi = op.layout.trace(node, point);
                if !phi.is_finite() {
                    return Err("nonfinite physical trace basis".into());
                }
                if phi != 0.0 {
                    values.push((i, phi));
                }
                flux[i] += phi * normal_weight[node.axis];
                if node.axis == 0 {
                    gradient[i][0] += phi * normal_weight[0];
                    gradient[i][1] += phi * normal_weight[1];
                } else {
                    gradient[i][1] += phi * normal_weight[0];
                    gradient[i][2] += phi * normal_weight[1];
                }
            }
            self.points.push(TracePoint {
                record: id,
                point,
                normal_weight,
                values,
            });
        }
        self.records.push(record);
        Ok((flux, gradient))
    }
}
