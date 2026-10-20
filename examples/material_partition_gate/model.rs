use super::eos::WATER;
#[derive(Debug, PartialEq)]
pub enum Refusal {
    Geometry,
    WetEnvelope,
}
pub struct Model {
    pub reference: Vec<f64>,
    pub mass: Vec<f64>,
    pub first: Vec<f64>,
    pub nodal_mass: Vec<f64>,
    pub gravity: f64,
    pub hydro: bool,
}
pub struct Value {
    pub force: Vec<f64>,
    pub pressure_force: Vec<f64>,
    pub hessian: Vec<Vec<f64>>,
    pub pressure: Vec<f64>,
    pub jacobian: Vec<f64>,
    pub energy: f64,
    pub volume: f64,
    pub mass_moment: f64,
    pub min_density: f64,
    pub max_density: f64,
}
impl Model {
    pub fn new(perturbed: bool, hydro: bool) -> Self {
        let reference: Vec<_> = (0..=8)
            .map(|i| {
                if i == 0 || i == 8 {
                    i as f64 / 8.0
                } else {
                    (i as f64
                        + if perturbed {
                            0.1 * (i as f64 * 2.399963229728653).sin()
                        } else {
                            0.0
                        })
                        / 8.0
                }
            })
            .collect();
        let gravity = if hydro { WATER.gravity } else { 0.0 };
        let mut mass = Vec::new();
        let mut first = Vec::new();
        let mut nodal_mass = vec![0.0; 9];
        for e in 0..8 {
            let l = reference[e + 1] - reference[e];
            let left = if hydro {
                WATER.rho0
                    * (1.0 + 100.0 / (WATER.rho0 * WATER.sound_speed.powi(2)))
                    * (gravity * (1.0 - reference[e]) / WATER.sound_speed.powi(2)).exp()
            } else {
                1.005 * WATER.rho0
            };
            let m = source_moment(left, gravity / WATER.sound_speed.powi(2), l, 0);
            let q = source_moment(left, gravity / WATER.sound_speed.powi(2), l, 1);
            mass.push(m);
            first.push(q);
            nodal_mass[e] += m - q / l;
            nodal_mass[e + 1] += q / l;
        }
        assert!(nodal_mass.iter().all(|m| *m > 0.0));
        Self {
            reference,
            mass,
            first,
            nodal_mass,
            gravity,
            hydro,
        }
    }
    pub fn source_density(&self, x: f64) -> f64 {
        if self.hydro {
            WATER.rho0
                * (1.0 + 100.0 / (WATER.rho0 * WATER.sound_speed.powi(2)))
                * (self.gravity * (1.0 - x) / WATER.sound_speed.powi(2)).exp()
        } else {
            1.005 * WATER.rho0
        }
    }
    pub fn evaluate(&self, x: &[f64]) -> Result<Value, Refusal> {
        if x.len() != 9 || x.iter().any(|x| !x.is_finite()) || x.windows(2).any(|w| w[1] <= w[0]) {
            return Err(Refusal::Geometry);
        }
        let mut pressure = vec![0.0; 8];
        let mut jacobian = vec![0.0; 8];
        let mut pressure_force = vec![0.0; 9];
        let mut hessian = vec![vec![0.0; 9]; 9];
        let mut energy = 0.0;
        let mut volume = 0.0;
        let mut mass_moment = 0.0;
        let mut min_density = f64::INFINITY;
        let mut max_density: f64 = 0.0;
        for e in 0..8 {
            let l = self.reference[e + 1] - self.reference[e];
            let dl = x[e] - self.reference[e];
            let dr = x[e + 1] - self.reference[e + 1];
            let eta = (dr - dl) / l;
            let j = 1.0 + eta;
            if !j.is_finite() || j <= 0.0 {
                return Err(Refusal::Geometry);
            }
            for a in [self.reference[e], self.reference[e + 1]] {
                let density = self.source_density(a) / j;
                if !density.is_finite() || density <= WATER.rho0 || density > 1.01 * WATER.rho0 {
                    return Err(Refusal::WetEnvelope);
                }
                min_density = min_density.min(density);
                max_density = max_density.max(density);
            }
            jacobian[e] = j;
            volume += x[e + 1] - x[e];
            pressure[e] = WATER.sound_speed.powi(2) * (self.mass[e] / (l * j) - WATER.rho0);
            pressure_force[e] -= pressure[e];
            pressure_force[e + 1] += pressure[e];
            let stiffness = WATER.sound_speed.powi(2) * self.mass[e] / (l * l * j * j);
            hessian[e][e] += stiffness;
            hessian[e + 1][e + 1] += stiffness;
            hessian[e][e + 1] -= stiffness;
            hessian[e + 1][e] -= stiffness;
            energy += WATER.sound_speed.powi(2)
                * ((WATER.rho0 * l - self.mass[e]) * eta + self.mass[e] * log_remainder(eta));
            energy += self.gravity * (self.mass[e] * dl + (dr - dl) * self.first[e] / l);
            mass_moment += self.mass[e] * x[e] + (x[e + 1] - x[e]) * self.first[e] / l;
        }
        let force = pressure_force
            .iter()
            .zip(&self.nodal_mass)
            .map(|(f, m)| f - self.gravity * m)
            .collect();
        Ok(Value {
            force,
            pressure_force,
            hessian,
            pressure,
            jacobian,
            energy,
            volume,
            mass_moment,
            min_density,
            max_density,
        })
    }
    pub fn manufactured_force(&self, x: &[f64], p: &[f64]) -> Vec<f64> {
        assert_eq!(x.len(), 9);
        assert_eq!(p.len(), 8);
        let mut f = vec![0.0; 9];
        for e in 0..8 {
            f[e] -= p[e];
            f[e + 1] += p[e];
        }
        f
    }
    pub fn wrong_half_mass(&self) -> Vec<f64> {
        let mut m = vec![0.0; 9];
        for e in 0..8 {
            m[e] += 0.5 * self.mass[e];
            m[e + 1] += 0.5 * self.mass[e];
        }
        m
    }
}
fn source_moment(left: f64, a: f64, l: f64, k: i32) -> f64 {
    assert!((a * l).abs() <= 0.01);
    let mut term = 1.0;
    let mut sum = 0.0;
    for r in 0..=12 {
        if r > 0 {
            term *= -a * l / r as f64;
        }
        sum += term / (r + k + 1) as f64;
    }
    left * l.powi(k + 1) * sum
}
fn log_remainder(eta: f64) -> f64 {
    if eta.abs() >= 0.01 {
        return eta - eta.ln_1p();
    }
    let mut power = eta * eta;
    let mut sum = 0.0;
    for r in 2..=18 {
        sum += if r % 2 == 0 {
            power / r as f64
        } else {
            -power / r as f64
        };
        power *= eta;
    }
    sum
}
pub fn cholesky_min(h: &[Vec<f64>]) -> f64 {
    let mut l = vec![vec![0.0; 7]; 7];
    let mut min = f64::INFINITY;
    for i in 0..7 {
        for j in 0..=i {
            let s = h[i + 1][j + 1] - (0..j).map(|k| l[i][k] * l[j][k]).sum::<f64>();
            if i == j {
                assert!(s > 0.0);
                min = min.min(s);
                l[i][j] = s.sqrt();
            } else {
                l[i][j] = s / l[j][j];
            }
        }
    }
    min
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn material_map_refuses_inversion_and_outside_wet_envelope() {
        let m = Model::new(true, false);
        let mut x = m.reference.clone();
        x.swap(2, 3);
        assert!(matches!(m.evaluate(&x), Err(Refusal::Geometry)));
        let x: Vec<_> = m.reference.iter().map(|x| 2.0 * x).collect();
        assert!(matches!(m.evaluate(&x), Err(Refusal::WetEnvelope)));
    }
    #[test]
    fn source_moments_retain_constant_source_and_lumped_mass() {
        let m = Model::new(true, false);
        for e in 0..8 {
            let l = m.reference[e + 1] - m.reference[e];
            assert!((m.first[e] - 0.5 * m.mass[e] * l).abs() < 1e-13);
        }
        assert!((m.nodal_mass.iter().sum::<f64>() - m.mass.iter().sum::<f64>()).abs() < 1e-12);
    }
    #[test]
    fn incidence_stiffness_has_no_fixed_end_displacement_null() {
        let m = Model::new(true, true);
        let v = m.evaluate(&m.reference).unwrap();
        assert!(cholesky_min(&v.hessian) > 0.0);
        let a: Vec<_> = (0..8)
            .map(|e| 100.0 + if e % 2 == 0 { 10.0 } else { -10.0 })
            .collect();
        let f = m.manufactured_force(&m.reference, &a);
        assert!(f[1..8].iter().all(|f| f.abs() == 20.0));
    }
}
