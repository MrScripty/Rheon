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
fn corridor() -> TriangleSurface {
    TriangleSurface::new(
        SurfaceStamp { id: 80, version: 9 },
        vec![
            [-1., -4., -4.],
            [-1., 4., -4.],
            [-1., 0., 4.],
            [1., -4., -4.],
            [1., 4., -4.],
            [1., 0., 4.],
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
) -> SphereIntervalRequest {
    let b = b.snapshot().body;
    SphereIntervalRequest {
        expected_body: b.stamp,
        expected_moving: b.surface,
        expected_static: s.stamp(),
        radius_m: 0.25,
        restitution: e,
        interval_s: h,
        settings: SphereContactSettings::default(),
        max_impacts: budget,
    }
}
fn close(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-12, "{a} != {b}");
}
#[test]
fn repeated_elastic_and_damped_corridor() {
    for (e, n, x, v, loss) in [
        (1., 3, 0.5, -4., 0.),
        (0.5, 2, -0.6875, 1., -15.),
        (0., 1, 0.75, 0., -16.),
    ] {
        let s = corridor();
        let mut b = body([0.; 3], [4., 0., 0.], [0., 0., 0.1]);
        let q = request(&b, &s, e, 1., n);
        let mut slots = [None; 65];
        let mut frames = Vec::new();
        let r = b
            .advance_static_sphere_interval(
                q,
                &s,
                &mut slots,
                |_, _, _| false,
                |_, owner, _| frames.push(owner.snapshot()),
            )
            .unwrap();
        assert_eq!(r.status, SphereIntervalStatus::Complete);
        assert_eq!(r.remaining_interval_s, 0.);
        assert_eq!(r.accepted_impacts, n);
        assert_eq!(r.accepted_segments, n + 1);
        assert_eq!(frames.len(), n + 1);
        close(r.after.body.center_of_mass[0], x);
        close(r.after.body.velocity_m_s[0], v);
        close(r.after.time_s, 1.);
        assert_eq!(r.after.body.stamp.generation, 2 + (n + 1) as u64);
        assert_eq!(r.after.body.angular_velocity_rad_s, [0., 0., 0.1]);
        close(r.accounting.unwrap().kinetic_change_j, loss);
        close(r.accounting.unwrap().energy_defect_j, 0.);
        if e == 0. {
            assert_eq!(
                slots[1].unwrap().departure.unwrap().kind,
                SphereDepartureKind::Tangent
            );
        }
        assert!(slots[n + 1..].iter().all(Option::is_none));
    }
}
#[test]
fn budget_stops_before_next_hit_but_allows_terminal_miss() {
    let s = corridor();
    for budget in [0, 1, 2] {
        let mut b = body([0.; 3], [4., 0., 0.], [0.; 3]);
        let q = request(&b, &s, 1., 1., budget);
        let mut slots = [None; 65];
        let r = b
            .advance_static_sphere_interval(q, &s, &mut slots, |_, _, _| false, |_, _, _| {})
            .unwrap();
        assert_eq!(r.status, SphereIntervalStatus::ImpactBudgetExhausted);
        assert_eq!(r.accepted_impacts, budget);
        assert_eq!(r.accepted_segments, budget);
        assert!(r.remaining_interval_s > 0.);
        close(
            r.after.time_s,
            if budget == 0 {
                0.
            } else {
                0.1875 + 0.375 * (budget - 1) as f64
            },
        );
    }
    let mut b = body([0.; 3], [0.; 3], [0.; 3]);
    let q = request(&b, &s, 1., 1., 0);
    let mut slots = [None];
    let r = b
        .advance_static_sphere_interval(q, &s, &mut slots, |_, _, _| false, |_, _, _| {})
        .unwrap();
    assert_eq!(r.status, SphereIntervalStatus::Complete);
    assert_eq!(r.accepted_segments, 1);
}
#[test]
fn initial_touch_separating_tangent_and_inward_are_explicit_stops() {
    let s = corridor();
    for v in [[-1., 0., 0.], [0., 1., 0.], [1., 0., 0.]] {
        let mut b = body([0.75, 0., 0.], v, [0.; 3]);
        let before = b.snapshot();
        let p = b.world_surface().vertices().to_vec();
        let q = request(&b, &s, 1., 1., 4);
        let mut slots = [None; 5];
        let r = b
            .advance_static_sphere_interval(
                q,
                &s,
                &mut slots,
                |_, _, _| false,
                |_, _, _| panic!("unexpected publication"),
            )
            .unwrap();
        assert!(matches!(
            r.status,
            SphereIntervalStatus::Stopped(SphereContactError::InitialContact { .. })
        ));
        assert_eq!(r.remaining_interval_s, 1.);
        assert_eq!(r.accepted_segments, 0);
        assert_eq!(b.snapshot(), before);
        assert_eq!(b.world_surface().vertices(), p);
        assert!(slots.iter().all(Option::is_none));
    }
}
#[test]
fn cancellation_and_later_motion_refusal_retain_the_accepted_prefix() {
    let s = corridor();
    for stage in [
        SphereIntervalStage::BetweenSegments,
        SphereIntervalStage::Contact(SphereContactStage::Query),
        SphereIntervalStage::Contact(SphereContactStage::Publication),
    ] {
        let mut b = body([0.; 3], [4., 0., 0.], [0.; 3]);
        let q = request(&b, &s, 1., 1., 4);
        let mut slots = [None; 5];
        let mut last = None;
        let r = b
            .advance_static_sphere_interval(
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
        assert_ne!(r.status, SphereIntervalStatus::Complete);
        assert_eq!(r.accepted_segments, 1);
        assert_eq!(r.remaining_interval_s, 0.8125);
        let last = last.unwrap();
        assert_eq!(b.snapshot(), last.0);
        assert_eq!(b.world_surface().vertices(), last.1);
        assert_eq!(b.peak_payload_bytes(), last.2);
        assert!(slots[1..].iter().all(Option::is_none));
    }
    let mut b = body([0.; 3], [4., 0., 0.], [0., 0., 1.]);
    let q = request(&b, &s, 1., 1., 4);
    let mut slots = [None; 5];
    let r = b
        .advance_static_sphere_interval(q, &s, &mut slots, |_, _, _| false, |_, _, _| {})
        .unwrap();
    assert_eq!(r.accepted_segments, 1);
    assert!(matches!(
        r.status,
        SphereIntervalStatus::Stopped(SphereContactError::Motion(_))
    ));
    assert_eq!(r.remaining_interval_s, 0.8125);
}
#[test]
fn admission_errors_and_end_exact_impact() {
    let s = corridor();
    let mut b = body([0.; 3], [4., 0., 0.], [0.; 3]);
    let before = b.snapshot();
    let mut slots = [None; 65];
    let q = request(&b, &s, 1., 1., 65);
    assert_eq!(
        b.advance_static_sphere_interval(q, &s, &mut slots, |_, _, _| false, |_, _, _| {}),
        Err(SphereIntervalAdmissionError::InvalidRequest)
    );
    let q = request(&b, &s, 1., 1., 2);
    assert_eq!(
        b.advance_static_sphere_interval(q, &s, &mut slots[..2], |_, _, _| false, |_, _, _| {}),
        Err(SphereIntervalAdmissionError::RecordCapacity)
    );
    assert_eq!(b.snapshot(), before);
    let q = request(&b, &s, 1., 0.1875, 1);
    let r = b
        .advance_static_sphere_interval(q, &s, &mut slots, |_, _, _| false, |_, _, _| {})
        .unwrap();
    assert_eq!(r.status, SphereIntervalStatus::Complete);
    assert_eq!(r.accepted_segments, 1);
    assert_eq!(r.remaining_interval_s, 0.);
}
#[test]
fn simultaneous_boundary_does_not_publish() {
    let s = TriangleSurface::new(
        SurfaceStamp { id: 80, version: 9 },
        vec![[1., -4., -4.], [1., 4., -4.], [1., 0., 4.]],
        vec![[0, 1, 2], [0, 1, 2]],
        SurfaceSettings::default(),
    )
    .unwrap();
    let mut b = body([0.; 3], [4., 0., 0.], [0.; 3]);
    let before = b.snapshot();
    let q = request(&b, &s, 1., 1., 4);
    let mut slots = [None; 5];
    let r = b
        .advance_static_sphere_interval(q, &s, &mut slots, |_, _, _| false, |_, _, _| {})
        .unwrap();
    assert!(matches!(
        r.status,
        SphereIntervalStatus::Stopped(SphereContactError::Simultaneous { .. })
    ));
    assert_eq!(b.snapshot(), before);
    assert_eq!(r.remaining_interval_s, 1.);
}
