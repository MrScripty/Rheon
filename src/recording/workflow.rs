//! Additive bounded workflow custody inside the existing SQLite v1 envelope.
//! RCP4 wraps an unchanged RCP2/RCP3 publication; RCT2 wraps unchanged RCT1
//! terminal facts. Failed attempts are terminal diagnostics, never accepted rows.
use super::provider::runtime::{
    self, RuntimeObservation, RuntimeTermination, RuntimeTerminationAudit, RuntimeTerminationCause,
};
use super::provider::{
    self, Decoder, Encoder, RecordedConfiguredReceipt, RecordedPressureSelection,
};
use super::{
    RecordedStep, RecordingError, RecordingReader, RecordingRun, RecordingStatus, StepRecorder,
};
use crate::workflow::{
    AttemptOutcome, MAX_NODES, NodeKind, NodeReceipt, NodeStatus, PressureFacts, WorkflowAttempt,
    WorkflowDescriptor,
};
use crate::{
    GridGeometry, PressureImplementation, PressureReport, PressureSettings, SessionSnapshot,
};
use std::sync::Arc;

pub const PUBLICATION_PROFILE: &str = "RCP4:";
pub const TERMINAL_PROFILE: &str = "RCT2:";

pub(super) type PublicationEnvelope = (String, Arc<WorkflowAttempt>, [u8; 32]);
pub(super) type TerminalEnvelope = (RecordingStatus, Option<Arc<WorkflowAttempt>>, [u8; 32]);

/// Publication evidence remains unavailable for historical profiles. Decode the
/// complete numerical publication before returning its associated graph facts.
pub fn publication(
    run: &RecordingRun,
    step: &RecordedStep,
) -> Result<Option<Arc<WorkflowAttempt>>, RecordingError> {
    if run.solver != provider::PROFILE {
        if (run.solver.starts_with("rheon-carrier-full-box-v1/")
            || run.solver.starts_with("rheon-sph-preview/"))
            && !step.controls.starts_with("RCP")
        {
            return Ok(None);
        }
        return Err(RecordingError::Unsupported(
            "workflow publication source profile",
        ));
    }
    provider::RecordedConfiguredSession::decode(run, step)?;
    Ok(split_publication_controls(&step.controls)?.map(|(_, attempt, _)| attempt))
}

/// Explicit selection-time validation against the immediately preceding retained
/// native state. Reads at most one additional row; cached paint does no I/O.
/// An archive without the prior publication cannot supply this extra proof.
pub fn selection(
    reader: &RecordingReader,
    step: &RecordedStep,
) -> Result<Option<Arc<WorkflowAttempt>>, RecordingError> {
    let evidence = publication(reader.run(), step)?;
    if let Some(attempt) = &evidence
        && step.ordinal > 0
    {
        let previous = reader.select(step.ordinal - 1)?;
        let native = provider::RecordedConfiguredSession::decode(reader.run(), &previous)?;
        if let Some((_, prior, _)) = split_publication_controls(&previous.controls)?
            && attempt.attempt_id <= prior.attempt_id
        {
            return Err(RecordingError::Invalid(
                "workflow selected attempt identity",
            ));
        }
        if native.generation != attempt.accepted_generation
            || native.time.to_bits() != attempt.interval_start.to_bits()
            || native_state_digest(&native)? != attempt.input_sha256
        {
            return Err(RecordingError::Invalid(
                "workflow selected prior native input",
            ));
        }
    }
    Ok(evidence)
}

pub fn configured_publication_with_workflow(
    snapshot: &SessionSnapshot<'_>,
    ordinal: usize,
    limit: usize,
    runtime: Option<&RuntimeObservation>,
) -> Result<RecordedStep, RecordingError> {
    let mut step =
        provider::configured_publication_with_runtime(snapshot, ordinal, limit, runtime)?;
    if let Some(attempt) = snapshot
        .configured_provider_report()
        .and_then(|report| report.workflow.as_ref())
    {
        let mut encoder = Encoder::default();
        encoder
            .0
            .extend_from_slice(&provider::source_binding(&provider::configured_run(
                snapshot,
                "workflow source",
            )?));
        encoder.boolean(step.controls.starts_with("RCP3:"));
        put_wrapped(
            &mut encoder,
            &step.controls,
            if step.controls.starts_with("RCP3:") {
                "RCP3:"
            } else {
                "RCP2:"
            },
        )?;
        put_attempt(&mut encoder, attempt)?;
        step.controls = encoder.text(PUBLICATION_PROFILE)?;
        if step.payload_bytes()? > limit {
            return Err(RecordingError::Limit("workflow publication"));
        }
        provider::RecordedConfiguredSession::decode(
            &provider::configured_run(snapshot, "workflow binding")?,
            &step,
        )?;
    }
    Ok(step)
}

pub(super) fn split_publication_controls(
    controls: &str,
) -> Result<Option<PublicationEnvelope>, RecordingError> {
    if !controls.starts_with(PUBLICATION_PROFILE) {
        return Ok(None);
    }
    let mut decoder = Decoder::text(controls, PUBLICATION_PROFILE)?;
    let source = decoder.array()?;
    let runtime = decoder.boolean()?;
    let original = get_wrapped(&mut decoder, if runtime { "RCP3:" } else { "RCP2:" })?;
    if !original.starts_with("RCP2:") && !original.starts_with("RCP3:") {
        return Err(RecordingError::Unsupported(
            "workflow wrapped publication profile",
        ));
    }
    let attempt = Arc::new(get_attempt(&mut decoder)?);
    decoder.end()?;
    Ok(Some((original, attempt, source)))
}

fn validate_source(run: &RecordingRun, attempt: &WorkflowAttempt) -> Result<(), RecordingError> {
    attempt.validate().map_err(RecordingError::Invalid)?;
    let config = provider::configured_configuration(run)?;
    let d = &attempt.descriptor;
    let p = config.transport.carrier.pressure;
    if attempt.run_id != run.identity.run_id
        || d.grid.counts().map(|n| n as u64) != run.counts
        || d.grid.spacing().map(f64::to_bits) != run.spacing.map(f64::to_bits)
        || d.grid.origin().map(f64::to_bits) != run.origin.map(f64::to_bits)
        || d.density.to_bits() != config.transport.carrier.density.to_bits()
        || d.implementation != config.implementation
        || d.pressure.max_iterations != p.max_iterations
        || [
            d.pressure.relative_residual,
            d.pressure.absolute_residual,
            d.pressure.divergence_limit,
            d.actual_divergence_limit,
            d.max_courant,
        ]
        .map(f64::to_bits)
            != [
                p.relative_residual,
                p.absolute_residual,
                p.divergence_limit,
                config.transport.carrier.actual_divergence_limit,
                config.transport.carrier.max_courant,
            ]
            .map(f64::to_bits)
        || attempt.attempted_generation > config.max_steps as u64
    {
        return Err(RecordingError::Invalid("workflow source association"));
    }
    Ok(())
}

/// Reuse the numerical owner's canonical native-input digest over the exact
/// decoded fields. Constructor pressure is explicitly zero in this profile.
pub(super) fn native_state_digest(
    recorded: &provider::RecordedConfiguredSession,
) -> Result<[u8; 32], RecordingError> {
    let mut initial_pressure = Vec::new();
    let pressure = if let Some((_, pressure)) = &recorded.interval_pressure {
        pressure.as_slice()
    } else {
        initial_pressure
            .try_reserve_exact(recorded.grid.cell_len())
            .map_err(|_| RecordingError::Limit("workflow constructor pressure"))?;
        initial_pressure.resize(recorded.grid.cell_len(), 0.0);
        initial_pressure.as_slice()
    };
    Ok(crate::workflow::input_digest(crate::LiquidTransportView {
        carrier: crate::StateView {
            x: &recorded.x,
            y: &recorded.y,
            z: &recorded.z,
            tracer: &recorded.tracer,
            time: recorded.time,
            generation: recorded.generation,
        },
        liquid: crate::LiquidVolumeView {
            fraction: &recorded.fraction,
            stamp: recorded.fraction_stamp,
            time: recorded.fraction_time,
            density: recorded.configuration.transport.represented_density,
        },
        pressure,
        carrier_stamp: recorded.carrier_stamp,
        pressure_surface: None,
        reconstructed_surface: None,
        pressure_columns: None,
        flat_column_mac: false,
    }))
}

