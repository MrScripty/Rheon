//! App source selection over real native/configured manifest and SQLite owners.
//! Configured fixtures stop honestly at constructor failure, before any runtime.
#![cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
#[path = "../src/bin/desktop/run_sets.rs"]
#[allow(dead_code)] // Other public owner APIs have dedicated contract suites.
mod run_sets;

use rheon::{
    recording::{provider::*, *},
    solver_provider::configured::ContextProviderIdentity,
    *,
};
use run_sets::{configured::*, source::*, *};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

const CAP: usize = 1024 * 1024;
const PHASE: &str = "RHEON_SAVED_SOURCE_PHASE";
const ROOT: &str = "RHEON_SAVED_SOURCE_ROOT";
const COLD_TEST: &str = "producer_exits_before_fresh_process_opens_both_sources_without_runtime";
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "rheon-saved-source-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn budget() -> ArchiveBudget {
    ArchiveBudget {
        max_database_bytes: 8 * 1024 * 1024,
        max_record_bytes: CAP,
        max_records: 3,
    }
}
fn native() -> RunSetRevision {
    RunSetRevision {
        schema_version: 1,
        set_id: "native-source-fixture".into(),
        parent_sha256: None,
        baseline_member: "base".into(),
        members: vec![RunSetMember {
            member_id: "base".into(),
            spec: FrozenSpec::new(NativeCarrierSpec {
                counts_xyz: [2, 1, 1],
                spacing_m: [0.25; 3],
                origin_m: [0.0; 3],
                density_kg_m3: 2.0,
                pressure_method: NativePressureMethod::JacobiPcgV1,
                tolerances: NativeTolerances {
                    relative_residual: 1e-12,
                    absolute_residual_m3_s2: 1e-12,
                    predicted_divergence_per_s: 1e-9,
                    actual_divergence_per_s: 1e-5,
                    max_courant: 0.5,
                    max_pressure_iterations: 100,
                },
                requested_dt_s: 1.0 / 16.0,
                requested_steps: 2,
                source: None,
                simulation_memory_limit_bytes: CAP,
                recording: budget(),
            })
            .unwrap(),
            differences_from_baseline: vec![],
            attempt: None,
        }],
    }
}
fn configured() -> ConfiguredRunSetRevision {
    assert_ne!(
        std::env::var(PHASE).ok().as_deref(),
        Some("inspect"),
        "cold fixture must not construct Session"
    );
    let session = SimulationSession::new(
        SessionIdentity {
            run_id: 77,
            source_revision: "source-host-fixture".into(),
        },
        GridGeometry::new([2, 1, 1], [0.25; 3], [0.0; 3]).unwrap(),
        LiquidTransportConfig {
            carrier: SimulationConfig {
                density: 2.0,
                memory_limit: CAP,
                pressure: PressureSettings {
                    relative_residual: 1e-12,
                    absolute_residual: 1e-12,
                    divergence_limit: 1e-9,
                    max_iterations: 100,
                },
                actual_divergence_limit: 1e-5,
                max_courant: 0.5,
            },
            represented_density: 2.0,
            volume_stamp: VolumeStamp { id: 17, version: 0 },
            carrier_id: 18,
        },
        PressureImplementation::JacobiPcgV1,
        vec![1.0; 2],
        CAP,
        2,
    )
    .unwrap();
    let pin = ConfiguredProviderPin::capture(&ContextProviderIdentity {
        id: "declared-never-launched-package".into(),
        version: "1".into(),
        manifest_sha256: [7; 32],
        checkpoint_sha256: None,
        training_run_id: None,
        runtime: "declared fixture runtime, never launched".into(),
        source_revision: "model-source-separate-from-host".into(),
    })
    .unwrap();
    let future = ConfiguredFutureIntent {
        requested_dt_s: 1.0 / 16.0,
        requested_steps: 2,
        forces: vec![],
        volume: VolumeControls {
            max_outward_courant: 0.5,
            actual_divergence_per_s: 1e-5,
        },
        viscosity: ViscosityIntent::Disabled,
        pressure: PressureIntent::NativePrefixThenConfigured {
            provider: Box::new(pin),
            deployment: DeploymentLocator::Absolute(
                "/nonexistent/rheon-saved-source/deployment.json".into(),
            ),
            requested_native_prefix_steps: 1,
            minimum_native_prefix_steps: 1,
        },
        provider_workspace_limit_bytes: CAP,
        viscosity_workspace_limit_bytes: None,
        recording: budget(),
        capture_constructor: true,
        executor: ExecutorDeclaration {
            id: "source-test-host".into(),
            source_revision: "source-host-fixture".into(),
            build_provenance: None,
        },
    };
    ConfiguredRunSetRevision {
        schema_version: CONFIGURED_VERSION,
        kind: ConfiguredSetKind::ConfiguredAcceptedSessionSet,
        set_id: "configured-source-fixture".into(),
        parent_sha256: None,
        baseline_member: "base".into(),
        members: vec![ConfiguredRunSetMember {
            member_id: "base".into(),
            spec: FrozenConfiguredSpec::capture_constructor(&session.snapshot(), future).unwrap(),
            differences_from_baseline: vec![],
            attempt: None,
        }],
    }
}
fn with_attempt(set: &mut SavedRunSet) {
    match set {
        SavedRunSet::Native(set) => {
            set.members[0].attempt = Some(RunAttempt {
                attempt_id: "native-attempt".into(),
                archive_relative_path: "native.sqlite".into(),
                expected_run_id: 77,
                expected_source_revision: "source-host-fixture".into(),
                finalized: None,
            })
        }
        SavedRunSet::Configured(set) => {
            set.members[0].attempt = Some(ConfiguredRunAttempt {
                attempt_id: "configured-attempt".into(),
                archive_relative_path: "configured.sqlite".into(),
                expected_run_id: 77,
                expected_source_revision: "source-host-fixture".into(),
                finalized: None,
            })
        }
    }
}
fn write_constructor(root: &Path, member: SavedMember<'_>) -> StepRecorder {
    assert_ne!(
        std::env::var(PHASE).ok().as_deref(),
        Some("inspect"),
        "cold phase must not construct numerical owners"
    );
    let attempt = member.attempt().unwrap();
    let identity = SessionIdentity {
        run_id: attempt.expected_run_id,
        source_revision: attempt.expected_source_revision.into(),
    };
    let limits = member.limits();
    let (run, publication) = match member {
        SavedMember::Native(member) => {
            let simulation = member.spec.spec.create_simulation().unwrap();
            (
                bound_native_run(
                    &member.spec,
                    &simulation,
                    identity,
                    "actual source-contract producer",
                )
                .unwrap(),
                carrier_publication(&simulation, 0, limits.max_record_bytes).unwrap(),
            )
        }
        SavedMember::Configured(member) => {
            let session = member.spec.create_session(identity).unwrap();
            (
                archive::bound_configured_run(&member.spec, &session.snapshot()).unwrap(),
                configured_publication(&session.snapshot(), 0, limits.max_record_bytes).unwrap(),
            )
        }
    };
    let mut writer =
        StepRecorder::create(&root.join(attempt.archive_relative_path), run, limits).unwrap();
    writer.append(&publication).unwrap();
    writer
}
fn pin(root: &Path, set: &mut SavedRunSet) {
    match set {
        SavedRunSet::Native(set) => {
            let member = &mut set.members[0];
            member
                .attempt
                .as_mut()
                .unwrap()
                .pin_closed(root, &member.spec)
                .unwrap();
        }
        SavedRunSet::Configured(set) => {
            let member = &mut set.members[0];
            member
                .attempt
                .as_mut()
                .unwrap()
                .pin_closed(root, &member.spec)
                .unwrap();
        }
    }
}
fn save(root: &Path, set: &SavedRunSet) -> PathBuf {
    let path = root.join(format!("{}.json", set.set_id()));
    match set {
        SavedRunSet::Native(set) => set.save_new(&path).unwrap(),
        SavedRunSet::Configured(set) => set.save_new(&path).unwrap(),
    }
    path
}

