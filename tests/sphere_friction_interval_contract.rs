use rheon::*;

fn body(c: [f64; 3], v: [f64; 3], w: [f64; 3]) -> SphericalRigidMotion {
    let stamp = SurfaceStamp { id: 17, version: 4 };
    let r = 0.125;
    let p = vec![
        [r, 0., 0.],
        [-r, 0., 0.],
        [0., r, 0.],
        [0., -r, 0.],
        [0., 0., r],
        [0., 0., -r],
    ]
    .into_iter()
    .map(|x| [c[0] + x[0], c[1] + x[1], c[2] + x[2]])
    .collect();
    let surface = TriangleSurface::new(
        stamp,
        p,
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
fn corridor_width(width: f64) -> TriangleSurface {
    TriangleSurface::new(
        SurfaceStamp { id: 80, version: 9 },
        vec![
            [-width, -128., -128.],
            [-width, 128., -128.],
            [-width, 0., 128.],
            [width, -128., -128.],
            [width, 128., -128.],
            [width, 0., 128.],
        ],
        vec![[0, 1, 2], [3, 4, 5]],
        SurfaceSettings::default(),
    )
    .unwrap()
}
fn request(
    b: &SphericalRigidMotion,
    s: &TriangleSurface,
    e: f64,
    h: f64,
    budget: usize,
    mu: f64,
) -> SphereFrictionIntervalRequest {
    let b = b.snapshot().body;
    SphereFrictionIntervalRequest {
        coulomb_coefficient: mu,
        interval: SphereIntervalRequest {
            expected_body: b.stamp,
            expected_moving: b.surface,
            expected_static: s.stamp(),
            radius_m: 0.25,
            restitution: e,
            interval_s: h,
            settings: SphereContactSettings::default(),
            max_impacts: budget,
        },
    }
}
fn close(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-12, "{a} != {b}");
}
#[test]
fn repeated_dyadic_capped_slip_updates_spin_and_full_ledgers() {
    let s = corridor_width(0.375);
    let mut b = body([0.; 3], [4., 1., 0.], [0., 0., 0.1]);
    let q = request(&b, &s, 1., 0.2, 3, 1. / 1024.);
    let mut slots = [None; 4];
    let mut frames = Vec::new();
    let r = b
        .advance_static_sphere_friction_interval(
            q,
            &s,
            &mut slots,
            |_, _, _| false,
            |index, owner, segment| {
                assert_eq!(segment.contact.after, owner.snapshot());
                assert_eq!(owner.snapshot().body.stamp.generation, 3 + index as u64);
                frames.push(owner.snapshot());
            },
        )
        .unwrap();
    assert_eq!(r.status, SphereFrictionIntervalStatus::Complete);
    assert_eq!((r.accepted_segments, r.accepted_impacts), (4, 3));
    assert_eq!(r.remaining_interval_s, 0.);
    for (index, entry) in slots[..3].iter().enumerate() {
        let c = entry.unwrap().contact;
        let i = c.impact.unwrap();
        assert_eq!(i.candidate, SphereFrictionCandidate::CoulombCapped);
        close(
            c.coast.requested_dt_s,
            if index == 0 { 0.03125 } else { 0.0625 },
        );
        close(i.tangent_impulse_n_s[1], -1. / 64.);
        close(c.after.body.velocity_m_s[1], 1. - (index + 1) as f64 / 128.);
        close(
            c.after.body.angular_velocity_rad_s[2],
            if index % 2 == 0 { 0.021875 } else { 0.1 },
        );
        close(
            c.coast.rotation_increment_rad,
            c.coast.requested_dt_s * c.before.body.angular_velocity_rad_s[2].abs(),
        );
        assert_eq!(
            c.before,
            if index == 0 {
                r.before
            } else {
                frames[index - 1]
            }
        );
        assert_eq!(entry.unwrap().departure.is_some(), index != 0);
    }
    assert!(slots[3].unwrap().contact.impact.is_none());
    let a = r.accounting.unwrap();
    for v in a
        .momentum_defect_n_s
        .into_iter()
        .chain(a.spin_momentum_defect_n_m_s)
        .chain(a.world_angular_defect_n_m_s)
    {
        close(v, 0.);
    }
    close(a.kinetic_change_j, a.summed_predicted_energy_change_j);
    close(a.kinetic_change_j, a.summed_event_energy_change_j);
    close(a.kinetic_change_j, a.summed_point_midpoint_work_j);
    assert!(a.kinetic_change_j < 0.);
}
#[test]
fn changed_spin_and_rounded_departure_stop_with_actual_prefix() {
    for (width, mu, h, prefix, rotation) in [(1., 0.01, 1., 1, true), (0.375, 0.1, 0.2, 2, false)] {
        let s = corridor_width(width);
        let mut b = body([0.; 3], [4., 1., 0.], [0., 0., 0.1]);
        let q = request(&b, &s, 1., h, 8, mu);
        let mut slots = [None; 9];
        let mut last = None;
        let r = b
            .advance_static_sphere_friction_interval(
                q,
                &s,
                &mut slots,
                |_, _, _| false,
                |_, owner, _| {
                    last = Some((owner.snapshot(), owner.world_surface().vertices().to_vec()))
                },
            )
            .unwrap();
        assert_eq!(r.accepted_segments, prefix);
        assert!(r.remaining_interval_s > 0.);
        assert_eq!(b.snapshot(), last.as_ref().unwrap().0);
        assert_eq!(b.world_surface().vertices(), last.unwrap().1);
        assert!(slots[prefix..].iter().all(Option::is_none));
        if rotation {
            assert_eq!(
                r.status,
                SphereFrictionIntervalStatus::Stopped(SphereFrictionError::Contact(
                    SphereContactError::Motion(RigidMotionError::RotationLimit)
                ))
            );
        } else {
            assert_eq!(
                r.status,
                SphereFrictionIntervalStatus::Stopped(SphereFrictionError::Contact(
                    SphereContactError::InvalidDeparture
                ))
            );
        }
    }
}
#[test]
fn all_cancellation_points_preserve_the_published_friction_prefix() {
    let mut stages = vec![
        SphereFrictionIntervalStage::BetweenSegments,
        SphereFrictionIntervalStage::Contact(SphereFrictionStage::Contact(
            SphereContactStage::Query,
        )),
        SphereFrictionIntervalStage::Contact(SphereFrictionStage::Contact(
            SphereContactStage::Impact,
        )),
        SphereFrictionIntervalStage::Contact(SphereFrictionStage::Friction),
        SphereFrictionIntervalStage::Contact(SphereFrictionStage::Contact(
            SphereContactStage::Publication,
        )),
    ];
    for s in [
        RigidMotionStage::Admission,
        RigidMotionStage::LoadAdmission,
        RigidMotionStage::LoadReduction,
        RigidMotionStage::Kick,
        RigidMotionStage::Drift,
        RigidMotionStage::Geometry,
    ] {
        stages.push(SphereFrictionIntervalStage::Contact(
            SphereFrictionStage::Contact(SphereContactStage::Coast(s)),
        ));
    }
    for stage in stages {
        let s = corridor_width(0.375);
        let mut b = body([0.; 3], [4., 1., 0.], [0., 0., 0.1]);
        let q = request(&b, &s, 1., 0.2, 3, 1. / 1024.);
        let mut slots = [None; 4];
        let mut last = None;
        let r = b
            .advance_static_sphere_friction_interval(
                q,
                &s,
                &mut slots,
                |at, segment, _| segment == 1 && at == stage,
                |_, owner, _| {
                    last = Some((
                        owner.snapshot(),
                        owner.world_surface().vertices().to_vec(),
                        owner.peak_payload_bytes(),
                    ))
                },
            )
            .unwrap();
        assert_eq!(r.accepted_segments, 1, "{stage:?}");
        assert_ne!(r.status, SphereFrictionIntervalStatus::Complete);
        close(r.remaining_interval_s, 0.2 - 0.03125);
        let last = last.unwrap();
        assert_eq!(b.snapshot(), last.0);
        assert_eq!(b.world_surface().vertices(), last.1);
        assert_eq!(b.peak_payload_bytes(), last.2);
        assert!(slots[1..].iter().all(Option::is_none));
    }
}
#[test]
fn budgets_misses_endpoints_and_zero_mu_have_truthful_time() {
    let s = corridor_width(1.);
    for budget in [0, 1, 2] {
        let mut b = body([0.; 3], [4., 0., 0.], [0.; 3]);
        let q = request(&b, &s, 1., 1., budget, 0.);
        let mut slots = [None; 65];
        let r = b
            .advance_static_sphere_friction_interval(
                q,
                &s,
                &mut slots,
                |_, _, _| false,
                |_, _, _| {},
            )
            .unwrap();
        assert_eq!(
            r.status,
            SphereFrictionIntervalStatus::ImpactBudgetExhausted
        );
        assert_eq!(r.accepted_impacts, budget);
        assert!(r.remaining_interval_s > 0.);
    }
    for (v, h, budget, impacts) in [
        ([0.; 3], 1., 0, 0),
        ([4., 0., 0.], 0.1875, 1, 1),
        ([4., 0., 0.], 0.2, 1, 1),
    ] {
        let mut b = body([0.; 3], v, [0.; 3]);
        let mut old = body([0.; 3], v, [0.; 3]);
        let q = request(&b, &s, 1., h, budget, 0.);
        let mut slots = [None; 65];
        let mut normal = [None; 65];
        let r = b
            .advance_static_sphere_friction_interval(
                q,
                &s,
                &mut slots,
                |_, _, _| false,
                |_, _, _| {},
            )
            .unwrap();
        let nr = old
            .advance_static_sphere_interval(
                q.interval,
                &s,
                &mut normal,
                |_, _, _| false,
                |_, _, _| {},
            )
            .unwrap();
        assert_eq!(r.status, SphereFrictionIntervalStatus::Complete);
        assert_eq!(r.accepted_impacts, impacts);
        assert_eq!(r.remaining_interval_s, 0.);
        assert_eq!(r.after, nr.after);
        assert_eq!(b.world_surface().vertices(), old.world_surface().vertices());
    }
}
#[test]
fn admission_and_initial_touch_preserve_owner_records_and_callbacks() {
    let s = corridor_width(1.);
    for (mu, budget, capacity) in [(-1., 1, 2), (0., 65, 65), (0., 2, 2)] {
        let mut b = body([0.; 3], [4., 0., 0.], [0.; 3]);
        let before = b.snapshot();
        let mesh = b.world_surface().vertices().to_vec();
        let q = request(&b, &s, 1., 1., budget, mu);
        let mut slots = vec![None; capacity];
        assert!(
            b.advance_static_sphere_friction_interval(
                q,
                &s,
                &mut slots,
                |_, _, _| panic!("admission callback"),
                |_, _, _| panic!("admission publication")
            )
            .is_err()
        );
        assert_eq!(b.snapshot(), before);
        assert_eq!(b.world_surface().vertices(), mesh);
        assert!(slots.iter().all(Option::is_none));
    }
    for vx in [-1., 0., 1.] {
        let mut b = body([0.75, 0., 0.], [vx, 1., 0.], [0.; 3]);
        let before = b.snapshot();
        let q = request(&b, &s, 1., 1., 4, 0.1);
        let mut slots = [None; 5];
        let r = b
            .advance_static_sphere_friction_interval(
                q,
                &s,
                &mut slots,
                |_, _, _| false,
                |_, _, _| panic!("touch publication"),
            )
            .unwrap();
        assert_eq!(
            r.status,
            SphereFrictionIntervalStatus::Stopped(SphereFrictionError::Contact(
                SphereContactError::InitialContact { triangle: 1 }
            ))
        );
        assert_eq!(r.after, before);
        assert_eq!(r.remaining_interval_s, 1.);
    }
}
