//! Real accepted Session/SQLite custody with explicitly authored runtime metadata.
//! This provider evaluates a two-cell equation in-process. No IPC, ModelProcess,
//! trained-model, authentication, restart or wall-time qualification follows.
#![cfg(not(target_arch = "wasm32"))]

use rheon::{
    recording::{
        comparison::Selection,
        diagnostic::NoCandidateReason,
        provider::{
            RecordedConfiguredSession, RecordedPressureSelection, RecordedProviderRejection,
            configured_publication, configured_publication_with_runtime, configured_run,
            runtime::{
                RuntimeObservation, RuntimeOutcome, RuntimeRequestStamp, RuntimeTerminalCoverage,
                RuntimeTermination, RuntimeTerminationCause, audit_runtime_termination,
            },
        },
        *,
    },
    solver_provider::{
        ProviderRejection,
        configured::{ContextPressureProvider, ContextPressureRequest, ContextProviderIdentity},
    },
    *,
};
use sha2::{Digest, Sha256};
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

const CAP: usize = 4 * 1024 * 1024;
const DT: f64 = 1.0 / 16.0;
const MU: f64 = 1.0 / 128.0;
const RUN: u64 = 991;

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "rheon-runtime-recording-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn archive(&self) -> PathBuf {
        self.0.join("archive.sqlite")
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
fn session() -> SimulationSession {
    SimulationSession::new(
        SessionIdentity {
            run_id: RUN,
            source_revision: "authored-runtime-metadata-fixture".into(),
        },
        grid(),
        LiquidTransportConfig {
            carrier: SimulationConfig {
                density: 2.0,
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
            represented_density: 2.0,
            volume_stamp: VolumeStamp {
                id: 510,
                version: 0,
            },
            carrier_id: 511,
        },
        PressureImplementation::JacobiPcgV1,
        vec![1.0; 2],
        CAP,
        8,
    )
    .unwrap()
}
fn workspaces() -> (ProviderStepWorkspace, ViscosityWorkspace) {
    (
        ProviderStepWorkspace::new(&grid(), 2.0, CAP, PressureImplementation::JacobiPcgV1).unwrap(),
        ViscosityWorkspace::new(grid(), CAP).unwrap(),
    )
}
fn inputs() -> LiquidStepInputs<'static> {
    static FORCE: [BodyForce; 1] = [BodyForce {
        value: [0.125, 0.0, 0.0],
        units: ForceUnits::Acceleration,
        region: None,
    }];
    LiquidStepInputs {
        requested_dt: DT,
        smoke_source: None,
        forces: &FORCE,
        inlet: LiquidInlet::new(
            VolumeStamp {
                id: 512,
                version: 0,
            },
            [[0.0; 2]; 3],
        )
        .unwrap(),
        source: None,
        volume: LiquidVolumeSettings {
            max_outward_courant: 1.0,
            actual_divergence_limit: 1e-5,
        },
    }
}
fn identity() -> Arc<ContextProviderIdentity> {
    Arc::new(ContextProviderIdentity {
        id: "authored-two-cell-runtime-metadata".into(),
        version: "1".into(),
        manifest_sha256: [7; 32],
        checkpoint_sha256: None,
        training_run_id: None,
        runtime: "authored-in-process-metadata-NOT-IPC".into(),
        source_revision: "authored-contract-fixture".into(),
    })
}
#[derive(Clone, Copy)]
enum Behavior {
    Good,
    Bad,
    Absent(NoCandidateReason, bool, Option<[u8; 32]>),
    AwaitSessionDeadline,
}
struct ApplicationProvider {
    identity: Arc<ContextProviderIdentity>,
    behavior: Behavior,
    decline: bool,
    supported_request: RefCell<Option<RuntimeRequestStamp>>,
    observation: Option<RuntimeObservation>,
    rhs_hash: Option<[u8; 32]>,
}
impl ApplicationProvider {
    fn new(behavior: Behavior) -> Self {
        Self {
            identity: identity(),
            behavior,
            decline: false,
            supported_request: RefCell::new(None),
            observation: None,
            rhs_hash: None,
        }
    }
    fn observed(&self) -> &RuntimeObservation {
        self.observation.as_ref().unwrap()
    }
}
impl ContextPressureProvider for ApplicationProvider {
    fn identity(&self) -> Arc<ContextProviderIdentity> {
        Arc::clone(&self.identity)
    }
    fn supports(&self, request: &ContextPressureRequest<'_>) -> Result<(), ProviderRejection> {
        *self.supported_request.borrow_mut() =
            Some(RuntimeRequestStamp::capture(request, &self.identity).unwrap());
        if self.decline || request.native_prefix_steps == 0 {
            Err(ProviderRejection::Unsupported(
                "authored minimum one native interval",
            ))
        } else {
            Ok(())
        }
    }
    fn predict(
        &mut self,
        request: &ContextPressureRequest<'_>,
        out: &mut [f64],
        cancelled: &mut dyn FnMut() -> bool,
    ) -> Result<(), ProviderRejection> {
        let stamp = RuntimeRequestStamp::capture(request, &self.identity).unwrap();
        self.rhs_hash = Some(hash_values(request.context.rhs()));
        let behavior = if matches!(self.behavior, Behavior::AwaitSessionDeadline) {
            while !cancelled() {
                std::thread::sleep(Duration::from_millis(1));
            }
            Behavior::Absent(NoCandidateReason::Cancelled, false, None)
        } else {
            self.behavior
        };
        let outcome = match behavior {
            Behavior::Absent(reason, dispatched, invalid_payload_sha256) => {
                RuntimeOutcome::NoCandidate {
                    reason,
                    detail: "Explicitly authored application/runtime observation, not IPC".into(),
                    dispatched,
                    invalid_payload_sha256,
                }
            }
            _ => RuntimeOutcome::CandidateReturned,
        };
        self.observation = Some(RuntimeObservation {
            request: stamp,
            outcome,
        });
        match behavior {
            Behavior::Absent(reason, _, _) => Err(match reason {
                NoCandidateReason::Unsupported | NoCandidateReason::UnsupportedContext => {
                    ProviderRejection::Unsupported("authored runtime unsupported")
                }
                NoCandidateReason::RuntimeFailure | NoCandidateReason::InvalidOutput => {
                    ProviderRejection::Execution("authored runtime/output failure")
                }
                NoCandidateReason::Cancelled => ProviderRejection::Cancelled,
            }),
            Behavior::Good | Behavior::Bad => {
                // Independent two-cell sealed equation: k=h/rho, A*p=(-k*p1,k*p1).
                let c = &request.context;
                assert_eq!(c.rhs()[0], -c.rhs()[1]);
                assert_ne!(c.rhs()[1], 0.0);
                out[0] = 0.0;
                out[1] = c.rhs()[1] * c.density() / c.grid().spacing()[0];
                if matches!(behavior, Behavior::Bad) {
                    out[1] += 100.0;
                }
                Ok(())
            }
            Behavior::AwaitSessionDeadline => unreachable!(),
        }
    }
}
fn hash_values(values: &[f64]) -> [u8; 32] {
    let mut h = Sha256::new();
    for v in values {
        h.update(v.to_bits().to_le_bytes());
    }
    h.finalize().into()
}
fn append(
    writer: &mut StepRecorder,
    session: &SimulationSession,
    observation: Option<&RuntimeObservation>,
) {
    writer
        .append(
            &configured_publication_with_runtime(
                &session.snapshot(),
                writer.recorded_count(),
                writer.limits().max_record_bytes,
                observation,
            )
            .unwrap(),
        )
        .unwrap();
}
fn writer(path: &Path, session: &SimulationSession, max_records: usize) -> StepRecorder {
    let mut w = StepRecorder::create(
        path,
        configured_run(
            &session.snapshot(),
            "Authored metadata fixture; no runtime execution attested",
        )
        .unwrap(),
        RecordingLimits {
            max_records,
            max_database_bytes: 8 * 1024 * 1024,
            max_record_bytes: 1024 * 1024,
        },
    )
    .unwrap();
    append(&mut w, session, None);
    w
}
fn advance(
    session: &mut SimulationSession,
    work: &mut (ProviderStepWorkspace, ViscosityWorkspace),
    provider: &mut ApplicationProvider,
) {
    session
        .step_with_configured_pressure(
            inputs(),
            &mut work.0,
            Some((&mut work.1, MU)),
            Some(provider),
            None,
            |_| false,
        )
        .unwrap();
}
fn seeded(
    path: &Path,
    behavior: Behavior,
    max_records: usize,
) -> (
    SimulationSession,
    (ProviderStepWorkspace, ViscosityWorkspace),
    ApplicationProvider,
    StepRecorder,
) {
    let mut s = session();
    let mut work = workspaces();
    let mut p = ApplicationProvider::new(behavior);
    let mut w = writer(path, &s, max_records);
    advance(&mut s, &mut work, &mut p);
    assert_eq!(s.snapshot().frame(), 1);
    assert!(p.observation.is_none());
    assert!(
        !s.snapshot()
            .configured_provider_report()
            .unwrap()
            .prediction_called
    );
    append(&mut w, &s, None);
    (s, work, p, w)
}
fn row(
    session: &SimulationSession,
    observation: &RuntimeObservation,
) -> Result<RecordedStep, RecordingError> {
    configured_publication_with_runtime(&session.snapshot(), 2, 1024 * 1024, Some(observation))
}
fn reader(path: &Path) -> RecordingReader {
    let r = RecordingReader::open(path, RecordingLimits::default()).unwrap();
    r.verify().unwrap();
    r
}
fn state(session: &SimulationSession) -> Vec<u8> {
    let mut out = Vec::new();
    session.snapshot().write_dense3d(&mut out).unwrap();
    out
}

