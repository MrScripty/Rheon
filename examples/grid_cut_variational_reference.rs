//! CV1 private cut-cell operator reference: no PDE solve or production-water claim.
#[allow(dead_code)]
mod grid_cut_variational_reference {
    pub mod assembly;
    pub mod basis;
    pub mod clip;
    #[path = "../grid_geometry_reference/exact.rs"]
    pub mod exact;
    pub mod export;
    #[path = "../grid_geometry_reference/geometry.rs"]
    pub mod geometry;
    pub mod observe;
    pub mod trace;
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
fn run(root: &Path, fixture: Fixture) -> Result<(usize, usize, f64), Box<dyn std::error::Error>> {
    let case = root.join(fixture.name);
    std::fs::create_dir(&case)?;
    export::inputs(
        &case,
        &fixture.vertices,
        fixture.n,
        fixture.origin,
        fixture.extent,
    )?;
    let source = Polygon::new(fixture.vertices)?;
    let op = Operators::build(fixture.n, fixture.origin, fixture.extent, source)?;
    let tr = Traces::build(&op)?;
    let report = Report::new(&op, &tr)?;
    // Export every completed raw observation before evaluating mandatory gates.
    export::all(&case, &op, &tr, &report)?;
    let worst = report.gates.values().copied().fold(0.0, f64::max);
    if worst > GATE {
        return Err(format!(
            "{}: fixed gate exceeded: {worst:.17e} > {GATE}",
            fixture.name
        )
        .into());
    }
    Ok((op.active.len(), op.cells.len(), worst))
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::env::args_os()
        .nth(1)
        .ok_or("fresh output directory required")?;
    let root = Path::new(&out);
    std::fs::create_dir(root)?;
    let mut summary = std::io::BufWriter::new(std::fs::File::create(root.join("summary.csv"))?);
    writeln!(
        summary,
        "case,active,pressure_cells,worst_fixed_gate,status"
    )?;
    for fixture in fixtures() {
        let name = fixture.name;
        match run(root, fixture) {
            Ok((active, cells, worst)) => {
                writeln!(summary, "{name},{active},{cells},{worst:.17e},passed")?;
                summary.flush()?;
                println!(
                    "{name}: active={active} pressure_cells={cells} maximum_gate={worst:.17e}"
                );
            }
            Err(e) => {
                writeln!(summary, "{name},,,refused")?;
                summary.flush()?;
                return Err(format!("{name}: {e}").into());
            }
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pressure_control_observes_each_nodal_load_and_absolute_coefficient() {
        let f = fixtures().remove(0);
        let op =
            Operators::build(f.n, f.origin, f.extent, Polygon::new(f.vertices).unwrap()).unwrap();
        let tr = Traces::build(&op).unwrap();
        let report = Report::new(&op, &tr).unwrap();
        assert!(report.observations["pressure_nodal_signal"] > 0.0);
        assert!((report.observations["constant_pressure_load_coefficient_Pa"] - 2.0).abs() < GATE);
        assert!(report.gates["affine_nodal_total_stress"] < GATE);
        assert!(report.gates["omitted_pressure_orientation"] < GATE);
    }
}
