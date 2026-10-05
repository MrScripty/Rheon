//! Prescribed normal inlet/outlet speeds on a fixed rectangular control volume.
//! This affine projection primitive does not move geometry or classify solids.
use crate::{
    Axis, GridGeometry, OperatorError, PressureError, PressureImplementation, PressureOperator,
    PressureReport, PressureSettings, PressureWorkspace,
};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoxFluxStamp {
    pub id: u64,
    pub version: u64,
}
/// Uniform outward-normal speed in m/s on each complete box side. Lower faces
/// have outward normal -axis; upper faces +axis. Interior face speeds are unknown.
/// Identity/version uniqueness is caller-owned. No pressure is also prescribed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrescribedBoxFlux {
    stamp: BoxFluxStamp,
    outward: [[f32; 2]; 3],
}
impl PrescribedBoxFlux {
    pub fn new(stamp: BoxFluxStamp, outward: [[f32; 2]; 3]) -> Result<Self, BoxFluxError> {
        if outward.into_iter().flatten().any(|v| !v.is_finite()) {
            return Err(BoxFluxError::InvalidBoundary);
        }
        Ok(Self { stamp, outward })
    }
    pub fn stamp(self) -> BoxFluxStamp {
        self.stamp
    }
    pub fn outward_speeds(self) -> [[f32; 2]; 3] {
        self.outward
    }
    fn component(self, d: usize, upper: bool) -> f32 {
        let speed = self.outward[d][usize::from(upper)];
        if speed == 0.0 {
            0.0
        } else if upper {
            speed
        } else {
            -speed
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct BoxFluxSettings {
    pub pressure: PressureSettings,
    /// Direct all-cell divergence of the stored f32 result, in 1/s.
    pub actual_divergence_limit: f64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoxFluxStage {
    BeforeSolve,
    PressureIteration,
    CorrectionSlice,
    BeforeAcceptance,
}
#[derive(Debug, Clone, PartialEq)]
pub enum BoxFluxError {
    Operator(OperatorError),
    Pressure(PressureError),
    InvalidBoundary,
    InvalidSettings,
    InvalidMass,
    AllocationFailed,
    BufferLimit { required: usize, limit: usize },
    IncompatibleFlux { net: f64, rounding_budget: f64 },
    Cancelled { stage: BoxFluxStage },
    DivergenceLimit { actual: f64, limit: f64 },
    ArithmeticFailure,
}
impl fmt::Display for BoxFluxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "box flux projection rejected operation: {self:?}")
    }
}
impl std::error::Error for BoxFluxError {}
impl From<OperatorError> for BoxFluxError {
    fn from(v: OperatorError) -> Self {
        Self::Operator(v)
    }
}
impl From<PressureError> for BoxFluxError {
    fn from(v: PressureError) -> Self {
        Self::Pressure(v)
    }
}

/// Energy uses only interior face degrees of freedom, each with rho*cell_volume
/// mass. Prescribed outer samples have no independent inertial degree of freedom.
#[derive(Debug, Clone, Copy, Default)]
pub struct BoxFluxWork {
    pub kinetic_before: f64,
    pub kinetic_after: f64,
    pub correction_energy: f64,
    /// -dt sum_cell p*Q, where Q is known outward integrated boundary flux.
    pub boundary_pressure_work: f64,
    /// dt sum_cell p*R, R=cell_volume*actual_divergence of stored velocities.
    pub divergence_residual_work: f64,
    /// sum_edge eta*v, eta=m*(v-u)+dt*area*(p_right-p_left).
    /// Includes stored f32 rounding and floating coefficient arithmetic.
    pub correction_residual_work: f64,
    pub budget_error: f64,
}
#[derive(Debug, Clone, Copy)]
pub struct BoxFluxReport {
    pub boundary: BoxFluxStamp,
    pub pressure: PressureReport,
    pub actual_divergence_max: f64,
    pub net_outward_flux: f64,
    pub flux_rounding_budget: f64,
    pub work: BoxFluxWork,
}

/// Preserved pressure workspace plus one reusable f64 cell array. Owns immutable
/// grid/density; boundary is copied for each projection. Limits cover
/// actual Vec capacities, not caller face arrays, allocator overhead or RSS.
pub struct BoxFluxWorkspace {
    grid: GridGeometry,
    density: f64,
    area: [f64; 3],
    factor: [f64; 3],
    side_area: [f64; 3],
    mass: f64,
    implementation: PressureImplementation,
    pressure: PressureWorkspace,
    scratch: Vec<f64>,
    allocated_bytes: usize,
}
impl BoxFluxWorkspace {
    pub fn new(
        grid: GridGeometry,
        density: f64,
        limit: usize,
        implementation: PressureImplementation,
    ) -> Result<Self, BoxFluxError> {
        PressureOperator::new(&grid, density)?;
        let h = grid.spacing();
        let dims = grid.counts();
        let area = [h[1] * h[2], h[0] * h[2], h[0] * h[1]];
        let side_area =
            std::array::from_fn(|d| area[d] * dims[(d + 1) % 3] as f64 * dims[(d + 2) % 3] as f64);
        let factor = std::array::from_fn(|d| 1.0 / (density * h[d]));
        let mass = density * grid.cell_volume();
        if !mass.is_finite() || mass <= 0.0 {
            return Err(BoxFluxError::InvalidMass);
        }
        if side_area.into_iter().any(|a| !a.is_finite() || a <= 0.0) {
            return Err(BoxFluxError::ArithmeticFailure);
        }
        let n = grid.cell_len();
        let required = n.checked_mul(56).ok_or(BoxFluxError::AllocationFailed)?;
        if required > limit {
            return Err(BoxFluxError::BufferLimit { required, limit });
        }
        let mut scratch = Vec::new();
        scratch
            .try_reserve_exact(n)
            .map_err(|_| BoxFluxError::AllocationFailed)?;
        let extra = scratch
            .capacity()
            .checked_mul(8)
            .ok_or(BoxFluxError::AllocationFailed)?;
        let remaining = limit.checked_sub(extra).ok_or(BoxFluxError::BufferLimit {
            required: extra,
            limit,
        })?;
        scratch.resize(n, 0.0);
        let pressure = PressureWorkspace::with_implementation(&grid, remaining, implementation)?;
        let allocated_bytes = extra
            .checked_add(pressure.allocated_bytes())
            .ok_or(BoxFluxError::AllocationFailed)?;
        Ok(Self {
            grid,
            density,
            area,
            factor,
            side_area,
            mass,
            implementation,
            pressure,
            scratch,
            allocated_bytes,
        })
    }
    pub fn grid(&self) -> &GridGeometry {
        &self.grid
    }
    pub fn allocated_bytes(&self) -> usize {
        self.allocated_bytes
    }
    pub fn implementation(&self) -> PressureImplementation {
        self.implementation
    }
    /// Last pressure scratch with gauge cell zero. Consume only after a successful
    /// report: failure/cancellation can leave this unaccepted scratch modified.
    pub fn pressure(&self) -> &[f64] {
        self.pressure.pressure()
    }
    /// Direct measured divergence of any finite full face field, including all
    /// external values. This never substitutes prescribed data for stored faces.
    pub fn divergence(&self, velocity: [&[f32]; 3], out: &mut [f64]) -> Result<(), BoxFluxError> {
        validate(&self.grid, velocity)?;
        divergence(&self.grid, velocity, out)
    }
    /// Project immutable provisional fields into caller-owned disposable output.
    /// External input samples are validated but replaced by prescribed data.
    /// Pressure solves allocate no buffers; net flux rejects independently of the
    /// provisional field before solving. No mean shifting or pressure opening.
    /// A failure may leave output partial after correction begins; the caller must
    /// publish it only with a successful report. Inputs/boundary remain unchanged.
    pub fn project(
        &mut self,
        old: [&[f32]; 3],
        dt: f64,
        boundary: PrescribedBoxFlux,
        settings: BoxFluxSettings,
        output: [&mut [f32]; 3],
        mut cancelled: impl FnMut(BoxFluxStage) -> bool,
    ) -> Result<BoxFluxReport, BoxFluxError> {
        if !dt.is_finite() || dt <= 0.0 {
            return Err(OperatorError::InvalidTimeStep.into());
        }
        if !settings.actual_divergence_limit.is_finite() || settings.actual_divergence_limit < 0.0 {
            return Err(BoxFluxError::InvalidSettings);
        }
        validate(&self.grid, old)?;
        for axis in Axis::ALL {
            if output[axis.index()].len() != self.grid.face_len(axis) {
                return Err(OperatorError::LengthMismatch.into());
            }
        }
        let terms: [f64; 6] = std::array::from_fn(|i| {
            self.side_area[i / 2] * f64::from(boundary.outward[i / 2][i % 2])
        });
        // A nonzero requested flux must not disappear before compatibility is
        // checked. Reject an unrepresentable product instead of calling it zero.
        if terms
            .iter()
            .enumerate()
            .any(|(i, &term)| term == 0.0 && boundary.outward[i / 2][i % 2] != 0.0)
        {
            return Err(BoxFluxError::ArithmeticFailure);
        }
        let net = compensated(terms.into_iter());
        let absolute = compensated(terms.into_iter().map(f64::abs));
        let flux_rounding_budget = 64.0 * f64::EPSILON * absolute;
        if !net.is_finite() || !absolute.is_finite() {
            return Err(BoxFluxError::ArithmeticFailure);
        }
        if net.abs() > flux_rounding_budget {
            return Err(BoxFluxError::IncompatibleFlux {
                net,
                rounding_budget: flux_rounding_budget,
            });
        }
        checkpoint(&mut cancelled, BoxFluxStage::BeforeSolve)?;
        let [nx, ny, nz] = self.grid.counts();
        for k in 0..nz {
            for j in 0..ny {
                for i in 0..nx {
                    let p = [i, j, k];
                    let mut flux = 0.0;
                    for axis in Axis::ALL {
                        let d = axis.index();
                        let mut high = p;
                        high[d] += 1;
                        let lower = if p[d] == 0 {
                            boundary.component(d, false)
                        } else {
                            old[d][self.grid.face_unchecked(axis, p)]
                        };
                        let upper = if high[d] == self.grid.counts()[d] {
                            boundary.component(d, true)
                        } else {
                            old[d][self.grid.face_unchecked(axis, high)]
                        };
                        flux += self.area[d] * (f64::from(lower) - f64::from(upper));
                    }
                    let rhs = flux / dt;
                    if !rhs.is_finite() {
                        return Err(BoxFluxError::ArithmeticFailure);
                    }
                    self.scratch[self.grid.cell_unchecked(p)] = rhs;
                }
            }
        }
        let operator = PressureOperator::new(&self.grid, self.density)?;
        let pressure =
            self.pressure
                .solve_rhs(&operator, &self.scratch, dt, settings.pressure, || {
                    cancelled(BoxFluxStage::PressureIteration)
                })?;
        let mut work = BoxFluxWork::default();
        for axis in Axis::ALL {
            let d = axis.index();
            let dims = self.grid.face_counts(axis);
            for k in 0..dims[2] {
                checkpoint(&mut cancelled, BoxFluxStage::CorrectionSlice)?;
                for j in 0..dims[1] {
                    for i in 0..dims[0] {
                        let p = [i, j, k];
                        let index = self.grid.face_unchecked(axis, p);
                        if p[d] == 0 || p[d] == self.grid.counts()[d] {
                            output[d][index] = boundary.component(d, p[d] != 0);
                            continue;
                        }
                        let mut left = p;
                        left[d] -= 1;
                        let difference = self.pressure.pressure()[self.grid.cell_unchecked(p)]
                            - self.pressure.pressure()[self.grid.cell_unchecked(left)];
                        let u = f64::from(old[d][index]);
                        let value = u - dt * self.factor[d] * difference;
                        if !value.is_finite() || value.abs() > f64::from(f32::MAX) {
                            return Err(BoxFluxError::ArithmeticFailure);
                        }
                        let stored = value as f32;
                        output[d][index] = stored;
                        let v = f64::from(stored);
                        let delta = v - u;
                        work.kinetic_before += 0.5 * self.mass * u * u;
                        work.kinetic_after += 0.5 * self.mass * v * v;
                        work.correction_energy += 0.5 * self.mass * delta * delta;
                        work.correction_residual_work +=
                            (self.mass * delta + dt * self.area[d] * difference) * v;
                    }
                }
            }
        }
        divergence(
            &self.grid,
            [output[0], output[1], output[2]],
            &mut self.scratch,
        )?;
        let actual = self.scratch.iter().fold(0.0_f64, |a, b| a.max(b.abs()));
        if actual > settings.actual_divergence_limit {
            return Err(BoxFluxError::DivergenceLimit {
                actual,
                limit: settings.actual_divergence_limit,
            });
        }
        for k in 0..nz {
            for j in 0..ny {
                for i in 0..nx {
                    let p = [i, j, k];
                    let index = self.grid.cell_unchecked(p);
                    let pressure = self.pressure.pressure()[index];
                    let mut q = 0.0;
                    for (d, &position) in p.iter().enumerate() {
                        if position == 0 {
                            q += self.area[d] * f64::from(boundary.outward[d][0]);
                        }
                        if position + 1 == self.grid.counts()[d] {
                            q += self.area[d] * f64::from(boundary.outward[d][1]);
                        }
                    }
                    work.boundary_pressure_work -= dt * pressure * q;
                    work.divergence_residual_work +=
                        dt * pressure * self.grid.cell_volume() * self.scratch[index];
                }
            }
        }
        work.budget_error = work.kinetic_after - work.kinetic_before + work.correction_energy
            - (work.boundary_pressure_work
                + work.divergence_residual_work
                + work.correction_residual_work);
        if [
            work.kinetic_before,
            work.kinetic_after,
            work.correction_energy,
            work.boundary_pressure_work,
            work.divergence_residual_work,
            work.correction_residual_work,
            work.budget_error,
        ]
        .into_iter()
        .any(|v| !v.is_finite())
        {
            return Err(BoxFluxError::ArithmeticFailure);
        }
        checkpoint(&mut cancelled, BoxFluxStage::BeforeAcceptance)?;
        Ok(BoxFluxReport {
            boundary: boundary.stamp(),
            pressure,
            actual_divergence_max: actual,
            net_outward_flux: net,
            flux_rounding_budget,
            work,
        })
    }
}
fn validate(grid: &GridGeometry, values: [&[f32]; 3]) -> Result<(), BoxFluxError> {
    for axis in Axis::ALL {
        let v = values[axis.index()];
        if v.len() != grid.face_len(axis) {
            return Err(OperatorError::LengthMismatch.into());
        }
        if v.iter().any(|v| !v.is_finite()) {
            return Err(OperatorError::NonFiniteInput.into());
        }
    }
    Ok(())
}
fn divergence(
    grid: &GridGeometry,
    values: [&[f32]; 3],
    out: &mut [f64],
) -> Result<(), BoxFluxError> {
    if out.len() != grid.cell_len() {
        return Err(OperatorError::LengthMismatch.into());
    }
    let [nx, ny, nz] = grid.counts();
    let h = grid.spacing();
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let p = [i, j, k];
                let mut value = 0.0;
                for axis in Axis::ALL {
                    let d = axis.index();
                    let mut hi = p;
                    hi[d] += 1;
                    value += (f64::from(values[d][grid.face_unchecked(axis, hi)])
                        - f64::from(values[d][grid.face_unchecked(axis, p)]))
                        / h[d];
                }
                if !value.is_finite() {
                    return Err(BoxFluxError::ArithmeticFailure);
                }
                out[grid.cell_unchecked(p)] = value;
            }
        }
    }
    Ok(())
}
fn checkpoint(
    cancelled: &mut impl FnMut(BoxFluxStage) -> bool,
    stage: BoxFluxStage,
) -> Result<(), BoxFluxError> {
    if cancelled(stage) {
        Err(BoxFluxError::Cancelled { stage })
    } else {
        Ok(())
    }
}
fn compensated(values: impl Iterator<Item = f64>) -> f64 {
    let mut sum = 0.0_f64;
    let mut correction = 0.0;
    for v in values {
        let next = sum + v;
        correction += if sum.abs() >= v.abs() {
            (sum - next) + v
        } else {
            (v - next) + sum
        };
        sum = next;
    }
    sum + correction
}
