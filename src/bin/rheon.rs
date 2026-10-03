//! Headless fixed-box smoke demonstration; desktop GUI is a later milestone.
use rheon::{
    GridGeometry, PressureSettings, Simulation, SimulationConfig, SmokeSource, write_guidance_png,
};
use std::{
    error::Error,
    fs::{self, OpenOptions},
    io::{BufWriter, Write},
    path::PathBuf,
    time::Instant,
};
struct Options {
    size: u64,
    steps: usize,
    dt: f64,
    memory_mib: usize,
    source_off_at: usize,
    output: PathBuf,
}
fn parse() -> Result<Option<Options>, Box<dyn Error>> {
    let mut o = Options {
        size: 64,
        steps: 30,
        dt: 0.02,
        memory_mib: 64,
        source_off_at: usize::MAX,
        output: PathBuf::from("rheon-demo"),
    };
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--help" || arg == "-h" {
            println!(
                "Rheon headless fixed-box smoke demo\n--size N (64) --steps N (30) --dt SECONDS (0.02)\n--memory-mib N (64 retained simulation array payload)\n--source-off-at STEP (source on for all steps by default)\n--output NEW_DIRECTORY (rheon-demo)\nWrites opacity.png, steps.csv and run.json. Existing directories are never overwritten.\n64 cubed and 64 MiB are configurable defaults, not real-time performance promises."
            );
            return Ok(None);
        }
        let value = args.next().ok_or("missing option value")?;
        match arg.as_str() {
            "--size" => o.size = value.parse()?,
            "--steps" => o.steps = value.parse()?,
            "--dt" => o.dt = value.parse()?,
            "--memory-mib" => o.memory_mib = value.parse()?,
            "--source-off-at" => o.source_off_at = value.parse()?,
            "--output" => o.output = PathBuf::from(value),
            _ => return Err(format!("unknown option {arg}").into()),
        }
    }
    if o.size == 0 || o.steps == 0 || o.steps > 10000 || !o.dt.is_finite() || o.dt <= 0.0 {
        return Err(
            "size and steps must be positive (steps <= 10000); dt must be finite and positive"
                .into(),
        );
    }
    if o.source_off_at == usize::MAX {
        o.source_off_at = o.steps;
    }
    if o.source_off_at > o.steps {
        return Err("source-off-at must not exceed steps".into());
    }
    Ok(Some(o))
}
fn run(o: Options) -> Result<(), Box<dyn Error>> {
    let h = 1.0 / o.size as f64;
    let grid = GridGeometry::new([o.size; 3], [h; 3], [0.0; 3])?;
    let budget = o
        .memory_mib
        .checked_mul(1024 * 1024)
        .ok_or("memory budget overflow")?;
    let cfg = SimulationConfig {
        density: 1.0,
        memory_limit: budget,
        pressure: PressureSettings {
            relative_residual: 1e-9,
            absolute_residual: 1e-12,
            divergence_limit: 1e-7,
            max_iterations: 2000,
        },
        actual_divergence_limit: 1e-5,
        max_courant: 1.0,
    };
    let mut sim = Simulation::new(grid, cfg)?;
    fs::create_dir(&o.output)?;
    let create = |name: &str| {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(o.output.join(name))
    };
    let mut csv = BufWriter::new(create("steps.csv")?);
    writeln!(
        csv,
        "step,time,dt,pressure_iterations,full_residual_max,actual_divergence_max,courant,tracer_integral,kinetic_energy"
    )?;
    let source = SmokeSource {
        lower: [0.35, 0.08, 0.35],
        upper: [0.65, 0.30, 0.65],
        tracer_rate: 2.0,
        vertical_acceleration: 1.0,
    };
    let start = Instant::now();
    let mut last = None;
    for step in 0..o.steps {
        let r = sim.step(o.dt, (step < o.source_off_at).then_some(source), |_| false)?;
        writeln!(
            csv,
            "{},{:.17e},{:.17e},{},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
            step + 1,
            r.time,
            r.dt,
            r.pressure.iterations,
            r.pressure.true_residual_max,
            r.actual_divergence_max,
            r.courant,
            r.tracer_integral,
            r.kinetic_energy
        )?;
        last = Some(r);
    }
    let elapsed = start.elapsed().as_secs_f64();
    csv.flush()?;
    let pixel_bytes = write_guidance_png(
        &sim,
        BufWriter::new(create("opacity.png")?),
        16 * 1024 * 1024,
    )?;
    let r = last.ok_or("no accepted step")?;
    // Completion manifest is written last. A failed run may leave diagnostic
    // partial outputs but cannot create a successful completion manifest.
    let mut metadata = BufWriter::new(create("run.json")?);
    writeln!(
        metadata,
        "{{\n  \"model\": \"fixed-box smoke tracer, prescribed localized Y acceleration\",\n  \"size\": {},\n  \"steps\": {},\n  \"source_off_at\": {},\n  \"requested_dt\": {:.17e},\n  \"accepted_time\": {:.17e},\n  \"managed_simulation_bytes\": {},\n  \"managed_budget_bytes\": {},\n  \"raw_export_pixel_bytes\": {},\n  \"measured_step_seconds\": {:.9},\n  \"last_divergence_max\": {:.17e},\n  \"whole_process_memory_cap_claimed\": false,\n  \"real_time_performance_claimed\": false\n}}",
        o.size,
        o.steps,
        o.source_off_at,
        o.dt,
        r.time,
        sim.allocated_bytes(),
        budget,
        pixel_bytes,
        elapsed,
        r.actual_divergence_max
    )?;
    metadata.flush()?;
    println!(
        "Accepted {} steps; time {:.6}; managed arrays {} bytes; step wall time {:.3}s. Wrote {}",
        o.steps,
        r.time,
        sim.allocated_bytes(),
        elapsed,
        o.output.display()
    );
    Ok(())
}
fn main() {
    let result = parse().and_then(|o| match o {
        Some(o) => run(o),
        None => Ok(()),
    });
    if let Err(e) = result {
        eprintln!("Rheon: {e}");
        std::process::exit(1);
    }
}
