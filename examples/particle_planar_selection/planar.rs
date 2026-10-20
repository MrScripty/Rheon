//! Necessary wall-normal selection screen, not an admitted production solver.
//! Parcels have mass per tangential area (kg/m²), volume per area (m).
//! Integrating the 3D equal tangential cell area A cancels A from AHA's
//! (V_i²+V_j²)/m_i and the tangential quadrature 1/A, yielding this 1D form.
use super::eos::{WATER, Water};
use super::kernel::Kernel;
#[derive(Clone, Copy, Debug)]
pub enum Method {
    Dummy,
    Gamma,
}
impl Method {
    pub fn name(self) -> &'static str {
        match self {
            Self::Dummy => "aha2012-linear-wendland-planar-screen-v1",
            Self::Gamma => "mayrhofer2013-k0-si-mu-planar-screen-v1",
        }
    }
}
#[derive(Clone)]
pub struct Column {
    pub y: Vec<f64>,
    pub rho: Vec<f64>,
    pub mass: Vec<f64>,
    pub velocity: Vec<f64>,
    pub height: f64,
    pub dx: f64,
    pub kernel: Kernel,
    pub water: Water,
    pub top_wall: bool,
}
pub struct Evaluation {
    pub pressure_acceleration: Vec<f64>,
    pub viscous_acceleration: Vec<f64>,
    pub density_rate: Vec<f64>,
    pub wall_pressure_force: f64,
    pub wall_viscous_force: f64,
    pub lower_viscous_force: f64,
    pub upper_viscous_force: f64,
    pub pressure_work_defect: f64,
    pub viscous_power: f64,
    pub max_neighbors: usize,
}
impl Column {
    pub fn hydrostatic(n: usize, ratio: f64, perturbed: bool) -> Self {
        let dx = 1.0 / n as f64;
        let mut y: Vec<_> = (0..n).map(|i| (i as f64 + 0.5) * dx).collect();
        if perturbed {
            for (i, p) in y.iter_mut().enumerate() {
                *p += 0.1 * dx * super::noise(i as u64, 11);
            }
        }
        let rho = y
            .iter()
            .map(|y| WATER.equilibrium(1.0 - y).unwrap().0)
            .collect();
        let mass = (0..n)
            .map(|i| WATER.column_mass(i as f64 * dx, (i + 1) as f64 * dx, 1.0))
            .collect();
        Self {
            y,
            rho,
            mass,
            velocity: vec![0.0; n],
            height: 1.0,
            dx,
            kernel: Kernel {
                radius: 2.0 * ratio * dx,
            },
            water: WATER,
            top_wall: false,
        }
    }
    pub fn channel(n: usize, ratio: f64) -> Self {
        let height = 0.001;
        let dx = height / n as f64;
        let water = Water {
            gravity: 0.0,
            ..WATER
        };
        Self {
            y: (0..n).map(|i| (i as f64 + 0.5) * dx).collect(),
            rho: vec![water.rho0; n],
            mass: vec![water.rho0 * dx; n],
            velocity: vec![0.0; n],
            height,
            dx,
            kernel: Kernel {
                radius: 2.0 * ratio * dx,
            },
            water,
            top_wall: true,
        }
    }
    pub fn energy(&self) -> f64 {
        self.y
            .iter()
            .enumerate()
            .map(|(i, y)| {
                self.mass[i]
                    * (0.5 * self.velocity[i].powi(2)
                        + self.water.gravity * y
                        + self.water.energy(self.rho[i]))
            })
            .sum()
    }
    pub fn evaluate(
        &self,
        method: Method,
        tangent: bool,
        upper_speed: f64,
        slip: bool,
    ) -> Evaluation {
        let n = self.y.len();
        let mut result = Evaluation {
            pressure_acceleration: vec![0.0; n],
            viscous_acceleration: vec![0.0; n],
            density_rate: vec![0.0; n],
            wall_pressure_force: 0.0,
            wall_viscous_force: 0.0,
            lower_viscous_force: 0.0,
            upper_viscous_force: 0.0,
            pressure_work_defect: 0.0,
            viscous_power: 0.0,
            max_neighbors: 0,
        };
        let pressure: Vec<_> = self.rho.iter().map(|r| self.water.pressure(*r)).collect();
        let volume: Vec<_> = self
            .rho
            .iter()
            .enumerate()
            .map(|(i, r)| self.mass[i] / r)
            .collect();
        let k = self.kernel;
        // Fixed wall quadrature and prescribed wall velocity in continuity.
        // Ghost velocity is exclusively the viscous extrapolation, never a
        // fictitious moving boundary in density evolution or wall-work ledgers.
        let mut ghosts = Vec::new();
        if matches!(method, Method::Dummy) {
            let layers = (k.radius / self.dx).ceil() as usize;
            for top in [false, true] {
                if top && !self.top_wall {
                    continue;
                }
                let uw = if top { upper_speed } else { 0.0 };
                for layer in 0..layers {
                    let yb = if top {
                        self.height + (layer as f64 + 0.5) * self.dx
                    } else {
                        -(layer as f64 + 0.5) * self.dx
                    };
                    let mut weight = 0.0;
                    let mut pb = 0.0;
                    let mut ub = 0.0;
                    for (j, fluid_pressure) in pressure.iter().enumerate() {
                        let w = k.line(yb - self.y[j]);
                        weight += w;
                        pb += w
                            * (fluid_pressure
                                - self.water.gravity * self.rho[j] * (yb - self.y[j]));
                        ub += w * self.velocity[j];
                    }
                    if weight > 0.0 {
                        pb /= weight;
                        let rhob = self.water.rho0 + pb / self.water.sound_speed.powi(2);
                        ghosts.push((yb, pb, rhob, 2.0 * uw - ub / weight, top));
                    }
                }
            }
        }
        let mut surfaces = Vec::new();
        if matches!(method, Method::Gamma) {
            for top in [false, true] {
                if top && !self.top_wall {
                    continue;
                }
                let yb = if top { self.height } else { 0.0 };
                let mut weight = 0.0;
                let mut pb = 0.0;
                for j in 0..n {
                    let w = volume[j] * k.line(yb - self.y[j]);
                    weight += w;
                    pb += w * (pressure[j] - self.water.gravity * self.rho[j] * (yb - self.y[j]));
                }
                if weight > 0.0 {
                    surfaces.push((yb, pb / weight, top));
                }
            }
        }
        for i in 0..n {
            let (gamma, gamma_prime) = k.gamma(self.y[i], self.height, self.top_wall);
            let mut neighbors = 0;
            for j in 0..n {
                if i == j {
                    continue;
                }
                let s = self.y[i] - self.y[j];
                if s.abs() >= k.radius {
                    continue;
                }
                neighbors += 1;
                let grad = k.line_gradient(s);
                let du = self.velocity[i] - self.velocity[j];
                match method {
                    Method::Dummy => {
                        let v2 = volume[i].powi(2) + volume[j].powi(2);
                        let pbar = (self.rho[j] * pressure[i] + self.rho[i] * pressure[j])
                            / (self.rho[i] + self.rho[j]);
                        result.pressure_acceleration[i] -= v2 * pbar * grad / self.mass[i];
                        result.viscous_acceleration[i] -=
                            self.water.mu * v2 / self.mass[i] * k.line_phi(s) * du;
                        if !tangent {
                            result.density_rate[i] += self.rho[i] * volume[j] * du * grad;
                        }
                    }
                    Method::Gamma => {
                        result.pressure_acceleration[i] -=
                            volume[j] * (pressure[i] + pressure[j]) * grad / (self.rho[i] * gamma);
                        result.viscous_acceleration[i] -= 2.0 * self.water.mu * volume[j]
                            / (self.rho[i] * gamma)
                            * k.line_phi(s)
                            * du;
                        if !tangent {
                            result.density_rate[i] += self.rho[i] / gamma * volume[j] * du * grad;
                        }
                    }
                }
            }
            result.max_neighbors = result.max_neighbors.max(neighbors);
            match method {
                Method::Dummy => {
                    for &(yb, pb, rhob, ghost_velocity, top) in &ghosts {
                        let s = self.y[i] - yb;
                        if s.abs() >= k.radius {
                            continue;
                        }
                        let v2 = volume[i].powi(2) + self.dx.powi(2);
                        let pbar = (rhob * pressure[i] + self.rho[i] * pb) / (self.rho[i] + rhob);
                        let ap = -v2 * pbar * k.line_gradient(s) / self.mass[i];
                        result.pressure_acceleration[i] += ap;
                        result.wall_pressure_force -= self.mass[i] * ap;
                        if !tangent {
                            result.density_rate[i] +=
                                self.rho[i] * self.dx * self.velocity[i] * k.line_gradient(s);
                        }
                        if !slip {
                            let av = -self.water.mu * v2 / self.mass[i]
                                * k.line_phi(s)
                                * (self.velocity[i] - ghost_velocity);
                            result.viscous_acceleration[i] += av;
                            let reaction = -self.mass[i] * av;
                            result.wall_viscous_force += reaction;
                            if top {
                                result.upper_viscous_force += reaction;
                            } else {
                                result.lower_viscous_force += reaction;
                            }
                        }
                    }
                }
                Method::Gamma => {
                    if !tangent {
                        result.density_rate[i] -=
                            self.rho[i] / gamma * self.velocity[i] * gamma_prime;
                    }
                    for &(yb, pb, top) in &surfaces {
                        let inward = if top { -1.0 } else { 1.0 };
                        let surface_gradient = inward * k.line(self.y[i] - yb);
                        let ap = (pressure[i] + pb) * surface_gradient / (self.rho[i] * gamma);
                        result.pressure_acceleration[i] += ap;
                        result.wall_pressure_force -= self.mass[i] * ap;
                        if !slip {
                            // Native near-wall linear wall law, eq20 adaptation;
                            // no exact analytic profile derivative is supplied.
                            let uw = if top { upper_speed } else { 0.0 };
                            let av = -2.0 * self.water.mu / (self.rho[i] * gamma)
                                * (self.velocity[i] - uw)
                                / (self.y[i] - yb).abs()
                                * k.line(self.y[i] - yb);
                            result.viscous_acceleration[i] += av;
                            let reaction = -self.mass[i] * av;
                            result.wall_viscous_force += reaction;
                            if top {
                                result.upper_viscous_force += reaction;
                            } else {
                                result.lower_viscous_force += reaction;
                            }
                        }
                    }
                }
            }
            if !tangent {
                result.pressure_work_defect += self.mass[i]
                    * (self.velocity[i] * result.pressure_acceleration[i]
                        + pressure[i] / self.rho[i].powi(2) * result.density_rate[i]);
            }
            result.viscous_power +=
                self.mass[i] * self.velocity[i] * result.viscous_acceleration[i];
        }
        result
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn closed_pressure_pairs_have_opposite_forces_but_work_is_measured() {
        for method in [Method::Dummy, Method::Gamma] {
            let mut c = Column::hydrostatic(32, 2.0, true);
            for (i, v) in c.velocity.iter_mut().enumerate() {
                *v = 0.01 * (std::f64::consts::PI * c.y[i]).sin();
            }
            let e = c.evaluate(method, false, 0.0, false);
            let fluid_force: f64 = c
                .mass
                .iter()
                .zip(&e.pressure_acceleration)
                .map(|(m, a)| m * a)
                .sum();
            // Gamma normalization can break pair cancellation; expose, not erase.
            assert!(e.pressure_work_defect.is_finite());
            if matches!(method, Method::Dummy) {
                assert!((fluid_force + e.wall_pressure_force).abs() < 1e-10);
            }
            assert!(e.pressure_work_defect.abs() > 1e-7);
        }
    }
    #[test]
    fn slip_and_no_slip_have_distinct_wall_tractions() {
        for method in [Method::Dummy, Method::Gamma] {
            let mut c = Column::channel(32, 2.0);
            c.velocity.fill(0.001);
            let slip = c.evaluate(method, true, 0.0, true);
            let no_slip = c.evaluate(method, true, 0.0, false);
            assert_eq!(slip.wall_viscous_force, 0.0);
            assert!(slip.viscous_acceleration.iter().all(|a| *a == 0.0));
            assert!(no_slip.wall_viscous_force > 0.0);
            assert!(no_slip.viscous_power < 0.0);
        }
    }
}
