//! Material controls keep displayed units separate from SI run configuration.
use eframe::egui;
use rheon::{DomainFill, FluidPhase, FluidPreset, ParticleFluidConfig};

#[derive(Clone, Copy, PartialEq, Eq)]
enum DensityUnit {
    KilogramsPerCubicMetre,
    GramsPerMillilitre,
    MilligramsPerCubicMillimetre,
}
impl DensityUnit {
    fn label(self) -> &'static str {
        match self {
            Self::KilogramsPerCubicMetre => "kg/m³",
            Self::GramsPerMillilitre => "g/mL",
            Self::MilligramsPerCubicMillimetre => "mg/mm³",
        }
    }
    fn factor(self) -> f64 {
        match self {
            Self::KilogramsPerCubicMetre => 1.0,
            _ => 1000.0,
        }
    }
}
pub struct MaterialControls {
    pub config: ParticleFluidConfig,
    preset: FluidPreset,
    ambient_preset: FluidPreset,
    unit: DensityUnit,
}
impl Default for MaterialControls {
    fn default() -> Self {
        Self {
            config: ParticleFluidConfig::default(),
            preset: FluidPreset::Water,
            ambient_preset: FluidPreset::Air,
            unit: DensityUnit::GramsPerMillilitre,
        }
    }
}
impl MaterialControls {
    pub fn color(&self) -> [u8; 3] {
        match self.preset {
            FluidPreset::Mercury => [200, 210, 220],
            FluidPreset::MoltenAluminium | FluidPreset::MoltenTin | FluidPreset::MoltenIron => {
                [255, 155, 65]
            }
            FluidPreset::Slime => [120, 225, 110],
            FluidPreset::Syrup => [210, 145, 50],
            FluidPreset::Argon => [180, 160, 240],
            FluidPreset::Helium => [190, 225, 250],
            _ => [85, 190, 245],
        }
    }
    pub fn fluid(&mut self, ui: &mut egui::Ui) {
        let custom = self.config.material != self.preset.material();
        egui::ComboBox::from_id_salt("material-preset")
            .selected_text(format!(
                "{}{}",
                self.preset.label(),
                if custom { " · edited" } else { "" }
            ))
            .width(245.0)
            .show_ui(ui, |ui| {
                for preset in FluidPreset::ALL {
                    if ui
                        .selectable_value(&mut self.preset, preset, preset.label())
                        .changed()
                    {
                        self.config.material = preset.material();
                    }
                }
            });
        ui.small("Approximate reference presets; edit values below.");
        let m = &mut self.config.material;
        ui.horizontal(|ui| {
            ui.label("State");
            ui.selectable_value(&mut m.phase, FluidPhase::Liquid, "Liquid");
            ui.selectable_value(&mut m.phase, FluidPhase::Gas, "Gas");
        });
        ui.horizontal(|ui| {
            ui.label("Density").on_hover_text(
                "Mass per volume. Weight also depends on gravity. 1 g/mL = 1 mg/mm³ = 1,000 kg/m³.",
            );
            let mut density = m.density / self.unit.factor();
            if ui
                .add(
                    egui::DragValue::new(&mut density)
                        .speed(0.001)
                        .range(1e-9 / self.unit.factor()..=1e9 / self.unit.factor())
                        .max_decimals(8),
                )
                .changed()
            {
                m.density = density * self.unit.factor();
            }
            egui::ComboBox::from_id_salt("density-unit")
                .selected_text(self.unit.label())
                .show_ui(ui, |ui| {
                    for unit in [
                        DensityUnit::KilogramsPerCubicMetre,
                        DensityUnit::GramsPerMillilitre,
                        DensityUnit::MilligramsPerCubicMillimetre,
                    ] {
                        ui.selectable_value(&mut self.unit, unit, unit.label());
                    }
                });
        });
        ui.small(format!("{:.3} kg/m³", m.density));
        quantity(ui,"Viscosity",&mut m.viscosity,"Pa·s",0.0001,0.0..=100000.0).on_hover_text("Dynamic viscosity: resistance to shear. This preview uses a Newtonian fluid; slime has no elastic stretching.");
        ui.add_enabled_ui(m.phase == FluidPhase::Liquid, |ui| {
            quantity(
                ui,
                "Surface tension",
                &mut m.surface_tension,
                "N/m",
                0.001,
                0.0..=10.0,
            )
            .on_hover_text(
                "Liquid cohesion approximation; larger values draw free surfaces together.",
            );
        });
        quantity(ui,"Wall drag",&mut m.wall_drag,"1/s",0.1,0.0..=1000.0).on_hover_text("Stickiness at walls: tangential velocity damping on contact. Separate from viscosity and surface tension; not a contact-angle model.");
        let mut temperature = m.temperature_kelvin - 273.15;
        if quantity(
            ui,
            "Fluid temperature",
            &mut temperature,
            "°C",
            1.0,
            -273.14..=4000.0,
        )
        .changed()
        {
            m.temperature_kelvin = temperature + 273.15;
        }
        if m.phase == FluidPhase::Gas {
            let mut molar_mass = m.molar_mass * 1000.0;
            if quantity(
                ui,
                "Molar mass",
                &mut molar_mass,
                "g/mol",
                0.1,
                0.1..=1000.0,
            )
            .changed()
            {
                m.molar_mass = molar_mass / 1000.0;
            }
            ui.small(format!(
                "Reference gas pressure {:.1} kPa",
                m.density * m.gas_constant() * m.temperature_kelvin / 1000.0
            ));
        }
        ui.small("Phase stays fixed: no boiling, freezing or elastic slime.");
        ui.small("Temperature affects gas pressure and wall exchange; viscosity stays at the entered value.");
    }
    pub fn domain(&mut self, ui: &mut egui::Ui, side: &mut f64) {
        quantity(ui, "Domain side", side, "m", 0.01, 0.001..=100.0);
        ui.small(format!(
            "Cube · {:.3} m³ · {:.3} L",
            side.powi(3),
            side.powi(3) * 1000.0
        ));
        egui::ComboBox::from_id_salt("domain-fill")
            .selected_text(match self.config.fill {
                DomainFill::Vacuum => "Empty / vacuum",
                DomainFill::EmittedFluid => "Filled with emitted fluid",
                DomainFill::OtherFluid => "Filled with another fluid",
            })
            .width(245.0)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.config.fill, DomainFill::Vacuum, "Empty / vacuum");
                ui.selectable_value(
                    &mut self.config.fill,
                    DomainFill::EmittedFluid,
                    "Filled with emitted fluid",
                );
                ui.selectable_value(
                    &mut self.config.fill,
                    DomainFill::OtherFluid,
                    "Filled with another fluid",
                );
            });
        if self.config.fill == DomainFill::OtherFluid {
            egui::ComboBox::from_id_salt("ambient-material")
                .selected_text(self.ambient_preset.label())
                .width(245.0)
                .show_ui(ui, |ui| {
                    for preset in FluidPreset::ALL {
                        if ui
                            .selectable_value(&mut self.ambient_preset, preset, preset.label())
                            .changed()
                        {
                            self.config.ambient_material = preset.material();
                        }
                    }
                });
            ui.small(
                "Surrounding fluid participates in forces; previews highlight emitted material.",
            );
        }
        if self.config.fill != DomainFill::Vacuum {
            let m = if self.config.fill == DomainFill::EmittedFluid {
                self.config.material
            } else {
                self.config.ambient_material
            };
            ui.small(format!("Initial mass {:.3} kg", side.powi(3) * m.density));
        }
        let mut ambient = self.config.ambient_temperature_kelvin - 273.15;
        if quantity(
            ui,
            "Ambient / wall temp",
            &mut ambient,
            "°C",
            1.0,
            -273.14..=4000.0,
        )
        .changed()
        {
            self.config.ambient_temperature_kelvin = ambient + 273.15;
        }
        quantity(ui,"Wall heat exchange",&mut self.config.wall_heat_exchange,"1/s",0.1,0.0..=100.0).on_hover_text("Temperature relaxes toward the wall temperature on contact. Vacuum has no surrounding gas or bulk heat bath.");
        ui.label("Gravity · m/s²");
        for d in 0..3 {
            quantity(
                ui,
                ["X", "Y", "Z"][d],
                &mut self.config.gravity[d],
                "m/s²",
                0.1,
                -100.0..=100.0,
            );
        }
        ui.horizontal(|ui| {
            if ui.button("Earth").clicked() {
                self.config.gravity = [0.0, -9.80665, 0.0];
            }
            if ui.button("Zero gravity").clicked() {
                self.config.gravity = [0.0; 3];
            }
        });
        ui.small("Closed, stationary domain walls.");
    }
}
pub fn quantity(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f64,
    unit: &str,
    speed: f64,
    range: std::ops::RangeInclusive<f64>,
) -> egui::Response {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(
            egui::DragValue::new(value)
                .speed(speed)
                .range(range)
                .max_decimals(8)
                .suffix(format!(" {unit}")),
        )
    })
    .inner
}
pub fn duration(seconds: f64) -> String {
    if seconds == 0.0 {
        "0 s".into()
    } else if seconds >= 1.0 {
        format!("{seconds:.3} s")
    } else if seconds >= 0.001 {
        format!("{:.3} ms", seconds * 1000.0)
    } else {
        format!("{:.1} µs", seconds * 1e6)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mass_density_units_use_independent_si_conversion() {
        assert_eq!(DensityUnit::GramsPerMillilitre.factor(), 1000.0);
        assert_eq!(DensityUnit::MilligramsPerCubicMillimetre.factor(), 1000.0);
        assert_eq!(DensityUnit::KilogramsPerCubicMetre.factor(), 1.0);
        assert_eq!(13.546 * DensityUnit::GramsPerMillilitre.factor(), 13546.0);
    }
}
