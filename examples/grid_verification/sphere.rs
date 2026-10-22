//! Cell-average sphere/box geometry; paired adaptive quadrature estimates are not rigorous enclosures.
use std::f64::consts::PI;
#[derive(Clone, Copy)]
pub struct Sphere {
    pub center: [f64; 3],
    pub radius: f64,
}
#[derive(Clone, Copy, Default)]
pub struct Integral {
    pub value: f64,
    pub estimate: f64,
    pub exhausted: bool,
}
fn quadrant(y: f64, z: f64, r: f64) -> f64 {
    if r == 0.0 {
        return 0.0;
    }
    let y = y.min(r);
    let z = z.min(r);
    let s = (r * r - z * z).max(0.0).sqrt().min(y);
    let primitive = |t: f64| {
        0.5 * (t * (r * r - t * t).max(0.0).sqrt() + r * r * (t / r).clamp(-1.0, 1.0).asin())
    };
    z * s + primitive(y) - primitive(s)
}
fn disk_area(y: [f64; 2], z: [f64; 2], r: f64) -> f64 {
    let nearest = |a: [f64; 2]| {
        if a[0] > 0.0 {
            a[0]
        } else if a[1] < 0.0 {
            -a[1]
        } else {
            0.0
        }
    };
    if nearest(y).powi(2) + nearest(z).powi(2) >= r * r {
        return 0.0;
    }
    if y[0].abs().max(y[1].abs()).powi(2) + z[0].abs().max(z[1].abs()).powi(2) <= r * r {
        return (y[1] - y[0]) * (z[1] - z[0]);
    }
    let c = |a: f64, b: f64| a.signum() * b.signum() * quadrant(a.abs(), b.abs(), r);
    c(y[1], z[1]) - c(y[0], z[1]) - c(y[1], z[0]) + c(y[0], z[0])
}
fn simpson(f: &impl Fn(f64) -> f64, a: f64, b: f64, tol: f64, depth: u32) -> Integral {
    fn refine(
        f: &impl Fn(f64) -> f64,
        a: f64,
        b: f64,
        v: [f64; 3],
        whole: f64,
        tol: f64,
        depth: u32,
    ) -> Integral {
        let mid = 0.5 * (a + b);
        let l = f(0.5 * (a + mid));
        let r = f(0.5 * (mid + b));
        let left = (mid - a) / 6.0 * (v[0] + 4.0 * l + v[1]);
        let right = (b - mid) / 6.0 * (v[1] + 4.0 * r + v[2]);
        let error = (left + right - whole).abs() / 15.0;
        if error <= tol || depth == 0 {
            return Integral {
                value: left + right + (left + right - whole) / 15.0,
                estimate: if depth == 0 { error.max(tol) } else { error },
                exhausted: depth == 0 && error > tol,
            };
        }
        let p = refine(f, a, mid, [v[0], l, v[1]], left, tol / 2.0, depth - 1);
        let q = refine(f, mid, b, [v[1], r, v[2]], right, tol / 2.0, depth - 1);
        Integral {
            value: p.value + q.value,
            estimate: p.estimate + q.estimate,
            exhausted: p.exhausted || q.exhausted,
        }
    }
    let v = [f(a), f(0.5 * (a + b)), f(b)];
    let whole = (b - a) / 6.0 * (v[0] + 4.0 * v[1] + v[2]);
    refine(f, a, b, v, whole, tol, depth)
}
impl Sphere {
    pub fn volume(self) -> f64 {
        4.0 * PI * self.radius.powi(3) / 3.0
    }
    pub fn intersect(self, lo: [f64; 3], hi: [f64; 3], tol: f64) -> Integral {
        let low: [f64; 3] = std::array::from_fn(|d| lo[d] - self.center[d]);
        let high: [f64; 3] = std::array::from_fn(|d| hi[d] - self.center[d]);
        let near: f64 = (0..3)
            .map(|d| {
                if low[d] > 0.0 {
                    low[d].powi(2)
                } else if high[d] < 0.0 {
                    high[d].powi(2)
                } else {
                    0.0
                }
            })
            .sum();
        if near >= self.radius * self.radius {
            return Integral::default();
        }
        if (0..3)
            .map(|d| low[d].abs().max(high[d].abs()).powi(2))
            .sum::<f64>()
            <= self.radius * self.radius
        {
            return Integral {
                value: (hi[0] - lo[0]) * (hi[1] - lo[1]) * (hi[2] - lo[2]),
                estimate: 0.0,
                exhausted: false,
            };
        }
        let a = low[0].max(-self.radius);
        let b = high[0].min(self.radius);
        if a >= b {
            return Integral::default();
        }
        let mut knots = vec![a, b];
        if a < 0.0 && b > 0.0 {
            knots.push(0.0);
        }
        // Cross-section edge/corner contacts and extrema change the analytic branch.
        for y in [0.0, low[1], high[1]] {
            for z in [0.0, low[2], high[2]] {
                let q = self.radius * self.radius - y * y - z * z;
                if q > 0.0 {
                    for x in [-q.sqrt(), q.sqrt()] {
                        if x > a && x < b {
                            knots.push(x);
                        }
                    }
                }
            }
        }
        knots.sort_by(f64::total_cmp);
        knots.dedup();
        let f = |x: f64| {
            disk_area(
                [low[1], high[1]],
                [low[2], high[2]],
                (self.radius * self.radius - x * x).max(0.0).sqrt(),
            )
        };
        let mut total = Integral::default();
        let pieces = knots.len() - 1;
        for w in knots.windows(2) {
            let r = simpson(&f, w[0], w[1], tol / pieces as f64, 22);
            total.value += r.value;
            total.estimate += r.estimate;
            total.exhausted |= r.exhausted;
        }
        total
    }
    pub fn periodic_cell(self, lo: [f64; 3], hi: [f64; 3], tol: f64) -> Integral {
        let mut sum = Integral::default();
        for kz in -1..=1 {
            for ky in -1..=1 {
                for kx in -1..=1 {
                    let s = Self {
                        center: [
                            self.center[0] + kx as f64,
                            self.center[1] + ky as f64,
                            self.center[2] + kz as f64,
                        ],
                        radius: self.radius,
                    };
                    if (0..3)
                        .any(|d| s.center[d] + s.radius <= lo[d] || s.center[d] - s.radius >= hi[d])
                    {
                        continue;
                    }
                    let q = s.intersect(lo, hi, tol);
                    sum.value += q.value;
                    sum.estimate += q.estimate;
                    sum.exhausted |= q.exhausted;
                }
            }
        }
        sum
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sphere_octants_and_geometric_full_empty() {
        let s = Sphere {
            center: [0.0; 3],
            radius: 0.15,
        };
        let q = s.intersect([0.0; 3], [0.15; 3], 1e-13);
        assert!((q.value - s.volume() / 8.0).abs() < 1e-11);
        assert_eq!(s.intersect([0.2; 3], [0.3; 3], 1e-13).value, 0.0);
        assert!((s.intersect([-0.01; 3], [0.01; 3], 1e-13).value - 0.02_f64.powi(3)).abs() < 1e-20);
    }
    #[test]
    fn disk_quadrant_primitive_is_normalized() {
        assert!((quadrant(1.0, 1.0, 1.0) - PI / 4.0).abs() < 1e-15);
    }
}
