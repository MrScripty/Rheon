//! Native finite contracts; continuum/refinement and experimental acceptance are separate.
#[allow(dead_code)]
#[path = "../experiments/submarine_stokes.rs"]
mod stokes;

use rheon::{GridGeometry, StaticObstacleGeometry, SurfaceSettings, SurfaceStamp, TriangleSurface};
use stokes::{AnalyticField, Config, Cuboid, Error, Request, Solver, Stage, WallPatch};

fn no_body() -> Config {
    Config {
        counts: [2; 3],
        origin: [0.0; 3],
        spacing: [0.5; 3],
        body: None,
        density: 1.0,
        viscosity: 1.0,
        u_ref: 0.01,
        l_ref: 1.0,
        dof_limit: 4096,
        payload_limit: 256 * 1024 * 1024,
    }
}
fn tiny() -> Config {
    Config {
        counts: [8, 5, 5],
        origin: [-1.0, -0.625, -0.625],
        spacing: [0.25; 3],
        body: Some(Cuboid {
            lower: [-0.5, -0.125, -0.125],
            upper: [0.5, 0.125, 0.125],
        }),
        ..no_body()
    }
}
fn owner(config: Config) -> StaticObstacleGeometry {
    let body = config.body.unwrap();
    let surface = TriangleSurface::new(
        SurfaceStamp { id: 1, version: 1 },
        (0..8)
            .map(|c| {
                std::array::from_fn(|d| {
                    if c & (1 << d) == 0 {
                        body.lower[d]
                    } else {
                        body.upper[d]
                    }
                })
            })
            .collect(),
        vec![
            [0, 2, 3],
            [0, 3, 1],
            [4, 5, 7],
            [4, 7, 6],
            [0, 1, 5],
            [0, 5, 4],
            [2, 6, 7],
            [2, 7, 3],
            [0, 4, 6],
            [0, 6, 2],
            [1, 3, 7],
            [1, 7, 5],
        ],
        SurfaceSettings::default(),
    )
    .unwrap();
    StaticObstacleGeometry::new(
        GridGeometry::new(
            config.counts.map(|n| n as u64),
            config.spacing,
            config.origin,
        )
        .unwrap(),
        surface,
        4_000_000,
        |_, _| false,
    )
    .unwrap()
}
fn request(velocity: [f64; 3]) -> Request {
    Request {
        boundary: AnalyticField::Uniform(velocity),
        pressure_offset: 0.0,
        moment_reference: [0.0; 3],
    }
}
fn close(actual: f64, expected: f64, scale: f64) {
    assert!(
        (actual - expected).abs() <= 8192.0 * f64::EPSILON * scale,
        "actual={actual:e}, expected={expected:e}, scale={scale:e}"
    );
}
fn bits(s: &stokes::Solution) -> Vec<u64> {
    let mut v = Vec::new();
    for a in &s.face_fields {
        v.extend(a.iter().map(|x| x.to_bits()));
    }
    for a in &s.volume_forcing {
        v.extend(a.iter().map(|x| x.to_bits()));
    }
    v.extend(
        s.pressure
            .iter()
            .chain(&s.variable_values)
            .chain(&s.reactions)
            .map(|x| x.to_bits()),
    );
    for p in &s.prescribed {
        v.push(p.value.to_bits());
    }
    for t in s.outer_traces.iter().chain(&s.body_traces) {
        v.extend(t.velocity.map(f64::to_bits));
    }
    for p in &s.wall_patches {
        v.push(p.pressure.to_bits());
        for a in p.gradient {
            v.extend(a.map(f64::to_bits));
        }
    }
    v
}

#[test]
fn counts_preflight_caps_and_exact_geometry_authority() {
    let config = tiny();
    let geometry = owner(config);
    let solver = Solver::new(config, Some(&geometry)).unwrap();
    assert_eq!(solver.active_velocity_dofs(), 474);
    assert_eq!(solver.nongauge_pressure_dofs(), 195);
    assert_eq!(solver.actual_dofs(), 669);
    assert!(solver.allocated_bytes() <= config.payload_limit);
    let mut small = config;
    small.dof_limit = 668;
    assert!(matches!(
        Solver::new(small, Some(&geometry)),
        Err(Error::DofLimit {
            required: 669,
            limit: 668
        })
    ));
    let mut capped = config;
    capped.payload_limit = 1;
    assert!(matches!(
        Solver::new(capped, Some(&geometry)),
        Err(Error::BufferLimit { .. })
    ));
    let mut mismatch = config;
    mismatch.origin[0] += 0.25;
    assert!(matches!(
        Solver::new(mismatch, Some(&geometry)),
        Err(Error::GeometryMismatch)
    ));
    let mut huge = no_body();
    huge.counts = [128; 3];
    assert!(matches!(
        Solver::new(huge, None),
        Err(Error::DofLimit { .. })
    ));
}

