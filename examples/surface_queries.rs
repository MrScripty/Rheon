//! Static geometry-query example; no fluid topology or moving-body solve.
use rheon::{SurfaceSettings, SurfaceStamp, TriangleSurface};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("case,surface_id,version,parameter,triangle,x,y,z,normal_x,normal_y,normal_z");
    for (version, shift) in [-0.1, 0.0, 0.1].into_iter().enumerate() {
        let mut vertices = vec![];
        for z in [-0.3, 0.3] {
            for y in [-0.3, 0.3] {
                for x in [-0.3, 0.3] {
                    vertices.push([x + shift, y, z]);
                }
            }
        }
        let mesh = TriangleSurface::new(
            SurfaceStamp {
                id: 1,
                version: version as u64,
            },
            vertices,
            vec![
                [0, 2, 3],
                [0, 3, 1],
                [4, 5, 7],
                [4, 7, 6],
                [0, 1, 5],
                [0, 5, 4],
                [2, 6, 7],
                [2, 7, 3],
                [0, 4, 6],
                [0, 6, 2],
                [1, 3, 7],
                [1, 7, 5],
            ],
            SurfaceSettings::default(),
        )?;
        for (label, start, end) in [
            ("crossing", [-0.8, 0.1, 0.0], [0.8, 0.1, 0.0]),
            ("miss", [-0.8, 0.5, 0.0], [0.8, 0.5, 0.0]),
            ("inside-start", [shift, 0.0, 0.0], [0.8, 0.0, 0.0]),
        ] {
            let clip = mesh.clip_segment(start, end, |_| false)?;
            if let Some(h) = clip.hit {
                println!(
                    "{},{},{},{:.17e},{},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
                    label,
                    h.surface.id,
                    h.surface.version,
                    h.parameter,
                    h.triangle,
                    clip.end[0],
                    clip.end[1],
                    clip.end[2],
                    h.normal[0],
                    h.normal[1],
                    h.normal[2]
                );
            } else {
                println!(
                    "{},{},{},,,{:.17e},{:.17e},{:.17e},,,",
                    label,
                    mesh.stamp().id,
                    mesh.stamp().version,
                    clip.end[0],
                    clip.end[1],
                    clip.end[2]
                );
            }
        }
        eprintln!(
            "static surface version {version}; retained array payload {} bytes",
            mesh.allocated_bytes()
        );
    }
    Ok(())
}
