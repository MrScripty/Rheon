use rheon::{GridGeometry, PressureError, PressureOperator, PressureSettings, PressureWorkspace};

fn settings() -> PressureSettings {
    PressureSettings {
        relative_residual: 1e-11,
        absolute_residual: 1e-12,
        divergence_limit: 1e-9,
        max_iterations: 1000,
    }
}

#[test]
fn solves_hand_chain_and_reports_full_residual() {
    let grid = GridGeometry::new([3, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let op = PressureOperator::new(&grid, 1.0).unwrap();
    let mut ws = PressureWorkspace::new(&grid, 3 * 48).unwrap();
    let report = ws
        .solve_rhs(&op, &[-4.0, 6.0, -2.0], 0.5, settings(), || false)
        .unwrap();
    assert!((ws.pressure()[1] - 4.0).abs() < 1e-10);
    assert!((ws.pressure()[2] - 2.0).abs() < 1e-10);
    assert_eq!(ws.pressure()[0], 0.0);
    assert!(report.true_residual_max < 1e-10);
    assert_eq!(ws.allocated_bytes(), 144);
}

#[test]
fn single_cell_zero_rhs_returns_without_iteration() {
    let grid = GridGeometry::new([1; 3], [1.0; 3], [0.0; 3]).unwrap();
    let op = PressureOperator::new(&grid, 1.0).unwrap();
    let mut ws = PressureWorkspace::new(&grid, 48).unwrap();
    let report = ws
        .solve_rhs(&op, &[0.0], 1.0, settings(), || false)
        .unwrap();
    assert_eq!(report.iterations, 0);
    assert_eq!(report.true_residual_max, 0.0);
}

#[test]
fn incompatible_rhs_is_not_mean_shifted() {
    let grid = GridGeometry::new([2, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let op = PressureOperator::new(&grid, 1.0).unwrap();
    let mut ws = PressureWorkspace::new(&grid, 96).unwrap();
    assert!(matches!(
        ws.solve_rhs(&op, &[1.0, 0.0], 1.0, settings(), || false),
        Err(PressureError::IncompatibleRhs { .. })
    ));
}

#[test]
fn eliminated_gauge_row_participates_in_physical_stopping() {
    let grid = GridGeometry::new([4, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let op = PressureOperator::new(&grid, 1.0).unwrap();
    let mut ws = PressureWorkspace::new(&grid, 192).unwrap();
    let loose_linear = PressureSettings {
        relative_residual: 1.0,
        absolute_residual: 1.0,
        divergence_limit: 0.0015,
        max_iterations: 0,
    };
    assert!(matches!(
        ws.solve_rhs(
            &op,
            &[-0.003, 0.001, 0.001, 0.001],
            1.0,
            loose_linear,
            || false
        ),
        Err(PressureError::IterationLimit { iterations: 0, .. })
    ));
}

#[test]
fn manufactured_three_dimensional_pressure_recovers_modulo_gauge() {
    let grid = GridGeometry::new([5, 4, 3], [0.2, 0.25, 0.3], [0.0; 3]).unwrap();
    let op = PressureOperator::new(&grid, 1.3).unwrap();
    let exact: Vec<f64> = (0..grid.cell_len())
        .map(|i| (i as f64 * 0.7).sin())
        .collect();
    let mut rhs = vec![0.0; grid.cell_len()];
    op.apply_full(&exact, &mut rhs).unwrap();
    let mut ws = PressureWorkspace::new(&grid, grid.cell_len() * 48).unwrap();
    let report = ws.solve_rhs(&op, &rhs, 0.02, settings(), || false).unwrap();
    for (actual, expected) in ws.pressure().iter().zip(&exact) {
        assert!((actual - (expected - exact[0])).abs() < 1e-8);
    }
    assert!(report.predicted_divergence_max <= settings().divergence_limit);
}

#[test]
fn cancellation_and_iteration_exhaustion_are_explicit() {
    let grid = GridGeometry::new([3, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let op = PressureOperator::new(&grid, 1.0).unwrap();
    let mut ws = PressureWorkspace::new(&grid, 144).unwrap();
    assert!(matches!(
        ws.solve_rhs(&op, &[-4.0, 6.0, -2.0], 0.5, settings(), || true),
        Err(PressureError::Cancelled { iterations: 0 })
    ));
    let mut short = settings();
    short.max_iterations = 1;
    assert!(matches!(
        ws.solve_rhs(&op, &[-4.0, 6.0, -2.0], 0.5, short, || false),
        Err(PressureError::IterationLimit { iterations: 1, .. })
    ));
}

#[test]
fn workspace_budget_and_invalid_tolerance_fail() {
    let grid = GridGeometry::new([2; 3], [1.0; 3], [0.0; 3]).unwrap();
    assert!(matches!(
        PressureWorkspace::new(&grid, 383),
        Err(PressureError::BufferLimit { .. })
    ));
    let mut ws = PressureWorkspace::new(&grid, 384).unwrap();
    let op = PressureOperator::new(&grid, 1.0).unwrap();
    let mut invalid = settings();
    invalid.relative_residual = f64::NAN;
    assert_eq!(
        ws.solve_rhs(&op, &[0.0; 8], 1.0, invalid, || false)
            .unwrap_err(),
        PressureError::InvalidSettings
    );
}

#[test]
fn random_three_dimensional_projection_checks_actual_f32_divergence() {
    use rheon::Axis;
    let grid = GridGeometry::new([8, 7, 6], [0.2, 0.3, 0.4], [0.0; 3]).unwrap();
    let op = PressureOperator::new(&grid, 1.0).unwrap();
    let axes = [Axis::X, Axis::Y, Axis::Z];
    let mut fields: [Vec<f32>; 3] = std::array::from_fn(|d| vec![0.0; grid.face_len(axes[d])]);
    let mut seed = 0x1234_5678_u64;
    for (d, field) in fields.iter_mut().enumerate() {
        let shape = grid.face_counts(axes[d]);
        for k in 0..shape[2] {
            for j in 0..shape[1] {
                for i in 0..shape[0] {
                    let coordinate = [i, j, k];
                    if coordinate[d] > 0 && coordinate[d] < grid.counts()[d] {
                        seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                        let value = ((seed >> 40) as f64 / ((1_u64 << 24) as f64) - 0.5) as f32;
                        field[grid.face_index(axes[d], coordinate).unwrap()] = value;
                    }
                }
            }
        }
    }
    let old = [&fields[0][..], &fields[1][..], &fields[2][..]];
    let mut ws = PressureWorkspace::new(&grid, 48 * grid.cell_len()).unwrap();
    let report = ws
        .solve_velocity(&op, old, 0.02, settings(), || false)
        .unwrap();
    assert!(report.predicted_divergence_max < 1e-9);
    let mut output: [Vec<f32>; 3] = std::array::from_fn(|d| vec![0.0; grid.face_len(axes[d])]);
    let [ref mut x, ref mut y, ref mut z] = output;
    op.correct_velocity(old, ws.pressure(), 0.02, [x, y, z])
        .unwrap();
    let mut divergence = vec![0.0; grid.cell_len()];
    op.divergence([&output[0], &output[1], &output[2]], &mut divergence)
        .unwrap();
    let maximum = divergence.iter().fold(0.0_f64, |a, b| a.max(b.abs()));
    assert!(maximum < 1e-5, "actual rounded divergence {maximum}");
    let before: f64 = fields.iter().flatten().map(|&v| f64::from(v).powi(2)).sum();
    let after: f64 = output.iter().flatten().map(|&v| f64::from(v).powi(2)).sum();
    assert!(after <= before + 1e-6);
}

#[test]
fn a_rectangular_cycle_retains_circulation() {
    let grid = GridGeometry::new([2, 2, 1], [1.0; 3], [0.0; 3]).unwrap();
    let op = PressureOperator::new(&grid, 1.0).unwrap();
    let x = [0.0_f32, 1.0, 0.0, 0.0, -1.0, 0.0];
    let y = [0.0_f32, 0.0, -1.0, 1.0, 0.0, 0.0];
    let z = [0.0_f32; 8];
    let mut ws = PressureWorkspace::new(&grid, 4 * 48).unwrap();
    let report = ws
        .solve_velocity(&op, [&x, &y, &z], 0.1, settings(), || false)
        .unwrap();
    assert_eq!(report.iterations, 0);
    let mut ox = [0.0_f32; 6];
    let mut oy = [0.0_f32; 6];
    let mut oz = [0.0_f32; 8];
    op.correct_velocity(
        [&x, &y, &z],
        ws.pressure(),
        0.1,
        [&mut ox, &mut oy, &mut oz],
    )
    .unwrap();
    assert_eq!(ox, x);
    assert_eq!(oy, y);
    assert_eq!(oz, z);
}
