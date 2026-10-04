use rheon::{Axis, BufferPlan, GeometryError, GridGeometry};

#[test]
fn rectangular_shapes_and_maximum_indices() {
    let g = GridGeometry::new([3, 2, 4], [0.2, 0.3, 0.4], [-1.0, 2.0, 0.0]).unwrap();
    assert_eq!(g.cell_len(), 24);
    assert_eq!(g.face_counts(Axis::X), [4, 2, 4]);
    assert_eq!(g.face_counts(Axis::Y), [3, 3, 4]);
    assert_eq!(g.face_counts(Axis::Z), [3, 2, 5]);
    assert_eq!(g.cell_index([2, 1, 3]), Some(23));
    assert_eq!(g.face_index(Axis::X, [3, 1, 3]), Some(31));
    assert_eq!(g.face_index(Axis::Y, [2, 2, 3]), Some(35));
    assert_eq!(g.face_index(Axis::Z, [2, 1, 4]), Some(29));
    assert_eq!(g.cell_index([3, 0, 0]), None);
    assert_eq!(g.face_index(Axis::Y, [3, 0, 0]), None);
}

#[test]
fn face_offsets_are_distinct_from_cell_offsets() {
    let g = GridGeometry::new([2, 2, 2], [2.0, 4.0, 6.0], [1.0, 2.0, 3.0]).unwrap();
    assert_eq!(g.cell_position([0, 0, 0]), Some([2.0, 4.0, 6.0]));
    assert_eq!(g.face_position(Axis::X, [0, 0, 0]), Some([1.0, 4.0, 6.0]));
    assert_eq!(g.face_position(Axis::Y, [0, 0, 0]), Some([2.0, 2.0, 6.0]));
    assert_eq!(g.face_position(Axis::Z, [0, 0, 0]), Some([2.0, 4.0, 3.0]));
}

#[test]
fn single_cell_and_one_cell_wide_domains_are_valid() {
    for dims in [[1, 1, 1], [1, 9, 1], [7, 1, 2]] {
        let g = GridGeometry::new(dims, [1.0; 3], [0.0; 3]).unwrap();
        assert_eq!(g.cell_index([0, 0, 0]), Some(0));
    }
    let g = GridGeometry::new([1, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    assert_eq!(g.cell_len(), 1);
    assert_eq!(g.face_len(Axis::X), 2);
}

#[test]
fn invalid_source_values_fail_before_allocation() {
    assert!(matches!(
        GridGeometry::new([0, 1, 1], [1.0; 3], [0.0; 3]),
        Err(GeometryError::ZeroDimension { axis: 0 })
    ));
    for value in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(matches!(
            GridGeometry::new([1; 3], [value, 1.0, 1.0], [0.0; 3]),
            Err(GeometryError::InvalidSpacing { axis: 0 })
        ));
    }
    assert!(matches!(
        GridGeometry::new([1; 3], [1.0; 3], [f64::NAN, 0.0, 0.0]),
        Err(GeometryError::InvalidOrigin { axis: 0 })
    ));
    assert!(GridGeometry::new([u64::MAX, 2, 2], [1.0; 3], [0.0; 3]).is_err());
    assert!(GridGeometry::new([1; 3], [f64::MIN_POSITIVE; 3], [0.0; 3]).is_err());
    assert!(GridGeometry::new([2; 3], [1.0; 3], [1e30, 0.0, 0.0]).is_err());
}

#[test]
fn planned_payload_is_exact_and_cap_is_explicit() {
    let g = GridGeometry::new([64; 3], [1.0 / 64.0; 3], [0.0; 3]).unwrap();
    let plan = BufferPlan::for_grid(&g, 64 * 1024 * 1024).unwrap();
    assert_eq!(plan.velocity_pair_bytes, 6_389_760);
    assert_eq!(plan.tracer_pair_bytes, 2_097_152);
    assert_eq!(plan.pressure_workspace_bytes, 12_582_912);
    assert_eq!(plan.total_bytes, 21_069_824);
    assert!(matches!(
        BufferPlan::for_grid(&g, plan.total_bytes - 1),
        Err(GeometryError::BufferLimit { .. })
    ));
    assert_eq!(BufferPlan::for_grid(&g, plan.total_bytes).unwrap(), plan);
}

#[test]
fn half_cell_resolution_regression_requires_no_allocation() {
    let n = (1_u64 << 52) + 3;
    // This is the old accepted counterexample: integer endpoints are distinct,
    // but the last two center expressions alias before any field allocation.
    let previous = (n - 2) as f64 + 0.5;
    let last = (n - 1) as f64 + 0.5;
    assert_eq!(previous, 4_503_599_627_370_498.0);
    assert_eq!(previous, last);
    assert!(matches!(
        GridGeometry::new([n, 1, 1], [1.0; 3], [0.0; 3]),
        Err(GeometryError::UnrepresentableCoordinates { axis: 0 })
    ));
}

#[test]
fn conservative_axis_margin_covers_large_origins_and_adjacent_samples() {
    for origin in [1e15, -1e15] {
        assert!(matches!(
            GridGeometry::new([4, 1, 1], [1.0; 3], [origin, 0.0, 0.0]),
            Err(GeometryError::UnrepresentableCoordinates { axis: 0 })
        ));
    }
    for origin in [1e12, -1e12, -0.5] {
        let g = GridGeometry::new([64, 1, 1], [1.0; 3], [origin, 0.0, 0.0]).unwrap();
        let mut previous = g.origin()[0];
        for i in 0..64 {
            let center = g.cell_position([i, 0, 0]).unwrap()[0];
            let next_face = g.face_position(Axis::X, [i + 1, 0, 0]).unwrap()[0];
            assert!(previous < center && center < next_face);
            previous = next_face;
        }
    }
}
