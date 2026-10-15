//! Real owned-worker and stdlib subprocess integration; no trained-model claim.
use super::*;
use crate::ai_comparison::{
    cpu_contract_tests::Fixture, diagnostic_runs::MemberState, engine::ExecutionPolicy,
};
use sha2::{Digest, Sha256};
use std::time::{Duration, Instant};
fn package(body: &str) -> Fixture {
    let prefix = include_str!("../../../../tests/fixtures/pressure_adapter_full.py")
        .split("for line in sys.stdin:")
        .next()
        .unwrap();
    Fixture::source(
        &format!(
            "{prefix}for line in sys.stdin:\n    request=json.loads(line)\n{}",
            body.lines()
                .map(|v| format!("    {v}\n"))
                .collect::<String>()
        ),
        false,
    )
}
fn zero() -> Fixture {
    package(
        "values=request['inputs']['coordinate']['values']\nprint(json.dumps(dict(kind='candidate',id=request['id'],role='full_pressure',pressure=[0.0]*len(values),device='cpu',optimizer_updates=0)),flush=True)",
    )
}
fn scene() -> Scene {
    Scene {
        counts: [5; 3],
        steps: 5,
        warmup_steps: 2,
        ..Default::default()
    }
}
fn wait(worker: &mut super::super::worker::Worker) {
    let end = Instant::now() + Duration::from_secs(12);
    while !worker.finished() {
        assert!(Instant::now() < end, "worker deadline");
        std::thread::sleep(Duration::from_millis(10));
    }
    worker.join_finished();
}
fn cold(worker: super::super::worker::Worker) -> (std::path::PathBuf, DiagnosticRunSet) {
    let path = worker
        .shared
        .lock()
        .unwrap()
        .diagnostic_manifest
        .clone()
        .unwrap();
    drop(worker);
    let set = DiagnosticRunSet::open(&path).unwrap();
    (path, set)
}
fn ready(set: &DiagnosticRunSet, path: &Path) -> rheon::recording::diagnostic::DiagnosticReader {
    match set.open_member(path.parent().unwrap(), 0) {
        MemberState::Ready(r) => r,
        MemberState::Invalid(e) => panic!("{e}"),
        _ => panic!("expected pinned ready archive"),
    }
}
#[test]
fn teacher_records_rejected_candidates_but_completes_native_future_and_cold_metrics() {
    let p = zero();
    let mut w = super::super::worker::Worker::start_with_policy(
        scene(),
        p.manifest.clone(),
        p.root.join("runs"),
        ExecutionPolicy::TeacherStage,
    )
    .unwrap();
    wait(&mut w);
    {
        let view = w.shared.lock().unwrap();
        assert_eq!(view.state, "completed");
        assert_eq!(view.frames.len(), 6);
        assert_eq!(view.statistics.as_ref().unwrap().rejected, 3);
    }
    let (path, set) = cold(w);
    let r = ready(&set, &path);
    assert_eq!(r.count().unwrap(), 3);
    for i in 0..3 {
        let a = r.select(i).unwrap();
        assert_eq!(a.transition, HistoryTransition::NativeTeacherAdvanced);
        let c = a.pressure_comparison().unwrap();
        assert!(c.metrics.l2 > 0.0);
        assert!(!c.gauge.unwrap().diagnostic);
        assert_eq!(c.gauge.unwrap().removed_offset_pa, 0.0);
    }
}
#[test]
fn first_rejection_is_retained_without_frame_or_history_advance() {
    let p = zero();
    let mut w = super::super::worker::Worker::start_with_policy(
        scene(),
        p.manifest.clone(),
        p.root.join("runs"),
        ExecutionPolicy::IndependentStopOnReject,
    )
    .unwrap();
    wait(&mut w);
    {
        let view = w.shared.lock().unwrap();
        assert_eq!(view.state, "stopped_on_rejection");
        assert_eq!(view.frames.len(), 3);
        assert_eq!(view.frames.last().unwrap().step, 2);
    }
    let (path, set) = cold(w);
    let r = ready(&set, &path);
    assert_eq!(r.count().unwrap(), 1);
    assert_eq!(
        r.select(0).unwrap().transition,
        HistoryTransition::NotAdvanced
    );
    assert_eq!(r.status().unwrap().unwrap().last_advanced_generation, None);
    assert!(!path.parent().unwrap().join("frame-0003.json").exists());
}
#[test]
fn cancel_in_native_warmup_closes_actual_zero_attempt_prefix() {
    let p = zero();
    let s = Scene {
        steps: 32,
        warmup_steps: 20,
        ..scene()
    };
    let mut w = super::super::worker::Worker::start_with_policy(
        s,
        p.manifest.clone(),
        p.root.join("runs"),
        ExecutionPolicy::TeacherStage,
    )
    .unwrap();
    let end = Instant::now() + Duration::from_secs(6);
    loop {
        if w.shared.lock().unwrap().frames.len() >= 2 {
            break;
        }
        assert!(Instant::now() < end);
        std::thread::sleep(Duration::from_millis(5));
    }
    w.cancel();
    wait(&mut w);
    let (path, set) = cold(w);
    let r = ready(&set, &path);
    assert_eq!(r.count().unwrap(), 0);
    assert!((1..20).contains(&r.run().native_prefix_steps));
    assert_eq!(
        r.status().unwrap().unwrap().outcome,
        rheon::recording::diagnostic::TerminalOutcome::Cancelled
    );
}
#[test]
fn cancellation_during_real_prediction_retains_dispatched_request_and_drains_child() {
    let p = package(
        "import time,os\n(path.parent/'called').write_text(str(os.getpid()))\ntime.sleep(20)",
    );
    let mut w = super::super::worker::Worker::start_with_policy(
        scene(),
        p.manifest.clone(),
        p.root.join("runs"),
        ExecutionPolicy::IndependentStopOnReject,
    )
    .unwrap();
    let called = p.root.join("called");
    let end = Instant::now() + Duration::from_secs(6);
    while !called.exists() {
        assert!(Instant::now() < end);
        std::thread::sleep(Duration::from_millis(5));
    }
    let pid = std::fs::read_to_string(called).unwrap().parse().unwrap();
    #[cfg(target_os = "linux")]
    super::super::cpu_contract_tests::assert_child_liveness(pid, true);
    w.cancel();
    wait(&mut w);
    let (path, set) = cold(w);
    #[cfg(target_os = "linux")]
    super::super::cpu_contract_tests::assert_child_liveness(pid, false);
    let r = ready(&set, &path);
    let a = r.select(0).unwrap();
    assert!(a.dispatched);
    assert!(matches!(
        a.outcome,
        PredictionOutcome::NoCandidate {
            reason: NoCandidateReason::Cancelled,
            ..
        }
    ));
    assert_eq!(a.transition, HistoryTransition::NotAdvanced);
}
#[test]
fn cold_native_seed_matches_actual_prefix_in_closed_and_incomplete_archives() {
    use rheon::{
        recording::diagnostic::{
            DiagnosticAttempt, DiagnosticReader, DiagnosticRequest, SeedOrigin,
        },
        solver_provider::context::{
            PressureContext, PressureContextInput, PressureContextMode, PressureHistory,
            PressureHistoryStatus,
        },
    };
    let p = zero();
    let identity = ModelProcess::preflight(&p.manifest, &scene()).unwrap();
    let spec = FrozenDiagnosticSpec::capture(
        scene(),
        ExecutionPolicy::TeacherStage,
        &p.manifest,
        &identity,
    )
    .unwrap();
    let mut engine =
        Engine::with_policy(spec.scene.clone(), spec.policy, "seed-template".into()).unwrap();
    for _ in 0..spec.scene.warmup_steps {
        engine
            .advance_typed(&AtomicBool::new(false), |_| {
                panic!("Native prefix must not infer")
            })
            .unwrap();
    }
    let template = engine
        .diagnostic_run(spec.provider.binding(), spec.source_revision.clone())
        .unwrap();
    for mutation in [
        "valid",
        "generation",
        "time",
        "zero-prefix",
        "accepted-origin",
    ] {
        for terminal in [
            None,
            Some(RunTerminal::Failed),
            Some(RunTerminal::Cancelled),
        ] {
            let id = format!("seed-{mutation}-{terminal:?}");
            let root = p.root.join(&id);
            std::fs::create_dir(&root).unwrap();
            let mut header = template.clone();
            header.run_id = id.clone();
            let seed = header.seed.as_mut().unwrap();
            match mutation {
                "generation" => seed.generation = 100,
                "time" => seed.interval_end = f64::from_bits(seed.interval_end.to_bits() + 1),
                "zero-prefix" => header.native_prefix_steps = 0,
                "accepted-origin" => {
                    seed.status = PressureHistoryStatus::Accepted;
                    header.seed_origin = Some(SeedOrigin::AcceptedNative);
                }
                _ => {}
            }
            spec.bind_header(&mut header).unwrap();
            let prior = header.seed.as_ref().unwrap();
            let rhs = vec![0.0; header.grid.cell_len()];
            let context = PressureContext::new(PressureContextInput {
                grid: &header.grid,
                density: header.density,
                dt: header.dt,
                interval_start: prior.interval_end,
                rhs: &rhs,
                prior: Some(PressureHistory {
                    values: &prior.pressure,
                    grid: &header.grid,
                    density: header.density,
                    run_id: &header.run_id,
                    generation: prior.generation,
                    interval_end: prior.interval_end,
                    status: prior.status,
                }),
                run_id: &header.run_id,
                generation: prior.generation + 1,
                mode: PressureContextMode::DiagnosticCandidate,
            })
            .unwrap();
            let attempt = DiagnosticAttempt {
                attempt_index: 0,
                request: DiagnosticRequest::capture(&context, header.viscosity, 1 << 20).unwrap(),
                dispatched: true,
                outcome: PredictionOutcome::NoCandidate {
                    reason: NoCandidateReason::RuntimeFailure,
                    detail: "Manufactured cold custody fixture".into(),
                    invalid_payload_sha256: None,
                },
                native_reference: None,
                transition: HistoryTransition::NotAdvanced,
            };
            let path = root.join("attempts.sqlite");
            let mut writer =
                DiagnosticRecorder::create(&path, header, spec.recording.limits()).unwrap();
            writer.append(&attempt).unwrap();
            if let Some(terminal) = terminal {
                writer
                    .finish(DiagnosticTerminal {
                        outcome: terminal.recording(),
                        detail: "Manufactured cold custody fixture".into(),
                        produced_attempts: 1,
                        last_advanced_generation: None,
                    })
                    .unwrap();
            }
            drop(writer);
            // These are valid generic archives, including a genuinely appended
            // request with no terminal. Only the application's prefix is false.
            let reader = DiagnosticReader::open(&path, spec.recording.limits()).unwrap();
            reader.verify().unwrap();
            assert_eq!(reader.count().unwrap(), 1);
            assert_eq!(reader.status().unwrap().is_some(), terminal.is_some());
            drop(reader);
            let set = DiagnosticRunSet::single(id, spec.clone(), None);
            if mutation == "valid" {
                assert!(matches!(
                    (terminal, set.open_member(&root, 0)),
                    (None, MemberState::Incomplete(_)) | (Some(_), MemberState::Unpinned(_))
                ));
            } else {
                assert!(matches!(
                    set.open_member(&root, 0),
                    MemberState::Invalid(error)
                        if error == "Diagnostic native seed differs from captured prefix"
                ));
            }
        }
    }
}

