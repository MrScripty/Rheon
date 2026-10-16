//! Real accepted-Session/SQLite custody contracts. The authored candidate calls
//! the native pressure solver; its declared fixture pins prove no model result.
#![cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
#[path = "../src/bin/desktop/run_sets.rs"]
#[allow(dead_code)] // The native schema-one API has its own contract suite.
mod run_sets;

use rheon::{
    recording::{provider::*, *},
    solver_provider::{
        ProviderRejection,
        configured::{ContextPressureProvider, ContextPressureRequest, ContextProviderIdentity},
        context::{PressureContextMode, PressureHistoryStatus},
    },
    *,
};
use run_sets::{
    ArchiveBudget, MemberState, TerminalOutcome,
    configured::{archive::*, *},
};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

const CAP: usize = 1024 * 1024;
const RHO: f64 = 2.0;
const MU: f64 = 1.0 / 128.0;
const DT: f64 = 1.0 / 16.0;
const RUN_ID: u64 = 901;
const PHASE: &str = "RHEON_CONFIGURED_BINDING_PHASE";
const ROOT: &str = "RHEON_CONFIGURED_BINDING_ROOT";
const PROCESS_TEST: &str =
    "producer_exits_before_fresh_process_inspects_frozen_members_without_runtime";

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "rheon-configured-binding-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn grid() -> GridGeometry {
    GridGeometry::new([2, 1, 1], [0.25; 3], [0.0; 3]).unwrap()
}
fn executor() -> ExecutorDeclaration {
    ExecutorDeclaration {
        id: "real-Session-contract-host".into(),
        source_revision: "configured-binding-host-fixture".into(),
        build_provenance: None,
    }
}
fn configuration() -> LiquidTransportConfig {
    LiquidTransportConfig {
        carrier: SimulationConfig {
            density: RHO,
            memory_limit: CAP,
            pressure: PressureSettings {
                relative_residual: 1e-12,
                absolute_residual: 1e-12,
                divergence_limit: 1e-9,
                max_iterations: 100,
            },
            actual_divergence_limit: 1e-5,
            max_courant: 0.5,
        },
        represented_density: RHO,
        volume_stamp: VolumeStamp {
            id: 710,
            version: 0,
        },
        carrier_id: 711,
    }
}
fn session_from(
    configuration: SessionConfiguration,
    identity: SessionIdentity,
) -> SimulationSession {
    assert_ne!(
        std::env::var(PHASE).ok().as_deref(),
        Some("inspect"),
        "cold phase must never construct Session"
    );
    SimulationSession::new(
        identity,
        grid(),
        configuration.transport,
        configuration.implementation,
        vec![1.0; 2],
        configuration.boundary_workspace_limit,
        configuration.max_steps,
    )
    .unwrap()
}
fn constructor() -> SimulationSession {
    session_from(
        SessionConfiguration {
            transport: configuration(),
            implementation: PressureImplementation::JacobiPcgV1,
            boundary_workspace_limit: CAP,
            max_steps: 128,
        },
        SessionIdentity {
            run_id: RUN_ID,
            source_revision: executor().source_revision,
        },
    )
}
fn fixture_identity() -> ContextProviderIdentity {
    ContextProviderIdentity {
        id: "authored-native-pressure-candidate".into(),
        version: "1".into(),
        manifest_sha256: [13; 32],
        checkpoint_sha256: None,
        training_run_id: None,
        runtime: "in-process native pressure test fixture; no learned model".into(),
        source_revision: "authored-fixture-source".into(),
    }
}
fn future() -> ConfiguredFutureIntent {
    ConfiguredFutureIntent {
        requested_dt_s: DT,
        requested_steps: 3,
        forces: vec![ForceSpec {
            value: [0.125, 0.0, 0.0],
            units: ForceUnitsSpec::Acceleration,
            region: None,
        }],
        volume: VolumeControls {
            max_outward_courant: 1.0,
            actual_divergence_per_s: 1e-5,
        },
        viscosity: ViscosityIntent::NewtonianConstant {
            dynamic_viscosity_pa_s: MU,
        },
        pressure: PressureIntent::NativeOnly,
        provider_workspace_limit_bytes: CAP,
        viscosity_workspace_limit_bytes: Some(CAP),
        recording: ArchiveBudget {
            max_database_bytes: 8 * 1024 * 1024,
            max_record_bytes: CAP,
            max_records: 4,
        },
        capture_constructor: true,
        executor: executor(),
    }
}
fn member(name: &str, configured: bool) -> ConfiguredRunSetMember {
    let mut intent = future();
    if configured {
        intent.pressure = PressureIntent::NativePrefixThenConfigured {
            provider: Box::new(ConfiguredProviderPin::capture(&fixture_identity()).unwrap()),
            deployment: DeploymentLocator::Absolute(
                "/nonexistent/rheon-binding-fixture/deployment.json".into(),
            ),
            requested_native_prefix_steps: 1,
            minimum_native_prefix_steps: 1,
        };
    }
    ConfiguredRunSetMember {
        member_id: name.into(),
        spec: FrozenConfiguredSpec::capture_constructor(&constructor().snapshot(), intent).unwrap(),
        differences_from_baseline: vec![],
        attempt: Some(ConfiguredRunAttempt {
            attempt_id: format!("{name}-attempt"),
            archive_relative_path: format!("{name}.sqlite"),
            expected_run_id: RUN_ID,
            expected_source_revision: executor().source_revision,
            finalized: None,
        }),
    }
}
fn reseal(member: &mut ConfiguredRunSetMember) {
    member.spec =
        FrozenConfiguredSpec::new(member.spec.constructor.clone(), member.spec.future.clone())
            .unwrap();
}
#[derive(Clone, Copy)]
enum Behavior {
    Solve,
    BadOnce,
    Decline,
}
struct Candidate {
    identity: Arc<ContextProviderIdentity>,
    pressure: PressureWorkspace,
    settings: PressureSettings,
    behavior: Behavior,
    calls: usize,
}
impl Candidate {
    fn new(
        settings: PressureSettings,
        behavior: Behavior,
        identity: ContextProviderIdentity,
    ) -> Self {
        assert_ne!(
            std::env::var(PHASE).ok().as_deref(),
            Some("inspect"),
            "cold phase must never create provider"
        );
        Self {
            identity: Arc::new(identity),
            pressure: PressureWorkspace::new(&grid(), CAP).unwrap(),
            settings,
            behavior,
            calls: 0,
        }
    }
}
impl ContextPressureProvider for Candidate {
    fn identity(&self) -> Arc<ContextProviderIdentity> {
        Arc::clone(&self.identity)
    }
    fn supports(&self, _: &ContextPressureRequest<'_>) -> Result<(), ProviderRejection> {
        if matches!(self.behavior, Behavior::Decline) {
            Err(ProviderRejection::Unsupported("authored fixture declines"))
        } else {
            Ok(())
        }
    }
    fn predict(
        &mut self,
        request: &ContextPressureRequest<'_>,
        out: &mut [f64],
        cancel: &mut dyn FnMut() -> bool,
    ) -> Result<(), ProviderRejection> {
        let c = &request.context;
        assert_eq!(c.mode(), PressureContextMode::AcceptedCandidate);
        if let Some(prior) = c.prior() {
            assert_eq!(prior.status, PressureHistoryStatus::Accepted);
            assert_eq!(prior.generation + 1, c.generation());
            assert_eq!(prior.interval_end.to_bits(), c.interval_start().to_bits());
        }
        self.pressure
            .solve_rhs(
                &PressureOperator::new(c.grid(), c.density()).unwrap(),
                c.rhs(),
                c.dt(),
                self.settings,
                cancel,
            )
            .map_err(|_| ProviderRejection::Execution("authored native candidate failed"))?;
        out.copy_from_slice(self.pressure.pressure());
        self.calls += 1;
        if matches!(self.behavior, Behavior::BadOnce) && self.calls == 1 {
            out[1] += 1.0;
        }
        Ok(())
    }
}
struct Execution {
    dt: f64,
    mu: Option<f64>,
    provider_after: Option<usize>,
    intervals: usize,
    record_constructor: bool,
    terminal: Option<&'static str>,
    limits: RecordingLimits,
    behavior: Behavior,
    identity: ContextProviderIdentity,
}
impl Execution {
    fn frozen(m: &ConfiguredRunSetMember) -> Self {
        let f = &m.spec.future;
        Self {
            dt: f.requested_dt_s,
            mu: match f.viscosity {
                ViscosityIntent::Disabled => None,
                ViscosityIntent::NewtonianConstant {
                    dynamic_viscosity_pa_s,
                } => Some(dynamic_viscosity_pa_s),
            },
            provider_after: match &f.pressure {
                PressureIntent::NativeOnly => None,
                PressureIntent::NativePrefixThenConfigured {
                    requested_native_prefix_steps,
                    ..
                } => Some(*requested_native_prefix_steps),
            },
            intervals: f.requested_steps,
            record_constructor: f.capture_constructor,
            terminal: Some("completed"),
            limits: f.recording.limits(),
            behavior: Behavior::Solve,
            identity: fixture_identity(),
        }
    }
}
fn inputs<'a>(
    m: &ConfiguredRunSetMember,
    forces: &'a [BodyForce],
    dt: f64,
) -> LiquidStepInputs<'a> {
    LiquidStepInputs {
        requested_dt: dt,
        smoke_source: None,
        forces,
        inlet: LiquidInlet::new(
            VolumeStamp {
                id: 712,
                version: 0,
            },
            [[0.0; 2]; 3],
        )
        .unwrap(),
        source: None,
        volume: m.spec.future.volume.settings(),
    }
}
/// Test producer only: existing Session and recorder retain numerical and
/// transaction ownership; no runtime, alternative archive or worker is added.
fn produce(root: &Path, m: &ConfiguredRunSetMember, e: Execution) -> Vec<RecordedStep> {
    let attempt = m.attempt.as_ref().unwrap();
    let mut s = m
        .spec
        .create_session(SessionIdentity {
            run_id: attempt.expected_run_id,
            source_revision: attempt.expected_source_revision.clone(),
        })
        .unwrap();
    let run = bound_configured_run(&m.spec, &s.snapshot()).unwrap();
    let mut writer =
        StepRecorder::create(&root.join(&attempt.archive_relative_path), run, e.limits).unwrap();
    let mut publications = vec![];
    if e.record_constructor {
        let publication =
            configured_publication(&s.snapshot(), 0, e.limits.max_record_bytes).unwrap();
        writer.append(&publication).unwrap();
        publications.push(publication);
    }
    let mut w = ProviderStepWorkspace::new(
        &grid(),
        RHO,
        CAP,
        m.spec.constructor.pressure_method.implementation(),
    )
    .unwrap();
    let mut v = ViscosityWorkspace::new(grid(), CAP).unwrap();
    let mut candidate = Candidate::new(
        m.spec.constructor.transport().carrier.pressure,
        e.behavior,
        e.identity,
    );
    let forces: Vec<_> = m
        .spec
        .future
        .forces
        .iter()
        .map(ForceSpec::body_force)
        .collect();
    for interval in 0..e.intervals {
        let preferred = e
            .provider_after
            .filter(|prefix| interval >= *prefix)
            .map(|_| &mut candidate as &mut dyn ContextPressureProvider);
        let viscosity = e.mu.map(|mu| (&mut v, mu));
        s.step_with_configured_pressure(
            inputs(m, &forces, e.dt),
            &mut w,
            viscosity,
            preferred,
            None,
            |_| false,
        )
        .unwrap();
        let publication =
            configured_publication(&s.snapshot(), publications.len(), e.limits.max_record_bytes)
                .unwrap();
        writer.append(&publication).unwrap();
        publications.push(publication);
    }
    if let Some(outcome) = e.terminal {
        writer
            .finish(
                outcome,
                "actual test producer terminal",
                Some(s.snapshot().state().carrier.generation),
            )
            .unwrap();
    }
    publications
}
fn pin(root: &Path, m: &mut ConfiguredRunSetMember) {
    m.attempt
        .as_mut()
        .unwrap()
        .pin_closed(root, &m.spec)
        .unwrap();
}
fn ready(root: &Path, m: &ConfiguredRunSetMember) -> RecordingReader {
    match open_member(root, m) {
        MemberState::Ready(r) => *r,
        MemberState::Invalid(e) => panic!("invalid member: {e}"),
        _ => panic!("expected pinned Ready member"),
    }
}
fn exact(step: &RecordedStep) -> Value {
    json!({"ordinal":step.ordinal,"generation":step.generation,"time_bits":step.time.to_bits(),
        "dt_bits":step.dt.map(f64::to_bits),"constructor":step.stage==SnapshotStage::Constructor,"controls":step.controls,
        "fields":step.fields.iter().map(|f|json!({"name":f.name,"units":f.units,"association":f.association,
            "stage":f.stage,"evaluation_time_bits":f.evaluation_time.map(f64::to_bits),"dimensions":f.dimensions,
            "components":f.components,"scalar_type":f.scalar_type,"data":f.data})).collect::<Vec<_>>()})
}
fn assert_exact(reader: &RecordingReader, expected: &[RecordedStep]) {
    assert_eq!(reader.count().unwrap(), expected.len());
    for (i, publication) in expected.iter().enumerate() {
        let actual = reader.select(i).unwrap();
        assert_eq!(exact(&actual), exact(publication));
        RecordedConfiguredSession::decode(reader.run(), &actual).unwrap();
    }
}
fn revision(members: Vec<ConfiguredRunSetMember>) -> ConfiguredRunSetRevision {
    let baseline = members[0].spec.clone();
    let baseline_member = members[0].member_id.clone();
    let members = members
        .into_iter()
        .map(|mut m| {
            m.differences_from_baseline =
                ConfiguredSpecDifference::between(&baseline, &m.spec).unwrap();
            m
        })
        .collect();
    ConfiguredRunSetRevision {
        schema_version: CONFIGURED_VERSION,
        kind: ConfiguredSetKind::ConfiguredAcceptedSessionSet,
        set_id: "real-configured-archive-contracts".into(),
        parent_sha256: None,
        baseline_member,
        members,
    }
}

