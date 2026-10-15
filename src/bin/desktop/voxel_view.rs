//! Shared native voxel geometry, water palette and depth-sorted egui mesh.
use super::camera::OrbitCamera;
use eframe::egui::{self, Color32, Rect};
#[derive(Clone)]
pub struct Face {
    pub points: [[f32; 3]; 4],
    pub color: Color32,
}
pub fn water_color(density: u8, shade: f32, material: Option<[u8; 3]>, alpha: u8) -> Color32 {
    let t = density as f32 / 255.0;
    let rgb = if let Some(base) = material {
        let brightness = (0.55 + 0.45 * t) * shade;
        base.map(|v| (v as f32 * brightness) as u8)
    } else {
        [
            ((24.0 + 85.0 * t) * shade) as u8,
            ((136.0 + 90.0 * t) * shade) as u8,
            ((210.0 + 44.0 * t) * shade) as u8,
        ]
    };
    Color32::from_rgba_unmultiplied(rgb[0], rgb[1], rgb[2], alpha)
}
pub fn point(counts: [usize; 3], extents: [f32; 3], cell: [usize; 3]) -> [f32; 3] {
    let scale = extents.into_iter().fold(0.0_f32, f32::max);
    std::array::from_fn(|d| {
        (1.0 - extents[d] / scale) * 0.5 + cell[d] as f32 * (extents[d] / counts[d] as f32 / scale)
    })
}
pub fn faces(
    counts: [usize; 3],
    extents: [f32; 3],
    density: &[u8],
    threshold: u8,
    opaque: bool,
    color: impl Fn(usize, f32, u8) -> Color32,
) -> Vec<Face> {
    let [nx, ny, nz] = counts;
    let mut faces = Vec::new();
    let threshold = threshold.max(1);
    for z in 0..nz {
        for y in 0..ny {
            for x in 0..nx {
                let p = [x, y, z];
                let row = x + nx * (y + ny * z);
                let value = density[row];
                if value < threshold {
                    continue;
                }
                let lo = point(counts, extents, p);
                let hi = point(counts, extents, p.map(|n| n + 1));
                for d in 0..3 {
                    for positive in [false, true] {
                        let mut q = p;
                        let boundary = if positive {
                            p[d] + 1 == counts[d]
                        } else {
                            p[d] == 0
                        };
                        if !boundary && opaque {
                            if positive {
                                q[d] += 1;
                            } else {
                                q[d] -= 1;
                            }
                            if density[q[0] + nx * (q[1] + ny * q[2])] >= threshold {
                                continue;
                            }
                        }
                        let a = (d + 1) % 3;
                        let b = (d + 2) % 3;
                        let points = std::array::from_fn(|corner| {
                            let mut v = lo;
                            v[d] = if positive { hi[d] } else { lo[d] };
                            if corner == 1 || corner == 2 {
                                v[a] = hi[a];
                            }
                            if corner >= 2 {
                                v[b] = hi[b];
                            }
                            v
                        });
                        let shade = match (d, positive) {
                            (1, true) => 1.0,
                            (2, true) => 0.83,
                            (0, true) => 0.73,
                            _ => 0.55,
                        };
                        faces.push(Face {
                            points,
                            color: color(row, shade, value),
                        });
                    }
                }
            }
        }
    }
    faces
}
pub fn mesh(mut faces: Vec<Face>, camera: &OrbitCamera, rect: Rect) -> egui::Mesh {
    let basis = camera.basis();
    let forward = basis.1;
    let depth = |face: &Face| {
        let center = std::array::from_fn::<_, 3, _>(|d| {
            face.points.iter().map(|p| p[d]).sum::<f32>() * 0.25
        });
        center
            .into_iter()
            .zip(forward)
            .map(|(a, b)| a * b)
            .sum::<f32>()
    };
    faces.sort_unstable_by(|a, b| depth(b).total_cmp(&depth(a)));
    let mut mesh = egui::Mesh::default();
    for face in faces {
        let projected = face.points.map(|p| camera.project_prepared(p, rect, basis));
        if let [Some((a, _)), Some((b, _)), Some((c, _)), Some((d, _))] = projected {
            let first = mesh.vertices.len() as u32;
            for pos in [a, b, c, d] {
                mesh.colored_vertex(pos, face.color);
            }
            mesh.indices.extend_from_slice(&[
                first,
                first + 1,
                first + 2,
                first,
                first + 2,
                first + 3,
            ]);
        }
    }
    mesh
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn translucent_cells_keep_internal_layers_and_cubic_edges() {
        let colors = |_, shade, value| water_color(value, shade, None, 12);
        assert_eq!(
            faces([2; 3], [1.0; 3], &[255; 8], 1, true, colors).len(),
            24
        );
        let transparent = faces([2; 3], [1.0; 3], &[255; 8], 1, false, colors);
        assert_eq!(transparent.len(), 48);
        assert!(transparent.iter().all(|f| f.color.a() == 12));
        let counts = [16, 10, 6];
        let extents = [1.0, 0.625, 0.375];
        let origin = point(counts, extents, [0; 3]);
        let edges = std::array::from_fn::<_, 3, _>(|d| {
            let mut p = [0; 3];
            p[d] = 1;
            (point(counts, extents, p)[d] - origin[d]).abs()
        });
        assert_eq!(edges, [0.0625; 3]);
    }
}
