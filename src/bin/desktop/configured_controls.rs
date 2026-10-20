//! Desktop controls for the closed accepted-Session profile. Inspection only
//! reads a trusted-local adapter package; freezing captures a constructor and
//! future intent, and never writes an artifact, starts a child, or advances time.
use super::{
    ai_comparison::adapter::{AdapterPackage, OutputRole, PriorPressure},
    run_sets::{ArchiveBudget, configured::*},
};
use eframe::egui;
use rheon::{
    GridGeometry, LiquidTransportConfig, PressureImplementation, PressureSettings, SessionIdentity,
    SimulationConfig, SimulationSession, VolumeStamp,
    solver_provider::configured::ContextProviderIdentity,
};
use std::path::Path;

const MAX_CELLS: usize = 65_536;
const MAX_INTERVALS: usize = 10_000;
const TEXT_LIMIT: usize = 1000;
const DEFAULT_WORKSPACE_BYTES: usize = 64 * 1024 * 1024;

/// Exact caller controls, including separate constructor and per-call volume
/// limits. The temporary constructor uses run id zero only to capture facts; it
/// is not a run-set attempt. The caller assigns actual attempt identities later.
pub(crate) struct ConfiguredLaunchInputs {
    pub set_id: String,
    pub grid: GridGeometry,
    pub pressure_implementation: PressureImplementation,
    pub pressure: PressureSettings,
    pub actual_divergence_limit: f64,
    pub max_courant: f64,
    pub simulation_memory_limit_bytes: usize,
    pub boundary_workspace_limit_bytes: usize,
    pub volume_stamp: VolumeStamp,
    pub carrier_id: u64,
    pub requested_dt_s: f64,
    pub requested_steps: usize,
    pub volume: VolumeControls,
    pub recording: ArchiveBudget,
    pub executor: ExecutorDeclaration,
}

struct Inspection {
    entered_path: String,
    package: AdapterPackage,
    deployment: DeploymentLocator,
    pin: ConfiguredProviderPin,
    description: String,
}

pub(crate) struct ConfiguredControls {
    paired: bool,
    density: f64,
    newtonian: bool,
    dynamic_viscosity: f64,
    force_enabled: bool,
    force: [f64; 3],
    force_units: ForceUnitsSpec,
    region_enabled: bool,
    region_lower: [f64; 3],
    region_upper: [f64; 3],
    requested_native_prefix_steps: usize,
    provider_workspace_limit_bytes: usize,
    viscosity_workspace_limit_bytes: usize,
    package_path: String,
    runtime_declaration: String,
    model_source_revision: String,
    inspection: Option<Inspection>,
    feedback: String,
}
impl Default for ConfiguredControls {
    fn default() -> Self {
        Self {
            paired: false,
            density: 1000.0,
            newtonian: false,
            dynamic_viscosity: 0.001,
            force_enabled: true,
            force: [0.125, 0.0, 0.0],
            force_units: ForceUnitsSpec::Acceleration,
            region_enabled: false,
            region_lower: [0.0; 3],
            region_upper: [1.0; 3],
            requested_native_prefix_steps: 1,
            provider_workspace_limit_bytes: DEFAULT_WORKSPACE_BYTES,
            viscosity_workspace_limit_bytes: DEFAULT_WORKSPACE_BYTES,
            package_path: String::new(),
            runtime_declaration: String::new(),
            model_source_revision: String::new(),
            inspection: None,
            feedback: String::new(),
        }
    }
}

impl ConfiguredControls {
    #[cfg(test)]
    pub(crate) fn density(&self) -> f64 {
        self.density
    }

    /// Read and pin package bytes without launching its interpreter. Relative
    /// user input is resolved now into an owned absolute deployment locator;
    /// archives need not share the package's directory. Runtime startup must
    /// subsequently revalidate these pins before executing mutable local code.
    pub(crate) fn inspect_package(&mut self) -> Result<(), String> {
        self.inspection = None;
        self.feedback.clear();
        let result = self.read_inspection();
        match result {
            Ok(inspection) => {
                self.feedback = "Package inspected; code has not been run.".into();
                self.inspection = Some(inspection);
                Ok(())
            }
            Err(error) => {
                self.feedback = error.clone();
                Err(error)
            }
        }
    }

