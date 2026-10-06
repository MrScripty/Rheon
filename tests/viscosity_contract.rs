use rheon::*;
fn grid(n: u64) -> GridGeometry {
    GridGeometry::new([n, n, 1], [1.0 / n as f64, 1.0 / n as f64, 0.75], [0.0; 3]).unwrap()
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
fn mode(g: &GridGeometry) -> [Vec<f32>; 3] {
    let mut v = fields(g);
    let n = g.counts()[0];
    for j in 0..n {
        for i in 1..n {
            v[0][g.face_index(Axis::X, [i, j, 0]).unwrap()] =
                ((std::f64::consts::PI * i as f64 / n as f64).sin()
                    * (std::f64::consts::PI * (j as f64 + 0.5) / n as f64).cos())
                    as f32;
            v[1][g.face_index(Axis::Y, [j, i, 0]).unwrap()] =
                -v[0][g.face_index(Axis::X, [i, j, 0]).unwrap()];
        }
    }
    v
}
#[test]
fn symmetric_strain_cross_derivative_and_independent_dense_energy() {
    // Four interior face unknowns. Independently derived normal strains
    // and one interior XY shear row couple components; scalar smoothing
    // would incorrectly leave both Y faces at zero.
    let g = GridGeometry::new([2, 2, 1], [1.0; 3], [0.0; 3]).unwrap();
    let mut u = fields(&g);
    u[0][g.face_index(Axis::X, [1, 0, 0]).unwrap()] = 1.0;
    // Input only one face; evaluate the independent strain rows.
    let mut w = ViscosityWorkspace::new(g.clone(), 1 << 20).unwrap();
    let mut out = fields(&g);
    let r = w
        .update(refs(&u), 2.0, 0.5, 0.125, muts(&mut out), |_| false)
        .unwrap();
    // K_x = 2*(1-0) + 2*(1-0) + (1-0) = 5.
    assert_eq!(
        out[0][g.face_index(Axis::X, [1, 0, 0]).unwrap()],
        1.0 - 5.0 / 32.0
    );
    assert_eq!(
        out[0][g.face_index(Axis::X, [1, 1, 0]).unwrap()],
        1.0 / 32.0
    );
    assert_eq!(
        out[1][g.face_index(Axis::Y, [0, 1, 0]).unwrap()],
        -1.0 / 32.0
    );
    assert_eq!(
        out[1][g.face_index(Axis::Y, [1, 1, 0]).unwrap()],
        1.0 / 32.0
    );
    assert_eq!(r.dissipation_before, 2.5);
    assert!(r.kinetic_after < r.kinetic_before);
    assert!(r.energy_identity_error.abs() < 1e-14);
}
#[test]
fn divergence_free_mode_matches_discrete_decay_and_refines() {
    let mut previous = f64::INFINITY;
    for n in [8, 16, 32] {
        let g = grid(n);
        let initial = mode(&g);
        let mut u = initial.clone();
        let mut out = fields(&g);
        let mut w = ViscosityWorkspace::new(g.clone(), 1 << 24).unwrap();
        let nu = 0.05;
        let duration = 0.1;
        let steps =
            (duration * nu * (2.0 * (n * n) as f64 + 1.0 / 0.75_f64.powi(2)) / 0.1).ceil() as usize;
        let dt = duration / steps as f64;
        let lambda = 8.0 * (std::f64::consts::PI / (2.0 * n as f64)).sin().powi(2) * (n * n) as f64;
        let factor = (1.0 - nu * dt * lambda).powi(steps as i32);
        let exact = (-nu * duration * 2.0 * std::f64::consts::PI.powi(2)).exp();
        let mut energy = f64::INFINITY;
        for _ in 0..steps {
            let r = w
                .update(refs(&u), 3.0, nu * 3.0, dt, muts(&mut out), |_| false)
                .unwrap();
            assert!(r.kinetic_after <= energy + r.energy_rounding_budget);
            energy = r.kinetic_after;
            std::mem::swap(&mut u, &mut out);
        }
        let mut err = 0.0;
        let mut modal = 0.0_f64;
        for (&a, &b) in u.iter().flatten().zip(initial.iter().flatten()) {
            err += (f64::from(a) - exact * f64::from(b)).powi(2);
            modal = modal.max((f64::from(a) - factor * f64::from(b)).abs());
        }
        err = (err / u.iter().map(Vec::len).sum::<usize>() as f64).sqrt();
        assert!(modal < 2e-6, "n={n}: {modal}");
        assert!(err < previous * 0.4, "n={n}: {err} previous={previous}");
        previous = err;
    }
}
#[test]
fn cancellation_all_stage_occurrences_and_failures_preserve_output() {
    let g = grid(4);
    let u = mode(&g);
    let mut w = ViscosityWorkspace::new(g.clone(), 1 << 20).unwrap();
    let mut visits = Vec::new();
    let mut reference = fields(&g);
    w.update(refs(&u), 1.0, 0.1, 0.01, muts(&mut reference), |s| {
        visits.push(s);
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
            w.update(refs(&u), 1.0, 0.1, 0.01, muts(&mut out), |_| {
                let hit = index == position;
                index += 1;
                hit
            }),
            Err(ViscosityError::Cancelled { .. })
        ));
        assert_eq!(out, before);
        w.update(refs(&u), 1.0, 0.1, 0.01, muts(&mut out), |_| false)
            .unwrap();
        assert_eq!(out, reference);
    }
    for (rho, mu, dt) in [
        (1.0, -1.0, 0.01),
        (1.0, f64::NAN, 0.01),
        (0.0, 0.1, 0.01),
        (1.0, 1.0, 1.0),
        (f64::MAX, f64::MIN_POSITIVE, 0.01),
        (1.0, 0.1, f64::from_bits(1)),
    ] {
        let mut out = fields(&g);
        for v in &mut out {
            v.fill(7.0);
        }
        let before = out.clone();
        assert!(
            w.update(refs(&u), rho, mu, dt, muts(&mut out), |_| false)
                .is_err()
        );
        assert_eq!(before, out);
    }
    let bytes = w.allocated_bytes();
    assert!(matches!(
        ViscosityWorkspace::new(g, bytes - 1),
        Err(ViscosityError::BufferLimit { .. })
    ));
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
                max_iterations: 2000,
            },
            actual_divergence_limit: 1e-5,
            max_courant: 1.0,
        },
        represented_density: 1.0,
        volume_stamp: VolumeStamp { id: 1, version: 0 },
        carrier_id: 2,
    }
}
fn inputs(forces: &[BodyForce]) -> LiquidStepInputs<'_> {
    LiquidStepInputs {
        requested_dt: 0.125,
        smoke_source: None,
        forces,
        inlet: LiquidInlet::new(VolumeStamp { id: 3, version: 0 }, [[0.0; 2]; 3]).unwrap(),
        source: None,
        volume: LiquidVolumeSettings {
            max_outward_courant: 1.0,
            actual_divergence_limit: 1e-5,
        },
    }
}
fn forces() -> [BodyForce; 4] {
    [
        ([0.5, 0.0, 0.0], [0.0, 0.0, 0.0], [2.0, 1.0, 1.0]),
        ([-0.5, 0.0, 0.0], [0.0, 1.0, 0.0], [2.0, 2.0, 1.0]),
        ([0.0, -0.5, 0.0], [0.0, 0.0, 0.0], [1.0, 2.0, 1.0]),
        ([0.0, 0.5, 0.0], [1.0, 0.0, 0.0], [2.0, 2.0, 1.0]),
    ]
    .map(|(value, lower, upper)| BodyForce {
        value,
        units: ForceUnits::Acceleration,
        region: Some(ForceRegion { lower, upper }),
    })
}
fn state(s: &LiquidTransportSimulation) -> Vec<u64> {
    let v = s.state();
    let mut b = v
        .pressure
        .iter()
        .chain(v.liquid.fraction)
        .map(|x| x.to_bits())
        .collect::<Vec<_>>();
    b.extend(
        v.carrier
            .x
            .iter()
            .chain(v.carrier.y)
            .chain(v.carrier.z)
            .chain(v.carrier.tracer)
            .map(|x| u64::from(x.to_bits())),
    );
    b.extend([
        v.carrier.time.to_bits(),
        v.liquid.time.to_bits(),
        v.carrier_stamp.version,
        v.liquid.stamp.version,
    ]);
    b
}
#[test]
fn coupled_viscosity_preserves_both_owners_on_every_callback_and_retries() {
    for method in PressureImplementation::ALL {
        let g = GridGeometry::new([2, 2, 1], [1.0; 3], [0.0; 3]).unwrap();
        let create =
            || LiquidTransportSimulation::new(g.clone(), config(), method, vec![1.0; 4]).unwrap();
        let f = forces();
        let mut reference = create();
        let mut w = ViscosityWorkspace::new(g.clone(), 1 << 20).unwrap();
        let mut visits = Vec::new();
        let report = reference
            .step_viscous(inputs(&f), &mut w, 0.125, |stage| {
                visits.push(stage);
                false
            })
            .unwrap();
        assert_eq!(
            report.total_array_bytes,
            report.owned_array_bytes + w.allocated_bytes()
        );
        assert_eq!(report.liquid.liquid_volume_after, 4.0);
        for position in 0..visits.len() {
            let mut s = create();
            let before = state(&s);
            let mut index = 0;
            assert!(
                s.step_viscous(inputs(&f), &mut w, 0.125, |_| {
                    let hit = index == position;
                    index += 1;
                    hit
                })
                .is_err()
            );
            assert_eq!(state(&s), before);
            s.step_viscous(inputs(&f), &mut w, 0.125, |_| false)
                .unwrap();
            assert_eq!(state(&s), state(&reference));
        }
        for _ in 0..8 {
            reference
                .step_viscous(inputs(&[]), &mut w, 0.125, |_| false)
                .unwrap();
            assert_eq!(reference.state().liquid.fraction, &[1.0; 4]);
        }
        let before = state(&reference);
        assert!(
            reference
                .step_viscous(inputs(&[]), &mut w, 100.0, |_| false)
                .is_err()
        );
        assert_eq!(before, state(&reference));
        let mut bad = inputs(&f);
        bad.volume.max_outward_courant = 1e-9;
        assert!(
            reference
                .step_viscous(bad, &mut w, 0.125, |_| false)
                .is_err()
        );
        assert_eq!(before, state(&reference));
    }
}
#[test]
fn viscosity_rejects_surface_partial_fill_density_mismatch_and_nonzero_walls() {
    let g = GridGeometry::new([2, 2, 1], [1.0; 3], [0.0; 3]).unwrap();
    let mut w = ViscosityWorkspace::new(g.clone(), 1 << 20).unwrap();
    for fraction in [vec![0.5; 4], vec![1.0, 1.0, 0.0, 0.0]] {
        let mut s = LiquidTransportSimulation::new(
            g.clone(),
            config(),
            PressureImplementation::JacobiPcgV1,
            fraction,
        )
        .unwrap();
        let b = state(&s);
        assert!(s.step_viscous(inputs(&[]), &mut w, 0.1, |_| false).is_err());
        assert_eq!(b, state(&s));
    }
    let stamp = VolumeStamp { id: 1, version: 0 };
    let mut surface = LiquidTransportSimulation::with_free_surface(
        g.clone(),
        config(),
        PressureImplementation::JacobiPcgV1,
        vec![1.0, 1.0, 0.0, 0.0],
        SlabFreeSurface::new(g.clone(), Axis::Y, 1, stamp).unwrap(),
    )
    .unwrap();
    let before = state(&surface);
    assert!(
        surface
            .step_viscous(inputs(&[]), &mut w, 0.1, |_| false)
            .is_err()
    );
    assert_eq!(before, state(&surface));
    let mut columns = LiquidTransportSimulation::with_reconstructed_surface(
        g.clone(),
        config(),
        PressureImplementation::JacobiPcgV1,
        vec![1.0, 1.0, 0.0, 0.0],
        Axis::Y,
    )
    .unwrap();
    let before = state(&columns);
    assert!(
        columns
            .step_viscous(inputs(&[]), &mut w, 0.1, |_| false)
            .is_err()
    );
    assert_eq!(before, state(&columns));
    let mut mismatch = config();
    mismatch.represented_density = 2.0;
    let mut s = LiquidTransportSimulation::new(
        g.clone(),
        mismatch,
        PressureImplementation::JacobiPcgV1,
        vec![1.0; 4],
    )
    .unwrap();
    let before = state(&s);
    assert!(s.step_viscous(inputs(&[]), &mut w, 0.1, |_| false).is_err());
    assert_eq!(before, state(&s));
    let mut u = fields(&g);
    u[0][0] = 1.0;
    let mut out = fields(&g);
    assert_eq!(
        w.update(refs(&u), 1.0, 0.1, 0.01, muts(&mut out), |_| false),
        Err(ViscosityError::NonzeroNormalWall)
    );
}

