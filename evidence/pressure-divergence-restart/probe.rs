use rheon::{GridGeometry, PressureOperator, PressureSettings, PressureWorkspace};

#[test]
fn pressure_restart_probe() {
    let grid = GridGeometry::new([5, 4, 3], [0.2, 0.25, 0.3], [0.0; 3]).unwrap();
    let operator = PressureOperator::new(&grid, 1.3).unwrap();
    let pressure: Vec<f64> = (0..grid.cell_len())
        .map(|i| ((i * 17) % 31) as f64 / 31.0)
        .collect();
    let mut rhs = vec![0.0; grid.cell_len()];
    operator.apply_full(&pressure, &mut rhs).unwrap();
    for relative_residual in [1e-3, 1e-6, 1e-11] {
        println!("CASE relative_residual={relative_residual:e}");
        let mut workspace = PressureWorkspace::new(&grid, 48 * grid.cell_len()).unwrap();
        let settings = PressureSettings {
            relative_residual,
            absolute_residual: 1e-12,
            divergence_limit: 1e-9,
            max_iterations: 120,
        };
        let result = workspace.solve_rhs(&operator, &rhs, 0.02, settings, || false);
        println!("RESULT {result:?}");
    }
}
