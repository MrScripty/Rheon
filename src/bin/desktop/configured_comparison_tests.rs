//! Actual Worker production composed with saved comparison/linked inspection.
//! Analytic CPU fixtures are not trained-model or performance qualification.
use super::configured_worker_tests::{Output, app, finish, launch, member, package, set};
use super::{
    comparison_archive::*, comparison_field_view::*, inspector::Inspector, saved_comparison::*,
};
use crate::run_sets::{self, configured::ConfiguredRunSetRevision, source::SavedRunSet};
use eframe::egui;
use rheon::{
    SnapshotStage,
    recording::{
        comparison::*,
        diagnostic::NoCandidateReason,
        provider::{runtime::*, *},
    },
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

const PHASE: &str = "RHEON_CONFIGURED_COMPARISON_PHASE";
const ROOT: &str = "RHEON_CONFIGURED_COMPARISON_ROOT";
const PROCESS_TEST: &str = "configured_comparison_tests::worker_process_exits_before_fresh_comparison_reader_without_model_runtime";
const EPS: f64 = 1e-12;

fn produce(
    output: &Output,
    correction: bool,
    behavior: &str,
    name: &str,
) -> (ConfiguredRunSetRevision, PathBuf) {
    assert_ne!(
        std::env::var(PHASE).ok().as_deref(),
        Some("inspect"),
        "cold reader must not construct App/Worker/Session/provider"
    );
    let fixture = package(correction, behavior);
    let mut application = app(output);
    let root = output.0.join(name);
    launch(&mut application, set(&fixture), &root);
    let revision = finish(&mut application);
    let manifest = PathBuf::from(&application.run_sets.path);
    drop(application); // Existing Worker finish joined its child before cold use.
    let package_path = fixture.manifest.clone();
    drop(fixture); // Model code/manifest are actually unavailable to cold consumers.
    assert!(!package_path.exists());
    (revision, manifest)
}
fn pressure_recipe(
    root: &Path,
    revision: &ConfiguredRunSetRevision,
    ordinals: &[usize],
) -> SavedRecipe {
    SavedRecipe::capture_source(
        root,
        SavedRunSet::Configured(revision.clone()),
        "native",
        "configured",
        ordinals,
        ordinals,
        Recipe {
            field: FieldRole::IntervalPressure,
            time_tolerance: 0.0,
            pressure: PressurePolicy::Canonical,
        },
    )
    .unwrap()
}
fn check_pressure(verified: &VerifiedComparison, ordinals: &[usize]) {
    assert_eq!(verified.coverage().pairs.len(), ordinals.len());
    assert!(!ordinals.is_empty());
    assert!(verified.coverage().reference_unmatched.is_empty());
    assert!(verified.coverage().candidate_unmatched.is_empty());
    for (pair, ordinal) in verified.coverage().pairs.iter().zip(ordinals) {
        assert_eq!(
            (pair.reference.ordinal, pair.candidate.ordinal),
            (*ordinal, *ordinal)
        );
        let pressure = pair
            .outcome
            .as_ref()
            .expect("actual accepted pressure is available");
        assert_eq!(pressure.metrics.support_count, 8);
        assert_eq!(pressure.units, "Pa");
        assert!(
            pressure.metrics.l2 <= EPS
                && pressure.metrics.rmse <= EPS
                && pressure.metrics.linf <= EPS
        );
        assert!(
            pressure
                .metrics
                .relative_l2
                .is_some_and(|error| error <= EPS)
        );
        let gauge = pressure.gauge.unwrap();
        assert!(gauge.reference_canonical && gauge.candidate_canonical);
        assert!(!gauge.diagnostic);
        assert_eq!(gauge.removed_offset_pa.to_bits(), 0.0f64.to_bits());
    }
}
fn selected<'a>(root: &'a Path, verified: &'a VerifiedComparison) -> SelectedNativePair<'a> {
    SelectedNativePair {
        root,
        recipe: verified.recipe(),
        pair: &verified.coverage().pairs[0],
    }
}
fn check_linked(root: &Path, verified: &VerifiedComparison, ordinal: usize) {
    let mut view = ComparisonFieldView::default();
    view.load(selected(root, verified)).unwrap();
    let data = view.data().unwrap();
    assert_eq!(data.geometry.counts_xyz, [2; 3]);
    assert_eq!(data.geometry.spacing_m, [0.25; 3]);
    assert_eq!(data.sampling.association, SampleAssociation::CellCenter);
    assert_eq!(data.sampling.dimensions, [2; 3]);
    assert_eq!(data.reference.len(), 8);
    assert_eq!(data.candidate.len(), 8);
    for values in [&data.reference, &data.candidate] {
        for (index, value) in values.iter().enumerate() {
            // Independently known sealed hydrostatic solution: dp/dx=rho*a,
            // rho=2, a=1/8, dx=1/4; cell-zero gauge yields x/16 Pa.
            assert!((value - (index % 2) as f64 / 16.0).abs() <= EPS);
        }
    }
    view.set_slice(SliceAxis::Z, 1).unwrap();
    view.set_probe([1, 1, 1]).unwrap();
    let probe = view.probe().unwrap();
    assert_eq!(probe.native_index, [1, 1, 1]);
    assert_eq!(probe.physical_m, [0.375; 3]);
    assert_eq!(probe.units, "Pa");
    assert!((probe.reference - 1.0 / 16.0).abs() <= EPS);
    assert!((probe.candidate - 1.0 / 16.0).abs() <= EPS);
    for facts in view.times().unwrap() {
        assert_eq!(facts.stage, SnapshotStage::Accepted);
        assert_eq!(facts.time_s.to_bits(), (ordinal as f64 / 16.0).to_bits());
        assert_eq!(
            facts.dt_s.map(f64::to_bits),
            Some((1.0f64 / 16.0).to_bits())
        );
        assert_eq!(
            facts.pressure_interval_start_s.map(f64::to_bits),
            Some(((ordinal - 1) as f64 / 16.0).to_bits())
        );
    }
}

struct Harness {
    context: egui::Context,
    page: SavedComparisonPage,
    time: f64,
}
impl Harness {
    fn new(root: &Path, manifest: &str) -> Self {
        let mut page = SavedComparisonPage::default();
        page.set_location(root.to_string_lossy(), manifest);
        Self {
            context: egui::Context::default(),
            page,
            time: 0.0,
        }
    }
    fn frame(&mut self, events: Vec<egui::Event>) -> egui::FullOutput {
        self.time += 0.016;
        self.context.run(
            egui::RawInput {
                time: Some(self.time),
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1800.0, 2400.0),
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
            .unwrap_or_else(|| panic!("missing page control {id}"))
            .center();
        self.at(pos)
    }
    fn text(&mut self, id: &str, value: &str) {
        self.click(id);
        self.frame(vec![
            egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers {
                    ctrl: true,
                    command: true,
                    ..Default::default()
                },
            },
            egui::Event::Text(value.into()),
        ]);
    }
    fn choice(&mut self, id: &str, label: &str) {
        self.click(id);
        let out = self.frame(vec![]);
        self.at(label_position(&out, label));
    }
}
fn label_position(output: &egui::FullOutput, label: &str) -> egui::Pos2 {
    output
        .shapes
        .iter()
        .rev()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Text(text) if text.galley.job.text == label => {
                Some(text.pos + text.galley.size() * 0.5)
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing UI label {label}"))
}
fn shown(output: &egui::FullOutput) -> String {
    output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Text(text) => Some(text.galley.job.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}
fn inspector_csv(root: &Path, revision: &ConfiguredRunSetRevision) {
    let declared = revision.members[1].attempt.as_ref().unwrap();
    let inspector = Inspector::default();
    inspector
        .reload(
            &root.join(&declared.archive_relative_path),
            revision.members[1].spec.future.recording.limits(),
        )
        .unwrap();
    inspector.select_timestep(2).unwrap();
    let context = egui::Context::default();
    let mut time = 0.0;
    let mut frame = |events| {
        time += 0.016;
        context.run(
            egui::RawInput {
                time: Some(time),
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1800.0, 2400.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| inspector.ui(ui));
            },
        )
    };
    let mut click = |label: &str| {
        frame(vec![]);
        let out = frame(vec![]);
        let pos = label_position(&out, label);
        frame(vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            },
        ]);
        frame(vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        }])
    };
    click("velocity_x");
    click("interval_pressure");
    let output = click("Copy selected field sample CSV");
    let csv = output
        .platform_output
        .commands
        .iter()
        .find_map(|command| match command {
            egui::OutputCommand::CopyText(text) => Some(text),
            _ => None,
        })
        .expect("actual Inspector clipboard command");
    assert!(csv.starts_with("run_id,ordinal,generation,publication_time_s,evaluation_time_s,field,scalar_index,value,units,"));
    let columns = csv.lines().nth(1).unwrap().split(',').collect::<Vec<_>>();
    assert_eq!(columns[0].parse::<u64>().unwrap(), declared.expected_run_id);
    assert_eq!(columns[1..3], ["2", "2"]);
    assert_eq!(
        columns[3].parse::<f64>().unwrap().to_bits(),
        (2.0f64 / 16.0).to_bits()
    );
    assert_eq!(
        columns[4].parse::<f64>().unwrap().to_bits(),
        (1.0f64 / 16.0).to_bits()
    );
    assert_eq!(columns[5], "\"interval_pressure\"");
    assert_eq!(columns[6], "0");
    assert_eq!(
        columns[7].parse::<f64>().unwrap().to_bits(),
        0.0f64.to_bits()
    );
    assert_eq!(columns[8], "\"Pa\"");
    assert!(csv.contains("accepted_interval_projection"));
    assert_eq!(
        inspector.selected_identity(),
        Some((declared.expected_run_id, 2, 2.0 / 16.0))
    );
}
struct HiddenArchives(Vec<(PathBuf, PathBuf)>);
impl HiddenArchives {
    fn new(root: &Path, revision: &ConfiguredRunSetRevision) -> Self {
        Self(
            revision
                .members
                .iter()
                .map(|member| {
                    let old = root.join(&member.attempt.as_ref().unwrap().archive_relative_path);
                    let hidden = old.with_extension("hidden-sqlite");
                    std::fs::rename(&old, &hidden).unwrap();
                    (old, hidden)
                })
                .collect(),
        )
    }
}
impl Drop for HiddenArchives {
    fn drop(&mut self) {
        for (old, hidden) in &self.0 {
            std::fs::rename(hidden, old).unwrap();
        }
    }
}

