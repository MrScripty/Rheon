//! Native moving-plate fixtures; JSON on stdout, no simulation file mutation.
#[allow(dead_code)]
#[path = "../experiments/plate_coupling.rs"]
mod plate_coupling;
use plate_coupling::{Interval, Plate, Workspace};
use rheon::{
    Axis, GridGeometry, ObstacleShearWall, StaticObstacleGeometry, StaticObstacleShear,
    SurfaceSettings, SurfaceStamp, TriangleSurface,
};
fn mesh(lo: [f64; 3], hi: [f64; 3]) -> TriangleSurface {
    TriangleSurface::new(
        SurfaceStamp { id: 42, version: 7 },
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

fn owner(layers: u64, axis: Axis, wall: f64) -> StaticObstacleGeometry {
    let d = match axis {
        Axis::X => 0,
        Axis::Y => 1,
        Axis::Z => 2,
    };
    let mut counts = [1; 3];
    counts[d] = layers;
    let mut spacing = [1.0; 3];
    spacing[d] = 1.0 / layers as f64;
    let mut lo = [0.0; 3];
    lo[d] = -1.0;
    let mut hi = [1.0; 3];
    hi[d] = wall;
    StaticObstacleGeometry::new(
        GridGeometry::new(counts, spacing, [0.0; 3]).unwrap(),
        mesh(lo, hi),
        1_000_000,
        |_, _| false,
    )
    .unwrap()
}

fn interval(x: Interval) -> [f64; 2] {
    [x.lo, x.hi]
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[ ");
    for case in 0..6 {
        let n = if case < 3 { 1 } else { 4 };
        let cut = if case >= 3 { 0.125 } else { 0.0 };
        let rho = if case >= 3 { 2.0 } else { 1.0 };
        let mu = if case >= 3 { 0.5 } else { 1.0 };
        let normal = [Axis::X, Axis::Y, Axis::Z][case % 3];
        let tangent = [Axis::Y, Axis::Z, Axis::X][case % 3];
        let closure = match case {
            4 => ObstacleShearWall::Navier { beta: 2.0 },
            5 => ObstacleShearWall::Navier { beta: 0.0 },
            _ => ObstacleShearWall::NoSlip,
        };
        let masses = match case {
            0 => [0.0, 0.0],
            1 => [1.0, 0.0],
            2 => [1.0, 1.0],
            _ => [2.0, 3.0],
        };
        let modes = masses.map(|mass| {
            if mass == 0.0 {
                Plate::Prescribed
            } else {
                Plate::Dynamic { mass_kg: mass }
            }
        });
        let g = owner(n, normal, cut);
        let op = StaticObstacleShear::new(&g, normal, rho, mu, closure, 1_000_000, |_, _| false)?;
        let mut workspace = Workspace::new(&op, tangent, modes, 1_000_000)?;
        let mut fluid = if case < 3 {
            vec![0.0]
        } else {
            vec![0.0, 0.25, -0.5, 0.75]
        };
        let mut walls = match case {
            0 => [0.0, 1.0],
            1 => [1.0, 0.0],
            _ => [1.0, -1.0],
        };
        let dt = 0.25;
        for step in 0..3 {
            let old = fluid.clone();
            let old_walls = walls;
            let report = workspace.step(&mut fluid, &mut walls, dt, |_, _| false)?;
            if case != 0 || step != 0 {
                println!(",");
            }
            let beta = match closure {
                ObstacleShearWall::NoSlip => "null".to_string(),
                ObstacleShearWall::Navier { beta } => beta.to_string(),
            };
            println!(
                "{{\"case\":{case},\"step\":{step},\"n\":{n},\"normal\":{},\"tangent\":{},\"cut\":{cut},\"rho\":{rho},\"mu\":{mu},\"beta\":{beta},\"dt\":{dt},\"plate_masses\":{masses:?},\"old\":{old:?},\"old_walls\":{old_walls:?},\"new\":{fluid:?},\"new_walls\":{walls:?},\"volumes\":{:?},\"centers\":{:?},\"interior_k\":{:?},\"wall_k\":{:?},\"fluid_impulses\":{:?},\"before\":{:?},\"after\":{:?},\"change\":{:?},\"increment\":{:?},\"dissipation\":{:?},\"work\":{:?},\"energy_defect\":{:?},\"energy_budget\":{:?},\"momentum\":{:?},\"external_impulse\":{:?},\"momentum_defect\":{:?},\"momentum_budget\":{:?},\"equation_max\":{}}}",
                case % 3,
                (case + 1) % 3,
                op.layer_volumes(),
                op.layer_centers(),
                op.interior_conductances(),
                op.wall_conductances(),
                report.fluid_impulse_n_s.map(interval),
                interval(report.kinetic_before_j),
                interval(report.kinetic_after_j),
                interval(report.kinetic_change_j),
                interval(report.increment_energy_j),
                interval(report.dissipation_j),
                interval(report.prescribed_work_j),
                interval(report.energy_defect_j),
                interval(report.energy_budget_j),
                interval(report.momentum_change_n_s),
                interval(report.prescribed_impulse_n_s),
                interval(report.momentum_defect_n_s),
                interval(report.momentum_budget_n_s),
                report.equation_defect_max_n_s
            );
        }
    }
    println!("]");
    Ok(())
}