    fn read_inspection(&self) -> Result<Inspection, String> {
        bounded_text(&self.package_path, "package path")?;
        bounded_text(&self.runtime_declaration, "runtime declaration")?;
        bounded_text(&self.model_source_revision, "model source declaration")?;
        let path = std::fs::canonicalize(Path::new(&self.package_path))
            .map_err(|error| format!("Cannot resolve package manifest: {error}"))?;
        let deployment = DeploymentLocator::Absolute(
            path.to_str()
                .ok_or("Package path must be UTF-8")?
                .to_owned(),
        );
        deployment.validate().map_err(|error| error.to_string())?;
        let package = AdapterPackage::load(&path).map_err(|error| error.to_string())?;
        let pin = ConfiguredProviderPin::capture(&ContextProviderIdentity {
            id: package.id().into(),
            version: package.version().into(),
            manifest_sha256: decode_sha256(package.manifest_sha256())?,
            // Adapter v1 does not declare checkpoint or training provenance.
            checkpoint_sha256: None,
            training_run_id: None,
            runtime: self.runtime_declaration.clone(),
            source_revision: self.model_source_revision.clone(),
        })
        .map_err(|error| error.to_string())?;
        let support = package.support();
        let context = package.context();
        let role = match package.output_role() {
            OutputRole::FullPressure => "full pressure",
            OutputRole::PressureCorrection => "pressure correction",
        };
        let prior = match context.prior_pressure {
            PriorPressure::Required => "required",
            PriorPressure::Optional => "optional",
        };
        let description = format!(
            "{} / {}\nManifest: {}\nManifest SHA-256: {}\nDeclared physical support: density {} kg/m³; dynamic viscosity {} Pa·s; dt {} s.\nGrid {:?} through {:?}, at most {} cells; cubic spacing {} through {} m; zero origin.\nMinimum native prefix: {}; accepted prior: {}. Output: {}, f64, x-fast, cell-zero gauge.\nHashes identify local bytes; declarations and support do not establish accuracy or authorship.",
            package.id(),
            package.version(),
            path.display(),
            package.manifest_sha256(),
            support.density_kg_m3,
            support.dynamic_viscosity_pa_s,
            support.dt_s,
            support.counts_min,
            support.counts_max,
            support.max_cells,
            support.spacing_min_m,
            support.spacing_max_m,
            context.minimum_native_steps,
            prior,
            role,
        );
        Ok(Inspection {
            entered_path: self.package_path.clone(),
            package,
            deployment,
            pin,
            description,
        })
    }

    fn current_inspection(&self) -> Result<&Inspection, String> {
        let inspection = self
            .inspection
            .as_ref()
            .ok_or("Inspect a package before freezing a paired run")?;
        if inspection.entered_path != self.package_path
            || inspection.pin.runtime != self.runtime_declaration
            || inspection.pin.source_revision != self.model_source_revision
        {
            return Err("Package path or provenance changed; inspect the package again".into());
        }
        Ok(inspection)
    }