#[test]
fn captured_request_binds_actual_accepted_context_without_field_arrays() {
    let d = Directory::new();
    let (mut s, mut work, mut p, _) = seeded(&d.archive(), Behavior::Good, 8);
    let prior_hash = hash_values(s.snapshot().state().pressure);
    advance(&mut s, &mut work, &mut p);
    let request = &p.observed().request;
    assert_eq!(request.run_id, RUN);
    assert_eq!(request.grid, grid());
    assert_eq!(request.density.to_bits(), 2.0_f64.to_bits());
    assert_eq!(request.dynamic_viscosity.to_bits(), MU.to_bits());
    assert_eq!(request.binding, *p.identity);
    assert_eq!(
        request.context,
        s.snapshot()
            .configured_provider_report()
            .unwrap()
            .context
            .unwrap()
    );
    assert_eq!(request.context.prior_pressure_sha256, Some(prior_hash));
    assert_eq!(request.context.rhs_sha256, p.rhs_hash.unwrap());
    assert_eq!(request.context.native_prefix_steps, 1);
    request.validate().unwrap();
    let mut zero_prior_endpoint = request.clone();
    zero_prior_endpoint.context.interval_start = 0.0;
    zero_prior_endpoint.context.interval_end = zero_prior_endpoint.context.dt;
    zero_prior_endpoint.context.prior_interval_end = Some(0.0);
    assert!(matches!(
        zero_prior_endpoint.validate(),
        Err(RecordingError::Invalid(_))
    ));
}

