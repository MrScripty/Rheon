use super::*;
use crate::projection_step::{
    ProjectionAttemptOutcome, ProjectionFault, ProjectionStepStage, ProjectionStepWorkspace,
};
use crate::recording::{RecordingLimits, provider::runtime::RuntimeTerminalCoverage};
use crate::{
    GridGeometry, LiquidInlet, LiquidStepInputs, LiquidStepStage, LiquidTransportConfig,
    LiquidVolumeSettings, PressureImplementation, PressureSettings, SessionIdentity,
    SimulationConfig, SimulationSession, VolumeStamp,
};
use std::sync::atomic::{AtomicU64, Ordering};
const CAP: usize = 8 * 1024 * 1024;
fn identity() -> SessionIdentity {
    SessionIdentity {
        run_id: 911,
        source_revision: "c1b2-persistence-native-fixture".into(),
    }
}
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
fn fixture(method: PressureImplementation) -> (SimulationSession, ProjectionStepWorkspace) {
    let grid = GridGeometry::new([2; 3], [0.5; 3], [0.0; 3]).unwrap();
    let workspace = ProjectionStepWorkspace::new(&grid, 998.2, CAP, method).unwrap();
    (
        SimulationSession::new_with_projection(
            identity(),
            grid,
            config(),
            method,
            vec![0.5; 8],
            CAP,
            4,
            ProjectionIntent::BuiltinNativeV1,
        )
        .unwrap(),
        workspace,
    )
}
fn fault_fixture() -> (SimulationSession, ProjectionStepWorkspace) {
    let grid = GridGeometry::new([4; 3], [0.25; 3], [0.0; 3]).unwrap();
    let method = PressureImplementation::JacobiPcgV1;
    let workspace = ProjectionStepWorkspace::new(&grid, 998.2, CAP, method).unwrap();
    let session = SimulationSession::new_with_projection(
        identity(),
        grid,
        config(),
        method,
        vec![0.5; 64],
        CAP,
        4,
        ProjectionIntent::BuiltinNativeV1,
    )
    .unwrap();
    (session, workspace)
}

