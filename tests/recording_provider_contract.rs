//! Accepted configured-session archive contracts. The authored candidate executes
//! the native solver; fixture package pins are not evidence of a trained model.
#![cfg(not(target_arch = "wasm32"))]

use rheon::{
    recording::{provider::*, *},
    solver_provider::{
        ProviderRejection,
        configured::{
            ConfiguredProviderReport, ContextPressureProvider, ContextPressureRequest,
            ContextProviderIdentity,
        },
    },
    *,
};
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

const CAP: usize = 4 * 1024 * 1024;
const MU: f64 = 0.0078125;

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "rheon-configured-recording-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn archive(&self) -> PathBuf {
        self.0.join("run.sqlite")
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

fn grid() -> GridGeometry {
    GridGeometry::new([5; 3], [0.25; 3], [-1.0, 2.0, 3.0]).unwrap()
}
fn configuration() -> LiquidTransportConfig {
    LiquidTransportConfig {
        carrier: SimulationConfig {
            density: 2.0,
            memory_limit: CAP,
            pressure: PressureSettings {
                relative_residual: 1e-14,
                absolute_residual: 1e-16,
                divergence_limit: 1e-7,
                max_iterations: 3000,
            },
            actual_divergence_limit: 1e-5,
            max_courant: 1.0 / 128.0,
        },
        represented_density: 2.0,
        volume_stamp: VolumeStamp { id: 40, version: 0 },
        carrier_id: 41,
    }
}
fn session() -> SimulationSession {
    session_with(901, 8)
}
fn session_with(run_id: u64, max_steps: usize) -> SimulationSession {
    SimulationSession::new(
        SessionIdentity {
            run_id,
            source_revision: "configured-recording-native-fixture".into(),
        },
        grid(),
        configuration(),
        PressureImplementation::JacobiPcgV1,
        vec![1.0; grid().cell_len()],
        CAP,
        max_steps,
    )
    .unwrap()
}
fn workspace() -> ProviderStepWorkspace {
    ProviderStepWorkspace::new(&grid(), 2.0, CAP, PressureImplementation::JacobiPcgV1).unwrap()
}
fn inputs(forces: &[BodyForce]) -> LiquidStepInputs<'_> {
    LiquidStepInputs {
        requested_dt: 0.0625,
        smoke_source: None,
        forces,
        inlet: LiquidInlet::new(VolumeStamp { id: 42, version: 0 }, [[0.0; 2]; 3]).unwrap(),
        source: None,
        volume: LiquidVolumeSettings {
            max_outward_courant: 1.0,
            actual_divergence_limit: 1e-5,
        },
    }
}
fn forces() -> [BodyForce; 1] {
    [BodyForce {
        value: [0.125, 0.0625, 0.0625],
        units: ForceUnits::Acceleration,
        region: None,
    }]
}

#[derive(Clone, Copy)]
enum Behavior {
    Solve,
    BadPressure,
    Overflow,
    Cancel,
    SupportQualityGate,
    PredictQualityGate,
}
struct Candidate {
    identity: Arc<ContextProviderIdentity>,
    pressure: PressureWorkspace,
    behavior: Behavior,
}
impl Candidate {
    fn new(behavior: Behavior) -> Self {
        Self {
            identity: Arc::new(ContextProviderIdentity {
                id: "authored-native-recording-fixture".into(),
                version: "1".into(),
                manifest_sha256: [3; 32],
                checkpoint_sha256: Some([4; 32]),
                training_run_id: Some("declared-fixture-no-training".into()),
                runtime: "in-process-native-test-fixture".into(),
                source_revision: "authored-fixture".into(),
            }),
            pressure: PressureWorkspace::new(&grid(), CAP).unwrap(),
            behavior,
        }
    }
}
impl ContextPressureProvider for Candidate {
    fn identity(&self) -> Arc<ContextProviderIdentity> {
        Arc::clone(&self.identity)
    }
    fn supports(&self, _: &ContextPressureRequest<'_>) -> Result<(), ProviderRejection> {
        if matches!(self.behavior, Behavior::SupportQualityGate) {
            Err(ProviderRejection::QualityGate)
        } else {
            Ok(())
        }
    }
    fn predict(
        &mut self,
        request: &ContextPressureRequest<'_>,
        pressure: &mut [f64],
        cancelled: &mut dyn FnMut() -> bool,
    ) -> Result<(), ProviderRejection> {
        if matches!(self.behavior, Behavior::Cancel) {
            return Err(ProviderRejection::Cancelled);
        }
        if matches!(self.behavior, Behavior::PredictQualityGate) {
            return Err(ProviderRejection::QualityGate);
        }
        let context = &request.context;
        self.pressure
            .solve_rhs(
                &PressureOperator::new(context.grid(), context.density()).unwrap(),
                context.rhs(),
                context.dt(),
                configuration().carrier.pressure,
                cancelled,
            )
            .map_err(|_| ProviderRejection::Execution("native recording fixture solve failed"))?;
        pressure.copy_from_slice(self.pressure.pressure());
        match self.behavior {
            Behavior::BadPressure => pressure[1] += 100.0,
            Behavior::Overflow => {
                pressure.fill(1e300);
                pressure[0] = 0.0;
            }
            _ => {}
        }
        Ok(())
    }
}

fn bits(values: &[f64]) -> Vec<u64> {
    values.iter().map(|v| v.to_bits()).collect()
}
fn pressure_bits(report: PressureReport) -> [u64; 4] {
    [
        report.iterations as u64,
        report.true_residual_l2.to_bits(),
        report.true_residual_max.to_bits(),
        report.predicted_divergence_max.to_bits(),
    ]
}
fn assert_receipt(actual: &RecordedConfiguredReceipt, expected: &ConfiguredProviderReport) {
    assert_eq!(actual.binding, expected.binding.as_deref().cloned());
    assert_eq!(actual.context, expected.context);
    assert_eq!(
        actual.dynamic_viscosity.to_bits(),
        expected.dynamic_viscosity.to_bits()
    );
    assert_eq!(actual.prediction_called, expected.prediction_called);
    assert_eq!(
        actual.accepted.iterations,
        expected.pressure.accepted.iterations
    );
    assert_eq!(
        pressure_bits(actual.accepted),
        pressure_bits(expected.pressure.accepted)
    );
    assert_eq!(
        actual.candidate.map(pressure_bits),
        expected.pressure.candidate.map(pressure_bits)
    );
    assert_eq!(
        (
            actual.total,
            actual.inference,
            actual.qualification,
            actual.fallback
        ),
        (
            expected.pressure.total,
            expected.pressure.inference,
            expected.pressure.qualification,
            expected.pressure.fallback
        )
    );
    match expected.pressure.rejection {
        None => assert!(actual.rejection.is_none()),
        Some(ProviderRejection::QualityGate) => assert!(matches!(
            actual.rejection,
            Some(RecordedProviderRejection::QualityGate)
        )),
        ref other => panic!("unhandled fixture rejection {other:?}"),
    }
    if expected.pressure.selected == "configured-pressure-context-v1" {
        assert!(matches!(
            actual.selected,
            RecordedPressureSelection::Configured
        ));
    } else {
        assert!(matches!(
            actual.selected,
            RecordedPressureSelection::Native(PressureImplementation::JacobiPcgV1)
        ));
    }
    if let Some(expected) = &expected.pressure.attempted {
        let actual = actual.attempted.as_ref().unwrap();
        assert_eq!(actual.id, expected.id);
        assert_eq!(actual.version, expected.version);
        assert_eq!(actual.stage, expected.stage);
        assert_eq!(actual.input.shape_xyz, expected.input.shape_xyz);
        assert_eq!(actual.input.layout, expected.input.layout);
        assert_eq!(actual.input.scalar, expected.input.scalar);
        assert_eq!(actual.input.units, expected.input.units);
        assert_eq!(actual.output.shape_xyz, expected.output.shape_xyz);
        assert_eq!(actual.output.layout, expected.output.layout);
        assert_eq!(actual.output.scalar, expected.output.scalar);
        assert_eq!(actual.output.units, expected.output.units);
        assert_eq!(actual.domain, expected.domain);
    } else {
        assert!(actual.attempted.is_none());
    }
}

