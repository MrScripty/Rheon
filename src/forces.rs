//! Explicit, caller-owned body forces on a stationary closed MAC box.
use crate::{Axis, GridGeometry, SimulationError, StepReport, StepStage};

/// Acceleration is m/s²; force density is N/m³ and is divided by the
/// simulation's constant positive density (kg/m³) before the velocity update.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForceUnits {
    Acceleration,
    ForceDensity,
}

/// World-space half-open support: lower <= position < upper on each axis.
/// Bounds must be finite, nonempty, and contained in the simulation box.
#[derive(Debug, Clone, Copy)]
pub struct ForceRegion {
    pub lower: [f64; 3],
    pub upper: [f64; 3],
}

/// A constant vector, sampled on each component's own face lattice. None
/// means the whole box. Prescribed zero normal wall velocities remain fixed.
/// The slice passed to a step is borrowed only for that call, not retained.
#[derive(Debug, Clone, Copy)]
pub struct BodyForce {
    pub value: [f64; 3],
    pub units: ForceUnits,
    pub region: Option<ForceRegion>,
}
impl BodyForce {
    fn acceleration(self, density: f64) -> [f64; 3] {
        match self.units {
            ForceUnits::Acceleration => self.value,
            ForceUnits::ForceDensity => self.value.map(|v| v / density),
        }
    }
    fn contains(self, position: [f64; 3]) -> bool {
        self.region
            .is_none_or(|r| (0..3).all(|d| position[d] >= r.lower[d] && position[d] < r.upper[d]))
    }
}

/// External-force stage only, after advection and the legacy smoke force and
/// before pressure. Energies use the core's rho * cell_volume face weights.
/// Work uses the actual stored f32 velocity change, including rounding, and
/// can be negative. It is not the energy change of the complete timestep.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ForceReport {
    pub kinetic_energy_before: f64,
    pub kinetic_energy_after: f64,
    pub applied_work: f64,
}
#[derive(Debug, Clone, Copy)]
pub struct ForcedStepReport {
    pub step: StepReport,
    /// None means no external force stage was requested.
    pub forces: Option<ForceReport>,
}

pub(crate) fn validate(
    grid: &GridGeometry,
    density: f64,
    forces: &[BodyForce],
) -> Result<f64, SimulationError> {
    let mut rate = 0.0;
    for force in forces {
        if force.value.iter().any(|v| !v.is_finite()) {
            return Err(SimulationError::InvalidForce);
        }
        if let Some(r) = force.region {
            for d in 0..3 {
                if !r.lower[d].is_finite()
                    || !r.upper[d].is_finite()
                    || r.lower[d] >= r.upper[d]
                    || r.lower[d] < grid.origin()[d]
                    || r.upper[d] > grid.upper()[d]
                {
                    return Err(SimulationError::InvalidForce);
                }
            }
        }
        for (a, h) in force.acceleration(density).into_iter().zip(grid.spacing()) {
            rate += a.abs() / h;
        }
    }
    if !rate.is_finite() {
        return Err(SimulationError::ArithmeticFailure);
    }
    Ok(rate)
}

