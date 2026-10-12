use rheon::{
    GridGeometry, PressureSettings, RotatingCylinderEmitter, Simulation, SimulationConfig,
    StepStage,
};
fn sim() -> Simulation {
    Simulation::new(
        GridGeometry::new([24; 3], [1.0 / 24.0; 3], [0.0; 3]).unwrap(),
        SimulationConfig {
            density: 1.0,
            memory_limit: 4 * 1024 * 1024,
            pressure: PressureSettings {
                relative_residual: 1e-9,
                absolute_residual: 1e-12,
                divergence_limit: 1e-7,
                max_iterations: 2000,
            },
            actual_divergence_limit: 1e-5,
            max_courant: 1.0,
        },
    )
    .unwrap()
}
fn bits(sim: &Simulation) -> Vec<u32> {
    let s = sim.state();
    s.x.iter()
        .chain(s.y)
        .chain(s.z)
        .chain(s.tracer)
        .map(|v| v.to_bits())
        .collect()
}
#[test]
fn cylinder_rotates_about_y_once_per_simulated_second() {
    let e = RotatingCylinderEmitter::default();
    assert_eq!(e.axis_at(0.0).unwrap(), e.axis_at(1.0).unwrap());
    let quarter = e.axis_at(0.25).unwrap();
    assert!(quarter[0].abs() < 1e-14);
    assert_eq!(quarter[1], 0.0);
    assert!((quarter[2] - 1.0).abs() < 1e-14);
    assert!(e.axis_at(f64::NAN).is_err());
}
#[test]
fn both_end_jets_create_real_projected_flow_and_bounded_concentration() {
    let mut s = sim();
    let r = s
        .step_with_emitter(0.02, RotatingCylinderEmitter::default(), |_| false)
        .unwrap();
    assert!(s.state().x.iter().any(|v| *v > 0.001));
    assert!(s.state().x.iter().any(|v| *v < -0.001));
    let g = s.grid();
    let [nx, ny, _] = g.counts();
    let mut left = false;
    let mut right = false;
    for (i, v) in s.state().tracer.iter().enumerate() {
        if *v > 0.0 {
            if i % nx < nx / 2 {
                left = true;
            } else {
                right = true;
            }
        }
        assert!((0.0..=1.0).contains(v));
    }
    assert!(left && right);
    assert_eq!(ny, 24);
    assert!(r.actual_divergence_max <= 1e-5);
    assert!(r.kinetic_energy > 0.0);
}
#[test]
fn emitter_errors_and_cancellation_preserve_accepted_state_and_retry() {
    let mut s = sim();
    let emitter = RotatingCylinderEmitter::default();
    s.step_with_emitter(0.02, emitter, |_| false).unwrap();
    let before = bits(&s);
    let time = s.state().time;
    let generation = s.state().generation;
    for stage in [
        StepStage::ForceSlice,
        StepStage::PressureIteration,
        StepStage::TracerSlice,
        StepStage::BeforeCommit,
    ] {
        assert!(
            s.step_with_emitter(0.02, emitter, |at| at == stage)
                .is_err()
        );
        assert_eq!(bits(&s), before);
        assert_eq!(s.state().time, time);
        assert_eq!(s.state().generation, generation);
    }
    let invalid = RotatingCylinderEmitter {
        pressure_heads: [f64::NAN, 1.0],
        ..emitter
    };
    assert!(s.step_with_emitter(0.02, invalid, |_| false).is_err());
    assert_eq!(bits(&s), before);
    let mut reference = sim();
    reference
        .step_with_emitter(0.02, emitter, |_| false)
        .unwrap();
    reference
        .step_with_emitter(0.02, emitter, |_| false)
        .unwrap();
    s.step_with_emitter(0.02, emitter, |_| false).unwrap();
    assert_eq!(bits(&s), bits(&reference));
}
