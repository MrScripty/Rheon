//! Independent small weighted-graph, hydrostatic and component boundary oracles.
//! These qualify M1 components only, not a coupled flow or native feature.
use rheon::voxel_material::{
    MaterialPair, MaterialProjectionWorkspace, MaterialProperties, VoxelMaterialError,
    VoxelMaterialState,
};
use rheon::{
    Axis, GridGeometry, OperatorError, PressureError, PressureOperator, PressureSettings,
    PressureWorkspace,
};

const AXES: [Axis; 3] = [Axis::X, Axis::Y, Axis::Z];
const LIMIT: usize = 16 * 1024 * 1024;
fn settings() -> PressureSettings {
    PressureSettings {
        relative_residual: 1e-12,
        absolute_residual: 1e-12,
        divergence_limit: 1e-8,
        max_iterations: 3000,
    }
}
fn grid() -> GridGeometry {
    GridGeometry::new([4, 5, 4], [0.2, 0.3, 0.4], [-1.2, 0.7, -0.3]).unwrap()
}
fn fields(g: &GridGeometry) -> [Vec<f64>; 3] {
    AXES.map(|a| vec![0.0; g.face_len(a)])
}
fn refs(v: &[Vec<f64>; 3]) -> [&[f64]; 3] {
    v.each_ref().map(|v| v.as_slice())
}
fn pair(ratio: f64) -> MaterialPair {
    MaterialPair::new(
        MaterialProperties::new(ratio, 0.001).unwrap(),
        MaterialProperties::new(1.0, 0.000018).unwrap(),
    )
}
fn fraction(g: &GridGeometry) -> Vec<f64> {
    (0..g.cell_len())
        .map(|i| [0.0, 1.0, 0.25, 0.75][i % 4])
        .collect()
}
fn density_faces(g: &GridGeometry, cell_rho: &[f64]) -> [Vec<f64>; 3] {
    let mut out = fields(g);
    let n = g.counts();
    for (d, &a) in AXES.iter().enumerate() {
        let size = g.face_counts(a);
        for k in 0..size[2] {
            for j in 0..size[1] {
                for i in 0..size[0] {
                    let p = [i, j, k];
                    let mut lo = p;
                    let mut hi = p;
                    lo[d] = p[d].saturating_sub(1);
                    hi[d] = p[d].min(n[d] - 1);
                    out[d][g.face_index(a, p).unwrap()] = (cell_rho[g.cell_index(lo).unwrap()]
                        + cell_rho[g.cell_index(hi).unwrap()])
                        * 0.5;
                }
            }
        }
    }
    out
}
// Assembles each undirected edge once into a dense symmetric graph matrix.
// It does not call operator indexing, diagonal, weights or apply_full.
fn dense(g: &GridGeometry, rho: &[Vec<f64>; 3]) -> Vec<Vec<f64>> {
    let n = g.cell_len();
    let mut m = vec![vec![0.0; n]; n];
    let [nx, ny, nz] = g.counts();
    let h = g.spacing();
    let v = g.cell_volume();
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let p = [i, j, k];
                let row = i + nx * (j + ny * k);
                for d in 0..3 {
                    if p[d] + 1 < g.counts()[d] {
                        let mut upper = p;
                        upper[d] += 1;
                        let col = upper[0] + nx * (upper[1] + ny * upper[2]);
                        let fd = g.face_counts(AXES[d]);
                        let f = upper[0] + fd[0] * (upper[1] + fd[1] * upper[2]);
                        let w = v / (h[d] * h[d] * rho[d][f]);
                        m[row][row] += w;
                        m[col][col] += w;
                        m[row][col] -= w;
                        m[col][row] -= w;
                    }
                }
            }
        }
    }
    m
}
fn matvec(m: &[Vec<f64>], x: &[f64]) -> Vec<f64> {
    m.iter()
        .map(|r| r.iter().zip(x).map(|(a, b)| a * b).sum())
        .collect()
}