    /// Freeze exact future controls from a real, capture-only constructor. No
    /// package is reloaded here: changes after inspection are detected by the
    /// runtime's later pinned startup, after the caller has saved this intent.
    pub(crate) fn freeze(
        &self,
        inputs: ConfiguredLaunchInputs,
    ) -> Result<ConfiguredRunSetRevision, String> {
        inputs.executor.validate().map_err(|e| e.to_string())?;
        bounded_text(&inputs.set_id, "run-set identity")?;
        // Bound the full-fraction allocation before constructing its owner.
        if inputs.grid.cell_len() > MAX_CELLS
            || inputs.requested_steps == 0
            || inputs.requested_steps > MAX_INTERVALS
        {
            return Err(
                "Closed configured profile supports 1–65536 cells and 1–10000 intervals".into(),
            );
        }
        let viscosity = if self.newtonian {
            ViscosityIntent::NewtonianConstant {
                dynamic_viscosity_pa_s: self.dynamic_viscosity,
            }
        } else {
            ViscosityIntent::Disabled
        };
        let configured_pressure = if self.paired {
            let inspection = self.current_inspection()?;
            let minimum = inspection.package.context().minimum_native_steps;
            if self.requested_native_prefix_steps < minimum
                || self.requested_native_prefix_steps >= inputs.requested_steps
            {
                return Err("Native prefix must satisfy the package minimum and leave a configured interval".into());
            }
            // This is an inert check of captured support and requested controls.
            // Actual per-request support remains the runtime bridge's authority;
            // a shortened actual dt can legitimately decline and fall back.
            inspection
                .package
                .validate_support(
                    &inputs.grid,
                    self.density,
                    if self.newtonian {
                        self.dynamic_viscosity
                    } else {
                        0.0
                    },
                    inputs.requested_dt_s,
                    self.requested_native_prefix_steps,
                    self.requested_native_prefix_steps > 0,
                )
                .map_err(|error| error.to_string())?;
            Some(PressureIntent::NativePrefixThenConfigured {
                provider: Box::new(inspection.pin.clone()),
                deployment: inspection.deployment.clone(),
                requested_native_prefix_steps: self.requested_native_prefix_steps,
                minimum_native_prefix_steps: minimum,
            })
        } else {
            None
        };
        let forces = if self.force_enabled {
            vec![ForceSpec {
                value: self.force,
                units: self.force_units,
                region: self.region_enabled.then_some(ForceRegionSpec {
                    lower_m: self.region_lower,
                    upper_m: self.region_upper,
                }),
            }]
        } else {
            Vec::new()
        };
        let future = ConfiguredFutureIntent {
            requested_dt_s: inputs.requested_dt_s,
            requested_steps: inputs.requested_steps,
            forces,
            volume: inputs.volume,
            viscosity,
            pressure: PressureIntent::NativeOnly,
            // This limit applies separately to the core pressure workspace and
            // CPU bridge scratch; it is not a pooled allocation or RSS bound.
            provider_workspace_limit_bytes: self.provider_workspace_limit_bytes,
            viscosity_workspace_limit_bytes: self
                .newtonian
                .then_some(self.viscosity_workspace_limit_bytes),
            recording: inputs.recording,
            capture_constructor: true,
            executor: inputs.executor.clone(),
        };
        let session = SimulationSession::new(
            SessionIdentity {
                run_id: 0,
                source_revision: inputs.executor.source_revision,
            },
            inputs.grid.clone(),
            LiquidTransportConfig {
                carrier: SimulationConfig {
                    density: self.density,
                    memory_limit: inputs.simulation_memory_limit_bytes,
                    pressure: inputs.pressure,
                    actual_divergence_limit: inputs.actual_divergence_limit,
                    max_courant: inputs.max_courant,
                },
                represented_density: self.density,
                volume_stamp: inputs.volume_stamp,
                carrier_id: inputs.carrier_id,
            },
            inputs.pressure_implementation,
            vec![1.0; inputs.grid.cell_len()],
            inputs.boundary_workspace_limit_bytes,
            inputs.requested_steps,
        )
        .map_err(|error| error.to_string())?;
        let snapshot = session.snapshot();
        let native = FrozenConfiguredSpec::capture_constructor(&snapshot, future.clone())
            .map_err(|error| error.to_string())?;
        let mut members = vec![ConfiguredRunSetMember {
            member_id: "native".into(),
            spec: native,
            differences_from_baseline: Vec::new(),
            attempt: None,
        }];
        if let Some(pressure) = configured_pressure {
            let configured = FrozenConfiguredSpec::capture_constructor(
                &snapshot,
                ConfiguredFutureIntent { pressure, ..future },
            )
            .map_err(|error| error.to_string())?;
            let differences_from_baseline =
                ConfiguredSpecDifference::between(&members[0].spec, &configured)
                    .map_err(|error| error.to_string())?;
            members.push(ConfiguredRunSetMember {
                member_id: "configured".into(),
                spec: configured,
                differences_from_baseline,
                attempt: None,
            });
        }
        let set = ConfiguredRunSetRevision {
            schema_version: CONFIGURED_VERSION,
            kind: ConfiguredSetKind::ConfiguredAcceptedSessionSet,
            set_id: inputs.set_id,
            parent_sha256: None,
            baseline_member: "native".into(),
            members,
        };
        set.validate().map_err(|error| error.to_string())?;
        Ok(set)
    }

