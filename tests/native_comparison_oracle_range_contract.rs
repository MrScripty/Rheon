//! Independent native-bit oracle range checks. Mutated pressures are analytic
//! storage/analysis fixtures, not evidence of physical acceptance or accuracy.
#![cfg(not(target_arch = "wasm32"))]

use rheon::recording::comparison::{
    FieldComparison, FieldRole, PressurePolicy, Recipe, Selection, compare, native_samples,
};
use rheon::recording::{
    RecordedStep, RecordingLimits, RecordingReader, RecordingRun, StepRecorder,
    carrier_publication, carrier_run,
};
use rheon::{GridGeometry, SessionIdentity, Simulation, SimulationConfig, SnapshotStage};
use std::{
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

struct Archive {
    path: PathBuf,
    selection: Selection,
}

impl Archive {
    fn closed(run: RecordingRun, step: RecordedStep) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "rheon-native-oracle-range-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("archive.sqlite");
        {
            let mut writer = StepRecorder::create(&path, run, RecordingLimits::default()).unwrap();
            writer.append(&step).unwrap();
            writer
                .finish(
                    "completed",
                    "native oracle range fixture",
                    Some(step.generation),
                )
                .unwrap();
        }
        // Fresh reader after writer closure: exercise actual persisted identities.
        let reader = RecordingReader::open(&path, RecordingLimits::default()).unwrap();
        reader.verify().unwrap();
        assert_eq!(reader.count().unwrap(), 1);
        let selection = Selection::read(&reader, 0).unwrap();
        assert_eq!(selection.publication().generation, step.generation);
        assert_eq!(selection.publication().dt, step.dt);
        assert_eq!(selection.publication().fields.len(), step.fields.len());
        for (actual, recorded) in selection.publication().fields.iter().zip(&step.fields) {
            assert_eq!(actual.name, recorded.name);
            assert_eq!(actual.data, recorded.data);
        }
        assert_eq!(selection.publication().stage, SnapshotStage::Accepted);
        assert_eq!(selection.publication().time, 0.01);
        assert_eq!(selection.identity().ordinal, 0);
        assert_eq!(
            selection.digest(),
            Selection::read(&reader, 0).unwrap().digest()
        );
        Self { path, selection }
    }
}

impl Drop for Archive {
    fn drop(&mut self) {
        std::fs::remove_dir_all(self.path.parent().unwrap()).unwrap();
    }
}

