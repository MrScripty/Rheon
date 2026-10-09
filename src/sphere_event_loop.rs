//! Shared bounded control loop. Impact laws own their atomic coast/finalizer;
//! this loop owns query/departure/progress/budget and accepted-prefix semantics.
use crate::sphere_departure::QualifiedDeparture;
use crate::*;

pub(crate) enum IntervalOutcomeStatus<E> {
    Complete,
    ImpactBudgetExhausted,
    TimeProgressStalled,
    Cancelled,
    Stopped(E),
}
pub(crate) struct IntervalOutcome<E> {
    pub before: RigidPoseSnapshot,
    pub after: RigidPoseSnapshot,
    pub remaining: f64,
    pub consumed: f64,
    pub segments: usize,
    pub impacts: usize,
    pub status: IntervalOutcomeStatus<E>,
}
pub(crate) fn admit_interval(
    owner: &SphericalRigidMotion,
    request: SphereIntervalRequest,
    surface: &TriangleSurface,
    capacity: usize,
) -> Result<(), SphereIntervalAdmissionError> {
    if !request.interval_s.is_finite()
        || request.interval_s <= 0.
        || !request.restitution.is_finite()
        || !(0. ..=1.).contains(&request.restitution)
        || request.max_impacts > MAX_SPHERE_INTERVAL_IMPACTS
    {
        return Err(SphereIntervalAdmissionError::InvalidRequest);
    }
    if capacity < request.max_impacts + 1 {
        return Err(SphereIntervalAdmissionError::RecordCapacity);
    }
    StaticSphereSweep::new(
        owner,
        request.expected_body,
        request.expected_moving,
        surface,
        request.expected_static,
        request.radius_m,
        request.settings,
    )
    .map_err(SphereIntervalAdmissionError::Contact)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn interval_loop<C, E: From<SphereContactError>>(
    owner: &mut SphericalRigidMotion,
    request: SphereIntervalRequest,
    surface: &TriangleSurface,
    record_capacity: usize,
    mut between_cancelled: impl FnMut(usize) -> bool,
    mut query_cancelled: impl FnMut(usize, usize) -> bool,
    mut coast: impl FnMut(
        &mut SphericalRigidMotion,
        RigidPoseSnapshot,
        Option<SphereContactHit>,
        Option<&QualifiedDeparture>,
        f64,
        usize,
    ) -> Result<C, E>,
    mut accepted: impl FnMut(usize, &SphericalRigidMotion, C, Option<SphereDepartureReport>),
) -> IntervalOutcome<E> {
    let before = owner.snapshot();
    let mut remaining = request.interval_s;
    let mut segments = 0;
    let mut impacts = 0;
    let mut previous_hit = None;
    let status = loop {
        if remaining == 0. {
            break IntervalOutcomeStatus::Complete;
        }
        if between_cancelled(segments) {
            break IntervalOutcomeStatus::Cancelled;
        }
        let departure = if let Some(hit) = previous_hit {
            match QualifiedDeparture::new(owner, surface, request.radius_m, hit) {
                Ok(d) => Some(d),
                Err(e) => break IntervalOutcomeStatus::Stopped(E::from(e)),
            }
        } else {
            None
        };
        let current = owner.snapshot();
        let query = StaticSphereSweep::new(
            owner,
            current.body.stamp,
            current.body.surface,
            surface,
            request.expected_static,
            request.radius_m,
            request.settings,
        );
        let hit = match query.and_then(|q| {
            q.first_contact_with_departure(remaining, departure.as_ref(), |i| {
                query_cancelled(segments, i)
            })
        }) {
            Ok(hit) => hit,
            Err(e) => break IntervalOutcomeStatus::Stopped(E::from(e)),
        };
        if hit.is_some() && impacts == request.max_impacts {
            break IntervalOutcomeStatus::ImpactBudgetExhausted;
        }
        let duration = hit.map_or(remaining, |h| h.requested_event_dt_s);
        let next_remaining = remaining - duration;
        if !duration.is_finite()
            || duration <= 0.
            || !next_remaining.is_finite()
            || next_remaining < 0.
            || next_remaining >= remaining
            || segments >= record_capacity
        {
            break IntervalOutcomeStatus::TimeProgressStalled;
        }
        // Capacity and control arithmetic are settled before the law publishes.
        let contact = match coast(owner, current, hit, departure.as_ref(), remaining, segments) {
            Ok(r) => r,
            Err(e) => break IntervalOutcomeStatus::Stopped(e),
        };
        remaining = next_remaining;
        impacts += usize::from(hit.is_some());
        previous_hit = hit;
        accepted(segments, owner, contact, departure.map(|d| d.report));
        segments += 1;
    };
    IntervalOutcome {
        before,
        after: owner.snapshot(),
        remaining,
        consumed: request.interval_s - remaining,
        segments,
        impacts,
        status,
    }
}
