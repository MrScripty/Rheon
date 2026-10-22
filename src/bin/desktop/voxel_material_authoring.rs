//! Native draft file actions. Durable bytes stay owned by the existing versioned
//! draft module; Scene owns identity, App adopts presentation geometry atomically.
use crate::{
    App,
    run_sets::{RunSetError, voxel_material::DraftRevision},
    scene_ui::SceneContext,
};
use rheon::GridGeometry;
use std::path::PathBuf;
#[derive(Debug)]
pub(crate) enum DraftFileAction {
    Save(PathBuf),
    Load(PathBuf),
}
fn native_authoring_geometry(grid: &GridGeometry) -> Result<(u64, f64), String> {
    let n = grid.counts();
    let h = grid.spacing();
    let side = h[0] * n[0] as f64;
    if !(4..=64).contains(&n[0])
        || n != [n[0]; 3]
        || h.map(f64::to_bits) != [h[0].to_bits(); 3]
        || grid.origin().iter().any(|&v| v != 0.0)
        || !(0.001..=100.0).contains(&side)
        || n[0] as f64 / side > f64::from(rheon::MAX_CELLS_PER_METRE).next_up()
    {
        return Err("Native draft editing requires a 4–64 cubic equal-spacing zero-origin domain, side 0.001–100 m and at most 1000 cells/m. Original file is retained; no resampling was performed".into());
    }
    Ok((n[0] as u64, side))
}
impl App {
    pub(super) fn draft_file_action(&mut self, action: DraftFileAction) -> Result<String, String> {
        if self.worker.is_some()
            || self.scene.context != SceneContext::Draft
            || self.scene.editing()
            || self.scene.contents.busy()
            || self
                .scene
                .contents
                .editor
                .as_ref()
                .is_some_and(|e| e.active())
        {
            return Err("Finish or cancel editing and stop the simulation before saving/loading initial contents".into());
        }
        let path = match &action {
            DraftFileAction::Save(p) | DraftFileAction::Load(p) => p,
        };
        if path.as_os_str().is_empty() || path.as_os_str().len() > 4096 {
            return Err("Choose a nonempty local draft path of at most 4096 bytes".into());
        }
        match action {
            DraftFileAction::Save(path) => {
                if let Some(target) = &self.scene.contents.uncertain_save {
                    return Err(format!(
                        "Save publication remains uncertain at {}; further session saves are paused. Reconcile this exact file read-only before another publication",
                        target.display()
                    ));
                }
                let e = self
                    .scene
                    .contents
                    .editor
                    .as_ref()
                    .ok_or("No initial contents draft to save")?;
                let revision = DraftRevision::capture(e).map_err(|e| e.to_string())?;
                if let Err(error) = revision.publish(&path) {
                    if matches!(error, RunSetError::PublicationUncertain(_)) {
                        self.scene.contents.uncertain_save = Some(path.clone());
                    }
                    return Err(format!(
                        "Save draft {}: {error}; existing files are never replaced",
                        path.display()
                    ));
                }
                Ok(format!(
                    "Saved draft {}. Use a new file name for another revision",
                    path.display()
                ))
            }
            DraftFileAction::Load(path) => {
                let loaded = DraftRevision::open(&path)
                    .and_then(|r| r.restore())
                    .map_err(|e| {
                        format!(
                            "Load draft {}: {e}; previous draft retained",
                            path.display()
                        )
                    })?;
                let (size, side) = native_authoring_geometry(loaded.document().geometry())?;
                let id = loaded.document().id();
                if self.running_request.as_ref().is_some_and(|r| {
                    r.force_fields
                        .iter()
                        .flatten()
                        .any(|f| f.scene().objects.iter().any(|o| o.id == id))
                }) {
                    return Err("Loaded contents ID conflicts with a retained run's force; previous draft retained".into());
                }
                self.scene.adopt_loaded_contents(loaded)?;
                // No fallible operation after Scene admission. Retained Request,
                // history, rows, textures and selection stay owned separately.
                self.size = size;
                self.domain_side = side;
                self.numerical_cells_per_metre = None;
                Ok(format!(
                    "Loaded draft {}; exact cells/domain restored, contents undo history reset",
                    path.display()
                ))
            }
        }
    }
    pub(super) fn service_draft_file_action(&mut self, ctx: &eframe::egui::Context) {
        if let Some(action) = self.scene.contents.draft_action.take() {
            let loading = matches!(&action, DraftFileAction::Load(_));
            let result = self.draft_file_action(action);
            if loading && result.is_ok() {
                self.scene.contents.drain_loaded_frame(ctx);
            }
            let feedback = result.unwrap_or_else(|s| s);
            self.scene.contents.io_feedback = feedback.clone();
            self.status = feedback;
        }
    }
}
#[cfg(test)]
#[path = "voxel_material_authoring_tests.rs"]
mod tests;
