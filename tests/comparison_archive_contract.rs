#![cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
use rheon::recording;
#[path = "../src/bin/desktop/comparison_archive.rs"]
mod comparison_archive;
use rheon::recording::comparison as native_comparison;
#[allow(dead_code)]
#[path = "../src/bin/desktop/run_sets.rs"]
mod run_sets;
use comparison_archive::*;
use native_comparison::{Coverage, FieldRole, PressurePolicy, Recipe};
use recording::*;
use run_sets::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "rheon-comparison-archive-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn spec(method: NativePressureMethod) -> NativeCarrierSpec {
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
fn produce_member(root: &Path, m: &RunSetMember, detail: &str) {
    let attempt = m.attempt.as_ref().unwrap();
    let spec = &m.spec.spec;
    let mut native = spec.create_simulation().unwrap();
    let run = bound_native_run(
        &m.spec,
        &native,
        rheon::SessionIdentity {
            run_id: attempt.expected_run_id,
            source_revision: attempt.expected_source_revision.clone(),
        },
        "native comparison artifact fixture; no model runtime",
    )
    .unwrap();
    let mut writer = StepRecorder::create(
        &root.join(&attempt.archive_relative_path),
        run,
        spec.recording.limits(),
    )
    .unwrap();
    writer
        .append(&carrier_publication(&native, 0, spec.recording.max_record_bytes).unwrap())
        .unwrap();
    for interval in 0..spec.requested_steps {
        native
            .step(spec.requested_dt_s, spec.source_at(interval), |_| false)
            .unwrap();
        writer
            .append(
                &carrier_publication(&native, interval + 1, spec.recording.max_record_bytes)
                    .unwrap(),
            )
            .unwrap();
    }
    writer
        .finish("completed", detail, Some(native.state().generation))
        .unwrap();
}
fn produce(root: &Path) -> RunSetRevision {
    let mut set = RunSetRevision {
        schema_version: 1,
        set_id: "saved-native-comparison".into(),
        parent_sha256: None,
        baseline_member: "jacobi".into(),
        members: Vec::new(),
    };
    for (id, method, run_id) in [
        ("jacobi", NativePressureMethod::JacobiPcgV1, 101),
        ("sgs", NativePressureMethod::SymmetricGaussSeidelPcgV1, 102),
    ] {
        let mut m = RunSetMember {
            member_id: id.into(),
            spec: FrozenSpec::new(spec(method)).unwrap(),
            differences_from_baseline: if id == "jacobi" {
                vec![]
            } else {
                vec![SpecDifference::PressureMethod]
            },
            attempt: Some(RunAttempt {
                attempt_id: format!("attempt-{id}"),
                archive_relative_path: format!("{id}.sqlite"),
                expected_run_id: run_id,
                expected_source_revision: "actual-artifact-native-test-source".into(),
                finalized: None,
            }),
        };
        produce_member(root, &m, "native fixture completed");
        m.attempt
            .as_mut()
            .unwrap()
            .pin_closed(root, &m.spec)
            .unwrap();
        set.members.push(m);
    }
    set.save_new(&root.join("run-set.json")).unwrap();
    set
}
fn recipe(root: &Path, set: RunSetRevision) -> SavedRecipe {
    SavedRecipe::capture(
        root,
        set,
        "jacobi",
        "sgs",
        &[1, 2, 3],
        &[1, 2],
        Recipe {
            field: FieldRole::IntervalPressure,
            time_tolerance: 0.0,
            pressure: PressurePolicy::Canonical,
        },
    )
    .unwrap()
}
fn saved(root: &Path) -> (SavedRecipe, PathBuf) {
    let r = recipe(root, produce(root));
    let path = root.join("comparison.json");
    r.compute(root).unwrap().save_new(&path).unwrap();
    (r, path)
}
#[derive(Serialize, Deserialize)]
struct FixtureEnvelope<T> {
    schema_version: u32,
    kind: String,
    sha256: [u8; 32],
    payload: T,
}
#[derive(Serialize, Deserialize)]
struct FixtureRecord {
    recipe: SavedRecipe,
    coverage: Coverage,
}
fn mutate_result(path: &Path, mutate: impl FnOnce(&mut FixtureRecord)) {
    let mut envelope: FixtureEnvelope<FixtureRecord> =
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    mutate(&mut envelope.payload);
    envelope.sha256 = Sha256::digest(serde_json::to_vec(&envelope.payload).unwrap()).into();
    std::fs::write(path, serde_json::to_vec(&envelope).unwrap()).unwrap();
}