#[test]
fn native_inner_json_and_digest_remain_exact_existing_owner_bytes() {
    let native = native();
    let bytes = serde_json::to_vec(&native).unwrap();
    let wrapper = SavedRunSet::Native(native.clone());
    assert_eq!(serde_json::to_vec(&wrapper).unwrap(), bytes);
    assert_eq!(wrapper.sha256().unwrap(), native.sha256().unwrap());
    assert_eq!(
        wrapper.sha256().unwrap(),
        <[u8; 32]>::from(Sha256::digest(&bytes))
    );
    assert_eq!(
        serde_json::from_slice::<SavedRunSet>(&bytes).unwrap(),
        wrapper
    );
    let object: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        object
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>(),
        [
            "baseline_member",
            "members",
            "parent_sha256",
            "schema_version",
            "set_id"
        ]
        .into_iter()
        .collect()
    );
    assert_eq!(wrapper.schema_version(), 1);
    assert!(!wrapper.is_configured());
    assert!(wrapper.native().is_some());
    assert!(wrapper.configured().is_none());
}

#[test]
fn configured_inner_decode_is_closed_and_validates_owner_version_kind_and_hashes() {
    let configured = configured();
    let wrapper = SavedRunSet::Configured(configured.clone());
    let inner = serde_json::to_value(&configured).unwrap();
    assert_eq!(serde_json::to_value(&wrapper).unwrap(), inner);
    assert_eq!(wrapper.sha256().unwrap(), configured.digest().unwrap());
    assert_eq!(
        serde_json::from_value::<SavedRunSet>(inner.clone()).unwrap(),
        wrapper
    );
    for bad in [
        {
            let mut v = inner.clone();
            v["schema_version"] = json!(3);
            v
        },
        {
            let mut v = inner.clone();
            v["kind"] = json!("unknown-kind");
            v
        },
        {
            let mut v = inner.clone();
            v["extra"] = json!(true);
            v
        },
        {
            let mut v = inner.clone();
            v["members"][0]["spec"]["future"]["extra"] = json!(true);
            v
        },
        {
            let mut v = inner.clone();
            v["members"][0]["spec"]["sha256"][0] =
                json!(v["members"][0]["spec"]["sha256"][0].as_u64().unwrap() ^ 1);
            v
        },
        {
            let mut v = inner.clone();
            v["members"][0]["spec"]["future"]["pressure"]["provider"]["runtime"] =
                json!("changed without resealing");
            v
        },
        json!({"Configured": inner}),
    ] {
        assert!(serde_json::from_value::<SavedRunSet>(bad).is_err());
    }
    let mut native_version = serde_json::to_value(native()).unwrap();
    native_version["schema_version"] = json!(2);
    assert!(serde_json::from_value::<SavedRunSet>(native_version).is_err());
    assert_eq!(wrapper.schema_version(), 2);
    assert!(wrapper.is_configured());
    assert!(wrapper.native().is_none());
}

