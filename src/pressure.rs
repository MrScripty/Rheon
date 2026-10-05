use crate::{GridGeometry, OperatorError, PressureOperator};
use std::fmt;

/// Preserved pressure approaches. Both use the same operator, gauge, residual
/// and physical acceptance gates; only the SPD preconditioner differs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PressureImplementation {
    #[default]
    JacobiPcgV1,
    SymmetricGaussSeidelPcgV1,
}
impl PressureImplementation {
    pub const ALL: [Self; 2] = [Self::JacobiPcgV1, Self::SymmetricGaussSeidelPcgV1];
    pub fn id(self) -> &'static str {
        match self {
            Self::JacobiPcgV1 => "jacobi-pcg-v1",
            Self::SymmetricGaussSeidelPcgV1 => "sgs-pcg-v1",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::JacobiPcgV1 => "Jacobi PCG (original v1)",
            Self::SymmetricGaussSeidelPcgV1 => "Symmetric Gauss-Seidel PCG (v1)",
        }
    }
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|method| method.id() == id)
    }
}

/// Linear-system and physical stopping conditions. Tolerances are explicit:
/// integrated residual has volume/time^2 units; divergence is inverse seconds.
#[derive(Debug, Clone, Copy)]
pub struct PressureSettings {
    pub relative_residual: f64,
    pub absolute_residual: f64,
    pub divergence_limit: f64,
    pub max_iterations: usize,
}

/// Successful pressure qualification before f32 velocity correction.
/// Actual corrected-velocity divergence remains a separate acceptance gate.
#[derive(Debug, Clone, Copy)]
pub struct PressureReport {
    pub iterations: usize,
    pub true_residual_l2: f64,
    pub true_residual_max: f64,
    pub predicted_divergence_max: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PressureError {
    Operator(OperatorError),
    InvalidSettings,
    BufferLimit {
        required: usize,
        limit: usize,
    },
    AllocationFailed,
    IncompatibleRhs {
        sum: f64,
        rounding_budget: f64,
    },
    Breakdown {
        iteration: usize,
    },
    IterationLimit {
        iterations: usize,
        residual_max: f64,
    },
    Cancelled {
        iterations: usize,
    },
}

impl fmt::Display for PressureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Operator(e) => write!(f, "{e}"),
            Self::InvalidSettings => write!(f, "pressure settings must be finite and nonnegative"),
            Self::BufferLimit { required, limit } => write!(
                f,
                "pressure buffers require {required} bytes, limit is {limit}"
            ),
            Self::AllocationFailed => write!(f, "pressure workspace allocation failed"),
            Self::IncompatibleRhs {
                sum,
                rounding_budget,
            } => write!(
                f,
                "sealed-domain RHS sum {sum} exceeds rounding budget {rounding_budget}"
            ),
            Self::Breakdown { iteration } => {
                write!(f, "pressure arithmetic broke down at iteration {iteration}")
            }
            Self::IterationLimit {
                iterations,
                residual_max,
            } => write!(
                f,
                "pressure iteration limit {iterations}, full residual maximum {residual_max}"
            ),
            Self::Cancelled { iterations } => {
                write!(f, "pressure solve cancelled after {iterations} iterations")
            }
        }
    }
}

impl std::error::Error for PressureError {}

impl From<OperatorError> for PressureError {
    fn from(value: OperatorError) -> Self {
        Self::Operator(value)
    }
}

fn allocate(n: usize) -> Result<Vec<f64>, PressureError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(n)
        .map_err(|_| PressureError::AllocationFailed)?;
    values.resize(n, 0.0);
    Ok(values)
}

fn compensated_sum(values: impl Iterator<Item = f64>) -> f64 {
    let mut sum = 0.0_f64;
    let mut correction = 0.0_f64;
    for value in values {
        let next = sum + value;
        correction += if sum.abs() >= value.abs() {
            (sum - next) + value
        } else {
            (value - next) + sum
        };
        sum = next;
    }
    sum + correction
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    compensated_sum(a.iter().zip(b).map(|(x, y)| x * y))
}

/// Six reusable f64 cell arrays. Solve calls allocate no heap buffers.
/// The allocation limit covers Vec-capacity payload, not allocator metadata,
/// stack diagnostics, velocity/tracer fields, output images or process RSS.
pub struct PressureWorkspace {
    implementation: PressureImplementation,
    pressure: Vec<f64>,
    rhs: Vec<f64>,
    residual: Vec<f64>,
    direction: Vec<f64>,
    product: Vec<f64>,
    preconditioned: Vec<f64>,
    allocated_bytes: usize,
}

