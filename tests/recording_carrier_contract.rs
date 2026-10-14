#![cfg(not(target_arch = "wasm32"))]
use rheon::{recording::*, *};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
fn path() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "rheon-carrier-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&dir).unwrap();
    dir.join("run.sqlite")
}
fn config() -> SimulationConfig {
    SimulationConfig {
        density: 1.0,
        memory_limit: 64 * 1024 * 1024,
        pressure: PressureSettings {
            relative_residual: 1e-10,
            absolute_residual: 1e-12,
            divergence_limit: 1e-8,
            max_iterations: 2000,
        },
        actual_divergence_limit: 1e-5,
        max_courant: 1.0,
    }
}
fn sim(method: PressureImplementation) -> Simulation {
    Simulation::with_implementation(
        GridGeometry::new([3, 4, 5], [0.2, 0.25, 0.3], [-2.0, 3.0, 5.0]).unwrap(),
        config(),
        method,
    )
    .unwrap()
}
fn source() -> SmokeSource {
    SmokeSource {
        lower: [-1.9, 3.1, 5.1],
        upper: [-1.6, 3.7, 6.1],
        tracer_rate: 2.0,
        vertical_acceleration: 1.0,
    }
}
fn identity() -> SessionIdentity {
    SessionIdentity {
        run_id: u64::MAX,
        source_revision: "carrier-fixture-declared-source".into(),
    }
}
fn arrays(xml: &str, name: &str) -> Vec<f64> {
    let marker = format!("Name=\"{name}\"");
    let begin = xml.find(&marker).unwrap();
    let begin = begin + xml[begin..].find('>').unwrap() + 1;
    let end = begin + xml[begin..].find("</DataArray>").unwrap();
    xml[begin..end]
        .split_whitespace()
        .map(|v| v.parse().unwrap())
        .collect()
}
#[test]
fn both_methods_round_trip_non_cubic_native_bits_actual_times_and_vtk() {
    for method in PressureImplementation::ALL {
        let mut sim = sim(method);
        let p = path();
        let limits = RecordingLimits::default();
        let run = carrier_run(identity(), &sim, "CPU fixture; no restart").unwrap();
        let mut writer = StepRecorder::create(&p, run.clone(), limits).unwrap();
        writer
            .append(&carrier_publication(&sim, 0, limits.max_record_bytes).unwrap())
            .unwrap();
        let ctor = carrier_publication(&sim, 0, limits.max_record_bytes).unwrap();
        assert_eq!(ctor.fields.len(), 4);
        assert!(ctor.field("interval_pressure").is_none());
        let mut selected_bytes = Vec::new();
        let mut expected_faces = Vec::new();
        let mut expected_tracer = Vec::new();
        let mut expected_pressure = Vec::new();
        let mut expected_start = 0.0;
        let mut expected_time = 0.0;
        for ordinal in 1..=3 {
            let old_time = sim.state().time;
            let report = sim.step(0.003, Some(source()), |_| false).unwrap();
            let pressure = sim.accepted_interval_pressure().unwrap();
            assert_eq!(pressure.interval_start.to_bits(), old_time.to_bits());
            assert_eq!(pressure.report.dt.to_bits(), report.dt.to_bits());
            assert_eq!(
                pressure.controls.requested_dt.to_bits(),
                0.003_f64.to_bits()
            );
            let record = carrier_publication(&sim, ordinal, limits.max_record_bytes).unwrap();
            assert_eq!(record.fields.len(), 5);
            assert!(record.controls.contains("tracer_rate: 2.0"));
            writer.append(&record).unwrap();
            if ordinal == 1 {
                expected_faces = [sim.state().x, sim.state().y, sim.state().z]
                    .iter()
                    .map(|v| v.iter().map(|v| v.to_bits()).collect::<Vec<_>>())
                    .collect();
                expected_tracer = sim.state().tracer.iter().map(|v| v.to_bits()).collect();
                expected_pressure = pressure.values.iter().map(|v| v.to_bits()).collect();
                expected_start = old_time;
                expected_time = sim.state().time;
                write_carrier_vti(
                    &mut selected_bytes,
                    &run.identity,
                    sim.grid(),
                    sim.state(),
                    Some(report.dt),
                    Some((old_time, pressure.values)),
                )
                .unwrap();
            }
        }
        writer
            .finish("completed", "all carrier accepted", Some(3))
            .unwrap();
        drop(writer);
        drop(sim);
        let reader = RecordingReader::open(&p, limits).unwrap();
        reader.verify().unwrap();
        let record = reader.select_generation(1).unwrap();
        let selected = RecordedCarrier::decode(reader.run(), &record).unwrap();
        assert_eq!(selected.implementation, method);
        assert_eq!(selected.time.to_bits(), expected_time.to_bits());
        for (actual, expected) in [&selected.x, &selected.y, &selected.z]
            .iter()
            .zip(&expected_faces)
        {
            assert_eq!(
                actual.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
                *expected
            );
        }
        assert_eq!(
            selected
                .tracer
                .iter()
                .map(|v| v.to_bits())
                .collect::<Vec<_>>(),
            expected_tracer
        );
        let (start, pressure) = selected.interval_pressure.as_ref().unwrap();
        assert_eq!(start.to_bits(), expected_start.to_bits());
        assert_eq!(
            pressure.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            expected_pressure
        );
        let mut bytes = Vec::new();
        selected.write_vti(&mut bytes, &run.identity).unwrap();
        assert_eq!(bytes, selected_bytes);
        let xml = String::from_utf8(bytes).unwrap();
        assert!(xml.contains("WholeExtent=\"0 3 0 4 0 5\""));
        assert!(!xml.contains("Name=\"fraction\""));
        assert_eq!(
            arrays(&xml, "pressure_interval_s"),
            vec![expected_start, expected_time]
        );
        assert_eq!(arrays(&xml, "velocity_x_dimensions"), vec![4.0, 4.0, 5.0]);
        assert_eq!(arrays(&xml, "interval_pressure_pa"), *pressure);
        assert_eq!(
            arrays(&xml, "tracer")
                .iter()
                .map(|v| (*v as f32).to_bits())
                .collect::<Vec<_>>(),
            selected
                .tracer
                .iter()
                .map(|v| v.to_bits())
                .collect::<Vec<_>>()
        );
        let initial = RecordedCarrier::decode(reader.run(), &reader.select(0).unwrap()).unwrap();
        let mut bytes = Vec::new();
        initial.write_vti(&mut bytes, &run.identity).unwrap();
        assert!(
            !String::from_utf8(bytes)
                .unwrap()
                .contains("Name=\"interval_pressure_pa\"")
        );
        drop(reader);
        std::fs::remove_dir_all(p.parent().unwrap()).unwrap();
    }
}
#[test]
fn rejected_cancelled_and_unsupported_attempts_invalidate_pressure_without_publication() {
    let mut sim = sim(PressureImplementation::default());
    let limit = RecordingLimits::default().max_record_bytes;
    for stage in [StepStage::PressureIteration, StepStage::BeforeCommit] {
        sim.step(0.003, Some(source()), |_| false).unwrap();
        let old = carrier_publication(&sim, 1, limit).unwrap();
        let result = sim.step(0.004, Some(source()), |s| s == stage);
        match stage {
            StepStage::PressureIteration => assert!(matches!(
                result,
                Err(SimulationError::Pressure(PressureError::Cancelled { .. }))
            )),
            _ => assert!(matches!(result,Err(SimulationError::Cancelled{stage:s}) if s==stage)),
        }
        assert!(sim.accepted_interval_pressure().is_none());
        assert!(matches!(
            carrier_publication(&sim, 2, limit),
            Err(RecordingError::Unsupported(_))
        ));
        let state = sim.state();
        assert_eq!(state.generation, old.generation);
        assert_eq!(state.time.to_bits(), old.time.to_bits());
        assert_eq!(
            old.field("velocity_x").unwrap().data,
            state
                .x
                .iter()
                .flat_map(|v| v.to_le_bytes())
                .collect::<Vec<_>>()
        );
    }
    sim.step(0.003, Some(source()), |_| false).unwrap();
    assert!(sim.accepted_interval_pressure().is_some());
    assert!(sim.step_with_forces(-1.0, None, &[], |_| false).is_err());
    assert!(sim.accepted_interval_pressure().is_none());
    let mut fresh = crate::sim(PressureImplementation::default());
    fresh.step(0.003, None, |_| false).unwrap();
    fresh.step_with_forces(0.003, None, &[], |_| false).unwrap();
    assert!(fresh.accepted_interval_pressure().is_none());
    assert!(carrier_publication(&fresh, 1, limit).is_err());
    fresh.step(0.003, None, |_| false).unwrap();
    fresh.reset().unwrap();
    assert!(fresh.accepted_interval_pressure().is_none());
    assert!(carrier_publication(&fresh, 0, limit).is_err());
}
#[test]
fn emitter_actual_reduced_interval_and_controls_are_retained() {
    for method in PressureImplementation::ALL {
        let mut sim = Simulation::with_implementation(
            GridGeometry::new([4; 3], [0.25; 3], [0.0; 3]).unwrap(),
            config(),
            method,
        )
        .unwrap();
        let emitter = RotatingCylinderEmitter {
            pressure_heads: [0.0; 2],
            rotations_per_second: 10.0,
            ..Default::default()
        };
        let report = sim.step_with_emitter(0.05, emitter, |_| false).unwrap();
        assert!(report.dt < 0.05);
        let view = sim.accepted_interval_pressure().unwrap();
        assert!(view.controls.source.is_none());
        assert_eq!(view.controls.emitter.unwrap().rotations_per_second, 10.0);
        assert_eq!(view.controls.requested_dt, 0.05);
        assert_eq!(view.interval_start, 0.0);
        let record =
            carrier_publication(&sim, 0, RecordingLimits::default().max_record_bytes).unwrap();
        assert_eq!(record.dt, Some(report.dt));
        assert!(record.controls.contains("rotations_per_second: 10.0"));
        let run = carrier_run(identity(), &sim, "emitter actual").unwrap();
        RecordedCarrier::decode(&run, &record).unwrap();
    }
}
#[test]
fn canonical_carrier_rejects_missing_schema_interval_gauge_and_payload_overflow() {
    let mut sim = sim(PressureImplementation::default());
    sim.step(0.003, Some(source()), |_| false).unwrap();
    let run = carrier_run(identity(), &sim, "negative").unwrap();
    let limit = RecordingLimits::default().max_record_bytes;
    for mutation in 0..7 {
        let mut step = carrier_publication(&sim, 0, limit).unwrap();
        match mutation {
            0 => {
                step.fields.pop();
            }
            1 => {
                step.fields[0].association = "cell_center".into();
            }
            2 => {
                step.fields[0].dimensions = [5, 4, 4];
            }
            3 => {
                step.fields[4].units = "kg".into();
            }
            4 => {
                step.fields[4].evaluation_time = Some(step.time);
            }
            5 => {
                step.fields[4].data[0..8].copy_from_slice(&1.0_f64.to_le_bytes());
            }
            6 => {
                step.fields[3].data[0..4].copy_from_slice(&2.0_f32.to_le_bytes());
            }
            _ => unreachable!(),
        }
        let error = RecordedCarrier::decode(&run, &step)
            .err()
            .expect("negative must reject");
        match mutation {
            0 => assert!(matches!(
                error,
                RecordingError::Invalid("carrier field inventory")
            )),
            1..=3 => assert!(matches!(
                error,
                RecordingError::Invalid("carrier field schema")
            )),
            4 => assert!(matches!(error, RecordingError::Invalid("carrier interval"))),
            5 => assert!(matches!(
                error,
                RecordingError::Invalid("carrier pressure gauge")
            )),
            6 => assert!(matches!(
                error,
                RecordingError::Invalid("carrier tracer range")
            )),
            _ => unreachable!(),
        }
    }
    assert!(matches!(
        carrier_publication(&sim, 0, 1),
        Err(RecordingError::Limit("carrier record payload"))
    ));
    let mut bytes = Vec::new();
    let view = sim.accepted_interval_pressure().unwrap();
    assert!(
        write_carrier_vti(
            &mut bytes,
            &identity(),
            sim.grid(),
            sim.state(),
            Some(view.report.dt),
            Some((sim.state().time, view.values))
        )
        .is_err()
    );
    assert!(bytes.is_empty());
}
