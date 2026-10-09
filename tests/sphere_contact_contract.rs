use rheon::*;

fn body(c: [f64; 3], v: [f64; 3], w: [f64; 3]) -> SphericalRigidMotion {
    let stamp = SurfaceStamp { id: 17, version: 4 };
    let r = 0.25;
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
        inertia_kg_m2: [0.2; 3],
        velocity_m_s: v,
        angular_velocity_rad_s: w,
    })
    .unwrap();
    SphericalRigidMotion::new(rigid, surface, 0., RigidMotionSettings::default()).unwrap()
}
fn static_mesh(duplicate: bool) -> TriangleSurface {
    TriangleSurface::new(
        SurfaceStamp { id: 80, version: 9 },
        vec![[0., 0., 0.], [4., 0., 0.], [0., 4., 0.]],
        if duplicate {
            vec![[0, 1, 2], [0, 1, 2]]
        } else {
            vec![[0, 1, 2]]
        },
        SurfaceSettings::default(),
    )
    .unwrap()
}
fn event(
    owner: &mut SphericalRigidMotion,
    surface: &TriangleSurface,
    e: f64,
    h: f64,
    cancel: impl FnMut(SphereContactStage, usize) -> bool,
) -> Result<SphereContactReport, SphereContactError> {
    let b = owner.snapshot().body;
    owner.coast_static_sphere(
        b.stamp,
        b.surface,
        surface,
        surface.stamp(),
        0.5,
        e,
        h,
        SphereContactSettings::default(),
        cancel,
    )
}
fn close(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-12, "{a} != {b}");
}
fn unchanged(owner: &SphericalRigidMotion, before: RigidPoseSnapshot, p: &[[f64; 3]], peak: usize) {
    assert_eq!(owner.snapshot(), before);
    assert_eq!(owner.world_surface().vertices(), p);
    assert_eq!(owner.peak_payload_bytes(), peak);
}

