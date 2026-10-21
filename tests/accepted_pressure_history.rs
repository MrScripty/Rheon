#![cfg(feature = "accepted-pressure-warm-start")]
use rheon::*;

fn grid() -> GridGeometry {
    GridGeometry::new([4, 3, 2], [0.25, 0.5, 0.75], [1.0, -2.0, 3.0]).unwrap()
}
fn config() -> LiquidTransportConfig {
    LiquidTransportConfig {
        carrier: SimulationConfig {
            density: 1000.0,
            memory_limit: 1 << 20,
            pressure: PressureSettings {
                relative_residual: 1e-12,
                absolute_residual: 1e-12,
                divergence_limit: 1e-9,
                max_iterations: 1000,
            },
            actual_divergence_limit: 1e-5,
            max_courant: 1.0,
        },
        represented_density: 1000.0,
        volume_stamp: VolumeStamp { id: 11, version: 0 },
        carrier_id: 17,
    }
}
fn make(method: PressureImplementation) -> LiquidTransportSimulation {
    let g = grid();
    LiquidTransportSimulation::new(g.clone(), config(), method, vec![0.5; g.cell_len()]).unwrap()
}
const FORCE: [BodyForce; 1] = [BodyForce {
    value: [0.125, -0.25, 0.0],
    units: ForceUnits::Acceleration,
    region: None,
}];
fn inputs() -> LiquidStepInputs<'static> {
    LiquidStepInputs {
        requested_dt: 0.125,
        smoke_source: None,
        forces: &FORCE,
        inlet: LiquidInlet::new(VolumeStamp { id: 19, version: 0 }, [[0.0; 2]; 3]).unwrap(),
        source: None,
        volume: LiquidVolumeSettings {
            max_outward_courant: 1.0,
            actual_divergence_limit: 1e-5,
        },
    }
}
const WARM: PressureInitializationPolicy = PressureInitializationPolicy::AcceptedPreviousV1;
type FieldSnapshot = (
    Vec<u32>,
    Vec<u64>,
    Vec<u64>,
    u64,
    u64,
    VolumeStamp,
    VolumeStamp,
);
fn fields(s: LiquidTransportView<'_>) -> FieldSnapshot {
    (
        s.carrier
            .x
            .iter()
            .chain(s.carrier.y)
            .chain(s.carrier.z)
            .chain(s.carrier.tracer)
            .map(|v| v.to_bits())
            .collect(),
        s.pressure.iter().map(|v| v.to_bits()).collect(),
        s.liquid.fraction.iter().map(|v| v.to_bits()).collect(),
        s.carrier.time.to_bits(),
        s.liquid.time.to_bits(),
        s.carrier_stamp,
        s.liquid.stamp,
    )
}

#[test]
fn accepted_seed_is_opt_in_and_pressure_stays_in_pa_when_dt_changes() {
    for method in PressureImplementation::ALL {
        let mut s = make(method);
        let bytes = s.allocated_bytes();
        assert_eq!(s.pressure_seed(), None);
        let first = s
            .step_with_initialization(inputs(), WARM, |_| false)
            .unwrap();
        assert_eq!(
            first.pressure_initialization.cold_reason,
            Some(PressureColdReason::Constructor)
        );
        let seed = s.pressure_seed().unwrap();
        assert_eq!(seed.accepted_generation, first.carrier.step.generation);
        assert_eq!(seed.stage, PressureSeedStage::AcceptedLiquidStep);
        let mut request = inputs();
        request.requested_dt *= 0.5;
        let next = s
            .step_with_initialization(request, WARM, |_| false)
            .unwrap();
        assert_eq!(next.pressure_initialization.seed, Some(seed));
        assert_eq!(next.pressure_initialization.actual, WARM);
        // Constant acceleration in a closed box has hydrostatic p=rho*a.x,
        // independent of dt; gauge is fixed at the first cell center.
        let g = grid();
        for k in 0..2 {
            for j in 0..3 {
                for i in 0..4 {
                    let expected = 1000.0 * (0.125 * i as f64 * 0.25 - 0.25 * j as f64 * 0.5);
                    assert!(
                        (s.state().pressure[g.cell_index([i, j, k]).unwrap()] - expected).abs()
                            < 1e-6
                    );
                }
            }
        }
        assert!(
            s.state()
                .carrier
                .x
                .iter()
                .chain(s.state().carrier.y)
                .chain(s.state().carrier.z)
                .all(|v| v.abs() < 1e-7)
        );
        assert!((next.liquid.volume_balance_error).abs() < 1e-12);
        assert_eq!(bytes, s.allocated_bytes());
        let cold = s.step(inputs(), |_| false).unwrap();
        assert_eq!(
            cold.pressure_initialization.actual,
            PressureInitializationPolicy::Cold
        );
        assert_eq!(
            cold.pressure_initialization.cold_reason,
            Some(PressureColdReason::PolicyCold)
        );
        assert!(s.pressure_seed().is_some());
        let mut other = make(method);
        other.step(inputs(), |_| false).unwrap();
        assert_ne!(
            s.pressure_seed().unwrap().owner_incarnation,
            other.pressure_seed().unwrap().owner_incarnation
        );
    }
}

