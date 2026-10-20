use super::*;
use crate::*;
use std::time::Instant;

const MAC_CAP: usize = 16 * 1024 * 1024;
const FORCE: [BodyForce; 1] = [BodyForce {
    value: [0.001, 0.0, 0.0],
    units: ForceUnits::Acceleration,
    region: None,
}];

fn identity() -> SessionIdentity {
    SessionIdentity {
        run_id: 101,
        source_revision: "c1a-frozen-native-reference".into(),
    }
}
fn expected(identity: &SessionIdentity, generation: u64) -> NativeInputExpectation<'_> {
    NativeInputExpectation {
        identity,
        generation,
    }
}
fn mac(max_steps: usize) -> (SimulationSession, ProviderStepWorkspace) {
    let grid = GridGeometry::new([16; 3], [1.0 / 16.0; 3], [0.0; 3]).unwrap();
    let owner = SimulationSession::new(
        identity(),
        grid.clone(),
        LiquidTransportConfig {
            carrier: SimulationConfig {
                density: 998.2,
                memory_limit: MAC_CAP,
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
        },
        PressureImplementation::JacobiPcgV1,
        vec![0.5; 16usize.pow(3)],
        MAC_CAP,
        max_steps,
    )
    .unwrap();
    let workspace =
        ProviderStepWorkspace::new(&grid, 998.2, MAC_CAP, PressureImplementation::JacobiPcgV1)
            .unwrap();
    (owner, workspace)
}
fn mac_inputs() -> LiquidStepInputs<'static> {
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
fn mac_request<'a>(
    identity: &'a SessionIdentity,
    generation: u64,
    workspace: &'a mut ProviderStepWorkspace,
) -> NativeStepRequest<'a> {
    NativeStepRequest::Mac(MacRequest {
        expected: expected(identity, generation),
        inputs: mac_inputs(),
        workspace,
        deadline: None,
    })
}
fn particle(max_steps: usize) -> MaterialSession {
    MaterialSession::new(
        identity(),
        GridGeometry::new([4; 3], [0.25; 3], [0.0; 3]).unwrap(),
        ParticleFluidConfig {
            samples_per_axis: 4,
            max_particles: 512,
            memory_limit: 1024 * 1024,
            fill: DomainFill::EmittedFluid,
            gravity: [0.0; 3],
            ..Default::default()
        },
        max_steps,
    )
    .unwrap()
}
fn particle_controls() -> MaterialStepControls {
    MaterialStepControls {
        requested_dt: 0.001,
        emitter: None,
    }
}
fn shortened_controls(generation: u64) -> MaterialStepControls {
    MaterialStepControls {
        requested_dt: 0.2,
        emitter: Some(MaterialEmitterControl {
            source_id: 9,
            source_version: generation,
            emitter: RotatingCylinderEmitter {
                pressure_heads: [0.0; 2],
                ..Default::default()
            },
        }),
    }
}
fn particle_request(
    identity: &SessionIdentity,
    generation: u64,
    controls: MaterialStepControls,
) -> NativeStepRequest<'_> {
    NativeStepRequest::ParticlePreview(ParticleRequest {
        expected: expected(identity, generation),
        controls,
        deadline: None,
    })
}

