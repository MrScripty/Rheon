//! Real subprocess contract/failure tests using independently authored stdlib
//! analytic adapters. These do not qualify the absent trained CPU deployment.
use super::{
    engine::Input,
    model_process::{ChildSlot, ModelProcess},
    scene::Scene,
};
use rheon::{GridGeometry, solver_provider::context::*};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    path::PathBuf,
    process::Command,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

/// Probe in the child's PID namespace; /proc can be mounted from another one.
#[cfg(target_os = "linux")]
pub(crate) fn assert_child_liveness(pid: u32, expected_alive: bool) {
    let result = Command::new("/usr/bin/kill")
        .args(["-0", "--", &pid.to_string()])
        .output()
        .expect("Linux lifecycle tests require /usr/bin/kill");
    assert!(
        matches!(result.status.code(), Some(0 | 1)),
        "invalid liveness probe status: {:?}",
        result.status
    );
    assert_eq!(
        result.status.success(),
        expected_alive,
        "child {pid} liveness: {}",
        String::from_utf8_lossy(&result.stderr)
    );
}

// Wall-clock timestamps can repeat on coarse clocks and concurrent test threads.
// Atomic directory creation owns the path; a stale or competing path is skipped.
fn fixture_directory(timestamp: u128) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    loop {
        let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "rheon-cpu-contract-{}-{timestamp}-{sequence}",
            std::process::id(),
        ));
        match std::fs::create_dir(&root) {
            Ok(()) => return root,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("create fixture directory {}: {error}", root.display()),
        }
    }
}

#[test]
fn fixture_directories_are_distinct_with_same_timestamp() {
    let roots: Vec<_> = std::thread::scope(|scope| {
        let threads: Vec<_> = (0..32)
            .map(|_| scope.spawn(|| fixture_directory(0)))
            .collect();
        threads
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .collect()
    });
    let unique: std::collections::HashSet<_> = roots.iter().collect();
    assert_eq!(unique.len(), roots.len());
    for root in roots {
        std::fs::remove_dir(root).unwrap();
    }
}

pub(crate) struct Fixture {
    pub root: PathBuf,
    pub manifest: PathBuf,
}
impl Fixture {
    pub fn new(correction: bool) -> Self {
        Self::source(
            if correction {
                include_str!("../../../../tests/fixtures/pressure_adapter_correction.py")
            } else {
                include_str!("../../../../tests/fixtures/pressure_adapter_full.py")
            },
            correction,
        )
    }
    pub fn source(source: &str, correction: bool) -> Self {
        let root = fixture_directory(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
        );
        let output =
            Command::new(std::env::var_os("RHEON_TEST_PYTHON").unwrap_or_else(|| "python3".into()))
                .args(["-I", "-S", "-c", "import sys; print(sys.executable)"])
                .output()
                .expect("CPU protocol tests require Python3");
        assert!(output.status.success());
        let python = PathBuf::from(String::from_utf8(output.stdout).unwrap().trim())
            .canonicalize()
            .unwrap();
        let bytes = std::fs::read(&python).unwrap();
        std::fs::write(root.join("adapter.py"), source).unwrap();
        let manifest = root.join("adapter.json");
        let inputs = if correction {
            json!({"locations":"x","prior":"previous_pressure"})
        } else {
            json!({"coordinate":"x"})
        };
        let packet = json!({"schema":"rheon-pressure-adapter-v1","id":if correction {"analytic-correction"}else{"analytic-full"},"version":"1","stage":STAGE_CONTRACT,
            "python":{"executable":python,"sha256":format!("{:x}",Sha256::digest(&bytes)),"bytes":bytes.len()},"entrypoint":"adapter.py",
            "files":{"adapter.py":{"sha256":format!("{:x}",Sha256::digest(source.as_bytes())),"bytes":source.len()}},
            "support":{"density_kg_m3":1000.0,"dynamic_viscosity_pa_s":0.001,"dt_s":1.0/60.0,"counts_min":[1,1,1],"counts_max":[128,128,128],"max_cells":65536,"spacing_min_m":1.0/128.0,"spacing_max_m":1.0,"origin":"zero","cells":"cubic"},
            "context":{"minimum_native_steps":0,"prior_pressure":if correction {"required"}else{"optional"}},"inputs":inputs,
            "output":{"role":if correction {"pressure_correction"}else{"full_pressure"},"dtype":"f64","layout":"x_fast","gauge":"cell_zero"}});
        std::fs::write(&manifest, serde_json::to_vec(&packet).unwrap()).unwrap();
        Self { root, manifest }
    }
    fn edit(&self, f: impl FnOnce(&mut Value)) {
        let mut packet: Value =
            serde_json::from_slice(&std::fs::read(&self.manifest).unwrap()).unwrap();
        f(&mut packet);
        std::fs::write(&self.manifest, serde_json::to_vec(&packet).unwrap()).unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.root).unwrap();
    }
}
fn slot() -> ChildSlot {
    Arc::new(Mutex::new(None))
}
fn input<'a>(
    grid: &'a GridGeometry,
    rhs: &'a [f64],
    prior: Option<&'a [f64]>,
    residual: &'a [f64],
    guess: &'a [f64],
) -> Input<'a> {
    let dt = 1.0 / 60.0;
    let context = PressureContext::new(PressureContextInput {
        grid,
        density: 1000.0,
        dt,
        interval_start: if prior.is_some() { dt } else { 0.0 },
        rhs,
        prior: prior.map(|values| PressureHistory {
            values,
            grid,
            density: 1000.0,
            run_id: "fixture",
            generation: 1,
            interval_end: dt,
            status: PressureHistoryStatus::Diagnostic { qualified: false },
        }),
        run_id: "fixture",
        generation: if prior.is_some() { 2 } else { 1 },
        mode: PressureContextMode::DiagnosticCandidate,
    })
    .unwrap();
    Input {
        context,
        initial_guess: guess,
        residual,
        viscosity: 0.001,
    }
}

