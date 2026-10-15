//! Perspective voxel surfaces and Blender-style navigation using egui's native
//! mesh renderer. Camera changes never modify simulation or timeline state.
use super::history::PreviewFrame;
use eframe::egui::{self, Color32, Rect, Stroke};
use rheon::{RotatingCylinderEmitter, WorkerVoxel};
use std::sync::Arc;

type V3 = [f32; 3];
fn add(a: V3, b: V3) -> V3 {
    std::array::from_fn(|d| a[d] + b[d])
}
fn mul(a: V3, s: f32) -> V3 {
    a.map(|v| v * s)
}
pub use super::camera::OrbitCamera;
use super::voxel_view::Face;
struct CachedScene {
    camera: OrbitCamera,
    rect: Rect,
    emitter: Option<RotatingCylinderEmitter>,
    time: f64,
    mesh: Arc<egui::Mesh>,
}
#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum ViewMode {
    #[default]
    Appearance,
    Particles,
    Obstacle,
    Transport,
}
pub struct Viewport {
    pub collision_mesh: Option<Arc<rheon::CollisionMesh>>,
    pub camera: OrbitCamera,
    engineering: Option<super::engineering::EngineeringView>,
    mode: ViewMode,
    particles: super::particle_view::ParticleView,
    transport: Option<Result<super::transport_view::TransportView, String>>,
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
            mode: ViewMode::Appearance,
            particles: Default::default(),
            transport: None,
            engineering_camera: OrbitCamera {
                distance: 1.8,
                ..Default::default()
            },
            touchpad_mode: false,
            threshold: 7,
            domain_side: 1.0,
            collision_mesh: None,
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
        inspector: Option<&super::inspector::Inspector>,
    ) {
        ui.horizontal_wrapped(|ui| {
            for (mode, label) in [
                (ViewMode::Appearance, "Appearance"),
                (ViewMode::Particles, "Particles"),
                (ViewMode::Obstacle, "Obstacle lab"),
                (ViewMode::Transport, "Transport session"),
            ] {
                ui.selectable_value(&mut self.mode, mode, label);
            }
            if self.mode != ViewMode::Appearance {
                ui.checkbox(&mut self.engineering_camera.orthographic, "Orthographic");
                if ui.button("Frame view").clicked() {
                    self.engineering_camera = OrbitCamera {
                        distance: 1.8,
                        ..Default::default()
                    };
                }
            }
        });
        match self.mode {
            ViewMode::Obstacle => self
                .engineering
                .get_or_insert_with(Default::default)
                .controls(ui),
            ViewMode::Particles => self.particles.controls(ui, inspector),
            ViewMode::Transport => match self
                .transport
                .get_or_insert_with(super::transport_view::TransportView::new)
            {
                Ok(view) => view.controls(ui),
                Err(error) => {
                    ui.colored_label(Color32::LIGHT_RED, error);
                }
            },
            ViewMode::Appearance => {}
        }
        let (rect, response) = ui.allocate_exact_size(
            ui.available_size().max(egui::vec2(1.0, 1.0)),
            egui::Sense::click_and_drag(),
        );
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, Color32::from_rgb(14, 20, 29));
        if self.mode != ViewMode::Appearance {
            std::mem::swap(&mut self.camera, &mut self.engineering_camera);
            self.input(ui, &response, rect, 1.8);
            match self.mode {
                ViewMode::Obstacle => {
                    if let Some(lab) = &mut self.engineering {
                        lab.paint(ui, rect, &response, &self.camera);
                    }
                }
                ViewMode::Particles => self.particles.paint(
                    ui,
                    rect,
                    &response,
                    &self.camera,
                    self.domain_side,
                    inspector,
                ),
                ViewMode::Transport => {
                    if let Some(Ok(view)) = &mut self.transport {
                        view.view.paint(ui, rect, &response, &self.camera);
                    }
                }
                ViewMode::Appearance => {}
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
            let mesh = super::voxel_view::mesh(faces, &self.camera, rect);
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
        if let Some(e) = emitter
            && let (Ok(center), Ok(axis)) = (e.center_at(time), e.axis_at(time))
        {
            let center = center.map(|v| v as f32);
            for (direction, color, label) in [
                (
                    if emitter_strengths(e)[0] > 0.0 {
                        axis
                    } else {
                        [0.0; 3]
                    },
                    Color32::from_rgb(80, 220, 230),
                    "Jet +",
                ),
                (
                    e.motion_velocity,
                    Color32::from_rgb(245, 170, 80),
                    "Emitter motion",
                ),
            ] {
                let n = direction[0].hypot(direction[1]).hypot(direction[2]);
                if n > 0.0 && n.is_finite() {
                    let end = add(center, direction.map(|v| (0.18 * v / n) as f32));
                    if let (Some((a, _)), Some((b, _))) = (
                        self.camera.project(center, rect),
                        self.camera.project(end, rect),
                    ) {
                        painter.arrow(a, b - a, Stroke::new(2.0, color));
                        painter.text(
                            b,
                            egui::Align2::LEFT_BOTTOM,
                            label,
                            egui::FontId::proportional(11.0),
                            color,
                        );
                    }
                }
            }
        }
        if let Some(mesh) = &self.collision_mesh {
            const DISPLAY_TRIANGLES: usize = 12_000;
            let stride = mesh.triangles().len().div_ceil(DISPLAY_TRIANGLES).max(1);
            for triangle in mesh.triangles().iter().step_by(stride) {
                for edge in 0..3 {
                    let a = mesh.vertices()[triangle[edge]].map(|v| (v / self.domain_side) as f32);
                    let b = mesh.vertices()[triangle[(edge + 1) % 3]]
                        .map(|v| (v / self.domain_side) as f32);
                    self.line(
                        &painter,
                        rect,
                        a,
                        b,
                        Stroke::new(1.0, Color32::from_rgb(245, 180, 65)),
                    );
                }
            }
            painter.text(
                rect.left_top() + egui::vec2(14.0, 38.0),
                egui::Align2::LEFT_TOP,
                format!(
                    "Amber: collision wireframe (x-ray), {:?}; showing 1 / {} triangles",
                    mesh.policy(),
                    stride
                ),
                egui::FontId::proportional(12.0),
                Color32::from_rgb(245, 180, 65),
            );
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
        painter.text(
            rect.left_bottom() + egui::vec2(14.0, -12.0),
            egui::Align2::LEFT_BOTTOM,
            "Drag orbit · Shift pan · Ctrl zoom · Home frame",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(142, 158, 176),
        );
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
    super::voxel_view::faces(
        [frame.edge; 3],
        [1.0; 3],
        &frame.density,
        threshold,
        true,
        |_, shade, density| {
            super::voxel_view::water_color(density, shade, frame.material_color, 255)
        },
    )
}

fn emitter_strengths(emitter: RotatingCylinderEmitter) -> [f64; 2] {
    emitter
        .emitted_velocity
        .map_or(emitter.pressure_heads, |v| {
            [v[0].hypot(v[1]).hypot(v[2]); 2]
        })
}

fn cylinder_faces(emitter: RotatingCylinderEmitter, time: f64) -> Vec<Face> {
    let (Ok(basis), Ok(center)) = (emitter.basis_at(time), emitter.center_at(time)) else {
        return Vec::new();
    };
    let [axis, up, across] = basis.map(|a| a.map(|v| v as f32));
    let center = center.map(|v| v as f32);
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
        let strengths = emitter_strengths(emitter);
        for (end, head) in [(1.0, strengths[0]), (-1.0, strengths[1])] {
            let c = add(center, mul(axis, end * emitter.half_length as f32));
            for level in 0..3 {
                let inner = level as f32 / 3.0 * emitter.radius as f32;
                let outer = (level + 1) as f32 / 3.0 * emitter.radius as f32;
                let strength = ((head / strengths[0].max(strengths[1]).max(1e-12)) as f32)
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
    use eframe::egui::Pos2;
    #[test]
    fn emitter_visual_strengths_follow_selected_mode_and_zero_disables_jets() {
        let mut e = RotatingCylinderEmitter::default();
        assert_eq!(emitter_strengths(e), e.pressure_heads);
        e.emitted_velocity = Some([3.0, 4.0, 0.0]);
        assert_eq!(emitter_strengths(e), [5.0; 2]);
        e.emitted_velocity = Some([0.0; 3]);
        assert_eq!(emitter_strengths(e), [0.0; 2]);
        e.emitted_velocity = None;
        e.pressure_heads = [0.0; 2];
        assert_eq!(emitter_strengths(e), [0.0; 2]);
    }
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
            mode: ViewMode::Obstacle,
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
                    .show(ctx, |ui| viewport.show(ui, None, None, &[], 32, None));
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
                    .show(ctx, |ui| viewport.show(ui, None, None, &[], 32, None));
            },
        );
        assert_eq!(viewport.camera, live);
        assert_eq!(viewport.engineering_camera.distance, 1.8);
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
                    .show(ctx, |ui| viewport.show(ui, None, None, &[], 32, None));
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
                    .show(ctx, |ui| viewport.show(ui, None, None, &[], 32, None));
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
