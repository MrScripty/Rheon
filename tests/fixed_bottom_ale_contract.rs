use rheon::*;
const CAP: [[f64; 2]; 5] = [[0., 1.], [0.25, 1.25], [0.5, 1.], [0.75, 1.5], [1., 1.]];
const BOTTOM: [f64; 5] = [0., 0.25, 0.5, 0.75, 1.];
fn geometry(mu: f64) -> FittedHeightGeometry<'static> {
    FittedHeightGeometry {
        cap: &CAP,
        bottom_x: &BOTTOM,
        extrusion_width: 1.,
        density: 3.,
        dynamic_viscosity: mu,
    }
}
fn initial(mu: f64, constant: bool) -> Vec<[f64; 3]> {
    let w = FittedHeightWorkspace::new(geometry(mu), FittedHeightSettings::default(), |_| false)
        .unwrap();
    let mut z = vec![0.; w.plan().reduced_velocity_unknowns];
    for n in w.nodes() {
        for d in [0, 2] {
            let row = w.velocity_embedding(n.periodic_index, d).unwrap();
            if row.weights[0] == 1.
                && let Some(j) = row.columns[0]
            {
                z[j] = if d == 0 {
                    0.25
                } else if constant {
                    0.125
                } else {
                    n.position[1] * n.position[1]
                };
            }
        }
    }
    let mut u = vec![[0.; 3]; w.plan().periodic_nodes];
    w.embed_velocity(&z, &mut u).unwrap();
    u
}
fn flow(settings: TranslatedViscousSettings) -> FixedBottomAleFlow {
    FixedBottomAleFlow::new(
        geometry(0.05),
        &initial(0.05, false),
        Default::default(),
        settings,
        71,
    )
    .unwrap()
}
#[derive(Debug, PartialEq, Eq)]
struct Snapshot {
    velocity: Vec<[u64; 3]>,
    positions: Vec<[u64; 2]>,
    masses: Vec<u64>,
    time: u64,
    offset: u64,
    stamp: TranslatedViscousStamp,
}
fn snapshot(w: &FixedBottomAleFlow) -> Snapshot {
    let s = w.state();
    Snapshot {
        velocity: s.velocity.iter().map(|v| v.map(f64::to_bits)).collect(),
        positions: s
            .geometry
            .nodes()
            .iter()
            .map(|n| n.position.map(f64::to_bits))
            .collect(),
        masses: s
            .geometry
            .nodal_mass()
            .iter()
            .map(|m| m.to_bits())
            .collect(),
        time: s.time.to_bits(),
        offset: s.cap_offset.to_bits(),
        stamp: s.stamp,
    }
}

