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
    #[cfg(feature = "parallel")]
    let _timer = crate::scheduling::SerialTimer::new(
        crate::observation::CalculationStage::Reduction,
        a.len(),
    );
    let probe = crate::observation::WorkProbe::new(crate::observation::CalculationStage::Reduction);
    compensated_sum(a.iter().zip(b).enumerate().map(|(i, (x, y))| {
        probe.visit_index(i);
        x * y
    }))
}

// SGS shares previously completed tiles through atomic f64 bit storage. It
// replaces the existing preconditioned array, retaining the same 8-byte payload
// per cell; Jacobi and core-only builds retain ordinary f64 storage.
enum Preconditioned {
    Serial(Vec<f64>),
    #[cfg(all(feature = "parallel", target_has_atomic = "64"))]
    Wavefront(Vec<std::sync::atomic::AtomicU64>),
}
impl Preconditioned {
    fn new(n: usize, method: PressureImplementation) -> Result<Self, PressureError> {
        #[cfg(all(feature = "parallel", target_has_atomic = "64"))]
        if method == PressureImplementation::SymmetricGaussSeidelPcgV1
            && n >= crate::sgs::MIN_VALUES
        {
            let mut values = Vec::new();
            values
                .try_reserve_exact(n)
                .map_err(|_| PressureError::AllocationFailed)?;
            values.resize_with(n, || std::sync::atomic::AtomicU64::new(0));
            return Ok(Self::Wavefront(values));
        }
        let _ = method;
        Ok(Self::Serial(allocate(n)?))
    }
    fn capacity(&self) -> usize {
        match self {
            Self::Serial(v) => v.capacity(),
            #[cfg(all(feature = "parallel", target_has_atomic = "64"))]
            Self::Wavefront(v) => v.capacity(),
        }
    }
    fn get(&self, i: usize) -> f64 {
        match self {
            Self::Serial(v) => v[i],
            #[cfg(all(feature = "parallel", target_has_atomic = "64"))]
            Self::Wavefront(v) => f64::from_bits(v[i].load(std::sync::atomic::Ordering::Relaxed)),
        }
    }
    fn set(&mut self, i: usize, value: f64) {
        match self {
            Self::Serial(v) => v[i] = value,
            #[cfg(all(feature = "parallel", target_has_atomic = "64"))]
            Self::Wavefront(v) => v[i].store(value.to_bits(), std::sync::atomic::Ordering::Relaxed),
        }
    }
    fn serial_mut(&mut self) -> &mut [f64] {
        match self {
            Self::Serial(v) => v,
            #[cfg(all(feature = "parallel", target_has_atomic = "64"))]
            Self::Wavefront(_) => unreachable!("Jacobi retains ordinary f64 storage"),
        }
    }
    fn dot(&self, values: &[f64]) -> f64 {
        match self {
            Self::Serial(v) => dot(values, v),
            #[cfg(all(feature = "parallel", target_has_atomic = "64"))]
            Self::Wavefront(_) => {
                let _timer = crate::scheduling::SerialTimer::new(
                    crate::observation::CalculationStage::Reduction,
                    values.len(),
                );
                let probe = crate::observation::WorkProbe::new(
                    crate::observation::CalculationStage::Reduction,
                );
                compensated_sum(values.iter().enumerate().map(|(i, v)| {
                    probe.visit_index(i);
                    v * self.get(i)
                }))
            }
        }
    }
    fn copy_to(&self, output: &mut [f64]) {
        match self {
            Self::Serial(v) => output.copy_from_slice(v),
            #[cfg(all(feature = "parallel", target_has_atomic = "64"))]
            Self::Wavefront(_) => crate::parallel::for_each_mut(output, 0, |i, v| *v = self.get(i)),
        }
    }
}

