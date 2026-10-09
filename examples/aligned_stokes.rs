//! Explicitly opted-in native Stokes experiment; fresh records outside Git.
//! Usage: aligned_stokes [CASE [STEPS(1..3)]] NEW_EXTERNAL_JSON
//! The exporter has its own small caller/export buffers outside Workspace's cap.
#[allow(dead_code)]
#[path = "../experiments/aligned_stokes.rs"]
mod aligned_stokes;
use aligned_stokes::{ComponentCertificate, Config, Interval, Report, Workspace};
use rheon::{
    AlignedStrain, AlignedStrainBoundary, Axis, GridGeometry, StaticObstacleGeometry,
    SurfaceSettings, SurfaceStamp, TriangleSurface,
};
use std::{
    error::Error,
    io::{BufWriter, Write},
    path::Path,
};
fn axis(a: Axis) -> usize {
    match a {
        Axis::X => 0,
        Axis::Y => 1,
        Axis::Z => 2,
    }
}
fn bits(out: &mut impl Write, x: f64) -> std::io::Result<()> {
    write!(out, "\"{:016x}\"", x.to_bits())
}
fn numbers(out: &mut impl Write, a: &[f64]) -> std::io::Result<()> {
    write!(out, "[")?;
    for (i, &x) in a.iter().enumerate() {
        if i > 0 {
            write!(out, ",")?;
        }
        bits(out, x)?;
    }
    write!(out, "]")
}
fn full(out: &mut impl Write, a: &[Vec<f64>; 3]) -> std::io::Result<()> {
    write!(out, "[")?;
    for (i, x) in a.iter().enumerate() {
        if i > 0 {
            write!(out, ",")?;
        }
        numbers(out, x)?;
    }
    write!(out, "]")
}
fn interval(out: &mut impl Write, x: Interval) -> std::io::Result<()> {
    numbers(out, &[x.lo, x.hi])
}
fn string(out: &mut impl Write, s: &str) -> std::io::Result<()> {
    write!(out, "\"")?;
    for c in s.chars() {
        match c {
            '"' => write!(out, "\\\"")?,
            '\\' => write!(out, "\\\\")?,
            '\n' => write!(out, "\\n")?,
            '\r' => write!(out, "\\r")?,
            '\t' => write!(out, "\\t")?,
            _ => write!(out, "{c}")?,
        }
    }
    write!(out, "\"")
}
fn certificates(out: &mut impl Write, c: &[ComponentCertificate]) -> std::io::Result<()> {
    write!(out, "[")?;
    for (i, c) in c.iter().enumerate() {
        if i > 0 {
            write!(out, ",")?;
        }
        write!(out, "{{\"defect\":")?;
        interval(out, c.defect)?;
        write!(out, ",\"scale\":")?;
        interval(out, c.scale)?;
        write!(out, ",\"allowance\":")?;
        interval(out, c.allowance)?;
        write!(out, "}}")?;
    }
    write!(out, "]")
}
fn report(out: &mut impl Write, w: &Workspace<'_, '_>, r: &Report) -> std::io::Result<()> {
    write!(out, "{{\"bound\":")?;
    interval(out, r.bound)?;
    write!(out, ",\"step_product\":")?;
    interval(out, r.step_product)?;
    write!(out, ",\"viscous_energy_delta\":")?;
    interval(out, r.viscous_energy_delta)?;
    write!(out, ",\"pressure_energy_delta\":")?;
    interval(out, r.pressure_energy_delta)?;
    write!(out, ",\"initial_divergence\":")?;
    interval(out, r.initial_divergence.bound)?;
    write!(out, ",\"final_divergence\":")?;
    interval(out, r.final_divergence.bound)?;
    write!(out, ",\"update\":")?;
    certificates(out, w.update_certificates().unwrap())?;
    write!(out, ",\"momentum\":")?;
    certificates(out, w.momentum_certificates().unwrap())?;
    write!(out, "}}")
}
fn geometry(out: &mut impl Write, op: &AlignedStrain<'_>) -> std::io::Result<()> {
    let g = op.geometry().grid();
    let (lo, hi) = op.box_plane_indices();
    write!(out, "{{\"counts\":{:?},\"spacing\":", g.counts())?;
    numbers(out, &g.spacing())?;
    write!(out, ",\"origin\":")?;
    numbers(out, &g.origin())?;
    write!(out, ",\"lower\":{lo:?},\"upper\":{hi:?},\"density\":")?;
    bits(out, op.density())?;
    write!(out, ",\"viscosity\":")?;
    bits(out, op.viscosity())?;
    write!(out, ",\"volumes\":")?;
    numbers(out, op.geometry().fluid_volumes())?;
    write!(out, "}}")
}
fn settings(out: &mut impl Write, c: Config, dt: f64) -> std::io::Result<()> {
    write!(out, "{{\"dt\":")?;
    bits(out, dt)?;
    write!(out, ",\"divergence_limit\":")?;
    bits(out, c.divergence_limit)?;
    write!(out, ",\"relative_update_limit\":")?;
    bits(out, c.relative_update_limit)?;
    write!(out, ",\"pressure_relative_residual\":")?;
    bits(out, c.pressure.relative_residual)?;
    write!(out, ",\"pressure_absolute_residual\":")?;
    bits(out, c.pressure.absolute_residual)?;
    write!(
        out,
        ",\"pressure_max_iterations\":{}}}",
        c.pressure.max_iterations
    )
}
fn operator(out: &mut impl Write, op: &AlignedStrain<'_>) -> std::io::Result<()> {
    write!(out, "\"active\":[")?;
    for (i, f) in op.active_faces().iter().enumerate() {
        if i > 0 {
            write!(out, ",")?;
        }
        write!(
            out,
            "{{\"id\":{i},\"axis\":{},\"face\":{},\"coordinates\":{:?},\"position\":",
            axis(f.axis),
            f.face,
            f.coordinates
        )?;
        numbers(out, &f.position)?;
        write!(out, ",\"area\":")?;
        bits(out, f.area)?;
        write!(out, ",\"distance\":")?;
        bits(out, f.distance)?;
        write!(out, ",\"mass\":")?;
        bits(out, f.mass)?;
        write!(
            out,
            ",\"negative_cell\":{},\"positive_cell\":{}}}",
            f.negative_cell, f.positive_cell
        )?;
    }
    write!(out, "],\"rows\":[")?;
    for (i, r) in op.rows().iter().enumerate() {
        if i > 0 {
            write!(out, ",")?;
        }
        let boundary = match r.boundary {
            AlignedStrainBoundary::Interior => 0,
            AlignedStrainBoundary::ObstacleFlat => 1,
            AlignedStrainBoundary::ObstacleCorner => 2,
            AlignedStrainBoundary::OuterFreeSlip => 3,
            AlignedStrainBoundary::Normal => 4,
        };
        write!(
            out,
            "{{\"id\":{i},\"axes\":[{},{}],\"coordinates\":{:?},\"quadrant\":{},\"boundary\":{boundary},\"weight\":",
            axis(r.axes[0]),
            axis(r.axes[1]),
            r.coordinates,
            r.quadrant
        )?;
        bits(out, r.weight)?;
        write!(out, ",\"terms\":[")?;
        for (j, t) in r.terms().iter().enumerate() {
            if j > 0 {
                write!(out, ",")?;
            }
            write!(out, "[{},", t.active)?;
            bits(out, t.coefficient)?;
            write!(out, "]")?;
        }
        write!(out, "]}}")?;
    }
    write!(out, "]")
}
fn zeros(o: &StaticObstacleGeometry) -> [Vec<f64>; 3] {
    std::array::from_fn(|d| vec![0.0; o.grid().face_len([Axis::X, Axis::Y, Axis::Z][d])])
}
fn expanded(op: &AlignedStrain<'_>, v: &[f64]) -> [Vec<f64>; 3] {
    let mut fields = zeros(op.geometry());
    for (f, &v) in op.active_faces().iter().zip(v) {
        fields[axis(f.axis)][f.face] = v;
    }
    fields
}
fn accepted(
    out: &mut impl Write,
    w: &Workspace<'_, '_>,
    initial: &[Vec<f64>; 3],
    final_field: &[Vec<f64>; 3],
    index: usize,
) -> std::io::Result<()> {
    let op = w.operator();
    let r = w.last_report().unwrap();
    write!(
        out,
        "{{\"schema\":\"rheon-aligned-stokes-native-v1\",\"status\":\"accepted\",\"geometry\":"
    )?;
    geometry(out, op)?;
    write!(out, ",\"settings\":")?;
    settings(out, w.config(), r.dt)?;
    write!(out, ",")?;
    operator(out, op)?;
    write!(
        out,
        ",\"gauge_cells\":{:?},\"allocation\":{{\"operator_bytes\":{},\"workspace_bytes\":{},\"combined_bytes\":{},\"limit\":4000000}},\"initial\":",
        w.gauge_cells(),
        op.allocated_bytes(),
        w.allocated_bytes(),
        w.combined_bytes()
    )?;
    full(out, initial)?;
    write!(out, ",\"viscous\":")?;
    full(out, &expanded(op, w.viscous_active().unwrap()))?;
    write!(out, ",\"final\":")?;
    full(out, final_field)?;
    write!(out, ",\"pressure\":")?;
    numbers(out, w.pressure().unwrap())?;
    write!(out, ",\"report\":")?;
    report(out, w, r)?;
    write!(
        out,
        ",\"state_before\":{{\"accepted_steps\":{index}}},\"state_after\":{{\"accepted_steps\":{}}}}}",
        index + 1
    )
}
fn witness(out: &mut impl Write, w: &Workspace<'_, '_>) -> std::io::Result<()> {
    write!(out, "{{\"initial\":")?;
    numbers(out, w.initial_active().unwrap())?;
    write!(out, ",\"viscous\":")?;
    numbers(out, w.viscous_active().unwrap())?;
    write!(out, ",\"final\":")?;
    numbers(out, w.final_active().unwrap())?;
    write!(out, ",\"pressure\":")?;
    numbers(out, w.pressure().unwrap())?;
    write!(out, ",\"report\":")?;
    report(out, w, w.last_report().unwrap())?;
    write!(out, "}}")
}
fn owner(h: [f64; 3], origin: [f64; 3]) -> Result<StaticObstacleGeometry, Box<dyn Error>> {
    let lo: [f64; 3] = std::array::from_fn(|d| origin[d] + h[d]);
    let hi: [f64; 3] = std::array::from_fn(|d| origin[d] + 2.0 * h[d]);
    let mesh = TriangleSurface::new(
        SurfaceStamp {
            id: 119,
            version: 1,
        },
        (0..8)
            .map(|c| std::array::from_fn(|d| if c & (1 << d) == 0 { lo[d] } else { hi[d] }))
            .collect(),
        vec![
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
        ],
        SurfaceSettings::default(),
    )?;
    Ok(StaticObstacleGeometry::new(
        GridGeometry::new([3; 3], h, origin)?,
        mesh,
        4_000_000,
        |_, _| false,
    )?)
}
fn curl(o: &StaticObstacleGeometry, amplitude: f64, case: &str) -> [Vec<f64>; 3] {
    let mut u = zeros(o);
    let samples = if case == "unit-cross" {
        [
            (Axis::X, [1, 0, 0], 0.5),
            (Axis::X, [1, 0, 1], -0.5),
            (Axis::Z, [0, 0, 1], -0.5),
            (Axis::Z, [1, 0, 1], 0.5),
        ]
    } else if case == "unit-reflected" {
        [
            (Axis::X, [2, 1, 2], 0.5),
            (Axis::X, [2, 2, 2], -0.5),
            (Axis::Y, [1, 2, 2], -0.5),
            (Axis::Y, [2, 2, 2], 0.5),
        ]
    } else {
        [
            (Axis::X, [1, 0, 0], 0.5),
            (Axis::X, [1, 1, 0], -0.5),
            (Axis::Y, [0, 1, 0], -0.5),
            (Axis::Y, [1, 1, 0], 0.5),
        ]
    };
    for (a, p, q) in samples {
        let f = o.grid().face_index(a, p).unwrap();
        u[axis(a)][f] = amplitude * q / o.open_areas(a)[f];
    }
    u
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args.len() > 3 {
        return Err("usage: aligned_stokes [CASE [STEPS(1..3)]] NEW_EXTERNAL_JSON".into());
    }
    let case = if args.len() == 1 {
        "unit-curl"
    } else {
        args[0].as_str()
    };
    let steps = if args.len() == 3 {
        args[1].parse::<usize>()?
    } else {
        1
    };
    if !(1..=3).contains(&steps) {
        return Err("steps must be between1and3".into());
    }
    let refused = matches!(
        case,
        "unit-tinystep" | "unit-boundary" | "unit-oversized" | "unit-underflow" | "unit-overflow"
    );
    if !matches!(
        case,
        "unit-curl"
            | "unit-rest"
            | "anisotropic-curl"
            | "unit-tiny"
            | "unit-large"
            | "unit-tinystep"
            | "unit-boundary"
            | "unit-oversized"
            | "unit-underflow"
            | "unit-overflow"
            | "unit-cross"
            | "unit-reflected"
    ) {
        return Err("unknown specimen case".into());
    }
    if refused && steps != 1 {
        return Err("refusal cases require one requested step".into());
    }
    let path = Path::new(args.last().unwrap());
    let parent = path.parent().unwrap_or(Path::new(".")).canonicalize()?;
    if std::process::Command::new("git")
        .arg("-C")
        .arg(parent)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()?
        .status
        .success()
    {
        return Err("generated records must be outside every Git checkout".into());
    }
    let (h, origin, rho, mu, mut dt) = if case == "anisotropic-curl" {
        (
            [0.3, 0.7, 1.1],
            [100_000_000.0, -100_000_000.0, 0.1],
            3.7,
            0.375,
            1.0 / 256.0,
        )
    } else {
        ([1.0; 3], [0.0; 3], 1.0, 1.0, 1.0 / 32.0)
    };
    let amplitude = match case {
        "unit-tiny" => f64::from_bits((1023 - 200) << 52),
        "unit-large" => f64::from_bits((1023 + 200) << 52),
        "unit-underflow" => f64::from_bits((1023 - 600) << 52),
        "unit-overflow" => f64::from_bits((1023 + 600) << 52),
        _ => 1.0,
    };
    if case == "unit-tinystep" {
        dt = f64::from_bits((1023 - 200) << 52);
    } else if case == "unit-boundary" {
        dt = (2.0_f64 / 17.0).next_up();
    } else if case == "unit-oversized" {
        dt = 0.5;
    }
    let o = owner(h, origin)?;
    let op = AlignedStrain::new(&o, rho, mu, 4_000_000, |_, _| false)?;
    let mut config = Config::default();
    if matches!(case, "unit-tiny" | "unit-large") {
        config.divergence_limit = amplitude * 1e-10;
        config.pressure.divergence_limit = config.divergence_limit;
        config.pressure.absolute_residual = 0.0;
    }
    let mut w = Workspace::new(&op, config, 4_000_000, |_, _| false)?;
    let mut out = Vec::new();
    if refused {
        let mut seed = curl(&o, 1.0, "unit-curl");
        let [x, y, z] = &mut seed;
        w.step([x, y, z], 1.0 / 32.0, |_, _| false)?;
        let mut before_witness = Vec::new();
        witness(&mut before_witness, &w)?;
        let mut u = curl(&o, amplitude, "unit-curl");
        let before = u.clone();
        let [x, y, z] = &mut u;
        let error = w
            .step([x, y, z], dt, |_, _| false)
            .err()
            .ok_or("refusal fixture unexpectedly accepted")?;
        write!(
            out,
            "{{\"schema\":\"rheon-aligned-stokes-refusal-v1\",\"case\":"
        )?;
        string(&mut out, case)?;
        write!(out, ",\"status\":\"refused\",\"error\":")?;
        string(&mut out, &format!("{error:?}"))?;
        write!(out, ",\"caller_before\":")?;
        full(&mut out, &before)?;
        write!(out, ",\"caller_after\":")?;
        full(&mut out, &u)?;
        write!(out, ",\"accepted_before\":")?;
        out.write_all(&before_witness)?;
        write!(out, ",\"accepted_after\":")?;
        witness(&mut out, &w)?;
        write!(
            out,
            ",\"state_before\":{{\"accepted_steps\":1}},\"state_after\":{{\"accepted_steps\":1}}}}"
        )?;
    } else {
        let mut u = if case == "unit-rest" {
            zeros(&o)
        } else {
            curl(&o, amplitude, case)
        };
        if steps > 1 {
            write!(
                out,
                "{{\"schema\":\"rheon-aligned-stokes-sequence-v1\",\"frames\":["
            )?;
        }
        for i in 0..steps {
            let before = u.clone();
            let [x, y, z] = &mut u;
            w.step([x, y, z], dt, |_, _| false)?;
            if i > 0 {
                write!(out, ",")?;
            }
            accepted(&mut out, &w, &before, &u, i)?;
        }
        if steps > 1 {
            write!(out, "]}}")?;
        }
    }
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    let mut file = BufWriter::new(file);
    file.write_all(&out)?;
    file.write_all(b"\n")?;
    file.flush()?;
    Ok(())
}
