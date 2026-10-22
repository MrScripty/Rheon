//! Argument/readiness/egui input checks for the actual-App entry transport.
//! These are source/component checks, not native-window or release-artifact evidence.
use super::*;
use crate::{SetupAction, SetupWorkflow, egui, history, voxel_material_integration_tests};
use rheon::GridGeometry;
use rheon::initial_contents::{BrushDepth, BrushOperation, ContentsBrush, ContentsMaterial};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

fn arguments(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

fn app() -> App {
    let mut app = App::default();
    app.size = 4;
    app.steps = 1;
    app.requested_dt = 0.001;
    app.history_budget_mib = 16;
    app.preview_resolution = history::PreviewResolution::Adaptive;
    app.recording_database_mib = 1;
    app.recording_record_mib = 1;
    app.recording_directory = voxel_material_integration_tests::root()
        .join("native-control-component")
        .to_string_lossy()
        .into();
    app.cylinder = false;
    app.materials.config.gravity = [0.0; 3];
    app.scene
        .create_contents(GridGeometry::new([4; 3], [0.25; 3], [0.0; 3]).unwrap())
        .unwrap();
    app.scene
        .contents
        .editor
        .as_mut()
        .unwrap()
        .fill_air()
        .unwrap();
    app
}

fn remove_cell(app: &mut App) {
    let editor = app.scene.contents.editor.as_mut().unwrap();
    editor
        .begin(ContentsBrush {
            operation: BrushOperation::Remove,
            material: ContentsMaterial::Water,
            radius: 0.1,
            axis: 2,
            layer: 1,
            depth: BrushDepth::Layer,
        })
        .unwrap();
    editor.segment([0.375; 3], [0.375; 3]).unwrap();
    assert!(editor.commit().unwrap());
}

fn context() -> egui::Context {
    let ctx = egui::Context::default();
    ctx.style_mut(|style| style.animation_time = 0.0);
    ctx
}

fn frame(
    ctx: &egui::Context,
    events: Vec<egui::Event>,
    mut paint: impl FnMut(&egui::Context),
) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(900.0, 1400.0),
            )),
            events,
            ..Default::default()
        },
        |ctx| paint(ctx),
    )
}

fn button_center(output: &egui::FullOutput, label: &str) -> egui::Pos2 {
    output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Text(text) if text.galley.text() == label => {
                let rect = egui::Rect::from_min_size(text.pos, text.galley.size());
                shape.clip_rect.intersects(rect).then(|| rect.center())
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing visible button {label}"))
}

fn pointer(pos: egui::Pos2, pressed: bool) -> Vec<egui::Event> {
    vec![
        egui::Event::PointerMoved(pos),
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        },
    ]
}

fn global(
    app: &mut App,
    ctx: &egui::Context,
    events: Vec<egui::Event>,
) -> (egui::FullOutput, Option<SetupAction>) {
    let mut action = None;
    let output = frame(ctx, events, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| action = app.run_controls(ui));
    });
    (output, action)
}

fn panel(
    app: &mut App,
    ctx: &egui::Context,
    events: Vec<egui::Event>,
    editable: bool,
) -> egui::FullOutput {
    let readiness = app.contents_run_readiness();
    frame(ctx, events, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            app.scene.contents.controls(ui, editable, readiness.clone());
        });
    })
}

#[test]
fn ordinary_startup_and_valid_contents_remain_unavailable_without_qualification() {
    let mut ordinary = App::default();
    assert!(!ordinary.configure_arguments(Vec::<String>::new()).unwrap());
    assert!(!ordinary.voxel_qualification);
    assert!(ordinary.worker.is_none() && ordinary.scene.contents.editor.is_none());
    let mut app = app();
    app.contents_capture_readiness().unwrap();
    assert!(
        app.contents_run_readiness()
            .unwrap_err()
            .contains("qualification")
    );
    let target = PathBuf::from(&app.recording_directory);
    let ctx = context();
    let _ = global(&mut app, &ctx, vec![]);
    let (output, _) = global(&mut app, &ctx, vec![]);
    let pos = button_center(&output, "Simulate contents");
    assert!(global(&mut app, &ctx, pointer(pos, true)).1.is_none());
    assert!(global(&mut app, &ctx, pointer(pos, false)).1.is_none());
    let ctx = context();
    let _ = panel(&mut app, &ctx, vec![], true);
    let output = panel(&mut app, &ctx, vec![], true);
    let pos = button_center(&output, "Simulate contents");
    panel(&mut app, &ctx, pointer(pos, true), true);
    panel(&mut app, &ctx, pointer(pos, false), true);
    assert!(!app.scene.contents.run_requested);
    assert!(app.worker.is_none() && app.history.is_none() && !target.exists());
}