impl PressureWorkspace {
    pub fn new(grid: &GridGeometry, limit: usize) -> Result<Self, PressureError> {
        Self::with_implementation(grid, limit, PressureImplementation::default())
    }

    pub fn with_implementation(
        grid: &GridGeometry,
        limit: usize,
        implementation: PressureImplementation,
    ) -> Result<Self, PressureError> {
        let n = grid.cell_len();
        let required = n.checked_mul(48).ok_or(PressureError::AllocationFailed)?;
        if required > limit {
            return Err(PressureError::BufferLimit { required, limit });
        }
        let pressure = allocate(n)?;
        let rhs = allocate(n)?;
        let residual = allocate(n)?;
        let direction = allocate(n)?;
        let product = allocate(n)?;
        let preconditioned = allocate(n)?;
        let allocated_bytes = [
            pressure.capacity(),
            rhs.capacity(),
            residual.capacity(),
            direction.capacity(),
            product.capacity(),
            preconditioned.capacity(),
        ]
        .iter()
        .try_fold(0_usize, |sum, &capacity| {
            capacity
                .checked_mul(8)
                .and_then(|bytes| sum.checked_add(bytes))
        })
        .ok_or(PressureError::AllocationFailed)?;
        if allocated_bytes > limit {
            return Err(PressureError::BufferLimit {
                required: allocated_bytes,
                limit,
            });
        }
        Ok(Self {
            implementation,
            pressure,
            rhs,
            residual,
            direction,
            product,
            preconditioned,
            allocated_bytes,
        })
    }

    pub fn allocated_bytes(&self) -> usize {
        self.allocated_bytes
    }

    /// Last pressure scratch. Closed mode fixes gauge cell zero; free-surface
    /// mode fixes air storage to atmospheric zero and does not pin a wet cell.
    /// Consumers must only use it
    /// as an accepted solve after a successful solve result.
    pub fn pressure(&self) -> &[f64] {
        &self.pressure
    }

    /// Reuses operator-product scratch after a solve. Pressure remains intact;
    /// this independently differentiates actual f32 faces on every cell.
    pub fn actual_divergence_max(
        &mut self,
        operator: &PressureOperator<'_>,
        velocity: [&[f32]; 3],
    ) -> Result<f64, PressureError> {
        operator.divergence(velocity, &mut self.product)?;
        Ok(self
            .product
            .iter()
            .enumerate()
            .filter(|(i, _)| operator.is_wet(*i))
            .fold(0.0_f64, |a, (_, b)| a.max(b.abs())))
    }

    pub fn solve_velocity(
        &mut self,
        operator: &PressureOperator<'_>,
        velocity: [&[f32]; 3],
        dt: f64,
        settings: PressureSettings,
        cancelled: impl FnMut() -> bool,
    ) -> Result<PressureReport, PressureError> {
        operator.build_rhs(velocity, dt, &mut self.rhs)?;
        self.solve_loaded(operator, dt, settings, cancelled)
    }

    /// Explicit RHS entry point for independently assembled fixtures/consumers.
    /// The full closed-domain RHS must be compatible; it is never mean-shifted.
    pub fn solve_rhs(
        &mut self,
        operator: &PressureOperator<'_>,
        rhs: &[f64],
        dt: f64,
        settings: PressureSettings,
        cancelled: impl FnMut() -> bool,
    ) -> Result<PressureReport, PressureError> {
        operator.validate_cells(rhs)?;
        if self.rhs.len() != rhs.len() {
            return Err(OperatorError::LengthMismatch.into());
        }
        if !operator.has_gauge() {
            for (cell, &value) in rhs.iter().enumerate() {
                if !operator.is_wet(cell) && value != 0.0 {
                    return Err(OperatorError::NonZeroAirRhs { cell }.into());
                }
            }
        }
        self.rhs.copy_from_slice(rhs);
        self.solve_loaded(operator, dt, settings, cancelled)
    }

    fn true_report(
        &mut self,
        operator: &PressureOperator<'_>,
        dt: f64,
        iterations: usize,
    ) -> Result<PressureReport, PressureError> {
        operator.apply_full(&self.pressure, &mut self.product)?;
        for i in 0..self.rhs.len() {
            self.product[i] = self.rhs[i] - self.product[i];
        }
        let squared = dot(&self.product, &self.product);
        let maximum = self.product.iter().fold(0.0_f64, |a, b| a.max(b.abs()));
        let divergence = maximum * dt / operator.geometry().cell_volume();
        if !squared.is_finite() || !divergence.is_finite() {
            return Err(PressureError::Breakdown {
                iteration: iterations,
            });
        }
        Ok(PressureReport {
            iterations,
            true_residual_l2: squared.sqrt(),
            true_residual_max: maximum,
            predicted_divergence_max: divergence,
        })
    }

