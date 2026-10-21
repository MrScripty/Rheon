#![cfg(not(target_arch = "wasm32"))]
use recording::*;
use rheon::recording::comparison::*;
use rheon::{GridGeometry, SessionIdentity, Simulation, SimulationConfig};
use rheon::{PressureImplementation, recording};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Archive {
    path: PathBuf,
}
impl Archive {
    fn empty() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "rheon-m3-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&dir).unwrap();
        Self {
            path: dir.join("archive.sqlite"),
        }
    }
    fn new(run: RecordingRun, steps: &[RecordedStep]) -> Self {
        let archive = Self::empty();
        let mut writer =
            StepRecorder::create(&archive.path, run, RecordingLimits::default()).unwrap();
        for s in steps {
            writer.append(s).unwrap();
        }
        writer
            .finish(
                "completed",
                "comparison fixture",
                steps.last().map(|s| s.generation),
            )
            .unwrap();
        archive
    }
    fn reader(&self) -> RecordingReader {
        RecordingReader::open(&self.path, RecordingLimits::default()).unwrap()
    }
    fn select(&self, n: usize) -> Selection {
        Selection::read(&self.reader(), n).unwrap()
    }
}
impl Drop for Archive {
    fn drop(&mut self) {
        std::fs::remove_dir_all(self.path.parent().unwrap()).unwrap();
    }
}
fn sim() -> Simulation {
    Simulation::new(
        GridGeometry::new([2, 3, 4], [0.25, 0.5, 0.75], [-1.0, 2.0, 3.0]).unwrap(),
        SimulationConfig {
            density: 1.0,
            memory_limit: 64 * 1024 * 1024,
            pressure: rheon::PressureSettings {
                relative_residual: 1e-10,
                absolute_residual: 1e-12,
                divergence_limit: 1e-8,
                max_iterations: 2000,
            },
            actual_divergence_limit: 1e-5,
            max_courant: 1.0,
        },
    )
    .unwrap()
}
fn fixture(dt: f64, constructor: bool) -> (RecordingRun, Vec<RecordedStep>) {
    let mut s = sim();
    let run = carrier_run(
        SessionIdentity {
            run_id: 7,
            source_revision: "comparison-fixture".into(),
        },
        &s,
        "actual native carrier capture; mutations are test data",
    )
    .unwrap();
    let mut steps = Vec::new();
    if constructor {
        steps.push(carrier_publication(&s, 0, 1 << 20).unwrap());
    }
    s.step(dt, None, |_| false).unwrap();
    steps.push(carrier_publication(&s, steps.len(), 1 << 20).unwrap());
    (run, steps)
}
fn recipe(field: FieldRole) -> Recipe {
    Recipe {
        field,
        time_tolerance: 0.0,
        pressure: PressurePolicy::Canonical,
    }
}
fn available(a: &Selection, b: &Selection, role: FieldRole) -> FieldComparison {
    compare(a, b, recipe(role)).unwrap().outcome.unwrap()
}

#[test]
fn c1b2_actual_projection_fields_are_not_configured_or_carrier_comparison_evidence() {
    use rheon::projection_step::{ProjectionIntent, ProjectionStepWorkspace};
    use rheon::{
        LiquidInlet, LiquidStepInputs, LiquidTransportConfig, LiquidVolumeSettings,
        PressureSettings, SimulationSession, VolumeStamp,
    };
    let cap = 4 * 1024 * 1024;
    let grid = GridGeometry::new([2, 3, 4], [0.25, 0.5, 0.75], [-1.0, 2.0, 3.0]).unwrap();
    let method = PressureImplementation::JacobiPcgV1;
    for intent in [
        ProjectionIntent::BuiltinNativeV1,
        ProjectionIntent::ClassicalSelectionV1 {
            producer: PressureImplementation::SymmetricGaussSeidelPcgV1,
            producer_max_iterations: 2000,
        },
    ] {
        let mut owner = SimulationSession::new_with_projection(
            SessionIdentity {
                run_id: 8,
                source_revision: "actual-projection-comparison-refusal".into(),
            },
            grid.clone(),
            LiquidTransportConfig {
                carrier: SimulationConfig {
                    density: 1.0,
                    memory_limit: cap,
                    pressure: PressureSettings {
                        relative_residual: 1e-10,
                        absolute_residual: 1e-12,
                        divergence_limit: 1e-8,
                        max_iterations: 2000,
                    },
                    actual_divergence_limit: 1e-5,
                    max_courant: 1.0,
                },
                represented_density: 1.0,
                volume_stamp: VolumeStamp { id: 1, version: 0 },
                carrier_id: 2,
            },
            method,
            vec![0.5; grid.cell_len()],
            cap,
            1,
            intent,
        )
        .unwrap();
        let mut workspace =
            ProjectionStepWorkspace::new_with_intent(&grid, 1.0, cap, method, intent).unwrap();
        let run =
            projection::projection_run(&owner.snapshot(), "actual typed projection publication")
                .unwrap();
        assert_eq!(
            run.solver,
            match intent {
                ProjectionIntent::BuiltinNativeV1 => "rheon-native-projection-session-v1",
                _ => "rheon-native-projection-selection-v1",
            }
        );
        owner
            .step_with_native_projection(
                LiquidStepInputs {
                    requested_dt: 0.01,
                    smoke_source: None,
                    forces: &[],
                    source: None,
                    inlet: LiquidInlet::new(VolumeStamp { id: 3, version: 0 }, [[0.0; 2]; 3])
                        .unwrap(),
                    volume: LiquidVolumeSettings {
                        max_outward_courant: 1.0,
                        actual_divergence_limit: 1e-5,
                    },
                },
                &mut workspace,
                None,
                |_| false,
            )
            .unwrap();
        let step = projection::projection_publication(&owner.snapshot(), 0, 1 << 20).unwrap();
        let new = Archive::new(run, &[step]);
        drop(workspace);
        drop(owner);
        let (run, steps) = fixture(0.01, false);
        let old = Archive::new(run, &steps);
        assert_eq!(
            native_samples(&new.select(0), FieldRole::Tracer),
            Err(Unavailable::UnsupportedProfile)
        );
        assert_eq!(
            compare(&old.select(0), &new.select(0), recipe(FieldRole::Tracer))
                .unwrap()
                .outcome,
            Err(Unavailable::UnsupportedProfile)
        );
    }
}
fn field_mut<'a>(s: &'a mut RecordedStep, name: &str) -> &'a mut RecordedField {
    s.fields.iter_mut().find(|f| f.name == name).unwrap()
}
fn put_f64(s: &mut RecordedStep, name: &str, v: impl Fn(usize) -> f64) {
    let f = field_mut(s, name);
    f.data = (0..f.data.len() / 8)
        .flat_map(|i| v(i).to_le_bytes())
        .collect();
}
fn put_f32(s: &mut RecordedStep, name: &str, v: impl Fn(usize) -> f32) {
    let f = field_mut(s, name);
    f.data = (0..f.data.len() / 4)
        .flat_map(|i| v(i).to_le_bytes())
        .collect();
}
fn hex(v: [u8; 32]) -> String {
    v.iter().map(|b| format!("{b:02x}")).collect()
}

