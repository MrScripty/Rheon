use super::{MU, NU, RHO, Result};
use rheon::{
    Axis, GridGeometry, ObstacleShearForce, ObstacleShearWall, StaticObstacleGeometry,
    StaticObstacleShear, SurfaceSettings, SurfaceStamp, TriangleSurface,
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
pub fn run(root: &Path) -> Result<()> {
    let mut out = std::io::BufWriter::new(std::fs::File::create(root.join("channel-summary.csv"))?);
    writeln!(
        out,
        "case,n,steps,dt,time,rms,relative_rms,max_energy_identity,max_momentum_identity,max_energy_increase,lower_traction,upper_traction"
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
        let mut u = exact.clone();
        let mut energy: f64 = 0.0;
        let mut momentum: f64 = 0.0;
        for _ in 0..20 {
            let r = w.step(
                &mut u,
                1.0,
                ObstacleShearForce::Acceleration(0.01),
                |_, _| false,
            )?;
            energy = energy.max(r.energy_identity_error.abs());
            momentum = momentum.max(r.momentum_identity_error.abs());
        }
        let square: f64 = u.iter().zip(&exact).map(|(a, b)| (a - b).powi(2)).sum();
        let rms = (square / n as f64).sqrt();
        let rel = (square / exact.iter().map(|u| u * u).sum::<f64>()).sqrt();
        let conductance = w.wall_conductances();
        writeln!(
            out,
            "steady,{n},20,1.0,20.0,{rms:.17e},{rel:.17e},{energy:.17e},{momentum:.17e},NaN,{:.17e},{:.17e}",
            conductance[0] * u[0] / 1e-6,
            conductance[1] * u[n - 1] / 1e-6
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
                for _ in 0..steps {
                    let r = w.step(&mut u, dt, ObstacleShearForce::Acceleration(0.0), |_, _| {
                        false
                    })?;
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
                    "temporal,{n},{steps},{dt:.17e},0.05,{rms:.17e},{rel:.17e},NaN,NaN,{increase:.17e},NaN,NaN"
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
    out.flush()?;
    Ok(())
}
