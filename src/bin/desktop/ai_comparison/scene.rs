//! Bounded diagnostic scene contract. Counts and scalar spacing keep cells cubic.
use serde::{Deserialize, Serialize};

pub const DT: f64 = 1.0 / 60.0;
pub const RHO: f64 = 1000.0;
pub const MU: f64 = 0.001;
pub const HISTORY_BYTES: usize = 96 * 1024 * 1024;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scene {
    pub seed: u64,
    pub counts: [usize; 3],
    pub cells_per_metre: u32,
    pub steps: usize,
    pub warmup_steps: usize,
    pub center: [f64; 3],
    pub acceleration: f64,
    pub radius: f64,
    pub swirl: f64,
    pub fluctuation: f64,
    pub frequency: f64,
    pub sgs: bool,
}
impl Default for Scene {
    fn default() -> Self {
        Self {
            seed: 20261016,
            counts: [16; 3],
            cells_per_metre: 16,
            steps: 64,
            warmup_steps: 15,
            center: [0.5; 3],
            acceleration: 1.0,
            radius: 0.25,
            swirl: 0.7,
            fluctuation: 0.4,
            frequency: 1.2,
            sgs: false,
        }
    }
}
impl Scene {
    pub fn validate(&self) -> Result<(), String> {
        if self.seed >= (1_u64 << 53) {
            return Err("Seed must fit an exact 53-bit integer".into());
        }
        if !self.counts.iter().all(|n| (5..=128).contains(n))
            || self.counts.iter().product::<usize>() > 65_536
        {
            return Err("Choose 5–128 cells per axis, at most 65,536 cells total".into());
        }
        if !(1..=128).contains(&self.cells_per_metre) || !(1..=512).contains(&self.steps) {
            return Err("Resolution/step budget exceeded".into());
        }
        if self.warmup_steps >= self.steps {
            return Err("Native warm-up must leave at least one AI step".into());
        }
        if (self.steps + 1) * self.counts.iter().product::<usize>() * 24 > HISTORY_BYTES {
            return Err(
                "Full-resolution paired history exceeds 96 MiB; reduce steps or counts".into(),
            );
        }
        for (value, lo, hi) in [
            (self.acceleration, 0.0, 4.0),
            (self.radius, 0.1, 0.5),
            (self.swirl, -1.0, 1.0),
            (self.fluctuation, 0.0, 0.8),
            (self.frequency, 0.0, 4.0),
        ] {
            if !value.is_finite() || !(lo..=hi).contains(&value) {
                return Err("Invalid force parameter".into());
            }
        }
        if self
            .center
            .iter()
            .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
        {
            return Err("Force center must lie within the domain".into());
        }
        Ok(())
    }
    pub fn h(&self) -> f64 {
        1.0 / f64::from(self.cells_per_metre)
    }
    pub fn extents(&self) -> [f64; 3] {
        self.counts.map(|n| n as f64 * self.h())
    }
    pub fn direction_phase(&self) -> ([f64; 3], f64) {
        let mut state = self.seed;
        let mut next = || {
            state = state.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = state;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            ((z ^ (z >> 31)) >> 11) as f64 / (1_u64 << 53) as f64
        };
        let direction = std::array::from_fn::<_, 3, _>(|_| 2.0 * next() - 1.0);
        let norm = direction
            .iter()
            .map(|v| v * v)
            .sum::<f64>()
            .sqrt()
            .max(1e-12);
        (direction.map(|v| v / norm), next() * std::f64::consts::TAU)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rectangular_domain_has_one_cell_size_and_bounded_history() {
        let mut s = Scene {
            counts: [24, 16, 12],
            ..Default::default()
        };
        s.validate().unwrap();
        assert_eq!(s.extents(), [1.5, 1.0, 0.75]);
        assert_eq!(s.h(), 0.0625);
        s.counts = [128; 3];
        assert!(s.validate().is_err());
        s.counts = [32; 3];
        s.steps = 512;
        assert!(s.validate().is_err());
    }
}
