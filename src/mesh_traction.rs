//! Consistent P1 surface loads on retained oriented triangles. No fluid coupling.
use crate::{SurfaceStamp, TriangleSurface};
use std::fmt;

/// Corner values in each triangle's indexed order; discontinuities are allowed.
#[derive(Debug, Clone, Copy)]
pub enum SurfaceLoading<'a> {
    /// World traction ON the supplied surface, in N/m². Independent of winding.
    Traction(&'a [[[f64; 3]; 3]]),
    /// Signed pressure in Pa, with traction -p*n_winding. The caller, not this
    /// API, establishes whether winding points out of a solid and which side
    /// supplied pressure. No solid classification or fluid interpolation.
    Pressure(&'a [[f64; 3]]),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshLoadStage {
    Admission,
    Reduction,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MeshLoadError {
    StaleSurface,
    InvalidReference,
    InvalidTwist,
    LoadCount,
    TriangleLimit {
        required: usize,
        limit: usize,
    },
    InvalidTriangle,
    NonFiniteLoad {
        triangle: usize,
    },
    ArithmeticFailure {
        triangle: usize,
    },
    Cancelled {
        stage: MeshLoadStage,
        triangle: usize,
    },
}
impl fmt::Display for MeshLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "mesh traction rejected operation: {self:?}")
    }
}
impl std::error::Error for MeshLoadError {}

/// Consistent corner forces in N; their moments integrate the varying load.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TriangleLoad {
    pub triangle: usize,
    pub vertices: [usize; 3],
    pub area_m2: f64,
    pub nodal_force: [[f64; 3]; 3],
    pub force: [f64; 3],
    /// Moment about the owner's declared reference, in N m.
    pub torque: [f64; 3],
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeshLoadReport {
    pub surface: SurfaceStamp,
    pub triangles: usize,
    pub moment_reference: [f64; 3],
    pub force: [f64; 3],
    pub torque: [f64; 3],
    /// V·F + omega·torque, in W, for a caller-specified virtual rigid twist.
    pub rigid_power: f64,
    /// Independently accumulated sum of consistent nodal force·nodal velocity.
    pub nodal_power: f64,
    /// Nearest-rounded discrepancy, not an error enclosure or stepping gate.
    pub power_defect: f64,
}

