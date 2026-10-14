//! Headless qualified pressure stage; optionally supply a verified model export.
//! The caller still owns velocity correction and actual-divergence acceptance.
use rheon::{solver_provider::LabModalPressure, *};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() > 1 {
        return Err("usage: pressure_session [VERIFIED_MODEL_EXPORT]".into());
    }
    let mut model = if let Some(path) = args.first() {
        Some(LabModalPressure::from_export(&std::fs::read(path)?).map_err(|e| format!("{e:?}"))?)
    } else {
        None
    };
    let grid = GridGeometry::new([32; 3], [1.0 / 32.0; 3], [0.0; 3])?;
    // Authored RHS: a simple pressure mode passed through the real full operator.
    let expected: Vec<_> = (0..grid.cell_len())
        .map(|i| (std::f64::consts::PI * ((i % 32) as f64 + 0.5) / 32.0).cos())
        .collect();
    let mut rhs = vec![0.0; grid.cell_len()];
    PressureOperator::new(&grid, 1.25)?.apply_full(&expected, &mut rhs)?;
    let mut session = PressureSession::new(
        SessionIdentity {
            run_id: 1,
            source_revision: "authored-pressure-example".into(),
        },
        grid,
        16 * 1024 * 1024,
        PressureImplementation::JacobiPcgV1,
        1,
    )?;
    let controls = PressureSolveControls {
        source_id: 1,
        source_version: 0,
        density: 1.25,
        dt: 1.0 / 60.0,
        settings: PressureSettings {
            relative_residual: 1e-8,
            absolute_residual: 1e-12,
            divergence_limit: 1e-7,
            max_iterations: 3000,
        },
    };
    let report = session.solve(
        &rhs,
        controls,
        model
            .as_mut()
            .map(|m| m as &mut dyn solver_provider::PressureProvider),
        None,
        || false,
    )?;
    let snapshot = session.snapshot().ok_or("missing qualified publication")?;
    println!(
        "stage={} selected={} generation={} residual_l2={:.17e} rejection={:?}",
        snapshot.stage(),
        report.selected,
        snapshot.generation(),
        report.accepted.true_residual_l2,
        report.rejection
    );
    if let Some(metadata) = &report.attempted {
        println!("attempted={metadata:?}");
    }
    session.stop();
    Ok(())
}