struct Expected {
    configuration: SessionConfiguration,
    carrier_stamp: VolumeStamp,
    fraction_stamp: VolumeStamp,
    fraction_time: f64,
    x: Vec<f32>,
    y: Vec<f32>,
    z: Vec<f32>,
    tracer: Vec<f32>,
    fraction: Vec<f64>,
    pressure: Option<(f64, Vec<f64>)>,
    receipt: Option<ConfiguredProviderReport>,
    viscosity_applied: bool,
    generation: u64,
    time: f64,
    dt: Option<f64>,
}
impl Expected {
    fn snapshot(snapshot: &SessionSnapshot<'_>) -> Self {
        let state = snapshot.state();
        let dt = snapshot.report().map(|report| report.carrier.step.dt);
        Self {
            configuration: *snapshot.configuration(),
            carrier_stamp: state.carrier_stamp,
            fraction_stamp: state.liquid.stamp,
            fraction_time: state.liquid.time,
            x: state.carrier.x.to_vec(),
            y: state.carrier.y.to_vec(),
            z: state.carrier.z.to_vec(),
            tracer: state.carrier.tracer.to_vec(),
            fraction: state.liquid.fraction.to_vec(),
            pressure: snapshot
                .accepted_interval_start()
                .map(|start| (start, state.pressure.to_vec())),
            receipt: snapshot.configured_provider_report().cloned(),
            viscosity_applied: snapshot.report().is_some_and(|r| r.viscosity.is_some()),
            generation: state.carrier.generation,
            time: state.carrier.time,
            dt,
        }
    }
    fn check(&self, actual: &RecordedConfiguredSession) {
        assert_eq!(actual.grid, grid());
        let configuration = actual.configuration;
        assert_eq!(
            configuration.implementation,
            self.configuration.implementation
        );
        assert_eq!(
            configuration.boundary_workspace_limit,
            self.configuration.boundary_workspace_limit
        );
        assert_eq!(configuration.max_steps, self.configuration.max_steps);
        let (actual_config, expected_config) =
            (configuration.transport, self.configuration.transport);
        assert_eq!(actual_config.carrier_id, expected_config.carrier_id);
        assert_eq!(actual_config.volume_stamp, expected_config.volume_stamp);
        assert_eq!(
            actual_config.carrier.memory_limit,
            expected_config.carrier.memory_limit
        );
        assert_eq!(
            actual_config.carrier.pressure.max_iterations,
            expected_config.carrier.pressure.max_iterations
        );
        for (actual, expected) in [
            (
                actual_config.represented_density,
                expected_config.represented_density,
            ),
            (
                actual_config.carrier.density,
                expected_config.carrier.density,
            ),
            (
                actual_config.carrier.actual_divergence_limit,
                expected_config.carrier.actual_divergence_limit,
            ),
            (
                actual_config.carrier.max_courant,
                expected_config.carrier.max_courant,
            ),
            (
                actual_config.carrier.pressure.relative_residual,
                expected_config.carrier.pressure.relative_residual,
            ),
            (
                actual_config.carrier.pressure.absolute_residual,
                expected_config.carrier.pressure.absolute_residual,
            ),
            (
                actual_config.carrier.pressure.divergence_limit,
                expected_config.carrier.pressure.divergence_limit,
            ),
        ] {
            assert_eq!(actual.to_bits(), expected.to_bits());
        }
        assert_eq!(actual.carrier_stamp, self.carrier_stamp);
        assert_eq!(actual.fraction_stamp, self.fraction_stamp);
        assert_eq!(actual.fraction_time.to_bits(), self.fraction_time.to_bits());
        assert_eq!(actual.generation, self.generation);
        assert_eq!(actual.time.to_bits(), self.time.to_bits());
        assert_eq!(actual.dt.map(f64::to_bits), self.dt.map(f64::to_bits));
        for (actual, expected) in [
            (&actual.x, &self.x),
            (&actual.y, &self.y),
            (&actual.z, &self.z),
            (&actual.tracer, &self.tracer),
        ] {
            assert_eq!(
                actual.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
                expected.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
            );
        }
        assert_eq!(bits(&actual.fraction), bits(&self.fraction));
        assert_eq!(
            actual
                .interval_pressure
                .as_ref()
                .map(|(t, p)| (t.to_bits(), bits(p))),
            self.pressure.as_ref().map(|(t, p)| (t.to_bits(), bits(p)))
        );
        match (&actual.receipt, &self.receipt) {
            (Some(actual), Some(expected)) => {
                assert_eq!(actual.viscosity_applied, self.viscosity_applied);
                assert_receipt(actual, expected);
            }
            (None, None) => {}
            _ => panic!("constructor/accepted receipt availability changed"),
        }
    }
}