#[test]
fn heterogeneous_operator_and_pcg_match_independent_dense_graph() {
    let g = grid();
    let f = fraction(&g);
    for ratio in [1.0, 2.0, 10.0, 100.0, 1000.0] {
        let rho: Vec<_> = f.iter().map(|f| 1.0 + f * (ratio - 1.0)).collect();
        let faces = density_faces(&g, &rho);
        let op = PressureOperator::with_face_densities(&g, refs(&faces)).unwrap();
        let m = dense(&g, &faces);
        let exact: Vec<_> = (0..g.cell_len())
            .map(|i| ((i % 7) as f64 - 3.0) * 0.25)
            .collect();
        let mut rhs = matvec(&m, &exact);
        let mut actual = vec![0.0; rhs.len()];
        op.apply_full(&exact, &mut actual).unwrap();
        for (a, b) in actual.iter().zip(&rhs) {
            assert!((a - b).abs() < 1e-12);
        }
        for (i, row) in m.iter().enumerate() {
            assert!(row.iter().sum::<f64>().abs() < 1e-12);
            for j in 0..m.len() {
                assert_eq!(row[j], m[j][i]);
            }
        }
        assert!(exact.iter().zip(&rhs).map(|(a, b)| a * b).sum::<f64>() > 0.0);
        // Independent edge accumulation avoids dense row-sum cancellation in
        // RHS compatibility; its entire difference remains within roundoff.
        op.build_rhs_f64(refs(&fields(&g)), 0.01, &mut actual)
            .unwrap();
        assert!(actual.iter().all(|v| *v == 0.0));
        let sum = rhs.iter().sum::<f64>();
        assert!(sum.abs() < 1e-12);
        let mut ws = PressureWorkspace::new(&g, LIMIT).unwrap();
        ws.solve_rhs(&op, &rhs, 0.01, settings(), || false).unwrap();
        for (a, b) in ws.pressure().iter().zip(&exact) {
            assert!(
                (a - (b - exact[0])).abs() < 2e-8,
                "ratio {ratio}: {a} versus {}",
                b - exact[0]
            );
        }
        rhs.fill(0.0);
        rhs[0] = 1.0;
        assert!(matches!(
            ws.solve_rhs(&op, &rhs, 0.01, settings(), || false),
            Err(PressureError::IncompatibleRhs { .. })
        ));
    }
}

#[test]
fn constant_variant_preserves_legacy_operator_and_f32_correction_contract() {
    let g = grid();
    let density = 1.25;
    let rho = AXES.map(|a| vec![density; g.face_len(a)]);
    let old = PressureOperator::new(&g, density).unwrap();
    let new = PressureOperator::with_face_densities(&g, refs(&rho)).unwrap();
    let p: Vec<_> = (0..g.cell_len()).map(|i| i as f64 * 0.125).collect();
    let mut a = vec![0.0; p.len()];
    let mut b = a.clone();
    old.apply_full(&p, &mut a).unwrap();
    new.apply_full(&p, &mut b).unwrap();
    for (a, b) in a.iter().zip(b) {
        assert!((a - b).abs() < 1e-12);
    }
    let mut velocity32 = AXES.map(|axis| vec![0.0_f32; g.face_len(axis)]);
    let [x, y, z] = &mut velocity32;
    new.correct_candidate_in_place(&p, 0.01, [x, y, z]).unwrap();
    let mut velocity64 = fields(&g);
    let [x, y, z] = &mut velocity64;
    new.correct_candidate_f64(&p, 0.01, [x, y, z]).unwrap();
    for d in 0..3 {
        for (a, b) in velocity32[d].iter().zip(&velocity64[d]) {
            assert_eq!(*a, *b as f32);
        }
    }
}

