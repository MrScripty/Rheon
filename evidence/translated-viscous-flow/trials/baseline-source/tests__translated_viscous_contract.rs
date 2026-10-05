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
fn flow(mu: f64, settings: TranslatedViscousSettings) -> TranslatedViscousFlow {
    TranslatedViscousFlow::new(
        geometry(mu),
        &initial(mu, false),
        FittedHeightSettings::default(),
        settings,
        17,
    )
    .unwrap()
}
fn snapshot(w: &TranslatedViscousFlow) -> (Vec<[u64; 3]>, u64, u64, TranslatedViscousStamp) {
    let s = w.state();
    (
        s.velocity.iter().map(|v| v.map(f64::to_bits)).collect(),
        s.time.to_bits(),
        s.offset.to_bits(),
        s.stamp,
    )
}
#[test]
fn repeated_translation_and_viscous_decay_publish_one_state() {
    let mut w = flow(0.05, TranslatedViscousSettings::default());
    assert_eq!(
        w.allocated_bytes(),
        TranslatedViscousFlow::nominal_bytes(4).unwrap()
    );
    assert_eq!(w.allocated_bytes(), 59184);
    let accepted_mass: f64 = w.state().template.nodal_mass().iter().sum();
    assert!((accepted_mass - 3.5625).abs() < 1e-15);
    let mut previous = f64::INFINITY;
    for version in 1..=20 {
        let r = w.step(0.025, |_| false).unwrap();
        let s = w.state();
        assert_eq!(s.stamp.version, version);
        assert_eq!(s.time, r.time_after);
        assert_eq!(s.offset, r.offset_after);
        assert_eq!(s.relative_pressure(), 0.);
        assert_eq!(r.liquid_mass.to_bits(), accepted_mass.to_bits());
        assert!(r.energy_after < r.energy_before);
        assert!(r.energy_after < previous);
        previous = r.energy_after;
        assert!(r.true_residual < 1e-11);
        assert!(r.work_identity_error.abs() < 1e-12);
        assert!(r.divergence_max < 1e-12);
        assert!(r.strain_power > 0.);
        for d in 0..3 {
            assert!((r.momentum_after[d] - r.momentum_before[d]).abs() < 1e-11);
        }
        for v in s.velocity {
            assert_eq!(v[0], 0.25);
            assert_eq!(v[1], 0.);
        }
        for (i, n) in s.template.nodes().iter().enumerate() {
            assert_eq!(
                s.physical_node(i),
                Some([n.position[0] + s.offset, n.position[1]])
            );
        }
    }
    assert!((w.state().offset - 0.125).abs() < 1e-15);
}
#[test]
fn each_cancellation_preserves_velocity_geometry_pressure_clock_and_identity() {
    for stage in [
        TranslatedViscousStage::BeforeSolve,
        TranslatedViscousStage::Iteration,
        TranslatedViscousStage::BeforeAcceptance,
        TranslatedViscousStage::BeforePublish,
    ] {
        let mut w = flow(0.05, TranslatedViscousSettings::default());
        w.step(0.025, |_| false).unwrap();
        let old = snapshot(&w);
        assert_eq!(
            w.step(0.025, |s| s == stage).unwrap_err(),
            TranslatedViscousError::Cancelled { stage }
        );
        assert_eq!(snapshot(&w), old);
        assert_eq!(w.state().relative_pressure(), 0.);
        // A rejected candidate does not poison a later successful attempt.
        let mut control = flow(0.05, TranslatedViscousSettings::default());
        control.step(0.025, |_| false).unwrap();
        control.step(0.025, |_| false).unwrap();
        w.step(0.025, |_| false).unwrap();
        assert_eq!(snapshot(&w), snapshot(&control));
    }
}
#[test]
fn arithmetic_and_iteration_failures_preserve_accepted_state() {
    let mut w = flow(0.05, TranslatedViscousSettings::default());
    let old = snapshot(&w);
    for dt in [
        0.,
        -1.,
        f64::NAN,
        f64::INFINITY,
        f64::from_bits(1),
        f64::MIN_POSITIVE,
    ] {
        assert!(w.step(dt, |_| false).is_err());
        assert_eq!(snapshot(&w), old);
    }
    let mut w = flow(
        0.05,
        TranslatedViscousSettings {
            max_iterations: 1,
            ..Default::default()
        },
    );
    let old = snapshot(&w);
    assert_eq!(
        w.step(0.1, |_| false).unwrap_err(),
        TranslatedViscousError::IterationLimit
    );
    assert_eq!(snapshot(&w), old);
}
#[test]
fn inviscid_and_constant_fields_are_preserved_without_solver_drift() {
    for (mu, constant) in [(0., false), (0.05, true)] {
        let u = initial(mu, constant);
        let mut w = TranslatedViscousFlow::new(
            geometry(mu),
            &u,
            FittedHeightSettings::default(),
            Default::default(),
            0,
        )
        .unwrap();
        for _ in 0..10 {
            let r = w.step(0.05, |_| false).unwrap();
            assert_eq!(r.iterations, 0);
            assert_eq!(r.energy_before, r.energy_after);
            assert_eq!(r.strain_power, 0.);
            assert_eq!(w.state().velocity, &u);
        }
    }
}
#[test]
fn unsupported_xy_and_trace_and_insufficient_memory_are_rejected() {
    let u = initial(0.05, false);
    let settings = TranslatedViscousSettings {
        memory_limit: 59183,
        ..Default::default()
    };
    assert!(matches!(
        TranslatedViscousFlow::new(
            geometry(0.05),
            &u,
            FittedHeightSettings::default(),
            settings,
            0
        ),
        Err(TranslatedViscousError::BufferLimit { .. })
    ));
    for d in [0, 1, 2] {
        let mut v = u.clone();
        v[1][d] += 0.01;
        // For z, explicitly perturb a constrained row, not a free degree.
        if d == 2 {
            let f =
                FittedHeightWorkspace::new(geometry(0.05), Default::default(), |_| false).unwrap();
            let i = (0..v.len())
                .find(|&i| f.velocity_embedding(i, 2).unwrap().weights[0] == 0.5)
                .unwrap();
            v = u.clone();
            v[i][2] += 0.01;
        }
        assert!(matches!(
            TranslatedViscousFlow::new(
                geometry(0.05),
                &v,
                FittedHeightSettings::default(),
                Default::default(),
                0
            ),
            Err(TranslatedViscousError::UnsupportedVelocity)
        ));
    }
}
