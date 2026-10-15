//! Explicit immutable manifest selection; no latest alias or runtime on cold open.
use super::{archive_view::ArchiveView, run_sets::*};
use eframe::egui;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct RunSetView {
    pub active: bool,
    pub path: String,
    revision: Option<RunSetRevision>,
    root: PathBuf,
    feedback: String,
    statuses: Vec<String>,
    ready: Vec<bool>,
}
impl RunSetView {
    pub fn open(&mut self) -> Result<(), String> {
        self.revision = None;
        self.statuses.clear();
        self.ready.clear();
        self.active = false;
        self.feedback.clear();
        let path = Path::new(&self.path);
        let revision = RunSetRevision::open(path).map_err(|e| e.to_string())?;
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        self.root = parent.canonicalize().map_err(|e| e.to_string())?;
        for member in &revision.members {
            let state = open_member(&self.root, member);
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
        ui.collapsing("Saved native run sets", |ui| {
            ui.small("Exact immutable manifest path. Cold inspection does not execute models.");
            ui.text_edit_singleline(&mut self.path);
            if ui
                .add_enabled(enabled, egui::Button::new("Open saved run set"))
                .clicked()
            {
                self.feedback = self.open().err().unwrap_or_default();
            }
            if !self.feedback.is_empty() {
                ui.label(&self.feedback);
            }
        });
    }
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        archive: &mut ArchiveView,
    ) -> Option<RunSetRevision> {
        let mut rerun = None;
        egui::TopBottomPanel::top("saved-set-navigation").show(ctx, |ui| {
            ui.heading("Saved native run set");
            if ui.button("Back to simulation").clicked() {
                self.active = false;
            }
            ui.label(&self.path);
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            let Some(revision) = &self.revision else { return; };
            ui.label(format!("{} · baseline {}", revision.set_id, revision.baseline_member));
            if ui.button("Rerun saved native set from zero").clicked() {
                rerun = Some(revision.clone());
                self.active = false;
            }
            ui.small("Ready verifies exact pinned artifact and typed intent; failed/cancelled outcomes keep their meaning. Archives are inspection data, not restart checkpoints.");
            ui.small("Member integrity/status was checked on open. Inspect rechecks the exact pinned artifact before selection.");
            for (i, member) in revision.members.iter().enumerate() {
                ui.separator();
                ui.label(format!("{} · {:?} · changes {:?}", member.member_id, member.spec.spec.pressure_method, member.differences_from_baseline));
                ui.label(&self.statuses[i]);
                if ui.add_enabled(self.ready[i], egui::Button::new(format!("Inspect {}", member.member_id))).clicked() {
                    if !matches!(open_member(&self.root, member), MemberState::Ready(_)) {
                        self.feedback = "Saved member changed or is unavailable; reopen the exact manifest".into();
                        self.ready[i] = false;
                        continue;
                    }
                    archive.path = self.root.join(&member.attempt.as_ref().expect("ready attempt").archive_relative_path).to_string_lossy().into();
                    match archive.open(member.spec.spec.recording.limits()) {
                        Ok(()) => self.active = false,
                        Err(e) => self.feedback = e,
                    }
                }
            }
            if !self.feedback.is_empty() { ui.label(&self.feedback); }
        });
        rerun
    }
}
