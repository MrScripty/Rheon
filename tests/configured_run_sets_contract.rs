#![cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
#[path = "../src/bin/desktop/run_sets.rs"]
#[allow(dead_code)] // Native-owner contracts are exercised by saved_run_set_contract.
mod run_sets;

use rheon::{solver_provider::configured::ContextProviderIdentity, *};
use run_sets::{
    ArchiveBudget, FinalizedArchive, FrozenSpec, NativePressureMethod, RunSetError, RunSetRevision,
    TerminalOutcome, configured::*,
};
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

const CAP: usize = 1024 * 1024;
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "rheon-configured-intent-{}-{}",
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
fn executor() -> ExecutorDeclaration {
    ExecutorDeclaration {
        id: "accepted-session-contract-host".into(),
        source_revision: "host-source-A".into(),
        build_provenance: None,
    }
}
fn future() -> ConfiguredFutureIntent {
    ConfiguredFutureIntent {
        requested_dt_s: 1.0 / 60.0,
        requested_steps: 3,
        forces: vec![ForceSpec {
            value: [0.125, 0.0, 0.0],
            units: ForceUnitsSpec::Acceleration,
            region: Some(ForceRegionSpec {
                lower_m: [0.0; 3],
                upper_m: [0.5, 0.25, 0.25],
            }),
        }],
        volume: VolumeControls {
            max_outward_courant: 0.5,
            actual_divergence_per_s: 5e-7,
        },
        viscosity: ViscosityIntent::NewtonianConstant {
            dynamic_viscosity_pa_s: 1.0 / 128.0,
        },
        pressure: PressureIntent::NativeOnly,
        provider_workspace_limit_bytes: CAP,
        viscosity_workspace_limit_bytes: Some(CAP),
        recording: ArchiveBudget {
            max_database_bytes: 8 * 1024 * 1024,
            max_record_bytes: CAP,
            max_records: 4,
        },
        capture_constructor: true,
        executor: executor(),
    }
}
fn constructor(method: PressureImplementation, represented_density: f64) -> SimulationSession {
    SimulationSession::new(
        SessionIdentity {
            run_id: 1,
            source_revision: executor().source_revision,
        },
        GridGeometry::new([2, 1, 1], [0.25; 3], [0.0; 3]).unwrap(),
        LiquidTransportConfig {
            carrier: SimulationConfig {
                density: 2.0,
                memory_limit: CAP,
                pressure: PressureSettings {
                    relative_residual: 1e-12,
                    absolute_residual: 1e-14,
                    divergence_limit: 1e-9,
                    max_iterations: 256,
                },
                actual_divergence_limit: 1e-6,
                max_courant: 0.75,
            },
            represented_density,
            volume_stamp: VolumeStamp { id: 17, version: 3 },
            carrier_id: 18,
        },
        method,
        vec![1.0; 2],
        2 * CAP,
        7,
    )
    .unwrap()
}
fn frozen() -> FrozenConfiguredSpec {
    FrozenConfiguredSpec::capture_constructor(
        &constructor(PressureImplementation::JacobiPcgV1, 2.0).snapshot(),
        future(),
    )
    .unwrap()
}
fn provider() -> ContextProviderIdentity {
    ContextProviderIdentity {
        id: "declared-package-fixture".into(),
        version: "4".into(),
        manifest_sha256: [1; 32],
        checkpoint_sha256: Some([2; 32]),
        training_run_id: None,
        runtime: "trusted-local-fixture-runtime".into(),
        source_revision: "different-model-source".into(),
    }
}
fn configured_future() -> ConfiguredFutureIntent {
    let mut f = future();
    f.pressure = PressureIntent::NativePrefixThenConfigured {
        provider: Box::new(ConfiguredProviderPin::capture(&provider()).unwrap()),
        deployment: DeploymentLocator::Absolute(
            "/not-supplied/retained pressure/deployment.json".into(),
        ),
        requested_native_prefix_steps: 1,
        minimum_native_prefix_steps: 1,
    };
    f
}
fn reseal(spec: FrozenConfiguredSpec) -> FrozenConfiguredSpec {
    FrozenConfiguredSpec::new(spec.constructor, spec.future).unwrap()
}
fn set() -> ConfiguredRunSetRevision {
    let native = frozen();
    let configured =
        FrozenConfiguredSpec::new(native.constructor.clone(), configured_future()).unwrap();
    ConfiguredRunSetRevision {
        schema_version: CONFIGURED_VERSION,
        kind: ConfiguredSetKind::ConfiguredAcceptedSessionSet,
        set_id: "two-accepted-session-intents".into(),
        parent_sha256: None,
        baseline_member: "native".into(),
        members: vec![
            ConfiguredRunSetMember {
                member_id: "native".into(),
                spec: native.clone(),
                differences_from_baseline: vec![],
                attempt: Some(ConfiguredRunAttempt {
                    attempt_id: "native-first-attempt".into(),
                    archive_relative_path: "native.sqlite".into(),
                    expected_run_id: 100,
                    expected_source_revision: executor().source_revision,
                    finalized: None,
                }),
            },
            ConfiguredRunSetMember {
                member_id: "configured".into(),
                differences_from_baseline: ConfiguredSpecDifference::between(&native, &configured)
                    .unwrap(),
                spec: configured,
                attempt: Some(ConfiguredRunAttempt {
                    attempt_id: "configured-first-attempt".into(),
                    archive_relative_path: "configured.sqlite".into(),
                    expected_run_id: 101,
                    expected_source_revision: executor().source_revision,
                    finalized: None,
                }),
            },
        ],
    }
}

