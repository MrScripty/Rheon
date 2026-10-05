use rheon::*;

fn fields(g: &GridGeometry) -> [Vec<f32>; 3] {
    [
        vec![0.0; g.face_len(Axis::X)],
        vec![0.0; g.face_len(Axis::Y)],
        vec![0.0; g.face_len(Axis::Z)],
    ]
}
fn refs(v: &[Vec<f32>; 3]) -> [&[f32]; 3] {
    [&v[0], &v[1], &v[2]]
}
fn settings() -> PressureSettings {
    PressureSettings {
        relative_residual: 1e-12,
        absolute_residual: 1e-12,
        divergence_limit: 1e-9,
        max_iterations: 2000,
    }
}
fn stamp(id: u64) -> VolumeStamp {
    VolumeStamp { id, version: 0 }
}
fn config() -> LiquidTransportConfig {
    LiquidTransportConfig {
        carrier: SimulationConfig {
            density: 1.0,
            memory_limit: 1024 * 1024,
            pressure: settings(),
            actual_divergence_limit: 1e-5,
            max_courant: 1.0,
        },
        represented_density: 1.0,
        volume_stamp: stamp(1),
        carrier_id: 2,
    }
}
fn request(forces: &[BodyForce]) -> LiquidStepInputs<'_> {
    LiquidStepInputs {
        requested_dt: 0.5,
        smoke_source: None,
        forces,
        inlet: LiquidInlet::new(stamp(3), [[0.0; 2]; 3]).unwrap(),
        source: None,
        volume: LiquidVolumeSettings {
            max_outward_courant: 1.0,
            actual_divergence_limit: 1e-5,
        },
    }
}
fn slab(g: &GridGeometry, axis: Axis, layers: usize) -> SlabFreeSurface {
    SlabFreeSurface::new(g.clone(), axis, layers, stamp(4)).unwrap()
}
fn state(
    g: &GridGeometry,
    method: PressureImplementation,
    surface: SlabFreeSurface,
) -> LiquidTransportSimulation {
    let fraction = (0..g.cell_len())
        .map(|i| if surface.is_wet_cell(i) { 1.0 } else { 0.0 })
        .collect();
    LiquidTransportSimulation::with_free_surface(g.clone(), config(), method, fraction, surface)
        .unwrap()
}
fn snapshot(
    s: &LiquidTransportSimulation,
) -> (
    Vec<u32>,
    Vec<u64>,
    u64,
    u64,
    VolumeStamp,
    VolumeStamp,
    Option<VolumeStamp>,
) {
    let v = s.state();
    (
        v.carrier
            .x
            .iter()
            .chain(v.carrier.y)
            .chain(v.carrier.z)
            .chain(v.carrier.tracer)
            .map(|x| x.to_bits())
            .collect(),
        v.pressure
            .iter()
            .chain(v.liquid.fraction)
            .map(|x| x.to_bits())
            .collect(),
        v.carrier.time.to_bits(),
        v.liquid.time.to_bits(),
        v.liquid.stamp,
        v.carrier_stamp,
        v.pressure_surface.map(|x| x.stamp()),
    )
}
fn near(a: f64, b: f64) {
    assert!(
        (a - b).abs() <= 1e-9 * a.abs().max(b.abs()).max(1.0),
        "{a} != {b}"
    );
}

