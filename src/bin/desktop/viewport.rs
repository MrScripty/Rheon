//! Perspective voxel surfaces and Blender-style navigation using egui's native
//! mesh renderer. Camera changes never modify simulation or timeline state.
use super::history::PreviewFrame;
use eframe::egui::{self, Color32, Rect, Stroke};
use rheon::{RotatingCylinderEmitter, WorkerVoxel};
use std::sync::Arc;

// Appearance payload policy, independent of physical sampling/history budget.
// At most 262144 surface faces: two face buffers plus two projected meshes
// (including the previous cached mesh) retain at most ~78 MiB of array payload.
// egui/GPU/allocator/other views are outside this bound.
const MAX_SURFACE_FACES: usize = 262_144;
const MAX_RENDER_VOXELS: usize = 512 * 512 * 512;
pub fn resource_estimate(edge: usize) -> String {
    let voxels = edge as u128 * edge as u128 * edge as u128;
    let bytes_per_face = 2 * std::mem::size_of::<Face>()
        + 8 * std::mem::size_of::<egui::epaint::Vertex>()
        + 12 * std::mem::size_of::<u32>();
    format!(
        "Appearance scan: {voxels} voxel visits; worst surface payload {:.1} MiB. Renderer refuses above {MAX_RENDER_VOXELS} visits or {MAX_SURFACE_FACES} exposed faces; preview is never silently reduced.",
        voxels as f64 * 6.0 * bytes_per_face as f64 / 1048576.0
    )
}
type V3 = [f32; 3];
fn add(a: V3, b: V3) -> V3 {
    std::array::from_fn(|d| a[d] + b[d])
}
fn mul(a: V3, s: f32) -> V3 {
    a.map(|v| v * s)
}
fn dot(a: V3, b: V3) -> f32 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}
pub use super::camera::OrbitCamera;
#[derive(Clone, Debug)]
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
#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum ViewMode {
    #[default]
    Appearance,
    Particles,
    Obstacle,
    Transport,
}
// Whole labelled tabs wrap between controls, never within a label.
fn view_mode_tabs(ui: &mut egui::Ui, current: &mut ViewMode, blocked: bool) -> [egui::Response; 4] {
    [(ViewMode::Appearance,"Appearance"),(ViewMode::Particles,"Particles"),(ViewMode::Obstacle,"Obstacle lab"),(ViewMode::Transport,"Transport session")].map(|(mode,label)|{
        let mut response=ui.add_enabled(!blocked||mode==ViewMode::Appearance,egui::Button::selectable(*current==mode,label).wrap_mode(egui::TextWrapMode::Extend));
        if response.clicked() && *current!=mode {*current=mode;response.mark_changed();}
        response.on_disabled_hover_text("Authored-force preview and lab runs are not integrated. Use the scene list and properties to author the draft.")
    })
}
pub struct Viewport {
    #[cfg(test)]
    presentation_only: bool,
    scene_handles: super::scene_handles::SceneHandles,
    /// Draft authored fields have no runtime/preview integration yet.
    pub authored_forces_pending: bool,
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
    surface_error: Option<String>,
    cached_threshold: u8,
    scene: Option<CachedScene>,
}
impl Default for Viewport {
    fn default() -> Self {
        Self {
            #[cfg(test)]
            presentation_only: false,
            scene_handles: Default::default(),
            authored_forces_pending: false,
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
            surface_error: None,
            cached_threshold: 0,
            scene: None,
        }
    }
}
pub struct ViewInputs<'a> {
    pub frame: Option<Arc<PreviewFrame>>,
    pub emitter: Option<RotatingCylinderEmitter>,
    pub workers: &'a [WorkerVoxel],
    pub grid_edge: usize,
    pub inspector: Option<&'a super::inspector::Inspector>,
    pub scene: Option<&'a mut super::scene_ui::SceneUi>,
    pub editable_scene: bool,
}
impl Viewport {
    #[cfg(test)]
    pub(super) fn presentation_only_appearance(&mut self) {
        self.presentation_only = true;
        self.mode = ViewMode::Appearance;
    }
    #[cfg(test)]
    pub(super) fn presentation_is_inert(&self) -> bool {
        self.mode == ViewMode::Appearance && self.engineering.is_none() && self.transport.is_none()
    }

