//! 3D Wendland C2, support R=2h; exact integration over tangential planes.
use std::f64::consts::PI;
#[derive(Clone, Copy)]
pub struct Kernel {
    pub radius: f64,
}
impl Kernel {
    pub fn volume(self, r: f64) -> f64 {
        let q = r / self.radius;
        if q >= 1.0 {
            return 0.0;
        }
        21.0 / (2.0 * PI * self.radius.powi(3)) * (1.0 - q).powi(4) * (1.0 + 4.0 * q)
    }
    /// -W'(r)/r in 3D, continuous at r=0.
    pub fn phi(self, r: f64) -> f64 {
        let q = r / self.radius;
        if q >= 1.0 {
            return 0.0;
        }
        210.0 / (PI * self.radius.powi(5)) * (1.0 - q).powi(3)
    }
    pub fn line(self, s: f64) -> f64 {
        let q = s.abs() / self.radius;
        if q >= 1.0 {
            return 0.0;
        }
        // Factorization avoids cancellation near the support endpoint.
        3.0 / (2.0 * self.radius) * (1.0 - q).powi(5) * (1.0 + 5.0 * q + 8.0 * q * q)
    }
    pub fn line_gradient(self, s: f64) -> f64 {
        -s * self.line_phi(s)
    }
    pub fn line_phi(self, s: f64) -> f64 {
        let q = s.abs() / self.radius;
        if q >= 1.0 {
            return 0.0;
        }
        21.0 / self.radius.powi(3) * (1.0 - q).powi(4) * (1.0 + 4.0 * q)
    }
    pub fn cdf(self, s: f64) -> f64 {
        let q = s.abs() / self.radius;
        if q >= 1.0 {
            return if s > 0.0 { 1.0 } else { 0.0 };
        }
        let v = 21.0
            * (q / 14.0 - q.powi(3) / 6.0 + q.powi(5) / 2.0 - 2.0 * q.powi(6) / 3.0
                + 5.0 * q.powi(7) / 14.0
                - q.powi(8) / 14.0);
        0.5 + s.signum() * v
    }
    pub fn gamma(self, y: f64, height: f64, top_wall: bool) -> (f64, f64) {
        let gamma = self.cdf(y) - if top_wall { self.cdf(y - height) } else { 0.0 };
        let gradient = self.line(y) - if top_wall { self.line(y - height) } else { 0.0 };
        (gamma, gradient)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent_integrals_and_gradient() {
        let k = Kernel { radius: 0.3 };
        let n = 100000;
        let dx = 2.0 * k.radius / n as f64;
        let mass: f64 = (0..n)
            .map(|i| k.line(-k.radius + (i as f64 + 0.5) * dx) * dx)
            .sum();
        assert!((mass - 1.0).abs() < 1e-12);
        let ds = 1e-6;
        for s in [0.0, 0.04, 0.15, 0.29] {
            let derivative = (k.line(s + ds) - k.line(s - ds)) / (2.0 * ds);
            assert!((derivative - k.line_gradient(s)).abs() < 2e-8);
            let derivative = (k.cdf(s + ds) - k.cdf(s - ds)) / (2.0 * ds);
            assert!((derivative - k.line(s)).abs() < 2e-8);
            // Independent radial annulus integration of the 3D source kernel.
            let dr = (k.radius - s) / n as f64;
            let integral: f64 = (0..n)
                .map(|i| {
                    let r = s + (i as f64 + 0.5) * dr;
                    2.0 * PI * r * k.volume(r) * dr
                })
                .sum();
            assert!((integral - k.line(s)).abs() < 1e-9);
        }
    }
}
