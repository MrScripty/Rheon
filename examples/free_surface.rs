//! Cell-aligned free-surface pressure laboratory, including rejected cases.
use rheon::*;
use std::{
    fs,
    io::{BufWriter, Write},
    path::Path,
};

fn config() -> LiquidTransportConfig {
    LiquidTransportConfig {
        carrier: SimulationConfig {
            density: 1.0,
            memory_limit: 1024 * 1024,
            pressure: PressureSettings {
                relative_residual: 1e-12,
                absolute_residual: 1e-12,
                divergence_limit: 1e-9,
                max_iterations: 2000,
            },
            actual_divergence_limit: 1e-5,
            max_courant: 1.0,
        },
        represented_density: 1.0,
        volume_stamp: VolumeStamp { id: 1, version: 0 },
        carrier_id: 2,
    }
}
fn input(forces: &[BodyForce], dt: f64) -> LiquidStepInputs<'_> {
    LiquidStepInputs {
        requested_dt: dt,
        smoke_source: None,
        forces,
        inlet: LiquidInlet::new(VolumeStamp { id: 3, version: 0 }, [[0.0; 2]; 3]).unwrap(),
        source: None,
        volume: LiquidVolumeSettings {
            max_outward_courant: 1.0,
            actual_divergence_limit: 1e-5,
        },
    }
}
fn make(
    g: GridGeometry,
    layers: usize,
    method: PressureImplementation,
) -> Result<LiquidTransportSimulation, Box<dyn std::error::Error>> {
    let surface = SlabFreeSurface::new(
        g.clone(),
        Axis::Y,
        layers,
        VolumeStamp { id: 4, version: 0 },
    )?;
    let fraction = (0..g.cell_len())
        .map(|i| if surface.is_wet_cell(i) { 1.0 } else { 0.0 })
        .collect();
    Ok(LiquidTransportSimulation::with_free_surface(
        g,
        config(),
        method,
        fraction,
        surface,
    )?)
}
fn bits(s: &LiquidTransportSimulation) -> Vec<u64> {
    let v = s.state();
    v.pressure
        .iter()
        .chain(v.liquid.fraction)
        .map(|x| x.to_bits())
        .chain(
            v.carrier
                .x
                .iter()
                .chain(v.carrier.y)
                .chain(v.carrier.z)
                .chain(v.carrier.tracer)
                .map(|x| u64::from(x.to_bits())),
        )
        .chain([
            v.carrier.time.to_bits(),
            v.liquid.time.to_bits(),
            v.carrier.generation,
            v.liquid.stamp.version,
        ])
        .collect()
}
fn save(s: &LiquidTransportSimulation, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let view = s.state();
    let g = s.grid();
    let [nx, ny, nz] = g.counts();
    let mut cells = BufWriter::new(fs::File::create(path.with_extension("csv"))?);
    writeln!(cells, "i,j,k,pressure,fraction")?;
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let index = g.cell_index([i, j, k]).unwrap();
                writeln!(
                    cells,
                    "{i},{j},{k},{:.17e},{:.17e}",
                    view.pressure[index], view.liquid.fraction[index]
                )?;
            }
        }
    }
    cells.flush()?;
    let face_name = format!(
        "{}-faces.csv",
        path.file_name()
            .ok_or("missing frame name")?
            .to_string_lossy()
    );
    let mut faces = BufWriter::new(fs::File::create(path.with_file_name(face_name))?);
    writeln!(faces, "axis,i,j,k,velocity")?;
    for (d, axis) in [Axis::X, Axis::Y, Axis::Z].into_iter().enumerate() {
        let field = [view.carrier.x, view.carrier.y, view.carrier.z][d];
        let dims = g.face_counts(axis);
        for k in 0..dims[2] {
            for j in 0..dims[1] {
                for i in 0..dims[0] {
                    writeln!(
                        faces,
                        "{d},{i},{j},{k},{:.17e}",
                        field[g.face_index(axis, [i, j, k]).unwrap()]
                    )?;
                }
            }
        }
    }
    faces.flush()?;
    write_liquid_guidance_png(s, fs::File::create(path.with_extension("png"))?, nx * ny)?;
    Ok(())
}
fn run(output: &Path, method: PressureImplementation) -> Result<(), Box<dyn std::error::Error>> {
    let root = output.join(method.id());
    fs::create_dir(&root)?;
    let gravity = [BodyForce {
        value: [0.0, -0.25, 0.0],
        units: ForceUnits::Acceleration,
        region: None,
    }];
    let mut rest = make(GridGeometry::new([1, 2, 1], [1.0; 3], [0.0; 3])?, 1, method)?;
    save(&rest, &root.join("rest-initial"))?;
    let mut csv = BufWriter::new(fs::File::create(root.join("rest-steps.csv"))?);
    writeln!(
        csv,
        "step,time,dt,pressure,volume_before,volume_after,inward,outward,source,balance,budget,wet_divergence"
    )?;
    for step in 1..=16 {
        let r = rest.step(input(&gravity, 0.5), |_| false)?;
        writeln!(
            csv,
            "{step},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
            r.carrier.step.time,
            r.carrier.step.dt,
            rest.state().pressure[0],
            r.liquid.liquid_volume_before,
            r.liquid.liquid_volume_after,
            r.liquid.inward_boundary_volume,
            r.liquid.outward_boundary_volume,
            r.liquid.source_volume,
            r.liquid.volume_balance_error,
            r.liquid.volume_rounding_budget,
            r.carrier.step.actual_divergence_max
        )?;
    }
    csv.flush()?;
    save(&rest, &root.join("rest-final"))?;
    let mut pulse = make(GridGeometry::new([2, 2, 1], [1.0; 3], [0.0; 3])?, 1, method)?;
    let forces = [
        BodyForce {
            value: [0.0, 0.5, 0.0],
            units: ForceUnits::Acceleration,
            region: Some(ForceRegion {
                lower: [0.0; 3],
                upper: [1.0, 2.0, 1.0],
            }),
        },
        BodyForce {
            value: [0.0, -0.5, 0.0],
            units: ForceUnits::Acceleration,
            region: Some(ForceRegion {
                lower: [1.0, 0.0, 0.0],
                upper: [2.0, 2.0, 1.0],
            }),
        },
    ];
    save(&pulse, &root.join("pulse-initial"))?;
    let r = pulse.step(input(&forces, 0.5), |_| false)?;
    save(&pulse, &root.join("pulse-final"))?;
    let before = bits(&pulse);
    let failure = pulse
        .step(input(&forces, 0.5), |_| false)
        .expect_err("unresolved next geometry must reject");
    if bits(&pulse) != before {
        return Err("failed next interval changed accepted pair".into());
    }
    fs::write(
        root.join("pulse.json"),
        format!(
            "{{\"time\":{},\"volume_before\":{},\"volume_after\":{},\"inward\":{},\"outward\":{},\"source\":{},\"balance\":{},\"budget\":{},\"wet_divergence\":{},\"next_interval_rejected\":true,\"accepted_bits_preserved\":true,\"next_error\":\"{:?}\",\"owned_array_bytes\":{}}}\n",
            r.carrier.step.time,
            r.liquid.liquid_volume_before,
            r.liquid.liquid_volume_after,
            r.liquid.inward_boundary_volume,
            r.liquid.outward_boundary_volume,
            r.liquid.source_volume,
            r.liquid.volume_balance_error,
            r.liquid.volume_rounding_budget,
            r.carrier.step.actual_divergence_max,
            failure,
            pulse.allocated_bytes()
        ),
    )?;
    let mut probes = BufWriter::new(fs::File::create(root.join("rounding-probes.csv"))?);
    writeln!(
        probes,
        "wet_layers,accepted,preserved_on_rejection,time,result"
    )?;
    for layers in [2, 4, 8, 16] {
        let g = GridGeometry::new(
            [1, layers + 1, 1],
            [1.0, 1.0 / layers as f64, 1.0],
            [0.0; 3],
        )?;
        let mut s = make(g, layers as usize, method)?;
        let before = bits(&s);
        match s.step(input(&gravity, 0.125), |_| false) {
            Ok(_) => writeln!(
                probes,
                "{layers},true,true,{},accepted",
                s.state().carrier.time
            )?,
            Err(error) => {
                let preserved = bits(&s) == before;
                if !preserved {
                    return Err("rounding failure changed accepted state".into());
                }
                writeln!(
                    probes,
                    "{layers},false,{preserved},{},\"{error:?}\"",
                    s.state().carrier.time
                )?;
            }
        }
    }
    probes.flush()?;
    println!(
        "{}: 16 resolved rest steps; conservative first surface pulse; mixed next interval rejected",
        method.id()
    );
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args_os()
        .nth(1)
        .ok_or("usage: free_surface FRESH_OUTPUT")?;
    let output = Path::new(&output);
    fs::create_dir(output)?;
    for method in PressureImplementation::ALL {
        run(output, method)?;
    }
    fs::write(
        output.join("complete.json"),
        "{\"model\":\"cell-aligned-slab-snapshot\",\"methods\":2,\"surface_reconstruction\":false,\"variable_density\":false}\n",
    )?;
    Ok(())
}
