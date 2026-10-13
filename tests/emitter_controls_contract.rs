use rheon::*;
fn grid() -> GridGeometry {
    GridGeometry::new([24; 3], [1.0 / 24.0; 3], [0.0; 3]).unwrap()
}
fn carrier() -> Simulation {
    Simulation::new(
        grid(),
        SimulationConfig {
            density: 1.25,
            memory_limit: 16 * 1024 * 1024,
            pressure: PressureSettings {
                relative_residual: 1e-9,
                absolute_residual: 1e-12,
                divergence_limit: 1e-7,
                max_iterations: 3000,
            },
            actual_divergence_limit: 1e-5,
            max_courant: 1.0,
        },
    )
    .unwrap()
}
fn config(preset: FluidPreset) -> ParticleFluidConfig {
    ParticleFluidConfig {
        material: preset.material(),
        samples_per_axis: 16,
        max_particles: 256,
        memory_limit: 16 * 1024 * 1024,
        gravity: [0.0; 3],
        ..Default::default()
    }
}
fn emitter() -> RotatingCylinderEmitter {
    RotatingCylinderEmitter {
        emitted_velocity: Some([3.0, 4.0, 0.0]),
        radius: 0.1,
        rotations_per_second: 0.25,
        ..Default::default()
    }
}
fn noise() -> EmitterTurbulence {
    EmitterTurbulence {
        seed: 812,
        strength: 0.4,
        spatial_scale: 0.08,
        time_scale: 0.03,
    }
}
#[test]
fn seeded_field_has_bounded_smooth_spatial_temporal_behavior() {
    let n = noise();
    for i in 0..100 {
        let p = [i as f64 * 0.012, -0.13, 0.04];
        let time = i as f64 * 0.005;
        let a = n.sample(p, time).unwrap();
        assert_eq!(a, n.sample(p, time).unwrap());
        assert!(a.iter().map(|x| x * x).sum::<f64>().sqrt() <= n.strength * (1.0 + 1e-12));
        let b = n.sample(p, time + 1e-9).unwrap();
        assert!(a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-6));
    }
    let p = [0.03, 0.04, 0.05];
    let a = n.sample(p, 0.071).unwrap();
    assert_ne!(
        a,
        EmitterTurbulence { seed: 813, ..n }
            .sample(p, 0.071)
            .unwrap()
    );
    assert_ne!(
        a,
        EmitterTurbulence {
            spatial_scale: 0.16,
            ..n
        }
        .sample(p, 0.071)
        .unwrap()
    );
    assert_ne!(
        a,
        EmitterTurbulence {
            time_scale: 0.06,
            ..n
        }
        .sample(p, 0.071)
        .unwrap()
    );
    assert_eq!(
        a,
        EmitterTurbulence {
            spatial_scale: 0.16,
            time_scale: 0.06,
            ..n
        }
        .sample(p.map(|x| x * 2.0), 0.142)
        .unwrap()
    );
    assert_eq!(
        EmitterTurbulence { strength: 0.0, ..n }
            .sample(p, 1e20)
            .unwrap(),
        [0.0; 3]
    );
    assert!(n.sample(p, 1e20).is_err());
    assert!(
        EmitterTurbulence {
            spatial_scale: 0.0,
            ..n
        }
        .sample(p, 0.0)
        .is_err()
    );
}
#[test]
fn vector_rotation_and_motion_are_separate_and_have_correct_units() {
    let e = RotatingCylinderEmitter {
        emitted_velocity: Some([3.0, 4.0, 0.0]),
        motion_velocity: [0.2, 0.0, 0.0],
        rotations_per_second: 1.0,
        ..Default::default()
    };
    let a = e.axis_at(0.25).unwrap();
    assert!(a[0].abs() < 1e-14);
    assert!((a[1] - 0.8).abs() < 1e-14);
    assert!((a[2] - 0.6).abs() < 1e-14);
    assert_eq!(e.center_at(0.25).unwrap(), [0.55, 0.5, 0.5]);
    let v = e.peak_velocity_at(0, 0.25, 1000.0).unwrap();
    assert!((v[0] - 0.2).abs() < 1e-14);
    assert_eq!(v[1], 4.0);
    assert_eq!(v[2], 3.0);
    let b = e.peak_velocity_at(1, 0.25, 1000.0).unwrap();
    for d in 0..3 {
        assert!((v[d] + b[d] - 2.0 * e.motion_velocity[d]).abs() < 1e-14);
    }
    let frame = e.basis_at(0.25).unwrap();
    for i in 0..3 {
        for j in 0..3 {
            let dot: f64 = frame[i].iter().zip(frame[j]).map(|(a, b)| a * b).sum();
            assert!((dot - f64::from(i == j)).abs() < 1e-12);
        }
    }
}
#[test]
fn liquid_and_gas_births_follow_rotated_velocity_vector() {
    for preset in [FluidPreset::Water, FluidPreset::Air] {
        let mut s = ParticleFluidSimulation::new(grid(), config(preset)).unwrap();
        let e = RotatingCylinderEmitter {
            motion_velocity: [0.03, 0.01, -0.02],
            ..emitter()
        };
        let mut found = false;
        for _ in 0..512 {
            let time = s.time();
            let r = s.step(0.01, Some(e), || false).unwrap();
            if !s.particles().is_empty() {
                assert_eq!(
                    s.particles().len(),
                    2,
                    "one separated birth per opposed end"
                );
                let a = e.axis_at(time + 0.5 * r.dt).unwrap();
                for p in s.particles() {
                    let relative: [f64; 3] =
                        std::array::from_fn(|d| p.velocity[d] - e.motion_velocity[d]);
                    let speed = relative.iter().map(|x| x * x).sum::<f64>().sqrt();
                    assert!(speed > 0.0 && speed <= 5.0);
                    let dot: f64 = relative.iter().zip(a).map(|(x, y)| x * y).sum();
                    for (velocity, direction) in relative.iter().zip(a) {
                        assert!((velocity - dot * direction).abs() < 1e-10);
                    }
                }
                found = true;
                break;
            }
        }
        assert!(found, "{} emits material", preset.label());
    }
}
#[test]
fn seeded_particle_replay_rejection_and_retry_preserve_source_state() {
    let cfg = config(FluidPreset::Water);
    let mut a = ParticleFluidSimulation::new(grid(), cfg).unwrap();
    let mut b = ParticleFluidSimulation::new(grid(), cfg).unwrap();
    let e = RotatingCylinderEmitter {
        turbulence: noise(),
        motion_velocity: [0.02, 0.0, 0.0],
        ..emitter()
    };
    for _ in 0..40 {
        let before = a.particles().to_vec();
        let time = a.time();
        assert!(a.step(0.01, Some(e), || true).is_err());
        assert_eq!(a.particles(), before);
        assert_eq!(a.time(), time);
        let ra = a.step(0.01, Some(e), || false).unwrap();
        let rb = b.step(0.01, Some(e), || false).unwrap();
        assert_eq!(ra.dt, rb.dt);
        assert_eq!(a.particles(), b.particles());
        assert_eq!(a.time(), b.time());
        assert!(ra.dt <= e.turbulence.time_scale * 0.25);
    }
    assert!(!a.particles().is_empty());
    let before = a.particles().to_vec();
    let time = a.time();
    for bad in [
        RotatingCylinderEmitter {
            turbulence: EmitterTurbulence {
                time_scale: 0.0,
                ..noise()
            },
            ..e
        },
        RotatingCylinderEmitter {
            motion_velocity: [1e6, 0.0, 0.0],
            ..e
        },
        RotatingCylinderEmitter {
            emitted_velocity: Some([f64::NAN, 0.0, 0.0]),
            ..e
        },
    ] {
        assert!(a.step(0.01, Some(bad), || false).is_err());
        assert_eq!(a.particles(), before);
        assert_eq!(a.time(), time);
    }
    a.step(0.01, Some(e), || false).unwrap();
    b.step(0.01, Some(e), || false).unwrap();
    assert_eq!(a.particles(), b.particles());
}
#[test]
fn zero_perturbation_and_zero_vector_have_no_random_source_effect() {
    let mut a = ParticleFluidSimulation::new(grid(), config(FluidPreset::Water)).unwrap();
    let mut b = ParticleFluidSimulation::new(grid(), config(FluidPreset::Water)).unwrap();
    let e = emitter();
    let other = RotatingCylinderEmitter {
        turbulence: EmitterTurbulence {
            seed: u64::MAX,
            strength: 0.0,
            spatial_scale: 0.031,
            time_scale: 0.001,
        },
        ..e
    };
    for _ in 0..40 {
        let ra = a.step(0.01, Some(e), || false).unwrap();
        let rb = b.step(0.01, Some(other), || false).unwrap();
        assert_eq!(ra.dt, rb.dt);
        assert_eq!(a.particles(), b.particles());
    }
    let mut off = ParticleFluidSimulation::new(grid(), config(FluidPreset::Water)).unwrap();
    for _ in 0..20 {
        let r = off
            .step(
                0.01,
                Some(RotatingCylinderEmitter {
                    emitted_velocity: Some([0.0; 3]),
                    ..e
                }),
                || false,
            )
            .unwrap();
        assert_eq!(r.emitted_mass, 0.0);
        assert!(off.particles().is_empty());
    }
}
#[test]
fn carrier_vector_mode_temporal_bound_and_transactional_refusal() {
    let mut a = carrier();
    let mut b = carrier();
    let e = RotatingCylinderEmitter {
        emitted_velocity: Some([0.0, 0.4, 0.0]),
        rotations_per_second: 0.0,
        turbulence: EmitterTurbulence {
            strength: 0.02,
            time_scale: 0.02,
            ..noise()
        },
        ..Default::default()
    };
    for _ in 0..3 {
        let r = a.step_with_emitter(0.02, e, |_| false).unwrap();
        b.step_with_emitter(0.02, e, |_| false).unwrap();
        assert!(r.dt <= 0.005);
        assert_eq!(a.state().y, b.state().y);
    }
    assert!(a.state().y.iter().any(|x| x.abs() > 1e-6));
    let before = a.state().y.to_vec();
    let time = a.state().time;
    assert!(
        a.step_with_emitter(0.02, e, |s| s == StepStage::BeforeCommit)
            .is_err()
    );
    assert_eq!(a.state().y, before);
    assert_eq!(a.state().time, time);
    let bad = RotatingCylinderEmitter {
        motion_velocity: [1e6, 0.0, 0.0],
        ..e
    };
    assert!(a.step_with_emitter(0.02, bad, |_| false).is_err());
    assert_eq!(a.state().y, before);
}
#[test]
fn material_snapshot_records_controls_and_preserves_them_on_failure() {
    let mut s = MaterialSession::new(
        SessionIdentity {
            run_id: 6,
            source_revision: "authored-emitter-controls".into(),
        },
        grid(),
        config(FluidPreset::Water),
        8,
    )
    .unwrap();
    let controls = MaterialStepControls {
        requested_dt: 0.01,
        emitter: Some(MaterialEmitterControl {
            source_id: 7,
            source_version: 2,
            emitter: RotatingCylinderEmitter {
                turbulence: noise(),
                motion_velocity: [0.01, 0.0, 0.0],
                ..emitter()
            },
        }),
    };
    s.step(controls, None, || false).unwrap();
    assert_eq!(s.snapshot().accepted_controls(), Some(controls));
    assert_eq!(
        s.capabilities().emitter_controls_version,
        EMITTER_CONTROLS_VERSION
    );
    let mut invalid = controls;
    invalid
        .emitter
        .as_mut()
        .unwrap()
        .emitter
        .turbulence
        .strength = f64::NAN;
    assert!(s.step(invalid, None, || false).is_err());
    assert_eq!(s.snapshot().accepted_controls(), Some(controls));
}

#[test]
fn nominal_emission_volume_and_sample_clock_do_not_depend_on_partition() {
    let e = RotatingCylinderEmitter {
        turbulence: noise(),
        rotations_per_second: 0.0,
        ..emitter()
    };
    let mut results = Vec::new();
    for interval in [0.0002_f64, 0.0001] {
        let mut s = ParticleFluidSimulation::new(grid(), config(FluidPreset::Water)).unwrap();
        let mut mass = 0.0;
        while s.time() < 0.01 {
            let r = s
                .step(interval.min(0.01 - s.time()), Some(e), || false)
                .unwrap();
            mass += r.emitted_mass;
        }
        assert_eq!(s.time(), 0.01);
        results.push((
            s.particles().len(),
            mass,
            e.turbulence.sample([0.03, 0.04, 0.05], s.time()).unwrap(),
        ));
    }
    assert!(results[0].0 > 0);
    assert_eq!(results[0], results[1]);
}
