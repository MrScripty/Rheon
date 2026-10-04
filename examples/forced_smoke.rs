//! Explicit force-density drive plus uniform gravity, without a GUI or export dependency.
use rheon::{
    BodyForce, ForceRegion, ForceUnits, GridGeometry, PressureImplementation, PressureSettings,
    Simulation, SimulationConfig, SmokeSource,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let method = match std::env::args().nth(1) {
        Some(id) => {
            PressureImplementation::from_id(&id).ok_or("unknown pressure implementation")?
        }
        None => PressureImplementation::default(),
    };
    let density = 1.2;
    let mut simulation = Simulation::with_implementation(
        GridGeometry::new([16; 3], [1.0 / 16.0; 3], [0.0; 3])?,
        SimulationConfig {
            density,
            memory_limit: 8 * 1024 * 1024,
            pressure: PressureSettings {
                relative_residual: 1e-9,
                absolute_residual: 1e-12,
                divergence_limit: 1e-7,
                max_iterations: 2000,
            },
            actual_divergence_limit: 1e-5,
            max_courant: 1.0,
        },
        method,
    )?;
    let forces = [
        BodyForce {
            value: [0.0, -9.81, 0.0],
            units: ForceUnits::Acceleration,
            region: None,
        },
        BodyForce {
            value: [0.6, 1.2, -0.3],
            units: ForceUnits::ForceDensity,
            region: Some(ForceRegion {
                lower: [0.3, 0.1, 0.3],
                upper: [0.7, 0.4, 0.7],
            }),
        },
    ];
    let source = SmokeSource {
        lower: [0.35, 0.08, 0.35],
        upper: [0.65, 0.3, 0.65],
        tracer_rate: 2.0,
        vertical_acceleration: 0.0,
    };
    println!(
        "step,time,dt,external_force_work,stage_energy_before,stage_energy_after,accepted_energy,divergence,pressure_iterations"
    );
    for step in 0..10 {
        let report = simulation.step_with_forces(0.02, Some(source), &forces, |_| false)?;
        let f = report.forces.expect("nonempty external forces");
        let r = report.step;
        println!(
            "{},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{}",
            step + 1,
            r.time,
            r.dt,
            f.applied_work,
            f.kinetic_energy_before,
            f.kinetic_energy_after,
            r.kinetic_energy,
            r.actual_divergence_max,
            r.pressure.iterations
        );
    }
    eprintln!(
        "{}; managed simulation arrays {} bytes; stationary fully filled box, passive smoke tracer",
        method.id(),
        simulation.allocated_bytes()
    );
    Ok(())
}
