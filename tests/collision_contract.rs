use rheon::{HitFacing, SurfaceError, SurfaceSettings, SurfaceStamp, TriangleSurface};
use std::mem::size_of;

fn stamp() -> SurfaceStamp {
    SurfaceStamp { id: 71, version: 2 }
}
fn triangle() -> TriangleSurface {
    TriangleSurface::new(
        stamp(),
        vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        vec![[0, 1, 2]],
        SurfaceSettings::default(),
    )
    .unwrap()
}
fn cube(shift: f64) -> TriangleSurface {
    let mut vertices = vec![];
    for z in [-0.3, 0.3] {
        for y in [-0.3, 0.3] {
            for x in [-0.3, 0.3] {
                vertices.push([x + shift, y, z]);
            }
        }
    }
    let triangles = vec![
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
    ];
    TriangleSurface::new(stamp(), vertices, triangles, SurfaceSettings::default()).unwrap()
}
fn near(a: f64, b: f64) {
    assert!((a - b).abs() < 2e-14, "{a} != {b}");
}

#[test]
fn book_cube_outside_crossing_miss_and_inside_exit_match_hand_arithmetic() {
    for shift in [-0.1, 0.0, 0.1] {
        let mesh = cube(shift);
        let hit = mesh
            .first_hit([-0.8, 0.1, 0.0], [0.8, 0.1, 0.0], |_| false)
            .unwrap()
            .unwrap();
        near(hit.parameter, (0.5 + shift) / 1.6);
        near(hit.position[0], -0.3 + shift);
        assert_eq!(hit.surface, stamp());
        assert_eq!(hit.normal, [-1.0, 0.0, 0.0]);
        assert_eq!(hit.facing, HitFacing::Front);
        assert!(
            mesh.first_hit([-0.8, 0.5, 0.0], [0.8, 0.5, 0.0], |_| false)
                .unwrap()
                .is_none()
        );
        let exit = mesh
            .first_hit([shift, 0.0, 0.0], [0.8, 0.0, 0.0], |_| false)
            .unwrap()
            .unwrap();
        near(exit.parameter, 0.3 / (0.8 - shift));
        assert_eq!(exit.facing, HitFacing::Back);
        // "Back" is geometric facing, not an inside classifier.
        assert_eq!(exit.normal, [1.0, 0.0, 0.0]);
    }
}

#[test]
fn transverse_interior_edges_vertices_and_segment_endpoints_are_explicit_hits() {
    let mesh = triangle();
    for (p, weights) in [
        ([0.25, 0.25], [0.5, 0.25, 0.25]),
        ([0.5, 0.0], [0.5, 0.5, 0.0]),
        ([0.0, 0.0], [1.0, 0.0, 0.0]),
        ([0.5, 0.5], [0.0, 0.5, 0.5]),
    ] {
        let h = mesh
            .first_hit([p[0], p[1], 1.0], [p[0], p[1], -1.0], |_| false)
            .unwrap()
            .unwrap();
        assert_eq!(h.parameter, 0.5);
        assert_eq!(h.barycentric, weights);
        assert_eq!(h.normal, [0.0, 0.0, 1.0]);
        let at_start = mesh
            .first_hit([p[0], p[1], 0.0], [p[0], p[1], -1.0], |_| false)
            .unwrap()
            .unwrap();
        assert_eq!(at_start.parameter, 0.0);
        let at_end = mesh
            .first_hit([p[0], p[1], 1.0], [p[0], p[1], 0.0], |_| false)
            .unwrap()
            .unwrap();
        assert_eq!(at_end.parameter, 1.0);
    }
    assert!(
        mesh.first_hit([0.75, 0.75, 1.0], [0.75, 0.75, -1.0], |_| false)
            .unwrap()
            .is_none()
    );
}

#[test]
fn oblique_facet_reconstructs_barycentric_point_and_unit_geometric_normal() {
    let vertices = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 1.0, 1.0]];
    let mesh = TriangleSurface::new(
        stamp(),
        vertices.clone(),
        vec![[0, 1, 2]],
        SurfaceSettings::default(),
    )
    .unwrap();
    // p=(1/4,1/4,1/2), n proportional to (-1,-1,1). A symmetric trace.
    let h = mesh
        .first_hit([0.0, 0.0, 0.75], [0.5, 0.5, 0.25], |_| false)
        .unwrap()
        .unwrap();
    near(h.parameter, 0.5);
    for (d, &position) in h.position.iter().enumerate() {
        let reconstructed = (0..3)
            .map(|i| h.barycentric[i] * vertices[i][d])
            .sum::<f64>();
        near(reconstructed, position);
    }
    near(h.normal.iter().map(|n| n * n).sum::<f64>(), 1.0);
    near(h.normal[0], -1.0 / 3.0_f64.sqrt());
    near(h.normal[2], 1.0 / 3.0_f64.sqrt());
}

