use rheon::{
    DomainFill, FluidPhase, FluidPreset, GridGeometry, ParticleFluidConfig, ParticleFluidError,
    ParticleFluidSimulation, RotatingCylinderEmitter,
};
fn grid(side: f64) -> GridGeometry {
    GridGeometry::new([32; 3], [side / 32.0; 3], [0.0; 3]).unwrap()
}
fn config() -> ParticleFluidConfig {
    ParticleFluidConfig {
        gravity: [0.0; 3],
        ..ParticleFluidConfig::default()
    }
}
fn emitter() -> RotatingCylinderEmitter {
    RotatingCylinderEmitter {
        radius: 0.2,
        pressure_heads: [100000.0; 2],
        rotations_per_second: 0.0,
        ..RotatingCylinderEmitter::default()
    }
}

#[test]
fn presets_have_explicit_units_states_and_reference_gas_pressure() {
    for preset in FluidPreset::ALL {
        assert!(preset.material().is_valid(), "{}", preset.label());
    }
    assert_eq!(FluidPreset::Water.material().phase, FluidPhase::Liquid);
    assert_eq!(FluidPreset::Helium.material().phase, FluidPhase::Gas);
    assert!(FluidPreset::Mercury.material().density > 13.0 * FluidPreset::Water.material().density);
    assert!(
        FluidPreset::Syrup.material().viscosity > 1000.0 * FluidPreset::Water.material().viscosity
    );
    for preset in [FluidPreset::Air, FluidPreset::Argon, FluidPreset::Helium] {
        let m = preset.material();
        assert!((m.pressure(m.density, m.temperature_kelvin, 20.0) - 101325.0).abs() < 1e-9);
        assert!(
            (m.pressure(2.0 * m.density, 2.0 * m.temperature_kelvin, 20.0) - 405300.0).abs() < 1e-8
        );
    }
}
#[test]
fn vacuum_contains_no_carrier_mass_and_filled_mass_scales_in_cubic_metres() {
    let mut vacuum = ParticleFluidSimulation::new(grid(1.0), config()).unwrap();
    assert!(vacuum.particles().is_empty());
    let r = vacuum.step(0.02, None, || false).unwrap();
    assert_eq!((r.particles, r.total_mass), (0, 0.0));
    assert_eq!(vacuum.time(), 0.02);
    for side in [0.01, 1.0, 3.0] {
        for fill in [DomainFill::EmittedFluid, DomainFill::OtherFluid] {
            let c = ParticleFluidConfig { fill, ..config() };
            let sim = ParticleFluidSimulation::new(grid(side), c).unwrap();
            let expected = side.powi(3)
                * if fill == DomainFill::EmittedFluid {
                    c.material.density
                } else {
                    c.ambient_material.density
                };
            let mass = sim.particles().iter().map(|p| p.mass).sum::<f64>();
            assert!((mass - expected).abs() < expected * 1e-12);
            assert!(
                sim.particles()
                    .iter()
                    .all(|p| p.position.iter().all(|v| *v > 0.0 && *v < side))
            );
        }
    }
}
#[test]
fn scaled_nozzles_inject_physical_cubic_volume_instead_of_unit_cube_mass() {
    let mut material = FluidPreset::Water.material();
    material.viscosity = 0.0;
    material.surface_tension = 0.0;
    let run = |side: f64| {
        let mut sim = ParticleFluidSimulation::new(
            grid(side),
            ParticleFluidConfig {
                material,
                ..config()
            },
        )
        .unwrap();
        let mut e = emitter();
        e.center = e.center.map(|v| v * side);
        e.half_length *= side;
        e.radius *= side;
        e.nozzle_depth *= side;
        let mut supplied = 0.0;
        for _ in 0..64 {
            supplied += sim.step(0.02, Some(e), || false).unwrap().emitted_mass;
        }
        (sim.particles().len(), supplied, sim.time())
    };
    let large = run(1.0);
    let small = run(0.1);
    assert!(large.0 > 0);
    assert_eq!(large.0, small.0);
    assert!((small.1 / large.1 - 0.001).abs() < 1e-12);
    assert!((small.2 / large.2 - 0.1).abs() < 1e-9);
}
#[test]
fn liquid_and_gas_emit_into_vacuum_and_retain_mass_without_silent_particle_loss() {
    for preset in [
        FluidPreset::Water,
        FluidPreset::Mercury,
        FluidPreset::MoltenTin,
        FluidPreset::Argon,
        FluidPreset::Helium,
        FluidPreset::Slime,
    ] {
        let c = ParticleFluidConfig {
            material: preset.material(),
            ..config()
        };
        let mut sim = ParticleFluidSimulation::new(grid(1.0), c).unwrap();
        let mut supplied = 0.0;
        for _ in 0..64 {
            let r = sim.step(0.02, Some(emitter()), || false).unwrap();
            supplied += r.emitted_mass;
            assert!((r.total_mass - supplied).abs() < 1e-10 * supplied.max(1.0));
        }
        assert!(!sim.particles().is_empty(), "{}", preset.label());
        assert!(
            sim.particles().iter().all(|p| p
                .position
                .iter()
                .chain(&p.velocity)
                .all(|v| v.is_finite()))
        );
        assert!(sim.time() > 0.0);
    }
}
#[test]
fn source_cancellation_and_capacity_failure_preserve_accepted_fields_clock_and_replay() {
    let mut a = ParticleFluidSimulation::new(grid(1.0), config()).unwrap();
    let mut b = ParticleFluidSimulation::new(grid(1.0), config()).unwrap();
    let mut calls = 0;
    assert_eq!(
        a.step(0.02, Some(emitter()), || {
            calls += 1;
            calls == 4
        })
        .unwrap_err(),
        ParticleFluidError::Cancelled
    );
    assert!(a.particles().is_empty());
    assert_eq!((a.time(), a.generation()), (0.0, 0));
    a.step(0.02, Some(emitter()), || false).unwrap();
    b.step(0.02, Some(emitter()), || false).unwrap();
    assert_eq!(a.particles(), b.particles());
    assert_eq!(a.time().to_bits(), b.time().to_bits());
    let mut small = ParticleFluidSimulation::new(
        grid(1.0),
        ParticleFluidConfig {
            max_particles: 4,
            ..config()
        },
    )
    .unwrap();
    for _ in 0..100 {
        let old = small.particles().to_vec();
        let time = small.time();
        let generation = small.generation();
        if let Err(e) = small.step(0.02, Some(emitter()), || false) {
            assert_eq!(e, ParticleFluidError::ParticleLimit { limit: 4 });
            assert_eq!(old, small.particles());
            assert_eq!(small.time(), time);
            assert_eq!(small.generation(), generation);
            return;
        }
    }
    panic!("source must reach the declared particle limit");
}
#[test]
fn invalid_units_geometry_and_budget_have_explicit_failures() {
    let mut c = config();
    c.material.density = 0.0;
    assert!(matches!(
        ParticleFluidSimulation::new(grid(1.0), c),
        Err(ParticleFluidError::InvalidSettings)
    ));
    assert!(matches!(
        ParticleFluidSimulation::new(
            grid(1.0),
            ParticleFluidConfig {
                memory_limit: 100,
                ..config()
            }
        ),
        Err(ParticleFluidError::BufferLimit { .. })
    ));
    let mut s = ParticleFluidSimulation::new(grid(1.0), config()).unwrap();
    let mut e = emitter();
    e.center = [2.0; 3];
    assert_eq!(
        s.step(0.02, Some(e), || false).unwrap_err(),
        ParticleFluidError::InvalidEmitter
    );
    assert_eq!(
        s.step(f64::NAN, None, || false).unwrap_err(),
        ParticleFluidError::InvalidTimeStep
    );
    assert_eq!(s.generation(), 0);
}

