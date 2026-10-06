use rheon::*;
const CAP: [[f64; 2]; 3] = [[0., 1.], [0.5, 1.25], [1., 1.]];
fn geometry() -> FittedHeightGeometry<'static> {
    FittedHeightGeometry {
        cap: &CAP,
        bottom_x: &[0., 0.5, 1.],
        extrusion_width: 1.,
        density: 3.,
        dynamic_viscosity: 0.05,
    }
}
fn velocity() -> [[f64; 3]; 16] {
    let w = FittedHeightWorkspace::new(geometry(), Default::default(), |_| false).unwrap();
    let mut z = [0.; 34];
    for n in w.nodes() {
        let e = w.velocity_embedding(n.periodic_index, 0).unwrap();
        if e.weights[0] == 1.
            && let Some(j) = e.columns[0]
        {
            z[j] = n.position[1];
        }
    }
    let mut u = [[0.; 3]; 16];
    w.embed_velocity(&z, &mut u).unwrap();
    u
}
fn owner(settings: TranslatedViscousSettings) -> CoupledDiscreteFlow {
    CoupledDiscreteFlow::new(geometry(), &velocity(), Default::default(), settings, 81).unwrap()
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
        velocity: s.velocity.iter().map(|v| v.map(f64::to_bits)).collect(),
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
fn genuine_coupled_pressure_strain_mass_and_work_publish_together() {
    let mut o = owner(Default::default());
    let bytes = o.allocated_bytes();
    assert_eq!(bytes, CoupledDiscreteFlow::nominal_bytes().unwrap());
    let before = snapshot(&o);
    let r = o.step(0.05, |_| false).unwrap();
    let s = o.state();
    assert_ne!(before, snapshot(&o));
    assert_eq!(s.stamp.version, 1);
    assert_eq!(s.time, r.time_after);
    assert_eq!(s.pressure_coefficients.as_slice(), &r.unknowns[6..]);
    assert!(s.pressure_coefficients.iter().any(|p| p.abs() > 1e-4));
    assert!(s.velocity.iter().any(|u| u[1].abs() > 1e-5));
    assert!(r.backward_euler_loss > 0. && r.mixing_loss > 0. && r.viscous_loss > 0.);
    assert!(r.energy_after < r.energy_before);
    assert!(r.finite_momentum_rate_norm <= 1e-11 && r.direct_momentum_rate_norm <= 1e-11);
    assert!(
        r.residual_work.abs() <= r.work_allowance
            && r.ledger_error.abs() <= r.work_allowance
            && r.pressure_work.abs() <= r.work_allowance
    );
    assert!((r.mass_after - 3.375).abs() < 1e-14);
    assert!(r.endpoint_vs_path_momentum_max > 1e-9);
    assert_eq!(bytes, o.allocated_bytes());
}
#[test]
fn every_barrier_preserves_all_accepted_bits_after_nonzero_state_and_resumes() {
    let mut baseline = owner(Default::default());
    baseline.step(0.05, |_| false).unwrap();
    baseline.step(0.025, |_| false).unwrap();
    let expected = snapshot(&baseline);
    for stage in [
        CoupledDiscreteStage::BeforeGeometry,
        CoupledDiscreteStage::Quadrature,
        CoupledDiscreteStage::BeforeSolve,
        CoupledDiscreteStage::Iteration,
        CoupledDiscreteStage::BeforeAcceptance,
        CoupledDiscreteStage::BeforePublish,
    ] {
        let mut o = owner(Default::default());
        o.step(0.05, |_| false).unwrap();
        let before = snapshot(&o);
        let mut calls = 0;
        let err = o
            .step(0.025, |s| {
                if s == stage {
                    calls += 1;
                }
                s == stage && (stage != CoupledDiscreteStage::Quadrature || calls == 7)
            })
            .unwrap_err();
        assert_eq!(err, CoupledDiscreteError::Cancelled { stage });
        assert_eq!(before, snapshot(&o));
        o.step(0.025, |_| false).unwrap();
        assert_eq!(expected, snapshot(&o));
    }
}
#[test]
fn invalid_scale_intervals_preserve_nonzero_accepted_state() {
    let mut o = owner(Default::default());
    o.step(0.05, |_| false).unwrap();
    let before = snapshot(&o);
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
        assert!(o.step(h, |_| false).is_err());
        assert_eq!(before, snapshot(&o));
    }
    o.step(0.025, |_| false).unwrap();
    assert_eq!(o.state().stamp.version, 2);
}
#[test]
fn bounded_iteration_failure_preserves_initial_frame_pressure_clock() {
    let mut o = owner(TranslatedViscousSettings {
        max_iterations: 1,
        ..Default::default()
    });
    let before = snapshot(&o);
    assert_eq!(
        o.step(0.05, |_| false).unwrap_err(),
        CoupledDiscreteError::IterationLimit
    );
    assert_eq!(before, snapshot(&o));
}
#[test]
fn memory_and_unsupported_field_fail_before_owner_creation() {
    let required = CoupledDiscreteFlow::nominal_bytes().unwrap();
    let settings = TranslatedViscousSettings {
        memory_limit: required - 1,
        ..Default::default()
    };
    assert!(matches!(
        CoupledDiscreteFlow::new(geometry(), &velocity(), Default::default(), settings, 0),
        Err(CoupledDiscreteError::Flow(
            TranslatedViscousError::BufferLimit { .. }
        ))
    ));
    let mut u = velocity();
    u[0][2] = 0.01;
    assert!(matches!(
        CoupledDiscreteFlow::new(geometry(), &u, Default::default(), Default::default(), 0),
        Err(CoupledDiscreteError::UnsupportedSlice)
    ));
    let mut u = velocity();
    u[0][0] = f64::from_bits(1);
    assert!(
        CoupledDiscreteFlow::new(geometry(), &u, Default::default(), Default::default(), 0)
            .is_err()
    );
    let mut u = velocity();
    u[4][1] = 0.01;
    assert!(
        CoupledDiscreteFlow::new(geometry(), &u, Default::default(), Default::default(), 0)
            .is_err()
    );
}