#[test]
fn manifest_open_preserves_owner_integrity_errors_and_bounded_version_selection() {
    let root = Directory::new();
    for set in [
        SavedRunSet::Native(native()),
        SavedRunSet::Configured(configured()),
    ] {
        let path = save(&root.0, &set);
        assert_eq!(SavedRunSet::open(&path).unwrap(), set);
        let mut packet: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        packet["sha256"][0] = json!(packet["sha256"][0].as_u64().unwrap() ^ 1);
        std::fs::write(&path, serde_json::to_vec(&packet).unwrap()).unwrap();
        assert!(matches!(
            SavedRunSet::open(&path),
            Err(RunSetError::Integrity(_))
        ));
        packet["schema_version"] = json!(99);
        std::fs::write(&path, serde_json::to_vec(&packet).unwrap()).unwrap();
        assert!(matches!(
            SavedRunSet::open(&path),
            Err(RunSetError::Unsupported("manifest version"))
        ));
    }
    let path = root.0.join("configured-source-fixture.json");
    let mut packet: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    packet["schema_version"] = json!(1);
    std::fs::write(&path, serde_json::to_vec(&packet).unwrap()).unwrap();
    assert!(
        matches!(SavedRunSet::open(&path), Err(RunSetError::Codec(_))),
        "schema-one malformed payload must not select configured fallback"
    );
    std::fs::write(&path, vec![b' '; 1024 * 1024 + 1]).unwrap();
    assert!(matches!(
        SavedRunSet::open(&path),
        Err(RunSetError::Invalid("manifest byte limit"))
    ));
    assert!(matches!(
        SavedRunSet::open(&root.0.join("absent.json")),
        Err(RunSetError::Io(_))
    ));
}

