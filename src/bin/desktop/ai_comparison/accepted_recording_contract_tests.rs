//! Accepted Session/process failure archives using the existing CPU driver.
//! Reuses the existing CPU child owner and authored two-cell equation fixture.
//! No trained model, scientific accuracy, resume/checkpoint or GUI claim.
use super::*;
use rheon::recording::{
    RecordingLimits, RecordingReader, StepRecorder,
    comparison::Selection,
    provider::{
        RecordedConfiguredSession, RecordedPressureSelection, RecordedProviderRejection,
        configured_publication, configured_run, verify_configured_recording,
    },
};
use std::{path::Path, process::Command, sync::atomic::Ordering};

const COLD_PATH: &str = "RHEON_ACCEPTED_RECORDING_COLD_PATH";
const COLD_OUTCOME: &str = "RHEON_ACCEPTED_RECORDING_COLD_OUTCOME";
const COLD_GENERATION: &str = "RHEON_ACCEPTED_RECORDING_COLD_GENERATION";
const COLD_LAST_KIND: &str = "RHEON_ACCEPTED_RECORDING_COLD_LAST_KIND";
const COLD_SHA: &str = "RHEON_ACCEPTED_RECORDING_COLD_SHA";
const COLD_SELECTIONS: &str = "RHEON_ACCEPTED_RECORDING_COLD_SELECTIONS";

fn file_sha(path: &Path) -> String {
    format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()))
}
fn selection_sha(reader: &RecordingReader, ordinal: usize) -> String {
    Selection::read(reader, ordinal)
        .unwrap()
        .digest()
        .iter()
        .map(|value| format!("{value:02x}"))
        .collect()
}
fn pressure_report_bits(report: PressureReport) -> [u64; 4] {
    [
        report.iterations as u64,
        report.true_residual_l2.to_bits(),
        report.true_residual_max.to_bits(),
        report.predicted_divergence_max.to_bits(),
    ]
}
fn append_actual(recorder: &mut StepRecorder, session: &SimulationSession) {
    let snapshot = session.snapshot();
    let row = configured_publication(
        &snapshot,
        recorder.recorded_count(),
        recorder.limits().max_record_bytes,
    )
    .unwrap();
    let decoded = RecordedConfiguredSession::decode(recorder.run(), &row).unwrap();
    let actual = snapshot.state();
    assert_eq!(decoded.grid, *snapshot.grid());
    assert_eq!(decoded.generation, actual.carrier.generation);
    assert_eq!(decoded.time.to_bits(), actual.carrier.time.to_bits());
    assert_eq!(decoded.carrier_stamp, actual.carrier_stamp);
    assert_eq!(decoded.fraction_stamp, actual.liquid.stamp);
    assert_eq!(
        decoded.fraction_time.to_bits(),
        actual.liquid.time.to_bits()
    );
    for (saved, native) in [
        (&decoded.x, actual.carrier.x),
        (&decoded.y, actual.carrier.y),
        (&decoded.z, actual.carrier.z),
        (&decoded.tracer, actual.carrier.tracer),
    ] {
        assert_eq!(
            saved.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            native.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
        );
    }
    assert_eq!(
        decoded
            .fraction
            .iter()
            .map(|v| v.to_bits())
            .collect::<Vec<_>>(),
        actual
            .liquid
            .fraction
            .iter()
            .map(|v| v.to_bits())
            .collect::<Vec<_>>()
    );
    if let Some(start) = snapshot.accepted_interval_start() {
        let (saved_start, saved_pressure) = decoded.interval_pressure.as_ref().unwrap();
        assert_eq!(saved_start.to_bits(), start.to_bits());
        assert_eq!(
            saved_pressure
                .iter()
                .map(|v| v.to_bits())
                .collect::<Vec<_>>(),
            actual
                .pressure
                .iter()
                .map(|v| v.to_bits())
                .collect::<Vec<_>>()
        );
        let saved = decoded.receipt.as_ref().unwrap();
        let receipt = snapshot.configured_provider_report().unwrap();
        assert_eq!(saved.binding, receipt.binding.as_deref().cloned());
        assert_eq!(saved.context, receipt.context);
        assert_eq!(saved.prediction_called, receipt.prediction_called);
        assert_eq!(
            saved.viscosity_applied,
            snapshot.report().unwrap().viscosity.is_some()
        );
        assert_eq!(
            saved.dynamic_viscosity.to_bits(),
            receipt.dynamic_viscosity.to_bits()
        );
        assert_eq!(
            pressure_report_bits(saved.accepted),
            pressure_report_bits(receipt.pressure.accepted)
        );
        assert_eq!(
            saved.candidate.map(pressure_report_bits),
            receipt.pressure.candidate.map(pressure_report_bits)
        );
        assert_eq!(
            (
                saved.total,
                saved.inference,
                saved.qualification,
                saved.fallback
            ),
            (
                receipt.pressure.total,
                receipt.pressure.inference,
                receipt.pressure.qualification,
                receipt.pressure.fallback
            )
        );
    } else {
        assert!(decoded.interval_pressure.is_none());
        assert!(decoded.receipt.is_none());
    }
    recorder.append(&row).unwrap();
}
fn new_archive(path: &Path, session: &SimulationSession) -> StepRecorder {
    let run = configured_run(
        &session.snapshot(),
        "actual test-only stdlib CPU process; no trained model, measurements or restart",
    )
    .unwrap();
    let limits = RecordingLimits {
        max_database_bytes: 8 * 1024 * 1024,
        max_record_bytes: 1024 * 1024,
        max_records: 4,
    };
    let mut recorder = StepRecorder::create(path, run, limits).unwrap();
    append_actual(&mut recorder, session);
    recorder
}
fn seed(
    session: &mut SimulationSession,
    w: &mut ProviderStepWorkspace,
    v: &mut ViscosityWorkspace,
    driver: &mut Driver,
    recorder: &mut StepRecorder,
) {
    let (_, report) = session
        .step_with_configured_pressure(inputs(), w, Some((v, MU)), Some(&mut *driver), None, |_| {
            false
        })
        .unwrap();
    assert_eq!(session.snapshot().frame(), 1);
    assert!(!report.prediction_called);
    assert!(matches!(
        report.pressure.rejection,
        Some(ProviderRejection::Unsupported(_))
    ));
    assert!(driver.predictions.is_empty());
    assert!(driver.failures.is_empty());
    append_actual(recorder, session);
}

