use super::*;
use crate::Axis;
use crate::projection_step::ProjectionFault;
use crate::solver_provider::composition::projection::{ProjectionCheckpoint, ProjectionSelection};
use crate::{
    BodyForce, ForceUnits, GridGeometry, LiquidInlet, LiquidVolumeSettings, PressureSettings,
    SimulationConfig, VolumeStamp,
};
const CAP: usize = 16 * 1024 * 1024;
const FORCE: [BodyForce; 1] = [BodyForce {
    value: [0.001, 0.0, 0.0],
    units: ForceUnits::Acceleration,
    region: None,
}];
fn config() -> LiquidTransportConfig {
    LiquidTransportConfig {
        carrier: SimulationConfig {
            density: 998.2,
            memory_limit: CAP,
            pressure: PressureSettings {
                relative_residual: 1e-10,
                absolute_residual: 1e-12,
                divergence_limit: 1e-8,
                max_iterations: 2000,
            },
            actual_divergence_limit: 1e-5,
            max_courant: 1.0,
        },
        represented_density: 998.2,
        volume_stamp: VolumeStamp { id: 1, version: 0 },
        carrier_id: 2,
    }
}
fn inputs() -> LiquidStepInputs<'static> {
    LiquidStepInputs {
        requested_dt: 0.001,
        smoke_source: None,
        forces: &FORCE,
        inlet: LiquidInlet::new(VolumeStamp { id: 3, version: 0 }, [[0.0; 2]; 3]).unwrap(),
        source: None,
        volume: LiquidVolumeSettings {
            max_outward_courant: 1.0,
            actual_divergence_limit: 1e-5,
        },
    }
}
fn owner(
    method: PressureImplementation,
    quota: usize,
) -> (SimulationSession, ProjectionStepWorkspace) {
    let grid = GridGeometry::new([4; 3], [0.25; 3], [0.0; 3]).unwrap();
    let session = SimulationSession::new_with_projection(
        SessionIdentity {
            run_id: 101,
            source_revision: "c1b2-private-fixed-native".into(),
        },
        grid.clone(),
        config(),
        method,
        vec![0.5; grid.cell_len()],
        CAP,
        quota,
        ProjectionIntent::BuiltinNativeV1,
    )
    .unwrap();
    let workspace = ProjectionStepWorkspace::new(&grid, 998.2, CAP, method).unwrap();
    (session, workspace)
}
fn state(session: &SimulationSession) -> ([u8; 32], usize, Option<Arc<ProjectionAttempt>>) {
    (
        crate::workflow::input_digest(session.snapshot().state()),
        session.snapshot().frame(),
        session.snapshot().projection_attempt().cloned(),
    )
}
fn unchanged(
    session: &SimulationSession,
    before: &([u8; 32], usize, Option<Arc<ProjectionAttempt>>),
) {
    let after = state(session);
    assert_eq!(after.0, before.0);
    assert_eq!(after.1, before.1);
    match (&after.2, &before.2) {
        (Some(a), Some(b)) => assert!(Arc::ptr_eq(a, b)),
        (None, None) => {}
        _ => panic!("accepted receipt changed"),
    }
}
fn validate(session: &SimulationSession, attempt: &ProjectionAttempt) {
    attempt
        .validate(
            session.snapshot().configuration(),
            session.snapshot().grid(),
        )
        .unwrap();
}
#[test]
fn c1b2_private_faults_keep_clean_fallback_and_refuse_mixed_fallback_without_publication() {
    for method in PressureImplementation::ALL {
        let (mut session, mut workspace) = owner(method, 10);
        for fault in [
            ProjectionFault::WrongVelocity,
            ProjectionFault::NonFiniteVelocity,
        ] {
            workspace.set_fault(fault);
            let accepted = session
                .step_with_native_projection(inputs(), &mut workspace, None, |_| false)
                .unwrap();
            validate(&session, &accepted.attempt);
            assert_eq!(
                accepted.attempt.qualification.as_ref().unwrap().selection,
                ProjectionSelection::NativeFallback
            );
            assert!(
                accepted
                    .attempt
                    .qualification
                    .as_ref()
                    .unwrap()
                    .witness
                    .is_none()
            );
            let report = session.snapshot().provider_report().unwrap().clone();
            assert!(
                report.attempted.is_none()
                    && report.rejection.is_none()
                    && report.candidate.is_none()
            );
            assert_eq!(report.selected, method.id());
        }
        for (fault, reason) in [
            (
                ProjectionFault::BadGauge,
                ProjectionPressureRejection::NonZeroGauge,
            ),
            (
                ProjectionFault::BadPressure,
                ProjectionPressureRejection::QualityGate,
            ),
        ] {
            workspace.set_fault(fault);
            let before = state(&session);
            let failure = session
                .step_with_native_projection(inputs(), &mut workspace, None, |_| false)
                .err()
                .unwrap();
            assert!(matches!(
                failure.error,
                ProjectionStepError::UnrepresentableMixedQualificationFallback
            ));
            let attempt = failure.attempt.unwrap();
            validate(&session, &attempt);
            unchanged(&session, &before);
            let q = attempt.qualification.as_ref().unwrap();
            assert_eq!(
                q.rejection,
                Some(ProjectionRejectionFacts::Pressure(reason))
            );
            assert_eq!(
                q.witness.as_ref().unwrap().outcome,
                ProjectionWitnessOutcome::Rejected(reason)
            );
            assert!(q.native_fallback.is_some());
            assert!(attempt.selected_native.is_none());
            workspace.set_fault(ProjectionFault::None);
            let accepted = session
                .step_with_native_projection(inputs(), &mut workspace, None, |_| false)
                .unwrap();
            assert_eq!(accepted.attempt.attempt_id, attempt.attempt_id + 1);
            validate(&session, &accepted.attempt);
        }
    }
}
#[test]
fn c1b2_cancellation_at_every_observed_checkpoint_preserves_complete_publication() {
    for method in PressureImplementation::ALL {
        let (mut probe, mut probe_workspace) = owner(method, 10);
        let mut checkpoints = Vec::new();
        probe
            .step_with_native_projection(inputs(), &mut probe_workspace, None, |stage| {
                if !checkpoints.contains(&stage) {
                    checkpoints.push(stage);
                }
                false
            })
            .unwrap();
        assert!(checkpoints.contains(&ProjectionStepStage::ProducerPressurePoll));
        for point in [
            ProjectionCheckpoint::BeforeQualification,
            ProjectionCheckpoint::BeforePressure,
            ProjectionCheckpoint::PressurePoll,
            ProjectionCheckpoint::AfterPressure,
            ProjectionCheckpoint::BeforeReturn,
        ] {
            assert!(checkpoints.contains(&ProjectionStepStage::Qualification(point)));
        }
        assert!(checkpoints.contains(&ProjectionStepStage::Liquid(LiquidStepStage::BeforeCommit)));
        for target in checkpoints {
            let (mut session, mut workspace) = owner(method, 2);
            session
                .step_with_native_projection(inputs(), &mut workspace, None, |_| false)
                .unwrap();
            let before = state(&session);
            let mut observed = false;
            let failure = session
                .step_with_native_projection(inputs(), &mut workspace, None, |stage| {
                    if stage == target {
                        observed = true;
                        true
                    } else {
                        false
                    }
                })
                .err()
                .unwrap();
            assert!(observed, "{target:?}");
            unchanged(&session, &before);
            let attempt = failure.attempt.unwrap();
            validate(&session, &attempt);
            assert!(matches!(
                attempt.outcome,
                ProjectionAttemptOutcome::Cancelled { .. }
            ));
            let accepted = session
                .step_with_native_projection(inputs(), &mut workspace, None, |_| false)
                .unwrap();
            assert_eq!(accepted.attempt.attempt_id, attempt.attempt_id + 1);
            assert_eq!(accepted.report.carrier.step.generation, 2);
        }
    }
}
#[test]
fn c1b2_deadline_stop_quota_counter_and_arc_exclusivity_leave_accepted_state_unchanged() {
    let method = PressureImplementation::JacobiPcgV1;
    let (mut session, mut workspace) = owner(method, 10);
    session
        .step_with_native_projection(inputs(), &mut workspace, None, |_| false)
        .unwrap();
    let before = state(&session);
    let now = Instant::now();
    let failure = session
        .projection_step_with_clock(
            inputs(),
            &mut workspace,
            Some(now),
            |_| panic!("expired callback"),
            || now,
        )
        .err()
        .unwrap();
    assert!(matches!(
        failure.error,
        ProjectionStepError::Session(SessionError::Deadline)
    ));
    validate(&session, &failure.attempt.unwrap());
    unchanged(&session, &before);
    let deadline = now + std::time::Duration::from_secs(1);
    let clock = Cell::new(now);
    let failure = session
        .projection_step_with_clock(
            inputs(),
            &mut workspace,
            Some(deadline),
            |stage| {
                if stage == ProjectionStepStage::Liquid(LiquidStepStage::BeforeCommit) {
                    clock.set(deadline);
                }
                false
            },
            || clock.get(),
        )
        .err()
        .unwrap();
    assert!(matches!(
        failure.error,
        ProjectionStepError::Session(SessionError::Deadline)
    ));
    validate(&session, &failure.attempt.unwrap());
    unchanged(&session, &before);
    for fault in [
        ProjectionFault::AliasBeforeWork,
        ProjectionFault::AliasBeforeSeal,
    ] {
        workspace.set_fault(fault);
        let failure = session
            .step_with_native_projection(inputs(), &mut workspace, None, |_| false)
            .err()
            .unwrap();
        assert!(matches!(failure.error, ProjectionStepError::ReceiptAliased));
        validate(&session, &failure.attempt.unwrap());
        unchanged(&session, &before);
    }
    workspace.set_fault(ProjectionFault::None);
    session.projection_attempt_counter = u64::MAX;
    let failure = session
        .step_with_native_projection(inputs(), &mut workspace, None, |_| {
            panic!("exhausted callback")
        })
        .err()
        .unwrap();
    assert!(matches!(
        failure.error,
        ProjectionStepError::AttemptExhausted
    ));
    assert!(failure.attempt.is_none());
    unchanged(&session, &before);
    let (mut quota, mut w) = owner(method, 1);
    quota
        .step_with_native_projection(inputs(), &mut w, None, |_| false)
        .unwrap();
    let old = state(&quota);
    let failure = quota
        .step_with_native_projection(inputs(), &mut w, None, |_| panic!("quota callback"))
        .err()
        .unwrap();
    assert!(matches!(
        failure.error,
        ProjectionStepError::Session(SessionError::StepLimit)
    ));
    validate(&quota, &failure.attempt.unwrap());
    unchanged(&quota, &old);
    quota.stop();
    let failure = quota
        .step_with_native_projection(inputs(), &mut w, None, |_| panic!("stopped callback"))
        .err()
        .unwrap();
    assert!(matches!(
        failure.error,
        ProjectionStepError::Session(SessionError::Stopped)
    ));
    validate(&quota, &failure.attempt.unwrap());
    unchanged(&quota, &old);
}
#[test]
fn c1b2_late_volume_failure_and_native_pressure_failure_allow_clean_retry() {
    for method in PressureImplementation::ALL {
        let (mut session, mut workspace) = owner(method, 3);
        session
            .step_with_native_projection(inputs(), &mut workspace, None, |_| false)
            .unwrap();
        let before = state(&session);
        let mut invalid = inputs();
        invalid.volume.actual_divergence_limit = 0.0;
        let failure = session
            .step_with_native_projection(invalid, &mut workspace, None, |_| false)
            .err()
            .unwrap();
        assert!(matches!(
            failure.error,
            ProjectionStepError::Session(SessionError::Step(LiquidStepError::Volume(
                crate::LiquidVolumeError::DivergenceLimit { .. }
            )))
        ));
        let attempt = failure.attempt.unwrap();
        validate(&session, &attempt);
        assert!(attempt.qualification.is_some());
        unchanged(&session, &before);
        let accepted = session
            .step_with_native_projection(inputs(), &mut workspace, None, |_| false)
            .unwrap();
        assert_eq!(accepted.attempt.attempt_id, attempt.attempt_id + 1);
        // Existing positive fallback iteration budget is admitted; exhaustion is
        // an actual native-producer error, never invented candidate fallback.
        let mut cfg = config();
        cfg.carrier.pressure.max_iterations = 1;
        let grid = GridGeometry::new([4; 3], [0.25; 3], [0.0; 3]).unwrap();
        let mut owner = SimulationSession::new_with_projection(
            SessionIdentity {
                run_id: 105,
                source_revision: "native-exhaustion".into(),
            },
            grid.clone(),
            cfg,
            method,
            vec![0.5; grid.cell_len()],
            CAP,
            2,
            ProjectionIntent::BuiltinNativeV1,
        )
        .unwrap();
        let mut w = ProjectionStepWorkspace::new(&grid, 998.2, CAP, method).unwrap();
        let old = state(&owner);
        let failure = owner
            .step_with_native_projection(inputs(), &mut w, None, |_| false)
            .err()
            .unwrap();
        assert!(matches!(
            failure.error,
            ProjectionStepError::Session(SessionError::Step(LiquidStepError::Carrier(
                crate::SimulationError::Pressure(crate::PressureError::IterationLimit {
                    iterations: 1,
                    ..
                })
            )))
        ));
        let attempt = failure.attempt.unwrap();
        validate(&owner, &attempt);
        assert!(attempt.producer.is_none());
        assert!(attempt.qualification.is_none());
        unchanged(&owner, &old);
        let mut zero = inputs();
        zero.forces = &[];
        owner
            .step_with_native_projection(zero, &mut w, None, |_| false)
            .unwrap();
    }
}
#[test]
fn c1b2_receipt_validator_rejects_crosslinked_reports_and_impossible_phase_prefixes() {
    let (mut session, mut workspace) = owner(PressureImplementation::JacobiPcgV1, 3);
    let accepted = session
        .step_with_native_projection(inputs(), &mut workspace, None, |_| false)
        .unwrap();
    for case in 0..8 {
        let mut a = (*accepted.attempt).clone();
        match case {
            0 => a.selected_native.as_mut().unwrap().pressure.iterations = 0,
            1 => {
                a.qualification
                    .as_mut()
                    .unwrap()
                    .witness
                    .as_mut()
                    .unwrap()
                    .diagnostics
                    .as_mut()
                    .unwrap()
                    .iterations = 1
            }
            2 => {
                a.outcome = ProjectionAttemptOutcome::Cancelled {
                    stage: ProjectionAttemptStage::SessionGuard,
                };
                a.output_sha256 = None;
            }
            3 => {
                a.outcome = ProjectionAttemptOutcome::Refused {
                    stage: ProjectionAttemptStage::BeforeCommit,
                    code: ProjectionFailureCode::Stopped,
                };
                a.output_sha256 = None;
            }
            4 => {
                a.qualification.as_mut().unwrap().rejection =
                    Some(ProjectionRejectionFacts::MomentumWitness {
                        axis: Axis::X,
                        index: usize::MAX,
                    })
            }
            5 => a.qualification.as_mut().unwrap().selection = ProjectionSelection::NativeFallback,
            6 => a.provisional_input_id = None,
            7 => a.actual_dt = Some(a.requested_dt * 2.0),
            _ => unreachable!(),
        }
        assert!(
            a.validate(
                session.snapshot().configuration(),
                session.snapshot().grid()
            )
            .is_err(),
            "case {case}"
        );
    }
    workspace.set_fault(ProjectionFault::BadPressure);
    let failed = session
        .step_with_native_projection(inputs(), &mut workspace, None, |_| false)
        .err()
        .unwrap()
        .attempt
        .unwrap();
    let mut ieee = (*failed).clone();
    ieee.qualification
        .as_mut()
        .unwrap()
        .witness
        .as_mut()
        .unwrap()
        .diagnostics
        .as_mut()
        .unwrap()
        .true_residual_l2 = f64::NAN;
    // Rejected measured diagnostics retain their IEEE payload; acceptance never
    // borrows that rule. This is a codec-validation negative fixture only.
    validate(&session, &ieee);
}

