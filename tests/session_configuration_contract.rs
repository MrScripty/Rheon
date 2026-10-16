//! Constructor readback is actual immutable Session configuration. Per-call
//! viscosity, provider choice, requested dt and future schedules stay separate.
use rheon::*;

const MEMORY: usize = 1024 * 1024;
const BOUNDARY_MEMORY: usize = 2 * 1024 * 1024;
const MAX_STEPS: usize = 3;

fn grid() -> GridGeometry {
    GridGeometry::new([2, 1, 1], [0.25; 3], [0.0; 3]).unwrap()
}
fn controls() -> LiquidTransportConfig {
    LiquidTransportConfig {
        carrier: SimulationConfig {
            density: 2.0,
            memory_limit: MEMORY,
            pressure: PressureSettings {
                relative_residual: 1e-11,
                absolute_residual: 1e-13,
                divergence_limit: 1e-8,
                max_iterations: 1234,
            },
            actual_divergence_limit: 1e-6,
            max_courant: 0.75,
        },
        represented_density: 2.0,
        volume_stamp: VolumeStamp {
            id: 123,
            version: 17,
        },
        carrier_id: 456,
    }
}
fn session(method: PressureImplementation, config: LiquidTransportConfig) -> SimulationSession {
    SimulationSession::new(
        SessionIdentity {
            run_id: 789,
            source_revision: "constructor-readback-fixture".into(),
        },
        grid(),
        config,
        method,
        vec![1.0; 2],
        BOUNDARY_MEMORY,
        MAX_STEPS,
    )
    .unwrap()
}
fn assert_controls(
    actual: &SessionConfiguration,
    expected: LiquidTransportConfig,
    method: PressureImplementation,
) {
    let (a, e) = (actual.transport, expected);
    assert_eq!(a.carrier.density.to_bits(), e.carrier.density.to_bits());
    assert_eq!(a.carrier.memory_limit, e.carrier.memory_limit);
    assert_eq!(
        a.carrier.pressure.relative_residual.to_bits(),
        e.carrier.pressure.relative_residual.to_bits()
    );
    assert_eq!(
        a.carrier.pressure.absolute_residual.to_bits(),
        e.carrier.pressure.absolute_residual.to_bits()
    );
    assert_eq!(
        a.carrier.pressure.divergence_limit.to_bits(),
        e.carrier.pressure.divergence_limit.to_bits()
    );
    assert_eq!(
        a.carrier.pressure.max_iterations,
        e.carrier.pressure.max_iterations
    );
    assert_eq!(
        a.carrier.actual_divergence_limit.to_bits(),
        e.carrier.actual_divergence_limit.to_bits()
    );
    assert_eq!(
        a.carrier.max_courant.to_bits(),
        e.carrier.max_courant.to_bits()
    );
    assert_eq!(
        a.represented_density.to_bits(),
        e.represented_density.to_bits()
    );
    assert_eq!(a.volume_stamp, e.volume_stamp);
    assert_eq!(a.carrier_id, e.carrier_id);
    assert_eq!(actual.implementation, method);
    assert_eq!(actual.boundary_workspace_limit, BOUNDARY_MEMORY);
    assert_eq!(actual.max_steps, MAX_STEPS);
    assert_ne!(actual.boundary_workspace_limit, a.carrier.memory_limit);
}
fn publication(s: &SimulationSession) -> Vec<u8> {
    let mut bytes = Vec::new();
    s.snapshot().write_dense3d(&mut bytes).unwrap();
    bytes
}
fn inputs() -> LiquidStepInputs<'static> {
    LiquidStepInputs {
        requested_dt: 1.0 / 16.0,
        smoke_source: None,
        forces: &[],
        inlet: LiquidInlet::new(
            VolumeStamp {
                id: 999,
                version: 0,
            },
            [[0.0; 2]; 3],
        )
        .unwrap(),
        source: None,
        volume: LiquidVolumeSettings {
            max_outward_courant: 0.5,
            actual_divergence_limit: 1e-7,
        },
    }
}