#[test]
fn cold_terminal_attempts_respect_actual_prefix_and_requested_horizon() {
    let p = zero();
    let identity = ModelProcess::preflight(&p.manifest, &scene()).unwrap();
    let spec = FrozenDiagnosticSpec::capture(
        scene(),
        ExecutionPolicy::TeacherStage,
        &p.manifest,
        &identity,
    )
    .unwrap();
    let horizon = spec.scene.steps - spec.scene.warmup_steps;
    for (case, full_prefix, produced, terminal, expected_error) in [
        (
            "before-prefix",
            false,
            1,
            RunTerminal::Failed,
            Some("Diagnostic attempts require completed native prefix"),
        ),
        (
            "beyond-horizon",
            true,
            horizon + 1,
            RunTerminal::Failed,
            Some("Diagnostic produced request horizon exceeds intent"),
        ),
        (
            "cancelled-beyond-horizon",
            true,
            horizon + 1,
            RunTerminal::Cancelled,
            Some("Diagnostic produced request horizon exceeds intent"),
        ),
        ("within-horizon-tail", true, 1, RunTerminal::Failed, None),
    ] {
        let root = p.root.join(case);
        std::fs::create_dir(&root).unwrap();
        let mut engine = Engine::with_policy(spec.scene.clone(), spec.policy, case.into()).unwrap();
        if full_prefix {
            let flag = AtomicBool::new(false);
            for _ in 0..spec.scene.warmup_steps {
                engine
                    .advance_typed(&flag, |_| panic!("Native prefix must not infer"))
                    .unwrap();
            }
        }
        let mut header = engine
            .diagnostic_run(spec.provider.binding(), spec.source_revision.clone())
            .unwrap();
        spec.bind_header(&mut header).unwrap();
        let prefix = header.native_prefix_steps;
        let mut writer = DiagnosticRecorder::create(
            &root.join("attempts.sqlite"),
            header,
            spec.recording.limits(),
        )
        .unwrap();
        // Deliberately constructed unrecorded observations exercise the saved
        // intent boundary beyond the generic archive's chronology guarantees.
        writer
            .finish(DiagnosticTerminal {
                outcome: terminal.recording(),
                detail: "Explicit unrecorded observation".into(),
                produced_attempts: produced,
                last_advanced_generation: None,
            })
            .unwrap();
        drop(writer);
        let mut set = DiagnosticRunSet::single(case.into(), spec.clone(), None);
        let observation = RunObservation {
            terminal,
            detail: "Explicit unrecorded observation".into(),
            statistics: RunStatistics {
                native_prefix_steps: prefix,
                produced_attempts: produced,
                unavailable: produced,
                ..RunStatistics::default()
            },
        };
        set.members[0].observed = Some(observation.clone());
        if let Some(expected) = expected_error {
            assert!(matches!(
                set.open_member(&root, 0),
                MemberState::Invalid(error) if error == expected
            ));
            assert_eq!(
                set.pin_closed(&root, 0, observation),
                Err("Diagnostic pin failed cold verification".into())
            );
            assert!(set.members[0].finalized.is_none());
        } else {
            set.pin_closed(&root, 0, observation).unwrap();
            assert!(matches!(set.open_member(&root, 0), MemberState::Ready(_)));
        }
    }
}