fn write_sequence(path: &Path) -> Vec<Expected> {
    let mut session = session();
    let mut workspace = workspace();
    let mut viscosity = ViscosityWorkspace::new(grid(), CAP).unwrap();
    let limits = RecordingLimits::default();
    let run = configured_run(
        &session.snapshot(),
        "CPU native fixture; no trained model or restart",
    )
    .unwrap();
    let mut recorder = StepRecorder::create(path, run, limits).unwrap();
    let mut expected = vec![Expected::snapshot(&session.snapshot())];
    recorder
        .append(&configured_publication(&session.snapshot(), 0, limits.max_record_bytes).unwrap())
        .unwrap();
    let force = forces();
    let mut candidate = Candidate::new(Behavior::Solve);
    for (index, behavior) in [
        None,
        Some(Behavior::Solve),
        Some(Behavior::BadPressure),
        Some(Behavior::Overflow),
    ]
    .into_iter()
    .enumerate()
    {
        if let Some(behavior) = behavior {
            candidate.behavior = behavior;
        }
        let preferred = behavior.map(|_| &mut candidate as &mut dyn ContextPressureProvider);
        session
            .step_with_configured_pressure(
                inputs(&force),
                &mut workspace,
                Some((&mut viscosity, MU)),
                preferred,
                None,
                |_| false,
            )
            .unwrap();
        let snapshot = session.snapshot();
        if index == 3 {
            let report = snapshot.configured_provider_report().unwrap();
            assert_eq!(
                report.pressure.rejection,
                Some(ProviderRejection::QualityGate)
            );
            assert!(
                !report
                    .pressure
                    .candidate
                    .unwrap()
                    .true_residual_l2
                    .is_finite()
            );
        }
        expected.push(Expected::snapshot(&snapshot));
        recorder
            .append(&configured_publication(&snapshot, index + 1, limits.max_record_bytes).unwrap())
            .unwrap();
    }
    recorder
        .finish(
            "completed",
            "constructor and four actual accepted intervals",
            Some(4),
        )
        .unwrap();
    expected
}

#[test]
fn actual_constructor_native_candidate_and_quality_fallback_round_trip_exact_bits() {
    let directory = Directory::new();
    let expected = write_sequence(&directory.archive());
    let reader = RecordingReader::open(&directory.archive(), RecordingLimits::default()).unwrap();
    reader.verify().unwrap();
    assert_eq!(reader.count().unwrap(), 5);
    assert_eq!(reader.status().unwrap().accepted_generation, Some(4));
    for (ordinal, expected) in expected.iter().enumerate() {
        let step = reader.select(ordinal).unwrap();
        expected.check(&RecordedConfiguredSession::decode(reader.run(), &step).unwrap());
    }
    let coverage = verify_configured_recording(&reader).unwrap();
    assert_eq!(coverage.verified_prior_links, 3);
    assert_eq!(coverage.unavailable_prior_links, 0);
}

#[test]
fn configured_producer_and_cold_inspector_are_separate_processes() {
    let directory = Directory::new();
    for action in ["write", "read"] {
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "configured_process_child", "--nocapture"])
            .env("RHEON_CONFIGURED_RECORDING_CHILD", action)
            .env("RHEON_CONFIGURED_RECORDING_PATH", directory.archive())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout)
                .contains(&format!("configured child {action} complete"))
        );
    }
}

#[test]
fn configured_process_child() {
    let Ok(action) = std::env::var("RHEON_CONFIGURED_RECORDING_CHILD") else {
        return;
    };
    let path = PathBuf::from(std::env::var_os("RHEON_CONFIGURED_RECORDING_PATH").unwrap());
    if action == "write" {
        write_sequence(&path);
    } else {
        assert_eq!(action, "read");
        // No SimulationSession, workspace, provider or runtime constructed here.
        let reader = RecordingReader::open(&path, RecordingLimits::default()).unwrap();
        reader.verify().unwrap();
        assert_eq!(reader.status().unwrap().outcome, "completed");
        assert_eq!(reader.count().unwrap(), 5);
        assert_eq!(
            verify_configured_recording(&reader)
                .unwrap()
                .verified_prior_links,
            3
        );
        let decoded: Vec<_> = (0..5)
            .map(|ordinal| {
                RecordedConfiguredSession::decode(reader.run(), &reader.select(ordinal).unwrap())
                    .unwrap()
            })
            .collect();
        assert!(decoded[0].receipt.is_none());
        assert!(matches!(
            decoded[1].receipt.as_ref().unwrap().selected,
            RecordedPressureSelection::Native(_)
        ));
        assert!(matches!(
            decoded[2].receipt.as_ref().unwrap().selected,
            RecordedPressureSelection::Configured
        ));
        assert!(matches!(
            decoded[3].receipt.as_ref().unwrap().rejection,
            Some(RecordedProviderRejection::QualityGate)
        ));
        assert!(
            !decoded[4]
                .receipt
                .as_ref()
                .unwrap()
                .candidate
                .unwrap()
                .true_residual_l2
                .is_finite()
        );
        assert_eq!(decoded[4].generation, 4);
    }
    println!("configured child {action} complete");
}

#[test]
fn record_limits_preserve_prefix_and_declare_actual_unrecorded_accepted_tail() {
    let directory = Directory::new();
    let mut session = session();
    let limits = RecordingLimits {
        max_records: 2,
        ..RecordingLimits::default()
    };
    let run = configured_run(&session.snapshot(), "bounded prefix fixture").unwrap();
    let mut recorder = StepRecorder::create(&directory.archive(), run, limits).unwrap();
    recorder
        .append(&configured_publication(&session.snapshot(), 0, limits.max_record_bytes).unwrap())
        .unwrap();
    assert!(matches!(
        configured_publication(&session.snapshot(), 1, 1),
        Err(RecordingError::Limit(_))
    ));
    let mut workspace = workspace();
    let force = forces();
    for ordinal in 1..=2 {
        session
            .step_with_configured_pressure(inputs(&force), &mut workspace, None, None, None, |_| {
                false
            })
            .unwrap();
        let step =
            configured_publication(&session.snapshot(), ordinal, limits.max_record_bytes).unwrap();
        let result = recorder.append(&step);
        if ordinal == 1 {
            result.unwrap();
        } else {
            assert!(matches!(result, Err(RecordingError::Limit(_))));
        }
    }
    assert_eq!(recorder.recorded_count(), 2);
    recorder
        .finish(
            "failed",
            "record limit; accepted generation2 has no retained row",
            Some(2),
        )
        .unwrap();
    drop(recorder);
    let reader = RecordingReader::open(&directory.archive(), limits).unwrap();
    reader.verify().unwrap();
    assert_eq!(reader.count().unwrap(), 2);
    assert_eq!(reader.select(1).unwrap().generation, 1);
    assert!(matches!(reader.select(2), Err(RecordingError::Missing(2))));
    assert_eq!(reader.status().unwrap().outcome, "failed");
    assert_eq!(reader.status().unwrap().accepted_generation, Some(2));
    verify_configured_recording(&reader).unwrap();
}