#[test]
fn c1b2_preparation_refusals_have_valid_failed_receipts_before_work() {
    for method in PressureImplementation::ALL {
        let (mut session, mut workspace) = owner(method, 3);
        session
            .step_with_native_projection(inputs(), &mut workspace, None, |_| false)
            .unwrap();
        let before = state(&session);
        let bad_rate = [0.0];
        for case in 0..2 {
            let mut input = inputs();
            if case == 0 {
                input.volume.max_outward_courant = 0.0;
            } else {
                input.source = Some(
                    crate::LiquidVolumeSource::new(VolumeStamp { id: 9, version: 1 }, &bad_rate)
                        .unwrap(),
                );
            }
            let failure = session
                .step_with_native_projection(input, &mut workspace, None, |_| {
                    panic!("preparation callback")
                })
                .err()
                .unwrap();
            assert!(matches!(
                failure.error,
                ProjectionStepError::Session(SessionError::Step(LiquidStepError::Volume(_)))
            ));
            let attempt = failure.attempt.unwrap();
            validate(&session, &attempt);
            unchanged(&session, &before);
            assert!(matches!(
                attempt.outcome,
                ProjectionAttemptOutcome::Failed {
                    stage: ProjectionAttemptStage::CarrierPreparation,
                    ..
                }
            ));
            assert!(attempt.producer.is_none() && attempt.qualification.is_none());
        }
        for (grid, density, implementation) in [
            (
                GridGeometry::new([5; 3], [0.2; 3], [0.0; 3]).unwrap(),
                998.2,
                method,
            ),
            (session.snapshot().grid().clone(), 1.0, method),
            (
                session.snapshot().grid().clone(),
                998.2,
                if method == PressureImplementation::JacobiPcgV1 {
                    PressureImplementation::SymmetricGaussSeidelPcgV1
                } else {
                    PressureImplementation::JacobiPcgV1
                },
            ),
        ] {
            let mut wrong =
                ProjectionStepWorkspace::new(&grid, density, CAP, implementation).unwrap();
            let failure = session
                .step_with_native_projection(inputs(), &mut wrong, None, |_| {
                    panic!("workspace refusal callback")
                })
                .err()
                .unwrap();
            assert!(matches!(
                failure.error,
                ProjectionStepError::WorkspaceMismatch
            ));
            validate(&session, &failure.attempt.unwrap());
            unchanged(&session, &before);
        }
        let grid = session.snapshot().grid().clone();
        assert!(ProjectionStepWorkspace::new(&grid, 998.2, 0, method).is_err());
        assert!(ProjectionStepWorkspace::new(&grid, 0.0, CAP, method).is_err());
        let required = workspace.allocated_bytes();
        assert_eq!(
            ProjectionStepWorkspace::new(&grid, 998.2, required, method)
                .unwrap()
                .allocated_bytes(),
            required
        );
        assert!(ProjectionStepWorkspace::new(&grid, 998.2, required - 1, method).is_err());
        let mut cfg = config();
        cfg.carrier.pressure.max_iterations = 0;
        assert!(
            SimulationSession::new_with_projection(
                SessionIdentity {
                    run_id: 2,
                    source_revision: "invalid-positive-fallback".into()
                },
                grid.clone(),
                cfg,
                method,
                vec![0.5; grid.cell_len()],
                CAP,
                2,
                ProjectionIntent::BuiltinNativeV1
            )
            .is_err()
        );
    }
}
