use rheon::solver_provider::context::*;
use rheon::{GridGeometry, OperatorError};

#[test]
fn residual_has_authoritative_integrated_sign_and_no_invented_history() {
    let grid = GridGeometry::new([2, 1, 1], [2.0, 3.0, 4.0], [5.0, 6.0, 7.0]).unwrap();
    let rhs = [12.0, -12.0];
    let make = |prior| PressureContextInput {
        grid: &grid,
        density: 3.0,
        dt: 0.25,
        interval_start: 0.25,
        rhs: &rhs,
        prior,
        run_id: "track-A",
        generation: 2,
        mode: PressureContextMode::DiagnosticCandidate,
    };
    let pressure = [0.0, 4.0];
    let context = PressureContext::new(make(Some(PressureHistory {
        values: &pressure,
        grid: &grid,
        density: 3.0,
        run_id: "track-A",
        generation: 1,
        interval_end: 0.25,
        status: PressureHistoryStatus::Diagnostic { qualified: false },
    })))
    .unwrap();
    let mut residual = [0.0; 2];
    context.initial_residual(&mut residual).unwrap();
    // Independent two-cell stencil: area/(rho*dx)=12/(3*2)=2;
    // A*[0,4]=[-8,8], hence b-A*p0=[20,-20].
    assert_eq!(residual, [20.0, -20.0]);
    assert_eq!(context.grid().origin(), [5.0, 6.0, 7.0]);
    assert_eq!(context.interval_end(), 0.5);
    assert!(matches!(
        PressureContext::new(make(None)),
        Err(PressureContextError::InvalidLineage)
    ));
    let initial = PressureContext::new(PressureContextInput {
        prior: None,
        generation: 1,
        interval_start: 0.0,
        ..make(None)
    })
    .unwrap();
    initial.initial_residual(&mut residual).unwrap();
    assert_eq!(residual, rhs);
    assert!(initial.prior().is_none());
}