#[test]
fn append_budget_failure_retains_reconciled_failed_prefix_and_explicit_unrecorded_tail() {
    let p = zero();
    let identity = ModelProcess::preflight(&p.manifest, &scene()).unwrap();
    let mut spec = FrozenDiagnosticSpec::capture(
        scene(),
        ExecutionPolicy::TeacherStage,
        &p.manifest,
        &identity,
    )
    .unwrap();
    spec.recording.max_records = 1;
    let mut w =
        super::super::worker::Worker::start_saved(spec, [9; 32], p.root.join("runs")).unwrap();
    wait(&mut w);
    {
        let v = w.shared.lock().unwrap();
        assert_eq!(v.state, "failed");
        let s = v.statistics.as_ref().unwrap();
        assert_eq!(s.produced_attempts, 2);
        assert_eq!(s.recorded_attempts, 1);
    }
    let (path, set) = cold(w);
    let r = ready(&set, &path);
    assert_eq!(r.count().unwrap(), 1);
    let terminal = r.status().unwrap().unwrap();
    assert_eq!(
        terminal.outcome,
        rheon::recording::diagnostic::TerminalOutcome::Failed
    );
    assert_eq!(terminal.produced_attempts, 2);
    assert_eq!(
        set.members[0]
            .observed
            .as_ref()
            .unwrap()
            .statistics
            .recorded_attempts,
        1
    );
}
#[test]
fn saved_runtime_failures_preserve_frozen_intent_parent_and_no_child_execution() {
    for failure in ["changed", "missing", "unsupported_scene"] {
        let source = include_str!("../../../../tests/fixtures/pressure_adapter_full.py").replace(
            "path = Path(sys.argv[-1])",
            "path = Path(sys.argv[-1])\n(path.parent/'spawn-marker').write_text('child executed')",
        );
        let p = Fixture::source(&source, false);
        if failure == "unsupported_scene" {
            let mut manifest: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&p.manifest).unwrap()).unwrap();
            manifest["support"]["counts_max"] = serde_json::json!([5, 5, 5]);
            std::fs::write(&p.manifest, serde_json::to_vec(&manifest).unwrap()).unwrap();
        }
        let identity = ModelProcess::preflight(&p.manifest, &scene()).unwrap();
        let mut spec = FrozenDiagnosticSpec::capture(
            scene(),
            ExecutionPolicy::TeacherStage,
            &p.manifest,
            &identity,
        )
        .unwrap();
        if failure == "unsupported_scene" {
            spec.scene.counts = [6; 3];
        }
        let frozen_digest = spec.digest().unwrap();
        let parent = [7; 32];
        if failure == "missing" {
            std::fs::remove_file(&p.manifest).unwrap();
        } else if failure == "changed" {
            let mut manifest: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&p.manifest).unwrap()).unwrap();
            manifest["version"] = serde_json::json!("replacement-after-freeze");
            std::fs::write(&p.manifest, serde_json::to_vec(&manifest).unwrap()).unwrap();
        }
        let mut worker =
            super::super::worker::Worker::start_saved(spec, parent, p.root.join("failed-rerun"))
                .unwrap();
        wait(&mut worker);
        let directory = {
            let view = worker.shared.lock().unwrap();
            assert_eq!(view.state, "failed");
            if failure == "changed" {
                assert_eq!(
                    view.message,
                    "Saved diagnostic provider pin changed; no model code executed"
                );
            } else if failure == "unsupported_scene" {
                assert_eq!(
                    view.message,
                    "CPU adapter unsupported: physical request is outside declared support"
                );
            }
            assert!(view.provider.is_none());
            assert!(view.frames.is_empty());
            view.directory.clone().unwrap()
        };
        drop(worker);
        assert!(!p.root.join("spawn-marker").exists());
        assert!(!directory.join("attempts.sqlite").exists());
        let intent = DiagnosticRunSet::open(&directory.join("intent.json")).unwrap();
        assert_eq!(intent.parent_sha256, Some(parent));
        assert_eq!(intent.members[0].spec.digest().unwrap(), frozen_digest);
        assert!(matches!(
            intent.open_member(&directory, 0),
            MemberState::NotRun
        ));
        let finished = DiagnosticRunSet::open(&directory.join("finished.json")).unwrap();
        assert_eq!(finished.parent_sha256, Some(parent));
        assert_eq!(finished.members[0].spec.digest().unwrap(), frozen_digest);
        assert!(finished.members[0].finalized.is_none());
        let observed = finished.members[0].observed.as_ref().unwrap();
        assert_eq!(observed.terminal, RunTerminal::Failed);
        assert_eq!(observed.statistics.produced_attempts, 0);
        assert_eq!(observed.statistics.recorded_attempts, 0);
        assert_eq!(observed.statistics.last_advanced_generation, None);
        assert!(matches!(
            finished.open_member(&directory, 0),
            MemberState::Missing
        ));
    }
}

