#![cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
use rheon::recording;
use rheon::recording::comparison as native_comparison;
#[allow(dead_code)]
#[path = "../src/bin/desktop/comparison.rs"]
mod comparison;
#[allow(dead_code)]
#[path = "../src/bin/desktop/comparison_archive.rs"]
mod comparison_archive;
#[path = "../src/bin/desktop/comparison_field_view.rs"]
mod comparison_field_view;
#[allow(dead_code)]
#[path = "../src/bin/desktop/run_sets.rs"]
mod run_sets;
use comparison::SelectedNativePair;
use comparison_archive::*;
use comparison_field_view::*;
use eframe::egui;
use native_comparison::{FieldRole, PressurePolicy, Recipe, Selection, native_samples};
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
            "rheon-linked-field-{}-{}",
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
fn verified(
    root: &Path,
    set: &RunSetRevision,
    role: FieldRole,
    pressure: PressurePolicy,
    ordinal: usize,
) -> VerifiedComparison {
    SavedRecipe::capture(
        root,
        set.clone(),
        "jacobi",
        "sgs",
        &[ordinal],
        &[ordinal],
        Recipe {
            field: role,
            time_tolerance: 1e-12,
            pressure,
        },
    )
    .unwrap()
    .compute(root)
    .unwrap()
}
fn selected<'a>(root: &'a Path, v: &'a VerifiedComparison) -> SelectedNativePair<'a> {
    SelectedNativePair {
        root,
        recipe: v.recipe(),
        pair: &v.coverage().pairs[0],
    }
}
fn duplicate<'a>(s: &SelectedNativePair<'a>) -> SelectedNativePair<'a> {
    SelectedNativePair {
        root: s.root,
        recipe: s.recipe,
        pair: s.pair,
    }
}
fn rebound(recipe: &mut SavedRecipe) {
    recipe.run_set_sha256 = recipe.run_set.sha256().unwrap();
    recipe.validate().unwrap();
}
struct Harness {
    ctx: egui::Context,
    view: ComparisonFieldView,
    time: f64,
}
impl Harness {
    fn new() -> Self {
        Self {
            ctx: egui::Context::default(),
            view: Default::default(),
            time: 0.0,
        }
    }
    fn frame(
        &mut self,
        current: Option<SelectedNativePair<'_>>,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        self.time += 0.016;
        self.ctx.run(
            egui::RawInput {
                time: Some(self.time),
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1600.0, 1800.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    self.view.show(ui, current.as_ref().map(duplicate))
                });
            },
        )
    }
    fn at(&mut self, current: &SelectedNativePair<'_>, pos: egui::Pos2) {
        self.frame(
            Some(duplicate(current)),
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: Default::default(),
                },
            ],
        );
        self.frame(
            Some(duplicate(current)),
            vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            }],
        );
    }
    fn click(&mut self, current: &SelectedNativePair<'_>, control: &str) {
        self.frame(Some(duplicate(current)), vec![]);
        let pos = self.view.control_rect(control).unwrap().center();
        self.at(current, pos);
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
fn actual_all_six_native_roles_preserve_arrays_sampling_geometry_times_and_gauge() {
    let root = Directory::new();
    let set = fixture(&root.0);
    for role in [
        FieldRole::Tracer,
        FieldRole::IntervalPressure,
        FieldRole::VelocityX,
        FieldRole::VelocityY,
        FieldRole::VelocityZ,
        FieldRole::VelocityDivergence,
    ] {
        let policy = if role == FieldRole::IntervalPressure {
            PressurePolicy::RemoveMeanOffsetDiagnostic
        } else {
            PressurePolicy::Canonical
        };
        let v = verified(&root.0, &set, role, policy, 2);
        let mut view = ComparisonFieldView::default();
        view.load(selected(&root.0, &v)).unwrap();
        let data = view.data().unwrap();
        let summary = v.coverage().pairs[0].outcome.as_ref().unwrap();
        assert_eq!(data.comparison, v.coverage().pairs[0]);
        assert_eq!(data.geometry.counts_xyz, [2, 3, 4]);
        assert_eq!(data.geometry.origin_m, [-1., 2., 3.]);
        assert_eq!(data.geometry.spacing_m, [0.25, 0.5, 0.75]);
        let (dims, offset, association) = match role {
            FieldRole::VelocityX => (
                [3, 3, 4],
                [0., 0.5, 0.5],
                native_comparison::SampleAssociation::XFace,
            ),
            FieldRole::VelocityY => (
                [2, 4, 4],
                [0.5, 0., 0.5],
                native_comparison::SampleAssociation::YFace,
            ),
            FieldRole::VelocityZ => (
                [2, 3, 5],
                [0.5, 0.5, 0.],
                native_comparison::SampleAssociation::ZFace,
            ),
            _ => (
                [2, 3, 4],
                [0.5; 3],
                native_comparison::SampleAssociation::CellCenter,
            ),
        };
        assert_eq!(data.sampling.dimensions, dims);
        assert_eq!(data.sampling.offset_cells, offset);
        assert_eq!(data.sampling.association, association);
        for (side, m) in set.members.iter().enumerate() {
            let reader = match open_member(&root.0, m) {
                MemberState::Ready(r) => r,
                _ => panic!("actual native member not ready"),
            };
            let selection = Selection::read(&reader, 2).unwrap();
            let raw = native_samples(&selection, role).unwrap();
            let expected: Vec<_> = raw
                .into_iter()
                .map(|x| {
                    if side == 1 && role == FieldRole::IntervalPressure {
                        x - summary.gauge.unwrap().removed_offset_pa
                    } else {
                        x
                    }
                })
                .collect();
            let actual = if side == 0 {
                &data.reference
            } else {
                &data.candidate
            };
            assert_eq!(*actual, expected);
            assert!(
                actual
                    .iter()
                    .all(|x| *x >= summary.color_bounds.min && *x <= summary.color_bounds.max)
            );
            let facts = view.times().unwrap()[side];
            assert_eq!(facts.time_s, selection.publication().time);
            assert_eq!(facts.dt_s, selection.publication().dt);
            assert_eq!(facts.stage, selection.publication().stage);
            assert_eq!(
                facts.pressure_interval_start_s,
                if role == FieldRole::IntervalPressure {
                    Some(0.003)
                } else {
                    None
                }
            );
        }
        let index = [dims[0] as usize - 1, 1, 1];
        view.set_probe(index).unwrap();
        let p = view.probe().unwrap();
        assert_eq!(p.native_index, index);
        assert_eq!(p.units, summary.units);
        assert_eq!(
            p.physical_m,
            std::array::from_fn(
                |d| [-1., 2., 3.][d] + (index[d] as f64 + offset[d]) * [0.25, 0.5, 0.75][d]
            )
        );
        if role == FieldRole::IntervalPressure {
            assert!(summary.gauge.unwrap().diagnostic);
        }
    }
}

#[test]
fn egui_explicit_load_shared_slice_edit_and_both_pane_probe_events() {
    let root = Directory::new();
    let set = fixture(&root.0);
    let v = verified(
        &root.0,
        &set,
        FieldRole::VelocityX,
        PressurePolicy::Canonical,
        2,
    );
    let s = selected(&root.0, &v);
    let mut h = Harness::new();
    h.frame(Some(duplicate(&s)), vec![]);
    assert!(!h.view.is_loaded());
    h.click(&s, "load");
    assert!(h.view.is_loaded());
    // DragValue enters text edit on double click. Actual keyboard replacement must
    // persist across paint and update the shared probe's normal coordinate.
    let pos = h.view.control_rect("slice-index").unwrap().center();
    h.at(&s, pos);
    h.at(&s, pos);
    let modifiers = egui::Modifiers {
        ctrl: true,
        command: true,
        ..Default::default()
    };
    h.frame(
        Some(duplicate(&s)),
        vec![
            egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            },
            egui::Event::Text("2".into()),
            egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Default::default(),
            },
        ],
    );
    h.frame(Some(duplicate(&s)), vec![]);
    assert_eq!(h.view.probe().unwrap().native_index[2], 2);
    for name in ["reference-canvas", "candidate-canvas"] {
        let rect = h.view.control_rect(name).unwrap();
        // X index 2 and Y index 1 in actual [3,3,4] x-fast native X faces.
        let pos = egui::pos2(
            rect.left() + rect.width() * 5. / 6.,
            rect.bottom() - rect.height() * 0.5,
        );
        h.at(&s, pos);
        assert_eq!(h.view.probe().unwrap().native_index, [2, 1, 2]);
    }
    let out = h.frame(Some(duplicate(&s)), vec![]);
    let labels = text(&out);
    assert!(labels.contains("XFace"));
    assert!(labels.contains("Actual endpoint"));
    assert!(labels.contains("ONE common color range"));
    assert!(labels.contains("physical XYZ"));
    assert!(labels.contains("m/s"));
}

