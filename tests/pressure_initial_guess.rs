//! Hand-assembled chain oracles; no production operator assembles these RHSs.
use rheon::{
    GridGeometry, PressureError, PressureImplementation, PressureOperator, PressureSettings,
    PressureWorkspace,
};

fn settings() -> PressureSettings {
    PressureSettings {
        relative_residual: 1e-11,
        absolute_residual: 1e-12,
        divergence_limit: 1e-9,
        max_iterations: 1000,
    }
}

#[test]
fn exact_and_approximate_guesses_recover_independent_chain_without_extra_arrays() {
    let grid = GridGeometry::new([3, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let op = PressureOperator::new(&grid, 1.0).unwrap();
    for method in PressureImplementation::ALL {
        let mut ws = PressureWorkspace::with_implementation(&grid, usize::MAX, method).unwrap();
        let bytes = ws.allocated_bytes();
        let report = ws
            .solve_rhs_with_initial_pressure(
                &op,
                &[-4.0, 6.0, -2.0],
                &[0.0, 4.0, 2.0],
                0.5,
                settings(),
                || false,
            )
            .unwrap();
        assert_eq!(report.iterations, 0);
        assert_eq!(report.true_residual_l2, 0.0);
        let report = ws
            .solve_rhs_with_initial_pressure(
                &op,
                &[-4.0, 6.0, -2.0],
                &[0.0, 3.0, -1.0],
                0.125,
                settings(),
                || false,
            )
            .unwrap();
        assert!(report.iterations > 0);
        for (p, exact) in ws.pressure().iter().zip([0.0, 4.0, 2.0]) {
            assert!((p - exact).abs() < 1e-10);
        }
        assert_eq!(bytes, ws.allocated_bytes());
    }
}

#[test]
fn zero_rhs_still_checks_seed_and_current_rhs_threshold() {
    let grid = GridGeometry::new([3, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let op = PressureOperator::new(&grid, 1.0).unwrap();
    let mut ws = PressureWorkspace::new(&grid, usize::MAX).unwrap();
    let no_iterations = PressureSettings {
        relative_residual: 1.0,
        absolute_residual: 0.0,
        divergence_limit: 100.0,
        max_iterations: 0,
    };
    assert!(matches!(
        ws.solve_rhs_with_initial_pressure(
            &op,
            &[0.0; 3],
            &[0.0, 4.0, 2.0],
            1.0,
            no_iterations,
            || false
        ),
        Err(PressureError::IterationLimit { iterations: 0, .. })
    ));
    let report = ws
        .solve_rhs_with_initial_pressure(&op, &[0.0; 3], &[0.0, 4.0, 2.0], 1.0, settings(), || {
            false
        })
        .unwrap();
    assert!(report.iterations > 0);
    assert!(ws.pressure().iter().all(|p| p.abs() < 1e-10));
}

#[test]
fn full_gauge_row_and_compatibility_are_not_hidden_by_guess() {
    let grid = GridGeometry::new([4, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let op = PressureOperator::new(&grid, 1.0).unwrap();
    let mut ws = PressureWorkspace::new(&grid, usize::MAX).unwrap();
    let loose = PressureSettings {
        relative_residual: 1.0,
        absolute_residual: 1.0,
        divergence_limit: 0.0015,
        max_iterations: 0,
    };
    assert!(matches!(
        ws.solve_rhs_with_initial_pressure(
            &op,
            &[-0.003, 0.001, 0.001, 0.001],
            &[0.0; 4],
            1.0,
            loose,
            || false
        ),
        Err(PressureError::IterationLimit { .. })
    ));
    assert!(matches!(
        ws.solve_rhs_with_initial_pressure(
            &op,
            &[1.0, 0.0, 0.0, 0.0],
            &[0.0; 4],
            1.0,
            settings(),
            || false
        ),
        Err(PressureError::IncompatibleRhs { .. })
    ));
}

#[test]
fn invalid_guess_and_cancelled_dirty_scratch_do_not_poison_retry() {
    let grid = GridGeometry::new([3, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let op = PressureOperator::new(&grid, 1.0).unwrap();
    let mut ws = PressureWorkspace::new(&grid, usize::MAX).unwrap();
    for seed in [
        &[0.0, 1.0][..],
        &[0.0, f64::NAN, 0.0],
        &[0.0, f64::INFINITY, 0.0],
        &[1.0, 4.0, 2.0],
    ] {
        assert!(
            ws.solve_rhs_with_initial_pressure(
                &op,
                &[-4.0, 6.0, -2.0],
                seed,
                0.5,
                settings(),
                || false
            )
            .is_err()
        );
    }
    let seed = [0.0, 3.0, -1.0];
    let mut calls = 0;
    assert!(matches!(
        ws.solve_rhs_with_initial_pressure(&op, &[-4.0, 6.0, -2.0], &seed, 0.5, settings(), || {
            calls += 1;
            calls == 3
        }),
        Err(PressureError::Cancelled { .. })
    ));
    let retry = ws
        .solve_rhs_with_initial_pressure(&op, &[-4.0, 6.0, -2.0], &seed, 0.5, settings(), || false)
        .unwrap();
    let mut clean = PressureWorkspace::new(&grid, usize::MAX).unwrap();
    let report = clean
        .solve_rhs_with_initial_pressure(&op, &[-4.0, 6.0, -2.0], &seed, 0.5, settings(), || false)
        .unwrap();
    assert_eq!(retry.iterations, report.iterations);
    assert_eq!(
        retry.true_residual_l2.to_bits(),
        report.true_residual_l2.to_bits()
    );
    assert_eq!(ws.pressure(), clean.pressure());
    let cold = ws
        .solve_rhs(&op, &[-4.0, 6.0, -2.0], 0.5, settings(), || false)
        .unwrap();
    let reference = clean
        .solve_rhs(&op, &[-4.0, 6.0, -2.0], 0.5, settings(), || false)
        .unwrap();
    assert_eq!(cold.iterations, reference.iterations);
    assert_eq!(ws.pressure(), clean.pressure());
}
