//! Accepted fixed-volume steps and the declared clamped appearance policy.
use rheon::{
    BodyForce, BoxFluxStamp, BoxFluxStepBoundary, BoxFluxStepWorkspace, BoxFluxTracerPolicy,
    ForceRegion, ForceUnits, GridGeometry, PrescribedBoxFlux, PressureImplementation,
    PressureSettings, Simulation, SimulationConfig, SmokeSource,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "implementation,step,time,boundary_version,kinetic_old,kinetic_accepted,advection_change,smoke_work,external_work,boundary_work,correction_energy,divergence_work,correction_work,budget_error,volume_imbalance,tracer_transport_change,tracer_source_change,actual_divergence,simulation_array_bytes,boundary_array_bytes"
    );
    for method in PressureImplementation::ALL {
        let grid = GridGeometry::new([3, 2, 1], [1.0; 3], [0.0; 3])?;
        let config = SimulationConfig {
            density: 2.0,
            memory_limit: 1024 * 1024,
            pressure: PressureSettings {
                relative_residual: 1e-12,
                absolute_residual: 1e-12,
                divergence_limit: 1e-9,
                max_iterations: 1000,
            },
            actual_divergence_limit: 1e-5,
            max_courant: 1.0,
        };
        let mut workspace =
            BoxFluxStepWorkspace::new(grid.clone(), config.density, 1024 * 1024, method)?;
        let mut sim = Simulation::with_implementation(grid, config, method)?;
        let seed = SmokeSource {
            lower: [0.0; 3],
            upper: [1.0, 2.0, 1.0],
            tracer_rate: 1.0,
            vertical_acceleration: 0.0,
        };
        let force = BodyForce {
            value: [0.0, 0.5, 0.0],
            units: ForceUnits::Acceleration,
            region: Some(ForceRegion {
                lower: [0.0; 3],
                upper: [1.0, 2.0, 1.0],
            }),
        };
        for step in 0..=4 {
            let speed = if step == 0 { 0.0 } else { 0.25 };
            let boundary = BoxFluxStepBoundary {
                flux: PrescribedBoxFlux::new(
                    BoxFluxStamp {
                        id: 73,
                        version: step,
                    },
                    [[-speed, speed], [0.0; 2], [0.0; 2]],
                )?,
                tracer: BoxFluxTracerPolicy::ClampedAppearance,
            };
            let report = sim.step_with_box_flux(
                if step == 0 { 0.5 } else { 0.25 },
                (step == 0).then_some(seed),
                if step == 0 {
                    &[]
                } else {
                    std::slice::from_ref(&force)
                },
                &mut workspace,
                boundary,
                |_| false,
            )?;
            let w = report.work;
            let p = report.projection.work;
            println!(
                "{},{step},{:.17e},{},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{},{}",
                method.id(),
                report.step.step.time,
                report.projection.boundary.version,
                w.kinetic_old,
                w.kinetic_accepted,
                w.advection_change,
                w.smoke_force_work,
                w.external_force_work,
                p.boundary_pressure_work,
                p.correction_energy,
                p.divergence_residual_work,
                p.correction_residual_work,
                w.budget_error,
                w.boundary_volume_imbalance,
                w.tracer_transport_change,
                w.tracer_source_change,
                report.step.step.actual_divergence_max,
                report.simulation_array_bytes,
                report.boundary_workspace_array_bytes
            );
        }
    }
    eprintln!(
        "Clamped appearance transport; boundary volume balance does not certify tracer conservation."
    );
    Ok(())
}
