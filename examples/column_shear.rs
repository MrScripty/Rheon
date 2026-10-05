//! Flat free-surface shear prerequisite: periodic lateral directions, fixed
//! geometry, constant density, natural zero shear traction at both endpoints.
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
fn fixture(
    axis: Axis,
    full: usize,
    top: f64,
) -> Result<(GridGeometry, ColumnSurfaceWorkspace), Box<dyn std::error::Error>> {
    let d = match axis {
        Axis::X => 0,
        Axis::Y => 1,
        Axis::Z => 2,
    };
    let mut n = [2, 2, 3];
    let mut h = [0.5, 0.5, 0.25];
    n[d] = (full + 2) as u64;
    h[d] = 1.0 / (full as f64 + top);
    let g = GridGeometry::new(n, h, [0.0; 3])?;
    let n = g.counts();
    let mut f = vec![0.0; g.cell_len()];
    for k in 0..n[2] {
        for j in 0..n[1] {
            for i in 0..n[0] {
                let p = [i, j, k];
                f[g.cell_index(p).unwrap()] = if p[d] < full {
                    1.0
                } else if p[d] == full {
                    top
                } else {
                    0.0
                };
            }
        }
    }
    let surface = ColumnSurfaceWorkspace::new(
        g.clone(),
        axis,
        &f,
        VolumeStamp { id: 1, version: 0 },
        1 << 24,
    )?;
    Ok((g, surface))
}
fn fill(g: &GridGeometry, axis: usize, wet: usize, kind: &str) -> [Vec<f32>; 3] {
    let mut u = fields(g);
    let mut t = 0;
    for (d, field) in u.iter_mut().enumerate() {
        if d == axis {
            continue;
        }
        let scale = if t == 0 { 1.0 } else { 0.5 };
        t += 1;
        let m = g.face_counts([Axis::X, Axis::Y, Axis::Z][d]);
        for k in 0..m[2] {
            for j in 0..m[1] {
                for i in 0..m[0] {
                    let p = [i, j, k];
                    let y = (p[axis] as f64 + 0.5) * g.spacing()[axis];
                    field[g.face_index([Axis::X, Axis::Y, Axis::Z][d], p).unwrap()] =
                        if p[axis] >= wet {
                            0.0
                        } else {
                            (scale
                                * match kind {
                                    "translation" => 0.25,
                                    "affine" => y,
                                    _ => (std::f64::consts::PI * y).cos(),
                                }) as f32
                        };
                }
            }
        }
    }
    u
}
fn sample(g: &GridGeometry, v: &[Vec<f32>; 3], d: usize, normal: usize, layer: usize) -> f32 {
    let mut p = [0; 3];
    p[normal] = layer;
    v[d][g.face_index([Axis::X, Axis::Y, Axis::Z][d], p).unwrap()]
}
fn faces(
    path: &Path,
    g: &GridGeometry,
    v: &[Vec<f32>; 3],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut csv = BufWriter::new(fs::File::create(path)?);
    writeln!(csv, "axis,i,j,k,velocity")?;
    for (d, field) in v.iter().enumerate() {
        let m = g.face_counts([Axis::X, Axis::Y, Axis::Z][d]);
        for k in 0..m[2] {
            for j in 0..m[1] {
                for i in 0..m[0] {
                    writeln!(
                        csv,
                        "{d},{i},{j},{k},{:.17e}",
                        field[g
                            .face_index([Axis::X, Axis::Y, Axis::Z][d], [i, j, k])
                            .unwrap()]
                    )?;
                }
            }
        }
    }
    csv.flush()?;
    Ok(())
}
fn png(
    path: &Path,
    g: &GridGeometry,
    v: &[Vec<f32>; 3],
    normal: usize,
    wet: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let d = if normal == 0 { 1 } else { 0 };
    let mut pixels = Vec::new();
    for layer in (0..wet).rev() {
        let u = f64::from(sample(g, v, d, normal, layer));
        let val = (255.0 * (0.5 + 0.5 * u)).round().clamp(0.0, 255.0) as u8;
        pixels.extend(std::iter::repeat_n(val, 24));
    }
    let mut encoder = png::Encoder::new(BufWriter::new(fs::File::create(path)?), 24, wet as u32);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(&pixels)?;
    Ok(())
}
fn case(
    root: &Path,
    axis: Axis,
    full: usize,
    top: f64,
    kind: &str,
    forced_steps: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    let normal = match axis {
        Axis::X => 0,
        Axis::Y => 1,
        Axis::Z => 2,
    };
    let (g, s) = fixture(axis, full, top)?;
    let wet = full + usize::from(top > 0.5);
    let tangents = match normal {
        0 => [1, 2],
        1 => [0, 2],
        _ => [0, 1],
    };
    let default_steps = if kind == "decay" {
        (0.05 * 0.1 / (0.1 * g.spacing()[normal].powi(2))).ceil() as usize
    } else {
        1
    };
    let steps = forced_steps.unwrap_or(default_steps);
    let dt = 0.1 / steps as f64;
    let name = if let Some(steps) = forced_steps {
        format!("time-a{normal}-n{full}-f{top}-s{steps}")
    } else {
        format!("{kind}-a{normal}-n{full}-f{top}")
    };
    let dir = root.join(name);
    fs::create_dir(&dir)?;
    let mut u = fill(&g, normal, wet, kind);
    let mut out = fields(&g);
    let mut w = ColumnShearWorkspace::new(g.clone(), axis, 1 << 24)?;
    fs::write(
        dir.join("case.json"),
        format!(
            "{{\"kind\":\"{kind}\",\"normal_axis\":{normal},\"full_layers\":{full},\"top_fraction\":{top},\"counts\":{:?},\"spacing\":{:?},\"density\":3,\"dynamic_viscosity\":0.15,\"duration\":0.1,\"steps\":{steps},\"workspace_bytes\":{},\"geometry_stamp\":[1,0],\"lateral_boundary\":\"periodic\",\"endpoint_shear_traction\":\"zero\",\"coupled_carrier_step\":false}}\n",
            g.counts(),
            g.spacing(),
            w.allocated_bytes()
        ),
    )?;
    faces(&dir.join("initial-faces.csv"), &g, &u)?;
    png(&dir.join("initial.png"), &g, &u, normal, wet)?;
    let mut ledger = BufWriter::new(fs::File::create(dir.join("steps.csv"))?);
    writeln!(
        ledger,
        "step,dt,stability,volume,mass_volume_error,mass_volume_budget,kinetic_before,kinetic_after,dissipation_before,update_energy,rounding_work,identity_error,energy_budget,force0,force1,force_budget0,force_budget1,momentum_before0,momentum_before1,momentum_after0,momentum_after1,rounding_momentum0,rounding_momentum1,momentum_error0,momentum_error1,momentum_budget0,momentum_budget1,geometry_id,geometry_version,workspace_bytes"
    )?;
    let mut profiles = BufWriter::new(fs::File::create(dir.join("profiles.csv"))?);
    writeln!(
        profiles,
        "step,layer,position,dual_length,mass,before0,before1,force0,force1,after0,after1"
    )?;
    for step in 1..=steps {
        let r = w.update(
            ColumnShearInputs {
                surface: s.state(),
                velocity: refs(&u),
                density: 3.0,
                dynamic_viscosity: 0.15,
                dt,
            },
            muts(&mut out),
            |_| false,
        )?;
        let [a, b] = tangents;
        write!(ledger, "{step}")?;
        for value in [
            r.dt,
            r.stability_number,
            r.geometry.liquid_volume,
            r.mass_volume_error,
            r.mass_volume_budget,
            r.kinetic_before,
            r.kinetic_after,
            r.dissipation_before,
            r.update_energy,
            r.rounding_work,
            r.identity_error,
            r.energy_budget,
            r.force_sum[a],
            r.force_sum[b],
            r.force_budget[a],
            r.force_budget[b],
            r.momentum_before[a],
            r.momentum_before[b],
            r.momentum_after[a],
            r.momentum_after[b],
            r.rounding_momentum[a],
            r.rounding_momentum[b],
            r.momentum_error[a],
            r.momentum_error[b],
            r.momentum_budget[a],
            r.momentum_budget[b],
        ] {
            write!(ledger, ",{value:.17e}")?;
        }
        writeln!(
            ledger,
            ",{},{},{}",
            r.geometry.stamp.id, r.geometry.stamp.version, r.workspace_bytes
        )?;
        for layer in 0..wet {
            writeln!(
                profiles,
                "{step},{layer},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
                (layer as f64 + 0.5) * g.spacing()[normal],
                r.geometry.dual_length(layer).unwrap(),
                w.mass_scratch()[layer],
                sample(&g, &u, a, normal, layer),
                sample(&g, &u, b, normal, layer),
                w.force_scratch()[0][layer],
                w.force_scratch()[1][layer],
                sample(&g, &out, a, normal, layer),
                sample(&g, &out, b, normal, layer)
            )?;
        }
        std::mem::swap(&mut u, &mut out);
    }
    ledger.flush()?;
    profiles.flush()?;
    faces(&dir.join("final-faces.csv"), &g, &u)?;
    png(&dir.join("final.png"), &g, &u, normal, wet)?;
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arg = std::env::args()
        .nth(1)
        .ok_or("provide fresh output directory")?;
    let root = Path::new(&arg);
    fs::create_dir(root)?;
    for top in [0.25, 0.5, 0.75] {
        for full in [8, 16, 32, 64] {
            case(root, Axis::Y, full, top, "decay", None)?;
        }
    }
    for steps in [64, 128, 256] {
        case(root, Axis::Y, 12, 0.75, "decay", Some(steps))?;
    }
    for axis in [Axis::X, Axis::Y, Axis::Z] {
        case(root, axis, 3, 0.75, "affine", None)?;
        for top in [0.25, 0.75] {
            case(root, axis, 3, top, "translation", None)?;
        }
    }
    println!("Completed 24 fixed-flat-surface shear prerequisite cases");
    Ok(())
}