#[test]
fn candidate_returned_preserves_actual_selected_or_quality_fallback_receipt() {
    for bad in [false, true] {
        let d = Directory::new();
        let (mut s, mut work, mut p, mut w) = seeded(
            &d.archive(),
            if bad { Behavior::Bad } else { Behavior::Good },
            8,
        );
        advance(&mut s, &mut work, &mut p);
        append(&mut w, &s, Some(p.observed()));
        w.finish(
            "completed",
            "authored metadata, actual accepted interval",
            Some(2),
        )
        .unwrap();
        drop(w);
        let r = reader(&d.archive());
        let raw = r.select(2).unwrap();
        assert!(raw.controls.starts_with("RCP3:"));
        let decoded = RecordedConfiguredSession::decode(r.run(), &raw).unwrap();
        assert!(matches!(
            decoded.runtime.unwrap().outcome,
            RuntimeOutcome::CandidateReturned
        ));
        let receipt = decoded.receipt.unwrap();
        assert!(receipt.prediction_called);
        assert_eq!(
            receipt.viscosity_applied,
            s.snapshot().report().unwrap().viscosity.is_some()
        );
        if bad {
            assert_eq!(
                receipt.selected,
                RecordedPressureSelection::Native(PressureImplementation::JacobiPcgV1)
            );
            assert_eq!(
                receipt.rejection,
                Some(RecordedProviderRejection::QualityGate)
            );
            assert!(receipt.candidate.is_some());
        } else {
            assert_eq!(receipt.selected, RecordedPressureSelection::Configured);
            assert!(receipt.rejection.is_none());
        }
        assert_eq!(
            decoded
                .interval_pressure
                .unwrap()
                .1
                .iter()
                .map(|v| v.to_bits())
                .collect::<Vec<_>>(),
            s.snapshot()
                .state()
                .pressure
                .iter()
                .map(|v| v.to_bits())
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn authored_no_candidate_reasons_survive_with_actual_native_fallback() {
    for reason in [
        NoCandidateReason::Unsupported,
        NoCandidateReason::UnsupportedContext,
        NoCandidateReason::RuntimeFailure,
        NoCandidateReason::InvalidOutput,
    ] {
        let digest = (reason == NoCandidateReason::InvalidOutput).then_some([0xa5; 32]);
        let d = Directory::new();
        let (mut s, mut work, mut p, mut w) =
            seeded(&d.archive(), Behavior::Absent(reason, true, digest), 8);
        advance(&mut s, &mut work, &mut p);
        append(&mut w, &s, Some(p.observed()));
        w.finish(
            "completed",
            "authored failure observation; native interval accepted",
            Some(2),
        )
        .unwrap();
        drop(w);
        let r = reader(&d.archive());
        let decoded = RecordedConfiguredSession::decode(r.run(), &r.select(2).unwrap()).unwrap();
        let RuntimeOutcome::NoCandidate {
            reason: saved,
            dispatched,
            invalid_payload_sha256,
            ..
        } = decoded.runtime.unwrap().outcome
        else {
            panic!("missing runtime observation")
        };
        assert_eq!(saved, reason);
        assert!(dispatched); // Authored metadata, not a measured dispatch event.
        assert_eq!(invalid_payload_sha256, digest);
        let receipt = decoded.receipt.unwrap();
        assert_eq!(
            receipt.selected,
            RecordedPressureSelection::Native(PressureImplementation::JacobiPcgV1)
        );
        assert!(receipt.candidate.is_none());
        match reason {
            NoCandidateReason::Unsupported | NoCandidateReason::UnsupportedContext => {
                assert!(matches!(
                    receipt.rejection,
                    Some(RecordedProviderRejection::Unsupported(_))
                ))
            }
            _ => assert!(matches!(
                receipt.rejection,
                Some(RecordedProviderRejection::Execution(_))
            )),
        }
    }
}

#[test]
fn invalid_payload_digest_requires_invalid_output_and_authored_dispatch() {
    let d = Directory::new();
    let (mut s, mut work, mut p, _) = seeded(&d.archive(), Behavior::Good, 8);
    advance(&mut s, &mut work, &mut p);
    for (reason, dispatched) in [
        (NoCandidateReason::RuntimeFailure, true),
        (NoCandidateReason::Unsupported, true),
        (NoCandidateReason::Cancelled, true),
        (NoCandidateReason::InvalidOutput, false),
    ] {
        let mut obs = p.observed().clone();
        obs.outcome = RuntimeOutcome::NoCandidate {
            reason,
            detail: "authored invalid digest combination".into(),
            dispatched,
            invalid_payload_sha256: Some([9; 32]),
        };
        assert!(obs.validate().is_err());
    }
}

#[test]
fn cancelled_runtime_observation_cannot_be_attached_to_accepted_publication() {
    let d = Directory::new();
    let (mut s, mut work, mut p, _) = seeded(&d.archive(), Behavior::Good, 8);
    advance(&mut s, &mut work, &mut p);
    let mut obs = p.observed().clone();
    obs.outcome = RuntimeOutcome::NoCandidate {
        reason: NoCandidateReason::Cancelled,
        detail: "authored cancellation".into(),
        dispatched: false,
        invalid_payload_sha256: None,
    };
    obs.validate().unwrap();
    assert!(row(&s, &obs).is_err());
}

#[test]
fn rcp2_keeps_runtime_absent_even_when_core_prediction_was_called() {
    let d = Directory::new();
    let (mut s, mut work, mut p, _) = seeded(&d.archive(), Behavior::Good, 8);
    advance(&mut s, &mut work, &mut p);
    for raw in [
        configured_publication(&s.snapshot(), 2, 1024 * 1024).unwrap(),
        configured_publication_with_runtime(&s.snapshot(), 2, 1024 * 1024, None).unwrap(),
    ] {
        assert!(raw.controls.starts_with("RCP2:"));
        let run = configured_run(&s.snapshot(), "legacy absence remains absent").unwrap();
        let decoded = RecordedConfiguredSession::decode(&run, &raw).unwrap();
        assert!(decoded.runtime.is_none());
        assert!(decoded.receipt.unwrap().prediction_called);
    }
}

#[test]
fn support_method_entry_does_not_invent_prediction_or_runtime_observation() {
    let d = Directory::new();
    let (mut s, mut work, mut p, _) = seeded(&d.archive(), Behavior::Good, 8);
    p.decline = true;
    advance(&mut s, &mut work, &mut p);
    assert!(p.observation.is_none());
    assert!(
        !s.snapshot()
            .configured_provider_report()
            .unwrap()
            .prediction_called
    );
    let obs = RuntimeObservation {
        request: p.supported_request.borrow().clone().unwrap(),
        outcome: RuntimeOutcome::NoCandidate {
            reason: NoCandidateReason::Unsupported,
            detail: "authored support-only decline".into(),
            dispatched: false,
            invalid_payload_sha256: None,
        },
    };
    obs.validate().unwrap();
    assert!(row(&s, &obs).is_err());
}

#[test]
fn stale_cross_run_package_context_and_physical_stamp_are_rejected() {
    let d = Directory::new();
    let (mut s, mut work, mut p, _) = seeded(&d.archive(), Behavior::Good, 8);
    advance(&mut s, &mut work, &mut p);
    let original = p.observed().clone();
    for case in 0..10 {
        let mut obs = original.clone();
        match case {
            0 => obs.request.run_id += 1,
            1 => obs.request.binding.manifest_sha256[0] ^= 1,
            2 => obs.request.context.rhs_sha256[0] ^= 1,
            3 => obs.request.context.prior_pressure_sha256.as_mut().unwrap()[0] ^= 1,
            4 => obs.request.dynamic_viscosity *= 2.0,
            5 => obs.request.density *= 2.0,
            6 => obs.request.grid = GridGeometry::new([2, 1, 1], [0.5; 3], [0.0; 3]).unwrap(),
            7 => obs.request.context.interval_start += DT,
            8 => obs.request.context.native_prefix_steps += 1,
            _ => obs.request.binding.runtime.push_str("-different"),
        }
        assert!(row(&s, &obs).is_err(), "stamp case {case}");
    }
    advance(&mut s, &mut work, &mut p);
    assert!(
        row(&s, &original).is_err(),
        "previous interval observation must stay stale"
    );
}

#[test]
fn cancellation_and_session_deadline_retain_seed_and_distinct_typed_causes() {
    for deadline in [false, true] {
        let d = Directory::new();
        let behavior = if deadline {
            Behavior::AwaitSessionDeadline
        } else {
            Behavior::Absent(NoCandidateReason::Cancelled, false, None)
        };
        let (mut s, mut work, mut p, mut w) = seeded(&d.archive(), behavior, 8);
        let before = state(&s);
        let result = s.step_with_configured_pressure(
            inputs(),
            &mut work.0,
            Some((&mut work.1, MU)),
            Some(&mut p),
            deadline.then_some(Instant::now() + Duration::from_secs(1)),
            |_| false,
        );
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
        assert_eq!(state(&s), before);
        assert_eq!(s.snapshot().frame(), 1);
        assert_eq!(w.recorded_count(), 2);
        let cause = if deadline {
            RuntimeTerminationCause::SessionDeadline
        } else {
            RuntimeTerminationCause::Cancelled
        };
        let termination = RuntimeTermination::capture(
            &s.snapshot(),
            cause,
            "actual Session abort; authored runtime metadata",
            Some(p.observed()),
        )
        .unwrap();
        termination.finish(&mut w).unwrap();
        drop(w);
        let r = reader(&d.archive());
        let audit = audit_runtime_termination(&r).unwrap();
        assert_eq!(audit.termination.cause, cause);
        assert_eq!(audit.termination.accepted_generation, 1);
        assert_eq!(audit.termination.accepted_time.to_bits(), DT.to_bits());
        assert!(matches!(
            audit.coverage,
            RuntimeTerminalCoverage::VerifiedLastPublication
        ));
        assert_eq!(
            r.status().unwrap().outcome,
            if deadline { "failed" } else { "cancelled" }
        );
        assert_eq!(r.count().unwrap(), 2);
        drop(r);
        drop(p);
        drop(work);
        drop(s);
        cold_reopen(&d.archive(), cause, 1, 2, "verified", "cancelled-next");
    }
}

#[test]
fn accepted_fallback_can_end_completed_or_runtime_failed_without_rollback() {
    for cause in [
        RuntimeTerminationCause::Completed,
        RuntimeTerminationCause::RuntimeFailure,
    ] {
        let d = Directory::new();
        let (mut s, mut work, mut p, mut w) = seeded(
            &d.archive(),
            Behavior::Absent(NoCandidateReason::RuntimeFailure, true, None),
            8,
        );
        advance(&mut s, &mut work, &mut p);
        append(&mut w, &s, Some(p.observed()));
        RuntimeTermination::capture(
            &s.snapshot(),
            cause,
            "caller disposition after actual accepted native fallback",
            Some(p.observed()),
        )
        .unwrap()
        .finish(&mut w)
        .unwrap();
        drop(w);
        let r = reader(&d.archive());
        let audit = audit_runtime_termination(&r).unwrap();
        assert_eq!(audit.termination.cause, cause);
        assert_eq!(audit.termination.accepted_generation, 2);
        assert!(matches!(
            audit.coverage,
            RuntimeTerminalCoverage::VerifiedLastPublication
        ));
        assert_eq!(
            r.status().unwrap().outcome,
            if matches!(cause, RuntimeTerminationCause::Completed) {
                "completed"
            } else {
                "failed"
            }
        );
    }
}

#[test]
fn startup_failure_has_no_invented_observation_or_recorded_publication() {
    let d = Directory::new();
    let s = session();
    let mut w = StepRecorder::create(
        &d.archive(),
        configured_run(&s.snapshot(), "authored startup failure").unwrap(),
        RecordingLimits::default(),
    )
    .unwrap();
    RuntimeTermination::capture(
        &s.snapshot(),
        RuntimeTerminationCause::StartupFailure,
        "authored failure before provider or numerical step",
        None,
    )
    .unwrap()
    .finish(&mut w)
    .unwrap();
    drop(w);
    let r = reader(&d.archive());
    let audit = audit_runtime_termination(&r).unwrap();
    assert_eq!(r.count().unwrap(), 0);
    assert_eq!(audit.termination.accepted_generation, 0);
    assert_eq!(audit.termination.accepted_time.to_bits(), 0.0_f64.to_bits());
    assert!(audit.termination.observation.is_none());
    assert!(matches!(
        audit.coverage,
        RuntimeTerminalCoverage::NoRecordedPublication
    ));
    drop(r);
    drop(s);
    cold_reopen(
        &d.archive(),
        RuntimeTerminationCause::StartupFailure,
        0,
        0,
        "empty",
        "none",
    );
}

#[test]
fn actual_numerical_rejection_retains_seed_without_inventing_runtime_request() {
    let d = Directory::new();
    let (mut s, mut work, mut p, mut w) = seeded(&d.archive(), Behavior::Good, 8);
    let before = state(&s);
    let last_support = p.supported_request.borrow().clone();
    let mut invalid = inputs();
    invalid.requested_dt = 0.0;
    assert!(
        s.step_with_configured_pressure(
            invalid,
            &mut work.0,
            Some((&mut work.1, MU)),
            Some(&mut p),
            None,
            |_| false,
        )
        .is_err()
    );
    assert_eq!(state(&s), before);
    assert_eq!(s.snapshot().frame(), 1);
    assert_eq!(w.recorded_count(), 2);
    assert!(p.observation.is_none());
    assert_eq!(*p.supported_request.borrow(), last_support);
    RuntimeTermination::capture(
        &s.snapshot(),
        RuntimeTerminationCause::NumericalFailure,
        "actual zero-dt Session rejection before provider request; retained accepted seed",
        None,
    )
    .unwrap()
    .finish(&mut w)
    .unwrap();
    drop(w);
    let r = reader(&d.archive());
    let audit = audit_runtime_termination(&r).unwrap();
    assert_eq!(
        audit.termination.cause,
        RuntimeTerminationCause::NumericalFailure
    );
    assert_eq!(audit.termination.accepted_generation, 1);
    assert_eq!(audit.termination.accepted_time.to_bits(), DT.to_bits());
    assert!(audit.termination.observation.is_none());
    assert!(matches!(
        audit.coverage,
        RuntimeTerminalCoverage::VerifiedLastPublication
    ));
    assert_eq!(r.status().unwrap().outcome, "failed");
    assert_eq!(r.count().unwrap(), 2);
}

#[test]
fn generic_active_and_legacy_details_do_not_decode_as_runtime_termination() {
    let d = Directory::new();
    let s = session();
    let mut w = writer(&d.archive(), &s, 8);
    {
        let r = reader(&d.archive());
        assert!(matches!(
            audit_runtime_termination(&r),
            Err(RecordingError::Unsupported(_))
        ));
    }
    w.finish("failed", "legacy caller declaration, not RCT1", Some(0))
        .unwrap();
    drop(w);
    let r = reader(&d.archive());
    assert!(matches!(
        RuntimeTermination::decode(r.run(), &r.status().unwrap()),
        Err(RecordingError::Unsupported(_))
    ));
    assert!(matches!(
        audit_runtime_termination(&r),
        Err(RecordingError::Unsupported(_))
    ));
}

#[test]
fn append_quota_failure_retains_explicit_unrecorded_accepted_tail_coverage() {
    let d = Directory::new();
    let (mut s, mut work, mut p, mut w) = seeded(&d.archive(), Behavior::Good, 2);
    advance(&mut s, &mut work, &mut p);
    let raw = row(&s, p.observed()).unwrap();
    assert!(matches!(w.append(&raw), Err(RecordingError::Limit(_))));
    assert!(
        w.append(&raw).is_err(),
        "poisoned writer must not retry guessed state"
    );
    RuntimeTermination::capture(
        &s.snapshot(),
        RuntimeTerminationCause::StorageFailure,
        "actual append count limit after numerical acceptance",
        Some(p.observed()),
    )
    .unwrap()
    .finish(&mut w)
    .unwrap();
    drop(w);
    let r = reader(&d.archive());
    let audit = audit_runtime_termination(&r).unwrap();
    assert_eq!(r.count().unwrap(), 2);
    assert_eq!(r.status().unwrap().outcome, "failed");
    assert_eq!(audit.termination.accepted_generation, 2);
    assert!(matches!(
        audit.coverage,
        RuntimeTerminalCoverage::UnrecordedAcceptedTail {
            last_recorded_generation: Some(1)
        }
    ));
    drop(r);
    drop(raw);
    drop(p);
    drop(work);
    drop(s);
    cold_reopen(
        &d.archive(),
        RuntimeTerminationCause::StorageFailure,
        2,
        2,
        "tail-1",
        "candidate-tail",
    );
}

#[test]
fn cold_unrecorded_tail_does_not_discard_available_clock_or_prior_evidence() {
    for corrupt_prior in [false, true] {
        let d = Directory::new();
        let (mut s, mut work, mut p, mut w) = seeded(&d.archive(), Behavior::Good, 2);
        advance(&mut s, &mut work, &mut p);
        let raw = row(&s, p.observed()).unwrap();
        assert!(matches!(w.append(&raw), Err(RecordingError::Limit(_))));
        assert_eq!(s.snapshot().frame(), 2);
        assert_eq!(w.recorded_count(), 2); // Constructor plus actual accepted seed1.
        let mut termination = RuntimeTermination::capture(
            &s.snapshot(),
            RuntimeTerminationCause::StorageFailure,
            "actual append quota failure; deliberately corrupted application terminal metadata",
            corrupt_prior.then_some(p.observed()),
        )
        .unwrap();
        if corrupt_prior {
            // Authored metadata corruption, not an observed runtime event:
            // gen2 was accepted but its request's gen1 prior IS still retained.
            termination
                .observation
                .as_mut()
                .unwrap()
                .request
                .context
                .prior_pressure_sha256
                .as_mut()
                .unwrap()[0] ^= 1;
        } else {
            // Positive and intrinsically finite, but cannot describe gen2 after
            // the retained gen1 publication at exactly this same endpoint.
            termination.accepted_time = DT;
        }
        termination.finish(&mut w).unwrap();
        drop(w);
        let r = reader(&d.archive()); // Generic archive integrity is still valid.
        assert_eq!(r.status().unwrap().accepted_generation, Some(2));
        assert_eq!(r.count().unwrap(), 2);
        let expected = if corrupt_prior {
            "runtime attempted prior association"
        } else {
            "runtime terminal unrecorded time"
        };
        assert!(matches!(audit_runtime_termination(&r),
            Err(RecordingError::Invalid(reason)) if reason == expected));
    }
}

#[test]
fn runtime_publication_codec_refuses_unknown_version_and_trailing_bytes() {
    let d = Directory::new();
    let (mut s, mut work, mut p, _) = seeded(&d.archive(), Behavior::Good, 8);
    advance(&mut s, &mut work, &mut p);
    let run = configured_run(&s.snapshot(), "codec adversarial input").unwrap();
    let raw = row(&s, p.observed()).unwrap();
    assert!(raw.controls.starts_with("RCP3:"));
    for unknown in [false, true] {
        let mut changed = row(&s, p.observed()).unwrap();
        if unknown {
            changed.controls = changed.controls.replacen("RCP3:", "RCP9:", 1);
        } else {
            changed.controls.push_str("00");
        }
        assert!(RecordedConfiguredSession::decode(&run, &changed).is_err());
    }
}

#[test]
fn oversized_observation_text_and_identity_are_bounded_before_publication() {
    let d = Directory::new();
    let (mut s, mut work, mut p, _) = seeded(&d.archive(), Behavior::Good, 8);
    advance(&mut s, &mut work, &mut p);
    let mut obs = p.observed().clone();
    obs.outcome = RuntimeOutcome::NoCandidate {
        reason: NoCandidateReason::RuntimeFailure,
        detail: "x".repeat(64 * 1024),
        dispatched: true,
        invalid_payload_sha256: None,
    };
    assert!(obs.validate().is_err());
    let mut identity = p.observed().clone();
    identity.request.binding.id = "x".repeat(1001);
    assert!(identity.validate().is_err());
    assert!(configured_publication_with_runtime(&s.snapshot(), 2, 1, Some(p.observed())).is_err());

    // Each declaration is independently valid. It is the duplicated binding
    // across the core receipt and explicit runtime supplement that exceeds the
    // shared 16 KiB encoded control-text budget, not an invalid individual field.
    let mut large = ApplicationProvider::new(Behavior::Good);
    let mut large_binding = (*large.identity).clone();
    large_binding.id = "i".repeat(1000);
    large_binding.version = "v".repeat(1000);
    large_binding.runtime = "r".repeat(1000);
    large_binding.source_revision = "s".repeat(1000);
    large_binding.training_run_id = Some("t".repeat(1000));
    large_binding.validate().unwrap();
    large.identity = Arc::new(large_binding);
    let mut actual = session();
    let mut actual_work = workspaces();
    advance(&mut actual, &mut actual_work, &mut large);
    assert_eq!(actual.snapshot().frame(), 1);
    assert!(large.observation.is_none());
    advance(&mut actual, &mut actual_work, &mut large);
    assert_eq!(actual.snapshot().frame(), 2);
    assert_eq!(
        actual
            .snapshot()
            .configured_provider_report()
            .unwrap()
            .pressure
            .selected,
        "configured-pressure-context-v1"
    );
    large.observed().validate().unwrap();
    let legacy = configured_publication(&actual.snapshot(), 2, 1024 * 1024).unwrap();
    assert!(legacy.controls.starts_with("RCP2:"));
    assert!(legacy.controls.len() <= 16 * 1024);
    let run = configured_run(&actual.snapshot(), "valid aggregate metadata boundary").unwrap();
    assert!(
        RecordedConfiguredSession::decode(&run, &legacy)
            .unwrap()
            .runtime
            .is_none()
    );
    assert!(matches!(
        configured_publication_with_runtime(
            &actual.snapshot(),
            2,
            1024 * 1024,
            Some(large.observed())
        ),
        Err(RecordingError::Limit(_))
    ));
    // The refused supplement neither truncates nor rewrites the valid RCP2 row.
    let again = configured_publication(&actual.snapshot(), 2, 1024 * 1024).unwrap();
    assert_eq!(legacy.controls, again.controls);
    assert_eq!(legacy.fields, again.fields);
}

#[test]
fn terminal_codec_refuses_versions_trailing_bytes_and_status_source_misassociation() {
    let d = Directory::new();
    let (mut s, mut work, mut p, mut w) = seeded(&d.archive(), Behavior::Good, 8);
    advance(&mut s, &mut work, &mut p);
    append(&mut w, &s, Some(p.observed()));
    RuntimeTermination::capture(
        &s.snapshot(),
        RuntimeTerminationCause::Completed,
        "complete authored runtime fixture",
        Some(p.observed()),
    )
    .unwrap()
    .finish(&mut w)
    .unwrap();
    drop(w);
    let r = reader(&d.archive());
    let original = r.status().unwrap();
    assert!(original.detail.starts_with("RCT1:"));
    for case in 0..4 {
        let mut changed = r.status().unwrap();
        match case {
            0 => changed.detail = changed.detail.replacen("RCT1:", "RCT9:", 1),
            1 => changed.detail.push_str("00"),
            2 => changed.accepted_generation = Some(1),
            _ => changed.outcome = "cancelled".into(),
        }
        assert!(
            RuntimeTermination::decode(r.run(), &changed).is_err(),
            "status case {case}"
        );
    }
    let mut other = r.run().clone();
    other.identity.run_id += 1;
    assert!(RuntimeTermination::decode(&other, &original).is_err());
}

#[test]
fn terminal_rejects_other_run_far_future_request_and_startup_with_prediction() {
    let d = Directory::new();
    let (mut s, mut work, mut p, _) = seeded(&d.archive(), Behavior::Good, 8);
    advance(&mut s, &mut work, &mut p);
    for case in 0..2 {
        let mut obs = p.observed().clone();
        if case == 0 {
            obs.request.run_id += 1;
        } else {
            let c = &mut obs.request.context;
            c.generation = 4;
            c.interval_start = 3.0 * DT;
            c.interval_end = 4.0 * DT;
            c.prior_generation = Some(3);
            c.prior_interval_end = Some(3.0 * DT);
        }
        assert!(
            RuntimeTermination::capture(
                &s.snapshot(),
                RuntimeTerminationCause::RuntimeFailure,
                "adversarial request association",
                Some(&obs)
            )
            .is_err()
        );
    }
    assert!(
        RuntimeTermination::capture(
            &s.snapshot(),
            RuntimeTerminationCause::StartupFailure,
            "cannot invent startup after prediction",
            Some(p.observed())
        )
        .is_err()
    );
}

fn file_sha(path: &Path) -> String {
    format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()))
}
fn selection_sha(r: &RecordingReader, ordinal: usize) -> String {
    Selection::read(r, ordinal)
        .unwrap()
        .digest()
        .iter()
        .map(|v| format!("{v:02x}"))
        .collect()
}

fn cold_reopen(
    path: &Path,
    cause: RuntimeTerminationCause,
    generation: u64,
    count: usize,
    coverage: &str,
    observation: &str,
) {
    // Expectations describe the known source scenario. Fingerprints bind exact
    // disk publications; neither the expected cause nor coverage comes from a
    // decode of the same declared terminal payload being tested by the child.
    let r = reader(path);
    assert_eq!(r.count().unwrap(), count);
    assert_eq!(r.status().unwrap().accepted_generation, Some(generation));
    let fingerprints = (0..count)
        .map(|i| selection_sha(&r, i))
        .collect::<Vec<_>>()
        .join(",");
    drop(r);
    let sha = file_sha(path);
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "cold_runtime_reader_child",
            "--nocapture",
            "--color",
            "never",
        ])
        .env("PATH", "")
        .env("RHEON_RUNTIME_RECORDING_COLD", path)
        .env("RHEON_RUNTIME_RECORDING_SHA", &sha)
        .env("RHEON_RUNTIME_RECORDING_SELECTIONS", fingerprints)
        .env("RHEON_RUNTIME_RECORDING_CAUSE", format!("{cause:?}"))
        .env("RHEON_RUNTIME_RECORDING_GENERATION", generation.to_string())
        .env("RHEON_RUNTIME_RECORDING_COUNT", count.to_string())
        .env("RHEON_RUNTIME_RECORDING_COVERAGE", coverage)
        .env("RHEON_RUNTIME_RECORDING_OBSERVATION", observation)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "{stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains("1 passed; 0 failed;"),
        "exact child must run one actual test: {stdout}"
    );
    assert!(stdout.contains("cold runtime archive verified without provider execution"));
    assert_eq!(file_sha(path), sha);
}

