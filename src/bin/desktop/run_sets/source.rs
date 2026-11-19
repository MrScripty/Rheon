//! App-owned selection over the existing native and configured manifest owners.
//! Serialization is the inner revision only; file envelopes, digests, archive
//! validation and readiness remain owned by their respective schema modules.
use super::voxel_material::run_set::{self, VoxelRunSetMember, VoxelRunSetRevision};
use super::{
    FinalizedArchive, MemberState, RunSetError, RunSetMember, RunSetRevision, SpecDifference,
    configured::{
        self, ConfiguredRunSetMember, ConfiguredRunSetRevision, ConfiguredSpecDifference,
        PressureIntent,
    },
};
use rheon::recording::RecordingLimits;
use serde::{Deserialize, Deserializer, Serialize, de::Error};
use std::path::Path;

/// Untagged inner payload for an app-owned envelope. Its consumer must bind its
/// own envelope version to the selected variant. This adds no native JSON tag.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum SavedRunSet {
    Native(RunSetRevision),
    Configured(ConfiguredRunSetRevision),
    Voxel(VoxelRunSetRevision),
}
impl<'de> Deserialize<'de> for SavedRunSet {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Inner {
            Native(RunSetRevision),
            Configured(ConfiguredRunSetRevision),
            Voxel(VoxelRunSetRevision),
        }
        let set = match Inner::deserialize(deserializer)? {
            Inner::Native(set) => Self::Native(set),
            Inner::Configured(set) => Self::Configured(set),
            Inner::Voxel(set) => Self::Voxel(set),
        };
        set.validate().map_err(D::Error::custom)?;
        Ok(set)
    }
}
impl SavedRunSet {
    /// Only an unsupported envelope version selects the next explicit owner.
    /// Versions one/two/three remain native/configured/voxel respectively. Invalid native bytes, integrity failures and I/O are preserved.
    pub fn open(path: &Path) -> Result<Self, RunSetError> {
        match RunSetRevision::open(path) {
            Ok(set) => Ok(Self::Native(set)),
            Err(RunSetError::Unsupported("manifest version")) => {
                match ConfiguredRunSetRevision::open(path) {
                    Ok(set) => Ok(Self::Configured(set)),
                    Err(RunSetError::Unsupported("manifest version")) => {
                        VoxelRunSetRevision::open(path).map(Self::Voxel)
                    }
                    Err(error) => Err(error),
                }
            }
            Err(error) => Err(error),
        }
    }
    pub fn validate(&self) -> Result<(), RunSetError> {
        match self {
            Self::Native(set) => set.validate(),
            Self::Configured(set) => set.validate(),
            Self::Voxel(set) => set.validate(),
        }
    }
    pub fn sha256(&self) -> Result<[u8; 32], RunSetError> {
        match self {
            Self::Native(set) => set.sha256(),
            Self::Configured(set) => set.digest(),
            Self::Voxel(set) => set.digest(),
        }
    }
    pub fn schema_version(&self) -> u32 {
        match self {
            Self::Native(set) => set.schema_version,
            Self::Configured(set) => set.schema_version,
            Self::Voxel(set) => set.schema_version,
        }
    }
    pub fn set_id(&self) -> &str {
        match self {
            Self::Native(set) => &set.set_id,
            Self::Configured(set) => &set.set_id,
            Self::Voxel(set) => &set.set_id,
        }
    }
    pub fn baseline_member_id(&self) -> &str {
        match self {
            Self::Native(set) => &set.baseline_member,
            Self::Configured(set) => &set.baseline_member,
            Self::Voxel(set) => &set.baseline_member,
        }
    }
    pub fn member_count(&self) -> usize {
        match self {
            Self::Native(set) => set.members.len(),
            Self::Configured(set) => set.members.len(),
            Self::Voxel(set) => set.members.len(),
        }
    }
    pub fn members(&self) -> impl Iterator<Item = SavedMember<'_>> {
        self.native()
            .into_iter()
            .flat_map(|set| set.members.iter().map(SavedMember::Native))
            .chain(
                self.configured()
                    .into_iter()
                    .flat_map(|set| set.members.iter().map(SavedMember::Configured)),
            )
            .chain(
                self.voxel()
                    .into_iter()
                    .flat_map(|set| set.members.iter().map(SavedMember::Voxel)),
            )
    }
    pub fn member_ids(&self) -> impl Iterator<Item = &str> {
        self.members().map(|member| member.id())
    }
    pub fn member(&self, id: &str) -> Option<SavedMember<'_>> {
        self.members().find(|member| member.id() == id)
    }
    pub fn is_configured(&self) -> bool {
        matches!(self, Self::Configured(_))
    }
    pub fn native(&self) -> Option<&RunSetRevision> {
        match self {
            Self::Native(set) => Some(set),
            Self::Configured(_) | Self::Voxel(_) => None,
        }
    }
    pub fn configured(&self) -> Option<&ConfiguredRunSetRevision> {
        match self {
            Self::Native(_) | Self::Voxel(_) => None,
            Self::Configured(set) => Some(set),
        }
    }
    pub fn voxel(&self) -> Option<&VoxelRunSetRevision> {
        match self {
            Self::Voxel(set) => Some(set),
            Self::Native(_) | Self::Configured(_) => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SavedMember<'a> {
    Native(&'a RunSetMember),
    Configured(&'a ConfiguredRunSetMember),
    Voxel(&'a VoxelRunSetMember),
}
/// Borrowed declaration/pin view. A present pin does not itself grant Ready;
/// open delegates complete archive verification to the owning schema.
#[derive(Debug, Clone, Copy)]
pub struct SavedAttempt<'a> {
    pub attempt_id: &'a str,
    pub archive_relative_path: &'a str,
    pub expected_run_id: u64,
    pub expected_source_revision: &'a str,
    pub finalized: Option<&'a FinalizedArchive>,
}
impl<'a> SavedMember<'a> {
    pub fn id(self) -> &'a str {
        match self {
            Self::Native(member) => &member.member_id,
            Self::Configured(member) => &member.member_id,
            Self::Voxel(member) => &member.member_id,
        }
    }
    pub fn attempt(self) -> Option<SavedAttempt<'a>> {
        match self {
            Self::Native(member) => member.attempt.as_ref().map(|attempt| SavedAttempt {
                attempt_id: &attempt.attempt_id,
                archive_relative_path: &attempt.archive_relative_path,
                expected_run_id: attempt.expected_run_id,
                expected_source_revision: &attempt.expected_source_revision,
                finalized: attempt.finalized.as_ref(),
            }),
            Self::Configured(member) => member.attempt.as_ref().map(|attempt| SavedAttempt {
                attempt_id: &attempt.attempt_id,
                archive_relative_path: &attempt.archive_relative_path,
                expected_run_id: attempt.expected_run_id,
                expected_source_revision: &attempt.expected_source_revision,
                finalized: attempt.finalized.as_ref(),
            }),
            Self::Voxel(member) => member.attempt.as_ref().map(|attempt| SavedAttempt {
                attempt_id: &attempt.attempt_id,
                archive_relative_path: &attempt.archive_relative_path,
                expected_run_id: attempt.expected_run_id,
                expected_source_revision: &attempt.expected_source_revision,
                finalized: attempt.finalized.as_ref(),
            }),
        }
    }
    pub fn limits(self) -> RecordingLimits {
        match self {
            Self::Native(member) => member.spec.spec.recording.limits(),
            Self::Configured(member) => member.spec.future.recording.limits(),
            Self::Voxel(member) => member.spec.execution().recording.limits(),
        }
    }
    pub fn description(self) -> String {
        match self {
            Self::Native(member) => format!(
                "{} · native pressure ({})",
                member.member_id,
                member.spec.spec.pressure_method.implementation().id(),
            ),
            Self::Voxel(member) => format!(
                "{} · occupied Water/Air voxel MAC · matched donor/strain/Jacobi",
                member.member_id
            ),
            Self::Configured(member) => match &member.spec.future.pressure {
                PressureIntent::NativeOnly => format!(
                    "{} · accepted Session · native pressure ({})",
                    member.member_id,
                    member
                        .spec
                        .constructor
                        .pressure_method
                        .implementation()
                        .id(),
                ),
                PressureIntent::NativePrefixThenConfigured {
                    provider,
                    requested_native_prefix_steps,
                    ..
                } => format!(
                    "{} · requested native prefix {} · configured pressure {} / {}",
                    member.member_id, requested_native_prefix_steps, provider.id, provider.version,
                ),
            },
        }
    }
    pub fn difference_labels(self) -> Vec<&'static str> {
        match self {
            Self::Voxel(_) => Vec::new(),
            Self::Native(member) => member
                .differences_from_baseline
                .iter()
                .map(|difference| match difference {
                    SpecDifference::Geometry => "Geometry",
                    SpecDifference::Density => "Density",
                    SpecDifference::PressureMethod => "Native pressure method",
                    SpecDifference::Tolerances => "Solver and transport tolerances",
                    SpecDifference::TimePolicy => "Requested intervals",
                    SpecDifference::Source => "Smoke source",
                    SpecDifference::MemoryBudget => "Simulation memory limit",
                    SpecDifference::ForceMetadata => "Force metadata",
                    SpecDifference::RecordingBudget => "Recording budget",
                })
                .collect(),
            Self::Configured(member) => member
                .differences_from_baseline
                .iter()
                .map(|difference| match difference {
                    ConfiguredSpecDifference::Constructor => "Constructor controls",
                    ConfiguredSpecDifference::RequestedIntervals => "Requested intervals",
                    ConfiguredSpecDifference::Forces => "Body forces",
                    ConfiguredSpecDifference::VolumeControls => "Volume controls",
                    ConfiguredSpecDifference::Viscosity => "Viscosity stage",
                    ConfiguredSpecDifference::PressureSelection => "Pressure selection",
                    ConfiguredSpecDifference::Workspaces => "Workspace limits",
                    ConfiguredSpecDifference::RecordingBudget => "Recording budget",
                    ConfiguredSpecDifference::ConstructorCapture => "Constructor capture",
                    ConfiguredSpecDifference::HostExecutor => "Host executor declaration",
                })
                .collect(),
        }
    }
    /// Cold inspection: no numerical owner or provider/runtime is constructed.
    pub fn open(self, root: &Path) -> MemberState {
        match self {
            Self::Native(member) => super::open_member(root, member),
            Self::Configured(member) => configured::archive::open_member(root, member),
            Self::Voxel(member) => run_set::open_member(root, member),
        }
    }
}