#[test]
fn every_observed_checkpoint_preserves_accepted_seed_and_replays_dirty_scratch() {
    for method in PressureImplementation::ALL {
        let changed = [BodyForce {
            value: [-0.0625, 0.125, 0.0625],
            ..FORCE[0]
        }];
        let mut request = inputs();
        request.forces = &changed;
        let mut reference = make(method);
        reference
            .step_with_initialization(inputs(), WARM, |_| false)
            .unwrap();
        let mut stages = Vec::new();
        reference
            .step_with_initialization(request, WARM, |stage| {
                stages.push(stage);
                false
            })
            .unwrap();
        assert!(stages.contains(&LiquidStepStage::Carrier(StepStage::PressureIteration)));
        assert!(stages.contains(&LiquidStepStage::Volume(VolumeStage::CellUpdateSlice)));
        assert_eq!(stages.last(), Some(&LiquidStepStage::BeforeCommit));
        let mut unique = stages.clone();
        unique.dedup();
        for stage in unique {
            let count = stages.iter().filter(|&&v| v == stage).count();
            for occurrence in [1, count] {
                let mut s = make(method);
                s.step_with_initialization(inputs(), WARM, |_| false)
                    .unwrap();
                let before = fields(s.state());
                let stamp = s.pressure_seed();
                let mut seen = 0;
                assert!(
                    s.step_with_initialization(request, WARM, |actual| {
                        if actual == stage {
                            seen += 1;
                        }
                        actual == stage && seen == occurrence
                    })
                    .is_err(),
                    "{method:?} {stage:?} #{occurrence}"
                );
                assert_eq!(fields(s.state()), before);
                assert_eq!(s.pressure_seed(), stamp);
                let retry = s
                    .step_with_initialization(request, WARM, |_| false)
                    .unwrap();
                assert_eq!(retry.pressure_initialization.seed, stamp);
                assert_eq!(fields(s.state()), fields(reference.state()));
            }
        }
    }
}

