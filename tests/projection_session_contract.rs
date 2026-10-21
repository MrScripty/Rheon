//! Actual integrated native projection substitution, not independent array calls.
use rheon::{projection_step::*, *};
use std::sync::Arc;

const CAP: usize = 16 * 1024 * 1024;
const FORCE: [BodyForce; 1] = [BodyForce {
    value: [0.001, 0.0, 0.0],
    units: ForceUnits::Acceleration,
    region: None,
}];

fn config() -> LiquidTransportConfig {
    LiquidTransportConfig {
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
    }
}

fn identity() -> SessionIdentity {
    SessionIdentity {
        run_id: 101,
        source_revision: "c1b2-frozen-integrated-reference".into(),
    }
}

fn inputs() -> LiquidStepInputs<'static> {
    LiquidStepInputs {
        requested_dt: 0.001,
        smoke_source: None,
        forces: &FORCE,
        inlet: LiquidInlet::new(VolumeStamp { id: 3, version: 0 }, [[0.0; 2]; 3]).unwrap(),
        source: None,
        volume: LiquidVolumeSettings {
            max_outward_courant: 1.0,
            actual_divergence_limit: 1e-5,
        },
    }
}

fn f32_bits(values: &[f32]) -> Vec<u32> {
    values.iter().map(|v| v.to_bits()).collect()
}
fn f64_bits(values: &[f64]) -> Vec<u64> {
    values.iter().map(|v| v.to_bits()).collect()
}
fn assert_native_equal(a: &SessionSnapshot<'_>, b: &SessionSnapshot<'_>) {
    let x = a.state();
    let y = b.state();
    assert_eq!(a.identity(), b.identity());
    assert_eq!(a.frame(), b.frame());
    assert_eq!(a.stage(), b.stage());
    assert_eq!(
        a.accepted_interval_start().map(f64::to_bits),
        b.accepted_interval_start().map(f64::to_bits)
    );
    for (left, right) in [
        (x.carrier.x, y.carrier.x),
        (x.carrier.y, y.carrier.y),
        (x.carrier.z, y.carrier.z),
        (x.carrier.tracer, y.carrier.tracer),
    ] {
        assert_eq!(f32_bits(left), f32_bits(right));
    }
    assert_eq!(f64_bits(x.pressure), f64_bits(y.pressure));
    assert_eq!(f64_bits(x.liquid.fraction), f64_bits(y.liquid.fraction));
    assert_eq!(x.carrier.time.to_bits(), y.carrier.time.to_bits());
    assert_eq!(x.carrier.generation, y.carrier.generation);
    assert_eq!(x.carrier_stamp, y.carrier_stamp);
    assert_eq!(x.liquid.time.to_bits(), y.liquid.time.to_bits());
    assert_eq!(x.liquid.stamp, y.liquid.stamp);
    assert_eq!(x.liquid.density.to_bits(), y.liquid.density.to_bits());
    assert!(x.pressure_surface.is_none() && y.pressure_surface.is_none());
    assert!(x.pressure_columns.is_none() && y.pressure_columns.is_none());
    assert!(x.reconstructed_surface.is_none() && y.reconstructed_surface.is_none());
    assert!(!x.flat_column_mac && !y.flat_column_mac);
}
fn diagnostics(report: PressureReport) -> (usize, [u64; 3]) {
    (
        report.iterations,
        [
            report.true_residual_l2.to_bits(),
            report.true_residual_max.to_bits(),
            report.predicted_divergence_max.to_bits(),
        ],
    )
}
fn assert_reports(a: LiquidStepReport, b: LiquidStepReport) {
    let x = a.carrier.step;
    let y = b.carrier.step;
    assert_eq!(x.generation, y.generation);
    assert_eq!(diagnostics(x.pressure), diagnostics(y.pressure));
    assert_eq!(
        [
            x.dt,
            x.time,
            x.actual_divergence_max,
            x.courant,
            x.tracer_integral,
            x.kinetic_energy
        ]
        .map(f64::to_bits),
        [
            y.dt,
            y.time,
            y.actual_divergence_max,
            y.courant,
            y.tracer_integral,
            y.kinetic_energy
        ]
        .map(f64::to_bits)
    );
    assert_eq!(a.carrier.forces, b.carrier.forces);
    // The native volume report has no wall-clock durations or workspace sizes.
    assert_eq!(format!("{:?}", a.liquid), format!("{:?}", b.liquid));
    assert!(a.boundary.is_none() && b.boundary.is_none());
    assert!(a.viscosity.is_none() && b.viscosity.is_none());
}

