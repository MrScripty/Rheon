//! Cold native run-set inspection and exact comparison. This page owns UI state
//! only; numerical fields, source verification and artifact publication retain
//! their existing owners. Root registration and linked field rendering are separate.
use super::comparison_archive::{SavedRecipe, UnverifiedComparison, VerifiedComparison};
use super::native_comparison::{
    ComparedValues, ComparedValuesError, Coverage, FieldComparison, FieldRole, PairComparison,
    PressurePolicy, Recipe, Selection, Unavailable, compared_values,
};
use super::run_sets::{
    MemberState, SpecDifference, archive_path, relative_path,
    source::{SavedMember, SavedRunSet},
};
use eframe::egui::{self, Color32};
use rheon::SnapshotStage;
use rheon::recording::provider::comparison::{
    self as configured_facts, ConfiguredAggregate, ConfiguredSelectionFacts, RhsRelation,
};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

const MAX_SELECTIONS: usize = 128;
const PATH_CHARACTERS: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Availability {
    Unchecked,
    Ready,
    NotRun,
    Missing,
    Incomplete,
    Unpinned,
    Unsupported,
    Invalid,
}
#[derive(Debug, Clone)]
pub struct MemberSummary {
    pub member_id: String,
    pub availability: Availability,
    pub detail: String,
    pub recorded_count: Option<usize>,
    pub recorded_generation: Option<u64>,
    pub accepted_generation: Option<u64>,
    pub unrecorded_tail: Option<bool>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Presentation {
    Empty,
    Unverified,
    Verified,
}
#[derive(Debug, Clone, Copy)]
struct PublicationTime {
    endpoint: f64,
    dt: Option<f64>,
    interval_start: Option<f64>,
    stage: SnapshotStage,
}
#[derive(Debug, Clone)]
struct PairTime {
    reference: PublicationTime,
    candidate: PublicationTime,
    configured: [Option<ConfiguredSelectionFacts>; 2],
    rhs: RhsRelation,
}
struct SourcePresentation {
    times: Vec<PairTime>,
    aggregates: [Option<ConfiguredAggregate>; 2],
}

/// Minimal seam for a future linked view; unavailable until exact verification.
/// Consumers must use the recipe's pinned source identities, never live Frames.
pub struct SelectedNativePair<'a> {
    pub root: &'a Path,
    pub recipe: &'a SavedRecipe,
    pub pair: &'a PairComparison,
}

#[derive(Debug)]
pub enum SelectedFieldsError {
    Source(String),
    Comparison(ComparedValuesError),
}
impl std::fmt::Display for SelectedFieldsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Source(reason) => write!(f, "selected native source: {reason}"),
            Self::Comparison(error) => write!(f, "selected native fields: {error}"),
        }
    }
}
impl std::error::Error for SelectedFieldsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Comparison(error) => Some(error),
            Self::Source(_) => None,
        }
    }
}

