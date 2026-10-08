//! Bounded isolated Coulomb events for an orientation-invariant sphere collider.
//! Atomic accepted prefixes; no persistent contact or force integration.
use crate::sphere_event_loop::{IntervalOutcomeStatus, admit_interval, interval_loop};
use crate::sphere_interval::{finite, kinetic};
use crate::*;
use std::cell::RefCell;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SphereFrictionIntervalRequest {
    pub interval: SphereIntervalRequest,
    pub coulomb_coefficient: f64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SphereFrictionIntervalStage {
    BetweenSegments,
    Contact(SphereFrictionStage),
}
#[derive(Debug, Clone, PartialEq)]
pub enum SphereFrictionIntervalAdmissionError {
    InvalidCoefficient,
    Interval(SphereIntervalAdmissionError),
}
#[derive(Debug, Clone, PartialEq)]
pub enum SphereFrictionIntervalStatus {
    Complete,
    ImpactBudgetExhausted,
    TimeProgressStalled,
    Cancelled,
    Stopped(SphereFrictionError),
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SphereFrictionIntervalSegment {
    pub contact: SphereFrictionReport,
    pub departure: Option<SphereDepartureReport>,
}
/// Unenclosed nearest-rounded diagnostics. Failure to represent an aggregate
/// makes this ledger unavailable; accepted records and owner remain available.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SphereFrictionIntervalAccounting {
    pub summed_impulse_n_s: [f64; 3],
    pub summed_spin_impulse_n_m_s: [f64; 3],
    pub summed_world_impulse_n_m_s: [f64; 3],
    pub momentum_defect_n_s: [f64; 3],
    pub spin_momentum_defect_n_m_s: [f64; 3],
    /// Endpoint world momentum minus all contact impulses, including coast error.
    pub world_angular_defect_n_m_s: [f64; 3],
    pub summed_predicted_energy_change_j: f64,
    pub summed_event_energy_change_j: f64,
    pub summed_point_midpoint_work_j: f64,
    pub kinetic_change_j: f64,
    pub energy_defect_j: f64,
    pub work_defect_j: f64,
    pub summed_segment_duration_s: f64,
    pub represented_elapsed_s: f64,
    pub clock_defect_s: f64,
    pub duration_defect_s: f64,
}
#[derive(Debug, Clone, PartialEq)]
pub struct SphereFrictionIntervalReport {
    pub request: SphereFrictionIntervalRequest,
    pub before: RigidPoseSnapshot,
    pub after: RigidPoseSnapshot,
    pub remaining_interval_s: f64,
    pub consumed_interval_s: f64,
    pub accepted_segments: usize,
    pub accepted_impacts: usize,
    pub status: SphereFrictionIntervalStatus,
    pub accounting: Option<SphereFrictionIntervalAccounting>,
}
impl SphericalRigidMotion {
    /// Uses the shared bounded event loop and the isolated Coulomb law. Each
    /// coast rotates with its pre-impact spin. The next event uses published V/w.
    /// Only the last privately certified facet may depart; later refusal keeps
    /// the accepted prefix and truthful unused time. No automatic substeps.
    pub fn advance_static_sphere_friction_interval(
        &mut self,
        request: SphereFrictionIntervalRequest,
        surface: &TriangleSurface,
        records: &mut [Option<SphereFrictionIntervalSegment>],
        cancelled: impl FnMut(SphereFrictionIntervalStage, usize, usize) -> bool,
        mut accepted: impl FnMut(usize, &SphericalRigidMotion, &SphereFrictionIntervalSegment),
    ) -> Result<SphereFrictionIntervalReport, SphereFrictionIntervalAdmissionError> {
        if !request.coulomb_coefficient.is_finite() || request.coulomb_coefficient < 0. {
            return Err(SphereFrictionIntervalAdmissionError::InvalidCoefficient);
        }
        admit_interval(self, request.interval, surface, records.len())
            .map_err(SphereFrictionIntervalAdmissionError::Interval)?;
        let callback = RefCell::new(cancelled);
        let interval = request.interval;
        let outcome = interval_loop(
            self,
            interval,
            surface,
            records.len(),
            |index| (callback.borrow_mut())(SphereFrictionIntervalStage::BetweenSegments, index, 0),
            |index, i| {
                (callback.borrow_mut())(
                    SphereFrictionIntervalStage::Contact(SphereFrictionStage::Contact(
                        SphereContactStage::Query,
                    )),
                    index,
                    i,
                )
            },
            |owner, current, hit, departure, remaining, index| {
                owner.coast_static_sphere_friction_selected(
                    SphereFrictionRequest {
                        expected_body: current.body.stamp,
                        expected_moving: current.body.surface,
                        expected_static: interval.expected_static,
                        radius_m: interval.radius_m,
                        restitution: interval.restitution,
                        coulomb_coefficient: request.coulomb_coefficient,
                        interval_s: remaining,
                        settings: interval.settings,
                    },
                    surface,
                    hit,
                    departure,
                    |stage, i| {
                        (callback.borrow_mut())(
                            SphereFrictionIntervalStage::Contact(stage),
                            index,
                            i,
                        )
                    },
                )
            },
            |index, owner, contact, departure| {
                let record = SphereFrictionIntervalSegment { contact, departure };
                records[index] = Some(record);
                accepted(index, owner, &record);
            },
        );
        let status = match outcome.status {
            IntervalOutcomeStatus::Complete => SphereFrictionIntervalStatus::Complete,
            IntervalOutcomeStatus::ImpactBudgetExhausted => {
                SphereFrictionIntervalStatus::ImpactBudgetExhausted
            }
            IntervalOutcomeStatus::TimeProgressStalled => {
                SphereFrictionIntervalStatus::TimeProgressStalled
            }
            IntervalOutcomeStatus::Cancelled => SphereFrictionIntervalStatus::Cancelled,
            IntervalOutcomeStatus::Stopped(e) => SphereFrictionIntervalStatus::Stopped(e),
        };
        Ok(SphereFrictionIntervalReport {
            request,
            before: outcome.before,
            after: outcome.after,
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
fn cross(a: [f64; 3], b: [f64; 3]) -> Option<[f64; 3]> {
    let mut out = [0.; 3];
    for i in 0..3 {
        let j = (i + 1) % 3;
        let k = (i + 2) % 3;
        out[i] = finite(finite(a[j] * b[k])? - finite(a[k] * b[j])?)?;
    }
    Some(out)
}
fn world_momentum(pose: RigidPoseSnapshot) -> Option<[f64; 3]> {
    let mut linear = [0.; 3];
    for (i, p) in linear.iter_mut().enumerate() {
        *p = finite(pose.body.mass_kg * pose.body.velocity_m_s[i])?;
    }
    let mut out = cross(pose.body.center_of_mass, linear)?;
    for (i, value) in out.iter_mut().enumerate() {
        *value = finite(
            *value + finite(pose.body.inertia_kg_m2[i] * pose.body.angular_velocity_rad_s[i])?,
        )?;
    }
    Some(out)
}
fn accounting(
    before: RigidPoseSnapshot,
    after: RigidPoseSnapshot,
    consumed: f64,
    records: &[Option<SphereFrictionIntervalSegment>],
) -> Option<SphereFrictionIntervalAccounting> {
    let mut impulse = [0.; 3];
    let mut spin = [0.; 3];
    let mut world = [0.; 3];
    let (mut predicted, mut events, mut work, mut duration) = (0., 0., 0., 0.);
    for entry in records {
        let c = entry.as_ref()?.contact;
        duration = finite(
            duration
                + c.hit
                    .map_or(c.request.interval_s, |h| h.requested_event_dt_s),
        )?;
        if let Some(i) = c.impact {
            let q_impulse = cross(c.hit?.point, i.impulse_n_s)?;
            for k in 0..3 {
                impulse[k] = finite(impulse[k] + i.impulse_n_s[k])?;
                spin[k] = finite(spin[k] + i.angular_impulse_n_m_s[k])?;
                world[k] = finite(world[k] + q_impulse[k])?;
            }
            predicted = finite(predicted + i.predicted_energy_change_j)?;
            events = finite(events + finite(i.kinetic_after_j - i.kinetic_before_j)?)?;
            work = finite(work + i.point_midpoint_work_j)?;
        }
    }
    let before_world = world_momentum(before)?;
    let after_world = world_momentum(after)?;
    let mut momentum = [0.; 3];
    let mut spin_defect = [0.; 3];
    let mut world_defect = [0.; 3];
    for i in 0..3 {
        momentum[i] = finite(
            finite(
                before.body.mass_kg
                    * finite(after.body.velocity_m_s[i] - before.body.velocity_m_s[i])?,
            )? - impulse[i],
        )?;
        spin_defect[i] = finite(
            finite(
                before.body.inertia_kg_m2[i]
                    * finite(
                        after.body.angular_velocity_rad_s[i]
                            - before.body.angular_velocity_rad_s[i],
                    )?,
            )? - spin[i],
        )?;
        world_defect[i] = finite(finite(after_world[i] - before_world[i])? - world[i])?;
    }
    let kinetic_change = finite(kinetic(after.body)? - kinetic(before.body)?)?;
    let elapsed = finite(after.time_s - before.time_s)?;
    Some(SphereFrictionIntervalAccounting {
        summed_impulse_n_s: impulse,
        summed_spin_impulse_n_m_s: spin,
        summed_world_impulse_n_m_s: world,
        momentum_defect_n_s: momentum,
        spin_momentum_defect_n_m_s: spin_defect,
        world_angular_defect_n_m_s: world_defect,
        summed_predicted_energy_change_j: predicted,
        summed_event_energy_change_j: events,
        summed_point_midpoint_work_j: work,
        kinetic_change_j: kinetic_change,
        energy_defect_j: finite(kinetic_change - predicted)?,
        work_defect_j: finite(kinetic_change - work)?,
        summed_segment_duration_s: duration,
        represented_elapsed_s: elapsed,
        clock_defect_s: finite(elapsed - duration)?,
        duration_defect_s: finite(consumed - duration)?,
    })
}
