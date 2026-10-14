//! Latest numerical state and a bounded selected-particle trace, separate from voxels.
use super::export_ui::ExportFile;
use eframe::egui;
use rheon::SessionIdentity;
use rheon::{FluidParticle, ParticleDiagnostic, ParticleFluidSimulation};
use std::collections::VecDeque;
use std::sync::Mutex;

#[derive(Clone, Copy)]
struct Sample {
    id: usize,
    generation: u64,
    time: f64,
    evaluation_time: Option<f64>,
    particle: FluidParticle,
    diagnostic: Option<ParticleDiagnostic>,
}
struct State {
    identity: SessionIdentity,
    export: ExportFile,
    selected: usize,
    generation: u64,
    time: f64,
    evaluation_time: Option<f64>,
    particles: Vec<FluidParticle>,
    diagnostics: Vec<ParticleDiagnostic>,
    trace: VecDeque<Sample>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            identity: super::identity::new_identity("material"),
            export: ExportFile::new(".generated/particles.vtp"),
            selected: 0,
            generation: 0,
            time: 0.0,
            evaluation_time: None,
            particles: Vec::new(),
            diagnostics: Vec::new(),
            trace: VecDeque::new(),
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
        self.0
            .lock()
            .ok()
            .map(|s| read(&s.identity, s.generation, s.time, &s.particles, s.selected))
    }
    pub fn select(&self, id: usize) {
        if let Ok(mut s) = self.0.lock()
            && id < s.particles.len()
            && id != s.selected
        {
            s.selected = id;
            s.restart_trace();
        }
    }
    pub fn capture(&self, sim: &ParticleFluidSimulation) -> Result<(), String> {
        let mut s = self.0.lock().map_err(|_| "Inspector owner failed")?;
        // Reuse storage. This is one latest state, not a particle archive per frame.
        let n = sim.particles().len();
        let additional = n.saturating_sub(s.particles.len());
        s.particles
            .try_reserve(additional)
            .map_err(|e| e.to_string())?;
        let additional = n.saturating_sub(s.diagnostics.len());
        s.diagnostics
            .try_reserve(additional)
            .map_err(|e| e.to_string())?;
        s.trace.try_reserve(1).map_err(|e| e.to_string())?;
        s.particles.clear();
        s.particles.extend_from_slice(sim.particles());
        s.diagnostics.clear();
        s.evaluation_time = sim.particle_diagnostics().map(|(time, values)| {
            s.diagnostics.extend_from_slice(values);
            time
        });
        s.generation = sim.generation();
        s.time = sim.time();
        if let Some(sample) = s.sample() {
            if s.trace.len() == 256 {
                s.trace.pop_front();
            }
            s.trace.push_back(sample);
        }
        Ok(())
    }
    pub fn ui(&self, ui: &mut egui::Ui) {
        let Ok(mut s) = self.0.lock() else {
            ui.label("Inspector unavailable");
            return;
        };
        ui.label("Latest numerical state (independent of visual scrubber)");
        ui.small(format!("Run {} (process-local)", s.identity.run_id));
        ui.small(
            "Particle IDs are append-only indices scoped to this run. New run clears history.",
        );
        ui.collapsing("Conservation diagnostics (accepted state)", |ui| {
            let mass: f64 = s.particles.iter().map(|p|p.mass).sum();
            let momentum: [f64;3] = std::array::from_fn(|d|s.particles.iter().map(|p|p.mass*p.velocity[d]).sum());
            let kinetic: f64 = s.particles.iter().map(|p|0.5*p.mass*p.velocity.iter().map(|v|v*v).sum::<f64>()).sum();
            ui.monospace(format!("mass {mass} kg\nmomentum {momentum:?} kg m/s\nkinetic energy {kinetic} J"));
            ui.small("Raw totals, not conservation residuals: emission, gravity, viscosity and walls exchange mass, momentum or energy. No closed-system conservation claim.");
        });
        let last = s.particles.len().saturating_sub(1);
        ui.horizontal(|ui| {
            ui.label("Particle ID");
            if ui
                .add(egui::DragValue::new(&mut s.selected).range(0..=last))
                .changed()
            {
                s.restart_trace();
            }
        });
        if let Some(sample) = s.sample() {
            ui.monospace(format!(
                "generation {}\ntime {} s",
                sample.generation, sample.time
            ));
            ui.collapsing("Position / velocity / material", |ui| {
                ui.monospace(format!(
                    "position {:?} m\nvelocity {:?} m/s\nmass {} kg\ntemperature {} K\nmaterial {}",
                    sample.particle.position,
                    sample.particle.velocity,
                    sample.particle.mass,
                    sample.particle.temperature_kelvin,
                    sample.particle.material
                ));
            });
            ui.collapsing("Density / pressure / force", |ui| {
                if let Some(d) = sample.diagnostic {
                    ui.label("Post-emission, pre-integration; excludes wall response");
                    ui.monospace(format!("evaluated at {} s\ndensity {} kg/m³\npressure {} Pa\nacceleration {:?} m/s²\nforce {:?} N",sample.evaluation_time.unwrap(),d.density,d.pressure,d.acceleration,d.acceleration.map(|a| a*sample.particle.mass)));
                } else { ui.label("Not evaluated (initial state)"); }
            });
            ui.label(format!(
                "{} selected-particle samples (latest 256 accepted captures)",
                s.trace.len()
            ));
            if ui.button("Copy numerical history CSV").clicked() {
                ui.ctx().copy_text(s.csv());
            }
        } else {
            ui.label("No particle at this ID");
        }
        ui.collapsing("VTK particle export", |ui| {
                ui.small("Accepted points; pre-integration diagnostics stored separately with their evaluation time.");
                let State { identity,generation,time,particles,diagnostics,evaluation_time,export,.. } = &mut *s;
                export.show(ui,"Save particle VTK (.vtp)",|out| rheon::write_particles_vtp(out,identity,*generation,*time,particles,evaluation_time.map(|t| (t,diagnostics.as_slice()))));
            });
    }
}
impl State {
    fn restart_trace(&mut self) {
        self.trace.clear();
        if let Some(sample) = self.sample() {
            self.trace.push_back(sample);
        }
    }
    fn sample(&self) -> Option<Sample> {
        self.particles
            .get(self.selected)
            .copied()
            .map(|particle| Sample {
                id: self.selected,
                generation: self.generation,
                time: self.time,
                evaluation_time: self.evaluation_time,
                particle,
                diagnostic: self.diagnostics.get(self.selected).copied(),
            })
    }
    fn csv(&self) -> String {
        use std::fmt::Write;
        let mut text = String::from(
            "particle_id,generation,time_s,evaluation_time_s,x_m,y_m,z_m,vx_m_s,vy_m_s,vz_m_s,mass_kg,temperature_K,material,density_kg_m3,pressure_Pa,ax_m_s2,ay_m_s2,az_m_s2\n",
        );
        for s in &self.trace {
            let p = s.particle;
            write!(
                text,
                "{},{},{},{},{},{},{},{},{},{},{},{},{}",
                s.id,
                s.generation,
                s.time,
                s.evaluation_time.map(|v| v.to_string()).unwrap_or_default(),
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
            if let Some(d) = s.diagnostic {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn owned_trace_is_bounded_and_round_trips_binary64() {
        let grid = rheon::GridGeometry::new([4; 3], [0.25; 3], [0.0; 3]).unwrap();
        let mut sim = ParticleFluidSimulation::new(
            grid,
            rheon::ParticleFluidConfig {
                fill: rheon::DomainFill::EmittedFluid,
                samples_per_axis: 4,
                max_particles: 64,
                gravity: [0.0; 3],
                ..Default::default()
            },
        )
        .unwrap();
        let inspector = Inspector::default();
        inspector.capture(&sim).unwrap();
        assert!(inspector.0.lock().unwrap().trace[0].diagnostic.is_none());
        let initial_csv = inspector.0.lock().unwrap().csv();
        let initial: Vec<_> = initial_csv.lines().nth(1).unwrap().split(',').collect();
        assert_eq!(initial.len(), 18);
        assert!(initial[13..].iter().all(|value| value.is_empty()));
        for _ in 0..260 {
            sim.step(1e-7, None, || false).unwrap();
            inspector.capture(&sim).unwrap();
        }
        let mut s = inspector.0.lock().unwrap();
        assert_eq!(s.trace.len(), 256);
        let csv = s.csv();
        let row: Vec<_> = csv.lines().last().unwrap().split(',').collect();
        assert_eq!(row.len(), 18);
        assert_eq!(
            row[4].parse::<f64>().unwrap().to_bits(),
            sim.particles()[0].position[0].to_bits()
        );
        assert_eq!(s.trace.back().unwrap().generation, 260);
        s.selected = 1;
        s.restart_trace();
        assert_eq!(s.trace.len(), 1);
        assert_eq!(s.csv().lines().count(), 2);
        assert_eq!(s.trace[0].id, 1);
        drop(sim);
        assert_eq!(s.trace.back().unwrap().generation, 260);
    }
}
