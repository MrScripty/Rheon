//! Canonical authoring values and one scene selection. No numerical execution.
use eframe::egui;
use rheon::{
    ForceKind, ForceObject, ForceScene, ForceSupport, ForceTime, ForceTransform, ForceUnits,
    SceneObjectId,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SceneContext {
    #[default]
    Draft,
    Retained,
}
#[derive(Clone, Copy)]
pub struct ModelSummary {
    pub triangles: usize,
    pub bounds: ([f64; 3], [f64; 3]),
}
#[derive(Clone, Copy, Debug)]
pub enum NewForce {
    Directional,
    Radial,
    Vortex,
}
impl NewForce {
    fn label(self) -> &'static str {
        match self {
            Self::Directional => "Directional",
            Self::Radial => "Radial",
            Self::Vortex => "Vortex",
        }
    }
}
struct DraftEdit {
    base: ForceObject,
    preview: ForceObject,
    valid: bool,
}
pub struct SceneUi {
    edit: Option<DraftEdit>,
    forces: ForceScene,
    selected: Option<SceneObjectId>,
    next_id: Option<u64>,
    model_id: Option<SceneObjectId>,
    retained_model_id: Option<SceneObjectId>,
    pub context: SceneContext,
    pub open: bool,
    error: Option<String>,
}
impl Default for SceneUi {
    fn default() -> Self {
        Self {
            edit: None,
            forces: ForceScene::default(),
            selected: None,
            next_id: Some(1),
            model_id: None,
            retained_model_id: None,
            context: SceneContext::Draft,
            open: false,
            error: None,
        }
    }
}
impl SceneUi {
    #[cfg(test)]
    pub(super) fn geometry_fixture(forces: ForceScene, model: bool) -> Self {
        Self {
            forces,
            model_id: model.then_some(SceneObjectId(u64::MAX)),
            ..Default::default()
        }
    }

