use rheon::solver_provider::*;
use rheon::*;
const CAP: usize = 32 * 1024 * 1024;
fn config() -> SimulationConfig {
    SimulationConfig {
        density: 1.25,
        memory_limit: CAP,
        pressure: PressureSettings {
            relative_residual: 1e-8,
            absolute_residual: 1e-12,
            divergence_limit: 1e-7,
            max_iterations: 3000,
        },
        actual_divergence_limit: 1e-5,
        max_courant: 1.0,
    }
}
fn grid(n: u64) -> GridGeometry {
    GridGeometry::new([n; 3], [1.0 / n as f64; 3], [0.0; 3]).unwrap()
}
fn session(n: u64, cfg: SimulationConfig) -> SimulationSession {
    let g = grid(n);
    let fractions = vec![0.5; g.cell_len()];
    SimulationSession::new(
        SessionIdentity {
            run_id: 710,
            source_revision: "authored-production-step-test".into(),
        },
        g,
        LiquidTransportConfig {
            carrier: cfg,
            represented_density: 1.25,
            volume_stamp: VolumeStamp { id: 11, version: 0 },
            carrier_id: 12,
        },
        PressureImplementation::JacobiPcgV1,
        fractions,
        CAP,
        64,
    )
    .unwrap()
}
fn workspace(n: u64) -> ProviderStepWorkspace {
    ProviderStepWorkspace::new(&grid(n), 1.25, CAP, PressureImplementation::JacobiPcgV1).unwrap()
}
fn inputs(forces: &[BodyForce]) -> LiquidStepInputs<'_> {
    LiquidStepInputs {
        requested_dt: 1.0 / 60.0,
        smoke_source: Some(SmokeSource {
            lower: [0.25; 3],
            upper: [0.75; 3],
            tracer_rate: 0.2,
            vertical_acceleration: 0.0,
        }),
        forces,
        inlet: LiquidInlet::new(VolumeStamp { id: 14, version: 0 }, [[0.0; 2]; 3]).unwrap(),
        source: None,
        volume: LiquidVolumeSettings {
            max_outward_courant: 1.0,
            actual_divergence_limit: 1e-5,
        },
    }
}
fn forces() -> [BodyForce; 1] {
    [BodyForce {
        value: [0.001, 0.002, 0.0],
        units: ForceUnits::Acceleration,
        region: Some(ForceRegion {
            lower: [0.25; 3],
            upper: [0.75; 3],
        }),
    }]
}
fn bytes(s: &SimulationSession) -> Vec<u8> {
    let mut b = Vec::new();
    s.snapshot().write_dense3d(&mut b).unwrap();
    b
}
struct Probe {
    reject: bool,
}
impl PressureProvider for Probe {
    fn metadata(&self) -> SolverMetadata {
        SolverMetadata {
            id: "authored-zero-probe",
            version: "1",
            stage: "pressure",
            input: TensorMetadata {
                shape_xyz: [4; 3],
                layout: "x-fastest",
                scalar: "f64",
                units: "m^3/s^2",
            },
            output: TensorMetadata {
                shape_xyz: [4; 3],
                layout: "x-fastest",
                scalar: "f64",
                units: "Pa",
            },
            domain: "sealed-box",
            provenance: None,
        }
    }
    fn supports(&self, _: &PressureRequest<'_>) -> Result<(), ProviderRejection> {
        Ok(())
    }
    fn predict(
        &mut self,
        _: &PressureRequest<'_>,
        p: &mut [f64],
        _: &mut dyn FnMut() -> bool,
    ) -> Result<(), ProviderRejection> {
        if self.reject {
            Err(ProviderRejection::Execution("injected"))
        } else {
            p.fill(0.0);
            Ok(())
        }
    }
}
#[test]
fn production_numerical_selection_matches_original_full_step_fields() {
    let g = grid(4);
    let cfg = config();
    let mut old = Simulation::new(g.clone(), cfg).unwrap();
    let mut new = Simulation::new(g, cfg).unwrap();
    let mut w = workspace(4);
    for _ in 0..6 {
        let i = inputs(&[]);
        let f = forces();
        let a = old
            .step_with_forces(i.requested_dt, i.smoke_source, &f, |_| false)
            .unwrap();
        let b = new
            .step_with_pressure_provider(i.requested_dt, i.smoke_source, &f, &mut w, None, |_| {
                false
            })
            .unwrap();
        assert_eq!(a.step.generation, b.step.step.generation);
        assert_eq!(a.step.dt, b.step.step.dt);
        assert_eq!(old.state().x, new.state().x);
        assert_eq!(old.state().y, new.state().y);
        assert_eq!(old.state().z, new.state().z);
        assert_eq!(old.state().tracer, new.state().tracer);
        assert_eq!(
            b.provider.selected,
            PressureImplementation::JacobiPcgV1.id()
        );
    }
}
#[test]
fn every_observed_callback_preserves_complete_session_publication() {
    let mut stages = Vec::new();
    let mut s = session(4, config());
    let mut w = workspace(4);
    let f = forces();
    s.step_with_pressure_provider(inputs(&f), &mut w, None, None, |stage| {
        if !stages.contains(&stage) {
            stages.push(stage)
        }
        false
    })
    .unwrap();
    assert!(stages.contains(&LiquidStepStage::BeforeCommit));
    assert!(stages.contains(&LiquidStepStage::Carrier(
        StepStage::BeforeProjectionAcceptance
    )));
    for stage in stages {
        let before = bytes(&s);
        let frame = s.snapshot().frame();
        let previous = s.snapshot().provider_report().unwrap().total;
        assert!(
            s.step_with_pressure_provider(inputs(&f), &mut w, None, None, |v| v == stage)
                .is_err(),
            "{stage:?}"
        );
        assert_eq!(bytes(&s), before, "{stage:?}");
        assert_eq!(s.snapshot().frame(), frame);
        assert_eq!(s.snapshot().provider_report().unwrap().total, previous);
        s.step_with_pressure_provider(inputs(&f), &mut w, None, None, |_| false)
            .unwrap();
    }
}
#[test]
fn fallback_and_post_pressure_rejection_do_not_confuse_stage_identity() {
    let mut s = session(4, config());
    let mut w = workspace(4);
    let f = forces();
    let mut probe = Probe { reject: true };
    let (step, p) = s
        .step_with_pressure_provider(inputs(&f), &mut w, Some(&mut probe), None, |_| false)
        .unwrap();
    assert_eq!(p.rejection, Some(ProviderRejection::Execution("injected")));
    assert_eq!(s.snapshot().stage(), SnapshotStage::Accepted);
    assert_eq!(
        s.snapshot().state().carrier.generation,
        step.carrier.step.generation
    );
    assert_eq!(s.snapshot().provider_report().unwrap().selected, p.selected);
    // A deliberately permissive pressure-stage tolerance cannot bypass the
    // independently stored-f32 velocity gate of the production step.
    let mut cfg = config();
    cfg.pressure.absolute_residual = 1e6;
    cfg.pressure.divergence_limit = 1e6;
    let mut s = session(4, cfg);
    probe.reject = false;
    s.step_with_pressure_provider(inputs(&[]), &mut w, Some(&mut probe), None, |_| false)
        .unwrap();
    let before = bytes(&s);
    let previous = s.snapshot().provider_report().unwrap().total;
    assert!(matches!(
        s.step_with_pressure_provider(inputs(&f), &mut w, Some(&mut probe), None, |_| false),
        Err(SessionError::Step(LiquidStepError::Carrier(
            SimulationError::DivergenceLimit { .. }
        )))
    ));
    assert_eq!(bytes(&s), before);
    assert_eq!(s.snapshot().provider_report().unwrap().total, previous);
}
#[test]
fn budgets_workspace_identity_and_legacy_transition_are_explicit() {
    let mut s = session(4, config());
    let mut w = workspace(4);
    let size = w.allocated_bytes();
    assert!(
        ProviderStepWorkspace::new(
            &grid(4),
            1.25,
            size - 1,
            PressureImplementation::JacobiPcgV1
        )
        .is_err()
    );
    assert_eq!(
        ProviderStepWorkspace::new(&grid(4), 1.25, size, PressureImplementation::JacobiPcgV1)
            .unwrap()
            .allocated_bytes(),
        size
    );
    let mut wrong = workspace(2);
    let before = bytes(&s);
    assert!(
        s.step_with_pressure_provider(inputs(&[]), &mut wrong, None, None, |_| false)
            .is_err()
    );
    assert_eq!(bytes(&s), before);
    s.step_with_pressure_provider(inputs(&[]), &mut w, None, None, |_| false)
        .unwrap();
    let boundary = BoxFluxStepBoundary {
        flux: PrescribedBoxFlux::new(BoxFluxStamp { id: 31, version: 1 }, [[0.0; 2]; 3]).unwrap(),
        tracer: BoxFluxTracerPolicy::ClampedAppearance,
    };
    s.step(inputs(&[]), boundary, None, |_| false).unwrap();
    assert!(s.snapshot().provider_report().is_none());
    s.stop();
    assert!(matches!(
        s.step_with_pressure_provider(inputs(&[]), &mut w, None, None, |_| false),
        Err(SessionError::Stopped)
    ));
}
#[test]
#[ignore = "requires RHEON_AI_EXPORT from the SHA256-verified checkpoint exporter"]
fn real_model_advances_production_session_for_twenty_four_intervals() {
    let path = std::env::var("RHEON_AI_EXPORT").unwrap();
    let mut model = LabModalPressure::from_export(&std::fs::read(path).unwrap()).unwrap();
    let mut s = session(32, config());
    let mut w = workspace(32);
    let f = forces();
    let mut model_steps = 0;
    let mut div = vec![0.0; grid(32).cell_len()];
    let g = grid(32);
    let op = PressureOperator::new(&g, 1.25).unwrap();
    for frame in 1..=24 {
        let before = bytes(&s);
        if frame == 12 {
            assert!(
                s.step_with_pressure_provider(inputs(&f), &mut w, Some(&mut model), None, |v| v
                    == LiquidStepStage::BeforeCommit)
                    .is_err()
            );
            assert_eq!(bytes(&s), before);
        }
        let mut request = inputs(&f);
        if frame == 9 {
            request.requested_dt *= 0.5;
        }
        let (step, p) = s
            .step_with_pressure_provider(request, &mut w, Some(&mut model), None, |_| false)
            .unwrap();
        if frame == 9 {
            assert!(matches!(
                p.rejection,
                Some(ProviderRejection::Unsupported(_))
            ));
            assert_eq!(p.selected, PressureImplementation::JacobiPcgV1.id());
        }
        model_steps += usize::from(p.selected == "lab-modal-pressure-v1");
        assert!(p.attempted.as_ref().unwrap().provenance.is_some());
        let snapshot = s.snapshot();
        let v = snapshot.state();
        assert_eq!(snapshot.frame(), frame);
        assert_eq!(v.carrier.generation, frame as u64);
        op.divergence([v.carrier.x, v.carrier.y, v.carrier.z], &mut div)
            .unwrap();
        let actual = div.iter().map(|x| x.abs()).fold(0.0, f64::max);
        assert_eq!(actual, step.carrier.step.actual_divergence_max);
        assert!(actual <= config().actual_divergence_limit);
        assert!(
            v.carrier
                .tracer
                .iter()
                .all(|x| x.is_finite() && (0.0..=1.0).contains(x))
        );
        assert!(
            v.liquid
                .fraction
                .iter()
                .all(|x| x.is_finite() && (0.0..=1.0).contains(x))
        );
    }
    assert!(
        model_steps > 0,
        "real provider must be selected, not merely attempted"
    );
    eprintln!("production provider trajectory: {model_steps}/24 selected model");
}