#[test]
fn real_native_and_authored_candidate_fallback_archives_reopen_exact_publications() {
    let root = Directory::new();
    for (name, configured, behavior) in [
        ("native", false, Behavior::Solve),
        ("configured", true, Behavior::Solve),
        ("fallback", true, Behavior::BadOnce),
        ("declined", true, Behavior::Decline),
    ] {
        let mut m = member(name, configured);
        let mut execution = Execution::frozen(&m);
        execution.behavior = behavior;
        let expected = produce(&root.0, &m, execution);
        pin(&root.0, &mut m);
        let reader = ready(&root.0, &m);
        assert_exact(&reader, &expected);
        let coverage = verify_configured_recording(&reader).unwrap();
        if configured {
            assert_eq!(coverage.verified_prior_links, 2);
        }
        let second =
            RecordedConfiguredSession::decode(reader.run(), &reader.select(2).unwrap()).unwrap();
        let receipt = second.receipt.unwrap();
        assert!(receipt.viscosity_applied);
        assert_eq!(receipt.dynamic_viscosity.to_bits(), MU.to_bits());
        match behavior {
            Behavior::BadOnce => {
                assert_eq!(
                    receipt.rejection,
                    Some(RecordedProviderRejection::QualityGate)
                );
                assert!(receipt.candidate.is_some());
                assert!(matches!(
                    receipt.selected,
                    RecordedPressureSelection::Native(_)
                ));
            }
            Behavior::Decline => {
                assert!(!receipt.prediction_called);
                assert!(matches!(
                    receipt.rejection,
                    Some(RecordedProviderRejection::Unsupported(_))
                ));
            }
            Behavior::Solve if configured => {
                assert_eq!(receipt.selected, RecordedPressureSelection::Configured)
            }
            Behavior::Solve => {
                assert!(receipt.binding.is_none());
                assert!(receipt.context.is_none());
            }
        }
        let last =
            RecordedConfiguredSession::decode(reader.run(), &reader.select(3).unwrap()).unwrap();
        if configured {
            let context = last.receipt.unwrap().context.unwrap();
            assert_eq!(context.prior_generation, Some(2));
            assert_eq!(
                context.native_prefix_steps,
                if matches!(behavior, Behavior::Solve) {
                    1
                } else {
                    2
                }
            );
        }
        assert_eq!(reader.status().unwrap().outcome, "completed");
        assert_eq!(
            m.attempt.unwrap().finalized.unwrap().accepted_generation,
            Some(3)
        );
    }
}

#[test]
fn disabled_and_applied_zero_viscosity_are_distinct_real_receipts() {
    let root = Directory::new();
    for applied in [false, true] {
        let mut m = member(if applied { "zero-applied" } else { "disabled" }, false);
        m.spec.future.viscosity = if applied {
            ViscosityIntent::NewtonianConstant {
                dynamic_viscosity_pa_s: 0.0,
            }
        } else {
            ViscosityIntent::Disabled
        };
        m.spec.future.viscosity_workspace_limit_bytes = applied.then_some(CAP);
        reseal(&mut m);
        produce(&root.0, &m, Execution::frozen(&m));
        pin(&root.0, &mut m);
        let reader = ready(&root.0, &m);
        let decoded =
            RecordedConfiguredSession::decode(reader.run(), &reader.select(1).unwrap()).unwrap();
        let receipt = decoded.receipt.unwrap();
        assert_eq!(receipt.viscosity_applied, applied);
        assert_eq!(receipt.dynamic_viscosity.to_bits(), 0.0f64.to_bits());
    }
}

