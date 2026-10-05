//! Native column reconstruction laboratory; imposed advection is labelled
//! separately from coupled pressure dynamics.
use rheon::*;
use std::{
    fs,
    io::{BufWriter, Write},
    path::Path,
};
fn stamp(id: u64) -> VolumeStamp {
    VolumeStamp { id, version: 0 }
}
fn config() -> LiquidTransportConfig {
    LiquidTransportConfig {
        carrier: SimulationConfig {
            density: 1.0,
            memory_limit: 1 << 24,
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
        volume_stamp: stamp(1),
        carrier_id: 2,
    }
}
fn inputs(forces: &[BodyForce]) -> LiquidStepInputs<'_> {
    LiquidStepInputs {
        requested_dt: 0.125,
        smoke_source: None,
        forces,
        inlet: LiquidInlet::new(stamp(3), [[0.0; 2]; 3]).unwrap(),
        source: None,
        volume: LiquidVolumeSettings {
            max_outward_courant: 1.0,
            actual_divergence_limit: 1e-5,
        },
    }
}
fn cells(
    path: &Path,
    g: &GridGeometry,
    fraction: &[f64],
    pressure: Option<&[f64]>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut csv = BufWriter::new(fs::File::create(path)?);
    writeln!(csv, "i,j,k,fraction,pressure")?;
    let [nx, ny, nz] = g.counts();
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let cell = g.cell_index([i, j, k]).unwrap();
                writeln!(
                    csv,
                    "{i},{j},{k},{:.17e},{:.17e}",
                    fraction[cell],
                    pressure.map_or(0.0, |p| p[cell])
                )?;
            }
        }
    }
    csv.flush()?;
    Ok(())
}
fn faces(path: &Path, g: &GridGeometry, v: [&[f32]; 3]) -> Result<(), Box<dyn std::error::Error>> {
    let mut csv = BufWriter::new(fs::File::create(path)?);
    writeln!(csv, "axis,i,j,k,velocity")?;
    for (d, axis) in [Axis::X, Axis::Y, Axis::Z].into_iter().enumerate() {
        let [nx, ny, nz] = g.face_counts(axis);
        for k in 0..nz {
            for j in 0..ny {
                for i in 0..nx {
                    writeln!(
                        csv,
                        "{d},{i},{j},{k},{:.17e}",
                        v[d][g.face_index(axis, [i, j, k]).unwrap()]
                    )?;
                }
            }
        }
    }
    csv.flush()?;
    Ok(())
}
fn frame(
    root: &Path,
    name: &str,
    s: &LiquidTransportSimulation,
) -> Result<(), Box<dyn std::error::Error>> {
    let view = s.state();
    cells(
        &root.join(format!("{name}-cells.csv")),
        s.grid(),
        view.liquid.fraction,
        Some(view.pressure),
    )?;
    faces(
        &root.join(format!("{name}-faces.csv")),
        s.grid(),
        [view.carrier.x, view.carrier.y, view.carrier.z],
    )?;
    let current = view.reconstructed_surface.ok_or("missing end geometry")?;
    let held = view.pressure_columns.ok_or("missing pressure geometry")?;
    let mut csv = BufWriter::new(fs::File::create(root.join(format!("{name}-geometry.csv")))?);
    writeln!(
        csv,
        "column,full_layers,top_fraction,geometry_version,pressure_full_layers,pressure_top_fraction,pressure_geometry_version"
    )?;
    for (index, (&end, &old)) in current.heights().iter().zip(held.heights()).enumerate() {
        writeln!(
            csv,
            "{index},{},{:.17e},{},{},{:.17e},{}",
            end.full_layers(),
            end.top_fraction(),
            current.stamp().version,
            old.full_layers(),
            old.top_fraction(),
            held.stamp().version
        )?;
    }
    csv.flush()?;
    write_liquid_guidance_png(
        s,
        fs::File::create(root.join(format!("{name}.png")))?,
        s.grid().counts()[0] * s.grid().counts()[1],
    )?;
    Ok(())
}
fn ledger_header(csv: &mut impl Write) -> std::io::Result<()> {
    writeln!(
        csv,
        "step,time,dt,volume_before,volume_after,inward,outward,source,balance,budget,wet_divergence,reconstruction_change,collapsed_columns,geometry_version,pressure_geometry_version"
    )
}
fn ledger(
    csv: &mut impl Write,
    step: usize,
    r: LiquidVolumeReport,
    g: ColumnReconstructionReport,
) -> std::io::Result<()> {
    writeln!(
        csv,
        "{step},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{},{},{}",
        r.time,
        r.dt,
        r.liquid_volume_before,
        r.liquid_volume_after,
        r.inward_boundary_volume,
        r.outward_boundary_volume,
        r.source_volume,
        r.volume_balance_error,
        r.volume_rounding_budget,
        r.actual_divergence_max,
        r.reconstruction_volume_change,
        g.collapsed_columns,
        g.output_stamp.version,
        g.input_stamp.version
    )
}
fn coupled(
    output: &Path,
    method: PressureImplementation,
    rest: bool,
    activation: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let name = format!(
        "{}-{}",
        method.id(),
        if rest {
            "mixed-rest"
        } else if activation {
            "activation"
        } else {
            "pulse"
        }
    );
    let root = output.join(&name);
    fs::create_dir(&root)?;
    let g = GridGeometry::new([2, if activation { 4 } else { 3 }, 1], [1.0; 3], [0.0; 3])?;
    let fraction = if rest {
        vec![1.0, 1.0, 0.25, 0.25, 0.0, 0.0]
    } else if activation {
        vec![1.0, 1.0, 0.5, 0.5, 0.0, 0.0, 0.0, 0.0]
    } else {
        vec![1.0, 1.0, 0.0, 0.0, 0.0, 0.0]
    };
    let mut s = LiquidTransportSimulation::with_reconstructed_surface(
        g,
        config(),
        method,
        fraction,
        Axis::Y,
    )?;
    let force = if rest {
        vec![BodyForce {
            value: [0.0, -0.25, 0.0],
            units: ForceUnits::Acceleration,
            region: None,
        }]
    } else {
        vec![
            BodyForce {
                value: [0.0, 0.5, 0.0],
                units: ForceUnits::Acceleration,
                region: Some(ForceRegion {
                    lower: [0.0; 3],
                    upper: [1.0, 3.0, 1.0],
                }),
            },
            BodyForce {
                value: [0.0, -0.5, 0.0],
                units: ForceUnits::Acceleration,
                region: Some(ForceRegion {
                    lower: [1.0, 0.0, 0.0],
                    upper: [2.0, 3.0, 1.0],
                }),
            },
        ]
    };
    frame(&root, "initial", &s)?;
    let mut csv = BufWriter::new(fs::File::create(root.join("steps.csv"))?);
    ledger_header(&mut csv)?;
    let steps = if rest { 16 } else { 8 };
    for step in 1..=steps {
        let r = s.step(inputs(&force), |_| false)?;
        ledger(
            &mut csv,
            step,
            r.liquid,
            r.column_reconstruction.ok_or("missing geometry report")?,
        )?;
        frame(&root, &format!("frame-{step:02}"), &s)?;
    }
    csv.flush()?;
    frame(&root, "final", &s)?;
    fs::write(
        root.join("run.json"),
        format!(
            "{{\"kind\":\"coupled-{}\",\"method\":\"{}\",\"counts\":[2,{},1],\"spacing\":[1,1,1],\"steps\":{},\"time\":{},\"owned_array_bytes\":{},\"axis\":\"Y\",\"surface_model\":\"bottom-attached-column-closure\",\"pressure_is_held_interval\":true}}\n",
            if rest {
                "mixed-rest"
            } else if activation {
                "activation"
            } else {
                "pulse"
            },
            method.id(),
            s.grid().counts()[1],
            steps,
            s.state().carrier.time,
            s.allocated_bytes()
        ),
    )?;
    println!("{name}: {steps} coupled intervals");
    Ok(())
}
fn primitive(x: f64) -> f64 {
    if x <= 0.25 {
        x
    } else if x >= 0.75 {
        0.5
    } else {
        0.25 + 0.5 * (x - 0.25)
            + (2.0 * std::f64::consts::PI * (x - 0.25)).sin() / (4.0 * std::f64::consts::PI)
    }
}
fn kinematic(
    output: &Path,
    n: u64,
    c: f64,
    vertical: Option<f32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let name = if let Some(v) = vertical {
        format!("vertical-{}", if v > 0.0 { "up" } else { "down" })
    } else {
        format!("advection-n{n}-c{}", if c == 0.25 { "025" } else { "0125" })
    };
    let root = output.join(&name);
    fs::create_dir(&root)?;
    let g = if vertical.is_some() {
        GridGeometry::new([1, 4, 1], [1.0; 3], [0.0; 3])?
    } else {
        GridGeometry::new([n, 4, 2], [1.0 / n as f64, 0.25, 0.5], [0.0; 3])?
    };
    let [nx, ny, nz] = g.counts();
    let mut fraction = vec![0.0; g.cell_len()];
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let cell = g.cell_index([i, j, k]).unwrap();
                fraction[cell] = if let Some(v) = vertical {
                    if j == 0 || (v < 0.0 && j == 1) {
                        1.0
                    } else if (v > 0.0 && j == 1) || (v < 0.0 && j == 2) {
                        0.25
                    } else {
                        0.0
                    }
                } else if j == 0 {
                    1.0
                } else if j == 1 {
                    (primitive((i + 1) as f64 / n as f64) - primitive(i as f64 / n as f64))
                        * n as f64
                } else {
                    0.0
                };
            }
        }
    }
    let mut geometry =
        ColumnSurfaceWorkspace::new(g.clone(), Axis::Y, &fraction, stamp(1), 1 << 24)?;
    let mut phase = LiquidVolumeState::new(g.clone(), 1.0, stamp(1), fraction, 1 << 24)?;
    let mut velocity = [
        vec![0.0; g.face_len(Axis::X)],
        vec![0.0; g.face_len(Axis::Y)],
        vec![0.0; g.face_len(Axis::Z)],
    ];
    if let Some(v) = vertical {
        velocity[1].fill(v);
    } else {
        let dims = g.face_counts(Axis::X);
        for k in 0..dims[2] {
            for j in 0..2 {
                for i in 0..dims[0] {
                    velocity[0][g.face_index(Axis::X, [i, j, k]).unwrap()] = 0.25;
                }
            }
        }
    }
    let dt = if vertical.is_some() {
        0.5
    } else {
        c * g.spacing()[0] / 0.25
    };
    let steps = if vertical.is_some() {
        16
    } else {
        (0.5 / dt).round() as usize
    };
    let inlet = if vertical.is_some() {
        [[0.0; 2], [1.0, 0.0], [0.0; 2]]
    } else {
        [[1.0; 2], [0.0; 2], [0.0; 2]]
    };
    cells(
        &root.join("initial-cells.csv"),
        &g,
        phase.state().fraction,
        None,
    )?;
    faces(
        &root.join("faces.csv"),
        &g,
        [&velocity[0], &velocity[1], &velocity[2]],
    )?;
    let mut csv = BufWriter::new(fs::File::create(root.join("steps.csv"))?);
    ledger_header(&mut csv)?;
    for step in 1..=steps {
        let flow = LiquidFlowInterval::new(
            &g,
            stamp(2),
            [&velocity[0], &velocity[1], &velocity[2]],
            phase.state().time,
            dt,
        )?;
        let r = phase.advance_reconstructed(
            ColumnVolumeInputs {
                flow,
                inlet: LiquidInlet::new(stamp(3), inlet)?,
                source: None,
                settings: LiquidVolumeSettings {
                    max_outward_courant: 1.0,
                    actual_divergence_limit: 1e-9,
                },
            },
            &mut geometry,
            |_| false,
        )?;
        ledger(&mut csv, step, r.volume, r.geometry)?;
    }
    csv.flush()?;
    cells(
        &root.join("final-cells.csv"),
        &g,
        phase.state().fraction,
        None,
    )?;
    fs::write(
        root.join("run.json"),
        format!(
            "{{\"kind\":\"imposed-{}\",\"counts\":[{nx},{ny},{nz}],\"spacing\":[{},{},{}],\"steps\":{steps},\"dt\":{dt},\"time\":{},\"speed\":{},\"courant\":{c},\"phase_array_bytes\":{},\"geometry_array_bytes\":{},\"pressure_solved\":false,\"axis\":\"Y\"}}\n",
            if vertical.is_some() {
                "vertical"
            } else {
                "transverse-advection"
            },
            g.spacing()[0],
            g.spacing()[1],
            g.spacing()[2],
            phase.state().time,
            vertical.map_or(0.25, f64::from),
            phase.allocated_bytes(),
            geometry.allocated_bytes()
        ),
    )?;
    println!("{name}: {steps} imposed-velocity geometric intervals");
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arg = std::env::args_os()
        .nth(1)
        .ok_or("usage: column_interface FRESH_OUTPUT")?;
    let output = Path::new(&arg);
    fs::create_dir(output)?;
    for method in PressureImplementation::ALL {
        coupled(output, method, false, false)?;
        coupled(output, method, false, true)?;
        coupled(output, method, true, false)?;
    }
    for v in [0.125, -0.125] {
        kinematic(output, 1, 0.25, Some(v))?;
    }
    for n in [16, 32, 64, 128] {
        kinematic(output, n, 0.25, None)?;
    }
    kinematic(output, 64, 0.125, None)?;
    fs::write(
        output.join("complete.json"),
        "{\"scenarios\":13,\"model\":\"bounded-column-closure\",\"imposed_advection_separate_from_pressure\":true}\n",
    )?;
    Ok(())
}