#[test]
fn unsupported_publication_invalidates_but_failed_boundary_and_provider_preserve() {
    let mut s = make(PressureImplementation::JacobiPcgV1);
    s.step(inputs(), |_| false).unwrap();
    let stamp = s.pressure_seed();
    let before = fields(s.state());
    let mut workspace =
        BoxFluxStepWorkspace::new(grid(), 1000.0, 1 << 20, PressureImplementation::JacobiPcgV1)
            .unwrap();
    let boundary = BoxFluxStepBoundary {
        flux: PrescribedBoxFlux::new(BoxFluxStamp { id: 37, version: 0 }, [[0.0; 2]; 3]).unwrap(),
        tracer: BoxFluxTracerPolicy::ClampedAppearance,
    };
    assert!(
        s.step_with_box_flux(inputs(), &mut workspace, boundary, |stage| stage
            == LiquidStepStage::BeforeCommit)
            .is_err()
    );
    assert_eq!(s.pressure_seed(), stamp);
    assert_eq!(fields(s.state()), before);
    s.step_with_box_flux(inputs(), &mut workspace, boundary, |_| false)
        .unwrap();
    assert_eq!(s.pressure_seed(), None);
    let next = s
        .step_with_initialization(inputs(), WARM, |_| false)
        .unwrap();
    assert_eq!(
        next.pressure_initialization.cold_reason,
        Some(PressureColdReason::Invalidated)
    );
    assert!(s.pressure_seed().unwrap().operator_revision > stamp.unwrap().operator_revision);
    let stamp = s.pressure_seed();
    let before = fields(s.state());
    let mut provider = ProviderStepWorkspace::new(
        &grid(),
        1000.0,
        1 << 20,
        PressureImplementation::JacobiPcgV1,
    )
    .unwrap();
    assert!(
        s.step_with_pressure_provider(inputs(), &mut provider, None, |stage| stage
            == LiquidStepStage::BeforeCommit)
            .is_err()
    );
    assert_eq!(s.pressure_seed(), stamp);
    assert_eq!(fields(s.state()), before);
    let (report, _) = s
        .step_with_pressure_provider(inputs(), &mut provider, None, |_| false)
        .unwrap();
    assert_eq!(
        report.pressure_initialization.actual,
        PressureInitializationPolicy::Cold
    );
    assert_eq!(s.pressure_seed(), None);
}

#[test]
fn memory_limit_remains_exact_and_invalid_requests_preserve_history() {
    let mut s = make(PressureImplementation::JacobiPcgV1);
    let bytes = s.allocated_bytes();
    let g = grid();
    let mut c = config();
    c.carrier.memory_limit = bytes;
    assert_eq!(
        LiquidTransportSimulation::new(
            g.clone(),
            c,
            PressureImplementation::JacobiPcgV1,
            vec![0.5; g.cell_len()]
        )
        .unwrap()
        .allocated_bytes(),
        bytes
    );
    c.carrier.memory_limit -= 1;
    assert!(
        LiquidTransportSimulation::new(
            g.clone(),
            c,
            PressureImplementation::JacobiPcgV1,
            vec![0.5; g.cell_len()]
        )
        .is_err()
    );
    s.step(inputs(), |_| false).unwrap();
    let stamp = s.pressure_seed();
    let before = fields(s.state());
    let mut request = inputs();
    request.requested_dt = f64::NAN;
    assert!(
        s.step_with_initialization(request, WARM, |_| false)
            .is_err()
    );
    assert_eq!(s.pressure_seed(), stamp);
    assert_eq!(fields(s.state()), before);
    request = inputs();
    request.volume.max_outward_courant = 0.0;
    assert!(
        s.step_with_initialization(request, WARM, |_| false)
            .is_err()
    );
    assert_eq!(s.pressure_seed(), stamp);
    assert_eq!(fields(s.state()), before);
}

#[test]
fn saturated_transport_rejection_preserves_seed_without_relaxing_fraction_bounds() {
    let g = grid();
    for method in PressureImplementation::ALL {
        let mut counts = Vec::new();
        for policy in [PressureInitializationPolicy::Cold, WARM] {
            let mut s = LiquidTransportSimulation::new(
                g.clone(),
                config(),
                method,
                vec![1.0; g.cell_len()],
            )
            .unwrap();
            let mut accepted = 0;
            for step in 0..16 {
                let before = fields(s.state());
                let seed = s.pressure_seed();
                let mut request = inputs();
                if step % 2 == 1 {
                    request.requested_dt *= 0.5;
                }
                let result = s.step_with_initialization(request, policy, |_| false);
                if let Err(error) = result {
                    eprintln!(
                        "saturated method={method:?} policy={policy:?} accepted={accepted}: {error:?}"
                    );
                    assert!(matches!(
                        error,
                        LiquidStepError::Volume(LiquidVolumeError::FractionBounds { .. })
                    ));
                    assert_eq!(fields(s.state()), before);
                    assert_eq!(s.pressure_seed(), seed);
                    let retry = s
                        .step_with_initialization(request, policy, |_| false)
                        .unwrap_err();
                    assert_eq!(error, retry);
                    assert_eq!(fields(s.state()), before);
                    assert_eq!(s.pressure_seed(), seed);
                    break;
                }
                accepted += 1;
            }
            eprintln!("saturated method={method:?} policy={policy:?} accepted={accepted}/16");
            counts.push(accepted);
        }
        assert_eq!(
            counts[0], counts[1],
            "cold/warm accepted horizons changed for {method:?}"
        );
    }
}