// Test-only fingerprints retain every accepted native field and publication fact.
// Native reports contain no elapsed times except ProviderReport; those times are
// excluded for cross-run equivalence, retained for before/after refusal checks.
fn mac_fields(snapshot: &SessionSnapshot<'_>) -> Vec<u64> {
    let state = snapshot.state();
    assert!(state.pressure_surface.is_none());
    assert!(state.reconstructed_surface.is_none());
    assert!(state.pressure_columns.is_none());
    assert!(!state.flat_column_mac);
    let mut bits = vec![
        state.carrier.generation,
        state.carrier.time.to_bits(),
        state.liquid.time.to_bits(),
        state.liquid.density.to_bits(),
        state.carrier_stamp.id,
        state.carrier_stamp.version,
        state.liquid.stamp.id,
        state.liquid.stamp.version,
        snapshot.frame() as u64,
    ];
    bits.push(
        snapshot
            .accepted_interval_start()
            .map_or(u64::MAX, f64::to_bits),
    );
    for values in [
        state.carrier.x,
        state.carrier.y,
        state.carrier.z,
        state.carrier.tracer,
    ] {
        bits.extend(values.iter().map(|v| u64::from(v.to_bits())));
    }
    bits.extend(state.pressure.iter().map(|v| v.to_bits()));
    bits.extend(state.liquid.fraction.iter().map(|v| v.to_bits()));
    bits
}
fn provider_numerics(report: &ProviderReport) -> String {
    format!(
        "{:?}",
        (
            report.selected,
            &report.attempted,
            &report.rejection,
            report.candidate,
            report.accepted
        )
    )
}
fn mac_publication(snapshot: &SessionSnapshot<'_>) -> (Vec<u64>, String) {
    (
        mac_fields(snapshot),
        format!(
            "{:?}",
            (
                snapshot.identity(),
                snapshot.configuration(),
                snapshot.grid(),
                snapshot.stage(),
                snapshot.report(),
                snapshot.provider_report()
            )
        ),
    )
}
fn particle_fields(snapshot: &MaterialSnapshot<'_>) -> Vec<u64> {
    let mut bits = vec![snapshot.generation(), snapshot.time().to_bits()];
    for p in snapshot.particles() {
        bits.extend(
            p.position
                .into_iter()
                .chain(p.velocity)
                .chain([p.mass, p.temperature_kelvin])
                .map(f64::to_bits),
        );
        bits.push(u64::from(p.material));
    }
    if let Some((time, diagnostics)) = snapshot.particle_diagnostics() {
        bits.push(time.to_bits());
        for d in diagnostics {
            bits.extend(
                [d.density, d.pressure]
                    .into_iter()
                    .chain(d.acceleration)
                    .map(f64::to_bits),
            );
        }
    }
    bits
}
fn particle_publication(snapshot: &MaterialSnapshot<'_>) -> (Vec<u64>, String) {
    (
        particle_fields(snapshot),
        format!(
            "{:?}",
            (
                snapshot.identity(),
                snapshot.grid(),
                snapshot.config(),
                snapshot.stage(),
                snapshot.report(),
                snapshot.accepted_controls(),
                snapshot.diagnostic_stage()
            )
        ),
    )
}
fn assert_particle_equivalent(actual: &MaterialSnapshot<'_>, direct: &MaterialSnapshot<'_>) {
    assert_eq!(particle_publication(actual), particle_publication(direct));
}
fn assert_refusal(result: Result<NativeStepReport, NativeStepError>, reason: NativeRefusal) {
    assert!(matches!(result,Err(NativeStepError::Refused(actual)) if actual==reason));
}

#[test]
fn c1a_mac_native_baseline_100_intervals_are_bitwise_owner_equivalent() {
    let identity = identity();
    let (mut owner, mut workspace) = mac(100);
    let (mut direct, mut reference_workspace) = mac(100);
    let mut native = NativeSession::Mac(&mut owner);
    for generation in 0..100 {
        let (reference_step, reference_pressure) = direct
            .step_with_pressure_provider(mac_inputs(), &mut reference_workspace, None, None, |_| {
                false
            })
            .unwrap();
        let report = native
            .step(
                mac_request(&identity, generation, &mut workspace),
                |point| {
                    assert!(matches!(point, NativeCancellationPoint::Mac(_)));
                    false
                },
            )
            .unwrap();
        assert_eq!(
            report.actual_dt().to_bits(),
            reference_step.carrier.step.dt.to_bits()
        );
        let NativeStepReport::Mac { step, pressure } = report else {
            panic!("wrong native report family")
        };
        assert_eq!(format!("{step:?}"), format!("{reference_step:?}"));
        assert_eq!(
            provider_numerics(&pressure),
            provider_numerics(&reference_pressure)
        );
        let NativeSnapshot::Mac(snapshot) = native.snapshot() else {
            panic!("wrong snapshot family")
        };
        let reference = direct.snapshot();
        assert_eq!(
            mac_fields(&snapshot),
            mac_fields(&reference),
            "accepted interval {}",
            generation + 1
        );
        assert_eq!(snapshot.identity(), reference.identity());
        assert_eq!(
            snapshot.accepted_interval_start(),
            reference.accepted_interval_start()
        );
        assert_eq!(snapshot.stage(), SnapshotStage::Accepted);
        assert_eq!(
            format!("{:?}", snapshot.configuration()),
            format!("{:?}", reference.configuration())
        );
        assert_eq!(
            provider_numerics(snapshot.provider_report().unwrap()),
            provider_numerics(reference.provider_report().unwrap())
        );
    }
}

