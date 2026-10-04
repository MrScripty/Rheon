use rheon::{
    GridGeometry, PressureError, PressureImplementation, PressureOperator, PressureSettings,
    PressureWorkspace, Simulation, SimulationConfig, SmokeSource, StepStage,
};

fn settings() -> PressureSettings {
    PressureSettings {
        relative_residual: 1e-11,
        absolute_residual: 1e-12,
        divergence_limit: 1e-9,
        max_iterations: 1000,
    }
}
fn simulation(method: PressureImplementation) -> Simulation {
    Simulation::with_implementation(
        GridGeometry::new([8, 7, 6], [0.125, 0.15, 0.17], [0.0; 3]).unwrap(),
        SimulationConfig {
            density: 1.0,
            memory_limit: 1024 * 1024,
            pressure: settings(),
            actual_divergence_limit: 1e-5,
            max_courant: 1.0,
        },
        method,
    )
    .unwrap()
}
fn snapshot(s: &Simulation) -> Vec<u32> {
    let s = s.state();
    s.x.iter()
        .chain(s.y)
        .chain(s.z)
        .chain(s.tracer)
        .map(|x| x.to_bits())
        .collect()
}
fn source() -> Option<SmokeSource> {
    Some(SmokeSource {
        lower: [0.2, 0.1, 0.2],
        upper: [0.7, 0.4, 0.7],
        tracer_rate: 2.0,
        vertical_acceleration: 1.0,
    })
}
#[test]
fn registry_has_stable_unique_ids_and_explicit_unknown_rejection() {
    assert_eq!(PressureImplementation::default().id(), "jacobi-pcg-v1");
    for method in PressureImplementation::ALL {
        assert_eq!(PressureImplementation::from_id(method.id()), Some(method));
    }
    assert_eq!(PressureImplementation::from_id("future-solver"), None);
    assert_ne!(
        PressureImplementation::ALL[0].id(),
        PressureImplementation::ALL[1].id()
    );
}
#[test]
fn all_methods_recover_anisotropic_manufactured_pressure_and_failure_contracts() {
    for method in PressureImplementation::ALL {
        for counts in [[1, 1, 1], [3, 1, 1], [5, 4, 3]] {
            let g = GridGeometry::new(counts, [0.2, 0.35, 0.7], [0.0; 3]).unwrap();
            let op = PressureOperator::new(&g, 1.3).unwrap();
            let exact: Vec<_> = (0..g.cell_len()).map(|i| (i as f64 * 0.7).sin()).collect();
            let mut rhs = vec![0.0; g.cell_len()];
            op.apply_full(&exact, &mut rhs).unwrap();
            let mut ws =
                PressureWorkspace::with_implementation(&g, 48 * g.cell_len(), method).unwrap();
            let report = ws.solve_rhs(&op, &rhs, 0.02, settings(), || false).unwrap();
            assert!(report.predicted_divergence_max <= 1e-9);
            for (p, e) in ws.pressure().iter().zip(&exact) {
                assert!((p - e + exact[0]).abs() < 1e-8, "{method:?}");
            }
            assert!(matches!(
                ws.solve_rhs(&op, &rhs, 0.02, settings(), || true),
                Err(PressureError::Cancelled { .. })
            ));
            rhs.fill(1.0);
            assert!(matches!(
                ws.solve_rhs(&op, &rhs, 0.02, settings(), || false),
                Err(PressureError::IncompatibleRhs { .. })
            ));
        }
    }
}
#[test]
fn every_implementation_preserves_transactional_cancellation_and_replay() {
    for method in PressureImplementation::ALL {
        let mut reference = simulation(method);
        reference.step(0.02, source(), |_| false).unwrap();
        for stage in [
            StepStage::BeforeAdvection,
            StepStage::VelocitySlice,
            StepStage::BeforePressure,
            StepStage::PressureIteration,
            StepStage::BeforeTracer,
            StepStage::TracerSlice,
            StepStage::BeforeCommit,
        ] {
            let mut s = simulation(method);
            let before = snapshot(&s);
            assert!(s.step(0.02, source(), |at| at == stage).is_err());
            assert_eq!(snapshot(&s), before);
            assert_eq!(s.state().time, 0.0);
            assert_eq!(s.state().generation, 0);
            s.step(0.02, source(), |_| false).unwrap();
            assert_eq!(snapshot(&s), snapshot(&reference));
            assert_eq!(s.implementation(), method);
        }
    }
}
#[test]
fn approaches_agree_with_equal_inputs_and_retained_budget() {
    let mut a = simulation(PressureImplementation::JacobiPcgV1);
    let mut b = simulation(PressureImplementation::SymmetricGaussSeidelPcgV1);
    assert_eq!(a.allocated_bytes(), b.allocated_bytes());
    for step in 0..6 {
        let input = if step < 3 { source() } else { None };
        let ar = a.step(0.02, input, |_| false).unwrap();
        let br = b.step(0.02, input, |_| false).unwrap();
        assert_eq!(ar.dt, br.dt);
        assert!(ar.actual_divergence_max < 1e-5 && br.actual_divergence_max < 1e-5);
    }
    let av = a.state();
    let bv = b.state();
    for (a, b) in
        av.x.iter()
            .chain(av.y)
            .chain(av.z)
            .chain(av.tracer)
            .zip(bv.x.iter().chain(bv.y).chain(bv.z).chain(bv.tracer))
    {
        assert!((a - b).abs() < 1e-6);
    }
}
