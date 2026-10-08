//! Atomic velocity response to an imposed mesh impulse at a fixed pose.
use crate::{MeshLoadError, SurfaceStamp, TriangleMeshLoad};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RigidStamp {
    pub id: u64,
    pub generation: u64,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RigidSnapshot {
    pub stamp: RigidStamp,
    pub surface: SurfaceStamp,
    pub center_of_mass: [f64; 3],
    pub mass_kg: f64,
    /// Principal moments about COM, diagonal in the fixed WORLD axes, kg m².
    pub inertia_kg_m2: [f64; 3],
    pub velocity_m_s: [f64; 3],
    pub angular_velocity_rad_s: [f64; 3],
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigidImpulseStage {
    Admission,
    Reduction,
    Proposal,
    Publication,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RigidImpulseError {
    InvalidBody,
    InvalidDuration,
    StaleBody,
    StaleSurface,
    ReferenceMismatch,
    GenerationOverflow,
    ArithmeticFailure,
    Cancelled {
        stage: RigidImpulseStage,
        triangle: usize,
    },
    Mesh(MeshLoadError),
}
impl fmt::Display for RigidImpulseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rigid mesh impulse rejected operation: {self:?}")
    }
}
impl std::error::Error for RigidImpulseError {}

/// All defects are nearest-rounded diagnostics, NOT error enclosures or gates
/// authorizing a fluid step, pose advance, or rigid-dynamics integration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RigidImpulseReport {
    pub before: RigidSnapshot,
    pub after: RigidSnapshot,
    pub equivalent_duration_s: f64,
    pub force_n: [f64; 3],
    pub torque_n_m: [f64; 3],
    pub impulse_n_s: [f64; 3],
    pub angular_impulse_n_m_s: [f64; 3],
    /// m*(ACTUAL stored V_after - V_before) - J, kg m/s.
    pub momentum_defect: [f64; 3],
    /// I*(ACTUAL stored omega_after - omega_before) - K, kg m²/s.
    pub angular_momentum_defect: [f64; 3],
    pub kinetic_before_j: f64,
    pub kinetic_after_j: f64,
    /// J dot V_mid + K dot omega_mid; may be negative, in joules.
    pub impulse_work_j: f64,
    /// Actual momentum increments paired with actual midpoint twist, joules.
    pub update_work_j: f64,
    /// Delta kinetic energy minus impulse work, joules.
    pub energy_defect_j: f64,
    /// Delta kinetic energy minus actual-update work, joules.
    pub update_energy_defect_j: f64,
}

/// Owns velocities, not mesh geometry or a clock. COM/inertia are caller-supplied
/// physical properties, not inferred from facets. No orientation/contact update.
#[derive(Debug)]
pub struct FrozenRigidBody {
    state: RigidSnapshot,
}
impl FrozenRigidBody {
    pub fn new(state: RigidSnapshot) -> Result<Self, RigidImpulseError> {
        if !positive(state.mass_kg)
            || !physical_inertia(state.inertia_kg_m2)
            || !state.center_of_mass.into_iter().all(f64::is_finite)
            || !state.velocity_m_s.into_iter().all(f64::is_finite)
            || !state.angular_velocity_rad_s.into_iter().all(f64::is_finite)
            || kinetic(state).is_none()
        {
            return Err(RigidImpulseError::InvalidBody);
        }
        Ok(Self { state })
    }
    pub fn snapshot(&self) -> RigidSnapshot {
        self.state
    }

