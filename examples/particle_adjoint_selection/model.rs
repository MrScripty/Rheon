use super::{eos::WATER, kernel::Kernel};
#[derive(Clone, Copy)]
pub enum Map {
    Gamma,
    Voronoi,
}
impl Map {
    pub fn name(self) -> &'static str {
        match self {
            Self::Gamma => "gamma-offset-adjoint-planar-v1",
            Self::Voronoi => "clipped-voronoi-1d-reference-v1",
        }
    }
}
pub struct State {
    pub map: Map,
    pub y: Vec<f64>,
    pub mass: Vec<f64>,
    pub offset: Vec<f64>,
    kernel: Kernel,
}
pub struct Value {
    pub rho: Vec<f64>,
    pub jac: Vec<Vec<f64>>,
    pub wall_jac: Vec<[f64; 2]>,
    pub min_gamma: f64,
    pub min_numerator: f64,
}
fn faces(y: &[f64]) -> Vec<f64> {
    std::iter::once(0.0)
        .chain(y.windows(2).map(|a| 0.5 * (a[0] + a[1])))
        .chain(std::iter::once(1.0))
        .collect()
}
impl State {
    pub fn new(map: Map, n: usize, perturbed: bool, hydro: bool) -> Self {
        let dx = 1.0 / n as f64;
        let y: Vec<_> = (0..n)
            .map(|i| {
                (i as f64
                    + 0.5
                    + if perturbed {
                        0.1 * ((i + 1) as f64 * 2.399963229728653).sin()
                    } else {
                        0.0
                    })
                    * dx
            })
            .collect();
        let f = faces(&y);
        let a = 1.0 + 1000.0 / (WATER.rho0 * WATER.sound_speed.powi(2));
        let mass: Vec<_> = (0..n)
            .map(|i| {
                let (lo, hi) = match map {
                    Map::Gamma => (i as f64 * dx, (i + 1) as f64 * dx),
                    Map::Voronoi => (f[i], f[i + 1]),
                };
                if hydro {
                    a * WATER.column_mass(lo, hi, 1.0)
                } else {
                    1.005 * WATER.rho0 * (hi - lo)
                }
            })
            .collect();
        let kernel = Kernel { radius: 3.0 * dx };
        let offset = (0..n)
            .map(|i| {
                if matches!(map, Map::Voronoi) {
                    return 0.0;
                }
                let rho = if hydro {
                    WATER.rho0
                        * a
                        * (WATER.gravity * (1.0 - y[i]) / WATER.sound_speed.powi(2)).exp()
                } else {
                    1.005 * WATER.rho0
                };
                rho * kernel.gamma(y[i], 1.0, true).0
                    - (0..n)
                        .map(|j| mass[j] * kernel.line(y[i] - y[j]))
                        .sum::<f64>()
            })
            .collect();
        Self {
            map,
            y,
            mass,
            offset,
            kernel,
        }
    }
    pub fn evaluate(&self, y: &[f64]) -> Value {
        let n = y.len();
        assert!(y[0] > 0.0 && y[n - 1] < 1.0 && y.windows(2).all(|a| a[1] > a[0]));
        let mut rho = vec![0.0; n];
        let mut jac = vec![vec![0.0; n]; n];
        let mut wall_jac = vec![[0.0; 2]; n];
        let mut min_gamma = if matches!(self.map, Map::Gamma) {
            f64::INFINITY
        } else {
            f64::NAN
        };
        let mut min_numerator = min_gamma;
        match self.map {
            Map::Gamma => {
                for i in 0..n {
                    let (gamma, gp) = self.kernel.gamma(y[i], 1.0, true);
                    let numerator = self.offset[i]
                        + (0..n)
                            .map(|j| self.mass[j] * self.kernel.line(y[i] - y[j]))
                            .sum::<f64>();
                    assert!(gamma > 0.0 && numerator > 0.0 && numerator.is_finite());
                    min_gamma = min_gamma.min(gamma);
                    min_numerator = min_numerator.min(numerator);
                    rho[i] = numerator / gamma;
                    for k in 0..n {
                        let g = self.kernel.line_gradient(y[i] - y[k]);
                        jac[i][i] += self.mass[k] * g / gamma;
                        jac[i][k] -= self.mass[k] * g / gamma;
                    }
                    jac[i][i] -= rho[i] * gp / gamma;
                    wall_jac[i] = [
                        rho[i] * self.kernel.line(y[i]) / gamma,
                        -rho[i] * self.kernel.line(y[i] - 1.0) / gamma,
                    ];
                }
            }
            Map::Voronoi => {
                let f = faces(y);
                for i in 0..n {
                    let volume = f[i + 1] - f[i];
                    assert!(volume > 0.0);
                    rho[i] = self.mass[i] / volume;
                    if i + 1 < n {
                        jac[i][i] -= 0.5 * rho[i] / volume;
                        jac[i][i + 1] -= 0.5 * rho[i] / volume;
                    }
                    if i > 0 {
                        jac[i][i - 1] += 0.5 * rho[i] / volume;
                        jac[i][i] += 0.5 * rho[i] / volume;
                    }
                    if i == 0 {
                        wall_jac[i][0] = rho[i] / volume;
                    }
                    if i + 1 == n {
                        wall_jac[i][1] = -rho[i] / volume;
                    }
                }
            }
        }
        assert!(rho.iter().all(|r| r.is_finite() && *r > WATER.rho0));
        Value {
            rho,
            jac,
            wall_jac,
            min_gamma,
            min_numerator,
        }
    }
    pub fn force(&self, y: &[f64], value: &Value, p: &[f64]) -> (Vec<f64>, [f64; 2]) {
        let n = y.len();
        let mut force = vec![0.0; n];
        let mut walls = [0.0; 2];
        match self.map {
            Map::Gamma => {
                let gamma: Vec<_> = y
                    .iter()
                    .map(|y| self.kernel.gamma(*y, 1.0, true).0)
                    .collect();
                for i in 0..n {
                    for j in i + 1..n {
                        let pair = -self.mass[i]
                            * self.mass[j]
                            * (p[i] / value.rho[i].powi(2) / gamma[i]
                                + p[j] / value.rho[j].powi(2) / gamma[j])
                            * self.kernel.line_gradient(y[i] - y[j]);
                        force[i] += pair;
                        force[j] -= pair;
                    }
                    let factor = self.mass[i] * p[i] / (value.rho[i] * gamma[i]);
                    let bottom = factor * self.kernel.line(y[i]);
                    let top = -factor * self.kernel.line(y[i] - 1.0);
                    force[i] += bottom + top;
                    walls[0] -= bottom;
                    walls[1] -= top;
                }
            }
            Map::Voronoi => {
                for j in 1..n {
                    let shared = 0.5 * (p[j - 1] - p[j]);
                    force[j - 1] += shared;
                    force[j] += shared;
                }
                walls = [-p[0], p[n - 1]];
            }
        }
        (force, walls)
    }
}
impl Value {
    pub fn density_rate(&self, v: &[f64]) -> Vec<f64> {
        self.jac.iter().map(|row| super::dot(row, v)).collect()
    }
    pub fn volume_gradient(&self, mass: &[f64]) -> Vec<f64> {
        (0..mass.len())
            .map(|k| {
                (0..mass.len())
                    .map(|i| -mass[i] / self.rho[i].powi(2) * self.jac[i][k])
                    .sum()
            })
            .collect()
    }
    pub fn total_volume(&self, mass: &[f64]) -> f64 {
        mass.iter().zip(&self.rho).map(|(m, r)| m / r).sum()
    }
    pub fn energy(&self, mass: &[f64]) -> f64 {
        mass.iter()
            .zip(&self.rho)
            .map(|(m, r)| m * WATER.energy(*r))
            .sum()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn jacobian_and_independent_pair_force_are_adjoint() {
        for map in [Map::Gamma, Map::Voronoi] {
            let s = State::new(map, 32, true, true);
            let v = s.evaluate(&s.y);
            let p: Vec<_> = (0..32).map(|i| 1000.0 + (i as f64).sin() * 100.0).collect();
            let (f, w) = s.force(&s.y, &v, &p);
            for (k, fk) in f.iter().enumerate() {
                let adjoint: f64 = (0..32)
                    .map(|i| -s.mass[i] * p[i] / v.rho[i].powi(2) * v.jac[i][k])
                    .sum();
                assert!((fk - adjoint).abs() < 1e-10);
            }
            for (b, wb) in w.iter().enumerate() {
                let expected: f64 = (0..32)
                    .map(|i| -s.mass[i] * p[i] / v.rho[i].powi(2) * v.wall_jac[i][b])
                    .sum();
                assert!((wb - expected).abs() < 1e-10);
            }
        }
    }
    #[test]
    fn reference_has_exact_partition_and_retains_null_mode() {
        let s = State::new(Map::Voronoi, 32, true, false);
        let v = s.evaluate(&s.y);
        assert!((v.total_volume(&s.mass) - 1.0).abs() < 1e-14);
        assert!(v.volume_gradient(&s.mass).iter().all(|g| g.abs() < 1e-14));
        let (f, _) = s.force(&s.y, &v, &[1234.0; 32]);
        assert!(f.iter().all(|f| *f == 0.0));
        let p: Vec<_> = (0..32)
            .map(|i| if i % 2 == 0 { 1100.0 } else { 900.0 })
            .collect();
        let (f, _) = s.force(&s.y, &v, &p);
        assert!(f[1..31].iter().all(|f| *f == 0.0));
        assert_eq!(f[0], 100.0);
        assert_eq!(f[31], 100.0);
    }
    #[test]
    fn offsets_are_not_reset_by_displacement() {
        let s = State::new(Map::Gamma, 32, true, false);
        let mut shifted = s.y.clone();
        shifted[8] += 1e-5;
        let original = s.evaluate(&s.y);
        let moved = s.evaluate(&shifted);
        assert!((moved.rho[8] - original.rho[8]).abs() > 1e-7);
    }
}
