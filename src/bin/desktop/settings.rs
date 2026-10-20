//! Material controls keep displayed units separate from SI run configuration.
use eframe::egui;
use rheon::{DomainFill, FluidPhase, FluidPreset, ParticleFluidConfig};

const DENSITY_HELP: &str =
    "Mass per volume. Weight also depends on gravity. 1 g/mL = 1 mg/mm³ = 1,000 kg/m³.";
const VISCOSITY_HELP: &str = "Dynamic viscosity: resistance to shear. This preview uses a Newtonian fluid; slime has no elastic stretching.";
const TENSION_HELP: &str =
    "Liquid cohesion approximation; larger values draw free surfaces together.";
const WALL_DRAG_HELP: &str = "Stickiness at walls: tangential velocity damping on contact. Separate from viscosity and surface tension; not a contact-angle model.";
const WALL_HEAT_HELP: &str = "Temperature relaxes toward the wall temperature on contact. Vacuum has no surrounding gas or bulk heat bath.";

/// Combo widths are minimums in egui; bound the complete label as well.
fn preset_text(ui: &egui::Ui, text: impl Into<egui::WidgetText>) -> egui::WidgetText {
    text.into()
        .into_galley(
            ui,
            Some(egui::TextWrapMode::Wrap),
            (ui.available_width()
                - 2.0 * ui.spacing().button_padding.x
                - ui.spacing().icon_spacing
                - ui.spacing().icon_width)
                .max(1.0),
            egui::TextStyle::Button,
        )
        .into()
}

