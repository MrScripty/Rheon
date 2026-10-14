#![cfg(not(target_arch = "wasm32"))]
use rheon::recording::*;
use rheon::*;
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
fn path(label: &str) -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "rheon-recording-{label}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&dir).unwrap();
    dir.join("run.sqlite")
}
fn clean(path: PathBuf) {
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
}
fn simulation() -> ParticleFluidSimulation {
    ParticleFluidSimulation::new(
        GridGeometry::new([4; 3], [0.25; 3], [0.0; 3]).unwrap(),
        ParticleFluidConfig {
            fill: DomainFill::EmittedFluid,
            samples_per_axis: 4,
            max_particles: 64,
            gravity: [-0.0, -1.0, 0.0],
            ..Default::default()
        },
    )
    .unwrap()
}
fn run(sim: &ParticleFluidSimulation) -> RecordingRun {
    material_run(
        SessionIdentity {
            run_id: u64::MAX,
            source_revision: "authored-test-source-identity".into(),
        },
        sim,
        "test producer; Linux CPU; Rust 1.92; source identity declared",
    )
    .unwrap()
}
fn controls() -> MaterialStepControls {
    MaterialStepControls {
        requested_dt: 1e-5,
        emitter: None,
    }
}
fn append(
    rec: &mut StepRecorder,
    sim: &ParticleFluidSimulation,
    report: Option<ParticleStepReport>,
) {
    rec.append(
        &material_publication(
            sim,
            report,
            report.map(|_| controls()),
            rec.recorded_count(),
            rec.limits().max_record_bytes,
        )
        .unwrap(),
    )
    .unwrap();
}
fn bits_equal(a: &[FluidParticle], b: &[FluidParticle]) {
    assert_eq!(a.len(), b.len());
    for (a, b) in a.iter().zip(b) {
        assert_eq!(a.material, b.material);
        for (a, b) in a
            .position
            .into_iter()
            .chain(a.velocity)
            .chain([a.mass, a.temperature_kelvin])
            .zip(
                b.position
                    .into_iter()
                    .chain(b.velocity)
                    .chain([b.mass, b.temperature_kelvin]),
            )
        {
            assert_eq!(a.to_bits(), b.to_bits());
        }
    }
}
#[test]
fn exact_round_trip_selected_step_units_times_identity_and_existing_vtk_contract() {
    let mut sim = simulation();
    let p = path("roundtrip");
    let limits = RecordingLimits::default();
    let expected_run = run(&sim);
    let mut recorder = StepRecorder::create(&p, expected_run.clone(), limits).unwrap();
    append(&mut recorder, &sim, None);
    let initial = sim.particles().to_vec();
    let report = sim.step(controls().requested_dt, None, || false).unwrap();
    append(&mut recorder, &sim, Some(report));
    let historical = sim.particles().to_vec();
    let (evaluation, d) = sim.particle_diagnostics().unwrap();
    let diagnostics = d.to_vec();
    let time = sim.time();
    let mut vtk_expected = Vec::new();
    write_particles_vtp(
        &mut vtk_expected,
        &expected_run.identity,
        sim.generation(),
        time,
        &historical,
        Some((evaluation, &diagnostics)),
    )
    .unwrap();
    for _ in 0..2 {
        let r = sim.step(controls().requested_dt, None, || false).unwrap();
        append(&mut recorder, &sim, Some(r));
    }
    assert_ne!(sim.particles(), historical);
    recorder
        .finish(
            "completed",
            "all accepted steps recorded",
            Some(sim.generation()),
        )
        .unwrap();
    drop(recorder);
    drop(sim);
    let reader = RecordingReader::open(&p, limits).unwrap();
    reader.verify().unwrap();
    assert_eq!(reader.count().unwrap(), 4);
    assert_eq!(reader.run(), &expected_run);
    assert_eq!(reader.recorded_limits(), limits);
    let zero = reader.select(0).unwrap();
    assert!(zero.field("pressure").is_none());
    bits_equal(
        &RecordedMaterial::decode(&zero).unwrap().particles,
        &initial,
    );
    let selected = reader.select_generation(1).unwrap();
    assert_eq!(selected.time.to_bits(), time.to_bits());
    assert_eq!(selected.dt.unwrap().to_bits(), report.dt.to_bits());
    assert!(selected.controls.contains("requested_dt: 1e-5"));
    assert_eq!(selected.field("position").unwrap().units, "m");
    assert_eq!(
        selected.field("pressure").unwrap().association,
        "particle_index_within_run"
    );
    assert_eq!(
        selected
            .field("pressure")
            .unwrap()
            .evaluation_time
            .unwrap()
            .to_bits(),
        evaluation.to_bits()
    );
    assert!(evaluation < time);
    let material = RecordedMaterial::decode(&selected).unwrap();
    bits_equal(&material.particles, &historical);
    let (t, d) = material.diagnostics.as_ref().unwrap();
    assert_eq!(t.to_bits(), evaluation.to_bits());
    for (a, b) in d.iter().zip(&diagnostics) {
        for (a, b) in [a.density, a.pressure]
            .into_iter()
            .chain(a.acceleration)
            .zip([b.density, b.pressure].into_iter().chain(b.acceleration))
        {
            assert_eq!(a.to_bits(), b.to_bits());
        }
    }
    let mut vtk = Vec::new();
    material
        .write_vtp(&mut vtk, &reader.run().identity)
        .unwrap();
    assert_eq!(vtk, vtk_expected);
    assert!(matches!(reader.select(4), Err(RecordingError::Missing(4))));
    assert!(reader.select_generation(4).is_err());
    drop(reader);
    clean(p);
}
#[test]
fn canonical_session_rejections_and_cancellation_do_not_publish_or_relabel() {
    let sim = simulation();
    let run = run(&sim);
    let mut session =
        MaterialSession::new(run.identity.clone(), sim.grid().clone(), sim.config(), 8).unwrap();
    let p = path("rejected");
    let limits = RecordingLimits::default();
    let mut writer = StepRecorder::create(&p, run, limits).unwrap();
    writer
        .append(
            &material_snapshot_publication(&session.snapshot(), 0, limits.max_record_bytes)
                .unwrap(),
        )
        .unwrap();
    assert!(session.step(controls(), None, || true).is_err());
    assert!(
        session
            .step(
                MaterialStepControls {
                    requested_dt: f64::NAN,
                    ..controls()
                },
                None,
                || false
            )
            .is_err()
    );
    assert_eq!(session.snapshot().generation(), 0);
    assert_eq!(writer.recorded_count(), 1);
    session.step(controls(), None, || false).unwrap();
    writer
        .append(
            &material_snapshot_publication(&session.snapshot(), 1, limits.max_record_bytes)
                .unwrap(),
        )
        .unwrap();
    assert!(session.step(controls(), None, || true).is_err());
    assert_eq!(session.snapshot().generation(), 1);
    // Accidentally recapturing an unchanged snapshot fails instead of retaining a rejected step.
    let duplicate =
        material_snapshot_publication(&session.snapshot(), 2, limits.max_record_bytes).unwrap();
    assert!(writer.append(&duplicate).is_err());
    assert!(matches!(
        writer.append(&duplicate),
        Err(RecordingError::Closed)
    ));
    writer
        .finish(
            "cancelled",
            "rejected call; accepted steps retained",
            Some(1),
        )
        .unwrap();
    drop(writer);
    let reader = RecordingReader::open(&p, limits).unwrap();
    assert_eq!(reader.count().unwrap(), 2);
    reader.verify().unwrap();
    assert_eq!(reader.status().unwrap().outcome, "cancelled");
    drop(reader);
    clean(p);
}
#[test]
fn record_quota_payload_and_database_limits_preserve_committed_prefix_and_disclose_tail() {
    let mut sim = simulation();
    let p = path("quota");
    let limits = RecordingLimits {
        max_records: 2,
        ..Default::default()
    };
    let mut writer = StepRecorder::create(&p, run(&sim), limits).unwrap();
    append(&mut writer, &sim, None);
    let r = sim.step(1e-5, None, || false).unwrap();
    append(&mut writer, &sim, Some(r));
    let r = sim.step(1e-5, None, || false).unwrap();
    let step =
        material_publication(&sim, Some(r), Some(controls()), 2, limits.max_record_bytes).unwrap();
    assert!(matches!(
        writer.append(&step),
        Err(RecordingError::Limit("record count"))
    ));
    assert!(
        writer
            .finish("completed", "incorrect completion", Some(2))
            .is_err()
    );
    writer
        .finish(
            "failed",
            "accepted generation 2 unrecorded: record quota",
            Some(2),
        )
        .unwrap();
    drop(writer);
    let reader = RecordingReader::open(&p, limits).unwrap();
    reader.verify().unwrap();
    assert_eq!(reader.count().unwrap(), 2);
    assert_eq!(reader.status().unwrap().accepted_generation, Some(2));
    assert!(reader.select_generation(2).is_err());
    drop(reader);
    clean(p);
    assert!(matches!(
        material_publication(&sim, Some(r), Some(controls()), 2, 1),
        Err(RecordingError::Limit(_))
    ));
    let p = path("disk");
    let limits = RecordingLimits {
        max_database_bytes: 65536,
        ..Default::default()
    };
    let mut sim = simulation();
    let mut writer = StepRecorder::create(&p, run(&sim), limits).unwrap();
    append(&mut writer, &sim, None);
    let mut exhausted = false;
    for _ in 0..20 {
        let r = sim.step(1e-5, None, || false).unwrap();
        let step = material_publication(
            &sim,
            Some(r),
            Some(controls()),
            writer.recorded_count(),
            limits.max_record_bytes,
        )
        .unwrap();
        match writer.append(&step) {
            Ok(()) => {}
            Err(RecordingError::Sqlite(rusqlite::Error::SqliteFailure(error, _)))
                if error.code == rusqlite::ErrorCode::DiskFull =>
            {
                exhausted = true;
                break;
            }
            Err(error) => panic!("database cap failed at unintended boundary: {error}"),
        }
    }
    assert!(exhausted);
    let committed = writer.recorded_count();
    assert!(std::fs::metadata(&p).unwrap().len() <= limits.max_database_bytes);
    let _status_result = writer.finish(
        "failed",
        "database full; accepted tail may be unrecorded",
        Some(sim.generation()),
    );
    drop(writer);
    let reader = RecordingReader::open(&p, limits).unwrap();
    reader.verify().unwrap();
    assert_eq!(reader.count().unwrap(), committed);
    assert!(reader.select_generation(sim.generation()).is_err());
    assert_ne!(reader.status().unwrap().outcome, "completed");
    drop(reader);
    clean(p);
    let p = path("invalid");
    assert!(
        StepRecorder::create(
            &p,
            run(&sim),
            RecordingLimits {
                max_records: 0,
                ..Default::default()
            }
        )
        .is_err()
    );
    assert!(!p.exists());
    clean(p);
}
#[test]
fn corrupt_blobs_metadata_shape_index_versions_and_truncation_are_rejected() {
    for (mutation, expected) in [
        (
            "UPDATE fields SET data=zeroblob(length(data)) WHERE ordinal=0 AND name='position'",
            "publication digest",
        ),
        (
            "UPDATE fields SET units='s' WHERE ordinal=0 AND name='position'",
            "publication digest",
        ),
        (
            "UPDATE fields SET dimensions=zeroblob(24) WHERE ordinal=0 AND name='position'",
            "field shape",
        ),
        (
            "DELETE FROM fields WHERE ordinal=0 AND name='position'",
            "field inventory",
        ),
        ("UPDATE run SET revision='different-source'", "run metadata"),
        ("UPDATE run SET max_records=max_records+1", "run metadata"),
        ("UPDATE lifecycle SET detail='changed'", "lifecycle digest"),
        (
            "UPDATE steps SET ordinal=1 WHERE ordinal=0",
            "noncontiguous index",
        ),
        ("PRAGMA user_version=2", "archive version"),
        (
            "INSERT INTO fields SELECT 99,field_index,name,units,association,stage,evaluation_time,dimensions,components,scalar_type,data FROM fields WHERE ordinal=0",
            "orphan fields",
        ),
    ] {
        let sim = simulation();
        let p = path("corrupt");
        let limits = RecordingLimits::default();
        let mut writer = StepRecorder::create(&p, run(&sim), limits).unwrap();
        append(&mut writer, &sim, None);
        drop(writer);
        let c = rusqlite::Connection::open(&p).unwrap();
        c.execute_batch("PRAGMA foreign_keys=OFF;").unwrap();
        c.execute_batch(mutation).unwrap();
        drop(c);
        let result = match RecordingReader::open(&p, limits) {
            Err(error) => Err(error),
            Ok(reader) => {
                if expected == "orphan fields" {
                    reader.verify()
                } else {
                    reader.select(0).map(|_| ())
                }
            }
        };
        match (expected, result) {
            ("archive version", Err(RecordingError::Unsupported("archive version"))) => {}
            ("field shape", Err(RecordingError::Invalid("field shape"))) => {}
            (expected, Err(RecordingError::Integrity(actual))) if expected == actual => {}
            (_, result) => {
                panic!("mutation {mutation} failed at an unintended boundary: {result:?}")
            }
        }
        clean(p);
    }
    let sim = simulation();
    let p = path("truncate");
    let limits = RecordingLimits::default();
    let mut writer = StepRecorder::create(&p, run(&sim), limits).unwrap();
    append(&mut writer, &sim, None);
    drop(writer);
    std::fs::OpenOptions::new()
        .write(true)
        .open(&p)
        .unwrap()
        .set_len(700)
        .unwrap();
    assert!(
        matches!(RecordingReader::open(&p,limits),Err(RecordingError::Sqlite(rusqlite::Error::SqliteFailure(error,_))) if error.code==rusqlite::ErrorCode::DatabaseCorrupt)
    );
    clean(p);
}
#[test]
fn generic_fields_preserve_f32_signed_zero_and_admit_zero_length_without_fabrication() {
    let sim = simulation();
    let p = path("generic");
    let limits = RecordingLimits::default();
    let mut writer = StepRecorder::create(&p, run(&sim), limits).unwrap();
    let values = [-0.0_f32, 1.25, f32::from_bits(1)];
    let field = RecordedField {
        name: "face_velocity_x".into(),
        units: "m/s".into(),
        association: "x_face; offset [0,0.5,0.5]; Cartesian XYZ".into(),
        stage: "constructor".into(),
        evaluation_time: None,
        dimensions: [3, 1, 1],
        components: 1,
        scalar_type: "f32".into(),
        data: values.iter().flat_map(|v| v.to_le_bytes()).collect(),
    };
    let empty = RecordedField {
        name: "absent_particles".into(),
        units: "category".into(),
        association: "particle_index_within_run".into(),
        stage: "constructor".into(),
        evaluation_time: None,
        dimensions: [0, 1, 1],
        components: 1,
        scalar_type: "u8".into(),
        data: Vec::new(),
    };
    writer
        .append(&RecordedStep {
            ordinal: 0,
            generation: 0,
            time: 0.0,
            dt: None,
            stage: SnapshotStage::Constructor,
            controls: "no controls".into(),
            fields: vec![field, empty],
        })
        .unwrap();
    drop(writer);
    let reader = RecordingReader::open(&p, limits).unwrap();
    let step = reader.select(0).unwrap();
    assert_eq!(
        step.fields[0].data,
        values
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect::<Vec<_>>()
    );
    assert!(step.field("pressure").is_none());
    assert_eq!(step.fields[1].dimensions, [0, 1, 1]);
    assert!(step.fields[1].data.is_empty());
    drop(reader);
    clean(p);
}
#[test]
fn live_reader_is_indexed_and_never_follows_writer_without_explicit_selection() {
    let mut sim = simulation();
    let p = path("live");
    let limits = RecordingLimits::default();
    let mut writer = StepRecorder::create(&p, run(&sim), limits).unwrap();
    append(&mut writer, &sim, None);
    let reader = RecordingReader::open(&p, limits).unwrap();
    let selected = reader.select(0).unwrap();
    for _ in 0..10 {
        let r = sim.step(1e-5, None, || false).unwrap();
        append(&mut writer, &sim, Some(r));
    }
    assert_eq!(reader.count().unwrap(), 11);
    assert_eq!(selected.generation, 0);
    assert_eq!(reader.select(0).unwrap().fields, selected.fields);
    assert_eq!(reader.select(10).unwrap().generation, 10);
    drop(writer);
    assert_eq!(reader.status().unwrap().outcome, "active");
    drop(reader);
    clean(p);
}
#[test]
fn crashed_process_rolls_back_partial_publication_and_cold_reopens_committed_prefix() {
    let p = path("crash");
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "process_crash_child", "--nocapture"])
        .env("RHEON_RECORDING_CRASH_TEST", &p)
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(73));
    let reader = RecordingReader::open(&p, RecordingLimits::default()).unwrap();
    reader.verify().unwrap();
    assert_eq!(reader.count().unwrap(), 1);
    assert_eq!(reader.status().unwrap().outcome, "active");
    assert!(reader.select(1).is_err());
    drop(reader);
    clean(p);
}
#[test]
fn process_crash_child() {
    let Some(p) = std::env::var_os("RHEON_RECORDING_CRASH_TEST") else {
        return;
    };
    let p = PathBuf::from(p);
    let sim = simulation();
    let mut writer = StepRecorder::create(&p, run(&sim), RecordingLimits::default()).unwrap();
    append(&mut writer, &sim, None);
    drop(writer);
    let c = rusqlite::Connection::open(&p).unwrap();
    c.execute_batch("PRAGMA cache_size=8; PRAGMA synchronous=FULL; BEGIN IMMEDIATE; UPDATE fields SET data=zeroblob(262144) WHERE ordinal=0; INSERT INTO steps SELECT 1,x'0100000000000000',time,dt,stage,controls,field_count,payload_bytes,digest FROM steps WHERE ordinal=0;").unwrap();
    // Forced process exit skips every destructor/rollback. SQLite must recover
    // the actual hot rollback journal, not an in-process transaction mock.
    assert!(p.with_extension("sqlite-journal").exists());
    std::process::exit(73);
}

