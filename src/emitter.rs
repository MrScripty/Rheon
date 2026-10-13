//! Prescribed pressure-head or velocity nozzles with two opposed end disks.
//! Carrier momentum/tracer and particle births use the same authored controls.
//! This is a source, not a moving solid boundary; inlet perturbations are not
//! a validated turbulence model. See docs/emitter-controls.md for semantics.
use crate::observation::{CalculationStage, WorkProbe};
use crate::{Axis, GridGeometry, SimulationError, StepStage};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RotatingCylinderEmitter {
    pub center: [f64; 3],
    pub half_length: f64,
    pub radius: f64,
    pub nozzle_depth: f64,
    /// Revolutions per simulated second about world +Y.
    pub rotations_per_second: f64,
    /// Pressure head (Pa) for the +axis and -axis ends respectively.
    pub pressure_heads: [f64; 2],
    pub tracer_rate: f64,
    /// Positive jet peak velocity relative to the nozzle at time zero (m/s).
    /// None retains pressure-head forcing. Some([0;3]) disables both jets.
    pub emitted_velocity: Option<[f64; 3]>,
    /// Translation of the emitter centre in world m/s, independently of jet speed.
    pub motion_velocity: [f64; 3],
    pub turbulence: crate::EmitterTurbulence,
}
impl Default for RotatingCylinderEmitter {
    fn default() -> Self {
        Self {
            center: [0.5; 3],
            half_length: 0.16,
            radius: 0.075,
            nozzle_depth: 0.08,
            rotations_per_second: 1.0,
            pressure_heads: [0.12, 0.08],
            tracer_rate: 12.0,
            emitted_velocity: None,
            motion_velocity: [0.0; 3],
            turbulence: crate::EmitterTurbulence::default(),
        }
    }
}
impl RotatingCylinderEmitter {
    pub fn axis_at(self, time: f64) -> Result<[f64; 3], SimulationError> {
        let turns = time * self.rotations_per_second;
        if !turns.is_finite() {
            return Err(SimulationError::InvalidEmitter);
        }
        let angle = turns.rem_euclid(1.0) * std::f64::consts::TAU;
        let initial = self.emitted_velocity.unwrap_or([1.0, 0.0, 0.0]);
        let length = length(initial);
        if !length.is_finite() {
            return Err(SimulationError::InvalidEmitter);
        }
        let initial = if length == 0.0 {
            [1.0, 0.0, 0.0]
        } else {
            initial.map(|v| v / length)
        };
        Ok([
            initial[0] * angle.cos() - initial[2] * angle.sin(),
            initial[1],
            initial[0] * angle.sin() + initial[2] * angle.cos(),
        ])
    }
    /// Initial centre plus absolute simulated time times motion velocity (metres).
    pub fn center_at(self, time: f64) -> Result<[f64; 3], SimulationError> {
        if !time.is_finite() {
            return Err(SimulationError::InvalidEmitter);
        }
        let center = std::array::from_fn(|d| self.center[d] + time * self.motion_velocity[d]);
        if center.iter().any(|v| !v.is_finite()) {
            return Err(SimulationError::InvalidEmitter);
        }
        Ok(center)
    }
    /// Unperturbed world velocity at peak profile: inherited motion plus the
    /// opposed relative jet. End 0 is positive; end 1 is negative. This is an
    /// inlet target, not a promise about the post-projection carrier velocity.
    pub fn peak_velocity_at(
        self,
        end: usize,
        time: f64,
        density: f64,
    ) -> Result<[f64; 3], SimulationError> {
        if end > 1 || !density.is_normal() || density <= 0.0 {
            return Err(SimulationError::InvalidEmitter);
        }
        let axis = self.axis_at(time)?;
        let speed = self.jet_speed(end, density);
        if !speed.is_finite() {
            return Err(SimulationError::InvalidEmitter);
        }
        let sign = if end == 0 { 1.0 } else { -1.0 };
        let velocity = std::array::from_fn(|d| self.motion_velocity[d] + axis[d] * sign * speed);
        if velocity.iter().any(|v| !v.is_finite()) {
            return Err(SimulationError::InvalidEmitter);
        }
        Ok(velocity)
    }
    pub(crate) fn jet_speed(self, end: usize, density: f64) -> f64 {
        self.emitted_velocity
            .map_or_else(|| (2.0 * self.pressure_heads[end] / density).sqrt(), length)
    }
    pub(crate) fn speed_bound(self, density: f64) -> f64 {
        self.jet_speed(0, density).max(self.jet_speed(1, density))
            + length(self.motion_velocity)
            + self.turbulence.strength
    }
    /// Orthonormal source axes: jet, disk-up, disk-across.
    pub fn basis_at(self, time: f64) -> Result<[[f64; 3]; 3], SimulationError> {
        let a = self.axis_at(time)?;
        if self.emitted_velocity.is_none() {
            return Ok([a, [0.0, 1.0, 0.0], [-a[2], 0.0, a[0]]]);
        }
        let h = a[0].hypot(a[2]);
        let across = if h > 1e-12 {
            [-a[2] / h, 0.0, a[0] / h]
        } else {
            [0.0, 0.0, 1.0]
        };
        let up = [
            across[1] * a[2] - across[2] * a[1],
            across[2] * a[0] - across[0] * a[2],
            across[0] * a[1] - across[1] * a[0],
        ];
        Ok([a, up, across])
    }
    pub(crate) fn validate_interval(
        self,
        grid: &GridGeometry,
        time: f64,
        dt: f64,
    ) -> Result<(), SimulationError> {
        for t in [time, time + dt] {
            let center = self.center_at(t)?;
            let axis = self.axis_at(t)?;
            let reach = (self.half_length + self.nozzle_depth).hypot(self.radius);
            let vertical = axis[1].abs() * (self.half_length + self.nozzle_depth)
                + self.radius * (1.0 - axis[1] * axis[1]).max(0.0).sqrt();
            for (d, extent) in [reach, vertical, reach].into_iter().enumerate() {
                if center[d] - extent < grid.origin()[d] || center[d] + extent > grid.upper()[d] {
                    return Err(SimulationError::InvalidEmitter);
                }
            }
            self.turbulence.sample(
                [
                    self.half_length + self.nozzle_depth,
                    self.radius,
                    self.radius,
                ],
                t,
            )?;
        }
        Ok(())
    }
    pub(crate) fn perturbation(
        self,
        local: [f64; 3],
        time: f64,
    ) -> Result<[f64; 3], SimulationError> {
        let v = self.turbulence.sample(local, time)?;
        let frame = self.basis_at(time)?;
        Ok(std::array::from_fn(|d| {
            frame[0][d] * v[0] + frame[1][d] * v[1] + frame[2][d] * v[2]
        }))
    }
    pub(crate) fn validate(
        self,
        grid: &GridGeometry,
        density: f64,
    ) -> Result<f64, SimulationError> {
        self.turbulence.validate()?;
        if self
            .motion_velocity
            .iter()
            .any(|v| !v.is_finite() || v.abs() > 1e6)
            || self
                .emitted_velocity
                .is_some_and(|v| v.iter().any(|x| !x.is_finite() || x.abs() > 1e6))
            || self.center.iter().any(|v| !v.is_finite())
            || [self.half_length, self.radius, self.nozzle_depth]
                .iter()
                .any(|v| !v.is_finite() || *v <= 0.0)
            || !self.rotations_per_second.is_finite()
            || self.rotations_per_second.abs() > 100.0
            || self
                .pressure_heads
                .iter()
                .any(|v| !v.is_finite() || *v < 0.0)
            || !self.tracer_rate.is_finite()
            || self.tracer_rate < 0.0
        {
            return Err(SimulationError::InvalidEmitter);
        }
        self.validate_interval(grid, 0.0, 0.0)?;
        let peak = if self.emitted_velocity.is_some() {
            self.speed_bound(density).powi(2) / self.nozzle_depth
        } else {
            self.pressure_heads[0].max(self.pressure_heads[1]) / (density * self.nozzle_depth)
                + (length(self.motion_velocity) + self.turbulence.strength)
                    * self.speed_bound(density)
                    / self.nozzle_depth
        };
        let rate = if self.emitted_velocity.is_none()
            && self.motion_velocity == [0.0; 3]
            && self.turbulence.strength == 0.0
        {
            peak * (1.0 / grid.spacing()[0] + 1.0 / grid.spacing()[2])
        } else {
            peak * grid.spacing().into_iter().map(|h| 1.0 / h).sum::<f64>()
        };
        if !rate.is_finite() {
            return Err(SimulationError::ArithmeticFailure);
        }
        Ok(rate)
    }
    fn sample(self, position: [f64; 3], axis: [f64; 3]) -> (f64, f64) {
        let v = std::array::from_fn::<_, 3, _>(|d| position[d] - self.center[d]);
        let axial = v[0] * axis[0] + v[1] * axis[1] + v[2] * axis[2];
        let end_distance = axial.abs() - self.half_length;
        if !(0.0..self.nozzle_depth).contains(&end_distance) {
            return (0.0, 0.0);
        }
        let h = axis[0].hypot(axis[2]);
        let across = if self.emitted_velocity.is_none() {
            -v[0] * axis[2] + v[2] * axis[0]
        } else if h > 1e-12 {
            (-v[0] * axis[2] + v[2] * axis[0]) / h
        } else {
            v[2]
        };
        let vertical = if self.emitted_velocity.is_none() {
            v[1]
        } else if h > 1e-12 {
            -axis[0] * axis[1] / h * v[0] + h * v[1] - axis[2] * axis[1] / h * v[2]
        } else {
            -axis[1] * v[0]
        };
        let radial = (across * across + vertical * vertical) / (self.radius * self.radius);
        if radial >= 1.0 {
            return (0.0, 0.0);
        }
        // Zero at the rim, deliberately stronger on the upper side. Frozen over
        // one accepted interval, with separate pressure heads on the two ends.
        let profile = (1.0 - radial).powi(2) * (0.75 + 0.25 * vertical / self.radius);
        let end = usize::from(axial < 0.0);
        let pressure = self
            .emitted_velocity
            .map_or(self.pressure_heads[end], length)
            * profile;
        (axial.signum() * pressure, pressure)
    }
    pub(crate) fn apply_velocity(
        self,
        grid: &GridGeometry,
        density: f64,
        time: f64,
        dt: f64,
        fields: [&mut [f32]; 3],
        cancel: &mut impl FnMut(StepStage) -> bool,
    ) -> Result<(), SimulationError> {
        let axis = self.axis_at(time)?;
        let frame = self.basis_at(time)?;
        let center = self.center_at(time)?;
        let shifted = Self { center, ..self };
        for (d, values) in fields.into_iter().enumerate() {
            if d == 1
                && self.emitted_velocity.is_none()
                && self.motion_velocity == [0.0; 3]
                && self.turbulence.strength == 0.0
            {
                continue;
            }
            let face_axis = Axis::ALL[d];
            let dimensions = grid.face_counts(face_axis);
            bands(
                values,
                dimensions,
                StepStage::ForceSlice,
                cancel,
                |p, value| {
                    if p[d] == 0 || p[d] == grid.counts()[d] {
                        return Ok(());
                    }
                    let position = grid
                        .face_position(face_axis, p)
                        .expect("bounded emitter face");
                    let (head, magnitude) = shifted.sample(position, axis);
                    if magnitude == 0.0 {
                        return Ok(());
                    }
                    let delta = std::array::from_fn::<_, 3, _>(|j| position[j] - center[j]);
                    let local = frame.map(|a| a.into_iter().zip(delta).map(|(a, b)| a * b).sum());
                    let noise = self.perturbation(local, time)?;
                    let speed = self.jet_speed(usize::from(head < 0.0), density);
                    let result = if self.emitted_velocity.is_some() {
                        let profile = (magnitude / speed).sqrt();
                        let target = axis[d] * head.signum() * speed * profile
                            + self.motion_velocity[d]
                            + noise[d] * profile;
                        let blend = -(-dt * speed / self.nozzle_depth).exp_m1();
                        f64::from(*value) + (target - f64::from(*value)) * blend
                    } else if self.motion_velocity == [0.0; 3] && self.turbulence.strength == 0.0 {
                        f64::from(*value) + dt * axis[d] * head / (density * self.nozzle_depth)
                    } else {
                        let profile = magnitude / self.pressure_heads[usize::from(head < 0.0)];
                        f64::from(*value)
                            + dt * (axis[d] * head / (density * self.nozzle_depth)
                                + (self.motion_velocity[d] + noise[d]) * speed / self.nozzle_depth
                                    * profile)
                    };
                    if !result.is_finite() || result.abs() > f64::from(f32::MAX) {
                        return Err(SimulationError::ArithmeticFailure);
                    }
                    *value = result as f32;
                    Ok(())
                },
            )?;
        }
        Ok(())
    }
    pub(crate) fn apply_tracer(
        self,
        grid: &GridGeometry,
        time: f64,
        dt: f64,
        tracer: &mut [f32],
        cancel: &mut impl FnMut(StepStage) -> bool,
    ) -> Result<(), SimulationError> {
        let axis = self.axis_at(time)?;
        let head = self
            .emitted_velocity
            .map_or(self.pressure_heads[0].max(self.pressure_heads[1]), length);
        let shifted = Self {
            center: self.center_at(time)?,
            ..self
        };
        if head == 0.0 {
            return Ok(());
        }
        bands(
            tracer,
            grid.counts(),
            StepStage::TracerSlice,
            cancel,
            |p, value| {
                let (_, pressure) =
                    shifted.sample(grid.cell_position(p).expect("bounded emitter cell"), axis);
                let result = f64::from(*value) + dt * self.tracer_rate * pressure / head;
                if !result.is_finite() {
                    return Err(SimulationError::ArithmeticFailure);
                }
                *value = result.min(1.0) as f32;
                Ok(())
            },
        )
    }
}
pub(crate) fn length(v: [f64; 3]) -> f64 {
    v[0].hypot(v[1]).hypot(v[2])
}
fn bands(
    values: &mut [f32],
    dimensions: [usize; 3],
    stage: StepStage,
    cancel: &mut impl FnMut(StepStage) -> bool,
    update: impl Fn([usize; 3], &mut f32) -> Result<(), SimulationError> + Send + Sync,
) -> Result<(), SimulationError> {
    let [nx, ny, _] = dimensions;
    let plane = nx * ny;
    let band_len = plane * (16 * 1024_usize).div_ceil(plane);
    for (band, values) in values.chunks_mut(band_len).enumerate() {
        if cancel(stage) {
            return Err(SimulationError::Cancelled { stage });
        }
        crate::parallel::try_for_each_chunk_mut(
            values,
            CalculationStage::Emitter,
            1024,
            |start, values| {
                let probe = WorkProbe::new(CalculationStage::Emitter);
                for (i, value) in values.iter_mut().enumerate() {
                    let row = band * band_len + start + i;
                    let p = [row % nx, (row / nx) % ny, row / plane];
                    probe.visit(p);
                    update(p, value)?;
                }
                Ok::<(), SimulationError>(())
            },
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nozzle_pressure_is_nonuniform_and_end_heads_are_independent() {
        let emitter = RotatingCylinderEmitter::default();
        let axis = emitter.axis_at(0.0).unwrap();
        let x = emitter.center[0] + emitter.half_length + emitter.nozzle_depth * 0.5;
        let center = emitter.sample([x, 0.5, 0.5], axis);
        let upper = emitter.sample([x, 0.53, 0.5], axis);
        let lower = emitter.sample([x, 0.47, 0.5], axis);
        let opposite = emitter.sample([1.0 - x, 0.5, 0.5], axis);
        assert!(center.0 > 0.0 && opposite.0 < 0.0);
        assert!(center.1 > upper.1 && upper.1 > lower.1);
        assert!((center.1 / opposite.1 - 1.5).abs() < 1e-12);
        assert_eq!(emitter.sample([x, 0.6, 0.5], axis), (0.0, 0.0));
    }
}
