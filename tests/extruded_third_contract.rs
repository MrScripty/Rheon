use rheon::*;
const CAP: [[f64; 2]; 3] = [[0., 1.], [0.5, 1.25], [1., 1.]];
const XI: [f64; 12] = [
    0.25, 0.5, -0.125, 0.375, 0.625, -0.25, 0.1875, 0.4375, -0.0625, 0.5625, 0.3125, 0.125,
];
fn geometry() -> FittedHeightGeometry<'static> {
    FittedHeightGeometry {
        cap: &CAP,
        bottom_x: &[0., 0.5, 1.],
        extrusion_width: 1.,
        density: 3.,
        dynamic_viscosity: 0.05,
    }
}
fn velocity(xi: [f64; 12]) -> [[f64; 3]; 16] {
    let g = FittedHeightWorkspace::new(geometry(), Default::default(), |_| false).unwrap();
    let mut z = [0.; 34];
    for n in g.nodes() {
        let e = g.velocity_embedding(n.periodic_index, 0).unwrap();
        if e.weights[0] == 1.
            && let Some(j) = e.columns[0]
        {
            z[j] = n.position[1];
        }
    }
    z[22..].copy_from_slice(&xi);
    let mut u = [[0.; 3]; 16];
    g.embed_velocity(&z, &mut u).unwrap();
    u
}
fn owner(xi: [f64; 12], settings: TranslatedViscousSettings) -> CoupledDiscreteFlow {
    CoupledDiscreteFlow::new_extruded(geometry(), &velocity(xi), Default::default(), settings, 91)
        .unwrap()
}
#[derive(Debug, PartialEq, Eq)]
struct Snapshot {
    velocity: Vec<[u64; 3]>,
    positions: Vec<[u64; 2]>,
    mass: Vec<u64>,
    pressure: [u64; 16],
    time: u64,
    stamp: TranslatedViscousStamp,
}
fn snapshot(o: &CoupledDiscreteFlow) -> Snapshot {
    let s = o.state();
    Snapshot {
        velocity: s.velocity.iter().map(|u| u.map(f64::to_bits)).collect(),
        positions: s
            .geometry
            .nodes()
            .iter()
            .map(|n| n.position.map(f64::to_bits))
            .collect(),
        mass: s
            .geometry
            .nodal_mass()
            .iter()
            .map(|m| m.to_bits())
            .collect(),
        pressure: s.pressure_coefficients.map(f64::to_bits),
        time: s.time.to_bits(),
        stamp: s.stamp,
    }
}
#[test]
fn actual_published_full_velocity_has_both_shears_and_total_work() {
    let mut o = owner(XI, Default::default());
    let before = snapshot(&o);
    let bytes = o.allocated_bytes();
    assert_eq!(
        bytes,
        CoupledDiscreteFlow::nominal_extruded_bytes().unwrap()
    );
    let r = o.step_extruded(0.05, |_| false).unwrap();
    let s = o.state();
    assert_ne!(before, snapshot(&o));
    assert_eq!(s.stamp, r.planar.after);
    assert_eq!(s.time, r.planar.time_after);
    assert_eq!(s.pressure_coefficients.as_slice(), &r.planar.unknowns[6..]);
    let mass = s.geometry.nodal_mass();
    let energy: f64 = mass
        .iter()
        .zip(s.velocity)
        .map(|(m, u)| 0.5 * m * u.iter().map(|v| v * v).sum::<f64>())
        .sum();
    assert!((energy - r.energy_after).abs() <= r.work_allowance);
    let momentum: f64 = mass.iter().zip(s.velocity).map(|(m, u)| m * u[2]).sum();
    assert!((momentum - r.third.momentum_after).abs() < 1e-14);
    assert!((r.third.momentum_after - r.third.momentum_before).abs() < 1e-14);
    assert!(r.third.shear_x_loss > 0. && r.third.shear_y_loss > 0. && r.third.mixing_loss > 0.);
    assert!(r.third.energy_after < r.third.energy_before && r.energy_after < r.energy_before);
    assert!(
        r.third.residual_work.abs() <= r.third.work_allowance
            && r.third.ledger_error.abs() <= r.third.work_allowance
    );
    assert!(r.residual_work.abs() <= r.work_allowance && r.ledger_error.abs() <= r.work_allowance);
    for (i, u) in s.velocity.iter().enumerate() {
        let e = s.geometry.velocity_embedding(i, 2).unwrap();
        let mut w = 0.;
        for k in 0..2 {
            if let Some(j) = e.columns[k] {
                w += e.weights[k] * r.third.coefficients[j - 22];
            }
        }
        assert_eq!(w.to_bits(), u[2].to_bits());
    }
    let cap = std::array::from_fn::<_, 3, _>(|i| s.geometry.nodes()[3 + i].position);
    let mut g = FittedHeightWorkspace::new(
        FittedHeightGeometry {
            cap: &cap,
            ..geometry()
        },
        Default::default(),
        |_| false,
    )
    .unwrap();
    let mut diagnostic = [FittedHeightNodeDiagnostic::default(); 16];
    let inspected = g
        .inspect(
            FittedHeightInputs {
                velocity: s.velocity,
                pressure_coefficients: s.pressure_coefficients,
            },
            &mut diagnostic,
            |_| false,
        )
        .unwrap();
    assert!((0.05 * inspected.strain_power - r.viscous_loss).abs() <= r.work_allowance);
    assert!(inspected.divergence_max <= 1e-11);
    assert_eq!(bytes, o.allocated_bytes());
}
#[test]
fn every_old_and_third_barrier_preserves_nonzero_published_state_and_retry_bits() {
    let mut baseline = owner(XI, Default::default());
    baseline.step_extruded(0.05, |_| false).unwrap();
    baseline.step_extruded(0.025, |_| false).unwrap();
    let expected = snapshot(&baseline);
    for stage in [
        CoupledDiscreteStage::BeforeGeometry,
        CoupledDiscreteStage::BeforeSolve,
        CoupledDiscreteStage::Quadrature,
        CoupledDiscreteStage::Iteration,
        CoupledDiscreteStage::BeforeAcceptance,
        CoupledDiscreteStage::BeforeThirdSolve,
        CoupledDiscreteStage::ThirdAssembly,
        CoupledDiscreteStage::AfterThirdSolve,
        CoupledDiscreteStage::BeforePublish,
    ] {
        let mut o = owner(XI, Default::default());
        o.step_extruded(0.05, |_| false).unwrap();
        let before = snapshot(&o);
        let mut visits = 0;
        let error = o
            .step_extruded(0.025, |s| {
                if s == stage {
                    visits += 1;
                }
                s == stage
                    && (!matches!(
                        stage,
                        CoupledDiscreteStage::Quadrature | CoupledDiscreteStage::ThirdAssembly
                    ) || visits == 7)
            })
            .unwrap_err();
        assert_eq!(error, CoupledDiscreteError::Cancelled { stage });
        assert_eq!(before, snapshot(&o));
        o.step_extruded(0.025, |_| false).unwrap();
        assert_eq!(expected, snapshot(&o));
    }
}
#[test]
fn constant_nonzero_third_velocity_survives_actual_moving_mass_publications() {
    let mut o = owner([0.25; 12], Default::default());
    let initial = snapshot(&o);
    for _ in 0..4 {
        let r = o.step_extruded(0.025, |_| false).unwrap();
        assert!(
            o.state()
                .velocity
                .iter()
                .all(|u| (u[2] - 0.25).abs() < 2e-15)
        );
        assert!(r.third.shear_x_loss + r.third.shear_y_loss < 1e-28);
        assert!(
            r.third.finite_momentum_rate_norm <= 1e-11
                && r.third.direct_momentum_rate_norm <= 1e-11
        );
    }
    assert_ne!(initial.positions, snapshot(&o).positions);
    assert_ne!(initial.mass, snapshot(&o).mass);
}
#[test]
fn invalid_arithmetic_and_wrong_api_preserve_nonzero_accepted_state() {
    let mut o = owner(XI, Default::default());
    o.step_extruded(0.05, |_| false).unwrap();
    let before = snapshot(&o);
    assert_eq!(
        o.step(0.025, |_| false).unwrap_err(),
        CoupledDiscreteError::UnsupportedSlice
    );
    assert_eq!(before, snapshot(&o));
    for h in [
        0.,
        -0.01,
        f64::NAN,
        f64::INFINITY,
        f64::from_bits(1),
        f64::MIN_POSITIVE,
        0.051,
        f64::MAX,
    ] {
        assert!(o.step_extruded(h, |_| false).is_err());
        assert_eq!(before, snapshot(&o));
    }
    o.step_extruded(0.025, |_| false).unwrap();
    assert_eq!(o.state().stamp.version, 2);
}
#[test]
fn bounded_solve_failure_preserves_all_initial_components() {
    let mut o = owner(
        XI,
        TranslatedViscousSettings {
            max_iterations: 1,
            ..Default::default()
        },
    );
    let before = snapshot(&o);
    assert_eq!(
        o.step_extruded(0.05, |_| false).unwrap_err(),
        CoupledDiscreteError::IterationLimit
    );
    assert_eq!(before, snapshot(&o));
}
#[test]
fn actual_third_linear_failure_preserves_the_published_constructor_state() {
    let mut o = owner(XI.map(|x| x * 1e120), Default::default());
    let before = snapshot(&o);
    let mut assembly_visits = 0;
    let error = o
        .step_extruded(0.05, |stage| {
            if stage == CoupledDiscreteStage::ThirdAssembly {
                assembly_visits += 1;
            }
            false
        })
        .unwrap_err();
    eprintln!(
        "ACTUAL third failure: {error:?}; assembly visits {assembly_visits}; accepted stamp {:?}",
        o.state().stamp
    );
    assert_eq!(assembly_visits, 24);
    assert!(matches!(
        error,
        CoupledDiscreteError::LinearFailure
            | CoupledDiscreteError::ThirdConservationFailure
            | CoupledDiscreteError::ThirdWorkFailure
    ));
    assert_eq!(before, snapshot(&o));
}
#[test]
fn additional_scratch_budget_and_initial_trace_are_enforced() {
    let bytes = CoupledDiscreteFlow::nominal_extruded_bytes().unwrap();
    assert!(bytes > CoupledDiscreteFlow::nominal_bytes().unwrap());
    assert!(matches!(
        CoupledDiscreteFlow::new_extruded(
            geometry(),
            &velocity(XI),
            Default::default(),
            TranslatedViscousSettings {
                memory_limit: bytes - 1,
                ..Default::default()
            },
            0
        ),
        Err(CoupledDiscreteError::Flow(
            TranslatedViscousError::BufferLimit { .. }
        ))
    ));
    let mut u = velocity(XI);
    u[8][2] += 0.01;
    assert!(
        CoupledDiscreteFlow::new_extruded(
            geometry(),
            &u,
            Default::default(),
            Default::default(),
            0
        )
        .is_err()
    );
    let mut u = velocity(XI);
    u[0][2] = f64::from_bits(1);
    assert!(
        CoupledDiscreteFlow::new_extruded(
            geometry(),
            &u,
            Default::default(),
            Default::default(),
            0
        )
        .is_err()
    );
}
#[test]
fn zero_third_opt_in_matches_legacy_actual_state_bit_for_bit() {
    let u = velocity([0.; 12]);
    let mut old =
        CoupledDiscreteFlow::new(geometry(), &u, Default::default(), Default::default(), 91)
            .unwrap();
    let mut new = owner([0.; 12], Default::default());
    assert_eq!(old.allocated_bytes(), 450320);
    for h in [0.05, 0.025] {
        old.step(h, |_| false).unwrap();
        let r = new.step_extruded(h, |_| false).unwrap();
        assert_eq!(snapshot(&old), snapshot(&new));
        assert_eq!(r.third.energy_after, 0.);
    }
}
