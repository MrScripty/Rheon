use rheon::*;
fn fixture(axis: Axis, full: usize, top: f64) -> (GridGeometry, ColumnSurfaceWorkspace) {
    let d = match axis {
        Axis::X => 0,
        Axis::Y => 1,
        Axis::Z => 2,
    };
    let mut n = [2, 3, 2];
    n[d] = (full + 2) as u64;
    let g = GridGeometry::new(n, [1.0; 3], [0.0; 3]).unwrap();
    let counts = g.counts();
    let mut f = vec![0.0; g.cell_len()];
    for k in 0..counts[2] {
        for j in 0..counts[1] {
            for i in 0..counts[0] {
                let p = [i, j, k];
                f[g.cell_index(p).unwrap()] = if p[d] < full {
                    1.0
                } else if p[d] == full {
                    top
                } else {
                    0.0
                };
            }
        }
    }
    let surface = ColumnSurfaceWorkspace::new(
        g.clone(),
        axis,
        &f,
        VolumeStamp { id: 1, version: 0 },
        1 << 20,
    )
    .unwrap();
    (g, surface)
}
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
fn muts(v: &mut [Vec<f32>; 3]) -> [&mut [f32]; 3] {
    let [x, y, z] = v;
    [x, y, z]
}
fn input<'a>(s: ColumnSurfaceView<'a>, u: &'a [Vec<f32>; 3]) -> ColumnShearInputs<'a> {
    ColumnShearInputs {
        surface: s,
        velocity: refs(u),
        density: 1.0,
        dynamic_viscosity: 1.0,
        dt: 0.125,
    }
}
fn profiles(g: &GridGeometry, axis: Axis, profile: &[f32]) -> [Vec<f32>; 3] {
    let d = match axis {
        Axis::X => 0,
        Axis::Y => 1,
        Axis::Z => 2,
    };
    let mut u = fields(g);
    for (component, field) in u.iter_mut().enumerate() {
        if component == d {
            continue;
        }
        let m = g.face_counts([Axis::X, Axis::Y, Axis::Z][component]);
        for k in 0..m[2] {
            for j in 0..m[1] {
                for i in 0..m[0] {
                    let p = [i, j, k];
                    field[g
                        .face_index([Axis::X, Axis::Y, Axis::Z][component], p)
                        .unwrap()] = profile.get(p[d]).copied().unwrap_or(0.0);
                }
            }
        }
    }
    u
}
#[test]
fn independent_partial_dual_mass_matrix_force_and_energy_all_axes() {
    for axis in [Axis::X, Axis::Y, Axis::Z] {
        let (g, s) = fixture(axis, 2, 0.25);
        let u = profiles(&g, axis, &[1.0, -1.0]);
        let mut out = fields(&g);
        let mut w = ColumnShearWorkspace::new(g.clone(), axis, 1 << 20).unwrap();
        let r = w
            .update(input(s.state(), &u), muts(&mut out), |_| false)
            .unwrap();
        let area = r.geometry.area;
        assert_eq!(r.geometry.wet_nodes, 2);
        assert_eq!(r.geometry.dual_length(0), Some(1.0));
        assert_eq!(r.geometry.dual_length(1), Some(1.25));
        assert_eq!(r.geometry.dual_length(2), None);
        assert_eq!(w.mass_scratch()[..2], [area, 1.25 * area]);
        for force in w.force_scratch() {
            assert_eq!(force[..2], [-2.0 * area, 2.0 * area]);
        }
        assert_eq!(r.dissipation_before, 8.0 * area);
        // Independent diagonal mass and [[k,-k],[-k,k]] stiffness give
        // two tangential profiles (0.75, -0.8), not full-box (0.75,-0.75).
        let expected = profiles(&g, axis, &[0.75, -0.8]);
        assert_eq!(out, expected);
        assert!(r.kinetic_after < r.kinetic_before);
        assert!(r.identity_error.abs() <= r.energy_budget);
        for d in 0..3 {
            assert!(r.momentum_error[d].abs() <= r.momentum_budget[d]);
            assert!(r.force_sum[d].abs() <= r.force_budget[d]);
        }
        assert_eq!(
            r.workspace_bytes,
            g.counts()[match axis {
                Axis::X => 0,
                Axis::Y => 1,
                Axis::Z => 2,
            }] * 40
        );
    }
}
#[test]
fn local_affine_shear_force_and_translation_null_mode() {
    let (g, s) = fixture(Axis::Y, 3, 0.75);
    let u = profiles(&g, Axis::Y, &[0.5, 1.5, 2.5, 3.5]);
    let mut out = fields(&g);
    let mut w = ColumnShearWorkspace::new(g.clone(), Axis::Y, 1 << 20).unwrap();
    let r = w
        .update(input(s.state(), &u), muts(&mut out), |_| false)
        .unwrap();
    let area = r.geometry.area;
    for f in w.force_scratch() {
        assert_eq!(f[..4], [area, 0.0, 0.0, -area]);
    }
    assert_eq!(r.dissipation_before, 6.0 * area);
    let translation = profiles(&g, Axis::Y, &[0.25; 4]);
    let r = w
        .update(input(s.state(), &translation), muts(&mut out), |_| false)
        .unwrap();
    assert_eq!(translation, out);
    assert_eq!(r.kinetic_before, r.kinetic_after);
    assert_eq!(r.dissipation_before, 0.0);
    assert_eq!(r.force_sum, [0.0; 3]);
}
#[test]
fn cut_heights_cover_both_center_classifications_and_single_node() {
    for (full, top, wet, last) in [
        (2, 0.25, 2, 1.25),
        (2, 0.5, 2, 1.5),
        (2, 0.75, 3, 0.75),
        (2, 0.0, 2, 1.0),
        (0, 0.75, 1, 0.75),
    ] {
        let (g, s) = fixture(Axis::Y, full, top);
        let u = profiles(&g, Axis::Y, &vec![0.25; wet]);
        let mut out = fields(&g);
        let mut w = ColumnShearWorkspace::new(g, Axis::Y, 1 << 20).unwrap();
        let r = w
            .update(input(s.state(), &u), muts(&mut out), |_| false)
            .unwrap();
        assert_eq!(r.geometry.wet_nodes, wet);
        assert_eq!(r.geometry.dual_length(wet - 1), Some(last));
        assert_eq!(u, out);
        assert_eq!(
            r.geometry.liquid_volume,
            r.geometry.area * (full as f64 + top)
        );
        assert_eq!(r.mass_volume_error, 0.0);
    }
}
#[test]
fn every_callback_occurrence_cancels_output_and_retry_is_equal() {
    let (g, s) = fixture(Axis::Y, 3, 0.75);
    let u = profiles(&g, Axis::Y, &[1.0, 0.5, -0.5, -1.0]);
    let mut w = ColumnShearWorkspace::new(g.clone(), Axis::Y, 1 << 20).unwrap();
    let mut reference = fields(&g);
    let mut visits = Vec::new();
    w.update(input(s.state(), &u), muts(&mut reference), |stage| {
        visits.push(stage);
        false
    })
    .unwrap();
    for position in 0..visits.len() {
        let mut out = fields(&g);
        for v in &mut out {
            v.fill(7.0);
        }
        let before = out.clone();
        let mut index = 0;
        assert!(matches!(
            w.update(input(s.state(), &u), muts(&mut out), |_| {
                let hit = index == position;
                index += 1;
                hit
            }),
            Err(ColumnShearError::Cancelled { .. })
        ));
        assert_eq!(before, out);
        w.update(input(s.state(), &u), muts(&mut out), |_| false)
            .unwrap();
        assert_eq!(reference, out);
    }
}
#[test]
fn explicit_admission_scale_and_stability_failures_preserve_output() {
    let (g, s) = fixture(Axis::Y, 2, 0.75);
    let u = profiles(&g, Axis::Y, &[1.0, 0.5, -0.5]);
    let mut w = ColumnShearWorkspace::new(g.clone(), Axis::Y, 1 << 20).unwrap();
    let bytes = w.allocated_bytes();
    assert!(matches!(
        ColumnShearWorkspace::new(g.clone(), Axis::Y, bytes - 1),
        Err(ColumnShearError::BufferLimit { .. })
    ));
    for (rho, mu, dt) in [
        (1.0, -1.0, 0.1),
        (0.0, 1.0, 0.1),
        (1.0, 1.0, 2.0),
        (1.0, f64::NAN, 0.1),
        (1.0, f64::MIN_POSITIVE, 0.01),
        (f64::MAX, f64::MIN_POSITIVE, 0.1),
    ] {
        let mut out = fields(&g);
        for v in &mut out {
            v.fill(7.0);
        }
        let before = out.clone();
        let mut inputs = input(s.state(), &u);
        inputs.density = rho;
        inputs.dynamic_viscosity = mu;
        inputs.dt = dt;
        assert!(w.update(inputs, muts(&mut out), |_| false).is_err());
        assert_eq!(out, before);
    }
    let mut changed = u.clone();
    changed[0][0] = f32::NAN;
    let mut out = fields(&g);
    assert!(matches!(
        w.update(input(s.state(), &changed), muts(&mut out), |_| false),
        Err(ColumnShearError::ArithmeticFailure)
    ));
    let mut changed = u.clone();
    changed[1][0] = 1.0;
    assert!(matches!(
        w.update(input(s.state(), &changed), muts(&mut out), |_| false),
        Err(ColumnShearError::UnsupportedVelocity)
    ));
    let mut changed = u.clone();
    changed[0][1] += 0.1;
    assert!(matches!(
        w.update(input(s.state(), &changed), muts(&mut out), |_| false),
        Err(ColumnShearError::UnsupportedVelocity)
    ));
    let mut fractions = vec![0.0; g.cell_len()];
    let n = g.counts();
    for k in 0..n[2] {
        for j in 0..n[1] {
            for i in 0..n[0] {
                fractions[g.cell_index([i, j, k]).unwrap()] = if j < 2 {
                    1.0
                } else if j == 2 {
                    if i == 0 { 0.75 } else { 0.5 }
                } else {
                    0.0
                };
            }
        }
    }
    let varied = ColumnSurfaceWorkspace::new(
        g,
        Axis::Y,
        &fractions,
        VolumeStamp { id: 1, version: 0 },
        1 << 20,
    )
    .unwrap();
    assert!(matches!(
        w.update(input(varied.state(), &u), muts(&mut out), |_| false),
        Err(ColumnShearError::VaryingHeight)
    ));
}

