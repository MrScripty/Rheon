use rheon::{
    Axis, BodyForce, BufferPlan, ForceRegion, ForceUnits, GridGeometry, PressureImplementation,
    PressureSettings, Simulation, SimulationConfig, SimulationError, SmokeSource, StepStage,
};

fn config(density: f64) -> SimulationConfig {
    SimulationConfig {
        density,
        memory_limit: 1024 * 1024,
        pressure: PressureSettings {
            relative_residual: 1e-11,
            absolute_residual: 1e-12,
            divergence_limit: 1e-9,
            max_iterations: 1000,
        },
        actual_divergence_limit: 1e-6,
        max_courant: 1.0,
    }
}
fn force(value: [f64; 3]) -> BodyForce {
    BodyForce {
        value,
        units: ForceUnits::Acceleration,
        region: None,
    }
}
fn snapshot(sim: &Simulation) -> (Vec<u32>, u64, u64) {
    let s = sim.state();
    (
        s.x.iter()
            .chain(s.y)
            .chain(s.z)
            .chain(s.tracer)
            .map(|v| v.to_bits())
            .collect(),
        s.time.to_bits(),
        s.generation,
    )
}
fn cycle_force() -> BodyForce {
    BodyForce {
        value: [2.0, 0.0, 0.0],
        units: ForceUnits::Acceleration,
        region: Some(ForceRegion {
            lower: [0.0, 0.0, 0.0],
            upper: [2.0, 1.0, 1.0],
        }),
    }
}
fn cycle_sim(method: PressureImplementation, density: f64) -> Simulation {
    Simulation::with_implementation(
        GridGeometry::new([2, 2, 1], [1.0; 3], [0.0; 3]).unwrap(),
        config(density),
        method,
    )
    .unwrap()
}

#[test]
fn signed_uniform_forces_on_each_axis_have_hand_computed_work_and_hydrostatic_rest() {
    for method in PressureImplementation::ALL {
        for (d, a) in [0.25, -0.5, 1.5].into_iter().enumerate() {
            let g = GridGeometry::new([3, 4, 2], [0.5, 0.25, 1.0], [-1.0, 2.0, -2.0]).unwrap();
            let expected_faces =
                (g.counts()[d] - 1) * g.counts()[(d + 1) % 3] * g.counts()[(d + 2) % 3];
            let expected_work =
                0.5 * 2.0 * g.cell_volume() * expected_faces as f64 * (0.125_f64 * a).powi(2);
            let mut value = [0.0; 3];
            value[d] = a;
            let mut sim = Simulation::with_implementation(g, config(2.0), method).unwrap();
            let report = sim
                .step_with_forces(0.125, None, &[force(value)], |_| false)
                .unwrap();
            let work = report.forces.unwrap();
            assert_eq!(report.step.dt, 0.125);
            assert_eq!(work.kinetic_energy_before, 0.0);
            assert_eq!(work.kinetic_energy_after, expected_work);
            assert_eq!(work.applied_work, expected_work);
            // A fully filled, stationary sealed box balances a uniform body
            // acceleration with pressure; this is not a free-surface tank.
            assert!(report.step.actual_divergence_max < 1e-6);
            assert!(report.step.kinetic_energy < 1e-16);
        }
    }
}

#[test]
fn localized_force_projects_to_the_independently_derived_four_edge_cycle() {
    // Unit-square cell graph: c=(1,-1,-1,1) spans its divergence-free
    // circulation. Forced u=(1/2,0,0,0); projection c*(u.c)/(c.c)=c/8.
    for method in PressureImplementation::ALL {
        let mut sim = cycle_sim(method, 1.0);
        let report = sim
            .step_with_forces(0.25, None, &[cycle_force()], |_| false)
            .unwrap();
        let g = sim.grid();
        let s = sim.state();
        for (axis, p, expected) in [
            (Axis::X, [1, 0, 0], 0.125),
            (Axis::X, [1, 1, 0], -0.125),
            (Axis::Y, [0, 1, 0], -0.125),
            (Axis::Y, [1, 1, 0], 0.125),
        ] {
            let field = match axis {
                Axis::X => s.x,
                Axis::Y => s.y,
                Axis::Z => s.z,
            };
            assert!((f64::from(field[g.face_index(axis, p).unwrap()]) - expected).abs() < 1e-8);
        }
        assert_eq!(report.forces.unwrap().applied_work, 0.125);
        assert!((report.step.kinetic_energy - 0.03125).abs() < 1e-8);
        assert_eq!(s.tracer, [0.0; 4]);
        assert!(s.z.iter().all(|&v| v == 0.0));
    }
}

