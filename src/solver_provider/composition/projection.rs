//! Stateless qualification of supplied sealed-MAC projection arrays.
//!
//! This is not a Session replacement or provider invocation API. Inputs and
//! candidate arrays are immutable caller data; returned arrays are disposable
//! scratch. Source tags are declared consistency facts, not authentication of
//! an accepted Session. Stateful candidates and external runtimes are absent.
use crate::solver_provider::{
    PressureProvider, PressureRequest, ProviderRejection, ProviderReport, ProviderWorkspace,
    SolverMetadata, TensorMetadata,
};
use crate::{
    Axis, GridGeometry, OperatorError, PressureError, PressureImplementation, PressureOperator,
    PressureSettings, SessionIdentity,
};
use sha2::{Digest, Sha256};
use std::fmt;

const CONTRACT: &str = "sealed-mac-projection-si-f32-faces-f64-cell-zero-gauge-v1";
const OPERATOR: &str = "sealed-constant-density-mac-cell-zero-gauge-v1";
const SUPPLIED_PRESSURE: &str = "supplied-projection-pressure-witness-v1";

/// Unsupported labels are explicit refusals, not additional geometry support.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionDomain {
    SealedStationaryBox,
    FreeSurface,
    EmbeddedObstacle,
    MovingWall,
}

pub struct ProjectionInput<'a> {
    pub grid: &'a GridGeometry,
    pub density: f64,
    pub interval_start: f64,
    pub dt: f64,
    pub source: &'a SessionIdentity,
    pub generation: u64,
    pub velocity: [&'a [f32]; 3],
    pub settings: PressureSettings,
    pub actual_divergence_limit: f64,
    pub domain: ProjectionDomain,
}

/// Digest of actual immutable inputs and the exact qualification contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectionInputId([u8; 32]);
impl ProjectionInputId {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

pub struct ProjectionRequest<'a> {
    input: ProjectionInput<'a>,
    input_id: ProjectionInputId,
}
impl<'a> ProjectionRequest<'a> {
    pub fn new(input: ProjectionInput<'a>) -> Result<Self, ProjectionError> {
        if input.domain != ProjectionDomain::SealedStationaryBox {
            return Err(ProjectionError::Refused(
                ProjectionRefusal::UnsupportedDomain(input.domain),
            ));
        }
        if !valid_text(&input.source.source_revision, 4096) {
            return Err(ProjectionError::Refused(ProjectionRefusal::InvalidIdentity));
        }
        let end = input.interval_start + input.dt;
        if !input.interval_start.is_finite()
            || input.interval_start < 0.0
            || !input.dt.is_finite()
            || input.dt <= 0.0
            || !end.is_finite()
            || end <= input.interval_start
            || input.settings.max_iterations == 0
            || [
                input.settings.relative_residual,
                input.settings.absolute_residual,
                input.settings.divergence_limit,
                input.actual_divergence_limit,
            ]
            .iter()
            .any(|v| !v.is_finite() || *v < 0.0)
        {
            return Err(ProjectionError::InvalidInput);
        }
        PressureOperator::new(input.grid, input.density)?.validate_velocity(input.velocity)?;
        let mut hash = Sha256::new();
        hash_text(&mut hash, CONTRACT);
        let representation = ProjectionRepresentation::sealed_v1(input.grid);
        hash.update(representation.version.to_le_bytes());
        hash_text(&mut hash, representation.operator);
        for metadata in std::iter::once(&representation.pressure).chain(&representation.velocity) {
            for count in metadata.shape_xyz {
                hash.update((count as u64).to_le_bytes());
            }
            hash_text(&mut hash, metadata.layout);
            hash_text(&mut hash, metadata.scalar);
            hash_text(&mut hash, metadata.units);
        }
        for n in input.grid.counts() {
            hash.update((n as u64).to_le_bytes());
        }
        for value in input.grid.spacing().into_iter().chain(input.grid.origin()) {
            hash.update(value.to_bits().to_le_bytes());
        }
        hash.update(input.source.run_id.to_le_bytes());
        hash_text(&mut hash, &input.source.source_revision);
        hash.update(input.generation.to_le_bytes());
        for value in [
            input.density,
            input.interval_start,
            input.dt,
            input.settings.relative_residual,
            input.settings.absolute_residual,
            input.settings.divergence_limit,
            input.actual_divergence_limit,
        ] {
            hash.update(value.to_bits().to_le_bytes());
        }
        hash.update((input.settings.max_iterations as u64).to_le_bytes());
        for values in input.velocity {
            hash.update((values.len() as u64).to_le_bytes());
            for value in values {
                hash.update(value.to_bits().to_le_bytes());
            }
        }
        Ok(Self {
            input,
            input_id: ProjectionInputId(hash.finalize().into()),
        })
    }
    pub fn input_id(&self) -> ProjectionInputId {
        self.input_id
    }
    pub fn representation(&self) -> ProjectionRepresentation {
        ProjectionRepresentation::sealed_v1(self.input.grid)
    }
}

