//! Closed accepted-Session intent, distinct from native carrier schema one.
//! This module captures constructor facts and future declarations. It never
//! loads a model, advances time, restores state, or infers a recording outcome.
//! Archive binding awaits the recording owner's concrete typed codec.
use super::{
    ArchiveBudget, FinalizedArchive, MAX_MANIFEST_BYTES, MAX_MEMBERS, NativePressureMethod,
    NativeTolerances, RunSetError, TerminalOutcome, digest, publish_new, read_versioned,
    relative_path, text,
};
use rheon::{
    BodyForce, ForceRegion, ForceUnits, GridGeometry, LiquidTransportConfig, LiquidVolumeSettings,
    PressureImplementation, PressureSettings, SessionConfiguration, SessionIdentity,
    SessionSnapshot, SimulationConfig, SimulationSession, SnapshotStage, VolumeStamp,
    solver_provider::configured::ContextProviderIdentity,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const CONFIGURED_VERSION: u32 = 2;
const MAX_CELLS: usize = 65_536;
const MAX_INTERVALS: usize = 10_000;
const MAX_FORCES: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfiguredSpecKind {
    ConfiguredAcceptedSession,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfiguredSetKind {
    ConfiguredAcceptedSessionSet,
}
fn version(value: u32) -> Result<(), RunSetError> {
    if value == CONFIGURED_VERSION {
        Ok(())
    } else {
        Err(RunSetError::Unsupported("configured manifest version"))
    }
}
fn same<T: Serialize>(a: &T, b: &T) -> Result<bool, RunSetError> {
    // Unlike derived float PartialEq, this preserves signed zero in frozen
    // finite constructor/intent values, matching the persisted digest framing.
    Ok(serde_json::to_vec(a)? == serde_json::to_vec(b)?)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutorDeclaration {
    pub id: String,
    pub source_revision: String,
    /// Absent means unspecified, never an inferred build/toolchain attestation.
    pub build_provenance: Option<String>,
}
impl ExecutorDeclaration {
    pub fn validate(&self) -> Result<(), RunSetError> {
        text(&self.id)?;
        text(&self.source_revision)?;
        if let Some(build) = &self.build_provenance {
            text(build)?;
        }
        Ok(())
    }
}
/// Exact declared model/runtime custody; independent of the host executor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfiguredProviderPin {
    pub id: String,
    pub version: String,
    pub manifest_sha256: [u8; 32],
    pub checkpoint_sha256: Option<[u8; 32]>,
    pub training_run_id: Option<String>,
    pub runtime: String,
    pub source_revision: String,
}
impl ConfiguredProviderPin {
    pub fn capture(identity: &ContextProviderIdentity) -> Result<Self, RunSetError> {
        identity
            .validate()
            .map_err(|_| RunSetError::Invalid("configured provider identity"))?;
        Ok(Self {
            id: identity.id.clone(),
            version: identity.version.clone(),
            manifest_sha256: identity.manifest_sha256,
            checkpoint_sha256: identity.checkpoint_sha256,
            training_run_id: identity.training_run_id.clone(),
            runtime: identity.runtime.clone(),
            source_revision: identity.source_revision.clone(),
        })
    }
    pub fn identity(&self) -> ContextProviderIdentity {
        ContextProviderIdentity {
            id: self.id.clone(),
            version: self.version.clone(),
            manifest_sha256: self.manifest_sha256,
            checkpoint_sha256: self.checkpoint_sha256,
            training_run_id: self.training_run_id.clone(),
            runtime: self.runtime.clone(),
            source_revision: self.source_revision.clone(),
        }
    }
    pub fn validate(&self) -> Result<(), RunSetError> {
        self.identity()
            .validate()
            .map_err(|_| RunSetError::Invalid("configured provider identity"))
    }
    /// A runtime must additionally validate actual files before executing code.
    pub fn validate_resolved_identity(
        &self,
        actual: &ContextProviderIdentity,
    ) -> Result<(), RunSetError> {
        self.validate()?;
        actual
            .validate()
            .map_err(|_| RunSetError::Invalid("resolved provider identity"))?;
        if self.identity() == *actual {
            Ok(())
        } else {
            Err(RunSetError::Integrity(
                "resolved provider differs from frozen pin",
            ))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForceUnitsSpec {
    Acceleration,
    ForceDensity,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ForceRegionSpec {
    pub lower_m: [f64; 3],
    pub upper_m: [f64; 3],
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ForceSpec {
    pub value: [f64; 3],
    pub units: ForceUnitsSpec,
    pub region: Option<ForceRegionSpec>,
}
impl ForceSpec {
    pub fn body_force(&self) -> BodyForce {
        BodyForce {
            value: self.value,
            units: match self.units {
                ForceUnitsSpec::Acceleration => ForceUnits::Acceleration,
                ForceUnitsSpec::ForceDensity => ForceUnits::ForceDensity,
            },
            region: self.region.as_ref().map(|r| ForceRegion {
                lower: r.lower_m,
                upper: r.upper_m,
            }),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VolumeControls {
    pub max_outward_courant: f64,
    pub actual_divergence_per_s: f64,
}
impl VolumeControls {
    pub fn settings(self) -> LiquidVolumeSettings {
        LiquidVolumeSettings {
            max_outward_courant: self.max_outward_courant,
            actual_divergence_limit: self.actual_divergence_per_s,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ViscosityIntent {
    Disabled,
    NewtonianConstant { dynamic_viscosity_pa_s: f64 },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "path",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum DeploymentLocator {
    ProjectRelative(String),
    Absolute(String),
}
impl DeploymentLocator {
    pub fn validate(&self) -> Result<(), RunSetError> {
        match self {
            Self::ProjectRelative(path) => relative_path(path)
                .map_err(|_| RunSetError::Invalid("project-relative deployment locator")),
            Self::Absolute(path) => {
                text(path)?;
                if Path::new(path).is_absolute() {
                    Ok(())
                } else {
                    Err(RunSetError::Invalid("absolute deployment locator"))
                }
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PressureIntent {
    NativeOnly,
    NativePrefixThenConfigured {
        provider: Box<ConfiguredProviderPin>,
        /// Runtime-owned resolution/pin checking. Cold open never resolves it.
        deployment: DeploymentLocator,
        requested_native_prefix_steps: usize,
        minimum_native_prefix_steps: usize,
    },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfiguredFutureIntent {
    pub requested_dt_s: f64,
    pub requested_steps: usize,
    /// Constant body-force recipe for each requested interval; no phase source.
    pub forces: Vec<ForceSpec>,
    pub volume: VolumeControls,
    pub viscosity: ViscosityIntent,
    pub pressure: PressureIntent,
    pub provider_workspace_limit_bytes: usize,
    /// Exactly None when viscosity is Disabled; no absent-as-zero convention.
    pub viscosity_workspace_limit_bytes: Option<usize>,
    pub recording: ArchiveBudget,
    /// Whether future recording includes the constructor publication.
    pub capture_constructor: bool,
    pub executor: ExecutorDeclaration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdentityStamp {
    pub id: u64,
    pub version: u64,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfiguredConstructor {
    pub counts_xyz: [u64; 3],
    pub spacing_m: [f64; 3],
    pub origin_m: [f64; 3],
    pub density_kg_m3: f64,
    pub represented_density_kg_m3: f64,
    pub pressure_method: NativePressureMethod,
    pub tolerances: NativeTolerances,
    pub simulation_memory_limit_bytes: usize,
    pub volume_stamp: IdentityStamp,
    pub carrier_id: u64,
    pub boundary_workspace_limit_bytes: usize,
    pub max_steps: usize,
}
impl ConfiguredConstructor {
    fn capture(snapshot: &SessionSnapshot<'_>) -> Result<Self, RunSetError> {
        let state = snapshot.state();
        if snapshot.grid().cell_len() > MAX_CELLS
            || snapshot.stage() != SnapshotStage::Constructor
            || snapshot.frame() != 0
            || state.carrier.generation != 0
            || state.carrier.time.to_bits() != 0.0f64.to_bits()
            || state.liquid.time.to_bits() != 0.0f64.to_bits()
            || snapshot.accepted_interval_start().is_some()
            || snapshot.report().is_some()
            || snapshot.provider_report().is_some()
            || snapshot.configured_provider_report().is_some()
            || state.pressure_surface.is_some()
            || state.reconstructed_surface.is_some()
            || state.pressure_columns.is_some()
            || state.flat_column_mac
            || state
                .carrier
                .x
                .iter()
                .chain(state.carrier.y)
                .chain(state.carrier.z)
                .chain(state.carrier.tracer)
                .any(|v| v.to_bits() != 0.0f32.to_bits())
            || state
                .pressure
                .iter()
                .any(|v| v.to_bits() != 0.0f64.to_bits())
            || state
                .liquid
                .fraction
                .iter()
                .any(|v| v.to_bits() != 1.0f64.to_bits())
        {
            return Err(RunSetError::Invalid(
                "configured intent requires sealed full constructor rest",
            ));
        }
        let SessionConfiguration {
            transport,
            implementation,
            boundary_workspace_limit,
            max_steps,
        } = *snapshot.configuration();
        let c = transport.carrier;
        let t = c.pressure;
        Ok(Self {
            counts_xyz: snapshot.grid().counts().map(|n| n as u64),
            spacing_m: snapshot.grid().spacing(),
            origin_m: snapshot.grid().origin(),
            density_kg_m3: c.density,
            represented_density_kg_m3: transport.represented_density,
            pressure_method: match implementation {
                PressureImplementation::JacobiPcgV1 => NativePressureMethod::JacobiPcgV1,
                PressureImplementation::SymmetricGaussSeidelPcgV1 => {
                    NativePressureMethod::SymmetricGaussSeidelPcgV1
                }
            },
            tolerances: NativeTolerances {
                relative_residual: t.relative_residual,
                absolute_residual_m3_s2: t.absolute_residual,
                predicted_divergence_per_s: t.divergence_limit,
                actual_divergence_per_s: c.actual_divergence_limit,
                max_courant: c.max_courant,
                max_pressure_iterations: t.max_iterations,
            },
            simulation_memory_limit_bytes: c.memory_limit,
            volume_stamp: IdentityStamp {
                id: transport.volume_stamp.id,
                version: transport.volume_stamp.version,
            },
            carrier_id: transport.carrier_id,
            boundary_workspace_limit_bytes: boundary_workspace_limit,
            max_steps,
        })
    }
    pub fn grid(&self) -> Result<GridGeometry, RunSetError> {
        GridGeometry::new(self.counts_xyz, self.spacing_m, self.origin_m)
            .map_err(|_| RunSetError::Invalid("configured geometry"))
    }
    pub fn transport(&self) -> LiquidTransportConfig {
        let t = &self.tolerances;
        LiquidTransportConfig {
            carrier: SimulationConfig {
                density: self.density_kg_m3,
                memory_limit: self.simulation_memory_limit_bytes,
                pressure: PressureSettings {
                    relative_residual: t.relative_residual,
                    absolute_residual: t.absolute_residual_m3_s2,
                    divergence_limit: t.predicted_divergence_per_s,
                    max_iterations: t.max_pressure_iterations,
                },
                actual_divergence_limit: t.actual_divergence_per_s,
                max_courant: t.max_courant,
            },
            represented_density: self.represented_density_kg_m3,
            volume_stamp: VolumeStamp {
                id: self.volume_stamp.id,
                version: self.volume_stamp.version,
            },
            carrier_id: self.carrier_id,
        }
    }
    fn validate(&self) -> Result<(), RunSetError> {
        let grid = self.grid()?;
        let t = &self.tolerances;
        if grid.cell_len() > MAX_CELLS
            || !self.density_kg_m3.is_finite()
            || self.density_kg_m3 <= 0.0
            || self.density_kg_m3.to_bits() != self.represented_density_kg_m3.to_bits()
            || self.simulation_memory_limit_bytes == 0
            || self.boundary_workspace_limit_bytes == 0
            || self.max_steps == 0
            || [
                t.relative_residual,
                t.absolute_residual_m3_s2,
                t.predicted_divergence_per_s,
                t.actual_divergence_per_s,
            ]
            .iter()
            .any(|v| !v.is_finite() || *v < 0.0)
            || !t.max_courant.is_finite()
            || t.max_courant <= 0.0
            || t.max_courant > 1.0
            || t.max_pressure_iterations == 0
        {
            return Err(RunSetError::Invalid("configured constructor controls"));
        }
        Ok(())
    }
}
impl ConfiguredFutureIntent {
    fn validate(&self, constructor: &ConfiguredConstructor) -> Result<(), RunSetError> {
        self.executor.validate()?;
        self.recording.limits().validate()?;
        if !self.requested_dt_s.is_normal()
            || self.requested_dt_s <= 0.0
            || self.requested_steps == 0
            || self.requested_steps > MAX_INTERVALS
            || self.requested_steps > constructor.max_steps
            || !(self.requested_dt_s * self.requested_steps as f64).is_finite()
            || self.provider_workspace_limit_bytes == 0
            || self.recording.max_records > MAX_INTERVALS + 1
            || self.forces.len() > MAX_FORCES
            || !self.volume.max_outward_courant.is_finite()
            || self.volume.max_outward_courant <= 0.0
            || self.volume.max_outward_courant > 1.0
            || !self.volume.actual_divergence_per_s.is_finite()
            || self.volume.actual_divergence_per_s < 0.0
        {
            return Err(RunSetError::Invalid("configured future controls"));
        }
        match self.viscosity {
            ViscosityIntent::Disabled if self.viscosity_workspace_limit_bytes.is_none() => {}
            ViscosityIntent::NewtonianConstant {
                dynamic_viscosity_pa_s: mu,
            } if (mu == 0.0 || mu.is_normal())
                && mu >= 0.0
                && self
                    .viscosity_workspace_limit_bytes
                    .is_some_and(|cap| cap > 0) => {}
            _ => return Err(RunSetError::Invalid("viscosity stage/cap contract")),
        }
        if let PressureIntent::NativePrefixThenConfigured {
            provider,
            deployment,
            requested_native_prefix_steps,
            minimum_native_prefix_steps,
        } = &self.pressure
        {
            provider.validate()?;
            deployment.validate()?;
            if *minimum_native_prefix_steps > 512
                || requested_native_prefix_steps < minimum_native_prefix_steps
                || *requested_native_prefix_steps >= self.requested_steps
            {
                return Err(RunSetError::Invalid("configured native prefix policy"));
            }
        }
        let g = constructor.grid()?;
        for force in &self.forces {
            if force.value.iter().any(|v| !v.is_finite())
                || force.region.as_ref().is_some_and(|r| {
                    (0..3).any(|d| {
                        !r.lower_m[d].is_finite()
                            || !r.upper_m[d].is_finite()
                            || r.lower_m[d] >= r.upper_m[d]
                            || r.lower_m[d] < g.origin()[d]
                            || r.upper_m[d] > g.upper()[d]
                    })
                })
            {
                return Err(RunSetError::Invalid("configured force recipe"));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenConfiguredSpec {
    pub schema_version: u32,
    pub kind: ConfiguredSpecKind,
    pub sha256: [u8; 32],
    pub constructor: ConfiguredConstructor,
    pub future: ConfiguredFutureIntent,
}
impl FrozenConfiguredSpec {
    pub fn capture_constructor(
        snapshot: &SessionSnapshot<'_>,
        future: ConfiguredFutureIntent,
    ) -> Result<Self, RunSetError> {
        if snapshot.identity().source_revision != future.executor.source_revision {
            return Err(RunSetError::Integrity(
                "constructor host source declaration",
            ));
        }
        Self::new(ConfiguredConstructor::capture(snapshot)?, future)
    }
    /// Create a new frozen declaration; only capture_constructor checks an
    /// actual owner. A document/hash alone cannot prove an execution occurred.
    pub fn new(
        constructor: ConfiguredConstructor,
        future: ConfiguredFutureIntent,
    ) -> Result<Self, RunSetError> {
        constructor.validate()?;
        future.validate(&constructor)?;
        let mut result = Self {
            schema_version: CONFIGURED_VERSION,
            kind: ConfiguredSpecKind::ConfiguredAcceptedSession,
            sha256: [0; 32],
            constructor,
            future,
        };
        result.sha256 = result.payload_digest()?;
        Ok(result)
    }
    fn payload_digest(&self) -> Result<[u8; 32], RunSetError> {
        digest(&(
            self.schema_version,
            self.kind,
            &self.constructor,
            &self.future,
        ))
    }
    pub fn validate(&self) -> Result<(), RunSetError> {
        version(self.schema_version)?;
        self.constructor.validate()?;
        self.future.validate(&self.constructor)?;
        if self.sha256 != self.payload_digest()? {
            return Err(RunSetError::Integrity("configured spec digest"));
        }
        Ok(())
    }
    pub fn digest(&self) -> Result<[u8; 32], RunSetError> {
        self.validate()?;
        Ok(self.sha256)
    }
    /// Historical open/validation never performs this current-executor check.
    /// The execution owner must also resolve and verify any package pin before
    /// model code, then drive the existing Session from this same intent.
    pub fn validate_execution(
        &self,
        current_executor: &ExecutorDeclaration,
    ) -> Result<(), RunSetError> {
        self.validate()?;
        current_executor.validate()?;
        if self.future.executor != *current_executor {
            return Err(RunSetError::Unsupported("captured host executor differs"));
        }
        Ok(())
    }
    pub fn validate_constructor(&self, snapshot: &SessionSnapshot<'_>) -> Result<(), RunSetError> {
        self.validate()?;
        if snapshot.identity().source_revision != self.future.executor.source_revision
            || !same(
                &self.constructor,
                &ConfiguredConstructor::capture(snapshot)?,
            )?
        {
            return Err(RunSetError::Integrity(
                "configured constructor/spec binding",
            ));
        }
        Ok(())
    }
    /// Construct fresh rest state only. This neither loads a provider nor runs
    /// an interval; source checks for actual execution remain explicit above.
    pub fn create_session(
        &self,
        fresh_identity: SessionIdentity,
    ) -> Result<SimulationSession, RunSetError> {
        self.validate()?;
        if fresh_identity.source_revision != self.future.executor.source_revision {
            return Err(RunSetError::Integrity("fresh Session host source differs"));
        }
        let g = self.constructor.grid()?;
        let n = g.cell_len();
        SimulationSession::new(
            fresh_identity,
            g,
            self.constructor.transport(),
            self.constructor.pressure_method.implementation(),
            vec![1.0; n],
            self.constructor.boundary_workspace_limit_bytes,
            self.constructor.max_steps,
        )
        .map_err(|_| RunSetError::Invalid("Session rejected configured constructor"))
    }
    pub fn save_new(&self, path: &Path) -> Result<(), RunSetError> {
        self.validate()?;
        publish_new(path, &serde_json::to_vec(self)?, MAX_MANIFEST_BYTES)
    }
    pub fn open(path: &Path) -> Result<Self, RunSetError> {
        let result: Self = read_versioned(path, CONFIGURED_VERSION, MAX_MANIFEST_BYTES)?;
        result.validate()?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfiguredSpecDifference {
    Constructor,
    RequestedIntervals,
    Forces,
    VolumeControls,
    Viscosity,
    PressureSelection,
    Workspaces,
    RecordingBudget,
    ConstructorCapture,
    HostExecutor,
}
impl ConfiguredSpecDifference {
    pub fn between(
        a: &FrozenConfiguredSpec,
        b: &FrozenConfiguredSpec,
    ) -> Result<Vec<Self>, RunSetError> {
        a.validate()?;
        b.validate()?;
        let (x, y) = (&a.future, &b.future);
        let mut result = Vec::new();
        for (changed, kind) in [
            (!same(&a.constructor, &b.constructor)?, Self::Constructor),
            (
                !same(
                    &(x.requested_dt_s, x.requested_steps),
                    &(y.requested_dt_s, y.requested_steps),
                )?,
                Self::RequestedIntervals,
            ),
            (!same(&x.forces, &y.forces)?, Self::Forces),
            (!same(&x.volume, &y.volume)?, Self::VolumeControls),
            (!same(&x.viscosity, &y.viscosity)?, Self::Viscosity),
            (x.pressure != y.pressure, Self::PressureSelection),
            (
                (
                    x.provider_workspace_limit_bytes,
                    x.viscosity_workspace_limit_bytes,
                ) != (
                    y.provider_workspace_limit_bytes,
                    y.viscosity_workspace_limit_bytes,
                ),
                Self::Workspaces,
            ),
            (x.recording != y.recording, Self::RecordingBudget),
            (
                x.capture_constructor != y.capture_constructor,
                Self::ConstructorCapture,
            ),
            (x.executor != y.executor, Self::HostExecutor),
        ] {
            if changed {
                result.push(kind)
            }
        }
        Ok(result)
    }
}
/// Declaration only. Final pins do not become verified until the recording
/// owner supplies configured archive binding/decoding; no native pin_closed API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfiguredRunAttempt {
    pub attempt_id: String,
    pub archive_relative_path: String,
    pub expected_run_id: u64,
    pub expected_source_revision: String,
    pub finalized: Option<FinalizedArchive>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfiguredRunSetMember {
    pub member_id: String,
    pub spec: FrozenConfiguredSpec,
    pub differences_from_baseline: Vec<ConfiguredSpecDifference>,
    pub attempt: Option<ConfiguredRunAttempt>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfiguredRunSetRevision {
    pub schema_version: u32,
    pub kind: ConfiguredSetKind,
    pub set_id: String,
    pub parent_sha256: Option<[u8; 32]>,
    pub baseline_member: String,
    pub members: Vec<ConfiguredRunSetMember>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SetEnvelope {
    schema_version: u32,
    kind: ConfiguredSetKind,
    sha256: [u8; 32],
    revision: ConfiguredRunSetRevision,
}
impl ConfiguredRunSetRevision {
    pub fn validate(&self) -> Result<(), RunSetError> {
        version(self.schema_version)?;
        text(&self.set_id)?;
        text(&self.baseline_member)?;
        if self.members.is_empty() || self.members.len() > MAX_MEMBERS {
            return Err(RunSetError::Invalid("configured member count"));
        }
        let baseline = self
            .members
            .iter()
            .find(|m| m.member_id == self.baseline_member)
            .ok_or(RunSetError::Invalid("configured baseline missing"))?;
        for (i, m) in self.members.iter().enumerate() {
            text(&m.member_id)?;
            m.spec.validate()?;
            if self.members[..i].iter().any(|p| p.member_id == m.member_id) {
                return Err(RunSetError::Invalid("duplicate configured member"));
            }
            if m.differences_from_baseline
                != ConfiguredSpecDifference::between(&baseline.spec, &m.spec)?
            {
                return Err(RunSetError::Invalid("configured declared differences"));
            }
            if let Some(a) = &m.attempt {
                text(&a.attempt_id)?;
                text(&a.expected_source_revision)?;
                relative_path(&a.archive_relative_path)?;
                if a.expected_source_revision != m.spec.future.executor.source_revision {
                    return Err(RunSetError::Integrity("attempt host source differs"));
                }
                if self.members[..i]
                    .iter()
                    .filter_map(|m| m.attempt.as_ref())
                    .any(|p| {
                        p.attempt_id == a.attempt_id
                            || p.archive_relative_path == a.archive_relative_path
                    })
                {
                    return Err(RunSetError::Invalid("duplicate configured attempt/archive"));
                }
                if let Some(pin) = &a.finalized {
                    if pin.recorded_count > m.spec.future.recording.max_records
                        || pin.recorded_count
                            > m.spec.future.requested_steps
                                + usize::from(m.spec.future.capture_constructor)
                        || pin
                            .accepted_generation
                            .is_some_and(|g| g > m.spec.future.requested_steps as u64)
                    {
                        return Err(RunSetError::Invalid("configured declared archive horizon"));
                    }
                    let accepted_prefix = pin
                        .recorded_count
                        .saturating_sub(usize::from(m.spec.future.capture_constructor));
                    if (pin.recorded_count > 0
                        && pin
                            .accepted_generation
                            .is_none_or(|g| g < accepted_prefix as u64))
                        || (pin.outcome == TerminalOutcome::Completed
                            && (pin.accepted_generation
                                != Some(m.spec.future.requested_steps as u64)
                                || pin.recorded_count
                                    != m.spec.future.requested_steps
                                        + usize::from(m.spec.future.capture_constructor)))
                    {
                        return Err(RunSetError::Invalid("configured declared terminal tail"));
                    }
                }
            }
        }
        Ok(())
    }
    pub fn digest(&self) -> Result<[u8; 32], RunSetError> {
        self.validate()?;
        digest(self)
    }
    pub fn save_new(&self, path: &Path) -> Result<(), RunSetError> {
        self.validate()?;
        let envelope = SetEnvelope {
            schema_version: CONFIGURED_VERSION,
            kind: ConfiguredSetKind::ConfiguredAcceptedSessionSet,
            sha256: self.digest()?,
            revision: self.clone(),
        };
        publish_new(path, &serde_json::to_vec(&envelope)?, MAX_MANIFEST_BYTES)
    }
    pub fn open(path: &Path) -> Result<Self, RunSetError> {
        let envelope: SetEnvelope = read_versioned(path, CONFIGURED_VERSION, MAX_MANIFEST_BYTES)?;
        envelope.revision.validate()?;
        if envelope.sha256 != envelope.revision.digest()? {
            return Err(RunSetError::Integrity("configured set digest"));
        }
        Ok(envelope.revision)
    }
}
