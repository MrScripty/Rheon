//! Actual fields for the parent-admitted disposable S1 validation suite v1.
use rheon::{
    COMPRESSIBLE_METHOD, CompressibleSimulation, CompressibleStepReport, ConservativeState,
    GridGeometry, IdealGas,
};
use std::{
    error::Error,
    fs::{self, File, OpenOptions},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

const AMPLITUDE: f64 = 1e-6;
const MAX_STEPS: usize = 10_000;
const MAX_OUTPUT_BYTES: u64 = 2_000_000_000;

struct Fixture {
    id: String,
    kind: &'static str,
    initialization: &'static str,
    shape: [u64; 3],
    spacing: [f64; 3],
    origin: [f64; 3],
    target: f64,
}

fn output_argument() -> Result<PathBuf, Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 || args[0] != "--output" {
        return Err("usage: compressible_reference --output FRESH_DIRECTORY".into());
    }
    Ok(PathBuf::from(&args[1]))
}
fn fresh_file(path: &Path) -> Result<BufWriter<File>, Box<dyn Error>> {
    Ok(BufWriter::new(
        OpenOptions::new().write(true).create_new(true).open(path)?,
    ))
}
fn fixtures() -> Vec<Fixture> {
    let mut cases = vec![
        Fixture {
            id: "uniform".into(),
            kind: "uniform",
            initialization: "uniform-v1",
            shape: [3, 2, 1],
            spacing: [0.2, 0.3, 0.4],
            origin: [0.25, -0.5, 1.0],
            target: 0.01,
        },
        Fixture {
            id: "differential".into(),
            kind: "differential",
            initialization: "exported-initial-v1",
            shape: [3, 4, 2],
            spacing: [0.2, 0.3, 0.4],
            origin: [0.25, -0.5, 1.0],
            target: 0.001,
        },
    ];
    for n in [8, 12, 20] {
        cases.push(Fixture {
            id: format!("acoustic-{n}"),
            kind: "acoustic",
            initialization: "linear-conservative-cell-average-v1",
            shape: [n; 3],
            spacing: [1.0 / n as f64; 3],
            origin: [0.0; 3],
            target: 0.08,
        });
    }
    for n in [24, 48, 64] {
        cases.push(Fixture {
            id: format!("sod-{n}"),
            kind: "sod",
            initialization: "sod-cell-average-v1",
            shape: [n, 2, 2],
            spacing: [1.0 / n as f64, 0.5, 0.5],
            origin: [0.0; 3],
            target: 0.08,
        });
    }
    cases
}

// This is the admitted linear conservative initial data, not a continuum
// evolution oracle or a copied flux. Each sinusoid's average is analytic.
fn acoustic_average(center: [f64; 3], spacing: [f64; 3], eos: IdealGas) -> ConservativeState {
    let modes = [[1.0, 1.0, 1.0], [1.0, -1.0, 2.0]];
    let weights = [1.0, 0.5];
    let phases = [0.0, std::f64::consts::PI / 3.0];
    let mut u = [1.0, 0.0, 0.0, 0.0, 1.0 / (eos.gamma() - 1.0)];
    for mode in 0..2 {
        let k = modes[mode];
        let norm = k.iter().map(|v| v * v).sum::<f64>().sqrt();
        let phase =
            std::f64::consts::TAU * (0..3).map(|d| k[d] * center[d]).sum::<f64>() + phases[mode];
        let factor = (0..3)
            .map(|d| {
                let q = std::f64::consts::PI * k[d] * spacing[d];
                if q == 0.0 { 1.0 } else { q.sin() / q }
            })
            .product::<f64>();
        let wave = AMPLITUDE * weights[mode] * factor * phase.sin();
        u[0] += wave;
        for d in 0..3 {
            u[d + 1] += eos.gamma().sqrt() * wave * k[d] / norm;
        }
        u[4] += eos.gamma() / (eos.gamma() - 1.0) * wave;
    }
    u
}

