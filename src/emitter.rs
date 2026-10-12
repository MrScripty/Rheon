//! A prescribed pressure-head nozzle model on the fixed-box carrier. The two
//! end disks have a nonuniform radial/vertical profile. Momentum and appearance
//! concentration are injected together before transactional publication; this
//! is an emitter, not a moving solid boundary or a liquid mass source.
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
        Ok([angle.cos(), 0.0, angle.sin()])
    }
    pub(crate) fn validate(
        self,
        grid: &GridGeometry,
        density: f64,
    ) -> Result<f64, SimulationError> {
        if self.center.iter().any(|v| !v.is_finite())
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
        let reach = (self.half_length + self.nozzle_depth).hypot(self.radius);
        for (d, extent) in [reach, self.radius, reach].into_iter().enumerate() {
            if self.center[d] - extent < grid.origin()[d]
                || self.center[d] + extent > grid.upper()[d]
            {
                return Err(SimulationError::InvalidEmitter);
            }
        }
        let peak =
            self.pressure_heads[0].max(self.pressure_heads[1]) / (density * self.nozzle_depth);
        let rate = peak * (1.0 / grid.spacing()[0] + 1.0 / grid.spacing()[2]);
        if !rate.is_finite() {
            return Err(SimulationError::ArithmeticFailure);
        }
        Ok(rate)
    }
    fn sample(self, position: [f64; 3], axis: [f64; 3]) -> (f64, f64) {
        let v = std::array::from_fn::<_, 3, _>(|d| position[d] - self.center[d]);
        let axial = v[0] * axis[0] + v[2] * axis[2];
        let end_distance = axial.abs() - self.half_length;
        if !(0.0..self.nozzle_depth).contains(&end_distance) {
            return (0.0, 0.0);
        }
        let across = -v[0] * axis[2] + v[2] * axis[0];
        let radial = (across * across + v[1] * v[1]) / (self.radius * self.radius);
        if radial >= 1.0 {
            return (0.0, 0.0);
        }
        // Zero at the rim, deliberately stronger on the upper side. Frozen over
        // one accepted interval, with separate pressure heads on the two ends.
        let profile = (1.0 - radial).powi(2) * (0.75 + 0.25 * v[1] / self.radius);
        let end = usize::from(axial < 0.0);
        let pressure = self.pressure_heads[end] * profile;
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
        for (d, values) in fields.into_iter().enumerate() {
            if d == 1 {
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
                    let (head, _) = self.sample(position, axis);
                    let result =
                        f64::from(*value) + dt * axis[d] * head / (density * self.nozzle_depth);
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
        let head = self.pressure_heads[0].max(self.pressure_heads[1]);
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
                    self.sample(grid.cell_position(p).expect("bounded emitter cell"), axis);
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