#[test]
fn c1b2_one_hundred_actual_intervals_match_the_original_session_for_each_method() {
    for method in PressureImplementation::ALL {
        compare_one_hundred(method, method, ProjectionIntent::BuiltinNativeV1);
    }
}

#[test]
fn c1b3_one_hundred_selected_intervals_match_direct_producer_for_each_pair() {
    for producer in PressureImplementation::ALL {
        let fallback = other_method(producer);
        compare_one_hundred(
            producer,
            fallback,
            ProjectionIntent::ClassicalSelectionV1 {
                producer,
                producer_max_iterations: 2000,
            },
        );
    }
}

fn other_method(method: PressureImplementation) -> PressureImplementation {
    PressureImplementation::ALL
        .into_iter()
        .find(|other| *other != method)
        .unwrap()
}

fn compare_one_hundred(
    method: PressureImplementation,
    fallback: PressureImplementation,
    intent: ProjectionIntent,
) {
    let grid = GridGeometry::new([16; 3], [1.0 / 16.0; 3], [0.0; 3]).unwrap();
    let mut direct = SimulationSession::new(
        identity(),
        grid.clone(),
        config(),
        method,
        vec![0.5; grid.cell_len()],
        CAP,
        100,
    )
    .unwrap();
    let mut substituted = SimulationSession::new_with_projection(
        identity(),
        grid.clone(),
        config(),
        fallback,
        vec![0.5; grid.cell_len()],
        CAP,
        100,
        intent,
    )
    .unwrap();
    let mut native = ProviderStepWorkspace::new(&grid, 998.2, CAP, method).unwrap();
    let mut projection = match intent {
        ProjectionIntent::BuiltinNativeV1 => {
            ProjectionStepWorkspace::new(&grid, 998.2, CAP, fallback)
        }
        _ => ProjectionStepWorkspace::new_with_intent(&grid, 998.2, CAP, fallback, intent),
    }
    .unwrap();
    assert!(projection.allocated_bytes() <= CAP);
    assert_native_equal(&direct.snapshot(), &substituted.snapshot());
    let mut previous_output = None;
    for generation in 1..=100u64 {
        let (expected, pressure) = direct
            .step_with_pressure_provider(inputs(), &mut native, None, None, |_| false)
            .unwrap();
        let accepted = substituted
            .step_with_native_projection(inputs(), &mut projection, None, |_| false)
            .unwrap();
        assert_eq!(
            accepted.report.provider_workspace_array_bytes,
            projection.allocated_bytes()
        );
        assert!(Arc::ptr_eq(
            &accepted.attempt,
            substituted.snapshot().projection_attempt().unwrap()
        ));
        assert_native_equal(&direct.snapshot(), &substituted.snapshot());
        assert_reports(expected, accepted.report);
        let attempt = &accepted.attempt;
        assert_eq!(
            substituted.snapshot().configuration().implementation,
            fallback
        );
        assert_eq!(attempt.intent, intent);
        assert_eq!(attempt.version, intent.version());
        assert!(attempt.producer_failure.is_none());
        assert_eq!(attempt.producer.as_ref().unwrap().implementation, method);
        assert_eq!(
            attempt.selected_native.as_ref().unwrap().implementation,
            method
        );
        assert_eq!(
            attempt.qualification.as_ref().unwrap().selection,
            rheon::solver_provider::composition::projection::ProjectionSelection::SuppliedCandidate
        );
        assert!(
            attempt
                .qualification
                .as_ref()
                .unwrap()
                .native_fallback
                .is_none()
        );
        attempt
            .validate(substituted.snapshot().configuration(), &grid)
            .unwrap();
        assert_eq!(attempt.source, identity());
        assert_eq!(attempt.attempt_id, generation);
        assert_eq!(attempt.accepted_generation, generation - 1);
        assert_eq!(attempt.attempted_generation, Some(generation));
        assert_eq!(attempt.requested_dt.to_bits(), 0.001f64.to_bits());
        assert_eq!(
            attempt.actual_dt.map(f64::to_bits),
            Some(expected.carrier.step.dt.to_bits())
        );
        assert!(attempt.provisional_input_id.is_some());
        assert!(attempt.qualification.is_some());
        assert_eq!(
            diagnostics(attempt.producer.as_ref().unwrap().pressure),
            diagnostics(pressure.accepted)
        );
        if let Some(prior) = previous_output {
            assert_eq!(attempt.input_sha256, prior);
        }
        previous_output = Some(attempt.output_sha256.unwrap());
    }
}

#[test]
fn c1b2_actual_shortened_interval_is_bound_and_matches_original_owner() {
    for method in PressureImplementation::ALL {
        compare_shortened(method, method, ProjectionIntent::BuiltinNativeV1);
    }
}

