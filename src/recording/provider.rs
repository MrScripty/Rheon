//! Typed configured-Session publications inside the existing SQLite v1 envelope.
//! This is inspection evidence, not a restart checkpoint or an executable recipe.
use super::*;
use crate::{
    Axis, LiquidTransportConfig, PressureImplementation, PressureReport, PressureSettings,
    SessionConfiguration, SessionSnapshot, SimulationConfig, VolumeStamp,
    solver_provider::{
        ProviderRejection, SolverMetadata,
        configured::{
            AcceptedPressureContextStamp, ConfiguredProviderReport, ContextProviderIdentity,
        },
    },
};
use std::time::Duration;

pub const PROFILE: &str = "rheon-configured-session-v1";
const CONFIG: &str = "RSC1:";
const RECEIPT: &str = "RCP1:";
const CELL: &str = "cell_center; offset [0.5,0.5,0.5]; Cartesian XYZ; x-fast";
const FACE: [&str; 3] = [
    "x_face; offset [0,0.5,0.5]; Cartesian XYZ; x-fast",
    "y_face; offset [0.5,0,0.5]; Cartesian XYZ; x-fast",
    "z_face; offset [0.5,0.5,0]; Cartesian XYZ; x-fast",
];
const PRESSURE: &str = "accepted_interval_projection; cell0 gauge=0; not endpoint pressure";
const ROLE: &str = "configured-pressure-context-v1";
const DOMAIN: &str = "host-composed sealed constant-density MAC; package pins in binding receipt";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordedPressureSelection {
    Native(PressureImplementation),
    Configured,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordedProviderRejection {
    Unsupported(String),
    Execution(String),
    NonFiniteOutput,
    NonZeroGauge,
    QualityGate,
    Cancelled,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedTensorMetadata {
    pub shape_xyz: [usize; 3],
    pub layout: String,
    pub scalar: String,
    pub units: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedSolverMetadata {
    pub id: String,
    pub version: String,
    pub stage: String,
    pub input: RecordedTensorMetadata,
    pub output: RecordedTensorMetadata,
    pub domain: String,
}
/// Owned observations. Rejected candidate metrics preserve all IEEE bits,
/// including NaN/Inf; accepted diagnostics remain finite and nonnegative.
#[derive(Debug, Clone)]
pub struct RecordedConfiguredReceipt {
    pub binding: Option<ContextProviderIdentity>,
    pub context: Option<AcceptedPressureContextStamp>,
    pub dynamic_viscosity: f64,
    pub prediction_called: bool,
    pub selected: RecordedPressureSelection,
    pub attempted: Option<RecordedSolverMetadata>,
    pub rejection: Option<RecordedProviderRejection>,
    pub candidate: Option<PressureReport>,
    pub accepted: PressureReport,
    pub total: Duration,
    pub inference: Duration,
    pub qualification: Duration,
    pub fallback: Duration,
}
/// One owned selected publication; no runtime/provider is constructed by decoding.
pub struct RecordedConfiguredSession {
    pub grid: GridGeometry,
    pub configuration: SessionConfiguration,
    pub generation: u64,
    pub time: f64,
    pub dt: Option<f64>,
    pub carrier_stamp: VolumeStamp,
    pub fraction_stamp: VolumeStamp,
    pub fraction_time: f64,
    pub x: Vec<f32>,
    pub y: Vec<f32>,
    pub z: Vec<f32>,
    pub tracer: Vec<f32>,
    pub fraction: Vec<f64>,
    pub interval_pressure: Option<(f64, Vec<f64>)>,
    pub receipt: Option<RecordedConfiguredReceipt>,
}
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ConfiguredHistoryCoverage {
    pub verified_prior_links: usize,
    pub unavailable_prior_links: usize,
}

/// Capture actual immutable constructor configuration, not Debug text or future
/// force/provider schedules. Provenance remains a caller declaration.
pub fn configured_run(
    snapshot: &SessionSnapshot<'_>,
    provenance: &str,
) -> Result<RecordingRun, RecordingError> {
    let grid = snapshot.grid();
    let mut e = Encoder::default();
    put_configuration(&mut e, snapshot.configuration());
    let run = RecordingRun {
        identity: snapshot.identity().clone(),
        solver: PROFILE.into(),
        configuration: e.text(CONFIG)?,
        counts: grid.counts().map(|n| n as u64),
        spacing: grid.spacing(),
        origin: grid.origin(),
        provenance: provenance.into(),
    };
    run.validate()?;
    configuration(&run)?;
    Ok(run)
}
/// Capture the Session's accepted arrays and receipt, never provider scratch.
/// A failed later attempt leaves the same snapshot; callers append only after
/// a successful interval. Recording failure never rolls physics back.
pub fn configured_publication(
    snapshot: &SessionSnapshot<'_>,
    ordinal: usize,
    limit: usize,
) -> Result<RecordedStep, RecordingError> {
    let state = snapshot.state();
    let run = configured_run(snapshot, "source binding")?;
    if state.pressure_surface.is_some()
        || state.pressure_columns.is_some()
        || state.reconstructed_surface.is_some()
        || state.flat_column_mac
    {
        return Err(RecordingError::Unsupported("configured recording geometry"));
    }
    let initial = snapshot.stage() == SnapshotStage::Constructor;
    let receipt = if initial {
        if snapshot.configured_provider_report().is_some() {
            return Err(RecordingError::Invalid("constructor provider receipt"));
        }
        None
    } else {
        Some(owned_receipt(
            snapshot
                .configured_provider_report()
                .ok_or(RecordingError::Unsupported(
                    "accepted step has no configured receipt",
                ))?,
        )?)
    };
    let report = snapshot.report();
    let dt = report.map(|r| r.carrier.step.dt);
    if report.is_some_and(|r| {
        r.carrier.step.generation != state.carrier.generation
            || r.carrier.step.time.to_bits() != state.carrier.time.to_bits()
    }) {
        return Err(RecordingError::Invalid("configured snapshot report"));
    }
    if state.liquid.density.to_bits()
        != snapshot
            .configuration()
            .transport
            .represented_density
            .to_bits()
    {
        return Err(RecordingError::Invalid("configured snapshot density"));
    }
    let start = snapshot.accepted_interval_start();
    if initial != start.is_none() {
        return Err(RecordingError::Invalid("configured snapshot interval"));
    }
    let mut e = Encoder::default();
    e.0.extend_from_slice(&source_binding(&run));
    e.u64(state.carrier_stamp.id);
    e.u64(state.carrier_stamp.version);
    e.u64(state.liquid.stamp.id);
    e.u64(state.liquid.stamp.version);
    e.f64(state.liquid.time);
    e.boolean(receipt.is_some());
    if let Some(r) = &receipt {
        put_receipt(&mut e, r)?;
    }
    let controls = e.text(RECEIPT)?;
    let n = state
        .carrier
        .x
        .len()
        .checked_add(state.carrier.y.len())
        .and_then(|n| n.checked_add(state.carrier.z.len()))
        .and_then(|n| n.checked_add(state.carrier.tracer.len()))
        .and_then(|n| n.checked_mul(4))
        .and_then(|n| n.checked_add(state.liquid.fraction.len().checked_mul(8)?))
        .and_then(|n| {
            n.checked_add(if initial {
                0
            } else {
                state.pressure.len().checked_mul(8)?
            })
        })
        .and_then(|n| n.checked_add(controls.len() + 2048))
        .ok_or(RecordingError::Limit("configured payload overflow"))?;
    if n > limit {
        return Err(RecordingError::Limit("configured record payload"));
    }
    let stage = if initial { "constructor" } else { "accepted" };
    let grid = snapshot.grid();
    let mut fields = Vec::new();
    for (i, (name, axis, values)) in [
        ("velocity_x", Axis::X, state.carrier.x),
        ("velocity_y", Axis::Y, state.carrier.y),
        ("velocity_z", Axis::Z, state.carrier.z),
    ]
    .into_iter()
    .enumerate()
    {
        fields.push(field(
            name,
            "m/s",
            FACE[i],
            stage,
            None,
            grid.face_counts(axis),
            "f32",
            bytes(values.iter().map(|v| v.to_le_bytes()), values.len(), 4)?,
        )?);
    }
    fields.push(field(
        "tracer",
        "1",
        CELL,
        stage,
        None,
        grid.counts(),
        "f32",
        bytes(
            state.carrier.tracer.iter().map(|v| v.to_le_bytes()),
            state.carrier.tracer.len(),
            4,
        )?,
    )?);
    fields.push(field(
        "fraction",
        "1",
        CELL,
        stage,
        None,
        grid.counts(),
        "f64",
        bytes(
            state.liquid.fraction.iter().map(|v| v.to_le_bytes()),
            state.liquid.fraction.len(),
            8,
        )?,
    )?);
    if !initial {
        fields.push(field(
            "interval_pressure",
            "Pa",
            CELL,
            PRESSURE,
            start,
            grid.counts(),
            "f64",
            bytes(
                state.pressure.iter().map(|v| v.to_le_bytes()),
                state.pressure.len(),
                8,
            )?,
        )?);
    }
    let step = RecordedStep {
        ordinal,
        generation: state.carrier.generation,
        time: state.carrier.time,
        dt,
        stage: snapshot.stage(),
        controls,
        fields,
    };
    if step.payload_bytes()? > limit {
        return Err(RecordingError::Limit("configured record payload"));
    }
    // The immutable Session owns publication association. Validate receipt
    // semantics here without allocating a second copy of every captured array.
    if let Some(receipt) = &receipt {
        validate_receipt(
            receipt,
            snapshot.configuration(),
            snapshot.grid(),
            &step,
            start.ok_or(RecordingError::Invalid("configured interval"))?,
        )?;
    }
    Ok(step)
}

impl RecordedConfiguredSession {
    pub fn decode(run: &RecordingRun, step: &RecordedStep) -> Result<Self, RecordingError> {
        run.validate()?;
        step.payload_bytes()?;
        let configuration = configuration(run)?;
        let grid = GridGeometry::new(run.counts, run.spacing, run.origin)
            .map_err(|_| RecordingError::Invalid("configured geometry"))?;
        let mut d = Decoder::text(&step.controls, RECEIPT)?;
        if d.array::<32>()? != source_binding(run) {
            return Err(RecordingError::Invalid("configured source binding"));
        }
        let carrier_stamp = VolumeStamp {
            id: d.u64()?,
            version: d.u64()?,
        };
        let fraction_stamp = VolumeStamp {
            id: d.u64()?,
            version: d.u64()?,
        };
        let fraction_time = d.f64()?;
        let receipt = if d.boolean()? {
            Some(get_receipt(&mut d)?)
        } else {
            None
        };
        d.end()?;
        let initial = step.stage == SnapshotStage::Constructor;
        if initial != receipt.is_none()
            || step.fields.len() != if initial { 5 } else { 6 }
            || carrier_stamp.id != configuration.transport.carrier_id
            || carrier_stamp.version != step.generation
            || fraction_stamp.id != configuration.transport.volume_stamp.id
            || configuration
                .transport
                .volume_stamp
                .version
                .checked_add(step.generation)
                != Some(fraction_stamp.version)
            || fraction_time.to_bits() != step.time.to_bits()
            || step.generation > configuration.max_steps as u64
        {
            return Err(RecordingError::Invalid(
                "configured publication association",
            ));
        }
        let numerical_bytes = step
            .fields
            .iter()
            .try_fold(0usize, |n, f| n.checked_add(f.data.len()))
            .ok_or(RecordingError::Limit("configured field bytes"))?;
        if numerical_bytes > configuration.transport.carrier.memory_limit {
            return Err(RecordingError::Invalid(
                "configured constructor memory bound",
            ));
        }
        let stage = if initial { "constructor" } else { "accepted" };
        let mut faces = Vec::new();
        for (i, (name, axis)) in [
            ("velocity_x", Axis::X),
            ("velocity_y", Axis::Y),
            ("velocity_z", Axis::Z),
        ]
        .into_iter()
        .enumerate()
        {
            faces.push(f32_values(required(
                step,
                name,
                "f32",
                "m/s",
                FACE[i],
                stage,
                grid.face_counts(axis),
                None,
            )?)?);
        }
        let tracer = f32_values(required(
            step,
            "tracer",
            "f32",
            "1",
            CELL,
            stage,
            grid.counts(),
            None,
        )?)?;
        let fraction = f64_values(required(
            step,
            "fraction",
            "f64",
            "1",
            CELL,
            stage,
            grid.counts(),
            None,
        )?)?;
        if tracer.iter().any(|v| !(0.0..=1.0).contains(v))
            || fraction.iter().any(|v| !(0.0..=1.0).contains(v))
        {
            return Err(RecordingError::Invalid("configured scalar range"));
        }
        if receipt.as_ref().is_some_and(|r| r.dynamic_viscosity > 0.0)
            && configuration.transport.carrier.density.to_bits()
                != configuration.transport.represented_density.to_bits()
        {
            return Err(RecordingError::Invalid("configured viscous domain"));
        }
        let interval_pressure = if let Some(r) = &receipt {
            let p = step
                .field("interval_pressure")
                .ok_or(RecordingError::Invalid("configured field inventory"))?;
            let start = p
                .evaluation_time
                .ok_or(RecordingError::Invalid("configured interval"))?;
            let dt = step
                .dt
                .ok_or(RecordingError::Invalid("configured interval"))?;
            if !start.is_finite()
                || start < 0.0
                || start >= step.time
                || (start + dt).to_bits() != step.time.to_bits()
                || (step.generation == 1 && start.to_bits() != 0.0_f64.to_bits())
            {
                return Err(RecordingError::Invalid("configured interval"));
            }
            validate_receipt(r, &configuration, &grid, step, start)?;
            let values = f64_values(required(
                step,
                "interval_pressure",
                "f64",
                "Pa",
                CELL,
                PRESSURE,
                grid.counts(),
                Some(start),
            )?)?;
            if values[0] != 0.0 {
                return Err(RecordingError::Invalid("configured pressure gauge"));
            }
            Some((start, values))
        } else {
            None
        };
        let z = faces.pop().unwrap();
        let y = faces.pop().unwrap();
        let x = faces.pop().unwrap();
        Ok(Self {
            grid,
            configuration,
            generation: step.generation,
            time: step.time,
            dt: step.dt,
            carrier_stamp,
            fraction_stamp,
            fraction_time,
            x,
            y,
            z,
            tracer,
            fraction,
            interval_pressure,
            receipt,
        })
    }
}

/// Recompute prior accepted-pressure hashes where the previous publication is
/// retained. RHS hashes are custody observations, not reconstructed RHS proofs.
pub fn verify_configured_recording(
    reader: &RecordingReader,
) -> Result<ConfiguredHistoryCoverage, RecordingError> {
    reader.verify()?;
    // Empty archives and unrecorded terminal tails still have a typed contract.
    let cfg = configuration(reader.run())?;
    if reader
        .status()?
        .accepted_generation
        .is_some_and(|g| g > cfg.max_steps as u64)
    {
        return Err(RecordingError::Invalid("configured terminal quota"));
    }
    let mut coverage = ConfiguredHistoryCoverage::default();
    // Only a digest and lineage survive each iteration, not a second field set.
    let mut previous: Option<(u64, f64, [u8; 32])> = None;
    let mut previous_time: Option<f64> = None;
    let mut history: Option<(f64, usize, bool)> = None;
    for ordinal in 0..reader.count()? {
        let step = reader.select(ordinal)?;
        let current = RecordedConfiguredSession::decode(reader.run(), &step)?;
        if let Some((start, _)) = &current.interval_pressure
            && previous_time.is_some_and(|time| time.to_bits() != start.to_bits())
        {
            return Err(RecordingError::Invalid("configured interval continuity"));
        }
        if let Some(r) = &current.receipt {
            let prior_history = history.filter(|h| h.0.to_bits() == r.dynamic_viscosity.to_bits());
            if r.binding.is_some()
                && r.context.is_none()
                && (current.generation == 1 || prior_history.is_some())
            {
                return Err(RecordingError::Invalid(
                    "configured missing available context",
                ));
            }
            if let Some(c) = r.context {
                if c.native_prefix_steps != prior_history.map_or(0, |h| h.1) {
                    return Err(RecordingError::Invalid("configured native prefix"));
                }
                if let Some(generation) = c.prior_generation {
                    match previous {
                        Some((prior_generation, time, hash)) => {
                            if prior_generation != generation
                                || c.prior_interval_end.map(f64::to_bits) != Some(time.to_bits())
                                || c.prior_pressure_sha256 != Some(hash)
                                || prior_history.is_none()
                            {
                                return Err(RecordingError::Invalid(
                                    "configured prior pressure link",
                                ));
                            }
                            coverage.verified_prior_links += 1;
                        }
                        None => coverage.unavailable_prior_links += 1,
                    }
                }
            }
            let native = matches!(r.selected, RecordedPressureSelection::Native(_));
            let all_native = native && prior_history.is_none_or(|h| h.2);
            let prefix = if all_native {
                prior_history.map_or(0, |h| h.1).saturating_add(1)
            } else {
                prior_history.map_or(0, |h| h.1)
            };
            history = Some((r.dynamic_viscosity, prefix, all_native));
        }
        previous = current.interval_pressure.as_ref().map(|(_, values)| {
            let mut hash = Sha256::new();
            for value in values {
                hash.update(value.to_le_bytes());
            }
            (current.generation, current.time, hash.finalize().into())
        });
        previous_time = Some(current.time);
    }
    Ok(coverage)
}

fn validate_receipt(
    r: &RecordedConfiguredReceipt,
    cfg: &SessionConfiguration,
    grid: &GridGeometry,
    step: &RecordedStep,
    start: f64,
) -> Result<(), RecordingError> {
    if !r.dynamic_viscosity.is_finite() || r.dynamic_viscosity < 0.0 {
        return Err(RecordingError::Invalid("configured viscosity"));
    }
    validate_pressure(r.accepted, true)?;
    if r.accepted.predicted_divergence_max > cfg.transport.carrier.pressure.divergence_limit
        || r.accepted.iterations > cfg.transport.carrier.pressure.max_iterations
    {
        return Err(RecordingError::Invalid(
            "configured accepted pressure gates",
        ));
    }
    if let Some(p) = r.candidate {
        validate_pressure(p, false)?;
        if p.iterations != 0 {
            return Err(RecordingError::Invalid("configured candidate diagnostics"));
        }
    }
    if let Some(identity) = &r.binding {
        identity
            .validate()
            .map_err(|_| RecordingError::Invalid("configured binding"))?;
    }
    if let Some(m) = &r.attempted {
        validate_metadata(m, grid)?;
    }
    let attempted = r.attempted.is_some();
    if !attempted
        && (r.binding.is_some()
            || r.context.is_some()
            || r.prediction_called
            || r.rejection.is_some()
            || r.candidate.is_some())
        || r.context.is_some() && r.binding.is_none()
        || r.prediction_called && (r.context.is_none() || r.binding.is_none())
        || r.candidate.is_some() && !r.prediction_called
        || matches!(r.rejection, Some(RecordedProviderRejection::Cancelled))
    {
        return Err(RecordingError::Invalid("configured receipt association"));
    }
    match r.selected {
        RecordedPressureSelection::Native(method) => {
            if method != cfg.implementation || attempted != r.rejection.is_some() {
                return Err(RecordingError::Invalid("configured native selection"));
            }
        }
        RecordedPressureSelection::Configured => {
            if !attempted
                || !r.prediction_called
                || r.binding.is_none()
                || r.context.is_none()
                || r.rejection.is_some()
                || r.candidate
                    .is_none_or(|candidate| pressure_bits(candidate) != pressure_bits(r.accepted))
            {
                return Err(RecordingError::Invalid("configured candidate selection"));
            }
        }
    }
    if r.candidate.is_some()
        && r.rejection.is_some()
        && !matches!(r.rejection, Some(RecordedProviderRejection::QualityGate))
    {
        return Err(RecordingError::Invalid("configured candidate diagnostics"));
    }
    if attempted
        && r.context.is_none()
        && !matches!(r.rejection, Some(RecordedProviderRejection::Unsupported(_)))
    {
        return Err(RecordingError::Invalid("configured unavailable context"));
    }
    if let Some(c) = r.context {
        let prior = c.prior_generation.is_some();
        if c.generation != step.generation
            || c.interval_start.to_bits() != start.to_bits()
            || c.interval_end.to_bits() != step.time.to_bits()
            || Some(c.dt.to_bits()) != step.dt.map(f64::to_bits)
            || c.native_prefix_steps as u64 >= c.generation
            || prior != c.prior_interval_end.is_some()
            || prior != c.prior_pressure_sha256.is_some()
            || c.prior_generation
                .is_some_and(|g| g.checked_add(1) != Some(c.generation))
            || c.prior_interval_end
                .is_some_and(|t| t.to_bits() != start.to_bits())
            || (c.generation > 1 && !prior)
            || (c.generation == 1 && prior)
        {
            return Err(RecordingError::Invalid("configured context association"));
        }
    }
    Ok(())
}
fn pressure_bits(p: PressureReport) -> [u64; 4] {
    [
        p.iterations as u64,
        p.true_residual_l2.to_bits(),
        p.true_residual_max.to_bits(),
        p.predicted_divergence_max.to_bits(),
    ]
}
fn validate_pressure(p: PressureReport, accepted: bool) -> Result<(), RecordingError> {
    if [
        p.true_residual_l2,
        p.true_residual_max,
        p.predicted_divergence_max,
    ]
    .into_iter()
    .any(|v| (accepted && !v.is_finite()) || (v.is_finite() && v < 0.0))
    {
        return Err(RecordingError::Invalid("configured pressure diagnostics"));
    }
    Ok(())
}
// Bind each publication to its actual Session identity and immutable source
// configuration/geometry. Caller provenance is intentionally not an identity.
fn source_binding(run: &RecordingRun) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(run.identity.run_id.to_le_bytes());
    hash_text(&mut hash, &run.identity.source_revision);
    hash_text(&mut hash, &run.solver);
    hash_text(&mut hash, &run.configuration);
    for n in run.counts {
        hash.update(n.to_le_bytes());
    }
    for value in run.spacing.into_iter().chain(run.origin) {
        hash.update(value.to_le_bytes());
    }
    hash.finalize().into()
}

fn configuration(run: &RecordingRun) -> Result<SessionConfiguration, RecordingError> {
    if run.solver != PROFILE {
        return Err(RecordingError::Unsupported("configured session profile"));
    }
    let mut d = Decoder::text(&run.configuration, CONFIG)?;
    let implementation = method(d.u8()?)?;
    let density = d.f64()?;
    let memory_limit = d.usize()?;
    let pressure = PressureSettings {
        relative_residual: d.f64()?,
        absolute_residual: d.f64()?,
        divergence_limit: d.f64()?,
        max_iterations: d.usize()?,
    };
    let actual_divergence_limit = d.f64()?;
    let max_courant = d.f64()?;
    let represented_density = d.f64()?;
    let volume_stamp = VolumeStamp {
        id: d.u64()?,
        version: d.u64()?,
    };
    let carrier_id = d.u64()?;
    let boundary_workspace_limit = d.usize()?;
    let max_steps = d.usize()?;
    d.end()?;
    if !density.is_finite()
        || density <= 0.0
        || !represented_density.is_finite()
        || represented_density <= 0.0
        || memory_limit == 0
        || boundary_workspace_limit == 0
        || !max_courant.is_finite()
        || max_courant <= 0.0
        || max_courant > 1.0
        || [
            pressure.relative_residual,
            pressure.absolute_residual,
            pressure.divergence_limit,
            actual_divergence_limit,
        ]
        .into_iter()
        .any(|v| !v.is_finite() || v < 0.0)
    {
        return Err(RecordingError::Invalid(
            "configured constructor configuration",
        ));
    }
    Ok(SessionConfiguration {
        transport: LiquidTransportConfig {
            carrier: SimulationConfig {
                density,
                memory_limit,
                pressure,
                actual_divergence_limit,
                max_courant,
            },
            represented_density,
            volume_stamp,
            carrier_id,
        },
        implementation,
        boundary_workspace_limit,
        max_steps,
    })
}
fn put_configuration(e: &mut Encoder, c: &SessionConfiguration) {
    let t = c.transport;
    let s = t.carrier;
    e.u8(method_id(c.implementation));
    e.f64(s.density);
    e.usize(s.memory_limit);
    e.f64(s.pressure.relative_residual);
    e.f64(s.pressure.absolute_residual);
    e.f64(s.pressure.divergence_limit);
    e.usize(s.pressure.max_iterations);
    e.f64(s.actual_divergence_limit);
    e.f64(s.max_courant);
    e.f64(t.represented_density);
    e.u64(t.volume_stamp.id);
    e.u64(t.volume_stamp.version);
    e.u64(t.carrier_id);
    e.usize(c.boundary_workspace_limit);
    e.usize(c.max_steps);
}
fn method_id(m: PressureImplementation) -> u8 {
    match m {
        PressureImplementation::JacobiPcgV1 => 0,
        PressureImplementation::SymmetricGaussSeidelPcgV1 => 1,
    }
}
fn method(id: u8) -> Result<PressureImplementation, RecordingError> {
    match id {
        0 => Ok(PressureImplementation::JacobiPcgV1),
        1 => Ok(PressureImplementation::SymmetricGaussSeidelPcgV1),
        _ => Err(RecordingError::Unsupported("configured native method")),
    }
}
fn owned_receipt(
    r: &ConfiguredProviderReport,
) -> Result<RecordedConfiguredReceipt, RecordingError> {
    let selected = if r.pressure.selected == ROLE {
        RecordedPressureSelection::Configured
    } else {
        RecordedPressureSelection::Native(
            PressureImplementation::from_id(r.pressure.selected)
                .ok_or(RecordingError::Unsupported("configured selected role"))?,
        )
    };
    let attempted = r
        .pressure
        .attempted
        .as_ref()
        .map(owned_metadata)
        .transpose()?;
    let rejection = r.pressure.rejection.as_ref().map(|r| match r {
        ProviderRejection::Unsupported(s) => RecordedProviderRejection::Unsupported((*s).into()),
        ProviderRejection::Execution(s) => RecordedProviderRejection::Execution((*s).into()),
        ProviderRejection::NonFiniteOutput => RecordedProviderRejection::NonFiniteOutput,
        ProviderRejection::NonZeroGauge => RecordedProviderRejection::NonZeroGauge,
        ProviderRejection::QualityGate => RecordedProviderRejection::QualityGate,
        ProviderRejection::Cancelled => RecordedProviderRejection::Cancelled,
    });
    Ok(RecordedConfiguredReceipt {
        binding: r.binding.as_deref().cloned(),
        context: r.context,
        dynamic_viscosity: r.dynamic_viscosity,
        prediction_called: r.prediction_called,
        selected,
        attempted,
        rejection,
        candidate: r.pressure.candidate,
        accepted: r.pressure.accepted,
        total: r.pressure.total,
        inference: r.pressure.inference,
        qualification: r.pressure.qualification,
        fallback: r.pressure.fallback,
    })
}
fn owned_metadata(m: &SolverMetadata) -> Result<RecordedSolverMetadata, RecordingError> {
    if m.provenance.is_some() {
        return Err(RecordingError::Unsupported("configured role provenance"));
    }
    let tensor = |m: &crate::solver_provider::TensorMetadata| RecordedTensorMetadata {
        shape_xyz: m.shape_xyz,
        layout: m.layout.into(),
        scalar: m.scalar.into(),
        units: m.units.into(),
    };
    Ok(RecordedSolverMetadata {
        id: m.id.into(),
        version: m.version.into(),
        stage: m.stage.into(),
        input: tensor(&m.input),
        output: tensor(&m.output),
        domain: m.domain.into(),
    })
}
fn validate_metadata(
    m: &RecordedSolverMetadata,
    grid: &GridGeometry,
) -> Result<(), RecordingError> {
    if m.id != ROLE
        || m.version != "1"
        || m.stage != crate::solver_provider::context::STAGE_CONTRACT
        || m.domain != DOMAIN
        || m.input.shape_xyz != grid.counts()
        || m.output.shape_xyz != grid.counts()
        || m.input.layout != "x_fast"
        || m.output.layout != "x_fast"
        || m.input.scalar != "f64"
        || m.output.scalar != "f64"
        || m.input.units != "m3/s2"
        || m.output.units != "Pa"
    {
        return Err(RecordingError::Invalid("configured attempted role"));
    }
    Ok(())
}

fn put_receipt(e: &mut Encoder, r: &RecordedConfiguredReceipt) -> Result<(), RecordingError> {
    e.boolean(r.binding.is_some());
    if let Some(b) = &r.binding {
        e.string(&b.id)?;
        e.string(&b.version)?;
        e.0.extend_from_slice(&b.manifest_sha256);
        e.boolean(b.checkpoint_sha256.is_some());
        if let Some(v) = b.checkpoint_sha256 {
            e.0.extend_from_slice(&v);
        }
        e.boolean(b.training_run_id.is_some());
        if let Some(v) = &b.training_run_id {
            e.string(v)?;
        }
        e.string(&b.runtime)?;
        e.string(&b.source_revision)?;
    }
    e.boolean(r.context.is_some());
    if let Some(c) = r.context {
        e.u64(c.generation);
        e.f64(c.interval_start);
        e.f64(c.interval_end);
        e.f64(c.dt);
        e.usize(c.native_prefix_steps);
        e.boolean(c.prior_generation.is_some());
        if let Some(v) = c.prior_generation {
            e.u64(v);
            e.f64(
                c.prior_interval_end
                    .ok_or(RecordingError::Invalid("configured prior"))?,
            );
            e.0.extend_from_slice(
                &c.prior_pressure_sha256
                    .ok_or(RecordingError::Invalid("configured prior"))?,
            );
        }
        e.0.extend_from_slice(&c.rhs_sha256);
    }
    e.f64(r.dynamic_viscosity);
    e.boolean(r.prediction_called);
    e.u8(match r.selected {
        RecordedPressureSelection::Native(m) => method_id(m),
        RecordedPressureSelection::Configured => 2,
    });
    e.boolean(r.attempted.is_some());
    if let Some(m) = &r.attempted {
        e.string(&m.id)?;
        e.string(&m.version)?;
        e.string(&m.stage)?;
        for t in [&m.input, &m.output] {
            for n in t.shape_xyz {
                e.usize(n);
            }
            e.string(&t.layout)?;
            e.string(&t.scalar)?;
            e.string(&t.units)?;
        }
        e.string(&m.domain)?;
    }
    match &r.rejection {
        None => e.u8(0),
        Some(RecordedProviderRejection::Unsupported(s)) => {
            e.u8(1);
            e.string(s)?;
        }
        Some(RecordedProviderRejection::Execution(s)) => {
            e.u8(2);
            e.string(s)?;
        }
        Some(RecordedProviderRejection::NonFiniteOutput) => e.u8(3),
        Some(RecordedProviderRejection::NonZeroGauge) => e.u8(4),
        Some(RecordedProviderRejection::QualityGate) => e.u8(5),
        Some(RecordedProviderRejection::Cancelled) => e.u8(6),
    }
    e.boolean(r.candidate.is_some());
    if let Some(p) = r.candidate {
        put_pressure(e, p);
    }
    put_pressure(e, r.accepted);
    for duration in [r.total, r.inference, r.qualification, r.fallback] {
        e.u64(duration.as_secs());
        e.u32(duration.subsec_nanos());
    }
    Ok(())
}
fn get_receipt(d: &mut Decoder) -> Result<RecordedConfiguredReceipt, RecordingError> {
    let binding = if d.boolean()? {
        Some(ContextProviderIdentity {
            id: d.string()?,
            version: d.string()?,
            manifest_sha256: d.array()?,
            checkpoint_sha256: if d.boolean()? { Some(d.array()?) } else { None },
            training_run_id: if d.boolean()? {
                Some(d.string()?)
            } else {
                None
            },
            runtime: d.string()?,
            source_revision: d.string()?,
        })
    } else {
        None
    };
    let context = if d.boolean()? {
        let generation = d.u64()?;
        let interval_start = d.f64()?;
        let interval_end = d.f64()?;
        let dt = d.f64()?;
        let native_prefix_steps = d.usize()?;
        let (prior_generation, prior_interval_end, prior_pressure_sha256) = if d.boolean()? {
            (Some(d.u64()?), Some(d.f64()?), Some(d.array()?))
        } else {
            (None, None, None)
        };
        Some(AcceptedPressureContextStamp {
            generation,
            interval_start,
            interval_end,
            dt,
            native_prefix_steps,
            prior_generation,
            prior_interval_end,
            prior_pressure_sha256,
            rhs_sha256: d.array()?,
        })
    } else {
        None
    };
    let dynamic_viscosity = d.f64()?;
    let prediction_called = d.boolean()?;
    let selected = match d.u8()? {
        2 => RecordedPressureSelection::Configured,
        id => RecordedPressureSelection::Native(method(id)?),
    };
    let attempted = if d.boolean()? {
        let id = d.string()?;
        let version = d.string()?;
        let stage = d.string()?;
        let mut tensor = || -> Result<RecordedTensorMetadata, RecordingError> {
            Ok(RecordedTensorMetadata {
                shape_xyz: [d.usize()?, d.usize()?, d.usize()?],
                layout: d.string()?,
                scalar: d.string()?,
                units: d.string()?,
            })
        };
        Some(RecordedSolverMetadata {
            id,
            version,
            stage,
            input: tensor()?,
            output: tensor()?,
            domain: d.string()?,
        })
    } else {
        None
    };
    let rejection = match d.u8()? {
        0 => None,
        1 => Some(RecordedProviderRejection::Unsupported(d.string()?)),
        2 => Some(RecordedProviderRejection::Execution(d.string()?)),
        3 => Some(RecordedProviderRejection::NonFiniteOutput),
        4 => Some(RecordedProviderRejection::NonZeroGauge),
        5 => Some(RecordedProviderRejection::QualityGate),
        6 => Some(RecordedProviderRejection::Cancelled),
        _ => return Err(RecordingError::Unsupported("configured rejection code")),
    };
    let candidate = if d.boolean()? {
        Some(get_pressure(d)?)
    } else {
        None
    };
    let accepted = get_pressure(d)?;
    let mut duration = || -> Result<Duration, RecordingError> {
        let seconds = d.u64()?;
        let nanos = d.u32()?;
        if nanos >= 1_000_000_000 {
            return Err(RecordingError::Invalid("configured duration"));
        }
        Ok(Duration::new(seconds, nanos))
    };
    Ok(RecordedConfiguredReceipt {
        binding,
        context,
        dynamic_viscosity,
        prediction_called,
        selected,
        attempted,
        rejection,
        candidate,
        accepted,
        total: duration()?,
        inference: duration()?,
        qualification: duration()?,
        fallback: duration()?,
    })
}
fn put_pressure(e: &mut Encoder, p: PressureReport) {
    for n in pressure_bits(p) {
        e.u64(n);
    }
}
fn get_pressure(d: &mut Decoder) -> Result<PressureReport, RecordingError> {
    Ok(PressureReport {
        iterations: d.usize()?,
        true_residual_l2: d.f64()?,
        true_residual_max: d.f64()?,
        predicted_divergence_max: d.f64()?,
    })
}

#[derive(Default)]
struct Encoder(Vec<u8>);
impl Encoder {
    fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    fn boolean(&mut self, v: bool) {
        self.u8(u8::from(v));
    }
    fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn usize(&mut self, v: usize) {
        self.u64(v as u64);
    }
    fn f64(&mut self, v: f64) {
        self.u64(v.to_bits());
    }
    fn string(&mut self, v: &str) -> Result<(), RecordingError> {
        bounded(v)?;
        self.u32(v.len() as u32);
        self.0.extend_from_slice(v.as_bytes());
        Ok(())
    }
    fn text(self, prefix: &str) -> Result<String, RecordingError> {
        if self.0.len() > (MAX_TEXT - prefix.len()) / 2 {
            return Err(RecordingError::Limit("configured receipt text"));
        }
        let mut out = String::with_capacity(prefix.len() + self.0.len() * 2);
        out.push_str(prefix);
        for b in self.0 {
            use std::fmt::Write;
            write!(&mut out, "{b:02x}").unwrap();
        }
        Ok(out)
    }
}
struct Decoder {
    data: Vec<u8>,
    at: usize,
}
impl Decoder {
    fn text(value: &str, prefix: &str) -> Result<Self, RecordingError> {
        if value.len() > MAX_TEXT {
            return Err(RecordingError::Limit("configured metadata"));
        }
        let text = value
            .strip_prefix(prefix)
            .ok_or(RecordingError::Unsupported("configured codec version"))?;
        if text.len() % 2 != 0 {
            return Err(RecordingError::Invalid("configured encoding"));
        }
        let mut data = Vec::with_capacity(text.len() / 2);
        for pair in text.as_bytes().chunks_exact(2) {
            let digit = |v: u8| match v {
                b'0'..=b'9' => Some(v - b'0'),
                b'a'..=b'f' => Some(v - b'a' + 10),
                _ => None,
            };
            data.push(
                digit(pair[0])
                    .zip(digit(pair[1]))
                    .map(|(a, b)| (a << 4) | b)
                    .ok_or(RecordingError::Invalid("configured encoding"))?,
            );
        }
        Ok(Self { data, at: 0 })
    }
    fn array<const N: usize>(&mut self) -> Result<[u8; N], RecordingError> {
        let end = self
            .at
            .checked_add(N)
            .ok_or(RecordingError::Invalid("configured encoding"))?;
        let value = self
            .data
            .get(self.at..end)
            .ok_or(RecordingError::Invalid("configured encoding"))?
            .try_into()
            .unwrap();
        self.at = end;
        Ok(value)
    }
    fn u8(&mut self) -> Result<u8, RecordingError> {
        Ok(self.array::<1>()?[0])
    }
    fn boolean(&mut self) -> Result<bool, RecordingError> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(RecordingError::Invalid("configured boolean")),
        }
    }
    fn u32(&mut self) -> Result<u32, RecordingError> {
        Ok(u32::from_le_bytes(self.array()?))
    }
    fn u64(&mut self) -> Result<u64, RecordingError> {
        Ok(u64::from_le_bytes(self.array()?))
    }
    fn usize(&mut self) -> Result<usize, RecordingError> {
        usize::try_from(self.u64()?).map_err(|_| RecordingError::Limit("configured integer"))
    }
    fn f64(&mut self) -> Result<f64, RecordingError> {
        Ok(f64::from_bits(self.u64()?))
    }
    fn string(&mut self) -> Result<String, RecordingError> {
        let len = self.u32()? as usize;
        if len > 1000 {
            return Err(RecordingError::Invalid("configured text"));
        }
        let end = self
            .at
            .checked_add(len)
            .ok_or(RecordingError::Invalid("configured text"))?;
        let text = std::str::from_utf8(
            self.data
                .get(self.at..end)
                .ok_or(RecordingError::Invalid("configured text"))?,
        )
        .map_err(|_| RecordingError::Invalid("configured text"))?;
        bounded(text)?;
        self.at = end;
        Ok(text.into())
    }
    fn end(self) -> Result<(), RecordingError> {
        if self.at != self.data.len() {
            Err(RecordingError::Invalid("configured trailing metadata"))
        } else {
            Ok(())
        }
    }
}
fn bounded(s: &str) -> Result<(), RecordingError> {
    if s.len() > 1000 {
        Err(RecordingError::Invalid("configured text"))
    } else {
        Ok(())
    }
}
fn bytes<const N: usize>(
    values: impl Iterator<Item = [u8; N]>,
    len: usize,
    width: usize,
) -> Result<Vec<u8>, RecordingError> {
    let size = len
        .checked_mul(width)
        .ok_or(RecordingError::Limit("configured field"))?;
    let mut data = Vec::new();
    data.try_reserve_exact(size)
        .map_err(|_| RecordingError::Limit("allocation"))?;
    for value in values {
        data.extend_from_slice(&value);
    }
    Ok(data)
}
#[allow(clippy::too_many_arguments)]
fn field(
    name: &str,
    units: &str,
    association: &str,
    stage: &str,
    time: Option<f64>,
    dimensions: [usize; 3],
    scalar: &str,
    data: Vec<u8>,
) -> Result<RecordedField, RecordingError> {
    let f = RecordedField {
        name: name.into(),
        units: units.into(),
        association: association.into(),
        stage: stage.into(),
        evaluation_time: time,
        dimensions: dimensions.map(|n| n as u64),
        components: 1,
        scalar_type: scalar.into(),
        data,
    };
    f.validate()?;
    Ok(f)
}
#[allow(clippy::too_many_arguments)]
fn required<'a>(
    step: &'a RecordedStep,
    name: &str,
    scalar: &str,
    units: &str,
    association: &str,
    stage: &str,
    dimensions: [usize; 3],
    time: Option<f64>,
) -> Result<&'a RecordedField, RecordingError> {
    let f = step
        .field(name)
        .ok_or(RecordingError::Invalid("configured field inventory"))?;
    if f.scalar_type != scalar
        || f.units != units
        || f.association != association
        || f.stage != stage
        || f.dimensions != dimensions.map(|n| n as u64)
        || f.components != 1
        || f.evaluation_time.map(f64::to_bits) != time.map(f64::to_bits)
    {
        return Err(RecordingError::Invalid("configured field descriptor"));
    }
    f.validate()?;
    Ok(f)
}
fn f32_values(f: &RecordedField) -> Result<Vec<f32>, RecordingError> {
    f.validate()?;
    let mut values = Vec::new();
    values
        .try_reserve_exact(f.data.len() / 4)
        .map_err(|_| RecordingError::Limit("allocation"))?;
    values.extend(
        f.data
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes(b.try_into().unwrap())),
    );
    Ok(values)
}
fn f64_values(f: &RecordedField) -> Result<Vec<f64>, RecordingError> {
    f.validate()?;
    let mut values = Vec::new();
    values
        .try_reserve_exact(f.data.len() / 8)
        .map_err(|_| RecordingError::Limit("allocation"))?;
    values.extend(
        f.data
            .chunks_exact(8)
            .map(|b| f64::from_le_bytes(b.try_into().unwrap())),
    );
    Ok(values)
}
