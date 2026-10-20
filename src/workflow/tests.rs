use super::*;
use crate::solver_provider::{
    ProviderRejection,
    configured::{ContextPressureProvider, ContextPressureRequest},
};
use crate::*;
use std::sync::Arc;

const CAP: usize = 16 * 1024 * 1024;
fn fixture(edge: usize, run_id: u64) -> (SimulationSession, ProviderStepWorkspace) {
    fixture_fraction(edge, run_id, 1.0)
}
fn fixture_fraction(
    edge: usize,
    run_id: u64,
    fraction: f64,
) -> (SimulationSession, ProviderStepWorkspace) {
    let grid = GridGeometry::new([edge as u64; 3], [1.0 / edge as f64; 3], [0.0; 3]).unwrap();
    let config = LiquidTransportConfig {
        carrier: SimulationConfig {
            density: 998.2,
            memory_limit: CAP,
            pressure: PressureSettings {
                relative_residual: 1e-10,
                absolute_residual: 1e-12,
                divergence_limit: 1e-8,
                max_iterations: 2000,
            },
            actual_divergence_limit: 1e-5,
            max_courant: 1.0,
        },
        represented_density: 998.2,
        volume_stamp: VolumeStamp { id: 1, version: 0 },
        carrier_id: 2,
    };
    let workspace =
        ProviderStepWorkspace::new(&grid, 998.2, CAP, PressureImplementation::JacobiPcgV1).unwrap();
    (
        SimulationSession::new(
            SessionIdentity {
                run_id,
                source_revision: "workflow-fixture".into(),
            },
            grid,
            config,
            PressureImplementation::JacobiPcgV1,
            vec![fraction; edge.pow(3)],
            CAP,
            1000,
        )
        .unwrap(),
        workspace,
    )
}
fn inputs(forces: &[BodyForce], dt: f64) -> LiquidStepInputs<'_> {
    LiquidStepInputs {
        requested_dt: dt,
        smoke_source: None,
        forces,
        source: None,
        inlet: LiquidInlet::new(VolumeStamp { id: 3, version: 0 }, [[0.0; 2]; 3]).unwrap(),
        volume: LiquidVolumeSettings {
            max_outward_courant: 1.0,
            actual_divergence_limit: 1e-5,
        },
    }
}
struct FixtureProvider {
    reject_support: bool,
    invalid_output: bool,
}
impl ContextPressureProvider for FixtureProvider {
    fn identity(&self) -> Arc<ContextProviderIdentity> {
        Arc::new(ContextProviderIdentity {
            id: "classical-zero-fixture".into(),
            version: "1".into(),
            manifest_sha256: [3; 32],
            checkpoint_sha256: None,
            training_run_id: None,
            runtime: "native-test".into(),
            source_revision: "workflow-test".into(),
        })
    }
    fn supports(&self, _: &ContextPressureRequest<'_>) -> Result<(), ProviderRejection> {
        if self.reject_support {
            Err(ProviderRejection::Unsupported("fixture support refusal"))
        } else {
            Ok(())
        }
    }
    fn predict(
        &mut self,
        _: &ContextPressureRequest<'_>,
        output: &mut [f64],
        _: &mut dyn FnMut() -> bool,
    ) -> Result<(), ProviderRejection> {
        output.fill(if self.invalid_output { f64::NAN } else { 0.0 });
        Ok(())
    }
}