    pub(crate) fn show(&mut self, ui: &mut egui::Ui, enabled: bool) {
        ui.add_enabled_ui(enabled, |ui| {
            ui.label("Sealed full liquid · starts at rest · no inlet");
            ui.small("Constant forcing only; no appearance, emitter, phase source, or model accuracy claim.");
            ui.checkbox(&mut self.paired, "Compare with CPU adapter");
            ui.horizontal(|ui| {
                ui.label("Density (kg/m³)");
                ui.add(egui::DragValue::new(&mut self.density).speed(1.0).range(1e-9..=1e9));
            });
            ui.checkbox(&mut self.newtonian, "Apply constant Newtonian viscosity");
            if self.newtonian {
                ui.horizontal(|ui| {
                    ui.label("Dynamic viscosity (Pa·s)");
                    ui.add(egui::DragValue::new(&mut self.dynamic_viscosity).speed(0.001).range(0.0..=1e5));
                });
                ui.small("Zero still applies the viscosity stage; disabling omits that stage.");
            }
            ui.checkbox(&mut self.force_enabled, "Constant body force");
            if self.force_enabled {
                egui::ComboBox::from_id_salt("configured_force_units")
                    .selected_text(match self.force_units {
                        ForceUnitsSpec::Acceleration => "Acceleration (m/s²)",
                        ForceUnitsSpec::ForceDensity => "Force density (N/m³)",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.force_units, ForceUnitsSpec::Acceleration, "Acceleration (m/s²)");
                        ui.selectable_value(&mut self.force_units, ForceUnitsSpec::ForceDensity, "Force density (N/m³)");
                    });
                vector_control(ui, "Force x / y / z", &mut self.force);
                ui.checkbox(&mut self.region_enabled, "Limit force to a box inside the domain");
                if self.region_enabled {
                    vector_control(ui, "Box lower (m)", &mut self.region_lower);
                    vector_control(ui, "Box upper (m)", &mut self.region_upper);
                }
            }
            ui.collapsing("Workspace limits", |ui| {
                byte_control(ui, "Pressure workspace / CPU scratch, each (bytes)", &mut self.provider_workspace_limit_bytes);
                if self.newtonian {
                    byte_control(ui, "Viscosity workspace (bytes)", &mut self.viscosity_workspace_limit_bytes);
                }
                ui.small("Each owner has its own limit; these are not pooled memory or process RSS limits.");
            });
            if self.paired {
                self.package_controls(ui);
            }
            if !self.feedback.is_empty() {
                ui.label(&self.feedback);
            }
        });
    }

    /// Returns intent only. The caller processes current form edits before
    /// freezing and saving immutable intent through the existing launch path.
    pub(crate) fn launch_button(&self, ui: &mut egui::Ui, enabled: bool) -> bool {
        let ready = !self.paired || self.current_inspection().is_ok();
        let label = if self.paired {
            "Save & run pair"
        } else {
            "Save & run native"
        };
        let launch = ui
            .add_enabled(
                enabled && ready,
                egui::Button::new(label).min_size(egui::vec2(ui.available_width(), 32.0)),
            )
            .clicked();
        ui.small("Saves immutable intent before running.");
        if !ready {
            ui.small("Inspect the adapter package before running.");
        }
        launch
    }

    fn package_controls(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.label("Trusted-local adapter manifest");
        ui.small("Relative paths resolve from the application working directory at inspection.");
        let mut changed = ui
            .add(
                egui::TextEdit::singleline(&mut self.package_path)
                    .char_limit(TEXT_LIMIT)
                    .hint_text("Manifest path"),
            )
            .changed();
        ui.label("Caller-declared runtime provenance");
        changed |= ui
            .add(
                egui::TextEdit::singleline(&mut self.runtime_declaration)
                    .char_limit(TEXT_LIMIT)
                    .hint_text("Runtime / interpreter declaration"),
            )
            .changed();
        ui.label("Caller-declared model source revision");
        changed |= ui
            .add(
                egui::TextEdit::singleline(&mut self.model_source_revision)
                    .char_limit(TEXT_LIMIT)
                    .hint_text("Model source revision"),
            )
            .changed();
        if changed {
            self.inspection = None;
            self.feedback = "Path or provenance changed; inspect the package again.".into();
        }
        ui.small("These declarations describe the provider; host executor provenance is recorded separately.");
        if ui.button("Inspect package (read only)").clicked() {
            let _ = self.inspect_package();
        }
        if let Some(inspection) = &self.inspection {
            ui.label(&inspection.description);
        }
        ui.horizontal(|ui| {
            ui.label("Requested native prefix (intervals)");
            ui.add(
                egui::DragValue::new(&mut self.requested_native_prefix_steps)
                    .range(0..=MAX_INTERVALS - 1),
            );
        });
    }
}

