//! CPU step + owned field capture benchmark. No renderer or device frame claim.
#[cfg(feature = "parallel")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use rheon::{
        CpuScheduler, GridGeometry, PressureImplementation, PressureSettings,
        RotatingCylinderEmitter, Simulation, SimulationConfig,
    };
    use std::time::Instant;
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 5 {
        return Err("expected: size threads steps adaptive|fixed solver-id".into());
    }
    let n: u64 = args[0].parse()?;
    let threads: usize = args[1].parse()?;
    let steps: usize = args[2].parse()?;
    if !(8..=64).contains(&n)
        || !(1..=256).contains(&threads)
        || !(1..=4096).contains(&steps)
        || !matches!(args[3].as_str(), "adaptive" | "fixed")
    {
        return Err("invalid benchmark settings".into());
    }
    let method = PressureImplementation::from_id(&args[4]).ok_or("unknown solver")?;
    let scheduler = CpuScheduler::new();
    let adaptive = args[3] == "adaptive";
    rayon::ThreadPoolBuilder::new().num_threads(threads).build_scoped(
        |thread| { if adaptive { scheduler.scope(|| thread.run()); } else { thread.run(); } },
        |pool| pool.install(|| -> Result<(), String> {
            let mut sim = Simulation::with_implementation(
                GridGeometry::new([n; 3], [1.0 / n as f64; 3], [0.0; 3]).map_err(|e| e.to_string())?,
                SimulationConfig { density: 1.0, memory_limit: 192 * 1024 * 1024,
                    pressure: PressureSettings { relative_residual: 1e-9, absolute_residual: 1e-12,
                        divergence_limit: 1e-7, max_iterations: 2000 },
                    actual_divergence_limit: 1e-5, max_courant: 1.0 }, method).map_err(|e| e.to_string())?;
            let state = sim.state();
            let mut capture = Vec::with_capacity(state.x.len() + state.y.len() + state.z.len() + state.tracer.len());
            let mut step_seconds = 0.0;
            let mut capture_seconds = 0.0;
            let mut time = 0.0;
            let mut divergence = 0.0_f64;
            let mut hash = 0xcbf29ce484222325_u64;
            let start = Instant::now();
            for _ in 0..steps {
                let start_step = Instant::now();
                let report = sim.step_with_emitter(0.02, RotatingCylinderEmitter::default(), |_| false)
                    .map_err(|e| e.to_string())?;
                step_seconds += start_step.elapsed().as_secs_f64();
                time = report.time;
                divergence = divergence.max(report.actual_divergence_max);
                let start_capture = Instant::now();
                capture.clear();
                let state = sim.state();
                for field in [state.x, state.y, state.z, state.tracer] { capture.extend_from_slice(field); }
                // Consume every frame, not only the final state. Included in CPU capture cost.
                for value in &capture { hash = (hash ^ u64::from(value.to_bits())).wrapping_mul(0x100000001b3); }
                std::hint::black_box(&capture);
                capture_seconds += start_capture.elapsed().as_secs_f64();
            }
            let frame_seconds = start.elapsed().as_secs_f64();
            println!("steps={steps} accepted_time={time:.17} divergence={divergence:.17e} step_seconds={step_seconds:.9} capture_seconds={capture_seconds:.9} frame_seconds={frame_seconds:.9} managed_bytes={} capture_bytes={} fingerprint={hash:016x}", sim.allocated_bytes(), capture.capacity() * std::mem::size_of::<f32>());
            Ok(())
        }),
    )??;
    Ok(())
}
#[cfg(not(feature = "parallel"))]
fn main() {
    eprintln!("cloud_frame_lab requires --features parallel");
    std::process::exit(2);
}
