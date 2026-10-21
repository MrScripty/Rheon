//! Isolated playback painting/input. Never call App::update, selected_frame,
//! history capture/view, viewport, Worker, simulations or filesystem fixtures.
use super::*;

fn context() -> egui::Context {
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    ctx.style_mut(|style| {
        style.animation_time = 0.0;
        style.spacing.item_spacing = egui::vec2(8.0, 7.0); // native app style
    });
    ctx
}
fn frame(
    ctx: &egui::Context,
    app: &mut App,
    width: f32,
    selected: Option<&PreviewFrame>,
    events: Vec<egui::Event>,
) -> (egui::FullOutput, egui::Rect) {
    let mut rect = egui::Rect::NOTHING;
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width, 650.0),
            )),
            events,
            ..Default::default()
        },
        |ctx| {
            // Reserve the actual setup sidebar width, without executing it.
            egui::SidePanel::left("timeline-test-sidebar")
                .exact_width(280.0)
                .resizable(false)
                .show(ctx, |_| {});
            rect = app.timeline_ui(ctx, if selected.is_some() { 5 } else { 0 }, selected);
        },
    );
    assert!(app.worker.is_none() && app.history.is_none());
    if let Some(request) = &app.running_request {
        assert!(
            request.collision_mesh.is_none()
                && request.material.is_none()
                && request.emitter.is_none()
                && request.saved_set.is_none()
                && request.configured_set.is_none()
                && request.manifest_receipt.is_none()
                && request.progress_gate.is_none()
                && request.failure_gate.is_none()
        );
    }
    (output, rect)
}
fn text_rect(output: &egui::FullOutput, label: &str) -> egui::Rect {
    output
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            egui::Shape::Text(t) if t.galley.text() == label => {
                Some(egui::Rect::from_min_size(t.pos, t.galley.size()).intersect(s.clip_rect))
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing {label}"))
}
fn assert_visible(output: &egui::FullOutput, panel: egui::Rect) {
    assert_visible_with_editor(output, panel, None);
}
fn assert_visible_with_editor(output: &egui::FullOutput, panel: egui::Rect, editor: Option<&str>) {
    assert!(
        panel.top() >= 0.0 && panel.bottom() <= 650.5,
        "panel outside bounds: {panel:?}"
    );
    let tree = output.platform_output.accesskit_update.as_ref().unwrap();
    let sizing_id = egui::accesskit::NodeId(egui::Id::new("timeline-sizing").value());
    let sizing = &tree
        .nodes
        .iter()
        .find(|(id, _)| *id == sizing_id)
        .unwrap()
        .1;
    assert!(
        sizing.is_hidden(),
        "sizing controls must not be exposed to AT"
    );
    let mut hidden = std::collections::HashSet::new();
    let mut pending = vec![sizing_id];
    while let Some(id) = pending.pop() {
        if hidden.insert(id) {
            let node = &tree.nodes.iter().find(|(key, _)| *key == id).unwrap().1;
            pending.extend_from_slice(node.children());
        }
    }
    let real_play_controls = tree
        .nodes
        .iter()
        .filter(|(id, node)| {
            !hidden.contains(id)
                && !node.is_hidden()
                && matches!(node.label(), Some("▶ Play" | "❚❚ Pause"))
        })
        .count();
    assert_eq!(
        real_play_controls, 1,
        "one accessible real playback control"
    );
    for shape in &output.shapes {
        if let egui::Shape::Text(t) = &shape.shape {
            let mut bounds = egui::Rect::from_min_size(t.pos, t.galley.size());
            if editor == Some(t.galley.text()) {
                // Single-line TextEdit deliberately scrolls/clips its edit buffer;
                // this exception applies only to the exact injected editor payload.
                bounds = bounds.intersect(shape.clip_rect);
                assert!(bounds.width() <= 64.0);
            }
            assert!(
                panel.expand(0.5).contains_rect(bounds)
                    && shape.clip_rect.expand(0.5).contains_rect(bounds),
                "clipped timeline text: {} at {bounds:?}, panel {panel:?}, clip {:?}",
                t.galley.text(),
                shape.clip_rect
            );
        }
    }
}
fn click(
    ctx: &egui::Context,
    app: &mut App,
    selected: &PreviewFrame,
    output: &egui::FullOutput,
    label: &str,
) {
    let pos = text_rect(output, label).center();
    for pressed in [true, false] {
        let _ = frame(
            ctx,
            app,
            900.0,
            Some(selected),
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ],
        );
    }
}

#[test]
fn timeline_empty_idle_uses_content_height_after_sidebar() {
    for width in [900.0, 1250.0] {
        let ctx = context();
        let mut app = App::default();
        let (output, rect) = frame(&ctx, &mut app, width, None, vec![]);
        assert_visible(&output, rect);
        for _ in 0..3 {
            let _ = frame(&ctx, &mut app, width, None, vec![]);
        }
        let (output, rect) = frame(&ctx, &mut app, width, None, vec![]);
        assert_visible(&output, rect);
        eprintln!(
            "idle timeline window={width}px strip={}px height={}px",
            rect.width(),
            rect.height()
        );
        assert!(rect.left() >= 280.0);
        assert!(
            rect.height() <= 70.0,
            "idle timeline {}px at window {width}",
            rect.height()
        );
        assert!(text_rect(&output, "▶ Play").is_positive());
        assert!(text_rect(&output, "Follow live").is_positive());
        assert!(app.follow_live && !app.playing);
    }
}