#[test]
fn paint_uses_cached_native_arrays_but_explicit_reload_rechecks_exact_source() {
    let root = Directory::new();
    let set = fixture(&root.0);
    let v = verified(
        &root.0,
        &set,
        FieldRole::Tracer,
        PressurePolicy::Canonical,
        1,
    );
    let s = selected(&root.0, &v);
    let mut h = Harness::new();
    h.view.load(duplicate(&s)).unwrap();
    let before = h.view.probe().unwrap();
    for id in ["jacobi", "sgs"] {
        std::fs::rename(
            root.0.join(format!("{id}.sqlite")),
            root.0.join(format!("{id}.hidden")),
        )
        .unwrap();
    }
    h.frame(Some(duplicate(&s)), vec![]);
    assert_eq!(h.view.probe(), Some(before));
    assert!(h.view.control_rect("reference-canvas").is_some());
    assert!(matches!(
        h.view.load(duplicate(&s)),
        Err(FieldViewError::Source { .. })
    ));
    assert!(!h.view.is_loaded());
    for id in ["jacobi", "sgs"] {
        std::fs::rename(
            root.0.join(format!("{id}.hidden")),
            root.0.join(format!("{id}.sqlite")),
        )
        .unwrap();
    }
    h.view.load(duplicate(&s)).unwrap();
    std::fs::remove_file(root.0.join("sgs.sqlite")).unwrap();
    produce(&root.0, &set.members[1], "changed");
    assert!(matches!(
        h.view.load(duplicate(&s)),
        Err(FieldViewError::Source {
            side: "candidate",
            ..
        })
    ));
    assert!(!h.view.is_loaded());
}

