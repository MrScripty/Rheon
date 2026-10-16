//! Existing track/Session/process owners with independent analytic CPU fixtures.
//! No trained checkpoint, GUI workflow or recording persistence is asserted.
use super::super::cpu_contract_tests::Fixture;
#[cfg(target_os = "linux")]
use super::super::cpu_contract_tests::assert_child_liveness;
use super::*;
use crate::run_sets::{ArchiveBudget, configured::*};
use rheon::{solver_provider::configured::ConfiguredProviderReport, *};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    sync::Mutex,
    thread,
    time::{Duration, Instant},
};

const CAP: usize = 1024 * 1024;
const RHO: f64 = 2.0;
const MU: f64 = 1.0 / 128.0;
const DT: f64 = 1.0 / 16.0;
fn executor() -> ExecutorDeclaration {
    ExecutorDeclaration {
        id: "configured-track-contract-host".into(),
        source_revision: "configured-track-host-fixture".into(),
        build_provenance: None,
    }
}
fn identity(run: u64) -> SessionIdentity {
    SessionIdentity {
        run_id: run,
        source_revision: executor().source_revision,
    }
}
fn geometry(counts: [u64; 3]) -> GridGeometry {
    GridGeometry::new(counts, [0.25; 3], [0.0; 3]).unwrap()
}
fn configuration() -> LiquidTransportConfig {
    LiquidTransportConfig {
        carrier: SimulationConfig {
            density: RHO,
            memory_limit: CAP,
            pressure: PressureSettings {
                relative_residual: 1e-12,
                absolute_residual: 1e-12,
                divergence_limit: 1e-9,
                max_iterations: 1000,
            },
            actual_divergence_limit: 1e-5,
            max_courant: 0.5,
        },
        represented_density: RHO,
        volume_stamp: VolumeStamp {
            id: 710,
            version: 0,
        },
        carrier_id: 711,
    }
}
fn edit(f: &Fixture, action: impl FnOnce(&mut Value)) {
    let mut manifest: Value = serde_json::from_slice(&std::fs::read(&f.manifest).unwrap()).unwrap();
    action(&mut manifest);
    std::fs::write(&f.manifest, serde_json::to_vec(&manifest).unwrap()).unwrap();
}
fn package(
    correction: bool,
    minimum: usize,
    required_prior: bool,
    mu: f64,
    dt: f64,
    behavior: &str,
) -> Fixture {
    let source = include_str!("../../../../tests/fixtures/configured_box_pressure_adapter.py")
        .replace("BEHAVIOR = \"solve\"", &format!("BEHAVIOR = {behavior:?}"));
    let f = Fixture::source(&source, correction);
    edit(&f, |m| {
        m["id"] = json!(if correction {
            "independent-box-correction"
        } else {
            "independent-box-full"
        });
        m["support"] = json!({
            "density_kg_m3": RHO,
            "dynamic_viscosity_pa_s": mu,
            "dt_s": dt,
            "counts_min": [1, 1, 1],
            "counts_max": [4, 4, 4],
            "max_cells": 64,
            "spacing_min_m": 0.25,
            "spacing_max_m": 0.25,
            "origin": "zero",
            "cells": "cubic"
        });
        m["context"] = json!({"minimum_native_steps": minimum, "prior_pressure": if required_prior {"required"} else {"optional"}});
        m["inputs"] = if correction {
            json!({
                "b": "rhs",
                "r": "initial_residual",
                "physical_z": "z",
                "rho": "density",
                "delta_t": "dt",
                "p0": "previous_pressure"
            })
        } else {
            let mut fields = json!({
                "equation_rhs": "rhs",
                "full_residual": "initial_residual",
                "centres_y": "y",
                "material_density": "density",
                "actual_dt": "dt"
            });
            if required_prior {
                fields["solved_previous"] = json!("previous_pressure");
            }
            fields
        };
    });
    f
}
fn pin(f: &Fixture) -> ConfiguredProviderPin {
    let package = AdapterPackage::load(&f.manifest).unwrap();
    let sha = package.manifest_sha256();
    ConfiguredProviderPin {
        id: package.id().into(),
        version: package.version().into(),
        manifest_sha256: std::array::from_fn(|i| {
            u8::from_str_radix(&sha[2 * i..2 * i + 2], 16).unwrap()
        }),
        checkpoint_sha256: None,
        training_run_id: None,
        runtime: "independent stdlib rational sealed pressure fixture".into(),
        source_revision: "authored-box-fixture-source".into(),
    }
}
fn spec(
    fixture: Option<&Fixture>,
    counts: [u64; 3],
    prefix: usize,
    mu: Option<f64>,
    dt: f64,
) -> FrozenConfiguredSpec {
    let grid = geometry(counts);
    let constructor = SimulationSession::new(
        identity(771),
        grid.clone(),
        configuration(),
        PressureImplementation::JacobiPcgV1,
        vec![1.0; grid.cell_len()],
        CAP,
        8,
    )
    .unwrap();
    let pressure = fixture.map_or(PressureIntent::NativeOnly, |f| {
        PressureIntent::NativePrefixThenConfigured {
            provider: Box::new(pin(f)),
            deployment: DeploymentLocator::ProjectRelative("adapter.json".into()),
            requested_native_prefix_steps: prefix,
            minimum_native_prefix_steps: AdapterPackage::load(&f.manifest)
                .unwrap()
                .context()
                .minimum_native_steps,
        }
    });
    FrozenConfiguredSpec::capture_constructor(
        &constructor.snapshot(),
        ConfiguredFutureIntent {
            requested_dt_s: dt,
            requested_steps: 3,
            forces: vec![ForceSpec {
                value: [0.125, 0.0, 0.0],
                units: ForceUnitsSpec::Acceleration,
                region: None,
            }],
            volume: VolumeControls {
                max_outward_courant: 1.0,
                actual_divergence_per_s: 1e-5,
            },
            viscosity: mu.map_or(ViscosityIntent::Disabled, |dynamic_viscosity_pa_s| {
                ViscosityIntent::NewtonianConstant {
                    dynamic_viscosity_pa_s,
                }
            }),
            pressure,
            provider_workspace_limit_bytes: CAP,
            viscosity_workspace_limit_bytes: mu.map(|_| CAP),
            recording: ArchiveBudget {
                max_database_bytes: 8 * CAP as u64,
                max_record_bytes: CAP,
                max_records: 4,
            },
            capture_constructor: true,
            executor: executor(),
        },
    )
    .unwrap()
}
fn reseal(spec: FrozenConfiguredSpec) -> FrozenConfiguredSpec {
    FrozenConfiguredSpec::new(spec.constructor, spec.future).unwrap()
}
fn prepare(spec: FrozenConfiguredSpec, flag: Arc<AtomicBool>) -> ConfiguredTrack {
    ConfiguredTrack::prepare(spec, identity(771), &executor(), flag).unwrap()
}
fn slot() -> ChildSlot {
    Arc::new(Mutex::new(None))
}
fn launch(track: &mut ConfiguredTrack, f: &Fixture, slot: &ChildSlot) {
    track.start_provider(&f.root, slot, |_| Ok(())).unwrap();
}
fn received(f: &Fixture) -> Vec<Value> {
    match std::fs::read_to_string(f.root.join("requests.jsonl")) {
        Ok(raw) => raw
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => vec![],
        Err(e) => panic!("{e}"),
    }
}
fn bits(v: LiquidTransportView<'_>) -> Vec<u64> {
    let mut bits = v
        .pressure
        .iter()
        .chain(v.liquid.fraction)
        .map(|x| x.to_bits())
        .collect::<Vec<_>>();
    bits.extend(
        v.carrier
            .x
            .iter()
            .chain(v.carrier.y)
            .chain(v.carrier.z)
            .chain(v.carrier.tracer)
            .map(|v| u64::from(v.to_bits())),
    );
    bits.extend([
        v.carrier.generation,
        v.carrier.time.to_bits(),
        v.liquid.time.to_bits(),
        v.carrier_stamp.id,
        v.carrier_stamp.version,
        v.liquid.stamp.id,
        v.liquid.stamp.version,
    ]);
    bits
}
fn publication(t: &ConfiguredTrack) -> Vec<u8> {
    let mut bytes = vec![];
    t.snapshot().write_dense3d(&mut bytes).unwrap();
    bytes
}
fn assert_receipt(a: &ConfiguredProviderReport, b: &ConfiguredProviderReport) {
    assert_eq!(a.binding, b.binding);
    assert_eq!(a.context, b.context);
    assert_eq!(a.dynamic_viscosity.to_bits(), b.dynamic_viscosity.to_bits());
    assert_eq!(a.prediction_called, b.prediction_called);
    let bits = |p: PressureReport| {
        (
            p.iterations,
            p.true_residual_l2.to_bits(),
            p.true_residual_max.to_bits(),
            p.predicted_divergence_max.to_bits(),
        )
    };
    let (a, b) = (&a.pressure, &b.pressure);
    assert_eq!(a.selected, b.selected);
    assert_eq!(a.attempted, b.attempted);
    assert_eq!(a.rejection, b.rejection);
    assert_eq!(a.candidate.map(bits), b.candidate.map(bits));
    assert_eq!(bits(a.accepted), bits(b.accepted));
    assert_eq!(
        (a.total, a.inference, a.qualification, a.fallback),
        (b.total, b.inference, b.qualification, b.fallback)
    );
}
fn retire(track: &mut ConfiguredTrack, slot: &ChildSlot) {
    let pid = slot.lock().unwrap().as_ref().map(|child| child.id());
    track.retire_runtime();
    assert!(slot.lock().unwrap().is_none());
    if let Some(pid) = pid {
        #[cfg(target_os = "linux")]
        assert_child_liveness(pid, false);
    }
}

