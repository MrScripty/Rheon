// Retained sealed warm-start contract fixtures, unchanged from lab patch b851b38d.
// The current public owner implements the capability; no patch is applied at runtime.
use rheon::{
    Axis, GridGeometry, OperatorError, PressureError, PressureImplementation, PressureOperator,
    PressureSettings, PressureWorkspace, SlabFreeSurface, VolumeStamp,
};

fn settings() -> PressureSettings {
    PressureSettings {
        relative_residual: 1e-10,
        absolute_residual: 1e-12,
        divergence_limit: 1e-9,
        max_iterations: 1000,
    }
}

fn workspace(g: &GridGeometry, method: PressureImplementation) -> PressureWorkspace {
    PressureWorkspace::with_implementation(g, g.cell_len() * 48, method).unwrap()
}

#[test]
fn zero_guess_preserves_old_pressure_and_report_bits_for_both_methods() {
    let g = GridGeometry::new([5, 4, 3], [0.25, 0.3, 0.4], [0.0; 3]).unwrap();
    let op = PressureOperator::new(&g, 1.25).unwrap();
    let exact: Vec<_> = (0..g.cell_len()).map(|i| (i as f64 * 0.7).sin()).collect();
    let mut rhs = vec![0.0; g.cell_len()];
    op.apply_full(&exact, &mut rhs).unwrap();
    for method in PressureImplementation::ALL {
        let mut old = workspace(&g, method);
        let mut new = workspace(&g, method);
        let a = old
            .solve_rhs(&op, &rhs, 0.03125, settings(), || false)
            .unwrap();
        let b = new
            .solve_rhs_with_initial_pressure(
                &op,
                &rhs,
                &vec![0.0; rhs.len()],
                0.03125,
                settings(),
                || false,
            )
            .unwrap();
        assert_eq!(old.pressure(), new.pressure());
        assert_eq!(a.iterations, b.iterations);
        assert_eq!(a.true_residual_l2.to_bits(), b.true_residual_l2.to_bits());
        assert_eq!(a.true_residual_max.to_bits(), b.true_residual_max.to_bits());
        assert_eq!(
            a.predicted_divergence_max.to_bits(),
            b.predicted_divergence_max.to_bits()
        );
        assert_eq!(old.allocated_bytes(), new.allocated_bytes());
    }
}