#[test]
fn actual_stage_mu_provider_prefix_dt_and_capture_mismatches_refuse_pin() {
    let root = Directory::new();
    for case in 0..9 {
        let mut m = member(&format!("mismatch-{case}"), matches!(case, 3..=6));
        let mut e = Execution::frozen(&m);
        match case {
            0 => e.mu = Some(2.0 * MU),
            1 => {
                m.spec.future.viscosity = ViscosityIntent::Disabled;
                m.spec.future.viscosity_workspace_limit_bytes = None;
                e.mu = Some(0.0);
            }
            2 => {
                m.spec.future.viscosity = ViscosityIntent::NewtonianConstant {
                    dynamic_viscosity_pa_s: 0.0,
                };
                e.mu = None;
            }
            3 => e.provider_after = Some(0),
            4 => e.provider_after = Some(2),
            5 => e.identity.manifest_sha256[0] ^= 1,
            6 => e.identity.runtime.push_str("-different"),
            7 => {
                m.spec.future.requested_dt_s = DT / 2.0;
                e.dt = DT;
            }
            _ => e.record_constructor = false,
        }
        reseal(&mut m);
        produce(&root.0, &m, e);
        assert!(
            m.attempt
                .as_mut()
                .unwrap()
                .pin_closed(&root.0, &m.spec)
                .is_err(),
            "case {case}"
        );
        assert!(m.attempt.as_ref().unwrap().finalized.is_none());
    }
}

#[test]
fn header_budgets_identity_constructor_and_frozen_intent_are_checked() {
    let root = Directory::new();
    for case in 0..3 {
        let mut m = member(&format!("budget-{case}"), false);
        let mut e = Execution::frozen(&m);
        match case {
            0 => e.limits.max_records += 1,
            1 => e.limits.max_record_bytes /= 2,
            _ => e.limits.max_database_bytes += 4096,
        }
        produce(&root.0, &m, e);
        assert!(
            m.attempt
                .as_mut()
                .unwrap()
                .pin_closed(&root.0, &m.spec)
                .is_err()
        );
    }
    for case in 0..5 {
        let mut m = member(&format!("header-{case}"), false);
        let original = constructor();
        let declared = bound_configured_run(&m.spec, &original.snapshot()).unwrap();
        let mut cfg = *original.snapshot().configuration();
        let mut identity = original.snapshot().identity().clone();
        match case {
            0 => identity.run_id += 1,
            1 => identity.source_revision.push_str("-other"),
            2 => cfg.boundary_workspace_limit *= 2,
            3 => cfg.transport.carrier.pressure.max_iterations += 1,
            _ => {}
        }
        let actual = session_from(cfg, identity);
        let mut run = configured_run(&actual.snapshot(), &declared.provenance).unwrap();
        if case == 4 {
            let mut binding: Value = serde_json::from_str(&run.provenance).unwrap();
            binding["spec_sha256"] = json!(vec![99; 32]);
            run.provenance = serde_json::to_string(&binding).unwrap();
        }
        let mut writer = StepRecorder::create(
            &root
                .0
                .join(&m.attempt.as_ref().unwrap().archive_relative_path),
            run,
            m.spec.future.recording.limits(),
        )
        .unwrap();
        writer
            .append(&configured_publication(&actual.snapshot(), 0, CAP).unwrap())
            .unwrap();
        writer
            .finish("cancelled", "actual constructor only", Some(0))
            .unwrap();
        drop(writer);
        assert!(
            m.attempt
                .as_mut()
                .unwrap()
                .pin_closed(&root.0, &m.spec)
                .is_err(),
            "header case {case}"
        );
    }
    let mut m = member("changed-intent", false);
    produce(&root.0, &m, Execution::frozen(&m));
    m.spec.future.forces[0].value[0] *= 2.0;
    reseal(&mut m);
    assert!(
        m.attempt
            .as_mut()
            .unwrap()
            .pin_closed(&root.0, &m.spec)
            .is_err()
    );
    let mut m = member("bad-constructor", false);
    m.spec.constructor.carrier_id += 1;
    reseal(&mut m);
    assert!(bound_configured_run(&m.spec, &constructor().snapshot()).is_err());
}

#[test]
fn member_states_do_not_promote_missing_active_unpinned_or_changed_material() {
    let root = Directory::new();
    let mut m = member("states", false);
    let mut not_run = m.clone();
    not_run.attempt = None;
    assert!(matches!(
        open_member(&root.0, &not_run),
        MemberState::NotRun
    ));
    assert!(matches!(open_member(&root.0, &m), MemberState::Missing));
    let mut e = Execution::frozen(&m);
    e.intervals = 1;
    e.terminal = None;
    produce(&root.0, &m, e);
    assert!(matches!(
        open_member(&root.0, &m),
        MemberState::Incomplete {
            recorded_count: 2,
            ..
        }
    ));
    assert!(
        m.attempt
            .as_mut()
            .unwrap()
            .pin_closed(&root.0, &m.spec)
            .is_err()
    );
    let mut closed = member("closed", false);
    produce(&root.0, &closed, Execution::frozen(&closed));
    assert!(matches!(
        open_member(&root.0, &closed),
        MemberState::Unpinned {
            recorded_count: 4,
            ..
        }
    ));
    pin(&root.0, &mut closed);
    assert!(matches!(
        open_member(&root.0, &closed),
        MemberState::Ready(_)
    ));
    let mut changed = closed.clone();
    changed
        .attempt
        .as_mut()
        .unwrap()
        .finalized
        .as_mut()
        .unwrap()
        .sha256[0] ^= 1;
    assert!(matches!(
        open_member(&root.0, &changed),
        MemberState::Invalid(_)
    ));
    let mut escaped = closed.clone();
    escaped.attempt.as_mut().unwrap().archive_relative_path = "../closed.sqlite".into();
    assert!(matches!(
        open_member(&root.0, &escaped),
        MemberState::Invalid(_)
    ));
    let mut unsupported = closed.clone();
    unsupported.attempt.as_mut().unwrap().finalized = None;
    let path = root
        .0
        .join(&unsupported.attempt.as_ref().unwrap().archive_relative_path);
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection.pragma_update(None, "user_version", 999).unwrap();
    drop(connection);
    assert!(matches!(
        open_member(&root.0, &unsupported),
        MemberState::Unsupported("archive version")
    ));
}