pub(super) fn validate_publication(
    run: &RecordingRun,
    step: &RecordedStep,
    attempt: &WorkflowAttempt,
    receipt: Option<&RecordedConfiguredReceipt>,
) -> Result<(), RecordingError> {
    validate_source(run, attempt)?;
    let receipt = receipt.ok_or(RecordingError::Invalid("workflow without accepted receipt"))?;
    if attempt.outcome != AttemptOutcome::Accepted
        || attempt.publication != Some(step.generation)
        || attempt.actual_dt.map(f64::to_bits) != step.dt.map(f64::to_bits)
        || attempt
            .actual_dt
            .is_none_or(|dt| (attempt.interval_start + dt).to_bits() != step.time.to_bits())
        || attempt.descriptor.viscosity.is_some() != receipt.viscosity_applied
        || attempt.descriptor.viscosity.unwrap_or(0.0).to_bits()
            != receipt.dynamic_viscosity.to_bits()
    {
        return Err(RecordingError::Invalid("workflow publication association"));
    }
    let pressure = attempt
        .pressure
        .as_ref()
        .ok_or(RecordingError::Invalid("workflow accepted pressure absent"))?;
    let selected = match receipt.selected {
        RecordedPressureSelection::Native(method) => method.id(),
        RecordedPressureSelection::Configured => "configured-pressure-context-v1",
    };
    if pressure.selected != selected
        || pressure.binding != receipt.binding
        || pressure.prediction_called != receipt.prediction_called
        || pressure.accepted.map(report_bits) != Some(report_bits(receipt.accepted))
        || pressure.candidate.map(report_bits) != receipt.candidate.map(report_bits)
        || receipt
            .context
            .as_ref()
            .is_some_and(|c| pressure.rhs_sha256 != Some(c.rhs_sha256))
        || pressure.rejection
            != receipt
                .rejection
                .as_ref()
                .map(|rejection| format!("{rejection:?}"))
        || pressure.pressure_call_ns != u64::try_from(receipt.total.as_nanos()).ok()
    {
        return Err(RecordingError::Invalid(
            "workflow pressure receipt association",
        ));
    }
    Ok(())
}

fn validate_runtime_outcome(
    attempt: &WorkflowAttempt,
    observation: &RuntimeObservation,
) -> Result<(), RecordingError> {
    use super::diagnostic::NoCandidateReason;
    let pressure = attempt
        .pressure
        .as_ref()
        .ok_or(RecordingError::Invalid("workflow runtime pressure absent"))?;
    let returned = attempt.nodes.iter().any(|node| {
        node.node == NodeKind::PressureCandidate && node.status == NodeStatus::CandidateReturned
    });
    let rejected = attempt.nodes.iter().any(|node| {
        node.node == NodeKind::PressureCandidate && node.status == NodeStatus::Rejected
    });
    let valid = match &observation.outcome {
        runtime::RuntimeOutcome::CandidateReturned => {
            returned
                && !rejected
                && pressure.rejection.as_deref().is_none_or(|reason| {
                    matches!(reason, "QualityGate" | "NonFiniteOutput" | "NonZeroGauge")
                })
        }
        runtime::RuntimeOutcome::NoCandidate { reason, .. } => {
            !returned
                && rejected
                && pressure.candidate.is_none()
                && match reason {
                    NoCandidateReason::Cancelled => {
                        pressure.rejection.as_deref() == Some("Cancelled")
                    }
                    NoCandidateReason::Unsupported | NoCandidateReason::UnsupportedContext => {
                        pressure
                            .rejection
                            .as_deref()
                            .is_some_and(|reason| reason.starts_with("Unsupported("))
                    }
                    NoCandidateReason::RuntimeFailure | NoCandidateReason::InvalidOutput => {
                        pressure
                            .rejection
                            .as_deref()
                            .is_some_and(|reason| reason.starts_with("Execution("))
                    }
                }
        }
    };
    if !valid {
        return Err(RecordingError::Invalid(
            "workflow runtime outcome association",
        ));
    }
    Ok(())
}

/// Terminal workflow facts have a separate publication association and may
/// describe an attempt after generation zero with no recorded publication.
#[derive(Debug, Clone)]
pub struct WorkflowTerminationAudit {
    pub runtime: RuntimeTerminationAudit,
    pub attempt: Option<Arc<WorkflowAttempt>>,
}

pub struct WorkflowTermination {
    runtime: RuntimeTermination,
    attempt: Option<Arc<WorkflowAttempt>>,
    source: [u8; 32],
}
impl WorkflowTermination {
    pub fn capture(
        snapshot: &SessionSnapshot<'_>,
        cause: RuntimeTerminationCause,
        detail: &str,
        runtime: Option<&RuntimeObservation>,
        attempt: Option<&Arc<WorkflowAttempt>>,
    ) -> Result<Self, RecordingError> {
        let runtime = RuntimeTermination::capture(snapshot, cause, detail, runtime)?;
        let value = Self {
            runtime,
            attempt: attempt.cloned(),
            source: provider::source_binding(&provider::configured_run(
                snapshot,
                "workflow terminal source",
            )?),
        };
        value.validate(&provider::configured_run(
            snapshot,
            "workflow terminal source",
        )?)?;
        value.encode()?;
        Ok(value)
    }
    fn validate(&self, run: &RecordingRun) -> Result<(), RecordingError> {
        if self.source != provider::source_binding(run) {
            return Err(RecordingError::Invalid("workflow terminal source"));
        }
        if let Some(attempt) = &self.attempt {
            validate_source(run, attempt)?;
            if let Some(observation) = &self.runtime.observation {
                let pressure = attempt.pressure.as_ref().ok_or(RecordingError::Invalid(
                    "workflow terminal runtime facts absent",
                ))?;
                if observation.request.context.generation != attempt.attempted_generation
                    || observation.request.context.interval_start.to_bits()
                        != attempt.interval_start.to_bits()
                    || Some(observation.request.context.dt.to_bits())
                        != attempt.actual_dt.map(f64::to_bits)
                    || pressure.binding.as_ref() != Some(&observation.request.binding)
                    || pressure.rhs_sha256 != Some(observation.request.context.rhs_sha256)
                    || !pressure.prediction_called
                {
                    return Err(RecordingError::Invalid(
                        "workflow terminal runtime association",
                    ));
                }
                validate_runtime_outcome(attempt, observation)?;
            }
            if let Some(generation) = attempt.publication {
                if generation != self.runtime.accepted_generation
                    || attempt.actual_dt.is_none_or(|dt| {
                        (attempt.interval_start + dt).to_bits()
                            != self.runtime.accepted_time.to_bits()
                    })
                {
                    return Err(RecordingError::Invalid(
                        "workflow terminal accepted association",
                    ));
                }
            } else if attempt.accepted_generation != self.runtime.accepted_generation
                || attempt.interval_start.to_bits() != self.runtime.accepted_time.to_bits()
                || self.runtime.cause == RuntimeTerminationCause::Completed
            {
                return Err(RecordingError::Invalid(
                    "workflow terminal attempted association",
                ));
            }
        }
        Ok(())
    }
    fn encode(&self) -> Result<String, RecordingError> {
        let mut encoder = Encoder::default();
        encoder.0.extend_from_slice(&self.source);
        put_wrapped(&mut encoder, &self.runtime.encode()?, "RCT1:")?;
        encoder.boolean(self.attempt.is_some());
        if let Some(attempt) = &self.attempt {
            put_attempt(&mut encoder, attempt)?;
        }
        encoder.text(TERMINAL_PROFILE)
    }
    pub fn finish(&self, recorder: &mut StepRecorder) -> Result<(), RecordingError> {
        self.validate(recorder.run())?;
        let status = match self.runtime.cause {
            RuntimeTerminationCause::Completed => "completed",
            RuntimeTerminationCause::Cancelled => "cancelled",
            _ => "failed",
        };
        recorder.finish(
            status,
            &self.encode()?,
            Some(self.runtime.accepted_generation),
        )
    }
}

