//! Immutable native entry recipes. Availability remains gated by actual native
//! end-to-end and whole-feature admission; these are not restart adapters.
use crate::{App, Request, run_sets, scene_ui, voxel_material_execution};
use run_sets::voxel_material::{
    ExecutionPolicy, FrozenInput, PressurePolicy,
    run_set::{VoxelRunSetMember, VoxelRunSetRevision, VoxelSetKind},
};
use std::sync::Arc;
/// Normal UI remains unavailable until the canonical native/whole-feature gates.
pub(crate) const AVAILABLE: bool = false;
pub(crate) const UNAVAILABLE: &str =
    "Voxel contents execution awaits actual native end-to-end qualification.";
impl App {
    /// Parse before the existing import/autostart side effects. Qualification is
    /// temporary local debug UI transport; ordinary startup stays unavailable.
    pub(super) fn configure_arguments(
        &mut self,
        arguments: impl IntoIterator<Item = String>,
    ) -> Result<bool, String> {
        let mut args = arguments.into_iter();
        let mut run = false;
        let mut qualification = None;
        let mut collision_option = false;
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--qualify-voxel-material" => {
                    if qualification.is_some() {
                        return Err("Duplicate --qualify-voxel-material".into());
                    }
                    qualification=Some(args.next().filter(|p|!p.is_empty() && !p.starts_with("--")).ok_or("--qualify-voxel-material requires a protected local output directory")?);
                }
                "--collision-mesh" => {
                    collision_option = true;
                    self.collision.path = args
                        .next()
                        .filter(|p| !p.is_empty() && !p.starts_with("--"))
                        .ok_or("--collision-mesh requires a local path")?;
                }
                "--collision-policy" => {
                    collision_option = true;
                    self.collision.solid = match args.next().as_deref() {
                        Some("solid") => true,
                        Some("surface") => false,
                        _ => return Err("--collision-policy requires solid or surface".into()),
                    };
                }
                "--collision-run" => {
                    collision_option = true;
                    run = true;
                    self.steps = 180;
                }
                _ => return Err(format!("Unknown argument: {arg}")),
            }
        }
        if let Some(output) = qualification {
            if !cfg!(debug_assertions) {
                return Err(
                    "Voxel qualification argument is available only in a debug verification build"
                        .into(),
                );
            }
            if collision_option || run || !self.collision.path.is_empty() {
                return Err(
                    "Voxel qualification cannot be combined with collision or autostart options"
                        .into(),
                );
            }
            if output.len() > 4096 {
                return Err("Qualification output directory exceeds 4096 bytes".into());
            }
            self.voxel_qualification = true;
            self.size = 8;
            self.domain_side = 1.0;
            self.numerical_cells_per_metre = None;
            self.steps = 8;
            self.requested_dt = 0.001;
            self.materials.config.gravity = [0.0, -9.80665, 0.0];
            self.cylinder = false;
            self.history_budget_mib = 16;
            self.preview_resolution = crate::history::PreviewResolution::Adaptive;
            self.recording_database_mib = 32;
            self.recording_record_mib = 4;
            self.recording_directory = output;
            self.scene.open = true;
            self.status="Native voxel qualification · create and paint Initial contents; no seed or simulation was started.".into();
        }
        Ok(run)
    }
    pub(super) fn contents_setup_ui(&mut self, ui: &mut crate::egui::Ui) {
        use crate::{egui, history::PreviewResolution, settings};
        ui.heading("Voxel Water/Air");
        ui.small("Incompressible · closed free-slip walls · starts at rest · no surface tension");
        if let Some(editor) = &self.scene.contents.editor {
            let g = editor.document().geometry();
            let n = g.counts();
            let h = g.spacing();
            ui.label(format!("{} × {} × {} cells", n[0], n[1], n[2]));
            ui.small(format!("Spacing {:.9}, {:.9}, {:.9} m", h[0], h[1], h[2]));
        }
        ui.add_enabled_ui(self.worker.is_none() && !self.closing,|ui| {
            settings::quantity(ui,"Maximum step",&mut self.requested_dt,"s",0.0001,0.000001..=1.0);
            ui.horizontal_wrapped(|ui|{ui.label("Intervals");ui.add(egui::DragValue::new(&mut self.steps).range(1..=10000));});
            ui.small("Accepted intervals can be shorter than this maximum.");
            ui.label("Gravity · m/s²");
            for d in 0..3 { settings::quantity(ui,["X","Y","Z"][d],&mut self.materials.config.gravity[d],"m/s²",0.1,-100.0..=100.0); }
            ui.horizontal_wrapped(|ui| {
                if ui.button("Earth").clicked() {self.materials.config.gravity=[0.0,-9.80665,0.0];}
                if ui.button("Zero gravity").clicked() {self.materials.config.gravity=[0.0;3];}
            });
            if self.cylinder && ui.button("Disable emitter for contents").clicked() {self.cylinder=false;}
            ui.collapsing("Recording and preview",|ui| {
                ui.label("Local recording directory");
                ui.add(egui::TextEdit::singleline(&mut self.recording_directory).char_limit(4096).desired_width(ui.available_width()));
                ui.add(egui::DragValue::new(&mut self.recording_database_mib).range(1..=16384).suffix(" MiB database cap"));
                ui.add(egui::DragValue::new(&mut self.recording_record_mib).range(1..=64).suffix(" MiB per publication"));
                ui.add(egui::DragValue::new(&mut self.history_budget_mib).range(16..=96).suffix(" MiB preview budget"));
                let preview = match self.preview_resolution { PreviewResolution::Adaptive => "Document grid".into(), PreviewResolution::CellsPerMetre(n) => format!("{n} cells/m"), PreviewResolution::Edge(n) => format!("{n} cells across") };
                ui.small(format!("Preview: {preview}"));
                if ui.button("Use document preview").clicked() {self.preview_resolution=PreviewResolution::Adaptive;}
                ui.small("Recording includes constructor and every accepted interval. Numerical array policy 128 MiB; retained results also count toward admission.");
            });
        });
        if let Err(error) = self.contents_run_readiness() {
            ui.colored_label(egui::Color32::YELLOW, error);
        }
    }
    fn voxel_request(&self, revision: VoxelRunSetRevision) -> Result<Request, String> {
        revision.validate().map_err(|e| e.to_string())?;
        let spec = &revision.members[0].spec;
        let initial = spec.initial_editor().map_err(|e| e.to_string())?;
        let grid = initial.document().geometry();
        let policy = spec.execution();
        Ok(Request {
            force_fields: Vec::new(),
            collision_mesh: None,
            size: grid.counts()[0] as u64,
            steps: policy.requested_steps,
            tight: false,
            threads: 1,
            methods: vec![rheon::PressureImplementation::JacobiPcgV1],
            emitter: None,
            observe: false,
            history_budget_mib: self.history_budget_mib,
            preview_resolution: self.preview_resolution,
            domain_side: grid.spacing()[0] * grid.counts()[0] as f64,
            requested_dt: policy.requested_dt_s,
            material: None,
            material_color: [45, 125, 215],
            recording_limits: policy.recording.limits(),
            recording_directory: self.recording_directory.clone(),
            saved_set: None,
            configured_set: None,
            prior_run_ids: revision
                .members
                .iter()
                .filter_map(|m| m.attempt.as_ref().map(|a| a.expected_run_id))
                .collect(),
            voxel: Some(voxel_material_execution::VoxelRequest {
                revision: Arc::new(revision),
                retained_payload_bytes: 0,
            }),
            manifest_receipt: None,
            #[cfg(test)]
            progress_gate: None,
            #[cfg(test)]
            failure_gate: None,
        })
    }
    pub(super) fn voxel_execution_available(&self) -> bool {
        AVAILABLE || self.voxel_qualification
    }
    pub(super) fn contents_run_readiness(&self) -> Result<(), String> {
        if !self.voxel_execution_available() {
            return Err(UNAVAILABLE.into());
        }
        self.contents_capture_readiness()
    }
    /// One read-only structural admission shared by displayed readiness and freeze.
    pub(super) fn contents_capture_readiness(&self) -> Result<(), String> {
        if self.closing
            || self.worker.is_some()
            || self.pending_voxel.is_some()
            || self.scene.context != scene_ui::SceneContext::Draft
            || !self.scene.contents_selected()
            || self.scene.editing()
            || self.scene.contents.busy()
        {
            return Err("Select idle Draft Initial contents and finish or cancel editing before freezing a voxel run".into());
        }
        if self.setup_workflow != crate::SetupWorkflow::Preview {
            return Err("Switch to Preview setup; voxel contents do not use the configured Session/provider workflow".into());
        }
        if self.cylinder || self.collision.mesh.is_some() || !self.scene.draft().objects.is_empty()
        {
            return Err("Voxel Water/Air supports no emitter, obstacle mesh or authored forces. Disable the cylinder emitter and clear unsupported draft features before freezing contents".into());
        }
        let editor = self
            .scene
            .contents
            .editor
            .as_ref()
            .ok_or("No initial contents document")?;
        crate::voxel_material_authoring::native_authoring_geometry(editor.document().geometry())?;
        if editor.active() {
            return Err(
                "Finish or cancel the active contents stroke before freezing a voxel run".into(),
            );
        }
        editor.document().validate().map_err(|e| e.to_string())
    }
    /// No publication or numerical construction; a failed freeze preserves all owners.
    pub(super) fn capture_contents_request(&self) -> Result<Request, String> {
        self.contents_capture_readiness()?;
        let editor = self
            .scene
            .contents
            .editor
            .as_ref()
            .ok_or("No initial contents document")?;
        let limits = self.selected_recording_limits()?;
        let spec = FrozenInput::capture(
            editor,
            ExecutionPolicy {
                requested_dt_s: self.requested_dt,
                requested_steps: self.steps,
                gravity_m_s2: self.materials.config.gravity,
                pressure: PressurePolicy {
                    relative_residual: 1e-11,
                    absolute_residual: 1e-12,
                    divergence_limit: 1e-8,
                    max_iterations: 2000,
                },
                numerical_memory_limit_bytes: 128 * 1024 * 1024,
                recording: run_sets::ArchiveBudget {
                    max_database_bytes: limits.max_database_bytes,
                    max_record_bytes: limits.max_record_bytes,
                    max_records: limits.max_records,
                },
                executor: voxel_material_execution::executor(),
            },
        )
        .map_err(|e| format!("Cannot freeze initial contents: {e}"))?;
        let revision = VoxelRunSetRevision {
            schema_version: 3,
            kind: VoxelSetKind::VoxelMaterialSet,
            set_id: format!(
                "contents-{}-revision-{}",
                editor.document().id().0,
                editor.document().revision()
            ),
            parent_sha256: None,
            baseline_member: "voxel".into(),
            members: vec![VoxelRunSetMember {
                member_id: "voxel".into(),
                spec,
                attempt: None,
            }],
        };
        self.voxel_request(revision)
    }
    pub(super) fn start_contents(&mut self) -> Result<(), String> {
        let request = self.capture_contents_request()?;
        self.launch_request(request)
    }
    /// Original recipe alone supplies constructor zero, regardless of whether
    /// an old accepted archive remains available. Never consume its final fields.
    pub(super) fn rerun_voxel(&mut self, original: VoxelRunSetRevision) -> Result<(), String> {
        let request = self.voxel_request(original)?;
        self.launch_request(request)
    }
}
#[cfg(test)]
#[path = "voxel_material_setup_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "voxel_material_native_tests.rs"]
mod native_tests;