fn initialize(f: &Fixture, center: [f64; 3], eos: IdealGas) -> ConservativeState {
    match f.kind {
        "uniform" => eos
            .conservative(1.7, [0.2, -0.4, 0.3], 0.9)
            .expect("fixed finite fixture"),
        "differential" => {
            let angle: [f64; 3] = std::array::from_fn(|d| {
                std::f64::consts::TAU * (center[d] - f.origin[d])
                    / (f.shape[d] as f64 * f.spacing[d])
            });
            let waves = angle.map(f64::sin);
            let rho = 1.0 + 0.03 * waves[0] + 0.02 * waves[1] + 0.01 * waves[2];
            let p = 1.0 + 0.04 * waves[2] + 0.02 * waves[0];
            let velocity = [
                0.2 + 0.07 * waves[1],
                -0.1 + 0.05 * waves[2],
                0.15 + 0.03 * waves[0],
            ];
            eos.conservative(rho, velocity, p)
                .expect("fixed bounded fixture")
        }
        "acoustic" => acoustic_average(center, f.spacing, eos),
        "sod" => {
            if center[0] < 0.5 {
                [1.0, 0.0, 0.0, 0.0, 1.0 / (eos.gamma() - 1.0)]
            } else {
                [0.125, 0.0, 0.0, 0.0, 0.1 / (eos.gamma() - 1.0)]
            }
        }
        _ => unreachable!("owned fixed fixture population"),
    }
}

fn fields(
    root: &Path,
    name: &str,
    state: &[ConservativeState],
    bytes: &mut u64,
) -> Result<(), Box<dyn Error>> {
    let upper = state.len() as u64 * 150 + 64;
    if bytes.checked_add(upper).ok_or("output size overflow")? > MAX_OUTPUT_BYTES {
        return Err("bounded output byte budget exhausted; partial files retained".into());
    }
    *bytes += upper;
    let mut out = fresh_file(&root.join(name))?;
    writeln!(out, "rho,mx,my,mz,E")?;
    for u in state {
        writeln!(
            out,
            "{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
            u[0], u[1], u[2], u[3], u[4]
        )?;
    }
    out.flush()?;
    Ok(())
}

fn array(out: &mut impl Write, values: ConservativeState) -> Result<(), Box<dyn Error>> {
    write!(out, "[")?;
    for (i, value) in values.into_iter().enumerate() {
        write!(out, "{}{value:.17e}", if i == 0 { "" } else { "," })?;
    }
    write!(out, "]")?;
    Ok(())
}

fn frame(
    out: &mut impl Write,
    sim: &CompressibleSimulation,
    file: &str,
    report: Option<&CompressibleStepReport>,
) -> Result<(), Box<dyn Error>> {
    let v = sim.view();
    let requested = report.map_or(0.0, |r| r.requested_dt);
    let dt = report.map_or(0.0, |r| r.accepted_dt);
    write!(
        out,
        "{{\"generation\":{},\"time_s\":{:.17e},\"requested_dt_s\":{requested:.17e},\"accepted_dt_s\":{dt:.17e},\"file\":\"{file}\",\"diagnostics\":{{\"totals\":",
        v.generation, v.time
    )?;
    array(out, v.diagnostics.totals)?;
    let d = v.diagnostics;
    write!(
        out,
        ",\"min_density\":{:.17e},\"min_pressure\":{:.17e},\"min_internal_energy_density\":{:.17e},\"min_temperature\":{:.17e},\"max_speed\":{:.17e},\"max_mach\":{:.17e}}}",
        d.min_density,
        d.min_pressure,
        d.min_internal_energy_density,
        d.min_temperature,
        d.max_speed,
        d.max_mach
    )?;
    if let Some(r) = report {
        write!(
            out,
            ",\"report\":{{\"incident_acoustic_rate\":{:.17e},\"cfl_bound_s\":{:.17e},\"controlling_cell\":{},\"attempts\":{},\"rejections\":{},\"flux_fallbacks\":{},\"host_step_seconds\":{:.17e},\"host_kernel_seconds\":{:.17e},\"ledger_residual\":",
            r.incident_acoustic_rate,
            r.cfl_bound,
            r.controlling_cell,
            r.attempts,
            r.rejections,
            r.flux_fallbacks,
            r.host_step_seconds,
            r.host_kernel_seconds
        )?;
        array(out, r.ledger_residual)?;
        write!(out, ",\"initial_total_drift\":")?;
        array(out, r.initial_total_drift)?;
        write!(out, ",\"last_rejection\":")?;
        if let Some(c) = r.last_rejection {
            write!(
                out,
                "{{\"stage\":\"{:?}\",\"cell\":{},\"component\":{},\"cause\":\"{:?}\"}}",
                c.stage,
                c.cell.map_or_else(|| "null".into(), |v| v.to_string()),
                c.component.map_or_else(|| "null".into(), |v| v.to_string()),
                c.cause
            )?;
        } else {
            write!(out, "null")?;
        }
        write!(out, "}}")?;
    }
    write!(out, "}}")?;
    Ok(())
}

