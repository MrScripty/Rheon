//! Actual published states on the doubly periodic, z-invariant height extrusion.
//! The cross-section moves; all three velocity components are exported. This
//! is a two-dimensional, three-component family with no extrusion end walls.
//! Diagnostic refinements retain all native gates and report actual refusals.
use rheon::*;
const CAP: [[f64; 2]; 3] = [[0., 1.], [0.5, 1.25], [1., 1.]];
const XI: [f64; 12] = [
    0.25, 0.5, -0.125, 0.375, 0.625, -0.25, 0.1875, 0.4375, -0.0625, 0.5625, 0.3125, 0.125,
];
fn emit(
    o: &CoupledDiscreteFlow,
    kind: &str,
    field: &str,
    h: f64,
    load: &str,
    report: Option<&CoupledForcedExtrudedReport>,
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
// Fixed diagnostic snapshot; no growing history or new accepted-state owner.
#[derive(PartialEq, Eq)]
struct Snapshot {
    velocity: [[u64; 3]; 16],
    positions: [[u64; 2]; 19],
    mass: [u64; 16],
    pressure: [u64; 16],
    time: u64,
    stamp: TranslatedViscousStamp,
    triangles: [[usize; 3]; 24],
    periodic_indices: [usize; 19],
}
fn snapshot(o: &CoupledDiscreteFlow) -> Snapshot {
    let s = o.state();
    Snapshot {
        velocity: std::array::from_fn(|i| s.velocity[i].map(f64::to_bits)),
        positions: std::array::from_fn(|i| s.geometry.nodes()[i].position.map(f64::to_bits)),
        mass: std::array::from_fn(|i| s.geometry.nodal_mass()[i].to_bits()),
        pressure: s.pressure_coefficients.map(f64::to_bits),
        time: s.time.to_bits(),
        stamp: s.stamp,
        triangles: std::array::from_fn(|i| s.geometry.triangles()[i].nodes),
        periodic_indices: std::array::from_fn(|i| s.geometry.nodes()[i].periodic_index),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let geometry = FittedHeightGeometry {
        cap: &CAP,
        bottom_x: &[0., 0.5, 1.],
        extrusion_width: 1.,
        density: 3.,
        dynamic_viscosity: 0.05,
    };
    for kind in ["initial", "pressure_state"] {
        for field in ["nonconstant"] {
            let g = FittedHeightWorkspace::new(geometry, Default::default(), |_| false)?;
            let mut z = [0.; 34];
            for n in g.nodes() {
                let e = g.velocity_embedding(n.periodic_index, 0).unwrap();
                if e.weights[0] == 1.
                    && let Some(j) = e.columns[0]
                {
                    z[j] = n.position[1];
                }
            }
            z[22..].copy_from_slice(&if field == "constant" { [0.25; 12] } else { XI });
            let mut velocity = [[0.; 3]; 16];
            g.embed_velocity(&z, &mut velocity)?;
            if kind == "pressure_state" {
                for i in 0..16 {
                    velocity[i][0] = PRESSURE_VELOCITY[i][0];
                    velocity[i][1] = PRESSURE_VELOCITY[i][1];
                }
            }
            for (load, sign) in [("forward", 1.), ("reversed", -1.)] {
                let force = BodyForce {
                    value: [0.0625 * sign, -0.125 * sign, 0.03125 * sign],
                    units: ForceUnits::Acceleration,
                    region: None,
                };
                for h in [0.0015625, 0.00078125] {
                    let mut o = CoupledDiscreteFlow::new_forced_extruded(
                        geometry,
                        &velocity,
                        Default::default(),
                        Default::default(),
                        131,
                    )?;
                    emit(&o, kind, field, h, load, None);
                    let expected = (0.1_f64 / h).round() as usize;
                    assert!(expected <= 128);
                    let mut accepted = 0;
                    let mut refused = false;
                    for attempted in 1..=expected {
                        let before = snapshot(&o);
                        match o.step_extruded_with_forces(h, &[force], |_| false) {
                            Ok(r) => {
                                accepted += 1;
                                emit(&o, kind, field, h, load, Some(&r));
                            }
                            Err(error) => {
                                assert!(snapshot(&o) == before, "refusal changed accepted state");
                                println!(
                                    "{{\"probe_status\":\"REFUSED\",\"kind\":\"{kind}\",\"field\":\"{field}\",\"load\":\"{load}\",\"h\":{h:?},\"accepted_steps\":{accepted},\"expected_steps\":{expected},\"attempted_step\":{attempted},\"state_preserved\":true,\"error\":{:?}}}",
                                    format!("{error:?}")
                                );
                                refused = true;
                                break;
                            }
                        }
                    }
                    if !refused {
                        println!(
                            "{{\"probe_status\":\"COMPLETE\",\"kind\":\"{kind}\",\"field\":\"{field}\",\"load\":\"{load}\",\"h\":{h:?},\"accepted_steps\":{accepted},\"expected_steps\":{expected}}}"
                        );
                    }
                }
            }
        }
    }
    Ok(())
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
