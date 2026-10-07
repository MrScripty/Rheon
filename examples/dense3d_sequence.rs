//! One bounded 16x8x4 transport-only accepted sequence; no comparison sweep.
use rheon::{
    BoxFluxStamp, BoxFluxStepBoundary, BoxFluxStepWorkspace, BoxFluxTracerPolicy, Dense3dLimits,
    Dense3dSequence, GridGeometry, LiquidInlet, LiquidStepInputs, LiquidTransportConfig,
    LiquidTransportSimulation, LiquidVolumeSettings, PrescribedBoxFlux, PressureImplementation,
    PressureSettings, SimulationConfig, VolumeStamp,
};
use std::{
    fs::{self, OpenOptions},
    path::Path,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 1 {
        return Err("usage: dense3d_sequence FRESH_OUTPUT_DIRECTORY".into());
    }
    let grid = GridGeometry::new([16, 8, 4], [0.0625, 0.125, 0.25], [0.0; 3])?;
    let mut fraction = vec![0.0; grid.cell_len()];
    for k in 0..4 {
        for j in 0..8 {
            for i in 4..8 {
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
    let method = PressureImplementation::JacobiPcgV1;
    let simulation = LiquidTransportSimulation::new(grid.clone(), config, method, fraction)?;
    let mut workspace = BoxFluxStepWorkspace::new(grid, 1000.0, 16 * 1024 * 1024, method)?;
    let boundary = BoxFluxStepBoundary {
        flux: PrescribedBoxFlux::new(
            BoxFluxStamp { id: 47, version: 0 },
            [[-0.25, 0.25], [0.0; 2], [0.0; 2]],
        )?,
        tracer: BoxFluxTracerPolicy::ClampedAppearance,
    };
    let inlet = LiquidInlet::new(VolumeStamp { id: 53, version: 0 }, [[0.0; 2]; 3])?;
    let directory = Path::new(&args[0]);
    fs::create_dir(directory)?;
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(directory.join("frames.jsonl"))?;
    // Unbuffered file: dropping the sequence after an error cannot retry output.
    let mut sequence = Dense3dSequence::new(
        simulation,
        &file,
        Dense3dLimits {
            max_cells: 512,
            max_frames: 9,
            max_bytes: 2 * 1024 * 1024,
        },
    )?;
    for _ in 0..8 {
        sequence.step_with_box_flux(
            LiquidStepInputs {
                requested_dt: 0.0625,
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
    }
    sequence.flush()?;
    file.sync_all()?;
    let state = sequence.state();
    // Successful metadata is emitted only after all frames have been synced.
    // The Python runner validates/hashes and publishes run.json last.
    println!(
        "{{\"frame_count\":{},\"time_s\":{:.17e},\"carrier_version\":\"{}\",\"liquid_version\":\"{}\",\"owned_array_bytes\":{},\"boundary_array_bytes\":{},\"frames_bytes\":{}}}",
        sequence.frame_count(),
        state.carrier.time,
        state.carrier_stamp.version,
        state.liquid.stamp.version,
        sequence.allocated_bytes(),
        workspace.allocated_bytes(),
        sequence.bytes_written()
    );
    Ok(())
}
