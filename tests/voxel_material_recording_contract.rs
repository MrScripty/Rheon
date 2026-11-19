#![cfg(not(target_arch = "wasm32"))]
use rheon::{recording::*, voxel_material::*, *};
use std::sync::atomic::{AtomicUsize, Ordering};
fn simulation() -> VoxelMaterialSimulation {
    let grid = GridGeometry::new([4; 3], [0.25; 3], [-0.0, -0.5, 0.125]).unwrap();
    let fractions = (0..64)
        .map(|i| if i % 4 < 2 && i / 16 < 2 { 1.0 } else { 0.0 })
        .collect::<Vec<_>>();
    VoxelMaterialSimulation::new(
        grid,
        MaterialPair::water_air(),
        &fractions,
        VoxelMaterialConfig {
            gravity: [-0.0, -1.0, 0.0],
            pressure: PressureSettings {
                relative_residual: 1e-11,
                absolute_residual: 1e-12,
                divergence_limit: 1e-8,
                max_iterations: 2000,
            },
            memory_limit: 64 * 1024 * 1024,
        },
    )
    .unwrap()
}
fn header(sim: &VoxelMaterialSimulation) -> RecordingRun {
    voxel_material_run(
        SessionIdentity {
            run_id: 73,
            source_revision: "voxel-recording-test".into(),
        },
        sim,
        [37; 32],
        "test declaration; no benchmark/native validation",
    )
    .unwrap()
}
fn path() -> std::path::PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "rheon-voxel-recording-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&dir).unwrap();
    dir.join("run.sqlite")
}
const LIMIT: usize = 128 * 1024;
fn exact(sim: &VoxelMaterialSimulation, record: &RecordedVoxelMaterial) {
    assert_eq!(record.liquid_fraction, sim.state().liquid_fraction());
    for (d, axis) in [Axis::X, Axis::Y, Axis::Z].into_iter().enumerate() {
        assert_eq!(record.velocity[d], sim.state().velocity(axis));
    }
    assert_eq!(record.metadata.initial_input_sha256, [37; 32]);
    let a = record.metadata.materials;
    let b = sim.state().materials();
    assert_eq!(
        [
            a.liquid().density(),
            a.liquid().viscosity(),
            a.gas().density(),
            a.gas().viscosity()
        ]
        .map(f64::to_bits),
        [
            b.liquid().density(),
            b.liquid().viscosity(),
            b.gas().density(),
            b.gas().viscosity()
        ]
        .map(f64::to_bits)
    );
    assert_eq!(
        record.metadata.config.gravity.map(f64::to_bits),
        sim.config().gravity.map(f64::to_bits)
    );
    assert_eq!(record.grid, *sim.state().geometry());
    assert_eq!(record.generation, sim.state().generation());
    assert_eq!(record.time.to_bits(), sim.state().time().to_bits());
}
#[test]
fn exact_constructor_and_accepted_fields_roundtrip_through_actual_sqlite() {
    let path = path();
    let mut sim = simulation();
    let run = header(&sim);
    let limits = RecordingLimits {
        max_records: 4,
        max_record_bytes: LIMIT,
        max_database_bytes: 1024 * 1024,
    };
    let mut writer = StepRecorder::create(&path, run.clone(), limits).unwrap();
    let initial = voxel_material_publication(&sim, 0, LIMIT).unwrap();
    assert!(initial.field("interval_pressure").is_none());
    let retained = RecordedVoxelMaterial::decode(&run, &initial).unwrap();
    exact(&sim, &retained);
    assert!(retained.interval_pressure.is_none());
    writer.append(&initial).unwrap();
    for ordinal in 1..=3 {
        sim.step(0.001, |_| false).unwrap();
        let step = voxel_material_publication(&sim, ordinal, LIMIT).unwrap();
        let decoded = RecordedVoxelMaterial::decode(&run, &step).unwrap();
        exact(&sim, &decoded);
        assert_eq!(
            decoded.interval_pressure.as_ref().unwrap().1,
            sim.state().pressure().unwrap()
        );
        assert_eq!(step.field("interval_pressure").unwrap().scalar_type, "f64");
        writer.append(&step).unwrap();
    }
    // Earlier inspection values own their bytes despite subsequent simulation.
    assert_eq!(retained.generation, 0);
    assert_eq!(retained.liquid_fraction.iter().sum::<f64>(), 16.0);
    writer
        .finish("completed", "three accepted component intervals", Some(3))
        .unwrap();
    drop(writer);
    let reader = RecordingReader::open(&path, limits).unwrap();
    assert_eq!(reader.status().unwrap().outcome, "completed");
    let step = reader.select(3).unwrap();
    let cold = RecordedVoxelMaterial::decode(reader.run(), &step).unwrap();
    exact(&sim, &cold);
    assert!(RecordedCarrier::decode(reader.run(), &step).is_err());
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
}
#[test]
fn strict_field_method_and_interval_validation_refuses_reinterpretation() {
    let mut sim = simulation();
    sim.step(0.001, |_| false).unwrap();
    let run = header(&sim);
    let make = || voxel_material_publication(&sim, 1, LIMIT).unwrap();
    let mut wrong = run.clone();
    wrong.solver = "rheon-carrier-full-box-v1/jacobi-pcg".into();
    assert!(matches!(
        RecordedVoxelMaterial::decode(&wrong, &make()),
        Err(RecordingError::Unsupported("voxel recording method"))
    ));
    wrong = run.clone();
    wrong.configuration.push('0');
    assert!(matches!(
        RecordedVoxelMaterial::decode(&wrong, &make()),
        Err(RecordingError::Invalid("voxel configuration length"))
    ));
    let mut step = make();
    step.fields[0].scalar_type = "f32".into();
    assert!(RecordedVoxelMaterial::decode(&run, &step).is_err());
    let mut step = make();
    step.fields[0].association = "cell_center".into();
    assert!(matches!(
        RecordedVoxelMaterial::decode(&run, &step),
        Err(RecordingError::Invalid("voxel field schema"))
    ));
    let mut step = make();
    step.fields[3].data[..8].copy_from_slice(&1.1_f64.to_le_bytes());
    assert!(matches!(
        RecordedVoxelMaterial::decode(&run, &step),
        Err(RecordingError::Invalid("voxel fraction range"))
    ));
    let mut step = make();
    step.fields[0].data[..8].copy_from_slice(&1.0_f64.to_le_bytes());
    assert!(matches!(
        RecordedVoxelMaterial::decode(&run, &step),
        Err(RecordingError::Invalid("voxel closed wall velocity"))
    ));
    let mut step = make();
    step.fields[4].evaluation_time = Some(step.time);
    assert!(matches!(
        RecordedVoxelMaterial::decode(&run, &step),
        Err(RecordingError::Invalid("voxel pressure interval"))
    ));
    let mut step = make();
    step.fields[4].data[..8].copy_from_slice(&1.0_f64.to_le_bytes());
    assert!(matches!(
        RecordedVoxelMaterial::decode(&run, &step),
        Err(RecordingError::Invalid("voxel pressure gauge"))
    ));
    let mut step = make();
    step.fields.pop();
    assert!(matches!(
        RecordedVoxelMaterial::decode(&run, &step),
        Err(RecordingError::Invalid("voxel field inventory"))
    ));
}
#[test]
fn payload_admission_and_cancelled_step_keep_exact_last_accepted_publication() {
    let mut sim = simulation();
    assert!(matches!(
        voxel_material_publication(&sim, 0, 1),
        Err(RecordingError::Limit("voxel publication payload"))
    ));
    sim.step(0.001, |_| false).unwrap();
    let before = voxel_material_publication(&sim, 1, LIMIT).unwrap();
    let mut reached = false;
    assert!(matches!(
        sim.step(0.001, |stage| {
            if stage == VoxelMaterialStage::BeforeCommit {
                reached = true;
                true
            } else {
                false
            }
        }),
        Err(VoxelMaterialError::Cancelled)
    ));
    assert!(reached);
    let after = voxel_material_publication(&sim, 1, LIMIT).unwrap();
    assert_eq!(before.fields, after.fields);
    assert_eq!(before.controls, after.controls);
    assert_eq!(before.time, after.time);
}