#[test]
fn capacity_failure_retains_real_accepted_unrecorded_tail_and_unknown_tail_is_not_zero() {
    let root = Directory::new();
    let mut m = member("capacity", false);
    m.spec.future.recording.max_records = 2;
    reseal(&mut m);
    let mut s = m
        .spec
        .create_session(SessionIdentity {
            run_id: RUN_ID,
            source_revision: executor().source_revision,
        })
        .unwrap();
    let mut writer = StepRecorder::create(
        &root.0.join("capacity.sqlite"),
        bound_configured_run(&m.spec, &s.snapshot()).unwrap(),
        m.spec.future.recording.limits(),
    )
    .unwrap();
    writer
        .append(&configured_publication(&s.snapshot(), 0, CAP).unwrap())
        .unwrap();
    let mut workspace =
        ProviderStepWorkspace::new(&grid(), RHO, CAP, PressureImplementation::JacobiPcgV1).unwrap();
    let mut viscosity = ViscosityWorkspace::new(grid(), CAP).unwrap();
    let forces: Vec<_> = m
        .spec
        .future
        .forces
        .iter()
        .map(ForceSpec::body_force)
        .collect();
    for generation in 1..=2 {
        s.step_with_configured_pressure(
            inputs(&m, &forces, DT),
            &mut workspace,
            Some((&mut viscosity, MU)),
            None,
            None,
            |_| false,
        )
        .unwrap();
        let publication = configured_publication(&s.snapshot(), generation, CAP).unwrap();
        if generation == 1 {
            writer.append(&publication).unwrap();
        } else {
            assert!(matches!(
                writer.append(&publication),
                Err(RecordingError::Limit("record count"))
            ));
            assert_eq!(s.snapshot().state().carrier.generation, 2);
            assert!(matches!(
                writer.append(&publication),
                Err(RecordingError::Closed)
            ));
        }
    }
    assert!(
        writer
            .finish("completed", "cannot conceal unrecorded tail", Some(2))
            .is_err()
    );
    writer
        .finish(
            "failed",
            "accepted generation2 not recorded after cap",
            Some(2),
        )
        .unwrap();
    drop(writer);
    drop(s);
    pin(&root.0, &mut m);
    let reader = ready(&root.0, &m);
    assert_eq!(reader.count().unwrap(), 2);
    assert_eq!(reader.status().unwrap().accepted_generation, Some(2));
    assert_eq!(reader.status().unwrap().outcome, "failed");
    assert_eq!(reader.select(1).unwrap().generation, 1);
    assert!(matches!(reader.select(2), Err(RecordingError::Missing(2))));
    let mut unknown = member("unknown-tail", false);
    unknown.spec.future.capture_constructor = false;
    reseal(&mut unknown);
    let s = constructor();
    let mut writer = StepRecorder::create(
        &root.0.join("unknown-tail.sqlite"),
        bound_configured_run(&unknown.spec, &s.snapshot()).unwrap(),
        unknown.spec.future.recording.limits(),
    )
    .unwrap();
    writer
        .finish("cancelled", "owner did not report accepted tail", None)
        .unwrap();
    drop(writer);
    assert!(
        unknown
            .attempt
            .as_mut()
            .unwrap()
            .pin_closed(&root.0, &unknown.spec)
            .is_err()
    );
    assert!(unknown.attempt.as_ref().unwrap().finalized.is_none());
    assert!(matches!(
        open_member(&root.0, &unknown),
        MemberState::Unpinned {
            recorded_count: 0,
            ..
        }
    ));
    let reader = RecordingReader::open(
        &root.0.join("unknown-tail.sqlite"),
        unknown.spec.future.recording.limits(),
    )
    .unwrap();
    assert_eq!(reader.status().unwrap().accepted_generation, None);
    let mut cancelled = member("cancelled-prefix", false);
    let mut e = Execution::frozen(&cancelled);
    e.intervals = 2;
    e.terminal = Some("cancelled");
    let expected = produce(&root.0, &cancelled, e);
    pin(&root.0, &mut cancelled);
    let reader = ready(&root.0, &cancelled);
    assert_exact(&reader, &expected);
    assert_eq!(reader.status().unwrap().outcome, "cancelled");
    assert_eq!(reader.status().unwrap().accepted_generation, Some(2));
    let mut short = member("completed-short", false);
    let mut e = Execution::frozen(&short);
    e.intervals = 2;
    produce(&root.0, &short, e);
    assert!(
        short
            .attempt
            .as_mut()
            .unwrap()
            .pin_closed(&root.0, &short.spec)
            .is_err()
    );
}

#[test]
fn actual_short_dt_and_nonbinary_owner_clock_are_retained_without_requested_dt_attestation() {
    let root = Directory::new();
    let mut shortened = member("shortened", false);
    shortened.spec.constructor.tolerances.max_courant = 1.0 / 128.0;
    shortened.spec.future.requested_dt_s = 2.0;
    reseal(&mut shortened);
    let expected = produce(&root.0, &shortened, Execution::frozen(&shortened));
    assert!(
        expected[1].dt.unwrap() < 2.0,
        "fixture must actually trigger host dt shortening"
    );
    pin(&root.0, &mut shortened);
    assert_exact(&ready(&root.0, &shortened), &expected);
    let mut clock = member("nonbinary-clock", false);
    clock.spec.future.requested_dt_s = 1.0 / 60.0;
    clock.spec.future.requested_steps = 64;
    clock.spec.future.recording.max_records = 65;
    clock.spec.future.forces.clear();
    reseal(&mut clock);
    let expected = produce(&root.0, &clock, Execution::frozen(&clock));
    pin(&root.0, &mut clock);
    let reader = ready(&root.0, &clock);
    assert_exact(&reader, &expected);
    let mut previous_time = 0.0_f64;
    let mut subtraction_differs = false;
    for ordinal in 1..=64 {
        let selected = reader.select(ordinal).unwrap();
        let cold = RecordedConfiguredSession::decode(reader.run(), &selected).unwrap();
        let actual_start = cold.interval_pressure.unwrap().0;
        assert_eq!(actual_start.to_bits(), previous_time.to_bits());
        subtraction_differs |= (cold.time - cold.dt.unwrap()).to_bits() != actual_start.to_bits();
        assert!(cold.dt.unwrap() <= clock.spec.future.requested_dt_s);
        previous_time = cold.time;
    }
    assert!(
        subtraction_differs,
        "nonbinary timeline must expose end-dt reconstruction error"
    );
}

#[test]
fn producer_exits_before_fresh_process_inspects_frozen_members_without_runtime() {
    match std::env::var(PHASE).ok().as_deref() {
        Some("produce") => {
            let root = PathBuf::from(std::env::var_os(ROOT).unwrap());
            let mut members = vec![member("native", false), member("configured-fallback", true)];
            for m in &mut members {
                let mut execution = Execution::frozen(m);
                execution.behavior = Behavior::BadOnce;
                let expected = produce(&root, m, execution);
                std::fs::write(
                    root.join(format!("{}-expected.json", m.member_id)),
                    serde_json::to_vec(&expected.iter().map(exact).collect::<Vec<_>>()).unwrap(),
                )
                .unwrap();
                m.spec
                    .save_new(&root.join(format!("{}-spec.json", m.member_id)))
                    .unwrap();
                pin(&root, m);
            }
            revision(members)
                .save_new(&root.join("frozen-set.json"))
                .unwrap();
            println!("producer closed two real Session archives");
        }
        Some("inspect") => {
            // This branch performs inspection only: no Session/provider/runtime
            // constructor, numerical solve, package resolution or model execution.
            let root = PathBuf::from(std::env::var_os(ROOT).unwrap());
            let revision = ConfiguredRunSetRevision::open(&root.join("frozen-set.json")).unwrap();
            for m in &revision.members {
                let frozen =
                    FrozenConfiguredSpec::open(&root.join(format!("{}-spec.json", m.member_id)))
                        .unwrap();
                assert_eq!(frozen, m.spec);
                if let PressureIntent::NativePrefixThenConfigured {
                    deployment: DeploymentLocator::Absolute(path),
                    ..
                } = &m.spec.future.pressure
                {
                    assert!(!Path::new(path).exists());
                }
                let expected: Vec<Value> = serde_json::from_slice(
                    &std::fs::read(root.join(format!("{}-expected.json", m.member_id))).unwrap(),
                )
                .unwrap();
                let reader = ready(&root, m);
                assert_eq!(reader.count().unwrap(), expected.len());
                assert_eq!(
                    reader.run().identity.source_revision,
                    executor().source_revision
                );
                let cfg = configured_configuration(reader.run()).unwrap();
                assert_eq!(cfg.max_steps, m.spec.constructor.max_steps);
                for (ordinal, expected) in expected.into_iter().enumerate() {
                    let selected = reader.select(ordinal).unwrap();
                    assert_eq!(exact(&selected), expected);
                    RecordedConfiguredSession::decode(reader.run(), &selected).unwrap();
                }
                assert_eq!(reader.status().unwrap().outcome, "completed");
                assert_eq!(
                    m.attempt
                        .as_ref()
                        .unwrap()
                        .finalized
                        .as_ref()
                        .unwrap()
                        .outcome,
                    TerminalOutcome::Completed
                );
            }
            println!("fresh reader inspected pinned material without runtime");
        }
        None => {
            let root = Directory::new();
            for (phase, marker) in [
                ("produce", "producer closed two real Session archives"),
                (
                    "inspect",
                    "fresh reader inspected pinned material without runtime",
                ),
            ] {
                let child = Command::new(std::env::current_exe().unwrap())
                    .args(["--exact", PROCESS_TEST, "--nocapture"])
                    .env(PHASE, phase)
                    .env(ROOT, &root.0)
                    .output()
                    .unwrap();
                assert!(
                    child.status.success(),
                    "phase {phase}: {}\n{}",
                    String::from_utf8_lossy(&child.stdout),
                    String::from_utf8_lossy(&child.stderr)
                );
                assert!(String::from_utf8_lossy(&child.stdout).contains(marker));
            }
        }
        Some(other) => panic!("unknown test phase {other}"),
    }
}