fn valid_text(value: &str, limit: usize) -> bool {
    !value.is_empty() && value.len() <= limit && !value.chars().any(char::is_control)
}
fn hash_text(hash: &mut Sha256, value: &str) {
    hash.update((value.len() as u64).to_le_bytes());
    hash.update(value.as_bytes());
}

/// Strict new supplied-array representation; pressure-v1 metadata stays
/// descriptive under its original API and is not silently strengthened.
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectionRepresentation {
    pub version: u32,
    pub operator: &'static str,
    pub pressure: TensorMetadata,
    pub velocity: [TensorMetadata; 3],
}
impl ProjectionRepresentation {
    pub fn sealed_v1(grid: &GridGeometry) -> Self {
        Self {
            version: 1,
            operator: OPERATOR,
            pressure: pressure_metadata(grid.counts()),
            velocity: std::array::from_fn(|d| TensorMetadata {
                shape_xyz: grid.face_counts([Axis::X, Axis::Y, Axis::Z][d]),
                layout: ["x-fastest-x-face", "x-fastest-y-face", "x-fastest-z-face"][d],
                scalar: "f32",
                units: "m/s",
            }),
        }
    }
}
fn pressure_metadata(shape_xyz: [usize; 3]) -> TensorMetadata {
    TensorMetadata {
        shape_xyz,
        layout: "x-fastest-cell-centered",
        scalar: "f64",
        units: "Pa",
    }
}