fn observed_runtime_files(fixture: &Fixture) -> [Option<Vec<u8>>; 2] {
    ["requests.jsonl", "first-request-observed"].map(|name| {
        match std::fs::read(fixture.root.join(name)) {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => panic!("observe owned runtime fixture {name}: {error}"),
        }
    })
}

// Cold inspection is a separate executable process, not a branch that happens
// to reopen after in-process ownership was dropped. Expected fingerprints are
// passed as bounded process arguments/environment, never as a second store.
fn cold_reopen(fixture: &Fixture, path: &Path, outcome: &str, generation: u64, last_kind: &str) {
    let reader = RecordingReader::open(path, RecordingLimits::default()).unwrap();
    reader.verify().unwrap();
    let identities = (0..reader.count().unwrap())
        .map(|ordinal| selection_sha(&reader, ordinal))
        .collect::<Vec<_>>()
        .join(",");
    drop(reader);
    let module = module_path!().split_once("::").unwrap().1;
    let test_name = format!("{module}::cold_configured_archive_child");
    let observed_before = observed_runtime_files(fixture);
    let manifest_sha = file_sha(&fixture.manifest);
    let hidden_manifest = fixture.manifest.with_extension("cold-hidden");
    assert!(!hidden_manifest.exists());
    // Only this test's generated deployment manifest moves. All runtime owners
    // have already drained, and the cold child must need only the SQLite file.
    std::fs::rename(&fixture.manifest, &hidden_manifest).unwrap();
    assert!(!fixture.manifest.exists());
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", &test_name, "--nocapture", "--test-threads=1"])
        .env(COLD_PATH, path)
        .env(COLD_OUTCOME, outcome)
        .env(COLD_GENERATION, generation.to_string())
        .env(COLD_LAST_KIND, last_kind)
        .env(COLD_SHA, file_sha(path))
        .env(COLD_SELECTIONS, identities)
        .output();
    // Restore even when spawning/inspection fails, before reporting the error.
    std::fs::rename(&hidden_manifest, &fixture.manifest).unwrap();
    assert_eq!(file_sha(&fixture.manifest), manifest_sha);
    assert_eq!(observed_runtime_files(fixture), observed_before);
    let output = output.unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("cold configured archive verified"));
}