pub(super) fn split_terminal_status(
    status: &RecordingStatus,
) -> Result<Option<TerminalEnvelope>, RecordingError> {
    if !status.detail.starts_with(TERMINAL_PROFILE) {
        return Ok(None);
    }
    let mut decoder = Decoder::text(&status.detail, TERMINAL_PROFILE)?;
    let source = decoder.array()?;
    let detail = get_wrapped(&mut decoder, "RCT1:")?;
    if !detail.starts_with("RCT1:") {
        return Err(RecordingError::Unsupported(
            "workflow wrapped terminal profile",
        ));
    }
    let attempt = if decoder.boolean()? {
        Some(Arc::new(get_attempt(&mut decoder)?))
    } else {
        None
    };
    decoder.end()?;
    Ok(Some((
        RecordingStatus {
            detail,
            ..status.clone()
        },
        attempt,
        source,
    )))
}

pub(super) fn validate_terminal_binding(
    run: &RecordingRun,
    runtime: &RuntimeTermination,
    attempt: &Option<Arc<WorkflowAttempt>>,
    source: [u8; 32],
) -> Result<(), RecordingError> {
    WorkflowTermination {
        runtime: runtime.clone(),
        attempt: attempt.clone(),
        source,
    }
    .validate(run)
}

pub fn terminal(
    reader: &RecordingReader,
) -> Result<Option<WorkflowTerminationAudit>, RecordingError> {
    let status = reader.status()?;
    let Some((_, attempt, source)) = split_terminal_status(&status)? else {
        if status.detail.starts_with("RCT1:") {
            runtime::audit_runtime_termination(reader)?;
        } else if status.detail.starts_with("RCT") {
            return Err(RecordingError::Unsupported("workflow terminal version"));
        }
        return Ok(None);
    };
    let runtime = runtime::audit_runtime_termination(reader)?;
    WorkflowTermination {
        runtime: runtime.termination.clone(),
        attempt: attempt.clone(),
        source,
    }
    .validate(reader.run())?;
    if let Some(attempt) = &attempt {
        let count = reader.count()?;
        if count > 0 {
            let row = reader.select(count - 1)?;
            if attempt.publication != Some(row.generation)
                && let Some((_, prior, _)) = split_publication_controls(&row.controls)?
                && attempt.attempt_id <= prior.attempt_id
            {
                return Err(RecordingError::Invalid(
                    "workflow terminal attempt identity",
                ));
            }
            if row.generation == attempt.accepted_generation {
                let prior = provider::RecordedConfiguredSession::decode(reader.run(), &row)?;
                if prior.time.to_bits() != attempt.interval_start.to_bits()
                    || native_state_digest(&prior)? != attempt.input_sha256
                {
                    return Err(RecordingError::Invalid("workflow terminal native input"));
                }
            }
            if attempt.publication == Some(row.generation) {
                if let Some(recorded) = publication(reader.run(), &row)? {
                    let mut a = Encoder::default();
                    let mut b = Encoder::default();
                    put_attempt(&mut a, attempt)?;
                    put_attempt(&mut b, &recorded)?;
                    if a.0 != b.0 {
                        return Err(RecordingError::Invalid(
                            "workflow terminal publication differs",
                        ));
                    }
                } else {
                    return Err(RecordingError::Invalid(
                        "workflow terminal accepted evidence missing",
                    ));
                }
            }
        }
    }
    Ok(Some(WorkflowTerminationAudit { runtime, attempt }))
}

// Store the existing hex profile's decoded payload once. This preserves its
// exact bytes while avoiding nested hex expansion and the 1000-byte text cap.
fn put_wrapped(e: &mut Encoder, text: &str, profile: &str) -> Result<(), RecordingError> {
    let original = Decoder::text(text, profile)?;
    e.u32(original.data.len() as u32);
    e.0.extend_from_slice(&original.data);
    Ok(())
}
fn get_wrapped(d: &mut Decoder, profile: &str) -> Result<String, RecordingError> {
    let len = d.u32()? as usize;
    if len > super::MAX_TEXT / 2 {
        return Err(RecordingError::Limit("workflow wrapped metadata"));
    }
    let mut bytes = Vec::with_capacity(len);
    for _ in 0..len {
        bytes.push(d.u8()?);
    }
    Encoder(bytes).text(profile)
}

