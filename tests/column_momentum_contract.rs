use rheon::*;
fn d(axis: Axis) -> usize {
    match axis {
        Axis::X => 0,
        Axis::Y => 1,
        Axis::Z => 2,
    }
}
fn cell(g: &GridGeometry, axis: Axis, c: usize, j: usize) -> usize {
    let mut p = [0; 3];
    let mut r = c;
    for (i, v) in p.iter_mut().enumerate() {
        if i != d(axis) {
            *v = r % g.counts()[i];
            r /= g.counts()[i];
        }
    }
    p[d(axis)] = j;
    g.cell_index(p).unwrap()
}
fn geometry(g: &GridGeometry, axis: Axis, heights: &[f64], version: u64) -> ColumnSurfaceWorkspace {
    let mut f = vec![0.0; g.cell_len()];
    for (c, &height) in heights.iter().enumerate() {
        for j in 0..g.counts()[d(axis)] {
            f[cell(g, axis, c, j)] = (height - j as f64).clamp(0.0, 1.0);
        }
    }
    ColumnSurfaceWorkspace::new(g.clone(), axis, &f, VolumeStamp { id: 7, version }, 1 << 20)
        .unwrap()
}
fn grid(axis: Axis, n: usize) -> GridGeometry {
    let mut counts = [2, 1, 1];
    counts[d(axis)] = n as u64;
    if axis == Axis::X {
        counts[1] = 2;
    }
    GridGeometry::new(counts, [1.0; 3], [0.0; 3]).unwrap()
}
fn profiles(g: &GridGeometry, axis: Axis, values: &[&[f32]]) -> [Vec<f32>; 2] {
    let mut u = [vec![0.0; g.cell_len()], vec![0.0; g.cell_len()]];
    for (c, profile) in values.iter().enumerate() {
        for (j, &v) in profile.iter().enumerate() {
            u[0][cell(g, axis, c, j)] = v;
            u[1][cell(g, axis, c, j)] = -0.5 * v;
        }
    }
    u
}
fn refs(u: &[Vec<f32>; 2]) -> [&[f32]; 2] {
    [&u[0], &u[1]]
}
fn muts(u: &mut [Vec<f32>; 2]) -> [&mut [f32]; 2] {
    let [a, b] = u;
    [a, b]
}
fn inputs<'a>(
    a: &'a ColumnSurfaceWorkspace,
    b: &'a ColumnSurfaceWorkspace,
    u: &'a [Vec<f32>; 2],
    donor: Option<&'a [[f32; 2]]>,
) -> ColumnMomentumInputs<'a> {
    ColumnMomentumInputs {
        before: a.state(),
        after: b.state(),
        density: 1.0,
        velocity: refs(u),
        added_velocity: donor,
    }
}
fn gates(r: ColumnMomentumReport) {
    assert!(r.mass_error.abs() <= r.mass_budget);
    for t in 0..2 {
        assert!(r.momentum_error[t].abs() <= r.momentum_budget[t]);
    }
    assert!(r.mixing_loss >= 0.0);
    assert!(r.energy_error.abs() <= r.energy_budget);
    assert!(
        r.kinetic_after - r.kinetic_before - r.kinetic_added + r.kinetic_removed <= r.energy_budget
    );
}
#[test]
fn hand_derived_closed_exchange_all_axes() {
    for axis in [Axis::X, Axis::Y, Axis::Z] {
        let g = grid(axis, 4);
        let a = geometry(&g, axis, &[2.75, 2.25], 0);
        let b = geometry(&g, axis, &[2.25, 2.75], 1);
        let u = profiles(&g, axis, &[&[0.0, 1.0, 3.0], &[0.0, -1.0]]);
        let donor = [[0.0, 0.0], [3.0, -1.5]];
        let mut out = profiles(&g, axis, &[]);
        let mut w = ColumnMomentumWorkspace::new(g.clone(), axis, 24 * g.cell_len()).unwrap();
        let r = w
            .remap(inputs(&a, &b, &u, Some(&donor)), muts(&mut out), |_| false)
            .unwrap();
        gates(r);
        let expected = profiles(&g, axis, &[&[0.0, 1.4], &[0.0, -1.0, 5.0 / 3.0]]);
        assert_eq!(out, expected);
        assert_eq!(
            (r.mass_before, r.mass_after, r.mass_added, r.mass_removed),
            (5.0, 5.0, 0.5, 0.5)
        );
        assert_eq!(r.momentum_added, r.momentum_removed);
        assert_eq!(r.kinetic_added, r.kinetic_removed);
        assert_eq!((r.activated_nodes, r.deactivated_nodes), (1, 1));
        assert_eq!(r.workspace_bytes, 24 * g.cell_len());
        assert!((r.mixing_loss - 13.0 / 6.0).abs() < 1e-14);
        assert!(r.kinetic_after < r.kinetic_before);
    }
}
#[test]
fn identity_and_constants_cover_cut_classification_and_single_node() {
    for axis in [Axis::X, Axis::Y, Axis::Z] {
        for (old, new) in [
            (0.75, 1.25),
            (1.5, 1.75),
            (2.25, 2.75),
            (2.75, 2.25),
            (1.25, 0.75),
        ] {
            let g = grid(axis, 4);
            let a = geometry(&g, axis, &[old, old], 0);
            let b = geometry(&g, axis, &[new, new], 1);
            let count = old.floor() as usize + usize::from(old.fract() > 0.5);
            let u = profiles(&g, axis, &[&vec![0.25; count], &vec![0.25; count]]);
            let donor = [[0.25, -0.125]; 2];
            let mut out = profiles(&g, axis, &[]);
            let mut w = ColumnMomentumWorkspace::new(g.clone(), axis, 1 << 20).unwrap();
            let r = w
                .remap(inputs(&a, &b, &u, Some(&donor)), muts(&mut out), |_| false)
                .unwrap();
            gates(r);
            assert_eq!(r.mixing_loss, 0.0);
            let wet = new.floor() as usize + usize::from(new.fract() > 0.5);
            assert_eq!(
                out,
                profiles(&g, axis, &[&vec![0.25; wet], &vec![0.25; wet]])
            );
            let identity = geometry(&g, axis, &[old, old], 1);
            let arbitrary = profiles(
                &g,
                axis,
                &[
                    &(0..count).map(|j| j as f32 - 0.25).collect::<Vec<_>>(),
                    &vec![-0.75; count],
                ],
            );
            let r = w
                .remap(
                    inputs(&a, &identity, &arbitrary, None),
                    muts(&mut out),
                    |_| false,
                )
                .unwrap();
            gates(r);
            assert_eq!(out, arbitrary);
            assert_eq!(r.mixing_loss, 0.0);
        }
    }
}
#[test]
fn every_callback_rolls_back_and_retry_is_identical() {
    let axis = Axis::Y;
    let g = grid(axis, 4);
    let a = geometry(&g, axis, &[2.75, 2.25], 0);
    let b = geometry(&g, axis, &[2.25, 2.75], 1);
    let u = profiles(&g, axis, &[&[0.0, 1.0, 3.0], &[0.0, -1.0]]);
    let donor = [[0.0; 2], [3.0, -1.5]];
    let mut clean = profiles(&g, axis, &[]);
    let mut w = ColumnMomentumWorkspace::new(g.clone(), axis, 1 << 20).unwrap();
    let mut calls = 0;
    let reference = w
        .remap(inputs(&a, &b, &u, Some(&donor)), muts(&mut clean), |_| {
            calls += 1;
            false
        })
        .unwrap();
    for fail_at in 0..calls {
        let mut out = [vec![7.0; g.cell_len()], vec![7.0; g.cell_len()]];
        let initial = out.clone();
        let mut seen = 0;
        assert!(matches!(
            w.remap(inputs(&a, &b, &u, Some(&donor)), muts(&mut out), |_| {
                let fail = seen == fail_at;
                seen += 1;
                fail
            }),
            Err(ColumnMomentumError::Cancelled { .. })
        ));
        assert_eq!(out, initial);
        let retry = w
            .remap(inputs(&a, &b, &u, Some(&donor)), muts(&mut out), |_| false)
            .unwrap();
        assert_eq!(retry, reference);
        assert_eq!(out, clean);
    }
}
#[test]
fn refused_inputs_and_late_scale_failure_preserve_outputs() {
    let axis = Axis::Y;
    let g = grid(axis, 4);
    let a = geometry(&g, axis, &[2.75, 2.25], 0);
    let b = geometry(&g, axis, &[2.25, 2.75], 1);
    let same = geometry(&g, axis, &[2.25, 2.75], 0);
    let mut w = ColumnMomentumWorkspace::new(g.clone(), axis, 1 << 20).unwrap();
    let u = profiles(&g, axis, &[&[0.0, 1.0, 3.0], &[0.0, -1.0]]);
    let donor = [[0.0; 2], [3.0, -1.5]];
    let mut out = [vec![7.0; g.cell_len()], vec![7.0; g.cell_len()]];
    let saved = out.clone();
    assert_eq!(
        w.remap(inputs(&a, &b, &u, None), muts(&mut out), |_| false),
        Err(ColumnMomentumError::MissingAddedVelocity)
    );
    assert_eq!(
        w.remap(inputs(&a, &same, &u, Some(&donor)), muts(&mut out), |_| {
            false
        }),
        Err(ColumnMomentumError::StampMismatch)
    );
    for bad in [f32::NAN, f32::INFINITY, f32::from_bits(1)] {
        let mut v = u.clone();
        v[0][0] = bad;
        assert_eq!(
            w.remap(inputs(&a, &b, &v, Some(&donor)), muts(&mut out), |_| false),
            Err(ColumnMomentumError::InvalidVelocity)
        );
    }
    let mut dry = u.clone();
    dry[0][cell(&g, axis, 1, 3)] = 1.0;
    assert_eq!(
        w.remap(inputs(&a, &b, &dry, Some(&donor)), muts(&mut out), |_| {
            false
        }),
        Err(ColumnMomentumError::InvalidVelocity)
    );
    for density in [
        0.0,
        -1.0,
        f64::NAN,
        f64::from_bits(1),
        f64::MAX,
        f64::MIN_POSITIVE,
    ] {
        let mut i = inputs(&a, &b, &u, Some(&donor));
        i.density = density;
        assert!(w.remap(i, muts(&mut out), |_| false).is_err());
        assert_eq!(out, saved);
    }
    let i = ColumnMomentumInputs {
        velocity: [&u[0][1..], &u[1]],
        ..inputs(&a, &b, &u, Some(&donor))
    };
    assert_eq!(
        w.remap(i, muts(&mut out), |_| false),
        Err(ColumnMomentumError::ShapeMismatch)
    );
    assert_eq!(out, saved);
    assert!(matches!(
        ColumnMomentumWorkspace::new(g.clone(), axis, 24 * g.cell_len() - 1),
        Err(ColumnMomentumError::BufferLimit { .. })
    ));
    // Input values are normal f32, but the exact average falls below normal f32.
    let mut tiny = profiles(
        &g,
        axis,
        &[&[0.0, f32::MIN_POSITIVE, -f32::MIN_POSITIVE], &[0.0, 0.0]],
    );
    tiny[1].fill(0.0);
    let zero = [[0.0; 2]; 2];
    let mut stages = 0;
    assert_eq!(
        w.remap(inputs(&a, &b, &tiny, Some(&zero)), muts(&mut out), |_| {
            stages += 1;
            false
        }),
        Err(ColumnMomentumError::ArithmeticFailure)
    );
    assert!(stages > 1);
    assert_eq!(out, saved);
}
#[test]
fn large_support_change_uses_explicit_parcels() {
    let axis = Axis::Y;
    let g = grid(axis, 7);
    let a = geometry(&g, axis, &[0.75, 0.75], 0);
    let b = geometry(&g, axis, &[5.25, 5.25], 1);
    let u = profiles(&g, axis, &[&[1.0], &[1.0]]);
    let donor = [[2.0, -1.0]; 2];
    let mut out = profiles(&g, axis, &[]);
    let mut w = ColumnMomentumWorkspace::new(g.clone(), axis, 1 << 20).unwrap();
    let r = w
        .remap(inputs(&a, &b, &u, Some(&donor)), muts(&mut out), |_| false)
        .unwrap();
    gates(r);
    assert_eq!(r.mass_added, 9.0);
    assert_eq!(
        out,
        profiles(
            &g,
            axis,
            &[&[1.25, 2.0, 2.0, 2.0, 2.0], &[1.25, 2.0, 2.0, 2.0, 2.0]]
        )
    );
}
#[test]
fn affine_parcel_means_refine_without_claiming_linear_exactness() {
    let axis = Axis::Y;
    let mut errors = Vec::new();
    for full in [8, 16, 32, 64] {
        let h = 1.0 / (full as f64 + 0.25);
        let g = GridGeometry::new([2, (full + 2) as u64, 1], [0.5, h, 0.25], [0.0; 3]).unwrap();
        let a = geometry(&g, axis, &[full as f64 + 0.25; 2], 0);
        let b = geometry(&g, axis, &[full as f64 + 0.75; 2], 1);
        let mut u = [vec![0.0; g.cell_len()], vec![0.0; g.cell_len()]];
        for c in 0..2 {
            for j in 0..full {
                let low = j as f64 * h;
                let high = if j + 1 == full {
                    1.0
                } else {
                    (j + 1) as f64 * h
                };
                u[0][cell(&g, axis, c, j)] = (1.0 + 0.5 * (low + high)) as f32;
            }
        }
        let donor = [[(2.0 + 0.25 * h) as f32, 0.0]; 2];
        let mut out = profiles(&g, axis, &[]);
        let mut w = ColumnMomentumWorkspace::new(g.clone(), axis, 1 << 20).unwrap();
        let r = w
            .remap(inputs(&a, &b, &u, Some(&donor)), muts(&mut out), |_| false)
            .unwrap();
        gates(r);
        let mut squared = 0.0;
        for c in 0..2 {
            for j in 0..=full {
                let low = j as f64 * h;
                let high = if j == full {
                    1.0 + 0.5 * h
                } else {
                    (j + 1) as f64 * h
                };
                let exact = 1.0 + 0.5 * (low + high);
                squared += (high - low) * (f64::from(out[0][cell(&g, axis, c, j)]) - exact).powi(2);
            }
        }
        let error = (squared / (2.0 * (1.0 + 0.5 * h))).sqrt();
        assert!(error > 0.0);
        errors.push(error);
    }
    for pair in errors.windows(2) {
        assert!(pair[0] / pair[1] > 2.5);
    }
}