#[test]
fn earliest_contact_is_independent_of_order_and_exact_ties_use_lowest_index() {
    let vertices = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [0.0, 1.0, 1.0],
    ];
    for triangles in [
        vec![[0, 1, 2], [3, 4, 5]],
        vec![[3, 4, 5], [0, 1, 2]],
        vec![[3, 4, 5], [3, 4, 5], [0, 1, 2]],
    ] {
        let mesh = TriangleSurface::new(
            stamp(),
            vertices.clone(),
            triangles.clone(),
            SurfaceSettings::default(),
        )
        .unwrap();
        let h = mesh
            .first_hit([0.25, 0.25, 3.0], [0.25, 0.25, -1.0], |_| false)
            .unwrap()
            .unwrap();
        assert_eq!(h.parameter, 0.5);
        assert_eq!(
            h.triangle,
            triangles.iter().position(|t| *t == [3, 4, 5]).unwrap()
        );
    }
}

#[test]
fn reversed_winding_changes_normal_and_facing_but_preserves_contact() {
    let forward = triangle();
    let reverse = TriangleSurface::new(
        stamp(),
        forward.vertices().to_vec(),
        vec![[0, 2, 1]],
        SurfaceSettings::default(),
    )
    .unwrap();
    let a = forward
        .first_hit([0.25, 0.25, 1.0], [0.25, 0.25, -1.0], |_| false)
        .unwrap()
        .unwrap();
    let b = reverse
        .first_hit([0.25, 0.25, 1.0], [0.25, 0.25, -1.0], |_| false)
        .unwrap()
        .unwrap();
    assert_eq!(a.position, b.position);
    assert_eq!(a.parameter, b.parameter);
    assert_eq!(a.facing, HitFacing::Front);
    assert_eq!(b.facing, HitFacing::Back);
    assert_eq!(b.normal, [0.0, 0.0, -1.0]);
}

#[test]
fn conditioning_is_relative_across_small_large_scales_and_translated_geometry() {
    for length in [1e-100, 1.0, 1e100] {
        let mesh = TriangleSurface::new(
            stamp(),
            vec![[0.0; 3], [length, 0.0, 0.0], [0.0, length, 0.0]],
            vec![[0, 1, 2]],
            SurfaceSettings::default(),
        )
        .unwrap();
        let h = mesh
            .first_hit(
                [0.25 * length, 0.25 * length, length],
                [0.25 * length, 0.25 * length, -length],
                |_| false,
            )
            .unwrap()
            .unwrap();
        assert_eq!(h.parameter, 0.5);
        assert_eq!(h.barycentric, [0.5, 0.25, 0.25]);
    }
    let s = 1e6;
    let o = 1e12;
    let mesh = TriangleSurface::new(
        stamp(),
        vec![[o, o, o], [o + s, o, o], [o, o + s, o]],
        vec![[0, 1, 2]],
        SurfaceSettings::default(),
    )
    .unwrap();
    let h = mesh
        .first_hit(
            [o + s / 4.0, o + s / 4.0, o + s],
            [o + s / 4.0, o + s / 4.0, o - s],
            |_| false,
        )
        .unwrap()
        .unwrap();
    assert_eq!(h.position, [o + s / 4.0, o + s / 4.0, o]);
    assert_eq!(h.parameter, 0.5);
}

#[test]
fn coplanar_near_parallel_and_near_boundary_candidates_are_not_silent_misses() {
    let mesh = triangle();
    for (a, b) in [
        ([0.1, 0.1, 0.0], [0.5, 0.1, 0.0]),
        ([0.1, 0.1, 1e-14], [0.5, 0.1, -1e-14]),
        ([0.5, 1e-14, 1.0], [0.5, 1e-14, -1.0]),
    ] {
        assert!(matches!(
            mesh.first_hit(a, b, |_| false),
            Err(SurfaceError::AmbiguousIntersection { triangle: 0 })
        ));
    }
    assert!(
        mesh.first_hit([0.1, 0.1, 0.5], [0.5, 0.1, 0.5], |_| false)
            .unwrap()
            .is_none()
    );
}

#[test]
fn an_ambiguous_facet_prevents_publishing_another_facets_definite_hit() {
    let mesh = TriangleSurface::new(
        stamp(),
        vec![
            [0.0; 3],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.25, 0.0, -2.0],
            [0.25, 1.0, -2.0],
            [0.25, 0.0, 2.0],
        ],
        vec![[0, 1, 2], [3, 4, 5]],
        SurfaceSettings::default(),
    )
    .unwrap();
    assert_eq!(
        mesh.first_hit([0.25, 0.25, 1.0], [0.25, 0.25, -1.0], |_| false)
            .unwrap_err(),
        SurfaceError::AmbiguousIntersection { triangle: 1 }
    );
}

#[test]
fn clipping_returns_first_contact_and_preserves_end_on_a_definite_miss() {
    let mesh = cube(0.0);
    let clipped = mesh
        .clip_segment([-0.8, 0.1, 0.0], [0.8, 0.1, 0.0], |_| false)
        .unwrap();
    let h = clipped.hit.unwrap();
    assert_eq!(clipped.end, h.position);
    near(clipped.end[0], -0.3);
    let end = [0.8, 0.5, 0.0];
    let miss = mesh.clip_segment([-0.8, 0.5, 0.0], end, |_| false).unwrap();
    assert!(miss.hit.is_none());
    assert_eq!(miss.end, end);
}