#[test]
fn cancelled_candidate_does_not_publish_or_replace_the_previous_receipt() {
    let directory = Directory::new();
    let mut session = session();
    let limits = RecordingLimits::default();
    let run = configured_run(&session.snapshot(), "cancelled interval fixture").unwrap();
    let mut recorder = StepRecorder::create(&directory.archive(), run, limits).unwrap();
    recorder
        .append(&configured_publication(&session.snapshot(), 0, limits.max_record_bytes).unwrap())
        .unwrap();
    let mut workspace = workspace();
    let force = forces();
    session
        .step_with_configured_pressure(inputs(&force), &mut workspace, None, None, None, |_| false)
        .unwrap();
    let before = configured_publication(&session.snapshot(), 1, limits.max_record_bytes).unwrap();
    recorder.append(&before).unwrap();
    let mut candidate = Candidate::new(Behavior::Cancel);
    assert!(
        session
            .step_with_configured_pressure(
                inputs(&force),
                &mut workspace,
                None,
                Some(&mut candidate),
                None,
                |_| false
            )
            .is_err()
    );
    let after = configured_publication(&session.snapshot(), 1, limits.max_record_bytes).unwrap();
    assert_eq!(after.generation, before.generation);
    assert_eq!(after.time.to_bits(), before.time.to_bits());
    assert_eq!(after.controls, before.controls);
    assert_eq!(after.fields, before.fields);
    assert_eq!(recorder.recorded_count(), 2);
    recorder
        .finish(
            "cancelled",
            "candidate cancelled; last accepted publication retained",
            Some(1),
        )
        .unwrap();
    drop(recorder);
    let reader = RecordingReader::open(&directory.archive(), limits).unwrap();
    reader.verify().unwrap();
    assert_eq!(reader.status().unwrap().outcome, "cancelled");
    assert_eq!(reader.status().unwrap().accepted_generation, Some(1));
    assert_eq!(reader.count().unwrap(), 2);
    RecordedConfiguredSession::decode(reader.run(), &reader.select(1).unwrap()).unwrap();
}

fn accepted_fixture() -> (RecordingRun, RecordedStep) {
    let mut session = session();
    let run = configured_run(&session.snapshot(), "typed decoder negative fixture").unwrap();
    let mut workspace = workspace();
    let force = forces();
    session
        .step_with_configured_pressure(inputs(&force), &mut workspace, None, None, None, |_| false)
        .unwrap();
    (
        run,
        configured_publication(
            &session.snapshot(),
            0,
            RecordingLimits::default().max_record_bytes,
        )
        .unwrap(),
    )
}

#[test]
fn typed_decoder_rejects_generic_valid_wrong_profile_descriptors_and_native_values() {
    type Mutation = fn(&mut RecordingRun, &mut RecordedStep);
    let mutations: [(&str, Mutation); 9] = [
        ("profile", |run, _| {
            run.solver = "future-configured-session-v99".into()
        }),
        ("version", |_, step| {
            assert!(step.controls.starts_with("RCP2"));
            step.controls.replace_range(..4, "RCP3");
        }),
        ("geometry", |run, _| run.counts[0] += 1),
        ("units", |_, step| {
            step.fields
                .iter_mut()
                .find(|f| f.name == "interval_pressure")
                .unwrap()
                .units = "m/s".into()
        }),
        ("association", |_, step| {
            step.fields
                .iter_mut()
                .find(|f| f.name == "interval_pressure")
                .unwrap()
                .association = "incorrect-cell-association".into()
        }),
        ("stage", |_, step| {
            step.fields
                .iter_mut()
                .find(|f| f.name == "interval_pressure")
                .unwrap()
                .stage = "wrong-pressure-stage".into()
        }),
        ("interval", |_, step| {
            step.fields
                .iter_mut()
                .find(|f| f.name == "interval_pressure")
                .unwrap()
                .evaluation_time = Some(step.time)
        }),
        ("gauge", |_, step| {
            step.fields
                .iter_mut()
                .find(|f| f.name == "interval_pressure")
                .unwrap()
                .data[..8]
                .copy_from_slice(&1.0_f64.to_le_bytes())
        }),
        ("missing-fraction", |_, step| {
            step.fields.retain(|f| f.name != "fraction")
        }),
    ];
    for (name, mutate) in mutations {
        let directory = Directory::new();
        let (mut run, mut step) = accepted_fixture();
        mutate(&mut run, &mut step);
        let limits = RecordingLimits::default();
        let mut recorder = StepRecorder::create(&directory.archive(), run, limits).unwrap();
        recorder.append(&step).unwrap();
        recorder
            .finish(
                "completed",
                "generic-byte-valid typed semantic negative",
                Some(1),
            )
            .unwrap();
        drop(recorder);
        let reader = RecordingReader::open(&directory.archive(), limits).unwrap();
        reader.verify().unwrap();
        let selected = reader.select(0).unwrap();
        let error = RecordedConfiguredSession::decode(reader.run(), &selected)
            .err()
            .expect(name);
        if matches!(name, "profile" | "version") {
            assert!(
                matches!(error, RecordingError::Unsupported(_)),
                "{name}: {error}"
            );
        } else {
            assert!(
                matches!(error, RecordingError::Invalid(_)),
                "{name}: {error}"
            );
        }
        assert!(verify_configured_recording(&reader).is_err(), "{name}");
    }
}

#[test]
fn nonfinite_native_field_is_rejected_by_generic_writer_before_any_row_commits() {
    let directory = Directory::new();
    let (run, mut step) = accepted_fixture();
    step.fields
        .iter_mut()
        .find(|field| field.name == "interval_pressure")
        .unwrap()
        .data[8..16]
        .copy_from_slice(&f64::NAN.to_le_bytes());
    let limits = RecordingLimits::default();
    let mut recorder = StepRecorder::create(&directory.archive(), run, limits).unwrap();
    assert!(matches!(
        recorder.append(&step),
        Err(RecordingError::Invalid("nonfinite field"))
    ));
    assert_eq!(recorder.recorded_count(), 0);
    drop(recorder);
    let reader = RecordingReader::open(&directory.archive(), limits).unwrap();
    reader.verify().unwrap();
    assert_eq!(reader.count().unwrap(), 0);
    assert_eq!(reader.status().unwrap().outcome, "active");
}