#[cfg(debug_assertions)]
#[test]
fn explicit_debug_arguments_select_bounded_defaults_without_document_or_output() {
    let output = voxel_material_integration_tests::root().join("debug-opt-in-no-output");
    let mut app = App::default();
    assert!(
        !app.configure_arguments(vec![
            "--qualify-voxel-material".into(),
            output.to_string_lossy().into()
        ])
        .unwrap()
    );
    assert!(app.voxel_qualification);
    assert_eq!(app.recording_directory, output.to_string_lossy());
    assert_eq!(
        (app.size, app.domain_side, app.steps, app.requested_dt),
        (8, 1.0, 8, 0.001)
    );
    assert_eq!(app.materials.config.gravity, [0.0, -9.80665, 0.0]);
    assert_eq!(
        (
            app.history_budget_mib,
            app.recording_database_mib,
            app.recording_record_mib
        ),
        (16, 32, 4)
    );
    assert!(!app.cylinder);
    assert!(
        app.scene.contents.editor.is_none()
            && app.worker.is_none()
            && app.pending_voxel.is_none()
            && app.running_request.is_none()
            && app.history.is_none()
    );
    assert!(!output.exists());
}

#[test]
fn argument_refusals_precede_collision_import_autostart_and_output() {
    let root = voxel_material_integration_tests::root();
    let output = root
        .join("qualification-refusal-no-output")
        .to_string_lossy()
        .into_owned();
    let missing_mesh = root
        .join("must-not-be-imported.obj")
        .to_string_lossy()
        .into_owned();
    let cases = [
        arguments(&["--qualify-voxel-material"]),
        arguments(&["--qualify-voxel-material", ""]),
        arguments(&["--qualify-voxel-material", "--collision-run"]),
        arguments(&[
            "--qualify-voxel-material",
            &output,
            "--qualify-voxel-material",
            &output,
        ]),
        arguments(&["--qualify-voxel-material", &output, "--unknown-option"]),
        arguments(&["--qualify-voxel-material", &output, "--collision-run"]),
        arguments(&["--collision-run", "--qualify-voxel-material", &output]),
        arguments(&[
            "--qualify-voxel-material",
            &output,
            "--collision-policy",
            "surface",
        ]),
        arguments(&[
            "--collision-mesh",
            &missing_mesh,
            "--qualify-voxel-material",
            &output,
        ]),
        arguments(&["--collision-policy"]),
    ];
    for values in cases {
        let mut app = App::default();
        let error = app.configure_arguments(values.clone()).unwrap_err();
        assert!(!error.is_empty(), "{values:?}");
        if values
            .iter()
            .any(|value| value == "--qualify-voxel-material")
            && values.iter().any(|value| value == "--collision-mesh")
        {
            assert!(
                error.to_ascii_lowercase().contains("qualif"),
                "conflict must precede import: {error}"
            );
        }
        assert!(
            app.worker.is_none()
                && app.history.is_none()
                && app.scene.contents.editor.is_none()
                && app.collision.mesh.is_none()
        );
        assert!(!PathBuf::from(&output).exists() && !PathBuf::from(&missing_mesh).exists());
    }
}

#[test]
fn legacy_collision_argument_assignment_remains_separate_from_import_and_start() {
    let mut app = App::default();
    assert!(
        app.configure_arguments(arguments(&[
            "--collision-policy",
            "solid",
            "--collision-policy",
            "surface",
            "--collision-mesh",
            "first-not-imported.obj",
            "--collision-mesh",
            "last-not-imported.obj",
            "--collision-run",
            "--collision-run",
        ]))
        .unwrap()
    );
    assert!(!app.voxel_qualification && !app.collision.solid);
    assert_eq!(app.collision.path, "last-not-imported.obj");
    assert_eq!(app.steps, 180);
    assert!(app.worker.is_none() && app.collision.mesh.is_none());
}

