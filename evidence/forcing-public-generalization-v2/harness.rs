#![allow(dead_code, clippy::needless_range_loop)]
use crate::*;
use candidate::CoupledDiscreteError;
#[path = "candidate.rs"]
mod candidate;
#[path = "reference.rs"]
mod reference;
#[path = "scalar.rs"]
mod scalar;
const KNOWN: [usize; 7] = [2, 3, 12, 0, 1, 4, 13];
const UNKNOWN: [usize; 15] = [5, 6, 7, 8, 9, 10, 11, 14, 15, 16, 17, 18, 19, 20, 21];
const ROWS: [usize; 15] = [0, 2, 3, 4, 5, 6, 8, 10, 11, 12, 14, 16, 17, 18, 20];
const KERNEL_STACK_CEILING: usize = 2048;
const POLICY_ID: &str = "E1-five-terminal-refusals-public-66KiB-v2";
#[derive(Default)]
struct Control {
    seen: [usize; 12],
}
#[inline(never)]
fn observe(c: &mut Control, s: usize) -> bool {
    c.seen[s] += 1;
    false
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

struct Case {
    index: usize,
    id: &'static str,
    h: f64,
    acceleration: [f64; 3],
    initial_velocity: [[f64; 3]; 16],
    initial: Snapshot,
    prefix: Snapshot,
}
include!("inputs.rs");
fn forces(case: &Case) -> [BodyForce; 1] {
    [BodyForce {
        value: case.acceleration,
        units: ForceUnits::Acceleration,
        region: None,
    }]
}
#[inline(never)]
fn verified_prefix(case: &Case) -> reference::CoupledDiscreteFlow {
    let mut original = reference::CoupledDiscreteFlow::new_forced_extruded(
        FittedHeightGeometry {
            cap: &[[0., 1.], [0.5, 1.25], [1., 1.]],
            bottom_x: &[0., 0.5, 1.],
            extrusion_width: 1.,
            density: 3.,
            dynamic_viscosity: 0.05,
        },
        &case.initial_velocity,
        Default::default(),
        Default::default(),
        131,
    )
    .unwrap();
    assert_eq!(native_snapshot(&original), case.initial);
    let load = forces(case);
    assert!(
        case.prefix.stamp.version <= 1,
        "fixed bounded historical prefix reconstruction"
    );
    for _ in 0..case.prefix.stamp.version {
        let prefix_report =
            reference::public_call(&mut original, &mut Control::default(), case.h, &load).unwrap();
        println!(
            "{{\"event\":\"historical_prefix_report\",\"case\":{},\"report\":{:?}}}",
            case.index,
            format!("{prefix_report:?}")
        );
    }
    assert_eq!(native_snapshot(&original), case.prefix);
    println!(
        "{{\"event\":\"verified_prefix\",\"case\":{},\"accepted\":{:?}}}",
        case.index,
        format!("{:?}", native_snapshot(&original))
    );
    original
}
#[test]
fn research_generalization_baseline() {
    for case in &CASES {
        println!(
            "{{\"event\":\"baseline_case\",\"case\":{},\"id\":{:?}}}",
            case.index, case.id
        );
        let original = verified_prefix(case);
        let load = forces(case);
        let mut native = original.clone_native();
        let native_result =
            reference::public_call(&mut native, &mut Control::default(), case.h, &load);
        assert!(matches!(
            native_result,
            Err(reference::CoupledDiscreteError::IterationLimit)
        ));
        assert_eq!(native_snapshot(&native), case.prefix);
        drop(native);
        let mut disabled = original.clone_candidate();
        let disabled_result =
            candidate::public_call(&mut disabled, &mut Control::default(), case.h, &load);
        assert_eq!(format!("{native_result:?}"), format!("{disabled_result:?}"));
        assert_eq!(candidate_snapshot(&disabled), case.prefix);
        assert_eq!(native_snapshot(&original), case.prefix);
        println!(
            "{{\"event\":\"baseline_conformance\",\"case\":{},\"native_result\":{:?},\"disabled_result\":{:?},\"generalization_executed\":false}}",
            case.index,
            format!("{native_result:?}"),
            format!("{disabled_result:?}")
        );
    }
}
#[test]
fn research_generalization_candidate_one() {
    assert_eq!(
        std::env::var("RHEON_PUBLIC_CALL_ALLOW").unwrap(),
        POLICY_ID,
        "new binary preflight must pass"
    );
    let index: usize = std::env::var("RHEON_CASE_INDEX").unwrap().parse().unwrap();
    let case = &CASES[index];
    assert_eq!(case.index, index);
    println!(
        "{{\"event\":\"generalization_case\",\"case\":{},\"id\":{:?}}}",
        case.index, case.id
    );
    let original = verified_prefix(case);
    let load = forces(case);
    let mut chart = scalar::ChartWorkspace::new();
    let mut owner = original.clone_candidate();
    assert_eq!(candidate_snapshot(&owner), case.prefix);
    owner.set_chart(&mut chart);
    println!(
        "{{\"event\":\"main_call_begin\",\"case\":{},\"allocated_bytes\":{}}}",
        case.index,
        owner.allocated_bytes()
    );
    let mut control = Control::default();
    let result = candidate::public_call(&mut owner, &mut control, case.h, &load);
    let accepted = candidate_snapshot(&owner);
    println!(
        "{{\"event\":\"generalization_public_result\",\"case\":{},\"result\":{:?},\"seen\":{:?},\"accepted\":{:?}}}",
        case.index,
        format!("{result:?}"),
        control.seen,
        format!("{accepted:?}")
    );
    match &result {
        Ok(report) => {
            assert_eq!(
                accepted.stamp,
                TranslatedViscousStamp {
                    id: 131,
                    version: case.prefix.stamp.version + 1
                }
            );
            let old_time = f64::from_bits(case.prefix.time);
            assert_eq!(accepted.time, (old_time + case.h).to_bits());
            assert_eq!(report.step.planar.after, accepted.stamp);
            assert_eq!(report.step.planar.time_after.to_bits(), accepted.time);
            assert!(control.seen[10] > 0);
        }
        Err(_) => assert_eq!(accepted, case.prefix),
    }
    assert_eq!(native_snapshot(&original), case.prefix);
    println!(
        "{{\"event\":\"generalization_complete\",\"case\":{},\"original_unchanged\":true,\"main_calls\":1,\"continued_trajectory\":false,\"parameter_search\":false}}",
        case.index
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
    assert!(extra <= 67584);
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
