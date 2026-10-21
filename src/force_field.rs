//! Solver-independent, static external fields in SI coordinates.
//! Authoring data is mutable; only a validated, owned snapshot can be evaluated.
use crate::ForceUnits;
use std::{fmt, sync::Arc};

pub const FORCE_SCENE_VERSION: u32 = 1;
pub const MAX_FORCE_OBJECTS: usize = 256;

/// Caller-owned identity, unique across models and forces in an authoring scene.
/// Zero is reserved. The force document checks its own subset for duplicates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    any(feature = "force-serde", feature = "desktop"),
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(any(feature = "force-serde", feature = "desktop"), serde(transparent))]
pub struct SceneObjectId(pub u64);

/// Translation in metres; unit quaternion [x,y,z,w], local to world. No scale.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(
    any(feature = "force-serde", feature = "desktop"),
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(
    any(feature = "force-serde", feature = "desktop"),
    serde(deny_unknown_fields)
)]
pub struct ForceTransform {
    pub translation: [f64; 3],
    pub rotation: [f64; 4],
}
impl Default for ForceTransform {
    fn default() -> Self {
        Self {
            translation: [0.0; 3],
            rotation: [0.0, 0.0, 0.0, 1.0],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(
    any(feature = "force-serde", feature = "desktop"),
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(
    any(feature = "force-serde", feature = "desktop"),
    serde(deny_unknown_fields)
)]
pub enum ForceKind {
    /// Local vector in the selected units (m/s² or N/m³).
    Directional { vector: [f64; 3] },
    /// Constant magnitude, positive outward; negative attracts. Zero at centre.
    Radial { strength: f64 },
    /// Constant tangential magnitude around local +Y, right-hand rule.
    /// Positive strength points toward -Z at +X. Zero on the axis.
    Vortex { strength: f64 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(
    any(feature = "force-serde", feature = "desktop"),
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(
    any(feature = "force-serde", feature = "desktop"),
    serde(deny_unknown_fields)
)]
pub enum ForceSupport {
    Global,
    /// Local sphere, open outer boundary. Edge fraction is in [0,1].
    Sphere {
        radius: f64,
        edge_fraction: f64,
    },
    /// Local box [-half_extent,+half_extent), with an optional smooth edge.
    Box {
        half_extent: [f64; 3],
        edge_fraction: f64,
    },
}

/// Static activation sampled at accepted interval start, in seconds.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(
    any(feature = "force-serde", feature = "desktop"),
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(
    any(feature = "force-serde", feature = "desktop"),
    serde(deny_unknown_fields)
)]
pub struct ForceTime {
    pub start_seconds: f64,
    /// Exclusive end; None has no end. Steps are not split at these endpoints.
    pub end_seconds: Option<f64>,
}
impl Default for ForceTime {
    fn default() -> Self {
        Self {
            start_seconds: 0.0,
            end_seconds: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    any(feature = "force-serde", feature = "desktop"),
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(
    any(feature = "force-serde", feature = "desktop"),
    serde(deny_unknown_fields)
)]
pub struct ForceObject {
    pub id: SceneObjectId,
    pub name: String,
    pub enabled: bool,
    pub transform: ForceTransform,
    pub units: ForceUnits,
    pub kind: ForceKind,
    pub support: ForceSupport,
    pub time: ForceTime,
}

/// Serialization carries explicit schema version; unknown fields/variants fail.
/// Deserialized authoring data still requires freeze() before solver use.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    any(feature = "force-serde", feature = "desktop"),
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(
    any(feature = "force-serde", feature = "desktop"),
    serde(deny_unknown_fields)
)]
pub struct ForceScene {
    pub version: u32,
    pub objects: Vec<ForceObject>,
}
impl Default for ForceScene {
    fn default() -> Self {
        Self {
            version: FORCE_SCENE_VERSION,
            objects: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForceFieldError {
    UnsupportedVersion,
    TooManyObjects,
    InvalidIdentity,
    InvalidObject,
    InvalidSample,
    ArithmeticFailure,
}
impl fmt::Display for ForceFieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "external force field: {self:?}")
    }
}
impl std::error::Error for ForceFieldError {}

/// Independent owned run inputs. No mutable access or deserialization bypass.
/// The host retains this with run metadata, separate from later draft edits.
#[derive(Debug, Clone)]
pub struct ForceSnapshot {
    scene: Arc<ForceScene>,
}
impl ForceScene {
    pub fn freeze(&self) -> Result<ForceSnapshot, ForceFieldError> {
        if self.version != FORCE_SCENE_VERSION {
            return Err(ForceFieldError::UnsupportedVersion);
        }
        if self.objects.len() > MAX_FORCE_OBJECTS {
            return Err(ForceFieldError::TooManyObjects);
        }
        for (i, object) in self.objects.iter().enumerate() {
            if object.id.0 == 0 || self.objects[..i].iter().any(|o| o.id == object.id) {
                return Err(ForceFieldError::InvalidIdentity);
            }
            object.validate()?;
        }
        Ok(ForceSnapshot {
            scene: Arc::new(self.clone()),
        })
    }
}
impl ForceSnapshot {
    pub fn scene(&self) -> &ForceScene {
        &self.scene
    }

    /// Acceleration at a world position. Force density is divided by the caller's
    /// positive local mass density, never by particle mass. No inverse-square law.
    pub fn acceleration(
        &self,
        position: [f64; 3],
        time: f64,
        density: f64,
    ) -> Result<[f64; 3], ForceFieldError> {
        if position.iter().any(|v| !v.is_finite())
            || !time.is_finite()
            || time < 0.0
            || !density.is_finite()
            || density <= 0.0
        {
            return Err(ForceFieldError::InvalidSample);
        }
        let mut sum = [0.0; 3];
        for object in self.scene.objects.iter().filter(|o| o.active(time)) {
            let value = object.evaluate(position)?;
            for d in 0..3 {
                sum[d] += match object.units {
                    ForceUnits::Acceleration => value[d],
                    ForceUnits::ForceDensity => value[d] / density,
                };
            }
        }
        finite(sum)
    }

    /// Conservative world-component bound for MAC displacement budgeting.
    /// Triangle inequality deliberately ignores support and cancellation.
    pub(crate) fn acceleration_bound(
        &self,
        time: f64,
        density: f64,
    ) -> Result<f64, ForceFieldError> {
        let mut bound = 0.0;
        for object in self.scene.objects.iter().filter(|o| o.active(time)) {
            let magnitude = match object.kind {
                ForceKind::Directional { vector } => length(vector),
                ForceKind::Radial { strength } | ForceKind::Vortex { strength } => strength.abs(),
            };
            // Unit quaternions are checked to finite tolerance; factor covers
            // the corresponding rotation matrix norm error conservatively.
            // A tiny absolute allowance also covers underflow/rounding during
            // normalization, rotation and unit conversion of subnormal values.
            let magnitude = magnitude * (1.0 + 1e-10) + f64::MIN_POSITIVE;
            bound += match object.units {
                ForceUnits::Acceleration => magnitude,
                ForceUnits::ForceDensity => magnitude / density + f64::MIN_POSITIVE,
            };
        }
        if bound.is_finite() {
            Ok(bound)
        } else {
            Err(ForceFieldError::ArithmeticFailure)
        }
    }
}
fn length(v: [f64; 3]) -> f64 {
    v[0].hypot(v[1]).hypot(v[2])
}
// Scale first: direct hypot can round a subnormal multi-axis norm enough to
// exceed unit magnitude, or overflow for otherwise finite world coordinates.
fn direction(v: [f64; 3]) -> [f64; 3] {
    let scale = v.into_iter().map(f64::abs).fold(0.0_f64, f64::max);
    if scale == 0.0 {
        return [0.0; 3];
    }
    let scaled = v.map(|x| x / scale);
    let norm = length(scaled);
    scaled.map(|x| x / norm)
}
fn finite(v: [f64; 3]) -> Result<[f64; 3], ForceFieldError> {
    if v.iter().all(|x| x.is_finite()) {
        Ok(v)
    } else {
        Err(ForceFieldError::ArithmeticFailure)
    }
}
fn rotate(q: [f64; 4], v: [f64; 3]) -> [f64; 3] {
    let [x, y, z, w] = q;
    [
        (1.0 - 2.0 * (y * y + z * z)) * v[0]
            + 2.0 * (x * y - z * w) * v[1]
            + 2.0 * (x * z + y * w) * v[2],
        2.0 * (x * y + z * w) * v[0]
            + (1.0 - 2.0 * (x * x + z * z)) * v[1]
            + 2.0 * (y * z - x * w) * v[2],
        2.0 * (x * z - y * w) * v[0]
            + 2.0 * (y * z + x * w) * v[1]
            + (1.0 - 2.0 * (x * x + y * y)) * v[2],
    ]
}
impl ForceObject {
    fn validate(&self) -> Result<(), ForceFieldError> {
        let q = self.transform.rotation;
        let q2: f64 = q.iter().map(|x| x * x).sum();
        let edge_valid = |e: f64| e.is_finite() && (0.0..=1.0).contains(&e);
        let support_valid = match self.support {
            ForceSupport::Global => true,
            ForceSupport::Sphere {
                radius,
                edge_fraction,
            } => radius.is_finite() && radius > 0.0 && edge_valid(edge_fraction),
            ForceSupport::Box {
                half_extent,
                edge_fraction,
            } => half_extent.iter().all(|v| v.is_finite() && *v > 0.0) && edge_valid(edge_fraction),
        };
        let magnitude = match self.kind {
            ForceKind::Directional { vector } => length(vector),
            ForceKind::Radial { strength } | ForceKind::Vortex { strength } => strength,
        };
        if self.name.len() > 256
            || self.transform.translation.iter().any(|x| !x.is_finite())
            || !q2.is_finite()
            || (q2 - 1.0).abs() > 1e-12
            || !magnitude.is_finite()
            || !support_valid
            || !self.time.start_seconds.is_finite()
            || self.time.start_seconds < 0.0
            || self
                .time
                .end_seconds
                .is_some_and(|end| !end.is_finite() || end <= self.time.start_seconds)
        {
            return Err(ForceFieldError::InvalidObject);
        }
        Ok(())
    }
    fn active(&self, time: f64) -> bool {
        self.enabled
            && time >= self.time.start_seconds
            && self.time.end_seconds.is_none_or(|end| time < end)
    }
    fn evaluate(&self, position: [f64; 3]) -> Result<[f64; 3], ForceFieldError> {
        let delta = finite(std::array::from_fn(|d| {
            position[d] - self.transform.translation[d]
        }))?;
        let [x, y, z, w] = self.transform.rotation;
        let local = finite(rotate([-x, -y, -z, w], delta))?;
        let (r, edge) = match self.support {
            ForceSupport::Global => (0.0, 0.0),
            ForceSupport::Sphere {
                radius,
                edge_fraction,
            } => {
                let r = length(local.map(|v| v / radius));
                if r >= 1.0 {
                    return Ok([0.0; 3]);
                }
                (r, edge_fraction)
            }
            ForceSupport::Box {
                half_extent,
                edge_fraction,
            } => {
                if (0..3).any(|d| local[d] < -half_extent[d] || local[d] >= half_extent[d]) {
                    return Ok([0.0; 3]);
                }
                (
                    (0..3)
                        .map(|d| local[d].abs() / half_extent[d])
                        .fold(0.0_f64, f64::max),
                    edge_fraction,
                )
            }
        };
        let weight = if edge == 0.0 {
            1.0
        } else {
            let t = ((1.0 - r) / edge).clamp(0.0, 1.0);
            t * t * (3.0 - 2.0 * t)
        };
        let local_value = match self.kind {
            ForceKind::Directional { vector } => vector,
            ForceKind::Radial { strength } => direction(local).map(|v| v * strength),
            ForceKind::Vortex { strength } => {
                direction([local[2], 0.0, -local[0]]).map(|v| v * strength)
            }
        };
        finite(rotate(
            self.transform.rotation,
            local_value.map(|v| v * weight),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rotated_subnormal_fields_respect_acceleration_bound() {
        let h = 0.5_f64.sqrt();
        for (kind, units) in [
            (
                ForceKind::Radial { strength: 2.0 },
                ForceUnits::Acceleration,
            ),
            (
                ForceKind::Vortex { strength: -2.0 },
                ForceUnits::Acceleration,
            ),
            (
                ForceKind::Directional {
                    vector: [1.0, -2.0, 3.0],
                },
                ForceUnits::ForceDensity,
            ),
            (
                ForceKind::Radial {
                    strength: f64::from_bits(1),
                },
                ForceUnits::Acceleration,
            ),
        ] {
            let fields = ForceScene {
                version: FORCE_SCENE_VERSION,
                objects: vec![ForceObject {
                    id: SceneObjectId(1),
                    name: "subnormal".into(),
                    enabled: true,
                    transform: ForceTransform {
                        translation: [0.0; 3],
                        rotation: [0.0, 0.0, h, h],
                    },
                    units,
                    kind,
                    support: ForceSupport::Global,
                    time: ForceTime::default(),
                }],
            }
            .freeze()
            .unwrap();
            let tiny = f64::from_bits(1);
            let actual = fields.acceleration([tiny, tiny, tiny], 0.0, 1.0).unwrap();
            let bound = fields.acceleration_bound(0.0, 1.0).unwrap();
            assert!(length(actual) <= bound);
            if matches!(
                kind,
                ForceKind::Radial { strength: 2.0 } | ForceKind::Vortex { strength: -2.0 }
            ) {
                assert!((length(actual) - 2.0).abs() < 1e-12);
            }
        }
    }
}