#[test]
fn rejected_cross_track_stale_and_diagnostic_accepted_history() {
    let grid = GridGeometry::new([2, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let rhs = [1.0, -1.0];
    let previous = [0.0, 1.0];
    let prior = PressureHistory {
        values: &previous,
        grid: &grid,
        density: 1000.0,
        run_id: "A",
        generation: 1,
        interval_end: 0.1,
        status: PressureHistoryStatus::Diagnostic { qualified: false },
    };
    let make = |prior| PressureContextInput {
        grid: &grid,
        density: 1000.0,
        dt: 0.1,
        interval_start: 0.1,
        rhs: &rhs,
        prior: Some(prior),
        run_id: "A",
        generation: 2,
        mode: PressureContextMode::AcceptedCandidate,
    };
    assert!(matches!(
        PressureContext::new(make(prior)),
        Err(PressureContextError::DiagnosticHistoryInAcceptedContext)
    ));
    for wrong in [
        PressureHistory {
            run_id: "B",
            ..prior
        },
        PressureHistory {
            generation: 0,
            ..prior
        },
        PressureHistory {
            interval_end: 0.0,
            ..prior
        },
    ] {
        assert!(matches!(
            PressureContext::new(make(wrong)),
            Err(PressureContextError::InvalidLineage)
        ));
    }
    assert!(
        PressureContext::new(make(PressureHistory {
            status: PressureHistoryStatus::Accepted,
            ..prior
        }))
        .is_ok()
    );
    assert!(matches!(
        PressureContext::new(PressureContextInput {
            dt: f64::NAN,
            ..make(prior)
        }),
        Err(PressureContextError::InvalidInterval)
    ));
    assert!(matches!(
        PressureContext::new(PressureContextInput {
            rhs: &[f64::NAN, 0.0],
            ..make(prior)
        }),
        Err(PressureContextError::Operator(
            OperatorError::NonFiniteInput
        ))
    ));
    let other_grid = GridGeometry::new([1, 2, 1], [1.0; 3], [0.0; 3]).unwrap();
    for wrong in [
        PressureHistory {
            grid: &other_grid,
            ..prior
        },
        PressureHistory {
            density: 1001.0,
            ..prior
        },
    ] {
        assert!(matches!(
            PressureContext::new(make(wrong)),
            Err(PressureContextError::IncompatibleHistoryDomain)
        ));
    }
    assert!(matches!(
        PressureContext::new(PressureContextInput {
            generation: 1,
            interval_start: 0.0,
            prior: Some(PressureHistory {
                generation: 0,
                interval_end: 0.0,
                status: PressureHistoryStatus::Accepted,
                ..prior
            }),
            ..make(prior)
        }),
        Err(PressureContextError::InvalidLineage)
    ));
}

#[test]
fn explicit_indexed_endpoints_keep_exact_history_without_accumulated_clock_drift() {
    let grid = GridGeometry::new([2, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let zero = [0.0; 2];
    let dt = 1.0 / 60.0;
    let mut previous_end: f64 = 0.0;
    let mut accumulated: f64 = 0.0;
    let mut observed_rounding = false;
    for generation in 1..=10_000_u64 {
        let start = (generation - 1) as f64 * dt;
        let end = generation as f64 * dt;
        assert_eq!(start.to_bits(), previous_end.to_bits());
        let context = PressureContext::with_interval_end(
            PressureContextInput {
                grid: &grid,
                density: 1000.0,
                dt,
                interval_start: start,
                rhs: &zero,
                run_id: "indexed-clock",
                generation,
                mode: PressureContextMode::DiagnosticCandidate,
                prior: (generation > 1).then_some(PressureHistory {
                    values: &zero,
                    grid: &grid,
                    density: 1000.0,
                    run_id: "indexed-clock",
                    generation: generation - 1,
                    interval_end: previous_end,
                    status: PressureHistoryStatus::Diagnostic { qualified: true },
                }),
            },
            end,
        )
        .unwrap();
        observed_rounding |= (start + dt).to_bits() != end.to_bits();
        assert_eq!(context.interval_start().to_bits(), start.to_bits());
        assert_eq!(context.interval_end().to_bits(), end.to_bits());
        assert_eq!(context.dt().to_bits(), dt.to_bits());
        previous_end = context.interval_end();
        accumulated += dt;
    }
    assert!(
        observed_rounding,
        "fixture must exercise indexed endpoint rounding"
    );
    assert_ne!(accumulated.to_bits(), previous_end.to_bits());
}

#[test]
fn explicit_end_rejects_material_interval_errors_and_never_relaxes_prior_alignment() {
    let grid = GridGeometry::new([2, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let zero = [0.0; 2];
    let make = || PressureContextInput {
        grid: &grid,
        density: 1000.0,
        dt: 0.25,
        interval_start: 0.25,
        rhs: &zero,
        run_id: "endpoint-negatives",
        generation: 2,
        mode: PressureContextMode::DiagnosticCandidate,
        prior: Some(PressureHistory {
            values: &zero,
            grid: &grid,
            density: 1000.0,
            run_id: "endpoint-negatives",
            generation: 1,
            interval_end: 0.25,
            status: PressureHistoryStatus::Diagnostic { qualified: true },
        }),
    };
    for end in [
        0.25,
        0.75,
        f64::NAN,
        f64::INFINITY,
        f64::from_bits(0.5_f64.to_bits() + 2),
    ] {
        assert!(matches!(
            PressureContext::with_interval_end(make(), end),
            Err(PressureContextError::InvalidInterval)
        ));
    }
    let mut input = make();
    input.prior.as_mut().unwrap().interval_end = f64::from_bits(0.25_f64.to_bits() + 1);
    assert!(matches!(
        PressureContext::with_interval_end(input, 0.5),
        Err(PressureContextError::InvalidLineage)
    ));
}
