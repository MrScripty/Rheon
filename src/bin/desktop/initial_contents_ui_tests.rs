//! Real egui press/move/release inputs at the UI owner; no native display claim.
use super::*;
struct Harness {
    ctx: egui::Context,
    tool: InitialContentsUi,
    camera: OrbitCamera,
    rect: Rect,
    selected: bool,
    editable: bool,
    focused: bool,
    width: f32,
}
impl Harness {
    fn new() -> Self {
        let mut tool = InitialContentsUi::default();
        tool.create(
            SceneObjectId(3),
            GridGeometry::new([4; 3], [0.25; 3], [0.0; 3]).unwrap(),
        )
        .unwrap();
        tool.editor.as_mut().unwrap().fill_air().unwrap();
        let mut h = Self {
            ctx: egui::Context::default(),
            tool,
            camera: OrbitCamera {
                yaw: 0.0,
                pitch: 0.0,
                orthographic: true,
                ..Default::default()
            },
            rect: Rect::NOTHING,
            selected: true,
            editable: true,
            focused: true,
            width: 600.0,
        };
        h.frame(Vec::new());
        h
    }
    fn frame(&mut self, events: Vec<egui::Event>) -> bool {
        let mut consumed = false;
        let _ = self.ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    Pos2::ZERO,
                    egui::vec2(self.width, 600.0),
                )),
                events,
                focused: self.focused,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let (rect, response) =
                        ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
                    self.rect = rect;
                    consumed = self.tool.canvas(
                        ui,
                        &response,
                        &self.camera,
                        1.0,
                        self.selected,
                        self.editable,
                    );
                });
            },
        );
        consumed
    }
    fn point(&self, x: f64, y: f64) -> Pos2 {
        self.camera
            .project([x as f32, y as f32, 0.625], self.rect)
            .unwrap()
            .0
    }
    fn press(&mut self, p: Pos2) {
        assert!(self.frame(vec![egui::Event::PointerMoved(p), button(p, true)]));
    }
    fn release(&mut self, p: Pos2) {
        assert!(self.frame(vec![button(p, false)]));
    }
    fn cells(&self) -> Vec<u16> {
        self.tool
            .editor
            .as_ref()
            .unwrap()
            .document()
            .cells()
            .to_vec()
    }
}
fn button(pos: Pos2, pressed: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    }
}
fn key(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    }
}
#[test]
fn owned_pointer_stroke_is_one_command_and_keyboard_dab_is_identical() {
    let mut h = Harness::new();
    let p = h.point(0.5, 0.5);
    let before = h.cells();
    let rev = h.tool.editor.as_ref().unwrap().document().revision();
    h.press(p);
    assert_eq!(h.cells(), before);
    assert!(h.tool.editor.as_ref().unwrap().active());
    h.release(p);
    let expected = h.cells();
    assert_ne!(expected, before);
    assert_eq!(
        h.tool.editor.as_ref().unwrap().document().revision(),
        rev + 1
    );
    h.tool.command(Action::Undo);
    assert_eq!(h.cells(), before);
    h.tool.keyboard = [0.5; 2];
    h.tool.apply_dab();
    assert_eq!(h.cells(), expected);
}
#[test]
fn coalesced_press_move_release_and_partitioned_frames_have_same_result() {
    let mut single = Harness::new();
    let a = single.point(0.25, 0.5);
    let b = single.point(0.75, 0.5);
    assert!(single.frame(vec![
        egui::Event::PointerMoved(a),
        button(a, true),
        egui::Event::PointerMoved(b),
        button(b, false)
    ]));
    let mut multiple = Harness::new();
    multiple.press(a);
    assert!(multiple.frame(vec![egui::Event::PointerMoved(b)]));
    multiple.release(b);
    assert_eq!(single.cells(), multiple.cells());
    assert_eq!(
        single.tool.editor.as_ref().unwrap().document().revision(),
        2
    );
}
#[test]
fn interruptions_cancel_and_latch_until_release_without_edits() {
    for cause in 0..7 {
        let mut h = Harness::new();
        let p = h.point(0.5, 0.5);
        let before = h.cells();
        h.press(p);
        let event = match cause {
            0 => {
                h.focused = false;
                Vec::new()
            }
            1 => vec![egui::Event::PointerGone],
            2 => {
                h.selected = false;
                Vec::new()
            }
            3 => {
                h.editable = false;
                Vec::new()
            }
            4 => {
                h.camera.yaw += 0.2;
                Vec::new()
            }
            5 => {
                h.width = 610.0;
                Vec::new()
            }
            _ => vec![egui::Event::PointerMoved(Pos2::new(-50.0, -50.0))],
        };
        assert!(h.frame(event), "cause {cause}");
        assert_eq!(h.cells(), before);
        assert!(!h.tool.editor.as_ref().unwrap().active());
        h.focused = true;
        h.selected = true;
        h.editable = true;
        assert!(h.frame(vec![egui::Event::PointerMoved(p)]));
        h.release(p);
        assert_eq!(h.cells(), before);
        assert!(!h.tool.busy());
    }
}
#[test]
fn invalid_terminal_release_discards_whole_candidate() {
    let mut h = Harness::new();
    let p = h.point(0.5, 0.5);
    let before = h.cells();
    h.press(p);
    h.release(Pos2::new(-10.0, 50.0));
    assert_eq!(h.cells(), before);
    assert!(!h.tool.busy());
}
#[test]
fn escape_and_first_undo_cancel_then_second_undo_routes_to_contents() {
    for action in [egui::Key::Escape, egui::Key::Z] {
        let mut h = Harness::new();
        let p = h.point(0.5, 0.5);
        let initial = h.cells();
        h.press(p);
        h.release(p);
        let painted = h.cells();
        h.press(p);
        let modifiers = if action == egui::Key::Z {
            egui::Modifiers::COMMAND
        } else {
            egui::Modifiers::NONE
        };
        assert!(h.frame(vec![key(action, modifiers)]));
        assert_eq!(h.cells(), painted);
        h.release(p);
        assert!(h.frame(vec![key(egui::Key::Z, egui::Modifiers::COMMAND)]));
        assert_eq!(h.cells(), initial);
        assert!(h.frame(vec![key(
            egui::Key::Z,
            egui::Modifiers::COMMAND | egui::Modifiers::SHIFT
        )]));
        assert_eq!(h.cells(), painted);
    }
}
#[test]
fn retained_or_force_selection_does_not_route_contents_undo() {
    let mut h = Harness::new();
    let p = h.point(0.5, 0.5);
    h.press(p);
    h.release(p);
    let before = h.cells();
    h.selected = false;
    assert!(!h.frame(vec![key(egui::Key::Z, egui::Modifiers::COMMAND)]));
    assert_eq!(h.cells(), before);
    h.selected = true;
    h.editable = false;
    assert!(!h.frame(vec![key(egui::Key::Z, egui::Modifiers::COMMAND)]));
    assert_eq!(h.cells(), before);
}

