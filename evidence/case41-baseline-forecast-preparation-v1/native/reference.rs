//! Bounded two-column periodic, z-invariant extrusion. The original xy API is
//! the zero-third slice; the explicit extrusion API advances all components.
//! The physical mass path is integrated; endpoint donor momentum is first order.
//! No continuum traction, sign-isolation or mesh-family theorem is implied.
#![allow(clippy::needless_range_loop, dead_code)]
use crate::translated_viscous::{add, check, div, dot, mul, norm};
use crate::*;

const N: usize = 16;
const V: usize = 22;
const P: usize = 16;
const T: usize = 24;
const F: usize = 72;
const KNOWN: [usize; 7] = [2, 3, 12, 0, 1, 4, 13];
const COORD: [usize; 7] = [0, 1, 2, 3, 4, 5, 2];
const UNKNOWN: [usize; 15] = [5, 6, 7, 8, 9, 10, 11, 14, 15, 16, 17, 18, 19, 20, 21];
const ROWS: [usize; 15] = [0, 2, 3, 4, 5, 6, 8, 10, 11, 12, 14, 16, 17, 18, 20];
const BOTTOM: [f64; 3] = [0., 0.5, 1.];
const LIMIT: f64 = 1e-11;
const NEWTON: f64 = 1e-13;
const W: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoupledDiscreteStage {
    BeforeForceInputs,
    ForceInput,
    BeforeGeometry,
    Quadrature,
    BeforeSolve,
    Iteration,
    BeforeAcceptance,
    BeforeThirdSolve,
    ThirdAssembly,
    AfterThirdSolve,
    BeforePublish,
}
#[derive(Debug, Clone, PartialEq)]
pub enum CoupledDiscreteError {
    Flow(TranslatedViscousError),
    UnsupportedSlice,
    InvalidForce,
    UnsupportedForceRegion,
    ForceLimit { provided: usize, maximum: usize },
    GeometryPathLimit,
    ConstraintFailure,
    LinearFailure,
    IterationLimit,
    QuadratureFailure,
    SignResolution { roots: usize, calls: usize },
    QuadratureRefinement { error: f64 },
    ConservationFailure,
    WorkFailure,
    ThirdConservationFailure,
    ThirdWorkFailure,
    Cancelled { stage: CoupledDiscreteStage },
}
impl From<TranslatedViscousError> for CoupledDiscreteError {
    fn from(e: TranslatedViscousError) -> Self {
        Self::Flow(e)
    }
}
impl From<FittedHeightError> for CoupledDiscreteError {
    fn from(e: FittedHeightError) -> Self {
        Self::Flow(e.into())
    }
}
impl std::fmt::Display for CoupledDiscreteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "coupled discrete step rejected: {self:?}")
    }
}
impl std::error::Error for CoupledDiscreteError {}
fn barrier(
    c: &mut impl FnMut(CoupledDiscreteStage) -> bool,
    s: CoupledDiscreteStage,
) -> Result<(), CoupledDiscreteError> {
    if c(s) {
        Err(CoupledDiscreteError::Cancelled { stage: s })
    } else {
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct Point {
    q: [f64; 3],
    eta: [f64; 6],
    z: [f64; V],
    u: [[f64; 3]; N],
    mass: [f64; N],
    d: [[f64; V]; T],
    ddot: [[f64; V]; T],
    instantaneous_force: [[f64; 3]; N],
    b: [[f64; V]; P],
    force: [[f64; 3]; N],
    flux: [f64; F],
    pairs: [[usize; 2]; F],
    faces: usize,
    strain_power: f64,
    pressure_work: f64,
    full_constraints: f64,
}
impl Default for Point {
    fn default() -> Self {
        Self {
            q: [0.; 3],
            eta: [0.; 6],
            z: [0.; V],
            u: [[0.; 3]; N],
            mass: [0.; N],
            d: [[0.; V]; T],
            ddot: [[0.; V]; T],
            instantaneous_force: [[0.; 3]; N],
            b: [[0.; V]; P],
            force: [[0.; 3]; N],
            flux: [0.; F],
            pairs: [[0; 2]; F],
            faces: 0,
            strain_power: 0.,
            pressure_work: 0.,
            full_constraints: 0.,
        }
    }
}
#[derive(Clone, Copy)]
struct Integrals {
    plus: [f64; F],
    minus: [f64; F],
    max_constraints: f64,
    physical_convection: [f64; V],
}
impl Default for Integrals {
    fn default() -> Self {
        Self {
            plus: [0.; F],
            minus: [0.; F],
            max_constraints: 0.,
            physical_convection: [0.; V],
        }
    }
}
#[derive(Clone, Copy)]
struct Equation {
    rate: [f64; V],
    direct_rate: [f64; V],
    start: Point,
    end: Point,
    integral: Integrals,
    endpoint_convection: [f64; V],
    sign_roots: usize,
}
struct Work {
    geometry: FittedHeightWorkspace,
    r: [[[f64; V]; 2]; N],
    diagnostic: [FittedHeightNodeDiagnostic; N],
    pressure: [f64; P],
}
/// Borrowed common accepted frame/velocity/pressure/clock. Liquid volume is the
/// fitted mass divided by fixed density, with no separately advancing phase.
pub struct CoupledDiscreteState<'a> {
    pub geometry: &'a FittedHeightWorkspace,
    pub velocity: &'a [[f64; 3]],
    pub pressure_coefficients: &'a [f64; P],
    pub time: f64,
    pub stamp: TranslatedViscousStamp,
}
#[derive(Debug, Clone, Copy)]
pub struct CoupledDiscreteReport {
    pub before: TranslatedViscousStamp,
    pub after: TranslatedViscousStamp,
    pub dt: f64,
    pub time_after: f64,
    pub unknowns: [f64; V],
    pub end_q: [f64; 3],
    pub end_eta: [f64; 6],
    pub energy_before: f64,
    pub energy_after: f64,
    pub backward_euler_loss: f64,
    pub mixing_loss: f64,
    pub viscous_loss: f64,
    pub pressure_work: f64,
    pub gcl_work: f64,
    pub residual_work: f64,
    pub ledger_error: f64,
    pub work_allowance: f64,
    pub finite_momentum_rate_norm: f64,
    pub direct_momentum_rate_norm: f64,
    pub gcl_max: f64,
    pub quadrature_error: f64,
    pub full_constraints: f64,
    /// Loop passes, including a converged seed check. A terminal residual
    /// validation adds no pass or correction; see equation_evaluations.
    pub iterations: usize,
    pub equation_evaluations: usize,
    pub sign_roots: usize,
    pub mass_before: f64,
    pub mass_after: f64,
    pub endpoint_vs_path_momentum_max: f64,
}
/// Third component on the same periodic extrusion, observed from actual old/new
/// nodal velocity. Both engineering shears have weight mu, not 2 mu.
#[derive(Debug, Clone, Copy, Default)]
pub struct CoupledThirdReport {
    pub coefficients: [f64; W],
    pub momentum_before: f64,
    pub momentum_after: f64,
    pub energy_before: f64,
    pub energy_after: f64,
    pub backward_euler_loss: f64,
    pub mixing_loss: f64,
    pub shear_x_loss: f64,
    pub shear_y_loss: f64,
    pub viscous_loss: f64,
    pub gcl_work: f64,
    pub residual_work: f64,
    pub ledger_error: f64,
    pub work_allowance: f64,
    pub finite_momentum_rate_norm: f64,
    pub direct_momentum_rate_norm: f64,
}
/// Full-vector work on the moving, doubly periodic, z-invariant extrusion.
/// Pressure/geometry are the unchanged xy model; this is not general 3D flow.
#[derive(Debug, Clone, Copy)]
pub struct CoupledExtrudedReport {
    pub planar: CoupledDiscreteReport,
    pub third: CoupledThirdReport,
    pub energy_before: f64,
    pub energy_after: f64,
    pub backward_euler_loss: f64,
    pub mixing_loss: f64,
    pub viscous_loss: f64,
    pub gcl_work: f64,
    pub residual_work: f64,
    pub ledger_error: f64,
    pub work_allowance: f64,
}
/// Signed external work for the accepted-mass, endpoint-velocity force rule.
/// This is part of the complete coupled work budget, not an explicit kick.
#[derive(Debug, Clone, Copy, Default)]
pub struct CoupledForceReport {
    pub acceleration: [f64; 3],
    pub force_count: usize,
    pub planar_work: f64,
    pub third_work: f64,
    pub total_work: f64,
    pub horizontal_impulse: f64,
    pub third_impulse: f64,
}
#[derive(Debug, Clone, Copy)]
pub struct CoupledForcedExtrudedReport {
    pub step: CoupledExtrudedReport,
    pub forces: CoupledForceReport,
}
/// Small qualified topology only. Reuses the existing accepted-state owner and
/// one candidate frame. Arrays and stack scratch are fixed and budgeted; steps
/// allocate no heap storage and retain no accepted-state history.
pub struct CoupledDiscreteFlow {
    flow: TranslatedViscousFlow,
    work: Work,
    pressure: [f64; P],
    allocated_bytes: usize,
}
// Conservative sum of every simultaneously live caller/callee scratch object,
// including finite-difference copies, quadrature/root points and linear factors.
const STEP_STACK_BYTES: usize = 4 * std::mem::size_of::<[[f64; 38]; 38]>()
    + 6 * std::mem::size_of::<[f64; 38]>()
    + 6 * std::mem::size_of::<Equation>()
    + 8 * std::mem::size_of::<Point>()
    + 4 * std::mem::size_of::<Integrals>()
    + 6 * std::mem::size_of::<[[f64; V]; V]>()
    + 12 * std::mem::size_of::<[f64; V]>()
    + 4 * std::mem::size_of::<[[f64; 15]; 15]>()
    + 8 * std::mem::size_of::<[f64; 66]>();
// Additional conservative fixed caller/callee reservation for the opt-in third
// block. Existing scalar vectors are reused; no third accepted owner is added.
const THIRD_STACK_BYTES: usize = 8 * std::mem::size_of::<[[f64; W]; W]>()
    + 8 * std::mem::size_of::<[f64; W]>()
    + 4 * std::mem::size_of::<[[f64; W]; N]>()
    + 4 * std::mem::size_of::<[[f64; 3]; N]>()
    + 4 * std::mem::size_of::<CoupledExtrudedReport>();
const FORCE_STACK_BYTES: usize = 8 * std::mem::size_of::<[f64; 3]>()
    + 4 * std::mem::size_of::<CoupledForceReport>()
    + 4 * std::mem::size_of::<CoupledForcedExtrudedReport>();
const MAX_FORCES: usize = 16;
impl CoupledDiscreteFlow {
    pub fn nominal_bytes() -> Result<usize, CoupledDiscreteError> {
        TranslatedViscousFlow::nominal_bytes(2)?
            .checked_add(FittedHeightPlan::new(2)?.nominal_bytes)
            .and_then(|x| x.checked_add(std::mem::size_of::<Self>()))
            .and_then(|x| x.checked_add(STEP_STACK_BYTES))
            .ok_or(TranslatedViscousError::AllocationFailed.into())
    }
    pub fn new(
        geometry: FittedHeightGeometry<'_>,
        velocity: &[[f64; 3]],
        fitted: FittedHeightSettings,
        settings: TranslatedViscousSettings,
        id: u64,
    ) -> Result<Self, CoupledDiscreteError> {
        Self::initialize(geometry, velocity, fitted, settings, id, false, false)
    }
    /// Same two-column graph with x/z periods one and every field independent
    /// of z. Bottom is impermeable/free-slip; cap traction is weakly natural.
    /// Nonzero third velocity is permitted by periodic z, without end walls.
    pub fn new_extruded(
        geometry: FittedHeightGeometry<'_>,
        velocity: &[[f64; 3]],
        fitted: FittedHeightSettings,
        settings: TranslatedViscousSettings,
        id: u64,
    ) -> Result<Self, CoupledDiscreteError> {
        Self::initialize(geometry, velocity, fitted, settings, id, true, false)
    }
    pub fn nominal_extruded_bytes() -> Result<usize, CoupledDiscreteError> {
        Self::nominal_bytes()?
            .checked_add(THIRD_STACK_BYTES)
            .ok_or(TranslatedViscousError::AllocationFailed.into())
    }
    /// Existing accepted owner, with a fixed reservation for interval forcing.
    /// Forces are borrowed per call; geometry/material restrictions are unchanged.
    pub fn new_forced_extruded(
        geometry: FittedHeightGeometry<'_>,
        velocity: &[[f64; 3]],
        fitted: FittedHeightSettings,
        settings: TranslatedViscousSettings,
        id: u64,
    ) -> Result<Self, CoupledDiscreteError> {
        Self::initialize(geometry, velocity, fitted, settings, id, true, true)
    }
    pub fn nominal_forced_extruded_bytes() -> Result<usize, CoupledDiscreteError> {
        Self::nominal_extruded_bytes()?
            .checked_add(FORCE_STACK_BYTES)
            .ok_or(TranslatedViscousError::AllocationFailed.into())
    }
    #[allow(clippy::too_many_arguments)]
    fn initialize(
        geometry: FittedHeightGeometry<'_>,
        velocity: &[[f64; 3]],
        fitted: FittedHeightSettings,
        settings: TranslatedViscousSettings,
        id: u64,
        extruded: bool,
        forced: bool,
    ) -> Result<Self, CoupledDiscreteError> {
        if geometry.cap.len() != 3
            || geometry.bottom_x != BOTTOM
            || geometry.extrusion_width != 1.
            || geometry.density != 3.
            || geometry.dynamic_viscosity != 0.05
            || geometry.cap[0][1] + geometry.cap[1][1] != 2.25
            || velocity.len() != N
            || (!extruded && velocity.iter().any(|u| u[2] != 0.))
        {
            return Err(CoupledDiscreteError::UnsupportedSlice);
        }
        let default = TranslatedViscousSettings::default();
        if settings.relative_residual != default.relative_residual
            || settings.absolute_residual != default.absolute_residual
            || settings.momentum_limit != default.momentum_limit
            || settings.divergence_limit != default.divergence_limit
        {
            return Err(CoupledDiscreteError::UnsupportedSlice);
        }
        let extra = (if extruded { THIRD_STACK_BYTES } else { 0 })
            + if forced { FORCE_STACK_BYTES } else { 0 };
        let required = Self::nominal_bytes()?
            .checked_add(extra)
            .ok_or(TranslatedViscousError::AllocationFailed)?;
        if required > settings.memory_limit {
            return Err(TranslatedViscousError::BufferLimit {
                required,
                limit: settings.memory_limit,
            }
            .into());
        }
        let flow = if extruded {
            TranslatedViscousFlow::new_coupled_extruded(geometry, velocity, fitted, settings, id)?
        } else {
            TranslatedViscousFlow::new_coupled_xy(geometry, velocity, fitted, settings, id)?
        };
        let frame = FittedHeightWorkspace::new(geometry, fitted, |_| false)?;
        let mut work = Work {
            geometry: frame,
            r: [[[0.; V]; 2]; N],
            diagnostic: [Default::default(); N],
            pressure: [0.; P],
        };
        for i in 0..N {
            for d in 0..2 {
                let e = work
                    .geometry
                    .velocity_embedding(i, d)
                    .ok_or(CoupledDiscreteError::UnsupportedSlice)?;
                for k in 0..2 {
                    if let Some(j) = e.columns[k] {
                        if j >= V {
                            return Err(CoupledDiscreteError::UnsupportedSlice);
                        }
                        work.r[i][d][j] = add(work.r[i][d][j], e.weights[k])?;
                    }
                }
            }
        }
        if work.r[2][0] != unit(2)
            || work.r[3][0] != unit(3)
            || work.r[2][1] != unit(12)
            || work.r[3][1] != unit(13)
        {
            return Err(CoupledDiscreteError::UnsupportedSlice);
        }
        let accepted = flow.state();
        let (q, eta) = coordinates(accepted.template, accepted.velocity, &work.r)?;
        let zero = [0.; V];
        let initial = work.point(q, eta, &zero, 0., &mut |_| false)?;
        for i in 0..N {
            for d in 0..2 {
                if add(initial.u[i][d], -velocity[i][d])?.abs() > LIMIT {
                    return Err(CoupledDiscreteError::ConstraintFailure);
                }
            }
        }
        let allocated_bytes = flow
            .allocated_bytes()
            .checked_add(work.geometry.allocated_bytes())
            .and_then(|x| x.checked_add(std::mem::size_of::<Self>()))
            .and_then(|x| x.checked_add(STEP_STACK_BYTES))
            .and_then(|x| x.checked_add(extra))
            .ok_or(TranslatedViscousError::AllocationFailed)?;
        if allocated_bytes > settings.memory_limit {
            return Err(TranslatedViscousError::BufferLimit {
                required: allocated_bytes,
                limit: settings.memory_limit,
            }
            .into());
        }
        let accepted_z = coefficients(velocity, &work.r)?;
        for row in &initial.d {
            if dot(row, &accepted_z)?.abs() > LIMIT {
                return Err(CoupledDiscreteError::ConstraintFailure);
            }
        }
        let initial_unknown = work.seed(q, eta, [0.; 3], &mut |_| false)?;
        let pressure: [f64; P] = initial_unknown[6..]
            .try_into()
            .map_err(|_| CoupledDiscreteError::UnsupportedSlice)?;
        let report = work.geometry.inspect(
            FittedHeightInputs {
                velocity,
                pressure_coefficients: &pressure,
            },
            &mut work.diagnostic,
            |_| false,
        )?;
        if report.divergence_max > LIMIT {
            return Err(CoupledDiscreteError::ConstraintFailure);
        }
        Ok(Self {
            flow,
            work,
            pressure,
            allocated_bytes,
        })
    }
    pub fn state(&self) -> CoupledDiscreteState<'_> {
        let s = self.flow.state();
        CoupledDiscreteState {
            geometry: s.template,
            velocity: s.velocity,
            pressure_coefficients: &self.pressure,
            time: s.time,
            stamp: s.stamp,
        }
    }
    pub fn allocated_bytes(&self) -> usize {
        self.allocated_bytes
    }
    pub fn step(
        &mut self,
        h: f64,
        cancel: impl FnMut(CoupledDiscreteStage) -> bool,
    ) -> Result<CoupledDiscreteReport, CoupledDiscreteError> {
        if self.flow.state().velocity.iter().any(|u| u[2] != 0.) {
            return Err(CoupledDiscreteError::UnsupportedSlice);
        }
        self.advance(h, false, [0.; 3], None, cancel)
            .map(|r| r.planar)
    }
    /// Advance the explicitly periodic z-invariant three-component model.
    /// All arithmetic, third/total work and candidate inspection precede the
    /// existing common geometry/velocity/pressure/clock publication barrier.
    pub fn step_extruded(
        &mut self,
        h: f64,
        cancel: impl FnMut(CoupledDiscreteStage) -> bool,
    ) -> Result<CoupledExtrudedReport, CoupledDiscreteError> {
        if self.allocated_bytes < Self::nominal_extruded_bytes()? {
            return Err(CoupledDiscreteError::UnsupportedSlice);
        }
        self.advance(h, true, [0.; 3], None, cancel)
    }
    /// At most sixteen whole-domain, constant vectors for this interval.
    /// Acceleration is m/s²; force density is N/m³ divided by rho=3. Regions
    /// are refused. Sources use accepted masses; work tests endpoint velocity.
    pub fn step_extruded_with_forces(
        &mut self,
        h: f64,
        forces: &[BodyForce],
        mut cancel: impl FnMut(CoupledDiscreteStage) -> bool,
    ) -> Result<CoupledForcedExtrudedReport, CoupledDiscreteError> {
        if self.allocated_bytes < Self::nominal_forced_extruded_bytes()? {
            return Err(CoupledDiscreteError::UnsupportedSlice);
        }
        if forces.len() > MAX_FORCES {
            return Err(CoupledDiscreteError::ForceLimit {
                provided: forces.len(),
                maximum: MAX_FORCES,
            });
        }
        barrier(&mut cancel, CoupledDiscreteStage::BeforeForceInputs)?;
        let mut acceleration = [0.; 3];
        for force in forces {
            barrier(&mut cancel, CoupledDiscreteStage::ForceInput)?;
            if force.region.is_some() {
                return Err(CoupledDiscreteError::UnsupportedForceRegion);
            }
            for d in 0..3 {
                if force.value[d] != 0. && !force.value[d].is_normal() {
                    return Err(CoupledDiscreteError::InvalidForce);
                }
                let a = match force.units {
                    ForceUnits::Acceleration => force.value[d],
                    ForceUnits::ForceDensity => div(force.value[d], 3.)?,
                };
                acceleration[d] = add(acceleration[d], a)?;
            }
        }
        let mut report = CoupledForceReport {
            acceleration,
            force_count: forces.len(),
            ..Default::default()
        };
        let step = self.advance(h, true, acceleration, Some(&mut report), cancel)?;
        // All calculations and checks were completed before publication.
        Ok(CoupledForcedExtrudedReport {
            step,
            forces: report,
        })
    }
    fn advance(
        &mut self,
        h: f64,
        extruded: bool,
        acceleration: [f64; 3],
        forcing: Option<&mut CoupledForceReport>,
        mut cancel: impl FnMut(CoupledDiscreteStage) -> bool,
    ) -> Result<CoupledExtrudedReport, CoupledDiscreteError> {
        if !h.is_normal() || h <= 0. || h > 0.05 {
            return Err(TranslatedViscousError::InvalidSettings.into());
        }
        barrier(&mut cancel, CoupledDiscreteStage::BeforeGeometry)?;
        let accepted = self.flow.state();
        let before = accepted.stamp;
        let time_after = add(accepted.time, h)?;
        if time_after <= accepted.time {
            return Err(TranslatedViscousError::TimeResolution.into());
        }
        let after = TranslatedViscousStamp {
            id: before.id,
            version: before
                .version
                .checked_add(1)
                .ok_or(TranslatedViscousError::VersionOverflow)?,
        };
        let (q, eta) = coordinates(accepted.template, accepted.velocity, &self.work.r)?;
        let old_mass: [f64; N] = accepted
            .template
            .nodal_mass()
            .try_into()
            .map_err(|_| CoupledDiscreteError::UnsupportedSlice)?;
        // Borrow accepted velocity throughout the candidate solve. No rollback copy.
        let old_velocity = accepted.velocity;
        barrier(&mut cancel, CoupledDiscreteStage::BeforeSolve)?;
        let mut unknown = self.work.seed(q, eta, acceleration, &mut cancel)?;
        let mut iterations = 0;
        let mut calls = 0;
        let mut converged = false;
        for iteration in 0..self.flow.settings().max_iterations.min(7) {
            barrier(&mut cancel, CoupledDiscreteStage::Iteration)?;
            let e = self.work.equation(
                q,
                eta,
                &unknown,
                h,
                16,
                old_velocity,
                &old_mass,
                acceleration,
                &mut cancel,
            )?;
            calls += 1;
            iterations = iteration + 1;
            // BEGIN RHEON REFUSAL DIAGNOSTIC
            #[cfg(rheon_newton_trace)]
            eprintln!(
                "{{\"event\":\"newton_check\",\"iteration\":{},\"calls\":{},\"h\":{:?},\"accepted_version\":{},\"accepted_time\":{:?},\"q\":{:?},\"eta\":{:?},\"unknown\":{:?},\"rate_norm\":{:?},\"rate\":{:?},\"direct_rate\":{:?},\"candidate_q\":{:?},\"candidate_planar_velocity\":{:?},\"candidate_mass\":{:?},\"newton_threshold\":{:?},\"iteration_limit\":{}}}",
                iterations,
                calls,
                h,
                before.version,
                accepted.time,
                q,
                eta,
                unknown,
                norm(&e.rate)?,
                e.rate,
                e.direct_rate,
                e.end.q,
                e.end.u,
                e.end.mass,
                NEWTON,
                self.flow.settings().max_iterations.min(7),
            );
            // END RHEON REFUSAL DIAGNOSTIC
            if norm(&e.rate)? <= NEWTON {
                converged = true;
                break;
            }
            let mut jacobian = [[0.; V]; V];
            for j in 0..6 {
                let delta = mul(1e-6, add(unknown[j].abs(), 0.01)?)?;
                let mut perturbed = unknown;
                perturbed[j] = add(perturbed[j], delta)?;
                let p = self.work.equation(
                    q,
                    eta,
                    &perturbed,
                    h,
                    16,
                    old_velocity,
                    &old_mass,
                    acceleration,
                    &mut cancel,
                )?;
                calls += 1;
                for i in 0..V {
                    jacobian[i][j] = div(add(p.rate[i], -e.rate[i])?, delta)?;
                }
            }
            // Pressure enters linearly through exactly assembled endpoint B^T.
            for i in 0..V {
                for p in 0..P {
                    jacobian[i][6 + p] = e.end.b[p][i];
                }
            }
            let correction = linear(jacobian, e.rate.map(|x| -x))?;
            for i in 0..V {
                unknown[i] = add(unknown[i], correction[i])?;
            }
            // BEGIN RHEON REFUSAL DIAGNOSTIC
            #[cfg(rheon_newton_trace)]
            eprintln!(
                "{{\"event\":\"correction\",\"iteration\":{},\"calls\":{},\"correction\":{:?},\"unknown_after\":{:?}}}",
                iterations, calls, correction, unknown,
            );
            // END RHEON REFUSAL DIAGNOSTIC
            if calls > 200 {
                // BEGIN RHEON REFUSAL DIAGNOSTIC
                #[cfg(rheon_newton_trace)]
                eprintln!(
                    "{{\"event\":\"refusal\",\"reason\":\"equation_call_budget\",\"checks\":{},\"calls\":{}}}",
                    iterations, calls,
                );
                // END RHEON REFUSAL DIAGNOSTIC
                return Err(CoupledDiscreteError::IterationLimit);
            }
        }
        // The budget bounds computed corrections. Validate the final authorized
        // correction before exhaustion, without another Jacobian or solve.
        if !converged {
            if calls >= 200 {
                return Err(CoupledDiscreteError::IterationLimit);
            }
            barrier(&mut cancel, CoupledDiscreteStage::Iteration)?;
            let terminal = self.work.equation(
                q,
                eta,
                &unknown,
                h,
                16,
                old_velocity,
                &old_mass,
                acceleration,
                &mut cancel,
            )?;
            calls += 1;
            converged = norm(&terminal.rate)? <= NEWTON;
            // BEGIN RHEON REFUSAL DIAGNOSTIC
            #[cfg(rheon_newton_trace)]
            eprintln!(
                "{{\"event\":\"terminal_validation\",\"corrections\":{},\"calls\":{},\"rate_norm\":{:?},\"newton_threshold\":{:?},\"converged\":{}}}",
                iterations,
                calls,
                norm(&terminal.rate)?,
                NEWTON,
                converged,
            );
            // END RHEON REFUSAL DIAGNOSTIC
        }
        if !converged {
            // BEGIN RHEON REFUSAL DIAGNOSTIC
            #[cfg(rheon_newton_trace)]
            {
                eprintln!(
                    "{{\"event\":\"refusal\",\"reason\":\"correction_budget_exhausted\",\"checks\":{},\"calls\":{},\"accepted_version\":{},\"h\":{:?},\"newton_threshold\":{:?}}}",
                    iterations, calls, before.version, h, NEWTON,
                );
            }
            // END RHEON REFUSAL DIAGNOSTIC
            return Err(CoupledDiscreteError::IterationLimit);
        }
        barrier(&mut cancel, CoupledDiscreteStage::BeforeAcceptance)?;
        let coarse = self.work.equation(
            q,
            eta,
            &unknown,
            h,
            16,
            old_velocity,
            &old_mass,
            acceleration,
            &mut cancel,
        )?;
        let fine = self.work.equation(
            q,
            eta,
            &unknown,
            h,
            32,
            old_velocity,
            &old_mass,
            acceleration,
            &mut cancel,
        )?;
        let report = qualify(
            &fine,
            &coarse,
            h,
            unknown,
            before,
            after,
            time_after,
            iterations,
            calls,
            acceleration,
        )?;
        // Reassemble/inspect endpoint last, so published geometry and diagnostics
        // belong to the accepted velocity/pressure rather than a quadrature point.
        let endpoint = self.work.point(q, eta, &unknown, h, &mut cancel)?;
        let scratch = self.flow.ale_scratch();
        scratch.candidate.copy_from_slice(&endpoint.u);
        let (third, third_force_work) = if extruded {
            third_step(
                &mut self.work.geometry,
                &mut self.work.diagnostic,
                &unknown[6..],
                scratch,
                &fine,
                h,
                acceleration[2],
                &mut cancel,
            )?
        } else {
            (CoupledThirdReport::default(), 0.)
        };
        let planar_force_work = force_work(&old_mass, &endpoint.u, h, acceleration)?;
        let total_force_work = add(planar_force_work, third_force_work)?;
        let complete = combined_report(report, third, total_force_work)?;
        if let Some(forcing) = forcing {
            forcing.planar_work = planar_force_work;
            forcing.third_work = third_force_work;
            forcing.total_work = total_force_work;
            forcing.horizontal_impulse = mul(mul(h, report.mass_before)?, acceleration[0])?;
            forcing.third_impulse = mul(mul(h, report.mass_before)?, acceleration[2])?;
        }
        let candidate_pressure: [f64; P] = unknown[6..]
            .try_into()
            .map_err(|_| CoupledDiscreteError::UnsupportedSlice)?;
        barrier(&mut cancel, CoupledDiscreteStage::BeforePublish)?;
        self.flow
            .accept_ale(&mut self.work.geometry, time_after, 0., after);
        self.pressure = candidate_pressure;
        Ok(complete)
    }
}
#[allow(clippy::too_many_arguments)]
fn third_step(
    geometry: &mut FittedHeightWorkspace,
    diagnostic: &mut [FittedHeightNodeDiagnostic; N],
    pressure: &[f64],
    scratch: crate::translated_viscous::TranslatedViscousScratch<'_>,
    e: &Equation,
    h: f64,
    acceleration: f64,
    cancel: &mut impl FnMut(CoupledDiscreteStage) -> bool,
) -> Result<(CoupledThirdReport, f64), CoupledDiscreteError> {
    barrier(cancel, CoupledDiscreteStage::BeforeThirdSolve)?;
    let mut r = [[0.; W]; N];
    for i in 0..N {
        let row = geometry.velocity_embedding(i, 2).unwrap();
        let old_row = scratch.frame.velocity_embedding(i, 2).unwrap();
        if row.columns != old_row.columns || row.weights != old_row.weights {
            return Err(CoupledDiscreteError::UnsupportedSlice);
        }
        for k in 0..2 {
            if let Some(j) = row.columns[k] {
                let j = j
                    .checked_sub(V)
                    .filter(|&j| j < W)
                    .ok_or(CoupledDiscreteError::UnsupportedSlice)?;
                r[i][j] = add(r[i][j], row.weights[k])?;
            }
        }
        let sum = r[i].iter().try_fold(0., |sum, &x| add(sum, x))?;
        if add(sum, -1.)? != 0. {
            return Err(CoupledDiscreteError::UnsupportedSlice);
        }
    }
    let old_xi: [f64; W] = std::array::from_fn(|j| scratch.accepted[scratch.free_nodes[j]][2]);
    for i in 0..N {
        if add(dot(&r[i], &old_xi)?, -scratch.accepted[i][2])?.abs() > LIMIT {
            return Err(CoupledDiscreteError::ConstraintFailure);
        }
    }
    let mut stiffness = [[0.; W]; W];
    for (t, tri) in geometry.triangles().iter().enumerate() {
        barrier(cancel, CoupledDiscreteStage::ThirdAssembly)?;
        let k = geometry.triangle_stiffness(t)?;
        let ids = tri.nodes.map(|i| geometry.nodes()[i].periodic_index);
        for a in 0..3 {
            for b in 0..3 {
                for j in 0..W {
                    for l in 0..W {
                        stiffness[j][l] = add(
                            stiffness[j][l],
                            mul(mul(k[3 * a + 2][3 * b + 2], r[ids[a]][j])?, r[ids[b]][l])?,
                        )?;
                    }
                }
            }
        }
    }
    let mut matrix = [[0.; W]; W];
    scratch.rhs.fill(0.);
    for i in 0..N {
        for j in 0..W {
            scratch.rhs[j] = add(
                scratch.rhs[j],
                mul(r[i][j], mul(e.start.mass[i], scratch.accepted[i][2])?)?,
            )?;
            if acceleration != 0. {
                scratch.rhs[j] = add(
                    scratch.rhs[j],
                    mul(r[i][j], mul(mul(h, e.start.mass[i])?, acceleration)?)?,
                )?;
            }
            for k in 0..W {
                matrix[j][k] = add(matrix[j][k], mul(e.end.mass[i], mul(r[i][j], r[i][k])?)?)?;
            }
        }
    }
    for f in 0..e.end.faces {
        let [a, b] = e.end.pairs[f];
        for j in 0..W {
            let difference = add(r[a][j], -r[b][j])?;
            for k in 0..W {
                let donor = add(
                    mul(e.integral.plus[f], r[a][k])?,
                    -mul(e.integral.minus[f], r[b][k])?,
                )?;
                matrix[j][k] = add(matrix[j][k], mul(difference, donor)?)?;
            }
        }
    }
    for j in 0..W {
        for k in 0..W {
            matrix[j][k] = add(matrix[j][k], mul(h, stiffness[j][k])?)?;
        }
    }
    let rhs: [f64; W] = scratch
        .rhs
        .try_into()
        .map_err(|_| CoupledDiscreteError::UnsupportedSlice)?;
    let xi = linear(matrix, rhs)?;
    scratch.solution.copy_from_slice(&xi);
    for i in 0..N {
        scratch.candidate[i][2] = dot(&r[i], &xi)?;
    }
    // Inspect the actual full candidate with the same physical operators. This
    // leaves the published frame's diagnostics consistent with all components.
    let inspected = geometry.inspect(
        FittedHeightInputs {
            velocity: scratch.candidate,
            pressure_coefficients: pressure,
        },
        diagnostic,
        |_| false,
    )?;
    let mut out = CoupledThirdReport {
        coefficients: xi,
        ..Default::default()
    };
    let mut gcl = [0.; N];
    scratch.force.fill(0.);
    for i in 0..N {
        gcl[i] = add(e.end.mass[i], -e.start.mass[i])?;
        let w0 = scratch.accepted[i][2];
        let w1 = scratch.candidate[i][2];
        out.momentum_before = add(out.momentum_before, mul(e.start.mass[i], w0)?)?;
        out.momentum_after = add(out.momentum_after, mul(e.end.mass[i], w1)?)?;
        out.energy_before = add(
            out.energy_before,
            mul(mul(0.5, e.start.mass[i])?, mul(w0, w0)?)?,
        )?;
        out.energy_after = add(
            out.energy_after,
            mul(mul(0.5, e.end.mass[i])?, mul(w1, w1)?)?,
        )?;
        let dw = add(w1, -w0)?;
        out.backward_euler_loss = add(
            out.backward_euler_loss,
            mul(mul(0.5, e.start.mass[i])?, mul(dw, dw)?)?,
        )?;
    }
    for f in 0..e.end.faces {
        let [i, j] = e.end.pairs[f];
        let wi = scratch.candidate[i][2];
        let wj = scratch.candidate[j][2];
        let transported = add(mul(e.integral.plus[f], wi)?, -mul(e.integral.minus[f], wj)?)?;
        scratch.force[i] = add(scratch.force[i], transported)?;
        scratch.force[j] = add(scratch.force[j], -transported)?;
        let net = add(e.integral.plus[f], -e.integral.minus[f])?;
        gcl[i] = add(gcl[i], net)?;
        gcl[j] = add(gcl[j], -net)?;
        let dw = add(wi, -wj)?;
        out.mixing_loss = add(
            out.mixing_loss,
            mul(
                mul(0.5, add(e.integral.plus[f], e.integral.minus[f])?)?,
                mul(dw, dw)?,
            )?,
        )?;
    }
    let mut rate = [0.; W];
    let mut direct = [0.; W];
    let mut increment = [0.; W];
    for j in 0..W {
        increment[j] = add(xi[j], -old_xi[j])?;
    }
    for i in 0..N {
        let w0 = scratch.accepted[i][2];
        let w1 = scratch.candidate[i][2];
        let mut force = add(scratch.force[i], mul(h, diagnostic[i].strain_force[2])?)?;
        if acceleration != 0. {
            force = add(force, -mul(mul(h, e.start.mass[i])?, acceleration)?)?;
        }
        let stable = add(
            mul(e.end.mass[i], dot(&r[i], &increment)?)?,
            mul(add(e.end.mass[i], -e.start.mass[i])?, w0)?,
        )?;
        let direct_inertia = add(mul(e.end.mass[i], w1)?, -mul(e.start.mass[i], w0)?)?;
        for j in 0..W {
            rate[j] = add(rate[j], mul(r[i][j], add(stable, force)?)?)?;
            direct[j] = add(direct[j], mul(r[i][j], add(direct_inertia, force)?)?)?;
        }
        out.gcl_work = add(out.gcl_work, mul(mul(0.5, gcl[i])?, mul(w1, w1)?)?)?;
    }
    out.residual_work = dot(&xi, &rate)?;
    for j in 0..W {
        rate[j] = div(rate[j], h)?;
        direct[j] = div(direct[j], h)?;
    }
    out.finite_momentum_rate_norm = norm(&rate)?;
    out.direct_momentum_rate_norm = norm(&direct)?;
    // Local differences preserve the constant shear nullspace. These are the
    // two full engineering-strain entries already used in element assembly.
    for tri in geometry.triangles() {
        let ids = tri.nodes.map(|i| geometry.nodes()[i].periodic_index);
        let base = scratch.candidate[ids[0]][2];
        let mut gradient = [0.; 2];
        for a in 1..3 {
            for d in 0..2 {
                gradient[d] = add(
                    gradient[d],
                    mul(
                        tri.gradients[a][d],
                        add(scratch.candidate[ids[a]][2], -base)?,
                    )?,
                )?;
            }
        }
        let weight = mul(mul(h, 0.05)?, tri.area)?;
        out.shear_x_loss = add(
            out.shear_x_loss,
            mul(weight, mul(gradient[0], gradient[0])?)?,
        )?;
        out.shear_y_loss = add(
            out.shear_y_loss,
            mul(weight, mul(gradient[1], gradient[1])?)?,
        )?;
    }
    out.viscous_loss = add(out.shear_x_loss, out.shear_y_loss)?;
    out.ledger_error = add(
        add(
            add(
                add(
                    add(
                        add(out.energy_after, -out.energy_before)?,
                        out.backward_euler_loss,
                    )?,
                    out.mixing_loss,
                )?,
                out.viscous_loss,
            )?,
            out.gcl_work,
        )?,
        -out.residual_work,
    )?;
    let force_work = force_work(&e.start.mass, scratch.candidate, h, [0., 0., acceleration])?;
    if force_work != 0. {
        out.ledger_error = add(out.ledger_error, -force_work)?;
    }
    let mut scale = add(
        add(
            add(
                add(
                    add(
                        add(out.energy_before, out.energy_after)?,
                        out.backward_euler_loss,
                    )?,
                    out.mixing_loss,
                )?,
                out.viscous_loss,
            )?,
            out.gcl_work.abs(),
        )?,
        out.residual_work.abs(),
    )?;
    if force_work != 0. {
        scale = add(scale, force_work.abs())?;
    }
    out.work_allowance = mul(128. * f64::EPSILON, scale)?;
    let mut momentum_error = add(out.momentum_after, -out.momentum_before)?;
    if acceleration != 0. {
        let total_mass = e
            .start
            .mass
            .iter()
            .try_fold(0., |sum, &mass| add(sum, mass))?;
        momentum_error = add(momentum_error, -mul(mul(h, total_mass)?, acceleration)?)?;
    }
    if out.finite_momentum_rate_norm > LIMIT
        || out.direct_momentum_rate_norm > LIMIT
        || momentum_error.abs() > LIMIT
    {
        return Err(CoupledDiscreteError::ThirdConservationFailure);
    }
    if out.ledger_error.abs() > out.work_allowance
        || out.residual_work.abs() > out.work_allowance
        || out.backward_euler_loss < 0.
        || out.mixing_loss < 0.
        || out.viscous_loss < 0.
    {
        return Err(CoupledDiscreteError::ThirdWorkFailure);
    }
    let force_allowance = mul(
        128. * f64::EPSILON,
        add(e.end.strain_power, inspected.strain_power)?,
    )?;
    if add(
        mul(h, inspected.strain_power)?,
        -add(mul(h, e.end.strain_power)?, out.viscous_loss)?,
    )?
    .abs()
        > mul(h, force_allowance)?
        || inspected.divergence_max > LIMIT
    {
        return Err(CoupledDiscreteError::ThirdWorkFailure);
    }
    barrier(cancel, CoupledDiscreteStage::AfterThirdSolve)?;
    Ok((out, force_work))
}
fn combined_report(
    planar: CoupledDiscreteReport,
    third: CoupledThirdReport,
    force_work: f64,
) -> Result<CoupledExtrudedReport, CoupledDiscreteError> {
    let mut r = CoupledExtrudedReport {
        planar,
        third,
        energy_before: add(planar.energy_before, third.energy_before)?,
        energy_after: add(planar.energy_after, third.energy_after)?,
        backward_euler_loss: add(planar.backward_euler_loss, third.backward_euler_loss)?,
        mixing_loss: add(planar.mixing_loss, third.mixing_loss)?,
        viscous_loss: add(planar.viscous_loss, third.viscous_loss)?,
        gcl_work: add(planar.gcl_work, third.gcl_work)?,
        residual_work: add(planar.residual_work, third.residual_work)?,
        ledger_error: 0.,
        work_allowance: 0.,
    };
    r.ledger_error = add(
        add(
            add(
                add(
                    add(
                        add(
                            add(r.energy_after, -r.energy_before)?,
                            r.backward_euler_loss,
                        )?,
                        r.mixing_loss,
                    )?,
                    r.viscous_loss,
                )?,
                planar.pressure_work,
            )?,
            r.gcl_work,
        )?,
        -r.residual_work,
    )?;
    if force_work != 0. {
        r.ledger_error = add(r.ledger_error, -force_work)?;
    }
    let mut scale = add(
        add(
            add(
                add(
                    add(
                        add(add(r.energy_before, r.energy_after)?, r.backward_euler_loss)?,
                        r.mixing_loss,
                    )?,
                    r.viscous_loss,
                )?,
                planar.pressure_work.abs(),
            )?,
            r.gcl_work.abs(),
        )?,
        r.residual_work.abs(),
    )?;
    if force_work != 0. {
        scale = add(scale, force_work.abs())?;
    }
    r.work_allowance = mul(128. * f64::EPSILON, scale)?;
    if r.residual_work.abs() > r.work_allowance || r.ledger_error.abs() > r.work_allowance {
        return Err(CoupledDiscreteError::ThirdWorkFailure);
    }
    if norm(&[
        planar.finite_momentum_rate_norm,
        third.finite_momentum_rate_norm,
    ])? > LIMIT
        || norm(&[
            planar.direct_momentum_rate_norm,
            third.direct_momentum_rate_norm,
        ])? > LIMIT
    {
        return Err(CoupledDiscreteError::ThirdConservationFailure);
    }
    Ok(r)
}
fn unit(i: usize) -> [f64; V] {
    let mut z = [0.; V];
    z[i] = 1.;
    z
}
fn coordinates(
    frame: &FittedHeightWorkspace,
    u: &[[f64; 3]],
    r: &[[[f64; V]; 2]; N],
) -> Result<([f64; 3], [f64; 6]), CoupledDiscreteError> {
    let q = [
        frame.nodes()[3].position[0],
        frame.nodes()[4].position[0],
        frame.nodes()[3].position[1],
    ];
    let z = coefficients(u, r)?;
    Ok((q, [z[2], z[3], z[12], z[0], z[1], z[4]]))
}
fn coefficients(u: &[[f64; 3]], r: &[[[f64; V]; 2]; N]) -> Result<[f64; V], CoupledDiscreteError> {
    let mut z = [0.; V];
    for j in 0..V {
        let mut found = false;
        for i in 0..N {
            for d in 0..2 {
                if r[i][d] == unit(j) {
                    z[j] = check(u[i][d])?;
                    found = true;
                    break;
                }
            }
            if found {
                break;
            }
        }
        if !found {
            return Err(CoupledDiscreteError::UnsupportedSlice);
        }
    }
    Ok(z)
}