#[test]
fn frozen_track_full_and_correction_follow_real_native_prefix_and_exact_native_fields() {
    for correction in [false, true] {
        let f = package(correction, 1, true, MU, DT, "solve");
        let frozen = spec(Some(&f), [2, 1, 1], 1, Some(MU), DT);
        let mut track = prepare(frozen.clone(), Arc::new(AtomicBool::new(false)));
        assert_eq!(track.geometry(), &geometry([2, 1, 1]));
        assert_eq!(
            track.native_implementation(),
            PressureImplementation::JacobiPcgV1
        );
        assert_eq!(track.frozen(), &frozen);
        assert_eq!(track.inlet().fractions(), [[0.0; 2]; 3]);
        assert_ne!(track.inlet().stamp().id, frozen.constructor.carrier_id);
        assert_ne!(track.inlet().stamp().id, frozen.constructor.volume_stamp.id);
        let before = track.host_array_bytes();
        assert_eq!(before.cpu_adapter_scratch, 0);
        let slot = slot();
        launch(&mut track, &f, &slot);
        assert_eq!(track.host_array_bytes().cpu_adapter_scratch, 64 * 16);
        assert_eq!(track.host_array_bytes().total(), before.total() + 64 * 16);
        let mut native = LiquidTransportSimulation::new(
            geometry([2, 1, 1]),
            configuration(),
            PressureImplementation::JacobiPcgV1,
            vec![1.0; 2],
        )
        .unwrap();
        let mut v = ViscosityWorkspace::new(geometry([2, 1, 1]), CAP).unwrap();
        let forces: Vec<_> = frozen
            .future
            .forces
            .iter()
            .map(ForceSpec::body_force)
            .collect();
        for generation in 1..=3 {
            let prior = track.snapshot().state().pressure.to_vec();
            let report = track.advance(None).unwrap();
            native
                .step_viscous(
                    LiquidStepInputs {
                        requested_dt: DT,
                        smoke_source: None,
                        forces: &forces,
                        inlet: track.inlet(),
                        source: None,
                        volume: frozen.future.volume.settings(),
                    },
                    &mut v,
                    MU,
                    |_| false,
                )
                .unwrap();
            assert_eq!(bits(track.snapshot().state()), bits(native.state()));
            assert_eq!(report.carrier.step.dt.to_bits(), DT.to_bits());
            assert!(report.viscosity.is_some());
            let receipt = track.snapshot().configured_provider_report().unwrap();
            if generation == 1 {
                assert!(!receipt.prediction_called);
                assert!(receipt.binding.is_none());
                assert!(received(&f).is_empty());
            } else {
                assert!(receipt.prediction_called);
                assert!(receipt.pressure.rejection.is_none());
                assert_eq!(receipt.context.unwrap().native_prefix_steps, 1);
                let packet = received(&f).pop().unwrap();
                assert_eq!(packet["context"]["generation"], generation);
                assert_eq!(
                    packet["inputs"][if correction { "p0" } else { "solved_previous" }]["values"],
                    json!(prior)
                );
                assert_ne!(
                    packet["inputs"][if correction { "b" } else { "equation_rhs" }]["values"],
                    json!([0.0, 0.0])
                );
                assert_ne!(
                    receipt.binding.as_ref().unwrap().source_revision,
                    track.snapshot().identity().source_revision
                );
            }
            assert!(track.current_failure().is_none());
        }
        assert_eq!(received(&f).len(), 2);
        let saved = publication(&track);
        assert!(matches!(track.advance(None), Err(SessionError::StepLimit)));
        assert_eq!(publication(&track), saved);
        retire(&mut track, &slot);
        assert_eq!(track.host_array_bytes(), before);
    }
}

