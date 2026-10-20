//! Prepared post-custody numerical/SQLite cases. Compilation/listing is safe;
//! execution requires separate authorization and the existing bounded supervisor.
#![cfg(not(target_arch = "wasm32"))]
use rheon::{recording::*, *};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

// Own exactly one fresh directory; never remove an existing directory on create
// failure. Only explicit successful cleanup removes it; failures retain evidence.
struct Archive {
    directory: Option<PathBuf>,
}
impl Archive {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "rheon-force-custody-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        eprintln!("owned recording directory: {}", path.display());
        Self {
            directory: Some(path),
        }
    }
    fn path(&self) -> PathBuf {
        self.directory.as_ref().unwrap().join("run.sqlite")
    }
    fn cleanup(mut self) {
        let path = self.directory.as_ref().unwrap();
        let mut total = 0;
        let mut files = 0;
        for entry in std::fs::read_dir(path).unwrap() {
            let metadata = entry.unwrap().metadata().unwrap();
            assert!(metadata.is_file());
            assert!(metadata.len() <= 1024 * 1024);
            total += metadata.len();
            files += 1;
        }
        assert!(files <= 3 && total <= 3 * 1024 * 1024);
        std::fs::remove_dir_all(path).unwrap();
        self.directory = None;
    }
}
impl Drop for Archive {
    fn drop(&mut self) {
        if let Some(path) = &self.directory {
            eprintln!("preserved recording evidence: {}", path.display());
        }
    }
}

// Each test runs alone in an owned child. On panic retain open SQLite resources
// until process exit so their destructors cannot modify the original failure
// recording/journal before the supervisor preserves the logged owned directory.
// Successful explicit drops retain their normal behavior; no unsafe code.
struct KeepOnPanic<T>(Option<T>);
impl<T> KeepOnPanic<T> {
    fn new(value: T) -> Self {
        Self(Some(value))
    }
}
impl<T> std::ops::Deref for KeepOnPanic<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0.as_ref().unwrap()
    }
}
impl<T> std::ops::DerefMut for KeepOnPanic<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0.as_mut().unwrap()
    }
}
impl<T> Drop for KeepOnPanic<T> {
    fn drop(&mut self) {
        if std::thread::panicking()
            && let Some(value) = self.0.take()
        {
            std::mem::forget(value);
        }
    }
}
fn limits() -> RecordingLimits {
    RecordingLimits {
        max_database_bytes: 1024 * 1024,
        max_record_bytes: 64 * 1024,
        max_records: 3,
    }
}
fn identity(run_id: u64) -> SessionIdentity {
    SessionIdentity {
        run_id,
        source_revision: "prepared-force-custody-contract".into(),
    }
}
fn fields(name: &str) -> ForceSnapshot {
    ForceScene {
        objects: vec![ForceObject {
            id: SceneObjectId(1),
            name: name.into(),
            enabled: true,
            transform: ForceTransform::default(),
            units: ForceUnits::ForceDensity,
            kind: ForceKind::Directional {
                vector: [2.0, 0.0, 0.0],
            },
            support: ForceSupport::Global,
            time: ForceTime::default(),
        }],
        ..Default::default()
    }
    .freeze()
    .unwrap()
}
fn bits(state: StateView<'_>) -> (Vec<u32>, u64, u64) {
    (
        state
            .x
            .iter()
            .chain(state.y)
            .chain(state.z)
            .chain(state.tracer)
            .map(|v| v.to_bits())
            .collect(),
        state.time.to_bits(),
        state.generation,
    )
}
fn particle_bits(particles: &[FluidParticle]) -> Vec<u64> {
    particles
        .iter()
        .flat_map(|p| {
            p.position
                .into_iter()
                .chain(p.velocity)
                .chain([p.mass, p.temperature_kelvin])
                .map(f64::to_bits)
                .chain([u64::from(p.material)])
        })
        .collect()
}
fn diagnostic_bits(diagnostics: &[ParticleDiagnostic]) -> Vec<u64> {
    diagnostics
        .iter()
        .flat_map(|d| {
            [d.density, d.pressure]
                .into_iter()
                .chain(d.acceleration)
                .map(f64::to_bits)
        })
        .collect()
}
fn same_scene(actual: &ForceSnapshot, expected: &ForceSnapshot) {
    assert_eq!(
        encode_force_metadata(actual).unwrap(),
        encode_force_metadata(expected).unwrap()
    );
}
fn inject_changed_scene(step: &mut RecordedStep, changed: &ForceSnapshot) {
    let field = step
        .fields
        .iter_mut()
        .find(|f| f.name == "external_force_scene_v1")
        .unwrap();
    field.data = encode_force_metadata(changed).unwrap();
    field.dimensions = [field.data.len() as u64, 1, 1];
}

