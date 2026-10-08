use rheon::{
    Axis, GridGeometry, NO_FLUID_COMPONENT, ObstacleError, ObstacleStage, StaticObstacleGeometry,
    SurfaceSettings, SurfaceStamp, TriangleSurface,
};
use std::mem::size_of;
fn mesh(lo: [f64; 3], hi: [f64; 3]) -> TriangleSurface {
    let vertices = (0..8)
        .map(|c| std::array::from_fn(|d| if c & (1 << d) == 0 { lo[d] } else { hi[d] }))
        .collect();
    TriangleSurface::new(
        SurfaceStamp { id: 91, version: 4 },
        vertices,
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
fn grid() -> GridGeometry {
    GridGeometry::new([4, 3, 2], [0.5, 1.0, 2.0], [-1.0, -1.0, -1.0]).unwrap()
}
fn owner(lo: [f64; 3], hi: [f64; 3]) -> StaticObstacleGeometry {
    StaticObstacleGeometry::new(grid(), mesh(lo, hi), 1_000_000, |_, _| false).unwrap()
}
#[test]
fn dyadic_partial_box_measures_match_independent_hand_controls_and_collision_source() {
    // Grid domain[-1,1]×[-1,2]×[-1,3]. Box volume1/2×1×2=1.
    let o = owner([-0.25, -0.5, -0.5], [0.25, 0.5, 1.5]);
    assert_eq!(o.fluid_volumes().iter().sum::<f64>(), 23.0);
    let c = o.grid().cell_index([1, 0, 0]).unwrap();
    assert_eq!(o.fluid_volumes()[c], 1.0 - (0.25 * 0.5 * 1.5));
    let face = o.face(Axis::X, [2, 0, 0]).unwrap();
    assert_eq!(face.area, 2.0 - (0.5 * 1.5));
    assert_eq!(face.negative, o.grid().cell_index([1, 0, 0]));
    assert_eq!(face.positive, o.grid().cell_index([2, 0, 0]));
    assert_eq!(o.component_count(), 1);
    let hit = o
        .surface()
        .first_hit([-1.0, 0.125, 0.0], [1.0, 0.125, 0.0], |_| false)
        .unwrap()
        .unwrap();
    assert_eq!(hit.parameter, 0.375);
    assert_eq!(hit.position[0], o.box_bounds().0[0]);
    assert_eq!(hit.surface, o.stamp());
}
#[test]
fn aligned_separator_has_two_components_and_no_false_internal_connection() {
    let o = owner([-0.5, -1.0, -1.0], [0.0, 2.0, 3.0]);
    assert_eq!(o.component_count(), 2);
    assert_eq!(o.fluid_volumes().iter().sum::<f64>(), 18.0);
    for k in 0..2 {
        for j in 0..3 {
            assert_eq!(
                o.component_labels()[o.grid().cell_index([0, j, k]).unwrap()],
                0
            );
            assert_eq!(
                o.component_labels()[o.grid().cell_index([1, j, k]).unwrap()],
                NO_FLUID_COMPONENT
            );
            assert_eq!(
                o.component_labels()[o.grid().cell_index([2, j, k]).unwrap()],
                1
            );
            assert_eq!(o.face(Axis::X, [1, j, k]).unwrap().area, 0.0);
            assert_eq!(o.face(Axis::X, [2, j, k]).unwrap().area, 0.0);
        }
    }
}
#[test]
fn subcell_separator_is_refused_instead_of_merging_disconnected_fluid_pieces() {
    let r = StaticObstacleGeometry::new(
        grid(),
        mesh([-0.375, -1.0, -1.0], [-0.125, 2.0, 3.0]),
        1_000_000,
        |_, _| false,
    );
    assert!(matches!(
        r,
        Err(ObstacleError::UnresolvedCellTopology { .. })
    ));
}
#[test]
fn shared_area_flux_cancels_with_opposite_neighbor_incidence_on_every_axis() {
    let o = owner([-0.25, -0.5, -0.5], [0.25, 0.5, 1.5]);
    for axis in [Axis::X, Axis::Y, Axis::Z] {
        let d = match axis {
            Axis::X => 0,
            Axis::Y => 1,
            Axis::Z => 2,
        };
        let mut f = [1, 1, 1];
        f[d] = 1;
        let face = o.face(axis, f).unwrap();
        let mut negative = f;
        negative[d] -= 1;
        let positive = f;
        let mut velocity: [Vec<f64>; 3] =
            std::array::from_fn(|d| vec![0.0; o.grid().face_len([Axis::X, Axis::Y, Axis::Z][d])]);
        velocity[d][o.grid().face_index(axis, f).unwrap()] = 2.0;
        let v = velocity.each_ref().map(|v| v.as_slice());
        assert_eq!(o.outward_flux(negative, v).unwrap(), 2.0 * face.area);
        assert_eq!(o.outward_flux(positive, v).unwrap(), -2.0 * face.area);
        assert_eq!(
            (0..2)
                .flat_map(|k| (0..3).map(move |j| (j, k)))
                .flat_map(|(j, k)| (0..4).map(move |i| [i, j, k]))
                .map(|c| o.outward_flux(c, v).unwrap())
                .sum::<f64>(),
            0.0
        );
    }
}
#[test]
fn dry_wet_outside_touching_and_scaled_translated_domains_preserve_analytic_volume() {
    let dry = owner([-2.0; 3], [4.0; 3]);
    assert_eq!(dry.component_count(), 0);
    assert!(dry.fluid_volumes().iter().all(|v| *v == 0.0));
    let wet = owner([2.0, 3.0, 4.0], [3.0, 4.0, 5.0]);
    assert_eq!(wet.fluid_volumes().iter().sum::<f64>(), 24.0);
    let touch = owner([1.0, -1.0, -1.0], [2.0, 2.0, 3.0]);
    assert_eq!(touch.fluid_volumes().iter().sum::<f64>(), 24.0);
    assert_eq!(touch.face(Axis::X, [4, 1, 1]).unwrap().area, 0.0);
    let g = GridGeometry::new([2, 2, 2], [2.0, 4.0, 8.0], [4.0, -8.0, 16.0]).unwrap();
    let s = StaticObstacleGeometry::new(
        g,
        mesh([5.0, -6.0, 20.0], [7.0, -2.0, 28.0]),
        1_000_000,
        |_, _| false,
    )
    .unwrap();
    assert_eq!(s.fluid_volumes().iter().sum::<f64>(), 512.0 - 64.0);
}
#[test]
fn malformed_meshes_are_refused_without_changing_prior_owner() {
    let prior = owner([-0.25; 3], [0.25; 3]);
    let original = prior.fluid_volumes().to_vec();
    for mode in 0..4 {
        let m = mesh([-0.25; 3], [0.25; 3]);
        let mut v = m.vertices().to_vec();
        let mut t = m.triangles().to_vec();
        match mode {
            0 => {
                t.pop();
            }
            1 => t[0].swap(1, 2),
            2 => t[1] = t[0],
            _ => v[0][0] += 0.125,
        }
        let source = TriangleSurface::new(m.stamp(), v, t, SurfaceSettings::default()).unwrap();
        assert!(matches!(
            StaticObstacleGeometry::new(grid(), source, 1_000_000, |_, _| false),
            Err(ObstacleError::UnsupportedBoxMesh)
        ));
        assert_eq!(prior.fluid_volumes(), original);
    }
}
#[test]
fn actual_input_spare_capacity_constructor_queue_and_cancellation_are_accounted() {
    let m = mesh([-0.25; 3], [0.25; 3]);
    let mut v = m.vertices().to_vec();
    v.reserve_exact(100);
    let mut t = m.triangles().to_vec();
    t.reserve_exact(30);
    let input = (v.capacity() * size_of::<[f64; 3]>()) + (t.capacity() * size_of::<[usize; 3]>());
    let source = TriangleSurface::new(
        m.stamp(),
        v,
        t,
        SurfaceSettings {
            memory_limit: usize::MAX,
            ..SurfaceSettings::default()
        },
    )
    .unwrap();
    let o = StaticObstacleGeometry::new(grid(), source, usize::MAX, |_, _| false).unwrap();
    let retained = input
        + o.grid().cell_len() * (size_of::<f64>() + size_of::<usize>())
        + [Axis::X, Axis::Y, Axis::Z]
            .iter()
            .map(|a| o.grid().face_len(*a) * size_of::<f64>())
            .sum::<usize>();
    assert_eq!(o.allocation().retained_bytes, retained);
    assert_eq!(
        o.allocation().constructor_peak_bytes,
        retained + o.grid().cell_len() * size_of::<usize>()
    );
    assert!(matches!(
        StaticObstacleGeometry::new(grid(), mesh([-0.25; 3], [0.25; 3]), 1, |_, _| false),
        Err(ObstacleError::BufferLimit { .. })
    ));
    for stage in [
        ObstacleStage::Cells,
        ObstacleStage::Faces(Axis::X),
        ObstacleStage::Connectivity,
    ] {
        assert!(
            matches!(StaticObstacleGeometry::new(grid(),mesh([-0.25;3],[0.25;3]),1_000_000,|s,_|s==stage),Err(ObstacleError::Cancelled{stage:s,..}) if s==stage)
        );
    }
}

#[test]
fn tiny_positive_box_measure_cannot_disappear_by_underflow_or_subtraction_rounding() {
    for exponent in [-60, -400] {
        let h = 2.0_f64.powi(exponent);
        let g = GridGeometry::new([1, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
        assert!(matches!(
            StaticObstacleGeometry::new(g, mesh([0.0; 3], [h; 3]), 1_000_000, |_, _| false),
            Err(ObstacleError::ArithmeticFailure)
        ));
    }
}

#[test]
fn all_three_separator_orientations_have_matching_labels_and_unresolved_refusals() {
    for d in 0..3 {
        let g = GridGeometry::new([3, 3, 3], [1.0; 3], [0.0; 3]).unwrap();
        let mut lo = [0.0; 3];
        let mut hi = [3.0; 3];
        lo[d] = 1.0;
        hi[d] = 2.0;
        let o =
            StaticObstacleGeometry::new(g.clone(), mesh(lo, hi), 1_000_000, |_, _| false).unwrap();
        assert_eq!(o.component_count(), 2);
        assert_eq!(o.fluid_volumes().iter().sum::<f64>(), 18.0);
        lo[d] = 1.25;
        hi[d] = 1.75;
        assert!(matches!(
            StaticObstacleGeometry::new(g, mesh(lo, hi), 1_000_000, |_, _| false),
            Err(ObstacleError::UnresolvedCellTopology { .. })
        ));
    }
}
#[test]
fn non_dyadic_physical_endpoint_metrics_match_declared_rounding_allowance() {
    let g = GridGeometry::new([3, 4, 2], [0.3, 0.2, 0.7], [0.1, -0.2, 0.4]).unwrap();
    let lo = [0.21, -0.1, 0.5];
    let hi = [0.62, 0.37, 1.3];
    let o = StaticObstacleGeometry::new(g.clone(), mesh(lo, hi), 1_000_000, |_, _| false).unwrap();
    // Global rectangle oracle uses represented domain endpoints, independently
    // of per-cell overlap loops. This is assembly error, not temporal accuracy.
    let lengths: [f64; 3] = std::array::from_fn(|d| g.upper()[d] - g.origin()[d]);
    let expected =
        lengths.iter().product::<f64>() - (hi[0] - lo[0]) * (hi[1] - lo[1]) * (hi[2] - lo[2]);
    assert!(
        (o.fluid_volumes().iter().sum::<f64>() - expected).abs()
            <= 64.0 * f64::EPSILON * lengths.iter().product::<f64>()
    );
}