#[test]
fn full_and_correction_worker_outputs_compare_known_pressure_then_cold_linked_probe_and_inspector_csv()
 {
    for correction in [false, true] {
        let output = Output::new();
        let (revision, manifest) = produce(&output, correction, "solve", "paired.runset");
        let root = manifest.parent().unwrap();
        let reader = member(root, &revision, 1);
        let accepted =
            RecordedConfiguredSession::decode(reader.run(), &reader.select(2).unwrap()).unwrap();
        assert_eq!(
            accepted.receipt.as_ref().unwrap().selected,
            RecordedPressureSelection::Configured
        );
        assert!(matches!(
            accepted.runtime.unwrap().outcome,
            RuntimeOutcome::CandidateReturned
        ));
        assert_eq!(
            audit_runtime_termination(&reader)
                .unwrap()
                .termination
                .cause,
            RuntimeTerminationCause::Completed
        );
        drop(reader);
        let recipe = pressure_recipe(root, &revision, &[2, 3]);
        let verified = recipe.compute(root).unwrap();
        check_pressure(&verified, &[2, 3]);
        recipe.save_new(&root.join("recipe.json")).unwrap();
        verified.save_new(&root.join("result.json")).unwrap();
        for name in ["recipe.json", "result.json"] {
            let packet: serde_json::Value =
                serde_json::from_slice(&std::fs::read(root.join(name)).unwrap()).unwrap();
            assert_eq!(packet["schema_version"], 2);
        }
        let cold = SavedRecipe::open(&root.join("recipe.json"))
            .unwrap()
            .compute(root)
            .unwrap();
        let cold_result = UnverifiedComparison::open(&root.join("result.json"))
            .unwrap()
            .verify(root)
            .unwrap();
        assert_eq!(cold.sha256().unwrap(), verified.sha256().unwrap());
        assert_eq!(cold_result.sha256().unwrap(), verified.sha256().unwrap());
        check_linked(root, &cold_result, 2);
        inspector_csv(root, &revision);
    }
}