/// Caller-declared provenance of supplied data, not observed runtime execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectionCandidateIdentity {
    pub algorithm: &'static str,
    pub version: &'static str,
    pub source_revision: &'static str,
}
pub struct SuppliedProjection<'a> {
    pub input_id: ProjectionInputId,
    pub representation: ProjectionRepresentation,
    pub identity: ProjectionCandidateIdentity,
    pub pressure: &'a [f64],
    pub velocity: [&'a [f32]; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionSelection {
    SuppliedCandidate,
    NativeBaseline,
    NativeFallback,
}
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectionRejection {
    Field(OperatorError),
    MomentumWitness { axis: Axis, index: usize },
    Pressure(ProviderRejection),
    ActualDivergence { maximum: f64, limit: f64 },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionRefusal {
    UnsupportedDomain(ProjectionDomain),
    WorkspaceGrid,
    InputBinding,
    Representation,
    InvalidIdentity,
}
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectionError {
    Refused(ProjectionRefusal),
    Operator(OperatorError),
    Pressure(PressureError),
    InvalidInput,
    Cancelled,
    BufferLimit { required: usize, limit: usize },
    AllocationFailed,
    DivergenceExceeded { maximum: f64, limit: f64 },
}
impl fmt::Display for ProjectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "supplied projection qualification: {self:?}")
    }
}
impl std::error::Error for ProjectionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Operator(error) => Some(error),
            Self::Pressure(error) => Some(error),
            _ => None,
        }
    }
}
impl From<OperatorError> for ProjectionError {
    fn from(error: OperatorError) -> Self {
        Self::Operator(error)
    }
}
impl From<PressureError> for ProjectionError {
    fn from(error: PressureError) -> Self {
        Self::Pressure(error)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionCheckpoint {
    BeforeQualification,
    BeforePressure,
    PressurePoll,
    AfterPressure,
    BeforeReturn,
}
#[derive(Debug, Clone)]
pub struct ProjectionReport {
    pub input_id: ProjectionInputId,
    pub selection: ProjectionSelection,
    pub supplied_identity: Option<ProjectionCandidateIdentity>,
    pub rejection: Option<ProjectionRejection>,
    /// Final selected pressure dispatch facts. The private supplied-array bridge is
    /// a pressure witness, not observed candidate-provider execution.
    pub pressure: ProviderReport,
    /// Earlier accepted supplied pressure discarded by the subsequent actual
    /// f32 divergence gate, before a separate native fallback dispatch.
    pub discarded_candidate_pressure: Option<ProviderReport>,
    pub actual_divergence_max: f64,
}

/// Reusable disposable scratch. Managed array capacities include the original
/// pressure workspace, RHS/divergence and three f32 face arrays. Caller data,
/// allocator overhead, stack and metadata are excluded. No Session state lives
/// here and a failed call leaves no qualified result.
pub struct ProjectionWorkspace {
    grid: GridGeometry,
    pressure: ProviderWorkspace,
    rhs: Vec<f64>,
    divergence: Vec<f64>,
    velocity: [Vec<f32>; 3],
    allocated: usize,
}
impl ProjectionWorkspace {
    pub fn new(
        grid: &GridGeometry,
        limit: usize,
        fallback: PressureImplementation,
    ) -> Result<Self, ProjectionError> {
        let faces = [Axis::X, Axis::Y, Axis::Z]
            .into_iter()
            .try_fold(0usize, |n, axis| {
                n.checked_add(grid.face_len(axis))
                    .ok_or(ProjectionError::AllocationFailed)
            })?;
        let required = grid
            .cell_len()
            .checked_mul(16)
            .and_then(|n| faces.checked_mul(4).and_then(|m| n.checked_add(m)))
            .ok_or(ProjectionError::AllocationFailed)?;
        if required > limit {
            return Err(ProjectionError::BufferLimit { required, limit });
        }
        let rhs = allocate(grid.cell_len(), 0.0f64)?;
        let divergence = allocate(grid.cell_len(), 0.0f64)?;
        let velocity = [
            allocate(grid.face_len(Axis::X), 0.0f32)?,
            allocate(grid.face_len(Axis::Y), 0.0f32)?,
            allocate(grid.face_len(Axis::Z), 0.0f32)?,
        ];
        let extra = rhs
            .capacity()
            .checked_add(divergence.capacity())
            .and_then(|n| n.checked_mul(8))
            .and_then(|n| {
                velocity.iter().try_fold(n, |n, v| {
                    v.capacity().checked_mul(4).and_then(|m| n.checked_add(m))
                })
            })
            .ok_or(ProjectionError::AllocationFailed)?;
        let remaining = limit
            .checked_sub(extra)
            .ok_or(ProjectionError::BufferLimit {
                required: extra,
                limit,
            })?;
        let pressure = ProviderWorkspace::new(grid, remaining, fallback)?;
        let allocated = extra
            .checked_add(pressure.allocated_bytes())
            .ok_or(ProjectionError::AllocationFailed)?;
        if allocated > limit {
            return Err(ProjectionError::BufferLimit {
                required: allocated,
                limit,
            });
        }
        Ok(Self {
            grid: grid.clone(),
            pressure,
            rhs,
            divergence,
            velocity,
            allocated,
        })
    }
    pub fn allocated_bytes(&self) -> usize {
        self.allocated
    }

    /// Integrated owner reads only after successful qualification and before
    /// the next call. This scratch is never an accepted-state authority.
    pub(crate) fn pressure_for_integrated_step(&self) -> &[f64] {
        self.pressure.pressure()
    }

    /// Shares the same successful-call lifetime as the selected pressure.
    pub(crate) fn velocity_for_integrated_step(&self) -> [&[f32]; 3] {
        [&self.velocity[0], &self.velocity[1], &self.velocity[2]]
    }

    /// Qualify immutable supplied arrays or run the declared compatible native
    /// baseline. Header/binding refusals never become fallback acceptance.
    pub fn qualify(
        &mut self,
        request: &ProjectionRequest<'_>,
        candidate: Option<&SuppliedProjection<'_>>,
        mut cancelled: impl FnMut(ProjectionCheckpoint) -> bool,
    ) -> Result<QualifiedProjection<'_>, ProjectionError> {
        let input = &request.input;
        if input.grid != &self.grid {
            return Err(ProjectionError::Refused(ProjectionRefusal::WorkspaceGrid));
        }
        if let Some(candidate) = candidate {
            if candidate.input_id != request.input_id {
                return Err(ProjectionError::Refused(ProjectionRefusal::InputBinding));
            }
            if candidate.representation != request.representation() {
                return Err(ProjectionError::Refused(ProjectionRefusal::Representation));
            }
            if !valid_text(candidate.identity.algorithm, 256)
                || !valid_text(candidate.identity.version, 256)
                || !valid_text(candidate.identity.source_revision, 4096)
            {
                return Err(ProjectionError::Refused(ProjectionRefusal::InvalidIdentity));
            }
        }
        checkpoint(&mut cancelled, ProjectionCheckpoint::BeforeQualification)?;
        let operator = PressureOperator::new(input.grid, input.density)?;
        operator.build_rhs(input.velocity, input.dt, &mut self.rhs)?;
        let mut rejection = candidate.and_then(|candidate| {
            check_candidate(&operator, input, candidate, &mut self.velocity).err()
        });
        checkpoint(&mut cancelled, ProjectionCheckpoint::BeforePressure)?;
        let pressure_request = PressureRequest {
            grid: input.grid,
            density: input.density,
            dt: input.dt,
            rhs: &self.rhs,
        };
        let mut supplied =
            candidate
                .filter(|_| rejection.is_none())
                .map(|candidate| SuppliedPressure {
                    values: candidate.pressure,
                    shape: input.grid.counts(),
                });
        let preferred = supplied
            .as_mut()
            .map(|value| value as &mut dyn PressureProvider);
        let mut pressure_report =
            self.pressure
                .solve(pressure_request, input.settings, preferred, || {
                    cancelled(ProjectionCheckpoint::PressurePoll)
                })?;
        checkpoint(&mut cancelled, ProjectionCheckpoint::AfterPressure)?;
        let mut supplied_accepted = pressure_report.selected == SUPPLIED_PRESSURE;
        let mut discarded_candidate_pressure = None;
        if let Some(reason) = &pressure_report.rejection {
            rejection = Some(ProjectionRejection::Pressure(reason.clone()));
        }
        rebuild_velocity(
            &operator,
            input,
            self.pressure.pressure(),
            &mut self.velocity,
        )?;
        let mut divergence = actual_divergence(&operator, &self.velocity, &mut self.divergence)?;
        if supplied_accepted && divergence > input.actual_divergence_limit {
            rejection = Some(ProjectionRejection::ActualDivergence {
                maximum: divergence,
                limit: input.actual_divergence_limit,
            });
            // All candidate output is disposable. Resolve and correct from the
            // same immutable original input, never partially corrected scratch.
            let native_report =
                self.pressure
                    .solve(pressure_request, input.settings, None, || {
                        cancelled(ProjectionCheckpoint::PressurePoll)
                    })?;
            discarded_candidate_pressure =
                Some(std::mem::replace(&mut pressure_report, native_report));
            checkpoint(&mut cancelled, ProjectionCheckpoint::AfterPressure)?;
            supplied_accepted = false;
            rebuild_velocity(
                &operator,
                input,
                self.pressure.pressure(),
                &mut self.velocity,
            )?;
            divergence = actual_divergence(&operator, &self.velocity, &mut self.divergence)?;
        }
        if divergence > input.actual_divergence_limit {
            return Err(ProjectionError::DivergenceExceeded {
                maximum: divergence,
                limit: input.actual_divergence_limit,
            });
        }
        checkpoint(&mut cancelled, ProjectionCheckpoint::BeforeReturn)?;
        let selection = if supplied_accepted {
            ProjectionSelection::SuppliedCandidate
        } else if candidate.is_some() {
            ProjectionSelection::NativeFallback
        } else {
            ProjectionSelection::NativeBaseline
        };
        Ok(QualifiedProjection {
            pressure: self.pressure.pressure(),
            velocity: [&self.velocity[0], &self.velocity[1], &self.velocity[2]],
            report: ProjectionReport {
                input_id: request.input_id,
                selection,
                supplied_identity: candidate.map(|candidate| candidate.identity),
                rejection,
                pressure: pressure_report,
                discarded_candidate_pressure,
                actual_divergence_max: divergence,
            },
        })
    }
}

fn allocate<T: Clone>(length: usize, value: T) -> Result<Vec<T>, ProjectionError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(length)
        .map_err(|_| ProjectionError::AllocationFailed)?;
    values.resize(length, value);
    Ok(values)
}
fn checkpoint(
    cancel: &mut impl FnMut(ProjectionCheckpoint) -> bool,
    point: ProjectionCheckpoint,
) -> Result<(), ProjectionError> {
    if cancel(point) {
        Err(ProjectionError::Cancelled)
    } else {
        Ok(())
    }
}
fn rebuild_velocity(
    operator: &PressureOperator<'_>,
    input: &ProjectionInput<'_>,
    pressure: &[f64],
    velocity: &mut [Vec<f32>; 3],
) -> Result<(), OperatorError> {
    for (output, source) in velocity.iter_mut().zip(input.velocity) {
        output.copy_from_slice(source);
    }
    let [x, y, z] = velocity;
    operator.correct_candidate_in_place(pressure, input.dt, [x, y, z])
}
fn check_candidate(
    operator: &PressureOperator<'_>,
    input: &ProjectionInput<'_>,
    candidate: &SuppliedProjection<'_>,
    velocity: &mut [Vec<f32>; 3],
) -> Result<(), ProjectionRejection> {
    operator
        .validate_cells(candidate.pressure)
        .map_err(ProjectionRejection::Field)?;
    operator
        .validate_velocity(candidate.velocity)
        .map_err(ProjectionRejection::Field)?;
    rebuild_velocity(operator, input, candidate.pressure, velocity)
        .map_err(ProjectionRejection::Field)?;
    for (d, expected) in velocity.iter().enumerate() {
        if let Some(index) = expected
            .iter()
            .zip(candidate.velocity[d])
            .position(|(a, b)| a.to_bits() != b.to_bits())
        {
            return Err(ProjectionRejection::MomentumWitness {
                axis: [Axis::X, Axis::Y, Axis::Z][d],
                index,
            });
        }
    }
    Ok(())
}
fn actual_divergence(
    operator: &PressureOperator<'_>,
    velocity: &[Vec<f32>; 3],
    output: &mut [f64],
) -> Result<f64, OperatorError> {
    operator.divergence([&velocity[0], &velocity[1], &velocity[2]], output)?;
    Ok(output
        .iter()
        .fold(0.0f64, |maximum, value| maximum.max(value.abs())))
}

