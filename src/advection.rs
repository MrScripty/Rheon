//! Semi-Lagrangian midpoint transport. Results are scratch until the caller's
//! complete timestep acceptance gate succeeds. No field buffers are allocated;
//! the optional parallel path uses the caller's Rayon pool.
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
        advect_bands(
            out,
            dimensions,
            None,
            crate::observation::CalculationStage::VelocityAdvection,
            &mut cancelled,
            |p, value| {
                *value = if p[d] == 0 || p[d] + 1 == dimensions[d] {
                    0.0
                } else {
                    let position = grid
                        .face_position(axis, p)
                        .expect("loop bounds match face geometry");
                    let departure = velocity.backtrace(position, dt)?;
                    component.sample(departure)? as f32
                };
                Ok(())
            },
        )?;
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
    let support = tracer_support(grid, old, velocity, dt);
    advect_bands(
        output,
        dimensions,
        support,
        crate::observation::CalculationStage::TracerAdvection,
        &mut cancelled,
        |p, value| {
            let position = grid
                .cell_position(p)
                .expect("loop bounds match cell geometry");
            *value = scalar.sample(sampler.backtrace(position, dt)?)? as f32;
            Ok(())
        },
    )
}

#[derive(Clone, Copy)]
struct TracerSupport {
    lower: [usize; 3],
    upper: [usize; 3],
}
impl TracerSupport {
    fn contains(self, p: [usize; 3]) -> bool {
        (0..3).all(|d| p[d] >= self.lower[d] && p[d] < self.upper[d])
    }
    fn empty(self) -> bool {
        (0..3).any(|d| self.lower[d] >= self.upper[d])
    }
}
// Convex velocity samples cannot exceed each component's donor magnitude.
// Expand exact nonzero support by maximum RK2 displacement, plus two cells for
// interpolation support and rounding. Display thresholds never decide physics.
// Dense fallback preserves departure-overflow rejection on extreme inputs.
fn tracer_support(
    grid: &GridGeometry,
    old: &[f32],
    velocity: [&[f32]; 3],
    dt: f64,
) -> Option<TracerSupport> {
    let counts = grid.counts();
    let mut margin = [0; 3];
    for d in 0..3 {
        let speed = velocity[d]
            .iter()
            .fold(0.0_f64, |a, b| a.max(f64::from(*b).abs()));
        let displacement = dt * speed;
        let world = grid.origin()[d].abs().max(grid.upper()[d].abs());
        if !(world + displacement).is_finite() {
            return None;
        }
        margin[d] = ((displacement / grid.spacing()[d]).ceil() as usize).saturating_add(2);
    }
    let [nx, ny, _] = counts;
    let mut lower = counts;
    let mut upper = [0; 3];
    for (row, value) in old.iter().enumerate() {
        if *value != 0.0 {
            let p = [row % nx, (row / nx) % ny, row / (nx * ny)];
            for d in 0..3 {
                lower[d] = lower[d].min(p[d]);
                upper[d] = upper[d].max(p[d] + 1);
            }
        }
    }
    if upper == [0; 3] {
        return Some(TracerSupport {
            lower: counts,
            upper,
        });
    }
    Some(TracerSupport {
        lower: std::array::from_fn(|d| lower[d].saturating_sub(margin[d])),
        upper: std::array::from_fn(|d| upper[d].saturating_add(margin[d]).min(counts[d])),
    })
}

// Join each bounded band before the next caller-thread cancellation checkpoint.
// Callbacks need neither Send nor Sync and are never invoked by pool workers.
fn advect_bands(
    output: &mut [f32],
    dimensions: [usize; 3],
    support: Option<TracerSupport>,
    stage: crate::observation::CalculationStage,
    cancelled: &mut impl FnMut() -> bool,
    operation: impl Fn([usize; 3], &mut f32) -> Result<(), AdvectionError> + Send + Sync,
) -> Result<(), AdvectionError> {
    let [nx, ny, _] = dimensions;
    let plane = nx * ny;
    let slices = 1;
    #[cfg(feature = "parallel")]
    let slices = if output.len() >= 16 * 1024 && rayon::current_num_threads() > 1 {
        (16 * 1024_usize).div_ceil(plane)
    } else {
        slices
    };
    let band_len = plane * slices;
    for (band, values) in output.chunks_mut(band_len).enumerate() {
        for _ in 0..values.len() / plane {
            if cancelled() {
                return Err(AdvectionError::Cancelled);
            }
        }
        if support.is_some_and(|s| {
            s.empty()
                || band * band_len / plane >= s.upper[2]
                || (band * band_len + values.len()).div_ceil(plane) <= s.lower[2]
        }) {
            values.fill(0.0);
            continue;
        }
        crate::parallel::try_for_each_chunk_mut(values, stage, 1024, |start, values| {
            let probe = crate::observation::WorkProbe::new(stage);
            for (i, value) in values.iter_mut().enumerate() {
                let row = band * band_len + start + i;
                let p = [row % nx, (row / nx) % ny, row / plane];
                if support.is_some_and(|s| !s.contains(p)) {
                    *value = 0.0;
                    continue;
                }
                probe.visit(p);
                operation(p, value)?;
            }
            Ok::<(), AdvectionError>(())
        })?;
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

#[cfg(all(test, feature = "parallel"))]
mod scheduling_tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn a_large_plane_exposes_parallel_work_instead_of_one_task_per_band() {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(4)
            .build()
            .unwrap();
        let active = AtomicUsize::new(0);
        let peak = AtomicUsize::new(0);
        let mut out = vec![0.0_f32; 128 * 128];
        pool.install(|| {
            advect_bands(
                &mut out,
                [128, 128, 1],
                None,
                crate::observation::CalculationStage::TracerAdvection,
                &mut || false,
                |p, value| {
                    if p[0] == 0 && p[1] % 16 == 0 {
                        let current = active.fetch_add(1, Ordering::SeqCst) + 1;
                        peak.fetch_max(current, Ordering::SeqCst);
                        std::thread::sleep(std::time::Duration::from_millis(10));
                        active.fetch_sub(1, Ordering::SeqCst);
                    }
                    *value = 1.0;
                    Ok(())
                },
            )
        })
        .unwrap();
        assert!(out.iter().all(|v| *v == 1.0));
        assert!(
            peak.load(Ordering::SeqCst) > 1,
            "large planes starve the worker pool"
        );
    }
}
