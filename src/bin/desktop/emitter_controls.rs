//! Native editing of the same SI emitter controls retained by run requests.
use super::settings::quantity;
use eframe::egui;
use rheon::RotatingCylinderEmitter;
pub struct EmitterControls {
    direction: [f64; 3],
}
impl Default for EmitterControls {
    fn default() -> Self {
        Self {
            direction: [1.0, 0.0, 0.0],
        }
    }
}
fn norm(v: [f64; 3]) -> f64 {
    v[0].hypot(v[1]).hypot(v[2])
}
fn vector(direction: [f64; 3], speed: f64) -> Option<[f64; 3]> {
    let n = norm(direction);
    (n.is_finite() && n > 1e-12 && speed.is_finite() && speed >= 0.0)
        .then(|| direction.map(|v| v / n * speed))
}
impl EmitterControls {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        e: &mut RotatingCylinderEmitter,
        side: f64,
        density: f64,
    ) {
        for d in 0..3 {
            quantity(
                ui,
                ["Initial center X", "Initial center Y", "Initial center Z"][d],
                &mut e.center[d],
                "m",
                0.001,
                0.0..=side,
            );
        }
        let mut length = 2.0 * e.half_length;
        if quantity(
            ui,
            "Cylinder length",
            &mut length,
            "m",
            0.001,
            0.0001..=side,
        )
        .changed()
        {
            e.half_length = length * 0.5;
        }
        quantity(ui, "Radius", &mut e.radius, "m", 0.001, 0.0001..=side * 0.4);
        quantity(
            ui,
            "Nozzle depth",
            &mut e.nozzle_depth,
            "m",
            0.001,
            0.0001..=side * 0.4,
        );
        quantity(
            ui,
            "Spin about Y",
            &mut e.rotations_per_second,
            "rev/s",
            0.05,
            -10.0..=10.0,
        );
        let mut explicit = e.emitted_velocity.is_some();
        if ui
            .checkbox(&mut explicit, "Set jet velocity vector")
            .changed()
        {
            e.emitted_velocity =
                explicit.then(|| [(2.0 * e.pressure_heads[0] / density).sqrt(), 0.0, 0.0]);
        }
        if let Some(mut v) = e.emitted_velocity {
            ui.small("Positive jet relative to nozzle; other end is opposed.");
            for d in 0..3 {
                quantity(
                    ui,
                    ["Jet velocity X", "Jet velocity Y", "Jet velocity Z"][d],
                    &mut v[d],
                    "m/s",
                    0.01,
                    -1000.0..=1000.0,
                );
            }
            let mut speed = norm(v);
            if speed > 0.0 {
                self.direction = v.map(|x| x / speed);
            }
            let mut direction = self.direction;
            let mut changed =
                quantity(ui, "Jet speed", &mut speed, "m/s", 0.01, 0.0..=1000.0).changed();
            ui.horizontal(|ui| {
                ui.label("Direction");
                for x in &mut direction {
                    changed |= ui
                        .add(egui::DragValue::new(x).speed(0.01).range(-1.0..=1.0))
                        .changed();
                }
            });
            if changed && let Some(next) = vector(direction, speed) {
                v = next;
                self.direction = direction.map(|x| x / norm(direction));
            }
            e.emitted_velocity = Some(v);
        } else {
            for end in 0..2 {
                quantity(
                    ui,
                    if end == 0 { "End A head" } else { "End B head" },
                    &mut e.pressure_heads[end],
                    "Pa",
                    10.0,
                    0.0..=1000000.0,
                );
            }
        }
        egui::CollapsingHeader::new("Emitter motion").show(ui, |ui| {
            ui.small("Moves the source center; adds to injected world velocity.");
            for d in 0..3 {
                quantity(
                    ui,
                    ["Motion X", "Motion Y", "Motion Z"][d],
                    &mut e.motion_velocity[d],
                    "m/s",
                    0.001,
                    -1000.0..=1000.0,
                );
            }
        });
        egui::CollapsingHeader::new("Inlet turbulence").show(ui,|ui|{
            quantity(ui,"Strength bound",&mut e.turbulence.strength,"m/s",0.01,0.0..=1000.0);
            quantity(ui,"Spatial scale",&mut e.turbulence.spatial_scale,"m",0.001,1e-6..=1000.0);
            quantity(ui,"Evolution time",&mut e.turbulence.time_scale,"s",0.01,1e-6..=1000.0);
            ui.horizontal(|ui|{ui.label("Seed");ui.add(egui::DragValue::new(&mut e.turbulence.seed));});
            ui.small("Seeded inlet perturbation, not a validated turbulence model. Strength 0 disables it.");
        });
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn direction_speed_and_zero_are_consistent() {
        assert_eq!(vector([0.0, 3.0, 4.0], 10.0), Some([0.0, 6.0, 8.0]));
        assert_eq!(vector([0.0, 3.0, 4.0], 0.0), Some([0.0; 3]));
        assert!(vector([0.0; 3], 1.0).is_none());
    }
    #[test]
    fn rendering_controls_retains_si_fields() {
        let ctx = egui::Context::default();
        let mut ui = EmitterControls::default();
        let mut e = RotatingCylinderEmitter {
            emitted_velocity: Some([0.0, 3.0, 4.0]),
            motion_velocity: [0.1, 0.0, 0.0],
            ..Default::default()
        };
        let before = e;
        let output = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |panel| ui.show(panel, &mut e, 1.0, 1000.0));
        });
        assert!(!output.shapes.is_empty());
        assert_eq!(e, before);
    }
}
