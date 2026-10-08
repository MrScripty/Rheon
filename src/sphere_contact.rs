//! Explicit spherical collider versus a static two-sided finite triangle union.
//! One isolated frictionless event; no arbitrary moving-mesh CCD or resting law.
use crate::rigid_impulse::{add, div, mul};
use crate::{
    RigidMotionError, RigidMotionReport, RigidMotionStage, RigidPoseSnapshot, RigidSnapshot,
    RigidStamp, SphericalRigidMotion, SurfaceLoading, SurfaceStamp, TriangleSurface,
};
use std::{cell::RefCell, fmt};

pub const MAX_CONTACT_BODY_TRIANGLES: usize = 64;
#[derive(Debug, Clone, Copy)]
pub struct SphereContactSettings {
    pub static_triangle_limit: usize,
    /// Metres: allowed measured contact gap residual, not a true error bound.
    pub max_gap_residual_m: f64,
    /// Seconds: near-simultaneous distinct facets are unresolved, not combined.
    pub simultaneous_window_s: f64,
}
impl Default for SphereContactSettings {
    fn default() -> Self {
        Self {
            static_triangle_limit: 64,
            max_gap_residual_m: 1e-10,
            simultaneous_window_s: 1e-10,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SphereFeature {
    Face,
    /// Local triangle edge (0,1), (1,2) or (2,0).
    Edge(usize),
    Vertex(usize),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SphereContactStage {
    Query,
    Coast(RigidMotionStage),
    Impact,
    Publication,
}
#[derive(Debug, Clone, PartialEq)]
pub enum SphereContactError {
    InvalidSettings,
    InvalidRadius,
    RadiusInertiaMismatch,
    StaleBody,
    StaleMovingSurface,
    StaleStaticSurface,
    InvalidDuration,
    TriangleLimit,
    InitialContact {
        triangle: usize,
    },
    Ambiguous {
        triangle: usize,
    },
    Simultaneous {
        first: usize,
        second: usize,
    },
    GapResidual,
    ArithmeticFailure,
    Cancelled {
        stage: SphereContactStage,
        index: usize,
    },
    Motion(RigidMotionError),
}
impl fmt::Display for SphereContactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "spherical static contact refused: {self:?}")
    }
}
impl std::error::Error for SphereContactError {}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SphereContactHit {
    pub static_surface: SurfaceStamp,
    pub triangle: usize,
    pub feature: SphereFeature,
    pub parameter: f64,
    pub requested_event_dt_s: f64,
    pub center: [f64; 3],
    pub point: [f64; 3],
    /// Points from the static closest point toward the sphere center.
    pub normal: [f64; 3],
    pub barycentric: [f64; 3],
    /// Nearest-rounded and unenclosed, as are all subsequent diagnostics.
    pub gap_residual_m: f64,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SphereImpactReport {
    /// Actual stored free-coast endpoint, before impact publication.
    pub coast_body: RigidSnapshot,
    pub after: RigidSnapshot,
    pub normal: [f64; 3],
    pub normal_norm_defect: f64,
    pub gap_residual_m: f64,
    pub normal_velocity_before_m_s: f64,
    pub normal_velocity_after_m_s: f64,
    pub impulse_n_s: [f64; 3],
    pub momentum_defect: [f64; 3],
    pub restitution_defect_m_s: f64,
    pub kinetic_before_j: f64,
    pub kinetic_after_j: f64,
    pub predicted_energy_change_j: f64,
    pub energy_defect_j: f64,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SphereContactReport {
    pub before: RigidPoseSnapshot,
    pub after: RigidPoseSnapshot,
    pub static_surface: SurfaceStamp,
    pub radius_m: f64,
    pub restitution: f64,
    pub requested_interval_s: f64,
    /// Unused requested interval on a hit. There is no continuation solver.
    pub unused_interval_s: f64,
    pub hit: Option<SphereContactHit>,
    pub impact: Option<SphereImpactReport>,
    /// PR35 actual free-coast proposal; after.body precedes the impact.
    pub coast: RigidMotionReport,
}

/// Allocation-free borrowed query. The radius is a declared collision shape,
/// not inferred from the moving render/traction mesh. Caller declares a physical
/// isotropic mass distribution within R; admission only checks a necessary bound.
pub struct StaticSphereSweep<'a> {
    moving: &'a SphericalRigidMotion,
    surface: &'a TriangleSurface,
    radius: f64,
    settings: SphereContactSettings,
}
impl<'a> StaticSphereSweep<'a> {
    pub fn new(
        moving: &'a SphericalRigidMotion,
        expected: RigidStamp,
        expected_moving: SurfaceStamp,
        surface: &'a TriangleSurface,
        expected_static: SurfaceStamp,
        radius_m: f64,
        settings: SphereContactSettings,
    ) -> Result<Self, SphereContactError> {
        use SphereContactError as E;
        let body = moving.snapshot().body;
        if body.stamp != expected {
            return Err(E::StaleBody);
        }
        if body.surface != expected_moving {
            return Err(E::StaleMovingSurface);
        }
        if surface.stamp() != expected_static {
            return Err(E::StaleStaticSurface);
        }
        if !positive(radius_m) {
            return Err(E::InvalidRadius);
        }
        if !positive(settings.max_gap_residual_m) || !positive(settings.simultaneous_window_s) {
            return Err(E::InvalidSettings);
        }
        if surface.triangles().len() > settings.static_triangle_limit
            || moving.world_surface().triangles().len() > MAX_CONTACT_BODY_TRIANGLES
        {
            return Err(E::TriangleLimit);
        }
        // Conservative sufficient admission for the necessary 3I <= 2mR².
        // Outward adjacent floats at EACH nonnegative product; uncertain range
        // refuses. This is a numerical check, not a formal IEEE proof.
        let left = checked(body.inertia_kg_m2[0] * 3.)?.next_up();
        let right = checked(
            checked(checked(body.mass_kg * 2.)?.next_down() * radius_m)?.next_down() * radius_m,
        )?
        .next_down();
        if !left.is_finite() || right <= 0. || left > right {
            return Err(E::RadiusInertiaMismatch);
        }
        Ok(Self {
            moving,
            surface,
            radius: radius_m,
            settings,
        })
    }

