//! Prepared bounded adapter integration checks. Every test is ignored so normal
//! suites cannot execute these simulations before review/explicit authorization.
//! No emitters, model processes, native UI, files, threads, or workloads here.
use rheon::*;
fn fields(kind: ForceKind, units: ForceUnits) -> ForceSnapshot {
    ForceScene {
        objects: vec![ForceObject {
            id: SceneObjectId(1),
            name: "adapter".into(),
            enabled: true,
            transform: ForceTransform::default(),
            units,
            kind,
            support: ForceSupport::Global,
            time: ForceTime::default(),
        }],
        ..Default::default()
    }
    .freeze()
    .unwrap()
}
fn grid_sim() -> Simulation {
    Simulation::new(
        GridGeometry::new([2, 2, 1], [1.0; 3], [0.0; 3]).unwrap(),
        SimulationConfig {
            density: 2.0,
            memory_limit: 1024 * 1024,
            pressure: PressureSettings {
                relative_residual: 1e-11,
                absolute_residual: 1e-12,
                divergence_limit: 1e-9,
                max_iterations: 100,
            },
            actual_divergence_limit: 1e-6,
            max_courant: 1.0,
        },
    )
    .unwrap()
}
fn grid_state(s: &Simulation) -> (Vec<u32>, u64, u64) {
    let state = s.state();
    (
        state
            .x
            .iter()
            .chain(state.y)
            .chain(state.z)
            .chain(state.tracer)
            .map(|v| v.to_bits())
            .collect(),
        state.time.to_bits(),
        state.generation,
    )
}
#[test]
#[ignore = "bounded solver integration: review and explicit execution authorization required"]
fn authored_mac_matches_legacy_bounded_directional_force() {
    let mut authored = grid_sim();
    let mut legacy = grid_sim();
    let mut scene = fields(
        ForceKind::Directional {
            vector: [8.0, 0.0, 0.0],
        },
        ForceUnits::ForceDensity,
    )
    .scene()
    .clone();
    scene.objects[0].transform.translation = [1.0, 0.5, 0.5];
    scene.objects[0].support = ForceSupport::Box {
        half_extent: [1.0, 0.5, 0.5],
        edge_fraction: 0.0,
    };
    let scene = scene.freeze().unwrap();
    let a = authored
        .step_with_force_fields(0.001, None, &scene, |_| false)
        .unwrap();
    let b = legacy
        .step_with_forces(
            0.001,
            None,
            &[BodyForce {
                value: [8.0, 0.0, 0.0],
                units: ForceUnits::ForceDensity,
                region: Some(ForceRegion {
                    lower: [0.0; 3],
                    upper: [2.0, 1.0, 1.0],
                }),
            }],
            |_| false,
        )
        .unwrap();
    assert_eq!(grid_state(&authored), grid_state(&legacy));
    assert_eq!(a.forces, b.forces);
}
#[test]
#[ignore = "bounded solver integration: review and explicit execution authorization required"]
fn authored_mac_cancellation_retains_accepted_state_and_retry() {
    let f = fields(
        ForceKind::Vortex { strength: 0.25 },
        ForceUnits::Acceleration,
    );
    for stage in [
        StepStage::ForceSlice,
        StepStage::BeforePressure,
        StepStage::BeforeCommit,
    ] {
        let mut sim = grid_sim();
        let before = grid_state(&sim);
        assert!(matches!(
            sim.step_with_force_fields(0.001, None, &f, |s| s == stage),
            Err(SimulationError::Cancelled { .. })
        ));
        assert_eq!(grid_state(&sim), before);
        sim.step_with_force_fields(0.001, None, &f, |_| false)
            .unwrap();
        let mut reference = grid_sim();
        reference
            .step_with_force_fields(0.001, None, &f, |_| false)
            .unwrap();
        assert_eq!(grid_state(&sim), grid_state(&reference));
    }
}
fn particle_sim() -> ParticleFluidSimulation {
    ParticleFluidSimulation::new(
        GridGeometry::new([2; 3], [0.5; 3], [0.0; 3]).unwrap(),
        ParticleFluidConfig {
            fill: DomainFill::EmittedFluid,
            samples_per_axis: 4,
            max_particles: 64,
            memory_limit: 1024 * 1024,
            gravity: [0.0, -1.0, 0.0],
            ..Default::default()
        },
    )
    .unwrap()
}
#[test]
#[ignore = "bounded solver integration: review and explicit execution authorization required"]
fn authored_particle_density_conversion_adds_to_existing_acceleration() {
    for units in [ForceUnits::Acceleration, ForceUnits::ForceDensity] {
        let mut reference = particle_sim();
        let mut authored = particle_sim();
        let before: Vec<_> = authored.particles().iter().map(|p| p.position).collect();
        let f = fields(ForceKind::Radial { strength: -0.5 }, units);
        let r = reference.step(1e-6, None, || false).unwrap();
        let a = authored
            .step_with_force_fields(1e-6, None, &f, || false)
            .unwrap();
        assert_eq!(r.dt, a.dt);
        assert_eq!(a.particles, 64);
        let (_, baseline) = reference.particle_diagnostics().unwrap();
        let (_, actual) = authored.particle_diagnostics().unwrap();
        for ((p, b), a) in before.into_iter().zip(baseline).zip(actual) {
            assert_eq!(a.density, b.density);
            let expected = f.acceleration(p, 0.0, a.density).unwrap();
            for (d, value) in expected.into_iter().enumerate() {
                assert!((a.acceleration[d] - b.acceleration[d] - value).abs() < 1e-10);
            }
        }
    }
}
#[test]
#[ignore = "bounded solver integration: review and explicit execution authorization required"]
fn authored_particle_cancel_retains_state_and_retry() {
    let f = fields(
        ForceKind::Vortex { strength: 0.1 },
        ForceUnits::Acceleration,
    );
    let mut sim = particle_sim();
    let before = sim.particles().to_vec();
    let mut calls = 0;
    assert!(matches!(
        sim.step_with_force_fields(1e-6, None, &f, || {
            calls += 1;
            calls == 4
        }),
        Err(ParticleFluidError::Cancelled)
    ));
    assert_eq!(sim.particles(), before);
    assert_eq!(sim.time(), 0.0);
    assert_eq!(sim.generation(), 0);
    sim.step_with_force_fields(1e-6, None, &f, || false)
        .unwrap();
    let mut reference = particle_sim();
    reference
        .step_with_force_fields(1e-6, None, &f, || false)
        .unwrap();
    assert_eq!(sim.particles(), reference.particles());
    assert_eq!(sim.time(), reference.time());
    assert_eq!(sim.generation(), reference.generation());
}
#[test]
#[ignore = "bounded solver integration: review and explicit execution authorization required"]
fn empty_snapshot_preserves_both_legacy_paths() {
    let empty = ForceScene::default().freeze().unwrap();
    let mut a = grid_sim();
    let mut b = grid_sim();
    a.step_with_force_fields(0.001, None, &empty, |_| false)
        .unwrap();
    b.step(0.001, None, |_| false).unwrap();
    assert_eq!(grid_state(&a), grid_state(&b));
    let mut a = particle_sim();
    let mut b = particle_sim();
    a.step_with_force_fields(1e-6, None, &empty, || false)
        .unwrap();
    b.step(1e-6, None, || false).unwrap();
    assert_eq!(a.particles(), b.particles());
    assert_eq!(a.time(), b.time());
}
