//! Cold inspection uses archive identity/geometry; no live scene reconstruction.
use super::inspector::Inspector;
use eframe::egui;
use rheon::recording::RecordingLimits;
use std::path::Path;
#[derive(Default)]
pub struct ArchiveView {
    pub active: bool,
    pub path: String,
    pub inspector: Inspector,
    selected: usize,
    feedback: String,
}
impl ArchiveView {
    pub fn open(&mut self, limits: RecordingLimits) -> Result<(), String> {
        self.active = true;
        self.selected = 0;
        let result = self
            .inspector
            .reload(Path::new(&self.path), limits)
            .and_then(|()| {
                let count = self.inspector.recorded_count()?;
                if count == 0 {
                    return Err("No committed numerical publications".into());
                }
                self.inspector.select_timestep(0)
            });
        self.feedback = result.as_ref().err().cloned().unwrap_or_default();
        result
    }
    pub fn controls(&mut self, ui: &mut egui::Ui, enabled: bool, limits: RecordingLimits) {
        ui.collapsing("Open numerical archive",|ui| {
            ui.small("Local protected SQLite path; raw field inspection and selected exports. Opening performs synchronous quick-check; large files may take time.");
            ui.text_edit_singleline(&mut self.path);
            if ui.add_enabled(enabled,egui::Button::new("Open recorded run")).clicked() {let _=self.open(limits);}
            if !enabled {ui.small("Finish or cancel the active worker before opening an archive.");}
        });
    }
    pub fn show(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("archive-navigation").show(ctx,|ui| {
            ui.horizontal(|ui| {ui.heading("Recorded numerical run");if ui.button("Back to simulation").clicked() {self.active=false;}});
            ui.small("Archive-only inspection. Collision mesh, emitter and scene reconstruction are not supplied by this file.");
            let count=self.inspector.recorded_count().unwrap_or(0);
            if count>0 {
                let mut changed=false;
                ui.horizontal(|ui| {
                    if ui.add_enabled(self.selected>0,egui::Button::new("Previous recorded step")).clicked() {self.selected-=1;changed=true;}
                    if ui.add_enabled(self.selected+1<count,egui::Button::new("Next recorded step")).clicked() {self.selected+=1;changed=true;}
                });
                if (ui.add(egui::Slider::new(&mut self.selected,0..=count-1).text("Recorded ordinal")).changed() || changed)
                    && let Err(e)=self.inspector.select_timestep(self.selected) {self.feedback=e;}

            }
            if ui.button("Audit all committed publications").clicked() {
                self.feedback=match self.inspector.verify() {Ok(())=>"Full committed-prefix integrity audit passed; lifecycle still determines completeness".into(),Err(e)=>e};
            }
            ui.small("Audit is synchronous, reads every step, and may block a concurrent writer; pause the producer first.");
            if !self.feedback.is_empty() {ui.label(&self.feedback);}
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| self.inspector.ui(ui));
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rheon::*;
    #[test]
    fn cold_open_real_egui_navigation_and_invalid_reload_clear_selected_identity() {
        let path =
            std::env::temp_dir().join(format!("rheon-cold-ui-{}.sqlite", std::process::id()));
        let mut sim = Simulation::new(
            GridGeometry::new([3, 4, 5], [0.2, 0.25, 0.3], [-2.0, 3.0, 5.0]).unwrap(),
            SimulationConfig {
                density: 1.0,
                memory_limit: 64 * 1024 * 1024,
                pressure: PressureSettings {
                    relative_residual: 1e-10,
                    absolute_residual: 1e-12,
                    divergence_limit: 1e-8,
                    max_iterations: 2000,
                },
                actual_divergence_limit: 1e-5,
                max_courant: 1.0,
            },
        )
        .unwrap();
        let live = Inspector::default();
        let limits = RecordingLimits::default();
        let mut writer = live.begin_carrier(&sim, &path, limits).unwrap();
        let source = SmokeSource {
            lower: [-1.9, 3.1, 5.1],
            upper: [-1.6, 3.7, 6.1],
            tracer_rate: 2.0,
            vertical_acceleration: 1.0,
        };
        for _ in 0..2 {
            sim.step(0.003, Some(source), |_| false).unwrap();
            live.record_carrier(&mut writer, &sim).unwrap();
        }
        writer
            .finish("completed", "cold UI fixture", Some(2))
            .unwrap();
        live.select_timestep(1).unwrap();
        let expected = live.selected_identity().unwrap();
        drop(writer);
        drop(sim);
        drop(live);
        let ctx = egui::Context::default();
        let mut cold = ArchiveView {
            path: path.to_str().unwrap().into(),
            ..Default::default()
        };
        let render = |cold: &mut ArchiveView, events| {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1400.0, 1500.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    if cold.active {
                        cold.show(ctx);
                    } else {
                        egui::CentralPanel::default()
                            .show(ctx, |ui| cold.controls(ui, true, limits));
                    }
                },
            )
        };
        let click = |cold: &mut ArchiveView, label: &str| {
            let _ = render(cold, vec![]);
            let output = render(cold, vec![]);
            let pos = output
                .shapes
                .iter()
                .find_map(|s| match &s.shape {
                    egui::Shape::Text(t) if t.galley.job.text == label => {
                        Some(t.pos + t.galley.size() * 0.5)
                    }
                    _ => None,
                })
                .unwrap_or_else(|| panic!("missing cold control {label}"));
            let _ = render(
                cold,
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed: true,
                        modifiers: Default::default(),
                    },
                ],
            );
            render(
                cold,
                vec![egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: Default::default(),
                }],
            )
        };
        let _ = click(&mut cold, "Open numerical archive");
        let _ = click(&mut cold, "Open recorded run");
        assert!(cold.active);
        assert_eq!(cold.inspector.selected_identity().unwrap().1, 0);
        let _ = click(&mut cold, "Next recorded step");
        assert_eq!(cold.inspector.selected_identity(), Some(expected));
        let output = click(&mut cold, "Copy selected field sample CSV");
        assert!(output.platform_output.commands.iter().any(|c|matches!(c,egui::OutputCommand::CopyText(text) if text.lines().nth(1).unwrap().split(',').nth(2)==Some("1"))));
        let _ = click(&mut cold, "Audit all committed publications");
        assert!(cold.feedback.contains("passed"));
        let db = rusqlite::Connection::open(&path).unwrap();
        db.execute(
            "UPDATE fields SET units='s/s' WHERE ordinal=1 AND name='velocity_x'",
            [],
        )
        .unwrap();
        drop(db);
        assert!(cold.inspector.verify().is_err());
        assert!(cold.inspector.selected_identity().is_none());
        assert!(cold.inspector.select_timestep(1).is_err());
        assert!(cold.inspector.selected_identity().is_none());
        assert!(cold.inspector.select_timestep(99).is_err());
        assert!(cold.inspector.selected_identity().is_none());
        cold.path = path.with_extension("missing").to_str().unwrap().into();
        assert!(cold.open(limits).is_err());
        assert!(cold.active);
        assert!(cold.inspector.selected_identity().is_none());
        cold.path = path.to_str().unwrap().into();
        let db = rusqlite::Connection::open(&path).unwrap();
        db.execute(
            "UPDATE fields SET units='s/s' WHERE ordinal=0 AND name='velocity_x'",
            [],
        )
        .unwrap();
        drop(db);
        assert!(cold.open(limits).is_err());
        assert!(cold.inspector.selected_identity().is_none());
        assert!(cold.feedback.contains("publication digest"));
        drop(cold);
        std::fs::remove_file(path).unwrap();
    }
}