#[test]
fn unsupported_surface_domains_and_workspace_identity_reject_before_callbacks() {
    let g = grid(4);
    let method = PressureImplementation::JacobiPcgV1;
    let cfg = LiquidTransportConfig {
        carrier: config(),
        represented_density: 1.25,
        volume_stamp: VolumeStamp { id: 1, version: 0 },
        carrier_id: 2,
    };
    let slab =
        SlabFreeSurface::new(g.clone(), Axis::Y, 2, VolumeStamp { id: 3, version: 0 }).unwrap();
    let fractions: Vec<_> = (0..g.cell_len())
        .map(|i| if slab.is_wet_cell(i) { 1.0 } else { 0.0 })
        .collect();
    let mut owners = [
        LiquidTransportSimulation::with_free_surface(
            g.clone(),
            cfg,
            method,
            fractions.clone(),
            slab,
        )
        .unwrap(),
        LiquidTransportSimulation::with_reconstructed_surface(
            g.clone(),
            cfg,
            method,
            fractions,
            Axis::Y,
        )
        .unwrap(),
    ];
    let mut w = workspace(4);
    for owner in &mut owners {
        assert!(matches!(
            owner.step_with_pressure_provider(inputs(&[]), &mut w, None, |_| panic!(
                "unsupported domain callback"
            )),
            Err(LiquidStepError::Carrier(
                SimulationError::UnsupportedProviderDomain
            ))
        ));
        assert_eq!(owner.state().carrier.generation, 0);
    }
    let mut s = session(4, config());
    let before = bytes(&s);
    for (density, method) in [
        (1000.0, method),
        (1.25, PressureImplementation::SymmetricGaussSeidelPcgV1),
    ] {
        let mut wrong = ProviderStepWorkspace::new(&g, density, CAP, method).unwrap();
        assert!(matches!(
            s.step_with_pressure_provider(inputs(&[]), &mut wrong, None, None, |_| panic!(
                "mismatch callback"
            )),
            Err(SessionError::Step(LiquidStepError::Carrier(
                SimulationError::ProviderWorkspaceMismatch
            )))
        ));
        assert_eq!(bytes(&s), before);
    }
}