    /// Full finite-feature scan. Any unresolved relevant candidate rejects the
    /// whole interval even when a different facet has an earlier definite hit.
    /// Duplicate facets therefore can produce an unresolved simultaneous event.
    pub fn first_contact(
        &self,
        h: f64,
        mut cancelled: impl FnMut(usize) -> bool,
    ) -> Result<Option<SphereContactHit>, SphereContactError> {
        use SphereContactError as E;
        if !positive(h) {
            return Err(E::InvalidDuration);
        }
        let body = self.moving.snapshot().body;
        let direction = scale(h, body.velocity_m_s)?;
        let mut best: Option<SphereContactHit> = None;
        for (triangle, &indices) in self.surface.triangles().iter().enumerate() {
            if cancelled(triangle) {
                return Err(E::Cancelled {
                    stage: SphereContactStage::Query,
                    index: triangle,
                });
            }
            let vertices = indices.map(|i| self.surface.vertices()[i]);
            let tol = self.surface.relative_tolerance();
            let local = Local::new(body.center_of_mass, direction, vertices, self.radius)?;
            if local.distance(local.w)? <= local.r + tol {
                return Err(E::InitialContact { triangle });
            }
            if let Some(candidate) = local.sweep(tol, triangle)? {
                let dt = product(h, candidate.s)?;
                let center = sum(body.center_of_mass, scale(candidate.s, direction)?)?;
                let point = sum(vertices[0], scale(local.length, candidate.point)?)?;
                let radial = difference(center, point)?;
                let distance = norm(radial)?;
                let normal = scale(quotient(1., distance)?, radial)?;
                let gap = checked(distance - self.radius)?;
                if gap.abs() > self.settings.max_gap_residual_m {
                    return Err(E::GapResidual);
                }
                let hit = SphereContactHit {
                    static_surface: self.surface.stamp(),
                    triangle,
                    feature: candidate.feature,
                    parameter: candidate.s,
                    requested_event_dt_s: dt,
                    center,
                    point,
                    normal,
                    barycentric: candidate.barycentric,
                    gap_residual_m: gap,
                };
                if let Some(old) = best {
                    if (dt - old.requested_event_dt_s).abs() <= self.settings.simultaneous_window_s
                    {
                        return Err(E::Simultaneous {
                            first: old.triangle,
                            second: triangle,
                        });
                    }
                    if dt < old.requested_event_dt_s {
                        best = Some(hit);
                    }
                } else {
                    best = Some(hit);
                }
            }
        }
        Ok(best)
    }
}

impl SphericalRigidMotion {
    /// No loads: free coast to one isolated sphere/static-facet event, apply one
    /// frictionless normal impulse and STOP. On miss coast h. All refusal paths
    /// preserve body, clock, orientation, mesh, versions and payload accounting.
    #[allow(clippy::too_many_arguments)]
    pub fn coast_static_sphere(
        &mut self,
        expected: RigidStamp,
        expected_moving: SurfaceStamp,
        surface: &TriangleSurface,
        expected_static: SurfaceStamp,
        radius_m: f64,
        restitution: f64,
        h: f64,
        settings: SphereContactSettings,
        cancelled: impl FnMut(SphereContactStage, usize) -> bool,
    ) -> Result<SphereContactReport, SphereContactError> {
        use SphereContactError as E;
        if !restitution.is_finite() || !(0. ..=1.).contains(&restitution) {
            return Err(E::InvalidSettings);
        }
        let callback = RefCell::new(cancelled);
        let before = self.snapshot();
        let hit = StaticSphereSweep::new(
            self,
            expected,
            expected_moving,
            surface,
            expected_static,
            radius_m,
            settings,
        )?
        .first_contact(h, |i| (callback.borrow_mut())(SphereContactStage::Query, i))?;
        let dt = hit.map_or(h, |x| x.requested_event_dt_s);
        let unused = checked(h - dt)?;
        let zeros = [[[0.; 3]; 3]; MAX_CONTACT_BODY_TRIANGLES];
        let count = self.world_surface().triangles().len();
        let mut impact = None;
        let mut finalizer_error = None;
        let coast = self.advance_finalized(
            expected,
            expected_moving,
            SurfaceLoading::Traction(&zeros[..count]),
            dt,
            |stage, i| {
                (callback.borrow_mut())(
                    if stage == RigidMotionStage::Publication {
                        SphereContactStage::Publication
                    } else {
                        SphereContactStage::Coast(stage)
                    },
                    i,
                )
            },
            |state| {
                if let Some(contact) = hit {
                    let result = (|| {
                        if (callback.borrow_mut())(SphereContactStage::Impact, 0) {
                            return Err(E::Cancelled {
                                stage: SphereContactStage::Impact,
                                index: 0,
                            });
                        }
                        let report = respond(state, contact, radius_m, restitution, settings)?;
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
                } else {
                    Ok(state)
                }
            },
        );
        if let Some(error) = finalizer_error {
            return Err(error);
        }
        let coast = coast.map_err(|error| match error {
            RigidMotionError::Cancelled { stage, index } => E::Cancelled {
                stage: if stage == RigidMotionStage::Publication {
                    SphereContactStage::Publication
                } else {
                    SphereContactStage::Coast(stage)
                },
                index,
            },
            error => E::Motion(error),
        })?;
        Ok(SphereContactReport {
            before,
            after: self.snapshot(),
            static_surface: surface.stamp(),
            radius_m,
            restitution,
            requested_interval_s: h,
            unused_interval_s: unused,
            hit,
            impact,
            coast,
        })
    }
}

fn respond(
    state: RigidSnapshot,
    hit: SphereContactHit,
    radius: f64,
    restitution: f64,
    settings: SphereContactSettings,
) -> Result<SphereImpactReport, SphereContactError> {
    let radial = difference(state.center_of_mass, hit.point)?;
    let distance = norm(radial)?;
    let gap = checked(distance - radius)?;
    if gap.abs() > settings.max_gap_residual_m {
        return Err(SphereContactError::GapResidual);
    }
    let normal = scale(quotient(1., distance)?, radial)?;
    let vn = dot(state.velocity_m_s, normal)?;
    if vn >= -product(norm(state.velocity_m_s)?, 32. * f64::EPSILON)? {
        return Err(SphereContactError::Ambiguous {
            triangle: hit.triangle,
        });
    }
    let j = product(product(-(1. + restitution), state.mass_kg)?, vn)?;
    let impulse = scale(j, normal)?;
    let mut after = state;
    let mut defect = [0.; 3];
    for i in 0..3 {
        after.velocity_m_s[i] =
            checked(state.velocity_m_s[i] + quotient(impulse[i], state.mass_kg)?)?;
        defect[i] = checked(
            product(
                state.mass_kg,
                checked(after.velocity_m_s[i] - state.velocity_m_s[i])?,
            )? - impulse[i],
        )?;
    }
    let va = dot(after.velocity_m_s, normal)?;
    let kb = kinetic(state)?;
    let ka = kinetic(after)?;
    let predicted = product(
        product(
            -0.5 * state.mass_kg,
            checked(1. - product(restitution, restitution)?)?,
        )?,
        product(vn, vn)?,
    )?;
    Ok(SphereImpactReport {
        coast_body: state,
        after,
        normal,
        normal_norm_defect: checked(norm(normal)? - 1.)?,
        gap_residual_m: gap,
        normal_velocity_before_m_s: vn,
        normal_velocity_after_m_s: va,
        impulse_n_s: impulse,
        momentum_defect: defect,
        restitution_defect_m_s: checked(va + product(restitution, vn)?)?,
        kinetic_before_j: kb,
        kinetic_after_j: ka,
        predicted_energy_change_j: predicted,
        energy_defect_j: checked(checked(ka - kb)? - predicted)?,
    })
}

#[derive(Clone, Copy)]
struct Candidate {
    s: f64,
    point: [f64; 3],
    barycentric: [f64; 3],
    feature: SphereFeature,
}
type RegionPoint = ([f64; 3], [f64; 3]);
struct Local {
    w: [f64; 3],
    d: [f64; 3],
    p: [[f64; 3]; 3],
    r: f64,
    length: f64,
}
impl Local {
    fn new(
        c: [f64; 3],
        d: [f64; 3],
        vertices: [[f64; 3]; 3],
        r: f64,
    ) -> Result<Self, SphereContactError> {
        let w = difference(c, vertices[0])?;
        let b = difference(vertices[1], vertices[0])?;
        let e = difference(vertices[2], vertices[0])?;
        let length = w
            .into_iter()
            .chain(d)
            .chain(b)
            .chain(e)
            .fold(r, |a, x| a.max(x.abs()));
        let inverse = quotient(1., length)?;
        Ok(Self {
            w: scale(inverse, w)?,
            d: scale(inverse, d)?,
            p: [[0.; 3], scale(inverse, b)?, scale(inverse, e)?],
            r: product(r, inverse)?,
            length,
        })
    }
    fn bary(&self, x: [f64; 3]) -> Result<[f64; 3], SphereContactError> {
        let b = self.p[1];
        let c = self.p[2];
        let bb = dot(b, b)?;
        let bc = dot(b, c)?;
        let cc = dot(c, c)?;
        let det = checked(product(bb, cc)? - product(bc, bc)?)?;
        if det <= 0. {
            return Err(SphereContactError::ArithmeticFailure);
        }
        let xb = dot(x, b)?;
        let xc = dot(x, c)?;
        let u = quotient(checked(product(cc, xb)? - product(bc, xc)?)?, det)?;
        let v = quotient(checked(product(bb, xc)? - product(bc, xb)?)?, det)?;
        Ok([checked(1. - u - v)?, u, v])
    }
    fn distance(&self, x: [f64; 3]) -> Result<f64, SphereContactError> {
        let bary = self.bary(x)?;
        if bary.into_iter().all(|v| v >= 0.) {
            let q = sum(scale(bary[1], self.p[1])?, scale(bary[2], self.p[2])?)?;
            return norm(difference(x, q)?);
        }
        let mut nearest = f64::INFINITY;
        for k in 0..3 {
            let a = self.p[k];
            let e = difference(self.p[(k + 1) % 3], a)?;
            let t = quotient(dot(difference(x, a)?, e)?, dot(e, e)?)?.clamp(0., 1.);
            nearest = nearest.min(norm(difference(x, sum(a, scale(t, e)?)?)?)?);
        }
        Ok(nearest)
    }
    fn region(
        &self,
        feature: SphereFeature,
        x: [f64; 3],
        tol: f64,
        triangle: usize,
    ) -> Result<Option<RegionPoint>, SphereContactError> {
        let ambiguous = || SphereContactError::Ambiguous { triangle };
        match feature {
            SphereFeature::Face => {
                let bary = self.bary(x)?;
                if bary.into_iter().any(|v| v < -tol) {
                    return Ok(None);
                }
                if bary.into_iter().any(|v| v <= tol) {
                    return Err(ambiguous());
                }
                Ok(Some((
                    sum(scale(bary[1], self.p[1])?, scale(bary[2], self.p[2])?)?,
                    bary,
                )))
            }
            SphereFeature::Edge(k) => {
                let a = self.p[k];
                let b = self.p[(k + 1) % 3];
                let e = difference(b, a)?;
                let t = quotient(dot(difference(x, a)?, e)?, dot(e, e)?)?;
                if t < -tol || t > 1. + tol {
                    return Ok(None);
                }
                if t <= tol || t >= 1. - tol {
                    return Err(ambiguous());
                }
                let q = sum(a, scale(t, e)?)?;
                let radial = difference(x, q)?;
                let interior = difference(self.p[(k + 2) % 3], q)?;
                let sign = dot(radial, interior)?;
                let margin = product(tol, product(norm(radial)?, norm(interior)?)?)?;
                if sign > margin {
                    return Ok(None);
                }
                if sign >= -margin {
                    return Err(ambiguous());
                }
                let mut bary = [0.; 3];
                bary[k] = 1. - t;
                bary[(k + 1) % 3] = t;
                Ok(Some((q, bary)))
            }
            SphereFeature::Vertex(k) => {
                let radial = difference(x, self.p[k])?;
                let mut boundary = false;
                for j in [(k + 1) % 3, (k + 2) % 3] {
                    let edge = difference(self.p[j], self.p[k])?;
                    let sign = dot(radial, edge)?;
                    let margin = product(tol, product(norm(radial)?, norm(edge)?)?)?;
                    if sign > margin {
                        return Ok(None);
                    }
                    boundary |= sign >= -margin;
                }
                if boundary {
                    return Err(ambiguous());
                }
                let mut bary = [0.; 3];
                bary[k] = 1.;
                Ok(Some((self.p[k], bary)))
            }
        }
    }
    fn sweep(&self, tol: f64, triangle: usize) -> Result<Option<Candidate>, SphereContactError> {
        let speed = norm(self.d)?;
        if speed == 0. {
            return Ok(None);
        }
        let mut best: Option<Candidate> = None;
        let mut consider = |s: f64, feature, tangent: bool| -> Result<(), SphereContactError> {
            if !(0. ..=1.).contains(&s) {
                return Ok(());
            }
            let x = sum(self.w, scale(s, self.d)?)?;
            if let Some((point, barycentric)) = self.region(feature, x, tol, triangle)? {
                let radial = difference(x, point)?;
                let vn = quotient(dot(self.d, radial)?, norm(radial)?)?;
                if vn > product(tol, speed)? {
                    return Ok(());
                }
                if tangent || vn >= -product(tol, speed)? || s <= tol {
                    return Err(SphereContactError::Ambiguous { triangle });
                }
                let candidate = Candidate {
                    s,
                    point,
                    barycentric,
                    feature,
                };
                if best.is_none_or(|old| s < old.s) {
                    best = Some(candidate);
                }
            }
            Ok(())
        };
        let raw = cross(self.p[1], self.p[2])?;
        let n = scale(quotient(1., norm(raw)?)?, raw)?;
        let height = dot(self.w, n)?;
        let dh = dot(self.d, n)?;
        if dh != 0. {
            for sign in [-1., 1.] {
                let s = quotient(checked(sign * self.r - height)?, dh)?;
                consider(s, SphereFeature::Face, dh.abs() <= product(tol, speed)?)?;
            }
        }
        for k in 0..3 {
            let e = difference(self.p[(k + 1) % 3], self.p[k])?;
            let w = difference(self.w, self.p[k])?;
            let ee = dot(e, e)?;
            let wp = difference(w, scale(quotient(dot(w, e)?, ee)?, e)?)?;
            let dp = difference(self.d, scale(quotient(dot(self.d, e)?, ee)?, e)?)?;
            for (a, b, feature) in [
                (wp, dp, SphereFeature::Edge(k)),
                (w, self.d, SphereFeature::Vertex(k)),
            ] {
                let aa = dot(b, b)?;
                if aa == 0. {
                    continue;
                }
                let bb = dot(a, b)?;
                let cc = checked(dot(a, a)? - product(self.r, self.r)?)?;
                let b2 = product(bb, bb)?;
                let ac = product(aa, cc)?;
                let disc = checked(b2 - ac)?;
                let threshold = product(tol, checked(b2 + ac.abs())?)?;
                if disc < -threshold {
                    continue;
                }
                if disc <= threshold {
                    consider(quotient(-bb, aa)?, feature, true)?;
                    continue;
                }
                let q = checked(-bb - disc.sqrt().copysign(bb))?;
                let mut roots = [quotient(q, aa)?, quotient(cc, q)?];
                roots.sort_by(f64::total_cmp);
                for s in roots {
                    consider(s, feature, false)?;
                }
            }
        }
        Ok(best)
    }
}
fn positive(x: f64) -> bool {
    x.is_finite() && x > 0.
}
fn checked(x: f64) -> Result<f64, SphereContactError> {
    x.is_finite()
        .then_some(x)
        .ok_or(SphereContactError::ArithmeticFailure)
}
fn product(a: f64, b: f64) -> Result<f64, SphereContactError> {
    mul(a, b).ok_or(SphereContactError::ArithmeticFailure)
}
fn quotient(a: f64, b: f64) -> Result<f64, SphereContactError> {
    div(a, b).ok_or(SphereContactError::ArithmeticFailure)
}
fn sum(a: [f64; 3], b: [f64; 3]) -> Result<[f64; 3], SphereContactError> {
    let mut c = [0.; 3];
    for i in 0..3 {
        c[i] = add(a[i], b[i]).ok_or(SphereContactError::ArithmeticFailure)?;
    }
    Ok(c)
}
fn difference(a: [f64; 3], b: [f64; 3]) -> Result<[f64; 3], SphereContactError> {
    sum(a, b.map(|x| -x))
}
fn scale(s: f64, a: [f64; 3]) -> Result<[f64; 3], SphereContactError> {
    let mut c = [0.; 3];
    for i in 0..3 {
        c[i] = product(s, a[i])?;
    }
    Ok(c)
}
fn dot(a: [f64; 3], b: [f64; 3]) -> Result<f64, SphereContactError> {
    checked(checked(product(a[0], b[0])? + product(a[1], b[1])?)? + product(a[2], b[2])?)
}
fn norm(a: [f64; 3]) -> Result<f64, SphereContactError> {
    checked(a[0].hypot(a[1]).hypot(a[2]))
}
fn cross(a: [f64; 3], b: [f64; 3]) -> Result<[f64; 3], SphereContactError> {
    Ok([
        checked(product(a[1], b[2])? - product(a[2], b[1])?)?,
        checked(product(a[2], b[0])? - product(a[0], b[2])?)?,
        checked(product(a[0], b[1])? - product(a[1], b[0])?)?,
    ])
}
fn kinetic(state: RigidSnapshot) -> Result<f64, SphereContactError> {
    let mut energy = 0.;
    for i in 0..3 {
        energy = checked(
            energy
                + product(
                    0.5,
                    product(
                        product(state.mass_kg, state.velocity_m_s[i])?,
                        state.velocity_m_s[i],
                    )?,
                )?,
        )?;
        energy = checked(
            energy
                + product(
                    0.5,
                    product(
                        product(state.inertia_kg_m2[i], state.angular_velocity_rad_s[i])?,
                        state.angular_velocity_rad_s[i],
                    )?,
                )?,
        )?;
    }
    Ok(energy)
}
