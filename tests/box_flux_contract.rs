use rheon::{
    Axis, BoxFluxError, BoxFluxSettings, BoxFluxStage, BoxFluxStamp, BoxFluxWorkspace,
    GridGeometry, OperatorError, PrescribedBoxFlux, PressureError, PressureImplementation,
    PressureOperator, PressureSettings, PressureWorkspace,
};
fn settings() -> BoxFluxSettings {
    BoxFluxSettings {
        pressure: PressureSettings {
            relative_residual: 1e-12,
            absolute_residual: 1e-12,
            divergence_limit: 1e-9,
            max_iterations: 1000,
        },
        actual_divergence_limit: 1e-5,
    }
}
fn flux(outward: [[f32; 2]; 3]) -> PrescribedBoxFlux {
    PrescribedBoxFlux::new(BoxFluxStamp { id: 37, version: 8 }, outward).unwrap()
}
fn fields(g: &GridGeometry, value: f32) -> [Vec<f32>; 3] {
    [Axis::X, Axis::Y, Axis::Z].map(|a| vec![value; g.face_len(a)])
}
fn refs(v: &[Vec<f32>; 3]) -> [&[f32]; 3] {
    [&v[0], &v[1], &v[2]]
}
fn outs(v: &mut [Vec<f32>; 3]) -> [&mut [f32]; 3] {
    let [x, y, z] = v;
    [x, y, z]
}
fn near(a: f64, b: f64) {
    assert!(
        (a - b).abs() < 2e-11 * a.abs().max(b.abs()).max(1.0),
        "{a} != {b}"
    );
}
fn boundary_speed(f: PrescribedBoxFlux, d: usize, upper: bool) -> f32 {
    let q = f.outward_speeds()[d][usize::from(upper)];
    if upper { q } else { -q }
}
fn imposed(g: &GridGeometry, old: &[Vec<f32>; 3], f: PrescribedBoxFlux) -> [Vec<f32>; 3] {
    let mut v = old.clone();
    for (d, axis) in [Axis::X, Axis::Y, Axis::Z].into_iter().enumerate() {
        let [nx, ny, nz] = g.face_counts(axis);
        for k in 0..nz {
            for j in 0..ny {
                for i in 0..nx {
                    let p = [i, j, k];
                    if p[d] == 0 || p[d] == g.counts()[d] {
                        v[d][g.face_index(axis, p).unwrap()] = boundary_speed(f, d, p[d] != 0);
                    }
                }
            }
        }
    }
    v
}
fn independent_divergence(g: &GridGeometry, v: &[Vec<f32>; 3]) -> Vec<f64> {
    let [nx, ny, nz] = g.counts();
    let h = g.spacing();
    let mut out = vec![];
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let x = |x| f64::from(v[0][g.face_index(Axis::X, [x, j, k]).unwrap()]);
                let y = |y| f64::from(v[1][g.face_index(Axis::Y, [i, y, k]).unwrap()]);
                let z = |z| f64::from(v[2][g.face_index(Axis::Z, [i, j, z]).unwrap()]);
                out.push(
                    (x(i + 1) - x(i)) / h[0] + (y(j + 1) - y(j)) / h[1] + (z(k + 1) - z(k)) / h[2],
                );
            }
        }
    }
    out
}
#[test]
fn signed_through_flow_on_each_axis_matches_hand_pressure_energy_and_boundary_values() {
    for method in PressureImplementation::ALL {
        for axis in 0..3 {
            for speed in [-0.25_f32, 0.25] {
                let mut dims = [1; 3];
                dims[axis] = 3;
                let g = GridGeometry::new(dims, [1.0; 3], [0.0; 3]).unwrap();
                let mut bc = [[0.0; 2]; 3];
                bc[axis] = [-speed, speed];
                let boundary = flux(bc);
                let mut old = fields(&g, 0.0);
                // Irrelevant provisional outer values are replaced, not pressure-corrected.
                for (d, a) in [Axis::X, Axis::Y, Axis::Z].into_iter().enumerate() {
                    let [nx, ny, nz] = g.face_counts(a);
                    for k in 0..nz {
                        for j in 0..ny {
                            for i in 0..nx {
                                let p = [i, j, k];
                                if p[d] == 0 || p[d] == g.counts()[d] {
                                    old[d][g.face_index(a, p).unwrap()] = 17.0;
                                }
                            }
                        }
                    }
                }
                let saved = old.clone();
                let mut out = fields(&g, -777.0);
                let mut w = BoxFluxWorkspace::new(g.clone(), 2.0, 4096, method).unwrap();
                let r = w
                    .project(
                        refs(&old),
                        0.5,
                        boundary,
                        settings(),
                        outs(&mut out),
                        |_| false,
                    )
                    .unwrap();
                assert_eq!(old, saved);
                assert_eq!(r.boundary, boundary.stamp());
                assert_eq!(r.net_outward_flux, 0.0);
                for (i, &p) in w.pressure().iter().enumerate() {
                    near(p, -4.0 * f64::from(speed) * i as f64);
                }
                for (d, component) in out.iter().enumerate() {
                    assert!(
                        component
                            .iter()
                            .all(|&v| v == if d == axis { speed } else { 0.0 })
                    );
                }
                assert_eq!(r.actual_divergence_max, 0.0);
                near(r.work.kinetic_before, 0.0);
                near(r.work.kinetic_after, 0.125);
                near(r.work.correction_energy, 0.125);
                near(r.work.boundary_pressure_work, 0.25);
                near(r.work.divergence_residual_work, 0.0);
                near(r.work.correction_residual_work, 0.0);
                near(r.work.budget_error, 0.0);
            }
        }
    }
}

