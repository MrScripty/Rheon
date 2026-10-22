//! Actual existing Worker -> production bridge -> Session -> SQLite -> cold reader.
use super::*;
use ai_comparison::{adapter::AdapterPackage, cpu_contract_tests::Fixture};
use rheon::{
    recording::{
        provider::{runtime::*, *},
        *,
    },
    *,
};
use run_sets::{ArchiveBudget, MemberState, configured::*};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
};

const CAP: usize = 16 * 1024 * 1024;
pub(super) struct Output(pub(super) PathBuf);
impl Output {
    pub(super) fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "rheon-configured-worker-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Output {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
pub(super) fn package(correction: bool, behavior: &str) -> Fixture {
    let source = include_str!("../../../tests/fixtures/configured_box_pressure_adapter.py")
        .replace("BEHAVIOR = \"solve\"", &format!("BEHAVIOR = {behavior:?}"));
    let fixture = Fixture::source(&source, correction);
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(&fixture.manifest).unwrap()).unwrap();
    manifest["support"] = json!({"density_kg_m3":2.0,"dynamic_viscosity_pa_s":0.0,"dt_s":1.0/16.0,"counts_min":[1,1,1],"counts_max":[4,4,4],"max_cells":64,"spacing_min_m":0.25,"spacing_max_m":0.25,"origin":"zero","cells":"cubic"});
    manifest["context"] = json!({"minimum_native_steps":1,"prior_pressure":if correction {"required"} else {"optional"}});
    manifest["inputs"] = json!({"equation_rhs":"rhs","full_residual":"initial_residual","material_density":"density","actual_dt":"dt","centres_x":"x","solved_previous":"previous_pressure"});
    std::fs::write(&fixture.manifest, serde_json::to_vec(&manifest).unwrap()).unwrap();
    fixture
}
pub(super) fn set(fixture: &Fixture) -> ConfiguredRunSetRevision {
    let package = AdapterPackage::load(&fixture.manifest).unwrap();
    let manifest = package.manifest_sha256();
    let pin = ConfiguredProviderPin {
        id: package.id().into(),
        version: package.version().into(),
        manifest_sha256: std::array::from_fn(|i| {
            u8::from_str_radix(&manifest[2 * i..2 * i + 2], 16).unwrap()
        }),
        checkpoint_sha256: None,
        training_run_id: None,
        runtime: "actual-stdlib-python-cpu-fixture".into(),
        source_revision: "independent-rational-box-fixture".into(),
    };
    let constructor = SimulationSession::new(
        SessionIdentity {
            run_id: 0,
            source_revision: configured_executor().source_revision,
        },
        GridGeometry::new([2; 3], [0.25; 3], [0.0; 3]).unwrap(),
        LiquidTransportConfig {
            carrier: SimulationConfig {
                density: 2.0,
                memory_limit: CAP,
                pressure: PressureSettings {
                    relative_residual: 1e-12,
                    absolute_residual: 1e-12,
                    divergence_limit: 1e-9,
                    max_iterations: 1000,
                },
                actual_divergence_limit: 1e-5,
                max_courant: 0.5,
            },
            represented_density: 2.0,
            volume_stamp: VolumeStamp { id: 1, version: 0 },
            carrier_id: 2,
        },
        PressureImplementation::JacobiPcgV1,
        vec![1.0; 8],
        CAP,
        3,
    )
    .unwrap();
    let future = ConfiguredFutureIntent {
        requested_dt_s: 1.0 / 16.0,
        requested_steps: 3,
        forces: vec![ForceSpec {
            value: [0.125, 0.0, 0.0],
            units: ForceUnitsSpec::Acceleration,
            region: None,
        }],
        volume: VolumeControls {
            max_outward_courant: 0.5,
            actual_divergence_per_s: 1e-5,
        },
        viscosity: ViscosityIntent::Disabled,
        pressure: PressureIntent::NativeOnly,
        provider_workspace_limit_bytes: CAP,
        viscosity_workspace_limit_bytes: None,
        recording: ArchiveBudget {
            max_database_bytes: 32 * 1024 * 1024,
            max_record_bytes: 1024 * 1024,
            max_records: 4,
        },
        capture_constructor: true,
        executor: configured_executor(),
    };
    let baseline =
        FrozenConfiguredSpec::capture_constructor(&constructor.snapshot(), future.clone()).unwrap();
    let configured = FrozenConfiguredSpec::capture_constructor(
        &constructor.snapshot(),
        ConfiguredFutureIntent {
            pressure: PressureIntent::NativePrefixThenConfigured {
                provider: Box::new(pin),
                deployment: DeploymentLocator::Absolute(fixture.manifest.to_string_lossy().into()),
                requested_native_prefix_steps: 1,
                minimum_native_prefix_steps: 1,
            },
            ..future
        },
    )
    .unwrap();
    ConfiguredRunSetRevision {
        schema_version: CONFIGURED_VERSION,
        kind: ConfiguredSetKind::ConfiguredAcceptedSessionSet,
        set_id: "worker-fixture".into(),
        parent_sha256: None,
        baseline_member: "native".into(),
        members: vec![
            ConfiguredRunSetMember {
                member_id: "native".into(),
                spec: baseline.clone(),
                differences_from_baseline: vec![],
                attempt: None,
            },
            ConfiguredRunSetMember {
                member_id: "configured".into(),
                differences_from_baseline: ConfiguredSpecDifference::between(
                    &baseline,
                    &configured,
                )
                .unwrap(),
                spec: configured,
                attempt: None,
            },
        ],
    }
}
pub(super) fn app(output: &Output) -> App {
    let mut app = App::default();
    app.recording_directory = output.0.to_string_lossy().into();
    app.threads = 1;
    app.show_workers = false;
    app
}
fn wait(mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(20);
    while !condition() {
        assert!(Instant::now() < deadline, "worker fixture deadline");
        std::thread::sleep(Duration::from_millis(2));
    }
}
pub(super) fn launch(app: &mut App, set: ConfiguredRunSetRevision, root: &Path) {
    app.publish_and_launch_configured(set, root, vec![])
        .unwrap();
}
pub(super) fn finish(app: &mut App) -> ConfiguredRunSetRevision {
    wait(|| app.worker.as_ref().unwrap().handle.is_finished());
    app.poll(&egui::Context::default());
    assert!(app.worker.is_none());
    ConfiguredRunSetRevision::open(Path::new(&app.run_sets.path)).unwrap()
}
pub(super) fn member(root: &Path, set: &ConfiguredRunSetRevision, index: usize) -> RecordingReader {
    match archive::open_member(root, &set.members[index]) {
        MemberState::Ready(reader) => *reader,
        MemberState::Invalid(error) => panic!("configured member invalid: {error}"),
        MemberState::Unsupported(reason) => panic!("configured member unsupported: {reason}"),
        _ => panic!("configured member not ready"),
    }
}
fn signature(reader: &RecordingReader) -> String {
    let mut hash = Sha256::new();
    for ordinal in 0..reader.count().unwrap() {
        hash.update(
            rheon::recording::comparison::Selection::read(reader, ordinal)
                .unwrap()
                .digest(),
        );
    }
    format!("{:x}", hash.finalize())
}
fn cold(manifest: &Path, index: usize, reader: &RecordingReader, hidden: &Path) {
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "configured_worker_tests::configured_worker_cold_child",
            "--nocapture",
        ])
        .env_clear()
        .env("PATH", "/no-model-runtime")
        .env("RHEON_COLD_SET", manifest)
        .env("RHEON_COLD_MEMBER", index.to_string())
        .env("RHEON_COLD_SIGNATURE", signature(reader))
        .env("RHEON_COLD_HIDDEN", hidden)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("ACTUAL_CONFIGURED_WORKER_COLD_VERIFIED"),
        "{stdout}"
    );
    assert!(stdout.contains("1 passed"), "{stdout}");
}
#[test]
fn configured_worker_cold_child() {
    let Ok(manifest) = std::env::var("RHEON_COLD_SET") else {
        return;
    };
    assert!(!Path::new(&std::env::var("RHEON_COLD_HIDDEN").unwrap()).exists());
    let path = Path::new(&manifest);
    let set = ConfiguredRunSetRevision::open(path).unwrap();
    let index = std::env::var("RHEON_COLD_MEMBER").unwrap().parse().unwrap();
    let reader = member(path.parent().unwrap(), &set, index);
    audit_runtime_termination(&reader).unwrap();
    assert_eq!(
        signature(&reader),
        std::env::var("RHEON_COLD_SIGNATURE").unwrap()
    );
    println!("ACTUAL_CONFIGURED_WORKER_COLD_VERIFIED");
}
#[test]
fn actual_paired_worker_records_full_and_correction_then_cold_rerun_preserves_originals() {
    for correction in [false, true] {
        let fixture = package(correction, "solve");
        let output = Output::new();
        let mut app = app(&output);
        let root = output.0.join("first.runset");
        launch(&mut app, set(&fixture), &root);
        let labels = (
            app.running_request.as_ref().unwrap().track_label(0),
            app.running_request.as_ref().unwrap().track_label(1),
        );
        assert_ne!(labels.0, labels.1);
        assert!(labels.0.contains("native"));
        assert!(labels.1.contains("configured"));
        let saved = finish(&mut app);
        assert_eq!(app.rows.len(), 2, "{}", app.status);
        let reference = member(&root, &saved, 0);
        let candidate = member(&root, &saved, 1);
        assert_eq!(reference.count().unwrap(), 4);
        assert_eq!(candidate.count().unwrap(), 4);
        let accepted =
            RecordedConfiguredSession::decode(candidate.run(), &candidate.select(2).unwrap())
                .unwrap();
        assert_eq!(
            accepted.receipt.as_ref().unwrap().selected,
            RecordedPressureSelection::Configured
        );
        assert!(matches!(
            accepted.runtime.as_ref().unwrap().outcome,
            RuntimeOutcome::CandidateReturned
        ));
        assert_eq!(
            accepted
                .runtime
                .as_ref()
                .unwrap()
                .request
                .context
                .generation,
            2
        );
        assert_eq!(
            audit_runtime_termination(&candidate)
                .unwrap()
                .termination
                .cause,
            RuntimeTerminationCause::Completed
        );
        for ordinal in 0..4 {
            let a = reference.select(ordinal).unwrap();
            let b = candidate.select(ordinal).unwrap();
            assert_eq!(a.time.to_bits(), b.time.to_bits());
            for field in &a.fields {
                // Independent hydrostatic fixture: dp/dx=rho*a=1/4 Pa/m,
                // cell0 gauge and dx=1/4 m. Native iterative rounding and an
                // independent rational solve need not have identical bits.
                for actual in [field, b.field(&field.name).unwrap()] {
                    let values: Vec<f64> = match actual.scalar_type.as_str() {
                        "f32" => actual
                            .data
                            .chunks_exact(4)
                            .map(|v| f64::from(f32::from_le_bytes(v.try_into().unwrap())))
                            .collect(),
                        "f64" => actual
                            .data
                            .chunks_exact(8)
                            .map(|v| f64::from_le_bytes(v.try_into().unwrap()))
                            .collect(),
                        other => panic!("Unexpected native field scalar {other}"),
                    };
                    for (index, value) in values.iter().enumerate() {
                        let expected = match field.name.as_str() {
                            "interval_pressure" => (index % 2) as f64 / 16.0,
                            "fraction" => 1.0,
                            "velocity_x" | "velocity_y" | "velocity_z" | "tracer" => 0.0,
                            other => panic!("Unexpected native field {other}"),
                        };
                        assert!(
                            (value - expected).abs() <= 1e-12,
                            "{}[{index}] actual {value}, hydrostatic {expected}",
                            field.name
                        );
                    }
                }
            }
        }
        let path = root.join(
            &saved.members[1]
                .attempt
                .as_ref()
                .unwrap()
                .archive_relative_path,
        );
        let original = std::fs::read(&path).unwrap();
        let original_manifest = std::fs::read(&app.run_sets.path).unwrap();
        let frozen_manifest = PathBuf::from(&app.run_sets.path);
        let hidden = fixture.manifest.with_extension("hidden");
        std::fs::rename(&fixture.manifest, &hidden).unwrap();
        cold(&frozen_manifest, 1, &candidate, &fixture.manifest);
        drop(reference);
        drop(candidate);
        app.rerun_configured(saved.clone()).unwrap();
        let retry = finish(&mut app);
        let retry_root = Path::new(&app.run_sets.path).parent().unwrap();
        let retry_reader = member(retry_root, &retry, 1);
        assert_eq!(retry_reader.count().unwrap(), 1);
        let audit = audit_runtime_termination(&retry_reader).unwrap();
        assert_eq!(
            audit.termination.cause,
            RuntimeTerminationCause::StartupFailure
        );
        assert_eq!(audit.termination.accepted_generation, 0);
        assert!(audit.termination.observation.is_none());
        assert_ne!(
            retry.members[1].attempt.as_ref().unwrap().expected_run_id,
            saved.members[1].attempt.as_ref().unwrap().expected_run_id
        );
        assert_eq!(std::fs::read(path).unwrap(), original);
        assert_eq!(std::fs::read(frozen_manifest).unwrap(), original_manifest);
        std::fs::rename(hidden, &fixture.manifest).unwrap();
    }
}
#[test]
fn constructor_capture_failure_is_terminal_before_model_start() {
    let fixture = package(false, "solve");
    let output = Output::new();
    let mut app = app(&output);
    let mut set = set(&fixture);
    // A valid frozen byte cap can be smaller than the actual constructor row.
    for member in &mut set.members {
        let mut future = member.spec.future.clone();
        future.recording.max_record_bytes = 1;
        member.spec = FrozenConfiguredSpec::new(member.spec.constructor.clone(), future).unwrap();
    }
    set.members[1].differences_from_baseline =
        ConfiguredSpecDifference::between(&set.members[0].spec, &set.members[1].spec).unwrap();
    let root = output.0.join("small.runset");
    launch(&mut app, set, &root);
    let saved = finish(&mut app);
    let reader = member(&root, &saved, 0);
    assert_eq!(reader.count().unwrap(), 0);
    let audit = audit_runtime_termination(&reader).unwrap();
    assert_eq!(
        audit.termination.cause,
        RuntimeTerminationCause::StorageFailure
    );
    assert_eq!(audit.termination.accepted_generation, 0);
    assert!(matches!(
        audit.coverage,
        RuntimeTerminalCoverage::NoRecordedPublication
    ));
    assert!(audit.termination.observation.is_none());
    let mut page = crate::saved_comparison::SavedComparisonPage::default();
    page.set_location(
        root.to_string_lossy(),
        Path::new(&app.run_sets.path)
            .file_name()
            .unwrap()
            .to_string_lossy(),
    );
    page.load_manifest();
    let summary = &page.member_summaries()[0];
    assert_eq!(summary.member_id, "native");
    assert_eq!(
        summary.availability,
        crate::saved_comparison::Availability::Ready
    );
    assert_eq!(summary.recorded_count, Some(0));
    assert_eq!(summary.recorded_generation, None);
    assert_eq!(summary.accepted_generation, Some(0));
    assert_eq!(summary.unrecorded_tail, Some(false));
    assert!(summary.detail.contains("unrecorded accepted tail absent"));
    let mut cold = archive_view::ArchiveView::default();
    cold.path = root
        .join(
            &saved.members[0]
                .attempt
                .as_ref()
                .unwrap()
                .archive_relative_path,
        )
        .to_string_lossy()
        .into();
    cold.open(saved.members[0].spec.future.recording.limits())
        .unwrap();
    assert!(cold.active);
    assert!(cold.inspector.selected_identity().is_none());
    cold.inspector.verify().unwrap();
    let ctx = egui::Context::default();
    let render = || {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1400.0, 1500.0),
                )),
                ..Default::default()
            },
            |ctx| cold.show(ctx),
        )
    };
    let mut render = render;
    let _ = render();
    let output = render();
    let text = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Text(text) => Some(text.galley.job.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(text.contains("StorageFailure"), "{text}");
    assert!(
        text.contains("No committed numerical publications"),
        "{text}"
    );
    assert!(!text.contains("Copy selected field sample CSV"), "{text}");
    assert!(!fixture.root.join("requests.jsonl").exists());
    assert!(matches!(
        archive::open_member(&root, &saved.members[1]),
        MemberState::NotRun
    ));
}
#[test]
fn actual_runtime_failure_terminal_is_not_relabelled_by_late_worker_cancel() {
    let fixture = package(false, "exit_once");
    let output = Output::new();
    let mut app = app(&output);
    let root = output.0.join("fault.runset");
    // Publish real immutable intent, then use the production Worker with one
    // deterministic post-failure barrier before its typed terminal write.
    let revision = set(&fixture);
    let baseline = &revision.members[0].spec;
    std::fs::create_dir(&root).unwrap();
    revision.save_new(&root.join("intent.json")).unwrap();
    let (reached, at_failure) = std::sync::mpsc::sync_channel(0);
    let (release, resume) = std::sync::mpsc::sync_channel(0);
    let request = Request {
        force_fields: Vec::new(),
        collision_mesh: None,
        size: 2,
        steps: 3,
        tight: false,
        threads: 1,
        methods: vec![PressureImplementation::JacobiPcgV1; 2],
        emitter: None,
        observe: false,
        history_budget_mib: 16,
        preview_resolution: history::PreviewResolution::Adaptive,
        domain_side: 0.5,
        requested_dt: 1.0 / 16.0,
        material: None,
        material_color: [0; 3],
        recording_limits: baseline.future.recording.limits(),
        recording_directory: root.to_string_lossy().into(),
        saved_set: None,
        configured_set: Some(ConfiguredRequest {
            revision,
            project_root: fixture.root.clone(),
            runtime: None,
        }),
        prior_run_ids: vec![],
        voxel: None,
        manifest_receipt: Some(Arc::new(Mutex::new(root.join("intent.json")))),
        progress_gate: None,
        failure_gate: Some(Arc::new(ProgressGate {
            reached,
            release: Mutex::new(resume),
        })),
    };
    app.launch_request(request).unwrap();
    at_failure.recv_timeout(Duration::from_secs(20)).unwrap();
    app.cancel();
    release.send(()).unwrap();
    let saved = finish(&mut app);
    let reader = member(&root, &saved, 1);
    let audit = audit_runtime_termination(&reader).unwrap();
    assert_eq!(reader.status().unwrap().outcome, "failed");
    assert_eq!(
        audit.termination.cause,
        RuntimeTerminationCause::RuntimeFailure
    );
    assert_eq!(audit.termination.accepted_generation, 2);
    assert!(matches!(
        audit.termination.observation.unwrap().outcome,
        RuntimeOutcome::NoCandidate {
            reason: diagnostic::NoCandidateReason::RuntimeFailure,
            dispatched: true,
            ..
        }
    ));
}
#[test]
fn changed_host_refuses_configured_rerun_before_output_or_runtime() {
    let fixture = package(false, "solve");
    let output = Output::new();
    let mut app = app(&output);
    let mut set = set(&fixture);
    for member in &mut set.members {
        let mut future = member.spec.future.clone();
        future.executor.source_revision.push_str("-different");
        member.spec = FrozenConfiguredSpec::new(member.spec.constructor.clone(), future).unwrap();
    }
    set.members[1].differences_from_baseline =
        ConfiguredSpecDifference::between(&set.members[0].spec, &set.members[1].spec).unwrap();
    let before = std::fs::read_dir(&output.0).unwrap().count();
    assert!(app.rerun_configured(set).is_err());
    assert_eq!(std::fs::read_dir(&output.0).unwrap().count(), before);
    assert!(app.worker.is_none());
    assert!(!fixture.root.join("requests.jsonl").exists());
}