#[test]
fn cold_configured_archive_child() {
    let Some(path) = std::env::var_os(COLD_PATH) else {
        return;
    };
    // This branch constructs no Session, Driver, package, process or runtime.
    let path = std::path::PathBuf::from(path);
    let expected_sha = std::env::var(COLD_SHA).unwrap();
    assert_eq!(file_sha(&path), expected_sha);
    let generation: u64 = std::env::var(COLD_GENERATION).unwrap().parse().unwrap();
    let outcome = std::env::var(COLD_OUTCOME).unwrap();
    let kind = std::env::var(COLD_LAST_KIND).unwrap();
    let expected = std::env::var(COLD_SELECTIONS).unwrap();
    let identities = expected.split(',').collect::<Vec<_>>();
    assert!(identities.len() <= 4);
    let reader = RecordingReader::open(&path, RecordingLimits::default()).unwrap();
    reader.verify().unwrap();
    let coverage = verify_configured_recording(&reader).unwrap();
    assert_eq!(reader.status().unwrap().outcome, outcome);
    assert_eq!(
        reader.status().unwrap().accepted_generation,
        Some(generation)
    );
    assert_eq!(reader.count().unwrap(), generation as usize + 1);
    assert_eq!(reader.count().unwrap(), identities.len());
    assert_eq!(
        coverage.verified_prior_links,
        generation.saturating_sub(1) as usize
    );
    assert_eq!(coverage.unavailable_prior_links, 0);
    for (ordinal, expected) in identities.iter().enumerate() {
        assert_eq!(&selection_sha(&reader, ordinal), expected);
        let row = reader.select(ordinal).unwrap();
        let decoded = RecordedConfiguredSession::decode(reader.run(), &row).unwrap();
        assert_eq!(decoded.generation, ordinal as u64);
        if ordinal == 0 {
            assert!(decoded.receipt.is_none());
            assert!(decoded.interval_pressure.is_none());
        }
    }
    let row = reader.select(generation as usize).unwrap();
    let decoded = RecordedConfiguredSession::decode(reader.run(), &row).unwrap();
    let receipt = decoded.receipt.unwrap();
    assert!(receipt.viscosity_applied);
    assert_eq!(
        decoded.interval_pressure.unwrap().0.to_bits(),
        ((generation - 1) as f64 * DT).to_bits()
    );
    assert_eq!(decoded.time.to_bits(), (generation as f64 * DT).to_bits());
    assert!(receipt.accepted.true_residual_l2.is_finite());
    assert!(receipt.binding.is_some());
    match kind.as_str() {
        "execution" => {
            assert!(receipt.prediction_called);
            assert!(matches!(
                receipt.selected,
                RecordedPressureSelection::Native(PressureImplementation::JacobiPcgV1)
            ));
            assert!(matches!(
                receipt.rejection,
                Some(RecordedProviderRejection::Execution(_))
            ));
            assert!(receipt.candidate.is_none());
        }
        "quality" => {
            assert!(receipt.prediction_called);
            assert!(matches!(
                receipt.selected,
                RecordedPressureSelection::Native(PressureImplementation::JacobiPcgV1)
            ));
            assert!(matches!(
                receipt.rejection,
                Some(RecordedProviderRejection::QualityGate)
            ));
            assert!(receipt.candidate.is_some());
        }
        "seed" => {
            assert!(!receipt.prediction_called);
            assert!(matches!(
                receipt.selected,
                RecordedPressureSelection::Native(PressureImplementation::JacobiPcgV1)
            ));
            assert!(matches!(
                receipt.rejection,
                Some(RecordedProviderRejection::Unsupported(_))
            ));
        }
        "configured" => {
            assert!(receipt.prediction_called);
            assert!(matches!(
                receipt.selected,
                RecordedPressureSelection::Configured
            ));
            assert!(receipt.rejection.is_none());
        }
        other => panic!("unknown expected cold receipt {other}"),
    }
    // RCP2 has no detailed runtime NoCandidateReason/dispatched/digest fields.
    // These remain explicitly unavailable, not inferred from mapped rejection.
    assert_eq!(file_sha(&path), expected_sha);
    println!("cold configured archive verified");
}