#[test]
fn approved_corner_is_sector_half_cell_not_clipped_singleton() {
    let config = tiny();
    let geometry = owner(config);
    let solver = Solver::new(config, Some(&geometry)).unwrap();
    assert_eq!(stokes::CORNER_RULE, "rheon-i1-sector-half-cell-v1");
    let vx = solver.face_variable(0, [6, 3, 2]).unwrap();
    let vy = solver.face_variable(1, [6, 3, 2]).unwrap();
    let rows: Vec<_> = solver
        .strain_rows()
        .iter()
        .filter(|r| r.axes == [0, 1] && r.coordinates == [6, 3, 2])
        .collect();
    assert_eq!(rows.len(), 3);
    let mut half_dissipation = 0.0;
    for row in rows {
        let shear: f64 = row
            .terms()
            .iter()
            .map(|term| {
                term.coefficient
                    * if term.variable == vx {
                        0.25
                    } else if term.variable == vy {
                        -0.25
                    } else {
                        0.0
                    }
            })
            .sum();
        half_dissipation += 0.5 * row.hessian_weight * shear * shear;
    }
    // Independent local functional: mu*V*(X²+Y²+XY), X=1,Y=−1.
    assert_eq!(half_dissipation, 1.0 / 64.0);
    assert_ne!(half_dissipation, 3.0 / 8.0 * (1.0_f64 - 1.0).powi(2) / 64.0);
}

#[test]
fn uniform_all_axes_oblique_and_pressure_datum_are_actual_solutions() {
    let mut solver = Solver::new(no_body(), None).unwrap();
    let payload = solver.allocated_bytes();
    for velocity in [
        [0.01, 0.0, 0.0],
        [0.0, -0.01, 0.0],
        [0.0, 0.0, 0.01],
        [-0.01, 0.0025, -0.005],
    ] {
        let mut req = request(velocity);
        req.pressure_offset = 4.0;
        let s = solver.solve(req, |_, _| false).unwrap();
        assert_eq!(s.report.actual_dofs, 19);
        for (axis, field) in s.face_fields.iter().enumerate() {
            for &v in field {
                close(v, velocity[axis], 0.01);
            }
        }
        for &p in &s.pressure {
            close(p, 4.0, 4.0);
        }
        assert!(s.report.dissipation <= 8192.0 * f64::EPSILON * 0.01_f64.powi(2));
        assert!(s.wall_patches.is_empty());
        assert_eq!(solver.allocated_bytes(), payload);
    }
}

#[test]
fn every_cancel_stage_and_invalid_request_preserve_complete_publication() {
    let mut solver = Solver::new(no_body(), None).unwrap();
    solver
        .solve(request([0.01, 0.0025, -0.005]), |_, _| false)
        .unwrap();
    let before = bits(solver.accepted().unwrap());
    let report = solver.accepted().unwrap().report;
    let loads = solver.accepted().unwrap().loads;
    let prior = solver.accepted().unwrap().request;
    let payload = solver.allocated_bytes();
    for stage in [
        Stage::Input,
        Stage::Boundary,
        Stage::Assembly,
        Stage::Factorization,
        Stage::BackSubstitution,
        Stage::Residual,
        Stage::Reconstruction,
        Stage::Loads,
        Stage::Commit,
    ] {
        let err = solver
            .solve(request([-0.01, 0.005, 0.0025]), |s, _| s == stage)
            .unwrap_err();
        assert!(matches!(err,Error::Cancelled{stage:found,..} if found==stage));
        let accepted = solver.accepted().unwrap();
        assert_eq!(bits(accepted), before);
        assert_eq!(accepted.report, report);
        assert_eq!(accepted.loads, loads);
        assert_eq!(accepted.request, prior);
        assert_eq!(solver.allocated_bytes(), payload);
    }
    let mut invalid = request([0.01, 0.0, 0.0]);
    invalid.pressure_offset = f64::NAN;
    assert!(solver.solve(invalid, |_, _| false).is_err());
    assert_eq!(bits(solver.accepted().unwrap()), before);
    assert_eq!(solver.accepted().unwrap().report, report);
}

