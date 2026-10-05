//! Conservative represented volume and independent translated-slab shape errors.
use rheon::{
    Axis, GridGeometry, LiquidFlowInterval, LiquidInlet, LiquidVolumeSettings, LiquidVolumeState,
    VolumeStamp,
};
use std::fmt::Write;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args().nth(1).map(std::path::PathBuf::from);
    if let Some(path) = &output {
        std::fs::create_dir(path)?;
    }
    println!(
        "scenario,cells,courant,steps,time,liquid_volume,liquid_mass,max_balance_error,cell_average_l1_volume_error,centroid_error,mixed_cells,array_bytes"
    );
    for (n, courant, name) in [
        (16, 0.5, "slab-n16-c050"),
        (32, 0.5, "slab-n32-c050"),
        (64, 0.5, "slab-n64-c050"),
        (64, 0.25, "slab-n64-c025"),
    ] {
        let h = 1.0 / n as f64;
        let grid = GridGeometry::new([n, 1, 1], [h, 1.0, 1.0], [0.0; 3])?;
        let initial: Vec<f64> = (0..n)
            .map(|i| if i >= n / 4 && i < n / 2 { 1.0 } else { 0.0 })
            .collect();
        let mut volume = LiquidVolumeState::new(
            grid.clone(),
            1000.0,
            VolumeStamp { id: 1, version: 0 },
            initial,
            1024 * 1024,
        )?;
        let mut velocity = [Axis::X, Axis::Y, Axis::Z].map(|a| vec![0.0; grid.face_len(a)]);
        velocity[0].fill(0.25);
        let inlet = LiquidInlet::new(VolumeStamp { id: 4, version: 0 }, [[0.0; 2]; 3])?;
        let settings = LiquidVolumeSettings {
            max_outward_courant: 1.0,
            actual_divergence_limit: 0.0,
        };
        let dt = courant * h / 0.25;
        let steps = (0.5 / dt) as usize;
        let mut max_error = 0.0_f64;
        let mut final_report = None;
        for k in 0..steps {
            let flow = LiquidFlowInterval::new(
                &grid,
                VolumeStamp {
                    id: 2,
                    version: k as u64,
                },
                [&velocity[0], &velocity[1], &velocity[2]],
                volume.state().time,
                dt,
            )?;
            let report = volume.advance(flow, inlet, None, settings, |_| false)?;
            max_error = max_error.max(report.volume_balance_error.abs());
            final_report = Some(report);
        }
        let report = final_report.expect("positive interval has at least one update");
        let mut error = 0.0;
        let mut moment = 0.0;
        let mut data = String::from("cell,center,fraction,exact_fraction\n");
        for (i, &fraction) in volume.state().fraction.iter().enumerate() {
            let center = (i as f64 + 0.5) * h;
            let exact = (((i + 1) as f64 * h).min(0.625) - (i as f64 * h).max(0.375)).max(0.0) / h;
            error += h * (fraction - exact).abs();
            moment += h * fraction * center;
            writeln!(data, "{i},{center:.17e},{fraction:.17e},{exact:.17e}")?;
        }
        if let Some(path) = &output {
            std::fs::write(path.join(format!("{name}.csv")), data)?;
        }
        println!(
            "{name},{n},{courant},{steps},{:.17e},{:.17e},{:.17e},{max_error:.17e},{error:.17e},{:.17e},{},{}",
            report.time,
            report.liquid_volume_after,
            report.liquid_mass_after,
            (moment / report.liquid_volume_after - 0.5).abs(),
            report.mixed_cells,
            volume.allocated_bytes()
        );
    }
    eprintln!(
        "First-order represented-volume transport; no free-surface pressure or mesh-wall coupling."
    );
    Ok(())
}