// A retry is a new in-memory Session and a NEW complete archive. It never
// resumes an archive or imports its pressure as a constructor/accepted prior.
// The fixture helper deliberately reuses caller-assigned run identity771;
// distinct filenames do not attest independent/authenticated run provenance.
fn fresh_retry(
    fixture: &Fixture,
    original: &Path,
    original_sha: &str,
    expected_attempt: &Observation,
) {
    let retry_path = fixture.root.join("fresh-retry.sqlite");
    let mut driver = Driver::new(fixture);
    let (mut session, (mut w, mut v)) = (session(2), workspaces());
    let mut recorder = new_archive(&retry_path, &session);
    seed(&mut session, &mut w, &mut v, &mut driver, &mut recorder);
    let (_, report) = session
        .step_with_configured_pressure(
            inputs(),
            &mut w,
            Some((&mut v, MU)),
            Some(&mut driver),
            None,
            |_| false,
        )
        .unwrap();
    assert_eq!(&driver.predictions[0], expected_attempt);
    assert!(report.pressure.rejection.is_none());
    assert_eq!(report.pressure.selected, "configured-pressure-context-v1");
    check_context(&report, &driver.predictions[0]);
    append_actual(&mut recorder, &session);
    recorder
        .finish(
            "completed",
            "fresh in-memory retry; no checkpoint/resume",
            Some(2),
        )
        .unwrap();
    drop(recorder);
    drop(session);
    driver.drain();
    assert_eq!(file_sha(original), original_sha);
    cold_reopen(fixture, &retry_path, "completed", 2, "configured");
    assert_eq!(file_sha(original), original_sha);
}

#[test]
fn actual_cpu_failures_and_quality_rejections_record_only_accepted_native_fallbacks() {
    for mode in [
        "exit_once",
        "bad_once",
        "runtime_timeout",
        "malformed_reply",
    ] {
        let fixture = package(
            mode == "bad_once",
            match mode {
                "runtime_timeout" => "stall_once",
                "malformed_reply" => "solve",
                other => other,
            },
        );
        if mode == "malformed_reply" {
            let script_path = fixture.root.join("adapter.py");
            let original = std::fs::read_to_string(&script_path).unwrap();
            let needle = "    full = [0.0, b[1] * rho / 0.25]";
            assert_eq!(original.matches(needle).count(), 1);
            let authored = original.replacen(
                needle,
                &format!("    if first:\n        print(\"not-json\", flush=True)\n        continue\n{needle}"),
                1,
            );
            // Authored and re-pinned BEFORE loading the existing Driver; this
            // is not a deployment-custody tamper test or a new runtime owner.
            std::fs::write(&script_path, &authored).unwrap();
            fixture.edit(|manifest| {
                manifest["files"]["adapter.py"]["sha256"] =
                    json!(format!("{:x}", Sha256::digest(authored.as_bytes())));
                manifest["files"]["adapter.py"]["bytes"] = json!(authored.len());
            });
        }
        let mut driver = Driver::new(&fixture);
        if mode == "runtime_timeout" {
            driver.process.prediction_timeout = Duration::from_secs(1);
        }
        let (mut session, (mut w, mut v)) = (session(2), workspaces());
        let path = fixture.root.join("accepted-fallback.sqlite");
        let mut recorder = new_archive(&path, &session);
        seed(&mut session, &mut w, &mut v, &mut driver, &mut recorder);
        assert!(received(&fixture).is_empty());
        let (_, report) = session
            .step_with_configured_pressure(
                inputs(),
                &mut w,
                Some((&mut v, MU)),
                Some(&mut driver),
                None,
                |_| false,
            )
            .unwrap();
        assert_eq!(session.snapshot().frame(), 2);
        assert!(report.prediction_called);
        assert_eq!(
            report.pressure.selected,
            PressureImplementation::JacobiPcgV1.id()
        );
        assert_eq!(received(&fixture).len(), 1);
        assert!(fixture.root.join("first-request-observed").exists());
        let attempt = driver.predictions[0].clone();
        check_context(&report, &attempt);
        if mode == "bad_once" {
            assert_eq!(
                report.pressure.rejection,
                Some(ProviderRejection::QualityGate)
            );
            assert!(report.pressure.candidate.is_some());
            assert!(driver.failures.is_empty());
        } else {
            assert!(matches!(
                report.pressure.rejection,
                Some(ProviderRejection::Execution(_))
            ));
            assert!(report.pressure.candidate.is_none());
            let failure = &driver.failures[0];
            assert!(failure.dispatched);
            if mode == "malformed_reply" {
                assert_eq!(failure.reason, NoCandidateReason::InvalidOutput);
                let expected_digest: [u8; 32] = Sha256::digest(b"not-json\n").into();
                assert_eq!(failure.invalid_payload_sha256, Some(expected_digest));
            } else {
                assert_eq!(failure.reason, NoCandidateReason::RuntimeFailure);
                assert!(failure.invalid_payload_sha256.is_none());
            }
        }
        append_actual(&mut recorder, &session);
        recorder
            .finish(
                "completed",
                "CPU attempt failed or rejected; native full interval accepted",
                Some(2),
            )
            .unwrap();
        drop(recorder);
        drop(session);
        driver.drain();
        let original_sha = file_sha(&path);
        cold_reopen(
            &fixture,
            &path,
            "completed",
            2,
            if mode == "bad_once" {
                "quality"
            } else {
                "execution"
            },
        );
        fresh_retry(&fixture, &path, &original_sha, &attempt);
        assert_eq!(file_sha(&path), original_sha);
    }
}

