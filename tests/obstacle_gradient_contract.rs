//! Synthetic declared local data, not physical solutions or authenticated inputs.
#[path = "../examples/viscous_boundary_wrench.rs"]
#[allow(dead_code)]
mod fixture;
use rheon::*;
const AXES: [Axis; 3] = [Axis::X, Axis::Y, Axis::Z];
fn reference(id: u64) -> ObstacleEvidenceRef {
    ObstacleEvidenceRef::new(id, 0, [id as u8; 32]).unwrap()
}
fn metadata() -> (ObstaclePhysicalInputs, ObstacleStateFrame) {
    (
        ObstaclePhysicalInputs {
            units: ObstacleStateUnits::Si,
            model: ObstacleStateModel::TransientStokes,
            material: ObstacleStateMaterial {
                density: 1.,
                dynamic_viscosity: 1.,
            },
            problem: reference(1),
            boundary_evidence: reference(2),
            boundary: ObstacleStateBoundary::StationaryNoSlipSolidSealedFreeSlipOuter,
            initial: ObstacleInitialData::Supplied(reference(3)),
            initial_time: 0.,
            forcing: ObstacleForcingHistory {
                kind: ObstacleForcingKind::ExplicitNoForcing,
                evidence: reference(4),
                start_time: 0.,
                end_time: 0.,
            },
        },
        ObstacleStateFrame {
            time: 0.,
            generation: 0,
            origin: ObstacleStateOrigin::InitialData,
            errors: ObstacleVelocityErrors::unknown(),
        },
    )
}
fn buffers(g: &StaticObstacleGeometry) -> [Vec<f64>; 3] {
    std::array::from_fn(|a| vec![0.; g.grid().face_len(AXES[a])])
}
fn samples(g: &StaticObstacleGeometry, f: impl Fn(usize, [f64; 3]) -> f64) -> [Vec<f64>; 3] {
    std::array::from_fn(|a| {
        (0..g.grid().face_len(AXES[a]))
            .map(|face| {
                let n = g.grid().face_counts(AXES[a]);
                let p = [face % n[0], (face / n[0]) % n[1], face / (n[0] * n[1])];
                if p[a] == 0 || p[a] == g.grid().counts()[a] || g.open_areas(AXES[a])[face] == 0. {
                    0.
                } else {
                    f(a, g.grid().face_position(AXES[a], p).unwrap())
                }
            })
            .collect()
    })
}
fn state<'g>(g: &'g StaticObstacleGeometry, v: &[Vec<f64>; 3]) -> ObstacleFlowState<'g> {
    let (i, f) = metadata();
    ObstacleFlowState::new(g, i, f, [&v[0], &v[1], &v[2]], MAX_OBSTACLE_STATE_BYTES).unwrap()
}
fn geometry() -> StaticObstacleGeometry {
    fixture::geometry([6; 3], [1.; 3], [0.; 3], [2; 3], [4; 3]).unwrap()
}
fn close(a: f64, b: f64) {
    assert!(
        (a - b).abs() <= 2e-12 * (1. + a.abs().max(b.abs())),
        "{a} != {b}"
    );
}
fn encode(s: &ObstacleFlowState<'_>) -> Vec<u8> {
    let mut v = Vec::new();
    s.write_checkpoint(&mut v).unwrap();
    v
}

#[test]
fn all_directed_interior_affine_components_and_rotation_are_consistent() {
    let g = geometry();
    let matrix = [[0.5, -1., 0.25], [0.75, 1.5, -0.5], [-0.25, 0.5, -1.]];
    let v = samples(&g, |a, x| {
        2. + (0..3).map(|b| matrix[a][b] * x[b]).sum::<f64>()
    });
    let s = state(&g, &v);
    let mut sites = Vec::new();
    let mut expected = Vec::new();
    for a in 0..3 {
        sites.push(ObstacleGradientSite::Normal {
            axis: AXES[a],
            cell: [1; 3],
        });
        expected.push(matrix[a][a]);
        for b in 0..3 {
            if a != b {
                for quadrant in 0..4 {
                    sites.push(ObstacleGradientSite::Cross {
                        component: AXES[a],
                        derivative: AXES[b],
                        edge: [1; 3],
                        quadrant,
                    });
                    expected.push(matrix[a][b]);
                }
            }
        }
    }
    let op =
        ObstacleVelocityGradient::new(&s, &sites, MAX_OBSTACLE_STATE_BYTES, |_, _| false).unwrap();
    let mut result = vec![0.; sites.len()];
    op.gather(&mut result, |_, _| false).unwrap();
    for (&got, &want) in result.iter().zip(&expected) {
        close(got, want);
    }
    let skew = [[0., -0.5, 0.25], [0.5, 0., -0.75], [-0.25, 0.75, 0.]];
    let v = samples(&g, |a, x| (0..3).map(|b| skew[a][b] * x[b]).sum());
    let s = state(&g, &v);
    let op =
        ObstacleVelocityGradient::new(&s, &sites, MAX_OBSTACLE_STATE_BYTES, |_, _| false).unwrap();
    op.gather(&mut result, |_, _| false).unwrap();
    for (row, &got) in op.rows().iter().zip(&result) {
        close(
            got,
            skew[fixture::axis(row.component)][fixture::axis(row.derivative)],
        );
    }
}