#[test]
fn exact_guess_returns_at_zero_iterations_and_poor_guess_converges() {
    let g = GridGeometry::new([3, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let op = PressureOperator::new(&g, 1.0).unwrap();
    for method in PressureImplementation::ALL {
        let mut ws = workspace(&g, method);
        for (guess, exact) in [([0.0, 4.0, 2.0], true), ([0.0, -100.0, 33.0], false)] {
            let report = ws
                .solve_rhs_with_initial_pressure(
                    &op,
                    &[-4.0, 6.0, -2.0],
                    &guess,
                    0.5,
                    settings(),
                    || false,
                )
                .unwrap();
            assert_eq!(report.iterations == 0, exact);
            assert!(report.true_residual_l2 < 1e-9);
            assert!(report.predicted_divergence_max <= settings().divergence_limit);
            assert_eq!(ws.pressure()[0], 0.0);
            assert!((ws.pressure()[1] - 4.0).abs() < 1e-9);
            assert!((ws.pressure()[2] - 2.0).abs() < 1e-9);
        }
    }
}

#[test]
fn relative_tolerance_uses_original_rhs_not_initial_residual() {
    let g = GridGeometry::new([3, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let op = PressureOperator::new(&g, 1.0).unwrap();
    let limits = PressureSettings {
        relative_residual: 1e-3,
        absolute_residual: 0.0,
        divergence_limit: 1.0,
        max_iterations: 0,
    };
    for method in PressureImplementation::ALL {
        let report = workspace(&g, method)
            .solve_rhs_with_initial_pressure(
                &op,
                &[-4.0, 6.0, -2.0],
                &[0.0, 3.999, 1.999],
                0.5,
                limits,
                || false,
            )
            .unwrap();
        assert_eq!(report.iterations, 0);
        assert!(report.true_residual_l2 > 1e-3);
        assert!(report.true_residual_l2 < 1e-3 * 56.0_f64.sqrt());
    }
}

#[test]
fn initial_full_residual_keeps_eliminated_gauge_row_in_physical_gate() {
    let g = GridGeometry::new([4, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let op = PressureOperator::new(&g, 1.0).unwrap();
    // A*[0,1,2,3] plus a compatible residual whose gauge row alone fails.
    let rhs = [-1.003, 0.001, 0.001, 1.001];
    let limits = PressureSettings {
        relative_residual: 1.0,
        absolute_residual: 1.0,
        divergence_limit: 0.0015,
        max_iterations: 0,
    };
    for method in PressureImplementation::ALL {
        assert!(
            matches!(workspace(&g, method).solve_rhs_with_initial_pressure(
            &op, &rhs, &[0.0, 1.0, 2.0, 3.0], 1.0, limits, || false,
        ), Err(PressureError::IterationLimit { iterations: 0, residual_max }) if residual_max > 0.0029)
        );
    }
}

#[test]
fn invalid_guess_rhs_and_workspace_are_typed_failures() {
    let g = GridGeometry::new([3, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let op = PressureOperator::new(&g, 1.0).unwrap();
    for method in PressureImplementation::ALL {
        let mut ws = workspace(&g, method);
        for (guess, error) in [
            (
                vec![1.0, 5.0, 3.0],
                PressureError::InvalidInitialPressureGauge,
            ),
            (
                vec![0.0, f64::NAN, 2.0],
                OperatorError::NonFiniteInput.into(),
            ),
            (
                vec![0.0, f64::INFINITY, 2.0],
                OperatorError::NonFiniteInput.into(),
            ),
            (vec![0.0, 4.0], OperatorError::LengthMismatch.into()),
        ] {
            assert_eq!(
                ws.solve_rhs_with_initial_pressure(
                    &op,
                    &[-4.0, 6.0, -2.0],
                    &guess,
                    0.5,
                    settings(),
                    || false,
                )
                .unwrap_err(),
                error
            );
        }
        assert!(matches!(
            ws.solve_rhs_with_initial_pressure(
                &op,
                &[1.0, 0.0, 0.0],
                &[0.0; 3],
                0.5,
                settings(),
                || false,
            ),
            Err(PressureError::IncompatibleRhs { .. })
        ));
        assert_eq!(
            ws.solve_rhs_with_initial_pressure(
                &op,
                &[-4.0, 6.0, -2.0],
                &[0.0; 3],
                f64::NAN,
                settings(),
                || false,
            )
            .unwrap_err(),
            PressureError::InvalidSettings
        );
        let small = GridGeometry::new([2, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
        assert_eq!(
            workspace(&small, method)
                .solve_rhs_with_initial_pressure(
                    &op,
                    &[-4.0, 6.0, -2.0],
                    &[0.0; 3],
                    0.5,
                    settings(),
                    || false,
                )
                .unwrap_err(),
            PressureError::Operator(OperatorError::LengthMismatch)
        );
    }
}

#[test]
fn reuse_after_success_cancellation_and_iteration_failure_matches_fresh_workspace() {
    let g = GridGeometry::new([5, 4, 3], [0.25, 0.3, 0.4], [0.0; 3]).unwrap();
    let op = PressureOperator::new(&g, 1.25).unwrap();
    let p: Vec<_> = (0..g.cell_len()).map(|i| (i as f64 * 0.7).sin()).collect();
    let mut rhs = vec![0.0; g.cell_len()];
    op.apply_full(&p, &mut rhs).unwrap();
    let guess: Vec<_> = p.iter().map(|v| -10.0 * v).collect();
    for method in PressureImplementation::ALL {
        let mut reused = workspace(&g, method);
        let mut fresh = workspace(&g, method);
        let expected = fresh
            .solve_rhs_with_initial_pressure(&op, &rhs, &guess, 0.03125, settings(), || false)
            .unwrap();
        for failure in 0..4 {
            let mut limits = settings();
            let mut calls = 0;
            if failure == 2 {
                limits.max_iterations = 1;
            }
            let mut failed_rhs = rhs.clone();
            if failure == 3 {
                // Compatible finite input with an overflowing norm is a typed
                // numerical failure, followed by a fresh successful solve.
                failed_rhs.fill(0.0);
                failed_rhs[0] = -1e200;
                failed_rhs[1] = 1e200;
            }
            let result = reused.solve_rhs_with_initial_pressure(
                &op,
                &failed_rhs,
                &guess,
                0.03125,
                limits,
                || {
                    calls += 1;
                    failure == 0 || (failure == 1 && calls == 4)
                },
            );
            match failure {
                0 => assert!(matches!(
                    result,
                    Err(PressureError::Cancelled { iterations: 0 })
                )),
                1 => assert!(matches!(
                    result,
                    Err(PressureError::Cancelled { iterations: 2 })
                )),
                2 => assert!(matches!(
                    result,
                    Err(PressureError::IterationLimit { iterations: 1, .. })
                )),
                3 => assert!(matches!(
                    result,
                    Err(PressureError::Breakdown { iteration: 0 })
                )),
                _ => unreachable!(),
            }
            let actual = reused
                .solve_rhs_with_initial_pressure(&op, &rhs, &guess, 0.03125, settings(), || false)
                .unwrap();
            assert_eq!(actual.iterations, expected.iterations);
            assert_eq!(actual.true_residual_l2, expected.true_residual_l2);
            assert_eq!(reused.pressure(), fresh.pressure());
        }
    }
}

#[test]
fn warm_projection_passes_actual_rounded_f32_velocity_divergence() {
    let g = GridGeometry::new([5, 4, 3], [0.25, 0.3, 0.4], [0.0; 3]).unwrap();
    let op = PressureOperator::new(&g, 1.25).unwrap();
    let axes = [Axis::X, Axis::Y, Axis::Z];
    let mut velocity: [Vec<f32>; 3] = std::array::from_fn(|d| vec![0.0; g.face_len(axes[d])]);
    for (d, field) in velocity.iter_mut().enumerate() {
        let shape = g.face_counts(axes[d]);
        for k in 0..shape[2] {
            for j in 0..shape[1] {
                for i in 0..shape[0] {
                    let p = [i, j, k];
                    if p[d] > 0 && p[d] < g.counts()[d] {
                        let index = g.face_index(axes[d], p).unwrap();
                        field[index] = ((index * 17 + d * 3) % 31) as f32 / 31.0 - 0.5;
                    }
                }
            }
        }
    }
    let refs = [&velocity[0][..], &velocity[1][..], &velocity[2][..]];
    let mut rhs = vec![0.0; g.cell_len()];
    op.build_rhs(refs, 0.03125, &mut rhs).unwrap();
    let guess: Vec<_> = (0..g.cell_len()).map(|i| (i as f64 * 0.31).sin()).collect();
    for method in PressureImplementation::ALL {
        let mut old = workspace(&g, method);
        let old_report = old
            .solve_velocity(&op, refs, 0.03125, settings(), || false)
            .unwrap();
        let mut zero = workspace(&g, method);
        let zero_report = zero
            .solve_rhs_with_initial_pressure(
                &op,
                &rhs,
                &vec![0.0; rhs.len()],
                0.03125,
                settings(),
                || false,
            )
            .unwrap();
        assert_eq!(old.pressure(), zero.pressure());
        assert_eq!(old_report.iterations, zero_report.iterations);
        assert_eq!(
            old_report.true_residual_l2.to_bits(),
            zero_report.true_residual_l2.to_bits()
        );
        assert_eq!(
            old_report.true_residual_max.to_bits(),
            zero_report.true_residual_max.to_bits()
        );
        assert_eq!(
            old_report.predicted_divergence_max.to_bits(),
            zero_report.predicted_divergence_max.to_bits()
        );
        let mut ws = workspace(&g, method);
        ws.solve_rhs_with_initial_pressure(&op, &rhs, &guess, 0.03125, settings(), || false)
            .unwrap();
        let mut corrected = velocity.clone();
        let [x, y, z] = &mut corrected;
        op.correct_velocity(refs, ws.pressure(), 0.03125, [x, y, z])
            .unwrap();
        let div = ws
            .actual_divergence_max(&op, [&corrected[0], &corrected[1], &corrected[2]])
            .unwrap();
        assert!(div <= 1e-5, "{div}");
    }
}

#[test]
fn free_surface_is_explicitly_unsupported_by_new_entry_point() {
    let g = GridGeometry::new([3, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let surface =
        SlabFreeSurface::new(g.clone(), Axis::X, 2, VolumeStamp { id: 1, version: 0 }).unwrap();
    let op = PressureOperator::with_free_surface(&g, 1.0, &surface).unwrap();
    for method in PressureImplementation::ALL {
        assert_eq!(
            workspace(&g, method)
                .solve_rhs_with_initial_pressure(&op, &[0.0; 3], &[0.0; 3], 1.0, settings(), || {
                    false
                },)
                .unwrap_err(),
            PressureError::UnsupportedInitialPressureDomain
        );
        assert_eq!(
            workspace(&g, method)
                .solve_rhs(&op, &[0.0; 3], 1.0, settings(), || false)
                .unwrap()
                .iterations,
            0
        );
    }
}

#[test]
fn nonfinite_and_wrong_shape_rhs_are_rejected_before_warm_start() {
    let g = GridGeometry::new([3, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let op = PressureOperator::new(&g, 1.0).unwrap();
    for method in PressureImplementation::ALL {
        let mut ws = workspace(&g, method);
        for rhs in [
            vec![0.0, f64::NAN, 0.0],
            vec![0.0, f64::INFINITY, 0.0],
            vec![0.0, f64::NEG_INFINITY, 0.0],
        ] {
            assert_eq!(
                ws.solve_rhs_with_initial_pressure(&op, &rhs, &[0.0; 3], 0.5, settings(), || false)
                    .unwrap_err(),
                PressureError::Operator(OperatorError::NonFiniteInput)
            );
        }
        for rhs in [vec![], vec![0.0; 2], vec![0.0; 4]] {
            assert_eq!(
                ws.solve_rhs_with_initial_pressure(&op, &rhs, &[0.0; 3], 0.5, settings(), || false)
                    .unwrap_err(),
                PressureError::Operator(OperatorError::LengthMismatch)
            );
        }
    }
}

#[test]
fn single_cell_zero_warm_start_preserves_zero_iteration_reports() {
    let g = GridGeometry::new([1; 3], [1.0; 3], [0.0; 3]).unwrap();
    let op = PressureOperator::new(&g, 1.0).unwrap();
    for method in PressureImplementation::ALL {
        let mut ws = workspace(&g, method);
        let report = ws
            .solve_rhs_with_initial_pressure(&op, &[0.0], &[0.0], 0.5, settings(), || false)
            .unwrap();
        assert_eq!(report.iterations, 0);
        assert_eq!(report.true_residual_l2.to_bits(), 0.0_f64.to_bits());
        assert_eq!(report.true_residual_max.to_bits(), 0.0_f64.to_bits());
        assert_eq!(report.predicted_divergence_max.to_bits(), 0.0_f64.to_bits());
        assert_eq!(ws.pressure(), &[0.0]);
    }
}