#[test]
fn c1a_particle_preview_retains_native_controls_diagnostics_and_shortened_dt() {
    let identity = identity();
    let mut owner = particle(8);
    let mut direct = particle(8);
    let mut native = NativeSession::ParticlePreview(&mut owner);
    for generation in 0..8 {
        let controls = if generation < 4 {
            particle_controls()
        } else {
            shortened_controls(generation + 1)
        };
        let reference = direct.step(controls, None, || false).unwrap();
        let report = native
            .step(particle_request(&identity, generation, controls), |point| {
                assert_eq!(point, NativeCancellationPoint::ParticlePoll);
                false
            })
            .unwrap();
        assert_eq!(report.actual_dt().to_bits(), reference.dt.to_bits());
        if generation >= 4 {
            assert!(report.actual_dt() < controls.requested_dt);
        }
        let NativeStepReport::ParticlePreview(report) = report else {
            panic!("wrong report family")
        };
        assert_eq!(format!("{report:?}"), format!("{reference:?}"));
        let NativeSnapshot::ParticlePreview(snapshot) = native.snapshot() else {
            panic!("wrong snapshot family")
        };
        assert_particle_equivalent(&snapshot, &direct.snapshot());
        assert_eq!(snapshot.accepted_controls(), Some(controls));
        assert_eq!(
            snapshot.diagnostic_stage(),
            Some(ParticleEvaluationStage::PostEmissionPreIntegration)
        );
    }
}

#[test]
fn c1a_snapshots_borrow_original_native_fields_without_conversion() {
    let (mut mac_owner, _) = mac(1);
    let velocity = mac_owner.snapshot().state().carrier.x.as_ptr();
    let pressure = mac_owner.snapshot().state().pressure.as_ptr();
    let native = NativeSession::Mac(&mut mac_owner);
    let snapshot = native.snapshot();
    assert_eq!(snapshot.family(), NativeFamilyKind::Mac);
    assert_eq!(snapshot.generation(), 0);
    assert_eq!(snapshot.identity(), &identity());
    let NativeSnapshot::Mac(snapshot) = snapshot else {
        panic!("wrong snapshot")
    };
    assert_eq!(snapshot.state().carrier.x.as_ptr(), velocity);
    assert_eq!(snapshot.state().pressure.as_ptr(), pressure);
    let mut particle_owner = particle(1);
    let particles = particle_owner.snapshot().particles().as_ptr();
    let native = NativeSession::ParticlePreview(&mut particle_owner);
    let NativeSnapshot::ParticlePreview(snapshot) = native.snapshot() else {
        panic!("wrong snapshot")
    };
    assert_eq!(snapshot.particles().as_ptr(), particles);
}

