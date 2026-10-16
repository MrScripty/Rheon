//! Production bridge/Session contracts with an independent stdlib analytic
//! child. These exercise runtime composition, not a trained checkpoint.
use super::super::cpu_contract_tests::Fixture;
#[cfg(target_os = "linux")]
use super::super::cpu_contract_tests::assert_child_liveness;
use super::*;
use rheon::solver_provider::configured::ConfiguredProviderReport;
use rheon::*;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

const CAP: usize = 1024 * 1024;
const RHO: f64 = 2.0;
const MU: f64 = 1.0 / 128.0;
const DT: f64 = 1.0 / 16.0;
fn grid() -> GridGeometry {
    GridGeometry::new([2, 1, 1], [0.25; 3], [0.0; 3]).unwrap()
}
fn config() -> LiquidTransportConfig {
    LiquidTransportConfig {
        carrier: SimulationConfig {
            density: RHO,
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
        represented_density: RHO,
        volume_stamp: VolumeStamp {
            id: 710,
            version: 0,
        },
        carrier_id: 711,
    }
}
fn session(quota: usize) -> SimulationSession {
    SimulationSession::new(
        SessionIdentity {
            run_id: 771,
            source_revision: "bridge-host-fixture".into(),
        },
        grid(),
        config(),
        PressureImplementation::JacobiPcgV1,
        vec![1.0; 2],
        CAP,
        quota,
    )
    .unwrap()
}
fn workspaces() -> (ProviderStepWorkspace, ViscosityWorkspace) {
    (
        ProviderStepWorkspace::new(&grid(), RHO, CAP, PressureImplementation::JacobiPcgV1).unwrap(),
        ViscosityWorkspace::new(grid(), CAP).unwrap(),
    )
}
fn inputs() -> LiquidStepInputs<'static> {
    static FORCES: [BodyForce; 1] = [BodyForce {
        value: [0.125, 0.0, 0.0],
        units: ForceUnits::Acceleration,
        region: None,
    }];
    LiquidStepInputs {
        requested_dt: DT,
        smoke_source: None,
        forces: &FORCES,
        inlet: LiquidInlet::new(
            VolumeStamp {
                id: 712,
                version: 0,
            },
            [[0.0; 2]; 3],
        )
        .unwrap(),
        source: None,
        volume: LiquidVolumeSettings {
            max_outward_courant: 1.0,
            actual_divergence_limit: 1e-5,
        },
    }
}
fn edit(fixture: &Fixture, f: impl FnOnce(&mut Value)) {
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(&fixture.manifest).unwrap()).unwrap();
    f(&mut manifest);
    std::fs::write(&fixture.manifest, serde_json::to_vec(&manifest).unwrap()).unwrap();
}
fn package(correction: bool, behavior: &str) -> Fixture {
    let source = include_str!("../../../../tests/fixtures/accepted_pressure_adapter.py")
        .replace("BEHAVIOR = \"solve\"", &format!("BEHAVIOR = {behavior:?}"))
        .replace("    role = manifest[\"output\"][\"role\"]", r#"    if BEHAVIOR == "malformed":
        print('{"kind":"candidate","bogus":1}', flush=True)
        continue
    if BEHAVIOR == "unsupported_context":
        print(json.dumps(dict(kind="error", code="unsupported_context", id=request["id"], message="fixture context declined")), flush=True)
        continue
    role = manifest["output"]["role"]"#);
    let source = format!(
        "from pathlib import Path\nPath(__file__).with_name('child-started').write_text('started')\n{source}"
    );
    let fixture = Fixture::source(&source, correction);
    edit(&fixture, |m| {
        m["support"] = json!({
            "density_kg_m3": RHO,
            "dynamic_viscosity_pa_s": MU,
            "dt_s": DT,
            "counts_min": [2, 1, 1],
            "counts_max": [2, 1, 1],
            "max_cells": 2,
            "spacing_min_m": 0.25,
            "spacing_max_m": 0.25,
            "origin": "zero",
            "cells": "cubic"
        });
        m["context"] = json!({
            "minimum_native_steps": 1,
            "prior_pressure": "required"
        });
        m["inputs"] = if correction {
            json!({
                "b": "rhs",
                "p0": "previous_pressure",
                "r": "initial_residual",
                "x_centre": "x",
                "rho": "density",
                "delta_t": "dt"
            })
        } else {
            json!({
                "equation_rhs": "rhs",
                "solved_previous": "previous_pressure",
                "full_residual": "initial_residual",
                "physical_x": "x",
                "material_density": "density",
                "actual_interval": "dt"
            })
        };
    });
    fixture
}
fn identity(fixture: &Fixture) -> Arc<ContextProviderIdentity> {
    let p = AdapterPackage::load(&fixture.manifest).unwrap();
    let digest = p.manifest_sha256();
    Arc::new(ContextProviderIdentity {
        id: p.id().into(),
        version: p.version().into(),
        manifest_sha256: std::array::from_fn(|i| {
            u8::from_str_radix(&digest[2 * i..2 * i + 2], 16).unwrap()
        }),
        checkpoint_sha256: None,
        training_run_id: None,
        runtime: "analytic stdlib CPU fixture".into(),
        source_revision: "declared-fixture-model-source".into(),
    })
}
fn start(fixture: &Fixture) -> (AcceptedCpuProvider, ChildSlot) {
    let slot = Arc::new(Mutex::new(None));
    let provider = AcceptedCpuProvider::start(
        &fixture.manifest,
        identity(fixture),
        Arc::clone(&slot),
        Arc::new(AtomicBool::new(false)),
        CAP,
        |_| Ok(()),
    )
    .unwrap();
    (provider, slot)
}
fn drain(provider: AcceptedCpuProvider, slot: ChildSlot) {
    let pid = slot.lock().unwrap().as_ref().unwrap().id();
    drop(provider);
    assert!(slot.lock().unwrap().is_none());
    #[cfg(target_os = "linux")]
    assert_child_liveness(pid, false);
}
fn received(f: &Fixture) -> Vec<Value> {
    match std::fs::read_to_string(f.root.join("requests.jsonl")) {
        Ok(raw) => raw
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => vec![],
        Err(e) => panic!("{e}"),
    }
}
fn state_bits(v: LiquidTransportView<'_>) -> Vec<u64> {
    let mut bits = v
        .pressure
        .iter()
        .chain(v.liquid.fraction)
        .map(|v| v.to_bits())
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
        v.carrier_stamp.version,
        v.liquid.stamp.version,
    ]);
    bits
}
fn publication(s: &SimulationSession) -> Vec<u8> {
    let mut bytes = vec![];
    s.snapshot().write_dense3d(&mut bytes).unwrap();
    bytes
}
fn receipt_unchanged(a: &ConfiguredProviderReport, b: &ConfiguredProviderReport) {
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
fn seed(
    s: &mut SimulationSession,
    p: &mut AcceptedCpuProvider,
    w: &mut ProviderStepWorkspace,
    v: &mut ViscosityWorkspace,
) {
    let (_, report) = s
        .step_with_configured_pressure(inputs(), w, Some((v, MU)), Some(p), None, |_| false)
        .unwrap();
    assert!(!report.prediction_called);
    assert!(matches!(
        report.pressure.rejection,
        Some(ProviderRejection::Unsupported(_))
    ));
}

#[test]
fn production_bridge_full_and_correction_qualify_two_intervals_with_exact_native_fields() {
    for correction in [false, true] {
        let fixture = package(correction, "solve");
        let (mut provider, slot) = start(&fixture);
        assert_eq!(provider.scratch_bytes(), 2 * 2 * size_of::<f64>());
        let (mut s, (mut w, mut v)) = (session(4), workspaces());
        let mut native = LiquidTransportSimulation::new(
            grid(),
            config(),
            PressureImplementation::JacobiPcgV1,
            vec![1.0; 2],
        )
        .unwrap();
        let mut nv = ViscosityWorkspace::new(grid(), CAP).unwrap();
        for generation in 1..=3 {
            let prior = s.snapshot().state().pressure.to_vec();
            let (_, report) = s
                .step_with_configured_pressure(
                    inputs(),
                    &mut w,
                    Some((&mut v, MU)),
                    Some(&mut provider),
                    None,
                    |_| false,
                )
                .unwrap();
            native
                .step_viscous(inputs(), &mut nv, MU, |_| false)
                .unwrap();
            assert_eq!(state_bits(s.snapshot().state()), state_bits(native.state()));
            assert!(Arc::ptr_eq(
                report.binding.as_ref().unwrap(),
                &provider.identity()
            ));
            assert_ne!(
                report.binding.as_ref().unwrap().source_revision,
                s.snapshot().identity().source_revision
            );
            if generation == 1 {
                assert!(!report.prediction_called);
                assert!(received(&fixture).is_empty());
            } else {
                assert!(report.prediction_called);
                assert!(report.pressure.rejection.is_none());
                assert_eq!(report.pressure.selected, "configured-pressure-context-v1");
                let packets = received(&fixture);
                let packet = packets.last().unwrap();
                assert_eq!(packet["context"]["generation"], generation);
                assert_eq!(packet["context"]["prior"]["status"]["kind"], "accepted");
                let name = if correction { "p0" } else { "solved_previous" };
                assert_eq!(packet["inputs"][name]["values"], json!(prior));
                let rhs = if correction { "b" } else { "equation_rhs" };
                assert_ne!(packet["inputs"][rhs]["values"], json!([0.0, 0.0]));
                // The child independently checks b-A*prior, actual dt/rho,
                // port units/layout and contiguous accepted endpoint/history.
            }
            assert!(provider.last_failure().is_none());
        }
        assert_eq!(received(&fixture).len(), 2);
        drain(provider, slot);
    }
}

#[test]
fn support_decline_is_inert_and_preserves_context_and_native_fallback() {
    let fixture = package(false, "solve");
    edit(&fixture, |m| m["support"]["dt_s"] = json!(2.0 * DT));
    let (mut provider, slot) = start(&fixture);
    let (mut s, (mut w, mut v)) = (session(2), workspaces());
    for _ in 0..2 {
        let (_, report) = s
            .step_with_configured_pressure(
                inputs(),
                &mut w,
                Some((&mut v, MU)),
                Some(&mut provider),
                None,
                |_| false,
            )
            .unwrap();
        assert!(!report.prediction_called);
        assert!(report.context.is_some());
        assert!(matches!(
            report.pressure.rejection,
            Some(ProviderRejection::Unsupported(_))
        ));
        assert_eq!(
            report.pressure.selected,
            PressureImplementation::JacobiPcgV1.id()
        );
    }
    assert!(received(&fixture).is_empty());
    assert!(provider.last_failure().is_none());
    drain(provider, slot);
}

#[test]
fn invalid_pin_metadata_and_scratch_cap_never_start_a_child() {
    for case in 0..8 {
        let fixture = package(false, "solve");
        let mut pin = (*identity(&fixture)).clone();
        let mut cap = CAP;
        match case {
            0 => pin.id.push_str("-drift"),
            1 => pin.version.push_str("-drift"),
            2 => pin.manifest_sha256[0] ^= 1,
            3 => pin.checkpoint_sha256 = Some([3; 32]),
            4 => pin.training_run_id = Some("unverified-training".into()),
            5 => pin.runtime.push('\n'),
            6 => pin.source_revision = "x".repeat(1001),
            _ => cap = 2 * 2 * size_of::<f64>() - 1,
        }
        let slot = Arc::new(Mutex::new(None));
        let mut phases = 0;
        assert!(
            AcceptedCpuProvider::start(
                &fixture.manifest,
                Arc::new(pin),
                Arc::clone(&slot),
                Arc::new(AtomicBool::new(false)),
                cap,
                |_| {
                    phases += 1;
                    Ok(())
                }
            )
            .is_err(),
            "case {case}"
        );
        assert_eq!(phases, 0);
        assert!(slot.lock().unwrap().is_none());
        assert!(!fixture.root.join("child-started").exists());
    }
}

#[test]
fn rejected_candidate_preserves_same_request_native_pressure_as_next_prior() {
    let fixture = package(true, "bad_once");
    let (mut provider, slot) = start(&fixture);
    let (mut s, (mut w, mut v)) = (session(3), workspaces());
    let mut native = LiquidTransportSimulation::new(
        grid(),
        config(),
        PressureImplementation::JacobiPcgV1,
        vec![1.0; 2],
    )
    .unwrap();
    let mut nv = ViscosityWorkspace::new(grid(), CAP).unwrap();
    seed(&mut s, &mut provider, &mut w, &mut v);
    native
        .step_viscous(inputs(), &mut nv, MU, |_| false)
        .unwrap();
    let (_, rejected) = s
        .step_with_configured_pressure(
            inputs(),
            &mut w,
            Some((&mut v, MU)),
            Some(&mut provider),
            None,
            |_| false,
        )
        .unwrap();
    native
        .step_viscous(inputs(), &mut nv, MU, |_| false)
        .unwrap();
    assert_eq!(state_bits(s.snapshot().state()), state_bits(native.state()));
    assert_eq!(
        rejected.pressure.rejection,
        Some(ProviderRejection::QualityGate)
    );
    assert!(rejected.pressure.candidate.is_some());
    assert!(
        provider.last_failure().is_none(),
        "host qualification is not a process failure"
    );
    let native_prior = s.snapshot().state().pressure.to_vec();
    let (_, next) = s
        .step_with_configured_pressure(
            inputs(),
            &mut w,
            Some((&mut v, MU)),
            Some(&mut provider),
            None,
            |_| false,
        )
        .unwrap();
    assert!(next.pressure.rejection.is_none());
    let packets = received(&fixture);
    assert_eq!(packets[1]["inputs"]["p0"]["values"], json!(native_prior));
    assert_eq!(next.context.unwrap().native_prefix_steps, 2);
    drain(provider, slot);
}

#[test]
fn runtime_failure_retains_dispatch_fact_and_fresh_owner_retries_accepted_native_history() {
    let fixture = package(false, "exit_once");
    let (mut provider, slot) = start(&fixture);
    let (mut s, (mut w, mut v)) = (session(3), workspaces());
    seed(&mut s, &mut provider, &mut w, &mut v);
    let (_, report) = s
        .step_with_configured_pressure(
            inputs(),
            &mut w,
            Some((&mut v, MU)),
            Some(&mut provider),
            None,
            |_| false,
        )
        .unwrap();
    assert!(matches!(
        report.pressure.rejection,
        Some(ProviderRejection::Execution(_))
    ));
    let failure = provider.last_failure().unwrap();
    assert_eq!(failure.reason, NoCandidateReason::RuntimeFailure);
    assert!(failure.dispatched);
    assert!(failure.invalid_payload_sha256.is_none());
    assert!(failure.detail.len() <= 8000 && !failure.detail.chars().any(char::is_control));
    let prior = s.snapshot().state().pressure.to_vec();
    drain(provider, slot);
    let (mut fresh, slot) = start(&fixture);
    let (_, report) = s
        .step_with_configured_pressure(
            inputs(),
            &mut w,
            Some((&mut v, MU)),
            Some(&mut fresh),
            None,
            |_| false,
        )
        .unwrap();
    assert!(report.pressure.rejection.is_none());
    assert!(fresh.last_failure().is_none());
    assert_eq!(
        received(&fixture)[1]["inputs"]["solved_previous"]["values"],
        json!(prior)
    );
    drain(fresh, slot);
}

#[test]
fn typed_invalid_payload_digest_and_runtime_context_decline_survive_coarse_fallback() {
    for behavior in ["malformed", "unsupported_context"] {
        let fixture = package(false, behavior);
        let (mut provider, slot) = start(&fixture);
        let (mut s, (mut w, mut v)) = (session(2), workspaces());
        seed(&mut s, &mut provider, &mut w, &mut v);
        let (_, report) = s
            .step_with_configured_pressure(
                inputs(),
                &mut w,
                Some((&mut v, MU)),
                Some(&mut provider),
                None,
                |_| false,
            )
            .unwrap();
        let failure = provider.last_failure().unwrap();
        assert!(failure.dispatched);
        assert_eq!(received(&fixture).len(), 1);
        if behavior == "malformed" {
            assert_eq!(failure.reason, NoCandidateReason::InvalidOutput);
            assert_eq!(
                failure.invalid_payload_sha256,
                Some(Sha256::digest(b"{\"kind\":\"candidate\",\"bogus\":1}\n").into())
            );
            assert!(matches!(
                report.pressure.rejection,
                Some(ProviderRejection::Execution(_))
            ));
        } else {
            assert_eq!(failure.reason, NoCandidateReason::UnsupportedContext);
            assert!(failure.invalid_payload_sha256.is_none());
            assert!(matches!(
                report.pressure.rejection,
                Some(ProviderRejection::Unsupported(_))
            ));
        }
        assert_eq!(
            report.pressure.selected,
            PressureImplementation::JacobiPcgV1.id()
        );
        drain(provider, slot);
    }
}

#[test]
fn real_inflight_cancellation_and_deadline_keep_publication_receipts_quota_and_join_child() {
    for deadline in [false, true] {
        let fixture = package(false, "stall_once");
        let (mut provider, slot) = start(&fixture);
        let (mut s, (mut w, mut v)) = (session(2), workspaces());
        seed(&mut s, &mut provider, &mut w, &mut v);
        let bytes = publication(&s);
        let receipt = s.snapshot().configured_provider_report().unwrap().clone();
        let prior = s.snapshot().state().pressure.to_vec();
        let marker = fixture.root.join("first-request-observed");
        let began = Instant::now();
        let result = s.step_with_configured_pressure(
            inputs(),
            &mut w,
            Some((&mut v, MU)),
            Some(&mut provider),
            deadline.then_some(began + Duration::from_millis(500)),
            |_| !deadline && marker.exists(),
        );
        if deadline {
            assert!(matches!(result, Err(SessionError::Deadline)));
        } else {
            assert!(result.is_err());
        }
        assert!(marker.exists(), "must cancel a consumed request");
        assert!(began.elapsed() < Duration::from_secs(2));
        assert_eq!(publication(&s), bytes);
        receipt_unchanged(&receipt, s.snapshot().configured_provider_report().unwrap());
        let failure = provider.last_failure().unwrap();
        assert_eq!(failure.reason, NoCandidateReason::Cancelled);
        assert!(failure.dispatched);
        drain(provider, slot);
        let (mut fresh, slot) = start(&fixture);
        let (_, report) = s
            .step_with_configured_pressure(
                inputs(),
                &mut w,
                Some((&mut v, MU)),
                Some(&mut fresh),
                None,
                |_| false,
            )
            .unwrap();
        assert!(report.pressure.rejection.is_none());
        assert_eq!(
            received(&fixture)[1]["inputs"]["solved_previous"]["values"],
            json!(prior)
        );
        assert_eq!(s.snapshot().frame(), 2);
        assert!(matches!(
            s.step_with_configured_pressure(
                inputs(),
                &mut w,
                Some((&mut v, MU)),
                Some(&mut fresh),
                None,
                |_| false
            ),
            Err(SessionError::StepLimit)
        ));
        drain(fresh, slot);
    }
}

#[test]
fn arbitrary_supported_shape_uses_bounded_zero_guess_and_rejects_diagnostic_mode() {
    let fixture = Fixture::new(false);
    let (mut provider, slot) = start(&fixture);
    let g = GridGeometry::new([3, 2, 1], [0.25; 3], [0.0; 3]).unwrap();
    let rhs = [0.0; 6];
    let request = |mode| ContextPressureRequest {
        context: PressureContext::new(PressureContextInput {
            grid: &g,
            density: 1000.0,
            dt: 1.0 / 60.0,
            interval_start: 0.0,
            rhs: &rhs,
            prior: None,
            run_id: "generic-supported-shape",
            generation: 1,
            mode,
        })
        .unwrap(),
        dynamic_viscosity: 0.001,
        native_prefix_steps: 0,
    };
    let mut out = [99.0; 6];
    assert!(
        provider
            .predict(
                &request(PressureContextMode::DiagnosticCandidate),
                &mut out,
                &mut || false
            )
            .is_err()
    );
    assert_eq!(out, [99.0; 6]);
    assert!(!provider.last_failure().unwrap().dispatched);
    let mut wrong = [99.0; 5];
    assert!(
        provider
            .predict(
                &request(PressureContextMode::AcceptedCandidate),
                &mut wrong,
                &mut || false
            )
            .is_err()
    );
    assert!(!provider.last_failure().unwrap().dispatched);
    provider
        .predict(
            &request(PressureContextMode::AcceptedCandidate),
            &mut out,
            &mut || false,
        )
        .unwrap();
    assert_eq!(out, [0.0, 0.25, 0.5, 0.0, 0.25, 0.5]);
    assert!(provider.last_failure().is_none());
    assert_eq!(
        provider.predict(
            &request(PressureContextMode::AcceptedCandidate),
            &mut out,
            &mut || true
        ),
        Err(ProviderRejection::Cancelled)
    );
    let failure = provider.last_failure().unwrap();
    assert_eq!(failure.reason, NoCandidateReason::Cancelled);
    assert!(!failure.dispatched);
    assert!(failure.invalid_payload_sha256.is_none());
    drain(provider, slot);
}
