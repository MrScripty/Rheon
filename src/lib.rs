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
mod free_surface;
pub use free_surface::{FreeSurfaceError, SlabFreeSurface};
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
pub use export::{
    ExportError, guidance_pixels, liquid_guidance_pixels, write_guidance_png,
    write_liquid_guidance_png,
};

mod box_flux;
pub use box_flux::{
    BoxFluxError, BoxFluxReport, BoxFluxSettings, BoxFluxStage, BoxFluxStamp, BoxFluxWork,
    BoxFluxWorkspace, PrescribedBoxFlux,
};

mod box_flux_step;
pub use box_flux_step::{
    BoxFluxStepBoundary, BoxFluxStepReport, BoxFluxStepWork, BoxFluxStepWorkspace,
    BoxFluxTracerPolicy,
};

mod liquid_volume;
pub use liquid_volume::{
    LiquidFlowInterval, LiquidInlet, LiquidOccupancy, LiquidVolumeError, LiquidVolumeReport,
    LiquidVolumeSettings, LiquidVolumeSource, LiquidVolumeState, LiquidVolumeView,
    VolumeDivergenceDomain, VolumeStage, VolumeStamp,
};

mod liquid_step;
pub use liquid_step::{
    LiquidStepError, LiquidStepInputs, LiquidStepReport, LiquidStepStage, LiquidTransportConfig,
    LiquidTransportSimulation, LiquidTransportView,
};

mod column_surface;
pub use column_surface::{
    ColumnHeight, ColumnReconstructionReport, ColumnSurfaceView, ColumnSurfaceWorkspace,
    ReconstructionStage,
};

pub use column_surface::{ColumnVolumeInputs, ColumnVolumeReport, ColumnVolumeStage};

mod viscosity;
pub use viscosity::{ViscosityError, ViscosityReport, ViscosityStage, ViscosityWorkspace};
