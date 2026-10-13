//! Generate an asymmetric real-session fixture plus independent native values.
//! Usage: cargo run --example vtk_export_fixture -- /absolute/external/directory
//! Authored particles below are explicitly synthetic interchange fixtures.
use rheon::*;
use std::{
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::Path,
};

fn emit(
    path: &Path,
    action: impl FnOnce(&mut BufWriter<File>) -> io::Result<()>,
) -> io::Result<()> {
    let mut out = BufWriter::new(File::create(path)?);
    action(&mut out)?;
    out.flush()
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let destination = std::env::args_os()
        .nth(1)
        .ok_or("external destination directory required")?;
    let destination = Path::new(&destination);
    fs::create_dir_all(destination)?;
    let root = fs::canonicalize(env!("CARGO_MANIFEST_DIR"))?;
    if fs::canonicalize(destination)?.starts_with(root) {
        return Err("generated fixtures must stay outside the repository".into());
    }
    let grid = GridGeometry::new([3, 2, 4], [0.25, 0.5, 0.125], [-1.25, 2.5, -0.75])?;
    let fraction: Vec<_> = (0..grid.cell_len())
        .map(|i| (i as f64 + 1.0) / 32.0)
        .collect();
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
        volume_stamp: VolumeStamp {
            id: u64::MAX - 1,
            version: 0,
        },
        carrier_id: u64::MAX - 2,
    };
    let identity = SessionIdentity {
        run_id: u64::MAX,
        source_revision: "fixture<&\"'>–源".into(),
    };
    let mut session = SimulationSession::new(
        identity.clone(),
        grid,
        config,
        PressureImplementation::JacobiPcgV1,
        fraction,
        16 * 1024 * 1024,
        4,
    )?;
    emit(&destination.join("constructor.vti"), |w| {
        write_session_vti(w, &session.snapshot())
    })?;
    let boundary = BoxFluxStepBoundary {
        flux: PrescribedBoxFlux::new(
            BoxFluxStamp { id: 91, version: 0 },
            [[-0.125, 0.125], [-0.25, 0.25], [0.375, -0.375]],
        )?,
        tracer: BoxFluxTracerPolicy::ClampedAppearance,
    };
    let inputs = LiquidStepInputs {
        requested_dt: 0.015625,
        smoke_source: Some(SmokeSource {
            lower: [-1.25, 2.5, -0.75],
            upper: [-0.75, 3.0, -0.5],
            tracer_rate: 3.25,
            vertical_acceleration: 0.0,
        }),
        forces: &[],
        inlet: LiquidInlet::new(VolumeStamp { id: 93, version: 0 }, [[0.0; 2]; 3])?,
        source: None,
        volume: LiquidVolumeSettings {
            max_outward_courant: 1.0,
            actual_divergence_limit: 1e-5,
        },
    };
    session.step(inputs, boundary, None, |_| false)?;
    let snapshot = session.snapshot();
    emit(&destination.join("accepted.vti"), |w| {
        write_session_vti(w, &snapshot)
    })?;
    let state = snapshot.state();
    // Native arrays serialized independently of the VTK encoder for exact checks.
    emit(&destination.join("expected-session.json"), |w| {
        writeln!(
            w,
            "{{\"x\":{:?},\"y\":{:?},\"z\":{:?},\"fraction\":{:?},\"tracer\":{:?},\"pressure\":{:?},\"time\":{},\"dt\":{}}}",
            state.carrier.x,
            state.carrier.y,
            state.carrier.z,
            state.liquid.fraction,
            state.carrier.tracer,
            state.pressure,
            state.carrier.time,
            snapshot.report().unwrap().carrier.step.dt
        )
    })?;
    let particles = [
        FluidParticle {
            position: [-0.125, 2.75, 1.2345678901234567],
            velocity: [0.12345678901234567, -2.0, 3.0],
            mass: 0.12345678901234567,
            temperature_kelvin: 291.125,
            material: 0,
        },
        FluidParticle {
            position: [0.5, -1.0, 2.0],
            velocity: [-0.0, 1.0 / 3.0, -4.25],
            mass: 2.5,
            temperature_kelvin: 302.625,
            material: 1,
        },
    ];
    let diagnostics = [
        ParticleDiagnostic {
            density: 998.1234567890123,
            pressure: 12.123456789012345,
            acceleration: [-9.81, 0.0, 1.0 / 7.0],
        },
        ParticleDiagnostic {
            density: 1.25,
            pressure: 101325.125,
            acceleration: [1.0, -2.0, 3.0],
        },
    ];
    emit(&destination.join("particles.vtp"), |w| {
        write_particles_vtp(
            w,
            &identity,
            17,
            1.25,
            &particles,
            Some((1.125, &diagnostics)),
        )
    })?;
    emit(&destination.join("particles-initial.vtp"), |w| {
        write_particles_vtp(w, &identity, 0, 0.0, &particles, None)
    })?;
    emit(&destination.join("particles-empty.vtp"), |w| {
        write_particles_vtp(w, &identity, 0, 0.0, &[], None)
    })?;
    let mut material = MaterialSession::new(
        identity.clone(),
        GridGeometry::new([4; 3], [0.25; 3], [0.0; 3])?,
        ParticleFluidConfig {
            fill: DomainFill::EmittedFluid,
            samples_per_axis: 4,
            max_particles: 64,
            memory_limit: 1024 * 1024,
            ..Default::default()
        },
        2,
    )?;
    emit(&destination.join("material-constructor.vtp"), |w| {
        write_material_vtp(w, &material.snapshot())
    })?;
    material.step(
        MaterialStepControls {
            requested_dt: 1e-5,
            emitter: None,
        },
        None,
        || false,
    )?;
    let publication = material.snapshot();
    emit(&destination.join("material-accepted.vtp"), |w| {
        write_material_vtp(w, &publication)
    })?;
    let particles = publication.particles();
    let (diagnostic_time, diagnostics) = publication.particle_diagnostics().unwrap();
    emit(&destination.join("expected-material.json"), |w| {
        writeln!(
            w,
            "{{\"time\":{},\"diagnostic_time\":{},\"position\":{:?},\"velocity\":{:?},\"mass\":{:?},\"temperature\":{:?},\"material\":{:?},\"density\":{:?},\"pressure\":{:?},\"acceleration\":{:?}}}",
            publication.time(),
            diagnostic_time,
            particles.iter().map(|p| p.position).collect::<Vec<_>>(),
            particles.iter().map(|p| p.velocity).collect::<Vec<_>>(),
            particles.iter().map(|p| p.mass).collect::<Vec<_>>(),
            particles
                .iter()
                .map(|p| p.temperature_kelvin)
                .collect::<Vec<_>>(),
            particles.iter().map(|p| p.material).collect::<Vec<_>>(),
            diagnostics.iter().map(|p| p.density).collect::<Vec<_>>(),
            diagnostics.iter().map(|p| p.pressure).collect::<Vec<_>>(),
            diagnostics
                .iter()
                .map(|p| p.acceleration)
                .collect::<Vec<_>>()
        )
    })?;
    println!("VTK fixtures written to {}", destination.display());
    Ok(())
}