fn inputs() -> LiquidStepInputs<'static> {
    LiquidStepInputs {
        requested_dt: 0.03125,
        smoke_source: None,
        forces: &[],
        inlet: LiquidInlet::new(VolumeStamp { id: 3, version: 0 }, [[0.0; 2]; 3]).unwrap(),
        source: None,
        volume: LiquidVolumeSettings {
            max_outward_courant: 1.0,
            actual_divergence_limit: 1e-5,
        },
    }
}
fn path() -> std::path::PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    std::env::temp_dir().join(format!(
        "rheon-b2-recording-{}-{}.sqlite",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ))
}
fn unwrap_publication(
    run: &RecordingRun,
    row: &RecordedStep,
) -> (String, Option<Arc<ProjectionAttempt>>) {
    let (config, _) = configuration(run).unwrap();
    let grid = recording_grid(run).unwrap();
    let mut decoder =
        Decoder::text(&row.controls, profiles(configuration(run).unwrap().1).2).unwrap();
    assert_eq!(
        decoder.array::<32>().unwrap(),
        provider::source_binding(run)
    );
    let controls = get_wrapped(&mut decoder, "RCP2:").unwrap();
    let attempt = if decoder.boolean().unwrap() {
        Some(Arc::new(get_attempt(&mut decoder, &config, &grid).unwrap()))
    } else {
        None
    };
    decoder.end().unwrap();
    (controls, attempt)
}
fn wrap_publication(run: &RecordingRun, inner: &str, attempt: &ProjectionAttempt) -> String {
    let (config, _) = configuration(run).unwrap();
    let grid = recording_grid(run).unwrap();
    let mut encoder = Encoder::default();
    encoder.0.extend_from_slice(&provider::source_binding(run));
    put_wrapped(&mut encoder, inner, "RCP2:").unwrap();
    encoder.boolean(true);
    put_attempt(&mut encoder, attempt, &config, &grid).unwrap();
    encoder
        .text(profiles(configuration(run).unwrap().1).2)
        .unwrap()
}
fn assert_old_capture_refused(session: &SimulationSession) {
    assert!(matches!(
        provider::configured_run(&session.snapshot(), "wrong profile"),
        Err(RecordingError::Unsupported(_))
    ));
    assert!(matches!(
        provider::configured_publication(&session.snapshot(), 0, CAP),
        Err(RecordingError::Unsupported(_))
    ));
    assert!(matches!(
        provider::configured_publication_with_runtime(&session.snapshot(), 0, CAP, None),
        Err(RecordingError::Unsupported(_))
    ));
    assert!(matches!(
        super::super::workflow::configured_publication_with_workflow(
            &session.snapshot(),
            0,
            CAP,
            None
        ),
        Err(RecordingError::Unsupported(_))
    ));
}
#[test]
fn c1b2_projection_sqlite_round_trip_close_producer_reopen_exact_native_fields() {
    for method in PressureImplementation::ALL {
        let (mut session, mut workspace) = fixture(method);
        assert_old_capture_refused(&session);
        let run = projection_run(&session.snapshot(), "SQLite native projection").unwrap();
        let archive = path();
        let limits = RecordingLimits::default();
        let mut recorder = StepRecorder::create(&archive, run.clone(), limits).unwrap();
        let constructor = projection_publication(&session.snapshot(), 0, CAP).unwrap();
        let (inner, none) = unwrap_publication(&run, &constructor);
        assert!(none.is_none());
        let legacy = SimulationSession::new(
            identity(),
            session.snapshot().grid().clone(),
            config(),
            method,
            vec![0.5; 8],
            CAP,
            4,
        )
        .unwrap();
        let old_run = provider::configured_run(&legacy.snapshot(), &run.provenance).unwrap();
        let old_row = provider::configured_publication(&legacy.snapshot(), 0, CAP).unwrap();
        assert_eq!(inner, old_row.controls);
        assert_eq!(native_header(&run).unwrap().0, old_run);
        drop(legacy);
        recorder.append(&constructor).unwrap();
        let mut accepted = None;
        for ordinal in 1..=2 {
            let result = session
                .step_with_native_projection(inputs(), &mut workspace, None, |_| false)
                .unwrap();
            accepted = Some(result.attempt);
            assert_old_capture_refused(&session);
            let row = projection_publication(&session.snapshot(), ordinal, CAP).unwrap();
            let decoded = RecordedProjectionSession::decode(&run, &row).unwrap();
            assert_eq!(decoded.native.generation, ordinal as u64);
            assert_eq!(
                decoded.attempt.as_ref().unwrap().output_sha256,
                Some(super::super::workflow::native_state_digest(&decoded.native).unwrap())
            );
            let native = decoded.native.receipt.as_ref().unwrap();
            assert!(
                native.binding.is_none()
                    && native.context.is_none()
                    && !native.prediction_called
                    && native.attempted.is_none()
            );
            assert_eq!(native.selected, RecordedPressureSelection::Native(method));
            recorder.append(&row).unwrap();
        }
        ProjectionTermination::capture(
            &session.snapshot(),
            RuntimeTerminationCause::Completed,
            "done",
            accepted.as_ref(),
        )
        .unwrap()
        .finish(&mut recorder)
        .unwrap();
        drop(recorder);
        drop(accepted);
        drop(workspace);
        drop(session);
        let reader = RecordingReader::open(&archive, limits).unwrap();
        assert_eq!(reader.count().unwrap(), 3);
        assert_eq!(
            verify_projection_recording(&reader)
                .unwrap()
                .verified_prior_links,
            2
        );
        let row = reader.select(2).unwrap();
        let selected = selection(&reader, &row).unwrap();
        assert_eq!(selected.native.generation, 2);
        assert!(
            selected
                .attempt
                .as_ref()
                .unwrap()
                .producer
                .as_ref()
                .unwrap()
                .pressure
                .iterations
                <= 2000
        );
        let audit = terminal(&reader).unwrap();
        assert_eq!(audit.termination.accepted_generation, 2);
        assert_eq!(
            audit.coverage,
            RuntimeTerminalCoverage::VerifiedLastPublication
        );
        // None denotes unavailable G1 graph evidence, never a legacy/status claim.
        assert!(super::super::workflow::terminal(&reader).unwrap().is_none());
        drop(reader);
        std::fs::remove_file(archive).unwrap();
    }
}
#[test]
fn c1b2_projection_real_clean_fallback_and_mixed_failure_are_distinct_cold_facts() {
    for fault in [
        ProjectionFault::WrongVelocity,
        ProjectionFault::NonFiniteVelocity,
        ProjectionFault::BadGauge,
        ProjectionFault::BadPressure,
    ] {
        let (mut session, mut workspace) = fault_fixture();
        workspace.set_fault(fault);
        let run = projection_run(&session.snapshot(), "real bounded fault custody").unwrap();
        let archive = path();
        let limits = RecordingLimits::default();
        let mut recorder = StepRecorder::create(&archive, run, limits).unwrap();
        let result = session.step_with_native_projection(inputs(), &mut workspace, None, |_| false);
        let mixed = matches!(
            fault,
            ProjectionFault::BadGauge | ProjectionFault::BadPressure
        );
        let (attempt, cause) = match result {
            Ok(accepted) => {
                assert!(!mixed);
                let row = projection_publication(&session.snapshot(), 0, CAP).unwrap();
                recorder.append(&row).unwrap();
                (accepted.attempt, RuntimeTerminationCause::Completed)
            }
            Err(failure) => {
                assert!(mixed);
                assert_eq!(session.snapshot().state().carrier.generation, 0);
                (
                    failure.attempt.unwrap(),
                    RuntimeTerminationCause::NumericalFailure,
                )
            }
        };
        assert!(attempt.producer.is_some() && attempt.qualification.is_some());
        assert_eq!(attempt.selected_native.is_some(), !mixed);
        assert_eq!(publication_generation(&attempt).is_some(), !mixed);
        ProjectionTermination::capture(&session.snapshot(), cause, "fault result", Some(&attempt))
            .unwrap()
            .finish(&mut recorder)
            .unwrap();
        drop(recorder);
        drop(attempt);
        drop(session);
        drop(workspace);
        let reader = RecordingReader::open(&archive, limits).unwrap();
        let audit = terminal(&reader).unwrap();
        assert_eq!(reader.count().unwrap(), usize::from(!mixed));
        let attempt = audit.termination.attempt.unwrap();
        assert!(attempt.qualification.is_some());
        assert_eq!(attempt.output_sha256.is_some(), !mixed);
        drop(reader);
        std::fs::remove_file(archive).unwrap();
    }
}
#[test]
fn c1b2_projection_cancelled_generation_zero_has_terminal_but_no_accepted_row() {
    for before_work in [true, false] {
        let (mut session, mut workspace) = fixture(PressureImplementation::JacobiPcgV1);
        let run = projection_run(&session.snapshot(), "cancellation custody").unwrap();
        let failure = session
            .step_with_native_projection(inputs(), &mut workspace, None, |stage| {
                before_work || stage == ProjectionStepStage::Liquid(LiquidStepStage::BeforeCommit)
            })
            .err()
            .expect("the chosen cancellation must fail");
        let attempt = failure.attempt.unwrap();
        assert!(matches!(
            attempt.outcome,
            ProjectionAttemptOutcome::Cancelled { .. }
        ));
        assert_eq!(publication_generation(&attempt), None);
        let archive = path();
        let limits = RecordingLimits::default();
        let mut recorder = StepRecorder::create(&archive, run, limits).unwrap();
        ProjectionTermination::capture(
            &session.snapshot(),
            RuntimeTerminationCause::Cancelled,
            "cancelled",
            Some(&attempt),
        )
        .unwrap()
        .finish(&mut recorder)
        .unwrap();
        drop(recorder);
        drop(session);
        drop(workspace);
        let reader = RecordingReader::open(&archive, limits).unwrap();
        let audit = terminal(&reader).unwrap();
        assert_eq!(reader.count().unwrap(), 0);
        assert_eq!(audit.termination.accepted_generation, 0);
        assert_eq!(
            audit.coverage,
            RuntimeTerminalCoverage::NoRecordedPublication
        );
        assert_eq!(
            audit.termination.attempt.unwrap().producer.is_some(),
            !before_work
        );
        drop(reader);
        std::fs::remove_file(archive).unwrap();
    }
}
#[test]
fn c1b2_projection_actual_append_limit_preserves_unrecorded_accepted_tail() {
    let (mut session, mut workspace) = fixture(PressureImplementation::JacobiPcgV1);
    let run = projection_run(&session.snapshot(), "storage limit custody").unwrap();
    let archive = path();
    let limits = RecordingLimits {
        max_records: 1,
        ..RecordingLimits::default()
    };
    let mut recorder = StepRecorder::create(&archive, run, limits).unwrap();
    let constructor = projection_publication(&session.snapshot(), 0, CAP).unwrap();
    recorder.append(&constructor).unwrap();
    let accepted = session
        .step_with_native_projection(inputs(), &mut workspace, None, |_| false)
        .unwrap();
    let row = projection_publication(&session.snapshot(), 1, CAP).unwrap();
    assert!(matches!(
        recorder.append(&row),
        Err(RecordingError::Limit(_))
    ));
    ProjectionTermination::capture(
        &session.snapshot(),
        RuntimeTerminationCause::StorageFailure,
        "accepted row exceeds selected record count",
        Some(&accepted.attempt),
    )
    .unwrap()
    .finish(&mut recorder)
    .unwrap();
    drop(recorder);
    drop(session);
    drop(workspace);
    drop(accepted);
    let reader = RecordingReader::open(&archive, limits).unwrap();
    let audit = terminal(&reader).unwrap();
    assert_eq!(audit.termination.accepted_generation, 1);
    assert_eq!(
        audit.coverage,
        RuntimeTerminalCoverage::UnrecordedAcceptedTail {
            last_recorded_generation: Some(0)
        }
    );
    assert_eq!(reader.count().unwrap(), 1);
    drop(reader);
    std::fs::remove_file(archive).unwrap();
}
#[test]
fn c1b2_projection_rejects_profile_intent_size_source_interval_and_output_corruption() {
    let (mut session, mut workspace) = fixture(PressureImplementation::JacobiPcgV1);
    session
        .step_with_native_projection(inputs(), &mut workspace, None, |_| false)
        .unwrap();
    let run = projection_run(&session.snapshot(), "negative codec fixtures").unwrap();
    let mut row = projection_publication(&session.snapshot(), 0, CAP).unwrap();
    let good = row.controls.clone();
    let (inner, attempt) = unwrap_publication(&run, &row);
    let attempt = attempt.unwrap();
    let mut bad = run.clone();
    bad.solver = "rheon-native-projection-session-v2".into();
    assert!(matches!(
        configuration(&bad),
        Err(RecordingError::Unsupported(_))
    ));
    let mut bad = run.clone();
    bad.configuration = bad.configuration.replacen("RPS1:", "RPS2:", 1);
    assert!(matches!(
        configuration(&bad),
        Err(RecordingError::Unsupported(_))
    ));
    let mut bad = run.clone();
    bad.configuration.truncate(bad.configuration.len() - 2);
    bad.configuration.push_str("02");
    assert!(matches!(
        configuration(&bad),
        Err(RecordingError::Unsupported(_))
    ));
    let mut bad = run.clone();
    bad.configuration
        .push_str(&"0".repeat(super::super::MAX_TEXT));
    assert!(configuration(&bad).is_err());
    row.controls = good.replacen("RPP1:", "RPP2:", 1);
    assert!(matches!(
        RecordedProjectionSession::decode(&run, &row),
        Err(RecordingError::Unsupported(_))
    ));
    row.controls = good.clone();
    row.controls.pop();
    assert!(RecordedProjectionSession::decode(&run, &row).is_err());
    row.controls = good.clone() + &"0".repeat(super::super::MAX_TEXT);
    assert!(matches!(
        RecordedProjectionSession::decode(&run, &row),
        Err(RecordingError::Limit(_))
    ));
    let mut other = run.clone();
    other.identity.source_revision.push_str("-foreign");
    row.controls = wrap_publication(&other, &inner, &attempt);
    assert!(RecordedProjectionSession::decode(&run, &row).is_err());
    for variant in 0..3 {
        let mut bad = (*attempt).clone();
        match variant {
            0 => bad.output_sha256.as_mut().unwrap()[0] ^= 1,
            1 => bad.interval_start += 0.001,
            _ => bad.source.run_id += 1,
        };
        row.controls = wrap_publication(&run, &inner, &bad);
        assert!(RecordedProjectionSession::decode(&run, &row).is_err());
    }
    // Constructor-only archives must also reject a new-profile zero fallback
    // cap; the unchanged legacy RSC1 configuration remains readable.
    let (mut native, intent) = native_header(&run).unwrap();
    let mut d = Decoder::text(&native.configuration, "RSC1:").unwrap();
    let offset = 1 + 8 + 8 + 3 * 8;
    d.data[offset..offset + 8].copy_from_slice(&0u64.to_le_bytes());
    native.configuration = Encoder(d.data).text("RSC1:").unwrap();
    assert_eq!(
        provider::configured_configuration(&native)
            .unwrap()
            .transport
            .carrier
            .pressure
            .max_iterations,
        0
    );
    let mut e = Encoder::default();
    put_wrapped(&mut e, &native.configuration, "RSC1:").unwrap();
    put_intent(&mut e, intent);
    let mut bad = run.clone();
    bad.configuration = e.text(CONFIGURATION_PROFILE).unwrap();
    assert!(configuration(&bad).is_err());
}
#[test]
fn c1b2_projection_selected_and_history_reject_forged_input_and_duplicate_attempt() {
    for duplicate in [false, true] {
        let (mut session, mut workspace) = fixture(PressureImplementation::JacobiPcgV1);
        let run = projection_run(&session.snapshot(), "history negatives").unwrap();
        let constructor = projection_publication(&session.snapshot(), 0, CAP).unwrap();
        session
            .step_with_native_projection(inputs(), &mut workspace, None, |_| false)
            .unwrap();
        let first = projection_publication(&session.snapshot(), 1, CAP).unwrap();
        session
            .step_with_native_projection(inputs(), &mut workspace, None, |_| false)
            .unwrap();
        let mut second = projection_publication(&session.snapshot(), 2, CAP).unwrap();
        let (inner, attempt) = unwrap_publication(&run, &second);
        let mut forged = (*attempt.unwrap()).clone();
        if duplicate {
            forged.attempt_id = 1;
        } else {
            forged.input_sha256[0] ^= 1;
        }
        second.controls = wrap_publication(&run, &inner, &forged);
        RecordedProjectionSession::decode(&run, &second).unwrap();
        let archive = path();
        let limits = RecordingLimits::default();
        let mut recorder = StepRecorder::create(&archive, run, limits).unwrap();
        recorder.append(&constructor).unwrap();
        recorder.append(&first).unwrap();
        recorder.append(&second).unwrap();
        recorder
            .finish("failed", "authored cross-link rejection fixture", Some(2))
            .unwrap();
        drop(recorder);
        let reader = RecordingReader::open(&archive, limits).unwrap();
        assert!(selection(&reader, &second).is_err());
        assert!(verify_projection_recording(&reader).is_err());
        drop(reader);
        std::fs::remove_file(archive).unwrap();
    }
}
#[test]
fn c1b2_projection_terminal_rejects_unknown_source_version_cause_and_forged_snapshot() {
    let (mut session, mut workspace) = fixture(PressureImplementation::JacobiPcgV1);
    let accepted = session
        .step_with_native_projection(inputs(), &mut workspace, None, |_| false)
        .unwrap();
    assert!(matches!(
        ProjectionTermination::capture(
            &session.snapshot(),
            RuntimeTerminationCause::RuntimeFailure,
            "no runtime exists",
            Some(&accepted.attempt)
        ),
        Err(RecordingError::Unsupported(_))
    ));
    assert!(
        ProjectionTermination::capture(
            &session.snapshot(),
            RuntimeTerminationCause::StartupFailure,
            "work already exists",
            Some(&accepted.attempt)
        )
        .is_err()
    );
    let mut forged = (*accepted.attempt).clone();
    forged.output_sha256.as_mut().unwrap()[0] ^= 1;
    assert!(
        ProjectionTermination::capture(
            &session.snapshot(),
            RuntimeTerminationCause::StorageFailure,
            "bad native snapshot",
            Some(&Arc::new(forged))
        )
        .is_err()
    );
    let run = projection_run(&session.snapshot(), "terminal negative fixtures").unwrap();
    let actual = ProjectionTermination::capture(
        &session.snapshot(),
        RuntimeTerminationCause::StorageFailure,
        "unrecorded accepted tail",
        Some(&accepted.attempt),
    )
    .unwrap();
    let (config, _) = configuration(&run).unwrap();
    let grid = recording_grid(&run).unwrap();
    let encoded = actual
        .encode(&config, &grid, ProjectionIntent::BuiltinNativeV1)
        .unwrap();
    for corrupt in [
        encoded.replacen("RPT1:", "RPT2:", 1),
        encoded.clone() + &"0".repeat(super::super::MAX_TEXT),
    ] {
        let archive = path();
        let limits = RecordingLimits::default();
        let mut recorder = StepRecorder::create(&archive, run.clone(), limits).unwrap();
        if corrupt.len() > super::super::MAX_TEXT {
            assert!(recorder.finish("failed", &corrupt, Some(1)).is_err());
            drop(recorder);
            std::fs::remove_file(archive).unwrap();
            continue;
        }
        recorder.finish("failed", &corrupt, Some(1)).unwrap();
        drop(recorder);
        let reader = RecordingReader::open(&archive, limits).unwrap();
        assert!(terminal(&reader).is_err());
        drop(reader);
        std::fs::remove_file(archive).unwrap();
    }
    let mut stale = actual.clone();
    stale.source[0] ^= 1;
    let archive = path();
    let limits = RecordingLimits::default();
    let mut recorder = StepRecorder::create(&archive, run, limits).unwrap();
    recorder
        .finish(
            "failed",
            &stale
                .encode(&config, &grid, ProjectionIntent::BuiltinNativeV1)
                .unwrap(),
            Some(1),
        )
        .unwrap();
    drop(recorder);
    let reader = RecordingReader::open(&archive, limits).unwrap();
    assert!(terminal(&reader).is_err());
    drop(reader);
    std::fs::remove_file(archive).unwrap();
}
#[test]
fn c1b2_projection_cold_decoder_rejects_invalid_native_iteration_bytes() {
    let (mut session, mut workspace) = fixture(PressureImplementation::JacobiPcgV1);
    let accepted = session
        .step_with_native_projection(inputs(), &mut workspace, None, |_| false)
        .unwrap();
    let run = projection_run(&session.snapshot(), "iteration decoder negative").unwrap();
    let row = projection_publication(&session.snapshot(), 0, CAP).unwrap();
    let producer = accepted.attempt.producer.as_ref().unwrap();
    let mut signature = Encoder::default();
    put_method(&mut signature, producer.implementation);
    put_report(&mut signature, producer.pressure);
    signature.f64(producer.residual_l2_limit);
    put_duration(&mut signature, producer.elapsed);
    let mut data = Decoder::text(&row.controls, PUBLICATION_PROFILE)
        .unwrap()
        .data;
    let matches = data
        .windows(signature.0.len())
        .enumerate()
        .filter_map(|(i, bytes)| (bytes == signature.0.as_slice()).then_some(i))
        .collect::<Vec<_>>();
    assert_eq!(matches.len(), 1);
    let offset = matches[0] + 1;
    data[offset..offset + 8].copy_from_slice(&2001u64.to_le_bytes());
    let malformed = RecordedStep {
        controls: Encoder(data).text(PUBLICATION_PROFILE).unwrap(),
        ..row
    };
    let archive = path();
    let limits = RecordingLimits::default();
    let mut recorder = StepRecorder::create(&archive, run, limits).unwrap();
    recorder.append(&malformed).unwrap();
    recorder
        .finish("failed", "authored native iteration corruption", Some(1))
        .unwrap();
    drop(recorder);
    drop(session);
    drop(workspace);
    let reader = RecordingReader::open(&archive, limits).unwrap();
    assert!(RecordedProjectionSession::decode(reader.run(), &reader.select(0).unwrap()).is_err());
    drop(reader);
    std::fs::remove_file(archive).unwrap();
}

