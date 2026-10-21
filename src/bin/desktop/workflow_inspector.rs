//! Cached, read-only native projection of numerical workflow receipts.
//! Selection and explicit archive reads happen before paint; rendering never
//! starts a provider, reads a package, or opens a recording.
use super::inspector::Inspector;
use eframe::egui;
use rheon::{
    recording::{RecordedStep, RecordingRun},
    workflow::{EdgeKind, NodeKind, NodeStatus, WorkflowAttempt},
};
use std::sync::Arc;

const LANES: [&str; 5] = [
    "Geometry",
    "State",
    "Algorithm",
    "Qualification",
    "Publication",
];
const NODE_SIZE: egui::Vec2 = egui::vec2(174.0, 58.0);

#[derive(Default)]
pub struct WorkflowInspector {
    pub open: bool,
    source: Option<(RecordingRun, Arc<RecordedStep>)>,
    attempt: Option<Arc<WorkflowAttempt>>,
    message: String,
    selected: usize,
    zoom: f32,
    pan: egui::Vec2,
    pan_drag: Option<(egui::Pos2, egui::Vec2)>,
    evidence: bool,
    restore_focus: bool,
    terminal_request: bool,
    terminal_mode: bool,
    identity: Option<rheon::SessionIdentity>,
    return_focus: Option<egui::Id>,
    evidence_focus: Option<egui::Id>,
    navigated: bool,
    focus_text: bool,
    comparison_key: Option<super::saved_comparison::WorkflowComparisonKey>,
    comparison_source: Option<(RecordingRun, usize)>,
    comparison_context: bool,
    #[cfg(test)]
    controls: std::collections::BTreeMap<String, egui::Rect>,
}
impl WorkflowInspector {
    /// Consumes only an immutable cached exact publication, including clearing
    /// the previous graph when the upstream selection becomes unavailable.
    pub fn select(&mut self, inspector: Option<&Inspector>) {
        if self.comparison_context {
            self.terminal_request = false;
            self.terminal_mode = false;
        }
        self.comparison_key = None;
        self.comparison_source = None;
        self.comparison_context = false;
        let identity = inspector.and_then(Inspector::workflow_identity);
        let identity_changed = self.identity != identity;
        self.identity = identity;
        let cached = inspector.map_or(Ok(None), Inspector::workflow_selection);
        let attempt = cached
            .as_ref()
            .ok()
            .and_then(|s| s.as_ref())
            .and_then(|selection| selection.attempt.clone());
        let selection =
            cached.map(|selection| selection.map(|selection| (selection.run, selection.step)));
        if self.terminal_request {
            self.terminal_request = false;
            let source = selection.as_ref().ok().and_then(|s| s.as_ref()).cloned();
            self.terminal_mode = true;
            match inspector
                .ok_or_else(|| "No recorded source selected".to_string())
                .and_then(Inspector::load_workflow_terminal)
            {
                Ok(Some(attempt)) => self.set_attempt(attempt),
                Ok(None) => self.unavailable(
                    "Terminal workflow execution evidence unavailable (active or legacy archive).",
                ),
                Err(error) => self.unavailable(&format!(
                    "Invalid terminal workflow execution evidence: {error}"
                )),
            }
            self.source = source;
            return;
        }
        if self.terminal_mode && !identity_changed {
            let unchanged = match (selection.as_ref(), self.source.as_ref()) {
                (Ok(None), None) => true,
                (Ok(Some((run, step))), Some((old_run, old_step))) => {
                    old_run == run && Arc::ptr_eq(old_step, step)
                }
                _ => false,
            };
            if unchanged {
                return;
            }
        }
        self.terminal_mode = false;
        match selection {
            Ok(Some((run, step))) => {
                if self.source.as_ref().is_some_and(|(old_run, old_step)| {
                    old_run == &run && Arc::ptr_eq(old_step, &step)
                }) {
                    return;
                }
                self.attempt = None;
                self.message.clear();
                self.evidence = false;
                match attempt {
                    Some(attempt) => self.set_attempt(attempt),
                    None => self.message = "Workflow execution evidence unavailable for this publication (constructor or legacy archive).".into(),
                }
                self.source = Some((run, step));
            }
            Ok(None) => self.unavailable(
                "Workflow execution evidence unavailable: no exact recorded publication selected.",
            ),
            Err(error) => self.unavailable(&format!("Workflow selection unavailable: {error}")),
        }
    }
    fn unavailable(&mut self, message: &str) {
        self.pan_drag = None;
        self.source = None;
        self.attempt = None;
        self.evidence = false;
        self.message = message.into();
    }
    pub fn load_comparison(
        &mut self,
        loaded: Result<
            Option<super::saved_comparison::SelectedWorkflow>,
            super::saved_comparison::SelectedFieldsError,
        >,
    ) {
        self.unavailable(
            "Workflow execution evidence unavailable: choose a verified comparison publication.",
        );
        self.comparison_key = None;
        self.comparison_source = None;
        self.comparison_context = true;
        self.terminal_request = false;
        self.terminal_mode = false;
        match loaded {
            Ok(Some(loaded)) => {
                self.comparison_key = Some(loaded.key);
                self.comparison_source = Some((loaded.run, loaded.ordinal));
                match loaded.attempt {Some(attempt)=>self.set_attempt(attempt),None=>self.message="Workflow execution evidence unavailable for this comparison publication (constructor or legacy archive).".into()}
            }
            Ok(None) => {}
            Err(error) => {
                self.message =
                    format!("Invalid or unavailable selected comparison workflow: {error}")
            }
        }
    }
    pub fn comparison_side(&self) -> Option<super::comparison_archive::Side> {
        self.comparison_key.as_ref().map(|key| key.side)
    }
    pub fn validate_comparison(
        &mut self,
        current: Option<&super::saved_comparison::WorkflowComparisonKey>,
    ) {
        if !self.comparison_context
            || (self.comparison_key.is_some() && self.comparison_key.as_ref() != current)
        {
            self.unavailable("Workflow execution evidence unavailable: comparison source, recipe or selected pair changed; inspect the exact side again.");
            self.comparison_key = None;
            self.comparison_source = None;
            self.comparison_context = true;
        }
    }
    fn set_attempt(&mut self, attempt: Arc<WorkflowAttempt>) {
        self.pan_drag = None;
        match attempt.validate() {
            Ok(()) => {
                self.selected = self.selected.min(attempt.descriptor.nodes().len() - 1);
                self.attempt = Some(attempt);
                self.message.clear();
                if self.zoom == 0.0 {
                    self.reset();
                }
            }
            Err(error) => {
                self.unavailable(&format!("Invalid workflow execution evidence: {error}"))
            }
        }
    }
    pub fn open(&mut self) {
        self.pan_drag = None;
        self.open = true;
        self.focus_text = false;
        self.restore_focus = true;
    }
    pub fn open_from(&mut self, trigger: egui::Id) {
        self.open();
        self.return_focus = Some(trigger);
    }
    fn reset(&mut self) {
        self.pan_drag = None;
        self.zoom = 0.85;
        self.pan = egui::Vec2::ZERO;
    }
    fn control(&mut self, name: impl Into<String>, response: egui::Response) -> egui::Response {
        #[cfg(test)]
        self.controls.insert(name.into(), response.rect);
        #[cfg(not(test))]
        let _ = name;
        response
    }
    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.open {
            self.pan_drag = None;
            return;
        }
        if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            if self.evidence {
                self.evidence = false;
                if let Some(id) = self.evidence_focus {
                    ctx.memory_mut(|memory| memory.request_focus(id));
                }
            } else {
                self.open = false;
                if let Some(id) = self.return_focus {
                    ctx.memory_mut(|memory| memory.request_focus(id));
                }
                return;
            }
        }
        let mut open = true;
        egui::Window::new("Executed workflow")
            .id(egui::Id::new("executed-workflow"))
            .open(&mut open)
            .default_width(1100.0)
            .default_height(700.0)
            .resizable(true)
            .show(ctx, |ui| self.ui(ui));
        if !open {
            self.open = false;
            self.evidence = false;
        }
        if self.evidence {
            let mut open = true;
            egui::Window::new("Retained workflow evidence")
                .id(egui::Id::new("retained-workflow-evidence"))
                .open(&mut open)
                .default_width(640.0)
                .show(ctx, |ui| {
                    ui.label("Exact retained attempt; no provider execution is needed for inspection.");
                    let scroll = egui::ScrollArea::vertical()
                        .id_salt("workflow-evidence-scroll")
                        .max_height((ctx.content_rect().bottom() - ui.cursor().top() - 60.0).clamp(80.0, 600.0))
                        .show(ui, |ui| {
                    if let Some(attempt) = &self.attempt {
                        ui.monospace(format!("descriptor SHA-256 {}", hex(&attempt.descriptor_sha256)));
                        ui.monospace(format!("Accepted native input state SHA-256 {}", hex(&attempt.input_sha256)));
                        ui.monospace(format!("Actual request controls SHA-256 {}", hex(&attempt.request_sha256)));
                        ui.label(format!("{} prescribed forces · smoke source {} · volume source {}", attempt.force_count, attempt.smoke_source_present, attempt.volume_source_present));
                        ui.label(format!("Run {} · attempt {} · publication {:?}", attempt.run_id, attempt.attempt_id, attempt.publication));
                        ui.label(&attempt.diagnostic);
                        if let Some((run, step)) = &self.source {
                            ui.label(format!("Source {} · recorded ordinal {}", run.identity.source_revision, step.ordinal));
                            ui.label(&run.provenance);
                        }
                        if let Some((run, ordinal)) = &self.comparison_source {
                            ui.label(format!("Verified comparison source {} · recorded ordinal {}", run.identity.source_revision, ordinal));
                            ui.label(&run.provenance);
                        }
                        if let Some(pressure) = &attempt.pressure {
                            if let Some(binding) = &pressure.binding {
                                ui.label(format!("Provider binding: {} / {}", binding.id, binding.version));
                                ui.monospace(format!("Provider manifest SHA-256 {}", hex(&binding.manifest_sha256)));
                                ui.monospace(format!("Provider checkpoint SHA-256 {}", binding.checkpoint_sha256.map_or_else(|| "unavailable".into(), |hash| hex(&hash))));
                                ui.label(format!("Provider runtime {} · source {} · training run {}", binding.runtime, binding.source_revision, binding.training_run_id.as_deref().unwrap_or("unavailable")));
                            } else {
                                ui.label("Configured provider binding unavailable; native selection is recorded separately.");
                            }
                            if let Some(hash) = pressure.rhs_sha256 { ui.monospace(format!("Input RHS SHA-256 {}", hex(&hash))); }
                        }
                    }
                        });
                    #[cfg(test)]
                    self.controls.insert("evidence-scroll".into(), scroll.inner_rect);
                    #[cfg(not(test))]
                    let _ = scroll;
                    let close = ui.button("Close evidence");
                    if self.control("close-evidence", close).clicked() { self.evidence = false; }
                });
            self.evidence &= open;
            if !self.evidence
                && let Some(id) = self.evidence_focus
            {
                ctx.memory_mut(|memory| memory.request_focus(id));
            }
        }
        if !self.open
            && let Some(id) = self.return_focus
        {
            ctx.memory_mut(|memory| memory.request_focus(id));
        }
    }
    fn ui(&mut self, ui: &mut egui::Ui) {
        self.navigated = false;
        #[cfg(test)]
        self.controls.clear();
        ui.label("Read-only native sealed MAC composition and actual execution receipts");
        if let Some(key) = &self.comparison_key {
            ui.label(format!("Verified comparison side: {:?}", key.side));
        }
        ui.horizontal(|ui| {
            if self.comparison_context { ui.small("Exact verified comparison publication. Inspect its archive for terminal attempts."); }
            else {
            if self
                .control("terminal", ui.button("Load terminal attempt"))
                .clicked()
            {
                self.terminal_request = true;
            }
            if self
                .control("publication", ui.button("Inspect selected publication"))
                .clicked()
            {
                self.source = None;
                self.terminal_mode = false;
            }
            }
        });
        if !self.message.is_empty() {
            ui.label(&self.message);
        }
        let Some(attempt) = self.attempt.clone() else {
            if ui.button("Close workflow").clicked() {
                self.open = false;
            }
            return;
        };
        ui.horizontal_wrapped(|ui| {
            ui.label(format!(
                "Run {} · attempt {} · {:?}",
                attempt.run_id, attempt.attempt_id, attempt.outcome
            ));
            ui.label(format!(
                "Input generation {} → attempted {} · publication {:?}",
                attempt.accepted_generation, attempt.attempted_generation, attempt.publication
            ));
        });
        if let Some(pressure) = &attempt.pressure {
            ui.label(format!(
                "Actual pressure provider: {} · fallback/rejection: {}",
                if pressure.selected.is_empty() {
                    "unavailable"
                } else {
                    &pressure.selected
                },
                pressure.rejection.as_deref().unwrap_or("none recorded")
            ));
        }
        ui.horizontal_wrapped(|ui| {
            let response =
                ui.add(egui::Slider::new(&mut self.zoom, 0.5..=1.5).text("Workflow zoom"));
            self.control("zoom", response);
            let response = ui.button("Reset workflow view");
            if self.control("reset", response).clicked() {
                self.reset();
            }
            let response = ui.button("Open retained evidence");
            let response = self.control("evidence", response);
            if response.clicked() {
                self.evidence = true;
                self.evidence_focus = Some(response.id);
            }
            let response = ui.button("Close workflow");
            if self.control("close", response).clicked() {
                self.open = false;
                self.evidence = false;
            }
        });
        ui.small("Select a node with the pointer or Tab. Arrow keys / Home / End move the focused workflow selection. Drag background to pan; use zoom or Reset. Escape closes this view.");
        self.canvas(ui, &attempt);
        ui.separator();
        egui::ScrollArea::vertical()
            .id_salt("workflow-details")
            .max_height(300.0)
            .show(ui, |ui| {
                self.details(ui, &attempt);
                egui::CollapsingHeader::new("Pressure iteration loop details").show(ui, |ui| {
                    ui.label(format!("Native iteration cap {}", attempt.descriptor.pressure.max_iterations));
                    if let Some(report) = attempt.pressure.as_ref().and_then(|p| p.accepted) {
                        ui.label(format!("Accepted pressure iterations: {}", report.iterations));
                    } else { ui.label("Actual iteration count unavailable"); }
                    ui.small("Bounded summary only. Per-iteration residual history and whole-workflow timing are unavailable.");
                });
                egui::CollapsingHeader::new(
                    "Ordered text equivalent: nodes and typed dependencies",
                )
                .default_open(true)
                .show(ui, |ui| {
                    for (index, node) in attempt.descriptor.nodes().iter().copied().enumerate() {
                        let response = ui.scope_builder(egui::UiBuilder::new().id(("workflow-text-node", node as u8)), |ui| ui.selectable_label(
                            self.selected == index,
                            format!(
                                "{} · {} · {}",
                                index + 1,
                                node.label(),
                                status(&attempt, node)
                            ),
                        )).inner;
                        let response = self.control(format!("text-{index}"), response);
                        if self.restore_focus && self.focus_text && self.selected == index {
                            response.request_focus();response.scroll_to_me_animation(Some(egui::Align::Center), egui::style::ScrollAnimation::none());self.restore_focus=false;
                        }
                        if response.clicked() || (response.gained_focus() && !self.navigated) {
                            self.selected = index;self.focus_text=true;
                            if response.clicked(){response.request_focus();}
                        }
                        self.keyboard(ui, &response, attempt.descriptor.nodes().len());
                    }
                    for edge in attempt.descriptor.edges() {
                        ui.small(format!(
                            "{} → {} · {:?} · {:?}, {} · {}",
                            edge.from.label(),
                            edge.to.label(),
                            edge.kind,
                            edge.port,
                            edge.port.units(),
                            edge.port.association()
                        ));
                    }
                });
            });
    }
    fn keyboard(&mut self, ui: &egui::Ui, response: &egui::Response, count: usize) {
        if !response.has_focus() || self.navigated {
            return;
        }
        ui.memory_mut(|memory| {
            memory.set_focus_lock_filter(
                response.id,
                egui::EventFilter {
                    horizontal_arrows: true,
                    vertical_arrows: true,
                    escape: true,
                    tab: false,
                },
            )
        });
        ui.memory_mut(|memory| memory.move_focus(egui::FocusDirection::None));
        let previous = self.selected;
        ui.input_mut(|input| {
            if input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowRight)
                || input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown)
            {
                self.selected = (self.selected + 1).min(count - 1);
            }
            if input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowLeft)
                || input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp)
            {
                self.selected = self.selected.saturating_sub(1);
            }
            if input.consume_key(egui::Modifiers::NONE, egui::Key::Home) {
                self.selected = 0;
            }
            if input.consume_key(egui::Modifiers::NONE, egui::Key::End) {
                self.selected = count - 1;
            }
            if input.consume_key(egui::Modifiers::NONE, egui::Key::Escape) {
                self.open = false;
                self.evidence = false;
            }
        });
        self.restore_focus |= self.selected != previous;
        self.navigated = self.selected != previous;
    }
    fn canvas(&mut self, ui: &mut egui::Ui, attempt: &WorkflowAttempt) {
        let size = egui::vec2(ui.available_width().max(300.0), 390.0);
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::drag());
        let response = self.control("canvas", response);
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 5.0, ui.visuals().extreme_bg_color);
        let position = |node: NodeKind| {
            let rank = match node {
                NodeKind::Geometry | NodeKind::AcceptedState => 0,
                NodeKind::Interval => 1,
                NodeKind::Advection => 2,
                NodeKind::Forces => 3,
                NodeKind::Viscosity => 4,
                NodeKind::PressureRequest => 5,
                NodeKind::PressureCandidate => 6,
                NodeKind::PressureQualification | NodeKind::NativeFallback => 7,
                NodeKind::Projection => 8,
                NodeKind::TracerTransport => 9,
                NodeKind::VolumeTransport => 10,
                NodeKind::Publication => 11,
            };
            egui::pos2(node.lane() as f32 * 200.0 + 12.0, rank as f32 * 72.0 + 32.0)
        };
        if response.drag_started() {
            self.pan_drag = ui
                .input(|input| input.pointer.press_origin())
                .filter(|origin| {
                    rect.contains(*origin)
                        && !attempt.descriptor.nodes().iter().any(|node| {
                            egui::Rect::from_min_size(
                                rect.min + self.pan + position(*node).to_vec2() * self.zoom,
                                (NODE_SIZE * self.zoom).max(egui::vec2(80.0, 36.0)),
                            )
                            .contains(*origin)
                        })
                })
                .map(|origin| (origin, self.pan));
        }
        if response.dragged() {
            if let Some(((origin, initial_pan), pointer)) =
                self.pan_drag.zip(response.interact_pointer_pos())
            {
                self.pan = (initial_pan + (pointer - origin))
                    .clamp(egui::vec2(-1500.0, -1500.0), egui::vec2(1500.0, 1500.0));
            }
        } else {
            self.pan_drag = None;
        }
        if self.restore_focus && !self.focus_text {
            let node_position = position(attempt.descriptor.nodes()[self.selected]).to_vec2()
                * self.zoom
                + self.pan;
            if node_position.x < 0.0 {
                self.pan.x -= node_position.x;
            }
            if node_position.x + NODE_SIZE.x * self.zoom > rect.width() {
                self.pan.x -= node_position.x + NODE_SIZE.x * self.zoom - rect.width();
            }
            if node_position.y < 0.0 {
                self.pan.y -= node_position.y;
            }
            if node_position.y + NODE_SIZE.y * self.zoom > rect.height() {
                self.pan.y -= node_position.y + NODE_SIZE.y * self.zoom - rect.height();
            }
        }
        let zoom = self.zoom;
        let pan = self.pan;
        let transform = |p: egui::Pos2| rect.min + pan + p.to_vec2() * zoom;
        let mut graph = ui.new_child(egui::UiBuilder::new().max_rect(rect));
        graph.set_clip_rect(rect);
        for (lane, label) in LANES.iter().enumerate() {
            painter.text(
                transform(egui::pos2(lane as f32 * 200.0 + 12.0, 10.0)),
                egui::Align2::LEFT_TOP,
                label,
                egui::FontId::proportional(13.0),
                ui.visuals().text_color(),
            );
        }
        for edge in attempt.descriptor.edges() {
            let a = transform(position(edge.from) + NODE_SIZE * 0.5);
            let b = transform(position(edge.to) + NODE_SIZE * 0.5);
            let color = match edge.kind {
                EdgeKind::Fallback => egui::Color32::from_rgb(235, 175, 80),
                EdgeKind::TimeLoop | EdgeKind::IterationLoop => {
                    egui::Color32::from_rgb(150, 165, 220)
                }
                EdgeKind::Data => ui.visuals().weak_text_color(),
            };
            if edge.from == edge.to {
                let start = a + egui::vec2(NODE_SIZE.x * 0.5, 0.0) * self.zoom;
                let outer = start + egui::vec2(35.0, 0.0) * self.zoom;
                let top = outer + egui::vec2(0.0, -55.0) * self.zoom;
                let inner = egui::pos2(a.x, top.y);
                let end = a - egui::vec2(0.0, NODE_SIZE.y * 0.5) * self.zoom;
                painter.add(egui::Shape::line(
                    vec![start, outer, top, inner],
                    egui::Stroke::new(1.0, color),
                ));
                painter.arrow(inner, end - inner, egui::Stroke::new(1.0, color));
                painter.text(
                    outer + egui::vec2(4.0, -20.0),
                    egui::Align2::LEFT_CENTER,
                    "pressure iteration loop",
                    egui::FontId::proportional(10.0),
                    color,
                );
            } else {
                painter.arrow(a, b - a, egui::Stroke::new(1.0, color));
            }
            painter.circle_filled(a, 2.5, color);
            painter.circle_filled(b, 2.5, color);
            let selected = attempt.descriptor.nodes()[self.selected];
            if edge.from != edge.to && (edge.from == selected || edge.to == selected) {
                painter.text(
                    a.lerp(b, 0.5),
                    egui::Align2::CENTER_CENTER,
                    format!("{:?} · {} · {:?}", edge.port, edge.port.units(), edge.kind),
                    egui::FontId::proportional(10.0),
                    color,
                );
            }
        }
        for (index, node) in attempt.descriptor.nodes().iter().copied().enumerate() {
            let node_rect = egui::Rect::from_min_size(
                transform(position(node)),
                (NODE_SIZE * self.zoom).max(egui::vec2(80.0, 36.0)),
            );
            if !node_rect.intersects(rect) {
                continue;
            }
            let response = graph
                .scope_builder(
                    egui::UiBuilder::new().id(("workflow-canvas-node", node as u8)),
                    |ui| {
                        ui.put(
                            node_rect,
                            egui::Button::new(
                                egui::RichText::new(format!(
                                    "{}\n{}",
                                    node.label(),
                                    status(attempt, node)
                                ))
                                .size((13.0 * self.zoom).max(10.0)),
                            )
                            .wrap_mode(egui::TextWrapMode::Truncate)
                            .selected(self.selected == index),
                        )
                    },
                )
                .inner;
            let response = self.control(format!("node-{index}"), response);
            if self.restore_focus && !self.focus_text && self.selected == index {
                response.request_focus();
                self.restore_focus = false;
            }
            if response.clicked() || (response.gained_focus() && !self.navigated) {
                self.selected = index;
                self.focus_text = false;
                if response.clicked() {
                    response.request_focus();
                }
            }
            self.keyboard(ui, &response, attempt.descriptor.nodes().len());
            if response.has_focus() {
                painter.rect_stroke(
                    node_rect.expand(3.0),
                    3.0,
                    egui::Stroke::new(2.0, ui.visuals().text_color()),
                    egui::StrokeKind::Inside,
                );
            }
        }
    }
    fn details(&self, ui: &mut egui::Ui, attempt: &WorkflowAttempt) {
        let node = attempt.descriptor.nodes()[self.selected];
        ui.heading(format!("{} · {}", node.label(), status(attempt, node)));
        ui.label(format!(
            "Requested dt {:.9e} s · actual dt {} · interval start {:.9e} s",
            attempt.requested_dt,
            number(attempt.actual_dt, "s"),
            attempt.interval_start
        ));
        ui.small(format!(
            "{} · {}",
            attempt.descriptor.family(),
            attempt.descriptor.operator()
        ));
        ui.label(format!(
            "Density {} kg/m³ · viscosity {}",
            attempt.descriptor.density,
            attempt
                .descriptor
                .viscosity
                .map_or_else(|| "disabled".into(), |v| format!("{v:.9e} Pa·s (applied)"))
        ));
        ui.label(format!(
            "Grid {:?} · spacing {:?} m · origin {:?} m",
            attempt.descriptor.grid.counts(),
            attempt.descriptor.grid.spacing(),
            attempt.descriptor.grid.origin()
        ));
        for edge in attempt
            .descriptor
            .edges()
            .iter()
            .filter(|e| e.from == node || e.to == node)
        {
            ui.small(format!(
                "{} → {}: {:?} · {} · {} · {:?}",
                edge.from.label(),
                edge.to.label(),
                edge.port,
                edge.port.units(),
                edge.port.association(),
                edge.kind
            ));
        }
        if let Some(pressure) = &attempt.pressure {
            ui.label(format!(
                "Actual pressure provider: {} · support {:?} · provider method entered {}",
                if pressure.selected.is_empty() {
                    "unavailable"
                } else {
                    &pressure.selected
                },
                pressure.support,
                pressure.prediction_called
            ));
            ui.label(format!(
                "Candidate rejection / fallback reason: {}",
                pressure.rejection.as_deref().unwrap_or("none recorded")
            ));
            ui.label(format!(
                "Actual host L2 residual acceptance limit: {}",
                number(pressure.residual_l2_limit, "m³/s²")
            ));
            if let Some(report) = pressure.candidate {
                ui.label(format!("Candidate pressure (before host selection): residual L2 {:.6e} m³/s² · max {:.6e} m³/s² · predicted divergence {:.6e} 1/s · iterations {}", report.true_residual_l2, report.true_residual_max, report.predicted_divergence_max, report.iterations));
            } else {
                ui.label("Candidate pressure diagnostics unavailable");
            }
            if let Some(report) = pressure.accepted {
                ui.label(format!("Accepted pressure (selected provider): residual L2 {:.6e} m³/s² · max {:.6e} m³/s² · predicted divergence {:.6e} 1/s · iterations {}", report.true_residual_l2, report.true_residual_max, report.predicted_divergence_max, report.iterations));
            } else {
                ui.label("Accepted pressure diagnostics unavailable");
            }
            ui.label(format!(
                "Pressure-call time {}",
                pressure.pressure_call_ns.map_or_else(
                    || "unavailable".into(),
                    |n| format!("{n} ns (pressure call only)")
                )
            ));
        } else {
            ui.label("Pressure execution evidence unavailable for this attempt");
        }
        ui.label(format!("Pressure limits: relative residual {:.6e} (dimensionless) · absolute residual {:.6e} m³/s² · predicted divergence {:.6e} 1/s · iteration cap {}", attempt.descriptor.pressure.relative_residual, attempt.descriptor.pressure.absolute_residual, attempt.descriptor.pressure.divergence_limit, attempt.descriptor.pressure.max_iterations));
        ui.label(format!(
            "Actual divergence {} · limit {:.6e} 1/s",
            number(attempt.actual_divergence_max, "1/s"),
            attempt.descriptor.actual_divergence_limit
        ));
        ui.label(&attempt.diagnostic);
    }
}
fn status(attempt: &WorkflowAttempt, node: NodeKind) -> &'static str {
    match attempt
        .nodes
        .iter()
        .rev()
        .find(|receipt| receipt.node == node)
        .map(|r| r.status)
    {
        Some(NodeStatus::Started) => "incomplete: started",
        Some(NodeStatus::Completed) => "✓ completed",
        Some(NodeStatus::CandidateReturned) => "candidate returned",
        Some(NodeStatus::Rejected) => "! rejected",
        Some(NodeStatus::Selected) => "→ selected",
        Some(NodeStatus::Accepted) => "✓ accepted",
        None => "execution unavailable",
    }
}
fn number(value: Option<f64>, units: &str) -> String {
    value.map_or_else(
        || "unavailable".into(),
        |value| format!("{value:.9e} {units}"),
    )
}
fn hex(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
#[path = "workflow_inspector_tests.rs"]
mod tests;
