//! Bounded obstacle pressure projections and reduced-shear updates only.
//! No historical numerical campaign, free-surface or general fluid integration.
use rheon::{
    Axis, GridGeometry, ObstacleShearForce, ObstacleShearWall, PressureSettings,
    StaticObstacleGeometry, StaticObstaclePressure, StaticObstacleShear, SurfaceSettings,
    SurfaceStamp, TriangleSurface,
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
    .unwrap()
}
fn geometry(out: &mut impl Write, o: &StaticObstacleGeometry) -> Result<(), Box<dyn Error>> {
    write!(
        out,
        "\"counts\":{:?},\"spacing\":{:?},\"origin\":{:?},\"lower\":{:?},\"upper\":{:?},\"stamp\":[{},1],\"vertices\":{:?},\"triangles\":{:?},\"volumes\":{:?},\"areas\":[{:?},{:?},{:?}],\"labels\":[",
        o.grid().counts(),
        o.grid().spacing(),
        o.grid().origin(),
        o.box_bounds().0,
        o.box_bounds().1,
        o.stamp().id,
        o.surface().vertices(),
        o.surface().triangles(),
        o.fluid_volumes(),
        o.open_areas(Axis::X),
        o.open_areas(Axis::Y),
        o.open_areas(Axis::Z)
    )?;
    for (i, &l) in o.component_labels().iter().enumerate() {
        if i > 0 {
            write!(out, ",")?;
        }
        if l == rheon::NO_FLUID_COMPONENT {
            write!(out, "null")?;
        } else {
            write!(out, "{l}")?;
        }
    }
    write!(out, "],\"components\":{}", o.component_count())?;
    Ok(())
}
fn main() -> Result<(), Box<dyn Error>> {
    let output = std::env::args()
        .nth(1)
        .ok_or("usage: obstacle_flow NEW_RECORDS_JSON")?;
    let parent = Path::new(&output)
        .parent()
        .ok_or("output needs a parent")?
        .canonicalize()?;
    if std::process::Command::new("git")
        .args([
            "-C",
            parent.to_str().ok_or("UTF-8 output")?,
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
    write!(
        out,
        "{{\"schema\":\"rheon-obstacle-flow-records-v1\",\"pressure_cases\":["
    )?;
    for (case, (name, lo, hi, gradient)) in [
        (
            "partial-gradient",
            [0.25, 0.5, 0.75],
            [1.75, 1.5, 2.25],
            [1.0, 2.0, 3.0],
        ),
        ("outside-gradient", [4.0; 3], [5.0; 3], [1.0, 2.0, 3.0]),
        (
            "separator-gradient",
            [1.0, 0.0, 0.0],
            [2.0, 3.0, 3.0],
            [1.0, 2.0, 3.0],
        ),
        (
            "partial-rest",
            [0.25, 0.5, 0.75],
            [1.75, 1.5, 2.25],
            [0.0; 3],
        ),
    ]
    .into_iter()
    .enumerate()
    {
        if case > 0 {
            write!(out, ",")?;
        }
        let o = StaticObstacleGeometry::new(
            GridGeometry::new([3; 3], [1.0; 3], [0.0; 3])?,
            mesh(lo, hi, case as u64 + 1),
            1_000_000,
            |_, _| false,
        )?;
        let mut u: [Vec<f64>; 3] =
            std::array::from_fn(|d| vec![0.0; o.grid().face_len([Axis::X, Axis::Y, Axis::Z][d])]);
        for (d, axis) in [Axis::X, Axis::Y, Axis::Z].into_iter().enumerate() {
            let shape = o.grid().face_counts(axis);
            for (f, v) in u[d].iter_mut().enumerate() {
                let p = [
                    f % shape[0],
                    f / shape[0] % shape[1],
                    f / (shape[0] * shape[1]),
                ];
                if p[d] > 0 && p[d] < 3 && o.open_areas(axis)[f] > 0.0 {
                    *v = 0.5 * gradient[d] / 2.0;
                }
            }
        }
        let before = u.clone();
        let mut p = StaticObstaclePressure::new(&o, 2.0, 1_000_000, |_, _| false)?;
        let [x, y, z] = &mut u;
        let r = p.project(
            [x, y, z],
            0.5,
            PressureSettings {
                relative_residual: 1e-12,
                absolute_residual: 1e-12,
                divergence_limit: 1e-10,
                max_iterations: 300,
            },
            |_, _| false,
        )?;
        let l = r.corrected.ok_or("projection ledger")?;
        write!(out, "{{\"id\":{name:?},")?;
        geometry(&mut out, &o)?;
        write!(
            out,
            ",\"density\":2.0,\"dt\":0.5,\"gradient\":{gradient:?},\"gauges\":{:?},\"before\":{:?},\"after\":{:?},\"pressure\":{:?},\"iterations\":{},\"residual\":[{:?},{:?},{:?}],\"ledger\":[{:?},{:?},{:?},{:?},{:?},{:?},{:?}],\"workspace_bytes\":{}}}",
            p.gauge_cells(),
            before,
            u,
            p.pressure().ok_or("qualified pressure")?,
            r.iterations,
            r.residual_l2,
            r.residual_max,
            r.predicted_divergence_max,
            l.actual_divergence_max,
            l.kinetic_before,
            l.kinetic_after,
            l.correction_energy,
            l.residual_work,
            l.identity_error,
            l.rounding_budget,
            p.allocated_bytes()
        )?;
    }
    write!(out, "],\"shear_cases\":[")?;
    for (case, (name, wall, rho, force, cut)) in [
        (
            "cut-no-slip",
            ObstacleShearWall::NoSlip,
            2.0,
            ObstacleShearForce::Density(2.0),
            1.5,
        ),
        (
            "cut-navier",
            ObstacleShearWall::Navier { beta: 4.0 },
            2.0,
            ObstacleShearForce::Density(2.0),
            1.5,
        ),
        (
            "cut-free-slip",
            ObstacleShearWall::Navier { beta: 0.0 },
            2.0,
            ObstacleShearForce::Density(2.0),
            1.5,
        ),
        (
            "channel-density-one",
            ObstacleShearWall::NoSlip,
            1.0,
            ObstacleShearForce::Density(1.0),
            0.5,
        ),
        (
            "channel-density-two",
            ObstacleShearWall::NoSlip,
            2.0,
            ObstacleShearForce::Density(1.0),
            0.5,
        ),
        (
            "channel-acceleration-two",
            ObstacleShearWall::NoSlip,
            2.0,
            ObstacleShearForce::Acceleration(1.0),
            0.5,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        if case > 0 {
            write!(out, ",")?;
        }
        let counts = if cut == 1.5 { [1, 2, 1] } else { [1, 4, 1] };
        let o = StaticObstacleGeometry::new(
            GridGeometry::new(counts, [1.0; 3], [0.0; 3])?,
            mesh([0.0; 3], [1.0, cut, 1.0], case as u64 + 101),
            1_000_000,
            |_, _| false,
        )?;
        let mut s = StaticObstacleShear::new(&o, Axis::Y, rho, 1.0, wall, 1_000_000, |_, _| false)?;
        let mut u = vec![0.0; s.layer_volumes().len()];
        write!(out, "{{\"id\":{name:?},")?;
        geometry(&mut out, &o)?;
        let beta = match wall {
            ObstacleShearWall::NoSlip => "null".to_string(),
            ObstacleShearWall::Navier { beta } => format!("{beta}"),
        };
        let (kind, value) = match force {
            ObstacleShearForce::Density(q) => ("density", q),
            ObstacleShearForce::Acceleration(a) => ("acceleration", a),
        };
        write!(
            out,
            ",\"normal\":1,\"density\":{rho:?},\"viscosity\":1.0,\"beta\":{beta},\"force_kind\":{kind:?},\"force_value\":{value:?},\"dt\":0.125,\"centers\":{:?},\"layer_volumes\":{:?},\"interior_conductances\":{:?},\"wall_conductances\":{:?},\"workspace_bytes\":{},\"frames\":[{{\"step\":0,\"velocity\":{u:?}}}",
            s.layer_centers(),
            s.layer_volumes(),
            s.interior_conductances(),
            s.wall_conductances(),
            s.allocated_bytes()
        )?;
        for step in 1..=8 {
            let r = s.step(&mut u, 0.125, force, |_, _| false)?;
            write!(
                out,
                ",{{\"step\":{step},\"velocity\":{u:?},\"energy\":[{:?},{:?},{:?},{:?},{:?},{:?},{:?},{:?}],\"momentum\":[{:?},{:?},{:?},{:?},{:?},{:?}],\"residual_max\":{:?}}}",
                r.kinetic_before,
                r.kinetic_after,
                r.increment_energy,
                r.body_work,
                r.dissipation,
                r.boundary_dissipation,
                r.energy_identity_error,
                r.energy_rounding_budget,
                r.momentum_before,
                r.momentum_after,
                r.body_impulse,
                r.wall_impulse,
                r.momentum_identity_error,
                r.momentum_rounding_budget,
                r.residual_max
            )?;
        }
        write!(out, "]}}")?;
    }
    writeln!(out, "]}}")?;
    out.flush()?;
    Ok(())
}