fn selection_fixture(
    producer: PressureImplementation,
    producer_max_iterations: usize,
) -> (SimulationSession, ProjectionStepWorkspace) {
    let fallback = PressureImplementation::ALL
        .into_iter()
        .find(|m| *m != producer)
        .unwrap();
    let intent = ProjectionIntent::ClassicalSelectionV1 {
        producer,
        producer_max_iterations,
    };
    let grid = GridGeometry::new([4; 3], [0.25; 3], [0.0; 3]).unwrap();
    let workspace =
        ProjectionStepWorkspace::new_with_intent(&grid, 998.2, CAP, fallback, intent).unwrap();
    let session = SimulationSession::new_with_projection(
        identity(),
        grid,
        config(),
        fallback,
        vec![0.5; 64],
        CAP,
        4,
        intent,
    )
    .unwrap();
    (session, workspace)
}
fn selection_inputs() -> LiquidStepInputs<'static> {
    const FORCE: [crate::BodyForce; 1] = [crate::BodyForce {
        value: [0.001, 0.0, 0.0],
        units: crate::ForceUnits::Acceleration,
        region: None,
    }];
    LiquidStepInputs {
        requested_dt: 0.001,
        forces: &FORCE,
        ..inputs()
    }
}

#[test]
fn c1b3_projection_sqlite_both_producers_and_real_iteration_failure_fallback() {
    use crate::projection_step::ProjectionProducerFailure;
    use crate::solver_provider::composition::projection::ProjectionSelection;
    for producer in PressureImplementation::ALL {
        for cap in [2000, 1] {
            let (mut session, mut workspace) = selection_fixture(producer, cap);
            let fallback = session.snapshot().configuration().implementation;
            assert_old_capture_refused(&session);
            let run =
                projection_run(&session.snapshot(), "B3 actual native method custody").unwrap();
            assert_eq!(run.solver, SELECTION_PROFILE);
            assert!(
                run.configuration
                    .starts_with(SELECTION_CONFIGURATION_PROFILE)
            );
            let archive = path();
            let limits = RecordingLimits::default();
            let mut recorder = StepRecorder::create(&archive, run.clone(), limits).unwrap();
            recorder
                .append(&projection_publication(&session.snapshot(), 0, CAP).unwrap())
                .unwrap();
            let accepted = session
                .step_with_native_projection(selection_inputs(), &mut workspace, None, |_| false)
                .unwrap();
            let row = projection_publication(&session.snapshot(), 1, CAP).unwrap();
            assert!(row.controls.starts_with(SELECTION_PUBLICATION_PROFILE));
            let decoded = RecordedProjectionSession::decode(&run, &row).unwrap();
            assert_eq!(decoded.native.configuration.implementation, fallback);
            let a = decoded.attempt.as_ref().unwrap();
            assert_eq!(a.version, 2);
            let selected = if cap == 1 {
                assert!(a.producer.is_none());
                let failure = a
                    .producer_failure
                    .as_ref()
                    .expect("fixed cap-one fixture must really exhaust producer");
                assert_eq!(failure.implementation, producer);
                assert!(
                    matches!(failure.failure,ProjectionProducerFailure::IterationLimit{iterations:1,residual_max} if residual_max.is_finite() && residual_max>=0.0)
                );
                let q = a.qualification.as_ref().unwrap();
                assert_eq!(q.selection, ProjectionSelection::NativeBaseline);
                assert!(q.witness.is_none() && q.rejection.is_none());
                fallback
            } else {
                assert!(a.producer_failure.is_none());
                assert_eq!(a.producer.as_ref().unwrap().implementation, producer);
                assert_eq!(
                    a.qualification.as_ref().unwrap().selection,
                    ProjectionSelection::SuppliedCandidate
                );
                producer
            };
            assert_eq!(
                decoded.native.receipt.as_ref().unwrap().selected,
                RecordedPressureSelection::Native(selected)
            );
            assert_eq!(a.selected_native.as_ref().unwrap().implementation, selected);
            let (outer_header, _) = native_header(&run).unwrap();
            let (inner, _) = unwrap_publication(&run, &row);
            // The old decoder retains its constructor-method rule. Only the
            // new wrapper's private method adapter can admit selected producer.
            assert_eq!(
                provider::decode_native_with_controls(&outer_header, &row, &inner).is_ok(),
                selected == fallback
            );
            recorder.append(&row).unwrap();
            ProjectionTermination::capture(
                &session.snapshot(),
                RuntimeTerminationCause::Completed,
                "selected method complete",
                Some(&accepted.attempt),
            )
            .unwrap()
            .finish(&mut recorder)
            .unwrap();
            drop(decoded);
            drop(accepted);
            drop(recorder);
            drop(session);
            drop(workspace);
            let reader = RecordingReader::open(&archive, limits).unwrap();
            let cold = selection(&reader, &reader.select(1).unwrap()).unwrap();
            assert_eq!(cold.native.configuration.implementation, fallback);
            assert_eq!(
                cold.native.receipt.as_ref().unwrap().selected,
                RecordedPressureSelection::Native(selected)
            );
            assert_eq!(
                verify_projection_recording(&reader)
                    .unwrap()
                    .verified_prior_links,
                1
            );
            let audit = terminal(&reader).unwrap();
            assert!(matches!(
                audit.coverage,
                RuntimeTerminalCoverage::VerifiedLastPublication
            ));
            assert_eq!(
                audit
                    .termination
                    .attempt
                    .unwrap()
                    .producer_failure
                    .is_some(),
                cap == 1
            );
            assert!(super::super::workflow::terminal(&reader).unwrap().is_none());
            drop(cold);
            drop(reader);
            std::fs::remove_file(archive).unwrap();
        }
    }
}