    /// Apply J=s*F and K=s*tau as instantaneous impulses at this fixed pose.
    /// s is an equivalent force duration, NOT elapsed body time. In particular,
    /// this is not finite-duration torque integration for anisotropic inertia.
    /// Every failure/cancellation leaves the complete stored snapshot unchanged.
    pub fn apply_mesh_impulse(
        &mut self,
        expected: RigidStamp,
        load: &TriangleMeshLoad<'_>,
        equivalent_duration_s: f64,
        mut cancelled: impl FnMut(RigidImpulseStage, usize) -> bool,
    ) -> Result<RigidImpulseReport, RigidImpulseError> {
        use RigidImpulseError as E;
        let before = self.state;
        if expected != before.stamp {
            return Err(E::StaleBody);
        }
        if load.surface().stamp() != before.surface {
            return Err(E::StaleSurface);
        }
        if load.moment_reference() != before.center_of_mass {
            return Err(E::ReferenceMismatch);
        }
        if !positive(equivalent_duration_s) {
            return Err(E::InvalidDuration);
        }
        let generation = before
            .stamp
            .generation
            .checked_add(1)
            .ok_or(E::GenerationOverflow)?;
        let mut check = |stage, triangle| {
            if cancelled(stage, triangle) {
                Err(E::Cancelled { stage, triangle })
            } else {
                Ok(())
            }
        };
        check(RigidImpulseStage::Admission, 0)?;
        let reduced = load
            .reduce([0.0; 3], [0.0; 3], |_, i| {
                check(RigidImpulseStage::Reduction, i).is_err()
            })
            .map_err(|error| match error {
                MeshLoadError::Cancelled { triangle, .. } => E::Cancelled {
                    stage: RigidImpulseStage::Reduction,
                    triangle,
                },
                error => E::Mesh(error),
            })?;
        check(RigidImpulseStage::Proposal, 0)?;
        let report = (|| -> Option<RigidImpulseReport> {
            let mut after = before;
            after.stamp.generation = generation;
            let mut impulse = [0.0; 3];
            let mut angular_impulse = [0.0; 3];
            let mut momentum_defect = [0.0; 3];
            let mut angular_momentum_defect = [0.0; 3];
            let mut impulse_work = 0.0;
            let mut update_work = 0.0;
            for i in 0..3 {
                impulse[i] = mul(equivalent_duration_s, reduced.force[i])?;
                angular_impulse[i] = mul(equivalent_duration_s, reduced.torque[i])?;
                after.velocity_m_s[i] =
                    add(before.velocity_m_s[i], div(impulse[i], before.mass_kg)?)?;
                after.angular_velocity_rad_s[i] = add(
                    before.angular_velocity_rad_s[i],
                    div(angular_impulse[i], before.inertia_kg_m2[i])?,
                )?;
                let dp = mul(
                    before.mass_kg,
                    add(after.velocity_m_s[i], -before.velocity_m_s[i])?,
                )?;
                let dl = mul(
                    before.inertia_kg_m2[i],
                    add(
                        after.angular_velocity_rad_s[i],
                        -before.angular_velocity_rad_s[i],
                    )?,
                )?;
                momentum_defect[i] = add(dp, -impulse[i])?;
                angular_momentum_defect[i] = add(dl, -angular_impulse[i])?;
                let vmid = add(
                    mul(0.5, before.velocity_m_s[i])?,
                    mul(0.5, after.velocity_m_s[i])?,
                )?;
                let wmid = add(
                    mul(0.5, before.angular_velocity_rad_s[i])?,
                    mul(0.5, after.angular_velocity_rad_s[i])?,
                )?;
                impulse_work = add(
                    impulse_work,
                    add(mul(impulse[i], vmid)?, mul(angular_impulse[i], wmid)?)?,
                )?;
                update_work = add(update_work, add(mul(dp, vmid)?, mul(dl, wmid)?)?)?;
            }
            let kinetic_before = kinetic(before)?;
            let kinetic_after = kinetic(after)?;
            let delta = add(kinetic_after, -kinetic_before)?;
            Some(RigidImpulseReport {
                before,
                after,
                equivalent_duration_s,
                force_n: reduced.force,
                torque_n_m: reduced.torque,
                impulse_n_s: impulse,
                angular_impulse_n_m_s: angular_impulse,
                momentum_defect,
                angular_momentum_defect,
                kinetic_before_j: kinetic_before,
                kinetic_after_j: kinetic_after,
                impulse_work_j: impulse_work,
                update_work_j: update_work,
                energy_defect_j: add(delta, -impulse_work)?,
                update_energy_defect_j: add(delta, -update_work)?,
            })
        })()
        .ok_or(E::ArithmeticFailure)?;
        check(RigidImpulseStage::Publication, 0)?;
        self.state = report.after;
        Ok(report)
    }
}

fn positive(x: f64) -> bool {
    x.is_finite() && x > 0.0
}
/// For positive finite sorted a>=b>=c, physicality reduces to a<=b+c.
/// If a<=2b, Sterbenz makes a-b exact under ordinary IEEE gradual-underflow
/// semantics; comparing to c avoids rounding b+c and overflow. 2b is exact
/// whenever finite. This is not a formal IEEE proof; rational lab tests audit it.
fn physical_inertia(mut moments: [f64; 3]) -> bool {
    if !moments.into_iter().all(positive) {
        return false;
    }
    moments.sort_by(|a, b| b.total_cmp(a));
    let [a, b, c] = moments;
    let doubled = 2.0 * b;
    if doubled.is_finite() && a > doubled {
        return false;
    }
    a - b <= c
}
fn add(a: f64, b: f64) -> Option<f64> {
    let c = a + b;
    c.is_finite().then_some(c)
}
fn mul(a: f64, b: f64) -> Option<f64> {
    let c = a * b;
    (c.is_finite() && (c != 0.0 || a == 0.0 || b == 0.0)).then_some(c)
}
fn div(a: f64, b: f64) -> Option<f64> {
    let c = a / b;
    (c.is_finite() && (c != 0.0 || a == 0.0)).then_some(c)
}
fn kinetic(state: RigidSnapshot) -> Option<f64> {
    let mut energy = 0.0;
    for i in 0..3 {
        energy = add(
            energy,
            mul(
                0.5,
                mul(
                    mul(state.mass_kg, state.velocity_m_s[i])?,
                    state.velocity_m_s[i],
                )?,
            )?,
        )?;
        energy = add(
            energy,
            mul(
                0.5,
                mul(
                    mul(state.inertia_kg_m2[i], state.angular_velocity_rad_s[i])?,
                    state.angular_velocity_rad_s[i],
                )?,
            )?,
        )?;
    }
    Some(energy)
}