#[test]
fn source_domain_overflow_missing_initial_steps_and_reader_caps_are_rejected() {
    let mut sim = simulation();
    let p = path("sizing");
    let limits = RecordingLimits::default();
    let mut writer = StepRecorder::create(&p, run(&sim), limits).unwrap();
    let mut record = material_publication(&sim, None, None, 0, limits.max_record_bytes).unwrap();
    record.fields[0].dimensions = [u64::MAX, 2, 1];
    assert!(matches!(
        writer.append(&record),
        Err(RecordingError::Invalid("field size overflow"))
    ));
    drop(writer);
    clean(p);
    let p = path("gap");
    let mut writer = StepRecorder::create(&p, run(&sim), limits).unwrap();
    for _ in 0..2 {
        sim.step(1e-5, None, || false).unwrap();
    }
    let report = sim.step(1e-5, None, || false).unwrap();
    let record = material_publication(
        &sim,
        Some(report),
        Some(controls()),
        0,
        limits.max_record_bytes,
    )
    .unwrap();
    assert!(matches!(
        writer.append(&record),
        Err(RecordingError::Invalid("unrecorded initial steps"))
    ));
    drop(writer);
    clean(p);
    let sim = simulation();
    let p = path("reader-limits");
    let mut writer = StepRecorder::create(&p, run(&sim), limits).unwrap();
    append(&mut writer, &sim, None);
    drop(writer);
    let reader = RecordingReader::open(
        &p,
        RecordingLimits {
            max_record_bytes: 1,
            ..limits
        },
    )
    .unwrap();
    assert!(matches!(reader.select(0), Err(RecordingError::Limit(_))));
    drop(reader);
    clean(p);
}

