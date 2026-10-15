//! Exact selected disk publication. The worker owns recording; UI owns selection.
use super::export_ui::ExportFile;
use eframe::egui;
use rheon::recording::{
    RecordedCarrier, RecordedMaterial, RecordedStep, RecordingLimits, RecordingReader,
    StepRecorder, carrier_publication, carrier_run, material_publication, material_run,
};
use rheon::{
    FluidParticle, MaterialStepControls, ParticleFluidSimulation, ParticleStepReport,
    SessionIdentity, Simulation,
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
    carrier: Option<Arc<RecordedCarrier>>,
    record: Option<Arc<RecordedStep>>,
    field_name: String,
    scalar_index: usize,
    selected: usize,
    feedback: String,
    export: ExportFile,
    carrier_export: ExportFile,
}
impl Default for State {
    fn default() -> Self {
        Self {
            reader: None,
            path: None,
            limits: RecordingLimits::default(),
            selected_step: None,
            material: None,
            carrier: None,
            record: None,
            field_name: String::new(),
            scalar_index: 0,
            selected: 0,
            feedback: "No numerical recording selected".into(),
            export: ExportFile::new(".generated/selected-particles.vtp"),
            carrier_export: ExportFile::new(".generated/selected-carrier.vti"),
        }
    }
}
#[derive(Default)]
pub struct Inspector(Mutex<State>);
impl Inspector {
    #[cfg(test)]
    pub fn selected_identity(&self) -> Option<(u64, u64, f64)> {
        let s = self.0.lock().ok()?;
        let record = s.record.as_ref()?;
        Some((
            s.reader.as_ref()?.run().identity.run_id,
            record.generation,
            record.time,
        ))
    }
    #[cfg(test)]
    pub fn archive_path(&self) -> Option<PathBuf> {
        self.0.lock().ok()?.path.clone()
    }
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
    pub fn begin_carrier(
        &self,
        sim: &Simulation,
        path: &Path,
        limits: RecordingLimits,
    ) -> Result<StepRecorder, String> {
        let run=carrier_run(super::identity::new_identity("carrier"),sim,
            "native desktop; source revision/build/toolchain not supplied; fixed sealed box; no restart guarantee").map_err(|e|e.to_string())?;
        self.begin_bound_carrier(sim, path, limits, run)
    }
    pub fn begin_bound_carrier(
        &self,
        sim: &Simulation,
        path: &Path,
        limits: RecordingLimits,
        run: rheon::recording::RecordingRun,
    ) -> Result<StepRecorder, String> {
        let mut recorder = StepRecorder::create(path, run, limits).map_err(|e| e.to_string())?;
        recorder
            .append(
                &carrier_publication(sim, 0, limits.max_record_bytes).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
        self.reload(path, limits)?;
        self.select_timestep(0)?;
        Ok(recorder)
    }
    pub fn record_carrier(
        &self,
        recorder: &mut StepRecorder,
        sim: &Simulation,
    ) -> Result<(), String> {
        let actual = carrier_run(
            recorder.run().identity.clone(),
            sim,
            &recorder.run().provenance,
        )
        .map_err(|e| e.to_string())?;
        if &actual != recorder.run() {
            return Err("Carrier configuration/geometry changed; accepted step unrecorded".into());
        }
        let step = carrier_publication(
            sim,
            recorder.recorded_count(),
            recorder.limits().max_record_bytes,
        )
        .map_err(|e| e.to_string())?;
        recorder.append(&step).map_err(|e| {
            format!(
                "Accepted carrier generation {} not confirmed recorded: {e}",
                sim.state().generation
            )
        })
    }
    pub fn recorded_count(&self) -> Result<usize, String> {
        let mut s = self.0.lock().map_err(|_| "Inspector owner failed")?;
        let result = s
            .reader
            .as_ref()
            .ok_or("No archive selected")?
            .count()
            .map_err(|e| e.to_string());
        if let Err(e) = &result {
            s.record = None;
            s.material = None;
            s.carrier = None;
            s.feedback = e.clone();
        }
        result
    }
    pub fn verify(&self) -> Result<(), String> {
        let mut s = self.0.lock().map_err(|_| "Inspector owner failed")?;
        let result = s
            .reader
            .as_ref()
            .ok_or("No archive selected")?
            .verify()
            .map_err(|e| e.to_string());
        if let Err(e) = &result {
            s.record = None;
            s.material = None;
            s.carrier = None;
            s.feedback = e.clone();
        }
        result
    }
    /// Cold reload is also used by tests/API-host integrations. Keeps no live sim.
    pub fn reload(&self, path: &Path, limits: RecordingLimits) -> Result<(), String> {
        let mut s = self.0.lock().map_err(|_| "Inspector owner failed")?;
        // Invalidate selection before I/O, including failed reloads.
        s.material = None;
        s.carrier = None;
        s.record = None;
        s.path = None;
        s.field_name.clear();
        s.scalar_index = 0;
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
        s.selected_step = Some(step);
        s.material = None;
        s.carrier = None;
        s.record = None;
        let result = s
            .reader
            .as_ref()
            .ok_or_else(|| s.feedback.clone())
            .and_then(|r| r.select(step).map_err(|e| e.to_string()))
            .and_then(|record| {
                let run = s.reader.as_ref().ok_or("No archive selected")?.run();
                let material = if run.solver.starts_with("rheon-sph-preview/") {
                    Some(Arc::new(
                        RecordedMaterial::decode(&record).map_err(|e| e.to_string())?,
                    ))
                } else {
                    None
                };
                let carrier = if run.solver.starts_with("rheon-carrier-full-box-v1/") {
                    Some(Arc::new(
                        RecordedCarrier::decode(run, &record).map_err(|e| e.to_string())?,
                    ))
                } else {
                    None
                };
                Ok((Arc::new(record), material, carrier))
            });
        match result {
            Ok((record, material, carrier)) => {
                s.material = material;
                s.carrier = carrier;
                if s.field_name.is_empty() {
                    s.field_name = record
                        .fields
                        .first()
                        .map_or_else(String::new, |f| f.name.clone());
                }
                s.record = Some(record);
                s.feedback.clear();
                Ok(())
            }
            Err(e) => {
                s.feedback = e.clone();
                Err(e)
            }
        }
    }
    /// Repaint-only reuse of an immutable verified selection. Explicit selection,
    /// reload and audit perform I/O; any observed error clears cached values.
    pub fn ensure_timestep(&self, step: usize) -> Result<(), String> {
        {
            let s = self.0.lock().map_err(|_| "Inspector owner failed")?;
            if s.selected_step == Some(step) && s.record.is_some() {
                return Ok(());
            }
        }
        self.select_timestep(step)
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
        let mut metadata_error = None;
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
                    metadata_error = Some(e.to_string());
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
                    metadata_error = Some(e.to_string());
                    ui.label(format!("Recording status unavailable: {e}"));
                }
            }
        }
        if let Some(error) = metadata_error {
            s.record = None;
            s.material = None;
            s.carrier = None;
            s.feedback = error;
        }
        if !s.feedback.is_empty() {
            ui.label(&s.feedback);
        }
        raw_fields_ui(&mut s, ui);
        if let (Some(carrier), Some(reader)) = (&s.carrier, &s.reader) {
            let carrier = Arc::clone(carrier);
            let identity = reader.run().identity.clone();
            s.carrier_export
                .show(ui, "Save selected timestep carrier (.vti)", |out| {
                    carrier.write_vti(out, &identity)
                });
        }
        let State {
            material,
            selected,
            export,
            reader,
            ..
        } = &mut *s;
        let (Some(m), Some(reader)) = (material, reader) else {
            if s.record.is_none() {
                ui.label("Selected numerical fields unavailable");
            }
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
fn sample_value(field: &rheon::recording::RecordedField, index: usize) -> Option<String> {
    let width = match field.scalar_type.as_str() {
        "f64" | "u64" => 8,
        "f32" => 4,
        "u8" => 1,
        _ => return None,
    };
    let offset = index.checked_mul(width)?;
    let bytes = field.data.get(offset..offset.checked_add(width)?)?;
    Some(match field.scalar_type.as_str() {
        "f64" => f64::from_le_bytes(bytes.try_into().ok()?).to_string(),
        "f32" => f32::from_le_bytes(bytes.try_into().ok()?).to_string(),
        "u64" => u64::from_le_bytes(bytes.try_into().ok()?).to_string(),
        "u8" => bytes[0].to_string(),
        _ => return None,
    })
}
fn field_csv(
    run_id: u64,
    step: &RecordedStep,
    field_index: usize,
    scalar_index: usize,
) -> Option<String> {
    let field = step.fields.get(field_index)?;
    let value = sample_value(field, scalar_index)?;
    let quote = |text: &str| format!("\"{}\"", text.replace('"', "\"\""));
    let row = [
        run_id.to_string(),
        step.ordinal.to_string(),
        step.generation.to_string(),
        step.time.to_string(),
        field
            .evaluation_time
            .map_or_else(String::new, |t| t.to_string()),
        quote(&field.name),
        scalar_index.to_string(),
        value,
        quote(&field.units),
        quote(&field.association),
        quote(&field.stage),
        quote(&field.scalar_type),
        field.dimensions[0].to_string(),
        field.dimensions[1].to_string(),
        field.dimensions[2].to_string(),
        field.components.to_string(),
    ]
    .join(",");
    Some(format!(
        "run_id,ordinal,generation,publication_time_s,evaluation_time_s,field,scalar_index,value,units,association,stage,scalar_type,nx,ny,nz,components\n{row}\n"
    ))
}
fn raw_fields_ui(s: &mut State, ui: &mut egui::Ui) {
    let Some(reader) = &s.reader else {
        return;
    };
    ui.collapsing("Recorded model, geometry and provenance",|ui| {
        let run=reader.run();ui.label(&run.solver);ui.monospace(format!("counts {:?}\nspacing {:?} m\norigin {:?} m",run.counts,run.spacing,run.origin));
        ui.label(&run.configuration);ui.label(&run.provenance);
        ui.small("Opaque configuration/provenance; absent fields stay absent. No restart or universal-field completeness claim.");
    });
    let Some(step) = &s.record else {
        return;
    };
    let mut body = |ui: &mut egui::Ui| {
        ui.monospace(format!(
            "ordinal {} · generation {}\npublication {} s · dt {:?} s",
            step.ordinal, step.generation, step.time, step.dt
        ));
        ui.small(format!(
            "{} fields actually recorded; field order/index scopes to this publication",
            step.fields.len()
        ));
        egui::ComboBox::from_id_salt("recorded-field")
            .selected_text(
                step.fields
                    .iter()
                    .find(|f| f.name == s.field_name)
                    .map_or("Selected field unavailable", |f| f.name.as_str()),
            )
            .show_ui(ui, |ui| {
                for field in &step.fields {
                    ui.selectable_value(&mut s.field_name, field.name.clone(), &field.name);
                }
            });
        let Some(field) = step.field(&s.field_name) else {
            ui.label("Selected field absent at this timestep");
            return;
        };
        ui.small(format!(
            "{} · {} · dimensions {:?} · {} components",
            field.scalar_type, field.units, field.dimensions, field.components
        ));
        ui.label(&field.association);
        ui.label(&field.stage);
        ui.small(format!(
            "evaluation/interval-start {:?} s; publication {} s",
            field.evaluation_time, step.time
        ));
        ui.horizontal(|ui| {
            ui.label("Scalar index (x-fast, components interleaved)");
            ui.add(
                egui::DragValue::new(&mut s.scalar_index)
                    .range(0..=usize::MAX)
                    .clamp_existing_to_range(false),
            );
        });
        if let Some(value) = sample_value(field, s.scalar_index) {
            ui.monospace(format!("value {value} {}", field.units));
            let tuple = s.scalar_index / field.components;
            let dims = field.dimensions.map(|n| n as usize);
            if dims.iter().all(|n| *n > 0) {
                ui.small(format!(
                    "indices [{},{},{}], component {}",
                    tuple % dims[0],
                    tuple / dims[0] % dims[1],
                    tuple / (dims[0] * dims[1]),
                    s.scalar_index % field.components
                ));
            }
            if ui.button("Copy selected field sample CSV").clicked()
                && let Some(csv) = field_csv(
                    reader.run().identity.run_id,
                    step,
                    step.fields
                        .iter()
                        .position(|f| f.name == s.field_name)
                        .unwrap(),
                    s.scalar_index,
                )
            {
                ui.ctx().copy_text(csv);
            }
        } else {
            ui.label("No scalar at this index in selected field; no substituted value");
        }
    };
    if s.material.is_some() {
        ui.collapsing("Exact recorded field sample", body);
    } else {
        body(ui);
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
    #[test]
    fn carrier_field_selection_absence_real_egui_vti_export_and_metadata_errors() {
        let archive_path = test_path("carrier-fields");
        let mut sim = rheon::Simulation::new(
            rheon::GridGeometry::new([3, 2, 4], [0.2, 0.3, 0.4], [0.0; 3]).unwrap(),
            rheon::SimulationConfig {
                density: 1.0,
                memory_limit: 16 * 1024 * 1024,
                pressure: rheon::PressureSettings {
                    relative_residual: 1e-10,
                    absolute_residual: 1e-12,
                    divergence_limit: 1e-8,
                    max_iterations: 1000,
                },
                actual_divergence_limit: 1e-5,
                max_courant: 1.0,
            },
        )
        .unwrap();
        let inspector = Inspector::default();
        let limits = RecordingLimits::default();
        let mut writer = inspector
            .begin_carrier(&sim, &archive_path, limits)
            .unwrap();
        for _ in 0..2 {
            sim.step(0.01, None, |_| false).unwrap();
            inspector.record_carrier(&mut writer, &sim).unwrap();
        }
        writer
            .finish("completed", "UI carrier fields", Some(2))
            .unwrap();
        drop(writer);
        drop(sim);
        inspector.select_timestep(1).unwrap();
        let mut expected = Vec::new();
        {
            let mut state = inspector.0.lock().unwrap();
            state.field_name = "interval_pressure".into();
            state.scalar_index = 1;
            let carrier = state.carrier.as_ref().unwrap();
            carrier
                .write_vti(
                    &mut expected,
                    &state.reader.as_ref().unwrap().run().identity,
                )
                .unwrap();
        }
        let ctx = egui::Context::default();
        let render = |events| {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1400.0, 1600.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| inspector.ui(ui));
                },
            )
        };
        let output = render(vec![]);
        assert!(output.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.job.text==".generated/selected-carrier.vti")));
        let output_path = archive_path.with_extension("vti");
        inspector.0.lock().unwrap().carrier_export = ExportFile::new(output_path.to_str().unwrap());
        let click = |label: &str| {
            let _ = render(vec![]);
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
                .unwrap_or_else(|| panic!("missing carrier control {label}"));
            let _ = render(vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: Default::default(),
                },
            ]);
            render(vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            }])
        };
        let output = click("Copy selected field sample CSV");
        assert!(output.platform_output.commands.iter().any(|c|matches!(c,egui::OutputCommand::CopyText(text) if text.contains("\"interval_pressure\"") && text.lines().nth(1).unwrap().split(',').nth(2)==Some("1"))));
        let _ = click("Save selected timestep carrier (.vti)");
        assert_eq!(std::fs::read(&output_path).unwrap(), expected);
        inspector.select_timestep(0).unwrap();
        let _ = render(vec![]);
        {
            let state = inspector.0.lock().unwrap();
            assert_eq!(state.field_name, "interval_pressure");
            assert!(
                state
                    .record
                    .as_ref()
                    .unwrap()
                    .field(&state.field_name)
                    .is_none()
            );
        }
        let output = render(vec![]);
        assert!(!output.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.job.text=="Copy selected field sample CSV")));
        inspector.select_timestep(1).unwrap();
        inspector.0.lock().unwrap().scalar_index = usize::MAX;
        let _ = render(vec![]);
        assert_eq!(inspector.0.lock().unwrap().scalar_index, usize::MAX);
        let db = rusqlite::Connection::open(&archive_path).unwrap();
        db.execute("UPDATE lifecycle SET detail='corrupt lifecycle'", [])
            .unwrap();
        let output = render(vec![]);
        assert!(inspector.selected_identity().is_none());
        assert!(!output.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.job.text=="Save selected timestep carrier (.vti)")));
        drop(db);
        drop(inspector);
        std::fs::remove_file(archive_path).unwrap();
        std::fs::remove_file(output_path).unwrap();
        for mutation in [
            "UPDATE lifecycle SET detail='corrupt lifecycle'",
            "UPDATE steps SET ordinal=2 WHERE ordinal=0",
        ] {
            let path = test_path("metadata-error");
            let sim = ParticleFluidSimulation::new(
                rheon::GridGeometry::new([4; 3], [0.25; 3], [0.0; 3]).unwrap(),
                rheon::ParticleFluidConfig {
                    fill: rheon::DomainFill::EmittedFluid,
                    samples_per_axis: 4,
                    max_particles: 64,
                    ..Default::default()
                },
            )
            .unwrap();
            let inspector = Inspector::default();
            let mut writer = inspector.begin(&sim, &path, limits).unwrap();
            writer.finish("completed", "initial", Some(0)).unwrap();
            drop(writer);
            let db = rusqlite::Connection::open(&path).unwrap();
            db.execute_batch("PRAGMA foreign_keys=OFF").unwrap();
            db.execute_batch(mutation).unwrap();
            drop(db);
            let output = ctx.run(egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| inspector.ui(ui));
            });
            assert!(inspector.with_particles(|_, _, _, _, _| ()).is_none());
            assert!(inspector.selected_identity().is_none());
            assert!(!output.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.job.text=="Copy selected particle CSV")));
            drop(inspector);
            std::fs::remove_file(path).unwrap();
        }
    }
}