#[test]
fn closed_runtime_archive_reopens_in_exact_cold_reader_without_runtime_path() {
    for failure in [false, true] {
        let d = Directory::new();
        let path = d.archive();
        let behavior = if failure {
            Behavior::Absent(NoCandidateReason::InvalidOutput, true, Some([5; 32]))
        } else {
            Behavior::Good
        };
        let (mut s, mut work, mut p, mut w) = seeded(&path, behavior, 8);
        advance(&mut s, &mut work, &mut p);
        append(&mut w, &s, Some(p.observed()));
        RuntimeTermination::capture(
            &s.snapshot(),
            if failure {
                RuntimeTerminationCause::RuntimeFailure
            } else {
                RuntimeTerminationCause::Completed
            },
            "authored observation, actual accepted interval",
            Some(p.observed()),
        )
        .unwrap()
        .finish(&mut w)
        .unwrap();
        drop(w);
        drop(p);
        drop(s);
        drop(work);
        cold_reopen(
            &path,
            if failure {
                RuntimeTerminationCause::RuntimeFailure
            } else {
                RuntimeTerminationCause::Completed
            },
            2,
            3,
            "verified",
            if failure { "invalid" } else { "candidate" },
        );
    }
}

#[test]
fn cold_runtime_reader_child() {
    let Some(path) = std::env::var_os("RHEON_RUNTIME_RECORDING_COLD") else {
        return;
    };
    // Actual child branch uses only reader/decode/audit. No Session, provider,
    // package, executable lookup, model import or synthetic prediction is called.
    assert_eq!(std::env::var("PATH").unwrap(), "");
    let path = PathBuf::from(path);
    let sha = std::env::var("RHEON_RUNTIME_RECORDING_SHA").unwrap();
    assert_eq!(file_sha(&path), sha);
    let r = reader(&path);
    let expected_cause = std::env::var("RHEON_RUNTIME_RECORDING_CAUSE").unwrap();
    let generation: u64 = std::env::var("RHEON_RUNTIME_RECORDING_GENERATION")
        .unwrap()
        .parse()
        .unwrap();
    let count: usize = std::env::var("RHEON_RUNTIME_RECORDING_COUNT")
        .unwrap()
        .parse()
        .unwrap();
    assert!(count <= 3 && generation <= 2);
    let expected = std::env::var("RHEON_RUNTIME_RECORDING_SELECTIONS").unwrap();
    let expected = if expected.is_empty() {
        Vec::new()
    } else {
        expected.split(',').collect::<Vec<_>>()
    };
    assert_eq!(r.count().unwrap(), count);
    assert_eq!(expected.len(), count);
    assert_eq!(r.run().identity.run_id, RUN);
    for (i, expected) in expected.iter().enumerate() {
        assert_eq!(&selection_sha(&r, i), expected);
        let raw = r.select(i).unwrap();
        let decoded = RecordedConfiguredSession::decode(r.run(), &raw).unwrap();
        assert_eq!(decoded.generation, i as u64);
        if i < 2 {
            assert!(raw.controls.starts_with("RCP2:"));
            assert!(
                decoded.runtime.is_none(),
                "constructor/seed contain no runtime observation"
            );
        }
    }
    let audit = audit_runtime_termination(&r).unwrap();
    assert_eq!(format!("{:?}", audit.termination.cause), expected_cause);
    assert_eq!(audit.termination.accepted_generation, generation);
    assert_eq!(
        audit.termination.accepted_time.to_bits(),
        (generation as f64 * DT).to_bits()
    );
    let status = r.status().unwrap();
    assert_eq!(status.accepted_generation, Some(generation));
    assert_eq!(
        status.outcome,
        match expected_cause.as_str() {
            "Completed" => "completed",
            "Cancelled" => "cancelled",
            _ => "failed",
        }
    );
    let coverage = std::env::var("RHEON_RUNTIME_RECORDING_COVERAGE").unwrap();
    match coverage.as_str() {
        "verified" => assert!(matches!(
            audit.coverage,
            RuntimeTerminalCoverage::VerifiedLastPublication
        )),
        "tail-1" => assert!(matches!(
            audit.coverage,
            RuntimeTerminalCoverage::UnrecordedAcceptedTail {
                last_recorded_generation: Some(1)
            }
        )),
        "empty" => assert!(matches!(
            audit.coverage,
            RuntimeTerminalCoverage::NoRecordedPublication
        )),
        _ => panic!("unknown source-scenario coverage"),
    }
    let expected_observation = std::env::var("RHEON_RUNTIME_RECORDING_OBSERVATION").unwrap();
    if expected_observation == "none" {
        assert!(audit.termination.observation.is_none());
        assert_eq!((generation, count), (0, 0));
        assert_eq!(expected_cause, "StartupFailure");
    } else {
        let obs = audit.termination.observation.as_ref().unwrap();
        assert_eq!(obs.request.run_id, RUN);
        assert_eq!(obs.request.grid, grid());
        assert_eq!(obs.request.density.to_bits(), 2.0_f64.to_bits());
        assert_eq!(obs.request.dynamic_viscosity.to_bits(), MU.to_bits());
        assert_eq!(obs.request.context.generation, 2);
        assert_eq!(obs.request.context.native_prefix_steps, 1);
        assert_eq!(obs.request.context.prior_generation, Some(1));
        assert_eq!(obs.request.context.interval_start.to_bits(), DT.to_bits());
        assert_eq!(
            obs.request.context.interval_end.to_bits(),
            (2.0 * DT).to_bits()
        );
        let last =
            RecordedConfiguredSession::decode(r.run(), &r.select(count - 1).unwrap()).unwrap();
        match expected_observation.as_str() {
            "candidate" | "invalid" => {
                assert_eq!((generation, count), (2, 3));
                assert_eq!(last.runtime.as_ref(), Some(obs));
                if expected_observation == "candidate" {
                    assert!(matches!(obs.outcome, RuntimeOutcome::CandidateReturned));
                } else {
                    assert!(matches!(obs.outcome, RuntimeOutcome::NoCandidate {
                        reason: NoCandidateReason::InvalidOutput, dispatched: true,
                        invalid_payload_sha256: Some(digest), ..
                    } if digest == [5; 32]));
                }
            }
            "cancelled-next" | "candidate-tail" => {
                assert_eq!(count, 2);
                assert!(last.runtime.is_none());
                assert_eq!(last.generation, 1);
                assert_eq!(
                    obs.request.context.prior_pressure_sha256,
                    Some(hash_values(&last.interval_pressure.as_ref().unwrap().1))
                );
                if expected_observation == "cancelled-next" {
                    assert_eq!(generation, 1);
                    assert!(!last.receipt.unwrap().prediction_called);
                    assert!(matches!(
                        obs.outcome,
                        RuntimeOutcome::NoCandidate {
                            reason: NoCandidateReason::Cancelled,
                            dispatched: false,
                            invalid_payload_sha256: None,
                            ..
                        }
                    ));
                } else {
                    assert_eq!(generation, 2);
                    assert!(matches!(obs.outcome, RuntimeOutcome::CandidateReturned));
                    // The gen2 accepted fields are not recorded. Only available
                    // gen1 prior evidence was checked; coverage must stay a tail.
                    assert_eq!(coverage, "tail-1");
                }
            }
            _ => panic!("unknown source-scenario observation"),
        }
    }
    assert_eq!(file_sha(&path), sha);
    println!("cold runtime archive verified without provider execution");
}