#[test]
fn c1a_wrong_family_and_declared_source_are_refused_before_callbacks() {
    let identity = identity();
    let (mut mac_owner, mut workspace) = mac(2);
    let mut particle_owner = particle(2);
    let before_mac = mac_publication(&mac_owner.snapshot());
    let before_particle = particle_publication(&particle_owner.snapshot());
    let mut mac_native = NativeSession::Mac(&mut mac_owner);
    let mut particle_native = NativeSession::ParticlePreview(&mut particle_owner);
    assert_refusal(
        mac_native.step(particle_request(&identity, 0, particle_controls()), |_| {
            panic!("wrong family reached callback")
        }),
        NativeRefusal::WrongFamily {
            owner: NativeFamilyKind::Mac,
            requested: NativeFamilyKind::ParticlePreview,
        },
    );
    assert_refusal(
        particle_native.step(mac_request(&identity, 0, &mut workspace), |_| {
            panic!("wrong family reached callback")
        }),
        NativeRefusal::WrongFamily {
            owner: NativeFamilyKind::ParticlePreview,
            requested: NativeFamilyKind::Mac,
        },
    );
    for (run_id, source_revision) in [
        (102, identity.source_revision.clone()),
        (101, "other-declared-source".into()),
    ] {
        let wrong = SessionIdentity {
            run_id,
            source_revision,
        };
        for (family, native) in [
            (NativeFamilyKind::Mac, &mut mac_native),
            (NativeFamilyKind::ParticlePreview, &mut particle_native),
        ] {
            let request = match family {
                NativeFamilyKind::Mac => mac_request(&wrong, 0, &mut workspace),
                NativeFamilyKind::ParticlePreview => {
                    particle_request(&wrong, 0, particle_controls())
                }
            };
            assert_refusal(
                native.step(request, |_| panic!("wrong source reached callback")),
                NativeRefusal::WrongSource {
                    family,
                    expected_run_id: run_id,
                    actual_run_id: 101,
                    source_revision_mismatch: wrong.source_revision != identity.source_revision,
                },
            );
        }
    }
    let NativeSnapshot::Mac(snapshot) = mac_native.snapshot() else {
        unreachable!()
    };
    assert_eq!(mac_publication(&snapshot), before_mac);
    let NativeSnapshot::ParticlePreview(snapshot) = particle_native.snapshot() else {
        unreachable!()
    };
    assert_eq!(particle_publication(&snapshot), before_particle);
}

#[test]
fn c1a_stale_or_future_generation_cannot_relabel_accepted_history() {
    let identity = identity();
    let (mut owner, mut workspace) = mac(2);
    let mut native = NativeSession::Mac(&mut owner);
    native
        .step(mac_request(&identity, 0, &mut workspace), |_| false)
        .unwrap();
    let NativeSnapshot::Mac(snapshot) = native.snapshot() else {
        unreachable!()
    };
    let before = mac_publication(&snapshot);
    for generation in [0, 2] {
        assert_refusal(
            native.step(mac_request(&identity, generation, &mut workspace), |_| {
                panic!("stale request invoked callback")
            }),
            NativeRefusal::StaleGeneration {
                family: NativeFamilyKind::Mac,
                expected: generation,
                actual: 1,
            },
        );
        let NativeSnapshot::Mac(snapshot) = native.snapshot() else {
            unreachable!()
        };
        assert_eq!(mac_publication(&snapshot), before);
    }
    let mut owner = particle(2);
    let mut native = NativeSession::ParticlePreview(&mut owner);
    native
        .step(particle_request(&identity, 0, particle_controls()), |_| {
            false
        })
        .unwrap();
    let NativeSnapshot::ParticlePreview(snapshot) = native.snapshot() else {
        unreachable!()
    };
    let before = particle_publication(&snapshot);
    for generation in [0, 2] {
        assert_refusal(
            native.step(
                particle_request(&identity, generation, shortened_controls(99)),
                |_| panic!("stale request invoked callback"),
            ),
            NativeRefusal::StaleGeneration {
                family: NativeFamilyKind::ParticlePreview,
                expected: generation,
                actual: 1,
            },
        );
        let NativeSnapshot::ParticlePreview(snapshot) = native.snapshot() else {
            unreachable!()
        };
        assert_eq!(particle_publication(&snapshot), before);
    }
}