fn report_bits(report: PressureReport) -> [u64; 4] {
    [
        report.iterations as u64,
        report.true_residual_l2.to_bits(),
        report.true_residual_max.to_bits(),
        report.predicted_divergence_max.to_bits(),
    ]
}
fn put_report(e: &mut Encoder, report: PressureReport) {
    for n in report_bits(report) {
        e.u64(n);
    }
}
fn get_report(d: &mut Decoder) -> Result<PressureReport, RecordingError> {
    Ok(PressureReport {
        iterations: d.usize()?,
        true_residual_l2: d.f64()?,
        true_residual_max: d.f64()?,
        predicted_divergence_max: d.f64()?,
    })
}
fn put_f64(e: &mut Encoder, value: Option<f64>) {
    e.boolean(value.is_some());
    if let Some(v) = value {
        e.f64(v);
    }
}
fn get_f64(d: &mut Decoder) -> Result<Option<f64>, RecordingError> {
    Ok(if d.boolean()? { Some(d.f64()?) } else { None })
}
fn put_attempt(e: &mut Encoder, a: &WorkflowAttempt) -> Result<(), RecordingError> {
    a.validate().map_err(RecordingError::Invalid)?;
    let w = &a.descriptor;
    e.u32(w.version);
    for n in w.grid.counts() {
        e.usize(n);
    }
    for v in w.grid.spacing().into_iter().chain(w.grid.origin()) {
        e.f64(v);
    }
    e.f64(w.density);
    put_f64(e, w.viscosity);
    e.u8(match w.implementation {
        PressureImplementation::JacobiPcgV1 => 0,
        PressureImplementation::SymmetricGaussSeidelPcgV1 => 1,
    });
    e.f64(w.pressure.relative_residual);
    e.f64(w.pressure.absolute_residual);
    e.f64(w.pressure.divergence_limit);
    e.usize(w.pressure.max_iterations);
    e.f64(w.actual_divergence_limit);
    e.f64(w.max_courant);
    e.0.extend_from_slice(&a.descriptor_sha256);
    e.0.extend_from_slice(&a.input_sha256);
    e.0.extend_from_slice(&a.request_sha256);
    e.usize(a.force_count);
    e.boolean(a.smoke_source_present);
    e.boolean(a.volume_source_present);
    for v in [
        a.run_id,
        a.attempt_id,
        a.accepted_generation,
        a.attempted_generation,
    ] {
        e.u64(v);
    }
    e.f64(a.interval_start);
    e.f64(a.requested_dt);
    put_f64(e, a.actual_dt);
    e.boolean(a.publication.is_some());
    if let Some(v) = a.publication {
        e.u64(v);
    }
    e.u8(match a.outcome {
        AttemptOutcome::Accepted => 0,
        AttemptOutcome::Cancelled => 1,
        AttemptOutcome::Failed => 2,
        AttemptOutcome::Refused => 3,
    });
    e.usize(a.nodes.len());
    for n in &a.nodes {
        e.u8(n.node as u8);
        e.u8(match n.status {
            NodeStatus::Started => 5,
            NodeStatus::Completed => 0,
            NodeStatus::CandidateReturned => 1,
            NodeStatus::Rejected => 2,
            NodeStatus::Selected => 3,
            NodeStatus::Accepted => 4,
        });
    }
    e.boolean(a.pressure.is_some());
    if let Some(p) = &a.pressure {
        e.string(&p.selected)?;
        e.boolean(p.binding.is_some());
        if let Some(b) = &p.binding {
            runtime::put_binding(e, b)?;
        }
        e.boolean(p.prediction_called);
        e.u8(match p.support {
            None => 0,
            Some(false) => 1,
            Some(true) => 2,
        });
        e.boolean(p.rejection.is_some());
        if let Some(r) = &p.rejection {
            e.string(r)?;
        }
        e.boolean(p.rhs_sha256.is_some());
        if let Some(h) = p.rhs_sha256 {
            e.0.extend_from_slice(&h);
        }
        for r in [p.candidate, p.accepted] {
            e.boolean(r.is_some());
            if let Some(r) = r {
                put_report(e, r);
            }
        }
        e.boolean(p.pressure_call_ns.is_some());
        if let Some(v) = p.pressure_call_ns {
            e.u64(v);
        }
        put_f64(e, p.residual_l2_limit);
    }
    put_f64(e, a.actual_divergence_max);
    e.string(&a.diagnostic)
}
fn get_attempt(d: &mut Decoder) -> Result<WorkflowAttempt, RecordingError> {
    let version = d.u32()?;
    if version != crate::workflow::DESCRIPTOR_VERSION {
        return Err(RecordingError::Unsupported("workflow descriptor version"));
    }
    let counts = [d.u64()?, d.u64()?, d.u64()?];
    let spacing = [d.f64()?, d.f64()?, d.f64()?];
    let origin = [d.f64()?, d.f64()?, d.f64()?];
    let grid = GridGeometry::new(counts, spacing, origin)
        .map_err(|_| RecordingError::Invalid("workflow geometry"))?;
    let density = d.f64()?;
    let viscosity = get_f64(d)?;
    let implementation = match d.u8()? {
        0 => PressureImplementation::JacobiPcgV1,
        1 => PressureImplementation::SymmetricGaussSeidelPcgV1,
        _ => return Err(RecordingError::Unsupported("workflow pressure method")),
    };
    let pressure = PressureSettings {
        relative_residual: d.f64()?,
        absolute_residual: d.f64()?,
        divergence_limit: d.f64()?,
        max_iterations: d.usize()?,
    };
    let descriptor = WorkflowDescriptor {
        version,
        grid,
        density,
        viscosity,
        implementation,
        pressure,
        actual_divergence_limit: d.f64()?,
        max_courant: d.f64()?,
    };
    let descriptor_sha256 = d.array()?;
    let input_sha256 = d.array()?;
    let request_sha256 = d.array()?;
    let force_count = d.usize()?;
    let smoke_source_present = d.boolean()?;
    let volume_source_present = d.boolean()?;
    let run_id = d.u64()?;
    let attempt_id = d.u64()?;
    let accepted_generation = d.u64()?;
    let attempted_generation = d.u64()?;
    let interval_start = d.f64()?;
    let requested_dt = d.f64()?;
    let actual_dt = get_f64(d)?;
    let publication = if d.boolean()? { Some(d.u64()?) } else { None };
    let outcome = match d.u8()? {
        0 => AttemptOutcome::Accepted,
        1 => AttemptOutcome::Cancelled,
        2 => AttemptOutcome::Failed,
        3 => AttemptOutcome::Refused,
        _ => return Err(RecordingError::Unsupported("workflow outcome")),
    };
    let count = d.usize()?;
    if count > MAX_NODES {
        return Err(RecordingError::Limit("workflow node count"));
    }
    let mut nodes = Vec::with_capacity(count);
    for _ in 0..count {
        let node = *NodeKind::ALL
            .get(d.u8()? as usize)
            .ok_or(RecordingError::Unsupported("workflow node kind"))?;
        let status = match d.u8()? {
            5 => NodeStatus::Started,
            0 => NodeStatus::Completed,
            1 => NodeStatus::CandidateReturned,
            2 => NodeStatus::Rejected,
            3 => NodeStatus::Selected,
            4 => NodeStatus::Accepted,
            _ => return Err(RecordingError::Unsupported("workflow node status")),
        };
        nodes.push(NodeReceipt { node, status });
    }
    let pressure = if d.boolean()? {
        let selected = d.string()?;
        let binding = if d.boolean()? {
            Some(runtime::get_binding(d)?)
        } else {
            None
        };
        let prediction_called = d.boolean()?;
        let support = match d.u8()? {
            0 => None,
            1 => Some(false),
            2 => Some(true),
            _ => return Err(RecordingError::Unsupported("workflow support")),
        };
        let rejection = if d.boolean()? {
            Some(d.string()?)
        } else {
            None
        };
        let rhs_sha256 = if d.boolean()? { Some(d.array()?) } else { None };
        let candidate = if d.boolean()? {
            Some(get_report(d)?)
        } else {
            None
        };
        let accepted = if d.boolean()? {
            Some(get_report(d)?)
        } else {
            None
        };
        let pressure_call_ns = if d.boolean()? { Some(d.u64()?) } else { None };
        let residual_l2_limit = get_f64(d)?;
        Some(PressureFacts {
            selected,
            binding,
            prediction_called,
            support,
            rejection,
            rhs_sha256,
            candidate,
            accepted,
            pressure_call_ns,
            residual_l2_limit,
        })
    } else {
        None
    };
    let attempt = WorkflowAttempt {
        descriptor,
        descriptor_sha256,
        input_sha256,
        request_sha256,
        force_count,
        smoke_source_present,
        volume_source_present,
        run_id,
        attempt_id,
        accepted_generation,
        attempted_generation,
        interval_start,
        requested_dt,
        actual_dt,
        publication,
        outcome,
        nodes,
        pressure,
        actual_divergence_max: get_f64(d)?,
        diagnostic: d.string()?,
    };
    attempt.validate().map_err(RecordingError::Invalid)?;
    Ok(attempt)
}

