//! Three fixed native cases, 8192 steps/case, five saved snapshots each.
//! Generated HTML/CSV/JSON must go to a fresh directory outside Git.
use rheon::*;
use std::{
    fmt::Write as _,
    fs,
    io::{BufWriter, Write},
    path::Path,
    process::Command,
};
const AXES: [Axis; 3] = [Axis::X, Axis::Y, Axis::Z];
const STEPS: usize = 8192;
const DT: f64 = 96.0 / STEPS as f64;
const RHO: f64 = 3.0;
const MU: f64 = 0.15;
const SNAPSHOTS: [usize; 5] = [0, 128, 683, 2731, STEPS];
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
#[derive(Default)]
struct Ledger {
    initial_energy: f64,
    initial_momentum: f64,
    bulk: f64,
    wall_dissipation: f64,
    work: f64,
    update: f64,
    rounding: f64,
    impulse: [f64; 2],
    reaction: [f64; 2],
    rounding_momentum: f64,
    identity_max: f64,
    momentum_max: f64,
}
struct Samples<'a> {
    grid: &'a GridGeometry,
    velocity: &'a [Vec<f32>; 3],
    n: usize,
    ell: f64,
    step: usize,
    energy: f64,
    momentum: f64,
    ledger: &'a Ledger,
}
fn snapshot(
    json: &mut String,
    csv: &mut impl Write,
    s: Samples<'_>,
) -> Result<(f64, f64), Box<dyn std::error::Error>> {
    let mut continuum_error = 0.0_f64;
    let mut discrete_error = 0.0_f64;
    let l = s.ledger;
    write!(
        json,
        "{{\"step\":{},\"time\":{:.17e},\"energy\":{:.17e},\"momentum\":{:.17e},\"bulk\":{:.17e},\"wall_dissipation\":{:.17e},\"work\":{:.17e},\"update\":{:.17e},\"rounding\":{:.17e},\"impulse\":[{:.17e},{:.17e}],\"reaction\":[{:.17e},{:.17e}],\"rounding_momentum\":{:.17e},\"profiles\":[",
        s.step,
        s.step as f64 * DT,
        s.energy,
        s.momentum,
        l.bulk,
        l.wall_dissipation,
        l.work,
        l.update,
        l.rounding,
        l.impulse[0],
        l.impulse[1],
        l.reaction[0],
        l.reaction[1],
        l.rounding_momentum
    )?;
    for j in 0..s.n {
        let y = (j as f64 + 0.5) / s.n as f64;
        let actual = f64::from(s.velocity[0][s.grid.face_index(Axis::X, [0, j, 0]).unwrap()]);
        let continuum = (y + s.ell) / (1.0 + s.ell);
        let discrete = (j as f64 / s.n as f64 + s.ell) / ((s.n - 1) as f64 / s.n as f64 + s.ell);
        continuum_error = continuum_error.max((actual - continuum).abs());
        discrete_error = discrete_error.max((actual - discrete).abs());
        if j > 0 {
            json.push(',');
        }
        write!(
            json,
            "[{y:.17e},{actual:.17e},{continuum:.17e},{discrete:.17e}]"
        )?;
        writeln!(
            csv,
            "{:.17e},{y:.17e},{actual:.17e},{continuum:.17e},{discrete:.17e}",
            s.step as f64 * DT
        )?;
    }
    write!(
        json,
        "],\"continuum_deviation\":{continuum_error:.17e},\"discrete_deviation\":{discrete_error:.17e},\"energy_residual\":{:.17e},\"momentum_residual\":{:.17e}}}",
        s.energy - l.initial_energy + l.bulk + l.wall_dissipation - l.work - l.update - l.rounding,
        s.momentum - l.initial_momentum - l.impulse[0] - l.impulse[1] - l.rounding_momentum
    )?;
    Ok((continuum_error, discrete_error))
}
fn run(
    root: &Path,
    name: &str,
    n: usize,
    lower_navier: bool,
) -> Result<(String, f64, f64), Box<dyn std::error::Error>> {
    let g = GridGeometry::new([2, (n + 1) as u64, 2], [0.5, 1.0 / n as f64, 0.5], [0.0; 3])?;
    let mut fraction = vec![0.0; g.cell_len()];
    for k in 0..2 {
        for j in 0..n {
            for i in 0..2 {
                fraction[g.cell_index([i, j, k]).unwrap()] = 1.0;
            }
        }
    }
    let surface = ColumnSurfaceWorkspace::new(
        g.clone(),
        Axis::Y,
        &fraction,
        VolumeStamp {
            id: 182,
            version: 0,
        },
        1 << 20,
    )?;
    let mut u = fields(&g);
    // Compatible initial condition: the retained upper endpoint is already at
    // its prescribed speed. Its nonzero initial energy is kept in the ledger.
    let counts = g.face_counts(Axis::X);
    for k in 0..counts[2] {
        for i in 0..counts[0] {
            u[0][g.face_index(Axis::X, [i, n - 1, k]).unwrap()] = 1.0;
        }
    }
    let mut next = fields(&g);
    let mut workspace = ColumnShearWorkspace::new(g.clone(), Axis::Y, 1 << 20)?;
    let boundaries = [
        if lower_navier {
            ColumnShearBoundary::Navier(ColumnShearWall {
                velocity: [0.0; 3],
                friction: 0.3,
            })
        } else {
            ColumnShearBoundary::NoSlip { velocity: [0.0; 3] }
        },
        ColumnShearBoundary::NoSlip {
            velocity: [1.0, 0.0, 0.0],
        },
    ];
    let ell = if lower_navier { MU / 0.3 } else { 0.0 };
    let mut ledger = Ledger {
        initial_energy: 0.5 * RHO / n as f64,
        initial_momentum: RHO / n as f64,
        ..Ledger::default()
    };
    let mut csv = BufWriter::new(fs::File::create(root.join(format!("{name}.csv")))?);
    writeln!(csv, "time,y,velocity,continuum_steady,discrete_steady")?;
    let mut json = format!(
        "{{\"name\":\"{name}\",\"layers\":{n},\"rho\":{RHO},\"mu\":{MU},\"lower_navier\":{lower_navier},\"beta\":{},\"ell\":{ell},\"initial_energy\":{:.17e},\"initial_momentum\":{:.17e},\"snapshots\":[",
        if lower_navier { 0.3 } else { 0.0 },
        ledger.initial_energy,
        ledger.initial_momentum
    );
    snapshot(
        &mut json,
        &mut csv,
        Samples {
            grid: &g,
            velocity: &u,
            n,
            ell,
            step: 0,
            energy: ledger.initial_energy,
            momentum: ledger.initial_momentum,
            ledger: &ledger,
        },
    )?;
    let mut errors = (0.0, 0.0);
    for step in 1..=STEPS {
        let r = workspace.update_with_boundaries(
            ColumnShearInputs {
                surface: surface.state(),
                velocity: refs(&u),
                density: RHO,
                dynamic_viscosity: MU,
                dt: DT,
            },
            boundaries,
            muts(&mut next),
            |_| false,
        )?;
        ledger.bulk += DT * r.bulk_dissipation_before;
        ledger.wall_dissipation += DT * r.wall_dissipation_before;
        ledger.work += r.actuator_work;
        ledger.update += r.shear.update_energy;
        ledger.rounding += r.shear.rounding_work;
        ledger.rounding_momentum += r.shear.rounding_momentum[0];
        ledger.identity_max = ledger.identity_max.max(r.shear.identity_error.abs());
        ledger.momentum_max = ledger.momentum_max.max(r.shear.momentum_error[0].abs());
        for side in 0..2 {
            ledger.impulse[side] += r.wall_impulse[side][0];
            ledger.reaction[side] += r.reaction_impulse[side][0];
        }
        std::mem::swap(&mut u, &mut next);
        assert_eq!(
            u[0][g.face_index(Axis::X, [0, n - 1, 0]).unwrap()],
            1.0,
            "upper trace changed; stop"
        );
        if !lower_navier {
            assert_eq!(
                u[0][g.face_index(Axis::X, [0, 0, 0]).unwrap()],
                0.0,
                "lower trace changed; stop"
            );
        }
        if SNAPSHOTS.contains(&step) {
            json.push(',');
            errors = snapshot(
                &mut json,
                &mut csv,
                Samples {
                    grid: &g,
                    velocity: &u,
                    n,
                    ell,
                    step,
                    energy: r.shear.kinetic_after,
                    momentum: r.shear.momentum_after[0],
                    ledger: &ledger,
                },
            )?;
        }
    }
    write!(
        json,
        "],\"identity_max\":{:.17e},\"momentum_max\":{:.17e},\"workspace_payload_bytes\":{}}}",
        ledger.identity_max,
        ledger.momentum_max,
        workspace.allocated_bytes()
    )?;
    csv.flush()?;
    println!(
        "{name}: continuum_deviation={:.8e} discrete_deviation={:.8e} work={:.8e} impulse={:?} reaction={:?} identity_max={:.8e}",
        errors.0, errors.1, ledger.work, ledger.impulse, ledger.reaction, ledger.identity_max
    );
    Ok((json, errors.0, errors.1))
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
    let coarse = run(root, "no-slip-8", 8, false)?;
    let fine = run(root, "no-slip-16", 16, false)?;
    let mixed = run(root, "navier-no-slip-16", 16, true)?;
    // New fixed-basis engineering gates, declared before the experiment.
    // They do not change the old finite-Navier lab gates or establish order.
    assert!(
        fine.1 < coarse.1
            && fine.1 < 0.032
            && mixed.1 < 0.021
            && coarse.2 < 5e-5
            && fine.2 < 5e-5
            && mixed.2 < 5e-5,
        "fixed-slab profile acceptance failed; stop"
    );
    let data = format!(
        "{{\"dt\":{DT:.17e},\"steps\":{STEPS},\"cases\":[{},{},{}]}}",
        coarse.0, fine.0, mixed.0
    );
    fs::write(root.join("results.json"), &data)?;
    fs::write(
        root.join("index.html"),
        include_str!("column_no_slip.html").replace("__NATIVE_DATA__", &data),
    )?;
    Ok(())
}