/// Egui reparses cached edit text on blur. Only focused text needs lossless
/// formatting; compact idle formatting must never round the underlying value.
fn precise_drag(ui: &mut egui::Ui, value: f64, control: egui::DragValue<'_>) -> egui::Response {
    let id = ui.next_auto_id();
    let ctx = ui.ctx().clone();
    let formatter = ui.style().number_formatter.clone();
    let response = ui.add(control.custom_formatter(move |value, decimals| {
        // Query when invoked, after DragValue registers keyboard focus.
        if ctx.memory(|memory| memory.has_focus(id)) {
            // Debug keeps round-trip precision and uses scientific notation for
            // tiny values, avoiding hundreds of leading zeroes in the editor.
            format!("{value:?}")
        } else {
            formatter.format(value, decimals)
        }
    }));
    if response.clicked() && response.gained_focus() {
        // The button selected its compact text; the edit buffer will be longer.
        // Align this initial selection only, preserving later caret placement.
        let mut state = egui::TextEdit::load_state(ui.ctx(), id).unwrap_or_default();
        state
            .cursor
            .set_char_range(Some(egui::text::CCursorRange::two(
                egui::text::CCursor::new(0),
                egui::text::CCursor::new(format!("{value:?}").chars().count()),
            )));
        state.store(ui.ctx(), id);
    }
    response
}

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
        let preset_label = preset_text(
            ui,
            format!(
                "{}{}",
                self.preset.label(),
                if custom { " · edited" } else { "" }
            ),
        );
        egui::ComboBox::from_id_salt("material-preset")
            .selected_text(preset_label)
            .width(ui.available_width().min(245.0))
            .show_ui(ui, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                for preset in FluidPreset::ALL {
                    if ui
                        .selectable_value(&mut self.preset, preset, preset.label())
                        .changed()
                    {
                        self.config.material = preset.material();
                    }
                }
            });
        ui.small("Approximate presets · editable");
        let m = &mut self.config.material;
        ui.horizontal_wrapped(|ui| {
            ui.label("State");
            ui.selectable_value(&mut m.phase, FluidPhase::Liquid, "Liquid");
            ui.selectable_value(&mut m.phase, FluidPhase::Gas, "Gas");
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("Density").on_hover_text(DENSITY_HELP);
            let mut density = m.density / self.unit.factor();
            if precise_drag(
                ui,
                density,
                egui::DragValue::new(&mut density)
                    .speed(0.001)
                    .range(1e-9 / self.unit.factor()..=1e9 / self.unit.factor())
                    .max_decimals(8),
            )
            .changed()
            {
                m.density = density * self.unit.factor();
            }
            // The focused value can be wider; ComboBox's inner horizontal UI
            // cannot wrap itself onto the next row.
            if ui.available_size_before_wrap().x < 80.0 + 2.0 * ui.spacing().button_padding.x {
                ui.end_row();
            }
            egui::ComboBox::from_id_salt("density-unit")
                .selected_text(self.unit.label())
                .width(80.0)
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
        quantity(
            ui,
            "Viscosity",
            &mut m.viscosity,
            "Pa·s",
            0.0001,
            0.0..=100000.0,
        )
        .on_hover_text(VISCOSITY_HELP);
        ui.add_enabled_ui(m.phase == FluidPhase::Liquid, |ui| {
            quantity(
                ui,
                "Surface tension",
                &mut m.surface_tension,
                "N/m",
                0.001,
                0.0..=10.0,
            )
            .on_hover_text(TENSION_HELP);
        });
        quantity(ui, "Wall drag", &mut m.wall_drag, "1/s", 0.1, 0.0..=1000.0)
            .on_hover_text(WALL_DRAG_HELP);
        let mut temperature = m.temperature_kelvin - 273.15;
        egui::CollapsingHeader::new(format!("Temperature · {temperature:.2} °C"))
            .id_salt("fluid-temperature")
            .show(ui, |ui| {
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
                ui.small("Temperature affects gas pressure and wall exchange; viscosity stays at the entered value.");
            });
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
        egui::CollapsingHeader::new("Property help")
            .id_salt("material-property-help")
            .show(ui, |ui| {
                ui.label(format!("Density: {DENSITY_HELP}"));
                ui.label(VISCOSITY_HELP);
                ui.label(format!("Surface tension: {TENSION_HELP}"));
                ui.label(format!("Wall drag: {WALL_DRAG_HELP}"));
            });
    }
    pub fn domain(&mut self, ui: &mut egui::Ui, side: &mut f64) {
        quantity(ui, "Domain side", side, "m", 0.01, 0.001..=100.0);
        ui.small(format!(
            "Cube · {:.3} m³ · {:.3} L",
            side.powi(3),
            side.powi(3) * 1000.0
        ));
        let fill_label = preset_text(
            ui,
            match self.config.fill {
                DomainFill::Vacuum => "Empty / vacuum",
                DomainFill::EmittedFluid => "Filled with emitted fluid",
                DomainFill::OtherFluid => "Filled with another fluid",
            },
        );
        egui::ComboBox::from_id_salt("domain-fill")
            .selected_text(fill_label)
            .width(ui.available_width().min(245.0))
            .show_ui(ui, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
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
            let ambient_label = preset_text(ui, self.ambient_preset.label());
            egui::ComboBox::from_id_salt("ambient-material")
                .selected_text(ambient_label)
                .width(ui.available_width().min(245.0))
                .show_ui(ui, |ui| {
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
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
        egui::CollapsingHeader::new(format!("Wall heat · {ambient:.2} °C"))
            .id_salt("domain-thermal-environment")
            .show(ui, |ui| {
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
                quantity(
                    ui,
                    "Wall heat exchange",
                    &mut self.config.wall_heat_exchange,
                    "1/s",
                    0.1,
                    0.0..=100.0,
                )
                .on_hover_text(WALL_HEAT_HELP);
                ui.label(WALL_HEAT_HELP);
            });
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
        ui.horizontal_wrapped(|ui| {
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
    ui.horizontal_wrapped(|ui| {
        ui.label(label);
        precise_drag(
            ui,
            *value,
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

    fn context() -> egui::Context {
        let ctx = egui::Context::default();
        ctx.style_mut(|style| style.animation_time = 0.0);
        ctx
    }
    fn frame(
        ctx: &egui::Context,
        width: f32,
        controls: &mut MaterialControls,
        side: &mut f64,
        domain: bool,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        let mut right = 0.0;
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(width, 1600.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    if domain {
                        controls.domain(ui, side);
                    } else {
                        controls.fluid(ui);
                    }
                    right = ui.min_rect().right();
                });
            },
        );
        assert!(
            right <= width - 7.0,
            "form overflow {right} at width {width}, domain {domain}, preset {:?}: {:?}",
            controls.preset,
            output
                .shapes
                .iter()
                .filter_map(|s| match &s.shape {
                    egui::Shape::Text(t) if t.pos.x + t.galley.size().x > width - 7.0 =>
                        Some((t.galley.text(), t.pos.x, t.galley.size().x)),
                    _ => None,
                })
                .collect::<Vec<_>>()
        );
        output
    }
    fn text_rect(output: &egui::FullOutput, label: &str) -> Option<egui::Rect> {
        output.shapes.iter().find_map(|s| match &s.shape {
            egui::Shape::Text(t) if t.galley.text() == label => {
                Some(egui::Rect::from_min_size(t.pos, t.galley.size()))
            }
            _ => None,
        })
    }
    fn pointer(pos: egui::Pos2, pressed: bool) -> Vec<egui::Event> {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Default::default(),
            },
        ]
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
    fn click(
        ctx: &egui::Context,
        width: f32,
        controls: &mut MaterialControls,
        side: &mut f64,
        domain: bool,
        output: &egui::FullOutput,
        label: &str,
    ) {
        let pos = text_rect(output, label)
            .unwrap_or_else(|| panic!("missing {label}"))
            .center();
        for pressed in [true, false] {
            let _ = frame(ctx, width, controls, side, domain, pointer(pos, pressed));
        }
    }

    // UI-only helpers below use Context/MaterialControls and their pure config
    // defaults. They never construct a simulation, worker, recording or file.
    fn material_bits(controls: &MaterialControls) -> [u64; 6] {
        let m = controls.config.material;
        [
            m.density,
            m.viscosity,
            m.surface_tension,
            m.wall_drag,
            m.temperature_kelvin,
            m.molar_mass,
        ]
        .map(f64::to_bits)
    }
    // Density is the only unitless numeric editor in this form; molar mass
    // has a unique suffix. Do not assume the compact formatter's spelling.
    fn numeric_label(output: &egui::FullOutput, suffix: &str) -> String {
        output
            .shapes
            .iter()
            .find_map(|s| match &s.shape {
                egui::Shape::Text(t)
                    if t.galley
                        .text()
                        .strip_suffix(suffix)
                        .and_then(|s| s.parse::<f64>().ok())
                        .is_some() =>
                {
                    Some(t.galley.text().to_owned())
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("missing numeric editor with suffix {suffix}"))
    }
    fn replace_focused(
        ctx: &egui::Context,
        controls: &mut MaterialControls,
        side: &mut f64,
        text: &str,
    ) {
        let _ = frame(
            ctx,
            280.0,
            controls,
            side,
            false,
            key(
                egui::Key::A,
                egui::Modifiers {
                    ctrl: true,
                    command: true,
                    ..Default::default()
                },
            ),
        );
        let _ = frame(
            ctx,
            280.0,
            controls,
            side,
            false,
            vec![egui::Event::Text(text.into())],
        );
        let _ = frame(
            ctx,
            280.0,
            controls,
            side,
            false,
            key(egui::Key::Enter, Default::default()),
        );
        let _ = frame(ctx, 280.0, controls, side, false, vec![]);
    }

    #[test]
    fn ui_precision_tab_and_focus_preserve_material_bits() {
        for width in [240.0, 280.0] {
            for unit in [
                DensityUnit::KilogramsPerCubicMetre,
                DensityUnit::GramsPerMillilitre,
                DensityUnit::MilligramsPerCubicMillimetre,
            ] {
                for preset in [FluidPreset::Air, FluidPreset::Water] {
                    let ctx = context();
                    let mut controls = MaterialControls {
                        preset,
                        unit,
                        ..Default::default()
                    };
                    controls.config.material = preset.material();
                    if preset == FluidPreset::Water {
                        controls.config.material.viscosity = 1.2345678901234567e-200;
                        controls.config.material.surface_tension = 0.07281234567890123;
                        controls.config.material.wall_drag = 5.012345678901234;
                        controls.config.material.temperature_kelvin = 293.1512345678901;
                    }
                    let bits = material_bits(&controls);
                    let before = format!("{:?}", controls.config);
                    let mut side = 1.0;
                    let _ = frame(&ctx, width, &mut controls, &mut side, false, vec![]);
                    let output = frame(&ctx, width, &mut controls, &mut side, false, vec![]);
                    let density_label = numeric_label(&output, "");
                    click(
                        &ctx,
                        width,
                        &mut controls,
                        &mut side,
                        false,
                        &output,
                        &density_label,
                    );
                    let output = frame(&ctx, width, &mut controls, &mut side, false, vec![]);
                    assert_eq!(material_bits(&controls), bits, "pointer focus without edit");
                    let header = format!(
                        "Temperature · {:.2} °C",
                        controls.config.material.temperature_kelvin - 273.15
                    );
                    click(
                        &ctx,
                        width,
                        &mut controls,
                        &mut side,
                        false,
                        &output,
                        &header,
                    );
                    let _ = frame(&ctx, width, &mut controls, &mut side, false, vec![]);
                    let mut visited = std::collections::HashSet::new();
                    for shift in [false, true] {
                        for _ in 0..28 {
                            let _ = frame(
                                &ctx,
                                width,
                                &mut controls,
                                &mut side,
                                false,
                                key(
                                    egui::Key::Tab,
                                    egui::Modifiers {
                                        shift,
                                        ..Default::default()
                                    },
                                ),
                            );
                            let _ = frame(&ctx, width, &mut controls, &mut side, false, vec![]);
                            if let Some(id) = ctx.memory(|m| m.focused()) {
                                visited.insert(id);
                            }
                            assert_eq!(
                                material_bits(&controls),
                                bits,
                                "focus-only change for {}",
                                preset.label()
                            );
                            assert_eq!(format!("{:?}", controls.config), before);
                        }
                    }
                    assert!(visited.len() >= 10, "traverse actual form controls");
                    let _ = frame(
                        &ctx,
                        width,
                        &mut controls,
                        &mut side,
                        false,
                        key(egui::Key::Escape, Default::default()),
                    );
                    let _ = frame(&ctx, width, &mut controls, &mut side, false, vec![]);
                    assert_eq!(material_bits(&controls), bits);
                }
            }
        }
    }

    #[test]
    fn ui_precision_explicit_edits_keep_si_units_and_ranges() {
        for unit in [
            DensityUnit::KilogramsPerCubicMetre,
            DensityUnit::GramsPerMillilitre,
            DensityUnit::MilligramsPerCubicMillimetre,
        ] {
            let ctx = context();
            let mut controls = MaterialControls {
                unit,
                ..Default::default()
            };
            let mut side = 1.0;
            let mut expected = controls.config.material;
            let _ = frame(&ctx, 280.0, &mut controls, &mut side, false, vec![]);
            let output = frame(&ctx, 280.0, &mut controls, &mut side, false, vec![]);
            let initial = numeric_label(&output, "");
            click(
                &ctx,
                280.0,
                &mut controls,
                &mut side,
                false,
                &output,
                &initial,
            );
            // Ordinary click-to-replace must select the full precision text.
            let _ = frame(&ctx, 280.0, &mut controls, &mut side, false, vec![]);
            let _ = frame(
                &ctx,
                280.0,
                &mut controls,
                &mut side,
                false,
                vec![egui::Event::Text("2.5".into())],
            );
            let _ = frame(
                &ctx,
                280.0,
                &mut controls,
                &mut side,
                false,
                key(egui::Key::Enter, Default::default()),
            );
            let _ = frame(&ctx, 280.0, &mut controls, &mut side, false, vec![]);
            expected.density = 2.5 * unit.factor();
            assert_eq!(controls.config.material, expected);
            let output = frame(&ctx, 280.0, &mut controls, &mut side, false, vec![]);
            click(
                &ctx,
                280.0,
                &mut controls,
                &mut side,
                false,
                &output,
                &numeric_label(&output, ""),
            );
            replace_focused(&ctx, &mut controls, &mut side, "1e99");
            assert_eq!(controls.config.material.density, 1e9);
        }
        let ctx = context();
        let mut controls = MaterialControls {
            preset: FluidPreset::Air,
            ..Default::default()
        };
        controls.config.material = FluidPreset::Air.material();
        let mut side = 1.0;
        let _ = frame(&ctx, 280.0, &mut controls, &mut side, false, vec![]);
        let output = frame(&ctx, 280.0, &mut controls, &mut side, false, vec![]);
        click(
            &ctx,
            280.0,
            &mut controls,
            &mut side,
            false,
            &output,
            &numeric_label(&output, " g/mol"),
        );
        replace_focused(&ctx, &mut controls, &mut side, "32");
        assert_eq!(controls.config.material.molar_mass, 0.032);
        let output = frame(&ctx, 280.0, &mut controls, &mut side, false, vec![]);
        click(
            &ctx,
            280.0,
            &mut controls,
            &mut side,
            false,
            &output,
            &numeric_label(&output, " g/mol"),
        );
        let _ = frame(&ctx, 280.0, &mut controls, &mut side, false, vec![]);
        let _ = frame(
            &ctx,
            280.0,
            &mut controls,
            &mut side,
            false,
            key(egui::Key::ArrowUp, Default::default()),
        );
        assert_eq!(
            controls.config.material.molar_mass.to_bits(),
            (32.1_f64 / 1000.0).to_bits()
        );
        replace_focused(&ctx, &mut controls, &mut side, "32");
        let output = frame(&ctx, 280.0, &mut controls, &mut side, false, vec![]);
        click(
            &ctx,
            280.0,
            &mut controls,
            &mut side,
            false,
            &output,
            &numeric_label(&output, " g/mol"),
        );
        replace_focused(&ctx, &mut controls, &mut side, "not a number");
        assert_eq!(controls.config.material.molar_mass, 0.032);
        let output = frame(&ctx, 280.0, &mut controls, &mut side, false, vec![]);
        click(
            &ctx,
            280.0,
            &mut controls,
            &mut side,
            false,
            &output,
            &numeric_label(&output, " g/mol"),
        );
        replace_focused(&ctx, &mut controls, &mut side, "-4");
        assert_eq!(controls.config.material.molar_mass, 0.0001);
    }

    #[test]
    fn ui_material_thermal_disclosures_preserve_values_and_si_editing() {
        for domain in [false, true] {
            let ctx = context();
            let mut controls = MaterialControls::default();
            let mut side = 1.0;
            let before = format!("{:?}", controls.config);
            let header = if domain {
                "Wall heat · 20.00 °C"
            } else {
                "Temperature · 20.00 °C"
            };
            let label = if domain {
                "Ambient / wall temp"
            } else {
                "Fluid temperature"
            };
            let _ = frame(&ctx, 240.0, &mut controls, &mut side, domain, vec![]);
            let output = frame(&ctx, 240.0, &mut controls, &mut side, domain, vec![]);
            assert!(text_rect(&output, label).is_none());
            click(
                &ctx,
                240.0,
                &mut controls,
                &mut side,
                domain,
                &output,
                header,
            );
            let output = frame(&ctx, 240.0, &mut controls, &mut side, domain, vec![]);
            assert!(text_rect(&output, label).is_some());
            assert_eq!(format!("{:?}", controls.config), before);
            click(
                &ctx,
                240.0,
                &mut controls,
                &mut side,
                domain,
                &output,
                "20 °C",
            );
            let _ = frame(&ctx, 240.0, &mut controls, &mut side, domain, vec![]);
            let _ = frame(
                &ctx,
                240.0,
                &mut controls,
                &mut side,
                domain,
                vec![egui::Event::Text("37.25".into())],
            );
            let _ = frame(
                &ctx,
                240.0,
                &mut controls,
                &mut side,
                domain,
                key(egui::Key::Enter, Default::default()),
            );
            let output = frame(&ctx, 240.0, &mut controls, &mut side, domain, vec![]);
            let temperature = if domain {
                controls.config.ambient_temperature_kelvin
            } else {
                controls.config.material.temperature_kelvin
            };
            assert!(
                (temperature - 310.4).abs() < 1e-10,
                "Celsius edit retained in kelvin"
            );
            assert!(
                text_rect(&output, label).is_some(),
                "editing header value must not close disclosure"
            );
            let header = if domain {
                "Wall heat · 37.25 °C"
            } else {
                "Temperature · 37.25 °C"
            };
            let edited = format!("{:?}", controls.config);
            click(
                &ctx,
                240.0,
                &mut controls,
                &mut side,
                domain,
                &output,
                header,
            );
            let output = frame(&ctx, 240.0, &mut controls, &mut side, domain, vec![]);
            assert!(text_rect(&output, label).is_none());
            click(
                &ctx,
                240.0,
                &mut controls,
                &mut side,
                domain,
                &output,
                header,
            );
            let output = frame(&ctx, 240.0, &mut controls, &mut side, domain, vec![]);
            assert!(text_rect(&output, label).is_some());
            assert_eq!(format!("{:?}", controls.config), edited);
            click(
                &ctx,
                240.0,
                &mut controls,
                &mut side,
                domain,
                &output,
                "37.25 °C",
            );
            let _ = frame(&ctx, 240.0, &mut controls, &mut side, domain, vec![]);
            let _ = frame(
                &ctx,
                240.0,
                &mut controls,
                &mut side,
                domain,
                vec![egui::Event::Text("38.5".into())],
            );
            let _ = frame(
                &ctx,
                240.0,
                &mut controls,
                &mut side,
                domain,
                key(egui::Key::Enter, Default::default()),
            );
            let output = frame(&ctx, 240.0, &mut controls, &mut side, domain, vec![]);
            let temperature = if domain {
                controls.config.ambient_temperature_kelvin
            } else {
                controls.config.material.temperature_kelvin
            };
            assert!((temperature - 311.65).abs() < 1e-10);
            assert!(text_rect(&output, label).is_some());
            assert_eq!(side, 1.0);
        }
    }

    #[test]
    fn ui_material_help_is_tab_reachable_and_model_limits_remain_visible() {
        let ctx = context();
        let mut controls = MaterialControls {
            preset: FluidPreset::Air,
            ..Default::default()
        };
        controls.config.material = controls.preset.material();
        let before = format!("{:?}", controls.config);
        let mut side = 1.0;
        let mut output = frame(&ctx, 240.0, &mut controls, &mut side, false, vec![]);
        assert!(
            text_rect(
                &output,
                "Phase stays fixed: no boiling, freezing or elastic slime."
            )
            .is_some()
        );
        assert!(output.shapes.iter().any(|s| matches!(&s.shape, egui::Shape::Text(t) if t.galley.text().starts_with("Reference gas pressure "))));
        let help = "Density: Mass per volume. Weight also depends on gravity. 1 g/mL = 1 mg/mm³ = 1,000 kg/m³.";
        assert!(text_rect(&output, help).is_none());
        // Reach help through the actual reverse tab order, without focusing numeric
        // editors. General numeric-focus rounding is a separate existing UI issue.
        let _ = frame(&ctx, 240.0, &mut controls, &mut side, false, vec![]);
        let _ = frame(
            &ctx,
            240.0,
            &mut controls,
            &mut side,
            false,
            key(egui::Key::Tab, Default::default()),
        );
        let _ = frame(
            &ctx,
            240.0,
            &mut controls,
            &mut side,
            false,
            key(
                egui::Key::Tab,
                egui::Modifiers {
                    shift: true,
                    ..Default::default()
                },
            ),
        );
        output = frame(&ctx, 240.0, &mut controls, &mut side, false, vec![]);
        let focused = ctx
            .memory(|m| m.focused())
            .and_then(|id| ctx.read_response(id))
            .expect("help has keyboard focus");
        assert!(
            focused
                .rect
                .contains(text_rect(&output, "Property help").unwrap().center())
        );
        let _ = frame(
            &ctx,
            240.0,
            &mut controls,
            &mut side,
            false,
            key(egui::Key::Enter, Default::default()),
        );
        output = frame(&ctx, 240.0, &mut controls, &mut side, false, vec![]);
        assert!(text_rect(&output, help).is_some());
        let help_id = ctx.memory(|m| m.focused()).unwrap();
        let _ = frame(
            &ctx,
            240.0,
            &mut controls,
            &mut side,
            false,
            key(egui::Key::Space, Default::default()),
        );
        output = frame(&ctx, 240.0, &mut controls, &mut side, false, vec![]);
        assert!(text_rect(&output, help).is_none());
        let _ = frame(
            &ctx,
            240.0,
            &mut controls,
            &mut side,
            false,
            key(egui::Key::Tab, Default::default()),
        );
        let _ = frame(&ctx, 240.0, &mut controls, &mut side, false, vec![]);
        assert_ne!(ctx.memory(|m| m.focused()), Some(help_id));
        assert_eq!(format!("{:?}", controls.config), before);
    }

    #[test]
    fn ui_material_selection_updates_only_its_existing_owner() {
        for width in [240.0, 280.0] {
            for domain in [false, true] {
                let ctx = context();
                let mut controls = MaterialControls::default();
                controls.config.fill = DomainFill::OtherFluid;
                let mut expected = controls.config;
                let mut side = 1.0;
                let _ = frame(&ctx, width, &mut controls, &mut side, domain, vec![]);
                let output = frame(&ctx, width, &mut controls, &mut side, domain, vec![]);
                let label = if domain {
                    "Air · 20 °C"
                } else {
                    "Water · 20 °C"
                };
                click(
                    &ctx,
                    width,
                    &mut controls,
                    &mut side,
                    domain,
                    &output,
                    label,
                );
                let output = frame(&ctx, width, &mut controls, &mut side, domain, vec![]);
                click(
                    &ctx,
                    width,
                    &mut controls,
                    &mut side,
                    domain,
                    &output,
                    "Mercury · 20 °C",
                );
                let _ = frame(&ctx, width, &mut controls, &mut side, domain, vec![]);
                if domain {
                    expected.ambient_material = FluidPreset::Mercury.material();
                } else {
                    expected.material = FluidPreset::Mercury.material();
                }
                assert_eq!(format!("{:?}", controls.config), format!("{expected:?}"));
                if domain {
                    for (label, fill) in [
                        ("Filled with emitted fluid", DomainFill::EmittedFluid),
                        ("Empty / vacuum", DomainFill::Vacuum),
                    ] {
                        let output = frame(&ctx, width, &mut controls, &mut side, domain, vec![]);
                        let selected = if controls.config.fill == DomainFill::OtherFluid {
                            "Filled with another fluid"
                        } else {
                            "Filled with emitted fluid"
                        };
                        click(
                            &ctx,
                            width,
                            &mut controls,
                            &mut side,
                            domain,
                            &output,
                            selected,
                        );
                        let output = frame(&ctx, width, &mut controls, &mut side, domain, vec![]);
                        click(
                            &ctx,
                            width,
                            &mut controls,
                            &mut side,
                            domain,
                            &output,
                            label,
                        );
                        let _ = frame(&ctx, width, &mut controls, &mut side, domain, vec![]);
                        expected.fill = fill;
                        assert_eq!(format!("{:?}", controls.config), format!("{expected:?}"));
                    }
                } else {
                    for unit in [
                        DensityUnit::KilogramsPerCubicMetre,
                        DensityUnit::MilligramsPerCubicMillimetre,
                    ] {
                        let output = frame(&ctx, width, &mut controls, &mut side, domain, vec![]);
                        let selected = controls.unit.label();
                        click(
                            &ctx,
                            width,
                            &mut controls,
                            &mut side,
                            domain,
                            &output,
                            selected,
                        );
                        let output = frame(&ctx, width, &mut controls, &mut side, domain, vec![]);
                        click(
                            &ctx,
                            width,
                            &mut controls,
                            &mut side,
                            domain,
                            &output,
                            unit.label(),
                        );
                        let _ = frame(&ctx, width, &mut controls, &mut side, domain, vec![]);
                        assert_eq!(controls.unit.label(), unit.label());
                        assert_eq!(format!("{:?}", controls.config), format!("{expected:?}"));
                    }
                }
            }
        }
    }

    #[test]
    fn ui_material_presets_and_open_menus_fit_narrow_forms() {
        for width in [240.0, 280.0] {
            for domain in [false, true] {
                for preset in FluidPreset::ALL {
                    let ctx = context();
                    let mut controls = MaterialControls {
                        preset,
                        ambient_preset: preset,
                        ..Default::default()
                    };
                    controls.config.material = preset.material();
                    controls.config.material.viscosity += 0.0001; // longest edited caption too
                    controls.config.fill = DomainFill::OtherFluid;
                    controls.config.ambient_material = preset.material();
                    let before = format!("{:?}", controls.config);
                    let mut side = 1.0;
                    let _ = frame(&ctx, width, &mut controls, &mut side, domain, vec![]);
                    let output = frame(&ctx, width, &mut controls, &mut side, domain, vec![]);
                    let label = if domain {
                        "Filled with another fluid".to_owned()
                    } else {
                        format!("{} · edited", preset.label())
                    };
                    click(
                        &ctx,
                        width,
                        &mut controls,
                        &mut side,
                        domain,
                        &output,
                        &label,
                    );
                    let output = frame(&ctx, width, &mut controls, &mut side, domain, vec![]);
                    assert!(
                        text_rect(
                            &output,
                            if domain {
                                "Empty / vacuum"
                            } else {
                                "Water · 20 °C"
                            }
                        )
                        .is_some(),
                        "preset/fill menu actually opened"
                    );
                    for shape in &output.shapes {
                        if let egui::Shape::Text(t) = &shape.shape {
                            assert!(
                                t.pos.x + t.galley.size().x <= width + 0.5,
                                "menu text overflow: {}",
                                t.galley.text()
                            );
                        }
                    }
                    assert_eq!(format!("{:?}", controls.config), before);
                    let _ = frame(
                        &ctx,
                        width,
                        &mut controls,
                        &mut side,
                        domain,
                        key(egui::Key::Escape, Default::default()),
                    );
                    if domain {
                        let output = frame(&ctx, width, &mut controls, &mut side, domain, vec![]);
                        click(
                            &ctx,
                            width,
                            &mut controls,
                            &mut side,
                            domain,
                            &output,
                            preset.label(),
                        );
                        let output = frame(&ctx, width, &mut controls, &mut side, domain, vec![]);
                        let other = if preset == FluidPreset::Water {
                            "Mercury · 20 °C"
                        } else {
                            "Water · 20 °C"
                        };
                        assert!(
                            text_rect(&output, other).is_some(),
                            "ambient menu actually opened"
                        );
                        for shape in &output.shapes {
                            if let egui::Shape::Text(t) = &shape.shape {
                                assert!(
                                    t.pos.x + t.galley.size().x <= width + 0.5,
                                    "ambient menu overflow: {}",
                                    t.galley.text()
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn mass_density_units_use_independent_si_conversion() {
        assert_eq!(DensityUnit::GramsPerMillilitre.factor(), 1000.0);
        assert_eq!(DensityUnit::MilligramsPerCubicMillimetre.factor(), 1000.0);
        assert_eq!(DensityUnit::KilogramsPerCubicMetre.factor(), 1.0);
        assert_eq!(13.546 * DensityUnit::GramsPerMillilitre.factor(), 13546.0);
    }
}
