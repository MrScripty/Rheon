//! UI-only input/layout checks. These tests never dispatch launch intent.
use super::*;

fn frame(
    ctx: &egui::Context,
    size: egui::Vec2,
    events: Vec<egui::Event>,
    mut paint: impl FnMut(&egui::Context),
) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
            events,
            ..Default::default()
        },
        |ctx| paint(ctx),
    )
}
fn labels(output: &egui::FullOutput) -> Vec<(String, egui::Rect)> {
    output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Text(text) => {
                let rect = egui::Rect::from_min_size(text.pos, text.galley.size());
                shape
                    .clip_rect
                    .intersects(rect)
                    .then(|| (text.galley.text().to_owned(), rect))
            }
            _ => None,
        })
        .collect()
}
fn has(output: &egui::FullOutput, label: &str) -> bool {
    labels(output).iter().any(|(text, _)| text == label)
}
fn rect(output: &egui::FullOutput, label: &str) -> egui::Rect {
    labels(output)
        .into_iter()
        .find(|(text, _)| text == label)
        .unwrap_or_else(|| panic!("missing {label}: {:?}", labels(output)))
        .1
}
fn click(
    ctx: &egui::Context,
    size: egui::Vec2,
    output: &egui::FullOutput,
    label: &str,
    mut paint: impl FnMut(&egui::Context),
) {
    let pos = rect(output, label).center();
    for pressed in [true, false] {
        let _ = frame(
            ctx,
            size,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ],
            &mut paint,
        );
    }
}
fn context() -> egui::Context {
    let ctx = egui::Context::default();
    ctx.style_mut(|style| style.animation_time = 0.0);
    ctx
}

#[test]
fn ui_setup_pointer_switch_keeps_one_launch_and_preserves_inputs() {
    let ctx = context();
    let size = egui::vec2(1250.0, 820.0);
    let mut app = App::default();
    app.steps = 37;
    app.requested_dt = 0.013;
    let before = (
        app.domain_side,
        app.size,
        app.materials.config.material,
        app.selected,
    );
    for _ in 0..3 {
        let _ = frame(&ctx, size, vec![], |ctx| {
            app.sidebar(ctx);
        });
    }
    let output = frame(&ctx, size, vec![], |ctx| {
        app.sidebar(ctx);
    });
    assert!(has(&output, "Simulate"));
    assert!(!has(&output, "Save & run native"));
    assert!(!has(&output, "Compare with CPU adapter"));
    click(&ctx, size, &output, "Simulation preview", |ctx| {
        app.sidebar(ctx);
    });
    let output = frame(&ctx, size, vec![], |ctx| {
        app.sidebar(ctx);
    });
    click(&ctx, size, &output, "Accepted Session", |ctx| {
        app.sidebar(ctx);
    });
    let output = frame(&ctx, size, vec![], |ctx| {
        app.sidebar(ctx);
    });
    assert!(app.setup_workflow == SetupWorkflow::AcceptedSession);
    assert!(has(&output, "Save & run native"));
    assert!(!has(&output, "Simulate"));
    assert!(has(&output, "Compare with CPU adapter"));
    assert!(!has(&output, "Fluid material"));
    assert!(!has(&output, "Emitter"));
    assert_eq!(
        before,
        (
            app.domain_side,
            app.size,
            app.materials.config.material,
            app.selected
        )
    );
    assert!(app.material_mode);
    assert_eq!(app.steps, 37);
    assert_eq!(app.requested_dt, 0.013);
    assert!(app.worker.is_none() && app.running_request.is_none() && app.history.is_none());
}