#[test]
fn independent_eight_cell_cpu_candidate_qualifies_nonzero_rhs_without_prior_at_phase_zero() {
    let f = package(false, 0, false, MU, DT, "solve");
    let frozen = spec(Some(&f), [2; 3], 0, Some(MU), DT);
    let mut track = prepare(frozen, Arc::new(AtomicBool::new(false)));
    let slot = slot();
    launch(&mut track, &f, &slot);
    for generation in 1..=2 {
        track.advance(None).unwrap();
        let view = track.snapshot();
        let receipt = view.configured_provider_report().unwrap();
        assert!(receipt.prediction_called);
        assert!(receipt.pressure.rejection.is_none());
        assert_eq!(
            receipt.context.unwrap().prior_generation,
            (generation > 1).then_some(generation - 1)
        );
        assert_eq!(receipt.context.unwrap().native_prefix_steps, 0);
        assert!(
            view.state()
                .carrier
                .x
                .iter()
                .chain(view.state().carrier.y)
                .chain(view.state().carrier.z)
                .all(|v| *v == 0.0)
        );
        assert_eq!(view.state().liquid.fraction, &[1.0; 8]);
        let packets = received(&f);
        assert_eq!(
            packets.last().unwrap()["context"]["grid"]["counts_xyz"],
            json!([2, 2, 2])
        );
        assert_ne!(
            packets.last().unwrap()["inputs"]["equation_rhs"]["values"],
            json!(vec![0.0; 8])
        );
    }
    retire(&mut track, &slot);
}

