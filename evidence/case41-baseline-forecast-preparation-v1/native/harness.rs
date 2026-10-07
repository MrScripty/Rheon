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
const POLICY_ID: &str = "E2-working-affine-native-public-known-stored-inertia-v1";
const COORD: [usize; 7] = [0, 1, 2, 3, 4, 5, 2];
#[derive(Default)]
struct Control {
    seen: [usize; 12],
    target: Option<(usize, usize)>,
}
#[inline(never)]
fn observe(c: &mut Control, s: usize) -> bool {
    c.seen[s] += 1;
    c.target == Some((s, c.seen[s]))
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
struct FixedCase {
    index: usize,
    h: f64,
    acceleration: [f64; 3],
    q: [f64; 3],
    eta: [f64; 6],
    unknown: [f64; 22],
    expected_norm: f64,
    accepted: Snapshot,
}
include!("fixed_inputs.rs");
#[test]
fn scalar_affine_preflight() {
    let cases = [
        (-1., 1. + 2f64.powi(-52), 1. - 2f64.powi(-52)),
        (1., 2f64.powi(-54), 1.),
        (0., f64::MIN_POSITIVE, 0.5),
        (0., 2f64.powi(-600), 2f64.powi(-600)),
        (1., 1., 0.),
        (0., f64::MAX, 2.),
    ];
    for (case, (eta, time, alpha)) in cases.into_iter().enumerate() {
        let e = scalar::Scalar::from_f64(eta).unwrap();
        let t = scalar::Scalar::from_f64(time).unwrap();
        let a = scalar::Scalar::from_f64(alpha).unwrap();
        let result = t.mul(a).and_then(|v| e.add(v));
        print!(
            "{{\"event\":\"affine_probe\",\"case\":{case},\"eta\":{:?},\"time\":{:?},\"alpha\":{:?},",
            eta, time, alpha
        );
        match result {
            Ok(value) => {
                let (sig, exponent, negative) = value.parts();
                println!(
                    "\"parts\":[\"{sig}\",{exponent},{negative}],\"stored\":{:?},\"refused\":false}}",
                    value.to_f64().unwrap()
                );
            }
            Err(_) => println!("\"refused\":true}}"),
        }
    }
}

#[test]
fn case41_paired_layout_preflight() {
    println!(
        "{{\"event\":\"paired_layout\",\"candidate\":{:?},\"reference\":{:?},\"geometry\":{:?}}}",
        candidate::paired_layout(),
        reference::paired_layout(),
        FittedHeightWorkspace::research_paired_geometry_layout()
    );
}

#[test]
fn case41_forecast_input_layout_preflight() {
    candidate::forecast_input_layout();
}
#[test]
fn case41_baseline_and_frozen_forecast() {
    assert_eq!(
        std::env::var("RHEON_CASE41_TWO_OBSERVATION_ALLOW").unwrap(),
        "case41-baseline-and-single-frozen-forecast-reviewed-v1"
    );
    let mut chart = scalar::ChartWorkspace::new();
    candidate::case41_forecast_capture(&FIXED_CASES[0], &mut chart);
}
