//! Static, deliberately unqualified P1 method-selection probes; see frozen protocol.
// Reuses immutable preview-independent fixture modules; unused APIs stay private.
#[allow(dead_code)]
#[path = "particle_planar_selection/eos.rs"]
mod eos;
#[allow(dead_code)]
#[path = "particle_planar_selection/kernel.rs"]
mod kernel;
#[path = "particle_adjoint_selection/model.rs"]
mod model;
use eos::WATER;
use model::{Map, State};
use std::io::{BufWriter, Write};

fn main() -> std::io::Result<()> {
    let root = std::env::args_os()
        .nth(1)
        .expect("fresh output directory required");
    std::fs::create_dir(&root)?;
    let root = std::path::Path::new(&root);
    let mut summary = BufWriter::new(std::fs::File::create(root.join("summary.csv"))?);
    let mut sites = BufWriter::new(std::fs::File::create(root.join("sites.csv"))?);
    let mut fd = BufWriter::new(std::fs::File::create(root.join("derivatives.csv"))?);
    writeln!(
        summary,
        "map,n,perturbed,hydro,work_normalized,momentum_normalized,e0_force_over_pressure,hydro_max_over_g,hydro_rms_over_g,volume,min_gamma,min_numerator,min_offset,max_offset,linear_interior_error,linear_boundary_error,alternating_nonendpoint_force,work_absolute,momentum_absolute,bottom_load,top_load,bottom_load_error,top_load_error,hydro_interior_max_over_g,hydro_boundary_max_over_g,bottom_load_absolute_error,top_load_absolute_error,max_pressure_acceleration"
    )?;
    writeln!(
        sites,
        "map,n,perturbed,hydro,i,y,mass,rho,pressure,volume,force,density_rate,volume_gradient,bottom_density_derivative,top_density_derivative"
    )?;
    writeln!(
        fd,
        "map,n,perturbed,hydro,eps,energy_fd,energy_analytic,energy_error,volume_fd,volume_analytic,volume_error,density_error"
    )?;
    for map in [Map::Gamma, Map::Voronoi] {
        for n in [32, 64] {
            for perturbed in [false, true] {
                for hydro in [false, true] {
                    let state = State::new(map, n, perturbed, hydro);
                    let p_ref = if hydro {
                        1000.0 + WATER.rho0 * WATER.gravity
                    } else {
                        WATER.sound_speed.powi(2) * WATER.rho0 * 0.005
                    };
                    let u_ref = 0.01;
                    let value = state.evaluate(&state.y);
                    let boundary =
                        |i: usize| state.y[i] < 3.0 / n as f64 || state.y[i] > 1.0 - 3.0 / n as f64;
                    let v: Vec<_> = (0..n)
                        .map(|i| 0.01 * ((i + 1) as f64 * 2.0_f64.sqrt()).sin())
                        .collect();
                    let rate = value.density_rate(&v);
                    let p: Vec<_> = value.rho.iter().map(|r| WATER.pressure(*r)).collect();
                    let (force, walls) = state.force(&state.y, &value, &p);
                    let kinetic = dot(&v, &force);
                    let internal: f64 = (0..n)
                        .map(|i| state.mass[i] * p[i] / value.rho[i].powi(2) * rate[i])
                        .sum();
                    let work = (kinetic + internal).abs() / (p_ref * u_ref);
                    let momentum = (force.iter().sum::<f64>() + walls[0] + walls[1]).abs() / p_ref;
                    let volume_gradient = value.volume_gradient(&state.mass);
                    let expected_walls = if hydro {
                        let a = 1.0 + 1000.0 / (WATER.rho0 * WATER.sound_speed.powi(2));
                        [
                            -WATER.rho0
                                * WATER.sound_speed.powi(2)
                                * (a * (WATER.gravity / WATER.sound_speed.powi(2)).exp() - 1.0),
                            1000.0,
                        ]
                    } else {
                        [-p_ref, p_ref]
                    };
                    let uniform_pressure = WATER.pressure(1.005 * WATER.rho0);
                    let e0 = if hydro {
                        f64::NAN
                    } else {
                        max_abs(&force) / uniform_pressure
                    };
                    let residual: Vec<_> = force
                        .iter()
                        .zip(&state.mass)
                        .map(|(f, m)| {
                            if hydro {
                                (f / m - WATER.gravity) / WATER.gravity
                            } else {
                                f64::NAN
                            }
                        })
                        .collect();
                    let linear: Vec<_> = state.y.iter().map(|y| 1000.0 + 1000.0 * y).collect();
                    let (linear_force, _) = state.force(&state.y, &value, &linear);
                    let linear_error: Vec<_> = (0..n)
                        .map(|i| {
                            (linear_force[i] + 1000.0 * state.mass[i] / value.rho[i]).abs()
                                / (1000.0 * state.mass[i] / value.rho[i])
                        })
                        .collect();
                    let alternating: Vec<_> = (0..n)
                        .map(|i| 1000.0 + if i % 2 == 0 { 100.0 } else { -100.0 })
                        .collect();
                    let (alternating_force, _) = state.force(&state.y, &value, &alternating);
                    writeln!(
                        summary,
                        "{},{n},{perturbed},{hydro},{work:.17e},{momentum:.17e},{e0:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
                        map.name(),
                        if hydro { max_abs(&residual) } else { f64::NAN },
                        (dot(&residual, &residual) / n as f64).sqrt(),
                        value.total_volume(&state.mass),
                        value.min_gamma,
                        value.min_numerator,
                        state.offset.iter().copied().fold(f64::INFINITY, f64::min),
                        state
                            .offset
                            .iter()
                            .copied()
                            .fold(f64::NEG_INFINITY, f64::max),
                        (0..n)
                            .filter(|i| !boundary(*i))
                            .map(|i| linear_error[i])
                            .fold(0.0, f64::max),
                        (0..n)
                            .filter(|i| boundary(*i))
                            .map(|i| linear_error[i])
                            .fold(0.0, f64::max),
                        max_abs(&alternating_force[1..n - 1]),
                        (kinetic + internal).abs(),
                        (force.iter().sum::<f64>() + walls[0] + walls[1]).abs(),
                        walls[0],
                        walls[1],
                        (walls[0] - expected_walls[0]).abs() / expected_walls[0].abs(),
                        (walls[1] - expected_walls[1]).abs() / expected_walls[1].abs(),
                        if hydro {
                            (0..n)
                                .filter(|i| !boundary(*i))
                                .map(|i| residual[i].abs())
                                .fold(0.0, f64::max)
                        } else {
                            f64::NAN
                        },
                        if hydro {
                            (0..n)
                                .filter(|i| boundary(*i))
                                .map(|i| residual[i].abs())
                                .fold(0.0, f64::max)
                        } else {
                            f64::NAN
                        },
                        (walls[0] - expected_walls[0]).abs(),
                        (walls[1] - expected_walls[1]).abs(),
                        force
                            .iter()
                            .zip(&state.mass)
                            .map(|(f, m)| (f / m).abs())
                            .fold(0.0, f64::max)
                    )?;
                    for i in 0..n {
                        writeln!(
                            sites,
                            "{},{n},{perturbed},{hydro},{i},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
                            map.name(),
                            state.y[i],
                            state.mass[i],
                            value.rho[i],
                            p[i],
                            state.mass[i] / value.rho[i],
                            force[i],
                            rate[i],
                            volume_gradient[i],
                            value.wall_jac[i][0],
                            value.wall_jac[i][1]
                        )?;
                    }
                    for eps in [1e-5, 1e-6, 1e-7] {
                        let yp: Vec<_> = state.y.iter().zip(&v).map(|(y, v)| y + eps * v).collect();
                        let ym: Vec<_> = state.y.iter().zip(&v).map(|(y, v)| y - eps * v).collect();
                        let plus = state.evaluate(&yp);
                        let minus = state.evaluate(&ym);
                        let energy_fd =
                            (plus.energy(&state.mass) - minus.energy(&state.mass)) / (2.0 * eps);
                        let volume_fd = (plus.total_volume(&state.mass)
                            - minus.total_volume(&state.mass))
                            / (2.0 * eps);
                        let volume_analytic = dot(&volume_gradient, &v);
                        let density_error = (0..n)
                            .map(|i| {
                                ((plus.rho[i] - minus.rho[i]) / (2.0 * eps) - rate[i]).abs()
                                    / (WATER.rho0 * u_ref * n as f64)
                            })
                            .fold(0.0, f64::max);
                        writeln!(
                            fd,
                            "{},{n},{perturbed},{hydro},{eps:.17e},{energy_fd:.17e},{internal:.17e},{:.17e},{volume_fd:.17e},{volume_analytic:.17e},{:.17e},{density_error:.17e}",
                            map.name(),
                            (energy_fd - internal).abs() / (p_ref * u_ref),
                            (volume_fd - volume_analytic).abs() / u_ref
                        )?;
                    }
                }
            }
        }
    }
    summary.flush()?;
    sites.flush()?;
    fd.flush()
}
fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(a, b)| a * b).sum()
}
fn max_abs(a: &[f64]) -> f64 {
    a.iter().map(|a| a.abs()).fold(0.0, f64::max)
}