#[test]
fn heterogeneous_f32_corrections_match_each_independent_face_gradient() {
    let g = grid();
    let n = g.counts();
    let rho = AXES.map(|axis| {
        (0..g.face_len(axis))
            .map(|i| 1.0 + (i % 9) as f64 * 17.0)
            .collect::<Vec<_>>()
    });
    let op = PressureOperator::with_face_densities(&g, refs(&rho)).unwrap();
    let p = (0..g.cell_len())
        .map(|i| ((i * 7) % 13) as f64 * 0.125)
        .collect::<Vec<_>>();
    let old = AXES.map(|a| vec![0.0_f32; g.face_len(a)]);
    let mut separate = old.clone();
    let [x, y, z] = &mut separate;
    op.correct_velocity(old.each_ref().map(|v| v.as_slice()), &p, 0.01, [x, y, z])
        .unwrap();
    let mut inplace = old;
    let [x, y, z] = &mut inplace;
    op.correct_candidate_in_place(&p, 0.01, [x, y, z]).unwrap();
    for d in 0..3 {
        let dims = g.face_counts(AXES[d]);
        for k in 0..dims[2] {
            for j in 0..dims[1] {
                for i in 0..dims[0] {
                    let q = [i, j, k];
                    let face = i + dims[0] * (j + dims[1] * k);
                    let expected = if q[d] == 0 || q[d] == n[d] {
                        0.0
                    } else {
                        let mut lo = q;
                        lo[d] -= 1;
                        let low = lo[0] + n[0] * (lo[1] + n[1] * lo[2]);
                        let high = q[0] + n[0] * (q[1] + n[1] * q[2]);
                        (-0.01 * (p[high] - p[low]) / (rho[d][face] * g.spacing()[d])) as f32
                    };
                    assert!(
                        (separate[d][face] - expected).abs() <= 2.0 * f32::EPSILON * expected.abs()
                    );
                    assert!(
                        (inplace[d][face] - expected).abs() <= 2.0 * f32::EPSILON * expected.abs()
                    );
                }
            }
        }
    }
}

#[test]
fn two_material_hydrostatic_projection_uses_half_column_mass_on_every_axis() {
    let g = grid();
    let n = g.counts();
    for d in 0..3 {
        for materials in [
            pair(1.0),
            pair(10.0),
            pair(1000.0),
            MaterialPair::water_air(),
        ] {
            let f: Vec<_> = (0..g.cell_len())
                .map(|i| {
                    let p = [i % n[0], (i / n[0]) % n[1], i / (n[0] * n[1])];
                    if p[d] < 2 { 1.0 } else { 0.0 }
                })
                .collect();
            let state = VoxelMaterialState::new(g.clone(), materials, &f, LIMIT).unwrap();
            let mut ws = MaterialProjectionWorkspace::new(&state, LIMIT).unwrap();
            assert!(state.pressure().is_none());
            assert_eq!(state.generation(), 0);
            assert_eq!(state.time(), 0.0);
            let mut predictor = fields(&g);
            let dims = g.face_counts(AXES[d]);
            let gravity = -9.81;
            let dt = 0.002;
            for k in 0..dims[2] {
                for j in 0..dims[1] {
                    for i in 0..dims[0] {
                        let p = [i, j, k];
                        if p[d] > 0 && p[d] < n[d] {
                            predictor[d][g.face_index(AXES[d], p).unwrap()] = dt * gravity;
                        }
                    }
                }
            }
            let projected = ws
                .project(&state, refs(&predictor), dt, settings(), || false)
                .unwrap();
            assert!(projected.report().actual_divergence_max < 1e-8);
            for axis in AXES {
                assert!(projected.velocity(axis).iter().all(|v| v.abs() < 3e-10));
            }
            for k in 0..n[2] {
                for j in 0..n[1] {
                    for i in 0..n[0] {
                        let p = [i, j, k];
                        if p[d] + 1 < n[d] {
                            let mut q = p;
                            q[d] += 1;
                            let a = g.cell_index(p).unwrap();
                            let b = g.cell_index(q).unwrap();
                            let ra = if p[d] < 2 {
                                materials.liquid().density()
                            } else {
                                materials.gas().density()
                            };
                            let rb = if q[d] < 2 {
                                materials.liquid().density()
                            } else {
                                materials.gas().density()
                            };
                            let expected = gravity * g.spacing()[d] * (ra + rb) * 0.5;
                            assert!(
                                (projected.pressure()[b] - projected.pressure()[a] - expected)
                                    .abs()
                                    < 2e-6
                            );
                        }
                    }
                }
            }
            // Projection scratch is never confused with an accepted timestep.
            assert!(state.pressure().is_none());
            assert_eq!(state.generation(), 0);
        }
    }
}

