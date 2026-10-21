//! Headless fixed-box smoke demonstration and reproducible implementation selection.
use rheon::{
    Axis, BufferPlan, GridGeometry, MAX_CELLS_PER_METRE, PressureImplementation, PressureSettings,
    Simulation, SimulationConfig, SmokeSource, write_guidance_png,
};
use std::{
    error::Error,
    fs::{self, OpenOptions},
    io::{BufWriter, Write},
    path::PathBuf,
    time::Instant,
};
struct Options {
    threads: usize,
    implementation: PressureImplementation,
    pressure: PressureSettings,
    actual_divergence_limit: f64,
    size: u64,
    cells_per_metre: Option<u32>,
    preflight: bool,
    steps: usize,
    dt: f64,
    memory_mib: usize,
    source_off_at: usize,
    output: PathBuf,
}
fn parse() -> Result<Option<Options>, Box<dyn Error>> {
    let mut o = Options {
        threads: 1,
        implementation: PressureImplementation::default(),
        pressure: PressureSettings {
            relative_residual: 1e-9,
            absolute_residual: 1e-12,
            divergence_limit: 1e-7,
            max_iterations: 2000,
        },
        actual_divergence_limit: 1e-5,
        size: 64,
        cells_per_metre: None,
        preflight: false,
        steps: 30,
        dt: 0.02,
        memory_mib: 64,
        source_off_at: usize::MAX,
        output: PathBuf::from("rheon-demo"),
    };
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--preflight" {
            o.preflight = true;
            continue;
        }
        if arg == "--build-info" {
            println!(
                "{{\"version\":\"{}\",\"debug_assertions\":{},\"arch\":\"{}\",\"os\":\"{}\"}}",
                env!("CARGO_PKG_VERSION"),
                cfg!(debug_assertions),
                std::env::consts::ARCH,
                std::env::consts::OS
            );
            return Ok(None);
        }
        if arg == "--list-implementations" {
            println!(
                "[{}]",
                PressureImplementation::ALL
                    .into_iter()
                    .map(|m| format!("{{\"id\":\"{}\",\"label\":\"{}\"}}", m.id(), m.label()))
                    .collect::<Vec<_>>()
                    .join(",")
            );
            return Ok(None);
        }
        if arg == "--help" || arg == "-h" {
            println!(
                "Rheon headless fixed-box smoke demo\n--implementation ID (jacobi-pcg-v1) --list-implementations --build-info\n--size N (1–10000 cells per axis; default 64) --cells-per-metre N (1–1000; default unit cube) --preflight (no field allocation/output)\n--steps N (30) --dt SECONDS (0.02)\n--relative-residual N (1e-9) --pressure-divergence-limit N (1e-7)\n--actual-divergence-limit N (1e-5) --max-iterations N (2000)\n--memory-mib N (64 retained simulation array payload)\n--threads N (1; multiple threads require --features parallel)\n--source-off-at STEP (source on for all steps by default)\n--output NEW_DIRECTORY (rheon-demo)\nWrites opacity.png, steps.csv and run.json. Existing directories are never overwritten.\n64 cubed and 64 MiB are configurable defaults, not real-time performance promises."
            );
            return Ok(None);
        }
        let value = args.next().ok_or("missing option value")?;
        match arg.as_str() {
            "--threads" => o.threads = value.parse()?,
            "--implementation" => {
                o.implementation =
                    PressureImplementation::from_id(&value).ok_or("unknown implementation ID")?
            }
            "--relative-residual" => o.pressure.relative_residual = value.parse()?,
            "--pressure-divergence-limit" => o.pressure.divergence_limit = value.parse()?,
            "--actual-divergence-limit" => o.actual_divergence_limit = value.parse()?,
            "--max-iterations" => o.pressure.max_iterations = value.parse()?,
            "--size" => o.size = value.parse()?,
            "--cells-per-metre" => o.cells_per_metre = Some(value.parse()?),
            "--steps" => o.steps = value.parse()?,
            "--dt" => o.dt = value.parse()?,
            "--memory-mib" => o.memory_mib = value.parse()?,
            "--source-off-at" => o.source_off_at = value.parse()?,
            "--output" => o.output = PathBuf::from(value),
            _ => return Err(format!("unknown option {arg}").into()),
        }
    }
    if o.cells_per_metre
        .is_some_and(|v| !(1..=MAX_CELLS_PER_METRE).contains(&v))
    {
        return Err("cells-per-metre must be 1–1000; --size remains cells per axis".into());
    }
    if !(1..=rheon::MAX_GRID_EDGE).contains(&o.size)
        || o.steps == 0
        || o.steps > 10000
        || !o.dt.is_finite()
        || o.dt <= 0.0
    {
        return Err(
            "size must be 1–10000 cells per axis; steps must be 1–10000; dt must be finite and positive"
                .into(),
        );
    }
    if o.cells_per_metre.is_none() && o.size > u64::from(MAX_CELLS_PER_METRE) {
        return Err("Unit-box count mode exceeds 1000 cells/m; choose --cells-per-metre 1–1000 to set physical extent independently".into());
    }
    if o.source_off_at == usize::MAX {
        o.source_off_at = o.steps;
    }
    if o.source_off_at > o.steps {
        return Err("source-off-at must not exceed steps".into());
    }
    if o.threads == 0 || o.threads > std::thread::available_parallelism().map_or(1, usize::from) {
        return Err("threads must be between 1 and the available CPU count".into());
    }
    if o.threads > 1 && !cfg!(feature = "parallel") {
        return Err("multiple CPU threads require building with --features parallel".into());
    }
    Ok(Some(o))
}
fn run(o: Options) -> Result<(), Box<dyn Error>> {
    if o.preflight {
        return run_simulation(o);
    }
    #[cfg(feature = "parallel")]
    {
        let scheduler = rheon::CpuScheduler::new();
        rayon::ThreadPoolBuilder::new()
            .num_threads(o.threads)
            .thread_name(|i| format!("rheon-cpu-{i}"))
            .build_scoped(
                |thread| scheduler.scope(|| thread.run()),
                |pool| pool.install(|| run_simulation(o).map_err(|e| e.to_string())),
            )?
            .map_err(Into::into)
    }
    #[cfg(not(feature = "parallel"))]
    run_simulation(o)
}
fn run_simulation(o: Options) -> Result<(), Box<dyn Error>> {
    let grid = match o.cells_per_metre {
        Some(density) => GridGeometry::at_cell_density([o.size; 3], density, [0.0; 3])?,
        None => GridGeometry::new([o.size; 3], [1.0 / o.size as f64; 3], [0.0; 3])?,
    };
    let h = grid.spacing()[0];
    // Preserve the original declared unit box/source exactly in count mode;
    // n * (1/n) can round one ulp below 1 for ordinary counts such as49.
    let side_m = o.cells_per_metre.map_or(1.0, |_| grid.upper()[0]);
    let budget = o
        .memory_mib
        .checked_mul(1024 * 1024)
        .ok_or("memory budget overflow")?;
    let plan = BufferPlan::for_grid(&grid, usize::MAX)?;
    let cell_iterations = (grid.cell_len() as u128)
        .checked_mul(o.steps as u128)
        .and_then(|n| n.checked_mul(o.pressure.max_iterations as u128))
        .ok_or("work estimate overflow")?;
    if o.preflight {
        println!(
            "{{\"counts_xyz\":[{0},{0},{0}],\"cell_size_m\":{h:.17e},\"domain_side_m\":{side_m:.17e},\"cells\":{1},\"faces\":[{2},{3},{4}],\"native_array_payload_bytes\":{5},\"memory_limit_bytes\":{budget},\"pressure_cell_iteration_budget\":{cell_iterations},\"fits_memory_limit\":{6},\"whole_process_memory_cap_claimed\":false}}",
            o.size,
            grid.cell_len(),
            grid.face_len(Axis::X),
            grid.face_len(Axis::Y),
            grid.face_len(Axis::Z),
            plan.total_bytes,
            plan.total_bytes <= budget
        );
    }
    BufferPlan::for_grid(&grid, budget)?;
    if o.preflight {
        return Ok(());
    }
    let cfg = SimulationConfig {
        density: 1.0,
        memory_limit: budget,
        pressure: o.pressure,
        actual_divergence_limit: o.actual_divergence_limit,
        max_courant: 1.0,
    };
    let mut sim = Simulation::with_implementation(grid, cfg, o.implementation)?;
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
        lower: [0.35, 0.08, 0.35].map(|v| v * side_m),
        upper: [0.65, 0.30, 0.65].map(|v| v * side_m),
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
        "{{\n  \"schema_version\": 2,\n  \"implementation\": \"{}\",\n  \"cpu_threads\": {},\n  \"pressure_relative_residual\": {:.17e},\n  \"pressure_absolute_residual\": 1e-12,\n  \"pressure_divergence_limit\": {:.17e},\n  \"actual_divergence_limit\": {:.17e},\n  \"pressure_iteration_limit\": {},\n  \"model\": \"fixed-box smoke tracer, prescribed localized Y acceleration\",\n  \"cell_size_m\": {h:.17e},\n  \"domain_side_m\": {side_m:.17e},\n  \"size\": {},\n  \"steps\": {},\n  \"source_off_at\": {},\n  \"requested_dt\": {:.17e},\n  \"accepted_time\": {:.17e},\n  \"managed_simulation_bytes\": {},\n  \"managed_budget_bytes\": {},\n  \"raw_export_pixel_bytes\": {},\n  \"measured_step_seconds\": {:.9},\n  \"last_divergence_max\": {:.17e},\n  \"whole_process_memory_cap_claimed\": false,\n  \"real_time_performance_claimed\": false\n}}",
        o.implementation.id(),
        o.threads,
        o.pressure.relative_residual,
        o.pressure.divergence_limit,
        o.actual_divergence_limit,
        o.pressure.max_iterations,
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