#[test]
fn text_editor_keeps_local_undo_and_hidden_page_cancels_owned_stroke() {
    let ctx = egui::Context::default();
    let mut text = String::from("Draft name");
    let mut routed = Some(Action::Undo);
    let _ = ctx.run(
        egui::RawInput {
            events: vec![key(egui::Key::Z, egui::Modifiers::COMMAND)],
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let id = ui.make_persistent_id("contents-text-priority");
                ui.memory_mut(|m| m.request_focus(id));
                ui.add(egui::TextEdit::singleline(&mut text).id(id));
                routed = authoring_history::action(ui, true);
            });
        },
    );
    assert!(routed.is_none());
    let mut h = Harness::new();
    let p = h.point(0.5, 0.5);
    let before = h.cells();
    h.press(p);
    h.tool.hidden(&h.ctx);
    assert!(!h.tool.editor.as_ref().unwrap().active());
    assert!(h.tool.busy());
    h.release(p);
    assert_eq!(h.cells(), before);
    assert!(!h.tool.busy());
}
#[test]
fn canonical_selection_has_one_contents_identity_and_cannot_launch_legacy_run() {
    use super::super::force_run_policy::Target;
    use super::super::scene_ui::SceneUi;
    let mut scene = SceneUi::default();
    scene
        .create_contents(GridGeometry::new([4; 3], [0.25; 3], [0.0; 3]).unwrap())
        .unwrap();
    let id = scene.contents.id().unwrap();
    assert!(scene.contents_selected());
    assert!(scene.selected_force().is_none());
    assert!(
        scene
            .launch_readiness(Target::Unsupported)
            .unwrap_err()
            .contains("cannot launch a legacy method")
    );
    scene.replace_model(true).unwrap();
    assert_ne!(scene.model_id(), Some(id));
    scene.select(scene.model_id());
    assert!(!scene.contents_selected());
    scene.select(Some(id));
    assert!(scene.contents_selected());
    scene.begin_run(false);
    assert!(!scene.contents_selected());
    scene.select(Some(id));
    assert_eq!(scene.selected(), None);
}