#[test]
fn legacy_smoke_force_precedes_external_stage_and_its_work_is_separate() {
    for method in PressureImplementation::ALL {
        let mut sim = cycle_sim(method, 1.0);
        let source = SmokeSource {
            lower: [0.0; 3],
            upper: [1.0, 2.0, 1.0],
            tracer_rate: 0.0,
            vertical_acceleration: 2.0,
        };
        let report = sim
            .step_with_forces(0.25, Some(source), &[cycle_force()], |_| false)
            .unwrap();
        // Smoke supplies the left Y-edge impulse 1/2. External forcing adds
        // the bottom X-edge impulse 1/2. Their circulation cancels: u.c=0.
        let f = report.forces.unwrap();
        assert_eq!(f.kinetic_energy_before, 0.125);
        assert_eq!(f.kinetic_energy_after, 0.25);
        assert_eq!(f.applied_work, 0.125);
        assert!(report.step.kinetic_energy < 1e-16);
        assert_eq!(sim.state().tracer, [0.0; 4]);
    }
}

#[test]
fn force_density_divides_by_constant_density_and_matches_acceleration_bits() {
    for method in PressureImplementation::ALL {
        for density in [0.5, 2.0, 8.0] {
            let mut a = cycle_sim(method, density);
            let mut f = cycle_sim(method, density);
            let mut density_force = cycle_force();
            density_force.units = ForceUnits::ForceDensity;
            density_force.value[0] *= density;
            for _ in 0..3 {
                let ar = a
                    .step_with_forces(0.125, None, &[cycle_force()], |_| false)
                    .unwrap();
                let fr = f
                    .step_with_forces(0.125, None, &[density_force], |_| false)
                    .unwrap();
                assert_eq!(ar.forces, fr.forces);
                assert_eq!(snapshot(&a), snapshot(&f));
            }
        }
    }
}

#[test]
fn support_is_sampled_on_own_face_lattices_and_excludes_upper_planes_and_walls() {
    let mut sim = cycle_sim(PressureImplementation::default(), 1.0);
    let mut f = cycle_force();
    // X faces at x=1 are on the excluded upper plane; Y force is zero.
    f.region.as_mut().unwrap().upper[0] = 1.0;
    let r = sim.step_with_forces(0.25, None, &[f], |_| false).unwrap();
    assert_eq!(r.forces.unwrap().applied_work, 0.0);
    assert_eq!(r.step.kinetic_energy, 0.0);
    for axis in [Axis::X, Axis::Y, Axis::Z] {
        let [nx, ny, nz] = sim.grid().face_counts(axis);
        let field = match axis {
            Axis::X => sim.state().x,
            Axis::Y => sim.state().y,
            Axis::Z => sim.state().z,
        };
        let d = match axis {
            Axis::X => 0,
            Axis::Y => 1,
            Axis::Z => 2,
        };
        for k in 0..nz {
            for j in 0..ny {
                for i in 0..nx {
                    let p = [i, j, k];
                    if p[d] == 0 || p[d] == sim.grid().counts()[d] {
                        assert_eq!(field[sim.grid().face_index(axis, p).unwrap()], 0.0);
                    }
                }
            }
        }
    }
}

#[test]
fn overlapping_vectors_add_and_forcing_budget_covers_each_axis_and_units() {
    for method in PressureImplementation::ALL {
        let mut sim = cycle_sim(method, 2.0);
        let mut opposite = cycle_force();
        opposite.value[0] = -2.0;
        let r = sim
            .step_with_forces(0.125, None, &[cycle_force(), opposite], |_| false)
            .unwrap();
        assert_eq!(r.step.kinetic_energy, 0.0);
        assert_eq!(r.forces.unwrap().applied_work, 0.0);
        let force = BodyForce {
            value: [16.0, -16.0, 16.0],
            units: ForceUnits::ForceDensity,
            region: None,
        };
        let r = sim
            .step_with_forces(1.0, None, &[force], |_| false)
            .unwrap();
        // sum |f/rho|/h = 24. The existing half-budget policy gives sqrt(.5/24).
        assert_eq!(r.step.dt, (0.5_f64 / 24.0).sqrt());
        assert!(r.step.courant <= 1.0);
    }
}