#[test]
fn ui_setup_accepted_session_uses_carrier_inputs_without_changing_preview_mode() {
    let ctx = context();
    let size = egui::vec2(900.0, 2500.0);
    let mut app = App::default();
    app.setup_workflow = SetupWorkflow::AcceptedSession;
    let mut paint = |ctx: &egui::Context| {
        egui::CentralPanel::default().show(ctx, |ui| app.controls(ui));
    };
    for _ in 0..3 {
        let _ = frame(&ctx, size, vec![], &mut paint);
    }
    let output = frame(&ctx, size, vec![], &mut paint);
    click(&ctx, size, &output, "Simulation", &mut paint);
    let output = frame(&ctx, size, vec![], &mut paint);
    assert!(has(&output, "Numerical cells / axis (count mode)"));
    assert!(has(&output, "Tight pressure tolerance"));
    assert!(has(&output, "Maximum step"));
    assert!(has(&output, "Steps"));
    assert!(!has(&output, "Particle samples / side"));
    assert!(!has(&output, "Liquid preview stiffness"));
    assert!(!has(&output, "Liquid / gas material simulation"));
    click(&ctx, size, &output, "Domain & environment", &mut paint);
    let output = frame(&ctx, size, vec![], &mut paint);
    assert!(has(&output, "Domain side"));
    assert!(!has(&output, "Ambient / wall temp"));
    assert!(app.material_mode);
    assert!(app.worker.is_none());
}

#[test]
fn ui_setup_status_details_are_bounded_and_actions_survive_scroll_and_inspector() {
    for size in [egui::vec2(900.0, 650.0), egui::vec2(1250.0, 820.0)] {
        for status in [
            format!(
                "Recording unavailable: {}\nPrevious data retained; select a writable directory.",
                "long-path/".repeat(300)
            ),
            format!(
                "Blocking error: {}\nAccepted generation 1; acknowledged prefix 2; recording incomplete.",
                "x".repeat(16_384)
            ),
            format!(
                "Integrity verified; requested recording incomplete.\n{}\nDo not infer completion from integrity.",
                "Δt 0.02 s · μ Pa·s · 読み取り · e\u{301}\n".repeat(256)
            ),
        ] {
            let ctx = context();
            let mut app = App::default();
            app.status = status.clone();
            for _ in 0..3 {
                let _ = frame(&ctx, size, vec![], |ctx| {
                    app.sidebar(ctx);
                });
            }
            let output = frame(&ctx, size, vec![], |ctx| {
                app.sidebar(ctx);
            });
            click(&ctx, size, &output, "Status details", |ctx| {
                app.sidebar(ctx);
            });
            let output = frame(&ctx, size, vec![], |ctx| {
                app.sidebar(ctx);
            });
            let launch = rect(&output, "Simulate");
            assert!(launch.bottom() <= size.y && launch.top() > 0.0);
            assert!(rect(&output, "Status details").bottom() < size.y);
            assert!(output.shapes.iter().any(|shape| matches!(&shape.shape,
            egui::Shape::Text(text) if text.galley.text() == status && shape.clip_rect.height() <= 100.0)));
            let form_point = rect(&output, "Rheon").center() + egui::vec2(0.0, 30.0);
            let _ = frame(
                &ctx,
                size,
                vec![
                    egui::Event::PointerMoved(form_point),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0.0, -2000.0),
                        modifiers: Default::default(),
                    },
                ],
                |ctx| {
                    app.sidebar(ctx);
                },
            );
            let output = frame(&ctx, size, vec![], |ctx| {
                app.sidebar(ctx);
            });
            assert_eq!(launch, rect(&output, "Simulate"));
            assert!(
                !has(&output, "Rheon"),
                "setup form should scroll independently"
            );
            click(&ctx, size, &output, "Inspector", |ctx| {
                app.sidebar(ctx);
            });
            let output = frame(&ctx, size, vec![], |ctx| {
                app.sidebar(ctx);
            });
            assert!(has(&output, "Numerical inspector"));
            assert_eq!(launch, rect(&output, "Simulate"));
            let text_id = egui::Id::new("run-status-text");
            let status_point = ctx
                .read_response(text_id)
                .expect("visible full-status control")
                .interact_rect
                .center();
            for pressed in [true, false] {
                let _ = frame(
                    &ctx,
                    size,
                    vec![
                        egui::Event::PointerMoved(status_point),
                        egui::Event::PointerButton {
                            pos: status_point,
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: Default::default(),
                        },
                    ],
                    |ctx| {
                        app.sidebar(ctx);
                    },
                );
            }
            assert!(ctx.memory(|memory| memory.has_focus(text_id)));
            let modifiers = egui::Modifiers {
                ctrl: true,
                command: true,
                ..Default::default()
            };
            let _ = frame(
                &ctx,
                size,
                vec![egui::Event::Key {
                    key: egui::Key::End,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers,
                }],
                |ctx| {
                    app.sidebar(ctx);
                },
            );
            let state = egui::text_edit::TextEditState::load(&ctx, text_id).unwrap();
            assert_eq!(
                state.cursor.char_range().unwrap().primary.index,
                status.chars().count()
            );
            let _ = frame(
                &ctx,
                size,
                vec![egui::Event::Key {
                    key: egui::Key::A,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers,
                }],
                |ctx| {
                    app.sidebar(ctx);
                },
            );
            let output = frame(
                &ctx,
                size,
                vec![egui::Event::Copy, egui::Event::Text("unwanted edit".into())],
                |ctx| {
                    app.sidebar(ctx);
                },
            );
            assert!(
                output
                    .platform_output
                    .commands
                    .iter()
                    .any(|command| matches!(command,
            egui::OutputCommand::CopyText(text) if text == &status))
            );
            assert_eq!(app.status, status);
            assert!(app.worker.is_none());
            let _ = frame(
                &ctx,
                size,
                vec![egui::Event::Key {
                    key: egui::Key::Tab,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers {
                        shift: true,
                        ..Default::default()
                    },
                }],
                |ctx| {
                    app.sidebar(ctx);
                },
            );
            let _ = frame(&ctx, size, vec![], |ctx| {
                app.sidebar(ctx);
            });
            assert!(
                !ctx.memory(|memory| memory.has_focus(text_id)),
                "status must permit keyboard exit"
            );
        }
    }
}