#[test]
fn c1b3_shortened_interval_binds_selected_producer_and_fallback() {
    for producer in PressureImplementation::ALL {
        compare_shortened(
            producer,
            other_method(producer),
            ProjectionIntent::ClassicalSelectionV1 {
                producer,
                producer_max_iterations: 3000,
            },
        );
    }
}

fn compare_shortened(
    method: PressureImplementation,
    fallback: PressureImplementation,
    intent: ProjectionIntent,
) {
    let grid = GridGeometry::new([5; 3], [0.25; 3], [0.0; 3]).unwrap();
    let mut cfg = config();
    cfg.carrier.density = 2.0;
    cfg.represented_density = 2.0;
    cfg.carrier.max_courant = 1.0 / 128.0;
    cfg.carrier.pressure = PressureSettings {
        relative_residual: 1e-14,
        absolute_residual: 1e-16,
        divergence_limit: 1e-7,
        max_iterations: 3000,
    };
    let cap = 4 * 1024 * 1024;
    cfg.carrier.memory_limit = cap;
    let mut direct = SimulationSession::new(
        identity(),
        grid.clone(),
        cfg,
        method,
        vec![1.0; grid.cell_len()],
        cap,
        2,
    )
    .unwrap();
    let mut substituted = SimulationSession::new_with_projection(
        identity(),
        grid.clone(),
        cfg,
        fallback,
        vec![1.0; grid.cell_len()],
        cap,
        2,
        intent,
    )
    .unwrap();
    let mut native = ProviderStepWorkspace::new(&grid, 2.0, cap, method).unwrap();
    let mut projection = match intent {
        ProjectionIntent::BuiltinNativeV1 => {
            ProjectionStepWorkspace::new(&grid, 2.0, cap, fallback)
        }
        _ => ProjectionStepWorkspace::new_with_intent(&grid, 2.0, cap, fallback, intent),
    }
    .unwrap();
    let forces = [BodyForce {
        value: [0.125, 0.0625, 0.0625],
        units: ForceUnits::Acceleration,
        region: None,
    }];
    let mut request = inputs();
    request.forces = &forces;
    request.requested_dt = 2.0;
    let (expected, _) = direct
        .step_with_pressure_provider(request, &mut native, None, None, |_| false)
        .unwrap();
    let accepted = substituted
        .step_with_native_projection(request, &mut projection, None, |_| false)
        .unwrap();
    assert!(accepted.report.carrier.step.dt < request.requested_dt);
    assert_eq!(
        accepted.attempt.actual_dt.map(f64::to_bits),
        Some(expected.carrier.step.dt.to_bits())
    );
    assert_eq!(accepted.attempt.intent, intent);
    assert_eq!(
        accepted.attempt.producer.as_ref().unwrap().implementation,
        method
    );
    assert!(accepted.attempt.producer_failure.is_none());
    assert_eq!(
        accepted
            .attempt
            .selected_native
            .as_ref()
            .unwrap()
            .implementation,
        method
    );
    assert_reports(expected, accepted.report);
    assert_native_equal(&direct.snapshot(), &substituted.snapshot());
}

