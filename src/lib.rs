//! Rheon's fixed-box smoke/tracer core and headless guidance export.
//! See docs/MILESTONE.md for checked contracts, memory accounting, executable
//! evidence, and the remaining GUI/performance/project limitations.
//!
//! The Lean indexing theorem uses unbounded natural numbers. This implementation
//! separately checks target-integer arithmetic and physical-coordinate ranges.
mod geometry;
pub use geometry::{Axis, BufferPlan, GeometryError, GridGeometry};
mod collision;
pub use collision::{
    ClippedSegment, HitFacing, SurfaceError, SurfaceHit, SurfaceSettings, SurfaceStamp,
    TriangleSurface,
};
mod motion;
pub use motion::{TranslatedHit, TranslatedSegment, TranslationError, TranslationInterval};
mod operator;
pub use operator::{OperatorError, PressureOperator};
mod pressure;
pub use pressure::{
    PressureError, PressureImplementation, PressureReport, PressureSettings, PressureWorkspace,
};
mod sampling;
pub use sampling::{SamplingError, ScalarSampler, VelocitySampler};
mod advection;
pub use advection::{AdvectionError, advect_tracer, advect_velocity};
mod tracer_barrier;
pub use tracer_barrier::{
    BarrierStepReport, TracerBarrierError, TracerBarrierReport, TracerBarrierSampler,
    VisibleTracerSample, advect_tracer_with_barrier,
};
mod simulation;
pub use simulation::{
    Simulation, SimulationConfig, SimulationError, SmokeSource, StateView, StepReport, StepStage,
};
mod forces;
pub use forces::{BodyForce, ForceRegion, ForceReport, ForceUnits, ForcedStepReport};
#[cfg(feature = "png-export")]
mod export;
#[cfg(feature = "png-export")]
pub use export::{ExportError, guidance_pixels, write_guidance_png};