#[test]
fn long_rotating_scenes_and_surrounding_material_remain_finite_with_mass_accounted() {
    for (preset, fill, steps) in [
        (FluidPreset::Water, DomainFill::Vacuum, 2000),
        (FluidPreset::Argon, DomainFill::Vacuum, 1024),
        (FluidPreset::Slime, DomainFill::Vacuum, 1024),
        (FluidPreset::Mercury, DomainFill::Vacuum, 1024),
        (FluidPreset::Water, DomainFill::OtherFluid, 1024),
        (FluidPreset::Helium, DomainFill::OtherFluid, 1024),
    ] {
        let c = ParticleFluidConfig {
            material: preset.material(),
            fill,
            ..ParticleFluidConfig::default()
        };
        let mut s = ParticleFluidSimulation::new(grid(1.0), c).unwrap();
        let initial = s.particles().iter().map(|p| p.mass).sum::<f64>();
        let mut supplied = 0.0;
        let e = RotatingCylinderEmitter {
            pressure_heads: [1000.0, 800.0],
            ..RotatingCylinderEmitter::default()
        };
        for step in 0..steps {
            let r = s
                .step(0.02, Some(e), || false)
                .unwrap_or_else(|e| panic!("{} {fill:?} at {step}: {e}", preset.label()));
            supplied += r.emitted_mass;
            assert!((r.total_mass - initial - supplied).abs() < 1e-10 * r.total_mass.max(1.0));
            assert!(r.max_speed.is_finite() && r.max_pressure.is_finite());
        }
        assert!(
            s.particles().iter().any(|p| p.material == 0),
            "{} {fill:?}",
            preset.label()
        );
    }
}

#[test]
fn a_filling_water_floor_does_not_generate_a_ceiling_height_rebound() {
    let mut s = ParticleFluidSimulation::new(grid(1.0), ParticleFluidConfig::default()).unwrap();
    let emitter = RotatingCylinderEmitter {
        pressure_heads: [1000.0, 800.0],
        ..RotatingCylinderEmitter::default()
    };
    let mut hit_floor = false;
    for step in 0..12000 {
        s.step(0.02, Some(emitter), || false).unwrap();
        hit_floor |= s.particles().iter().any(|p| p.position[1] < 0.02);
        if hit_floor {
            let top = s
                .particles()
                .iter()
                .map(|p| p.position[1])
                .fold(0.0_f64, f64::max);
            assert!(
                top < 0.8,
                "step {step}, t={}: artificial floor rebound reached {top} m",
                s.time()
            );
        }
    }
    assert!(hit_floor);
    assert!(s.time() > 10.0);
    assert!(s.particles().len() > 800);
}
#[cfg(feature = "parallel")]
#[test]
fn particle_neighbors_and_results_replay_exactly_across_cpu_thread_counts() {
    let run = |threads| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap()
            .install(|| {
                let mut sim = ParticleFluidSimulation::new(
                    grid(1.0),
                    ParticleFluidConfig {
                        fill: DomainFill::EmittedFluid,
                        samples_per_axis: 26,
                        ..config()
                    },
                )
                .unwrap();
                sim.step(0.00001, None, || false).unwrap();
                (sim.time(), sim.particles().to_vec())
            })
    };
    assert_eq!(run(1), run(4));
}