#[test]
fn c1b2_legacy_entry_refusal_preserves_the_accepted_projection_receipt() {
    let grid = GridGeometry::new([2; 3], [0.5; 3], [0.0; 3]).unwrap();
    let method = PressureImplementation::JacobiPcgV1;
    let mut owner = SimulationSession::new_with_projection(
        identity(),
        grid.clone(),
        config(),
        method,
        vec![0.5; grid.cell_len()],
        CAP,
        4,
        ProjectionIntent::BuiltinNativeV1,
    )
    .unwrap();
    let mut projection = ProjectionStepWorkspace::new(&grid, 998.2, CAP, method).unwrap();
    let mut native = ProviderStepWorkspace::new(&grid, 998.2, CAP, method).unwrap();
    let first = owner
        .step_with_native_projection(inputs(), &mut projection, None, |_| false)
        .unwrap();
    let pressure_before = f64_bits(owner.snapshot().state().pressure);
    #[cfg(feature = "accepted-pressure-warm-start")]
    for policy in [
        PressureInitializationPolicy::Cold,
        PressureInitializationPolicy::AcceptedPreviousV1,
    ] {
        let mut called = false;
        assert!(matches!(
            owner.step_with_initialization(inputs(), policy, None, |_| {
                called = true;
                false
            }),
            Err(SessionError::PressureBinding(
                solver_provider::ProviderRejection::Unsupported(_)
            ))
        ));
        assert!(!called);
        assert!(Arc::ptr_eq(
            owner.snapshot().projection_attempt().unwrap(),
            &first.attempt
        ));
        assert_eq!(owner.snapshot().frame(), 1);
        assert_eq!(f64_bits(owner.snapshot().state().pressure), pressure_before);
    }
    assert!(
        owner
            .step_with_pressure_provider(inputs(), &mut native, None, None, |_| false)
            .is_err()
    );
    assert!(
        owner
            .step_with_configured_pressure(inputs(), &mut native, None, None, None, |_| false)
            .is_err()
    );
    let mut legacy = SimulationSession::new(
        identity(),
        grid.clone(),
        config(),
        method,
        vec![0.5; grid.cell_len()],
        CAP,
        1,
    )
    .unwrap();
    let mut evidence = None;
    legacy
        .step_with_configured_pressure_observed(
            inputs(),
            &mut native,
            None,
            None,
            None,
            |_| false,
            1,
            &mut evidence,
        )
        .unwrap();
    assert!(evidence.is_some());
    assert!(
        owner
            .step_with_configured_pressure_observed(
                inputs(),
                &mut native,
                None,
                None,
                None,
                |_| false,
                2,
                &mut evidence
            )
            .is_err()
    );
    assert!(
        evidence.is_none(),
        "a refused call must not expose an earlier G1 attempt"
    );
    assert_eq!(owner.snapshot().frame(), 1);
    assert_eq!(f64_bits(owner.snapshot().state().pressure), pressure_before);
    let second = owner
        .step_with_native_projection(inputs(), &mut projection, None, |_| false)
        .unwrap();
    assert_eq!(second.attempt.accepted_generation, 1);
    assert_eq!(second.attempt.attempt_id, first.attempt.attempt_id + 1);
    assert!(!Arc::ptr_eq(&first.attempt, &second.attempt));
}

#[test]
fn c1b3_real_iteration_failure_fallback_matches_direct_fallback_for_both_pairs() {
    use rheon::solver_provider::composition::projection::ProjectionSelection;
    for producer in PressureImplementation::ALL {
        let fallback = other_method(producer);
        let intent = ProjectionIntent::ClassicalSelectionV1 {
            producer,
            producer_max_iterations: 1,
        };
        let grid = GridGeometry::new([4; 3], [0.25; 3], [0.0; 3]).unwrap();
        let mut direct = SimulationSession::new(
            identity(),
            grid.clone(),
            config(),
            fallback,
            vec![0.5; grid.cell_len()],
            CAP,
            2,
        )
        .unwrap();
        let mut substituted = SimulationSession::new_with_projection(
            identity(),
            grid.clone(),
            config(),
            fallback,
            vec![0.5; grid.cell_len()],
            CAP,
            2,
            intent,
        )
        .unwrap();
        let mut native = ProviderStepWorkspace::new(&grid, 998.2, CAP, fallback).unwrap();
        let mut projection =
            ProjectionStepWorkspace::new_with_intent(&grid, 998.2, CAP, fallback, intent).unwrap();
        for generation in 1..=2 {
            let (expected, _) = direct
                .step_with_pressure_provider(inputs(), &mut native, None, None, |_| false)
                .unwrap();
            let accepted = substituted
                .step_with_native_projection(inputs(), &mut projection, None, |_| false)
                .unwrap();
            assert_eq!(accepted.attempt.attempted_generation, Some(generation));
            assert!(accepted.attempt.producer.is_none());
            let failure = accepted
                .attempt
                .producer_failure
                .as_ref()
                .expect("frozen cap1 fixture must fail producer");
            assert_eq!(failure.implementation, producer);
            assert!(matches!(
                failure.failure,
                ProjectionProducerFailure::IterationLimit { iterations: 1, .. }
            ));
            let qualification = accepted.attempt.qualification.as_ref().unwrap();
            assert_eq!(qualification.selection, ProjectionSelection::NativeBaseline);
            assert!(qualification.rejection.is_none() && qualification.witness.is_none());
            assert_eq!(
                qualification
                    .native_fallback
                    .as_ref()
                    .unwrap()
                    .implementation,
                fallback
            );
            assert_eq!(
                accepted
                    .attempt
                    .selected_native
                    .as_ref()
                    .unwrap()
                    .implementation,
                fallback
            );
            accepted
                .attempt
                .validate(substituted.snapshot().configuration(), &grid)
                .unwrap();
            assert_native_equal(&direct.snapshot(), &substituted.snapshot());
            assert_reports(expected, accepted.report);
        }
    }
}
