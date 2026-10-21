//! Accepted carriers share the desktop's existing outer worker and recorder.
//! This is a per-interval dispatch boundary, never another simulation loop.
use crate::{Request, accepted_session::configured_track::ConfiguredTrack, history::PreviewFrame};
use rheon::{GridGeometry, PressureImplementation, Simulation, SmokeSource, StateView, StepReport};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

// A thumbnail has its own payload budget, independent of the former128-axis
// numerical control. The native192MiB array budget admits >128-axis carriers.
const GUIDANCE_PIXEL_BUDGET: usize = 1024 * 1024;
pub(crate) enum AdvanceError {
    Native(rheon::SimulationError),
    Session(rheon::SessionError),
    Unsupported,
}
impl std::fmt::Display for AdvanceError {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Native(error) => error.fmt(out),
            Self::Session(error) => error.fmt(out),
            Self::Unsupported => out.write_str(
                "Authored fields require native carrier execution; configured owner unsupported",
            ),
        }
    }
}
impl AdvanceError {
    pub(crate) fn termination_cause(
        &self,
    ) -> rheon::recording::provider::runtime::RuntimeTerminationCause {
        use rheon::recording::provider::runtime::RuntimeTerminationCause as Cause;
        use rheon::solver_provider::ProviderRejection;
        use rheon::{
            AdvectionError, LiquidStepError, LiquidVolumeError, PressureError, SessionError,
            SimulationError, ViscosityError,
        };
        match self {
            Self::Session(SessionError::Deadline) => Cause::SessionDeadline,
            Self::Native(
                SimulationError::Cancelled { .. }
                | SimulationError::Pressure(PressureError::Cancelled { .. })
                | SimulationError::Advection(AdvectionError::Cancelled)
                | SimulationError::Viscosity(ViscosityError::Cancelled { .. }),
            )
            | Self::Session(SessionError::PressureBinding(ProviderRejection::Cancelled))
            | Self::Session(SessionError::Step(
                LiquidStepError::Cancelled
                | LiquidStepError::Carrier(
                    SimulationError::Cancelled { .. }
                    | SimulationError::Pressure(PressureError::Cancelled { .. })
                    | SimulationError::Advection(AdvectionError::Cancelled)
                    | SimulationError::Viscosity(ViscosityError::Cancelled { .. }),
                )
                | LiquidStepError::Volume(LiquidVolumeError::Cancelled { .. }),
            )) => Cause::Cancelled,
            Self::Unsupported => Cause::StartupFailure,
            _ => Cause::NumericalFailure,
        }
    }
}