#[test]
fn constructor_readback_preserves_every_control_and_mutable_copy_is_detached() {
    for method in [
        PressureImplementation::JacobiPcgV1,
        PressureImplementation::SymmetricGaussSeidelPcgV1,
    ] {
        // Independent represented density is legal at construction. A future
        // viscous call must separately enforce its equality requirement.
        let mut expected = controls();
        expected.represented_density = 3.0;
        let s = session(method, expected);
        assert_eq!(s.snapshot().stage(), SnapshotStage::Constructor);
        assert_eq!(s.snapshot().frame(), 0);
        assert_eq!(s.snapshot().grid(), &grid());
        assert_controls(s.snapshot().configuration(), expected, method);
        let before = publication(&s);

        let mut detached = *s.snapshot().configuration();
        detached.transport.carrier.density = 1000.0;
        detached.transport.carrier.memory_limit = 1;
        detached.transport.carrier.pressure.relative_residual = 0.1;
        detached.transport.carrier.pressure.absolute_residual = 0.2;
        detached.transport.carrier.pressure.divergence_limit = 0.3;
        detached.transport.carrier.pressure.max_iterations = 1;
        detached.transport.carrier.actual_divergence_limit = 0.4;
        detached.transport.carrier.max_courant = 0.5;
        detached.transport.represented_density = 999.0;
        detached.transport.volume_stamp = VolumeStamp { id: 0, version: 0 };
        detached.transport.carrier_id = 0;
        detached.implementation = match method {
            PressureImplementation::JacobiPcgV1 => {
                PressureImplementation::SymmetricGaussSeidelPcgV1
            }
            PressureImplementation::SymmetricGaussSeidelPcgV1 => {
                PressureImplementation::JacobiPcgV1
            }
        };
        detached.boundary_workspace_limit = 2;
        detached.max_steps = 0;
        assert_eq!(detached.transport.carrier.density, 1000.0);
        assert_ne!(detached.implementation, method);
        assert_eq!(detached.max_steps, 0);
        assert_controls(s.snapshot().configuration(), expected, method);
        assert_eq!(publication(&s), before);
    }
}

#[test]
fn constructor_configuration_stays_stable_across_accepted_failed_cancelled_and_stopped_calls() {
    for method in [
        PressureImplementation::JacobiPcgV1,
        PressureImplementation::SymmetricGaussSeidelPcgV1,
    ] {
        let expected = controls();
        let mut s = session(method, expected);
        let mut pressure = ProviderStepWorkspace::new(&grid(), 2.0, MEMORY, method).unwrap();
        let mut viscosity = ViscosityWorkspace::new(grid(), MEMORY).unwrap();
        let mu = 1.0 / 128.0;
        let (step, receipt) = s
            .step_with_configured_pressure(
                inputs(),
                &mut pressure,
                Some((&mut viscosity, mu)),
                None,
                None,
                |_| false,
            )
            .unwrap();
        assert_eq!(receipt.pressure.selected, method.id());
        assert_eq!(receipt.dynamic_viscosity, mu);
        assert_eq!(step.viscosity.unwrap().dynamic_viscosity, mu);
        assert_eq!(step.carrier.step.dt, inputs().requested_dt);
        assert_eq!(s.snapshot().frame(), 1);
        assert_eq!(s.snapshot().stage(), SnapshotStage::Accepted);
        assert_eq!(
            s.snapshot().state().liquid.stamp.version,
            expected.volume_stamp.version + 1
        );
        assert_controls(s.snapshot().configuration(), expected, method);
        let before = publication(&s);

        assert!(
            s.step_with_configured_pressure(
                inputs(),
                &mut pressure,
                Some((&mut viscosity, -1.0)),
                None,
                None,
                |_| false,
            )
            .is_err()
        );
        assert_eq!(publication(&s), before);
        assert_controls(s.snapshot().configuration(), expected, method);
        assert!(
            s.step_with_configured_pressure(
                inputs(),
                &mut pressure,
                Some((&mut viscosity, 2.0 * mu)),
                None,
                None,
                |stage| stage == LiquidStepStage::BeforeCommit,
            )
            .is_err()
        );
        assert_eq!(publication(&s), before);
        assert_controls(s.snapshot().configuration(), expected, method);
        assert_eq!(
            s.snapshot()
                .configured_provider_report()
                .unwrap()
                .dynamic_viscosity,
            mu
        );

        s.stop();
        assert!(s.is_stopped());
        assert!(matches!(
            s.step_with_configured_pressure(inputs(), &mut pressure, None, None, None, |_| false),
            Err(SessionError::Stopped)
        ));
        assert_eq!(publication(&s), before);
        assert_controls(s.snapshot().configuration(), expected, method);
    }
}