#[test]
#[ignore = "post-custody numerical/SQLite integration: explicit bounded execution authorization required"]
fn carrier_force_archive_round_trip_and_failed_attempt_invalidation() {
    let archive = Archive::new();
    let bounds = limits();
    let force = fields("accepted");
    let changed = fields("changed draft");
    let mut sim = Simulation::new(
        GridGeometry::new([2, 2, 1], [1.0; 3], [0.0; 3]).unwrap(),
        SimulationConfig {
            density: 2.0,
            memory_limit: 1024 * 1024,
            pressure: PressureSettings {
                relative_residual: 1e-11,
                absolute_residual: 1e-12,
                divergence_limit: 1e-9,
                max_iterations: 100,
            },
            actual_divergence_limit: 1e-6,
            max_courant: 1.0,
        },
    )
    .unwrap();
    assert!(sim.accepted_interval_pressure().is_none());
    assert!(sim.accepted_force_interval_pressure().is_none());
    let run = carrier_run(identity(1), &sim, "prepared bounded case")
        .unwrap()
        .with_force_fields(&force)
        .unwrap();
    let mut writer =
        KeepOnPanic::new(StepRecorder::create(&archive.path(), run.clone(), bounds).unwrap());
    writer
        .append(
            &carrier_publication_with_force_fields(&sim, &force, 0, bounds.max_record_bytes)
                .unwrap(),
        )
        .unwrap();
    let report = sim
        .step_with_force_fields(0.001, None, &force, |_| false)
        .unwrap();
    assert!(sim.accepted_interval_pressure().is_none());
    let pressure = sim.accepted_force_interval_pressure().unwrap();
    assert_eq!(pressure.interval_start.to_bits(), 0.0_f64.to_bits());
    assert_eq!(
        pressure.controls.requested_dt.to_bits(),
        0.001_f64.to_bits()
    );
    assert!(pressure.controls.source.is_none() && pressure.controls.emitter.is_none());
    assert_eq!(pressure.report.dt.to_bits(), report.step.dt.to_bits());
    assert_eq!(pressure.report.generation, report.step.generation);
    assert_eq!(pressure.work, report.forces);
    same_scene(pressure.fields, &force);
    let expected_pressure: Vec<_> = pressure.values.iter().map(|p| p.to_bits()).collect();
    let expected_state = bits(sim.state());
    assert!(
        carrier_publication_with_force_fields(&sim, &changed, 1, bounds.max_record_bytes).is_err()
    );
    writer
        .append(
            &carrier_publication_with_force_fields(&sim, &force, 1, bounds.max_record_bytes)
                .unwrap(),
        )
        .unwrap();
    let mut rejected =
        carrier_publication_with_force_fields(&sim, &force, 2, bounds.max_record_bytes).unwrap();
    assert!(matches!(
        sim.step_with_force_fields(0.001, None, &changed, |s| s == StepStage::BeforeCommit),
        Err(SimulationError::Cancelled { .. })
    ));
    assert_eq!(bits(sim.state()), expected_state);
    assert!(sim.accepted_interval_pressure().is_none());
    assert!(sim.accepted_force_interval_pressure().is_none());
    assert!(carrier_publication(&sim, 2, bounds.max_record_bytes).is_err());
    inject_changed_scene(&mut rejected, &changed);
    assert!(matches!(
        writer.append(&rejected),
        Err(RecordingError::Integrity(
            "run/step force document mismatch"
        ))
    ));
    assert_eq!(writer.recorded_count(), 2);
    writer
        .finish(
            "failed",
            "deliberately rejected changed force intent",
            Some(1),
        )
        .unwrap();
    drop(writer);
    let reader = KeepOnPanic::new(RecordingReader::open(&archive.path(), bounds).unwrap());
    reader.verify().unwrap();
    assert_eq!(reader.run(), &run);
    assert_eq!(reader.count().unwrap(), 2);
    let initial = RecordedCarrier::decode(reader.run(), &reader.select(0).unwrap()).unwrap();
    assert!(initial.interval_pressure.is_none());
    same_scene(initial.force_fields.as_ref().unwrap(), &force);
    let accepted = RecordedCarrier::decode(reader.run(), &reader.select(1).unwrap()).unwrap();
    assert_eq!(bits(accepted.state()), expected_state);
    assert_eq!(accepted.dt.unwrap().to_bits(), report.step.dt.to_bits());
    assert_eq!(
        accepted.interval_pressure.as_ref().unwrap().0.to_bits(),
        0.0_f64.to_bits()
    );
    same_scene(accepted.force_fields.as_ref().unwrap(), &force);
    assert_eq!(
        accepted
            .interval_pressure
            .as_ref()
            .unwrap()
            .1
            .iter()
            .map(|p| p.to_bits())
            .collect::<Vec<_>>(),
        expected_pressure
    );
    assert_eq!(reader.status().unwrap().outcome, "failed");
    drop(reader);
    drop(sim);
    archive.cleanup();
}