fn patch(
    point: [f64; 3],
    normal: [f64; 3],
    area: f64,
    p: f64,
    g: [[f64; 3]; 3],
    velocity: [f64; 3],
) -> WallPatch {
    WallPatch {
        cell: 0,
        axis: 0,
        side: 1,
        point,
        normal,
        area,
        velocity,
        pressure: p,
        gradient: g,
    }
}
#[test]
fn affine_pressure_buoyancy_moment_reference_and_newtonian_stress() {
    let dims = [1.0, 0.25, 0.25];
    let slope = [1.0, 2.0, -3.0];
    let reference = [0.125, -0.25, 0.375];
    let mut force = [0.0; 3];
    let mut moment = [0.0; 3];
    for axis in 0..3 {
        for side in [-1.0, 1.0] {
            let mut point = [0.0; 3];
            point[axis] = side * dims[axis] / 2.0;
            let mut normal = [0.0; 3];
            normal[axis] = side;
            let pressure = 1_000_000.0 + (0..3).map(|a| slope[a] * point[a]).sum::<f64>();
            let p = patch(
                point,
                normal,
                dims[(axis + 1) % 3] * dims[(axis + 2) % 3],
                pressure,
                [[0.0; 3]; 3],
                [0.0; 3],
            );
            let l = stokes::integrate_patch(&p, 1.0, reference).unwrap();
            for a in 0..3 {
                force[a] += l.pressure_force[a];
                moment[a] += l.pressure_moment[a];
            }
        }
    }
    assert_eq!(force, [-1.0 / 16.0, -2.0 / 16.0, 3.0 / 16.0]);
    let expected = [
        -reference[1] * force[2] + reference[2] * force[1],
        -reference[2] * force[0] + reference[0] * force[2],
        -reference[0] * force[1] + reference[1] * force[0],
    ];
    assert_eq!(moment, expected);
    let rotation = [[0.0, -2.0, 0.0], [2.0, 0.0, 0.0], [0.0; 3]];
    let p = patch([0.0; 3], [0.0, 1.0, 0.0], 1.0, 0.0, rotation, [0.0; 3]);
    assert_eq!(
        stokes::integrate_patch(&p, 1.0, [0.0; 3])
            .unwrap()
            .shear_force,
        [0.0; 3]
    );
    let shear = [[0.0, 1.0, 0.0], [0.0; 3], [0.0; 3]];
    let p = patch(
        [0.0, 1.0, 0.0],
        [0.0, -1.0, 0.0],
        1.0,
        0.0,
        shear,
        [1.0, 0.0, 0.0],
    );
    let load = stokes::integrate_patch(&p, 1.0, [0.0; 3]).unwrap();
    assert_eq!(load.shear_force, [-1.0, 0.0, 0.0]);
    assert_eq!(load.work, -1.0);
}

#[test]
fn tiny_body_solves_nonzero_drag_and_reversal_without_heap_growth() {
    let config = tiny();
    let geometry = owner(config);
    let mut solver = Solver::new(config, Some(&geometry)).unwrap();
    let payload = solver.allocated_bytes();
    let s = solver
        .solve(request([-0.01, 0.0, 0.0]), |_, _| false)
        .unwrap();
    assert_eq!(s.report.actual_dofs, 669);
    assert_eq!(s.report.solve_id, 1);
    assert_eq!(s.wall_patches.len(), 18);
    assert!(s.report.dissipation > 0.0);
    assert!(s.report.min_scaled_pivot >= 1e-12);
    assert!(s.loads.pressure_force[0] + s.loads.shear_force[0] < 0.0);
    // Body load is fluid-on-body; outer load is boundary-on-fluid. In this
    // unforced steady fixture their vectors agree by global momentum balance.
    assert!(s.loads.body_force[0] < 0.0);
    for a in 0..3 {
        close(s.loads.outer_force[a], s.loads.body_force[a], 0.01);
        close(s.loads.outer_moment[a], s.loads.body_moment[a], 0.01);
    }
    for t in &s.body_traces {
        assert_eq!(t.velocity, [0.0; 3]);
    }
    let load = s.loads;
    let dissipation = s.report.dissipation;
    let reverse = solver
        .solve(request([0.01, 0.0, 0.0]), |_, _| false)
        .unwrap();
    assert_eq!(reverse.report.solve_id, 2);
    for a in 0..3 {
        close(
            reverse.loads.pressure_force[a],
            -load.pressure_force[a],
            0.01,
        );
        close(reverse.loads.shear_force[a], -load.shear_force[a], 0.01);
        close(
            reverse.loads.pressure_moment[a],
            -load.pressure_moment[a],
            0.01,
        );
        close(reverse.loads.shear_moment[a], -load.shear_moment[a], 0.01);
    }
    close(reverse.report.dissipation, dissipation, dissipation);
    assert_eq!(solver.allocated_bytes(), payload);
}