    fn precondition(&mut self, operator: &PressureOperator<'_>) -> Result<f64, PressureError> {
        match self.implementation {
            PressureImplementation::JacobiPcgV1 => self.precondition_jacobi(operator),
            PressureImplementation::SymmetricGaussSeidelPcgV1 => self.precondition_sgs(operator),
        }
    }

    /// M = (D+L) D^-1 (D+L)^T on the gauge-eliminated SPD system.
    /// Forward and reverse triangular solves share one buffer; no new arrays.
    fn precondition_sgs(&mut self, operator: &PressureOperator<'_>) -> Result<f64, PressureError> {
        let [nx, ny, nz] = operator.geometry().counts();
        let strides = [1, nx, nx * ny];
        let weight = operator.weights();
        if operator.has_gauge() {
            self.residual[0] = 0.0;
            self.preconditioned[0] = 0.0;
        }
        let first = usize::from(operator.has_gauge());
        for row in first..self.residual.len() {
            let p = [row % nx, (row / nx) % ny, row / (nx * ny)];
            let diagonal = operator.diagonal(p);
            let mut value = self.residual[row];
            for d in 0..3 {
                if p[d] > 0 && operator.is_wet(row) && operator.is_wet(row - strides[d]) {
                    value += weight[d] * self.preconditioned[row - strides[d]];
                }
            }
            self.preconditioned[row] = value / diagonal;
            if !self.preconditioned[row].is_finite() || diagonal <= 0.0 || !diagonal.is_finite() {
                return Err(PressureError::Breakdown { iteration: 0 });
            }
        }
        for row in (first..self.residual.len()).rev() {
            let p = [row % nx, (row / nx) % ny, row / (nx * ny)];
            let diagonal = operator.diagonal(p);
            let mut correction = 0.0;
            for d in 0..3 {
                if p[d] + 1 < [nx, ny, nz][d]
                    && operator.is_wet(row)
                    && operator.is_wet(row + strides[d])
                {
                    correction += weight[d] * self.preconditioned[row + strides[d]];
                }
            }
            self.preconditioned[row] += correction / diagonal;
            if !self.preconditioned[row].is_finite() {
                return Err(PressureError::Breakdown { iteration: 0 });
            }
        }
        Ok(dot(&self.residual, &self.preconditioned))
    }

    fn precondition_jacobi(
        &mut self,
        operator: &PressureOperator<'_>,
    ) -> Result<f64, PressureError> {
        let grid = operator.geometry();
        let [nx, ny, nz] = grid.counts();
        if operator.has_gauge() {
            self.residual[0] = 0.0;
            self.preconditioned[0] = 0.0;
        }
        for k in 0..nz {
            for j in 0..ny {
                for i in 0..nx {
                    let p = [i, j, k];
                    let row = i + nx * (j + ny * k);
                    if row == 0 && operator.has_gauge() {
                        continue;
                    }
                    let diagonal = operator.diagonal(p);
                    if !diagonal.is_finite() || diagonal <= 0.0 {
                        return Err(PressureError::Breakdown { iteration: 0 });
                    }
                    self.preconditioned[row] = self.residual[row] / diagonal;
                }
            }
        }
        Ok(dot(&self.residual, &self.preconditioned))
    }