#[test]
fn history_verification_refuses_sparse_prefix_and_changed_retained_prior() {
    let source = Directory::new();
    write_sequence(&source.archive());
    let reader = RecordingReader::open(&source.archive(), RecordingLimits::default()).unwrap();
    // A row can be decoded independently, but the canonical archive writer
    // refuses to relabel a missing initial prefix as a continuous recording.
    let sparse = Directory::new();
    let mut selected = reader.select(2).unwrap();
    selected.ordinal = 0;
    RecordedConfiguredSession::decode(reader.run(), &selected).unwrap();
    let mut recorder = StepRecorder::create(
        &sparse.archive(),
        reader.run().clone(),
        RecordingLimits::default(),
    )
    .unwrap();
    assert!(matches!(
        recorder.append(&selected),
        Err(RecordingError::Invalid("unrecorded initial steps"))
    ));
    assert_eq!(recorder.recorded_count(), 0);
    drop(recorder);

    // This remains generic hash-valid and each publication is individually
    // decodable, but the retained predecessor no longer matches the exact
    // native-pressure digest named by the candidate's authoritative context.
    let changed = Directory::new();
    let mut prior = reader.select(1).unwrap();
    prior.ordinal = 0;
    let pressure = prior
        .fields
        .iter_mut()
        .find(|field| field.name == "interval_pressure")
        .unwrap();
    let value = f64::from_le_bytes(pressure.data[8..16].try_into().unwrap());
    pressure.data[8..16].copy_from_slice(&(value + 0.25).to_le_bytes());
    let mut selected = reader.select(2).unwrap();
    selected.ordinal = 1;
    let mut recorder = StepRecorder::create(
        &changed.archive(),
        reader.run().clone(),
        RecordingLimits::default(),
    )
    .unwrap();
    recorder.append(&prior).unwrap();
    recorder.append(&selected).unwrap();
    recorder
        .finish(
            "completed",
            "changed predecessor semantic negative",
            Some(2),
        )
        .unwrap();
    drop(recorder);
    let changed_reader =
        RecordingReader::open(&changed.archive(), RecordingLimits::default()).unwrap();
    changed_reader.verify().unwrap();
    for ordinal in 0..2 {
        RecordedConfiguredSession::decode(
            changed_reader.run(),
            &changed_reader.select(ordinal).unwrap(),
        )
        .unwrap();
    }
    assert!(matches!(
        verify_configured_recording(&changed_reader),
        Err(RecordingError::Invalid(_))
    ));
}

#[test]
fn nonbinary_native_intervals_retain_actual_previous_publication_time_bits() {
    let directory = Directory::new();
    let mut session = session();
    let mut workspace = workspace();
    let limits = RecordingLimits::default();
    let run = configured_run(&session.snapshot(), "actual interval-start lineage fixture").unwrap();
    let mut recorder = StepRecorder::create(&directory.archive(), run, limits).unwrap();
    recorder
        .append(&configured_publication(&session.snapshot(), 0, limits.max_record_bytes).unwrap())
        .unwrap();
    let force = forces();
    let mut starts = Vec::new();
    let mut subtraction_loses_bits = false;
    for ordinal in 1..=6 {
        let actual_start = session.snapshot().state().carrier.time;
        let mut input = inputs(&force);
        input.requested_dt = 1.0 / 60.0;
        session
            .step_with_configured_pressure(input, &mut workspace, None, None, None, |_| false)
            .unwrap();
        let snapshot = session.snapshot();
        let record = configured_publication(&snapshot, ordinal, limits.max_record_bytes).unwrap();
        let stored_start = record
            .field("interval_pressure")
            .unwrap()
            .evaluation_time
            .unwrap();
        assert_eq!(stored_start.to_bits(), actual_start.to_bits());
        subtraction_loses_bits |=
            (record.time - record.dt.unwrap()).to_bits() != actual_start.to_bits();
        starts.push(actual_start);
        recorder.append(&record).unwrap();
    }
    assert!(
        subtraction_loses_bits,
        "fixture must distinguish actual start from end-dt reconstruction"
    );
    recorder
        .finish("completed", "six actual native intervals", Some(6))
        .unwrap();
    drop(recorder);
    let reader = RecordingReader::open(&directory.archive(), limits).unwrap();
    reader.verify().unwrap();
    verify_configured_recording(&reader).unwrap();
    for (index, expected_start) in starts.into_iter().enumerate() {
        let selected = reader.select(index + 1).unwrap();
        let decoded = RecordedConfiguredSession::decode(reader.run(), &selected).unwrap();
        assert_eq!(
            decoded.interval_pressure.unwrap().0.to_bits(),
            expected_start.to_bits()
        );
    }

    // A native row has no prior-context stamp. A locally plausible alternate
    // start must still fail exact continuity with the retained predecessor.
    let changed_ordinal = (1..reader.count().unwrap())
        .find(|&ordinal| {
            let step = reader.select(ordinal).unwrap();
            let dt = step.dt.unwrap();
            let inverse = step.time - dt;
            let start = step
                .field("interval_pressure")
                .unwrap()
                .evaluation_time
                .unwrap();
            inverse.to_bits() != start.to_bits() && (inverse + dt).to_bits() == step.time.to_bits()
        })
        .expect("nonbinary fixture must supply a locally valid alternate start");
    let changed = Directory::new();
    let mut recorder =
        StepRecorder::create(&changed.archive(), reader.run().clone(), limits).unwrap();
    for ordinal in 0..reader.count().unwrap() {
        let mut step = reader.select(ordinal).unwrap();
        if ordinal == changed_ordinal {
            let inverse = step.time - step.dt.unwrap();
            step.fields
                .iter_mut()
                .find(|field| field.name == "interval_pressure")
                .unwrap()
                .evaluation_time = Some(inverse);
            // Row-local decoding cannot prove its predecessor's actual time.
            RecordedConfiguredSession::decode(reader.run(), &step).unwrap();
        }
        recorder.append(&step).unwrap();
    }
    recorder
        .finish(
            "completed",
            "authored alternate interval-start negative",
            Some(6),
        )
        .unwrap();
    drop(recorder);
    let cold = RecordingReader::open(&changed.archive(), limits).unwrap();
    cold.verify().unwrap();
    assert!(matches!(
        verify_configured_recording(&cold),
        Err(RecordingError::Invalid(_))
    ));
}

#[test]
fn terminal_generation_cannot_exceed_the_actual_session_step_quota() {
    let directory = Directory::new();
    let (run, step) = accepted_fixture();
    let limits = RecordingLimits::default();
    let mut recorder = StepRecorder::create(&directory.archive(), run, limits).unwrap();
    recorder.append(&step).unwrap();
    // Deliberately authored terminal metadata; no generation9 was simulated.
    recorder
        .finish("failed", "forged terminal generation above quota8", Some(9))
        .unwrap();
    drop(recorder);
    let reader = RecordingReader::open(&directory.archive(), limits).unwrap();
    reader.verify().unwrap();
    RecordedConfiguredSession::decode(reader.run(), &reader.select(0).unwrap()).unwrap();
    assert!(matches!(
        verify_configured_recording(&reader),
        Err(RecordingError::Invalid(_))
    ));
}

#[test]
fn different_session_identity_or_configuration_cannot_attach_to_the_same_run_header() {
    let owner = session();
    let run = configured_run(&owner.snapshot(), "actual owner header").unwrap();
    for (run_id, max_steps) in [(902, 8), (901, 9)] {
        let directory = Directory::new();
        let mut other = session_with(run_id, max_steps);
        let mut workspace = workspace();
        let force = forces();
        other
            .step_with_configured_pressure(inputs(&force), &mut workspace, None, None, None, |_| {
                false
            })
            .unwrap();
        let step = configured_publication(
            &other.snapshot(),
            0,
            RecordingLimits::default().max_record_bytes,
        )
        .unwrap();
        let limits = RecordingLimits::default();
        let mut recorder = StepRecorder::create(&directory.archive(), run.clone(), limits).unwrap();
        recorder.append(&step).unwrap();
        recorder
            .finish(
                "completed",
                "generic-valid source-binding negative",
                Some(1),
            )
            .unwrap();
        drop(recorder);
        let reader = RecordingReader::open(&directory.archive(), limits).unwrap();
        reader.verify().unwrap();
        assert!(matches!(
            RecordedConfiguredSession::decode(reader.run(), &reader.select(0).unwrap()),
            Err(RecordingError::Invalid(_))
        ));
        assert!(verify_configured_recording(&reader).is_err());
    }
}