#[test]
fn independent_anchored_matrix_has_half_distance_surface_and_no_wet_gauge() {
    let g = GridGeometry::new([3, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let s = slab(&g, Axis::X, 2);
    let op = PressureOperator::with_free_surface(&g, 1.0, &s).unwrap();
    let matrix = [[1.0, -1.0, 0.0], [-1.0, 3.0, 0.0], [0.0, 0.0, 1.0]];
    for col in 0..3 {
        let mut p = vec![0.0; 3];
        p[col] = 1.0;
        let mut out = vec![0.0; 3];
        op.apply_full(&p, &mut out).unwrap();
        for row in 0..3 {
            assert_eq!(out[row], matrix[row][col]);
            assert_eq!(matrix[row][col], matrix[col][row]);
        }
    }
    for method in PressureImplementation::ALL {
        let mut ws = PressureWorkspace::with_implementation(&g, 144, method).unwrap();
        ws.solve_rhs(&op, &[0.5, 0.0, 0.0], 0.5, settings(), || false)
            .unwrap();
        near(ws.pressure()[0], 0.75);
        near(ws.pressure()[1], 0.25);
        assert_eq!(ws.pressure()[2], 0.0);
        assert!(matches!(
            ws.solve_rhs(&op, &[0.0, 0.0, 1.0], 0.5, settings(), || false),
            Err(PressureError::Operator(OperatorError::NonZeroAirRhs {
                cell: 2
            }))
        ));
        let closed = PressureOperator::new(&g, 1.0).unwrap();
        assert!(matches!(
            ws.solve_rhs(&closed, &[0.5, 0.0, 0.0], 0.5, settings(), || false),
            Err(PressureError::IncompatibleRhs { .. })
        ));
    }
}

#[test]
fn hydrostatic_columns_match_pressure_at_cell_centers_and_zero_air_pressure() {
    for axis in [Axis::X, Axis::Y, Axis::Z] {
        for layers in [1, 2, 4, 8] {
            let mut dims = [1; 3];
            let d = match axis {
                Axis::X => 0,
                Axis::Y => 1,
                Axis::Z => 2,
            };
            dims[d] = layers + 2;
            let mut h = [1.0; 3];
            h[d] = 0.5;
            let g = GridGeometry::new(dims, h, [0.0; 3]).unwrap();
            let surface = slab(&g, axis, layers as usize);
            let op = PressureOperator::with_free_surface(&g, 2.0, &surface).unwrap();
            let mut v = fields(&g);
            for j in 1..=layers as usize {
                let mut p = [0; 3];
                p[d] = j;
                v[d][g.face_index(axis, p).unwrap()] = -0.125;
            }
            for method in PressureImplementation::ALL {
                let mut ws =
                    PressureWorkspace::with_implementation(&g, g.cell_len() * 48, method).unwrap();
                ws.solve_velocity(&op, refs(&v), 0.5, settings(), || false)
                    .unwrap();
                for (i, &p) in ws.pressure().iter().enumerate() {
                    let exact = if i < layers as usize {
                        2.0 * 0.25 * (surface.position() - (i as f64 + 0.5) * 0.5)
                    } else {
                        0.0
                    };
                    near(p, exact);
                }
                let mut corrected = fields(&g);
                let [x, y, z] = &mut corrected;
                op.correct_velocity(refs(&v), ws.pressure(), 0.5, [x, y, z])
                    .unwrap();
                assert!(
                    corrected
                        .iter()
                        .flatten()
                        .all(|&x| f64::from(x).abs() < 1e-10)
                );
                assert!(ws.actual_divergence_max(&op, refs(&corrected)).unwrap() < 1e-9);
            }
        }
    }
}

#[test]
fn coupled_surface_redistribution_conserves_volume_and_rejects_unresolved_next_geometry() {
    let g = GridGeometry::new([2, 2, 1], [1.0; 3], [0.0; 3]).unwrap();
    let forces = [
        BodyForce {
            value: [0.0, 0.5, 0.0],
            units: ForceUnits::Acceleration,
            region: Some(ForceRegion {
                lower: [0.0; 3],
                upper: [1.0, 2.0, 1.0],
            }),
        },
        BodyForce {
            value: [0.0, -0.5, 0.0],
            units: ForceUnits::Acceleration,
            region: Some(ForceRegion {
                lower: [1.0, 0.0, 0.0],
                upper: [2.0, 2.0, 1.0],
            }),
        },
    ];
    for method in PressureImplementation::ALL {
        let mut s = state(&g, method, slab(&g, Axis::Y, 1));
        let r = s.step(request(&forces), |_| false).unwrap();
        assert_eq!(r.pressure_surface, Some(stamp(4)));
        assert_eq!(
            r.liquid.divergence_domain,
            VolumeDivergenceDomain::LiquidSlab {
                axis: Axis::Y,
                wet_layers: 1
            }
        );
        assert_eq!(s.state().liquid.fraction, [1.0, 0.9375, 0.0625, 0.0]);
        near(s.state().pressure[0], -0.125);
        near(s.state().pressure[1], 0.125);
        assert_eq!(&s.state().pressure[2..], &[0.0; 2]);
        assert_eq!(r.liquid.liquid_volume_after, 2.0);
        assert_eq!(r.liquid.volume_balance_error, 0.0);
        assert_eq!(r.carrier.step.actual_divergence_max, 0.0);
        assert_eq!(s.state().carrier.time, s.state().liquid.time);
        // Air divergence is permitted by the declared wet-only pressure gate.
        let op = PressureOperator::new(&g, 1.0).unwrap();
        let view = s.state().carrier;
        let mut div = vec![0.0; 4];
        op.divergence([view.x, view.y, view.z], &mut div).unwrap();
        assert_eq!(&div[2..], &[-0.125, 0.125]);
        let before = snapshot(&s);
        assert!(matches!(
            s.step(request(&forces), |_| panic!(
                "mixed geometry reached callback"
            )),
            Err(LiquidStepError::FreeSurface(
                FreeSurfaceError::FractionMismatch { .. }
            ))
        ));
        assert_eq!(snapshot(&s), before);
    }
}

#[test]
fn resolved_hydrostatic_interval_replays_and_cancelled_pair_preserves_pressure_and_phase() {
    let g = GridGeometry::new([1, 2, 1], [1.0; 3], [0.0; 3]).unwrap();
    let gravity = [BodyForce {
        value: [0.0, -0.25, 0.0],
        units: ForceUnits::Acceleration,
        region: None,
    }];
    for method in PressureImplementation::ALL {
        let mut reference = state(&g, method, slab(&g, Axis::Y, 1));
        for _ in 0..16 {
            let r = reference.step(request(&gravity), |_| false).unwrap();
            assert_eq!(r.liquid.liquid_volume_after, 1.0);
        }
        assert_eq!(reference.state().liquid.fraction, [1.0, 0.0]);
        near(reference.state().pressure[0], 0.125);
        let mut probe = state(&g, method, slab(&g, Axis::Y, 1));
        probe.step(request(&gravity), |_| false).unwrap();
        let mut stages = vec![];
        probe
            .step(request(&gravity), |stage| {
                if !stages.contains(&stage) {
                    stages.push(stage);
                }
                false
            })
            .unwrap();
        for target in stages {
            let mut s = state(&g, method, slab(&g, Axis::Y, 1));
            s.step(request(&gravity), |_| false).unwrap();
            let before = snapshot(&s);
            assert!(s.step(request(&gravity), |stage| stage == target).is_err());
            assert_eq!(snapshot(&s), before);
            s.step(request(&gravity), |_| false).unwrap();
            assert_eq!(snapshot(&s), snapshot(&probe));
        }
    }
}

#[test]
fn free_surface_admission_is_explicit_and_uses_existing_array_inventory() {
    let g = GridGeometry::new([1, 2, 1], [1.0; 3], [0.0; 3]).unwrap();
    assert!(matches!(
        SlabFreeSurface::new(g.clone(), Axis::Y, 0, stamp(4)),
        Err(FreeSurfaceError::InvalidSlab)
    ));
    assert!(matches!(
        SlabFreeSurface::new(g.clone(), Axis::Y, 2, stamp(4)),
        Err(FreeSurfaceError::InvalidSlab)
    ));
    let surface = slab(&g, Axis::Y, 1);
    let mut c = config();
    c.represented_density = 800.0;
    assert!(matches!(
        LiquidTransportSimulation::with_free_surface(
            g.clone(),
            c,
            PressureImplementation::JacobiPcgV1,
            vec![1.0, 0.0],
            surface.clone()
        ),
        Err(LiquidStepError::FreeSurface(
            FreeSurfaceError::DensityMismatch
        ))
    ));
    assert!(matches!(
        LiquidTransportSimulation::with_free_surface(
            g.clone(),
            config(),
            PressureImplementation::JacobiPcgV1,
            vec![0.5, 0.0],
            surface.clone()
        ),
        Err(LiquidStepError::FreeSurface(
            FreeSurfaceError::FractionMismatch { cell: 0 }
        ))
    ));
    let ordinary = LiquidTransportSimulation::new(
        g.clone(),
        config(),
        PressureImplementation::JacobiPcgV1,
        vec![1.0, 0.0],
    )
    .unwrap();
    let mut s = state(&g, PressureImplementation::JacobiPcgV1, surface);
    assert_eq!(s.allocated_bytes(), ordinary.allocated_bytes());
    let before = snapshot(&s);
    let mut inputs = request(&[]);
    inputs.smoke_source = Some(SmokeSource {
        lower: [0.0; 3],
        upper: [1.0; 3],
        tracer_rate: 0.0,
        vertical_acceleration: 0.25,
    });
    assert!(matches!(
        s.step(inputs, |_| panic!("unsupported source reached callback")),
        Err(LiquidStepError::FreeSurface(
            FreeSurfaceError::UnsupportedSmokeSource
        ))
    ));
    let mut workspace = BoxFluxStepWorkspace::new(
        g.clone(),
        1.0,
        1024 * 1024,
        PressureImplementation::JacobiPcgV1,
    )
    .unwrap();
    let boundary = BoxFluxStepBoundary {
        flux: PrescribedBoxFlux::new(BoxFluxStamp { id: 13, version: 0 }, [[0.0; 2]; 3]).unwrap(),
        tracer: BoxFluxTracerPolicy::ClampedAppearance,
    };
    assert!(matches!(
        s.step_with_box_flux(request(&[]), &mut workspace, boundary, |_| panic!(
            "unsupported boundary reached callback"
        )),
        Err(LiquidStepError::FreeSurface(
            FreeSurfaceError::UnsupportedBoxFlux
        ))
    ));
    assert_eq!(snapshot(&s), before);
}

#[test]
fn rectangular_manufactured_pressure_matches_independent_wet_air_assembly() {
    let g = GridGeometry::new([3, 4, 2], [0.5, 0.25, 1.0], [0.0; 3]).unwrap();
    let surface = slab(&g, Axis::Y, 2);
    let op = PressureOperator::with_free_surface(&g, 2.0, &surface).unwrap();
    let mut exact = vec![0.0; g.cell_len()];
    let mut rhs = vec![0.0; g.cell_len()];
    for k in 0..2 {
        for j in 0..2 {
            for i in 0..3 {
                exact[g.cell_index([i, j, k]).unwrap()] =
                    (i + 1) as f64 / 16.0 + (j + 1) as f64 / 8.0 + (k + 1) as f64 / 32.0;
            }
        }
    }
    let weights = [0.25, 1.0, 0.0625];
    let dims = g.counts();
    for k in 0..2 {
        for j in 0..2 {
            for i in 0..3 {
                let p = [i, j, k];
                let row = g.cell_index(p).unwrap();
                for d in 0..3 {
                    for sign in [-1, 1] {
                        let neighbor = p[d] as isize + sign;
                        if neighbor < 0 || neighbor >= dims[d] as isize {
                            continue;
                        }
                        let mut q = p;
                        q[d] = neighbor as usize;
                        let col = g.cell_index(q).unwrap();
                        rhs[row] += if q[1] < 2 {
                            weights[d] * (exact[row] - exact[col])
                        } else {
                            2.0 * weights[d] * exact[row]
                        };
                    }
                }
            }
        }
    }
    for method in PressureImplementation::ALL {
        let mut ws = PressureWorkspace::with_implementation(&g, 48 * g.cell_len(), method).unwrap();
        let report = ws.solve_rhs(&op, &rhs, 0.5, settings(), || false).unwrap();
        assert!(report.true_residual_max < 1e-9);
        for (&value, &expected) in ws.pressure().iter().zip(&exact) {
            near(value, expected);
        }
        let mut product = vec![0.0; g.cell_len()];
        op.apply_full(&exact, &mut product).unwrap();
        assert_eq!(product, rhs);
    }
}

#[test]
fn late_volume_rejection_retains_previous_free_surface_pressure_and_liquid() {
    let g = GridGeometry::new([1, 2, 1], [1.0; 3], [0.0; 3]).unwrap();
    for method in PressureImplementation::ALL {
        let mut s = state(&g, method, slab(&g, Axis::Y, 1));
        let old = [BodyForce {
            value: [0.0, -0.25, 0.0],
            units: ForceUnits::Acceleration,
            region: None,
        }];
        s.step(request(&old), |_| false).unwrap();
        let before = snapshot(&s);
        let new = [BodyForce {
            value: [0.0, -0.5, 0.0],
            units: ForceUnits::Acceleration,
            region: None,
        }];
        let rates = [1.0, 0.0];
        let mut inputs = request(&new);
        inputs.source = Some(LiquidVolumeSource::new(stamp(10), &rates).unwrap());
        let mut volume_reached = false;
        assert!(matches!(
            s.step(inputs, |stage| {
                volume_reached |= stage == LiquidStepStage::Volume(VolumeStage::BeforeFlux);
                false
            }),
            Err(LiquidStepError::Volume(
                LiquidVolumeError::FractionBounds { .. }
            ))
        ));
        assert!(volume_reached);
        assert_eq!(snapshot(&s), before);
        near(s.state().pressure[0], 0.125);
        s.step(request(&new), |_| false).unwrap();
        near(s.state().pressure[0], 0.25);
        assert_eq!(s.state().liquid.fraction, [1.0, 0.0]);
    }
}