#[test]
fn physical_interval_matches_across_different_ordinals_and_native_cold_sources() {
    let (run, steps) = fixture(0.01, true);
    let a = Archive::new(run, &steps);
    let (run, steps) = fixture(0.01, false);
    let b = Archive::new(run, &steps);
    let result = available(&a.select(1), &b.select(0), FieldRole::IntervalPressure);
    assert_eq!(result.metrics.l2, 0.0);
    assert_eq!(result.metrics.support_count, 24);
    let coverage = align(
        &[a.select(0), a.select(1)],
        &[b.select(0)],
        recipe(FieldRole::IntervalPressure),
    )
    .unwrap();
    assert_eq!(coverage.analysis_version, 1);
    assert_eq!(coverage.recipe, recipe(FieldRole::IntervalPressure));
    assert_eq!(coverage.pairs[0].analysis_version, 1);
    assert_eq!(coverage.pairs[0].recipe, coverage.recipe);
    assert_eq!(coverage.pairs.len(), 1);
    assert_eq!(coverage.reference_unmatched.len(), 1);
    assert!(coverage.candidate_unmatched.is_empty());
}
#[test]
fn equal_ordinal_does_not_match_unequal_time_or_projection_interval() {
    let (run, steps) = fixture(0.01, false);
    let a = Archive::new(run, &steps);
    let (run, steps) = fixture(0.02, false);
    let b = Archive::new(run, &steps);
    assert_eq!(
        compare(&a.select(0), &b.select(0), recipe(FieldRole::Tracer))
            .unwrap()
            .outcome,
        Err(Unavailable::PhysicalTime)
    );
    let coverage = align(&[a.select(0)], &[b.select(0)], recipe(FieldRole::Tracer)).unwrap();
    assert_eq!(coverage.reference_unmatched.len(), 1);
    assert_eq!(coverage.candidate_unmatched.len(), 1);
    let mut native = sim();
    native.step(0.01, None, |_| false).unwrap();
    let first = carrier_publication(&native, 0, 1 << 20).unwrap();
    native.step(0.01, None, |_| false).unwrap();
    let second = carrier_publication(&native, 1, 1 << 20).unwrap();
    let run = carrier_run(
        SessionIdentity {
            run_id: 7,
            source_revision: "interval-fixture".into(),
        },
        &native,
        "two actual native intervals",
    )
    .unwrap();
    let a = Archive::new(run, &[first, second]);
    assert_eq!(
        compare(
            &a.select(1),
            &b.select(0),
            recipe(FieldRole::IntervalPressure)
        )
        .unwrap()
        .outcome,
        Err(Unavailable::PhysicalTime)
    );
    assert_eq!(
        available(&a.select(1), &b.select(0), FieldRole::Tracer)
            .metrics
            .l2,
        0.0
    );
}
#[test]
fn stage_geometry_and_unknown_profile_are_explicit() {
    let (run, steps) = fixture(0.01, true);
    let a = Archive::new(run, &steps);
    assert_eq!(
        compare(&a.select(0), &a.select(1), recipe(FieldRole::Tracer))
            .unwrap()
            .outcome,
        Err(Unavailable::PublicationStage)
    );
    let (mut run, steps) = fixture(0.01, true);
    run.origin[0] += 1.0;
    let b = Archive::new(run, &steps);
    assert_eq!(
        compare(&a.select(1), &b.select(1), recipe(FieldRole::Tracer))
            .unwrap()
            .outcome,
        Err(Unavailable::Geometry)
    );
    let (mut run, steps) = fixture(0.01, true);
    run.solver = "future-profile/unknown".into();
    let b = Archive::new(run, &steps);
    assert_eq!(
        native_samples(&b.select(1), FieldRole::Tracer),
        Err(Unavailable::UnsupportedProfile)
    );
    assert_eq!(
        compare(&a.select(1), &b.select(1), recipe(FieldRole::Tracer))
            .unwrap()
            .outcome,
        Err(Unavailable::UnsupportedProfile)
    );
    let (run, mut steps) = fixture(0.01, true);
    field_mut(&mut steps[1], "tracer").stage = "tentative".into();
    let b = Archive::new(run, &steps);
    assert_eq!(
        compare(&a.select(1), &b.select(1), recipe(FieldRole::Tracer))
            .unwrap()
            .outcome,
        Err(Unavailable::FieldSchema)
    );
}
#[test]
fn missing_field_remains_unavailable_inside_matched_coverage() {
    let (run, steps) = fixture(0.01, false);
    let a = Archive::new(run, &steps);
    let (run, mut steps) = fixture(0.01, false);
    steps[0].fields.retain(|f| f.name != "tracer");
    let b = Archive::new(run, &steps);
    let coverage = align(&[a.select(0)], &[b.select(0)], recipe(FieldRole::Tracer)).unwrap();
    assert_eq!(coverage.pairs[0].outcome, Err(Unavailable::MissingField));
    assert!(coverage.reference_unmatched.is_empty());
}
#[test]
fn pressure_offset_diagnostic_retains_canonical_gauge_failure_and_offset() {
    let (run, mut steps) = fixture(0.01, false);
    put_f64(&mut steps[0], "interval_pressure", |i| i as f64);
    let a = Archive::new(run, &steps);
    let (run, mut steps) = fixture(0.01, false);
    put_f64(&mut steps[0], "interval_pressure", |i| i as f64 + 8.0);
    let b = Archive::new(run, &steps);
    assert_eq!(
        compare(
            &a.select(0),
            &b.select(0),
            recipe(FieldRole::IntervalPressure)
        )
        .unwrap()
        .outcome,
        Err(Unavailable::NoncanonicalGauge)
    );
    let mut r = recipe(FieldRole::IntervalPressure);
    r.pressure = PressurePolicy::RemoveMeanOffsetDiagnostic;
    let result = compare(&a.select(0), &b.select(0), r)
        .unwrap()
        .outcome
        .unwrap();
    assert!(result.metrics.linf < 1e-13);
    let gauge = result.gauge.unwrap();
    assert!(gauge.reference_canonical);
    assert!(!gauge.candidate_canonical);
    assert!(gauge.diagnostic);
    assert!((gauge.removed_offset_pa - 8.0).abs() < 1e-13);
    oracle_with_policy(&a, &b, FieldRole::IntervalPressure, &result, true);
}
#[test]
fn analytic_native_f32_metrics_zero_denominator_and_common_bounds() {
    let (run, steps) = fixture(0.01, false);
    let a = Archive::new(run, &steps);
    let (run, mut steps) = fixture(0.01, false);
    put_f32(
        &mut steps[0],
        "tracer",
        |i| if i % 2 == 0 { 0.25 } else { 0.5 },
    );
    let b = Archive::new(run, &steps);
    let samples = native_samples(&b.select(0), FieldRole::Tracer).unwrap();
    assert_eq!(samples.len(), 24);
    for (i, sample) in samples.iter().enumerate() {
        assert_eq!(*sample, if i % 2 == 0 { 0.25 } else { 0.5 });
    }
    let result = available(&a.select(0), &b.select(0), FieldRole::Tracer);
    assert_eq!(result.metrics.relative_l2, None);
    assert!((result.metrics.l2 - (12.0_f64 * 0.3125).sqrt()).abs() < 1e-14);
    assert!((result.metrics.rmse - 0.15625_f64.sqrt()).abs() < 1e-14);
    assert_eq!(result.metrics.linf, 0.5);
    assert_eq!(result.color_bounds, ColorBounds { min: 0.0, max: 0.5 });
    oracle(&a, &b, FieldRole::Tracer, &result);
}
fn analytic_faces(s: &mut RecordedStep, scale: f32) {
    put_f32(s, "velocity_x", |i| scale * (i % 3) as f32 * 0.25);
    put_f32(s, "velocity_y", |i| {
        scale * 2.0 * ((i / 2) % 4) as f32 * 0.5
    });
    put_f32(s, "velocity_z", |i| -scale * (i / 6) as f32 * 0.75);
}
#[test]
fn divergence_uses_native_staggered_faces_spacing_and_x_fast_indices() {
    let (run, mut steps) = fixture(0.01, false);
    analytic_faces(&mut steps[0], 1.0);
    let a = Archive::new(run, &steps);
    let (run, mut steps) = fixture(0.01, false);
    analytic_faces(&mut steps[0], 2.0);
    let b = Archive::new(run, &steps);
    let result = available(&a.select(0), &b.select(0), FieldRole::VelocityDivergence);
    assert!((result.metrics.l2 - 2.0 * 24.0_f64.sqrt()).abs() < 1e-13);
    assert!((result.metrics.rmse - 2.0).abs() < 1e-14);
    assert_eq!(result.metrics.linf, 2.0);
    assert_eq!(result.metrics.relative_l2, Some(1.0));
    assert_eq!(result.units, "1/s");
    assert_eq!(result.color_bounds, ColorBounds { min: 2.0, max: 4.0 });
    oracle(&a, &b, FieldRole::VelocityDivergence, &result);
    for role in [
        FieldRole::VelocityX,
        FieldRole::VelocityY,
        FieldRole::VelocityZ,
    ] {
        let result = available(&a.select(0), &b.select(0), role);
        oracle(&a, &b, role, &result);
    }
}
fn oracle(a: &Archive, b: &Archive, role: FieldRole, result: &FieldComparison) {
    oracle_with_policy(a, b, role, result, false);
}
fn oracle_with_policy(
    a: &Archive,
    b: &Archive,
    role: FieldRole,
    result: &FieldComparison,
    diagnostic: bool,
) {
    let status = oracle_command(a, b, role, result, diagnostic, false)
        .status()
        .unwrap();
    assert!(status.success());
}
fn oracle_command(
    a: &Archive,
    b: &Archive,
    role: FieldRole,
    result: &FieldComparison,
    diagnostic: bool,
    optimized: bool,
) -> std::process::Command {
    let name = match role {
        FieldRole::Tracer => "tracer",
        FieldRole::VelocityDivergence => "divergence",
        FieldRole::VelocityX => "velocity_x",
        FieldRole::VelocityY => "velocity_y",
        FieldRole::VelocityZ => "velocity_z",
        FieldRole::IntervalPressure => "interval_pressure",
    };
    let mut command = std::process::Command::new("python3");
    command.env_remove("PYTHONOPTIMIZE");
    if optimized {
        command.arg("-O");
    }
    command.arg(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tools/verify_recording_comparison.py"
    ));
    if diagnostic {
        command.arg("--remove-mean-offset");
    }
    command
        .arg(&a.path)
        .arg(&b.path)
        .args(["--field", name, "--expect"])
        .arg(format!("{:.17e}", result.metrics.l2))
        .arg(format!("{:.17e}", result.metrics.rmse))
        .arg(format!("{:.17e}", result.metrics.linf))
        .arg(
            result
                .metrics
                .relative_l2
                .map_or("undefined".into(), |v| format!("{v:.17e}")),
        )
        .arg("--expect-selection")
        .arg(hex(a.select(0).digest()))
        .arg("--expect-candidate-selection")
        .arg(hex(b.select(0).digest()));
    command
}
#[test]
fn independent_oracle_rejects_wrong_identities_and_metrics_with_python_optimization() {
    let (run, steps) = fixture(0.01, false);
    let a = Archive::new(run, &steps);
    let (run, mut steps) = fixture(0.01, false);
    put_f32(&mut steps[0], "tracer", |_| 0.5);
    let b = Archive::new(run, &steps);
    let role = FieldRole::Tracer;
    let result = available(&a.select(0), &b.select(0), role);
    // Each mode first proves the same valid native-bit fixture reaches success.
    // Negative invocations then reach the intended identity or metric check.
    for (optimized, environment) in [(false, false), (true, false), (false, true)] {
        let command = || {
            let mut command = oracle_command(&a, &b, role, &result, false, optimized);
            if environment {
                command.env("PYTHONOPTIMIZE", "1");
            }
            command
        };
        let valid = command().output().unwrap();
        assert!(valid.status.success(), "{valid:?}");
        for (flag, archive, message) in [
            (
                "--expect-selection",
                &a,
                "selection digest differs from independent byte oracle",
            ),
            (
                "--expect-candidate-selection",
                &b,
                "candidate selection digest differs",
            ),
        ] {
            let mut wrong_digest = archive.select(0).digest();
            wrong_digest[0] ^= 1;
            let output = command().arg(flag).arg(hex(wrong_digest)).output().unwrap();
            assert!(!output.status.success(), "{output:?}");
            assert!(String::from_utf8_lossy(&output.stderr).contains(message));
        }
        let output = command()
            .arg("--expect")
            .arg(format!("{:.17e}", result.metrics.l2 + 1.0))
            .arg(format!("{:.17e}", result.metrics.rmse))
            .arg(format!("{:.17e}", result.metrics.linf))
            .arg("undefined")
            .output()
            .unwrap();
        assert!(!output.status.success(), "{output:?}");
        assert!(String::from_utf8_lossy(&output.stderr).contains("ValueError: L2:"));
    }
}
#[test]
fn append_and_terminal_status_do_not_change_selected_identity() {
    let mut s = sim();
    let run = carrier_run(
        SessionIdentity {
            run_id: 99,
            source_revision: "append-source".into(),
        },
        &s,
        "append identity",
    )
    .unwrap();
    let archive = Archive::empty();
    let mut writer = StepRecorder::create(&archive.path, run, RecordingLimits::default()).unwrap();
    writer
        .append(&carrier_publication(&s, 0, 1 << 20).unwrap())
        .unwrap();
    let reader = archive.reader();
    let before = Selection::read(&reader, 0).unwrap();
    s.step(0.01, None, |_| false).unwrap();
    writer
        .append(&carrier_publication(&s, 1, 1 << 20).unwrap())
        .unwrap();
    writer.finish("completed", "done", Some(1)).unwrap();
    drop(writer);
    let after = archive.select(0);
    assert_eq!(before.identity(), after.identity());
    assert_eq!(before.run_digest(), after.run_digest());
    assert_eq!(after.run().identity.run_id, 99);
    assert_eq!(after.publication().generation, 0);
}
#[test]
fn digest_changes_with_native_bits_and_source_provenance() {
    let (run, steps) = fixture(0.01, false);
    let a = Archive::new(run, &steps);
    let (run, mut steps) = fixture(0.01, false);
    put_f32(&mut steps[0], "tracer", |_| f32::from_bits(1));
    let b = Archive::new(run, &steps);
    assert_ne!(a.select(0).digest(), b.select(0).digest());
    assert_eq!(a.select(0).run_digest(), b.select(0).run_digest());
    let (mut run, steps) = fixture(0.01, false);
    run.provenance = "different provenance".into();
    let b = Archive::new(run, &steps);
    assert_ne!(a.select(0).run_digest(), b.select(0).run_digest());
}
#[test]
fn tolerance_is_explicit_and_ambiguous_matching_never_picks_by_order() {
    let (run, steps) = fixture(0.01, false);
    let a = Archive::new(run, &steps);
    let mut r = recipe(FieldRole::Tracer);
    r.time_tolerance = f64::NAN;
    assert_eq!(
        compare(&a.select(0), &a.select(0), r),
        Err(RecipeError::InvalidTolerance)
    );
    r.time_tolerance = 0.1;
    assert_eq!(
        align(&[a.select(0)], &[a.select(0), a.select(0)], r),
        Err(RecipeError::AmbiguousAlignment)
    );
    assert_eq!(
        align(&[a.select(0), a.select(0)], &[a.select(0)], r),
        Err(RecipeError::AmbiguousAlignment)
    );
}