#[test]
fn c1b3_projection_cold_cancel_failed_and_accepted_unrecorded_tails() {
    use crate::solver_provider::composition::projection::ProjectionCheckpoint;
    for mode in 0..4 {
        let (mut session, mut workspace) = selection_fixture(
            PressureImplementation::JacobiPcgV1,
            if mode == 0 { 1 } else { 2000 },
        );
        let run = projection_run(&session.snapshot(), "B3 tail custody").unwrap();
        let archive = path();
        let limits = RecordingLimits::default();
        let mut recorder = StepRecorder::create(&archive, run, limits).unwrap();
        if mode == 3 {
            recorder
                .append(&projection_publication(&session.snapshot(), 0, CAP).unwrap())
                .unwrap();
        }
        if mode == 2 {
            workspace.set_fault(ProjectionFault::BadGauge);
        }
        let result = session.step_with_native_projection(
            selection_inputs(),
            &mut workspace,
            None,
            |stage| {
                mode == 0
                    && stage
                        == ProjectionStepStage::Qualification(ProjectionCheckpoint::BeforePressure)
                    || mode == 1
                        && stage == ProjectionStepStage::Liquid(LiquidStepStage::BeforeCommit)
            },
        );
        let (attempt, cause) = match result {
            Ok(a) => {
                assert_eq!(mode, 3);
                (a.attempt, RuntimeTerminationCause::StorageFailure)
            }
            Err(f) => {
                assert_ne!(mode, 3);
                (
                    f.attempt.unwrap(),
                    if mode == 2 {
                        RuntimeTerminationCause::NumericalFailure
                    } else {
                        RuntimeTerminationCause::Cancelled
                    },
                )
            }
        };
        if mode == 0 {
            assert!(attempt.producer_failure.is_some());
            assert!(attempt.qualification.is_none());
        }
        ProjectionTermination::capture(
            &session.snapshot(),
            cause,
            "observed unrecorded attempt",
            Some(&attempt),
        )
        .unwrap()
        .finish(&mut recorder)
        .unwrap();
        assert!(
            recorder
                .run()
                .configuration
                .starts_with(SELECTION_CONFIGURATION_PROFILE)
        );
        drop(attempt);
        drop(recorder);
        drop(session);
        drop(workspace);
        let reader = RecordingReader::open(&archive, limits).unwrap();
        assert!(
            reader
                .status()
                .unwrap()
                .detail
                .starts_with(SELECTION_TERMINAL_PROFILE)
        );
        let audit = terminal(&reader).unwrap();
        assert_eq!(audit.termination.accepted_generation, u64::from(mode == 3));
        assert_eq!(audit.termination.attempt.as_ref().unwrap().version, 2);
        if mode == 3 {
            assert!(matches!(
                audit.coverage,
                RuntimeTerminalCoverage::UnrecordedAcceptedTail {
                    last_recorded_generation: Some(0)
                }
            ));
        } else {
            assert!(matches!(
                audit.coverage,
                RuntimeTerminalCoverage::NoRecordedPublication
            ));
        }
        drop(reader);
        std::fs::remove_file(archive).unwrap();
    }
}

