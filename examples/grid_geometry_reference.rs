//! Q2 common-geometry prerequisite; no coupled or production-water claim.
mod grid_geometry_reference {
    pub mod diagnostics;
    pub mod geometry;
}
use grid_geometry_reference::{diagnostics, geometry::Polygon};
use std::{io::Write, path::Path};
fn disk(n: usize) -> Polygon {
    Polygon::new(
        (0..8 * n)
            .map(|k| {
                let theta = std::f64::consts::TAU * k as f64 / (8 * n) as f64;
                [0.75 * theta.cos(), 0.75 * theta.sin()]
            })
            .collect(),
    )
    .unwrap()
}
fn square(degrees: f64) -> Polygon {
    let a = degrees.to_radians();
    let (s, c) = a.sin_cos();
    Polygon::new(
        [[-0.15, -0.15], [0.15, -0.15], [0.15, 0.15], [-0.15, 0.15]]
            .into_iter()
            .map(|[x, y]| [0.537 + c * x - s * y, 0.479 + s * x + c * y])
            .collect(),
    )
    .unwrap()
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::env::args_os()
        .nth(1)
        .expect("fresh output directory required");
    let root = Path::new(&out);
    std::fs::create_dir(root)?;
    let mut summary = std::io::BufWriter::new(std::fs::File::create(root.join("summary.csv"))?);
    writeln!(
        summary,
        "case,n,mu,source_vertices,area,mx,my,radial_second,exact_area,exact_mx,exact_my,exact_radial_second,normal_closure,linear_closure,cubic_defect,solenoidal_defect,traction_defect,face_mismatch,flux,exact_flux,force_x,force_y,exact_force,wrong_force_x,wrong_force_y,dual_relative_defect,min_positive_volume,mxx,mxy,myy,exact_mxx,exact_mxy,exact_myy"
    )?;
    for case in ["disk", "square-0", "square-17", "square-31"] {
        let levels = if case == "disk" {
            [32, 64, 128]
        } else {
            [16, 32, 64]
        };
        for n in levels {
            let (source, origin, extent, exact_area, center, exact_radial) = if case == "disk" {
                let area = std::f64::consts::PI * 0.75_f64.powi(2);
                (
                    disk(n),
                    [-1.0, -1.0],
                    2.0,
                    area,
                    [0.0, 0.0],
                    std::f64::consts::PI * 0.75_f64.powi(4) / 2.0,
                )
            } else {
                let degrees = case.strip_prefix("square-").unwrap().parse::<f64>()?;
                (
                    square(degrees),
                    [0.0, 0.0],
                    1.0,
                    0.09,
                    [0.537, 0.479],
                    0.09 * (0.537_f64.powi(2) + 0.479_f64.powi(2) + 0.3_f64.powi(2) / 6.0),
                )
            };
            let vertices = source.vertices.len();
            let g = grid_geometry_reference::geometry::Geometry::new(n, origin, extent, source)?;
            for mu in [0.1, 0.001002] {
                let tag = format!("{case}-n{n}-mu{mu}");
                let m = diagnostics::run(&root.join(&tag), &g, mu)?;
                let exact_second = if case == "disk" {
                    [exact_radial / 2.0, 0.0, exact_radial / 2.0]
                } else {
                    [
                        exact_area * (center[0] * center[0] + 0.3 * 0.3 / 12.0),
                        exact_area * center[0] * center[1],
                        exact_area * (center[1] * center[1] + 0.3 * 0.3 / 12.0),
                    ]
                };
                writeln!(
                    summary,
                    "{case},{n},{mu:.17e},{vertices},{:.17e},{:.17e},{:.17e},{:.17e},{exact_area:.17e},{:.17e},{:.17e},{exact_radial:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
                    m.area,
                    m.first[0],
                    m.first[1],
                    m.radial_second,
                    exact_area * center[0],
                    exact_area * center[1],
                    m.closure,
                    m.linear_closure,
                    m.cubic_defect,
                    m.solenoidal_defect,
                    m.traction_defect,
                    m.face_mismatch,
                    m.flux,
                    3.0 * exact_radial,
                    m.force[0],
                    m.force[1],
                    mu * exact_area,
                    m.wrong_pressure_force[0],
                    m.wrong_pressure_force[1],
                    m.dual_relative_defect,
                    m.min_positive_volume,
                    m.second[0],
                    m.second[1],
                    m.second[2],
                    exact_second[0],
                    exact_second[1],
                    exact_second[2]
                )?;
                eprintln!(
                    "Q2 geometry {tag}: area={:.12e}, closure={:.3e}, flux={:.3e}, stress={:.3e}",
                    m.area, m.linear_closure, m.cubic_defect, m.traction_defect
                );
            }
        }
    }
    summary.flush()?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use grid_geometry_reference::geometry::{Geometry, Segment, moments};
    #[test]
    fn independent_rectangle_moments_and_closed_flux() {
        let p = Polygon::new(vec![[0.1, 0.2], [0.7, 0.2], [0.7, 0.9], [0.1, 0.9]]).unwrap();
        let v = p.clip([0.3, 0.1], [0.9, 0.8]).unwrap();
        let m = moments(&v).unwrap();
        assert!((m.area - 0.4 * 0.6).abs() < 1e-15);
        assert!((m.first[0] - 0.4 * 0.6 * 0.5).abs() < 1e-15);
        assert!((m.second[2] - 0.4 * (0.8_f64.powi(3) - 0.2_f64.powi(3)) / 3.0).abs() < 1e-15);
        let flux: f64 = grid_geometry_reference::geometry::edges(&v)
            .map(|(a, b)| Segment { a, b }.integrate(diagnostics::cubic))
            .sum();
        assert!((flux - 3.0 * (m.second[0] + m.second[2])).abs() < 1e-15);
    }
    #[test]
    fn unsupported_orientation_coincidence_and_domain_refuse() {
        assert!(Polygon::new(vec![[0.0, 0.0], [0.0, 1.0], [1.0, 0.0]]).is_err());
        let doubled = vec![
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [0.0, 1.0],
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [0.0, 1.0],
        ];
        assert!(Polygon::new(doubled).is_err());
        assert!(moments(&[[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]]).is_err());
        let p = Polygon::new(vec![[0.0, 0.0], [0.5, 0.0], [0.5, 0.5], [0.0, 0.5]]).unwrap();
        assert!(
            Geometry::new(4, [0.0, 0.0], 1.0, p)
                .err()
                .unwrap()
                .contains("coincident")
        );
        assert!(Geometry::new(2, [0.0, 0.0], 0.1, square(17.0)).is_err());
    }
    #[test]
    fn nonzero_pressure_traction_and_wrong_boundary_are_distinct() {
        let s = Segment {
            a: [0.0, 0.0],
            b: [1.0, 0.0],
        };
        let full = diagnostics::traction(s, 0.1, true);
        let wrong = diagnostics::traction(s, 0.1, false);
        assert!((full[1] - wrong[1] - 1.05).abs() < 1e-15);
    }
}
