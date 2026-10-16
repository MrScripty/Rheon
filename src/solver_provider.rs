//! Qualified pressure-stage providers for sealed, constant-density MAC boxes.
//! A provider is a candidate generator. Rheon, not the provider, accepts pressure.
//! Corrected f32 velocity divergence must still be checked by the caller.
use crate::{
    GridGeometry, OperatorError, PressureError, PressureImplementation, PressureOperator,
    PressureReport, PressureSettings, PressureWorkspace,
};
use std::time::{Duration, Instant};
pub mod configured;
pub mod context;
mod modal;
pub use modal::LabModalPressure;

/// Descriptive, versioned capability metadata. New stages need their own typed
/// request/qualification API; a stage label alone never grants compatibility.
#[derive(Debug, Clone, PartialEq)]
pub struct SolverMetadata {
    pub id: &'static str,
    pub version: &'static str,
    pub stage: &'static str,
    pub input: TensorMetadata,
    pub output: TensorMetadata,
    pub domain: &'static str,
    pub provenance: Option<ModelProvenance>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct TensorMetadata {
    pub shape_xyz: [usize; 3],
    pub layout: &'static str,
    pub scalar: &'static str,
    pub units: &'static str,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ModelProvenance {
    pub repository: &'static str,
    pub commit: &'static str,
    pub checkpoint_sha256: &'static str,
    pub export_sha256: &'static str,
    pub runtime: &'static str,
}

/// RHS of the *full*, ungauged integrated operator, x-fastest, in m³/s².
/// Construction of a sealed operator is owned by the dispatcher: no obstacle,
/// variable-density or free-surface operator can be substituted through metadata.
#[derive(Clone, Copy)]
pub struct PressureRequest<'a> {
    pub grid: &'a GridGeometry,
    pub density: f64,
    pub dt: f64,
    pub rhs: &'a [f64],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderRejection {
    Unsupported(&'static str),
    Execution(&'static str),
    NonFiniteOutput,
    NonZeroGauge,
    QualityGate,
    Cancelled,
}

/// Caller-supplied implementations run synchronously in-process. They must honor
/// cancellation and bound their own allocations. Panics are programming errors,
/// not recovered inference failures. All output cells must be written.
pub trait PressureProvider {
    fn metadata(&self) -> SolverMetadata;
    fn supports(&self, request: &PressureRequest<'_>) -> Result<(), ProviderRejection>;
    fn predict(
        &mut self,
        request: &PressureRequest<'_>,
        pressure: &mut [f64],
        cancelled: &mut dyn FnMut() -> bool,
    ) -> Result<(), ProviderRejection>;
}

#[derive(Debug, Clone)]
pub struct ProviderReport {
    pub selected: &'static str,
    pub attempted: Option<SolverMetadata>,
    pub rejection: Option<ProviderRejection>,
    /// Recomputed prediction diagnostics, retained even when quality rejects it.
    pub candidate: Option<PressureReport>,
    pub accepted: PressureReport,
    /// Complete call including input checks, inference, validation and fallback.
    /// Workspace/model construction and caller's velocity correction are separate.
    pub total: Duration,
    pub inference: Duration,
    pub qualification: Duration,
    pub fallback: Duration,
}

/// Eight cell arrays including the six numerical-reference arrays. A failed call
/// invalidates pressure(); callers must publish only after Ok and their own
/// corrected-velocity gate. Construction is separate from invocation timing.
pub struct ProviderWorkspace {
    grid: GridGeometry,
    reference: PressureWorkspace,
    fallback: PressureImplementation,
    pressure: Vec<f64>,
    product: Vec<f64>,
}
impl ProviderWorkspace {
    pub fn new(
        grid: &GridGeometry,
        limit: usize,
        fallback: PressureImplementation,
    ) -> Result<Self, PressureError> {
        let extra = grid
            .cell_len()
            .checked_mul(16)
            .ok_or(PressureError::AllocationFailed)?;
        if extra > limit {
            return Err(PressureError::BufferLimit {
                required: extra,
                limit,
            });
        }
        let reference = PressureWorkspace::with_implementation(grid, limit - extra, fallback)?;
        let allocate = || {
            let mut v = Vec::new();
            v.try_reserve_exact(grid.cell_len())
                .map_err(|_| PressureError::AllocationFailed)?;
            v.resize(grid.cell_len(), 0.0);
            Ok::<_, PressureError>(v)
        };
        let pressure = allocate()?;
        let product = allocate()?;
        let actual = reference.allocated_bytes() + (pressure.capacity() + product.capacity()) * 8;
        if actual > limit {
            return Err(PressureError::BufferLimit {
                required: actual,
                limit,
            });
        }
        Ok(Self {
            grid: grid.clone(),
            reference,
            fallback,
            pressure,
            product,
        })
    }
    pub fn pressure(&self) -> &[f64] {
        &self.pressure
    }
    pub fn allocated_bytes(&self) -> usize {
        self.reference.allocated_bytes() + (self.pressure.capacity() + self.product.capacity()) * 8
    }
    pub fn solve(
        &mut self,
        request: PressureRequest<'_>,
        settings: PressureSettings,
        preferred: Option<&mut dyn PressureProvider>,
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<ProviderReport, PressureError> {
        let start = Instant::now();
        if request.grid != &self.grid {
            return Err(OperatorError::LengthMismatch.into());
        }
        let operator = PressureOperator::new(request.grid, request.density)?;
        operator.validate_cells(request.rhs)?;
        if !request.dt.is_finite()
            || request.dt <= 0.0
            || [
                settings.relative_residual,
                settings.absolute_residual,
                settings.divergence_limit,
            ]
            .iter()
            .any(|v| !v.is_finite() || *v < 0.0)
        {
            return Err(PressureError::InvalidSettings);
        }
        let sum = sum(request.rhs.iter().copied());
        let sum_abs = self::sum(request.rhs.iter().map(|v| v.abs()));
        let norm = self::sum(request.rhs.iter().map(|v| v * v)).sqrt();
        let threshold = settings
            .absolute_residual
            .max(settings.relative_residual * norm);
        if !sum.is_finite() || !sum_abs.is_finite() || !norm.is_finite() || !threshold.is_finite() {
            return Err(PressureError::Breakdown { iteration: 0 });
        }
        let rounding_budget = 64.0 * f64::EPSILON * sum_abs;
        if sum.abs() > rounding_budget {
            return Err(PressureError::IncompatibleRhs {
                sum,
                rounding_budget,
            });
        }
        if cancelled() {
            return Err(PressureError::Cancelled { iterations: 0 });
        }
        let mut report = ProviderReport {
            selected: self.fallback.id(),
            attempted: None,
            rejection: None,
            candidate: None,
            accepted: PressureReport {
                iterations: 0,
                true_residual_l2: 0.0,
                true_residual_max: 0.0,
                predicted_divergence_max: 0.0,
            },
            total: Duration::ZERO,
            inference: Duration::ZERO,
            qualification: Duration::ZERO,
            fallback: Duration::ZERO,
        };
        if let Some(provider) = preferred {
            report.attempted = Some(provider.metadata());
            let attempt = provider.supports(&request).and_then(|()| {
                self.pressure.fill(f64::NAN);
                let begin = Instant::now();
                let result = provider.predict(&request, &mut self.pressure, &mut cancelled);
                report.inference = begin.elapsed();
                result
            });
            if cancelled() || attempt == Err(ProviderRejection::Cancelled) {
                return Err(PressureError::Cancelled { iterations: 0 });
            }
            let begin = Instant::now();
            let qualified = attempt.and_then(|()| {
                if self.pressure.iter().any(|v| !v.is_finite()) {
                    return Err(ProviderRejection::NonFiniteOutput);
                }
                if self.pressure[0] != 0.0 {
                    return Err(ProviderRejection::NonZeroGauge);
                }
                operator
                    .apply_full(&self.pressure, &mut self.product)
                    .map_err(|_| ProviderRejection::NonFiniteOutput)?;
                for (a, b) in self.product.iter_mut().zip(request.rhs) {
                    *a = b - *a;
                }
                let l2 = self::sum(self.product.iter().map(|v| v * v)).sqrt();
                let max = self.product.iter().fold(0.0_f64, |a, b| a.max(b.abs()));
                let divergence = operator.scaled_residual_divergence(&self.product, request.dt);
                let candidate = PressureReport {
                    iterations: 0,
                    true_residual_l2: l2,
                    true_residual_max: max,
                    predicted_divergence_max: divergence,
                };
                report.candidate = Some(candidate);
                if !l2.is_finite()
                    || !divergence.is_finite()
                    || l2 > threshold
                    || divergence > settings.divergence_limit
                {
                    return Err(ProviderRejection::QualityGate);
                }
                Ok(candidate)
            });
            report.qualification = begin.elapsed();
            if cancelled() {
                return Err(PressureError::Cancelled { iterations: 0 });
            }
            match qualified {
                Ok(accepted) => {
                    report.selected = report.attempted.as_ref().unwrap().id;
                    report.accepted = accepted;
                    report.total = start.elapsed();
                    return Ok(report);
                }
                Err(reason) => report.rejection = Some(reason),
            }
        }
        let begin = Instant::now();
        report.accepted = self.reference.solve_rhs(
            &operator,
            request.rhs,
            request.dt,
            settings,
            &mut cancelled,
        )?;
        self.pressure.copy_from_slice(self.reference.pressure());
        report.fallback = begin.elapsed();
        report.total = start.elapsed();
        Ok(report)
    }
}
// Neumaier sum, matching the reference's compatibility and residual reductions.
fn sum(values: impl Iterator<Item = f64>) -> f64 {
    let mut total = 0.0_f64;
    let mut correction = 0.0_f64;
    for value in values {
        let next = total + value;
        correction += if total.abs() >= value.abs() {
            (total - next) + value
        } else {
            (value - next) + total
        };
        total = next;
    }
    total + correction
}
