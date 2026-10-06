//! Actual bounded native endpoints, exported explicitly for independent replay.
use rheon::*;
const CAP: [[f64; 2]; 3] = [[0., 1.], [0.5, 1.25], [1., 1.]];
fn main() -> Result<(), Box<dyn std::error::Error>> {
    for kind in ["initial", "pressure_state"] {
        let geometry = FittedHeightGeometry {
            cap: &CAP,
            bottom_x: &[0., 0.5, 1.],
            extrusion_width: 1.,
            density: 3.,
            dynamic_viscosity: 0.05,
        };
        let w = FittedHeightWorkspace::new(geometry, Default::default(), |_| false)?;
        let mut velocity = [[0.; 3]; 16];
        for node in w.nodes() {
            velocity[node.periodic_index][0] = node.position[1];
        }
        if kind == "pressure_state" {
            velocity = PRESSURE_VELOCITY;
        }
        for h in [0.05, 0.025, 0.0125, 0.00625, 0.003125] {
            let mut owner = CoupledDiscreteFlow::new(
                geometry,
                &velocity,
                Default::default(),
                Default::default(),
                71,
            )?;
            for _ in 0..(0.1_f64 / h).round() as usize {
                let r = owner.step(h, |_| false)?;
                let s = owner.state();
                let positions: [[f64; 2]; 19] =
                    std::array::from_fn(|i| s.geometry.nodes()[i].position);
                let triangles: [[usize; 3]; 24] =
                    std::array::from_fn(|i| s.geometry.triangles()[i].nodes);
                let mut physical_pressure = [0.; 24];
                for term in s.geometry.pressure_basis() {
                    physical_pressure[term.triangle] +=
                        term.value * s.pressure_coefficients[term.mode];
                }
                println!(
                    "{{\"kind\":\"{kind}\",\"h\":{h:?},\"step\":{},\"stamp\":{{\"id\":{},\"version\":{}}},\"unknowns\":{:?},\"end_q\":{:?},\"end_eta\":{:?},\"velocity\":{:?},\"pressure_coefficients\":{:?},\"time\":{:?},\"positions\":{:?},\"triangles\":{:?},\"mass\":{:?},\"physical_pressure\":{:?},\"report\":{{\"energy_old\":{:?},\"energy_new\":{:?},\"backward_Euler_loss\":{:?},\"mixing_loss\":{:?},\"viscous_loss\":{:?},\"pressure_work\":{:?},\"GCL_work\":{:?},\"discrete_residual_work\":{:?},\"energy_ledger_error\":{:?},\"fixed_work_allowance\":{:?},\"finite_momentum_rate_norm\":{:?},\"direct_momentum_rate_norm\":{:?},\"gcl_max\":{:?},\"quadrature_error\":{:?},\"full_constraints\":{:?},\"iterations\":{},\"equation_evaluations\":{},\"sign_roots\":{},\"endpoint_vs_path_momentum_max\":{:?}}},\"allocated_bytes\":{}}}",
                    s.stamp.version,
                    s.stamp.id,
                    s.stamp.version,
                    r.unknowns,
                    r.end_q,
                    r.end_eta,
                    s.velocity,
                    s.pressure_coefficients,
                    s.time,
                    positions,
                    triangles,
                    s.geometry.nodal_mass(),
                    physical_pressure,
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
                    r.finite_momentum_rate_norm,
                    r.direct_momentum_rate_norm,
                    r.gcl_max,
                    r.quadrature_error,
                    r.full_constraints,
                    r.iterations,
                    r.equation_evaluations,
                    r.sign_roots,
                    r.endpoint_vs_path_momentum_max,
                    owner.allocated_bytes()
                );
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