#[test]
fn disabled_and_newtonian_zero_remain_distinct_and_native_only_start_is_inert() {
    for mu in [None, Some(0.0)] {
        let frozen = spec(None, [2, 1, 1], 0, mu, DT);
        let mut track = prepare(frozen.clone(), Arc::new(AtomicBool::new(false)));
        let slot = slot();
        track
            .start_provider(
                Path::new("/nonexistent/unused/native-project"),
                &slot,
                |_| panic!("native-only start must be inert"),
            )
            .unwrap();
        assert!(!track.runtime_ready());
        assert!(slot.lock().unwrap().is_none());
        assert_eq!(
            track.host_array_bytes().viscosity_workspace > 0,
            mu.is_some()
        );
        let report = track.advance(None).unwrap();
        assert_eq!(report.viscosity.is_some(), mu.is_some());
        assert!(
            track
                .snapshot()
                .configured_provider_report()
                .unwrap()
                .binding
                .is_none()
        );
        assert_eq!(track.frozen(), &frozen);
    }
}

#[test]
fn executor_package_prefix_physics_and_caps_fail_before_child_and_preserve_constructor() {
    for case in 0..9 {
        let f = package(false, 1, true, MU, DT, "solve");
        let mut frozen = spec(Some(&f), [2, 1, 1], 1, Some(MU), DT);
        if case == 6 {
            edit(&f, |m| {
                m["support"]["counts_max"] = json!([128, 128, 128]);
                m["support"]["max_cells"] = json!(65536);
            });
            frozen = spec(Some(&f), [2, 1, 1], 1, Some(MU), DT);
            frozen.future.provider_workspace_limit_bytes = 8192;
        }
        match case {
            0 => {
                let mut wrong = executor();
                wrong.id.push_str("-other");
                assert!(
                    ConfiguredTrack::prepare(
                        frozen,
                        identity(771),
                        &wrong,
                        Arc::new(AtomicBool::new(false))
                    )
                    .is_err()
                );
                continue;
            }
            1 => {
                let mut wrong = identity(771);
                wrong.source_revision.push_str("-other");
                assert!(
                    ConfiguredTrack::prepare(
                        frozen,
                        wrong,
                        &executor(),
                        Arc::new(AtomicBool::new(false))
                    )
                    .is_err()
                );
                continue;
            }
            2 => {
                if let PressureIntent::NativePrefixThenConfigured { provider, .. } =
                    &mut frozen.future.pressure
                {
                    provider.manifest_sha256[0] ^= 1;
                }
            }
            3 => {
                if let PressureIntent::NativePrefixThenConfigured {
                    minimum_native_prefix_steps,
                    ..
                } = &mut frozen.future.pressure
                {
                    *minimum_native_prefix_steps = 0;
                }
            }
            4 => {
                frozen.constructor.density_kg_m3 = 3.0;
                frozen.constructor.represented_density_kg_m3 = 3.0;
            }
            5 => {
                if let PressureIntent::NativePrefixThenConfigured { provider, .. } =
                    &mut frozen.future.pressure
                {
                    provider.checkpoint_sha256 = Some([9; 32]);
                }
            }
            7 => {
                frozen.future.provider_workspace_limit_bytes = 1;
                let frozen = reseal(frozen);
                assert!(
                    ConfiguredTrack::prepare(
                        frozen,
                        identity(771),
                        &executor(),
                        Arc::new(AtomicBool::new(false))
                    )
                    .is_err()
                );
                continue;
            }
            8 => {
                frozen.future.viscosity_workspace_limit_bytes = Some(1);
                let frozen = reseal(frozen);
                assert!(
                    ConfiguredTrack::prepare(
                        frozen,
                        identity(771),
                        &executor(),
                        Arc::new(AtomicBool::new(false))
                    )
                    .is_err()
                );
                continue;
            }
            _ => {}
        }
        frozen = reseal(frozen);
        let mut track = prepare(frozen, Arc::new(AtomicBool::new(false)));
        let constructor = publication(&track);
        let slot = slot();
        assert!(
            track
                .start_provider(&f.root, &slot, |_| panic!(
                    "prelaunch refusal must precede progress"
                ))
                .is_err(),
            "case {case}"
        );
        assert_eq!(publication(&track), constructor);
        assert_eq!(track.snapshot().frame(), 0);
        assert!(slot.lock().unwrap().is_none());
        assert!(!f.root.join("child-started").exists());
    }
    let f = package(true, 0, true, MU, DT, "solve");
    let mut track = prepare(
        spec(Some(&f), [2, 1, 1], 0, Some(MU), DT),
        Arc::new(AtomicBool::new(false)),
    );
    let slot = slot();
    assert!(
        track.start_provider(&f.root, &slot, |_| Ok(())).is_err(),
        "closed profile refuses prefix0 when prior required"
    );
    assert!(!f.root.join("child-started").exists());
}

