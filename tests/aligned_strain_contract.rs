use rheon::{
    AlignedStrain, AlignedStrainBoundary, AlignedStrainError, Axis, GridGeometry,
    ObstacleFlowError, ObstacleFlowStage, StaticObstacleGeometry, SurfaceSettings, SurfaceStamp,
    TriangleSurface,
};
fn mesh(lo: [f64; 3], hi: [f64; 3]) -> TriangleSurface {
    TriangleSurface::new(
        SurfaceStamp { id: 91, version: 2 },
        (0..8)
            .map(|c| std::array::from_fn(|d| if c & (1 << d) == 0 { lo[d] } else { hi[d] }))
            .collect(),
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
fn owner(
    n: [u64; 3],
    h: [f64; 3],
    origin: [f64; 3],
    lo: [usize; 3],
    hi: [usize; 3],
) -> StaticObstacleGeometry {
    let lower = std::array::from_fn(|d| origin[d] + lo[d] as f64 * h[d]);
    let upper = std::array::from_fn(|d| origin[d] + hi[d] as f64 * h[d]);
    StaticObstacleGeometry::new(
        GridGeometry::new(n, h, origin).unwrap(),
        mesh(lower, upper),
        16_000_000,
        |_, _| false,
    )
    .unwrap()
}
fn op(o: &StaticObstacleGeometry) -> AlignedStrain<'_> {
    AlignedStrain::new(o, 2.0, 0.375, 16_000_000, |_, _| false).unwrap()
}
fn axis(a: Axis) -> usize {
    match a {
        Axis::X => 0,
        Axis::Y => 1,
        Axis::Z => 2,
    }
}
fn close(a: f64, b: f64) {
    assert!(
        (a - b).abs() < 1e-11 * (1.0 + a.abs() + b.abs()),
        "{a} != {b}"
    );
}
fn rows_at<'a>(
    op: &'a AlignedStrain<'_>,
    a: Axis,
    b: Axis,
    p: [usize; 3],
) -> Vec<&'a rheon::AlignedStrainRow> {
    op.rows()
        .iter()
        .filter(|r| r.axes == [a, b] && r.coordinates == p)
        .collect()
}
fn block(rows: &[&rheon::AlignedStrainRow], ids: [usize; 2]) -> [[f64; 2]; 2] {
    let mut result = [[0.0; 2]; 2];
    for r in rows {
        for i in 0..2 {
            for j in 0..2 {
                let c = |id| {
                    r.terms()
                        .iter()
                        .find(|t| t.active == id)
                        .map_or(0.0, |t| t.coefficient)
                };
                result[i][j] += r.weight * c(ids[i]) * c(ids[j]);
            }
        }
    }
    result
}
#[test]
fn active_space_mass_and_zero_rows_match_sealed_pressure_contract() {
    let o = owner([3, 4, 3], [0.5, 0.75, 1.25], [0.0; 3], [1, 1, 1], [2, 3, 2]);
    let s = op(&o);
    assert!(std::ptr::eq(s.geometry(), &o));
    let mut ids = 0;
    for a in [Axis::X, Axis::Y, Axis::Z] {
        let d = axis(a);
        let shape = o.grid().face_counts(a);
        for f in 0..o.grid().face_len(a) {
            let p = [
                f % shape[0],
                f / shape[0] % shape[1],
                f / (shape[0] * shape[1]),
            ];
            let active = p[d] > 0 && p[d] < o.grid().counts()[d] && o.open_areas(a)[f] > 0.0;
            if active {
                let face = &s.active_faces()[ids];
                assert_eq!(s.active_index(a, f), Some(ids));
                assert_eq!(face.axis, a);
                assert_eq!(face.face, f);
                let origin = o.grid().origin()[d];
                let h = o.grid().spacing()[d];
                let distance =
                    (origin + (p[d] as f64 + 0.5) * h) - (origin + (p[d] as f64 - 0.5) * h);
                assert_eq!(face.distance.to_bits(), distance.to_bits());
                assert_eq!(
                    face.mass.to_bits(),
                    ((2.0 * o.open_areas(a)[f]) * distance).to_bits()
                );
                ids += 1;
            } else {
                assert_eq!(s.active_index(a, f), None);
            }
        }
    }
    assert_eq!(ids, s.active_faces().len());
    assert!(s.rows().iter().any(|r| r.terms().is_empty()));
    assert!(s.rows().iter().filter(|r| r.axes[0] != r.axes[1]).all(|r| {
        let a = axis(r.axes[0]);
        let b = axis(r.axes[1]);
        r.coordinates[a] > 0
            && r.coordinates[a] < o.grid().counts()[a]
            && r.coordinates[b] > 0
            && r.coordinates[b] < o.grid().counts()[b]
    }));
}
#[test]
fn unit_corner_is_two_by_two_cross_component_block() {
    let o = owner([4; 3], [1.0; 3], [0.0; 3], [1; 3], [2; 3]);
    let s = op(&o);
    let p = [2, 2, 1];
    let rows = rows_at(&s, Axis::X, Axis::Y, p);
    assert_eq!(rows.len(), 3);
    assert!(
        rows.iter()
            .all(|r| r.boundary == AlignedStrainBoundary::ObstacleCorner)
    );
    let ids = [
        s.active_index(Axis::X, o.grid().face_index(Axis::X, p).unwrap())
            .unwrap(),
        s.active_index(Axis::Y, o.grid().face_index(Axis::Y, p).unwrap())
            .unwrap(),
    ];
    assert_eq!(block(&rows, ids), [[2.0, 1.0], [1.0, 2.0]]);
    let cross = rows.iter().find(|r| r.terms().len() == 2).unwrap();
    assert_eq!(cross.quadrant, 3);
}
#[test]
fn all_reflected_anisotropic_corners_have_signed_hinge_cross_terms() {
    let o = owner([4; 3], [0.5, 1.5, 2.0], [0.0; 3], [1; 3], [3; 3]);
    let s = op(&o);
    for (x, y, missing) in [(1, 1, 3usize), (1, 3, 1), (3, 1, 2), (3, 3, 0)] {
        let p = [x, y, 1];
        let rows = rows_at(&s, Axis::X, Axis::Y, p);
        assert_eq!(rows.len(), 3);
        assert!(rows.iter().all(|r| r.quadrant as usize != missing));
        let mut up = p;
        if missing & 2 != 0 {
            up[1] -= 1;
        }
        let mut vp = p;
        if missing & 1 != 0 {
            vp[0] -= 1;
        }
        let ids = [
            s.active_index(Axis::X, o.grid().face_index(Axis::X, up).unwrap())
                .unwrap(),
            s.active_index(Axis::Y, o.grid().face_index(Axis::Y, vp).unwrap())
                .unwrap(),
        ];
        let k = block(&rows, ids);
        close(k[0][0], 4.0 / 3.0);
        close(k[1][1], 12.0);
        let sign = if (missing & 1 != 0) == (missing & 2 != 0) {
            1.0
        } else {
            -1.0
        };
        close(k[0][1], 2.0 * sign);
        close(k[1][0], 2.0 * sign);
    }
}
#[test]
fn represented_nonmidpoint_flat_sectors_use_actual_width_and_wall_distance() {
    let origin = [100_000_000.0, -100_000_000.0, 0.1];
    let h = [0.3, 0.7, 1.1];
    let o = owner([5; 3], h, origin, [1; 3], [4; 3]);
    let s = op(&o);
    let p = [2, 4, 2];
    let rows = rows_at(&s, Axis::X, Axis::Y, p);
    assert_eq!(rows.len(), 2);
    let delta = (origin[1] + (4.0 + 0.5) * h[1]) - (origin[1] + 4.0 * h[1]);
    assert_ne!(delta.to_bits(), (h[1] * 0.5).to_bits());
    for r in &rows {
        assert_eq!(r.boundary, AlignedStrainBoundary::ObstacleFlat);
        assert_eq!(r.terms().len(), 1);
        assert_eq!(r.terms()[0].coefficient.to_bits(), (1.0 / delta).to_bits());
        let xcell = if r.quadrant & 1 == 0 { 1.0 } else { 2.0 };
        let da = ((origin[0] + (xcell + 0.5) * h[0]) - (origin[0] + 2.0 * h[0])).abs();
        let dz = (origin[2] + 3.0 * h[2]) - (origin[2] + 2.0 * h[2]);
        assert_eq!(r.weight.to_bits(), ((da * delta) * dz).to_bits());
    }
    let conductance: f64 = rows
        .iter()
        .map(|r| r.weight * r.terms()[0].coefficient.powi(2))
        .sum();
    close(
        conductance,
        rows.iter().map(|r| r.weight).sum::<f64>() / delta.powi(2),
    );
}
#[test]
fn interior_affine_strain_and_compatible_rotation_use_symmetric_gradient() {
    let o = owner([6; 3], [0.5, 0.75, 1.25], [0.0; 3], [2; 3], [3; 3]);
    let s = op(&o);
    let matrix = [[0.5, 2.0, -1.0], [3.0, -0.75, 4.0], [5.0, -2.0, 1.5]];
    let field: Vec<f64> = s
        .active_faces()
        .iter()
        .map(|f| {
            let d = axis(f.axis);
            (0..3).map(|j| matrix[d][j] * f.position[j]).sum()
        })
        .collect();
    let eval = |r: &rheon::AlignedStrainRow, u: &[f64]| {
        r.terms()
            .iter()
            .map(|t| t.coefficient * u[t.active])
            .sum::<f64>()
    };
    for r in s.rows().iter().filter(|r| r.coordinates == [4, 4, 1]) {
        let a = axis(r.axes[0]);
        let b = axis(r.axes[1]);
        let expected = if a == b {
            matrix[a][a]
        } else {
            matrix[a][b] + matrix[b][a]
        };
        close(eval(r, &field), expected);
    }
    let rotation = [[0.0, -2.0, 3.0], [2.0, 0.0, -4.0], [-3.0, 4.0, 0.0]];
    let field: Vec<f64> = s
        .active_faces()
        .iter()
        .map(|f| {
            let d = axis(f.axis);
            (0..3).map(|j| rotation[d][j] * f.position[j]).sum()
        })
        .collect();
    for r in s.rows().iter().filter(|r| r.coordinates == [4, 4, 1]) {
        close(eval(r, &field), 0.0);
    }
}
#[test]
fn action_is_symmetric_and_force_work_matches_dissipation() {
    let o = owner([4, 5, 3], [0.5, 0.75, 1.25], [0.0; 3], [1, 2, 1], [3, 3, 2]);
    let s = op(&o);
    let n = s.active_faces().len();
    let a: Vec<f64> = (0..n).map(|i| (i % 7) as f64 / 4.0 - 0.5).collect();
    let b: Vec<f64> = (0..n).map(|i| (i % 11) as f64 / 8.0 - 0.25).collect();
    let mut ka = vec![0.0; n];
    let mut kb = vec![0.0; n];
    s.apply(&a, &mut ka, |_, _| false).unwrap();
    s.apply(&b, &mut kb, |_, _| false).unwrap();
    close(
        a.iter().zip(&kb).map(|(a, b)| a * b).sum(),
        b.iter().zip(&ka).map(|(a, b)| a * b).sum(),
    );
    let ledger = s.diagnose(&a, &mut ka, |_, _| false).unwrap();
    assert!(ledger.dissipation > 0.0);
    assert!(ledger.force_work < 0.0);
    assert!(ledger.identity_error.abs() <= ledger.rounding_budget);
    let mut expected = 0.0f64;
    for (i, f) in s.active_faces().iter().enumerate() {
        let bound: f64 = s
            .rows()
            .iter()
            .map(|r| {
                r.weight
                    * r.terms()
                        .iter()
                        .find(|t| t.active == i)
                        .map_or(0.0, |t| t.coefficient.abs())
                    * r.terms().iter().map(|t| t.coefficient.abs()).sum::<f64>()
            })
            .sum();
        expected = expected.max(bound / f.mass);
    }
    close(s.unenclosed_b_estimate(), expected);
}
#[test]
fn unsupported_bounds_capacity_inputs_and_cancellation_refuse_consistently() {
    let g = GridGeometry::new([4; 3], [1.0; 3], [0.0; 3]).unwrap();
    for (lo, hi) in [([0.0, 1.0, 1.0], [2.0; 3]), ([1.0; 3], [2.5, 2.0, 2.0])] {
        let o =
            StaticObstacleGeometry::new(g.clone(), mesh(lo, hi), 16_000_000, |_, _| false).unwrap();
        assert!(matches!(
            AlignedStrain::new(&o, 1.0, 1.0, 16_000_000, |_, _| false),
            Err(AlignedStrainError::UnsupportedGeometry)
        ));
    }
    let o = owner([4; 3], [1.0; 3], [0.0; 3], [1; 3], [2; 3]);
    let s = op(&o);
    assert!(matches!(
        AlignedStrain::new(&o, 2.0, 0.375, s.allocated_bytes() - 1, |_, _| false),
        Err(AlignedStrainError::Flow(
            ObstacleFlowError::BufferLimit { .. }
        ))
    ));
    assert_eq!(
        AlignedStrain::new(&o, 2.0, 0.375, s.allocated_bytes(), |_, _| false)
            .unwrap()
            .allocated_bytes(),
        s.allocated_bytes()
    );
    assert!(matches!(
        AlignedStrain::new(&o, 2.0, 0.375, 16_000_000, |_, _| true),
        Err(AlignedStrainError::Flow(ObstacleFlowError::Cancelled {
            stage: ObstacleFlowStage::Assembly,
            ..
        }))
    ));
    let n = s.active_faces().len();
    let mut out = vec![7.0; n];
    let before = out.clone();
    let mut u = vec![0.0; n];
    u[0] = f64::NAN;
    assert!(matches!(
        s.apply(&u, &mut out, |_, _| false),
        Err(AlignedStrainError::Flow(ObstacleFlowError::NonFiniteInput))
    ));
    assert_eq!(out, before);
    assert!(matches!(
        s.apply(&[], &mut out, |_, _| false),
        Err(AlignedStrainError::Flow(ObstacleFlowError::ShapeMismatch))
    ));
    assert!(matches!(
        s.apply(&vec![0.0; n], &mut out, |_, _| true),
        Err(AlignedStrainError::Flow(ObstacleFlowError::Cancelled {
            stage: ObstacleFlowStage::Correction,
            ..
        }))
    ));
}