    #[cfg(test)]
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        frame: Option<Arc<PreviewFrame>>,
        emitter: Option<RotatingCylinderEmitter>,
        workers: &[WorkerVoxel],
        grid_edge: usize,
        inspector: Option<&super::inspector::Inspector>,
    ) {
        self.show_scene(
            ui,
            ViewInputs {
                frame,
                emitter,
                workers,
                grid_edge,
                inspector,
                scene: None,
                editable_scene: false,
            },
        );
    }
    pub fn show_scene(&mut self, ui: &mut egui::Ui, inputs: ViewInputs<'_>) {
        let ViewInputs {
            frame,
            emitter,
            workers,
            grid_edge,
            inspector,
            mut scene,
            editable_scene,
        } = inputs;
        if self.authored_forces_pending || self.scene_handles.busy() {
            self.mode = ViewMode::Appearance;
        }
        ui.horizontal_wrapped(|ui| {
            view_mode_tabs(
                ui,
                &mut self.mode,
                self.authored_forces_pending || self.scene_handles.busy(),
            );
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
        #[cfg(test)]
        if self.presentation_only {
            self.mode = ViewMode::Appearance;
        }
        if self.mode == ViewMode::Appearance
            && let Some(scene) = scene.as_deref()
        {
            self.scene_handles.controls(ui, scene, editable_scene);
        }
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
        let consumed = scene.as_deref_mut().is_some_and(|scene| {
            self.scene_handles.update(
                ui,
                &response,
                &self.camera,
                self.domain_side,
                scene,
                editable_scene,
            )
        });
        if !consumed {
            self.input(ui, &response, rect, 2.35);
        }
        if !consumed && let Some(scene) = scene.as_deref_mut() {
            let mesh = self
                .collision_mesh
                .as_ref()
                .map(|m| super::scene_view::MeshRef {
                    vertices: m.vertices(),
                    triangles: m.triangles(),
                });
            super::scene_view::interact(ui, &response, &self.camera, self.domain_side, mesh, scene);
        }
        let changed = match (&self.frame, &frame) {
            (Some(a), Some(b)) => !Arc::ptr_eq(a, b),
            (None, None) => false,
            _ => true,
        };
        if changed || self.cached_threshold != self.threshold {
            match frame
                .as_ref()
                .map_or_else(|| Ok(Vec::new()), |f| surface_faces(f, self.threshold))
            {
                Ok(faces) => {
                    self.faces = faces;
                    self.surface_error = None;
                }
                Err(error) => {
                    self.faces.clear();
                    self.surface_error = Some(error);
                }
            }
            self.frame = frame.clone();
            self.cached_threshold = self.threshold;
            self.scene = None;
        }
        if let Some(error) = &self.surface_error {
            painter.text(
                rect.left_top() + egui::vec2(12.0, 12.0),
                egui::Align2::LEFT_TOP,
                error,
                egui::FontId::proportional(13.0),
                Color32::LIGHT_RED,
            );
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
            let emitter_faces = emitter.map_or_else(Vec::new, |e| cylinder_faces(e, time));
            let count = self.faces.len() + emitter_faces.len();
            let mut faces = Vec::new();
            let mut mesh = egui::Mesh::default();
            let reserved = faces
                .try_reserve_exact(count)
                .and_then(|()| mesh.vertices.try_reserve_exact(count * 4))
                .and_then(|()| mesh.indices.try_reserve_exact(count * 6));
            if let Err(error) = reserved {
                self.scene = None;
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("Cannot allocate appearance mesh: {error}"),
                    egui::FontId::proportional(13.0),
                    Color32::LIGHT_RED,
                );
                return;
            }
            faces.extend_from_slice(&self.faces);
            faces.extend(emitter_faces);
            let basis = self.camera.basis();
            let forward = basis.1;
            faces.sort_unstable_by(|a, b| {
                let center = |f: &Face| mul(f.points.into_iter().fold([0.0; 3], add), 0.25);
                dot(center(b), forward).total_cmp(&dot(center(a), forward))
            });
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
        let caption_color = Color32::from_rgb(115, 144, 171);
        let caption = painter.layout(
            format!(
                "3D / ORBIT VIEW · domain {:.3} m · preview voxel {:.3} mm",
                self.domain_side,
                self.domain_side * 1000.0 / frame.as_ref().map_or(grid_edge, |f| f.edge) as f64
            ),
            egui::FontId::monospace(12.0),
            caption_color,
            (rect.width() - 28.0).max(1.0),
        );
        let mut scene_warning_top = rect.top() + 12.0 + caption.size().y + 8.0;
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
            let mesh_color = Color32::from_rgb(245, 180, 65);
            let mesh_caption = painter.layout(
                format!(
                    "Amber: collision wireframe (x-ray), {:?}; showing 1 / {} triangles",
                    mesh.policy(),
                    stride
                ),
                egui::FontId::proportional(12.0),
                mesh_color,
                (rect.width() - 28.0).max(1.0),
            );
            scene_warning_top += mesh_caption.size().y + 8.0;
            painter.galley(
                rect.left_top() + egui::vec2(14.0, 12.0 + caption.size().y + 8.0),
                mesh_caption,
                mesh_color,
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
        painter.galley(
            rect.left_top() + egui::vec2(14.0, 12.0),
            caption,
            caption_color,
        );
        let help_color = Color32::from_rgb(142, 158, 176);
        let help = painter.layout(
            "Drag orbit · Shift pan · Ctrl zoom · Home frame".into(),
            egui::FontId::proportional(11.0),
            help_color,
            (rect.width() - 28.0).max(1.0),
        );
        let help_origin = rect.left_bottom() + egui::vec2(14.0, -12.0 - help.size().y);
        painter.galley(help_origin, help, help_color);
        if let Some(scene) = scene {
            let mesh = self
                .collision_mesh
                .as_ref()
                .map(|m| super::scene_view::MeshRef {
                    vertices: m.vertices(),
                    triangles: m.triangles(),
                });
            super::scene_view::paint(
                ui,
                super::scene_view::PaintArea {
                    rect,
                    warning_top: scene_warning_top,
                },
                &self.camera,
                self.domain_side,
                mesh,
                scene,
            );
            self.scene_handles.paint(
                ui,
                rect,
                &self.camera,
                self.domain_side,
                scene,
                editable_scene,
            );
        }
    }
    fn input(&mut self, ui: &egui::Ui, response: &egui::Response, rect: Rect, frame_distance: f32) {
        if !(response.hovered() || response.dragged()) {
            return;
        }
        // Response focus and Context keyboard ownership acquire context locks.
        // Snapshot them before input holds its context guard.
        let keyboard_owned = response.has_focus() || !ui.ctx().wants_keyboard_input();
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
            if keyboard_owned
                && (input.key_pressed(egui::Key::Home) || input.key_pressed(egui::Key::F))
            {
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
fn surface_faces(frame: &PreviewFrame, threshold: u8) -> Result<Vec<Face>, String> {
    let count = frame
        .edge
        .checked_pow(3)
        .ok_or("Appearance voxel count overflow")?;
    if count > MAX_RENDER_VOXELS || count != frame.density.len() || frame.edge == 0 {
        return Err("Appearance unavailable: voxel scan exceeds renderer policy or frame shape is invalid; numerical fields and stored preview retain their resolution".into());
    }
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
                        if faces.len() >= MAX_SURFACE_FACES {
                            return Err(format!(
                                "Appearance unavailable: more than {MAX_SURFACE_FACES} exposed faces; lower preview density or raise the visibility threshold. Numerical resolution is unchanged"
                            ));
                        }
                        if faces.len() == faces.capacity() {
                            let additional = (MAX_SURFACE_FACES - faces.len()).min(1024);
                            faces
                                .try_reserve_exact(additional)
                                .map_err(|e| format!("Cannot allocate appearance faces: {e}"))?;
                        }
                        faces.push(Face { points, color });
                    }
                }
            }
        }
    }
    Ok(faces)
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
    fn excessive_surface_is_refused_without_downsampling() {
        let n = 64;
        let frame = PreviewFrame {
            method: rheon::PressureImplementation::default(),
            step: 0,
            time: 0.0,
            seconds: 0.0,
            edge: n,
            nonzero_tracer_cells: 0,
            density: (0..n * n * n)
                .map(|i| {
                    if (i % n + i / n % n + i / n / n) % 2 == 0 {
                        255
                    } else {
                        0
                    }
                })
                .collect(),
            report: None,
            particle_report: None,
            material_color: None,
        };
        assert!(
            surface_faces(&frame, 1)
                .unwrap_err()
                .contains("exposed faces")
        );
        assert_eq!(frame.edge, 64);
        assert!(resource_estimate(10000).contains("1000000000000 voxel visits"));
    }
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
        assert_eq!(surface_faces(&frame, 1).unwrap().len(), 24);
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
    /// Execute by exact name under a hard external timeout: the pre-fix nested
    /// context lock blocks even on plain hover, before any key is pressed.
    #[test]
    fn enabled_scene_input_hover_and_keyboard_focus_complete_without_nested_context_lock() {
        let ctx = egui::Context::default();
        let mut viewport = Viewport::default();
        let mut scene =
            super::super::scene_ui::SceneUi::geometry_fixture(rheon::ForceScene::default(), false);
        let mut editor = String::from("Keyboard owner");
        let mut draw = |viewport: &mut Viewport, key: Option<egui::Key>, focus_editor: bool| {
            let mut events = vec![egui::Event::PointerMoved(egui::pos2(400.0, 300.0))];
            if let Some(key) = key {
                events.push(egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Default::default(),
                });
            }
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0))),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::SidePanel::left("focus-owner")
                        .exact_width(150.0)
                        .show(ctx, |ui| {
                            let response = ui.text_edit_singleline(&mut editor);
                            if focus_editor {
                                response.request_focus();
                            }
                        });
                    egui::CentralPanel::default().show(ctx, |ui| {
                        assert!(ui.is_enabled());
                        viewport.show_scene(
                            ui,
                            ViewInputs {
                                frame: None,
                                emitter: None,
                                workers: &[],
                                grid_edge: 2,
                                inspector: None,
                                scene: Some(&mut scene),
                                editable_scene: true,
                            },
                        );
                    });
                },
            );
        };
        draw(&mut viewport, None, false);
        draw(&mut viewport, None, false); // enabled, hovered, no shortcut needed
        viewport.camera.distance = 4.0;
        draw(&mut viewport, Some(egui::Key::Home), false);
        assert_eq!(viewport.camera.distance, 2.35);
        draw(&mut viewport, None, true);
        viewport.camera.distance = 4.0;
        draw(&mut viewport, Some(egui::Key::F), true);
        assert_eq!(viewport.camera.distance, 4.0, "text editor owns keyboard");
        assert!(viewport.engineering.is_none() && viewport.transport.is_none());
        assert!(!viewport.presentation_only);
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

