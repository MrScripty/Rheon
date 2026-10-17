//! Public consumer: real native prefix, typed same-request rejection, no fallback.
#![cfg(all(feature = "diagnostic", not(target_arch = "wasm32")))]
use rheon::diagnostic::{AdvanceStop, DiagnosticSession, ExecutionPolicy, Scene};
use rheon::recording::diagnostic::{HistoryTransition, PredictionOutcome};
use std::sync::atomic::AtomicBool;

#[test]
fn public_session_retains_raw_first_prediction_after_fifteen_actual_native_steps() {
    let mut session = DiagnosticSession::with_policy(
        Scene {
            counts: [5; 3],
            steps: 16,
            ..Default::default()
        },
        ExecutionPolicy::IndependentStopOnReject,
        "public-diagnostic-consumer".into(),
    )
    .unwrap();
    let flag = AtomicBool::new(false);
    for step in 1..=15 {
        let result = session
            .advance_typed(&flag, |_| panic!("early dispatch"))
            .unwrap();
        assert!(result.attempt.is_none());
        let frame = result.frame.unwrap();
        assert_eq!(frame.step, step);
        assert!(frame.reference_quality.unwrap().qualified);
        assert!(frame.ai_quality.is_none());
    }
    let prior = session.initial();
    let mut calls = 0;
    let result = session
        .advance_typed(&flag, |input| {
            calls += 1;
            assert_eq!(input.context.generation(), 16);
            assert_eq!(input.context.prior().unwrap().generation, 15);
            assert_eq!(input.initial_guess, prior.ai_pressure);
            let ports = serde_json::from_value(serde_json::json!({
            "base": "previous_pressure", "residual": "initial_residual",
            "rho": "density", "dt": "dt",
        })).unwrap();
        let request = input.adapter_request(&ports).unwrap();
        assert_eq!(request["context"]["generation"], 16);
        assert_eq!(request["context"]["prior"]["generation"], 15);
        assert_eq!(request["context"]["interval_end_s"], 16.0 * rheon::diagnostic::DT);
        assert_eq!(request["inputs"]["base"]["values"], serde_json::json!(input.initial_guess));
        assert_eq!(request["inputs"]["residual"]["values"], serde_json::json!(input.residual));
        let error = input.adapter_reply(
            b"{\"kind\":\"error\",\"code\":\"unavailable\",\"id\":16,\"message\":\"fixture unavailable\"}\n",
            rheon::diagnostic::OutputRole::FullPressure).unwrap_err();
        assert!(error.dispatched);
        assert_eq!(error.reason, rheon::recording::diagnostic::NoCandidateReason::RuntimeFailure);
        let mut reply = serde_json::to_vec(&serde_json::json!({
            "kind": "candidate", "role": "full_pressure", "id": 16,
            "pressure": vec![0.0; input.context.grid().cell_len()],
            "device": "cpu", "optimizer_updates": 0,
        })).unwrap();
        reply.push(b'\n');
        input.adapter_reply(&reply, rheon::diagnostic::OutputRole::FullPressure)
        })
        .unwrap();
    assert_eq!(calls, 1);
    assert_eq!(result.stop, Some(AdvanceStop::Rejected));
    assert!(result.frame.is_none());
    let attempt = result.attempt.unwrap();
    assert!(attempt.dispatched);
    assert_eq!(attempt.transition, HistoryTransition::NotAdvanced);
    assert!(
        matches!(attempt.outcome, PredictionOutcome::Candidate(sample)
        if !sample.observation.qualified)
    );
    let native = attempt.native_reference.unwrap();
    assert_eq!(native.request_digest, attempt.request.digest());
    assert!(native.sample.observation.qualified);
    let after = session.initial();
    assert_eq!(after.step, 15);
    assert_eq!(after.ai_pressure, prior.ai_pressure);
    assert_eq!(after.reference_pressure, prior.reference_pressure);
}