#[test]
fn ui_setup_uninspected_adapter_does_not_emit_launch_intent() {
    let ctx = context();
    let size = egui::vec2(900.0, 1800.0);
    let mut controls = configured_controls::ConfiguredControls::default();
    let mut requested = false;
    let mut paint = |ctx: &egui::Context| {
        egui::CentralPanel::default().show(ctx, |ui| {
            controls.show(ui, true);
            requested |= controls.launch_button(ui, true);
        });
    };
    for _ in 0..3 {
        let _ = frame(&ctx, size, vec![], &mut paint);
    }
    let output = frame(&ctx, size, vec![], &mut paint);
    click(&ctx, size, &output, "Compare with CPU adapter", &mut paint);
    let output = frame(&ctx, size, vec![], &mut paint);
    assert!(has(&output, "Inspect the adapter package before running."));
    click(&ctx, size, &output, "Save & run pair", &mut paint);
    assert!(!requested);
}

#[test]
fn ui_setup_running_summary_uses_frozen_physical_samples_and_timestep() {
    // An inert request snapshot exercises projection without constructing a Worker.
    let material = rheon::ParticleFluidConfig {
        samples_per_axis: 7,
        ..Default::default()
    };
    let request = Request {
        force_fields: Vec::new(),
        collision_mesh: None,
        size: 32,
        steps: 37,
        tight: false,
        threads: 1,
        methods: vec![PressureImplementation::default()],
        emitter: None,
        observe: false,
        history_budget_mib: 16,
        preview_resolution: history::PreviewResolution::Adaptive,
        domain_side: 2.0,
        requested_dt: 0.013,
        material: Some(material),
        material_color: [0; 3],
        recording_limits: Default::default(),
        recording_directory: String::new(),
        saved_set: None,
        configured_set: None,
        prior_run_ids: vec![],
        manifest_receipt: None,
        progress_gate: None,
        failure_gate: None,
    };
    let ctx = context();
    let output = frame(&ctx, egui::vec2(900.0, 650.0), vec![], |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| running_setup_ui(ui, &request));
    });
    assert!(has(&output, "Inputs frozen at launch."));
    assert!(has(&output, "7 samples/axis · domain 2.000 m"));
    assert!(has(&output, "37 steps · maximum step 13.000 ms"));
    assert!(has(&output, "Liquid / gas material preview"));
    assert!(!has(&output, "Simulate"));
}