#[test]
fn comparison_archive_child() {
    let Ok(mode) = std::env::var("RHEON_COMPARISON_ARCHIVE_CHILD_MODE") else {
        return;
    };
    let root = PathBuf::from(std::env::var_os("RHEON_COMPARISON_ARCHIVE_CHILD_ROOT").unwrap());
    match mode.as_str() {
        "produce" => {
            let r = recipe(&root, produce(&root));
            r.save_new(&root.join("recipe.json")).unwrap();
            r.compute(&root)
                .unwrap()
                .save_new(&root.join("result.json"))
                .unwrap();
        }
        "verify" => {
            let r = SavedRecipe::open(&root.join("recipe.json")).unwrap();
            let raw = UnverifiedComparison::open(&root.join("result.json")).unwrap();
            assert_eq!(raw.recipe(), &r);
            assert_eq!(raw.recorded_coverage().pairs.len(), 2);
            let verified = raw.verify(&root).unwrap();
            assert_eq!(verified.recipe().sha256().unwrap(), r.sha256().unwrap());
            assert_eq!(verified.coverage().reference_unmatched.len(), 1);
            assert!(verified.coverage().candidate_unmatched.is_empty());
            assert!(
                verified.coverage().pairs.iter().all(|p| p
                    .outcome
                    .as_ref()
                    .unwrap()
                    .metrics
                    .support_count
                    == 24)
            );
            assert_eq!(
                verified.sha256().unwrap(),
                r.compute(&root).unwrap().sha256().unwrap()
            );
        }
        _ => panic!("unexpected child mode"),
    }
}
#[test]
fn real_producer_process_exits_then_fresh_process_reopens_recipe_and_verifies_result() {
    let root = Directory::new();
    for mode in ["produce", "verify"] {
        let status = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "comparison_archive_child", "--nocapture"])
            .env("RHEON_COMPARISON_ARCHIVE_CHILD_MODE", mode)
            .env("RHEON_COMPARISON_ARCHIVE_CHILD_ROOT", &root.0)
            .status()
            .unwrap();
        assert!(status.success(), "child {mode}");
    }
}
#[test]
fn no_overwrite_and_missing_source_never_substitutes_latest_or_claims_verified() {
    let root = Directory::new();
    let (r, path) = saved(&root.0);
    let bytes = std::fs::read(&path).unwrap();
    let verified = r.compute(&root.0).unwrap();
    assert!(
        matches!(verified.save_new(&path),Err(ComparisonArchiveError::Artifact(RunSetError::Io(e))) if e.kind()==std::io::ErrorKind::AlreadyExists)
    );
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    let raw = UnverifiedComparison::open(&path).unwrap();
    std::fs::rename(
        root.0.join("jacobi.sqlite"),
        root.0.join("newer-jacobi.sqlite"),
    )
    .unwrap();
    assert!(matches!(
        raw.verify(&root.0),
        Err(ComparisonArchiveError::Source {
            side: Side::Reference,
            problem: SourceProblem::Missing
        })
    ));
    // The file is still readable as explicitly unverified derived evidence.
    assert_eq!(
        UnverifiedComparison::open(&path)
            .unwrap()
            .recorded_coverage()
            .pairs
            .len(),
        2
    );
}
#[test]
fn changed_terminal_artifact_and_corrupt_source_are_explicit() {
    let root = Directory::new();
    let (r, path) = saved(&root.0);
    let raw = UnverifiedComparison::open(&path).unwrap();
    std::fs::remove_file(root.0.join("sgs.sqlite")).unwrap();
    produce_member(
        &root.0,
        &r.run_set.members[1],
        "different terminal receipt; exact publication fields unchanged",
    );
    assert!(matches!(
        raw.verify(&root.0),
        Err(ComparisonArchiveError::Source {
            side: Side::Candidate,
            problem: SourceProblem::Invalid(RunSetError::Integrity("finalized archive changed"))
        })
    ));
    std::fs::write(root.0.join("jacobi.sqlite"), b"corrupt sqlite bytes").unwrap();
    assert!(matches!(
        raw.verify(&root.0),
        Err(ComparisonArchiveError::Source {
            side: Side::Reference,
            problem: SourceProblem::Invalid(_)
        })
    ));
}
#[test]
fn exact_selection_hash_and_plausible_rehashed_metric_are_checked_by_recomputation() {
    let root = Directory::new();
    let (mut r, path) = saved(&root.0);
    r.reference.selections[0].sha256[0] ^= 1;
    assert!(matches!(
        r.compute(&root.0),
        Err(ComparisonArchiveError::Source {
            side: Side::Reference,
            problem: SourceProblem::SelectionChanged
        })
    ));
    mutate_result(&path, |record| {
        record.coverage.pairs[0]
            .outcome
            .as_mut()
            .unwrap()
            .metrics
            .l2 += 1.0;
    });
    let raw = UnverifiedComparison::open(&path).unwrap();
    assert!(matches!(
        raw.verify(&root.0),
        Err(ComparisonArchiveError::Integrity(
            "result differs from exact source recomputation"
        ))
    ));
}
#[test]
fn unknown_versions_enums_duplicate_fields_truncation_and_hash_damage_reject_precisely() {
    let root = Directory::new();
    let (_, path) = saved(&root.0);
    let original = std::fs::read_to_string(&path).unwrap();
    std::fs::write(
        &path,
        original.replacen("\"schema_version\":1", "\"schema_version\":999999999999", 1),
    )
    .unwrap();
    assert!(matches!(
        UnverifiedComparison::open(&path),
        Err(ComparisonArchiveError::Artifact(RunSetError::Unsupported(
            "manifest version"
        )))
    ));
    std::fs::write(
        &path,
        original.replacen("\"interval_pressure\"", "\"unknown_native_role\"", 1),
    )
    .unwrap();
    assert!(
        matches!(UnverifiedComparison::open(&path),Err(ComparisonArchiveError::Artifact(RunSetError::Codec(e))) if e.is_data())
    );
    std::fs::write(
        &path,
        original.replacen(
            "\"schema_version\":1",
            "\"schema_version\":1,\"schema_version\":1",
            1,
        ),
    )
    .unwrap();
    assert!(
        matches!(UnverifiedComparison::open(&path),Err(ComparisonArchiveError::Artifact(RunSetError::Codec(e))) if e.is_data())
    );
    std::fs::write(&path, &original.as_bytes()[..original.len() / 2]).unwrap();
    assert!(
        matches!(UnverifiedComparison::open(&path),Err(ComparisonArchiveError::Artifact(RunSetError::Codec(e))) if e.is_eof())
    );
    std::fs::write(&path, &original).unwrap();
    let mut envelope: FixtureEnvelope<FixtureRecord> = serde_json::from_str(&original).unwrap();
    envelope.sha256[0] ^= 1;
    std::fs::write(&path, serde_json::to_vec(&envelope).unwrap()).unwrap();
    assert!(matches!(
        UnverifiedComparison::open(&path),
        Err(ComparisonArchiveError::Integrity("artifact payload digest"))
    ));
    std::fs::write(&path, &original).unwrap();
    mutate_result(&path, |record| record.recipe.analysis_version = 2);
    assert!(matches!(
        UnverifiedComparison::open(&path),
        Err(ComparisonArchiveError::Unsupported("analysis version"))
    ));
}
#[test]
fn rehashed_structural_result_damage_is_invalid_before_any_source_access() {
    let root = Directory::new();
    let (_, path) = saved(&root.0);
    let original = std::fs::read(&path).unwrap();
    for mutation in 0..5 {
        std::fs::write(&path, &original).unwrap();
        mutate_result(&path, |record| {
            let pair = &mut record.coverage.pairs[0];
            match mutation {
                0 => pair.outcome.as_mut().unwrap().metrics.rmse = -1.0,
                1 => pair.outcome.as_mut().unwrap().color_bounds.min = f64::MAX,
                2 => {
                    pair.outcome
                        .as_mut()
                        .unwrap()
                        .gauge
                        .as_mut()
                        .unwrap()
                        .diagnostic = true
                }
                3 => pair.outcome.as_mut().unwrap().units = "1",
                4 => {
                    record.coverage.pairs.pop();
                }
                _ => unreachable!(),
            }
        });
        let expected = match mutation {
            0 | 1 => "finite result metrics and bounds",
            2 => "pressure gauge report",
            3 => "result units",
            4 => "result selection coverage",
            _ => unreachable!(),
        };
        assert!(
            matches!(UnverifiedComparison::open(&path),Err(ComparisonArchiveError::Invalid(r)) if r==expected),
            "mutation {mutation}"
        );
    }
}
#[test]
fn bounded_ordered_selection_contract_and_recipe_hash_are_checked() {
    let root = Directory::new();
    let set = produce(&root.0);
    let r = Recipe {
        field: FieldRole::Tracer,
        time_tolerance: 0.0,
        pressure: PressurePolicy::Canonical,
    };
    for ordinals in [vec![], vec![1, 1], vec![2, 1], (0..129).collect()] {
        assert!(matches!(
            SavedRecipe::capture(&root.0, set.clone(), "jacobi", "sgs", &ordinals, &[1], r),
            Err(ComparisonArchiveError::Invalid(
                "ordered bounded selections"
            ))
        ));
    }
    let mut saved = recipe(&root.0, set);
    saved.run_set_sha256[0] ^= 1;
    assert!(matches!(
        saved.validate(),
        Err(ComparisonArchiveError::Integrity("run-set revision digest"))
    ));
}