#[test]
fn unverified_changed_pair_root_and_unavailable_clear_old_panes_same_frame() {
    let root = Directory::new();
    let set = fixture(&root.0);
    let v = verified(
        &root.0,
        &set,
        FieldRole::Tracer,
        PressurePolicy::Canonical,
        1,
    );
    let other = verified(
        &root.0,
        &set,
        FieldRole::Tracer,
        PressurePolicy::Canonical,
        2,
    );
    let unavailable = verified(
        &root.0,
        &set,
        FieldRole::IntervalPressure,
        PressurePolicy::Canonical,
        0,
    );
    assert!(unavailable.coverage().pairs[0].outcome.is_err());
    let mut h = Harness::new();
    let s = selected(&root.0, &v);
    h.view.load(duplicate(&s)).unwrap();
    h.frame(None, vec![]);
    assert!(!h.view.is_loaded());
    h.view.load(duplicate(&s)).unwrap();
    h.frame(Some(selected(&root.0, &other)), vec![]);
    assert!(!h.view.is_loaded());
    h.view.load(duplicate(&s)).unwrap();
    h.frame(
        Some(SelectedNativePair {
            root: Path::new("/missing/exact/root"),
            ..s
        }),
        vec![],
    );
    assert!(!h.view.is_loaded());
    h.view.load(duplicate(&s)).unwrap();
    h.frame(Some(selected(&root.0, &unavailable)), vec![]);
    assert!(!h.view.is_loaded());
    assert!(h.view.message().contains("unavailable"));
    assert!(matches!(
        h.view.load(selected(&root.0, &unavailable)),
        Err(FieldViewError::Unavailable(_))
    ));
}

#[test]
fn forged_summary_nonfinite_binding_and_geometry_are_rejected() {
    let root = Directory::new();
    let set = fixture(&root.0);
    let v = verified(
        &root.0,
        &set,
        FieldRole::Tracer,
        PressurePolicy::Canonical,
        1,
    );
    let s = selected(&root.0, &v);
    let mut view = ComparisonFieldView::default();
    let mut pair = s.pair.clone();
    pair.outcome.as_mut().unwrap().units = "kg";
    assert!(matches!(
        view.load(SelectedNativePair { pair: &pair, ..s }),
        Err(FieldViewError::SummaryMismatch)
    ));
    let mut pair = s.pair.clone();
    pair.outcome.as_mut().unwrap().color_bounds.min = f64::NAN;
    assert!(matches!(
        view.load(SelectedNativePair { pair: &pair, ..s }),
        Err(FieldViewError::Invalid("finite selected summary"))
    ));
    let mut pair = s.pair.clone();
    pair.reference.ordinal = 3;
    assert!(matches!(
        view.load(SelectedNativePair { pair: &pair, ..s }),
        Err(FieldViewError::Invalid("pair publication membership"))
    ));
    let mut recipe = s.recipe.clone();
    for m in &mut native_set_mut(&mut recipe.run_set).members {
        let mut spec = m.spec.spec.clone();
        spec.origin_m[0] -= 0.005;
        m.spec = FrozenSpec::new(spec).unwrap();
    }
    rebound(&mut recipe);
    assert!(matches!(
        view.load(SelectedNativePair {
            recipe: &recipe,
            ..s
        }),
        Err(FieldViewError::Source { .. })
    ));
    assert!(!view.is_loaded());
}

