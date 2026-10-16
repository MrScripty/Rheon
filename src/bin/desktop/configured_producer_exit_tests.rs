//! Actual Worker storage failure after numerical acceptance, followed by actual
//! producer-OS exit and model-absent cold inspection. This tests custody and UI
//! metadata, not trained-model accuracy, throughput or deadline controls.
use super::configured_worker_tests::{Output, app, finish, launch, member, package, set};
use super::{
    archive_view::ArchiveView,
    saved_comparison::{Availability, SavedComparisonPage},
};
use crate::run_sets::{
    self,
    configured::{ConfiguredRunSetRevision, FrozenConfiguredSpec, PressureIntent, archive},
};
use eframe::egui;
use rheon::recording::{
    RecordingError, RecordingReader,
    comparison::Selection,
    provider::{RecordedConfiguredSession, runtime::*},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    mem::ManuallyDrop,
    path::{Path, PathBuf},
    process::Command,
};

pub(super) const STORAGE_TAIL_PROCESS_TEST: &str = "configured_producer_exit_tests::worker_storage_tail_survives_producer_exit_then_model_absent_reader";
const PHASE: &str = "RHEON_CONFIGURED_STORAGE_PHASE";
const ROOT: &str = "RHEON_CONFIGURED_STORAGE_ROOT";
const RETAIN_ROOT: &str = "RHEON_CONFIGURED_STORAGE_TAIL_RETAIN";
const DT: f64 = 1.0 / 16.0;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct ArtifactPin {
    relative_path: PathBuf,
    sha256: [u8; 32],
}
#[derive(Serialize, Deserialize)]
struct CaseReceipt {
    correction: bool,
    manifest_relative: PathBuf,
    package_path: PathBuf,
    run_id: u64,
    source_revision: String,
    archive_sha256: [u8; 32],
    selected_sha256: [[u8; 32]; 2],
    manifest_sha256: [u8; 32],
    rhs_sha256: [u8; 32],
    original_artifacts: Vec<ArtifactPin>,
}
fn digest(path: &Path) -> [u8; 32] {
    Sha256::digest(std::fs::read(path).unwrap()).into()
}
fn artifact_pins(root: &Path) -> Vec<ArtifactPin> {
    // Known fixture directories contain intent, started/finished revisions and
    // one SQLite archive. Bound the enumeration before collecting any payload.
    let entries: Vec<_> = std::fs::read_dir(root).unwrap().take(9).collect();
    assert!(entries.len() <= 8, "unexpected case-directory inventory");
    let mut pins: Vec<_> = entries
        .into_iter()
        .map(|entry| {
            let entry = entry.unwrap();
            assert!(
                entry.file_type().unwrap().is_file(),
                "case artifact must be a regular file"
            );
            ArtifactPin {
                relative_path: PathBuf::from(entry.file_name()),
                sha256: digest(&entry.path()),
            }
        })
        .collect();
    pins.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    pins
}
fn selections(reader: &RecordingReader) -> [[u8; 32]; 2] {
    std::array::from_fn(|ordinal| Selection::read(reader, ordinal).unwrap().digest())
}
fn check_tail(
    reader: &RecordingReader,
    revision: &ConfiguredRunSetRevision,
) -> RuntimeTerminationAudit {
    assert_eq!(revision.members.len(), 1);
    assert_eq!(revision.baseline_member, "configured");
    let saved = &revision.members[0];
    assert_eq!(saved.member_id, "configured");
    assert!(saved.differences_from_baseline.is_empty());
    assert_eq!(saved.spec.future.recording.max_records, 2);
    assert_eq!(reader.count().unwrap(), 2);
    assert_eq!(reader.select(0).unwrap().generation, 0);
    let seed = reader.select(1).unwrap();
    assert_eq!(seed.generation, 1);
    assert_eq!(seed.time.to_bits(), DT.to_bits());
    assert!(
        RecordedConfiguredSession::decode(reader.run(), &seed)
            .unwrap()
            .runtime
            .is_none()
    );
    assert!(matches!(reader.select(2), Err(RecordingError::Missing(2))));
    let status = reader.status().unwrap();
    assert_eq!(status.outcome, "failed");
    assert_eq!(status.accepted_generation, Some(2));
    let audit = audit_runtime_termination(reader).unwrap();
    assert_eq!(
        audit.termination.cause,
        RuntimeTerminationCause::StorageFailure
    );
    assert_eq!(audit.termination.accepted_generation, 2);
    assert_eq!(
        audit.termination.accepted_time.to_bits(),
        (2.0 * DT).to_bits()
    );
    assert!(audit.termination.detail.contains("record count"));
    assert_eq!(
        audit.coverage,
        RuntimeTerminalCoverage::UnrecordedAcceptedTail {
            last_recorded_generation: Some(1),
        }
    );
    let observation = audit.termination.observation.as_ref().unwrap();
    assert_eq!(observation.outcome, RuntimeOutcome::CandidateReturned);
    assert_eq!(observation.request.run_id, reader.run().identity.run_id);
    assert_eq!(observation.request.grid.counts(), [2; 3]);
    assert_eq!(observation.request.density.to_bits(), 2.0f64.to_bits());
    assert_eq!(
        observation.request.dynamic_viscosity.to_bits(),
        0.0f64.to_bits()
    );
    let context = &observation.request.context;
    assert_eq!(context.generation, 2);
    assert_eq!(context.interval_start.to_bits(), DT.to_bits());
    assert_eq!(context.interval_end.to_bits(), (2.0 * DT).to_bits());
    assert_eq!(context.dt.to_bits(), DT.to_bits());
    assert_eq!(context.native_prefix_steps, 1);
    assert_eq!(context.prior_generation, Some(1));
    assert_eq!(
        context.prior_interval_end.map(f64::to_bits),
        Some(DT.to_bits())
    );
    let pressure_sha: [u8; 32] =
        Sha256::digest(&seed.field("interval_pressure").unwrap().data).into();
    assert_eq!(context.prior_pressure_sha256, Some(pressure_sha));
    let PressureIntent::NativePrefixThenConfigured { provider, .. } = &saved.spec.future.pressure
    else {
        panic!("storage fixture lost its configured provider declaration");
    };
    provider
        .validate_resolved_identity(&observation.request.binding)
        .unwrap();
    let archive::ConfiguredRuntimeCustody::Typed(frozen) =
        archive::runtime_custody(reader, &saved.spec).unwrap()
    else {
        panic!("real typed terminal became unavailable");
    };
    assert_eq!(frozen.coverage, audit.coverage);
    assert_eq!(frozen.termination, audit.termination);
    let pin = saved.attempt.as_ref().unwrap().finalized.as_ref().unwrap();
    assert_eq!(pin.recorded_count, 2);
    assert_eq!(pin.accepted_generation, Some(2));
    assert_eq!(pin.outcome, run_sets::TerminalOutcome::Failed);
    audit
}
fn check_cold_ui(manifest: &Path, revision: &ConfiguredRunSetRevision, reader: &RecordingReader) {
    let root = manifest.parent().unwrap();
    let mut page = SavedComparisonPage::default();
    page.set_location(
        root.to_string_lossy(),
        manifest.file_name().unwrap().to_string_lossy(),
    );
    page.load_manifest();
    let summaries = page.member_summaries();
    assert_eq!(summaries.len(), 1, "{}", page.message());
    let summary = &summaries[0];
    assert_eq!(summary.member_id, "configured");
    assert_eq!(summary.availability, Availability::Ready);
    assert_eq!(summary.recorded_count, Some(2));
    assert_eq!(summary.recorded_generation, Some(1));
    assert_eq!(summary.accepted_generation, Some(2));
    assert_eq!(summary.unrecorded_tail, Some(true));
    assert!(page.selected_pair().is_none());
    let saved = &revision.members[0];
    let mut view = ArchiveView::default();
    view.path = root
        .join(&saved.attempt.as_ref().unwrap().archive_relative_path)
        .to_string_lossy()
        .into();
    view.open(saved.spec.future.recording.limits()).unwrap();
    view.inspector.verify().unwrap();
    view.inspector.select_timestep(1).unwrap();
    assert_eq!(
        view.inspector.selected_identity(),
        Some((reader.run().identity.run_id, 1, DT))
    );
    assert!(view.inspector.select_timestep(2).is_err());
    assert!(view.inspector.selected_identity().is_none());
    view.inspector.select_timestep(1).unwrap();
    let ctx = egui::Context::default();
    let mut render = || {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1800.0, 2200.0),
                )),
                ..Default::default()
            },
            |ctx| view.show(ctx),
        )
    };
    let _ = render();
    let shown = render();
    let text = shown
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Text(text) => Some(text.galley.job.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(text.contains("StorageFailure"), "{text}");
    assert!(text.contains("accepted generation 2"), "{text}");
    assert!(
        text.contains("unrecorded accepted tail remains unavailable"),
        "{text}"
    );
    assert_eq!(
        view.inspector.selected_identity(),
        Some((reader.run().identity.run_id, 1, DT))
    );
}
fn produce(root: &Path, correction: bool) -> CaseReceipt {
    assert_ne!(std::env::var(PHASE).ok().as_deref(), Some("inspect"));
    let output = ManuallyDrop::new(Output(root.to_path_buf())); // Parent owns cleanup after both OS children.
    let fixture = package(correction, "solve");
    let mut request = set(&fixture);
    let mut configured = request.members.remove(1);
    let mut future = configured.spec.future.clone();
    future.recording.max_records = 2;
    configured.spec =
        FrozenConfiguredSpec::new(configured.spec.constructor.clone(), future).unwrap();
    configured.differences_from_baseline.clear();
    request.baseline_member = configured.member_id.clone();
    request.members = vec![configured];
    request.set_id = if correction {
        "correction-storage-tail"
    } else {
        "full-storage-tail"
    }
    .into();
    request.validate().unwrap();
    let mut application = app(&output);
    let run_root = root.join(if correction {
        "correction.runset"
    } else {
        "full.runset"
    });
    launch(&mut application, request, &run_root);
    let revision = finish(&mut application);
    assert!(application.rows.is_empty(), "{}", application.status);
    assert!(
        application.status.contains("record count"),
        "{}",
        application.status
    );
    let (count, last) = application
        .history
        .as_ref()
        .unwrap()
        .view(0, usize::MAX, true)
        .unwrap();
    assert_eq!(
        count, 2,
        "constructor plus native seed only; accepted g2 append failed before preview publication"
    );
    let last = last.unwrap();
    assert_eq!(last.step, 1);
    assert_eq!(last.time.to_bits(), DT.to_bits());
    let manifest = PathBuf::from(&application.run_sets.path);
    let reader = member(&run_root, &revision, 0);
    let audit = check_tail(&reader, &revision);
    let observation = audit.termination.observation.as_ref().unwrap();
    let attempt = revision.members[0].attempt.as_ref().unwrap();
    let archive_path = run_root.join(&attempt.archive_relative_path);
    let manifest_sha256 = digest(&manifest);
    let archive_sha256 = digest(&archive_path);
    assert_eq!(archive_sha256, attempt.finalized.as_ref().unwrap().sha256);
    let receipt = CaseReceipt {
        correction,
        manifest_relative: manifest.strip_prefix(root).unwrap().to_path_buf(),
        package_path: fixture.manifest.clone(),
        run_id: reader.run().identity.run_id,
        source_revision: reader.run().identity.source_revision.clone(),
        archive_sha256,
        selected_sha256: selections(&reader),
        manifest_sha256,
        rhs_sha256: observation.request.context.rhs_sha256,
        original_artifacts: artifact_pins(&run_root),
    };
    let names: Vec<_> = receipt
        .original_artifacts
        .iter()
        .map(|pin| pin.relative_path.as_path())
        .collect();
    assert_eq!(names.len(), 4);
    for required in [
        Path::new("intent.json"),
        Path::new("member-0-started.json"),
        Path::new("member-0-finished.json"),
        Path::new(&attempt.archive_relative_path),
    ] {
        assert!(
            names.contains(&required),
            "missing original artifact {required:?}"
        );
    }
    drop(reader);
    drop(application);
    drop(fixture); // The actual Python manifest/code root is removed before producer exit.
    assert!(!receipt.package_path.exists());
    receipt
}
fn inspect(root: &Path) {
    assert_eq!(std::env::var_os("PATH").unwrap(), "");
    let receipts: Vec<CaseReceipt> =
        serde_json::from_slice(&std::fs::read(root.join("producer-receipts.json")).unwrap())
            .unwrap();
    assert_eq!(receipts.len(), 2);
    for (receipt, correction) in receipts.iter().zip([false, true]) {
        assert_eq!(receipt.correction, correction);
        assert!(!receipt.package_path.exists());
        assert!(!receipt.package_path.parent().unwrap().exists());
        let manifest = root.join(&receipt.manifest_relative);
        assert_eq!(digest(&manifest), receipt.manifest_sha256);
        let run_root = manifest.parent().unwrap();
        assert_eq!(artifact_pins(run_root), receipt.original_artifacts);
        let revision = ConfiguredRunSetRevision::open(&manifest).unwrap();
        let reader = member(run_root, &revision, 0); // Exact pinned cold Ready.
        assert_eq!(reader.run().identity.run_id, receipt.run_id);
        assert_eq!(
            reader.run().identity.source_revision,
            receipt.source_revision
        );
        assert_eq!(selections(&reader), receipt.selected_sha256);
        let audit = check_tail(&reader, &revision);
        assert_eq!(
            audit
                .termination
                .observation
                .as_ref()
                .unwrap()
                .request
                .context
                .rhs_sha256,
            receipt.rhs_sha256
        );
        let archive_path = run_root.join(
            &revision.members[0]
                .attempt
                .as_ref()
                .unwrap()
                .archive_relative_path,
        );
        assert_eq!(digest(&archive_path), receipt.archive_sha256);
        check_cold_ui(&manifest, &revision, &reader);
        assert_eq!(digest(&archive_path), receipt.archive_sha256);
        assert_eq!(digest(&manifest), receipt.manifest_sha256);
        assert_eq!(selections(&reader), receipt.selected_sha256);
        assert_eq!(artifact_pins(run_root), receipt.original_artifacts);
    }
}

