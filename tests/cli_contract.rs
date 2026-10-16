#![cfg(feature = "png-export")]
use std::{
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
fn directory(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rheon-test-{label}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ))
}
fn run(dir: &PathBuf, extra: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_rheon"))
        .args(["--size", "4", "--steps", "3", "--output"])
        .arg(dir)
        .args(extra)
        .output()
        .unwrap()
}
#[test]
fn executable_exports_reproducible_png_csv_and_completion_manifest() {
    let a = directory("a");
    let b = directory("b");
    for dir in [&a, &b] {
        let out = run(dir, &[]);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    assert_eq!(
        std::fs::read(a.join("opacity.png")).unwrap(),
        std::fs::read(b.join("opacity.png")).unwrap()
    );
    let image = std::fs::read(a.join("opacity.png")).unwrap();
    let decoder = png::Decoder::new(std::io::Cursor::new(image));
    let mut reader = decoder.read_info().unwrap();
    assert_eq!((reader.info().width, reader.info().height), (4, 4));
    let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
    reader.next_frame(&mut pixels).unwrap();
    assert!(pixels.iter().any(|&x| x > 0));
    let csv = std::fs::read_to_string(a.join("steps.csv")).unwrap();
    assert_eq!(csv.lines().count(), 4);
    let metadata = std::fs::read_to_string(a.join("run.json")).unwrap();
    assert!(metadata.contains("\"whole_process_memory_cap_claimed\": false"));
    std::fs::remove_dir_all(a).unwrap();
    std::fs::remove_dir_all(b).unwrap();
}
#[test]
fn existing_output_is_not_overwritten() {
    let dir = directory("existing");
    std::fs::create_dir(&dir).unwrap();
    std::fs::write(dir.join("keep"), b"original").unwrap();
    assert!(!run(&dir, &[]).status.success());
    assert_eq!(std::fs::read(dir.join("keep")).unwrap(), b"original");
    assert!(!dir.join("run.json").exists());
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
fn bad_cli_values_and_cap_fail_before_output_creation() {
    for extra in [
        ["--size", "0"],
        ["--size", "-1"],
        ["--size", "10001"],
        ["--size", "1001"],
        ["--cells-per-metre", "1001"],
        ["--cells-per-metre", "0"],
        ["--dt", "NaN"],
        ["--memory-mib", "0"],
        ["--steps", "-1"],
        ["--source-off-at", "4"],
        ["--threads", "0"],
        ["--threads", "-1"],
        ["--threads", "999999"],
    ] {
        let dir = directory("invalid");
        assert!(!run(&dir, &extra).status.success(), "{extra:?}");
        assert!(!dir.exists());
    }
}

#[test]
fn thread_selection_reports_the_count_or_rejects_an_unavailable_feature() {
    let dir = directory("threads");
    let threads = std::thread::available_parallelism()
        .map_or(1, usize::from)
        .min(2);
    let count = threads.to_string();
    let out = run(&dir, &["--threads", &count]);
    if cfg!(feature = "parallel") || threads == 1 {
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let metadata = std::fs::read_to_string(dir.join("run.json")).unwrap();
        assert!(metadata.contains(&format!("\"cpu_threads\": {threads}")));
        std::fs::remove_dir_all(dir).unwrap();
    } else {
        assert!(!out.status.success());
        assert!(String::from_utf8_lossy(&out.stderr).contains("--features parallel"));
        assert!(!dir.exists());
    }
}

#[test]
fn explicit_original_selection_replays_default_and_unknown_is_rejected() {
    let base = std::env::temp_dir().join(format!("rheon-choice-{}", std::process::id()));
    std::fs::create_dir(&base).unwrap();
    for (name, method) in [
        ("default", None),
        ("original", Some("jacobi-pcg-v1")),
        ("alternative", Some("sgs-pcg-v1")),
    ] {
        let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_rheon"));
        command
            .args([
                "--size",
                "8",
                "--steps",
                "4",
                "--source-off-at",
                "2",
                "--output",
            ])
            .arg(base.join(name));
        if let Some(method) = method {
            command.args(["--implementation", method]);
        }
        assert!(command.output().unwrap().status.success());
        let manifest = std::fs::read_to_string(base.join(name).join("run.json")).unwrap();
        assert!(manifest.contains(method.unwrap_or("jacobi-pcg-v1")));
    }
    for file in ["opacity.png", "steps.csv"] {
        assert_eq!(
            std::fs::read(base.join("default").join(file)).unwrap(),
            std::fs::read(base.join("original").join(file)).unwrap()
        );
    }
    let bad = std::process::Command::new(env!("CARGO_BIN_EXE_rheon"))
        .args(["--implementation", "unknown", "--output"])
        .arg(base.join("invalid"))
        .output()
        .unwrap();
    assert!(!bad.status.success());
    assert!(!base.join("invalid").exists());
    std::fs::remove_dir_all(base).unwrap();
}

#[test]
fn explicit_density_runs_on_a_small_domain_and_large_axis_preflight_never_allocates() {
    let dir = directory("density");
    let out = run(
        &dir,
        &[
            "--size",
            "16",
            "--cells-per-metre",
            "1000",
            "--dt",
            "0.000001",
        ],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let metadata = std::fs::read_to_string(dir.join("run.json")).unwrap();
    assert!(metadata.contains("\"cell_size_m\": 1.00000000000000002e-3"));
    assert!(metadata.contains("\"domain_side_m\": 1.60000000000000003e-2"));
    std::fs::remove_dir_all(dir).unwrap();
    let absent = directory("preflight");
    let small = run(
        &absent,
        &["--size", "16", "--cells-per-metre", "1000", "--preflight"],
    );
    assert!(
        small.status.success(),
        "{}",
        String::from_utf8_lossy(&small.stderr)
    );
    assert!(String::from_utf8_lossy(&small.stdout).contains("\"cells\":4096"));
    assert!(!absent.exists());
    let huge = run(
        &absent,
        &[
            "--size",
            "10000",
            "--cells-per-metre",
            "1000",
            "--preflight",
        ],
    );
    assert!(!huge.status.success());
    #[cfg(target_pointer_width = "64")]
    {
        let stdout = String::from_utf8_lossy(&huge.stdout);
        assert!(stdout.contains("\"cells\":1000000000000"));
        assert!(stdout.contains("\"fits_memory_limit\":false"));
        assert!(String::from_utf8_lossy(&huge.stderr).contains("buffer payload requires"));
    }
    assert!(!absent.exists());
}

#[cfg(feature = "desktop")]
#[test]
fn comparison_preflight_counts_native_solves_for_each_execution_policy() {
    for (mode, solves, budget) in [
        ("legacy", 64, 786_432_000_u64),
        ("teacher", 64, 786_432_000_u64),
        ("stop-on-reject", 113, 1_388_544_000_u64),
    ] {
        let absent = directory(mode);
        let out = Command::new(env!("CARGO_BIN_EXE_rheon-ai-compare"))
            .args([
                "--counts",
                "16,16,16",
                "--cells-per-metre",
                "1000",
                "--steps",
                "64",
                "--warmup-steps",
                "15",
                "--mode",
                mode,
                "--preflight",
                "--project-dir",
            ])
            .arg(&absent)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(report["native_pressure_iteration_limit"], 3000);
        assert_eq!(report["native_pressure_solve_budget"], solves);
        assert_eq!(
            report["native_pressure_cell_iteration_budget_total"],
            budget
        );
        assert_eq!(report["requested_steps"], 64);
        assert_eq!(report["warmup_steps"], 15);
        assert_eq!(report["model_executed"], false);
        assert_eq!(report["model_support_checked"], false);
        assert_eq!(report["whole_process_memory_cap_claimed"], false);
        assert_eq!(
            report["native_simulation_array_payload_reference_bytes"],
            333824
        );
        assert!(
            report
                .get("pressure_cell_iteration_budget_per_track")
                .is_none()
        );
        assert!(report.get("native_array_payload_bytes_per_track").is_none());
        assert!(!absent.exists());
    }
}