#[test]
fn zero_gravity_arbitrary_material_state_is_at_rest_and_gas_has_mass() {
    let g = grid();
    let f = fraction(&g);
    let state = VoxelMaterialState::new(g.clone(), MaterialPair::water_air(), &f, LIMIT).unwrap();
    assert_eq!(state.density(0), Some(state.materials().gas().density()));
    assert!(state.density(0).unwrap() * g.cell_volume() > 0.0);
    let mut ws = MaterialProjectionWorkspace::new(&state, LIMIT).unwrap();
    let zero = fields(&g);
    let projected = ws
        .project(&state, refs(&zero), 0.01, settings(), || false)
        .unwrap();
    assert_eq!(projected.report().pressure.iterations, 0);
    assert_eq!(projected.report().actual_divergence_max, 0.0);
    for a in AXES {
        assert!(projected.velocity(a).iter().all(|v| *v == 0.0));
    }
}

#[test]
fn component_validation_is_targeted_and_projection_failure_preserves_state() {
    let g = grid();
    let f = fraction(&g);
    let materials = MaterialPair::water_air();
    assert!(matches!(
        MaterialProperties::new(0.0, 1.0),
        Err(VoxelMaterialError::InvalidMaterial)
    ));
    let mut bad = f.clone();
    bad[7] = f64::NAN;
    assert!(matches!(
        VoxelMaterialState::new(g.clone(), materials, &bad, LIMIT),
        Err(VoxelMaterialError::InvalidFraction { cell: 7 })
    ));
    assert!(matches!(
        VoxelMaterialState::new(g.clone(), materials, &f, 0),
        Err(VoxelMaterialError::BufferLimit { .. })
    ));
    let state = VoxelMaterialState::new(g.clone(), materials, &f, LIMIT).unwrap();
    assert!(matches!(
        MaterialProjectionWorkspace::new(&state, state.allocated_bytes()),
        Err(VoxelMaterialError::BufferLimit { .. })
    ));
    let mut ws = MaterialProjectionWorkspace::new(&state, LIMIT).unwrap();
    let total = state.allocated_bytes() + ws.allocated_bytes();
    assert!(matches!(
        MaterialProjectionWorkspace::new(&state, total - 1),
        Err(VoxelMaterialError::BufferLimit { .. })
    ));
    let mut predictor = fields(&g);
    predictor[0][0] = 1.0;
    assert!(matches!(
        ws.project(&state, refs(&predictor), 0.01, settings(), || false),
        Err(VoxelMaterialError::Operator(
            OperatorError::NonZeroWallVelocity {
                axis: Axis::X,
                index: 0
            }
        ))
    ));
    predictor[0][0] = 0.0;
    predictor[0][g.face_index(Axis::X, [1, 1, 1]).unwrap()] = 1.0;
    let mut s = settings();
    s.max_iterations = 0;
    assert!(matches!(
        ws.project(&state, refs(&predictor), 0.01, s, || false),
        Err(VoxelMaterialError::Pressure(
            PressureError::IterationLimit { iterations: 0, .. }
        ))
    ));
    assert!(matches!(
        ws.project(&state, refs(&predictor), 0.01, settings(), || true),
        Err(VoxelMaterialError::Cancelled)
    ));
    assert_eq!(state.liquid_fraction(), f);
    assert_eq!(state.generation(), 0);
    assert!(state.pressure().is_none());
    for a in AXES {
        assert!(state.velocity(a).iter().all(|v| *v == 0.0));
    }
}

#[test]
fn face_coefficient_validation_refuses_bad_shapes_nonfinite_and_zero() {
    let g = grid();
    let mut rho = AXES.map(|a| vec![1.0; g.face_len(a)]);
    rho[0][1] = 0.0;
    assert!(matches!(
        PressureOperator::with_face_densities(&g, refs(&rho)),
        Err(OperatorError::InvalidDensity)
    ));
    rho[0][1] = f64::NAN;
    assert!(matches!(
        PressureOperator::with_face_densities(&g, refs(&rho)),
        Err(OperatorError::InvalidDensity)
    ));
    rho[0][1] = 1.0;
    rho[2].pop();
    assert!(matches!(
        PressureOperator::with_face_densities(&g, refs(&rho)),
        Err(OperatorError::LengthMismatch)
    ));
}
