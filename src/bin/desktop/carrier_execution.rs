//! Accepted carriers share the desktop's existing outer worker and recorder.
//! This is a per-interval dispatch boundary, never another simulation loop.
use crate::{Request, accepted_session::configured_track::ConfiguredTrack, history::PreviewFrame};
use rheon::{GridGeometry, PressureImplementation, Simulation, SmokeSource, StateView, StepReport};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

pub(crate) enum AdvanceError {
    Native(rheon::SimulationError),
    Session(rheon::SessionError),
}
impl std::fmt::Display for AdvanceError {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Native(error) => error.fmt(out),
            Self::Session(error) => error.fmt(out),
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
    ) -> Result<StepReport, AdvanceError> {
        match self {
            Self::Native(sim) => {
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
            Self::Configured(track) => track
                .advance(None)
                .map(|report| report.carrier.step)
                .map_err(AdvanceError::Session),
        }
    }
    pub(crate) fn guidance(&self) -> Result<Vec<u8>, String> {
        match self {
            Self::Native(sim) => rheon::guidance_pixels(sim, 128 * 128).map_err(|e| e.to_string()),
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
                    .filter(|n| *n <= 128 * 128)
                    .ok_or("Guidance pixel budget")?;
                Ok(vec![0; count])
            }
        }
    }
}
