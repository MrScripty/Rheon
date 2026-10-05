//! Native isolated symmetric-strain decay and separately labelled coupled box.
use rheon::*;
use std::{
    fs,
    io::{BufWriter, Write},
    path::Path,
};
fn fields(g: &GridGeometry) -> [Vec<f32>; 3] {
    [
        vec![0.0; g.face_len(Axis::X)],
        vec![0.0; g.face_len(Axis::Y)],
        vec![0.0; g.face_len(Axis::Z)],
    ]
}
fn refs(v: &[Vec<f32>; 3]) -> [&[f32]; 3] {
    [&v[0], &v[1], &v[2]]
}
fn muts(v: &mut [Vec<f32>; 3]) -> [&mut [f32]; 3] {
    let [x, y, z] = v;
    [x, y, z]
}
fn mode(g: &GridGeometry) -> [Vec<f32>; 3] {
    let mut v = fields(g);
    let [n, _, nz] = g.counts();
    for k in 0..nz {
        for j in 0..n {
            for i in 1..n {
                v[0][g.face_index(Axis::X, [i, j, k]).unwrap()] =
                    ((std::f64::consts::PI * i as f64 / n as f64).sin()
                        * (std::f64::consts::PI * (j as f64 + 0.5) / n as f64).cos())
                        as f32;
                v[1][g.face_index(Axis::Y, [j, i, k]).unwrap()] =
                    -v[0][g.face_index(Axis::X, [i, j, k]).unwrap()];
            }
        }
    }
    v
}
fn face_csv(
    path: &Path,
    g: &GridGeometry,
    v: [&[f32]; 3],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut csv = BufWriter::new(fs::File::create(path)?);
    writeln!(csv, "axis,i,j,k,velocity")?;
    for (d, a) in [Axis::X, Axis::Y, Axis::Z].into_iter().enumerate() {
        let m = g.face_counts(a);
        for k in 0..m[2] {
            for j in 0..m[1] {
                for i in 0..m[0] {
                    writeln!(
                        csv,
                        "{d},{i},{j},{k},{:.17e}",
                        v[d][g.face_index(a, [i, j, k]).unwrap()]
                    )?;
                }
            }
        }
    }
    csv.flush()?;
    Ok(())
}
fn velocity_png(
    path: &Path,
    g: &GridGeometry,
    v: [&[f32]; 3],
) -> Result<(), Box<dyn std::error::Error>> {
    let [nx, ny, nz] = g.counts();
    let mut pixels = Vec::new();
    for j in (0..ny).rev() {
        for i in 0..nx {
            let a = v[0][g.face_index(Axis::X, [i, j, nz / 2]).unwrap()];
            let b = v[0][g.face_index(Axis::X, [i + 1, j, nz / 2]).unwrap()];
            let value = 0.5 + 0.25 * f64::from(a + b);
            pixels.push((255.0 * value).round().clamp(0.0, 255.0) as u8);
        }
    }
    let mut encoder = png::Encoder::new(
        BufWriter::new(fs::File::create(path)?),
        nx as u32,
        ny as u32,
    );
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(&pixels)?;
    Ok(())
}
fn report(
    csv: &mut impl Write,
    step: usize,
    r: ViscosityReport,
    amplitude: f64,
    modal: f64,
) -> std::io::Result<()> {
    writeln!(
        csv,
        "{step},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
        r.dt,
        r.stability_number,
        r.kinetic_before,
        r.kinetic_after,
        r.dissipation_before,
        r.update_energy,
        r.rounding_work,
        r.energy_identity_error,
        r.energy_rounding_budget,
        amplitude,
        modal
    )
}
fn header(csv: &mut impl Write) -> std::io::Result<()> {
    writeln!(
        csv,
        "step,dt,stability,kinetic_before,kinetic_after,dissipation_before,update_energy,rounding_work,identity_error,rounding_budget,amplitude,modal_error"
    )
}
fn isolated(
    root: &Path,
    n: u64,
    nu: f64,
    steps: usize,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let case = root.join(label);
    fs::create_dir_all(&case)?;
    let g = GridGeometry::new([n, n, 3], [1.0 / n as f64, 1.0 / n as f64, 0.25], [0.0; 3])?;
    let initial = mode(&g);
    let mut u = initial.clone();
    let mut out = fields(&g);
    let mut w = ViscosityWorkspace::new(g.clone(), 1 << 28)?;
    let dt = 0.1 / steps as f64;
    fs::write(
        case.join("case.json"),
        format!(
            "{{\"kind\":\"isolated\",\"counts\":[{n},{n},3],\"spacing\":[{}, {},0.25],\"density\":3,\"dynamic_viscosity\":{},\"duration\":0.1,\"steps\":{steps},\"workspace_bytes\":{},\"wall\":\"stationary-impermeable-free-slip\",\"free_surface\":false}}\n",
            1.0 / n as f64,
            1.0 / n as f64,
            nu * 3.0,
            w.allocated_bytes()
        ),
    )?;
    face_csv(&case.join("initial-faces.csv"), &g, refs(&u))?;
    velocity_png(&case.join("initial.png"), &g, refs(&u))?;
    let lambda = 8.0 * (std::f64::consts::PI / (2.0 * n as f64)).sin().powi(2) * (n * n) as f64;
    let mut csv = BufWriter::new(fs::File::create(case.join("steps.csv"))?);
    header(&mut csv)?;
    let norm = initial
        .iter()
        .flatten()
        .map(|&x| f64::from(x).powi(2))
        .sum::<f64>();
    for step in 1..=steps {
        let r = w.update(refs(&u), 3.0, 3.0 * nu, dt, muts(&mut out), |_| false)?;
        let amplitude = out
            .iter()
            .flatten()
            .zip(initial.iter().flatten())
            .map(|(&u, &i)| f64::from(u) * f64::from(i))
            .sum::<f64>()
            / norm;
        let expected = (1.0 - nu * dt * lambda).powi(step as i32);
        let modal = out
            .iter()
            .flatten()
            .zip(initial.iter().flatten())
            .map(|(&u, &i)| (f64::from(u) - expected * f64::from(i)).abs())
            .fold(0.0_f64, f64::max);
        report(&mut csv, step, r, amplitude, modal)?;
        std::mem::swap(&mut u, &mut out);
    }
    csv.flush()?;
    face_csv(&case.join("final-faces.csv"), &g, refs(&u))?;
    velocity_png(&case.join("final.png"), &g, refs(&u))?;
    Ok(())
}
fn config() -> LiquidTransportConfig {
    LiquidTransportConfig {
        carrier: SimulationConfig {
            density: 1.0,
            memory_limit: 1 << 20,
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
fn inputs(forces: &[BodyForce]) -> LiquidStepInputs<'_> {
    LiquidStepInputs {
        requested_dt: 0.125,
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
fn forces() -> [BodyForce; 4] {
    [
        ([0.5, 0.0, 0.0], [0.0, 0.0, 0.0], [2.0, 1.0, 1.0]),
        ([-0.5, 0.0, 0.0], [0.0, 1.0, 0.0], [2.0, 2.0, 1.0]),
        ([0.0, -0.5, 0.0], [0.0, 0.0, 0.0], [1.0, 2.0, 1.0]),
        ([0.0, 0.5, 0.0], [1.0, 0.0, 0.0], [2.0, 2.0, 1.0]),
    ]
    .map(|(value, lower, upper)| BodyForce {
        value,
        units: ForceUnits::Acceleration,
        region: Some(ForceRegion { lower, upper }),
    })
}
fn coupled(
    root: &Path,
    method: PressureImplementation,
    mu: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    let case = root.join(format!("{}-coupled-mu{}", method.id(), mu));
    fs::create_dir_all(&case)?;
    let g = GridGeometry::new([2, 2, 1], [1.0; 3], [0.0; 3])?;
    let mut s = LiquidTransportSimulation::new(g.clone(), config(), method, vec![1.0; 4])?;
    let mut w = ViscosityWorkspace::new(g.clone(), 1 << 20)?;
    fs::write(
        case.join("case.json"),
        format!(
            "{{\"kind\":\"coupled\",\"counts\":[2,2,1],\"spacing\":[1,1,1],\"density\":1,\"dynamic_viscosity\":{mu},\"steps\":9,\"workspace_bytes\":{},\"owned_bytes\":{},\"wall\":\"stationary-impermeable-free-slip\",\"free_surface\":false}}\n",
            w.allocated_bytes(),
            s.allocated_bytes()
        ),
    )?;
    let mut csv = BufWriter::new(fs::File::create(case.join("steps.csv"))?);
    header(&mut csv)?;
    let mut ledger = BufWriter::new(fs::File::create(case.join("liquid.csv"))?);
    writeln!(
        ledger,
        "step,time,carrier_version,liquid_version,volume_before,volume_after,inward,outward,source,balance,budget,divergence,owned_bytes,workspace_bytes,total_bytes,accepted_energy"
    )?;
    let f = forces();
    for step in 1..=9 {
        let r = s.step_viscous(inputs(if step == 1 { &f } else { &[] }), &mut w, mu, |_| {
            false
        })?;
        let v = s.state();
        report(&mut csv, step, r.viscosity.unwrap(), 0.0, 0.0)?;
        writeln!(
            ledger,
            "{step},{:.17e},{},{},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{},{},{},{:.17e}",
            v.carrier.time,
            v.carrier_stamp.version,
            v.liquid.stamp.version,
            r.liquid.liquid_volume_before,
            r.liquid.liquid_volume_after,
            r.liquid.inward_boundary_volume,
            r.liquid.outward_boundary_volume,
            r.liquid.source_volume,
            r.liquid.volume_balance_error,
            r.liquid.volume_rounding_budget,
            r.carrier.step.actual_divergence_max,
            r.owned_array_bytes,
            r.viscosity_workspace_array_bytes,
            r.total_array_bytes,
            r.carrier.step.kinetic_energy
        )?;
        face_csv(
            &case.join(format!("frame-{step:02}-faces.csv")),
            &g,
            [v.carrier.x, v.carrier.y, v.carrier.z],
        )?;
        let mut cells = BufWriter::new(fs::File::create(
            case.join(format!("frame-{step:02}-cells.csv")),
        )?);
        writeln!(cells, "cell,fraction,pressure")?;
        for (i, (&fraction, &pressure)) in v.liquid.fraction.iter().zip(v.pressure).enumerate() {
            writeln!(cells, "{i},{fraction:.17e},{pressure:.17e}")?;
        }
        cells.flush()?;
        velocity_png(
            &case.join(format!("frame-{step:02}.png")),
            &g,
            [v.carrier.x, v.carrier.y, v.carrier.z],
        )?;
    }
    csv.flush()?;
    ledger.flush()?;
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args()
        .nth(1)
        .ok_or("provide fresh output directory")?;
    let root = Path::new(&output);
    fs::create_dir(root)?;
    for n in [8, 16, 32, 64] {
        let steps = (0.1 * 0.05 * (2.0 * (n * n) as f64 + 16.0) / 0.1).ceil() as usize;
        isolated(root, n, 0.05, steps, &format!("space-n{n}"))?;
    }
    for steps in [32, 64, 128] {
        isolated(root, 12, 0.05, steps, &format!("time-n12-s{steps}"))?;
    }
    for nu in [0.01_f64, 0.1] {
        let steps = (0.1 * nu * (512.0 + 16.0) / 0.02).ceil() as usize;
        isolated(root, 16, nu, steps, &format!("material-n16-nu{nu}"))?;
    }
    for method in PressureImplementation::ALL {
        for mu in [0.0, 0.125] {
            coupled(root, method, mu)?;
        }
    }
    println!("Completed 9 isolated decay cases and 4 coupled full-box cases");
    Ok(())
}
