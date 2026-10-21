//! Actual stdlib Python subprocess observations, not trained-model qualification.
use super::super::cpu_contract_tests::Fixture;
use super::*;
use rheon::{GridGeometry, solver_provider::context::*};

fn slot() -> ChildSlot {
    Arc::new(Mutex::new(None))
}
fn start(package: &Fixture, flag: &AtomicBool) -> ModelProcess {
    ModelProcess::start(&package.manifest, slot(), flag, |_| Ok(())).unwrap()
}
fn input<'a>(grid: &'a GridGeometry, zero: &'a [f64], prior: Option<&'a [f64]>) -> Input<'a> {
    let dt = 1.0 / 60.0;
    Input {
        context: PressureContext::new(PressureContextInput {
            grid,
            density: 1000.0,
            dt,
            interval_start: if prior.is_some() { dt } else { 0.0 },
            rhs: zero,
            prior: prior.map(|p| PressureHistory {
                values: p,
                grid,
                density: 1000.0,
                run_id: "typed-runtime",
                generation: 1,
                interval_end: dt,
                status: PressureHistoryStatus::Diagnostic { qualified: true },
            }),
            run_id: "typed-runtime",
            generation: if prior.is_some() { 2 } else { 1 },
            mode: PressureContextMode::DiagnosticCandidate,
        })
        .unwrap(),
        initial_guess: prior.unwrap_or(zero),
        residual: zero,
        viscosity: 0.001,
    }
}
fn source(body: &str, correction: bool) -> Fixture {
    let prefix = include_str!("../../../../tests/fixtures/pressure_adapter_full.py")
        .split("for line in sys.stdin:")
        .next()
        .unwrap();
    let body = body
        .lines()
        .map(|line| format!("    {line}\n"))
        .collect::<String>();
    Fixture::source(
        &format!("{prefix}for line in sys.stdin:\n    request = json.loads(line)\n{body}"),
        correction,
    )
}
fn packet(raw: &str, correction: bool) -> Fixture {
    source(
        &format!(
            "sys.stdout.write({}); sys.stdout.flush()",
            serde_json::to_string(raw).unwrap()
        ),
        correction,
    )
}
fn edit(package: &Fixture, change: impl FnOnce(&mut Value)) {
    let mut value: Value =
        serde_json::from_slice(&std::fs::read(&package.manifest).unwrap()).unwrap();
    change(&mut value);
    std::fs::write(&package.manifest, serde_json::to_vec(&value).unwrap()).unwrap();
}
#[test]
fn typed_success_preserves_the_actual_subprocess_and_legacy_result() {
    let package = Fixture::new(false);
    let flag = AtomicBool::new(false);
    let grid = GridGeometry::new([2, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let zero = [0.0; 2];
    let mut typed = start(&package, &flag);
    assert_eq!(
        typed
            .predict_typed(input(&grid, &zero, None), &flag)
            .unwrap(),
        [0.0, 1.0]
    );
    drop(typed);
    let mut legacy = start(&package, &flag);
    assert_eq!(
        legacy.predict(input(&grid, &zero, None), &flag).unwrap(),
        [0.0, 1.0]
    );
}
#[test]
fn predispatch_support_context_cancellation_and_closed_input_have_typed_reasons() {
    let grid = GridGeometry::new([2, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let zero = [0.0; 2];
    let flag = AtomicBool::new(false);
    for (case, reason) in [
        (0, NoCandidateReason::Unsupported),
        (1, NoCandidateReason::UnsupportedContext),
        (2, NoCandidateReason::Cancelled),
        (3, NoCandidateReason::RuntimeFailure),
        (4, NoCandidateReason::UnsupportedContext),
    ] {
        let package = Fixture::new(case == 1);
        let mut process = start(&package, &flag);
        let other = GridGeometry::new([2, 1, 1], [1.0; 3], [1.0, 0.0, 0.0]).unwrap();
        let mut request = input(if case == 0 { &other } else { &grid }, &zero, None);
        if case == 2 {
            flag.store(true, Ordering::Relaxed);
        }
        if case == 3 {
            process.sender.take();
        }
        if case == 4 {
            request.residual = &[];
        }
        let failure = process.predict_typed(request, &flag).unwrap_err();
        assert_eq!(failure.reason, reason, "{failure:?}");
        assert!(!failure.dispatched);
        assert_eq!(failure.invalid_payload_sha256, None);
        drop(process);
        flag.store(false, Ordering::Relaxed);
    }
}
#[test]
fn actual_error_codes_override_misleading_message_words_without_text_classification() {
    let grid = GridGeometry::new([2, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let zero = [0.0; 2];
    let flag = AtomicBool::new(false);
    for (code, reason) in [
        ("unsupported", NoCandidateReason::Unsupported),
        ("unsupported_context", NoCandidateReason::UnsupportedContext),
        ("invalid", NoCandidateReason::UnsupportedContext),
        ("unavailable", NoCandidateReason::RuntimeFailure),
    ] {
        let raw = format!(
            "{}\n",
            json!({"kind":"error","code":code,"id":1,"message":format!("Cancelled Unsupported RuntimeFailure InvalidOutput\u{0}{}","z".repeat(4000))})
        );
        let package = packet(&raw, false);
        let mut process = start(&package, &flag);
        let failure = process
            .predict_typed(input(&grid, &zero, None), &flag)
            .unwrap_err();
        assert_eq!(failure.reason, reason, "{failure:?}");
        assert!(failure.dispatched);
        assert_eq!(failure.invalid_payload_sha256, None);
        assert!(failure.detail.chars().count() <= 2000);
        assert!(!failure.detail.chars().any(char::is_control));
    }
}
#[test]
fn invalid_wire_replies_keep_the_complete_line_digest_after_dispatch() {
    let grid = GridGeometry::new([2, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let zero = [0.0; 2];
    let flag = AtomicBool::new(false);
    let packets = [
        "{\"kind\":\"candidate\",\"id\":999,\"role\":\"full_pressure\",\"pressure\":[0,1],\"device\":\"cpu\",\"optimizer_updates\":0}\n",
        "{\"kind\":\"candidate\",\"id\":1,\"role\":\"full_pressure\",\"pressure\":[0],\"device\":\"cpu\",\"optimizer_updates\":0}\n",
        "{\"kind\":\"candidate\",\"id\":1,\"role\":\"full_pressure\",\"pressure\":[1,1],\"device\":\"cpu\",\"optimizer_updates\":0}\n",
        "{\"kind\":\"candidate\",\"id\":1,\"role\":\"full_pressure\",\"pressure\":[0,1],\"device\":\"cuda\",\"optimizer_updates\":0}\n",
        "{\"kind\":\"candidate\",\"id\":1,\"role\":\"full_pressure\",\"pressure\":[0,1],\"device\":\"cpu\",\"optimizer_updates\":1}\n",
        "{\"kind\":\"candidate\",\"id\":1,\"role\":\"pressure_correction\",\"pressure\":[0,1],\"device\":\"cpu\",\"optimizer_updates\":0}\n",
        "{\"kind\":\"candidate\",\"id\":1,\"role\":\"full_pressure\",\"pressure\":[0,1e999],\"device\":\"cpu\",\"optimizer_updates\":0}\n",
        "{\"kind\":\"candidate\",\"id\":1,\"role\":\"full_pressure\",\"pressure\":[0,NaN],\"device\":\"cpu\",\"optimizer_updates\":0}\n",
        "{\"kind\":\"loading\",\"phase\":\"unexpected in prediction\"}\n",
        "{\"kind\":\"error\",\"code\":\"unavailable\",\"id\":2,\"message\":\"wrong request\"}\n",
        "{malformed JSON}\n",
    ];
    for raw in packets {
        let package = packet(raw, false);
        let mut process = start(&package, &flag);
        let failure = process
            .predict_typed(input(&grid, &zero, None), &flag)
            .unwrap_err();
        assert_eq!(
            failure.reason,
            NoCandidateReason::InvalidOutput,
            "{failure:?}"
        );
        assert!(failure.dispatched);
        assert_eq!(
            failure.invalid_payload_sha256,
            Some(Sha256::digest(raw.as_bytes()).into())
        );
    }
}
#[test]
fn correction_overflow_is_invalid_output_with_the_actual_reply_digest() {
    let raw = "{\"kind\":\"candidate\",\"id\":2,\"role\":\"pressure_correction\",\"pressure\":[0,1e308],\"device\":\"cpu\",\"optimizer_updates\":0}\n";
    let package = packet(raw, true);
    let flag = AtomicBool::new(false);
    let grid = GridGeometry::new([2, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let zero = [0.0; 2];
    let prior = [0.0, 1e308];
    let mut process = start(&package, &flag);
    let failure = process
        .predict_typed(input(&grid, &zero, Some(&prior)), &flag)
        .unwrap_err();
    assert_eq!(failure.reason, NoCandidateReason::InvalidOutput);
    assert!(failure.dispatched);
    assert_eq!(
        failure.invalid_payload_sha256,
        Some(Sha256::digest(raw.as_bytes()).into())
    );
}
#[test]
fn reply_deadline_and_child_eof_are_runtime_failures_after_dispatch() {
    let grid = GridGeometry::new([2, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let zero = [0.0; 2];
    let flag = AtomicBool::new(false);
    for body in ["import time; time.sleep(60)", "raise SystemExit(0)"] {
        let package = source(body, false);
        let mut process = start(&package, &flag);
        process.prediction_timeout = Duration::from_millis(100);
        let failure = process
            .predict_typed(input(&grid, &zero, None), &flag)
            .unwrap_err();
        assert_eq!(failure.reason, NoCandidateReason::RuntimeFailure);
        assert!(failure.dispatched);
        assert_eq!(failure.invalid_payload_sha256, None);
    }
}
#[test]
fn cancellation_after_child_observes_request_preserves_dispatch() {
    let package = source(
        "Path(__file__).with_suffix('.observed').write_text('request received')\nimport time; time.sleep(60)",
        false,
    );
    let flag = Arc::new(AtomicBool::new(false));
    let mut process = start(&package, &flag);
    let marker = package.root.join("adapter.observed");
    let cancel = Arc::clone(&flag);
    let watcher = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !marker.exists() {
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(5));
        }
        cancel.store(true, Ordering::Relaxed);
    });
    let grid = GridGeometry::new([2, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let zero = [0.0; 2];
    let failure = process
        .predict_typed(input(&grid, &zero, None), &flag)
        .unwrap_err();
    watcher.join().unwrap();
    assert_eq!(failure.reason, NoCandidateReason::Cancelled);
    assert!(failure.dispatched);
    assert_eq!(failure.invalid_payload_sha256, None);
}
#[test]
fn request_wire_budget_is_rejected_before_dispatch() {
    let package = Fixture::new(false);
    edit(&package, |manifest| {
        let mut ports = serde_json::Map::new();
        for n in 0..32 {
            ports.insert(format!("x{n}"), json!("x"));
        }
        manifest["inputs"] = Value::Object(ports);
    });
    let flag = AtomicBool::new(false);
    let mut process = start(&package, &flag);
    let grid = GridGeometry::new([40; 3], [1.0; 3], [0.0; 3]).unwrap();
    let zero = vec![0.0; grid.cell_len()];
    let failure = process
        .predict_typed(input(&grid, &zero, None), &flag)
        .unwrap_err();
    assert_eq!(failure.reason, NoCandidateReason::UnsupportedContext);
    assert!(!failure.dispatched);
    assert_eq!(failure.invalid_payload_sha256, None);
}
#[test]
fn pinned_start_rejects_changed_binding_before_any_child_side_effect() {
    let marker_source="from pathlib import Path\nPath(__file__).with_suffix('.spawned').write_text('model code executed')\n".to_owned()
        +include_str!("../../../../tests/fixtures/pressure_adapter_full.py");
    let package = Fixture::source(&marker_source, false);
    let marker = package.root.join("adapter.spawned");
    let expected =
        ModelProcess::preflight(&package.manifest, &super::super::scene::Scene::default()).unwrap();
    assert!(!marker.exists());
    edit(&package, |value| value["version"] = json!("changed"));
    let flag = AtomicBool::new(false);
    let child = slot();
    let mut phases = 0;
    let result = ModelProcess::start_pinned(
        &package.manifest,
        Arc::clone(&child),
        &flag,
        &expected,
        |_| {
            phases += 1;
            Ok(())
        },
    );
    let error = match result {
        Ok(_) => panic!("changed binding executed"),
        Err(e) => e,
    };
    assert!(error.contains("frozen pin"));
    assert_eq!(phases, 0);
    assert!(child.lock().unwrap().is_none());
    assert!(!marker.exists());
    let current =
        ModelProcess::preflight(&package.manifest, &super::super::scene::Scene::default()).unwrap();
    let mut different = current.clone();
    different.training_run_id = Some("unexpected training identity".into());
    assert!(
        ModelProcess::start_pinned(
            &package.manifest,
            Arc::clone(&child),
            &flag,
            &different,
            |_| Ok(())
        )
        .is_err()
    );
    assert!(!marker.exists());
    let process = ModelProcess::start_pinned(
        &package.manifest,
        Arc::clone(&child),
        &flag,
        &current,
        |_| Ok(()),
    )
    .unwrap();
    assert!(marker.exists());
    drop(process);
    assert!(child.lock().unwrap().is_none());
}