#[test]
fn fresh_rerun_preserves_source_and_rechecks_counts_provider_and_frozen_spec() {
    let p = zero();
    let mut w = super::super::worker::Worker::start_with_policy(
        scene(),
        p.manifest.clone(),
        p.root.join("runs"),
        ExecutionPolicy::TeacherStage,
    )
    .unwrap();
    wait(&mut w);
    let (path, set) = cold(w);
    let root = path.parent().unwrap();
    let original: Vec<_> = std::fs::read_dir(root)
        .unwrap()
        .map(|v| {
            let path = v.unwrap().path();
            let digest: [u8; 32] = Sha256::digest(std::fs::read(&path).unwrap()).into();
            (path, digest)
        })
        .collect();
    let mut rerun = super::super::worker::Worker::start_saved(
        set.members[0].spec.clone(),
        set.digest().unwrap(),
        p.root.join("runs"),
    )
    .unwrap();
    wait(&mut rerun);
    let (new_path, new_set) = cold(rerun);
    drop(ready(&new_set, &new_path));
    assert_ne!(new_set.members[0].run_id, set.members[0].run_id);
    assert_eq!(new_set.parent_sha256, Some(set.digest().unwrap()));
    assert_ne!(new_path, path);
    for (path, digest) in original {
        assert_eq!(
            <[u8; 32]>::from(Sha256::digest(std::fs::read(path).unwrap())),
            digest
        );
    }
    let mut changed = set.clone();
    changed.members[0]
        .observed
        .as_mut()
        .unwrap()
        .statistics
        .qualified = 1;
    changed.members[0]
        .observed
        .as_mut()
        .unwrap()
        .statistics
        .rejected -= 1;
    let bad = root.join("rehash-counts.json");
    changed.save_new(&bad).unwrap();
    let changed = DiagnosticRunSet::open(&bad).unwrap();
    assert!(matches!(
        changed.open_member(root, 0),
        MemberState::Invalid(_)
    ));
    let mut changed = set.clone();
    changed.members[0].spec.scene.seed += 1;
    assert!(matches!(
        changed.open_member(root, 0),
        MemberState::Invalid(_)
    ));
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&p.manifest).unwrap()).unwrap();
    manifest["version"] = serde_json::json!("changed");
    std::fs::write(&p.manifest, serde_json::to_vec(&manifest).unwrap()).unwrap();
    let mut refused = super::super::worker::Worker::start_saved(
        set.members[0].spec.clone(),
        set.digest().unwrap(),
        p.root.join("runs"),
    )
    .unwrap();
    wait(&mut refused);
    let view = refused.shared.lock().unwrap();
    assert_eq!(view.state, "failed");
    assert!(view.provider.is_none());
    assert!(view.frames.is_empty());
    // Cold inspection is independent of the runtime package even after it changes.
    drop(ready(&set, &path));
}

