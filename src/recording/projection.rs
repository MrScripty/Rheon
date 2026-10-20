//! Native projection Session inspection in the existing SQLite v1 owner.
//!
//! RPS1/RPP1/RPT1 and selection RPS2/RPP2/RPT2 retain native RSC1/RCP2 bytes. These are
//! observations, not restart state, execution authentication, or a GUI graph.
//! The older G1 `workflow::terminal` returns None for this profile: G1 evidence
//! is unavailable, which says neither that this is legacy nor that it has no
//! terminal status. Use this module's typed terminal reader for RPT1 custody.
use super::provider::runtime::{RuntimeTerminalCoverage, RuntimeTerminationCause};
use super::provider::{
    self, Decoder, Encoder, RecordedConfiguredReceipt, RecordedConfiguredSession,
    RecordedPressureSelection,
};
use super::{RecordedStep, RecordingError, RecordingReader, RecordingRun, StepRecorder};
use crate::projection_step::{ProjectionAttempt, ProjectionIntent};
use crate::{SessionConfiguration, SessionSnapshot, SnapshotStage};
use std::sync::Arc;

pub const PROFILE: &str = "rheon-native-projection-session-v1";
pub const CONFIGURATION_PROFILE: &str = "RPS1:";
pub const PUBLICATION_PROFILE: &str = "RPP1:";
pub const TERMINAL_PROFILE: &str = "RPT1:";
/// Finite distinct producer/fallback selection; the v1 namespace is unchanged.
pub const SELECTION_PROFILE: &str = "rheon-native-projection-selection-v1";
pub const SELECTION_CONFIGURATION_PROFILE: &str = "RPS2:";
pub const SELECTION_PUBLICATION_PROFILE: &str = "RPP2:";
pub const SELECTION_TERMINAL_PROFILE: &str = "RPT2:";

fn profiles(intent: ProjectionIntent) -> (&'static str, &'static str, &'static str, &'static str) {
    match intent {
        ProjectionIntent::BuiltinNativeV1 => (
            PROFILE,
            CONFIGURATION_PROFILE,
            PUBLICATION_PROFILE,
            TERMINAL_PROFILE,
        ),
        ProjectionIntent::ClassicalSelectionV1 { .. } => (
            SELECTION_PROFILE,
            SELECTION_CONFIGURATION_PROFILE,
            SELECTION_PUBLICATION_PROFILE,
            SELECTION_TERMINAL_PROFILE,
        ),
    }
}

