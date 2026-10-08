use rheon::{
    Axis, GridGeometry, ObstacleFlowError, ObstacleFlowStage, ObstacleShearForce,
    ObstacleShearWall, PressureSettings, StaticObstacleGeometry, StaticObstaclePressure,
    StaticObstacleShear, SurfaceSettings, SurfaceStamp, TriangleSurface,
};
fn mesh(lo: [f64; 3], hi: [f64; 3]) -> TriangleSurface {
    TriangleSurface::new(
        SurfaceStamp { id: 42, version: 7 },
        (0..8)
            .map(|c| std::array::from_fn(|d| if c & (1 << d) == 0 { lo[d] } else { hi[d] }))
            .collect(),
        vec![
            [0, 2, 3],
            [0, 3, 1],
            [4, 5, 7],
            [4, 7, 6],
            [0, 1, 5],
            [0, 5, 4],
            [2, 6, 7],
            [2, 7, 3],
            [0, 4, 6],
            [0, 6, 2],
            [1, 3, 7],
            [1, 7, 5],
        ],
        SurfaceSettings::default(),
    )
    .unwrap()
}
fn owner(n: [u64; 3], h: [f64; 3], lo: [f64; 3], hi: [f64; 3]) -> StaticObstacleGeometry {
    StaticObstacleGeometry::new(
        GridGeometry::new(n, h, [0.0; 3]).unwrap(),
        mesh(lo, hi),
        1_000_000,
        |_, _| false,
    )
    .unwrap()
}
fn settings() -> PressureSettings {
    PressureSettings {
        relative_residual: 1e-12,
        absolute_residual: 1e-12,
        divergence_limit: 1e-10,
        max_iterations: 300,
    }
}
fn zeros(o: &StaticObstacleGeometry) -> [Vec<f64>; 3] {
    std::array::from_fn(|d| vec![0.0; o.grid().face_len([Axis::X, Axis::Y, Axis::Z][d])])
}
fn project(
    p: &mut StaticObstaclePressure<'_>,
    u: &mut [Vec<f64>; 3],
) -> rheon::ObstaclePressureReport {
    let [x, y, z] = u;
    p.project([x, y, z], 0.5, settings(), |_, _| false).unwrap()
}
fn close(a: f64, b: f64, tol: f64) {
    assert!((a - b).abs() <= tol, "{a} != {b}, tolerance {tol}");
}
#[test]
fn two_cell_partial_obstacle_has_independent_matrix_pressure_field_and_energy() {
    let o = owner([2, 1, 1], [1.0; 3], [0.0; 3], [2.0, 0.25, 1.0]);
    let mut p = StaticObstaclePressure::new(&o, 2.0, 1_000_000, |_, _| false).unwrap();
    assert!(std::ptr::eq(p.geometry(), &o));
    assert_eq!(p.gauge_cells(), [0]);
    let mut ap = [0.0; 2];
    p.apply(&[0.0, 4.0], &mut ap).unwrap();
    assert_eq!(ap, [-1.5, 1.5]);
    let mut u = zeros(&o);
    let f = o.grid().face_index(Axis::X, [1, 0, 0]).unwrap();
    u[0][f] = 1.0;
    let report = project(&mut p, &mut u);
    assert_eq!(p.pressure().unwrap(), [0.0, 4.0]);
    assert!(u.iter().flatten().all(|v| *v == 0.0));
    let l = report.corrected.unwrap();
    assert_eq!(l.kinetic_before, 0.75);
    assert_eq!(l.correction_energy, 0.75);
    assert_eq!(l.kinetic_after, 0.0);
    assert_eq!(l.actual_divergence_max, 0.0);
    assert_eq!(o.fluid_volumes(), [0.75, 0.75]);
    assert_eq!(o.stamp(), SurfaceStamp { id: 42, version: 7 });
}
#[test]
fn partial_three_dimensional_gradient_is_removed_with_same_owner() {
    let o = owner([3, 3, 3], [1.0; 3], [0.25, 0.5, 0.75], [1.75, 1.5, 2.25]);
    let mut u = zeros(&o);
    for (d, axis) in [Axis::X, Axis::Y, Axis::Z].into_iter().enumerate() {
        let shape = o.grid().face_counts(axis);
        for f in 0..u[d].len() {
            let c = [
                f % shape[0],
                f / shape[0] % shape[1],
                f / (shape[0] * shape[1]),
            ];
            if c[d] > 0 && c[d] < 3 && o.open_areas(axis)[f] > 0.0 {
                u[d][f] = 0.5 * [1.0, 2.0, 3.0][d] / 2.0;
            }
        }
    }
    let mut p = StaticObstaclePressure::new(&o, 2.0, 1_000_000, |_, _| false).unwrap();
    let r = project(&mut p, &mut u);
    assert!(u.iter().flatten().all(|v| v.abs() < 1e-11));
    assert!(r.predicted_divergence_max < 1e-10);
    assert!(r.corrected.unwrap().actual_divergence_max < 1e-10);
    for (i, &v) in p.pressure().unwrap().iter().enumerate() {
        let expected = (i % 3) as f64 + 2.0 * ((i / 3) % 3) as f64 + 3.0 * (i / 9) as f64;
        close(v, expected, 1e-10);
    }
}
#[test]
fn disconnected_components_have_separate_gauges_and_compatibility() {
    let o = owner([5, 1, 1], [1.0; 3], [2.0, 0.0, 0.0], [3.0, 1.0, 1.0]);
    let mut p = StaticObstaclePressure::new(&o, 1.0, 1_000_000, |_, _| false).unwrap();
    assert_eq!(p.gauge_cells(), [0, 3]);
    assert!(matches!(
        p.solve_rhs(&[1.0, 0.0, 0.0, 0.0, -1.0], 1.0, settings(), |_, _| false),
        Err(ObstacleFlowError::IncompatibleComponent { component: 0, .. })
    ));
    assert!(matches!(
        p.solve_rhs(&[0.0, 0.0, 1.0, 0.0, 0.0], 1.0, settings(), |_, _| false),
        Err(ObstacleFlowError::NonzeroDryRhs { cell: 2 })
    ));
    p.solve_rhs(&[-1.0, 1.0, 0.0, -2.0, 2.0], 1.0, settings(), |_, _| false)
        .unwrap();
    assert_eq!(p.pressure().unwrap(), [0.0, 1.0, 0.0, 0.0, 2.0]);
    let mut out = [0.0; 5];
    p.apply(&[7.0, 7.0, 999.0, -2.0, -2.0], &mut out).unwrap();
    assert_eq!(out, [0.0; 5]);
}
#[test]
fn small_positive_volume_cannot_hide_behind_integrated_residual() {
    let h = 2.0f64.powi(-30);
    let o = owner([2, 1, 1], [1.0; 3], [0.0; 3], [1.0 - h, 1.0, 1.0]);
    assert_eq!(o.fluid_volumes()[0], h);
    let mut p = StaticObstaclePressure::new(&o, 1.0, 1_000_000, |_, _| false).unwrap();
    let loose = PressureSettings {
        absolute_residual: 100.0,
        relative_residual: 0.0,
        divergence_limit: 1.0,
        max_iterations: 0,
    };
    assert!(
        matches!(p.solve_rhs(&[-1.0,1.0],1.0,loose,|_,_|false),Err(ObstacleFlowError::IterationLimit{divergence,..}) if divergence==1.0/h)
    );
    p.solve_rhs(&[-1.0, 1.0], 1.0, settings(), |_, _| false)
        .unwrap();
    assert_eq!(p.pressure().unwrap(), [0.0, 1.0]);
}
#[test]
fn rest_outside_dry_and_singleton_components_are_admitted() {
    for (lo, hi) in [
        ([4.0; 3], [5.0; 3]),
        ([-1.0; 3], [4.0; 3]),
        ([1.0, 0.0, 0.0], [2.0, 1.0, 1.0]),
    ] {
        let o = owner([3, 1, 1], [1.0; 3], lo, hi);
        let mut p = StaticObstaclePressure::new(&o, 1.0, 1_000_000, |_, _| false).unwrap();
        let mut u = zeros(&o);
        let r = project(&mut p, &mut u);
        assert_eq!(r.iterations, 0);
        assert_eq!(r.corrected.unwrap().kinetic_after, 0.0);
    }
}
#[test]
fn projection_failure_and_every_cancellation_stage_preserve_input() {
    let o = owner([2, 1, 1], [1.0; 3], [0.0; 3], [2.0, 0.25, 1.0]);
    let mut p = StaticObstaclePressure::new(&o, 2.0, 1_000_000, |_, _| false).unwrap();
    for stage in [
        ObstacleFlowStage::Solve,
        ObstacleFlowStage::Correction,
        ObstacleFlowStage::Acceptance,
    ] {
        let mut u = zeros(&o);
        u[0][1] = 1.0;
        let old = u.clone();
        let [x, y, z] = &mut u;
        assert!(
            matches!(p.project([x,y,z],0.5,settings(),|s,_|s==stage),Err(ObstacleFlowError::Cancelled{stage:s,..}) if s==stage)
        );
        assert_eq!(u, old);
        assert!(p.pressure().is_none());
    }
    let mut u = zeros(&o);
    u[0][0] = 1.0;
    let old = u.clone();
    let [x, y, z] = &mut u;
    assert!(matches!(
        p.project([x, y, z], 0.5, settings(), |_, _| false),
        Err(ObstacleFlowError::NonzeroWallSpeed { .. })
    ));
    assert_eq!(u, old);
    assert!(matches!(
        StaticObstaclePressure::new(&o, 1.0, 1, |_, _| false),
        Err(ObstacleFlowError::BufferLimit { .. })
    ));
    assert!(matches!(
        StaticObstaclePressure::new(&o, 1.0, 1_000_000, |_, _| true),
        Err(ObstacleFlowError::Cancelled { .. })
    ));
}
#[test]
fn shear_one_cut_layer_matches_hand_backward_euler_force_and_work() {
    let o = owner([1, 2, 1], [1.0; 3], [0.0; 3], [1.0, 1.5, 1.0]);
    let mut s = StaticObstacleShear::new(
        &o,
        Axis::Y,
        2.0,
        1.0,
        ObstacleShearWall::NoSlip,
        1_000_000,
        |_, _| false,
    )
    .unwrap();
    assert!(std::ptr::eq(s.geometry(), &o));
    assert_eq!(s.layer_volumes(), [0.5]);
    assert_eq!(s.layer_centers(), [1.75]);
    assert_eq!(s.wall_conductances(), [4.0, 4.0]);
    let mut u = [1.0];
    let r = s
        .step(&mut u, 0.25, ObstacleShearForce::Density(2.0), |_, _| false)
        .unwrap();
    // M=1, K=8, load=qV=1; (1+dt*8)v=1+dt*1.
    close(u[0], 5.0 / 12.0, 1e-15);
    close(r.body_impulse, 0.25, 0.0);
    close(r.wall_impulse, -5.0 / 6.0, 1e-15);
    assert!(r.energy_identity_error.abs() <= r.energy_rounding_budget);
}
#[test]
fn shear_three_layer_matrix_is_independent_and_density_force_units_differ() {
    let o = owner([1, 4, 1], [1.0; 3], [0.0; 3], [1.0, 1.0, 1.0]);
    for (rho, force, expected) in [
        (
            1.0,
            ObstacleShearForce::Density(1.0),
            [2.0 / 5.0, 3.0 / 5.0, 2.0 / 5.0],
        ),
        (
            2.0,
            ObstacleShearForce::Density(1.0),
            [5.0 / 18.0, 7.0 / 18.0, 5.0 / 18.0],
        ),
        (
            2.0,
            ObstacleShearForce::Acceleration(1.0),
            [5.0 / 9.0, 7.0 / 9.0, 5.0 / 9.0],
        ),
    ] {
        let mut s = StaticObstacleShear::new(
            &o,
            Axis::Y,
            rho,
            1.0,
            ObstacleShearWall::NoSlip,
            1_000_000,
            |_, _| false,
        )
        .unwrap();
        let mut u = [0.0; 3];
        s.step(&mut u, 1.0, force, |_, _| false).unwrap();
        for (a, b) in u.into_iter().zip(expected) {
            close(a, b, 1e-14);
        }
    }
}
#[test]
fn navier_wall_free_slip_and_no_slip_have_actual_distinct_responses() {
    let o = owner([1, 2, 1], [1.0; 3], [0.0; 3], [1.0, 1.5, 1.0]);
    for (wall, k, expected) in [
        (ObstacleShearWall::NoSlip, 4.0, 1.0 / 3.0),
        (ObstacleShearWall::Navier { beta: 4.0 }, 2.0, 0.4),
        (ObstacleShearWall::Navier { beta: 0.0 }, 0.0, 0.5),
    ] {
        let mut s =
            StaticObstacleShear::new(&o, Axis::Y, 2.0, 1.0, wall, 1_000_000, |_, _| false).unwrap();
        assert_eq!(s.wall_conductances()[0], k);
        let mut u = [1.0];
        let r = s
            .step(&mut u, 0.25, ObstacleShearForce::Density(0.0), |_, _| false)
            .unwrap();
        close(u[0], expected, 1e-15);
        assert!(r.kinetic_after < r.kinetic_before);
        assert_eq!(r.body_work, 0.0);
    }
}
#[test]
fn shear_all_orientations_and_nonuniform_cut_layer_use_owner_measures() {
    for d in 0..3 {
        let mut hi = [2.0; 3];
        hi[d] = 0.25;
        let o = owner([2, 2, 2], [1.0; 3], [0.0; 3], hi);
        let s = StaticObstacleShear::new(
            &o,
            [Axis::X, Axis::Y, Axis::Z][d],
            1.0,
            1.0,
            ObstacleShearWall::NoSlip,
            1_000_000,
            |_, _| false,
        )
        .unwrap();
        assert_eq!(s.layer_volumes(), [3.0, 4.0]);
        assert_eq!(s.layer_centers(), [0.625, 1.5]);
        close(s.interior_conductances()[0], 32.0 / 7.0, 1e-15);
        assert_eq!(
            s.layer_volumes().iter().sum::<f64>(),
            o.fluid_volumes().iter().sum::<f64>()
        );
    }
}
#[test]
fn shear_refuses_general_obstacles_and_failures_do_not_publish() {
    let o = owner([2, 2, 2], [1.0; 3], [0.25; 3], [0.75; 3]);
    assert!(matches!(
        StaticObstacleShear::new(
            &o,
            Axis::Y,
            1.0,
            1.0,
            ObstacleShearWall::NoSlip,
            1_000_000,
            |_, _| false
        ),
        Err(ObstacleFlowError::UnsupportedShearGeometry)
    ));
    let o = owner([1, 4, 1], [1.0; 3], [0.0; 3], [1.0, 0.5, 1.0]);
    let mut s = StaticObstacleShear::new(
        &o,
        Axis::Y,
        1.0,
        1.0,
        ObstacleShearWall::NoSlip,
        1_000_000,
        |_, _| false,
    )
    .unwrap();
    for stage in [
        ObstacleFlowStage::Assembly,
        ObstacleFlowStage::Solve,
        ObstacleFlowStage::Correction,
        ObstacleFlowStage::Acceptance,
    ] {
        let mut u = [1.0; 4];
        assert!(
            matches!(s.step(&mut u,0.1,ObstacleShearForce::Density(0.0),|st,_|st==stage),Err(ObstacleFlowError::Cancelled{stage:st,..}) if st==stage)
        );
        assert_eq!(u, [1.0; 4]);
    }
    assert!(matches!(
        StaticObstacleShear::new(
            &o,
            Axis::Y,
            1.0,
            1.0,
            ObstacleShearWall::NoSlip,
            1,
            |_, _| false
        ),
        Err(ObstacleFlowError::BufferLimit { .. })
    ));
}