#[test]
fn missing_retired_or_occupied_runtime_never_silently_substitutes_native_configuration() {
    let f = package(false, 1, true, MU, DT, "solve");
    let frozen = spec(Some(&f), [2, 1, 1], 1, Some(MU), DT);
    let mut track = prepare(frozen.clone(), Arc::new(AtomicBool::new(false)));
    track.advance(None).unwrap();
    let before = publication(&track);
    assert!(matches!(
        track.advance(None),
        Err(SessionError::PressureBinding(_))
    ));
    assert_eq!(publication(&track), before);
    let slot = slot();
    launch(&mut track, &f, &slot);
    let pid = slot.lock().unwrap().as_ref().unwrap().id();
    let mut other = prepare(frozen, Arc::new(AtomicBool::new(false)));
    assert!(other.start_provider(&f.root, &slot, |_| Ok(())).is_err());
    assert_eq!(slot.lock().unwrap().as_ref().unwrap().id(), pid);
    retire(&mut track, &slot);
    assert!(matches!(
        track.advance(None),
        Err(SessionError::PressureBinding(_))
    ));
    assert_eq!(publication(&track), before);
    assert!(track.current_failure().is_none());
}

#[test]
fn host_shortened_dt_support_declines_without_fake_dispatch_and_retains_native_step() {
    let f = package(false, 0, false, MU, 2.0, "solve");
    let mut frozen = spec(Some(&f), [2, 1, 1], 0, Some(MU), 2.0);
    frozen.constructor.tolerances.max_courant = 1.0 / 128.0;
    let frozen = reseal(frozen);
    let mut track = prepare(frozen, Arc::new(AtomicBool::new(false)));
    let slot = slot();
    launch(&mut track, &f, &slot);
    let report = track.advance(None).unwrap();
    assert!(report.carrier.step.dt < 2.0);
    let receipt = track.snapshot().configured_provider_report().unwrap();
    assert!(!receipt.prediction_called);
    assert!(receipt.context.is_some());
    assert!(matches!(
        receipt.pressure.rejection,
        Some(ProviderRejection::Unsupported(_))
    ));
    assert_eq!(
        receipt.context.unwrap().dt.to_bits(),
        report.carrier.step.dt.to_bits()
    );
    assert!(received(&f).is_empty());
    assert!(track.current_failure().is_none());
    assert!(track.runtime_ready());
    retire(&mut track, &slot);
}