fn little_endian_hex(bits: u64) -> String {
    bits.to_le_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn authored_rejected_candidate_diagnostic_vectors_preserve_ieee_bits_without_changing_accepted_fields()
 {
    let source = Directory::new();
    write_sequence(&source.archive());
    let reader = RecordingReader::open(&source.archive(), RecordingLimits::default()).unwrap();
    let original = reader.select(4).unwrap();
    let decoded = RecordedConfiguredSession::decode(reader.run(), &original).unwrap();
    let original_bits = decoded
        .receipt
        .as_ref()
        .unwrap()
        .candidate
        .unwrap()
        .true_residual_l2
        .to_bits();
    assert!(!f64::from_bits(original_bits).is_finite());
    let original_hex = little_endian_hex(original_bits);
    assert_eq!(
        original.controls.matches(&original_hex).count(),
        1,
        "fixture must identify only the actual rejected candidate L2, without encoded offsets"
    );
    // These are explicitly authored codec vectors. Only original_bits above is
    // observed from the actual algorithm; these do not claim numerical outputs.
    for diagnostic_bits in [
        0x7ff8_0000_0000_0042, // quiet NaN payload
        0x7ff0_0000_0000_0042, // signaling NaN payload
        0xfff8_0000_0000_0042, // signed NaN payload
        f64::INFINITY.to_bits(),
        f64::NEG_INFINITY.to_bits(),
        (-0.0_f64).to_bits(),
        1, // smallest positive subnormal
        1e-200_f64.to_bits(),
        0.0_f64.to_bits(),
        1.25_f64.to_bits(),
        (-1.0_f64).to_bits(), // finite negative residual is an invalid diagnostic
    ] {
        let directory = Directory::new();
        let mut changed = reader.select(4).unwrap();
        changed.controls =
            changed
                .controls
                .replacen(&original_hex, &little_endian_hex(diagnostic_bits), 1);
        assert_eq!(changed.fields, original.fields);
        let mut recorder = StepRecorder::create(
            &directory.archive(),
            reader.run().clone(),
            RecordingLimits::default(),
        )
        .unwrap();
        for ordinal in 0..4 {
            recorder.append(&reader.select(ordinal).unwrap()).unwrap();
        }
        recorder.append(&changed).unwrap();
        recorder
            .finish(
                "completed",
                "authored rejected diagnostic codec vector",
                Some(4),
            )
            .unwrap();
        drop(recorder);
        let cold = RecordingReader::open(&directory.archive(), RecordingLimits::default()).unwrap();
        cold.verify().unwrap();
        let step = cold.select(4).unwrap();
        assert_eq!(step.fields, original.fields);
        let result = RecordedConfiguredSession::decode(cold.run(), &step);
        if diagnostic_bits == (-1.0_f64).to_bits() {
            assert!(matches!(result, Err(RecordingError::Invalid(_))));
        } else {
            let changed = result.unwrap();
            let actual = changed.receipt.as_ref().unwrap();
            assert_eq!(
                actual.candidate.unwrap().true_residual_l2.to_bits(),
                diagnostic_bits
            );
            assert_eq!(
                pressure_bits(actual.accepted),
                pressure_bits(decoded.receipt.as_ref().unwrap().accepted)
            );
            assert!(matches!(
                actual.rejection,
                Some(RecordedProviderRejection::QualityGate)
            ));
            assert_eq!(
                bits(&changed.interval_pressure.unwrap().1),
                bits(&decoded.interval_pressure.as_ref().unwrap().1)
            );
            verify_configured_recording(&cold).unwrap();
        }
    }

    // Locate the complete accepted diagnostic tuple, not guessed codec offsets.
    // Nonfinite observations may survive only on rejected candidate diagnostics.
    let accepted = decoded.receipt.as_ref().unwrap().accepted;
    let accepted_hex: String = pressure_bits(accepted)
        .into_iter()
        .map(little_endian_hex)
        .collect();
    assert_eq!(original.controls.matches(&accepted_hex).count(), 1);
    for invalid in [
        PressureReport {
            true_residual_l2: f64::NAN,
            ..accepted
        },
        PressureReport {
            predicted_divergence_max: configuration().carrier.pressure.divergence_limit * 2.0,
            ..accepted
        },
        PressureReport {
            iterations: configuration().carrier.pressure.max_iterations + 1,
            ..accepted
        },
    ] {
        let invalid_hex: String = pressure_bits(invalid)
            .into_iter()
            .map(little_endian_hex)
            .collect();
        let mut changed = reader.select(4).unwrap();
        changed.controls = changed.controls.replacen(&accepted_hex, &invalid_hex, 1);
        let directory = Directory::new();
        let mut recorder = StepRecorder::create(
            &directory.archive(),
            reader.run().clone(),
            RecordingLimits::default(),
        )
        .unwrap();
        for ordinal in 0..4 {
            recorder.append(&reader.select(ordinal).unwrap()).unwrap();
        }
        recorder.append(&changed).unwrap();
        recorder
            .finish(
                "completed",
                "authored invalid accepted diagnostic negative",
                Some(4),
            )
            .unwrap();
        drop(recorder);
        let cold = RecordingReader::open(&directory.archive(), RecordingLimits::default()).unwrap();
        cold.verify().unwrap();
        assert!(matches!(
            RecordedConfiguredSession::decode(cold.run(), &cold.select(4).unwrap()),
            Err(RecordingError::Invalid(_))
        ));
    }
}

#[test]
fn empty_unrelated_profile_cannot_pass_the_configured_archive_verifier() {
    let directory = Directory::new();
    let mut run =
        configured_run(&session().snapshot(), "empty unsupported profile fixture").unwrap();
    run.solver = "unrelated-profile-v1".into();
    let recorder =
        StepRecorder::create(&directory.archive(), run, RecordingLimits::default()).unwrap();
    drop(recorder);
    let reader = RecordingReader::open(&directory.archive(), RecordingLimits::default()).unwrap();
    reader.verify().unwrap();
    assert_eq!(reader.count().unwrap(), 0);
    assert!(matches!(
        verify_configured_recording(&reader),
        Err(RecordingError::Unsupported(_))
    ));
}

#[test]
fn explicit_provider_quality_gate_rejections_need_no_host_candidate_diagnostic() {
    for behavior in [Behavior::SupportQualityGate, Behavior::PredictQualityGate] {
        let directory = Directory::new();
        let mut session = session();
        let limits = RecordingLimits::default();
        let run = configured_run(
            &session.snapshot(),
            "actual provider-declared QualityGate rejection fixture",
        )
        .unwrap();
        let mut recorder = StepRecorder::create(&directory.archive(), run, limits).unwrap();
        recorder
            .append(
                &configured_publication(&session.snapshot(), 0, limits.max_record_bytes).unwrap(),
            )
            .unwrap();
        let mut workspace = workspace();
        let force = forces();
        session
            .step_with_configured_pressure(inputs(&force), &mut workspace, None, None, None, |_| {
                false
            })
            .unwrap();
        recorder
            .append(
                &configured_publication(&session.snapshot(), 1, limits.max_record_bytes).unwrap(),
            )
            .unwrap();
        let mut candidate = Candidate::new(behavior);
        let (_, report) = session
            .step_with_configured_pressure(
                inputs(&force),
                &mut workspace,
                None,
                Some(&mut candidate),
                None,
                |_| false,
            )
            .unwrap();
        assert_eq!(
            report.pressure.rejection,
            Some(ProviderRejection::QualityGate)
        );
        assert!(report.pressure.candidate.is_none());
        assert_eq!(
            report.prediction_called,
            matches!(behavior, Behavior::PredictQualityGate)
        );
        let expected = Expected::snapshot(&session.snapshot());
        recorder
            .append(
                &configured_publication(&session.snapshot(), 2, limits.max_record_bytes).unwrap(),
            )
            .unwrap();
        recorder
            .finish(
                "completed",
                "provider rejection retained with native fallback",
                Some(2),
            )
            .unwrap();
        drop(recorder);
        let reader = RecordingReader::open(&directory.archive(), limits).unwrap();
        reader.verify().unwrap();
        let selected = reader.select(2).unwrap();
        expected.check(&RecordedConfiguredSession::decode(reader.run(), &selected).unwrap());
        assert_eq!(
            verify_configured_recording(&reader)
                .unwrap()
                .verified_prior_links,
            1
        );
    }
}

#[test]
fn composition_reset_context_absence_cannot_bypass_retained_same_viscosity_history() {
    let original = Directory::new();
    let mut a = session();
    let limits = RecordingLimits::default();
    let run = configured_run(&a.snapshot(), "actual viscosity-reset fixture").unwrap();
    let mut recorder = StepRecorder::create(&original.archive(), run.clone(), limits).unwrap();
    recorder
        .append(&configured_publication(&a.snapshot(), 0, limits.max_record_bytes).unwrap())
        .unwrap();
    let mut workspace_a = workspace();
    let mut viscosity_a = ViscosityWorkspace::new(grid(), CAP).unwrap();
    let force = forces();
    a.step_with_configured_pressure(
        inputs(&force),
        &mut workspace_a,
        Some((&mut viscosity_a, MU)),
        None,
        None,
        |_| false,
    )
    .unwrap();
    recorder
        .append(&configured_publication(&a.snapshot(), 1, limits.max_record_bytes).unwrap())
        .unwrap();
    let mut candidate = Candidate::new(Behavior::Solve);
    let (_, report) = a
        .step_with_configured_pressure(
            inputs(&force),
            &mut workspace_a,
            Some((&mut viscosity_a, 2.0 * MU)),
            Some(&mut candidate),
            None,
            |_| false,
        )
        .unwrap();
    assert!(report.binding.is_some());
    assert!(report.context.is_none());
    assert!(!report.prediction_called);
    assert!(matches!(
        report.pressure.rejection,
        Some(ProviderRejection::Unsupported(_))
    ));
    recorder
        .append(&configured_publication(&a.snapshot(), 2, limits.max_record_bytes).unwrap())
        .unwrap();
    recorder
        .finish(
            "completed",
            "actual changed-viscosity native fallback",
            Some(2),
        )
        .unwrap();
    drop(recorder);
    let original_reader = RecordingReader::open(&original.archive(), limits).unwrap();
    verify_configured_recording(&original_reader).unwrap();

    // Both source rows are untouched actual captures with the same run identity
    // and constructor configuration. Only their cross-call composition is false.
    let mut b = session();
    let mut workspace_b = workspace();
    let mut viscosity_b = ViscosityWorkspace::new(grid(), CAP).unwrap();
    b.step_with_configured_pressure(
        inputs(&force),
        &mut workspace_b,
        Some((&mut viscosity_b, 2.0 * MU)),
        None,
        None,
        |_| false,
    )
    .unwrap();
    let prior = configured_publication(&b.snapshot(), 0, limits.max_record_bytes).unwrap();
    let mut next = original_reader.select(2).unwrap();
    next.ordinal = 1;
    RecordedConfiguredSession::decode(&run, &prior).unwrap();
    RecordedConfiguredSession::decode(&run, &next).unwrap();
    let changed = Directory::new();
    let mut recorder = StepRecorder::create(&changed.archive(), run, limits).unwrap();
    recorder.append(&prior).unwrap();
    recorder.append(&next).unwrap();
    recorder
        .finish(
            "completed",
            "authored cross-call context-absence negative",
            Some(2),
        )
        .unwrap();
    drop(recorder);
    let reader = RecordingReader::open(&changed.archive(), limits).unwrap();
    reader.verify().unwrap();
    assert!(matches!(
        verify_configured_recording(&reader),
        Err(RecordingError::Invalid(
            "configured missing available context"
        ))
    ));
}

#[test]
fn constructor_header_accessor_validates_failed_start_without_publications() {
    let directory = Directory::new();
    let session = session();
    let run = configured_run(&session.snapshot(), "empty failed-start header").unwrap();
    let mut recorder = StepRecorder::create(
        &directory.archive(),
        run.clone(),
        RecordingLimits::default(),
    )
    .unwrap();
    recorder
        .finish("failed", "start failed before any publication", Some(0))
        .unwrap();
    drop(recorder);
    let reader = RecordingReader::open(&directory.archive(), RecordingLimits::default()).unwrap();
    assert_eq!(reader.count().unwrap(), 0);
    verify_configured_recording(&reader).unwrap();
    let header = configured_configuration(reader.run()).unwrap();
    let row = configured_publication(
        &session.snapshot(),
        0,
        RecordingLimits::default().max_record_bytes,
    )
    .unwrap();
    let mut decoded = RecordedConfiguredSession::decode(&run, &row).unwrap();
    decoded.configuration = header;
    // Existing bitwise comparator covers every constructor control.
    Expected::snapshot(&session.snapshot()).check(&decoded);
    for mutation in 0..4 {
        let mut changed = run.clone();
        match mutation {
            0 => changed.solver = "future-profile".into(),
            1 => changed.configuration.push_str("00"),
            2 => changed.counts = [0; 3],
            _ => changed.identity.source_revision.clear(),
        }
        assert!(configured_configuration(&changed).is_err());
    }
}

#[test]
fn disabled_and_applied_zero_viscosity_reopen_in_separate_processes() {
    for applied in [false, true] {
        let directory = Directory::new();
        let mut session = session();
        let mut workspace = workspace();
        let mut viscosity = ViscosityWorkspace::new(grid(), CAP).unwrap();
        let force = forces();
        let run = configured_run(&session.snapshot(), "actual zero versus disabled stage").unwrap();
        let limits = RecordingLimits::default();
        let mut recorder = StepRecorder::create(&directory.archive(), run, limits).unwrap();
        recorder
            .append(
                &configured_publication(&session.snapshot(), 0, limits.max_record_bytes).unwrap(),
            )
            .unwrap();
        session
            .step_with_configured_pressure(
                inputs(&force),
                &mut workspace,
                applied.then_some((&mut viscosity, 0.0)),
                None,
                None,
                |_| false,
            )
            .unwrap();
        assert_eq!(
            session.snapshot().report().unwrap().viscosity.is_some(),
            applied
        );
        recorder
            .append(
                &configured_publication(&session.snapshot(), 1, limits.max_record_bytes).unwrap(),
            )
            .unwrap();
        recorder
            .finish(
                "completed",
                "actual accepted zero or disabled stage",
                Some(1),
            )
            .unwrap();
        drop(recorder);
        drop(session);
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "viscosity_stage_cold_child", "--nocapture"])
            .env("RHEON_STAGE_COLD_PATH", directory.archive())
            .env(
                "RHEON_STAGE_COLD_APPLIED",
                if applied { "true" } else { "false" },
            )
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("cold viscosity stage verified"));
    }
}

