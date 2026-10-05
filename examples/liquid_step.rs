//! Reproducible 3D slab transport with one public carrier/liquid publication.
//! Output is occupancy-integral guidance, not a reconstructed liquid surface.
use rheon::{
    Axis, BoxFluxStamp, BoxFluxStepBoundary, BoxFluxStepWorkspace, BoxFluxTracerPolicy,
    GridGeometry, LiquidInlet, LiquidStepInputs, LiquidTransportConfig, LiquidTransportSimulation,
    LiquidVolumeSettings, PrescribedBoxFlux, PressureImplementation, PressureSettings,
    SimulationConfig, VolumeStamp, write_liquid_guidance_png,
};
use std::{
    fs,
    io::{BufWriter, Write},
    path::Path,
};

fn scenario(
    output: &Path,
    n: usize,
    courant: f64,
    method: PressureImplementation,
) -> Result<(), Box<dyn std::error::Error>> {
    let name = format!(
        "{}-n{n}-c{}",
        method.id(),
        if courant == 0.25 { "025" } else { "0125" }
    );
    let directory = output.join(&name);
    fs::create_dir(&directory)?;
    let grid = GridGeometry::new(
        [u64::try_from(n)?, 8, 4],
        [1.0 / n as f64, 0.125, 0.25],
        [0.0; 3],
    )?;
    let mut fraction = vec![0.0; grid.cell_len()];
    for k in 0..4 {
        for j in 0..8 {
            for i in n / 4..n / 2 {
                fraction[grid.cell_index([i, j, k]).unwrap()] = 0.5;
            }
        }
    }
    let config = LiquidTransportConfig {
        carrier: SimulationConfig {
            density: 1000.0,
            memory_limit: 16 * 1024 * 1024,
            pressure: PressureSettings {
                relative_residual: 1e-12,
                absolute_residual: 1e-12,
                divergence_limit: 1e-9,
                max_iterations: 10000,
            },
            actual_divergence_limit: 1e-5,
            max_courant: 1.0,
        },
        represented_density: 800.0,
        volume_stamp: VolumeStamp { id: 41, version: 0 },
        carrier_id: 43,
    };
    let mut simulation = LiquidTransportSimulation::new(grid.clone(), config, method, fraction)?;
    let mut workspace = BoxFluxStepWorkspace::new(
        grid.clone(),
        config.carrier.density,
        16 * 1024 * 1024,
        method,
    )?;
    let boundary = BoxFluxStepBoundary {
        flux: PrescribedBoxFlux::new(
            BoxFluxStamp { id: 47, version: 0 },
            [[-0.25, 0.25], [0.0; 2], [0.0; 2]],
        )?,
        tracer: BoxFluxTracerPolicy::ClampedAppearance,
    };
    let inlet = LiquidInlet::new(VolumeStamp { id: 53, version: 0 }, [[0.0; 2]; 3])?;
    let dt = courant * grid.spacing()[0] / 0.25;
    let steps = (0.5 / dt).round() as usize;
    write_liquid_guidance_png(
        &simulation,
        fs::File::create(directory.join("initial.png"))?,
        n * 8,
    )?;
    let mut csv = BufWriter::new(fs::File::create(directory.join("steps.csv"))?);
    writeln!(
        csv,
        "step,time,dt,generation,volume_version,pressure_iterations,residual_max,carrier_divergence,volume_divergence,outward_courant,volume_before,volume_after,inward,outward,source,balance,rounding_budget,mass_after,owned_bytes,boundary_bytes,total_bytes"
    )?;
    for step in 1..=steps {
        let report = simulation.step_with_box_flux(
            LiquidStepInputs {
                requested_dt: dt,
                smoke_source: None,
                forces: &[],
                inlet,
                source: None,
                volume: LiquidVolumeSettings {
                    max_outward_courant: 1.0,
                    actual_divergence_limit: 1e-5,
                },
            },
            &mut workspace,
            boundary,
            |_| false,
        )?;
        let carrier = report.carrier.step;
        let liquid = report.liquid;
        writeln!(
            csv,
            "{step},{:.17e},{:.17e},{},{},{},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{},{},{}",
            carrier.time,
            carrier.dt,
            carrier.generation,
            liquid.stamp.version,
            carrier.pressure.iterations,
            carrier.pressure.true_residual_max,
            carrier.actual_divergence_max,
            liquid.actual_divergence_max,
            liquid.max_outward_courant,
            liquid.liquid_volume_before,
            liquid.liquid_volume_after,
            liquid.inward_boundary_volume,
            liquid.outward_boundary_volume,
            liquid.source_volume,
            liquid.volume_balance_error,
            liquid.volume_rounding_budget,
            liquid.liquid_mass_after,
            report.owned_array_bytes,
            report.boundary_workspace_array_bytes,
            report.total_array_bytes
        )?;
        if step == 1 {
            write_liquid_guidance_png(
                &simulation,
                fs::File::create(directory.join("first.png"))?,
                n * 8,
            )?;
            let mut p = BufWriter::new(fs::File::create(directory.join("first-pressure.csv"))?);
            writeln!(p, "cell,pressure")?;
            for (cell, pressure) in simulation.state().pressure.iter().enumerate() {
                writeln!(p, "{cell},{pressure:.17e}")?;
            }
        }
    }
    csv.flush()?;
    write_liquid_guidance_png(
        &simulation,
        fs::File::create(directory.join("final.png"))?,
        n * 8,
    )?;
    let state = simulation.state();
    let mut cells = BufWriter::new(fs::File::create(directory.join("cells.csv"))?);
    writeln!(cells, "i,j,k,x,fraction,pressure")?;
    for k in 0..4 {
        for j in 0..8 {
            for i in 0..n {
                let cell = grid.cell_index([i, j, k]).unwrap();
                writeln!(
                    cells,
                    "{i},{j},{k},{:.17e},{:.17e},{:.17e}",
                    grid.cell_position([i, j, k]).unwrap()[0],
                    state.liquid.fraction[cell],
                    state.pressure[cell]
                )?;
            }
        }
    }
    cells.flush()?;
    let mut faces = BufWriter::new(fs::File::create(directory.join("faces.csv"))?);
    writeln!(faces, "axis,i,j,k,velocity")?;
    for (d, axis) in [Axis::X, Axis::Y, Axis::Z].into_iter().enumerate() {
        let field = [state.carrier.x, state.carrier.y, state.carrier.z][d];
        let counts = grid.face_counts(axis);
        for k in 0..counts[2] {
            for j in 0..counts[1] {
                for i in 0..counts[0] {
                    writeln!(
                        faces,
                        "{d},{i},{j},{k},{:.17e}",
                        field[grid.face_index(axis, [i, j, k]).unwrap()]
                    )?;
                }
            }
        }
    }
    faces.flush()?;
    fs::write(
        directory.join("run.json"),
        format!(
            "{{\n  \"method\": \"{}\",\n  \"counts\": [{n},8,4],\n  \"steps\": {steps},\n  \"requested_courant\": {courant},\n  \"time\": {:.17e},\n  \"carrier_generation\": {},\n  \"volume_version\": {},\n  \"carrier_density\": 1000,\n  \"represented_density\": 800,\n  \"owned_array_bytes\": {},\n  \"boundary_array_bytes\": {},\n  \"free_surface_pressure\": false,\n  \"surface_reconstruction\": false,\n  \"render\": \"+Z fraction integral; opacity=1-exp(-8*integral); +Y up\"\n}}\n",
            method.id(),
            state.carrier.time,
            state.carrier.generation,
            state.liquid.stamp.version,
            simulation.allocated_bytes(),
            workspace.allocated_bytes()
        ),
    )?;
    println!(
        "{name}: {steps} accepted steps, t={}, {} owned array bytes",
        state.carrier.time,
        simulation.allocated_bytes()
    );
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args_os()
        .nth(1)
        .ok_or("usage: liquid_step FRESH_OUTPUT_DIRECTORY")?;
    let output = Path::new(&output);
    fs::create_dir(output)?;
    for method in PressureImplementation::ALL {
        for (n, courant) in [(16, 0.25), (32, 0.25), (64, 0.25), (64, 0.125)] {
            scenario(output, n, courant, method)?;
        }
    }
    fs::write(
        output.join("complete.json"),
        "{\"scenarios\":8,\"transport_only\":true}\n",
    )?;
    Ok(())
}
