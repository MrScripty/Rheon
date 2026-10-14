use rheon::*;
use std::time::Instant;
fn grid() -> GridGeometry {
    GridGeometry::new([4; 3], [0.25; 3], [0.0; 3]).unwrap()
}
fn controls() -> PressureSolveControls {
    PressureSolveControls {
        source_id: 17,
        source_version: 2,
        density: 1.25,
        dt: 1.0 / 60.0,
        settings: PressureSettings {
            relative_residual: 1e-8,
            absolute_residual: 1e-12,
            divergence_limit: 1e-7,
            max_iterations: 3000,
        },
    }
}
fn session(grid: GridGeometry, max: usize) -> PressureSession {
    PressureSession::new(
        SessionIdentity {
            run_id: 51,
            source_revision: "authored-pressure-test".into(),
        },
        grid,
        16 * 1024 * 1024,
        PressureImplementation::JacobiPcgV1,
        max,
    )
    .unwrap()
}
fn rhs(grid: &GridGeometry) -> Vec<f64> {
    let values: Vec<_> = (0..grid.cell_len())
        .map(|i| (i as f64 * 0.3).sin())
        .collect();
    let mut rhs = vec![0.0; grid.cell_len()];
    PressureOperator::new(grid, 1.25)
        .unwrap()
        .apply_full(&values, &mut rhs)
        .unwrap();
    rhs
}
#[test]
fn numerical_selection_publishes_qualified_stage_and_preserves_on_rejection() {
    let grid = grid();
    let rhs = rhs(&grid);
    let mut session = session(grid.clone(), 3);
    assert!(session.snapshot().is_none());
    assert!(!session.capabilities().full_step_acceptance);
    let report = session
        .solve(&rhs, controls(), None, None, || false)
        .unwrap();
    assert_eq!(report.selected, PressureImplementation::JacobiPcgV1.id());
    assert!(report.attempted.is_none());
    let snapshot = session.snapshot().unwrap();
    assert_eq!(snapshot.stage(), "qualified-sealed-box-pressure");
    assert_eq!(snapshot.generation(), 1);
    assert_eq!(snapshot.controls().source_version, 2);
    let saved = snapshot.pressure_pa().to_vec();
    let mut product = vec![0.0; rhs.len()];
    PressureOperator::new(&grid, 1.25)
        .unwrap()
        .apply_full(&saved, &mut product)
        .unwrap();
    let max = product
        .iter()
        .zip(&rhs)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0, f64::max);
    assert!(max <= report.accepted.true_residual_max + 1e-12);
    let mut invalid = controls();
    invalid.source_version = 99;
    invalid.dt = f64::NAN;
    assert!(session.solve(&rhs, invalid, None, None, || false).is_err());
    assert!(matches!(
        session.solve(&rhs, controls(), None, Some(Instant::now()), || false),
        Err(PressureSessionError::Deadline)
    ));
    assert!(
        session
            .solve(&rhs, controls(), None, None, || true)
            .is_err()
    );
    let snapshot = session.snapshot().unwrap();
    assert_eq!(snapshot.pressure_pa(), saved);
    assert_eq!(snapshot.controls().source_version, 2);
    assert_eq!(snapshot.generation(), 1);
    session
        .solve(&rhs, controls(), None, None, || false)
        .unwrap();
    assert_eq!(session.snapshot().unwrap().generation(), 2);
    session.stop();
    assert!(matches!(
        session.solve(&rhs, controls(), None, None, || false),
        Err(PressureSessionError::Stopped)
    ));
}
#[test]
fn final_session_checkpoint_can_cancel_without_replacing_publication() {
    let rhs = rhs(&grid());
    let mut session = session(grid(), 4);
    let mut visits = 0;
    session
        .solve(&rhs, controls(), None, None, || {
            visits += 1;
            false
        })
        .unwrap();
    let before = session.snapshot().unwrap().pressure_pa().to_vec();
    let mut count = 0;
    assert!(matches!(
        session.solve(&rhs, controls(), None, None, || {
            count += 1;
            count == visits
        }),
        Err(PressureSessionError::Pressure(
            PressureError::Cancelled { .. }
        ))
    ));
    assert_eq!(count, visits);
    assert_eq!(session.snapshot().unwrap().pressure_pa(), before);
    assert_eq!(session.snapshot().unwrap().generation(), 1);
    session
        .solve(&rhs, controls(), None, None, || false)
        .unwrap();
}
#[test]
fn construction_and_solve_quota_are_bounded() {
    let id = || SessionIdentity {
        run_id: 1,
        source_revision: "test".into(),
    };
    assert!(matches!(
        PressureSession::new(id(), grid(), 1, PressureImplementation::JacobiPcgV1, 1),
        Err(PressureSessionError::Pressure(
            PressureError::BufferLimit { .. }
        ))
    ));
    let mut session = session(grid(), 0);
    assert!(matches!(
        session.solve(&rhs(&grid()), controls(), None, None, || false),
        Err(PressureSessionError::SolveLimit)
    ));
    assert!(session.snapshot().is_none());
}
#[test]
#[ignore = "requires RHEON_AI_EXPORT from the SHA256-verified frozen checkpoint exporter"]
fn frozen_model_selection_and_unsupported_domain_fallback_are_visible() {
    use rheon::solver_provider::LabModalPressure;
    let bytes =
        std::fs::read(std::env::var("RHEON_AI_EXPORT").expect("verified model export path"))
            .unwrap();
    let mut model = LabModalPressure::from_export(&bytes).unwrap();
    let large = GridGeometry::new([32; 3], [1.0 / 32.0; 3], [0.0; 3]).unwrap();
    let mut session = session(large.clone(), 2);
    // Nonzero smooth mode exercises real inference at the declared tolerances.
    let mode: Vec<_> = (0..large.cell_len())
        .map(|i| (std::f64::consts::PI * ((i % 32) as f64 + 0.5) / 32.0).cos())
        .collect();
    let mut smooth_rhs = vec![0.0; large.cell_len()];
    PressureOperator::new(&large, 1.25)
        .unwrap()
        .apply_full(&mode, &mut smooth_rhs)
        .unwrap();
    let report = session
        .solve(&smooth_rhs, controls(), Some(&mut model), None, || false)
        .unwrap();
    assert_eq!(report.selected, "lab-modal-pressure-v1");
    assert!(report.rejection.is_none());
    assert!(
        session
            .snapshot()
            .unwrap()
            .report()
            .attempted
            .as_ref()
            .unwrap()
            .provenance
            .is_some()
    );
    assert_eq!(session.snapshot().unwrap().controls().source_id, 17);
    let prior = session.snapshot().unwrap().pressure_pa().to_vec();
    // This high-frequency authored request did not qualify and exhausted the
    // 3000-iteration fallback budget during initial acceptance. Preserve that
    // negative case rather than loosening thresholds or claiming universal fit.
    assert!(matches!(
        session.solve(&rhs(&large), controls(), Some(&mut model), None, || false),
        Err(PressureSessionError::Pressure(
            PressureError::IterationLimit { .. }
        ))
    ));
    assert_eq!(session.snapshot().unwrap().pressure_pa(), prior);
    assert_eq!(session.snapshot().unwrap().generation(), 1);
    let mut small = PressureSession::new(
        SessionIdentity {
            run_id: 52,
            source_revision: "test".into(),
        },
        grid(),
        1024 * 1024,
        PressureImplementation::SymmetricGaussSeidelPcgV1,
        1,
    )
    .unwrap();
    let report = small
        .solve(&rhs(&grid()), controls(), Some(&mut model), None, || false)
        .unwrap();
    assert_eq!(
        report.selected,
        PressureImplementation::SymmetricGaussSeidelPcgV1.id()
    );
    assert!(matches!(
        report.rejection,
        Some(rheon::solver_provider::ProviderRejection::Unsupported(_))
    ));
    assert_eq!(
        small
            .snapshot()
            .unwrap()
            .report()
            .attempted
            .as_ref()
            .unwrap()
            .id,
        "lab-modal-pressure-v1"
    );
}