struct SuppliedPressure<'a> {
    values: &'a [f64],
    shape: [usize; 3],
}
impl PressureProvider for SuppliedPressure<'_> {
    fn metadata(&self) -> SolverMetadata {
        SolverMetadata {
            id: SUPPLIED_PRESSURE,
            version: "1",
            stage: "pressure",
            input: TensorMetadata {
                shape_xyz: self.shape,
                layout: "x-fastest-cell-centered",
                scalar: "f64",
                units: "m^3/s^2",
            },
            output: pressure_metadata(self.shape),
            domain: "sealed box supplied-array witness only",
            provenance: None,
        }
    }
    fn supports(&self, _: &PressureRequest<'_>) -> Result<(), ProviderRejection> {
        Ok(())
    }
    fn predict(
        &mut self,
        _: &PressureRequest<'_>,
        output: &mut [f64],
        _: &mut dyn FnMut() -> bool,
    ) -> Result<(), ProviderRejection> {
        output.copy_from_slice(self.values);
        Ok(())
    }
}

/// Qualified disposable arrays, not accepted Session state. The immutable borrow
/// prevents another qualification while this view remains in use.
///
/// ```compile_fail,E0499
/// use rheon::solver_provider::composition::projection::{ProjectionWorkspace, ProjectionRequest};
/// fn no_overwrite(workspace: &mut ProjectionWorkspace, request: &ProjectionRequest<'_>) {
///     let qualified = workspace.qualify(request, None, |_| false).unwrap();
///     let _next = workspace.qualify(request, None, |_| false);
///     println!("{:?}", qualified.velocity());
/// }
/// ```
pub struct QualifiedProjection<'a> {
    pressure: &'a [f64],
    velocity: [&'a [f32]; 3],
    report: ProjectionReport,
}
impl QualifiedProjection<'_> {
    pub fn pressure(&self) -> &[f64] {
        self.pressure
    }
    pub fn velocity(&self) -> [&[f32]; 3] {
        self.velocity
    }
    pub fn report(&self) -> &ProjectionReport {
        &self.report
    }
}

#[cfg(test)]
#[path = "projection_tests.rs"]
mod tests;