fn run_case(
    root: &Path,
    f: &Fixture,
    eos: IdealGas,
    manifest: &mut impl Write,
    bytes: &mut u64,
) -> Result<(), Box<dyn Error>> {
    let grid = GridGeometry::new(f.shape, f.spacing, f.origin)?;
    let mut sim = CompressibleSimulation::from_fn(grid, eos, 0.5, 64 * 1024 * 1024, |_, p| {
        initialize(f, p, eos)
    })?;
    fs::create_dir(root.join(&f.id))?;
    write!(
        manifest,
        "{{\"id\":\"{}\",\"kind\":\"{}\",\"shape\":{:?},\"spacing_m\":{:?},\"origin_m\":{:?},\"boundary\":\"periodic\",\"layout\":\"x-fast\",\"initialization\":\"{}\",\"cfl\":0.5,\"target_time_s\":{:.17e},\"managed_capacity_bytes\":{},\"frames\":[",
        f.id,
        f.kind,
        f.shape,
        f.spacing,
        f.origin,
        f.initialization,
        f.target,
        sim.allocated_bytes()
    )?;
    let mut ordinal = 0;
    let name = format!("{}/state-{ordinal:06}.csv", f.id);
    fields(root, &name, sim.view().conservative, bytes)?;
    frame(manifest, &sim, &name, None)?;
    while sim.view().time < f.target {
        if ordinal == MAX_STEPS {
            return Err("bounded reference exhausted step budget; partial files retained".into());
        }
        let report = sim.step(f.target - sim.view().time, |_| false)?;
        ordinal += 1;
        let name = format!("{}/state-{ordinal:06}.csv", f.id);
        fields(root, &name, sim.view().conservative, bytes)?;
        write!(manifest, ",")?;
        frame(manifest, &sim, &name, Some(&report))?;
    }
    write!(manifest, "]}}")?;
    println!(
        "{}: {ordinal} accepted steps, t={:.17e}, {} managed bytes",
        f.id,
        sim.view().time,
        sim.allocated_bytes()
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let output = output_argument()?;
    // Acquire a fresh output directory; never overwrite previous artifacts.
    fs::create_dir(&output)?;
    let eos = IdealGas::new(1.4, 287.0)?;
    let staged = output.join("suite.partial.json");
    let mut manifest = fresh_file(&staged)?;
    write!(
        manifest,
        "{{\"schema_version\":1,\"method\":\"{}\",\"gamma\":1.4,\"gas_constant\":287.0,\"units\":[\"kg/m^3\",\"kg/(m^2*s)\",\"kg/(m^2*s)\",\"kg/(m^2*s)\",\"J/m^3\"],\"max_steps_per_case\":{},\"max_raw_field_bytes\":{},\"cases\":[",
        COMPRESSIBLE_METHOD, MAX_STEPS, MAX_OUTPUT_BYTES
    )?;
    let mut bytes = 0;
    for (i, fixture) in fixtures().iter().enumerate() {
        if i > 0 {
            write!(manifest, ",")?;
        }
        run_case(&output, fixture, eos, &mut manifest, &mut bytes)?;
    }
    writeln!(manifest, "]}}")?;
    manifest.flush()?;
    drop(manifest);
    // The suite marker appears last. Partial output remains on any earlier error.
    fs::rename(staged, output.join("suite.json"))?;
    Ok(())
}
