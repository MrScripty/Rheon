//! Stateless, bounded inlet perturbations. Not a turbulence closure or spectrum.
use crate::SimulationError;
pub const EMITTER_CONTROLS_VERSION: u32 = 2;
/// Smooth seeded four-dimensional value noise attached to emitter coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmitterTurbulence {
    pub seed: u64,
    /// Upper bound on perturbation-vector magnitude, m/s; zero disables noise.
    pub strength: f64,
    /// Spatial lattice spacing in metres.
    pub spatial_scale: f64,
    /// Time between independent lattice planes, seconds. Smoothly interpolated.
    pub time_scale: f64,
}
impl Default for EmitterTurbulence {
    fn default() -> Self {
        Self {
            seed: 0,
            strength: 0.0,
            spatial_scale: 0.05,
            time_scale: 0.25,
        }
    }
}
impl EmitterTurbulence {
    pub(crate) fn validate(self) -> Result<(), SimulationError> {
        if !self.strength.is_finite()
            || self.strength < 0.0
            || self.strength > 1e6
            || !self.spatial_scale.is_normal()
            || self.spatial_scale <= 0.0
            || !self.time_scale.is_normal()
            || self.time_scale <= 0.0
        {
            return Err(SimulationError::InvalidEmitter);
        }
        Ok(())
    }
    /// Same coordinates/time/seed reproduce the same bounded field on the same
    /// numerical platform. Calls and retries consume no RNG state. Coordinates
    /// outside +/-2^40 lattice cells are refused rather than silently aliased.
    pub fn sample(self, local_position: [f64; 3], time: f64) -> Result<[f64; 3], SimulationError> {
        self.validate()?;
        if local_position.iter().any(|v| !v.is_finite()) || !time.is_finite() {
            return Err(SimulationError::InvalidEmitter);
        }
        if self.strength == 0.0 {
            return Ok([0.0; 3]);
        }
        let q = [
            local_position[0] / self.spatial_scale,
            local_position[1] / self.spatial_scale,
            local_position[2] / self.spatial_scale,
            time / self.time_scale,
        ];
        if q.iter()
            .any(|v| !v.is_finite() || v.abs() > (1_u64 << 40) as f64)
        {
            return Err(SimulationError::InvalidEmitter);
        }
        let base = q.map(|v| v.floor() as i64);
        let weight = q.map(|v| {
            let f = v - v.floor();
            f * f * (3.0 - 2.0 * f)
        });
        let mut value = [0.0; 3];
        for corner in 0..16 {
            let mut hash = self.seed;
            let mut w = 1.0;
            for d in 0..4 {
                let bit = (corner >> d) & 1;
                w *= if bit == 0 { 1.0 - weight[d] } else { weight[d] };
                hash = mix(hash
                    ^ (base[d] + bit as i64) as u64
                    ^ (d as u64).wrapping_mul(0x9e3779b97f4a7c15));
            }
            for (d, out) in value.iter_mut().enumerate() {
                let h = mix(hash ^ (d as u64 + 1).wrapping_mul(0xd1b54a32d192ed03));
                *out += w * (2.0 * ((h >> 11) as f64 / ((1_u64 << 53) as f64)) - 1.0);
            }
        }
        Ok(value.map(|v| v * self.strength / 3.0_f64.sqrt()))
    }
}
fn mix(mut x: u64) -> u64 {
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}