#[test]
fn c1a_mac_errors_deadline_and_cancellation_preserve_the_native_publication() {
    let identity = identity();
    let (mut owner, mut workspace) = mac(2);
    let (mut direct, mut reference_workspace) = mac(2);
    owner
        .step_with_pressure_provider(mac_inputs(), &mut workspace, None, None, |_| false)
        .unwrap();
    direct
        .step_with_pressure_provider(mac_inputs(), &mut reference_workspace, None, None, |_| {
            false
        })
        .unwrap();
    let before = mac_publication(&owner.snapshot());
    let mut native = NativeSession::Mac(&mut owner);
    let mut invalid = mac_inputs();
    invalid.requested_dt = f64::NAN;
    let expected_error = direct
        .step_with_pressure_provider(invalid, &mut reference_workspace, None, None, |_| false)
        .unwrap_err();
    let error = native
        .step(
            NativeStepRequest::Mac(MacRequest {
                expected: expected(&identity, 1),
                inputs: invalid,
                workspace: &mut workspace,
                deadline: None,
            }),
            |_| false,
        )
        .unwrap_err();
    assert!(
        matches!(error,NativeStepError::Mac(ref e) if format!("{e:?}")==format!("{expected_error:?}"))
    );
    let error = native
        .step(
            NativeStepRequest::Mac(MacRequest {
                expected: expected(&identity, 1),
                inputs: mac_inputs(),
                workspace: &mut workspace,
                deadline: Some(Instant::now()),
            }),
            |_| panic!("expired request invoked callback"),
        )
        .unwrap_err();
    assert!(matches!(
        error,
        NativeStepError::Mac(SessionError::Deadline)
    ));
    for final_commit in [false, true] {
        let mut callbacks = 0;
        let mut first = None;
        let result = native.step(mac_request(&identity, 1, &mut workspace), |point| {
            callbacks += 1;
            first.get_or_insert(point);
            !final_commit || point == NativeCancellationPoint::Mac(LiquidStepStage::BeforeCommit)
        });
        assert!(callbacks > 0);
        assert_eq!(
            first,
            Some(NativeCancellationPoint::Mac(LiquidStepStage::Carrier(
                StepStage::BeforeAdvection
            )))
        );
        if final_commit {
            assert!(matches!(
                result,
                Err(NativeStepError::Mac(SessionError::Step(
                    LiquidStepError::Cancelled
                )))
            ));
        } else {
            assert!(matches!(
                result,
                Err(NativeStepError::Mac(SessionError::Step(
                    LiquidStepError::Carrier(SimulationError::Cancelled {
                        stage: StepStage::BeforeAdvection
                    })
                )))
            ));
        }
        let NativeSnapshot::Mac(snapshot) = native.snapshot() else {
            unreachable!()
        };
        assert_eq!(mac_publication(&snapshot), before);
    }
    native
        .step(mac_request(&identity, 1, &mut workspace), |_| false)
        .unwrap();
    direct
        .step_with_pressure_provider(mac_inputs(), &mut reference_workspace, None, None, |_| {
            false
        })
        .unwrap();
    let NativeSnapshot::Mac(snapshot) = native.snapshot() else {
        unreachable!()
    };
    assert_eq!(mac_fields(&snapshot), mac_fields(&direct.snapshot()));
}

