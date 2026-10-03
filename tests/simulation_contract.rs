use rheon::{
    Axis, BufferPlan, GridGeometry, PressureOperator, PressureSettings, PressureWorkspace,
    Simulation, SimulationConfig, SimulationError, SmokeSource, StateView, StepStage,
};
fn config() -> SimulationConfig {
    SimulationConfig {
        density: 1.0,
        memory_limit: 64 * 1024 * 1024,
        pressure: PressureSettings {
            relative_residual: 1e-10,
            absolute_residual: 1e-12,
            divergence_limit: 1e-8,
            max_iterations: 1000,
        },
        actual_divergence_limit: 1e-5,
        max_courant: 1.0,
    }
}
fn grid() -> GridGeometry {
    GridGeometry::new([8, 8, 8], [0.125; 3], [0.0; 3]).unwrap()
}
fn source() -> SmokeSource {
    SmokeSource {
        lower: [0.25, 0.125, 0.25],
        upper: [0.75, 0.375, 0.75],
        tracer_rate: 1.0,
        vertical_acceleration: 0.5,
    }
}
fn snapshot(s: StateView<'_>) -> (Vec<u32>, u64, u64) {
    (
        s.x.iter()
            .chain(s.y)
            .chain(s.z)
            .chain(s.tracer)
            .map(|x| x.to_bits())
            .collect(),
        s.time.to_bits(),
        s.generation,
    )
}
#[test]
fn all_owned_capacities_are_accounted_and_cap_checked() {
    let g = grid();
    let expected = BufferPlan::for_grid(&g, usize::MAX).unwrap().total_bytes;
    let mut c = config();
    c.memory_limit = expected;
    let sim = Simulation::new(g.clone(), c).unwrap();
    assert_eq!(sim.allocated_bytes(), expected);
    c.memory_limit = expected - 1;
    assert!(Simulation::new(g, c).is_err());
}
#[test]
fn localized_smoke_is_nontrivial_finite_and_independently_divergence_checked() {
    let mut sim = Simulation::new(grid(), config()).unwrap();
    for _ in 0..6 {
        let report = sim.step(0.02, Some(source()), |_| false).unwrap();
        assert!(report.tracer_integral > 0.0 && report.kinetic_energy > 0.0);
        let g = sim.grid();
        let s = sim.state();
        let mut maximum = 0.0_f64;
        let [nx, ny, nz] = g.counts();
        let h = g.spacing();
        for k in 0..nz {
            for j in 0..ny {
                for i in 0..nx {
                    let ux = |x| f64::from(s.x[g.face_index(Axis::X, [x, j, k]).unwrap()]);
                    let vy = |y| f64::from(s.y[g.face_index(Axis::Y, [i, y, k]).unwrap()]);
                    let wz = |z| f64::from(s.z[g.face_index(Axis::Z, [i, j, z]).unwrap()]);
                    let div = (ux(i + 1) - ux(i)) / h[0]
                        + (vy(j + 1) - vy(j)) / h[1]
                        + (wz(k + 1) - wz(k)) / h[2];
                    maximum = maximum.max(div.abs());
                }
            }
        }
        assert_eq!(report.actual_divergence_max, maximum);
        assert!(maximum <= 1e-5);
        assert!(
            s.tracer
                .iter()
                .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
        );
    }
}
#[test]
fn cancellation_at_every_stage_preserves_all_accepted_bits_and_metadata() {
    for stage in [
        StepStage::BeforeAdvection,
        StepStage::VelocitySlice,
        StepStage::BeforePressure,
        StepStage::PressureIteration,
        StepStage::BeforeTracer,
        StepStage::TracerSlice,
        StepStage::BeforeCommit,
    ] {
        let mut sim = Simulation::new(grid(), config()).unwrap();
        sim.step(0.02, Some(source()), |_| false).unwrap();
        let before = snapshot(sim.state());
        assert!(
            sim.step(0.02, Some(source()), |s| s == stage).is_err(),
            "{stage:?}"
        );
        assert_eq!(snapshot(sim.state()), before, "{stage:?}");
        sim.step(0.02, Some(source()), |_| false).unwrap();
        let mut reference = Simulation::new(grid(), config()).unwrap();
        reference.step(0.02, Some(source()), |_| false).unwrap();
        reference.step(0.02, Some(source()), |_| false).unwrap();
        assert_eq!(snapshot(sim.state()), snapshot(reference.state()));
    }
}
#[test]
fn solver_exhaustion_and_invalid_source_are_transactional() {
    let mut c = config();
    c.pressure.max_iterations = 0;
    let mut sim = Simulation::new(grid(), c).unwrap();
    let before = snapshot(sim.state());
    assert!(sim.step(0.02, Some(source()), |_| false).is_err());
    assert_eq!(snapshot(sim.state()), before);
    let mut bad = source();
    bad.tracer_rate = f64::NAN;
    assert_eq!(
        sim.step(0.02, Some(bad), |_| false).unwrap_err(),
        SimulationError::InvalidSource
    );
    assert_eq!(snapshot(sim.state()), before);
}
#[test]
fn exact_replay_pause_reset_and_source_toggle() {
    let mut a = Simulation::new(grid(), config()).unwrap();
    let mut b = Simulation::new(grid(), config()).unwrap();
    for _ in 0..4 {
        a.step(0.02, Some(source()), |_| false).unwrap();
        b.step(0.02, Some(source()), |_| false).unwrap();
    }
    assert_eq!(snapshot(a.state()), snapshot(b.state()));
    let before = snapshot(a.state());
    a.set_paused(true);
    assert_eq!(
        a.step(0.02, None, |_| false).unwrap_err(),
        SimulationError::Paused
    );
    assert_eq!(snapshot(a.state()), before);
    a.set_paused(false);
    a.step(0.02, None, |_| false).unwrap();
    a.reset().unwrap();
    assert_eq!(a.state().time, 0.0);
    assert_eq!(a.state().generation, 6);
    let r = a.step(0.02, None, |_| false).unwrap();
    assert_eq!(r.tracer_integral, 0.0);
    assert_eq!(r.kinetic_energy, 0.0);
}
#[test]
fn single_cell_rest_and_saturating_source_are_supported() {
    let g = GridGeometry::new([1; 3], [1.0; 3], [0.0; 3]).unwrap();
    let mut s = Simulation::new(g, config()).unwrap();
    let source = SmokeSource {
        lower: [0.0; 3],
        upper: [1.0; 3],
        tracer_rate: 10.0,
        vertical_acceleration: 0.0,
    };
    let r = s.step(0.25, Some(source), |_| false).unwrap();
    assert_eq!(r.actual_divergence_max, 0.0);
    assert_eq!(s.state().tracer, [1.0]);
    assert_eq!(r.pressure.iterations, 0);
}
#[test]
fn time_step_policy_reports_reduced_dt_for_large_forcing() {
    let mut s = Simulation::new(grid(), config()).unwrap();
    let mut source = source();
    source.vertical_acceleration = 8.0;
    let r = s.step(1.0, Some(source), |_| false).unwrap();
    assert!(r.dt < 1.0);
    assert!(r.courant <= 1.0);
}
#[test]
fn in_place_candidate_matches_separate_output_and_scratch_diagnostic() {
    let g = GridGeometry::new([3, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let op = PressureOperator::new(&g, 1.0).unwrap();
    let u = [0.0, 2.0, -1.0, 0.0];
    let v = [0.0; 6];
    let w = [0.0; 6];
    let pressure = [0.0, 4.0, 2.0];
    let (mut a, mut b, mut c) = ([0.0; 4], [0.0; 6], [0.0; 6]);
    op.correct_velocity([&u, &v, &w], &pressure, 0.5, [&mut a, &mut b, &mut c])
        .unwrap();
    let (mut x, mut y, mut z) = (u, v, w);
    op.correct_candidate_in_place(&pressure, 0.5, [&mut x, &mut y, &mut z])
        .unwrap();
    assert_eq!((a, b, c), (x, y, z));
    let mut workspace = PressureWorkspace::new(&g, 144).unwrap();
    assert_eq!(
        workspace.actual_divergence_max(&op, [&x, &y, &z]).unwrap(),
        0.0
    );
}

#[test]
fn actual_float_divergence_rejection_preserves_accepted_state() {
    let mut c = config();
    c.actual_divergence_limit = 0.0;
    let mut sim = Simulation::new(grid(), c).unwrap();
    sim.step(0.02, None, |_| false).unwrap();
    let before = snapshot(sim.state());
    assert!(matches!(
        sim.step(0.02, Some(source()), |_| false),
        Err(SimulationError::DivergenceLimit { .. })
    ));
    assert_eq!(snapshot(sim.state()), before);
}
