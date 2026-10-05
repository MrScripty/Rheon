//! First-order shared-face represented liquid-volume transport on a fixed MAC
//! grid. This volume-only state has no free-surface pressure or mesh-solid mask.
use crate::{Axis, GridGeometry};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VolumeStamp {
    pub id: u64,
    pub version: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiquidOccupancy {
    Dry,
    Mixed,
    Full,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolumeStage {
    BeforeFlux,
    FaceFluxSlice,
    CellUpdateSlice,
    BeforeCommit,
}
#[derive(Debug, Clone, PartialEq)]
pub enum LiquidVolumeError {
    InvalidDensity,
    InvalidFraction {
        cell: usize,
    },
    InvalidInlet,
    InvalidSource,
    InvalidSettings,
    InvalidTimeStep,
    TimeResolution,
    TimeMismatch,
    GeometryMismatch,
    LengthMismatch,
    NonFiniteVelocity,
    VersionOverflow,
    AllocationFailed,
    BufferLimit {
        required: usize,
        limit: usize,
    },
    Cancelled {
        stage: VolumeStage,
    },
    CourantLimit {
        cell: usize,
        actual: f64,
        limit: f64,
    },
    DivergenceLimit {
        actual: f64,
        limit: f64,
    },
    FractionBounds {
        cell: usize,
        fraction: f64,
    },
    BalanceLimit {
        error: f64,
        rounding_budget: f64,
    },
    ArithmeticFailure,
}
impl fmt::Display for LiquidVolumeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "liquid volume transport rejected operation: {self:?}")
    }
}
impl std::error::Error for LiquidVolumeError {}

/// Uniform outside liquid fraction per complete side, used only on inflow.
/// Outflow uses the old adjacent cell fraction. Unique identity is caller-owned.
#[derive(Debug, Clone, Copy)]
pub struct LiquidInlet {
    stamp: VolumeStamp,
    fraction: [[f64; 2]; 3],
}
impl LiquidInlet {
    pub fn new(stamp: VolumeStamp, fraction: [[f64; 2]; 3]) -> Result<Self, LiquidVolumeError> {
        if fraction
            .into_iter()
            .flatten()
            .any(|v| !v.is_finite() || !(0.0..=1.0).contains(&v))
        {
            return Err(LiquidVolumeError::InvalidInlet);
        }
        Ok(Self { stamp, fraction })
    }
    pub fn stamp(self) -> VolumeStamp {
        self.stamp
    }
    pub fn fractions(self) -> [[f64; 2]; 3] {
        self.fraction
    }
}
/// Immutable carrier velocities held constant over one explicit interval.
/// The stamp describes supplied data, not certified pressure/interface coupling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolumeDivergenceDomain {
    AllCells,
    LiquidSlab { axis: Axis, wet_layers: usize },
}
#[derive(Clone, Copy)]
pub struct LiquidFlowInterval<'a> {
    grid: &'a GridGeometry,
    stamp: VolumeStamp,
    velocity: [&'a [f32]; 3],
    start: f64,
    dt: f64,
    end: f64,
    domain: VolumeDivergenceDomain,
}
impl<'a> LiquidFlowInterval<'a> {
    pub fn new(
        grid: &'a GridGeometry,
        stamp: VolumeStamp,
        velocity: [&'a [f32]; 3],
        start: f64,
        dt: f64,
    ) -> Result<Self, LiquidVolumeError> {
        if !start.is_finite() || start < 0.0 || !dt.is_finite() || dt <= 0.0 {
            return Err(LiquidVolumeError::InvalidTimeStep);
        }
        let end = start + dt;
        if !end.is_finite() || end <= start {
            return Err(LiquidVolumeError::TimeResolution);
        }
        for axis in Axis::ALL {
            let field = velocity[axis.index()];
            if field.len() != grid.face_len(axis) {
                return Err(LiquidVolumeError::LengthMismatch);
            }
            if field.iter().any(|v| !v.is_finite()) {
                return Err(LiquidVolumeError::NonFiniteVelocity);
            }
        }
        Ok(Self {
            grid,
            stamp,
            velocity,
            start,
            dt,
            end,
            domain: VolumeDivergenceDomain::AllCells,
        })
    }
    pub(crate) fn on_slab(mut self, surface: &crate::SlabFreeSurface) -> Self {
        self.domain = VolumeDivergenceDomain::LiquidSlab {
            axis: surface.axis(),
            wet_layers: surface.wet_layers(),
        };
        self
    }
    pub fn stamp(self) -> VolumeStamp {
        self.stamp
    }
    pub fn start(self) -> f64 {
        self.start
    }
    pub fn end(self) -> f64 {
        self.end
    }
    pub fn dt(self) -> f64 {
        self.dt
    }
}
/// Signed liquid volume source rates, in m³/s per cell, borrowed for this call.
#[derive(Clone, Copy)]
pub struct LiquidVolumeSource<'a> {
    stamp: VolumeStamp,
    rate: &'a [f64],
}
impl<'a> LiquidVolumeSource<'a> {
    pub fn new(stamp: VolumeStamp, rate: &'a [f64]) -> Result<Self, LiquidVolumeError> {
        if rate.iter().any(|v| !v.is_finite()) {
            return Err(LiquidVolumeError::InvalidSource);
        }
        Ok(Self { stamp, rate })
    }
}
#[derive(Debug, Clone, Copy)]
pub struct LiquidVolumeSettings {
    /// Local summed outward volume Courant number; finite and in (0,1].
    pub max_outward_courant: f64,
    /// Direct all-cell carrier divergence in 1/s. Bounds are checked separately;
    /// a small accepted carrier residual can still cause a raw fraction rejection.
    pub actual_divergence_limit: f64,
}
#[derive(Debug, Clone, Copy)]
pub struct LiquidVolumeReport {
    pub stamp: VolumeStamp,
    pub flow: VolumeStamp,
    pub inlet: VolumeStamp,
    pub source: Option<VolumeStamp>,
    pub dt: f64,
    pub time: f64,
    pub liquid_volume_before: f64,
    pub liquid_volume_after: f64,
    pub inward_boundary_volume: f64,
    pub outward_boundary_volume: f64,
    pub source_volume: f64,
    pub volume_balance_error: f64,
    pub volume_rounding_budget: f64,
    pub liquid_mass_before: f64,
    pub liquid_mass_after: f64,
    pub max_outward_courant: f64,
    pub actual_divergence_max: f64,
    pub divergence_domain: VolumeDivergenceDomain,
    pub fraction_min: f64,
    pub fraction_max: f64,
    pub dry_cells: usize,
    pub mixed_cells: usize,
    pub full_cells: usize,
}
#[derive(Clone, Copy)]
pub struct LiquidVolumeView<'a> {
    pub fraction: &'a [f64],
    pub stamp: VolumeStamp,
    pub time: f64,
    /// Constant represented-liquid density, not a carrier pressure coefficient.
    pub density: f64,
}

