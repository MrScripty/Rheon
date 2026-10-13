//! Embedded material consumer: bounded game-loop updates and full-precision CSV.
//! CSV is illustrative inspection output, not the dense3D/Tuldok dataset schema.
use rheon::*;
use std::{
    io::Write,
    time::{Duration, Instant},
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let grid = GridGeometry::new([4; 3], [0.25; 3], [0.0; 3])?;
    let config = ParticleFluidConfig {
        samples_per_axis: 4,
        max_particles: 512,
        memory_limit: 1024 * 1024,
        fill: DomainFill::EmittedFluid,
        ..Default::default()
    };
    let mut session = MaterialSession::new(
        SessionIdentity {
            run_id: 1,
            source_revision: "authored-material-example".into(),
        },
        grid,
        config,
        8,
    )?;
    let controls = MaterialStepControls {
        requested_dt: 0.001,
        emitter: None,
    };
    let mut accumulator = 0.002;
    let deadline = Instant::now() + Duration::from_secs(1);
    for _ in 0..2 {
        if accumulator < controls.requested_dt {
            break;
        }
        match session.step(controls, Some(deadline), || false) {
            Ok(report) => accumulator -= report.dt,
            Err(MaterialSessionError::Deadline) => break,
            Err(error) => return Err(error.into()),
        }
    }
    let state = session.snapshot();
    let mut output = std::io::stdout().lock();
    writeln!(
        output,
        "run_id,generation,particle_id,accepted_time_s,evaluation_time_s,evaluation_stage,x_m,y_m,z_m,density_kg_m3,pressure_pa"
    )?;
    if let Some((evaluated, diagnostics)) = state.particle_diagnostics() {
        for (id, (particle, diagnostic)) in state.particles().iter().zip(diagnostics).enumerate() {
            writeln!(
                output,
                "{},{},{},{:.17e},{:.17e},post-emission-pre-integration,{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
                state.identity().run_id,
                state.generation(),
                id,
                state.time(),
                evaluated,
                particle.position[0],
                particle.position[1],
                particle.position[2],
                diagnostic.density,
                diagnostic.pressure
            )?;
        }
    }
    session.stop();
    Ok(())
}
