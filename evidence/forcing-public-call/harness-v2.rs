#![allow(dead_code, clippy::needless_range_loop)]
use crate::*;
use candidate::CoupledDiscreteError;
#[path = "candidate-v2.rs"]
mod candidate;
#[path = "reference-v2.rs"]
mod reference;
#[path = "scalar.rs"]
mod scalar;
include!("inputs.rs");
const KNOWN: [usize; 7] = [2, 3, 12, 0, 1, 4, 13];
const UNKNOWN: [usize; 15] = [5, 6, 7, 8, 9, 10, 11, 14, 15, 16, 17, 18, 19, 20, 21];
const ROWS: [usize; 15] = [0, 2, 3, 4, 5, 6, 8, 10, 11, 12, 14, 16, 17, 18, 20];
const KERNEL_STACK_CEILING: usize = 2048;
const POLICY_ID: &str = "E1-public-fixed-pressure-forward-106-native-zero-refinement-v2";
const FORCES: [BodyForce; 1] = [BodyForce {
    value: [0.0625, -0.125, 0.03125],
    units: ForceUnits::Acceleration,
    region: None,
}];
#[derive(Default)]
struct Control {
    target: Option<usize>,
    seen: [usize; 12],
}
#[inline(never)]
fn observe(c: &mut Control, s: usize) -> bool {
    c.seen[s] += 1;
    c.target == Some(s)
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Snapshot {
    velocity: [[u64; 3]; 16],
    positions: [[u64; 2]; 19],
    mass: [u64; 16],
    pressure: [u64; 16],
    triangles: [[usize; 3]; 24],
    periodic: [usize; 19],
    time: u64,
    stamp: TranslatedViscousStamp,
}
fn snapshot_fields(
    g: &FittedHeightWorkspace,
    v: &[[f64; 3]],
    p: &[f64; 16],
    time: f64,
    stamp: TranslatedViscousStamp,
) -> Snapshot {
    Snapshot {
        velocity: std::array::from_fn(|i| v[i].map(f64::to_bits)),
        positions: std::array::from_fn(|i| g.nodes()[i].position.map(f64::to_bits)),
        mass: std::array::from_fn(|i| g.nodal_mass()[i].to_bits()),
        pressure: p.map(f64::to_bits),
        triangles: std::array::from_fn(|i| g.triangles()[i].nodes),
        periodic: std::array::from_fn(|i| g.nodes()[i].periodic_index),
        time: time.to_bits(),
        stamp,
    }
}
fn native_snapshot(o: &reference::CoupledDiscreteFlow) -> Snapshot {
    let s = o.state();
    snapshot_fields(
        s.geometry,
        s.velocity,
        s.pressure_coefficients,
        s.time,
        s.stamp,
    )
}
fn candidate_snapshot(o: &candidate::CoupledDiscreteFlow<'_>) -> Snapshot {
    let s = o.state();
    snapshot_fields(
        s.geometry,
        s.velocity,
        s.pressure_coefficients,
        s.time,
        s.stamp,
    )
}
fn verify_fields(s: &Snapshot, prefix: bool) {
    assert_eq!(
        s.velocity,
        if prefix {
            PREFIX_VELOCITY
        } else {
            INITIAL_VELOCITY
        }
        .map(|v| v.map(f64::to_bits))
    );
    assert_eq!(
        s.positions,
        if prefix {
            PREFIX_POSITIONS
        } else {
            INITIAL_POSITIONS
        }
        .map(|v| v.map(f64::to_bits))
    );
    assert_eq!(
        s.mass,
        if prefix { PREFIX_MASS } else { INITIAL_MASS }.map(f64::to_bits)
    );
    assert_eq!(
        s.pressure,
        if prefix {
            PREFIX_PRESSURE
        } else {
            INITIAL_PRESSURE
        }
        .map(f64::to_bits)
    );
    assert_eq!(
        s.time,
        if prefix { PREFIX_TIME } else { INITIAL_TIME }.to_bits()
    );
    assert_eq!(
        s.stamp,
        TranslatedViscousStamp {
            id: 131,
            version: if prefix { 1 } else { 0 }
        }
    );
}
#[inline(never)]
fn verified_prefix() -> reference::CoupledDiscreteFlow {
    let mut o = reference::CoupledDiscreteFlow::new_forced_extruded(
        FittedHeightGeometry {
            cap: &[[0., 1.], [0.5, 1.25], [1., 1.]],
            bottom_x: &[0., 0.5, 1.],
            extrusion_width: 1.,
            density: 3.,
            dynamic_viscosity: 0.05,
        },
        &INITIAL_VELOCITY,
        Default::default(),
        Default::default(),
        131,
    )
    .unwrap();
    verify_fields(&native_snapshot(&o), false);
    println!("{{\"event\":\"public_phase\",\"phase\":\"original_prefix\"}}");
    let report = reference::public_call(&mut o, &mut Control::default()).unwrap();
    verify_fields(&native_snapshot(&o), true);
    assert_eq!(
        o.accepted_coefficients().map(f64::to_bits),
        ACCEPTED_Z.map(f64::to_bits)
    );
    println!(
        "{{\"event\":\"verified_prefix\",\"version\":1,\"time\":{:?},\"report\":{:?}}}",
        o.state().time,
        format!("{report:?}")
    );
    o
}
#[test]
fn research_public_baseline_conformance() {
    let original = verified_prefix();
    let frozen = native_snapshot(&original);
    let mut native = original.clone_native();
    assert_eq!(native_snapshot(&native), frozen);
    println!("{{\"event\":\"public_phase\",\"phase\":\"native_fixed_call\"}}");
    let result = reference::public_call(&mut native, &mut Control::default());
    assert!(matches!(
        result,
        Err(reference::CoupledDiscreteError::IterationLimit)
    ));
    assert_eq!(native_snapshot(&native), frozen);
    drop(native);
    let mut adapted = original.clone_candidate();
    assert_eq!(candidate_snapshot(&adapted), frozen);
    println!("{{\"event\":\"public_phase\",\"phase\":\"candidate_native_chart_disabled\"}}");
    let result = candidate::public_call(&mut adapted, &mut Control::default());
    assert!(matches!(
        result,
        Err(candidate::CoupledDiscreteError::IterationLimit)
    ));
    assert_eq!(candidate_snapshot(&adapted), frozen);
    assert_eq!(native_snapshot(&original), frozen);
    println!(
        "{{\"event\":\"baseline_conformance\",\"pass\":true,\"candidate_E1_executed\":false,\"fixed_call_native_refusal_preserved\":true}}"
    );
}
#[test]
fn research_public_candidate_and_controls() {
    assert_eq!(
        std::env::var("RHEON_PUBLIC_CALL_ALLOW").unwrap(),
        POLICY_ID,
        "external new-binary preflight must pass"
    );
    let original = verified_prefix();
    let frozen = native_snapshot(&original);
    let mut chart = scalar::ChartWorkspace::new();
    let mut owner = original.clone_candidate();
    assert_eq!(candidate_snapshot(&owner), frozen);
    owner.set_chart(&mut chart);
    println!("{{\"event\":\"public_phase\",\"phase\":\"E1_fixed_public_call\"}}");
    let mut control = Control::default();
    let result = candidate::public_call(&mut owner, &mut control);
    println!(
        "{{\"event\":\"E1_public_result\",\"result\":{:?},\"seen\":{:?},\"accepted\":{:?}}}",
        format!("{result:?}"),
        control.seen,
        format!("{:?}", candidate_snapshot(&owner))
    );
    let expected = candidate_snapshot(&owner);
    let succeeded = result.is_ok();
    if succeeded {
        assert_eq!(expected.stamp.version, 2);
        assert_eq!(expected.time, (2. * H).to_bits());
    } else {
        assert_eq!(expected, frozen);
    }
    assert_eq!(native_snapshot(&original), frozen);
    drop(owner);
    for stage in 0..12 {
        if control.seen[stage] == 0 {
            println!(
                "{{\"event\":\"cancellation_control\",\"stage\":{stage},\"status\":\"NOT_REACHED_BY_FIXED_CALL\"}}"
            );
            continue;
        }
        let mut retry = original.clone_candidate();
        retry.set_chart(&mut chart);
        let mut cancel = Control {
            target: Some(stage),
            ..Default::default()
        };
        let cancelled = candidate::public_call(&mut retry, &mut cancel);
        assert!(matches!(
            cancelled,
            Err(candidate::CoupledDiscreteError::Cancelled { .. })
        ));
        assert_eq!(candidate_snapshot(&retry), frozen);
        let retried = candidate::public_call(&mut retry, &mut Control::default());
        assert_eq!(retried.is_ok(), succeeded);
        assert_eq!(format!("{retried:?}"), format!("{result:?}"));
        assert_eq!(candidate_snapshot(&retry), expected);
        assert_eq!(native_snapshot(&original), frozen);
        println!(
            "{{\"event\":\"cancellation_control\",\"stage\":{stage},\"status\":\"CANCEL_PRESERVES_AND_RETRY_REPRODUCES\"}}"
        );
    }
    println!(
        "{{\"event\":\"public_experiment_complete\",\"new_cases\":0,\"parameters_changed\":false,\"original_unchanged\":true}}"
    );
}
#[test]
fn research_public_scalar_preflight() {
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
        reference::layout().0,
        reference::layout().1,
        reference::layout().2
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
fn research_public_type_layout() {
    println!(
        "{{\"event\":\"public_type_layout\",\"native\":{:?},\"candidate\":{:?},\"snapshot_bytes\":{},\"control_bytes\":{},\"workspace_bytes\":{}}}",
        reference::layout(),
        candidate::layout(),
        std::mem::size_of::<Snapshot>(),
        std::mem::size_of::<Control>(),
        std::mem::size_of::<scalar::ChartWorkspace>()
    );
}