#[test]
fn preflight_sample_and_publication_budgets_precede_source_io() {
    let root = Directory::new();
    let set = fixture(&root.0);
    let v = verified(
        &root.0,
        &set,
        FieldRole::Tracer,
        PressurePolicy::Canonical,
        1,
    );
    let s = selected(&root.0, &v);
    let mut view = ComparisonFieldView::default();
    for m in &set.members {
        std::fs::remove_file(
            root.0
                .join(&m.attempt.as_ref().unwrap().archive_relative_path),
        )
        .unwrap();
    }
    let mut recipe = s.recipe.clone();
    for m in &mut native_set_mut(&mut recipe.run_set).members {
        let mut spec = m.spec.spec.clone();
        spec.counts_xyz = [128, 128, 128];
        m.spec = FrozenSpec::new(spec).unwrap();
    }
    rebound(&mut recipe);
    assert!(matches!(
        view.load(SelectedNativePair {
            recipe: &recipe,
            ..s
        }),
        Err(FieldViewError::Limit("estimated promoted samples"))
    ));
    let mut recipe = s.recipe.clone();
    for m in &mut native_set_mut(&mut recipe.run_set).members {
        let mut spec = m.spec.spec.clone();
        spec.counts_xyz = [100, 100, 100];
        m.spec = FrozenSpec::new(spec).unwrap();
    }
    rebound(&mut recipe);
    assert!(matches!(
        view.load(SelectedNativePair {
            recipe: &recipe,
            ..s
        }),
        Err(FieldViewError::Limit("estimated native selected payload"))
    ));
    let mut recipe = s.recipe.clone();
    native_set_mut(&mut recipe.run_set).members[0]
        .attempt
        .as_mut()
        .unwrap()
        .finalized
        .as_mut()
        .unwrap()
        .recorded_count = 4097;
    rebound(&mut recipe);
    assert!(matches!(
        view.load(SelectedNativePair {
            recipe: &recipe,
            ..s
        }),
        Err(FieldViewError::Limit("source publication count"))
    ));
    let sparse = std::fs::File::create(root.0.join("jacobi.sqlite")).unwrap();
    sparse.set_len(256 * 1024 * 1024 + 1).unwrap();
    assert!(matches!(
        view.load(duplicate(&s)),
        Err(FieldViewError::Limit("archive file bytes"))
    ));
    assert!(!view.is_loaded());
}

#[test]
fn single_sample_axes_and_slice_work_limit_are_explicit() {
    let root = Directory::new();
    let mut set = RunSetRevision {
        schema_version: 1,
        set_id: "thin-real-native".into(),
        parent_sha256: None,
        baseline_member: "jacobi".into(),
        members: vec![
            member("jacobi", NativePressureMethod::JacobiPcgV1),
            member("sgs", NativePressureMethod::SymmetricGaussSeidelPcgV1),
        ],
    };
    for m in &mut set.members {
        let mut spec = m.spec.spec.clone();
        spec.counts_xyz = [1, 1, 1];
        spec.source = None;
        spec.requested_steps = 1;
        m.spec = FrozenSpec::new(spec).unwrap();
        produce(&root.0, m, "completed");
        m.attempt
            .as_mut()
            .unwrap()
            .pin_closed(&root.0, &m.spec)
            .unwrap();
    }
    let v = verified(
        &root.0,
        &set,
        FieldRole::VelocityX,
        PressurePolicy::Canonical,
        1,
    );
    let mut h = Harness::new();
    let s = selected(&root.0, &v);
    h.view.load(duplicate(&s)).unwrap();
    let labels = text(&h.frame(Some(duplicate(&s)), vec![]));
    assert!(h.view.control_rect("reference-canvas").is_some());
    assert!(labels.contains("constant range, neutral color"));
    h.view.set_probe([1, 0, 0]).unwrap();
    assert_eq!(h.view.probe().unwrap().physical_m, [-0.75, 2.25, 3.375]);
    assert!(h.view.set_slice(SliceAxis::Z, 1).is_err());
    assert!(!h.view.is_loaded());
    // Real zero-state thin volume: X-normal plane exceeds the draw-work cap,
    // while Y/Z planes fit. No simulation or data values are invented.
    let root = Directory::new();
    for m in &mut set.members {
        let mut spec = m.spec.spec.clone();
        spec.counts_xyz = [1, 130, 130];
        m.spec = FrozenSpec::new(spec).unwrap();
        m.attempt.as_mut().unwrap().finalized = None;
        produce(&root.0, m, "completed");
        m.attempt
            .as_mut()
            .unwrap()
            .pin_closed(&root.0, &m.spec)
            .unwrap();
    }
    let v = verified(
        &root.0,
        &set,
        FieldRole::Tracer,
        PressurePolicy::Canonical,
        1,
    );
    h.view.load(selected(&root.0, &v)).unwrap();
    assert!(matches!(
        h.view.set_slice(SliceAxis::X, 0),
        Err(FieldViewError::Limit("slice work"))
    ));
    assert!(!h.view.is_loaded());
}

fn native_set_mut(set: &mut source::SavedRunSet) -> &mut RunSetRevision {
    match set {
        source::SavedRunSet::Native(set) => set,
        source::SavedRunSet::Configured(_) | source::SavedRunSet::Voxel(_) => {
            panic!("native fixture required")
        }
    }
}
