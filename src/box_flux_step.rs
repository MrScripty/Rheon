//! Explicit fixed-volume boundary and appearance-transport policy for stepping.
use crate::{
    Axis, BoxFluxReport, BoxFluxWorkspace, ForcedStepReport, GridGeometry, PrescribedBoxFlux,
    PressureImplementation, SimulationError,
};

/// Existing midpoint/clamped interpolation, including nearest sample extension
/// at inflow. No specified reservoir concentration or conservative scalar flux.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoxFluxTracerPolicy {
    ClampedAppearance,
}
/// One copied end-of-step boundary request. Velocity advection uses the previous
/// accepted velocity, then projection imposes this snapshot; tracer uses the
/// resulting velocity and the explicitly selected extension policy.
#[derive(Debug, Clone, Copy)]
pub struct BoxFluxStepBoundary {
    pub flux: PrescribedBoxFlux,
    pub tracer: BoxFluxTracerPolicy,
}

/// Interior unknown-face energy ledger. Advection change is measured rather
/// than assumed dissipative. Force and projection terms use stored increments.
/// Tracer transport change is measured, not claimed as a conservative flux.
#[derive(Debug, Clone, Copy)]
pub struct BoxFluxStepWork {
    pub kinetic_old: f64,
    pub kinetic_after_advection: f64,
    pub kinetic_after_smoke_force: f64,
    pub kinetic_before_projection: f64,
    pub kinetic_accepted: f64,
    pub advection_change: f64,
    pub smoke_force_work: f64,
    pub external_force_work: f64,
    pub budget_error: f64,
    /// dt times the accepted report's numerical net outward volume flux.
    /// The control volume stays fixed; no mass state is evolved from this value.
    pub boundary_volume_imbalance: f64,
    pub tracer_old_integral: f64,
    pub tracer_transport_change: f64,
    pub tracer_source_change: f64,
}
#[derive(Debug, Clone, Copy)]
pub struct BoxFluxStepReport {
    /// On this opt-in path StepReport.kinetic_energy uses interior faces only.
    pub step: ForcedStepReport,
    pub projection: BoxFluxReport,
    pub work: BoxFluxStepWork,
    pub tracer_policy: BoxFluxTracerPolicy,
    pub simulation_array_bytes: usize,
    pub boundary_workspace_array_bytes: usize,
}

/// Caller-owned independently capped scratch: the preserved affine pressure
/// workspace plus three provisional face arrays. Simulation's original closed
/// pressure workspace remains retained and included in its own capacity report.
/// No step-time allocation; a workspace cannot be used with mismatched geometry,
/// density or solver identity. Pressure and provisional values are never state.
pub struct BoxFluxStepWorkspace {
    pub(crate) projection: BoxFluxWorkspace,
    pub(crate) provisional: [Vec<f32>; 3],
    density: f64,
    allocated_bytes: usize,
}
impl BoxFluxStepWorkspace {
    pub fn new(
        grid: GridGeometry,
        density: f64,
        limit: usize,
        implementation: PressureImplementation,
    ) -> Result<Self, SimulationError> {
        let faces = Axis::ALL.into_iter().try_fold(0_usize, |n, a| {
            n.checked_add(grid.face_len(a))
                .ok_or(SimulationError::AllocationFailed)
        })?;
        let face_bytes = faces
            .checked_mul(4)
            .ok_or(SimulationError::AllocationFailed)?;
        let required = grid
            .cell_len()
            .checked_mul(56)
            .and_then(|n| n.checked_add(face_bytes))
            .ok_or(SimulationError::AllocationFailed)?;
        if required > limit {
            return Err(SimulationError::BufferLimit { required, limit });
        }
        let projection =
            BoxFluxWorkspace::new(grid.clone(), density, limit - face_bytes, implementation)?;
        let mut remaining = limit - projection.allocated_bytes();
        let provisional = [
            crate::simulation::allocate(grid.face_len(Axis::X), &mut remaining)?,
            crate::simulation::allocate(grid.face_len(Axis::Y), &mut remaining)?,
            crate::simulation::allocate(grid.face_len(Axis::Z), &mut remaining)?,
        ];
        Ok(Self {
            projection,
            provisional,
            density,
            allocated_bytes: limit - remaining,
        })
    }
    pub fn allocated_bytes(&self) -> usize {
        self.allocated_bytes
    }
    pub fn grid(&self) -> &GridGeometry {
        self.projection.grid()
    }
    pub fn implementation(&self) -> PressureImplementation {
        self.projection.implementation()
    }
    pub(crate) fn matches(
        &self,
        grid: &GridGeometry,
        density: f64,
        method: PressureImplementation,
    ) -> bool {
        self.grid() == grid
            && self.density.to_bits() == density.to_bits()
            && self.implementation() == method
    }
}

pub(crate) fn interior_energy(
    grid: &GridGeometry,
    density: f64,
    velocity: [&[f32]; 3],
) -> Result<f64, SimulationError> {
    let mut sum = 0.0;
    for axis in Axis::ALL {
        let d = axis.index();
        let [nx, ny, nz] = grid.face_counts(axis);
        for k in 0..nz {
            for j in 0..ny {
                for i in 0..nx {
                    let p = [i, j, k];
                    if p[d] > 0 && p[d] < grid.counts()[d] {
                        sum += f64::from(velocity[d][grid.face_unchecked(axis, p)]).powi(2);
                    }
                }
            }
        }
    }
    let energy = 0.5 * density * grid.cell_volume() * sum;
    if energy.is_finite() {
        Ok(energy)
    } else {
        Err(SimulationError::ArithmeticFailure)
    }
}
