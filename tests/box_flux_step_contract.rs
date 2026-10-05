use rheon::{
    Axis, BodyForce, BoxFluxError, BoxFluxStamp, BoxFluxStepBoundary, BoxFluxStepWorkspace,
    BoxFluxTracerPolicy, BufferPlan, ForceRegion, ForceUnits, GridGeometry, PrescribedBoxFlux,
    PressureImplementation, PressureSettings, Simulation, SimulationConfig, SimulationError,
    SmokeSource, StateView, StepStage,
};
fn config() -> SimulationConfig {
    SimulationConfig {
        density: 2.0,
        memory_limit: 1024 * 1024,
        pressure: PressureSettings {
            relative_residual: 1e-12,
            absolute_residual: 1e-12,
            divergence_limit: 1e-9,
            max_iterations: 1000,
        },
        actual_divergence_limit: 1e-5,
        max_courant: 1.0,
    }
}
fn boundary(axis: usize, speed: f32, version: u64) -> BoxFluxStepBoundary {
    let mut outward = [[0.0; 2]; 3];
    outward[axis] = [-speed, speed];
    BoxFluxStepBoundary {
        flux: PrescribedBoxFlux::new(BoxFluxStamp { id: 73, version }, outward).unwrap(),
        tracer: BoxFluxTracerPolicy::ClampedAppearance,
    }
}
fn workspace(
    g: &GridGeometry,
    c: SimulationConfig,
    method: PressureImplementation,
) -> BoxFluxStepWorkspace {
    BoxFluxStepWorkspace::new(g.clone(), c.density, 1024 * 1024, method).unwrap()
}
fn snapshot(s: StateView<'_>) -> (Vec<u32>, u64, u64) {
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
fn near(a: f64, b: f64) {
    assert!(
        (a - b).abs() < 2e-10 * a.abs().max(b.abs()).max(1.0),
        "{a} != {b}"
    );
}
fn divergence(g: &GridGeometry, s: StateView<'_>) -> f64 {
    let fields = [s.x, s.y, s.z];
    let h = g.spacing();
    let [nx, ny, nz] = g.counts();
    let mut max = 0.0_f64;
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let low = [i, j, k];
                let mut div = 0.0;
                for (d, a) in [Axis::X, Axis::Y, Axis::Z].into_iter().enumerate() {
                    let mut hi = low;
                    hi[d] += 1;
                    div += (f64::from(fields[d][g.face_index(a, hi).unwrap()])
                        - f64::from(fields[d][g.face_index(a, low).unwrap()]))
                        / h[d];
                }
                max = max.max(div.abs());
            }
        }
    }
    max
}
#[test]
fn signed_end_boundary_commits_hand_flow_and_full_step_work_in_all_axes() {
    for method in PressureImplementation::ALL {
        for axis in 0..3 {
            for speed in [-0.25_f32, 0.25] {
                let mut dims = [1; 3];
                dims[axis] = 3;
                let g = GridGeometry::new(dims, [1.0; 3], [0.0; 3]).unwrap();
                let c = config();
                let mut s = Simulation::with_implementation(g.clone(), c, method).unwrap();
                let mut w = workspace(&g, c, method);
                let b = boundary(axis, speed, 4);
                for step in 1..=3 {
                    let r = s
                        .step_with_box_flux(0.5, None, &[], &mut w, b, |_| false)
                        .unwrap();
                    assert_eq!(r.projection.boundary, b.flux.stamp());
                    assert_eq!(r.step.step.generation, step);
                    assert_eq!(r.step.step.time, 0.5 * step as f64);
                    for (d, field) in [s.state().x, s.state().y, s.state().z]
                        .into_iter()
                        .enumerate()
                    {
                        assert!(
                            field
                                .iter()
                                .all(|&v| v == if d == axis { speed } else { 0.0 })
                        );
                    }
                    assert_eq!(divergence(&g, s.state()), 0.0);
                    assert_eq!(r.step.step.actual_divergence_max, 0.0);
                    near(r.work.kinetic_accepted, 0.125);
                    near(r.step.step.kinetic_energy, 0.125);
                    near(r.work.budget_error, 0.0);
                    assert_eq!(r.work.boundary_volume_imbalance, 0.0);
                    if step == 1 {
                        near(r.work.kinetic_old, 0.0);
                        near(r.projection.work.boundary_pressure_work, 0.25);
                        near(r.projection.work.correction_energy, 0.125);
                    } else {
                        near(r.work.kinetic_old, 0.125);
                        near(r.work.advection_change, 0.0);
                        near(r.projection.work.boundary_pressure_work, 0.0);
                    }
                    assert_eq!(r.simulation_array_bytes, s.allocated_bytes());
                    assert_eq!(r.boundary_workspace_array_bytes, w.allocated_bytes());
                }
            }
        }
    }
}
#[test]
fn current_boundary_speed_controls_initial_dt_and_accepted_courant() {
    for method in PressureImplementation::ALL {
        let g = GridGeometry::new([3, 1, 1], [0.5, 1.0, 1.0], [0.0; 3]).unwrap();
        let c = config();
        let mut s = Simulation::with_implementation(g.clone(), c, method).unwrap();
        let mut w = workspace(&g, c, method);
        let r = s
            .step_with_box_flux(20.0, None, &[], &mut w, boundary(0, 2.0, 0), |_| false)
            .unwrap();
        assert_eq!(r.step.step.dt, 0.125);
        assert_eq!(r.step.step.courant, 0.5);
        assert_eq!(s.state().time, 0.125);
    }
}
#[test]
fn copied_changing_boundary_versions_are_published_only_on_success() {
    for method in PressureImplementation::ALL {
        let g = GridGeometry::new([3, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
        let c = config();
        let mut s = Simulation::with_implementation(g.clone(), c, method).unwrap();
        let mut w = workspace(&g, c, method);
        let first = boundary(0, 0.25, 8);
        s.step_with_box_flux(0.5, None, &[], &mut w, first, |_| false)
            .unwrap();
        let before = snapshot(s.state());
        let second = boundary(0, -0.25, 9);
        assert!(
            s.step_with_box_flux(0.5, None, &[], &mut w, second, |stage| stage
                == StepStage::BeforeCommit)
                .is_err()
        );
        assert_eq!(snapshot(s.state()), before);
        let r = s
            .step_with_box_flux(0.5, None, &[], &mut w, second, |_| false)
            .unwrap();
        assert_eq!(r.projection.boundary, second.flux.stamp());
        assert_eq!(first.flux.outward_speeds()[0], [-0.25, 0.25]);
        assert!(s.state().x.iter().all(|&v| v == -0.25));
        near(r.work.kinetic_old, 0.125);
        near(r.work.budget_error, 0.0);
    }
}
#[test]
fn external_and_smoke_force_terms_join_the_projection_work_ledger() {
    for method in PressureImplementation::ALL {
        let g = GridGeometry::new([2, 2, 1], [1.0; 3], [0.0; 3]).unwrap();
        let c = config();
        let mut s = Simulation::with_implementation(g.clone(), c, method).unwrap();
        let mut w = workspace(&g, c, method);
        let source = SmokeSource {
            lower: [0.0; 3],
            upper: [2.0, 2.0, 1.0],
            tracer_rate: 0.5,
            vertical_acceleration: 0.5,
        };
        let force = BodyForce {
            value: [1.0, -0.25, 0.0],
            units: ForceUnits::Acceleration,
            region: None,
        };
        let r = s
            .step_with_box_flux(
                0.25,
                Some(source),
                &[force],
                &mut w,
                boundary(0, 0.25, 0),
                |_| false,
            )
            .unwrap();
        near(r.work.smoke_force_work, 0.03125);
        near(r.work.external_force_work, 0.1015625);
        near(r.work.kinetic_before_projection, 0.1328125);
        near(r.work.kinetic_accepted, 0.125);
        near(
            r.step.forces.unwrap().applied_work,
            r.work.external_force_work,
        );
        near(r.work.tracer_source_change, 0.5);
        near(r.work.tracer_transport_change, 0.0);
        near(r.work.budget_error, 0.0);
        assert_eq!(r.step.step.actual_divergence_max, divergence(&g, s.state()));
        let p = r.projection.work;
        near(
            r.work.kinetic_accepted - r.work.kinetic_old + p.correction_energy,
            r.work.advection_change
                + r.work.smoke_force_work
                + r.work.external_force_work
                + p.boundary_pressure_work
                + p.divergence_residual_work
                + p.correction_residual_work,
        );
    }
}
#[test]
fn clamped_appearance_transport_and_saturated_source_have_separate_integrals() {
    for method in PressureImplementation::ALL {
        let g = GridGeometry::new([3, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
        let c = config();
        let mut s = Simulation::with_implementation(g.clone(), c, method).unwrap();
        let mut w = workspace(&g, c, method);
        let source = SmokeSource {
            lower: [0.0; 3],
            upper: [1.0; 3],
            tracer_rate: 10.0,
            vertical_acceleration: 0.0,
        };
        s.step_with_box_flux(0.5, Some(source), &[], &mut w, boundary(0, 0.0, 0), |_| {
            false
        })
        .unwrap();
        assert_eq!(s.state().tracer, [1.0, 0.0, 0.0]);
        let r = s
            .step_with_box_flux(0.5, None, &[], &mut w, boundary(0, 0.25, 1), |_| false)
            .unwrap();
        assert_eq!(s.state().tracer, [1.0, 0.125, 0.0]);
        near(r.work.tracer_old_integral, 1.0);
        near(r.work.tracer_transport_change, 0.125);
        near(r.work.tracer_source_change, 0.0);
        assert_eq!(r.work.boundary_volume_imbalance, 0.0);
        assert_eq!(r.tracer_policy, BoxFluxTracerPolicy::ClampedAppearance);
    }
}
#[test]
fn cancellation_through_partial_stages_preserves_state_and_retry_matches_fresh() {
    for method in PressureImplementation::ALL {
        for stage in [
            StepStage::BeforeAdvection,
            StepStage::VelocitySlice,
            StepStage::BeforeForces,
            StepStage::ForceSlice,
            StepStage::BeforePressure,
            StepStage::PressureIteration,
            StepStage::ProjectionSlice,
            StepStage::BeforeProjectionAcceptance,
            StepStage::BeforeTracer,
            StepStage::TracerSlice,
            StepStage::BeforeCommit,
        ] {
            let g = GridGeometry::new([3, 2, 2], [1.0; 3], [0.0; 3]).unwrap();
            let c = config();
            let mut s = Simulation::with_implementation(g.clone(), c, method).unwrap();
            let mut w = workspace(&g, c, method);
            let b = boundary(0, 0.25, 0);
            let source = SmokeSource {
                lower: [0.0; 3],
                upper: [1.0, 2.0, 2.0],
                tracer_rate: 0.5,
                vertical_acceleration: 0.125,
            };
            let force = BodyForce {
                value: [0.125, -0.25, 0.125],
                units: ForceUnits::Acceleration,
                region: Some(ForceRegion {
                    lower: [0.0; 3],
                    upper: [2.0; 3],
                }),
            };
            s.step_with_box_flux(0.125, Some(source), &[force], &mut w, b, |_| false)
                .unwrap();
            let before = snapshot(s.state());
            let mut calls = 0;
            assert!(
                s.step_with_box_flux(0.125, Some(source), &[force], &mut w, b, |at| {
                    if at == stage {
                        calls += 1;
                        return calls
                            == if matches!(
                                stage,
                                StepStage::VelocitySlice
                                    | StepStage::ForceSlice
                                    | StepStage::ProjectionSlice
                                    | StepStage::TracerSlice
                            ) {
                                2
                            } else {
                                1
                            };
                    }
                    false
                })
                .is_err(),
                "{method:?} {stage:?}"
            );
            assert!(calls > 0);
            assert_eq!(snapshot(s.state()), before, "{stage:?}");
            let retry = s
                .step_with_box_flux(0.125, Some(source), &[force], &mut w, b, |_| false)
                .unwrap();
            let mut fresh = Simulation::with_implementation(g.clone(), c, method).unwrap();
            let mut fw = workspace(&g, c, method);
            fresh
                .step_with_box_flux(0.125, Some(source), &[force], &mut fw, b, |_| false)
                .unwrap();
            let expected = fresh
                .step_with_box_flux(0.125, Some(source), &[force], &mut fw, b, |_| false)
                .unwrap();
            assert_eq!(snapshot(s.state()), snapshot(fresh.state()));
            assert_eq!(retry.work.budget_error, expected.work.budget_error);
        }
    }
}
#[test]
fn incompatible_flux_solver_exhaustion_and_late_source_overflow_never_publish() {
    for method in PressureImplementation::ALL {
        let g = GridGeometry::new([3, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
        let mut c = config();
        let mut s = Simulation::with_implementation(g.clone(), c, method).unwrap();
        let mut w = workspace(&g, c, method);
        let before = snapshot(s.state());
        let bad = BoxFluxStepBoundary {
            flux: PrescribedBoxFlux::new(
                BoxFluxStamp { id: 73, version: 0 },
                [[0.0, 0.25], [0.0; 2], [0.0; 2]],
            )
            .unwrap(),
            tracer: BoxFluxTracerPolicy::ClampedAppearance,
        };
        assert!(matches!(
            s.step_with_box_flux(0.5, None, &[], &mut w, bad, |_| false),
            Err(SimulationError::BoxFlux(
                BoxFluxError::IncompatibleFlux { .. }
            ))
        ));
        assert_eq!(snapshot(s.state()), before);
        let source = SmokeSource {
            lower: [0.0; 3],
            upper: [3.0, 1.0, 1.0],
            tracer_rate: f64::MAX,
            vertical_acceleration: 0.0,
        };
        assert_eq!(
            s.step_with_box_flux(4.0, Some(source), &[], &mut w, boundary(0, 0.0, 0), |_| {
                false
            })
            .unwrap_err(),
            SimulationError::ArithmeticFailure
        );
        assert_eq!(snapshot(s.state()), before);
        s.step_with_box_flux(0.5, None, &[], &mut w, boundary(0, 0.25, 0), |_| false)
            .unwrap();
        c.pressure.max_iterations = 0;
        let mut exhausted = Simulation::with_implementation(g.clone(), c, method).unwrap();
        let before = snapshot(exhausted.state());
        assert!(matches!(
            exhausted.step_with_box_flux(0.5, None, &[], &mut w, boundary(0, 0.25, 0), |_| false),
            Err(SimulationError::Pressure(_))
        ));
        assert_eq!(snapshot(exhausted.state()), before);
        exhausted
            .step_with_box_flux(0.5, None, &[], &mut w, boundary(0, 0.0, 1), |_| false)
            .unwrap();
    }
}
#[test]
fn workspace_matches_geometry_density_and_solver_before_any_callback() {
    let g = GridGeometry::new([3, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let c = config();
    let mut s =
        Simulation::with_implementation(g.clone(), c, PressureImplementation::ALL[0]).unwrap();
    let before = snapshot(s.state());
    for (other, density, method) in [
        (
            GridGeometry::new([3, 1, 1], [1.0; 3], [1.0, 0.0, 0.0]).unwrap(),
            c.density,
            PressureImplementation::ALL[0],
        ),
        (g.clone(), 1.0, PressureImplementation::ALL[0]),
        (g.clone(), c.density, PressureImplementation::ALL[1]),
    ] {
        let mut w = BoxFluxStepWorkspace::new(other, density, 1024, method).unwrap();
        let mut callbacks = 0;
        assert_eq!(
            s.step_with_box_flux(0.5, None, &[], &mut w, boundary(0, 0.25, 0), |_| {
                callbacks += 1;
                false
            })
            .unwrap_err(),
            SimulationError::BoundaryWorkspaceMismatch
        );
        assert_eq!(callbacks, 0);
        assert_eq!(snapshot(s.state()), before);
    }
}
#[test]
fn independently_capped_workspace_retains_legacy_capacity_and_supports_reset_pause() {
    for method in PressureImplementation::ALL {
        let g = GridGeometry::new([1; 3], [1.0; 3], [0.0; 3]).unwrap();
        let c = config();
        let expected = BufferPlan::for_grid(&g, usize::MAX).unwrap().total_bytes;
        let mut s = Simulation::with_implementation(g.clone(), c, method).unwrap();
        let mut w = BoxFluxStepWorkspace::new(g.clone(), c.density, 80, method).unwrap();
        assert_eq!(w.allocated_bytes(), 80);
        assert_eq!(s.allocated_bytes(), expected);
        assert!(matches!(
            BoxFluxStepWorkspace::new(g, c.density, 79, method),
            Err(SimulationError::BufferLimit {
                required: 80,
                limit: 79
            })
        ));
        let r = s
            .step_with_box_flux(0.5, None, &[], &mut w, boundary(0, 0.25, 0), |_| false)
            .unwrap();
        assert_eq!(r.step.step.kinetic_energy, 0.0);
        assert_eq!(r.projection.pressure.iterations, 0);
        assert_eq!(r.step.step.actual_divergence_max, 0.0);
        let before = snapshot(s.state());
        s.set_paused(true);
        assert_eq!(
            s.step_with_box_flux(0.5, None, &[], &mut w, boundary(0, 0.25, 1), |_| false)
                .unwrap_err(),
            SimulationError::Paused
        );
        assert_eq!(snapshot(s.state()), before);
        s.set_paused(false);
        s.reset().unwrap();
        assert!(s.state().x.iter().all(|&v| v == 0.0));
        assert_eq!(s.state().time, 0.0);
        assert_eq!(s.state().generation, 2);
        s.step_with_box_flux(0.5, None, &[], &mut w, boundary(0, -0.25, 2), |_| false)
            .unwrap();
    }
}
#[test]
fn zero_boundary_agrees_with_legacy_fields_and_default_callbacks_stay_legacy() {
    for method in PressureImplementation::ALL {
        let g = GridGeometry::new([3, 2, 2], [0.5, 1.0, 2.0], [0.0; 3]).unwrap();
        let c = config();
        let mut closed = Simulation::with_implementation(g.clone(), c, method).unwrap();
        let mut zero = Simulation::with_implementation(g.clone(), c, method).unwrap();
        let mut w = workspace(&g, c, method);
        let source = SmokeSource {
            lower: [0.0; 3],
            upper: g.upper(),
            tracer_rate: 0.25,
            vertical_acceleration: 0.5,
        };
        let force = BodyForce {
            value: [0.5, -0.125, 0.25],
            units: ForceUnits::Acceleration,
            region: Some(ForceRegion {
                lower: [0.0; 3],
                upper: [1.0, 1.0, 2.0],
            }),
        };
        for _ in 0..3 {
            let mut legacy_stages = vec![];
            let a = closed
                .step_with_forces(0.125, Some(source), &[force], |s| {
                    legacy_stages.push(s);
                    false
                })
                .unwrap();
            let b = zero
                .step_with_box_flux(
                    0.125,
                    Some(source),
                    &[force],
                    &mut w,
                    boundary(0, 0.0, 0),
                    |_| false,
                )
                .unwrap();
            assert_eq!(snapshot(closed.state()), snapshot(zero.state()));
            assert_eq!(a.step.pressure.iterations, b.step.step.pressure.iterations);
            assert_eq!(
                a.step.actual_divergence_max,
                b.step.step.actual_divergence_max
            );
            assert!(!legacy_stages.contains(&StepStage::ProjectionSlice));
            assert!(!legacy_stages.contains(&StepStage::BeforeProjectionAcceptance));
            near(b.work.budget_error, 0.0);
        }
    }
}

#[test]
fn stored_divergence_rejection_after_projection_preserves_all_accepted_bits() {
    for method in PressureImplementation::ALL {
        let g = GridGeometry::new([3, 2, 2], [0.5, 1.0, 2.0], [0.0; 3]).unwrap();
        let mut c = config();
        c.actual_divergence_limit = 0.0;
        let mut s = Simulation::with_implementation(g.clone(), c, method).unwrap();
        let mut w = workspace(&g, c, method);
        let before = snapshot(s.state());
        let force = BodyForce {
            value: [0.1, -0.13, 0.2],
            units: ForceUnits::Acceleration,
            region: Some(ForceRegion {
                lower: [0.0; 3],
                upper: [1.0, 1.0, 2.0],
            }),
        };
        assert!(
            matches!(s.step_with_box_flux(0.125,None,&[force],&mut w,boundary(0,0.375,0),|_|false),Err(SimulationError::BoxFlux(BoxFluxError::DivergenceLimit {actual,..})) if actual>0.0)
        );
        assert_eq!(snapshot(s.state()), before);
        s.step_with_box_flux(0.125, None, &[], &mut w, boundary(0, 0.0, 1), |_| false)
            .unwrap();
    }
}

#[test]
fn balanced_volume_flux_does_not_certify_tracer_mass_conservation() {
    for method in PressureImplementation::ALL {
        let g = GridGeometry::new([3, 2, 1], [1.0; 3], [0.0; 3]).unwrap();
        let c = config();
        let mut s = Simulation::with_implementation(g.clone(), c, method).unwrap();
        let mut w = workspace(&g, c, method);
        let seed = SmokeSource {
            lower: [0.0; 3],
            upper: [1.0, 2.0, 1.0],
            tracer_rate: 1.0,
            vertical_acceleration: 0.0,
        };
        s.step_with_box_flux(0.5, Some(seed), &[], &mut w, boundary(0, 0.0, 0), |_| false)
            .unwrap();
        let force = BodyForce {
            value: [0.0, 0.5, 0.0],
            units: ForceUnits::Acceleration,
            region: Some(ForceRegion {
                lower: [0.0; 3],
                upper: [1.0, 2.0, 1.0],
            }),
        };
        let r = s
            .step_with_box_flux(0.25, None, &[force], &mut w, boundary(0, 0.25, 1), |_| {
                false
            })
            .unwrap();
        assert_eq!(r.work.boundary_volume_imbalance, 0.0);
        assert_eq!(r.work.tracer_source_change, 0.0);
        assert!(s.state().tracer.iter().all(|v| (0.0..=0.5).contains(v)));
        // A conservative endpoint upwind boundary update with the old adjacent
        // concentration would add dt*U*(0.5+0.5)=0.0625. Midpoint clamped
        // appearance transport has no such boundary-flux balance contract.
        let conservative_reference = 0.0625;
        println!(
            "{method:?} actual_transport_change={:.17e} conservative_reference={conservative_reference}",
            r.work.tracer_transport_change
        );
        assert!((r.work.tracer_transport_change - conservative_reference).abs() > 1e-6);
        near(r.work.budget_error, 0.0);
    }
}
