//! Headless material generation with recorded SI emitter controls.
use rheon::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let grid = GridGeometry::new([24; 3], [1.0 / 24.0; 3], [0.0; 3])?;
    let mut session = MaterialSession::new(
        SessionIdentity {
            run_id: 1,
            source_revision: "emitter-example-v2".into(),
        },
        grid,
        ParticleFluidConfig {
            samples_per_axis: 16,
            max_particles: 256,
            memory_limit: 16 * 1024 * 1024,
            gravity: [0.0; 3],
            ..Default::default()
        },
        64,
    )?;
    let controls = MaterialStepControls {
        requested_dt: 0.01,
        emitter: Some(MaterialEmitterControl {
            source_id: 7,
            source_version: 1,
            emitter: RotatingCylinderEmitter {
                emitted_velocity: Some([3.0, 4.0, 0.0]),
                motion_velocity: [0.01, 0.0, 0.0],
                turbulence: EmitterTurbulence {
                    seed: 812,
                    strength: 0.4,
                    spatial_scale: 0.08,
                    time_scale: 0.03,
                },
                ..Default::default()
            },
        }),
    };
    for _ in 0..40 {
        session.step(controls, None, || false)?;
    }
    let snapshot = session.snapshot();
    println!("capabilities={:?}", session.capabilities());
    println!("accepted_controls={:?}", snapshot.accepted_controls());
    println!(
        "time_s={} generation={} stage={:?} particles={}",
        snapshot.time(),
        snapshot.generation(),
        snapshot.stage(),
        snapshot.particles().len()
    );
    println!("first_particle={:?}", snapshot.particles().first());
    Ok(())
}
