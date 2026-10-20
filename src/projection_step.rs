//! Closed native projection production and attempt evidence. This opt-in path
//! invokes no caller provider, process, model or stateful replacement.
use crate::solver_provider::composition::projection::{
    ProjectionCandidateIdentity, ProjectionCheckpoint, ProjectionDomain, ProjectionInput,
    ProjectionRejection, ProjectionRequest, ProjectionSelection, ProjectionWorkspace,
    SuppliedProjection,
};
use crate::solver_provider::{PressureRequest, ProviderReport, ProviderWorkspace};
use crate::{
    Axis, GridGeometry, LiquidStepReport, LiquidStepStage, PressureImplementation,
    PressureOperator, PressureReport, PressureSettings, SessionConfiguration, SessionError,
    SessionIdentity, SimulationConfig, SimulationError,
};
use std::{
    cell::RefCell,
    fmt,
    sync::Arc,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionIntent {
    BuiltinNativeV1,
}
impl ProjectionIntent {
    pub fn validate_configuration(self, c: &SessionConfiguration) -> Result<(), &'static str> {
        let config = c.transport.carrier;
        if config.memory_limit == 0
            || c.boundary_workspace_limit == 0
            || !config.density.is_finite()
            || config.density <= 0.0
            || !c.transport.represented_density.is_finite()
            || c.transport.represented_density <= 0.0
            || !config.max_courant.is_finite()
            || config.max_courant <= 0.0
            || config.pressure.max_iterations == 0
            || [
                config.pressure.relative_residual,
                config.pressure.absolute_residual,
                config.pressure.divergence_limit,
                config.actual_divergence_limit,
            ]
            .into_iter()
            .any(|v| !v.is_finite() || v < 0.0)
        {
            return Err("invalid native projection configuration");
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionStepStage {
    Liquid(LiquidStepStage),
    ProducerPressurePoll,
    Qualification(ProjectionCheckpoint),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionAttemptStage {
    SessionGuard,
    CarrierPreparation,
    NativeProducer,
    HostQualification,
    CarrierTransport,
    VolumeTransport,
    BeforeCommit,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionFailureCode {
    Stopped,
    StepLimit,
    Deadline,
    IntentMismatch,
    WorkspaceMismatch,
    InvalidInput,
    NativePressure,
    HostQualification,
    Carrier,
    Volume,
    ReceiptAliased,
    GenerationExhausted,
    UnrepresentableMixedQualificationFallback,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectionAttemptOutcome {
    Published {
        generation: u64,
    },
    Refused {
        stage: ProjectionAttemptStage,
        code: ProjectionFailureCode,
    },
    Cancelled {
        stage: ProjectionAttemptStage,
    },
    Failed {
        stage: ProjectionAttemptStage,
        code: ProjectionFailureCode,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionFieldRejection {
    LengthMismatch,
    NonFiniteInput,
    NonZeroWallVelocity { axis: Axis, index: usize },
    ArithmeticFailure,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionPressureRejection {
    NonFiniteOutput,
    NonZeroGauge,
    QualityGate,
}
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectionRejectionFacts {
    Field(ProjectionFieldRejection),
    MomentumWitness { axis: Axis, index: usize },
    Pressure(ProjectionPressureRejection),
    ActualDivergence { maximum: f64, limit: f64 },
}
#[derive(Debug, Clone)]
pub struct NativeProjectionProducerFacts {
    pub implementation: PressureImplementation,
    pub pressure: PressureReport,
    /// Actual max(abs, rel*||host RHS||), computed with the original compensated
    /// accumulation for receipt validation only; it changes no solve criterion.
    pub residual_l2_limit: f64,
    /// Complete builtin call: RHS, native solve and copied-input correction.
    pub elapsed: Duration,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionWitnessOutcome {
    Accepted,
    Rejected(ProjectionPressureRejection),
}
#[derive(Debug, Clone)]
pub struct ProjectionPressureWitnessFacts {
    pub outcome: ProjectionWitnessOutcome,
    pub diagnostics: Option<PressureReport>,
}
#[derive(Debug, Clone)]
pub struct NativePressureSolveFacts {
    pub implementation: PressureImplementation,
    pub pressure: PressureReport,
    /// Existing dispatcher's measured native solve only.
    pub elapsed: Duration,
}
#[derive(Debug, Clone)]
pub struct ProjectionQualificationFacts {
    pub selection: ProjectionSelection,
    pub rejection: Option<ProjectionRejectionFacts>,
    pub witness: Option<ProjectionPressureWitnessFacts>,
    pub native_fallback: Option<NativePressureSolveFacts>,
    pub actual_divergence_max: f64,
    pub elapsed: Duration,
}
/// Exact clean native-only dispatcher facts. Construction checks that no
/// attempted-provider metadata is removed; decoding uses the closed method enum.
#[derive(Debug, Clone)]
pub struct NativePressureDispatchFacts {
    pub implementation: PressureImplementation,
    pub pressure: PressureReport,
    pub total: Duration,
    pub inference: Duration,
    pub qualification: Duration,
    pub fallback: Duration,
}
impl NativePressureDispatchFacts {
    fn capture(
        report: &ProviderReport,
        implementation: PressureImplementation,
    ) -> Result<Self, SimulationError> {
        if !clean_native(report, implementation) {
            return Err(SimulationError::ProjectionMixedQualificationFallback);
        }
        Ok(Self {
            implementation,
            pressure: report.accepted,
            total: report.total,
            inference: report.inference,
            qualification: report.qualification,
            fallback: report.fallback,
        })
    }
    pub fn report(&self) -> ProviderReport {
        ProviderReport {
            selected: self.implementation.id(),
            attempted: None,
            rejection: None,
            candidate: None,
            accepted: self.pressure,
            total: self.total,
            inference: self.inference,
            qualification: self.qualification,
            fallback: self.fallback,
        }
    }
}
#[derive(Debug, Clone)]
pub struct ProjectionAttempt {
    pub version: u32,
    pub intent: ProjectionIntent,
    pub source: SessionIdentity,
    pub attempt_id: u64,
    pub accepted_generation: u64,
    pub attempted_generation: Option<u64>,
    pub interval_start: f64,
    pub requested_dt: f64,
    pub actual_dt: Option<f64>,
    pub input_sha256: [u8; 32],
    pub controls_sha256: [u8; 32],
    pub provisional_input_id: Option<[u8; 32]>,
    pub producer: Option<NativeProjectionProducerFacts>,
    pub qualification: Option<ProjectionQualificationFacts>,
    pub selected_native: Option<NativePressureDispatchFacts>,
    pub outcome: ProjectionAttemptOutcome,
    pub output_sha256: Option<[u8; 32]>,
    pub diagnostic: String,
}
impl ProjectionAttempt {
    /// Intrinsic and constructor-linked checks. Archive history links and exact
    /// native input/output hashes are checked by the recording owner separately.
    pub fn validate(
        &self,
        configuration: &SessionConfiguration,
        grid: &GridGeometry,
    ) -> Result<(), &'static str> {
        self.intent.validate_configuration(configuration)?;
        if self.version != 1
            || !self.source.is_valid()
            || self.attempt_id == 0
            || self.diagnostic.len() > 960
            || self.diagnostic.chars().count() > 240
        {
            return Err("projection attempt header");
        }
        if !self.interval_start.is_finite()
            || self.interval_start < 0.0
            || self.attempted_generation != self.accepted_generation.checked_add(1)
        {
            return Err("projection attempt input position");
        }
        if let Some(dt) = self.actual_dt {
            if !dt.is_finite()
                || dt <= 0.0
                || !self.requested_dt.is_finite()
                || dt > self.requested_dt
                || self.interval_start + dt <= self.interval_start
                || !(self.interval_start + dt).is_finite()
            {
                return Err("projection actual interval");
            }
        }
        if self.provisional_input_id.is_some() != self.actual_dt.is_some()
            || self.producer.is_some() && self.provisional_input_id.is_none()
            || self.qualification.is_some() && self.producer.is_none()
            || self.selected_native.is_some() && self.qualification.is_none()
        {
            return Err("projection observation order");
        }
        let settings = configuration.transport.carrier.pressure;
        if let Some(p) = &self.producer {
            if p.implementation != configuration.implementation
                || !p.residual_l2_limit.is_finite()
                || p.residual_l2_limit < 0.0
                || !native_metrics(p.pressure, settings, p.residual_l2_limit)
            {
                return Err("projection producer metrics");
            }
        }
        if let Some(q) = &self.qualification {
            let producer = self.producer.as_ref().ok_or("projection producer absent")?;
            if !q.actual_divergence_max.is_finite()
                || q.actual_divergence_max < 0.0
                || q.actual_divergence_max > configuration.transport.carrier.actual_divergence_limit
            {
                return Err("projection qualified divergence");
            }
            if let Some(w) = &q.witness {
                if w.diagnostics.is_some_and(|p| p.iterations != 0) {
                    return Err("projection witness iterations");
                }
                match w.outcome {
                    ProjectionWitnessOutcome::Accepted => {
                        if w.diagnostics.is_none_or(|p| {
                            !native_metrics(p, settings, producer.residual_l2_limit)
                        }) {
                            return Err("projection accepted witness metrics");
                        }
                    }
                    ProjectionWitnessOutcome::Rejected(
                        ProjectionPressureRejection::QualityGate,
                    ) => {
                        if w.diagnostics.is_none() {
                            return Err("projection rejected quality diagnostics absent");
                        }
                    }
                    ProjectionWitnessOutcome::Rejected(_) => {
                        if w.diagnostics.is_some() {
                            return Err("projection rejected pressure diagnostics association");
                        }
                    }
                }
            }
            if let Some(f) = &q.native_fallback {
                if f.implementation != configuration.implementation
                    || !native_metrics(f.pressure, settings, producer.residual_l2_limit)
                {
                    return Err("projection fallback metrics");
                }
            }
            match q.selection {
                ProjectionSelection::SuppliedCandidate => {
                    let witness = q
                        .witness
                        .as_ref()
                        .ok_or("projection candidate witness absent")?;
                    if q.rejection.is_some()
                        || q.native_fallback.is_some()
                        || witness.outcome != ProjectionWitnessOutcome::Accepted
                        || witness
                            .diagnostics
                            .is_none_or(|p| metric_bits(p) != metric_bits(producer.pressure))
                    {
                        return Err("projection candidate association");
                    }
                }
                ProjectionSelection::NativeFallback => {
                    if q.native_fallback.is_none() {
                        return Err("projection fallback absent");
                    }
                    match q.rejection.as_ref().ok_or("projection fallback rejection absent")? {
                        ProjectionRejectionFacts::Field(field) => {
                            if q.witness.is_some() {return Err("projection field rejection witness");}
                            if let ProjectionFieldRejection::NonZeroWallVelocity{axis,index}=field {
                                if *index>=grid.face_len(*axis) || !wall_index(grid,*axis,*index) {return Err("projection wall index");}
                            }
                        }
                        ProjectionRejectionFacts::MomentumWitness{axis,index} => {
                            if q.witness.is_some() || *index>=grid.face_len(*axis) {return Err("projection momentum witness association");}
                        }
                        ProjectionRejectionFacts::Pressure(reason) => {
                            if q.witness.as_ref().is_none_or(|w|w.outcome!=ProjectionWitnessOutcome::Rejected(*reason))
                                || self.selected_native.is_some() || !matches!(self.outcome,ProjectionAttemptOutcome::Failed{
                                    stage:ProjectionAttemptStage::HostQualification,code:ProjectionFailureCode::UnrepresentableMixedQualificationFallback}) {
                                return Err("projection mixed fallback association");
                            }
                        }
                        ProjectionRejectionFacts::ActualDivergence{maximum,limit} => {
                            if !maximum.is_finite() || !limit.is_finite() || *maximum<=*limit
                                || limit.to_bits()!=configuration.transport.carrier.actual_divergence_limit.to_bits()
                                || q.witness.as_ref().is_none_or(|w|w.outcome!=ProjectionWitnessOutcome::Accepted) {
                                return Err("projection discarded witness association");
                            }
                        }
                    }
                }
                ProjectionSelection::NativeBaseline => {
                    return Err("projection builtin must be attempted");
                }
            }
        }
        if let Some(native) = &self.selected_native {
            let q = self
                .qualification
                .as_ref()
                .ok_or("projection native qualification absent")?;
            let p = self
                .producer
                .as_ref()
                .ok_or("projection native producer absent")?;
            let expected = if q.selection == ProjectionSelection::SuppliedCandidate {
                p.pressure
            } else {
                q.native_fallback
                    .as_ref()
                    .ok_or("projection native fallback absent")?
                    .pressure
            };
            if native.implementation != configuration.implementation
                || !native_metrics(native.pressure, settings, p.residual_l2_limit)
                || report_bits(native.pressure) != report_bits(expected)
                || native.inference != Duration::ZERO
                || native.qualification != Duration::ZERO
                || native.fallback > native.total
                || q.selection == ProjectionSelection::SuppliedCandidate && native.total > p.elapsed
                || q.selection == ProjectionSelection::NativeFallback && native.total > q.elapsed
                || q.native_fallback
                    .as_ref()
                    .is_some_and(|f| native.fallback != f.elapsed)
            {
                return Err("projection selected native report");
            }
        }
        let prefix = |stage: ProjectionAttemptStage| match stage {
            ProjectionAttemptStage::SessionGuard | ProjectionAttemptStage::CarrierPreparation => {
                self.actual_dt.is_none()
                    && self.producer.is_none()
                    && self.qualification.is_none()
                    && self.selected_native.is_none()
            }
            ProjectionAttemptStage::NativeProducer => {
                self.actual_dt.is_some()
                    && self.producer.is_none()
                    && self.qualification.is_none()
                    && self.selected_native.is_none()
            }
            ProjectionAttemptStage::HostQualification => {
                self.producer.is_some() && self.selected_native.is_none()
            }
            ProjectionAttemptStage::CarrierTransport
            | ProjectionAttemptStage::VolumeTransport
            | ProjectionAttemptStage::BeforeCommit => {
                self.qualification.is_some() && self.selected_native.is_some()
            }
        };
        match self.outcome {
            ProjectionAttemptOutcome::Published { generation } => {
                if Some(generation) != self.attempted_generation
                    || self.qualification.is_none()
                    || self.selected_native.is_none()
                    || self.output_sha256.is_none()
                    || !self.diagnostic.is_empty()
                {
                    return Err("projection publication association");
                }
            }
            ProjectionAttemptOutcome::Refused { stage, code } => {
                if self.output_sha256.is_some()
                    || stage != ProjectionAttemptStage::SessionGuard
                    || !prefix(stage)
                    || !matches!(
                        code,
                        ProjectionFailureCode::Stopped
                            | ProjectionFailureCode::StepLimit
                            | ProjectionFailureCode::IntentMismatch
                            | ProjectionFailureCode::WorkspaceMismatch
                            | ProjectionFailureCode::GenerationExhausted
                            | ProjectionFailureCode::ReceiptAliased
                    )
                {
                    return Err("projection refusal prefix");
                }
            }
            ProjectionAttemptOutcome::Cancelled { stage } => {
                if self.output_sha256.is_some()
                    || !prefix(stage)
                    || stage == ProjectionAttemptStage::HostQualification
                        && self.qualification.is_some()
                {
                    return Err("projection cancellation prefix");
                }
            }
            ProjectionAttemptOutcome::Failed { stage, code } => {
                if stage == ProjectionAttemptStage::HostQualification {
                    match code {
                        ProjectionFailureCode::UnrepresentableMixedQualificationFallback => {
                            if self.qualification.as_ref().is_none_or(|q| {
                                q.selection != ProjectionSelection::NativeFallback
                                    || !matches!(
                                        q.rejection,
                                        Some(ProjectionRejectionFacts::Pressure(_))
                                    )
                            }) {
                                return Err("projection mixed failure observations absent");
                            }
                        }
                        ProjectionFailureCode::HostQualification => {
                            if self.qualification.is_some() {
                                return Err("projection failed qualification has completed result");
                            }
                        }
                        _ => {}
                    }
                }
                let allowed = match stage {
                    ProjectionAttemptStage::SessionGuard => false,
                    ProjectionAttemptStage::CarrierPreparation
                    | ProjectionAttemptStage::CarrierTransport => matches!(
                        code,
                        ProjectionFailureCode::Carrier
                            | ProjectionFailureCode::InvalidInput
                            | ProjectionFailureCode::ReceiptAliased
                    ),
                    ProjectionAttemptStage::NativeProducer => matches!(
                        code,
                        ProjectionFailureCode::NativePressure
                            | ProjectionFailureCode::ReceiptAliased
                    ),
                    ProjectionAttemptStage::HostQualification => matches!(
                        code,
                        ProjectionFailureCode::HostQualification
                            | ProjectionFailureCode::UnrepresentableMixedQualificationFallback
                            | ProjectionFailureCode::ReceiptAliased
                    ),
                    ProjectionAttemptStage::VolumeTransport => matches!(
                        code,
                        ProjectionFailureCode::Volume | ProjectionFailureCode::ReceiptAliased
                    ),
                    ProjectionAttemptStage::BeforeCommit => matches!(
                        code,
                        ProjectionFailureCode::Carrier
                            | ProjectionFailureCode::InvalidInput
                            | ProjectionFailureCode::ReceiptAliased
                    ),
                };
                if !allowed || !prefix(stage) || self.output_sha256.is_some() {
                    return Err("projection failure prefix");
                }
            }
        }
        Ok(())
    }
}
fn metric_bits(p: PressureReport) -> [u64; 3] {
    [
        p.true_residual_l2.to_bits(),
        p.true_residual_max.to_bits(),
        p.predicted_divergence_max.to_bits(),
    ]
}
fn report_bits(p: PressureReport) -> [u64; 4] {
    [
        p.iterations as u64,
        p.true_residual_l2.to_bits(),
        p.true_residual_max.to_bits(),
        p.predicted_divergence_max.to_bits(),
    ]
}
fn wall_index(grid: &GridGeometry, axis: Axis, index: usize) -> bool {
    let shape = grid.face_counts(axis);
    let coordinate = [
        index % shape[0],
        (index / shape[0]) % shape[1],
        index / (shape[0] * shape[1]),
    ];
    coordinate[axis.index()] == 0 || coordinate[axis.index()] == grid.counts()[axis.index()]
}

fn finite_metrics(p: PressureReport) -> bool {
    [
        p.true_residual_l2,
        p.true_residual_max,
        p.predicted_divergence_max,
    ]
    .into_iter()
    .all(|v| v.is_finite() && v >= 0.0)
}
fn native_metrics(p: PressureReport, settings: PressureSettings, limit: f64) -> bool {
    finite_metrics(p)
        && p.iterations <= settings.max_iterations
        && p.true_residual_l2 <= limit
        && p.predicted_divergence_max <= settings.divergence_limit
}

#[derive(Debug)]
pub enum ProjectionStepError {
    Session(SessionError),
    IntentMismatch,
    WorkspaceMismatch,
    AttemptExhausted,
    ReceiptAliased,
    InvalidReceipt(&'static str),
    UnrepresentableMixedQualificationFallback,
}
impl fmt::Display for ProjectionStepError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "native projection step: {self:?}")
    }
}
impl std::error::Error for ProjectionStepError {}
pub struct ProjectionStepAccepted {
    pub report: LiquidStepReport,
    pub attempt: Arc<ProjectionAttempt>,
}
#[derive(Debug)]
pub struct ProjectionStepFailure {
    pub error: ProjectionStepError,
    pub attempt: Option<Arc<ProjectionAttempt>>,
}
impl fmt::Display for ProjectionStepFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error.fmt(f)
    }
}
impl std::error::Error for ProjectionStepFailure {}

pub(crate) struct ProjectionCapture {
    pub(crate) attempt: Arc<ProjectionAttempt>,
    pub(crate) stage: ProjectionAttemptStage,
}
impl ProjectionCapture {
    pub(crate) fn update(
        &mut self,
        f: impl FnOnce(&mut ProjectionAttempt),
    ) -> Result<(), SimulationError> {
        let attempt =
            Arc::get_mut(&mut self.attempt).ok_or(SimulationError::ProjectionReceiptAliased)?;
        f(attempt);
        Ok(())
    }
    pub(crate) fn seal(
        &mut self,
        state: crate::LiquidTransportView<'_>,
        configuration: &SessionConfiguration,
        grid: &GridGeometry,
    ) -> Result<(), SimulationError> {
        self.update(|a| {
            a.outcome = ProjectionAttemptOutcome::Published {
                generation: state.carrier.generation,
            };
            a.output_sha256 = Some(crate::workflow::input_digest(state));
            a.diagnostic.clear();
        })?;
        self.attempt
            .validate(configuration, grid)
            .map_err(SimulationError::ProjectionReceiptInvalid)
    }
}
pub(crate) trait ProjectionCancellation {
    fn poll(&self, stage: ProjectionStepStage) -> bool;
}
pub(crate) struct ProjectionControl<'a> {
    pub(crate) callback: RefCell<&'a mut dyn FnMut(ProjectionStepStage) -> bool>,
}
impl ProjectionCancellation for ProjectionControl<'_> {
    fn poll(&self, stage: ProjectionStepStage) -> bool {
        (self.callback.borrow_mut())(stage)
    }
}
pub(crate) struct ProjectionStepInput<'a> {
    pub(crate) workspace: &'a mut ProjectionStepWorkspace,
    pub(crate) capture: &'a mut ProjectionCapture,
    pub(crate) source: &'a SessionIdentity,
    pub(crate) configuration: &'a SessionConfiguration,
    pub(crate) control: &'a dyn ProjectionCancellation,
}

/// All managed arrays are aggregate-capped: native dispatcher/RHS, B1 workspace
/// and three producer velocity arrays. No accepted history is retained here.
pub struct ProjectionStepWorkspace {
    grid: GridGeometry,
    density: f64,
    implementation: PressureImplementation,
    producer: ProviderWorkspace,
    rhs: Vec<f64>,
    velocity: [Vec<f32>; 3],
    qualifier: ProjectionWorkspace,
    allocated: usize,
    #[cfg(test)]
    fault: ProjectionFault,
}
impl ProjectionStepWorkspace {
    pub fn new(
        grid: &GridGeometry,
        density: f64,
        limit: usize,
        implementation: PressureImplementation,
    ) -> Result<Self, SimulationError> {
        PressureOperator::new(grid, density)?;
        let mut remaining = limit;
        let rhs = allocate(grid.cell_len(), 0.0_f64, &mut remaining)?;
        let velocity = [
            allocate(grid.face_len(Axis::X), 0.0_f32, &mut remaining)?,
            allocate(grid.face_len(Axis::Y), 0.0_f32, &mut remaining)?,
            allocate(grid.face_len(Axis::Z), 0.0_f32, &mut remaining)?,
        ];
        let producer = ProviderWorkspace::new(grid, remaining, implementation)?;
        remaining = remaining
            .checked_sub(producer.allocated_bytes())
            .ok_or(SimulationError::AllocationFailed)?;
        let qualifier = ProjectionWorkspace::new(grid, remaining, implementation)
            .map_err(SimulationError::Projection)?;
        remaining = remaining
            .checked_sub(qualifier.allocated_bytes())
            .ok_or(SimulationError::AllocationFailed)?;
        Ok(Self {
            grid: grid.clone(),
            density,
            implementation,
            producer,
            rhs,
            velocity,
            qualifier,
            allocated: limit - remaining,
            #[cfg(test)]
            fault: ProjectionFault::None,
        })
    }
    #[cfg(test)]
    pub(crate) fn set_fault(&mut self, fault: ProjectionFault) {
        self.fault = fault;
    }
    #[cfg(test)]
    pub(crate) fn alias_before_work(&self) -> bool {
        matches!(self.fault, ProjectionFault::AliasBeforeWork)
    }
    #[cfg(test)]
    pub(crate) fn alias_before_seal(&self) -> bool {
        matches!(self.fault, ProjectionFault::AliasBeforeSeal)
    }
    pub fn allocated_bytes(&self) -> usize {
        self.allocated
    }
    pub(crate) fn matches(
        &self,
        grid: &GridGeometry,
        density: f64,
        implementation: PressureImplementation,
    ) -> bool {
        self.grid == *grid
            && self.density.to_bits() == density.to_bits()
            && self.implementation == implementation
    }
    pub(crate) fn pressure(&self) -> &[f64] {
        self.qualifier_pressure()
    }
    pub(crate) fn velocity(&self) -> [&[f32]; 3] {
        self.qualifier.velocity_for_integrated_step()
    }
    fn qualifier_pressure(&self) -> &[f64] {
        // The borrow returned by qualify is released only after copying every
        // qualified field; retain the exact final pressure in B1 scratch.
        self.qualifier.pressure_for_integrated_step()
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn project(
        &mut self,
        config: SimulationConfig,
        source: &SessionIdentity,
        generation: u64,
        start: f64,
        dt: f64,
        provisional: [&[f32]; 3],
        capture: &mut ProjectionCapture,
        mut cancel: impl FnMut(ProjectionStepStage) -> bool,
    ) -> Result<ProviderReport, SimulationError> {
        let request = ProjectionRequest::new(ProjectionInput {
            grid: &self.grid,
            density: self.density,
            interval_start: start,
            dt,
            source,
            generation,
            velocity: provisional,
            settings: config.pressure,
            actual_divergence_limit: config.actual_divergence_limit,
            domain: ProjectionDomain::SealedStationaryBox,
        })
        .map_err(SimulationError::Projection)?;
        capture.update(|a| {
            a.actual_dt = Some(dt);
            a.provisional_input_id = Some(*request.input_id().as_bytes());
        })?;
        capture.stage = ProjectionAttemptStage::NativeProducer;
        let begin = Instant::now();
        let operator = PressureOperator::new(&self.grid, self.density)?;
        operator.build_rhs(provisional, dt, &mut self.rhs)?;
        let residual_l2_limit = config
            .pressure
            .absolute_residual
            .max(config.pressure.relative_residual * compensated_norm(&self.rhs));
        let producer = self.producer.solve(
            PressureRequest {
                grid: &self.grid,
                density: self.density,
                dt,
                rhs: &self.rhs,
            },
            config.pressure,
            None,
            || cancel(ProjectionStepStage::ProducerPressurePoll),
        )?;
        for (output, input) in self.velocity.iter_mut().zip(provisional) {
            output.copy_from_slice(input);
        }
        let [x, y, z] = &mut self.velocity;
        operator.correct_candidate_in_place(self.producer.pressure(), dt, [x, y, z])?;
        capture.update(|a| {
            a.producer = Some(NativeProjectionProducerFacts {
                implementation: self.implementation,
                pressure: producer.accepted,
                residual_l2_limit,
                elapsed: begin.elapsed(),
            })
        })?;
        #[cfg(test)]
        self.fault.apply(&mut self.velocity);
        #[cfg(test)]
        let mut fault_pressure = [0.0_f64; 64];
        let candidate_pressure = self.producer.pressure();
        #[cfg(test)]
        let candidate_pressure = if matches!(
            self.fault,
            ProjectionFault::BadGauge | ProjectionFault::BadPressure
        ) {
            assert_eq!(self.grid.cell_len(), 64, "private fixed fault grid");
            if matches!(self.fault, ProjectionFault::BadGauge) {
                fault_pressure.fill(1.0);
            } else {
                fault_pressure[1] = 1e-4;
            }
            for (output, input) in self.velocity.iter_mut().zip(provisional) {
                output.copy_from_slice(input);
            }
            let [x, y, z] = &mut self.velocity;
            operator.correct_candidate_in_place(&fault_pressure, dt, [x, y, z])?;
            &fault_pressure[..]
        } else {
            candidate_pressure
        };
        let supplied = SuppliedProjection {
            input_id: request.input_id(),
            representation: request.representation(),
            identity: ProjectionCandidateIdentity {
                algorithm: self.implementation.id(),
                version: "1",
                source_revision: "rheon-builtin-native-projection-v1",
            },
            pressure: candidate_pressure,
            velocity: [&self.velocity[0], &self.velocity[1], &self.velocity[2]],
        };
        capture.stage = ProjectionAttemptStage::HostQualification;
        let begin = Instant::now();
        let qualified = self
            .qualifier
            .qualify(&request, Some(&supplied), |stage| {
                cancel(ProjectionStepStage::Qualification(stage))
            })
            .map_err(SimulationError::Projection)?;
        let report = qualified.report();
        let facts = qualification_facts(report, self.implementation, begin.elapsed())?;
        capture.update(|a| a.qualification = Some(facts))?;
        let selected = match report.selection {
            ProjectionSelection::SuppliedCandidate => producer,
            ProjectionSelection::NativeFallback
                if clean_native(&report.pressure, self.implementation) =>
            {
                report.pressure.clone()
            }
            _ => return Err(SimulationError::ProjectionMixedQualificationFallback),
        };
        let native = NativePressureDispatchFacts::capture(&selected, self.implementation)?;
        capture.update(|a| a.selected_native = Some(native))?;
        Ok(selected)
    }
}
fn clean_native(report: &ProviderReport, implementation: PressureImplementation) -> bool {
    report.selected == implementation.id()
        && report.attempted.is_none()
        && report.rejection.is_none()
        && report.candidate.is_none()
}
fn qualification_facts(
    report: &crate::solver_provider::composition::projection::ProjectionReport,
    implementation: PressureImplementation,
    elapsed: Duration,
) -> Result<ProjectionQualificationFacts, SimulationError> {
    let rejection = report.rejection.as_ref().map(rejection_facts).transpose()?;
    let witness_report = report
        .discarded_candidate_pressure
        .as_ref()
        .unwrap_or(&report.pressure);
    let witness = if witness_report.attempted.is_some() {
        let outcome = match witness_report.rejection.as_ref() {
            Some(reason) => ProjectionWitnessOutcome::Rejected(pressure_rejection(reason)?),
            None => ProjectionWitnessOutcome::Accepted,
        };
        Some(ProjectionPressureWitnessFacts {
            outcome,
            diagnostics: witness_report.candidate,
        })
    } else {
        None
    };
    let native_fallback = (report.selection == ProjectionSelection::NativeFallback).then(|| {
        NativePressureSolveFacts {
            implementation,
            pressure: report.pressure.accepted,
            elapsed: report.pressure.fallback,
        }
    });
    Ok(ProjectionQualificationFacts {
        selection: report.selection,
        rejection,
        witness,
        native_fallback,
        actual_divergence_max: report.actual_divergence_max,
        elapsed,
    })
}
fn pressure_rejection(
    reason: &crate::solver_provider::ProviderRejection,
) -> Result<ProjectionPressureRejection, SimulationError> {
    use crate::solver_provider::ProviderRejection as R;
    match reason {
        R::NonFiniteOutput => Ok(ProjectionPressureRejection::NonFiniteOutput),
        R::NonZeroGauge => Ok(ProjectionPressureRejection::NonZeroGauge),
        R::QualityGate => Ok(ProjectionPressureRejection::QualityGate),
        _ => Err(SimulationError::ProjectionReceiptInvalid(
            "unrepresentable pressure rejection",
        )),
    }
}
fn rejection_facts(
    reason: &ProjectionRejection,
) -> Result<ProjectionRejectionFacts, SimulationError> {
    use crate::OperatorError as O;
    Ok(match reason {
        ProjectionRejection::Field(error) => ProjectionRejectionFacts::Field(match error {
            O::LengthMismatch => ProjectionFieldRejection::LengthMismatch,
            O::NonFiniteInput => ProjectionFieldRejection::NonFiniteInput,
            O::NonZeroWallVelocity { axis, index } => {
                ProjectionFieldRejection::NonZeroWallVelocity {
                    axis: *axis,
                    index: *index,
                }
            }
            O::ArithmeticFailure => ProjectionFieldRejection::ArithmeticFailure,
            _ => {
                return Err(SimulationError::ProjectionReceiptInvalid(
                    "unrepresentable field rejection",
                ));
            }
        }),
        ProjectionRejection::MomentumWitness { axis, index } => {
            ProjectionRejectionFacts::MomentumWitness {
                axis: *axis,
                index: *index,
            }
        }
        ProjectionRejection::Pressure(reason) => {
            ProjectionRejectionFacts::Pressure(pressure_rejection(reason)?)
        }
        ProjectionRejection::ActualDivergence { maximum, limit } => {
            ProjectionRejectionFacts::ActualDivergence {
                maximum: *maximum,
                limit: *limit,
            }
        }
    })
}
fn compensated_norm(rhs: &[f64]) -> f64 {
    let mut sum = 0.0_f64;
    let mut correction = 0.0;
    for value in rhs.iter().map(|v| v * v) {
        let next = sum + value;
        correction += if sum.abs() >= value.abs() {
            (sum - next) + value
        } else {
            (value - next) + sum
        };
        sum = next;
    }
    (sum + correction).sqrt()
}
fn allocate<T: Clone>(
    len: usize,
    value: T,
    remaining: &mut usize,
) -> Result<Vec<T>, SimulationError> {
    let required = len
        .checked_mul(size_of::<T>())
        .ok_or(SimulationError::AllocationFailed)?;
    if required > *remaining {
        return Err(SimulationError::BufferLimit {
            required,
            limit: *remaining,
        });
    }
    let mut values = Vec::new();
    values
        .try_reserve_exact(len)
        .map_err(|_| SimulationError::AllocationFailed)?;
    let required = values
        .capacity()
        .checked_mul(size_of::<T>())
        .ok_or(SimulationError::AllocationFailed)?;
    if required > *remaining {
        return Err(SimulationError::BufferLimit {
            required,
            limit: *remaining,
        });
    }
    values.resize(len, value);
    *remaining -= required;
    Ok(values)
}

#[cfg(test)]
#[derive(Clone, Copy)]
pub(crate) enum ProjectionFault {
    None,
    WrongVelocity,
    NonFiniteVelocity,
    BadGauge,
    BadPressure,
    AliasBeforeWork,
    AliasBeforeSeal,
}
#[cfg(test)]
impl ProjectionFault {
    fn apply(self, velocity: &mut [Vec<f32>; 3]) {
        match self {
            Self::None => {}
            Self::WrongVelocity => velocity[0][1] = f32::from_bits(velocity[0][1].to_bits() ^ 1),
            Self::NonFiniteVelocity => velocity[0][1] = f32::NAN,
            Self::BadGauge | Self::BadPressure | Self::AliasBeforeWork | Self::AliasBeforeSeal => {}
        }
    }
}
