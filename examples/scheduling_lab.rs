//! Local scheduling measurements using the real advection and cylinder step.
//! Arguments: size threads repetitions [sparse|dense|step-jacobi|step-sgs].
use rheon::{
    Axis, CalculationObserver, GridGeometry, PressureImplementation, PressureSettings,
    RotatingCylinderEmitter, Simulation, SimulationConfig, advect_tracer, advect_velocity,
};
use std::time::Instant;

fn fingerprint<'a>(fields: impl IntoIterator<Item = &'a [f32]>) -> u64 {
    fields
        .into_iter()
        .flatten()
        .fold(0xcbf29ce484222325, |hash, v| {
            (hash ^ u64::from(v.to_bits())).wrapping_mul(0x100000001b3)
        })
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let n: usize = args.first().ok_or("size required")?.parse()?;
    let threads: usize = args.get(1).ok_or("threads required")?.parse()?;
    let repeats: usize = args.get(2).ok_or("repetitions required")?.parse()?;
    let mode = args.get(3).map_or("sparse", String::as_str);
    if !(8..=128).contains(&n) || threads == 0 || repeats == 0 {
        return Err("Invalid lab settings".into());
    }
    let grid = GridGeometry::new([n as u64; 3], [1.0 / n as f64; 3], [0.0; 3])?;
    let observer = CalculationObserver::new(&grid, threads)?;
    let scheduler = rheon::CpuScheduler::new();
    rayon::ThreadPoolBuilder::new().num_threads(threads).build_scoped(
            |thread| scheduler.scope(|| {
                if mode.ends_with("-no-boxes") { thread.run(); } else { observer.observe(|| thread.run()); }
            }),
        |pool| pool.install(|| -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            if mode.starts_with("step-") || mode.starts_with("smoke-") {
                let method = if mode == "step-sgs" {
                    PressureImplementation::SymmetricGaussSeidelPcgV1
                } else {
                    PressureImplementation::JacobiPcgV1
                };
                let mut sim = Simulation::with_implementation(grid, SimulationConfig {
                    density: 1.0, memory_limit: 192 * 1024 * 1024,
                    pressure: PressureSettings { relative_residual: if mode.starts_with("smoke-") { 1e-11 } else { 1e-9 }, absolute_residual: 1e-12,
                        divergence_limit: if mode.starts_with("smoke-") { 1e-9 } else { 1e-7 }, max_iterations: 2000 },
                    actual_divergence_limit: 1e-5, max_courant: 1.0,
                }, method)?;
                let start = Instant::now();
                for step in 0..repeats {
                    if mode.starts_with("smoke-") {
                        sim.step(0.02, (step < 512).then_some(rheon::SmokeSource {
                            lower: [0.35,0.08,0.35], upper: [0.65,0.30,0.65],
                            tracer_rate: 2.0, vertical_acceleration: 1.0,
                        }), |_| false)?;
                    } else {
                        sim.step_with_emitter(0.02, RotatingCylinderEmitter::default(), |_| false)?;
                    }
                    if mode.starts_with("smoke-") && (step+1)%64==0 {
                        println!("progress={} seconds={:.9}", step+1, start.elapsed().as_secs_f64());
                    }
                }
                let seconds = start.elapsed().as_secs_f64();
                let state = sim.state();
                println!("mode={mode} size={n} threads={threads} repetitions={repeats} seconds={seconds:.9} fingerprint={:016x}",
                    fingerprint([state.x, state.y, state.z, state.tracer]));
                for s in scheduler.snapshot() {
                    println!("profile={:?} calls={} seconds={:.9} batch={}", s.stage, s.calls,
                        s.total_wall_nanoseconds as f64 * 1e-9, s.batch_cells);
                }
                return Ok(());
            }
            let velocity: [Vec<f32>; 3] = std::array::from_fn(|d| {
                vec![0.15 * (d + 1) as f32; grid.face_len([Axis::X, Axis::Y, Axis::Z][d])]
            });
            let fields = std::array::from_fn(|d| velocity[d].as_slice());
            let mut tracer = vec![0.0; grid.cell_len()];
            for z in 0..n { for y in 0..n { for x in 0..n {
                if mode == "dense" || (n/3..n/3+4).contains(&x)
                    && (n/3..n/3+4).contains(&y) && (n/3..n/3+4).contains(&z) {
                    tracer[x+n*(y+n*z)] = 0.7;
                }
            }}}
            let mut out = vec![0.0; tracer.len()];
            let mut advected: [Vec<f32>; 3] = std::array::from_fn(|d| vec![0.0; velocity[d].len()]);
            // One warmup through the same real kernel; pool startup and allocation
            // are excluded from the reported kernel time.
            advect_tracer(&grid, &tracer, fields, 0.02, &mut out, || false)?;
            let start = Instant::now();
            for _ in 0..repeats { advect_tracer(&grid, &tracer, fields, 0.02, &mut out, || false)?; }
            let seconds = start.elapsed().as_secs_f64();
            let workers = observer.snapshot().len();
            println!("mode={mode} size={n} threads={threads} repetitions={repeats} stage=tracer seconds={seconds:.9} workers={workers} fingerprint={:016x}", fingerprint([out.as_slice()]));
            let start = Instant::now();
            for _ in 0..repeats {
                let [u,v,w] = &mut advected;
                advect_velocity(&grid, fields, 0.02, [u,v,w], || false)?;
            }
            let seconds = start.elapsed().as_secs_f64();
            println!("mode={mode} size={n} threads={threads} repetitions={repeats} stage=velocity seconds={seconds:.9} fingerprint={:016x}", fingerprint(advected.iter().map(Vec::as_slice)));
            Ok(())
        }),
    )?.map_err(|e| e.to_string())?;
    Ok(())
}