#[test]
fn timeline_controls_retain_transitions_and_physical_time() {
    // Direct presentation data, not captured/generated numerical evidence.
    let selected = PreviewFrame {
        method: PressureImplementation::JacobiPcgV1,
        step: 2,
        time: 0.04,
        seconds: 0.002,
        edge: 1,
        nonzero_tracer_cells: 0,
        density: vec![],
        report: None,
        particle_report: None,
        material_color: None,
    };
    let ctx = context();
    let mut app = App::default();
    for width in [900.0, 1250.0] {
        let cold = context();
        let mut cold_app = App::default();
        let (output, rect) = frame(&cold, &mut cold_app, width, Some(&selected), vec![]);
        assert_visible(&output, rect);
    }
    let (output, rect) = frame(&ctx, &mut app, 900.0, None, vec![]);
    assert_visible(&output, rect);
    app.timeline_step = 2;
    app.follow_live = false;
    let (output, rect) = frame(&ctx, &mut app, 900.0, Some(&selected), vec![]);
    assert_visible(&output, rect); // empty -> selected, without warm-up hiding clips
    for _ in 0..3 {
        let _ = frame(&ctx, &mut app, 900.0, Some(&selected), vec![]);
    }
    let (output, rect) = frame(&ctx, &mut app, 900.0, Some(&selected), vec![]);
    assert!(text_rect(&output, "Simulated 40.000 ms").is_positive());
    assert!(text_rect(&output, "Step calculation 2.000 ms").is_positive());
    for s in &output.shapes {
        if let egui::Shape::Text(t) = &s.shape {
            let bounds = egui::Rect::from_min_size(t.pos, t.galley.size());
            assert!(
                rect.expand(0.5).contains_rect(bounds),
                "caption overflow: {}",
                t.galley.text()
            );
        }
    }
    click(&ctx, &mut app, &selected, &output, "Step ▶");
    assert_eq!(app.timeline_step, 3);
    assert!(!app.follow_live && !app.playing);
    let (output, _) = frame(&ctx, &mut app, 900.0, Some(&selected), vec![]);
    click(&ctx, &mut app, &selected, &output, "◀ Step");
    assert_eq!(app.timeline_step, 2);
    let (output, _) = frame(&ctx, &mut app, 900.0, Some(&selected), vec![]);
    click(&ctx, &mut app, &selected, &output, "▶ Play");
    assert!(app.playing && !app.follow_live);
    assert_eq!(app.play_time.to_bits(), selected.time.to_bits());
    let (output, _) = frame(&ctx, &mut app, 900.0, Some(&selected), vec![]);
    click(&ctx, &mut app, &selected, &output, "❚❚ Pause");
    assert!(!app.playing);
    let (output, _) = frame(&ctx, &mut app, 900.0, Some(&selected), vec![]);
    click(&ctx, &mut app, &selected, &output, "Follow live");
    assert!(app.follow_live && !app.playing);
    assert_eq!(app.playback_speed, 1.0);
}

// Pure values only. No Worker, history, mesh, file, process or numerical owner.
fn inert_carrier_request() -> Request {
    Request {
        force_fields: Vec::new(),
        #[cfg(test)]
        force_validation: false,
        collision_mesh: None,
        size: 4,
        steps: 4,
        tight: false,
        threads: 1,
        methods: PressureImplementation::ALL.to_vec(),
        emitter: None,
        observe: false,
        history_budget_mib: 16,
        preview_resolution: history::PreviewResolution::Adaptive,
        domain_side: 1.0,
        requested_dt: 0.01,
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
    }
}
fn presentation() -> PreviewFrame {
    PreviewFrame {
        method: PressureImplementation::JacobiPcgV1,
        step: 2,
        time: 0.04,
        seconds: 0.002,
        edge: 1,
        nonzero_tracer_cells: 0,
        density: vec![],
        report: None,
        particle_report: None,
        material_color: None,
    }
}
fn key(key: egui::Key, modifiers: egui::Modifiers) -> Vec<egui::Event> {
    [true, false]
        .map(|pressed| egui::Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers,
        })
        .into()
}
fn replace_speed(ctx: &egui::Context, app: &mut App, selected: &PreviewFrame, text: &str) {
    let _ = frame(
        ctx,
        app,
        900.0,
        Some(selected),
        key(
            egui::Key::A,
            egui::Modifiers {
                ctrl: true,
                command: true,
                ..Default::default()
            },
        ),
    );
    let _ = frame(
        ctx,
        app,
        900.0,
        Some(selected),
        vec![egui::Event::Text(text.into())],
    );
}

