//! Accepted interval start comes from the actual pre-step owner clock, never
//! reconstructed by subtracting dt from a rounded accumulated endpoint.
use rheon::*;

const CAP: usize = 1024 * 1024;
const DT: f64 = 1.0 / 60.0;
const MU: f64 = 1.0 / 128.0;
const QUOTA: usize = 65;

#[derive(Clone, Copy, Debug)]
enum Path {
    Configured,
    LegacyProvider,
    SealedBoundary,
    #[cfg(feature = "accepted-pressure-warm-start")]
    Warm,
}
fn grid() -> GridGeometry {
    GridGeometry::new([2, 1, 1], [0.25; 3], [0.0; 3]).unwrap()
}
fn session(method: PressureImplementation) -> SimulationSession {
    SimulationSession::new(
        SessionIdentity {
            run_id: 87,
            source_revision: "actual-start-clock-contract".into(),
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
            volume_stamp: VolumeStamp { id: 88, version: 0 },
            carrier_id: 89,
        },
        method,
        vec![1.0; 2],
        CAP,
        QUOTA,
    )
    .unwrap()
}
fn inputs() -> LiquidStepInputs<'static> {
    LiquidStepInputs {
        requested_dt: DT,
        smoke_source: None,
        forces: &[],
        inlet: LiquidInlet::new(VolumeStamp { id: 90, version: 0 }, [[0.0; 2]; 3]).unwrap(),
        source: None,
        volume: LiquidVolumeSettings {
            max_outward_courant: 1.0,
            actual_divergence_limit: 1e-5,
        },
    }
}
fn step(
    path: Path,
    owner: &mut SimulationSession,
    pressure: &mut ProviderStepWorkspace,
    viscosity: &mut ViscosityWorkspace,
    cancel_final: bool,
) -> Result<LiquidStepReport, SessionError> {
    let cancel = |stage| cancel_final && stage == LiquidStepStage::BeforeCommit;
    match path {
        #[cfg(feature = "accepted-pressure-warm-start")]
        Path::Warm => owner.step_with_initialization(
            inputs(),
            PressureInitializationPolicy::AcceptedPreviousV1,
            None,
            cancel,
        ),
        Path::Configured => owner
            .step_with_configured_pressure(
                inputs(),
                pressure,
                Some((viscosity, MU)),
                None,
                None,
                cancel,
            )
            .map(|(step, _)| step),
        Path::LegacyProvider => owner
            .step_with_pressure_provider(inputs(), pressure, None, None, cancel)
            .map(|(step, _)| step),
        Path::SealedBoundary => owner.step(
            inputs(),
            BoxFluxStepBoundary {
                flux: PrescribedBoxFlux::new(BoxFluxStamp { id: 91, version: 0 }, [[0.0; 2]; 3])
                    .unwrap(),
                tracer: BoxFluxTracerPolicy::ClampedAppearance,
            },
            None,
            cancel,
        ),
    }
}
fn publication(owner: &SimulationSession) -> Vec<u8> {
    let mut bytes = Vec::new();
    owner.snapshot().write_dense3d(&mut bytes).unwrap();
    bytes
}
fn receipt_addresses(owner: &SimulationSession) -> [Option<usize>; 3] {
    let snapshot = owner.snapshot();
    [
        snapshot.report().map(|r| std::ptr::from_ref(r) as usize),
        snapshot
            .provider_report()
            .map(|r| std::ptr::from_ref(r) as usize),
        snapshot
            .configured_provider_report()
            .map(|r| std::ptr::from_ref(r) as usize),
    ]
}

fn configured_receipt_facts(owner: &SimulationSession) -> Option<(u64, bool, std::time::Duration)> {
    owner.snapshot().configured_provider_report().map(|r| {
        (
            r.dynamic_viscosity.to_bits(),
            r.prediction_called,
            r.pressure.total,
        )
    })
}

