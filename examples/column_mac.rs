//! Actual existing-owner static MAC materialization and accepted-state export.
use rheon::*;
use std::{fs, io::Write, path::Path};
fn coordinate(i: usize, n: [usize; 3]) -> [usize; 3] {
    [i % n[0], (i / n[0]) % n[1], i / (n[0] * n[1])]
}
fn render(
    path: &Path,
    g: &GridGeometry,
    normal: usize,
    wet: usize,
    fields: [&[f32]; 3],
) -> Result<(), Box<dyn std::error::Error>> {
    let tangents = match normal {
        0 => [1, 2],
        1 => [0, 2],
        _ => [0, 1],
    };
    let width = g.counts()[tangents[0]] + 1;
    let height = g.counts()[tangents[1]];
    let mut pixels = Vec::with_capacity(width * height);
    for j in 0..height {
        for i in 0..width {
            let mut p = [0; 3];
            p[normal] = wet - 1;
            p[tangents[0]] = i;
            p[tangents[1]] = j;
            let value = f64::from(
                fields[tangents[0]][g
                    .face_index([Axis::X, Axis::Y, Axis::Z][tangents[0]], p)
                    .unwrap()],
            );
            if !(-1.0..=1.0).contains(&value) {
                return Err("raster range exceeded".into());
            }
            pixels.push(((value + 1.0) * 127.5).round() as u8);
        }
    }
    let mut encoder = png::Encoder::new(fs::File::create(path)?, width as u32, height as u32);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(&pixels)?;
    Ok(())
}
fn transfer(out: &mut fs::File, r: ColumnMacTransferReport) -> std::io::Result<()> {
    write!(
        out,
        "{{\"geometry_id\":{},\"geometry_version\":{},\"direction\":\"{:?}\",",
        r.geometry.id, r.geometry.version, r.direction
    )?;
    for (key, value) in [
        ("profile_mass", r.profile_mass),
        ("mass_budget", r.mass_budget),
        ("kinetic_before", r.kinetic_before),
        ("kinetic_after", r.kinetic_after),
        ("wall_energy_removed", r.wall_energy_removed),
        ("mixing_loss", r.mixing_loss),
        ("rounding_work", r.rounding_work),
        ("energy_error", r.energy_error),
        ("energy_budget", r.energy_budget),
        ("omitted_normal_momentum", r.omitted_normal_momentum),
        ("omitted_normal_energy", r.omitted_normal_energy),
    ] {
        write!(out, "\"{key}\":{value:.17e},")?;
    }
    for (key, value) in [
        ("momentum_before", r.momentum_before),
        ("momentum_after", r.momentum_after),
        ("wall_impulse", r.wall_impulse),
        ("rounding_momentum", r.rounding_momentum),
        ("momentum_error", r.momentum_error),
        ("momentum_budget", r.momentum_budget),
    ] {
        write!(out, "\"{key}\":{value:?},")?;
    }
    write!(
        out,
        "\"mac_mass\":{:?},\"mass_error\":{:?},\"workspace_bytes\":{}}}",
        r.mac_mass, r.mass_error, r.workspace_bytes
    )
}
fn publication(out: &mut fs::File, r: ColumnMacPublicationReport) -> std::io::Result<()> {
    let p = r.projection;
    write!(
        out,
        "{{\"before_carrier_version\":{},\"after_carrier_version\":{},\"before_volume_version\":{},\"after_volume_version\":{},\"accepted_energy_before\":{:.17e},\"prescribed_energy_change\":{:.17e},\"accepted_momentum_before\":{:?},\"prescribed_momentum_change\":{:?},\"owned_array_bytes\":{},\"workspace_array_bytes\":{},\"total_array_bytes\":{},\"transfer\":",
        r.before.carrier.version,
        r.after.carrier.version,
        r.before.volume.version,
        r.after.volume.version,
        r.accepted_energy_before,
        r.prescribed_energy_change,
        r.accepted_momentum_before,
        r.prescribed_momentum_change,
        r.owned_array_bytes,
        r.workspace_array_bytes,
        r.total_array_bytes
    )?;
    transfer(out, r.transfer)?;
    write!(
        out,
        ",\"before_carrier_id\":{},\"after_carrier_id\":{},\"before_volume_id\":{},\"after_volume_id\":{}",
        r.before.carrier.id, r.after.carrier.id, r.before.volume.id, r.after.volume.id
    )?;
    write!(
        out,
        ",\"represented_phase_mass\":{:.17e},\"phase_mass_error\":{:.17e},\"phase_mass_budget\":{:.17e}",
        r.represented_phase_mass, r.phase_mass_error, r.phase_mass_budget
    )?;
    write!(out, ",\"projection\":{{")?;
    for (key, value) in [
        ("actual_divergence_max", p.actual_divergence_max),
        ("kinetic_before", p.kinetic_before),
        ("kinetic_after", p.kinetic_after),
        ("correction_energy", p.correction_energy),
        ("residual_work", p.residual_work),
        ("rounding_work", p.rounding_work),
        ("energy_error", p.energy_error),
        ("energy_budget", p.energy_budget),
        ("true_residual_l2", p.pressure.true_residual_l2),
        ("true_residual_max", p.pressure.true_residual_max),
        (
            "predicted_divergence_max",
            p.pressure.predicted_divergence_max,
        ),
    ] {
        write!(out, "\"{key}\":{value:.17e},")?;
    }
    for (key, value) in [
        ("momentum_before", p.momentum_before),
        ("momentum_after", p.momentum_after),
        ("pressure_impulse", p.pressure_impulse),
        ("rounding_momentum", p.rounding_momentum),
        ("momentum_error", p.momentum_error),
        ("momentum_budget", p.momentum_budget),
    ] {
        write!(out, "\"{key}\":{value:?},")?;
    }
    write!(out, "\"iterations\":{}}}}}", p.pressure.iterations)
}
#[allow(clippy::too_many_arguments)]
fn run(
    root: &Path,
    name: &str,
    axis: Axis,
    top: f64,
    n: usize,
    method: PressureImplementation,
    steps: usize,
    kind: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let normal = match axis {
        Axis::X => 0,
        Axis::Y => 1,
        Axis::Z => 2,
    };
    let tangents = match normal {
        0 => [1, 2],
        1 => [0, 2],
        _ => [0, 1],
    };
    let mut counts = [n as u64; 3];
    counts[normal] = 4;
    let mut spacing = [1.0 / n as f64; 3];
    spacing[normal] = 0.25;
    let g = GridGeometry::new(counts, spacing, [0.0; 3])?;
    let height = 2.0 + top;
    let wet = 2 + usize::from(top > 0.5);
    let fractions = (0..g.cell_len())
        .map(|i| (height - coordinate(i, g.counts())[normal] as f64).clamp(0.0, 1.0))
        .collect::<Vec<_>>();
    let initial_fractions = fractions.clone();
    let cfg = LiquidTransportConfig {
        carrier: SimulationConfig {
            density: 1.0,
            memory_limit: 1 << 28,
            pressure: PressureSettings {
                relative_residual: 1e-12,
                absolute_residual: 1e-12,
                divergence_limit: 1e-9,
                max_iterations: 4000,
            },
            actual_divergence_limit: 1e-5,
            max_courant: 1.0,
        },
        represented_density: 1.0,
        volume_stamp: VolumeStamp {
            id: 11,
            version: 12,
        },
        carrier_id: 17,
    };
    let mut owner = LiquidTransportSimulation::with_reconstructed_surface(
        g.clone(),
        cfg,
        method,
        fractions,
        axis,
    )?;
    let mut w = ColumnMacWorkspace::new(g.clone(), 1 << 27)?;
    let mut u: [Vec<f32>; 2] = std::array::from_fn(|t| {
        (0..g.cell_len())
            .map(|i| {
                let p = coordinate(i, g.counts());
                if p[normal] >= wet {
                    0.0
                } else if kind == "curl" {
                    let x = (p[tangents[0]] as f64 + 0.5) / n as f64;
                    let z = (p[tangents[1]] as f64 + 0.5) / n as f64;
                    let pi = std::f64::consts::PI;
                    if t == 0 {
                        ((pi * x).sin() * (pi * z).cos()) as f32
                    } else {
                        (-(pi * x).cos() * (pi * z).sin()) as f32
                    }
                } else {
                    (0.1 * (1 + p[tangents[t]]) as f64 * (1 + p[normal]) as f64) as f32
                }
            })
            .collect()
    });
    let dest = root.join(name);
    fs::create_dir(&dest)?;
    let mut out = fs::File::create(dest.join("case.json"))?;
    write!(
        out,
        "{{\"name\":\"{name}\",\"kind\":\"{kind}\",\"normal\":{normal},\"top_fraction\":{top},\"counts\":{counts:?},\"spacing\":{spacing:?},\"density\":1.0,\"projection_dt\":0.125,\"implementation\":\"{}\",\"model\":\"fixed_flat_static_mac_materialization\",\"wall_velocity\":0.0,\"atmospheric_pressure\":0.0,\"physical_time_advanced\":false,\"moving_surface_dynamics\":false,\"velocity_raster_range\":[-1.0,1.0],\"initial_fractions\":{initial_fractions:?},\"steps\":[",
        method.id()
    )?;
    for step in 0..steps {
        let state = owner.state();
        let before = [
            state.carrier.x.to_vec(),
            state.carrier.y.to_vec(),
            state.carrier.z.to_vec(),
        ];
        let expected = ColumnMacStateStamp {
            carrier: state.carrier_stamp,
            volume: state.liquid.stamp,
        };
        let geom = FlatColumnMacGeometry::new(state.reconstructed_surface.unwrap(), 1.0)?;
        let mut lifted: [Vec<f32>; 3] =
            std::array::from_fn(|d| vec![0.0; g.face_len([Axis::X, Axis::Y, Axis::Z][d])]);
        let [a, b, c] = &mut lifted;
        w.lift(geom, [&u[0], &u[1]], [a, b, c], |_| false)?;
        if step == 0 {
            render(
                &dest.join("initial.png"),
                &g,
                normal,
                wet,
                [&lifted[0], &lifted[1], &lifted[2]],
            )?;
        }
        let r = owner.materialize_flat_column_profiles(
            &mut w,
            ColumnMacPublishInputs {
                expected,
                profiles: [&u[0], &u[1]],
                projection_dt: 0.125,
            },
            |_| false,
        )?;
        let state = owner.state();
        if state.liquid.fraction != initial_fractions
            || state.carrier.time != 0.0
            || state.liquid.time != 0.0
            || !state.flat_column_mac
            || state.pressure_columns.unwrap().stamp() != state.liquid.stamp
            || state.reconstructed_surface.unwrap().stamp() != state.liquid.stamp
        {
            return Err("shared accepted state violated".into());
        }
        let mut exported: [Vec<f32>; 2] = std::array::from_fn(|_| vec![0.0; g.cell_len()]);
        let [a, b] = &mut exported;
        let er = owner.export_flat_column_profiles(&mut w, r.after, [a, b], |_| false)?;
        if step > 0 {
            write!(out, ",")?;
        }
        write!(
            out,
            "{{\"index\":{step},\"input_profiles\":{u:?},\"before_velocity\":{before:?},\"lifted_velocity\":{lifted:?},\"after_velocity\":{:?},\"pressure\":{:?},\"exported_profiles\":{exported:?},\"physical_time\":0.0,\"phase_unchanged\":true,\"flat_column_mac\":true,\"pressure_geometry_version\":{},\"end_geometry_version\":{},\"publication\":",
            [state.carrier.x, state.carrier.y, state.carrier.z],
            state.pressure,
            state.pressure_columns.unwrap().stamp().version,
            state.reconstructed_surface.unwrap().stamp().version
        )?;
        publication(&mut out, r)?;
        write!(
            out,
            ",\"export_carrier_id\":{},\"export_carrier_version\":{},\"export_volume_id\":{},\"export_volume_version\":{},\"pressure_geometry_id\":{},\"end_geometry_id\":{}",
            er.source.carrier.id,
            er.source.carrier.version,
            er.source.volume.id,
            er.source.volume.version,
            state.pressure_columns.unwrap().stamp().id,
            state.reconstructed_surface.unwrap().stamp().id
        )?;
        write!(out, ",\"export\":")?;
        transfer(&mut out, er.transfer)?;
        write!(out, "}}")?;
        u = exported;
    }
    writeln!(out, "]}}")?;
    let state = owner.state();
    render(
        &dest.join("final.png"),
        &g,
        normal,
        wet,
        [state.carrier.x, state.carrier.y, state.carrier.z],
    )?;
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arg = std::env::args().nth(1).ok_or("output directory required")?;
    let root = Path::new(&arg);
    fs::create_dir(root)?;
    for (axis, label) in [(Axis::X, "x"), (Axis::Y, "y"), (Axis::Z, "z")] {
        for top in [0.25, 0.5, 0.75] {
            for method in PressureImplementation::ALL {
                let method_name = if method == PressureImplementation::JacobiPcgV1 {
                    "jacobi"
                } else {
                    "sgs"
                };
                run(
                    root,
                    &format!("pulse-{label}-{top}-{method_name}"),
                    axis,
                    top,
                    2,
                    method,
                    4,
                    "pulse",
                )?;
            }
        }
    }
    for n in [8, 16, 32, 64] {
        run(
            root,
            &format!("curl-n{n}"),
            Axis::Y,
            0.25,
            n,
            PressureImplementation::JacobiPcgV1,
            1,
            "curl",
        )?;
    }
    println!(
        "PASS 22 cases / 76 existing-owner static publications and accepted-profile exports; no clock advancement"
    );
    Ok(())
}