fn dense_solve(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Vec<f64> {
    let n = b.len();
    for i in 0..n {
        let pivot = (i..n)
            .max_by(|&x, &y| a[x][i].abs().total_cmp(&a[y][i].abs()))
            .unwrap();
        a.swap(i, pivot);
        b.swap(i, pivot);
        assert!(a[i][i].abs() > 1e-14);
        let diagonal = a[i][i];
        for value in &mut a[i][i..] {
            *value /= diagonal;
        }
        let pivot_row = a[i][i..].to_vec();
        b[i] /= diagonal;
        for row in 0..n {
            if row != i {
                let factor = a[row][i];
                for (value, pivot_value) in a[row][i..].iter_mut().zip(&pivot_row) {
                    *value -= factor * pivot_value;
                }
                b[row] -= factor * b[i];
            }
        }
    }
    b
}
fn dense_pressure(
    g: &GridGeometry,
    old: &[Vec<f32>; 3],
    f: PrescribedBoxFlux,
    rho: f64,
    dt: f64,
) -> Vec<f64> {
    let n = g.cell_len();
    let h = g.spacing();
    let area = [h[1] * h[2], h[0] * h[2], h[0] * h[1]];
    let mut matrix = vec![vec![0.0; n]; n];
    let [nx, ny, nz] = g.counts();
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let p = [i, j, k];
                let row = g.cell_index(p).unwrap();
                for d in 0..3 {
                    if p[d] + 1 < g.counts()[d] {
                        let mut q = p;
                        q[d] += 1;
                        let column = g.cell_index(q).unwrap();
                        let weight = area[d] / (rho * h[d]);
                        matrix[row][row] += weight;
                        matrix[column][column] += weight;
                        matrix[row][column] -= weight;
                        matrix[column][row] -= weight;
                    }
                }
            }
        }
    }
    let full = imposed(g, old, f);
    let rhs: Vec<f64> = independent_divergence(g, &full)
        .iter()
        .map(|d| -g.cell_volume() * d / dt)
        .collect();
    let reduced: Vec<Vec<f64>> = (1..n).map(|i| matrix[i][1..].to_vec()).collect();
    let mut p = vec![0.0];
    p.extend(dense_solve(reduced, rhs[1..].to_vec()));
    p
}
fn mixed() -> (GridGeometry, [Vec<f32>; 3], PrescribedBoxFlux) {
    let g = GridGeometry::new([3, 2, 1], [0.5, 1.0, 2.0], [-1.0, 2.0, 0.0]).unwrap();
    let mut old = fields(&g, 0.0);
    for (d, v) in old.iter_mut().enumerate() {
        for (i, u) in v.iter_mut().enumerate() {
            *u = ((i * 3 + d) % 7) as f32 * 0.13 - 0.2;
        }
    }
    (g, old, flux([[-0.375, 0.0], [0.0, 0.5], [0.0, 0.0]]))
}
#[test]
fn unequal_side_areas_match_independent_dense_projection_and_all_cell_divergence() {
    let (g, old, bc) = mixed();
    let expected = dense_pressure(&g, &old, bc, 1.5, 0.2);
    for method in PressureImplementation::ALL {
        let mut w = BoxFluxWorkspace::new(g.clone(), 1.5, 4096, method).unwrap();
        let mut out = fields(&g, 0.0);
        let r = w
            .project(refs(&old), 0.2, bc, settings(), outs(&mut out), |_| false)
            .unwrap();
        for (&a, &b) in w.pressure().iter().zip(&expected) {
            near(a, b);
        }
        let imposed = imposed(&g, &old, bc);
        for (d, axis) in [Axis::X, Axis::Y, Axis::Z].into_iter().enumerate() {
            let [nx, ny, nz] = g.face_counts(axis);
            for k in 0..nz {
                for j in 0..ny {
                    for i in 0..nx {
                        let p = [i, j, k];
                        let index = g.face_index(axis, p).unwrap();
                        if p[d] == 0 || p[d] == g.counts()[d] {
                            assert_eq!(out[d][index], imposed[d][index]);
                        } else {
                            let mut left = p;
                            left[d] -= 1;
                            let v = f64::from(old[d][index])
                                - 0.2 / (1.5 * g.spacing()[d])
                                    * (expected[g.cell_index(p).unwrap()]
                                        - expected[g.cell_index(left).unwrap()]);
                            assert!((f64::from(out[d][index]) - v).abs() < 1e-6);
                        }
                    }
                }
            }
        }
        let div = independent_divergence(&g, &out);
        let actual = div.iter().fold(0.0_f64, |a, b| a.max(b.abs()));
        assert_eq!(actual, r.actual_divergence_max);
        let mut direct = vec![0.0; g.cell_len()];
        w.divergence(refs(&out), &mut direct).unwrap();
        assert_eq!(direct, div);
        near(r.work.budget_error, 0.0);
    }
}
#[test]
fn affine_adjoint_requires_boundary_work_and_is_gauge_invariant_when_balanced() {
    let (g, old, bc) = mixed();
    let v = imposed(&g, &old, bc);
    let h = g.spacing();
    let area = [h[1] * h[2], h[0] * h[2], h[0] * h[1]];
    let pressure: Vec<f64> = (0..g.cell_len()).map(|i| i as f64 * 0.75 - 1.0).collect();
    let div = independent_divergence(&g, &v);
    let lhs: f64 = pressure
        .iter()
        .zip(div)
        .map(|(p, d)| p * g.cell_volume() * d)
        .sum();
    let mut interior = 0.0;
    let mut boundary = 0.0;
    let mut shifted = 0.0;
    let [nx, ny, nz] = g.counts();
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let p = [i, j, k];
                let row = g.cell_index(p).unwrap();
                let mut q = 0.0;
                for d in 0..3 {
                    if p[d] == 0 {
                        q += area[d] * f64::from(bc.outward_speeds()[d][0]);
                    }
                    if p[d] + 1 == g.counts()[d] {
                        q += area[d] * f64::from(bc.outward_speeds()[d][1]);
                    }
                    if p[d] + 1 < g.counts()[d] {
                        let mut right = p;
                        right[d] += 1;
                        let col = g.cell_index(right).unwrap();
                        let axis = [Axis::X, Axis::Y, Axis::Z][d];
                        interior -= area[d]
                            * (pressure[col] - pressure[row])
                            * f64::from(v[d][g.face_index(axis, right).unwrap()]);
                    }
                }
                boundary += pressure[row] * q;
                shifted += (pressure[row] + 7.0) * q;
            }
        }
    }
    near(lhs, interior + boundary);
    near(boundary, shifted);
    assert!(boundary.abs() > 0.1);
    assert!((lhs - interior).abs() > 0.1);
}
#[test]
fn known_net_flux_rejects_even_when_provisional_rhs_rounding_scale_is_enormous() {
    let g = GridGeometry::new([3, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let old = fields(&g, 1e20);
    let bc = flux([[-1.0, 0.0], [0.0; 2], [0.0; 2]]);
    for method in PressureImplementation::ALL {
        let mut w = BoxFluxWorkspace::new(g.clone(), 1.0, 4096, method).unwrap();
        let mut out = fields(&g, -777.0);
        let saved = out.clone();
        assert!(matches!(
            w.project(refs(&old), 0.1, bc, settings(), outs(&mut out), |_| false),
            Err(BoxFluxError::IncompatibleFlux { net: -1.0, .. })
        ));
        assert_eq!(out, saved);
    }
}
#[test]
fn zero_flux_matches_preserved_operator_solver_and_stored_face_bits() {
    let (g, old, _) = mixed();
    let zero = flux([[0.0; 2]; 3]);
    let old = imposed(&g, &old, zero);
    for method in PressureImplementation::ALL {
        let op = PressureOperator::new(&g, 1.5).unwrap();
        let mut original = PressureWorkspace::with_implementation(&g, 4096, method).unwrap();
        let report = original
            .solve_velocity(&op, refs(&old), 0.2, settings().pressure, || false)
            .unwrap();
        let mut expected = fields(&g, 0.0);
        op.correct_velocity(refs(&old), original.pressure(), 0.2, outs(&mut expected))
            .unwrap();
        let mut w = BoxFluxWorkspace::new(g.clone(), 1.5, 4096, method).unwrap();
        let mut out = fields(&g, 0.0);
        let r = w
            .project(refs(&old), 0.2, zero, settings(), outs(&mut out), |_| false)
            .unwrap();
        assert_eq!(w.pressure(), original.pressure());
        assert_eq!(r.pressure.iterations, report.iterations);
        for d in 0..3 {
            assert_eq!(
                out[d].iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
                expected[d].iter().map(|v| v.to_bits()).collect::<Vec<_>>()
            );
        }
        assert!(r.work.kinetic_after <= r.work.kinetic_before + 1e-14);
        near(r.work.boundary_pressure_work, 0.0);
        near(r.work.budget_error, 0.0);
    }
}
#[test]
fn cancellation_has_explicit_partial_output_semantics_and_retry_preserves_inputs() {
    let (g, old, bc) = mixed();
    let saved = old.clone();
    for method in PressureImplementation::ALL {
        for stage in [
            BoxFluxStage::BeforeSolve,
            BoxFluxStage::PressureIteration,
            BoxFluxStage::CorrectionSlice,
            BoxFluxStage::BeforeAcceptance,
        ] {
            let mut w = BoxFluxWorkspace::new(g.clone(), 1.5, 4096, method).unwrap();
            let mut out = fields(&g, -777.0);
            let before = out.clone();
            let mut slices = 0;
            let result = w.project(refs(&old), 0.2, bc, settings(), outs(&mut out), |s| {
                if s == BoxFluxStage::CorrectionSlice {
                    slices += 1;
                }
                s == stage && (stage != BoxFluxStage::CorrectionSlice || slices == 2)
            });
            if stage == BoxFluxStage::PressureIteration {
                assert!(matches!(
                    result,
                    Err(BoxFluxError::Pressure(PressureError::Cancelled { .. }))
                ));
            } else {
                assert!(matches!(result,Err(BoxFluxError::Cancelled {stage:s}) if s==stage));
            }
            if matches!(
                stage,
                BoxFluxStage::BeforeSolve | BoxFluxStage::PressureIteration
            ) {
                assert_eq!(out, before);
            } else {
                assert_ne!(out, before);
            }
            assert_eq!(old, saved);
            assert_eq!(bc.stamp(), BoxFluxStamp { id: 37, version: 8 });
            let r = w
                .project(refs(&old), 0.2, bc, settings(), outs(&mut out), |_| false)
                .unwrap();
            let mut fresh = BoxFluxWorkspace::new(g.clone(), 1.5, 4096, method).unwrap();
            let mut expected = fields(&g, 0.0);
            let reference = fresh
                .project(refs(&old), 0.2, bc, settings(), outs(&mut expected), |_| {
                    false
                })
                .unwrap();
            assert_eq!(out, expected);
            assert_eq!(w.pressure(), fresh.pressure());
            assert_eq!(r.pressure.iterations, reference.pressure.iterations);
        }
    }
}
#[test]
fn physical_stored_divergence_gate_rejects_after_solver_acceptance() {
    let (g, old, bc) = mixed();
    let mut strict = settings();
    strict.actual_divergence_limit = 0.0;
    for method in PressureImplementation::ALL {
        let mut w = BoxFluxWorkspace::new(g.clone(), 1.5, 4096, method).unwrap();
        let mut out = fields(&g, -777.0);
        assert!(
            matches!(w.project(refs(&old),0.2,bc,strict,outs(&mut out),|_|false),Err(BoxFluxError::DivergenceLimit {actual,..}) if actual>0.0)
        );
        w.project(refs(&old), 0.2, bc, settings(), outs(&mut out), |_| false)
            .unwrap();
    }
}
#[test]
fn iteration_budget_failure_precedes_any_face_output_write() {
    let (g, old, bc) = mixed();
    let mut budget = settings();
    budget.pressure.max_iterations = 0;
    let mut w =
        BoxFluxWorkspace::new(g.clone(), 1.5, 4096, PressureImplementation::default()).unwrap();
    let mut out = fields(&g, -777.0);
    let before = out.clone();
    assert!(matches!(
        w.project(refs(&old), 0.2, bc, budget, outs(&mut out), |_| false),
        Err(BoxFluxError::Pressure(PressureError::IterationLimit {
            iterations: 0,
            ..
        }))
    ));
    assert_eq!(out, before);
}
#[test]
fn singleton_flow_and_exact_capacity_limits_are_supported() {
    for method in PressureImplementation::ALL {
        let g = GridGeometry::new([1, 1, 1], [0.5, 1.0, 2.0], [0.0; 3]).unwrap();
        assert!(matches!(
            BoxFluxWorkspace::new(g.clone(), 1.0, 55, method),
            Err(BoxFluxError::BufferLimit {
                required: 56,
                limit: 55
            })
        ));
        let mut w = BoxFluxWorkspace::new(g.clone(), 1.0, 56, method).unwrap();
        assert_eq!(w.allocated_bytes(), 56);
        let old = fields(&g, 0.0);
        let mut out = fields(&g, 0.0);
        let r = w
            .project(
                refs(&old),
                0.25,
                flux([[-0.5, 0.5], [0.0; 2], [0.0; 2]]),
                settings(),
                outs(&mut out),
                |_| false,
            )
            .unwrap();
        assert_eq!(r.pressure.iterations, 0);
        assert_eq!(r.actual_divergence_max, 0.0);
        assert_eq!(w.pressure(), [0.0]);
        assert_eq!(r.work.kinetic_before, 0.0);
        assert_eq!(r.work.kinetic_after, 0.0);
    }
}
#[test]
fn invalid_inputs_reject_before_output_mutation() {
    assert!(matches!(
        PrescribedBoxFlux::new(BoxFluxStamp { id: 0, version: 0 }, [[f32::NAN; 2]; 3]),
        Err(BoxFluxError::InvalidBoundary)
    ));
    let g = GridGeometry::new([2, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    assert!(matches!(
        BoxFluxWorkspace::new(g.clone(), 0.0, 4096, PressureImplementation::default()),
        Err(BoxFluxError::Operator(OperatorError::InvalidDensity))
    ));
    for (spacing, density) in [
        ([1e-100, 1e-100, 1.0], 1e-200),
        ([1e100, 1e100, 1.0], 1e200),
    ] {
        let scaled = GridGeometry::new([2, 1, 1], spacing, [0.0; 3]).unwrap();
        assert!(matches!(
            BoxFluxWorkspace::new(scaled, density, 4096, PressureImplementation::default()),
            Err(BoxFluxError::InvalidMass)
        ));
    }
    let mut w =
        BoxFluxWorkspace::new(g.clone(), 1.0, 4096, PressureImplementation::default()).unwrap();
    let old = fields(&g, 0.0);
    let mut out = fields(&g, -777.0);
    let before = out.clone();
    let bc = flux([[0.0; 2]; 3]);
    assert!(matches!(
        w.project(refs(&old), 0.0, bc, settings(), outs(&mut out), |_| false),
        Err(BoxFluxError::Operator(OperatorError::InvalidTimeStep))
    ));
    assert_eq!(out, before);
    let mut bad = old.clone();
    bad[0][0] = f32::NAN;
    assert!(matches!(
        w.project(refs(&bad), 0.1, bc, settings(), outs(&mut out), |_| false),
        Err(BoxFluxError::Operator(OperatorError::NonFiniteInput))
    ));
    assert_eq!(out, before);
    let mut invalid = settings();
    invalid.actual_divergence_limit = f64::NAN;
    assert!(matches!(
        w.project(refs(&old), 0.1, bc, invalid, outs(&mut out), |_| false),
        Err(BoxFluxError::InvalidSettings)
    ));
    assert_eq!(out, before);
    let mut short = fields(&g, -777.0);
    short[2].pop();
    let saved = short.clone();
    assert!(matches!(
        w.project(refs(&old), 0.1, bc, settings(), outs(&mut short), |_| false),
        Err(BoxFluxError::Operator(OperatorError::LengthMismatch))
    ));
    assert_eq!(short, saved);
}

#[test]
fn nonzero_boundary_flux_cannot_underflow_out_of_the_compatibility_gate() {
    let grid = GridGeometry::new([1, 1, 1], [1e100, 1e-150, 1e-150], [0.0; 3]).unwrap();
    let mut w = BoxFluxWorkspace::new(
        grid.clone(),
        1e-100,
        4096,
        PressureImplementation::default(),
    )
    .unwrap();
    let old = fields(&grid, 0.0);
    let mut output = fields(&grid, -777.0);
    let saved = output.clone();
    let bc = flux([[1e-30, 0.0], [0.0; 2], [0.0; 2]]);
    let error = w
        .project(refs(&old), 1.0, bc, settings(), outs(&mut output), |_| {
            false
        })
        .unwrap_err();
    assert_eq!(error, BoxFluxError::ArithmeticFailure);
    assert_eq!(output, saved);
}
