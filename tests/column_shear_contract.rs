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
fn no_slip_exact_stability_edge_and_one_ulp_incompatibility() {
    let (g, s) = fixture(Axis::Y, 3, 0.0);
    let mut u = profiles(&g, Axis::Y, &[0.0, 0.0, 1.0]);
    u[2].fill(0.0);
    let mut out = fields(&g);
    let mut workspace = ColumnShearWorkspace::new(g.clone(), Axis::Y, 1 << 20).unwrap();
    let walls = [0.0_f32, 1.0].map(|speed| ColumnShearBoundary::NoSlip {
        velocity: [speed, 0.0, 0.0],
    });
    let r = workspace
        .update_with_boundaries(
            ColumnShearInputs {
                dt: 0.5,
                ..input(s.state(), &u)
            },
            walls,
            muts(&mut out),
            |_| false,
        )
        .unwrap();
    assert_eq!(r.shear.stability_number, 1.0);
    let mut expected = profiles(&g, Axis::Y, &[0.0, 0.5, 1.0]);
    expected[2].fill(0.0);
    assert_eq!(out, expected);
    let before = out.clone();
    assert!(
        matches!(workspace.update_with_boundaries(ColumnShearInputs {
        dt: 0.5_f64.next_up(), ..input(s.state(), &u)
    }, walls, muts(&mut out), |_| false), Err(ColumnShearError::StabilityLimit { actual, limit: 1.0 }) if actual > 1.0)
    );
    assert_eq!(out, before);
    let incompatible = [
        walls[0],
        ColumnShearBoundary::NoSlip {
            velocity: [1.0_f32.next_up(), 0.0, 0.0],
        },
    ];
    assert_eq!(
        workspace.update_with_boundaries(
            input(s.state(), &u),
            incompatible,
            muts(&mut out),
            |_| false
        ),
        Err(ColumnShearError::IncompatibleNoSlip)
    );
    assert_eq!(out, before);
}

#[test]
fn no_slip_stored_subnormal_and_reaction_impulse_overflow_are_transactional() {
    let (g, s) = fixture(Axis::Y, 3, 0.0);
    let mut u = profiles(&g, Axis::Y, &[0.0, 0.0, f32::MIN_POSITIVE]);
    u[2].fill(0.0);
    let mut out = fields(&g);
    out[0].fill(7.0);
    let before = out.clone();
    let mut workspace = ColumnShearWorkspace::new(g, Axis::Y, 1 << 20).unwrap();
    let walls = [0.0_f32, f32::MIN_POSITIVE].map(|speed| ColumnShearBoundary::NoSlip {
        velocity: [speed, 0.0, 0.0],
    });
    // Stable exact-real interior speed is MIN_POSITIVE/8, which the declared
    // stored-f32 policy refuses instead of flushing or publishing a subnormal.
    assert_eq!(
        workspace.update_with_boundaries(input(s.state(), &u), walls, muts(&mut out), |_| false),
        Err(ColumnShearError::ArithmeticFailure)
    );
    assert_eq!(out, before);
    let (g, s) = fixture(Axis::Y, 2, 0.0);
    let mut u = profiles(&g, Axis::Y, &[0.0, 1.0]);
    u[2].fill(0.0);
    let mut out = fields(&g);
    out[0].fill(7.0);
    let before = out.clone();
    let mut workspace = ColumnShearWorkspace::new(g, Axis::Y, 1 << 20).unwrap();
    let walls = [0.0_f32, 1.0].map(|speed| ColumnShearBoundary::NoSlip {
        velocity: [speed, 0.0, 0.0],
    });
    // No free row imposes a diffusion step restriction, but an unrepresentable
    // dt * reaction force still refuses before publication.
    assert_eq!(
        workspace.update_with_boundaries(
            ColumnShearInputs {
                dt: f64::MAX / 2.0,
                ..input(s.state(), &u)
            },
            walls,
            muts(&mut out),
            |_| false
        ),
        Err(ColumnShearError::ArithmeticFailure)
    );
    assert_eq!(out, before);
}

#[test]
fn no_slip_invalid_scalars_and_output_shape_do_not_publish() {
    let (g, s) = fixture(Axis::Y, 3, 0.0);
    let mut u = profiles(&g, Axis::Y, &[0.0, 0.5, 1.0]);
    u[2].fill(0.0);
    let mut out = fields(&g);
    out[0].fill(7.0);
    let before = out.clone();
    let mut workspace = ColumnShearWorkspace::new(g, Axis::Y, 1 << 20).unwrap();
    let walls = [0.0_f32, 1.0].map(|speed| ColumnShearBoundary::NoSlip {
        velocity: [speed, 0.0, 0.0],
    });
    for (rho, mu, dt, error) in [
        (0.0, 1.0, 0.125, ColumnShearError::InvalidDensity),
        (f64::NAN, 1.0, 0.125, ColumnShearError::InvalidDensity),
        (1.0, -1.0, 0.125, ColumnShearError::InvalidCoefficient),
        (
            1.0,
            f64::INFINITY,
            0.125,
            ColumnShearError::InvalidCoefficient,
        ),
        (1.0, 1.0, 0.0, ColumnShearError::InvalidTimeStep),
        (1.0, 1.0, f64::INFINITY, ColumnShearError::InvalidTimeStep),
    ] {
        assert_eq!(
            workspace.update_with_boundaries(
                ColumnShearInputs {
                    density: rho,
                    dynamic_viscosity: mu,
                    dt,
                    ..input(s.state(), &u)
                },
                walls,
                muts(&mut out),
                |_| false
            ),
            Err(error)
        );
        assert_eq!(out, before);
    }
    let [x, y, z] = &mut out;
    assert_eq!(
        workspace
            .update_with_boundaries(input(s.state(), &u), walls, [&mut x[..1], y, z], |_| false),
        Err(ColumnShearError::GeometryMismatch)
    );
    assert_eq!(out, before);
}