#[test]
fn anisotropic_three_dimensional_shear_matches_all_component_pairs() {
    for (a, b) in [(0, 1), (0, 2), (1, 2)] {
        let n = [6usize, 8, 4];
        let h = [0.25, 0.125, 0.5];
        let g = GridGeometry::new(n.map(|x| x as u64), h, [0.0; 3]).unwrap();
        let mut u = fields(&g);
        let wave = n.map(|n| std::f64::consts::PI / n as f64);
        let ratio = (wave[a] * 0.5).sin() / h[a] / ((wave[b] * 0.5).sin() / h[b]);
        for (d, field) in u.iter_mut().enumerate() {
            if d != a && d != b {
                continue;
            }
            let m = g.face_counts([Axis::X, Axis::Y, Axis::Z][d]);
            for k in 0..m[2] {
                for j in 0..m[1] {
                    for i in 0..m[0] {
                        let p = [i, j, k];
                        if p[d] == 0 || p[d] == n[d] {
                            continue;
                        }
                        let mut value = if d == a { 1.0 } else { -ratio };
                        for q in 0..3 {
                            value *= if q == d {
                                (wave[q] * p[q] as f64).sin()
                            } else {
                                (wave[q] * (p[q] as f64 + 0.5)).cos()
                            };
                        }
                        field[g.face_index([Axis::X, Axis::Y, Axis::Z][d], p).unwrap()] =
                            value as f32;
                    }
                }
            }
        }
        let lambda = (0..3)
            .map(|d| 4.0 * (wave[d] * 0.5).sin().powi(2) / h[d].powi(2))
            .sum::<f64>();
        let factor = 1.0 - 0.05 * 0.01 * lambda;
        let mut out = fields(&g);
        let mut w = ViscosityWorkspace::new(g, 1 << 20).unwrap();
        let r = w
            .update(refs(&u), 2.0, 0.1, 0.01, muts(&mut out), |_| false)
            .unwrap();
        assert!(r.kinetic_after < r.kinetic_before);
        for (&v, &old) in out.iter().flatten().zip(u.iter().flatten()) {
            assert!((f64::from(v) - factor * f64::from(old)).abs() < 3e-7);
        }
    }
}
#[test]
fn fixed_grid_time_refinement_and_zero_coefficient() {
    let g = grid(12);
    let initial = mode(&g);
    let lambda = 8.0 * (std::f64::consts::PI / 24.0).sin().powi(2) * 144.0;
    let mut previous = f64::INFINITY;
    let mut w = ViscosityWorkspace::new(g.clone(), 1 << 20).unwrap();
    let mut zero = fields(&g);
    let r = w
        .update(refs(&initial), 1.0, 0.0, 0.01, muts(&mut zero), |_| false)
        .unwrap();
    assert_eq!(initial, zero);
    assert_eq!(r.dissipation_before, 0.0);
    for steps in [32, 64, 128] {
        let mut u = initial.clone();
        let mut out = fields(&g);
        let dt = 0.1 / steps as f64;
        for _ in 0..steps {
            w.update(refs(&u), 1.0, 0.05, dt, muts(&mut out), |_| false)
                .unwrap();
            std::mem::swap(&mut u, &mut out);
        }
        let exact = (-0.05 * 0.1 * lambda).exp();
        let error = u
            .iter()
            .flatten()
            .zip(initial.iter().flatten())
            .map(|(&a, &b)| (f64::from(a) - exact * f64::from(b)).abs())
            .fold(0.0_f64, f64::max);
        assert!(error < previous * 0.55);
        previous = error;
    }
}