/// Accepted and candidate f64 fraction arrays plus three shared f64 face-transfer
/// arrays. Actual owned Vec capacities are capped. Flow/source inputs, allocator
/// overhead and RSS are excluded. Advance allocates no heap buffers.
/// Occupancy is derived from accepted fractions; it is not a pressure/solid mask.
pub struct LiquidVolumeState {
    grid: GridGeometry,
    density: f64,
    stamp: VolumeStamp,
    time: f64,
    accepted: Vec<f64>,
    candidate: Vec<f64>,
    transfer: [Vec<f64>; 3],
    allocated_bytes: usize,
}
impl LiquidVolumeState {
    pub fn new(
        grid: GridGeometry,
        density: f64,
        stamp: VolumeStamp,
        fraction: Vec<f64>,
        limit: usize,
    ) -> Result<Self, LiquidVolumeError> {
        if !density.is_finite() || density <= 0.0 {
            return Err(LiquidVolumeError::InvalidDensity);
        }
        if fraction.len() != grid.cell_len() {
            return Err(LiquidVolumeError::LengthMismatch);
        }
        for (cell, &value) in fraction.iter().enumerate() {
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                return Err(LiquidVolumeError::InvalidFraction { cell });
            }
            product(grid.cell_volume(), value)?;
        }
        let initial_volume = sum(fraction.iter().map(|&v| grid.cell_volume() * v));
        if !initial_volume.is_finite() || !product(density, initial_volume)?.is_finite() {
            return Err(LiquidVolumeError::ArithmeticFailure);
        }
        let lengths = [
            grid.cell_len(),
            grid.face_len(Axis::X),
            grid.face_len(Axis::Y),
            grid.face_len(Axis::Z),
        ];
        let capacities = lengths
            .into_iter()
            .try_fold(fraction.capacity(), |total, n| {
                total
                    .checked_add(n)
                    .ok_or(LiquidVolumeError::AllocationFailed)
            })?;
        let required = capacities
            .checked_mul(8)
            .ok_or(LiquidVolumeError::AllocationFailed)?;
        if required > limit {
            return Err(LiquidVolumeError::BufferLimit { required, limit });
        }
        let accepted_bytes = fraction
            .capacity()
            .checked_mul(8)
            .ok_or(LiquidVolumeError::AllocationFailed)?;
        let mut remaining = limit - accepted_bytes;
        let candidate = allocate(grid.cell_len(), &mut remaining)?;
        let transfer = [
            allocate(grid.face_len(Axis::X), &mut remaining)?,
            allocate(grid.face_len(Axis::Y), &mut remaining)?,
            allocate(grid.face_len(Axis::Z), &mut remaining)?,
        ];
        Ok(Self {
            grid,
            density,
            stamp,
            time: 0.0,
            accepted: fraction,
            candidate,
            transfer,
            allocated_bytes: limit - remaining,
        })
    }
    pub fn grid(&self) -> &GridGeometry {
        &self.grid
    }
    pub fn allocated_bytes(&self) -> usize {
        self.allocated_bytes
    }
    pub fn state(&self) -> LiquidVolumeView<'_> {
        LiquidVolumeView {
            fraction: &self.accepted,
            stamp: self.stamp,
            time: self.time,
            density: self.density,
        }
    }
    pub fn occupancy(&self, cell: usize) -> Option<LiquidOccupancy> {
        self.accepted.get(cell).map(|&f| {
            if f == 0.0 {
                LiquidOccupancy::Dry
            } else if f == 1.0 {
                LiquidOccupancy::Full
            } else {
                LiquidOccupancy::Mixed
            }
        })
    }
    /// First-order unsplit donor-cell update of represented liquid amounts. Each
    /// signed face transfer is computed once and reused by adjacent cells.
    /// No fraction clamp, mean correction or implicit substep changes the ledger.
    /// Failure/cancellation preserves accepted fractions, stamp and time; scratch
    /// may be partial. This does not atomically publish a separate fluid solver.
    pub fn advance(
        &mut self,
        flow: LiquidFlowInterval<'_>,
        inlet: LiquidInlet,
        source: Option<LiquidVolumeSource<'_>>,
        settings: LiquidVolumeSettings,
        cancel: impl FnMut(VolumeStage) -> bool,
    ) -> Result<LiquidVolumeReport, LiquidVolumeError> {
        let report = self.prepare_advance(flow, inlet, source, settings, cancel)?;
        self.commit_prepared(&report);
        Ok(report)
    }
    pub(crate) fn validate_advance(
        &self,
        source: Option<LiquidVolumeSource<'_>>,
        settings: LiquidVolumeSettings,
    ) -> Result<(), LiquidVolumeError> {
        if !settings.max_outward_courant.is_finite()
            || settings.max_outward_courant <= 0.0
            || settings.max_outward_courant > 1.0
            || !settings.actual_divergence_limit.is_finite()
            || settings.actual_divergence_limit < 0.0
        {
            return Err(LiquidVolumeError::InvalidSettings);
        }
        if source.is_some_and(|s| s.rate.len() != self.grid.cell_len()) {
            return Err(LiquidVolumeError::LengthMismatch);
        }
        self.stamp
            .version
            .checked_add(1)
            .ok_or(LiquidVolumeError::VersionOverflow)?;
        Ok(())
    }
    pub(crate) fn commit_prepared(&mut self, report: &LiquidVolumeReport) {
        std::mem::swap(&mut self.accepted, &mut self.candidate);
        self.time = report.time;
        self.stamp = report.stamp;
    }
    pub(crate) fn prepare_advance(
        &mut self,
        flow: LiquidFlowInterval<'_>,
        inlet: LiquidInlet,
        source: Option<LiquidVolumeSource<'_>>,
        settings: LiquidVolumeSettings,
        mut cancel: impl FnMut(VolumeStage) -> bool,
    ) -> Result<LiquidVolumeReport, LiquidVolumeError> {
        if flow.grid != &self.grid {
            return Err(LiquidVolumeError::GeometryMismatch);
        }
        if flow.start.to_bits() != self.time.to_bits() {
            return Err(LiquidVolumeError::TimeMismatch);
        }
        self.validate_advance(source, settings)?;
        let next = self.stamp.version + 1;
        checkpoint(&mut cancel, VolumeStage::BeforeFlux)?;
        let h = self.grid.spacing();
        let area = [
            product(h[1], h[2])?,
            product(h[0], h[2])?,
            product(h[0], h[1])?,
        ];
        for axis in Axis::ALL {
            let d = axis.index();
            let dims = self.grid.face_counts(axis);
            for k in 0..dims[2] {
                checkpoint(&mut cancel, VolumeStage::FaceFluxSlice)?;
                for j in 0..dims[1] {
                    for i in 0..dims[0] {
                        let p = [i, j, k];
                        let face = self.grid.face_unchecked(axis, p);
                        let velocity = f64::from(flow.velocity[d][face]);
                        let donor = if velocity > 0.0 {
                            if p[d] == 0 {
                                inlet.fraction[d][0]
                            } else {
                                let mut low = p;
                                low[d] -= 1;
                                self.accepted[self.grid.cell_unchecked(low)]
                            }
                        } else if velocity < 0.0 {
                            if p[d] == self.grid.counts()[d] {
                                inlet.fraction[d][1]
                            } else {
                                self.accepted[self.grid.cell_unchecked(p)]
                            }
                        } else {
                            0.0
                        };
                        self.transfer[d][face] =
                            product(product(product(flow.dt, area[d])?, velocity)?, donor)?;
                    }
                }
            }
        }
        let mut max_courant = 0.0_f64;
        let mut max_divergence = 0.0_f64;
        let volume = self.grid.cell_volume();
        let [nx, ny, nz] = self.grid.counts();
        for k in 0..nz {
            checkpoint(&mut cancel, VolumeStage::CellUpdateSlice)?;
            for j in 0..ny {
                for i in 0..nx {
                    let p = [i, j, k];
                    let cell = self.grid.cell_unchecked(p);
                    let mut amount = product(volume, self.accepted[cell])?;
                    let mut courant = 0.0;
                    let mut div = 0.0;
                    for axis in Axis::ALL {
                        let d = axis.index();
                        let mut hi = p;
                        hi[d] += 1;
                        let low = self.grid.face_unchecked(axis, p);
                        let high = self.grid.face_unchecked(axis, hi);
                        amount += self.transfer[d][low] - self.transfer[d][high];
                        let a = f64::from(flow.velocity[d][low]);
                        let b = f64::from(flow.velocity[d][high]);
                        courant += quotient(product(flow.dt, b.max(0.0) - a.min(0.0))?, h[d])?;
                        div += quotient(b - a, h[d])?;
                    }
                    if !courant.is_finite() || !div.is_finite() {
                        return Err(LiquidVolumeError::ArithmeticFailure);
                    }
                    if courant > settings.max_outward_courant {
                        return Err(LiquidVolumeError::CourantLimit {
                            cell,
                            actual: courant,
                            limit: settings.max_outward_courant,
                        });
                    }
                    max_courant = max_courant.max(courant);
                    let check_divergence = match flow.domain {
                        VolumeDivergenceDomain::AllCells => true,
                        VolumeDivergenceDomain::LiquidSlab { axis, wet_layers } => {
                            p[axis.index()] < wet_layers
                        }
                    };
                    if check_divergence {
                        max_divergence = max_divergence.max(div.abs());
                    }
                    if let Some(s) = source {
                        amount += product(flow.dt, s.rate[cell])?;
                    }
                    let fraction = quotient(amount, volume)?;
                    if !(0.0..=1.0).contains(&fraction) {
                        return Err(LiquidVolumeError::FractionBounds { cell, fraction });
                    }
                    // Reconstructed amounts must also stay within the supported
                    // normal-or-exact-zero scale.
                    product(volume, fraction)?;
                    self.candidate[cell] = fraction;
                }
            }
        }
        if max_divergence > settings.actual_divergence_limit {
            return Err(LiquidVolumeError::DivergenceLimit {
                actual: max_divergence,
                limit: settings.actual_divergence_limit,
            });
        }
        let before = sum(self.accepted.iter().map(|&f| volume * f));
        let after = sum(self.candidate.iter().map(|&f| volume * f));
        let mut inward = Accumulator::default();
        let mut outward = Accumulator::default();
        for axis in Axis::ALL {
            let d = axis.index();
            let dims = self.grid.face_counts(axis);
            for k in 0..dims[2] {
                for j in 0..dims[1] {
                    for i in 0..dims[0] {
                        let p = [i, j, k];
                        if p[d] != 0 && p[d] != self.grid.counts()[d] {
                            continue;
                        }
                        let value = self.transfer[d][self.grid.face_unchecked(axis, p)]
                            * if p[d] == 0 { -1.0 } else { 1.0 };
                        if value < 0.0 {
                            inward.add(-value);
                        } else {
                            outward.add(value);
                        }
                    }
                }
            }
        }
        let inward = inward.total();
        let outward = outward.total();
        let mut source_total = Accumulator::default();
        let mut source_abs = Accumulator::default();
        if let Some(s) = source {
            for &rate in s.rate {
                let v = product(flow.dt, rate)?;
                source_total.add(v);
                source_abs.add(v.abs());
            }
        }
        let source_total = source_total.total();
        let balance = after - before + outward - inward - source_total;
        let budget = (64.0 * f64::EPSILON)
            * (before.abs() + after.abs() + outward + inward + source_abs.total());
        let mass_before = product(self.density, before)?;
        let mass_after = product(self.density, after)?;
        if [
            before,
            after,
            inward,
            outward,
            source_total,
            balance,
            budget,
            mass_before,
            mass_after,
        ]
        .iter()
        .any(|v| !v.is_finite())
        {
            return Err(LiquidVolumeError::ArithmeticFailure);
        }
        if balance.abs() > budget {
            return Err(LiquidVolumeError::BalanceLimit {
                error: balance,
                rounding_budget: budget,
            });
        }
        let mut min = 1.0_f64;
        let mut max = 0.0_f64;
        let mut dry = 0;
        let mut full = 0;
        for &f in &self.candidate {
            min = min.min(f);
            max = max.max(f);
            if f == 0.0 {
                dry += 1;
            }
            if f == 1.0 {
                full += 1;
            }
        }
        checkpoint(&mut cancel, VolumeStage::BeforeCommit)?;
        Ok(LiquidVolumeReport {
            stamp: VolumeStamp {
                id: self.stamp.id,
                version: next,
            },
            flow: flow.stamp,
            inlet: inlet.stamp,
            source: source.map(|s| s.stamp),
            dt: flow.dt,
            time: flow.end,
            liquid_volume_before: before,
            liquid_volume_after: after,
            inward_boundary_volume: inward,
            outward_boundary_volume: outward,
            source_volume: source_total,
            volume_balance_error: balance,
            volume_rounding_budget: budget,
            liquid_mass_before: mass_before,
            liquid_mass_after: mass_after,
            max_outward_courant: max_courant,
            actual_divergence_max: max_divergence,
            divergence_domain: flow.domain,
            fraction_min: min,
            fraction_max: max,
            dry_cells: dry,
            mixed_cells: self.grid.cell_len() - dry - full,
            full_cells: full,
        })
    }
}
pub(crate) fn allocate(n: usize, remaining: &mut usize) -> Result<Vec<f64>, LiquidVolumeError> {
    let required = n
        .checked_mul(8)
        .ok_or(LiquidVolumeError::AllocationFailed)?;
    if required > *remaining {
        return Err(LiquidVolumeError::BufferLimit {
            required,
            limit: *remaining,
        });
    }
    let mut v = Vec::new();
    v.try_reserve_exact(n)
        .map_err(|_| LiquidVolumeError::AllocationFailed)?;
    let bytes = v
        .capacity()
        .checked_mul(8)
        .ok_or(LiquidVolumeError::AllocationFailed)?;
    if bytes > *remaining {
        return Err(LiquidVolumeError::BufferLimit {
            required: bytes,
            limit: *remaining,
        });
    }
    v.resize(n, 0.0);
    *remaining -= bytes;
    Ok(v)
}
fn product(a: f64, b: f64) -> Result<f64, LiquidVolumeError> {
    let v = a * b;
    // Gradual underflow can severely distort a later normal result even when
    // this intermediate rounds to a nonzero subnormal. Reject that scale too.
    if !v.is_finite() || v.is_subnormal() || (v == 0.0 && a != 0.0 && b != 0.0) {
        Err(LiquidVolumeError::ArithmeticFailure)
    } else {
        Ok(v)
    }
}
fn quotient(a: f64, b: f64) -> Result<f64, LiquidVolumeError> {
    let v = a / b;
    if !v.is_finite() || v.is_subnormal() || (v == 0.0 && a != 0.0) {
        Err(LiquidVolumeError::ArithmeticFailure)
    } else {
        Ok(v)
    }
}
fn checkpoint(
    cancel: &mut impl FnMut(VolumeStage) -> bool,
    stage: VolumeStage,
) -> Result<(), LiquidVolumeError> {
    if cancel(stage) {
        Err(LiquidVolumeError::Cancelled { stage })
    } else {
        Ok(())
    }
}
#[derive(Default)]
struct Accumulator {
    sum: f64,
    correction: f64,
}
impl Accumulator {
    fn add(&mut self, value: f64) {
        let next = self.sum + value;
        self.correction += if self.sum.abs() >= value.abs() {
            (self.sum - next) + value
        } else {
            (value - next) + self.sum
        };
        self.sum = next;
    }
    fn total(self) -> f64 {
        self.sum + self.correction
    }
}
fn sum(values: impl Iterator<Item = f64>) -> f64 {
    let mut a = Accumulator::default();
    for v in values {
        a.add(v);
    }
    a.total()
}
