//! Bounded continuation of isolated frictionless sphere/static-facet impacts.
//! Each segment publishes atomically; a later stop preserves the accepted prefix.
use crate::sphere_event_loop::{IntervalOutcomeStatus, admit_interval, interval_loop};
use crate::*;
use std::cell::RefCell;

pub const MAX_SPHERE_INTERVAL_IMPACTS: usize = 64;
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SphereIntervalRequest {
    pub expected_body: RigidStamp,
    pub expected_moving: SurfaceStamp,
    pub expected_static: SurfaceStamp,
    pub radius_m: f64,
    pub restitution: f64,
    pub interval_s: f64,
    pub settings: SphereContactSettings,
    pub max_impacts: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SphereIntervalStage {
    BetweenSegments,
    Contact(SphereContactStage),
}
#[derive(Debug, Clone, PartialEq)]
pub enum SphereIntervalAdmissionError {
    InvalidRequest,
    RecordCapacity,
    Contact(SphereContactError),
}
#[derive(Debug, Clone, PartialEq)]
pub enum SphereIntervalStatus {
    Complete,
    ImpactBudgetExhausted,
    TimeProgressStalled,
    Cancelled,
    Stopped(SphereContactError),
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SphereIntervalSegment {
    pub contact: SphereContactReport,
    /// Certificate on the actual start state used to exclude one previous facet.
    pub departure: Option<SphereDepartureReport>,
}
/// Nearest-rounded, unenclosed diagnostics; unavailable on arithmetic overflow.
/// Raw accepted segment reports always remain available in the caller's buffer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SphereIntervalAccounting {
    pub summed_impulse_n_s: [f64; 3],
    pub momentum_defect: [f64; 3],
    pub summed_predicted_energy_change_j: f64,
    pub summed_event_energy_change_j: f64,
    pub kinetic_change_j: f64,
    pub energy_defect_j: f64,
    pub summed_segment_duration_s: f64,
    pub represented_elapsed_s: f64,
    pub clock_defect_s: f64,
    pub duration_defect_s: f64,
}
#[derive(Debug, Clone, PartialEq)]
pub struct SphereIntervalReport {
    pub before: RigidPoseSnapshot,
    pub after: RigidPoseSnapshot,
    pub requested_interval_s: f64,
    pub remaining_interval_s: f64,
    /// Requested minus remaining; this nominal consumption is separately audited
    /// against actual clock advancement and the sum of accepted segment durations.
    pub consumed_interval_s: f64,
    pub accepted_segments: usize,
    pub accepted_impacts: usize,
    pub status: SphereIntervalStatus,
    pub accounting: Option<SphereIntervalAccounting>,
}
impl SphericalRigidMotion {
    /// No external forces. No simultaneous/resting manifold solver. Initial
    /// contact is an explicit stop; only a privately certified last impact can
    /// depart at zero normal velocity. Records[0..accepted_segments] are replaced;
    /// other slots are untouched. Admission errors change neither owner nor slots.
    /// Observer receives the actual stored owner after each accepted publication.
    /// Observer panics, like callback panics, are outside the returned-result contract.
    pub fn advance_static_sphere_interval(
        &mut self,
        request: SphereIntervalRequest,
        surface: &TriangleSurface,
        records: &mut [Option<SphereIntervalSegment>],
        cancelled: impl FnMut(SphereIntervalStage, usize, usize) -> bool,
        mut accepted: impl FnMut(usize, &SphericalRigidMotion, &SphereIntervalSegment),
    ) -> Result<SphereIntervalReport, SphereIntervalAdmissionError> {
        admit_interval(self, request, surface, records.len())?;
        let callback = RefCell::new(cancelled);
        let outcome = interval_loop(
            self,
            request,
            surface,
            records.len(),
            |index| (callback.borrow_mut())(SphereIntervalStage::BetweenSegments, index, 0),
            |index, i| {
                (callback.borrow_mut())(
                    SphereIntervalStage::Contact(SphereContactStage::Query),
                    index,
                    i,
                )
            },
            |owner, current, hit, departure, remaining, index| {
                owner.coast_static_sphere_selected(
                    current.body.stamp,
                    current.body.surface,
                    surface,
                    request.radius_m,
                    request.restitution,
                    remaining,
                    request.settings,
                    hit,
                    departure,
                    |stage, i| {
                        (callback.borrow_mut())(SphereIntervalStage::Contact(stage), index, i)
                    },
                )
            },
            |index, owner, contact, departure| {
                let record = SphereIntervalSegment { contact, departure };
                records[index] = Some(record);
                accepted(index, owner, &record);
            },
        );
        let status = match outcome.status {
            IntervalOutcomeStatus::Complete => SphereIntervalStatus::Complete,
            IntervalOutcomeStatus::ImpactBudgetExhausted => {
                SphereIntervalStatus::ImpactBudgetExhausted
            }
            IntervalOutcomeStatus::TimeProgressStalled => SphereIntervalStatus::TimeProgressStalled,
            IntervalOutcomeStatus::Cancelled => SphereIntervalStatus::Cancelled,
            IntervalOutcomeStatus::Stopped(e) => SphereIntervalStatus::Stopped(e),
        };
        Ok(SphereIntervalReport {
            before: outcome.before,
            after: outcome.after,
            requested_interval_s: request.interval_s,
            remaining_interval_s: outcome.remaining,
            consumed_interval_s: outcome.consumed,
            accepted_segments: outcome.segments,
            accepted_impacts: outcome.impacts,
            status,
            accounting: accounting(
                outcome.before,
                outcome.after,
                outcome.consumed,
                &records[..outcome.segments],
            ),
        })
    }
}
pub(crate) fn finite(value: f64) -> Option<f64> {
    value.is_finite().then_some(value)
}
pub(crate) fn kinetic(state: RigidSnapshot) -> Option<f64> {
    let mut result = 0.;
    for i in 0..3 {
        result = finite(
            result
                + finite(
                    0.5 * state.mass_kg * finite(state.velocity_m_s[i] * state.velocity_m_s[i])?,
                )?,
        )?;
        result = finite(
            result
                + finite(
                    0.5 * state.inertia_kg_m2[i]
                        * finite(
                            state.angular_velocity_rad_s[i] * state.angular_velocity_rad_s[i],
                        )?,
                )?,
        )?;
    }
    Some(result)
}
fn accounting(
    before: RigidPoseSnapshot,
    after: RigidPoseSnapshot,
    consumed: f64,
    records: &[Option<SphereIntervalSegment>],
) -> Option<SphereIntervalAccounting> {
    let mut impulse = [0.; 3];
    let mut predicted = 0.;
    let mut events = 0.;
    let mut duration = 0.;
    for entry in records {
        let c = entry.as_ref()?.contact;
        duration = finite(
            duration
                + c.hit
                    .map_or(c.requested_interval_s, |h| h.requested_event_dt_s),
        )?;
        if let Some(i) = c.impact {
            for (sum, value) in impulse.iter_mut().zip(i.impulse_n_s) {
                *sum = finite(*sum + value)?;
            }
            predicted = finite(predicted + i.predicted_energy_change_j)?;
            events = finite(events + finite(i.kinetic_after_j - i.kinetic_before_j)?)?;
        }
    }
    let mut momentum = [0.; 3];
    for i in 0..3 {
        momentum[i] = finite(
            finite(
                before.body.mass_kg
                    * finite(after.body.velocity_m_s[i] - before.body.velocity_m_s[i])?,
            )? - impulse[i],
        )?;
    }
    let kinetic_change = finite(kinetic(after.body)? - kinetic(before.body)?)?;
    let elapsed = finite(after.time_s - before.time_s)?;
    Some(SphereIntervalAccounting {
        summed_impulse_n_s: impulse,
        momentum_defect: momentum,
        summed_predicted_energy_change_j: predicted,
        summed_event_energy_change_j: events,
        kinetic_change_j: kinetic_change,
        energy_defect_j: finite(kinetic_change - predicted)?,
        summed_segment_duration_s: duration,
        represented_elapsed_s: elapsed,
        clock_defect_s: finite(elapsed - duration)?,
        duration_defect_s: finite(consumed - duration)?,
    })
}
