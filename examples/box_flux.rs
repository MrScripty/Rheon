//! Fixed-volume normal inlet/outlet projection with explicit pressure work.
use rheon::{
    Axis, BoxFluxSettings, BoxFluxStamp, BoxFluxWorkspace, GridGeometry, PrescribedBoxFlux,
    PressureImplementation, PressureSettings,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "implementation,speed,iterations,p_first,p_last,kinetic_before,kinetic_after,correction_energy,boundary_work,divergence_work,correction_work,budget_error,actual_divergence,array_bytes"
    );
    for method in PressureImplementation::ALL {
        for speed in [-0.25_f32, 0.0, 0.25] {
            let grid = GridGeometry::new([3, 1, 1], [1.0; 3], [0.0; 3])?;
            let old = [Axis::X, Axis::Y, Axis::Z].map(|a| vec![0.0; grid.face_len(a)]);
            let mut output = old.clone();
            let mut workspace = BoxFluxWorkspace::new(grid, 2.0, 1024, method)?;
            let boundary = PrescribedBoxFlux::new(
                BoxFluxStamp { id: 41, version: 0 },
                [[-speed, speed], [0.0; 2], [0.0; 2]],
            )?;
            let [x, y, z] = &mut output;
            let report = workspace.project(
                [&old[0], &old[1], &old[2]],
                0.5,
                boundary,
                BoxFluxSettings {
                    pressure: PressureSettings {
                        relative_residual: 1e-12,
                        absolute_residual: 1e-12,
                        divergence_limit: 1e-9,
                        max_iterations: 1000,
                    },
                    actual_divergence_limit: 1e-6,
                },
                [x, y, z],
                |_| false,
            )?;
            let w = report.work;
            println!(
                "{},{speed},{},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{}",
                method.id(),
                report.pressure.iterations,
                workspace.pressure()[0],
                workspace.pressure()[2],
                w.kinetic_before,
                w.kinetic_after,
                w.correction_energy,
                w.boundary_pressure_work,
                w.divergence_residual_work,
                w.correction_residual_work,
                w.budget_error,
                report.actual_divergence_max,
                workspace.allocated_bytes()
            );
        }
    }
    eprintln!(
        "Fixed rectangular control volume; prescribed normal speeds; standalone projection scratch."
    );
    Ok(())
}