#[test]
fn all_native_units_and_explicit_gauge_zero_reference_semantics_round_trip() {
    let root = Directory::new();
    let set = produce(&root.0);
    for (i, field) in [
        FieldRole::Tracer,
        FieldRole::IntervalPressure,
        FieldRole::VelocityX,
        FieldRole::VelocityY,
        FieldRole::VelocityZ,
        FieldRole::VelocityDivergence,
    ]
    .into_iter()
    .enumerate()
    {
        let r = SavedRecipe::capture(
            &root.0,
            set.clone(),
            "jacobi",
            "sgs",
            &[1, 2],
            &[1, 2],
            Recipe {
                field,
                time_tolerance: 0.0,
                pressure: PressurePolicy::Canonical,
            },
        )
        .unwrap();
        let original = r.compute(&root.0).unwrap();
        let path = root.0.join(format!("role-{i}.json"));
        original.save_new(&path).unwrap();
        let raw = UnverifiedComparison::open(&path).unwrap();
        let verified = raw.verify(&root.0).unwrap();
        assert_eq!(verified.sha256().unwrap(), original.sha256().unwrap());
        assert_eq!(verified.coverage(), original.coverage());
    }
    let zero_recipe = SavedRecipe::capture(
        &root.0,
        set.clone(),
        "jacobi",
        "sgs",
        &[0],
        &[0],
        Recipe {
            field: FieldRole::Tracer,
            time_tolerance: 0.0,
            pressure: PressurePolicy::Canonical,
        },
    )
    .unwrap();
    let zero = zero_recipe.compute(&root.0).unwrap();
    let zero_path = root.0.join("zero-reference.json");
    zero.save_new(&zero_path).unwrap();
    let verified = UnverifiedComparison::open(&zero_path)
        .unwrap()
        .verify(&root.0)
        .unwrap();
    let field = verified.coverage().pairs[0].outcome.as_ref().unwrap();
    assert_eq!(field.metrics.relative_l2, None);
    assert_eq!(field.metrics.l2, 0.0);
    assert_eq!(
        field.color_bounds,
        native_comparison::ColorBounds { min: 0.0, max: 0.0 }
    );
    let mut r = recipe(&root.0, set);
    r.recipe.pressure = PressurePolicy::RemoveMeanOffsetDiagnostic;
    let original = r.compute(&root.0).unwrap();
    let path = root.0.join("offset.json");
    original.save_new(&path).unwrap();
    let verified = UnverifiedComparison::open(&path)
        .unwrap()
        .verify(&root.0)
        .unwrap();
    assert!(
        verified.coverage().pairs[0]
            .outcome
            .as_ref()
            .unwrap()
            .gauge
            .unwrap()
            .diagnostic
    );
}
#[test]
fn unavailable_geometry_outcomes_and_unmatched_coverage_are_preserved() {
    let root = Directory::new();
    let mut set = produce(&root.0);
    let candidate = &mut set.members[1];
    std::fs::remove_file(root.0.join("sgs.sqlite")).unwrap();
    let mut altered = candidate.spec.spec.clone();
    altered.counts_xyz[2] += 1;
    candidate.spec = FrozenSpec::new(altered).unwrap();
    candidate.differences_from_baseline =
        vec![SpecDifference::Geometry, SpecDifference::PressureMethod];
    candidate.attempt.as_mut().unwrap().finalized = None;
    produce_member(&root.0, candidate, "different recorded grid");
    candidate
        .attempt
        .as_mut()
        .unwrap()
        .pin_closed(&root.0, &candidate.spec)
        .unwrap();
    let r = recipe(&root.0, set);
    let computed = r.compute(&root.0).unwrap();
    let path = root.0.join("unavailable.json");
    computed.save_new(&path).unwrap();
    let verified = UnverifiedComparison::open(&path)
        .unwrap()
        .verify(&root.0)
        .unwrap();
    assert_eq!(verified.coverage().pairs.len(), 2);
    assert_eq!(verified.coverage().reference_unmatched.len(), 1);
    assert!(
        verified
            .coverage()
            .pairs
            .iter()
            .all(|p| p.outcome == Err(native_comparison::Unavailable::Geometry))
    );
}
#[test]
fn required_nullable_facts_and_unknown_keys_are_not_silently_defaulted() {
    let root = Directory::new();
    let (_, path) = saved(&root.0);
    let bytes = std::fs::read(&path).unwrap();
    for target in ["relative_l2", "gauge", "extra_fact"] {
        let mut document: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let field = &mut document["payload"]["coverage"]["pairs"][0]["outcome"]["Ok"];
        match target {
            "relative_l2" => {
                field["metrics"]
                    .as_object_mut()
                    .unwrap()
                    .remove("relative_l2");
            }
            "gauge" => {
                field.as_object_mut().unwrap().remove("gauge");
            }
            "extra_fact" => {
                field
                    .as_object_mut()
                    .unwrap()
                    .insert("extra_fact".into(), serde_json::Value::Bool(true));
            }
            _ => unreachable!(),
        }
        std::fs::write(&path, serde_json::to_vec(&document).unwrap()).unwrap();
        assert!(
            matches!(UnverifiedComparison::open(&path),Err(ComparisonArchiveError::Artifact(RunSetError::Codec(e))) if e.is_data()),
            "{target}"
        );
    }
}

#[test]
fn artifact_byte_budget_rejects_before_decoding_an_oversized_valid_json_file() {
    let root = Directory::new();
    let (_, path) = saved(&root.0);
    let mut bytes = std::fs::read(&path).unwrap();
    bytes.resize(1024 * 1024 + 1, b' ');
    std::fs::write(&path, bytes).unwrap();
    assert!(matches!(
        UnverifiedComparison::open(&path),
        Err(ComparisonArchiveError::Artifact(RunSetError::Invalid(
            "manifest byte limit"
        )))
    ));
}