fn put_wrapped(encoder: &mut Encoder, text: &str, profile: &str) -> Result<(), RecordingError> {
    let inner = Decoder::text(text, profile)?;
    encoder.u32(inner.data.len() as u32);
    encoder.0.extend_from_slice(&inner.data);
    Ok(())
}
fn get_wrapped(decoder: &mut Decoder, profile: &str) -> Result<String, RecordingError> {
    let len = decoder.u32()? as usize;
    if len > super::MAX_TEXT / 2 {
        return Err(RecordingError::Limit("projection wrapped metadata"));
    }
    let mut data = Vec::with_capacity(len);
    for _ in 0..len {
        data.push(decoder.u8()?);
    }
    Encoder(data).text(profile)
}
fn put_intent(encoder: &mut Encoder, intent: ProjectionIntent) {
    match intent {
        ProjectionIntent::BuiltinNativeV1 => encoder.u8(1),
        ProjectionIntent::ClassicalSelectionV1 {
            producer,
            producer_max_iterations,
        } => {
            encoder.u8(2);
            put_method(encoder, producer);
            encoder.u64(producer_max_iterations as u64);
        }
    }
}
fn get_intent(decoder: &mut Decoder, version: u32) -> Result<ProjectionIntent, RecordingError> {
    match (version, decoder.u8()?) {
        (1, 1) => Ok(ProjectionIntent::BuiltinNativeV1),
        (2, 2) => Ok(ProjectionIntent::ClassicalSelectionV1 {
            producer: get_method(decoder)?,
            producer_max_iterations: decoder.usize()?,
        }),
        _ => Err(RecordingError::Unsupported("projection intent version")),
    }
}
fn recording_grid(run: &RecordingRun) -> Result<crate::GridGeometry, RecordingError> {
    crate::GridGeometry::new(run.counts, run.spacing, run.origin)
        .map_err(|_| RecordingError::Invalid("projection recorded geometry"))
}
fn native_header(run: &RecordingRun) -> Result<(RecordingRun, ProjectionIntent), RecordingError> {
    run.validate()?;
    let (profile, version) = match run.solver.as_str() {
        PROFILE => (CONFIGURATION_PROFILE, 1),
        SELECTION_PROFILE => (SELECTION_CONFIGURATION_PROFILE, 2),
        _ => return Err(RecordingError::Unsupported("projection Session profile")),
    };
    let mut decoder = Decoder::text(&run.configuration, profile)?;
    let configuration = get_wrapped(&mut decoder, "RSC1:")?;
    let intent = get_intent(&mut decoder, version)?;
    if profiles(intent).0 != run.solver {
        return Err(RecordingError::Invalid("projection profile intent"));
    }
    decoder.end()?;
    let native = RecordingRun {
        solver: provider::PROFILE.into(),
        configuration,
        ..run.clone()
    };
    let config = provider::configured_configuration(&native)?;
    intent
        .validate_configuration(&config)
        .map_err(RecordingError::Invalid)?;
    Ok((native, intent))
}
/// Decode immutable native configuration and closed projection intent without
/// creating a Session, producer workspace, or runtime.
pub fn configuration(
    run: &RecordingRun,
) -> Result<(SessionConfiguration, ProjectionIntent), RecordingError> {
    let (native, intent) = native_header(run)?;
    Ok((provider::configured_configuration(&native)?, intent))
}
/// Capture actual immutable constructor controls and projection intent.
pub fn projection_run(
    snapshot: &SessionSnapshot<'_>,
    provenance: &str,
) -> Result<RecordingRun, RecordingError> {
    let intent = snapshot
        .projection_intent()
        .ok_or(RecordingError::Unsupported(
            "projection capture of legacy Session",
        ))?;
    let mut native = provider::native_run(snapshot, provenance)?;
    let mut encoder = Encoder::default();
    put_wrapped(&mut encoder, &native.configuration, "RSC1:")?;
    put_intent(&mut encoder, intent);
    native.configuration = encoder.text(profiles(intent).1)?;
    native.solver = profiles(intent).0.into();
    native_header(&native)?;
    Ok(native)
}
fn native_receipt(
    snapshot: &SessionSnapshot<'_>,
) -> Result<Option<RecordedConfiguredReceipt>, RecordingError> {
    if snapshot.stage() == SnapshotStage::Constructor {
        return Ok(None);
    }
    let report = snapshot
        .provider_report()
        .ok_or(RecordingError::Invalid("projection native report absent"))?;
    let attempt = snapshot
        .projection_attempt()
        .ok_or(RecordingError::Invalid(
            "projection accepted attempt absent",
        ))?;
    let run = projection_run(snapshot, "projection selected role")?;
    validate_attempt_source(
        &run,
        snapshot.configuration(),
        snapshot.projection_intent().unwrap(),
        snapshot.grid(),
        attempt,
    )?;
    let method = attempt
        .selected_native
        .as_ref()
        .ok_or(RecordingError::Invalid("projection selected method absent"))?
        .implementation;
    // This is an admissibility check, not stripping a supplied-witness report.
    if report.selected != method.id()
        || report.attempted.is_some()
        || report.rejection.is_some()
        || report.candidate.is_some()
    {
        return Err(RecordingError::Invalid(
            "projection selected report is not native-only",
        ));
    }
    Ok(Some(RecordedConfiguredReceipt {
        binding: None,
        context: None,
        viscosity_applied: false,
        dynamic_viscosity: 0.0,
        prediction_called: false,
        selected: RecordedPressureSelection::Native(method),
        attempted: None,
        rejection: None,
        candidate: None,
        accepted: report.accepted,
        total: report.total,
        inference: report.inference,
        qualification: report.qualification,
        fallback: report.fallback,
    }))
}
/// Capture accepted arrays once, retaining an unchanged native-only RCP2 payload.
pub fn projection_publication(
    snapshot: &SessionSnapshot<'_>,
    ordinal: usize,
    limit: usize,
) -> Result<RecordedStep, RecordingError> {
    let run = projection_run(snapshot, "projection publication source")?;
    let attempt = snapshot.projection_attempt();
    if (snapshot.stage() == SnapshotStage::Accepted) != attempt.is_some() {
        return Err(RecordingError::Invalid("projection snapshot receipt stage"));
    }
    let receipt = native_receipt(snapshot)?;
    let method = receipt
        .as_ref()
        .map_or(snapshot.configuration().implementation, |r| {
            match r.selected {
                RecordedPressureSelection::Native(method) => method,
                _ => unreachable!("native_receipt only emits native selection"),
            }
        });
    let mut step = provider::capture_native_with_method(snapshot, ordinal, limit, receipt, method)?;
    let mut encoder = Encoder::default();
    encoder.0.extend_from_slice(&provider::source_binding(&run));
    put_wrapped(&mut encoder, &step.controls, "RCP2:")?;
    encoder.boolean(attempt.is_some());
    if let Some(attempt) = attempt {
        put_attempt(
            &mut encoder,
            attempt,
            snapshot.configuration(),
            snapshot.grid(),
        )?;
    }
    step.controls = encoder.text(profiles(snapshot.projection_intent().unwrap()).2)?;
    if step.payload_bytes()? > limit {
        return Err(RecordingError::Limit("projection publication"));
    }
    if let Some(attempt) = attempt {
        validate_attempt_source(
            &run,
            snapshot.configuration(),
            snapshot.projection_intent().unwrap(),
            snapshot.grid(),
            attempt,
        )?;
        let report = snapshot
            .report()
            .ok_or(RecordingError::Invalid("projection snapshot report absent"))?;
        let selected = snapshot
            .provider_report()
            .ok_or(RecordingError::Invalid("projection native report absent"))?;
        if publication_generation(attempt) != Some(step.generation)
            || attempt.actual_dt.map(f64::to_bits) != Some(report.carrier.step.dt.to_bits())
            || (attempt.interval_start + report.carrier.step.dt).to_bits() != step.time.to_bits()
            || attempt.output_sha256 != Some(crate::workflow::input_digest(snapshot.state()))
            || attempt.selected_native.as_ref().is_none_or(|native| {
                report_bits(native.pressure) != report_bits(selected.accepted)
                    || native.total != selected.total
                    || native.inference != selected.inference
                    || native.qualification != selected.qualification
                    || native.fallback != selected.fallback
            })
        {
            return Err(RecordingError::Invalid(
                "projection captured publication association",
            ));
        }
    }
    Ok(step)
}

