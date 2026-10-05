//! Prescribed column geometry/cap exchanges; no carrier or physical clock.
use rheon::*;
use std::{fs, io::Write, path::Path};
fn cell(g: &GridGeometry, d: usize, c: usize, j: usize) -> usize {
    let mut p = [0; 3];
    let mut r = c;
    for (i, v) in p.iter_mut().enumerate() {
        if i != d {
            *v = r % g.counts()[i];
            r /= g.counts()[i];
        }
    }
    p[d] = j;
    g.cell_index(p).unwrap()
}
fn surface(
    g: &GridGeometry,
    axis: Axis,
    d: usize,
    h: &[f64],
    version: u64,
) -> Result<ColumnSurfaceWorkspace, FreeSurfaceError> {
    let mut f = vec![0.0; g.cell_len()];
    for (c, &height) in h.iter().enumerate() {
        for j in 0..g.counts()[d] {
            f[cell(g, d, c, j)] = (height - j as f64).clamp(0.0, 1.0);
        }
    }
    ColumnSurfaceWorkspace::new(
        g.clone(),
        axis,
        &f,
        VolumeStamp { id: 31, version },
        1 << 24,
    )
}
fn render(
    path: &Path,
    g: &GridGeometry,
    d: usize,
    u: &[Vec<f32>; 2],
) -> Result<(), Box<dyn std::error::Error>> {
    let n = g.counts()[d];
    let cols = g.cell_len() / n;
    let width = cols * 2;
    let mut pixels = Vec::with_capacity(width * n);
    for j in (0..n).rev() {
        for c in 0..cols {
            for v in u {
                let value = f64::from(v[cell(g, d, c, j)]);
                pixels.push(((value + 4.0) * 255.0 / 8.0).round() as u8);
            }
        }
    }
    let mut encoder = png::Encoder::new(fs::File::create(path)?, width as u32, n as u32);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(&pixels)?;
    Ok(())
}
fn run(
    root: &Path,
    name: &str,
    axis: Axis,
    n: usize,
    kind: &str,
    steps: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let d = match axis {
        Axis::X => 0,
        Axis::Y => 1,
        Axis::Z => 2,
    };
    let mut counts = [1; 3];
    counts[d] = n as u64;
    counts[(d + 1) % 3] = 2;
    let mut spacing = [0.5, 0.25, 0.75];
    spacing[d] = if kind == "affine" {
        1.0 / (n as f64 - 1.75)
    } else {
        1.0
    };
    let g = GridGeometry::new(counts, spacing, [0.0; 3])?;
    let h = spacing[d];
    let mut heights = if kind == "affine" {
        vec![n as f64 - 1.75; 2]
    } else {
        vec![2.75, 2.25]
    };
    let mut u = [vec![0.0; g.cell_len()], vec![0.0; g.cell_len()]];
    for (c, &height) in heights.iter().enumerate() {
        let wet = height.floor() as usize + usize::from(height.fract() > 0.5);
        for j in 0..wet {
            let hi = if j + 1 == wet {
                height * h
            } else {
                (j + 1) as f64 * h
            };
            let lo = j as f64 * h;
            let value = match kind {
                "constant" => 0.25,
                "affine" => 1.0 + 0.5 * (lo + hi),
                _ => {
                    if c == 0 {
                        [0.0, 1.0, 3.0][j]
                    } else {
                        [0.0, -1.0][j]
                    }
                }
            };
            u[0][cell(&g, d, c, j)] = value as f32;
            u[1][cell(&g, d, c, j)] = (-0.5 * value) as f32;
        }
    }
    let dest = root.join(name);
    fs::create_dir(&dest)?;
    render(&dest.join("initial.png"), &g, d, &u)?;
    let mut json = fs::File::create(dest.join("case.json"))?;
    write!(
        json,
        "{{\"name\":\"{name}\",\"kind\":\"{kind}\",\"normal\":{d},\"counts\":{counts:?},\"spacing\":{spacing:?},\"density\":3.0,\"field_kind\":\"cell_lattice_lumped_tangential_parcel_means\",\"pressure_published\":false,\"physical_time_claimed\":false,\"raster_range\":[-4.0,4.0],\"steps\":["
    )?;
    let mut w = ColumnMomentumWorkspace::new(g.clone(), axis, 24 * g.cell_len())?;
    for step in 0..steps {
        let next = if kind == "affine" {
            heights.iter().map(|v| v + 0.5).collect::<Vec<_>>()
        } else if kind == "identity" {
            heights.clone()
        } else {
            vec![heights[1], heights[0]]
        };
        let mut donor = [[0.0; 2]; 2];
        if kind == "affine" {
            for c in 0..2 {
                let avg = 1.0 + 0.5 * (heights[c] + next[c]) * h;
                donor[c] = [avg as f32, (-0.5 * avg) as f32];
            }
        } else {
            for c in 0..2 {
                if next[c] > heights[c] {
                    let from = 1 - c;
                    let wet =
                        heights[from].floor() as usize + usize::from(heights[from].fract() > 0.5);
                    donor[c] = [
                        u[0][cell(&g, d, from, wet - 1)],
                        u[1][cell(&g, d, from, wet - 1)],
                    ];
                }
            }
        }
        let before = surface(&g, axis, d, &heights, step as u64)?;
        let after = surface(&g, axis, d, &next, step as u64 + 1)?;
        let mut v = [vec![0.0; g.cell_len()], vec![0.0; g.cell_len()]];
        let [a, b] = &mut v;
        let r = w.remap(
            ColumnMomentumInputs {
                before: before.state(),
                after: after.state(),
                density: 3.0,
                velocity: [&u[0], &u[1]],
                added_velocity: Some(&donor),
            },
            [a, b],
            |_| false,
        )?;
        if step > 0 {
            write!(json, ",")?;
        }
        write!(
            json,
            "{{\"index\":{step},\"before_heights\":{heights:?},\"after_heights\":{next:?},\"before\":{u:?},\"after\":{v:?},\"donor\":{donor:?},\"report\":{{"
        )?;
        for (key, value) in [
            ("mass_before", r.mass_before),
            ("mass_after", r.mass_after),
            ("mass_added", r.mass_added),
            ("mass_removed", r.mass_removed),
            ("mass_error", r.mass_error),
            ("mass_budget", r.mass_budget),
            ("kinetic_before", r.kinetic_before),
            ("kinetic_after", r.kinetic_after),
            ("kinetic_added", r.kinetic_added),
            ("kinetic_removed", r.kinetic_removed),
            ("mixing_loss", r.mixing_loss),
            ("rounding_work", r.rounding_work),
            ("energy_error", r.energy_error),
            ("energy_budget", r.energy_budget),
        ] {
            write!(json, "\"{key}\":{value:.17e},")?;
        }
        for (key, value) in [
            ("momentum_before", r.momentum_before),
            ("momentum_after", r.momentum_after),
            ("momentum_added", r.momentum_added),
            ("momentum_removed", r.momentum_removed),
            ("rounding_momentum", r.rounding_momentum),
            ("momentum_error", r.momentum_error),
            ("momentum_budget", r.momentum_budget),
        ] {
            write!(json, "\"{key}\":{value:?},")?;
        }
        write!(
            json,
            "\"activated_nodes\":{},\"deactivated_nodes\":{},\"workspace_bytes\":{},\"before_version\":{},\"after_version\":{}}}}}",
            r.activated_nodes,
            r.deactivated_nodes,
            r.workspace_bytes,
            r.before.version,
            r.after.version
        )?;
        heights = next;
        u = v;
    }
    writeln!(json, "]}}")?;
    render(&dest.join("final.png"), &g, d, &u)?;
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arg = std::env::args().nth(1).ok_or("output directory required")?;
    let root = Path::new(&arg);
    fs::create_dir(root)?;
    for (axis, label) in [(Axis::X, "x"), (Axis::Y, "y"), (Axis::Z, "z")] {
        for (kind, steps) in [("exchange", 32), ("constant", 8), ("identity", 1)] {
            run(root, &format!("{kind}-{label}"), axis, 4, kind, steps)?;
        }
    }
    for n in [10, 18, 34, 66] {
        run(root, &format!("affine-n{}", n - 2), Axis::Y, n, "affine", 1)?;
    }
    println!(
        "PASS 13 prescribed-geometry cases / 127 remaps; no physical time/pressure publication"
    );
    Ok(())
}
