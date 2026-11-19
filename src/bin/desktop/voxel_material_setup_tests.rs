//! Real App entry and cold saved constructor checks; no native-window claim.
use super::*;
use crate::{
    egui, voxel_material_authoring::DraftFileAction, voxel_material_integration_tests::root,
};
use rheon::{
    GridGeometry,
    initial_contents::*,
    recording::{RecordedVoxelMaterial, RecordingReader},
};
use std::{
    path::{Path, PathBuf},
    sync::atomic::Ordering,
    time::{Duration, Instant},
};
fn finish(app: &mut App) -> PathBuf {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !app.worker.as_ref().unwrap().handle.is_finished() {
        assert!(
            Instant::now() < deadline,
            "bounded App worker did not finish"
        );
        std::thread::yield_now();
    }
    app.poll(&egui::Context::default());
    assert!(app.status.starts_with("Complete"), "{}", app.status);
    app.running_request
        .as_ref()
        .unwrap()
        .manifest_receipt
        .as_ref()
        .unwrap()
        .lock()
        .unwrap()
        .clone()
}
fn dab(editor: &mut ContentsEditor, material: ContentsMaterial, operation: BrushOperation) {
    editor
        .begin(ContentsBrush {
            operation,
            material,
            radius: 0.14,
            axis: 2,
            layer: 1,
            depth: BrushDepth::Layer,
        })
        .unwrap();
    editor.segment([0.375; 3], [0.375; 3]).unwrap();
    assert!(editor.commit().unwrap());
}
fn app() -> App {
    let mut app = App::default();
    app.size = 4;
    app.steps = 2;
    app.requested_dt = 0.001;
    app.history_budget_mib = 16;
    app.preview_resolution = crate::history::PreviewResolution::Adaptive;
    app.recording_database_mib = 1;
    app.recording_record_mib = 1;
    app.recording_directory = root().join("app-entry").to_string_lossy().into();
    app.cylinder = false;
    app.materials.config.gravity = [0.0, -1.0, 0.0];
    app.scene
        .create_contents(GridGeometry::new([4; 3], [0.25; 3], [-0.0; 3]).unwrap())
        .unwrap();
    let editor = app.scene.contents.editor.as_mut().unwrap();
    editor.fill_air().unwrap();
    dab(editor, ContentsMaterial::Water, BrushOperation::Replace);
    app
}
fn reader(path: &Path) -> (VoxelRunSetRevision, Box<RecordingReader>) {
    let set = VoxelRunSetRevision::open(path).unwrap();
    let run_sets::MemberState::Ready(reader) =
        run_sets::voxel_material::run_set::open_member(path.parent().unwrap(), &set.members[0])
    else {
        panic!("closed voxel member not Ready")
    };
    (set, reader)
}
fn compare(before: &RecordingReader, after: &RecordingReader) {
    assert_ne!(before.run().identity.run_id, after.run().identity.run_id);
    assert_eq!(before.count().unwrap(), after.count().unwrap());
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
fn actual_app_save_load_capture_and_rerun_preserve_seed_and_charge_related_sources() {
    let mut app = app();
    let saved = root().join("painted-draft.json");
    let ids = app
        .scene
        .contents
        .editor
        .as_ref()
        .unwrap()
        .document()
        .cells()
        .to_vec();
    assert_eq!(ids.iter().filter(|&&id| id == 1).count(), 1);
    assert_eq!(ids[21], 1);
    app.draft_file_action(DraftFileAction::Save(saved.clone()))
        .unwrap();
    app.scene
        .contents
        .editor
        .as_mut()
        .unwrap()
        .fill_air()
        .unwrap();
    app.draft_file_action(DraftFileAction::Load(saved)).unwrap();
    assert_eq!(
        app.scene
            .contents
            .editor
            .as_ref()
            .unwrap()
            .document()
            .cells(),
        ids
    );
    let captured = app.capture_contents_request().unwrap();
    let frozen = captured.voxel.as_ref().unwrap().revision.members[0]
        .spec
        .clone();
    assert!(
        captured.material.is_none()
            && captured.emitter.is_none()
            && !captured.observe
            && captured.threads == 1
    );
    assert_eq!(frozen.execution().gravity_m_s2, [0.0, -1.0, 0.0]);
    app.scene.contents.run_requested = true;
    app.service_initial_contents_commands(&egui::Context::default());
    assert!(app.worker.is_some() && app.pending_voxel.is_some());
    let original_path = finish(&mut app);
    let original_bytes = std::fs::read(&original_path).unwrap();
    let (original, before) = reader(&original_path);
    assert_eq!(original.members[0].spec, frozen);
    let ctor = RecordedVoxelMaterial::decode(before.run(), &before.select(0).unwrap()).unwrap();
    let expected = (0..64)
        .map(|i| if i == 21 { 1.0 } else { 0.0 })
        .collect::<Vec<_>>();
    assert_eq!(ctor.liquid_fraction, expected);
    assert!(ctor.interval_pressure.is_none());
    assert!(ctor.velocity.iter().flatten().all(|v| v.to_bits() == 0));
    app.run_sets.path = original_path.to_string_lossy().into();
    app.run_sets.open().unwrap();
    app.run_sets.active = false;
    app.archive.path = original_path
        .parent()
        .unwrap()
        .join("run.sqlite")
        .to_string_lossy()
        .into();
    app.archive
        .open(frozen.execution().recording.limits())
        .unwrap();
    app.archive.active = false;
    let related = app.run_sets.related_voxel_payload_bytes(&original).unwrap()
        + app
            .archive
            .inspector
            .related_voxel_payload_bytes(frozen.digest())
            .unwrap();
    assert!(related > original.owned_capacity().unwrap());
    let retained = app.retained_payload_for_voxel().unwrap();
    app.scene.context = scene_ui::SceneContext::Draft;
    app.scene
        .contents
        .editor
        .as_mut()
        .unwrap()
        .fill_air()
        .unwrap();
    // Change current settings deliberately; rerun must consume original recipe.
    app.steps = 7;
    app.requested_dt = 0.25;
    app.size = 8;
    app.domain_side = 2.0;
    app.cylinder = true;
    app.materials.config.gravity = [9.0, 0.0, 0.0];
    app.recording_database_mib = 2;
    let retained_after = app.retained_payload_for_voxel().unwrap();
    assert!(retained_after >= retained);
    app.rerun_voxel(original.clone()).unwrap();
    assert_eq!(
        app.pending_voxel
            .as_ref()
            .unwrap()
            .request
            .voxel
            .as_ref()
            .unwrap()
            .retained_payload_bytes,
        retained_after + related
    );
    let rerun_path = finish(&mut app);
    let (rerun, after) = reader(&rerun_path);
    assert_eq!(original.members[0].spec, rerun.members[0].spec);
    compare(&before, &after);
    assert_eq!(std::fs::read(original_path).unwrap(), original_bytes);
    let intent =
        VoxelRunSetRevision::open(&rerun_path.parent().unwrap().join("intent.json")).unwrap();
    assert_eq!(intent.parent_sha256, Some(original.digest().unwrap()));
}
#[test]
fn capture_refuses_vacuum_active_stroke_and_selected_unsupported_inputs_without_output() {
    let mut app = app();
    let target = PathBuf::from(&app.recording_directory);
    app.cylinder = true;
    assert!(
        app.capture_contents_request()
            .err()
            .unwrap()
            .contains("emitter")
    );
    app.cylinder = false;
    app.setup_workflow = crate::SetupWorkflow::AcceptedSession;
    assert!(
        app.capture_contents_request()
            .err()
            .unwrap()
            .contains("configured")
    );
    app.setup_workflow = crate::SetupWorkflow::Preview;
    dab(
        app.scene.contents.editor.as_mut().unwrap(),
        ContentsMaterial::Water,
        BrushOperation::Remove,
    );
    assert!(app.start_contents().unwrap_err().contains("Vacuum"));
    let e = app.scene.contents.editor.as_mut().unwrap();
    e.undo().unwrap();
    e.begin(ContentsBrush {
        operation: BrushOperation::Replace,
        material: ContentsMaterial::Water,
        radius: 0.1,
        axis: 2,
        layer: 1,
        depth: BrushDepth::Layer,
    })
    .unwrap();
    assert!(app.start_contents().unwrap_err().contains("active"));
    assert!(app.worker.is_none() && !target.exists());
}
#[test]
fn capture_uses_original_document_spacing_instead_of_current_native_scalar_preferences() {
    let mut app = app();
    app.scene.context = scene_ui::SceneContext::Draft;
    let e = ContentsEditor::new(
        InitialContents::new(
            rheon::SceneObjectId(100),
            GridGeometry::new([6; 3], [0.1; 3], [-0.0; 3]).unwrap(),
            ContentsMaterial::Air,
        )
        .unwrap(),
    );
    app.scene.adopt_loaded_contents(e).unwrap();
    app.size = 32;
    app.domain_side = 4.0;
    let r = app.capture_contents_request().unwrap();
    assert_eq!(r.size, 6);
    assert_eq!(r.domain_side.to_bits(), (0.1_f64 * 6.0).to_bits());
    assert_ne!((r.domain_side / 6.0).to_bits(), 0.1_f64.to_bits());
    app.launch_request(r).unwrap();
    let path = finish(&mut app);
    let (_, r) = reader(&path);
    assert_eq!(
        r.run().spacing.map(f64::to_bits),
        [0.1_f64; 3].map(f64::to_bits)
    );
    assert_eq!(
        r.run().origin.map(f64::to_bits),
        [-0.0_f64; 3].map(f64::to_bits)
    );
}
#[test]
fn actual_app_cold_rerun_opens_original_recipe_in_a_fresh_process() {
    let mut app = app();
    app.start_contents().unwrap();
    let path = finish(&mut app);
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "voxel_material_setup::tests::cold_app_child",
            "--ignored",
            "--nocapture",
        ])
        .env("RHEON_VOXEL_COLD_APP_INPUT", &path)
        .status()
        .unwrap();
    assert!(status.success());
}
#[test]
#[ignore = "executed only by actual_app_cold_rerun_opens_original_recipe_in_a_fresh_process"]
fn cold_app_child() {
    let path = PathBuf::from(std::env::var_os("RHEON_VOXEL_COLD_APP_INPUT").unwrap());
    let (original, before) = reader(&path);
    let mut app = App::default();
    app.steps = 19;
    app.size = 64;
    app.requested_dt = 0.125;
    app.materials.config.gravity = [3.0, 2.0, 1.0];
    app.history_budget_mib = 16;
    app.recording_directory = path
        .parent()
        .unwrap()
        .join("fresh-app-rerun")
        .to_string_lossy()
        .into();
    assert!(app.scene.contents.editor.is_none());
    app.rerun_voxel(original.clone()).unwrap();
    let path = finish(&mut app);
    let (after_set, after) = reader(&path);
    assert_eq!(original.members[0].spec, after_set.members[0].spec);
    compare(&before, &after);
    assert_eq!(
        app.history
            .as_ref()
            .unwrap()
            .active_track
            .load(Ordering::Relaxed),
        0
    );
}
#[test]
fn new_capture_enforces_authoring_density_but_saved_adaptive_replay_keeps_original_bits() {
    let mut app = app();
    app.start_contents().unwrap();
    let source = finish(&mut app);
    let (mut saved, _) = reader(&source);
    let old_history = Arc::clone(app.history.as_ref().unwrap());
    let old_frozen = app
        .running_request
        .as_ref()
        .unwrap()
        .voxel
        .as_ref()
        .unwrap()
        .revision
        .members[0]
        .spec
        .digest();
    app.scene.context = scene_ui::SceneContext::Draft;
    let editor = ContentsEditor::new(
        InitialContents::new(
            rheon::SceneObjectId(100),
            GridGeometry::new([4; 3], [0.0003; 3], [0.0; 3]).unwrap(),
            ContentsMaterial::Air,
        )
        .unwrap(),
    );
    app.scene.adopt_loaded_contents(editor).unwrap();
    let before = run_sets::voxel_material::DraftRevision::capture(
        app.scene.contents.editor.as_ref().unwrap(),
    )
    .unwrap()
    .digest();
    let selected = app.scene.selected();
    let refused = root().join("density-refusal");
    app.recording_directory = refused.to_string_lossy().into();
    assert!(
        app.capture_contents_request()
            .err()
            .unwrap()
            .contains("1000 cells/m")
    );
    assert!(app.start_contents().unwrap_err().contains("1000 cells/m"));
    assert!(app.worker.is_none() && !refused.exists() && app.rows.len() == 1);
    assert!(Arc::ptr_eq(&old_history, app.history.as_ref().unwrap()));
    assert_eq!(
        app.running_request
            .as_ref()
            .unwrap()
            .voxel
            .as_ref()
            .unwrap()
            .revision
            .members[0]
            .spec
            .digest(),
        old_frozen
    );
    assert_eq!(app.scene.selected(), selected);
    assert_eq!(
        run_sets::voxel_material::DraftRevision::capture(
            app.scene.contents.editor.as_ref().unwrap()
        )
        .unwrap()
        .digest(),
        before
    );
    let mut policy = saved.members[0].spec.execution().clone();
    policy.requested_steps = 1;
    policy.requested_dt_s = 0.00001;
    policy.gravity_m_s2 = [0.0; 3];
    saved.members[0].spec =
        FrozenInput::capture(app.scene.contents.editor.as_ref().unwrap(), policy).unwrap();
    saved.members[0].attempt = None;
    app.rerun_voxel(saved).unwrap();
    let path = finish(&mut app);
    let (_, reader) = reader(&path);
    assert_eq!(
        reader.run().spacing.map(f64::to_bits),
        [0.0003_f64; 3].map(f64::to_bits)
    );
    assert_eq!(reader.run().counts, [4; 3]);
}
