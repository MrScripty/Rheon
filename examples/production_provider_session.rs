//! Bounded host loop over the production owner, with optional frozen AI export.
//! cargo run --release --example production_provider_session -- /path/modal.bin
use rheon::solver_provider::{LabModalPressure, PressureProvider};
use rheon::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut model = std::env::args()
        .nth(1)
        .map(
            |path| -> Result<LabModalPressure, Box<dyn std::error::Error>> {
                let bytes = std::fs::read(path)?;
                LabModalPressure::from_export(&bytes)
                    .map_err(|e| format!("model export: {e:?}").into())
            },
        )
        .transpose()?;
    let grid = GridGeometry::new([32; 3], [1.0 / 32.0; 3], [0.0; 3])?;
    let cap = 32 * 1024 * 1024;
    let method = PressureImplementation::JacobiPcgV1;
    let mut workspace = ProviderStepWorkspace::new(&grid, 1.25, cap, method)?;
    let fractions = vec![0.5; grid.cell_len()];
    let config = LiquidTransportConfig {
        carrier: SimulationConfig {
            density: 1.25,
            memory_limit: cap,
            pressure: PressureSettings {
                relative_residual: 1e-8,
                absolute_residual: 1e-12,
                divergence_limit: 1e-7,
                max_iterations: 3000,
            },
            actual_divergence_limit: 1e-5,
            max_courant: 1.0,
        },
        represented_density: 1.25,
        volume_stamp: VolumeStamp { id: 1, version: 0 },
        carrier_id: 2,
    };
    let mut session = SimulationSession::new(
        SessionIdentity {
            run_id: 1,
            source_revision: "authored-provider-session-example".into(),
        },
        grid,
        config,
        method,
        fractions,
        cap,
        24,
    )?;
    let forces = [BodyForce {
        value: [0.001, 0.002, 0.0],
        units: ForceUnits::Acceleration,
        region: Some(ForceRegion {
            lower: [0.25; 3],
            upper: [0.75; 3],
        }),
    }];
    let inlet = LiquidInlet::new(VolumeStamp { id: 3, version: 0 }, [[0.0; 2]; 3])?;
    println!("frame,time_s,selected,actual_divergence_per_s,pressure_stage_ms");
    // One synchronous interval per host update. No catch-up queue, worker,
    // deadline guarantee or real-time claim. Use accepted dt to advance a host clock.
    for _ in 0..24 {
        let preferred = model.as_mut().map(|m| m as &mut dyn PressureProvider);
        let (step, provider) = session.step_with_pressure_provider(
            LiquidStepInputs {
                requested_dt: 1.0 / 60.0,
                smoke_source: Some(SmokeSource {
                    lower: [0.25; 3],
                    upper: [0.75; 3],
                    tracer_rate: 0.2,
                    vertical_acceleration: 0.0,
                }),
                forces: &forces,
                inlet,
                source: None,
                volume: LiquidVolumeSettings {
                    max_outward_courant: 1.0,
                    actual_divergence_limit: 1e-5,
                },
            },
            &mut workspace,
            preferred,
            None,
            |_| false,
        )?;
        let snapshot = session.snapshot();
        println!(
            "{},{},{},{},{}",
            snapshot.frame(),
            snapshot.state().carrier.time,
            provider.selected,
            step.carrier.step.actual_divergence_max,
            provider.total.as_secs_f64() * 1000.0
        );
        // Borrow snapshot.state() here for rendering/inspection. Release the
        // borrow before the next call; consumer-owned retained copies count separately.
    }
    session.stop();
    Ok(())
}
