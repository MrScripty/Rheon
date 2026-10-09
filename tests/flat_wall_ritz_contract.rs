//! Bounded synthetic algebra/source checks. NEVER calls the physical provider solve.
#[path = "../examples/viscous_boundary_wrench.rs"]
#[allow(dead_code)]
mod fixture;
use rheon::*;
fn evidence(id: u64) -> ObstacleEvidenceRef {
    ObstacleEvidenceRef::new(id, 0, [id as u8; 32]).unwrap()
}
fn geometry(n: usize) -> StaticObstacleGeometry {
    fixture::geometry(
        [n as u64; 3],
        [3. / n as f64; 3],
        [0.; 3],
        [n / 3; 3],
        [2 * n / 3; 3],
    )
    .unwrap()
}
fn inputs() -> ObstaclePhysicalInputs {
    ObstaclePhysicalInputs {
        units: ObstacleStateUnits::Si,
        model: ObstacleStateModel::TransientStokes,
        material: ObstacleStateMaterial {
            density: 1.,
            dynamic_viscosity: 1.,
        },
        problem: evidence(1),
        boundary_evidence: evidence(2),
        boundary: ObstacleStateBoundary::StationaryNoSlipSolidSealedFreeSlipOuter,
        initial: ObstacleInitialData::KnownRest(evidence(3)),
        initial_time: 0.,
        forcing: ObstacleForcingHistory {
            kind: ObstacleForcingKind::DeclaredExternalForcing,
            evidence: evidence(4),
            start_time: 0.,
            end_time: 0.,
        },
    }
}
fn zero_force() -> FlatWallPolynomialForce {
    FlatWallPolynomialForce::new(Vec::new(), evidence(4)).unwrap()
}
fn buffers(g: &StaticObstacleGeometry) -> [Vec<f64>; 3] {
    std::array::from_fn(|a| vec![0.; g.grid().face_len([Axis::X, Axis::Y, Axis::Z][a])])
}
fn supplied<'g>(g: &'g StaticObstacleGeometry, v: &[Vec<f64>; 3]) -> ObstacleFlowState<'g> {
    let mut i = inputs();
    i.initial = ObstacleInitialData::Supplied(evidence(5));
    ObstacleFlowState::new(
        g,
        i,
        ObstacleStateFrame {
            time: 0.,
            generation: 0,
            origin: ObstacleStateOrigin::InitialData,
            errors: ObstacleVelocityErrors::unknown(),
        },
        [&v[0], &v[1], &v[2]],
        MAX_OBSTACLE_STATE_BYTES,
    )
    .unwrap()
}
fn close(a: f64, b: f64) {
    assert!(
        (a - b).abs() <= 2e-12 * (1. + a.abs().max(b.abs())),
        "{a} != {b}"
    );
}
#[test]
fn all_basis_roster_support_stored_area_divergence_and_actual_allocations() {
    for (n, nq, rows, blocks) in [(6, 2, 176, 100), (9, 12, 681, 381), (12, 36, 1680, 936)] {
        let g = geometry(n);
        let provider = FlatWallRitzProvider::new(
            &g,
            zero_force(),
            inputs(),
            FLAT_WALL_RITZ_ENVELOPE,
            |_, _| false,
        )
        .unwrap();
        let plan = provider.plan();
        assert_eq!(plan.columns().len(), nq);
        assert_eq!(plan.sites().len(), rows);
        assert_eq!(plan.expected_blocks(), blocks);
        assert_eq!(provider.coverage().selected_rows, rows);
        assert!(provider.coverage().zero_corner_directed_sectors > 0);
        assert!(provider.coverage().outer_even_reflection);
        assert!(matches!(
            provider.solved_velocity(),
            Err(FlatWallRitzError::NotSolved)
        ));
        assert!(matches!(
            provider.acquire_initial_state(evidence(6)),
            Err(FlatWallRitzError::NotSolved)
        ));
        for i in 0..nq {
            let mut q = vec![0.; nq];
            q[i] = 1.;
            let mut v = buffers(&g);
            let [x, y, z] = &mut v;
            plan.apply_flux_curl(&q, [x, y, z]).unwrap();
            for t in plan.columns()[i].terms {
                assert_eq!(t.coefficient.abs(), 1. / g.open_areas(t.component)[t.face]);
            }
            let state = supplied(&g, &v);
            assert!(!state.pressure_available());
            assert_eq!(
                state.qualification(),
                ObstacleStateQualification::Unqualified
            );
            let (observed, bound) = flat_wall_divergence(&g, state.velocity()).unwrap();
            assert!(observed <= bound);
            assert!(bound < 2e-13);
            if n != 9 {
                assert_eq!(observed, 0.);
            }
            flat_wall_owned_traction(&state, FlatWallNormalTraction::FirstRowP1, |_, _| false)
                .unwrap();
        }
        let allocation = provider.allocation();
        assert!(allocation.assembly_peak_managed_bytes < FLAT_WALL_RITZ_ENVELOPE);
        println!(
            "allocation n={n} retained={} assembly={} acquisition={} rows={rows} blocks={blocks}",
            allocation.retained_managed_bytes,
            allocation.assembly_peak_managed_bytes,
            allocation.acquisition_peak_managed_bytes
        );
        assert!(
            FlatWallRitzProvider::new(
                &g,
                zero_force(),
                inputs(),
                allocation.assembly_peak_managed_bytes,
                |_, _| false
            )
            .is_ok()
        );
        assert!(
            FlatWallRitzProvider::new(
                &g,
                zero_force(),
                inputs(),
                allocation.assembly_peak_managed_bytes - 1,
                |_, _| false
            )
            .is_err()
        );
    }
}
#[test]
fn matrix_matches_actual_immutable_stress_action_for_synthetic_q() {
    for n in [6, 9] {
        let g = geometry(n);
        let provider = FlatWallRitzProvider::new(
            &g,
            zero_force(),
            inputs(),
            FLAT_WALL_RITZ_ENVELOPE,
            |_, _| false,
        )
        .unwrap();
        let plan = provider.plan();
        let nq = plan.columns().len();
        let q: Vec<f64> = (0..nq).map(|i| (i as f64 - 3.) / 16.).collect();
        let mut v = buffers(&g);
        let [x, y, z] = &mut v;
        plan.apply_flux_curl(&q, [x, y, z]).unwrap();
        let state = supplied(&g, &v);
        let grad = ObstacleVelocityGradient::new(
            &state,
            plan.sites(),
            MAX_OBSTACLE_STATE_BYTES,
            |_, _| false,
        )
        .unwrap();
        let stress =
            ObstacleViscousStress::new(&grad, MAX_OBSTACLE_STATE_BYTES, |_, _| false).unwrap();
        let mut scratch = vec![0.; plan.sites().len()];
        let mut f = buffers(&g);
        let [x, y, z] = &mut f;
        let w = stress
            .diagnose(&mut scratch, [x, y, z], |_, _| false)
            .unwrap();
        close(w.force_work, -w.dissipation);
        assert_eq!(provider.rhs(), vec![0.; nq]);
        for i in 0..nq {
            let actual: f64 = plan.columns()[i]
                .terms
                .iter()
                .map(|t| t.coefficient * f[t.component as usize][t.face])
                .sum();
            let expected: f64 = (0..nq).map(|j| provider.matrix()[i * nq + j] * q[j]).sum();
            close(actual, -expected);
            for j in 0..nq {
                assert_eq!(provider.matrix()[i * nq + j], provider.matrix()[j * nq + i]);
            }
        }
    }
}
#[test]
fn predicted_coarse_p1_and_normal_only_p2_torque_failures_are_preserved() {
    for n in [6, 12] {
        let g = geometry(n);
        let plan = FlatWallRitzPlan::new(&g, FLAT_WALL_RITZ_ENVELOPE).unwrap();
        let nq = plan.columns().len();
        for i in 0..nq {
            let mut q = vec![0.; nq];
            q[i] = 1. / 16.;
            let mut v = buffers(&g);
            let [x, y, z] = &mut v;
            plan.apply_flux_curl(&q, [x, y, z]).unwrap();
            let s = supplied(&g, &v);
            let p1 = flat_wall_owned_traction(&s, FlatWallNormalTraction::FirstRowP1, |_, _| false)
                .unwrap();
            close(p1.torque[2], (0.5 - 3. / n as f64) * p1.force[0]);
            for a in 0..3 {
                assert!(p1.arithmetic_force[a].contains(p1.force[a]));
                assert!(p1.arithmetic_torque[a].contains(p1.torque[a]));
            }
            if n == 6 {
                assert_eq!(p1.torque[2], 0.);
                let p2 =
                    flat_wall_owned_traction(&s, FlatWallNormalTraction::TwoPlaneP2, |_, _| false)
                        .unwrap();
                assert_eq!(p2.force[0], p1.force[0]);
                close(p2.torque[2], -0.5 * p2.force[0]);
                assert!(p2.force[0] < 0.);
                assert!(p2.torque[2] > 0.);
            }
        }
    }
}
#[test]
fn analytic_polynomial_dual_integration_and_stored_volume_are_distinct() {
    let g = geometry(6);
    let force = FlatWallPolynomialForce::new(
        vec![FlatWallForceTerm {
            component: Axis::X,
            coefficient: 1.,
            powers: [1, 1, 0],
        }],
        evidence(4),
    )
    .unwrap();
    let f = g.grid().face_index(Axis::X, [3, 1, 2]).unwrap();
    let value = force.integrate_face(&g, Axis::X, f).unwrap();
    assert_eq!(value.force, 3. / 64.);
    assert!(value.arithmetic_interval.contains(3. / 64.));
    assert_eq!(value.clipped_geometric_volume, 1. / 8.);
    assert_eq!(value.stored_face_volume, 1. / 8.);
    let face = g.grid().face_index(Axis::X, [2, 1, 2]).unwrap();
    let clipped = force.integrate_face(&g, Axis::X, face).unwrap();
    assert_eq!(clipped.clipped_geometric_volume, 1. / 16.);
    assert_eq!(clipped.stored_face_volume, 1. / 8.);
    assert_eq!(force.integrate_face(&g, Axis::Z, 0).unwrap().force, 0.);
    assert!(
        force
            .integrate_face(&g, Axis::X, g.grid().face_len(Axis::X))
            .is_err()
    );
}
#[test]
fn unsupported_geometry_force_provenance_support_and_shapes_refuse() {
    let wrong = fixture::geometry([6; 3], [1.; 3], [0.; 3], [2; 3], [4; 3]).unwrap();
    assert!(FlatWallRitzPlan::new(&wrong, FLAT_WALL_RITZ_ENVELOPE).is_err());
    for coefficient in [f64::NAN, f64::INFINITY, f64::MIN_POSITIVE / 2.] {
        assert!(
            FlatWallPolynomialForce::new(
                vec![FlatWallForceTerm {
                    component: Axis::X,
                    coefficient,
                    powers: [0; 3]
                }],
                evidence(4)
            )
            .is_err()
        );
    }
    assert!(
        FlatWallPolynomialForce::new(
            vec![FlatWallForceTerm {
                component: Axis::X,
                coefficient: 1.,
                powers: [17, 0, 0]
            }],
            evidence(4)
        )
        .is_err()
    );
    let g = geometry(6);
    let mut bad = inputs();
    bad.forcing.evidence = evidence(8);
    assert!(
        FlatWallRitzProvider::new(&g, zero_force(), bad, FLAT_WALL_RITZ_ENVELOPE, |_, _| false)
            .is_err()
    );
    for limit in [0, FLAT_WALL_RITZ_ENVELOPE + 1] {
        assert!(FlatWallRitzPlan::new(&g, limit).is_err());
    }
    let plan = FlatWallRitzPlan::new(&g, FLAT_WALL_RITZ_ENVELOPE).unwrap();
    let mut v = buffers(&g);
    let [x, y, z] = &mut v;
    assert!(plan.apply_flux_curl(&[], [x, y, z]).is_err());
    let outside = g.grid().face_index(Axis::X, [1, 0, 0]).unwrap();
    v[0][outside] = 1.;
    let s = supplied(&g, &v);
    assert!(matches!(
        flat_wall_owned_traction(&s, FlatWallNormalTraction::FirstRowP1, |_, _| false),
        Err(FlatWallRitzError::CoverageMismatch)
    ));
}
#[test]
fn coverage_and_traction_cancellation_leave_owned_snapshot_unchanged() {
    let g = geometry(6);
    let plan = FlatWallRitzPlan::new(&g, FLAT_WALL_RITZ_ENVELOPE).unwrap();
    let v = buffers(&g);
    let s = supplied(&g, &v);
    let mut before = Vec::new();
    s.write_checkpoint(&mut before).unwrap();
    assert!(plan.verify_coverage(&s, |_, _| true).is_err());
    assert!(flat_wall_owned_traction(&s, FlatWallNormalTraction::TwoPlaneP2, |_, _| true).is_err());
    let mut after = Vec::new();
    s.write_checkpoint(&mut after).unwrap();
    assert_eq!(before, after);
    assert!(
        FlatWallRitzProvider::new(
            &g,
            zero_force(),
            inputs(),
            FLAT_WALL_RITZ_ENVELOPE,
            |_, _| true
        )
        .is_err()
    );
}
