use rheon::*;
fn body(c: [f64; 3], v: [f64; 3], w: [f64; 3], time: f64) -> SphericalRigidMotion {
    let stamp = SurfaceStamp { id: 17, version: 4 };
    let vertices = [[0., 0., 0.], [0.125, 0., 0.], [0., 0.125, 0.]]
        .map(|p| [c[0] + p[0], c[1] + p[1], c[2] + p[2]])
        .to_vec();
    let surface =
        TriangleSurface::new(stamp, vertices, vec![[0, 1, 2]], SurfaceSettings::default()).unwrap();
    let b = FrozenRigidBody::new(RigidSnapshot {
        stamp: RigidStamp {
            id: 7,
            generation: 2,
        },
        surface: stamp,
        center_of_mass: c,
        mass_kg: 2.,
        inertia_kg_m2: [0.05; 3],
        velocity_m_s: v,
        angular_velocity_rad_s: w,
    })
    .unwrap();
    SphericalRigidMotion::new(b, surface, time, RigidMotionSettings::default()).unwrap()
}
fn surface(vertices: Vec<[f64; 3]>, triangles: Vec<[usize; 3]>) -> TriangleSurface {
    TriangleSurface::new(
        SurfaceStamp { id: 80, version: 9 },
        vertices,
        triangles,
        SurfaceSettings::default(),
    )
    .unwrap()
}
fn floor() -> TriangleSurface {
    surface(
        vec![[-8., -8., 0.], [8., -8., 0.], [0., 8., 0.]],
        vec![[0, 1, 2]],
    )
}
fn request(b: &SphericalRigidMotion, s: &TriangleSurface) -> SphereSupportRequest {
    let a = b.snapshot().body;
    SphereSupportRequest {
        expected_body: a.stamp,
        expected_moving: a.surface,
        expected_static: s.stamp(),
        radius_m: 0.25,
        contact_point_m: [0.; 3],
        gravity_m_s2: [0., 0., -9.81],
        equivalent_interval_s: 0.125,
        settings: SphereContactSettings::default(),
    }
}
const ZERO: [[[f64; 3]; 3]; 1] = [[[0.; 3]; 3]];
#[test]
fn gravity_support_is_immutable_then_exactly_stationary_over_repeated_holds() {
    let s = floor();
    let mut b = body([0., 0., 0.25], [0.; 3], [0.; 3], 0.);
    let initial = b.snapshot();
    let vertices = b.world_surface().vertices().to_vec();
    let q = request(&b, &s);
    let f = b
        .stationary_single_face_support(q, &s, SurfaceLoading::Traction(&ZERO), |_, _| false)
        .unwrap();
    assert_eq!(b.snapshot(), initial);
    assert_eq!(f.gravity_force_n, [0., 0., -19.62]);
    assert_eq!(f.support_force_n, [0., 0., 19.62]);
    assert_eq!(f.net_force_n, [0.; 3]);
    for _ in 0..1000 {
        let q = request(&b, &s);
        let r = b
            .advance_stationary_single_face_support(
                q,
                &s,
                SurfaceLoading::Traction(&ZERO),
                |_, _| false,
            )
            .unwrap();
        assert_eq!(r.zero_load_motion.impulse.force_n, [0.; 3]);
        assert_eq!(r.forces.external_work_j, 0.);
        assert_eq!(r.forces.support_work_j, 0.);
        assert_eq!(r.zero_load_motion.clock_defect_s, 0.);
        assert_eq!(
            b.snapshot().body.center_of_mass,
            initial.body.center_of_mass
        );
        assert_eq!(b.snapshot().body.velocity_m_s, [0.; 3]);
        assert_eq!(b.snapshot().body.angular_velocity_rad_s, [0.; 3]);
        assert_eq!(b.snapshot().orientation, initial.orientation);
        assert_eq!(b.world_surface().vertices(), vertices);
    }
    assert_eq!(b.snapshot().time_s, 125.);
    assert_eq!(b.snapshot().body.stamp.generation, 1002);
}
#[test]
fn exact_oblique_reflected_and_neutral_equilibria() {
    for sign in [-1., 1.] {
        let s = surface(
            vec![[-4., 3., -4.], [4., -3., -4.], [0., 0., 4.]],
            vec![[0, 1, 2]],
        );
        let mut b = body([3. * sign, 4. * sign, 0.], [0.; 3], [0.; 3], 0.);
        let mut q = request(&b, &s);
        q.radius_m = 5.;
        q.gravity_m_s2 = [-3. * sign, -4. * sign, 0.];
        let r = b
            .advance_stationary_single_face_support(
                q,
                &s,
                SurfaceLoading::Traction(&ZERO),
                |_, _| false,
            )
            .unwrap();
        assert_eq!(r.forces.support_force_n, [6. * sign, 8. * sign, 0.]);
        assert_eq!(r.forces.support_torque_n_m, [0.; 3]);
    }
    let s = floor();
    let mut b = body([0., 0., 0.25], [0.; 3], [0.; 3], 0.);
    let mut q = request(&b, &s);
    q.gravity_m_s2 = [0.; 3];
    let r = b
        .advance_stationary_single_face_support(q, &s, SurfaceLoading::Traction(&ZERO), |_, _| {
            false
        })
        .unwrap();
    assert_eq!(r.forces.support_magnitude_n, 0.);
}
#[test]
fn unsupported_physics_and_geometry_refuse_without_publication() {
    let s = floor();
    for (g, error) in [
        (
            [1., 0., -9.81],
            SphereSupportError::TangentialLoadUnsupported,
        ),
        ([0., 0., 1.], SphereSupportError::LiftOff),
    ] {
        let mut b = body([0., 0., 0.25], [0.; 3], [0.; 3], 0.);
        let before = b.snapshot();
        let mut q = request(&b, &s);
        q.gravity_m_s2 = g;
        assert_eq!(
            b.advance_stationary_single_face_support(
                q,
                &s,
                SurfaceLoading::Traction(&ZERO),
                |_, _| false
            ),
            Err(error)
        );
        assert_eq!(b.snapshot(), before);
    }
    for (v, w) in [
        ([0., 0., -2_f64.powi(-100)], [0.; 3]),
        ([0.; 3], [0., 1., 0.]),
    ] {
        let mut b = body([0., 0., 0.25], v, w, 0.);
        let q = request(&b, &s);
        let before = b.snapshot();
        assert_eq!(
            b.advance_stationary_single_face_support(
                q,
                &s,
                SurfaceLoading::Traction(&ZERO),
                |_, _| false
            ),
            Err(SphereSupportError::StationaryTwistOnly)
        );
        assert_eq!(b.snapshot(), before);
    }
    let mut b = body([0., 0., 0.25], [0.; 3], [0.; 3], 0.);
    let before = b.snapshot();
    let mut q = request(&b, &s);
    q.radius_m = f64::from_bits(0.25f64.to_bits() + 1);
    assert_eq!(
        b.advance_stationary_single_face_support(q, &s, SurfaceLoading::Traction(&ZERO), |_, _| {
            false
        }),
        Err(SphereSupportError::InvalidRadiusWitness)
    );
    assert_eq!(b.snapshot(), before);
    let multiple = surface(s.vertices().to_vec(), vec![[0, 1, 2], [0, 2, 1]]);
    q = request(&b, &multiple);
    assert_eq!(
        b.advance_stationary_single_face_support(
            q,
            &multiple,
            SurfaceLoading::Traction(&ZERO),
            |_, _| false
        ),
        Err(SphereSupportError::SingleFacetOnly)
    );
    let edge = surface(
        vec![[0., 0., 0.], [8., 0., 0.], [0., 8., 0.]],
        vec![[0, 1, 2]],
    );
    q = request(&b, &edge);
    assert_eq!(
        b.advance_stationary_single_face_support(
            q,
            &edge,
            SurfaceLoading::Traction(&ZERO),
            |_, _| false
        ),
        Err(SphereSupportError::FaceBoundary)
    );
    assert_eq!(b.snapshot(), before);
}
#[test]
fn all_callback_cancellations_are_atomic_and_old_touch_stays_refused() {
    let s = floor();
    let mut stages = vec![];
    let mut b = body([0., 0., 0.25], [0.; 3], [0.; 3], 0.);
    let q = request(&b, &s);
    b.advance_stationary_single_face_support(q, &s, SurfaceLoading::Traction(&ZERO), |stage, i| {
        stages.push((stage, i));
        false
    })
    .unwrap();
    for (target, index) in stages {
        let mut b = body([0., 0., 0.25], [0.; 3], [0.; 3], 0.);
        let before = b.snapshot();
        let mesh = b.world_surface().vertices().to_vec();
        let q = request(&b, &s);
        assert_eq!(
            b.advance_stationary_single_face_support(
                q,
                &s,
                SurfaceLoading::Traction(&ZERO),
                |stage, i| stage == target && i == index
            ),
            Err(SphereSupportError::Cancelled {
                stage: target,
                index
            })
        );
        assert_eq!(b.snapshot(), before);
        assert_eq!(b.world_surface().vertices(), mesh);
    }
    let mut b = body([0., 0., 0.25], [0.; 3], [0.; 3], 0.);
    let a = b.snapshot().body;
    let impact = SphereFrictionRequest {
        expected_body: a.stamp,
        expected_moving: a.surface,
        expected_static: s.stamp(),
        radius_m: 0.25,
        restitution: 0.,
        coulomb_coefficient: 0.,
        interval_s: 0.125,
        settings: SphereContactSettings::default(),
    };
    assert!(
        b.coast_static_sphere_friction(impact, &s, |_, _| false)
            .is_err()
    );
}
#[test]
fn clock_and_impulse_underflow_are_explicit_prepublication_refusals() {
    let s = floor();
    let mut b = body([0., 0., 0.25], [0.; 3], [0.; 3], 1e20);
    let before = b.snapshot();
    let q = request(&b, &s);
    assert_eq!(
        b.advance_stationary_single_face_support(q, &s, SurfaceLoading::Traction(&ZERO), |_, _| {
            false
        }),
        Err(SphereSupportError::Motion(RigidMotionError::ClockAbsorbed))
    );
    assert_eq!(b.snapshot(), before);
    let mut b = body([0., 0., 0.25], [0.; 3], [0.; 3], 0.);
    let before = b.snapshot();
    let mut q = request(&b, &s);
    q.gravity_m_s2 = [0., 0., -2_f64.powi(-100)];
    q.equivalent_interval_s = f64::from_bits(1);
    assert!(
        b.advance_stationary_single_face_support(q, &s, SurfaceLoading::Traction(&ZERO), |_, _| {
            false
        })
        .is_err()
    );
    assert_eq!(b.snapshot(), before);
}