#[test]
fn consumed_request_worker_cancel_retains_native_seed_and_unaccepted_runtime_terminal() {
    let fixture = package(false, "stall_once");
    let output = Output::new();
    let mut app = app(&output);
    let root = output.0.join("cancel.runset");
    launch(&mut app, set(&fixture), &root);
    wait(|| fixture.root.join("first-request-observed").exists());
    app.cancel();
    let saved = finish(&mut app);
    let reader = member(&root, &saved, 1);
    assert_eq!(reader.count().unwrap(), 2);
    let audit = audit_runtime_termination(&reader).unwrap();
    assert_eq!(audit.termination.cause, RuntimeTerminationCause::Cancelled);
    assert_eq!(audit.termination.accepted_generation, 1);
    assert_eq!(
        audit.termination.accepted_time.to_bits(),
        (1.0_f64 / 16.0).to_bits()
    );
    let observation = audit.termination.observation.unwrap();
    assert_eq!(observation.request.context.generation, 2);
    assert!(matches!(
        observation.outcome,
        RuntimeOutcome::NoCandidate {
            reason: diagnostic::NoCandidateReason::Cancelled,
            dispatched: true,
            ..
        }
    ));
    let native =
        RecordedConfiguredSession::decode(reader.run(), &reader.select(1).unwrap()).unwrap();
    assert!(native.runtime.is_none());
    assert_eq!(
        native.receipt.unwrap().selected,
        RecordedPressureSelection::Native(PressureImplementation::JacobiPcgV1)
    );
    let hidden = fixture.manifest.with_extension("hidden");
    std::fs::rename(&fixture.manifest, &hidden).unwrap();
    cold(Path::new(&app.run_sets.path), 1, &reader, &fixture.manifest);
    std::fs::rename(hidden, &fixture.manifest).unwrap();
}
