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
