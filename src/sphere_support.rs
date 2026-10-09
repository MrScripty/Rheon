//! Strict one-face stationary support, separate from impact/settling APIs.
use crate::sphere_contact::{checked, cross, difference, norm, quotient, scale, sum};
use crate::sphere_departure::{ProductSign, difference_dot};
use crate::*;
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SphereSupportRequest {
    pub expected_body: RigidStamp,
    pub expected_moving: SurfaceStamp,
    pub expected_static: SurfaceStamp,
    pub radius_m: f64,
    pub contact_point_m: [f64; 3],
    pub gravity_m_s2: [f64; 3],
    /// Requested equivalent impulse duration; not an enclosed stability bound.
    pub equivalent_interval_s: f64,
    pub settings: SphereContactSettings,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SphereSupportStage {
    Admission,
    GeometryProbe,
    GeometryCertificate,
    LoadAdmission,
    LoadReduction,
    Balance,
    Coast(RigidMotionStage),
}
#[derive(Debug, Clone, PartialEq)]
pub enum SphereSupportError {
    InvalidRequest,
    SingleFacetOnly,
    StationaryTwistOnly,
    UnsupportedHeldOrientation,
    MissingProbe,
    InvalidRadiusWitness,
    InvalidPlaneWitness,
    FaceBoundary,
    OutsideFace,
    TangentialLoadUnsupported,
    UnsupportedTorque,
    LiftOff,
    RoundedWrenchResidual,
    Cancelled {
        stage: SphereSupportStage,
        index: usize,
    },
    Contact(SphereContactError),
    Surface(SurfaceError),
    Load(MeshLoadError),
    Motion(RigidMotionError),
}
impl From<SphereContactError> for SphereSupportError {
    fn from(e: SphereContactError) -> Self {
        Self::Contact(e)
    }
}
impl std::fmt::Display for SphereSupportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "stationary sphere support rejected: {self:?}")
    }
}
impl std::error::Error for SphereSupportError {}
/// Actual retained force proposal. All scalar/vector diagnostics are nearest
/// rounded and unenclosed. Public fields cannot authorize held publication.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SphereSupportForces {
    pub request: SphereSupportRequest,
    pub from: RigidPoseSnapshot,
    pub probe: SurfaceHit,
    pub probe_point_defect_m: [f64; 3],
    pub external_mesh: MeshLoadReport,
    pub gravity_force_n: [f64; 3],
    pub total_external_force_n: [f64; 3],
    pub support_force_n: [f64; 3],
    pub contact_lever_m: [f64; 3],
    pub support_torque_n_m: [f64; 3],
    pub net_force_n: [f64; 3],
    pub net_torque_n_m: [f64; 3],
    pub normal_estimate: [f64; 3],
    pub normal_norm_defect: f64,
    pub support_magnitude_n: f64,
    /// Every component impulse below uses requested equivalent_interval_s.
    pub mesh_impulse_n_s: [f64; 3],
    pub gravity_impulse_n_s: [f64; 3],
    pub support_impulse_n_s: [f64; 3],
    pub mesh_angular_impulse_n_m_s: [f64; 3],
    pub support_angular_impulse_n_m_s: [f64; 3],
    pub linear_impulse_ledger_defect_n_s: [f64; 3],
    pub angular_impulse_ledger_defect_n_m_s: [f64; 3],
    pub external_power_w: f64,
    pub support_power_w: f64,
    pub external_work_j: f64,
    pub support_work_j: f64,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SphereSupportHeldReport {
    pub forces: SphereSupportForces,
    /// Existing PR35 zero-load proxy advance; its load report is zero and
    /// does not replace forces.external_mesh or prove physical correspondence.
    pub zero_load_motion: RigidMotionReport,
}
fn cancelled(
    callback: &mut impl FnMut(SphereSupportStage, usize) -> bool,
    stage: SphereSupportStage,
    index: usize,
) -> Result<(), SphereSupportError> {
    if callback(stage, index) {
        Err(SphereSupportError::Cancelled { stage, index })
    } else {
        Ok(())
    }
}
fn load_error(e: MeshLoadError) -> SphereSupportError {
    match e {
        MeshLoadError::Cancelled { stage, triangle } => SphereSupportError::Cancelled {
            stage: match stage {
                MeshLoadStage::Admission => SphereSupportStage::LoadAdmission,
                MeshLoadStage::Reduction => SphereSupportStage::LoadReduction,
            },
            index: triangle,
        },
        e => SphereSupportError::Load(e),
    }
}
// Exact projected determinant (a-c)_j*(b-c)_k - (a-c)_k*(b-c)_j.
fn orientation(
    a: [f64; 3],
    b: [f64; 3],
    c: [f64; 3],
    j: usize,
    k: usize,
) -> Result<Ordering, SphereSupportError> {
    let mut s = ProductSign::default();
    for (u, v, negative) in [(j, k, false), (k, j, true)] {
        s.add(a[u], b[v], negative)?;
        s.add(a[u], c[v], !negative)?;
        s.add(c[u], b[v], !negative)?;
        s.add(c[u], c[v], negative)?;
    }
    Ok(s.sign())
}
fn witness(
    c: [f64; 3],
    p: [f64; 3],
    radius: f64,
    vertices: [[f64; 3]; 3],
) -> Result<[f64; 3], SphereSupportError> {
    let mut radial = difference_dot(c, p, c, p)?;
    radial.add(radius, radius, true)?;
    if radial.sign() != Ordering::Equal {
        return Err(SphereSupportError::InvalidRadiusWitness);
    }
    for v in vertices {
        if difference_dot(c, p, v, p)?.sign() != Ordering::Equal {
            return Err(SphereSupportError::InvalidPlaneWitness);
        }
    }
    let a = difference(c, p)?;
    let drop = (0..3)
        .max_by(|i, j| a[*i].abs().total_cmp(&a[*j].abs()))
        .unwrap();
    let j = (drop + 1) % 3;
    let k = (drop + 2) % 3;
    let winding = orientation(vertices[0], vertices[1], vertices[2], j, k)?;
    if winding == Ordering::Equal {
        return Err(SphereSupportError::InvalidPlaneWitness);
    }
    for i in 0..3 {
        let side = orientation(vertices[i], vertices[(i + 1) % 3], p, j, k)?;
        if side == Ordering::Equal {
            return Err(SphereSupportError::FaceBoundary);
        }
        if side != winding {
            return Err(SphereSupportError::OutsideFace);
        }
    }
    Ok(a)
}
impl SphericalRigidMotion {
    /// Immutable zero-twist, one-face equilibrium proposal. No impact query,
    /// gap correction, tangent friction or approximate velocity acceptance.
    pub fn stationary_single_face_support(
        &self,
        request: SphereSupportRequest,
        surface: &TriangleSurface,
        loading: SurfaceLoading<'_>,
        mut callback: impl FnMut(SphereSupportStage, usize) -> bool,
    ) -> Result<SphereSupportForces, SphereSupportError> {
        use SphereSupportError as E;
        if !request.equivalent_interval_s.is_finite()
            || request.equivalent_interval_s <= 0.
            || !request
                .contact_point_m
                .iter()
                .chain(request.gravity_m_s2.iter())
                .all(|x| x.is_finite())
        {
            return Err(E::InvalidRequest);
        }
        cancelled(&mut callback, SphereSupportStage::Admission, 0)?;
        StaticSphereSweep::new(
            self,
            request.expected_body,
            request.expected_moving,
            surface,
            request.expected_static,
            request.radius_m,
            request.settings,
        )?;
        if surface.triangles().len() != 1 {
            return Err(E::SingleFacetOnly);
        }
        let from = self.snapshot();
        if from.body.velocity_m_s != [0.; 3] || from.body.angular_velocity_rad_s != [0.; 3] {
            return Err(E::StationaryTwistOnly);
        }
        let c = from.body.center_of_mass;
        let p = request.contact_point_m;
        // Cancellation is mapped uniformly; the probe cannot authorize touch.
        let mut probe_cancelled = None;
        let probe_end = sum(p, difference(p, c)?)?;
        let probe = surface.first_hit(c, probe_end, |i| {
            let hit = callback(SphereSupportStage::GeometryProbe, i);
            if hit {
                probe_cancelled = Some(i);
            }
            hit
        });
        if let Some(index) = probe_cancelled {
            return Err(E::Cancelled {
                stage: SphereSupportStage::GeometryProbe,
                index,
            });
        }
        let probe = probe.map_err(E::Surface)?.ok_or(E::MissingProbe)?;
        cancelled(&mut callback, SphereSupportStage::GeometryCertificate, 0)?;
        let ids = surface.triangles()[0];
        let a = witness(c, p, request.radius_m, ids.map(|i| surface.vertices()[i]))?;
        let load = TriangleMeshLoad::new(
            self.world_surface(),
            request.expected_moving,
            loading,
            c,
            MAX_CONTACT_BODY_TRIANGLES,
            |stage, i| {
                callback(
                    match stage {
                        MeshLoadStage::Admission => SphereSupportStage::LoadAdmission,
                        MeshLoadStage::Reduction => SphereSupportStage::LoadReduction,
                    },
                    i,
                )
            },
        )
        .map_err(load_error)?;
        let external_mesh = load
            .reduce([0.; 3], [0.; 3], |stage, i| {
                callback(
                    match stage {
                        MeshLoadStage::Admission => SphereSupportStage::LoadAdmission,
                        MeshLoadStage::Reduction => SphereSupportStage::LoadReduction,
                    },
                    i,
                )
            })
            .map_err(load_error)?;
        cancelled(&mut callback, SphereSupportStage::Balance, 0)?;
        let gravity_force_n = scale(from.body.mass_kg, request.gravity_m_s2)?;
        let f = sum(external_mesh.force, gravity_force_n)?;
        if external_mesh.torque != [0.; 3] {
            return Err(E::UnsupportedTorque);
        }
        for i in 0..3 {
            let j = (i + 1) % 3;
            let k = (i + 2) % 3;
            let mut s = ProductSign::default();
            s.add(c[j], f[k], false)?;
            s.add(p[j], f[k], true)?;
            s.add(c[k], f[j], true)?;
            s.add(p[k], f[j], false)?;
            if s.sign() != Ordering::Equal {
                return Err(E::TangentialLoadUnsupported);
            }
        }
        if difference_dot(c, p, f, [0.; 3])?.sign() == Ordering::Greater {
            return Err(E::LiftOff);
        }
        let support_force_n = f.map(|x| -x);
        let contact_lever_m = difference(p, c)?;
        let support_torque_n_m = cross(contact_lever_m, support_force_n)?;
        let net_force_n = sum(f, support_force_n)?;
        let net_torque_n_m = sum(external_mesh.torque, support_torque_n_m)?;
        if net_force_n != [0.; 3] || net_torque_n_m != [0.; 3] {
            return Err(E::RoundedWrenchResidual);
        }
        let h = request.equivalent_interval_s;
        let mesh_impulse_n_s = scale(h, external_mesh.force)?;
        let gravity_impulse_n_s = scale(h, gravity_force_n)?;
        let support_impulse_n_s = scale(h, support_force_n)?;
        let mesh_angular_impulse_n_m_s = scale(h, external_mesh.torque)?;
        let support_angular_impulse_n_m_s = scale(h, support_torque_n_m)?;
        let normal_estimate = scale(quotient(1., request.radius_m)?, a)?;
        Ok(SphereSupportForces {
            request,
            from,
            probe,
            probe_point_defect_m: difference(probe.position, p)?,
            external_mesh,
            gravity_force_n,
            total_external_force_n: f,
            support_force_n,
            contact_lever_m,
            support_torque_n_m,
            net_force_n,
            net_torque_n_m,
            normal_estimate,
            normal_norm_defect: checked(norm(normal_estimate)? - 1.)?,
            support_magnitude_n: norm(support_force_n)?,
            mesh_impulse_n_s,
            gravity_impulse_n_s,
            support_impulse_n_s,
            mesh_angular_impulse_n_m_s,
            support_angular_impulse_n_m_s,
            linear_impulse_ledger_defect_n_s: sum(
                sum(mesh_impulse_n_s, gravity_impulse_n_s)?,
                support_impulse_n_s,
            )?,
            angular_impulse_ledger_defect_n_m_s: sum(
                mesh_angular_impulse_n_m_s,
                support_angular_impulse_n_m_s,
            )?,
            external_power_w: external_mesh.rigid_power,
            support_power_w: 0.,
            external_work_j: 0.,
            support_work_j: 0.,
        })
    }
    /// Recomputes all current predicates and loads before the old atomic
    /// zero-load advance. A mutable/public proposal is never accepted as a ticket.
    pub fn advance_stationary_single_face_support(
        &mut self,
        request: SphereSupportRequest,
        surface: &TriangleSurface,
        loading: SurfaceLoading<'_>,
        mut callback: impl FnMut(SphereSupportStage, usize) -> bool,
    ) -> Result<SphereSupportHeldReport, SphereSupportError> {
        let forces =
            self.stationary_single_face_support(request, surface, loading, &mut callback)?;
        if forces.from.orientation != [1., 0., 0., 0.] {
            return Err(SphereSupportError::UnsupportedHeldOrientation);
        }
        let zeros = [[[0.; 3]; 3]; MAX_CONTACT_BODY_TRIANGLES];
        let zero_load_motion = self
            .advance(
                request.expected_body,
                request.expected_moving,
                SurfaceLoading::Traction(&zeros[..self.world_surface().triangles().len()]),
                request.equivalent_interval_s,
                |stage, i| callback(SphereSupportStage::Coast(stage), i),
            )
            .map_err(|e| match e {
                RigidMotionError::Cancelled { stage, index } => SphereSupportError::Cancelled {
                    stage: SphereSupportStage::Coast(stage),
                    index,
                },
                e => SphereSupportError::Motion(e),
            })?;
        Ok(SphereSupportHeldReport {
            forces,
            zero_load_motion,
        })
    }
}