#[test]
fn actual_constructor_readback_roundtrips_every_raw_control_and_fresh_identity() {
    for method in [
        PressureImplementation::JacobiPcgV1,
        PressureImplementation::SymmetricGaussSeidelPcgV1,
    ] {
        let owner = constructor(method, 2.0);
        let spec = FrozenConfiguredSpec::capture_constructor(&owner.snapshot(), future()).unwrap();
        let c = &spec.constructor;
        assert_eq!(c.counts_xyz, [2, 1, 1]);
        assert_eq!(c.spacing_m, [0.25; 3]);
        assert_eq!(c.origin_m, [0.0; 3]);
        assert_eq!(c.density_kg_m3, 2.0);
        assert_eq!(c.represented_density_kg_m3, 2.0);
        assert_eq!(c.pressure_method.implementation(), method);
        assert_eq!(c.tolerances.relative_residual, 1e-12);
        assert_eq!(c.tolerances.absolute_residual_m3_s2, 1e-14);
        assert_eq!(c.tolerances.predicted_divergence_per_s, 1e-9);
        assert_eq!(c.tolerances.actual_divergence_per_s, 1e-6);
        assert_eq!(c.tolerances.max_courant, 0.75);
        assert_eq!(c.tolerances.max_pressure_iterations, 256);
        assert_eq!(c.simulation_memory_limit_bytes, CAP);
        assert_eq!(c.boundary_workspace_limit_bytes, 2 * CAP);
        assert_eq!(c.max_steps, 7);
        assert_eq!(c.volume_stamp, IdentityStamp { id: 17, version: 3 });
        assert_eq!(c.carrier_id, 18);
        let fresh = spec
            .create_session(SessionIdentity {
                run_id: 88,
                source_revision: executor().source_revision,
            })
            .unwrap();
        assert_eq!(fresh.snapshot().identity().run_id, 88);
        spec.validate_constructor(&fresh.snapshot()).unwrap();
        assert_eq!(fresh.snapshot().state().liquid.fraction, &[1.0; 2]);
        assert!(fresh.snapshot().accepted_interval_start().is_none());
        assert_eq!(fresh.snapshot().frame(), 0);
        assert!(
            spec.create_session(SessionIdentity {
                run_id: 89,
                source_revision: "unrelated-host-source".into()
            })
            .is_err()
        );
    }
}
#[test]
fn changed_copies_require_new_digest_and_exact_float_constructor_binding() {
    let spec = frozen();
    let original = spec.digest().unwrap();
    let mut changed = spec.clone();
    changed.future.volume.actual_divergence_per_s *= 2.0;
    assert!(matches!(
        changed.validate(),
        Err(RunSetError::Integrity("configured spec digest"))
    ));
    let changed = reseal(changed);
    assert_ne!(changed.digest().unwrap(), original);
    assert_eq!(spec.digest().unwrap(), original);
    let mut negative_zero = spec.clone();
    negative_zero.constructor.origin_m[0] = -0.0;
    let negative_zero = reseal(negative_zero);
    assert_ne!(negative_zero.digest().unwrap(), original);
    assert!(
        negative_zero
            .validate_constructor(&constructor(PressureImplementation::JacobiPcgV1, 2.0).snapshot())
            .is_err()
    );
    let actual = negative_zero
        .create_session(SessionIdentity {
            run_id: 90,
            source_revision: executor().source_revision,
        })
        .unwrap();
    negative_zero
        .validate_constructor(&actual.snapshot())
        .unwrap();
    let mut disabled = spec.clone();
    disabled.future.viscosity = ViscosityIntent::Disabled;
    disabled.future.viscosity_workspace_limit_bytes = None;
    let disabled = reseal(disabled);
    let mut zero_mu = spec.clone();
    zero_mu.future.viscosity = ViscosityIntent::NewtonianConstant {
        dynamic_viscosity_pa_s: 0.0,
    };
    let zero_mu = reseal(zero_mu);
    assert_ne!(disabled.digest().unwrap(), zero_mu.digest().unwrap());
    assert!(
        ConfiguredSpecDifference::between(&disabled, &zero_mu)
            .unwrap()
            .contains(&ConfiguredSpecDifference::Viscosity)
    );
}
#[test]
fn historical_open_requires_no_package_and_executor_custody_is_separate_from_model() {
    let dir = Directory::new();
    let owner = constructor(PressureImplementation::JacobiPcgV1, 2.0);
    let spec =
        FrozenConfiguredSpec::capture_constructor(&owner.snapshot(), configured_future()).unwrap();
    spec.save_new(&dir.0.join("historical.json")).unwrap();
    let opened = FrozenConfiguredSpec::open(&dir.0.join("historical.json")).unwrap();
    assert_eq!(opened, spec);
    opened.validate_execution(&executor()).unwrap();
    for field in 0..3 {
        let mut current = executor();
        match field {
            0 => current.id = "another-host".into(),
            1 => current.source_revision = "host-source-B".into(),
            _ => current.build_provenance = Some("now-specified-build".into()),
        };
        assert!(matches!(
            opened.validate_execution(&current),
            Err(RunSetError::Unsupported("captured host executor differs"))
        ));
    }
    if let PressureIntent::NativePrefixThenConfigured {
        provider: pin,
        deployment,
        ..
    } = &opened.future.pressure
    {
        assert_ne!(pin.source_revision, opened.future.executor.source_revision);
        assert_eq!(pin.identity(), provider());
        pin.validate_resolved_identity(&provider()).unwrap();
        let mut drift = provider();
        drift.manifest_sha256[0] ^= 1;
        assert!(pin.validate_resolved_identity(&drift).is_err());
        assert!(matches!(deployment, DeploymentLocator::Absolute(_)));
    } else {
        panic!("configured recipe lost");
    }
    assert!(
        FrozenConfiguredSpec::capture_constructor(
            &owner.snapshot(),
            ConfiguredFutureIntent {
                executor: ExecutorDeclaration {
                    source_revision: "other-source".into(),
                    ..executor()
                },
                ..future()
            }
        )
        .is_err()
    );
}
#[test]
fn immutable_two_variant_sets_roundtrip_and_native_schema_one_stays_separate() {
    let dir = Directory::new();
    let first = set();
    first.validate().unwrap();
    assert_eq!(
        first.members[1].differences_from_baseline,
        vec![ConfiguredSpecDifference::PressureSelection]
    );
    first.save_new(&dir.0.join("set-v2.json")).unwrap();
    let exact = std::fs::read(dir.0.join("set-v2.json")).unwrap();
    assert_eq!(
        ConfiguredRunSetRevision::open(&dir.0.join("set-v2.json")).unwrap(),
        first
    );
    first.members[0]
        .spec
        .save_new(&dir.0.join("spec-v2.json"))
        .unwrap();
    assert!(matches!(
        FrozenSpec::open(&dir.0.join("spec-v2.json")),
        Err(RunSetError::Unsupported(_))
    ));
    assert!(matches!(
        RunSetRevision::open(&dir.0.join("set-v2.json")),
        Err(RunSetError::Unsupported(_))
    ));
    let mut next = first.clone();
    next.parent_sha256 = Some(first.digest().unwrap());
    next.members[1].spec.future.requested_dt_s = 0.125;
    next.members[1].spec = reseal(next.members[1].spec.clone());
    next.members[1].differences_from_baseline =
        ConfiguredSpecDifference::between(&next.members[0].spec, &next.members[1].spec).unwrap();
    assert!(next.save_new(&dir.0.join("set-v2.json")).is_err());
    assert_eq!(std::fs::read(dir.0.join("set-v2.json")).unwrap(), exact);
    next.save_new(&dir.0.join("set-next.json")).unwrap();
    assert_ne!(next.digest().unwrap(), first.digest().unwrap());
    assert_eq!(
        ConfiguredRunSetRevision::open(&dir.0.join("set-v2.json")).unwrap(),
        first
    );
    // No archive is opened or outcome fabricated by an intent declaration.
    assert!(!dir.0.join("native.sqlite").exists());
    assert!(!dir.0.join("configured.sqlite").exists());
}
#[test]
fn unsupported_versions_kinds_unknown_fields_and_tampered_intent_fail_closed() {
    let dir = Directory::new();
    let spec = frozen();
    let original = serde_json::to_value(&spec).unwrap();
    for case in 0..7 {
        let mut v = original.clone();
        match case {
            0 => v["schema_version"] = json!(3),
            1 => v["kind"] = json!("native_carrier"),
            2 => v["future"]["extra"] = json!(true),
            3 => v["constructor"]["density_kg_m3"] = json!(3.0),
            4 => v["future"]["requested_dt_s"] = json!(0.125),
            5 => v["future"]["viscosity"]["new_stage"] = json!(true),
            _ => v["sha256"][0] = json!(255),
        }
        let path = dir.0.join(format!("bad-{case}.json"));
        std::fs::write(&path, serde_json::to_vec(&v).unwrap()).unwrap();
        assert!(FrozenConfiguredSpec::open(&path).is_err(), "case{case}");
    }
    let raw = serde_json::to_string(&spec).unwrap();
    let duplicate = raw.replacen(
        "\"schema_version\":2",
        "\"schema_version\":2,\"schema_version\":2",
        1,
    );
    std::fs::write(dir.0.join("duplicate.json"), duplicate).unwrap();
    assert!(FrozenConfiguredSpec::open(&dir.0.join("duplicate.json")).is_err());
    let path = dir.0.join("set-envelope.json");
    let valid_set = set();
    valid_set.save_new(&path).unwrap();
    assert_eq!(ConfiguredRunSetRevision::open(&path).unwrap(), valid_set);
    let original_packet: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    for nested in [false, true] {
        let mut packet = original_packet.clone();
        if nested {
            packet["revision"]["kind"] = json!("unsupported_set_kind");
        } else {
            packet["kind"] = json!("unsupported_set_kind");
        }
        let wrong = dir.0.join(format!("wrong-set-kind-{nested}.json"));
        std::fs::write(&wrong, serde_json::to_vec(&packet).unwrap()).unwrap();
        match ConfiguredRunSetRevision::open(&wrong) {
            Err(RunSetError::Codec(error)) => {
                assert!(error.to_string().contains("unknown variant"), "{error}");
                assert!(
                    error.to_string().contains("unsupported_set_kind"),
                    "{error}"
                );
            }
            other => panic!("expected unsupported kind decoding failure, got {other:?}"),
        }
    }
    let mut packet = original_packet;
    packet["revision"]["set_id"] = json!("changed-without-digest");
    std::fs::write(&path, serde_json::to_vec(&packet).unwrap()).unwrap();
    assert!(ConfiguredRunSetRevision::open(&path).is_err());
}
#[test]
fn deployment_locators_and_archive_paths_have_distinct_bounded_contracts() {
    for locator in [
        DeploymentLocator::ProjectRelative("packages/deployment.json".into()),
        DeploymentLocator::Absolute("/external/lab deployment.json".into()),
    ] {
        locator.validate().unwrap();
    }
    for locator in [
        DeploymentLocator::ProjectRelative("../deployment.json".into()),
        DeploymentLocator::ProjectRelative("/external/deployment.json".into()),
        DeploymentLocator::Absolute("relative.json".into()),
        DeploymentLocator::Absolute("/bad\nname".into()),
    ] {
        assert!(locator.validate().is_err());
    }
    for path in [
        "../escape.sqlite",
        "/external/archive.sqlite",
        "folder//bad.sqlite",
        "folder/./bad.sqlite",
        "C:\\bad.sqlite",
    ] {
        let mut revision = set();
        revision.members[0]
            .attempt
            .as_mut()
            .unwrap()
            .archive_relative_path = path.into();
        assert!(revision.validate().is_err());
    }
    let dir = Directory::new();
    let path = dir.0.join("too-large.json");
    std::fs::write(&path, vec![b' '; 1024 * 1024 + 1]).unwrap();
    assert!(FrozenConfiguredSpec::open(&path).is_err());
    assert!(FrozenConfiguredSpec::open(&dir.0).is_err());
    #[cfg(unix)]
    {
        let target = dir.0.join("good.json");
        frozen().save_new(&target).unwrap();
        let link = dir.0.join("link.json");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        assert!(FrozenConfiguredSpec::open(&link).is_err());
    }
}
#[test]
fn invalid_physics_resource_and_prefix_declarations_cannot_be_frozen() {
    for case in 0..13 {
        let mut s = frozen();
        match case {
            0 => s.constructor.represented_density_kg_m3 = 3.0,
            1 => s.constructor.tolerances.relative_residual = f64::NAN,
            2 => s.future.requested_steps = 8,
            3 => s.future.requested_dt_s = 0.0,
            4 => {
                s.future.viscosity = ViscosityIntent::NewtonianConstant {
                    dynamic_viscosity_pa_s: -1.0,
                }
            }
            5 => s.future.viscosity = ViscosityIntent::Disabled,
            6 => s.future.viscosity_workspace_limit_bytes = None,
            7 => s.future.provider_workspace_limit_bytes = 0,
            8 => s.future.forces[0].value[0] = f64::INFINITY,
            9 => s.future.forces[0].region.as_mut().unwrap().upper_m[0] = 2.0,
            10 => s.future.volume.actual_divergence_per_s = -1.0,
            11 => {
                s.future = configured_future();
                if let PressureIntent::NativePrefixThenConfigured {
                    requested_native_prefix_steps,
                    ..
                } = &mut s.future.pressure
                {
                    *requested_native_prefix_steps = 0;
                }
            }
            _ => {
                s.future = configured_future();
                if let PressureIntent::NativePrefixThenConfigured {
                    requested_native_prefix_steps,
                    ..
                } = &mut s.future.pressure
                {
                    *requested_native_prefix_steps = 3;
                }
            }
        }
        assert!(
            FrozenConfiguredSpec::new(s.constructor, s.future).is_err(),
            "case{case}"
        );
    }
    assert!(
        FrozenConfiguredSpec::capture_constructor(
            &constructor(PressureImplementation::JacobiPcgV1, 3.0).snapshot(),
            future()
        )
        .is_err()
    );
    let mut s = frozen()
        .create_session(SessionIdentity {
            run_id: 2,
            source_revision: executor().source_revision,
        })
        .unwrap();
    let mut w = ProviderStepWorkspace::new(
        s.snapshot().grid(),
        2.0,
        CAP,
        PressureImplementation::JacobiPcgV1,
    )
    .unwrap();
    let input = LiquidStepInputs {
        requested_dt: 1.0 / 60.0,
        smoke_source: None,
        forces: &[],
        inlet: LiquidInlet::new(VolumeStamp { id: 19, version: 0 }, [[0.0; 2]; 3]).unwrap(),
        source: None,
        volume: LiquidVolumeSettings {
            max_outward_courant: 1.0,
            actual_divergence_limit: 1e-5,
        },
    };
    s.step_with_configured_pressure(input, &mut w, None, None, None, |_| false)
        .unwrap();
    assert!(frozen().validate_constructor(&s.snapshot()).is_err());
    assert!(FrozenConfiguredSpec::capture_constructor(&s.snapshot(), future()).is_err());
}
#[test]
fn typed_force_conversion_and_set_difference_attempt_checks_are_exact() {
    let mut force = future().forces.remove(0);
    force.units = ForceUnitsSpec::ForceDensity;
    let native = force.body_force();
    assert_eq!(native.value, force.value);
    assert_eq!(native.units, ForceUnits::ForceDensity);
    assert_eq!(native.region.unwrap().upper, force.region.unwrap().upper_m);
    let mut revision = set();
    revision.members[1].differences_from_baseline.clear();
    assert!(revision.validate().is_err());
    let mut revision = set();
    revision.members[1]
        .attempt
        .as_mut()
        .unwrap()
        .archive_relative_path = "native.sqlite".into();
    assert!(revision.validate().is_err());
    let mut revision = set();
    revision.members[1]
        .attempt
        .as_mut()
        .unwrap()
        .expected_source_revision = "model-source-not-host".into();
    assert!(revision.validate().is_err());
    let mut revision = set();
    revision.members[1].member_id = "native".into();
    assert!(revision.validate().is_err());
    let mut revision = set();
    revision.members = vec![revision.members[0].clone(); 65];
    assert!(revision.validate().is_err());
    let mut id = provider();
    id.runtime = "".into();
    assert!(ConfiguredProviderPin::capture(&id).is_err());
    assert_eq!(
        frozen().constructor.pressure_method,
        NativePressureMethod::JacobiPcgV1
    );
}

