//! Test-only Session/CPU-driver composition using the existing process owner.
//! The independent Python two-cell solver is not a trained model or a production
//! GUI/runtime adapter. No diagnostic history is promoted to accepted state.
use super::super::{
    adapter::{AdapterError, AdapterPackage},
    engine::Input,
    model_process::{ChildSlot, ModelProcess, PredictionFailure, ProviderIdentity},
};
use super::Fixture;
#[cfg(target_os = "linux")]
use super::assert_child_liveness;
use rheon::{
    recording::diagnostic::NoCandidateReason,
    solver_provider::{
        ProviderRejection,
        configured::{
            ConfiguredProviderReport, ContextPressureProvider, ContextPressureRequest,
            ContextProviderIdentity,
        },
        context::{
            PressureContext, PressureContextInput, PressureContextMode, PressureHistoryStatus,
        },
    },
    *,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    cell::RefCell,
    sync::{Arc, Mutex, atomic::AtomicBool},
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
            source_revision: "accepted-cpu-contract-fixture".into(),
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
fn hash(values: &[f64]) -> [u8; 32] {
    let mut h = Sha256::new();
    for value in values {
        h.update(value.to_bits().to_le_bytes());
    }
    h.finalize().into()
}
fn hex_digest(value: &str) -> [u8; 32] {
    assert_eq!(value.len(), 64);
    std::array::from_fn(|i| u8::from_str_radix(&value[2 * i..2 * i + 2], 16).unwrap())
}
fn publication(s: &SimulationSession) -> Vec<u8> {
    let mut bytes = Vec::new();
    s.snapshot().write_dense3d(&mut bytes).unwrap();
    bytes
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
fn package(correction: bool, mode: &str) -> Fixture {
    let source = include_str!("../../../../tests/fixtures/accepted_pressure_adapter.py")
        .replace("BEHAVIOR = \"solve\"", &format!("BEHAVIOR = {mode:?}"));
    let fixture = Fixture::source(&source, correction);
    fixture.edit(|m| {
        m["id"] = json!(if correction {
            "accepted-two-cell-correction-fixture"
        } else {
            "accepted-two-cell-full-fixture"
        });
        m["support"]["density_kg_m3"] = json!(RHO);
        m["support"]["dynamic_viscosity_pa_s"] = json!(MU);
        m["support"]["dt_s"] = json!(DT);
        m["support"]["counts_min"] = json!([2, 1, 1]);
        m["support"]["counts_max"] = json!([2, 1, 1]);
        m["support"]["max_cells"] = json!(2);
        m["support"]["spacing_min_m"] = json!(0.25);
        m["support"]["spacing_max_m"] = json!(0.25);
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
fn received(fixture: &Fixture) -> Vec<Value> {
    match std::fs::read_to_string(fixture.root.join("requests.jsonl")) {
        Ok(raw) => raw
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(e) => panic!("{e}"),
    }
}
#[derive(Clone, Debug, PartialEq)]
struct Observation {
    generation: u64,
    start: f64,
    end: f64,
    rhs: [f64; 2],
    prior: Option<[f64; 2]>,
    prefix: usize,
}
fn observation(r: &ContextPressureRequest<'_>) -> Observation {
    let c = &r.context;
    assert_eq!(c.mode(), PressureContextMode::AcceptedCandidate);
    assert_eq!(c.run_id(), "771");
    assert_eq!(c.grid(), &grid());
    assert_eq!(c.density(), RHO);
    assert_eq!(c.dt(), DT);
    assert_eq!(r.dynamic_viscosity, MU);
    let prior = c.prior().map(|p| {
        assert_eq!(p.status, PressureHistoryStatus::Accepted);
        assert_eq!(p.run_id, c.run_id());
        assert_eq!(p.generation + 1, c.generation());
        assert_eq!(p.interval_end.to_bits(), c.interval_start().to_bits());
        p.values.try_into().unwrap()
    });
    Observation {
        generation: c.generation(),
        start: c.interval_start(),
        end: c.interval_end(),
        rhs: c.rhs().try_into().unwrap(),
        prior,
        prefix: r.native_prefix_steps,
    }
}
struct Driver {
    package: AdapterPackage,
    process: ModelProcess,
    slot: ChildSlot,
    flag: AtomicBool,
    identity: Arc<ContextProviderIdentity>,
    supports: RefCell<Vec<Observation>>,
    predictions: Vec<Observation>,
    failures: Vec<PredictionFailure>,
}
impl Driver {
    fn new(fixture: &Fixture) -> Self {
        let package = AdapterPackage::load(&fixture.manifest).unwrap();
        let slot = Arc::new(Mutex::new(None));
        let flag = AtomicBool::new(false);
        let expected = ProviderIdentity {
            id: package.id().into(),
            version: package.version().into(),
            manifest_sha256: package.manifest_sha256().into(),
            checkpoint_sha256: None,
            training_run_id: None,
        };
        let mut process = ModelProcess::start_pinned(
            &fixture.manifest,
            Arc::clone(&slot),
            &flag,
            &expected,
            |_| Ok(()),
        )
        .unwrap();
        process.prediction_timeout = Duration::from_secs(5);
        let identity = Arc::new(ContextProviderIdentity {
            id: expected.id,
            version: expected.version,
            manifest_sha256: hex_digest(&expected.manifest_sha256),
            checkpoint_sha256: None,
            training_run_id: None,
            runtime: "test-only stdlib CPU subprocess".into(),
            source_revision: "authored-two-cell-contract-fixture".into(),
        });
        Self {
            package,
            process,
            slot,
            flag,
            identity,
            supports: RefCell::new(Vec::new()),
            predictions: Vec::new(),
            failures: Vec::new(),
        }
    }
    fn drain(self) {
        let slot = Arc::clone(&self.slot);
        let pid = slot.lock().unwrap().as_ref().unwrap().id();
        drop(self);
        assert!(slot.lock().unwrap().is_none());
        #[cfg(target_os = "linux")]
        assert_child_liveness(pid, false);
    }
}
impl ContextPressureProvider for Driver {
    fn identity(&self) -> Arc<ContextProviderIdentity> {
        Arc::clone(&self.identity)
    }
    fn supports(&self, r: &ContextPressureRequest<'_>) -> Result<(), ProviderRejection> {
        self.supports.borrow_mut().push(observation(r));
        match self.package.validate_support(
            r.context.grid(),
            r.context.density(),
            r.dynamic_viscosity,
            r.context.dt(),
            r.native_prefix_steps,
            r.context.prior().is_some(),
        ) {
            Ok(()) => Ok(()),
            Err(AdapterError::Unsupported(_)) => Err(ProviderRejection::Unsupported(
                "test package physical/context support declined",
            )),
            Err(AdapterError::Invalid(_) | AdapterError::Unavailable(_)) => Err(
                ProviderRejection::Execution("test package support unavailable"),
            ),
        }
    }
    fn predict(
        &mut self,
        r: &ContextPressureRequest<'_>,
        out: &mut [f64],
        poll: &mut dyn FnMut() -> bool,
    ) -> Result<(), ProviderRejection> {
        let observed = observation(r);
        assert_ne!(observed.rhs, [0.0; 2]);
        self.predictions.push(observed);
        let c = &r.context;
        let guess: [f64; 2] = c.prior().map_or([0.0; 2], |p| p.values.try_into().unwrap());
        let mut residual = [0.0; 2];
        c.initial_residual(&mut residual).unwrap();
        let context = PressureContext::new(PressureContextInput {
            grid: c.grid(),
            density: c.density(),
            dt: c.dt(),
            interval_start: c.interval_start(),
            rhs: c.rhs(),
            prior: c.prior(),
            run_id: c.run_id(),
            generation: c.generation(),
            mode: c.mode(),
        })
        .unwrap();
        match self.process.predict_typed_with_poll(
            Input {
                context,
                initial_guess: &guess,
                residual: &residual,
                viscosity: r.dynamic_viscosity,
            },
            &self.flag,
            poll,
        ) {
            Ok(values) => {
                assert_eq!(values.len(), 2);
                out.copy_from_slice(&values);
                Ok(())
            }
            Err(failure) => {
                let rejection = match failure.reason {
                    NoCandidateReason::Cancelled => ProviderRejection::Cancelled,
                    NoCandidateReason::Unsupported | NoCandidateReason::UnsupportedContext => {
                        ProviderRejection::Unsupported("typed CPU support/context failure")
                    }
                    NoCandidateReason::RuntimeFailure | NoCandidateReason::InvalidOutput => {
                        ProviderRejection::Execution("typed CPU runtime/output failure")
                    }
                };
                self.failures.push(failure);
                Err(rejection)
            }
        }
    }
}
fn receipt_unchanged(before: &ConfiguredProviderReport, after: &ConfiguredProviderReport) {
    assert_eq!(before.binding, after.binding);
    assert_eq!(before.context, after.context);
    assert_eq!(before.prediction_called, after.prediction_called);
    assert_eq!(
        before.dynamic_viscosity.to_bits(),
        after.dynamic_viscosity.to_bits()
    );
    let p = |r: PressureReport| {
        (
            r.iterations,
            r.true_residual_l2.to_bits(),
            r.true_residual_max.to_bits(),
            r.predicted_divergence_max.to_bits(),
        )
    };
    let (a, b) = (&before.pressure, &after.pressure);
    assert_eq!(a.selected, b.selected);
    assert_eq!(a.attempted, b.attempted);
    assert_eq!(a.rejection, b.rejection);
    assert_eq!(a.candidate.map(p), b.candidate.map(p));
    assert_eq!(p(a.accepted), p(b.accepted));
    assert_eq!(
        (a.total, a.inference, a.qualification, a.fallback),
        (b.total, b.inference, b.qualification, b.fallback)
    );
}
fn check_context(receipt: &ConfiguredProviderReport, observed: &Observation) {
    let stamp = receipt.context.unwrap();
    assert_eq!(stamp.generation, observed.generation);
    assert_eq!(stamp.interval_start.to_bits(), observed.start.to_bits());
    assert_eq!(stamp.interval_end.to_bits(), observed.end.to_bits());
    assert_eq!(stamp.rhs_sha256, hash(&observed.rhs));
    assert_eq!(
        stamp.prior_pressure_sha256,
        observed.prior.as_ref().map(|p| hash(p))
    );
    assert_eq!(stamp.native_prefix_steps, observed.prefix);
}

#[test]
fn actual_cpu_full_and_correction_packages_qualify_after_native_seed_with_exact_fields() {
    for correction in [false, true] {
        let fixture = package(correction, "solve");
        let mut driver = Driver::new(&fixture);
        let (mut s, (mut w, mut v)) = (session(4), workspaces());
        let mut control = LiquidTransportSimulation::new(
            grid(),
            config(),
            PressureImplementation::JacobiPcgV1,
            vec![1.0; 2],
        )
        .unwrap();
        let mut cv = ViscosityWorkspace::new(grid(), CAP).unwrap();
        for generation in 1..=3 {
            let prior = s.snapshot().state().pressure.to_vec();
            let (step, report) = s
                .step_with_configured_pressure(
                    inputs(),
                    &mut w,
                    Some((&mut v, MU)),
                    Some(&mut driver),
                    None,
                    |_| false,
                )
                .unwrap();
            control
                .step_viscous(inputs(), &mut cv, MU, |_| false)
                .unwrap();
            assert_eq!(
                state_bits(s.snapshot().state()),
                state_bits(control.state())
            );
            assert_eq!(step.carrier.step.dt, DT);
            assert_eq!(step.viscosity.unwrap().dynamic_viscosity, MU);
            assert_eq!(
                step.total_array_bytes,
                step.owned_array_bytes + w.allocated_bytes() + v.allocated_bytes()
            );
            assert!(Arc::ptr_eq(
                report.binding.as_ref().unwrap(),
                &driver.identity
            ));
            if generation == 1 {
                assert!(!report.prediction_called);
                assert!(report.pressure.rejection.is_some());
                assert!(driver.predictions.is_empty());
                assert!(received(&fixture).is_empty());
            } else {
                assert!(report.prediction_called);
                assert!(report.pressure.rejection.is_none());
                assert_eq!(report.pressure.selected, "configured-pressure-context-v1");
                let observed = driver.predictions.last().unwrap();
                assert_eq!(observed.prior, Some(prior.try_into().unwrap()));
                assert_eq!(observed.prefix, 1);
                check_context(&report, observed);
            }
        }
        let packets = received(&fixture);
        assert_eq!(packets.len(), 2);
        assert_eq!(packets[0]["context"]["generation"], 2);
        assert_eq!(packets[1]["context"]["generation"], 3);
        assert!(driver.failures.is_empty());
        driver.drain();
    }
}
#[test]
fn physical_support_declines_before_prediction_and_keeps_native_context() {
    let fixture = package(false, "solve");
    fixture.edit(|m| m["support"]["dt_s"] = json!(2.0 * DT));
    let mut driver = Driver::new(&fixture);
    let (mut s, (mut w, mut v)) = (session(3), workspaces());
    for generation in 1..=2 {
        let (_, r) = s
            .step_with_configured_pressure(
                inputs(),
                &mut w,
                Some((&mut v, MU)),
                Some(&mut driver),
                None,
                |_| false,
            )
            .unwrap();
        assert!(!r.prediction_called);
        assert_eq!(
            r.pressure.selected,
            PressureImplementation::JacobiPcgV1.id()
        );
        assert!(matches!(
            r.pressure.rejection,
            Some(ProviderRejection::Unsupported(_))
        ));
        let observed = driver.supports.borrow().last().unwrap().clone();
        assert_eq!(observed.generation, generation);
        check_context(&r, &observed);
    }
    assert!(received(&fixture).is_empty());
    assert!(driver.predictions.is_empty());
    assert!(driver.failures.is_empty());
    driver.drain();
}
#[test]
fn real_bad_candidate_falls_back_on_same_rhs_and_supplies_native_next_prior() {
    let fixture = package(true, "bad_once");
    let mut driver = Driver::new(&fixture);
    let (mut s, (mut w, mut v)) = (session(4), workspaces());
    let mut control = LiquidTransportSimulation::new(
        grid(),
        config(),
        PressureImplementation::JacobiPcgV1,
        vec![1.0; 2],
    )
    .unwrap();
    let mut cv = ViscosityWorkspace::new(grid(), CAP).unwrap();
    for generation in 1..=2 {
        let (_, r) = s
            .step_with_configured_pressure(
                inputs(),
                &mut w,
                Some((&mut v, MU)),
                Some(&mut driver),
                None,
                |_| false,
            )
            .unwrap();
        control
            .step_viscous(inputs(), &mut cv, MU, |_| false)
            .unwrap();
        assert_eq!(
            state_bits(s.snapshot().state()),
            state_bits(control.state())
        );
        if generation == 2 {
            assert!(r.prediction_called);
            assert_eq!(r.pressure.rejection, Some(ProviderRejection::QualityGate));
            assert!(r.pressure.candidate.is_some());
            assert_eq!(
                r.pressure.selected,
                PressureImplementation::JacobiPcgV1.id()
            );
            check_context(&r, driver.predictions.last().unwrap());
        }
    }
    let native: [f64; 2] = s.snapshot().state().pressure.try_into().unwrap();
    let (_, next) = s
        .step_with_configured_pressure(
            inputs(),
            &mut w,
            Some((&mut v, MU)),
            Some(&mut driver),
            None,
            |_| false,
        )
        .unwrap();
    assert_eq!(driver.predictions.last().unwrap().prior, Some(native));
    assert_eq!(driver.predictions.last().unwrap().prefix, 2);
    assert!(next.pressure.rejection.is_none());
    assert!(driver.failures.is_empty());
    assert_eq!(received(&fixture).len(), 2);
    driver.drain();
}
#[test]
fn actual_child_exit_preserves_typed_failure_and_accepted_native_fallback() {
    let fixture = package(false, "exit_once");
    let mut driver = Driver::new(&fixture);
    let (mut s, (mut w, mut v)) = (session(3), workspaces());
    s.step_with_configured_pressure(
        inputs(),
        &mut w,
        Some((&mut v, MU)),
        Some(&mut driver),
        None,
        |_| false,
    )
    .unwrap();
    let (_, report) = s
        .step_with_configured_pressure(
            inputs(),
            &mut w,
            Some((&mut v, MU)),
            Some(&mut driver),
            None,
            |_| false,
        )
        .unwrap();
    assert!(report.prediction_called);
    assert_eq!(
        report.pressure.selected,
        PressureImplementation::JacobiPcgV1.id()
    );
    assert!(matches!(
        report.pressure.rejection,
        Some(ProviderRejection::Execution(_))
    ));
    let failure = driver.failures.last().unwrap();
    assert_eq!(failure.reason, NoCandidateReason::RuntimeFailure);
    assert!(failure.dispatched);
    assert!(failure.invalid_payload_sha256.is_none());
    assert_eq!(received(&fixture).len(), 1);
    let prior: [f64; 2] = s.snapshot().state().pressure.try_into().unwrap();
    driver.drain();
    let mut restarted = Driver::new(&fixture);
    let (_, next) = s
        .step_with_configured_pressure(
            inputs(),
            &mut w,
            Some((&mut v, MU)),
            Some(&mut restarted),
            None,
            |_| false,
        )
        .unwrap();
    assert_eq!(restarted.predictions[0].prior, Some(prior));
    assert!(next.pressure.rejection.is_none());
    restarted.drain();
}
#[test]
fn real_inflight_session_cancellation_and_deadline_preserve_publication_and_retry_quota() {
    for deadline_case in [false, true] {
        let fixture = package(false, "stall_once");
        let mut driver = Driver::new(&fixture);
        let (mut s, (mut w, mut v)) = (session(2), workspaces());
        s.step_with_configured_pressure(
            inputs(),
            &mut w,
            Some((&mut v, MU)),
            Some(&mut driver),
            None,
            |_| false,
        )
        .unwrap();
        let before = publication(&s);
        let receipt = s.snapshot().configured_provider_report().unwrap().clone();
        let prior: [f64; 2] = s.snapshot().state().pressure.try_into().unwrap();
        let marker = fixture.root.join("first-request-observed");
        let start = Instant::now();
        let deadline = deadline_case.then_some(start + Duration::from_millis(500));
        let result = s.step_with_configured_pressure(
            inputs(),
            &mut w,
            Some((&mut v, MU)),
            Some(&mut driver),
            deadline,
            |_| !deadline_case && marker.exists(),
        );
        assert!(result.is_err());
        if deadline_case {
            assert!(matches!(result, Err(SessionError::Deadline)));
        }
        assert!(marker.exists(), "must cancel an actually consumed request");
        assert!(start.elapsed() < Duration::from_secs(2));
        assert_eq!(publication(&s), before);
        assert_eq!(s.snapshot().frame(), 1);
        receipt_unchanged(&receipt, s.snapshot().configured_provider_report().unwrap());
        let failure = driver.failures.last().unwrap();
        assert_eq!(failure.reason, NoCandidateReason::Cancelled);
        assert!(failure.dispatched);
        assert!(failure.invalid_payload_sha256.is_none());
        let attempted = driver.predictions.last().unwrap().clone();
        driver.drain();
        let mut restarted = Driver::new(&fixture);
        let (_, accepted) = s
            .step_with_configured_pressure(
                inputs(),
                &mut w,
                Some((&mut v, MU)),
                Some(&mut restarted),
                None,
                |_| false,
            )
            .unwrap();
        assert_eq!(restarted.predictions[0], attempted);
        assert_eq!(restarted.predictions[0].prior, Some(prior));
        assert!(accepted.pressure.rejection.is_none());
        assert_eq!(s.snapshot().frame(), 2);
        assert!(matches!(
            s.step_with_configured_pressure(
                inputs(),
                &mut w,
                Some((&mut v, MU)),
                Some(&mut restarted),
                None,
                |_| false
            ),
            Err(SessionError::StepLimit)
        ));
        assert_eq!(received(&fixture).len(), 2);
        restarted.drain();
    }
}

fn direct_input<'a>(g: &'a GridGeometry, zeros: &'a [f64]) -> Input<'a> {
    Input {
        context: PressureContext::new(PressureContextInput {
            grid: g,
            density: 1000.0,
            dt: 1.0 / 60.0,
            interval_start: 0.0,
            rhs: zeros,
            prior: None,
            run_id: "direct-runtime-cancellation-fixture",
            generation: 1,
            mode: PressureContextMode::AcceptedCandidate,
        })
        .unwrap(),
        initial_guess: zeros,
        residual: zeros,
        viscosity: 0.001,
    }
}
fn drop_process(process: ModelProcess, slot: &ChildSlot) {
    let pid = slot.lock().unwrap().as_ref().unwrap().id();
    drop(process);
    assert!(slot.lock().unwrap().is_none());
    #[cfg(target_os = "linux")]
    assert_child_liveness(pid, false);
}
#[test]
fn cooperative_poll_cancels_before_dispatch_without_consuming_request() {
    let source = include_str!("../../../../tests/fixtures/pressure_adapter_full.py").replace(
        "    request = json.loads(line)",
        "    Path(__file__).with_suffix('.consumed').write_text('observed')\n    request = json.loads(line)",
    );
    let fixture = Fixture::source(&source, false);
    let slot = Arc::new(Mutex::new(None));
    let flag = AtomicBool::new(false);
    let mut process =
        ModelProcess::start(&fixture.manifest, Arc::clone(&slot), &flag, |_| Ok(())).unwrap();
    let g = GridGeometry::new([2, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let zeros = [0.0; 2];
    let mut polls = 0;
    let failure = process
        .predict_typed_with_poll(direct_input(&g, &zeros), &flag, &mut || {
            polls += 1;
            true
        })
        .unwrap_err();
    assert_eq!(polls, 1);
    assert_eq!(failure.reason, NoCandidateReason::Cancelled);
    assert!(!failure.dispatched);
    assert!(failure.invalid_payload_sha256.is_none());
    drop_process(process, &slot);
    assert!(!fixture.root.join("adapter.consumed").exists());
}
#[test]
fn cooperative_poll_cancels_blocked_writer_after_initiation_and_joins_io() {
    let prefix = include_str!("../../../../tests/fixtures/pressure_adapter_full.py")
        .split("for line in sys.stdin:")
        .next()
        .unwrap();
    let fixture = Fixture::source(&format!("{prefix}\nimport time\ntime.sleep(60)\n"), false);
    fixture.edit(|m| {
        let mut ports = serde_json::Map::new();
        for i in 0..32 {
            ports.insert(format!("coordinate_{i}"), json!("x"));
        }
        m["inputs"] = Value::Object(ports);
    });
    let slot = Arc::new(Mutex::new(None));
    let flag = AtomicBool::new(false);
    let mut process =
        ModelProcess::start(&fixture.manifest, Arc::clone(&slot), &flag, |_| Ok(())).unwrap();
    let g = GridGeometry::new([32, 16, 16], [1.0; 3], [0.0; 3]).unwrap();
    let zeros = vec![0.0; g.cell_len()];
    let mut polls = 0;
    let start = Instant::now();
    let failure = process
        .predict_typed_with_poll(direct_input(&g, &zeros), &flag, &mut || {
            polls += 1;
            polls >= 6
        })
        .unwrap_err();
    assert_eq!(failure.reason, NoCandidateReason::Cancelled);
    assert!(failure.dispatched);
    assert!(failure.invalid_payload_sha256.is_none());
    assert!(polls >= 6);
    assert!(start.elapsed() < Duration::from_secs(2));
    drop_process(process, &slot);
}

#[path = "accepted_recording_contract_tests.rs"]
mod accepted_recording_contract_tests;