#[test]
fn viscosity_stage_cold_child() {
    let Some(path) = std::env::var_os("RHEON_STAGE_COLD_PATH") else {
        return;
    };
    // Only a reader and typed codecs: no Session/workspace/provider/runtime.
    let reader = RecordingReader::open(&PathBuf::from(path), RecordingLimits::default()).unwrap();
    verify_configured_recording(&reader).unwrap();
    let initial =
        RecordedConfiguredSession::decode(reader.run(), &reader.select(0).unwrap()).unwrap();
    assert!(initial.receipt.is_none());
    let accepted =
        RecordedConfiguredSession::decode(reader.run(), &reader.select(1).unwrap()).unwrap();
    let receipt = accepted.receipt.unwrap();
    assert_eq!(
        receipt.viscosity_applied,
        std::env::var("RHEON_STAGE_COLD_APPLIED").unwrap() == "true"
    );
    assert_eq!(receipt.dynamic_viscosity.to_bits(), 0.0_f64.to_bits());
    assert!(receipt.binding.is_none());
    println!("cold viscosity stage verified");
}

#[test]
fn cold_typed_decode_refuses_nonzero_viscosity_without_stage_and_old_receipt_version() {
    let mut session = session();
    let mut workspace = workspace();
    let mut viscosity = ViscosityWorkspace::new(grid(), CAP).unwrap();
    let force = forces();
    let limits = RecordingLimits::default();
    let run = configured_run(&session.snapshot(), "authored stage absence negative").unwrap();
    session
        .step_with_configured_pressure(
            inputs(&force),
            &mut workspace,
            Some((&mut viscosity, MU)),
            None,
            None,
            |_| false,
        )
        .unwrap();
    for old_version in [false, true] {
        let directory = Directory::new();
        let mut row =
            configured_publication(&session.snapshot(), 0, limits.max_record_bytes).unwrap();
        if old_version {
            row.controls.replace_range(..4, "RCP1");
        } else {
            // RCP2 source/stamps/time then receipt/binding/context/stage flags.
            // This native-only receipt has neither binding nor context.
            let offset = 5 + (32 + 16 + 16 + 8) * 2;
            assert_eq!(&row.controls[offset..offset + 8], "01000001");
            row.controls.replace_range(offset + 6..offset + 8, "00");
        }
        let mut recorder = StepRecorder::create(&directory.archive(), run.clone(), limits).unwrap();
        recorder.append(&row).unwrap();
        recorder
            .finish(
                "completed",
                "authored generic-valid semantic negative",
                Some(1),
            )
            .unwrap();
        drop(recorder);
        let reader = RecordingReader::open(&directory.archive(), limits).unwrap();
        reader.verify().unwrap();
        assert!(
            RecordedConfiguredSession::decode(reader.run(), &reader.select(0).unwrap()).is_err()
        );
        assert!(verify_configured_recording(&reader).is_err());
    }
}