#[test]
fn force_cancellation_and_pressure_exhaustion_preserve_state_and_retry() {
    for method in PressureImplementation::ALL {
        let mut reference = cycle_sim(method, 1.0);
        reference
            .step_with_forces(0.125, None, &[cycle_force()], |_| false)
            .unwrap();
        reference
            .step_with_forces(0.125, None, &[cycle_force()], |_| false)
            .unwrap();
        for stage in [
            StepStage::BeforeForces,
            StepStage::ForceSlice,
            StepStage::BeforePressure,
            StepStage::PressureIteration,
            StepStage::BeforeCommit,
        ] {
            let mut sim = cycle_sim(method, 1.0);
            sim.step_with_forces(0.125, None, &[cycle_force()], |_| false)
                .unwrap();
            let before = snapshot(&sim);
            let mut slices = 0;
            let result = sim.step_with_forces(0.125, None, &[cycle_force()], |s| {
                if s == StepStage::ForceSlice {
                    slices += 1;
                }
                s == stage && (stage != StepStage::ForceSlice || slices == 2)
            });
            assert!(result.is_err(), "{method:?} {stage:?}");
            if matches!(stage, StepStage::BeforeForces | StepStage::ForceSlice) {
                assert_eq!(result.unwrap_err(), SimulationError::Cancelled { stage });
            }
            assert_eq!(snapshot(&sim), before);
            sim.step_with_forces(0.125, None, &[cycle_force()], |_| false)
                .unwrap();
            assert_eq!(snapshot(&sim), snapshot(&reference));
        }
        let mut c = config(1.0);
        c.pressure.max_iterations = 0;
        let mut sim = Simulation::with_implementation(reference.grid().clone(), c, method).unwrap();
        let before = snapshot(&sim);
        assert!(
            sim.step_with_forces(0.125, None, &[cycle_force()], |_| false)
                .is_err()
        );
        assert_eq!(snapshot(&sim), before);
    }
}

#[test]
fn invalid_or_unrepresentable_forces_reject_without_accepted_mutation() {
    let mut sim = cycle_sim(PressureImplementation::default(), 0.5);
    sim.step_with_forces(0.125, None, &[cycle_force()], |_| false)
        .unwrap();
    let before = snapshot(&sim);
    for bad in [
        force([f64::NAN, 0.0, 0.0]),
        force([0.0, f64::INFINITY, 0.0]),
        BodyForce {
            region: Some(ForceRegion {
                lower: [0.0; 3],
                upper: [3.0; 3],
            }),
            ..cycle_force()
        },
        BodyForce {
            region: Some(ForceRegion {
                lower: [0.0; 3],
                upper: [0.0; 3],
            }),
            ..cycle_force()
        },
    ] {
        assert_eq!(
            sim.step_with_forces(0.125, None, &[bad], |_| false)
                .unwrap_err(),
            SimulationError::InvalidForce
        );
        assert_eq!(snapshot(&sim), before);
    }
    for (bad, expected) in [
        (force([f64::MAX, 0.0, 0.0]), SimulationError::TimeResolution),
        (
            BodyForce {
                units: ForceUnits::ForceDensity,
                ..force([f64::MAX, 0.0, 0.0])
            },
            SimulationError::ArithmeticFailure,
        ),
    ] {
        assert_eq!(
            sim.step_with_forces(0.125, None, &[bad], |_| false)
                .unwrap_err(),
            expected
        );
        assert_eq!(snapshot(&sim), before);
    }
    // At time zero, the reduced timestep is representable, but the velocity
    // increment is outside f32 range; reject the candidate without publishing.
    let mut fresh = cycle_sim(PressureImplementation::default(), 0.5);
    let before = snapshot(&fresh);
    assert_eq!(
        fresh
            .step_with_forces(0.125, None, &[force([f64::MAX, 0.0, 0.0])], |_| false)
            .unwrap_err(),
        SimulationError::ArithmeticFailure
    );
    assert_eq!(snapshot(&fresh), before);
}

#[test]
fn empty_forces_keep_legacy_stages_fields_reports_and_array_memory() {
    for method in PressureImplementation::ALL {
        let mut old = cycle_sim(method, 1.0);
        let mut new = cycle_sim(method, 1.0);
        let expected = BufferPlan::for_grid(old.grid(), usize::MAX)
            .unwrap()
            .total_bytes;
        let source = SmokeSource {
            lower: [0.0; 3],
            upper: [1.0, 1.5, 1.0],
            tracer_rate: 0.5,
            vertical_acceleration: 0.25,
        };
        for _ in 0..3 {
            let (mut old_stages, mut new_stages) = (vec![], vec![]);
            let a = old
                .step(0.125, Some(source), |s| {
                    old_stages.push(s);
                    false
                })
                .unwrap();
            let b = new
                .step_with_forces(0.125, Some(source), &[], |s| {
                    new_stages.push(s);
                    false
                })
                .unwrap();
            assert!(b.forces.is_none());
            assert_eq!(old_stages, new_stages);
            assert_eq!(snapshot(&old), snapshot(&new));
            assert_eq!(a.kinetic_energy.to_bits(), b.step.kinetic_energy.to_bits());
            assert_eq!(a.pressure.iterations, b.step.pressure.iterations);
            assert_eq!(old.allocated_bytes(), expected);
            assert_eq!(new.allocated_bytes(), expected);
        }
        new.step_with_forces(0.125, None, &[cycle_force()], |_| false)
            .unwrap();
        assert_eq!(new.allocated_bytes(), expected);
    }
}
