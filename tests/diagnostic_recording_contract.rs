#![cfg(not(target_arch = "wasm32"))]
use rheon::{
    GridGeometry, PressureOperator, SessionIdentity,
    recording::{
        RecordingError, RecordingLimits, RecordingReader, RecordingRun, StepRecorder, diagnostic::*,
    },
    solver_provider::context::{
        PressureContext, PressureContextInput, PressureContextMode, PressureHistory,
        PressureHistoryStatus,
    },
};
use rusqlite::{Connection, params};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

struct Archive(PathBuf);
impl Archive {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "rheon-diagnostic-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&dir).unwrap();
        Self(dir.join("diagnostic.sqlite"))
    }
    fn reader(&self) -> DiagnosticReader {
        DiagnosticReader::open(&self.0, RecordingLimits::default()).unwrap()
    }
}
impl Drop for Archive {
    fn drop(&mut self) {
        std::fs::remove_dir_all(self.0.parent().unwrap()).unwrap();
    }
}
fn pressure(n: usize, scale: f64) -> Vec<f64> {
    (0..n).map(|i| i as f64 * scale).collect()
}
fn run(mode: DiagnosticMode, seed: bool) -> DiagnosticRun {
    let grid = GridGeometry::new([2, 3, 4], [0.25, 0.5, 0.75], [-1.0, 2.0, 3.0]).unwrap();
    let seed = seed.then(|| ObservedHistory {
        generation: 7,
        interval_end: 0.07,
        status: if mode == DiagnosticMode::TeacherStage {
            PressureHistoryStatus::Accepted
        } else {
            PressureHistoryStatus::Diagnostic { qualified: true }
        },
        pressure: pressure(grid.cell_len(), 0.125),
    });
    DiagnosticRun {
        run_id: "diagnostic-track".into(),
        source_revision: "fixture-source".into(),
        grid,
        density: 1000.0,
        viscosity: 0.001,
        dt: 0.01,
        mode,
        native_prefix_steps: if seed.is_some() { 7 } else { 0 },
        binding: BindingIdentity {
            id: "fixture-provider".into(),
            version: "1".into(),
            manifest_sha256: [7; 32],
            checkpoint_sha256: Some([9; 32]),
            training_run_id: Some("fixture-only".into()),
        },
        provenance: "manufactured bounded storage fixture; qualification is a host observation"
            .into(),
        seed,
    }
}
fn request(run: &DiagnosticRun, prior: Option<&ObservedHistory>) -> DiagnosticRequest {
    let mut rhs = vec![0.0; run.grid.cell_len()];
    PressureOperator::new(&run.grid, run.density)
        .unwrap()
        .apply_full(&pressure(run.grid.cell_len(), 0.25), &mut rhs)
        .unwrap();
    let context = PressureContext::new(PressureContextInput {
        grid: &run.grid,
        density: run.density,
        dt: run.dt,
        interval_start: prior.map_or(0.0, |p| p.interval_end),
        rhs: &rhs,
        prior: prior.map(|p| PressureHistory {
            values: &p.pressure,
            grid: &run.grid,
            density: run.density,
            run_id: &run.run_id,
            generation: p.generation,
            interval_end: p.interval_end,
            status: p.status,
        }),
        run_id: &run.run_id,
        generation: prior.map_or(1, |p| p.generation + 1),
        mode: PressureContextMode::DiagnosticCandidate,
    })
    .unwrap();
    DiagnosticRequest::capture(&context, run.viscosity, 1 << 20).unwrap()
}
fn sample(run: &DiagnosticRun, scale: f64, qualified: bool) -> PressureSample {
    PressureSample {
        pressure: pressure(run.grid.cell_len(), scale),
        observation: QualificationObservation {
            policy: QualificationPolicy::HostPressureGateV1,
            residual_l2: 0.0,
            relative_residual: Some(0.0),
            predicted_divergence: 0.0,
            actual_divergence: 0.0,
            courant: 0.01,
            net_flux: 0.0,
            relative_limit: 1e-6,
            absolute_limit: 1e-12,
            predicted_limit: 1e-7,
            actual_limit: 1e-5,
            courant_limit: 0.5,
            flux_limit: 1e-12,
            qualified,
            hard_stop: false,
        },
    }
}
fn attempt(
    run: &DiagnosticRun,
    index: usize,
    prior: Option<&ObservedHistory>,
    qualified: bool,
) -> DiagnosticAttempt {
    let request = request(run, prior);
    let native_reference = (run.mode == DiagnosticMode::TeacherStage).then(|| NativeReference {
        request_digest: request.digest(),
        sample: sample(run, 0.25, true),
        history_status: PressureHistoryStatus::Diagnostic { qualified: true },
    });
    DiagnosticAttempt {
        attempt_index: index,
        request,
        dispatched: true,
        outcome: PredictionOutcome::Candidate(sample(run, 8.0, qualified)),
        native_reference,
        transition: match run.mode {
            DiagnosticMode::TeacherStage => HistoryTransition::NativeTeacherAdvanced,
            DiagnosticMode::IndependentStopOnReject if qualified => {
                HistoryTransition::DiagnosticAdvanced
            }
            _ => HistoryTransition::NotAdvanced,
        },
    }
}
fn advanced(a: &DiagnosticAttempt, teacher: bool) -> ObservedHistory {
    let sample = if teacher {
        &a.native_reference.as_ref().unwrap().sample
    } else {
        match &a.outcome {
            PredictionOutcome::Candidate(s) => s,
            _ => panic!("candidate required"),
        }
    };
    ObservedHistory {
        generation: a.request.generation(),
        interval_end: a.request.interval_end(),
        status: if teacher {
            a.native_reference.as_ref().unwrap().history_status
        } else {
            PressureHistoryStatus::Diagnostic {
                qualified: sample.observation.qualified,
            }
        },
        pressure: sample.pressure.clone(),
    }
}
fn terminal(outcome: TerminalOutcome, count: usize, advanced: Option<u64>) -> DiagnosticTerminal {
    DiagnosticTerminal {
        outcome,
        detail: "fixture terminal observation".into(),
        produced_attempts: count,
        last_advanced_generation: advanced,
    }
}