#[test]
fn member_lookup_and_borrowed_metadata_preserve_limits_attempts_and_typed_difference_labels() {
    let root = Directory::new();
    let mut native = native();
    let mut alternate = native.members[0].clone();
    alternate.member_id = "alternate".into();
    alternate.spec.spec.pressure_method = NativePressureMethod::SymmetricGaussSeidelPcgV1;
    alternate.spec = FrozenSpec::new(alternate.spec.spec).unwrap();
    alternate.differences_from_baseline = vec![SpecDifference::PressureMethod];
    native.members.push(alternate);
    let mut configured = configured();
    let mut alternate = configured.members[0].clone();
    alternate.member_id = "alternate".into();
    alternate.spec.future.pressure = PressureIntent::NativeOnly;
    alternate.spec =
        FrozenConfiguredSpec::new(alternate.spec.constructor, alternate.spec.future).unwrap();
    alternate.differences_from_baseline =
        ConfiguredSpecDifference::between(&configured.members[0].spec, &alternate.spec).unwrap();
    configured.members.push(alternate);
    for mut set in [
        SavedRunSet::Native(native),
        SavedRunSet::Configured(configured),
    ] {
        set.validate().unwrap();
        assert_eq!(set.baseline_member_id(), "base");
        assert_eq!(set.member_count(), 2);
        assert_eq!(set.member_ids().collect::<Vec<_>>(), ["base", "alternate"]);
        assert_eq!(set.members().count(), 2);
        assert!(set.member("missing").is_none());
        let base = set.member("base").unwrap();
        assert_eq!(base.id(), "base");
        assert_eq!(base.limits(), budget().limits());
        assert!(base.attempt().is_none());
        assert!(base.difference_labels().is_empty());
        assert!(matches!(base.open(&root.0), MemberState::NotRun));
        let alternate = set.member("alternate").unwrap();
        assert_eq!(
            alternate.difference_labels(),
            if set.is_configured() {
                vec!["Pressure selection"]
            } else {
                vec!["Native pressure method"]
            }
        );
        assert!(base.description().contains(if set.is_configured() {
            "declared-never-launched-package"
        } else {
            "jacobi-pcg-v1"
        }));
        with_attempt(&mut set);
        let attempt = set.member("base").unwrap().attempt().unwrap();
        assert_eq!(attempt.expected_run_id, 77);
        assert_eq!(attempt.expected_source_revision, "source-host-fixture");
        assert!(attempt.finalized.is_none());
        assert!(matches!(
            set.member("base").unwrap().open(&root.0),
            MemberState::Missing
        ));
    }
}

