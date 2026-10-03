//! Semi-Lagrangian midpoint transport. Results are scratch until the caller's
//! complete timestep acceptance gate succeeds. No allocation occurs here.
use crate::{Axis, GridGeometry, SamplingError, ScalarSampler, VelocitySampler};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum AdvectionError {
    Sampling(SamplingError),
    OutputLengthMismatch,
    Cancelled,
}
impl fmt::Display for AdvectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "advection failed: {self:?}")
    }
}
impl std::error::Error for AdvectionError {}
impl From<SamplingError> for AdvectionError {
    fn from(value: SamplingError) -> Self {
        Self::Sampling(value)
    }
}

/// Each component samples its own staggered field at the common RK2 departure.
/// Normal boundary faces are assigned exactly zero for the fixed-box model.
/// Cancellation or error may leave output partially written; input is immutable.
pub fn advect_velocity(
    grid: &GridGeometry,
    old: [&[f32]; 3],
    dt: f64,
    output: [&mut [f32]; 3],
    mut cancelled: impl FnMut() -> bool,
) -> Result<(), AdvectionError> {
    validate_dt(dt)?;
    let velocity = VelocitySampler::new(grid, old)?;
    let axes = [Axis::X, Axis::Y, Axis::Z];
    for d in 0..3 {
        if output[d].len() != grid.face_len(axes[d]) {
            return Err(AdvectionError::OutputLengthMismatch);
        }
    }
    for (d, out) in output.into_iter().enumerate() {
        let axis = axes[d];
        let dimensions = grid.face_counts(axis);
        let component = ScalarSampler::faces(grid, axis, old[d])?;
        for z in 0..dimensions[2] {
            if cancelled() {
                return Err(AdvectionError::Cancelled);
            }
            for y in 0..dimensions[1] {
                for x in 0..dimensions[0] {
                    let p = [x, y, z];
                    let index = grid.face_unchecked(axis, p);
                    out[index] = if p[d] == 0 || p[d] + 1 == dimensions[d] {
                        0.0
                    } else {
                        let position = grid
                            .face_position(axis, p)
                            .expect("loop bounds match face geometry");
                        let departure = velocity.backtrace(position, dt)?;
                        component.sample(departure)? as f32
                    };
                }
            }
        }
    }
    Ok(())
}

/// Passive concentration transport, not a finite-volume conservative update.
/// Bounded interpolation preserves donor range; it does not preserve total mass.
pub fn advect_tracer(
    grid: &GridGeometry,
    old: &[f32],
    velocity: [&[f32]; 3],
    dt: f64,
    output: &mut [f32],
    mut cancelled: impl FnMut() -> bool,
) -> Result<(), AdvectionError> {
    validate_dt(dt)?;
    if output.len() != grid.cell_len() {
        return Err(AdvectionError::OutputLengthMismatch);
    }
    let scalar = ScalarSampler::cells(grid, old)?;
    let sampler = VelocitySampler::new(grid, velocity)?;
    let dimensions = grid.counts();
    for z in 0..dimensions[2] {
        if cancelled() {
            return Err(AdvectionError::Cancelled);
        }
        for y in 0..dimensions[1] {
            for x in 0..dimensions[0] {
                let p = [x, y, z];
                let index = grid.cell_unchecked(p);
                let position = grid
                    .cell_position(p)
                    .expect("loop bounds match cell geometry");
                output[index] = scalar.sample(sampler.backtrace(position, dt)?)? as f32;
            }
        }
    }
    Ok(())
}
fn validate_dt(dt: f64) -> Result<(), AdvectionError> {
    if !dt.is_finite() || dt <= 0.0 {
        Err(SamplingError::InvalidTimeStep.into())
    } else {
        Ok(())
    }
}
