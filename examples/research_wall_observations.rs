//! Research-only observation acquisition and generic surface estimator; no stepping.
#[path = "viscous_boundary_wrench.rs"]
#[allow(dead_code)]
mod frozen;
use rheon::{AlignedStrain, AlignedViscousBoundaryWrench};
use std::{
    error::Error,
    fs::OpenOptions,
    io::{BufWriter, Write},
    mem::size_of,
    path::Path,
};
#[derive(Clone, Copy)]
struct Observation {
    normal: usize,
    component: usize,
    side: i8,
    p: [f64; 3],
    a: f64,
    b: f64,
    u: f64,
    v: Option<f64>,
    area: f64,
}
// Geometry and measured values only; no analytic derivatives or target.
fn load(rows: &[Observation], second: bool, reference: [f64; 3]) -> Option<[f64; 6]> {
    let mut w = [0.; 6];
    for r in rows {
        if !(r.a > 0. && r.area > 0. && r.component != r.normal && [-1, 1].contains(&r.side)) {
            return None;
        }
        let derivative = if second {
            let v = r.v?;
            if r.b <= r.a {
                return None;
            }
            r.b / (r.a * (r.b - r.a)) * r.u - r.a / (r.b * (r.b - r.a)) * v
        } else {
            r.u / r.a
        };
        let mut f = [0.; 3];
        f[r.component] = r.area * derivative;
        let x = std::array::from_fn::<_, 3, _>(|d| r.p[d] - reference[d]);
        let t = [
            x[1] * f[2] - x[2] * f[1],
            x[2] * f[0] - x[0] * f[2],
            x[0] * f[1] - x[1] * f[0],
        ];
        for d in 0..3 {
            w[d] += f[d];
            w[d + 3] += t[d];
        }
    }
    Some(w)
}
// Diagnostic initializer, kept separate from the generic load action.
fn sample(i: usize, x: [f64; 3]) -> f64 {
    let p = x.map(|t| (t - 1.) * (t - 2.));
    let tilt = x[0] - 0.5;
    match i {
        0 => 450. * tilt * p[0] * p[0] * p[1] * (2. * x[1] - 3.) * p[2] * p[2],
        1 => {
            -225. * (p[0] * p[0] + 2. * tilt * p[0] * (2. * x[0] - 3.)) * p[1] * p[1] * p[2] * p[2]
        }
        _ => 0.,
    }
}
fn observations(
    s: usize,
    native: Option<&AlignedStrain<'_>>,
    u: &[f64],
) -> Result<Vec<Observation>, Box<dyn Error>> {
    let count = if native.is_some() {
        12 * s * (s + 1)
    } else {
        12 * s * s
    };
    if count
        .checked_mul(size_of::<Observation>())
        .ok_or("overflow")?
        > 2_000_000
    {
        return Err("observation capacity refused before allocation".into());
    }
    let mut rows = Vec::with_capacity(count);
    let h = 1. / s as f64;
    for n in 0..3 {
        for side in [-1i8, 1] {
            let wall = if side < 0 { 1. } else { 2. };
            for i in 0..3 {
                if i == n {
                    continue;
                }
                let j = 3 - n - i;
                for ki in 0..(s + usize::from(native.is_some())) {
                    for kj in 0..s {
                        let mut p = [0.; 3];
                        p[n] = wall;
                        p[i] = 1. + h * (ki as f64 + if native.is_some() { 0. } else { 0.5 });
                        p[j] = 1. + h * (kj as f64 + 0.5);
                        let mut x = p;
                        x[n] += side as f64 * h / 2.;
                        let mut y = p;
                        y[n] += side as f64 * 3. * h / 2.;
                        let (first, second) = if let Some(op) = native {
                            let a = op
                                .active_faces()
                                .iter()
                                .position(|f| frozen::axis(f.axis) == i && f.position == x)
                                .ok_or("missing first MAC sample")?;
                            let b = op
                                .active_faces()
                                .iter()
                                .position(|f| frozen::axis(f.axis) == i && f.position == y);
                            (u[a], b.map(|k| u[k]))
                        } else {
                            (sample(i, x), if s == 1 { None } else { Some(sample(i, y)) })
                        };
                        let area = h
                            * h
                            * if native.is_some() && (ki == 0 || ki == s) {
                                0.5
                            } else {
                                1.
                            };
                        rows.push(Observation {
                            normal: n,
                            component: i,
                            side,
                            p,
                            a: h / 2.,
                            b: 3. * h / 2.,
                            u: first,
                            v: second,
                            area,
                        });
                    }
                }
            }
        }
    }
    assert_eq!(rows.len(), count);
    Ok(rows)
}
fn bits(out: &mut impl Write, x: f64) -> std::io::Result<()> {
    write!(out, "\"{:016x}\"", x.to_bits())
}
fn values(out: &mut impl Write, x: &[f64]) -> std::io::Result<()> {
    write!(out, "[")?;
    for (k, v) in x.iter().enumerate() {
        if k > 0 {
            write!(out, ",")?;
        }
        bits(out, *v)?;
    }
    write!(out, "]")
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        return Err("MODE SCALE NEW_EXTERNAL_JSON".into());
    }
    let s: usize = args[1].parse()?;
    let mode = args[0].as_str();
    if !(if mode == "native" {
        [1, 2, 4].contains(&s)
    } else {
        mode == "wall" && [1, 2, 4, 8, 16, 32].contains(&s)
    }) {
        return Err("unreviewed roster: refused before allocation".into());
    }
    let path = Path::new(&args[2]);
    let canonical_parent = path.parent().ok_or("missing parent")?.canonicalize()?;
    if canonical_parent.starts_with(Path::new(env!("CARGO_MANIFEST_DIR"))) {
        return Err("output must be outside Git worktree".into());
    }
    let h = 1. / s as f64;
    let mut native_data = None;
    if mode == "native" {
        let owner = frozen::geometry([3 * s as u64; 3], [h; 3], [0.; 3], [s; 3], [2 * s; 3])?;
        let op = AlignedStrain::new(&owner, 1., 1., frozen::CAP, |_, _| false)?;
        let u: Vec<_> = op
            .active_faces()
            .iter()
            .map(|f| sample(frozen::axis(f.axis), f.position))
            .collect();
        let rows = observations(s, Some(&op), &u)?;
        let wrench = AlignedViscousBoundaryWrench::new(&op, [1.5; 3], frozen::CAP, |_, _| false)?;
        let mut force = vec![0.; u.len()];
        let result = wrench.diagnose(&u, &mut force, |_, _| false)?;
        let points: Vec<_> = op
            .active_faces()
            .iter()
            .zip(u)
            .map(|(f, u)| (frozen::axis(f.axis), f.position, u))
            .collect();
        native_data = Some((
            rows,
            points,
            op.rows().len(),
            wrench.combined_operator_bytes(),
            owner.allocation().retained_bytes,
            result.solid_wrench,
        ));
    }
    let wall_rows;
    if native_data.is_none() {
        wall_rows = observations(s, None, &[])?;
    } else {
        wall_rows = Vec::new();
    }
    let rows = if let Some((r, ..)) = &native_data {
        r
    } else {
        &wall_rows
    };
    let file = OpenOptions::new().write(true).create_new(true).open(path)?;
    let mut out = BufWriter::new(file);
    write!(
        out,
        "{{\"schema\":\"independent-wall-observations-v1\",\"mode\":\"{mode}\",\"scale\":{s},\"row_size\":{},\"observation_bytes\":{},\"P1\":",
        size_of::<Observation>(),
        rows.capacity() * size_of::<Observation>()
    )?;
    values(&mut out, &load(rows, false, [1.5; 3]).ok_or("P1 failure")?)?;
    write!(out, ",\"P2\":")?;
    if let Some(w) = load(rows, true, [1.5; 3]) {
        values(&mut out, &w)?;
    } else {
        write!(out, "null")?;
    }
    write!(out, ",\"observations\":[")?;
    for (k, r) in rows.iter().enumerate() {
        if k > 0 {
            write!(out, ",")?;
        }
        write!(
            out,
            "{{\"normal\":{},\"component\":{},\"side\":{},\"p\":",
            r.normal, r.component, r.side
        )?;
        values(&mut out, &r.p)?;
        for (name, v) in [("a", r.a), ("b", r.b), ("u", r.u), ("area", r.area)] {
            write!(out, ",\"{name}\":")?;
            bits(&mut out, v)?;
        }
        write!(out, ",\"v\":")?;
        if let Some(v) = r.v {
            bits(&mut out, v)?;
        } else {
            write!(out, "null")?;
        }
        write!(out, "}}")?;
    }
    write!(out, "],\"native\":")?;
    if let Some((_, points, nr, bytes, geometry, w)) = native_data {
        write!(
            out,
            "{{\"active_count\":{},\"row_count\":{nr},\"combined_bytes\":{bytes},\"geometry_bytes\":{geometry},\"old_generalized_wrench\":",
            points.len()
        )?;
        values(&mut out, &w)?;
        write!(out, ",\"samples\":[")?;
        for (k, (i, p, u)) in points.iter().enumerate() {
            if k > 0 {
                write!(out, ",")?;
            }
            write!(out, "{{\"component\":{i},\"p\":")?;
            values(&mut out, p)?;
            write!(out, ",\"u\":")?;
            bits(&mut out, *u)?;
            write!(out, "}}")?;
        }
        write!(out, "]}}")?;
    } else {
        write!(out, "null")?;
    }
    write!(out, "}}\n")?;
    out.flush()?;
    if path.metadata()?.len()
        > if mode == "native" {
            2_000_000
        } else {
            8_000_000
        }
    {
        return Err("external record exceeds frozen size cap".into());
    }
    Ok(())
}
