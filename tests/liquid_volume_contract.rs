use rheon::{
    Axis, GridGeometry, LiquidFlowInterval, LiquidInlet, LiquidOccupancy, LiquidVolumeError,
    LiquidVolumeSettings, LiquidVolumeSource, LiquidVolumeState, VolumeStage, VolumeStamp,
};
fn stamp(id: u64, version: u64) -> VolumeStamp {
    VolumeStamp { id, version }
}
fn settings() -> LiquidVolumeSettings {
    LiquidVolumeSettings {
        max_outward_courant: 1.0,
        actual_divergence_limit: 1e-9,
    }
}
fn fields(g: &GridGeometry) -> [Vec<f32>; 3] {
    [Axis::X, Axis::Y, Axis::Z].map(|a| vec![0.0; g.face_len(a)])
}
fn refs(v: &[Vec<f32>; 3]) -> [&[f32]; 3] {
    [&v[0], &v[1], &v[2]]
}
fn inlet(alpha: [[f64; 2]; 3]) -> LiquidInlet {
    LiquidInlet::new(stamp(4, 2), alpha).unwrap()
}
fn state(g: &GridGeometry, values: Vec<f64>) -> LiquidVolumeState {
    LiquidVolumeState::new(g.clone(), 1000.0, stamp(1, 0), values, 1024 * 1024).unwrap()
}
fn snapshot(s: &LiquidVolumeState) -> (Vec<u64>, VolumeStamp, u64) {
    (
        s.state().fraction.iter().map(|v| v.to_bits()).collect(),
        s.state().stamp,
        s.state().time.to_bits(),
    )
}
fn near(a: f64, b: f64) {
    assert!(
        (a - b).abs() <= 2e-12 * a.abs().max(b.abs()).max(1.0),
        "{a} != {b}"
    );
}
#[test]
fn signed_zero_is_valid_empty_liquid_with_zero_volume_transfer() {
    let g = GridGeometry::new([1; 3], [1.0; 3], [0.0; 3]).unwrap();
    let mut s = state(&g, vec![-0.0]);
    let v = [vec![-0.0, 0.0], vec![-0.0, 0.0], vec![-0.0, 0.0]];
    let flow = LiquidFlowInterval::new(&g, stamp(2, 0), refs(&v), 0.0, 0.5).unwrap();
    let r = s
        .advance(flow, inlet([[-0.0, 0.0]; 3]), None, settings(), |_| false)
        .unwrap();
    assert_eq!(r.liquid_volume_after, 0.0);
    assert_eq!(s.state().fraction[0], 0.0);
    assert_eq!(s.occupancy(0), Some(LiquidOccupancy::Dry));
}

