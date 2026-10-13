//! CPU particle control workload; no neighbor or boundary implementation changes.
//! Args: samples-per-axis workers steps. Dense water fill, dt=0.001 seconds.
#[cfg(feature = "parallel")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use rheon::{
        CpuScheduler, DomainFill, GridGeometry, ParticleFluidConfig, ParticleFluidSimulation,
    };
    use std::time::Instant;
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        return Err("expected: samples-per-axis workers steps".into());
    }
    let samples: usize = args[0].parse()?;
    let workers: usize = args[1].parse()?;
    let steps: usize = args[2].parse()?;
    if !(4..=32).contains(&samples) || !(1..=4).contains(&workers) || !(1..=1000).contains(&steps) {
        return Err("invalid workload".into());
    }
    let scheduler = CpuScheduler::new();
    rayon::ThreadPoolBuilder::new().num_threads(workers).build_scoped(
        |thread| scheduler.scope(|| thread.run()),
        |pool| pool.install(|| -> Result<(), String> {
            let mut simulation = ParticleFluidSimulation::new(
                GridGeometry::new([32; 3], [1.0 / 32.0; 3], [0.0; 3]).map_err(|e| e.to_string())?,
                ParticleFluidConfig { fill: DomainFill::EmittedFluid, samples_per_axis: samples,
                    max_particles: samples.pow(3), ..ParticleFluidConfig::default() },
            ).map_err(|e| e.to_string())?;
            let mut capture = Vec::with_capacity(simulation.particles().len());
            let mut hash = 0xcbf29ce484222325_u64;
            let mut durations = Vec::with_capacity(steps);
            let mut capture_seconds = 0.0;
            let start = Instant::now();
            for _ in 0..steps {
                let step = Instant::now();
                simulation.step(0.001, None, || false).map_err(|e| e.to_string())?;
                durations.push(step.elapsed().as_secs_f64());
                let capture_start = Instant::now();
                capture.clear();
                capture.extend_from_slice(simulation.particles());
                for particle in &capture {
                    for value in particle.position.into_iter().chain(particle.velocity)
                        .chain([particle.mass, particle.temperature_kelvin]) {
                        hash = (hash ^ value.to_bits()).wrapping_mul(0x100000001b3);
                    }
                }
                std::hint::black_box(&capture);
                capture_seconds += capture_start.elapsed().as_secs_f64();
            }
            println!("particles={} steps={steps} accepted_time={:.17} frame_seconds={:.9} capture_seconds={capture_seconds:.9} managed_bytes={} fingerprint={hash:016x} samples={durations:?}",
                simulation.particles().len(), simulation.time(), start.elapsed().as_secs_f64(), simulation.allocated_bytes());
            for stage in scheduler.snapshot() {
                println!("phase={:?} calls={} seconds={:.9}", stage.stage, stage.calls, stage.total_wall_nanoseconds as f64 * 1e-9);
            }
            Ok(())
        }),
    )??;
    Ok(())
}
#[cfg(not(feature = "parallel"))]
fn main() {
    eprintln!("cpu_particle_lab requires --features parallel");
    std::process::exit(2);
}
