use rheon::*;

fn body(c: [f64; 3], v: [f64; 3], w: [f64; 3]) -> SphericalRigidMotion {
    let stamp = SurfaceStamp { id: 17, version: 4 };
    let r = 0.125;
    let vertices = [
        [r, 0., 0.],
        [-r, 0., 0.],
        [0., r, 0.],
        [0., -r, 0.],
        [0., 0., r],
        [0., 0., -r],
    ]
    .map(|x| [c[0] + x[0], c[1] + x[1], c[2] + x[2]])
    .to_vec();
    let surface = TriangleSurface::new(
        stamp,
        vertices,
        vec![
            [0, 2, 4],
            [2, 1, 4],
            [1, 3, 4],
            [3, 0, 4],
            [2, 0, 5],
            [1, 2, 5],
            [3, 1, 5],
            [0, 3, 5],
        ],
        SurfaceSettings::default(),
    )
    .unwrap();
    let rigid = FrozenRigidBody::new(RigidSnapshot {
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
    SphericalRigidMotion::new(rigid, surface, 0., RigidMotionSettings::default()).unwrap()
}
fn floor() -> TriangleSurface {
    TriangleSurface::new(
        SurfaceStamp { id: 80, version: 9 },
        vec![[-8., -8., 0.], [8., -8., 0.], [0., 8., 0.]],
        vec![[0, 1, 2]],
        SurfaceSettings::default(),
    )
    .unwrap()
}
fn request(
    b: &SphericalRigidMotion,
    s: &TriangleSurface,
    e: f64,
    mu: f64,
    h: f64,
) -> SphereFrictionRequest {
    let body = b.snapshot().body;
    SphereFrictionRequest {
        expected_body: body.stamp,
        expected_moving: body.surface,
        expected_static: s.stamp(),
        radius_m: 0.25,
        restitution: e,
        coulomb_coefficient: mu,
        interval_s: h,
        settings: SphereContactSettings::default(),
    }
}
fn close(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-12, "{a} != {b}");
}
#[test]
fn rational_capped_and_cancelled_slip_include_spin_energy() {
    let s = floor();
    for (mu, mode, vx, wy, loss) in [
        (0.1, SphereFrictionCandidate::CoulombCapped, 1.6, 4., -1.04),
        (
            0.5,
            SphereFrictionCandidate::SlipCancellation,
            10. / 7.,
            40. / 7.,
            -8. / 7.,
        ),
    ] {
        let mut b = body([0., 0., 1.], [2., 0., -2.], [0.; 3]);
        let q = request(&b, &s, 1., mu, 0.5);
        let r = b.coast_static_sphere_friction(q, &s, |_, _| false).unwrap();
        let i = r.impact.unwrap();
        assert_eq!(i.candidate, mode);
        close(r.after.time_s, 0.375);
        close(r.unused_interval_s, 0.125);
        close(i.after.velocity_m_s[0], vx);
        close(i.after.velocity_m_s[2], 2.);
        close(i.after.angular_velocity_rad_s[1], wy);
        close(i.kinetic_after_j - i.kinetic_before_j, loss);
        close(i.energy_defect_j, 0.);
        close(i.work_defect_j, 0.);
        assert_eq!(
            r.normal_proposal.unwrap().after.angular_velocity_rad_s,
            [0.; 3]
        );
        assert_eq!(b.snapshot(), r.after);
        assert_eq!(r.after.body.stamp.generation, 3);
        for x in i
            .momentum_defect_n_s
            .into_iter()
            .chain(i.spin_momentum_defect_n_m_s)
            .chain(i.world_angular_defect_n_m_s)
        {
            close(x, 0.);
        }
    }
}
#[test]
fn spin_can_transfer_energy_to_translation_without_total_gain() {
    let s = floor();
    let mut b = body([0., 0., 0.5], [0., 0., -2.], [0., 1., 0.]);
    let q = request(&b, &s, 1., 0.5, 0.2);
    let r = b.coast_static_sphere_friction(q, &s, |_, _| false).unwrap();
    let i = r.impact.unwrap();
    close(i.after.velocity_m_s[0], 1. / 14.);
    close(i.after.angular_velocity_rad_s[1], 2. / 7.);
    assert!(i.after.velocity_m_s[0] != 0.);
    assert!(i.kinetic_after_j < i.kinetic_before_j);
    close(i.slip_after_m_s, 0.);
    assert_ne!(r.after.orientation, [1., 0., 0., 0.]);
}
#[test]
fn zero_mu_zero_computed_slip_and_mode_neighbors_are_explicit() {
    let s = floor();
    let mut old = body([0., 0., 1.], [2., 0., -2.], [0.; 3]);
    let mut new = body([0., 0., 1.], [2., 0., -2.], [0.; 3]);
    let q = request(&new, &s, 0.5, 0., 0.5);
    let prior = old
        .coast_static_sphere(
            q.expected_body,
            q.expected_moving,
            &s,
            q.expected_static,
            q.radius_m,
            q.restitution,
            q.interval_s,
            q.settings,
            |_, _| false,
        )
        .unwrap();
    let r = new
        .coast_static_sphere_friction(q, &s, |_, _| false)
        .unwrap();
    assert_eq!(prior.after, r.after);
    assert_eq!(
        old.world_surface().vertices(),
        new.world_surface().vertices()
    );
    assert_eq!(
        r.impact.unwrap().candidate,
        SphereFrictionCandidate::NoTangentialImpulse
    );
    // Lower COM position keeps coast spin below the existing PR35 cap.
    let mut b = body([0., 0., 0.5], [1., 0., -4.], [0., 4., 0.]);
    let q = request(&b, &s, 1., 0.5, 0.125);
    let i = b
        .coast_static_sphere_friction(q, &s, |_, _| false)
        .unwrap()
        .impact
        .unwrap();
    assert_eq!(i.candidate, SphereFrictionCandidate::NoTangentialImpulse);
    assert_eq!(i.slip_before_m_s, 0.);
    // Exact representable tie: slip=7, stop=4, normal impulse=8, mu=1/2.
    for (mu, mode) in [
        (0.5_f64.next_down(), SphereFrictionCandidate::CoulombCapped),
        (0.5, SphereFrictionCandidate::SlipCancellation),
        (0.5_f64.next_up(), SphereFrictionCandidate::SlipCancellation),
    ] {
        let mut b = body([0., 0., 1.], [7., 0., -2.], [0.; 3]);
        let q = request(&b, &s, 1., mu, 0.5);
        let i = b
            .coast_static_sphere_friction(q, &s, |_, _| false)
            .unwrap()
            .impact
            .unwrap();
        assert_eq!(i.candidate, mode);
    }
}
#[test]
fn every_cancellation_preserves_full_owner_and_mesh() {
    let s = floor();
    for stage in [
        SphereFrictionStage::Contact(SphereContactStage::Query),
        SphereFrictionStage::Contact(SphereContactStage::Coast(RigidMotionStage::Admission)),
        SphereFrictionStage::Contact(SphereContactStage::Coast(RigidMotionStage::LoadAdmission)),
        SphereFrictionStage::Contact(SphereContactStage::Coast(RigidMotionStage::LoadReduction)),
        SphereFrictionStage::Contact(SphereContactStage::Coast(RigidMotionStage::Kick)),
        SphereFrictionStage::Contact(SphereContactStage::Coast(RigidMotionStage::Drift)),
        SphereFrictionStage::Contact(SphereContactStage::Coast(RigidMotionStage::Geometry)),
        SphereFrictionStage::Contact(SphereContactStage::Impact),
        SphereFrictionStage::Friction,
        SphereFrictionStage::Contact(SphereContactStage::Publication),
    ] {
        let mut b = body([0., 0., 1.], [2., 0., -2.], [0.; 3]);
        let before = b.snapshot();
        let vertices = b.world_surface().vertices().to_vec();
        let payload = b.peak_payload_bytes();
        let q = request(&b, &s, 1., 0.1, 0.5);
        let error = b
            .coast_static_sphere_friction(q, &s, |st, _| st == stage)
            .unwrap_err();
        assert!(matches!(error,SphereFrictionError::Cancelled{stage:st,..} if st==stage));
        assert_eq!(b.snapshot(), before);
        assert_eq!(b.world_surface().vertices(), vertices);
        assert_eq!(b.peak_payload_bytes(), payload);
    }
}
#[test]
fn admission_and_cap_overflow_refuse_atomically_and_touch_is_not_resting() {
    let s = floor();
    for mu in [-1., f64::NAN, f64::INFINITY, f64::MAX] {
        let mut b = body([0., 0., 1.], [2., 0., -2.], [0.; 3]);
        let before = b.snapshot();
        let vertices = b.world_surface().vertices().to_vec();
        let q = request(&b, &s, 1., mu, 0.5);
        let error = b
            .coast_static_sphere_friction(q, &s, |_, _| false)
            .unwrap_err();
        if mu == f64::MAX {
            assert_eq!(
                error,
                SphereFrictionError::Contact(SphereContactError::ArithmeticFailure)
            );
        } else {
            assert_eq!(error, SphereFrictionError::InvalidCoefficient);
        }
        assert_eq!(b.snapshot(), before);
        assert_eq!(b.world_surface().vertices(), vertices);
    }
    for v in [[0., 0., 2.], [1., 0., 0.], [0., 0., -2.]] {
        let mut b = body([0., 0., 0.25], v, [0.; 3]);
        let before = b.snapshot();
        let q = request(&b, &s, 1., 0.5, 0.5);
        assert_eq!(
            b.coast_static_sphere_friction(q, &s, |_, _| false)
                .unwrap_err(),
            SphereFrictionError::Contact(SphereContactError::InitialContact { triangle: 0 })
        );
        assert_eq!(b.snapshot(), before);
    }
    let mut b = body([0., 0., 1.], [2., 0., -2.], [0.; 3]);
    let before = b.snapshot();
    let mut q = request(&b, &s, 1., 0.1, 0.5);
    q.expected_static.version += 1;
    assert_eq!(
        b.coast_static_sphere_friction(q, &s, |_, _| false)
            .unwrap_err(),
        SphereFrictionError::Contact(SphereContactError::StaleStaticSurface)
    );
    assert_eq!(b.snapshot(), before);
}
#[test]
fn miss_and_endpoint_hit_have_distinct_unused_time_contracts() {
    let s = floor();
    let mut b = body([0., 0., 1.], [2., 0., 2.], [0.; 3]);
    let q = request(&b, &s, 1., 0.1, 0.5);
    let r = b.coast_static_sphere_friction(q, &s, |_, _| false).unwrap();
    assert_eq!(r.unused_interval_s, 0.);
    assert!(r.hit.is_none());
    assert!(r.impact.is_none());
    close(r.after.time_s, 0.5);
    let mut b = body([0., 0., 1.], [2., 0., -2.], [0.; 3]);
    let q = request(&b, &s, 1., 0.1, 0.375);
    let r = b.coast_static_sphere_friction(q, &s, |_, _| false).unwrap();
    assert_eq!(r.unused_interval_s, 0.);
    assert!(r.impact.is_some());
    close(r.after.time_s, 0.375);
}
