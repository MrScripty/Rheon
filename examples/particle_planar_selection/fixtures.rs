use super::{
    matrix,
    planar::{Column, Method},
};
use std::io::Write;

pub fn hydro(
    method: Method,
    n: usize,
    ratio: f64,
    perturbed: bool,
    dt: f64,
    summary: &mut impl Write,
    raw: &mut impl Write,
) -> std::io::Result<()> {
    let mut c = Column::hydrostatic(n, ratio, perturbed);
    let initial_state = c.clone();
    let mut rejected_state = None;
    let initial_energy = c.energy();
    let mass0: f64 = c.mass.iter().sum();
    let initial = c.evaluate(method, false, 0.0, false);
    let defect = (initial
        .pressure_acceleration
        .iter()
        .map(|a| (a - c.water.gravity).powi(2))
        .sum::<f64>()
        / n as f64)
        .sqrt();
    let mut time = 0.0_f64;
    let mut max_speed = 0.0_f64;
    let mut pressure_work = 0.0;
    let mut viscous_work = 0.0;
    let mut momentum_defect = 0.0;
    let mut kick_drift_remainder = 0.0;
    let mut density_kick_remainder = 0.0;
    let mut eos_chain_remainder = 0.0;
    let mut steps = 0;
    let mut status = "complete";
    let mut max_neighbors = initial.max_neighbors;
    while time < 0.1 {
        let actual_dt = dt.min(0.1 - time);
        let e = c.evaluate(method, false, 0.0, false);
        max_neighbors = max_neighbors.max(e.max_neighbors);
        let pair_defect: f64 = c
            .mass
            .iter()
            .zip(e.pressure_acceleration.iter().zip(&e.viscous_acceleration))
            .map(|(m, (a, b))| m * (a + b))
            .sum::<f64>()
            + e.wall_pressure_force
            + e.wall_viscous_force;
        // Velocity kick then continuity/drift with the kicked velocity. This
        // research adaptation is NOT simultaneous forward Euler acoustics.
        let mut kicked_state = c.clone();
        for i in 0..n {
            kicked_state.velocity[i] += actual_dt
                * (e.pressure_acceleration[i] + e.viscous_acceleration[i] - c.water.gravity);
        }
        let kicked = kicked_state.evaluate(method, false, 0.0, false);
        let mut next = kicked_state;
        for i in 0..n {
            next.rho[i] += actual_dt * kicked.density_rate[i];
            next.y[i] += actual_dt * next.velocity[i];
        }
        let finite = next
            .y
            .iter()
            .chain(&next.rho)
            .chain(&next.velocity)
            .all(|v| v.is_finite());
        let speed = next
            .velocity
            .iter()
            .map(|v| v.abs())
            .fold(0.0_f64, f64::max);
        if !finite {
            status = "nonfinite-refused";
            rejected_state = Some(next);
            break;
        }
        if next.rho.iter().any(|r| *r <= 0.0) {
            status = "density-refused";
            rejected_state = Some(next);
            break;
        }
        if next.y.iter().any(|y| *y <= 0.0) {
            status = "crossing-refused";
            rejected_state = Some(next);
            break;
        }
        if speed / c.water.sound_speed > 0.1 {
            status = "mach-refused";
            rejected_state = Some(next);
            break;
        }
        // Independent exact finite-step decomposition: kinetic kick and
        // affine gravitational drift, density's use of kicked velocity, then
        // the nonlinear EOS Taylor remainder. None is inferred from ΔE.
        for i in 0..n {
            let acceleration =
                e.pressure_acceleration[i] + e.viscous_acceleration[i] - c.water.gravity;
            kick_drift_remainder += c.mass[i]
                * actual_dt.powi(2)
                * (0.5 * acceleration.powi(2) + c.water.gravity * acceleration);
            let eprime = c.water.pressure(c.rho[i]) / c.rho[i].powi(2);
            density_kick_remainder +=
                c.mass[i] * eprime * actual_dt * (kicked.density_rate[i] - e.density_rate[i]);
            eos_chain_remainder += c.mass[i]
                * (c.water.energy(next.rho[i])
                    - c.water.energy(c.rho[i])
                    - eprime * (next.rho[i] - c.rho[i]));
        }
        pressure_work += actual_dt * e.pressure_work_defect;
        viscous_work += actual_dt * e.viscous_power;
        momentum_defect += actual_dt * pair_defect;
        c = next;
        time += actual_dt;
        steps += 1;
        max_speed = max_speed.max(speed);
    }
    let density_error =
        (c.y.iter()
            .enumerate()
            .map(|(i, y)| {
                let exact = c.water.equilibrium((c.height - y).max(0.0)).unwrap().0;
                ((c.rho[i] - exact) / exact).powi(2)
            })
            .sum::<f64>()
            / n as f64)
            .sqrt();
    let pressure_error =
        (c.y.iter()
            .enumerate()
            .map(|(i, y)| {
                let exact = c.water.equilibrium((c.height - y).max(0.0)).unwrap().1;
                (c.water.pressure(c.rho[i]) - exact).powi(2)
            })
            .sum::<f64>()
            / n as f64)
            .sqrt()
            / (c.water.rho0 * c.water.gravity * c.height);
    let energy_change = c.energy() - initial_energy;
    let mass: f64 = c.mass.iter().sum();
    // Closure now compares independently evaluated finite-step terms.
    let remainder = energy_change
        - pressure_work
        - viscous_work
        - kick_drift_remainder
        - density_kick_remainder
        - eos_chain_remainder;
    writeln!(
        summary,
        "{},{},{},{},{:.17e},{},{},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{}",
        method.name(),
        n,
        ratio,
        perturbed,
        dt,
        status,
        steps,
        time,
        defect,
        max_speed,
        density_error,
        pressure_error,
        (mass - mass0) / mass0,
        energy_change,
        pressure_work,
        viscous_work,
        remainder,
        kick_drift_remainder,
        density_kick_remainder,
        eos_chain_remainder,
        momentum_defect,
        max_neighbors
    )?;
    writeln!(
        raw,
        "state,index,time_s,y_m,velocity_m_s,rho_kg_m3,pressure_pa,mass_kg_m2"
    )?;
    for (label, state, t) in [
        ("initial", &initial_state, 0.0),
        ("last-accepted", &c, time),
    ] {
        for i in 0..n {
            writeln!(
                raw,
                "{},{},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
                label,
                i,
                t,
                state.y[i],
                state.velocity[i],
                state.rho[i],
                state.water.pressure(state.rho[i]),
                state.mass[i]
            )?;
        }
    }
    if let Some(state) = rejected_state {
        for i in 0..n {
            writeln!(
                raw,
                "refused-candidate,{},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
                i,
                time + dt.min(0.1 - time),
                state.y[i],
                state.velocity[i],
                state.rho[i],
                state.water.pressure(state.rho[i]),
                state.mass[i]
            )?;
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
pub enum ShearKind {
    Couette,
    Poiseuille,
    Slip,
}
pub fn shear(
    method: Method,
    n: usize,
    ratio: f64,
    dt: f64,
    kind: ShearKind,
    summary: &mut impl Write,
    raw: &mut impl Write,
) -> std::io::Result<()> {
    let mut c = Column::channel(n, ratio);
    let nu = c.water.mu / c.water.rho0;
    let couette = matches!(kind, ShearKind::Couette);
    let slip = matches!(kind, ShearKind::Slip);
    let upper = if couette { 0.001 } else { 0.0 };
    let body = if matches!(kind, ShearKind::Poiseuille) {
        0.01
    } else {
        0.0
    };
    let exact: Vec<_> =
        c.y.iter()
            .map(|y| {
                if slip {
                    0.001
                } else if couette {
                    upper * y / c.height
                } else {
                    body * y * (c.height - y) / (2.0 * nu)
                }
            })
            .collect();
    // Assemble from the same native evaluation that advances the profile,
    // including nonlocal extrapolated dummy velocities and wall slope law.
    let mut a = vec![vec![0.0; n]; n];
    for j in 0..n {
        c.velocity.fill(0.0);
        c.velocity[j] = 1.0;
        let e = c.evaluate(method, true, 0.0, slip);
        for (i, row) in a.iter_mut().enumerate() {
            row[j] = e.viscous_acceleration[i];
        }
    }
    c.velocity.fill(0.0);
    let source = c.evaluate(method, true, upper, slip).viscous_acceleration;
    // Uniform channel masses make the Euclidean norm exactly proportional
    // to kinetic energy. Slip has a constant null mode, so use its directly
    // derived nonnegative row/Markov bound only for this constant invariant.
    let bound = if slip {
        Some(
            1.0 / a
                .iter()
                .enumerate()
                .map(|(i, row)| -row[i])
                .fold(0.0_f64, f64::max),
        )
    } else {
        matrix::contraction_limit(&a)
    };
    let mut status = "complete";
    let mut time = 0.0_f64;
    let mut actual_dt = 0.0_f64;
    let mut steps = 0;
    let mut final_dt = 0.0;
    let mut lower_impulse = 0.0;
    let mut upper_impulse = 0.0;
    let mut source_impulse = 0.0;
    let mut wall_work = 0.0;
    let mut viscous_fluid_work = 0.0;
    let mut max_momentum_defect = 0.0_f64;
    let mut temporal_energy = 0.0;
    let mass: f64 = c.mass.iter().sum();
    c.velocity = exact.clone();
    if let Some(limit) = bound {
        actual_dt = dt.min(0.9 * limit);
        while time < 0.1 {
            let step = actual_dt.min(0.1 - time);
            let e = c.evaluate(method, true, upper, slip);
            let acceleration = matrix::multiply(&a, &c.velocity);
            let momentum_rate: f64 = c
                .mass
                .iter()
                .zip(&acceleration)
                .enumerate()
                .map(|(i, (m, a))| m * (a + source[i]))
                .sum();
            let pair_defect = momentum_rate + e.wall_viscous_force;
            max_momentum_defect = max_momentum_defect.max(pair_defect.abs());
            lower_impulse += step * e.lower_viscous_force;
            upper_impulse += step * e.upper_viscous_force;
            source_impulse += step * body * mass;
            wall_work -= step * e.upper_viscous_force * upper;
            viscous_fluid_work += step * e.viscous_power;
            for i in 0..n {
                let acc = acceleration[i] + source[i] + body;
                temporal_energy += 0.5 * c.mass[i] * (step * acc).powi(2);
                c.velocity[i] += step * acc;
            }
            final_dt = step;
            time += step;
            steps += 1;
        }
    } else {
        status = "viscous-contraction-refused";
    }
    let numerator: f64 = c
        .velocity
        .iter()
        .zip(&exact)
        .map(|(u, e)| (u - e).powi(2))
        .sum();
    let denominator: f64 = exact.iter().map(|v| v * v).sum();
    let error = (numerator / denominator).sqrt();
    let e = c.evaluate(method, true, upper, slip);
    let exact_lower = if couette {
        c.water.mu * upper / c.height
    } else {
        c.water.rho0 * body * c.height / 2.0
    };
    let exact_upper = if couette { -exact_lower } else { exact_lower };
    let traction_error = if slip {
        e.lower_viscous_force.abs().max(e.upper_viscous_force.abs())
            / (c.water.mu * 0.001 / c.height)
    } else {
        ((e.lower_viscous_force - exact_lower)
            .abs()
            .max((e.upper_viscous_force - exact_upper).abs()))
            / exact_lower.abs()
    };
    let momentum_change: f64 = c
        .mass
        .iter()
        .zip(c.velocity.iter().zip(&exact))
        .map(|(m, (u, e))| m * (u - e))
        .sum();
    let momentum_ledger = momentum_change - source_impulse + lower_impulse + upper_impulse;
    writeln!(
        summary,
        "{},{},{},{},{:.17e},{},{},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
        method.name(),
        if slip {
            "free-slip-constant"
        } else if couette {
            "couette"
        } else {
            "poiseuille"
        },
        n,
        ratio,
        dt,
        status,
        steps,
        actual_dt,
        bound.unwrap_or(0.0),
        error,
        traction_error,
        momentum_ledger,
        max_momentum_defect,
        wall_work,
        viscous_fluid_work,
        viscous_fluid_work - wall_work,
        temporal_energy,
        e.lower_viscous_force,
        e.upper_viscous_force,
        time,
        final_dt
    )?;
    writeln!(raw, "index,y_m,velocity_x_m_s,exact_velocity_x_m_s")?;
    for (i, expected) in exact.iter().enumerate() {
        writeln!(
            raw,
            "{},{:.17e},{:.17e},{:.17e}",
            i, c.y[i], c.velocity[i], expected
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn free_slip_fixture_advances_a_nonzero_invariant_and_measures_traction() {
        for method in [Method::Dummy, Method::Gamma] {
            let mut summary = Vec::new();
            let mut raw = Vec::new();
            shear(
                method,
                16,
                1.5,
                1e-4,
                ShearKind::Slip,
                &mut summary,
                &mut raw,
            )
            .unwrap();
            let summary = String::from_utf8(summary).unwrap();
            let fields: Vec<_> = summary.trim().split(',').collect();
            assert_eq!(fields.len(), 21);
            assert_eq!(fields[5], "complete");
            assert!(fields[9].parse::<f64>().unwrap() < 1e-12);
            assert_eq!(fields[10].parse::<f64>().unwrap(), 0.0);
            assert_eq!(fields[19].parse::<f64>().unwrap(), 0.1);
            for line in String::from_utf8(raw).unwrap().lines().skip(1) {
                let fields: Vec<f64> = line.split(',').map(|v| v.parse().unwrap()).collect();
                assert_eq!(fields[3], 0.001);
                assert!((fields[2] - 0.001).abs() < 1e-15);
            }
        }
    }
}