#[test]
fn incompatible_geometry_and_nonfinite_controls_refuse_without_publication() {
    let mut cfg = no_body();
    cfg.spacing[1] = f64::INFINITY;
    assert!(Solver::new(cfg, None).is_err());
    let mut solver = Solver::new(no_body(), None).unwrap();
    let req = Request {
        boundary: AnalyticField::PolynomialMms {
            amplitude: 0.0001,
            pressure_linear: [1.0, 2.0, -3.0],
            pressure_quadratic: [1.0, -1.0, 0.5],
        },
        ..request([0.0; 3])
    };
    assert!(solver.solve(req, |_, _| false).is_err());
    assert!(solver.accepted().is_none());
    assert!(
        solver
            .solve(request([f64::NAN, 0.0, 0.0]), |_, _| false)
            .is_err()
    );
    assert!(solver.accepted().is_none());
}

#[test]
fn partial_wall_reconstruction_and_precommit_cancel_preserve_body_publication() {
    let config = tiny();
    let geometry = owner(config);
    let mut solver = Solver::new(config, Some(&geometry)).unwrap();
    solver
        .solve(request([-0.01, 0.0, 0.0]), |_, _| false)
        .unwrap();
    let before = bits(solver.accepted().unwrap());
    let report = solver.accepted().unwrap().report;
    let loads = solver.accepted().unwrap().loads;
    let prior = solver.accepted().unwrap().request;
    let payload = solver.allocated_bytes();
    for (stage, index) in [(Stage::Reconstruction, 5), (Stage::Commit, 0)] {
        let result = solver.solve(request([0.01, 0.0025, -0.005]), |s, i| {
            s == stage && i == index
        });
        assert!(
            matches!(result, Err(Error::Cancelled { stage: found, index: at }) if found == stage && at == index)
        );
        let accepted = solver.accepted().unwrap();
        assert_eq!(bits(accepted), before);
        assert_eq!(accepted.report, report);
        assert_eq!(accepted.loads, loads);
        assert_eq!(accepted.request, prior);
        assert_eq!(solver.allocated_bytes(), payload);
    }
}

#[test]
fn tiny_mms_small_velocity_satisfies_physical_budgets_with_large_pressure() {
    let config = tiny();
    let geometry = owner(config);
    let mut solver = Solver::new(config, Some(&geometry)).unwrap();
    let payload = solver.allocated_bytes();
    let req = Request {
        boundary: AnalyticField::PolynomialMms {
            amplitude: 0.0001,
            pressure_linear: [1.0, 2.0, -3.0],
            pressure_quadratic: [1.0, -1.0, 0.5],
        },
        ..request([0.0; 3])
    };
    let accepted = solver.solve(req, |_, _| false).unwrap();
    assert_eq!(accepted.report.actual_dofs, 669);
    assert!(accepted.report.max_residual_ratio <= 1.0);
    assert!(accepted.report.dissipation > 0.0);
    assert!(accepted.face_fields[0].iter().any(|u| u.abs() > 1e-8));
    let before = bits(accepted);
    let prior_report = accepted.report;
    let prior_loads = accepted.loads;
    let mut factor_starts = 0;
    let refused = solver.solve(req, |stage, index| {
        if stage == Stage::Factorization && index == 0 {
            factor_starts += 1;
        }
        // Initial factorization has an entry and a k=0 checkpoint; this
        // checkpoint is reached only when a correction factorization starts.
        factor_starts == 3
    });
    assert!(matches!(
        refused,
        Err(Error::Cancelled {
            stage: Stage::Factorization,
            index: 0
        })
    ));
    let retained = solver.accepted().unwrap();
    assert_eq!(bits(retained), before);
    assert_eq!(retained.report, prior_report);
    assert_eq!(retained.loads, prior_loads);
    assert_eq!(retained.request, req);
    assert_eq!(solver.allocated_bytes(), payload);
}