#[test]
fn cancelled_batches_keep_ownership_through_release_then_new_held_press() {
    for cause in 0..3 {
        let mut h = Harness::new();
        let p = h.point(0.5, 0.5);
        let before = h.cells();
        h.press(p);
        let revision = h.tool.editor.as_ref().unwrap().document().revision();
        let history = h.tool.editor.as_ref().unwrap().history_bytes();
        if cause == 0 {
            assert!(h.frame(vec![
                egui::Event::PointerMoved(Pos2::new(-10.0, 0.0)),
                button(p, false),
                button(p, true)
            ]));
        } else {
            assert!(h.frame(vec![key(egui::Key::Escape, egui::Modifiers::NONE)]));
            if cause == 1 {
                assert!(h.frame(vec![button(p, false), button(p, true)]));
            } else {
                let _ = h.ctx.run(
                    egui::RawInput {
                        events: vec![button(p, false), button(p, true)],
                        ..Default::default()
                    },
                    |ctx| h.tool.hidden(ctx),
                );
            }
        }
        assert!(h.tool.busy());
        assert!(!h.tool.editor.as_ref().unwrap().active());
        h.tool.command(Action::Undo); // cancelled held tail cannot undo older work
        assert!(h.frame(vec![egui::Event::PointerMoved(p + egui::vec2(25.0, 0.0))]));
        assert_eq!(h.cells(), before);
        assert_eq!(
            h.tool.editor.as_ref().unwrap().document().revision(),
            revision
        );
        assert_eq!(h.tool.editor.as_ref().unwrap().history_bytes(), history);
        h.release(p);
        assert!(!h.tool.busy());
        assert_eq!(h.cells(), before);
    }
}

fn panel_frame(
    ctx: &egui::Context,
    tool: &mut InitialContentsUi,
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(340.0, 1100.0))),
            events,
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| tool.controls(ui, true));
        },
    )
}
fn named_numeric_id(output: &egui::FullOutput, label: &str) -> egui::accesskit::NodeId {
    let tree = output.platform_output.accesskit_update.as_ref().unwrap();
    let label_id = tree
        .nodes
        .iter()
        .find(|(_, node)| node.value() == Some(label))
        .unwrap_or_else(|| panic!("missing {label}"))
        .0;
    let (id, node) = tree
        .nodes
        .iter()
        .find(|(_, node)| node.labelled_by().contains(&label_id))
        .unwrap_or_else(|| panic!("unnamed input {label}"));
    assert!(matches!(
        node.role(),
        egui::accesskit::Role::SpinButton | egui::accesskit::Role::TextInput
    ));
    *id
}
fn tab_to(ctx: &egui::Context, tool: &mut InitialContentsUi, id: egui::accesskit::NodeId) {
    for _ in 0..40 {
        if ctx
            .memory(|m| m.focused())
            .is_some_and(|focus| focus.value() == id.0)
        {
            return;
        }
        panel_frame(
            ctx,
            tool,
            vec![
                key(egui::Key::Tab, egui::Modifiers::NONE),
                egui::Event::Key {
                    key: egui::Key::Tab,
                    physical_key: None,
                    pressed: false,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
    }
    panic!("named control not keyboard reachable: {id:?}");
}
#[test]
fn named_controls_accept_keyboard_coordinates_and_activate_the_actual_dab_button() {
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    ctx.style_mut(|s| s.animation_time = 0.0);
    let mut tool = Harness::new().tool;
    let output = panel_frame(&ctx, &mut tool, Vec::new());
    let details = output
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            egui::Shape::Text(t) if t.galley.text() == "Details" => {
                Some(Rect::from_min_size(t.pos, t.galley.size()).center())
            }
            _ => None,
        })
        .unwrap();
    panel_frame(
        &ctx,
        &mut tool,
        vec![egui::Event::PointerMoved(details), button(details, true)],
    );
    let output = panel_frame(&ctx, &mut tool, vec![button(details, false)]);
    let fields = [
        ("Radius · m", "0.2"),
        ("Center X · m", "0.4"),
        ("Center Y · m", "0.6"),
    ];
    let ids = fields.map(|(label, _)| named_numeric_id(&output, label));
    for ((_, value), id) in fields.into_iter().zip(ids) {
        tab_to(&ctx, &mut tool, id);
        panel_frame(
            &ctx,
            &mut tool,
            vec![key(
                egui::Key::A,
                egui::Modifiers {
                    ctrl: true,
                    command: true,
                    ..Default::default()
                },
            )],
        );
        panel_frame(&ctx, &mut tool, vec![egui::Event::Text(value.into())]);
        panel_frame(
            &ctx,
            &mut tool,
            vec![key(egui::Key::Enter, egui::Modifiers::NONE)],
        );
    }
    assert_eq!(tool.brush.radius, 0.2);
    assert_eq!(tool.keyboard, [0.4, 0.6]);
    let output = panel_frame(&ctx, &mut tool, Vec::new());
    let apply = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some("Apply dab"))
        .unwrap()
        .0;
    tab_to(&ctx, &mut tool, apply);
    panel_frame(
        &ctx,
        &mut tool,
        vec![key(egui::Key::Space, egui::Modifiers::NONE)],
    );
    assert_eq!(tool.editor.as_ref().unwrap().document().revision(), 2);
    let mut pointer = Harness::new();
    pointer.tool.brush.radius = 0.2;
    let pos = pointer.point(0.4, 0.6);
    pointer.press(pos);
    pointer.release(pos);
    assert_eq!(
        tool.editor.as_ref().unwrap().document().cells(),
        pointer.cells()
    );
}