#[test]
fn observed_candidate_fallback_and_applied_zero_are_real_owner_facts() {
    for (reject_support, invalid_output) in [(false, false), (true, false), (false, true)] {
        let (mut session, mut workspace) = fixture(3, 10);
        let mut viscosity =
            ViscosityWorkspace::new(session.snapshot().grid().clone(), CAP).unwrap();
        let mut provider = FixtureProvider {
            reject_support,
            invalid_output,
        };
        let mut evidence = None;
        let (_, report) = session
            .step_with_configured_pressure_observed(
                inputs(&[], 0.03125),
                &mut workspace,
                Some((&mut viscosity, 0.0)),
                Some(&mut provider),
                None,
                |_| false,
                7,
                &mut evidence,
            )
            .unwrap();
        let evidence = evidence.unwrap();
        evidence.validate().unwrap();
        assert_eq!(evidence.run_id, 10);
        assert_eq!(evidence.attempt_id, 7);
        assert_eq!(evidence.accepted_generation, 0);
        assert_eq!(evidence.publication, Some(1));
        assert_eq!(evidence.descriptor.viscosity, Some(0.0));
        let p = evidence.pressure.as_ref().unwrap();
        assert_eq!(p.binding.as_ref().unwrap().id, "classical-zero-fixture");
        assert_eq!(p.support, Some(!reject_support));
        assert_eq!(p.prediction_called, !reject_support);
        assert_eq!(p.rejection.is_some(), reject_support || invalid_output);
        assert_eq!(p.selected, report.pressure.selected);
        assert!(p.rhs_sha256.is_some());
        assert_eq!(
            evidence
                .nodes
                .iter()
                .any(|n| n.node == NodeKind::NativeFallback),
            reject_support || invalid_output
        );
    }
}

#[test]
fn cancellation_before_provider_after_return_and_before_commit_never_publishes() {
    let stops = [
        LiquidStepStage::Carrier(StepStage::BeforeAdvection),
        LiquidStepStage::Carrier(StepStage::BeforeProjectionAcceptance),
        LiquidStepStage::BeforeCommit,
    ];
    for (index, stop) in stops.into_iter().enumerate() {
        let (mut session, mut workspace) = fixture(3, 11);
        let before = input_digest(session.snapshot().state());
        let mut provider = FixtureProvider {
            reject_support: false,
            invalid_output: false,
        };
        let mut evidence = None;
        assert!(
            session
                .step_with_configured_pressure_observed(
                    inputs(&[], 0.02),
                    &mut workspace,
                    None,
                    Some(&mut provider),
                    None,
                    |stage| stage == stop,
                    index as u64 + 1,
                    &mut evidence
                )
                .is_err()
        );
        let evidence = evidence.unwrap();
        evidence.validate().unwrap();
        assert_eq!(evidence.outcome, AttemptOutcome::Cancelled);
        assert_eq!(evidence.publication, None);
        assert_eq!(evidence.accepted_generation, 0);
        assert_eq!(evidence.attempted_generation, 1);
        assert_eq!(evidence.input_sha256, before);
        assert_eq!(input_digest(session.snapshot().state()), before);
        assert!(
            !evidence
                .nodes
                .iter()
                .any(|n| n.node == NodeKind::Publication)
        );
        if index == 0 {
            assert!(evidence.pressure.is_none());
        } else {
            assert!(evidence.pressure.as_ref().unwrap().accepted.is_some());
        }
    }
}

#[test]
fn actual_shortened_interval_and_native_trajectory_match_existing_pressure_path() {
    let (mut observed, mut ow) = fixture_fraction(4, 12, 0.5);
    let (mut baseline, mut bw) = fixture_fraction(4, 13, 0.5);
    let forces = [BodyForce {
        value: [2.0, 0.0, 0.0],
        units: ForceUnits::Acceleration,
        region: None,
    }];
    let mut descriptor = None;
    for index in 1..=4 {
        let mut evidence = None;
        let (report, _) = observed
            .step_with_configured_pressure_observed(
                inputs(&forces, 1.0),
                &mut ow,
                None,
                None,
                None,
                |_| false,
                index,
                &mut evidence,
            )
            .unwrap();
        baseline
            .step_with_pressure_provider(inputs(&forces, 1.0), &mut bw, None, None, |_| false)
            .unwrap();
        let evidence = evidence.unwrap();
        evidence.validate().unwrap();
        assert!(report.carrier.step.dt < 1.0);
        assert_eq!(
            evidence.actual_dt.unwrap().to_bits(),
            report.carrier.step.dt.to_bits()
        );
        assert_eq!(
            input_digest(observed.snapshot().state()),
            input_digest(baseline.snapshot().state())
        );
        if let Some(previous) = descriptor {
            assert_eq!(evidence.descriptor_sha256, previous);
        }
        descriptor = Some(evidence.descriptor_sha256);
    }
}

