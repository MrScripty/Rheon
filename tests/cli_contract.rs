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
        ["--dt", "NaN"],
        ["--memory-mib", "0"],
        ["--steps", "-1"],
        ["--source-off-at", "4"],
    ] {
        let dir = directory("invalid");
        assert!(!run(&dir, &extra).status.success(), "{extra:?}");
        assert!(!dir.exists());
    }
}
