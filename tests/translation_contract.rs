use rheon::{
    HitFacing, SurfaceError, SurfaceSettings, SurfaceStamp, TranslatedHit, TranslationError,
    TranslationInterval, TriangleSurface,
};

fn mesh() -> TriangleSurface {
    TriangleSurface::new(
        SurfaceStamp { id: 87, version: 3 },
        vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        vec![[0, 1, 2]],
        SurfaceSettings::default(),
    )
    .unwrap()
}
fn near(a: f64, b: f64) {
    assert!(
        (a - b).abs() < 4e-14 * a.abs().max(b.abs()).max(1.0),
        "{a} != {b}"
    );
}
fn reconstruct(interval: &TranslationInterval<'_>, hit: TranslatedHit) {
    let h = hit.reference_hit;
    let indices = interval.surface().triangles()[h.triangle];
    let t = h.parameter;
    for d in 0..3 {
        let p: f64 = (0..3)
            .map(|i| h.barycentric[i] * interval.surface().vertices()[indices[i]][d])
            .sum();
        near(p, h.position[d]);
        let offsets = interval.offsets();
        near(
            p + (1.0 - t) * offsets[0][d] + t * offsets[1][d],
            hit.world_position[d],
        );
    }
}

#[test]
fn moving_wall_crosses_stationary_world_point_at_physical_time() {
    let surface = mesh();
    let interval = TranslationInterval::new(
        &surface,
        101,
        [4.0, 6.0],
        [[0.0, 0.0, -1.0], [0.0, 0.0, 1.0]],
    )
    .unwrap();
    let p = [0.25, 0.25, 0.0];
    let hit = interval.first_hit(p, p, |_| false).unwrap().unwrap();
    assert_eq!(hit.reference_hit.parameter, 0.5);
    assert_eq!(hit.time, 5.0);
    assert_eq!(hit.wall_velocity, [0.0, 0.0, 1.0]);
    assert_eq!(hit.world_position, p);
    assert_eq!(hit.interval_id, 101);
    assert_eq!(hit.reference_hit.surface, surface.stamp());
    assert_eq!(hit.reference_hit.facing, HitFacing::Front);
    reconstruct(&interval, hit);
}

#[test]
fn final_pose_query_misses_an_actual_space_time_crossing() {
    let surface = mesh();
    let interval = TranslationInterval::new(
        &surface,
        102,
        [0.0, 1.0],
        [[0.0, 0.0, -1.0], [0.0, 0.0, 1.0]],
    )
    .unwrap();
    let (start, end) = ([0.25, 0.25, 0.2], [0.25, 0.25, 0.4]);
    let hit = interval.first_hit(start, end, |_| false).unwrap().unwrap();
    near(hit.reference_hit.parameter, 2.0 / 3.0);
    near(hit.world_position[2], 1.0 / 3.0);
    // Sampling only the end pose leaves both point endpoints below its plane.
    assert!(
        surface
            .first_hit([0.25, 0.25, -0.8], [0.25, 0.25, -0.6], |_| false)
            .unwrap()
            .is_none()
    );
    reconstruct(&interval, hit);
}

#[test]
fn zero_translation_preserves_every_static_hit_field_and_miss() {
    let surface = mesh();
    let interval = TranslationInterval::new(&surface, 0, [-2.0, 2.0], [[0.0; 3]; 2]).unwrap();
    let (start, end) = ([0.25, 0.25, 1.0], [0.25, 0.25, -1.0]);
    let expected = surface.first_hit(start, end, |_| false).unwrap().unwrap();
    let hit = interval.first_hit(start, end, |_| false).unwrap().unwrap();
    assert_eq!(hit.reference_hit, expected);
    assert_eq!(hit.world_position, expected.position);
    assert_eq!(hit.time, 0.0);
    assert_eq!(hit.wall_velocity, [0.0; 3]);
    assert!(
        interval
            .first_hit([2.0, 2.0, 1.0], [2.0, 2.0, -1.0], |_| false)
            .unwrap()
            .is_none()
    );
}

#[test]
fn common_linear_frame_shift_preserves_contact_parameter_normal_and_weights() {
    let surface = mesh();
    let start = [0.25, 0.25, 1.0];
    let end = [0.25, 0.25, -1.0];
    let expected = surface.first_hit(start, end, |_| false).unwrap().unwrap();
    let offsets = [[2.0, -1.0, 0.5], [3.0, 0.0, 2.5]];
    let interval = TranslationInterval::new(&surface, 103, [3.0, 5.0], offsets).unwrap();
    let a = std::array::from_fn(|d| start[d] + offsets[0][d]);
    let b = std::array::from_fn(|d| end[d] + offsets[1][d]);
    let hit = interval.first_hit(a, b, |_| false).unwrap().unwrap();
    assert_eq!(hit.reference_hit, expected);
    assert_eq!(hit.time, 4.0);
    assert_eq!(hit.wall_velocity, [0.5, 0.5, 1.0]);
    reconstruct(&interval, hit);
}

