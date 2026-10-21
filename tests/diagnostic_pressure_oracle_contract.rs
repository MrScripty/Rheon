//! Numeric-range regression for the independent diagnostic-pressure oracle.
//! Real schema-2 recorder/reader custody is exercised; the consumed manifest is
//! deliberately a minimal oracle wrapper, not FrozenRunSet admission evidence.
//! Pressures and host qualification observations are manufactured storage facts,
//! without a model-runtime, hardware, or scientific-qualification claim.
#![cfg(all(feature = "desktop", not(target_arch = "wasm32")))]

use rheon::{
    GridGeometry,
    recording::{RecordingLimits, diagnostic::*},
    solver_provider::context::{
        PressureContext, PressureContextInput, PressureContextMode, PressureHistoryStatus,
    },
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "rheon-diagnostic-pressure-oracle-{}-{}",
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

fn sample(pressure: Vec<f64>, qualified: bool) -> PressureSample {
    PressureSample {
        pressure,
        observation: QualificationObservation {
            policy: QualificationPolicy::HostPressureGateV1,
            residual_l2: 0.0,
            relative_residual: Some(0.0),
            predicted_divergence: 0.0,
            actual_divergence: 0.0,
            courant: 0.0,
            net_flux: 0.0,
            relative_limit: 1e-6,
            absolute_limit: 1e-12,
            predicted_limit: 1e-7,
            actual_limit: 1e-5,
            courant_limit: 0.5,
            flux_limit: 1e-12,
            qualified,
            hard_stop: false,
            walls_closed: true,
        },
    }
}

fn record_summary(root: &Path, scale: f64) -> Value {
    let run = DiagnosticRun {
        run_id: "finite-range-storage-fixture".into(),
        source_revision: "diagnostic-pressure-oracle-contract-v1".into(),
        grid: GridGeometry::new([2, 3, 4], [0.25, 0.5, 0.75], [-1.0, 2.0, 3.0]).unwrap(),
        density: 1000.0,
        viscosity: 0.001,
        dt: 0.01,
        mode: DiagnosticMode::TeacherStage,
        native_prefix_steps: 0,
        binding: BindingIdentity {
            id: "manufactured-storage-provider".into(),
            version: "1".into(),
            manifest_sha256: [7; 32],
            checkpoint_sha256: None,
            training_run_id: None,
        },
        provenance: "manufactured native/candidate pressure and host observation; no runtime claim"
            .into(),
        seed_origin: None,
        seed: None,
    };
    let n = run.grid.cell_len();
    let rhs = vec![0.0; n];
    let context = PressureContext::new(PressureContextInput {
        grid: &run.grid,
        density: run.density,
        dt: run.dt,
        interval_start: 0.0,
        rhs: &rhs,
        prior: None,
        run_id: &run.run_id,
        generation: 1,
        mode: PressureContextMode::DiagnosticCandidate,
    })
    .unwrap();
    let request = DiagnosticRequest::capture(&context, run.viscosity, 1 << 20).unwrap();
    let native: Vec<_> = (0..n).map(|i| i as f64 * scale).collect();
    let candidate: Vec<_> = native.iter().map(|x| 2.0 * x).collect();
    let attempt = DiagnosticAttempt {
        attempt_index: 0,
        native_reference: Some(NativeReference {
            request_digest: request.digest(),
            sample: sample(native.clone(), true),
            history_status: PressureHistoryStatus::Diagnostic { qualified: true },
        }),
        request,
        dispatched: true,
        outcome: PredictionOutcome::Candidate(sample(candidate.clone(), false)),
        transition: HistoryTransition::NativeTeacherAdvanced,
    };
    let archive = root.join("diagnostic.sqlite");
    let mut writer = DiagnosticRecorder::create(&archive, run, RecordingLimits::default()).unwrap();
    writer.append(&attempt).unwrap();
    writer
        .finish(DiagnosticTerminal {
            outcome: TerminalOutcome::Completed,
            detail: "closed manufactured finite-range oracle fixture".into(),
            produced_attempts: 1,
            last_advanced_generation: Some(attempt.request.generation()),
        })
        .unwrap();
    drop(writer);

    let reader = DiagnosticReader::open(&archive, RecordingLimits::default()).unwrap();
    reader.verify().unwrap();
    assert_eq!(reader.count().unwrap(), 1);
    let cold = reader.select(0).unwrap();
    assert_eq!(
        cold.native_reference.as_ref().unwrap().sample.pressure,
        native
    );
    let PredictionOutcome::Candidate(cold_candidate) = &cold.outcome else {
        panic!("cold candidate absent")
    };
    assert_eq!(cold_candidate.pressure, candidate);
    let comparison = cold.pressure_comparison().unwrap();
    assert_eq!(comparison, attempt.pressure_comparison().unwrap());
    let metrics = &comparison.metrics;
    // Sum(i^2, i=0..n-1) is an integer closed form. No streaming norm or
    // square of the dimensional extreme is used to construct this expectation.
    let sum_squares = n * (n - 1) * (2 * n - 1) / 6;
    assert_eq!(sum_squares, 4324);
    let expected_l2 = scale * (sum_squares as f64).sqrt();
    let expected_rmse = scale * (sum_squares as f64 / n as f64).sqrt();
    let expected_linf = (n - 1) as f64 * scale;
    for (label, actual, expected) in [
        ("l2", metrics.l2, expected_l2),
        ("rmse", metrics.rmse, expected_rmse),
        ("linf", metrics.linf, expected_linf),
    ] {
        assert!(actual.is_finite(), "{label}: finite {scale:e}");
        if expected == 0.0 {
            assert_eq!(actual, 0.0, "{label}: zero fixture");
        } else {
            assert!(
                (actual / expected - 1.0).abs() < 5e-14,
                "{label}: {actual:e} != closed form {expected:e}"
            );
        }
    }
    assert_eq!(metrics.support_count, n);
    assert_eq!(
        metrics.relative_l2,
        if scale == 0.0 { None } else { Some(1.0) }
    );
    assert_eq!(comparison.units, "Pa");
    assert_eq!(comparison.color_bounds.min, 0.0);
    assert_eq!(comparison.color_bounds.max, 2.0 * expected_linf);
    drop(reader);
    let archive_sha: [u8; 32] = Sha256::digest(std::fs::read(&archive).unwrap()).into();
    // The Python oracle consumes these exact fields only. This wrapper does not
    // impersonate full application manifest/admission validation.
    let run_set_sha = [3u8; 32];
    let manifest = json!({"sha256":run_set_sha,"run_set":{"members":[{
        "member_id":"fixture","run_id":"finite-range-storage-fixture","archive_file":"diagnostic.sqlite",
        "finalized":{"sha256":archive_sha}
    }]}});
    std::fs::write(
        root.join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    json!({
        "schema":"rheon-diagnostic-pressure-comparison-v1","analysis_version":1,
        "run_set_file":"manifest.json","run_set_sha256":run_set_sha,"member_id":"fixture",
        "run_id":"finite-range-storage-fixture","archive_sha256":archive_sha,
        "attempt_index":0,"request_sha256":cold.request.digest(),"counts":[2,3,4],
        "interval_start":cold.request.interval_start(),"interval_end":cold.request.interval_end(),"dt":cold.request.dt(),
        "comparison":comparison,"unavailable":null
    })
}

#[derive(Debug, Clone, Copy)]
enum PythonMode {
    Normal,
    FlagOptimized,
    EnvironmentOptimized,
}
impl PythonMode {
    fn run(self, oracle: &Path, summary: &Path) -> Output {
        let mut command = Command::new("python3");
        command.env_remove("PYTHONOPTIMIZE");
        match self {
            Self::Normal => {}
            Self::FlagOptimized => {
                command.arg("-O");
            }
            Self::EnvironmentOptimized => {
                command.env("PYTHONOPTIMIZE", "1");
            }
        }
        command
            .arg(oracle)
            .arg(summary)
            .output()
            .expect("Python3 oracle must execute")
    }
}

#[test]
fn closed_schema2_extremes_match_analytic_norms_and_oracle_in_every_python_mode() {
    let oracle = Path::new(env!("CARGO_MANIFEST_DIR")).join("tools/verify_diagnostic_pressure.py");
    for scale in [1e-200, 1e200, 0.125, 0.0] {
        let root = Directory::new();
        let summary = record_summary(&root.0, scale);
        let summary_path = root.0.join("summary.json");
        for mode in [
            PythonMode::Normal,
            PythonMode::FlagOptimized,
            PythonMode::EnvironmentOptimized,
        ] {
            std::fs::write(&summary_path, serde_json::to_vec(&summary).unwrap()).unwrap();
            let result = mode.run(&oracle, &summary_path);
            assert!(
                result.status.success(),
                "scale={scale:e} mode={mode:?}: {}",
                String::from_utf8_lossy(&result.stderr)
            );
            assert!(String::from_utf8_lossy(&result.stdout).contains("oracle passed: 24 cells"));
            for metric in ["l2", "rmse", "linf", "relative_l2", "support_count"] {
                let mut damaged = summary.clone();
                let original = damaged["comparison"]["metrics"][metric].as_f64();
                damaged["comparison"]["metrics"][metric] = if metric == "support_count" {
                    json!(25)
                } else {
                    json!(original.filter(|v| *v != 0.0).map_or(1.0, |v| v * 1.25))
                };
                std::fs::write(&summary_path, serde_json::to_vec(&damaged).unwrap()).unwrap();
                let result = mode.run(&oracle, &summary_path);
                assert!(
                    !result.status.success(),
                    "damaged {metric} accepted: scale={scale:e} mode={mode:?}"
                );
                assert!(
                    String::from_utf8_lossy(&result.stderr)
                        .contains(&format!("ValueError: metric {metric}")),
                    "wrong failure for {metric},scale={scale:e},mode={mode:?}: {}",
                    String::from_utf8_lossy(&result.stderr)
                );
            }
        }
    }
}