#[test]
fn exact_no_slip_reactions_mass_work_and_momentum_all_axes() {
    for axis in [Axis::X, Axis::Y, Axis::Z] {
        let normal = [Axis::X, Axis::Y, Axis::Z]
            .iter()
            .position(|&a| a == axis)
            .unwrap();
        let (g, s) = fixture(axis, 3, 0.25);
        let u = profiles(&g, axis, &[1.0, 2.0, -1.0]);
        let mut out = fields(&g);
        let mut w = ColumnShearWorkspace::new(g.clone(), axis, 1 << 20).unwrap();
        let walls = [1.0_f32, -1.0].map(|speed| ColumnShearBoundary::NoSlip {
            velocity: std::array::from_fn(|d| if d == normal { 0.0 } else { speed }),
        });
        for (rho, mu) in [(1.0, 1.0), (4.0, 2.0)] {
            let r = w
                .update_with_boundaries(
                    ColumnShearInputs {
                        density: rho,
                        dynamic_viscosity: mu,
                        ..input(s.state(), &u)
                    },
                    walls,
                    muts(&mut out),
                    |_| false,
                )
                .unwrap();
            let a = r.shear.geometry.area;
            let delta = -0.5 * mu / rho;
            assert_eq!(out, profiles(&g, axis, &[1.0, (2.0 + delta) as f32, -1.0]));
            assert_eq!(w.mass_scratch()[..3], [rho * a, rho * a, 1.25 * rho * a]);
            assert_eq!(r.shear.stability_number, 0.25 * mu / rho);
            assert_eq!(r.bulk_dissipation_before, 20.0 * mu * a);
            assert_eq!(r.wall_dissipation_before, 0.0);
            assert_eq!(r.actuator_work, 0.5 * mu * a);
            assert_eq!(r.shear.workspace_bytes, w.allocated_bytes());
            assert_eq!(r.shear.update_energy, rho * a * delta * delta);
            for d in 0..3 {
                let tangent = f64::from(d != normal);
                assert_eq!(r.wall_force[0][d], -tangent * mu * a);
                assert_eq!(r.wall_force[1][d], -3.0 * tangent * mu * a);
                assert_eq!(r.reaction_impulse[0][d], -0.125 * tangent * mu * a);
                assert_eq!(r.reaction_impulse[1][d], -0.375 * tangent * mu * a);
                assert_eq!(r.wall_impulse[0][d], r.reaction_impulse[0][d]);
                assert_eq!(r.wall_impulse[1][d], r.reaction_impulse[1][d]);
                assert_eq!(r.shear.force_sum[d], -4.0 * tangent * mu * a);
                assert!(r.shear.momentum_error[d].abs() <= r.shear.momentum_budget[d]);
            }
            assert!(r.shear.identity_error.abs() <= r.shear.energy_budget);
        }
    }
}

#[test]
fn no_slip_common_translation_preserves_reaction_and_can_extract_net_work() {
    let (g, s) = fixture(Axis::Y, 3, 0.25);
    let mut workspace = ColumnShearWorkspace::new(g.clone(), Axis::Y, 1 << 20).unwrap();
    let mut out = fields(&g);
    let u = profiles(&g, Axis::Y, &[1.0, 2.0, -1.0]);
    let walls = [1.0_f32, -1.0].map(|speed| ColumnShearBoundary::NoSlip {
        velocity: [speed, 0.0, speed],
    });
    let original = workspace
        .update_with_boundaries(input(s.state(), &u), walls, muts(&mut out), |_| false)
        .unwrap();
    let shifted_u = profiles(&g, Axis::Y, &[3.0, 4.0, 1.0]);
    let shifted_walls = [3.0_f32, 1.0].map(|speed| ColumnShearBoundary::NoSlip {
        velocity: [speed, 0.0, speed],
    });
    let shifted = workspace
        .update_with_boundaries(
            input(s.state(), &shifted_u),
            shifted_walls,
            muts(&mut out),
            |_| false,
        )
        .unwrap();
    let a = original.shear.geometry.area;
    assert_eq!(out, profiles(&g, Axis::Y, &[3.0, 3.5, 1.0]));
    assert_eq!(original.wall_force, shifted.wall_force);
    assert_eq!(original.reaction_impulse, shifted.reaction_impulse);
    assert_eq!(
        original.bulk_dissipation_before,
        shifted.bulk_dissipation_before
    );
    assert_eq!(shifted.actuator_work, -1.5 * a);
    assert_eq!(
        shifted.shear.kinetic_after - shifted.shear.kinetic_before,
        -3.75 * a
    );
    assert_eq!(shifted.shear.identity_error, 0.0);
}

