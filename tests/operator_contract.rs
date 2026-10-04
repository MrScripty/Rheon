use rheon::{Axis, GridGeometry, OperatorError, PressureOperator};

#[test]
fn three_cell_hand_calculation_and_residual_identity() {
    let grid = GridGeometry::new([3, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let op = PressureOperator::new(&grid, 1.0).unwrap();
    let x = [0.0_f32, 2.0, -1.0, 0.0];
    let y = [0.0_f32; 6];
    let z = [0.0_f32; 6];
    let mut rhs = [0.0; 3];
    op.build_rhs([&x, &y, &z], 0.5, &mut rhs).unwrap();
    assert_eq!(rhs, [-4.0, 6.0, -2.0]);
    let pressure = [0.0, 4.0, 2.0];
    let mut product = [0.0; 3];
    op.apply_full(&pressure, &mut product).unwrap();
    assert_eq!(product, rhs);
    let mut ox = [0.0_f32; 4];
    let mut oy = [0.0_f32; 6];
    let mut oz = [0.0_f32; 6];
    op.correct_velocity([&x, &y, &z], &pressure, 0.5, [&mut ox, &mut oy, &mut oz])
        .unwrap();
    assert_eq!(ox, [0.0; 4]);
    let mut div = [0.0; 3];
    op.divergence([&ox, &oy, &oz], &mut div).unwrap();
    assert_eq!(div, [0.0; 3]);
}

#[test]
fn independent_rectangular_dense_operator() {
    let grid = GridGeometry::new([3, 2, 1], [0.5, 2.0, 3.0], [0.0; 3]).unwrap();
    let op = PressureOperator::new(&grid, 2.0).unwrap();
    // Independent hand-listed adjacency, with area/(rho*distance).
    let edges = [
        (0, 1, 6.0),
        (1, 2, 6.0),
        (3, 4, 6.0),
        (4, 5, 6.0),
        (0, 3, 0.375),
        (1, 4, 0.375),
        (2, 5, 0.375),
    ];
    let p = [0.7, -0.3, 1.4, 2.0, -1.0, 0.9];
    let mut expected = [0.0; 6];
    for (a, b, weight) in edges {
        let value = weight * (p[a] - p[b]);
        expected[a] += value;
        expected[b] -= value;
    }
    let mut actual = [0.0; 6];
    op.apply_full(&p, &mut actual).unwrap();
    for i in 0..6 {
        assert!((actual[i] - expected[i]).abs() < 1e-12);
    }
    let mut constant = [0.0; 6];
    op.apply_full(&[3.0; 6], &mut constant).unwrap();
    assert_eq!(constant, [0.0; 6]);
}

#[test]
fn integrated_rhs_matches_independent_physical_divergence() {
    let grid = GridGeometry::new([2, 1, 1], [0.25, 0.5, 2.0], [0.0; 3]).unwrap();
    let op = PressureOperator::new(&grid, 1.0).unwrap();
    let x = [0.0, 0.75, 0.0];
    let y = [0.0; 4];
    let z = [0.0; 4];
    let mut rhs = [0.0; 2];
    let mut div = [0.0; 2];
    op.build_rhs([&x, &y, &z], 0.1, &mut rhs).unwrap();
    op.divergence([&x, &y, &z], &mut div).unwrap();
    assert_eq!(div, [3.0, -3.0]);
    assert_eq!(rhs, [-7.5, 7.5]);
    for i in 0..2 {
        assert!((div[i] + 0.1 * rhs[i] / grid.cell_volume()).abs() < 1e-12);
    }
}

#[test]
fn all_six_faces_of_single_cell_are_walls() {
    let grid = GridGeometry::new([1; 3], [1.0; 3], [0.0; 3]).unwrap();
    let op = PressureOperator::new(&grid, 1.0).unwrap();
    let velocity = [0.0; 2];
    let mut rhs = [99.0];
    op.build_rhs([&velocity, &velocity, &velocity], 1.0, &mut rhs)
        .unwrap();
    assert_eq!(rhs, [0.0]);
    op.apply_full(&[5.0], &mut rhs).unwrap();
    assert_eq!(rhs, [0.0]);
    let nonzero = [0.0, 1.0];
    assert!(matches!(
        op.build_rhs([&nonzero, &velocity, &velocity], 1.0, &mut rhs),
        Err(OperatorError::NonZeroWallVelocity { axis: Axis::X, .. })
    ));
}

#[test]
fn nonfinite_fields_density_and_lengths_are_rejected() {
    let grid = GridGeometry::new([2; 3], [1.0; 3], [0.0; 3]).unwrap();
    assert!(matches!(
        PressureOperator::new(&grid, f64::NAN),
        Err(OperatorError::InvalidDensity)
    ));
    let op = PressureOperator::new(&grid, 1.0).unwrap();
    let mut out = [0.0; 8];
    assert_eq!(
        op.apply_full(&[f64::INFINITY; 8], &mut out),
        Err(OperatorError::NonFiniteInput)
    );
    assert_eq!(
        op.apply_full(&[0.0; 7], &mut out),
        Err(OperatorError::LengthMismatch)
    );
}
