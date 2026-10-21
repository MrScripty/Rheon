//! Rheon's fixed-box smoke/tracer core and headless guidance export.
//! See docs/MILESTONE.md for checked contracts, memory accounting, executable
//! evidence, and the remaining GUI/performance/project limitations.
//!
//! The Lean indexing theorem uses unbounded natural numbers. This implementation
//! separately checks target-integer arithmetic and physical-coordinate ranges.
mod geometry;
mod observation;
mod parallel;
#[cfg(feature = "parallel")]
pub use observation::CalculationStage;
#[cfg(feature = "work-observation")]
pub use observation::{CalculationObserver, ObservationError, WorkerVoxel};
#[cfg(feature = "parallel")]
mod scheduling;
#[cfg(feature = "parallel")]
pub use scheduling::{BatchStatistics, CpuScheduler};
#[cfg(all(feature = "parallel", target_has_atomic = "64"))]
mod sgs;
pub use geometry::{
    Axis, BufferPlan, GeometryError, GridGeometry, MAX_CELLS_PER_METRE, MAX_GRID_EDGE,
};
mod collision;
pub mod mesh_collision;
pub use mesh_collision::{CollisionMesh, MeshCollisionError, MeshPolicy};
// Native filesystem import is unavailable in the wasm32 browser host.
#[cfg(not(target_arch = "wasm32"))]
pub mod mesh_import;
pub use collision::{
    ClippedSegment, HitFacing, SurfaceError, SurfaceHit, SurfaceSettings, SurfaceStamp,
    TriangleSurface,
};
mod static_obstacle;
pub use static_obstacle::{
    NO_FLUID_COMPONENT, ObstacleAllocation, ObstacleError, ObstacleFace, ObstacleStage,
    StaticObstacleGeometry,
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
#[cfg(feature = "accepted-pressure-warm-start")]
mod pressure_initialization;
#[cfg(feature = "accepted-pressure-warm-start")]
pub use pressure_initialization::{
    PressureColdReason, PressureInitializationPolicy, PressureInitializationReport,
    PressureSeedStage, PressureSeedStamp,
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
    AcceptedForceIntervalPressure, AcceptedIntervalPressure, CarrierStepControls, Simulation,
    SimulationConfig, SimulationError, SmokeSource, StateView, StepReport, StepStage,
};
pub mod force_field;
pub use force_field::{
    FORCE_SCENE_VERSION, ForceFieldError, ForceKind, ForceObject, ForceScene, ForceSnapshot,
    ForceSupport, ForceTime, ForceTransform, MAX_FORCE_OBJECTS, SceneObjectId,
};
mod forces;
pub use forces::{BodyForce, ForceRegion, ForceReport, ForceUnits, ForcedStepReport};
mod emitter;
pub use emitter::RotatingCylinderEmitter;
mod material;
pub use material::{DomainFill, FluidMaterial, FluidPhase, FluidPreset, ParticleFluidConfig};
mod particle_fluid;
pub use particle_fluid::{
    AcceptedParticleForceControls, FluidParticle, ParticleDiagnostic, ParticleFluidError,
    ParticleFluidSimulation, ParticleStepReport,
};
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

mod column_shear;
pub use column_shear::{
    ColumnBoundaryShearReport, ColumnForcedShearReport, ColumnShearBoundary, ColumnShearError,
    ColumnShearGeometry, ColumnShearInputs, ColumnShearReport, ColumnShearStage, ColumnShearWall,
    ColumnShearWorkspace, ColumnWallShearReport, UniformColumnForce,
};

mod column_momentum;
pub use column_momentum::{
    ColumnMomentumError, ColumnMomentumInputs, ColumnMomentumReport, ColumnMomentumStage,
    ColumnMomentumWorkspace,
};

mod column_mac;
pub use column_mac::{
    ColumnMacDirection, ColumnMacError, ColumnMacExportReport, ColumnMacProjectionReport,
    ColumnMacPublicationReport, ColumnMacPublishInputs, ColumnMacStage, ColumnMacStateStamp,
    ColumnMacTransferReport, ColumnMacWorkspace, FlatColumnMacGeometry,
};

mod fitted_height;
pub use fitted_height::{
    FittedHeightDualFace, FittedHeightError, FittedHeightGeometry, FittedHeightInputs,
    FittedHeightNode, FittedHeightNodeDiagnostic, FittedHeightPlan, FittedHeightPressureTerm,
    FittedHeightReport, FittedHeightSettings, FittedHeightStage, FittedHeightTriangle,
    FittedHeightVelocityRow, FittedHeightWorkspace,
};

mod translated_viscous;
pub use translated_viscous::{
    TranslatedViscousError, TranslatedViscousFlow, TranslatedViscousReport,
    TranslatedViscousSettings, TranslatedViscousStage, TranslatedViscousStamp,
    TranslatedViscousState,
};

mod fixed_bottom_ale;
pub use fixed_bottom_ale::{
    FixedBottomAleError, FixedBottomAleFlow, FixedBottomAleReport, FixedBottomAleStage,
    FixedBottomAleState, FixedBottomAleTransfer,
};

mod coupled_discrete;
pub use coupled_discrete::{
    CoupledDiscreteError, CoupledDiscreteFlow, CoupledDiscreteReport, CoupledDiscreteStage,
    CoupledDiscreteState, CoupledExtrudedReport, CoupledThirdReport,
};

mod obstacle_pressure;
pub use obstacle_pressure::{
    ObstacleFlowError, ObstacleFlowStage, ObstaclePressureReport, ObstacleProjectionLedger,
    StaticObstaclePressure,
};
mod obstacle_shear;
pub use obstacle_shear::{
    ObstacleShearForce, ObstacleShearReport, ObstacleShearWall, StaticObstacleShear,
};

mod aligned_strain;
pub use aligned_strain::{
    AlignedStrain, AlignedStrainBoundary, AlignedStrainError, AlignedStrainFace,
    AlignedStrainLedger, AlignedStrainRow, AlignedStrainTerm,
};
mod dense3d_sequence;
pub use dense3d_sequence::{Dense3dError, Dense3dLimits, Dense3dSequence, write_dense3d_frame};

mod session;
pub use session::{
    SESSION_CAPABILITIES, SessionCapabilities, SessionConfiguration, SessionError, SessionIdentity,
    SessionSnapshot, SimulationSession, SnapshotStage,
};

mod material_session;
pub use material_session::{
    MATERIAL_SESSION_CAPABILITIES, MaterialEmitterControl, MaterialSession,
    MaterialSessionCapabilities, MaterialSessionError, MaterialSnapshot, MaterialStepControls,
    ParticleEvaluationStage,
};
/// Optional learned pressure components with reference fallback.
pub mod solver_provider;

mod pressure_session;
pub use pressure_session::{
    PRESSURE_SESSION_CAPABILITIES, PressureSession, PressureSessionCapabilities,
    PressureSessionError, PressureSolveControls, QualifiedPressureSnapshot,
};
mod vtk_export;
pub use vtk_export::{
    write_carrier_vti, write_material_vtp, write_particles_vtp, write_session_vti,
};

mod provider_step;
pub use provider_step::{ProviderStepReport, ProviderStepWorkspace};

mod emitter_controls;
pub use emitter_controls::{EMITTER_CONTROLS_VERSION, EmitterTurbulence};

/// Native accepted-publication archive; inspection data, not restart checkpoints.
#[cfg(not(target_arch = "wasm32"))]
pub mod recording;

mod compressible;
pub use compressible::{
    COMPRESSIBLE_METHOD, CompressibleCause, CompressibleCheckpoint, CompressibleDiagnostics,
    CompressibleError, CompressibleFailure, CompressiblePrimitive, CompressibleSimulation,
    CompressibleStage, CompressibleStepReport, CompressibleView, ConservativeState, IdealGas,
};
