//! Test-only synthetic presentation. Never compiled into the production launch.
use super::*;
const LABEL: &str = "Synthetic presentation data · no worker or physics";
const TIME_NOTE: &str =
    "Supplied timestamps; calculation duration is unmeasured. No archive or scientific result.";
fn style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(22, 98, 138);
    style.spacing.item_spacing = egui::vec2(8.0, 7.0);
    ctx.set_style(style);
}
fn fixture() -> App {
    let request = Request {
        force_fields: Vec::new(),
        collision_mesh: None,
        size: 2,
        steps: 1,
        tight: false,
        threads: 1,
        methods: PressureImplementation::ALL.to_vec(),
        emitter: None,
        observe: false,
        history_budget_mib: 16,
        preview_resolution: history::PreviewResolution::Edge(2),
        domain_side: 1.0,
        requested_dt: 0.25,
        material: None,
        material_color: [0; 3],
        recording_limits: rheon::recording::RecordingLimits {
            max_database_bytes: 65536,
            max_record_bytes: 16384,
            max_records: 2,
        },
        recording_directory: "synthetic-unused-no-output".into(),
        saved_set: None,
        configured_set: None,
        prior_run_ids: vec![],
        voxel: None,
        manifest_receipt: None,
        progress_gate: None,
        failure_gate: None,
    };
    validate_request(&request).expect("literal metadata is legal; no solver constructed");
    let history = request
        .preview_history()
        .expect("bounded in-memory display history");
    for (track, method) in request.methods.iter().enumerate() {
        for (step, time) in [0.0, 0.25].into_iter().enumerate() {
            let density = match (track, step) {
                (0, 0) => vec![0; 8],
                (0, 1) => vec![255, 0, 0, 0, 0, 0, 0, 0],
                (1, 0) => vec![0, 0, 0, 0, 0, 0, 0, 128],
                _ => vec![0, 128, 128, 0, 0, 0, 0, 0],
            };
            history
                .publish(
                    track,
                    Arc::new(PreviewFrame {
                        method: *method,
                        step,
                        time,
                        seconds: 0.0,
                        edge: 2,
                        nonzero_tracer_cells: density.iter().filter(|&&v| v != 0).count(),
                        density,
                        report: None,
                        particle_report: None,
                        voxel: None,
                        material_color: None,
                    }),
                )
                .unwrap();
        }
    }
    let mut app = App::default();
    app.history = Some(history);
    app.running_request = Some(request);
    app.follow_live = false;
    app.track = 0;
    app.timeline_step = 0;
    app.playing = false;
    app.play_time = 0.0;
    app.status = LABEL.into();
    app.scene.begin_run(false);
    app
}
struct Presentation {
    app: App,
    deadline: Instant,
}
impl Presentation {
    fn new() -> Self {
        Self {
            app: fixture(),
            deadline: Instant::now() + Duration::from_secs(300),
        }
    }
    fn paint(&mut self, ctx: &egui::Context) -> Option<egui::Rect> {
        assert!(self.app.worker.is_none());
        if Instant::now() >= self.deadline {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        egui::TopBottomPanel::top("synthetic-label").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.strong(LABEL);
                if ui.button("Close presentation").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
            ui.small(TIME_NOTE);
        });
        // Match production sidebar reservation without composing any run controls.
        if !(self.app.scene.open && ctx.content_rect().width() < 1100.0) {
            egui::SidePanel::left("synthetic-settings-geometry")
                .exact_width(280.0)
                .show(ctx, |ui| {
                    ui.label("Frozen synthetic Request");
                    ui.label("2 tracks · 2 literal frames each");
                    ui.label("1 m · 2³ display voxels");
                    ui.label("No worker, recorder, model or solver");
                });
        }
        if self.app.scene.context == scene_ui::SceneContext::Draft {
            self.app.playing = false;
        }
        if self.app.playing {
            self.app.play_time +=
                self.app.playback_speed * f64::from(ctx.input(|i| i.stable_dt.min(0.1)));
        }
        let retained_view = self.app.prepare_scene_presentation();
        let previous = self.app.scene.context;
        self.app.scene.show(
            ctx,
            true,
            false,
            None,
            force_run_policy::Target::NativeCarrier,
            Err("Contents execution disabled in this fixture".into()),
        );
        if previous != self.app.scene.context {
            self.app.playing = false;
        }
        let retained = self.app.scene.context == scene_ui::SceneContext::Retained;
        let (count, frame) = if retained {
            retained_view.unwrap_or_else(|| self.app.selected_frame_bound_track())
        } else {
            (0, None)
        };
        let timeline = self.app.scene_timeline_ui(ctx, count, frame.as_deref());
        self.app.viewport.domain_side = 1.0;
        self.app.viewport.authored_forces_pending =
            !self.app.scene.display_scene().objects.is_empty();
        // Every frame resets to the fresh default Appearance owner. Disabled UI
        // prevents user dispatch into numerical labs; no production App::update.
        self.app.viewport.presentation_only_appearance();
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                ui.add_enabled_ui(false, |ui| {
                    self.app.viewport.show_scene(
                        ui,
                        viewport::ViewInputs {
                            frame,
                            emitter: None,
                            workers: &[],
                            grid_edge: 2,
                            inspector: None,
                            scene: Some(&mut self.app.scene),
                            editable_scene: false,
                        },
                    )
                });
            });
        assert!(self.app.viewport.presentation_is_inert());
        ctx.request_repaint_after(Duration::from_millis(if self.app.playing {
            16
        } else {
            100
        }));
        timeline
    }
}
impl eframe::App for Presentation {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.paint(ctx);
    }
}
#[test]
fn synthetic_retained_history_timeline_and_disabled_modes_are_workerless() {
    let mut h = Presentation::new();
    let original = h.app.running_request.as_ref().unwrap().clone();
    let deadline = h.deadline;
    for (width, height) in [(1250.0, 820.0), (900.0, 650.0), (600.0, 433.33334)] {
        let ctx = egui::Context::default();
        style(&ctx);
        for _ in 0..2 {
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(width, height),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    assert!(h.paint(ctx).is_some());
                },
            );
            assert!(
                output
                    .shapes
                    .iter()
                    .any(|s| matches!(&s.shape,egui::Shape::Text(t)if t.galley.text()==LABEL))
            );
        }
        let mut tabs = vec![];
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(width, height),
                )),
                ..Default::default()
            },
            |ctx| {
                h.paint(ctx);
            },
        );
        for s in output.shapes {
            if let egui::Shape::Text(t) = s.shape
                && ["Particles", "Obstacle lab", "Transport session"].contains(&t.galley.text())
            {
                tabs.push(t.pos + t.galley.size() * 0.5);
            }
        }
        assert_eq!(tabs.len(), 3);
        for p in tabs {
            for pressed in [true, false] {
                let _ = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(width, height),
                        )),
                        events: vec![
                            egui::Event::PointerMoved(p),
                            egui::Event::PointerButton {
                                pos: p,
                                button: egui::PointerButton::Primary,
                                pressed,
                                modifiers: egui::Modifiers::NONE,
                            },
                        ],
                        ..Default::default()
                    },
                    |ctx| {
                        h.paint(ctx);
                    },
                );
                assert!(h.app.viewport.presentation_is_inert());
            }
        }
    }
    let history = h.app.history.as_ref().unwrap();
    assert_eq!(history.preview_edge, 2);
    assert_eq!(history.view(0, 0, false).unwrap().1.unwrap().time, 0.0);
    assert_eq!(history.view(1, 1, false).unwrap().1.unwrap().time, 0.25);
    h.app.track = 1;
    h.app.timeline_step = 1;
    h.app.follow_live = false;
    let ctx = egui::Context::default();
    style(&ctx);
    h.app.scene.context = scene_ui::SceneContext::Draft;
    h.app.playing = true;
    let _ = ctx.run(Default::default(), |ctx| assert!(h.paint(ctx).is_none()));
    assert!(!h.app.playing);
    assert_eq!(h.app.timeline_step, 1);
    h.app.scene.context = scene_ui::SceneContext::Retained;
    let _ = ctx.run(Default::default(), |ctx| assert!(h.paint(ctx).is_some()));
    assert_eq!(h.app.track, 1);
    assert_eq!(h.app.timeline_step, 1);
    assert_eq!(h.deadline, deadline);
    assert!(h.app.worker.is_none());
    let request = h.app.running_request.as_ref().unwrap();
    assert_eq!(request.size, original.size);
    assert_eq!(request.methods, original.methods);
    assert_eq!(
        request.requested_dt.to_bits(),
        original.requested_dt.to_bits()
    );
    assert!(
        request.collision_mesh.is_none() && request.material.is_none() && request.emitter.is_none()
    );
}
// Compile with cargo rustc --bin rheon-desktop -- --cfg test (no --test).
// This entry is the main-thread standalone test-config build, never an exact
// unit-test execution or the normal production binary launch.
pub fn run_native_presentation() -> eframe::Result {
    eframe::run_native(
        "Rheon · synthetic presentation · no physics",
        eframe::NativeOptions {
            persist_window: false,
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1250.0, 820.0])
                .with_min_inner_size([600.0, 430.0]),
            ..Default::default()
        },
        Box::new(|cc| {
            style(&cc.egui_ctx);
            Ok(Box::new(Presentation::new()))
        }),
    )
}