#[test]
fn all_reflected_flat_wall_pairs_use_actual_traces_and_keep_zero_rows() {
    let g = geometry();
    for a in 0..3 {
        for b in 0..3 {
            if a == b {
                continue;
            }
            for wall_index in [2, 4] {
                let mut edge = [2; 3];
                edge[a] = 3;
                edge[b] = wall_index;
                let wall = wall_index as f64;
                let v = samples(&g, |component, x| {
                    if component == a {
                        0.75 * (x[b] - wall)
                    } else {
                        0.
                    }
                });
                let s = state(&g, &v);
                let bit = if b < a { 1 } else { 2 };
                let q = if wall_index == 4 { bit } else { 0 };
                let sites = [
                    ObstacleGradientSite::Cross {
                        component: AXES[a],
                        derivative: AXES[b],
                        edge,
                        quadrant: q,
                    },
                    ObstacleGradientSite::Cross {
                        component: AXES[b],
                        derivative: AXES[a],
                        edge,
                        quadrant: q,
                    },
                ];
                let op =
                    ObstacleVelocityGradient::new(&s, &sites, MAX_OBSTACLE_STATE_BYTES, |_, _| {
                        false
                    })
                    .unwrap();
                let mut out = [0.; 2];
                op.gather(&mut out, |_, _| false).unwrap();
                close(out[0], 0.75);
                assert_eq!(out[1], 0.);
                assert_eq!(op.rows()[0].boundary, ObstacleGradientBoundary::FlatWallRay);
                assert_eq!(op.rows()[0].active_term_count(), 1);
                assert_eq!(
                    op.rows()[1].boundary,
                    ObstacleGradientBoundary::FlatStationaryTrace
                );
                assert_eq!(op.rows()[1].active_term_count(), 0);
                for endpoint in op.rows()[1].endpoints {
                    assert_eq!(endpoint.source, ObstacleGradientSource::StationarySolid);
                    assert_eq!(endpoint.position[b], wall);
                }
                assert!(
                    op.rows()[0].endpoints[0].position[b] < op.rows()[0].endpoints[1].position[b]
                );
            }
        }
    }
}

#[test]
fn represented_anisotropic_nonmidpoint_distances_are_not_snapped() {
    let spacing = [0.3, 0.7, 0.2];
    let origin = [0.1, -0.3, 1.1];
    let g = fixture::geometry([6; 3], spacing, origin, [2; 3], [4; 3]).unwrap();
    let mut asymmetries = 0;
    for a in 0..3 {
        for b in 0..3 {
            if a == b {
                continue;
            }
            for wall_index in [2, 4] {
                let wall = origin[b] + wall_index as f64 * spacing[b];
                let v = samples(
                    &g,
                    |component, x| if component == a { x[b] - wall } else { 0. },
                );
                let s = state(&g, &v);
                let mut edge = [2; 3];
                edge[a] = 3;
                edge[b] = wall_index;
                let bit = if b < a { 1 } else { 2 };
                let q = if wall_index == 4 { bit } else { 0 };
                let site = ObstacleGradientSite::Cross {
                    component: AXES[a],
                    derivative: AXES[b],
                    edge,
                    quadrant: q,
                };
                let op =
                    ObstacleVelocityGradient::new(&s, &[site], MAX_OBSTACLE_STATE_BYTES, |_, _| {
                        false
                    })
                    .unwrap();
                let row = &op.rows()[0];
                let delta = row.endpoints[1].position[b] - row.endpoints[0].position[b];
                assert_eq!(
                    row.endpoints[1].coefficient.to_bits(),
                    (1. / delta).to_bits()
                );
                if delta != 0.5 * spacing[b] {
                    asymmetries += 1;
                }
                let mut out = [0.];
                op.gather(&mut out, |_, _| false).unwrap();
                close(out[0], 1.);
                for e in row.endpoints {
                    if e.source == ObstacleGradientSource::StationarySolid {
                        assert_eq!(e.position[b].to_bits(), wall.to_bits());
                    }
                }
            }
        }
    }
    assert!(asymmetries > 0);
}

