use rheon::*;
fn construct(
    points: &[[f64; 2]],
    width: f64,
    density: f64,
    mu: f64,
    settings: FittedHeightSettings,
    cancel: impl FnMut(FittedHeightStage) -> bool,
) -> Result<FittedHeightWorkspace, FittedHeightError> {
    let bottom = points.iter().map(|p| p[0]).collect::<Vec<_>>();
    FittedHeightWorkspace::new(
        FittedHeightGeometry {
            cap: points,
            bottom_x: &bottom,
            extrusion_width: width,
            density,
            dynamic_viscosity: mu,
        },
        settings,
        cancel,
    )
}
fn cap(c: usize) -> Vec<[f64; 2]> {
    (0..=c)
        .map(|i| {
            [
                i as f64 / c as f64,
                if i == c {
                    1.0
                } else {
                    1.0 + 0.25 * ((i * i) % 3) as f64
                },
            ]
        })
        .collect()
}
fn reference_cap() -> Vec<[f64; 2]> {
    vec![
        [0.0, 1.0],
        [0.25, 1.25],
        [0.5, 1.0],
        [0.75, 1.5],
        [1.0, 1.0],
    ]
}
fn workspace(points: &[[f64; 2]]) -> FittedHeightWorkspace {
    construct(
        points,
        1.0,
        3.0,
        0.5,
        FittedHeightSettings::default(),
        |_| false,
    )
    .unwrap()
}
fn field(w: &FittedHeightWorkspace, f: impl Fn([f64; 2]) -> [f64; 3]) -> Vec<[f64; 3]> {
    let mut z = vec![0.0; w.plan().reduced_velocity_unknowns];
    for n in w.nodes() {
        for d in 0..3 {
            let row = w.velocity_embedding(n.periodic_index, d).unwrap();
            if row.weights[0] == 1.0
                && let Some(column) = row.columns[0]
            {
                z[column] = f(n.position)[d];
            }
        }
    }
    let mut u = vec![[0.0; 3]; w.plan().periodic_nodes];
    w.embed_velocity(&z, &mut u).unwrap();
    u
}
fn near(a: f64, b: f64, t: f64) {
    assert!(
        (a - b).abs() <= t * (1.0 + a.abs() + b.abs()),
        "{a:.17e} != {b:.17e}"
    );
}
fn sentinel(n: usize) -> Vec<FittedHeightNodeDiagnostic> {
    vec![
        FittedHeightNodeDiagnostic {
            mass_rate: 13.0,
            mass_flux_sum: -9.0,
            divergence_mass: 7.0,
            convection: [2.0; 3],
            strain_force: [-3.0; 3],
            pressure_force: [5.0; 3]
        };
        n
    ]
}
fn inspect(
    w: &mut FittedHeightWorkspace,
    u: &[[f64; 3]],
    p: &[f64],
) -> (FittedHeightReport, Vec<FittedHeightNodeDiagnostic>) {
    let mut out = sentinel(w.plan().periodic_nodes);
    let r = w
        .inspect(
            FittedHeightInputs {
                velocity: u,
                pressure_coefficients: p,
            },
            &mut out,
            |_| false,
        )
        .unwrap();
    (r, out)
}
#[test]
fn declared_capacity_and_real_payload_are_bounded() {
    let w = workspace(&reference_cap());
    let plan = w.plan();
    assert_eq!(
        (
            plan.geometric_nodes,
            plan.periodic_nodes,
            plan.triangles,
            plan.pressure_modes
        ),
        (35, 32, 48, 33)
    );
    assert_eq!(w.allocated_bytes(), plan.nominal_bytes);
    let settings = FittedHeightSettings {
        memory_limit: plan.nominal_bytes - 1,
        ..Default::default()
    };
    assert!(matches!(
        construct(&reference_cap(), 1.0, 3.0, 0.5, settings, |_| false),
        Err(FittedHeightError::BufferLimit { .. })
    ));
    assert_eq!(
        FittedHeightPlan::new(usize::MAX),
        Err(FittedHeightError::IntegerOverflow)
    );
    assert_eq!(
        FittedHeightPlan::new(1),
        Err(FittedHeightError::InvalidGeometry)
    );
    let settings = FittedHeightSettings {
        max_columns: 3,
        ..Default::default()
    };
    assert!(matches!(
        construct(&reference_cap(), 1.0, 3.0, 0.5, settings, |_| false),
        Err(FittedHeightError::ColumnLimit)
    ));
    let settings = FittedHeightSettings {
        max_pressure_terms: 1,
        ..Default::default()
    };
    assert!(matches!(
        construct(&reference_cap(), 1.0, 3.0, 0.5, settings, |_| false),
        Err(FittedHeightError::PressureTermLimit)
    ));
}
#[test]
fn shared_geometry_mass_and_affine_gradient_partition() {
    let w = workspace(&reference_cap());
    near(w.nodal_mass().iter().sum(), 57.0 / 16.0, 2e-15);
    assert!(w.nodal_mass().iter().all(|&m| m > 0.0 && m.is_normal()));
    let mut area = 0.0;
    for tri in w.triangles() {
        area += tri.area;
        for d in 0..2 {
            near(tri.gradients.iter().map(|g| g[d]).sum(), 0.0, 2e-14);
            for e in 0..2 {
                near(
                    (0..3)
                        .map(|j| tri.gradients[j][d] * w.nodes()[tri.nodes[j]].position[e])
                        .sum(),
                    if d == e { 1.0 } else { 0.0 },
                    2e-14,
                );
            }
        }
    }
    near(area, 19.0 / 16.0, 2e-15);
}
#[test]
fn full_vector_affine_strain_and_prescribed_traction_patch() {
    let w = workspace(&reference_cap());
    let (mu, pi) = (0.5, 1.0 / 3.0);
    let mut total = 0.0;
    let sigma = [
        [mu - pi, mu * 0.5, mu * 0.5],
        [mu * 0.5, -mu - pi, mu * 0.25],
        [mu * 0.5, mu * 0.25, -pi],
    ];
    for (t, tri) in w.triangles().iter().enumerate() {
        let k = w.triangle_stiffness(t).unwrap();
        let p = tri.nodes.map(|i| w.nodes()[i].position);
        let v = p.map(|[x, y]| [0.5 * x + 0.75 * y, -0.25 * x - 0.5 * y, 0.5 * x + 0.25 * y]);
        let mut load = [[0.0; 3]; 3];
        for j in 0..3 {
            let h = (j + 1) % 3;
            let normal = [p[h][1] - p[j][1], p[j][0] - p[h][0], 0.0];
            for d in 0..3 {
                let traction = (0..3).map(|e| sigma[d][e] * normal[e]).sum::<f64>();
                load[j][d] += traction / 2.0;
                load[h][d] += traction / 2.0;
            }
        }
        for a in 0..9 {
            let ku = (0..9).map(|b| k[a][b] * v[b / 3][b % 3]).sum::<f64>();
            let pf = if a % 3 < 2 {
                -tri.area * pi * tri.gradients[a / 3][a % 3]
            } else {
                0.0
            };
            near(ku + pf, load[a / 3][a % 3], 1e-13);
            total += v[a / 3][a % 3] * ku;
            for (b, row) in k.iter().enumerate() {
                near(k[a][b], row[a], 1e-15);
            }
        }
    }
    near(total, 475.0 / 512.0, 2e-13);
}
#[test]
fn rigid_rotation_and_full_vector_translation_are_strain_null_modes() {
    let w = workspace(&reference_cap());
    for (t, tri) in w.triangles().iter().enumerate() {
        let k = w.triangle_stiffness(t).unwrap();
        for mode in 0..2 {
            let v = tri.nodes.map(|i| {
                let [x, y] = w.nodes()[i].position;
                if mode == 0 {
                    [-y, x, 0.125]
                } else {
                    [0.25, -0.75, 0.125]
                }
            });
            for row in &k {
                near((0..9).map(|j| row[j] * v[j / 3][j % 3]).sum(), 0.0, 3e-13);
            }
        }
    }
}
fn cholesky(a: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = a.len();
    let mut l = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..=i {
            let s = a[i][j] - (0..j).map(|k| l[i][k] * l[j][k]).sum::<f64>();
            if i == j {
                assert!(s > 1e-10, "nonpositive pressure Gram pivot {s}");
                l[i][j] = s.sqrt();
            } else {
                l[i][j] = s / l[j][j];
            }
        }
    }
    l
}
#[test]
fn numerical_pressure_image_rank_constant_mode_and_transpose() {
    for c in [2, 3, 4, 6, 8, 12] {
        let w = workspace(&cap(c));
        let plan = w.plan();
        assert_eq!(plan.pressure_modes, 8 * c + 1);
        let mut q = vec![vec![0.0; plan.triangles]; plan.pressure_modes];
        for term in w.pressure_basis() {
            q[term.mode][term.triangle] += term.value;
        }
        let gram = (0..q.len())
            .map(|i| {
                (0..q.len())
                    .map(|j| {
                        (0..plan.triangles)
                            .map(|t| w.triangles()[t].area * q[i][t] * q[j][t])
                            .sum()
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let l = cholesky(&gram);
        // B on the selected admissible velocity columns is minus this Gram.
        for (mode, &col) in w.pressure_columns().iter().enumerate() {
            let mut coeff = vec![0.0; plan.reduced_velocity_unknowns];
            coeff[col] = 1.0;
            let mut u = vec![[0.0; 3]; plan.periodic_nodes];
            w.embed_velocity(&coeff, &mut u).unwrap();
            for (t, tri) in w.triangles().iter().enumerate() {
                let d = (0..3)
                    .map(|j| {
                        let v = u[w.nodes()[tri.nodes[j]].periodic_index];
                        tri.gradients[j][0] * v[0] + tri.gradients[j][1] * v[1]
                    })
                    .sum();
                near(d, q[mode][t], 1e-14);
            }
        }
        // Solve the positive Gram for the constant pressure representation.
        let mut rhs = (0..q.len())
            .map(|i| {
                (0..plan.triangles)
                    .map(|t| w.triangles()[t].area * q[i][t])
                    .sum::<f64>()
            })
            .collect::<Vec<_>>();
        for i in 0..q.len() {
            rhs[i] = (rhs[i] - (0..i).map(|j| l[i][j] * rhs[j]).sum::<f64>()) / l[i][i];
        }
        for i in (0..q.len()).rev() {
            rhs[i] = (rhs[i] - (i + 1..q.len()).map(|j| l[j][i] * rhs[j]).sum::<f64>()) / l[i][i];
        }
        for t in 0..plan.triangles {
            near((0..q.len()).map(|i| rhs[i] * q[i][t]).sum(), 1.0, 1e-10);
        }
    }
}
#[test]
fn actual_graph_translation_has_shared_gcl_and_full_vector_work() {
    let mut w = workspace(&reference_cap());
    let u = field(&w, |[_, y]| [0.25, 0.0, y * y]);
    let p = vec![0.0; w.plan().pressure_modes];
    let (r, out) = inspect(&mut w, &u, &p);
    near(r.divergence_max, 0.0, 1e-13);
    near(r.continuity_defect_max, 0.0, 1e-13);
    near(r.geometric_identity_error, 0.0, 1e-13);
    near(r.total_mass_rate, 0.0, 1e-13);
    assert_eq!(out.iter().filter(|o| o.mass_rate.abs() > 1e-13).count(), 22);
    assert_eq!(
        w.shared_flux_scratch()
            .iter()
            .filter(|f| f.flux.abs() > 1e-13)
            .count(),
        76
    );
    near(r.strain_power, 2600634373.0 / 1259712000.0, 1e-13);
    near(
        r.advection_dissipation,
        11833343748992221.0 / 26447905382400000.0,
        1e-13,
    );
    near(r.convection_work_error, 0.0, 1e-13);
    near(r.strain_work_error, 0.0, 1e-13);
    for d in 0..3 {
        near(r.momentum_flux_sum[d], 0.0, 1e-13);
        near(r.strain_force_sum[d], 0.0, 1e-13);
    }
}
#[test]
fn constant_velocity_carries_every_component_with_changing_masses() {
    let mut w = workspace(&reference_cap());
    let u = field(&w, |_| [0.25, 0.0, -0.875]);
    let p = vec![0.0; w.plan().pressure_modes];
    let (r, out) = inspect(&mut w, &u, &p);
    near(r.strain_power, 0.0, 1e-25);
    near(r.advection_dissipation, 0.0, 1e-25);
    for (i, o) in out.iter().enumerate() {
        for (d, &v) in u[i].iter().enumerate() {
            near(o.mass_rate * v + o.convection[d], 0.0, 1e-13);
        }
    }
}
#[test]
fn divergence_defect_is_reported_and_nonzero_pressure_work_is_adjoint() {
    let mut w = workspace(&reference_cap());
    let u = field(&w, |[_, y]| [0.0, y, 0.125]);
    let p = (0..w.plan().pressure_modes)
        .map(|i| ((i * 7) % 13) as f64 / 11.0 - 0.5)
        .collect::<Vec<_>>();
    let (r, out) = inspect(&mut w, &u, &p);
    near(r.divergence_max, 1.0, 2e-13);
    near(r.total_mass_rate, 57.0 / 16.0, 2e-13);
    assert!(r.continuity_defect_max > 0.01);
    near(r.geometric_identity_error, 0.0, 2e-13);
    near(r.pressure_adjoint_error, 0.0, 2e-13);
    assert!(r.pressure_work.abs() > 1e-3);
    for (o, m) in out.iter().zip(w.nodal_mass()) {
        near(o.mass_rate + o.mass_flux_sum, *m, 2e-13);
    }
}
#[test]
fn every_cancellation_preserves_output_and_retry_matches() {
    let mut w = workspace(&reference_cap());
    let u = field(&w, |[_, y]| [0.25, 0.0, y * y]);
    let p = vec![0.0; w.plan().pressure_modes];
    let (expected_r, expected) = inspect(&mut w, &u, &p);
    let mut calls = 0;
    w.inspect(
        FittedHeightInputs {
            velocity: &u,
            pressure_coefficients: &p,
        },
        &mut expected.clone(),
        |_| {
            calls += 1;
            false
        },
    )
    .unwrap();
    let before = sentinel(w.plan().periodic_nodes);
    let allocated = w.allocated_bytes();
    for stop in 1..=calls {
        let mut count = 0;
        let mut out = before.clone();
        let e = w
            .inspect(
                FittedHeightInputs {
                    velocity: &u,
                    pressure_coefficients: &p,
                },
                &mut out,
                |_| {
                    count += 1;
                    count == stop
                },
            )
            .unwrap_err();
        assert!(matches!(e, FittedHeightError::Cancelled { .. }));
        assert_eq!(out, before);
        let (retry, actual) = inspect(&mut w, &u, &p);
        assert_eq!(actual, expected);
        assert_eq!(
            retry.strain_power.to_bits(),
            expected_r.strain_power.to_bits()
        );
        assert_eq!(w.allocated_bytes(), allocated);
    }
}
#[test]
fn arithmetic_kinematic_shape_failures_preserve_output() {
    let mut w = workspace(&reference_cap());
    let good = field(&w, |_| [0.25, 0.0, 0.125]);
    let p = vec![0.0; w.plan().pressure_modes];
    let before = sentinel(w.plan().periodic_nodes);
    for value in [f64::NAN, f64::INFINITY, f64::from_bits(1), 1e-200, 1e200] {
        let bad = field(&w, |_| [0.25, 0.0, 0.125]);
        let mut bad = bad;
        if value.is_finite() && value.is_normal() {
            bad = field(&w, |_| [0.25, 0.0, value]);
        } else {
            bad[0][0] = value;
        }
        let mut out = before.clone();
        assert!(
            w.inspect(
                FittedHeightInputs {
                    velocity: &bad,
                    pressure_coefficients: &p
                },
                &mut out,
                |_| false
            )
            .is_err()
        );
        assert_eq!(out, before);
    }
    let mut bad = good.clone();
    bad[0][1] = 1.0;
    let mut out = before.clone();
    assert_eq!(
        w.inspect(
            FittedHeightInputs {
                velocity: &bad,
                pressure_coefficients: &p
            },
            &mut out,
            |_| false
        )
        .unwrap_err(),
        FittedHeightError::KinematicMismatch
    );
    assert_eq!(out, before);
    assert_eq!(
        w.inspect(
            FittedHeightInputs {
                velocity: &good,
                pressure_coefficients: &p[..p.len() - 1]
            },
            &mut out,
            |_| false
        )
        .unwrap_err(),
        FittedHeightError::ShapeMismatch
    );
    assert_eq!(out, before);
    inspect(&mut w, &good, &p);
}
#[test]
fn marginal_correct_wrong_flux_circulation_is_rejected() {
    let mut w = workspace(&reference_cap());
    let u = field(&w, |_| [0.25, 0.0, 0.125]);
    let p = vec![0.0; w.plan().pressure_modes];
    inspect(&mut w, &u, &p);
    let input = FittedHeightInputs {
        velocity: &u,
        pressure_coefficients: &p,
    };
    let stored = w.shared_flux_scratch().to_vec();
    w.audit_shared_flux(input, &stored, |_| false).unwrap();
    let tri = w.triangles()[0].nodes.map(|i| w.nodes()[i].periodic_index);
    let mut wrong = stored.clone();
    let mut delta = vec![0.0; u.len()];
    for j in 0..3 {
        let i = tri[j];
        let k = tri[(j + 1) % 3];
        let pair = [i.min(k), i.max(k)];
        let f = wrong.iter_mut().find(|f| f.nodes == pair).unwrap();
        let change = if i < k { 0.001 } else { -0.001 };
        f.flux += change;
        delta[i] += 0.001;
        delta[k] -= 0.001;
    }
    assert!(delta.iter().all(|&x| x == 0.0));
    assert_eq!(
        w.audit_shared_flux(input, &wrong, |_| false),
        Err(FittedHeightError::ProvenanceMismatch)
    );
    w.audit_shared_flux(input, &stored, |_| false).unwrap();
}
#[test]
fn invalid_geometry_and_uncertain_pressure_rank_are_rejections() {
    let mut points = reference_cap();
    points[4][1] = 1.01;
    assert!(workspace_error(&points).is_some());
    points = reference_cap();
    points[1][0] = 0.0;
    assert!(workspace_error(&points).is_some());
    points = reference_cap();
    points[2][1] = 0.0;
    assert!(workspace_error(&points).is_some());
    let settings = FittedHeightSettings {
        rank_relative_tolerance: 0.9,
        ..Default::default()
    };
    assert!(matches!(
        construct(&reference_cap(), 1.0, 3.0, 0.5, settings, |_| false),
        Err(FittedHeightError::PressureRank { .. })
    ));
    assert!(matches!(
        construct(
            &reference_cap(),
            1.0,
            3.0,
            0.5,
            FittedHeightSettings::default(),
            |s| s == FittedHeightStage::PressureImage
        ),
        Err(FittedHeightError::Cancelled { .. })
    ));
}
fn workspace_error(points: &[[f64; 2]]) -> Option<FittedHeightError> {
    construct(
        points,
        1.0,
        3.0,
        0.5,
        FittedHeightSettings::default(),
        |_| false,
    )
    .err()
}

#[test]
fn actual_rebuilt_geometry_matches_mass_derivative() {
    let points = reference_cap();
    let bottom = points.iter().map(|p| p[0]).collect::<Vec<_>>();
    let mut w = workspace(&points);
    let u = field(&w, |[x, y]| {
        [0.2 + 0.03 * y, 0.07 * y * (1.0 + x * (1.0 - x)), 0.0]
    });
    let p = vec![0.0; w.plan().pressure_modes];
    let (_, out) = inspect(&mut w, &u, &p);
    let eps = 1e-5;
    let mut masses = Vec::new();
    for sign in [-1.0, 1.0] {
        let mut moved = points
            .iter()
            .enumerate()
            .map(|(i, &[x, y])| {
                let raw = points.len() + i;
                let v = u[w.nodes()[raw].periodic_index];
                [x + sign * eps * v[0], y + sign * eps * v[1]]
            })
            .collect::<Vec<_>>();
        let last = moved.len() - 1;
        moved[last][0] = moved[0][0] + 1.0;
        moved[last][1] = moved[0][1];
        let rebuilt = FittedHeightWorkspace::new(
            FittedHeightGeometry {
                cap: &moved,
                bottom_x: &bottom,
                extrusion_width: 1.0,
                density: 3.0,
                dynamic_viscosity: 0.5,
            },
            FittedHeightSettings::default(),
            |_| false,
        )
        .unwrap();
        masses.push(rebuilt.nodal_mass().to_vec());
    }
    for (i, o) in out.iter().enumerate() {
        near(
            (masses[1][i] - masses[0][i]) / (2.0 * eps),
            o.mass_rate,
            2e-9,
        );
    }
}

#[test]
fn density_width_viscosity_scaling_and_zero_viscosity_are_explicit() {
    let points = reference_cap();
    let mut a = workspace(&points);
    let mut b = construct(
        &points,
        2.0,
        1.5,
        0.25,
        FittedHeightSettings::default(),
        |_| false,
    )
    .unwrap();
    let mut zero = construct(
        &points,
        1.0,
        3.0,
        0.0,
        FittedHeightSettings::default(),
        |_| false,
    )
    .unwrap();
    let u = field(&a, |[_, y]| [0.25, 0.0, y * y]);
    let p = vec![0.0; a.plan().pressure_modes];
    let (ra, oa) = inspect(&mut a, &u, &p);
    let (rb, ob) = inspect(&mut b, &u, &p);
    let (rz, oz) = inspect(&mut zero, &u, &p);
    assert_eq!(a.nodal_mass(), b.nodal_mass());
    assert_eq!(oa, ob);
    assert_eq!(ra.strain_power.to_bits(), rb.strain_power.to_bits());
    assert_eq!(rz.strain_power, 0.0);
    assert!(oz.iter().all(|o| o.strain_force == [0.0; 3]));
}