#[test]
fn sqlite_close_reopen_preserves_exact_native_bits_context_binding_and_terminal() {
    let archive = Archive::new();
    let run = run(DiagnosticMode::IndependentStopOnReject, true);
    let first = attempt(&run, 0, run.seed.as_ref(), true);
    let prior = advanced(&first, false);
    let second = attempt(&run, 1, Some(&prior), true);
    let mut writer =
        DiagnosticRecorder::create(&archive.0, run.clone(), RecordingLimits::default()).unwrap();
    assert_eq!(writer.run(), &run);
    assert_eq!(writer.path(), archive.0);
    writer.append(&first).unwrap();
    writer.append(&second).unwrap();
    assert_eq!(writer.recorded_count(), 2);
    let status = terminal(
        TerminalOutcome::Completed,
        2,
        Some(second.request.generation()),
    );
    writer.finish(status.clone()).unwrap();
    assert!(matches!(
        writer.append(&second),
        Err(RecordingError::Closed)
    ));
    drop(writer);
    let reader = archive.reader();
    reader.verify().unwrap();
    assert_eq!(reader.run(), &run);
    assert_eq!(reader.count().unwrap(), 2);
    assert_eq!(reader.select(0).unwrap(), first);
    assert_eq!(reader.select(1).unwrap(), second);
    assert_eq!(reader.status().unwrap(), Some(status));
    assert_eq!(
        reader
            .select(0)
            .unwrap()
            .request
            .prior()
            .unwrap()
            .generation,
        7
    );
    let db = Connection::open(&archive.0).unwrap();
    assert_eq!(
        db.pragma_query_value::<i64, _>(None, "application_id", |r| r.get(0))
            .unwrap(),
        APPLICATION_ID
    );
    assert_eq!(
        db.pragma_query_value::<i64, _>(None, "user_version", |r| r.get(0))
            .unwrap(),
        VERSION
    );
    let bytes: Vec<u8> = db
        .query_row("SELECT rhs FROM attempts WHERE attempt_index=0", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(&bytes[..8], &first.request.rhs()[0].to_le_bytes());
}
#[test]
fn teacher_history_uses_native_equal_request_sample_despite_biased_rejected_model() {
    let archive = Archive::new();
    let run = run(DiagnosticMode::TeacherStage, true);
    let first = attempt(&run, 0, run.seed.as_ref(), false);
    let native_prior = advanced(&first, true);
    let biased_prior = advanced(&first, false);
    let second = attempt(&run, 1, Some(&native_prior), false);
    let mut writer =
        DiagnosticRecorder::create(&archive.0, run.clone(), RecordingLimits::default()).unwrap();
    writer.append(&first).unwrap();
    writer.append(&second).unwrap();
    writer
        .finish(terminal(
            TerminalOutcome::Completed,
            2,
            Some(second.request.generation()),
        ))
        .unwrap();
    drop(writer);
    let reader = archive.reader();
    reader.verify().unwrap();
    assert_eq!(
        reader.select(1).unwrap().request.prior(),
        Some(&native_prior)
    );
    assert_ne!(native_prior.pressure, biased_prior.pressure);
    let wrong = Archive::new();
    let mut writer =
        DiagnosticRecorder::create(&wrong.0, run.clone(), RecordingLimits::default()).unwrap();
    writer.append(&first).unwrap();
    assert!(matches!(
        writer.append(&attempt(&run, 1, Some(&biased_prior), false)),
        Err(RecordingError::Invalid("diagnostic history continuity"))
    ));
    writer
        .finish(terminal(
            TerminalOutcome::Failed,
            2,
            Some(second.request.generation()),
        ))
        .unwrap();
}
#[test]
fn teacher_multi_attempts_preserve_actual_accepted_native_history_status() {
    use rheon::{
        PressureSettings, PressureWorkspace, ProviderStepWorkspace, Simulation, SimulationConfig,
        SmokeSource,
        solver_provider::{
            PressureProvider, PressureRequest, ProviderRejection, SolverMetadata, TensorMetadata,
        },
    };
    struct NativeCapture {
        grid: GridGeometry,
        solver: PressureWorkspace,
        settings: PressureSettings,
        prior: Option<ObservedHistory>,
        request: Option<DiagnosticRequest>,
        output: Vec<f64>,
    }
    impl PressureProvider for NativeCapture {
        fn metadata(&self) -> SolverMetadata {
            let field = TensorMetadata {
                shape_xyz: self.grid.counts(),
                layout: "x-fast",
                scalar: "f64",
                units: "fixture physical field",
            };
            SolverMetadata {
                id: "native-storage-fixture",
                version: "1",
                stage: "pressure",
                input: field.clone(),
                output: field,
                domain: "native sealed-box fixture",
                provenance: None,
            }
        }
        fn supports(&self, r: &PressureRequest<'_>) -> Result<(), ProviderRejection> {
            if r.grid != &self.grid {
                return Err(ProviderRejection::Unsupported("fixture grid"));
            }
            Ok(())
        }
        fn predict(
            &mut self,
            r: &PressureRequest<'_>,
            out: &mut [f64],
            cancelled: &mut dyn FnMut() -> bool,
        ) -> Result<(), ProviderRejection> {
            let context = PressureContext::new(PressureContextInput {
                grid: r.grid,
                density: r.density,
                dt: r.dt,
                interval_start: self.prior.as_ref().map_or(0.0, |p| p.interval_end),
                rhs: r.rhs,
                prior: self.prior.as_ref().map(|p| PressureHistory {
                    values: &p.pressure,
                    grid: r.grid,
                    density: r.density,
                    run_id: "diagnostic-track",
                    generation: p.generation,
                    interval_end: p.interval_end,
                    status: p.status,
                }),
                run_id: "diagnostic-track",
                generation: self.prior.as_ref().map_or(1, |p| p.generation + 1),
                mode: PressureContextMode::DiagnosticCandidate,
            })
            .map_err(|_| ProviderRejection::Execution("fixture context"))?;
            self.request = Some(
                DiagnosticRequest::capture(&context, 0.0, 1 << 20)
                    .map_err(|_| ProviderRejection::Execution("fixture capture"))?,
            );
            self.solver
                .solve_rhs(
                    &PressureOperator::new(r.grid, r.density).unwrap(),
                    r.rhs,
                    r.dt,
                    self.settings,
                    cancelled,
                )
                .map_err(|_| ProviderRejection::Execution("fixture native solve"))?;
            out.copy_from_slice(self.solver.pressure());
            self.output = out.to_vec();
            Ok(())
        }
    }
    let archive = Archive::new();
    let mut run = run(DiagnosticMode::TeacherStage, false);
    run.viscosity = 0.0;
    let settings = PressureSettings {
        relative_residual: 1e-8,
        absolute_residual: 1e-12,
        divergence_limit: 1e-7,
        max_iterations: 3000,
    };
    let config = SimulationConfig {
        density: run.density,
        memory_limit: 64 << 20,
        pressure: settings,
        actual_divergence_limit: 1e-5,
        max_courant: 0.5,
    };
    let mut native = Simulation::new(run.grid.clone(), config).unwrap();
    let mut control = Simulation::new(run.grid.clone(), config).unwrap();
    let mut workspace =
        ProviderStepWorkspace::new(&run.grid, run.density, 64 << 20, native.implementation())
            .unwrap();
    let mut provider = NativeCapture {
        grid: run.grid.clone(),
        solver: PressureWorkspace::new(&run.grid, 64 << 20).unwrap(),
        settings,
        prior: None,
        request: None,
        output: Vec::new(),
    };
    let source = SmokeSource {
        lower: [-0.99, 2.1, 3.1],
        upper: [-0.51, 2.9, 4.4],
        tracer_rate: 1.0,
        vertical_acceleration: 0.5,
    };
    // The first actual committed native step is the observed warmup seed.
    let report = native
        .step_with_pressure_provider(
            run.dt,
            Some(source),
            &[],
            &mut workspace,
            Some(&mut provider),
            |_| false,
        )
        .unwrap();
    assert_eq!(report.provider.selected, "native-storage-fixture");
    assert!(report.provider.rejection.is_none());
    control.step(run.dt, Some(source), |_| false).unwrap();
    let mut prior = ObservedHistory {
        generation: native.state().generation,
        interval_end: native.state().time,
        status: PressureHistoryStatus::Accepted,
        pressure: provider.output.clone(),
    };
    run.seed = Some(prior.clone());
    run.native_prefix_steps = 1;
    let mut writer =
        DiagnosticRecorder::create(&archive.0, run.clone(), RecordingLimits::default()).unwrap();
    for index in 0..2 {
        provider.prior = Some(prior.clone());
        let report = native
            .step_with_pressure_provider(
                run.dt,
                Some(source),
                &[],
                &mut workspace,
                Some(&mut provider),
                |_| false,
            )
            .unwrap();
        assert_eq!(report.provider.selected, "native-storage-fixture");
        assert!(report.provider.rejection.is_none());
        control.step(run.dt, Some(source), |_| false).unwrap();
        assert_eq!(native.state().x, control.state().x);
        assert_eq!(native.state().y, control.state().y);
        assert_eq!(native.state().z, control.state().z);
        let request = provider.request.take().unwrap();
        assert_eq!(
            request.prior().unwrap().status,
            PressureHistoryStatus::Accepted
        );
        let mut observed = sample(&run, 0.0, true).observation;
        observed.residual_l2 = report.provider.accepted.true_residual_l2;
        let norm = request.rhs().iter().fold(0.0_f64, |n, v| n.hypot(*v));
        observed.relative_residual = (norm > 0.0).then(|| observed.residual_l2 / norm);
        observed.predicted_divergence = report.provider.accepted.predicted_divergence_max;
        observed.actual_divergence = report.step.step.actual_divergence_max;
        observed.courant = report.step.step.courant;
        let mut divergence = vec![0.0; run.grid.cell_len()];
        let state = native.state();
        PressureOperator::new(&run.grid, run.density)
            .unwrap()
            .divergence([state.x, state.y, state.z], &mut divergence)
            .unwrap();
        observed.net_flux = divergence.iter().sum::<f64>() * run.grid.cell_volume();
        observed.relative_limit = settings.relative_residual;
        let native_reference = NativeReference {
            request_digest: request.digest(),
            sample: PressureSample {
                pressure: provider.output.clone(),
                observation: observed,
            },
            history_status: PressureHistoryStatus::Accepted,
        };
        // This biased model pressure is only an observation. It is never passed
        // to either native owner or used to build the next teacher request.
        let candidate = sample(&run, 8.0 + index as f64, false);
        let a = DiagnosticAttempt {
            attempt_index: index,
            request,
            dispatched: true,
            outcome: PredictionOutcome::Candidate(candidate),
            native_reference: Some(native_reference),
            transition: HistoryTransition::NativeTeacherAdvanced,
        };
        writer.append(&a).unwrap();
        prior = ObservedHistory {
            generation: state.generation,
            interval_end: state.time,
            status: PressureHistoryStatus::Accepted,
            pressure: provider.output.clone(),
        };
    }
    writer
        .finish(terminal(
            TerminalOutcome::Completed,
            2,
            Some(prior.generation),
        ))
        .unwrap();
    drop(writer);
    let reader = archive.reader();
    reader.verify().unwrap();
    assert_eq!(reader.run().native_prefix_steps, 1);
    for index in 0..2 {
        let a = reader.select(index).unwrap();
        assert_eq!(
            a.request.prior().unwrap().status,
            PressureHistoryStatus::Accepted
        );
        assert_eq!(
            a.native_reference.unwrap().history_status,
            PressureHistoryStatus::Accepted
        );
    }
}
#[test]
fn teacher_cannot_advance_native_history_reported_as_unqualified() {
    let archive = Archive::new();
    let run = run(DiagnosticMode::TeacherStage, true);
    let mut a = attempt(&run, 0, run.seed.as_ref(), false);
    a.native_reference.as_mut().unwrap().history_status =
        PressureHistoryStatus::Diagnostic { qualified: false };
    let mut writer =
        DiagnosticRecorder::create(&archive.0, run, RecordingLimits::default()).unwrap();
    assert!(matches!(
        writer.append(&a),
        Err(RecordingError::Invalid(
            "teacher advance requires qualified native reference"
        ))
    ));
    writer
        .finish(terminal(TerminalOutcome::Failed, 1, None))
        .unwrap();
    assert_eq!(archive.reader().count().unwrap(), 0);
}
#[test]
fn independent_first_rejection_is_recorded_without_advancing_or_accepting_another_attempt() {
    let archive = Archive::new();
    let run = run(DiagnosticMode::IndependentStopOnReject, false);
    let a = attempt(&run, 0, None, false);
    let mut writer =
        DiagnosticRecorder::create(&archive.0, run, RecordingLimits::default()).unwrap();
    writer.append(&a).unwrap();
    writer
        .finish(terminal(TerminalOutcome::StoppedOnRejection, 1, None))
        .unwrap();
    drop(writer);
    let reader = archive.reader();
    reader.verify().unwrap();
    assert_eq!(
        reader.select(0).unwrap().transition,
        HistoryTransition::NotAdvanced
    );
    assert_eq!(
        reader.status().unwrap().unwrap().outcome,
        TerminalOutcome::StoppedOnRejection
    );
    let archive = Archive::new();
    let run = run_fixture();
    let mut writer =
        DiagnosticRecorder::create(&archive.0, run.clone(), RecordingLimits::default()).unwrap();
    writer.append(&attempt(&run, 0, None, false)).unwrap();
    assert!(matches!(
        writer.append(&attempt(&run, 1, None, false)),
        Err(RecordingError::Closed)
    ));
    writer
        .finish(terminal(TerminalOutcome::Failed, 1, None))
        .unwrap();
    archive.reader().verify().unwrap();
}
fn run_fixture() -> DiagnosticRun {
    run(DiagnosticMode::IndependentStopOnReject, false)
}
#[test]
fn cancellation_and_invalid_output_preserve_explicit_absence_and_drop_stays_incomplete() {
    for reason in [
        NoCandidateReason::Cancelled,
        NoCandidateReason::Unsupported,
        NoCandidateReason::UnsupportedContext,
        NoCandidateReason::RuntimeFailure,
        NoCandidateReason::InvalidOutput,
    ] {
        let archive = Archive::new();
        let run = run_fixture();
        let mut a = attempt(&run, 0, None, false);
        a.dispatched = !matches!(reason, NoCandidateReason::Unsupported);
        a.outcome = PredictionOutcome::NoCandidate {
            reason,
            detail: "no canonical candidate was produced".into(),
            invalid_payload_sha256: Some([17; 32]),
        };
        let mut writer =
            DiagnosticRecorder::create(&archive.0, run, RecordingLimits::default()).unwrap();
        writer.append(&a).unwrap();
        let status = terminal(
            if reason == NoCandidateReason::Cancelled {
                TerminalOutcome::Cancelled
            } else {
                TerminalOutcome::Failed
            },
            1,
            None,
        );
        writer.finish(status.clone()).unwrap();
        drop(writer);
        let reader = archive.reader();
        reader.verify().unwrap();
        assert_eq!(reader.select(0).unwrap(), a);
        assert_eq!(reader.status().unwrap(), Some(status));
    }
    // The provider can fail before a request is sent (for example, a closed
    // runtime input), and this must not be misreported as dispatched work.
    let archive = Archive::new();
    let run = run_fixture();
    let mut a = attempt(&run, 0, None, false);
    a.dispatched = false;
    a.outcome = PredictionOutcome::NoCandidate {
        reason: NoCandidateReason::RuntimeFailure,
        detail: "provider input unavailable before dispatch".into(),
        invalid_payload_sha256: None,
    };
    let mut writer =
        DiagnosticRecorder::create(&archive.0, run, RecordingLimits::default()).unwrap();
    writer.append(&a).unwrap();
    writer
        .finish(terminal(TerminalOutcome::Failed, 1, None))
        .unwrap();
    drop(writer);
    assert_eq!(archive.reader().select(0).unwrap(), a);
    let archive = Archive::new();
    let run = run_fixture();
    let mut writer =
        DiagnosticRecorder::create(&archive.0, run.clone(), RecordingLimits::default()).unwrap();
    writer.append(&attempt(&run, 0, None, true)).unwrap();
    drop(writer);
    let reader = archive.reader();
    assert_eq!(reader.status().unwrap(), None);
    reader.verify().unwrap();
    assert_eq!(reader.count().unwrap(), 1);
}
#[test]
fn malformed_transitions_domain_changes_and_wrong_equal_request_reference_poison_admission() {
    for mutation in 0..18 {
        let archive = Archive::new();
        let run = run_fixture();
        let mut a = attempt(&run, 0, None, true);
        match mutation {
            0 => a.attempt_index = 1,
            1 => a.transition = HistoryTransition::NativeTeacherAdvanced,
            2 => a.transition = HistoryTransition::NotAdvanced,
            3 => a.dispatched = false,
            4 => {
                if let PredictionOutcome::Candidate(s) = &mut a.outcome {
                    s.pressure[1] = f64::NAN;
                }
            }
            5 => {
                if let PredictionOutcome::Candidate(s) = &mut a.outcome {
                    s.pressure[0] = 1.0;
                }
            }
            6 => {
                if let PredictionOutcome::Candidate(s) = &mut a.outcome {
                    s.pressure.pop();
                }
            }
            7 => {
                let mut other = run.clone();
                other.run_id = "other-track".into();
                a.request = request(&other, None);
            }
            8 => {
                a.native_reference = Some(NativeReference {
                    request_digest: [0; 32],
                    sample: sample(&run, 1.0, true),
                    history_status: PressureHistoryStatus::Diagnostic { qualified: true },
                })
            }
            9 => {
                let mut other = run.clone();
                other.grid =
                    GridGeometry::new([3, 2, 4], run.grid.spacing(), run.grid.origin()).unwrap();
                a.request = request(&other, None);
            }
            10 => {
                let mut other = run.clone();
                other.density = 999.0;
                a.request = request(&other, None);
            }
            11 => {
                let mut other = run.clone();
                other.viscosity = 0.002;
                a.request = request(&other, None);
            }
            12 => {
                let mut other = run.clone();
                other.dt = 0.02;
                a.request = request(&other, None);
            }
            13 => {
                let seeded = self::run(DiagnosticMode::IndependentStopOnReject, true);
                a.request = request(&run, seeded.seed.as_ref());
            }
            14 => {
                if let PredictionOutcome::Candidate(s) = &mut a.outcome {
                    s.observation.hard_stop = true;
                }
            }
            15 => {
                if let PredictionOutcome::Candidate(s) = &mut a.outcome {
                    s.observation.net_flux = f64::INFINITY;
                }
            }
            16 => {
                if let PredictionOutcome::Candidate(s) = &mut a.outcome {
                    s.observation.absolute_limit = -1.0;
                }
            }
            17 => {
                a.dispatched = false;
                a.transition = HistoryTransition::NotAdvanced;
                a.outcome = PredictionOutcome::NoCandidate {
                    reason: NoCandidateReason::InvalidOutput,
                    detail: "bad output without a provider call".into(),
                    invalid_payload_sha256: None,
                };
            }
            _ => unreachable!(),
        }
        let mut writer =
            DiagnosticRecorder::create(&archive.0, run.clone(), RecordingLimits::default())
                .unwrap();
        assert!(writer.append(&a).is_err(), "mutation {mutation}");
        assert!(matches!(
            writer.append(&attempt(&run, 0, None, true)),
            Err(RecordingError::Closed)
        ));
        writer
            .finish(terminal(TerminalOutcome::Failed, 1, None))
            .unwrap();
        assert_eq!(archive.reader().count().unwrap(), 0);
    }
}
#[test]
fn quotas_and_actual_sqlite_full_failure_preserve_auditable_prefix() {
    let run = run_fixture();
    let first = attempt(&run, 0, None, true);
    let prior = advanced(&first, false);
    let second = attempt(&run, 1, Some(&prior), true);
    let archive = Archive::new();
    let limits = RecordingLimits {
        max_records: 1,
        ..Default::default()
    };
    let mut writer = DiagnosticRecorder::create(&archive.0, run.clone(), limits).unwrap();
    writer.append(&first).unwrap();
    assert!(matches!(
        writer.append(&second),
        Err(RecordingError::Limit("diagnostic attempt count"))
    ));
    writer
        .finish(terminal(
            TerminalOutcome::Failed,
            2,
            Some(second.request.generation()),
        ))
        .unwrap();
    drop(writer);
    let reader = archive.reader();
    reader.verify().unwrap();
    assert_eq!(reader.count().unwrap(), 1);
    let small = Archive::new();
    let limits = RecordingLimits {
        max_record_bytes: 1024,
        ..Default::default()
    };
    let mut writer = DiagnosticRecorder::create(&small.0, run.clone(), limits).unwrap();
    assert!(matches!(
        writer.append(&first),
        Err(RecordingError::Limit("diagnostic attempt payload"))
    ));
    drop(writer);
    assert_eq!(small.reader().count().unwrap(), 0);
    // Fill the actual bounded database, not a simulated write-error hook.
    let archive = Archive::new();
    let limits = RecordingLimits {
        max_database_bytes: 65536,
        ..Default::default()
    };
    let mut writer = DiagnosticRecorder::create(&archive.0, run.clone(), limits).unwrap();
    let mut prior = None;
    let mut completed = 0;
    let mut produced = 0;
    for index in 0..100 {
        let a = attempt(&run, index, prior.as_ref(), true);
        produced = index + 1;
        match writer.append(&a) {
            Ok(()) => {
                completed += 1;
                prior = Some(advanced(&a, false));
            }
            Err(RecordingError::Sqlite(_)) => break,
            other => panic!("unexpected {other:?}"),
        }
    }
    assert!(completed > 0 && completed < produced);
    writer
        .finish(terminal(
            TerminalOutcome::Failed,
            produced,
            Some(produced as u64),
        ))
        .unwrap();
    drop(writer);
    let reader = archive.reader();
    reader.verify().unwrap();
    assert_eq!(reader.count().unwrap(), completed);
    let restricted = RecordingLimits {
        max_records: completed - 1,
        ..Default::default()
    };
    assert!(DiagnosticReader::open(&archive.0, restricted).is_err());
}
#[test]
fn metadata_payload_and_terminal_tampering_fail_without_mutating_selected_sources() {
    for mutation in 0..11 {
        let archive = Archive::new();
        let run = run_fixture();
        let a = attempt(&run, 0, None, true);
        let mut writer =
            DiagnosticRecorder::create(&archive.0, run, RecordingLimits::default()).unwrap();
        writer.append(&a).unwrap();
        writer
            .finish(terminal(TerminalOutcome::Completed, 1, Some(1)))
            .unwrap();
        drop(writer);
        let reader = archive.reader();
        let selected = reader.select(0).unwrap();
        let db = Connection::open(&archive.0).unwrap();
        match mutation {
            0 => db
                .execute("UPDATE diagnostic_run SET binding_version='tampered'", [])
                .unwrap(),
            1 => db
                .execute("UPDATE attempts SET rhs=?1", params![vec![0_u8; 24 * 8]])
                .unwrap(),
            2 => db
                .execute(
                    "UPDATE pressure_samples SET request_digest=zeroblob(32)",
                    [],
                )
                .unwrap(),
            3 => db
                .execute("UPDATE pressure_samples SET qualified=0", [])
                .unwrap(),
            4 => db
                .execute("UPDATE diagnostic_lifecycle SET produced_attempts=2", [])
                .unwrap(),
            5 => db
                .execute("UPDATE attempts SET residual=zeroblob(1048576)", [])
                .unwrap(),
            6 => db
                .execute(
                    "UPDATE attempts SET prior_generation=?1",
                    params![1_u64.to_le_bytes().as_slice()],
                )
                .unwrap(),
            7 => {
                db.pragma_update(None, "application_id", 0).unwrap();
                1
            }
            8 => {
                db.pragma_update(None, "user_version", VERSION + 1).unwrap();
                1
            }
            9 => db
                .execute("UPDATE pressure_samples SET policy=99", [])
                .unwrap(),
            10 => db.execute("UPDATE attempts SET transition=99", []).unwrap(),
            _ => unreachable!(),
        };
        drop(db);
        assert!(reader.verify().is_err(), "mutation {mutation}");
        assert_eq!(selected, a);
    }
}
#[test]
fn terminal_attempt_closes_even_on_validation_error_and_covers_durable_tail() {
    for status in [
        terminal(TerminalOutcome::Completed, 0, None),
        terminal(TerminalOutcome::Cancelled, 0, None),
        terminal(TerminalOutcome::Failed, 1, None),
    ] {
        let archive = Archive::new();
        let run = run_fixture();
        let mut writer =
            DiagnosticRecorder::create(&archive.0, run.clone(), RecordingLimits::default())
                .unwrap();
        writer.append(&attempt(&run, 0, None, true)).unwrap();
        assert!(writer.finish(status).is_err());
        assert!(matches!(
            writer.finish(terminal(TerminalOutcome::Failed, 1, Some(1))),
            Err(RecordingError::Closed)
        ));
        drop(writer);
        assert_eq!(archive.reader().status().unwrap(), None);
    }
}
#[test]
fn terminal_sqlite_busy_failure_closes_admission_and_leaves_an_honest_active_prefix() {
    let archive = Archive::new();
    let run = run_fixture();
    let first = attempt(&run, 0, None, true);
    let mut writer =
        DiagnosticRecorder::create(&archive.0, run, RecordingLimits::default()).unwrap();
    writer.append(&first).unwrap();
    let blocker = Connection::open(&archive.0).unwrap();
    blocker.execute_batch("BEGIN IMMEDIATE").unwrap();
    assert!(matches!(
        writer.finish(terminal(TerminalOutcome::Completed, 1, Some(1))),
        Err(RecordingError::Sqlite(_))
    ));
    blocker.execute_batch("ROLLBACK").unwrap();
    assert!(matches!(writer.append(&first), Err(RecordingError::Closed)));
    assert!(matches!(
        writer.finish(terminal(TerminalOutcome::Failed, 1, Some(1))),
        Err(RecordingError::Closed)
    ));
    drop(writer);
    let reader = archive.reader();
    reader.verify().unwrap();
    assert_eq!(reader.count().unwrap(), 1);
    assert_eq!(reader.status().unwrap(), None);
}
#[test]
fn diagnostic_and_accepted_archive_kinds_are_isolated_and_no_overwrite_is_allowed() {
    let diagnostic = Archive::new();
    let run = run_fixture();
    let writer =
        DiagnosticRecorder::create(&diagnostic.0, run.clone(), RecordingLimits::default()).unwrap();
    drop(writer);
    let db = Connection::open(&diagnostic.0).unwrap();
    assert_eq!(
        db.pragma_query_value::<i64, _>(None, "user_version", |r| r.get(0))
            .unwrap(),
        1
    );
    drop(db);
    assert!(matches!(
        RecordingReader::open(&diagnostic.0, RecordingLimits::default()),
        Err(RecordingError::Unsupported("archive kind"))
    ));
    assert!(DiagnosticRecorder::create(&diagnostic.0, run, RecordingLimits::default()).is_err());
    let accepted = Archive::new();
    let mut writer = StepRecorder::create(
        &accepted.0,
        RecordingRun {
            identity: SessionIdentity {
                run_id: 17,
                source_revision: "accepted-fixture".into(),
            },
            solver: "accepted fixture".into(),
            configuration: "fixture".into(),
            counts: [2, 3, 4],
            spacing: [0.25, 0.5, 0.75],
            origin: [-1.0, 2.0, 3.0],
            provenance: "kind isolation only".into(),
        },
        RecordingLimits::default(),
    )
    .unwrap();
    writer
        .finish("failed", "no numerical publications", None)
        .unwrap();
    drop(writer);
    assert!(matches!(
        DiagnosticReader::open(&accepted.0, RecordingLimits::default()),
        Err(RecordingError::Unsupported(
            "diagnostic archive kind/version"
        ))
    ));
    let live = RecordingReader::open(&accepted.0, RecordingLimits::default()).unwrap();
    live.verify().unwrap();
    assert!(matches!(live.select(0), Err(RecordingError::Missing(0))));
    Connection::open(&accepted.0)
        .unwrap()
        .pragma_update(None, "application_id", APPLICATION_ID)
        .unwrap();
    assert!(matches!(
        live.select(0),
        Err(RecordingError::Unsupported("archive kind"))
    ));
    assert!(matches!(
        live.verify(),
        Err(RecordingError::Unsupported("archive kind"))
    ));
    assert!(matches!(
        RecordingReader::open(&accepted.0, RecordingLimits::default()),
        Err(RecordingError::Unsupported("archive kind"))
    ));
}
#[test]
fn request_capture_budget_and_accepted_context_are_rejected_before_material_copy() {
    let run = run_fixture();
    let rhs = vec![0.0; run.grid.cell_len()];
    let make = |mode| {
        PressureContext::new(PressureContextInput {
            grid: &run.grid,
            density: run.density,
            dt: run.dt,
            interval_start: 0.0,
            rhs: &rhs,
            prior: None,
            run_id: &run.run_id,
            generation: 1,
            mode,
        })
        .unwrap()
    };
    assert!(matches!(
        DiagnosticRequest::capture(
            &make(PressureContextMode::DiagnosticCandidate),
            run.viscosity,
            1
        ),
        Err(RecordingError::Limit("request payload"))
    ));
    assert!(matches!(
        DiagnosticRequest::capture(
            &make(PressureContextMode::AcceptedCandidate),
            run.viscosity,
            1 << 20
        ),
        Err(RecordingError::Invalid("diagnostic request domain"))
    ));
}
#[test]
fn native_prefix_declaration_is_bounded_hashed_and_never_inferred_from_seed_generation() {
    for declared in [0, 3, MAX_NATIVE_PREFIX_STEPS] {
        let archive = Archive::new();
        let mut run = run(DiagnosticMode::TeacherStage, true);
        run.native_prefix_steps = declared;
        if declared == MAX_NATIVE_PREFIX_STEPS {
            let seed = run.seed.as_mut().unwrap();
            seed.generation = declared as u64 + 7;
            seed.interval_end = seed.generation as f64 * run.dt;
        }
        let writer =
            DiagnosticRecorder::create(&archive.0, run.clone(), RecordingLimits::default())
                .unwrap();
        drop(writer);
        let reader = archive.reader();
        reader.verify().unwrap();
        assert_eq!(reader.run().native_prefix_steps, declared);
        assert_eq!(reader.run().seed, run.seed);
    }
    for (seed, declared) in [(true, MAX_NATIVE_PREFIX_STEPS + 1), (false, 1), (true, 8)] {
        let archive = Archive::new();
        let mut run = run(DiagnosticMode::TeacherStage, seed);
        run.native_prefix_steps = declared;
        assert!(DiagnosticRecorder::create(&archive.0, run, RecordingLimits::default()).is_err());
        assert!(!archive.0.exists());
    }
    let archive = Archive::new();
    let run = run(DiagnosticMode::TeacherStage, true);
    drop(DiagnosticRecorder::create(&archive.0, run, RecordingLimits::default()).unwrap());
    let reader = archive.reader();
    Connection::open(&archive.0)
        .unwrap()
        .execute("UPDATE diagnostic_run SET native_prefix_steps=3", [])
        .unwrap();
    assert!(reader.verify().is_err());
}
#[test]
fn unqualified_seeds_cannot_resume_and_teacher_seed_requires_observed_accepted_native_history() {
    for (mode, status) in [
        (
            DiagnosticMode::IndependentStopOnReject,
            PressureHistoryStatus::Diagnostic { qualified: false },
        ),
        (
            DiagnosticMode::TeacherStage,
            PressureHistoryStatus::Diagnostic { qualified: false },
        ),
        (
            DiagnosticMode::TeacherStage,
            PressureHistoryStatus::Diagnostic { qualified: true },
        ),
    ] {
        let archive = Archive::new();
        let mut run = run(mode, true);
        run.seed.as_mut().unwrap().status = status;
        assert!(matches!(
            DiagnosticRecorder::create(&archive.0, run, RecordingLimits::default()),
            Err(RecordingError::Invalid(
                "unsupported diagnostic seed history"
            ))
        ));
        assert!(!archive.0.exists());
    }
}
#[test]
fn failed_and_cancelled_terminal_chronology_bounds_declared_unrecorded_advancement() {
    for outcome in [TerminalOutcome::Failed, TerminalOutcome::Cancelled] {
        for (seed, produced, advanced, overflow) in [
            (false, 1, Some(u64::MAX), false),
            (true, 1, Some(1), false),
            (true, 1, Some(7), false),
            (true, 1, Some(9), false),
            (true, 2, Some(u64::MAX), true),
        ] {
            let archive = Archive::new();
            let mut run = run(DiagnosticMode::IndependentStopOnReject, seed);
            if overflow {
                run.seed.as_mut().unwrap().generation = u64::MAX - 1;
            }
            let mut writer =
                DiagnosticRecorder::create(&archive.0, run, RecordingLimits::default()).unwrap();
            assert!(
                writer
                    .finish(terminal(outcome, produced, advanced))
                    .is_err()
            );
            drop(writer);
            let reader = archive.reader();
            assert_eq!(reader.status().unwrap(), None);
            assert_eq!(reader.count().unwrap(), 0);
            reader.verify().unwrap();
        }
        // An explicit unrecorded attempt may have advanced beyond an observed
        // seed, or may have produced no advance. Neither invents intervening data.
        for advanced in [None, Some(8)] {
            let archive = Archive::new();
            let run = run(DiagnosticMode::IndependentStopOnReject, true);
            let mut writer =
                DiagnosticRecorder::create(&archive.0, run, RecordingLimits::default()).unwrap();
            let status = terminal(outcome, 1, advanced);
            writer.finish(status.clone()).unwrap();
            drop(writer);
            let reader = archive.reader();
            reader.verify().unwrap();
            assert_eq!(reader.status().unwrap(), Some(status));
            assert_eq!(reader.count().unwrap(), 0);
        }
    }
    // A known rejection closes the trajectory, so even a declared unrecorded
    // attempt cannot claim that the stopped history advanced afterwards.
    let archive = Archive::new();
    let run = run_fixture();
    let a = attempt(&run, 0, None, false);
    let mut writer =
        DiagnosticRecorder::create(&archive.0, run, RecordingLimits::default()).unwrap();
    writer.append(&a).unwrap();
    assert!(
        writer
            .finish(terminal(TerminalOutcome::Failed, 2, Some(1)))
            .is_err()
    );
}
#[test]
fn signed_zero_domain_changes_fail_before_commit_and_preserve_cold_prefix() {
    for viscosity in [true, false] {
        let archive = Archive::new();
        let mut run = run_fixture();
        run.viscosity = 0.0;
        run.grid = GridGeometry::new([2, 3, 4], run.grid.spacing(), [0.0, 2.0, 3.0]).unwrap();
        let first = attempt(&run, 0, None, true);
        let prior = advanced(&first, false);
        let mut other = run.clone();
        if viscosity {
            other.viscosity = -0.0;
        } else {
            other.grid =
                GridGeometry::new([2, 3, 4], run.grid.spacing(), [-0.0, 2.0, 3.0]).unwrap();
        }
        assert_eq!(other.grid, run.grid);
        assert_eq!(other.viscosity, run.viscosity);
        let changed = attempt(&other, 1, Some(&prior), true);
        assert_ne!(
            changed.request.digest(),
            request(&run, Some(&prior)).digest()
        );
        let mut writer =
            DiagnosticRecorder::create(&archive.0, run, RecordingLimits::default()).unwrap();
        writer.append(&first).unwrap();
        assert!(matches!(
            writer.append(&changed),
            Err(RecordingError::Invalid("request differs from run domain"))
        ));
        assert_eq!(writer.recorded_count(), 1);
        writer
            .finish(terminal(TerminalOutcome::Failed, 2, Some(1)))
            .unwrap();
        drop(writer);
        let reader = archive.reader();
        reader.verify().unwrap();
        assert_eq!(reader.count().unwrap(), 1);
        assert_eq!(reader.select(0).unwrap(), first);
    }
}
