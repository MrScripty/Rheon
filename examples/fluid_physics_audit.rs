//! Independent CPU audit probes. Prints measurements, never edits solver state.
//! See docs/audits/general-fluid-20261019/protocol.md for fixed criteria.
use rheon::*;
use std::{error::Error, f64::consts::PI, fs, io::Write, path::Path, sync::Arc};
const RHO: f64 = 998.2;
const MU: f64 = 0.001_002;
const NU: f64 = MU / RHO;
const GRAVITY: f64 = 9.80665;
const TRIANGLES: [[usize; 3]; 12] = [
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
fn vertices(lo: [f64; 3], hi: [f64; 3]) -> Vec<[f64; 3]> {
    (0..8)
        .map(|c| std::array::from_fn(|d| if c & (1 << d) == 0 { lo[d] } else { hi[d] }))
        .collect()
}
fn triangles() -> Vec<[usize; 3]> {
    TRIANGLES.to_vec()
}
fn solid(lo: [f64; 3], hi: [f64; 3], angle: f64) -> Arc<CollisionMesh> {
    let center: [f64; 3] = std::array::from_fn(|d| (lo[d] + hi[d]) / 2.0);
    let v = vertices(lo, hi)
        .into_iter()
        .map(|p| {
            let x = p[0] - center[0];
            let y = p[1] - center[1];
            [
                center[0] + angle.cos() * x - angle.sin() * y,
                center[1] + angle.sin() * x + angle.cos() * y,
                p[2],
            ]
        })
        .collect();
    Arc::new(CollisionMesh::new(v, triangles(), MeshPolicy::ConvexSolid).unwrap())
}
fn surface(lo: [f64; 3], hi: [f64; 3]) -> TriangleSurface {
    TriangleSurface::new(
        SurfaceStamp {
            id: 321,
            version: 0,
        },
        vertices(lo, hi),
        triangles(),
        SurfaceSettings::default(),
    )
    .unwrap()
}
fn fields(g: &GridGeometry) -> [Vec<f32>; 3] {
    [Axis::X, Axis::Y, Axis::Z].map(|a| vec![0.0; g.face_len(a)])
}
fn refs(v: &[Vec<f32>; 3]) -> [&[f32]; 3] {
    v.each_ref().map(|x| x.as_slice())
}
fn settings() -> PressureSettings {
    PressureSettings {
        relative_residual: 1e-12,
        absolute_residual: 1e-12,
        divergence_limit: 1e-8,
        max_iterations: 3000,
    }
}
fn metric(out: &mut fs::File, case: &str, n: usize, variant: &str, name: &str, value: f64) {
    writeln!(out, "{case},{n},{variant},{name},{value:.17e}").unwrap();
}
fn hydro(out: &mut fs::File) -> Result<(), Box<dyn Error>> {
    for n in [8, 16, 32] {
        let h = 0.5 / n as f64;
        let g = GridGeometry::new([2, (n + 2) as u64, 2], [0.1, h, 0.1], [0.0; 3])?;
        let s = SlabFreeSurface::new(g.clone(), Axis::Y, n, VolumeStamp { id: 1, version: 0 })?;
        let op = PressureOperator::with_free_surface(&g, RHO, &s)?;
        let mut u = fields(&g);
        for k in 0..2 {
            for j in 1..=n {
                for i in 0..2 {
                    u[1][g.face_index(Axis::Y, [i, j, k]).unwrap()] = (-GRAVITY * 0.001) as f32;
                }
            }
        }
        for method in PressureImplementation::ALL {
            let mut ws = PressureWorkspace::with_implementation(&g, 1 << 24, method)?;
            ws.solve_velocity(&op, refs(&u), 0.001, settings(), || false)?;
            let mut err = 0.0_f64;
            for k in 0..2 {
                for j in 0..n {
                    for i in 0..2 {
                        let p = ws.pressure()[g.cell_index([i, j, k]).unwrap()];
                        err = err.max((p - RHO * GRAVITY * (0.5 - (j as f64 + 0.5) * h)).abs());
                    }
                }
            }
            let mut corrected = fields(&g);
            let [x, y, z] = &mut corrected;
            op.correct_velocity(refs(&u), ws.pressure(), 0.001, [x, y, z])?;
            metric(out, "hydro", n, method.id(), "pressure_max_error_pa", err);
            metric(
                out,
                "hydro",
                n,
                method.id(),
                "speed_max_m_s",
                corrected
                    .iter()
                    .flatten()
                    .map(|x| f64::from(*x).abs())
                    .fold(0.0, f64::max),
            );
            metric(
                out,
                "hydro",
                n,
                method.id(),
                "divergence_max_s_inv",
                ws.actual_divergence_max(&op, refs(&corrected))?,
            );
        }
    }
    Ok(())
}
fn pressure_mms(out: &mut fs::File) -> Result<(), Box<dyn Error>> {
    for n in [8, 16, 32, 64] {
        let length = 0.5;
        let h = length / n as f64;
        let grid = GridGeometry::new([1, n as u64, 1], [0.1, h, 0.1], [0.0; 3])?;
        let op = PressureOperator::new(&grid, RHO)?;
        let values: Vec<f64> = (0..n)
            .map(|j| 1000.0 * (PI * (j as f64 + 0.5) / n as f64).cos())
            .collect();
        let rhs: Vec<f64> = values
            .iter()
            .map(|p| grid.cell_volume() / RHO * (PI / length).powi(2) * p)
            .collect();
        for method in PressureImplementation::ALL {
            let mut ws = PressureWorkspace::with_implementation(&grid, 1 << 24, method)?;
            ws.solve_rhs(&op, &rhs, 0.001, settings(), || false)?;
            let error = (ws
                .pressure()
                .iter()
                .zip(&values)
                .map(|(p, e)| (p - (e - values[0])).powi(2))
                .sum::<f64>()
                / n as f64)
                .sqrt();
            metric(out, "pressure_mms", n, method.id(), "rms_error_pa", error);
        }
    }
    Ok(())
}
fn channel(out: &mut fs::File) -> Result<(), Box<dyn Error>> {
    let gap = 0.001;
    let bottom = 0.001;
    for n in [8, 16, 32] {
        let dy = gap / n as f64;
        let g = GridGeometry::new([1, (2 * n) as u64, 1], [0.001, dy, 0.001], [0.0; 3])?;
        let o = StaticObstacleGeometry::new(
            g,
            surface([0.0; 3], [0.001, bottom, 0.001]),
            1 << 24,
            |_, _| false,
        )?;
        let mut w = StaticObstacleShear::new(
            &o,
            Axis::Y,
            RHO,
            MU,
            ObstacleShearWall::NoSlip,
            1 << 24,
            |_, _| false,
        )?;
        let exact: Vec<f64> = w
            .layer_centers()
            .iter()
            .map(|y| {
                let y = y - bottom;
                0.01 * y * (gap - y) / (2.0 * NU)
            })
            .collect();
        let mut u = exact.clone();
        let mut residual = 0.0_f64;
        for _ in 0..20 {
            let r = w.step(
                &mut u,
                1.0,
                ObstacleShearForce::Acceleration(0.01),
                |_, _| false,
            )?;
            residual = residual.max(r.energy_identity_error.abs());
        }
        let rms = (u
            .iter()
            .zip(&exact)
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            / exact.iter().map(|b| b * b).sum::<f64>())
        .sqrt();
        metric(out, "channel", n, "steady", "relative_rms", rms);
        metric(
            out,
            "channel",
            n,
            "steady",
            "energy_identity_error_j",
            residual,
        );
        let mut f = fs::File::create(format!(
            "{}/channel-{n}.csv",
            std::env::args().nth(1).unwrap()
        ))?;
        writeln!(f, "y_m,u_m_s,exact_m_s")?;
        for ((y, u), e) in w.layer_centers().iter().zip(&u).zip(&exact) {
            writeln!(f, "{y:.17e},{u:.17e},{e:.17e}")?;
        }
        if n == 32 {
            let lambda = 4.0 * (PI / (2.0 * n as f64)).sin().powi(2) / dy.powi(2);
            for steps in [8, 16, 32] {
                let initial: Vec<f64> = w
                    .layer_centers()
                    .iter()
                    .map(|y| (PI * (y - bottom) / gap).sin())
                    .collect();
                let mut u = initial.clone();
                let mut max_increase = 0.0_f64;
                for _ in 0..steps {
                    let r = w.step(
                        &mut u,
                        0.05 / steps as f64,
                        ObstacleShearForce::Acceleration(0.0),
                        |_, _| false,
                    )?;
                    max_increase = max_increase.max(r.kinetic_after - r.kinetic_before);
                }
                let factor = (-NU * lambda * 0.05).exp();
                let err = (u
                    .iter()
                    .zip(initial)
                    .map(|(a, b)| (a - factor * b).powi(2))
                    .sum::<f64>()
                    / n as f64)
                    .sqrt();
                metric(out, "channel_time", steps, "sine", "rms_semidiscrete", err);
                metric(
                    out,
                    "channel_time",
                    steps,
                    "sine",
                    "max_energy_increase_j",
                    max_increase,
                );
            }
        }
    }
    Ok(())
}
fn viscous(out: &mut fs::File) -> Result<(), Box<dyn Error>> {
    for n in [8, 16, 32] {
        let length = 0.001;
        let h = length / n as f64;
        let g = GridGeometry::new([n as u64, n as u64, 1], [h, h, length], [0.0; 3])?;
        let mut u = fields(&g);
        for j in 0..n {
            for i in 1..n {
                let v = ((PI * i as f64 / n as f64).sin()
                    * (PI * (j as f64 + 0.5) / n as f64).cos()) as f32;
                u[0][g.face_index(Axis::X, [i, j, 0]).unwrap()] = v;
                u[1][g.face_index(Axis::Y, [j, i, 0]).unwrap()] = -v;
            }
        }
        let initial = u.clone();
        let mut v = fields(&g);
        let steps = (0.01 * NU * (2.0 / (h * h) + 1.0 / (length * length)) / 0.1).ceil() as usize;
        let mut w = ViscosityWorkspace::new(g, 1 << 24)?;
        let mut increase = 0.0_f64;
        for _ in 0..steps {
            let [x, y, z] = &mut v;
            let r = w.update(refs(&u), RHO, MU, 0.01 / steps as f64, [x, y, z], |_| false)?;
            increase = increase.max(r.kinetic_after - r.kinetic_before);
            std::mem::swap(&mut u, &mut v);
        }
        let factor = (-NU * 0.01 * 2.0 * PI * PI / (length * length)).exp();
        let err = (u
            .iter()
            .flatten()
            .zip(initial.iter().flatten())
            .map(|(a, b)| (f64::from(*a) - factor * f64::from(*b)).powi(2))
            .sum::<f64>()
            / u.iter().map(Vec::len).sum::<usize>() as f64)
            .sqrt();
        metric(out, "viscous", n, "vortex", "rms_continuum", err);
        metric(out, "viscous", n, "vortex", "steps", steps as f64);
        metric(
            out,
            "viscous",
            n,
            "vortex",
            "max_energy_increase_j",
            increase,
        );
    }
    Ok(())
}
fn dump_particles(
    dir: &Path,
    case: &str,
    n: usize,
    stage: &str,
    s: &ParticleFluidSimulation,
) -> Result<(), Box<dyn Error>> {
    let mut f = fs::File::create(dir.join(format!("{case}-{n}-{stage}.csv")))?;
    writeln!(
        f,
        "x,y,z,vx,vy,vz,mass_kg,density_kg_m3,pressure_pa,ax,ay,az"
    )?;
    let diag = s.particle_diagnostics().map(|(_, d)| d);
    for (i, p) in s.particles().iter().enumerate() {
        let d = diag.map(|d| d[i]).unwrap_or_default();
        writeln!(
            f,
            "{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
            p.position[0],
            p.position[1],
            p.position[2],
            p.velocity[0],
            p.velocity[1],
            p.velocity[2],
            p.mass,
            d.density,
            d.pressure,
            d.acceleration[0],
            d.acceleration[1],
            d.acceleration[2]
        )?;
    }
    Ok(())
}
fn crosses_barrier(a: [f64; 3], b: [f64; 3]) -> bool {
    let plane = |p: [f64; 3]| p[1] - 0.4 - (p[0] - 0.05) * 0.2 / 0.9;
    let (da, db) = (plane(a), plane(b));
    if da * db >= 0.0 {
        return false;
    }
    let t = da / (da - db);
    let x = a[0] + t * (b[0] - a[0]);
    let z = a[2] + t * (b[2] - a[2]);
    (0.05..=0.95).contains(&x) && (0.05..=0.95).contains(&z)
}
fn particle_case(
    out: &mut fs::File,
    dir: &Path,
    case: &str,
    n: usize,
) -> Result<(), Box<dyn Error>> {
    let grid = GridGeometry::new([8; 3], [0.125; 3], [0.0; 3])?;
    let config = ParticleFluidConfig {
        fill: DomainFill::EmittedFluid,
        samples_per_axis: n,
        max_particles: 8192,
        ..ParticleFluidConfig::default()
    };
    let mut s = ParticleFluidSimulation::new(grid, config)?;
    let full_mass = s.particles().iter().map(|p| p.mass).sum::<f64>();
    if case == "surface_box" || case == "release" {
        s.set_collision_mesh(Some(solid([0.0, 0.5, 0.0], [1.0; 3], 0.0)))?;
        s.set_collision_mesh(None)?;
    }
    let mesh = match case {
        "submerged_box" => Some(solid([0.35; 3], [0.65; 3], 0.37)),
        "surface_box" => Some(solid([0.35, 0.25, 0.35], [0.65, 0.75, 0.65], 0.37)),
        "barrier" => Some(Arc::new(CollisionMesh::new(
            vec![
                [0.05, 0.4, 0.05],
                [0.95, 0.6, 0.05],
                [0.95, 0.6, 0.95],
                [0.05, 0.4, 0.95],
            ],
            vec![[0, 1, 2], [0, 2, 3]],
            MeshPolicy::TwoSidedSurface,
        )?)),
        "release" => {
            s.set_collision_mesh(Some(solid([0.5, 0.0, 0.0], [1.0; 3], 0.0)))?;
            s.set_collision_mesh(None)?;
            Some(solid([0.60, 0.0, 0.35], [0.75, 0.35, 0.65], 0.0))
        }
        _ => None,
    };
    s.set_collision_mesh(mesh.clone())?;
    let m0 = s.particles().iter().map(|p| p.mass).sum::<f64>();
    dump_particles(dir, case, n, "initial", &s)?;
    let mut inside = 0;
    let mut crossings = 0;
    let mut initial_estimated_volume = None;
    let mut max_speed = 0.0_f64;
    let mut mass_drift = 0.0_f64;
    let end = if case == "release" { 0.4 } else { 0.1 };
    while s.time() < end - 1e-14 {
        let before: Vec<[f64; 3]> = if case == "barrier" {
            s.particles().iter().map(|p| p.position).collect()
        } else {
            Vec::new()
        };
        let r = s.step((end - s.time()).min(0.001), None, || false)?;
        if initial_estimated_volume.is_none() {
            initial_estimated_volume = Some(
                s.particles()
                    .iter()
                    .zip(s.particle_diagnostics().unwrap().1)
                    .map(|(p, d)| p.mass / d.density)
                    .sum::<f64>(),
            );
        }
        for (a, p) in before.iter().zip(s.particles()) {
            if crosses_barrier(*a, p.position) {
                crossings += 1;
            }
        }
        max_speed = max_speed.max(r.max_speed);
        mass_drift = mass_drift.max((r.total_mass - m0).abs() / m0);
        if let Some(mesh) = &mesh {
            inside += s
                .particles()
                .iter()
                .filter(|p| mesh.contains_or_touches(p.position))
                .count();
        }
    }
    dump_particles(dir, case, n, "final", &s)?;
    metric(
        out,
        case,
        n,
        "particle",
        "initial_excluded_mass_kg",
        full_mass - m0,
    );
    metric(out, case, n, "particle", "initial_mass_kg", m0);
    metric(
        out,
        case,
        n,
        "particle",
        "max_relative_mass_drift",
        mass_drift,
    );
    metric(
        out,
        case,
        n,
        "particle",
        "penetrating_centers_all_steps",
        inside as f64,
    );
    metric(out, case, n, "particle", "max_speed_m_s", max_speed);
    metric(out, case, n, "particle", "time_s", s.time());
    metric(
        out,
        case,
        n,
        "particle",
        "barrier_crossings_all_steps",
        crossings as f64,
    );
    metric(out, case, n, "particle", "steps", s.generation() as f64);
    if let Some((_, d)) = s.particle_diagnostics() {
        let estimated_volume = s
            .particles()
            .iter()
            .zip(d)
            .map(|(p, d)| p.mass / d.density)
            .sum::<f64>();
        metric(
            out,
            case,
            n,
            "particle",
            "density_estimated_volume_m3",
            estimated_volume,
        );
        metric(
            out,
            case,
            n,
            "particle",
            "density_estimated_volume_relative_drift",
            estimated_volume / initial_estimated_volume.unwrap() - 1.0,
        );
        metric(
            out,
            case,
            n,
            "particle",
            "height_max_m",
            s.particles()
                .iter()
                .map(|p| p.position[1])
                .fold(0.0, f64::max),
        );
        let density = (d
            .iter()
            .map(|d| (d.density / RHO - 1.0).powi(2))
            .sum::<f64>()
            / d.len() as f64)
            .sqrt();
        metric(out, case, n, "particle", "density_relative_rms", density);
        // Diagnostics describe preintegration state. Final positions differ by <= one accepted dt.
        let diffs: Vec<f64> = s
            .particles()
            .iter()
            .zip(d)
            .map(|(p, d)| d.pressure - RHO * GRAVITY * (1.0 - p.position[1]))
            .collect();
        let offset = diffs.iter().sum::<f64>() / diffs.len() as f64;
        let err = (diffs.iter().map(|x| (x - offset).powi(2)).sum::<f64>() / diffs.len() as f64)
            .sqrt()
            / (RHO * GRAVITY);
        metric(
            out,
            case,
            n,
            "particle",
            "hydrostatic_pressure_gauge_relative_rms",
            err,
        );
        let kinetic = s
            .particles()
            .iter()
            .map(|p| 0.5 * p.mass * p.velocity.iter().map(|v| v * v).sum::<f64>())
            .sum::<f64>();
        let potential = s
            .particles()
            .iter()
            .map(|p| p.mass * GRAVITY * p.position[1])
            .sum::<f64>();
        metric(out, case, n, "particle", "final_kinetic_j", kinetic);
        metric(out, case, n, "particle", "final_potential_j", potential);
    }
    if case == "barrier" {
        // Evaluation densities are from initial positions; obtain a fresh tiny step to compare at known positions.
        let positions = s.particles().to_vec();
        s.step(1e-10, None, || false)?;
        let h = 2.0 * s.particle_spacing();
        let poly = 315.0 / (64.0 * PI * h.powi(9));
        let mut err_all = 0.0_f64;
        let mut removed = 0.0;
        let mut cross_pairs = 0;
        let d = s.particle_diagnostics().unwrap().1;
        for (i, p) in positions.iter().enumerate() {
            let mut all = 0.0;
            let mut same = 0.0;
            for q in &positions {
                let r2 = (0..3)
                    .map(|a| (p.position[a] - q.position[a]).powi(2))
                    .sum::<f64>();
                if r2 < h * h {
                    let contribution = q.mass * poly * (h * h - r2).powi(3);
                    all += contribution;
                    if !crosses_barrier(p.position, q.position) {
                        same += contribution;
                    } else {
                        cross_pairs += 1;
                    }
                }
            }
            err_all = err_all.max((all.max(0.05 * RHO) - d[i].density).abs());
            removed += (all - same).abs();
        }
        metric(
            out,
            case,
            n,
            "particle",
            "all_neighbor_density_max_error",
            err_all,
        );
        metric(
            out,
            case,
            n,
            "particle",
            "mean_cross_plane_density_contribution",
            removed / positions.len() as f64,
        );
        metric(
            out,
            case,
            n,
            "particle",
            "cross_plane_neighbor_pairs",
            cross_pairs as f64,
        );
    }
    Ok(())
}
fn sweeps(out: &mut fs::File) -> Result<(), Box<dyn Error>> {
    for (index, scale) in [0.001, 1.0, 1000.0].into_iter().enumerate() {
        let mesh = solid([0.35 * scale; 3], [0.65 * scale; 3], 0.37);
        let mut penetration = 0;
        let mut max_vn = 0.0_f64;
        for j in 0..100 {
            let z = (0.36 + 0.28 * (j as f64 + 0.5) / 100.0) * scale;
            let (p, v) = mesh.resolve_segment(
                [0.1 * scale, 0.5 * scale, z],
                [0.9 * scale, 0.5 * scale, z],
                [1.0, 0.0, 0.0],
            )?;
            penetration += usize::from(mesh.contains_or_touches(p));
            let normal = [-0.37_f64.cos(), -0.37_f64.sin(), 0.0];
            max_vn = max_vn.max((0..3).map(|d| normal[d] * v[d]).sum::<f64>().abs());
        }
        metric(out, "sweep", index, "rotated", "scale_m", scale);
        metric(
            out,
            "sweep",
            index,
            "rotated",
            "penetrations",
            penetration as f64,
        );
        metric(
            out,
            "sweep",
            index,
            "rotated",
            "max_normal_speed_m_s",
            max_vn,
        );
    }
    let v = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
    let refused = CollisionMesh::new(v, vec![[0, 1, 2]], MeshPolicy::ConvexSolid).is_err();
    metric(
        out,
        "invalid",
        0,
        "open_solid",
        "refused",
        f64::from(refused),
    );
    Ok(())
}
fn main() -> Result<(), Box<dyn Error>> {
    let destination = std::env::args()
        .nth(1)
        .ok_or("supply a fresh external output directory")?;
    let dir = Path::new(&destination);
    fs::create_dir(dir)?;
    let mut out = fs::File::create(dir.join("measurements.csv"))?;
    writeln!(out, "case,n,variant,metric,value")?;
    hydro(&mut out)?;
    pressure_mms(&mut out)?;
    channel(&mut out)?;
    viscous(&mut out)?;
    sweeps(&mut out)?;
    for case in ["rest", "submerged_box", "surface_box", "barrier", "release"] {
        for n in [8, 12, 16] {
            println!("running {case} n={n}");
            particle_case(&mut out, dir, case, n)?;
        }
    }
    println!("Finished CPU probes at {}", dir.display());
    Ok(())
}
