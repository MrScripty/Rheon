//! CV2 static planar coupled reference; no PDE/production admission.
#[allow(dead_code)]
mod grid_cut_variational_reference {
    #[path = "../grid_cut_variational_reference/assembly.rs"]
    pub mod assembly;
    #[path = "../grid_cut_variational_reference/basis.rs"]
    pub mod basis;
    #[path = "../grid_cut_variational_reference/clip.rs"]
    pub mod clip;
    #[path = "../grid_geometry_reference/exact.rs"]
    pub mod exact;
    #[path = "../grid_cut_variational_reference/export.rs"]
    pub mod export;
    #[path = "../grid_geometry_reference/geometry.rs"]
    pub mod geometry;
    #[path = "../grid_cut_variational_reference/observe.rs"]
    pub mod observe;
    #[path = "../grid_cut_variational_reference/trace.rs"]
    pub mod trace;
}
mod grid_cut_coupled_reference {
    pub mod experiment;
    pub mod solver;
}
use grid_cut_variational_reference::{
    assembly::Operators, export, geometry::Polygon, observe::Report, trace::Traces,
};
use std::{io::Write, path::Path};
const RHO: f64 = 1.0;
const MU: f64 = 0.1;
const GATE: f64 = 1e-10;
struct Fixture {
    name: &'static str,
    n: usize,
    origin: [f64; 2],
    extent: f64,
    vertices: Vec<[f64; 2]>,
}
fn fixtures() -> Vec<Fixture> {
    let (s, c) = 17.0_f64.to_radians().sin_cos();
    let square = [[-0.15, -0.15], [0.15, -0.15], [0.15, 0.15], [-0.15, 0.15]]
        .into_iter()
        .map(|[x, y]| [0.537 + c * x - s * y, 0.479 + s * x + c * y])
        .collect();
    let disk = (0..64)
        .map(|k| {
            let t = std::f64::consts::TAU * k as f64 / 64.0;
            [0.75 * t.cos(), 0.75 * t.sin()]
        })
        .collect();
    vec![
        Fixture {
            name: "rectangle",
            n: 4,
            origin: [0.0, 0.0],
            extent: 1.0,
            vertices: vec![[0.17, 0.23], [0.83, 0.23], [0.83, 0.79], [0.17, 0.79]],
        },
        Fixture {
            name: "square-17",
            n: 8,
            origin: [0.0, 0.0],
            extent: 1.0,
            vertices: square,
        },
        Fixture {
            name: "disk-64",
            n: 8,
            origin: [-1.0, -1.0],
            extent: 2.0,
            vertices: disk,
        },
        Fixture {
            name: "corner-triangle",
            n: 4,
            origin: [-1.0, -1.0],
            extent: 4.0,
            vertices: vec![[0.81, 0.99], [0.99, 0.81], [0.99, 0.99]],
        },
    ]
}
fn run_fixture(
    root: &Path,
    f: Fixture,
    operators: &mut impl Write,
    summary: &mut impl Write,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let operator_root = root.join("operators").join(f.name);
    std::fs::create_dir(&operator_root)?;
    export::inputs(&operator_root, &f.vertices, f.n, f.origin, f.extent)?;
    let op = Operators::build(f.n, f.origin, f.extent, Polygon::new(f.vertices)?)?;
    let tr = Traces::build(&op)?;
    let report = Report::new(&op, &tr)?;
    export::all(&operator_root, &op, &tr, &report)?;
    let worst = report.gates.values().copied().fold(0.0_f64, f64::max);
    let status = if worst <= GATE { "passed" } else { "refused" };
    writeln!(
        operators,
        "{},{},{},{worst:.17e},{status}",
        f.name,
        op.active.len(),
        op.cells.len()
    )?;
    operators.flush()?;
    if worst > GATE {
        return Err("immutable CV1 operator gate failed".into());
    }
    let case_root = root.join("solves").join(f.name);
    std::fs::create_dir(&case_root)?;
    let mut failures = Vec::new();
    for dt in [0.125, 1.0] {
        let dt_root = case_root.join(format!("dt-{dt}"));
        std::fs::create_dir(&dt_root)?;
        let factor = grid_cut_coupled_reference::experiment::assemble(&op, dt).and_then(|f| {
            grid_cut_coupled_reference::experiment::factor_export(&dt_root, &f)?;
            Ok(f)
        });
        for scenario in ["unforced", "rigid", "affine"] {
            let step = dt_root.join(scenario);
            std::fs::create_dir(&step)?;
            let result = match &factor {
                Ok(f) => {
                    grid_cut_coupled_reference::experiment::run(&step, &op, &tr, f, dt, scenario)
                }
                Err(e) => Err(e.clone()),
            };
            match result {
                Ok(w) => {
                    writeln!(summary, "{},{dt},{scenario},{w:.17e},passed", f.name)?;
                    println!("{}/dt{dt}/{scenario}: native_gate={w:.17e}", f.name);
                }
                Err(e) => {
                    let e = format!("{}/dt{dt}/{scenario}: {e}", f.name);
                    std::fs::write(step.join("refusal.txt"), &e)?;
                    eprintln!("{e}");
                    failures.push(e);
                    writeln!(summary, "{},{dt},{scenario},,refused", f.name)?;
                }
            }
            summary.flush()?;
        }
    }
    Ok(failures)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::env::args_os()
        .nth(1)
        .ok_or("fresh output directory required")?;
    let root = Path::new(&out);
    std::fs::create_dir(root)?;
    std::fs::create_dir(root.join("operators"))?;
    std::fs::create_dir(root.join("solves"))?;
    let mut operators =
        std::io::BufWriter::new(std::fs::File::create(root.join("operators/summary.csv"))?);
    writeln!(
        operators,
        "case,active,pressure_cells,worst_fixed_gate,status"
    )?;
    let mut summary = std::io::BufWriter::new(std::fs::File::create(root.join("summary.csv"))?);
    writeln!(summary, "case,dt,scenario,worst_native_gate,status")?;
    let mut failures = Vec::new();
    for f in fixtures() {
        let name = f.name;
        match run_fixture(root, f, &mut operators, &mut summary) {
            Ok(errors) => failures.extend(errors),
            Err(e) => {
                let e = format!("{name}: {e}");
                std::fs::write(root.join(format!("{name}-refusal.txt")), &e)?;
                eprintln!("{e}");
                failures.push(e);
            }
        }
    }
    operators.flush()?;
    summary.flush()?;
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n").into())
    }
}