fn cross(a: [f64; 2], b: [f64; 2]) -> Result<f64, CoupledDiscreteError> {
    Ok(add(mul(a[0], b[1])?, -mul(a[1], b[0])?)?)
}
fn linear<const L: usize>(
    mut a: [[f64; L]; L],
    mut rhs: [f64; L],
) -> Result<[f64; L], CoupledDiscreteError> {
    let original = a;
    let b = rhs;
    for j in 0..L {
        let pivot = (j..L)
            .max_by(|&i, &k| a[i][j].abs().total_cmp(&a[k][j].abs()))
            .ok_or(CoupledDiscreteError::LinearFailure)?;
        a.swap(j, pivot);
        rhs.swap(j, pivot);
        if !a[j][j].is_normal() {
            return Err(CoupledDiscreteError::LinearFailure);
        }
        for i in j + 1..L {
            let factor = div(a[i][j], a[j][j])?;
            for k in j + 1..L {
                a[i][k] = add(a[i][k], -mul(factor, a[j][k])?)?;
            }
            rhs[i] = add(rhs[i], -mul(factor, rhs[j])?)?;
            a[i][j] = 0.;
        }
    }
    let mut z = [0.; L];
    for i in (0..L).rev() {
        let mut tail = 0.;
        for j in i + 1..L {
            tail = add(tail, mul(a[i][j], z[j])?)?;
        }
        z[i] = div(add(rhs[i], -tail)?, a[i][i])?;
    }
    for i in 0..L {
        if add(dot(&original[i], &z)?, -b[i])?.abs() > LIMIT {
            return Err(CoupledDiscreteError::LinearFailure);
        }
    }
    Ok(z)
}
impl Work {
    fn seed(
        &mut self,
        q: [f64; 3],
        eta: [f64; 6],
        acceleration: [f64; 3],
        cancel: &mut impl FnMut(CoupledDiscreteStage) -> bool,
    ) -> Result<[f64; V], CoupledDiscreteError> {
        let p = self.point(q, eta, &[0.; V], 0., cancel)?;
        let mut matrix = [[0.; 38]; 38];
        let mut rhs = [0.; 38];
        for i in 0..V {
            for n in 0..N {
                for d in 0..2 {
                    rhs[i] = add(rhs[i], -mul(self.r[n][d][i], p.instantaneous_force[n][d])?)?;
                    if acceleration[d] != 0. {
                        rhs[i] = add(
                            rhs[i],
                            mul(self.r[n][d][i], mul(p.mass[n], acceleration[d])?)?,
                        )?;
                    }
                    for j in 0..V {
                        matrix[i][j] = add(
                            matrix[i][j],
                            mul(mul(p.mass[n], self.r[n][d][i])?, self.r[n][d][j])?,
                        )?;
                    }
                }
            }
            for k in 0..P {
                matrix[i][V + k] = p.b[k][i];
            }
        }
        let rows = [0, 2, 3, 4, 5, 6, 8, 10, 11, 12, 14, 16, 17, 18, 20, 22];
        for k in 0..P {
            matrix[V + k][..V].copy_from_slice(&p.d[rows[k]]);
            rhs[V + k] = -dot(&p.ddot[rows[k]], &p.z)?;
        }
        let solution = linear(matrix, rhs)?;
        let mut unknown = [0.; V];
        for i in 0..6 {
            unknown[i] = solution[KNOWN[i]];
        }
        unknown[6..].copy_from_slice(&solution[V..]);
        Ok(unknown)
    }
    fn point(
        &mut self,
        q0: [f64; 3],
        eta0: [f64; 6],
        unknown: &[f64; V],
        t: f64,
        cancel: &mut impl FnMut(CoupledDiscreteStage) -> bool,
    ) -> Result<Point, CoupledDiscreteError> {
        barrier(cancel, CoupledDiscreteStage::Quadrature)?;
        let mut p = Point::default();
        for i in 0..6 {
            p.eta[i] = add(eta0[i], mul(t, unknown[i])?)?;
        }
        for i in 0..3 {
            p.q[i] = add(
                add(q0[i], mul(t, eta0[i])?)?,
                mul(mul(mul(0.5, t)?, t)?, unknown[i])?,
            )?;
            if add(p.q[i], -q0[i])?.abs() > 0.125 {
                return Err(CoupledDiscreteError::GeometryPathLimit);
            }
        }
        let cap = [
            [p.q[0], p.q[2]],
            [p.q[1], add(2.25, -p.q[2])?],
            [add(p.q[0], 1.)?, p.q[2]],
        ];
        self.geometry.reassemble(&cap, &BOTTOM)?;
        for (i, row) in self.r.iter().enumerate() {
            for (d, component) in row.iter().enumerate() {
                let e = self
                    .geometry
                    .velocity_embedding(i, d)
                    .ok_or(CoupledDiscreteError::UnsupportedSlice)?;
                let mut actual = [0.; V];
                for k in 0..2 {
                    if let Some(j) = e.columns[k] {
                        if j >= V {
                            return Err(CoupledDiscreteError::UnsupportedSlice);
                        }
                        actual[j] = add(actual[j], e.weights[k])?;
                    }
                }
                if actual != *component {
                    return Err(CoupledDiscreteError::UnsupportedSlice);
                }
            }
        }
        for (k, &j) in KNOWN.iter().enumerate() {
            p.z[j] = if k == 6 {
                -p.eta[COORD[k]]
            } else {
                p.eta[COORD[k]]
            };
        }
        for (t, tri) in self.geometry.triangles().iter().enumerate() {
            for i in 0..3 {
                let node = self.geometry.nodes()[tri.nodes[i]].periodic_index;
                for j in 0..V {
                    for d in 0..2 {
                        p.d[t][j] = add(p.d[t][j], mul(tri.gradients[i][d], self.r[node][d][j])?)?;
                    }
                }
            }
        }
        let mut block = [[0.; 15]; 15];
        let mut rhs = [0.; 15];
        for i in 0..15 {
            for j in 0..15 {
                block[i][j] = p.d[ROWS[i]][UNKNOWN[j]];
            }
            rhs[i] = -dot(&p.d[ROWS[i]], &p.z)?;
        }
        let values = linear(block, rhs)?;
        for j in 0..15 {
            p.z[UNKNOWN[j]] = values[j];
        }
        let mut embedded = [0.; 34];
        embedded[..V].copy_from_slice(&p.z);
        self.geometry.embed_velocity(&embedded, &mut p.u)?;
        self.pressure.copy_from_slice(&unknown[6..]);
        let report = self.geometry.inspect(
            FittedHeightInputs {
                velocity: &p.u,
                pressure_coefficients: &self.pressure,
            },
            &mut self.diagnostic,
            |_| false,
        )?;
        p.strain_power = report.strain_power;
        p.pressure_work = report.pressure_work;
        p.mass.copy_from_slice(self.geometry.nodal_mass());
        for i in 0..N {
            for d in 0..3 {
                p.instantaneous_force[i][d] = add(
                    add(
                        mul(self.diagnostic[i].mass_rate, p.u[i][d])?,
                        self.diagnostic[i].convection[d],
                    )?,
                    self.diagnostic[i].strain_force[d],
                )?;
                p.force[i][d] = add(
                    self.diagnostic[i].strain_force[d],
                    self.diagnostic[i].pressure_force[d],
                )?;
            }
        }
        p.faces = self.geometry.shared_flux_scratch().len();
        if p.faces > F {
            return Err(CoupledDiscreteError::UnsupportedSlice);
        }
        for (i, f) in self.geometry.shared_flux_scratch().iter().enumerate() {
            p.flux[i] = f.flux;
            p.pairs[i] = f.nodes;
        }
        for term in self.geometry.pressure_basis() {
            for j in 0..V {
                p.b[term.mode][j] = add(
                    p.b[term.mode][j],
                    -mul(
                        mul(term.value, self.geometry.triangles()[term.triangle].area)?,
                        p.d[term.triangle][j],
                    )?,
                )?;
            }
        }
        // Differentiate all physical triangle gradients using actual PS motion.
        let mut derivative = [[0.; V]; T];
        for (t, tri) in self.geometry.triangles().iter().enumerate() {
            let x = tri.nodes.map(|i| self.geometry.nodes()[i].position);
            let w = tri
                .nodes
                .map(|i| self.geometry.node_motion_scratch(i).unwrap());
            let mut ab = [0.; 2];
            let mut ac = [0.; 2];
            let mut dab = [0.; 2];
            let mut dac = [0.; 2];
            for d in 0..2 {
                ab[d] = add(x[1][d], -x[0][d])?;
                ac[d] = add(x[2][d], -x[0][d])?;
                dab[d] = add(w[1][d], -w[0][d])?;
                dac[d] = add(w[2][d], -w[0][d])?;
            }
            let area2 = mul(2., tri.area)?;
            let area_rate = add(cross(dab, ac)?, cross(ab, dac)?)?;
            for i in 0..3 {
                let j = (i + 1) % 3;
                let k = (i + 2) % 3;
                let grad = [
                    div(
                        add(
                            add(w[j][1], -w[k][1])?,
                            -mul(tri.gradients[i][0], area_rate)?,
                        )?,
                        area2,
                    )?,
                    div(
                        add(
                            add(w[k][0], -w[j][0])?,
                            -mul(tri.gradients[i][1], area_rate)?,
                        )?,
                        area2,
                    )?,
                ];
                let n = self.geometry.nodes()[tri.nodes[i]].periodic_index;
                for v in 0..V {
                    for d in 0..2 {
                        derivative[t][v] = add(derivative[t][v], mul(grad[d], self.r[n][d][v])?)?;
                    }
                }
            }
        }
        p.ddot = derivative;
        let mut acceleration = [0.; V];
        for (k, &j) in KNOWN.iter().enumerate() {
            acceleration[j] = if k == 6 {
                -unknown[COORD[k]]
            } else {
                unknown[COORD[k]]
            };
        }
        for i in 0..15 {
            rhs[i] = -add(
                dot(&derivative[ROWS[i]], &p.z)?,
                dot(&p.d[ROWS[i]], &acceleration)?,
            )?;
        }
        let values = linear(block, rhs)?;
        for j in 0..15 {
            acceleration[UNKNOWN[j]] = values[j];
        }
        for t in 0..T {
            p.full_constraints = p
                .full_constraints
                .max(dot(&p.d[t], &p.z)?.abs())
                .max(add(dot(&p.d[t], &acceleration)?, dot(&derivative[t], &p.z)?)?.abs());
        }
        p.full_constraints = p
            .full_constraints
            .max(add(p.u[2][0], -p.eta[0])?.abs())
            .max(add(p.u[3][0], -p.eta[1])?.abs())
            .max(add(p.u[2][1], -p.eta[2])?.abs());
        if p.full_constraints > LIMIT {
            return Err(CoupledDiscreteError::ConstraintFailure);
        }
        Ok(p)
    }
    fn partition(
        &mut self,
        q: [f64; 3],
        eta: [f64; 6],
        u: &[f64; V],
        h: f64,
        cancel: &mut impl FnMut(CoupledDiscreteStage) -> bool,
    ) -> Result<([f64; 66], usize), CoupledDiscreteError> {
        let mut boundaries = [0.; 66];
        boundaries[1] = h;
        let mut count = 2;
        let mut root_calls = 0;
        let mut a = 0.;
        let mut previous = self.point(q, eta, u, 0., cancel)?;
        for k in 1..=16 {
            let b = mul(h, k as f64 / 16.)?;
            let next = self.point(q, eta, u, b, cancel)?;
            if previous.pairs != next.pairs || previous.faces != next.faces {
                return Err(CoupledDiscreteError::UnsupportedSlice);
            }
            for f in 0..next.faces {
                if previous.flux[f] != 0.
                    && next.flux[f] != 0.
                    && previous.flux[f].is_sign_negative() != next.flux[f].is_sign_negative()
                {
                    let mut left = a;
                    let mut right = b;
                    let sign = previous.flux[f].is_sign_negative();
                    for _ in 0..20 {
                        let middle = mul(add(left, right)?, 0.5)?;
                        if middle == left || middle == right {
                            break;
                        }
                        let value = self.point(q, eta, u, middle, cancel)?.flux[f];
                        root_calls += 1;
                        if root_calls > 4096 {
                            return Err(CoupledDiscreteError::SignResolution {
                                roots: count - 2,
                                calls: root_calls,
                            });
                        }
                        if value == 0. {
                            left = middle;
                            right = middle;
                            break;
                        }
                        if value.is_sign_negative() == sign {
                            left = middle;
                        } else {
                            right = middle;
                        }
                    }
                    let root = mul(add(left, right)?, 0.5)?;
                    if root > 0. && root < h {
                        if count == 66 {
                            return Err(CoupledDiscreteError::SignResolution {
                                roots: count - 2,
                                calls: root_calls,
                            });
                        }
                        boundaries[count] = root;
                        count += 1;
                    }
                }
            }
            a = b;
            previous = next;
        }
        boundaries[..count].sort_by(f64::total_cmp);
        let mut unique = 1;
        for i in 1..count {
            if boundaries[i] != boundaries[unique - 1] {
                boundaries[unique] = boundaries[i];
                unique += 1;
            }
        }
        Ok((boundaries, unique))
    }
    #[allow(clippy::too_many_arguments)]
    fn equation(
        &mut self,
        q: [f64; 3],
        eta: [f64; 6],
        unknown: &[f64; V],
        h: f64,
        order: usize,
        old: &[[f64; 3]],
        m0: &[f64; N],
        acceleration: [f64; 3],
        cancel: &mut impl FnMut(CoupledDiscreteStage) -> bool,
    ) -> Result<Equation, CoupledDiscreteError> {
        let mut start = self.point(q, eta, unknown, 0., cancel)?;
        let accepted_z = coefficients(old, &self.r)?;
        for i in 0..N {
            for d in 0..2 {
                if add(dot(&self.r[i][d], &accepted_z)?, -old[i][d])?.abs() > LIMIT {
                    return Err(CoupledDiscreteError::ConstraintFailure);
                }
            }
        }
        for row in &start.d {
            if dot(row, &accepted_z)?.abs() > LIMIT {
                return Err(CoupledDiscreteError::ConstraintFailure);
            }
        }
        start.z = accepted_z;
        for i in 0..N {
            start.u[i] = [old[i][0], old[i][1], 0.];
        }
        start.mass = *m0;
        let end = self.point(q, eta, unknown, h, cancel)?;
        let mut integral = Integrals {
            max_constraints: start.full_constraints.max(end.full_constraints),
            ..Default::default()
        };
        let (partition, count) = self.partition(q, eta, unknown, h, cancel)?;
        for segment in 0..count - 1 {
            let width = add(partition[segment + 1], -partition[segment])?;
            for k in 0..order {
                let (node, weight) = rule(order, k)?;
                let t = add(partition[segment], mul(mul(width, 0.5)?, add(1., node)?)?)?;
                let factor = mul(mul(width, 0.5)?, weight)?;
                let p = self.point(q, eta, unknown, t, cancel)?;
                if p.pairs != end.pairs || p.faces != end.faces {
                    return Err(CoupledDiscreteError::UnsupportedSlice);
                }
                integral.max_constraints = integral.max_constraints.max(p.full_constraints);
                for f in 0..p.faces {
                    integral.plus[f] = add(integral.plus[f], mul(factor, p.flux[f].max(0.))?)?;
                    integral.minus[f] = add(integral.minus[f], mul(factor, (-p.flux[f]).max(0.))?)?;
                    let [i, j] = p.pairs[f];
                    let donor = if p.flux[f] >= 0. { i } else { j };
                    for v in 0..V {
                        for d in 0..2 {
                            integral.physical_convection[v] = add(
                                integral.physical_convection[v],
                                mul(
                                    mul(mul(factor, p.flux[f])?, p.u[donor][d])?,
                                    add(self.r[i][d][v], -self.r[j][d][v])?,
                                )?,
                            )?;
                        }
                    }
                }
            }
        }
        let mut endpoint_convection = [0.; V];
        let mut rate = [0.; V];
        let mut direct_rate = [0.; V];
        for i in 0..N {
            for d in 0..2 {
                let mut dz = [0.; V];
                for j in 0..V {
                    dz[j] = add(end.z[j], -start.z[j])?;
                }
                let increment = dot(&self.r[i][d], &dz)?;
                let stable = add(
                    mul(end.mass[i], increment)?,
                    mul(add(end.mass[i], -m0[i])?, old[i][d])?,
                )?;
                let direct = add(mul(end.mass[i], end.u[i][d])?, -mul(m0[i], old[i][d])?)?;
                for v in 0..V {
                    let mut force = mul(h, end.force[i][d])?;
                    if acceleration[d] != 0. {
                        force = add(force, -mul(mul(h, m0[i])?, acceleration[d])?)?;
                    }
                    rate[v] = add(rate[v], mul(self.r[i][d][v], add(stable, force)?)?)?;
                    direct_rate[v] =
                        add(direct_rate[v], mul(self.r[i][d][v], add(direct, force)?)?)?;
                }
            }
        }
        for f in 0..end.faces {
            let [i, j] = end.pairs[f];
            for d in 0..2 {
                let transported = add(
                    mul(integral.plus[f], end.u[i][d])?,
                    -mul(integral.minus[f], end.u[j][d])?,
                )?;
                for v in 0..V {
                    let value = mul(add(self.r[i][d][v], -self.r[j][d][v])?, transported)?;
                    rate[v] = add(rate[v], value)?;
                    direct_rate[v] = add(direct_rate[v], value)?;
                    endpoint_convection[v] = add(endpoint_convection[v], value)?;
                }
            }
        }
        for i in 0..V {
            rate[i] = div(rate[i], h)?;
            direct_rate[i] = div(direct_rate[i], h)?;
        }
        Ok(Equation {
            rate,
            direct_rate,
            start,
            end,
            integral,
            endpoint_convection,
            sign_roots: count - 2,
        })
    }
}
fn energy(p: &Point) -> Result<f64, CoupledDiscreteError> {
    let mut e = 0.;
    for i in 0..N {
        e = add(e, mul(mul(0.5, p.mass[i])?, dot(&p.u[i], &p.u[i])?)?)?;
    }
    Ok(e)
}
fn force_work(
    mass: &[f64; N],
    velocity: &[[f64; 3]],
    h: f64,
    acceleration: [f64; 3],
) -> Result<f64, CoupledDiscreteError> {
    let mut work = 0.;
    for i in 0..N {
        for d in 0..3 {
            if acceleration[d] != 0. {
                work = add(
                    work,
                    mul(mul(mul(h, mass[i])?, velocity[i][d])?, acceleration[d])?,
                )?;
            }
        }
    }
    Ok(work)
}
#[allow(clippy::too_many_arguments)]
fn qualify(
    e: &Equation,
    coarse: &Equation,
    h: f64,
    unknown: [f64; V],
    before: TranslatedViscousStamp,
    after: TranslatedViscousStamp,
    time_after: f64,
    iterations: usize,
    calls: usize,
    acceleration: [f64; 3],
) -> Result<CoupledDiscreteReport, CoupledDiscreteError> {
    let mut gcl = [0.; N];
    let mut dbe = 0.;
    let mut dmix = 0.;
    let mut quadrature: f64 = 0.;
    let mut mass_before = 0.;
    let mut mass_after = 0.;
    let mut px = 0.;
    for i in 0..N {
        gcl[i] = add(e.end.mass[i], -e.start.mass[i])?;
        let mut du = [0.; 3];
        for d in 0..3 {
            du[d] = add(e.end.u[i][d], -e.start.u[i][d])?;
        }
        dbe = add(dbe, mul(mul(0.5, e.start.mass[i])?, dot(&du, &du)?)?)?;
        mass_before = add(mass_before, e.start.mass[i])?;
        mass_after = add(mass_after, e.end.mass[i])?;
        px = add(
            px,
            add(
                mul(e.end.mass[i], e.end.u[i][0])?,
                -mul(e.start.mass[i], e.start.u[i][0])?,
            )?,
        )?;
    }
    for f in 0..e.end.faces {
        let [i, j] = e.end.pairs[f];
        let net = add(e.integral.plus[f], -e.integral.minus[f])?;
        gcl[i] = add(gcl[i], net)?;
        gcl[j] = add(gcl[j], -net)?;
        let mut du = [0.; 3];
        for d in 0..3 {
            du[d] = add(e.end.u[i][d], -e.end.u[j][d])?;
        }
        dmix = add(
            dmix,
            mul(
                mul(0.5, add(e.integral.plus[f], e.integral.minus[f])?)?,
                dot(&du, &du)?,
            )?,
        )?;
        quadrature = quadrature
            .max(add(e.integral.plus[f], -coarse.integral.plus[f])?.abs())
            .max(add(e.integral.minus[f], -coarse.integral.minus[f])?.abs());
    }
    for v in 0..V {
        quadrature = quadrature.max(
            add(
                e.integral.physical_convection[v],
                -coarse.integral.physical_convection[v],
            )?
            .abs(),
        );
    }
    let mut wg = 0.;
    let mut gcl_max: f64 = 0.;
    for i in 0..N {
        let allowance = mul(128. * f64::EPSILON, add(e.start.mass[i], e.end.mass[i])?)?;
        if gcl[i].abs() > allowance {
            return Err(CoupledDiscreteError::ConservationFailure);
        }
        wg = add(wg, mul(mul(0.5, gcl[i])?, dot(&e.end.u[i], &e.end.u[i])?)?)?;
        gcl_max = gcl_max.max(gcl[i].abs());
    }
    let e0 = energy(&e.start)?;
    let e1 = energy(&e.end)?;
    let dmu = mul(h, e.end.strain_power)?;
    let wp = mul(h, e.end.pressure_work)?;
    let wr = mul(h, dot(&e.end.z, &e.rate)?)?;
    let mut ledger = add(
        add(add(add(add(add(e1, -e0)?, dbe)?, dmix)?, dmu)?, wp)?,
        add(wg, -wr)?,
    )?;
    let external_work = force_work(&e.start.mass, &e.end.u, h, acceleration)?;
    if external_work != 0. {
        ledger = add(ledger, -external_work)?;
    }
    let mut scale = add(
        add(
            add(
                add(add(add(add(e0, e1)?, dbe)?, dmix)?, dmu.abs())?,
                wp.abs(),
            )?,
            wg.abs(),
        )?,
        wr.abs(),
    )?;
    if external_work != 0. {
        scale = add(scale, external_work.abs())?;
    }
    let allowance = mul(128. * f64::EPSILON, scale)?;
    if dbe < 0.
        || dmix < 0.
        || dmu < 0.
        || wr.abs() > allowance
        || ledger.abs() > allowance
        || wp.abs() > allowance
    {
        return Err(CoupledDiscreteError::WorkFailure);
    }
    if quadrature > 1e-15 {
        return Err(CoupledDiscreteError::QuadratureRefinement { error: quadrature });
    }
    if norm(&e.rate)? > LIMIT
        || norm(&e.direct_rate)? > LIMIT
        || add(px, -mul(mul(h, mass_before)?, acceleration[0])?)?.abs() > LIMIT
        || add(mass_after, -mass_before)?.abs() > LIMIT
    {
        return Err(CoupledDiscreteError::ConservationFailure);
    }
    let mut endpoint_vs_path_momentum_max: f64 = 0.;
    for v in 0..V {
        endpoint_vs_path_momentum_max = endpoint_vs_path_momentum_max
            .max(add(e.endpoint_convection[v], -e.integral.physical_convection[v])?.abs());
    }
    Ok(CoupledDiscreteReport {
        before,
        after,
        dt: h,
        time_after,
        unknowns: unknown,
        end_q: e.end.q,
        end_eta: e.end.eta,
        energy_before: e0,
        energy_after: e1,
        backward_euler_loss: dbe,
        mixing_loss: dmix,
        viscous_loss: dmu,
        pressure_work: wp,
        gcl_work: wg,
        residual_work: wr,
        ledger_error: ledger,
        work_allowance: allowance,
        finite_momentum_rate_norm: norm(&e.rate)?,
        direct_momentum_rate_norm: norm(&e.direct_rate)?,
        gcl_max,
        quadrature_error: quadrature,
        full_constraints: e.integral.max_constraints,
        iterations,
        equation_evaluations: calls,
        sign_roots: e.sign_roots,
        mass_before,
        mass_after,
        endpoint_vs_path_momentum_max,
    })
}
fn rule(order: usize, k: usize) -> Result<(f64, f64), CoupledDiscreteError> {
    match order {
        16 => Ok(RULE16[k]),
        32 => Ok(RULE32[k]),
        _ => Err(CoupledDiscreteError::QuadratureFailure),
    }
}
#[allow(clippy::excessive_precision)]
const RULE16: [(f64, f64); 16] = [
    (-9.89400934991649939e-01, 2.71524594117541762e-02),
    (-9.44575023073232600e-01, 6.22535239386474565e-02),
    (-8.65631202387831755e-01, 9.51585116824926053e-02),
    (-7.55404408355002999e-01, 1.24628971255534071e-01),
    (-6.17876244402643771e-01, 1.49595988816576708e-01),
    (-4.58016777657227370e-01, 1.69156519395002647e-01),
    (-2.81603550779258915e-01, 1.82603415044923639e-01),
    (-9.50125098376374405e-02, 1.89450610455068641e-01),
    (9.50125098376374405e-02, 1.89450610455068641e-01),
    (2.81603550779258915e-01, 1.82603415044923639e-01),
    (4.58016777657227370e-01, 1.69156519395002647e-01),
    (6.17876244402643771e-01, 1.49595988816576708e-01),
    (7.55404408355002999e-01, 1.24628971255534071e-01),
    (8.65631202387831755e-01, 9.51585116824926053e-02),
    (9.44575023073232600e-01, 6.22535239386474565e-02),
    (9.89400934991649939e-01, 2.71524594117541762e-02),
];
#[allow(clippy::excessive_precision)]
const RULE32: [(f64, f64); 32] = [
    (-9.97263861849481570e-01, 7.01861000947050576e-03),
    (-9.85611511545268382e-01, 1.62743947309057432e-02),
    (-9.64762255587506390e-01, 2.53920653092620241e-02),
    (-9.34906075937739667e-01, 3.42738629130217645e-02),
    (-8.96321155766052091e-01, 4.28358980222268357e-02),
    (-8.49367613732569970e-01, 5.09980592623760914e-02),
    (-7.94483795967942386e-01, 5.86840934785355650e-02),
    (-7.32182118740289711e-01, 6.58222227763616829e-02),
    (-6.63044266930215231e-01, 7.23457941088483381e-02),
    (-5.87715757240762304e-01, 7.81938957870702278e-02),
    (-5.06899908932229359e-01, 8.33119242269467070e-02),
    (-4.21351276130635333e-01, 8.76520930044037833e-02),
    (-3.31868602282127667e-01, 9.11738786957637798e-02),
    (-2.39287362252137065e-01, 9.38443990808045109e-02),
    (-1.44471961582796488e-01, 9.56387200792747083e-02),
    (-4.83076656877383243e-02, 9.65400885147276594e-02),
    (4.83076656877383243e-02, 9.65400885147276594e-02),
    (1.44471961582796488e-01, 9.56387200792747083e-02),
    (2.39287362252137065e-01, 9.38443990808045109e-02),
    (3.31868602282127667e-01, 9.11738786957637798e-02),
    (4.21351276130635333e-01, 8.76520930044037833e-02),
    (5.06899908932229359e-01, 8.33119242269467070e-02),
    (5.87715757240762304e-01, 7.81938957870702278e-02),
    (6.63044266930215231e-01, 7.23457941088483381e-02),
    (7.32182118740289711e-01, 6.58222227763616829e-02),
    (7.94483795967942386e-01, 5.86840934785355650e-02),
    (8.49367613732569970e-01, 5.09980592623760914e-02),
    (8.96321155766052091e-01, 4.28358980222268357e-02),
    (9.34906075937739667e-01, 3.42738629130217645e-02),
    (9.64762255587506390e-01, 2.53920653092620241e-02),
    (9.85611511545268382e-01, 1.62743947309057432e-02),
    (9.97263861849481570e-01, 7.01861000947050576e-03),
];

include!("reference_helpers.rs");

include!("reference_probe.rs");

include!("reference_paired_adapter.rs");
include!("../paired_probe.rs");
