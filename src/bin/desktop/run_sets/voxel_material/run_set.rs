//! Explicit immutable voxel profile and cold closed-archive binding. Inspection
//! never constructs a simulation, resolves a provider or promotes an unpinned file.
use super::super::{
    FinalizedArchive, MemberState, RunAttempt, RunSetError, TerminalOutcome, archive_path,
    configured::ExecutorDeclaration, digest, file_digest, publish_new, read_versioned, text,
};
use super::{FrozenInput, INPUT_VERSION, InputEnvelope};
use rheon::{
    SessionIdentity, SnapshotStage,
    recording::{
        RecordedVoxelMaterial, RecordingReader, RecordingRun, voxel_material_metadata,
        voxel_material_run,
    },
    voxel_material::VoxelMaterialSimulation,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
const VERSION: u32 = 3;
const MAX_BYTES: u64 = 1024 * 1024;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoxelSetKind {
    VoxelMaterialSet,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VoxelRunSetMember {
    pub member_id: String,
    #[serde(with = "recipe")]
    pub spec: FrozenInput,
    pub attempt: Option<RunAttempt>,
}
mod recipe {
    use super::*;
    pub fn serialize<S: serde::Serializer>(value: &FrozenInput, s: S) -> Result<S::Ok, S::Error> {
        InputEnvelope {
            schema_version: INPUT_VERSION,
            input: value.input.clone(),
            sha256: value.sha256,
        }
        .serialize(s)
    }
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(d: D) -> Result<FrozenInput, D::Error> {
        let value = InputEnvelope::deserialize(d)?;
        if value.schema_version != INPUT_VERSION {
            return Err(serde::de::Error::custom("unsupported voxel input version"));
        }
        let frozen = FrozenInput {
            input: value.input,
            sha256: value.sha256,
        };
        frozen.validate().map_err(serde::de::Error::custom)?;
        Ok(frozen)
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VoxelRunSetRevision {
    pub schema_version: u32,
    pub kind: VoxelSetKind,
    pub set_id: String,
    pub parent_sha256: Option<[u8; 32]>,
    pub baseline_member: String,
    pub members: Vec<VoxelRunSetMember>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    schema_version: u32,
    revision: VoxelRunSetRevision,
    sha256: [u8; 32],
}
impl VoxelRunSetRevision {
    pub fn validate(&self) -> Result<(), RunSetError> {
        if self.schema_version != VERSION {
            return Err(RunSetError::Unsupported("voxel manifest version"));
        }
        text(&self.set_id)?;
        text(&self.baseline_member)?;
        if self.members.len() != 1 || self.members[0].member_id != self.baseline_member {
            return Err(RunSetError::Unsupported(
                "voxel run set requires one baseline member",
            ));
        }
        let member = &self.members[0];
        text(&member.member_id)?;
        member.spec.validate()?;
        if let Some(a) = &member.attempt {
            a.validate()?;
            if a.expected_source_revision != member.spec.execution().executor.source_revision {
                return Err(RunSetError::Integrity("voxel attempt executor source"));
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
        publish_new(
            path,
            &serde_json::to_vec(&Envelope {
                schema_version: VERSION,
                revision: self.clone(),
                sha256: digest(self)?,
            })?,
            MAX_BYTES,
        )
    }
    pub fn open(path: &Path) -> Result<Self, RunSetError> {
        let e: Envelope = read_versioned(path, VERSION, MAX_BYTES)?;
        e.revision.validate()?;
        if digest(&e.revision)? != e.sha256 {
            return Err(RunSetError::Integrity("voxel run-set digest"));
        }
        Ok(e.revision)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum BindingKind {
    VoxelMaterialIntent,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    schema_version: u32,
    kind: BindingKind,
    input_sha256: [u8; 32],
    executor: ExecutorDeclaration,
}
fn binding(spec: &FrozenInput) -> Binding {
    Binding {
        schema_version: 1,
        kind: BindingKind::VoxelMaterialIntent,
        input_sha256: spec.digest(),
        executor: spec.execution().executor.clone(),
    }
}
/// Bind an actual constructor to captured intent before creating any archive.
/// This is a producer declaration, not authentication of future execution.
pub fn bound_run(
    spec: &FrozenInput,
    sim: &VoxelMaterialSimulation,
    identity: SessionIdentity,
) -> Result<RecordingRun, RunSetError> {
    spec.validate()?;
    let doc = spec.input.initial.restore()?;
    let state = sim.state();
    if state.generation() != 0
        || state.time() != 0.0
        || state.geometry() != doc.geometry()
        || [rheon::Axis::X, rheon::Axis::Y, rheon::Axis::Z]
            .iter()
            .any(|&a| state.velocity(a).iter().any(|&v| v.to_bits() != 0))
        || state
            .liquid_fraction()
            .iter()
            .zip(doc.cells())
            .any(|(&f, &id)| f.to_bits() != if id == 1 { 1.0_f64.to_bits() } else { 0 })
        || identity.source_revision != spec.execution().executor.source_revision
    {
        return Err(RunSetError::Integrity(
            "voxel constructor differs from input",
        ));
    }
    let run = voxel_material_run(
        identity,
        sim,
        spec.digest(),
        &serde_json::to_string(&binding(spec))?,
    )?;
    require_metadata(&run, spec)?;
    Ok(run)
}
fn require_metadata(run: &RecordingRun, spec: &FrozenInput) -> Result<(), RunSetError> {
    let metadata = voxel_material_metadata(run)?;
    let initial = &spec.input.initial;
    let pair = initial.materials()?;
    let p = spec.execution();
    let c = metadata.config;
    let actual = metadata.materials;
    let material_bits = |m: rheon::voxel_material::MaterialPair| {
        [
            m.liquid().density(),
            m.liquid().viscosity(),
            m.gas().density(),
            m.gas().viscosity(),
        ]
        .map(f64::to_bits)
    };
    if run.counts != initial.counts_xyz
        || run.spacing.map(f64::to_bits) != initial.spacing_m.map(f64::to_bits)
        || run.origin.map(f64::to_bits) != initial.origin_m.map(f64::to_bits)
        || metadata.initial_input_sha256 != spec.digest()
        || material_bits(actual) != material_bits(pair)
        || c.gravity.map(f64::to_bits) != p.gravity_m_s2.map(f64::to_bits)
        || [
            c.pressure.relative_residual,
            c.pressure.absolute_residual,
            c.pressure.divergence_limit,
        ]
        .map(f64::to_bits)
            != [
                p.pressure.relative_residual,
                p.pressure.absolute_residual,
                p.pressure.divergence_limit,
            ]
            .map(f64::to_bits)
        || c.pressure.max_iterations != p.pressure.max_iterations
        || c.memory_limit != p.numerical_memory_limit_bytes
    {
        return Err(RunSetError::Integrity(
            "voxel archive captured configuration",
        ));
    }
    let declaration: Binding = serde_json::from_str(&run.provenance)?;
    if declaration != binding(spec) {
        return Err(RunSetError::Integrity("voxel archive intent declaration"));
    }
    Ok(())
}
fn validate_attempt(a: &RunAttempt, spec: &FrozenInput) -> Result<(), RunSetError> {
    spec.validate()?;
    a.validate()?;
    if a.expected_source_revision != spec.execution().executor.source_revision {
        return Err(RunSetError::Integrity("voxel attempt executor source"));
    }
    Ok(())
}
fn header(reader: &RecordingReader, a: &RunAttempt, spec: &FrozenInput) -> Result<(), RunSetError> {
    require_metadata(reader.run(), spec)?;
    if reader.run().identity.run_id != a.expected_run_id
        || reader.run().identity.source_revision != a.expected_source_revision
        || reader.recorded_limits() != spec.execution().recording.limits()
    {
        return Err(RunSetError::Integrity("voxel archive identity or budget"));
    }
    Ok(())
}
/// Audit declared schedule/fields and committed coverage, not future physics.
fn publications(
    reader: &RecordingReader,
    spec: &FrozenInput,
    outcome: TerminalOutcome,
) -> Result<(), RunSetError> {
    let count = reader.count()?;
    let policy = spec.execution();
    if count > policy.requested_steps + 1
        || (outcome == TerminalOutcome::Completed && count != policy.requested_steps + 1)
    {
        return Err(RunSetError::Integrity("voxel publication count"));
    }
    let initial = spec.input.initial.restore()?;
    let mut time = 0.0;
    for ordinal in 0..count {
        let step = reader.select(ordinal)?;
        let decoded = RecordedVoxelMaterial::decode(reader.run(), &step)?;
        if step.generation != ordinal as u64 {
            return Err(RunSetError::Integrity("voxel publication generation"));
        }
        if ordinal == 0 {
            if decoded
                .velocity
                .iter()
                .any(|v| v.iter().any(|x| x.to_bits() != 0))
            {
                return Err(RunSetError::Integrity(
                    "voxel original constructor velocity",
                ));
            }
            if step.stage != SnapshotStage::Constructor
                || decoded
                    .liquid_fraction
                    .iter()
                    .zip(initial.cells())
                    .any(|(&f, &id)| f.to_bits() != if id == 1 { 1.0_f64.to_bits() } else { 0 })
            {
                return Err(RunSetError::Integrity("voxel original constructor fields"));
            }
        } else {
            if step.stage != SnapshotStage::Accepted {
                return Err(RunSetError::Integrity("voxel accepted stage"));
            }
            let dt = step.dt.ok_or(RunSetError::Integrity("voxel accepted dt"))?;
            // Real retries shorten by Courant/stress ratios and next_down;
            // a cold record cannot authenticate their cause or attempt count.
            let admitted = dt.is_normal() && dt > 0.0 && dt <= policy.requested_dt_s;
            let start = decoded
                .interval_pressure
                .as_ref()
                .ok_or(RunSetError::Integrity("voxel interval pressure"))?
                .0;
            if !admitted
                || step.time.to_bits() != (time + dt).to_bits()
                || start.to_bits() != time.to_bits()
            {
                return Err(RunSetError::Integrity("voxel accepted interval schedule"));
            }
        }
        time = step.time;
    }
    let status = reader.status()?;
    let accepted = status
        .accepted_generation
        .ok_or(RunSetError::Integrity("voxel terminal accepted coverage"))?;
    let recorded_generation = count.saturating_sub(1) as u64;
    if accepted > policy.requested_steps as u64
        || accepted < recorded_generation
        || accepted
            > if count == 0 {
                0
            } else {
                recorded_generation + 1
            }
        || (outcome == TerminalOutcome::Completed && accepted != policy.requested_steps as u64)
    {
        return Err(RunSetError::Integrity("voxel terminal accepted coverage"));
    }
    Ok(())
}
/// Only after closing the producer, on protected immutable caller-owned paths.
pub fn pin_closed(a: &mut RunAttempt, root: &Path, spec: &FrozenInput) -> Result<(), RunSetError> {
    validate_attempt(a, spec)?;
    if a.finalized.is_some() {
        return Err(RunSetError::Invalid("voxel attempt already pinned"));
    }
    let path = archive_path(root, &a.archive_relative_path)?;
    let cap = spec.execution().recording.max_database_bytes;
    let before = file_digest(&path, cap)?;
    let reader = RecordingReader::open(&path, spec.execution().recording.limits())?;
    header(&reader, a, spec)?;
    let status = reader.status()?;
    let outcome = TerminalOutcome::from_status(&status.outcome)?;
    reader.verify()?;
    publications(&reader, spec, outcome)?;
    let count = reader.count()?;
    drop(reader);
    let after = file_digest(&path, cap)?;
    if before != after {
        return Err(RunSetError::Integrity("voxel archive changed during pin"));
    }
    a.finalized = Some(FinalizedArchive {
        sha256: after,
        recorded_count: count,
        outcome,
        accepted_generation: status.accepted_generation,
    });
    Ok(())
}
pub fn open_member(root: &Path, member: &VoxelRunSetMember) -> MemberState {
    match open_inner(root, member) {
        Ok(s) => s,
        Err(RunSetError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => MemberState::Missing,
        Err(RunSetError::Recording(rheon::recording::RecordingError::Io(e)))
            if e.kind() == std::io::ErrorKind::NotFound =>
        {
            MemberState::Missing
        }
        Err(
            RunSetError::Unsupported(r)
            | RunSetError::Recording(rheon::recording::RecordingError::Unsupported(r)),
        ) => MemberState::Unsupported(r),
        Err(e) => MemberState::Invalid(e),
    }
}
fn open_inner(root: &Path, member: &VoxelRunSetMember) -> Result<MemberState, RunSetError> {
    member.spec.validate()?;
    let Some(a) = &member.attempt else {
        return Ok(MemberState::NotRun);
    };
    validate_attempt(a, &member.spec)?;
    let path = archive_path(root, &a.archive_relative_path)?;
    let before = file_digest(&path, member.spec.execution().recording.max_database_bytes)?;
    let reader = RecordingReader::open(&path, member.spec.execution().recording.limits())?;
    header(&reader, a, &member.spec)?;
    let status = reader.status()?;
    let count = reader.count()?;
    let Some(pin) = &a.finalized else {
        return Ok(if status.outcome == "active" {
            MemberState::Incomplete {
                recorded_count: count,
                status,
            }
        } else {
            MemberState::Unpinned {
                recorded_count: count,
                status,
            }
        });
    };
    if pin.sha256 != before
        || pin.recorded_count != count
        || pin.outcome != TerminalOutcome::from_status(&status.outcome)?
        || pin.accepted_generation != status.accepted_generation
    {
        return Err(RunSetError::Integrity("voxel finalized archive changed"));
    }
    reader.verify()?;
    publications(&reader, &member.spec, pin.outcome)?;
    if file_digest(&path, member.spec.execution().recording.max_database_bytes)? != before {
        return Err(RunSetError::Integrity("voxel archive changed during open"));
    }
    Ok(MemberState::Ready(Box::new(reader)))
}
#[cfg(test)]
#[path = "../../voxel_material_run_set_tests.rs"]
mod tests;