#[test]
fn unsupported_surface_and_column_requests_stay_cold() {
    let g = GridGeometry::new([1, 3, 1], [1.0; 3], [0.0; 3]).unwrap();
    let surface =
        SlabFreeSurface::new(g.clone(), Axis::Y, 1, VolumeStamp { id: 83, version: 0 }).unwrap();
    let slab = LiquidTransportSimulation::with_free_surface(
        g.clone(),
        config(),
        PressureImplementation::JacobiPcgV1,
        vec![1.0, 0.0, 0.0],
        surface,
    )
    .unwrap();
    let columns = LiquidTransportSimulation::with_reconstructed_surface(
        g,
        config(),
        PressureImplementation::JacobiPcgV1,
        vec![1.0, 0.5, 0.0],
        Axis::Y,
    )
    .unwrap();
    for mut s in [slab, columns] {
        let mut request = inputs();
        request.forces = &[];
        for _ in 0..2 {
            let report = s
                .step_with_initialization(request, WARM, |_| false)
                .unwrap();
            assert_eq!(
                report.pressure_initialization.actual,
                PressureInitializationPolicy::Cold
            );
            assert_eq!(
                report.pressure_initialization.cold_reason,
                Some(PressureColdReason::UnsupportedPath)
            );
            assert_eq!(s.pressure_seed(), None);
        }
    }
}

#[test]
fn localized_trajectory_matches_cold_accuracy_and_independent_flux_balance() {
    let g = grid();
    let mut cold = make(PressureImplementation::JacobiPcgV1);
    let mut warm = make(PressureImplementation::JacobiPcgV1);
    let force = [BodyForce {
        region: Some(ForceRegion {
            lower: g.origin(),
            upper: [1.5, -1.25, 3.75],
        }),
        ..FORCE[0]
    }];
    let mut request = inputs();
    request.forces = &force;
    let initial_volume = 0.5 * g.cell_len() as f64 * g.cell_volume();
    for _ in 0..8 {
        cold.step(request, |_| false).unwrap();
        warm.step_with_initialization(request, WARM, |_| false)
            .unwrap();
        let a = cold.state();
        let b = warm.state();
        for (x, y) in a
            .carrier
            .x
            .iter()
            .chain(a.carrier.y)
            .chain(a.carrier.z)
            .zip(b.carrier.x.iter().chain(b.carrier.y).chain(b.carrier.z))
        {
            assert!((f64::from(*x) - f64::from(*y)).abs() < 1e-7);
        }
        for (x, y) in a.pressure.iter().zip(b.pressure) {
            assert!((x - y).abs() < 1e-6);
        }
        for state in [a, b] {
            let volume = state.liquid.fraction.iter().sum::<f64>() * g.cell_volume();
            assert!((volume - initial_volume).abs() <= 64.0 * f64::EPSILON * initial_volume);
            let velocity = [state.carrier.x, state.carrier.y, state.carrier.z];
            for k in 0..2 {
                for j in 0..3 {
                    for i in 0..4 {
                        let cell = [i, j, k];
                        let mut divergence = 0.0;
                        for (d, axis) in [Axis::X, Axis::Y, Axis::Z].into_iter().enumerate() {
                            let mut upper = cell;
                            upper[d] += 1;
                            divergence +=
                                (f64::from(velocity[d][g.face_index(axis, upper).unwrap()])
                                    - f64::from(velocity[d][g.face_index(axis, cell).unwrap()]))
                                    / g.spacing()[d];
                        }
                        assert!(divergence.abs() <= config().carrier.actual_divergence_limit);
                    }
                }
            }
        }
    }
}
