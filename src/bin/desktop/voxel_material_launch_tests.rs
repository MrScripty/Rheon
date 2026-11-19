//! Actual existing-worker adoption and retained-owner checks. No native evidence.
use super::*;
use crate::egui;
use crate::{
    history::LiveHistory,
    voxel_material_integration_tests::{recipe, request, root},
};
use std::{
    sync::{Mutex, mpsc},
    time::{Duration, Instant},
};
fn wait(test: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !test() {
        assert!(
            Instant::now() < deadline,
            "worker did not reach bounded boundary"
        );
        std::thread::yield_now();
    }
}
fn old_result() -> App {
    old_result_with_profile(true)
}
#[allow(clippy::field_reassign_with_default)]
fn old_result_with_profile(voxel: bool) -> App {
    let output = root();
    let (_, spec) = recipe(0.001, 2, [0.0; 3], 1024 * 1024);
    let mut r = request(spec, &output.join("old"));
    if !voxel {
        r.voxel = None;
    }
    let w = Worker::start(r.clone()).unwrap();
    let history = Arc::clone(&w.history);
    let rows = w.handle.join().unwrap().unwrap();
    history.carrier_inspectors[0].ensure_timestep(1).unwrap();
    let mut app = App::default();
    app.history = Some(history);
    app.running_request = Some(r);
    app.rows = rows;
    app.playing = true;
    app.play_time = 0.00125;
    app.timeline_step = 1;
    app.follow_live = false;
    app.scene.begin_run(false);
    if voxel {
        app.scene.bind_retained_no_forces(0);
    } else {
        app.scene.bind_retained_fields(0, None).unwrap();
    }
    app
}
fn gate() -> (
    Arc<crate::ProgressGate>,
    mpsc::Receiver<()>,
    mpsc::Sender<()>,
) {
    let (tx, rx) = mpsc::sync_channel(1);
    let (release, wait) = mpsc::channel();
    (
        Arc::new(crate::ProgressGate {
            reached: tx,
            release: Mutex::new(wait),
        }),
        rx,
        release,
    )
}
#[test]
fn held_old_selection_adopts_only_typed_constructor_then_actual_prefix() {
    let mut app = old_result();
    let old = Arc::clone(app.history.as_ref().unwrap());
    let old_identity = old.carrier_inspectors[0].selected_identity().unwrap();
    let old_size = old.retained_payload_bytes().unwrap();
    let (_, spec) = recipe(0.001, 2, [0.0, -1.0, 0.0], 1024 * 1024);
    let frozen_digest = spec.digest();
    let output = root().join("next");
    let mut next = request(spec, &output);
    let (prepare, reached, release) = gate();
    let (accepted, accepted_rx, accepted_release) = gate();
    next.failure_gate = Some(prepare);
    next.progress_gate = Some(accepted);
    let ctx = egui::Context::default();
    app.launch_request(next).unwrap();
    reached.recv_timeout(Duration::from_secs(10)).unwrap();
    assert!(Arc::ptr_eq(&old, app.history.as_ref().unwrap()));
    assert!(app.playing);
    assert_eq!(app.play_time, 0.00125);
    assert!(app.rows.len() == 1 && app.pending_voxel.is_some());
    assert!(!app.acknowledge_voxel_constructor().unwrap());
    let before = app
        .running_request
        .as_ref()
        .unwrap()
        .voxel
        .as_ref()
        .unwrap()
        .revision
        .digest()
        .unwrap();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        app.sidebar(ctx);
        let view = app.prepare_scene_presentation().unwrap();
        assert_eq!(view.1.as_ref().unwrap().step, 1);
        app.scene_timeline_ui(ctx, 0, None);
    });
    assert_eq!(app.timeline_step, 1);
    assert_eq!(app.play_time, 0.00125);
    assert!(app.playing);
    assert_eq!(
        old.carrier_inspectors[0].selected_identity().unwrap(),
        old_identity
    );
    assert_eq!(old.retained_payload_bytes().unwrap(), old_size);
    assert_eq!(
        app.running_request
            .as_ref()
            .unwrap()
            .voxel
            .as_ref()
            .unwrap()
            .revision
            .digest()
            .unwrap(),
        before
    );
    release.send(()).unwrap();
    accepted_rx.recv_timeout(Duration::from_secs(10)).unwrap();
    app.poll(&ctx);
    assert!(app.pending_voxel.is_none());
    assert!(!Arc::ptr_eq(&old, app.history.as_ref().unwrap()));
    assert!(app.rows.is_empty() && !app.playing && app.track == 0);
    assert_eq!(
        app.history.as_ref().unwrap().carrier_inspectors[0]
            .with_voxel(|v| v.metadata.initial_input_sha256),
        Some(frozen_digest)
    );
    app.prepare_scene_presentation();
    assert!(app.scene.display_scene().objects.is_empty());
    accepted_release.send(()).unwrap();
    wait(|| app.worker.as_ref().unwrap().handle.is_finished());
    app.poll(&ctx);
    assert!(app.worker.is_none() && app.rows.len() == 1);
    assert!(app.status.starts_with("Complete"), "{}", app.status);
    app.timeline_step = 1;
    app.follow_live = false;
    assert_eq!(app.selected_frame().1.unwrap().step, 1);
    assert_eq!(
        app.history.as_ref().unwrap().carrier_inspectors[0]
            .selected_identity()
            .unwrap()
            .1,
        1
    );
}
#[test]
fn fast_completed_worker_adopts_before_join_and_late_cancel_is_not_cancelled() {
    let mut app = old_result();
    let (_, spec) = recipe(0.001, 1, [0.0; 3], 1024 * 1024);
    app.launch_request(request(spec, &root().join("fast")))
        .unwrap();
    wait(|| app.worker.as_ref().unwrap().handle.is_finished());
    app.cancel();
    app.poll(&egui::Context::default());
    assert!(app.pending_voxel.is_none() && app.worker.is_none());
    assert_eq!(app.rows.len(), 1);
    assert!(app.status.starts_with("Complete"), "{}", app.status);
}
#[test]
fn preconstructor_cancellation_and_output_failure_keep_old_owners_and_path() {
    for cancelled in [true, false] {
        let mut app = old_result();
        let old = Arc::clone(app.history.as_ref().unwrap());
        app.run_sets.path = "old-retained-intent.json".into();
        let old_id = old.carrier_inspectors[0].selected_identity();
        let (_, spec) = recipe(0.001, 1, [0.0; 3], 1024 * 1024);
        let output = root().join("new-output");
        if !cancelled {
            std::fs::write(&output, b"existing unrelated file").unwrap();
        }
        let mut next = request(spec, &output);
        let (prepare, reached, release) = gate();
        next.failure_gate = Some(prepare);
        app.launch_request(next).unwrap();
        reached.recv_timeout(Duration::from_secs(10)).unwrap();
        if cancelled {
            app.cancel();
        }
        release.send(()).unwrap();
        wait(|| app.worker.as_ref().unwrap().handle.is_finished());
        app.poll(&egui::Context::default());
        assert!(Arc::ptr_eq(&old, app.history.as_ref().unwrap()));
        assert!(app.pending_voxel.is_none() && app.worker.is_none());
        assert!(app.playing && app.play_time == 0.00125 && app.timeline_step == 1);
        assert_eq!(app.rows.len(), 1);
        assert_eq!(old.carrier_inspectors[0].selected_identity(), old_id);
        assert_eq!(app.run_sets.path, "old-retained-intent.json");
        assert!(app.status.contains("Previous result"), "{}", app.status);
        assert!(
            app.status.contains(output.to_str().unwrap()),
            "{}",
            app.status
        );
        if cancelled {
            assert!(!output.exists());
        } else {
            assert_eq!(std::fs::read(output).unwrap(), b"existing unrelated file");
        }
    }
}
#[test]
fn actual_spare_capacity_is_charged_and_refuses_before_output() {
    let mut app = old_result();
    let base = app.retained_payload_for_voxel().unwrap();
    app.rows[0].pixels.reserve_exact(4096);
    let delta = app.rows[0].pixels.capacity() - 16;
    assert!(app.retained_payload_for_voxel().unwrap() >= base + delta);
    let new_path = root().join("refused");
    let (_, spec) = recipe(0.001, 1, [0.0; 3], 1024 * 1024);
    // Real retained allocation, not caller-supplied claimed bytes.
    app.rows[0].pixels.reserve_exact(240 * 1024 * 1024);
    let old = Arc::clone(app.history.as_ref().unwrap());
    let error = app.launch_request(request(spec, &new_path)).unwrap_err();
    assert!(error.contains("384 MiB"), "{error}");
    assert!(!new_path.exists() && app.worker.is_none());
    assert!(Arc::ptr_eq(&old, app.history.as_ref().unwrap()));
}
#[test]
fn history_real_capacity_includes_all_selected_owners() {
    let h = LiveHistory::new(4, 2, 1).unwrap();
    let empty = h.retained_payload_bytes().unwrap();
    let (_, spec) = recipe(0.001, 1, [0.0; 3], 1024 * 1024);
    let sim = spec.construct().unwrap();
    let frame = PreviewFrame::capture_voxel(&sim, 4, 0.0).unwrap();
    let expected = frame.density.capacity() + std::mem::size_of::<PreviewFrame>();
    h.publish(0, frame).unwrap();
    assert!(h.retained_payload_bytes().unwrap() >= empty + expected);
}
#[test]
fn cancellation_after_constructor_adopts_new_prefix_before_terminal() {
    let mut app = old_result();
    let old = Arc::clone(app.history.as_ref().unwrap());
    let (_, spec) = recipe(0.001, 3, [0.0; 3], 1024 * 1024);
    let mut next = request(spec, &root().join("after-constructor"));
    let (accepted, reached, release) = gate();
    next.progress_gate = Some(accepted);
    app.launch_request(next).unwrap();
    reached.recv_timeout(Duration::from_secs(10)).unwrap();
    app.cancel();
    release.send(()).unwrap();
    wait(|| app.worker.as_ref().unwrap().handle.is_finished());
    app.poll(&egui::Context::default());
    assert!(app.worker.is_none() && app.pending_voxel.is_none());
    assert!(!Arc::ptr_eq(&old, app.history.as_ref().unwrap()));
    assert!(app.rows.is_empty());
    assert!(app.status.contains("Cancelled"), "{}", app.status);
    assert!(app.run_sets.path.ends_with("finished.json"));
    let (count, frame) = app
        .history
        .as_ref()
        .unwrap()
        .view(0, usize::MAX, true)
        .unwrap();
    assert_eq!((count, frame.unwrap().step), (2, 1));
}
#[test]
fn cloned_request_receipt_cannot_mutate_old_attempt_while_pending() {
    let mut app = old_result();
    let old_receipt = Arc::clone(
        app.running_request
            .as_ref()
            .unwrap()
            .manifest_receipt
            .as_ref()
            .unwrap(),
    );
    let old_path = old_receipt.lock().unwrap().clone();
    let old_capacity = old_receipt.lock().unwrap().capacity();
    let mut next = app.running_request.as_ref().unwrap().clone();
    assert!(Arc::ptr_eq(
        next.manifest_receipt.as_ref().unwrap(),
        &old_receipt
    ));
    next.recording_directory = root()
        .join("cloned-request-with-new-output")
        .to_string_lossy()
        .into();
    let (accepted, reached, release) = gate();
    next.progress_gate = Some(accepted);
    app.launch_request(next).unwrap();
    reached.recv_timeout(Duration::from_secs(10)).unwrap();
    let new_receipt = Arc::clone(
        app.pending_voxel
            .as_ref()
            .unwrap()
            .request
            .manifest_receipt
            .as_ref()
            .unwrap(),
    );
    assert!(!Arc::ptr_eq(&old_receipt, &new_receipt));
    assert_eq!(*old_receipt.lock().unwrap(), old_path);
    assert_eq!(old_receipt.lock().unwrap().capacity(), old_capacity);
    let new_started = new_receipt.lock().unwrap().clone();
    assert!(new_started.ends_with("started.json") && new_started != old_path);
    app.cancel();
    release.send(()).unwrap();
    wait(|| app.worker.as_ref().unwrap().handle.is_finished());
    app.poll(&egui::Context::default());
    assert!(app.run_sets.path.ends_with("finished.json"));
    assert_ne!(app.run_sets.path, old_path.to_string_lossy());
    assert_eq!(*old_receipt.lock().unwrap(), old_path);
    assert_eq!(old_receipt.lock().unwrap().capacity(), old_capacity);
}
#[test]
fn queued_authoring_refuses_before_output_and_pending_boundary_discards_stale_commands() {
    let mut app = old_result_with_profile(false);
    assert!(app.scene.contents.editor.is_none()); // creation would succeed without the guard
    app.scene.open = true;
    let old = Arc::clone(app.history.as_ref().unwrap());
    let old_id = app.scene.selected();
    let old_bytes = app.retained_payload_for_voxel().unwrap();
    let output = root().join("queued-refusal");
    let (_, spec) = recipe(0.001, 1, [0.0; 3], 1024 * 1024);
    app.scene.contents.create_requested = true;
    assert!(
        app.launch_request(request(spec.clone(), &output))
            .unwrap_err()
            .contains("queued")
    );
    app.scene.contents.create_requested = false;
    app.scene.contents.draft_action = Some(crate::voxel_material_authoring::DraftFileAction::Load(
        root().join("absent.json"),
    ));
    assert!(
        app.launch_request(request(spec.clone(), &output))
            .unwrap_err()
            .contains("queued")
    );
    assert!(app.worker.is_none() && !output.exists());
    assert!(app.scene.contents.editor.is_none() && app.scene.selected() == old_id);
    assert!(Arc::ptr_eq(&old, app.history.as_ref().unwrap()));
    assert_eq!(app.retained_payload_for_voxel().unwrap(), old_bytes);
    app.scene.contents.draft_action = None;
    let mut next = request(spec, &output);
    let (prepare, reached, release) = gate();
    next.failure_gate = Some(prepare);
    app.launch_request(next).unwrap();
    reached.recv_timeout(Duration::from_secs(10)).unwrap();
    app.scene.contents.create_requested = true;
    app.scene.contents.draft_action = Some(crate::voxel_material_authoring::DraftFileAction::Save(
        root().join("never-written.json"),
    ));
    let status = app.status.clone();
    let ctx = egui::Context::default();
    let mut pointer = egui::pos2(450.0, 350.0);
    // Actual preparation Scene, production command boundary, held timeline and
    // disabled real viewport receive raw press/move/release/undo input.
    for (pressed, key) in [
        (None, false),
        (Some(true), false),
        (None, true),
        (Some(false), false),
    ] {
        let mut events = vec![egui::Event::PointerMoved(pointer)];
        if let Some(pressed) = pressed {
            events.push(egui::Event::PointerButton {
                pos: pointer,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            });
        }
        if key {
            events.push(egui::Event::Key {
                key: egui::Key::Z,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::CTRL,
            });
        }
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(900.0, 650.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                app.scene.show_preparing(ctx);
                app.service_initial_contents_commands(ctx);
                let (count, frame) = app.prepare_scene_presentation().unwrap();
                app.scene_timeline_ui(ctx, count, frame.as_deref());
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.add_enabled_ui(app.pending_voxel.is_none(), |ui| {
                        app.viewport.show_scene(
                            ui,
                            crate::viewport::ViewInputs {
                                frame,
                                emitter: None,
                                workers: &[],
                                grid_edge: 4,
                                inspector: Some(&old.inspector),
                                scene: Some(&mut app.scene),
                                editable_scene: false,
                            },
                        )
                    });
                });
            },
        );
        if pressed.is_none() && !key {
            pointer = output
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    egui::Shape::Text(text) if text.galley.text() == "Particles" => {
                        Some(text.pos + text.galley.size() * 0.5)
                    }
                    _ => None,
                })
                .expect("real disabled Particles tab was painted");
        }
        assert!(app.viewport.retained_payload_bytes_for_voxel().is_ok());
    }
    assert!(app.scene.contents.editor.is_none() && app.scene.selected() == old_id);
    assert!(!app.scene.contents.create_requested && app.scene.contents.draft_action.is_none());
    assert!(app.scene.contents.io_feedback.contains("not executed"));
    assert_eq!(app.status, status);
    assert!(Arc::ptr_eq(&old, app.history.as_ref().unwrap()));
    assert!(app.timeline_step == 1 && app.play_time == 0.00125 && app.playing);
    app.cancel();
    release.send(()).unwrap();
    wait(|| app.worker.as_ref().unwrap().handle.is_finished());
    app.poll(&ctx);
    assert!(app.scene.contents.editor.is_none());
    // Freshly issued idle-Draft creation still uses the same production owner.
    app.scene.context = scene_ui::SceneContext::Draft;
    app.size = 4;
    app.scene.contents.create_requested = true;
    app.service_initial_contents_commands(&ctx);
    assert!(app.scene.contents.editor.is_some() && app.scene.contents_selected());
}
