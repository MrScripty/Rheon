//! Prescribed surface translation and geometric contact, without a fluid solve.
use rheon::{SurfaceSettings, SurfaceStamp, TranslationInterval, TriangleSurface};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let surface = TriangleSurface::new(
        SurfaceStamp { id: 1, version: 0 },
        vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        vec![[0, 1, 2]],
        SurfaceSettings::default(),
    )?;
    let offsets = [[0.0, 0.0, -1.0], [0.0, 0.0, 1.0]];
    let interval = TranslationInterval::new(&surface, 5, [4.0, 6.0], offsets)?;
    println!("case,interval,parameter,time,x,y,z,wall_x,wall_y,wall_z");
    for (case, start, end) in [
        ("stationary-point", [0.25, 0.25, 0.0], [0.25, 0.25, 0.0]),
        ("moving-point", [0.25, 0.25, 0.2], [0.25, 0.25, 0.4]),
        ("miss", [2.0, 2.0, 0.0], [2.0, 2.0, 0.0]),
    ] {
        let clip = interval.clip_segment(start, end, |_| false)?;
        if let Some(hit) = clip.hit {
            println!(
                "{case},{},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
                hit.interval_id,
                hit.reference_hit.parameter,
                hit.time,
                hit.world_position[0],
                hit.world_position[1],
                hit.world_position[2],
                hit.wall_velocity[0],
                hit.wall_velocity[1],
                hit.wall_velocity[2]
            );
        } else {
            println!("{case},{},,,,,,,,", interval.interval_id());
        }
    }
    eprintln!(
        "{} retained surface-array bytes; interval borrows geometry; no fluid response",
        surface.allocated_bytes()
    );
    Ok(())
}
