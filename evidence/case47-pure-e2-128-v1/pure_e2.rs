// Borrow accepted fields for external bitwise comparison; retain no snapshot.
#[inline(never)]
fn pure_e2_bits(owner: &candidate::CoupledDiscreteFlow<'_>, label: &str, step: usize) {
    let s = owner.state();
    print!(
        "{{\"event\":\"accepted_bits\",\"label\":{label:?},\"step\":{step},\"time\":{},\"stamp\":[{},{}],\"velocity\":[",
        s.time.to_bits(),
        s.stamp.id,
        s.stamp.version
    );
    for i in 0..16 {
        if i > 0 {
            print!(",");
        }
        print!("{:?}", s.velocity[i].map(f64::to_bits));
    }
    print!("],\"positions\":[");
    for i in 0..19 {
        if i > 0 {
            print!(",");
        }
        print!("{:?}", s.geometry.nodes()[i].position.map(f64::to_bits));
    }
    print!("],\"mass\":[");
    for i in 0..16 {
        if i > 0 {
            print!(",");
        }
        print!("{}", s.geometry.nodal_mass()[i].to_bits());
    }
    print!("],\"pressure\":[");
    for i in 0..16 {
        if i > 0 {
            print!(",");
        }
        print!("{}", s.pressure_coefficients[i].to_bits());
    }
    print!("],\"triangles\":[");
    for i in 0..24 {
        if i > 0 {
            print!(",");
        }
        print!("{:?}", s.geometry.triangles()[i].nodes);
    }
    print!("],\"periodic\":[");
    for i in 0..19 {
        if i > 0 {
            print!(",");
        }
        print!("{}", s.geometry.nodes()[i].periodic_index);
    }
    println!("]}}");
}

#[inline(never)]
fn pure_e2_initial(owner: &candidate::CoupledDiscreteFlow<'_>) {
    let s = owner.state();
    let expected = &CASES[1].initial;
    for i in 0..16 {
        assert_eq!(s.velocity[i].map(f64::to_bits), expected.velocity[i]);
        assert_eq!(s.geometry.nodal_mass()[i].to_bits(), expected.mass[i]);
        assert_eq!(s.pressure_coefficients[i].to_bits(), expected.pressure[i]);
    }
    for i in 0..19 {
        assert_eq!(
            s.geometry.nodes()[i].position.map(f64::to_bits),
            expected.positions[i]
        );
        assert_eq!(s.geometry.nodes()[i].periodic_index, expected.periodic[i]);
    }
    for i in 0..24 {
        assert_eq!(s.geometry.triangles()[i].nodes, expected.triangles[i]);
    }
    assert_eq!(s.time.to_bits(), expected.time);
    assert_eq!(s.stamp, expected.stamp);
}

#[test]
fn case47_pure_e2_original_128_or_cancel() {
    assert_eq!(
        std::env::var("RHEON_CASE47_PURE_E2_ALLOW").unwrap(),
        "case47-pure-E2-original128-first-visit-cancel-v1"
    );
    let stage = std::env::var("RHEON_CASE47_CANCEL_STAGE")
        .ok()
        .map(|s| s.parse::<usize>().unwrap());
    if let Some(s) = stage {
        assert!(s <= 10);
    }
    let case = &TRAJECTORIES[47];
    assert_eq!(case.input, 2);
    assert_eq!(case.steps, 128);
    assert_eq!(case.h.to_bits(), 0.00078125f64.to_bits());
    assert_eq!(case.load, "reversed");
    let input = &CONSTRUCTORS[case.input];
    assert_eq!(input.kind, "pressure_state");
    assert_eq!(input.field, "nonconstant");
    let load = [BodyForce {
        value: [-0.0625, 0.125, -0.03125],
        units: ForceUnits::Acceleration,
        region: None,
    }];
    let mut chart = scalar::ChartWorkspace::new();
    let mut owner = candidate::public_constructor(&input.velocity).unwrap();
    owner.set_chart(&mut chart);
    owner.select_working_affine(true);
    assert_eq!(owner.allocated_bytes(), 542352);
    pure_e2_initial(&owner);
    println!(
        "{{\"event\":\"pure_e2_begin\",\"case\":47,\"expected_steps\":128,\"h\":{:?},\"nominal_end_time\":0.1,\"cancel_stage\":{},\"E2_before_first_advance\":true,\"retries\":0}}",
        case.h,
        stage.map_or(-1, |s| s as i32)
    );
    pure_e2_bits(&owner, "constructor", 0);
    emit(&owner, input.kind, input.field, case.h, case.load, None);
    let end = if stage.is_some() { 26 } else { 128 };
    let mut accepted = 0;
    let mut expected_time = 0.;
    for attempted in 1..=end {
        pure_e2_bits(&owner, "before", attempted);
        let mut control = Control {
            target: if attempted == 26 {
                stage.map(|s| (s, 1))
            } else {
                None
            },
            ..Default::default()
        };
        let result = candidate::public_call(&mut owner, &mut control, case.h, &load);
        println!(
            "{{\"event\":\"pure_e2_result\",\"step\":{attempted},\"result\":{:?},\"seen\":{:?}}}",
            format!("{result:?}"),
            control.seen
        );
        pure_e2_bits(&owner, "after", attempted);
        match result {
            Ok(report) => {
                accepted += 1;
                expected_time += case.h;
                assert_eq!(owner.state().stamp.version, accepted);
                assert_eq!(owner.state().time.to_bits(), expected_time.to_bits());
                emit(
                    &owner,
                    input.kind,
                    input.field,
                    case.h,
                    case.load,
                    Some(&report),
                );
                if attempted == 26 && stage.is_some() {
                    println!(
                        "{{\"event\":\"pure_e2_terminal\",\"status\":\"UNEXPECTED_PUBLICATION\",\"accepted_steps\":{accepted},\"attempted_step\":{attempted},\"retries\":0}}"
                    );
                    return;
                }
            }
            Err(error) => {
                let status = if attempted == 26
                    && stage.is_some()
                    && matches!(error, candidate::CoupledDiscreteError::Cancelled { .. })
                {
                    "CANCELLED"
                } else {
                    "REFUSED"
                };
                println!(
                    "{{\"event\":\"pure_e2_terminal\",\"status\":{status:?},\"accepted_steps\":{accepted},\"attempted_step\":{attempted},\"error\":{:?},\"retries\":0}}",
                    format!("{error:?}")
                );
                return;
            }
        }
    }
    println!(
        "{{\"event\":\"pure_e2_terminal\",\"status\":\"COMPLETE\",\"accepted_steps\":{accepted},\"expected_steps\":128,\"actual_time\":{:?},\"retries\":0}}",
        owner.state().time
    );
}
