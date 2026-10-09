//! Static geometry records only. No pressure solve, time integration or reference solver.
use rheon::{
    Axis, GridGeometry, StaticObstacleGeometry, SurfaceSettings, SurfaceStamp, TriangleSurface,
};
use std::{
    error::Error,
    fs::OpenOptions,
    io::{BufWriter, Write},
    path::Path,
};

fn mesh(lo: [f64; 3], hi: [f64; 3], id: u64) -> TriangleSurface {
    TriangleSurface::new(
        SurfaceStamp { id, version: 1 },
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
    )
    .expect("fixed nondegenerate box fixture")
}

fn main() -> Result<(), Box<dyn Error>> {
    let output = std::env::args()
        .nth(1)
        .ok_or("usage: static_obstacle NEW_RECORDS_JSON")?;
    // Refuse repository outputs and replacement of earlier records.
    let destination = Path::new(&output)
        .parent()
        .ok_or("output needs a parent")?
        .canonicalize()?;
    if std::process::Command::new("git")
        .args([
            "-C",
            destination.to_str().ok_or("UTF-8 output")?,
            "rev-parse",
            "--is-inside-work-tree",
        ])
        .output()?
        .status
        .success()
    {
        return Err("records must be outside Git".into());
    }
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&output)?;
    let mut out = BufWriter::new(file);
    let base = ([3, 3, 3], [1.0; 3], [0.0; 3]);
    let mut cases = vec![
        ("partial", base, [0.25, 0.5, 0.75], [1.75, 1.5, 2.25]),
        ("separator-x", base, [1.0, 0.0, 0.0], [2.0, 3.0, 3.0]),
        ("separator-y", base, [0.0, 1.0, 0.0], [3.0, 2.0, 3.0]),
        ("separator-z", base, [0.0, 0.0, 1.0], [3.0, 3.0, 2.0]),
        ("dry", base, [-1.0; 3], [4.0; 3]),
        ("outside", base, [4.0; 3], [5.0; 3]),
        ("touch", base, [3.0, 0.0, 0.0], [4.0, 3.0, 3.0]),
        (
            "translated-scaled",
            ([3, 3, 3], [0.5, 1.0, 2.0], [4.0, -8.0, 16.0]),
            [4.25, -7.5, 17.0],
            [5.25, -6.5, 19.0],
        ),
        (
            "non-dyadic",
            ([3, 4, 2], [0.3, 0.2, 0.7], [0.1, -0.2, 0.4]),
            [0.21, -0.1, 0.5],
            [0.62, 0.37, 1.3],
        ),
    ];
    writeln!(
        out,
        "{{\"schema\":\"rheon-static-obstacle-records-v1\",\"fluid_advances\":0,\"cases\":["
    )?;
    for (case_index, (name, (counts, spacing, origin), lo, hi)) in cases.drain(..).enumerate() {
        if case_index > 0 {
            writeln!(out, ",")?;
        }
        let owner = StaticObstacleGeometry::new(
            GridGeometry::new(counts, spacing, origin)?,
            mesh(lo, hi, case_index as u64 + 1),
            1_000_000,
            |_, _| false,
        )?;
        writeln!(
            out,
            "{{\"id\":{name:?},\"counts\":{counts:?},\"spacing\":{spacing:?},\"origin\":{origin:?},\"lower\":{lo:?},\"upper\":{hi:?},\"stamp\":[{},1],\"vertices\":{:?},\"triangles\":{:?},\"volumes\":{:?},\"areas\":[{:?},{:?},{:?}],\"components\":{},\"allocation\":[{},{}],\"labels\":[",
            case_index + 1,
            owner.surface().vertices(),
            owner.surface().triangles(),
            owner.fluid_volumes(),
            owner.open_areas(Axis::X),
            owner.open_areas(Axis::Y),
            owner.open_areas(Axis::Z),
            owner.component_count(),
            owner.allocation().retained_bytes,
            owner.allocation().constructor_peak_bytes
        )?;
        for (i, label) in owner.component_labels().iter().enumerate() {
            if i > 0 {
                write!(out, ",")?;
            }
            if *label == rheon::NO_FLUID_COMPONENT {
                write!(out, "null")?;
            } else {
                write!(out, "{label}")?;
            }
        }
        writeln!(out, "],\"probes\":[")?;
        for d in 0..3 {
            if d > 0 {
                writeln!(out, ",")?;
            }
            let width = hi[d] - lo[d];
            let mut start =
                std::array::from_fn(|a| lo[a] + (hi[a] - lo[a]) * [0.29, 0.37, 0.41][a]);
            let mut end = start;
            start[d] = lo[d] - width;
            end[d] = hi[d] + width;
            let hit = owner
                .surface()
                .first_hit(start, end, |_| false)?
                .ok_or("fixed crossing missed")?;
            writeln!(
                out,
                "{{\"axis\":{d},\"start\":{start:?},\"end\":{end:?},\"t\":{:?},\"position\":{:?},\"triangle\":{},\"stamp\":[{},{}]}}",
                hit.parameter, hit.position, hit.triangle, hit.surface.id, hit.surface.version
            )?;
        }
        writeln!(out, "],\"flux_controls\":[")?;
        for (d, axis) in [Axis::X, Axis::Y, Axis::Z].into_iter().enumerate() {
            if d > 0 {
                writeln!(out, ",")?;
            }
            let p = [1, 1, 1];
            let mut negative = p;
            negative[d] -= 1;
            let mut speeds: [Vec<f64>; 3] = std::array::from_fn(|a| {
                vec![0.0; owner.grid().face_len([Axis::X, Axis::Y, Axis::Z][a])]
            });
            speeds[d][owner.grid().face_index(axis, p).unwrap()] = 2.0;
            let fields = speeds.each_ref().map(|v| v.as_slice());
            let face = owner.face(axis, p).unwrap();
            writeln!(
                out,
                "{{\"axis\":{d},\"coordinate\":{p:?},\"area\":{:?},\"negative\":{},\"positive\":{},\"speed\":2.0,\"outward\":[{:?},{:?}]}}",
                face.area,
                face.negative.unwrap(),
                face.positive.unwrap(),
                owner.outward_flux(negative, fields)?,
                owner.outward_flux(p, fields)?
            )?;
        }
        writeln!(out, "]}}")?;
    }
    writeln!(out, "],\"refusals\":[")?;
    for d in 0..3 {
        if d > 0 {
            writeln!(out, ",")?;
        }
        let mut lo = [0.0; 3];
        let mut hi = [3.0; 3];
        lo[d] = 1.25;
        hi[d] = 1.75;
        let error = StaticObstacleGeometry::new(
            GridGeometry::new(base.0, base.1, base.2)?,
            mesh(lo, hi, 100 + d as u64),
            1_000_000,
            |_, _| false,
        )
        .expect_err("unresolved separator must refuse");
        writeln!(
            out,
            "{{\"axis\":{d},\"lower\":{lo:?},\"upper\":{hi:?},\"error\":{:?}}}",
            error.to_string()
        )?;
    }
    writeln!(out, "]}}")?;
    out.flush()?;
    Ok(())
}
