//! Bounded point-glyph cache derived from the inspector's accepted particles.
//! Selected numerical timestep is supplied by the inspector; absent fields stay absent.
use super::{inspector::Inspector, viewport::OrbitCamera};
use eframe::egui::{self, Color32, Rect, Stroke};
use rheon::FluidParticle;
const MAX_POINTS: usize = 8192;
#[derive(Default)]
pub struct ParticleView {
    key: Option<(u64, u64)>,
    points: Vec<(usize, FluidParticle)>,
    count: usize,
    range: [f64; 2],
    time: f64,
    selected: Option<(usize, FluidParticle)>,
    arrows: bool,
}
impl ParticleView {
    fn refresh(&mut self, inspector: &Inspector) {
        if inspector
            .with_particles(|identity, generation, time, particles, selected| {
                self.selected = particles.get(selected).copied().map(|p| (selected, p));
                let key = (identity.run_id, generation);
                if self.key == Some(key) {
                    return;
                }
                self.key = Some(key);
                self.time = time;
                self.count = particles.len();
                self.points.clear();
                self.range = [f64::INFINITY, f64::NEG_INFINITY];
                let stride = particles.len().div_ceil(MAX_POINTS).max(1);
                for (i, p) in particles.iter().enumerate() {
                    let speed = norm(p.velocity);
                    self.range[0] = self.range[0].min(speed);
                    self.range[1] = self.range[1].max(speed);
                    if i % stride == 0 {
                        self.points.push((i, *p));
                    }
                }
            })
            .is_none()
        {
            self.key = None;
            self.points.clear();
            self.selected = None;
            self.count = 0;
        }
    }
    pub fn controls(&mut self, ui: &mut egui::Ui, inspector: Option<&Inspector>) {
        if let Some(inspector) = inspector {
            self.refresh(inspector);
        } else {
            self.key = None;
            self.points.clear();
            self.selected = None;
            self.count = 0;
        }
        ui.label("Accepted particle speed at selected recorded timestep");
        if let Some((run, generation)) = self.key {
            ui.small(format!(
                "Run {run} · generation {generation} · {:.6} s · {} / {} points shown",
                self.time,
                self.points.len(),
                self.count
            ));
        }
        if self.count == 0 {
            ui.label("No particles at the selected step, or numerical fields unavailable.");
            return;
        }
        ui.horizontal_wrapped(|ui| {
            ui.label(format!(
                "Selected-state speed: {:.6} … {:.6} m/s",
                self.range[0], self.range[1]
            ));
            for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
                ui.colored_label(color(t), "■");
            }
            ui.checkbox(&mut self.arrows, "Velocity overlay (× 0.05 s)");
        });
        ui.small("All materials included. Display sampling does not change ranges or export. Click a visible point; all IDs remain available in Inspector.");
        if let Some((id, p)) = self.selected {
            ui.label(format!(
                "Particle {id} · position {:.4?} m · speed {:.6} m/s · material {}",
                p.position,
                norm(p.velocity),
                p.material
            ));
        }
        ui.small("Pressure is pre-integration data: inspect its evaluation time in Inspector; no synchronized pressure coloring is implied.");
    }
    pub fn paint(
        &self,
        ui: &egui::Ui,
        rect: Rect,
        response: &egui::Response,
        camera: &OrbitCamera,
        domain_side: f64,
        inspector: Option<&Inspector>,
    ) {
        let painter = ui.painter_at(rect);
        let point = |p: [f64; 3]| p.map(|v| (v / domain_side) as f32);
        let mut projected: Vec<_> = self
            .points
            .iter()
            .filter_map(|(id, p)| {
                camera
                    .project(point(p.position), rect)
                    .map(|(xy, z)| (*id, *p, xy, z))
            })
            .collect();
        projected.sort_by(|a, b| b.3.total_cmp(&a.3));
        let mut pick = None;
        for (id, p, xy, _) in projected {
            let span = self.range[1] - self.range[0];
            let t = if span > 0.0 {
                (norm(p.velocity) - self.range[0]) / span
            } else {
                0.5
            };
            painter.circle_filled(xy, 3.0, color(t));
            if self.selected.is_some_and(|(i, _)| i == id) {
                painter.circle_stroke(xy, 6.0, Stroke::new(1.5, Color32::WHITE));
            }
            if self.arrows
                && let Some((end, _)) = camera.project(
                    point(std::array::from_fn(|d| {
                        p.position[d] + 0.05 * p.velocity[d]
                    })),
                    rect,
                )
            {
                painter.arrow(xy, end - xy, Stroke::new(1.0, Color32::WHITE));
            }
            if response.clicked()
                && response
                    .interact_pointer_pos()
                    .is_some_and(|q| q.distance(xy) < 7.0)
            {
                pick = Some(id);
            }
        }
        if let (Some(id), Some(inspector)) = (pick, inspector) {
            inspector.select(id);
        }
        for i in 0..8 {
            for d in 0..3 {
                let j = i ^ (1 << d);
                if i < j {
                    let v = |k| std::array::from_fn(|a| if k & (1 << a) == 0 { 0.0 } else { 1.0 });
                    if let (Some((a, _)), Some((b, _))) =
                        (camera.project(v(i), rect), camera.project(v(j), rect))
                    {
                        painter.line_segment([a, b], Stroke::new(1.0, Color32::GRAY));
                    }
                }
            }
        }
    }
}
fn norm(v: [f64; 3]) -> f64 {
    v.iter().map(|x| x * x).sum::<f64>().sqrt()
}
fn color(t: f64) -> Color32 {
    Color32::from_rgb(
        (30.0 + 215.0 * t) as u8,
        (65.0 + 170.0 * t) as u8,
        (150.0 - 110.0 * t) as u8,
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selected_recorded_points_follow_timestep_and_clear_when_run_disappears() {
        let grid = rheon::GridGeometry::new([4; 3], [0.25; 3], [0.0; 3]).unwrap();
        let mut sim = rheon::ParticleFluidSimulation::new(
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
        let path =
            std::env::temp_dir().join(format!("rheon-particle-view-{}.sqlite", std::process::id()));
        let mut writer = inspector
            .begin(&sim, &path, rheon::recording::RecordingLimits::default())
            .unwrap();
        let mut view = ParticleView::default();
        view.refresh(&inspector);
        assert_eq!(view.count, sim.particles().len());
        assert_eq!(view.range, [0.0, 0.0]);
        let report = sim.step(1e-5, None, || false).unwrap();
        inspector
            .record(
                &mut writer,
                &sim,
                report,
                rheon::MaterialStepControls {
                    requested_dt: 1e-5,
                    emitter: None,
                },
            )
            .unwrap();
        inspector.select_timestep(1).unwrap();
        view.refresh(&inspector);
        assert_eq!(view.time, sim.time());
        assert_eq!(view.points[0].1, sim.particles()[0]);
        inspector.select(3);
        view.refresh(&inspector);
        assert_eq!(view.selected.unwrap().0, 3);
        let other = Inspector::default();
        view.refresh(&other);
        assert_eq!(view.count, 0);
        assert!(view.points.is_empty());
        assert!(view.selected.is_none());
        drop(writer);
        drop(inspector);
        std::fs::remove_file(path).unwrap();
    }
}