#[test]
fn exact_field_schemas_reject_single_mismatches_and_divergence_missing_faces() {
    let (run, steps) = fixture(0.01, false);
    let a = Archive::new(run, &steps);
    for mutation in 0..5 {
        let (run, mut steps) = fixture(0.01, false);
        let field = field_mut(&mut steps[0], "tracer");
        match mutation {
            0 => field.units = "kg".into(),
            1 => field.association = "particle".into(),
            2 => field.dimensions = [3, 2, 4],
            3 => field.evaluation_time = Some(0.0),
            4 => field.stage = "diagnostic_rejected".into(),
            _ => unreachable!(),
        }
        let b = Archive::new(run, &steps);
        assert_eq!(
            compare(&a.select(0), &b.select(0), recipe(FieldRole::Tracer))
                .unwrap()
                .outcome,
            Err(Unavailable::FieldSchema),
            "mutation {mutation}"
        );
    }
    let (run, mut steps) = fixture(0.01, false);
    steps[0].fields.retain(|f| f.name != "velocity_z");
    let b = Archive::new(run, &steps);
    assert_eq!(
        compare(
            &a.select(0),
            &b.select(0),
            recipe(FieldRole::VelocityDivergence)
        )
        .unwrap()
        .outcome,
        Err(Unavailable::MissingField)
    );
}
#[test]
fn finite_source_overflow_is_unavailable_and_nonfinite_source_is_rejected() {
    let (run, mut steps) = fixture(0.01, false);
    put_f64(&mut steps[0], "interval_pressure", |i| {
        if i == 0 { 0.0 } else { f64::MAX }
    });
    let a = Archive::new(run, &steps);
    let (run, mut steps) = fixture(0.01, false);
    put_f64(&mut steps[0], "interval_pressure", |i| {
        if i == 0 { 0.0 } else { -f64::MAX }
    });
    let b = Archive::new(run, &steps);
    assert_eq!(
        compare(
            &a.select(0),
            &b.select(0),
            recipe(FieldRole::IntervalPressure)
        )
        .unwrap()
        .outcome,
        Err(Unavailable::ArithmeticRange)
    );
    // Valid metadata and finite face bytes can still overflow the derived rate.
    // This is a synthetic read-only arithmetic fixture, not accepted physics.
    let (mut run, mut steps) = fixture(0.01, false);
    run.spacing = [1e-300, 1e150, 1e150];
    run.origin = [0.0; 3];
    put_f32(&mut steps[0], "velocity_x", |i| {
        if i % 3 == 0 { -f32::MAX } else { f32::MAX }
    });
    let overflowing = Archive::new(run, &steps);
    assert_eq!(
        native_samples(&overflowing.select(0), FieldRole::VelocityDivergence),
        Err(Unavailable::ArithmeticRange)
    );
    for role in [FieldRole::Tracer, FieldRole::IntervalPressure] {
        let archive = Archive::empty();
        let (run, mut steps) = fixture(0.01, false);
        if role == FieldRole::Tracer {
            put_f32(&mut steps[0], "tracer", |_| f32::NAN);
        } else {
            put_f64(&mut steps[0], "interval_pressure", |_| f64::INFINITY);
        }
        let mut writer =
            StepRecorder::create(&archive.path, run, RecordingLimits::default()).unwrap();
        assert!(matches!(
            writer.append(&steps[0]),
            Err(RecordingError::Invalid("nonfinite field"))
        ));
        assert_eq!(archive.reader().count().unwrap(), 0);
    }
}
#[test]
fn positive_tolerance_is_retained_and_constant_bounds_stay_finite() {
    let (run, steps) = fixture(0.01, false);
    let a = Archive::new(run, &steps);
    let (run, steps) = fixture(0.0101, false);
    let b = Archive::new(run, &steps);
    let mut r = recipe(FieldRole::Tracer);
    r.time_tolerance = 0.001;
    let result = compare(&a.select(0), &b.select(0), r).unwrap();
    assert_eq!(result.recipe, r);
    let metrics = result.outcome.unwrap();
    assert_eq!(metrics.metrics.relative_l2, None);
    assert_eq!(metrics.metrics.l2, 0.0);
    assert_eq!(metrics.color_bounds, ColorBounds { min: 0.0, max: 0.0 });
    assert_eq!(
        align(&[a.select(0)], &[b.select(0)], r)
            .unwrap()
            .pairs
            .len(),
        1
    );
}

