//! Explicit immutable manifest selection; cold inspection never starts a model.
use super::{archive_view::ArchiveView, run_sets::*};
use eframe::egui;
use std::path::{Path, PathBuf};

pub use super::run_sets::source::SavedRunSet;

#[derive(Default)]
pub struct RunSetView {
    pub active: bool,
    pub path: String,
    revision: Option<SavedRunSet>,
    root: PathBuf,
    feedback: String,
    statuses: Vec<String>,
    ready: Vec<bool>,
}

impl RunSetView {
    /// Charge the still-retained source recipe, without reopening its archive.
    /// Equality compares the complete immutable value, not a path or current UI
    /// recipe. Inline enum storage is conservatively charged in addition to the
    /// revision owner's capacity; status/feedback controls remain UI metadata.
    pub fn related_voxel_payload_bytes(
        &self,
        source: &voxel_material::run_set::VoxelRunSetRevision,
    ) -> Result<usize, String> {
        let Some(SavedRunSet::Voxel(stored)) = &self.revision else {
            return Ok(0);
        };
        if stored != source {
            return Ok(0);
        }
        stored
            .owned_capacity()
            .map_err(|e| e.to_string())?
            .checked_add(std::mem::size_of::<SavedRunSet>())
            .and_then(|n| n.checked_add(self.root.capacity()))
            .ok_or_else(|| "Related saved voxel capacity overflow".into())
    }
    pub fn open(&mut self) -> Result<(), String> {
        self.revision = None;
        self.statuses.clear();
        self.ready.clear();
        self.active = false;
        self.feedback.clear();
        let path = Path::new(&self.path);
        let revision = SavedRunSet::open(path).map_err(|e| e.to_string())?;
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        self.root = parent.canonicalize().map_err(|e| e.to_string())?;
        for member in revision.members() {
            let state = member.open(&self.root);
            self.ready.push(matches!(state, MemberState::Ready(_)));
            self.statuses.push(match state {
                MemberState::Ready(reader) => format!(
                    "Ready · {} committed publications · {:?}",
                    reader.count().map_err(|e| e.to_string())?,
                    reader.status().map_err(|e| e.to_string())?.outcome
                ),
                MemberState::NotRun => "Not run".into(),
                MemberState::Missing => "Missing".into(),
                MemberState::Incomplete {
                    recorded_count,
                    status,
                } => format!(
                    "Incomplete · {recorded_count} publications · {}",
                    status.outcome
                ),
                MemberState::Unpinned {
                    recorded_count,
                    status,
                } => format!(
                    "Unpinned · {recorded_count} publications · {}",
                    status.outcome
                ),
                MemberState::Unsupported(reason) => format!("Unsupported · {reason}"),
                MemberState::Invalid(error) => format!("Invalid · {error}"),
            });
        }
        self.revision = Some(revision);
        self.active = true;
        Ok(())
    }
    pub fn controls(&mut self, ui: &mut egui::Ui, enabled: bool) {
        ui.collapsing("Saved run sets",|ui| {
            ui.small("Exact immutable native, configured Session or voxel-material manifest. Cold inspection does not execute models.");
            ui.text_edit_singleline(&mut self.path);
            if ui.add_enabled(enabled,egui::Button::new("Open saved run set")).clicked() {
                self.feedback=self.open().err().unwrap_or_default();
            }
            if !self.feedback.is_empty(){ui.label(&self.feedback);}
        });
    }
    pub fn show(&mut self, ctx: &egui::Context, archive: &mut ArchiveView) -> Option<SavedRunSet> {
        let mut rerun = None;
        egui::TopBottomPanel::top("saved-set-navigation").show(ctx, |ui| {
            ui.heading("Saved run set");
            if ui.button("Back to simulation").clicked() {
                self.active = false;
            }
            ui.label(&self.path);
        });
        egui::CentralPanel::default().show(ctx,|ui| {
            let Some(revision)=&self.revision else{return;};
            ui.small(format!("{} saved members",revision.member_count()));
            ui.label(format!("{} · {} baseline {}", revision.set_id(), if revision.voxel().is_some(){"voxel material"}else if revision.is_configured(){"configured Session"}else{"native"}, revision.baseline_member_id()));
            if ui.button("Rerun saved set from zero").clicked(){rerun=Some(revision.clone());self.active=false;}
            ui.small("Ready verifies the exact closed artifact and typed intent; failed/cancelled outcomes retain their meaning. Archives are inspection data, not restart checkpoints.");
            ui.small("Status is cached from explicit open. Inspect rechecks the exact pinned member before selection.");
            for (index,member) in revision.members().enumerate() {
                let description=member.description();
                let relative=member.attempt().map_or("", |a| a.archive_relative_path);
                let limits=member.limits();
                ui.separator();ui.label(description);ui.label(&self.statuses[index]);
                if ui.add_enabled(self.ready[index],egui::Button::new(format!("Inspect member {}",index+1))).clicked() {
                    if !matches!(member.open(&self.root),MemberState::Ready(_)) {
                        self.feedback="Saved member changed or is unavailable; reopen the exact manifest".into();self.ready[index]=false;continue;
                    }
                    archive.path=self.root.join(relative).to_string_lossy().into();
                    match archive.open(limits){Ok(())=>self.active=false,Err(error)=>self.feedback=error}
                }
            }
            if !self.feedback.is_empty(){ui.label(&self.feedback);}
        });
        rerun
    }
}

#[cfg(test)]
mod payload_tests {
    use super::*;
    use crate::voxel_material_integration_tests::{recipe, request};

    #[test]
    fn related_source_compares_values_and_charges_stored_spare_capacity() {
        let (_, frozen) = recipe(0.001, 1, [0.0; 3], 1024 * 1024);
        let req = request(frozen, Path::new("unused-capacity-query"));
        let original = req.voxel.unwrap().revision.as_ref().clone();
        let mut view = RunSetView {
            revision: Some(SavedRunSet::Voxel(original.clone())),
            ..Default::default()
        };
        let before = view.related_voxel_payload_bytes(&original).unwrap();
        assert!(before > 0);
        let added = {
            let Some(SavedRunSet::Voxel(stored)) = &mut view.revision else {
                unreachable!()
            };
            let old = stored.owned_capacity().unwrap();
            stored.set_id.reserve_exact(4096);
            stored.members.reserve_exact(7);
            stored.members[0].member_id.reserve_exact(2048);
            stored.owned_capacity().unwrap() - old
        };
        assert!(added > 0);
        assert_eq!(
            view.related_voxel_payload_bytes(&original).unwrap(),
            before + added
        );
        let mut different = original.clone();
        different.set_id.push_str("-different");
        assert_eq!(view.related_voxel_payload_bytes(&different).unwrap(), 0);
        view.revision = None;
        assert_eq!(view.related_voxel_payload_bytes(&original).unwrap(), 0);
    }
}
