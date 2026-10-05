use rheon::*;
fn d(a: Axis) -> usize {
    match a {
        Axis::X => 0,
        Axis::Y => 1,
        Axis::Z => 2,
    }
}
fn coords(i: usize, n: [usize; 3]) -> [usize; 3] {
    [i % n[0], (i / n[0]) % n[1], i / (n[0] * n[1])]
}
fn fraction(g: &GridGeometry, axis: Axis, height: f64) -> Vec<f64> {
    (0..g.cell_len())
        .map(|i| (height - coords(i, g.counts())[d(axis)] as f64).clamp(0.0, 1.0))
        .collect()
}
fn surface(g: &GridGeometry, axis: Axis, height: f64) -> ColumnSurfaceWorkspace {
    ColumnSurfaceWorkspace::new(
        g.clone(),
        axis,
        &fraction(g, axis, height),
        VolumeStamp {
            id: 11,
            version: 12,
        },
        1 << 20,
    )
    .unwrap()
}
fn fields(g: &GridGeometry) -> [Vec<f32>; 3] {
    std::array::from_fn(|i| vec![0.0; g.face_len([Axis::X, Axis::Y, Axis::Z][i])])
}
fn refs(v: &[Vec<f32>; 3]) -> [&[f32]; 3] {
    [&v[0], &v[1], &v[2]]
}
fn muts(v: &mut [Vec<f32>; 3]) -> [&mut [f32]; 3] {
    let [a, b, c] = v;
    [a, b, c]
}
fn profiles(g: &GridGeometry, axis: Axis, height: f64) -> [Vec<f32>; 2] {
    let normal = d(axis);
    let tangents = match axis {
        Axis::X => [1, 2],
        Axis::Y => [0, 2],
        Axis::Z => [0, 1],
    };
    std::array::from_fn(|t| {
        (0..g.cell_len())
            .map(|index| {
                let p = coords(index, g.counts());
                if (p[normal] as f64 + 0.5) < height {
                    (0.1 * (1 + p[tangents[t]]) as f64 * (1 + p[normal]) as f64) as f32
                } else {
                    0.0
                }
            })
            .collect()
    })
}
fn profile_refs(v: &[Vec<f32>; 2]) -> [&[f32]; 2] {
    [&v[0], &v[1]]
}
fn profile_muts(v: &mut [Vec<f32>; 2]) -> [&mut [f32]; 2] {
    let [a, b] = v;
    [a, b]
}
fn config() -> LiquidTransportConfig {
    LiquidTransportConfig {
        carrier: SimulationConfig {
            density: 1.0,
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
        represented_density: 1.0,
        volume_stamp: VolumeStamp {
            id: 11,
            version: 12,
        },
        carrier_id: 17,
    }
}
fn owner(
    g: &GridGeometry,
    axis: Axis,
    height: f64,
    method: PressureImplementation,
) -> LiquidTransportSimulation {
    LiquidTransportSimulation::with_reconstructed_surface(
        g.clone(),
        config(),
        method,
        fraction(g, axis, height),
        axis,
    )
    .unwrap()
}
fn stamp(owner: &LiquidTransportSimulation) -> ColumnMacStateStamp {
    let s = owner.state();
    ColumnMacStateStamp {
        carrier: s.carrier_stamp,
        volume: s.liquid.stamp,
    }
}
fn bits(owner: &LiquidTransportSimulation) -> Vec<u64> {
    let s = owner.state();
    let mut data = s
        .pressure
        .iter()
        .chain(s.liquid.fraction)
        .map(|v| v.to_bits())
        .collect::<Vec<_>>();
    data.extend(
        s.carrier
            .x
            .iter()
            .chain(s.carrier.y)
            .chain(s.carrier.z)
            .chain(s.carrier.tracer)
            .map(|v| u64::from(v.to_bits())),
    );
    data.extend([
        s.carrier.time.to_bits(),
        s.liquid.time.to_bits(),
        s.carrier_stamp.id,
        s.carrier_stamp.version,
        s.liquid.stamp.id,
        s.liquid.stamp.version,
        u64::from(s.flat_column_mac),
    ]);
    for v in [
        s.reconstructed_surface.unwrap(),
        s.pressure_columns.unwrap(),
    ] {
        data.extend([v.stamp().id, v.stamp().version]);
        for h in v.heights() {
            data.extend([h.full_layers() as u64, h.top_fraction().to_bits()]);
        }
    }
    data
}
fn publish(
    o: &mut LiquidTransportSimulation,
    w: &mut ColumnMacWorkspace,
    u: &[Vec<f32>; 2],
) -> Result<ColumnMacPublicationReport, LiquidStepError> {
    o.materialize_flat_column_profiles(
        w,
        ColumnMacPublishInputs {
            expected: stamp(o),
            profiles: profile_refs(u),
            projection_dt: 0.125,
        },
        |_| false,
    )
}
#[test]
fn independently_assembled_partial_mass_flux_pressure_matrix_and_adjoint() {
    let g = GridGeometry::new([2, 3, 1], [1.0; 3], [0.0; 3]).unwrap();
    let s = surface(&g, Axis::Y, 1.75);
    let geom = FlatColumnMacGeometry::new(s.state(), 1.0).unwrap();
    assert_eq!(geom.dual_length(0), Some(1.0));
    assert_eq!(geom.dual_length(1), Some(0.75));
    assert_eq!(geom.cell_mass(2), Some(0.75));
    assert_eq!(geom.face_mass(Axis::X, [1, 1, 0]), Some(0.75));
    assert_eq!(geom.flux_area(Axis::X, [1, 1, 0]), Some(0.75));
    assert_eq!(geom.face_mass(Axis::X, [0, 1, 0]), Some(0.375));
    assert_eq!(geom.flux_area(Axis::X, [0, 1, 0]), Some(0.0));
    assert_eq!(geom.face_mass(Axis::Y, [0, 2, 0]), Some(0.25));
    assert_eq!(geom.flux_area(Axis::Y, [0, 2, 0]), Some(1.0));
    let op = PressureOperator::with_flat_column_masses(&g, 1.0, s.state()).unwrap();
    let a = [
        [2.0, -1.0, -1.0, 0.0, 0.0, 0.0],
        [-1.0, 2.0, 0.0, -1.0, 0.0, 0.0],
        [-1.0, 0.0, 5.75, -0.75, 0.0, 0.0],
        [0.0, -1.0, -0.75, 5.75, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 0.0, 0.0, 1.0],
    ];
    for col in 0..6 {
        let mut p = [0.0; 6];
        p[col] = 1.0;
        let mut out = [0.0; 6];
        op.apply_full(&p, &mut out).unwrap();
        for row in 0..6 {
            assert_eq!(out[row], a[row][col]);
        }
    }
    let p = [1.0, 2.0, 3.0, 4.0, 0.0, 0.0];
    let mut ap = [0.0; 6];
    op.apply_full(&p, &mut ap).unwrap();
    assert_eq!(ap, [-3.0, -1.0, 13.25, 18.75, 0.0, 0.0]);
    let zero = fields(&g);
    let mut corrected = fields(&g);
    op.correct_velocity(refs(&zero), &p, 1.0, muts(&mut corrected))
        .unwrap();
    let squared: f64 = corrected
        .iter()
        .enumerate()
        .map(|(d, field)| {
            field
                .iter()
                .enumerate()
                .map(|(i, &v)| {
                    geom.face_mass(
                        [Axis::X, Axis::Y, Axis::Z][d],
                        coords(i, g.face_counts([Axis::X, Axis::Y, Axis::Z][d])),
                    )
                    .unwrap()
                        * f64::from(v).powi(2)
                })
                .sum::<f64>()
        })
        .sum();
    assert_eq!(squared, p.iter().zip(ap).map(|(p, a)| p * a).sum::<f64>());
    let mut rhs = [0.0; 6];
    let mut divergence = [0.0; 6];
    op.build_rhs(refs(&corrected), 0.5, &mut rhs).unwrap();
    op.divergence(refs(&corrected), &mut divergence).unwrap();
    for row in 0..4 {
        assert_eq!(
            rhs[row],
            -divergence[row] * geom.cell_volume(row).unwrap() / 0.5
        );
    }
}
#[test]
fn hand_transfer_and_lossy_roundtrip_wall_ledgers_all_axes() {
    for axis in [Axis::X, Axis::Y, Axis::Z] {
        let normal = d(axis);
        let t = match axis {
            Axis::X => 1,
            _ => 0,
        };
        let mut counts = [1; 3];
        counts[normal] = 3;
        counts[t] = 2;
        let g = GridGeometry::new(counts, [1.0; 3], [0.0; 3]).unwrap();
        let s = surface(&g, axis, 1.75);
        let geom = FlatColumnMacGeometry::new(s.state(), 1.0).unwrap();
        let mut u = [vec![0.0; g.cell_len()], vec![0.0; g.cell_len()]];
        for (i, value) in u[0].iter_mut().enumerate() {
            let p = coords(i, g.counts());
            if p[normal] < 2 {
                *value = [[1.0, 3.0], [2.0, 6.0]][p[normal]][p[t]];
            }
        }
        let f: [usize; 3] = std::array::from_fn(|i| g.face_len([Axis::X, Axis::Y, Axis::Z][i]));
        let mut w = ColumnMacWorkspace::new(g.clone(), 8 * f.iter().sum::<usize>()).unwrap();
        let mut out = fields(&g);
        let r = w
            .lift(geom, profile_refs(&u), muts(&mut out), |_| false)
            .unwrap();
        assert_eq!(r.profile_mass, 3.5);
        assert_eq!(r.mac_mass, [3.5; 3]);
        assert_eq!(r.momentum_before, [10.0, 0.0]);
        assert_eq!(r.momentum_after, [5.0, 0.0]);
        assert_eq!(r.wall_impulse, [-5.0, 0.0]);
        assert_eq!(
            (
                r.kinetic_before,
                r.kinetic_after,
                r.wall_energy_removed,
                r.mixing_loss
            ),
            (20.0, 8.0, 10.0, 2.0)
        );
        assert_eq!(r.energy_error, 0.0);
        assert_eq!(r.workspace_bytes, 8 * f.iter().sum::<usize>());
        let mut back = [vec![7.0; g.cell_len()], vec![7.0; g.cell_len()]];
        let r = w
            .restrict(geom, refs(&out), profile_muts(&mut back), |_| false)
            .unwrap();
        assert_eq!(r.wall_impulse, [0.0; 2]);
        assert_eq!(r.momentum_before, r.momentum_after);
        assert_eq!(
            (r.kinetic_before, r.kinetic_after, r.mixing_loss),
            (8.0, 4.0, 4.0)
        );
        for (i, &value) in back[0].iter().enumerate() {
            let p = coords(i, g.counts());
            assert_eq!(
                value,
                if p[normal] < 2 {
                    (p[normal] + 1) as f32
                } else {
                    0.0
                }
            );
        }
        assert_eq!(back[1], vec![0.0; g.cell_len()]);
        assert!(matches!(
            ColumnMacWorkspace::new(g.clone(), 8 * f.iter().sum::<usize>() - 1),
            Err(ColumnMacError::BufferLimit { .. })
        ));
    }
}
#[test]
fn both_solvers_publish_velocity_pressure_and_geometry_revisions_all_axes() {
    for axis in [Axis::X, Axis::Y, Axis::Z] {
        for method in PressureImplementation::ALL {
            for height in [2.25, 2.5, 2.75] {
                let mut n = [2; 3];
                n[d(axis)] = 4;
                let g = GridGeometry::new(n, [1.0; 3], [0.0; 3]).unwrap();
                let mut o = owner(&g, axis, height, method);
                let mut w = ColumnMacWorkspace::new(g.clone(), 1 << 20).unwrap();
                let initial = o.state().liquid.fraction.to_vec();
                let u = profiles(&g, axis, height);
                let r = publish(&mut o, &mut w, &u).unwrap();
                let s = o.state();
                assert_eq!(r.before.volume.version, 12);
                assert_eq!(r.after.volume.version, 13);
                assert_eq!(r.after.carrier.version, 1);
                assert_eq!(s.carrier_stamp, r.after.carrier);
                assert_eq!(s.liquid.stamp, r.after.volume);
                assert_eq!(s.pressure_columns.unwrap().stamp(), r.after.volume);
                assert_eq!(s.reconstructed_surface.unwrap().stamp(), r.after.volume);
                assert_eq!(s.liquid.fraction, initial);
                assert_eq!(s.carrier.time, 0.0);
                assert_eq!(s.liquid.time, 0.0);
                assert!(s.flat_column_mac);
                assert!(s.pressure.iter().any(|p| *p != 0.0));
                assert!(r.projection.actual_divergence_max <= 1e-5);
                assert!(r.projection.energy_error.abs() <= r.projection.energy_budget);
                assert!(
                    r.projection.kinetic_after
                        <= r.projection.kinetic_before + r.projection.energy_budget
                );
                assert_eq!(r.owned_array_bytes, o.allocated_bytes());
                assert_eq!(
                    r.total_array_bytes,
                    o.allocated_bytes() + w.allocated_bytes()
                );
                let snapshot = bits(&o);
                assert!(matches!(
                    o.materialize_flat_column_profiles(
                        &mut w,
                        ColumnMacPublishInputs {
                            expected: r.before,
                            profiles: profile_refs(&u),
                            projection_dt: 0.125
                        },
                        |_| false
                    ),
                    Err(LiquidStepError::ColumnMac(ColumnMacError::StampMismatch))
                ));
                assert_eq!(bits(&o), snapshot);
                let r2 = publish(&mut o, &mut w, &u).unwrap();
                assert_eq!(r2.before, r.after);
                assert_eq!(r2.after.carrier.version, 2);
                assert_eq!(r2.after.volume.version, 14);
            }
        }
    }
}
#[test]
fn every_owner_callback_preserves_nonzero_accepted_state_and_retry() {
    let g = GridGeometry::new([2, 4, 2], [1.0; 3], [0.0; 3]).unwrap();
    let axis = Axis::Y;
    let height = 2.75;
    let u = profiles(&g, axis, height);
    let seed = |o: &mut LiquidTransportSimulation, w: &mut ColumnMacWorkspace| {
        publish(o, w, &u).unwrap();
    };
    for method in PressureImplementation::ALL {
        let mut clean = owner(&g, axis, height, method);
        let mut wc = ColumnMacWorkspace::new(g.clone(), 1 << 20).unwrap();
        seed(&mut clean, &mut wc);
        let expected = stamp(&clean);
        let mut calls = 0;
        clean
            .materialize_flat_column_profiles(
                &mut wc,
                ColumnMacPublishInputs {
                    expected,
                    profiles: profile_refs(&u),
                    projection_dt: 0.125,
                },
                |_| {
                    calls += 1;
                    false
                },
            )
            .unwrap();
        let reference = bits(&clean);
        for at in 0..calls {
            let mut o = owner(&g, axis, height, method);
            let mut w = ColumnMacWorkspace::new(g.clone(), 1 << 20).unwrap();
            seed(&mut o, &mut w);
            let saved = bits(&o);
            let mut seen = 0;
            assert!(
                o.materialize_flat_column_profiles(
                    &mut w,
                    ColumnMacPublishInputs {
                        expected,
                        profiles: profile_refs(&u),
                        projection_dt: 0.125
                    },
                    |_| {
                        let fail = seen == at;
                        seen += 1;
                        fail
                    }
                )
                .is_err()
            );
            assert_eq!(bits(&o), saved);
            publish(&mut o, &mut w, &u).unwrap();
            assert_eq!(bits(&o), reference);
        }
    }
}
#[test]
fn transfer_cancellation_all_occurrences_shape_and_scale_failures() {
    let g = GridGeometry::new([2, 4, 2], [1.0; 3], [0.0; 3]).unwrap();
    let s = surface(&g, Axis::Y, 2.75);
    let geom = FlatColumnMacGeometry::new(s.state(), 1.0).unwrap();
    let u = profiles(&g, Axis::Y, 2.75);
    let mut w = ColumnMacWorkspace::new(g.clone(), 1 << 20).unwrap();
    let mut clean = fields(&g);
    let mut calls = 0;
    w.lift(geom, profile_refs(&u), muts(&mut clean), |_| {
        calls += 1;
        false
    })
    .unwrap();
    for at in 0..calls {
        let mut out = fields(&g);
        for f in &mut out {
            f.fill(7.0);
        }
        let before = out.clone();
        let mut seen = 0;
        assert!(
            w.lift(geom, profile_refs(&u), muts(&mut out), |_| {
                let fail = seen == at;
                seen += 1;
                fail
            })
            .is_err()
        );
        assert_eq!(out, before);
        w.lift(geom, profile_refs(&u), muts(&mut out), |_| false)
            .unwrap();
        assert_eq!(out, clean);
    }
    let mut reference = [vec![0.0; g.cell_len()], vec![0.0; g.cell_len()]];
    let mut calls = 0;
    let r = w
        .restrict(geom, refs(&clean), profile_muts(&mut reference), |_| {
            calls += 1;
            false
        })
        .unwrap();
    for at in 0..calls {
        let mut out = [vec![7.0; g.cell_len()], vec![7.0; g.cell_len()]];
        let before = out.clone();
        let mut seen = 0;
        assert!(
            w.restrict(geom, refs(&clean), profile_muts(&mut out), |_| {
                let fail = seen == at;
                seen += 1;
                fail
            })
            .is_err()
        );
        assert_eq!(out, before);
        assert_eq!(
            w.restrict(geom, refs(&clean), profile_muts(&mut out), |_| false)
                .unwrap(),
            r
        );
        assert_eq!(out, reference);
    }
    let mut out = fields(&g);
    for f in &mut out {
        f.fill(7.0);
    }
    let before = out.clone();
    let mut bad = u.clone();
    bad[0][0] = f32::from_bits(1);
    assert_eq!(
        w.lift(geom, profile_refs(&bad), muts(&mut out), |_| false),
        Err(ColumnMacError::InvalidField)
    );
    assert_eq!(out, before);
    bad = u.clone();
    bad[0][0] = f32::MIN_POSITIVE;
    bad[0][1] = -f32::MIN_POSITIVE;
    bad[0][2] = f32::MIN_POSITIVE;
    bad[0][3] = 0.0;
    assert!(
        w.lift(geom, profile_refs(&bad), muts(&mut out), |_| false)
            .is_err()
    );
    assert_eq!(out, before);
    assert!(FlatColumnMacGeometry::new(s.state(), f64::MIN_POSITIVE).is_err());
    assert!(FlatColumnMacGeometry::new(s.state(), f64::MAX).is_err());
}
#[test]
fn owner_export_revision_omission_and_dynamic_refusal() {
    let g = GridGeometry::new([2, 4, 2], [1.0; 3], [0.0; 3]).unwrap();
    let mut o = owner(&g, Axis::Y, 2.75, PressureImplementation::JacobiPcgV1);
    let mut w = ColumnMacWorkspace::new(g.clone(), 1 << 20).unwrap();
    let u = profiles(&g, Axis::Y, 2.75);
    let pubr = publish(&mut o, &mut w, &u).unwrap();
    let saved = bits(&o);
    let mut out = [vec![7.0; g.cell_len()], vec![7.0; g.cell_len()]];
    let r = o
        .export_flat_column_profiles(&mut w, pubr.after, profile_muts(&mut out), |_| false)
        .unwrap();
    assert_eq!(r.source, pubr.after);
    assert_eq!(r.transfer.geometry, pubr.after.volume);
    assert!(r.transfer.omitted_normal_energy > 0.0);
    assert_eq!(bits(&o), saved);
    let output = out.clone();
    assert!(matches!(
        o.export_flat_column_profiles(&mut w, pubr.before, profile_muts(&mut out), |_| false),
        Err(LiquidStepError::ColumnMac(ColumnMacError::StampMismatch))
    ));
    assert_eq!(out, output);
    let inputs = LiquidStepInputs {
        requested_dt: 0.125,
        smoke_source: None,
        forces: &[],
        inlet: LiquidInlet::new(VolumeStamp { id: 99, version: 0 }, [[0.0; 2]; 3]).unwrap(),
        source: None,
        volume: LiquidVolumeSettings {
            max_outward_courant: 1.0,
            actual_divergence_limit: 1e-5,
        },
    };
    assert!(matches!(
        o.step(inputs, |_| false),
        Err(LiquidStepError::ColumnMac(ColumnMacError::UnsupportedState))
    ));
    let mut viscosity = ViscosityWorkspace::new(g, 1 << 20).unwrap();
    assert!(
        o.step_viscous(inputs, &mut viscosity, 1.0, |_| false)
            .is_err()
    );
    assert_eq!(bits(&o), saved);
}
#[test]
fn owner_nonflat_paused_version_shape_and_late_projection_failures() {
    let g = GridGeometry::new([2, 4, 2], [1.0; 3], [0.0; 3]).unwrap();
    let axis = Axis::Y;
    let u = profiles(&g, axis, 2.75);
    let mut w = ColumnMacWorkspace::new(g.clone(), 1 << 20).unwrap();
    let mut o = owner(&g, axis, 2.75, PressureImplementation::JacobiPcgV1);
    let saved = bits(&o);
    let mut bad = u.clone();
    bad[0].pop();
    assert!(publish(&mut o, &mut w, &bad).is_err());
    assert_eq!(bits(&o), saved);
    o.set_paused(true);
    assert!(matches!(
        publish(&mut o, &mut w, &u),
        Err(LiquidStepError::ColumnMac(ColumnMacError::Paused))
    ));
    assert_eq!(bits(&o), saved);
    o.set_paused(false);
    let mut f = fraction(&g, axis, 2.75);
    for (index, v) in f.iter_mut().enumerate() {
        let p = coords(index, g.counts());
        if p[0] == 0 && p[1] == 2 {
            *v = 0.25;
        }
    }
    let mut nonflat = LiquidTransportSimulation::with_reconstructed_surface(
        g.clone(),
        config(),
        PressureImplementation::JacobiPcgV1,
        f,
        axis,
    )
    .unwrap();
    let saved = bits(&nonflat);
    assert!(matches!(
        publish(&mut nonflat, &mut w, &u),
        Err(LiquidStepError::ColumnMac(
            ColumnMacError::UnsupportedHeights
        ))
    ));
    assert_eq!(bits(&nonflat), saved);
    let mut cfg = config();
    cfg.volume_stamp.version = u64::MAX;
    let mut overflow = LiquidTransportSimulation::with_reconstructed_surface(
        g.clone(),
        cfg,
        PressureImplementation::JacobiPcgV1,
        fraction(&g, axis, 2.75),
        axis,
    )
    .unwrap();
    let saved = bits(&overflow);
    assert!(matches!(
        publish(&mut overflow, &mut w, &u),
        Err(LiquidStepError::ColumnMac(ColumnMacError::VersionOverflow))
    ));
    assert_eq!(bits(&overflow), saved);
    cfg = config();
    cfg.carrier.actual_divergence_limit = 0.0;
    let make = || {
        LiquidTransportSimulation::with_reconstructed_surface(
            g.clone(),
            cfg,
            PressureImplementation::JacobiPcgV1,
            fraction(&g, axis, 2.75),
            axis,
        )
        .unwrap()
    };
    let mut late = make();
    let saved = bits(&late);
    let mut reached = false;
    let expected = stamp(&late);
    assert!(
        late.materialize_flat_column_profiles(
            &mut w,
            ColumnMacPublishInputs {
                expected,
                profiles: profile_refs(&u),
                projection_dt: 0.125
            },
            |s| {
                reached |= s == ColumnMacStage::BeforeProjectionAcceptance;
                false
            }
        )
        .is_err()
    );
    assert!(reached);
    assert_eq!(bits(&late), saved);
    let zero = std::array::from_fn(|_| vec![0.0; g.cell_len()]);
    publish(&mut late, &mut w, &zero).unwrap();
    let mut clean = make();
    let mut wc = ColumnMacWorkspace::new(g, 1 << 20).unwrap();
    publish(&mut clean, &mut wc, &zero).unwrap();
    assert_eq!(bits(&late), bits(&clean));
}
#[test]
fn single_wet_node_decimal_geometry_phase_mass_and_legacy_transition_guard() {
    let g = GridGeometry::new([2, 4, 2], [1.0, 0.1, 0.1], [0.0; 3]).unwrap();
    for height in [0.75, 2.6] {
        let mut o = owner(&g, Axis::Y, height, PressureImplementation::JacobiPcgV1);
        let mut w = ColumnMacWorkspace::new(g.clone(), 1 << 20).unwrap();
        let u = profiles(&g, Axis::Y, height);
        let r = publish(&mut o, &mut w, &u).unwrap();
        assert!(r.phase_mass_error.abs() <= r.phase_mass_budget);
        assert_eq!(r.after.volume, o.state().liquid.stamp);
    }
    let mut old = owner(&g, Axis::Y, 2.75, PressureImplementation::JacobiPcgV1);
    let inputs = LiquidStepInputs {
        requested_dt: 0.125,
        smoke_source: None,
        forces: &[],
        inlet: LiquidInlet::new(VolumeStamp { id: 99, version: 0 }, [[0.0; 2]; 3]).unwrap(),
        source: None,
        volume: LiquidVolumeSettings {
            max_outward_courant: 1.0,
            actual_divergence_limit: 1e-5,
        },
    };
    old.step(inputs, |_| false).unwrap();
    let saved = bits(&old);
    let mut w = ColumnMacWorkspace::new(g.clone(), 1 << 20).unwrap();
    let u = profiles(&g, Axis::Y, 2.75);
    assert!(matches!(
        publish(&mut old, &mut w, &u),
        Err(LiquidStepError::ColumnMac(ColumnMacError::UnsupportedState))
    ));
    assert_eq!(bits(&old), saved);
}
