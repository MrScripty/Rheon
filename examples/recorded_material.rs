//! CPU recording and a separate-process indexed inspector using existing VTK.
use rheon::recording::{
    RecordedMaterial, RecordingLimits, RecordingReader, StepRecorder,
    material_snapshot_publication, material_snapshot_run,
};
use rheon::{
    DomainFill, GridGeometry, MaterialSession, MaterialStepControls, ParticleFluidConfig,
    SessionIdentity,
};
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
        _=>Err("usage: recorded_material record NEW.sqlite SOURCE_REVISION | inspect RUN.sqlite ORDINAL NEW.vtp".into()),
    }
}
fn record(path: &Path, revision: &str) -> Result<(), Box<dyn Error>> {
    let grid = GridGeometry::new([4; 3], [0.25; 3], [0.0; 3])?;
    let mut session = MaterialSession::new(
        SessionIdentity {
            run_id: 612,
            source_revision: revision.into(),
        },
        grid,
        ParticleFluidConfig {
            fill: DomainFill::EmittedFluid,
            samples_per_axis: 4,
            max_particles: 64,
            ..Default::default()
        },
        5,
    )?;
    let limits = RecordingLimits {
        max_database_bytes: 8 * 1024 * 1024,
        max_record_bytes: 1024 * 1024,
        max_records: 6,
    };
    let run = material_snapshot_run(
        &session.snapshot(),
        "recorded_material example; source revision supplied by caller; build/toolchain unspecified",
    )?;
    let mut recorder = StepRecorder::create(path, run, limits)?;
    recorder.append(&material_snapshot_publication(
        &session.snapshot(),
        0,
        limits.max_record_bytes,
    )?)?;
    let controls = MaterialStepControls {
        requested_dt: 1e-5,
        emitter: None,
    };
    let result = (|| -> Result<(), Box<dyn Error>> {
        for _ in 0..5 {
            session.step(controls, None, || false)?;
            recorder.append(&material_snapshot_publication(
                &session.snapshot(),
                recorder.recorded_count(),
                limits.max_record_bytes,
            )?)?;
        }
        Ok(())
    })();
    let detail = match &result {
        Ok(_) => "all accepted steps retained".into(),
        Err(e) => format!(
            "{e}; accepted generation {}; tail may be unrecorded",
            session.snapshot().generation()
        ),
    };
    recorder.finish(
        if result.is_ok() {
            "completed"
        } else {
            "failed"
        },
        &detail,
        Some(session.snapshot().generation()),
    )?;
    result?;
    println!(
        "{} publications recorded to {}; keep all / stop at limits; not a restart checkpoint",
        recorder.recorded_count(),
        path.display()
    );
    Ok(())
}
fn inspect(path: &Path, ordinal: usize, out: &Path) -> Result<(), Box<dyn Error>> {
    let reader = RecordingReader::open(path, RecordingLimits::default())?;
    reader.verify()?;
    let selected = reader.select(ordinal)?;
    println!(
        "run {} source {} solver {}",
        reader.run().identity.run_id,
        reader.run().identity.source_revision,
        reader.run().solver
    );
    println!(
        "selected ordinal {} generation {} publication {} s dt {:?}; {:?}",
        selected.ordinal,
        selected.generation,
        selected.time,
        selected.dt,
        reader.status()?
    );
    for f in &selected.fields {
        println!(
            "{}: {} {} components {:?}; {}; {}; evaluated {:?}",
            f.name,
            f.scalar_type,
            f.components,
            f.dimensions,
            f.units,
            f.association,
            f.evaluation_time
        );
    }
    let material = RecordedMaterial::decode(&selected)?;
    let mut writer = BufWriter::new(OpenOptions::new().write(true).create_new(true).open(out)?);
    material.write_vtp(&mut writer, &reader.run().identity)?;
    writer.flush()?;
    println!("selected publication exported to {}", out.display());
    Ok(())
}