/// Exact accepted native fields plus their distinct projection observation.
/// For the selection profile, `native.configuration` is the authoritative outer
/// fallback constructor while `native.receipt.selected` identifies the validated
/// actual producer or fallback. This invariant does not apply to old configured readers.
pub struct RecordedProjectionSession {
    pub native: RecordedConfiguredSession,
    pub intent: ProjectionIntent,
    pub attempt: Option<Arc<ProjectionAttempt>>,
}
impl RecordedProjectionSession {
    pub fn decode(run: &RecordingRun, step: &RecordedStep) -> Result<Self, RecordingError> {
        let (header, intent) = native_header(run)?;
        let config = provider::configured_configuration(&header)?;
        let grid = recording_grid(run)?;
        let mut decoder = Decoder::text(&step.controls, profiles(intent).2)?;
        if decoder.array::<32>()? != provider::source_binding(run) {
            return Err(RecordingError::Invalid("projection publication source"));
        }
        let controls = get_wrapped(&mut decoder, "RCP2:")?;
        let attempt = if decoder.boolean()? {
            Some(Arc::new(get_attempt(&mut decoder, &config, &grid)?))
        } else {
            None
        };
        decoder.end()?;
        // Validate the outer role before varying the sole inner constructor field.
        let method = if let Some(a) = attempt.as_deref() {
            validate_attempt_source(run, &config, intent, &grid, a)?;
            a.selected_native
                .as_ref()
                .ok_or(RecordingError::Invalid("projection selected method absent"))?
                .implementation
        } else {
            config.implementation
        };
        let adapted = provider::native_header_with_method(&header, method)?;
        let mut native = provider::decode_native_with_controls(&adapted, step, &controls)?;
        native.configuration = config;
        validate_publication(run, &config, intent, &native, attempt.as_deref())?;
        Ok(Self {
            native,
            intent,
            attempt,
        })
    }
}

fn validate_attempt_source(
    run: &RecordingRun,
    config: &SessionConfiguration,
    intent: ProjectionIntent,
    grid: &crate::GridGeometry,
    attempt: &ProjectionAttempt,
) -> Result<(), RecordingError> {
    attempt
        .validate(config, grid)
        .map_err(RecordingError::Invalid)?;
    if attempt.source != run.identity || attempt.intent != intent {
        return Err(RecordingError::Invalid(
            "projection attempt source or intent",
        ));
    }
    Ok(())
}
fn validate_publication(
    run: &RecordingRun,
    config: &SessionConfiguration,
    intent: ProjectionIntent,
    native: &RecordedConfiguredSession,
    attempt: Option<&ProjectionAttempt>,
) -> Result<(), RecordingError> {
    if native.generation == 0 {
        if attempt.is_some() || native.receipt.is_some() {
            return Err(RecordingError::Invalid("projection constructor attempt"));
        }
        return Ok(());
    }
    let attempt = attempt.ok_or(RecordingError::Invalid(
        "projection accepted attempt absent",
    ))?;
    validate_attempt_source(run, config, intent, &native.grid, attempt)?;
    if publication_generation(attempt) != Some(native.generation)
        || attempt.actual_dt.map(f64::to_bits) != native.dt.map(f64::to_bits)
        || native
            .dt
            .is_none_or(|dt| (attempt.interval_start + dt).to_bits() != native.time.to_bits())
        || attempt.output_sha256 != Some(super::workflow::native_state_digest(native)?)
    {
        return Err(RecordingError::Invalid(
            "projection publication association",
        ));
    }
    let receipt = native
        .receipt
        .as_ref()
        .ok_or(RecordingError::Invalid("projection native receipt absent"))?;
    if receipt.binding.is_some()
        || receipt.context.is_some()
        || receipt.prediction_called
        || receipt.attempted.is_some()
        || receipt.rejection.is_some()
        || receipt.candidate.is_some()
        || receipt.viscosity_applied
        || receipt.dynamic_viscosity.to_bits() != 0.0f64.to_bits()
        || attempt.selected_native.as_ref().is_none_or(|selected| {
            receipt.selected != RecordedPressureSelection::Native(selected.implementation)
        })
        || selected_pressure(attempt).map(report_bits) != Some(report_bits(receipt.accepted))
        || attempt.selected_native.as_ref().is_none_or(|native| {
            native.total != receipt.total
                || native.inference != receipt.inference
                || native.qualification != receipt.qualification
                || native.fallback != receipt.fallback
        })
    {
        return Err(RecordingError::Invalid(
            "projection native receipt association",
        ));
    }
    Ok(())
}
/// Validate a selected publication and its immediate retained input without
/// materializing a second history or executing a producer.
pub fn selection(
    reader: &RecordingReader,
    step: &RecordedStep,
) -> Result<RecordedProjectionSession, RecordingError> {
    let current = RecordedProjectionSession::decode(reader.run(), step)?;
    if let Some(attempt) = &current.attempt
        && step.ordinal > 0
    {
        let row = reader.select(step.ordinal - 1)?;
        let prior = RecordedProjectionSession::decode(reader.run(), &row)?;
        validate_prior(&prior, attempt)?;
    }
    Ok(current)
}
fn validate_prior(
    prior: &RecordedProjectionSession,
    attempt: &ProjectionAttempt,
) -> Result<(), RecordingError> {
    if prior.native.generation != attempt.accepted_generation
        || prior.native.time.to_bits() != attempt.interval_start.to_bits()
        || super::workflow::native_state_digest(&prior.native)? != attempt.input_sha256
        || prior
            .attempt
            .as_ref()
            .is_some_and(|previous| previous.attempt_id >= attempt.attempt_id)
    {
        return Err(RecordingError::Invalid(
            "projection prior native input or attempt",
        ));
    }
    Ok(())
}
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ProjectionHistoryCoverage {
    pub verified_prior_links: usize,
    pub unavailable_prior_links: usize,
}
/// Stream exact typed rows, retaining only prior scalar identity and digest.
pub fn verify_projection_recording(
    reader: &RecordingReader,
) -> Result<ProjectionHistoryCoverage, RecordingError> {
    reader.verify()?;
    let (config, _) = configuration(reader.run())?;
    let status = reader.status()?;
    if status
        .accepted_generation
        .is_some_and(|generation| generation > config.max_steps as u64)
    {
        return Err(RecordingError::Invalid("projection terminal quota"));
    }
    let mut previous: Option<(u64, f64, [u8; 32], Option<u64>)> = None;
    let mut coverage = ProjectionHistoryCoverage::default();
    for ordinal in 0..reader.count()? {
        let row = reader.select(ordinal)?;
        let current = RecordedProjectionSession::decode(reader.run(), &row)?;
        if let Some(attempt) = &current.attempt {
            if let Some((generation, time, digest, id)) = previous {
                if generation != attempt.accepted_generation
                    || time.to_bits() != attempt.interval_start.to_bits()
                    || digest != attempt.input_sha256
                    || id.is_some_and(|id| id >= attempt.attempt_id)
                {
                    return Err(RecordingError::Invalid(
                        "projection history native input or attempt",
                    ));
                }
                coverage.verified_prior_links += 1;
            } else {
                coverage.unavailable_prior_links += 1;
            }
        }
        previous = Some((
            current.native.generation,
            current.native.time,
            super::workflow::native_state_digest(&current.native)?,
            current.attempt.as_ref().map(|attempt| attempt.attempt_id),
        ));
    }
    Ok(coverage)
}