#[test]
fn host_rejected_candidate_keeps_runtime_and_native_fallback_as_next_accepted_prior() {
    let f = package(true, 1, true, MU, DT, "bad_once");
    let frozen = spec(Some(&f), [2, 1, 1], 1, Some(MU), DT);
    let mut track = prepare(frozen, Arc::new(AtomicBool::new(false)));
    let slot = slot();
    launch(&mut track, &f, &slot);
    track.advance(None).unwrap();
    track.advance(None).unwrap();
    let receipt = track.snapshot().configured_provider_report().unwrap();
    assert_eq!(
        receipt.pressure.rejection,
        Some(ProviderRejection::QualityGate)
    );
    assert!(receipt.pressure.candidate.is_some());
    assert!(track.current_failure().is_none());
    assert!(
        track.runtime_ready(),
        "host quality rejection is not a runtime failure"
    );
    let native_prior = track.snapshot().state().pressure.to_vec();
    track.advance(None).unwrap();
    let receipt = track.snapshot().configured_provider_report().unwrap();
    assert!(receipt.pressure.rejection.is_none());
    assert_eq!(receipt.context.unwrap().native_prefix_steps, 2);
    assert_eq!(
        received(&f)[1]["inputs"]["p0"]["values"],
        json!(native_prior)
    );
    assert!(track.current_failure().is_none());
    retire(&mut track, &slot);
}