fn vector_control(ui: &mut egui::Ui, label: &str, values: &mut [f64; 3]) {
    ui.horizontal(|ui| {
        ui.label(label);
        for value in values {
            ui.add(egui::DragValue::new(value).speed(0.01));
        }
    });
}
fn byte_control(ui: &mut egui::Ui, label: &str, bytes: &mut usize) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(
            egui::DragValue::new(bytes)
                .speed(1024.0 * 1024.0)
                .range(1..=usize::MAX),
        );
    });
}
fn bounded_text(value: &str, name: &str) -> Result<(), String> {
    if value.is_empty() || value.len() > TEXT_LIMIT || value.chars().any(char::is_control) {
        Err(format!(
            "{name} must be nonempty, at most {TEXT_LIMIT} bytes, and contain no control characters"
        ))
    } else {
        Ok(())
    }
}
fn decode_sha256(value: &str) -> Result<[u8; 32], String> {
    if value.len() != 64 {
        return Err("Invalid package manifest SHA-256".into());
    }
    let mut bytes = [0; 32];
    for (output, pair) in bytes.iter_mut().zip(value.as_bytes().chunks_exact(2)) {
        fn nibble(value: u8) -> Option<u8> {
            match value {
                b'0'..=b'9' => Some(value - b'0'),
                b'a'..=b'f' => Some(value - b'a' + 10),
                _ => None,
            }
        }
        *output = nibble(pair[0])
            .zip(nibble(pair[1]))
            .map(|(high, low)| (high << 4) | low)
            .ok_or("Invalid package manifest SHA-256")?;
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run_sets::NativePressureMethod;
    use rheon::SnapshotStage;
    use serde_json::json;
    use sha2::{Digest, Sha256};
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
    };

    const CAP: usize = 1024 * 1024;
    fn inputs() -> ConfiguredLaunchInputs {
        ConfiguredLaunchInputs {
            set_id: "configured-controls-fixture".into(),
            grid: GridGeometry::new([2, 1, 1], [0.25; 3], [0.0; 3]).unwrap(),
            pressure_implementation: PressureImplementation::JacobiPcgV1,
            pressure: PressureSettings {
                relative_residual: 1e-12,
                absolute_residual: 1e-14,
                divergence_limit: 1e-9,
                max_iterations: 257,
            },
            actual_divergence_limit: 1e-6,
            max_courant: 0.75,
            simulation_memory_limit_bytes: CAP,
            boundary_workspace_limit_bytes: 2 * CAP,
            volume_stamp: VolumeStamp { id: 17, version: 3 },
            carrier_id: 18,
            requested_dt_s: 1.0 / 16.0,
            requested_steps: 3,
            volume: VolumeControls {
                max_outward_courant: 0.5,
                actual_divergence_per_s: 5e-7,
            },
            recording: ArchiveBudget {
                max_database_bytes: 8 * 1024 * 1024,
                max_record_bytes: CAP,
                max_records: 4,
            },
            executor: ExecutorDeclaration {
                id: "controls-test-host".into(),
                source_revision: "actual-host-source".into(),
                build_provenance: Some("declared-test-build".into()),
            },
        }
    }
    fn controls() -> ConfiguredControls {
        ConfiguredControls {
            density: 2.0,
            force_units: ForceUnitsSpec::ForceDensity,
            region_enabled: true,
            region_upper: [0.5, 0.25, 0.25],
            provider_workspace_limit_bytes: 3 * CAP,
            viscosity_workspace_limit_bytes: 4 * CAP,
            ..ConfiguredControls::default()
        }
    }

    struct Package(PathBuf);
    impl Package {
        fn new() -> Self {
            static SERIAL: AtomicUsize = AtomicUsize::new(0);
            let root = std::env::temp_dir().join(format!(
                "rheon-controls-{}-{}",
                std::process::id(),
                SERIAL.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&root).unwrap();
            let executable = root.join("never-run-interpreter");
            let marker_guard = format!(
                "#!/bin/sh\ntouch '{}'\nexit 1\n",
                root.join("executed").display()
            );
            std::fs::write(&executable, &marker_guard).unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700))
                    .unwrap();
            }
            let source = b"raise RuntimeError('inert inspection fixture; never execute')\n";
            std::fs::write(root.join("adapter.py"), source).unwrap();
            let manifest = json!({
                "schema": "rheon-pressure-adapter-v1", "stage": "sealed-mac-pressure-context-v1",
                "id": "inert-controls-fixture", "version": "1",
                "python": {"executable": executable, "sha256": format!("{:x}", Sha256::digest(marker_guard.as_bytes())), "bytes": marker_guard.len()},
                "entrypoint": "adapter.py", "files": {"adapter.py": {"sha256": format!("{:x}", Sha256::digest(source)), "bytes": source.len()}},
                "support": {"density_kg_m3": 2.0, "dynamic_viscosity_pa_s": 1.0/128.0, "dt_s": 1.0/16.0,
                    "counts_min": [1,1,1], "counts_max": [4,4,4], "max_cells": 64,
                    "spacing_min_m": 0.25, "spacing_max_m": 0.25, "origin": "zero", "cells": "cubic"},
                "context": {"minimum_native_steps": 1, "prior_pressure": "required"},
                "inputs": {"physical_rhs": "rhs", "accepted_prior": "previous_pressure"},
                "output": {"role": "pressure_correction", "dtype": "f64", "layout": "x_fast", "gauge": "cell_zero"}
            });
            std::fs::write(
                root.join("manifest.json"),
                serde_json::to_vec(&manifest).unwrap(),
            )
            .unwrap();
            Self(root)
        }
        fn inspect(&self, controls: &mut ConfiguredControls) {
            controls.paired = true;
            controls.newtonian = true;
            controls.dynamic_viscosity = 1.0 / 128.0;
            controls.package_path = self.0.join("manifest.json").to_str().unwrap().into();
            controls.runtime_declaration = "declared inert fixture executable".into();
            controls.model_source_revision = "independent-package-source".into();
            controls.inspect_package().unwrap();
        }
    }
    impl Drop for Package {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn actual_constructor_capture_preserves_raw_controls_for_both_native_methods() {
        for method in [
            PressureImplementation::JacobiPcgV1,
            PressureImplementation::SymmetricGaussSeidelPcgV1,
        ] {
            let form = controls();
            let mut launch = inputs();
            launch.pressure_implementation = method;
            let set = form.freeze(launch).unwrap();
            assert_eq!(set.members.len(), 1);
            let spec = &set.members[0].spec;
            let session = spec
                .create_session(SessionIdentity {
                    run_id: 42,
                    source_revision: "actual-host-source".into(),
                })
                .unwrap();
            let snapshot = session.snapshot();
            spec.validate_constructor(&snapshot).unwrap();
            let configuration = snapshot.configuration();
            let carrier = configuration.transport.carrier;
            assert_eq!(snapshot.grid().counts(), [2, 1, 1]);
            assert_eq!(snapshot.grid().spacing(), [0.25; 3]);
            assert_eq!(snapshot.grid().origin(), [0.0; 3]);
            assert_eq!(configuration.implementation, method);
            assert_eq!(configuration.boundary_workspace_limit, 2 * CAP);
            assert_eq!(configuration.max_steps, 3);
            assert_eq!(carrier.memory_limit, CAP);
            assert_eq!(carrier.density.to_bits(), form.density().to_bits());
            assert_eq!(
                configuration.transport.represented_density.to_bits(),
                form.density().to_bits()
            );
            assert_eq!(
                carrier.pressure.relative_residual.to_bits(),
                1e-12f64.to_bits()
            );
            assert_eq!(
                carrier.pressure.absolute_residual.to_bits(),
                1e-14f64.to_bits()
            );
            assert_eq!(
                carrier.pressure.divergence_limit.to_bits(),
                1e-9f64.to_bits()
            );
            assert_eq!(carrier.pressure.max_iterations, 257);
            assert_eq!(carrier.actual_divergence_limit.to_bits(), 1e-6f64.to_bits());
            assert_eq!(carrier.max_courant.to_bits(), 0.75f64.to_bits());
            assert_eq!(
                configuration.transport.volume_stamp,
                VolumeStamp { id: 17, version: 3 }
            );
            assert_eq!(configuration.transport.carrier_id, 18);
            assert_eq!(
                spec.constructor.pressure_method,
                match method {
                    PressureImplementation::JacobiPcgV1 => NativePressureMethod::JacobiPcgV1,
                    PressureImplementation::SymmetricGaussSeidelPcgV1 => {
                        NativePressureMethod::SymmetricGaussSeidelPcgV1
                    }
                }
            );
            assert_eq!(snapshot.stage(), SnapshotStage::Constructor);
            assert_eq!(snapshot.frame(), 0);
            assert_eq!(snapshot.state().carrier.time.to_bits(), 0.0f64.to_bits());
            assert_eq!(snapshot.state().carrier.generation, 0);
            assert!(
                snapshot
                    .state()
                    .liquid
                    .fraction
                    .iter()
                    .all(|v| v.to_bits() == 1.0f64.to_bits())
            );
            assert!(
                snapshot
                    .state()
                    .pressure
                    .iter()
                    .all(|v| v.to_bits() == 0.0f64.to_bits())
            );
            assert!(snapshot.accepted_interval_start().is_none());
            assert!(snapshot.configured_provider_report().is_none());
            assert!(set.members[0].attempt.is_none());
            assert!(spec.future.capture_constructor);
            assert_eq!(spec.future.volume, inputs().volume);
            assert_ne!(
                spec.future.volume.actual_divergence_per_s.to_bits(),
                carrier.actual_divergence_limit.to_bits()
            );
            assert_eq!(spec.future.forces[0].units, ForceUnitsSpec::ForceDensity);
            assert_eq!(
                spec.future.forces[0].region.as_ref().unwrap().upper_m,
                [0.5, 0.25, 0.25]
            );
            assert_eq!(spec.future.provider_workspace_limit_bytes, 3 * CAP);
            assert_eq!(spec.future.recording, inputs().recording);
        }
    }

    #[test]
    fn disabled_and_applied_zero_viscosity_freeze_distinct_stage_and_workspace_intent() {
        let mut form = controls();
        let disabled = form.freeze(inputs()).unwrap();
        form.newtonian = true;
        form.dynamic_viscosity = 0.0;
        let applied_zero = form.freeze(inputs()).unwrap();
        let a = &disabled.members[0].spec;
        let b = &applied_zero.members[0].spec;
        assert_eq!(a.constructor, b.constructor);
        assert_eq!(a.future.viscosity, ViscosityIntent::Disabled);
        assert_eq!(a.future.viscosity_workspace_limit_bytes, None);
        assert_eq!(
            b.future.viscosity,
            ViscosityIntent::NewtonianConstant {
                dynamic_viscosity_pa_s: 0.0
            }
        );
        assert_eq!(b.future.viscosity_workspace_limit_bytes, Some(4 * CAP));
        assert_eq!(
            ConfiguredSpecDifference::between(a, b).unwrap(),
            [
                ConfiguredSpecDifference::Viscosity,
                ConfiguredSpecDifference::Workspaces
            ]
        );
        assert_ne!(a.digest().unwrap(), b.digest().unwrap());
    }

    #[test]
    fn inert_inspection_pins_pair_and_freeze_uses_captured_bytes_without_reloading() {
        let package = Package::new();
        let mut form = controls();
        let manifest = package.0.join("manifest.json");
        let expected_digest: [u8; 32] = Sha256::digest(std::fs::read(&manifest).unwrap()).into();
        package.inspect(&mut form);
        assert!(!package.0.join("executed").exists());
        let description = &form.inspection.as_ref().unwrap().description;
        assert!(description.contains("pressure correction"));
        assert!(description.contains("Minimum native prefix: 1"));
        assert!(!description.contains("physical_rhs"));
        // Freezing after removal is deliberate: launch will revalidate mutable
        // files against this retained identity after intent publication.
        std::fs::remove_file(&manifest).unwrap();
        let set = form.freeze(inputs()).unwrap();
        assert_eq!(set.members.len(), 2);
        assert_eq!(
            set.members[0].spec.future.pressure,
            PressureIntent::NativeOnly
        );
        let PressureIntent::NativePrefixThenConfigured {
            provider,
            deployment,
            requested_native_prefix_steps,
            minimum_native_prefix_steps,
        } = &set.members[1].spec.future.pressure
        else {
            panic!("configured member missing")
        };
        assert_eq!(provider.manifest_sha256, expected_digest);
        assert_eq!(provider.id, "inert-controls-fixture");
        assert_eq!(provider.version, "1");
        assert_eq!(provider.checkpoint_sha256, None);
        assert_eq!(provider.training_run_id, None);
        assert_eq!(provider.source_revision, "independent-package-source");
        assert_ne!(
            provider.source_revision,
            set.members[1].spec.future.executor.source_revision
        );
        assert!(
            matches!(deployment, DeploymentLocator::Absolute(path) if Path::new(path).is_absolute())
        );
        assert_eq!(
            (*requested_native_prefix_steps, *minimum_native_prefix_steps),
            (1, 1)
        );
        assert_eq!(
            set.members[1].differences_from_baseline,
            [ConfiguredSpecDifference::PressureSelection]
        );
        assert!(
            set.members
                .iter()
                .all(|member| member.attempt.is_none() && member.spec.future.capture_constructor)
        );
        assert!(!package.0.join("executed").exists());
        assert!(form.inspect_package().is_err());
        assert!(form.inspection.is_none());
        assert!(form.freeze(inputs()).is_err());
    }

    #[test]
    fn changed_declarations_and_path_require_explicit_recapture_and_support_is_checked_inertly() {
        let package = Package::new();
        let mut form = controls();
        package.inspect(&mut form);
        form.runtime_declaration.push_str(" changed");
        assert!(form.freeze(inputs()).unwrap_err().contains("inspect"));
        form.inspect_package().unwrap();
        assert!(form.freeze(inputs()).is_ok());
        form.model_source_revision.push_str(" changed");
        assert!(form.freeze(inputs()).unwrap_err().contains("inspect"));
        form.inspect_package().unwrap();
        assert!(form.freeze(inputs()).is_ok());
        form.package_path.push_str(".other");
        assert!(form.freeze(inputs()).unwrap_err().contains("inspect"));
        package.inspect(&mut form);
        form.requested_native_prefix_steps = 0;
        assert!(form.freeze(inputs()).is_err());
        form.requested_native_prefix_steps = 3;
        assert!(form.freeze(inputs()).is_err());
        form.requested_native_prefix_steps = 1;
        form.dynamic_viscosity = 0.0;
        assert!(form.freeze(inputs()).is_err());
        form.dynamic_viscosity = 1.0 / 128.0;
        let mut changed_dt = inputs();
        changed_dt.requested_dt_s *= 0.5;
        assert!(form.freeze(changed_dt).is_err());
        assert!(!package.0.join("executed").exists());
    }

    #[test]
    fn invalid_physics_regions_limits_and_raw_constructor_controls_fail_closed() {
        let mut form = controls();
        form.region_upper[0] = 0.75;
        assert!(form.freeze(inputs()).is_err());
        form.region_upper[0] = 0.5;
        form.density = f64::NAN;
        assert!(form.freeze(inputs()).is_err());
        form.density = 2.0;
        form.provider_workspace_limit_bytes = 0;
        assert!(form.freeze(inputs()).is_err());
        form.provider_workspace_limit_bytes = CAP;
        form.newtonian = true;
        form.dynamic_viscosity = -1.0;
        assert!(form.freeze(inputs()).is_err());
        form.dynamic_viscosity = 0.0;
        form.viscosity_workspace_limit_bytes = 0;
        assert!(form.freeze(inputs()).is_err());
        form.viscosity_workspace_limit_bytes = CAP;
        let mut launch = inputs();
        launch.pressure.max_iterations = 0;
        assert!(form.freeze(launch).is_err());
        let mut launch = inputs();
        launch.volume.max_outward_courant = 1.5;
        assert!(form.freeze(launch).is_err());
        let mut launch = inputs();
        launch.requested_steps = MAX_INTERVALS + 1;
        assert!(form.freeze(launch).is_err());
        let mut launch = inputs();
        launch.grid = GridGeometry::new([65_537, 1, 1], [0.25; 3], [0.0; 3]).unwrap();
        assert!(form.freeze(launch).is_err());
    }
}