#[test]
fn analytic_traction_free_shear_refines_with_liquid_only_weights() {
    for top in [0.25, 0.75] {
        let mut previous = f64::INFINITY;
        for full in [8, 16, 32] {
            let h = 1.0 / (full as f64 + top);
            let g = GridGeometry::new([2, (full + 2) as u64, 1], [0.5, h, 0.75], [0.0; 3]).unwrap();
            let mut fraction = vec![0.0; g.cell_len()];
            for j in 0..full + 2 {
                for i in 0..2 {
                    fraction[g.cell_index([i, j, 0]).unwrap()] = if j < full {
                        1.0
                    } else if j == full {
                        top
                    } else {
                        0.0
                    };
                }
            }
            let surface = ColumnSurfaceWorkspace::new(
                g.clone(),
                Axis::Y,
                &fraction,
                VolumeStamp { id: 1, version: 0 },
                1 << 20,
            )
            .unwrap();
            let wet = full + usize::from(top > 0.5);
            let initial = (0..wet)
                .map(|j| (std::f64::consts::PI * (j as f64 + 0.5) * h).cos() as f32)
                .collect::<Vec<_>>();
            let mut u = profiles(&g, Axis::Y, &initial);
            let mut out = fields(&g);
            let mut w = ColumnShearWorkspace::new(g.clone(), Axis::Y, 1 << 20).unwrap();
            let steps = (0.05 * 0.1 / (0.1 * h * h)).ceil() as usize;
            let dt = 0.1 / steps as f64;
            let mut geometry = None;
            for _ in 0..steps {
                let r = w
                    .update(
                        ColumnShearInputs {
                            surface: surface.state(),
                            velocity: refs(&u),
                            density: 3.0,
                            dynamic_viscosity: 0.15,
                            dt,
                        },
                        muts(&mut out),
                        |_| false,
                    )
                    .unwrap();
                geometry = Some(r.geometry);
                std::mem::swap(&mut u, &mut out);
            }
            let geom = geometry.unwrap();
            let decay = (-0.05 * 0.1 * std::f64::consts::PI.powi(2)).exp();
            let error = (0..wet)
                .map(|j| {
                    let value = f64::from(u[0][g.face_index(Axis::X, [0, j, 0]).unwrap()]);
                    let exact = f64::from(initial[j]) * decay;
                    geom.dual_length(j).unwrap() * (value - exact).powi(2)
                })
                .sum::<f64>()
                .sqrt();
            assert!(
                error < previous * 0.4,
                "n={full} top={top} error={error} previous={previous}"
            );
            previous = error;
        }
    }
}