#[test]
fn real_runtime_failure_and_invalid_output_retire_after_truthful_fallback_and_allow_explicit_fresh_owner()
 {
    for behavior in ["exit_once", "invalid_once"] {
        let f = package(false, 1, true, MU, DT, behavior);
        let frozen = spec(Some(&f), [2, 1, 1], 1, Some(MU), DT);
        let mut track = prepare(frozen, Arc::new(AtomicBool::new(false)));
        let slot = slot();
        launch(&mut track, &f, &slot);
        track.advance(None).unwrap();
        track.advance(None).unwrap();
        let failure = track.current_failure().unwrap().clone();
        assert!(failure.dispatched);
        assert!(!track.runtime_ready());
        assert!(slot.lock().unwrap().is_none());
        assert_eq!(
            failure.reason,
            if behavior == "exit_once" {
                NoCandidateReason::RuntimeFailure
            } else {
                NoCandidateReason::InvalidOutput
            }
        );
        assert_eq!(
            failure.invalid_payload_sha256,
            if behavior == "invalid_once" {
                Some(Sha256::digest(b"{\"kind\":\"candidate\",\"bogus\":1}\n").into())
            } else {
                None
            }
        );
        assert!(failure.detail.len() <= 8000 && !failure.detail.chars().any(char::is_control));
        let prior = track.snapshot().state().pressure.to_vec();
        assert!(
            track
                .snapshot()
                .configured_provider_report()
                .unwrap()
                .pressure
                .rejection
                .is_some()
        );
        let accepted = publication(&track);
        assert!(track.advance(None).is_err());
        assert_eq!(publication(&track), accepted);
        assert!(
            track.current_failure().is_none(),
            "current attempt must not reuse a prior IPC failure"
        );
        launch(&mut track, &f, &slot); // Explicit API retry, not an implicit product resume.
        track.advance(None).unwrap();
        assert!(
            track
                .snapshot()
                .configured_provider_report()
                .unwrap()
                .pressure
                .rejection
                .is_none()
        );
        assert_eq!(
            received(&f)[1]["inputs"]["solved_previous"]["values"],
            json!(prior)
        );
        assert!(track.current_failure().is_none());
        retire(&mut track, &slot);
    }
}

#[test]
fn real_inflight_cancellation_and_deadline_keep_accepted_receipt_fields() {
    for deadline in [false, true] {
        let f = package(false, 1, true, MU, DT, "stall_once");
        let frozen = spec(Some(&f), [2, 1, 1], 1, Some(MU), DT);
        let flag = Arc::new(AtomicBool::new(false));
        let mut track = prepare(frozen.clone(), Arc::clone(&flag));
        let slot = slot();
        launch(&mut track, &f, &slot);
        track.advance(None).unwrap();
        let saved = publication(&track);
        let receipt = track
            .snapshot()
            .configured_provider_report()
            .unwrap()
            .clone();
        let marker = f.root.join("first-request-observed");
        let cancel_thread = (!deadline).then(|| {
            let marker = marker.clone();
            let flag = Arc::clone(&flag);
            thread::spawn(move || {
                let limit = Instant::now() + Duration::from_secs(3);
                while !marker.exists() && Instant::now() < limit {
                    thread::sleep(Duration::from_millis(2));
                }
                assert!(marker.exists(), "cancel only after request consumption");
                flag.store(true, Ordering::Relaxed);
            })
        });
        let began = Instant::now();
        let result = track.advance(deadline.then_some(began + Duration::from_millis(500)));
        if let Some(cancel_thread) = cancel_thread {
            cancel_thread.join().unwrap();
        }
        if deadline {
            assert!(matches!(result, Err(SessionError::Deadline)));
        } else {
            assert!(result.is_err());
        }
        assert!(marker.exists());
        assert!(began.elapsed() < Duration::from_secs(2));
        assert_eq!(publication(&track), saved);
        assert_eq!(track.snapshot().frame(), 1);
        assert_receipt(
            &receipt,
            track.snapshot().configured_provider_report().unwrap(),
        );
        let failure = track.current_failure().unwrap();
        assert_eq!(failure.reason, NoCandidateReason::Cancelled);
        assert!(failure.dispatched);
        assert!(!track.runtime_ready());
        assert!(slot.lock().unwrap().is_none());
        assert!(flag.load(Ordering::Relaxed));
        // A new run owns a fresh flag and constructor. No old history is restored.
        let mut fresh = prepare(frozen, Arc::new(AtomicBool::new(false)));
        launch(&mut fresh, &f, &slot);
        fresh.advance(None).unwrap();
        fresh.advance(None).unwrap();
        assert_eq!(fresh.snapshot().frame(), 2);
        assert!(
            fresh
                .snapshot()
                .configured_provider_report()
                .unwrap()
                .pressure
                .rejection
                .is_none()
        );
        retire(&mut fresh, &slot);
    }
}
