//! CPU accepted carrier recording, then separate-process historical VTI export.
use rheon::{recording::*, *};
use std::{
    error::Error,
    fs::OpenOptions,
    io::{BufWriter, Write},
    path::Path,
};
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("record") if args.len()==3=>record(Path::new(&args[1]),&args[2]),
        Some("inspect") if args.len()==4=>inspect(Path::new(&args[1]),args[2].parse()?,Path::new(&args[3])),
        _=>Err("usage: recorded_carrier record NEW.sqlite SOURCE_REVISION | inspect RUN.sqlite ORDINAL NEW.vti".into()),
    }
}
fn record(path: &Path, revision: &str) -> Result<(), Box<dyn Error>> {
    let mut sim = Simulation::new(
        GridGeometry::new([3, 4, 5], [0.2, 0.25, 0.3], [-2.0, 3.0, 5.0])?,
        SimulationConfig {
            density: 1.0,
            memory_limit: 16 * 1024 * 1024,
            pressure: PressureSettings {
                relative_residual: 1e-10,
                absolute_residual: 1e-12,
                divergence_limit: 1e-8,
                max_iterations: 2000,
            },
            actual_divergence_limit: 1e-5,
            max_courant: 1.0,
        },
    )?;
    let limits = RecordingLimits {
        max_database_bytes: 8 * 1024 * 1024,
        max_record_bytes: 1024 * 1024,
        max_records: 4,
    };
    let run = carrier_run(
        SessionIdentity {
            run_id: 613,
            source_revision: revision.into(),
        },
        &sim,
        "recorded_carrier CPU example; source revision supplied by caller; build/toolchain unspecified; no restart",
    )?;
    let mut writer = StepRecorder::create(path, run, limits)?;
    writer.append(&carrier_publication(&sim, 0, limits.max_record_bytes)?)?;
    let source = SmokeSource {
        lower: [-1.9, 3.1, 5.1],
        upper: [-1.6, 3.7, 6.1],
        tracer_rate: 2.0,
        vertical_acceleration: 1.0,
    };
    let result = (|| -> Result<(), Box<dyn Error>> {
        for _ in 0..3 {
            sim.step(0.003, Some(source), |_| false)?;
            writer.append(&carrier_publication(
                &sim,
                writer.recorded_count(),
                limits.max_record_bytes,
            )?)?;
        }
        Ok(())
    })();
    let detail = match &result {
        Ok(()) => "all accepted carrier steps retained".into(),
        Err(e) => format!(
            "{e}; accepted generation {}; tail may be unrecorded",
            sim.state().generation
        ),
    };
    writer.finish(
        if result.is_ok() {
            "completed"
        } else {
            "failed"
        },
        &detail,
        Some(sim.state().generation),
    )?;
    result?;
    println!(
        "{} publications recorded to {}; fixed sealed-box fields only; no restart",
        writer.recorded_count(),
        path.display()
    );
    Ok(())
}
fn inspect(path: &Path, ordinal: usize, out: &Path) -> Result<(), Box<dyn Error>> {
    let reader = RecordingReader::open(path, RecordingLimits::default())?;
    reader.verify()?;
    let step = reader.select(ordinal)?;
    let carrier = RecordedCarrier::decode(reader.run(), &step)?;
    println!(
        "selected run {} ordinal {} generation {} publication {} s dt {:?}; {:?}",
        reader.run().identity.run_id,
        ordinal,
        step.generation,
        step.time,
        step.dt,
        reader.status()?
    );
    for f in &step.fields {
        println!(
            "{}: {} {} {:?}; {}; {}; evaluation/interval-start {:?} s",
            f.name, f.scalar_type, f.units, f.dimensions, f.association, f.stage, f.evaluation_time
        );
    }
    let mut file = BufWriter::new(OpenOptions::new().write(true).create_new(true).open(out)?);
    let result = carrier
        .write_vti(&mut file, &reader.run().identity)
        .and_then(|()| file.flush());
    drop(file);
    if let Err(e) = result {
        std::fs::remove_file(out)?;
        return Err(e.into());
    }
    println!("selected carrier exported to {}", out.display());
    Ok(())
}
