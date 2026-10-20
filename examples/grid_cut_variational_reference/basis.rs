//! One declared continuous field for bulk and physical trace.
use super::geometry::{Point, Polygon};
use rheon::{Axis, GridGeometry};
#[derive(Clone, Debug)]
pub struct Node {
    pub global: usize,
    pub axis: usize,
    pub face: usize,
    pub index: [usize; 2],
    pub point: Point,
}
#[derive(Clone, Copy)]
pub struct Value {
    pub global: usize,
    pub axis: usize,
    pub phi: f64,
    pub dx: f64,
    pub dy: f64,
}
pub struct Layout {
    pub grid: GridGeometry,
    pub n: usize,
    pub h: f64,
    pub origin: Point,
    pub nodes: Vec<Node>,
}
impl Layout {
    pub fn new(n: usize, origin: Point, extent: f64, source: &Polygon) -> Result<Self, String> {
        if !(2..=16).contains(&n) || !extent.is_finite() || extent <= 0.0 {
            return Err(
                "bounded private reference grid n2..16 and positive extent required".into(),
            );
        }
        let h = extent / n as f64;
        let grid = GridGeometry::new(
            [n as u64, n as u64, 1],
            [h, h, 1.0],
            [origin[0], origin[1], 0.0],
        )
        .map_err(|e| e.to_string())?;
        if source.vertices().iter().any(|p| {
            (0..2).any(|d| p[d] < origin[d] + h / 2.0 || p[d] > origin[d] + extent - h / 2.0)
        }) {
            return Err("source outside complete component nodal hull; no zero halo".into());
        }
        for (a, b) in super::geometry::edges(source.vertices()) {
            for d in 0..2 {
                if a[d] == b[d] && (0..=n).any(|i| a[d] == origin[d] + i as f64 * h) {
                    return Err("finite source edge on primal plane unsupported".into());
                }
            }
        }
        let capacity = n
            .checked_add(1)
            .and_then(|v| n.checked_mul(v))
            .and_then(|v| v.checked_mul(2))
            .filter(|&v| v <= 544)
            .ok_or("private reference nodal allocation overflow/ceiling")?;
        let mut nodes = Vec::with_capacity(capacity);
        for (axis, native_axis) in [Axis::X, Axis::Y].into_iter().enumerate() {
            let counts = grid.face_counts(native_axis);
            for j in 0..counts[1] {
                for i in 0..counts[0] {
                    let coordinate = [i, j, 0];
                    let position = grid
                        .face_position(native_axis, coordinate)
                        .ok_or("native face position missing")?;
                    nodes.push(Node {
                        global: nodes.len(),
                        axis,
                        face: grid
                            .face_index(native_axis, coordinate)
                            .ok_or("native face index missing")?,
                        index: [i, j],
                        point: [position[0], position[1]],
                    });
                }
            }
        }
        Ok(Self {
            grid,
            n,
            h,
            origin,
            nodes,
        })
    }
    pub fn quarter_bounds(&self, i: usize, j: usize) -> (Point, Point) {
        (
            [
                self.origin[0] + i as f64 * self.h / 2.0,
                self.origin[1] + j as f64 * self.h / 2.0,
            ],
            [
                self.origin[0] + (i + 1) as f64 * self.h / 2.0,
                self.origin[1] + (j + 1) as f64 * self.h / 2.0,
            ],
        )
    }
    pub fn local(&self, i: usize, j: usize, p: Point) -> Result<Vec<Value>, String> {
        if i == 0 || j == 0 || i + 1 >= 2 * self.n || j + 1 >= 2 * self.n {
            return Err("positive patch requires complete both-component halo".into());
        }
        let mut out = Vec::with_capacity(8);
        for axis in 0..2 {
            let (ix, iy, shift, counts, offset) = if axis == 0 {
                (i / 2, (j - 1) / 2, [0.0, 0.5], [self.n + 1, self.n], 0)
            } else {
                (
                    (i - 1) / 2,
                    j / 2,
                    [0.5, 0.0],
                    [self.n, self.n + 1],
                    self.n * (self.n + 1),
                )
            };
            if ix + 1 >= counts[0] || iy + 1 >= counts[1] {
                return Err("missing nodal support".into());
            }
            let lo = [
                self.origin[0] + (ix as f64 + shift[0]) * self.h,
                self.origin[1] + (iy as f64 + shift[1]) * self.h,
            ];
            let t = [(p[0] - lo[0]) / self.h, (p[1] - lo[1]) / self.h];
            if t.iter().any(|x| !x.is_finite() || *x < 0.0 || *x > 1.0) {
                return Err("quadrature outside declared basis patch".into());
            }
            for y in 0..2 {
                for x in 0..2 {
                    let fx = if x == 0 { 1.0 - t[0] } else { t[0] };
                    let fy = if y == 0 { 1.0 - t[1] } else { t[1] };
                    out.push(Value {
                        global: offset + ix + x + counts[0] * (iy + y),
                        axis,
                        phi: fx * fy,
                        dx: if x == 0 { -fy / self.h } else { fy / self.h },
                        dy: if y == 0 { -fx / self.h } else { fx / self.h },
                    });
                }
            }
        }
        Ok(out)
    }
    pub fn trace(&self, node: &Node, p: Point) -> f64 {
        // Compact hat support is the basis definition, not a volume/aperture floor.
        (1.0 - ((p[0] - node.point[0]) / self.h).abs()).max(0.0)
            * (1.0 - ((p[1] - node.point[1]) / self.h).abs()).max(0.0)
    }
}
pub const GAUSS_WEIGHTS: [f64; 3] = [5.0 / 18.0, 4.0 / 9.0, 5.0 / 18.0];
pub fn gauss_nodes() -> [f64; 3] {
    let r = (3.0_f64 / 5.0).sqrt();
    [(1.0 - r) / 2.0, 0.5, (1.0 + r) / 2.0]
}
pub fn work_vectors(node: &Node) -> [f64; 2] {
    let [x, y] = node.point;
    if node.axis == 0 {
        [
            0.25 + x / 2.0 - y / 8.0 + x * y / 4.0,
            -0.25 + x * x / 8.0 + y / 4.0,
        ]
    } else {
        [
            -0.125 + x / 4.0 + y / 2.0 - x * y / 8.0,
            0.5 - x / 8.0 + y * y / 4.0,
        ]
    }
}
pub fn affine(node: &Node) -> f64 {
    let [x, y] = node.point;
    if node.axis == 0 {
        0.25 * x + 0.75 * y + 0.1
    } else {
        -0.2 * x - 0.25 * y - 0.3
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn rectangle() -> Polygon {
        Polygon::new(vec![[0.2, 0.2], [0.8, 0.2], [0.8, 0.8], [0.2, 0.8]]).unwrap()
    }
    #[test]
    fn halo_coincidence_and_orientation_refuse() {
        assert!(
            Layout::new(
                4,
                [0.0; 2],
                1.0,
                &Polygon::new(vec![[0.01, 0.2], [0.8, 0.2], [0.8, 0.8], [0.01, 0.8]]).unwrap()
            )
            .err()
            .unwrap()
            .contains("hull")
        );
        assert!(
            Layout::new(
                4,
                [0.0; 2],
                1.0,
                &Polygon::new(vec![[0.25, 0.3], [0.7, 0.3], [0.7, 0.7], [0.25, 0.7]]).unwrap()
            )
            .err()
            .unwrap()
            .contains("primal plane")
        );
        assert!(Polygon::new(vec![[0.2, 0.2], [0.2, 0.8], [0.8, 0.2]]).is_err());
    }
    #[test]
    fn polynomial_basis_reproduces_values_and_derivatives() {
        let l = Layout::new(4, [0.0; 2], 1.0, &rectangle()).unwrap();
        let p = [0.31, 0.42];
        let b = l.local(2, 3, p).unwrap();
        for axis in 0..2 {
            let selected: Vec<_> = b.iter().filter(|v| v.axis == axis).collect();
            assert!((selected.iter().map(|v| v.phi).sum::<f64>() - 1.0).abs() < 1e-14);
            for (d, expected) in p.iter().enumerate() {
                assert!(
                    (selected
                        .iter()
                        .map(|v| v.phi * l.nodes[v.global].point[d])
                        .sum::<f64>()
                        - expected)
                        .abs()
                        < 1e-14
                );
            }
            assert!(selected.iter().map(|v| v.dx).sum::<f64>().abs() < 1e-14);
            assert!(selected.iter().map(|v| v.dy).sum::<f64>().abs() < 1e-14);
        }
    }
}
