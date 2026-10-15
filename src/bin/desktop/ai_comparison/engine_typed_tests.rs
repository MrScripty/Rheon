use super::*;
use rheon::recording::{
    RecordingLimits,
    diagnostic::{DiagnosticReader, DiagnosticRecorder, DiagnosticTerminal, TerminalOutcome},
};
use std::path::PathBuf;

struct Archive(PathBuf);
impl Archive {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "rheon-engine-attempts-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path.join("attempts.sqlite"))
    }
}
impl Drop for Archive {
    fn drop(&mut self) {
        std::fs::remove_dir_all(self.0.parent().unwrap()).unwrap();
    }
}
fn binding() -> BindingIdentity {
    BindingIdentity {
        id: "native-analytic-test".into(),
        version: "1".into(),
        manifest_sha256: [4; 32],
        checkpoint_sha256: None,
        training_run_id: None,
    }
}
fn native(input: Input<'_>) -> Result<Vec<f64>, PredictionFailure> {
    let mut solver = PressureWorkspace::with_implementation(
        input.context.grid(),
        128 * 1024 * 1024,
        PressureImplementation::JacobiPcgV1,
    )
    .unwrap();
    let op = PressureOperator::new(input.context.grid(), input.context.density()).unwrap();
    solver
        .solve_rhs(
            &op,
            input.context.rhs(),
            input.context.dt(),
            PressureSettings {
                relative_residual: 1e-10,
                absolute_residual: 1e-12,
                divergence_limit: 1e-7,
                max_iterations: 3000,
            },
            || false,
        )
        .unwrap();
    Ok(solver.pressure().to_vec())
}
fn scene() -> Scene {
    Scene {
        counts: [5; 3],
        steps: 32,
        warmup_steps: 2,
        ..Default::default()
    }
}
fn warmup(e: &mut Engine, flag: &AtomicBool) {
    for _ in 0..e.scene.warmup_steps {
        assert!(
            e.advance_typed(flag, |_| panic!("warmup dispatch"))
                .unwrap()
                .attempt
                .is_none()
        );
    }
}
#[test]
fn teacher_bias_never_changes_native_request_history_and_exact_clock_cold_verifies() {
    let flag = AtomicBool::new(false);
    let mut control = Engine::with_policy(
        scene(),
        ExecutionPolicy::TeacherStage,
        "same-test-request-identity".into(),
    )
    .unwrap();
    let mut biased = Engine::with_policy(
        scene(),
        ExecutionPolicy::TeacherStage,
        "same-test-request-identity".into(),
    )
    .unwrap();
    warmup(&mut control, &flag);
    warmup(&mut biased, &flag);
    let archive = Archive::new();
    let header = biased
        .diagnostic_run(binding(), "typed-engine-fixture".into())
        .unwrap();
    assert_eq!(
        header.seed_origin,
        Some(SeedOrigin::QualifiedNativeDiagnostic)
    );
    assert!(matches!(
        header.seed.as_ref().unwrap().status,
        PressureHistoryStatus::Diagnostic { qualified: true }
    ));
    let mut writer =
        DiagnosticRecorder::create(&archive.0, header, RecordingLimits::default()).unwrap();
    let mut generations = Vec::new();
    for _ in 2..32 {
        let a = control.advance_typed(&flag, native).unwrap();
        let b = biased
            .advance_typed(&flag, |input| {
                Ok(vec![0.0; input.context.grid().cell_len()])
            })
            .unwrap();
        let aa = a.attempt.unwrap();
        let bb = b.attempt.unwrap();
        assert_eq!(aa.request.digest(), bb.request.digest());
        assert_eq!(bb.transition, HistoryTransition::NativeTeacherAdvanced);
        assert!(matches!(&bb.outcome, PredictionOutcome::Candidate(p) if !p.observation.qualified));
        let x = a.frame.unwrap();
        let y = b.frame.unwrap();
        assert_eq!(x.reference_pressure, y.reference_pressure);
        assert_eq!(x.reference_tracer, y.reference_tracer);
        assert_eq!(x.time_s.to_bits(), y.time_s.to_bits());
        assert_eq!(control.reference.velocity, biased.reference.velocity);
        generations.push(bb.request.generation());
        writer.append(&bb).unwrap();
    }
    writer
        .finish(DiagnosticTerminal {
            outcome: TerminalOutcome::Completed,
            detail: "teacher fixture horizon".into(),
            produced_attempts: 30,
            last_advanced_generation: generations.last().copied(),
        })
        .unwrap();
    drop(writer);
    let reader = DiagnosticReader::open(&archive.0, RecordingLimits::default()).unwrap();
    reader.verify().unwrap();
    assert_eq!(reader.count().unwrap(), 30);
    let mut prior_end = reader.run().seed.as_ref().unwrap().interval_end;
    for i in 0..30 {
        let a = reader.select(i).unwrap();
        assert_eq!(a.request.interval_start().to_bits(), prior_end.to_bits());
        prior_end = a.request.interval_end();
    }
}
#[test]
fn independent_first_rejection_preserves_both_histories_and_equal_request_reference() {
    let flag = AtomicBool::new(false);
    let mut e = Engine::with_policy(
        scene(),
        ExecutionPolicy::IndependentStopOnReject,
        "stop-fixture".into(),
    )
    .unwrap();
    warmup(&mut e, &flag);
    let old = e.initial();
    let old_velocity = e.ai.velocity.clone();
    let archive = Archive::new();
    let mut writer = DiagnosticRecorder::create(
        &archive.0,
        e.diagnostic_run(binding(), "stop-fixture".into()).unwrap(),
        RecordingLimits::default(),
    )
    .unwrap();
    let result = e
        .advance_typed(&flag, |input| {
            Ok(vec![0.0; input.context.grid().cell_len()])
        })
        .unwrap();
    assert_eq!(result.stop, Some(AdvanceStop::Rejected));
    assert!(result.frame.is_none());
    assert_eq!(e.step, old.step);
    assert_eq!(e.clock.to_bits(), old.time_s.to_bits());
    assert_eq!(e.ai.pressure, old.ai_pressure);
    assert_eq!(e.reference.pressure, old.reference_pressure);
    assert_eq!(e.ai.velocity, old_velocity);
    let attempt = result.attempt.unwrap();
    assert_eq!(attempt.transition, HistoryTransition::NotAdvanced);
    let reference = attempt.native_reference.as_ref().unwrap();
    assert_eq!(reference.request_digest, attempt.request.digest());
    assert!(reference.sample.observation.qualified);
    writer.append(&attempt).unwrap();
    writer
        .finish(DiagnosticTerminal {
            outcome: TerminalOutcome::StoppedOnRejection,
            detail: "first rejection".into(),
            produced_attempts: 1,
            last_advanced_generation: None,
        })
        .unwrap();
    drop(writer);
    DiagnosticReader::open(&archive.0, RecordingLimits::default())
        .unwrap()
        .verify()
        .unwrap();
    assert!(e.advance_typed(&flag, native).is_err());
}
#[test]
fn independent_qualified_multistep_context_and_time_survive_cold_verification() {
    let flag = AtomicBool::new(false);
    let mut e = Engine::with_policy(
        scene(),
        ExecutionPolicy::IndependentStopOnReject,
        "qualified-fixture".into(),
    )
    .unwrap();
    warmup(&mut e, &flag);
    let archive = Archive::new();
    let mut writer = DiagnosticRecorder::create(
        &archive.0,
        e.diagnostic_run(binding(), "native exact candidate fixture".into())
            .unwrap(),
        RecordingLimits::default(),
    )
    .unwrap();
    let mut end = None;
    for _ in 2..32 {
        let result = e.advance_typed(&flag, native).unwrap();
        assert!(result.stop.is_none());
        let a = result.attempt.unwrap();
        assert_eq!(a.transition, HistoryTransition::DiagnosticAdvanced);
        end = Some(a.request.generation());
        writer.append(&a).unwrap();
    }
    writer
        .finish(DiagnosticTerminal {
            outcome: TerminalOutcome::Completed,
            detail: "qualified independent fixture".into(),
            produced_attempts: 30,
            last_advanced_generation: end,
        })
        .unwrap();
    drop(writer);
    DiagnosticReader::open(&archive.0, RecordingLimits::default())
        .unwrap()
        .verify()
        .unwrap();
}
#[test]
fn cancelled_qualified_candidate_is_retained_without_committing_or_publishing_frame() {
    for policy in [
        ExecutionPolicy::TeacherStage,
        ExecutionPolicy::IndependentStopOnReject,
    ] {
        let flag = AtomicBool::new(false);
        let mut e = Engine::with_policy(
            Scene {
                warmup_steps: 0,
                ..scene()
            },
            policy,
            "cancel-fixture".into(),
        )
        .unwrap();
        let archive = Archive::new();
        let mut writer = DiagnosticRecorder::create(
            &archive.0,
            e.diagnostic_run(binding(), "cancel fixture".into())
                .unwrap(),
            RecordingLimits::default(),
        )
        .unwrap();
        let result = e
            .advance_typed(&flag, |input| {
                let pressure = native(input);
                flag.store(true, Ordering::Relaxed);
                pressure
            })
            .unwrap();
        assert_eq!(result.stop, Some(AdvanceStop::Cancelled));
        assert!(result.frame.is_none());
        assert_eq!(e.step, 0);
        assert_eq!(e.clock, 0.0);
        let attempt = result.attempt.unwrap();
        assert!(
            matches!(&attempt.outcome,PredictionOutcome::Candidate(p) if p.observation.qualified)
        );
        assert_eq!(
            attempt.transition,
            HistoryTransition::CancelledBeforeAdvance
        );
        writer.append(&attempt).unwrap();
        writer
            .finish(DiagnosticTerminal {
                outcome: TerminalOutcome::Cancelled,
                detail: "cancel before commit".into(),
                produced_attempts: 1,
                last_advanced_generation: None,
            })
            .unwrap();
        drop(writer);
        DiagnosticReader::open(&archive.0, RecordingLimits::default())
            .unwrap()
            .verify()
            .unwrap();
    }
}
#[test]
fn malformed_output_is_explicit_unavailable_with_digest_and_no_advanced_frame() {
    let flag = AtomicBool::new(false);
    let mut e = Engine::with_policy(
        Scene {
            warmup_steps: 0,
            ..scene()
        },
        ExecutionPolicy::IndependentStopOnReject,
        "invalid-fixture".into(),
    )
    .unwrap();
    let result = e
        .advance_typed(&flag, |input| {
            let mut p = vec![f64::NAN; input.context.grid().cell_len()];
            p[0] = 0.0;
            Ok(p)
        })
        .unwrap();
    assert!(result.frame.is_none());
    assert_eq!(e.step, 0);
    assert!(matches!(
        result.attempt.unwrap().outcome,
        PredictionOutcome::NoCandidate {
            reason: NoCandidateReason::InvalidOutput,
            invalid_payload_sha256: None,
            ..
        }
    ));
}
