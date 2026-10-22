//! Public accepted-session contracts. The candidate fixture executes the native
//! pressure solver; its declared package pins are fixture identity, not a model.
use rheon::{
    solver_provider::{
        ProviderRejection,
        configured::*,
        context::{PressureContextMode, PressureHistoryStatus},
    },
    *,
};
use sha2::{Digest, Sha256};
use std::{cell::RefCell, sync::Arc, time::Instant};

const CAP: usize = 4 * 1024 * 1024;
const RHO: f64 = 2.0;
const MU: f64 = 0.0078125;
const AXES: [Axis; 3] = [Axis::X, Axis::Y, Axis::Z];

fn grid() -> GridGeometry {
    GridGeometry::new([5; 3], [0.25; 3], [0.0; 3]).unwrap()
}
fn config() -> LiquidTransportConfig {
    LiquidTransportConfig {
        carrier: SimulationConfig {
            density: RHO,
            memory_limit: CAP,
            pressure: PressureSettings {
                relative_residual: 1e-14,
                absolute_residual: 1e-16,
                divergence_limit: 1e-7,
                max_iterations: 3000,
            },
            actual_divergence_limit: 1e-5,
            max_courant: 1.0 / 128.0,
        },
        represented_density: RHO,
        volume_stamp: VolumeStamp { id: 40, version: 0 },
        carrier_id: 41,
    }
}
fn session_with(cfg: LiquidTransportConfig, fraction: f64, quota: usize) -> SimulationSession {
    SimulationSession::new(
        SessionIdentity {
            run_id: 901,
            source_revision: "configured-session-native-fixture".into(),
        },
        grid(),
        cfg,
        PressureImplementation::JacobiPcgV1,
        vec![fraction; grid().cell_len()],
        CAP,
        quota,
    )
    .unwrap()
}
fn session(quota: usize) -> SimulationSession {
    session_with(config(), 1.0, quota)
}
fn workspace() -> ProviderStepWorkspace {
    ProviderStepWorkspace::new(&grid(), RHO, CAP, PressureImplementation::JacobiPcgV1).unwrap()
}
fn viscosity() -> ViscosityWorkspace {
    ViscosityWorkspace::new(grid(), CAP).unwrap()
}
// A binary, spatially uniform pressure-driving acceleration gives a legitimate
// saturated closed-box fixture without relying on approximate divergence-free
// storage of a localized 3D flow. The nonzero RHS and viscous normal strains
// remain exercised; every original fraction/pressure/velocity gate stays active.
fn forces() -> [BodyForce; 1] {
    [BodyForce {
        value: [0.125, 0.0625, 0.0625],
        units: ForceUnits::Acceleration,
        region: None,
    }]
}
fn inputs(forces: &[BodyForce]) -> LiquidStepInputs<'_> {
    LiquidStepInputs {
        requested_dt: 0.0625,
        smoke_source: None,
        forces,
        inlet: LiquidInlet::new(VolumeStamp { id: 42, version: 0 }, [[0.0; 2]; 3]).unwrap(),
        source: None,
        volume: LiquidVolumeSettings {
            max_outward_courant: 1.0,
            actual_divergence_limit: 1e-5,
        },
    }
}
fn digest(values: &[f64]) -> [u8; 32] {
    let mut hash = Sha256::new();
    for value in values {
        hash.update(value.to_bits().to_le_bytes());
    }
    hash.finalize().into()
}
fn state_bits(v: LiquidTransportView<'_>) -> Vec<u64> {
    let mut bits: Vec<_> = v
        .pressure
        .iter()
        .chain(v.liquid.fraction)
        .map(|x| x.to_bits())
        .collect();
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
fn bytes(s: &SimulationSession) -> Vec<u8> {
    let mut bytes = Vec::new();
    s.snapshot().write_dense3d(&mut bytes).unwrap();
    bytes
}

#[derive(Clone, Debug, PartialEq)]
struct Capture {
    generation: u64,
    start: f64,
    end: f64,
    dt: f64,
    mu: f64,
    native_prefix: usize,
    rhs: Vec<f64>,
    prior: Option<(u64, f64, Vec<f64>)>,
}
impl Capture {
    fn new(request: &ContextPressureRequest<'_>) -> Self {
        let c = &request.context;
        assert_eq!(c.mode(), PressureContextMode::AcceptedCandidate);
        assert_eq!(c.run_id(), "901");
        assert_eq!(c.grid(), &grid());
        assert_eq!(c.density(), RHO);
        assert_eq!(
            c.interval_end().to_bits(),
            (c.interval_start() + c.dt()).to_bits()
        );
        let prior = c.prior().map(|prior| {
            assert_eq!(prior.status, PressureHistoryStatus::Accepted);
            assert_eq!(prior.run_id, c.run_id());
            assert_eq!(prior.grid, c.grid());
            assert_eq!(prior.density, c.density());
            assert_eq!(prior.generation + 1, c.generation());
            assert_eq!(prior.interval_end.to_bits(), c.interval_start().to_bits());
            assert_eq!(prior.values[0], 0.0);
            (prior.generation, prior.interval_end, prior.values.to_vec())
        });
        Self {
            generation: c.generation(),
            start: c.interval_start(),
            end: c.interval_end(),
            dt: c.dt(),
            mu: request.dynamic_viscosity,
            native_prefix: request.native_prefix_steps,
            rhs: c.rhs().to_vec(),
            prior,
        }
    }
    fn check_stamp(&self, report: &ConfiguredProviderReport) {
        let stamp = report
            .context
            .expect("authoritative context survives support decline");
        assert_eq!(stamp.generation, self.generation);
        assert_eq!(stamp.interval_start.to_bits(), self.start.to_bits());
        assert_eq!(stamp.interval_end.to_bits(), self.end.to_bits());
        assert_eq!(stamp.dt.to_bits(), self.dt.to_bits());
        assert_eq!(stamp.native_prefix_steps, self.native_prefix);
        assert_eq!(stamp.rhs_sha256, digest(&self.rhs));
        assert_eq!(stamp.prior_generation, self.prior.as_ref().map(|p| p.0));
        assert_eq!(stamp.prior_interval_end, self.prior.as_ref().map(|p| p.1));
        assert_eq!(
            stamp.prior_pressure_sha256,
            self.prior.as_ref().map(|p| digest(&p.2))
        );
        assert_eq!(report.dynamic_viscosity.to_bits(), self.mu.to_bits());
    }
}
#[derive(Clone, Copy)]
enum Behavior {
    Solve,
    Decline,
    BadPressure,
    Cancel,
}
struct NativeCandidate {
    identity: Arc<ContextProviderIdentity>,
    supports: RefCell<Vec<Capture>>,
    predictions: Vec<Capture>,
    behavior: Behavior,
    pressure: PressureWorkspace,
}
impl NativeCandidate {
    fn new(behavior: Behavior) -> Self {
        Self {
            identity: Arc::new(ContextProviderIdentity {
                id: "authored-native-pressure-fixture".into(),
                version: "1".into(),
                manifest_sha256: [3; 32],
                checkpoint_sha256: Some([4; 32]),
                training_run_id: Some("declared-fixture-no-training".into()),
                runtime: "in-process-native-test-fixture".into(),
                source_revision: "authored-fixture".into(),
            }),
            supports: RefCell::new(Vec::new()),
            predictions: Vec::new(),
            behavior,
            pressure: PressureWorkspace::new(&grid(), CAP).unwrap(),
        }
    }
}
impl ContextPressureProvider for NativeCandidate {
    fn identity(&self) -> Arc<ContextProviderIdentity> {
        Arc::clone(&self.identity)
    }
    fn supports(&self, request: &ContextPressureRequest<'_>) -> Result<(), ProviderRejection> {
        self.supports.borrow_mut().push(Capture::new(request));
        if matches!(self.behavior, Behavior::Decline) {
            Err(ProviderRejection::Unsupported("authored support decline"))
        } else {
            Ok(())
        }
    }
    fn predict(
        &mut self,
        request: &ContextPressureRequest<'_>,
        out: &mut [f64],
        cancelled: &mut dyn FnMut() -> bool,
    ) -> Result<(), ProviderRejection> {
        self.predictions.push(Capture::new(request));
        if matches!(self.behavior, Behavior::Cancel) {
            return Err(ProviderRejection::Cancelled);
        }
        let c = &request.context;
        self.pressure
            .solve_rhs(
                &PressureOperator::new(c.grid(), c.density()).unwrap(),
                c.rhs(),
                c.dt(),
                config().carrier.pressure,
                cancelled,
            )
            .map_err(|_| ProviderRejection::Execution("native fixture solve failed"))?;
        out.copy_from_slice(self.pressure.pressure());
        if matches!(self.behavior, Behavior::BadPressure) {
            out[1] += 100.0;
        }
        Ok(())
    }
}

#[test]
fn configured_native_viscosity_is_bit_exact_with_existing_multistep_transport() {
    let mut accepted = session(8);
    let mut old = LiquidTransportSimulation::new(
        grid(),
        config(),
        PressureImplementation::JacobiPcgV1,
        vec![1.0; grid().cell_len()],
    )
    .unwrap();
    let (mut w, mut v, mut old_v) = (workspace(), viscosity(), viscosity());
    let f = forces();
    let owner_bytes = accepted.allocated_array_bytes();
    for dt in [0.05, 0.03125, 0.0625, 0.025] {
        let mut i = inputs(&f);
        i.requested_dt = dt;
        let reference = old.step_viscous(i, &mut old_v, MU, |_| false).unwrap();
        let (step, receipt) = accepted
            .step_with_configured_pressure(i, &mut w, Some((&mut v, MU)), None, None, |_| false)
            .unwrap();
        assert_eq!(
            state_bits(accepted.snapshot().state()),
            state_bits(old.state())
        );
        assert_eq!(
            step.carrier.step.dt.to_bits(),
            reference.carrier.step.dt.to_bits()
        );
        assert_eq!(step.viscosity.unwrap().dynamic_viscosity, MU);
        assert_eq!(
            step.total_array_bytes,
            step.owned_array_bytes + w.allocated_bytes() + v.allocated_bytes()
        );
        assert_eq!(accepted.allocated_array_bytes(), owner_bytes);
        assert!(receipt.binding.is_none());
        assert!(receipt.context.is_none());
        assert!(!receipt.prediction_called);
        assert_eq!(
            receipt.pressure.selected,
            PressureImplementation::JacobiPcgV1.id()
        );
        assert!(receipt.pressure.attempted.is_none());
    }
}

#[test]
fn candidate_observes_actual_dt_post_viscosity_rhs_and_owned_accepted_history() {
    let (mut s, mut w, mut v) = (session(8), workspace(), viscosity());
    let mut fixture = NativeCandidate::new(Behavior::Solve);
    let f = forces();
    let mut i = inputs(&f);
    i.requested_dt = 2.0;
    let (step, receipt) = s
        .step_with_configured_pressure(
            i,
            &mut w,
            Some((&mut v, MU)),
            Some(&mut fixture),
            None,
            |_| false,
        )
        .unwrap();
    let observed = &fixture.predictions[0];
    assert!(step.carrier.step.dt < i.requested_dt);
    assert_eq!(observed.dt.to_bits(), step.carrier.step.dt.to_bits());
    assert!(observed.prior.is_none());
    assert_eq!(observed.native_prefix, 0);
    assert_eq!(fixture.supports.borrow()[0], *observed);
    assert!(receipt.prediction_called);
    assert!(receipt.pressure.rejection.is_none());
    assert_eq!(receipt.pressure.selected, "configured-pressure-context-v1");
    assert!(Arc::ptr_eq(
        receipt.binding.as_ref().unwrap(),
        &fixture.identity
    ));
    observed.check_stamp(&receipt);
    // At rest advection is exactly zero. Independently sample the declared
    // acceleration on physical interior faces, then use public stage APIs.
    let g = grid();
    let mut forced: [Vec<f32>; 3] = std::array::from_fn(|d| vec![0.0; g.face_len(AXES[d])]);
    for (d, axis) in AXES.into_iter().enumerate() {
        let [nx, ny, nz] = g.face_counts(axis);
        for k in 0..nz {
            for j in 0..ny {
                for x in 0..nx {
                    let coordinate = [x, j, k];
                    if coordinate[d] > 0 && coordinate[d] < g.counts()[d] {
                        forced[d][g.face_index(axis, coordinate).unwrap()] =
                            (observed.dt * f[0].value[d]) as f32;
                    }
                }
            }
        }
    }
    let mut viscous = forced.clone();
    let [a, b, c] = &mut viscous;
    viscosity()
        .update(
            [&forced[0], &forced[1], &forced[2]],
            RHO,
            MU,
            observed.dt,
            [a, b, c],
            |_| false,
        )
        .unwrap();
    let op = PressureOperator::new(&g, RHO).unwrap();
    let mut expected = vec![0.0; g.cell_len()];
    let mut inviscid = expected.clone();
    op.build_rhs(
        [&viscous[0], &viscous[1], &viscous[2]],
        observed.dt,
        &mut expected,
    )
    .unwrap();
    op.build_rhs(
        [&forced[0], &forced[1], &forced[2]],
        observed.dt,
        &mut inviscid,
    )
    .unwrap();
    assert_ne!(digest(&expected), digest(&inviscid));
    assert_eq!(digest(&observed.rhs), digest(&expected));
    let prior = s.snapshot().state().pressure.to_vec();
    let end = step.carrier.step.time;
    let (_, next) = s
        .step_with_configured_pressure(
            inputs(&f),
            &mut w,
            Some((&mut v, MU)),
            Some(&mut fixture),
            None,
            |_| false,
        )
        .unwrap();
    let next_input = fixture.predictions.last().unwrap();
    assert_eq!(next_input.prior, Some((1, end, prior)));
    assert_eq!(next_input.native_prefix, 0);
    next_input.check_stamp(&next);
    assert_eq!(
        s.snapshot().configured_provider_report().unwrap().context,
        next.context
    );
}

#[test]
fn support_decline_and_bad_candidate_fallback_keep_native_prior_and_prefix() {
    for behavior in [Behavior::Decline, Behavior::BadPressure] {
        let (mut s, mut w, mut v) = (session(8), workspace(), viscosity());
        let mut control = LiquidTransportSimulation::new(
            grid(),
            config(),
            PressureImplementation::JacobiPcgV1,
            vec![1.0; grid().cell_len()],
        )
        .unwrap();
        let mut control_v = viscosity();
        let f = forces();
        s.step_with_configured_pressure(inputs(&f), &mut w, Some((&mut v, MU)), None, None, |_| {
            false
        })
        .unwrap();
        control
            .step_viscous(inputs(&f), &mut control_v, MU, |_| false)
            .unwrap();
        let mut fixture = NativeCandidate::new(behavior);
        let (_, rejected) = s
            .step_with_configured_pressure(
                inputs(&f),
                &mut w,
                Some((&mut v, MU)),
                Some(&mut fixture),
                None,
                |_| false,
            )
            .unwrap();
        control
            .step_viscous(inputs(&f), &mut control_v, MU, |_| false)
            .unwrap();
        assert_eq!(
            state_bits(s.snapshot().state()),
            state_bits(control.state())
        );
        assert_eq!(
            rejected.pressure.selected,
            PressureImplementation::JacobiPcgV1.id()
        );
        assert_eq!(
            rejected.prediction_called,
            matches!(behavior, Behavior::BadPressure)
        );
        assert_eq!(
            fixture.predictions.len(),
            usize::from(rejected.prediction_called)
        );
        fixture.supports.borrow()[0].check_stamp(&rejected);
        assert_eq!(fixture.supports.borrow()[0].native_prefix, 1);
        assert_eq!(
            rejected.pressure.rejection,
            Some(if matches!(behavior, Behavior::Decline) {
                ProviderRejection::Unsupported("authored support decline")
            } else {
                ProviderRejection::QualityGate
            })
        );
        let prior = s.snapshot().state().pressure.to_vec();
        assert_eq!(prior, control.state().pressure);
        fixture.behavior = Behavior::Solve;
        let (_, accepted) = s
            .step_with_configured_pressure(
                inputs(&f),
                &mut w,
                Some((&mut v, MU)),
                Some(&mut fixture),
                None,
                |_| false,
            )
            .unwrap();
        let c = fixture.predictions.last().unwrap();
        assert_eq!(c.prior.as_ref().unwrap().2, prior);
        assert_eq!(c.prior.as_ref().unwrap().0, 2);
        assert_eq!(c.native_prefix, 2);
        c.check_stamp(&accepted);
        assert!(accepted.pressure.rejection.is_none());
    }
}

#[test]
fn changed_viscosity_and_boundary_gaps_reseed_native_history_before_provider_recovery() {
    for boundary_gap in [false, true] {
        let (mut s, mut w, mut v) = (session(8), workspace(), viscosity());
        let f = forces();
        s.step_with_configured_pressure(inputs(&f), &mut w, Some((&mut v, MU)), None, None, |_| {
            false
        })
        .unwrap();
        if boundary_gap {
            let boundary = BoxFluxStepBoundary {
                flux: PrescribedBoxFlux::new(BoxFluxStamp { id: 50, version: 0 }, [[0.0; 2]; 3])
                    .unwrap(),
                tracer: BoxFluxTracerPolicy::ClampedAppearance,
            };
            s.step(inputs(&f), boundary, None, |_| false).unwrap();
            assert!(s.snapshot().configured_provider_report().is_none());
        }
        let mu = if boundary_gap { MU } else { 2.0 * MU };
        let mut fixture = NativeCandidate::new(Behavior::Solve);
        let (_, fallback) = s
            .step_with_configured_pressure(
                inputs(&f),
                &mut w,
                Some((&mut v, mu)),
                Some(&mut fixture),
                None,
                |_| false,
            )
            .unwrap();
        assert!(fallback.context.is_none());
        assert!(!fallback.prediction_called);
        assert!(fixture.supports.borrow().is_empty());
        assert!(fixture.predictions.is_empty());
        assert!(matches!(
            fallback.pressure.rejection,
            Some(ProviderRejection::Unsupported(_))
        ));
        let prior = s.snapshot().state().pressure.to_vec();
        let generation = s.snapshot().state().carrier.generation;
        let (_, recovered) = s
            .step_with_configured_pressure(
                inputs(&f),
                &mut w,
                Some((&mut v, mu)),
                Some(&mut fixture),
                None,
                |_| false,
            )
            .unwrap();
        let observed = fixture.predictions.last().unwrap();
        assert_eq!(observed.mu, mu);
        assert_eq!(observed.native_prefix, 1);
        assert_eq!(observed.prior.as_ref().unwrap().0, generation);
        assert_eq!(observed.prior.as_ref().unwrap().2, prior);
        observed.check_stamp(&recovered);
        assert!(recovered.pressure.rejection.is_none());
    }
}

fn assert_receipt_unchanged(before: &ConfiguredProviderReport, after: &ConfiguredProviderReport) {
    assert_eq!(before.binding, after.binding);
    assert_eq!(before.context, after.context);
    assert_eq!(before.prediction_called, after.prediction_called);
    assert_eq!(
        before.dynamic_viscosity.to_bits(),
        after.dynamic_viscosity.to_bits()
    );
    let diagnostics = |p: PressureReport| {
        (
            p.iterations,
            [
                p.true_residual_l2.to_bits(),
                p.true_residual_max.to_bits(),
                p.predicted_divergence_max.to_bits(),
            ],
        )
    };
    let (a, b) = (&before.pressure, &after.pressure);
    assert_eq!(a.selected, b.selected);
    assert_eq!(a.attempted, b.attempted);
    assert_eq!(a.rejection, b.rejection);
    assert_eq!(a.candidate.map(diagnostics), b.candidate.map(diagnostics));
    assert_eq!(diagnostics(a.accepted), diagnostics(b.accepted));
    assert_eq!(
        (a.total, a.inference, a.qualification, a.fallback),
        (b.total, b.inference, b.qualification, b.fallback)
    );
}

#[test]
fn every_checkpoint_cancellation_preserves_fields_receipt_history_and_retry_quota() {
    let (mut s, mut w, mut v) = (session(128), workspace(), viscosity());
    let f = forces();
    let mut fixture = NativeCandidate::new(Behavior::Solve);
    let mut stages = Vec::new();
    s.step_with_configured_pressure(
        inputs(&f),
        &mut w,
        Some((&mut v, MU)),
        Some(&mut fixture),
        None,
        |stage| {
            if !stages.contains(&stage) {
                stages.push(stage);
            }
            false
        },
    )
    .unwrap();
    assert!(stages.contains(&LiquidStepStage::BeforeCommit));
    assert!(
        stages
            .iter()
            .any(|s| matches!(s, LiquidStepStage::Carrier(StepStage::Viscosity(_))))
    );
    for stage in stages {
        let before = bytes(&s);
        let frame = s.snapshot().frame();
        let previous = s.snapshot().configured_provider_report().unwrap().clone();
        assert!(
            s.step_with_configured_pressure(
                inputs(&f),
                &mut w,
                Some((&mut v, MU)),
                Some(&mut fixture),
                None,
                |current| current == stage
            )
            .is_err(),
            "{stage:?}"
        );
        assert_eq!(bytes(&s), before, "{stage:?}");
        assert_eq!(s.snapshot().frame(), frame);
        assert_receipt_unchanged(
            &previous,
            s.snapshot().configured_provider_report().unwrap(),
        );
        let prior = s.snapshot().state().pressure.to_vec();
        s.step_with_configured_pressure(
            inputs(&f),
            &mut w,
            Some((&mut v, MU)),
            Some(&mut fixture),
            None,
            |_| false,
        )
        .unwrap();
        let observed = fixture.predictions.last().unwrap();
        assert_eq!(observed.generation, frame as u64 + 1);
        assert_eq!(observed.prior.as_ref().unwrap().2, prior);
    }
}

#[test]
fn provider_cancellation_deadline_and_post_pressure_volume_failure_do_not_publish() {
    let (mut s, mut w) = (session(2), workspace());
    let f = forces();
    let mut fixture = NativeCandidate::new(Behavior::Solve);
    s.step_with_configured_pressure(inputs(&f), &mut w, None, Some(&mut fixture), None, |_| {
        false
    })
    .unwrap();
    let before = bytes(&s);
    let previous = s.snapshot().configured_provider_report().unwrap().clone();
    fixture.behavior = Behavior::Cancel;
    assert!(
        s.step_with_configured_pressure(inputs(&f), &mut w, None, Some(&mut fixture), None, |_| {
            false
        })
        .is_err()
    );
    assert_eq!(bytes(&s), before);
    assert_receipt_unchanged(
        &previous,
        s.snapshot().configured_provider_report().unwrap(),
    );
    let calls = fixture.predictions.len();
    assert!(matches!(
        s.step_with_configured_pressure(
            inputs(&f),
            &mut w,
            None,
            Some(&mut fixture),
            Some(Instant::now()),
            |_| false
        ),
        Err(SessionError::Deadline)
    ));
    assert_eq!(fixture.predictions.len(), calls);
    assert_eq!(bytes(&s), before);
    fixture.behavior = Behavior::Solve;
    let rates = vec![100.0; grid().cell_len()];
    let mut bad = inputs(&f);
    bad.source = Some(LiquidVolumeSource::new(VolumeStamp { id: 60, version: 0 }, &rates).unwrap());
    assert!(matches!(
        s.step_with_configured_pressure(bad, &mut w, None, Some(&mut fixture), None, |_| false),
        Err(SessionError::Step(LiquidStepError::Volume(_)))
    ));
    assert!(fixture.predictions.len() > calls);
    assert_eq!(bytes(&s), before);
    assert_receipt_unchanged(
        &previous,
        s.snapshot().configured_provider_report().unwrap(),
    );
    s.step_with_configured_pressure(inputs(&f), &mut w, None, Some(&mut fixture), None, |_| {
        false
    })
    .unwrap();
    assert_eq!(s.snapshot().frame(), 2);
    let final_bytes = bytes(&s);
    assert!(matches!(
        s.step_with_configured_pressure(inputs(&f), &mut w, None, Some(&mut fixture), None, |_| {
            false
        }),
        Err(SessionError::StepLimit)
    ));
    assert_eq!(bytes(&s), final_bytes);
    s.stop();
    assert!(matches!(
        s.step_with_configured_pressure(inputs(&f), &mut w, None, None, None, |_| false),
        Err(SessionError::Stopped)
    ));
}

#[test]
fn workspace_caps_domains_and_invalid_coefficients_fail_before_publication() {
    let p = workspace().allocated_bytes();
    assert!(
        ProviderStepWorkspace::new(&grid(), RHO, p - 1, PressureImplementation::JacobiPcgV1)
            .is_err()
    );
    assert_eq!(
        ProviderStepWorkspace::new(&grid(), RHO, p, PressureImplementation::JacobiPcgV1)
            .unwrap()
            .allocated_bytes(),
        p
    );
    let v = viscosity().allocated_bytes();
    assert!(ViscosityWorkspace::new(grid(), v - 1).is_err());
    assert_eq!(
        ViscosityWorkspace::new(grid(), v)
            .unwrap()
            .allocated_bytes(),
        v
    );
    let f = forces();
    for (fraction, density, mu) in [
        (0.5, RHO, MU),
        (1.0, 3.0, MU),
        (1.0, RHO, -1.0),
        (1.0, RHO, f64::NAN),
        (1.0, RHO, 1e6),
    ] {
        let mut cfg = config();
        cfg.represented_density = density;
        let (mut s, mut w, mut v) = (session_with(cfg, fraction, 4), workspace(), viscosity());
        let mut fixture = NativeCandidate::new(Behavior::Solve);
        let before = bytes(&s);
        assert!(
            s.step_with_configured_pressure(
                inputs(&f),
                &mut w,
                Some((&mut v, mu)),
                Some(&mut fixture),
                None,
                |_| false
            )
            .is_err()
        );
        assert!(fixture.supports.borrow().is_empty());
        assert!(fixture.predictions.is_empty());
        assert_eq!(bytes(&s), before);
        assert!(s.snapshot().configured_provider_report().is_none());
    }
    let (mut s, mut v) = (session(4), viscosity());
    let before = bytes(&s);
    for (density, method) in [
        (1.0, PressureImplementation::JacobiPcgV1),
        (RHO, PressureImplementation::SymmetricGaussSeidelPcgV1),
    ] {
        let mut wrong = ProviderStepWorkspace::new(&grid(), density, CAP, method).unwrap();
        assert!(
            s.step_with_configured_pressure(
                inputs(&f),
                &mut wrong,
                Some((&mut v, MU)),
                None,
                None,
                |_| panic!("workspace mismatch reached callback")
            )
            .is_err()
        );
        assert_eq!(bytes(&s), before);
    }
    let mut w = workspace();
    let mut wrong_viscosity =
        ViscosityWorkspace::new(GridGeometry::new([4; 3], [0.25; 3], [0.0; 3]).unwrap(), CAP)
            .unwrap();
    assert!(
        s.step_with_configured_pressure(
            inputs(&f),
            &mut w,
            Some((&mut wrong_viscosity, MU)),
            None,
            None,
            |_| panic!("viscosity mismatch reached callback")
        )
        .is_err()
    );
    assert_eq!(bytes(&s), before);
}

#[test]
fn invalid_identity_never_reaches_provider_or_published_binding() {
    for id in ["x".repeat(1001), "control\nidentity".into()] {
        let (mut s, mut w, mut v) = (session(2), workspace(), viscosity());
        let mut fixture = NativeCandidate::new(Behavior::Solve);
        Arc::get_mut(&mut fixture.identity).unwrap().id = id;
        let f = forces();
        let (_, receipt) = s
            .step_with_configured_pressure(
                inputs(&f),
                &mut w,
                Some((&mut v, MU)),
                Some(&mut fixture),
                None,
                |_| false,
            )
            .unwrap();
        assert!(fixture.supports.borrow().is_empty());
        assert!(fixture.predictions.is_empty());
        assert!(receipt.binding.is_none());
        assert!(receipt.context.is_none());
        assert!(!receipt.prediction_called);
        assert!(matches!(
            receipt.pressure.rejection,
            Some(ProviderRejection::Unsupported(_))
        ));
        assert_eq!(
            receipt.pressure.selected,
            PressureImplementation::JacobiPcgV1.id()
        );
        assert_eq!(s.snapshot().stage(), SnapshotStage::Accepted);
        assert!(
            s.snapshot()
                .configured_provider_report()
                .unwrap()
                .binding
                .is_none()
        );
    }
}

#[cfg(feature = "accepted-pressure-warm-start")]
#[test]
fn initialized_publication_clears_configured_receipt_only_after_acceptance() {
    let mut owner = session(8);
    let mut pressure = workspace();
    owner
        .step_with_configured_pressure(inputs(&[]), &mut pressure, None, None, None, |_| false)
        .unwrap();
    assert!(owner.snapshot().configured_provider_report().is_some());
    let before = bytes(&owner);
    let start = owner.snapshot().accepted_interval_start();
    let warm = PressureInitializationPolicy::AcceptedPreviousV1;
    assert!(
        owner
            .step_with_initialization(inputs(&[]), warm, None, |stage| stage
                == LiquidStepStage::BeforeCommit)
            .is_err()
    );
    assert_eq!(bytes(&owner), before);
    assert_eq!(owner.snapshot().accepted_interval_start(), start);
    assert!(owner.snapshot().configured_provider_report().is_some());
    let actual_start = owner.snapshot().state().carrier.time;
    let report = owner
        .step_with_initialization(inputs(&[]), warm, None, |_| false)
        .unwrap();
    assert_eq!(
        report.pressure_initialization.actual,
        PressureInitializationPolicy::Cold
    );
    assert!(owner.snapshot().configured_provider_report().is_none());
    assert!(owner.snapshot().provider_report().is_none());
    assert_eq!(
        owner.snapshot().accepted_interval_start(),
        Some(actual_start)
    );
    let report = owner
        .step_with_initialization(inputs(&[]), warm, None, |_| false)
        .unwrap();
    assert_eq!(report.pressure_initialization.actual, warm);
    // A later configured publication is cold and replaces native seed eligibility.
    owner
        .step_with_configured_pressure(inputs(&[]), &mut pressure, None, None, None, |_| false)
        .unwrap();
    let report = owner
        .step_with_initialization(inputs(&[]), warm, None, |_| false)
        .unwrap();
    assert_eq!(
        report.pressure_initialization.actual,
        PressureInitializationPolicy::Cold
    );
    assert_eq!(
        report.pressure_initialization.cold_reason,
        Some(PressureColdReason::Invalidated)
    );
}