#[test]
fn c1a_particle_failures_retain_controls_diagnostics_and_retry_identity() {
    let identity = identity();
    let mut owner = particle(2);
    let mut direct = particle(2);
    owner.step(particle_controls(), None, || false).unwrap();
    direct.step(particle_controls(), None, || false).unwrap();
    let before = particle_publication(&owner.snapshot());
    let mut native = NativeSession::ParticlePreview(&mut owner);
    let mut invalid = particle_controls();
    invalid.requested_dt = f64::NAN;
    assert!(matches!(
        native.step(particle_request(&identity, 1, invalid), |_| false),
        Err(NativeStepError::ParticlePreview(
            MaterialSessionError::Physics(ParticleFluidError::InvalidTimeStep)
        ))
    ));
    assert!(matches!(
        native.step(
            NativeStepRequest::ParticlePreview(ParticleRequest {
                expected: expected(&identity, 1),
                controls: particle_controls(),
                deadline: Some(Instant::now())
            }),
            |_| panic!("expired request invoked callback")
        ),
        Err(NativeStepError::ParticlePreview(
            MaterialSessionError::Deadline
        ))
    ));
    let controls = shortened_controls(2);
    let mut checkpoints = 0;
    let report = direct
        .step(controls, None, || {
            checkpoints += 1;
            false
        })
        .unwrap();
    assert!(checkpoints > 1);
    for selected in [1, checkpoints] {
        let mut visits = 0;
        assert!(matches!(
            native.step(particle_request(&identity, 1, controls), |point| {
                assert_eq!(point, NativeCancellationPoint::ParticlePoll);
                visits += 1;
                visits == selected
            }),
            Err(NativeStepError::ParticlePreview(
                MaterialSessionError::Physics(ParticleFluidError::Cancelled)
            ))
        ));
        assert_eq!(visits, selected);
        let NativeSnapshot::ParticlePreview(snapshot) = native.snapshot() else {
            unreachable!()
        };
        assert_eq!(particle_publication(&snapshot), before);
    }
    let actual = native
        .step(particle_request(&identity, 1, controls), |_| false)
        .unwrap();
    assert_eq!(actual.actual_dt().to_bits(), report.dt.to_bits());
    let NativeSnapshot::ParticlePreview(snapshot) = native.snapshot() else {
        unreachable!()
    };
    assert_particle_equivalent(&snapshot, &direct.snapshot());
}

#[test]
fn c1a_quota_and_stop_stay_native_owner_failures() {
    let identity = identity();
    let (mut owner, mut workspace) = mac(1);
    let mut native = NativeSession::Mac(&mut owner);
    native
        .step(mac_request(&identity, 0, &mut workspace), |_| false)
        .unwrap();
    let NativeSnapshot::Mac(snapshot) = native.snapshot() else {
        unreachable!()
    };
    let before = mac_publication(&snapshot);
    assert!(matches!(
        native.step(mac_request(&identity, 1, &mut workspace), |_| panic!(
            "quota invoked callback"
        )),
        Err(NativeStepError::Mac(SessionError::StepLimit))
    ));
    let NativeSession::Mac(owner) = &mut native else {
        unreachable!()
    };
    owner.stop();
    assert!(matches!(
        native.step(mac_request(&identity, 1, &mut workspace), |_| panic!(
            "stopped owner invoked callback"
        )),
        Err(NativeStepError::Mac(SessionError::Stopped))
    ));
    let NativeSnapshot::Mac(snapshot) = native.snapshot() else {
        unreachable!()
    };
    assert_eq!(mac_publication(&snapshot), before);
    let mut owner = particle(1);
    let mut native = NativeSession::ParticlePreview(&mut owner);
    native
        .step(particle_request(&identity, 0, particle_controls()), |_| {
            false
        })
        .unwrap();
    let NativeSnapshot::ParticlePreview(snapshot) = native.snapshot() else {
        unreachable!()
    };
    let before = particle_publication(&snapshot);
    assert!(matches!(
        native.step(
            particle_request(&identity, 1, particle_controls()),
            |_| panic!("quota invoked callback")
        ),
        Err(NativeStepError::ParticlePreview(
            MaterialSessionError::StepLimit
        ))
    ));
    let NativeSession::ParticlePreview(owner) = &mut native else {
        unreachable!()
    };
    owner.stop();
    assert!(matches!(
        native.step(
            particle_request(&identity, 1, particle_controls()),
            |_| panic!("stopped owner invoked callback")
        ),
        Err(NativeStepError::ParticlePreview(
            MaterialSessionError::Stopped
        ))
    ));
    let NativeSnapshot::ParticlePreview(snapshot) = native.snapshot() else {
        unreachable!()
    };
    assert_eq!(particle_publication(&snapshot), before);
}

#[test]
fn c1a_named_cross_family_transfers_are_explicitly_unavailable() {
    for operator in [
        TransferOperator::MacToParticleV1,
        TransferOperator::ParticleToMacV1,
    ] {
        assert_eq!(
            require_transfer(operator),
            Err(TransferUnavailable { operator })
        );
    }
}
