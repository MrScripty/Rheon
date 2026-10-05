//! Prescribed constant translation of an immutable surface over one interval.
//! Relative-coordinate queries are geometric contacts, not fluid wall response.
use crate::{SurfaceError, SurfaceHit, TriangleSurface};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TranslationError {
    InvalidInterval,
    UnrepresentablePose { vertex: usize },
    InvalidTrajectory,
    NoRelativeMotion,
    ArithmeticFailure,
    Surface(SurfaceError),
}
impl fmt::Display for TranslationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "translation query rejected operation: {self:?}")
    }
}
impl std::error::Error for TranslationError {}
impl From<SurfaceError> for TranslationError {
    fn from(value: SurfaceError) -> Self {
        Self::Surface(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TranslatedHit {
    /// Hit in the unshifted surface's reference coordinates, with its original
    /// stamp, triangle, geometric normal and barycentric coordinates.
    pub reference_hit: SurfaceHit,
    pub world_position: [f64; 3],
    pub time: f64,
    pub wall_velocity: [f64; 3],
    /// Caller-owned interval identity, distinct from the surface's version.
    pub interval_id: u64,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TranslatedSegment {
    pub end: [f64; 3],
    pub hit: Option<TranslatedHit>,
}

/// Borrowed immutable geometry and a linearly interpolated translation. No
/// geometry copy, heap allocation, rotation, response or extrapolation. The
/// caller owns interval-id uniqueness and must use a new interval for new motion.
#[derive(Debug)]
pub struct TranslationInterval<'a> {
    surface: &'a TriangleSurface,
    interval_id: u64,
    times: [f64; 2],
    offsets: [[f64; 3]; 2],
    wall_velocity: [f64; 3],
}
impl<'a> TranslationInterval<'a> {
    /// Times are physical seconds; offsets are world translations of the same
    /// reference geometry, in metres. Both endpoint poses must be representable.
    pub fn new(
        surface: &'a TriangleSurface,
        interval_id: u64,
        times: [f64; 2],
        offsets: [[f64; 3]; 2],
    ) -> Result<Self, TranslationError> {
        let dt = times[1] - times[0];
        let displacement = sub(offsets[1], offsets[0]);
        let wall_velocity = displacement.map(|v| v / dt);
        if !times.into_iter().all(f64::is_finite)
            || !dt.is_finite()
            || dt <= 0.0
            || !offsets.into_iter().all(finite)
            || !finite(displacement)
            || !finite(wall_velocity)
            || (0..3).any(|d| displacement[d] != 0.0 && wall_velocity[d] == 0.0)
        {
            return Err(TranslationError::InvalidInterval);
        }
        for (vertex, &p) in surface.vertices().iter().enumerate() {
            for offset in offsets {
                if !finite(std::array::from_fn(|d| p[d] + offset[d])) {
                    return Err(TranslationError::UnrepresentablePose { vertex });
                }
            }
        }
        Ok(Self {
            surface,
            interval_id,
            times,
            offsets,
            wall_velocity,
        })
    }
    pub fn surface(&self) -> &'a TriangleSurface {
        self.surface
    }
    pub fn interval_id(&self) -> u64 {
        self.interval_id
    }
    pub fn times(&self) -> [f64; 2] {
        self.times
    }
    pub fn offsets(&self) -> [[f64; 3]; 2] {
        self.offsets
    }
    pub fn wall_velocity(&self) -> [f64; 3] {
        self.wall_velocity
    }

    /// The point moves linearly from start at times[0] to end at times[1]. A
    /// stationary world point is supported if the relative segment is nonzero.
    /// A zero relative segment is explicitly unsupported: this operation does
    /// not classify persistent contact or the distance of a stationary point.
    /// Per-facet cancellation and numerical ambiguity propagate without a
    /// partial result; immutable geometry and interval data remain reusable.
    pub fn first_hit(
        &self,
        start: [f64; 3],
        end: [f64; 3],
        cancelled: impl FnMut(usize) -> bool,
    ) -> Result<Option<TranslatedHit>, TranslationError> {
        let a = sub(start, self.offsets[0]);
        let b = sub(end, self.offsets[1]);
        let relative = sub(b, a);
        if !finite(start)
            || !finite(end)
            || !finite(sub(end, start))
            || !finite(a)
            || !finite(b)
            || !finite(relative)
        {
            return Err(TranslationError::InvalidTrajectory);
        }
        if relative == [0.0; 3] {
            return Err(TranslationError::NoRelativeMotion);
        }
        let Some(reference_hit) = self.surface.first_hit(a, b, cancelled)? else {
            return Ok(None);
        };
        let t = reference_hit.parameter;
        let world_position = std::array::from_fn(|d| (1.0 - t) * start[d] + t * end[d]);
        let time = (1.0 - t) * self.times[0] + t * self.times[1];
        if !finite(world_position) || !time.is_finite() {
            return Err(TranslationError::ArithmeticFailure);
        }
        Ok(Some(TranslatedHit {
            reference_hit,
            world_position,
            time,
            wall_velocity: self.wall_velocity,
            interval_id: self.interval_id,
        }))
    }
    /// Clip the linear point trajectory at the first space-time contact; a
    /// definite miss preserves end. No contact continuation is performed.
    pub fn clip_segment(
        &self,
        start: [f64; 3],
        end: [f64; 3],
        cancelled: impl FnMut(usize) -> bool,
    ) -> Result<TranslatedSegment, TranslationError> {
        let hit = self.first_hit(start, end, cancelled)?;
        Ok(TranslatedSegment {
            end: hit.map_or(end, |h| h.world_position),
            hit,
        })
    }
}
fn finite(p: [f64; 3]) -> bool {
    p.into_iter().all(f64::is_finite)
}
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|d| a[d] - b[d])
}