#[test]
fn liquid_failure_after_qualified_pressure_preserves_pair_and_metadata() {
    let mut s = session(4, config());
    let mut w = workspace(4);
    let f = forces();
    let (step, _) = s
        .step_with_pressure_provider(inputs(&f), &mut w, None, None, |_| false)
        .unwrap();
    assert_eq!(step.provider_workspace_array_bytes, w.allocated_bytes());
    assert_eq!(
        step.total_array_bytes,
        step.owned_array_bytes
            + step.boundary_workspace_array_bytes
            + step.viscosity_workspace_array_bytes
            + step.provider_workspace_array_bytes
    );
    let before = bytes(&s);
    let previous = s.snapshot().provider_report().unwrap().total;
    let rates = vec![100.0; grid(4).cell_len()];
    let mut request = inputs(&f);
    request.source =
        Some(LiquidVolumeSource::new(VolumeStamp { id: 19, version: 2 }, &rates).unwrap());
    let mut pressure_passed = false;
    assert!(matches!(
        s.step_with_pressure_provider(request, &mut w, None, None, |stage| {
            if stage == LiquidStepStage::Carrier(StepStage::BeforeCommit) {
                pressure_passed = true
            }
            false
        }),
        Err(SessionError::Step(LiquidStepError::Volume(
            LiquidVolumeError::FractionBounds { .. }
        )))
    ));
    assert!(pressure_passed);
    assert_eq!(bytes(&s), before);
    assert_eq!(s.snapshot().provider_report().unwrap().total, previous);
    s.step_with_pressure_provider(inputs(&f), &mut w, None, None, |_| false)
        .unwrap();
}