#[test]
fn actual_egui_configured_page_compute_exports_and_edits_invalidate_both_linked_and_unverified_sources()
 {
    let output = Output::new();
    let (revision, manifest) = produce(&output, false, "solve", "ui.runset");
    let root = manifest.parent().unwrap();
    let mut h = Harness::new(root, manifest.file_name().unwrap().to_str().unwrap());
    h.click("open-manifest");
    assert_eq!(h.page.member_summaries().len(), 2);
    assert!(
        h.page
            .member_summaries()
            .iter()
            .all(|m| m.availability == Availability::Ready
                && m.recorded_count == Some(4)
                && m.recorded_generation == Some(3)
                && m.accepted_generation == Some(3)
                && m.unrecorded_tail == Some(false))
    );
    h.choice("role", "Interval pressure");
    h.text("reference-start", "2");
    h.text("candidate-start", "2");
    h.click("compute");
    assert_eq!(
        h.page.presentation(),
        Presentation::Verified,
        "{}",
        h.page.message()
    );
    let values = h.page.load_selected_fields().unwrap().unwrap();
    assert_eq!(values.reference.len(), 8);
    assert!(values.reference.iter().any(|p| *p > 0.0));
    let mut linked = ComparisonFieldView::default();
    linked.load(h.page.selected_pair().unwrap()).unwrap();
    linked.set_probe([1, 1, 1]).unwrap();
    assert!((linked.probe().unwrap().candidate - 1.0 / 16.0).abs() <= EPS);
    h.text("export-path", "page-recipe.json");
    h.click("export-recipe");
    h.text("export-path", "page-result.json");
    h.click("export-result");
    assert!(SavedRecipe::open(&root.join("page-recipe.json")).is_ok());
    assert!(UnverifiedComparison::open(&root.join("page-result.json")).is_ok());
    let hidden = HiddenArchives::new(root, &revision);
    // Changing a recipe invalidates cached metrics/fields without reopening
    // missing sources. Linked paint likewise drops the old selection cache.
    h.text("tolerance", "0.000001");
    assert_eq!(h.page.presentation(), Presentation::Empty);
    assert!(h.page.selected_pair().is_none());
    assert!(h.page.load_selected_fields().unwrap().is_none());
    h.time += 0.016;
    let _ = h.context.run(
        egui::RawInput {
            time: Some(h.time),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| linked.show(ui, h.page.selected_pair()));
        },
    );
    assert!(!linked.is_loaded());
    h.text("saved-path", "page-recipe.json");
    h.click("open-recipe");
    assert!(
        h.page
            .member_summaries()
            .iter()
            .all(|m| m.availability == Availability::Unchecked)
    );
    assert!(h.page.selected_pair().is_none());
    assert!(h.page.load_selected_fields().unwrap().is_none());
    h.text("saved-path", "page-result.json");
    let out = h.click("open-result");
    assert_eq!(h.page.presentation(), Presentation::Unverified);
    assert!(shown(&out).contains("UNVERIFIED stored evidence"));
    assert!(
        h.page
            .member_summaries()
            .iter()
            .all(|m| m.availability == Availability::Unchecked)
    );
    assert!(h.page.selected_pair().is_none());
    assert!(h.page.load_selected_fields().unwrap().is_none());
    h.text("export-path", "unverified-must-not-export.json");
    h.page.export_result();
    assert!(!root.join("unverified-must-not-export.json").exists());
    h.click("open-result");
    h.click("verify");
    assert_eq!(h.page.presentation(), Presentation::Empty);
    assert!(h.page.selected_pair().is_none());
    drop(hidden);
    h.click("open-result");
    h.click("verify");
    assert_eq!(
        h.page.presentation(),
        Presentation::Verified,
        "{}",
        h.page.message()
    );
    assert_eq!(h.page.selected_pair().unwrap().pair.reference.ordinal, 2);
    h.text("root", output.0.join("other").to_str().unwrap());
    assert_eq!(h.page.presentation(), Presentation::Empty);
    assert!(h.page.member_summaries().is_empty());
    assert!(h.page.selected_pair().is_none());
}

