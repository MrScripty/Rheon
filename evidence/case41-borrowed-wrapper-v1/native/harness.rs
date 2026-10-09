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

struct Constructor {
    kind: &'static str,
    field: &'static str,
    velocity: [[f64; 3]; 16],
}
struct Trajectory {
    index: usize,
    input: usize,
    load: &'static str,
    h: f64,
    steps: usize,
}
include!("trajectory_inputs.rs");
#[test]
fn research_constructor_baseline() {
    for (index, input) in CONSTRUCTORS.iter().enumerate() {
        let native = reference::public_constructor(&input.velocity).unwrap();
        let disabled = candidate::public_constructor(&input.velocity).unwrap();
        assert_eq!(native_snapshot(&native), candidate_snapshot(&disabled));
        assert_eq!(native.allocated_bytes(), 474768);
        assert_eq!(disabled.allocated_bytes(), 474776);
        println!(
            "{{\"event\":\"constructor_conformance\",\"input\":{index},\"accepted\":{:?},\"candidate_without_chart_bytes\":{}}}",
            format!("{:?}", candidate_snapshot(&disabled)),
            disabled.allocated_bytes()
        );
    }
}
fn emit(
    o: &candidate::CoupledDiscreteFlow<'_>,
    kind: &str,
    field: &str,
    h: f64,
    load: &str,
    report: Option<&candidate::CoupledForcedExtrudedReport>,
) {
    let s = o.state();
    let positions: [[f64; 2]; 19] = std::array::from_fn(|i| s.geometry.nodes()[i].position);
    let periodic_indices: [usize; 19] =
        std::array::from_fn(|i| s.geometry.nodes()[i].periodic_index);
    let triangles: [[usize; 3]; 24] = std::array::from_fn(|i| s.geometry.triangles()[i].nodes);
    let mut physical_pressure = [0.; 24];
    for t in s.geometry.pressure_basis() {
        physical_pressure[t.triangle] += t.value * s.pressure_coefficients[t.mode];
    }
    let q = [positions[3][0], positions[4][0], positions[3][1]];
    let eta = [
        s.velocity[2][0],
        s.velocity[3][0],
        s.velocity[2][1],
        s.velocity[0][0],
        s.velocity[1][0],
        s.velocity[4][0],
    ];
    let mut xi = [0.; 12];
    for i in 0..16 {
        let e = s.geometry.velocity_embedding(i, 2).unwrap();
        if e.weights[0] == 1.
            && let Some(j) = e.columns[0]
        {
            xi[j - 22] = s.velocity[i][2];
        }
    }
    let unknowns = report.map_or_else(
        || "null".to_owned(),
        |r| format!("{:?}", r.step.planar.unknowns),
    );
    let reports=report.map_or_else(||"null".to_owned(),|r| {
        let p=r.step.planar;let w=r.step.third;let r=r.step;
        format!("{{\"planar\":{{\"energy_old\":{:?},\"energy_new\":{:?},\"backward_Euler_loss\":{:?},\"mixing_loss\":{:?},\"viscous_loss\":{:?},\"pressure_work\":{:?},\"GCL_work\":{:?},\"discrete_residual_work\":{:?},\"energy_ledger_error\":{:?},\"fixed_work_allowance\":{:?},\"finite_momentum_rate_norm\":{:?},\"direct_momentum_rate_norm\":{:?},\"gcl_max\":{:?},\"quadrature_error\":{:?},\"full_constraints\":{:?},\"iterations\":{},\"equation_evaluations\":{},\"sign_roots\":{},\"endpoint_vs_path_momentum_max\":{:?}}},\"third\":{{\"coefficients\":{:?},\"momentum_before\":{:?},\"momentum_after\":{:?},\"energy_old\":{:?},\"energy_new\":{:?},\"backward_Euler_loss\":{:?},\"mixing_loss\":{:?},\"shear_x_loss\":{:?},\"shear_y_loss\":{:?},\"viscous_loss\":{:?},\"GCL_work\":{:?},\"discrete_residual_work\":{:?},\"energy_ledger_error\":{:?},\"fixed_work_allowance\":{:?},\"finite_momentum_rate_norm\":{:?},\"direct_momentum_rate_norm\":{:?}}},\"total\":{{\"energy_old\":{:?},\"energy_new\":{:?},\"backward_Euler_loss\":{:?},\"mixing_loss\":{:?},\"viscous_loss\":{:?},\"GCL_work\":{:?},\"discrete_residual_work\":{:?},\"energy_ledger_error\":{:?},\"fixed_work_allowance\":{:?}}}}}",
            p.energy_before,p.energy_after,p.backward_euler_loss,p.mixing_loss,p.viscous_loss,p.pressure_work,p.gcl_work,p.residual_work,p.ledger_error,p.work_allowance,p.finite_momentum_rate_norm,p.direct_momentum_rate_norm,p.gcl_max,p.quadrature_error,p.full_constraints,p.iterations,p.equation_evaluations,p.sign_roots,p.endpoint_vs_path_momentum_max,
            w.coefficients,w.momentum_before,w.momentum_after,w.energy_before,w.energy_after,w.backward_euler_loss,w.mixing_loss,w.shear_x_loss,w.shear_y_loss,w.viscous_loss,w.gcl_work,w.residual_work,w.ledger_error,w.work_allowance,w.finite_momentum_rate_norm,w.direct_momentum_rate_norm,
            r.energy_before,r.energy_after,r.backward_euler_loss,r.mixing_loss,r.viscous_loss,r.gcl_work,r.residual_work,r.ledger_error,r.work_allowance)
    });
    let forcing=report.map_or_else(||"null".to_owned(),|r| {
        let f=r.forces;
        format!("{{\"acceleration\":{:?},\"force_count\":{},\"planar_work\":{:?},\"third_work\":{:?},\"total_work\":{:?},\"horizontal_impulse\":{:?},\"third_impulse\":{:?}}}",f.acceleration,f.force_count,f.planar_work,f.third_work,f.total_work,f.horizontal_impulse,f.third_impulse)
    });
    println!(
        "{{\"model\":\"periodic-z-invariant-forced\",\"kind\":\"{kind}\",\"field\":\"{field}\",\"load\":\"{load}\",\"h\":{h:?},\"step\":{},\"stamp\":{{\"id\":{},\"version\":{}}},\"unknowns\":{},\"end_q\":{:?},\"end_eta\":{:?},\"third_coefficients\":{:?},\"velocity\":{:?},\"pressure_coefficients\":{:?},\"time\":{:?},\"positions\":{:?},\"triangles\":{:?},\"periodic_indices\":{:?},\"mass\":{:?},\"physical_pressure\":{:?},\"report\":{},\"forcing\":{},\"allocated_bytes\":{}}}",
        s.stamp.version,
        s.stamp.id,
        s.stamp.version,
        unknowns,
        q,
        eta,
        xi,
        s.velocity,
        s.pressure_coefficients,
        s.time,
        positions,
        triangles,
        periodic_indices,
        s.geometry.nodal_mass(),
        physical_pressure,
        reports,
        forcing,
        o.allocated_bytes()
    );
}