#[test]
fn real_jacobi_and_sgs_cold_archives_compare_common_source_and_unmatched_horizon() {
    let record = |method, intervals| {
        let template = sim();
        let mut native = Simulation::with_implementation(
            GridGeometry::new([2, 3, 4], [0.25, 0.5, 0.75], [-1.0, 2.0, 3.0]).unwrap(),
            template.config(),
            method,
        )
        .unwrap();
        let run = carrier_run(
            SessionIdentity {
                run_id: if method == PressureImplementation::JacobiPcgV1 {
                    101
                } else {
                    102
                },
                source_revision: "real-native-comparison-test-source".into(),
            },
            &native,
            "actual CPU native sealed-box source; no manufactured field mutation",
        )
        .unwrap();
        let source = rheon::SmokeSource {
            lower: [-0.99, 2.1, 3.1],
            upper: [-0.51, 2.9, 4.4],
            tracer_rate: 2.0,
            vertical_acceleration: 1.0,
        };
        let mut steps = Vec::new();
        for ordinal in 0..intervals {
            native.step(0.003, Some(source), |_| false).unwrap();
            steps.push(carrier_publication(&native, ordinal, 1 << 20).unwrap());
        }
        drop(native);
        Archive::new(run, &steps)
    };
    let jacobi = record(PressureImplementation::JacobiPcgV1, 3);
    let sgs = record(PressureImplementation::SymmetricGaussSeidelPcgV1, 2);
    let a = jacobi.reader();
    let b = sgs.reader();
    a.verify().unwrap();
    b.verify().unwrap();
    assert_eq!(a.status().unwrap().outcome, "completed");
    assert_eq!(b.status().unwrap().outcome, "completed");
    let reference: Vec<_> = (0..3).map(|i| Selection::read(&a, i).unwrap()).collect();
    let candidate: Vec<_> = (0..2).map(|i| Selection::read(&b, i).unwrap()).collect();
    let coverage = align(&reference, &candidate, recipe(FieldRole::IntervalPressure)).unwrap();
    assert_eq!(coverage.pairs.len(), 2);
    assert_eq!(
        coverage.reference_unmatched,
        vec![reference[2].identity().clone()]
    );
    assert!(coverage.candidate_unmatched.is_empty());
    assert_eq!(coverage.pairs[0].reference, *reference[0].identity());
    assert_eq!(coverage.pairs[0].candidate, *candidate[0].identity());
    assert_ne!(reference[0].digest(), candidate[0].digest());
    let decoded = RecordedCarrier::decode(a.run(), reference[0].publication()).unwrap();
    assert!(decoded.tracer.iter().any(|v| *v > 0.0));
    assert!(
        decoded
            .interval_pressure
            .as_ref()
            .unwrap()
            .1
            .iter()
            .any(|v| *v != 0.0)
    );
    for role in [
        FieldRole::Tracer,
        FieldRole::IntervalPressure,
        FieldRole::VelocityX,
        FieldRole::VelocityY,
        FieldRole::VelocityZ,
        FieldRole::VelocityDivergence,
    ] {
        let result = available(&reference[0], &candidate[0], role);
        assert!(result.metrics.support_count > 0);
        oracle(&jacobi, &sgs, role, &result);
    }
}