#[test]
fn weighted_transpose_matches_independent_full_face_scatter_and_work() {
    let g = geometry();
    let v = samples(&g, |a, x| {
        ((a + 1) as f64) * 0.25 + x[0] - 0.5 * x[1] + 0.125 * x[2]
    });
    let s = state(&g, &v);
    let before = encode(&s);
    let sites = [
        ObstacleGradientSite::Normal {
            axis: Axis::X,
            cell: [1; 3],
        },
        ObstacleGradientSite::Cross {
            component: Axis::X,
            derivative: Axis::Y,
            edge: [1; 3],
            quadrant: 0,
        },
        ObstacleGradientSite::Cross {
            component: Axis::Y,
            derivative: Axis::X,
            edge: [3, 2, 2],
            quadrant: 0,
        },
        ObstacleGradientSite::Cross {
            component: Axis::X,
            derivative: Axis::Y,
            edge: [3, 2, 2],
            quadrant: 1,
        },
    ];
    let op =
        ObstacleVelocityGradient::new(&s, &sites, MAX_OBSTACLE_STATE_BYTES, |_, _| false).unwrap();
    let q = [0.25, -1., 2., 0.75];
    let mut output = buffers(&g);
    let mut direct = buffers(&g);
    for (row, &test) in op.rows().iter().zip(&q) {
        for endpoint in row.endpoints {
            if let ObstacleGradientSource::VelocityFace { face } = endpoint.source {
                direct[fixture::axis(row.component)][face] +=
                    row.weight * test * endpoint.coefficient;
            }
        }
    }
    let [x, y, z] = &mut output;
    let report = op.diagnose(&q, [x, y, z], |_, _| false).unwrap();
    for a in 0..3 {
        for (&got, &expected) in output[a].iter().zip(&direct[a]) {
            close(got, expected);
        }
    }
    let rhs = (0..3)
        .map(|a| {
            s.velocity()[a]
                .iter()
                .zip(&output[a])
                .map(|(u, f)| u * f)
                .sum::<f64>()
        })
        .sum();
    close(report.face_pairing, rhs);
    close(report.row_pairing, rhs);
    close(report.unenclosed_defect, 0.);
    assert_eq!(encode(&s), before);
    assert!(std::ptr::eq(op.state(), &s));
    assert_eq!(op.state().frame().errors, ObstacleVelocityErrors::unknown());
    assert_eq!(op.qualification(), ObstacleStateQualification::Unqualified);
}

#[test]
fn explicit_unsupported_sites_refuse_without_state_changes() {
    let g = geometry();
    let v = buffers(&g);
    let s = state(&g, &v);
    let before = encode(&s);
    for (site, which) in [
        (ObstacleGradientSite::CoarseFine, 0),
        (
            ObstacleGradientSite::Cross {
                component: Axis::X,
                derivative: Axis::Y,
                edge: [2, 2, 2],
                quadrant: 0,
            },
            1,
        ),
        (
            ObstacleGradientSite::Cross {
                component: Axis::X,
                derivative: Axis::Y,
                edge: [0, 1, 0],
                quadrant: 0,
            },
            2,
        ),
        (
            ObstacleGradientSite::Normal {
                axis: Axis::X,
                cell: [2; 3],
            },
            3,
        ),
        (
            ObstacleGradientSite::Cross {
                component: Axis::X,
                derivative: Axis::X,
                edge: [1; 3],
                quadrant: 0,
            },
            3,
        ),
        (
            ObstacleGradientSite::Cross {
                component: Axis::X,
                derivative: Axis::Y,
                edge: [usize::MAX, 1, 1],
                quadrant: 0,
            },
            3,
        ),
    ] {
        let err =
            ObstacleVelocityGradient::new(&s, &[site], MAX_OBSTACLE_STATE_BYTES, |_, _| false)
                .err()
                .unwrap();
        assert!(match which {
            0 => matches!(err, ObstacleGradientError::UnsupportedCoarseFine),
            1 => matches!(err, ObstacleGradientError::UnsupportedCorner),
            2 => matches!(err, ObstacleGradientError::UnsupportedOuterEdge),
            _ => matches!(err, ObstacleGradientError::InvalidSite),
        });
    }
    assert_eq!(encode(&s), before);
}