#[derive(Serialize, Deserialize)]
struct Packet<T> {
    schema_version: u32,
    kind: String,
    sha256: [u8; 32],
    payload: T,
}
#[derive(Serialize, Deserialize)]
struct ResultPayload {
    recipe: SavedRecipe,
    coverage: Coverage,
}
fn write_packet<T: Serialize>(path: &Path, packet: &mut Packet<T>) {
    packet.sha256 = Sha256::digest(serde_json::to_vec(&packet.payload).unwrap()).into();
    std::fs::write(path, serde_json::to_vec(packet).unwrap()).unwrap();
}
// A codec-only native declaration for cross-type envelope negatives. Its pins
// still refer to configured archives; it is deliberately not native evidence.
fn native_declaration(recipe: &SavedRecipe) -> SavedRecipe {
    let configured = recipe.run_set.configured().unwrap();
    let members = configured
        .members
        .iter()
        .map(|member| {
            let c = &member.spec.constructor;
            let a = member.attempt.as_ref().unwrap();
            run_sets::RunSetMember {
                member_id: member.member_id.clone(),
                spec: run_sets::FrozenSpec::new(run_sets::NativeCarrierSpec {
                    counts_xyz: c.counts_xyz,
                    spacing_m: c.spacing_m,
                    origin_m: c.origin_m,
                    density_kg_m3: c.density_kg_m3,
                    pressure_method: c.pressure_method,
                    tolerances: c.tolerances.clone(),
                    requested_dt_s: member.spec.future.requested_dt_s,
                    requested_steps: member.spec.future.requested_steps,
                    source: None,
                    simulation_memory_limit_bytes: c.simulation_memory_limit_bytes,
                    recording: member.spec.future.recording,
                })
                .unwrap(),
                differences_from_baseline: vec![],
                attempt: Some(run_sets::RunAttempt {
                    attempt_id: a.attempt_id.clone(),
                    archive_relative_path: a.archive_relative_path.clone(),
                    expected_run_id: a.expected_run_id,
                    expected_source_revision: a.expected_source_revision.clone(),
                    finalized: a.finalized.clone(),
                }),
            }
        })
        .collect();
    let mut native = recipe.clone();
    native.run_set = SavedRunSet::Native(run_sets::RunSetRevision {
        schema_version: 1,
        set_id: configured.set_id.clone(),
        parent_sha256: None,
        baseline_member: configured.baseline_member.clone(),
        members,
    });
    native.run_set_sha256 = native.run_set.sha256().unwrap();
    native.validate().unwrap();
    native
}