    fn solve_loaded(
        &mut self,
        operator: &PressureOperator<'_>,
        dt: f64,
        settings: PressureSettings,
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<PressureReport, PressureError> {
        if self.rhs.len() != operator.geometry().cell_len() {
            return Err(OperatorError::LengthMismatch.into());
        }
        if !dt.is_finite()
            || dt <= 0.0
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
        if cancelled() {
            return Err(PressureError::Cancelled { iterations: 0 });
        }
        let sum = compensated_sum(self.rhs.iter().copied());
        let sum_abs = compensated_sum(self.rhs.iter().map(|v| v.abs()));
        let rounding_budget = (64.0 * f64::EPSILON) * sum_abs;
        if !sum.is_finite() || !sum_abs.is_finite() {
            return Err(PressureError::Breakdown { iteration: 0 });
        }
        if operator.has_gauge() && sum.abs() > rounding_budget {
            return Err(PressureError::IncompatibleRhs {
                sum,
                rounding_budget,
            });
        }
        self.pressure.fill(0.0);
        let rhs_l2 = dot(&self.rhs, &self.rhs).sqrt();
        let threshold = settings
            .absolute_residual
            .max(settings.relative_residual * rhs_l2);
        if !rhs_l2.is_finite() || !threshold.is_finite() {
            return Err(PressureError::Breakdown { iteration: 0 });
        }
        let accepted = |report: PressureReport| {
            report.true_residual_l2 <= threshold
                && report.predicted_divergence_max <= settings.divergence_limit
        };
        let mut report = self.true_report(operator, dt, 0)?;
        if accepted(report) {
            return Ok(report);
        }
        if settings.max_iterations == 0 {
            return Err(PressureError::IterationLimit {
                iterations: 0,
                residual_max: report.true_residual_max,
            });
        }
        self.residual.copy_from_slice(&self.product);
        let mut rz = self.precondition(operator)?;
        self.direction.copy_from_slice(&self.preconditioned);
        for iteration in 1..=settings.max_iterations {
            if cancelled() {
                return Err(PressureError::Cancelled {
                    iterations: iteration - 1,
                });
            }
            operator.apply_full(&self.direction, &mut self.product)?;
            if operator.has_gauge() {
                self.product[0] = 0.0;
            }
            let curvature = dot(&self.direction, &self.product);
            if !curvature.is_finite() || curvature <= 0.0 || !rz.is_finite() || rz <= 0.0 {
                return Err(PressureError::Breakdown { iteration });
            }
            let alpha = rz / curvature;
            if !alpha.is_finite() {
                return Err(PressureError::Breakdown { iteration });
            }
            for i in usize::from(operator.has_gauge())..self.rhs.len() {
                self.pressure[i] += alpha * self.direction[i];
                self.residual[i] -= alpha * self.product[i];
            }
            let recursive_l2 = dot(&self.residual, &self.residual).sqrt();
            let candidate = recursive_l2 <= threshold;
            // Checking true residuals must not discard conjugate directions
            // solely because the physical divergence needs more iterations.
            // Replace/restart when the recursive linear candidate is rejected.
            let mut replaced = false;
            if candidate || iteration == settings.max_iterations {
                report = self.true_report(operator, dt, iteration)?;
                if accepted(report) {
                    return Ok(report);
                }
                if report.true_residual_l2 > threshold {
                    self.residual.copy_from_slice(&self.product);
                    replaced = true;
                }
            }
            if iteration == settings.max_iterations {
                return Err(PressureError::IterationLimit {
                    iterations: iteration,
                    residual_max: report.true_residual_max,
                });
            }
            let next_rz = self.precondition(operator)?;
            if !next_rz.is_finite() || next_rz <= 0.0 {
                return Err(PressureError::Breakdown { iteration });
            }
            if replaced {
                self.direction.copy_from_slice(&self.preconditioned);
            } else {
                let beta = next_rz / rz;
                for i in usize::from(operator.has_gauge())..self.rhs.len() {
                    self.direction[i] = self.preconditioned[i] + beta * self.direction[i];
                }
            }
            rz = next_rz;
        }
        // Nonempty iteration range returns success or a typed failure above.
        Err(PressureError::IterationLimit {
            iterations: settings.max_iterations,
            residual_max: report.true_residual_max,
        })
    }
}

#[cfg(test)]
mod comparison_tests {
    use super::*;

    #[test]
    fn sgs_matches_independently_assembled_triangular_system() {
        // 2x2x1 uniform grid, eliminate cell0: A=[[2,0,-1],[0,2,-1],[-1,-1,2]].
        // r=[2,4,6]: forward y=[1,2,4.5], reverse z=[3.25,4.25,4.5].
        let g = GridGeometry::new([2, 2, 1], [1.0; 3], [0.0; 3]).unwrap();
        let op = PressureOperator::new(&g, 1.0).unwrap();
        let mut ws = PressureWorkspace::with_implementation(
            &g,
            192,
            PressureImplementation::SymmetricGaussSeidelPcgV1,
        )
        .unwrap();
        ws.residual.copy_from_slice(&[-12.0, 2.0, 4.0, 6.0]);
        let rz = ws.precondition(&op).unwrap();
        assert_eq!(ws.preconditioned, [0.0, 3.25, 4.25, 4.5]);
        assert_eq!(rz, 50.5);
    }
}