#[test]
fn consumed_request_cancellation_and_session_deadline_leave_only_the_durable_seed() {
    for deadline_case in [false, true] {
        let fixture = package(false, "stall_once");
        let mut driver = Driver::new(&fixture);
        if deadline_case {
            driver.process.prediction_timeout = Duration::from_secs(10);
        }
        let (mut session, (mut w, mut v)) = (session(2), workspaces());
        let path = fixture.root.join("aborted-attempt.sqlite");
        let mut recorder = new_archive(&path, &session);
        seed(&mut session, &mut w, &mut v, &mut driver, &mut recorder);
        let before = publication(&session);
        let before_state = state_bits(session.snapshot().state());
        let before_receipt = session
            .snapshot()
            .configured_provider_report()
            .unwrap()
            .clone();
        let marker = fixture.root.join("first-request-observed");
        let started = Instant::now();
        let result = session.step_with_configured_pressure(
            inputs(),
            &mut w,
            Some((&mut v, MU)),
            Some(&mut driver),
            deadline_case.then_some(started + Duration::from_secs(3)),
            |_| !deadline_case && marker.exists(),
        );
        if deadline_case {
            assert!(matches!(result, Err(SessionError::Deadline)));
        } else {
            assert!(matches!(
                result,
                Err(SessionError::Step(LiquidStepError::Carrier(
                    SimulationError::Pressure(PressureError::Cancelled { .. })
                )))
            ));
        }
        assert!(
            marker.exists(),
            "fixture must consume the actual request before abort"
        );
        assert_eq!(received(&fixture).len(), 1);
        assert_eq!(session.snapshot().frame(), 1);
        assert_eq!(publication(&session), before);
        assert_eq!(state_bits(session.snapshot().state()), before_state);
        receipt_unchanged(
            &before_receipt,
            session.snapshot().configured_provider_report().unwrap(),
        );
        let failure = &driver.failures[0];
        assert_eq!(failure.reason, NoCandidateReason::Cancelled);
        assert!(failure.dispatched);
        assert!(failure.invalid_payload_sha256.is_none());
        let attempt = driver.predictions[0].clone();
        assert_eq!(recorder.recorded_count(), 2);
        // No append after the rejected call. Deadline is not a supported SQLite
        // lifecycle enum: preserve failed + honest caller-declared detail.
        let outcome = if deadline_case { "failed" } else { "cancelled" };
        recorder
            .finish(
                outcome,
                if deadline_case {
                    "Session deadline; no interval accepted"
                } else {
                    "Session cancelled after request consumption; no interval accepted"
                },
                Some(1),
            )
            .unwrap();
        drop(recorder);
        drop(session);
        driver.drain();
        let original_sha = file_sha(&path);
        cold_reopen(&fixture, &path, outcome, 1, "seed");
        fresh_retry(&fixture, &path, &original_sha, &attempt);
        assert_eq!(file_sha(&path), original_sha);
        cold_reopen(&fixture, &path, outcome, 1, "seed");
    }
}

