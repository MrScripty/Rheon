#![cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
use rheon::recording;
#[path = "../src/bin/desktop/comparison.rs"]
mod comparison;
#[allow(dead_code)]
#[path = "../src/bin/desktop/comparison_archive.rs"]
mod comparison_archive;
use rheon::recording::comparison as native_comparison;
#[allow(dead_code)]
#[path = "../src/bin/desktop/run_sets.rs"]
mod run_sets;
use comparison::*;
use comparison_archive::*;
use eframe::egui;
use native_comparison::{FieldRole, PressurePolicy, Recipe};
use recording::*;
use run_sets::*;
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let p = std::env::temp_dir().join(format!(
            "rheon-comparison-ui-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn make_spec(method: NativePressureMethod) -> NativeCarrierSpec {
    NativeCarrierSpec {
        counts_xyz: [2, 3, 4],
        spacing_m: [0.25, 0.5, 0.75],
        origin_m: [-1.0, 2.0, 3.0],
        density_kg_m3: 1.0,
        pressure_method: method,
        tolerances: NativeTolerances {
            relative_residual: 1e-10,
            absolute_residual_m3_s2: 1e-12,
            predicted_divergence_per_s: 1e-8,
            actual_divergence_per_s: 1e-5,
            max_courant: 1.0,
            max_pressure_iterations: 2000,
        },
        requested_dt_s: 0.003,
        requested_steps: 3,
        source: Some(SourceSpec {
            lower_m: [-0.99, 2.1, 3.1],
            upper_m: [-0.51, 2.9, 4.4],
            tracer_rate_per_s: 2.0,
            vertical_acceleration_m_s2: 1.0,
            active_steps: 3,
        }),
        simulation_memory_limit_bytes: 64 * 1024 * 1024,
        recording: ArchiveBudget {
            max_database_bytes: 8 * 1024 * 1024,
            max_record_bytes: 1024 * 1024,
            max_records: 16,
        },
    }
}
fn member(id: &str, method: NativePressureMethod) -> RunSetMember {
    RunSetMember {
        member_id: id.into(),
        spec: FrozenSpec::new(make_spec(method)).unwrap(),
        differences_from_baseline: if method == NativePressureMethod::JacobiPcgV1 {
            vec![]
        } else {
            vec![SpecDifference::PressureMethod]
        },
        attempt: Some(RunAttempt {
            attempt_id: format!("attempt-{id}"),
            archive_relative_path: format!("{id}.sqlite"),
            expected_run_id: if method == NativePressureMethod::JacobiPcgV1 {
                41
            } else {
                42
            },
            expected_source_revision: "actual-native-page-test-source".into(),
            finalized: None,
        }),
    }
}
fn produce(root: &Path, m: &RunSetMember, mode: &str) {
    let a = m.attempt.as_ref().unwrap();
    let s = &m.spec.spec;
    let mut sim = s.create_simulation().unwrap();
    let run = bound_native_run(
        &m.spec,
        &sim,
        rheon::SessionIdentity {
            run_id: a.expected_run_id,
            source_revision: a.expected_source_revision.clone(),
        },
        "native page fixture; actual numerical owner",
    )
    .unwrap();
    let mut writer = StepRecorder::create(
        &root.join(&a.archive_relative_path),
        run,
        s.recording.limits(),
    )
    .unwrap();
    if mode == "unknown-generation" {
        // A legacy failure can retain neither a publication nor terminal generation.
        writer
            .finish("failed", "terminal generation unavailable", None)
            .unwrap();
        return;
    }
    writer
        .append(&carrier_publication(&sim, 0, s.recording.max_record_bytes).unwrap())
        .unwrap();
    if mode == "active" {
        return;
    }
    for i in 0..s.requested_steps {
        sim.step(s.requested_dt_s, s.source_at(i), |_| false)
            .unwrap();
        if mode != "tail" {
            writer
                .append(&carrier_publication(&sim, i + 1, s.recording.max_record_bytes).unwrap())
                .unwrap();
        }
    }
    writer
        .finish(
            if mode == "tail" {
                "failed"
            } else {
                "completed"
            },
            if mode == "changed" {
                "changed terminal receipt"
            } else {
                "actual accepted native fixture"
            },
            Some(sim.state().generation),
        )
        .unwrap();
}
fn fixture(root: &Path) -> RunSetRevision {
    let mut set = RunSetRevision {
        schema_version: 1,
        set_id: "native-page-fixture".into(),
        parent_sha256: None,
        baseline_member: "jacobi".into(),
        members: vec![
            member("jacobi", NativePressureMethod::JacobiPcgV1),
            member("sgs", NativePressureMethod::SymmetricGaussSeidelPcgV1),
        ],
    };
    for m in &mut set.members {
        produce(root, m, "completed");
        m.attempt
            .as_mut()
            .unwrap()
            .pin_closed(root, &m.spec)
            .unwrap();
    }
    set.save_new(&root.join("set.json")).unwrap();
    set
}
struct Harness {
    ctx: egui::Context,
    page: SavedComparisonPage,
    time: f64,
}
impl Harness {
    fn new(root: &Path) -> Self {
        let mut page = SavedComparisonPage::default();
        page.set_location(root.to_string_lossy(), "set.json");
        Self {
            ctx: egui::Context::default(),
            page,
            time: 0.0,
        }
    }
    fn frame(&mut self, events: Vec<egui::Event>) -> egui::FullOutput {
        self.time += 0.016;
        self.ctx.run(
            egui::RawInput {
                time: Some(self.time),
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1600.0, 2000.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| self.page.show(ui));
            },
        )
    }
    fn at(&mut self, pos: egui::Pos2) -> egui::FullOutput {
        self.frame(vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            },
        ]);
        self.frame(vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        }])
    }
    fn click(&mut self, id: &str) -> egui::FullOutput {
        self.frame(vec![]);
        let pos = self
            .page
            .control_rect(id)
            .unwrap_or_else(|| panic!("missing {id}"))
            .center();
        self.at(pos)
    }
    fn text(&mut self, id: &str, text: &str) {
        self.click(id);
        let modifiers = egui::Modifiers {
            ctrl: true,
            command: true,
            ..Default::default()
        };
        self.frame(vec![
            egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            },
            egui::Event::Text(text.into()),
        ]);
    }
    fn choice(&mut self, id: &str, label: &str) {
        self.click(id);
        let out = self.frame(vec![]);
        let pos = out
            .shapes
            .iter()
            .rev()
            .find_map(|s| match &s.shape {
                egui::Shape::Text(t) if t.galley.job.text == label => {
                    Some(t.pos + t.galley.size() * 0.5)
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("missing choice {label}"));
        self.at(pos);
    }
}
fn text(out: &egui::FullOutput) -> String {
    out.shapes
        .iter()
        .filter_map(|s| match &s.shape {
            egui::Shape::Text(t) => Some(t.galley.job.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn real_headless_page_open_compute_export_and_unverified_verify_flow() {
    let root = Directory::new();
    fixture(&root.0);
    let mut h = Harness::new(&root.0);
    h.click("open-manifest");
    assert_eq!(h.page.member_summaries().len(), 2);
    assert!(
        h.page
            .member_summaries()
            .iter()
            .all(|m| m.availability == Availability::Ready
                && m.recorded_generation == Some(3)
                && m.accepted_generation == Some(3)
                && m.unrecorded_tail == Some(false))
    );
    h.click("compute");
    assert_eq!(h.page.presentation(), Presentation::Verified);
    let pair = h.page.selected_pair().unwrap();
    assert_eq!(pair.root, root.0);
    assert_eq!(pair.pair.reference.ordinal, 0);
    assert_eq!(pair.recipe.reference.member_id, "jacobi");
    let out = h.frame(vec![]);
    let shown = text(&out);
    assert!(shown.contains("Verified recorded-source comparison"));
    assert!(shown.contains("actual endpoint"));
    assert!(shown.contains("Common bounds"));
    assert!(shown.contains("Relative L2 unavailable: reference norm is zero"));
    h.text("export-path", "recipe.json");
    h.click("export-recipe");
    assert!(SavedRecipe::open(&root.0.join("recipe.json")).is_ok());
    h.text("export-path", "result.json");
    h.click("export-result");
    let original = std::fs::read(root.0.join("result.json")).unwrap();
    assert!(UnverifiedComparison::open(&root.0.join("result.json")).is_ok());
    h.click("export-result");
    assert_eq!(h.page.presentation(), Presentation::Empty);
    assert_eq!(std::fs::read(root.0.join("result.json")).unwrap(), original);
    h.text("saved-path", "result.json");
    let out = h.click("open-result");
    assert_eq!(h.page.presentation(), Presentation::Unverified);
    assert!(h.page.selected_pair().is_none());
    assert!(text(&out).contains("UNVERIFIED stored evidence"));
    assert!(
        h.page
            .member_summaries()
            .iter()
            .all(|m| m.availability == Availability::Unchecked)
    );
    h.click("verify");
    assert_eq!(h.page.presentation(), Presentation::Verified);
    assert!(h.page.selected_pair().is_some());
    h.text("root", root.0.join("other").to_str().unwrap());
    assert_eq!(h.page.presentation(), Presentation::Empty);
    assert!(h.page.member_summaries().is_empty());
    assert!(h.page.selected_pair().is_none());
}
#[test]
fn all_six_roles_gauge_and_tolerance_use_actual_ui_controls() {
    let root = Directory::new();
    fixture(&root.0);
    let mut h = Harness::new(&root.0);
    h.click("open-manifest");
    for (label, role) in [
        ("Interval pressure", FieldRole::IntervalPressure),
        ("Velocity X", FieldRole::VelocityX),
        ("Velocity Y", FieldRole::VelocityY),
        ("Velocity Z", FieldRole::VelocityZ),
        ("Velocity divergence", FieldRole::VelocityDivergence),
        ("Tracer", FieldRole::Tracer),
    ] {
        h.choice("role", label);
        assert_eq!(h.page.presentation(), Presentation::Empty);
        h.text("reference-start", "1");
        h.text("candidate-start", "1");
        h.click("compute");
        assert_eq!(h.page.selected_pair().unwrap().recipe.recipe.field, role);
    }
    h.choice("role", "Interval pressure");
    h.choice("gauge", "Remove mean offset (diagnostic)");
    h.text("tolerance", "0.000001");
    h.click("compute");
    let pair = h.page.selected_pair().unwrap();
    assert_eq!(pair.recipe.recipe.time_tolerance, 1e-6);
    assert!(
        pair.pair
            .outcome
            .as_ref()
            .unwrap()
            .gauge
            .unwrap()
            .diagnostic
    );
    assert!(text(&h.frame(vec![])).contains("offset-aligned diagnostic"));
}
#[test]
fn invalid_ranges_tolerance_loads_and_path_edits_clear_stale_presentation() {
    let root = Directory::new();
    fixture(&root.0);
    let mut h = Harness::new(&root.0);
    h.click("open-manifest");
    h.click("compute");
    h.text("reference-end", "128");
    assert_eq!(h.page.presentation(), Presentation::Empty);
    h.click("compute");
    assert!(h.page.message().contains("at most 128"));
    assert!(h.page.selected_pair().is_none());
    h.text("reference-end", "3");
    h.click("compute");
    assert_eq!(h.page.presentation(), Presentation::Verified);
    h.text("tolerance", "NaN");
    h.click("compute");
    assert_eq!(h.page.presentation(), Presentation::Empty);
    assert!(h.page.message().contains("InvalidTolerance"));
    h.text("tolerance", "0");
    h.click("compute");
    assert_eq!(h.page.presentation(), Presentation::Verified);
    h.text("manifest", "missing.json");
    assert_eq!(h.page.presentation(), Presentation::Empty);
    h.click("open-manifest");
    assert!(h.page.member_summaries().is_empty());
    h.text("manifest", "set.json");
    h.click("open-manifest");
    h.click("compute");
    h.text("saved-path", "missing-result.json");
    assert_eq!(h.page.presentation(), Presentation::Empty);
    h.click("open-result");
    assert!(h.page.selected_pair().is_none());
}
#[test]
fn saved_gapped_recipe_keeps_exact_ordinals_and_missing_source_verify_clears() {
    let root = Directory::new();
    let set = fixture(&root.0);
    let r = SavedRecipe::capture(
        &root.0,
        set,
        "jacobi",
        "sgs",
        &[1, 3],
        &[1, 3],
        Recipe {
            field: FieldRole::IntervalPressure,
            time_tolerance: 0.0,
            pressure: PressurePolicy::Canonical,
        },
    )
    .unwrap();
    r.save_new(&root.0.join("gapped.json")).unwrap();
    r.compute(&root.0)
        .unwrap()
        .save_new(&root.0.join("result.json"))
        .unwrap();
    let mut h = Harness::new(&root.0);
    h.text("saved-path", "gapped.json");
    h.click("open-recipe");
    assert_eq!(h.page.presentation(), Presentation::Empty);
    h.click("compute");
    let selected = h.page.selected_pair().unwrap();
    assert_eq!(
        selected
            .recipe
            .reference
            .selections
            .iter()
            .map(|s| s.ordinal)
            .collect::<Vec<_>>(),
        vec![1, 3]
    );
    h.text("saved-path", "result.json");
    h.click("open-result");
    std::fs::rename(root.0.join("jacobi.sqlite"), root.0.join("latest.sqlite")).unwrap();
    h.click("verify");
    assert_eq!(h.page.presentation(), Presentation::Empty);
    assert!(h.page.message().contains("missing exact archive"));
    assert!(h.page.selected_pair().is_none());
}
#[test]
fn manifest_source_states_and_real_unrecorded_accepted_tail_are_honest() {
    let root = Directory::new();
    let mut set = fixture(&root.0);
    for (id, mode) in [
        ("missing", "missing"),
        ("incomplete", "active"),
        ("unpinned", "completed"),
        ("not-run", "none"),
        ("unsupported", "future"),
        ("invalid", "bad"),
        ("tail", "tail"),
        ("unknown-generation", "unknown-generation"),
    ] {
        let mut m = member(id, NativePressureMethod::JacobiPcgV1);
        match mode {
            "missing" => {
                m.attempt.as_mut().unwrap().finalized =
                    set.members[0].attempt.as_ref().unwrap().finalized.clone();
            }
            "none" => m.attempt = None,
            "bad" => {
                std::fs::write(root.0.join("invalid.sqlite"), b"not sqlite").unwrap();
            }
            "future" => {
                produce(&root.0, &m, "completed");
                m.attempt
                    .as_mut()
                    .unwrap()
                    .pin_closed(&root.0, &m.spec)
                    .unwrap();
                rusqlite::Connection::open(root.0.join("unsupported.sqlite"))
                    .unwrap()
                    .pragma_update(None, "user_version", 99)
                    .unwrap();
            }
            _ => {
                produce(&root.0, &m, mode);
                if matches!(mode, "tail" | "unknown-generation") {
                    m.attempt
                        .as_mut()
                        .unwrap()
                        .pin_closed(&root.0, &m.spec)
                        .unwrap();
                }
            }
        }
        set.members.push(m);
    }
    set.save_new(&root.0.join("states.json")).unwrap();
    let mut h = Harness::new(&root.0);
    h.page.set_location(root.0.to_string_lossy(), "states.json");
    h.click("open-manifest");
    for (id, state) in [
        ("missing", Availability::Missing),
        ("incomplete", Availability::Incomplete),
        ("unpinned", Availability::Unpinned),
        ("not-run", Availability::NotRun),
        ("unsupported", Availability::Unsupported),
        ("invalid", Availability::Invalid),
    ] {
        assert_eq!(
            h.page
                .member_summaries()
                .iter()
                .find(|s| s.member_id == id)
                .unwrap()
                .availability,
            state,
            "{id}"
        );
    }
    let tail = h
        .page
        .member_summaries()
        .iter()
        .find(|s| s.member_id == "tail")
        .unwrap();
    assert_eq!(tail.availability, Availability::Ready);
    assert_eq!(tail.recorded_count, Some(1));
    assert_eq!(tail.recorded_generation, Some(0));
    assert_eq!(tail.accepted_generation, Some(3));
    assert_eq!(tail.unrecorded_tail, Some(true));
    assert!(tail.detail.contains("failed"));
    let unknown = h
        .page
        .member_summaries()
        .iter()
        .find(|s| s.member_id == "unknown-generation")
        .unwrap();
    assert_eq!(unknown.availability, Availability::Ready);
    assert_eq!(unknown.recorded_count, Some(0));
    assert_eq!(unknown.recorded_generation, None);
    assert_eq!(unknown.accepted_generation, None);
    assert_eq!(unknown.unrecorded_tail, None);
    assert!(unknown.detail.contains("failed"));
    assert!(
        unknown
            .detail
            .contains("unrecorded accepted tail unavailable")
    );
    assert!(!unknown.detail.contains("unrecorded accepted tail absent"));
}

#[test]
fn member_selection_and_absent_constructor_pressure_are_visible_without_fabricated_metrics() {
    let root = Directory::new();
    fixture(&root.0);
    let mut h = Harness::new(&root.0);
    h.click("open-manifest");
    h.click("compute");
    h.choice("reference-member", "sgs");
    assert_eq!(h.page.presentation(), Presentation::Empty);
    h.choice("candidate-member", "jacobi");
    h.click("compute");
    let pair = h.page.selected_pair().unwrap();
    assert_eq!(pair.recipe.reference.member_id, "sgs");
    assert_eq!(pair.recipe.candidate.member_id, "jacobi");
    h.choice("role", "Interval pressure");
    h.click("compute");
    let shown = text(&h.frame(vec![]));
    assert!(shown.contains("Metric unavailable: recorded field is absent"));
    assert_eq!(
        h.page.selected_pair().unwrap().pair.outcome,
        Err(native_comparison::Unavailable::MissingField)
    );
    assert!(!shown.contains("RMSE"));
}
#[test]
fn missing_source_refreshes_availability_on_both_compute_and_verify_failures() {
    let root = Directory::new();
    fixture(&root.0);
    let mut h = Harness::new(&root.0);
    h.click("open-manifest");
    h.click("compute");
    h.text("export-path", "result.json");
    h.click("export-result");
    std::fs::rename(root.0.join("jacobi.sqlite"), root.0.join("moved.sqlite")).unwrap();
    h.click("compute");
    assert_eq!(h.page.presentation(), Presentation::Empty);
    assert_eq!(
        h.page.member_summaries()[0].availability,
        Availability::Missing
    );
    h.text("saved-path", "result.json");
    h.click("open-result");
    assert_eq!(h.page.presentation(), Presentation::Unverified);
    h.click("verify");
    assert_eq!(h.page.presentation(), Presentation::Empty);
    assert_eq!(
        h.page.member_summaries()[0].availability,
        Availability::Missing
    );
    h.text("saved-path", "result.json");
    h.click("open-result");
    assert_eq!(h.page.presentation(), Presentation::Unverified);
    h.text("root", root.0.join("other-root").to_str().unwrap());
    assert_eq!(h.page.presentation(), Presentation::Empty);
    assert!(h.page.member_summaries().is_empty());
}

#[test]
fn explicit_selected_field_materialization_requires_verified_exact_sources() {
    let root = Directory::new();
    let set = fixture(&root.0);
    let recipe = SavedRecipe::capture(
        &root.0,
        set,
        "jacobi",
        "sgs",
        &[1],
        &[1],
        Recipe {
            field: FieldRole::VelocityX,
            time_tolerance: 0.0,
            pressure: PressurePolicy::Canonical,
        },
    )
    .unwrap();
    recipe
        .compute(&root.0)
        .unwrap()
        .save_new(&root.0.join("result.json"))
        .unwrap();
    let mut h = Harness::new(&root.0);
    assert!(h.page.load_selected_fields().unwrap().is_none());
    h.text("saved-path", "result.json");
    h.click("open-result");
    std::fs::rename(
        root.0.join("jacobi.sqlite"),
        root.0.join("unavailable.sqlite"),
    )
    .unwrap();
    // Missing source is immaterial here: Unverified must not attempt source I/O.
    assert!(h.page.load_selected_fields().unwrap().is_none());
    assert_eq!(h.page.presentation(), Presentation::Unverified);
    std::fs::rename(
        root.0.join("unavailable.sqlite"),
        root.0.join("jacobi.sqlite"),
    )
    .unwrap();
    h.click("verify");
    let data = h.page.load_selected_fields().unwrap().unwrap();
    assert_eq!(
        data.sampling.association,
        native_comparison::SampleAssociation::XFace
    );
    assert_eq!(data.sampling.dimensions, [3, 3, 4]);
    assert_eq!(data.reference.len(), 36);
    assert_eq!(data.comparison, *h.page.selected_pair().unwrap().pair);
    std::fs::rename(root.0.join("jacobi.sqlite"), root.0.join("changed.sqlite")).unwrap();
    assert!(matches!(
        h.page.load_selected_fields(),
        Err(SelectedFieldsError::Source(_))
    ));
    assert_eq!(h.page.presentation(), Presentation::Empty);
    assert_eq!(
        h.page.member_summaries()[0].availability,
        Availability::Missing
    );
    std::fs::rename(root.0.join("changed.sqlite"), root.0.join("jacobi.sqlite")).unwrap();
    h.click("open-result");
    h.click("verify");
    let candidate = h
        .page
        .selected_pair()
        .unwrap()
        .recipe
        .run_set
        .native()
        .unwrap()
        .members[1]
        .clone();
    std::fs::remove_file(root.0.join("sgs.sqlite")).unwrap();
    produce(&root.0, &candidate, "changed");
    assert!(
        matches!(h.page.load_selected_fields(),Err(SelectedFieldsError::Source(reason)) if reason.contains("finalized archive changed"))
    );
    assert_eq!(h.page.presentation(), Presentation::Empty);
    assert_eq!(
        h.page.member_summaries()[1].availability,
        Availability::Invalid
    );
}
#[test]
fn unavailable_native_fields_remain_verified_analysis_without_materialized_values() {
    let root = Directory::new();
    fixture(&root.0);
    let mut h = Harness::new(&root.0);
    h.click("open-manifest");
    h.choice("role", "Interval pressure");
    h.click("compute");
    assert!(matches!(
        h.page.load_selected_fields(),
        Err(SelectedFieldsError::Comparison(
            native_comparison::ComparedValuesError::Unavailable(
                native_comparison::Unavailable::MissingField
            )
        ))
    ));
    assert_eq!(h.page.presentation(), Presentation::Verified);
    assert!(h.page.selected_pair().is_some());
}

#[test]
fn legacy_comparison_workflows_keep_exact_sides_and_execution_evidence_unavailable() {
    let root = Directory::new();
    fixture(&root.0);
    let mut h = Harness::new(&root.0);
    assert!(
        h.page
            .load_selected_workflow(Side::Reference)
            .unwrap()
            .is_none()
    );
    h.click("open-manifest");
    h.click("compute");
    let reference = h
        .page
        .load_selected_workflow(Side::Reference)
        .unwrap()
        .unwrap();
    let candidate = h
        .page
        .load_selected_workflow(Side::Candidate)
        .unwrap()
        .unwrap();
    assert!(reference.key != candidate.key);
    assert!(h.page.workflow_key(Side::Reference).as_ref() == Some(&reference.key));
    assert!(h.page.workflow_key(Side::Candidate).as_ref() == Some(&candidate.key));
    assert!(reference.key.side == Side::Reference && candidate.key.side == Side::Candidate);
    assert_eq!(reference.run.identity.run_id, 41);
    assert_eq!(candidate.run.identity.run_id, 42);
    assert_eq!(reference.ordinal, candidate.ordinal);
    assert!(reference.attempt.is_none() && candidate.attempt.is_none());
}
