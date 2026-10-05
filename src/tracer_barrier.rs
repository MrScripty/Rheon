//! An opt-in passive-tracer barrier, separate from fixed-box velocity/pressure.
//! Static thin surfaces block trace legs and donor visibility. This is bounded
//! appearance transport, not a conservative or impermeable fluid-wall scheme.
use crate::{
    ForcedStepReport, GridGeometry, SamplingError, ScalarSampler, SurfaceError, SurfaceStamp,
    TriangleSurface, VelocitySampler,
};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TracerBarrierError {
    Sampling(SamplingError),
    Surface(SurfaceError),
    OutputLengthMismatch,
    PositionOutsideBox,
    NoVisibleDonor,
    Cancelled,
    ArithmeticFailure,
}
impl fmt::Display for TracerBarrierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tracer barrier rejected transport: {self:?}")
    }
}
impl std::error::Error for TracerBarrierError {}
impl From<SamplingError> for TracerBarrierError {
    fn from(value: SamplingError) -> Self {
        Self::Sampling(value)
    }
}
impl From<SurfaceError> for TracerBarrierError {
    fn from(value: SurfaceError) -> Self {
        match value {
            SurfaceError::Cancelled { .. } => Self::Cancelled,
            other => Self::Surface(other),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VisibleTracerSample {
    pub value: f64,
    /// Sum of positive trilinear weights whose direct donor segments are clear.
    pub visible_weight: f64,
    /// Positive-weight corners only; duplicate singleton corners have zero weight.
    pub blocked_donors: usize,
}

/// Immutable cell values and one static two-sided surface. No geometry copy or
/// heap allocation. Input validation occurs once. Visibility is numerical direct
/// line of sight; no solid sign, rotating/moving interval or topology is inferred.
pub struct TracerBarrierSampler<'a> {
    grid: &'a GridGeometry,
    values: &'a [f32],
    surface: &'a TriangleSurface,
    unfiltered: ScalarSampler<'a>,
}
impl<'a> TracerBarrierSampler<'a> {
    pub fn new(
        grid: &'a GridGeometry,
        values: &'a [f32],
        surface: &'a TriangleSurface,
    ) -> Result<Self, TracerBarrierError> {
        Ok(Self {
            grid,
            values,
            surface,
            unfiltered: ScalarSampler::cells(grid, values)?,
        })
    }
    /// Reject outside-box positions. Within the box, retain the original clamped
    /// cell-sample support. Check every positive-weight donor before selecting
    /// a result; ambiguity/cancellation rejects the whole sample. A coincident
    /// point/donor has no crossing segment and is retained. If all are blocked,
    /// return NoVisibleDonor rather than inventing a value or an offset.
    pub fn sample(
        &self,
        position: [f64; 3],
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<VisibleTracerSample, TracerBarrierError> {
        if !position.into_iter().all(f64::is_finite) {
            return Err(SamplingError::NonFinitePosition.into());
        }
        let origin = self.grid.origin();
        let upper = self.grid.upper();
        if (0..3).any(|d| position[d] < origin[d] || position[d] > upper[d]) {
            return Err(TracerBarrierError::PositionOutsideBox);
        }
        let spacing = self.grid.spacing();
        let dims = self.grid.counts();
        let mut lo = [0; 3];
        let mut hi = lo;
        let mut fractions = [0.0; 3];
        for d in 0..3 {
            let q = ((position[d] - origin[d]) / spacing[d] - 0.5).clamp(0.0, (dims[d] - 1) as f64);
            if !q.is_finite() {
                return Err(TracerBarrierError::ArithmeticFailure);
            }
            lo[d] = q.floor() as usize;
            hi[d] = (lo[d] + 1).min(dims[d] - 1);
            fractions[d] = q - lo[d] as f64;
        }
        let mut weights = [0.0; 8];
        let mut values = [0.0; 8];
        let mut visible_weight = 0.0;
        let mut blocked_donors = 0;
        let mut minimum = f64::INFINITY;
        let mut maximum = f64::NEG_INFINITY;
        for corner in 0..8 {
            let bits = [corner & 1, (corner >> 1) & 1, corner >> 2];
            let weight: f64 = (0..3)
                .map(|d| {
                    if bits[d] == 0 {
                        1.0 - fractions[d]
                    } else {
                        fractions[d]
                    }
                })
                .product();
            if weight == 0.0 {
                continue;
            }
            let p = std::array::from_fn(|d| if bits[d] == 0 { lo[d] } else { hi[d] });
            let donor = self.grid.cell_position(p).expect("bounded cell donor");
            if !clear_segment(self.surface, position, donor, &mut cancelled)? {
                blocked_donors += 1;
                continue;
            }
            let value = f64::from(self.values[self.grid.cell_unchecked(p)]);
            weights[corner] = weight;
            values[corner] = value;
            visible_weight += weight;
            minimum = minimum.min(value);
            maximum = maximum.max(value);
        }
        if visible_weight <= 0.0 || !visible_weight.is_finite() {
            return Err(TracerBarrierError::NoVisibleDonor);
        }
        let value = if blocked_donors == 0 {
            // Preserve the original arithmetic when no positive donor is blocked.
            self.unfiltered.sample(position)?
        } else {
            let sum: f64 = (0..8)
                .map(|i| (weights[i] / visible_weight) * values[i])
                .sum();
            if !sum.is_finite() {
                return Err(TracerBarrierError::ArithmeticFailure);
            }
            sum.clamp(minimum, maximum)
        };
        Ok(VisibleTracerSample {
            value: value.clamp(minimum, maximum),
            visible_weight,
            blocked_donors,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TracerBarrierReport {
    pub surface: SurfaceStamp,
    pub reverted_traces: usize,
    pub samples_with_blocked_donors: usize,
    pub blocked_donors: usize,
}
#[derive(Debug, Clone, Copy)]
pub struct BarrierStepReport {
    pub step: ForcedStepReport,
    /// None requests the preserved legacy tracer kernel.
    pub tracer_barrier: Option<TracerBarrierReport>,
}

/// Midpoint passive concentration transport in seconds, with frozen velocity in
/// m/s and static world geometry in metres. Check arrival-to-midpoint and
/// midpoint-to-departure trace legs. Any contact reverts to the old arrival value;
/// otherwise use normalized visible positive-weight donors at departure. This
/// does not alter velocity sampling or pressure, enforce wall flux, or conserve
/// scalar mass. Scratch output may be partially written on error/cancellation;
/// inputs remain immutable and only a successful caller may publish the output.
pub fn advect_tracer_with_barrier(
    grid: &GridGeometry,
    old: &[f32],
    velocity: [&[f32]; 3],
    dt: f64,
    surface: &TriangleSurface,
    output: &mut [f32],
    mut cancelled: impl FnMut() -> bool,
) -> Result<TracerBarrierReport, TracerBarrierError> {
    if !dt.is_finite() || dt <= 0.0 {
        return Err(SamplingError::InvalidTimeStep.into());
    }
    if output.len() != grid.cell_len() {
        return Err(TracerBarrierError::OutputLengthMismatch);
    }
    let scalar = TracerBarrierSampler::new(grid, old, surface)?;
    let sampler = VelocitySampler::new(grid, velocity)?;
    let mut report = TracerBarrierReport {
        surface: surface.stamp(),
        reverted_traces: 0,
        samples_with_blocked_donors: 0,
        blocked_donors: 0,
    };
    let [nx, ny, nz] = grid.counts();
    for z in 0..nz {
        if cancelled() {
            return Err(TracerBarrierError::Cancelled);
        }
        for y in 0..ny {
            for x in 0..nx {
                let p = [x, y, z];
                let index = grid.cell_unchecked(p);
                let arrival = grid.cell_position(p).expect("bounded arrival cell");
                let midpoint = departure(grid, arrival, sampler.sample(arrival)?, 0.5 * dt)?;
                if !clear_segment(surface, arrival, midpoint, &mut cancelled)? {
                    output[index] = old[index];
                    report.reverted_traces += 1;
                    continue;
                }
                let end = departure(grid, arrival, sampler.sample(midpoint)?, dt)?;
                if !clear_segment(surface, midpoint, end, &mut cancelled)? {
                    output[index] = old[index];
                    report.reverted_traces += 1;
                    continue;
                }
                let sample = scalar.sample(end, &mut cancelled)?;
                output[index] = sample.value as f32;
                if sample.blocked_donors > 0 {
                    report.samples_with_blocked_donors += 1;
                    report.blocked_donors = report
                        .blocked_donors
                        .checked_add(sample.blocked_donors)
                        .ok_or(TracerBarrierError::ArithmeticFailure)?;
                }
            }
        }
    }
    Ok(report)
}
fn clear_segment(
    surface: &TriangleSurface,
    a: [f64; 3],
    b: [f64; 3],
    cancelled: &mut impl FnMut() -> bool,
) -> Result<bool, TracerBarrierError> {
    if cancelled() {
        return Err(TracerBarrierError::Cancelled);
    }
    if a == b {
        return Ok(true);
    }
    Ok(surface.first_hit(a, b, |_| cancelled())?.is_none())
}
fn departure(
    grid: &GridGeometry,
    position: [f64; 3],
    velocity: [f64; 3],
    dt: f64,
) -> Result<[f64; 3], TracerBarrierError> {
    let mut result = [0.0; 3];
    for d in 0..3 {
        let value = position[d] - dt * velocity[d];
        if !value.is_finite() {
            return Err(SamplingError::ArithmeticFailure.into());
        }
        result[d] = value.clamp(grid.origin()[d], grid.upper()[d]);
    }
    Ok(result)
}
