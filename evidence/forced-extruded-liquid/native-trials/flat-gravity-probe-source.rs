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
fn owner(
    pressure: bool,
    xi: [f64; 12],
    settings: TranslatedViscousSettings,
) -> CoupledDiscreteFlow {
    let mut u = velocity(xi);
    if pressure {
        for i in 0..16 {
            u[i][0] = PRESSURE_VELOCITY[i][0];
            u[i][1] = PRESSURE_VELOCITY[i][1];
        }
    }
    CoupledDiscreteFlow::new_forced_extruded(geometry(), &u, Default::default(), settings, 131)
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

#[allow(clippy::excessive_precision)]
const PRESSURE_VELOCITY: [[f64; 3]; 16] = [
    [
        3.05711307537577743e-03,
        0.00000000000000000e+00,
        0.00000000000000000e+00,
    ],
    [
        2.99012462001119281e-03,
        0.00000000000000000e+00,
        0.00000000000000000e+00,
    ],
    [
        9.97567673065870020e-01,
        -1.29229135485843158e-04,
        0.00000000000000000e+00,
    ],
    [
        1.24654222231385337e+00,
        1.29229135485843158e-04,
        0.00000000000000000e+00,
    ],
    [
        4.17369181981730564e-01,
        5.58237128038205027e-05,
        0.00000000000000000e+00,
    ],
    [
        7.49030321079115602e-01,
        1.75522252119088821e-04,
        0.00000000000000000e+00,
    ],
    [
        3.34607844704580992e-01,
        -4.46589702430564076e-05,
        0.00000000000000000e+00,
    ],
    [
        7.48948613597292256e-01,
        -1.34668511207567900e-04,
        0.00000000000000000e+00,
    ],
    [
        3.02361884769348534e-03,
        0.00000000000000000e+00,
        0.00000000000000000e+00,
    ],
    [
        5.41819082891848214e-01,
        1.59381619717028403e-05,
        0.00000000000000000e+00,
    ],
    [
        6.01677256844570740e-01,
        2.52039089822280328e-04,
        0.00000000000000000e+00,
    ],
    [
        3.02361884769348534e-03,
        0.00000000000000000e+00,
        0.00000000000000000e+00,
    ],
    [
        5.83158897789511466e-01,
        -9.52321577325538103e-05,
        0.00000000000000000e+00,
    ],
    [
        5.18728612129379263e-01,
        -1.46034043484988198e-04,
        0.00000000000000000e+00,
    ],
    [
        1.12205494768986158e+00,
        0.00000000000000000e+00,
        0.00000000000000000e+00,
    ],
    [
        1.12205494768986158e+00,
        0.00000000000000000e+00,
        0.00000000000000000e+00,
    ],
];

const A: [f64; 3] = [0.0625, -0.125, 0.03125];
fn force(a: [f64; 3]) -> BodyForce {
    BodyForce {
        value: a,
        units: ForceUnits::Acceleration,
        region: None,
    }
}
fn momentum(o: &CoupledDiscreteFlow, d: usize) -> f64 {
    let s = o.state();
    s.geometry
        .nodal_mass()
        .iter()
        .zip(s.velocity)
        .map(|(m, u)| m * u[d])
        .sum()
}
fn energy(o: &CoupledDiscreteFlow) -> f64 {
    let s = o.state();
    s.geometry
        .nodal_mass()
        .iter()
        .zip(s.velocity)
        .map(|(m, u)| 0.5 * m * u.iter().map(|x| x * x).sum::<f64>())
        .sum()
}
#[test]
fn actual_forced_publication_has_known_impulses_and_complete_signed_work() {
    for pressure in [false, true] {
        let mut o = owner(pressure, XI, Default::default());
        let m0 = o.state().geometry.nodal_mass().to_vec();
        let p0 = *o.state().pressure_coefficients;
        let px = momentum(&o, 0);
        let pz = momentum(&o, 2);
        let e0 = energy(&o);
        let bytes = o.allocated_bytes();
        let r = o
            .step_extruded_with_forces(0.05, &[force(A)], |_| false)
            .unwrap();
        let s = o.state();
        assert_eq!(s.stamp, r.step.planar.after);
        assert_eq!(s.time, r.step.planar.time_after);
        assert_eq!(
            s.pressure_coefficients.as_slice(),
            &r.step.planar.unknowns[6..]
        );
        assert_ne!(*s.pressure_coefficients, p0);
        assert!((momentum(&o, 0) - px - r.forces.horizontal_impulse).abs() < 1e-13);
        assert!((momentum(&o, 2) - pz - r.forces.third_impulse).abs() < 1e-13);
        assert!((r.forces.horizontal_impulse - 0.05 * 3.375 * A[0]).abs() < 1e-15);
        assert!((r.forces.third_impulse - 0.05 * 3.375 * A[2]).abs() < 1e-15);
        let wf: f64 = m0
            .iter()
            .zip(s.velocity)
            .map(|(m, u)| 0.05 * m * (u[0] * A[0] + u[1] * A[1] + u[2] * A[2]))
            .sum();
        assert!((wf - r.forces.total_work).abs() < r.step.work_allowance);
        let ledger = energy(&o) - e0
            + r.step.backward_euler_loss
            + r.step.mixing_loss
            + r.step.viscous_loss
            + r.step.planar.pressure_work
            + r.step.gcl_work
            - r.forces.total_work
            - r.step.residual_work;
        assert!(ledger.abs() <= r.step.work_allowance);
        assert!(r.step.ledger_error.abs() <= r.step.work_allowance);
        assert!(r.step.third.shear_x_loss > 0. && r.step.third.shear_y_loss > 0.);
        assert!(
            r.step.planar.finite_momentum_rate_norm <= 1e-13
                && r.step.third.finite_momentum_rate_norm <= 1e-11
        );
        assert_eq!(bytes, o.allocated_bytes());
    }
}
#[test]
fn constant_third_acceleration_and_force_supplied_energy_growth() {
    let mut o = owner(false, [0.25; 12], Default::default());
    let e0 = energy(&o);
    for n in 1..=4 {
        let r = o
            .step_extruded_with_forces(0.025, &[force([0., 0., 0.25])], |_| false)
            .unwrap();
        for u in o.state().velocity {
            assert!(
                (u[2] - (0.25 + f64::from(n) * 0.025 * 0.25)).abs() < 128. * f64::EPSILON * 0.25
            );
        }
        assert!(r.forces.third_work > 0.);
        assert!(r.step.third.ledger_error.abs() <= r.step.third.work_allowance);
    }
    assert!(energy(&o) > e0);
    let r = o
        .step_extruded_with_forces(0.025, &[force([0., 0., -0.25])], |_| false)
        .unwrap();
    assert!(r.forces.third_work < 0.);
    assert!(r.step.third.energy_after < r.step.third.energy_before);
}
#[test]
fn force_density_and_ordered_split_acceleration_publish_identical_bits() {
    for pressure in [false, true] {
        let mut a = owner(pressure, XI, Default::default());
        let mut density = owner(pressure, XI, Default::default());
        let mut split = owner(pressure, XI, Default::default());
        for (h, sign) in [(0.05, 1.), (0.025, -1.)] {
            let value = A.map(|x| sign * x);
            a.step_extruded_with_forces(h, &[force(value)], |_| false)
                .unwrap();
            density
                .step_extruded_with_forces(
                    h,
                    &[BodyForce {
                        value: value.map(|x| 3. * x),
                        units: ForceUnits::ForceDensity,
                        region: None,
                    }],
                    |_| false,
                )
                .unwrap();
            split
                .step_extruded_with_forces(
                    h,
                    &[force(value.map(|x| x / 2.)), force(value.map(|x| x / 2.))],
                    |_| false,
                )
                .unwrap();
            assert_eq!(snapshot(&a), snapshot(&density));
            assert_eq!(snapshot(&a), snapshot(&split));
        }
    }
}
#[test]
fn all_force_and_candidate_barriers_preserve_both_fixtures_and_retry_bits() {
    let forces = [force(A.map(|x| x / 2.)), force(A.map(|x| x / 2.))];
    for pressure in [false, true] {
        let mut baseline = owner(pressure, XI, Default::default());
        baseline
            .step_extruded_with_forces(0.05, &forces, |_| false)
            .unwrap();
        baseline
            .step_extruded_with_forces(0.025, &forces, |_| false)
            .unwrap();
        let expected = snapshot(&baseline);
        for stage in [
            CoupledDiscreteStage::BeforeForceInputs,
            CoupledDiscreteStage::ForceInput,
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
            let mut o = owner(pressure, XI, Default::default());
            o.step_extruded_with_forces(0.05, &forces, |_| false)
                .unwrap();
            let before = snapshot(&o);
            let mut visits = 0;
            let error = o
                .step_extruded_with_forces(0.025, &forces, |s| {
                    if s == stage {
                        visits += 1;
                    }
                    s == stage
                        && visits
                            >= match stage {
                                CoupledDiscreteStage::ForceInput => 2,
                                CoupledDiscreteStage::Quadrature
                                | CoupledDiscreteStage::ThirdAssembly => 7,
                                _ => 1,
                            }
                })
                .unwrap_err();
            assert_eq!(error, CoupledDiscreteError::Cancelled { stage });
            assert_eq!(before, snapshot(&o));
            o.step_extruded_with_forces(0.025, &forces, |_| false)
                .unwrap();
            assert_eq!(expected, snapshot(&o));
        }
    }
}
#[test]
fn force_inputs_and_time_refusals_preserve_nonzero_accepted_pressure_state() {
    let mut o = owner(true, XI, Default::default());
    o.step_extruded_with_forces(0.05, &[force(A)], |_| false)
        .unwrap();
    let before = snapshot(&o);
    let mut region = force(A);
    region.region = Some(ForceRegion {
        lower: [0.; 3],
        upper: [1.; 3],
    });
    for invalid in [
        force([f64::NAN, 0., 0.]),
        force([f64::INFINITY, 0., 0.]),
        force([f64::from_bits(1), 0., 0.]),
        region,
        BodyForce {
            value: [f64::MIN_POSITIVE, 0., 0.],
            units: ForceUnits::ForceDensity,
            region: None,
        },
    ] {
        assert!(
            o.step_extruded_with_forces(0.025, &[invalid], |_| false)
                .is_err()
        );
        assert_eq!(before, snapshot(&o));
    }
    assert!(matches!(
        o.step_extruded_with_forces(0.025, &[force(A); 17], |_| false),
        Err(CoupledDiscreteError::ForceLimit {
            provided: 17,
            maximum: 16
        })
    ));
    assert_eq!(before, snapshot(&o));
    assert!(
        o.step_extruded_with_forces(0.025, &[force([f64::MAX, 0., 0.]); 2], |_| false)
            .is_err()
    );
    assert_eq!(before, snapshot(&o));
    for h in [
        0.,
        -0.025,
        f64::NAN,
        f64::INFINITY,
        f64::from_bits(1),
        0.0500001,
    ] {
        assert!(
            o.step_extruded_with_forces(h, &[force(A)], |_| false)
                .is_err()
        );
        assert_eq!(before, snapshot(&o));
    }
}
#[test]
fn actual_late_third_failure_preserves_published_forced_pressure_fixture() {
    let mut o = owner(true, XI, Default::default());
    o.step_extruded_with_forces(0.05, &[force(A)], |_| false)
        .unwrap();
    let before = snapshot(&o);
    let mut assembly = 0;
    let error = o
        .step_extruded_with_forces(0.025, &[force([0., 0., 1e160])], |s| {
            if s == CoupledDiscreteStage::ThirdAssembly {
                assembly += 1;
            }
            false
        })
        .unwrap_err();
    eprintln!("actual late forced failure: {error:?}, assembled triangles={assembly}");
    assert_eq!(assembly, 24);
    assert_eq!(before, snapshot(&o));
    o.step_extruded_with_forces(0.025, &[force(A)], |_| false)
        .unwrap();
    let mut baseline = owner(true, XI, Default::default());
    baseline
        .step_extruded_with_forces(0.05, &[force(A)], |_| false)
        .unwrap();
    baseline
        .step_extruded_with_forces(0.025, &[force(A)], |_| false)
        .unwrap();
    assert_eq!(snapshot(&o), snapshot(&baseline));
}
#[test]
fn force_budget_and_iteration_refusals_are_atomic() {
    let required = CoupledDiscreteFlow::nominal_forced_extruded_bytes().unwrap();
    assert!(required > CoupledDiscreteFlow::nominal_extruded_bytes().unwrap());
    let error = CoupledDiscreteFlow::new_forced_extruded(
        geometry(),
        &velocity(XI),
        Default::default(),
        TranslatedViscousSettings {
            memory_limit: required - 1,
            ..Default::default()
        },
        131,
    )
    .err()
    .unwrap();
    assert!(
        matches!(error,CoupledDiscreteError::Flow(TranslatedViscousError::BufferLimit {required:r,limit:l})if r==required&&l==required-1)
    );
    for pressure in [false, true] {
        let mut o = owner(
            pressure,
            XI,
            TranslatedViscousSettings {
                max_iterations: 1,
                ..Default::default()
            },
        );
        let before = snapshot(&o);
        assert!(matches!(
            o.step_extruded_with_forces(0.05, &[force(A)], |_| false),
            Err(CoupledDiscreteError::IterationLimit)
        ));
        assert_eq!(before, snapshot(&o));
    }
    let mut old = CoupledDiscreteFlow::new_extruded(
        geometry(),
        &velocity(XI),
        Default::default(),
        Default::default(),
        131,
    )
    .unwrap();
    let before = snapshot(&old);
    assert!(matches!(
        old.step_extruded_with_forces(0.025, &[force(A)], |_| false),
        Err(CoupledDiscreteError::UnsupportedSlice)
    ));
    assert_eq!(before, snapshot(&old));
}
#[test]
fn empty_forcing_retains_published_legacy_extrusion_and_budget() {
    let mut forced = owner(false, XI, Default::default());
    let mut old = CoupledDiscreteFlow::new_extruded(
        geometry(),
        &velocity(XI),
        Default::default(),
        Default::default(),
        131,
    )
    .unwrap();
    assert_eq!(old.allocated_bytes(), 470992);
    assert_eq!(CoupledDiscreteFlow::nominal_bytes().unwrap(), 450320);
    for h in [0.05, 0.025] {
        let r = forced.step_extruded_with_forces(h, &[], |_| false).unwrap();
        old.step_extruded(h, |_| false).unwrap();
        assert_eq!(snapshot(&forced), snapshot(&old));
        assert_eq!(r.forces.total_work, 0.);
    }
}
#[test]
fn actual_flat_gravity_probe_checks_analytic_hydrostatic_pressure() {
    let cap = [[0., 1.125], [0.5, 1.125], [1., 1.125]];
    let result = CoupledDiscreteFlow::new_forced_extruded(
        FittedHeightGeometry {
            cap: &cap,
            ..geometry()
        },
        &[[0.; 3]; 16],
        Default::default(),
        Default::default(),
        131,
    );
    let mut o = result.unwrap();
    let r = o
        .step_extruded_with_forces(0.05, &[force([0., -0.125, 0.])], |_| false)
        .unwrap();
    let s = o.state();
    let mut p = [0.; 24];
    for term in s.geometry.pressure_basis() {
        p[term.triangle] += term.value * s.pressure_coefficients[term.mode];
    }
    for (j, tri) in s.geometry.triangles().iter().enumerate() {
        let y = tri
            .nodes
            .iter()
            .map(|&i| s.geometry.nodes()[i].position[1])
            .sum::<f64>()
            / 3.;
        assert!((p[j] - 3. * 0.125 * (1.125 - y)).abs() < 1e-11);
    }
    for u in s.velocity {
        assert!(u.iter().all(|x| x.abs() < 1e-11));
    }
    assert!(r.step.ledger_error.abs() <= r.step.work_allowance);
    eprintln!(
        "actual native flat hydrostatic step passed at stamp {:?}",
        s.stamp
    );
}