/// Terminal numerical position and the latest observed attempt are distinct.
/// Accepted fields omitted by storage failure remain an unrecorded tail.
#[derive(Debug, Clone)]
pub struct ProjectionTermination {
    pub cause: RuntimeTerminationCause,
    pub detail: String,
    pub accepted_generation: u64,
    pub accepted_time: f64,
    pub attempt: Option<Arc<ProjectionAttempt>>,
    source: [u8; 32],
}
impl ProjectionTermination {
    pub fn capture(
        snapshot: &SessionSnapshot<'_>,
        cause: RuntimeTerminationCause,
        detail: &str,
        attempt: Option<&Arc<ProjectionAttempt>>,
    ) -> Result<Self, RecordingError> {
        let run = projection_run(snapshot, "projection terminal source")?;
        let state = snapshot.state();
        let value = Self {
            cause,
            detail: detail.into(),
            accepted_generation: state.carrier.generation,
            accepted_time: state.carrier.time,
            attempt: attempt.cloned(),
            source: provider::source_binding(&run),
        };
        value.validate(&run)?;
        if let Some(attempt) = &value.attempt {
            let actual = crate::workflow::input_digest(state);
            if if publication_generation(attempt).is_some() {
                attempt.output_sha256 != Some(actual)
            } else {
                attempt.input_sha256 != actual
            } {
                return Err(RecordingError::Invalid(
                    "projection terminal snapshot digest",
                ));
            }
        }
        value.encode(
            snapshot.configuration(),
            snapshot.grid(),
            snapshot.projection_intent().unwrap(),
        )?;
        Ok(value)
    }
    fn validate(&self, run: &RecordingRun) -> Result<(), RecordingError> {
        let (config, intent) = configuration(run)?;
        if self.cause == RuntimeTerminationCause::RuntimeFailure {
            return Err(RecordingError::Unsupported(
                "projection has no runtime execution",
            ));
        }
        if self.cause == RuntimeTerminationCause::StartupFailure
            && (self.accepted_generation != 0 || self.attempt.is_some())
        {
            return Err(RecordingError::Invalid("projection startup terminal work"));
        }
        if self.source != provider::source_binding(run)
            || self.detail.len() > 1000
            || self.accepted_generation > config.max_steps as u64
            || !self.accepted_time.is_finite()
            || (self.accepted_generation == 0 && self.accepted_time.to_bits() != 0.0f64.to_bits())
            || (self.accepted_generation > 0 && self.accepted_time <= 0.0)
        {
            return Err(RecordingError::Invalid(
                "projection terminal source or position",
            ));
        }
        if let Some(attempt) = &self.attempt {
            validate_attempt_source(run, &config, intent, &recording_grid(run)?, attempt)?;
            if let Some(generation) = publication_generation(attempt) {
                if generation != self.accepted_generation
                    || attempt.actual_dt.is_none_or(|dt| {
                        (attempt.interval_start + dt).to_bits() != self.accepted_time.to_bits()
                    })
                {
                    return Err(RecordingError::Invalid(
                        "projection terminal accepted association",
                    ));
                }
            } else if attempt.accepted_generation != self.accepted_generation
                || attempt.interval_start.to_bits() != self.accepted_time.to_bits()
                || self.cause == RuntimeTerminationCause::Completed
            {
                return Err(RecordingError::Invalid(
                    "projection terminal attempted association",
                ));
            }
        } else if self.cause == RuntimeTerminationCause::Completed && self.accepted_generation > 0 {
            return Err(RecordingError::Invalid(
                "projection completed attempt absent",
            ));
        }
        Ok(())
    }
    fn encode(
        &self,
        config: &SessionConfiguration,
        grid: &crate::GridGeometry,
        intent: ProjectionIntent,
    ) -> Result<String, RecordingError> {
        let mut encoder = Encoder::default();
        encoder.0.extend_from_slice(&self.source);
        put_cause(&mut encoder, self.cause);
        encoder.string(&self.detail)?;
        encoder.u64(self.accepted_generation);
        encoder.f64(self.accepted_time);
        encoder.boolean(self.attempt.is_some());
        if let Some(attempt) = &self.attempt {
            put_attempt(&mut encoder, attempt, config, grid)?;
        }
        encoder.text(profiles(intent).3)
    }
    pub fn finish(&self, recorder: &mut StepRecorder) -> Result<(), RecordingError> {
        self.validate(recorder.run())?;
        let (config, intent) = configuration(recorder.run())?;
        recorder.finish(
            cause_outcome(self.cause),
            &self.encode(&config, &recording_grid(recorder.run())?, intent)?,
            Some(self.accepted_generation),
        )
    }
}
#[derive(Debug, Clone)]
pub struct ProjectionTerminationAudit {
    pub termination: ProjectionTermination,
    pub coverage: RuntimeTerminalCoverage,
}
/// Typed RPT1/RPT2 inspection. Unknown source/versions refuse; this never falls back
/// to G1 or interprets a missing publication as an accepted numerical row.
pub fn terminal(reader: &RecordingReader) -> Result<ProjectionTerminationAudit, RecordingError> {
    verify_projection_recording(reader)?;
    let (config, intent) = configuration(reader.run())?;
    let grid = recording_grid(reader.run())?;
    let status = reader.status()?;
    if status.outcome == "active" {
        return Err(RecordingError::Unsupported("projection active termination"));
    }
    let mut decoder = Decoder::text(&status.detail, profiles(intent).3)?;
    let source = decoder.array()?;
    let cause = get_cause(&mut decoder)?;
    let detail = decoder.string()?;
    let accepted_generation = decoder.u64()?;
    let accepted_time = decoder.f64()?;
    let attempt = if decoder.boolean()? {
        Some(Arc::new(get_attempt(&mut decoder, &config, &grid)?))
    } else {
        None
    };
    decoder.end()?;
    let termination = ProjectionTermination {
        source,
        cause,
        detail,
        accepted_generation,
        accepted_time,
        attempt,
    };
    termination.validate(reader.run())?;
    if status.accepted_generation != Some(accepted_generation)
        || status.outcome != cause_outcome(cause)
    {
        return Err(RecordingError::Invalid("projection terminal lifecycle"));
    }
    let count = reader.count()?;
    let coverage = if count == 0 {
        if accepted_generation == 0 {
            RuntimeTerminalCoverage::NoRecordedPublication
        } else {
            RuntimeTerminalCoverage::UnrecordedAcceptedTail {
                last_recorded_generation: None,
            }
        }
    } else {
        let row = reader.select(count - 1)?;
        let last = RecordedProjectionSession::decode(reader.run(), &row)?;
        if accepted_generation < last.native.generation
            || (accepted_generation == last.native.generation
                && accepted_time.to_bits() != last.native.time.to_bits())
            || (accepted_generation > last.native.generation && accepted_time <= last.native.time)
        {
            return Err(RecordingError::Invalid("projection terminal recorded tail"));
        }
        if let Some(attempt) = &termination.attempt {
            if publication_generation(attempt) == Some(last.native.generation) {
                let recorded = last.attempt.as_ref().ok_or(RecordingError::Invalid(
                    "projection terminal accepted evidence absent",
                ))?;
                let mut a = Encoder::default();
                let mut b = Encoder::default();
                put_attempt(&mut a, attempt, &config, &grid)?;
                put_attempt(&mut b, recorded, &config, &grid)?;
                if a.0 != b.0 {
                    return Err(RecordingError::Invalid(
                        "projection terminal publication differs",
                    ));
                }
            } else {
                if last
                    .attempt
                    .as_ref()
                    .is_some_and(|prior| prior.attempt_id >= attempt.attempt_id)
                {
                    return Err(RecordingError::Invalid(
                        "projection terminal attempt identity",
                    ));
                }
                if last.native.generation == attempt.accepted_generation {
                    validate_prior(&last, attempt)?;
                }
            }
        }
        if accepted_generation == last.native.generation {
            RuntimeTerminalCoverage::VerifiedLastPublication
        } else {
            RuntimeTerminalCoverage::UnrecordedAcceptedTail {
                last_recorded_generation: Some(last.native.generation),
            }
        }
    };
    Ok(ProjectionTerminationAudit {
        termination,
        coverage,
    })
}
fn cause_outcome(cause: RuntimeTerminationCause) -> &'static str {
    match cause {
        RuntimeTerminationCause::Completed => "completed",
        RuntimeTerminationCause::Cancelled => "cancelled",
        _ => "failed",
    }
}
fn put_cause(e: &mut Encoder, cause: RuntimeTerminationCause) {
    e.u8(match cause {
        RuntimeTerminationCause::Completed => 0,
        RuntimeTerminationCause::Cancelled => 1,
        RuntimeTerminationCause::SessionDeadline => 2,
        RuntimeTerminationCause::RuntimeFailure => 3,
        RuntimeTerminationCause::StartupFailure => 4,
        RuntimeTerminationCause::NumericalFailure => 5,
        RuntimeTerminationCause::StorageFailure => 6,
    });
}
fn get_cause(d: &mut Decoder) -> Result<RuntimeTerminationCause, RecordingError> {
    match d.u8()? {
        0 => Ok(RuntimeTerminationCause::Completed),
        1 => Ok(RuntimeTerminationCause::Cancelled),
        2 => Ok(RuntimeTerminationCause::SessionDeadline),
        3 => Ok(RuntimeTerminationCause::RuntimeFailure),
        4 => Ok(RuntimeTerminationCause::StartupFailure),
        5 => Ok(RuntimeTerminationCause::NumericalFailure),
        6 => Ok(RuntimeTerminationCause::StorageFailure),
        _ => Err(RecordingError::Unsupported("projection terminal cause")),
    }
}
fn report_bits(report: crate::PressureReport) -> [u64; 4] {
    [
        report.iterations as u64,
        report.true_residual_l2.to_bits(),
        report.true_residual_max.to_bits(),
        report.predicted_divergence_max.to_bits(),
    ]
}