#[test]
fn exported_publication_budget_admits_constructor_and_accepted_payload() {
    let mut sim = simulation();
    for ordinal in 0..=1 {
        let accepted = ordinal != 0;
        let budget = voxel_material_publication_bytes(sim.state().geometry(), accepted).unwrap();
        let publication = voxel_material_publication(&sim, ordinal, budget).unwrap();
        assert_eq!(publication.fields.len(), if accepted { 5 } else { 4 });
        assert!(matches!(
            voxel_material_publication(&sim, ordinal, budget - 1),
            Err(RecordingError::Limit("voxel publication payload"))
        ));
        if !accepted {
            sim.step(0.001, |_| false).unwrap();
        }
    }
}

#[test]
fn interval_timestamp_preserves_original_clock_instead_of_inverse_subtraction() {
    let original = simulation();
    let mut sim = VoxelMaterialSimulation::new(
        original.state().geometry().clone(),
        original.state().materials(),
        original.state().liquid_fraction(),
        VoxelMaterialConfig {
            gravity: [0.0; 3],
            ..original.config()
        },
    )
    .unwrap();
    let mut exposed = false;
    for ordinal in 1..=3 {
        let start = sim.state().time();
        let report = sim.step(0.1, |_| false).unwrap();
        let step = voxel_material_publication(&sim, ordinal, LIMIT).unwrap();
        assert_eq!(report.interval_start.to_bits(), start.to_bits());
        assert_eq!(
            step.field("interval_pressure")
                .unwrap()
                .evaluation_time
                .unwrap()
                .to_bits(),
            start.to_bits()
        );
        exposed |= (report.time - report.dt).to_bits() != start.to_bits();
    }
    assert!(exposed, "fixture must expose inverse-subtraction rounding");
}