#[cfg(test)]
mod header_ui_tests {
    use super::*;
    #[test]
    fn responsive_mode_tabs_keep_whole_labels_at_900_650_150_percent() {
        for scale in [1.0, 1.5] {
            for blocked in [false, true] {
                let ctx = egui::Context::default();
                ctx.set_pixels_per_point(scale);
                let mut mode = ViewMode::Appearance;
                for _ in 0..2 {
                    let _ = ctx.run(
                        egui::RawInput {
                            screen_rect: Some(Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::vec2(900.0 / scale, 650.0 / scale),
                            )),
                            ..Default::default()
                        },
                        |ctx| {
                            egui::SidePanel::left("settings")
                                .exact_width(280.0)
                                .show(ctx, |ui| {
                                    ui.label("Setup");
                                });
                            egui::SidePanel::right("scene-hidden")
                                .exact_width(64.0)
                                .show(ctx, |ui| {
                                    let _ = ui.button("Scene");
                                });
                            egui::CentralPanel::default().show(ctx, |ui| {
                                let available = ui.available_rect_before_wrap();
                                let row = ui.horizontal_wrapped(|ui| {
                                    view_mode_tabs(ui, &mut mode, blocked)
                                });
                                assert!(
                                    row.response.rect.height() <= 60.0,
                                    "tab rows too tall: {:?}",
                                    row.response.rect
                                );
                                for (i, response) in row.inner.iter().enumerate() {
                                    assert!(
                                        response.rect.height()
                                            <= ui.spacing().interact_size.y + 1.0,
                                        "label wrapped vertically"
                                    );
                                    assert!(
                                        response.rect.left() >= available.left() - 1.0
                                            && response.rect.right() <= available.right() + 1.0,
                                        "whole tab outside header"
                                    );
                                    assert_eq!(response.enabled(), !blocked || i == 0);
                                }
                                assert!(ui.available_height() > available.height() * 0.7);
                            });
                        },
                    );
                }
                assert!(mode == ViewMode::Appearance);
            }
        }
    }
}