/// Immutable borrowed geometry and load data. Allocates no heap storage.
/// Each pass scans at most the admitted triangle limit with fixed-size scratch.
/// Open, duplicate, intersecting and disconnected facets retain the surface's
/// semantics; every supplied facet contributes once. This is a load operator,
/// not a fluid pressure map, solid validator or body integrator.
#[derive(Debug)]
pub struct TriangleMeshLoad<'a> {
    surface: &'a TriangleSurface,
    loading: SurfaceLoading<'a>,
    reference: [f64; 3],
}
impl<'a> TriangleMeshLoad<'a> {
    pub fn new(
        surface: &'a TriangleSurface,
        expected_stamp: SurfaceStamp,
        loading: SurfaceLoading<'a>,
        moment_reference: [f64; 3],
        triangle_limit: usize,
        mut cancelled: impl FnMut(MeshLoadStage, usize) -> bool,
    ) -> Result<Self, MeshLoadError> {
        if expected_stamp != surface.stamp() {
            return Err(MeshLoadError::StaleSurface);
        }
        if !finite(moment_reference) {
            return Err(MeshLoadError::InvalidReference);
        }
        let count = match loading {
            SurfaceLoading::Traction(values) => values.len(),
            SurfaceLoading::Pressure(values) => values.len(),
        };
        if count != surface.triangles().len() {
            return Err(MeshLoadError::LoadCount);
        }
        if count > triangle_limit {
            return Err(MeshLoadError::TriangleLimit {
                required: count,
                limit: triangle_limit,
            });
        }
        let owner = Self {
            surface,
            loading,
            reference: moment_reference,
        };
        for triangle in 0..count {
            if cancelled(MeshLoadStage::Admission, triangle) {
                return Err(MeshLoadError::Cancelled {
                    stage: MeshLoadStage::Admission,
                    triangle,
                });
            }
            owner.triangle_load(triangle)?;
        }
        Ok(owner)
    }
    pub fn surface(&self) -> &'a TriangleSurface {
        self.surface
    }
    pub fn moment_reference(&self) -> [f64; 3] {
        self.reference
    }

    /// One facet's actual consistent loads; no centroid torque approximation.
    pub fn triangle_load(&self, triangle: usize) -> Result<TriangleLoad, MeshLoadError> {
        let indices = *self
            .surface
            .triangles()
            .get(triangle)
            .ok_or(MeshLoadError::InvalidTriangle)?;
        let points = indices.map(|i| self.surface.vertices()[i]);
        let arithmetic = || MeshLoadError::ArithmeticFailure { triangle };
        let result = (|| -> Option<TriangleLoad> {
            let e1 = subtract(points[1], points[0])?;
            let e2 = subtract(points[2], points[0])?;
            let scale = e1.into_iter().chain(e2).map(f64::abs).fold(0.0, f64::max);
            if scale <= 0.0 || !scale.is_finite() {
                return None;
            }
            let a = e1.map(|x| x / scale);
            let b = e2.map(|x| x / scale);
            let normal = cross(a, b)?;
            let norm = normal[0].hypot(normal[1]).hypot(normal[2]);
            if norm <= 0.0 || !norm.is_finite() {
                return None;
            }
            let area = mul(mul(0.5 * norm, scale)?, scale)?;
            let weight = area / 12.0;
            if weight <= 0.0 || !weight.is_finite() {
                return None;
            }
            let traction = match self.loading {
                SurfaceLoading::Traction(values) => values[triangle],
                SurfaceLoading::Pressure(values) => {
                    let mut traction = [[0.0; 3]; 3];
                    for (i, p) in values[triangle].into_iter().enumerate() {
                        for (d, n) in normal.into_iter().enumerate() {
                            traction[i][d] = mul(-p, n / norm)?;
                        }
                    }
                    traction
                }
            };
            let mut sum = [0.0; 3];
            for t in traction {
                sum = add(sum, t)?;
            }
            let mut nodal_force = [[0.0; 3]; 3];
            let mut force = [0.0; 3];
            let mut torque = [0.0; 3];
            for i in 0..3 {
                for (d, &s) in sum.iter().enumerate() {
                    nodal_force[i][d] = mul(weight, scalar_add(s, traction[i][d])?)?;
                }
                force = add(force, nodal_force[i])?;
                let arm = subtract(points[i], self.reference)?;
                torque = add(torque, cross(arm, nodal_force[i])?)?;
            }
            Some(TriangleLoad {
                triangle,
                vertices: indices,
                area_m2: area,
                nodal_force,
                force,
                torque,
            })
        })();
        // Separate input errors from numerical refusal, without storing partial data.
        let valid = match self.loading {
            SurfaceLoading::Traction(v) => v[triangle].into_iter().all(finite),
            SurfaceLoading::Pressure(v) => v[triangle].into_iter().all(f64::is_finite),
        };
        if !valid {
            return Err(MeshLoadError::NonFiniteLoad { triangle });
        }
        result.ok_or_else(arithmetic)
    }

    /// Reduce physical forces/torques and evaluate a virtual rigid velocity field.
    /// Failure/cancellation returns no report and mutates no geometry or loads.
    pub fn reduce(
        &self,
        translation_velocity: [f64; 3],
        angular_velocity: [f64; 3],
        mut cancelled: impl FnMut(MeshLoadStage, usize) -> bool,
    ) -> Result<MeshLoadReport, MeshLoadError> {
        if !finite(translation_velocity) || !finite(angular_velocity) {
            return Err(MeshLoadError::InvalidTwist);
        }
        let mut force = [0.0; 3];
        let mut torque = [0.0; 3];
        let mut nodal_power = 0.0;
        for triangle in 0..self.surface.triangles().len() {
            if cancelled(MeshLoadStage::Reduction, triangle) {
                return Err(MeshLoadError::Cancelled {
                    stage: MeshLoadStage::Reduction,
                    triangle,
                });
            }
            let load = self.triangle_load(triangle)?;
            let error = || MeshLoadError::ArithmeticFailure { triangle };
            force = add(force, load.force).ok_or_else(error)?;
            torque = add(torque, load.torque).ok_or_else(error)?;
            for i in 0..3 {
                let p = self.surface.vertices()[load.vertices[i]];
                let power = (|| {
                    let arm = subtract(p, self.reference)?;
                    let velocity = add(translation_velocity, cross(angular_velocity, arm)?)?;
                    dot(load.nodal_force[i], velocity)
                })()
                .ok_or_else(error)?;
                nodal_power = scalar_add(nodal_power, power).ok_or_else(error)?;
            }
        }
        let error = || MeshLoadError::ArithmeticFailure {
            triangle: self.surface.triangles().len() - 1,
        };
        let rigid_power = (|| {
            scalar_add(
                dot(translation_velocity, force)?,
                dot(angular_velocity, torque)?,
            )
        })()
        .ok_or_else(error)?;
        let power_defect = scalar_add(nodal_power, -rigid_power).ok_or_else(error)?;
        Ok(MeshLoadReport {
            surface: self.surface.stamp(),
            triangles: self.surface.triangles().len(),
            moment_reference: self.reference,
            force,
            torque,
            rigid_power,
            nodal_power,
            power_defect,
        })
    }
}
fn finite(v: [f64; 3]) -> bool {
    v.into_iter().all(f64::is_finite)
}
fn scalar_add(a: f64, b: f64) -> Option<f64> {
    let value = a + b;
    value.is_finite().then_some(value)
}
fn mul(a: f64, b: f64) -> Option<f64> {
    let value = a * b;
    (value.is_finite() && (value != 0.0 || a == 0.0 || b == 0.0)).then_some(value)
}
fn add(a: [f64; 3], b: [f64; 3]) -> Option<[f64; 3]> {
    Some([
        scalar_add(a[0], b[0])?,
        scalar_add(a[1], b[1])?,
        scalar_add(a[2], b[2])?,
    ])
}
fn subtract(a: [f64; 3], b: [f64; 3]) -> Option<[f64; 3]> {
    add(a, b.map(|x| -x))
}
fn dot(a: [f64; 3], b: [f64; 3]) -> Option<f64> {
    scalar_add(
        scalar_add(mul(a[0], b[0])?, mul(a[1], b[1])?)?,
        mul(a[2], b[2])?,
    )
}
fn cross(a: [f64; 3], b: [f64; 3]) -> Option<[f64; 3]> {
    Some([
        scalar_add(mul(a[1], b[2])?, -mul(a[2], b[1])?)?,
        scalar_add(mul(a[2], b[0])?, -mul(a[0], b[2])?)?,
        scalar_add(mul(a[0], b[1])?, -mul(a[1], b[0])?)?,
    ])
}
