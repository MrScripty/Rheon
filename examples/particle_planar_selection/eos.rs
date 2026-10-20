//! Linear compressed EOS reference, SI. No production defaults are changed.
#[derive(Clone, Copy, Debug)]
pub struct Water {
    pub rho0: f64,
    pub mu: f64,
    pub gravity: f64,
    pub sound_speed: f64,
}
pub const WATER: Water = Water {
    rho0: 998.2,
    mu: 0.001002,
    gravity: 9.80665,
    sound_speed: 32.0,
};
#[derive(Debug, PartialEq)]
pub enum AdmissionError {
    InvalidInput,
    Compression,
    Mach,
    /// No selected, qualified discrete pressure/geometry stability bound exists.
    UnqualifiedAcousticOperator,
}
impl Water {
    pub fn validate(self) -> Result<Self, AdmissionError> {
        if [self.rho0, self.sound_speed]
            .iter()
            .any(|x| !x.is_finite() || *x <= 0.0)
            || !self.mu.is_finite()
            || self.mu < 0.0
            || !self.gravity.is_finite()
            || self.gravity < 0.0
            || !self.sound_speed.powi(2).is_normal()
        {
            return Err(AdmissionError::InvalidInput);
        }
        Ok(self)
    }
    pub fn equilibrium(self, depth: f64) -> Result<(f64, f64), AdmissionError> {
        self.validate()?;
        if !depth.is_finite() || depth < 0.0 {
            return Err(AdmissionError::InvalidInput);
        }
        let compression = (self.gravity * depth / self.sound_speed.powi(2)).exp_m1();
        let rho = self.rho0 * (1.0 + compression);
        let p = self.rho0 * self.sound_speed.powi(2) * compression;
        if !rho.is_finite() || !p.is_finite() {
            return Err(AdmissionError::InvalidInput);
        }
        Ok((rho, p))
    }
    /// Exact exponential profile integrated through a parcel; kg/m².
    pub fn column_mass(self, lower: f64, upper: f64, height: f64) -> f64 {
        if self.gravity == 0.0 {
            return self.rho0 * (upper - lower);
        }
        let a = self.gravity / self.sound_speed.powi(2);
        self.rho0 * (a * (height - upper)).exp() * (a * (upper - lower)).exp_m1() / a
    }
    pub fn pressure(self, rho: f64) -> f64 {
        self.sound_speed.powi(2) * (rho - self.rho0).max(0.0)
    }
    pub fn energy(self, rho: f64) -> f64 {
        let x = (rho / self.rho0 - 1.0).max(0.0);
        // Avoid cancellation of the two O(x) terms near the branch endpoint.
        let reduced = if x < 1e-3 {
            x * x * (0.5 + x * (-2.0 / 3.0 + x * (0.75 + x * (-0.8 + x * 5.0 / 6.0))))
        } else {
            x.ln_1p() - x / (1.0 + x)
        };
        self.sound_speed.powi(2) * reduced
    }
    pub fn envelope(self, height: f64, epsilon: f64, speed: f64) -> Result<(), AdmissionError> {
        self.validate()?;
        if !height.is_finite()
            || height < 0.0
            || !epsilon.is_finite()
            || epsilon <= 0.0
            || !speed.is_finite()
            || speed < 0.0
        {
            return Err(AdmissionError::InvalidInput);
        }
        if self.sound_speed.powi(2) < self.gravity * height / epsilon.ln_1p() {
            return Err(AdmissionError::Compression);
        }
        if speed / self.sound_speed > 0.1 {
            return Err(AdmissionError::Mach);
        }
        Ok(())
    }
    /// Deliberately refuses production admission until the selected operator
    /// has a verified discrete bound. A scale such as h/c is not that bound.
    pub fn production_dt(self, height: f64, speed: f64) -> Result<f64, AdmissionError> {
        self.envelope(height, 0.01, speed)?;
        Err(AdmissionError::UnqualifiedAcousticOperator)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn eos_equilibrium_and_thermodynamic_derivative() {
        let (rho, p) = WATER.equilibrium(1.0).unwrap();
        assert!((WATER.pressure(rho) - p).abs() < 2e-10);
        assert!(rho / WATER.rho0 - 1.0 < 0.01);
        let d = 1e-3;
        let derivative = (WATER.energy(rho + d) - WATER.energy(rho - d)) / (2.0 * d);
        assert!((derivative - p / rho.powi(2)).abs() < 1e-10);
        assert_eq!(WATER.energy(WATER.rho0 * 0.9), 0.0);
        let mass = WATER.column_mass(0.0, 1.0, 1.0);
        let independent = WATER.rho0 * WATER.sound_speed.powi(2) / WATER.gravity
            * (WATER.gravity / WATER.sound_speed.powi(2)).exp_m1();
        assert!((mass - independent).abs() < 1e-12);
    }
    #[test]
    fn admission_never_confuses_scale_with_stability() {
        let zero_g = Water {
            gravity: 0.0,
            ..WATER
        };
        zero_g.validate().unwrap();
        assert_eq!(zero_g.equilibrium(1.0).unwrap(), (WATER.rho0, 0.0));
        assert_eq!(zero_g.column_mass(0.0, 1.0, 1.0), WATER.rho0);
        assert_eq!(
            WATER.production_dt(1.0, 0.0),
            Err(AdmissionError::UnqualifiedAcousticOperator)
        );
        assert_eq!(WATER.envelope(1.0, 0.01, 3.21), Err(AdmissionError::Mach));
        let soft = Water {
            sound_speed: 20.0,
            ..WATER
        };
        assert_eq!(
            soft.envelope(1.0, 0.01, 0.0),
            Err(AdmissionError::Compression)
        );
        assert_eq!(
            WATER.envelope(f64::NAN, 0.01, 0.0),
            Err(AdmissionError::InvalidInput)
        );
        assert_eq!(
            WATER.equilibrium(f64::INFINITY),
            Err(AdmissionError::InvalidInput)
        );
    }
}
