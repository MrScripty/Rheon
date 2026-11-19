//! Real existing-Worker/SQLite checks; no native-window or accuracy claim.
use super::*;
use rheon::{SceneObjectId, initial_contents::*, recording::*};
use run_sets::voxel_material::{ExecutionPolicy, FrozenInput, PressurePolicy, run_set::*};
use std::{
    path::{Path, PathBuf},
    sync::{Mutex, mpsc},
};
pub(super) fn root() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let p = PathBuf::from(".generated/voxel-material/m4-worker").join(format!(
        "{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&p).unwrap();
    p
}
pub(super) fn recipe(
    dt: f64,
    steps: usize,
    gravity: [f64; 3],
    database: u64,
) -> (ContentsEditor, FrozenInput) {
    let grid = GridGeometry::new([4; 3], [0.25; 3], [-0.0; 3]).unwrap();
    // Lower-left liquid column rather than an equilibrium horizontal layer.
    let ids = (0..64)
        .map(|i| if i % 4 < 2 && i / 16 < 2 { 1 } else { 2 })
        .collect::<Vec<_>>();
    let e = ContentsEditor::new(
        InitialContents::restore(
            SceneObjectId(91),
            3,
            grid,
            rheon::voxel_material::MaterialPair::water_air(),
            &ids,
        )
        .unwrap(),
    );
    let spec = FrozenInput::capture(
        &e,
        ExecutionPolicy {
            requested_dt_s: dt,
            requested_steps: steps,
            gravity_m_s2: gravity,
            pressure: PressurePolicy {
                relative_residual: 1e-11,
                absolute_residual: 1e-12,
                divergence_limit: 1e-8,
                max_iterations: 2000,
            },
            numerical_memory_limit_bytes: 8 * 1024 * 1024,
            recording: run_sets::ArchiveBudget {
                max_database_bytes: database,
                max_record_bytes: 128 * 1024,
                max_records: steps + 1,
            },
            executor: voxel_material_execution::executor(),
        },
    )
    .unwrap();
    (e, spec)
}
pub(super) fn request(spec: FrozenInput, output: &Path) -> Request {
    let p = spec.execution();
    Request {
        force_fields: vec![],
        collision_mesh: None,
        size: 4,
        steps: p.requested_steps,
        tight: false,
        threads: 1,
        methods: vec![PressureImplementation::JacobiPcgV1],
        emitter: None,
        observe: false,
        history_budget_mib: 16,
        preview_resolution: history::PreviewResolution::Edge(4),
        domain_side: 1.0,
        requested_dt: p.requested_dt_s,
        material: None,
        material_color: [45, 125, 215],
        recording_limits: p.recording.limits(),
        recording_directory: output.to_string_lossy().into_owned(),
        saved_set: None,
        configured_set: None,
        prior_run_ids: vec![1],
        voxel: Some(voxel_material_execution::VoxelRequest {
            revision: Arc::new(VoxelRunSetRevision {
                schema_version: 3,
                kind: VoxelSetKind::VoxelMaterialSet,
                set_id: "worker-water-air".into(),
                parent_sha256: None,
                baseline_member: "voxel".into(),
                members: vec![VoxelRunSetMember {
                    member_id: "voxel".into(),
                    spec,
                    attempt: None,
                }],
            }),
            retained_payload_bytes: 0,
        }),
        manifest_receipt: Some(Arc::new(Mutex::new(PathBuf::new()))),
        progress_gate: None,
        failure_gate: None,
    }
}
fn finished(receipt: &Arc<Mutex<PathBuf>>) -> (PathBuf, VoxelRunSetRevision, RecordingReader) {
    let p = receipt.lock().unwrap().clone();
    assert_eq!(p.file_name().unwrap(), "finished.json");
    let set = VoxelRunSetRevision::open(&p).unwrap();
    let run_sets::MemberState::Ready(reader) = open_member(p.parent().unwrap(), &set.members[0])
    else {
        panic!("Expected pinned actual worker archive")
    };
    (p, set, *reader)
}
#[test]
fn existing_worker_records_real_voxel_fields_and_captures_only_immutable_intent() {
    let root = root();
    let (mut editor, spec) = recipe(0.001, 3, [0.0, -1.0, 0.0], 1024 * 1024);
    let input = spec.digest();
    let r = request(spec.clone(), &root);
    let receipt = Arc::clone(r.manifest_receipt.as_ref().unwrap());
    editor.fill_air().unwrap();
    assert_eq!(editor.document().counts(), [0, 0, 64]);
    let worker = Worker::start(r).unwrap();
    let history = Arc::clone(&worker.history);
    let rows = worker.handle.join().unwrap().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].label, voxel_material_execution::METHOD);
    assert_eq!(rows[0].pixels.len(), 16);
    assert_eq!(rows[0].time.to_bits(), 0.003_f64.to_bits());
    let (_, set, reader) = finished(&receipt);
    assert_eq!(set.members[0].spec, spec);
    assert_ne!(reader.run().identity.run_id, 1);
    assert_eq!(reader.count().unwrap(), 4);
    assert_eq!(reader.status().unwrap().outcome, "completed");
    let mut expected = spec.construct().unwrap();
    for i in 0..4 {
        if i > 0 {
            expected.step(0.001, |_| false).unwrap();
        }
        let step = reader.select(i).unwrap();
        let actual = RecordedVoxelMaterial::decode(reader.run(), &step).unwrap();
        assert_eq!(actual.metadata.initial_input_sha256, input);
        assert_eq!(
            actual.grid.origin().map(f64::to_bits),
            [-0.0_f64; 3].map(f64::to_bits)
        );
        assert_eq!(actual.liquid_fraction, expected.state().liquid_fraction());
        for (d, a) in [rheon::Axis::X, rheon::Axis::Y, rheon::Axis::Z]
            .into_iter()
            .enumerate()
        {
            assert_eq!(actual.velocity[d], expected.state().velocity(a));
        }
        let gas_mass = actual.liquid_fraction.iter().map(|f| 1.0 - f).sum::<f64>()
            * actual.grid.cell_volume()
            * actual.metadata.materials.gas().density();
        assert!(gas_mass > 0.0);
        let (_, frame) = history.view(0, i, false).unwrap();
        let frame = frame.unwrap();
        assert_eq!(frame.voxel.as_ref().unwrap().report.is_some(), i > 0);
        assert!(frame.report.is_none() && frame.particle_report.is_none());
        assert_eq!(frame.time.to_bits(), actual.time.to_bits());
    }
    history.carrier_inspectors[0].select_timestep(3).unwrap();
    assert_eq!(
        history.carrier_inspectors[0].selected_identity().unwrap().1,
        3
    );
    eprintln!("completed worker evidence {}", root.display());
}
#[test]
fn worker_cancellation_retains_acknowledged_prefix_and_pins_cancelled_attempt() {
    let root = root();
    let (_, spec) = recipe(0.001, 3, [0.0; 3], 1024 * 1024);
    let mut r = request(spec, &root);
    let receipt = Arc::clone(r.manifest_receipt.as_ref().unwrap());
    let (reached, rx) = mpsc::sync_channel(0);
    let (tx, release) = mpsc::sync_channel(0);
    r.progress_gate = Some(Arc::new(ProgressGate {
        reached,
        release: Mutex::new(release),
    }));
    let worker = Worker::start(r).unwrap();
    rx.recv_timeout(Duration::from_secs(10)).unwrap();
    worker.cancel();
    tx.send(()).unwrap();
    let history = Arc::clone(&worker.history);
    let error = worker.handle.join().unwrap().err().unwrap();
    assert!(error.contains("Cancelled"));
    let (_, _, reader) = finished(&receipt);
    assert_eq!(reader.count().unwrap(), 2);
    assert_eq!(reader.status().unwrap().outcome, "cancelled");
    assert_eq!(reader.status().unwrap().accepted_generation, Some(1));
    assert_eq!(history.view(0, 0, true).unwrap().0, 2);
}
#[test]
fn all_new_mode_refusals_precede_any_output_and_do_not_mutate_recipe() {
    let root = root().join("must-remain-absent");
    let (_, spec) = recipe(0.001, 1, [0.0; 3], 1024 * 1024);
    let base = request(spec, &root);
    let original = base.voxel.as_ref().unwrap().revision.members[0]
        .spec
        .digest();
    let mut r = base.clone();
    r.threads = 2;
    assert!(validate_request(&r).unwrap_err().contains("serial"));
    r = base.clone();
    r.observe = true;
    assert!(validate_request(&r).is_err());
    r = base.clone();
    r.methods.push(PressureImplementation::JacobiPcgV1);
    assert!(validate_request(&r).is_err());
    r = base.clone();
    r.voxel.as_mut().unwrap().retained_payload_bytes = 384 * 1024 * 1024;
    assert!(
        Worker::start(r)
            .err()
            .unwrap()
            .to_string()
            .contains("reserved payload")
    );
    r = base.clone();
    r.requested_dt = 0.002;
    assert!(validate_request(&r).unwrap_err().contains("immutable"));
    r = base.clone();
    r.domain_side = 2.0;
    assert!(validate_request(&r).unwrap_err().contains("domain"));
    r = base.clone();
    r.material = Some(Default::default());
    assert!(validate_request(&r).is_err());
    assert!(!root.exists());
    assert_eq!(
        base.voxel.as_ref().unwrap().revision.members[0]
            .spec
            .digest(),
        original
    );
}
#[test]
fn actual_capacity_and_shape_refusal_precedes_history_or_decoded_installation() {
    let (_, spec) = recipe(0.001, 1, [0.0; 3], 1024 * 1024);
    let sim = spec.construct().unwrap();
    let history = LiveHistory::with_voxel_resolution(4, 1, 16 * 1024 * 1024, true).unwrap();
    let mut bad = PreviewFrame::capture_voxel(&sim, 4, 0.0).unwrap();
    Arc::get_mut(&mut bad).unwrap().density.pop();
    assert!(history.publish(0, bad).unwrap_err().contains("shape"));
    let mut large = PreviewFrame::capture_voxel(&sim, 4, 0.0).unwrap();
    Arc::get_mut(&mut large)
        .unwrap()
        .density
        .reserve_exact(16 * 1024 * 1024);
    assert!(history.publish(0, large).unwrap_err().contains("capacity"));
    assert_eq!(history.view(0, 0, true).unwrap().0, 0);
    history
        .publish(0, PreviewFrame::capture_voxel(&sim, 4, 0.0).unwrap())
        .unwrap();
    let run = bound_run(
        &spec,
        &sim,
        rheon::SessionIdentity {
            run_id: 9,
            source_revision: spec.execution().executor.source_revision.clone(),
        },
    )
    .unwrap();
    let cap = voxel_material_publication_bytes(sim.state().geometry(), true).unwrap();
    let mut raw = voxel_material_publication(&sim, 0, cap).unwrap();
    raw.fields[0].data.reserve_exact(cap);
    assert!(matches!(
        RecordedVoxelMaterial::decode(&run, &raw),
        Err(RecordingError::Limit("voxel raw selection capacity"))
    ));
}
#[test]
fn actual_worker_accepts_ratio_shortened_intervals_and_keeps_original_schedule() {
    let root = root();
    let (_, spec) = recipe(1.0, 1, [0.0, -9.81, 0.0], 1024 * 1024);
    let r = request(spec, &root);
    let receipt = Arc::clone(r.manifest_receipt.as_ref().unwrap());
    let worker = Worker::start(r).unwrap();
    let history = Arc::clone(&worker.history);
    worker.handle.join().unwrap().unwrap();
    let (_, set, reader) = finished(&receipt);
    let step = reader.select(1).unwrap();
    assert!(step.dt.unwrap() > 0.0 && step.dt.unwrap() < 1.0);
    assert_eq!(set.members[0].spec.execution().requested_dt_s, 1.0);
    let (_, f) = history.view(0, 1, false).unwrap();
    let report = f.unwrap().voxel.as_ref().unwrap().report.unwrap();
    assert!(report.attempts > 1);
    assert_eq!(report.dt.to_bits(), step.dt.unwrap().to_bits());
}
#[test]
fn cold_process_reruns_original_intent_through_the_same_existing_worker() {
    let root = root();
    let (_, spec) = recipe(0.001, 2, [0.0, -1.0, 0.0], 1024 * 1024);
    let r = request(spec, &root);
    let receipt = Arc::clone(r.manifest_receipt.as_ref().unwrap());
    Worker::start(r).unwrap().handle.join().unwrap().unwrap();
    let (path, _, _) = finished(&receipt);
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "voxel_material_integration_tests::cold_worker_child",
            "--ignored",
            "--nocapture",
        ])
        .env("RHEON_VOXEL_COLD_WORKER_INPUT", &path)
        .status()
        .unwrap();
    assert!(status.success());
}
#[test]
#[ignore = "child entrypoint, exercised by cold_process_reruns_original_intent_through_the_same_existing_worker"]
fn cold_worker_child() {
    let path = PathBuf::from(std::env::var_os("RHEON_VOXEL_COLD_WORKER_INPUT").unwrap());
    let original = VoxelRunSetRevision::open(&path).unwrap();
    let run_sets::MemberState::Ready(before) =
        open_member(path.parent().unwrap(), &original.members[0])
    else {
        panic!("original not Ready")
    };
    let mut r = request(
        original.members[0].spec.clone(),
        &path.parent().unwrap().join("cold-rerun"),
    );
    r.voxel.as_mut().unwrap().revision = Arc::new(original.clone());
    let receipt = Arc::clone(r.manifest_receipt.as_ref().unwrap());
    Worker::start(r).unwrap().handle.join().unwrap().unwrap();
    let (_, after_set, after) = finished(&receipt);
    assert_eq!(original.members[0].spec, after_set.members[0].spec);
    assert_ne!(before.run().identity.run_id, after.run().identity.run_id);
    for i in 0..before.count().unwrap() {
        let a = before.select(i).unwrap();
        let b = after.select(i).unwrap();
        assert_eq!(a.fields, b.fields);
        assert_eq!(a.generation, b.generation);
        assert_eq!(a.time.to_bits(), b.time.to_bits());
        assert_eq!(a.dt.map(f64::to_bits), b.dt.map(f64::to_bits));
    }
}
#[test]
fn real_record_limit_failure_keeps_prefix_and_reports_accepted_unrecorded_tail() {
    let root = root();
    let (_, spec) = recipe(0.001, 100, [0.0; 3], 65536);
    let r = request(spec, &root);
    let receipt = Arc::clone(r.manifest_receipt.as_ref().unwrap());
    let worker = Worker::start(r).unwrap();
    let history = Arc::clone(&worker.history);
    let error = worker.handle.join().unwrap().err().unwrap();
    assert!(error.contains("append not confirmed"), "{error}");
    let path = receipt.lock().unwrap().clone();
    let set = VoxelRunSetRevision::open(&path).unwrap();
    // A poisoned writer admits no further append. A separately confirmed failed
    // terminal/pin may be Ready for inspection; it is not completed execution.
    let archive = path.parent().unwrap().join("run.sqlite");
    let limits = set.members[0].spec.execution().recording.limits();
    let reader = RecordingReader::open(&archive, limits).unwrap();
    let count = reader.count().unwrap();
    assert!((1..101).contains(&count));
    reader.verify().unwrap();
    assert_eq!(history.view(0, 0, true).unwrap().0, count);
    assert_eq!(reader.status().unwrap().outcome, "failed");
    assert_eq!(
        reader.status().unwrap().accepted_generation,
        Some(count as u64)
    );
    assert!(!error.contains("terminal status not confirmed"));
    assert_eq!(path.file_name().unwrap(), "finished.json");
    assert!(set.members[0].attempt.as_ref().unwrap().finalized.is_some());
    eprintln!("bounded record-limit evidence {}: {error}", root.display());
}
#[test]
fn running_setup_identifies_actual_voxel_backend() {
    let root = root();
    let (_, spec) = recipe(0.001, 1, [0.0; 3], 1024 * 1024);
    let request = request(spec, &root);
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(900.0, 650.0),
            )),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| running_setup_ui(ui, &request));
        },
    );
    let labels = output
        .shapes
        .iter()
        .filter_map(|s| match &s.shape {
            egui::Shape::Text(t) => Some(t.galley.text().to_owned()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        labels.contains(voxel_material_execution::METHOD),
        "{labels}"
    );
    assert!(labels.contains("Filled incompressible Water/Air"));
    assert!(
        !labels.contains("Filled carrier-fluid")
            && !labels.contains("Liquid / gas material preview")
    );
    assert_eq!(request.track_label(0), voxel_material_execution::METHOD);
}
#[test]
fn captured_geometry_bits_survive_legacy_scalar_presentation_and_executor_mismatch_refuses() {
    let root = root();
    let (_, base) = recipe(0.001, 1, [0.0; 3], 1024 * 1024);
    let e = ContentsEditor::new(
        InitialContents::new(
            SceneObjectId(17),
            GridGeometry::new([6; 3], [0.1; 3], [-0.0; 3]).unwrap(),
            ContentsMaterial::Air,
        )
        .unwrap(),
    );
    let mut policy = base.execution().clone();
    policy.executor.source_revision.push_str("-different");
    let mut r = request(
        FrozenInput::capture(&e, policy).unwrap(),
        &root.join("refused"),
    );
    r.size = 6;
    r.domain_side = 0.1 * 6.0;
    r.preview_resolution = history::PreviewResolution::Adaptive;
    assert!(
        validate_request(&r)
            .unwrap_err()
            .contains("compiled executor")
    );
    assert!(!root.join("refused").exists());
    let spec = FrozenInput::capture(&e, base.execution().clone()).unwrap();
    let mut r = request(spec, &root.join("valid"));
    r.size = 6;
    r.domain_side = 0.1 * 6.0;
    r.preview_resolution = history::PreviewResolution::Adaptive;
    assert_ne!((r.domain_side / r.size as f64).to_bits(), 0.1_f64.to_bits());
    let receipt = Arc::clone(r.manifest_receipt.as_ref().unwrap());
    Worker::start(r).unwrap().handle.join().unwrap().unwrap();
    let (_, _, reader) = finished(&receipt);
    assert_eq!(
        reader.run().spacing.map(f64::to_bits),
        [0.1_f64; 3].map(f64::to_bits)
    );
    assert_eq!(
        reader.run().origin.map(f64::to_bits),
        [-0.0_f64; 3].map(f64::to_bits)
    );
    // Saved-spacing exception is explicit for this backend; independent exact
    // previews still refuse a requested density above their display policy.
    let small = ContentsEditor::new(
        InitialContents::new(
            SceneObjectId(18),
            GridGeometry::new([4; 3], [0.0003; 3], [0.0; 3]).unwrap(),
            ContentsMaterial::Air,
        )
        .unwrap(),
    );
    let mut r = request(
        FrozenInput::capture(&small, base.execution().clone()).unwrap(),
        &root.join("saved-small"),
    );
    r.domain_side = 0.0003 * 4.0;
    r.preview_resolution = history::PreviewResolution::Adaptive;
    validate_request(&r).unwrap();
    r.preview_resolution = history::PreviewResolution::Edge(4);
    assert!(validate_request(&r).unwrap_err().contains("1000/m"));
}
#[test]
fn final_pin_four_group_reservation_refuses_old_three_group_edge_before_output() {
    let output = root().join("four-group-refusal");
    let (_, spec) = recipe(0.001, 1, [0.0; 3], 1024 * 1024);
    let grid = spec.initial_editor().unwrap().document().geometry().clone();
    let publication = voxel_material_publication_bytes(&grid, true).unwrap();
    let remainder = (384 - 8 - 16 - 128) * 1024 * 1024;
    let mut r = request(spec, &output);
    r.voxel.as_mut().unwrap().retained_payload_bytes = remainder - 3 * publication;
    assert!(
        Worker::start(r.clone())
            .err()
            .unwrap()
            .to_string()
            .contains("reserved payload")
    );
    assert!(!output.exists());
    // Exact corrected admission edge, without allocating the declared old
    // payload. This is validation evidence, not production old-context proof.
    r.voxel.as_mut().unwrap().retained_payload_bytes -= publication;
    validate_request(&r).unwrap();
    assert!(!output.exists());
}
#[test]
fn voxel_appearance_has_known_phase_mask_resampling_and_accepted_mixed_bytes() {
    let (_, spec) = recipe(0.001, 1, [0.0; 3], 1024 * 1024);
    let sim = spec.construct().unwrap();
    let original = sim.state().liquid_fraction().to_vec();
    let geometry = sim.state().geometry().clone();
    // Original liquid occupies x<1/2,z<1/2, all y. Max coverage admits three
    // columns/layers at5 and two at3. These fixed planes are independent byte
    // oracles, not a second resampling implementation.
    let mut expected4 = [255, 255, 0, 0].repeat(8);
    expected4.extend([0; 32]);
    let mut expected5 = [255, 255, 255, 0, 0].repeat(15);
    expected5.extend([0; 50]);
    let mut expected3 = [255, 255, 0].repeat(6);
    expected3.extend([0; 9]);
    for (edge, expected) in [(4, expected4), (5, expected5), (3, expected3)] {
        let frame = PreviewFrame::capture_voxel(&sim, edge, 0.0).unwrap();
        assert_eq!(frame.density, expected);
        assert!(frame.voxel.is_some());
        assert!(frame.voxel.as_ref().unwrap().report.is_none());
    }
    assert_eq!(sim.state().liquid_fraction(), original);
    assert_eq!(sim.state().geometry(), &geometry);
    let mut f = [0.0; 64];
    f[21] = 0.25;
    f[42] = 0.5;
    let mut mixed = rheon::voxel_material::VoxelMaterialSimulation::new(
        geometry.clone(),
        rheon::voxel_material::MaterialPair::water_air(),
        &f,
        sim.config(),
    )
    .unwrap();
    mixed.step(0.001, |_| false).unwrap();
    let before = mixed.state().liquid_fraction().to_vec();
    let frame = PreviewFrame::capture_voxel(&mixed, 4, 0.0).unwrap();
    let mut expected = vec![0; 64];
    expected[21] = 64;
    expected[42] = 128;
    assert_eq!(frame.density, expected);
    assert_eq!(frame.nonzero_tracer_cells, 2);
    assert_eq!(frame.voxel.as_ref().unwrap().report.unwrap().generation, 1);
    assert_eq!(mixed.state().liquid_fraction(), before);
    assert_eq!(before, f);
    assert_eq!(mixed.state().geometry(), &geometry);
}