#[test]
fn large_mesh_compensated_energy_gate_preserves_fixed_budget() {
    // The ordinary diagnostic sum failed before interval 35 for the native
    // 64-cell small-step study. Accuracy at long f32 histories is a separate
    // retained limitation; this regression checks the unchanged energy gate.
    let g = GridGeometry::new([64, 64, 3], [1.0 / 64.0, 1.0 / 64.0, 0.25], [0.0; 3]).unwrap();
    let mut u = mode(&g); // populate all slices independently below
    for k in 1..3 {
        for j in 0..64 {
            for i in 1..64 {
                let x = u[0][g.face_index(Axis::X, [i, j, 0]).unwrap()];
                let y = u[1][g.face_index(Axis::Y, [j, i, 0]).unwrap()];
                u[0][g.face_index(Axis::X, [i, j, k]).unwrap()] = x;
                u[1][g.face_index(Axis::Y, [j, i, k]).unwrap()] = y;
            }
        }
    }
    let mut out = fields(&g);
    let mut w = ViscosityWorkspace::new(g, 1 << 24).unwrap();
    for _ in 0..40 {
        let r = w
            .update(refs(&u), 3.0, 0.15, 0.1 / 2053.0, muts(&mut out), |_| false)
            .unwrap();
        assert!(r.energy_identity_error.abs() <= r.energy_rounding_budget);
        assert!(r.kinetic_after <= r.kinetic_before + r.energy_rounding_budget);
        std::mem::swap(&mut u, &mut out);
    }
}