#[test]
fn axis_permuted_hand_oracles_cover_signed_and_tangential_translation() {
    let mut queries = 0;
    for axis in 0..3 {
        let mut a = [0.0; 3];
        a[(axis + 1) % 3] = 1.0;
        let mut b = [0.0; 3];
        b[(axis + 2) % 3] = 1.0;
        let surface = TriangleSurface::new(
            SurfaceStamp {
                id: 9,
                version: axis as u64,
            },
            vec![[0.0; 3], a, b],
            vec![[0, 1, 2]],
            SurfaceSettings::default(),
        )
        .unwrap();
        for direction in [-1.0, 1.0] {
            for distance in [0.25, 0.5, 1.0] {
                for travel in [0.5, 1.0, 2.0] {
                    for speed in [-0.5, 0.0, 0.5, 1.0] {
                        let offsets = [[0.125; 3], [speed; 3]];
                        let mut p = [0.0; 3];
                        p[(axis + 1) % 3] = 0.25;
                        p[(axis + 2) % 3] = 0.125;
                        let mut q = p;
                        p[axis] = direction * distance;
                        q[axis] = -direction * travel;
                        let start = std::array::from_fn(|d| p[d] + offsets[0][d]);
                        let end = std::array::from_fn(|d| q[d] + offsets[1][d]);
                        let interval =
                            TranslationInterval::new(&surface, queries, [7.0, 7.5], offsets)
                                .unwrap();
                        let hit = interval.first_hit(start, end, |_| false).unwrap().unwrap();
                        let t = distance / (distance + travel);
                        near(hit.reference_hit.parameter, t);
                        near(hit.time, 7.0 + 0.5 * t);
                        assert_eq!(hit.reference_hit.barycentric, [0.625, 0.25, 0.125]);
                        assert_eq!(
                            hit.reference_hit.facing,
                            if direction > 0.0 {
                                HitFacing::Front
                            } else {
                                HitFacing::Back
                            }
                        );
                        reconstruct(&interval, hit);
                        queries += 1;
                    }
                }
            }
        }
    }
    assert_eq!(queries, 216);
}

#[test]
fn reversed_winding_changes_facing_but_not_translated_contact() {
    let vertices = vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
    let forward = mesh();
    let reverse = TriangleSurface::new(
        forward.stamp(),
        vertices,
        vec![[0, 2, 1]],
        SurfaceSettings::default(),
    )
    .unwrap();
    let motion = [[0.0, 0.0, -1.0], [0.0, 0.0, 1.0]];
    let point = [0.25, 0.25, 0.0];
    let a = TranslationInterval::new(&forward, 1, [0.0, 1.0], motion)
        .unwrap()
        .first_hit(point, point, |_| false)
        .unwrap()
        .unwrap();
    let b = TranslationInterval::new(&reverse, 1, [0.0, 1.0], motion)
        .unwrap()
        .first_hit(point, point, |_| false)
        .unwrap()
        .unwrap();
    assert_eq!(a.world_position, b.world_position);
    assert_eq!(a.time, b.time);
    assert_eq!(a.reference_hit.normal, [0.0, 0.0, 1.0]);
    assert_eq!(b.reference_hit.normal, [0.0, 0.0, -1.0]);
    assert_eq!(b.reference_hit.facing, HitFacing::Back);
}

#[test]
fn endpoint_contacts_and_clipping_keep_space_time_coordinates() {
    let surface = mesh();
    let interval =
        TranslationInterval::new(&surface, 8, [2.0, 4.0], [[0.0; 3], [0.0, 0.0, 1.0]]).unwrap();
    let start = [0.25, 0.25, 1.0];
    let end = [0.25, 0.25, 0.0];
    let clipped = interval.clip_segment(start, end, |_| false).unwrap();
    assert_eq!(clipped.end, [0.25, 0.25, 0.5]);
    assert_eq!(clipped.hit.unwrap().time, 3.0);
    let at_start = interval
        .first_hit([0.25, 0.25, 0.0], end, |_| false)
        .unwrap()
        .unwrap();
    assert_eq!(at_start.reference_hit.parameter, 0.0);
    assert_eq!(at_start.time, 2.0);
    let at_end = interval
        .first_hit(start, start, |_| false)
        .unwrap()
        .unwrap();
    assert_eq!(at_end.reference_hit.parameter, 1.0);
    assert_eq!(at_end.time, 4.0);
    let miss_end = [2.0, 2.0, 0.0];
    let miss = interval
        .clip_segment([2.0, 2.0, 1.0], miss_end, |_| false)
        .unwrap();
    assert_eq!(miss.end, miss_end);
    assert!(miss.hit.is_none());
}