#[test]
fn actual_nonbinary_interval_starts_survive_all_session_paths_and_failed_attempts() {
    for method in [
        PressureImplementation::JacobiPcgV1,
        PressureImplementation::SymmetricGaussSeidelPcgV1,
    ] {
        for path in [
            Path::Configured,
            Path::LegacyProvider,
            Path::SealedBoundary,
            #[cfg(feature = "accepted-pressure-warm-start")]
            Path::Warm,
        ] {
            let mut owner = session(method);
            let mut pressure = ProviderStepWorkspace::new(&grid(), 2.0, CAP, method).unwrap();
            let mut viscosity = ViscosityWorkspace::new(grid(), CAP).unwrap();
            assert_eq!(owner.snapshot().accepted_interval_start(), None);
            let mut subtraction_counterexample = false;
            for generation in 1..=64 {
                let actual_pre_time = owner.snapshot().state().carrier.time;
                let report = step(path, &mut owner, &mut pressure, &mut viscosity, false).unwrap();
                let snapshot = owner.snapshot();
                assert_eq!(snapshot.state().carrier.generation, generation);
                assert_eq!(report.carrier.step.dt.to_bits(), DT.to_bits());
                assert_eq!(
                    snapshot.accepted_interval_start().map(f64::to_bits),
                    Some(actual_pre_time.to_bits()),
                    "{method:?} / {path:?} / {generation}"
                );
                subtraction_counterexample |= (report.carrier.step.time - report.carrier.step.dt)
                    .to_bits()
                    != actual_pre_time.to_bits();
            }
            assert!(
                subtraction_counterexample,
                "fixture must distinguish captured clock from end-dt: {method:?} / {path:?}"
            );

            let before = publication(&owner);
            let accepted_start_bits = owner.snapshot().accepted_interval_start().map(f64::to_bits);
            let next_actual_pre_time = owner.snapshot().state().carrier.time;
            let receipt_addresses_before = receipt_addresses(&owner);
            let provider_duration = owner.snapshot().provider_report().map(|r| r.total);
            let configured_facts = configured_receipt_facts(&owner);
            assert!(step(path, &mut owner, &mut pressure, &mut viscosity, true).is_err());
            assert_eq!(publication(&owner), before);
            assert_eq!(owner.snapshot().frame(), 64);
            assert_eq!(
                owner.snapshot().accepted_interval_start().map(f64::to_bits),
                accepted_start_bits
            );
            assert_eq!(receipt_addresses(&owner), receipt_addresses_before);
            assert_eq!(
                owner.snapshot().provider_report().map(|r| r.total),
                provider_duration
            );
            assert_eq!(configured_receipt_facts(&owner), configured_facts);

            assert!(
                owner
                    .step_with_configured_pressure(
                        inputs(),
                        &mut pressure,
                        Some((&mut viscosity, -1.0)),
                        None,
                        None,
                        |_| false
                    )
                    .is_err()
            );
            assert_eq!(publication(&owner), before);
            assert_eq!(
                owner.snapshot().accepted_interval_start().map(f64::to_bits),
                accepted_start_bits
            );
            assert_eq!(receipt_addresses(&owner), receipt_addresses_before);
            assert_eq!(
                owner.snapshot().provider_report().map(|r| r.total),
                provider_duration
            );
            assert_eq!(configured_receipt_facts(&owner), configured_facts);
            assert_eq!(
                owner.snapshot().state().carrier.time.to_bits(),
                next_actual_pre_time.to_bits()
            );

            step(path, &mut owner, &mut pressure, &mut viscosity, false).unwrap();
            assert_eq!(owner.snapshot().frame(), QUOTA);
            assert_eq!(owner.snapshot().state().carrier.generation, QUOTA as u64);
            assert_eq!(
                owner.snapshot().accepted_interval_start().map(f64::to_bits),
                Some(next_actual_pre_time.to_bits())
            );
            let final_publication = publication(&owner);
            let final_start = owner.snapshot().accepted_interval_start().map(f64::to_bits);
            assert!(matches!(
                step(path, &mut owner, &mut pressure, &mut viscosity, false),
                Err(SessionError::StepLimit)
            ));
            assert_eq!(publication(&owner), final_publication);
            assert_eq!(
                owner.snapshot().accepted_interval_start().map(f64::to_bits),
                final_start
            );
            owner.stop();
            assert!(matches!(
                step(path, &mut owner, &mut pressure, &mut viscosity, false),
                Err(SessionError::Stopped)
            ));
            assert_eq!(publication(&owner), final_publication);
            assert_eq!(
                owner.snapshot().accepted_interval_start().map(f64::to_bits),
                final_start
            );
        }
    }
}