#[test]
fn shared_capture_readiness_refuses_current_vacuum_active_unsupported_and_closing_state() {
    let mut app = app();
    let target = PathBuf::from(&app.recording_directory);
    app.contents_capture_readiness().unwrap();
    app.cylinder = true;
    assert!(
        app.contents_capture_readiness()
            .unwrap_err()
            .contains("emitter")
    );
    assert!(
        app.capture_contents_request()
            .err()
            .unwrap()
            .contains("emitter")
    );
    app.cylinder = false;
    app.setup_workflow = SetupWorkflow::AcceptedSession;
    assert!(
        app.contents_capture_readiness()
            .unwrap_err()
            .contains("configured")
    );
    app.setup_workflow = SetupWorkflow::Preview;
    remove_cell(&mut app);
    assert!(
        app.contents_capture_readiness()
            .unwrap_err()
            .contains("Vacuum")
    );
    assert!(
        app.capture_contents_request()
            .err()
            .unwrap()
            .contains("Vacuum")
    );
    let editor = app.scene.contents.editor.as_mut().unwrap();
    editor.undo().unwrap();
    editor
        .begin(ContentsBrush {
            operation: BrushOperation::Replace,
            material: ContentsMaterial::Water,
            radius: 0.1,
            axis: 2,
            layer: 1,
            depth: BrushDepth::Layer,
        })
        .unwrap();
    assert!(app.contents_capture_readiness().is_err());
    assert!(app.capture_contents_request().is_err());
    app.scene.contents.editor.as_mut().unwrap().cancel();
    app.contents_capture_readiness().unwrap();
    app.closing = true;
    assert!(app.contents_capture_readiness().is_err());
    assert!(app.capture_contents_request().is_err());
    assert!(app.worker.is_none() && !target.exists());
}

#[test]
fn enabled_global_button_dispatches_voxel_worker_and_panel_revalidates_queued_intent() {
    let mut app = app();
    app.voxel_qualification = true;
    app.contents_run_readiness().unwrap();
    let ctx = context();
    let _ = global(&mut app, &ctx, vec![]);
    let (output, _) = global(&mut app, &ctx, vec![]);
    let pos = button_center(&output, "Simulate contents");
    assert!(global(&mut app, &ctx, pointer(pos, true)).1.is_none());
    let action = global(&mut app, &ctx, pointer(pos, false)).1;
    assert!(matches!(action, Some(SetupAction::RunContents)));
    app.dispatch_setup_action(&ctx, action);
    let worker = app
        .worker
        .as_ref()
        .expect("same global action must start the existing Worker");
    assert!(app.pending_voxel.as_ref().unwrap().request.voxel.is_some());
    let deadline = Instant::now() + Duration::from_secs(10);
    while !worker.handle.is_finished() {
        assert!(
            Instant::now() < deadline,
            "bounded 4³ one-interval dispatch did not finish"
        );
        std::thread::yield_now();
    }
    app.poll(&ctx);
    assert!(app.status.starts_with("Complete"), "{}", app.status);
    assert!(app.running_request.as_ref().unwrap().voxel.is_some());
    assert!(app.running_request.as_ref().unwrap().material.is_none());

    let mut panel_app = self::app();
    panel_app.voxel_qualification = true;
    let target = PathBuf::from(&panel_app.recording_directory);
    let ctx = context();
    let _ = panel(&mut panel_app, &ctx, vec![], false);
    let output = panel(&mut panel_app, &ctx, vec![], false);
    let pos = button_center(&output, "Simulate contents");
    panel(&mut panel_app, &ctx, pointer(pos, true), false);
    panel(&mut panel_app, &ctx, pointer(pos, false), false);
    assert!(
        !panel_app.scene.contents.run_requested,
        "editable state must also gate the panel"
    );
    let output = panel(&mut panel_app, &ctx, vec![], true);
    let pos = button_center(&output, "Simulate contents");
    panel(&mut panel_app, &ctx, pointer(pos, true), true);
    panel(&mut panel_app, &ctx, pointer(pos, false), true);
    assert!(panel_app.scene.contents.run_requested);
    remove_cell(&mut panel_app);
    panel_app.service_initial_contents_commands(&ctx);
    assert!(!panel_app.scene.contents.run_requested);
    assert!(panel_app.status.contains("Vacuum"), "{}", panel_app.status);
    assert!(panel_app.worker.is_none() && panel_app.history.is_none() && !target.exists());
}
