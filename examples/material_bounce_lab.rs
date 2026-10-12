//! Reproduce floor rebound with real material steps; optional source duration.
use rheon::{
    FluidPreset, GridGeometry, ParticleFluidConfig, ParticleFluidSimulation,
    RotatingCylinderEmitter,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let steps: usize = args.first().map_or(Ok(12000), |s| s.parse())?;
    let source_until: f64 = args.get(1).map_or(Ok(2.0), |s| s.parse())?;
    let preset = match args.get(2).map(String::as_str) {
        Some("mercury") => FluidPreset::Mercury,
        Some("slime") => FluidPreset::Slime,
        _ => FluidPreset::Water,
    };
    let mut s = ParticleFluidSimulation::new(
        GridGeometry::new([32; 3], [1.0 / 32.0; 3], [0.0; 3])?,
        ParticleFluidConfig {
            material: preset.material(),
            ..ParticleFluidConfig::default()
        },
    )?;
    let e = RotatingCylinderEmitter {
        pressure_heads: [1000.0, 800.0],
        ..RotatingCylinderEmitter::default()
    };
    let mut floor_hit = false;
    let mut peak_y = 0.0_f64;
    let mut peak_vy = 0.0_f64;
    for step in 0..steps {
        let r = s.step(0.02, (s.time() < source_until).then_some(e), || false)?;
        floor_hit |= s.particles().iter().any(|p| p.position[1] < 0.02);
        if floor_hit {
            peak_y = peak_y.max(
                s.particles()
                    .iter()
                    .map(|p| p.position[1])
                    .fold(0.0_f64, f64::max),
            );
            peak_vy = peak_vy.max(
                s.particles()
                    .iter()
                    .map(|p| p.velocity[1])
                    .fold(0.0_f64, f64::max),
            );
        }
        if step % 1000 == 0 || step + 1 == steps || (floor_hit && peak_y > 0.95) {
            let energy = s
                .particles()
                .iter()
                .map(|p| {
                    p.mass
                        * (0.5 * p.velocity.iter().map(|v| v * v).sum::<f64>()
                            + 9.80665 * p.position[1])
                })
                .sum::<f64>();
            println!(
                "step={step} time={:.6} particles={} floor={floor_hit} peak_y={peak_y:.6} peak_vy={peak_vy:.6} speed={:.6} pressure={:.3} energy={energy:.6}",
                s.time(),
                r.particles,
                r.max_speed,
                r.max_pressure
            );
        }
        if floor_hit && peak_y > 0.95 {
            return Err("floor rebound reached the ceiling".into());
        }
    }
    Ok(())
}
