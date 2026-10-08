//! Opt-in spherical-inertia mesh kick and quaternion exponential drift.
use crate::rigid_impulse::{add, div, mul};
use crate::{
    FrozenRigidBody, RigidImpulseError, RigidImpulseReport, RigidImpulseStage, RigidSnapshot,
    RigidStamp, SurfaceError, SurfaceLoading, SurfaceSettings, TriangleMeshLoad, TriangleSurface,
};
use std::{fmt, mem::size_of};

/// Chosen sampling/accuracy limit, not a global or IEEE stability certificate.
pub const MAX_RIGID_ROTATION_RAD: f64 = 0.25;
#[derive(Debug, Clone, Copy)]
pub struct RigidMotionSettings {
    pub triangle_limit: usize,
    /// Reference vertex, current surface and proposed surface Vec capacities.
    /// Excludes input creation, allocator metadata, fixed owner headers and RSS.
    pub peak_payload_limit_bytes: usize,
    /// Caller-selected sampling limit in seconds, not an accuracy guarantee.
    pub max_dt_s: f64,
}
impl Default for RigidMotionSettings {
    fn default() -> Self {
        Self {
            triangle_limit: 64,
            peak_payload_limit_bytes: 16 * 1024 * 1024,
            max_dt_s: 1.,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RigidPoseSnapshot {
    pub body: RigidSnapshot,
    pub time_s: f64,
    /// Unit quaternion [real,x,y,z], mapping initial body axes to WORLD axes.
    pub orientation: [f64; 4],
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigidMotionStage {
    Admission,
    LoadAdmission,
    LoadReduction,
    Kick,
    Drift,
    Geometry,
    Publication,
}
#[derive(Debug, Clone, PartialEq)]
pub enum RigidMotionError {
    InvalidSettings,
    UnsupportedInertia,
    StaleBody,
    StaleSurface,
    InvalidDuration,
    SurfaceVersionOverflow,
    ArithmeticFailure,
    ClockAbsorbed,
    RotationLimit,
    UnrepresentableReference {
        vertex: usize,
    },
    AllocationFailure,
    PayloadLimit {
        required: usize,
        limit: usize,
    },
    TriangleLimit {
        required: usize,
        limit: usize,
    },
    Cancelled {
        stage: RigidMotionStage,
        index: usize,
    },
    Impulse(RigidImpulseError),
    Surface(SurfaceError),
}
impl fmt::Display for RigidMotionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rigid pose update rejected: {self:?}")
    }
}
impl std::error::Error for RigidMotionError {}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RigidMotionReport {
    pub before: RigidPoseSnapshot,
    pub after: RigidPoseSnapshot,
    /// The PR34 response at the OLD pose; after.body carries the NEW COM/stamp.
    pub impulse: RigidImpulseReport,
    pub requested_dt_s: f64,
    pub represented_elapsed_s: f64,
    pub clock_defect_s: f64,
    pub rotation_increment_rad: f64,
    /// Nearest-rounded diagnostics, unenclosed and without stepping authority.
    pub quaternion_norm_defect: f64,
    pub translation_defect_m: [f64; 3],
    pub peak_payload_bytes: usize,
}

/// Owns actual current world geometry, pose/time and PR34 body velocities.
/// Spherical principal moments are required: arbitrary anisotropic motion is
/// unsupported. No contact enforcement, swept queries or fluid reaction.
#[derive(Debug)]
pub struct SphericalRigidMotion {
    body: FrozenRigidBody,
    world: TriangleSurface,
    reference_vertices: Vec<[f64; 3]>,
    orientation: [f64; 4],
    time_s: f64,
    settings: RigidMotionSettings,
    peak_payload_bytes: usize,
}
impl SphericalRigidMotion {
    /// Initial geometry is world geometry at identity orientation. Body-relative
    /// translation must round-trip exactly; no approximate inverse rotation.
    pub fn new(
        body: FrozenRigidBody,
        world: TriangleSurface,
        time_s: f64,
        settings: RigidMotionSettings,
    ) -> Result<Self, RigidMotionError> {
        use RigidMotionError as E;
        let state = body.snapshot();
        if !time_s.is_finite() || !settings.max_dt_s.is_finite() || settings.max_dt_s <= 0. {
            return Err(E::InvalidSettings);
        }
        if state.surface != world.stamp() {
            return Err(E::StaleSurface);
        }
        if state.inertia_kg_m2[0] != state.inertia_kg_m2[1]
            || state.inertia_kg_m2[0] != state.inertia_kg_m2[2]
        {
            return Err(E::UnsupportedInertia);
        }
        if world.triangles().len() > settings.triangle_limit {
            return Err(E::TriangleLimit {
                required: world.triangles().len(),
                limit: settings.triangle_limit,
            });
        }
        let reference_min = world
            .vertices()
            .len()
            .checked_mul(size_of::<[f64; 3]>())
            .ok_or(E::ArithmeticFailure)?;
        let min_peak = world
            .allocated_bytes()
            .checked_mul(2)
            .and_then(|x| x.checked_add(reference_min))
            .ok_or(E::ArithmeticFailure)?;
        if min_peak > settings.peak_payload_limit_bytes {
            return Err(E::PayloadLimit {
                required: min_peak,
                limit: settings.peak_payload_limit_bytes,
            });
        }
        let mut reference_vertices = Vec::new();
        reference_vertices
            .try_reserve_exact(world.vertices().len())
            .map_err(|_| E::AllocationFailure)?;
        for (vertex, p) in world.vertices().iter().enumerate() {
            let mut r = [0.; 3];
            for i in 0..3 {
                r[i] = add(p[i], -state.center_of_mass[i])
                    .ok_or(E::UnrepresentableReference { vertex })?;
                if add(state.center_of_mass[i], r[i]) != Some(p[i]) {
                    return Err(E::UnrepresentableReference { vertex });
                }
            }
            reference_vertices.push(r);
        }
        let peak_payload_bytes = payload(
            reference_vertices.capacity(),
            world.allocated_bytes(),
            world.allocated_bytes(),
        )
        .ok_or(E::ArithmeticFailure)?;
        if peak_payload_bytes > settings.peak_payload_limit_bytes {
            return Err(E::PayloadLimit {
                required: peak_payload_bytes,
                limit: settings.peak_payload_limit_bytes,
            });
        }
        Ok(Self {
            body,
            world,
            reference_vertices,
            orientation: [1., 0., 0., 0.],
            time_s,
            settings,
            peak_payload_bytes,
        })
    }
    pub fn snapshot(&self) -> RigidPoseSnapshot {
        RigidPoseSnapshot {
            body: self.body.snapshot(),
            time_s: self.time_s,
            orientation: self.orientation,
        }
    }
    pub fn world_surface(&self) -> &TriangleSurface {
        &self.world
    }
    pub fn peak_payload_bytes(&self) -> usize {
        self.peak_payload_bytes
    }

    /// Sample supplied world loads at the current pose, kick via PR34, then coast
    /// for h at post-kick twist. q'=exp(h*[0,omega']/2)*q (LEFT multiplication).
    /// First order for smooth sampled force histories. For an imposed kick then
    /// spherical free coast, drift is exact-real. No adaptive/automatic timestep.
    pub fn advance(
        &mut self,
        expected: RigidStamp,
        expected_surface: crate::SurfaceStamp,
        loading: SurfaceLoading<'_>,
        h: f64,
        mut cancelled: impl FnMut(RigidMotionStage, usize) -> bool,
    ) -> Result<RigidMotionReport, RigidMotionError> {
        use RigidMotionError as E;
        let before = self.snapshot();
        if expected != before.body.stamp {
            return Err(E::StaleBody);
        }
        if expected_surface != before.body.surface {
            return Err(E::StaleSurface);
        }
        if !h.is_finite() || h <= 0. || h > self.settings.max_dt_s {
            return Err(E::InvalidDuration);
        }
        let version = expected_surface
            .version
            .checked_add(1)
            .ok_or(E::SurfaceVersionOverflow)?;
        let time = add(before.time_s, h).ok_or(E::ArithmeticFailure)?;
        if time <= before.time_s {
            return Err(E::ClockAbsorbed);
        }
        let mut check = |stage, index| {
            if cancelled(stage, index) {
                Err(E::Cancelled { stage, index })
            } else {
                Ok(())
            }
        };
        check(RigidMotionStage::Admission, 0)?;
        let load = TriangleMeshLoad::new(
            &self.world,
            expected_surface,
            loading,
            before.body.center_of_mass,
            self.settings.triangle_limit,
            |_, i| check(RigidMotionStage::LoadAdmission, i).is_err(),
        )
        .map_err(|e| match e {
            crate::MeshLoadError::Cancelled { triangle, .. } => E::Cancelled {
                stage: RigidMotionStage::LoadAdmission,
                index: triangle,
            },
            e => E::Impulse(RigidImpulseError::Mesh(e)),
        })?;
        let mut proposal = FrozenRigidBody::new(before.body).map_err(E::Impulse)?;
        let impulse = proposal
            .apply_mesh_impulse(expected, &load, h, |stage, i| {
                let mapped = match stage {
                    RigidImpulseStage::Reduction => RigidMotionStage::LoadReduction,
                    _ => RigidMotionStage::Kick,
                };
                check(mapped, i).is_err()
            })
            .map_err(|e| match e {
                RigidImpulseError::Cancelled { stage, triangle } => E::Cancelled {
                    stage: if stage == RigidImpulseStage::Reduction {
                        RigidMotionStage::LoadReduction
                    } else {
                        RigidMotionStage::Kick
                    },
                    index: triangle,
                },
                e => E::Impulse(e),
            })?;
        check(RigidMotionStage::Drift, 0)?;
        let arithmetic = || E::ArithmeticFailure;
        let mut state = proposal.snapshot();
        let mut translation_defect_m = [0.; 3];
        for (i, defect) in translation_defect_m.iter_mut().enumerate() {
            let displacement = mul(h, state.velocity_m_s[i]).ok_or_else(arithmetic)?;
            state.center_of_mass[i] =
                add(before.body.center_of_mass[i], displacement).ok_or_else(arithmetic)?;
            *defect = add(
                add(state.center_of_mass[i], -before.body.center_of_mass[i])
                    .ok_or_else(arithmetic)?,
                -displacement,
            )
            .ok_or_else(arithmetic)?;
        }
        let omega = state.angular_velocity_rad_s;
        let norm = omega[0].hypot(omega[1]).hypot(omega[2]);
        let theta = mul(h, norm).ok_or_else(arithmetic)?;
        if theta > MAX_RIGID_ROTATION_RAD {
            return Err(E::RotationLimit);
        }
        let dq = if norm == 0. {
            [1., 0., 0., 0.]
        } else {
            let (s, c) = mul(0.5, theta).ok_or_else(arithmetic)?.sin_cos();
            let mut q = [c, 0., 0., 0.];
            for i in 0..3 {
                q[i + 1] =
                    mul(s, div(omega[i], norm).ok_or_else(arithmetic)?).ok_or_else(arithmetic)?;
            }
            q
        };
        let qraw = qmul(dq, before.orientation).ok_or_else(arithmetic)?;
        let qnorm = qraw[0].hypot(qraw[1]).hypot(qraw[2]).hypot(qraw[3]);
        if !qnorm.is_finite() || qnorm <= 0. {
            return Err(E::ArithmeticFailure);
        }
        let mut orientation = [0.; 4];
        for i in 0..4 {
            orientation[i] = div(qraw[i], qnorm).ok_or_else(arithmetic)?;
        }
        let quaternion_norm_defect = add(
            orientation[0]
                .hypot(orientation[1])
                .hypot(orientation[2])
                .hypot(orientation[3]),
            -1.,
        )
        .ok_or_else(arithmetic)?;
        state.surface.version = version;
        let mut vertices = Vec::new();
        vertices
            .try_reserve_exact(self.reference_vertices.len())
            .map_err(|_| E::AllocationFailure)?;
        let mut triangles = Vec::new();
        triangles
            .try_reserve_exact(self.world.triangles().len())
            .map_err(|_| E::AllocationFailure)?;
        let proposed_bytes = vertices
            .capacity()
            .checked_mul(size_of::<[f64; 3]>())
            .and_then(|n| {
                triangles
                    .capacity()
                    .checked_mul(size_of::<[usize; 3]>())
                    .and_then(|t| n.checked_add(t))
            })
            .ok_or_else(arithmetic)?;
        let peak = payload(
            self.reference_vertices.capacity(),
            self.world.allocated_bytes(),
            proposed_bytes,
        )
        .ok_or_else(arithmetic)?;
        if peak > self.settings.peak_payload_limit_bytes {
            return Err(E::PayloadLimit {
                required: peak,
                limit: self.settings.peak_payload_limit_bytes,
            });
        }
        for (i, &r) in self.reference_vertices.iter().enumerate() {
            check(RigidMotionStage::Geometry, i)?;
            let rotated = rotate(orientation, r).ok_or_else(arithmetic)?;
            let mut p = [0.; 3];
            for j in 0..3 {
                p[j] = add(state.center_of_mass[j], rotated[j]).ok_or_else(arithmetic)?;
            }
            vertices.push(p);
        }
        triangles.extend_from_slice(self.world.triangles());
        let world = TriangleSurface::new(
            state.surface,
            vertices,
            triangles,
            SurfaceSettings {
                relative_tolerance: self.world.relative_tolerance(),
                memory_limit: self.settings.peak_payload_limit_bytes,
            },
        )
        .map_err(E::Surface)?;
        let body = FrozenRigidBody::new(state).map_err(E::Impulse)?;
        let after = RigidPoseSnapshot {
            body: body.snapshot(),
            time_s: time,
            orientation,
        };
        let elapsed = add(time, -before.time_s).ok_or_else(arithmetic)?;
        let report = RigidMotionReport {
            before,
            after,
            impulse,
            requested_dt_s: h,
            represented_elapsed_s: elapsed,
            clock_defect_s: add(elapsed, -h).ok_or_else(arithmetic)?,
            rotation_increment_rad: theta,
            quaternion_norm_defect,
            translation_defect_m,
            peak_payload_bytes: peak,
        };
        check(RigidMotionStage::Publication, 0)?;
        self.body = body;
        self.world = world;
        self.orientation = orientation;
        self.time_s = time;
        self.peak_payload_bytes = self.peak_payload_bytes.max(peak);
        Ok(report)
    }
}
fn payload(reference_capacity: usize, current: usize, proposed: usize) -> Option<usize> {
    reference_capacity
        .checked_mul(size_of::<[f64; 3]>())?
        .checked_add(current)?
        .checked_add(proposed)
}
fn qmul(a: [f64; 4], b: [f64; 4]) -> Option<[f64; 4]> {
    // Hamilton product, each intermediate checked; no fused/fast arithmetic.
    let mut q = [0.; 4];
    let terms = [
        [(0, 0, 1.), (1, 1, -1.), (2, 2, -1.), (3, 3, -1.)],
        [(0, 1, 1.), (1, 0, 1.), (2, 3, 1.), (3, 2, -1.)],
        [(0, 2, 1.), (2, 0, 1.), (3, 1, 1.), (1, 3, -1.)],
        [(0, 3, 1.), (3, 0, 1.), (1, 2, 1.), (2, 1, -1.)],
    ];
    for i in 0..4 {
        for &(j, k, sign) in &terms[i] {
            q[i] = add(q[i], sign * mul(a[j], b[k])?)?;
        }
    }
    Some(q)
}
fn rotate(q: [f64; 4], p: [f64; 3]) -> Option<[f64; 3]> {
    let qp = qmul(q, [0., p[0], p[1], p[2]])?;
    let result = qmul(qp, [q[0], -q[1], -q[2], -q[3]])?;
    Some([result[1], result[2], result[3]])
}
