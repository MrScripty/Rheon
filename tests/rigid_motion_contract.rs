use rheon::*;
fn cube(stamp: SurfaceStamp, tol: f64) -> TriangleSurface {
    TriangleSurface::new(
        stamp,
        vec![
            [-0.5, -0.5, -0.5],
            [0.5, -0.5, -0.5],
            [0.5, 0.5, -0.5],
            [-0.5, 0.5, -0.5],
            [-0.5, -0.5, 0.5],
            [0.5, -0.5, 0.5],
            [0.5, 0.5, 0.5],
            [-0.5, 0.5, 0.5],
        ],
        vec![
            [0, 2, 1],
            [0, 3, 2],
            [4, 5, 6],
            [4, 6, 7],
            [0, 1, 5],
            [0, 5, 4],
            [3, 7, 6],
            [3, 6, 2],
            [0, 4, 7],
            [0, 7, 3],
            [1, 2, 6],
            [1, 6, 5],
        ],
        SurfaceSettings {
            relative_tolerance: tol,
            ..Default::default()
        },
    )
    .unwrap()
}
fn state() -> RigidSnapshot {
    RigidSnapshot {
        stamp: RigidStamp {
            id: 7,
            generation: 2,
        },
        surface: SurfaceStamp { id: 17, version: 4 },
        center_of_mass: [0.; 3],
        mass_kg: 2.,
        inertia_kg_m2: [1.; 3],
        velocity_m_s: [1., 2., 3.],
        angular_velocity_rad_s: [0.; 3],
    }
}
fn owner(s: RigidSnapshot, time: f64) -> SphericalRigidMotion {
    SphericalRigidMotion::new(
        FrozenRigidBody::new(s).unwrap(),
        cube(s.surface, 2e-10),
        time,
        RigidMotionSettings::default(),
    )
    .unwrap()
}
fn equal(a: f64, b: f64) {
    assert!((a - b).abs() <= 1e-12, "{a} != {b}");
}
fn preserved(
    body: &SphericalRigidMotion,
    before: RigidPoseSnapshot,
    points: &[[f64; 3]],
    peak: usize,
) {
    assert_eq!(body.snapshot(), before);
    assert_eq!(body.world_surface().vertices(), points);
    assert_eq!(body.world_surface().stamp(), before.body.surface);
    assert_eq!(body.peak_payload_bytes(), peak);
}
#[test]
fn zero_spin_coast_updates_actual_mesh_clock_and_stamps() {
    let mut body = owner(state(), 2.);
    let initial = body.snapshot();
    let points = body.world_surface().vertices().to_vec();
    let loads = [[[0.; 3]; 3]; 12];
    let r = body
        .advance(
            initial.body.stamp,
            initial.body.surface,
            SurfaceLoading::Traction(&loads),
            0.125,
            |_, _| false,
        )
        .unwrap();
    assert_eq!(body.snapshot(), r.after);
    assert_eq!(r.after.time_s, 2.125);
    assert_eq!(r.after.orientation, [1., 0., 0., 0.]);
    assert_eq!(r.after.body.center_of_mass, [0.125, 0.25, 0.375]);
    assert_eq!(r.after.body.velocity_m_s, state().velocity_m_s);
    assert_eq!(r.after.body.stamp.generation, 3);
    assert_eq!(r.after.body.surface.version, 5);
    assert_eq!(body.world_surface().relative_tolerance(), 2e-10);
    for (p, old) in body.world_surface().vertices().iter().zip(points) {
        for i in 0..3 {
            assert_eq!(p[i], old[i] + r.after.body.center_of_mass[i]);
        }
    }
    assert_eq!(r.impulse.kinetic_after_j, r.impulse.kinetic_before_j);
    assert_eq!(r.clock_defect_s, 0.);
}
#[test]
fn free_spin_matches_analytic_axis_rotation() {
    let s = RigidSnapshot {
        velocity_m_s: [0.; 3],
        angular_velocity_rad_s: [0., 0., 1.],
        ..state()
    };
    let mut body = owner(s, 0.);
    let loads = [[[0.; 3]; 3]; 12];
    for _ in 0..4 {
        let b = body.snapshot();
        body.advance(
            b.body.stamp,
            b.body.surface,
            SurfaceLoading::Traction(&loads),
            0.125,
            |_, _| false,
        )
        .unwrap();
    }
    let pose = body.snapshot();
    equal(pose.orientation[0], 0.25f64.cos());
    equal(pose.orientation[3], 0.25f64.sin());
    let (sn, cs) = 0.5f64.sin_cos();
    let p = body.world_surface().vertices()[0];
    equal(p[0], -0.5 * cs + 0.5 * sn);
    equal(p[1], -0.5 * sn - 0.5 * cs);
    equal(p[2], -0.5);
    assert_eq!(pose.body.angular_velocity_rad_s, s.angular_velocity_rad_s);
}
#[test]
fn constant_force_has_expected_first_order_position_error() {
    let s = RigidSnapshot {
        velocity_m_s: [0.; 3],
        ..state()
    };
    let loads = [[[1., 0., 0.]; 3]; 12];
    for h in [0.125, 0.0625] {
        let mut body = owner(s, 0.);
        let n = (1. / h) as usize;
        for _ in 0..n {
            let b = body.snapshot();
            let r = body
                .advance(
                    b.body.stamp,
                    b.body.surface,
                    SurfaceLoading::Traction(&loads),
                    h,
                    |_, _| false,
                )
                .unwrap();
            equal(r.impulse.force_n[0], 6.);
            equal(r.impulse.torque_n_m[2], 0.);
            equal(r.impulse.energy_defect_j, 0.);
        }
        equal(body.snapshot().body.velocity_m_s[0], 3.);
        equal(body.snapshot().body.center_of_mass[0], 1.5 * (1. + h));
    }
}
#[test]
fn cancellation_every_stage_preserves_all_owned_fields() {
    let loads = [[[1., 0., 0.]; 3]; 12];
    for phase in [
        RigidMotionStage::Admission,
        RigidMotionStage::LoadAdmission,
        RigidMotionStage::LoadReduction,
        RigidMotionStage::Kick,
        RigidMotionStage::Drift,
        RigidMotionStage::Geometry,
        RigidMotionStage::Publication,
    ] {
        let mut body = owner(state(), 0.);
        let before = body.snapshot();
        let points = body.world_surface().vertices().to_vec();
        let peak = body.peak_payload_bytes();
        let result = body.advance(
            before.body.stamp,
            before.body.surface,
            SurfaceLoading::Traction(&loads),
            0.1,
            |stage, index| stage == phase && (stage != RigidMotionStage::Geometry || index == 2),
        );
        assert!(matches!(result,Err(RigidMotionError::Cancelled{stage,..}) if stage==phase));
        preserved(&body, before, &points, peak);
    }
}
#[test]
fn stale_and_unsupported_duration_rotation_clock_refusals() {
    let mut body = owner(state(), 1.);
    let before = body.snapshot();
    let points = body.world_surface().vertices().to_vec();
    let peak = body.peak_payload_bytes();
    let loads = [[[0.; 3]; 3]; 12];
    for h in [0., -1., 2., f64::NAN, f64::INFINITY] {
        assert!(
            body.advance(
                before.body.stamp,
                before.body.surface,
                SurfaceLoading::Traction(&loads),
                h,
                |_, _| false
            )
            .is_err()
        );
        preserved(&body, before, &points, peak);
    }
    let r = body
        .advance(
            before.body.stamp,
            before.body.surface,
            SurfaceLoading::Traction(&loads),
            0.1,
            |_, _| false,
        )
        .unwrap();
    let mut scanned = false;
    assert_eq!(
        body.advance(
            before.body.stamp,
            r.after.body.surface,
            SurfaceLoading::Traction(&loads),
            0.1,
            |_, _| {
                scanned = true;
                false
            }
        ),
        Err(RigidMotionError::StaleBody)
    );
    assert!(!scanned);
    assert_eq!(
        body.advance(
            r.after.body.stamp,
            before.body.surface,
            SurfaceLoading::Traction(&loads),
            0.1,
            |_, _| false
        ),
        Err(RigidMotionError::StaleSurface)
    );
    let mut fast = owner(
        RigidSnapshot {
            angular_velocity_rad_s: [0., 0., 3.],
            ..state()
        },
        0.,
    );
    let b = fast.snapshot();
    let p = fast.world_surface().vertices().to_vec();
    let peak = fast.peak_payload_bytes();
    assert_eq!(
        fast.advance(
            b.body.stamp,
            b.body.surface,
            SurfaceLoading::Traction(&loads),
            0.1,
            |_, _| false
        ),
        Err(RigidMotionError::RotationLimit)
    );
    preserved(&fast, b, &p, peak);
    let mut clock = owner(state(), 1e20);
    let b = clock.snapshot();
    assert_eq!(
        clock.advance(
            b.body.stamp,
            b.body.surface,
            SurfaceLoading::Traction(&loads),
            0.1,
            |_, _| false
        ),
        Err(RigidMotionError::ClockAbsorbed)
    );
    assert_eq!(clock.snapshot(), b);
}
#[test]
fn late_mesh_collapse_and_stamp_overflow_are_atomic() {
    let mut body = owner(
        RigidSnapshot {
            velocity_m_s: [1e17, 1e17, 1e17],
            ..state()
        },
        0.,
    );
    let before = body.snapshot();
    let points = body.world_surface().vertices().to_vec();
    let peak = body.peak_payload_bytes();
    let loads = [[[0.; 3]; 3]; 12];
    assert!(matches!(
        body.advance(
            before.body.stamp,
            before.body.surface,
            SurfaceLoading::Traction(&loads),
            1.,
            |_, _| false
        ),
        Err(RigidMotionError::Surface(_))
    ));
    preserved(&body, before, &points, peak);
    let s = RigidSnapshot {
        surface: SurfaceStamp {
            version: u64::MAX,
            ..state().surface
        },
        ..state()
    };
    let mut b = owner(s, 0.);
    let before = b.snapshot();
    assert_eq!(
        b.advance(
            s.stamp,
            s.surface,
            SurfaceLoading::Traction(&loads),
            0.1,
            |_, _| false
        ),
        Err(RigidMotionError::SurfaceVersionOverflow)
    );
    assert_eq!(b.snapshot(), before);
}
#[test]
fn admission_requires_spherical_inertia_and_peak_payload() {
    let s = state();
    assert!(matches!(
        SphericalRigidMotion::new(
            FrozenRigidBody::new(RigidSnapshot {
                inertia_kg_m2: [1., 1., 2.],
                ..s
            })
            .unwrap(),
            cube(s.surface, 1e-12),
            0.,
            RigidMotionSettings::default()
        ),
        Err(RigidMotionError::UnsupportedInertia)
    ));
    let surface = cube(s.surface, 1e-12);
    let limit = surface.allocated_bytes() * 2 + surface.vertices().len() * 24 - 1;
    assert!(matches!(
        SphericalRigidMotion::new(
            FrozenRigidBody::new(s).unwrap(),
            surface,
            0.,
            RigidMotionSettings {
                peak_payload_limit_bytes: limit,
                ..Default::default()
            }
        ),
        Err(RigidMotionError::PayloadLimit { .. })
    ));
    assert!(
        SphericalRigidMotion::new(
            FrozenRigidBody::new(s).unwrap(),
            cube(s.surface, 1e-12),
            0.,
            RigidMotionSettings {
                triangle_limit: 11,
                ..Default::default()
            }
        )
        .is_err()
    );
}
