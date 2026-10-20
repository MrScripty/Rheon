//! Bounded runtime custody beside the host's pressure acceptance receipt.
//! Declarations and hashes are inspection evidence, not authentication, an RHS
//! reconstruction, a restart checkpoint, or a shutdown/preemption guarantee.
use super::*;
use crate::{
    recording::diagnostic::NoCandidateReason,
    solver_provider::{
        configured::ContextPressureRequest,
        context::{PressureContextMode, PressureHistoryStatus},
    },
};

const TERMINATION: &str = "RCT1:";

/// Exact accepted-candidate request lineage, with no retained field/history arrays.
#[derive(Debug, Clone)]
pub struct RuntimeRequestStamp {
    pub run_id: u64,
    pub grid: GridGeometry,
    pub density: f64,
    pub dynamic_viscosity: f64,
    pub context: AcceptedPressureContextStamp,
    pub binding: ContextProviderIdentity,
}
impl RuntimeRequestStamp {
    pub fn capture(
        request: &ContextPressureRequest<'_>,
        binding: &ContextProviderIdentity,
    ) -> Result<Self, RecordingError> {
        let c = &request.context;
        let run_id = c
            .run_id()
            .parse::<u64>()
            .map_err(|_| RecordingError::Invalid("runtime request run identity"))?;
        if c.mode() != PressureContextMode::AcceptedCandidate
            || run_id.to_string() != c.run_id()
            || c.prior().is_some_and(|p| {
                p.status != PressureHistoryStatus::Accepted
                    || !same_grid(p.grid, c.grid())
                    || p.density.to_bits() != c.density().to_bits()
            })
        {
            return Err(RecordingError::Invalid("runtime accepted request"));
        }
        binding
            .validate()
            .map_err(|_| RecordingError::Invalid("runtime binding"))?;
        let prior = c.prior();
        let stamp = Self {
            run_id,
            grid: c.grid().clone(),
            density: c.density(),
            dynamic_viscosity: request.dynamic_viscosity,
            context: AcceptedPressureContextStamp {
                generation: c.generation(),
                interval_start: c.interval_start(),
                interval_end: c.interval_end(),
                dt: c.dt(),
                native_prefix_steps: request.native_prefix_steps,
                prior_generation: prior.map(|p| p.generation),
                prior_interval_end: prior.map(|p| p.interval_end),
                prior_pressure_sha256: prior.map(|p| hash_values(p.values)),
                rhs_sha256: hash_values(c.rhs()),
            },
            binding: binding.clone(),
        };
        stamp.validate()?;
        Ok(stamp)
    }