#[test]
fn c1b3_projection_selected_method_source_budget_and_profile_corruptions_refuse() {
    let producer = PressureImplementation::JacobiPcgV1;
    let (mut session, mut workspace) = selection_fixture(producer, 2000);
    let run = projection_run(&session.snapshot(), "B3 ingress negatives").unwrap();
    session
        .step_with_native_projection(selection_inputs(), &mut workspace, None, |_| false)
        .unwrap();
    let row = projection_publication(&session.snapshot(), 0, CAP).unwrap();
    let (inner, attempt) = unwrap_publication(&run, &row);
    let attempt = attempt.unwrap();
    let (header, _) = native_header(&run).unwrap();
    // Rebind valid RCP2 bytes to the fallback header without changing the outer
    // producer facts: matching metrics alone must not authorize the method.
    let mut raw = Decoder::text(&inner, "RCP2:").unwrap().data;
    raw[..32].copy_from_slice(&provider::source_binding(&header));
    let bad_inner = Encoder(raw).text("RCP2:").unwrap();
    let mut bad = projection_publication(&session.snapshot(), row.ordinal, CAP).unwrap();
    bad.controls = wrap_publication(&run, &bad_inner, &attempt);
    assert!(RecordedProjectionSession::decode(&run, &bad).is_err());
    for bad_intent in [
        ProjectionIntent::BuiltinNativeV1,
        ProjectionIntent::ClassicalSelectionV1 {
            producer,
            producer_max_iterations: 0,
        },
        ProjectionIntent::ClassicalSelectionV1 {
            producer,
            producer_max_iterations: 2001,
        },
        ProjectionIntent::ClassicalSelectionV1 {
            producer: session.snapshot().configuration().implementation,
            producer_max_iterations: 2000,
        },
    ] {
        let mut e = Encoder::default();
        put_wrapped(&mut e, &header.configuration, "RSC1:").unwrap();
        put_intent(&mut e, bad_intent);
        let mut bad_run = run.clone();
        bad_run.configuration = e.text(SELECTION_CONFIGURATION_PROFILE).unwrap();
        assert!(configuration(&bad_run).is_err());
    }
    for change in 0..5 {
        let mut bad = projection_publication(&session.snapshot(), row.ordinal, CAP).unwrap();
        match change {
            0 => bad.controls = bad.controls.replacen("RPP2:", "RPP1:", 1),
            1 => bad.controls = bad.controls.replacen("RPP2:", "RPP3:", 1),
            2 => bad.controls.push_str(&"0".repeat(super::super::MAX_TEXT)),
            3 => bad.time += 0.001,
            _ => {
                let mut d = Decoder::text(&bad.controls, SELECTION_PUBLICATION_PROFILE).unwrap();
                d.data[0] ^= 1;
                bad.controls = Encoder(d.data).text(SELECTION_PUBLICATION_PROFILE).unwrap();
            }
        }
        assert!(RecordedProjectionSession::decode(&run, &bad).is_err());
    }
    // Version is structurally located after source, wrapped RCP2 and presence.
    let mut d = Decoder::text(&row.controls, SELECTION_PUBLICATION_PROFILE).unwrap();
    d.array::<32>().unwrap();
    get_wrapped(&mut d, "RCP2:").unwrap();
    assert!(d.boolean().unwrap());
    let offset = 32 + 4 + Decoder::text(&inner, "RCP2:").unwrap().data.len() + 1;
    d.data[offset..offset + 4].copy_from_slice(&1u32.to_le_bytes());
    let bad = RecordedStep {
        controls: Encoder(d.data).text(SELECTION_PUBLICATION_PROFILE).unwrap(),
        ..projection_publication(&session.snapshot(), row.ordinal, CAP).unwrap()
    };
    assert!(RecordedProjectionSession::decode(&run, &bad).is_err());
    let mut old_profile = run.clone();
    old_profile.solver = PROFILE.into();
    assert!(configuration(&old_profile).is_err());
    assert!(provider::configured_configuration(&run).is_err());
}