#[test]
fn entered_prediction_can_cancel_before_writer_dispatch_without_an_accepted_row() {
    let fixture = package(false, "solve");
    let mut driver = Driver::new(&fixture);
    let (mut session, (mut w, mut v)) = (session(2), workspaces());
    let path = fixture.root.join("predispatch-cancel.sqlite");
    let mut recorder = new_archive(&path, &session);
    seed(&mut session, &mut w, &mut v, &mut driver, &mut recorder);
    let before = publication(&session);
    let receipt = session
        .snapshot()
        .configured_provider_report()
        .unwrap()
        .clone();
    // Use the existing runtime owner's cancellation flag, not a synthetic
    // dispatch observation or an additional runtime/worker implementation.
    driver.flag.store(true, Ordering::Relaxed);
    let result = session.step_with_configured_pressure(
        inputs(),
        &mut w,
        Some((&mut v, MU)),
        Some(&mut driver),
        None,
        |_| false,
    );
    assert!(matches!(
        result,
        Err(SessionError::Step(LiquidStepError::Carrier(
            SimulationError::Pressure(PressureError::Cancelled { .. })
        )))
    ));
    assert_eq!(driver.predictions.len(), 1); // wrapper method entry only
    let failure = &driver.failures[0];
    assert_eq!(failure.reason, NoCandidateReason::Cancelled);
    assert!(!failure.dispatched); // existing writer never accepted this request
    assert!(failure.invalid_payload_sha256.is_none());
    assert!(received(&fixture).is_empty());
    assert!(!fixture.root.join("first-request-observed").exists());
    assert_eq!(publication(&session), before);
    receipt_unchanged(
        &receipt,
        session.snapshot().configured_provider_report().unwrap(),
    );
    assert_eq!(session.snapshot().frame(), 1);
    assert_eq!(recorder.recorded_count(), 2);
    let attempt = driver.predictions[0].clone();
    recorder
        .finish(
            "cancelled",
            "predict entered; runtime cancelled before writer dispatch",
            Some(1),
        )
        .unwrap();
    drop(recorder);
    drop(session);
    driver.drain();
    let original_sha = file_sha(&path);
    cold_reopen(&fixture, &path, "cancelled", 1, "seed");
    fresh_retry(&fixture, &path, &original_sha, &attempt);
    assert_eq!(file_sha(&path), original_sha);
}

#[test]
fn direct_runtime_write_deadline_reports_dispatch_and_keeps_owner_until_drop() {
    // Direct phase probe only: this deliberately large request is NOT a Session
    // accepted interval, and no accepted archive is created for this case.
    let prefix = include_str!("../../../../tests/fixtures/pressure_adapter_full.py")
        .split("for line in sys.stdin:")
        .next()
        .unwrap();
    let fixture = Fixture::source(&format!("{prefix}\nimport time\ntime.sleep(60)\n"), false);
    fixture.edit(|manifest| {
        let mut ports = serde_json::Map::new();
        for index in 0..32 {
            ports.insert(format!("coordinate_{index}"), json!("x"));
        }
        manifest["inputs"] = Value::Object(ports);
    });
    let slot = Arc::new(Mutex::new(None));
    let flag = AtomicBool::new(false);
    let mut process =
        ModelProcess::start(&fixture.manifest, Arc::clone(&slot), &flag, |_| Ok(())).unwrap();
    process.prediction_timeout = Duration::from_secs(1);
    let pid = slot.lock().unwrap().as_ref().unwrap().id();
    #[cfg(target_os = "linux")]
    assert_child_liveness(pid, true);
    #[cfg(not(target_os = "linux"))]
    let _ = pid;
    let grid = GridGeometry::new([32, 16, 16], [1.0; 3], [0.0; 3]).unwrap();
    let zeros = vec![0.0; grid.cell_len()];
    let failure = process
        .predict_typed_with_poll(direct_input(&grid, &zeros), &flag, &mut || false)
        .unwrap_err();
    assert_eq!(failure.reason, NoCandidateReason::RuntimeFailure);
    assert_eq!(failure.detail, "CPU model write deadline exceeded");
    assert!(failure.dispatched);
    assert!(failure.invalid_payload_sha256.is_none());
    assert!(!flag.load(Ordering::Relaxed));
    // An internal write timeout reports failure; it does not itself cancel the
    // child. The existing owner still performs immediate-child wait/join on Drop.
    #[cfg(target_os = "linux")]
    assert_child_liveness(pid, true);
    drop_process(process, &slot);
    assert!(slot.lock().unwrap().is_none());
    // No elapsed-time assertion or process-tree/hard-preemption claim.
}

