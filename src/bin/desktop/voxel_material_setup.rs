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
    /// No publication or numerical construction; a failed freeze preserves all owners.
    pub(super) fn capture_contents_request(&self) -> Result<Request, String> {
        if self.worker.is_some()
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
