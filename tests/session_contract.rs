#[path = "../examples/session_support/mod.rs"]
mod support;
use rheon::*;
use std::time::Instant;

#[test]
fn lifecycle_identity_export_and_quota() {
    let (mut session, boundary) = support::fixture([4, 2, 2], 0);
    assert_eq!(session.snapshot().stage(), SnapshotStage::Constructor);
    assert_eq!(session.snapshot().identity().run_id, 1);
    assert_eq!(session.capabilities().api_version, 1);
    assert!(!session.capabilities().hard_realtime);
    for frame in 1..=8 {
        let report = session
            .step(support::inputs(), boundary, None, |_| false)
            .unwrap();
        let snapshot = session.snapshot();
        assert_eq!(snapshot.stage(), SnapshotStage::Accepted);
        assert_eq!(snapshot.frame(), frame);
        assert_eq!(snapshot.state().carrier.time, report.carrier.step.time);
        assert_eq!(snapshot.state().carrier_stamp.version, frame as u64);
        let mut expected = Vec::new();
        write_dense3d_frame(
            &mut expected,
            snapshot.grid(),
            snapshot.state(),
            frame,
            Some(&report),
        )
        .unwrap();
        let mut actual = Vec::new();
        snapshot.write_dense3d(&mut actual).unwrap();
        assert_eq!(actual, expected);
    }
    assert!(matches!(
        session.step(support::inputs(), boundary, None, |_| false),
        Err(SessionError::StepLimit)
    ));
    session.stop();
    assert!(matches!(
        session.step(support::inputs(), boundary, None, |_| false),
        Err(SessionError::Stopped)
    ));
    assert_eq!(session.snapshot().frame(), 8);
}

#[test]
fn cancellation_deadline_and_invalid_inputs_preserve_publication_and_retry() {
    let (mut session, boundary) = support::fixture([4, 2, 2], 0);
    let mut before = Vec::new();
    session.snapshot().write_dense3d(&mut before).unwrap();
    assert!(matches!(
        session.step(support::inputs(), boundary, Some(Instant::now()), |_| false),
        Err(SessionError::Deadline)
    ));
    assert!(matches!(
        session.step(support::inputs(), boundary, None, |s| s
            == LiquidStepStage::BeforeCommit),
        Err(SessionError::Step(LiquidStepError::Cancelled))
    ));
    let mut invalid = support::inputs();
    invalid.requested_dt = f64::NAN;
    assert!(session.step(invalid, boundary, None, |_| false).is_err());
    let mut after = Vec::new();
    session.snapshot().write_dense3d(&mut after).unwrap();
    assert_eq!(before, after);
    session
        .step(support::inputs(), boundary, None, |_| false)
        .unwrap();
    assert_eq!(session.snapshot().frame(), 1);
}

#[test]
fn output_failure_does_not_poison_session() {
    let (mut session, boundary) = support::fixture([4, 2, 2], 0);
    let mut empty = &mut [][..];
    assert!(session.snapshot().write_dense3d(&mut empty).is_err());
    session
        .step(support::inputs(), boundary, None, |_| false)
        .unwrap();
    assert_eq!(session.snapshot().frame(), 1);
}

#[test]
fn every_observed_checkpoint_preserves_last_accepted_state() {
    let (mut probe, boundary) = support::fixture([4, 2, 2], 0);
    probe
        .step(support::inputs(), boundary, None, |_| false)
        .unwrap();
    let mut stages = Vec::new();
    probe
        .step(support::inputs(), boundary, None, |stage| {
            if !stages.contains(&stage) {
                stages.push(stage);
            }
            false
        })
        .unwrap();
    assert!(stages.contains(&LiquidStepStage::BeforeCommit));
    for stage in stages {
        let (mut session, boundary) = support::fixture([4, 2, 2], 0);
        session
            .step(support::inputs(), boundary, None, |_| false)
            .unwrap();
        let mut before = Vec::new();
        session.snapshot().write_dense3d(&mut before).unwrap();
        assert!(
            session
                .step(support::inputs(), boundary, None, |s| s == stage)
                .is_err()
        );
        let mut after = Vec::new();
        session.snapshot().write_dense3d(&mut after).unwrap();
        assert_eq!(before, after, "checkpoint {stage:?}");
        assert_eq!(session.snapshot().stage(), SnapshotStage::Accepted);
        session
            .step(support::inputs(), boundary, None, |_| false)
            .unwrap();
        assert_eq!(session.snapshot().frame(), 2);
    }
}

#[test]
fn host_clock_uses_shortened_accepted_interval() {
    let (mut session, boundary) = support::fixture([4, 2, 2], 0);
    session
        .step(support::inputs(), boundary, None, |_| false)
        .unwrap();
    let start = session.snapshot().state().carrier.time;
    let mut inputs = support::inputs();
    inputs.requested_dt = 10.0;
    let report = session.step(inputs, boundary, None, |_| false).unwrap();
    assert!(report.carrier.step.dt < inputs.requested_dt);
    assert_eq!(
        session.snapshot().state().carrier.time,
        start + report.carrier.step.dt
    );
}