#[test]
fn ui_setup_expanded_forms_keep_text_inside_sidebar_width() {
    for workflow in [SetupWorkflow::Preview, SetupWorkflow::AcceptedSession] {
        let ctx = context();
        let size = egui::vec2(600.0, 6000.0);
        let mut app = App::default();
        app.setup_workflow = workflow;
        let mut paint = |ctx: &egui::Context| {
            egui::SidePanel::left("narrow-form")
                .exact_width(280.0)
                .resizable(false)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| app.controls(ui));
                });
        };
        for _ in 0..3 {
            let _ = frame(&ctx, size, vec![], &mut paint);
        }
        for section in [
            "Domain & environment",
            "Simulation",
            "Numerical recording (every accepted step)",
            "Workspace limits",
            "Viewport & navigation",
            "CPU workers",
        ] {
            let output = frame(&ctx, size, vec![], &mut paint);
            if has(&output, section) {
                click(&ctx, size, &output, section, &mut paint);
            }
        }
        let output = frame(&ctx, size, vec![], &mut paint);
        let overflow: Vec<_> = output
            .shapes
            .iter()
            .filter_map(|shape| {
                if let egui::Shape::Text(text) = &shape.shape {
                    let right = text.pos.x + text.galley.size().x;
                    (right > shape.clip_rect.right() + 1.0).then(|| text.galley.text().to_owned())
                } else {
                    None
                }
            })
            .collect();
        assert!(
            overflow.is_empty(),
            "horizontally clipped form text: {overflow:?}"
        );
        assert!(app.worker.is_none());
    }
}

#[test]
fn ui_setup_viewport_caption_wraps_at_narrow_scaled_width() {
    let ctx = context();
    let output = frame(&ctx, egui::vec2(300.0, 300.0), vec![], |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut viewport = viewport::Viewport::default();
            viewport.collision_mesh = Some(Arc::new(
                rheon::CollisionMesh::new(
                    vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                    vec![[0, 1, 2]],
                    rheon::MeshPolicy::TwoSidedSurface,
                )
                .unwrap(),
            ));
            viewport.show(ui, None, None, &[], 32, None);
        });
    });
    let (_, bounds) = labels(&output)
        .into_iter()
        .find(|(text, _)| text.starts_with("3D / ORBIT VIEW"))
        .expect("viewport information caption");
    assert!(
        bounds.right() <= 300.0,
        "caption spills past viewport: {bounds:?}"
    );
    assert!(bounds.height() > 12.0, "narrow caption should wrap");
    let (_, mesh_bounds) = labels(&output)
        .into_iter()
        .find(|(text, _)| text.starts_with("Amber: collision wireframe"))
        .expect("collision mesh caption");
    assert!(
        mesh_bounds.top() > bounds.bottom(),
        "caption overlaps mesh note"
    );
    assert!(
        mesh_bounds.right() <= 300.0,
        "mesh note spills past viewport"
    );
}

