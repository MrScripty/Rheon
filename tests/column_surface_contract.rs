use rheon::*;
fn stamp(id: u64) -> VolumeStamp {
    VolumeStamp { id, version: 0 }
}
fn settings() -> PressureSettings {
    PressureSettings {
        relative_residual: 1e-12,
        absolute_residual: 1e-12,
        divergence_limit: 1e-9,
        max_iterations: 2000,
    }
}
fn config() -> LiquidTransportConfig {
    LiquidTransportConfig {
        carrier: SimulationConfig {
            density: 1.0,
            memory_limit: 1 << 20,
            pressure: settings(),
            actual_divergence_limit: 1e-5,
            max_courant: 1.0,
        },
        represented_density: 1.0,
        volume_stamp: stamp(1),
        carrier_id: 2,
    }
}
fn inputs(forces: &[BodyForce]) -> LiquidStepInputs<'_> {
    LiquidStepInputs {
        requested_dt: 0.125,
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
fn near(a: f64, b: f64) {
    assert!(a.is_finite() && (a - b).abs() < 1e-9, "{a} != {b}");
}
fn bits(s: &LiquidTransportSimulation) -> Vec<u64> {
    let v = s.state();
    let mut result = v
        .pressure
        .iter()
        .chain(v.liquid.fraction)
        .map(|x| x.to_bits())
        .collect::<Vec<_>>();
    result.extend(
        v.carrier
            .x
            .iter()
            .chain(v.carrier.y)
            .chain(v.carrier.z)
            .chain(v.carrier.tracer)
            .map(|x| u64::from(x.to_bits())),
    );
    result.extend([
        v.carrier.time.to_bits(),
        v.liquid.time.to_bits(),
        v.carrier_stamp.version,
        v.liquid.stamp.version,
    ]);
    for surface in [v.reconstructed_surface, v.pressure_columns]
        .into_iter()
        .flatten()
    {
        result.extend([surface.stamp().id, surface.stamp().version]);
        for h in surface.heights() {
            result.extend([h.full_layers() as u64, h.top_fraction().to_bits()]);
        }
    }
    result
}
#[test]
fn independent_variable_distance_matrix_and_air_storage() {
    let g = GridGeometry::new([2, 3, 1], [1.0; 3], [0.0; 3]).unwrap();
    let surface = ColumnSurfaceWorkspace::new(
        g.clone(),
        Axis::Y,
        &[1.0, 1.0, 0.75, 0.25, 0.0, 0.0],
        stamp(1),
        1 << 20,
    )
    .unwrap();
    let op = PressureOperator::with_columns(&g, 1.0, surface.state()).unwrap();
    let matrix = [
        [2.0, -1.0, -1.0, 0.0, 0.0, 0.0],
        [-1.0, 7.0 / 3.0, 0.0, 0.0, 0.0, 0.0],
        [-1.0, 0.0, 7.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 0.0, 0.0, 1.0],
    ];
    for col in 0..6 {
        let mut p = [0.0; 6];
        p[col] = 1.0;
        let mut out = [0.0; 6];
        op.apply_full(&p, &mut out).unwrap();
        for row in 0..6 {
            near(out[row], matrix[row][col]);
        }
    }
    let exact = [0.125, 0.25, 0.5, 0.0, 0.0, 0.0];
    let rhs = matrix.map(|row| row.iter().zip(exact).map(|(a, p)| a * p).sum::<f64>());
    for method in PressureImplementation::ALL {
        let mut w = PressureWorkspace::with_implementation(&g, 6 * 48, method).unwrap();
        w.solve_rhs(&op, &rhs, 0.5, settings(), || false).unwrap();
        for (&p, q) in w.pressure().iter().zip(exact) {
            near(p, q);
        }
    }
}
#[test]
fn mixed_hydrostatic_columns_repeat_and_track_both_geometry_versions() {
    for method in PressureImplementation::ALL {
        for top in [0.125, 0.25, 0.5, 0.75] {
            let g = GridGeometry::new([1, 3, 1], [1.0; 3], [0.0; 3]).unwrap();
            let mut s = LiquidTransportSimulation::with_reconstructed_surface(
                g,
                config(),
                method,
                vec![1.0, top, 0.0],
                Axis::Y,
            )
            .unwrap();
            let gravity = [BodyForce {
                value: [0.0, -0.25, 0.0],
                units: ForceUnits::Acceleration,
                region: None,
            }];
            for version in 1..=16 {
                let r = s.step(inputs(&gravity), |_| false).unwrap();
                near(r.liquid.liquid_volume_after, 1.0 + top);
                let v = s.state();
                assert_eq!(v.reconstructed_surface.unwrap().stamp().version, version);
                assert_eq!(v.pressure_columns.unwrap().stamp().version, version - 1);
                near(v.pressure[0], 0.25 * (0.5 + top));
                near(
                    v.pressure[1],
                    if top > 0.5 { 0.25 * (top - 0.5) } else { 0.0 },
                );
            }
        }
    }
}
#[test]
fn consecutive_coupled_pulses_advance_mixed_fractions_and_preserve_geometry_on_cancel() {
    let g = GridGeometry::new([2, 3, 1], [1.0; 3], [0.0; 3]).unwrap();
    let force = [
        BodyForce {
            value: [0.0, 0.5, 0.0],
            units: ForceUnits::Acceleration,
            region: Some(ForceRegion {
                lower: [0.0; 3],
                upper: [1.0, 3.0, 1.0],
            }),
        },
        BodyForce {
            value: [0.0, -0.5, 0.0],
            units: ForceUnits::Acceleration,
            region: Some(ForceRegion {
                lower: [1.0, 0.0, 0.0],
                upper: [2.0, 3.0, 1.0],
            }),
        },
    ];
    for method in PressureImplementation::ALL {
        let make = || {
            LiquidTransportSimulation::with_reconstructed_surface(
                g.clone(),
                config(),
                method,
                vec![1.0, 1.0, 0.0, 0.0, 0.0, 0.0],
                Axis::Y,
            )
            .unwrap()
        };
        let mut s = make();
        let mut stages = vec![];
        for step in 1..=8 {
            let r = s
                .step(inputs(&force), |stage| {
                    if !stages.contains(&stage) {
                        stages.push(stage);
                    }
                    false
                })
                .unwrap();
            near(r.liquid.liquid_volume_after, 2.0);
            assert_eq!(s.state().liquid.stamp.version, step);
        }
        assert!(
            s.state()
                .liquid
                .fraction
                .iter()
                .any(|&f| f > 0.0 && f < 1.0)
        );
        for target in stages {
            let mut a = make();
            a.step(inputs(&force), |_| false).unwrap();
            let before = bits(&a);
            assert!(a.step(inputs(&force), |stage| stage == target).is_err());
            assert_eq!(bits(&a), before);
            a.step(inputs(&force), |_| false).unwrap();
            let mut b = make();
            b.step(inputs(&force), |_| false).unwrap();
            b.step(inputs(&force), |_| false).unwrap();
            assert_eq!(bits(&a), bits(&b));
        }
    }
}
#[test]
fn geometric_vertical_translation_crosses_layers_with_exact_volume_ledger() {
    for speed in [0.125_f32, -0.125] {
        let g = GridGeometry::new([1, 4, 1], [1.0; 3], [0.0; 3]).unwrap();
        let initial = if speed > 0.0 {
            vec![1.0, 0.25, 0.0, 0.0]
        } else {
            vec![1.0, 1.0, 0.25, 0.0]
        };
        let start = initial.iter().sum::<f64>();
        let mut geometry =
            ColumnSurfaceWorkspace::new(g.clone(), Axis::Y, &initial, stamp(1), 1 << 20).unwrap();
        let mut phase = LiquidVolumeState::new(g.clone(), 1.0, stamp(1), initial, 1 << 20).unwrap();
        let x = vec![0.0; g.face_len(Axis::X)];
        let y = vec![speed; g.face_len(Axis::Y)];
        let z = vec![0.0; g.face_len(Axis::Z)];
        for step in 1..=16 {
            let flow = LiquidFlowInterval::new(&g, stamp(2), [&x, &y, &z], phase.state().time, 0.5)
                .unwrap();
            let r = phase
                .advance_reconstructed(
                    ColumnVolumeInputs {
                        flow,
                        inlet: LiquidInlet::new(stamp(3), [[0.0; 2], [1.0, 0.0], [0.0; 2]])
                            .unwrap(),
                        source: None,
                        settings: LiquidVolumeSettings {
                            max_outward_courant: 1.0,
                            actual_divergence_limit: 1e-9,
                        },
                    },
                    &mut geometry,
                    |_| false,
                )
                .unwrap();
            let height = start + f64::from(speed) * 0.5 * f64::from(step);
            assert_eq!(r.volume.liquid_volume_after, height);
            assert_eq!(r.volume.volume_balance_error, 0.0);
            assert_eq!(
                geometry.state().heights()[0].full_layers(),
                height.floor() as usize
            );
            assert_eq!(
                geometry.state().heights()[0].top_fraction(),
                height - height.floor()
            );
        }
    }
}
#[test]
fn admission_and_memory_are_bounded_and_explicit() {
    let g = GridGeometry::new([2, 3, 1], [1.0; 3], [0.0; 3]).unwrap();
    assert!(matches!(
        ColumnSurfaceWorkspace::new(
            g.clone(),
            Axis::Y,
            &[1.0, 1.0, 0.25, 0.25, 0.25, 0.0],
            stamp(1),
            1 << 20
        ),
        Err(FreeSurfaceError::UnsupportedColumn { column: 0 })
    ));
    assert!(matches!(
        ColumnSurfaceWorkspace::new(
            g.clone(),
            Axis::Y,
            &[0.5, 1.0, 0.0, 0.0, 0.0, 0.0],
            stamp(1),
            1 << 20
        ),
        Err(FreeSurfaceError::UnresolvedColumn { column: 0 })
    ));
    let initial = vec![1.0, 1.0, 0.25, 0.25, 0.0, 0.0];
    let ordinary = LiquidTransportSimulation::new(
        g.clone(),
        config(),
        PressureImplementation::JacobiPcgV1,
        initial.clone(),
    )
    .unwrap();
    let s = LiquidTransportSimulation::with_reconstructed_surface(
        g.clone(),
        config(),
        PressureImplementation::JacobiPcgV1,
        initial.clone(),
        Axis::Y,
    )
    .unwrap();
    let required = ordinary.allocated_bytes() + 3 * 2 * std::mem::size_of::<ColumnHeight>();
    assert_eq!(s.allocated_bytes(), required);
    let mut c = config();
    c.carrier.memory_limit = required - 1;
    assert!(matches!(
        LiquidTransportSimulation::with_reconstructed_surface(
            g,
            c,
            PressureImplementation::JacobiPcgV1,
            initial,
            Axis::Y
        ),
        Err(LiquidStepError::BufferLimit { .. })
    ));
}

#[test]
fn late_unsupported_reconstruction_keeps_accepted_pressure_phase_and_both_geometries() {
    let g = GridGeometry::new([2, 4, 1], [1.0; 3], [0.0; 3]).unwrap();
    let force = [
        BodyForce {
            value: [0.0, 0.5, 0.0],
            units: ForceUnits::Acceleration,
            region: Some(ForceRegion {
                lower: [0.0; 3],
                upper: [1.0, 4.0, 1.0],
            }),
        },
        BodyForce {
            value: [0.0, -0.5, 0.0],
            units: ForceUnits::Acceleration,
            region: Some(ForceRegion {
                lower: [1.0, 0.0, 0.0],
                upper: [2.0, 4.0, 1.0],
            }),
        },
    ];
    for method in PressureImplementation::ALL {
        let make = || {
            LiquidTransportSimulation::with_reconstructed_surface(
                g.clone(),
                config(),
                method,
                vec![1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                Axis::Y,
            )
            .unwrap()
        };
        let mut s = make();
        s.step(inputs(&force), |_| false).unwrap();
        let accepted = bits(&s);
        for rates in [
            [0.0, 0.0, 0.0, 6.0, 0.0, 4.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        ] {
            let mut request = inputs(&force);
            request.source = Some(LiquidVolumeSource::new(stamp(10), &rates).unwrap());
            let mut reached = false;
            let error = s
                .step(request, |stage| {
                    reached |= matches!(stage, LiquidStepStage::Reconstruction(_));
                    false
                })
                .unwrap_err();
            assert!(
                matches!(
                    error,
                    LiquidStepError::FreeSurface(
                        FreeSurfaceError::SteepColumn { .. }
                            | FreeSurfaceError::UnsupportedColumn { .. }
                    )
                ),
                "{error:?}"
            );
            assert!(reached);
            assert_eq!(bits(&s), accepted);
        }
        s.step(inputs(&force), |_| false).unwrap();
        let mut reference = make();
        reference.step(inputs(&force), |_| false).unwrap();
        reference.step(inputs(&force), |_| false).unwrap();
        assert_eq!(bits(&s), bits(&reference));
    }
}

#[test]
fn unrepresentable_ghost_distance_coefficient_rejects_without_clamping() {
    let g = GridGeometry::new([1, 2, 1], [1.0; 3], [0.0; 3]).unwrap();
    let surface = ColumnSurfaceWorkspace::new(
        g.clone(),
        Axis::Y,
        &[0.5000000000000001, 0.0],
        stamp(1),
        1 << 20,
    )
    .unwrap();
    assert!(matches!(
        PressureOperator::with_columns(&g, 1e-300, surface.state()),
        Err(OperatorError::InvalidCoefficient)
    ));
}

#[test]
fn a_moving_column_activates_a_new_pressure_center_on_the_next_interval() {
    let g = GridGeometry::new([2, 4, 1], [1.0; 3], [0.0; 3]).unwrap();
    let force = [
        BodyForce {
            value: [0.0, 0.5, 0.0],
            units: ForceUnits::Acceleration,
            region: Some(ForceRegion {
                lower: [0.0; 3],
                upper: [1.0, 4.0, 1.0],
            }),
        },
        BodyForce {
            value: [0.0, -0.5, 0.0],
            units: ForceUnits::Acceleration,
            region: Some(ForceRegion {
                lower: [1.0, 0.0, 0.0],
                upper: [2.0, 4.0, 1.0],
            }),
        },
    ];
    for method in PressureImplementation::ALL {
        let mut s = LiquidTransportSimulation::with_reconstructed_surface(
            g.clone(),
            config(),
            method,
            vec![1.0, 1.0, 0.5, 0.5, 0.0, 0.0, 0.0, 0.0],
            Axis::Y,
        )
        .unwrap();
        s.step(inputs(&force), |_| false).unwrap();
        assert!(s.state().reconstructed_surface.unwrap().is_wet_cell(2));
        assert!(!s.state().pressure_columns.unwrap().is_wet_cell(2));
        for _ in 0..7 {
            let report = s.step(inputs(&force), |_| false).unwrap();
            near(report.liquid.liquid_volume_after, 3.0);
        }
        assert!(s.state().pressure_columns.unwrap().is_wet_cell(2));
        assert_ne!(s.state().pressure[2], 0.0);
    }
}

#[test]
fn unequal_spacing_column_hydrostatics_work_on_all_three_axes() {
    for axis in [Axis::X, Axis::Y, Axis::Z] {
        let d = match axis {
            Axis::X => 0,
            Axis::Y => 1,
            Axis::Z => 2,
        };
        let mut counts = [2_u64; 3];
        counts[d] = 4;
        let spacing = [0.5, 0.25, 0.75];
        let g = GridGeometry::new(counts, spacing, [1.0, 2.0, 3.0]).unwrap();
        let [nx, ny, nz] = g.counts();
        let mut phase = vec![0.0; g.cell_len()];
        for k in 0..nz {
            for j in 0..ny {
                for i in 0..nx {
                    let p = [i, j, k];
                    phase[g.cell_index(p).unwrap()] = if p[d] == 0 {
                        1.0
                    } else if p[d] == 1 {
                        0.75
                    } else {
                        0.0
                    };
                }
            }
        }
        let geometry =
            ColumnSurfaceWorkspace::new(g.clone(), axis, &phase, stamp(1), 1 << 20).unwrap();
        let op = PressureOperator::with_columns(&g, 2.0, geometry.state()).unwrap();
        let mut v = [
            vec![0.0; g.face_len(Axis::X)],
            vec![0.0; g.face_len(Axis::Y)],
            vec![0.0; g.face_len(Axis::Z)],
        ];
        let dims = g.face_counts(axis);
        for k in 0..dims[2] {
            for j in 0..dims[1] {
                for i in 0..dims[0] {
                    let p = [i, j, k];
                    if p[d] == 1 || p[d] == 2 {
                        v[d][g.face_index(axis, p).unwrap()] = -0.03125;
                    }
                }
            }
        }
        for method in PressureImplementation::ALL {
            let mut w =
                PressureWorkspace::with_implementation(&g, 48 * g.cell_len(), method).unwrap();
            w.solve_velocity(&op, [&v[0], &v[1], &v[2]], 0.125, settings(), || false)
                .unwrap();
            for k in 0..nz {
                for j in 0..ny {
                    for i in 0..nx {
                        let p = [i, j, k];
                        let exact = if p[d] < 2 {
                            0.5 * (1.75 - (p[d] as f64 + 0.5)) * spacing[d]
                        } else {
                            0.0
                        };
                        near(w.pressure()[g.cell_index(p).unwrap()], exact);
                    }
                }
            }
        }
    }
}

#[test]
fn rectangular_manufactured_variable_ghost_distances_match_independent_assembly() {
    let g = GridGeometry::new([3, 4, 2], [0.5, 0.25, 0.75], [0.0; 3]).unwrap();
    let fractions = [0.25, 0.75, 0.5, 0.75, 0.25, 0.5];
    let mut phase = vec![0.0; g.cell_len()];
    let mut exact = vec![0.0; g.cell_len()];
    for k in 0..2 {
        for j in 0..4 {
            for i in 0..3 {
                let cell = g.cell_index([i, j, k]).unwrap();
                phase[cell] = if j == 0 {
                    1.0
                } else if j == 1 {
                    fractions[i + 3 * k]
                } else {
                    0.0
                };
                if (j as f64 + 0.5) < 1.0 + fractions[i + 3 * k] {
                    exact[cell] = (i + 2 * j + 3 * k + 1) as f64 / 64.0;
                }
            }
        }
    }
    let surface =
        ColumnSurfaceWorkspace::new(g.clone(), Axis::Y, &phase, stamp(1), 1 << 20).unwrap();
    let op = PressureOperator::with_columns(&g, 2.0, surface.state()).unwrap();
    let h = g.spacing();
    let mut rhs = vec![0.0; g.cell_len()];
    for k in 0..2 {
        for j in 0..4 {
            for i in 0..3 {
                let p = [i, j, k];
                let row = g.cell_index(p).unwrap();
                let phi = j as f64 + 0.5 - (1.0 + fractions[i + 3 * k]);
                if phi >= 0.0 {
                    continue;
                }
                for d in 0..3 {
                    for sign in [-1, 1] {
                        let coordinate = p[d] as isize + sign;
                        if coordinate < 0 || coordinate >= g.counts()[d] as isize {
                            continue;
                        }
                        let mut q = p;
                        q[d] = coordinate as usize;
                        let other = g.cell_index(q).unwrap();
                        let neighbor_phi = q[1] as f64 + 0.5 - (1.0 + fractions[q[0] + 3 * q[2]]);
                        let area = h[(d + 1) % 3] * h[(d + 2) % 3];
                        let weight = area / (2.0 * h[d]);
                        rhs[row] += if neighbor_phi < 0.0 {
                            weight * (exact[row] - exact[other])
                        } else {
                            weight / ((-phi) / (neighbor_phi - phi)) * exact[row]
                        };
                    }
                }
            }
        }
    }
    let mut actual = vec![0.0; g.cell_len()];
    op.apply_full(&exact, &mut actual).unwrap();
    for (&a, &b) in actual.iter().zip(&rhs) {
        near(a, b);
    }
    for method in PressureImplementation::ALL {
        let mut w = PressureWorkspace::with_implementation(&g, 48 * g.cell_len(), method).unwrap();
        w.solve_rhs(&op, &rhs, 0.125, settings(), || false).unwrap();
        for (&p, &q) in w.pressure().iter().zip(&exact) {
            near(p, q);
        }
    }
}
