use rheon::{GridGeometry, ParticleFluidConfig, ParticleFluidSimulation, RotatingCylinderEmitter};

#[test]
fn emitted_indices_retain_identity_and_evaluations_share_the_accepted_generation() {
    let grid = GridGeometry::new([16; 3], [1.0 / 16.0; 3], [0.0; 3]).unwrap();
    let mut sim = ParticleFluidSimulation::new(
        grid,
        ParticleFluidConfig {
            gravity: [0.0; 3],
            ..Default::default()
        },
    )
    .unwrap();
    let emitter = RotatingCylinderEmitter {
        radius: 0.2,
        pressure_heads: [100000.0; 2],
        rotations_per_second: 0.0,
        ..Default::default()
    };
    for _ in 0..8 {
        let old = sim.particles().to_vec();
        let time = sim.time();
        sim.step(0.02, Some(emitter), || false).unwrap();
        let (evaluated, diagnostics) = sim.particle_diagnostics().unwrap();
        assert_eq!(evaluated, time);
        assert_eq!(diagnostics.len(), sim.particles().len());
        for (before, after) in old.iter().zip(sim.particles()) {
            assert_eq!(before.mass, after.mass);
            assert_eq!(before.material, after.material);
        }
        assert!(diagnostics.iter().all(|d| d.density.is_finite()
            && d.density > 0.0
            && d.pressure.is_finite()
            && d.acceleration.iter().all(|v| v.is_finite())));
    }
    assert!(!sim.particles().is_empty());
}
