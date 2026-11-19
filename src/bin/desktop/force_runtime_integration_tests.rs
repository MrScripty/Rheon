//! Prepared numerical/SQLite probes. Ignored; never execute without separate
//! bounded admission. No cleanup guard: all artifacts survive success or panic.
use super::*;
use rheon::recording::{RecordedCarrier, RecordedMaterial, RecordingReader};
use rheon::{
    ForceKind, ForceObject, ForceScene, ForceSnapshot, ForceSupport, ForceTime, ForceTransform,
    ForceUnits, SceneObjectId, SnapshotStage,
};
use std::path::{Path, PathBuf};

fn fields() -> ForceSnapshot {
    ForceScene {
        version: 1,
        objects: vec![ForceObject {
            id: SceneObjectId(1),
            name: "Desktop custody probe".into(),
            enabled: true,
            transform: ForceTransform {
                translation: [0.5; 3],
                ..Default::default()
            },
            units: ForceUnits::Acceleration,
            kind: ForceKind::Directional {
                vector: [0.25, 0.0, 0.0],
            },
            support: ForceSupport::Global,
            time: ForceTime::default(),
        }],
    }
    .freeze()
    .unwrap()
}
fn bytes(fields: &ForceSnapshot) -> Vec<u8> {
    rheon::recording::encode_force_metadata(fields).unwrap()
}
fn output_root() -> PathBuf {
    let root = PathBuf::from(
        std::env::var_os("RHEON_UI_FORCE_TEST_ROOT")
            .expect("supervisor must supply one fresh exclusive root; no temp fallback"),
    );
    assert!(root.is_absolute());
    let metadata = std::fs::symlink_metadata(&root).unwrap();
    assert!(metadata.is_dir() && !metadata.file_type().is_symlink());
    assert_eq!(root.canonicalize().unwrap(), root);
    // SQLite temp_store is FILE in the canonical owner. Keep any transient
    // files inside this same disposable data root, distinct from supervisor logs.
    for key in ["TMPDIR", "SQLITE_TMPDIR"] {
        assert_eq!(std::env::var_os(key), Some(root.as_os_str().to_os_string()));
    }
    assert!(
        std::fs::read_dir(&root).unwrap().next().is_none(),
        "root must be fresh"
    );
    eprintln!("retained artifact root: {}", root.display());
    root
}
fn request(root: &Path, fields: &ForceSnapshot) -> Request {
    Request {
        force_fields: vec![Some(fields.clone())],
        collision_mesh: None,
        size: 2,
        steps: 1,
        tight: false,
        threads: 1,
        methods: vec![PressureImplementation::JacobiPcgV1],
        emitter: None,
        observe: false,
        history_budget_mib: 16,
        preview_resolution: history::PreviewResolution::Edge(2),
        domain_side: 1.0,
        requested_dt: 1e-6,
        material: None,
        material_color: [0; 3],
        recording_limits: rheon::recording::RecordingLimits {
            max_database_bytes: 1024 * 1024,
            max_record_bytes: 65536,
            max_records: 2,
        },
        recording_directory: root.to_str().unwrap().into(),
        saved_set: None,
        configured_set: None,
        prior_run_ids: vec![],
        voxel: None,
        manifest_receipt: None,
        progress_gate: None,
        failure_gate: None,
    }
}
fn worker(request: Request) -> (Arc<LiveHistory>, Vec<ResultRow>) {
    // Join before any outcome assertions. No App::Drop, cancellation, second
    // scheduler or detached work owner. External supervisor bounds the wait.
    let Worker {
        history,
        progress,
        handle,
        ..
    } = Worker::start(request).unwrap();
    let joined = handle.join();
    let result = joined.expect("worker panic; artifacts retained");
    let rows = result.expect("worker error; artifacts retained");
    assert_eq!(progress.load(Ordering::Relaxed), 1);
    assert_eq!(rows.len(), 1);
    (history, rows)
}
fn readback(path: &Path, intended: &Request) -> RecordingReader {
    let reader = RecordingReader::open(path, intended.recording_limits).unwrap();
    reader.verify().unwrap();
    assert_eq!(reader.count().unwrap(), 2);
    let status = reader.status().unwrap();
    assert_eq!(status.outcome, "completed");
    assert_eq!(status.accepted_generation, Some(1));
    let fields = intended.fields_for_track(0).unwrap();
    inspector::verify_force_fields(reader.run(), Some(fields)).unwrap();
    for ordinal in 0..2 {
        let step = reader.select(ordinal).unwrap();
        assert_eq!(step.generation, ordinal as u64);
        assert_eq!(
            step.stage,
            if ordinal == 0 {
                SnapshotStage::Constructor
            } else {
                SnapshotStage::Accepted
            }
        );
        let actual = rheon::recording::recorded_force_fields(&step)
            .unwrap()
            .unwrap();
        assert_eq!(bytes(&actual), bytes(fields));
    }
    reader
}
fn inventory(root: &Path) {
    use sha2::{Digest, Sha256};
    let mut total = 0_u64;
    let mut files = 0;
    for entry in std::fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        let metadata = std::fs::symlink_metadata(&path).unwrap();
        assert!(metadata.is_file() && !metadata.file_type().is_symlink());
        assert!(metadata.len() <= 1024 * 1024);
        total += metadata.len();
        files += 1;
        let data = std::fs::read(&path).unwrap();
        eprintln!(
            "artifact={} bytes={} sha256={:x}",
            path.display(),
            data.len(),
            Sha256::digest(data)
        );
    }
    assert!(files <= 8 && total <= 8 * 1024 * 1024);
    eprintln!("retained files={files} bytes={total}; no cleanup performed");
}
#[test]
#[ignore = "numerical/SQLite worker integration; separate bounded execution admission required"]
fn native_force_dispatch_records_and_cold_binds_v2() {
    let root = output_root();
    let fields = fields();
    let mut request = request(&root, &fields);
    let native = run_sets::NativeCarrierSpec {
        counts_xyz: [2; 3],
        spacing_m: [0.5; 3],
        origin_m: [0.0; 3],
        density_kg_m3: 2.0,
        pressure_method: run_sets::NativePressureMethod::JacobiPcgV1,
        tolerances: run_sets::NativeTolerances {
            relative_residual: 1e-11,
            absolute_residual_m3_s2: 1e-12,
            predicted_divergence_per_s: 1e-9,
            actual_divergence_per_s: 1e-6,
            max_courant: 1.0,
            max_pressure_iterations: 100,
        },
        requested_dt_s: 1e-6,
        requested_steps: 1,
        source: None,
        simulation_memory_limit_bytes: 1024 * 1024,
        recording: run_sets::ArchiveBudget {
            max_database_bytes: 1024 * 1024,
            max_record_bytes: 65536,
            max_records: 2,
        },
    };
    let frozen = run_sets::FrozenSpec::new(native)
        .unwrap()
        .with_force_fields(&fields)
        .unwrap();
    let intent = run_sets::RunSetRevision {
        schema_version: 1,
        set_id: "desktop-force-probe".into(),
        parent_sha256: None,
        baseline_member: "native-0".into(),
        members: vec![run_sets::RunSetMember {
            member_id: "native-0".into(),
            spec: frozen.clone(),
            differences_from_baseline: vec![],
            attempt: None,
        }],
    };
    intent.save_new(&root.join("intent.json")).unwrap();
    request.saved_set = Some(intent);
    request.manifest_receipt = Some(Arc::new(std::sync::Mutex::new(root.join("intent.json"))));
    let retained = request.clone();
    let (history, rows) = worker(request);
    let finished = run_sets::RunSetRevision::open(&root.join("member-0-finished.json")).unwrap();
    let member = &finished.members[0];
    assert_eq!(member.spec.sha256, frozen.sha256);
    let run_sets::MemberState::Ready(cold) = run_sets::open_member(&root, member) else {
        panic!("cold immutable binding failed; preserve all artifacts")
    };
    let reader = readback(
        &root.join(&member.attempt.as_ref().unwrap().archive_relative_path),
        &retained,
    );
    assert_eq!(cold.run(), reader.run());
    let initial = RecordedCarrier::decode(reader.run(), &reader.select(0).unwrap()).unwrap();
    let raw = reader.select(1).unwrap();
    let accepted = RecordedCarrier::decode(reader.run(), &raw).unwrap();
    assert!(initial.interval_pressure.is_none());
    assert!(
        initial
            .x
            .iter()
            .chain(&initial.y)
            .chain(&initial.z)
            .all(|v| *v == 0.0)
    );
    let (start, pressure) = accepted.interval_pressure.as_ref().unwrap();
    assert_eq!(*start, 0.0);
    // A uniform X acceleration in this closed box produces a nonzero interval
    // pressure gradient. Metadata alone cannot satisfy this dispatch-effect guard.
    assert_eq!(pressure.len(), 8);
    for (index, pressure) in pressure.iter().enumerate() {
        // Hydrostatic dp/dx=rho*a:2kg/m³×0.25m/s²×0.5m=0.25Pa.
        // Predeclared coarse20% guard distinguishes an applied field from zero;
        // it is not a pressure-accuracy validation or tuned after execution.
        let expected = (index % 2) as f64 * 0.25;
        assert!(pressure.is_finite() && (pressure - expected).abs() <= 0.05);
    }
    assert!(raw.controls.contains("external-force-metadata-v1"));
    assert_eq!(accepted.dt, Some(retained.requested_dt));
    let (count, preview) = history.view(0, 1, false).unwrap();
    assert_eq!(count, 2);
    assert_eq!(preview.unwrap().time.to_bits(), accepted.time.to_bits());
    assert_eq!(rows[0].time.to_bits(), accepted.time.to_bits());
    // Prepare constructor-zero rerun intent from the cold v2 member, without
    // executing a second step/worker or consulting the mutable authoring draft.
    // It still has old output/receipt paths: this is NOT an executable rerun. A
    // production rerun must allocate fresh paths through the App owner.
    let mut rerun = retained.clone();
    let mut next = finished.clone();
    next.parent_sha256 = Some(finished.sha256().unwrap());
    next.set_id = "prepared-rerun".into();
    next.members[0].attempt = None;
    rerun.force_fields = vec![next.members[0].spec.force_snapshot().unwrap()];
    rerun.saved_set = Some(next);
    validate_request(&rerun).unwrap();
    assert_eq!(bytes(rerun.fields_for_track(0).unwrap()), bytes(&fields));
    let mut changed_draft = fields.scene().clone();
    changed_draft.objects[0].name = "Later draft".into();
    assert_ne!(
        bytes(&changed_draft.freeze().unwrap()),
        bytes(rerun.fields_for_track(0).unwrap())
    );
    // Re-read both archive records AFTER mutating a separate authoring draft.
    for ordinal in 0..2 {
        let step = reader.select(ordinal).unwrap();
        assert_eq!(
            bytes(
                &rheon::recording::recorded_force_fields(&step)
                    .unwrap()
                    .unwrap()
            ),
            bytes(&fields)
        );
    }
    inventory(&root);
}
#[test]
#[ignore = "numerical/SQLite worker integration; separate bounded execution admission required"]
fn material_force_dispatch_records_immutable_scene() {
    let root = output_root();
    let fields = fields();
    let mut request = request(&root, &fields);
    request.material = Some(rheon::ParticleFluidConfig {
        fill: rheon::DomainFill::EmittedFluid,
        samples_per_axis: 4,
        max_particles: 64,
        memory_limit: 1024 * 1024,
        gravity: [0.0, -1.0, 0.0],
        ..Default::default()
    });
    let retained = request.clone();
    let (history, rows) = worker(request);
    let path = history.inspector.archive_path().unwrap();
    let reader = readback(&path, &retained);
    assert!(
        reader
            .run()
            .configuration
            .starts_with(&format!("{:?}; collision=", retained.material.unwrap()))
    );
    assert_eq!(reader.run().counts, [2; 3]);
    assert_eq!(reader.run().spacing, [0.5; 3]);
    assert_eq!(reader.run().origin, [0.0; 3]);
    let initial = RecordedMaterial::decode(&reader.select(0).unwrap()).unwrap();
    let raw = reader.select(1).unwrap();
    let accepted = RecordedMaterial::decode(&raw).unwrap();
    assert_eq!(initial.particles.len(), 64);
    assert_eq!(accepted.particles.len(), 64);
    assert!(initial.particles.iter().all(|p| p.velocity == [0.0; 3]));
    let initial_momentum: f64 = initial
        .particles
        .iter()
        .map(|p| p.mass * p.velocity[0])
        .sum();
    let momentum: f64 = accepted
        .particles
        .iter()
        .map(|p| p.mass * p.velocity[0])
        .sum();
    // Symmetric pair forces and vertical gravity cannot supply positive net X
    // momentum here. A document-only path does not satisfy this effect guard.
    let dt = accepted.dt.unwrap();
    assert!(dt.is_finite() && dt > 0.0 && dt <= retained.requested_dt);
    let mass: f64 = initial.particles.iter().map(|p| p.mass).sum();
    let expected_momentum = mass * 0.25 * dt;
    let change = momentum - initial_momentum;
    // Predeclared50% coarse guard excludes tiny positive pair-force roundoff.
    assert!(mass.is_finite() && mass > 0.0 && change.is_finite());
    assert!((0.5 * expected_momentum..=1.5 * expected_momentum).contains(&change));
    assert!(raw.controls.contains("external-force-metadata-v1"));
    assert_eq!(accepted.time.to_bits(), dt.to_bits());
    let (_, diagnostics) = accepted.diagnostics.as_ref().unwrap();
    assert_eq!(diagnostics.len(), 64);
    let (count, preview) = history.view(0, 1, false).unwrap();
    assert_eq!(count, 2);
    assert_eq!(preview.unwrap().time.to_bits(), accepted.time.to_bits());
    assert_eq!(rows[0].particle_report.unwrap().dt.to_bits(), dt.to_bits());
    // Existing material workflows retain Request in memory, not a serialized
    // material FrozenSpec. Check that retained typed intent can rebind the cold
    // recorded scene; this does NOT claim a production cold material-rerun API.
    let mut rerun = retained.clone();
    rerun.force_fields = vec![accepted.force_fields.clone()];
    validate_request(&rerun).unwrap();
    assert_eq!(
        format!("{:?}", rerun.material),
        format!("{:?}", retained.material)
    );
    assert_eq!(bytes(rerun.fields_for_track(0).unwrap()), bytes(&fields));
    let mut changed_draft = fields.scene().clone();
    changed_draft.objects[0].enabled = false;
    assert_ne!(
        bytes(&changed_draft.freeze().unwrap()),
        bytes(rerun.fields_for_track(0).unwrap())
    );
    // Re-read both archive records AFTER mutating a separate authoring draft.
    for ordinal in 0..2 {
        let step = reader.select(ordinal).unwrap();
        assert_eq!(
            bytes(
                &rheon::recording::recorded_force_fields(&step)
                    .unwrap()
                    .unwrap()
            ),
            bytes(&fields)
        );
    }
    inventory(&root);
}