#[test]
fn finalized_declarations_bound_the_durable_prefix_without_claiming_archive_verification() {
    let pin = |count, generation, outcome| FinalizedArchive {
        sha256: [9; 32],
        recorded_count: count,
        outcome,
        accepted_generation: generation,
    };
    for declaration in [
        pin(4, Some(3), TerminalOutcome::Completed),
        // A failed recorder may leave the constructor and one accepted step
        // durable while the owner had already accepted a later step.
        pin(2, Some(3), TerminalOutcome::Failed),
        pin(0, None, TerminalOutcome::Cancelled),
    ] {
        let mut revision = set();
        revision.members[0].attempt.as_mut().unwrap().finalized = Some(declaration);
        revision.validate().unwrap();
        let dir = Directory::new();
        let path = dir.0.join("declaration-only.json");
        revision.save_new(&path).unwrap();
        assert_eq!(ConfiguredRunSetRevision::open(&path).unwrap(), revision);
        assert!(!dir.0.join("native.sqlite").exists());
    }
    for declaration in [
        pin(5, Some(3), TerminalOutcome::Failed),
        pin(2, Some(4), TerminalOutcome::Failed),
        pin(3, Some(1), TerminalOutcome::Cancelled),
        pin(1, None, TerminalOutcome::Failed),
        pin(3, Some(3), TerminalOutcome::Completed),
        pin(4, Some(2), TerminalOutcome::Completed),
    ] {
        let mut revision = set();
        revision.members[0].attempt.as_mut().unwrap().finalized = Some(declaration);
        assert!(revision.validate().is_err());
    }
}