#[test]
fn resealed_saved_artifacts_enforce_source_version_kind_and_exact_recomputation() {
    let output = Output::new();
    let (revision, manifest) = produce(&output, false, "solve", "versions.runset");
    let root = manifest.parent().unwrap();
    let recipe = pressure_recipe(root, &revision, &[2, 3]);
    recipe.save_new(&root.join("recipe.json")).unwrap();
    recipe
        .compute(root)
        .unwrap()
        .save_new(&root.join("result.json"))
        .unwrap();
    let recipe_bytes = std::fs::read(root.join("recipe.json")).unwrap();
    let result_bytes = std::fs::read(root.join("result.json")).unwrap();
    let path = root.join("negative.json");
    for version in [1, 99] {
        let mut r: Packet<SavedRecipe> = serde_json::from_slice(&recipe_bytes).unwrap();
        r.schema_version = version;
        write_packet(&path, &mut r);
        assert!(SavedRecipe::open(&path).is_err());
        let mut r: Packet<ResultPayload> = serde_json::from_slice(&result_bytes).unwrap();
        r.schema_version = version;
        write_packet(&path, &mut r);
        assert!(UnverifiedComparison::open(&path).is_err());
    }
    let native = native_declaration(&recipe);
    assert!(
        native.compute(root).is_err(),
        "a codec-only declaration must not verify configured files as native evidence"
    );
    let mut native_result = Packet {
        schema_version: 1,
        kind: "result".into(),
        sha256: [0; 32],
        payload: ResultPayload {
            recipe: native.clone(),
            coverage: serde_json::from_slice::<Packet<ResultPayload>>(&result_bytes)
                .unwrap()
                .payload
                .coverage,
        },
    };
    write_packet(&path, &mut native_result);
    assert!(
        UnverifiedComparison::open(&path).is_ok(),
        "native result codec control remains unverified"
    );
    native_result.schema_version = 2;
    write_packet(&path, &mut native_result);
    assert!(
        UnverifiedComparison::open(&path).is_err(),
        "schema2 result must reject native source even with fresh digest"
    );
    let mut packet = Packet {
        schema_version: 1,
        kind: "recipe".into(),
        sha256: [0; 32],
        payload: native,
    };
    write_packet(&path, &mut packet);
    assert!(
        SavedRecipe::open(&path).is_ok(),
        "valid native codec control, not source verification"
    );
    packet.schema_version = 2;
    write_packet(&path, &mut packet);
    assert!(
        SavedRecipe::open(&path).is_err(),
        "schema2 must not accept native source even with fresh digest"
    );
    let mut recipe_packet: Packet<SavedRecipe> = serde_json::from_slice(&recipe_bytes).unwrap();
    recipe_packet.kind = "result".into();
    write_packet(&path, &mut recipe_packet);
    assert!(SavedRecipe::open(&path).is_err());
    let mut result_packet: Packet<ResultPayload> = serde_json::from_slice(&result_bytes).unwrap();
    result_packet.kind = "recipe".into();
    write_packet(&path, &mut result_packet);
    assert!(UnverifiedComparison::open(&path).is_err());
    let mut result_packet: Packet<ResultPayload> = serde_json::from_slice(&result_bytes).unwrap();
    let field = result_packet.payload.coverage.pairs[0]
        .outcome
        .as_mut()
        .unwrap();
    field.metrics.l2 = 0.001;
    field.metrics.rmse = 0.001 / 8.0f64.sqrt();
    field.metrics.linf = 0.001;
    field.metrics.relative_l2 = Some(0.008);
    write_packet(&path, &mut result_packet);
    let unverified = UnverifiedComparison::open(&path)
        .expect("finite resealed stored metrics remain unverified");
    assert!(
        unverified.verify(root).is_err(),
        "recomputed exact sources must reject invented stored metric"
    );
}