#[test]
fn descriptor_digest_changes_only_with_actual_composition_and_invalid_binding_is_rejected() {
    let (mut session, mut workspace) = fixture(3, 14);
    let mut evidence = None;
    session
        .step_with_configured_pressure_observed(
            inputs(&[], 0.01),
            &mut workspace,
            None,
            None,
            None,
            |_| false,
            1,
            &mut evidence,
        )
        .unwrap();
    let evidence = evidence.unwrap();
    let mut changed = (*evidence).clone();
    changed.descriptor.viscosity = Some(0.0);
    assert_ne!(changed.descriptor.digest(), evidence.descriptor_sha256);
    assert_eq!(changed.validate(), Err("workflow descriptor digest"));
    let mut changed = (*evidence).clone();
    changed.publication = Some(2);
    assert_eq!(changed.validate(), Err("workflow publication association"));
    let mut changed = (*evidence).clone();
    changed.nodes.resize(
        MAX_NODES + 1,
        NodeReceipt {
            node: NodeKind::Geometry,
            status: NodeStatus::Completed,
        },
    );
    assert_eq!(changed.validate(), Err("workflow attempt association"));
}

#[test]
fn in_stage_cancellation_and_invalid_attempt_id_preserve_exact_terminal_facts() {
    let (mut session, mut workspace) = fixture(3, 21);
    let mut evidence = None;
    let result = session.step_with_configured_pressure_observed(
        inputs(&[], 0.01),
        &mut workspace,
        None,
        None,
        None,
        |stage| stage == LiquidStepStage::Carrier(StepStage::PressureIteration),
        1,
        &mut evidence,
    );
    assert!(matches!(
        result,
        Err(SessionError::Step(LiquidStepError::Carrier(
            SimulationError::Pressure(PressureError::Cancelled { .. })
        )))
    ));
    let trace = evidence.unwrap();
    trace.validate().unwrap();
    assert_eq!(trace.outcome, AttemptOutcome::Cancelled);
    assert!(trace.pressure.as_ref().unwrap().rhs_sha256.is_some());
    let mut viscosity = ViscosityWorkspace::new(session.snapshot().grid().clone(), CAP).unwrap();
    let mut evidence = None;
    assert!(
        session
            .step_with_configured_pressure_observed(
                inputs(&[], 0.01),
                &mut workspace,
                Some((&mut viscosity, 0.001002)),
                None,
                None,
                |stage| matches!(stage, LiquidStepStage::Carrier(StepStage::Viscosity(_))),
                2,
                &mut evidence
            )
            .is_err()
    );
    let trace = evidence.unwrap();
    trace.validate().unwrap();
    assert_eq!(trace.outcome, AttemptOutcome::Cancelled);
    let mut evidence = None;
    assert!(
        session
            .step_with_configured_pressure_observed(
                inputs(&[], 0.01),
                &mut workspace,
                None,
                None,
                None,
                |_| false,
                0,
                &mut evidence
            )
            .is_err()
    );
    assert!(evidence.is_none());
    assert_eq!(session.snapshot().state().carrier.generation, 0);
}

