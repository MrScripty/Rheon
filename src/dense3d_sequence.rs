//! Bounded streaming of constructor and successful accepted liquid states.
//! I/O follows physics publication; an output error poisons export, never physics.
use crate::{
    BoxFluxStepBoundary, BoxFluxStepWorkspace, GridGeometry, LiquidStepError, LiquidStepInputs,
    LiquidStepReport, LiquidStepStage, LiquidTransportSimulation, LiquidTransportView,
};
use std::{
    fmt,
    io::{self, Write},
};

#[derive(Debug, Clone, Copy)]
pub struct Dense3dLimits {
    pub max_cells: usize,
    /// Includes the constructor frame.
    pub max_frames: usize,
    /// Total encoded JSONL bytes, including constructor and newlines.
    pub max_bytes: usize,
}
#[derive(Debug)]
pub enum Dense3dError {
    InvalidState(&'static str),
    Limit,
    Aborted,
    Step(LiquidStepError),
    Io(io::Error),
}
impl fmt::Display for Dense3dError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "dense3D export: {self:?}")
    }
}
impl std::error::Error for Dense3dError {}
impl From<io::Error> for Dense3dError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

/// Owns the simulation to enforce one output per accepted publication. Errors are
/// terminal; state() can still inspect physics after an output failure. No resume.
pub struct Dense3dSequence<W: Write> {
    simulation: LiquidTransportSimulation,
    output: Counted<W>,
    limits: Dense3dLimits,
    frames: usize,
    aborted: bool,
}
struct Counted<W> {
    inner: W,
    bytes: usize,
    limit: usize,
}
impl<W: Write> Write for Counted<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.bytes) {
            return Err(io::Error::other("dense3D byte budget exhausted"));
        }
        let n = self.inner.write(bytes)?;
        self.bytes += n;
        Ok(n)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}