// Linux x86_64 actual accepted-Session write-phase evidence.
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod accepted_session_write_contract {
    use super::*;

    #[test]
    fn actual_session_blocked_write_cancel_and_deadline_preserve_the_recorded_seed() {
        for deadline_case in [false, true] {
            let fixture = package(false, "solve");
            let script_path = fixture.root.join("adapter.py");
            let original_script = std::fs::read(&script_path).unwrap();
            let original_manifest = std::fs::read(&fixture.manifest).unwrap();
            let original_text = std::str::from_utf8(&original_script).unwrap();
            let loop_needle = "while raw := sys.stdin.buffer.readline(8193):";
            assert_eq!(original_text.matches(loop_needle).count(), 1);
            let prefix = original_text.split_once(loop_needle).unwrap().0;
            let ready_needle = "print(json.dumps(dict(kind=\"adapter_ready\"";
            assert_eq!(prefix.matches(ready_needle).count(), 1);
            let pipe_setup = r#"
# Linux-only authored test fixture: shrink BEFORE actual ready acknowledgement.
# No stdin read occurs anywhere in this authored process.
import array
import fcntl
import termios
stdin_fd = sys.stdin.fileno()
requested_capacity = 4096
actual_capacity = fcntl.fcntl(stdin_fd, fcntl.F_SETPIPE_SZ, requested_capacity)
require(actual_capacity == requested_capacity, "actual 4096-byte stdin pipe")
(root / "writer-pipe-capacity.json").write_text(json.dumps(dict(
    requested_bytes=requested_capacity, capacity_bytes=actual_capacity)), encoding="utf-8")
"#;
            let ready_prefix =
                prefix.replacen(ready_needle, &format!("{pipe_setup}\n{ready_needle}"), 1);
            // Preserve the original fixture's real ready packet and pins, then
            // observe occupancy with ioctl. FIONREAD does not consume a byte.
            let blocked_script = format!(
                r#"{ready_prefix}
while True:
    unread = array.array("i", [0])
    fcntl.ioctl(stdin_fd, termios.FIONREAD, unread, True)
    if unread[0] == actual_capacity:
        pending = root / "writer-pipe-full.pending"
        pending.write_text(json.dumps(dict(capacity_bytes=actual_capacity,
            unread_bytes=unread[0])), encoding="utf-8")
        pending.replace(root / "writer-pipe-full.json")
        break
    time.sleep(0.001)
time.sleep(60)
"#
            );
            std::fs::write(&script_path, &blocked_script).unwrap();
            let names = (0..32)
                .map(|index| format!("p{index:02}_{}", "x".repeat(124)))
                .collect::<Vec<_>>();
            assert!(
                names
                    .iter()
                    .all(|name| name.len() == 128 && name.is_ascii())
            );
            // Distinct ASCII JSON keys require their bytes + two quotes + colon.
            // This lower bound omits all values, commas, context and wrappers.
            // 32*(128+3)=4192 >4096: the full request cannot fit in this pipe
            // without a reader, even though the physical Session has TWO cells.
            let request_lower_bound = names.iter().map(|name| name.len() + 3).sum::<usize>();
            assert_eq!(request_lower_bound, 4192);
            assert!(request_lower_bound > 4096);
            fixture.edit(|manifest| {
                let mut ports = serde_json::Map::new();
                for name in &names {
                    ports.insert(name.clone(), json!("x"));
                }
                manifest["inputs"] = Value::Object(ports);
                manifest["files"]["adapter.py"]["sha256"] =
                    json!(format!("{:x}", Sha256::digest(blocked_script.as_bytes())));
                manifest["files"]["adapter.py"]["bytes"] = json!(blocked_script.len());
            });
            // Load AFTER re-signing these fixture-owned bytes. This is an
            // authored pinned fixture, never a tamper of a loaded deployment.
            let mut driver = Driver::new(&fixture);
            driver.process.prediction_timeout = Duration::from_secs(10);
            let capacity: Value = serde_json::from_slice(
                &std::fs::read(fixture.root.join("writer-pipe-capacity.json")).unwrap(),
            )
            .unwrap();
            assert_eq!(capacity["requested_bytes"], json!(4096));
            assert_eq!(capacity["capacity_bytes"], json!(4096));
            let (mut session, (mut w, mut v)) = (session(2), workspaces());
            let path = fixture.root.join("blocked-session-write.sqlite");
            let mut recorder = new_archive(&path, &session);
            seed(&mut session, &mut w, &mut v, &mut driver, &mut recorder);
            assert!(received(&fixture).is_empty());
            assert!(!fixture.root.join("first-request-observed").exists());
            let before = publication(&session);
            let before_state = state_bits(session.snapshot().state());
            let before_receipt = session
                .snapshot()
                .configured_provider_report()
                .unwrap()
                .clone();
            let full_marker = fixture.root.join("writer-pipe-full.json");
            assert!(!full_marker.exists());
            let started = Instant::now();
            let result = session.step_with_configured_pressure(
                inputs(),
                &mut w,
                Some((&mut v, MU)),
                Some(&mut driver),
                deadline_case.then_some(started + Duration::from_secs(3)),
                |_| !deadline_case && full_marker.exists(),
            );
            if deadline_case {
                assert!(matches!(result, Err(SessionError::Deadline)));
            } else {
                assert!(matches!(
                    result,
                    Err(SessionError::Step(LiquidStepError::Carrier(
                        SimulationError::Pressure(PressureError::Cancelled { .. })
                    )))
                ));
            }
            // Marker publication is atomic: callback existence cannot race an
            // incompletely written JSON payload before killing the child.
            let full_evidence = std::fs::read(&full_marker).unwrap();
            let full: Value = serde_json::from_slice(&full_evidence).unwrap();
            assert_eq!(full["capacity_bytes"], json!(4096));
            assert_eq!(full["unread_bytes"], json!(4096));
            assert!(request_lower_bound > full["capacity_bytes"].as_u64().unwrap() as usize);
            assert_eq!(driver.predictions.len(), 1);
            let failure = &driver.failures[0];
            assert_eq!(failure.reason, NoCandidateReason::Cancelled);
            assert!(failure.dispatched); // typed existing writer-queue fact
            assert!(failure.invalid_payload_sha256.is_none());
            assert!(driver.flag.load(Ordering::Relaxed)); // existing cooperative cancel path
            // Dispatch did not mean child consumption. The authored child never
            // reads stdin; its independent phase evidence is unread pipe bytes.
            assert!(received(&fixture).is_empty());
            assert!(!fixture.root.join("first-request-observed").exists());
            assert_eq!(session.snapshot().frame(), 1);
            assert_eq!(publication(&session), before);
            assert_eq!(state_bits(session.snapshot().state()), before_state);
            receipt_unchanged(
                &before_receipt,
                session.snapshot().configured_provider_report().unwrap(),
            );
            assert_eq!(recorder.recorded_count(), 2);
            let attempt = driver.predictions[0].clone();
            let outcome = if deadline_case { "failed" } else { "cancelled" };
            recorder
                .finish(
                    outcome,
                    if deadline_case {
                        "Session deadline during actual blocked request write; no interval accepted"
                    } else {
                        "Session cancelled at observed full stdin pipe; no interval accepted"
                    },
                    Some(1),
                )
                .unwrap();
            drop(recorder);
            drop(session);
            driver.drain();
            let original_sha = file_sha(&path);
            // Existing helper hides ONLY this generated manifest while its new
            // reader process verifies the accepted archive without deployment.
            cold_reopen(&fixture, &path, outcome, 1, "seed");
            assert_eq!(std::fs::read(&full_marker).unwrap(), full_evidence);
            assert!(received(&fixture).is_empty());
            assert!(!fixture.root.join("first-request-observed").exists());
            // Restore ONLY this fixture's saved original deployment. A fresh
            // Driver then attests the restored manifest's DIFFERENT pins. No
            // claim of identical deployment or persisted checkpoint recovery.
            std::fs::write(&script_path, &original_script).unwrap();
            std::fs::write(&fixture.manifest, &original_manifest).unwrap();
            fresh_retry(&fixture, &path, &original_sha, &attempt);
            assert_eq!(file_sha(&path), original_sha);
            cold_reopen(&fixture, &path, outcome, 1, "seed");
            // No elapsed-time, process-tree, portable pipe-size, or arbitrary
            // callback preemption guarantee follows from this Linux fixture.
        }
    }
}
