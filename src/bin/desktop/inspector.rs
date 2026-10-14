//! Exact selected disk publication. The worker owns recording; UI owns selection.
use super::export_ui::ExportFile;
use eframe::egui;
use rheon::recording::{
    RecordedMaterial, RecordingLimits, RecordingReader, StepRecorder, material_publication,
    material_run,
};
use rheon::{
    FluidParticle, MaterialStepControls, ParticleFluidSimulation, ParticleStepReport,
    SessionIdentity,
};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

struct State {
    reader: Option<RecordingReader>,
    path: Option<PathBuf>,
    limits: RecordingLimits,
    selected_step: Option<usize>,
    material: Option<Arc<RecordedMaterial>>,
    selected: usize,
    feedback: String,
    export: ExportFile,
}
impl Default for State {
    fn default() -> Self {
        Self {
            reader: None,
            path: None,
            limits: RecordingLimits::default(),
            selected_step: None,
            material: None,
            selected: 0,
            feedback: "No numerical recording. Carrier previews do not retain numerical fields."
                .into(),
            export: ExportFile::new(".generated/selected-particles.vtp"),
        }
    }
}
#[derive(Default)]
pub struct Inspector(Mutex<State>);
impl Inspector {
    pub fn with_particles<R>(
        &self,
        read: impl FnOnce(&SessionIdentity, u64, f64, &[FluidParticle], usize) -> R,
    ) -> Option<R> {
        let (identity, m, selected) = {
            let s = self.0.lock().ok()?;
            (
                s.reader.as_ref()?.run().identity.clone(),
                Arc::clone(s.material.as_ref()?),
                s.selected,
            )
        };
        // A callback may re-enter selection. Retain this immutable publication
        // through the callback, without retaining the inspector's mutex.
        Some(read(
            &identity,
            m.generation,
            m.time,
            &m.particles,
            selected,
        ))
    }
    pub fn select(&self, id: usize) {
        if let Ok(mut s) = self.0.lock()
            && s.material.as_ref().is_some_and(|m| id < m.particles.len())
        {
            s.selected = id;
        }
    }
    pub fn begin(
        &self,
        sim: &ParticleFluidSimulation,
        path: &Path,
        limits: RecordingLimits,
    ) -> Result<StepRecorder, String> {
        let run=material_run(super::identity::new_identity("material"),sim,
            "native desktop; caller-owned local path; source revision/build/toolchain not supplied; no restart guarantee")
            .map_err(|e|e.to_string())?;
        let mut recorder = StepRecorder::create(path, run, limits).map_err(|e| e.to_string())?;
        recorder
            .append(
                &material_publication(sim, None, None, 0, limits.max_record_bytes)
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
        self.reload(path, limits)?;
        self.select_timestep(0)?;
        Ok(recorder)
    }
    /// Cold reload is also used by tests/API-host integrations. Keeps no live sim.
    pub fn reload(&self, path: &Path, limits: RecordingLimits) -> Result<(), String> {
        let mut s = self.0.lock().map_err(|_| "Inspector owner failed")?;
        // Invalidate selection before I/O, including failed reloads.
        s.material = None;
        s.selected_step = None;
        s.reader = None;
        match RecordingReader::open(path, limits) {
            Ok(reader) => {
                s.reader = Some(reader);
                s.path = Some(path.to_owned());
                s.limits = limits;
                s.feedback.clear();
                Ok(())
            }
            Err(e) => {
                s.feedback = e.to_string();
                Err(s.feedback.clone())
            }
        }
    }
    pub fn record(
        &self,
        recorder: &mut StepRecorder,
        sim: &ParticleFluidSimulation,
        report: ParticleStepReport,
        controls: MaterialStepControls,
    ) -> Result<(), String> {
        let actual = material_run(
            recorder.run().identity.clone(),
            sim,
            &recorder.run().provenance,
        )
        .map_err(|e| e.to_string())?;
        if &actual != recorder.run() {
            return Err(
                "Recording configuration/geometry changed; accepted step is unrecorded".into(),
            );
        }
        let step = material_publication(
            sim,
            Some(report),
            Some(controls),
            recorder.recorded_count(),
            recorder.limits().max_record_bytes,
        )
        .map_err(|e| e.to_string())?;
        recorder.append(&step).map_err(|e| {
            format!(
                "Accepted generation {} is not confirmed recorded: {e}",
                sim.generation()
            )
        })
    }
    /// Missing/corrupt selected steps clear values. Never substitute a latest step.
    pub fn select_timestep(&self, step: usize) -> Result<(), String> {
        let mut s = self.0.lock().map_err(|_| "Inspector owner failed")?;
        if s.selected_step == Some(step) && s.material.is_some() {
            return Ok(());
        }
        s.selected_step = Some(step);
        s.material = None;
        let result = s
            .reader
            .as_ref()
            .ok_or_else(|| s.feedback.clone())
            .and_then(|r| r.select(step).map_err(|e| e.to_string()))
            .and_then(|record| RecordedMaterial::decode(&record).map_err(|e| e.to_string()));
        match result {
            Ok(m) => {
                s.material = Some(Arc::new(m));
                s.feedback.clear();
                Ok(())
            }
            Err(e) => {
                s.feedback = e.clone();
                Err(e)
            }
        }
    }
    pub fn ui(&self, ui: &mut egui::Ui) {
        let Ok(mut s) = self.0.lock() else {
            ui.label("Inspector unavailable");
            return;
        };
        ui.label("Numerical state at selected recorded timestep");
        if let Some(path) = &s.path {
            ui.small(format!("Archive: {}", path.display()));
        }
        if let Some(reader) = &s.reader {
            ui.small(format!(
                "Run {} · source {}",
                reader.run().identity.run_id,
                reader.run().identity.source_revision
            ));
            match reader.count() {
                Ok(n) => {
                    ui.small(format!(
                        "{n} recorded publications; keep all / stop at limits"
                    ));
                }
                Err(e) => {
                    ui.label(e.to_string());
                }
            }
            match reader.status() {
                Ok(status) => {
                    ui.small(format!("{}: {}", status.outcome, status.detail));
                    if status.outcome == "active" {
                        ui.small("Incomplete until producer declares completion. A crash may leave an unrecorded accepted tail.");
                    }
                }
                Err(e) => {
                    ui.label(format!("Recording status unavailable: {e}"));
                }
            }
        }
        if !s.feedback.is_empty() {
            ui.label(&s.feedback);
        }
        let State {
            material,
            selected,
            export,
            reader,
            ..
        } = &mut *s;
        let (Some(m), Some(reader)) = (material, reader) else {
            ui.label("Selected numerical fields unavailable");
            return;
        };
        ui.small("Particle IDs are append-only indices scoped to this recorded run.");
        ui.monospace(format!(
            "generation {}\npublication {} s\naccepted dt {:?} s",
            m.generation, m.time, m.dt
        ));
        ui.collapsing("Totals at selected publication",|ui| {
            let mass:f64=m.particles.iter().map(|p|p.mass).sum();
            let momentum:[f64;3]=std::array::from_fn(|d|m.particles.iter().map(|p|p.mass*p.velocity[d]).sum());
            let kinetic:f64=m.particles.iter().map(|p|0.5*p.mass*p.velocity.iter().map(|v|v*v).sum::<f64>()).sum();
            ui.monospace(format!("mass {mass} kg\nmomentum {momentum:?} kg m/s\nkinetic energy {kinetic} J"));
            ui.small("Derived raw totals; emission, gravity, viscosity and walls exchange mass/momentum/energy. No conservation-residual claim.");
        });
        ui.horizontal(|ui| {
            ui.label("Particle ID");
            ui.add(
                egui::DragValue::new(selected)
                    .range(0..=m.particles.len().saturating_sub(1))
                    .clamp_existing_to_range(false),
            );
        });
        if let Some(p) = m.particles.get(*selected) {
            ui.monospace(format!(
                "position {:?} m\nvelocity {:?} m/s\nmass {} kg\ntemperature {} K\nmaterial {}",
                p.position, p.velocity, p.mass, p.temperature_kelvin, p.material
            ));
            if let Some((t, d)) = &m.diagnostics {
                let d = d[*selected];
                ui.small("Post-emission, pre-integration; excludes wall response. Evaluation positions are not retained.");
                ui.monospace(format!("evaluated {t} s\ndensity {} kg/m³\npressure {} Pa\nacceleration {:?} m/s²\nderived force {:?} N",d.density,d.pressure,d.acceleration,d.acceleration.map(|a|a*p.mass)));
            } else {
                ui.label("Diagnostics not recorded for this publication; no substituted values");
            }
            if ui.button("Copy selected particle CSV").clicked() {
                ui.ctx()
                    .copy_text(csv(m, *selected, reader.run().identity.run_id));
            }
        } else {
            ui.label("No particle at this ID in the selected step");
        }
        ui.collapsing("Selected timestep VTK export", |ui| {
            export.show(ui, "Save selected timestep particles (.vtp)", |out| {
                m.write_vtp(out, &reader.run().identity)
            });
        });
    }
}
fn csv(m: &RecordedMaterial, id: usize, run_id: u64) -> String {
    use std::fmt::Write;
    let mut text = String::from(
        "run_id,particle_id,generation,publication_time_s,evaluation_time_s,x_m,y_m,z_m,vx_m_s,vy_m_s,vz_m_s,mass_kg,temperature_K,material,density_kg_m3,pressure_Pa,ax_m_s2,ay_m_s2,az_m_s2\n",
    );
    if let Some(p) = m.particles.get(id) {
        let t = m
            .diagnostics
            .as_ref()
            .map(|(t, _)| t.to_string())
            .unwrap_or_default();
        write!(
            text,
            "{run_id},{id},{},{},{t},{},{},{},{},{},{},{},{},{}",
            m.generation,
            m.time,
            p.position[0],
            p.position[1],
            p.position[2],
            p.velocity[0],
            p.velocity[1],
            p.velocity[2],
            p.mass,
            p.temperature_kelvin,
            p.material
        )
        .unwrap();
        if let Some((_, d)) = &m.diagnostics {
            let d = d[id];
            writeln!(
                text,
                ",{},{},{},{},{}",
                d.density, d.pressure, d.acceleration[0], d.acceleration[1], d.acceleration[2]
            )
            .unwrap();
        } else {
            text.push_str(",,,,,\n");
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    pub fn test_path(label: &str) -> PathBuf {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        std::env::temp_dir().join(format!(
            "rheon-inspector-{label}-{}-{}.sqlite",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ))
    }
    #[test]
    fn historical_selection_survives_capture_reload_and_egui_export_uses_that_step() {
        let grid = rheon::GridGeometry::new([4; 3], [0.25; 3], [0.0; 3]).unwrap();
        let mut sim = ParticleFluidSimulation::new(
            grid,
            rheon::ParticleFluidConfig {
                fill: rheon::DomainFill::EmittedFluid,
                samples_per_axis: 4,
                max_particles: 64,
                ..Default::default()
            },
        )
        .unwrap();
        let inspector = Inspector::default();
        let path = test_path("history");
        let limits = RecordingLimits::default();
        let mut writer = inspector.begin(&sim, &path, limits).unwrap();
        let initial = sim.particles().to_vec();
        let controls = MaterialStepControls {
            requested_dt: 1e-5,
            emitter: None,
        };
        for _ in 0..3 {
            let r = sim.step(controls.requested_dt, None, || false).unwrap();
            inspector.record(&mut writer, &sim, r, controls).unwrap();
        }
        assert_eq!(
            inspector.with_particles(|_, g, _, p, _| {
                assert_eq!(p, initial);
                g
            }),
            Some(0)
        );
        inspector.select_timestep(1).unwrap();
        let selected = inspector
            .with_particles(|_, _, _, p, _| p.to_vec())
            .unwrap();
        assert_ne!(selected, sim.particles());
        writer
            .finish(
                "completed",
                "all accepted steps retained",
                Some(sim.generation()),
            )
            .unwrap();
        drop(writer);
        drop(sim);
        inspector.reload(&path, limits).unwrap();
        inspector.select_timestep(1).unwrap();
        assert_eq!(
            inspector.with_particles(|_, _, _, p, _| p.to_vec()),
            Some(selected)
        );
        let s = inspector.0.lock().unwrap();
        let m = s.material.as_ref().unwrap();
        let mut vtk = Vec::new();
        m.write_vtp(&mut vtk, &s.reader.as_ref().unwrap().run().identity)
            .unwrap();
        let expected_vtk = vtk.clone();
        assert!(String::from_utf8(vtk).unwrap().contains(
            "Name=\"generation\" NumberOfComponents=\"1\" NumberOfTuples=\"1\" format=\"ascii\">1 "
        ));
        let row = csv(m, 0, s.reader.as_ref().unwrap().run().identity.run_id);
        let values: Vec<_> = row.lines().nth(1).unwrap().split(',').collect();
        assert_eq!(values.len(), 19);
        assert_eq!(values[2], "1");
        drop(s);
        let ctx = egui::Context::default();
        let vtk_path = path.with_extension("vtp");
        inspector.0.lock().unwrap().export = ExportFile::new(vtk_path.to_str().unwrap());
        let render = |events| {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1000.0, 1400.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| inspector.ui(ui));
                },
            )
        };
        let click = |label: &str| {
            render(vec![]);
            let output = render(vec![]);
            let pos = output
                .shapes
                .iter()
                .find_map(|s| match &s.shape {
                    egui::Shape::Text(t) if t.galley.job.text == label => {
                        Some(t.pos + t.galley.size() * 0.5)
                    }
                    _ => None,
                })
                .unwrap_or_else(|| panic!("missing UI control {label}"));
            render(vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::default(),
                },
            ]);
            render(vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::default(),
            }])
        };
        let output = click("Copy selected particle CSV");
        assert!(output.platform_output.commands.iter().any(|c|matches!(c,egui::OutputCommand::CopyText(text) if text.lines().nth(1).unwrap().split(',').nth(2)==Some("1"))));
        click("Selected timestep VTK export");
        click("Save selected timestep particles (.vtp)");
        assert_eq!(std::fs::read(&vtk_path).unwrap(), expected_vtk);
        std::fs::remove_file(vtk_path).unwrap();
        inspector.0.lock().unwrap().selected = 100;
        render(vec![]);
        assert_eq!(inspector.0.lock().unwrap().selected, 100);
        assert!(
            inspector
                .with_particles(|_, _, _, p, id| p.get(id).copied())
                .unwrap()
                .is_none()
        );
        inspector.select(0);
        // Re-entrant particle selection cannot deadlock a retained publication.
        assert_eq!(
            inspector.with_particles(|_, g, _, _, _| {
                inspector.select(1);
                g
            }),
            Some(1)
        );
        assert!(inspector.select_timestep(4).is_err());
        assert!(inspector.with_particles(|_, _, _, _, _| ()).is_none());
        drop(inspector);
        std::fs::remove_file(path).unwrap();
    }
}