#[test]
fn real_constructor_archives_delegate_incomplete_unpinned_failed_ready_and_changed_artifact_states()
{
    let root = Directory::new();
    for mut set in [
        SavedRunSet::Native(native()),
        SavedRunSet::Configured(configured()),
    ] {
        with_attempt(&mut set);
        let mut writer = write_constructor(&root.0, set.member("base").unwrap());
        assert!(matches!(
            set.member("base").unwrap().open(&root.0),
            MemberState::Incomplete {
                recorded_count: 1,
                ..
            }
        ));
        writer
            .finish(
                "failed",
                "explicit fixture stop before interval/runtime",
                Some(0),
            )
            .unwrap();
        drop(writer);
        assert!(matches!(
            set.member("base").unwrap().open(&root.0),
            MemberState::Unpinned {
                recorded_count: 1,
                ..
            }
        ));
        pin(&root.0, &mut set);
        let member = set.member("base").unwrap();
        let pin = member.attempt().unwrap().finalized.unwrap();
        assert_eq!(pin.recorded_count, 1);
        assert_eq!(pin.accepted_generation, Some(0));
        assert_eq!(pin.outcome, TerminalOutcome::Failed);
        let MemberState::Ready(reader) = member.open(&root.0) else {
            panic!("closed pinned failed archive must be Ready for inspection")
        };
        assert_eq!(reader.status().unwrap().outcome, "failed");
        assert_eq!(reader.count().unwrap(), 1);
        assert_eq!(reader.select(0).unwrap().stage, SnapshotStage::Constructor);
        drop(reader);
        let path = root.0.join(member.attempt().unwrap().archive_relative_path);
        let mut bytes = std::fs::read(&path).unwrap();
        bytes.push(0);
        std::fs::write(path, bytes).unwrap();
        assert!(matches!(member.open(&root.0), MemberState::Invalid(_)));
    }
}

#[test]
fn producer_exits_before_fresh_process_opens_both_sources_without_runtime() {
    match std::env::var(PHASE).ok().as_deref() {
        Some("produce") => {
            let root = PathBuf::from(std::env::var(ROOT).unwrap());
            for mut set in [
                SavedRunSet::Native(native()),
                SavedRunSet::Configured(configured()),
            ] {
                with_attempt(&mut set);
                let mut writer = write_constructor(&root, set.member("base").unwrap());
                writer
                    .finish("failed", "producer exited before execution", Some(0))
                    .unwrap();
                drop(writer);
                pin(&root, &mut set);
                save(&root, &set);
            }
        }
        Some("inspect") => {
            let root = PathBuf::from(std::env::var(ROOT).unwrap());
            for (name, configured) in [
                ("native-source-fixture", false),
                ("configured-source-fixture", true),
            ] {
                let set = SavedRunSet::open(&root.join(format!("{name}.json"))).unwrap();
                assert_eq!(set.is_configured(), configured);
                let member = set.member("base").unwrap();
                let MemberState::Ready(reader) = member.open(&root) else {
                    panic!("cold pinned member must remain inspectable")
                };
                assert_eq!(reader.status().unwrap().outcome, "failed");
                assert_eq!(reader.select(0).unwrap().generation, 0);
                if let Some(set) = set.configured() {
                    let PressureIntent::NativePrefixThenConfigured {
                        deployment: DeploymentLocator::Absolute(path),
                        ..
                    } = &set.members[0].spec.future.pressure
                    else {
                        panic!("missing declared provider intent")
                    };
                    assert!(!Path::new(path).exists());
                }
            }
        }
        None => {
            let root = Directory::new();
            for phase in ["produce", "inspect"] {
                let output = Command::new(std::env::current_exe().unwrap())
                    .args(["--exact", COLD_TEST, "--nocapture"])
                    .env(PHASE, phase)
                    .env(ROOT, &root.0)
                    .output()
                    .unwrap();
                assert!(
                    output.status.success(),
                    "{phase}: {}\n{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
            }
        }
        Some(_) => panic!("unknown source-contract phase"),
    }
}
