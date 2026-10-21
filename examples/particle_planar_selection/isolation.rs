//! Infinite planar volumetric slab screen, deliberately no imported mesh claim.
use super::kernel::Kernel;
#[derive(Clone, Copy, Debug)]
pub struct Slab {
    normal: [f64; 3],
    lower: f64,
    upper: f64,
}
pub fn run(output: &mut impl std::io::Write) -> std::io::Result<()> {
    writeln!(
        output,
        "angle_deg,thickness_over_spacing,volumetric_support,unmasked_density_kernel,masked_density_kernel,masked_pressure_gradient,masked_viscous_kernel"
    )?;
    let spacing = 0.025;
    let k = Kernel {
        radius: 4.0 * spacing,
    };
    for angle in [0.0_f64, 17.0, 31.0] {
        let theta = angle.to_radians();
        let normal = [theta.cos(), theta.sin(), 0.0];
        for cells in [8.1, 4.0, 2.0, 1.0, 0.5, 0.0] {
            let thickness = cells * spacing;
            let slab = Slab::new(normal, -thickness / 2.0, thickness / 2.0).unwrap();
            let offset = thickness / 2.0 + 0.01 * k.radius;
            let a = normal.map(|v| -offset * v);
            let b = normal.map(|v| offset * v);
            let values = slab.pair(a, b, k);
            writeln!(
                output,
                "{},{},{},{:.17e},{:.17e},{:.17e},{:.17e}",
                angle,
                cells,
                if slab.admit_volumetric_support(k.radius).is_ok() {
                    "supported-screen"
                } else {
                    "refused"
                },
                k.volume(2.0 * offset),
                values[0],
                values[1],
                values[2]
            )?;
        }
    }
    Ok(())
}
#[derive(Debug, PartialEq)]
pub enum SlabError {
    InvalidGeometry,
    UnresolvedSupport,
}
impl Slab {
    pub fn new(normal: [f64; 3], lower: f64, upper: f64) -> Result<Self, SlabError> {
        let norm = normal.iter().map(|v| v * v).sum::<f64>().sqrt();
        if normal.iter().any(|v| !v.is_finite())
            || !lower.is_finite()
            || !upper.is_finite()
            || upper < lower
            || (norm - 1.0).abs() > 1e-12
        {
            return Err(SlabError::InvalidGeometry);
        }
        Ok(Self {
            normal,
            lower,
            upper,
        })
    }
    pub fn admit_volumetric_support(self, radius: f64) -> Result<Self, SlabError> {
        if !radius.is_finite() || radius <= 0.0 {
            return Err(SlabError::InvalidGeometry);
        }
        if self.upper - self.lower < 2.0 * radius {
            return Err(SlabError::UnresolvedSupport);
        }
        Ok(self)
    }
    fn side(self, p: [f64; 3]) -> i8 {
        let d: f64 = self.normal.iter().zip(p).map(|(n, p)| n * p).sum();
        if d < self.lower {
            -1
        } else if d > self.upper {
            1
        } else {
            0
        }
    }
    pub fn visible(self, a: [f64; 3], b: [f64; 3]) -> bool {
        let side = self.side(a);
        side != 0 && side == self.side(b)
    }
    /// All three native contributions share the same side/segment gate.
    pub fn pair(self, a: [f64; 3], b: [f64; 3], k: Kernel) -> [f64; 3] {
        if !self.visible(a, b) {
            return [0.0; 3];
        }
        let r2: f64 = a.iter().zip(b).map(|(a, b)| (a - b).powi(2)).sum();
        let r = r2.sqrt();
        [k.volume(r), k.phi(r) * r, k.phi(r)]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rotated_slab_oracle_covers_all_three_interactions_and_refusals() {
        for degrees in [0.0_f64, 17.0, 31.0] {
            let theta = degrees.to_radians();
            let normal = [theta.cos(), theta.sin(), 0.0];
            let k = Kernel { radius: 0.2 };
            let slab = Slab::new(normal, -0.25, 0.25)
                .unwrap()
                .admit_volumetric_support(k.radius)
                .unwrap();
            // Independent local-coordinate construction/inverse rotation.
            for x in [-0.7, -0.26, 0.0, 0.26, 0.7] {
                for y in [-0.7, -0.26, 0.0, 0.26, 0.7] {
                    let a = [theta.cos() * x, theta.sin() * x, 0.01];
                    let b = [theta.cos() * y, theta.sin() * y, -0.01];
                    let crosses = if x.min(y) <= 0.25 && x.max(y) >= -0.25 {
                        // Infinite slab intersects the full segment interval.
                        true
                    } else {
                        false
                    };
                    assert_eq!(slab.visible(a, b), !crosses);
                    if crosses {
                        assert_eq!(slab.pair(a, b, k), [0.0; 3]);
                    }
                }
            }
            let thin = Slab::new(normal, -0.01, 0.01).unwrap();
            let a = normal.map(|v| v * -0.02);
            let b = normal.map(|v| v * 0.02);
            assert!(k.volume(0.04) > 0.0 && k.phi(0.04) > 0.0);
            assert_eq!(thin.pair(a, b, k), [0.0; 3]);
            assert_eq!(
                Slab::new(normal, -0.1, 0.1)
                    .unwrap()
                    .admit_volumetric_support(k.radius)
                    .unwrap_err(),
                SlabError::UnresolvedSupport
            );
            assert_eq!(
                Slab::new(normal, 0.0, 0.0)
                    .unwrap()
                    .admit_volumetric_support(k.radius)
                    .unwrap_err(),
                SlabError::UnresolvedSupport
            );
        }
    }
}