#[test]
fn cold_decode_enforces_applied_zero_density_domain_and_normal_viscosity() {
    let limits = RecordingLimits::default();
    for unequal_density in [false, true] {
        let directory = Directory::new();
        let mut config = configuration();
        if unequal_density {
            config.represented_density = 1.0;
        }
        let mut session = SimulationSession::new(
            SessionIdentity {
                run_id: 901,
                source_revision: "actual viscosity domain fixture".into(),
            },
            grid(),
            config,
            PressureImplementation::JacobiPcgV1,
            vec![1.0; grid().cell_len()],
            CAP,
            8,
        )
        .unwrap();
        let run = configured_run(
            &session.snapshot(),
            "authored generic-valid viscosity domain negative",
        )
        .unwrap();
        let mut workspace = workspace();
        let force = forces();
        // Unequal densities are legitimate without a viscosity stage.
        session
            .step_with_configured_pressure(inputs(&force), &mut workspace, None, None, None, |_| {
                false
            })
            .unwrap();
        let mut row =
            configured_publication(&session.snapshot(), 0, limits.max_record_bytes).unwrap();
        RecordedConfiguredSession::decode(&run, &row).unwrap();
        let flag = 5 + (32 + 16 + 16 + 8 + 3) * 2;
        assert_eq!(&row.controls[flag..flag + 2], "00");
        row.controls.replace_range(flag..flag + 2, "01");
        if !unequal_density {
            // Positive subnormal: finite but forbidden by the real stage.
            assert_eq!(&row.controls[flag + 2..flag + 18], "0000000000000000");
            row.controls
                .replace_range(flag + 2..flag + 18, "0100000000000000");
        }
        let mut recorder = StepRecorder::create(&directory.archive(), run, limits).unwrap();
        recorder.append(&row).unwrap();
        recorder
            .finish(
                "completed",
                "authored impossible applied-stage domain",
                Some(1),
            )
            .unwrap();
        drop(recorder);
        let reader = RecordingReader::open(&directory.archive(), limits).unwrap();
        reader.verify().unwrap();
        assert!(matches!(
            RecordedConfiguredSession::decode(reader.run(), &reader.select(0).unwrap()),
            Err(RecordingError::Invalid(
                "configured viscous domain" | "configured viscosity"
            ))
        ));
        assert!(verify_configured_recording(&reader).is_err());
    }
}