#[test]
fn timeline_speed_edit_focus_bounds_validation_and_keyboard() {
    let ctx = context();
    let selected = presentation();
    let mut app = App::default();
    app.running_request = Some(inert_carrier_request());
    app.playback_speed = 1.2345678901234567;
    app.playback_speed_text = app.playback_speed.to_string();
    let before = app.playback_speed.to_bits();
    let (output, rect) = frame(&ctx, &mut app, 900.0, Some(&selected), vec![]);
    assert_visible_with_editor(&output, rect, Some(&app.playback_speed_text));
    click(&ctx, &mut app, &selected, &output, "1.2345678901234567");
    let (focused, panel) = frame(&ctx, &mut app, 900.0, Some(&selected), vec![]);
    assert_visible_with_editor(&focused, panel, Some(&app.playback_speed_text));
    let tree = focused.platform_output.accesskit_update.as_ref().unwrap();
    let node = &tree
        .nodes
        .iter()
        .find(|(id, _)| *id == tree.focus)
        .unwrap()
        .1;
    assert_eq!(node.role(), egui::accesskit::Role::TextInput);
    assert!(!node.labelled_by().is_empty());
    assert_eq!(app.playback_speed.to_bits(), before);
    for _ in 0..2 {
        let _ = frame(
            &ctx,
            &mut app,
            900.0,
            Some(&selected),
            key(egui::Key::Tab, Default::default()),
        );
    }
    assert_eq!(app.playback_speed.to_bits(), before);
    let (output, _) = frame(&ctx, &mut app, 900.0, Some(&selected), vec![]);
    click(&ctx, &mut app, &selected, &output, "1.2345678901234567");
    replace_speed(&ctx, &mut app, &selected, "2.5");
    assert_eq!(app.playback_speed, 2.5);
    let _ = frame(
        &ctx,
        &mut app,
        900.0,
        Some(&selected),
        key(egui::Key::ArrowUp, Default::default()),
    );
    assert_eq!(app.playback_speed, 2.5 + 0.1);
    let kept = app.playback_speed.to_bits();
    for text in ["NaN".to_owned(), "x".repeat(4096)] {
        replace_speed(&ctx, &mut app, &selected, &text);
        assert_eq!(app.playback_speed.to_bits(), kept);
        assert!(app.playback_speed_invalid);
        let (output, rect) = frame(&ctx, &mut app, 900.0, Some(&selected), vec![]);
        assert_visible_with_editor(&output, rect, Some(&text));
        assert!(text_rect(&output, "Invalid speed").is_positive());
    }
    for (input, expected) in [("9", 4.0), ("0.01", 0.1), ("1.5", 1.5)] {
        replace_speed(&ctx, &mut app, &selected, input);
        assert_eq!(app.playback_speed, expected);
        assert!(!app.playback_speed_invalid);
    }
    let (output, rect) = frame(&ctx, &mut app, 900.0, Some(&selected), vec![]);
    assert_visible(&output, rect);
}

#[test]
fn timeline_track_selection_and_report_rows_fit_cold_and_changed_states() {
    for width in [900.0, 1250.0] {
        let ctx = context();
        let mut app = App::default();
        app.running_request = Some(inert_carrier_request());
        let mut selected = presentation();
        selected.report = Some(rheon::StepReport {
            dt: 0.01,
            time: 0.04,
            generation: 2,
            pressure: rheon::PressureReport {
                iterations: usize::MAX,
                true_residual_l2: 0.0,
                true_residual_max: 0.0,
                predicted_divergence_max: 0.0,
            },
            actual_divergence_max: 1.25e-5,
            courant: 0.2,
            tracer_integral: 0.0,
            kinetic_energy: 0.0,
        });
        selected.particle_report = Some(rheon::ParticleStepReport {
            dt: 0.01,
            particles: usize::MAX,
            emitted_mass: 0.0,
            total_mass: 1.234567,
            max_speed: 0.0,
            max_pressure: 0.0,
            mean_temperature_kelvin: 293.15,
        });
        let (output, rect) = frame(&ctx, &mut app, width, Some(&selected), vec![]);
        assert_visible(&output, rect);
        assert!(
            text_rect(
                &output,
                &format!("{} particles · 1.234567 kg · 20.0 °C", usize::MAX)
            )
            .is_positive()
        );
        // No popup geometry is compared with the panel; popups are separate layers.
        if width == 900.0 {
            click(&ctx, &mut app, &selected, &output, "jacobi-pcg-v1");
            let (output, _) = frame(&ctx, &mut app, width, Some(&selected), vec![]);
            click(&ctx, &mut app, &selected, &output, "sgs-pcg-v1");
            assert_eq!(app.track, 1);
            assert!(!app.follow_live && !app.playing);
            assert_eq!(app.timeline_step, 0);
        }
        let (output, rect) = frame(&ctx, &mut app, width, Some(&selected), vec![]);
        assert_visible(&output, rect);
        selected.report = None;
        selected.particle_report = None;
        let (output, rect) = frame(&ctx, &mut app, width, Some(&selected), vec![]);
        assert_visible(&output, rect);
    }
}