#[test]
fn cancellation_after_contact_exposes_no_result_and_preserves_retry() {
    let surface = TriangleSurface::new(
        mesh().stamp(),
        mesh().vertices().to_vec(),
        vec![[0, 1, 2]; 3],
        SurfaceSettings::default(),
    )
    .unwrap();
    let interval = TranslationInterval::new(
        &surface,
        19,
        [4.0, 5.0],
        [[0.0, 0.0, -1.0], [0.0, 0.0, 1.0]],
    )
    .unwrap();
    let point = [0.25, 0.25, 0.0];
    let expected = interval.first_hit(point, point, |_| false).unwrap();
    let vertices = surface.vertices().to_vec();
    let bytes = surface.allocated_bytes();
    let mut seen = vec![];
    assert_eq!(
        interval.first_hit(point, point, |i| {
            seen.push(i);
            i == 2
        }),
        Err(TranslationError::Surface(SurfaceError::Cancelled {
            triangle: 2
        }))
    );
    assert_eq!(seen, [0, 1, 2]);
    assert_eq!(surface.vertices(), vertices);
    assert_eq!(surface.allocated_bytes(), bytes);
    assert_eq!(interval.times(), [4.0, 5.0]);
    assert_eq!(interval.interval_id(), 19);
    assert_eq!(
        interval.first_hit(point, point, |_| false).unwrap(),
        expected
    );
    assert_eq!(expected.unwrap().reference_hit.triangle, 0);
}

#[test]
fn invalid_intervals_trajectories_and_zero_relative_motion_are_explicit() {
    let surface = mesh();
    for times in [
        [1.0, 1.0],
        [2.0, 1.0],
        [f64::NAN, 2.0],
        [-f64::MAX, f64::MAX],
    ] {
        assert!(matches!(
            TranslationInterval::new(&surface, 1, times, [[0.0; 3]; 2]),
            Err(TranslationError::InvalidInterval)
        ));
    }
    for offsets in [[[f64::INFINITY; 3]; 2], [[-f64::MAX; 3], [f64::MAX; 3]]] {
        assert!(matches!(
            TranslationInterval::new(&surface, 1, [0.0, 1.0], offsets),
            Err(TranslationError::InvalidInterval)
        ));
    }
    assert!(matches!(
        TranslationInterval::new(&surface, 1, [0.0, f64::from_bits(1)], [[0.0; 3], [1.0; 3]]),
        Err(TranslationError::InvalidInterval)
    ));
    let interval =
        TranslationInterval::new(&surface, 1, [0.0, 1.0], [[0.0; 3], [0.0, 0.0, 1.0]]).unwrap();
    assert_eq!(
        interval.first_hit([0.25, 0.25, 0.0], [0.25, 0.25, 1.0], |_| false),
        Err(TranslationError::NoRelativeMotion)
    );
    assert_eq!(
        interval.first_hit([f64::NAN; 3], [0.0; 3], |_| false),
        Err(TranslationError::InvalidTrajectory)
    );
    assert_eq!(
        interval.first_hit([-f64::MAX; 3], [f64::MAX; 3], |_| false),
        Err(TranslationError::InvalidTrajectory)
    );
    assert!(matches!(
        TranslationInterval::new(&surface, 1, [0.0, f64::MAX], [[0.0; 3], [1e-100; 3]]),
        Err(TranslationError::InvalidInterval)
    ));
    let still = TranslationInterval::new(&surface, 2, [0.0, 1.0], [[0.0; 3]; 2]).unwrap();
    assert_eq!(
        still.first_hit([0.25, 0.25, 0.0], [0.5, 0.25, 0.0], |_| false),
        Err(TranslationError::Surface(
            SurfaceError::AmbiguousIntersection { triangle: 0 }
        ))
    );
}

#[test]
fn unrepresentable_endpoint_pose_rejects_before_query() {
    let huge = f64::MAX * 0.75;
    let surface = TriangleSurface::new(
        mesh().stamp(),
        vec![[huge, 0.0, 0.0], [huge, 1.0, 0.0], [huge, 0.0, 1.0]],
        vec![[0, 1, 2]],
        SurfaceSettings::default(),
    )
    .unwrap();
    assert!(matches!(
        TranslationInterval::new(&surface, 1, [0.0, 1.0], [[huge, 0.0, 0.0]; 2]),
        Err(TranslationError::UnrepresentablePose { vertex: 0 })
    ));
}
