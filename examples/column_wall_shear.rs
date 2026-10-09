//! Bounded native wall-friction lab: six cases, 8192 steps/case, 96 seconds.
//! Writes only to a caller-selected fresh directory outside a Git worktree.
use rheon::*;
use std::{
    fs,
    io::{BufWriter, Write},
    path::Path,
    process::Command,
};
const AXES: [Axis; 3] = [Axis::X, Axis::Y, Axis::Z];
fn refs(v: &[Vec<f32>; 3]) -> [&[f32]; 3] {
    [&v[0], &v[1], &v[2]]
}
fn muts(v: &mut [Vec<f32>; 3]) -> [&mut [f32]; 3] {
    let [x, y, z] = v;
    [x, y, z]
}
fn fields(g: &GridGeometry) -> [Vec<f32>; 3] {
    std::array::from_fn(|d| vec![0.0; g.face_len(AXES[d])])
}
fn fixture(n: usize) -> Result<(GridGeometry, ColumnSurfaceWorkspace), Box<dyn std::error::Error>> {
    let g = GridGeometry::new([2, (n + 1) as u64, 2], [0.5, 1.0 / n as f64, 0.5], [0.0; 3])?;
    let mut fraction = vec![0.0; g.cell_len()];
    for k in 0..2 {
        for j in 0..n {
            for i in 0..2 {
                fraction[g.cell_index([i, j, k]).unwrap()] = 1.0;
            }
        }
    }
    let s = ColumnSurfaceWorkspace::new(
        g.clone(),
        Axis::Y,
        &fraction,
        VolumeStamp {
            id: 181,
            version: 0,
        },
        1 << 20,
    )?;
    Ok((g, s))
}
fn run(
    root: &Path,
    name: &str,
    n: usize,
    rho: f64,
    mu: f64,
    beta: f64,
) -> Result<(f64, f64), Box<dyn std::error::Error>> {
    let (g, s) = fixture(n)?;
    let mut u = fields(&g);
    let mut next = fields(&g);
    let mut w = ColumnShearWorkspace::new(g.clone(), Axis::Y, 1 << 20)?;
    let walls = [
        ColumnShearWall {
            velocity: [0.0; 3],
            friction: beta,
        },
        ColumnShearWall {
            velocity: [1.0, 0.0, 0.0],
            friction: beta,
        },
    ];
    let dt = 96.0 / 8192.0;
    let ell = mu / beta;
    let mut csv = BufWriter::new(fs::File::create(root.join(format!("{name}.csv")))?);
    writeln!(csv, "time,y,velocity,continuum_steady,discrete_steady")?;
    let mut last = None;
    let mut identity_max = 0.0_f64;
    let mut momentum_max = 0.0_f64;
    let mut bulk = 0.0;
    let mut friction = 0.0;
    let mut work = 0.0;
    let mut rounding = 0.0;
    let mut update = 0.0;
    for step in 1..=8192 {
        let r = w.update_with_walls(
            ColumnShearInputs {
                surface: s.state(),
                velocity: refs(&u),
                density: rho,
                dynamic_viscosity: mu,
                dt,
            },
            walls,
            muts(&mut next),
            |_| false,
        )?;
        identity_max = identity_max.max(r.shear.identity_error.abs());
        momentum_max = momentum_max.max(r.shear.momentum_error[0].abs());
        bulk += dt * r.bulk_dissipation_before;
        friction += dt * r.wall_dissipation_before;
        work += r.actuator_work;
        rounding += r.shear.rounding_work;
        update += r.shear.update_energy;
        std::mem::swap(&mut u, &mut next);
        if [128, 683, 2731, 8192].contains(&step) {
            for j in 0..n {
                let y = (j as f64 + 0.5) / n as f64;
                let actual = f64::from(u[0][g.face_index(Axis::X, [0, j, 0]).unwrap()]);
                writeln!(
                    csv,
                    "{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
                    step as f64 * dt,
                    y,
                    actual,
                    (y + ell) / (1.0 + 2.0 * ell),
                    (j as f64 / n as f64 + ell) / ((n - 1) as f64 / n as f64 + 2.0 * ell)
                )?;
            }
        }
        last = Some(r);
    }
    csv.flush()?;
    let r = last.unwrap();
    let mut continuum_error = 0.0_f64;
    let mut discrete_error = 0.0_f64;
    let mut html = BufWriter::new(fs::File::create(root.join(format!("{name}.html")))?);
    writeln!(
        html,
        "<!doctype html><meta charset=utf-8><title>{name}</title><style>body{{font:18px system-ui;max-width:850px;margin:40px auto}}table{{border-collapse:collapse}}td,th{{padding:8px;border:1px solid #ddd}}</style><h1>Native Rust wall-friction lab: {name}</h1><p>ρ={rho} kg/m³, μ={mu} Pa s, β={beta} Pa s/m; {n} layers; 8192 steps; T=96 s. Fixed flat slab; prescribed lower/upper tangential walls. Blue: actual Rust velocity. Grey: continuum steady reference. CSV includes four actual transient snapshots.</p><svg viewBox='0 0 500 330' role='img' aria-label='Velocity against slab height'><path d='M40 10 V290 H470' stroke='black' fill='none'/>"
    )?;
    for (native, color) in [(false, "#777"), (true, "#087eae")] {
        write!(
            html,
            "<polyline fill='none' stroke='{color}' stroke-width='3' points='"
        )?;
        for j in 0..n {
            let y = (j as f64 + 0.5) / n as f64;
            let actual = f64::from(u[0][g.face_index(Axis::X, [0, j, 0]).unwrap()]);
            let analytic = (y + ell) / (1.0 + 2.0 * ell);
            let discrete = (j as f64 / n as f64 + ell) / ((n - 1) as f64 / n as f64 + 2.0 * ell);
            continuum_error = continuum_error.max((actual - analytic).abs());
            discrete_error = discrete_error.max((actual - discrete).abs());
            write!(
                html,
                "{},{} ",
                40.0 + 420.0 * if native { actual } else { analytic },
                290.0 - 280.0 * y
            )?;
        }
        writeln!(html, "'/>")?;
    }
    writeln!(
        html,
        "<text x='170' y='325'>Tangential velocity (m/s)</text></svg><table><tr><th>Measure</th><th>Value</th></tr><tr><td>Max continuum steady error (m/s)</td><td>{continuum_error:.8e}</td></tr><tr><td>Max discrete steady error (m/s)</td><td>{discrete_error:.8e}</td></tr><tr><td>Actuator work (J)</td><td>{work:.8e}</td></tr><tr><td>Bulk / wall dissipation (J)</td><td>{bulk:.8e} / {friction:.8e}</td></tr><tr><td>Final kinetic energy (J)</td><td>{:.8e}</td></tr><tr><td>Largest per-step work / momentum identity error</td><td>{identity_max:.8e} / {momentum_max:.8e}</td></tr></table><p>Integrated ledger residual: {:.8e} J. Endpoint basis extends center values constantly to each wall, giving spatial error that must decrease under refinement. This lab does not model wetting, adhesion, contact lines, collision meshes or an evolving interface.</p><p><a href='{name}.csv'>Native transient CSV</a></p>",
        r.shear.kinetic_after,
        r.shear.kinetic_after + bulk + friction - work - update - rounding
    )?;
    html.flush()?;
    println!(
        "{name}: continuum_max={continuum_error:.8e} discrete_max={discrete_error:.8e} energy={:.8e} wall_traction={:?} identity_max={identity_max:.8e}",
        r.shear.kinetic_after, r.wall_force
    );
    Ok((continuum_error, discrete_error))
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::env::args()
        .nth(1)
        .ok_or("supply a fresh external output directory")?;
    let root = Path::new(&out);
    if root.exists() {
        return Err("fresh output directory required".into());
    }
    let ancestor = root
        .ancestors()
        .find(|p| p.is_dir())
        .ok_or("existing output ancestor required")?;
    let probe = Command::new("git")
        .arg("-C")
        .arg(ancestor)
        .args(["rev-parse", "--show-toplevel"])
        .output()?;
    if probe.status.success() {
        return Err("outputs must be outside Git".into());
    }
    if probe.status.code() != Some(128)
        || !String::from_utf8_lossy(&probe.stderr).contains("not a git repository")
    {
        return Err("cannot verify external output location".into());
    }
    fs::create_dir(root)?;
    let a = run(root, "baseline-8", 8, 3.0, 0.15, 0.3)?;
    let b = run(root, "baseline-16", 16, 3.0, 0.15, 0.3)?;
    // Fixed engineering gates, declared before this native experiment.
    assert!(
        b.0 < a.0 && b.0 < 0.01 && a.1 < 5e-4 && b.1 < 5e-4,
        "Couette profile acceptance failed; stop"
    );
    for (name, rho, mu, beta) in [
        ("density-6", 6.0, 0.15, 0.3),
        ("viscosity-030", 3.0, 0.3, 0.3),
        ("friction-015", 3.0, 0.15, 0.15),
        ("friction-060", 3.0, 0.15, 0.6),
    ] {
        run(root, name, 16, rho, mu, beta)?;
    }
    fs::write(
        root.join("index.html"),
        "<!doctype html><meta charset=utf-8><title>Rheon native wall friction</title><h1>Native flat-slab wall-friction laboratory</h1><p>Six actual Rust cases, bounded to 8192 steps each. Parameters change physical momentum and wall traction; they do not merely change drawing style.</p><ul><li><a href='baseline-8.html'>8-layer baseline</a></li><li><a href='baseline-16.html'>16-layer baseline</a></li><li><a href='density-6.html'>Double density</a></li><li><a href='viscosity-030.html'>Double viscosity</a></li><li><a href='friction-015.html'>Half wall friction</a></li><li><a href='friction-060.html'>Double wall friction</a></li></ul>",
    )?;
    Ok(())
}