#[test]
fn worker_storage_tail_survives_producer_exit_then_model_absent_reader() {
    match std::env::var(PHASE).ok().as_deref() {
        Some("produce") => {
            let root = PathBuf::from(std::env::var_os(ROOT).unwrap());
            let receipts = [produce(&root, false), produce(&root, true)];
            std::fs::write(
                root.join("producer-receipts.json"),
                serde_json::to_vec(&receipts).unwrap(),
            )
            .unwrap();
            println!("ACTUAL_CONFIGURED_STORAGE_PRODUCER_CLOSED");
        }
        Some("inspect") => {
            inspect(Path::new(&std::env::var_os(ROOT).unwrap()));
            println!("ACTUAL_CONFIGURED_STORAGE_COLD_VERIFIED");
        }
        None => {
            let retained = std::env::var_os(RETAIN_ROOT).map(PathBuf::from);
            let output = if let Some(path) = &retained {
                // Only a new, explicitly opted-in task-owned /tmp child can be
                // retained. Existing trees, symlink destinations and parent paths
                // are never adopted or deleted by this fixture.
                assert_eq!(path.parent(), Some(Path::new("/tmp")));
                assert!(
                    path.file_name()
                        .unwrap()
                        .to_string_lossy()
                        .starts_with("rheon-configured-storage-")
                );
                std::fs::create_dir(path).unwrap();
                Output(path.clone())
            } else {
                Output::new()
            };
            for (phase, marker) in [
                ("produce", "ACTUAL_CONFIGURED_STORAGE_PRODUCER_CLOSED"),
                ("inspect", "ACTUAL_CONFIGURED_STORAGE_COLD_VERIFIED"),
            ] {
                let mut command = Command::new(std::env::current_exe().unwrap());
                command.args(["--exact", STORAGE_TAIL_PROCESS_TEST, "--nocapture"]);
                if phase == "inspect" {
                    command.env_clear().env("PATH", "");
                }
                let child = command
                    .env(PHASE, phase)
                    .env(ROOT, &output.0)
                    .output()
                    .unwrap();
                std::fs::write(output.0.join(format!("{phase}.stdout.log")), &child.stdout)
                    .unwrap();
                std::fs::write(output.0.join(format!("{phase}.stderr.log")), &child.stderr)
                    .unwrap();
                let stdout = String::from_utf8_lossy(&child.stdout);
                assert!(
                    child.status.success(),
                    "{phase}: {stdout}\n{}",
                    String::from_utf8_lossy(&child.stderr)
                );
                assert!(stdout.contains(marker), "{stdout}");
                assert!(
                    stdout.contains("1 passed; 0 failed;"),
                    "exact child test did not execute: {stdout}"
                );
                // output() reaps the actual producer OS before fresh cold reader creation.
            }
            if retained.is_some() {
                std::mem::forget(output);
            }
        }
        Some(other) => panic!("unknown storage fixture phase {other}"),
    }
}