#[test]
#[ignore = "post-custody numerical/SQLite integration: explicit bounded execution authorization required"]
fn particle_force_archive_round_trip_and_rejected_control_custody() {
    let archive = Archive::new();
    let bounds = limits();
    let force = fields("accepted");
    let changed = fields("changed draft");
    let mut sim = ParticleFluidSimulation::new(
        GridGeometry::new([2; 3], [0.5; 3], [0.0; 3]).unwrap(),
        ParticleFluidConfig {
            fill: DomainFill::EmittedFluid,
            samples_per_axis: 4,
            max_particles: 64,
            memory_limit: 1024 * 1024,
            gravity: [0.0, -1.0, 0.0],
            ..Default::default()
        },
    )
    .unwrap();
    let run = material_run(identity(2), &sim, "prepared bounded case")
        .unwrap()
        .with_force_fields(&force)
        .unwrap();
    let mut writer =
        KeepOnPanic::new(StepRecorder::create(&archive.path(), run.clone(), bounds).unwrap());
    writer
        .append(
            &material_publication_with_force_fields(
                &sim,
                None,
                None,
                &force,
                0,
                bounds.max_record_bytes,
            )
            .unwrap(),
        )
        .unwrap();
    let report = sim
        .step_with_force_fields(1e-6, None, &force, || false)
        .unwrap();
    let controls = MaterialStepControls {
        requested_dt: 1e-6,
        emitter: None,
    };
    let stamp = sim.accepted_force_controls().unwrap();
    same_scene(&stamp.fields, &force);
    assert_eq!(
        stamp.requested_dt.to_bits(),
        controls.requested_dt.to_bits()
    );
    assert_eq!(stamp.report.dt.to_bits(), report.dt.to_bits());
    assert_eq!(stamp.report.particles, 64);
    let expected_particles = particle_bits(sim.particles());
    let (evaluation, diagnostics) = sim.particle_diagnostics().unwrap();
    let expected_diagnostics = diagnostic_bits(diagnostics);
    assert_eq!(stamp.interval_start.to_bits(), evaluation.to_bits());
    let expected_time = sim.time().to_bits();
    let expected_generation = sim.generation();
    writer
        .append(
            &material_publication_with_force_fields(
                &sim,
                Some(report),
                Some(controls),
                &force,
                1,
                bounds.max_record_bytes,
            )
            .unwrap(),
        )
        .unwrap();
    let mut cancellation_calls = 0;
    assert!(matches!(
        sim.step_with_force_fields(1e-6, None, &changed, || {
            cancellation_calls += 1;
            cancellation_calls == 4
        }),
        Err(ParticleFluidError::Cancelled)
    ));
    assert!(matches!(
        sim.step_with_force_fields(0.0, None, &changed, || false),
        Err(ParticleFluidError::InvalidTimeStep)
    ));
    same_scene(&sim.accepted_force_controls().unwrap().fields, &force);
    assert_eq!(particle_bits(sim.particles()), expected_particles);
    assert_eq!(
        diagnostic_bits(sim.particle_diagnostics().unwrap().1),
        expected_diagnostics
    );
    assert_eq!(
        sim.particle_diagnostics().unwrap().0.to_bits(),
        evaluation.to_bits()
    );
    assert_eq!(sim.time().to_bits(), expected_time);
    assert_eq!(sim.generation(), expected_generation);
    assert!(
        material_publication_with_force_fields(
            &sim,
            Some(report),
            Some(controls),
            &changed,
            2,
            bounds.max_record_bytes
        )
        .is_err()
    );
    assert!(
        material_publication_with_force_fields(
            &sim,
            Some(report),
            Some(MaterialStepControls {
                requested_dt: 2e-6,
                emitter: None
            }),
            &force,
            2,
            bounds.max_record_bytes
        )
        .is_err()
    );
    let mut wrong_report = report;
    wrong_report.dt *= 2.0;
    assert!(
        material_publication_with_force_fields(
            &sim,
            Some(wrong_report),
            Some(controls),
            &force,
            2,
            bounds.max_record_bytes
        )
        .is_err()
    );
    let mut rejected = material_publication_with_force_fields(
        &sim,
        Some(report),
        Some(controls),
        &force,
        2,
        bounds.max_record_bytes,
    )
    .unwrap();
    inject_changed_scene(&mut rejected, &changed);
    assert!(matches!(
        writer.append(&rejected),
        Err(RecordingError::Integrity(
            "run/step force document mismatch"
        ))
    ));
    assert_eq!(writer.recorded_count(), 2);
    writer
        .finish(
            "failed",
            "deliberately rejected changed force intent",
            Some(1),
        )
        .unwrap();
    drop(writer);
    let reader = KeepOnPanic::new(RecordingReader::open(&archive.path(), bounds).unwrap());
    reader.verify().unwrap();
    assert_eq!(reader.run(), &run);
    assert_eq!(reader.count().unwrap(), 2);
    let initial = RecordedMaterial::decode(&reader.select(0).unwrap()).unwrap();
    assert!(initial.diagnostics.is_none());
    same_scene(initial.force_fields.as_ref().unwrap(), &force);
    let accepted = RecordedMaterial::decode(&reader.select(1).unwrap()).unwrap();
    same_scene(accepted.force_fields.as_ref().unwrap(), &force);
    assert_eq!(particle_bits(&accepted.particles), expected_particles);
    let (actual_evaluation, actual_diagnostics) = accepted.diagnostics.as_ref().unwrap();
    assert_eq!(actual_evaluation.to_bits(), evaluation.to_bits());
    assert_eq!(diagnostic_bits(actual_diagnostics), expected_diagnostics);
    assert_eq!(accepted.dt.unwrap().to_bits(), report.dt.to_bits());
    assert_eq!(accepted.time.to_bits(), expected_time);
    assert_eq!(accepted.generation, expected_generation);
    assert_eq!(reader.status().unwrap().outcome, "failed");
    drop(reader);
    drop(sim);
    archive.cleanup();
}