#[test]
fn actual_quality_fallback_and_invalid_payload_comparisons_keep_runtime_and_host_outcomes_distinct()
{
    for behavior in ["bad_once", "invalid_once"] {
        let output = Output::new();
        let (revision, manifest) = produce(&output, false, behavior, "fallback.runset");
        let root = manifest.parent().unwrap();
        let reader = member(root, &revision, 1);
        let accepted =
            RecordedConfiguredSession::decode(reader.run(), &reader.select(2).unwrap()).unwrap();
        let receipt = accepted.receipt.as_ref().unwrap();
        assert!(receipt.prediction_called);
        assert!(matches!(
            receipt.selected,
            RecordedPressureSelection::Native(_)
        ));
        let runtime = &accepted.runtime.as_ref().unwrap().outcome;
        if behavior == "bad_once" {
            assert_eq!(
                receipt.rejection,
                Some(RecordedProviderRejection::QualityGate)
            );
            assert!(receipt.candidate.is_some());
            assert!(matches!(runtime, RuntimeOutcome::CandidateReturned));
            assert_eq!(reader.status().unwrap().outcome, "completed");
        } else {
            assert!(matches!(
                receipt.rejection,
                Some(RecordedProviderRejection::Execution(_))
            ));
            assert!(receipt.candidate.is_none());
            let RuntimeOutcome::NoCandidate {
                reason,
                dispatched,
                invalid_payload_sha256,
                ..
            } = runtime
            else {
                panic!("invalid payload is not a host-qualified candidate")
            };
            assert_eq!(*reason, NoCandidateReason::InvalidOutput);
            assert!(*dispatched);
            assert_eq!(
                *invalid_payload_sha256,
                Some(Sha256::digest(b"{\"kind\":\"candidate\",\"bogus\":1}\n").into())
            );
            assert_eq!(reader.status().unwrap().outcome, "failed");
            assert_eq!(
                audit_runtime_termination(&reader)
                    .unwrap()
                    .termination
                    .cause,
                RuntimeTerminationCause::RuntimeFailure
            );
        }
        drop(reader);
        let verified = pressure_recipe(root, &revision, &[2])
            .compute(root)
            .unwrap();
        check_pressure(&verified, &[2]);
        check_linked(root, &verified, 2);
    }
}