pub struct SavedComparisonPage {
    root: String,
    manifest_path: String,
    saved_artifact_path: String,
    export_path: String,
    loaded: Option<SavedRunSet>,
    origin: String,
    members: Vec<MemberSummary>,
    reference: String,
    candidate: String,
    reference_start: String,
    reference_end: String,
    candidate_start: String,
    candidate_end: String,
    field: FieldRole,
    tolerance: String,
    pressure: PressurePolicy,
    pending_recipe: Option<SavedRecipe>,
    unverified: Option<UnverifiedComparison>,
    verified: Option<VerifiedComparison>,
    times: Vec<PairTime>,
    aggregates: [Option<ConfiguredAggregate>; 2],
    selected: usize,
    message: String,
    failed: bool,
    #[cfg(test)]
    controls: BTreeMap<&'static str, egui::Rect>,
}
impl Default for SavedComparisonPage {
    fn default() -> Self {
        Self {
            root: ".".into(),
            manifest_path: "run-set.json".into(),
            saved_artifact_path: "comparison.json".into(),
            export_path: "comparison-new.json".into(),
            loaded: None,
            origin: String::new(),
            members: Vec::new(),
            reference: String::new(),
            candidate: String::new(),
            reference_start: "0".into(),
            reference_end: "0".into(),
            candidate_start: "0".into(),
            candidate_end: "0".into(),
            field: FieldRole::Tracer,
            tolerance: "0".into(),
            pressure: PressurePolicy::Canonical,
            pending_recipe: None,
            unverified: None,
            verified: None,
            times: Vec::new(),
            aggregates: [None, None],
            selected: 0,
            message: "Open an immutable run-set manifest or saved comparison.".into(),
            failed: false,
            #[cfg(test)]
            controls: BTreeMap::new(),
        }
    }
}
impl SavedComparisonPage {
    /// Setting a source namespace invalidates every old verification and loaded binding.
    pub fn set_location(&mut self, root: impl Into<String>, manifest: impl Into<String>) {
        self.root = root.into();
        self.manifest_path = manifest.into();
        self.invalidate_binding();
    }
    pub fn presentation(&self) -> Presentation {
        if self.verified.is_some() {
            Presentation::Verified
        } else if self.unverified.is_some() {
            Presentation::Unverified
        } else {
            Presentation::Empty
        }
    }
    pub fn member_summaries(&self) -> &[MemberSummary] {
        &self.members
    }
    pub fn message(&self) -> &str {
        &self.message
    }
    pub fn selected_pair(&self) -> Option<SelectedNativePair<'_>> {
        let result = self.verified.as_ref()?;
        Some(SelectedNativePair {
            root: Path::new(&self.root),
            recipe: result.recipe(),
            pair: result.coverage().pairs.get(self.selected)?,
        })
    }
    /// Explicit synchronous materialization, not a per-frame getter. An
    /// unverified/no-pair page returns None without source I/O. Callers may cache
    /// successful arrays by the exact pair identities and recipe for rendering.
    pub fn load_selected_fields(&mut self) -> Result<Option<ComparedValues>, SelectedFieldsError> {
        let result = (|| {
            let Some(selected) = self.selected_pair() else {
                return Ok(None);
            };
            let mut sources = Vec::new();
            for (member_id, expected) in [
                (
                    &selected.recipe.reference.member_id,
                    &selected.pair.reference,
                ),
                (
                    &selected.recipe.candidate.member_id,
                    &selected.pair.candidate,
                ),
            ] {
                let member =
                    selected.recipe.run_set.member(member_id).ok_or_else(|| {
                        SelectedFieldsError::Source("selected member missing".into())
                    })?;
                let reader = match member.open(selected.root) {
                    MemberState::Ready(reader) => reader,
                    MemberState::Missing => {
                        return Err(SelectedFieldsError::Source("exact archive missing".into()));
                    }
                    MemberState::Invalid(error) => {
                        return Err(SelectedFieldsError::Source(error.to_string()));
                    }
                    MemberState::Unsupported(reason) => {
                        return Err(SelectedFieldsError::Source(reason.into()));
                    }
                    _ => {
                        return Err(SelectedFieldsError::Source(
                            "exact archive is not ready and pinned".into(),
                        ));
                    }
                };
                let source = Selection::read(&reader, expected.ordinal)
                    .map_err(|e| SelectedFieldsError::Source(e.to_string()))?;
                if source.identity() != expected {
                    return Err(SelectedFieldsError::Source(
                        "exact selected publication changed".into(),
                    ));
                }
                sources.push(source);
            }
            let data = compared_values(&sources[0], &sources[1], selected.recipe.recipe)
                .map_err(SelectedFieldsError::Comparison)?;
            if data.comparison != *selected.pair {
                return Err(SelectedFieldsError::Source(
                    "materialized summary differs from verified comparison".into(),
                ));
            }
            Ok(Some(data))
        })();
        if let Err(error) = &result
            && matches!(
                error,
                SelectedFieldsError::Source(_)
                    | SelectedFieldsError::Comparison(ComparedValuesError::Recipe(_))
            )
        {
            self.fail(error);
        }
        result
    }
    fn clear_presented(&mut self) {
        self.verified = None;
        self.unverified = None;
        self.times.clear();
        self.aggregates = [None, None];
        self.selected = 0;
    }
    fn invalidate_analysis(&mut self) {
        self.clear_presented();
        self.pending_recipe = None;
        self.failed = false;
        self.message = "Selection changed. Compute a new comparison.".into();
    }
    fn invalidate_binding(&mut self) {
        self.invalidate_analysis();
        self.loaded = None;
        self.members.clear();
        self.origin.clear();
        self.reference.clear();
        self.candidate.clear();
        self.message = "Source location changed. Open the intended artifact again.".into();
    }
    fn fail(&mut self, error: impl ToString) {
        self.clear_presented();
        self.pending_recipe = None;
        if let Some(set) = &self.loaded {
            self.members = summaries(set, Some(Path::new(&self.root)));
        }
        self.failed = true;
        self.message = error.to_string();
    }
    fn source_path(&self, relative: &str) -> Result<PathBuf, String> {
        relative_path(relative).map_err(|e| e.to_string())?;
        archive_path(Path::new(&self.root), relative).map_err(|e| e.to_string())
    }
    fn destination(&self) -> Result<PathBuf, String> {
        relative_path(&self.export_path).map_err(|e| e.to_string())?;
        if let Some(parent) = Path::new(&self.export_path)
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
        {
            archive_path(
                Path::new(&self.root),
                parent.to_str().ok_or("UTF-8 destination required")?,
            )
            .map_err(|e| e.to_string())?;
        }
        Ok(Path::new(&self.root).join(&self.export_path))
    }
    pub fn load_manifest(&mut self) {
        self.invalidate_binding();
        let result = self
            .source_path(&self.manifest_path)
            .and_then(|p| SavedRunSet::open(&p).map_err(|e| e.to_string()));
        match result {
            Ok(set) => {
                self.reference = set.baseline_member_id().to_owned();
                self.candidate = set
                    .member_ids()
                    .find(|id| *id != self.reference)
                    .map_or_else(|| self.reference.clone(), str::to_owned);
                self.members = summaries(&set, Some(Path::new(&self.root)));
                self.loaded = Some(set);
                self.reset_ranges();
                self.origin = "Opened manifest revision".into();
                self.failed = false;
                self.message = "Source status checked. Select publications and compute.".into();
            }
            Err(e) => self.fail(e),
        }
    }
    fn reset_ranges(&mut self) {
        for (id, start, end) in [
            (
                &self.reference,
                &mut self.reference_start,
                &mut self.reference_end,
            ),
            (
                &self.candidate,
                &mut self.candidate_start,
                &mut self.candidate_end,
            ),
        ] {
            *start = "0".into();
            *end = self
                .members
                .iter()
                .find(|m| &m.member_id == id)
                .and_then(|m| m.recorded_count)
                .map_or(0, |n| n.saturating_sub(1).min(MAX_SELECTIONS - 1))
                .to_string();
        }
    }
    fn install_recipe_inputs(&mut self, recipe: &SavedRecipe) {
        self.loaded = Some(recipe.run_set.clone());
        self.members = summaries(&recipe.run_set, None);
        self.reference = recipe.reference.member_id.clone();
        self.candidate = recipe.candidate.member_id.clone();
        self.reference_start = recipe
            .reference
            .selections
            .first()
            .unwrap()
            .ordinal
            .to_string();
        self.reference_end = recipe
            .reference
            .selections
            .last()
            .unwrap()
            .ordinal
            .to_string();
        self.candidate_start = recipe
            .candidate
            .selections
            .first()
            .unwrap()
            .ordinal
            .to_string();
        self.candidate_end = recipe
            .candidate
            .selections
            .last()
            .unwrap()
            .ordinal
            .to_string();
        self.field = recipe.recipe.field;
        self.tolerance = recipe.recipe.time_tolerance.to_string();
        self.pressure = recipe.recipe.pressure;
    }
    pub fn open_saved_recipe(&mut self) {
        self.invalidate_binding();
        let result = self
            .source_path(&self.saved_artifact_path)
            .and_then(|p| SavedRecipe::open(&p).map_err(|e| e.to_string()));
        match result {
            Ok(recipe) => {
                self.install_recipe_inputs(&recipe);
                self.pending_recipe = Some(recipe);
                self.origin = "Embedded saved recipe revision".into();
                self.failed = false;
                self.message = "Pinned recipe opened. Compute checks its exact source pins.".into();
            }
            Err(e) => self.fail(e),
        }
    }
    pub fn open_saved_result(&mut self) {
        self.invalidate_binding();
        let result = self
            .source_path(&self.saved_artifact_path)
            .and_then(|p| UnverifiedComparison::open(&p).map_err(|e| e.to_string()));
        match result {
            Ok(raw) => {
                self.install_recipe_inputs(raw.recipe());
                self.pending_recipe = Some(raw.recipe().clone());
                self.unverified = Some(raw);
                self.origin = "Embedded saved result revision".into();
                self.failed = false;
                self.message =
                    "Unverified stored result. Verify exact sources before using these metrics."
                        .into();
            }
            Err(e) => self.fail(e),
        }
    }
    fn input_recipe(&self) -> Result<Recipe, String> {
        let recipe = Recipe {
            field: self.field,
            time_tolerance: self
                .tolerance
                .parse()
                .map_err(|_| "Time tolerance must be a finite nonnegative number of seconds")?,
            pressure: self.pressure,
        };
        recipe.validate().map_err(|e| e.to_string())?;
        Ok(recipe)
    }
    pub fn compute(&mut self) {
        self.clear_presented();
        let result = (|| {
            let recipe = if let Some(pinned) = &self.pending_recipe {
                pinned.clone()
            } else {
                let set = self
                    .loaded
                    .as_ref()
                    .ok_or("Open an immutable source revision first")?;
                let capture = match set {
                    SavedRunSet::Native(native) => SavedRecipe::capture(
                        Path::new(&self.root),
                        native.clone(),
                        &self.reference,
                        &self.candidate,
                        &ordinals(&self.reference_start, &self.reference_end)?,
                        &ordinals(&self.candidate_start, &self.candidate_end)?,
                        self.input_recipe()?,
                    ),
                    SavedRunSet::Configured(_) => SavedRecipe::capture_source(
                        Path::new(&self.root),
                        set.clone(),
                        &self.reference,
                        &self.candidate,
                        &ordinals(&self.reference_start, &self.reference_end)?,
                        &ordinals(&self.candidate_start, &self.candidate_end)?,
                        self.input_recipe()?,
                    ),
                };
                capture.map_err(|e| e.to_string())?
            };
            recipe
                .compute(Path::new(&self.root))
                .map_err(|e| e.to_string())
        })();
        self.finish_computation(result);
    }
    pub fn verify_saved_result(&mut self) {
        let raw = self.unverified.take();
        self.clear_presented();
        let result = raw
            .ok_or_else(|| "Open a saved result before verification".into())
            .and_then(|raw| raw.verify(Path::new(&self.root)).map_err(|e| e.to_string()));
        self.finish_computation(result);
    }
    fn finish_computation(&mut self, result: Result<VerifiedComparison, String>) {
        match result.and_then(|verified| {
            let times = pair_times(
                Path::new(&self.root),
                verified.recipe(),
                verified.coverage(),
            )?;
            Ok((verified, times))
        }) {
            Ok((verified, times)) => {
                self.members = summaries(&verified.recipe().run_set, Some(Path::new(&self.root)));
                if [
                    &verified.recipe().reference.member_id,
                    &verified.recipe().candidate.member_id,
                ]
                .iter()
                .any(|id| {
                    self.members
                        .iter()
                        .find(|s| &s.member_id == *id)
                        .is_none_or(|s| s.availability != Availability::Ready)
                }) {
                    self.fail(
                        "Exact selected source became unavailable during the source-status check",
                    );
                    return;
                }
                self.pending_recipe = Some(verified.recipe().clone());
                self.verified = Some(verified);
                self.times = times.times;
                self.aggregates = times.aggregates;
                self.failed = false;
                self.message="Verified against exact recorded sources. Derived analysis; physics acceptance is separate.".into();
            }
            Err(e) => self.fail(e),
        }
    }
    pub fn export_recipe(&mut self) {
        self.export(false);
    }
    pub fn export_result(&mut self) {
        self.export(true);
    }
    fn export(&mut self, result: bool) {
        let outcome = (|| {
            let verified = self
                .verified
                .as_ref()
                .ok_or("Compute or verify a comparison before export")?;
            let path = self.destination()?;
            if result {
                verified.save_new(&path)
            } else {
                verified.recipe().save_new(&path)
            }
            .map_err(|e| e.to_string())?;
            Ok::<_, String>(path)
        })();
        match outcome {
            Ok(path) => {
                self.failed = false;
                self.message = format!(
                    "Published immutable {}: {}",
                    if result { "result" } else { "recipe" },
                    path.display()
                );
            }
            Err(e) => self.fail(e),
        }
    }
    fn remember(&mut self, _id: &'static str, response: egui::Response) -> egui::Response {
        #[cfg(test)]
        {
            self.controls.insert(_id, response.rect);
        }
        response
    }
    #[cfg(test)]
    pub fn control_rect(&self, id: &str) -> Option<egui::Rect> {
        self.controls.get(id).copied()
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        #[cfg(test)]
        self.controls.clear();
        egui::ScrollArea::vertical().id_salt("saved-native-comparison").show(ui,|ui|{
            ui.heading("Saved recorded comparisons");
            ui.label("Inspect immutable run sets and compare exact recorded publications.");
            ui.horizontal(|ui|{ui.label("Dataset root");let r=ui.add(egui::TextEdit::singleline(&mut self.root).char_limit(PATH_CHARACTERS));if self.remember("root",r).changed(){self.invalidate_binding();}});
            ui.horizontal(|ui|{ui.label("Manifest relative path");let r=ui.add(egui::TextEdit::singleline(&mut self.manifest_path).char_limit(PATH_CHARACTERS));if self.remember("manifest",r).changed(){self.invalidate_binding();}let r=ui.button("Open manifest");if self.remember("open-manifest",r).clicked(){self.load_manifest();}});
            ui.horizontal(|ui|{
                ui.label("Saved artifact relative path");let r=ui.add(egui::TextEdit::singleline(&mut self.saved_artifact_path).char_limit(PATH_CHARACTERS));if self.remember("saved-path",r).changed(){self.invalidate_binding();}
                let r=ui.button("Open recipe");if self.remember("open-recipe",r).clicked(){self.open_saved_recipe();}
                let r=ui.button("Open result");if self.remember("open-result",r).clicked(){self.open_saved_result();}
            });
            ui.separator();
            if let Some(set)=&self.loaded {
                ui.label(format!("{} · set {} · baseline {}",self.origin,set.set_id(),set.baseline_member_id()));
                for m in set.members() {
                    let summary=self.members.iter().find(|s|s.member_id==m.id());
                    ui.collapsing(format!("{} · {}",m.id(),summary.map_or("Not checked",|s|availability_label(s.availability))),|ui|{
                        if let Some(s)=summary {ui.label(&s.detail);}
                        let differences=match m {
                            SavedMember::Native(native)=>native.differences_from_baseline.iter().map(|d|difference_label(*d)).collect::<Vec<_>>(),
                            SavedMember::Configured(_)=>m.difference_labels(),
                        };
                        ui.label(format!("Declared differences: {}",if differences.is_empty(){"none".into()}else{differences.join(", ")}));
                        ui.label(m.description());
                        let (counts,spacing,origin,steps,dt)=match m {
                            SavedMember::Native(m)=>{let s=&m.spec.spec;(s.counts_xyz,s.spacing_m,s.origin_m,s.requested_steps,s.requested_dt_s)},
                            SavedMember::Configured(m)=>{let c=&m.spec.constructor;let f=&m.spec.future;
                                ui.label(format!("Density {} kg/m³ · viscosity {:?} · forces {:?}",c.density_kg_m3,f.viscosity,f.forces));
                                (c.counts_xyz,c.spacing_m,c.origin_m,f.requested_steps,f.requested_dt_s)},
                        };
                        ui.label(format!("Intended {} intervals at {:.17e} s",steps,dt));
                        ui.label(format!("Intended grid {:?}, spacing {:?} m, origin {:?} m",counts,spacing,origin));
                        if let Some(a)=m.attempt() {ui.monospace(format!("attempt {} · run {} · source {}\narchive {}",a.attempt_id,a.expected_run_id,a.expected_source_revision,a.archive_relative_path));if let Some(pin)=a.finalized {ui.monospace(format!("archive SHA256 {}",hex(pin.sha256)));}}
                    });
                }
            }
            let ids:Vec<_>=self.loaded.as_ref().map_or_else(Vec::new,|set|set.member_ids().map(str::to_owned).collect());
            if !self.members.is_empty(){ui.small("Availability is checked on explicit load, compute or verification; sources are not monitored in the background.");}
            let mut changed=false;
            ui.horizontal(|ui|{
                ui.label("Reference");let response = egui::ComboBox::from_id_salt("saved-reference").selected_text(&self.reference).show_ui(ui,|ui|{for id in &ids {changed|=ui.selectable_value(&mut self.reference,id.clone(),id).changed();}}); let _=self.remember("reference-member",response.response);
                ui.label("Candidate");let response = egui::ComboBox::from_id_salt("saved-candidate").selected_text(&self.candidate).show_ui(ui,|ui|{for id in &ids {changed|=ui.selectable_value(&mut self.candidate,id.clone(),id).changed();}}); let _=self.remember("candidate-member",response.response);
            });
            ui.label("Inclusive source ordinals (1–128 explicit publications per side)");
            ui.horizontal(|ui|{
                ui.label("Reference");let r=ui.add(egui::TextEdit::singleline(&mut self.reference_start).desired_width(60.0).char_limit(20));changed|=self.remember("reference-start",r).changed();ui.label("through");let r=ui.add(egui::TextEdit::singleline(&mut self.reference_end).desired_width(60.0).char_limit(20));changed|=self.remember("reference-end",r).changed();
                ui.label("Candidate");let r=ui.add(egui::TextEdit::singleline(&mut self.candidate_start).desired_width(60.0).char_limit(20));changed|=self.remember("candidate-start",r).changed();ui.label("through");let r=ui.add(egui::TextEdit::singleline(&mut self.candidate_end).desired_width(60.0).char_limit(20));changed|=self.remember("candidate-end",r).changed();
            });
            ui.horizontal(|ui|{
                ui.label("Field");let response = egui::ComboBox::from_id_salt("saved-role").selected_text(role_label(self.field)).show_ui(ui,|ui|{for role in ROLES {changed|=ui.selectable_value(&mut self.field,role,role_label(role)).changed();}}); let _=self.remember("role",response.response);
                ui.label("Time tolerance (s)");let r=ui.add(egui::TextEdit::singleline(&mut self.tolerance).desired_width(100.0).char_limit(32));changed|=self.remember("tolerance",r).changed();
                let response=egui::ComboBox::from_id_salt("saved-gauge").selected_text(if self.pressure==PressurePolicy::Canonical{"Canonical pressure gauge"}else{"Remove mean offset (diagnostic)"}).show_ui(ui,|ui|{
                    changed|=ui.selectable_value(&mut self.pressure,PressurePolicy::Canonical,"Canonical pressure gauge").changed();changed|=ui.selectable_value(&mut self.pressure,PressurePolicy::RemoveMeanOffsetDiagnostic,"Remove mean offset (diagnostic)").changed();
                }); let _=self.remember("gauge",response.response);
            });
            if changed {self.invalidate_analysis();}
            if let Some(pinned)=&self.pending_recipe {
                ui.small(format!("Exact pinned ordinals: reference {:?}; candidate {:?}. Editing inputs creates a new recipe.",pinned.reference.selections.iter().map(|s|s.ordinal).collect::<Vec<_>>(),pinned.candidate.selections.iter().map(|s|s.ordinal).collect::<Vec<_>>()));
            }
            ui.horizontal(|ui|{
                let r=ui.add_enabled(self.loaded.is_some(),egui::Button::new("Compute exact comparison"));if self.remember("compute",r).clicked(){self.compute();}
                let r=ui.add_enabled(self.unverified.is_some(),egui::Button::new("Verify saved result sources"));if self.remember("verify",r).clicked(){self.verify_saved_result();}
            });
            ui.horizontal(|ui|{
                ui.label("New export relative path");let r=ui.add(egui::TextEdit::singleline(&mut self.export_path).char_limit(PATH_CHARACTERS));let _=self.remember("export-path",r);
                let r=ui.add_enabled(self.verified.is_some(),egui::Button::new("Export recipe"));if self.remember("export-recipe",r).clicked(){self.export_recipe();}
                let r=ui.add_enabled(self.verified.is_some(),egui::Button::new("Export result"));if self.remember("export-result",r).clicked(){self.export_result();}
            });
            ui.colored_label(if self.failed {Color32::LIGHT_RED}else{Color32::LIGHT_BLUE},&self.message);
            if self.loaded.as_ref().is_none_or(|set|!set.is_configured()) {
                ui.small("Native archive v1 has no typed timing, provider-attempt, warmup, fallback or RHS diagnostics. Those facts are unavailable.");
            } else {
                ui.small("Configured core acceptance, optional runtime custody and pressure-call timing have separate scopes. Missing runtime facts remain unavailable; matching physical time does not establish equal RHS or trajectory.");
            }
            ui.separator();self.show_coverage(ui);
        });
    }
    fn show_coverage(&mut self, ui: &mut egui::Ui) {
        let verified = self.verified.is_some();
        let coverage = if let Some(v) = &self.verified {
            v.coverage()
        } else if let Some(v) = &self.unverified {
            v.recorded_coverage()
        } else {
            return;
        };
        ui.colored_label(
            if verified {
                Color32::LIGHT_GREEN
            } else {
                Color32::YELLOW
            },
            if verified {
                "Verified recorded-source comparison"
            } else {
                "UNVERIFIED stored evidence — exact sources have not been checked"
            },
        );
        if verified {
            for (side, summary) in [
                ("Reference", &self.aggregates[0]),
                ("Candidate", &self.aggregates[1]),
            ] {
                if let Some(s) = summary {
                    ui.collapsing(format!("{side} sampled pressure facts"),|ui|{
                    ui.small(format!("{} selected publications · {} constructors · {} accepted · {} attempted bindings · {} method calls · {} configured selected · {} native selected · {} host rejections · {} runtime observations",s.selections.len(),s.constructor_count,s.accepted_count,s.attempted_count,s.prediction_called_count,s.configured_selected_count,s.native_selected_count,s.rejected_count,s.runtime_observed_count));
                    ui.small(format!("Pressure-call durations: total {:?} · inference {:?} · qualification {:?} · fallback {:?}",s.durations.total,s.durations.inference,s.durations.qualification,s.durations.fallback));
                    ui.small("Exactly the selected subset, including gaps. Pressure-call timing is not full-step wall time, isolated model compute, warmup, throughput or speedup. Missing history and terminal tails are not reconstructed.");
                });
                }
            }
        }
        ui.label(format!(
            "{} temporal pairs · {} reference unmatched · {} candidate unmatched",
            coverage.pairs.len(),
            coverage.reference_unmatched.len(),
            coverage.candidate_unmatched.len()
        ));
        for (side, unmatched) in [
            ("Reference", &coverage.reference_unmatched),
            ("Candidate", &coverage.candidate_unmatched),
        ] {
            if !unmatched.is_empty() {
                ui.label(format!(
                    "{side} unmatched ordinals: {:?}",
                    unmatched.iter().map(|s| s.ordinal).collect::<Vec<_>>()
                ));
            }
        }
        if coverage.pairs.is_empty() {
            ui.label("No matched physical publications. No numerical metric is available.");
            return;
        }
        ui.add(
            egui::Slider::new(&mut self.selected, 0..=coverage.pairs.len() - 1)
                .text("Matched pair"),
        );
        let pair = &coverage.pairs[self.selected];
        ui.label(format!(
            "Reference ordinal {}, generation {} · candidate ordinal {}, generation {}",
            pair.reference.ordinal,
            pair.reference.generation,
            pair.candidate.ordinal,
            pair.candidate.generation
        ));
        if let Some(time) = self.times.get(self.selected).filter(|_| verified) {
            for (name, t) in [("Reference", time.reference), ("Candidate", time.candidate)] {
                ui.monospace(format!(
                    "{name}: actual endpoint {:.17e} s · stage {}",
                    t.endpoint,
                    if t.stage == SnapshotStage::Accepted {
                        "accepted"
                    } else {
                        "constructor"
                    }
                ));
                if let Some(dt) = t.dt {
                    ui.monospace(format!("actual dt {dt:.17e} s"));
                }
                if let Some(start) = t.interval_start {
                    ui.monospace(format!(
                        "projection interval [{start:.17e}, {:.17e}] s",
                        t.endpoint
                    ));
                }
            }
        } else {
            ui.label("Actual source time/stage has not been reopened for this stored result.");
        }
        if let Some(time) = self.times.get(self.selected).filter(|_| verified)
            && time.configured.iter().any(Option::is_some)
        {
            ui.label(format!(
                "Authoritative RHS payload hash relation: {:?}",
                time.rhs
            ));
            ui.small("Equal hashes do not establish equal operators, requests, trajectories or authenticated execution.");
            show_configured_facts(ui, "Reference", time.configured[0].as_ref());
            show_configured_facts(ui, "Candidate", time.configured[1].as_ref());
        }
        match &pair.outcome {
            Ok(field) => show_field(ui, field),
            Err(reason) => {
                ui.colored_label(
                    Color32::YELLOW,
                    format!("Metric unavailable: {}", unavailable_label(*reason)),
                );
            }
        }
        ui.collapsing("Exact selected source identities", |ui| {
            for (name, id) in [
                ("Reference", &pair.reference),
                ("Candidate", &pair.candidate),
            ] {
                ui.monospace(format!(
                    "{name}: run {} · source {}\npublication SHA256 {}",
                    id.run_id,
                    id.source_revision,
                    hex(id.sha256)
                ));
            }
        });
    }
}
const ROLES: [FieldRole; 6] = [
    FieldRole::Tracer,
    FieldRole::IntervalPressure,
    FieldRole::VelocityX,
    FieldRole::VelocityY,
    FieldRole::VelocityZ,
    FieldRole::VelocityDivergence,
];
fn ordinals(start: &str, end: &str) -> Result<Vec<usize>, String> {
    let start: usize = start
        .parse()
        .map_err(|_| "Ordinal start must be a nonnegative integer")?;
    let end: usize = end
        .parse()
        .map_err(|_| "Ordinal end must be a nonnegative integer")?;
    let count = end
        .checked_sub(start)
        .and_then(|n| n.checked_add(1))
        .filter(|n| *n <= MAX_SELECTIONS)
        .ok_or("Select an increasing inclusive range of at most 128 publications")?;
    Ok((0..count).map(|i| start + i).collect())
}
fn summaries(set: &SavedRunSet, root: Option<&Path>) -> Vec<MemberSummary> {
    set.members().map(|m|{
        let mut view=MemberSummary {member_id:m.id().into(),availability:Availability::Unchecked,detail:"Source not checked in this operation".into(),recorded_count:None,recorded_generation:None,accepted_generation:None,unrecorded_tail:None};
        let Some(root)=root else{return view;};
        match m.open(root) {
            MemberState::Ready(reader)=>{
                let facts=(||{let count=reader.count()?;let last=if count==0{None}else{Some(reader.select(count-1)?.generation)};Ok::<_,rheon::recording::RecordingError>((count,last,reader.status()?))})();
                match facts {Ok((count,last,status))=>{
                    view.availability=Availability::Ready;view.recorded_count=Some(count);view.recorded_generation=last;view.accepted_generation=status.accepted_generation;
                    view.unrecorded_tail=status.accepted_generation.map(|g|last.map_or(g>0,|last|g>last));
                    view.detail=format!("Terminal {} · {} recorded publications · recorded generation {:?} · accepted generation {:?} · unrecorded accepted tail {}",status.outcome,count,last,status.accepted_generation,match view.unrecorded_tail{Some(true)=>"present",Some(false)=>"absent",None=>"unavailable"});
                },Err(e)=>{view.availability=Availability::Invalid;view.detail=e.to_string();}}
            },
            MemberState::NotRun=>{view.availability=Availability::NotRun;view.detail="No run attempt saved".into();},
            MemberState::Missing=>{view.availability=Availability::Missing;view.detail="Exact referenced archive is missing".into();},
            MemberState::Incomplete{recorded_count,status}=>{view.availability=Availability::Incomplete;view.recorded_count=Some(recorded_count);view.accepted_generation=status.accepted_generation;view.detail=format!("Incomplete {} · {} recorded publications · accepted generation {:?}; recorded tail not inspected",status.outcome,recorded_count,status.accepted_generation);},
            MemberState::Unpinned{recorded_count,status}=>{view.availability=Availability::Unpinned;view.recorded_count=Some(recorded_count);view.accepted_generation=status.accepted_generation;view.detail=format!("Unpinned terminal {} · {} recorded publications · accepted generation {:?}",status.outcome,recorded_count,status.accepted_generation);},
            MemberState::Unsupported(e)=>{view.availability=Availability::Unsupported;view.detail=e.into();},
            MemberState::Invalid(e)=>{view.availability=Availability::Invalid;view.detail=e.to_string();},
        }view
    }).collect()
}
fn pair_times(
    root: &Path,
    recipe: &SavedRecipe,
    coverage: &Coverage,
) -> Result<SourcePresentation, String> {
    let sources = recipe.sources(root).map_err(|e| e.to_string())?;
    let mut aggregates = [None, None];
    for (side, selected) in sources.iter().enumerate() {
        if selected
            .first()
            .is_some_and(|s| s.run().solver == rheon::recording::provider::PROFILE)
        {
            aggregates[side] =
                Some(configured_facts::summarize(selected).map_err(|e| e.to_string())?);
        }
    }
    let maps = sources
        .iter()
        .map(|side| {
            side.iter()
                .map(|s| (s.identity().ordinal, s))
                .collect::<BTreeMap<_, _>>()
        })
        .collect::<Vec<_>>();
    let mut times = Vec::new();
    for pair in &coverage.pairs {
        let mut selected = Vec::new();
        for (side, expected) in [&pair.reference, &pair.candidate].into_iter().enumerate() {
            let source = *maps[side]
                .get(&expected.ordinal)
                .ok_or("Exact selected publication missing")?;
            if source.identity() != expected {
                return Err("Exact selected publication changed".into());
            }
            selected.push(source);
        }
        let time = |s: &Selection| {
            let p = s.publication();
            PublicationTime {
                endpoint: p.time,
                dt: p.dt,
                stage: p.stage,
                interval_start: if recipe.recipe.field == FieldRole::IntervalPressure {
                    p.field("interval_pressure").and_then(|f| f.evaluation_time)
                } else {
                    None
                },
            }
        };
        let facts = |s: &Selection| {
            if s.run().solver == rheon::recording::provider::PROFILE {
                configured_facts::inspect(s)
                    .map(Some)
                    .map_err(|e| e.to_string())
            } else {
                Ok(None)
            }
        };
        times.push(PairTime {
            reference: time(selected[0]),
            candidate: time(selected[1]),
            configured: [facts(selected[0])?, facts(selected[1])?],
            rhs: if selected
                .iter()
                .any(|s| s.run().solver == rheon::recording::provider::PROFILE)
            {
                configured_facts::rhs_relation(selected[0], selected[1])
                    .map_err(|e| e.to_string())?
            } else {
                RhsRelation::Unavailable
            },
        });
    }
    Ok(SourcePresentation { times, aggregates })
}
fn show_configured_facts(ui: &mut egui::Ui, side: &str, facts: Option<&ConfiguredSelectionFacts>) {
    let Some(facts) = facts else {
        ui.small(format!(
            "{side}: configured pressure/runtime facts unavailable for this profile"
        ));
        return;
    };
    if let Some(receipt) = &facts.receipt {
        ui.label(format!(
            "{side}: accepted pressure {:?} · host rejection {:?} · provider method entered {}",
            receipt.selected, receipt.rejection, receipt.prediction_called
        ));
        ui.small(format!(
            "Viscosity applied {} · dynamic viscosity {} Pa·s",
            receipt.viscosity_applied, receipt.dynamic_viscosity
        ));
        if let Some(context) = &receipt.context {
            ui.small(format!(
                "Accepted request native prefix {}",
                context.native_prefix_steps
            ));
        }
    } else {
        ui.small(format!(
            "{side}: constructor; no accepted pressure interval"
        ));
    }
    if let Some(runtime) = &facts.runtime {
        ui.small(format!(
            "Runtime request generation {}",
            runtime.request.context.generation
        ));
        match &runtime.outcome {
            rheon::recording::provider::runtime::RuntimeOutcome::CandidateReturned => {
                ui.small("Candidate returned from the external runtime; host acceptance is reported separately.");
            }
            rheon::recording::provider::runtime::RuntimeOutcome::NoCandidate {
                reason,
                detail,
                dispatched,
                invalid_payload_sha256,
            } => {
                ui.small(format!(
                    "Runtime {:?} · writer dispatch {} · {}",
                    reason, dispatched, detail
                ));
                if let Some(hash) = invalid_payload_sha256 {
                    ui.monospace(format!("Complete invalid reply SHA256 {}", hex(*hash)));
                }
            }
        }
        ui.small("Runtime outcome and writer dispatch have a different scope from core method entry and pressure acceptance.");
    } else {
        ui.small("Runtime observation unavailable; no dispatch conclusion follows.");
    }
}
fn show_field(ui: &mut egui::Ui, field: &FieldComparison) {
    let m = field.metrics;
    ui.monospace(format!(
        "L2 {:.17e} {}\nRMSE {:.17e} {}\nL∞ {:.17e} {}\nsupport {} native samples",
        m.l2, field.units, m.rmse, field.units, m.linf, field.units, m.support_count
    ));
    ui.monospace(m.relative_l2.map_or_else(
        || "Relative L2 unavailable: reference norm is zero".into(),
        |v| format!("Relative L2 {v:.17e}"),
    ));
    ui.monospace(format!(
        "Common bounds [{:.17e}, {:.17e}] {}",
        field.color_bounds.min, field.color_bounds.max, field.units
    ));
    if let Some(g) = field.gauge {
        ui.label(format!("Pressure gauge: reference canonical {}, candidate canonical {} · {} · removed offset {:.17e} Pa",g.reference_canonical,g.candidate_canonical,if g.diagnostic{"offset-aligned diagnostic"}else{"direct canonical comparison"},g.removed_offset_pa));
    }
}
fn hex(value: [u8; 32]) -> String {
    value.iter().map(|b| format!("{b:02x}")).collect()
}
fn availability_label(a: Availability) -> &'static str {
    match a {
        Availability::Unchecked => "Not checked",
        Availability::Ready => "Ready",
        Availability::NotRun => "Not run",
        Availability::Missing => "Missing",
        Availability::Incomplete => "Incomplete",
        Availability::Unpinned => "Unpinned",
        Availability::Unsupported => "Unsupported",
        Availability::Invalid => "Invalid",
    }
}
fn role_label(r: FieldRole) -> &'static str {
    match r {
        FieldRole::Tracer => "Tracer",
        FieldRole::IntervalPressure => "Interval pressure",
        FieldRole::VelocityX => "Velocity X",
        FieldRole::VelocityY => "Velocity Y",
        FieldRole::VelocityZ => "Velocity Z",
        FieldRole::VelocityDivergence => "Velocity divergence",
    }
}
fn difference_label(d: SpecDifference) -> &'static str {
    match d {
        SpecDifference::Geometry => "geometry",
        SpecDifference::Density => "density",
        SpecDifference::PressureMethod => "pressure method",
        SpecDifference::Tolerances => "tolerances",
        SpecDifference::TimePolicy => "time policy",
        SpecDifference::Source => "source schedule",
        SpecDifference::MemoryBudget => "memory budget",
        SpecDifference::RecordingBudget => "recording budget",
        SpecDifference::ForceMetadata => "force metadata",
    }
}
fn unavailable_label(r: Unavailable) -> &'static str {
    match r {
        Unavailable::UnsupportedProfile => "unsupported source profile",
        Unavailable::Geometry => "native geometry differs",
        Unavailable::PublicationStage => "publication stages differ",
        Unavailable::PhysicalTime => "physical times or intervals differ",
        Unavailable::MissingField => "recorded field is absent",
        Unavailable::FieldSchema => "native field schema is incompatible",
        Unavailable::NoncanonicalGauge => "canonical pressure gauge failed",
        Unavailable::EmptySupport => "empty native support",
        Unavailable::ArithmeticRange => "arithmetic result is not finite",
    }
}
