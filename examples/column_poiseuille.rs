//! Seven fixed Poiseuille cases; simultaneous explicit viscosity/forcing.
//! Outputs require a fresh directory outside Git. No automatic numerical retry.
use rheon::*;
use std::{
    fmt::Write as _,
    fs,
    io::{BufWriter, Write},
    path::Path,
    process::Command,
};
const STEPS: usize = 8192;
const DT: f64 = 96.0 / STEPS as f64;
const SAVED: [usize; 5] = [0, 128, 683, 2731, STEPS];
const AXES: [Axis; 3] = [Axis::X, Axis::Y, Axis::Z];
#[derive(Clone, Copy)]
struct Case {
    name: &'static str,
    n: usize,
    rho: f64,
    mu: f64,
    value: f64,
    units: ForceUnits,
}
const CASES: [Case; 7] = [
    Case {
        name: "q-8",
        n: 8,
        rho: 3.0,
        mu: 0.15,
        value: 0.3,
        units: ForceUnits::ForceDensity,
    },
    Case {
        name: "q-16",
        n: 16,
        rho: 3.0,
        mu: 0.15,
        value: 0.3,
        units: ForceUnits::ForceDensity,
    },
    Case {
        name: "a-16",
        n: 16,
        rho: 3.0,
        mu: 0.15,
        value: 0.1,
        units: ForceUnits::Acceleration,
    },
    Case {
        name: "q-rho6-16",
        n: 16,
        rho: 6.0,
        mu: 0.15,
        value: 0.3,
        units: ForceUnits::ForceDensity,
    },
    Case {
        name: "a-rho6-16",
        n: 16,
        rho: 6.0,
        mu: 0.15,
        value: 0.1,
        units: ForceUnits::Acceleration,
    },
    Case {
        name: "q-mu2-16",
        n: 16,
        rho: 3.0,
        mu: 0.3,
        value: 0.3,
        units: ForceUnits::ForceDensity,
    },
    Case {
        name: "q-negative-16",
        n: 16,
        rho: 3.0,
        mu: 0.15,
        value: -0.3,
        units: ForceUnits::ForceDensity,
    },
];
impl Case {
    fn q(self) -> f64 {
        match self.units {
            ForceUnits::Acceleration => self.rho * self.value,
            ForceUnits::ForceDensity => self.value,
        }
    }
    fn steady(self, j: usize) -> f64 {
        self.q() / (2.0 * self.mu) * j as f64 * (self.n - 1 - j) as f64 / (self.n * self.n) as f64
    }
}
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
// Closed form of the exact-real explicit discrete recurrence from zero.
// D=(L-1)h; DST coefficients of the separately derived equilibrium. No
// reference trajectory is integrated, and this is not a continuum transient.
struct Modal {
    coefficient: Vec<f64>,
    eigenvalue: Vec<f64>,
}
impl Modal {
    fn new(c: Case) -> Self {
        let m = (c.n - 1) as f64;
        let mut coefficient = Vec::new();
        let mut eigenvalue = Vec::new();
        for k in 1..c.n - 1 {
            let angle = std::f64::consts::PI * k as f64 / m;
            coefficient.push(
                2.0 / m
                    * (1..c.n - 1)
                        .map(|j| c.steady(j) * (angle * j as f64).sin())
                        .sum::<f64>(),
            );
            eigenvalue.push(
                1.0 - 4.0 * DT * c.mu / c.rho * (c.n * c.n) as f64 * (angle / 2.0).sin().powi(2),
            );
        }
        Self {
            coefficient,
            eigenvalue,
        }
    }
    fn at(&self, c: Case, j: usize, step: usize) -> f64 {
        if step == 0 || j == 0 || j + 1 == c.n {
            return 0.0;
        }
        c.steady(j)
            - self
                .coefficient
                .iter()
                .zip(&self.eigenvalue)
                .enumerate()
                .map(|(k, (&b, &lambda))| {
                    b * lambda.powi(step as i32)
                        * (std::f64::consts::PI * (k + 1) as f64 * j as f64 / (c.n - 1) as f64)
                            .sin()
                })
                .sum::<f64>()
    }
}
#[derive(Default)]
struct Ledger {
    bulk: f64,
    wall_loss: f64,
    wall_work: f64,
    body_work: f64,
    update: f64,
    rounding: f64,
    wall_impulse: [f64; 2],
    body_impulse: f64,
    round_p: f64,
    identity_max: f64,
    momentum_max: f64,
}
struct Run {
    json: String,
    final_profile: Vec<f64>,
    early_peak: f64,
    continuum_error: f64,
}
struct Snapshot<'a> {
    c: Case,
    g: &'a GridGeometry,
    u: &'a [Vec<f32>; 3],
    modal: &'a Modal,
    step: usize,
    energy: f64,
    momentum: f64,
    ledger: &'a Ledger,
}
fn snapshot(
    json: &mut String,
    csv: &mut impl Write,
    s: Snapshot<'_>,
) -> Result<(f64, f64), Box<dyn std::error::Error>> {
    let c = s.c;
    let l = s.ledger;
    let mut continuum = 0.0_f64;
    let mut discrete = 0.0_f64;
    let mut transient = 0.0_f64;
    let mut flow = 0.0;
    write!(
        json,
        "{{\"step\":{},\"time\":{:.17e},\"energy\":{:.17e},\"momentum\":{:.17e},\"bulk\":{:.17e},\"wall_loss\":{:.17e},\"wall_work\":{:.17e},\"body_work\":{:.17e},\"update\":{:.17e},\"rounding\":{:.17e},\"wall_impulse\":[{:.17e},{:.17e}],\"body_impulse\":{:.17e},\"rounding_momentum\":{:.17e},\"profiles\":[",
        s.step,
        s.step as f64 * DT,
        s.energy,
        s.momentum,
        l.bulk,
        l.wall_loss,
        l.wall_work,
        l.body_work,
        l.update,
        l.rounding,
        l.wall_impulse[0],
        l.wall_impulse[1],
        l.body_impulse,
        l.round_p
    )?;
    for j in 0..c.n {
        let y = (j as f64 + 0.5) / c.n as f64;
        let actual = f64::from(s.u[0][s.g.face_index(Axis::X, [0, j, 0]).unwrap()]);
        let steady = c.q() * y * (1.0 - y) / (2.0 * c.mu);
        let eq = c.steady(j);
        let exact = s.modal.at(c, j, s.step);
        continuum = continuum.max((actual - steady).abs());
        discrete = discrete.max((actual - eq).abs());
        transient = transient.max((actual - exact).abs());
        flow += actual / c.n as f64;
        if j > 0 {
            json.push(',');
        }
        write!(
            json,
            "[{y:.17e},{actual:.17e},{steady:.17e},{eq:.17e},{exact:.17e}]"
        )?;
        writeln!(
            csv,
            "{:.17e},{y:.17e},{actual:.17e},{steady:.17e},{eq:.17e},{exact:.17e}",
            s.step as f64 * DT
        )?;
    }
    let er = s.energy + l.bulk + l.wall_loss - l.wall_work - l.body_work - l.update - l.rounding;
    let pr = s.momentum - l.wall_impulse[0] - l.wall_impulse[1] - l.body_impulse - l.round_p;
    // Predeclared saved-time gates; no changing tolerance after a refusal.
    assert!(
        transient < 3e-5 && er.abs() < 1e-9 && pr.abs() < 1e-9,
        "saved transient/ledger gate failed; stop"
    );
    write!(
        json,
        "],\"flow_per_width\":{flow:.17e},\"continuum_deviation\":{continuum:.17e},\"discrete_deviation\":{discrete:.17e},\"transient_deviation\":{transient:.17e},\"energy_residual\":{er:.17e},\"momentum_residual\":{pr:.17e}}}"
    )?;
    if s.step == STEPS {
        assert!(discrete < 3e-5, "steady relaxation gate failed; stop");
    }
    Ok((continuum, actual_peak(c, s.g, s.u)))
}
fn actual_peak(c: Case, g: &GridGeometry, u: &[Vec<f32>; 3]) -> f64 {
    (0..c.n)
        .map(|j| f64::from(u[0][g.face_index(Axis::X, [0, j, 0]).unwrap()]).abs())
        .fold(0.0, f64::max)
}
fn run(root: &Path, c: Case) -> Result<Run, Box<dyn std::error::Error>> {
    let g = GridGeometry::new(
        [2, (c.n + 1) as u64, 2],
        [0.5, 1.0 / c.n as f64, 0.5],
        [0.0; 3],
    )?;
    let mut fraction = vec![0.0; g.cell_len()];
    for k in 0..2 {
        for j in 0..c.n {
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
            id: 183,
            version: 0,
        },
        1 << 20,
    )?;
    let mut u = fields(&g);
    let mut next = fields(&g);
    let mut workspace = ColumnShearWorkspace::new(g.clone(), Axis::Y, 1 << 20)?;
    let boundaries = [ColumnShearBoundary::NoSlip { velocity: [0.0; 3] }; 2];
    let forcing = UniformColumnForce {
        value: [c.value, 0.0, 0.0],
        units: c.units,
    };
    let modal = Modal::new(c);
    let mut ledger = Ledger::default();
    let mut csv = BufWriter::new(fs::File::create(root.join(format!("{}.csv", c.name)))?);
    writeln!(
        csv,
        "time,y,velocity,continuum_steady,discrete_steady,explicit_discrete_transient"
    )?;
    let units = match c.units {
        ForceUnits::Acceleration => "acceleration",
        ForceUnits::ForceDensity => "force_density",
    };
    let mut json = format!(
        "{{\"name\":\"{}\",\"layers\":{},\"rho\":{},\"mu\":{},\"value\":{},\"units\":\"{units}\",\"q\":{:.17e},\"acceleration\":{:.17e},\"continuum_flow\":{:.17e},\"snapshots\":[",
        c.name,
        c.n,
        c.rho,
        c.mu,
        c.value,
        c.q(),
        c.q() / c.rho,
        c.q() / (12.0 * c.mu)
    );
    snapshot(
        &mut json,
        &mut csv,
        Snapshot {
            c,
            g: &g,
            u: &u,
            modal: &modal,
            step: 0,
            energy: 0.0,
            momentum: 0.0,
            ledger: &ledger,
        },
    )?;
    let mut errors = (0.0, 0.0);
    let mut early_peak = 0.0;
    for step in 1..=STEPS {
        let forced = workspace.update_with_forcing(
            ColumnShearInputs {
                surface: surface.state(),
                velocity: refs(&u),
                density: c.rho,
                dynamic_viscosity: c.mu,
                dt: DT,
            },
            boundaries,
            forcing,
            muts(&mut next),
            |_| false,
        )?;
        let r = forced.boundary;
        ledger.bulk += DT * r.bulk_dissipation_before;
        ledger.wall_loss += DT * r.wall_dissipation_before;
        ledger.wall_work += r.actuator_work;
        ledger.body_work += forced.external_work_before;
        ledger.update += r.shear.update_energy;
        ledger.rounding += r.shear.rounding_work;
        ledger.body_impulse += forced.external_impulse[0];
        ledger.round_p += r.shear.rounding_momentum[0];
        ledger.identity_max = ledger.identity_max.max(r.shear.identity_error.abs());
        ledger.momentum_max = ledger.momentum_max.max(r.shear.momentum_error[0].abs());
        for side in 0..2 {
            ledger.wall_impulse[side] += r.wall_impulse[side][0];
        }
        std::mem::swap(&mut u, &mut next);
        assert_eq!(
            u[0][g.face_index(Axis::X, [0, 0, 0]).unwrap()],
            0.0,
            "lower trace changed; stop"
        );
        assert_eq!(
            u[0][g.face_index(Axis::X, [0, c.n - 1, 0]).unwrap()],
            0.0,
            "upper trace changed; stop"
        );
        if SAVED.contains(&step) {
            json.push(',');
            errors = snapshot(
                &mut json,
                &mut csv,
                Snapshot {
                    c,
                    g: &g,
                    u: &u,
                    modal: &modal,
                    step,
                    energy: r.shear.kinetic_after,
                    momentum: r.shear.momentum_after[0],
                    ledger: &ledger,
                },
            )?;
            if step == 683 {
                early_peak = errors.1;
            }
        }
    }
    csv.flush()?;
    let final_profile = (0..c.n)
        .map(|j| f64::from(u[0][g.face_index(Axis::X, [0, j, 0]).unwrap()]))
        .collect();
    write!(
        json,
        "],\"max_step_energy_residual\":{:.17e},\"max_step_momentum_residual\":{:.17e}}}",
        ledger.identity_max, ledger.momentum_max
    )?;
    if c.n == 16 {
        assert!(
            errors.0 < 0.031 * (c.q() / c.mu).abs() / 2.0,
            "continuum fixed-basis gate failed; stop"
        );
    }
    Ok(Run {
        json,
        final_profile,
        early_peak,
        continuum_error: errors.0,
    })
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
        .ok_or("existing ancestor required")?;
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
    let runs = CASES
        .into_iter()
        .map(|c| run(root, c))
        .collect::<Result<Vec<_>, _>>()?;
    assert!(
        runs[1].continuum_error < runs[0].continuum_error,
        "endpoint refinement gate failed; stop"
    );
    for j in 0..16 {
        assert!(
            (runs[1].final_profile[j] - runs[2].final_profile[j]).abs() < 2e-7,
            "unit equivalence gate failed; stop"
        );
        assert!(
            (runs[1].final_profile[j] - runs[3].final_profile[j]).abs() < 3e-5,
            "fixed-q density steady gate failed; stop"
        );
        assert!(
            (2.0 * runs[1].final_profile[j] - runs[4].final_profile[j]).abs() < 3e-5,
            "fixed-a density gate failed; stop"
        );
        assert!(
            (0.5 * runs[1].final_profile[j] - runs[5].final_profile[j]).abs() < 3e-5,
            "viscosity steady gate failed; stop"
        );
        assert!(
            (runs[1].final_profile[j] + runs[6].final_profile[j]).abs() < 2e-7,
            "signed forcing gate failed; stop"
        );
    }
    assert!(
        runs[3].early_peak < runs[1].early_peak && runs[4].early_peak > runs[1].early_peak,
        "density transient gate failed; stop"
    );
    let data = format!(
        "{{\"dt\":{DT:.17e},\"steps\":{STEPS},\"snapshot_playback\":true,\"cases\":[{}]}}",
        runs.iter()
            .map(|r| r.json.as_str())
            .collect::<Vec<_>>()
            .join(",")
    );
    fs::write(root.join("results.json"), &data)?;
    fs::write(
        root.join("index.html"),
        include_str!("column_poiseuille.html").replace("__NATIVE_DATA__", &data),
    )?;
    Ok(())
}