#[test]
fn signed_all_axis_upwind_inlet_outlet_matches_hand_volume_and_occupancy() {
    for axis in 0..3 {
        for speed in [-0.5_f32, 0.5] {
            let mut counts = [1; 3];
            counts[axis] = 4;
            let g = GridGeometry::new(counts, [1.0; 3], [2.0, 3.0, 4.0]).unwrap();
            let mut s = state(&g, vec![0.0, 0.25, 0.75, 1.0]);
            let mut v = fields(&g);
            v[axis].fill(speed);
            let before = v.clone();
            let mut alpha = [[0.0; 2]; 3];
            alpha[axis] = [0.5, 0.125];
            let b = inlet(alpha);
            let flow = LiquidFlowInterval::new(&g, stamp(2, 3), refs(&v), 0.0, 1.0).unwrap();
            let r = s.advance(flow, b, None, settings(), |_| false).unwrap();
            let expected = if speed > 0.0 {
                [0.25, 0.125, 0.5, 0.875]
            } else {
                [0.125, 0.5, 0.875, 0.5625]
            };
            assert_eq!(s.state().fraction, expected);
            assert_eq!(v, before);
            assert_eq!(r.flow, flow.stamp());
            assert_eq!(r.inlet, b.stamp());
            assert_eq!(r.stamp, stamp(1, 1));
            assert_eq!(r.time, 1.0);
            assert_eq!(r.volume_balance_error, 0.0);
            assert_eq!(r.actual_divergence_max, 0.0);
            assert_eq!(r.max_outward_courant, 0.5);
            if speed > 0.0 {
                assert_eq!(r.inward_boundary_volume, 0.25);
                assert_eq!(r.outward_boundary_volume, 0.5);
                assert_eq!(r.liquid_volume_after, 1.75);
            } else {
                assert_eq!(r.inward_boundary_volume, 0.0625);
                assert_eq!(r.outward_boundary_volume, 0.0);
                assert_eq!(r.liquid_volume_after, 2.0625);
            }
            assert_eq!(r.liquid_mass_after, 1000.0 * r.liquid_volume_after);
            assert_eq!(r.dry_cells, 0);
            assert_eq!(r.mixed_cells, 4);
            assert_eq!(r.full_cells, 0);
            assert_eq!(s.occupancy(4), None);
            assert_eq!(s.occupancy(0), Some(LiquidOccupancy::Mixed));
        }
    }
}
#[test]
fn signed_sources_are_volume_per_second_and_density_scales_mass_only() {
    let g = GridGeometry::new([2, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let v = fields(&g);
    let flow = LiquidFlowInterval::new(&g, stamp(2, 0), refs(&v), 0.0, 0.5).unwrap();
    let rates = [0.25, -0.125];
    let source = LiquidVolumeSource::new(stamp(3, 7), &rates).unwrap();
    for density in [1.0, 1000.0] {
        let mut s = LiquidVolumeState::new(g.clone(), density, stamp(1, 0), vec![0.25, 0.75], 1024)
            .unwrap();
        let r = s
            .advance(flow, inlet([[0.0; 2]; 3]), Some(source), settings(), |_| {
                false
            })
            .unwrap();
        assert_eq!(s.state().fraction, [0.375, 0.6875]);
        assert_eq!(r.source, Some(stamp(3, 7)));
        assert_eq!(r.source_volume, 0.0625);
        assert_eq!(r.liquid_volume_before, 1.0);
        assert_eq!(r.liquid_volume_after, 1.0625);
        assert_eq!(r.volume_balance_error, 0.0);
        assert_eq!(r.liquid_mass_after, density * 1.0625);
        assert_eq!(rates, [0.25, -0.125]);
    }
}
#[test]
fn closed_anisotropic_circulation_reuses_shared_faces_and_preserves_volume() {
    let g = GridGeometry::new([2; 3], [0.5, 1.0, 2.0], [0.0; 3]).unwrap();
    let mut v = fields(&g);
    for z in 0..2 {
        v[0][g.face_index(Axis::X, [1, 0, z]).unwrap()] = 0.25;
        v[0][g.face_index(Axis::X, [1, 1, z]).unwrap()] = -0.25;
        v[1][g.face_index(Axis::Y, [0, 1, z]).unwrap()] = -0.5;
        v[1][g.face_index(Axis::Y, [1, 1, z]).unwrap()] = 0.5;
    }
    let mut s = state(&g, vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.25]);
    let flow = LiquidFlowInterval::new(&g, stamp(2, 0), refs(&v), 0.0, 0.5).unwrap();
    let r = s
        .advance(flow, inlet([[1.0; 2]; 3]), None, settings(), |_| false)
        .unwrap();
    assert_eq!(
        s.state().fraction,
        [0.75, 0.25, 0.0, 0.0, 0.0, 0.375, 0.0625, 0.3125]
    );
    assert_eq!(r.liquid_volume_before, 1.75);
    assert_eq!(r.liquid_volume_after, 1.75);
    assert_eq!(r.inward_boundary_volume, 0.0);
    assert_eq!(r.outward_boundary_volume, 0.0);
    assert_eq!(r.actual_divergence_max, 0.0);
    assert_eq!(r.volume_balance_error, 0.0);
}
#[test]
fn oblique_constant_fraction_preserves_constant_and_balanced_boundary_transfer() {
    let g = GridGeometry::new([4; 3], [0.25, 0.5, 1.0], [0.0; 3]).unwrap();
    let mut v = fields(&g);
    for (field, speed) in v.iter_mut().zip([0.125, 0.25, 0.5]) {
        field.fill(speed);
    }
    let mut s = state(&g, vec![0.375; 64]);
    let flow = LiquidFlowInterval::new(&g, stamp(2, 0), refs(&v), 0.0, 0.5).unwrap();
    let r = s
        .advance(flow, inlet([[0.375; 2]; 3]), None, settings(), |_| false)
        .unwrap();
    assert!(s.state().fraction.iter().all(|&f| f == 0.375));
    assert_eq!(r.liquid_volume_before, 3.0);
    assert_eq!(r.liquid_volume_after, 3.0);
    assert_eq!(r.inward_boundary_volume, 0.5625);
    assert_eq!(r.outward_boundary_volume, 0.5625);
    assert_eq!(r.max_outward_courant, 0.75);
    assert_eq!(r.volume_balance_error, 0.0);
}
#[test]
fn unsplit_three_dimensional_transfer_is_bounded_but_diffuses_geometric_cube() {
    let g = GridGeometry::new([3; 3], [1.0; 3], [0.0; 3]).unwrap();
    let mut initial = vec![0.0; 27];
    initial[g.cell_index([1; 3]).unwrap()] = 1.0;
    let mut s = state(&g, initial);
    let mut v = fields(&g);
    for field in &mut v {
        field.fill(0.25);
    }
    let flow = LiquidFlowInterval::new(&g, stamp(2, 0), refs(&v), 0.0, 1.0).unwrap();
    let r = s
        .advance(flow, inlet([[0.0; 2]; 3]), None, settings(), |_| false)
        .unwrap();
    let mut expected = vec![0.0; 27];
    for p in [[1, 1, 1], [2, 1, 1], [1, 2, 1], [1, 1, 2]] {
        expected[g.cell_index(p).unwrap()] = 0.25;
    }
    assert_eq!(s.state().fraction, expected);
    assert_eq!(r.liquid_volume_after, 1.0);
    assert_eq!(r.volume_balance_error, 0.0);
    let mut error = 0.0;
    for z in 0..3 {
        for y in 0..3 {
            for x in 0..3 {
                let p = [x, y, z];
                let exact = p
                    .into_iter()
                    .map(|i| ((i + 1) as f64).min(2.25) - (i as f64).max(1.25))
                    .map(|overlap| overlap.max(0.0))
                    .product::<f64>();
                error += (s.state().fraction[g.cell_index(p).unwrap()] - exact).abs();
            }
        }
    }
    near(error, 0.65625);
    assert_eq!(r.full_cells, 0);
    assert_eq!(r.mixed_cells, 4);
}
fn choose(n: usize, r: usize) -> u64 {
    let r = r.min(n - r);
    let mut value = 1_u64;
    for k in 0..r {
        value = value * (n - k) as u64 / (k + 1) as u64;
    }
    value
}
#[test]
fn translated_slab_has_exact_binomial_oracle_and_independent_spatial_refinement() {
    let mut errors = vec![];
    for (n, courant) in [(16, 0.5), (32, 0.5), (64, 0.5), (64, 0.25)] {
        let h = 1.0 / n as f64;
        let g = GridGeometry::new([n, 1, 1], [h, 1.0, 1.0], [0.0; 3]).unwrap();
        let initial: Vec<f64> = (0..n)
            .map(|i| if i >= n / 4 && i < n / 2 { 1.0 } else { 0.0 })
            .collect();
        let mut s = state(&g, initial.clone());
        let mut v = fields(&g);
        v[0].fill(0.25);
        let dt = courant * h / 0.25;
        let steps = (0.5 / dt) as usize;
        let mut max_balance = 0.0_f64;
        for k in 0..steps {
            let flow =
                LiquidFlowInterval::new(&g, stamp(2, k as u64), refs(&v), s.state().time, dt)
                    .unwrap();
            let r = s
                .advance(flow, inlet([[0.0; 2]; 3]), None, settings(), |_| false)
                .unwrap();
            max_balance = max_balance.max(r.volume_balance_error.abs());
        }
        let mut error = 0.0;
        let mut centroid = 0.0;
        for (i, &value) in s.state().fraction.iter().enumerate() {
            let binomial = (0..=steps)
                .filter(|&r| r <= i)
                .map(|r| {
                    choose(steps, r) as f64
                        * courant.powi(r as i32)
                        * (1.0 - courant).powi((steps - r) as i32)
                        * initial[i - r]
                })
                .sum::<f64>();
            near(value, binomial);
            let exact = (((i + 1) as f64 * h).min(0.625) - (i as f64 * h).max(0.375)).max(0.0) / h;
            error += h * (value - exact).abs();
            centroid += h * value * (i as f64 + 0.5) * h;
        }
        near(s.state().fraction.iter().sum::<f64>() * h, 0.25);
        near(centroid / 0.25, 0.5);
        assert!(max_balance < 1e-14);
        println!(
            "slab n={n} courant={courant} steps={steps} l1_volume_error={error:.17e} max_balance={max_balance:.17e}"
        );
        errors.push(error);
    }
    assert!(errors[1] < errors[0] && errors[2] < errors[1]);
    assert!(errors[3] > errors[2]);
}
#[test]
fn thin_positive_fractions_and_merging_supports_are_not_erased_by_clamping() {
    let g = GridGeometry::new([8, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let mut initial = vec![0.0; 8];
    initial[2] = 1e-12;
    initial[4] = 1e-12;
    let mut s = state(&g, initial);
    let mut v = fields(&g);
    v[0].fill(0.5);
    for k in 0..2 {
        let flow =
            LiquidFlowInterval::new(&g, stamp(2, k), refs(&v), s.state().time, 0.25).unwrap();
        let r = s
            .advance(flow, inlet([[0.0; 2]; 3]), None, settings(), |_| false)
            .unwrap();
        assert!((r.liquid_volume_after - 2e-12).abs() < 1e-25);
        assert!(r.volume_balance_error.abs() <= r.volume_rounding_budget);
    }
    assert!(s.state().fraction[2..=6].iter().all(|&f| f > 0.0));
    assert!((s.state().fraction[4] - 0.78125e-12).abs() < 1e-26);
    assert_eq!(s.occupancy(4), Some(LiquidOccupancy::Mixed));
}
#[test]
fn courant_divergence_and_raw_source_bounds_fail_without_changing_accepted_state() {
    let g = GridGeometry::new([1; 3], [1.0; 3], [0.0; 3]).unwrap();
    let mut s = state(&g, vec![0.25]);
    let mut v = fields(&g);
    v[0].fill(2.0);
    let before = snapshot(&s);
    let flow = LiquidFlowInterval::new(&g, stamp(2, 0), refs(&v), 0.0, 1.0).unwrap();
    assert!(matches!(
        s.advance(flow, inlet([[0.25; 2]; 3]), None, settings(), |_| false),
        Err(LiquidVolumeError::CourantLimit { actual: 2.0, .. })
    ));
    assert_eq!(snapshot(&s), before);
    let still = fields(&g);
    let flow = LiquidFlowInterval::new(&g, stamp(2, 0), refs(&still), 0.0, 1.0).unwrap();
    for rate in [1.0, -1.0] {
        let rates = [rate];
        let source = LiquidVolumeSource::new(stamp(3, 0), &rates).unwrap();
        assert!(matches!(
            s.advance(flow, inlet([[0.0; 2]; 3]), Some(source), settings(), |_| {
                false
            }),
            Err(LiquidVolumeError::FractionBounds { .. })
        ));
        assert_eq!(snapshot(&s), before);
    }
    s.advance(flow, inlet([[0.0; 2]; 3]), None, settings(), |_| false)
        .unwrap();
    let g = GridGeometry::new([2, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let mut s = state(&g, vec![0.25, 0.5]);
    let mut v = fields(&g);
    v[0][1] = 0.25;
    let before = snapshot(&s);
    let flow = LiquidFlowInterval::new(&g, stamp(2, 0), refs(&v), 0.0, 1.0).unwrap();
    assert!(matches!(
        s.advance(flow, inlet([[0.0; 2]; 3]), None, settings(), |_| false),
        Err(LiquidVolumeError::DivergenceLimit { actual: 0.25, .. })
    ));
    assert_eq!(snapshot(&s), before);
    let relaxed = LiquidVolumeSettings {
        actual_divergence_limit: 0.25,
        ..settings()
    };
    let r = s
        .advance(flow, inlet([[0.0; 2]; 3]), None, relaxed, |_| false)
        .unwrap();
    assert_eq!(r.liquid_volume_after, 0.75);
    let mut full = state(&g, vec![1.0; 2]);
    assert!(matches!(
        full.advance(flow, inlet([[1.0; 2]; 3]), None, relaxed, |_| false),
        Err(LiquidVolumeError::FractionBounds { fraction: 1.25, .. })
    ));
}
#[test]
fn partial_flux_and_cell_cancellation_preserve_bits_and_retry_matches_fresh() {
    let g = GridGeometry::new([3, 2, 2], [1.0, 2.0, 4.0], [0.0; 3]).unwrap();
    let initial: Vec<f64> = (0..12).map(|i| 0.1 + 0.05 * i as f64).collect();
    let mut v = fields(&g);
    for (field, speed) in v.iter_mut().zip([0.125, 0.25, 0.5]) {
        field.fill(speed);
    }
    let rates = [0.01; 12];
    let source = LiquidVolumeSource::new(stamp(3, 5), &rates).unwrap();
    let b = inlet([[0.2, 0.8]; 3]);
    let first = LiquidFlowInterval::new(&g, stamp(2, 0), refs(&v), 0.0, 0.25).unwrap();
    let second = LiquidFlowInterval::new(&g, stamp(2, 1), refs(&v), 0.25, 0.25).unwrap();
    for stage in [
        VolumeStage::BeforeFlux,
        VolumeStage::FaceFluxSlice,
        VolumeStage::CellUpdateSlice,
        VolumeStage::BeforeCommit,
    ] {
        let mut s = state(&g, initial.clone());
        s.advance(first, b, Some(source), settings(), |_| false)
            .unwrap();
        let before = snapshot(&s);
        let mut count = 0;
        let error = s
            .advance(second, b, Some(source), settings(), |at| {
                if at == stage {
                    count += 1;
                    return count
                        == if matches!(
                            stage,
                            VolumeStage::FaceFluxSlice | VolumeStage::CellUpdateSlice
                        ) {
                            2
                        } else {
                            1
                        };
                }
                false
            })
            .unwrap_err();
        assert_eq!(error, LiquidVolumeError::Cancelled { stage });
        assert_eq!(snapshot(&s), before);
        let retry = s
            .advance(second, b, Some(source), settings(), |_| false)
            .unwrap();
        let mut fresh = state(&g, initial.clone());
        fresh
            .advance(first, b, Some(source), settings(), |_| false)
            .unwrap();
        let expected = fresh
            .advance(second, b, Some(source), settings(), |_| false)
            .unwrap();
        assert_eq!(snapshot(&s), snapshot(&fresh));
        assert_eq!(retry.volume_balance_error, expected.volume_balance_error);
        assert_eq!(rates, [0.01; 12]);
    }
}
#[test]
fn geometry_interval_source_identity_and_version_limits_reject_before_callbacks() {
    let g = GridGeometry::new([1; 3], [1.0; 3], [0.0; 3]).unwrap();
    let other = GridGeometry::new([1; 3], [1.0; 3], [1.0, 0.0, 0.0]).unwrap();
    let v = fields(&g);
    let mut s = state(&g, vec![0.25]);
    let before = snapshot(&s);
    for (grid, start, error) in [
        (&other, 0.0, LiquidVolumeError::GeometryMismatch),
        (&g, 0.5, LiquidVolumeError::TimeMismatch),
    ] {
        let flow = LiquidFlowInterval::new(grid, stamp(2, 0), refs(&v), start, 0.5).unwrap();
        let mut count = 0;
        assert_eq!(
            s.advance(flow, inlet([[0.0; 2]; 3]), None, settings(), |_| {
                count += 1;
                false
            })
            .unwrap_err(),
            error
        );
        assert_eq!(count, 0);
        assert_eq!(snapshot(&s), before);
    }
    let flow = LiquidFlowInterval::new(&g, stamp(2, 0), refs(&v), 0.0, 0.5).unwrap();
    let source = LiquidVolumeSource::new(stamp(3, 0), &[]).unwrap();
    assert_eq!(
        s.advance(flow, inlet([[0.0; 2]; 3]), Some(source), settings(), |_| {
            false
        })
        .unwrap_err(),
        LiquidVolumeError::LengthMismatch
    );
    let mut exhausted =
        LiquidVolumeState::new(g.clone(), 1000.0, stamp(1, u64::MAX), vec![0.25], 64).unwrap();
    assert_eq!(
        exhausted
            .advance(flow, inlet([[0.0; 2]; 3]), None, settings(), |_| false)
            .unwrap_err(),
        LiquidVolumeError::VersionOverflow
    );
}
#[test]
fn actual_owned_capacity_invalid_inputs_and_resolved_clock_are_checked() {
    let g = GridGeometry::new([1; 3], [1.0; 3], [0.0; 3]).unwrap();
    assert_eq!(
        LiquidVolumeState::new(g.clone(), 1.0, stamp(1, 0), vec![0.0], 64)
            .unwrap()
            .allocated_bytes(),
        64
    );
    assert!(matches!(
        LiquidVolumeState::new(g.clone(), 1.0, stamp(1, 0), vec![0.0], 63),
        Err(LiquidVolumeError::BufferLimit {
            required: 64,
            limit: 63
        })
    ));
    let mut reserved = Vec::with_capacity(8);
    reserved.push(0.25);
    assert_eq!(
        LiquidVolumeState::new(g.clone(), 1.0, stamp(1, 0), reserved, 120)
            .unwrap()
            .allocated_bytes(),
        120
    );
    for density in [0.0, -1.0, f64::INFINITY] {
        assert!(matches!(
            LiquidVolumeState::new(g.clone(), density, stamp(1, 0), vec![0.0], 64),
            Err(LiquidVolumeError::InvalidDensity)
        ));
    }
    for value in [-0.1, 1.1, f64::NAN] {
        assert!(matches!(
            LiquidVolumeState::new(g.clone(), 1.0, stamp(1, 0), vec![value], 64),
            Err(LiquidVolumeError::InvalidFraction { cell: 0 })
        ));
        assert!(LiquidInlet::new(stamp(4, 0), [[value; 2]; 3]).is_err());
    }
    assert!(LiquidVolumeSource::new(stamp(3, 0), &[f64::NAN]).is_err());
    let mut v = fields(&g);
    assert!(matches!(
        LiquidFlowInterval::new(&g, stamp(2, 0), refs(&v), f64::MAX, 1.0),
        Err(LiquidVolumeError::TimeResolution)
    ));
    assert!(LiquidFlowInterval::new(&g, stamp(2, 0), refs(&v), 0.0, 0.0).is_err());
    v[0][0] = f32::NAN;
    assert!(matches!(
        LiquidFlowInterval::new(&g, stamp(2, 0), refs(&v), 0.0, 0.5),
        Err(LiquidVolumeError::NonFiniteVelocity)
    ));
    v[0][0] = 0.0;
    let flow = LiquidFlowInterval::new(&g, stamp(2, 0), refs(&v), 0.0, 0.5).unwrap();
    let mut s = state(&g, vec![0.25]);
    for bad in [
        LiquidVolumeSettings {
            max_outward_courant: 1.01,
            ..settings()
        },
        LiquidVolumeSettings {
            actual_divergence_limit: f64::NAN,
            ..settings()
        },
    ] {
        assert_eq!(
            s.advance(flow, inlet([[0.0; 2]; 3]), None, bad, |_| false)
                .unwrap_err(),
            LiquidVolumeError::InvalidSettings
        );
    }
}
#[test]
fn unrepresentable_initial_amount_or_nonzero_face_transfer_cannot_disappear() {
    let tiny = GridGeometry::new([1; 3], [1e-100; 3], [0.0; 3]).unwrap();
    assert!(matches!(
        LiquidVolumeState::new(tiny, 1.0, stamp(1, 0), vec![1e-30], 64),
        Err(LiquidVolumeError::ArithmeticFailure)
    ));
    let g = GridGeometry::new([1; 3], [1e100, 1e-150, 1e-150], [0.0; 3]).unwrap();
    let mut s = state(&g, vec![0.0]);
    let before = snapshot(&s);
    let mut v = fields(&g);
    v[0].fill(1e-30);
    let flow = LiquidFlowInterval::new(&g, stamp(2, 0), refs(&v), 0.0, 1.0).unwrap();
    assert_eq!(
        s.advance(flow, inlet([[1.0; 2]; 3]), None, settings(), |_| false)
            .unwrap_err(),
        LiquidVolumeError::ArithmeticFailure
    );
    assert_eq!(snapshot(&s), before);
}
#[test]
fn both_accepted_pressure_methods_supply_read_only_carrier_not_free_surface_feedback() {
    use rheon::{
        BoxFluxStamp, BoxFluxStepBoundary, BoxFluxStepWorkspace, BoxFluxTracerPolicy,
        PrescribedBoxFlux, PressureImplementation, PressureSettings, Simulation, SimulationConfig,
    };
    for method in PressureImplementation::ALL {
        let g = GridGeometry::new([3, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
        let config = SimulationConfig {
            density: 1000.0,
            memory_limit: 1024 * 1024,
            pressure: PressureSettings {
                relative_residual: 1e-12,
                absolute_residual: 1e-12,
                divergence_limit: 1e-9,
                max_iterations: 1000,
            },
            actual_divergence_limit: 1e-9,
            max_courant: 1.0,
        };
        let mut carrier = Simulation::with_implementation(g.clone(), config, method).unwrap();
        let mut workspace =
            BoxFluxStepWorkspace::new(g.clone(), 1000.0, 1024 * 1024, method).unwrap();
        let b = BoxFluxStepBoundary {
            flux: PrescribedBoxFlux::new(
                BoxFluxStamp { id: 9, version: 0 },
                [[-0.25, 0.25], [0.0; 2], [0.0; 2]],
            )
            .unwrap(),
            tracer: BoxFluxTracerPolicy::ClampedAppearance,
        };
        let r = carrier
            .step_with_box_flux(0.5, None, &[], &mut workspace, b, |_| false)
            .unwrap();
        let velocity = carrier.state();
        let flow = LiquidFlowInterval::new(
            &g,
            stamp(2, velocity.generation),
            [velocity.x, velocity.y, velocity.z],
            0.0,
            r.step.step.dt,
        )
        .unwrap();
        let mut volume = state(&g, vec![0.0, 1.0, 0.0]);
        let report = volume
            .advance(flow, inlet([[0.0; 2]; 3]), None, settings(), |_| false)
            .unwrap();
        assert_eq!(volume.state().fraction, [0.0, 0.875, 0.125]);
        assert_eq!(report.liquid_volume_after, 1.0);
        assert_eq!(report.time, velocity.time);
        assert_eq!(report.actual_divergence_max, 0.0);
        let mut failing = state(&g, vec![1.0; 3]);
        let source = [1.0; 3];
        let source = LiquidVolumeSource::new(stamp(3, 0), &source).unwrap();
        let before = snapshot(&failing);
        assert!(matches!(
            failing.advance(flow, inlet([[1.0; 2]; 3]), Some(source), settings(), |_| {
                false
            }),
            Err(LiquidVolumeError::FractionBounds { .. })
        ));
        assert_eq!(snapshot(&failing), before);
        assert_eq!(carrier.state().time, 0.5);
        assert!(carrier.state().x.iter().all(|&u| u == 0.25));
    }
}
