//! Cold configured archive binding. No Session, package or runtime is created
//! during inspection. Future forces remain producer declarations: an RHS digest
//! does not reconstruct or independently prove the execution recipe.
use super::super::{
    FinalizedArchive, MemberState, RunSetError, TerminalOutcome, archive_path, file_digest, text,
};
use super::{
    ConfiguredConstructor, ConfiguredProviderPin, ConfiguredRunAttempt, ConfiguredRunSetMember,
    ExecutorDeclaration, FrozenConfiguredSpec, PressureIntent, ViscosityIntent, same,
};
use rheon::{
    SessionSnapshot, SnapshotStage,
    recording::{
        RecordingError, RecordingReader, RecordingRun,
        provider::{
            PROFILE, RecordedConfiguredSession, RecordedPressureSelection,
            configured_configuration, configured_run,
            runtime::{RuntimeTerminationAudit, audit_runtime_termination},
            verify_configured_recording,
        },
    },
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct IntentBinding {
    schema_version: u32,
    kind: String,
    spec_sha256: [u8; 32],
    executor: ExecutorDeclaration,
}
const BINDING_KIND: &str = "configured-accepted-session-intent-v1";

/// Runtime custody is separate from physical acceptance and member readiness.
#[derive(Debug, Clone)]
pub enum ConfiguredRuntimeCustody {
    /// No typed terminal custody is available. This does not imply that no
    /// request was dispatched or that no accepted publication has RCP3 metadata.
    LegacyUnavailable,
    /// Preserve the core audit's last-publication, unrecorded-tail, or empty
    /// coverage without promoting any of them to execution authentication.
    Typed(Box<RuntimeTerminationAudit>),
}

/// Audit available runtime custody and bind terminal requests to frozen intent.
/// This does not establish complete member readiness: `open_member` separately
/// checks the header, provenance, publication schedule, path and immutable hash.
/// Inspection never resolves a deployment or constructs a Session or process.
pub fn runtime_custody(
    reader: &RecordingReader,
    spec: &FrozenConfiguredSpec,
) -> Result<ConfiguredRuntimeCustody, RunSetError> {
    spec.validate()?;
    let status = reader.status()?;
    let typed_tag = status.detail.strip_prefix("RCT").map(|suffix| {
        let digits = suffix.bytes().take_while(u8::is_ascii_digit).count();
        suffix.split_at(digits)
    });
    let Some((version, payload)) =
        typed_tag.filter(|(version, _)| !version.is_empty() && status.outcome != "active")
    else {
        verify_configured_recording(reader)?;
        return Ok(ConfiguredRuntimeCustody::LegacyUnavailable);
    };
    if version != "1" {
        return Err(RunSetError::Unsupported(
            "configured runtime termination version",
        ));
    }
    if !payload.starts_with(':') {
        return Err(RunSetError::Invalid(
            "configured runtime termination payload",
        ));
    }
    let audit = audit_runtime_termination(reader)?;
    let future = &spec.future;
    if audit.termination.accepted_generation > future.requested_steps as u64 {
        return Err(RunSetError::Integrity(
            "configured runtime terminal horizon",
        ));
    }
    // This frozen profile starts at +0 and every accepted interval is at most
    // requested_dt. Repeat the same IEEE additions as the owner: multiplying
    // generation by dt can underestimate a valid accumulated clock.
    let mut latest_accepted_time = 0.0;
    for _ in 0..audit.termination.accepted_generation {
        latest_accepted_time += future.requested_dt_s;
    }
    if audit.termination.accepted_time > latest_accepted_time {
        return Err(RunSetError::Integrity(
            "configured runtime terminal clock exceeds intent",
        ));
    }
    if let Some(observation) = &audit.termination.observation {
        let request = &observation.request;
        if request.context.generation > future.requested_steps as u64 {
            return Err(RunSetError::Integrity("configured runtime request horizon"));
        }
        if request.context.dt > future.requested_dt_s {
            return Err(RunSetError::Integrity(
                "configured runtime dt exceeds request",
            ));
        }
        if request.dynamic_viscosity.to_bits() != declared_viscosity(spec).1.to_bits() {
            return Err(RunSetError::Integrity(
                "configured runtime viscosity differs",
            ));
        }
        let pin = expected_provider(spec, request.context.generation).ok_or(
            RunSetError::Integrity("configured runtime request in native phase"),
        )?;
        if let PressureIntent::NativePrefixThenConfigured {
            requested_native_prefix_steps,
            ..
        } = &future.pressure
            && request.context.native_prefix_steps < *requested_native_prefix_steps
        {
            return Err(RunSetError::Integrity(
                "configured runtime native prefix below intent",
            ));
        }
        pin.validate_resolved_identity(&request.binding)?;
        // The request records mu, not whether an aborted viscosity stage ran.
        // In particular, zero mu does not prove Disabled or applied presence.
    }
    Ok(ConfiguredRuntimeCustody::Typed(Box::new(audit)))
}

fn declared_viscosity(spec: &FrozenConfiguredSpec) -> (bool, f64) {
    match spec.future.viscosity {
        ViscosityIntent::Disabled => (false, 0.0),
        ViscosityIntent::NewtonianConstant {
            dynamic_viscosity_pa_s,
        } => (true, dynamic_viscosity_pa_s),
    }
}

fn expected_provider(
    spec: &FrozenConfiguredSpec,
    generation: u64,
) -> Option<&ConfiguredProviderPin> {
    match &spec.future.pressure {
        PressureIntent::NativeOnly => None,
        PressureIntent::NativePrefixThenConfigured {
            provider,
            requested_native_prefix_steps,
            ..
        } if generation > *requested_native_prefix_steps as u64 => Some(provider),
        PressureIntent::NativePrefixThenConfigured { .. } => None,
    }
}

/// Bind a real constructor to immutable intent before numerical/model execution.
/// This is a producer declaration, not authentication or proof of future forces.
pub fn bound_configured_run(
    spec: &FrozenConfiguredSpec,
    constructor: &SessionSnapshot<'_>,
) -> Result<RecordingRun, RunSetError> {
    spec.validate_constructor(constructor)?;
    let provenance = serde_json::to_string(&IntentBinding {
        schema_version: 1,
        kind: BINDING_KIND.into(),
        spec_sha256: spec.sha256,
        executor: spec.future.executor.clone(),
    })?;
    Ok(configured_run(constructor, &provenance)?)
}

impl ConfiguredRunAttempt {
    fn validate_binding_declaration(&self, spec: &FrozenConfiguredSpec) -> Result<(), RunSetError> {
        spec.validate()?;
        text(&self.attempt_id)?;
        text(&self.expected_source_revision)?;
        super::super::relative_path(&self.archive_relative_path)?;
        if self.expected_source_revision != spec.future.executor.source_revision {
            return Err(RunSetError::Integrity("configured attempt host source"));
        }
        Ok(())
    }

    /// Call after closing the writer. Protected immutable local paths are still
    /// required; the two hashes detect ordinary changes during this operation.
    pub fn pin_closed(
        &mut self,
        root: &Path,
        spec: &FrozenConfiguredSpec,
    ) -> Result<(), RunSetError> {
        self.validate_binding_declaration(spec)?;
        if self.finalized.is_some() {
            return Err(RunSetError::Invalid("configured attempt already pinned"));
        }
        let path = archive_path(root, &self.archive_relative_path)?;
        let before = file_digest(&path, spec.future.recording.max_database_bytes)?;
        let reader = RecordingReader::open(&path, spec.future.recording.limits())?;
        validate_header(&reader, self, spec)?;
        let status = reader.status()?;
        let outcome = TerminalOutcome::from_status(&status.outcome)?;
        validate_publications(&reader, spec, outcome)?;
        let count = reader.count()?;
        drop(reader);
        let after = file_digest(&path, spec.future.recording.max_database_bytes)?;
        if before != after {
            return Err(RunSetError::Integrity(
                "configured archive changed during pin",
            ));
        }
        self.finalized = Some(FinalizedArchive {
            sha256: after,
            recorded_count: count,
            outcome,
            accepted_generation: status.accepted_generation,
        });
        Ok(())
    }
}

/// Resolve exactly this immutable member. No live/latest, model resolution,
/// missing-field substitution, numerical construction or resume operation.
pub fn open_member(root: &Path, member: &ConfiguredRunSetMember) -> MemberState {
    match open_inner(root, member) {
        Ok(state) => state,
        Err(RunSetError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => MemberState::Missing,
        Err(RunSetError::Recording(RecordingError::Io(e)))
            if e.kind() == std::io::ErrorKind::NotFound =>
        {
            MemberState::Missing
        }
        Err(
            RunSetError::Unsupported(reason)
            | RunSetError::Recording(RecordingError::Unsupported(reason)),
        ) => MemberState::Unsupported(reason),
        Err(error) => MemberState::Invalid(error),
    }
}
fn open_inner(root: &Path, member: &ConfiguredRunSetMember) -> Result<MemberState, RunSetError> {
    member.spec.validate()?;
    let Some(attempt) = &member.attempt else {
        return Ok(MemberState::NotRun);
    };
    attempt.validate_binding_declaration(&member.spec)?;
    let path = archive_path(root, &attempt.archive_relative_path)?;
    if let Some(pin) = &attempt.finalized
        && file_digest(&path, member.spec.future.recording.max_database_bytes)? != pin.sha256
    {
        return Err(RunSetError::Integrity(
            "configured finalized archive changed",
        ));
    }
    let reader = RecordingReader::open(&path, member.spec.future.recording.limits())?;
    validate_header(&reader, attempt, &member.spec)?;
    let status = reader.status()?;
    let count = reader.count()?;
    // Even active/unpinned states retain typed profile and publication checks;
    // neither returns numerical arrays to a Ready consumer.
    runtime_custody(&reader, &member.spec)?;
    let Some(pin) = &attempt.finalized else {
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
    let outcome = TerminalOutcome::from_status(&status.outcome)?;
    if count != pin.recorded_count
        || outcome != pin.outcome
        || status.accepted_generation != pin.accepted_generation
    {
        return Err(RunSetError::Integrity(
            "configured finalized status changed",
        ));
    }
    validate_publications(&reader, &member.spec, outcome)?;
    if file_digest(&path, member.spec.future.recording.max_database_bytes)? != pin.sha256 {
        return Err(RunSetError::Integrity(
            "configured archive changed during open",
        ));
    }
    Ok(MemberState::Ready(Box::new(reader)))
}
fn validate_header(
    reader: &RecordingReader,
    attempt: &ConfiguredRunAttempt,
    spec: &FrozenConfiguredSpec,
) -> Result<(), RunSetError> {
    if reader.recorded_limits() != spec.future.recording.limits() {
        return Err(RunSetError::Integrity("configured recording budget"));
    }
    let run = reader.run();
    if run.solver != PROFILE {
        return Err(RunSetError::Unsupported(
            "configured Session recording profile",
        ));
    }
    let configuration = configured_configuration(run)?;
    let grid = spec.constructor.grid()?;
    if run.counts != spec.constructor.counts_xyz
        || run.spacing.map(f64::to_bits) != grid.spacing().map(f64::to_bits)
        || run.origin.map(f64::to_bits) != grid.origin().map(f64::to_bits)
        || run.identity.run_id != attempt.expected_run_id
        || run.identity.source_revision != attempt.expected_source_revision
        || !same(
            &ConfiguredConstructor::from_configuration(&grid, &configuration)?,
            &spec.constructor,
        )?
    {
        return Err(RunSetError::Integrity(
            "configured constructor/header binding",
        ));
    }
    let binding: IntentBinding = serde_json::from_str(&run.provenance)?;
    binding.executor.validate()?;
    if binding.schema_version != 1
        || binding.kind != BINDING_KIND
        || binding.spec_sha256 != spec.sha256
        || binding.executor != spec.future.executor
    {
        return Err(RunSetError::Integrity("configured frozen intent binding"));
    }
    Ok(())
}
fn validate_publications(
    reader: &RecordingReader,
    spec: &FrozenConfiguredSpec,
    outcome: TerminalOutcome,
) -> Result<(), RunSetError> {
    runtime_custody(reader, spec)?;
    let count = reader.count()?;
    let future = &spec.future;
    let constructor_count = usize::from(future.capture_constructor);
    let horizon = future.requested_steps + constructor_count;
    let status = reader.status()?;
    let accepted = status.accepted_generation.ok_or(RunSetError::Integrity(
        "configured accepted tail unspecified",
    ))?;
    if count > horizon || accepted > future.requested_steps as u64 {
        return Err(RunSetError::Integrity("configured publication horizon"));
    }
    let (viscosity_applied, mu) = declared_viscosity(spec);
    let mut last_generation = 0;
    for ordinal in 0..count {
        let step = reader.select(ordinal)?;
        let observed = RecordedConfiguredSession::decode(reader.run(), &step)?;
        let initial = future.capture_constructor && ordinal == 0;
        let generation = (ordinal + 1 - constructor_count) as u64;
        if observed.generation != generation
            || step.stage
                != if initial {
                    SnapshotStage::Constructor
                } else {
                    SnapshotStage::Accepted
                }
        {
            return Err(RunSetError::Integrity("configured publication schedule"));
        }
        if initial {
            if observed
                .x
                .iter()
                .chain(&observed.y)
                .chain(&observed.z)
                .chain(&observed.tracer)
                .any(|v| v.to_bits() != 0.0f32.to_bits())
                || observed
                    .fraction
                    .iter()
                    .any(|v| v.to_bits() != 1.0f64.to_bits())
                || observed.time.to_bits() != 0.0f64.to_bits()
            {
                return Err(RunSetError::Integrity("configured constructor rest fields"));
            }
        } else {
            let dt = observed
                .dt
                .ok_or(RunSetError::Integrity("configured accepted dt absent"))?;
            if dt > future.requested_dt_s {
                return Err(RunSetError::Integrity(
                    "configured actual dt exceeds request",
                ));
            }
            let receipt = observed
                .receipt
                .as_ref()
                .ok_or(RunSetError::Integrity("configured accepted receipt absent"))?;
            if receipt.viscosity_applied != viscosity_applied
                || receipt.dynamic_viscosity.to_bits() != mu.to_bits()
            {
                return Err(RunSetError::Integrity(
                    "configured applied viscosity differs",
                ));
            }
            match expected_provider(spec, generation) {
                Some(pin) => {
                    let actual = receipt
                        .binding
                        .as_ref()
                        .ok_or(RunSetError::Integrity("configured package binding absent"))?;
                    pin.validate_resolved_identity(actual)?;
                    if receipt.context.is_none() || receipt.attempted.is_none() {
                        return Err(RunSetError::Integrity("configured frozen context absent"));
                    }
                }
                None => {
                    if receipt.binding.is_some()
                        || receipt.attempted.is_some()
                        || receipt.selected
                            != RecordedPressureSelection::Native(
                                spec.constructor.pressure_method.implementation(),
                            )
                    {
                        return Err(RunSetError::Integrity(
                            "configured native-prefix selection differs",
                        ));
                    }
                }
            }
        }
        last_generation = generation;
    }
    if accepted < last_generation
        || outcome == TerminalOutcome::Completed
            && (accepted != future.requested_steps as u64 || count != horizon)
    {
        return Err(RunSetError::Integrity(
            "configured completed horizon or terminal tail",
        ));
    }
    Ok(())
}
