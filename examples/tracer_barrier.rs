//! Passive tracer barriers in accepted fixed-box steps, not fluid-wall pressure.
use rheon::{
    BodyForce, ForceRegion, ForceUnits, GridGeometry, PressureImplementation, PressureSettings,
    Simulation, SimulationConfig, SmokeSource, SurfaceSettings, SurfaceStamp, TriangleSurface,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sheet = TriangleSurface::new(
        SurfaceStamp { id: 12, version: 0 },
        vec![[1.0, -16.0, -16.0], [1.0, 64.0, -16.0], [1.0, -16.0, 64.0]],
        vec![[0, 1, 2]],
        SurfaceSettings::default(),
    )?;
    let config = SimulationConfig {
        density: 1.0,
        memory_limit: 1024 * 1024,
        pressure: PressureSettings {
            relative_residual: 1e-11,
            absolute_residual: 1e-12,
            divergence_limit: 1e-9,
            max_iterations: 1000,
        },
        actual_divergence_limit: 1e-6,
        max_courant: 1.0,
    };
    let source = SmokeSource {
        lower: [0.0; 3],
        upper: [1.0, 2.0, 1.0],
        tracer_rate: 1.0,
        vertical_acceleration: 0.0,
    };
    let force = BodyForce {
        value: [2.0, 0.0, 0.0],
        units: ForceUnits::Acceleration,
        region: Some(ForceRegion {
            lower: [0.0; 3],
            upper: [2.0, 1.0, 1.0],
        }),
    };
    println!(
        "implementation,step,time,legacy_right,barrier_right,reverted,blocked_donors,divergence_max,simulation_array_bytes,surface_array_bytes"
    );
    for method in PressureImplementation::ALL {
        let grid = GridGeometry::new([2, 2, 1], [1.0; 3], [0.0; 3])?;
        let mut legacy = Simulation::with_implementation(grid.clone(), config, method)?;
        let mut barrier = Simulation::with_implementation(grid, config, method)?;
        legacy.step(0.1, Some(source), |_| false)?;
        barrier.step(0.1, Some(source), |_| false)?;
        for step in 1..=8 {
            legacy.step_with_forces(0.125, None, &[force], |_| false)?;
            let report =
                barrier.step_with_tracer_barrier(0.125, None, &[force], Some(&sheet), |_| false)?;
            let tracer = report
                .tracer_barrier
                .expect("requested barrier has accepted diagnostics");
            println!(
                "{},{step},{:.17e},{:.17e},{:.17e},{},{},{:.17e},{},{}",
                method.id(),
                report.step.step.time,
                legacy.state().tracer[1],
                barrier.state().tracer[1],
                tracer.reverted_traces,
                tracer.blocked_donors,
                report.step.step.actual_divergence_max,
                barrier.allocated_bytes(),
                sheet.allocated_bytes()
            );
        }
    }
    eprintln!("Static passive-concentration barrier; velocity and pressure use the fixed box.");
    Ok(())
}