#[test]
fn finite_face_event_stops_and_stores_response() {
    let s = static_mesh(false);
    let mut b = body([1., 1., 2.], [0., 0., -2.], [0.1, 0.2, 0.]);
    let r = event(&mut b, &s, 0.5, 1., |_, _| false).unwrap();
    assert_eq!(r.hit.unwrap().feature, SphereFeature::Face);
    close(b.snapshot().time_s, 0.75);
    close(r.unused_interval_s, 0.25);
    close(b.snapshot().body.center_of_mass[2], 0.5);
    close(b.snapshot().body.velocity_m_s[2], 1.);
    assert_eq!(b.snapshot().body.angular_velocity_rad_s, [0.1, 0.2, 0.]);
    assert_eq!(b.snapshot().body.stamp.generation, 3);
    assert_eq!(b.world_surface().stamp().version, 5);
    close(r.impact.unwrap().predicted_energy_change_j, -3.);
    // No continuation/resting-law substitution on the next call.
    let before = b.snapshot();
    let vertices = b.world_surface().vertices().to_vec();
    let peak = b.peak_payload_bytes();
    assert!(matches!(
        event(&mut b, &s, 0.5, 0.1, |_, _| false),
        Err(SphereContactError::InitialContact { .. })
    ));
    unchanged(&b, before, &vertices, peak);
}
#[test]
fn edge_normal_is_radial_and_cross_component_impulse_is_real() {
    let s = static_mesh(false);
    let mut b = body([1., -0.3, 2.], [0., 0., -2.], [0.; 3]);
    let r = event(&mut b, &s, 0.5, 1., |_, _| false).unwrap();
    assert_eq!(r.hit.unwrap().feature, SphereFeature::Edge(0));
    close(b.snapshot().time_s, 0.8);
    let i = r.impact.unwrap();
    close(i.normal[1], -0.6);
    close(i.normal[2], 0.8);
    close(i.after.velocity_m_s[1], -1.44);
    close(i.after.velocity_m_s[2], -0.08);
}
#[test]
fn finite_vertex_event_differs_from_infinite_plane() {
    let s = static_mesh(false);
    let mut b = body([-0.3, -0.3, 2.], [0., 0., -2.], [0.; 3]);
    let r = event(&mut b, &s, 1., 1., |_, _| false).unwrap();
    assert_eq!(r.hit.unwrap().feature, SphereFeature::Vertex(0));
    close(b.snapshot().time_s, (2. - 0.07_f64.sqrt()) / 2.);
    close(r.impact.unwrap().predicted_energy_change_j, 0.);
}
#[test]
fn backside_and_winding_reverse_preserve_response() {
    for indices in [[0, 1, 2], [0, 2, 1]] {
        let s = TriangleSurface::new(
            SurfaceStamp { id: 80, version: 9 },
            vec![[0., 0., 0.], [4., 0., 0.], [0., 4., 0.]],
            vec![indices],
            SurfaceSettings::default(),
        )
        .unwrap();
        let mut b = body([1., 1., -2.], [0., 0., 2.], [0.; 3]);
        let r = event(&mut b, &s, 0., 1., |_, _| false).unwrap();
        close(r.hit.unwrap().normal[2], -1.);
        close(b.snapshot().body.velocity_m_s[2], 0.);
    }
}
#[test]
fn finite_triangle_miss_and_stationary_center_coast() {
    let s = static_mesh(false);
    for v in [[0., 0., -2.], [0.; 3]] {
        let mut b = body([5., 5., 2.], v, [0., 0., 0.1]);
        let r = event(&mut b, &s, 1., 1., |_, _| false).unwrap();
        assert!(r.hit.is_none());
        close(b.snapshot().time_s, 1.);
        assert_eq!(b.snapshot().body.velocity_m_s, v);
    }
}
#[test]
fn initial_grazing_partition_and_simultaneous_refuse_atomically() {
    for (c, s) in [
        ([1., 1., 0.4], static_mesh(false)),
        ([-0.3, -0.4, 2.], static_mesh(false)),
        ([1., 0., 2.], static_mesh(false)),
        ([1., 1., 2.], static_mesh(true)),
    ] {
        let mut b = body(c, [0., 0., -2.], [0.; 3]);
        let before = b.snapshot();
        let p = b.world_surface().vertices().to_vec();
        let peak = b.peak_payload_bytes();
        assert!(event(&mut b, &s, 0.5, 1., |_, _| false).is_err());
        unchanged(&b, before, &p, peak);
    }
}
#[test]
fn cancellation_at_query_coast_impact_and_publication_is_atomic() {
    let s = static_mesh(false);
    for stage in [
        SphereContactStage::Query,
        SphereContactStage::Coast(RigidMotionStage::Admission),
        SphereContactStage::Coast(RigidMotionStage::LoadAdmission),
        SphereContactStage::Coast(RigidMotionStage::LoadReduction),
        SphereContactStage::Coast(RigidMotionStage::Kick),
        SphereContactStage::Coast(RigidMotionStage::Drift),
        SphereContactStage::Coast(RigidMotionStage::Geometry),
        SphereContactStage::Impact,
        SphereContactStage::Publication,
    ] {
        let mut b = body([1., 1., 2.], [0., 0., -2.], [0.; 3]);
        let before = b.snapshot();
        let p = b.world_surface().vertices().to_vec();
        let peak = b.peak_payload_bytes();
        let mut seen = false;
        assert!(
            event(&mut b, &s, 0.5, 1., |x, _| {
                if x == stage {
                    seen = true;
                    true
                } else {
                    false
                }
            })
            .is_err()
        );
        assert!(seen, "{stage:?}");
        unchanged(&b, before, &p, peak);
    }
}
#[test]
fn stamps_radius_properties_settings_and_duration_refuse() {
    let s = static_mesh(false);
    let mut b = body([1., 1., 2.], [0., 0., -2.], [0.; 3]);
    let before = b.snapshot();
    let stamp = before.body.stamp;
    let moving = before.body.surface;
    assert!(matches!(
        StaticSphereSweep::new(
            &b,
            RigidStamp { id: 8, ..stamp },
            moving,
            &s,
            s.stamp(),
            0.5,
            SphereContactSettings::default()
        ),
        Err(SphereContactError::StaleBody)
    ));
    assert!(matches!(
        StaticSphereSweep::new(
            &b,
            stamp,
            SurfaceStamp { id: 18, ..moving },
            &s,
            s.stamp(),
            0.5,
            SphereContactSettings::default()
        ),
        Err(SphereContactError::StaleMovingSurface)
    ));
    assert!(matches!(
        StaticSphereSweep::new(
            &b,
            stamp,
            moving,
            &s,
            SurfaceStamp {
                id: 81,
                ..s.stamp()
            },
            0.5,
            SphereContactSettings::default()
        ),
        Err(SphereContactError::StaleStaticSurface)
    ));
    for radius in [0., f64::NAN, 0.1] {
        assert!(
            StaticSphereSweep::new(
                &b,
                stamp,
                moving,
                &s,
                s.stamp(),
                radius,
                SphereContactSettings::default()
            )
            .is_err()
        );
    }
    for h in [0., -1., f64::INFINITY] {
        assert!(event(&mut b, &s, 0.5, h, |_, _| false).is_err());
    }
    assert!(event(&mut b, &s, 1.01, 1., |_, _| false).is_err());
    assert_eq!(b.snapshot(), before);
    assert!(
        StaticSphereSweep::new(
            &b,
            stamp,
            moving,
            &s,
            s.stamp(),
            0.5,
            SphereContactSettings {
                static_triangle_limit: 0,
                ..SphereContactSettings::default()
            }
        )
        .is_err()
    );
}