#[test]
fn normal_stationary_outer_and_solid_traces_preserve_all_zero_rows() {
    let g = fixture::geometry([3; 3], [1.; 3], [0.; 3], [1; 3], [2; 3]).unwrap();
    let v = buffers(&g);
    let s = state(&g, &v);
    let mut sites = Vec::new();
    for k in 0..3 {
        for j in 0..3 {
            for i in 0..3 {
                let cell = [i, j, k];
                if g.fluid_volumes()[g.grid().cell_index(cell).unwrap()] > 0. {
                    for axis in AXES {
                        sites.push(ObstacleGradientSite::Normal { axis, cell });
                    }
                }
            }
        }
    }
    let op =
        ObstacleVelocityGradient::new(&s, &sites, MAX_OBSTACLE_STATE_BYTES, |_, _| false).unwrap();
    assert_eq!(op.rows().len(), 78);
    assert_eq!(
        op.rows()
            .iter()
            .filter(|r| r.active_term_count() == 0)
            .count(),
        6
    );
    assert!(
        op.rows()
            .iter()
            .flat_map(|r| r.endpoints)
            .any(|e| e.source == ObstacleGradientSource::StationaryOuterNormal)
    );
    assert!(
        op.rows()
            .iter()
            .flat_map(|r| r.endpoints)
            .any(|e| e.source == ObstacleGradientSource::StationarySolid)
    );
    let mut result = vec![7.; 78];
    op.gather(&mut result, |_, _| false).unwrap();
    assert!(result.iter().all(|&x| x == 0.));
}

#[test]
fn budgets_invalid_tests_cancellation_and_arithmetic_failure_are_explicit() {
    let g = geometry();
    let v = samples(&g, |a, _| if a == 0 { f64::MAX } else { 0. });
    let s = state(&g, &v);
    let sites = [ObstacleGradientSite::Normal {
        axis: Axis::X,
        cell: [1; 3],
    }];
    let needed = ObstacleVelocityGradient::planned_payload_bytes(&s, 1).unwrap();
    assert!(
        matches!(ObstacleVelocityGradient::new(&s,&sites,needed-1,|_,_|false),Err(ObstacleGradientError::Flow(ObstacleFlowError::BufferLimit{required,..})) if required==needed)
    );
    for cap in [0, MAX_OBSTACLE_STATE_BYTES + 1] {
        assert!(matches!(
            ObstacleVelocityGradient::new(&s, &sites, cap, |_, _| false),
            Err(ObstacleGradientError::InvalidBudget { .. })
        ));
    }
    assert!(matches!(
        ObstacleVelocityGradient::planned_payload_bytes(&s, usize::MAX),
        Err(ObstacleGradientError::Flow(
            ObstacleFlowError::CapacityOverflow
        ))
    ));
    assert!(matches!(
        ObstacleVelocityGradient::new(&s, &sites, needed, |_, _| true),
        Err(ObstacleGradientError::Flow(
            ObstacleFlowError::Cancelled { .. }
        ))
    ));
    let op = ObstacleVelocityGradient::new(&s, &sites, needed, |_, _| false).unwrap();
    assert_eq!(op.combined_payload_bytes(), needed);
    let mut output = buffers(&g);
    for bad in [f64::NAN, f64::INFINITY, f64::from_bits(1)] {
        let [x, y, z] = &mut output;
        assert!(matches!(
            op.transpose(&[bad], [x, y, z], |_, _| false),
            Err(ObstacleGradientError::Flow(
                ObstacleFlowError::NonFiniteInput
            ))
        ));
    }
    assert!(output.iter().flatten().all(|&x| x == 0.));
    let [x, y, z] = &mut output;
    assert!(matches!(
        op.diagnose(&[2.], [x, y, z], |_, _| false),
        Err(ObstacleGradientError::Flow(
            ObstacleFlowError::ArithmeticFailure
        ))
    ));
    let mut empty = [];
    assert!(matches!(
        op.gather(&mut empty, |_, _| false),
        Err(ObstacleGradientError::Flow(
            ObstacleFlowError::ShapeMismatch
        ))
    ));
    let mut scratch = [19.];
    assert!(matches!(
        op.gather(&mut scratch, |_, _| true),
        Err(ObstacleGradientError::Flow(
            ObstacleFlowError::Cancelled { .. }
        ))
    ));
    assert_eq!(scratch, [19.]);
}
