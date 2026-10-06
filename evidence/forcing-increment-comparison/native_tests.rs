include!("native_inputs.rs");
const POLICY_ID: &str = "E1-106-nearest-even-native-geometry-zero-refinement-v1";
const KERNEL_STACK_CEILING: usize = 2048;
fn same<const L: usize>(a: &[f64; L], b: &[f64; L]) {
    assert_eq!(a.map(f64::to_bits), b.map(f64::to_bits));
}
fn print_equation(stage: &str, e: &Equation, r: &[[[f64; 22]; 2]; 16]) {
    println!(
        "{{\"event\":\"equation\",\"stage\":\"{stage}\",\"r\":{:?},\"start_z\":{:?},\"end_z\":{:?},\"end_q\":{:?},\"end_eta\":{:?},\"start_mass\":{:?},\"end_mass\":{:?},\"end_velocity\":{:?},\"end_force\":{:?},\"end_b\":{:?},\"end_d\":{:?},\"pairs\":{:?},\"plus\":{:?},\"minus\":{:?},\"rate\":{:?},\"direct_rate\":{:?},\"norm\":{:?},\"newton_pass\":{},\"published\":false}}",
        r,
        e.start.z,
        e.end.z,
        e.end.q,
        e.end.eta,
        e.start.mass,
        e.end.mass,
        e.end.u,
        e.end.force,
        e.end.b,
        e.end.d,
        &e.end.pairs[..e.end.faces],
        &e.integral.plus[..e.end.faces],
        &e.integral.minus[..e.end.faces],
        e.rate,
        e.direct_rate,
        norm(&e.rate).unwrap(),
        norm(&e.rate).unwrap() <= NEWTON
    );
}
fn qualification(stage: &str, coarse: &Equation, fine: &Equation) {
    match qualify(
        fine,
        coarse,
        H,
        UNKNOWN0,
        TranslatedViscousStamp {
            id: 131,
            version: 1,
        },
        TranslatedViscousStamp {
            id: 131,
            version: 2,
        },
        2. * H,
        7,
        50,
        [0.0625, -0.125, 0.03125],
    ) {
        Ok(r) => println!(
            "{{\"event\":\"isolated_planar_gates\",\"stage\":\"{stage}\",\"pass\":true,\"energy_before\":{:?},\"energy_after\":{:?},\"be\":{:?},\"mix\":{:?},\"viscous\":{:?},\"pressure_work\":{:?},\"gcl_work\":{:?},\"residual_work\":{:?},\"ledger_error\":{:?},\"allowance\":{:?},\"gcl_max\":{:?},\"quadrature\":{:?},\"constraints\":{:?},\"finite_norm\":{:?},\"direct_norm\":{:?},\"no_third_or_owner_advance\":true}}",
            r.energy_before,
            r.energy_after,
            r.backward_euler_loss,
            r.mixing_loss,
            r.viscous_loss,
            r.pressure_work,
            r.gcl_work,
            r.residual_work,
            r.ledger_error,
            r.work_allowance,
            r.gcl_max,
            r.quadrature_error,
            r.full_constraints,
            r.finite_momentum_rate_norm,
            r.direct_momentum_rate_norm
        ),
        Err(err) => println!(
            "{{\"event\":\"isolated_planar_gates\",\"stage\":\"{stage}\",\"pass\":false,\"error\":\"{err:?}\",\"no_third_or_owner_advance\":true}}"
        ),
    }
}
#[test]
fn research_scalar_preflight() {
    let mut scratch = scalar::ChartWorkspace::new();
    let groups = scratch
        .conformance()
        .expect("genuine scalar conformance failure; comparison forbidden");
    let extra = std::mem::size_of::<scalar::ChartWorkspace>()
        + std::mem::size_of::<Option<&mut scalar::ChartWorkspace>>()
        + KERNEL_STACK_CEILING;
    assert_eq!(std::mem::size_of::<scalar::Scalar>(), 32);
    assert!(extra <= 65536);
    println!(
        "{{\"event\":\"preflight_layout\",\"scalar_bytes\":{},\"workspace_bytes\":{},\"borrow_descriptor_bytes\":{},\"kernel_stack_ceiling\":{},\"additional_bytes_with_ceiling\":{},\"baseline_forced_nominal_bytes\":{},\"baseline_step_stack_bytes\":{},\"baseline_third_stack_bytes\":{},\"baseline_force_stack_bytes\":{},\"conformance_groups\":{groups},\"policy_id\":\"{POLICY_ID}\",\"e1_not_executed\":true}}",
        std::mem::size_of::<scalar::Scalar>(),
        std::mem::size_of::<scalar::ChartWorkspace>(),
        std::mem::size_of::<Option<&mut scalar::ChartWorkspace>>(),
        KERNEL_STACK_CEILING,
        extra,
        crate::CoupledDiscreteFlow::nominal_forced_extruded_bytes().unwrap(),
        STEP_STACK_BYTES,
        THIRD_STACK_BYTES,
        FORCE_STACK_BYTES
    );
    let pairs = [
        (1., 2f64.powi(-106)),
        (1., 2f64.powi(-105)),
        (1.5, 3.5),
        (
            f64::MIN_POSITIVE,
            f64::from_bits(f64::MIN_POSITIVE.to_bits() + 1),
        ),
        (f64::MAX, 2.),
        (f64::MIN_POSITIVE, 0.5),
        (1., -1.),
        (0., -0.),
        (-0., -0.),
        (4., 3.),
        (2f64.powi(-600), 2f64.powi(-600)),
        (2f64.powi(800), 2f64.powi(-800)),
    ];
    for (i, (a, b)) in pairs.into_iter().enumerate() {
        let a = scalar::Scalar::from_f64(a).unwrap();
        let b = scalar::Scalar::from_f64(b).unwrap();
        for (name, result) in [("add", a.add(b)), ("mul", a.mul(b)), ("div", a.div(b))] {
            let (asig, ae, an) = a.parts();
            let (bsig, be, bn) = b.parts();
            match result {
                Ok(x) => {
                    let (sig, e, n) = x.parts();
                    println!(
                        "{{\"event\":\"scalar_probe\",\"case\":{i},\"op\":\"{name}\",\"a\":[\"{asig}\",{ae},{an}],\"b\":[\"{bsig}\",{be},{bn}],\"out\":[\"{sig}\",{e},{n}],\"refused\":false}}"
                    );
                }
                Err(_) => println!(
                    "{{\"event\":\"scalar_probe\",\"case\":{i},\"op\":\"{name}\",\"a\":[\"{asig}\",{ae},{an}],\"b\":[\"{bsig}\",{be},{bn}],\"refused\":true}}"
                ),
            }
        }
    }
}
#[test]
fn research_pinned_comparison() {
    assert_eq!(
        std::env::var("RHEON_INCREMENT_COMPARISON_ALLOW").unwrap(),
        POLICY_ID,
        "external scalar/stack receipt must pass before E1"
    );
    let mut scratch = scalar::ChartWorkspace::new();
    scratch.set_accepted(Z0);
    let cap = [[0., 1.], [0.5, 1.25], [1., 1.]];
    let geometry = FittedHeightWorkspace::new(
        FittedHeightGeometry {
            cap: &cap,
            bottom_x: &BOTTOM,
            extrusion_width: 1.,
            density: 3.,
            dynamic_viscosity: 0.05,
        },
        Default::default(),
        |_| false,
    )
    .unwrap();
    let mut work = Work {
        geometry,
        r: [[[0.; 22]; 2]; 16],
        diagnostic: [Default::default(); 16],
        pressure: [0.; 16],
        research: None,
    };
    for i in 0..16 {
        for d in 0..2 {
            let e = work.geometry.velocity_embedding(i, d).unwrap();
            for k in 0..2 {
                if let Some(j) = e.columns[k] {
                    work.r[i][d][j] = add(work.r[i][d][j], e.weights[k]).unwrap();
                }
            }
        }
    }
    for i in 0..16 {
        for d in 0..2 {
            same(&work.r[i][d], &R_EXPECT[i][d]);
        }
    }
    let baseline = work
        .equation(
            Q0,
            ETA0,
            &UNKNOWN0,
            H,
            16,
            &OLD,
            &M0,
            [0.0625, -0.125, 0.03125],
            &mut |_| false,
        )
        .unwrap();
    let baseline_fine = work
        .equation(
            Q0,
            ETA0,
            &UNKNOWN0,
            H,
            32,
            &OLD,
            &M0,
            [0.0625, -0.125, 0.03125],
            &mut |_| false,
        )
        .unwrap();
    same(&baseline.start.z, &Z0);
    same(&baseline.end.z, &Z1);
    same(&baseline.end.mass, &END_M);
    same(&baseline.rate, &RATE);
    same(&baseline.direct_rate, &DIRECT);
    assert_eq!(
        norm(&baseline.rate).unwrap().to_bits(),
        EXPECTED_NORM.to_bits()
    );
    for i in 0..16 {
        same(&baseline.end.u[i], &END_U[i]);
        same(&baseline.end.force[i], &FORCE[i]);
    }
    for i in 0..24 {
        same(&baseline.end.d[i], &D_EXPECT[i]);
    }
    for i in 0..16 {
        same(&baseline.end.b[i], &B_EXPECT[i]);
    }
    assert_eq!(&baseline.end.pairs[..baseline.end.faces], &PAIRS);
    for i in 0..40 {
        assert_eq!(baseline.integral.plus[i].to_bits(), PLUS[i].to_bits());
        assert_eq!(baseline.integral.minus[i].to_bits(), MINUS[i].to_bits());
    }
    assert!(norm(&baseline.rate).unwrap() > NEWTON);
    print_equation("B0-16", &baseline, &work.r);
    print_equation("B0-32", &baseline_fine, &work.r);
    qualification("B0", &baseline, &baseline_fine);
    work.research = Some(&mut scratch);
    let candidate = match work.equation(
        Q0,
        ETA0,
        &UNKNOWN0,
        H,
        16,
        &OLD,
        &M0,
        [0.0625, -0.125, 0.03125],
        &mut |_| false,
    ) {
        Ok(x) => x,
        Err(err) => {
            println!(
                "{{\"event\":\"E1_failure\",\"order\":16,\"error\":\"{err:?}\",\"published\":false}}"
            );
            return;
        }
    };
    print_equation("E1-16", &candidate, &work.r);
    let fine = match work.equation(
        Q0,
        ETA0,
        &UNKNOWN0,
        H,
        32,
        &OLD,
        &M0,
        [0.0625, -0.125, 0.03125],
        &mut |_| false,
    ) {
        Ok(x) => x,
        Err(err) => {
            println!(
                "{{\"event\":\"E1_failure\",\"order\":32,\"error\":\"{err:?}\",\"published\":false}}"
            );
            return;
        }
    };
    print_equation("E1-32", &fine, &work.r);
    qualification("E1", &candidate, &fine);
    println!(
        "{{\"event\":\"comparison_complete\",\"chart_evaluations\":{},\"owner_advances\":0,\"searches\":0,\"policy_id\":\"{POLICY_ID}\"}}",
        work.research.as_ref().unwrap().evaluations()
    );
}
