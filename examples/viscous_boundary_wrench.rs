//! Bounded native variational viscous-wrench fixture; no stepping or pressure.
use rheon::{
    AlignedStrain, AlignedStrainBoundary, AlignedViscousBoundaryWrench, Axis, GridGeometry,
    StaticObstacleGeometry, SurfaceSettings, SurfaceStamp, TriangleSurface,
};
use std::{error::Error, io::Write, path::Path};
pub fn axis(a: Axis) -> usize {
    match a {
        Axis::X => 0,
        Axis::Y => 1,
        Axis::Z => 2,
    }
}
pub const CAP: usize = 16_000_000;
pub const SOLID_TWIST: [f64; 6] = [0.25, -0.5, 0.125, 0.75, -0.25, 0.5];
pub const OUTER_TWIST: [f64; 6] = [-0.5, 0.25, 0.75, -0.125, 0.5, -0.25];
pub fn geometry(
    n: [u64; 3],
    h: [f64; 3],
    origin: [f64; 3],
    lo: [usize; 3],
    hi: [usize; 3],
) -> Result<StaticObstacleGeometry, Box<dyn Error>> {
    geometry_with_stamp(n, h, origin, lo, hi, SurfaceStamp { id: 73, version: 1 })
}
pub fn geometry_with_stamp(
    n: [u64; 3],
    h: [f64; 3],
    origin: [f64; 3],
    lo: [usize; 3],
    hi: [usize; 3],
    stamp: SurfaceStamp,
) -> Result<StaticObstacleGeometry, Box<dyn Error>> {
    let lower: [f64; 3] = std::array::from_fn(|d| origin[d] + lo[d] as f64 * h[d]);
    let upper: [f64; 3] = std::array::from_fn(|d| origin[d] + hi[d] as f64 * h[d]);
    let surface = TriangleSurface::new(
        stamp,
        (0..8)
            .map(|c| {
                std::array::from_fn(|d| {
                    if c & (1 << d) == 0 {
                        lower[d]
                    } else {
                        upper[d]
                    }
                })
            })
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
        GridGeometry::new(n, h, origin)?,
        surface,
        CAP,
        |_, _| false,
    )?)
}
pub fn rigid_component(axis: usize, x: [f64; 3], reference: [f64; 3], twist: [f64; 6]) -> f64 {
    let r = std::array::from_fn::<_, 3, _>(|d| x[d] - reference[d]);
    twist[axis]
        + match axis {
            0 => twist[4] * r[2] - twist[5] * r[1],
            1 => twist[5] * r[0] - twist[3] * r[2],
            _ => twist[3] * r[1] - twist[4] * r[0],
        }
}
pub fn fields(op: &AlignedStrain<'_>, reference: [f64; 3]) -> Vec<(String, Vec<f64>)> {
    let n = op.active_faces().len();
    let matrix = [
        [0.25, 0.5, -0.125],
        [-0.25, -0.5, 0.375],
        [0.125, -0.25, 0.5],
    ];
    let mut fields = vec![
        ("rest".into(), vec![0.0; n]),
        (
            "translation".into(),
            op.active_faces()
                .iter()
                .map(|f| if f.axis == Axis::X { 1.0 } else { 0.0 })
                .collect(),
        ),
        (
            "affine-strain".into(),
            op.active_faces()
                .iter()
                .map(|f| {
                    (0..3)
                        .map(|d| matrix[axis(f.axis)][d] * (f.position[d] - reference[d]))
                        .sum()
                })
                .collect(),
        ),
        (
            "rigid-sample".into(),
            op.active_faces()
                .iter()
                .map(|f| {
                    rigid_component(
                        axis(f.axis),
                        f.position,
                        reference,
                        [0.0, 0.0, 0.0, 0.5, -0.25, 0.125],
                    )
                })
                .collect(),
        ),
        (
            "crosscomponent".into(),
            (0..n)
                .map(|i| (((i * 17 + 3) % 29) as i32 - 14) as f64 / 16.0)
                .collect(),
        ),
    ];
    let g = op.geometry().grid();
    let edge = [
        g.origin()[0] + g.spacing()[0],
        g.origin()[1] + g.spacing()[1],
    ];
    let mut local = vec![0.0; n];
    for (axis, p) in [
        (Axis::X, [1, 0, 0]),
        (Axis::X, [1, 1, 0]),
        (Axis::Y, [0, 1, 0]),
        (Axis::Y, [1, 1, 0]),
    ] {
        if let Some(id) = g.face_index(axis, p).and_then(|f| op.active_index(axis, f)) {
            let f = &op.active_faces()[id];
            local[id] = if axis == Axis::X {
                -(f.position[1] - edge[1])
            } else {
                f.position[0] - edge[0]
            };
        }
    }
    fields.push(("compatible-local-rotation".into(), local));
    let (lower, upper) = op.geometry().box_bounds();
    let polynomial = op
        .active_faces()
        .iter()
        .map(|f| {
            let x = f.position;
            let p = std::array::from_fn::<_, 3, _>(|d| (x[d] - lower[d]) * (x[d] - upper[d]));
            match f.axis {
                Axis::X => {
                    2.0 * p[0] * p[0] * p[1] * (2.0 * x[1] - lower[1] - upper[1]) * p[2] * p[2]
                }
                Axis::Y => {
                    -2.0 * p[0] * (2.0 * x[0] - lower[0] - upper[0]) * p[1] * p[1] * p[2] * p[2]
                }
                Axis::Z => 0.0,
            }
        })
        .collect();
    fields.push(("polynomial-curl".into(), polynomial));
    let center_x = lower[0] + 0.5 * (upper[0] - lower[0]);
    let tilted = op
        .active_faces()
        .iter()
        .map(|f| {
            let x = f.position;
            let p = std::array::from_fn::<_, 3, _>(|d| (x[d] - lower[d]) * (x[d] - upper[d]));
            let tilt = 1.0 + (x[0] - center_x);
            match f.axis {
                Axis::X => {
                    225.0
                        * tilt
                        * p[0]
                        * p[0]
                        * 2.0
                        * p[1]
                        * (2.0 * x[1] - lower[1] - upper[1])
                        * p[2]
                        * p[2]
                }
                Axis::Y => {
                    -225.0
                        * (p[0] * p[0] + 2.0 * tilt * p[0] * (2.0 * x[0] - lower[0] - upper[0]))
                        * p[1]
                        * p[1]
                        * p[2]
                        * p[2]
                }
                Axis::Z => 0.0,
            }
        })
        .collect();
    fields.push(("polynomial-tilted-curl".into(), tilted));
    for (name, [a, b, c]) in [
        ("polynomial-tilted-curl-yz", [1, 2, 0]),
        ("polynomial-tilted-curl-zx", [2, 0, 1]),
    ] {
        let center = lower[a] + 0.5 * (upper[a] - lower[a]);
        let values = op
            .active_faces()
            .iter()
            .map(|f| {
                let x = f.position;
                let p = std::array::from_fn::<_, 3, _>(|d| (x[d] - lower[d]) * (x[d] - upper[d]));
                let tilt = 1.0 + (x[a] - center);
                let component = axis(f.axis);
                if component == a {
                    225.0
                        * tilt
                        * p[a]
                        * p[a]
                        * 2.0
                        * p[b]
                        * (2.0 * x[b] - lower[b] - upper[b])
                        * p[c]
                        * p[c]
                } else if component == b {
                    -225.0
                        * (p[a] * p[a] + 2.0 * tilt * p[a] * (2.0 * x[a] - lower[a] - upper[a]))
                        * p[b]
                        * p[b]
                        * p[c]
                        * p[c]
                } else {
                    0.0
                }
            })
            .collect();
        fields.push((name.into(), values));
    }
    let mut seen = std::collections::BTreeSet::new();
    for row in op.rows() {
        if row.boundary == AlignedStrainBoundary::ObstacleCorner
            && row.terms().len() == 2
            && seen.insert((axis(row.axes[0]), axis(row.axes[1]), row.coordinates))
        {
            let mut u = vec![0.0; n];
            u[row.terms()[0].active] = 0.625;
            u[row.terms()[1].active] = -0.375;
            let p = row.coordinates;
            fields.push((
                format!(
                    "corner-{}-{}-{}-{}-{}",
                    axis(row.axes[0]),
                    axis(row.axes[1]),
                    p[0],
                    p[1],
                    p[2]
                ),
                u,
            ));
        }
    }
    fields
}
fn bits(out: &mut impl Write, x: f64) -> std::io::Result<()> {
    write!(out, "\"{:016x}\"", x.to_bits())
}
fn values(out: &mut impl Write, x: &[f64]) -> std::io::Result<()> {
    write!(out, "[")?;
    for (i, &v) in x.iter().enumerate() {
        if i != 0 {
            write!(out, ",")?;
        }
        bits(out, v)?;
    }
    write!(out, "]")
}
fn external(path: &Path) -> Result<(), Box<dyn Error>> {
    let parent = path.parent().unwrap_or(Path::new(".")).canonicalize()?;
    if std::process::Command::new("git")
        .arg("-C")
        .arg(parent)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()?
        .status
        .success()
    {
        return Err("native output must be outside Git".into());
    }
    Ok(())
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: viscous_boundary_wrench CASE NEW_EXTERNAL_JSON".into());
    }
    let (n, h, origin, lo, hi, rho, mu) = match args[0].as_str() {
        "unit-center" | "reference-shift" | "polynomial-coarse" => {
            ([3; 3], [1.0; 3], [0.0; 3], [1; 3], [2; 3], 1.0, 1.0)
        }
        "polynomial-bounded" => ([6; 3], [0.5; 3], [0.0; 3], [2; 3], [4; 3], 1.0, 1.0),
        "anisotropic" => (
            [4, 5, 3],
            [0.5, 0.75, 1.25],
            [0.0; 3],
            [1, 2, 1],
            [3, 3, 2],
            2.0,
            0.375,
        ),
        "translated-nonmidpoint" => (
            [3; 3],
            [0.3, 0.7, 1.1],
            [1e8, -1e8, 0.1],
            [1; 3],
            [2; 3],
            3.7,
            0.375,
        ),
        "translated-large" => (
            [3; 3],
            [0.3, 0.7, 1.1],
            [1e12, -1e12, 0.1],
            [1; 3],
            [2; 3],
            3.7,
            0.375,
        ),
        _ => return Err("unknown bounded fixture case".into()),
    };
    let path = Path::new(&args[1]);
    external(path)?;
    let owner = geometry(n, h, origin, lo, hi)?;
    let op = AlignedStrain::new(&owner, rho, mu, CAP, |_, _| false)?;
    let (lower, upper) = owner.box_bounds();
    let reference = if args[0] == "reference-shift" {
        [-0.25, 2.5, 0.75]
    } else {
        std::array::from_fn(|d| lower[d] + 0.5 * (upper[d] - lower[d]))
    };
    let wrench = AlignedViscousBoundaryWrench::new(&op, reference, CAP, |_, _| false)?;
    let mut out = Vec::new();
    write!(
        out,
        "{{\"schema\":\"rheon-viscous-boundary-wrench-v1\",\"meta\":{{\"counts\":{:?},\"spacing\":",
        n
    )?;
    values(&mut out, &h)?;
    write!(out, ",\"origin\":")?;
    values(&mut out, &origin)?;
    write!(out, ",\"lo\":{:?},\"hi\":{:?},\"rho\":", lo, hi)?;
    bits(&mut out, rho)?;
    write!(out, ",\"mu\":")?;
    bits(&mut out, mu)?;
    write!(out, "}},\"reference\":")?;
    values(&mut out, &reference)?;
    write!(out, ",\"active\":[")?;
    for (i, f) in op.active_faces().iter().enumerate() {
        if i != 0 {
            write!(out, ",")?;
        }
        write!(
            out,
            "{{\"id\":{i},\"axis\":{},\"face\":{},\"coordinates\":{:?},\"position\":",
            axis(f.axis),
            f.face,
            f.coordinates
        )?;
        values(&mut out, &f.position)?;
        for (name, v) in [("area", f.area), ("distance", f.distance), ("mass", f.mass)] {
            write!(out, ",\"{name}\":")?;
            bits(&mut out, v)?;
        }
        write!(out, "}}")?;
    }
    write!(out, "],\"rows\":[")?;
    for (i, (r, c)) in op.rows().iter().zip(wrench.rows()).enumerate() {
        if i != 0 {
            write!(out, ",")?;
        }
        let boundary = match r.boundary {
            AlignedStrainBoundary::Normal => "normal",
            AlignedStrainBoundary::Interior => "interior",
            AlignedStrainBoundary::ObstacleFlat => "flat",
            AlignedStrainBoundary::ObstacleCorner => "corner",
            AlignedStrainBoundary::OuterFreeSlip => {
                return Err("unexpected enumerated outer shear row".into())
            }
        };
        write!(out,"{{\"id\":{i},\"axes\":[{},{}],\"coordinates\":{:?},\"quadrant\":{},\"boundary\":\"{boundary}\",\"weight\":",axis(r.axes[0]),axis(r.axes[1]),r.coordinates,r.quadrant)?;
        bits(&mut out, r.weight)?;
        write!(out, ",\"terms\":[")?;
        for (j, t) in r.terms().iter().enumerate() {
            if j != 0 {
                write!(out, ",")?;
            }
            write!(out, "{{\"active\":{},\"coefficient\":", t.active)?;
            bits(&mut out, t.coefficient)?;
            write!(out, "}}")?;
        }
        write!(out, "],\"solid\":")?;
        values(&mut out, &c.solid)?;
        write!(out, ",\"outer\":")?;
        values(&mut out, &c.outer)?;
        write!(out, "}}")?;
    }
    write!(out, "],\"fields\":[")?;
    let mut force = vec![0.0; op.active_faces().len()];
    for (i, (name, u)) in fields(&op, reference).iter().enumerate() {
        if i != 0 {
            write!(out, ",")?;
        }
        let r = wrench.diagnose(u, &mut force, |_, _| false)?;
        let virtual_work =
            wrench.virtual_boundary_work(u, SOLID_TWIST, OUTER_TWIST, |_, _| false)?;
        write!(out, "{{\"name\":\"{name}\",\"values\":")?;
        values(&mut out, u)?;
        write!(
            out,
            ",\"identity\":{{\"surface_stamp\":{{\"id\":\"{}\",\"version\":\"{}\"}},\"reference\":",
            r.surface_stamp.id, r.surface_stamp.version
        )?;
        values(&mut out, &r.reference)?;
        write!(out, ",\"density\":")?;
        bits(&mut out, r.density)?;
        write!(out, ",\"viscosity\":")?;
        bits(&mut out, r.viscosity)?;
        write!(out, "}}")?;
        for (name, v) in [
            ("force", force.as_slice()),
            ("solid_wrench", r.solid_wrench.as_slice()),
            ("outer_wrench", r.outer_wrench.as_slice()),
            ("fluid_wrench", r.fluid_wrench.as_slice()),
            ("balance_defect", r.balance_defect.as_slice()),
        ] {
            write!(out, ",\"{name}\":")?;
            values(&mut out, v)?;
        }
        for (name, v) in [
            ("dissipation", r.dissipation),
            ("force_work", r.force_work),
            ("work_defect", r.work_defect),
        ] {
            write!(out, ",\"{name}\":")?;
            bits(&mut out, v)?;
        }
        write!(out, ",\"virtual_work\":{{\"solid_twist\":")?;
        values(&mut out, &SOLID_TWIST)?;
        write!(out, ",\"outer_twist\":")?;
        values(&mut out, &OUTER_TWIST)?;
        for (name, v) in [
            ("wrench_work", virtual_work.wrench_work),
            ("row_work", virtual_work.row_work),
            ("defect", virtual_work.defect),
        ] {
            write!(out, ",\"{name}\":")?;
            bits(&mut out, v)?;
        }
        write!(out, "}}}}")?;
    }
    write!(out, "],\"report\":{{\"common_rigid_residual_max\":")?;
    values(&mut out, &wrench.common_rigid_residual_max())?;
    writeln!(
        out,
        ",\"allocated_bytes\":{},\"combined_operator_bytes\":{}}}}}",
        wrench.allocated_bytes(),
        wrench.combined_operator_bytes()
    )?;
    if out.len() > 2 * 1024 * 1024 {
        return Err("bounded native output exceeded".into());
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(&out)?;
    file.sync_all()?;
    Ok(())
}