#[test]
fn ui_setup_long_recording_path_and_pressure_controls_stay_within_form_width() {
    for width in [240.0, 280.0] {
        for method in PressureImplementation::ALL {
            let ctx = context();
            let size = egui::vec2(600.0, 6000.0);
            let mut app = App::default();
            app.setup_workflow = SetupWorkflow::AcceptedSession;
            app.selected = method;
            let path = format!("/retained/{}/recordings", "long-directory/".repeat(150));
            app.recording_directory = path.clone();
            let mut bounds = None;
            let mut paint = |ctx: &egui::Context| {
                egui::SidePanel::left("bounded-form")
                    .exact_width(width)
                    .resizable(false)
                    .show(ctx, |ui| {
                        let form = egui::ScrollArea::vertical().show(ui, |ui| app.controls(ui));
                        bounds = Some((form.content_size.x, form.inner_rect.width()));
                    });
            };
            for _ in 0..3 {
                let _ = frame(&ctx, size, vec![], &mut paint);
            }
            for section in ["Simulation", "Numerical recording (every accepted step)"] {
                let output = frame(&ctx, size, vec![], &mut paint);
                click(&ctx, size, &output, section, &mut paint);
            }
            let output = frame(&ctx, size, vec![], &mut paint);
            assert!(has(&output, "Local recording directory"));
            assert!(has(&output, method.label()));
            let path_field = output
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    egui::Shape::Text(text) if text.galley.text() == path => {
                        Some(shape.clip_rect.center())
                    }
                    _ => None,
                })
                .expect("rendered recording path field");
            for pressed in [true, false] {
                let _ = frame(
                    &ctx,
                    size,
                    vec![
                        egui::Event::PointerMoved(path_field),
                        egui::Event::PointerButton {
                            pos: path_field,
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: Default::default(),
                        },
                    ],
                    &mut paint,
                );
            }
            let path_id = ctx
                .memory(|memory| memory.focused())
                .expect("path has pointer focus");
            let modifiers = egui::Modifiers {
                ctrl: true,
                command: true,
                ..Default::default()
            };
            for key in [egui::Key::End, egui::Key::A] {
                let _ = frame(
                    &ctx,
                    size,
                    vec![egui::Event::Key {
                        key,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers,
                    }],
                    &mut paint,
                );
                if key == egui::Key::End {
                    let state = egui::text_edit::TextEditState::load(&ctx, path_id).unwrap();
                    assert_eq!(
                        state.cursor.char_range().unwrap().primary.index,
                        path.chars().count()
                    );
                }
            }
            let copied = frame(&ctx, size, vec![egui::Event::Copy], &mut paint);
            assert!(copied.platform_output.commands.iter().any(
                |command| matches!(command, egui::OutputCommand::CopyText(text) if text == &path)
            ));
            let (content, available) = bounds.unwrap();
            assert!(
                content <= available + 1.0,
                "form width {width}, method {}: content {content} exceeds {available}",
                method.label()
            );
            assert_eq!(app.recording_directory, path);
            assert_eq!(app.selected, method);
            assert!(app.worker.is_none());
        }
    }
}

#[test]
fn ui_setup_synthetic_active_worker_keeps_progress_frozen_inputs_and_cancel_signal() {
    // Reuse the inert owned-handle pattern from dropping_app_cancels_and_joins_owned_worker.
    // The handle waits for explicit test release, not for cancellation. Nothing calls
    // Worker::start/run_owned, publishes frames, opens recordings or advances physics.
    for size in [egui::vec2(900.0, 650.0), egui::vec2(1250.0, 820.0)] {
        let ctx = context();
        let cancel = Arc::new(AtomicBool::new(false));
        let progress = Arc::new(AtomicUsize::new(1));
        let (release, wait) = std::sync::mpsc::sync_channel(1);
        let handle = std::thread::spawn(move || {
            let _ = wait.recv_timeout(Duration::from_secs(5));
            Err("Synthetic UI fixture released; no simulation executed".into())
        });
        let history = LiveHistory::with_budget(4, 4, 1, 16 * 1024 * 1024).unwrap();
        history.begin_step(0, 2).unwrap();
        let mut app = App::default();
        app.setup_workflow = SetupWorkflow::AcceptedSession;
        app.steps = 99;
        app.requested_dt = 0.001;
        app.running_request = Some(Request {
            force_fields: Vec::new(),
            collision_mesh: None,
            size: 4,
            steps: 4,
            tight: false,
            threads: 1,
            methods: vec![PressureImplementation::default()],
            emitter: None,
            observe: false,
            history_budget_mib: 16,
            preview_resolution: history::PreviewResolution::Adaptive,
            domain_side: 2.0,
            requested_dt: 0.013,
            material: None,
            material_color: [0; 3],
            recording_limits: Default::default(),
            recording_directory: String::new(),
            saved_set: None,
            configured_set: None,
            prior_run_ids: vec![],
            manifest_receipt: None,
            progress_gate: None,
            failure_gate: None,
        });
        app.worker = Some(Worker {
            scheduler: rheon::CpuScheduler::new(),
            cancel: Arc::clone(&cancel),
            progress: Arc::clone(&progress),
            total: 4,
            history,
            observer: None,
            child: None,
            handle,
        });
        for _ in 0..3 {
            let _ = frame(&ctx, size, vec![], |ctx| {
                app.sidebar(ctx);
            });
        }
        let output = frame(&ctx, size, vec![], |ctx| {
            app.sidebar(ctx);
        });
        assert!(has(&output, "1/4 steps"));
        assert!(has(&output, "Simulation preview"));
        assert!(has(&output, "4 steps · maximum step 13.000 ms"));
        assert!(has(&output, "4 samples/axis · domain 2.000 m"));
        assert!(!has(&output, "Simulate") && !has(&output, "Save & run native"));
        let cancel_rect = rect(&output, "Cancel simulation");
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, size);
        assert!(screen.contains_rect(cancel_rect));
        assert!(screen.contains_rect(rect(&output, "1/4 steps")));
        assert!(cancel_rect.right() <= 280.0);
        click(&ctx, size, &output, "Inspector", |ctx| {
            app.sidebar(ctx);
        });
        let output = frame(&ctx, size, vec![], |ctx| {
            app.sidebar(ctx);
        });
        assert!(has(&output, "Numerical inspector"));
        assert_eq!(cancel_rect, rect(&output, "Cancel simulation"));
        click(&ctx, size, &output, "Cancel simulation", |ctx| {
            app.sidebar(ctx);
        });
        assert!(cancel.load(Ordering::Relaxed));
        assert_eq!(progress.load(Ordering::Relaxed), 1);
        assert!(
            !app.worker.as_ref().unwrap().handle.is_finished(),
            "Cancel routes a signal; this fixture does not simulate worker retirement"
        );
        assert_eq!((app.steps, app.requested_dt), (99, 0.001));
        release.send(()).unwrap();
        let worker = app.worker.take().unwrap();
        assert!(
            worker
                .handle
                .join()
                .unwrap()
                .err()
                .unwrap()
                .contains("Synthetic UI fixture")
        );
    }
}