#[test]
fn nonzero_relative_transport_and_viscosity_publish_one_geometry() {
    let mut w = flow(Default::default());
    assert_eq!(
        w.allocated_bytes(),
        FixedBottomAleFlow::nominal_bytes(4).unwrap()
    );
    assert_eq!(w.allocated_bytes(), 136352);
    let old_mass = w.state().geometry.nodal_mass().to_vec();
    let mut previous = f64::INFINITY;
    for version in 1..=20 {
        let actual_dt = 0.025_f64.min((0.125 - w.state().cap_offset) / 0.25);
        let r = w.step(actual_dt, |_| false).unwrap();
        let s = w.state();
        assert_eq!(s.stamp.version, version);
        assert_eq!(s.time, r.time_after);
        assert_eq!(s.cap_offset, r.cap_offset_after);
        assert_eq!(s.relative_pressure(), 0.);
        assert!(r.relative_transfer_l1 > 0.);
        assert!(r.advection_loss > 0. && r.strain_power > 0.);
        assert!(r.energy_after < r.energy_before && r.energy_after < previous);
        previous = r.energy_after;
        assert!(r.gcl_max < 1e-13 && r.quadrature_error_max < 1e-13);
        assert!(r.work_error.abs() < 1e-12 && r.true_residual < 1e-11);
        assert!(r.divergence_max < 1e-12);
        for d in 0..3 {
            assert!((r.momentum_after[d] - r.momentum_before[d]).abs() < 1e-11);
        }
        for (i, &x) in BOTTOM.iter().enumerate() {
            assert_eq!(s.physical_node(i).unwrap()[0], x);
        }
        for v in s.velocity {
            assert_eq!(v[0], 0.25);
            assert_eq!(v[1], 0.);
        }
    }
    assert!(
        w.state()
            .geometry
            .nodal_mass()
            .iter()
            .zip(old_mass)
            .any(|(&a, b)| (a - b).abs() > 0.001)
    );
    assert!((w.state().cap_offset - 0.125).abs() < 1e-15);
}
#[test]
fn cancellation_preserves_nonzero_accepted_geometry_and_recovers_cleanly() {
    for stage in [
        FixedBottomAleStage::BeforeGeometry,
        FixedBottomAleStage::Quadrature,
        FixedBottomAleStage::BeforeSolve,
        FixedBottomAleStage::Iteration,
        FixedBottomAleStage::BeforeAcceptance,
        FixedBottomAleStage::BeforePublish,
    ] {
        let mut w = flow(Default::default());
        w.step(0.025, |_| false).unwrap();
        let old = snapshot(&w);
        let mut calls = 0;
        let error = w
            .step(0.025, |s| {
                if s == stage {
                    calls += 1;
                    stage != FixedBottomAleStage::Quadrature || calls == 7
                } else {
                    false
                }
            })
            .unwrap_err();
        assert_eq!(error, FixedBottomAleError::Cancelled { stage });
        assert_eq!(snapshot(&w), old);
        let mut control = flow(Default::default());
        control.step(0.025, |_| false).unwrap();
        control.step(0.025, |_| false).unwrap();
        w.step(0.025, |_| false).unwrap();
        assert_eq!(snapshot(&w), snapshot(&control));
    }
}
#[test]
fn arithmetic_iteration_and_path_failures_preserve_state() {
    let mut w = flow(Default::default());
    w.step(0.025, |_| false).unwrap();
    let old = snapshot(&w);
    for dt in [
        0.,
        -1.,
        f64::NAN,
        f64::INFINITY,
        f64::from_bits(1),
        f64::MIN_POSITIVE,
        1.,
        1e16,
    ] {
        assert!(w.step(dt, |_| false).is_err());
        assert_eq!(snapshot(&w), old);
    }
    let mut w = flow(TranslatedViscousSettings {
        max_iterations: 1,
        ..Default::default()
    });
    let old = snapshot(&w);
    assert_eq!(
        w.step(0.025, |_| false).unwrap_err(),
        FixedBottomAleError::IterationLimit
    );
    assert_eq!(snapshot(&w), old);
}
#[test]
fn memory_budget_and_unsupported_carrier_reject_before_owner_creation() {
    let u = initial(0.05, false);
    let settings = TranslatedViscousSettings {
        memory_limit: 136351,
        ..Default::default()
    };
    assert!(matches!(
        FixedBottomAleFlow::new(geometry(0.05), &u, Default::default(), settings, 0),
        Err(FixedBottomAleError::Flow(
            TranslatedViscousError::BufferLimit { .. }
        ))
    ));
    let mut u = u;
    for v in &mut u {
        v[0] = -0.25;
    }
    assert!(matches!(
        FixedBottomAleFlow::new(
            geometry(0.05),
            &u,
            Default::default(),
            Default::default(),
            0
        ),
        Err(FixedBottomAleError::Flow(
            TranslatedViscousError::UnsupportedVelocity
        ))
    ));
}
#[test]
fn constants_remain_constant_and_zero_viscosity_still_transports() {
    let u = initial(0., true);
    let mut w =
        FixedBottomAleFlow::new(geometry(0.), &u, Default::default(), Default::default(), 0)
            .unwrap();
    for _ in 0..5 {
        let r = w.step(0.025, |_| false).unwrap();
        assert_eq!(w.state().velocity, u);
        assert_eq!(r.advection_loss, 0.);
        assert_eq!(r.strain_power, 0.);
        assert_eq!(r.iterations, 0);
    }
    let u = initial(0., false);
    let mut w =
        FixedBottomAleFlow::new(geometry(0.), &u, Default::default(), Default::default(), 0)
            .unwrap();
    let r = w.step(0.025, |_| false).unwrap();
    assert!(r.advection_loss > 0.);
    assert_eq!(r.strain_power, 0.);
    assert!(r.energy_after < r.energy_before);
}
#[test]
fn strict_acceptance_failure_preserves_the_accepted_pair() {
    let mut w = flow(TranslatedViscousSettings {
        momentum_limit: 1e-30,
        ..Default::default()
    });
    let old = snapshot(&w);
    assert_eq!(
        w.step(0.025, |_| false).unwrap_err(),
        FixedBottomAleError::AcceptanceFailure
    );
    assert_eq!(snapshot(&w), old);
}

#[test]
fn constrained_velocity_convex_bound_failure_is_retained_as_a_limit() {
    let frame = FittedHeightWorkspace::new(geometry(0.), Default::default(), |_| false).unwrap();
    let mut z = vec![0.; frame.plan().reduced_velocity_unknowns];
    // The positive nodal basis fixture uses the actual existing embedding,
    // not an invented pressure mode or arbitrary candidate donor field.
    for n in frame.nodes() {
        let row = frame.velocity_embedding(n.periodic_index, 0).unwrap();
        if row.weights[0] == 1.
            && let Some(col) = row.columns[0]
        {
            z[col] = 0.25;
        }
    }
    z[11 * 4 + 12] = 1.;
    let mut u = vec![[0.; 3]; frame.plan().periodic_nodes];
    frame.embed_velocity(&z, &mut u).unwrap();
    assert!(u.iter().all(|v| v[2] >= 0. && v[2] <= 1.));
    let mut w =
        FixedBottomAleFlow::new(geometry(0.), &u, Default::default(), Default::default(), 0)
            .unwrap();
    for _ in 0..20 {
        let dt = 0.025_f64.min((0.125 - w.state().cap_offset) / 0.25);
        let r = w.step(dt, |_| false).unwrap();
        assert!(r.work_error.abs() < 1e-12);
        assert!((r.momentum_after[2] - r.momentum_before[2]).abs() < 1e-11);
    }
    let minimum = w
        .state()
        .velocity
        .iter()
        .map(|v| v[2])
        .fold(f64::INFINITY, f64::min);
    assert!(
        minimum < -1e-4,
        "a positivity theorem must not be claimed for this constrained solve: {minimum}"
    );
}