#[test]
fn fully_constrained_slab_has_reaction_work_without_an_explicit_free_row() {
    let (g, s) = fixture(Axis::Y, 2, 0.0);
    let mut u = profiles(&g, Axis::Y, &[0.0, 1.0]);
    u[2].fill(0.0);
    let mut out = fields(&g);
    let mut w = ColumnShearWorkspace::new(g, Axis::Y, 1 << 20).unwrap();
    let boundaries = [0.0_f32, 1.0].map(|speed| ColumnShearBoundary::NoSlip {
        velocity: [speed, 0.0, 0.0],
    });
    let r = w
        .update_with_boundaries(
            ColumnShearInputs {
                dt: 2.0,
                ..input(s.state(), &u)
            },
            boundaries,
            muts(&mut out),
            |_| false,
        )
        .unwrap();
    assert_eq!(out, u);
    let a = r.shear.geometry.area;
    assert_eq!(r.shear.stability_number, 0.0);
    assert_eq!(
        r.reaction_impulse,
        [[-2.0 * a, 0.0, 0.0], [2.0 * a, 0.0, 0.0]]
    );
    assert_eq!(r.actuator_work, 2.0 * a);
    assert_eq!(r.shear.kinetic_before, r.shear.kinetic_after);
    assert_eq!(r.shear.update_energy, 0.0);
    assert_eq!(r.shear.rounding_work, 0.0);
    assert_eq!(r.shear.identity_error, 0.0);
}

#[test]
fn one_node_mixed_laws_have_unique_opposing_impulses_and_signed_work() {
    let (g, s) = fixture(Axis::Y, 1, 0.0);
    let mut w = ColumnShearWorkspace::new(g.clone(), Axis::Y, 1 << 20).unwrap();
    let mut out = fields(&g);
    for (fluid, slipping_wall) in [(1.0_f32, 0.0_f64), (1.0, 2.0), (-1.0, 0.0)] {
        let mut u = profiles(&g, Axis::Y, &[fluid]);
        u[2].fill(0.0);
        let boundaries = [
            ColumnShearBoundary::NoSlip {
                velocity: [fluid, 0.0, 0.0],
            },
            ColumnShearBoundary::Navier(ColumnShearWall {
                velocity: [slipping_wall, 0.0, 0.0],
                friction: 2.0,
            }),
        ];
        let r = w
            .update_with_boundaries(input(s.state(), &u), boundaries, muts(&mut out), |_| false)
            .unwrap();
        let a = r.shear.geometry.area;
        let navier_impulse = -0.25 * a * (f64::from(fluid) - slipping_wall);
        assert_eq!(out, u);
        assert_eq!(r.reaction_impulse, [[-navier_impulse, 0.0, 0.0], [0.0; 3]]);
        assert_eq!(
            r.wall_impulse,
            [[-navier_impulse, 0.0, 0.0], [navier_impulse, 0.0, 0.0]]
        );
        assert_eq!(r.actuator_work, 0.25 * a);
        assert_eq!(r.shear.force_sum, [0.0; 3]);
        assert_eq!(r.shear.identity_error, 0.0);
    }
}