#[test]
fn distinct_normal_simultaneous_contacts_refuse_the_single_impact_model() {
    let s = TriangleSurface::new(
        SurfaceStamp { id: 80, version: 9 },
        vec![[0., 0., 0.], [4., 0., 0.], [0., 4., 0.], [0., 0., 4.]],
        vec![[0, 1, 2], [0, 2, 3]],
        SurfaceSettings::default(),
    )
    .unwrap();
    let mut b = body([2., 1., 2.], [-2., 0., -2.], [0.; 3]);
    let before = b.snapshot();
    let p = b.world_surface().vertices().to_vec();
    let peak = b.peak_payload_bytes();
    assert!(matches!(
        event(&mut b, &s, 0.5, 1., |_, _| false),
        Err(SphereContactError::Simultaneous { .. })
    ));
    unchanged(&b, before, &p, peak);
}

#[test]
fn event_clock_versions_and_late_rotation_refusals_preserve_owner() {
    let s = static_mesh(false);
    for mode in 0..4 {
        let initial = body(
            [1., 1., 2.],
            [0., 0., -2.],
            [0., 0., if mode == 3 { 0.5 } else { 0. }],
        );
        let mut state = initial.snapshot().body;
        if mode == 0 {
            state.stamp.generation = u64::MAX;
        }
        if mode == 1 {
            state.surface.version = u64::MAX;
        }
        let mesh = TriangleSurface::new(
            state.surface,
            initial.world_surface().vertices().to_vec(),
            initial.world_surface().triangles().to_vec(),
            SurfaceSettings::default(),
        )
        .unwrap();
        let mut b = SphericalRigidMotion::new(
            FrozenRigidBody::new(state).unwrap(),
            mesh,
            if mode == 2 { 1e16 } else { 0. },
            RigidMotionSettings::default(),
        )
        .unwrap();
        let before = b.snapshot();
        let p = b.world_surface().vertices().to_vec();
        let peak = b.peak_payload_bytes();
        assert!(event(&mut b, &s, 0.5, 1., |_, _| false).is_err());
        unchanged(&b, before, &p, peak);
    }
}