#[test]
fn cancellation_after_a_found_hit_returns_no_partial_result_or_mesh_mutation() {
    let mesh = cube(0.0);
    let before = (
        mesh.vertices().to_vec(),
        mesh.triangles().to_vec(),
        mesh.stamp(),
        mesh.allocated_bytes(),
    );
    let normal = mesh
        .first_hit([-0.8, 0.1, 0.0], [0.8, 0.1, 0.0], |_| false)
        .unwrap();
    assert_eq!(
        mesh.first_hit([-0.8, 0.1, 0.0], [0.8, 0.1, 0.0], |i| i == 11)
            .unwrap_err(),
        SurfaceError::Cancelled { triangle: 11 }
    );
    assert_eq!(
        mesh.first_hit([-0.8, 0.1, 0.0], [0.8, 0.1, 0.0], |_| false)
            .unwrap(),
        normal
    );
    assert_eq!(
        before,
        (
            mesh.vertices().to_vec(),
            mesh.triangles().to_vec(),
            mesh.stamp(),
            mesh.allocated_bytes()
        )
    );
}

#[test]
fn memory_limit_accounts_for_owned_capacities_not_only_lengths() {
    let mut vertices = Vec::with_capacity(32);
    vertices.extend([[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
    let mut triangles = Vec::with_capacity(16);
    triangles.push([0, 1, 2]);
    let bytes = vertices.capacity() * size_of::<[f64; 3]>()
        + triangles.capacity() * size_of::<[usize; 3]>();
    let mesh = TriangleSurface::new(
        stamp(),
        vertices,
        triangles,
        SurfaceSettings {
            memory_limit: bytes,
            ..SurfaceSettings::default()
        },
    )
    .unwrap();
    assert_eq!(mesh.allocated_bytes(), bytes);
    let v = mesh.vertices().to_vec();
    let t = mesh.triangles().to_vec();
    let required = v.capacity() * size_of::<[f64; 3]>() + t.capacity() * size_of::<[usize; 3]>();
    assert_eq!(
        TriangleSurface::new(
            stamp(),
            v,
            t,
            SurfaceSettings {
                memory_limit: required - 1,
                ..SurfaceSettings::default()
            }
        )
        .unwrap_err(),
        SurfaceError::BufferLimit {
            required,
            limit: required - 1
        }
    );
}

#[test]
fn invalid_admission_and_queries_report_the_actual_unsupported_input() {
    let v = triangle().vertices().to_vec();
    for triangles in [vec![[0, 0, 1]], vec![[0, 1, 1]]] {
        assert_eq!(
            TriangleSurface::new(stamp(), v.clone(), triangles, SurfaceSettings::default())
                .unwrap_err(),
            SurfaceError::DegenerateTriangle { triangle: 0 }
        );
    }
    assert_eq!(
        TriangleSurface::new(
            stamp(),
            v.clone(),
            vec![[0, 1, 7]],
            SurfaceSettings::default()
        )
        .unwrap_err(),
        SurfaceError::InvalidTriangleIndex {
            triangle: 0,
            vertex: 7
        }
    );
    let mut bad = v.clone();
    bad[2][1] = f64::NAN;
    assert_eq!(
        TriangleSurface::new(stamp(), bad, vec![[0, 1, 2]], SurfaceSettings::default())
            .unwrap_err(),
        SurfaceError::NonFiniteVertex { vertex: 2 }
    );
    assert_eq!(
        TriangleSurface::new(stamp(), v.clone(), vec![], SurfaceSettings::default()).unwrap_err(),
        SurfaceError::EmptySurface
    );
    for tolerance in [f64::NAN, 0.0, 1.0] {
        assert_eq!(
            TriangleSurface::new(
                stamp(),
                v.clone(),
                vec![[0, 1, 2]],
                SurfaceSettings {
                    relative_tolerance: tolerance,
                    ..SurfaceSettings::default()
                }
            )
            .unwrap_err(),
            SurfaceError::InvalidSettings
        );
    }
    assert_eq!(
        TriangleSurface::new(
            stamp(),
            vec![[-f64::MAX, 0.0, 0.0], [f64::MAX, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
            SurfaceSettings::default()
        )
        .unwrap_err(),
        SurfaceError::UnrepresentableGeometry { triangle: 0 }
    );
    let mesh = triangle();
    for (a, b) in [
        ([0.0; 3], [0.0; 3]),
        ([f64::NAN, 0.0, 0.0], [0.0; 3]),
        ([-f64::MAX, 0.0, 0.0], [f64::MAX, 0.0, 0.0]),
    ] {
        assert_eq!(
            mesh.first_hit(a, b, |_| false).unwrap_err(),
            SurfaceError::InvalidSegment
        );
    }
}
