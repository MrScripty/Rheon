//! Actual App draft actions and egui input; no solver/native-window claim.
use super::*;
use crate::{
    PressureImplementation,
    history::{LiveHistory, PreviewFrame},
    scene_ui::SceneUi,
};
use eframe::egui;
use rheon::{SceneObjectId, initial_contents::*};
use std::{
    fs,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
fn directory() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let p = PathBuf::from(".generated/voxel-material/m4-draft-ui").join(format!(
        "{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&p).unwrap();
    p
}
fn editor(id: u64, grid: GridGeometry) -> ContentsEditor {
    ContentsEditor::new(
        InitialContents::new(SceneObjectId(id), grid, ContentsMaterial::Air).unwrap(),
    )
}
fn grid() -> GridGeometry {
    GridGeometry::new([4; 3], [0.25; 3], [-0.0; 3]).unwrap()
}
fn paint(e: &mut ContentsEditor) {
    let p = e.document().geometry().cell_position([1; 3]).unwrap();
    e.begin(ContentsBrush {
        operation: BrushOperation::Replace,
        material: ContentsMaterial::Water,
        radius: 0.1,
        axis: 2,
        layer: 1,
        depth: BrushDepth::Layer,
    })
    .unwrap();
    e.segment(p, p).unwrap();
    e.commit().unwrap();
}
fn old_app() -> App {
    let mut app = App::default();
    app.size = 4;
    app.domain_side = 1.0;
    app.scene.create_contents(grid()).unwrap();
    app.scene
        .contents
        .editor
        .as_mut()
        .unwrap()
        .fill_air()
        .unwrap();
    app
}
fn digest(app: &App) -> [u8; 32] {
    DraftRevision::capture(app.scene.contents.editor.as_ref().unwrap())
        .unwrap()
        .digest()
}
#[test]
fn actual_app_save_preserves_redo_and_refuses_overwrite_without_mutating_bytes() {
    let path = directory().join("draft.json");
    let mut app = old_app();
    let e = app.scene.contents.editor.as_mut().unwrap();
    paint(e);
    e.undo().unwrap();
    let revision = e.document().revision();
    let redo = e.redo_label();
    app.draft_file_action(DraftFileAction::Save(path.clone()))
        .unwrap();
    let original = fs::read(&path).unwrap();
    let e = app.scene.contents.editor.as_ref().unwrap();
    assert_eq!(e.redo_label(), redo);
    assert_eq!(e.document().revision(), revision);
    app.scene.contents.editor.as_mut().unwrap().redo().unwrap();
    assert!(
        app.draft_file_action(DraftFileAction::Save(path.clone()))
            .unwrap_err()
            .contains("never replaced")
    );
    assert_eq!(fs::read(path).unwrap(), original);
}
#[test]
fn successful_load_adopts_exact_domain_and_fresh_history_preserving_retained_payload() {
    let path = directory().join("source.json");
    let g = GridGeometry::new([6; 3], [0.1; 3], [-0.0; 3]).unwrap();
    let mut e = editor(91, g.clone());
    paint(&mut e);
    let saved = DraftRevision::capture(&e).unwrap();
    saved.publish(&path).unwrap();
    let mut app = old_app();
    let history = LiveHistory::new(4, 1, 1).unwrap();
    history
        .publish(
            0,
            Arc::new(PreviewFrame {
                method: PressureImplementation::JacobiPcgV1,
                step: 0,
                time: 0.0,
                seconds: 0.0,
                edge: 4,
                nonzero_tracer_cells: 64,
                density: vec![17; 64],
                report: None,
                particle_report: None,
                voxel: None,
                material_color: None,
            }),
        )
        .unwrap();
    app.history = Some(Arc::clone(&history));
    app.timeline_step = 37;
    app.play_time = 7.0;
    app.follow_live = false;
    app.numerical_cells_per_metre = Some(50);
    let feedback = app.draft_file_action(DraftFileAction::Load(path)).unwrap();
    assert!(feedback.contains("undo history reset"));
    assert_eq!(app.size, 6);
    assert_eq!(app.domain_side.to_bits(), (0.1_f64 * 6.0).to_bits());
    assert_eq!(app.numerical_cells_per_metre, None);
    let e = app.scene.contents.editor.as_ref().unwrap();
    assert_eq!(DraftRevision::capture(e).unwrap(), saved);
    assert_eq!(
        e.document().geometry().origin().map(f64::to_bits),
        [-0.0_f64; 3].map(f64::to_bits)
    );
    assert!(e.undo_label().is_none() && e.redo_label().is_none());
    assert_eq!(app.scene.selected(), Some(SceneObjectId(91)));
    assert!(Arc::ptr_eq(app.history.as_ref().unwrap(), &history));
    assert_eq!(app.timeline_step, 37);
    assert_eq!(app.play_time, 7.0);
    assert!(!app.follow_live);
    assert_eq!(
        history.view(0, 0, false).unwrap().1.unwrap().density,
        vec![17; 64]
    );
    app.scene.replace_model(true).unwrap();
    assert_eq!(app.scene.model_id(), Some(SceneObjectId(92)));
}
#[test]
fn failed_loads_and_active_strokes_leave_draft_geometry_selection_and_redo_unchanged() {
    let dir = directory();
    let mut app = old_app();
    let e = app.scene.contents.editor.as_mut().unwrap();
    paint(e);
    e.undo().unwrap();
    let original = digest(&app);
    let selected = app.scene.selected();
    let redo = app.scene.contents.editor.as_ref().unwrap().redo_label();
    let invalid = dir.join("invalid.json");
    fs::write(&invalid, b"invalid").unwrap();
    assert!(
        app.draft_file_action(DraftFileAction::Load(invalid))
            .is_err()
    );
    let unsupported = dir.join("offset.json");
    DraftRevision::capture(&editor(
        21,
        GridGeometry::new([4; 3], [0.5; 3], [1.0; 3]).unwrap(),
    ))
    .unwrap()
    .publish(&unsupported)
    .unwrap();
    assert!(
        app.draft_file_action(DraftFileAction::Load(unsupported))
            .unwrap_err()
            .contains("no resampling")
    );
    let brush = ContentsBrush {
        operation: BrushOperation::Remove,
        material: ContentsMaterial::Water,
        radius: 0.1,
        axis: 2,
        layer: 1,
        depth: BrushDepth::Layer,
    };
    app.scene
        .contents
        .editor
        .as_mut()
        .unwrap()
        .begin(brush)
        .unwrap();
    let missing = dir.join("must-not-exist.json");
    assert!(
        app.draft_file_action(DraftFileAction::Save(missing.clone()))
            .is_err()
    );
    assert!(
        app.draft_file_action(DraftFileAction::Load(missing.clone()))
            .is_err()
    );
    assert!(!missing.exists());
    assert!(app.scene.contents.editor.as_ref().unwrap().active());
    app.scene.contents.editor.as_mut().unwrap().cancel();
    assert_eq!(digest(&app), original);
    assert_eq!(app.size, 4);
    assert_eq!(app.domain_side, 1.0);
    assert_eq!(app.scene.selected(), selected);
    assert_eq!(
        app.scene.contents.editor.as_ref().unwrap().redo_label(),
        redo
    );
}
#[test]
fn force_model_collisions_and_exhaustion_preserve_scene_identity_contract() {
    let dir = directory();
    let mut app = old_app();
    app.scene.replace_model(true).unwrap();
    let original = digest(&app);
    let collision = dir.join("collision.json");
    DraftRevision::capture(&editor(2, grid()))
        .unwrap()
        .publish(&collision)
        .unwrap();
    assert!(
        app.draft_file_action(DraftFileAction::Load(collision))
            .unwrap_err()
            .contains("conflicts")
    );
    assert_eq!(digest(&app), original);
    let force = rheon::ForceObject {
        id: SceneObjectId(22),
        name: "Unchanged force".into(),
        enabled: false,
        transform: Default::default(),
        units: rheon::ForceUnits::Acceleration,
        kind: rheon::ForceKind::Directional { vector: [0.0; 3] },
        support: rheon::ForceSupport::Global,
        time: Default::default(),
    };
    app.scene = SceneUi::geometry_fixture(
        rheon::ForceScene {
            version: 1,
            objects: vec![force.clone()],
        },
        false,
    );
    let collision = dir.join("force.json");
    DraftRevision::capture(&editor(22, grid()))
        .unwrap()
        .publish(&collision)
        .unwrap();
    assert!(
        app.draft_file_action(DraftFileAction::Load(collision))
            .unwrap_err()
            .contains("conflicts")
    );
    assert!(app.scene.contents.id().is_none());
    let max = dir.join("max.json");
    DraftRevision::capture(&editor(u64::MAX, grid()))
        .unwrap()
        .publish(&max)
        .unwrap();
    app.draft_file_action(DraftFileAction::Load(max)).unwrap();
    assert_eq!(app.scene.draft().objects, [force]);
    assert!(
        app.scene
            .replace_model(true)
            .unwrap_err()
            .contains("exhausted")
    );
    assert_eq!(app.scene.contents.id(), Some(SceneObjectId(u64::MAX)));
}
#[test]
fn uncertain_save_pauses_writes_and_load_remains_read_only_until_reconciliation() {
    let dir = directory();
    let source = dir.join("source.json");
    let mut app = old_app();
    app.draft_file_action(DraftFileAction::Save(source.clone()))
        .unwrap();
    let bytes = fs::read(&source).unwrap();
    // Test only reaction to an existing uncertain owner state; no filesystem
    // failure or successful durable reconciliation is fabricated here.
    app.scene.contents.uncertain_save = Some(source.clone());
    let target = dir.join("must-not-exist.json");
    assert!(
        app.draft_file_action(DraftFileAction::Save(target.clone()))
            .unwrap_err()
            .contains("uncertain")
    );
    assert!(!target.exists());
    app.draft_file_action(DraftFileAction::Load(source.clone()))
        .unwrap();
    assert_eq!(fs::read(source).unwrap(), bytes);
    assert!(app.scene.contents.uncertain_save.is_some());
}
fn frame(ctx: &egui::Context, app: &mut App, events: Vec<egui::Event>) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(650.0, 650.0),
            )),
            events,
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default()
                .show(ctx, |ui| app.scene.contents.file_controls(ui, true));
        },
    )
}
fn text_rect(output: &egui::FullOutput, label: &str) -> egui::Rect {
    output
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            egui::Shape::Text(t) if t.galley.text() == label => {
                Some(egui::Rect::from_min_size(t.pos, t.galley.size()))
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing {label}"))
}
fn click(ctx: &egui::Context, app: &mut App, pos: egui::Pos2) -> egui::FullOutput {
    frame(
        ctx,
        app,
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            },
        ],
    );
    frame(
        ctx,
        app,
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        }],
    )
}
#[test]
fn actual_pointer_keyboard_file_controls_dispatch_save_and_load_to_the_app() {
    let path = directory().join("typed.json");
    let mut app = old_app();
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    ctx.style_mut(|s| s.animation_time = 0.0);
    let out = frame(&ctx, &mut app, vec![]);
    let out = click(&ctx, &mut app, text_rect(&out, "Draft file").center());
    click(
        &ctx,
        &mut app,
        text_rect(&out, "initial-contents.json").center(),
    );
    let out = frame(
        &ctx,
        &mut app,
        vec![
            egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers {
                    ctrl: true,
                    command: true,
                    ..Default::default()
                },
            },
            egui::Event::Text(path.to_string_lossy().into_owned()),
        ],
    );
    assert_eq!(app.scene.contents.file_path, path.to_string_lossy());
    click(&ctx, &mut app, text_rect(&out, "Save draft").center());
    assert!(matches!(
        app.scene.contents.draft_action,
        Some(DraftFileAction::Save(_))
    ));
    app.service_draft_file_action(&ctx);
    assert!(path.exists());
    assert!(app.scene.contents.io_feedback.contains("Saved draft"));
    app.scene.contents.editor.as_mut().unwrap().undo().unwrap();
    let out = frame(&ctx, &mut app, vec![]);
    click(&ctx, &mut app, text_rect(&out, "Load draft").center());
    assert!(matches!(
        app.scene.contents.draft_action,
        Some(DraftFileAction::Load(_))
    ));
    app.service_draft_file_action(&ctx);
    assert_eq!(
        app.scene
            .contents
            .editor
            .as_ref()
            .unwrap()
            .document()
            .counts(),
        [0, 0, 64]
    );
    assert!(
        app.scene
            .contents
            .editor
            .as_ref()
            .unwrap()
            .undo_label()
            .is_none()
    );
    assert!(app.worker.is_none());
}
fn voxel_request(app: &App, path: &std::path::Path) -> crate::Request {
    use crate::{
        run_sets::{
            ArchiveBudget,
            voxel_material::{ExecutionPolicy, FrozenInput, PressurePolicy, run_set::*},
        },
        voxel_material_execution,
    };
    let spec = FrozenInput::capture(
        app.scene.contents.editor.as_ref().unwrap(),
        ExecutionPolicy {
            requested_dt_s: 0.001,
            requested_steps: 3,
            gravity_m_s2: [0.0; 3],
            pressure: PressurePolicy {
                relative_residual: 1e-11,
                absolute_residual: 1e-12,
                divergence_limit: 1e-8,
                max_iterations: 2000,
            },
            numerical_memory_limit_bytes: 8 * 1024 * 1024,
            recording: ArchiveBudget {
                max_database_bytes: 1024 * 1024,
                max_record_bytes: 128 * 1024,
                max_records: 4,
            },
            executor: voxel_material_execution::executor(),
        },
    )
    .unwrap();
    crate::Request {
        force_fields: vec![],
        collision_mesh: None,
        size: 4,
        steps: 3,
        tight: false,
        threads: 1,
        methods: vec![PressureImplementation::JacobiPcgV1],
        emitter: None,
        observe: false,
        history_budget_mib: 16,
        preview_resolution: crate::history::PreviewResolution::Edge(4),
        domain_side: 1.0,
        requested_dt: 0.001,
        material: None,
        material_color: [45, 125, 215],
        recording_limits: spec.execution().recording.limits(),
        recording_directory: path.to_string_lossy().into_owned(),
        saved_set: None,
        configured_set: None,
        prior_run_ids: vec![],
        voxel: Some(voxel_material_execution::VoxelRequest {
            revision: Arc::new(VoxelRunSetRevision {
                schema_version: 3,
                kind: VoxelSetKind::VoxelMaterialSet,
                set_id: "draft-busy-check".into(),
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
        manifest_receipt: None,
        progress_gate: None,
        failure_gate: None,
    }
}
#[test]
fn live_existing_worker_refuses_both_file_actions_before_io() {
    use std::sync::{Mutex, mpsc};
    let dir = directory();
    let mut app = old_app();
    let original = digest(&app);
    let mut request = voxel_request(&app, &dir.join("run"));
    let (reached, rx) = mpsc::sync_channel(0);
    let (tx, release) = mpsc::sync_channel(0);
    request.progress_gate = Some(Arc::new(crate::ProgressGate {
        reached,
        release: Mutex::new(release),
    }));
    app.worker = Some(crate::Worker::start(request).unwrap());
    rx.recv_timeout(std::time::Duration::from_secs(10)).unwrap();
    let file = dir.join("must-not-exist.json");
    assert!(
        app.draft_file_action(DraftFileAction::Save(file.clone()))
            .unwrap_err()
            .contains("stop the simulation")
    );
    assert!(
        app.draft_file_action(DraftFileAction::Load(file.clone()))
            .is_err()
    );
    assert!(!file.exists());
    assert_eq!(digest(&app), original);
    let worker = app.worker.take().unwrap();
    worker.cancel();
    tx.send(()).unwrap();
    assert!(
        worker
            .handle
            .join()
            .unwrap()
            .err()
            .unwrap()
            .contains("Cancelled")
    );
}
#[test]
fn load_checks_all_retained_request_tracks_and_preserves_their_original_bindings() {
    let dir = directory();
    let mut app = old_app();
    let original = digest(&app);
    let force = rheon::ForceObject {
        id: SceneObjectId(22),
        name: "Retained other track".into(),
        enabled: false,
        transform: Default::default(),
        units: rheon::ForceUnits::Acceleration,
        kind: rheon::ForceKind::Directional { vector: [0.0; 3] },
        support: rheon::ForceSupport::Global,
        time: Default::default(),
    };
    let fields = rheon::ForceScene {
        version: 1,
        objects: vec![force],
    }
    .freeze()
    .unwrap();
    let mut request = voxel_request(&app, &dir.join("unused"));
    request.voxel = None;
    request.methods = PressureImplementation::ALL.to_vec();
    request.force_fields = vec![None, Some(fields.clone())];
    app.running_request = Some(request);
    let path = dir.join("conflict.json");
    DraftRevision::capture(&editor(22, grid()))
        .unwrap()
        .publish(&path)
        .unwrap();
    assert!(
        app.draft_file_action(DraftFileAction::Load(path))
            .unwrap_err()
            .contains("retained run")
    );
    assert_eq!(digest(&app), original);
    let path = dir.join("valid.json");
    DraftRevision::capture(&editor(23, grid()))
        .unwrap()
        .publish(&path)
        .unwrap();
    app.draft_file_action(DraftFileAction::Load(path)).unwrap();
    let retained = app.running_request.as_ref().unwrap();
    assert_eq!(retained.size, 4);
    assert_eq!(retained.domain_side, 1.0);
    assert_eq!(
        retained.force_fields[1].as_ref().unwrap().scene(),
        fields.scene()
    );
    assert!(!dir.join("unused").exists());
}
fn scene_frame(
    ctx: &egui::Context,
    app: &mut App,
    events: Vec<egui::Event>,
    service: bool,
) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1250.0, 900.0),
            )),
            events,
            focused: true,
            ..Default::default()
        },
        |ctx| {
            app.scene.show(
                ctx,
                false,
                false,
                None,
                crate::force_run_policy::Target::NativeCarrier,
                Err(crate::voxel_material_setup::UNAVAILABLE.into()),
            );
            if service {
                app.service_draft_file_action(ctx);
            }
            egui::CentralPanel::default().show(ctx, |ui| {
                app.viewport.domain_side = app.domain_side;
                app.viewport.show_scene(
                    ui,
                    crate::viewport::ViewInputs {
                        frame: None,
                        emitter: None,
                        workers: &[],
                        grid_edge: app.size as usize,
                        inspector: None,
                        scene: Some(&mut app.scene),
                        editable_scene: true,
                    },
                );
            });
        },
    )
}
fn canvas_rect(output: &egui::FullOutput) -> egui::Rect {
    output
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            egui::Shape::Rect(r) if r.fill == egui::Color32::from_rgb(14, 20, 29) => Some(r.rect),
            _ => None,
        })
        .expect("actual viewport canvas")
}
fn button(pos: egui::Pos2, pressed: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Default::default(),
    }
}
#[test]
fn composed_file_button_service_and_viewport_drain_coalesced_and_held_load_tails() {
    let path = directory().join("source.json");
    DraftRevision::capture(&editor(91, grid()))
        .unwrap()
        .publish(&path)
        .unwrap();
    let mut app = old_app();
    app.scene.open = true;
    app.scene.contents.file_path = path.to_string_lossy().into_owned();
    app.viewport.camera = crate::camera::OrbitCamera {
        yaw: 0.0,
        pitch: 0.0,
        orthographic: true,
        ..Default::default()
    };
    let ctx = egui::Context::default();
    ctx.style_mut(|s| s.animation_time = 0.0);
    let out = scene_frame(&ctx, &mut app, vec![], false);
    let header = text_rect(&out, "Draft file").center();
    scene_frame(
        &ctx,
        &mut app,
        vec![egui::Event::PointerMoved(header), button(header, true)],
        false,
    );
    scene_frame(&ctx, &mut app, vec![button(header, false)], false);
    let camera = app.viewport.camera.clone();
    for held in [false, true] {
        let out = scene_frame(&ctx, &mut app, vec![], false);
        let load = text_rect(&out, "Load draft").center();
        let point = canvas_rect(&out).center();
        // Queue through the actual button. Delay servicing only in this fixture
        // to exercise a pending action with a coalesced input batch at its real
        // Scene→App-service→Viewport boundary, without assigning a latch.
        scene_frame(
            &ctx,
            &mut app,
            vec![egui::Event::PointerMoved(load), button(load, true)],
            false,
        );
        scene_frame(&ctx, &mut app, vec![button(load, false)], false);
        assert!(matches!(
            app.scene.contents.draft_action,
            Some(DraftFileAction::Load(_))
        ));
        let mut events = vec![egui::Event::PointerMoved(point), button(point, true)];
        if !held {
            events.push(button(point, false));
        }
        scene_frame(&ctx, &mut app, events, true);
        let e = app.scene.contents.editor.as_ref().unwrap();
        assert_eq!(e.document().revision(), 0);
        assert_eq!(e.document().counts(), [0, 0, 64]);
        assert!(e.undo_label().is_none() && e.redo_label().is_none());
        assert_eq!(app.viewport.camera, camera);
        assert!(app.scene.draft().objects.is_empty());
        if held {
            scene_frame(
                &ctx,
                &mut app,
                vec![egui::Event::PointerMoved(point + egui::vec2(10.0, 0.0))],
                true,
            );
            assert_eq!(
                app.scene
                    .contents
                    .editor
                    .as_ref()
                    .unwrap()
                    .document()
                    .revision(),
                0
            );
            scene_frame(&ctx, &mut app, vec![button(point, false)], true);
            assert_eq!(
                app.scene
                    .contents
                    .editor
                    .as_ref()
                    .unwrap()
                    .document()
                    .revision(),
                0
            );
        }
        scene_frame(
            &ctx,
            &mut app,
            vec![egui::Event::PointerMoved(point), button(point, true)],
            true,
        );
        scene_frame(&ctx, &mut app, vec![button(point, false)], true);
        let e = app.scene.contents.editor.as_ref().unwrap();
        assert_eq!(e.document().revision(), 1);
        assert!(e.document().counts()[1] > 0);
        assert_eq!(app.viewport.camera, camera);
    }
}
