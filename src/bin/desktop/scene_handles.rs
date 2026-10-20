//! Draft-only geometric manipulators. No solver or field evaluator calls.
use super::{
    camera::OrbitCamera,
    scene_ui::SceneUi,
    scene_view::{Line, line_hit, normalized, rotate, segment},
};
use eframe::egui::{self, Color32, Pos2, Rect, Stroke};
use rheon::{ForceObject, ForceSupport};
type V = [f64; 3];
fn add(a: V, b: V) -> V {
    std::array::from_fn(|d| a[d] + b[d])
}
fn sub(a: V, b: V) -> V {
    std::array::from_fn(|d| a[d] - b[d])
}
fn mul(a: V, s: f64) -> V {
    a.map(|v| v * s)
}
fn dot(a: V, b: V) -> f64 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}
fn cross(a: V, b: V) -> V {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn unit(a: V) -> Option<V> {
    let n = dot(a, a).sqrt();
    (n.is_finite() && n > 1e-12).then(|| mul(a, 1.0 / n))
}
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum Tool {
    #[default]
    Translate,
    Rotate,
    Support,
}
impl Tool {
    fn label(self) -> &'static str {
        match self {
            Self::Translate => "Move",
            Self::Rotate => "Rotate",
            Self::Support => "Size",
        }
    }
}
#[derive(Clone, Copy)]
struct Ray {
    near_parameter: f64,
    origin: V,
    direction: V,
}
fn ray(camera: &OrbitCamera, rect: Rect, p: Pos2) -> Option<Ray> {
    if !p.is_finite()
        || !rect.is_finite()
        || rect.height() <= 0.0
        || !camera.distance.is_finite()
        || camera.distance <= 0.0
    {
        return None;
    }
    let (eye, forward, right, up) = camera.basis();
    let eye = eye.map(f64::from);
    let f = forward.map(f64::from);
    let r = right.map(f64::from);
    let u = up.map(f64::from);
    let dx = f64::from(p.x - rect.center().x) / f64::from(rect.height() * 0.88);
    let dy = f64::from(rect.center().y - p.y) / f64::from(rect.height() * 0.88);
    let (origin, direction) = if camera.orthographic {
        (
            add(
                eye,
                mul(add(mul(r, dx), mul(u, dy)), f64::from(camera.distance)),
            ),
            f,
        )
    } else {
        (eye, add(f, add(mul(r, dx), mul(u, dy))))
    };
    let direction = unit(direction)?;
    let near_parameter = 0.0151 / dot(f, direction);
    (near_parameter.is_finite() && near_parameter > 0.0 && origin.iter().all(|v| v.is_finite()))
        .then_some(Ray {
            near_parameter,
            origin,
            direction,
        })
}
fn axis_parameter(ray: Ray, center: V, axis: V) -> Option<f64> {
    let b = dot(axis, ray.direction);
    let denominator = 1.0 - b * b;
    if denominator < 1e-6 {
        return None;
    }
    let w = sub(center, ray.origin);
    let t = (b * dot(ray.direction, w) - dot(axis, w)) / denominator;
    (t.is_finite() && dot(ray.direction, w) + b * t >= ray.near_parameter).then_some(t)
}
fn plane_vector(ray: Ray, center: V, axis: V) -> Option<V> {
    let denominator = dot(ray.direction, axis);
    if denominator.abs() < 1e-6 {
        return None;
    }
    let t = dot(sub(center, ray.origin), axis) / denominator;
    if !t.is_finite() || t < ray.near_parameter {
        return None;
    }
    unit(sub(add(ray.origin, mul(ray.direction, t)), center))
}
fn quaternion_product([x, y, z, w]: [f64; 4], [a, b, c, d]: [f64; 4]) -> [f64; 4] {
    [
        w * a + x * d + y * c - z * b,
        w * b - x * c + y * d + z * a,
        w * c + x * b - y * a + z * d,
        w * d - x * a - y * b - z * c,
    ]
}
fn axis(object: &ForceObject, tool: Tool, d: usize) -> V {
    let mut a = [0.0; 3];
    a[d] = 1.0;
    if tool == Tool::Translate {
        a
    } else {
        rotate(object.transform.rotation, a)
    }
}
struct Handle {
    axis: usize,
    lines: Vec<Line>,
    endpoint: Option<Pos2>,
}
fn handles(
    object: &ForceObject,
    tool: Tool,
    camera: &OrbitCamera,
    rect: Rect,
    side: f64,
) -> Vec<Handle> {
    let mut result = vec![];
    let center = object.transform.translation;
    let count = if tool == Tool::Support {
        match object.support {
            ForceSupport::Global => 0,
            ForceSupport::Sphere { .. } => 1,
            ForceSupport::Box { .. } => 3,
        }
    } else {
        3
    };
    for d in 0..count {
        let a = axis(object, tool, d);
        let mut lines = vec![];
        let mut line = |a: V, b: V| {
            if let (Some(a), Some(b)) = (normalized(a, side), normalized(b, side))
                && let Some(l) = segment(camera, rect, a, b)
            {
                lines.push(l);
            }
        };
        let endpoint = if tool == Tool::Rotate {
            let b = axis(object, tool, (d + 1) % 3);
            let c = axis(object, tool, (d + 2) % 3);
            let point = |i: usize| {
                let t = i as f64 * std::f64::consts::TAU / 48.0;
                add(
                    center,
                    mul(add(mul(b, t.cos()), mul(c, t.sin())), side * 0.15),
                )
            };
            for i in 0..48 {
                line(point(i), point(i + 1));
            }
            normalized(point(0), side)
                .and_then(|p| camera.project(p, rect))
                .map(|p| p.0)
                .filter(|p| p.is_finite() && rect.contains(*p))
        } else {
            let length = if tool == Tool::Support {
                match object.support {
                    ForceSupport::Sphere { radius, .. } => radius,
                    ForceSupport::Box { half_extent, .. } => half_extent[d],
                    ForceSupport::Global => 0.0,
                }
            } else {
                side * 0.18
            };
            let end = add(center, mul(a, length));
            line(center, end);
            normalized(end, side)
                .and_then(|p| camera.project(p, rect))
                .map(|p| p.0)
                .filter(|p| p.is_finite() && rect.contains(*p))
        };
        result.push(Handle {
            axis: d,
            lines,
            endpoint,
        });
    }
    result
}
fn hit(handles: &[Handle], tool: Tool, p: Pos2, ortho: bool) -> Option<usize> {
    handles
        .iter()
        .filter_map(|h| {
            let distance = if tool == Tool::Rotate {
                h.lines
                    .iter()
                    .filter_map(|&l| line_hit(l, p, ortho).map(|v| v.0))
                    .min_by(f32::total_cmp)
            } else {
                h.endpoint.map(|e| e.distance(p)).filter(|v| *v <= 9.0)
            }?;
            Some((h.axis, distance))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)))
        .map(|h| h.0)
}
struct Drag {
    start: Pos2,
    base: ForceObject,
    tool: Tool,
    axis: V,
    index: usize,
    initial: f64,
    initial_vector: Option<V>,
    camera: OrbitCamera,
    rect: Rect,
    side: f64,
}
impl Drag {
    fn new(
        base: ForceObject,
        tool: Tool,
        index: usize,
        camera: &OrbitCamera,
        rect: Rect,
        side: f64,
        p: Pos2,
    ) -> Option<Self> {
        if !side.is_finite() || side <= 0.0 {
            return None;
        }
        let axis = unit(axis(&base, tool, index))?;
        let center = base.transform.translation.map(|v| v / side);
        let r = ray(camera, rect, p)?;
        let (initial, initial_vector) = if tool == Tool::Rotate {
            (0.0, Some(plane_vector(r, center, axis)?))
        } else {
            (axis_parameter(r, center, axis)?, None)
        };
        Some(Self {
            start: p,
            base,
            tool,
            axis,
            index,
            initial,
            initial_vector,
            camera: camera.clone(),
            rect,
            side,
        })
    }
    fn candidate(&self, p: Pos2) -> Option<ForceObject> {
        if p == self.start {
            return Some(self.base.clone());
        }
        let mut candidate = self.base.clone();
        let center = self.base.transform.translation.map(|v| v / self.side);
        let r = ray(&self.camera, self.rect, p)?;
        if self.tool == Tool::Rotate {
            let v = plane_vector(r, center, self.axis)?;
            let start = self.initial_vector?;
            let angle = dot(self.axis, cross(start, v)).atan2(dot(start, v));
            if angle == 0.0 {
                return Some(self.base.clone());
            }
            let half = angle * 0.5;
            let q = [
                self.axis[0] * half.sin(),
                self.axis[1] * half.sin(),
                self.axis[2] * half.sin(),
                half.cos(),
            ];
            candidate.transform.rotation = quaternion_product(q, self.base.transform.rotation);
        } else {
            let delta = (axis_parameter(r, center, self.axis)? - self.initial) * self.side;
            if delta == 0.0 {
                return Some(self.base.clone());
            }
            if self.tool == Tool::Translate {
                candidate.transform.translation[self.index] =
                    self.base.transform.translation[self.index] + delta;
            } else {
                match &mut candidate.support {
                    ForceSupport::Sphere { radius, .. } => *radius += delta,
                    ForceSupport::Box { half_extent, .. } => half_extent[self.index] += delta,
                    ForceSupport::Global => return None,
                }
            }
        }
        Some(candidate)
    }
}
#[derive(Default)]
pub struct SceneHandles {
    tool: Tool,
    drag: Option<Drag>,
    latch: bool,
    message: Option<&'static str>,
}
impl SceneHandles {
    pub fn busy(&self) -> bool {
        self.drag.is_some() || self.latch
    }
    pub fn controls(&mut self, ui: &mut egui::Ui, scene: &SceneUi, editable: bool) {
        if scene.selected_force().is_none() && !self.latch {
            return;
        }
        ui.horizontal_wrapped(|ui| {
            for tool in [Tool::Translate,Tool::Rotate,Tool::Support] {
                let available=tool!=Tool::Support||scene.selected_force().is_some_and(|o|o.support!=ForceSupport::Global);
                let response=ui.add_enabled(editable&&!scene.editing()&&!self.latch&&available,
                    egui::Button::selectable(self.tool==tool,tool.label()).wrap_mode(egui::TextWrapMode::Extend));
                if response.clicked(){self.tool=tool;}
                if !available {response.on_disabled_hover_text("Global support has no boundary to resize. Choose Sphere or Box in properties.");}
            }
            ui.add(egui::Label::new(if self.tool==Tool::Translate {"World axes · m"}else if self.tool==Tool::Rotate {"Local rotation"}else{"Local size · m"}).wrap_mode(egui::TextWrapMode::Extend))
                .on_hover_text("Drag X/Y/Z. Numeric properties provide keyboard editing. Moving a Global Directional force changes its anchor only; its field remains uniform.");
        });
        let text = self.message.unwrap_or("Release applies · Esc cancels");
        let height = ui.text_style_height(&egui::TextStyle::Small) * 2.0;
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.add(
                    egui::Label::new(egui::RichText::new(text).small().color(
                        if self.message.is_some() {
                            Color32::YELLOW
                        } else {
                            Color32::GRAY
                        },
                    ))
                    .wrap(),
                );
            },
        );
    }
    fn cancel(&mut self, scene: &mut SceneUi, down: bool, message: &'static str) {
        scene.finish_edit(false);
        self.drag = None;
        self.latch = down;
        self.message = Some(message);
    }
    /// True consumes this entire frame, including terminal release/cancellation.
    pub fn update(
        &mut self,
        ui: &egui::Ui,
        response: &egui::Response,
        camera: &OrbitCamera,
        side: f64,
        scene: &mut SceneUi,
        editable: bool,
    ) -> bool {
        let (down, pressed, released, p, focused, gone) = ui.input(|i| {
            (
                i.pointer.button_down(egui::PointerButton::Primary),
                i.pointer.button_pressed(egui::PointerButton::Primary),
                i.events.iter().any(|e| {
                    matches!(
                        e,
                        egui::Event::PointerButton {
                            button: egui::PointerButton::Primary,
                            pressed: false,
                            ..
                        }
                    )
                }),
                i.pointer.latest_pos(),
                i.focused,
                i.events
                    .iter()
                    .any(|e| matches!(e, egui::Event::PointerGone)),
            )
        });
        if self.latch {
            if released {
                self.latch = false;
            }
            return true;
        }
        if let Some(drag) = &self.drag {
            let lost = !editable
                || !focused
                || (!down && !released)
                || response.lost_focus()
                || gone
                || p.is_none()
                || !scene.edit_matches()
                || *camera != drag.camera
                || response.rect != drag.rect
                || side.to_bits() != drag.side.to_bits();
            let escape = ui.input_mut(|i| i.consume_key(i.modifiers, egui::Key::Escape));
            if lost || escape {
                self.cancel(
                    scene,
                    !released,
                    if escape {
                        "Edit cancelled"
                    } else {
                        "Edit cancelled · context changed"
                    },
                );
                return true;
            }
            let valid = scene.preview_edit(drag.candidate(p.unwrap()));
            self.message = if valid {
                None
            } else {
                Some("Invalid preview · release cancels")
            };
            if released || !down {
                let committed = scene.finish_edit(valid && released);
                self.drag = None;
                self.message = if committed {
                    None
                } else {
                    Some("Edit cancelled")
                };
            }
            return true;
        }
        if !editable
            || !pressed
            || !focused
            || !response.contains_pointer()
            || !ui.input(|i| i.modifiers.is_none())
        {
            return false;
        }
        if scene.display_scene().freeze().is_err() {
            return false;
        }
        let Some(p) = p else {
            return false;
        };
        let Some(base) = scene.selected_force().cloned() else {
            return false;
        };
        let Some(index) = hit(
            &handles(&base, self.tool, camera, response.rect, side),
            self.tool,
            p,
            camera.orthographic,
        ) else {
            return false;
        };
        response.request_focus();
        if !scene.begin_edit() {
            self.latch = down;
            self.message = Some("Fix invalid draft before dragging");
            return true;
        }
        self.drag = Drag::new(base, self.tool, index, camera, response.rect, side, p);
        if self.drag.is_none() {
            self.cancel(
                scene,
                down,
                "Handle edge-on · change view or use properties",
            );
        } else {
            self.message = None;
            // Native input may coalesce a short press and release in one frame.
            // A handle click still owns that frame and preserves the exact base.
            if released {
                scene.finish_edit(true);
                self.drag = None;
            }
        }
        true
    }
    pub fn paint(
        &self,
        ui: &egui::Ui,
        rect: Rect,
        camera: &OrbitCamera,
        side: f64,
        scene: &SceneUi,
        editable: bool,
    ) {
        if !editable {
            return;
        }
        let display = scene.display_scene();
        let Some(object) = display
            .objects
            .iter()
            .find(|o| Some(o.id) == scene.selected())
        else {
            return;
        };
        if display.freeze().is_err() {
            return;
        }
        let painter = ui.painter_at(rect);
        for h in handles(object, self.tool, camera, rect, side) {
            let color = [
                Color32::from_rgb(255, 130, 130),
                Color32::from_rgb(145, 230, 145),
                Color32::from_rgb(140, 175, 255),
            ][h.axis];
            for l in h.lines {
                painter.line_segment([l.a.pos, l.b.pos], Stroke::new(2.0, color));
            }
            if let Some(p) = h.endpoint {
                painter.circle_filled(p, 4.0, color);
                let label = ["X", "Y", "Z"][h.axis];
                let galley =
                    painter.layout_no_wrap(label.into(), egui::FontId::proportional(12.0), color);
                let origin = egui::pos2(
                    (p.x + 7.0).clamp(
                        rect.left(),
                        (rect.right() - galley.size().x).max(rect.left()),
                    ),
                    (p.y - 7.0 - galley.size().y).clamp(
                        rect.top(),
                        (rect.bottom() - galley.size().y).max(rect.top()),
                    ),
                );
                painter.galley(origin, galley, color);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rheon::{ForceKind, ForceScene, ForceTime, ForceTransform, ForceUnits, SceneObjectId};
    fn object() -> ForceObject {
        ForceObject {
            id: SceneObjectId(1),
            name: "Test force".into(),
            enabled: true,
            transform: ForceTransform {
                translation: [0.5; 3],
                ..Default::default()
            },
            units: ForceUnits::Acceleration,
            kind: ForceKind::Directional { vector: [0.0; 3] },
            support: ForceSupport::Sphere {
                radius: 0.1,
                edge_fraction: 0.2,
            },
            time: ForceTime::default(),
        }
    }
    fn scene() -> SceneUi {
        let mut s = SceneUi::geometry_fixture(
            ForceScene {
                version: 1,
                objects: vec![object()],
            },
            false,
        );
        s.select(Some(SceneObjectId(1)));
        s
    }
    fn rect() -> Rect {
        Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(600.0, 400.0))
    }
    fn screen(camera: &OrbitCamera, rect: Rect, p: V, side: f64) -> Pos2 {
        camera
            .project(p.map(|v| (v / side) as f32), rect)
            .unwrap()
            .0
    }
    #[test]
    fn handles_geometry_ray_translation_units_and_nonidentity_local_rotation() {
        let c = OrbitCamera {
            yaw: 0.0,
            pitch: 0.0,
            orthographic: true,
            ..Default::default()
        };
        let r = rect();
        let mut base = object();
        base.transform.translation[1] = -0.0;
        let p = screen(&c, r, [0.86, 0.0, 0.5], 2.0);
        let drag = Drag::new(base.clone(), Tool::Translate, 0, &c, r, 2.0, p).unwrap();
        let candidate = drag.candidate(p + egui::vec2(40.0, 0.0)).unwrap();
        let expected = 40.0 * f64::from(c.distance) / f64::from(r.height() * 0.88) * 2.0;
        assert!(
            (candidate.transform.translation[0] - base.transform.translation[0] - expected).abs()
                < 1e-8
        );
        assert_eq!(
            candidate.transform.translation.map(f64::to_bits)[1..],
            base.transform.translation.map(f64::to_bits)[1..]
        );
        assert_eq!(drag.candidate(p).unwrap().transform, base.transform);
        let parallel = Ray {
            near_parameter: 0.0151,
            origin: [0.0; 3],
            direction: [1.0, 0.0, 0.0],
        };
        assert!(axis_parameter(parallel, [1.0, 0.0, 0.0], [1.0, 0.0, 0.0]).is_none());
        assert!(plane_vector(parallel, [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]).is_none());
        let near = Ray {
            origin: [0.0; 3],
            direction: [0.0, 0.0, 1.0],
            near_parameter: 0.0151,
        };
        assert!(axis_parameter(near, [0.0, 0.0, 0.005], [1.0, 0.0, 0.0]).is_none());
        assert!(plane_vector(near, [0.0, 0.0, 0.005], [0.0, 0.0, 1.0]).is_none());
        let c = OrbitCamera::default();
        let mut base = object();
        let half = std::f64::consts::FRAC_PI_4;
        base.transform.rotation = [0.0, 0.0, half.sin(), half.cos()];
        let start = screen(&c, r, [0.35, 0.5, 0.5], 1.0);
        let end = screen(&c, r, [0.5, 0.5, 0.65], 1.0);
        let drag = Drag::new(base, Tool::Rotate, 0, &c, r, 1.0, start).unwrap();
        let candidate = drag.candidate(end).unwrap();
        let v = rotate(candidate.transform.rotation, [0.0, 1.0, 0.0]);
        assert!(
            v[0].abs() < 2e-6 && v[1].abs() < 2e-6 && (v[2] - 1.0).abs() < 2e-6,
            "{v:?}"
        );
        let mut box_object = object();
        box_object.transform.rotation = [0.0, 0.0, half.sin(), half.cos()];
        box_object.support = ForceSupport::Box {
            half_extent: [0.2, 0.3, 0.4],
            edge_fraction: 0.2,
        };
        let start = screen(&c, r, [0.5, 0.7, 0.5], 1.0);
        let end = screen(&c, r, [0.5, 0.75, 0.5], 1.0);
        let box_drag = Drag::new(box_object, Tool::Support, 0, &c, r, 1.0, start).unwrap();
        let boxed = box_drag.candidate(end).unwrap();
        if let ForceSupport::Box { half_extent, .. } = boxed.support {
            assert!((half_extent[0] - 0.25).abs() < 1e-6);
            assert_eq!(half_extent[1..], [0.3, 0.4]);
        } else {
            panic!("lost box support");
        }
        let mut s = scene();
        assert!(s.begin_edit());
        assert!(s.preview_edit(Some(candidate)));
        assert!(s.finish_edit(true));
    }
    #[test]
    fn handles_transaction_invalid_final_position_never_commits_stale_preview() {
        let mut s = scene();
        let frozen = s.draft().freeze().unwrap();
        let base = s.draft().clone();
        assert!(s.begin_edit());
        let mut candidate = object();
        candidate.support = ForceSupport::Sphere {
            radius: 0.2,
            edge_fraction: 0.2,
        };
        assert!(s.preview_edit(Some(candidate.clone())));
        assert_eq!(s.draft(), &base);
        assert_eq!(s.display_scene().objects[0], candidate);
        candidate.support = ForceSupport::Sphere {
            radius: -0.2,
            edge_fraction: 0.2,
        };
        assert!(!s.preview_edit(Some(candidate)));
        assert!(!s.finish_edit(true));
        assert_eq!(s.draft(), &base);
        assert_eq!(frozen.scene(), &base);
        assert!(s.begin_edit());
        let mut candidate = object();
        candidate.transform.translation[1] = 0.75;
        assert!(s.preview_edit(Some(candidate)));
        s.select(None);
        assert_eq!(s.display_scene().as_ref(), &base);
        assert!(!s.finish_edit(true));
        assert_eq!(s.draft(), &base);
    }
    struct Harness {
        ctx: egui::Context,
        scene: SceneUi,
        handles: SceneHandles,
        camera: OrbitCamera,
        rect: Rect,
        consumed: bool,
        canvas_size: egui::Vec2,
        side: f64,
    }
    impl Harness {
        fn new(tool: Tool) -> Self {
            let mut h = Self {
                ctx: egui::Context::default(),
                scene: scene(),
                handles: SceneHandles {
                    tool,
                    ..Default::default()
                },
                camera: OrbitCamera {
                    yaw: 0.0,
                    pitch: 0.0,
                    orthographic: true,
                    ..Default::default()
                },
                rect: Rect::NOTHING,
                consumed: false,
                canvas_size: egui::vec2(600.0, 400.0),
                side: 1.0,
            };
            h.frame(vec![], true, true, egui::Modifiers::NONE);
            h.frame(vec![], true, true, egui::Modifiers::NONE);
            h
        }
        fn frame(
            &mut self,
            events: Vec<egui::Event>,
            focused: bool,
            editable: bool,
            modifiers: egui::Modifiers,
        ) {
            let _ = self.ctx.run(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(700.0, 500.0),
                    )),
                    events,
                    focused,
                    modifiers,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let (rect, response) =
                            ui.allocate_exact_size(self.canvas_size, egui::Sense::click_and_drag());
                        self.rect = rect;
                        self.consumed = self.handles.update(
                            ui,
                            &response,
                            &self.camera,
                            self.side,
                            &mut self.scene,
                            editable,
                        );
                        self.handles.paint(
                            ui,
                            rect,
                            &self.camera,
                            self.side,
                            &self.scene,
                            editable,
                        );
                    });
                },
            );
        }
        fn point(&self, x: f64) -> Pos2 {
            screen(&self.camera, self.rect, [x, 0.5, 0.5], self.side)
        }
        fn button(&mut self, p: Pos2, pressed: bool, modifiers: egui::Modifiers) {
            self.frame(
                vec![
                    egui::Event::PointerMoved(p),
                    egui::Event::PointerButton {
                        pos: p,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers,
                    },
                ],
                true,
                true,
                modifiers,
            );
        }
        fn press(&mut self) -> Pos2 {
            let x = if self.handles.tool == Tool::Support {
                0.6
            } else {
                0.68
            };
            let p = self.point(x);
            self.frame(
                vec![egui::Event::PointerMoved(p)],
                true,
                true,
                egui::Modifiers::NONE,
            );
            self.button(p, true, egui::Modifiers::NONE);
            assert!(self.consumed && self.scene.editing());
            p
        }
    }
    #[test]
    fn handles_pointer_release_commits_once_and_escape_consumes_until_release() {
        let mut short = Harness::new(Tool::Translate);
        let p = short.point(0.68);
        let base = short.scene.draft().clone();
        short.frame(
            vec![
                egui::Event::PointerMoved(p),
                egui::Event::PointerButton {
                    pos: p,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
                egui::Event::PointerButton {
                    pos: p,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            true,
            true,
            egui::Modifiers::NONE,
        );
        assert!(short.consumed && !short.handles.busy() && !short.scene.editing());
        assert_eq!(short.scene.draft(), &base);
        let mut h = Harness::new(Tool::Translate);
        let p = h.press();
        let base = h.scene.draft().clone();
        let end = p + egui::vec2(30.0, 0.0);
        h.frame(
            vec![egui::Event::PointerMoved(end)],
            true,
            true,
            egui::Modifiers::NONE,
        );
        assert!(h.consumed);
        assert_eq!(h.scene.draft(), &base);
        h.button(end, false, egui::Modifiers::NONE);
        assert!(h.consumed && !h.scene.editing());
        assert!(h.scene.draft().objects[0].transform.translation[0] > 0.5);
        assert_eq!(h.scene.selected(), Some(SceneObjectId(1)));
        h.frame(vec![], true, true, egui::Modifiers::NONE);
        assert!(!h.consumed);
        let mut h = Harness::new(Tool::Translate);
        let p = h.press();
        let base = h.scene.draft().clone();
        h.frame(
            vec![
                egui::Event::PointerMoved(p + egui::vec2(20.0, 0.0)),
                egui::Event::Key {
                    key: egui::Key::Escape,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            true,
            true,
            egui::Modifiers::NONE,
        );
        assert!(h.consumed && !h.scene.editing() && h.handles.latch);
        assert_eq!(h.scene.draft(), &base);
        h.frame(
            vec![egui::Event::PointerMoved(p + egui::vec2(50.0, 0.0))],
            true,
            true,
            egui::Modifiers::NONE,
        );
        assert!(h.consumed);
        h.button(p, false, egui::Modifiers::NONE);
        assert!(h.consumed && !h.handles.latch);
        assert_eq!(h.scene.draft(), &base);
    }
    #[test]
    fn handles_pointer_invalid_support_release_restores_base_and_snapshot() {
        let mut h = Harness::new(Tool::Support);
        let frozen = h.scene.draft().freeze().unwrap();
        h.press();
        let valid = h.point(0.7);
        h.frame(
            vec![egui::Event::PointerMoved(valid)],
            true,
            true,
            egui::Modifiers::NONE,
        );
        assert!(h.scene.editing());
        let invalid = h.point(0.3);
        h.frame(
            vec![egui::Event::PointerMoved(invalid)],
            true,
            true,
            egui::Modifiers::NONE,
        );
        assert_eq!(h.handles.message, Some("Invalid preview · release cancels"));
        h.button(invalid, false, egui::Modifiers::NONE);
        assert!(h.consumed && !h.scene.editing());
        assert_eq!(h.scene.draft(), frozen.scene());
    }
    #[test]
    fn handles_cancellation_context_focus_pointer_and_camera_loss_never_hands_off() {
        for reason in 0..8 {
            let mut h = Harness::new(Tool::Translate);
            let p = h.press();
            let base = h.scene.draft().clone();
            let mut events = vec![];
            let mut focused = true;
            let mut editable = true;
            match reason {
                0 => focused = false,
                1 => events.push(egui::Event::PointerGone),
                2 => h.scene.select(None),
                3 => h.camera.yaw += 0.2,
                4 => editable = false,
                5 => h.scene.begin_run(false),
                6 => h.canvas_size.x -= 20.0,
                _ => h.side = 2.0,
            }
            h.frame(events, focused, editable, egui::Modifiers::NONE);
            assert!(h.consumed && !h.scene.editing());
            assert_eq!(h.scene.draft(), &base);
            h.button(p, false, egui::Modifiers::NONE);
            assert!(h.consumed);
            assert_eq!(h.scene.draft(), &base);
        }
        let mut outside = Harness::new(Tool::Translate);
        outside.press();
        let p = egui::pos2(outside.rect.right() + 50.0, outside.rect.center().y);
        outside.frame(
            vec![egui::Event::PointerMoved(p)],
            true,
            true,
            egui::Modifiers::NONE,
        );
        assert!(outside.consumed && outside.scene.editing());
        outside.button(p, false, egui::Modifiers::NONE);
        assert!(outside.consumed && !outside.scene.editing());
        assert!(outside.scene.draft().objects[0].transform.translation[0] > 1.0);
        let mut middle = Harness::new(Tool::Translate);
        let p = middle.point(0.68);
        middle.frame(
            vec![
                egui::Event::PointerMoved(p),
                egui::Event::PointerButton {
                    pos: p,
                    button: egui::PointerButton::Middle,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            true,
            true,
            egui::Modifiers::NONE,
        );
        assert!(!middle.consumed && !middle.scene.editing());
        let mut edge = Harness::new(Tool::Translate);
        let p = edge.point(0.5);
        edge.button(p, true, egui::Modifiers::NONE);
        assert!(edge.consumed && edge.handles.latch && !edge.scene.editing());
        assert_eq!(
            edge.handles.message,
            Some("Handle edge-on · change view or use properties")
        );
        edge.button(p, false, egui::Modifiers::NONE);
        assert!(edge.consumed && !edge.handles.latch);
        let mut h = Harness::new(Tool::Translate);
        let p = h.point(0.68);
        h.button(p, true, egui::Modifiers::SHIFT);
        assert!(!h.consumed && !h.scene.editing());
        h.button(p, false, egui::Modifiers::SHIFT);
    }
    #[test]
    fn handles_named_controls_fit_narrow_without_touching_authoring_values() {
        for width in [220.0, 480.0] {
            let ctx = egui::Context::default();
            let s = scene();
            let base = s.draft().clone();
            let mut handles = SceneHandles::default();
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(width + 16.0, 500.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let response = ui.scope(|ui| handles.controls(ui, &s, true));
                        assert!(
                            response.response.rect.height() < 100.0,
                            "{:?}",
                            response.response.rect
                        );
                    });
                },
            );
            assert_eq!(s.draft(), &base);
            assert!(!s.editing());
        }
    }
}