fn publication_generation(attempt: &ProjectionAttempt) -> Option<u64> {
    match attempt.outcome {
        crate::projection_step::ProjectionAttemptOutcome::Published { generation } => {
            Some(generation)
        }
        _ => None,
    }
}
fn selected_pressure(attempt: &ProjectionAttempt) -> Option<crate::PressureReport> {
    attempt
        .selected_native
        .as_ref()
        .map(|native| native.pressure)
}

fn put_report(e: &mut Encoder, report: crate::PressureReport) {
    for value in report_bits(report) {
        e.u64(value);
    }
}
fn get_report(d: &mut Decoder) -> Result<crate::PressureReport, RecordingError> {
    Ok(crate::PressureReport {
        iterations: d.usize()?,
        true_residual_l2: d.f64()?,
        true_residual_max: d.f64()?,
        predicted_divergence_max: d.f64()?,
    })
}
fn put_duration(e: &mut Encoder, duration: std::time::Duration) {
    e.u64(duration.as_secs());
    e.u32(duration.subsec_nanos());
}
fn get_duration(d: &mut Decoder) -> Result<std::time::Duration, RecordingError> {
    let secs = d.u64()?;
    let nanos = d.u32()?;
    if nanos >= 1_000_000_000 {
        return Err(RecordingError::Invalid("projection duration"));
    }
    Ok(std::time::Duration::new(secs, nanos))
}
fn put_method(e: &mut Encoder, method: crate::PressureImplementation) {
    e.u8(match method {
        crate::PressureImplementation::JacobiPcgV1 => 0,
        crate::PressureImplementation::SymmetricGaussSeidelPcgV1 => 1,
    });
}
fn get_method(d: &mut Decoder) -> Result<crate::PressureImplementation, RecordingError> {
    match d.u8()? {
        0 => Ok(crate::PressureImplementation::JacobiPcgV1),
        1 => Ok(crate::PressureImplementation::SymmetricGaussSeidelPcgV1),
        _ => Err(RecordingError::Unsupported("projection native method")),
    }
}
fn put_axis(e: &mut Encoder, axis: crate::Axis) {
    e.u8(match axis {
        crate::Axis::X => 0,
        crate::Axis::Y => 1,
        crate::Axis::Z => 2,
    });
}
fn get_axis(d: &mut Decoder) -> Result<crate::Axis, RecordingError> {
    match d.u8()? {
        0 => Ok(crate::Axis::X),
        1 => Ok(crate::Axis::Y),
        2 => Ok(crate::Axis::Z),
        _ => Err(RecordingError::Unsupported("projection diagnostic axis")),
    }
}
fn put_stage(e: &mut Encoder, stage: crate::projection_step::ProjectionAttemptStage) {
    use crate::projection_step::ProjectionAttemptStage::*;
    e.u8(match stage {
        SessionGuard => 0,
        CarrierPreparation => 1,
        NativeProducer => 2,
        HostQualification => 3,
        CarrierTransport => 4,
        VolumeTransport => 5,
        BeforeCommit => 6,
    });
}
fn get_stage(
    d: &mut Decoder,
) -> Result<crate::projection_step::ProjectionAttemptStage, RecordingError> {
    use crate::projection_step::ProjectionAttemptStage::*;
    match d.u8()? {
        0 => Ok(SessionGuard),
        1 => Ok(CarrierPreparation),
        2 => Ok(NativeProducer),
        3 => Ok(HostQualification),
        4 => Ok(CarrierTransport),
        5 => Ok(VolumeTransport),
        6 => Ok(BeforeCommit),
        _ => Err(RecordingError::Unsupported("projection attempt stage")),
    }
}
fn put_code(e: &mut Encoder, code: crate::projection_step::ProjectionFailureCode) {
    use crate::projection_step::ProjectionFailureCode::*;
    e.u8(match code {
        Stopped => 0,
        StepLimit => 1,
        Deadline => 2,
        IntentMismatch => 3,
        WorkspaceMismatch => 4,
        InvalidInput => 5,
        NativePressure => 6,
        HostQualification => 7,
        Carrier => 8,
        Volume => 9,
        ReceiptAliased => 10,
        GenerationExhausted => 11,
        UnrepresentableMixedQualificationFallback => 12,
    });
}
fn get_code(
    d: &mut Decoder,
) -> Result<crate::projection_step::ProjectionFailureCode, RecordingError> {
    use crate::projection_step::ProjectionFailureCode::*;
    match d.u8()? {
        0 => Ok(Stopped),
        1 => Ok(StepLimit),
        2 => Ok(Deadline),
        3 => Ok(IntentMismatch),
        4 => Ok(WorkspaceMismatch),
        5 => Ok(InvalidInput),
        6 => Ok(NativePressure),
        7 => Ok(HostQualification),
        8 => Ok(Carrier),
        9 => Ok(Volume),
        10 => Ok(ReceiptAliased),
        11 => Ok(GenerationExhausted),
        12 => Ok(UnrepresentableMixedQualificationFallback),
        _ => Err(RecordingError::Unsupported("projection failure code")),
    }
}
fn put_pressure_rejection(
    e: &mut Encoder,
    reason: crate::projection_step::ProjectionPressureRejection,
) {
    use crate::projection_step::ProjectionPressureRejection::*;
    e.u8(match reason {
        NonFiniteOutput => 0,
        NonZeroGauge => 1,
        QualityGate => 2,
    });
}
fn get_pressure_rejection(
    d: &mut Decoder,
) -> Result<crate::projection_step::ProjectionPressureRejection, RecordingError> {
    use crate::projection_step::ProjectionPressureRejection::*;
    match d.u8()? {
        0 => Ok(NonFiniteOutput),
        1 => Ok(NonZeroGauge),
        2 => Ok(QualityGate),
        _ => Err(RecordingError::Unsupported("projection pressure rejection")),
    }
}
fn put_rejection(e: &mut Encoder, reason: &crate::projection_step::ProjectionRejectionFacts) {
    use crate::projection_step::{
        ProjectionFieldRejection as FieldReason, ProjectionRejectionFacts::*,
    };
    match reason {
        Field(reason) => {
            e.u8(0);
            match reason {
                FieldReason::LengthMismatch => e.u8(0),
                FieldReason::NonFiniteInput => e.u8(1),
                FieldReason::NonZeroWallVelocity { axis, index } => {
                    e.u8(2);
                    put_axis(e, *axis);
                    e.usize(*index);
                }
                FieldReason::ArithmeticFailure => e.u8(3),
            }
        }
        MomentumWitness { axis, index } => {
            e.u8(1);
            put_axis(e, *axis);
            e.usize(*index);
        }
        Pressure(reason) => {
            e.u8(2);
            put_pressure_rejection(e, *reason);
        }
        ActualDivergence { maximum, limit } => {
            e.u8(3);
            e.f64(*maximum);
            e.f64(*limit);
        }
    }
}
fn get_rejection(
    d: &mut Decoder,
) -> Result<crate::projection_step::ProjectionRejectionFacts, RecordingError> {
    use crate::projection_step::{
        ProjectionFieldRejection as FieldReason, ProjectionRejectionFacts::*,
    };
    match d.u8()? {
        0 => Ok(Field(match d.u8()? {
            0 => FieldReason::LengthMismatch,
            1 => FieldReason::NonFiniteInput,
            2 => FieldReason::NonZeroWallVelocity {
                axis: get_axis(d)?,
                index: d.usize()?,
            },
            3 => FieldReason::ArithmeticFailure,
            _ => return Err(RecordingError::Unsupported("projection field rejection")),
        })),
        1 => Ok(MomentumWitness {
            axis: get_axis(d)?,
            index: d.usize()?,
        }),
        2 => Ok(Pressure(get_pressure_rejection(d)?)),
        3 => Ok(ActualDivergence {
            maximum: d.f64()?,
            limit: d.f64()?,
        }),
        _ => Err(RecordingError::Unsupported(
            "projection qualification rejection",
        )),
    }
}
fn put_attempt(
    e: &mut Encoder,
    attempt: &ProjectionAttempt,
    config: &SessionConfiguration,
    grid: &crate::GridGeometry,
) -> Result<(), RecordingError> {
    use crate::projection_step::{ProjectionAttemptOutcome::*, ProjectionWitnessOutcome};
    use crate::solver_provider::composition::projection::ProjectionSelection;
    attempt
        .validate(config, grid)
        .map_err(RecordingError::Invalid)?;
    e.u32(attempt.version);
    put_intent(e, attempt.intent);
    e.u64(attempt.source.run_id);
    e.string(&attempt.source.source_revision)?;
    e.u64(attempt.attempt_id);
    e.u64(attempt.accepted_generation);
    e.boolean(attempt.attempted_generation.is_some());
    if let Some(generation) = attempt.attempted_generation {
        e.u64(generation);
    }
    e.f64(attempt.interval_start);
    e.f64(attempt.requested_dt);
    e.boolean(attempt.actual_dt.is_some());
    if let Some(dt) = attempt.actual_dt {
        e.f64(dt);
    }
    e.0.extend_from_slice(&attempt.input_sha256);
    e.0.extend_from_slice(&attempt.controls_sha256);
    e.boolean(attempt.provisional_input_id.is_some());
    if let Some(id) = attempt.provisional_input_id {
        e.0.extend_from_slice(&id);
    }
    e.boolean(attempt.producer.is_some());
    if let Some(producer) = &attempt.producer {
        put_method(e, producer.implementation);
        put_report(e, producer.pressure);
        e.f64(producer.residual_l2_limit);
        put_duration(e, producer.elapsed);
    }
    if attempt.version == 2 {
        e.boolean(attempt.producer_failure.is_some());
        if let Some(failure) = &attempt.producer_failure {
            put_method(e, failure.implementation);
            match failure.failure {
                crate::projection_step::ProjectionProducerFailure::IterationLimit {
                    iterations,
                    residual_max,
                } => {
                    e.u8(0);
                    e.u64(iterations as u64);
                    e.f64(residual_max);
                }
            }
            e.f64(failure.residual_l2_limit);
            put_duration(e, failure.elapsed);
        }
    }
    e.boolean(attempt.qualification.is_some());
    if let Some(qualification) = &attempt.qualification {
        e.u8(match qualification.selection {
            ProjectionSelection::SuppliedCandidate => 0,
            ProjectionSelection::NativeBaseline => 1,
            ProjectionSelection::NativeFallback => 2,
        });
        e.boolean(qualification.rejection.is_some());
        if let Some(reason) = &qualification.rejection {
            put_rejection(e, reason);
        }
        e.boolean(qualification.witness.is_some());
        if let Some(witness) = &qualification.witness {
            match witness.outcome {
                ProjectionWitnessOutcome::Accepted => e.u8(0),
                ProjectionWitnessOutcome::Rejected(reason) => {
                    e.u8(1);
                    put_pressure_rejection(e, reason);
                }
            }
            e.boolean(witness.diagnostics.is_some());
            if let Some(report) = witness.diagnostics {
                put_report(e, report);
            }
        }
        e.boolean(qualification.native_fallback.is_some());
        if let Some(fallback) = &qualification.native_fallback {
            put_method(e, fallback.implementation);
            put_report(e, fallback.pressure);
            put_duration(e, fallback.elapsed);
        }
        e.f64(qualification.actual_divergence_max);
        put_duration(e, qualification.elapsed);
    }
    e.boolean(attempt.selected_native.is_some());
    if let Some(native) = &attempt.selected_native {
        put_method(e, native.implementation);
        put_report(e, native.pressure);
        for duration in [
            native.total,
            native.inference,
            native.qualification,
            native.fallback,
        ] {
            put_duration(e, duration);
        }
    }
    match attempt.outcome {
        Published { generation } => {
            e.u8(0);
            e.u64(generation);
        }
        Refused { stage, code } => {
            e.u8(1);
            put_stage(e, stage);
            put_code(e, code);
        }
        Cancelled { stage } => {
            e.u8(2);
            put_stage(e, stage);
        }
        Failed { stage, code } => {
            e.u8(3);
            put_stage(e, stage);
            put_code(e, code);
        }
    }
    e.boolean(attempt.output_sha256.is_some());
    if let Some(hash) = attempt.output_sha256 {
        e.0.extend_from_slice(&hash);
    }
    e.string(&attempt.diagnostic)
}
fn get_attempt(
    d: &mut Decoder,
    config: &SessionConfiguration,
    grid: &crate::GridGeometry,
) -> Result<ProjectionAttempt, RecordingError> {
    use crate::projection_step::*;
    use crate::solver_provider::composition::projection::ProjectionSelection;
    let version = d.u32()?;
    if version != 1 && version != 2 {
        return Err(RecordingError::Unsupported("projection attempt version"));
    }
    let intent = get_intent(d, version)?;
    let source = crate::SessionIdentity {
        run_id: d.u64()?,
        source_revision: d.string()?,
    };
    let attempt_id = d.u64()?;
    let accepted_generation = d.u64()?;
    let attempted_generation = if d.boolean()? { Some(d.u64()?) } else { None };
    let interval_start = d.f64()?;
    let requested_dt = d.f64()?;
    let actual_dt = if d.boolean()? { Some(d.f64()?) } else { None };
    let input_sha256 = d.array()?;
    let controls_sha256 = d.array()?;
    let provisional_input_id = if d.boolean()? { Some(d.array()?) } else { None };
    let producer = if d.boolean()? {
        Some(NativeProjectionProducerFacts {
            implementation: get_method(d)?,
            pressure: get_report(d)?,
            residual_l2_limit: d.f64()?,
            elapsed: get_duration(d)?,
        })
    } else {
        None
    };
    let producer_failure = if version == 2 && d.boolean()? {
        Some(NativeProjectionProducerFailureFacts {
            implementation: get_method(d)?,
            failure: match d.u8()? {
                0 => ProjectionProducerFailure::IterationLimit {
                    iterations: d.usize()?,
                    residual_max: d.f64()?,
                },
                _ => return Err(RecordingError::Unsupported("projection producer failure")),
            },
            residual_l2_limit: d.f64()?,
            elapsed: get_duration(d)?,
        })
    } else {
        None
    };
    let qualification = if d.boolean()? {
        let selection = match d.u8()? {
            0 => ProjectionSelection::SuppliedCandidate,
            1 => ProjectionSelection::NativeBaseline,
            2 => ProjectionSelection::NativeFallback,
            _ => return Err(RecordingError::Unsupported("projection selection")),
        };
        let rejection = if d.boolean()? {
            Some(get_rejection(d)?)
        } else {
            None
        };
        let witness = if d.boolean()? {
            Some(ProjectionPressureWitnessFacts {
                outcome: match d.u8()? {
                    0 => ProjectionWitnessOutcome::Accepted,
                    1 => ProjectionWitnessOutcome::Rejected(get_pressure_rejection(d)?),
                    _ => return Err(RecordingError::Unsupported("projection witness outcome")),
                },
                diagnostics: if d.boolean()? {
                    Some(get_report(d)?)
                } else {
                    None
                },
            })
        } else {
            None
        };
        let native_fallback = if d.boolean()? {
            Some(NativePressureSolveFacts {
                implementation: get_method(d)?,
                pressure: get_report(d)?,
                elapsed: get_duration(d)?,
            })
        } else {
            None
        };
        Some(ProjectionQualificationFacts {
            selection,
            rejection,
            witness,
            native_fallback,
            actual_divergence_max: d.f64()?,
            elapsed: get_duration(d)?,
        })
    } else {
        None
    };
    let selected_native = if d.boolean()? {
        Some(NativePressureDispatchFacts {
            implementation: get_method(d)?,
            pressure: get_report(d)?,
            total: get_duration(d)?,
            inference: get_duration(d)?,
            qualification: get_duration(d)?,
            fallback: get_duration(d)?,
        })
    } else {
        None
    };
    let outcome = match d.u8()? {
        0 => ProjectionAttemptOutcome::Published {
            generation: d.u64()?,
        },
        1 => ProjectionAttemptOutcome::Refused {
            stage: get_stage(d)?,
            code: get_code(d)?,
        },
        2 => ProjectionAttemptOutcome::Cancelled {
            stage: get_stage(d)?,
        },
        3 => ProjectionAttemptOutcome::Failed {
            stage: get_stage(d)?,
            code: get_code(d)?,
        },
        _ => return Err(RecordingError::Unsupported("projection attempt outcome")),
    };
    let output_sha256 = if d.boolean()? { Some(d.array()?) } else { None };
    let attempt = ProjectionAttempt {
        version,
        intent,
        source,
        attempt_id,
        accepted_generation,
        attempted_generation,
        interval_start,
        requested_dt,
        actual_dt,
        input_sha256,
        controls_sha256,
        provisional_input_id,
        producer,
        producer_failure,
        qualification,
        selected_native,
        outcome,
        output_sha256,
        diagnostic: d.string()?,
    };
    attempt
        .validate(config, grid)
        .map_err(RecordingError::Invalid)?;
    Ok(attempt)
}

#[cfg(test)]
#[path = "projection_tests.rs"]
mod tests;