    /// Validate intrinsic coherence; this does not attest a caller's declaration.
    pub fn validate(&self) -> Result<(), RecordingError> {
        self.binding
            .validate()
            .map_err(|_| RecordingError::Invalid("runtime binding"))?;
        let c = &self.context;
        let prior = c.prior_generation.is_some();
        if !self.density.is_finite()
            || self.density <= 0.0
            || !self.dynamic_viscosity.is_finite()
            || self.dynamic_viscosity < 0.0
            || !(self.dynamic_viscosity == 0.0 || self.dynamic_viscosity.is_normal())
            || c.generation == 0
            || !c.interval_start.is_finite()
            || c.interval_start < 0.0
            || !c.interval_end.is_finite()
            || c.interval_end <= c.interval_start
            || !c.dt.is_finite()
            || c.dt <= 0.0
            || (c.interval_start + c.dt).to_bits() != c.interval_end.to_bits()
            || u64::try_from(c.native_prefix_steps).is_err()
            || c.native_prefix_steps as u64 >= c.generation
            || prior != c.prior_interval_end.is_some()
            || prior != c.prior_pressure_sha256.is_some()
            || c.prior_generation
                .is_some_and(|g| g.checked_add(1) != Some(c.generation))
            || c.prior_interval_end
                .is_some_and(|t| t.to_bits() != c.interval_start.to_bits())
            || (c.generation == 1 && (prior || c.interval_start.to_bits() != 0.0_f64.to_bits()))
            || (c.generation > 1 && (!prior || c.interval_start <= 0.0))
        {
            return Err(RecordingError::Invalid("runtime request association"));
        }
        Ok(())
    }
}
impl PartialEq for RuntimeRequestStamp {
    fn eq(&self, other: &Self) -> bool {
        self.run_id == other.run_id
            && same_grid(&self.grid, &other.grid)
            && self.density.to_bits() == other.density.to_bits()
            && self.dynamic_viscosity.to_bits() == other.dynamic_viscosity.to_bits()
            && same_context(&self.context, &other.context)
            && self.binding == other.binding
    }
}
impl Eq for RuntimeRequestStamp {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeOutcome {
    CandidateReturned,
    NoCandidate {
        reason: NoCandidateReason,
        detail: String,
        /// Existing runtime writer accepted the request; not proof of consumption.
        dispatched: bool,
        /// Complete received invalid payload only; never a bounded-prefix digest.
        invalid_payload_sha256: Option<[u8; 32]>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeObservation {
    pub request: RuntimeRequestStamp,
    pub outcome: RuntimeOutcome,
}
impl RuntimeObservation {
    pub fn validate(&self) -> Result<(), RecordingError> {
        self.request.validate()?;
        if let RuntimeOutcome::NoCandidate {
            reason,
            detail,
            dispatched,
            invalid_payload_sha256,
        } = &self.outcome
        {
            bounded(detail)?;
            if invalid_payload_sha256.is_some()
                && (*reason != NoCandidateReason::InvalidOutput || !dispatched)
            {
                return Err(RecordingError::Invalid("runtime invalid payload custody"));
            }
        }
        Ok(())
    }
}

pub(super) fn validate_publication(
    observation: &RuntimeObservation,
    run: &RecordingRun,
    receipt: &RecordedConfiguredReceipt,
) -> Result<(), RecordingError> {
    observation.validate()?;
    validate_request_source(&observation.request, run)?;
    let request = &observation.request;
    if !receipt.prediction_called
        || receipt.binding.as_ref() != Some(&request.binding)
        || receipt
            .context
            .as_ref()
            .is_none_or(|c| !same_context(c, &request.context))
        || receipt.dynamic_viscosity.to_bits() != request.dynamic_viscosity.to_bits()
    {
        return Err(RecordingError::Invalid("runtime publication association"));
    }
    let matches = match &observation.outcome {
        RuntimeOutcome::CandidateReturned => matches!(
            receipt.rejection,
            None | Some(
                RecordedProviderRejection::QualityGate
                    | RecordedProviderRejection::NonFiniteOutput
                    | RecordedProviderRejection::NonZeroGauge
            )
        ),
        RuntimeOutcome::NoCandidate { reason, .. } => {
            receipt.candidate.is_none()
                && match reason {
                    NoCandidateReason::Unsupported | NoCandidateReason::UnsupportedContext => {
                        matches!(
                            receipt.rejection,
                            Some(RecordedProviderRejection::Unsupported(_))
                        )
                    }
                    NoCandidateReason::RuntimeFailure | NoCandidateReason::InvalidOutput => {
                        matches!(
                            receipt.rejection,
                            Some(RecordedProviderRejection::Execution(_))
                        )
                    }
                    NoCandidateReason::Cancelled => false,
                }
        }
    };
    if !matches {
        return Err(RecordingError::Invalid("runtime host outcome association"));
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeTerminationCause {
    Completed,
    Cancelled,
    SessionDeadline,
    RuntimeFailure,
    StartupFailure,
    NumericalFailure,
    StorageFailure,
}
impl RuntimeTerminationCause {
    fn outcome(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
            _ => "failed",
        }
    }
}

/// Actual accepted snapshot position plus optional last runtime observation.
/// The private source binding excludes caller provenance, just like RCP3 rows.
#[derive(Debug, Clone)]
pub struct RuntimeTermination {
    pub cause: RuntimeTerminationCause,
    pub detail: String,
    pub accepted_generation: u64,
    pub accepted_time: f64,
    pub observation: Option<RuntimeObservation>,
    source: [u8; 32],
}
impl PartialEq for RuntimeTermination {
    fn eq(&self, other: &Self) -> bool {
        self.cause == other.cause
            && self.detail == other.detail
            && self.accepted_generation == other.accepted_generation
            && self.accepted_time.to_bits() == other.accepted_time.to_bits()
            && self.observation == other.observation
            && self.source == other.source
    }
}
impl Eq for RuntimeTermination {}
impl RuntimeTermination {
    pub fn capture(
        snapshot: &SessionSnapshot<'_>,
        cause: RuntimeTerminationCause,
        detail: &str,
        observation: Option<&RuntimeObservation>,
    ) -> Result<Self, RecordingError> {
        bounded(detail)?;
        if let Some(observation) = observation {
            observation.validate()?;
        }
        let run = configured_run(snapshot, "runtime termination source binding")?;
        let state = snapshot.state();
        let termination = Self {
            cause,
            detail: detail.into(),
            accepted_generation: state.carrier.generation,
            accepted_time: state.carrier.time,
            observation: observation.cloned(),
            source: source_binding(&run),
        };
        termination.validate(&run)?;
        if let Some(observation) = &termination.observation {
            if state.pressure_surface.is_some()
                || state.pressure_columns.is_some()
                || state.reconstructed_surface.is_some()
                || state.flat_column_mac
            {
                return Err(RecordingError::Unsupported("runtime snapshot geometry"));
            }
            if observation.request.context.generation == termination.accepted_generation {
                let receipt = owned_receipt(
                    snapshot
                        .configured_provider_report()
                        .ok_or(RecordingError::Invalid("runtime accepted receipt"))?,
                    snapshot.report().is_some_and(|r| r.viscosity.is_some()),
                )?;
                validate_publication(observation, &run, &receipt)?;
            } else {
                let previous_mu = snapshot
                    .report()
                    .map(|r| r.viscosity.map_or(0.0, |v| v.dynamic_viscosity));
                validate_prior(
                    &observation.request,
                    termination.accepted_generation,
                    termination.accepted_time,
                    previous_mu.filter(|_| snapshot.provider_report().is_some()),
                    (termination.accepted_generation > 0).then(|| hash_values(state.pressure)),
                    None,
                )?;
            }
        }
        // Exercise the aggregate metadata budget before returning an owned value.
        termination.encode()?;
        Ok(termination)
    }

    pub fn finish(&self, recorder: &mut StepRecorder) -> Result<(), RecordingError> {
        self.validate(recorder.run())?;
        let detail = self.encode()?;
        recorder.finish(
            self.cause.outcome(),
            &detail,
            Some(self.accepted_generation),
        )
    }

    pub fn decode(run: &RecordingRun, status: &RecordingStatus) -> Result<Self, RecordingError> {
        let workflow = crate::recording::workflow::split_terminal_status(status)?;
        let status = workflow.as_ref().map_or(status, |(status, _, _)| status);
        if status.outcome == "active" {
            return Err(RecordingError::Unsupported("runtime active termination"));
        }
        configured_configuration(run)?;
        let mut decoder = Decoder::text(&status.detail, TERMINATION)?;
        let source = decoder.array()?;
        if source != source_binding(run) {
            return Err(RecordingError::Invalid("runtime termination source"));
        }
        let cause = match decoder.u8()? {
            0 => RuntimeTerminationCause::Completed,
            1 => RuntimeTerminationCause::Cancelled,
            2 => RuntimeTerminationCause::SessionDeadline,
            3 => RuntimeTerminationCause::RuntimeFailure,
            4 => RuntimeTerminationCause::StartupFailure,
            5 => RuntimeTerminationCause::NumericalFailure,
            6 => RuntimeTerminationCause::StorageFailure,
            _ => return Err(RecordingError::Unsupported("runtime termination cause")),
        };
        let accepted_generation = decoder.u64()?;
        let accepted_time = decoder.f64()?;
        let detail = decoder.string()?;
        let observation = if decoder.boolean()? {
            Some(get_observation(&mut decoder)?)
        } else {
            None
        };
        decoder.end()?;
        let termination = Self {
            cause,
            detail,
            accepted_generation,
            accepted_time,
            observation,
            source,
        };
        termination.validate(run)?;
        if let Some((_, attempt, source)) = &workflow {
            crate::recording::workflow::validate_terminal_binding(
                run,
                &termination,
                attempt,
                *source,
            )?;
        }
        if status.outcome != cause.outcome()
            || status.accepted_generation != Some(accepted_generation)
        {
            return Err(RecordingError::Invalid(
                "runtime terminal lifecycle association",
            ));
        }
        Ok(termination)
    }

    fn validate(&self, run: &RecordingRun) -> Result<(), RecordingError> {
        bounded(&self.detail)?;
        let cfg = configured_configuration(run)?;
        if self.source != source_binding(run)
            || self.accepted_generation > cfg.max_steps as u64
            || !self.accepted_time.is_finite()
            || (self.accepted_generation == 0 && self.accepted_time.to_bits() != 0.0_f64.to_bits())
            || (self.accepted_generation > 0 && self.accepted_time <= 0.0)
            || (self.cause == RuntimeTerminationCause::StartupFailure && self.observation.is_some())
        {
            return Err(RecordingError::Invalid(
                "runtime terminal snapshot association",
            ));
        }
        if let Some(observation) = &self.observation {
            observation.validate()?;
            validate_request_source(&observation.request, run)?;
            let c = &observation.request.context;
            if c.generation == self.accepted_generation {
                if c.interval_end.to_bits() != self.accepted_time.to_bits()
                    || matches!(
                        observation.outcome,
                        RuntimeOutcome::NoCandidate {
                            reason: NoCandidateReason::Cancelled,
                            ..
                        }
                    )
                {
                    return Err(RecordingError::Invalid(
                        "runtime terminal accepted observation",
                    ));
                }
            } else if self.accepted_generation.checked_add(1) != Some(c.generation)
                || c.interval_start.to_bits() != self.accepted_time.to_bits()
                || self.cause == RuntimeTerminationCause::Completed
            {
                return Err(RecordingError::Invalid(
                    "runtime terminal attempted observation",
                ));
            }
        }
        Ok(())
    }

    pub(in crate::recording) fn encode(&self) -> Result<String, RecordingError> {
        let mut encoder = Encoder::default();
        encoder.0.extend_from_slice(&self.source);
        encoder.u8(match self.cause {
            RuntimeTerminationCause::Completed => 0,
            RuntimeTerminationCause::Cancelled => 1,
            RuntimeTerminationCause::SessionDeadline => 2,
            RuntimeTerminationCause::RuntimeFailure => 3,
            RuntimeTerminationCause::StartupFailure => 4,
            RuntimeTerminationCause::NumericalFailure => 5,
            RuntimeTerminationCause::StorageFailure => 6,
        });
        encoder.u64(self.accepted_generation);
        encoder.f64(self.accepted_time);
        encoder.string(&self.detail)?;
        encoder.boolean(self.observation.is_some());
        if let Some(observation) = &self.observation {
            put_observation(&mut encoder, observation)?;
        }
        encoder.text(TERMINATION)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeTerminalCoverage {
    VerifiedLastPublication,
    UnrecordedAcceptedTail {
        last_recorded_generation: Option<u64>,
    },
    NoRecordedPublication,
}
#[derive(Debug, Clone)]
pub struct RuntimeTerminationAudit {
    pub termination: RuntimeTermination,
    pub coverage: RuntimeTerminalCoverage,
}

/// Verify available archive associations before reporting terminal custody.
/// This does not reconstruct an RHS, authenticate execution, or verify an
/// unrecorded accepted tail. Only scalar lineage/digests survive each iteration.
pub fn audit_runtime_termination(
    reader: &RecordingReader,
) -> Result<RuntimeTerminationAudit, RecordingError> {
    verify_configured_recording(reader)?;
    let termination = RuntimeTermination::decode(reader.run(), &reader.status()?)?;
    let count = reader.count()?;
    if count == 0 {
        let coverage = if termination.accepted_generation == 0 {
            RuntimeTerminalCoverage::NoRecordedPublication
        } else {
            RuntimeTerminalCoverage::UnrecordedAcceptedTail {
                last_recorded_generation: None,
            }
        };
        return Ok(RuntimeTerminationAudit {
            termination,
            coverage,
        });
    }
    let (generation, time, receipt, runtime, pressure_hash) = {
        let row = reader.select(count - 1)?;
        // Whole-field qualification happened above. Reuse the parent's shared
        // controls decoder; no fixed RCP offsets or second field extraction.
        let (receipt, runtime) = recorded_metadata(reader.run(), &row)?;
        let hash = row.field("interval_pressure").map(|field| {
            let hash: [u8; 32] = Sha256::digest(&field.data).into();
            hash
        });
        (row.generation, row.time, receipt, runtime, hash)
    };
    if termination.accepted_generation < generation {
        return Err(RecordingError::Invalid("runtime terminal recorded tail"));
    }
    if termination.accepted_generation > generation {
        if termination.accepted_time <= time {
            return Err(RecordingError::Invalid("runtime terminal unrecorded time"));
        }
        if let Some(observation) = &termination.observation {
            let request = &observation.request;
            if generation.checked_add(1) == Some(request.context.generation) {
                let history = retained_history(reader)?;
                validate_prior(
                    request,
                    generation,
                    time,
                    history.map(|history| history.0),
                    pressure_hash,
                    history.map(|history| history.1),
                )?;
            } else if request
                .context
                .prior_generation
                .is_some_and(|prior| prior > generation)
                && request.context.interval_start <= time
            {
                return Err(RecordingError::Invalid(
                    "runtime terminal unrecorded prior time",
                ));
            }
        }
        return Ok(RuntimeTerminationAudit {
            termination,
            coverage: RuntimeTerminalCoverage::UnrecordedAcceptedTail {
                last_recorded_generation: Some(generation),
            },
        });
    }
    if termination.accepted_time.to_bits() != time.to_bits() {
        return Err(RecordingError::Invalid("runtime terminal accepted time"));
    }
    if let Some(observation) = &termination.observation {
        if observation.request.context.generation == generation {
            validate_publication(
                observation,
                reader.run(),
                receipt
                    .as_ref()
                    .ok_or(RecordingError::Invalid("runtime terminal accepted receipt"))?,
            )?;
            if runtime
                .as_ref()
                .is_some_and(|runtime| runtime != observation)
            {
                return Err(RecordingError::Invalid(
                    "runtime terminal publication observation",
                ));
            }
        } else {
            let history = retained_history(reader)?;
            validate_prior(
                &observation.request,
                generation,
                time,
                history.map(|history| history.0),
                pressure_hash,
                history.map(|history| history.1),
            )?;
        }
    }
    Ok(RuntimeTerminationAudit {
        termination,
        coverage: RuntimeTerminalCoverage::VerifiedLastPublication,
    })
}

// Same accepted native-prefix transition as the Session/core verifier. The core
// audit above supplies contiguous row and context qualification; this fold adds
// no numerical arrays and is used only for a terminal next-attempt association.
fn retained_history(
    reader: &RecordingReader,
) -> Result<Option<(f64, usize, bool)>, RecordingError> {
    let mut history: Option<(f64, usize, bool)> = None;
    for ordinal in 0..reader.count()? {
        let row = reader.select(ordinal)?;
        let (receipt, _) = recorded_metadata(reader.run(), &row)?;
        if let Some(receipt) = receipt {
            let prior = history
                .filter(|history| history.0.to_bits() == receipt.dynamic_viscosity.to_bits());
            let native = matches!(receipt.selected, RecordedPressureSelection::Native(_));
            let all_native = native && prior.is_none_or(|history| history.2);
            let prefix = if all_native {
                prior.map_or(0, |history| history.1).saturating_add(1)
            } else {
                prior.map_or(0, |history| history.1)
            };
            history = Some((receipt.dynamic_viscosity, prefix, all_native));
        }
    }
    Ok(history)
}

fn hash_values(values: &[f64]) -> [u8; 32] {
    let mut hash = Sha256::new();
    for value in values {
        hash.update(value.to_bits().to_le_bytes());
    }
    hash.finalize().into()
}
fn same_grid(a: &GridGeometry, b: &GridGeometry) -> bool {
    a.counts() == b.counts()
        && a.spacing().map(f64::to_bits) == b.spacing().map(f64::to_bits)
        && a.origin().map(f64::to_bits) == b.origin().map(f64::to_bits)
}
fn same_context(a: &AcceptedPressureContextStamp, b: &AcceptedPressureContextStamp) -> bool {
    a.generation == b.generation
        && a.interval_start.to_bits() == b.interval_start.to_bits()
        && a.interval_end.to_bits() == b.interval_end.to_bits()
        && a.dt.to_bits() == b.dt.to_bits()
        && a.native_prefix_steps == b.native_prefix_steps
        && a.prior_generation == b.prior_generation
        && a.prior_interval_end.map(f64::to_bits) == b.prior_interval_end.map(f64::to_bits)
        && a.prior_pressure_sha256 == b.prior_pressure_sha256
        && a.rhs_sha256 == b.rhs_sha256
}
fn validate_request_source(
    request: &RuntimeRequestStamp,
    run: &RecordingRun,
) -> Result<(), RecordingError> {
    request.validate()?;
    let cfg = configured_configuration(run)?;
    let grid = GridGeometry::new(run.counts, run.spacing, run.origin)
        .map_err(|_| RecordingError::Invalid("runtime source geometry"))?;
    if request.run_id != run.identity.run_id
        || !same_grid(&request.grid, &grid)
        || request.density.to_bits() != cfg.transport.carrier.density.to_bits()
        || (request.dynamic_viscosity > 0.0
            && cfg.transport.represented_density.to_bits()
                != cfg.transport.carrier.density.to_bits())
        || request.context.generation > cfg.max_steps as u64
    {
        return Err(RecordingError::Invalid("runtime source association"));
    }
    Ok(())
}
fn validate_prior(
    request: &RuntimeRequestStamp,
    generation: u64,
    time: f64,
    previous_mu: Option<f64>,
    pressure_hash: Option<[u8; 32]>,
    prefix: Option<usize>,
) -> Result<(), RecordingError> {
    let c = &request.context;
    if generation.checked_add(1) != Some(c.generation)
        || c.interval_start.to_bits() != time.to_bits()
    {
        return Err(RecordingError::Invalid(
            "runtime attempted prior association",
        ));
    }
    if generation == 0 {
        if c.prior_generation.is_some() || c.native_prefix_steps != 0 {
            return Err(RecordingError::Invalid("runtime constructor history"));
        }
    } else if c.prior_generation != Some(generation)
        || c.prior_interval_end.map(f64::to_bits) != Some(time.to_bits())
        || c.prior_pressure_sha256 != pressure_hash
        || previous_mu.map(f64::to_bits) != Some(request.dynamic_viscosity.to_bits())
        || prefix.is_some_and(|prefix| prefix != c.native_prefix_steps)
    {
        return Err(RecordingError::Invalid(
            "runtime attempted prior association",
        ));
    }
    Ok(())
}

pub(super) fn put_observation(
    encoder: &mut Encoder,
    observation: &RuntimeObservation,
) -> Result<(), RecordingError> {
    observation.validate()?;
    let request = &observation.request;
    encoder.u64(request.run_id);
    for count in request.grid.counts() {
        encoder.usize(count);
    }
    for value in request
        .grid
        .spacing()
        .into_iter()
        .chain(request.grid.origin())
    {
        encoder.f64(value);
    }
    encoder.f64(request.density);
    encoder.f64(request.dynamic_viscosity);
    put_context(encoder, &request.context);
    put_binding(encoder, &request.binding)?;
    match &observation.outcome {
        RuntimeOutcome::CandidateReturned => encoder.u8(0),
        RuntimeOutcome::NoCandidate {
            reason,
            detail,
            dispatched,
            invalid_payload_sha256,
        } => {
            encoder.u8(1);
            encoder.u8(match reason {
                NoCandidateReason::Unsupported => 0,
                NoCandidateReason::UnsupportedContext => 1,
                NoCandidateReason::Cancelled => 2,
                NoCandidateReason::RuntimeFailure => 3,
                NoCandidateReason::InvalidOutput => 4,
            });
            encoder.string(detail)?;
            encoder.boolean(*dispatched);
            encoder.boolean(invalid_payload_sha256.is_some());
            if let Some(digest) = invalid_payload_sha256 {
                encoder.0.extend_from_slice(digest);
            }
        }
    }
    Ok(())
}
pub(super) fn get_observation(decoder: &mut Decoder) -> Result<RuntimeObservation, RecordingError> {
    let run_id = decoder.u64()?;
    let counts = [decoder.u64()?, decoder.u64()?, decoder.u64()?];
    let spacing = [decoder.f64()?, decoder.f64()?, decoder.f64()?];
    let origin = [decoder.f64()?, decoder.f64()?, decoder.f64()?];
    let grid = GridGeometry::new(counts, spacing, origin)
        .map_err(|_| RecordingError::Invalid("runtime request geometry"))?;
    let request = RuntimeRequestStamp {
        run_id,
        grid,
        density: decoder.f64()?,
        dynamic_viscosity: decoder.f64()?,
        context: get_context(decoder)?,
        binding: get_binding(decoder)?,
    };
    let outcome = match decoder.u8()? {
        0 => RuntimeOutcome::CandidateReturned,
        1 => {
            let reason = match decoder.u8()? {
                0 => NoCandidateReason::Unsupported,
                1 => NoCandidateReason::UnsupportedContext,
                2 => NoCandidateReason::Cancelled,
                3 => NoCandidateReason::RuntimeFailure,
                4 => NoCandidateReason::InvalidOutput,
                _ => return Err(RecordingError::Unsupported("runtime no-candidate reason")),
            };
            RuntimeOutcome::NoCandidate {
                reason,
                detail: decoder.string()?,
                dispatched: decoder.boolean()?,
                invalid_payload_sha256: if decoder.boolean()? {
                    Some(decoder.array()?)
                } else {
                    None
                },
            }
        }
        _ => return Err(RecordingError::Unsupported("runtime outcome")),
    };
    let observation = RuntimeObservation { request, outcome };
    observation.validate()?;
    Ok(observation)
}
fn put_context(encoder: &mut Encoder, context: &AcceptedPressureContextStamp) {
    encoder.u64(context.generation);
    encoder.f64(context.interval_start);
    encoder.f64(context.interval_end);
    encoder.f64(context.dt);
    encoder.usize(context.native_prefix_steps);
    encoder.boolean(context.prior_generation.is_some());
    if let Some(generation) = context.prior_generation {
        encoder.u64(generation);
        // All callers validate the complete optional triple before encoding.
        encoder.f64(context.prior_interval_end.unwrap());
        encoder
            .0
            .extend_from_slice(&context.prior_pressure_sha256.unwrap());
    }
    encoder.0.extend_from_slice(&context.rhs_sha256);
}
fn get_context(decoder: &mut Decoder) -> Result<AcceptedPressureContextStamp, RecordingError> {
    let generation = decoder.u64()?;
    let interval_start = decoder.f64()?;
    let interval_end = decoder.f64()?;
    let dt = decoder.f64()?;
    let native_prefix_steps = decoder.usize()?;
    let (prior_generation, prior_interval_end, prior_pressure_sha256) = if decoder.boolean()? {
        (
            Some(decoder.u64()?),
            Some(decoder.f64()?),
            Some(decoder.array()?),
        )
    } else {
        (None, None, None)
    };
    Ok(AcceptedPressureContextStamp {
        generation,
        interval_start,
        interval_end,
        dt,
        native_prefix_steps,
        prior_generation,
        prior_interval_end,
        prior_pressure_sha256,
        rhs_sha256: decoder.array()?,
    })
}
pub(in crate::recording) fn put_binding(
    encoder: &mut Encoder,
    binding: &ContextProviderIdentity,
) -> Result<(), RecordingError> {
    encoder.string(&binding.id)?;
    encoder.string(&binding.version)?;
    encoder.0.extend_from_slice(&binding.manifest_sha256);
    encoder.boolean(binding.checkpoint_sha256.is_some());
    if let Some(digest) = binding.checkpoint_sha256 {
        encoder.0.extend_from_slice(&digest);
    }
    encoder.boolean(binding.training_run_id.is_some());
    if let Some(training) = &binding.training_run_id {
        encoder.string(training)?;
    }
    encoder.string(&binding.runtime)?;
    encoder.string(&binding.source_revision)
}
pub(in crate::recording) fn get_binding(
    decoder: &mut Decoder,
) -> Result<ContextProviderIdentity, RecordingError> {
    Ok(ContextProviderIdentity {
        id: decoder.string()?,
        version: decoder.string()?,
        manifest_sha256: decoder.array()?,
        checkpoint_sha256: if decoder.boolean()? {
            Some(decoder.array()?)
        } else {
            None
        },
        training_run_id: if decoder.boolean()? {
            Some(decoder.string()?)
        } else {
            None
        },
        runtime: decoder.string()?,
        source_revision: decoder.string()?,
    })
}