pub(crate) enum CarrierExecution {
    Native(Box<Simulation>),
    Configured(Box<ConfiguredTrack>),
}
impl CarrierExecution {
    pub(crate) fn native(&self) -> Result<&Simulation, String> {
        match self {
            Self::Native(sim) => Ok(sim),
            Self::Configured(_) => Err("Native carrier owner required".into()),
        }
    }
    pub(crate) fn configured(&self) -> Option<&ConfiguredTrack> {
        match self {
            Self::Configured(track) => Some(track),
            Self::Native(_) => None,
        }
    }
    pub(crate) fn configured_mut(&mut self) -> Option<&mut ConfiguredTrack> {
        match self {
            Self::Configured(track) => Some(track),
            Self::Native(_) => None,
        }
    }
    pub(crate) fn grid(&self) -> &GridGeometry {
        match self {
            Self::Native(sim) => sim.grid(),
            Self::Configured(track) => track.geometry(),
        }
    }
    pub(crate) fn state(&self) -> StateView<'_> {
        match self {
            Self::Native(sim) => sim.state(),
            Self::Configured(track) => track.snapshot().state().carrier,
        }
    }
    pub(crate) fn implementation(&self) -> PressureImplementation {
        match self {
            Self::Native(sim) => sim.implementation(),
            Self::Configured(track) => track.native_implementation(),
        }
    }
    pub(crate) fn bytes(&self) -> usize {
        match self {
            Self::Native(sim) => sim.allocated_bytes(),
            Self::Configured(track) => track.host_array_bytes().total(),
        }
    }
    pub(crate) fn preview(
        &self,
        edge: usize,
        step: usize,
        seconds: f64,
        report: Option<StepReport>,
    ) -> Result<Arc<PreviewFrame>, String> {
        match self {
            Self::Native(sim) => PreviewFrame::capture(sim, edge, step, seconds, report),
            Self::Configured(_) => PreviewFrame::capture_state(
                self.grid(),
                self.state(),
                self.implementation(),
                edge,
                step,
                seconds,
                report,
            ),
        }
    }
    pub(crate) fn advance(
        &mut self,
        request: &Request,
        source: Option<SmokeSource>,
        cancelled: &AtomicBool,
        fields: Option<&rheon::ForceSnapshot>,
    ) -> Result<StepReport, AdvanceError> {
        match self {
            Self::Native(sim) => {
                if let Some(fields) = fields {
                    return if let Some(emitter) = request.emitter {
                        sim.step_with_emitter_and_force_fields(
                            request.requested_dt,
                            emitter,
                            fields,
                            |_| cancelled.load(Ordering::Relaxed),
                        )
                    } else {
                        sim.step_with_force_fields(request.requested_dt, source, fields, |_| {
                            cancelled.load(Ordering::Relaxed)
                        })
                    }
                    .map(|report| report.step)
                    .map_err(AdvanceError::Native);
                }
                let result = if let Some(emitter) = request.emitter {
                    sim.step_with_emitter(request.requested_dt, emitter, |_| {
                        cancelled.load(Ordering::Relaxed)
                    })
                } else {
                    sim.step(request.requested_dt, source, |_| {
                        cancelled.load(Ordering::Relaxed)
                    })
                };
                result.map_err(AdvanceError::Native)
            }
            Self::Configured(_) if fields.is_some() => Err(AdvanceError::Unsupported),
            Self::Configured(track) => track
                .advance(None)
                .map(|report| report.carrier.step)
                .map_err(AdvanceError::Session),
        }
    }
    pub(crate) fn guidance(&self) -> Result<Vec<u8>, String> {
        match self {
            Self::Native(sim) => {
                rheon::guidance_pixels(sim, GUIDANCE_PIXEL_BUDGET).map_err(|e| e.to_string())
            }
            Self::Configured(_) => {
                // The declared closed profile starts with zero appearance tracer
                // and has no appearance source. Its tracer projection is zero;
                // actual velocity/pressure remain available in the Inspector.
                if self.state().tracer.iter().any(|v| *v != 0.0) {
                    return Err("Configured closed-profile appearance invariant".into());
                }
                let [nx, ny, _] = self.grid().counts();
                let count = nx
                    .checked_mul(ny)
                    .filter(|n| *n <= GUIDANCE_PIXEL_BUDGET)
                    .ok_or("Guidance pixel budget")?;
                Ok(vec![0; count])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn admitted_carrier_above_old_axis_limit_can_finish_its_thumbnail() {
        let grid = GridGeometry::new([129; 3], [0.001; 3], [0.0; 3]).unwrap();
        let sim = Simulation::new(
            grid,
            rheon::SimulationConfig {
                density: 1.0,
                memory_limit: 192 * 1024 * 1024,
                pressure: rheon::PressureSettings {
                    relative_residual: 1e-9,
                    absolute_residual: 1e-12,
                    divergence_limit: 1e-7,
                    max_iterations: 2000,
                },
                actual_divergence_limit: 1e-5,
                max_courant: 1.0,
            },
        )
        .unwrap();
        assert_eq!(
            CarrierExecution::Native(Box::new(sim))
                .guidance()
                .unwrap()
                .len(),
            129 * 129
        );
    }
}