#[test]
fn empty_material_publications_preserve_absence_and_f64_bits_survive_generic_archive() {
    let mut sim = ParticleFluidSimulation::new(
        GridGeometry::new([4; 3], [0.25; 3], [0.0; 3]).unwrap(),
        ParticleFluidConfig {
            samples_per_axis: 4,
            max_particles: 64,
            ..Default::default()
        },
    )
    .unwrap();
    let p = path("vacuum");
    let limits = RecordingLimits::default();
    let mut writer = StepRecorder::create(&p, run(&sim), limits).unwrap();
    append(&mut writer, &sim, None);
    let r = sim.step(1e-5, None, || false).unwrap();
    append(&mut writer, &sim, Some(r));
    drop(writer);
    let reader = RecordingReader::open(&p, limits).unwrap();
    reader.verify().unwrap();
    let constructor = RecordedMaterial::decode(&reader.select(0).unwrap()).unwrap();
    assert!(constructor.particles.is_empty());
    assert!(constructor.diagnostics.is_none());
    let accepted = RecordedMaterial::decode(&reader.select(1).unwrap()).unwrap();
    assert!(accepted.particles.is_empty());
    assert!(accepted.diagnostics.unwrap().1.is_empty());
    drop(reader);
    clean(p);
    let sim = simulation();
    let p = path("f64-bits");
    let mut writer = StepRecorder::create(&p, run(&sim), limits).unwrap();
    let values = [
        -0.0_f64,
        f64::from_bits(1),
        f64::from_bits(0x3ff0000000000001),
    ];
    let mut record = material_publication(&sim, None, None, 0, limits.max_record_bytes).unwrap();
    record.fields = vec![RecordedField {
        name: "exact_float64".into(),
        units: "dimensionless".into(),
        association: "cell; x-fast".into(),
        stage: "constructor".into(),
        evaluation_time: None,
        dimensions: [3, 1, 1],
        components: 1,
        scalar_type: "f64".into(),
        data: values.iter().flat_map(|v| v.to_le_bytes()).collect(),
    }];
    writer.append(&record).unwrap();
    drop(writer);
    let reader = RecordingReader::open(&p, limits).unwrap();
    assert_eq!(
        reader.select(0).unwrap().fields[0].data,
        record.fields[0].data
    );
    drop(reader);
    clean(p);
}

#[test]
fn already_open_reader_detects_changed_header_instead_of_trusting_cached_identity() {
    let sim = simulation();
    let p = path("live-corruption");
    let limits = RecordingLimits::default();
    let mut writer = StepRecorder::create(&p, run(&sim), limits).unwrap();
    append(&mut writer, &sim, None);
    drop(writer);
    let reader = RecordingReader::open(&p, limits).unwrap();
    assert_eq!(reader.select(0).unwrap().generation, 0);
    let c = rusqlite::Connection::open(&p).unwrap();
    c.execute("UPDATE run SET revision='changed-after-open'", [])
        .unwrap();
    drop(c);
    assert!(matches!(
        reader.select(0),
        Err(RecordingError::Integrity("run metadata"))
    ));
    assert!(matches!(
        reader.verify(),
        Err(RecordingError::Integrity("run metadata"))
    ));
    drop(reader);
    clean(p);
}