#[test]
fn dispatch_cancellation_retains_candidate_return_qualification_and_fallback_start_separately() {
    for stop in [2, 3, 4] {
        let (mut session, mut workspace) = fixture(3, 22);
        let mut provider = FixtureProvider {
            reject_support: false,
            invalid_output: true,
        };
        let mut calls = 0;
        let mut evidence = None;
        assert!(
            session
                .step_with_configured_pressure_observed(
                    inputs(&[], 0.01),
                    &mut workspace,
                    None,
                    Some(&mut provider),
                    None,
                    |stage| {
                        if stage == LiquidStepStage::Carrier(StepStage::PressureIteration) {
                            calls += 1;
                            calls == stop
                        } else {
                            false
                        }
                    },
                    1,
                    &mut evidence
                )
                .is_err()
        );
        let trace = evidence.unwrap();
        trace.validate().unwrap();
        assert_eq!(trace.outcome, AttemptOutcome::Cancelled);
        assert!(
            trace
                .nodes
                .iter()
                .any(|n| n.node == NodeKind::PressureCandidate
                    && n.status == NodeStatus::CandidateReturned)
        );
        assert_eq!(
            trace
                .nodes
                .iter()
                .any(|n| n.node == NodeKind::PressureQualification),
            stop >= 3
        );
        assert_eq!(
            trace
                .nodes
                .iter()
                .any(|n| n.node == NodeKind::NativeFallback && n.status == NodeStatus::Started),
            stop >= 4
        );
        assert!(
            !trace
                .nodes
                .iter()
                .any(|n| n.node == NodeKind::NativeFallback && n.status == NodeStatus::Selected)
        );
        assert!(trace.publication.is_none());
    }
}

#[test]
fn corrupted_prefixes_and_request_controls_are_detected() {
    let (mut session, mut workspace) = fixture(3, 23);
    let mut evidence = None;
    session
        .step_with_configured_pressure_observed(
            inputs(&[], 0.01),
            &mut workspace,
            None,
            None,
            None,
            |_| false,
            1,
            &mut evidence,
        )
        .unwrap();
    let trace = evidence.unwrap();
    let mut forged = (*trace).clone();
    forged.nodes = vec![NodeReceipt {
        node: NodeKind::Publication,
        status: NodeStatus::Accepted,
    }];
    assert!(forged.validate().is_err());
    let mut forged = (*trace).clone();
    forged.nodes.swap(2, 3);
    assert_eq!(forged.validate(), Err("workflow execution ordering"));
    let mut forged = (*trace).clone();
    forged.pressure = None;
    assert_eq!(
        forged.validate(),
        Err("workflow pressure request association")
    );
    let mut forged = (*trace).clone();
    forged.outcome = AttemptOutcome::Cancelled;
    forged.publication = None;
    forged.actual_divergence_max = None;
    forged.nodes.truncate(
        forged
            .nodes
            .iter()
            .position(|n| n.node == NodeKind::PressureRequest)
            .unwrap()
            + 1,
    );
    assert_eq!(
        forged.validate(),
        Err("workflow pressure diagnostic stage association")
    );
    assert!(trace.matches_request(inputs(&[], 0.01)));
    let forces = [BodyForce {
        value: [1.0, 0.0, 0.0],
        units: ForceUnits::Acceleration,
        region: None,
    }];
    assert!(!trace.matches_request(inputs(&forces, 0.01)));
    let rates = [0.0; 27];
    let mut source = inputs(&[], 0.01);
    source.source =
        Some(LiquidVolumeSource::new(VolumeStamp { id: 7, version: 0 }, &rates).unwrap());
    assert_ne!(request_digest(source), trace.request_sha256);
    let mut changed = inputs(&[], 0.01);
    changed.volume.actual_divergence_limit = 1e-6;
    assert!(!trace.matches_request(changed));
    let edges = trace.descriptor.edges();
    assert!(edges.iter().any(|e| e.from == NodeKind::Projection
        && e.to == NodeKind::VolumeTransport
        && e.port == PortKind::FaceVelocity));
    assert!(
        !edges
            .iter()
            .any(|e| e.from == NodeKind::TracerTransport && e.to == NodeKind::VolumeTransport)
    );
}