#[test]
fn typed_headless_failure_returns_error_after_preserving_terminal_observation() {
    let p = zero();
    let output = p.root.join("headless");
    let result = crate::ai_comparison::headless_with_policy(
        p.root.join("absent-manifest.json"),
        output.clone(),
        16,
        ExecutionPolicy::TeacherStage,
    );
    assert!(result.is_err());
    let directory = std::fs::read_dir(output)
        .unwrap()
        .map(|v| v.unwrap().path())
        .find(|p| p.is_dir())
        .unwrap();
    let completion: serde_json::Value =
        serde_json::from_slice(&std::fs::read(directory.join("completion.json")).unwrap()).unwrap();
    assert_eq!(completion["state"], "failed");
}

#[test]
fn changed_executor_stamp_blocks_saved_execution_but_historical_cold_source_stays_readable() {
    use crate::ai_comparison::engine::Engine;
    use rheon::recording::diagnostic::DiagnosticTerminal;
    let source = include_str!("../../../../tests/fixtures/pressure_adapter_full.py").replace(
        "path = Path(sys.argv[-1])",
        "path = Path(sys.argv[-1])\n(path.parent/'spawn-marker').write_text('child executed')",
    );
    let p = Fixture::source(&source, false);
    let identity = ModelProcess::preflight(&p.manifest, &scene()).unwrap();
    let matching = FrozenDiagnosticSpec::capture(
        scene(),
        ExecutionPolicy::TeacherStage,
        &p.manifest,
        &identity,
    )
    .unwrap();
    let mut historical = matching.clone();
    historical.source_revision = "rheon-diagnostic-engine/old; older-known-build".into();
    historical.validate().unwrap();
    assert!(historical.validate_execution().is_err());
    assert!(
        super::super::worker::Worker::start_saved(
            historical.clone(),
            [1; 32],
            p.root.join("refused")
        )
        .is_err()
    );
    assert!(!p.root.join("spawn-marker").exists());
    assert!(!p.root.join("refused").exists());
    // A manufactured historical metadata fixture verifies that inspection does
    // not require source/runtime availability or reinterpret historical stamps.
    let directory = p.root.join("historical");
    std::fs::create_dir(&directory).unwrap();
    let engine = Engine::with_policy(
        scene(),
        ExecutionPolicy::TeacherStage,
        "historical-fixture".into(),
    )
    .unwrap();
    let mut header = engine
        .diagnostic_run(
            historical.provider.binding(),
            historical.source_revision.clone(),
        )
        .unwrap();
    historical.bind_header(&mut header).unwrap();
    let mut recorder = DiagnosticRecorder::create(
        &directory.join("attempts.sqlite"),
        header,
        historical.recording.limits(),
    )
    .unwrap();
    recorder
        .finish(DiagnosticTerminal {
            outcome: RunTerminal::Cancelled.recording(),
            detail: "Manufactured historical source fixture".into(),
            produced_attempts: 0,
            last_advanced_generation: None,
        })
        .unwrap();
    drop(recorder);
    let mut set = DiagnosticRunSet::single("historical-fixture".into(), historical, None);
    set.pin_closed(
        &directory,
        0,
        RunObservation {
            terminal: RunTerminal::Cancelled,
            detail: "Manufactured historical source fixture".into(),
            statistics: RunStatistics::default(),
        },
    )
    .unwrap();
    let path = directory.join("finished.json");
    set.save_new(&path).unwrap();
    let loaded = DiagnosticRunSet::open(&path).unwrap();
    drop(ready(&loaded, &path));
    assert!(!p.root.join("spawn-marker").exists());
    let mut worker =
        super::super::worker::Worker::start_saved(matching, [2; 32], p.root.join("matching"))
            .unwrap();
    wait(&mut worker);
    assert!(p.root.join("spawn-marker").exists());
    assert_eq!(worker.shared.lock().unwrap().state, "completed");
}