/// Six reusable binary64 cell arrays. Solve calls allocate no field buffers; optional
/// parallel scratch writes use the caller's Rayon pool and scheduler.
/// The allocation limit covers Vec-capacity payload, not allocator metadata,
/// stack diagnostics, velocity/tracer fields, output images or process RSS.
pub struct PressureWorkspace {
    implementation: PressureImplementation,
    pressure: Vec<f64>,
    rhs: Vec<f64>,
    residual: Vec<f64>,
    direction: Vec<f64>,
    product: Vec<f64>,
    preconditioned: Preconditioned,
    #[cfg(all(feature = "parallel", target_has_atomic = "64"))]
    sgs_plan: crate::sgs::SweepPlan,
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
        let preconditioned = Preconditioned::new(n, implementation)?;
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
            #[cfg(all(feature = "parallel", target_has_atomic = "64"))]
            sgs_plan: crate::sgs::SweepPlan::default(),
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
        crate::parallel::for_each_mut(&mut self.product, 0, |i, product| {
            *product = self.rhs[i] - *product;
        });
        let squared = dot(&self.product, &self.product);
        let maximum = self.product.iter().fold(0.0_f64, |a, b| a.max(b.abs()));
        let divergence = operator.scaled_residual_divergence(&self.product, dt);
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
        if operator.has_gauge() {
            self.residual[0] = 0.0;
            self.preconditioned.set(0, 0.0);
        }
        #[cfg(all(feature = "parallel", target_has_atomic = "64"))]
        if let Preconditioned::Wavefront(values) = &self.preconditioned {
            crate::sgs::apply(operator, &self.residual, values, &mut self.sgs_plan)?;
            return Ok(self.preconditioned.dot(&self.residual));
        }
        let [nx, ny, nz] = operator.geometry().counts();
        let strides = [1, nx, nx * ny];
        let first = usize::from(operator.has_gauge());
        {
            let probe = crate::observation::WorkProbe::new(
                crate::observation::CalculationStage::SgsForward,
            );
            for row in first..self.residual.len() {
                probe.visit_index(row);
                let p = [row % nx, (row / nx) % ny, row / (nx * ny)];
                let diagonal = operator.diagonal(p);
                let mut value = self.residual[row];
                for d in 0..3 {
                    if p[d] > 0 && operator.is_wet(row) && operator.is_wet(row - strides[d]) {
                        value += operator.neighbor_weight(crate::Axis::ALL[d], p)
                            * self.preconditioned.get(row - strides[d]);
                    }
                }
                self.preconditioned.set(row, value / diagonal);
                if !self.preconditioned.get(row).is_finite()
                    || diagonal <= 0.0
                    || !diagonal.is_finite()
                {
                    return Err(PressureError::Breakdown { iteration: 0 });
                }
            }
        }
        {
            let probe = crate::observation::WorkProbe::new(
                crate::observation::CalculationStage::SgsReverse,
            );
            for row in (first..self.residual.len()).rev() {
                probe.visit_index(row);
                let p = [row % nx, (row / nx) % ny, row / (nx * ny)];
                let diagonal = operator.diagonal(p);
                let mut correction = 0.0;
                for d in 0..3 {
                    if p[d] + 1 < [nx, ny, nz][d]
                        && operator.is_wet(row)
                        && operator.is_wet(row + strides[d])
                    {
                        let mut upper = p;
                        upper[d] += 1;
                        correction += operator.neighbor_weight(crate::Axis::ALL[d], upper)
                            * self.preconditioned.get(row + strides[d]);
                    }
                }
                self.preconditioned
                    .set(row, self.preconditioned.get(row) + correction / diagonal);
                if !self.preconditioned.get(row).is_finite() {
                    return Err(PressureError::Breakdown { iteration: 0 });
                }
            }
        }
        Ok(self.preconditioned.dot(&self.residual))
    }

    fn precondition_jacobi(
        &mut self,
        operator: &PressureOperator<'_>,
    ) -> Result<f64, PressureError> {
        let grid = operator.geometry();
        if operator.has_gauge() {
            self.residual[0] = 0.0;
            self.preconditioned.set(0, 0.0);
        }
        crate::parallel::try_for_each_chunk_mut(
            self.preconditioned.serial_mut(),
            crate::observation::CalculationStage::Jacobi,
            2048,
            |start, out| {
                let probe = crate::observation::WorkProbe::new(
                    crate::observation::CalculationStage::Jacobi,
                );
                crate::parallel::for_each_row_mut(
                    out,
                    start,
                    grid.counts(),
                    |row_start, origin, out| {
                        for (local, value) in out.iter_mut().enumerate() {
                            let row = row_start + local;
                            let p = [origin[0] + local, origin[1], origin[2]];
                            probe.visit(p);
                            if row == 0 && operator.has_gauge() {
                                continue;
                            }
                            let diagonal = operator.diagonal(p);
                            if !diagonal.is_finite() || diagonal <= 0.0 {
                                return Err(PressureError::Breakdown { iteration: 0 });
                            }
                            *value = self.residual[row] / diagonal;
                        }
                        Ok(())
                    },
                )
            },
        )?;
        Ok(self.preconditioned.dot(&self.residual))
    }

    #[cfg(feature = "work-observation")]
    fn fused_update_residual(&mut self, alpha: f64, has_gauge: bool) -> f64 {
        let _timer = crate::scheduling::SerialTimer::new(
            crate::observation::CalculationStage::Reduction,
            self.residual.len(),
        );
        let probe =
            crate::observation::WorkProbe::new(crate::observation::CalculationStage::Reduction);
        // Observed single-worker updates and their ordered norm share one
        // traversal and one diagnostic store per cell. Reduction timing
        // includes the update here. Unobserved/parallel loops stay separate.
        // Include the pinned zero in the norm, exactly as the separate dot.
        compensated_sum(
            self.pressure
                .iter_mut()
                .zip(&mut self.residual)
                .enumerate()
                .map(|(i, (pressure, residual))| {
                    probe.visit_index(i);
                    if i != 0 || !has_gauge {
                        *pressure += alpha * self.direction[i];
                        *residual -= alpha * self.product[i];
                    }
                    *residual * *residual
                }),
        )
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
        self.preconditioned.copy_to(&mut self.direction);
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
            #[cfg(feature = "work-observation")]
            let fused_squared =
                if rayon::current_num_threads() == 1 && crate::observation::is_active() {
                    Some(self.fused_update_residual(alpha, operator.has_gauge()))
                } else {
                    None
                };
            #[cfg(not(feature = "work-observation"))]
            let fused_squared: Option<f64> = None;
            let recursive_l2 = if let Some(squared) = fused_squared {
                squared.sqrt()
            } else {
                let first = usize::from(operator.has_gauge());
                crate::parallel::for_each_pair_mut(
                    &mut self.pressure[first..],
                    &mut self.residual[first..],
                    first,
                    |i, pressure, residual| {
                        let i = i + first;
                        *pressure += alpha * self.direction[i];
                        *residual -= alpha * self.product[i];
                    },
                );
                dot(&self.residual, &self.residual).sqrt()
            };
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
                self.preconditioned.copy_to(&mut self.direction);
            } else {
                let beta = next_rz / rz;
                let first = usize::from(operator.has_gauge());
                crate::parallel::for_each_mut(
                    &mut self.direction[first..],
                    first,
                    |i, direction| {
                        *direction = self.preconditioned.get(i + first) + beta * *direction;
                    },
                );
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

    #[cfg(feature = "work-observation")]
    #[test]
    fn fused_update_matches_separate_ordered_reference() {
        let check = || {
            for gauge in [false, true] {
                for length in [1, 257, 32768] {
                    let grid = GridGeometry::new([length, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
                    let n = grid.cell_len();
                    let mut workspace = PressureWorkspace::new(&grid, n * 48).unwrap();
                    for i in 0..n {
                        workspace.pressure[i] = (i as f64 - 128.0) * 0.03125;
                        workspace.residual[i] = ((i * 17) % 113) as f64 - 56.0;
                        workspace.direction[i] = ((i * 31) % 97) as f64 * 1e-9;
                        workspace.product[i] = ((i * 13) % 79) as f64 * 1e8;
                    }
                    if gauge {
                        workspace.pressure[0] = 0.0;
                        workspace.residual[0] = 0.0;
                    }
                    let mut pressure = workspace.pressure.clone();
                    let mut residual = workspace.residual.clone();
                    for alpha in [-0.0, 0.125, -1e-9, 1e9, f64::MIN_POSITIVE, f64::MAX] {
                        for i in usize::from(gauge)..n {
                            pressure[i] += alpha * workspace.direction[i];
                            residual[i] -= alpha * workspace.product[i];
                        }
                        let expected = dot(&residual, &residual);
                        let actual = {
                            let observer = crate::CalculationObserver::new(&grid, 4).unwrap();
                            observer.observe(|| workspace.fused_update_residual(alpha, gauge))
                        };
                        assert_eq!(actual.to_bits(), expected.to_bits());
                        for (actual, expected) in workspace
                            .pressure
                            .iter()
                            .zip(&pressure)
                            .chain(workspace.residual.iter().zip(&residual))
                        {
                            assert_eq!(actual.to_bits(), expected.to_bits());
                        }
                    }
                }
            }
        };
        for workers in [1, 2, 4] {
            rayon::ThreadPoolBuilder::new()
                .num_threads(workers)
                .build()
                .unwrap()
                .install(check);
        }
    }

    #[cfg(all(feature = "parallel", target_has_atomic = "64"))]
    #[test]
    fn tiled_sgs_matches_serial_bits_across_boundaries_gauge_and_free_surfaces() {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(4)
            .build()
            .unwrap();
        let check = |operator: &PressureOperator<'_>| {
            let grid = operator.geometry();
            let n = grid.cell_len();
            let mut serial = PressureWorkspace::with_implementation(
                grid,
                n * 48,
                PressureImplementation::SymmetricGaussSeidelPcgV1,
            )
            .unwrap();
            let mut parallel = PressureWorkspace::with_implementation(
                grid,
                n * 48,
                PressureImplementation::SymmetricGaussSeidelPcgV1,
            )
            .unwrap();
            serial.preconditioned = Preconditioned::Serial(allocate(n).unwrap());
            assert!(matches!(
                parallel.preconditioned,
                Preconditioned::Wavefront(_)
            ));
            for repeat in 0..2 {
                for i in 0..n {
                    let r = if operator.is_wet(i) {
                        ((i * 17 + repeat * 29) % 113) as f64 / 113.0 - 0.5
                    } else {
                        0.0
                    };
                    serial.residual[i] = r;
                    parallel.residual[i] = r;
                }
                // Exercise the original lexicographic sweep directly; avoid
                // invoking the wavefront path from this reference call.
                let reference_pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(1)
                    .build()
                    .unwrap();
                let expected = reference_pool
                    .install(|| serial.precondition_sgs(operator))
                    .unwrap();
                let actual = pool
                    .install(|| parallel.precondition_sgs(operator))
                    .unwrap();
                assert_eq!(actual.to_bits(), expected.to_bits());
                for i in 0..n {
                    assert_eq!(
                        parallel.preconditioned.get(i).to_bits(),
                        serial.preconditioned.get(i).to_bits(),
                        "cell {i}, repetition {repeat}"
                    );
                }
            }
        };
        let grid = GridGeometry::new([35, 32, 31], [0.04, 0.07, 0.13], [0.0; 3]).unwrap();
        check(&PressureOperator::new(&grid, 1.3).unwrap());
        for axis in crate::Axis::ALL {
            let surface = crate::SlabFreeSurface::new(
                grid.clone(),
                axis,
                17,
                crate::VolumeStamp { id: 0, version: 0 },
            )
            .unwrap();
            check(&PressureOperator::with_free_surface(&grid, 1.3, &surface).unwrap());
        }
        let grid = GridGeometry::new([1, 129, 257], [0.05; 3], [0.0; 3]).unwrap();
        check(&PressureOperator::new(&grid, 1.0).unwrap());
        let grid = GridGeometry::new([1, 1, 32769], [0.05; 3], [0.0; 3]).unwrap();
        check(&PressureOperator::new(&grid, 1.0).unwrap());
    }

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
        let mut values = [0.0; 4];
        ws.preconditioned.copy_to(&mut values);
        assert_eq!(values, [0.0, 3.25, 4.25, 4.5]);
        assert_eq!(rz, 50.5);
    }
}