#[test]
fn malformed_pressure_interval_remains_a_paired_schema_failure() {
    let (run, steps) = fixture(0.01, false);
    let a = Archive::new(run, &steps);
    let (run, mut steps) = fixture(0.01, false);
    field_mut(&mut steps[0], "interval_pressure").evaluation_time = Some(0.005);
    let b = Archive::new(run, &steps);
    assert_eq!(
        compare(
            &a.select(0),
            &b.select(0),
            recipe(FieldRole::IntervalPressure)
        )
        .unwrap()
        .outcome,
        Err(Unavailable::FieldSchema)
    );
    let coverage = align(
        &[a.select(0)],
        &[b.select(0)],
        recipe(FieldRole::IntervalPressure),
    )
    .unwrap();
    assert_eq!(coverage.pairs.len(), 1);
    assert_eq!(coverage.pairs[0].outcome, Err(Unavailable::FieldSchema));
    assert!(coverage.reference_unmatched.is_empty());
    assert!(coverage.candidate_unmatched.is_empty());
}

fn presentation_archive(method: PressureImplementation) -> Archive {
    let mut native = Simulation::with_implementation(
        GridGeometry::new([2, 3, 4], [0.25, 0.5, 0.75], [-1.0, 2.0, 3.0]).unwrap(),
        sim().config(),
        method,
    )
    .unwrap();
    let run = carrier_run(
        SessionIdentity {
            run_id: if method == PressureImplementation::JacobiPcgV1 {
                301
            } else {
                302
            },
            source_revision: "actual-native-presentation-source".into(),
        },
        &native,
        "native fields; no presentation remapping",
    )
    .unwrap();
    native
        .step(
            0.003,
            Some(rheon::SmokeSource {
                lower: [-0.99, 2.1, 3.1],
                upper: [-0.51, 2.9, 4.4],
                tracer_rate: 2.0,
                vertical_acceleration: 1.0,
            }),
            |_| false,
        )
        .unwrap();
    Archive::new(run, &[carrier_publication(&native, 0, 1 << 20).unwrap()])
}
fn native_bits(s: &Selection, name: &str) -> Vec<u64> {
    let field = s.publication().field(name).unwrap();
    if field.scalar_type == "f64" {
        field
            .data
            .chunks_exact(8)
            .map(|b| f64::from_le_bytes(b.try_into().unwrap()).to_bits())
            .collect()
    } else {
        field
            .data
            .chunks_exact(4)
            .map(|b| (f32::from_le_bytes(b.try_into().unwrap()) as f64).to_bits())
            .collect()
    }
}
fn value_oracle_status(
    a: &Archive,
    b: &Archive,
    role: FieldRole,
    data: &ComparedValues,
    diagnostic: bool,
    mode: u8,
    perturb: bool,
) -> bool {
    use std::io::Write;
    use std::process::Stdio;
    // Independent coordinate dictionaries, IEEE unpacking and fsum; this adds
    // pointwise evidence without changing the existing metric oracle.
    let script = r#"
import math, sqlite3, struct, sys

def samples(path, role):
    with sqlite3.connect(path) as db:
        geometry=db.execute('SELECT geometry FROM run WHERE id=1').fetchone()[0]
        fields={name:(kind,raw) for name,kind,raw in db.execute('SELECT name,scalar_type,data FROM fields WHERE ordinal=0')}
    values={name:[v[0] for v in struct.iter_unpack('<f' if kind=='f32' else '<d',raw)] for name,(kind,raw) in fields.items()}
    if role!='divergence': return values[role], [0.0]*len(values[role])
    nx,ny,nz=struct.unpack('<3Q',geometry[:24]);spacing=struct.unpack('<3d',geometry[24:48])
    faces=[]
    for axis,name in enumerate(('velocity_x','velocity_y','velocity_z')):
        shape=[nx,ny,nz];shape[axis]+=1
        coords=((i,j,k) for k in range(shape[2]) for j in range(shape[1]) for i in range(shape[0]))
        faces.append(dict(zip(coords,values[name],strict=True)))
    result=[];bounds=[]
    for k in range(nz):
        for j in range(ny):
            for i in range(nx):
                lower=(i,j,k);terms=[]
                for axis in range(3):
                    upper=list(lower);upper[axis]+=1
                    terms.append((faces[axis][tuple(upper)]-faces[axis][lower])/spacing[axis])
                result.append(math.fsum(terms));bounds.append(4*math.ulp(1.0)*math.fsum(map(abs,terms))+4*math.ulp(0.0))
    return result,bounds

x,xb=samples(sys.argv[1],sys.argv[3]);y,yb=samples(sys.argv[2],sys.argv[3])
if sys.argv[4]=='diagnostic':
    offset=math.fsum(b-a for a,b in zip(x,y,strict=True))/len(x)
    y=[v-offset for v in y];yb=[1e-13]*len(y)
actual=[v[0] for v in struct.iter_unpack('<d',sys.stdin.buffer.read())]
if len(actual)!=len(x)+len(y): raise ValueError('native sample count')
for actual,expected,bound in zip(actual,x+y,xb+yb,strict=True):
    if not all(map(math.isfinite,(actual,expected,bound))): raise ValueError('nonfinite native sample')
    if bound==0:
        if struct.pack('<d',actual)!=struct.pack('<d',expected): raise ValueError(('native bits',actual,expected))
    elif abs(actual-expected)>bound: raise ValueError(('native value bound',actual,expected,bound))
print('independent native presentation values passed:',sys.argv[3])
"#;
    let name = match role {
        FieldRole::Tracer => "tracer",
        FieldRole::IntervalPressure => "interval_pressure",
        FieldRole::VelocityX => "velocity_x",
        FieldRole::VelocityY => "velocity_y",
        FieldRole::VelocityZ => "velocity_z",
        FieldRole::VelocityDivergence => "divergence",
    };
    let mut command = std::process::Command::new("python3");
    command.env_remove("PYTHONOPTIMIZE");
    if mode == 1 {
        command.arg("-O");
    } else if mode == 2 {
        command.env("PYTHONOPTIMIZE", "1");
    }
    let mut child = command
        .args(["-c", script])
        .arg(&a.path)
        .arg(&b.path)
        .arg(name)
        .arg(if diagnostic { "diagnostic" } else { "direct" })
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut bytes: Vec<_> = data
        .reference
        .iter()
        .chain(&data.candidate)
        .flat_map(|v| v.to_le_bytes())
        .collect();
    if perturb {
        let original = f64::from_le_bytes(bytes[..8].try_into().unwrap());
        bytes[..8].copy_from_slice(&(original + 1.0).to_le_bytes());
    }
    child.stdin.take().unwrap().write_all(&bytes).unwrap();
    child.wait().unwrap().success()
}
fn value_oracle(
    a: &Archive,
    b: &Archive,
    role: FieldRole,
    data: &ComparedValues,
    diagnostic: bool,
) {
    assert!(value_oracle_status(a, b, role, data, diagnostic, 0, false));
}
#[test]
fn independent_pointwise_oracle_rejects_changed_samples_with_python_optimization() {
    let a = presentation_archive(PressureImplementation::JacobiPcgV1);
    let b = presentation_archive(PressureImplementation::SymmetricGaussSeidelPcgV1);
    let role = FieldRole::Tracer;
    let data = compared_values(&a.select(0), &b.select(0), recipe(role)).unwrap();
    for mode in 0..3 {
        assert!(value_oracle_status(&a, &b, role, &data, false, mode, false));
        assert!(!value_oracle_status(&a, &b, role, &data, false, mode, true));
    }
}
#[test]
fn presentation_preserves_all_real_native_arrays_sampling_and_existing_oracle_metrics() {
    let a = presentation_archive(PressureImplementation::JacobiPcgV1);
    let b = presentation_archive(PressureImplementation::SymmetricGaussSeidelPcgV1);
    let reference = a.select(0);
    let candidate = b.select(0);
    for (role, association, dimensions, offset, name) in [
        (
            FieldRole::Tracer,
            SampleAssociation::CellCenter,
            [2, 3, 4],
            [0.5, 0.5, 0.5],
            Some("tracer"),
        ),
        (
            FieldRole::IntervalPressure,
            SampleAssociation::CellCenter,
            [2, 3, 4],
            [0.5, 0.5, 0.5],
            Some("interval_pressure"),
        ),
        (
            FieldRole::VelocityX,
            SampleAssociation::XFace,
            [3, 3, 4],
            [0.0, 0.5, 0.5],
            Some("velocity_x"),
        ),
        (
            FieldRole::VelocityY,
            SampleAssociation::YFace,
            [2, 4, 4],
            [0.5, 0.0, 0.5],
            Some("velocity_y"),
        ),
        (
            FieldRole::VelocityZ,
            SampleAssociation::ZFace,
            [2, 3, 5],
            [0.5, 0.5, 0.0],
            Some("velocity_z"),
        ),
        (
            FieldRole::VelocityDivergence,
            SampleAssociation::CellCenter,
            [2, 3, 4],
            [0.5, 0.5, 0.5],
            None,
        ),
    ] {
        let data = compared_values(&reference, &candidate, recipe(role)).unwrap();
        assert_eq!(
            data.comparison,
            compare(&reference, &candidate, recipe(role)).unwrap()
        );
        assert_eq!(
            data.geometry,
            NativeGeometry {
                counts_xyz: [2, 3, 4],
                spacing_m: [0.25, 0.5, 0.75],
                origin_m: [-1.0, 2.0, 3.0]
            }
        );
        assert_eq!(
            data.sampling,
            NativeSampling {
                association,
                dimensions,
                offset_cells: offset
            }
        );
        assert_eq!(
            data.reference.len(),
            dimensions.into_iter().product::<u64>() as usize
        );
        assert_eq!(data.candidate.len(), data.reference.len());
        if let Some(name) = name {
            assert_eq!(
                data.reference
                    .iter()
                    .map(|v| v.to_bits())
                    .collect::<Vec<_>>(),
                native_bits(&reference, name)
            );
            assert_eq!(
                data.candidate
                    .iter()
                    .map(|v| v.to_bits())
                    .collect::<Vec<_>>(),
                native_bits(&candidate, name)
            );
        }
        oracle(&a, &b, role, data.comparison.outcome.as_ref().unwrap());
        value_oracle(&a, &b, role, &data, false);
    }
}
#[test]
fn presentation_keeps_native_bits_and_uses_only_declared_pressure_offset_and_divergence() {
    let (run, mut steps) = fixture(0.01, false);
    put_f32(&mut steps[0], "tracer", |i| match i % 3 {
        0 => -0.0,
        1 => f32::from_bits(1),
        _ => f32::from_bits(0.25_f32.to_bits() + 1),
    });
    put_f64(&mut steps[0], "interval_pressure", |i| i as f64);
    analytic_faces(&mut steps[0], 1.0);
    let a = Archive::new(run, &steps);
    let (run, mut steps) = fixture(0.01, false);
    put_f64(&mut steps[0], "interval_pressure", |i| i as f64 + 8.0);
    analytic_faces(&mut steps[0], 2.0);
    let b = Archive::new(run, &steps);
    let x = a.select(0);
    let y = b.select(0);
    let tracer = compared_values(&x, &y, recipe(FieldRole::Tracer)).unwrap();
    assert_eq!(tracer.reference[0].to_bits(), (-0.0_f64).to_bits());
    assert_eq!(tracer.reference[1], f32::from_bits(1) as f64);
    value_oracle(&a, &b, FieldRole::Tracer, &tracer, false);
    let mut r = recipe(FieldRole::IntervalPressure);
    r.pressure = PressurePolicy::RemoveMeanOffsetDiagnostic;
    let pressure = compared_values(&x, &y, r).unwrap();
    let summary = pressure.comparison.outcome.as_ref().unwrap();
    let gauge = summary.gauge.unwrap();
    assert!(gauge.reference_canonical);
    assert!(!gauge.candidate_canonical);
    assert!(gauge.diagnostic);
    assert_eq!(gauge.removed_offset_pa, 8.0);
    assert_eq!(pressure.reference, pressure.candidate);
    assert_eq!(
        summary.color_bounds,
        ColorBounds {
            min: 0.0,
            max: 23.0
        }
    );
    value_oracle(&a, &b, FieldRole::IntervalPressure, &pressure, true);
    let divergence = compared_values(&x, &y, recipe(FieldRole::VelocityDivergence)).unwrap();
    assert!(divergence.reference.iter().all(|v| *v == 2.0));
    assert!(divergence.candidate.iter().all(|v| *v == 4.0));
    value_oracle(&a, &b, FieldRole::VelocityDivergence, &divergence, false);
}
#[test]
fn presentation_refuses_recipe_and_native_pair_mutations_with_original_failure_meaning() {
    let (run, steps) = fixture(0.01, true);
    let a = Archive::new(run, &steps);
    let reference = a.select(1);
    let mut invalid = recipe(FieldRole::Tracer);
    invalid.time_tolerance = f64::NAN;
    assert_eq!(
        compared_values(&reference, &reference, invalid).unwrap_err(),
        ComparedValuesError::Recipe(RecipeError::InvalidTolerance)
    );
    assert_eq!(
        compared_values(&a.select(0), &reference, recipe(FieldRole::Tracer)).unwrap_err(),
        ComparedValuesError::Unavailable(Unavailable::PublicationStage)
    );
    for mutation in 0..7 {
        let (mut run, mut steps) = fixture(0.01, false);
        let mut role = FieldRole::Tracer;
        let expected = match mutation {
            0 => {
                run.solver = "future-profile/unknown".into();
                Unavailable::UnsupportedProfile
            }
            1 => {
                run.spacing[0] *= 2.0;
                Unavailable::Geometry
            }
            2 => {
                steps[0].time = 0.02;
                steps[0].dt = Some(0.02);
                Unavailable::PhysicalTime
            }
            3 => {
                steps[0].fields.retain(|f| f.name != "tracer");
                Unavailable::MissingField
            }
            4 => {
                role = FieldRole::VelocityX;
                field_mut(&mut steps[0], "velocity_x").association = "cell_center".into();
                Unavailable::FieldSchema
            }
            5 => {
                role = FieldRole::IntervalPressure;
                put_f64(&mut steps[0], "interval_pressure", |_| 1.0);
                Unavailable::NoncanonicalGauge
            }
            6 => {
                role = FieldRole::IntervalPressure;
                field_mut(&mut steps[0], "interval_pressure").evaluation_time = Some(0.005);
                Unavailable::FieldSchema
            }
            _ => unreachable!(),
        };
        let b = Archive::new(run, &steps);
        assert_eq!(
            compared_values(&reference, &b.select(0), recipe(role)).unwrap_err(),
            ComparedValuesError::Unavailable(expected),
            "mutation {mutation}"
        );
    }
    let (run, mut steps) = fixture(0.01, false);
    put_f64(&mut steps[0], "interval_pressure", |i| {
        if i == 0 { 0.0 } else { f64::MAX }
    });
    let a = Archive::new(run, &steps);
    let (run, mut steps) = fixture(0.01, false);
    put_f64(&mut steps[0], "interval_pressure", |i| {
        if i == 0 { 0.0 } else { -f64::MAX }
    });
    let b = Archive::new(run, &steps);
    assert_eq!(
        compared_values(
            &a.select(0),
            &b.select(0),
            recipe(FieldRole::IntervalPressure)
        )
        .unwrap_err(),
        ComparedValuesError::Unavailable(Unavailable::ArithmeticRange)
    );
}