// These observations are explicitly authored application/runtime metadata. The
// provider still solves through the existing native Candidate; there is no IPC.
use rheon::recording::{
    diagnostic::NoCandidateReason,
    provider::runtime::{
        RuntimeObservation, RuntimeOutcome, RuntimeRequestStamp, RuntimeTerminalCoverage,
        RuntimeTermination, RuntimeTerminationCause,
    },
};
use sha2::{Digest, Sha256};
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
enum RuntimeMode {
    Candidate,
    HostFallback,
    Cancel,
    Deadline,
}
struct ObservedCandidate {
    native: Candidate,
    mode: RuntimeMode,
    observation: Option<RuntimeObservation>,
}
impl ContextPressureProvider for ObservedCandidate {
    fn identity(&self) -> Arc<ContextProviderIdentity> {
        self.native.identity()
    }
    fn supports(&self, request: &ContextPressureRequest<'_>) -> Result<(), ProviderRejection> {
        self.native.supports(request)
    }
    fn predict(
        &mut self,
        request: &ContextPressureRequest<'_>,
        output: &mut [f64],
        cancel: &mut dyn FnMut() -> bool,
    ) -> Result<(), ProviderRejection> {
        let request_stamp = RuntimeRequestStamp::capture(request, &self.native.identity).unwrap();
        if matches!(self.mode, RuntimeMode::Deadline) {
            while !cancel() {
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        let aborted = matches!(self.mode, RuntimeMode::Cancel | RuntimeMode::Deadline);
        self.observation = Some(RuntimeObservation {
            request: request_stamp,
            outcome: if aborted {
                RuntimeOutcome::NoCandidate {
                    reason: NoCandidateReason::Cancelled,
                    detail: "authored application observation, no runtime dispatch".into(),
                    dispatched: false,
                    invalid_payload_sha256: None,
                }
            } else {
                RuntimeOutcome::CandidateReturned
            },
        });
        if aborted {
            Err(ProviderRejection::Cancelled)
        } else {
            self.native.predict(request, output, cancel)
        }
    }
}
struct RuntimeFixture {
    session: SimulationSession,
    pressure: ProviderStepWorkspace,
    viscosity: ViscosityWorkspace,
    provider: ObservedCandidate,
    writer: StepRecorder,
    actual_mu: f64,
    expected: Vec<Value>,
}
impl RuntimeFixture {
    fn new(root: &Path, m: &ConfiguredRunSetMember, mode: RuntimeMode) -> Self {
        let session = m
            .spec
            .create_session(SessionIdentity {
                run_id: RUN_ID,
                source_revision: executor().source_revision,
            })
            .unwrap();
        assert_eq!(session.snapshot().configuration().max_steps, 128);
        let writer = StepRecorder::create(
            &root.join(&m.attempt.as_ref().unwrap().archive_relative_path),
            bound_configured_run(&m.spec, &session.snapshot()).unwrap(),
            m.spec.future.recording.limits(),
        )
        .unwrap();
        let mut fixture = Self {
            session,
            pressure: ProviderStepWorkspace::new(
                &grid(),
                RHO,
                CAP,
                PressureImplementation::JacobiPcgV1,
            )
            .unwrap(),
            viscosity: ViscosityWorkspace::new(grid(), CAP).unwrap(),
            provider: ObservedCandidate {
                native: Candidate::new(
                    configuration().carrier.pressure,
                    if matches!(mode, RuntimeMode::HostFallback) {
                        Behavior::BadOnce
                    } else {
                        Behavior::Solve
                    },
                    fixture_identity(),
                ),
                mode,
                observation: None,
            },
            writer,
            actual_mu: MU,
            expected: vec![],
        };
        if m.spec.future.capture_constructor {
            fixture.append(false).unwrap();
        }
        fixture
    }
    fn append(&mut self, runtime: bool) -> Result<(), RecordingError> {
        let row = configured_publication_with_runtime(
            &self.session.snapshot(),
            self.expected.len(),
            CAP,
            runtime.then(|| self.provider.observation.as_ref().unwrap()),
        )
        .unwrap();
        let result = self.writer.append(&row);
        if result.is_ok() {
            self.expected.push(exact(&row));
        }
        result
    }
    fn seed(&mut self, m: &ConfiguredRunSetMember) {
        let forces: Vec<_> = m
            .spec
            .future
            .forces
            .iter()
            .map(ForceSpec::body_force)
            .collect();
        self.session
            .step_with_configured_pressure(
                inputs(m, &forces, DT),
                &mut self.pressure,
                Some((&mut self.viscosity, self.actual_mu)),
                None,
                None,
                |_| false,
            )
            .unwrap();
        self.append(false).unwrap();
    }
    fn attempt(&mut self, m: &ConfiguredRunSetMember) {
        let before = exact(&configured_publication(&self.session.snapshot(), 0, CAP).unwrap());
        let forces: Vec<_> = m
            .spec
            .future
            .forces
            .iter()
            .map(ForceSpec::body_force)
            .collect();
        let deadline = matches!(self.provider.mode, RuntimeMode::Deadline);
        let result = self.session.step_with_configured_pressure(
            inputs(m, &forces, DT),
            &mut self.pressure,
            Some((&mut self.viscosity, self.actual_mu)),
            Some(&mut self.provider),
            deadline.then(|| Instant::now() + Duration::from_secs(1)),
            |_| false,
        );
        match self.provider.mode {
            RuntimeMode::Candidate | RuntimeMode::HostFallback => {
                result.unwrap();
                assert_eq!(self.session.snapshot().state().carrier.generation, 2);
            }
            RuntimeMode::Cancel | RuntimeMode::Deadline => {
                if deadline {
                    assert!(matches!(result, Err(SessionError::Deadline)));
                } else {
                    assert!(matches!(
                        result,
                        Err(SessionError::Step(LiquidStepError::Carrier(
                            SimulationError::Pressure(PressureError::Cancelled { .. })
                        )))
                    ));
                }
                assert_eq!(
                    exact(&configured_publication(&self.session.snapshot(), 0, CAP).unwrap()),
                    before
                );
                assert_eq!(self.session.snapshot().state().carrier.generation, 1);
            }
        }
        assert_eq!(
            self.provider
                .observation
                .as_ref()
                .unwrap()
                .request
                .context
                .generation,
            2
        );
    }
    fn termination(&self, cause: RuntimeTerminationCause) -> RuntimeTermination {
        RuntimeTermination::capture(
            &self.session.snapshot(),
            cause,
            "actual Session; explicitly authored runtime metadata",
            self.provider.observation.as_ref(),
        )
        .unwrap()
    }
}
fn raw_reader(root: &Path, m: &ConfiguredRunSetMember) -> RecordingReader {
    RecordingReader::open(
        &root.join(&m.attempt.as_ref().unwrap().archive_relative_path),
        m.spec.future.recording.limits(),
    )
    .unwrap()
}
fn typed_custody(
    root: &Path,
    m: &ConfiguredRunSetMember,
) -> rheon::recording::provider::runtime::RuntimeTerminationAudit {
    match runtime_custody(&raw_reader(root, m), &m.spec).unwrap() {
        ConfiguredRuntimeCustody::Typed(audit) => *audit,
        ConfiguredRuntimeCustody::LegacyUnavailable => panic!("typed custody was discarded"),
    }
}
fn refused_before_pin(root: &Path, m: &mut ConfiguredRunSetMember, unsupported: bool) {
    assert!(m.attempt.as_ref().unwrap().finalized.is_none());
    assert!(
        m.attempt
            .as_mut()
            .unwrap()
            .pin_closed(root, &m.spec)
            .is_err()
    );
    assert!(m.attempt.as_ref().unwrap().finalized.is_none());
    if unsupported {
        assert!(matches!(open_member(root, m), MemberState::Unsupported(_)));
    } else {
        assert!(matches!(open_member(root, m), MemberState::Invalid(_)));
    }
}

#[test]
fn generic_rcp2_active_and_legacy_rct_prose_keep_runtime_custody_unavailable() {
    let root = Directory::new();
    for (index, detail) in [
        None,
        Some("ordinary legacy terminal"),
        Some("RCT"),
        Some("RCT diagnostics unavailable"),
        Some("RCTx unknown prose"),
        Some("legacy RCT1: prose"),
    ]
    .into_iter()
    .enumerate()
    {
        let m = member(&format!("runtime-legacy-{index}"), false);
        let mut fixture = RuntimeFixture::new(&root.0, &m, RuntimeMode::Candidate);
        fixture.seed(&m);
        if let Some(detail) = detail {
            fixture.writer.finish("failed", detail, Some(1)).unwrap();
        }
        drop(fixture);
        let reader = raw_reader(&root.0, &m);
        assert!(
            RecordedConfiguredSession::decode(reader.run(), &reader.select(1).unwrap())
                .unwrap()
                .runtime
                .is_none()
        );
        assert!(matches!(
            runtime_custody(&reader, &m.spec).unwrap(),
            ConfiguredRuntimeCustody::LegacyUnavailable
        ));
        assert!(matches!(
            open_member(&root.0, &m),
            MemberState::Incomplete { .. } | MemberState::Unpinned { .. }
        ));
    }
    // Per-row observations do not manufacture typed terminal custody.
    let mut m = member("runtime-legacy-with-rcp3", true);
    let mut fixture = RuntimeFixture::new(&root.0, &m, RuntimeMode::Candidate);
    fixture.seed(&m);
    fixture.attempt(&m);
    fixture.append(true).unwrap();
    fixture
        .writer
        .finish(
            "failed",
            "generic legacy terminal after actual accepted candidate",
            Some(2),
        )
        .unwrap();
    drop(fixture);
    let reader = raw_reader(&root.0, &m);
    assert!(
        RecordedConfiguredSession::decode(reader.run(), &reader.select(2).unwrap())
            .unwrap()
            .runtime
            .is_some()
    );
    assert!(matches!(
        runtime_custody(&reader, &m.spec).unwrap(),
        ConfiguredRuntimeCustody::LegacyUnavailable
    ));
    assert!(matches!(
        open_member(&root.0, &m),
        MemberState::Unpinned {
            recorded_count: 3,
            ..
        }
    ));
    drop(reader);
    pin(&root.0, &mut m);
    let reader = ready(&root.0, &m);
    let decoded =
        RecordedConfiguredSession::decode(reader.run(), &reader.select(2).unwrap()).unwrap();
    assert_eq!(
        decoded.runtime.unwrap().outcome,
        RuntimeOutcome::CandidateReturned
    );
    assert_eq!(
        decoded.receipt.unwrap().selected,
        RecordedPressureSelection::Configured
    );
    assert!(matches!(
        runtime_custody(&reader, &m.spec).unwrap(),
        ConfiguredRuntimeCustody::LegacyUnavailable
    ));
}

#[test]
fn reserved_runtime_terminal_tags_refuse_before_unpinned_and_pin() {
    let root = Directory::new();
    for (index, detail) in [
        "RCT1", "RCT1x", "RCT1:", "RCT1:zz", "RCT9:00", "RCT10:00", "RCT01:00",
    ]
    .into_iter()
    .enumerate()
    {
        let mut m = member(&format!("runtime-reserved-{index}"), false);
        let mut fixture = RuntimeFixture::new(&root.0, &m, RuntimeMode::Candidate);
        fixture.seed(&m);
        fixture.writer.finish("failed", detail, Some(1)).unwrap();
        drop(fixture);
        refused_before_pin(&root.0, &mut m, index >= 4);
    }
}

#[test]
fn typed_runtime_candidate_and_real_quality_fallback_preserve_last_publication_custody() {
    let root = Directory::new();
    for fallback in [false, true] {
        let mut m = member(
            if fallback {
                "runtime-fallback"
            } else {
                "runtime-candidate"
            },
            true,
        );
        m.spec.future.requested_steps = 2;
        reseal(&mut m);
        let mut fixture = RuntimeFixture::new(
            &root.0,
            &m,
            if fallback {
                RuntimeMode::HostFallback
            } else {
                RuntimeMode::Candidate
            },
        );
        fixture.seed(&m);
        fixture.attempt(&m);
        fixture.append(true).unwrap();
        fixture
            .termination(RuntimeTerminationCause::Completed)
            .finish(&mut fixture.writer)
            .unwrap();
        let expected = fixture.expected.clone();
        drop(fixture);
        pin(&root.0, &mut m);
        let reader = ready(&root.0, &m);
        for (ordinal, value) in expected.iter().enumerate() {
            assert_eq!(&exact(&reader.select(ordinal).unwrap()), value);
        }
        let observed =
            RecordedConfiguredSession::decode(reader.run(), &reader.select(2).unwrap()).unwrap();
        assert_eq!(
            observed.runtime.as_ref().unwrap().outcome,
            RuntimeOutcome::CandidateReturned
        );
        let receipt = observed.receipt.unwrap();
        if fallback {
            assert_eq!(
                receipt.rejection,
                Some(RecordedProviderRejection::QualityGate)
            );
            assert!(matches!(
                receipt.selected,
                RecordedPressureSelection::Native(_)
            ));
        } else {
            assert_eq!(receipt.selected, RecordedPressureSelection::Configured);
        }
        let audit = typed_custody(&root.0, &m);
        assert_eq!(audit.termination.cause, RuntimeTerminationCause::Completed);
        assert_eq!(
            audit.coverage,
            RuntimeTerminalCoverage::VerifiedLastPublication
        );
    }
}

#[test]
fn cancelled_and_deadline_next_attempts_retain_real_seed_and_typed_custody() {
    let root = Directory::new();
    for deadline in [false, true] {
        let mut m = member(
            if deadline {
                "runtime-deadline"
            } else {
                "runtime-cancel"
            },
            true,
        );
        let mut fixture = RuntimeFixture::new(
            &root.0,
            &m,
            if deadline {
                RuntimeMode::Deadline
            } else {
                RuntimeMode::Cancel
            },
        );
        fixture.seed(&m);
        fixture.attempt(&m);
        let cause = if deadline {
            RuntimeTerminationCause::SessionDeadline
        } else {
            RuntimeTerminationCause::Cancelled
        };
        fixture
            .termination(cause)
            .finish(&mut fixture.writer)
            .unwrap();
        drop(fixture);
        pin(&root.0, &mut m);
        let reader = ready(&root.0, &m);
        assert_eq!(reader.count().unwrap(), 2);
        assert_eq!(reader.select(1).unwrap().generation, 1);
        assert!(
            RecordedConfiguredSession::decode(reader.run(), &reader.select(1).unwrap())
                .unwrap()
                .runtime
                .is_none()
        );
        let audit = typed_custody(&root.0, &m);
        assert_eq!(audit.termination.cause, cause);
        assert_eq!(audit.termination.accepted_generation, 1);
        assert_eq!(
            audit
                .termination
                .observation
                .unwrap()
                .request
                .context
                .generation,
            2
        );
        assert_eq!(
            audit.coverage,
            RuntimeTerminalCoverage::VerifiedLastPublication
        );
    }
}

#[test]
fn terminal_unaccepted_request_obeys_frozen_future_not_larger_session_quota() {
    let root = Directory::new();
    for case in 0..9 {
        let mut m = member(&format!("runtime-intent-{case}"), true);
        match case {
            0 | 8 => {
                m.spec.future.requested_steps = 1;
                m.spec.future.pressure = PressureIntent::NativeOnly;
            }
            1 => {
                if let PressureIntent::NativePrefixThenConfigured {
                    requested_native_prefix_steps,
                    ..
                } = &mut m.spec.future.pressure
                {
                    *requested_native_prefix_steps = 2;
                }
            }
            2 => m.spec.future.pressure = PressureIntent::NativeOnly,
            3 => {
                if let PressureIntent::NativePrefixThenConfigured { provider, .. } =
                    &mut m.spec.future.pressure
                {
                    let mut identity = fixture_identity();
                    identity.manifest_sha256[0] ^= 1;
                    *provider = Box::new(ConfiguredProviderPin::capture(&identity).unwrap());
                }
            }
            4 => {
                m.spec.future.viscosity = ViscosityIntent::NewtonianConstant {
                    dynamic_viscosity_pa_s: MU / 2.0,
                }
            }
            5 => m.spec.future.requested_dt_s = DT / 2.0,
            _ => {}
        }
        reseal(&mut m);
        let mut fixture = RuntimeFixture::new(
            &root.0,
            &m,
            if case == 8 {
                RuntimeMode::Candidate
            } else {
                RuntimeMode::Cancel
            },
        );
        fixture.seed(&m);
        fixture.attempt(&m);
        let mut terminal = fixture.termination(if case == 8 {
            RuntimeTerminationCause::StorageFailure
        } else {
            RuntimeTerminationCause::Cancelled
        });
        if case == 6 {
            terminal
                .observation
                .as_mut()
                .unwrap()
                .request
                .context
                .prior_pressure_sha256
                .as_mut()
                .unwrap()[0] ^= 1;
        }
        if case == 7 {
            terminal
                .observation
                .as_mut()
                .unwrap()
                .request
                .context
                .native_prefix_steps = 0;
        }
        terminal.finish(&mut fixture.writer).unwrap();
        drop(fixture);
        if case < 6 || case == 8 {
            rheon::recording::provider::runtime::audit_runtime_termination(&raw_reader(
                &root.0, &m,
            ))
            .unwrap();
        }
        let error = runtime_custody(&raw_reader(&root.0, &m), &m.spec).unwrap_err();
        match case {
            0 => assert!(matches!(
                error,
                run_sets::RunSetError::Integrity("configured runtime request horizon")
            )),
            1 | 2 => assert!(matches!(
                error,
                run_sets::RunSetError::Integrity("configured runtime request in native phase")
            )),
            3 => assert!(matches!(
                error,
                run_sets::RunSetError::Integrity("resolved provider differs from frozen pin")
            )),
            8 => assert!(matches!(
                error,
                run_sets::RunSetError::Integrity("configured runtime terminal horizon")
            )),
            4 => assert!(matches!(
                error,
                run_sets::RunSetError::Integrity("configured runtime viscosity differs")
            )),
            5 => assert!(matches!(
                error,
                run_sets::RunSetError::Integrity("configured runtime dt exceeds request")
            )),
            6 | 7 => assert!(matches!(
                error,
                run_sets::RunSetError::Recording(RecordingError::Invalid(
                    "runtime attempted prior association"
                ))
            )),
            _ => {}
        }
        refused_before_pin(&root.0, &mut m, false);
    }
}

// A single producer helper also supplies the separate-OS cold matrix below.
// Expected fields come from live captures, not decoding an already stored row.
fn produce_runtime_case(root: &Path, case: &str) -> (ConfiguredRunSetMember, Vec<Value>) {
    let mut m = member(case, true);
    if matches!(case, "candidate" | "fallback") {
        m.spec.future.requested_steps = 2;
    }
    if case == "tail" {
        m.spec.future.recording.max_records = 2;
    }
    if case == "empty" {
        m.spec.future.capture_constructor = false;
    }
    if case == "horizon" {
        m.spec.future.requested_steps = 1;
        m.spec.future.pressure = PressureIntent::NativeOnly;
    }
    if matches!(case, "zero-disabled" | "zero-applied") {
        m.spec.future.viscosity = if case == "zero-disabled" {
            ViscosityIntent::Disabled
        } else {
            ViscosityIntent::NewtonianConstant {
                dynamic_viscosity_pa_s: 0.0,
            }
        };
        m.spec.future.viscosity_workspace_limit_bytes = (case == "zero-applied").then_some(CAP);
    }
    reseal(&mut m);
    let mode = match case {
        "candidate" | "tail" => RuntimeMode::Candidate,
        "fallback" => RuntimeMode::HostFallback,
        "deadline" => RuntimeMode::Deadline,
        _ => RuntimeMode::Cancel,
    };
    let mut fixture = RuntimeFixture::new(root, &m, mode);
    if case.starts_with("zero-") {
        fixture.actual_mu = 0.0;
    }
    if case != "empty" {
        if case == "zero-disabled" {
            let forces: Vec<_> = m
                .spec
                .future
                .forces
                .iter()
                .map(ForceSpec::body_force)
                .collect();
            fixture
                .session
                .step_with_configured_pressure(
                    inputs(&m, &forces, DT),
                    &mut fixture.pressure,
                    None,
                    None,
                    None,
                    |_| false,
                )
                .unwrap();
            fixture.append(false).unwrap();
        } else {
            fixture.seed(&m);
        }
        if !matches!(case, "malformed" | "unknown") {
            fixture.attempt(&m);
        }
    }
    if matches!(case, "malformed" | "unknown") {
        fixture
            .writer
            .finish(
                "failed",
                if case == "malformed" {
                    "RCT1:zz"
                } else {
                    "RCT10:00"
                },
                Some(1),
            )
            .unwrap();
    } else {
        let cause = match case {
            "candidate" | "fallback" => RuntimeTerminationCause::Completed,
            "tail" => RuntimeTerminationCause::StorageFailure,
            "empty" => RuntimeTerminationCause::StartupFailure,
            "deadline" => RuntimeTerminationCause::SessionDeadline,
            _ => RuntimeTerminationCause::Cancelled,
        };
        if matches!(case, "candidate" | "fallback") {
            fixture.append(true).unwrap();
        }
        if case == "tail" {
            assert!(matches!(
                fixture.append(true),
                Err(RecordingError::Limit("record count"))
            ));
            assert_eq!(fixture.session.snapshot().state().carrier.generation, 2);
        }
        let mut terminal = fixture.termination(cause);
        if case == "prior" {
            terminal
                .observation
                .as_mut()
                .unwrap()
                .request
                .context
                .prior_pressure_sha256
                .as_mut()
                .unwrap()[0] ^= 1;
        }
        terminal.finish(&mut fixture.writer).unwrap();
    }
    let expected = fixture.expected.clone();
    drop(fixture); // Session, candidate, workspaces and SQLite writer all drop.
    if !matches!(case, "horizon" | "prior" | "malformed" | "unknown") {
        pin(root, &mut m);
    }
    (m, expected)
}

#[test]
fn typed_custody_preserves_unrecorded_tail_empty_and_known_zero_stage_limits() {
    let root = Directory::new();
    for case in ["tail", "empty", "zero-disabled", "zero-applied"] {
        let (m, expected) = produce_runtime_case(&root.0, case);
        let reader = ready(&root.0, &m);
        assert_eq!(reader.count().unwrap(), expected.len());
        let audit = typed_custody(&root.0, &m);
        match case {
            "tail" => {
                assert_eq!(
                    audit.coverage,
                    RuntimeTerminalCoverage::UnrecordedAcceptedTail {
                        last_recorded_generation: Some(1)
                    }
                );
                assert_eq!(audit.termination.accepted_generation, 2);
                assert_eq!(reader.select(1).unwrap().generation, 1);
                assert!(matches!(reader.select(2), Err(RecordingError::Missing(2))));
            }
            "empty" => {
                assert_eq!(
                    audit.coverage,
                    RuntimeTerminalCoverage::NoRecordedPublication
                );
                assert_eq!(audit.termination.accepted_generation, 0);
                assert!(audit.termination.observation.is_none());
            }
            _ => {
                assert_eq!(
                    audit.coverage,
                    RuntimeTerminalCoverage::VerifiedLastPublication
                );
                let receipt =
                    RecordedConfiguredSession::decode(reader.run(), &reader.select(1).unwrap())
                        .unwrap()
                        .receipt
                        .unwrap();
                assert_eq!(receipt.viscosity_applied, case == "zero-applied");
                let request = audit.termination.observation.unwrap().request;
                assert_eq!(request.dynamic_viscosity.to_bits(), 0.0f64.to_bits());
                // The aborted request declares mu only. Neither this accessor
                // nor the test invents an applied-stage flag for that attempt.
            }
        }
    }
}

const RUNTIME_PROCESS_TEST: &str =
    "runtime_custody_producer_exits_before_path_empty_cold_inspection";
const RUNTIME_CASES: [&str; 12] = [
    "candidate",
    "fallback",
    "cancel",
    "deadline",
    "tail",
    "empty",
    "zero-disabled",
    "zero-applied",
    "horizon",
    "prior",
    "malformed",
    "unknown",
];
#[test]
fn runtime_custody_producer_exits_before_path_empty_cold_inspection() {
    match std::env::var(PHASE).ok().as_deref() {
        Some("produce") => {
            let root = PathBuf::from(std::env::var_os(ROOT).unwrap());
            let mut members = vec![];
            for case in RUNTIME_CASES {
                let (m, expected) = produce_runtime_case(&root, case);
                let path = root.join(&m.attempt.as_ref().unwrap().archive_relative_path);
                let sha: [u8; 32] = Sha256::digest(std::fs::read(path).unwrap()).into();
                std::fs::write(
                    root.join(format!("{case}-cold.json")),
                    serde_json::to_vec(&json!({"sha":sha,"rows":expected})).unwrap(),
                )
                .unwrap();
                m.spec
                    .save_new(&root.join(format!("{case}-spec.json")))
                    .unwrap();
                members.push(m);
            }
            std::fs::write(
                root.join("runtime-members.json"),
                serde_json::to_vec(&members).unwrap(),
            )
            .unwrap();
            println!("runtime producer closed twelve archives");
        }
        Some("inspect") => {
            // Only immutable manifests/SQLite and existing typed readers are
            // used here. Session/Candidate constructors are guarded above.
            assert_eq!(std::env::var_os("PATH").unwrap(), "");
            let root = PathBuf::from(std::env::var_os(ROOT).unwrap());
            let members: Vec<ConfiguredRunSetMember> =
                serde_json::from_slice(&std::fs::read(root.join("runtime-members.json")).unwrap())
                    .unwrap();
            assert_eq!(members.len(), RUNTIME_CASES.len());
            for (m, case) in members.iter().zip(RUNTIME_CASES) {
                assert_eq!(m.member_id, case);
                assert_eq!(
                    FrozenConfiguredSpec::open(&root.join(format!("{case}-spec.json"))).unwrap(),
                    m.spec
                );
                assert_eq!(
                    m.attempt.as_ref().unwrap().expected_source_revision,
                    "configured-binding-host-fixture"
                );
                if let PressureIntent::NativePrefixThenConfigured {
                    provider,
                    deployment: DeploymentLocator::Absolute(path),
                    ..
                } = &m.spec.future.pressure
                {
                    assert_eq!(provider.manifest_sha256, fixture_identity().manifest_sha256);
                    assert!(!Path::new(path).exists());
                }
                let expected: Value = serde_json::from_slice(
                    &std::fs::read(root.join(format!("{case}-cold.json"))).unwrap(),
                )
                .unwrap();
                let hash: [u8; 32] = Sha256::digest(
                    std::fs::read(root.join(&m.attempt.as_ref().unwrap().archive_relative_path))
                        .unwrap(),
                )
                .into();
                assert_eq!(json!(hash), expected["sha"]);
                let reader = raw_reader(&root, m);
                assert_eq!(reader.run().identity.run_id, RUN_ID);
                let expected_count = match case {
                    "candidate" | "fallback" => 3,
                    "empty" => 0,
                    _ => 2,
                };
                assert_eq!(reader.count().unwrap(), expected_count);
                let status = reader.status().unwrap();
                assert_eq!(
                    status.outcome,
                    match case {
                        "candidate" | "fallback" => "completed",
                        "cancel" | "zero-disabled" | "zero-applied" | "horizon" | "prior" =>
                            "cancelled",
                        _ => "failed",
                    }
                );
                assert_eq!(
                    status.accepted_generation,
                    Some(match case {
                        "candidate" | "fallback" | "tail" => 2,
                        "empty" => 0,
                        _ => 1,
                    })
                );
                for ordinal in 0..expected_count {
                    assert_eq!(
                        exact(&reader.select(ordinal).unwrap()),
                        expected["rows"][ordinal]
                    );
                    let decoded = RecordedConfiguredSession::decode(
                        reader.run(),
                        &reader.select(ordinal).unwrap(),
                    )
                    .unwrap();
                    assert_eq!(decoded.runtime.is_some(), ordinal == 2);
                }
                if matches!(case, "horizon" | "prior" | "malformed" | "unknown") {
                    assert!(m.attempt.as_ref().unwrap().finalized.is_none());
                    if case == "unknown" {
                        assert!(matches!(open_member(&root, m), MemberState::Unsupported(_)));
                    } else {
                        assert!(matches!(open_member(&root, m), MemberState::Invalid(_)));
                    }
                    assert!(runtime_custody(&reader, &m.spec).is_err());
                    continue;
                }
                assert!(matches!(open_member(&root, m), MemberState::Ready(_)));
                let audit = typed_custody(&root, m);
                let expected_generation = match case {
                    "candidate" | "fallback" | "tail" => 2,
                    "empty" => 0,
                    _ => 1,
                };
                assert_eq!(audit.termination.accepted_generation, expected_generation);
                let expected_cause = match case {
                    "candidate" | "fallback" => RuntimeTerminationCause::Completed,
                    "tail" => RuntimeTerminationCause::StorageFailure,
                    "empty" => RuntimeTerminationCause::StartupFailure,
                    "deadline" => RuntimeTerminationCause::SessionDeadline,
                    _ => RuntimeTerminationCause::Cancelled,
                };
                assert_eq!(audit.termination.cause, expected_cause);
                match case {
                    "tail" => assert_eq!(
                        audit.coverage,
                        RuntimeTerminalCoverage::UnrecordedAcceptedTail {
                            last_recorded_generation: Some(1)
                        }
                    ),
                    "empty" => assert_eq!(
                        audit.coverage,
                        RuntimeTerminalCoverage::NoRecordedPublication
                    ),
                    _ => assert_eq!(
                        audit.coverage,
                        RuntimeTerminalCoverage::VerifiedLastPublication
                    ),
                }
                if case == "empty" {
                    assert!(audit.termination.observation.is_none());
                } else {
                    let observation = audit.termination.observation.unwrap();
                    assert_eq!(observation.request.context.generation, 2);
                    assert_eq!(observation.request.binding, fixture_identity());
                    assert_eq!(observation.request.context.prior_generation, Some(1));
                    if !matches!(case, "candidate" | "fallback" | "tail") {
                        assert!(matches!(
                            observation.outcome,
                            RuntimeOutcome::NoCandidate {
                                reason: NoCandidateReason::Cancelled,
                                dispatched: false,
                                ..
                            }
                        ));
                    }
                }
            }
            println!("PATH-empty reader verified twelve runtime custody cases");
        }
        None => {
            let root = Directory::new();
            // Waiting for output reaps the producing OS before launching reader.
            for (phase, marker) in [
                ("produce", "runtime producer closed twelve archives"),
                (
                    "inspect",
                    "PATH-empty reader verified twelve runtime custody cases",
                ),
            ] {
                let child = Command::new(std::env::current_exe().unwrap())
                    .args(["--exact", RUNTIME_PROCESS_TEST, "--nocapture"])
                    .env(PHASE, phase)
                    .env(ROOT, &root.0)
                    .env("PATH", "")
                    .output()
                    .unwrap();
                let stdout = String::from_utf8_lossy(&child.stdout);
                assert!(
                    child.status.success(),
                    "{phase}: {stdout}\n{}",
                    String::from_utf8_lossy(&child.stderr)
                );
                assert!(stdout.contains(marker));
                assert!(
                    stdout.contains("1 passed; 0 failed;"),
                    "actual child test was not executed: {stdout}"
                );
            }
        }
        Some(other) => panic!("unknown runtime phase {other}"),
    }
}

#[test]
fn frozen_native_seed_lower_bound_survives_unrecorded_tail_without_exact_prefix_claim() {
    let root = Directory::new();
    for case in ["good", "prefix", "mu", "dt"] {
        let mut m = member(&format!("runtime-tail-policy-{case}"), true);
        m.spec.future.recording.max_records = 1;
        reseal(&mut m);
        let mut fixture = RuntimeFixture::new(&root.0, &m, RuntimeMode::HostFallback);
        let forces: Vec<_> = m
            .spec
            .future
            .forces
            .iter()
            .map(ForceSpec::body_force)
            .collect();
        fixture
            .session
            .step_with_configured_pressure(
                inputs(&m, &forces, DT),
                &mut fixture.pressure,
                Some((&mut fixture.viscosity, MU)),
                None,
                None,
                |_| false,
            )
            .unwrap();
        // Real configured candidate is rejected by the host quality gate; this
        // native fallback legitimately increases the actual prefix beyond P=1.
        fixture.attempt(&m);
        let receipt = RecordedConfiguredSession::decode(
            &configured_run(&fixture.session.snapshot(), "temporary live inspection").unwrap(),
            &configured_publication(&fixture.session.snapshot(), 0, CAP).unwrap(),
        )
        .unwrap()
        .receipt
        .unwrap();
        assert_eq!(
            receipt.rejection,
            Some(RecordedProviderRejection::QualityGate)
        );
        assert!(matches!(
            receipt.selected,
            RecordedPressureSelection::Native(_)
        ));
        fixture.provider.mode = RuntimeMode::Cancel;
        let result = fixture.session.step_with_configured_pressure(
            inputs(&m, &forces, DT),
            &mut fixture.pressure,
            Some((&mut fixture.viscosity, MU)),
            Some(&mut fixture.provider),
            None,
            |_| false,
        );
        assert!(matches!(
            result,
            Err(SessionError::Step(LiquidStepError::Carrier(
                SimulationError::Pressure(PressureError::Cancelled { .. })
            )))
        ));
        assert_eq!(fixture.session.snapshot().state().carrier.generation, 2);
        let mut terminal = fixture.termination(RuntimeTerminationCause::Cancelled);
        let context = &mut terminal.observation.as_mut().unwrap().request.context;
        assert_eq!(context.generation, 3);
        assert_eq!(context.native_prefix_steps, 2);
        match case {
            "prefix" => context.native_prefix_steps = 0,
            "dt" => {
                context.dt = 2.0 * DT;
                context.interval_end = context.interval_start + context.dt;
            }
            _ => {}
        }
        if case == "mu" {
            terminal
                .observation
                .as_mut()
                .unwrap()
                .request
                .dynamic_viscosity = 2.0 * MU;
        }
        terminal.finish(&mut fixture.writer).unwrap();
        drop(fixture);
        let reader = raw_reader(&root.0, &m);
        assert_eq!(reader.count().unwrap(), 1);
        let core = rheon::recording::provider::runtime::audit_runtime_termination(&reader).unwrap();
        assert_eq!(
            core.coverage,
            RuntimeTerminalCoverage::UnrecordedAcceptedTail {
                last_recorded_generation: Some(0)
            }
        );
        if case != "good" {
            let expected = match case {
                "prefix" => "configured runtime native prefix below intent",
                "mu" => "configured runtime viscosity differs",
                "dt" => "configured runtime dt exceeds request",
                _ => unreachable!(),
            };
            assert!(matches!(runtime_custody(&reader, &m.spec),
                Err(run_sets::RunSetError::Integrity(reason)) if reason == expected));
            drop(reader);
            refused_before_pin(&root.0, &mut m, false);
        } else {
            assert_eq!(
                typed_custody(&root.0, &m).coverage,
                RuntimeTerminalCoverage::UnrecordedAcceptedTail {
                    last_recorded_generation: Some(0)
                }
            );
            drop(reader);
            pin(&root.0, &mut m);
            assert!(matches!(open_member(&root.0, &m), MemberState::Ready(_)));
        }
    }
}
