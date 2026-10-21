//! Version 1 physical pressure context. Tensor preparation belongs to the adapter.
//! The caller declares lineage; validation proves coherence, not authenticity or
//! the correctness of upstream viscosity/advection composition.
use crate::{GridGeometry, OperatorError, PressureOperator};
use std::fmt;

pub const STAGE_CONTRACT: &str = "sealed-mac-pressure-context-v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PressureContextMode {
    AcceptedCandidate,
    DiagnosticCandidate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PressureHistoryStatus {
    Accepted,
    Diagnostic { qualified: bool },
}

#[derive(Clone, Copy, Debug)]
pub struct PressureHistory<'a> {
    pub values: &'a [f64],
    pub grid: &'a GridGeometry,
    pub density: f64,
    pub run_id: &'a str,
    pub generation: u64,
    pub interval_end: f64,
    pub status: PressureHistoryStatus,
}

/// Borrowed caller inputs. Successful construction retains immutable references;
/// previous pressure is absent at construction rather than invented solved zeros.
pub struct PressureContextInput<'a> {
    pub grid: &'a GridGeometry,
    pub density: f64,
    pub dt: f64,
    pub interval_start: f64,
    pub rhs: &'a [f64],
    pub prior: Option<PressureHistory<'a>>,
    pub run_id: &'a str,
    pub generation: u64,
    pub mode: PressureContextMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PressureContextError {
    Operator(OperatorError),
    InvalidInterval,
    InvalidLineage,
    InvalidGauge,
    IncompatibleHistoryDomain,
    DiagnosticHistoryInAcceptedContext,
}
impl fmt::Display for PressureContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Operator(e) => write!(f, "pressure context: {e}"),
            Self::InvalidInterval => f.write_str("invalid pressure interval"),
            Self::InvalidLineage => f.write_str("missing, stale or other-run pressure lineage"),
            Self::InvalidGauge => f.write_str("previous pressure must have cell-zero gauge"),
            Self::IncompatibleHistoryDomain => {
                f.write_str("previous pressure grid/operator identity differs")
            }
            Self::DiagnosticHistoryInAcceptedContext => {
                f.write_str("diagnostic history cannot become accepted history")
            }
        }
    }
}
impl std::error::Error for PressureContextError {}
impl From<OperatorError> for PressureContextError {
    fn from(value: OperatorError) -> Self {
        Self::Operator(value)
    }
}

pub struct PressureContext<'a> {
    input: PressureContextInput<'a>,
    interval_end: f64,
}
impl<'a> PressureContext<'a> {
    pub fn new(input: PressureContextInput<'a>) -> Result<Self, PressureContextError> {
        let end = input.interval_start + input.dt;
        if !input.dt.is_finite()
            || input.dt <= 0.0
            || !input.interval_start.is_finite()
            || input.interval_start < 0.0
            || !end.is_finite()
            || end <= input.interval_start
        {
            return Err(PressureContextError::InvalidInterval);
        }
        if input.run_id.is_empty()
            || input.run_id.len() > 4096
            || input.run_id.chars().any(char::is_control)
            || input.generation == 0
        {
            return Err(PressureContextError::InvalidLineage);
        }
        let operator = PressureOperator::new(input.grid, input.density)?;
        operator.validate_cells(input.rhs)?;
        if let Some(prior) = input.prior {
            if prior.grid != input.grid || prior.density != input.density {
                return Err(PressureContextError::IncompatibleHistoryDomain);
            }
            operator.validate_cells(prior.values)?;
            if prior.values.first() != Some(&0.0) {
                return Err(PressureContextError::InvalidGauge);
            }
            if prior.generation == 0
                || prior.interval_end <= 0.0
                || prior.run_id != input.run_id
                || prior.generation.checked_add(1) != Some(input.generation)
                || prior.interval_end != input.interval_start
            {
                return Err(PressureContextError::InvalidLineage);
            }
            if input.mode == PressureContextMode::AcceptedCandidate
                && prior.status != PressureHistoryStatus::Accepted
            {
                return Err(PressureContextError::DiagnosticHistoryInAcceptedContext);
            }
        } else if input.generation != 1 || input.interval_start != 0.0 {
            return Err(PressureContextError::InvalidLineage);
        }
        Ok(Self {
            input,
            interval_end: end,
        })
    }
    pub fn grid(&self) -> &GridGeometry {
        self.input.grid
    }
    pub fn density(&self) -> f64 {
        self.input.density
    }
    pub fn dt(&self) -> f64 {
        self.input.dt
    }
    pub fn interval_start(&self) -> f64 {
        self.input.interval_start
    }
    pub fn interval_end(&self) -> f64 {
        self.interval_end
    }
    pub fn rhs(&self) -> &[f64] {
        self.input.rhs
    }
    pub fn prior(&self) -> Option<PressureHistory<'a>> {
        self.input.prior
    }
    pub fn run_id(&self) -> &str {
        self.input.run_id
    }
    pub fn generation(&self) -> u64 {
        self.input.generation
    }
    pub fn mode(&self) -> PressureContextMode {
        self.input.mode
    }

    /// Compute b-A*p0 using Rheon's authoritative full sealed operator. Absent
    /// history means the explicit zero initial guess, not solved pressure history.
    pub fn initial_residual(&self, output: &mut [f64]) -> Result<(), PressureContextError> {
        if output.len() != self.grid().cell_len() {
            return Err(OperatorError::LengthMismatch.into());
        }
        if let Some(prior) = self.prior() {
            PressureOperator::new(self.grid(), self.density())?.apply_full(prior.values, output)?;
            for (value, rhs) in output.iter_mut().zip(self.rhs()) {
                *value = *rhs - *value;
                if !value.is_finite() {
                    return Err(OperatorError::ArithmeticFailure.into());
                }
            }
        } else {
            output.copy_from_slice(self.rhs());
        }
        Ok(())
    }
}
