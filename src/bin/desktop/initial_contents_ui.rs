//! One slice brush, exact CPU candidate and canonical Scene selection.
use super::{
    authoring_history::{self, Action},
    camera::OrbitCamera,
};
use eframe::egui::{self, Color32, Pos2, Rect, Stroke};
use rheon::{GridGeometry, SceneObjectId, initial_contents::*};
struct Gesture {
    camera: OrbitCamera,
    rect: Rect,
    side: f64,
    previous: [f64; 3],
}
pub struct InitialContentsUi {
    pub(super) editor: Option<ContentsEditor>,
    pub(super) create_requested: bool,
    pub(super) run_requested: bool,
    pub(super) draft_action: Option<super::voxel_material_authoring::DraftFileAction>,
    pub(super) file_path: String,
    pub(super) io_feedback: String,
    pub(super) uncertain_save: Option<std::path::PathBuf>,
    brush: ContentsBrush,
    keyboard: [f64; 2],
    show_air: bool,
    gesture: Option<Gesture>,
    latch: bool,
    drain_frame: Option<u64>,
    error: Option<String>,
}
impl Default for InitialContentsUi {
    fn default() -> Self {
        Self {
            editor: None,
            create_requested: false,
            run_requested: false,
            draft_action: None,
            file_path: "initial-contents.json".into(),
            io_feedback: String::new(),
            uncertain_save: None,
            brush: ContentsBrush {
                operation: BrushOperation::Replace,
                material: ContentsMaterial::Water,
                radius: 0.1,
                axis: 2,
                layer: 0,
                depth: BrushDepth::Layer,
            },
            keyboard: [0.5; 2],
            show_air: true,
            gesture: None,
            latch: false,
            drain_frame: None,
            error: None,
        }
    }
}
impl InitialContentsUi {
    pub fn id(&self) -> Option<SceneObjectId> {
        self.editor.as_ref().map(|e| e.document().id())
    }
    pub fn create(&mut self, id: SceneObjectId, grid: GridGeometry) -> Result<(), String> {
        if self.editor.is_some() {
            return Err("Initial contents already exist; save before creating a new domain".into());
        }
        let editor = ContentsEditor::new(
            InitialContents::new(id, grid.clone(), ContentsMaterial::Vacuum)
                .map_err(|e| e.to_string())?,
        );
        self.brush.layer = grid.counts()[2] / 2;
        self.brush.radius = grid.spacing().into_iter().fold(f64::INFINITY, f64::min) * 1.5;
        self.keyboard = std::array::from_fn(|d| (grid.origin()[d] + grid.upper()[d]) * 0.5);
        self.editor = Some(editor);
        self.error = None;
        Ok(())
    }
    pub(super) fn install_loaded(&mut self, editor: ContentsEditor) {
        let grid = editor.document().geometry();
        self.brush = ContentsBrush {
            operation: BrushOperation::Replace,
            material: ContentsMaterial::Water,
            radius: grid.spacing()[0] * 1.5,
            axis: 2,
            layer: grid.counts()[2] / 2,
            depth: BrushDepth::Layer,
        };
        self.keyboard = std::array::from_fn(|d| (grid.origin()[d] + grid.upper()[d]) * 0.5);
        self.editor = Some(editor);
        self.gesture = None;
        self.latch = false;
        self.drain_frame = None;
        self.error = None;
        self.create_requested = false;
        self.draft_action = None;
        self.show_air = true;
    }
    pub(super) fn drain_loaded_frame(&mut self, ctx: &egui::Context) {
        self.drain_frame = Some(ctx.cumulative_frame_nr());
        self.latch = ctx.input(|i| i.pointer.button_down(egui::PointerButton::Primary));
    }
    pub(super) fn file_controls(&mut self, ui: &mut egui::Ui, editable: bool) {
        use super::voxel_material_authoring::DraftFileAction;
        ui.collapsing("Draft file", |ui| {
            let ready =
                editable && !self.busy() && !self.editor.as_ref().is_some_and(|e| e.active());
            ui.add_enabled_ui(ready, |ui| {
                let label = ui.label("Local draft path");
                ui.add(
                    egui::TextEdit::singleline(&mut self.file_path)
                        .char_limit(4096)
                        .desired_width(f32::INFINITY),
                )
                .labelled_by(label.id);
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .add_enabled(
                            self.editor.is_some() && self.uncertain_save.is_none(),
                            egui::Button::new("Save draft"),
                        )
                        .clicked()
                        && self.draft_action.is_none()
                    {
                        self.draft_action =
                            Some(DraftFileAction::Save(self.file_path.clone().into()));
                    }
                    if ui.button("Load draft").clicked() && self.draft_action.is_none() {
                        self.draft_action =
                            Some(DraftFileAction::Load(self.file_path.clone().into()));
                    }
                });
            });
            ui.small(
                "Save uses a new file. Load replaces contents/domain and clears contents undo.",
            );
            if !self.io_feedback.is_empty() {
                ui.label(&self.io_feedback);
            }
            if let Some(path) = &self.uncertain_save {
                ui.label(format!(
                    "Save outcome uncertain at {}; further saves paused pending reconciliation",
                    path.display()
                ));
            }
        });
    }
    pub fn busy(&self) -> bool {
        self.gesture.is_some() || self.latch
    }
    pub fn cancel(&mut self) {
        if let Some(e) = &mut self.editor {
            e.cancel();
        }
        self.latch |= self.gesture.take().is_some();
    }
    pub fn hidden(&mut self, ctx: &egui::Context) {
        self.cancel();
        let (released, down) = ctx.input(|i| {
            (
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
                i.pointer.button_down(egui::PointerButton::Primary),
            )
        });
        if down {
            self.latch = true;
        }
        if released && !down {
            self.latch = false;
        }
    }
    fn outcome<T>(&mut self, result: Result<T, ContentsError>) {
        self.error = result.err().map(|e| e.to_string());
    }
    fn command(&mut self, action: Action) {
        if self.latch {
            return;
        }
        if self.gesture.is_some() || action == Action::Cancel {
            self.cancel();
            return;
        }
        if let Some(editor) = &mut self.editor {
            let result = match action {
                Action::Undo => editor.undo(),
                Action::Redo => editor.redo(),
                Action::Cancel => unreachable!(),
            };
            self.outcome(result);
        }
    }
    fn plane_axes(&self) -> [usize; 2] {
        match self.brush.axis {
            0 => [1, 2],
            1 => [0, 2],
            _ => [0, 1],
        }
    }
    fn keyboard_point(&self) -> Result<[f64; 3], ContentsError> {
        let editor = self.editor.as_ref().ok_or(ContentsError::InvalidIdentity)?;
        let mut p = [0.0; 3];
        p[self.brush.axis] = self.brush.slice_position(editor.document().geometry())?;
        let axes = self.plane_axes();
        p[axes[0]] = self.keyboard[0];
        p[axes[1]] = self.keyboard[1];
        Ok(p)
    }
    fn apply_dab(&mut self) {
        if self.busy() {
            return;
        }
        let result = (|| {
            let p = self.keyboard_point()?;
            let editor = self.editor.as_mut().ok_or(ContentsError::InvalidIdentity)?;
            editor.begin(self.brush)?;
            editor.segment(p, p)?;
            editor.commit()
        })();
        self.outcome(result);
    }
    pub fn controls(
        &mut self,
        ui: &mut egui::Ui,
        editable: bool,
        run_readiness: Result<(), String>,
    ) {
        ui.label("Two incompressible materials · filled closed box · no surface tension");
        let before = self.brush;
        ui.add_enabled_ui(editable, |ui| {
            ui.horizontal_wrapped(|ui| {
                for (value, label) in [
                    (BrushOperation::Add, "Add"),
                    (BrushOperation::Remove, "Remove"),
                    (BrushOperation::Replace, "Replace"),
                ] {
                    ui.selectable_value(&mut self.brush.operation, value, label);
                }
            });
            egui::ComboBox::from_label("Material")
                .selected_text(self.brush.material.label())
                .show_ui(ui, |ui| {
                    for m in [ContentsMaterial::Water, ContentsMaterial::Air] {
                        ui.selectable_value(&mut self.brush.material, m, m.label());
                    }
                });
            ui.horizontal(|ui| {
                let label = ui.label("Radius · m");
                ui.add(egui::DragValue::new(&mut self.brush.radius).speed(0.001))
                    .labelled_by(label.id);
            });
            ui.horizontal_wrapped(|ui| {
                ui.label("Slice");
                for (axis, label) in ["X", "Y", "Z"].into_iter().enumerate() {
                    ui.selectable_value(&mut self.brush.axis, axis, label);
                }
            });
            if let Some(e) = &self.editor {
                self.brush.layer = self
                    .brush
                    .layer
                    .min(e.document().geometry().counts()[self.brush.axis] - 1);
                ui.add(
                    egui::Slider::new(
                        &mut self.brush.layer,
                        0..=e.document().geometry().counts()[self.brush.axis] - 1,
                    )
                    .text("Layer"),
                );
                if let Ok(position) = self.brush.slice_position(e.document().geometry()) {
                    ui.small(format!("Position {position:.5} m"));
                }
            }
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.brush.depth, BrushDepth::Layer, "Layer");
                ui.selectable_value(&mut self.brush.depth, BrushDepth::Volume, "Volume");
            });
        });
        if self.brush != before {
            self.cancel();
        }
        if self.brush.operation == BrushOperation::Add {
            ui.small("Add fills Vacuum. For occupied cells, use Replace.");
        }
        if ui.checkbox(&mut self.show_air, "Show Air").changed() {
            self.cancel();
        }
        let busy = self.busy();
        ui.horizontal_wrapped(|ui| {
            let undo = self
                .editor
                .as_ref()
                .is_some_and(|e| e.undo_label().is_some() || e.active());
            let redo = self
                .editor
                .as_ref()
                .is_some_and(|e| e.redo_label().is_some());
            if ui
                .add_enabled(
                    editable && undo && (!busy || self.gesture.is_some()),
                    egui::Button::new("Undo Contents"),
                )
                .clicked()
            {
                self.command(Action::Undo);
            }
            if ui
                .add_enabled(
                    editable && !busy && redo,
                    egui::Button::new("Redo Contents"),
                )
                .clicked()
            {
                self.command(Action::Redo);
            }
        });
        if ui
            .add_enabled(editable && !busy, egui::Button::new("Fill domain with Air"))
            .clicked()
            && let Some(e) = &mut self.editor
        {
            let r = e.fill_air();
            self.outcome(r);
        }
        ui.collapsing("Details", |ui| {
            let axes = self.plane_axes();
            ui.add_enabled_ui(editable && !self.busy(), |ui| {
                for (index, axis) in axes.into_iter().enumerate() {
                    ui.horizontal(|ui| {
                        let label = ui.label(format!("Center {} · m", ["X", "Y", "Z"][axis]));
                        ui.add(egui::DragValue::new(&mut self.keyboard[index]).speed(0.001))
                            .labelled_by(label.id);
                    });
                }
                if ui.button("Apply dab").clicked() {
                    self.apply_dab();
                }
            });
            if let Some(e) = &self.editor {
                let counts = e.document().counts();
                let pair = e.document().materials();
                let v = e.document().geometry().cell_volume();
                ui.label(format!(
                    "Water: {} cells · {:.6} m³ · {:.6} kg",
                    counts[1],
                    counts[1] as f64 * v,
                    counts[1] as f64 * v * pair.liquid().density()
                ));
                ui.label(format!(
                    "Air: {} cells · {:.6} m³ · {:.6} kg",
                    counts[2],
                    counts[2] as f64 * v,
                    counts[2] as f64 * v * pair.gas().density()
                ));
                ui.small(format!(
                    "Vacuum: {} cells · revision {}",
                    counts[0],
                    e.document().revision()
                ));
                ui.small(format!(
                    "Undo floor: {} discarded commands · {:.2} MiB history",
                    e.evicted_commands(),
                    e.history_bytes() as f64 / 1048576.0
                ));
            }
        });
        self.file_controls(ui, editable);
        if ui
            .add_enabled(!busy, egui::Button::new("Validate"))
            .clicked()
            && let Some(e) = &self.editor
        {
            let r = e.document().validate();
            self.outcome(r);
        }
        if let Some(e) = &self.editor
            && let Err(reason) = e.document().validate()
        {
            ui.colored_label(Color32::YELLOW, reason.to_string());
        }
        if ui
            .add_enabled(
                run_readiness.is_ok() && editable && !busy,
                egui::Button::new("Simulate contents"),
            )
            .on_disabled_hover_text(run_readiness.as_ref().err().map_or("", String::as_str))
            .clicked()
        {
            self.run_requested = true;
        }
        if let Some(error) = &self.error {
            ui.colored_label(Color32::LIGHT_RED, error);
        }
        ui.small("Water · blue   Air · amber   Vacuum · dark\nPrimary paints · middle navigates · Esc cancels");
    }
    fn pick(
        &self,
        camera: &OrbitCamera,
        rect: Rect,
        side: f64,
        pos: Pos2,
    ) -> Result<[f64; 3], ContentsError> {
        let editor = self.editor.as_ref().ok_or(ContentsError::InvalidIdentity)?;
        if !rect.is_finite()
            || rect.height() <= 0.0
            || !rect.contains(pos)
            || !pos.is_finite()
            || !side.is_normal()
            || side <= 0.0
        {
            return Err(ContentsError::InvalidPoint);
        }
        let (eye, forward, right, up) = camera.basis();
        let dx = f64::from(pos.x - rect.center().x) / f64::from(rect.height() * 0.88);
        let dy = f64::from(rect.center().y - pos.y) / f64::from(rect.height() * 0.88);
        let offset: [f64; 3] =
            std::array::from_fn(|d| f64::from(right[d]) * dx + f64::from(up[d]) * dy);
        let (origin, direction) = if camera.orthographic {
            (
                std::array::from_fn(|d| {
                    (f64::from(eye[d]) + offset[d] * f64::from(camera.distance)) * side
                }),
                forward.map(f64::from),
            )
        } else {
            (
                eye.map(|v| f64::from(v) * side),
                std::array::from_fn(|d| f64::from(forward[d]) + offset[d]),
            )
        };
        // Core pick normalizes direction and takes near in physical metres.
        let forward_rate = direction
            .iter()
            .zip(forward)
            .map(|(a, b)| a * f64::from(b))
            .sum::<f64>();
        if forward_rate <= 0.0 {
            return Err(ContentsError::InvalidPoint);
        }
        let magnitude = direction.into_iter().fold(0.0_f64, f64::hypot);
        self.brush.pick(
            editor.document().geometry(),
            origin,
            direction,
            0.0151 * side * magnitude / forward_rate,
        )
    }
    /// True consumes the complete terminal frame and every latched release.
    pub fn canvas(
        &mut self,
        ui: &egui::Ui,
        response: &egui::Response,
        camera: &OrbitCamera,
        side: f64,
        selected: bool,
        editable: bool,
    ) -> bool {
        let (events, focused, down, latest) = ui.input(|i| {
            (
                i.events.clone(),
                i.focused,
                i.pointer.button_down(egui::PointerButton::Primary),
                i.pointer.latest_pos(),
            )
        });
        if self.drain_frame.take() == Some(ui.ctx().cumulative_frame_nr()) {
            self.latch = down;
            if selected {
                self.paint(ui, response.rect, camera, side, latest);
            }
            return true;
        }
        let released = events.iter().any(|e| {
            matches!(
                e,
                egui::Event::PointerButton {
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    ..
                }
            )
        });
        if self.latch {
            if released {
                self.latch = down;
            }
            if selected {
                self.paint(ui, response.rect, camera, side, latest);
            }
            return true;
        }
        let had_gesture = self.gesture.is_some();
        if let Some(g) = &self.gesture
            && (!selected
                || !editable
                || !focused
                || response.lost_focus()
                || *camera != g.camera
                || response.rect != g.rect
                || side.to_bits() != g.side.to_bits()
                || (!down && !released)
                || events.iter().any(|e| matches!(e, egui::Event::PointerGone)))
        {
            self.cancel();
            if released {
                self.latch = down;
            }
            if selected {
                self.paint(ui, response.rect, camera, side, latest);
            }
            return true;
        }
        if let Some(action) = authoring_history::action(ui, selected && editable) {
            self.command(action);
            if released {
                self.latch = down;
            }
            self.paint(ui, response.rect, camera, side, latest);
            return true;
        }
        let mut consumed = had_gesture;
        for event in events {
            match event {
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers,
                } if selected
                    && editable
                    && focused
                    && modifiers.is_none()
                    && response.rect.contains(pos) =>
                {
                    consumed = true;
                    let result: Result<[f64; 3], ContentsError> = (|| {
                        let hit = self.pick(camera, response.rect, side, pos)?;
                        let e = self.editor.as_mut().ok_or(ContentsError::InvalidIdentity)?;
                        e.begin(self.brush)?;
                        e.segment(hit, hit)?;
                        Ok(hit)
                    })();
                    match result {
                        Ok(previous) => {
                            response.request_focus();
                            self.gesture = Some(Gesture {
                                camera: camera.clone(),
                                rect: response.rect,
                                side,
                                previous,
                            });
                            self.error = None;
                        }
                        Err(error) => {
                            self.cancel();
                            self.latch = true;
                            self.error = Some(error.to_string());
                        }
                    }
                }
                egui::Event::PointerMoved(pos) if self.gesture.is_some() => {
                    let result = self.pick(camera, response.rect, side, pos);
                    match result {
                        Ok(hit) => {
                            let g = self.gesture.as_mut().unwrap();
                            let result = self.editor.as_mut().unwrap().segment(g.previous, hit);
                            if result.is_ok() {
                                g.previous = hit;
                            } else {
                                self.cancel();
                            }
                            self.outcome(result);
                        }
                        Err(error) => {
                            self.cancel();
                            self.error = Some(error.to_string());
                        }
                    }
                }
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    ..
                } if self.gesture.is_some() => {
                    let result = (|| {
                        let hit = self.pick(camera, response.rect, side, pos)?;
                        let previous = self.gesture.as_ref().unwrap().previous;
                        let e = self.editor.as_mut().unwrap();
                        e.segment(previous, hit)?;
                        e.commit()
                    })();
                    if result.is_err() {
                        self.cancel();
                    }
                    self.gesture = None;
                    self.latch = down;
                    self.outcome(result);
                    consumed = true;
                }
                _ => {}
            }
            if self.latch {
                if released {
                    self.latch = down;
                }
                break;
            }
        }
        if selected {
            self.paint(ui, response.rect, camera, side, latest);
        }
        consumed
    }
    fn paint(
        &self,
        ui: &egui::Ui,
        rect: Rect,
        camera: &OrbitCamera,
        side: f64,
        pointer: Option<Pos2>,
    ) {
        if !side.is_normal()
            || side <= 0.0
            || !rect.is_finite()
            || !camera.distance.is_normal()
            || camera.distance <= 0.0
            || !camera.yaw.is_finite()
            || !camera.pitch.is_finite()
            || !camera.target.iter().all(|x| x.is_finite())
        {
            return;
        }
        let Some(e) = &self.editor else {
            return;
        };
        let g = e.document().geometry();
        let Ok(plane) = self.brush.slice_position(g) else {
            return;
        };
        let axes = self.plane_axes();
        let n = g.counts();
        let painter = ui.painter_at(rect);
        let project = |p: [f64; 3]| {
            camera
                .project(p.map(|v| (v / side) as f32), rect)
                .map(|p| p.0)
        };
        for j in 0..n[axes[1]] {
            for i in 0..n[axes[0]] {
                let mut cell = [0; 3];
                cell[self.brush.axis] = self.brush.layer;
                cell[axes[0]] = i;
                cell[axes[1]] = j;
                let id = e.preview_cells()[g.cell_index(cell).unwrap()];
                if id == 2 && !self.show_air {
                    continue;
                }
                let points = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]].map(|offset| {
                    let mut p = [0.0; 3];
                    p[self.brush.axis] = plane;
                    for d in 0..2 {
                        let axis = axes[d];
                        p[axis] =
                            g.origin()[axis] + (cell[axis] as f64 + offset[d]) * g.spacing()[axis];
                    }
                    p
                });
                let color = match id {
                    1 => Color32::from_rgb(45, 125, 215),
                    2 => Color32::from_rgb(135, 103, 49),
                    _ => Color32::from_rgb(35, 40, 48),
                };
                // Clip in normalized camera space before projection, including when
                // the camera is inside the volume or the slice crosses near.
                let mut polygon = points.map(|p| p.map(|v| (v / side) as f32)).to_vec();
                let (eye, forward, _, _) = camera.basis();
                let depth = |p: [f32; 3]| (0..3).map(|d| (p[d] - eye[d]) * forward[d]).sum::<f32>();
                let mut clipped = Vec::with_capacity(5);
                for index in 0..polygon.len() {
                    let a = polygon[index];
                    let b = polygon[(index + 1) % polygon.len()];
                    let da = depth(a);
                    let db = depth(b);
                    if da >= 0.0151 {
                        clipped.push(a);
                    }
                    if (da >= 0.0151) != (db >= 0.0151) {
                        let t = (0.0151 - da) / (db - da);
                        clipped.push(std::array::from_fn(|d| a[d] + t * (b[d] - a[d])));
                    }
                }
                polygon = clipped;
                let screen = polygon
                    .into_iter()
                    .filter_map(|p| camera.project(p, rect).map(|p| p.0))
                    .collect::<Vec<_>>();
                if screen.len() >= 3 {
                    painter.add(egui::Shape::convex_polygon(
                        screen,
                        color,
                        Stroke::new(0.5, Color32::from_gray(65)),
                    ));
                }
            }
        }
        if let Some(pos) = pointer
            && let Ok(hit) = self.pick(camera, rect, side, pos)
        {
            // Volume preview includes three great circles, so its complete
            // depth is visible; Layer uses the slice circle alone.
            let pairs = if self.brush.depth == BrushDepth::Layer {
                vec![(axes[0], axes[1])]
            } else {
                vec![(0, 1), (0, 2), (1, 2)]
            };
            for (a, b) in pairs {
                let mut last = None;
                for step in 0..=64 {
                    let angle = step as f64 * std::f64::consts::TAU / 64.0;
                    let mut p = hit;
                    p[a] += self.brush.radius * angle.cos();
                    p[b] += self.brush.radius * angle.sin();
                    let current = project(p);
                    if let (Some(x), Some(y)) = (last, current) {
                        painter.line_segment([x, y], Stroke::new(1.5, Color32::WHITE));
                    }
                    last = current;
                }
            }
        }
        let caption = if e.active() {
            let changes = e
                .preview_cells()
                .iter()
                .zip(e.document().cells())
                .filter(|(a, b)| a != b)
                .count();
            format!("Draft initial contents · {changes} pending cell changes")
        } else {
            "Draft initial contents · slice · exact cell centers".into()
        };
        painter.text(
            rect.left_top() + egui::vec2(12.0, 12.0),
            egui::Align2::LEFT_TOP,
            caption,
            egui::FontId::proportional(13.0),
            Color32::WHITE,
        );
    }
}

#[cfg(test)]
#[path = "initial_contents_ui_tests.rs"]
mod tests;