pub(crate) fn apply(
    grid: &GridGeometry,
    density: f64,
    forces: &[BodyForce],
    dt: f64,
    velocity: [&mut [f32]; 3],
    cancel: &mut impl FnMut(StepStage) -> bool,
) -> Result<ForceReport, SimulationError> {
    apply_on_domain(grid, density, forces, dt, velocity, None, cancel)
}
pub(crate) fn apply_on_domain(
    grid: &GridGeometry,
    density: f64,
    forces: &[BodyForce],
    dt: f64,
    velocity: [&mut [f32]; 3],
    surface: Option<&crate::SlabFreeSurface>,
    cancel: &mut impl FnMut(StepStage) -> bool,
) -> Result<ForceReport, SimulationError> {
    let mass = density * grid.cell_volume();
    let energy = |fields: &[&mut [f32]; 3]| {
        0.5 * mass
            * fields
                .iter()
                .flat_map(|v| v.iter())
                .map(|&v| f64::from(v).powi(2))
                .sum::<f64>()
    };
    let before = energy(&velocity);
    let mut work = 0.0;
    for axis in Axis::ALL {
        let d = axis.index();
        let [nx, ny, nz] = grid.face_counts(axis);
        for k in 0..nz {
            if cancel(StepStage::ForceSlice) {
                return Err(SimulationError::Cancelled {
                    stage: StepStage::ForceSlice,
                });
            }
            for j in 0..ny {
                for i in 0..nx {
                    let p = [i, j, k];
                    if p[d] == 0 || p[d] == grid.counts()[d] {
                        continue;
                    }
                    if surface.is_some_and(|s| !s.face_active(axis, p)) {
                        continue;
                    }
                    let position = grid.face_position(axis, p).expect("bounded face");
                    let mut a = 0.0;
                    for force in forces {
                        if force.contains(position) {
                            a += force.acceleration(density)[d];
                        }
                    }
                    let index = grid.face_unchecked(axis, p);
                    let old = f64::from(velocity[d][index]);
                    let updated = old + dt * a;
                    if !updated.is_finite() || updated.abs() > f64::from(f32::MAX) {
                        return Err(SimulationError::ArithmeticFailure);
                    }
                    let new = updated as f32;
                    let delta = f64::from(new) - old;
                    work += mass * delta * (old + 0.5 * delta);
                    velocity[d][index] = new;
                }
            }
        }
    }
    let after = energy(&velocity);
    if !before.is_finite() || !after.is_finite() || !work.is_finite() {
        return Err(SimulationError::ArithmeticFailure);
    }
    Ok(ForceReport {
        kinetic_energy_before: before,
        kinetic_energy_after: after,
        applied_work: work,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finite_step_work_can_be_negative_and_uses_both_velocity_endpoints() {
        let g = GridGeometry::new([2, 1, 1], [0.5, 1.0, 1.0], [0.0; 3]).unwrap();
        let (mut x, mut y, mut z) = ([0.0, 2.0, 0.0], [0.0; 4], [0.0; 4]);
        let force = BodyForce {
            value: [-8.0, 0.0, 0.0],
            units: ForceUnits::ForceDensity,
            region: None,
        };
        let report = apply(
            &g,
            2.0,
            &[force],
            0.25,
            [&mut x, &mut y, &mut z],
            &mut |_| false,
        )
        .unwrap();
        // Face mass=1, old=2, new=1: ΔE=-3/2. Old-velocity power alone
        // gives -2 and misses the positive finite-step term +1/2.
        assert_eq!(x, [0.0, 1.0, 0.0]);
        assert_eq!(report.kinetic_energy_before, 2.0);
        assert_eq!(report.kinetic_energy_after, 0.5);
        assert_eq!(report.applied_work, -1.5);
    }

    #[test]
    fn work_accounts_for_actual_storage_rounding_and_closed_wall_faces() {
        let g = GridGeometry::new([2, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
        let (mut x, mut y, mut z) = ([0.0, 1.0, 0.0], [0.0; 4], [0.0; 4]);
        let force = BodyForce {
            value: [1e-10, 2.0, -3.0],
            units: ForceUnits::Acceleration,
            region: None,
        };
        let report = apply(
            &g,
            1.0,
            &[force],
            0.25,
            [&mut x, &mut y, &mut z],
            &mut |_| false,
        )
        .unwrap();
        // Increment is below f32 resolution at 1; every Y/Z face is a wall.
        assert_eq!(x, [0.0, 1.0, 0.0]);
        assert_eq!(report.applied_work, 0.0);
        assert_eq!(report.kinetic_energy_before, report.kinetic_energy_after);
        assert_eq!(y, [0.0; 4]);
        assert_eq!(z, [0.0; 4]);
    }
}