impl<W: Write> Dense3dSequence<W> {
    pub fn new(
        simulation: LiquidTransportSimulation,
        output: W,
        limits: Dense3dLimits,
    ) -> Result<Self, Dense3dError> {
        if limits.max_frames == 0
            || limits.max_bytes == 0
            || simulation.grid().cell_len() > limits.max_cells
        {
            return Err(Dense3dError::Limit);
        }
        let state = simulation.state();
        if state.carrier.time != 0.0 || state.carrier.generation != 0 || state.liquid.time != 0.0 {
            return Err(Dense3dError::InvalidState("constructor required"));
        }
        let mut sequence = Self {
            simulation,
            output: Counted {
                inner: output,
                bytes: 0,
                limit: limits.max_bytes,
            },
            limits,
            frames: 0,
            aborted: false,
        };
        write_dense3d_frame(
            &mut sequence.output,
            sequence.simulation.grid(),
            sequence.simulation.state(),
            0,
            None,
        )?;
        sequence.frames = 1;
        Ok(sequence)
    }
    pub fn state(&self) -> LiquidTransportView<'_> {
        self.simulation.state()
    }
    pub fn grid(&self) -> &GridGeometry {
        self.simulation.grid()
    }
    pub fn frame_count(&self) -> usize {
        self.frames
    }
    pub fn bytes_written(&self) -> usize {
        self.output.bytes
    }
    pub fn allocated_bytes(&self) -> usize {
        self.simulation.allocated_bytes()
    }
    pub fn is_aborted(&self) -> bool {
        self.aborted
    }
    /// Frame count is preflighted before physics. Step rejection and any writer
    /// error permanently abort export. A partial trailing line is possible.
    pub fn step_with_box_flux<F: FnMut(LiquidStepStage) -> bool>(
        &mut self,
        inputs: LiquidStepInputs<'_>,
        workspace: &mut BoxFluxStepWorkspace,
        boundary: BoxFluxStepBoundary,
        cancel: F,
    ) -> Result<LiquidStepReport, Dense3dError> {
        if self.aborted {
            return Err(Dense3dError::Aborted);
        }
        if self.frames >= self.limits.max_frames {
            self.aborted = true;
            return Err(Dense3dError::Limit);
        }
        // Reserve an upper bound before physics; decimal scientific formatting
        // uses fewer than 32 bytes per scalar, and metadata fewer than 4096.
        let grid = self.simulation.grid();
        let scalars = [crate::Axis::X, crate::Axis::Y, crate::Axis::Z]
            .into_iter()
            .fold(grid.cell_len().checked_mul(3), |n, a| {
                n.and_then(|n| n.checked_add(grid.face_len(a)))
            });
        let bound = scalars
            .and_then(|n| n.checked_mul(32))
            .and_then(|n| n.checked_add(4096));
        if bound.is_none_or(|n| n > self.limits.max_bytes.saturating_sub(self.output.bytes)) {
            self.aborted = true;
            return Err(Dense3dError::Limit);
        }
        let report = match self
            .simulation
            .step_with_box_flux(inputs, workspace, boundary, cancel)
        {
            Ok(report) => report,
            Err(e) => {
                self.aborted = true;
                return Err(Dense3dError::Step(e));
            }
        };
        if let Err(e) = write_dense3d_frame(
            &mut self.output,
            self.simulation.grid(),
            self.simulation.state(),
            self.frames,
            Some(&report),
        ) {
            self.aborted = true;
            return Err(e);
        }
        self.frames += 1;
        Ok(report)
    }
    /// Flush is terminal on failure. Filesystem callers must also sync their file
    /// before publishing a completion manifest, which is outside this writer.
    pub fn flush(&mut self) -> Result<(), Dense3dError> {
        if self.aborted {
            return Err(Dense3dError::Aborted);
        }
        if let Err(e) = self.output.flush() {
            self.aborted = true;
            return Err(e.into());
        }
        Ok(())
    }
}
fn invalid(condition: bool, message: &'static str) -> Result<(), Dense3dError> {
    if condition {
        Ok(())
    } else {
        Err(Dense3dError::InvalidState(message))
    }
}
/// Serialize only the supplied immutable accepted view. Native floats use
/// scientific formatting with 10/f32 and 18/f64 significant digits. Preflight
/// rejects nonfinite values and malformed views before writing any bytes.
pub fn write_dense3d_frame<W: Write>(
    out: &mut W,
    grid: &GridGeometry,
    state: LiquidTransportView<'_>,
    frame: usize,
    report: Option<&LiquidStepReport>,
) -> Result<(), Dense3dError> {
    let axes = [crate::Axis::X, crate::Axis::Y, crate::Axis::Z];
    let velocities = [state.carrier.x, state.carrier.y, state.carrier.z];
    invalid(
        state.pressure_surface.is_none()
            && state.pressure_columns.is_none()
            && state.reconstructed_surface.is_none()
            && !state.flat_column_mac,
        "transport-only view required",
    )?;
    for (axis, field) in axes.into_iter().zip(velocities) {
        invalid(field.len() == grid.face_len(axis), "face shape")?;
        invalid(field.iter().all(|v| v.is_finite()), "nonfinite velocity")?;
    }
    invalid(
        state.carrier.tracer.len() == grid.cell_len()
            && state.liquid.fraction.len() == grid.cell_len()
            && state.pressure.len() == grid.cell_len(),
        "cell shape",
    )?;
    invalid(
        state.carrier.tracer.iter().all(|v| v.is_finite())
            && state.pressure.iter().all(|v| v.is_finite())
            && state
                .liquid
                .fraction
                .iter()
                .all(|v| v.is_finite() && (0.0..=1.0).contains(v)),
        "nonfinite or invalid cell field",
    )?;
    invalid(
        state.carrier.time.is_finite()
            && state.carrier.time == state.liquid.time
            && state.carrier_stamp.version == state.carrier.generation,
        "state clock/stamp",
    )?;
    if let Some(r) = report {
        let c = r.carrier.step;
        let l = r.liquid;
        invalid(
            c.time == state.carrier.time
                && l.time == c.time
                && c.dt == l.dt
                && c.dt > 0.0
                && c.generation == state.carrier.generation
                && l.stamp == state.liquid.stamp
                && l.flow == state.carrier_stamp,
            "report continuity",
        )?;
        invalid(
            [
                c.dt,
                c.pressure.true_residual_max,
                c.actual_divergence_max,
                l.actual_divergence_max,
                l.liquid_volume_before,
                l.liquid_volume_after,
                l.liquid_mass_after,
                l.inward_boundary_volume,
                l.outward_boundary_volume,
                l.source_volume,
                l.volume_balance_error,
                l.volume_rounding_budget,
            ]
            .into_iter()
            .all(f64::is_finite),
            "nonfinite report",
        )?;
    } else {
        invalid(
            frame == 0 && state.carrier.time == 0.0 && state.carrier.generation == 0,
            "initial frame",
        )?;
    }
    write!(
        out,
        "{{\"frame\":{frame},\"time_s\":{:.17e},\"dt_s\":{:.17e},\"carrier_stamp\":{{\"id\":\"{}\",\"version\":\"{}\"}},\"liquid_stamp\":{{\"id\":\"{}\",\"version\":\"{}\"}},\"fields\":{{",
        state.carrier.time,
        report.map_or(0.0, |r| r.carrier.step.dt),
        state.carrier_stamp.id,
        state.carrier_stamp.version,
        state.liquid.stamp.id,
        state.liquid.stamp.version
    )?;
    for (name, values) in [
        ("velocity_x", state.carrier.x),
        ("velocity_y", state.carrier.y),
        ("velocity_z", state.carrier.z),
        ("tracer", state.carrier.tracer),
    ] {
        write!(out, "\"{name}\":[")?;
        for (i, v) in values.iter().enumerate() {
            if i != 0 {
                write!(out, ",")?;
            }
            write!(out, "{v:.9e}")?;
        }
        write!(out, "],")?;
    }
    for (i, (name, values)) in [
        ("fraction", state.liquid.fraction),
        ("pressure", state.pressure),
    ]
    .into_iter()
    .enumerate()
    {
        if i != 0 {
            write!(out, ",")?;
        }
        write!(out, "\"{name}\":[")?;
        for (i, v) in values.iter().enumerate() {
            if i != 0 {
                write!(out, ",")?;
            }
            write!(out, "{v:.17e}")?;
        }
        write!(out, "]")?;
    }
    write!(out, "}},\"diagnostics\":")?;
    if let Some(r) = report {
        let c = r.carrier.step;
        let l = r.liquid;
        write!(
            out,
            "{{\"pressure_iterations\":{},\"pressure_residual_m3_s2\":{:.17e},\"carrier_divergence_s_inv\":{:.17e},\"liquid_divergence_s_inv\":{:.17e},\"volume_before_m3\":{:.17e},\"volume_after_m3\":{:.17e},\"mass_after_kg\":{:.17e},\"inward_m3\":{:.17e},\"outward_m3\":{:.17e},\"source_m3\":{:.17e},\"balance_m3\":{:.17e},\"rounding_budget_m3\":{:.17e}}}",
            c.pressure.iterations,
            c.pressure.true_residual_max,
            c.actual_divergence_max,
            l.actual_divergence_max,
            l.liquid_volume_before,
            l.liquid_volume_after,
            l.liquid_mass_after,
            l.inward_boundary_volume,
            l.outward_boundary_volume,
            l.source_volume,
            l.volume_balance_error,
            l.volume_rounding_budget
        )?;
    } else {
        write!(out, "null")?;
    }
    writeln!(out, "}}")?;
    Ok(())
}
