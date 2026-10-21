//! One n8 material-map reference gate. Not an SPH or production solver.
#[allow(dead_code)]
#[path = "particle_planar_selection/eos.rs"]
mod eos;
#[path = "material_partition_gate/model.rs"]
mod model;
use eos::WATER;
use model::{Model, cholesky_min};
use std::io::{BufWriter, Write};
fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(a, b)| a * b).sum()
}
fn maxabs(a: &[f64]) -> f64 {
    a.iter().map(|a| a.abs()).fold(0.0, f64::max)
}
fn main() -> std::io::Result<()> {
    let root = std::env::args_os()
        .nth(1)
        .expect("fresh output directory required");
    std::fs::create_dir(&root)?;
    let root = std::path::Path::new(&root);
    let mut s = BufWriter::new(std::fs::File::create(root.join("summary.csv"))?);
    let mut nodes = BufWriter::new(std::fs::File::create(root.join("nodes.csv"))?);
    let mut elements = BufWriter::new(std::fs::File::create(root.join("elements.csv"))?);
    let mut probes = BufWriter::new(std::fs::File::create(root.join("probes.csv"))?);
    writeln!(
        s,
        "perturbed,hydro,deformed,volume_error,mass_error,moment_error,work_error,work_absolute,momentum_error,momentum_absolute,equilibrium_error,bottom_load_error,top_load_error,bottom_load,top_load,wrong_gravity_error,wrong_wall_error,affine_error,cholesky_min,row_bound,quadratic_error,alternating_max,min_density,max_density"
    )?;
    writeln!(
        nodes,
        "perturbed,hydro,deformed,i,X,x,mass,force,pressure_force,gravity_force,volume"
    )?;
    writeln!(
        elements,
        "perturbed,hydro,deformed,e,L,mass,first,J,pressure"
    )?;
    writeln!(
        probes,
        "perturbed,hydro,deformed,translation,eps,energy_fd,energy_analytic,energy_error,volume_error,hessian_error"
    )?;
    for perturbed in [false, true] {
        for hydro in [false, true] {
            let m = Model::new(perturbed, hydro);
            let p_ref = if hydro {
                100.0 + WATER.rho0 * WATER.gravity
            } else {
                WATER.rho0 * WATER.sound_speed.powi(2) * 0.005
            };
            let power = p_ref * 0.01;
            for deformed in [false, true] {
                let x: Vec<_> = m
                    .reference
                    .iter()
                    .enumerate()
                    .map(|(i, x)| {
                        x + if deformed && i > 0 && i < 8 {
                            0.001 * (std::f64::consts::PI * x).sin()
                        } else {
                            0.0
                        }
                    })
                    .collect();
                let v = m.evaluate(&x).unwrap();
                let direction: Vec<_> = (0..=8)
                    .map(|i| {
                        if i == 0 || i == 8 {
                            0.0
                        } else {
                            0.01 * (i as f64 * 2.0_f64.sqrt()).sin()
                        }
                    })
                    .collect();
                let ue_rate: f64 = (0..8)
                    .map(|e| -v.pressure[e] * (direction[e + 1] - direction[e]))
                    .sum();
                let ug_rate = m.gravity * dot(&m.nodal_mass, &direction);
                let work = dot(&direction, &v.force) + ue_rate + ug_rate;
                let momentum =
                    v.force.iter().sum::<f64>() + m.gravity * m.nodal_mass.iter().sum::<f64>();
                let equilibrium = if deformed {
                    f64::NAN
                } else if hydro {
                    (1..8)
                        .map(|i| v.force[i].abs() / (m.gravity * m.nodal_mass[i]))
                        .fold(0.0, f64::max)
                } else {
                    maxabs(&v.force[1..8]) / p_ref
                };
                let pb = WATER.sound_speed.powi(2) * (m.source_density(0.0) - WATER.rho0);
                let pt = WATER.sound_speed.powi(2) * (m.source_density(1.0) - WATER.rho0);
                let bottom_error = if deformed {
                    f64::NAN
                } else {
                    (v.force[0] + pb).abs() / p_ref
                };
                let top_error = if deformed {
                    f64::NAN
                } else {
                    (v.force[8] - pt).abs() / p_ref
                };
                let wrong = m.wrong_half_mass();
                let wrong_gravity = if hydro && !deformed {
                    (1..8)
                        .map(|i| {
                            (v.pressure_force[i] - m.gravity * wrong[i]).abs()
                                / (m.gravity * m.nodal_mass[i])
                        })
                        .fold(0.0, f64::max)
                } else {
                    f64::NAN
                };
                let wrong_wall = if hydro && !deformed {
                    (v.pressure_force[0] + pb)
                        .abs()
                        .max((v.pressure_force[8] - pt).abs())
                        / p_ref
                } else {
                    f64::NAN
                };
                let cell_pressure: Vec<_> = x
                    .windows(2)
                    .map(|a| 100.0 + 1000.0 * 0.5 * (a[0] + a[1]))
                    .collect();
                let affine = m.manufactured_force(&x, &cell_pressure);
                let nodal_volume: Vec<_> = (0..=8)
                    .map(|i| {
                        0.5 * (if i > 0 { x[i] - x[i - 1] } else { 0.0 })
                            + 0.5 * (if i < 8 { x[i + 1] - x[i] } else { 0.0 })
                    })
                    .collect();
                let affine_error = (0..=8)
                    .map(|i| {
                        let lift = if i == 0 {
                            -(100.0 + 1000.0 * x[0])
                        } else if i == 8 {
                            100.0 + 1000.0 * x[8]
                        } else {
                            0.0
                        };
                        (affine[i] - lift + 1000.0 * nodal_volume[i]).abs() / 1000.0
                    })
                    .fold(0.0, f64::max);
                let q: Vec<_> = (0..=8)
                    .map(|i| {
                        if i == 0 || i == 8 {
                            0.0
                        } else {
                            0.001 * (i as f64 * 3.0_f64.sqrt()).sin()
                        }
                    })
                    .collect();
                let quadratic: f64 = (0..=8)
                    .map(|i| (0..=8).map(|j| q[i] * v.hessian[i][j] * q[j]).sum::<f64>())
                    .sum();
                let edge: f64 = (0..8)
                    .map(|e| {
                        WATER.sound_speed.powi(2) * m.mass[e]
                            / ((m.reference[e + 1] - m.reference[e]).powi(2)
                                * v.jacobian[e].powi(2))
                            * (q[e + 1] - q[e]).powi(2)
                    })
                    .sum();
                let row_bound = (1..8)
                    .map(|i| {
                        (1..8)
                            .map(|j| {
                                v.hessian[i][j].abs() / (m.nodal_mass[i] * m.nodal_mass[j]).sqrt()
                            })
                            .sum::<f64>()
                    })
                    .fold(0.0, f64::max);
                let alternate: Vec<_> = (0..8)
                    .map(|e| 100.0 + if e % 2 == 0 { 10.0 } else { -10.0 })
                    .collect();
                let alternating = m.manufactured_force(&x, &alternate);
                writeln!(
                    s,
                    "{perturbed},{hydro},{deformed},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{equilibrium:.17e},{bottom_error:.17e},{top_error:.17e},{:.17e},{:.17e},{wrong_gravity:.17e},{wrong_wall:.17e},{affine_error:.17e},{:.17e},{row_bound:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
                    (v.volume - (x[8] - x[0])).abs(),
                    (m.mass.iter().sum::<f64>() - m.nodal_mass.iter().sum::<f64>()).abs()
                        / WATER.rho0,
                    (v.mass_moment - dot(&m.nodal_mass, &x)).abs() / WATER.rho0,
                    work.abs() / power,
                    work.abs(),
                    momentum.abs() / p_ref,
                    momentum.abs(),
                    v.force[0],
                    v.force[8],
                    cholesky_min(&v.hessian),
                    (quadratic - edge).abs() / p_ref.max(quadratic.abs()),
                    maxabs(&alternating[1..8]),
                    v.min_density,
                    v.max_density
                )?;
                for i in 0..=8 {
                    writeln!(
                        nodes,
                        "{perturbed},{hydro},{deformed},{i},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
                        m.reference[i],
                        x[i],
                        m.nodal_mass[i],
                        v.force[i],
                        v.pressure_force[i],
                        -m.gravity * m.nodal_mass[i],
                        nodal_volume[i]
                    )?;
                }
                for e in 0..8 {
                    writeln!(
                        elements,
                        "{perturbed},{hydro},{deformed},{e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
                        m.reference[e + 1] - m.reference[e],
                        m.mass[e],
                        m.first[e],
                        v.jacobian[e],
                        v.pressure[e]
                    )?;
                }
                for translation in [false, true] {
                    let velocity = if translation {
                        vec![0.01; 9]
                    } else {
                        direction.clone()
                    };
                    let expected = if translation {
                        m.gravity * m.nodal_mass.iter().sum::<f64>() * 0.01
                    } else {
                        -dot(&v.force, &velocity)
                    };
                    for eps in [1e-5, 1e-6, 1e-7] {
                        let xp: Vec<_> =
                            x.iter().zip(&velocity).map(|(x, v)| x + eps * v).collect();
                        let xm: Vec<_> =
                            x.iter().zip(&velocity).map(|(x, v)| x - eps * v).collect();
                        let plus = m.evaluate(&xp).unwrap();
                        let minus = m.evaluate(&xm).unwrap();
                        let rate = (plus.energy - minus.energy) / (2.0 * eps);
                        let hessian_error = (0..=8)
                            .map(|i| {
                                let analytic: f64 =
                                    (0..=8).map(|j| -v.hessian[i][j] * velocity[j]).sum();
                                ((plus.force[i] - minus.force[i]) / (2.0 * eps) - analytic).abs()
                                    / (WATER.rho0 * WATER.sound_speed.powi(2) * 0.01)
                            })
                            .fold(0.0, f64::max);
                        let volume_error = if translation {
                            (plus.volume - v.volume)
                                .abs()
                                .max((minus.volume - v.volume).abs())
                        } else {
                            0.0
                        };
                        writeln!(
                            probes,
                            "{perturbed},{hydro},{deformed},{translation},{eps:.17e},{rate:.17e},{expected:.17e},{:.17e},{volume_error:.17e},{hessian_error:.17e}",
                            (rate - expected).abs() / power
                        )?;
                    }
                }
            }
        }
    }
    s.flush()?;
    nodes.flush()?;
    elements.flush()?;
    probes.flush()
}