#[test]
fn research_repeated_trajectory_one() {
    assert_eq!(std::env::var("RHEON_PUBLIC_CALL_ALLOW").unwrap(), POLICY_ID);
    let index: usize = std::env::var("RHEON_TRAJECTORY_INDEX")
        .unwrap()
        .parse()
        .unwrap();
    let control_mode = std::env::var("RHEON_LIFECYCLE_CONTROL").ok().as_deref() == Some("1");
    assert!(index < TRAJECTORIES.len());
    if control_mode {
        assert!(CONTROLS.contains(&index));
    }
    let case = &TRAJECTORIES[index];
    let input = &CONSTRUCTORS[case.input];
    let sign = if case.load == "forward" { 1. } else { -1. };
    let load = [BodyForce {
        value: [0.0625 * sign, -0.125 * sign, 0.03125 * sign],
        units: ForceUnits::Acceleration,
        region: None,
    }];
    let mut chart = scalar::ChartWorkspace::new();
    let mut owner = candidate::public_constructor(&input.velocity).unwrap();
    owner.set_chart(&mut chart);
    assert_eq!(owner.allocated_bytes(), 542352);
    println!(
        "{{\"event\":\"trajectory_begin\",\"index\":{index},\"lifecycle_control\":{control_mode},\"expected_steps\":{}}}",
        case.steps
    );
    emit(&owner, input.kind, input.field, case.h, case.load, None);
    let mut expected_time = 0.;
    let mut accepted = 0;
    for attempted in 1..=case.steps {
        let before = candidate_snapshot(&owner);
        let target = if control_mode {
            match attempted {
                2 => Some((3, 2)),
                5 => Some((3, 32)),
                9 => Some((8, 24)),
                16 => Some((10, 1)),
                _ => None,
            }
        } else {
            None
        };
        if let Some((stage, visit)) = target {
            for repetition in 0..2 {
                let mut cancel = Control {
                    target: Some((stage, visit)),
                    ..Default::default()
                };
                let result = candidate::public_call(&mut owner, &mut cancel, case.h, &load);
                if matches!(
                    result,
                    Err(candidate::CoupledDiscreteError::Cancelled { .. })
                ) {
                    assert_eq!(candidate_snapshot(&owner), before);
                    println!(
                        "{{\"event\":\"lifecycle_cancel\",\"index\":{index},\"step\":{attempted},\"stage\":{stage},\"visit\":{visit},\"repetition\":{repetition},\"status\":\"CANCELLED_EXACT_STATE_PRESERVED\",\"seen\":{:?},\"accepted\":{:?}}}",
                        cancel.seen,
                        format!("{:?}", candidate_snapshot(&owner))
                    );
                } else {
                    assert_eq!(
                        candidate_snapshot(&owner),
                        before,
                        "unreached target must not silently advance"
                    );
                    println!(
                        "{{\"event\":\"lifecycle_cancel\",\"index\":{index},\"step\":{attempted},\"stage\":{stage},\"visit\":{visit},\"repetition\":{repetition},\"status\":\"TARGET_NOT_REACHED_REFUSAL\",\"result\":{:?}}}",
                        format!("{result:?}")
                    );
                    break;
                }
            }
        }
        println!(
            "{{\"event\":\"trajectory_attempt\",\"index\":{index},\"step\":{attempted},\"before\":{:?}}}",
            format!("{before:?}")
        );
        let mut control = Control::default();
        let result = candidate::public_call(&mut owner, &mut control, case.h, &load);
        println!(
            "{{\"event\":\"trajectory_result\",\"index\":{index},\"step\":{attempted},\"result\":{:?},\"seen\":{:?},\"accepted\":{:?}}}",
            format!("{result:?}"),
            control.seen,
            format!("{:?}", candidate_snapshot(&owner))
        );
        match result {
            Ok(report) => {
                accepted += 1;
                expected_time += case.h;
                assert_eq!(owner.state().stamp.version, accepted);
                assert_eq!(owner.state().time.to_bits(), expected_time.to_bits());
                assert!(control.seen[10] > 0);
                emit(
                    &owner,
                    input.kind,
                    input.field,
                    case.h,
                    case.load,
                    Some(&report),
                );
            }
            Err(error) => {
                assert_eq!(candidate_snapshot(&owner), before);
                let retry =
                    candidate::public_call(&mut owner, &mut Control::default(), case.h, &load);
                assert_eq!(
                    format!("{retry:?}"),
                    format!(
                        "{:?}",
                        Err::<candidate::CoupledForcedExtrudedReport, _>(&error)
                    )
                );
                assert_eq!(candidate_snapshot(&owner), before);
                println!(
                    "{{\"event\":\"trajectory_terminal\",\"index\":{index},\"status\":\"REFUSED\",\"accepted_steps\":{accepted},\"expected_steps\":{},\"attempted_step\":{attempted},\"error\":{:?},\"same_owner_repeat\":\"EXACT_REFUSAL_AND_STATE\",\"state_preserved\":true}}",
                    case.steps,
                    format!("{error:?}")
                );
                return;
            }
        }
    }
    println!(
        "{{\"event\":\"trajectory_terminal\",\"index\":{index},\"status\":\"COMPLETE\",\"accepted_steps\":{accepted},\"expected_steps\":{},\"actual_time\":{:?}}}",
        case.steps,
        owner.state().time
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
fn fixed_capture_reference_preflight() {
    let mut scratch = scalar::ChartWorkspace::new();
    for case in &FIXED_CASES {
        reference::fixed_capture(case, &mut scratch);
    }
}
#[test]
fn fixed_capture_e2_experiment() {
    assert_eq!(
        std::env::var("RHEON_E2_FIXED_CAPTURE_ALLOW").unwrap(),
        "E2-two-fixed-candidates-native-known-v1"
    );
    let mut scratch = scalar::ChartWorkspace::new();
    for case in &FIXED_CASES {
        candidate::fixed_capture(case, &mut scratch);
    }
}

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
fn case41_borrowed_candidate_roster() {
    panic!("FD candidate runner is not authorized in this source freeze");
}
#[test]
fn case41_borrowed_reference_roster() {
    panic!("reference equation runner is not authorized in this source freeze");
}
#[inline(never)]
fn case41_borrowed_candidate_outer() {
    let mut chart = scalar::ChartWorkspace::new();
    candidate::case41_fd_capture(&FIXED_CASES[0], &mut chart);
}
#[inline(never)]
fn case41_borrowed_reference_outer() {
    let mut chart = scalar::ChartWorkspace::new();
    reference::case41_fd_capture(&FIXED_CASES[0], &mut chart);
}
#[used]
static CANDIDATE_OBSERVER: fn() = case41_borrowed_candidate_outer;
#[used]
static REFERENCE_OBSERVER: fn() = case41_borrowed_reference_outer;
#[test]
fn borrowed_wrapper_layout() {
    println!(
        "candidate={:?} reference={:?} plan={:?}",
        candidate::borrowed_layout(),
        reference::borrowed_layout(),
        FittedHeightPlan::new(2).unwrap()
    );
    crate::fitted_height::borrowed_geometry_layout();
}