fn simulation(density: f64) -> Simulation {
    Simulation::new(
        GridGeometry::new([2, 3, 4], [0.25, 0.5, 0.75], [-1.0, 2.0, 3.0]).unwrap(),
        SimulationConfig {
            density,
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

fn publication(simulation: &Simulation, run_id: u64) -> (RecordingRun, RecordedStep) {
    (
        carrier_run(
            SessionIdentity {
                run_id,
                source_revision: "native-oracle-range-contract".into(),
            },
            simulation,
            "Bounded numerical range fixture; no physical accuracy claim",
        )
        .unwrap(),
        carrier_publication(simulation, 0, 1 << 20).unwrap(),
    )
}

fn analytic(scale: f64, multiplier: f64) -> Archive {
    let mut simulation = simulation(1.0);
    simulation.step(0.01, None, |_| false).unwrap();
    let (run, mut step) = publication(&simulation, multiplier as u64);
    let pressure = step
        .fields
        .iter_mut()
        .find(|field| field.name == "interval_pressure")
        .unwrap();
    assert_eq!(pressure.scalar_type, "f64");
    assert_eq!(pressure.data.len(), 24 * 8);
    // Deliberately replace only pressure bytes before recording this analytic case.
    pressure.data = (0..24)
        .flat_map(|i| (multiplier * (i as f64 * scale)).to_le_bytes())
        .collect();
    let archive = Archive::closed(run, step);
    let actual = native_samples(&archive.selection, FieldRole::IntervalPressure).unwrap();
    for (i, actual) in actual.into_iter().enumerate() {
        assert_eq!(
            actual.to_bits(),
            (multiplier * (i as f64 * scale)).to_bits()
        );
    }
    archive
}

fn comparison(reference: &Archive, candidate: &Archive) -> FieldComparison {
    let result = compare(
        &reference.selection,
        &candidate.selection,
        Recipe {
            field: FieldRole::IntervalPressure,
            time_tolerance: 0.0,
            pressure: PressurePolicy::Canonical,
        },
    )
    .unwrap()
    .outcome
    .unwrap();
    assert_eq!(result.metrics.support_count, 24);
    assert!(
        [result.metrics.l2, result.metrics.rmse, result.metrics.linf]
            .into_iter()
            .all(f64::is_finite)
    );
    result
}

fn expected(comparison: &FieldComparison) -> [String; 4] {
    let metrics = comparison.metrics;
    [
        metrics.l2.to_string(),
        metrics.rmse.to_string(),
        metrics.linf.to_string(),
        metrics
            .relative_l2
            .map_or("undefined".into(), |v| v.to_string()),
    ]
}

fn hex(digest: [u8; 32]) -> String {
    digest.iter().map(|v| format!("{v:02x}")).collect()
}

fn invoke(reference: &Archive, candidate: &Archive, mode: usize, values: &[String; 4]) -> Output {
    let mut command = Command::new("python3");
    command.env_remove("PYTHONOPTIMIZE");
    match mode {
        0 => {}
        1 => {
            command.arg("-O");
        }
        2 => {
            command.env("PYTHONOPTIMIZE", "1");
        }
        _ => unreachable!(),
    }
    command
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tools/verify_recording_comparison.py"
        ))
        .arg(&reference.path)
        .arg(&candidate.path)
        .args(["--field", "interval_pressure", "--expect"])
        .args(values)
        .arg("--expect-selection")
        .arg(hex(reference.selection.digest()))
        .arg("--expect-candidate-selection")
        .arg(hex(candidate.selection.digest()));
    command.output().unwrap()
}

fn check_modes(reference: &Archive, candidate: &Archive, values: [String; 4]) {
    for mode in 0..3 {
        let output = invoke(reference, candidate, mode, &values);
        assert!(
            output.status.success(),
            "mode {mode}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("24 samples"));
        for (index, diagnostic) in ["L2", "RMSE", "Linf", "relative L2"]
            .into_iter()
            .enumerate()
        {
            let mut wrong = values.clone();
            wrong[index] = match wrong[index].parse::<f64>() {
                // Perturb outside the existing 1e-14 absolute tolerance even for
                // zero/tiny norms; +1 alone would round away at large magnitudes.
                Ok(value) => (value + (value.abs() * 0.25).max(0.25)).to_string(),
                Err(_) => "1".into(),
            };
            let output = invoke(reference, candidate, mode, &wrong);
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                !output.status.success(),
                "mode {mode}: accepted wrong {diagnostic}"
            );
            assert!(
                stderr.contains(&format!("ValueError: {diagnostic}:")),
                "mode {mode}: {stderr}"
            );
        }
    }
}

#[test]
fn analytic_closed_archive_norms_cover_tiny_large_ordinary_and_zero() {
    for scale in [1e-200, 1e200, 0.125, 0.0] {
        let reference = analytic(scale, 1.0);
        let candidate = analytic(scale, 2.0);
        let result = comparison(&reference, &candidate);
        // Closed-form sum i^2, i=0..23. No second implementation of the reducer.
        let sum_squares = 24 * 23 * 47 / 6;
        assert_eq!(sum_squares, 4324);
        let analytic = [
            scale * (sum_squares as f64).sqrt(),
            scale * (sum_squares as f64 / 24.0).sqrt(),
            scale * 23.0,
        ];
        for (actual, expected) in [result.metrics.l2, result.metrics.rmse, result.metrics.linf]
            .into_iter()
            .zip(analytic)
        {
            if scale == 0.0 {
                assert_eq!(actual, 0.0);
            } else {
                assert!(actual > 0.0);
                assert!(
                    (actual / expected - 1.0).abs() < 5e-14,
                    "{actual} vs {expected}"
                );
            }
        }
        assert_eq!(
            result.metrics.relative_l2,
            if scale == 0.0 { None } else { Some(1.0) }
        );
        let values = [
            analytic[0].to_string(),
            analytic[1].to_string(),
            analytic[2].to_string(),
            if scale == 0.0 { "undefined" } else { "1" }.into(),
        ];
        check_modes(&reference, &candidate, values);
    }
}

#[test]
fn unmodified_native_extreme_density_pressures_are_oracle_checkable() {
    // Unchanged native gates and source; numerical-range stress, not an accuracy
    // benchmark or a claim about physical applicability of these densities.
    let archives: Vec<_> = [1e200, 2e200]
        .into_iter()
        .enumerate()
        .map(|(i, density)| {
            let mut simulation = simulation(density);
            simulation
                .step(
                    0.01,
                    Some(rheon::SmokeSource {
                        lower: [-0.99, 2.1, 3.1],
                        upper: [-0.51, 2.9, 4.4],
                        tracer_rate: 2.0,
                        vertical_acceleration: 1.0,
                    }),
                    |_| false,
                )
                .unwrap();
            let (run, step) = publication(&simulation, 901 + i as u64);
            Archive::closed(run, step)
        })
        .collect();
    let x = native_samples(&archives[0].selection, FieldRole::IntervalPressure).unwrap();
    let y = native_samples(&archives[1].selection, FieldRole::IntervalPressure).unwrap();
    assert!(x.iter().any(|v| *v != 0.0));
    for (x, y) in x.iter().zip(y) {
        assert_eq!((2.0 * x).to_bits(), y.to_bits());
    }
    let result = comparison(&archives[0], &archives[1]);
    assert!(result.metrics.l2 > 1e199);
    assert_eq!(result.metrics.relative_l2, Some(1.0));
    check_modes(&archives[0], &archives[1], expected(&result));
}
