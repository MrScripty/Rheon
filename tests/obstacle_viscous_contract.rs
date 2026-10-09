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

fn interior_sites() -> Vec<ObstacleGradientSite> {
    let mut sites = Vec::new();
    for a in 0..3 {
        sites.push(ObstacleGradientSite::Normal {
            axis: AXES[a],
            cell: [1; 3],
        });
        for b in 0..3 {
            if a != b {
                for quadrant in 0..4 {
                    sites.push(ObstacleGradientSite::Cross {
                        component: AXES[a],
                        derivative: AXES[b],
                        edge: [1; 3],
                        quadrant,
                    });
                }
            }
        }
    }
    sites
}
#[test]
fn affine_symmetric_stress_has_hand_derived_power_and_actual_force_work() {
    let g = geometry();
    let matrix = [[0.5, -1., 0.25], [0.75, 1.5, -0.5], [-0.25, 0.5, -1.]];
    let v = samples(&g, |a, x| {
        2. + (0..3).map(|b| matrix[a][b] * x[b]).sum::<f64>()
    });
    let (mut inputs, frame) = metadata();
    inputs.material.dynamic_viscosity = 2.5;
    let s = ObstacleFlowState::new(
        &g,
        inputs,
        frame,
        [&v[0], &v[1], &v[2]],
        MAX_OBSTACLE_STATE_BYTES,
    )
    .unwrap();
    let before = encode(&s);
    let sites = interior_sites();
    let grad =
        ObstacleVelocityGradient::new(&s, &sites, MAX_OBSTACLE_STATE_BYTES, |_, _| false).unwrap();
    let op = ObstacleViscousStress::new(&grad, MAX_OBSTACLE_STATE_BYTES, |_, _| false).unwrap();
    assert_eq!(op.blocks().len(), 15);
    assert_eq!(op.dynamic_viscosity(), 2.5);
    let mut stress = vec![0.; sites.len()];
    let mut force = buffers(&g);
    let [x, y, z] = &mut force;
    let work = op.diagnose(&mut stress, [x, y, z], |_, _| false).unwrap();
    for (row, &tau) in grad.rows().iter().zip(&stress) {
        let a = fixture::axis(row.component);
        let b = fixture::axis(row.derivative);
        close(tau, 2.5 * (matrix[a][b] + matrix[b][a]));
    }
    assert_eq!(work.dissipation, 2.5 * 113. / 16.);
    assert_eq!(work.rayleigh_potential, 2.5 * 113. / 32.);
    close(work.force_work, -work.dissipation);
    assert_eq!(encode(&s), before);
    assert_eq!(op.qualification(), ObstacleStateQualification::Unqualified);
    assert!(force.iter().flatten().any(|&f| f != 0.));
    let mut standalone = vec![0.; stress.len()];
    op.stress(&mut standalone, |_, _| false).unwrap();
    assert_eq!(standalone, stress);
    let [x, y, z] = &mut force;
    op.force(&mut standalone, [x, y, z], |_, _| false).unwrap();
    let independent_work: f64 = s
        .velocity()
        .iter()
        .zip(&force)
        .flat_map(|(u, f)| u.iter().zip(f).map(|(&u, &f)| u * f))
        .sum();
    close(independent_work, work.force_work);
}
#[test]
fn compatible_local_rotation_does_not_dissipate_nonzero_cross_derivatives() {
    let g = geometry();
    let m = [[0., -0.5, 0.25], [0.5, 0., -0.75], [-0.25, 0.75, 0.]];
    let v = samples(&g, |a, x| (0..3).map(|b| m[a][b] * x[b]).sum());
    let s = state(&g, &v);
    let sites = interior_sites();
    let grad =
        ObstacleVelocityGradient::new(&s, &sites, MAX_OBSTACLE_STATE_BYTES, |_, _| false).unwrap();
    let op = ObstacleViscousStress::new(&grad, MAX_OBSTACLE_STATE_BYTES, |_, _| false).unwrap();
    let mut scratch = vec![0.; sites.len()];
    grad.gather(&mut scratch, |_, _| false).unwrap();
    assert!(scratch.iter().any(|&x| x != 0.));
    let mut f = buffers(&g);
    let [x, y, z] = &mut f;
    let w = op.diagnose(&mut scratch, [x, y, z], |_, _| false).unwrap();
    assert_eq!(w.dissipation, 0.);
    assert_eq!(w.force_work, 0.);
    assert!(scratch.iter().chain(f.iter().flatten()).all(|&x| x == 0.));
}
#[test]
fn reflected_anisotropic_and_nonmidpoint_flat_pairs_keep_nonzero_trace_stress() {
    for (spacing, origin) in [
        ([0.5, 1., 2.], [0.; 3]),
        ([0.3, 0.7, 0.2], [0.1, -0.3, 1.1]),
    ] {
        let g = fixture::geometry([6; 3], spacing, origin, [2; 3], [4; 3]).unwrap();
        for a in 0..3 {
            for b in 0..3 {
                if a == b {
                    continue;
                }
                for wall_index in [2, 4] {
                    let mut edge = [2; 3];
                    edge[a] = 3;
                    edge[b] = wall_index;
                    let wall = origin[b] + wall_index as f64 * spacing[b];
                    let v = samples(&g, |d, x| if d == a { 0.75 * (x[b] - wall) } else { 0. });
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
                    let grad = ObstacleVelocityGradient::new(
                        &s,
                        &sites,
                        MAX_OBSTACLE_STATE_BYTES,
                        |_, _| false,
                    )
                    .unwrap();
                    let op =
                        ObstacleViscousStress::new(&grad, MAX_OBSTACLE_STATE_BYTES, |_, _| false)
                            .unwrap();
                    let mut tau = [0.; 2];
                    let mut f = buffers(&g);
                    let [x, y, z] = &mut f;
                    let w = op.diagnose(&mut tau, [x, y, z], |_, _| false).unwrap();
                    close(tau[0], 0.75);
                    assert_eq!(tau[0], tau[1]);
                    assert_eq!(grad.rows()[1].active_term_count(), 0);
                    close(w.dissipation, grad.rows()[0].weight * 0.75f64.powi(2));
                    close(w.force_work, -w.dissipation);
                    for e in grad.rows()[0].endpoints {
                        if let ObstacleGradientSource::VelocityFace { face } = e.source {
                            close(f[a][face], -grad.rows()[0].weight * e.coefficient * 0.75);
                        }
                    }
                }
            }
        }
    }
}
#[test]
fn incomplete_duplicate_and_unsupported_layouts_are_refused() {
    let g = geometry();
    let v = buffers(&g);
    let s = state(&g, &v);
    let cross = ObstacleGradientSite::Cross {
        component: Axis::X,
        derivative: Axis::Y,
        edge: [1; 3],
        quadrant: 0,
    };
    for (sites, expected) in [
        (vec![cross], ObstacleViscousError::MissingShearPartner),
        (vec![cross, cross], ObstacleViscousError::DuplicateSite),
        (
            vec![
                ObstacleGradientSite::Normal {
                    axis: Axis::X,
                    cell: [1; 3]
                };
                2
            ],
            ObstacleViscousError::DuplicateSite,
        ),
    ] {
        let grad =
            ObstacleVelocityGradient::new(&s, &sites, MAX_OBSTACLE_STATE_BYTES, |_, _| false)
                .unwrap();
        assert!(
            matches!(ObstacleViscousStress::new(&grad,MAX_OBSTACLE_STATE_BYTES,|_,_|false),Err(e) if e==expected)
        );
    }
    for query in [
        ObstacleGradientSite::CoarseFine,
        ObstacleGradientSite::Cross {
            component: Axis::X,
            derivative: Axis::Y,
            edge: [2; 3],
            quadrant: 0,
        },
    ] {
        assert!(
            ObstacleVelocityGradient::new(&s, &[query], MAX_OBSTACLE_STATE_BYTES, |_, _| false)
                .is_err()
        );
    }
}
#[test]
fn reordered_pairs_are_equivalent_and_empty_action_is_zero() {
    let g = geometry();
    let v = samples(&g, |a, x| x[(a + 1) % 3]);
    let s = state(&g, &v);
    let sites = interior_sites();
    let mut reversed = sites.clone();
    reversed.reverse();
    let mut forces = Vec::new();
    for query in [&sites, &reversed] {
        let grad = ObstacleVelocityGradient::new(&s, query, MAX_OBSTACLE_STATE_BYTES, |_, _| false)
            .unwrap();
        let op = ObstacleViscousStress::new(&grad, MAX_OBSTACLE_STATE_BYTES, |_, _| false).unwrap();
        let mut tau = vec![0.; query.len()];
        let mut f = buffers(&g);
        let [x, y, z] = &mut f;
        let w = op.diagnose(&mut tau, [x, y, z], |_, _| false).unwrap();
        forces.push((f, w));
    }
    assert_eq!(forces[0], forces[1]);
    let grad =
        ObstacleVelocityGradient::new(&s, &[], MAX_OBSTACLE_STATE_BYTES, |_, _| false).unwrap();
    let op = ObstacleViscousStress::new(&grad, MAX_OBSTACLE_STATE_BYTES, |_, _| false).unwrap();
    let mut f = buffers(&g);
    for a in &mut f {
        a.fill(19.)
    }
    let [x, y, z] = &mut f;
    let w = op.diagnose(&mut [], [x, y, z], |_, _| false).unwrap();
    assert_eq!(w.dissipation, 0.);
    assert!(f.iter().flatten().all(|&x| x == 0.));
}
#[test]
fn constructor_peak_cap_cancellation_and_output_preflight_preserve_owners() {
    let g = geometry();
    let v = samples(&g, |a, x| x[a]);
    let s = state(&g, &v);
    let before = encode(&s);
    let sites = interior_sites();
    let grad =
        ObstacleVelocityGradient::new(&s, &sites, MAX_OBSTACLE_STATE_BYTES, |_, _| false).unwrap();
    let op = ObstacleViscousStress::new(&grad, MAX_OBSTACLE_STATE_BYTES, |_, _| false).unwrap();
    let peak = op.constructor_peak_payload_bytes();
    assert!(peak >= op.combined_payload_bytes());
    assert!(ObstacleViscousStress::new(&grad, peak, |_, _| false).is_ok());
    assert!(matches!(
        ObstacleViscousStress::new(&grad, peak - 1, |_, _| false),
        Err(ObstacleViscousError::Flow(
            ObstacleFlowError::BufferLimit { .. }
        ))
    ));
    assert!(matches!(
        ObstacleViscousStress::new(&grad, MAX_OBSTACLE_STATE_BYTES, |_, _| true),
        Err(ObstacleViscousError::Flow(
            ObstacleFlowError::Cancelled { .. }
        ))
    ));
    for limit in [0, MAX_OBSTACLE_STATE_BYTES + 1] {
        assert!(matches!(
            ObstacleViscousStress::new(&grad, limit, |_, _| false),
            Err(ObstacleViscousError::InvalidBudget { .. })
        ));
    }
    let mut tau = vec![17.; sites.len()];
    let mut f = buffers(&g);
    for b in &mut f {
        b.fill(23.)
    }
    let mut wrong = [];
    let [_, y, z] = &mut f;
    assert!(matches!(
        op.force(&mut tau, [&mut wrong, y, z], |_, _| false),
        Err(ObstacleViscousError::Flow(ObstacleFlowError::ShapeMismatch))
    ));
    assert!(tau.iter().all(|&x| x == 17.));
    assert!(f.iter().flatten().all(|&x| x == 23.));
    assert!(matches!(
        op.stress(&mut tau, |_, _| true),
        Err(ObstacleViscousError::Gradient(ObstacleGradientError::Flow(
            ObstacleFlowError::Cancelled { .. }
        )))
    ));
    assert_eq!(encode(&s), before);
}
#[test]
fn representable_stress_can_refuse_unrepresentable_power_without_mutating_state() {
    let g = geometry();
    let v = samples(&g, |a, x| if a == 0 { 1e200 * x[0] } else { 0. });
    let s = state(&g, &v);
    let before = encode(&s);
    let grad = ObstacleVelocityGradient::new(
        &s,
        &[ObstacleGradientSite::Normal {
            axis: Axis::X,
            cell: [1; 3],
        }],
        MAX_OBSTACLE_STATE_BYTES,
        |_, _| false,
    )
    .unwrap();
    let op = ObstacleViscousStress::new(&grad, MAX_OBSTACLE_STATE_BYTES, |_, _| false).unwrap();
    let mut tau = [0.];
    op.stress(&mut tau, |_, _| false).unwrap();
    assert!(tau[0].is_normal());
    let mut f = buffers(&g);
    let [x, y, z] = &mut f;
    assert!(matches!(
        op.diagnose(&mut tau, [x, y, z], |_, _| false),
        Err(ObstacleViscousError::Flow(
            ObstacleFlowError::ArithmeticFailure
        ))
    ));
    assert_eq!(encode(&s), before);
}