#[test]
fn c1b3_projection_cold_iteration_failure_corruption_and_history_links_refuse() {
    use crate::projection_step::ProjectionProducerFailure;
    let (mut session, mut workspace) = selection_fixture(PressureImplementation::JacobiPcgV1, 1);
    let run = projection_run(&session.snapshot(), "B3 cold corruptions").unwrap();
    let constructor = projection_publication(&session.snapshot(), 0, CAP).unwrap();
    let result = session
        .step_with_native_projection(selection_inputs(), &mut workspace, None, |_| false)
        .unwrap();
    let row = projection_publication(&session.snapshot(), 1, CAP).unwrap();
    let a = &result.attempt;
    let failure = a
        .producer_failure
        .as_ref()
        .expect("fixed cap-one fixture must exhaust");
    let mut signature = Encoder::default();
    put_method(&mut signature, failure.implementation);
    let ProjectionProducerFailure::IterationLimit {
        iterations,
        residual_max,
    } = failure.failure;
    signature.u8(0);
    signature.u64(iterations as u64);
    signature.f64(residual_max);
    signature.f64(failure.residual_l2_limit);
    put_duration(&mut signature, failure.elapsed);
    for change in 0..5 {
        let mut bad = projection_publication(&session.snapshot(), row.ordinal, CAP).unwrap();
        let mut d = Decoder::text(&row.controls, SELECTION_PUBLICATION_PROFILE).unwrap();
        let offsets: Vec<_> = d
            .data
            .windows(signature.0.len())
            .enumerate()
            .filter_map(|(i, b)| (b == signature.0).then_some(i))
            .collect();
        assert_eq!(
            offsets.len(),
            1,
            "structural typed failure signature must be unique"
        );
        let start = offsets[0];
        match change {
            0 => d.data[start + 2..start + 10].copy_from_slice(&0u64.to_le_bytes()),
            1 => d.data[start + 10..start + 18].copy_from_slice(&f64::NAN.to_le_bytes()),
            2 => d.data[start] = if d.data[start] == 0 { 1 } else { 0 },
            _ => {}
        }
        bad.controls = Encoder(d.data).text(SELECTION_PUBLICATION_PROFILE).unwrap();
        if change >= 3 {
            let (inner, attempt) = unwrap_publication(&run, &row);
            let mut altered = (*attempt.unwrap()).clone();
            if change == 3 {
                altered.input_sha256[0] ^= 1;
            } else {
                altered.output_sha256.as_mut().unwrap()[0] ^= 1;
            }
            bad.controls = wrap_publication(&run, &inner, &altered);
        }
        let archive = path();
        let limits = RecordingLimits::default();
        let mut recorder = StepRecorder::create(&archive, run.clone(), limits).unwrap();
        recorder.append(&constructor).unwrap();
        recorder.append(&bad).unwrap();
        recorder
            .finish("failed", "authored B3 raw codec corruption", Some(1))
            .unwrap();
        drop(recorder);
        let reader = RecordingReader::open(&archive, limits).unwrap();
        assert!(selection(&reader, &reader.select(1).unwrap()).is_err());
        assert!(verify_projection_recording(&reader).is_err());
        drop(reader);
        std::fs::remove_file(archive).unwrap();
    }
    // Duplicate attempt identities are separately rejected across valid retained
    // rows, even when all new v2 failure facts are individually admissible.
    session
        .step_with_native_projection(selection_inputs(), &mut workspace, None, |_| false)
        .unwrap();
    let second = projection_publication(&session.snapshot(), 2, CAP).unwrap();
    let (inner, attempt) = unwrap_publication(&run, &second);
    let mut altered = (*attempt.unwrap()).clone();
    altered.attempt_id = a.attempt_id;
    let second = RecordedStep {
        controls: wrap_publication(&run, &inner, &altered),
        ..second
    };
    let archive = path();
    let limits = RecordingLimits::default();
    let mut recorder = StepRecorder::create(&archive, run, limits).unwrap();
    recorder.append(&constructor).unwrap();
    recorder.append(&row).unwrap();
    recorder.append(&second).unwrap();
    recorder
        .finish("failed", "duplicate B3 attempt", Some(2))
        .unwrap();
    drop(recorder);
    let reader = RecordingReader::open(&archive, limits).unwrap();
    assert!(verify_projection_recording(&reader).is_err());
    drop(reader);
    std::fs::remove_file(archive).unwrap();
}

#[test]
fn c1b3_projection_intent_decoder_refuses_foreign_version_before_payload() {
    // The foreign tag is sufficient for Unsupported: v1 must never consume
    // the new producer/method payload and turn this into a truncation error.
    for (version, tag) in [(1, 2), (2, 1)] {
        let mut decoder =
            Decoder::text(&Encoder(vec![tag]).text("test:").unwrap(), "test:").unwrap();
        assert!(matches!(
            get_intent(&mut decoder, version),
            Err(RecordingError::Unsupported(_))
        ));
    }
    let mut decoder = Decoder::text(&Encoder(vec![2]).text("test:").unwrap(), "test:").unwrap();
    assert!(matches!(
        get_intent(&mut decoder, 2),
        Err(RecordingError::Invalid(_))
    ));
}