#[test]
fn two_actual_packages_change_ports_and_output_role_without_host_changes() {
    let grid = GridGeometry::new([2, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    // Independent two-cell operator with h/rho=.001 and p=[0,1] Pa.
    let rhs = [-0.001, 0.001];
    let previous = [0.0, 0.25];
    let zero = [0.0; 2];
    let cancel = AtomicBool::new(false);
    for correction in [false, true] {
        let package = Fixture::new(correction);
        let child = slot();
        let mut process =
            ModelProcess::start(&package.manifest, Arc::clone(&child), &cancel, |_| Ok(()))
                .unwrap();
        let mut residual = [0.0; 2];
        input(
            &grid,
            &rhs,
            if correction { Some(&previous) } else { None },
            &zero,
            &zero,
        )
        .context
        .initial_residual(&mut residual)
        .unwrap();
        let pressure = process
            .predict(
                input(
                    &grid,
                    &rhs,
                    if correction { Some(&previous) } else { None },
                    &residual,
                    if correction { &previous } else { &zero },
                ),
                &cancel,
            )
            .unwrap();
        assert_eq!(pressure, [0.0, 1.0]);
        assert!(process.identity.checkpoint_sha256.is_none());
        let pid = child.lock().unwrap().as_ref().unwrap().id();
        #[cfg(target_os = "linux")]
        assert_child_liveness(pid, true);
        drop(process);
        assert!(child.lock().unwrap().is_none());
        #[cfg(target_os = "linux")]
        assert_child_liveness(pid, false);
    }
}

#[test]
fn support_preflight_is_inert_and_rejects_before_execution() {
    let source =
        "from pathlib import Path\nPath(__file__).with_suffix('.executed').write_text('bad')\n";
    let package = Fixture::source(source, false);
    package.edit(|m| m["support"]["dt_s"] = json!(0.5));
    assert!(
        ModelProcess::preflight(&package.manifest, &Scene::default())
            .unwrap_err()
            .contains("outside declared support")
    );
    assert!(!package.root.join("adapter.executed").exists());
}

#[test]
fn higher_density_does_not_widen_legacy_or_adapter_physical_support() {
    let source =
        "from pathlib import Path\nPath(__file__).with_suffix('.executed').write_text('bad')\n";
    let package = Fixture::source(source, false);
    let scene = Scene {
        cells_per_metre: 1000,
        ..Default::default()
    };
    assert!(
        ModelProcess::preflight(&package.manifest, &scene)
            .unwrap_err()
            .contains("outside declared support")
    );
    // A package may explicitly declare a smaller spacing. This is an inert
    // contract check only, not a numerical/model qualification.
    package.edit(|m| m["support"]["spacing_min_m"] = json!(0.001));
    assert!(ModelProcess::preflight(&package.manifest, &scene).is_ok());
    assert!(!package.root.join("adapter.executed").exists());
    let adapter: Value =
        serde_json::from_slice(&std::fs::read(&package.manifest).unwrap()).unwrap();
    std::fs::write(package.root.join("cpu_bridge.py"), source).unwrap();
    let legacy = json!({"schema":"rheon-pressure-cpu-deployment-v2","python":adapter["python"]["executable"],"bridge":"cpu_bridge.py","python_sha256":adapter["python"]["sha256"],
        "files":{"cpu_bridge.py":{"sha256":format!("{:x}",Sha256::digest(source.as_bytes())),"bytes":source.len()}},
        "checkpoint_sha256":"a".repeat(64),"normalization_sha256":"b".repeat(64),"source_commit":"c".repeat(40),"training_run_id":"d".repeat(32),"training_updates":1,
        "profile":"sealed-water-seven-channel-float32","physical_contract":{"density_kg_m3":1000.0,"dynamic_viscosity_pa_s":0.001,"cells":"cubic"},
        "context_policy":{"mode":"native_prefix","minimum_native_steps":0,"training_snapshot_indices":[0]},"assumption":"Synthetic inert descriptor, no trained model"});
    std::fs::write(&package.manifest, serde_json::to_vec(&legacy).unwrap()).unwrap();
    assert!(ModelProcess::preflight(&package.manifest, &Scene::default()).is_ok());
    assert!(
        ModelProcess::preflight(&package.manifest, &scene)
            .unwrap_err()
            .contains("legacy v2 package supports at most 128 cells/m")
    );
    assert!(!package.root.join("cpu_bridge.executed").exists());
}

#[test]
fn stalled_input_write_times_out_and_owned_child_and_io_threads_drain() {
    let package = Fixture::new(false);
    let original = std::fs::read_to_string(package.root.join("adapter.py")).unwrap();
    let source = original
        .split("for line in sys.stdin:")
        .next()
        .unwrap()
        .to_owned()
        + "\nimport time\ntime.sleep(60)\n";
    std::fs::write(package.root.join("adapter.py"), &source).unwrap();
    package.edit(|m| {
        m["files"]["adapter.py"]["bytes"] = json!(source.len());
        m["files"]["adapter.py"]["sha256"] =
            json!(format!("{:x}", Sha256::digest(source.as_bytes())));
    });
    let child = slot();
    let cancel = AtomicBool::new(false);
    let mut process =
        ModelProcess::start(&package.manifest, Arc::clone(&child), &cancel, |_| Ok(())).unwrap();
    process.prediction_timeout = Duration::from_millis(200);
    let grid = GridGeometry::new([40, 40, 40], [1.0; 3], [0.0; 3]).unwrap();
    let zero = vec![0.0; 64000];
    let result = process.predict(input(&grid, &zero, None, &zero, &zero), &cancel);
    assert!(result.unwrap_err().contains("write deadline"));
    let pid = child.lock().unwrap().as_ref().unwrap().id();
    #[cfg(target_os = "linux")]
    assert_child_liveness(pid, true);
    drop(process);
    assert!(child.lock().unwrap().is_none());
    #[cfg(target_os = "linux")]
    assert_child_liveness(pid, false);
}

#[test]
fn legacy_records_still_match_the_actual_lab_v2_reader_contract() {
    let package = Fixture::new(false);
    let scene = Scene {
        counts: [5; 3],
        steps: 16,
        ..Default::default()
    };
    let mut record = super::record::Record::begin(&package.root.join("legacy"), &scene).unwrap();
    let identity = super::model_process::ProviderIdentity {
        id: "a".repeat(64),
        version: "legacy-v2".into(),
        manifest_sha256: "b".repeat(64),
        checkpoint_sha256: Some("a".repeat(64)),
        training_run_id: Some("c".repeat(32)),
    };
    record.model(&identity, &package.manifest).unwrap();
    let frame = super::engine::Engine::new(scene.clone()).unwrap().initial();
    record.frame(&frame).unwrap();
    let model: Value =
        serde_json::from_slice(&std::fs::read(record.directory.join("model.json")).unwrap())
            .unwrap();
    let packet: Value =
        serde_json::from_slice(&std::fs::read(record.directory.join("frame-0000.json")).unwrap())
            .unwrap();
    // Independent consumer conditions from pinned lab a842606 export_fields.py.
    assert_eq!(packet["schema"], "rheon-live-water-comparison-frame-v2");
    assert_eq!(packet["counts_xyz"], json!(scene.counts));
    assert_eq!(packet["cell_size_m"], json!(scene.h()));
    assert_eq!(packet["checkpoint_sha256"], model["checkpoint_sha256"]);
    assert_eq!(packet["frame"]["step"], 0);
    record
        .model(
            &super::model_process::ProviderIdentity {
                checkpoint_sha256: None,
                training_run_id: None,
                id: "generic".into(),
                ..identity
            },
            &package.manifest,
        )
        .unwrap();
    let mut next = frame;
    next.step = 1;
    record.frame(&next).unwrap();
    let generic: Value =
        serde_json::from_slice(&std::fs::read(record.directory.join("frame-0001.json")).unwrap())
            .unwrap();
    assert_eq!(generic["schema"], "rheon-live-water-comparison-frame-v3");
    assert!(generic["checkpoint_sha256"].is_null());
    assert_eq!(generic["provider"]["id"], "generic");
}

#[test]
fn inert_preflight_result_survives_finished_worker_rendering() {
    let package = Fixture::new(false);
    let mut app = super::ComparisonApp::new(
        package.manifest.to_string_lossy().into_owned(),
        package.root.join("project"),
    );
    app.scene = Scene {
        counts: [8, 6, 5],
        steps: 16,
        ..Default::default()
    };
    app.start();
    let deadline = std::time::Instant::now() + Duration::from_secs(20);
    while app.running() {
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(
        app.worker.as_ref().unwrap().shared.lock().unwrap().state,
        "completed"
    );
    app.deployment = package
        .root
        .join("missing.json")
        .to_string_lossy()
        .into_owned();
    app.check_package();
    let diagnostic = app.preflight_status.clone();
    assert!(diagnostic.is_some());
    let ctx = eframe::egui::Context::default();
    let _ = ctx.run(Default::default(), |ctx| {
        app.show(ctx);
    });
    assert_eq!(app.preflight_status, diagnostic);
    assert!(app.status.contains("completed"));
}

#[path = "accepted_provider_contract_tests.rs"]
mod accepted_provider_contract_tests;