#[test]
fn translated_anisotropic_pressure_uses_physical_area_and_distance() {
    let g = GridGeometry::new([2, 1, 1], [0.5, 2.0, 4.0], [8.0, -4.0, 16.0]).unwrap();
    let o = StaticObstacleGeometry::new(
        g,
        mesh([8.0, -4.0, 16.0], [9.0, -3.5, 20.0]),
        1_000_000,
        |_, _| false,
    )
    .unwrap();
    assert_eq!(o.fluid_volumes(), [3.0, 3.0]);
    let mut p = StaticObstaclePressure::new(&o, 4.0, 1_000_000, |_, _| false).unwrap();
    let mut ap = [0.0; 2];
    p.apply(&[0.0, 2.0], &mut ap).unwrap();
    // A=6 m², d=.5 m, rho=4 => w=3. Pressure difference2 Pa.
    assert_eq!(ap, [-6.0, 6.0]);
    let mut u = zeros(&o);
    u[0][1] = 0.5;
    let r = project(&mut p, &mut u);
    assert_eq!(p.pressure().unwrap(), [0.0, 2.0]);
    assert_eq!(r.corrected.unwrap().kinetic_before, 1.5);
    assert!(u.iter().flatten().all(|v| *v == 0.0));
}

#[test]
fn invalid_coefficients_and_fields_refuse_without_publishing() {
    let o = owner([2, 1, 1], [1.0; 3], [0.0; 3], [2.0, 0.25, 1.0]);
    for rho in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(matches!(
            StaticObstaclePressure::new(&o, rho, 1_000_000, |_, _| false),
            Err(ObstacleFlowError::InvalidParameter)
        ));
    }
    let mut p = StaticObstaclePressure::new(&o, 1.0, 1_000_000, |_, _| false).unwrap();
    let mut u = zeros(&o);
    u[0][1] = 1.0;
    let original = u.clone();
    let [x, y, z] = &mut u;
    assert!(matches!(
        p.project([x, y, z], 0.0, settings(), |_, _| false),
        Err(ObstacleFlowError::InvalidParameter)
    ));
    assert_eq!(u, original);
    let o = owner([1, 2, 1], [1.0; 3], [0.0; 3], [1.0, 1.5, 1.0]);
    for wall in [
        ObstacleShearWall::Navier { beta: -1.0 },
        ObstacleShearWall::Navier { beta: f64::NAN },
    ] {
        assert!(matches!(
            StaticObstacleShear::new(&o, Axis::Y, 1.0, 1.0, wall, 1_000_000, |_, _| false),
            Err(ObstacleFlowError::InvalidParameter)
        ));
    }
    let mut s = StaticObstacleShear::new(
        &o,
        Axis::Y,
        1.0,
        1.0,
        ObstacleShearWall::NoSlip,
        1_000_000,
        |_, _| false,
    )
    .unwrap();
    let mut u = [1.0];
    assert!(
        s.step(
            &mut u,
            1.0,
            ObstacleShearForce::Density(f64::NAN),
            |_, _| false
        )
        .is_err()
    );
    assert_eq!(u, [1.0]);
}