#[cfg(feature = "accepted-pressure-warm-start")]
#[test]
fn preferred_and_fallback_remain_cold_after_accepted_native_history() {
    for reject in [false, true] {
        let mut a = session(4, config());
        let mut b = session(4, config());
        let mut aw = workspace(4);
        let mut bw = workspace(4);
        let policy = PressureInitializationPolicy::AcceptedPreviousV1;
        for _ in 0..2 {
            a.step_with_initialization(inputs(&[]), policy, None, |_| false)
                .unwrap();
            b.step_with_initialization(
                inputs(&[]),
                PressureInitializationPolicy::Cold,
                None,
                |_| false,
            )
            .unwrap();
        }
        assert_eq!(bytes(&a), bytes(&b));
        assert_eq!(
            a.snapshot()
                .report()
                .unwrap()
                .pressure_initialization
                .actual,
            policy
        );
        let (ar, ap) = a
            .step_with_pressure_provider(
                inputs(&[]),
                &mut aw,
                Some(&mut Probe { reject }),
                None,
                |_| false,
            )
            .unwrap();
        let (br, bp) = b
            .step_with_pressure_provider(
                inputs(&[]),
                &mut bw,
                Some(&mut Probe { reject }),
                None,
                |_| false,
            )
            .unwrap();
        assert_eq!(
            ar.pressure_initialization.actual,
            PressureInitializationPolicy::Cold
        );
        assert_eq!(
            br.pressure_initialization.actual,
            PressureInitializationPolicy::Cold
        );
        assert_eq!(ap.selected, bp.selected);
        assert_eq!(bytes(&a), bytes(&b));
        let accepted = bytes(&a);
        let initialization = a.snapshot().report().unwrap().pressure_initialization;
        let selected = a.snapshot().provider_report().unwrap().selected;
        assert!(
            a.step_with_initialization(inputs(&[]), policy, None, |stage| stage
                == LiquidStepStage::BeforeCommit)
                .is_err()
        );
        assert_eq!(bytes(&a), accepted);
        assert_eq!(
            a.snapshot().report().unwrap().pressure_initialization,
            initialization
        );
        assert_eq!(a.snapshot().provider_report().unwrap().selected, selected);
        let next = a
            .step_with_initialization(inputs(&[]), policy, None, |_| false)
            .unwrap();
        assert_eq!(
            next.pressure_initialization.cold_reason,
            Some(PressureColdReason::Invalidated)
        );
    }
}