    pub fn draft(&self) -> &ForceScene {
        &self.forces
    }
    pub fn selected_force(&self) -> Option<&ForceObject> {
        (self.context == SceneContext::Draft)
            .then(|| {
                self.forces
                    .objects
                    .iter()
                    .find(|o| Some(o.id) == self.selected)
            })
            .flatten()
    }
    pub fn editing(&self) -> bool {
        self.edit.is_some()
    }
    pub fn display_scene(&self) -> std::borrow::Cow<'_, ForceScene> {
        if let Some(edit) = &self.edit
            && self.edit_matches()
        {
            let mut scene = self.forces.clone();
            if let Some(object) = scene.objects.iter_mut().find(|o| o.id == edit.base.id) {
                *object = edit.preview.clone();
            }
            std::borrow::Cow::Owned(scene)
        } else {
            std::borrow::Cow::Borrowed(&self.forces)
        }
    }
    pub fn begin_edit(&mut self) -> bool {
        if self.edit.is_some() || self.forces.freeze().is_err() {
            return false;
        }
        let Some(base) = self.selected_force().cloned() else {
            return false;
        };
        self.edit = Some(DraftEdit {
            preview: base.clone(),
            base,
            valid: true,
        });
        true
    }
    pub fn edit_matches(&self) -> bool {
        self.edit.as_ref().is_some_and(|edit| {
            self.selected_force().is_some_and(|current| {
                current == &edit.base
                    && serde_json::to_vec(current)
                        .ok()
                        .is_some_and(|bytes| serde_json::to_vec(&edit.base).ok() == Some(bytes))
            })
        })
    }
    pub fn preview_edit(&mut self, candidate: Option<ForceObject>) -> bool {
        let Some(edit) = &self.edit else {
            return false;
        };
        let candidate = candidate.filter(|c| c.id == edit.base.id);
        let valid = candidate.as_ref().is_some_and(|candidate| {
            let mut scene = self.forces.clone();
            let Some(object) = scene.objects.iter_mut().find(|o| o.id == candidate.id) else {
                return false;
            };
            *object = candidate.clone();
            scene.freeze().is_ok()
        });
        let edit = self.edit.as_mut().unwrap();
        edit.valid = valid;
        if valid {
            edit.preview = candidate.unwrap();
        }
        valid
    }
    pub fn finish_edit(&mut self, apply: bool) -> bool {
        let matches = self.edit_matches();
        let Some(edit) = self.edit.take() else {
            return false;
        };
        if apply && matches && edit.valid {
            let mut candidate_scene = self.forces.clone();
            let Some(candidate) = candidate_scene
                .objects
                .iter_mut()
                .find(|o| o.id == edit.base.id)
            else {
                return false;
            };
            *candidate = edit.preview.clone();
            if candidate_scene.freeze().is_err() {
                return false;
            }
            if let Some(object) = self
                .forces
                .objects
                .iter_mut()
                .find(|o| o.id == edit.base.id)
            {
                *object = edit.preview;
                self.validate();
                return true;
            }
        }
        false
    }
    pub fn selected(&self) -> Option<SceneObjectId> {
        self.selected
    }
    pub fn model_id(&self) -> Option<SceneObjectId> {
        match self.context {
            SceneContext::Draft => self.model_id,
            SceneContext::Retained => self.retained_model_id,
        }
    }
    fn allocate(&mut self) -> Result<SceneObjectId, String> {
        let id = self
            .next_id
            .filter(|id| *id != 0)
            .ok_or("Scene object IDs exhausted; previous scene retained")?;
        self.next_id = id.checked_add(1);
        Ok(SceneObjectId(id))
    }
    /// Caller restores old accepted geometry if admission fails. No state changes on error.
    pub fn replace_model(&mut self, present: bool) -> Result<(), String> {
        let replacement = if present {
            Some(self.allocate()?)
        } else {
            None
        };
        if self.selected == self.model_id {
            self.selected = None;
        }
        self.model_id = replacement;
        Ok(())
    }
    pub fn begin_run(&mut self, model_present: bool) {
        self.selected = None;
        self.retained_model_id = if model_present { self.model_id } else { None };
        self.set_context(SceneContext::Retained);
    }
    pub fn clear_run(&mut self) {
        self.retained_model_id = None;
        self.set_context(SceneContext::Draft);
    }
    fn set_context(&mut self, context: SceneContext) {
        if context != self.context {
            self.selected = None;
            self.context = context;
        }
    }
    /// This is the sole selection owner. Viewport picking will call this same API.
    pub fn select(&mut self, id: Option<SceneObjectId>) {
        self.selected = id.filter(|id| {
            Some(*id) == self.model_id()
                || (self.context == SceneContext::Draft
                    && self.forces.objects.iter().any(|o| o.id == *id))
        });
    }
    pub fn launch_readiness(&self) -> Result<(), String> {
        if self.draft().objects.is_empty() {
            Ok(())
        } else {
            Err("Authored forces cannot run yet: runtime and archive integration is pending. Remove authored forces to run the existing setup.".into())
        }
    }
    fn validate(&mut self) {
        self.error = self.forces.freeze().err().map(|e| {
            format!(
                "Invalid authored force: {e}. Check name (256 UTF-8 bytes), support size, orientation and times."
            )
        });
    }
    fn add(&mut self, kind: NewForce) -> Result<SceneObjectId, String> {
        if self.forces.objects.len() >= rheon::MAX_FORCE_OBJECTS {
            return Err("Scene limit: 256 authored forces".into());
        }
        let id = self.allocate()?;
        self.forces.objects.push(ForceObject {
            id,
            name: format!("{} {}", kind.label(), id.0),
            enabled: true,
            transform: ForceTransform::default(),
            units: ForceUnits::Acceleration,
            kind: match kind {
                NewForce::Directional => ForceKind::Directional { vector: [0.0; 3] },
                NewForce::Radial => ForceKind::Radial { strength: 0.0 },
                NewForce::Vortex => ForceKind::Vortex { strength: 0.0 },
            },
            support: ForceSupport::Global,
            time: ForceTime::default(),
        });
        self.selected = Some(id);
        self.validate();
        Ok(id)
    }
    fn remove_selected(&mut self) {
        if let Some(id) = self.selected {
            self.forces.objects.retain(|o| o.id != id);
        }
        self.selected = None;
        self.validate();
    }
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        has_run: bool,
        active_worker: bool,
        model: Option<ModelSummary>,
    ) {
        if !self.open {
            egui::SidePanel::right("scene-collapsed")
                .exact_width(64.0)
                .resizable(false)
                .show(ctx, |ui| {
                    if ui.button("Scene").clicked() {
                        self.open = true;
                    }
                });
            return;
        }
        egui::SidePanel::right("scene-properties").default_width(272.0).min_width(240.0).max_width(340.0).show(ctx,|ui| {
            ui.horizontal_wrapped(|ui| {ui.heading("Scene");if ui.button("Hide scene").clicked(){self.open=false;}});
            let previous=self.context;
            ui.horizontal_wrapped(|ui| {
                ui.selectable_value(&mut self.context,SceneContext::Draft,"Draft");
                ui.add_enabled_ui(has_run,|ui|{ui.selectable_value(&mut self.context,SceneContext::Retained,"Retained run");});
            });
            if previous!=self.context {self.selected=None;}
            let editable=self.context==SceneContext::Draft && !active_worker && !self.editing();
            if self.context==SceneContext::Draft && !self.forces.objects.is_empty() {
                ui.colored_label(egui::Color32::YELLOW,"Authored-force runs unavailable")
                    .on_hover_text("Runtime and archive integration is pending. No authored force will be silently ignored.");
            }
            if self.context==SceneContext::Draft && let Some(error)=&self.error {ui.colored_label(egui::Color32::LIGHT_RED,error);}
            egui::ScrollArea::vertical().id_salt("scene-properties-scroll").show(ui,|ui| {
                if let Some(id)=self.model_id() {
                    let response=ui.selectable_label(self.selected()==Some(id),"Collision model");
                    if response.clicked(){self.select(Some(id));}
                }
                ui.heading("Authored forces");
                if self.context==SceneContext::Draft {
                    ui.add_enabled_ui(editable,|ui| {ui.menu_button("Add force",|ui| {
                        for kind in [NewForce::Directional,NewForce::Radial,NewForce::Vortex] {
                            if ui.button(kind.label()).clicked() {if let Err(e)=self.add(kind){self.error=Some(e);}ui.close();}
                        }
                    });});
                    egui::ScrollArea::vertical().id_salt("force-object-list").max_height(160.0).show(ui,|ui| {
                        for object in &self.forces.objects {
                            let label=if object.enabled {object.name.clone()} else {format!("{} (disabled)",object.name)};
                            if ui.push_id(object.id.0, |ui| ui.selectable_label(self.selected==Some(object.id),label)).inner.clicked() {self.selected=Some(object.id);}
                        }
                    });
                    ui.small("Gravity remains additive.").on_hover_text("Acceleration fields add to configured gravity. Avoid applying Earth gravity twice. Force density divides once by local mass density, never particle mass.");
                } else {ui.label("Authored-force metadata unavailable").on_hover_text("This legacy retained run has no canonical authored-force document. Its physics is not inferred from the current draft.");}
                ui.separator();
                ui.heading("Properties");
                if self.editing() {ui.small("Drag preview · release applies · Esc cancels");}
                if self.selected.is_some() && self.selected==self.model_id() {
                    if let Some(model)=model {ui.label(format!("{} triangles",model.triangles));ui.label(format!("Bounds {:?} … {:?} m",model.bounds.0,model.bounds.1));ui.small("Accepted geometry · placement uses Import / apply mesh in setup.");}
                } else if self.context==SceneContext::Draft {
                    if self.edit_matches() {
                        let mut candidate=self.edit.as_ref().unwrap().preview.clone();
                        ui.add_enabled_ui(false,|ui|force_properties(ui,&mut candidate));
                    } else if let Some(object)=self.forces.objects.iter_mut().find(|o|Some(o.id)==self.selected) {
                        let changed=ui.push_id(("force-properties",object.id.0), |ui| ui.add_enabled_ui(editable,|ui|force_properties(ui,object)).inner).inner;
                        if changed {self.validate();}
                        if ui.add_enabled(editable,egui::Button::new("Remove force")).clicked(){self.remove_selected();}
                    } else {ui.label("Select an object");}
                } else {ui.label("Select an object");}
            });
        });
    }
}
fn quantity(ui: &mut egui::Ui, label: &str, value: &mut f64, unit: &str) -> bool {
    super::settings::quantity(ui, label, value, unit, 0.01, -f64::MAX..=f64::MAX).changed()
}
fn force_properties(ui: &mut egui::Ui, object: &mut ForceObject) -> bool {
    let mut changed = false;
    let name_label = ui
        .label("Name")
        .on_hover_text("At most 256 UTF-8 bytes; disabled objects are validated too.");
    changed |= ui
        .add(
            egui::TextEdit::singleline(&mut object.name)
                .desired_width(ui.available_width())
                .char_limit(256),
        )
        .labelled_by(name_label.id)
        .changed();
    changed |= ui.checkbox(&mut object.enabled, "Enabled").changed();
    ui.horizontal_wrapped(|ui| {
        let units_label = ui.label("Units");
        egui::ComboBox::from_id_salt("force-units")
            .selected_text(match object.units {
                ForceUnits::Acceleration => "m/s²",
                ForceUnits::ForceDensity => "N/m³",
            })
            .show_ui(ui, |ui| {
                changed |= ui
                    .selectable_value(
                        &mut object.units,
                        ForceUnits::Acceleration,
                        "Acceleration · m/s²",
                    )
                    .changed();
                changed |= ui
                    .selectable_value(
                        &mut object.units,
                        ForceUnits::ForceDensity,
                        "Force density · N/m³",
                    )
                    .changed();
            }).response.labelled_by(units_label.id).on_hover_text("Changing units reinterprets the authored numbers. No automatic density conversion is applied here.");
    });
    let unit = match object.units {
        ForceUnits::Acceleration => "m/s²",
        ForceUnits::ForceDensity => "N/m³",
    };
    match &mut object.kind {
        ForceKind::Directional { vector } => {
            for (d, label) in ["Vector X", "Vector Y", "Vector Z"].iter().enumerate() {
                changed |= quantity(ui, label, &mut vector[d], unit);
            }
        }
        ForceKind::Radial { strength } => {
            changed |= quantity(ui, "Strength", strength, unit);
            ui.small("Positive repels · negative attracts").on_hover_text("Constant magnitude before falloff; zero at the centre. Not an inverse-square law.");
        }
        ForceKind::Vortex { strength } => {
            changed |= quantity(ui, "Strength", strength, unit);
            ui.small("Around local +Y").on_hover_text("Positive strength follows the right-hand rule: at local +X the field points toward −Z. Zero on the axis.");
        }
    }
    ui.collapsing("Transform",|ui| {
        for(d,label)in["Position X","Position Y","Position Z"].iter().enumerate(){changed|=quantity(ui,label,&mut object.transform.translation[d],"m");}
        let mut angles=euler_degrees(object.transform.rotation);let mut rotate=false;
        for(d,label)in["Rotation X","Rotation Y","Rotation Z"].iter().enumerate(){rotate|=super::settings::quantity(ui,label,&mut angles[d],"°",1.0,-360.0..=360.0).changed();}
        if rotate {object.transform.rotation=quaternion(angles);changed=true;}
        if ui.button("Reset orientation").clicked() { object.transform.rotation=ForceTransform::default().rotation; changed=true; }
        ui.small("Local XYZ orientation").on_hover_text("Local-to-world rotation Rz·Ry·Rx, in degrees. Translation and support dimensions are metres; no scale or parenting.");
    });
    ui.collapsing("Support & activation",|ui| {
        let mut support=match object.support {ForceSupport::Global=>0,ForceSupport::Sphere{..}=>1,ForceSupport::Box{..}=>2};
        let before=support;
        egui::ComboBox::from_id_salt("force-support").selected_text(["Global","Sphere","Box"][support]).show_ui(ui,|ui|{for(i,label)in["Global","Sphere","Box"].iter().enumerate(){ui.selectable_value(&mut support,i,*label);}});
        if before!=support {object.support=match support {1=>ForceSupport::Sphere{radius:0.25,edge_fraction:0.25},2=>ForceSupport::Box{half_extent:[0.25;3],edge_fraction:0.25},_=>ForceSupport::Global};changed=true;}
        match &mut object.support {
            ForceSupport::Global=>{},
            ForceSupport::Sphere{radius,edge_fraction}=>{changed|=quantity(ui,"Radius",radius,"m");changed|=quantity(ui,"Edge fraction",edge_fraction,"");},
            ForceSupport::Box{half_extent,edge_fraction}=>{for(d,label)in["Half-size X","Half-size Y","Half-size Z"].iter().enumerate(){changed|=quantity(ui,label,&mut half_extent[d],"m");}changed|=quantity(ui,"Edge fraction",edge_fraction,"");}
        }
        changed|=quantity(ui,"Start",&mut object.time.start_seconds,"s");
        let mut finite_end=object.time.end_seconds.is_some();
        if ui.checkbox(&mut finite_end,"End time").changed(){object.time.end_seconds=finite_end.then_some(object.time.start_seconds+1.0);changed=true;}
        if let Some(end)=&mut object.time.end_seconds {changed|=quantity(ui,"End (exclusive)",end,"s");}
        ui.small("Sampled at accepted interval start").on_hover_text("Activation is [start,end). Steps are not split at endpoints. Sphere outer surface is excluded; box bounds are half-open. Falloff edge fraction is0–1; box falloff uses max-norm shells.");
    });
    changed
}
fn quaternion(degrees: [f64; 3]) -> [f64; 4] {
    let [x, y, z] = degrees.map(|v| v.to_radians() * 0.5);
    let (sx, cx) = x.sin_cos();
    let (sy, cy) = y.sin_cos();
    let (sz, cz) = z.sin_cos();
    [
        sx * cy * cz - cx * sy * sz,
        cx * sy * cz + sx * cy * sz,
        cx * cy * sz - sx * sy * cz,
        cx * cy * cz + sx * sy * sz,
    ]
}
fn euler_degrees([x, y, z, w]: [f64; 4]) -> [f64; 3] {
    [
        (2.0 * (w * x + y * z)).atan2(1.0 - 2.0 * (x * x + y * y)),
        (2.0 * (w * y - z * x)).clamp(-1.0, 1.0).asin(),
        (2.0 * (w * z + x * y)).atan2(1.0 - 2.0 * (y * y + z * z)),
    ]
    .map(f64::to_degrees)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn context() -> egui::Context {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        ctx.style_mut(|s| s.animation_time = 0.0);
        ctx
    }
    fn frame(
        ctx: &egui::Context,
        scene: &mut SceneUi,
        width: f32,
        events: Vec<egui::Event>,
    ) -> (egui::FullOutput, egui::Rect) {
        let mut available = egui::Rect::NOTHING;
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(width, 820.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                if width >= 1100.0 {
                    egui::SidePanel::left("inert-setup")
                        .exact_width(280.0)
                        .show(ctx, |_| {});
                }
                scene.show(ctx, true, false, None);
                available = ctx.available_rect();
            },
        );
        assert!(available.width() >= 300.0, "viewport width {available:?}");
        (output, available)
    }
    fn text_rect(output: &egui::FullOutput, label: &str, last: bool) -> egui::Rect {
        let matches: Vec<_> = output
            .shapes
            .iter()
            .filter_map(|s| match &s.shape {
                egui::Shape::Text(t) if t.galley.text() == label => {
                    Some(egui::Rect::from_min_size(t.pos, t.galley.size()).intersect(s.clip_rect))
                }
                _ => None,
            })
            .filter(|r| r.is_positive())
            .collect();
        *if last {
            matches.last()
        } else {
            matches.first()
        }
        .unwrap_or_else(|| panic!("missing visible {label}"))
    }
    fn click(
        ctx: &egui::Context,
        scene: &mut SceneUi,
        width: f32,
        output: &egui::FullOutput,
        label: &str,
        last: bool,
    ) {
        let pos = text_rect(output, label, last).center();
        for pressed in [true, false] {
            let _ = frame(
                ctx,
                scene,
                width,
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
    fn replace(ctx: &egui::Context, scene: &mut SceneUi, width: f32, text: &str) {
        let _ = frame(
            ctx,
            scene,
            width,
            key(
                egui::Key::A,
                egui::Modifiers {
                    ctrl: true,
                    command: true,
                    ..Default::default()
                },
            ),
        );
        let _ = frame(ctx, scene, width, vec![egui::Event::Text(text.into())]);
    }
    #[test]
    fn scene_identity_context_removal_and_exhaustion_preserve_canonical_values() {
        let mut scene = SceneUi::default();
        scene.replace_model(true).unwrap();
        let model = scene.model_id().unwrap();
        let a = scene.add(NewForce::Directional).unwrap();
        let b = scene.add(NewForce::Radial).unwrap();
        let c = scene.add(NewForce::Vortex).unwrap();
        assert_eq!([model.0, a.0, b.0, c.0], [1, 2, 3, 4]);
        assert_eq!(scene.selected(), Some(c));
        let frozen = scene.draft().freeze().unwrap();
        scene.forces.objects[0].name = "Changed draft".into();
        assert_ne!(
            scene.draft().objects[0].name,
            frozen.scene().objects[0].name
        );
        scene.select(Some(a));
        scene.remove_selected();
        assert_eq!(scene.selected(), None);
        assert!(scene.draft().objects.iter().all(|o| o.id != a));
        scene.begin_run(true);
        assert_eq!(scene.context, SceneContext::Retained);
        scene.select(Some(b));
        assert_eq!(scene.selected(), None);
        scene.select(Some(model));
        assert_eq!(scene.selected(), Some(model));
        scene.begin_run(true);
        assert_eq!(scene.selected(), None);
        scene.set_context(SceneContext::Draft);
        assert_eq!(scene.selected(), None);
        scene.next_id = Some(u64::MAX);
        let last = scene.add(NewForce::Directional).unwrap();
        assert_eq!(last.0, u64::MAX);
        let before = scene.forces.clone();
        let selected = scene.selected();
        assert!(scene.add(NewForce::Radial).is_err());
        assert_eq!(scene.forces, before);
        assert!(scene.replace_model(true).is_err());
        assert_eq!(scene.model_id(), Some(model));
        assert_eq!(scene.selected(), selected);
        scene.replace_model(false).unwrap();
        assert_eq!(scene.model_id(), None);
    }
    #[test]
    fn scene_pointer_add_select_edit_collapse_and_retained_context() {
        for width in [900.0, 1250.0] {
            let ctx = context();
            let mut scene = SceneUi {
                open: true,
                ..Default::default()
            };
            for kind in ["Directional", "Radial", "Vortex"] {
                let (output, _) = frame(&ctx, &mut scene, width, vec![]);
                click(&ctx, &mut scene, width, &output, "Add force", false);
                let (output, _) = frame(&ctx, &mut scene, width, vec![]);
                click(&ctx, &mut scene, width, &output, kind, false);
                assert_eq!(scene.selected(), scene.forces.objects.last().map(|o| o.id));
            }
            assert_eq!(scene.forces.objects.len(), 3);
            let selected = scene.selected();
            let name = scene.forces.objects[2].name.clone();
            let (output, _) = frame(&ctx, &mut scene, width, vec![]);
            click(&ctx, &mut scene, width, &output, &name, true);
            replace(&ctx, &mut scene, width, "Swirl");
            assert_eq!(scene.forces.objects[2].name, "Swirl");
            assert_eq!(scene.selected(), selected);
            let (output, _) = frame(&ctx, &mut scene, width, vec![]);
            click(&ctx, &mut scene, width, &output, "m/s²", false);
            let (output, _) = frame(&ctx, &mut scene, width, vec![]);
            click(
                &ctx,
                &mut scene,
                width,
                &output,
                "Force density · N/m³",
                false,
            );
            assert_eq!(scene.forces.objects[2].units, ForceUnits::ForceDensity);
            let (output, _) = frame(&ctx, &mut scene, width, vec![]);
            let caption = output
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    egui::Shape::Text(text)
                        if text
                            .galley
                            .text()
                            .strip_suffix(" N/m³")
                            .and_then(|v| v.parse::<f64>().ok())
                            == Some(0.0) =>
                    {
                        Some(text.galley.text().to_owned())
                    }
                    _ => None,
                })
                .expect("visible zero strength with force-density units");
            click(&ctx, &mut scene, width, &output, &caption, false);
            replace(&ctx, &mut scene, width, "-2.5");
            assert_eq!(
                scene.forces.objects[2].kind,
                ForceKind::Vortex { strength: -2.5 }
            );
            assert!(scene.forces.freeze().is_ok());
            let (output, _) = frame(&ctx, &mut scene, width, vec![]);
            click(&ctx, &mut scene, width, &output, "Enabled", false);
            assert!(!scene.forces.objects[2].enabled);
            assert!(scene.launch_readiness().is_err());
            let (output, _) = frame(&ctx, &mut scene, width, vec![]);
            click(&ctx, &mut scene, width, &output, "Hide scene", false);
            assert!(!scene.open);
            assert_eq!(scene.selected(), selected);
            let (output, _) = frame(&ctx, &mut scene, width, vec![]);
            click(&ctx, &mut scene, width, &output, "Scene", false);
            assert!(scene.open);
            assert_eq!(scene.selected(), selected);
            let before = scene.forces.clone();
            scene.begin_run(false);
            let (output, _) = frame(&ctx, &mut scene, width, vec![]);
            assert!(text_rect(&output, "Authored-force metadata unavailable", false).is_positive());
            assert_eq!(scene.selected(), None);
            assert_eq!(scene.forces, before);
            click(&ctx, &mut scene, width, &output, "Draft", false);
            assert_eq!(scene.context, SceneContext::Draft);
            assert_eq!(scene.forces, before);
        }
    }
    #[test]
    fn scene_invalid_support_and_rotation_are_visible_without_snapshot_mutation() {
        let ctx = context();
        let mut scene = SceneUi {
            open: true,
            ..Default::default()
        };
        scene.add(NewForce::Radial).unwrap();
        let snapshot = scene.draft().freeze().unwrap();
        scene.forces.objects[0].support = ForceSupport::Sphere {
            radius: 0.0,
            edge_fraction: 0.25,
        };
        scene.validate();
        assert!(scene.error.is_some());
        assert!(scene.draft().freeze().is_err());
        assert!(scene.launch_readiness().is_err());
        assert_eq!(snapshot.scene().objects[0].support, ForceSupport::Global);
        let (output, _) = frame(&ctx, &mut scene, 900.0, vec![]);
        assert!(output.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.text().contains("Invalid authored force"))));
        scene.forces.objects[0].support = ForceSupport::Sphere {
            radius: 0.4,
            edge_fraction: 0.25,
        };
        scene.validate();
        assert!(scene.error.is_none());
        for angles in [[0.0, 0.0, 0.0], [30.0, 45.0, -90.0], [180.0, -90.0, 360.0]] {
            scene.forces.objects[0].transform.rotation = quaternion(angles);
            assert!(scene.draft().freeze().is_ok());
        }
        // Revalidate the entire current containing document on release, not only
        // the earlier candidate. No solver or recording owner is involved.
        let mut s = SceneUi::default();
        let id = s.add(NewForce::Directional).unwrap();
        s.forces.objects[0].transform.translation[1] = -0.0;
        let base = s.forces.objects[0].clone();
        assert!(s.begin_edit());
        let mut candidate = base.clone();
        candidate.transform.translation[0] = 0.25;
        assert!(s.preview_edit(Some(candidate)));
        s.add(NewForce::Radial).unwrap();
        s.select(Some(id));
        s.forces.objects[1].support = ForceSupport::Sphere {
            radius: 0.0,
            edge_fraction: 0.0,
        };
        assert!(!s.finish_edit(true));
        assert_eq!(
            s.forces.objects[0].transform.translation.map(f64::to_bits),
            base.transform.translation.map(f64::to_bits)
        );
        s.forces.objects[1].support = ForceSupport::Global;
        assert!(s.begin_edit());
        s.forces.objects[0].transform.translation[1] = 0.0;
        assert!(
            !s.finish_edit(true),
            "signed-zero base conflict must not commit"
        );
    }
    #[test]
    fn scene_force_refusal_projects_disabled_run_without_dispatch() {
        // Paint/capture intent only: even a broken guard cannot start work here.
        fn project(
            ctx: &egui::Context,
            app: &mut super::super::App,
            events: Vec<egui::Event>,
        ) -> (egui::FullOutput, bool) {
            let mut requested = false;
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(900.0, 650.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        requested = app.run_controls(ui).is_some();
                    });
                },
            );
            (output, requested)
        }
        let ctx = context();
        let mut app = super::super::App::default();
        app.scene.add(NewForce::Directional).unwrap();
        let (output, requested) = project(&ctx, &mut app, vec![]);
        assert!(!requested);
        assert!(output.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.text()=="Authored-force runs unavailable")));
        let pos = text_rect(&output, "Simulate", false).center();
        for pressed in [true, false] {
            let (_, requested) = project(
                &ctx,
                &mut app,
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
            assert!(!requested);
        }
        assert!(app.worker.is_none() && app.history.is_none() && app.running_request.is_none());
        app.scene.forces.objects[0].enabled = false;
        assert!(app.scene.launch_readiness().is_err());
    }
    #[test]
    fn scene_draft_timeline_roundtrip_preserves_retained_index_without_history() {
        let ctx = context();
        let mut app = super::super::App::default();
        app.timeline_step = 7;
        app.track = 1;
        app.follow_live = false;
        app.play_time = 1.25;
        let before = (app.timeline_step, app.track, app.play_time.to_bits());
        let selected = super::super::PreviewFrame {
            method: rheon::PressureImplementation::JacobiPcgV1,
            step: 7,
            time: 1.25,
            seconds: 0.002,
            edge: 1,
            nonzero_tracer_cells: 0,
            density: vec![],
            report: None,
            particle_report: None,
            material_color: None,
        };
        for context in [
            SceneContext::Draft,
            SceneContext::Retained,
            SceneContext::Draft,
            SceneContext::Retained,
        ] {
            app.scene.set_context(context);
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(900.0, 650.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    let painted = app.scene_timeline_ui(
                        ctx,
                        if context == SceneContext::Retained {
                            10
                        } else {
                            0
                        },
                        if context == SceneContext::Retained {
                            Some(&selected)
                        } else {
                            None
                        },
                    );
                    assert_eq!(painted.is_some(), context == SceneContext::Retained);
                },
            );
            assert_eq!(
                (app.timeline_step, app.track, app.play_time.to_bits()),
                before
            );
        }
        assert!(app.worker.is_none() && app.history.is_none() && app.running_request.is_none());
    }
}
