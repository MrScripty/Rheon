//! Bounded trilinear interpolation at each field's own staggered locations.
//! This convex interpolation is not mass conservative. Clamping is a deliberate
//! fixed-box extension, not a general solid-boundary treatment.
use crate::{Axis, GridGeometry};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SamplingError {
    LengthMismatch,
    NonFiniteField,
    NonFinitePosition,
    InvalidTimeStep,
    ArithmeticFailure,
}
impl fmt::Display for SamplingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sampling failed: {self:?}")
    }
}
impl std::error::Error for SamplingError {}

/// Validates the immutable input once, so repeated samples do not rescan it.
/// Construction and sampling allocate no heap memory.
pub struct ScalarSampler<'a> {
    grid: &'a GridGeometry,
    values: &'a [f32],
    dimensions: [usize; 3],
    offset: [f64; 3],
}
impl<'a> ScalarSampler<'a> {
    pub fn cells(grid: &'a GridGeometry, values: &'a [f32]) -> Result<Self, SamplingError> {
        Self::new(grid, values, grid.counts(), [0.5; 3], grid.cell_len())
    }
    pub fn faces(
        grid: &'a GridGeometry,
        axis: Axis,
        values: &'a [f32],
    ) -> Result<Self, SamplingError> {
        let offset = match axis {
            Axis::X => [0.0, 0.5, 0.5],
            Axis::Y => [0.5, 0.0, 0.5],
            Axis::Z => [0.5, 0.5, 0.0],
        };
        Self::new(
            grid,
            values,
            grid.face_counts(axis),
            offset,
            grid.face_len(axis),
        )
    }
    fn new(
        grid: &'a GridGeometry,
        values: &'a [f32],
        dimensions: [usize; 3],
        offset: [f64; 3],
        len: usize,
    ) -> Result<Self, SamplingError> {
        if values.len() != len {
            return Err(SamplingError::LengthMismatch);
        }
        if values.iter().any(|v| !v.is_finite()) {
            return Err(SamplingError::NonFiniteField);
        }
        Ok(Self {
            grid,
            values,
            dimensions,
            offset,
        })
    }
    /// Clamp to sample support, including singleton dimensions. Interpolation
    /// and its donor min/max bound are computed in f64 from f32 samples.
    pub fn sample(&self, position: [f64; 3]) -> Result<f64, SamplingError> {
        if position.iter().any(|x| !x.is_finite()) {
            return Err(SamplingError::NonFinitePosition);
        }
        let origin = self.grid.origin();
        let upper = self.grid.upper();
        let spacing = self.grid.spacing();
        let mut lo = [0_usize; 3];
        let mut hi = lo;
        let mut t = [0.0; 3];
        for d in 0..3 {
            // Clamp in world coordinates first to avoid overflow for far-away
            // finite queries. Then clamp to this component's sample support.
            let x = position[d].clamp(origin[d], upper[d]);
            let q = ((x - origin[d]) / spacing[d] - self.offset[d])
                .clamp(0.0, (self.dimensions[d] - 1) as f64);
            if !q.is_finite() {
                return Err(SamplingError::ArithmeticFailure);
            }
            lo[d] = q.floor() as usize;
            hi[d] = (lo[d] + 1).min(self.dimensions[d] - 1);
            t[d] = q - lo[d] as f64;
        }
        let mut result = 0.0;
        let mut minimum = f64::INFINITY;
        let mut maximum = f64::NEG_INFINITY;
        for z in 0..2 {
            for y in 0..2 {
                for x in 0..2 {
                    let bits = [x, y, z];
                    let p: [usize; 3] =
                        std::array::from_fn(|d| if bits[d] == 0 { lo[d] } else { hi[d] });
                    let weight: f64 = (0..3)
                        .map(|d| if bits[d] == 0 { 1.0 - t[d] } else { t[d] })
                        .product();
                    let value = f64::from(
                        self.values[p[0] + self.dimensions[0] * (p[1] + self.dimensions[1] * p[2])],
                    );
                    minimum = minimum.min(value);
                    maximum = maximum.max(value);
                    result += weight * value;
                }
            }
        }
        if !result.is_finite() {
            return Err(SamplingError::ArithmeticFailure);
        }
        Ok(result.clamp(minimum, maximum))
    }
}

pub struct VelocitySampler<'a> {
    grid: &'a GridGeometry,
    components: [ScalarSampler<'a>; 3],
}
impl<'a> VelocitySampler<'a> {
    /// Does not impose wall values: this sampler also supports manufactured
    /// velocity fixtures. The simulation's boundary validator must impose them.
    pub fn new(grid: &'a GridGeometry, values: [&'a [f32]; 3]) -> Result<Self, SamplingError> {
        Ok(Self {
            grid,
            components: [
                ScalarSampler::faces(grid, Axis::X, values[0])?,
                ScalarSampler::faces(grid, Axis::Y, values[1])?,
                ScalarSampler::faces(grid, Axis::Z, values[2])?,
            ],
        })
    }
    pub fn sample(&self, position: [f64; 3]) -> Result<[f64; 3], SamplingError> {
        Ok([
            self.components[0].sample(position)?,
            self.components[1].sample(position)?,
            self.components[2].sample(position)?,
        ])
    }
    /// Midpoint RK2 backtrace, with both midpoint and departure clamped to the
    /// physical box. No temporal accuracy claim applies when clamping activates.
    pub fn backtrace(&self, position: [f64; 3], dt: f64) -> Result<[f64; 3], SamplingError> {
        if !dt.is_finite() || dt <= 0.0 {
            return Err(SamplingError::InvalidTimeStep);
        }
        let initial = self.sample(position)?;
        let midpoint = self.departure(position, initial, 0.5 * dt)?;
        let velocity = self.sample(midpoint)?;
        self.departure(position, velocity, dt)
    }
    fn departure(
        &self,
        position: [f64; 3],
        velocity: [f64; 3],
        dt: f64,
    ) -> Result<[f64; 3], SamplingError> {
        let mut result = [0.0; 3];
        for d in 0..3 {
            let value = position[d] - dt * velocity[d];
            if !value.is_finite() {
                return Err(SamplingError::ArithmeticFailure);
            }
            result[d] = value.clamp(self.grid.origin()[d], self.grid.upper()[d]);
        }
        Ok(result)
    }
}