#[cfg(test)]
mod tests {
    use super::super::RecordingLimits;
    use super::*;
    use crate::{
        LiquidInlet, LiquidStepInputs, LiquidTransportConfig, LiquidVolumeSettings,
        ProviderStepWorkspace, SessionIdentity, SimulationConfig, SimulationSession, VolumeStamp,
    };
    use std::sync::atomic::{AtomicU64, Ordering};
    const CAP: usize = 1024 * 1024;
    fn fixture() -> (SimulationSession, ProviderStepWorkspace) {
        fixture_with_iterations(1000)
    }
    fn fixture_with_iterations(
        max_iterations: usize,
    ) -> (SimulationSession, ProviderStepWorkspace) {
        let grid = GridGeometry::new([2; 3], [0.5; 3], [0.0; 3]).unwrap();
        let config = LiquidTransportConfig {
            carrier: SimulationConfig {
                density: 1000.0,
                memory_limit: CAP,
                pressure: PressureSettings {
                    relative_residual: 1e-12,
                    absolute_residual: 1e-12,
                    divergence_limit: 1e-9,
                    max_iterations,
                },
                actual_divergence_limit: 1e-5,
                max_courant: 1.0,
            },
            represented_density: 1000.0,
            volume_stamp: VolumeStamp { id: 1, version: 0 },
            carrier_id: 2,
        };
        let workspace =
            ProviderStepWorkspace::new(&grid, 1000.0, CAP, PressureImplementation::JacobiPcgV1)
                .unwrap();
        (
            SimulationSession::new(
                SessionIdentity {
                    run_id: 711,
                    source_revision: "workflow-custody-fixture".into(),
                },
                grid,
                config,
                PressureImplementation::JacobiPcgV1,
                vec![1.0; 8],
                CAP,
                4,
            )
            .unwrap(),
            workspace,
        )
    }
    fn inputs() -> LiquidStepInputs<'static> {
        LiquidStepInputs {
            requested_dt: 0.0625,
            smoke_source: None,
            forces: &[],
            source: None,
            inlet: LiquidInlet::new(VolumeStamp { id: 3, version: 0 }, [[0.0; 2]; 3]).unwrap(),
            volume: LiquidVolumeSettings {
                max_outward_courant: 1.0,
                actual_divergence_limit: 1e-5,
            },
        }
    }
    fn path() -> std::path::PathBuf {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        std::env::temp_dir().join(format!(
            "rheon-workflow-{}-{}.sqlite",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ))
    }
    fn wrap(run: &RecordingRun, original: &str, attempt: &WorkflowAttempt) -> String {
        let mut e = Encoder::default();
        e.0.extend_from_slice(&provider::source_binding(run));
        e.boolean(original.starts_with("RCP3:"));
        put_wrapped(
            &mut e,
            original,
            if original.starts_with("RCP3:") {
                "RCP3:"
            } else {
                "RCP2:"
            },
        )
        .unwrap();
        put_attempt(&mut e, attempt).unwrap();
        e.text(PUBLICATION_PROFILE).unwrap()
    }
    #[test]
    fn accepted_round_trip_and_legacy_profiles_preserve_original_meaning() {
        let (mut session, mut workspace) = fixture();
        session
            .step_with_configured_pressure(inputs(), &mut workspace, None, None, None, |_| false)
            .unwrap();
        let snapshot = session.snapshot();
        let run = provider::configured_run(&snapshot, "fixture").unwrap();
        let legacy = provider::configured_publication(&snapshot, 0, CAP).unwrap();
        assert!(legacy.controls.starts_with("RCP2:"));
        assert!(publication(&run, &legacy).unwrap().is_none());
        let step = configured_publication_with_workflow(&snapshot, 0, CAP, None).unwrap();
        let (original, expected, _) = split_publication_controls(&step.controls).unwrap().unwrap();
        assert_eq!(original, legacy.controls);
        let decoded = publication(&run, &step).unwrap().unwrap();
        assert_eq!(decoded.descriptor_sha256, expected.descriptor_sha256);
        assert_eq!(decoded.input_sha256, expected.input_sha256);
        assert_eq!(decoded.request_sha256, expected.request_sha256);
        assert_eq!(decoded.force_count, expected.force_count);
        assert_eq!(decoded.publication, Some(1));
        assert_eq!(decoded.accepted_generation, 0);
        let archive = path();
        let limits = RecordingLimits::default();
        let mut recorder = StepRecorder::create(&archive, run, limits).unwrap();
        recorder.append(&step).unwrap();
        WorkflowTermination::capture(
            &snapshot,
            RuntimeTerminationCause::Completed,
            "done",
            None,
            Some(&decoded),
        )
        .unwrap()
        .finish(&mut recorder)
        .unwrap();
        drop(recorder);
        drop(session);
        let reader = RecordingReader::open(&archive, limits).unwrap();
        let terminal = terminal(&reader).unwrap().unwrap();
        assert_eq!(terminal.runtime.termination.accepted_generation, 1);
        assert_eq!(terminal.attempt.unwrap().publication, Some(1));
        provider::verify_configured_recording(&reader).unwrap();
        drop(reader);
        std::fs::remove_file(archive).unwrap();
    }
    struct CandidateFixture {
        invalid: bool,
        observation: Option<RuntimeObservation>,
    }
    impl crate::solver_provider::configured::ContextPressureProvider for CandidateFixture {
        fn identity(&self) -> Arc<crate::solver_provider::configured::ContextProviderIdentity> {
            Arc::new(
                crate::solver_provider::configured::ContextProviderIdentity {
                    id: "workflow-classical-fixture".into(),
                    version: "1".into(),
                    manifest_sha256: [3; 32],
                    checkpoint_sha256: None,
                    training_run_id: None,
                    runtime: "native-fixture".into(),
                    source_revision: "fixture-source".into(),
                },
            )
        }
        fn supports(
            &self,
            _: &crate::solver_provider::configured::ContextPressureRequest<'_>,
        ) -> Result<(), crate::solver_provider::ProviderRejection> {
            Ok(())
        }
        fn predict(
            &mut self,
            request: &crate::solver_provider::configured::ContextPressureRequest<'_>,
            output: &mut [f64],
            _: &mut dyn FnMut() -> bool,
        ) -> Result<(), crate::solver_provider::ProviderRejection> {
            self.observation = Some(RuntimeObservation {
                request: runtime::RuntimeRequestStamp::capture(request, &self.identity()).unwrap(),
                outcome: runtime::RuntimeOutcome::CandidateReturned,
            });
            output.fill(if self.invalid { f64::NAN } else { 0.0 });
            Ok(())
        }
    }
    #[test]
    fn candidate_and_fallback_runtime_profiles_round_trip_actual_provider_identity() {
        for invalid in [false, true] {
            let (mut session, mut workspace) = fixture();
            let mut provider = CandidateFixture {
                invalid,
                observation: None,
            };
            let mut viscosity =
                crate::ViscosityWorkspace::new(session.snapshot().grid().clone(), CAP).unwrap();
            let mut evidence = None;
            session
                .step_with_configured_pressure_observed(
                    inputs(),
                    &mut workspace,
                    Some((&mut viscosity, 0.0)),
                    Some(&mut provider),
                    None,
                    |_| false,
                    6,
                    &mut evidence,
                )
                .unwrap();
            let evidence = evidence.unwrap();
            let run = provider::configured_run(&session.snapshot(), "runtime fixture").unwrap();
            let old = provider::configured_publication_with_runtime(
                &session.snapshot(),
                0,
                CAP,
                provider.observation.as_ref(),
            )
            .unwrap();
            assert!(old.controls.starts_with("RCP3:"));
            assert!(publication(&run, &old).unwrap().is_none());
            let row = configured_publication_with_workflow(
                &session.snapshot(),
                0,
                CAP,
                provider.observation.as_ref(),
            )
            .unwrap();
            let recorded = publication(&run, &row).unwrap().unwrap();
            assert_eq!(recorded.descriptor.viscosity, Some(0.0));
            assert_eq!(
                recorded.pressure.as_ref().unwrap().rejection.is_some(),
                invalid
            );
            assert_eq!(
                recorded
                    .pressure
                    .as_ref()
                    .unwrap()
                    .binding
                    .as_ref()
                    .unwrap()
                    .id,
                "workflow-classical-fixture"
            );
            let archive = path();
            let limits = RecordingLimits::default();
            let mut recorder = StepRecorder::create(&archive, run, limits).unwrap();
            recorder.append(&row).unwrap();
            WorkflowTermination::capture(
                &session.snapshot(),
                RuntimeTerminationCause::Completed,
                "done",
                provider.observation.as_ref(),
                Some(&evidence),
            )
            .unwrap()
            .finish(&mut recorder)
            .unwrap();
            drop(recorder);
            drop(session);
            drop(provider);
            let reader = RecordingReader::open(&archive, limits).unwrap();
            let retained = terminal(&reader).unwrap().unwrap();
            assert_eq!(retained.attempt.unwrap().attempt_id, 6);
            assert!(retained.runtime.termination.observation.is_some());
            drop(reader);
            std::fs::remove_file(archive).unwrap();
        }
    }

    #[test]
    fn cancellation_at_generation_zero_is_cold_inspectable_without_accepted_row() {
        for final_checkpoint in [false, true] {
            let (mut session, mut workspace) = fixture();
            let mut evidence = None;
            assert!(
                session
                    .step_with_configured_pressure_observed(
                        inputs(),
                        &mut workspace,
                        None,
                        None,
                        None,
                        |stage| !final_checkpoint || stage == crate::LiquidStepStage::BeforeCommit,
                        9,
                        &mut evidence
                    )
                    .is_err()
            );
            let attempt = evidence.unwrap();
            assert_eq!(attempt.accepted_generation, 0);
            assert_eq!(attempt.attempted_generation, 1);
            assert_eq!(attempt.publication, None);
            assert_eq!(attempt.outcome, AttemptOutcome::Cancelled);
            let archive = path();
            let limits = RecordingLimits::default();
            let run = provider::configured_run(&session.snapshot(), "empty cancellation").unwrap();
            let mut recorder = StepRecorder::create(&archive, run, limits).unwrap();
            WorkflowTermination::capture(
                &session.snapshot(),
                RuntimeTerminationCause::Cancelled,
                "cancelled",
                None,
                Some(&attempt),
            )
            .unwrap()
            .finish(&mut recorder)
            .unwrap();
            drop(recorder);
            drop(session);
            let reader = RecordingReader::open(&archive, limits).unwrap();
            assert_eq!(reader.count().unwrap(), 0);
            let retained = terminal(&reader).unwrap().unwrap();
            assert_eq!(retained.runtime.termination.accepted_generation, 0);
            let retained = retained.attempt.unwrap();
            assert_eq!(retained.attempt_id, 9);
            assert_eq!(retained.publication, None);
            assert_eq!(retained.pressure.is_some(), final_checkpoint);
            drop(reader);
            std::fs::remove_file(archive).unwrap();
        }
    }
    #[test]
    fn malformed_cross_linked_and_unknown_workflow_evidence_is_rejected() {
        let (mut session, mut workspace) = fixture();
        session
            .step_with_configured_pressure(inputs(), &mut workspace, None, None, None, |_| false)
            .unwrap();
        let run = provider::configured_run(&session.snapshot(), "fixture").unwrap();
        let mut step =
            configured_publication_with_workflow(&session.snapshot(), 0, CAP, None).unwrap();
        let (original, attempt, _) = split_publication_controls(&step.controls).unwrap().unwrap();
        let good = step.controls.clone();
        let mut other = (*attempt).clone();
        other.run_id += 1;
        step.controls = wrap(&run, &original, &other);
        assert!(publication(&run, &step).is_err());
        let mut other_source = run.clone();
        other_source.identity.source_revision.push_str("-different");
        step.controls = wrap(&other_source, &original, &attempt);
        assert!(publication(&run, &step).is_err());
        step.controls = good.clone();
        step.controls.pop();
        assert!(publication(&run, &step).is_err());
        step.controls = good;
        step.time += 0.01;
        assert!(publication(&run, &step).is_err());
        let mut e = Encoder::default();
        put_attempt(&mut e, &attempt).unwrap();
        e.0[0..4].copy_from_slice(&99u32.to_le_bytes());
        let text = e.text("TEST:").unwrap();
        let mut d = Decoder::text(&text, "TEST:").unwrap();
        assert!(matches!(
            get_attempt(&mut d),
            Err(RecordingError::Unsupported(_))
        ));
        let mut e = Encoder::default();
        e.u32((super::super::MAX_TEXT + 1) as u32);
        let text = e.text("TEST:").unwrap();
        let mut d = Decoder::text(&text, "TEST:").unwrap();
        assert!(matches!(
            get_wrapped(&mut d, "RCT1:"),
            Err(RecordingError::Limit(_))
        ));
    }
    #[test]
    fn terminal_rejects_contradictory_runtime_outcome_and_unknown_version() {
        let (mut session, mut workspace) = fixture();
        let mut provider = CandidateFixture {
            invalid: false,
            observation: None,
        };
        let mut evidence = None;
        assert!(
            session
                .step_with_configured_pressure_observed(
                    inputs(),
                    &mut workspace,
                    None,
                    Some(&mut provider),
                    None,
                    |stage| stage == crate::LiquidStepStage::BeforeCommit,
                    4,
                    &mut evidence
                )
                .is_err()
        );
        let evidence = evidence.unwrap();
        let mut contradiction = provider.observation.clone().unwrap();
        contradiction.outcome = runtime::RuntimeOutcome::NoCandidate {
            reason: super::super::diagnostic::NoCandidateReason::Cancelled,
            detail: "authored contradiction".into(),
            dispatched: true,
            invalid_payload_sha256: None,
        };
        assert!(matches!(
            WorkflowTermination::capture(
                &session.snapshot(),
                RuntimeTerminationCause::Cancelled,
                "cancelled",
                Some(&contradiction),
                Some(&evidence)
            ),
            Err(RecordingError::Invalid(
                "workflow runtime outcome association"
            ))
        ));
        let mut terminal = WorkflowTermination::capture(
            &session.snapshot(),
            RuntimeTerminationCause::Cancelled,
            "cancelled",
            provider.observation.as_ref(),
            Some(&evidence),
        )
        .unwrap();
        terminal.runtime.observation = Some(contradiction);
        let archive = path();
        let limits = RecordingLimits::default();
        let run = provider::configured_run(&session.snapshot(), "runtime contradiction").unwrap();
        let mut recorder = StepRecorder::create(&archive, run, limits).unwrap();
        recorder
            .finish("cancelled", &terminal.encode().unwrap(), Some(0))
            .unwrap();
        drop(recorder);
        let reader = RecordingReader::open(&archive, limits).unwrap();
        assert!(matches!(
            super::terminal(&reader),
            Err(RecordingError::Invalid(
                "workflow runtime outcome association"
            ))
        ));
        drop(reader);
        std::fs::remove_file(archive).unwrap();
        let archive = path();
        let run = provider::configured_run(&session.snapshot(), "unknown terminal").unwrap();
        let mut recorder = StepRecorder::create(&archive, run, limits).unwrap();
        recorder.finish("failed", "RCT3:00", Some(0)).unwrap();
        drop(recorder);
        let reader = RecordingReader::open(&archive, limits).unwrap();
        assert!(matches!(
            super::terminal(&reader),
            Err(RecordingError::Unsupported("workflow terminal version"))
        ));
        drop(reader);
        std::fs::remove_file(archive).unwrap();
    }

    #[test]
    fn forged_native_input_hash_is_rejected_by_bounded_selection_and_archive_chain() {
        let (mut session, mut workspace) = fixture();
        let constructor = provider::configured_publication(&session.snapshot(), 0, CAP).unwrap();
        session
            .step_with_configured_pressure(inputs(), &mut workspace, None, None, None, |_| false)
            .unwrap();
        let run = provider::configured_run(&session.snapshot(), "input-link fixture").unwrap();
        let mut row =
            configured_publication_with_workflow(&session.snapshot(), 1, CAP, None).unwrap();
        let (original, attempt, _) = split_publication_controls(&row.controls).unwrap().unwrap();
        assert_eq!(
            native_state_digest(
                &provider::RecordedConfiguredSession::decode(&run, &constructor).unwrap()
            )
            .unwrap(),
            attempt.input_sha256
        );
        let mut forged = (*attempt).clone();
        forged.input_sha256[0] ^= 1;
        row.controls = wrap(&run, &original, &forged);
        // The row remains intrinsically well formed; only the prior native
        // publication proves this reachable cross-link is false.
        publication(&run, &row).unwrap();
        let archive = path();
        let limits = RecordingLimits::default();
        let mut recorder = StepRecorder::create(&archive, run, limits).unwrap();
        recorder.append(&constructor).unwrap();
        recorder.append(&row).unwrap();
        recorder
            .finish("failed", "authored cross-link refusal fixture", Some(1))
            .unwrap();
        drop(recorder);
        let reader = RecordingReader::open(&archive, limits).unwrap();
        assert!(matches!(
            selection(&reader, &row),
            Err(RecordingError::Invalid(
                "workflow selected prior native input"
            ))
        ));
        assert!(matches!(
            provider::verify_configured_recording(&reader),
            Err(RecordingError::Invalid("workflow prior native input"))
        ));
        drop(reader);
        std::fs::remove_file(archive).unwrap();
    }

    #[test]
    fn duplicate_attempt_ids_are_rejected_for_publications_and_next_terminal() {
        let (mut session, mut workspace) = fixture();
        session
            .step_with_configured_pressure(inputs(), &mut workspace, None, None, None, |_| false)
            .unwrap();
        let first =
            configured_publication_with_workflow(&session.snapshot(), 0, CAP, None).unwrap();
        let run =
            provider::configured_run(&session.snapshot(), "attempt sequence fixture").unwrap();
        let mut cancelled = None;
        assert!(
            session
                .step_with_configured_pressure_observed(
                    inputs(),
                    &mut workspace,
                    None,
                    None,
                    None,
                    |stage| stage == crate::LiquidStepStage::BeforeCommit,
                    1,
                    &mut cancelled
                )
                .is_err()
        );
        let archive = path();
        let limits = RecordingLimits::default();
        let mut recorder = StepRecorder::create(&archive, run.clone(), limits).unwrap();
        recorder.append(&first).unwrap();
        WorkflowTermination::capture(
            &session.snapshot(),
            RuntimeTerminationCause::Cancelled,
            "duplicate attempted id",
            None,
            cancelled.as_ref(),
        )
        .unwrap()
        .finish(&mut recorder)
        .unwrap();
        drop(recorder);
        let reader = RecordingReader::open(&archive, limits).unwrap();
        assert!(matches!(
            terminal(&reader),
            Err(RecordingError::Invalid(
                "workflow terminal attempt identity"
            ))
        ));
        drop(reader);
        std::fs::remove_file(archive).unwrap();
        session
            .step_with_configured_pressure(inputs(), &mut workspace, None, None, None, |_| false)
            .unwrap();
        let mut second =
            configured_publication_with_workflow(&session.snapshot(), 1, CAP, None).unwrap();
        let (original, attempt, _) = split_publication_controls(&second.controls)
            .unwrap()
            .unwrap();
        let mut forged = (*attempt).clone();
        forged.attempt_id = 1;
        second.controls = wrap(&run, &original, &forged);
        let archive = path();
        let mut recorder = StepRecorder::create(&archive, run, limits).unwrap();
        recorder.append(&first).unwrap();
        recorder.append(&second).unwrap();
        recorder
            .finish("failed", "authored attempt-id refusal fixture", Some(2))
            .unwrap();
        drop(recorder);
        let reader = RecordingReader::open(&archive, limits).unwrap();
        assert!(matches!(
            selection(&reader, &second),
            Err(RecordingError::Invalid(
                "workflow selected attempt identity"
            ))
        ));
        assert!(matches!(
            provider::verify_configured_recording(&reader),
            Err(RecordingError::Invalid(
                "workflow attempt identity sequence"
            ))
        ));
        drop(reader);
        std::fs::remove_file(archive).unwrap();
    }

    #[test]
    fn legacy_terminal_is_unavailable_and_storage_failure_preserves_accepted_tail() {
        let (mut session, mut workspace) = fixture();
        let limits = RecordingLimits::default();
        let legacy_path = path();
        let run = provider::configured_run(&session.snapshot(), "legacy").unwrap();
        let mut recorder = StepRecorder::create(&legacy_path, run, limits).unwrap();
        RuntimeTermination::capture(
            &session.snapshot(),
            RuntimeTerminationCause::StartupFailure,
            "legacy",
            None,
        )
        .unwrap()
        .finish(&mut recorder)
        .unwrap();
        drop(recorder);
        let reader = RecordingReader::open(&legacy_path, limits).unwrap();
        assert!(terminal(&reader).unwrap().is_none());
        runtime::audit_runtime_termination(&reader).unwrap();
        drop(reader);
        std::fs::remove_file(legacy_path).unwrap();
        let constructor = provider::configured_publication(&session.snapshot(), 0, CAP).unwrap();
        session
            .step_with_configured_pressure(inputs(), &mut workspace, None, None, None, |_| false)
            .unwrap();
        let evidence = session
            .snapshot()
            .configured_provider_report()
            .unwrap()
            .workflow
            .clone()
            .unwrap();
        for (keep_constructor, forge_input) in [(false, false), (true, false), (true, true)] {
            let archive = path();
            let run = provider::configured_run(&session.snapshot(), "unrecorded tail").unwrap();
            let mut recorder = StepRecorder::create(&archive, run, limits).unwrap();
            if keep_constructor {
                recorder.append(&constructor).unwrap();
            }
            let mut retained = (*evidence).clone();
            if forge_input {
                retained.input_sha256[0] ^= 1;
            }
            WorkflowTermination::capture(
                &session.snapshot(),
                RuntimeTerminationCause::StorageFailure,
                "row not recorded",
                None,
                Some(&Arc::new(retained)),
            )
            .unwrap()
            .finish(&mut recorder)
            .unwrap();
            drop(recorder);
            let reader = RecordingReader::open(&archive, limits).unwrap();
            if forge_input {
                assert!(matches!(
                    terminal(&reader),
                    Err(RecordingError::Invalid("workflow terminal native input"))
                ));
            } else {
                let audit = terminal(&reader).unwrap().unwrap();
                assert!(
                    matches!(audit.runtime.coverage,runtime::RuntimeTerminalCoverage::UnrecordedAcceptedTail {last_recorded_generation} if last_recorded_generation == keep_constructor.then_some(0))
                );
                assert_eq!(audit.attempt.unwrap().publication, Some(1));
            }
            drop(reader);
            std::fs::remove_file(archive).unwrap();
        }
    }

    #[derive(Clone, Copy, Debug)]
    enum IterationTrace {
        Native,
        RejectedCandidate,
        AcceptedCandidate,
    }
    fn iteration_trace(
        kind: IterationTrace,
        cancelled: bool,
        max_iterations: usize,
    ) -> (SimulationSession, Arc<WorkflowAttempt>) {
        use crate::solver_provider::configured::ContextPressureProvider;
        let (mut session, mut workspace) = fixture_with_iterations(max_iterations);
        let mut candidate = CandidateFixture {
            invalid: false,
            observation: None,
        };
        let forces = [crate::BodyForce {
            value: [1.0, 0.0, 0.0],
            units: crate::ForceUnits::Acceleration,
            region: None,
        }];
        let mut request = inputs();
        if matches!(kind, IterationTrace::RejectedCandidate) {
            request.forces = &forces;
        }
        let preferred: Option<&mut dyn ContextPressureProvider> =
            if matches!(kind, IterationTrace::Native) {
                None
            } else {
                Some(&mut candidate)
            };
        let mut evidence = None;
        let result = session.step_with_configured_pressure_observed(
            request,
            &mut workspace,
            None,
            preferred,
            None,
            |stage| cancelled && stage == crate::LiquidStepStage::BeforeCommit,
            1,
            &mut evidence,
        );
        assert_eq!(result.is_err(), cancelled);
        let evidence = evidence.unwrap();
        evidence.validate().unwrap();
        if matches!(kind, IterationTrace::RejectedCandidate) {
            assert_eq!(
                evidence.pressure.as_ref().unwrap().rejection.as_deref(),
                Some("QualityGate")
            );
            assert!(evidence.pressure.as_ref().unwrap().candidate.is_some());
        }
        (session, evidence)
    }
    fn iteration_bad_count(trace: &WorkflowAttempt, kind: IterationTrace) -> WorkflowAttempt {
        let mut bad = trace.clone();
        let pressure = bad.pressure.as_mut().unwrap();
        match kind {
            IterationTrace::Native => {
                pressure.accepted.as_mut().unwrap().iterations =
                    bad.descriptor.pressure.max_iterations + 1
            }
            IterationTrace::RejectedCandidate => {
                let candidate = pressure.candidate.as_mut().unwrap();
                // Rejected numerical diagnostics retain their IEEE payloads;
                // their iteration count still belongs to the supplied stage.
                candidate.true_residual_l2 = f64::NAN;
                candidate.true_residual_max = f64::INFINITY;
                candidate.predicted_divergence_max = f64::NEG_INFINITY;
                candidate.iterations = 1;
            }
            IterationTrace::AcceptedCandidate => {
                pressure.candidate.as_mut().unwrap().iterations = 1;
                pressure.accepted.as_mut().unwrap().iterations = 1;
            }
        }
        bad
    }
    // This negative-fixture helper first uses the real encoder on an admissible
    // trace, then changes only the serialized iteration u64s. It deliberately
    // bypasses outbound validation so cold-reader refusal is independently tested.
    fn iteration_bad_attempt_bytes(attempt: &WorkflowAttempt) -> Vec<u8> {
        let mut valid = attempt.clone();
        let pressure = valid.pressure.as_mut().unwrap();
        if let Some(report) = &mut pressure.candidate {
            report.iterations = 0;
        }
        if let Some(report) = &mut pressure.accepted {
            report.iterations = 0;
        }
        let mut encoder = Encoder::default();
        put_attempt(&mut encoder, &valid).unwrap();
        let pressure = attempt.pressure.as_ref().unwrap();
        // Tail after the accepted report: optional pressure-call ns, optional
        // residual limit, optional actual divergence, then bounded UTF-8 detail.
        let tail = 1
            + usize::from(pressure.pressure_call_ns.is_some()) * 8
            + 1
            + usize::from(pressure.residual_l2_limit.is_some()) * 8
            + 1
            + usize::from(attempt.actual_divergence_max.is_some()) * 8
            + 4
            + attempt.diagnostic.len();
        let end = encoder.0.len() - tail;
        if let Some(report) = pressure.accepted {
            encoder.0[end - 32..end - 24]
                .copy_from_slice(&(report.iterations as u64).to_le_bytes());
        }
        if let Some(report) = pressure.candidate {
            let candidate_end = end - 1 - usize::from(pressure.accepted.is_some()) * 32;
            encoder.0[candidate_end - 32..candidate_end - 24]
                .copy_from_slice(&(report.iterations as u64).to_le_bytes());
        }
        encoder.0
    }
    fn iteration_rcp4_controls(
        run: &RecordingRun,
        inner: &str,
        attempt: &WorkflowAttempt,
    ) -> String {
        let mut encoder = Encoder::default();
        encoder.0.extend_from_slice(&provider::source_binding(run));
        encoder.boolean(false);
        put_wrapped(&mut encoder, inner, "RCP2:").unwrap();
        encoder
            .0
            .extend_from_slice(&iteration_bad_attempt_bytes(attempt));
        encoder.text(PUBLICATION_PROFILE).unwrap()
    }
    fn iteration_check_standalone(kind: IterationTrace) {
        for cancelled in [false, true] {
            let (_, trace) = iteration_trace(kind, cancelled, 1000);
            let mut bad = iteration_bad_count(&trace, kind);
            assert!(
                bad.validate().is_err(),
                "accepted malformed {kind:?}; cancelled={cancelled}"
            );
            if cancelled {
                // A failed later stage has the same completed pressure prefix.
                bad.outcome = AttemptOutcome::Failed;
                assert!(
                    bad.validate().is_err(),
                    "accepted malformed failed-later {kind:?}"
                );
            }
        }
    }
    #[test]
    fn iteration_limits_standalone_native_pressure_count() {
        iteration_check_standalone(IterationTrace::Native);
    }
    #[test]
    fn iteration_limits_standalone_rejected_candidate_count() {
        iteration_check_standalone(IterationTrace::RejectedCandidate);
    }
    #[test]
    fn iteration_limits_standalone_accepted_candidate_count() {
        iteration_check_standalone(IterationTrace::AcceptedCandidate);
    }
    #[test]
    fn iteration_limits_valid_edges_and_zero_budget_candidate_are_preserved() {
        let (_, native) = iteration_trace(IterationTrace::Native, false, 1000);
        let mut at_cap = (*native).clone();
        at_cap
            .pressure
            .as_mut()
            .unwrap()
            .accepted
            .as_mut()
            .unwrap()
            .iterations = 1000;
        at_cap.validate().unwrap();
        let (_, candidate) = iteration_trace(IterationTrace::AcceptedCandidate, false, 0);
        assert_eq!(candidate.descriptor.pressure.max_iterations, 0);
        assert_eq!(
            candidate
                .pressure
                .as_ref()
                .unwrap()
                .candidate
                .unwrap()
                .iterations,
            0
        );
        assert_eq!(
            candidate
                .pressure
                .as_ref()
                .unwrap()
                .accepted
                .unwrap()
                .iterations,
            0
        );
        candidate.validate().unwrap();
        let (_, rejected) = iteration_trace(IterationTrace::RejectedCandidate, true, 1000);
        let mut ieee = iteration_bad_count(&rejected, IterationTrace::RejectedCandidate);
        ieee.pressure
            .as_mut()
            .unwrap()
            .candidate
            .as_mut()
            .unwrap()
            .iterations = 0;
        ieee.validate().unwrap();
        let mut encoder = Encoder::default();
        put_attempt(&mut encoder, &ieee).unwrap();
        let text = encoder.text("TEST:").unwrap();
        let mut decoder = Decoder::text(&text, "TEST:").unwrap();
        let decoded = get_attempt(&mut decoder).unwrap();
        let report = decoded.pressure.as_ref().unwrap().candidate.unwrap();
        assert_eq!(report.true_residual_l2.to_bits(), f64::NAN.to_bits());
        assert_eq!(report.true_residual_max.to_bits(), f64::INFINITY.to_bits());
        assert_eq!(
            report.predicted_divergence_max.to_bits(),
            f64::NEG_INFINITY.to_bits()
        );
    }
    #[test]
    fn iteration_limits_rcp4_matching_inner_counts_already_hit_old_receipt_gate() {
        for kind in [
            IterationTrace::Native,
            IterationTrace::RejectedCandidate,
            IterationTrace::AcceptedCandidate,
        ] {
            let (session, trace) = iteration_trace(kind, false, 1000);
            let run = provider::configured_run(&session.snapshot(), "old-inner iteration control")
                .unwrap();
            let mut row = provider::configured_publication(&session.snapshot(), 0, CAP).unwrap();
            let mut bad = iteration_bad_count(&trace, kind);
            // Only counts differ here: the coherent inner and outer diagnostic
            // values otherwise remain the real, finite producer observations.
            if matches!(kind, IterationTrace::RejectedCandidate) {
                bad.pressure.as_mut().unwrap().candidate = trace
                    .pressure
                    .as_ref()
                    .unwrap()
                    .candidate
                    .map(|mut report| {
                        report.iterations = 1;
                        report
                    });
            }
            let pressure = bad.pressure.as_ref().unwrap();
            let mut inner = Decoder::text(&row.controls, "RCP2:").unwrap().data;
            // Unchanged RCP2 receipt ends in accepted PressureReport (32 bytes)
            // and four duration pairs (u64 seconds, u32 nanoseconds).
            let accepted_start = inner.len() - 4 * 12 - 32;
            inner[accepted_start..accepted_start + 8]
                .copy_from_slice(&(pressure.accepted.unwrap().iterations as u64).to_le_bytes());
            if let Some(candidate) = pressure.candidate {
                let candidate_start = accepted_start - 32;
                inner[candidate_start..candidate_start + 8]
                    .copy_from_slice(&(candidate.iterations as u64).to_le_bytes());
            }
            let inner = Encoder(inner).text("RCP2:").unwrap();
            row.controls = inner.clone();
            let expected = if matches!(kind, IterationTrace::Native) {
                "configured accepted pressure gates"
            } else {
                "configured candidate diagnostics"
            };
            assert!(
                matches!(provider::RecordedConfiguredSession::decode(&run, &row), Err(RecordingError::Invalid(reason)) if reason == expected),
                "unchanged RCP2 failed to refuse {kind:?}"
            );
            row.controls = iteration_rcp4_controls(&run, &inner, &bad);
            assert!(
                publication(&run, &row).is_err(),
                "matching forged RCP4 escaped {kind:?}"
            );
        }
    }
    fn iteration_check_cold_terminal(kind: IterationTrace) {
        for cancelled in [false, true] {
            let (session, trace) = iteration_trace(kind, cancelled, 1000);
            let bad = iteration_bad_count(&trace, kind);
            let cause = if cancelled {
                RuntimeTerminationCause::Cancelled
            } else {
                RuntimeTerminationCause::StorageFailure
            };
            let runtime = RuntimeTermination::capture(
                &session.snapshot(),
                cause,
                "authored malformed iteration evidence",
                None,
            )
            .unwrap();
            let run =
                provider::configured_run(&session.snapshot(), "cold iteration negative fixture")
                    .unwrap();
            let mut encoder = Encoder::default();
            encoder.0.extend_from_slice(&provider::source_binding(&run));
            put_wrapped(&mut encoder, &runtime.encode().unwrap(), "RCT1:").unwrap();
            encoder.boolean(true);
            encoder
                .0
                .extend_from_slice(&iteration_bad_attempt_bytes(&bad));
            let archive = path();
            let limits = RecordingLimits::default();
            let mut recorder = StepRecorder::create(&archive, run, limits).unwrap();
            recorder
                .finish(
                    if cancelled { "cancelled" } else { "failed" },
                    &encoder.text(TERMINAL_PROFILE).unwrap(),
                    Some(session.snapshot().state().carrier.generation),
                )
                .unwrap();
            drop(recorder);
            drop(session);
            let reader = RecordingReader::open(&archive, limits).unwrap();
            assert_eq!(reader.count().unwrap(), 0);
            let result = terminal(&reader);
            let refused = matches!(result, Err(RecordingError::Invalid(_)));
            drop(reader);
            std::fs::remove_file(archive).unwrap();
            assert!(
                refused,
                "cold RCT2 accepted malformed {kind:?}; cancelled={cancelled}"
            );
        }
    }
    #[test]
    fn iteration_limits_rct2_native_pressure_count() {
        iteration_check_cold_terminal(IterationTrace::Native);
    }
    #[test]
    fn iteration_limits_rct2_rejected_candidate_count() {
        iteration_check_cold_terminal(IterationTrace::RejectedCandidate);
    }
    #[test]
    fn iteration_limits_rct2_accepted_candidate_count() {
        iteration_check_cold_terminal(IterationTrace::AcceptedCandidate);
    }
}
