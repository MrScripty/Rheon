#[allow(dead_code)]
#[path = "../experiments/aligned_stokes.rs"]
mod aligned_stokes;
use aligned_stokes::{Config, Error, Stage, Workspace};
use rheon::{
    AlignedStrain, Axis, GridGeometry, StaticObstacleGeometry, SurfaceSettings, SurfaceStamp,
    TriangleSurface,
};
fn owner(h: [f64; 3], origin: [f64; 3]) -> StaticObstacleGeometry {
    let lo: [f64; 3] = std::array::from_fn(|d| origin[d] + h[d]);
    let hi: [f64; 3] = std::array::from_fn(|d| origin[d] + 2.0 * h[d]);
    let mesh = TriangleSurface::new(
        SurfaceStamp {
            id: 108,
            version: 1,
        },
        (0..8)
            .map(|c| std::array::from_fn(|d| if c & (1 << d) == 0 { lo[d] } else { hi[d] }))
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
        GridGeometry::new([3; 3], h, origin).unwrap(),
        mesh,
        4_000_000,
        |_, _| false,
    )
    .unwrap()
}
fn zeros(o: &StaticObstacleGeometry) -> [Vec<f64>; 3] {
    std::array::from_fn(|d| vec![0.0; o.grid().face_len([Axis::X, Axis::Y, Axis::Z][d])])
}
fn curl(o: &StaticObstacleGeometry) -> [Vec<f64>; 3] {
    let mut u = zeros(o);
    for (a, p, q) in [
        (Axis::X, [1, 0, 0], 0.5),
        (Axis::X, [1, 1, 0], -0.5),
        (Axis::Y, [0, 1, 0], -0.5),
        (Axis::Y, [1, 1, 0], 0.5),
    ] {
        let d = match a {
            Axis::X => 0,
            Axis::Y => 1,
            Axis::Z => 2,
        };
        let f = o.grid().face_index(a, p).unwrap();
        u[d][f] = q / o.open_areas(a)[f];
    }
    u
}
fn step(
    w: &mut Workspace<'_, '_>,
    u: &mut [Vec<f64>; 3],
    dt: f64,
) -> Result<aligned_stokes::Report, Error> {
    let [x, y, z] = u;
    w.step([x, y, z], dt, |_, _| false)
}
#[test]
fn resting_state_is_certified_identity() {
    let o = owner([1.0; 3], [0.0; 3]);
    let op = AlignedStrain::new(&o, 1.0, 1.0, 4_000_000, |_, _| false).unwrap();
    let mut w = Workspace::new(&op, Config::default(), 4_000_000, |_, _| false).unwrap();
    let mut u = zeros(&o);
    assert!(w.last_report().is_none());
    let r = step(&mut w, &mut u, 1.0 / 32.0).unwrap();
    assert_eq!(r.viscous_energy_delta.lo, 0.0);
    assert_eq!(r.viscous_energy_delta.hi, 0.0);
    assert_eq!(r.pressure_energy_delta.hi, 0.0);
    assert!(u.iter().flatten().all(|&v| v == 0.0));
}
#[test]
fn nonzero_curl_executes_real_pressure_correction() {
    let o = owner([1.0; 3], [0.0; 3]);
    let op = AlignedStrain::new(&o, 1.0, 1.0, 4_000_000, |_, _| false).unwrap();
    let mut w = Workspace::new(&op, Config::default(), 4_000_000, |_, _| false).unwrap();
    let mut u = curl(&o);
    let r = step(&mut w, &mut u, 1.0 / 32.0).unwrap();
    assert!(r.viscous_energy_delta.hi < 0.0);
    assert!(r.pressure_energy_delta.hi < 0.0);
    assert!(r.final_divergence.bound.hi <= Config::default().divergence_limit);
    assert!(
        w.viscous_active()
            .unwrap()
            .iter()
            .zip(w.final_active().unwrap())
            .any(|(v, z)| v.to_bits() != z.to_bits())
    );
    let energy = |field: &[f64]| {
        field
            .iter()
            .zip(op.active_faces())
            .map(|(u, f)| 0.5 * f.mass * u * u)
            .sum::<f64>()
    };
    assert_eq!(energy(w.initial_active().unwrap()), 0.5);
    assert_eq!(energy(w.viscous_active().unwrap()), 679.0 / 2048.0);
    assert!((energy(w.final_active().unwrap()) - 303995.0 / 917504.0).abs() < 1e-14);
}
#[test]
fn three_accepted_substeps_evolve_the_same_caller_state() {
    let o = owner([1.0; 3], [0.0; 3]);
    let op = AlignedStrain::new(&o, 1.0, 1.0, 4_000_000, |_, _| false).unwrap();
    let mut w = Workspace::new(&op, Config::default(), 4_000_000, |_, _| false).unwrap();
    let mut u = curl(&o);
    let mut last_energy = 0.5;
    for _ in 0..3 {
        let before: Vec<f64> = op
            .active_faces()
            .iter()
            .map(|f| {
                u[match f.axis {
                    Axis::X => 0,
                    Axis::Y => 1,
                    Axis::Z => 2,
                }][f.face]
            })
            .collect();
        let r = step(&mut w, &mut u, 1.0 / 32.0).unwrap();
        assert_eq!(w.initial_active().unwrap(), before);
        assert_eq!(r.dt, 1.0 / 32.0);
        assert!(r.viscous_energy_delta.hi < 0.0 && r.pressure_energy_delta.hi < 0.0);
        let energy: f64 = w
            .final_active()
            .unwrap()
            .iter()
            .zip(op.active_faces())
            .map(|(u, f)| 0.5 * f.mass * u * u)
            .sum();
        assert!(energy < last_energy);
        last_energy = energy;
        assert_eq!(w.last_report(), Some(&r));
    }
}
#[test]
fn every_cancellation_stage_preserves_caller_and_previous_accepted_witness() {
    let o = owner([1.0; 3], [0.0; 3]);
    let op = AlignedStrain::new(&o, 1.0, 1.0, 4_000_000, |_, _| false).unwrap();
    let mut w = Workspace::new(&op, Config::default(), 4_000_000, |_, _| false).unwrap();
    let mut u = curl(&o);
    step(&mut w, &mut u, 1.0 / 32.0).unwrap();
    let accepted = u.clone();
    let report = *w.last_report().unwrap();
    let initial = w.initial_active().unwrap().to_vec();
    let viscous = w.viscous_active().unwrap().to_vec();
    let final_field = w.final_active().unwrap().to_vec();
    let pressure = w.pressure().unwrap().to_vec();
    let update = w.update_certificates().unwrap().to_vec();
    let momentum = w.momentum_certificates().unwrap().to_vec();
    let stages = [
        Stage::Input,
        Stage::InitialDivergence,
        Stage::Bound,
        Stage::Action,
        Stage::ViscousUpdate,
        Stage::ViscousEquation,
        Stage::ViscousEnergy,
        Stage::Pressure(rheon::ObstacleFlowStage::Solve),
        Stage::Pressure(rheon::ObstacleFlowStage::Correction),
        Stage::Pressure(rheon::ObstacleFlowStage::Acceptance),
        Stage::PressureEquation,
        Stage::PressureEnergy,
        Stage::FinalDivergence,
        Stage::Commit,
    ];
    for stage in stages {
        let [x, y, z] = &mut u;
        assert!(
            matches!(w.step([x,y,z],1.0/32.0,|s,_|s==stage),Err(Error::Cancelled{stage:s,..}) if s==stage)
        );
        assert_eq!(u, accepted);
        assert_eq!(w.last_report(), Some(&report));
        assert_eq!(w.initial_active().unwrap(), initial);
        assert_eq!(w.viscous_active().unwrap(), viscous);
        assert_eq!(w.final_active().unwrap(), final_field);
        assert_eq!(w.pressure().unwrap(), pressure);
        assert_eq!(w.update_certificates().unwrap(), update);
        assert_eq!(w.momentum_certificates().unwrap(), momentum);
    }
}
#[test]
fn bound_uncertainty_and_large_step_refuse_without_rounding_authority() {
    let o = owner([1.0; 3], [0.0; 3]);
    let op = AlignedStrain::new(&o, 1.0, 1.0, 4_000_000, |_, _| false).unwrap();
    let mut w = Workspace::new(&op, Config::default(), 4_000_000, |_, _| false).unwrap();
    assert!(w.bound().lo <= 17.0 && w.bound().hi > 17.0);
    for dt in [2.0 / 17.0, 1.0] {
        let mut u = curl(&o);
        let old = u.clone();
        assert!(matches!(
            step(&mut w, &mut u, dt),
            Err(Error::StepBound { .. })
        ));
        assert_eq!(u, old);
        assert!(w.pressure().is_none() && w.last_report().is_none());
    }
}
#[test]
fn independent_initial_and_final_flux_gates_include_gauge_cell() {
    let o = owner([1.0; 3], [0.0; 3]);
    let op = AlignedStrain::new(&o, 1.0, 1.0, 4_000_000, |_, _| false).unwrap();
    let mut w = Workspace::new(&op, Config::default(), 4_000_000, |_, _| false).unwrap();
    let mut u = curl(&o);
    let f = o.grid().face_index(Axis::X, [1, 0, 0]).unwrap();
    u[0][f] += 0.1;
    let old = u.clone();
    assert!(matches!(
        step(&mut w, &mut u, 1.0 / 32.0),
        Err(Error::Divergence {
            stage: Stage::InitialDivergence,
            cell: 0,
            ..
        })
    ));
    assert_eq!(u, old);
    let c = Config {
        divergence_limit: 0.0,
        ..Config::default()
    };
    let mut w = Workspace::new(&op, c, 4_000_000, |_, _| false).unwrap();
    let mut u = curl(&o);
    let old = u.clone();
    assert!(matches!(
        step(&mut w, &mut u, 1.0 / 32.0),
        Err(Error::Divergence {
            stage: Stage::FinalDivergence,
            ..
        })
    ));
    assert_eq!(u, old);
}
#[test]
fn zero_relative_allowance_refuses_unresolved_defect_with_no_absolute_floor() {
    let o = owner([1.0; 3], [0.0; 3]);
    let op = AlignedStrain::new(&o, 1.0, 1.0, 4_000_000, |_, _| false).unwrap();
    let c = Config {
        relative_update_limit: 0.0,
        ..Config::default()
    };
    let mut w = Workspace::new(&op, c, 4_000_000, |_, _| false).unwrap();
    let mut u = curl(&o);
    let old = u.clone();
    assert!(matches!(
        step(&mut w, &mut u, 1.0 / 32.0),
        Err(Error::Equation {
            stage: Stage::ViscousEquation,
            ..
        })
    ));
    assert_eq!(u, old);
    let mut rest = zeros(&o);
    assert!(step(&mut w, &mut rest, 1.0 / 32.0).is_ok());
}
#[test]
fn anisotropic_nonmidpoint_step_checks_stored_mass_mismatch() {
    let o = owner([0.3, 0.7, 1.1], [100_000_000.0, -100_000_000.0, 0.1]);
    let op = AlignedStrain::new(&o, 3.7, 0.375, 4_000_000, |_, _| false).unwrap();
    let mut w = Workspace::new(&op, Config::default(), 4_000_000, |_, _| false).unwrap();
    let mut u = curl(&o);
    let r = step(&mut w, &mut u, 1.0 / 256.0).unwrap();
    assert!(r.viscous_energy_delta.hi < 0.0 && r.pressure_energy_delta.hi < 0.0);
    assert!(
        w.momentum_certificates()
            .unwrap()
            .iter()
            .all(|c| c.defect.lo.abs().max(c.defect.hi.abs()) <= c.allowance.lo)
    );
    assert!(
        op.active_faces()
            .iter()
            .any(|f| f.mass / (op.density() * f.distance) != f.area)
    );
}
#[test]
fn managed_cap_counts_operator_pressure_attempt_and_accepted_caches() {
    let o = owner([1.0; 3], [0.0; 3]);
    let op = AlignedStrain::new(&o, 1.0, 1.0, 4_000_000, |_, _| false).unwrap();
    let w = Workspace::new(&op, Config::default(), 4_000_000, |_, _| false).unwrap();
    let cap = w.combined_bytes();
    assert_eq!(cap, w.allocated_bytes() + op.allocated_bytes());
    assert!(std::ptr::eq(w.operator(), &op));
    assert!(Workspace::new(&op, Config::default(), cap, |_, _| false).is_ok());
    assert!(matches!(
        Workspace::new(&op, Config::default(), cap - 1, |_, _| false),
        Err(Error::BufferLimit { .. })
    ));
    assert!(matches!(
        Workspace::new(
            &op,
            Config {
                relative_update_limit: 1.1e-8,
                ..Config::default()
            },
            cap,
            |_, _| false
        ),
        Err(Error::InvalidParameter)
    ));
    assert!(matches!(
        Workspace::new(&op, Config::default(), cap, |_, _| true),
        Err(Error::Cancelled {
            stage: Stage::Construction,
            ..
        })
    ));
}
#[test]
fn bad_inputs_and_pressure_proposal_refusal_are_transactional() {
    let o = owner([1.0; 3], [0.0; 3]);
    let op = AlignedStrain::new(&o, 1.0, 1.0, 4_000_000, |_, _| false).unwrap();
    let mut w = Workspace::new(&op, Config::default(), 4_000_000, |_, _| false).unwrap();
    for value in [f64::NAN, f64::from_bits(1)] {
        let mut u = curl(&o);
        u[0][1] = value;
        let bits: Vec<_> = u.iter().flatten().map(|x| x.to_bits()).collect();
        assert!(step(&mut w, &mut u, 1.0 / 32.0).is_err());
        assert_eq!(
            u.iter().flatten().map(|x| x.to_bits()).collect::<Vec<_>>(),
            bits
        );
    }
    let mut u = curl(&o);
    u[0][0] = 0.25;
    let old = u.clone();
    assert!(matches!(
        step(&mut w, &mut u, 1.0 / 32.0),
        Err(Error::NonzeroWallSpeed {
            axis: Axis::X,
            face: 0
        })
    ));
    assert_eq!(u, old);
    let mut u = curl(&o);
    u[1].clear();
    let old = u.clone();
    assert!(matches!(
        step(&mut w, &mut u, 1.0 / 32.0),
        Err(Error::ShapeMismatch)
    ));
    assert_eq!(u, old);
    let mut c = Config::default();
    c.pressure.max_iterations = 0;
    let mut w = Workspace::new(&op, c, 4_000_000, |_, _| false).unwrap();
    let mut u = curl(&o);
    let old = u.clone();
    assert!(matches!(
        step(&mut w, &mut u, 1.0 / 32.0),
        Err(Error::Pressure(
            rheon::ObstacleFlowError::IterationLimit { .. }
        ))
    ));
    assert_eq!(u, old);
    assert!(w.last_report().is_none());
}
