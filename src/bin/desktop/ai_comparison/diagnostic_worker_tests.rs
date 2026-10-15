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