#[test]
fn worker_process_exits_before_fresh_comparison_reader_without_model_runtime() {
    match std::env::var(PHASE).ok().as_deref() {
        Some("produce") => {
            // The orchestrating parent's Output guard owns and removes this
            // namespace after BOTH children exit. The child borrows its owner.
            let output =
                std::mem::ManuallyDrop::new(Output(PathBuf::from(std::env::var(ROOT).unwrap())));
            let (revision, manifest) = produce(&output, false, "solve", "process.runset");
            let root = manifest.parent().unwrap();
            let recipe = pressure_recipe(root, &revision, &[2, 3]);
            recipe.save_new(&root.join("recipe.json")).unwrap();
            let verified = recipe.compute(root).unwrap();
            check_pressure(&verified, &[2, 3]);
            verified.save_new(&root.join("result.json")).unwrap();
            std::fs::write(
                output.0.join("worker-exited-source-ready"),
                manifest.to_string_lossy().as_bytes(),
            )
            .unwrap();
            println!("ACTUAL_WORKER_COMPARISON_PRODUCER_FINISHED");
        }
        Some("inspect") => {
            let output = PathBuf::from(std::env::var(ROOT).unwrap());
            assert!(output.join("worker-exited-source-ready").is_file());
            let root = output.join("process.runset");
            let recipe = SavedRecipe::open(&root.join("recipe.json")).unwrap();
            let configured = recipe.run_set.configured().unwrap();
            let crate::run_sets::configured::PressureIntent::NativePrefixThenConfigured {
                deployment: crate::run_sets::configured::DeploymentLocator::Absolute(path),
                ..
            } = &configured.members[1].spec.future.pressure
            else {
                panic!("configured frozen provider declaration missing")
            };
            assert!(!Path::new(path).exists());
            let verified = UnverifiedComparison::open(&root.join("result.json"))
                .unwrap()
                .verify(&root)
                .unwrap();
            check_pressure(&verified, &[2, 3]);
            check_linked(&root, &verified, 2);
            inspector_csv(&root, configured);
            let mut h = Harness::new(&root, "manifest-not-needed.json");
            h.text("saved-path", "result.json");
            h.click("open-result");
            assert_eq!(h.page.presentation(), Presentation::Unverified);
            assert!(h.page.load_selected_fields().unwrap().is_none());
            h.click("verify");
            assert_eq!(
                h.page.presentation(),
                Presentation::Verified,
                "{}",
                h.page.message()
            );
            assert_eq!(
                h.page
                    .load_selected_fields()
                    .unwrap()
                    .unwrap()
                    .reference
                    .len(),
                8
            );
            println!("ACTUAL_FRESH_COMPARISON_READER_VERIFIED_WITHOUT_RUNTIME");
        }
        None => {
            let output = Output::new();
            for (phase, marker) in [
                ("produce", "ACTUAL_WORKER_COMPARISON_PRODUCER_FINISHED"),
                (
                    "inspect",
                    "ACTUAL_FRESH_COMPARISON_READER_VERIFIED_WITHOUT_RUNTIME",
                ),
            ] {
                let mut child = Command::new(std::env::current_exe().unwrap());
                child.args(["--exact", PROCESS_TEST, "--nocapture"]);
                if phase == "inspect" {
                    child.env_clear().env("PATH", "/no-model-runtime");
                }
                let result = child
                    .env(PHASE, phase)
                    .env(ROOT, &output.0)
                    .output()
                    .unwrap();
                assert!(
                    result.status.success(),
                    "{phase}: {}\n{}",
                    String::from_utf8_lossy(&result.stdout),
                    String::from_utf8_lossy(&result.stderr)
                );
                let stdout = String::from_utf8_lossy(&result.stdout);
                assert!(stdout.contains(marker), "{phase}: {stdout}");
                assert!(stdout.contains("1 passed"), "{phase}: {stdout}");
            }
        }
        Some(_) => panic!("unknown configured comparison phase"),
    }
}
