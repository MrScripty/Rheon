//! One isolated Coulomb impulse for the declared isotropic sphere collider.
//! Fixed contact disk, Newton normal restitution, actual q-c torque lever.
//! No persistent contact, finite force history or frictional continuation.
use crate::sphere_contact::{
    checked, cross, difference, dot, kinetic, norm, product, quotient, respond, scale, sum,
};
use crate::sphere_departure::QualifiedDeparture;
use crate::{
    MAX_CONTACT_BODY_TRIANGLES, RigidMotionError, RigidMotionReport, RigidMotionStage,
    RigidPoseSnapshot, RigidSnapshot, RigidStamp, SphereContactError, SphereContactHit,
    SphereContactSettings, SphereContactStage, SphereImpactReport, SphericalRigidMotion,
    StaticSphereSweep, SurfaceLoading, SurfaceStamp, TriangleSurface,
};
use std::{cell::RefCell, fmt};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SphereFrictionRequest {
    pub expected_body: RigidStamp,
    pub expected_moving: SurfaceStamp,
    pub expected_static: SurfaceStamp,
    pub radius_m: f64,
    pub restitution: f64,
    /// Dimensionless Coulomb impulse-disk coefficient, finite and nonnegative.
    pub coulomb_coefficient: f64,
    pub interval_s: f64,
    pub settings: SphereContactSettings,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SphereFrictionStage {
    Contact(SphereContactStage),
    /// Both normal proposal and friction response remain unpublished here.
    Friction,
}
#[derive(Debug, Clone, PartialEq)]
pub enum SphereFrictionError {
    InvalidCoefficient,
    /// Rounded point speed contradicts the admitted approaching COM speed.
    UnresolvedPointApproach,
    Contact(SphereContactError),
    Cancelled {
        stage: SphereFrictionStage,
        index: usize,
    },
}
impl fmt::Display for SphereFrictionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "isolated sphere friction refused: {self:?}")
    }
}
impl std::error::Error for SphereFrictionError {}
impl From<SphereContactError> for SphereFrictionError {
    fn from(error: SphereContactError) -> Self {
        match error {
            SphereContactError::Cancelled { stage, index } => Self::Cancelled {
                stage: SphereFrictionStage::Contact(stage),
                index,
            },
            error => Self::Contact(error),
        }
    }
}
/// Names describe nearest-rounded candidates, never a certified material mode.
/// Zero computed slip or zero mu has priority; a positive exact float tie uses
/// SlipCancellation. No threshold band or artificial slip floor is introduced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SphereFrictionCandidate {
    NoTangentialImpulse,
    SlipCancellation,
    CoulombCapped,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SphereFrictionImpact {
    pub after: RigidSnapshot,
    pub candidate: SphereFrictionCandidate,
    pub contact_lever_m: [f64; 3],
    pub represented_contact_distance_m: f64,
    pub lever_normal_cross_m: [f64; 3],
    pub point_velocity_before_m_s: [f64; 3],
    pub point_velocity_after_m_s: [f64; 3],
    pub tangent_velocity_before_m_s: [f64; 3],
    pub tangent_velocity_after_m_s: [f64; 3],
    pub slip_before_m_s: f64,
    pub slip_after_m_s: f64,
    pub inverse_tangent_mass_kg_inv: f64,
    pub normal_impulse_n_s: f64,
    pub cancellation_impulse_n_s: f64,
    pub coulomb_cap_n_s: f64,
    pub tangent_magnitude_n_s: f64,
    pub tangent_impulse_n_s: [f64; 3],
    pub impulse_n_s: [f64; 3],
    pub angular_impulse_n_m_s: [f64; 3],
    pub momentum_defect_n_s: [f64; 3],
    pub spin_momentum_defect_n_m_s: [f64; 3],
    pub world_angular_defect_n_m_s: [f64; 3],
    pub point_normal_coupling_defect_m_s: f64,
    pub tangent_impulse_normal_defect_n_s: f64,
    pub cone_excess_n_s: f64,
    pub tangent_law_defect_m_s: [f64; 3],
    pub restitution_defect_m_s: f64,
    pub kinetic_before_j: f64,
    pub kinetic_after_j: f64,
    pub predicted_energy_change_j: f64,
    pub point_midpoint_work_j: f64,
    pub energy_defect_j: f64,
    pub work_defect_j: f64,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SphereFrictionReport {
    pub request: SphereFrictionRequest,
    pub before: RigidPoseSnapshot,
    pub after: RigidPoseSnapshot,
    /// Positive on an early hit. This API stops after that single impact.
    pub unused_interval_s: f64,
    pub hit: Option<SphereContactHit>,
    /// Diagnostic normal-only intermediate; this state is NEVER published.
    pub normal_proposal: Option<SphereImpactReport>,
    pub impact: Option<SphereFrictionImpact>,
    pub coast: RigidMotionReport,
}

impl SphericalRigidMotion {
    /// Free coast, unique contact, one fixed-disk friction impulse, then STOP.
    /// All arithmetic (including diagnostics) must be representable. Overflow
    /// and detected nonzero product/division underflow refuse BEFORE publication.
    /// Cap comparisons use nearest-rounded floats; none enclose exact modes.
    /// Every error preserves owner pose, mesh, clock and all stamps.
    pub fn coast_static_sphere_friction(
        &mut self,
        request: SphereFrictionRequest,
        surface: &TriangleSurface,
        cancelled: impl FnMut(SphereFrictionStage, usize) -> bool,
    ) -> Result<SphereFrictionReport, SphereFrictionError> {
        use SphereFrictionError as E;
        if !request.coulomb_coefficient.is_finite() || request.coulomb_coefficient < 0. {
            return Err(E::InvalidCoefficient);
        }
        if !request.restitution.is_finite() || !(0. ..=1.).contains(&request.restitution) {
            return Err(SphereContactError::InvalidSettings.into());
        }
        let callback = RefCell::new(cancelled);
        let hit = StaticSphereSweep::new(
            self,
            request.expected_body,
            request.expected_moving,
            surface,
            request.expected_static,
            request.radius_m,
            request.settings,
        )?
        .first_contact(request.interval_s, |index| {
            (callback.borrow_mut())(
                SphereFrictionStage::Contact(SphereContactStage::Query),
                index,
            )
        })?;
        self.coast_static_sphere_friction_selected(
            request,
            surface,
            hit,
            None,
            callback.into_inner(),
        )
    }

    pub(crate) fn coast_static_sphere_friction_selected(
        &mut self,
        request: SphereFrictionRequest,
        surface: &TriangleSurface,
        hit: Option<SphereContactHit>,
        departure: Option<&QualifiedDeparture>,
        cancelled: impl FnMut(SphereFrictionStage, usize) -> bool,
    ) -> Result<SphereFrictionReport, SphereFrictionError> {
        use SphereFrictionError as E;
        let callback = RefCell::new(cancelled);
        let before = self.snapshot();
        if departure.is_some_and(|d| !d.matches(self, surface, request.radius_m)) {
            return Err(SphereContactError::InvalidDeparture.into());
        }
        let dt = hit.map_or(request.interval_s, |contact| contact.requested_event_dt_s);
        let unused_interval_s = checked(request.interval_s - dt)?;
        let zeros = [[[0.; 3]; 3]; MAX_CONTACT_BODY_TRIANGLES];
        let count = self.world_surface().triangles().len();
        let mut normal_proposal = None;
        let mut impact = None;
        let mut finalizer_error = None;
        let coast = self.advance_finalized(
            request.expected_body,
            request.expected_moving,
            SurfaceLoading::Traction(&zeros[..count]),
            dt,
            |stage, index| {
                (callback.borrow_mut())(SphereFrictionStage::Contact(contact_stage(stage)), index)
            },
            |state| {
                if let Some(d) = departure
                    && let Err(error) = d.endpoint(state.center_of_mass)
                {
                    finalizer_error = Some(E::from(error));
                    return Err(RigidMotionError::ArithmeticFailure);
                }
                let Some(contact) = hit else {
                    return Ok(state);
                };
                let result = (|| {
                    if (callback.borrow_mut())(
                        SphereFrictionStage::Contact(SphereContactStage::Impact),
                        0,
                    ) {
                        return Err(E::Cancelled {
                            stage: SphereFrictionStage::Contact(SphereContactStage::Impact),
                            index: 0,
                        });
                    }
                    let normal = respond(
                        state,
                        contact,
                        request.radius_m,
                        request.restitution,
                        request.settings,
                    )?;
                    if (callback.borrow_mut())(SphereFrictionStage::Friction, 0) {
                        return Err(E::Cancelled {
                            stage: SphereFrictionStage::Friction,
                            index: 0,
                        });
                    }
                    let report = friction_response(
                        normal,
                        contact.point,
                        request.coulomb_coefficient,
                        request.restitution,
                    )?;
                    normal_proposal = Some(normal);
                    impact = Some(report);
                    Ok(report.after)
                })();
                match result {
                    Ok(after) => Ok(after),
                    Err(error) => {
                        finalizer_error = Some(error);
                        Err(RigidMotionError::ArithmeticFailure)
                    }
                }
            },
        );
        if let Some(error) = finalizer_error {
            return Err(error);
        }
        let coast = coast.map_err(|error| match error {
            RigidMotionError::Cancelled { stage, index } => E::Cancelled {
                stage: SphereFrictionStage::Contact(contact_stage(stage)),
                index,
            },
            error => E::Contact(SphereContactError::Motion(error)),
        })?;
        Ok(SphereFrictionReport {
            request,
            before,
            after: self.snapshot(),
            unused_interval_s,
            hit,
            normal_proposal,
            impact,
            coast,
        })
    }
}
fn contact_stage(stage: RigidMotionStage) -> SphereContactStage {
    if stage == RigidMotionStage::Publication {
        SphereContactStage::Publication
    } else {
        SphereContactStage::Coast(stage)
    }
}
fn point_velocity(state: RigidSnapshot, r: [f64; 3]) -> Result<[f64; 3], SphereContactError> {
    sum(state.velocity_m_s, cross(state.angular_velocity_rad_s, r)?)
}
fn tangent(v: [f64; 3], n: [f64; 3]) -> Result<[f64; 3], SphereContactError> {
    difference(v, scale(dot(v, n)?, n)?)
}
fn friction_response(
    normal: SphereImpactReport,
    point: [f64; 3],
    mu: f64,
    restitution: f64,
) -> Result<SphereFrictionImpact, SphereFrictionError> {
    let state = normal.coast_body;
    let r = difference(point, state.center_of_mass)?;
    let d = norm(r)?;
    let n = normal.normal;
    let g = point_velocity(state, r)?;
    if dot(g, n)? >= 0. {
        return Err(SphereFrictionError::UnresolvedPointApproach);
    }
    let vt = tangent(g, n)?;
    let slip = norm(vt)?;
    let k =
        checked(quotient(1., state.mass_kg)? + quotient(product(d, d)?, state.inertia_kg_m2[0])?)?;
    let jn = product(
        product(-(1. + restitution), state.mass_kg)?,
        normal.normal_velocity_before_m_s,
    )?;
    let q_stop = quotient(slip, k)?;
    let cap = product(mu, jn)?;
    let (candidate, qt) = if slip == 0. || mu == 0. {
        (SphereFrictionCandidate::NoTangentialImpulse, 0.)
    } else if q_stop <= cap {
        (SphereFrictionCandidate::SlipCancellation, q_stop)
    } else {
        (SphereFrictionCandidate::CoulombCapped, cap)
    };
    let jt = if qt == 0. {
        [0.; 3]
    } else {
        scale(-quotient(qt, slip)?, vt)?
    };
    let j = sum(normal.impulse_n_s, jt)?;
    // Total actual-point torque, including any rounded normal radial residual.
    let angular = cross(r, j)?;
    let mut after = normal.after;
    after.velocity_m_s = sum(after.velocity_m_s, scale(quotient(1., state.mass_kg)?, jt)?)?;
    after.angular_velocity_rad_s = sum(
        state.angular_velocity_rad_s,
        scale(quotient(1., state.inertia_kg_m2[0])?, angular)?,
    )?;
    let ga = point_velocity(after, r)?;
    let vta = tangent(ga, n)?;
    let pd = difference(
        scale(
            state.mass_kg,
            difference(after.velocity_m_s, state.velocity_m_s)?,
        )?,
        j,
    )?;
    let sd = difference(
        scale(
            state.inertia_kg_m2[0],
            difference(after.angular_velocity_rad_s, state.angular_velocity_rad_s)?,
        )?,
        angular,
    )?;
    let world = difference(
        sum(
            cross(
                state.center_of_mass,
                scale(
                    state.mass_kg,
                    difference(after.velocity_m_s, state.velocity_m_s)?,
                )?,
            )?,
            scale(
                state.inertia_kg_m2[0],
                difference(after.angular_velocity_rad_s, state.angular_velocity_rad_s)?,
            )?,
        )?,
        cross(point, j)?,
    )?;
    let tangent_change =
        checked(-product(qt, slip)? + product(0.5, product(k, product(qt, qt)?)?)?)?;
    let predicted = checked(normal.predicted_energy_change_j + tangent_change)?;
    let kb = kinetic(state)?;
    let ka = kinetic(after)?;
    let work = dot(scale(0.5, sum(g, ga)?)?, j)?;
    Ok(SphereFrictionImpact {
        after,
        candidate,
        contact_lever_m: r,
        represented_contact_distance_m: d,
        lever_normal_cross_m: cross(r, n)?,
        point_velocity_before_m_s: g,
        point_velocity_after_m_s: ga,
        tangent_velocity_before_m_s: vt,
        tangent_velocity_after_m_s: vta,
        slip_before_m_s: slip,
        slip_after_m_s: norm(vta)?,
        inverse_tangent_mass_kg_inv: k,
        normal_impulse_n_s: jn,
        cancellation_impulse_n_s: q_stop,
        coulomb_cap_n_s: cap,
        tangent_magnitude_n_s: qt,
        tangent_impulse_n_s: jt,
        impulse_n_s: j,
        angular_impulse_n_m_s: angular,
        momentum_defect_n_s: pd,
        spin_momentum_defect_n_m_s: sd,
        world_angular_defect_n_m_s: world,
        point_normal_coupling_defect_m_s: checked(dot(g, n)? - normal.normal_velocity_before_m_s)?,
        tangent_impulse_normal_defect_n_s: dot(jt, n)?,
        cone_excess_n_s: checked(norm(jt)? - cap)?.max(0.),
        tangent_law_defect_m_s: difference(vta, sum(vt, scale(k, jt)?)?)?,
        restitution_defect_m_s: checked(
            dot(after.velocity_m_s, n)? + product(restitution, normal.normal_velocity_before_m_s)?,
        )?,
        kinetic_before_j: kb,
        kinetic_after_j: ka,
        predicted_energy_change_j: predicted,
        point_midpoint_work_j: work,
        energy_defect_j: checked(checked(ka - kb)? - predicted)?,
        work_defect_j: checked(checked(ka - kb)? - work)?,
    })
}
