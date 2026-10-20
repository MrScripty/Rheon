use super::{MU, NU, RHO, Result};
use rheon::{
    Axis, GridGeometry, ObstacleShearForce, ObstacleShearReport, ObstacleShearWall,
    StaticObstacleGeometry, StaticObstacleShear, SurfaceSettings, SurfaceStamp, TriangleSurface,
};
use std::{io::Write, path::Path};
fn surface() -> TriangleSurface {
    let vertices = (0..8)
        .map(|i| std::array::from_fn(|d| if i & (1 << d) == 0 { 0.0 } else { 0.001 }))
        .collect();
    let triangles = vec![
        [0, 2, 3],
        [0, 3, 1],
        [4, 5, 7],
        [4, 7, 6],
        [0, 1, 5],
        [0, 5, 4],
        [2, 6, 7],
        [2, 7, 3],
        [0, 4, 6],
        [0, 6, 2],
        [1, 3, 7],
        [1, 7, 5],
    ];
    TriangleSurface::new(
        SurfaceStamp {
            id: 321,
            version: 0,
        },
        vertices,
        triangles,
        SurfaceSettings::default(),
    )
    .unwrap()
}
fn ledger_row(
    out: &mut impl Write,
    case: &str,
    n: usize,
    steps: usize,
    step: usize,
    r: &ObstacleShearReport,
) -> std::io::Result<()> {
    write!(out, "{case},{n},{steps},{step},{:.17e}", step as f64 * r.dt)?;
    for value in [
        r.dt,
        r.force_density,
        r.kinetic_before,
        r.kinetic_after,
        r.increment_energy,
        r.body_work,
        r.dissipation,
        r.boundary_dissipation,
        r.energy_identity_error,
        r.energy_rounding_budget,
        r.momentum_before,
        r.momentum_after,
        r.body_impulse,
        r.wall_impulse,
        r.momentum_identity_error,
        r.momentum_rounding_budget,
        r.residual_max,
    ] {
        write!(out, ",{value:.17e}")?;
    }
    writeln!(out)
}
fn state_rows(
    out: &mut impl Write,
    case: &str,
    steps: usize,
    step_time: (usize, f64),
    centers: &[f64],
    u: &[f64],
    exact: &[f64],
) -> std::io::Result<()> {
    let (step, time) = step_time;
    for (layer, ((y, value), reference)) in centers.iter().zip(u).zip(exact).enumerate() {
        writeln!(
            out,
            "{case},{},{steps},{step},{time:.17e},{layer},{y:.17e},{value:.17e},{reference:.17e}",
            u.len()
        )?;
    }
    Ok(())
}
pub fn run(root: &Path) -> Result<()> {
    let mut out = std::io::BufWriter::new(std::fs::File::create(root.join("channel-summary.csv"))?);
    writeln!(
        out,
        "case,n,steps,dt,time,rms,relative_rms,max_energy_identity,max_momentum_identity,max_energy_increase,lower_traction,upper_traction,flux,exact_flux,max_force_residual"
    )?;
    let mut ledger =
        std::io::BufWriter::new(std::fs::File::create(root.join("channel-ledger.csv"))?);
    writeln!(
        ledger,
        "case,n,steps,step,time,dt,force_density,kinetic_before,kinetic_after,increment_energy,body_work,dissipation,boundary_dissipation,energy_identity_error,energy_rounding_budget,momentum_before,momentum_after,body_impulse,wall_impulse,momentum_identity_error,momentum_rounding_budget,residual_max"
    )?;
    let mut states =
        std::io::BufWriter::new(std::fs::File::create(root.join("channel-states.csv"))?);
    writeln!(states, "case,n,steps,step,time,layer,y,u,exact_reference")?;
    let mut geometry_out =
        std::io::BufWriter::new(std::fs::File::create(root.join("channel-geometry.csv"))?);
    writeln!(
        geometry_out,
        "n,layer,center,volume,mass,conductance_to_next,lower_wall_conductance,upper_wall_conductance"
    )?;
    for n in [8, 16, 32] {
        let dy = 0.001 / n as f64;
        let grid = GridGeometry::new([1, (2 * n) as u64, 1], [0.001, dy, 0.001], [0.0; 3])?;
        let geometry = StaticObstacleGeometry::new(grid, surface(), 1 << 24, |_, _| false)?;
        let mut w = StaticObstacleShear::new(
            &geometry,
            Axis::Y,
            RHO,
            MU,
            ObstacleShearWall::NoSlip,
            1 << 24,
            |_, _| false,
        )?;
        let exact: Vec<_> = w
            .layer_centers()
            .iter()
            .map(|y| {
                let y = y - 0.001;
                0.01 * y * (0.001 - y) / (2.0 * NU)
            })
            .collect();
        let walls = w.wall_conductances();
        for layer in 0..n {
            writeln!(
                geometry_out,
                "{n},{layer},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
                w.layer_centers()[layer],
                w.layer_volumes()[layer],
                RHO * w.layer_volumes()[layer],
                w.interior_conductances().get(layer).copied().unwrap_or(0.0),
                walls[0],
                walls[1]
            )?;
        }
        let mut u = exact.clone();
        state_rows(
            &mut states,
            "steady",
            20,
            (0, 0.0),
            w.layer_centers(),
            &u,
            &exact,
        )?;
        let mut energy: f64 = 0.0;
        let mut momentum: f64 = 0.0;
        let mut residual: f64 = 0.0;
        for step in 1..=20 {
            let r = w.step(
                &mut u,
                1.0,
                ObstacleShearForce::Acceleration(0.01),
                |_, _| false,
            )?;
            ledger_row(&mut ledger, "steady", n, 20, step, &r)?;
            state_rows(
                &mut states,
                "steady",
                20,
                (step, step as f64),
                w.layer_centers(),
                &u,
                &exact,
            )?;
            energy = energy.max(r.energy_identity_error.abs());
            momentum = momentum.max(r.momentum_identity_error.abs());
            residual = residual.max(r.residual_max);
        }
        let square: f64 = u.iter().zip(&exact).map(|(a, b)| (a - b).powi(2)).sum();
        let rms = (square / n as f64).sqrt();
        let rel = (square / exact.iter().map(|u| u * u).sum::<f64>()).sqrt();
        let conductance = w.wall_conductances();
        writeln!(
            out,
            "steady,{n},20,1.0,20.0,{rms:.17e},{rel:.17e},{energy:.17e},{momentum:.17e},NaN,{:.17e},{:.17e},{:.17e},{:.17e},{residual:.17e}",
            conductance[0] * u[0] / 1e-6,
            conductance[1] * u[n - 1] / 1e-6,
            0.001 * dy * u.iter().sum::<f64>(),
            0.001 * 0.01 * 0.001_f64.powi(3) / (12.0 * NU)
        )?;
        let mut fields = std::io::BufWriter::new(std::fs::File::create(
            root.join(format!("channel-{n}.csv")),
        )?);
        writeln!(fields, "y,u,exact")?;
        for i in 0..n {
            writeln!(
                fields,
                "{:.17e},{:.17e},{:.17e}",
                w.layer_centers()[i],
                u[i],
                exact[i]
            )?;
        }
        fields.flush()?;
        if n == 32 {
            let lambda = 4.0 * (std::f64::consts::PI / (2.0 * n as f64)).sin().powi(2) / dy.powi(2);
            for steps in [8, 16, 32] {
                let initial: Vec<_> = w
                    .layer_centers()
                    .iter()
                    .map(|y| (std::f64::consts::PI * (y - 0.001) / 0.001).sin())
                    .collect();
                let mut u = initial.clone();
                let mut increase: f64 = 0.0;
                let dt = 0.05 / steps as f64;
                state_rows(
                    &mut states,
                    "temporal",
                    steps,
                    (0, 0.0),
                    w.layer_centers(),
                    &u,
                    &initial,
                )?;
                for step in 1..=steps {
                    let r = w.step(&mut u, dt, ObstacleShearForce::Acceleration(0.0), |_, _| {
                        false
                    })?;
                    ledger_row(&mut ledger, "temporal", n, steps, step, &r)?;
                    let reference: Vec<_> = initial
                        .iter()
                        .map(|v| v * (-NU * lambda * step as f64 * dt).exp())
                        .collect();
                    state_rows(
                        &mut states,
                        "temporal",
                        steps,
                        (step, step as f64 * dt),
                        w.layer_centers(),
                        &u,
                        &reference,
                    )?;
                    increase = increase.max(r.kinetic_after - r.kinetic_before);
                }
                let factor = (-NU * lambda * 0.05).exp();
                let square: f64 = u
                    .iter()
                    .zip(&initial)
                    .map(|(a, b)| (a - factor * b).powi(2))
                    .sum();
                let rms = (square / n as f64).sqrt();
                let rel = (square / (factor * factor * initial.iter().map(|u| u * u).sum::<f64>()))
                    .sqrt();
                writeln!(
                    out,
                    "temporal,{n},{steps},{dt:.17e},0.05,{rms:.17e},{rel:.17e},NaN,NaN,{increase:.17e},NaN,NaN,NaN,NaN,NaN"
                )?;
                let mut fields = std::io::BufWriter::new(std::fs::File::create(
                    root.join(format!("channel-time-{steps}.csv")),
                )?);
                writeln!(fields, "y,u,exact_semidiscrete")?;
                for i in 0..n {
                    writeln!(
                        fields,
                        "{:.17e},{:.17e},{:.17e}",
                        w.layer_centers()[i],
                        u[i],
                        factor * initial[i]
                    )?;
                }
                fields.flush()?;
            }
        }
    }
    ledger.flush()?;
    states.flush()?;
    geometry_out.flush()?;
    out.flush()?;
    Ok(())
}