#[test]
#[ignore = "Numerical workload: separate admission and sealed external supervisor required"]
fn bounded_genuine_cancel_contract() {
    // This case is deliberately excluded from ui_setup_ and every default test run.
    assert_eq!(
        std::env::var("RHEON_UI_CANCEL_ADMISSION").as_deref(),
        Ok("genuine-cancel-v1")
    );
    let root = std::path::PathBuf::from(
        std::env::var("RHEON_UI_CANCEL_ROOT").expect("supervised fresh root"),
    );
    assert!(root.is_absolute());
    let seal_path = root.join("contract.json");
    assert!(std::fs::metadata(&seal_path).unwrap().len() <= 65_536);
    let seal: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&seal_path).unwrap()).unwrap();
    assert_eq!(
        seal["test"],
        "setup_ui_tests::bounded_genuine_cancel_contract"
    );
    assert_eq!(seal["commit"], build_identity::source_revision());
    assert_ne!(
        build_identity::source_revision(),
        "build revision unspecified"
    );
    // Supervisor executes immutable memfd bytes, so current_exe is a memfd, not
    // the original build pathname. The supervisor binds its SHA256/size to seal.
    assert_eq!(
        std::fs::metadata("/proc/self/exe").unwrap().len(),
        seal["binary_bytes"].as_u64().unwrap()
    );
    let recording_root = root.join("recordings");
    assert_eq!(std::fs::read_dir(&recording_root).unwrap().count(), 0);
    let limits = rheon::recording::RecordingLimits {
        max_database_bytes: 8 * 1024 * 1024,
        max_record_bytes: 1024 * 1024,
        max_records: 3,
    };
    // Preserve each reached stage before any assertion can fail. No cleanup runs.
    let event = |stage: &str| {
        use std::io::Write;
        let mut log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(root.join("events.jsonl"))
            .unwrap();
        writeln!(log, "{}", serde_json::json!({"stage": stage})).unwrap();
        log.sync_all().unwrap();
    };
    event("admission_checked_before_worker_start");
    let (reached, gate) = std::sync::mpsc::sync_channel(1);
    let (release, resume) = std::sync::mpsc::sync_channel(1);
    let request = Request {
        force_fields: Vec::new(),
        collision_mesh: None,
        size: 4,
        steps: 2,
        tight: false,
        threads: 1,
        methods: vec![PressureImplementation::JacobiPcgV1],
        emitter: None,
        observe: false,
        history_budget_mib: 16,
        preview_resolution: history::PreviewResolution::Edge(4),
        domain_side: 1.0,
        requested_dt: 0.02,
        material: None,
        material_color: [85, 190, 245],
        recording_limits: limits,
        recording_directory: recording_root.to_str().unwrap().into(),
        saved_set: None,
        configured_set: None,
        prior_run_ids: vec![],
        manifest_receipt: None,
        progress_gate: Some(Arc::new(ProgressGate {
            reached,
            release: std::sync::Mutex::new(resume),
        })),
        failure_gate: None,
    };
    let mut app = App::default();
    app.running_request = Some(request.clone());
    let worker = Worker::start(request).unwrap();
    app.history = Some(Arc::clone(&worker.history));
    let progress = Arc::clone(&worker.progress);
    let cancel = Arc::clone(&worker.cancel);
    app.worker = Some(worker);
    let ctx = context();
    let size = egui::vec2(900.0, 650.0);
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match gate.try_recv() {
            Ok(()) => break,
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
            Err(error) => panic!("gate disconnected: {error}"),
        }
        assert!(Instant::now() < deadline, "first-step observation timeout");
        assert!(
            !app.worker.as_ref().unwrap().handle.is_finished(),
            "worker ended before gate"
        );
        let _ = frame(&ctx, size, vec![], |ctx| {
            app.sidebar(ctx);
        });
        std::thread::yield_now();
    }
    event("first_accepted_step_gate_held");
    assert_eq!(progress.load(Ordering::Relaxed), 1);
    assert!(!app.worker.as_ref().unwrap().handle.is_finished());
    let output = frame(&ctx, size, vec![], |ctx| {
        app.sidebar(ctx);
    });
    assert!(has(&output, "1/2 steps"));
    click(&ctx, size, &output, "Cancel simulation", |ctx| {
        app.sidebar(ctx);
    });
    assert!(cancel.load(Ordering::Relaxed));
    assert!(
        !app.worker.as_ref().unwrap().handle.is_finished(),
        "gate must still hold worker"
    );
    event("actual_egui_cancel_clicked_while_gate_held");
    release.send(()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !app.worker.as_ref().unwrap().handle.is_finished() {
        assert!(Instant::now() < deadline, "retirement observation timeout");
        let _ = frame(&ctx, size, vec![], |ctx| {
            app.sidebar(ctx);
        });
        std::thread::yield_now();
    }
    // Join only after is_finished; neither observation loop blocks on recv/join.
    app.poll(&ctx);
    event("actual_worker_joined");
    assert!(app.worker.is_none() && app.rows.is_empty() && app.textures.is_empty());
    assert_eq!(progress.load(Ordering::Relaxed), 1);
    assert!(app.status.contains("Cancelled") || app.status.contains("cancelled"));
    let path = app.history.as_ref().unwrap().carrier_inspectors[0]
        .archive_path()
        .unwrap();
    assert!(path.starts_with(&recording_root));
    let reader = rheon::recording::RecordingReader::open(&path, limits).unwrap();
    reader.verify().unwrap();
    assert_eq!(reader.count().unwrap(), 2);
    assert_eq!(reader.status().unwrap().outcome, "cancelled");
    assert_eq!(reader.status().unwrap().accepted_generation, Some(1));
    let result = serde_json::json!({
        "contract": "genuine-cancel-v1", "test": "setup_ui_tests::bounded_genuine_cancel_contract",
        "source": build_identity::source_revision(), "binary_sha256": seal["binary_sha256"],
        "gate_held": true, "ui_cancel": true, "joined": true, "progress": 1,
        "integrity_verified": true, "cancelled": true, "record_count": 2,
        "accepted_generation": 1, "requested_complete": false, "archive": path,
        "ui_status": app.status,
    });
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root.join("result.json"))
        .unwrap();
    file.write_all(serde_json::to_string_pretty(&result).unwrap().as_bytes())
        .unwrap();
    file.sync_all().unwrap();
    event("complete_contract_result_preserved");
}