#[test]
fn no_slip_rejections_and_all_constraint_callbacks_preserve_output() {
    let (g, s) = fixture(Axis::Y, 3, 0.0);
    let mut u = profiles(&g, Axis::Y, &[0.0, 0.5, 1.0]);
    u[2].fill(0.0);
    let mut out = fields(&g);
    let before = out.clone();
    let mut w = ColumnShearWorkspace::new(g.clone(), Axis::Y, 1 << 20).unwrap();
    let walls = [0.0_f32, 1.0].map(|speed| ColumnShearBoundary::NoSlip {
        velocity: [speed, 0.0, 0.0],
    });
    for occurrence in 1..=4 {
        let mut count = 0;
        assert!(matches!(
            w.update_with_boundaries(input(s.state(), &u), walls, muts(&mut out), |stage| {
                if stage == ColumnShearStage::WallConstraint {
                    count += 1;
                }
                stage == ColumnShearStage::WallConstraint && count == occurrence
            }),
            Err(ColumnShearError::Cancelled {
                stage: ColumnShearStage::WallConstraint
            })
        ));
        assert_eq!(out, before);
    }
    assert!(matches!(
        w.update_with_boundaries(input(s.state(), &u), walls, muts(&mut out), |stage| stage
            == ColumnShearStage::BeforeAcceptance),
        Err(ColumnShearError::Cancelled { .. })
    ));
    assert_eq!(out, before);
    assert!(matches!(
        w.update_with_boundaries(
            ColumnShearInputs {
                dt: 2.0,
                ..input(s.state(), &u)
            },
            walls,
            muts(&mut out),
            |_| false
        ),
        Err(ColumnShearError::StabilityLimit { .. })
    ));
    assert_eq!(out, before);
    let incompatible = [
        ColumnShearBoundary::NoSlip {
            velocity: [0.25, 0.0, 0.0],
        },
        walls[1],
    ];
    assert_eq!(
        w.update_with_boundaries(input(s.state(), &u), incompatible, muts(&mut out), |_| {
            false
        }),
        Err(ColumnShearError::IncompatibleNoSlip)
    );
    assert_eq!(out, before);
    for invalid in [
        [f32::NAN, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [f32::from_bits(1), 0.0, 0.0],
    ] {
        assert!(
            w.update_with_boundaries(
                input(s.state(), &u),
                [ColumnShearBoundary::NoSlip { velocity: invalid }, walls[1]],
                muts(&mut out),
                |_| false
            )
            .is_err()
        );
        assert_eq!(out, before);
    }
    assert_eq!(
        w.update_with_boundaries(
            ColumnShearInputs {
                dynamic_viscosity: 0.0,
                ..input(s.state(), &u)
            },
            walls,
            muts(&mut out),
            |_| false
        ),
        Err(ColumnShearError::InvalidCoefficient)
    );
    assert_eq!(out, before);
    let (single_g, single_s) = fixture(Axis::Y, 1, 0.0);
    let single_u = fields(&single_g);
    let mut single_out = fields(&single_g);
    let mut single_w = ColumnShearWorkspace::new(single_g, Axis::Y, 1 << 20).unwrap();
    let stationary = ColumnShearBoundary::NoSlip { velocity: [0.0; 3] };
    assert_eq!(
        single_w.update_with_boundaries(
            input(single_s.state(), &single_u),
            [stationary; 2],
            muts(&mut single_out),
            |_| false
        ),
        Err(ColumnShearError::OverlappingNoSlip)
    );
    assert_eq!(single_out, single_u);
}

#[test]
fn boundary_api_retains_original_finite_navier_report_and_fields() {
    let (g, s) = fixture(Axis::Y, 3, 0.25);
    let u = profiles(&g, Axis::Y, &[0.0, 0.5, 1.0]);
    let mut a = fields(&g);
    let mut b = fields(&g);
    let mut w = ColumnShearWorkspace::new(g, Axis::Y, 1 << 20).unwrap();
    let walls = [ColumnShearWall {
        velocity: [0.0; 3],
        friction: 0.5,
    }; 2];
    let old = w
        .update_with_walls(input(s.state(), &u), walls, muts(&mut a), |_| false)
        .unwrap();
    let new = w
        .update_with_boundaries(
            input(s.state(), &u),
            walls.map(ColumnShearBoundary::Navier),
            muts(&mut b),
            |_| false,
        )
        .unwrap();
    assert_eq!(a, b);
    assert_eq!(old.shear, new.shear);
    assert_eq!(old.wall_force, new.wall_force);
    assert_eq!(old.actuator_work, new.actuator_work);
    assert_eq!(old.bulk_dissipation_before, new.bulk_dissipation_before);
    assert_eq!(old.wall_dissipation_before, new.wall_dissipation_before);
    assert_eq!(new.reaction_impulse, [[0.0; 3]; 2]);
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

#[test]
fn wall_friction_has_physical_mass_scaling_and_moving_wall_work() {
    for axis in [Axis::X, Axis::Y, Axis::Z] {
        let (g, s) = fixture(axis, 1, 0.0);
        let normal = match axis {
            Axis::X => 0,
            Axis::Y => 1,
            Axis::Z => 2,
        };
        let tangent = if normal == 0 { 1 } else { 0 };
        let mut walls = [ColumnShearWall {
            velocity: [0.0; 3],
            friction: 1.0,
        }; 2];
        let u = profiles(&g, axis, &[1.0]);
        let mut out = fields(&g);
        let mut w = ColumnShearWorkspace::new(g.clone(), axis, 1 << 20).unwrap();
        let r = w
            .update_with_walls(
                ColumnShearInputs {
                    dynamic_viscosity: 1.0,
                    dt: 0.125,
                    ..input(s.state(), &u)
                },
                walls,
                muts(&mut out),
                |_| false,
            )
            .unwrap();
        assert_eq!(
            out[tangent][g
                .face_index([Axis::X, Axis::Y, Axis::Z][tangent], [0; 3])
                .unwrap()],
            0.75
        );
        assert!(r.shear.kinetic_after < r.shear.kinetic_before);
        assert_eq!(r.actuator_work, 0.0);
        assert_eq!(r.wall_dissipation_before, 4.0 * r.shear.geometry.area);
        assert!(r.shear.identity_error.abs() <= r.shear.energy_budget);
        assert!(r.shear.momentum_error[tangent].abs() <= r.shear.momentum_budget[tangent]);
        let heavy = w
            .update_with_walls(
                ColumnShearInputs {
                    density: 2.0,
                    dynamic_viscosity: 1.0,
                    dt: 0.125,
                    ..input(s.state(), &u)
                },
                walls,
                muts(&mut out),
                |_| false,
            )
            .unwrap();
        assert_eq!(
            out[tangent][g
                .face_index([Axis::X, Axis::Y, Axis::Z][tangent], [0; 3])
                .unwrap()],
            0.875
        );
        assert_eq!(heavy.shear.density, 2.0);
        walls[0].velocity[tangent] = 2.0;
        walls[1].velocity[tangent] = 2.0;
        let rest = profiles(&g, axis, &[0.0]);
        let driven = w
            .update_with_walls(
                ColumnShearInputs {
                    dynamic_viscosity: 1.0,
                    dt: 0.125,
                    ..input(s.state(), &rest)
                },
                walls,
                muts(&mut out),
                |_| false,
            )
            .unwrap();
        assert_eq!(
            out[tangent][g
                .face_index([Axis::X, Axis::Y, Axis::Z][tangent], [0; 3])
                .unwrap()],
            0.5
        );
        assert!(driven.shear.kinetic_after > 0.0 && driven.actuator_work > 0.0);
        // Common tangential translation has no relative wall drag or strain.
        for wall in &mut walls {
            wall.velocity = [0.0; 3];
            for d in 0..3 {
                if d != normal {
                    wall.velocity[d] = 1.0;
                }
            }
        }
        let common = w
            .update_with_walls(input(s.state(), &u), walls, muts(&mut out), |_| false)
            .unwrap();
        assert_eq!(out, u);
        assert_eq!(common.wall_dissipation_before, 0.0);
        assert_eq!(common.actuator_work, 0.0);
    }
}

#[test]
fn zero_wall_coefficient_preserves_original_update_and_wall_cancellation_preserves_output() {
    let (g, s) = fixture(Axis::Y, 2, 0.25);
    let u = profiles(&g, Axis::Y, &[1.0, -1.0]);
    let mut a = fields(&g);
    let mut b = fields(&g);
    let mut w = ColumnShearWorkspace::new(g.clone(), Axis::Y, 1 << 20).unwrap();
    let walls = [ColumnShearWall {
        velocity: [0.0; 3],
        friction: 0.0,
    }; 2];
    let old = w
        .update(input(s.state(), &u), muts(&mut a), |_| false)
        .unwrap();
    let new = w
        .update_with_walls(input(s.state(), &u), walls, muts(&mut b), |_| false)
        .unwrap();
    assert_eq!(a, b);
    assert_eq!(old, new.shear);
    let walls = [ColumnShearWall {
        velocity: [0.0; 3],
        friction: 0.5,
    }; 2];
    for occurrence in 1..=4 {
        let before = b.clone();
        let mut n = 0;
        assert!(matches!(
            w.update_with_walls(input(s.state(), &u), walls, muts(&mut b), |stage| {
                if stage == ColumnShearStage::WallTraction {
                    n += 1;
                }
                stage == ColumnShearStage::WallTraction && n == occurrence
            }),
            Err(ColumnShearError::Cancelled {
                stage: ColumnShearStage::WallTraction
            })
        ));
        assert_eq!(b, before);
    }
    let before = b.clone();
    assert!(matches!(
        w.update_with_walls(
            ColumnShearInputs {
                dt: 2.0,
                ..input(s.state(), &u)
            },
            walls,
            muts(&mut b),
            |_| false
        ),
        Err(ColumnShearError::StabilityLimit { .. })
    ));
    assert_eq!(b, before);
}

#[test]
fn moving_wall_extracts_energy_when_fluid_outruns_it() {
    let (g, s) = fixture(Axis::Y, 1, 0.0);
    let mut u = profiles(&g, Axis::Y, &[2.0]);
    u[2].fill(0.0);
    let mut out = fields(&g);
    let mut w = ColumnShearWorkspace::new(g.clone(), Axis::Y, 1 << 20).unwrap();
    let walls = [ColumnShearWall {
        velocity: [1.0, 0.0, 0.0],
        friction: 1.0,
    }; 2];
    let r = w
        .update_with_walls(input(s.state(), &u), walls, muts(&mut out), |_| false)
        .unwrap();
    let area = r.shear.geometry.area;
    assert_eq!(out[0][g.face_index(Axis::X, [0; 3]).unwrap()], 1.75);
    assert_eq!(r.wall_force, [[-area, 0.0, 0.0]; 2]);
    assert_eq!(r.actuator_work, -0.25 * area);
    assert_eq!(r.wall_dissipation_before, 2.0 * area);
    assert_eq!(r.shear.kinetic_before, 2.0 * area);
    assert_eq!(r.shear.kinetic_after, 1.53125 * area);
    assert!(r.shear.identity_error.abs() <= r.shear.energy_budget);
    for d in 0..3 {
        assert!(r.shear.momentum_error[d].abs() <= r.shear.momentum_budget[d]);
    }
}

#[test]
fn signed_wall_motion_and_unsupported_wall_laws_are_explicit() {
    let (g, s) = fixture(Axis::Y, 1, 0.0);
    let u = profiles(&g, Axis::Y, &[0.0]);
    let mut out = fields(&g);
    let mut w = ColumnShearWorkspace::new(g.clone(), Axis::Y, 1 << 20).unwrap();
    for speed in [-2.0, 2.0] {
        let walls = [ColumnShearWall {
            velocity: [speed, 0.0, 0.0],
            friction: 1.0,
        }; 2];
        let r = w
            .update_with_walls(input(s.state(), &u), walls, muts(&mut out), |_| false)
            .unwrap();
        assert_eq!(
            out[0][g.face_index(Axis::X, [0; 3]).unwrap()],
            (speed * 0.25) as f32
        );
        assert!(r.actuator_work > 0.0);
        assert_eq!(r.wall_force[0][0].signum(), speed.signum());
    }
    let before = out.clone();
    for wall in [
        ColumnShearWall {
            velocity: [0.0; 3],
            friction: -1.0,
        },
        ColumnShearWall {
            velocity: [0.0; 3],
            friction: f64::NAN,
        },
        ColumnShearWall {
            velocity: [0.0, 1.0, 0.0],
            friction: 1.0,
        },
    ] {
        assert!(
            w.update_with_walls(input(s.state(), &u), [wall; 2], muts(&mut out), |_| false)
                .is_err()
        );
        assert_eq!(out, before);
    }
    assert!(matches!(
        w.update_with_walls(
            ColumnShearInputs {
                dynamic_viscosity: 0.0,
                ..input(s.state(), &u)
            },
            [ColumnShearWall {
                velocity: [0.0; 3],
                friction: 1.0
            }; 2],
            muts(&mut out),
            |_| false
        ),
        Err(ColumnShearError::InvalidCoefficient)
    ));
    assert_eq!(out, before);
}

fn stationary_no_slip() -> [ColumnShearBoundary; 2] {
    [ColumnShearBoundary::NoSlip { velocity: [0.0; 3] }; 2]
}
fn uniform_force(value: [f64; 3], units: ForceUnits) -> UniformColumnForce {
    UniformColumnForce { value, units }
}
#[test]
fn uniform_forcing_uses_actual_dual_volume_mass_and_both_tangents_all_axes() {
    for (normal, axis) in [Axis::X, Axis::Y, Axis::Z].into_iter().enumerate() {
        let (g, s) = fixture(axis, 3, 0.25);
        let u = profiles(&g, axis, &[1.0, 1.0, 1.0]);
        let mut out = fields(&g);
        let mut w = ColumnShearWorkspace::new(g.clone(), axis, 1 << 20).unwrap();
        let value = std::array::from_fn(|d| if d == normal { 0.0 } else { 2.0 });
        let free = [ColumnShearBoundary::Navier(ColumnShearWall {
            velocity: [0.0; 3],
            friction: 0.0,
        }); 2];
        let r = w
            .update_with_forcing(
                ColumnShearInputs {
                    density: 4.0,
                    ..input(s.state(), &u)
                },
                free,
                uniform_force(value, ForceUnits::Acceleration),
                muts(&mut out),
                |_| false,
            )
            .unwrap();
        assert_eq!(out, profiles(&g, axis, &[1.25, 1.25, 1.25]));
        let volume = r.boundary.shear.geometry.liquid_volume;
        assert_eq!(
            w.mass_scratch()[..3],
            [
                4.0 * r.boundary.shear.geometry.area,
                4.0 * r.boundary.shear.geometry.area,
                5.0 * r.boundary.shear.geometry.area
            ]
        );
        for (d, &v) in value.iter().enumerate() {
            assert_eq!(r.external_force[d], 4.0 * volume * v);
            assert_eq!(r.external_impulse[d], 0.5 * volume * v);
            assert!(
                r.boundary.shear.momentum_error[d].abs() <= r.boundary.shear.momentum_budget[d]
            );
        }
        assert_eq!(r.external_work_before, 2.0 * volume);
        assert_eq!(r.boundary.shear.update_energy, 0.25 * volume);
        let acceleration_out = out.clone();
        let q = value.map(|v| 4.0 * v);
        let density_report = w
            .update_with_forcing(
                ColumnShearInputs {
                    density: 4.0,
                    ..input(s.state(), &u)
                },
                free,
                uniform_force(q, ForceUnits::ForceDensity),
                muts(&mut out),
                |_| false,
            )
            .unwrap();
        assert_eq!(out, acceleration_out);
        assert_eq!(r.boundary, density_report.boundary);
        assert_eq!(r.external_force, density_report.external_force);
        // Negative old-speed work is retained and can decelerate a uniform field.
        let negative = w
            .update_with_forcing(
                ColumnShearInputs {
                    density: 4.0,
                    ..input(s.state(), &u)
                },
                free,
                uniform_force(value.map(|v| -v), ForceUnits::Acceleration),
                muts(&mut out),
                |_| false,
            )
            .unwrap();
        assert_eq!(negative.external_work_before, -2.0 * volume);
        assert_eq!(out, profiles(&g, axis, &[0.75, 0.75, 0.75]));
        assert_eq!(negative.boundary.shear.identity_error, 0.0);
    }
}
#[test]
fn poiseuille_discrete_equilibrium_and_stationary_wall_reactions_are_exact() {
    let (g, s) = fixture(Axis::Y, 5, 0.0);
    let mut u = profiles(&g, Axis::Y, &[0.0, 3.0, 4.0, 3.0, 0.0]);
    u[2].fill(0.0);
    let mut out = fields(&g);
    let mut w = ColumnShearWorkspace::new(g, Axis::Y, 1 << 20).unwrap();
    for rho in [1.0, 4.0] {
        let r = w
            .update_with_forcing(
                ColumnShearInputs {
                    density: rho,
                    ..input(s.state(), &u)
                },
                stationary_no_slip(),
                uniform_force([2.0, 0.0, 0.0], ForceUnits::ForceDensity),
                muts(&mut out),
                |_| false,
            )
            .unwrap();
        assert_eq!(out, u);
        let a = r.boundary.shear.geometry.area;
        assert_eq!(r.external_force, [10.0 * a, 0.0, 0.0]);
        assert_eq!(r.boundary.wall_force, [[-5.0 * a, 0.0, 0.0]; 2]);
        assert_eq!(r.boundary.shear.force_sum, [0.0; 3]);
        assert_eq!(r.external_work_before, 2.5 * a);
        assert_eq!(r.boundary.bulk_dissipation_before, 20.0 * a);
        assert_eq!(r.boundary.actuator_work, 0.0);
        assert_eq!(r.boundary.shear.update_energy, 0.0);
        assert_eq!(r.boundary.shear.identity_error, 0.0);
    }
}
#[test]
fn forcing_from_rest_has_explicit_increment_energy_and_density_dependent_transient() {
    let (g, s) = fixture(Axis::Y, 5, 0.0);
    let u = fields(&g);
    let mut out = fields(&g);
    let mut w = ColumnShearWorkspace::new(g.clone(), Axis::Y, 1 << 20).unwrap();
    for (rho, units, magnitude, expected) in [
        (1.0, ForceUnits::ForceDensity, 2.0, 0.25),
        (4.0, ForceUnits::ForceDensity, 2.0, 0.0625),
        (4.0, ForceUnits::Acceleration, 2.0, 0.25),
    ] {
        let r = w
            .update_with_forcing(
                ColumnShearInputs {
                    density: rho,
                    ..input(s.state(), &u)
                },
                stationary_no_slip(),
                uniform_force([magnitude, 0.0, 0.0], units),
                muts(&mut out),
                |_| false,
            )
            .unwrap();
        let mut expected_fields = profiles(&g, Axis::Y, &[0.0, expected, expected, expected, 0.0]);
        expected_fields[2].fill(0.0);
        assert_eq!(out, expected_fields);
        assert_eq!(r.external_work_before, 0.0);
        assert!(r.boundary.shear.kinetic_after > 0.0);
        assert_eq!(
            r.boundary.shear.kinetic_after,
            r.boundary.shear.update_energy
        );
        assert_eq!(r.boundary.shear.identity_error, 0.0);
    }
}
#[test]
fn fully_constrained_body_force_is_reported_and_exactly_reacted() {
    let (g, s) = fixture(Axis::Y, 2, 0.0);
    let u = fields(&g);
    let mut out = fields(&g);
    let mut w = ColumnShearWorkspace::new(g, Axis::Y, 1 << 20).unwrap();
    let r = w
        .update_with_forcing(
            ColumnShearInputs {
                dt: 8.0,
                ..input(s.state(), &u)
            },
            stationary_no_slip(),
            uniform_force([2.0, 0.0, 0.0], ForceUnits::ForceDensity),
            muts(&mut out),
            |_| false,
        )
        .unwrap();
    let area = r.boundary.shear.geometry.area;
    assert_eq!(out, u);
    assert_eq!(r.external_force, [4.0 * area, 0.0, 0.0]);
    assert_eq!(r.external_impulse, [32.0 * area, 0.0, 0.0]);
    assert_eq!(r.boundary.reaction_impulse, [[-16.0 * area, 0.0, 0.0]; 2]);
    assert_eq!(r.boundary.shear.force_sum, [0.0; 3]);
    assert_eq!(r.boundary.shear.stability_number, 0.0);
    assert_eq!(r.boundary.shear.identity_error, 0.0);
    assert_eq!(r.boundary.shear.momentum_error, [0.0; 3]);
}
#[test]
fn body_force_reaction_includes_shared_navier_and_moving_wall_work() {
    let (g, s) = fixture(Axis::Y, 1, 0.25);
    let mut u = profiles(&g, Axis::Y, &[1.0]);
    u[2].fill(0.0);
    let mut out = fields(&g);
    let mut w = ColumnShearWorkspace::new(g, Axis::Y, 1 << 20).unwrap();
    let b = [
        ColumnShearBoundary::NoSlip {
            velocity: [1.0, 0.0, 0.0],
        },
        ColumnShearBoundary::Navier(ColumnShearWall {
            velocity: [0.0; 3],
            friction: 2.0,
        }),
    ];
    let r = w
        .update_with_forcing(
            input(s.state(), &u),
            b,
            uniform_force([4.0, 0.0, 0.0], ForceUnits::ForceDensity),
            muts(&mut out),
            |_| false,
        )
        .unwrap();
    assert_eq!(out, u);
    let a = r.boundary.shear.geometry.area;
    assert_eq!(r.external_force[0], 5.0 * a);
    assert_eq!(r.boundary.wall_force[0][0], -3.0 * a);
    assert_eq!(r.boundary.wall_force[1][0], -2.0 * a);
    assert_eq!(r.boundary.actuator_work, -0.375 * a);
    assert_eq!(r.external_work_before, 0.625 * a);
    assert_eq!(r.boundary.shear.identity_error, 0.0);
}
#[test]
fn zero_forcing_retains_original_navier_and_no_slip_reports_and_fields() {
    let (g, s) = fixture(Axis::Y, 3, 0.25);
    let mut u = profiles(&g, Axis::Y, &[0.0, 0.5, 1.0]);
    u[2].fill(0.0);
    let mut out = fields(&g);
    let mut w = ColumnShearWorkspace::new(g, Axis::Y, 1 << 20).unwrap();
    let b = [
        ColumnShearBoundary::Navier(ColumnShearWall {
            velocity: [0.0; 3],
            friction: 0.5,
        }),
        ColumnShearBoundary::NoSlip {
            velocity: [1.0, 0.0, 0.0],
        },
    ];
    let old = w
        .update_with_boundaries(input(s.state(), &u), b, muts(&mut out), |_| false)
        .unwrap();
    let old_out = out.clone();
    for units in [ForceUnits::Acceleration, ForceUnits::ForceDensity] {
        let new = w
            .update_with_forcing(
                input(s.state(), &u),
                b,
                uniform_force([0.0; 3], units),
                muts(&mut out),
                |_| false,
            )
            .unwrap();
        assert_eq!(new.boundary, old);
        assert_eq!(out, old_out);
        assert_eq!(new.external_impulse, [0.0; 3]);
        assert_eq!(new.external_work_before, 0.0);
    }
}
#[test]
fn forcing_rejection_stability_and_every_force_callback_preserve_output() {
    let (g, s) = fixture(Axis::Y, 3, 0.0);
    let u = fields(&g);
    let mut out = fields(&g);
    out[0].fill(7.0);
    let before = out.clone();
    let mut w = ColumnShearWorkspace::new(g, Axis::Y, 1 << 20).unwrap();
    for value in [
        [f64::NAN, 0.0, 0.0],
        [f64::INFINITY, 0.0, 0.0],
        [f64::from_bits(1), 0.0, 0.0],
        [0.0, 1.0, 0.0],
    ] {
        assert_eq!(
            w.update_with_forcing(
                input(s.state(), &u),
                stationary_no_slip(),
                uniform_force(value, ForceUnits::Acceleration),
                muts(&mut out),
                |_| false
            ),
            Err(ColumnShearError::UnsupportedForce)
        );
        assert_eq!(out, before);
    }
    let force = uniform_force([1.0, 0.0, 0.0], ForceUnits::Acceleration);
    for occurrence in 1..=6 {
        let mut count = 0;
        assert!(matches!(
            w.update_with_forcing(
                input(s.state(), &u),
                stationary_no_slip(),
                force,
                muts(&mut out),
                |stage| {
                    if stage == ColumnShearStage::BodyForceNode {
                        count += 1;
                    }
                    stage == ColumnShearStage::BodyForceNode && count == occurrence
                }
            ),
            Err(ColumnShearError::Cancelled {
                stage: ColumnShearStage::BodyForceNode
            })
        ));
        assert_eq!(out, before);
    }
    for stage in [
        ColumnShearStage::WallConstraint,
        ColumnShearStage::BeforeAcceptance,
    ] {
        assert!(matches!(
            w.update_with_forcing(
                input(s.state(), &u),
                stationary_no_slip(),
                force,
                muts(&mut out),
                |s| s == stage
            ),
            Err(ColumnShearError::Cancelled { .. })
        ));
        assert_eq!(out, before);
    }
    assert!(matches!(
        w.update_with_forcing(
            ColumnShearInputs {
                dt: 0.5_f64.next_up(),
                ..input(s.state(), &u)
            },
            stationary_no_slip(),
            force,
            muts(&mut out),
            |_| false
        ),
        Err(ColumnShearError::StabilityLimit { .. })
    ));
    assert_eq!(out, before);
    // Stable explicit first step would store a subnormal; do not flush/publish.
    assert_eq!(
        w.update_with_forcing(
            input(s.state(), &u),
            stationary_no_slip(),
            uniform_force(
                [f64::from(f32::MIN_POSITIVE), 0.0, 0.0],
                ForceUnits::Acceleration
            ),
            muts(&mut out),
            |_| false
        ),
        Err(ColumnShearError::ArithmeticFailure)
    );
    assert_eq!(out, before);
    // Normal input whose node force product overflows, not a silently ignored cost.
    assert_eq!(
        w.update_with_forcing(
            ColumnShearInputs {
                density: 4.0,
                ..input(s.state(), &u)
            },
            stationary_no_slip(),
            uniform_force([f64::MAX, 0.0, 0.0], ForceUnits::Acceleration),
            muts(&mut out),
            |_| false
        ),
        Err(ColumnShearError::ArithmeticFailure)
    );
    assert_eq!(out, before);
}
