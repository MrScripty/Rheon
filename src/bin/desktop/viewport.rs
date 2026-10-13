//! Perspective voxel surfaces and Blender-style navigation using egui's native
//! mesh renderer. Camera changes never modify simulation or timeline state.
use super::history::PreviewFrame;
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use rheon::{RotatingCylinderEmitter, WorkerVoxel};
use std::sync::Arc;

type V3 = [f32; 3];
fn add(a: V3, b: V3) -> V3 {
    std::array::from_fn(|d| a[d] + b[d])
}
fn sub(a: V3, b: V3) -> V3 {
    std::array::from_fn(|d| a[d] - b[d])
}
fn mul(a: V3, s: f32) -> V3 {
    a.map(|v| v * s)
}
fn dot(a: V3, b: V3) -> f32 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}
fn cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn normalize(a: V3) -> V3 {
    mul(a, 1.0 / dot(a, a).sqrt().max(1e-8))
}
#[derive(Clone, Debug, PartialEq)]
pub struct OrbitCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub target: V3,
    pub orthographic: bool,
}
impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            yaw: 0.72,
            pitch: 0.42,
            distance: 2.35,
            target: [0.5; 3],
            orthographic: false,
        }
    }
}
impl OrbitCamera {
    fn basis(&self) -> (V3, V3, V3, V3) {
        let eye = add(
            self.target,
            mul(
                [
                    self.yaw.sin() * self.pitch.cos(),
                    self.pitch.sin(),
                    self.yaw.cos() * self.pitch.cos(),
                ],
                self.distance,
            ),
        );
        let forward = normalize(sub(self.target, eye));
        let right = normalize(cross(forward, [0.0, 1.0, 0.0]));
        let up = cross(right, forward);
        (eye, forward, right, up)
    }
    pub fn orbit(&mut self, delta: Vec2) {
        self.yaw -= delta.x * 0.006;
        self.pitch = (self.pitch + delta.y * 0.006).clamp(-1.52, 1.52);
    }
    pub fn pan(&mut self, delta: Vec2, height: f32) {
        let (_, _, right, up) = self.basis();
        let scale = self.distance / height.max(1.0);
        self.target = add(
            self.target,
            add(mul(right, -delta.x * scale), mul(up, delta.y * scale)),
        )
        .map(|v| v.clamp(-3.0, 4.0));
    }
    pub fn zoom(&mut self, log_delta: f32) {
        self.distance = (self.distance * (-log_delta.clamp(-2.0, 2.0)).exp()).clamp(0.35, 12.0);
    }
    pub(super) fn project(&self, p: V3, rect: Rect) -> Option<(Pos2, f32)> {
        self.project_prepared(p, rect, self.basis())
    }
    fn project_prepared(&self, p: V3, rect: Rect, basis: (V3, V3, V3, V3)) -> Option<(Pos2, f32)> {
        let (eye, forward, right, up) = basis;
        let relative = sub(p, eye);
        let depth = dot(relative, forward);
        if depth <= 0.015 {
            return None;
        }
        let scale = rect.height() * 0.88
            / if self.orthographic {
                self.distance
            } else {
                depth
            };
        Some((
            rect.center() + egui::vec2(dot(relative, right) * scale, -dot(relative, up) * scale),
            depth,
        ))
    }
}
#[derive(Clone)]
struct Face {
    points: [V3; 4],
    color: Color32,
}
struct CachedScene {
    camera: OrbitCamera,
    rect: Rect,
    emitter: Option<RotatingCylinderEmitter>,
    time: f64,
    mesh: Arc<egui::Mesh>,
}
pub struct Viewport {
    pub camera: OrbitCamera,
    engineering: Option<super::engineering::EngineeringView>,
    engineering_active: bool,
    engineering_camera: OrbitCamera,
    pub touchpad_mode: bool,
    pub threshold: u8,
    pub domain_side: f64,
    frame: Option<Arc<PreviewFrame>>,
    faces: Vec<Face>,
    cached_threshold: u8,
    scene: Option<CachedScene>,
}
impl Default for Viewport {
    fn default() -> Self {
        Self {
            camera: OrbitCamera::default(),
            engineering: None,
            engineering_active: false,
            engineering_camera: OrbitCamera {
                distance: 1.4,
                ..Default::default()
            },
            touchpad_mode: false,
            threshold: 7,
            domain_side: 1.0,
            frame: None,
            faces: Vec::new(),
            cached_threshold: 0,
            scene: None,
        }
    }
}
impl Viewport {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        frame: Option<Arc<PreviewFrame>>,
        emitter: Option<RotatingCylinderEmitter>,
        workers: &[WorkerVoxel],
        grid_edge: usize,
    ) {
        ui.horizontal_wrapped(|ui| {
            ui.selectable_value(&mut self.engineering_active, false, "Live preview");
            ui.selectable_value(
                &mut self.engineering_active,
                true,
                "Obstacle projection lab",
            );
            if self.engineering_active {
                ui.checkbox(
                    &mut self.engineering_camera.orthographic,
                    "Orthographic lab",
                );
                if ui.button("Frame lab").clicked() {
                    self.engineering_camera = OrbitCamera {
                        distance: 1.4,
                        ..Default::default()
                    };
                }
            }
        });
        if self.engineering_active {
            self.engineering
                .get_or_insert_with(Default::default)
                .controls(ui);
        }
        let (rect, response) = ui.allocate_exact_size(
            ui.available_size().max(egui::vec2(150.0, 150.0)),
            egui::Sense::click_and_drag(),
        );
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, Color32::from_rgb(14, 20, 29));
        if self.engineering_active {
            std::mem::swap(&mut self.camera, &mut self.engineering_camera);
            self.input(ui, &response, rect, 1.4);
            if let Some(lab) = &mut self.engineering {
                lab.paint(ui, rect, &response, &self.camera);
            }
            std::mem::swap(&mut self.camera, &mut self.engineering_camera);
            return;
        }
        self.input(ui, &response, rect, 2.35);
        let changed = match (&self.frame, &frame) {
            (Some(a), Some(b)) => !Arc::ptr_eq(a, b),
            (None, None) => false,
            _ => true,
        };
        if changed || self.cached_threshold != self.threshold {
            self.faces = frame
                .as_ref()
                .map_or_else(Vec::new, |f| surface_faces(f, self.threshold));
            self.frame = frame.clone();
            self.cached_threshold = self.threshold;
            self.scene = None;
        }
        for i in 0..=10 {
            let p = i as f32 / 10.0;
            self.line(
                &painter,
                rect,
                [p, 0.0, 0.0],
                [p, 0.0, 1.0],
                Stroke::new(1.0, Color32::from_rgb(39, 50, 64)),
            );
            self.line(
                &painter,
                rect,
                [0.0, 0.0, p],
                [1.0, 0.0, p],
                Stroke::new(1.0, Color32::from_rgb(39, 50, 64)),
            );
        }
        let time = frame.as_ref().map_or(0.0, |f| f.time);
        if self.scene.as_ref().is_none_or(|c| {
            c.camera != self.camera || c.rect != rect || c.emitter != emitter || c.time != time
        }) {
            let mut faces = self.faces.clone();
            if let Some(emitter) = emitter {
                faces.extend(cylinder_faces(emitter, time));
            }
            let basis = self.camera.basis();
            let forward = basis.1;
            faces.sort_unstable_by(|a, b| {
                let center = |f: &Face| mul(f.points.into_iter().fold([0.0; 3], add), 0.25);
                dot(center(b), forward).total_cmp(&dot(center(a), forward))
            });
            let mut mesh = egui::Mesh::default();
            for face in faces {
                let projected = face
                    .points
                    .map(|p| self.camera.project_prepared(p, rect, basis));
                if let [Some((a, _)), Some((b, _)), Some((c, _)), Some((d, _))] = projected {
                    let first = mesh.vertices.len() as u32;
                    for pos in [a, b, c, d] {
                        mesh.colored_vertex(pos, face.color);
                    }
                    mesh.indices.extend_from_slice(&[
                        first,
                        first + 1,
                        first + 2,
                        first,
                        first + 2,
                        first + 3,
                    ]);
                }
            }
            self.scene = Some(CachedScene {
                camera: self.camera.clone(),
                rect,
                emitter,
                time,
                mesh: Arc::new(mesh),
            });
        }
        if let Some(scene) = &self.scene {
            painter.add(egui::Shape::mesh(Arc::clone(&scene.mesh)));
        }
        self.wire_box(
            &painter,
            rect,
            [0.0; 3],
            [1.0; 3],
            Stroke::new(1.2, Color32::from_rgb(83, 106, 130)),
        );
        for (end, color, label) in [
            ([1.15, 0.0, 0.0], Color32::from_rgb(239, 93, 99), "X"),
            ([0.0, 1.15, 0.0], Color32::from_rgb(105, 208, 131), "Y"),
            ([0.0, 0.0, 1.15], Color32::from_rgb(97, 159, 249), "Z"),
        ] {
            self.line(&painter, rect, [0.0; 3], end, Stroke::new(2.0, color));
            if let Some((p, _)) = self.camera.project(end, rect) {
                painter.text(
                    p,
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional(13.0),
                    color,
                );
            }
        }
        if let Some(worker) = workers.iter().find(|w| w.active) {
            painter.text(
                rect.right_top() + egui::vec2(-14.0, 12.0),
                egui::Align2::RIGHT_TOP,
                format!(
                    "{} workers calculating · {:?}",
                    workers.iter().filter(|w| w.active).count(),
                    worker.stage
                ),
                egui::FontId::monospace(12.0),
                Color32::from_rgb(255, 157, 65),
            );
        }
        for worker in workers {
            let h = 1.0 / grid_edge.max(1) as f32;
            let lower = worker.coordinate.map(|v| v as f32 * h);
            let upper = lower.map(|v| v + h);
            let color = if worker.active {
                Color32::from_rgb(255, 146, 35)
            } else {
                Color32::from_rgba_unmultiplied(208, 116, 34, 80)
            };
            self.wire_box(
                &painter,
                rect,
                lower,
                upper,
                Stroke::new(if worker.active { 2.3 } else { 1.0 }, color),
            );
            if let Some((pos, _)) = self.camera.project(upper, rect) {
                painter.text(
                    pos + egui::vec2(3.0, -3.0),
                    egui::Align2::LEFT_BOTTOM,
                    format!("{}", worker.worker + 1),
                    egui::FontId::monospace(11.0),
                    color,
                );
            }
        }
        painter.text(
            rect.left_top() + egui::vec2(14.0, 12.0),
            egui::Align2::LEFT_TOP,
            format!(
                "3D / ORBIT VIEW · domain {:.3} m · preview voxel {:.3} mm",
                self.domain_side,
                self.domain_side * 1000.0 / frame.as_ref().map_or(grid_edge, |f| f.edge) as f64
            ),
            egui::FontId::monospace(12.0),
            Color32::from_rgb(115, 144, 171),
        );
        painter.text(rect.left_bottom()+egui::vec2(14.0,-12.0),egui::Align2::LEFT_BOTTOM,
            "Drag orbit · Shift+drag pan · Ctrl+drag zoom · wheel/pinch zoom · 1/3/7 views · Home frame",egui::FontId::proportional(11.0),Color32::from_rgb(142,158,176));
    }
    fn input(&mut self, ui: &egui::Ui, response: &egui::Response, rect: Rect, frame_distance: f32) {
        if !(response.hovered() || response.dragged()) {
            return;
        }
        ui.input(|input| {
            let middle = input.pointer.button_down(egui::PointerButton::Middle);
            let primary = input.pointer.button_down(egui::PointerButton::Primary);
            if (middle || primary) && response.dragged() {
                let delta = input.pointer.delta();
                if input.modifiers.ctrl {
                    self.camera.zoom(-delta.y * 0.01);
                } else if input.modifiers.shift {
                    self.camera.pan(delta, rect.height());
                } else {
                    self.camera.orbit(delta);
                }
            }
            for event in &input.events {
                match event {
                    egui::Event::MouseWheel {
                        unit,
                        delta,
                        modifiers,
                    } => {
                        if modifiers.ctrl {
                            self.camera.zoom(delta.y * 0.012);
                        } else if *unit == egui::MouseWheelUnit::Point || self.touchpad_mode {
                            let delta = if *unit == egui::MouseWheelUnit::Line {
                                *delta * 18.0
                            } else {
                                *delta
                            };
                            if modifiers.shift {
                                self.camera.pan(delta, rect.height());
                            } else {
                                self.camera.orbit(delta);
                            }
                        } else {
                            self.camera.zoom(delta.y * 0.12);
                        }
                    }
                    egui::Event::Zoom(factor) if *factor > 0.0 => self.camera.zoom(factor.ln()),
                    egui::Event::Rotate(angle) => self.camera.yaw += *angle,
                    _ => {}
                }
            }
            if input.key_pressed(egui::Key::Home) || input.key_pressed(egui::Key::F) {
                self.camera = OrbitCamera {
                    distance: frame_distance,
                    ..Default::default()
                };
            }
            if input.key_pressed(egui::Key::Num1) {
                self.camera.yaw = 0.0;
                self.camera.pitch = 0.0;
            }
            if input.key_pressed(egui::Key::Num3) {
                self.camera.yaw = std::f32::consts::FRAC_PI_2;
                self.camera.pitch = 0.0;
            }
            if input.key_pressed(egui::Key::Num7) {
                self.camera.pitch = 1.52;
            }
            if input.key_pressed(egui::Key::Num5) {
                self.camera.orthographic = !self.camera.orthographic;
            }
        });
    }
    fn line(&self, painter: &egui::Painter, rect: Rect, a: V3, b: V3, stroke: Stroke) {
        if let (Some((a, _)), Some((b, _))) =
            (self.camera.project(a, rect), self.camera.project(b, rect))
        {
            painter.line_segment([a, b], stroke);
        }
    }
    fn wire_box(&self, painter: &egui::Painter, rect: Rect, lo: V3, hi: V3, stroke: Stroke) {
        let vertices: [V3; 8] = std::array::from_fn(|i| {
            std::array::from_fn(|d| if i & (1 << d) == 0 { lo[d] } else { hi[d] })
        });
        for i in 0..8 {
            for d in 0..3 {
                let j = i ^ (1 << d);
                if i < j {
                    self.line(painter, rect, vertices[i], vertices[j], stroke);
                }
            }
        }
    }
}
fn surface_faces(frame: &PreviewFrame, threshold: u8) -> Vec<Face> {
    let n = frame.edge;
    let h = 1.0 / n as f32;
    let mut faces = Vec::new();
    let threshold = threshold.max(1);
    for z in 0..n {
        for y in 0..n {
            for x in 0..n {
                let p = [x, y, z];
                let row = x + n * (y + n * z);
                let density = frame.density[row];
                if density < threshold {
                    continue;
                }
                let lo = p.map(|v| v as f32 * h);
                let hi = lo.map(|v| v + h);
                for d in 0..3 {
                    for positive in [false, true] {
                        let mut q = p;
                        let boundary = if positive { p[d] + 1 == n } else { p[d] == 0 };
                        if !boundary {
                            if positive {
                                q[d] += 1;
                            } else {
                                q[d] -= 1;
                            }
                            if frame.density[q[0] + n * (q[1] + n * q[2])] >= threshold {
                                continue;
                            }
                        }
                        let a = (d + 1) % 3;
                        let b = (d + 2) % 3;
                        let points = std::array::from_fn(|corner| {
                            let mut v = lo;
                            v[d] = if positive { hi[d] } else { lo[d] };
                            if corner == 1 || corner == 2 {
                                v[a] = hi[a];
                            }
                            if corner >= 2 {
                                v[b] = hi[b];
                            }
                            v
                        });
                        let shade = match (d, positive) {
                            (1, true) => 1.0,
                            (2, true) => 0.83,
                            (0, true) => 0.73,
                            _ => 0.55,
                        };
                        let t = density as f32 / 255.0;
                        let color = if let Some(base) = frame.material_color {
                            let brightness = (0.55 + 0.45 * t) * shade;
                            Color32::from_rgb(
                                (base[0] as f32 * brightness) as u8,
                                (base[1] as f32 * brightness) as u8,
                                (base[2] as f32 * brightness) as u8,
                            )
                        } else {
                            Color32::from_rgb(
                                ((24.0 + 85.0 * t) * shade) as u8,
                                ((136.0 + 90.0 * t) * shade) as u8,
                                ((210.0 + 44.0 * t) * shade) as u8,
                            )
                        };
                        faces.push(Face { points, color });
                    }
                }
            }
        }
    }
    faces
}
fn cylinder_faces(emitter: RotatingCylinderEmitter, time: f64) -> Vec<Face> {
    let Ok(axis) = emitter.axis_at(time) else {
        return Vec::new();
    };
    let axis = axis.map(|v| v as f32);
    let center = emitter.center.map(|v| v as f32);
    let across = [-axis[2], 0.0, axis[0]];
    let up = [0.0, 1.0, 0.0];
    let ring = |end: f32, index: usize, radius: f32| {
        let angle = index as f32 / 24.0 * std::f32::consts::TAU;
        add(
            add(center, mul(axis, end * emitter.half_length as f32)),
            add(
                mul(up, angle.cos() * radius),
                mul(across, angle.sin() * radius),
            ),
        )
    };
    let mut result = Vec::new();
    for i in 0..24 {
        result.push(Face {
            points: [
                ring(-1.0, i, emitter.radius as f32),
                ring(-1.0, i + 1, emitter.radius as f32),
                ring(1.0, i + 1, emitter.radius as f32),
                ring(1.0, i, emitter.radius as f32),
            ],
            color: Color32::from_rgb(105, 119, 144),
        });
        for (end, head) in [
            (1.0, emitter.pressure_heads[0]),
            (-1.0, emitter.pressure_heads[1]),
        ] {
            let c = add(center, mul(axis, end * emitter.half_length as f32));
            for level in 0..3 {
                let inner = level as f32 / 3.0 * emitter.radius as f32;
                let outer = (level + 1) as f32 / 3.0 * emitter.radius as f32;
                let strength = ((head
                    / emitter.pressure_heads[0]
                        .max(emitter.pressure_heads[1])
                        .max(1e-12)) as f32)
                    * (1.0 - level as f32 * 0.22);
                let color = Color32::from_rgb(
                    (105.0 + 120.0 * strength) as u8,
                    (65.0 + 55.0 * strength) as u8,
                    (147.0 + 42.0 * strength) as u8,
                );
                result.push(Face {
                    points: [
                        if level == 0 { c } else { ring(end, i, inner) },
                        ring(end, i, outer),
                        ring(end, i + 1, outer),
                        if level == 0 {
                            c
                        } else {
                            ring(end, i + 1, inner)
                        },
                    ],
                    color,
                });
            }
        }
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn perspective_projection_and_camera_motion_are_finite_and_independent_of_history() {
        let mut c = OrbitCamera::default();
        let rect = Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0));
        assert!((c.project([0.5; 3], rect).unwrap().0 - rect.center()).length() < 0.01);
        let before = c.project([0.9, 0.4, 0.2], rect).unwrap().0;
        c.orbit(egui::vec2(50.0, 30.0));
        assert_ne!(c.project([0.9, 0.4, 0.2], rect).unwrap().0, before);
        let distance = c.distance;
        c.pan(egui::vec2(20.0, 0.0), 600.0);
        assert_eq!(c.distance, distance);
        for _ in 0..100 {
            c.zoom(1.0);
        }
        assert_eq!(c.distance, 0.35);
    }
    #[test]
    fn lab_navigation_preserves_the_live_camera_and_uses_its_own_framing() {
        let ctx = egui::Context::default();
        let mut viewport = Viewport {
            engineering_active: true,
            ..Default::default()
        };
        viewport.camera.yaw = 1.2;
        viewport.engineering_camera.distance = 4.0;
        let live = viewport.camera.clone();
        // egui establishes hit regions on the first frame before accepting hover.
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0))),
                events: vec![egui::Event::PointerMoved(egui::pos2(400.0, 400.0))],
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default()
                    .show(ctx, |ui| viewport.show(ui, None, None, &[], 32));
            },
        );
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0))),
                events: vec![
                    egui::Event::PointerMoved(egui::pos2(400.0, 400.0)),
                    egui::Event::Key {
                        key: egui::Key::Home,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: Default::default(),
                    },
                ],
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default()
                    .show(ctx, |ui| viewport.show(ui, None, None, &[], 32));
            },
        );
        assert_eq!(viewport.camera, live);
        assert_eq!(viewport.engineering_camera.distance, 1.4);
    }
    #[test]
    fn voxel_surface_culls_internal_faces() {
        let frame = PreviewFrame {
            method: rheon::PressureImplementation::default(),
            step: 1,
            time: 0.02,
            seconds: 0.1,
            edge: 2,
            nonzero_tracer_cells: 8,
            density: vec![255; 8],
            report: None,
            particle_report: None,
            material_color: None,
        };
        assert_eq!(surface_faces(&frame, 1).len(), 24);
    }
    #[test]
    fn primary_drag_orbits_pans_and_zooms_without_alt() {
        let ctx = egui::Context::default();
        let mut viewport = Viewport::default();
        let draw = |viewport: &mut Viewport, events, modifiers| {
            let input = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0))),
                events,
                modifiers,
                ..Default::default()
            };
            let _ = ctx.run(input, |ctx| {
                egui::CentralPanel::default()
                    .show(ctx, |ui| viewport.show(ui, None, None, &[], 32));
            });
        };
        for (modifiers, mode) in [
            (egui::Modifiers::default(), 0),
            (
                egui::Modifiers {
                    shift: true,
                    ..Default::default()
                },
                1,
            ),
            (
                egui::Modifiers {
                    ctrl: true,
                    ..Default::default()
                },
                2,
            ),
        ] {
            let start = egui::pos2(300.0, 300.0);
            draw(
                &mut viewport,
                vec![egui::Event::PointerMoved(start)],
                modifiers,
            );
            draw(
                &mut viewport,
                vec![egui::Event::PointerButton {
                    pos: start,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers,
                }],
                modifiers,
            );
            let before = viewport.camera.clone();
            let end = start + egui::vec2(35.0, 25.0);
            draw(
                &mut viewport,
                vec![egui::Event::PointerMoved(end)],
                modifiers,
            );
            match mode {
                0 => assert_ne!(before.yaw, viewport.camera.yaw),
                1 => {
                    assert_ne!(before.target, viewport.camera.target);
                    assert_eq!(before.yaw, viewport.camera.yaw);
                }
                _ => {
                    assert_ne!(before.distance, viewport.camera.distance);
                    assert_eq!(before.target, viewport.camera.target);
                }
            }
            draw(
                &mut viewport,
                vec![egui::Event::PointerButton {
                    pos: end,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers,
                }],
                modifiers,
            );
        }
    }
    #[test]
    fn native_touchpad_scroll_pans_orbits_and_pinches_through_the_viewport_input_path() {
        let ctx = egui::Context::default();
        let mut viewport = Viewport::default();
        let draw = |viewport: &mut Viewport, events| {
            let input = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0))),
                events,
                ..Default::default()
            };
            let _ = ctx.run(input, |ctx| {
                egui::CentralPanel::default()
                    .show(ctx, |ui| viewport.show(ui, None, None, &[], 32));
            });
        };
        draw(
            &mut viewport,
            vec![egui::Event::PointerMoved(egui::pos2(300.0, 300.0))],
        );
        let yaw = viewport.camera.yaw;
        draw(
            &mut viewport,
            vec![egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(25.0, 5.0),
                modifiers: Default::default(),
            }],
        );
        assert_ne!(yaw, viewport.camera.yaw);
        let target = viewport.camera.target;
        draw(
            &mut viewport,
            vec![egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(15.0, 0.0),
                modifiers: egui::Modifiers {
                    shift: true,
                    ..Default::default()
                },
            }],
        );
        assert_ne!(target, viewport.camera.target);
        let distance = viewport.camera.distance;
        draw(&mut viewport, vec![egui::Event::Zoom(1.2)]);
        assert!(viewport.camera.distance < distance);
    }
}
