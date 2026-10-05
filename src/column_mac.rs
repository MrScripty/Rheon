//! Fixed-flat column parcel/MAC bridge. All masses and pressure flux measures
//! derive from the same borrowed geometry. No moving-domain step is supplied.
use crate::{Axis, ColumnSurfaceView, GridGeometry, VolumeStamp};
use std::fmt;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnMacStage {
    BeforeTransfer,
    TransferSlice,
    BeforeAcceptance,
    BeforeTransferCommit,
    PressureIteration,
    BeforeProjectionAcceptance,
    BeforePublish,
}
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnMacError {
    GeometryMismatch,
    UnsupportedHeights,
    UnsupportedState,
    ShapeMismatch,
    InvalidDensity,
    InvalidField,
    ArithmeticFailure,
    AcceptanceFailure,
    StampMismatch,
    VersionOverflow,
    Paused,
    Cancelled { stage: ColumnMacStage },
    AllocationFailed,
    BufferLimit { required: usize, limit: usize },
    Pressure(crate::PressureError),
    Operator(crate::OperatorError),
}
impl fmt::Display for ColumnMacError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "column MAC operation rejected: {self:?}")
    }
}
impl std::error::Error for ColumnMacError {}
impl From<crate::PressureError> for ColumnMacError {
    fn from(e: crate::PressureError) -> Self {
        Self::Pressure(e)
    }
}
impl From<crate::OperatorError> for ColumnMacError {
    fn from(e: crate::OperatorError) -> Self {
        Self::Operator(e)
    }
}
pub(crate) fn check(x: f64) -> Result<f64, ColumnMacError> {
    if x == 0.0 || x.is_normal() {
        Ok(x)
    } else {
        Err(ColumnMacError::ArithmeticFailure)
    }
}
fn mul(a: f64, b: f64) -> Result<f64, ColumnMacError> {
    let x = check(a * b)?;
    if a != 0.0 && b != 0.0 && x == 0.0 {
        Err(ColumnMacError::ArithmeticFailure)
    } else {
        Ok(x)
    }
}
fn div(a: f64, b: f64) -> Result<f64, ColumnMacError> {
    if !b.is_normal() || b == 0.0 {
        return Err(ColumnMacError::ArithmeticFailure);
    }
    let x = check(a / b)?;
    if a != 0.0 && x == 0.0 {
        Err(ColumnMacError::ArithmeticFailure)
    } else {
        Ok(x)
    }
}
fn stored(x: f64) -> Result<f64, ColumnMacError> {
    check(x)?;
    let v = x as f32;
    if !(v == 0.0 || v.is_normal()) || (v == 0.0 && x != 0.0) {
        Err(ColumnMacError::ArithmeticFailure)
    } else {
        Ok(f64::from(v))
    }
}
#[derive(Default)]
struct Sum {
    value: f64,
    correction: f64,
}
impl Sum {
    fn add(&mut self, x: f64) -> Result<(), ColumnMacError> {
        let next = check(self.value + x)?;
        let error = if self.value.abs() >= x.abs() {
            check(check(self.value - next)? + x)?
        } else {
            check(check(x - next)? + self.value)?
        };
        self.correction = check(self.correction + error)?;
        self.value = next;
        Ok(())
    }
    fn finish(self) -> Result<f64, ColumnMacError> {
        check(self.value + self.correction)
    }
}
fn checkpoint(
    cancel: &mut impl FnMut(ColumnMacStage) -> bool,
    stage: ColumnMacStage,
) -> Result<(), ColumnMacError> {
    if cancel(stage) {
        Err(ColumnMacError::Cancelled { stage })
    } else {
        Ok(())
    }
}
fn coordinate(index: usize, n: [usize; 3]) -> [usize; 3] {
    [index % n[0], (index / n[0]) % n[1], index / (n[0] * n[1])]
}
/// A borrowed fixed-flat liquid control-volume geometry. No accepted-state or
/// cached mass arrays are owned here. Air-centered caps belong to the last wet
/// dual slab, matching the parcel representation rather than full grid cells.
#[derive(Clone, Copy)]
pub struct FlatColumnMacGeometry<'a> {
    surface: ColumnSurfaceView<'a>,
    density: f64,
    wet: usize,
    height: f64,
    area: f64,
    rho_area: f64,
}
impl<'a> FlatColumnMacGeometry<'a> {
    pub fn new(surface: ColumnSurfaceView<'a>, density: f64) -> Result<Self, ColumnMacError> {
        if !density.is_normal() || density <= 0.0 {
            return Err(ColumnMacError::InvalidDensity);
        }
        let first = surface.heights()[0];
        if surface.heights().iter().any(|v| {
            v.full_layers() != first.full_layers() || v.top_fraction() != first.top_fraction()
        }) {
            return Err(ColumnMacError::UnsupportedHeights);
        }
        let normal = surface.axis().index();
        let spacing = surface.grid().spacing();
        let height = mul(
            check(first.full_layers() as f64 + first.top_fraction())?,
            spacing[normal],
        )?;
        let mut area = 1.0;
        for (d, h) in spacing.into_iter().enumerate() {
            if d != normal {
                area = mul(area, h)?;
            }
        }
        let rho_area = mul(density, area)?;
        let result = Self {
            surface,
            density,
            wet: first.full_layers() + usize::from(first.top_fraction() > 0.5),
            height,
            area,
            rho_area,
        };
        for cell in 0..surface.grid().cell_len() {
            let volume = result.cell_volume(cell).unwrap();
            let mass = result.cell_mass(cell).unwrap();
            check(volume)?;
            check(mass)?;
            if result.is_wet(cell) && !(volume > 0.0 && mass > 0.0) {
                return Err(ColumnMacError::ArithmeticFailure);
            }
        }
        for axis in Axis::ALL {
            for index in 0..surface.grid().face_len(axis) {
                let p = coordinate(index, surface.grid().face_counts(axis));
                let mass = result.face_mass(axis, p).unwrap();
                let q = result.flux_area(axis, p).unwrap();
                check(mass)?;
                check(q)?;
                if surface.face_active(axis, p) && !(mass > 0.0 && q > 0.0) {
                    return Err(ColumnMacError::ArithmeticFailure);
                }
                if q > 0.0 {
                    let a = div(q, mass)?;
                    mul(q, a)?;
                }
            }
        }
        Ok(result)
    }
    pub(crate) fn surface_view(self) -> ColumnSurfaceView<'a> {
        self.surface
    }
    pub fn grid(self) -> &'a GridGeometry {
        self.surface.grid()
    }
    pub fn stamp(self) -> VolumeStamp {
        self.surface.stamp()
    }
    pub fn normal(self) -> Axis {
        self.surface.axis()
    }
    pub fn density(self) -> f64 {
        self.density
    }
    pub fn height(self) -> f64 {
        self.height
    }
    pub fn wet_nodes(self) -> usize {
        self.wet
    }
    pub fn dual_length(self, layer: usize) -> Option<f64> {
        if layer >= self.wet {
            None
        } else {
            Some(if layer + 1 == self.wet {
                self.height - layer as f64 * self.grid().spacing()[self.normal().index()]
            } else {
                self.grid().spacing()[self.normal().index()]
            })
        }
    }
    pub(crate) fn is_wet(self, cell: usize) -> bool {
        coordinate(cell, self.grid().counts())[self.normal().index()] < self.wet
    }
    pub fn cell_volume(self, cell: usize) -> Option<f64> {
        if cell >= self.grid().cell_len() {
            return None;
        }
        Some(
            self.area
                * self
                    .dual_length(coordinate(cell, self.grid().counts())[self.normal().index()])
                    .unwrap_or(0.0),
        )
    }
    pub fn cell_mass(self, cell: usize) -> Option<f64> {
        self.cell_volume(cell)?;
        Some(
            self.rho_area
                * self
                    .dual_length(coordinate(cell, self.grid().counts())[self.normal().index()])
                    .unwrap_or(0.0),
        )
    }
    /// Includes fixed-wall dual mass; fixed wall velocity is always zero.
    pub fn face_mass(self, axis: Axis, p: [usize; 3]) -> Option<f64> {
        self.grid().face_index(axis, p)?;
        let normal = self.normal().index();
        let d = axis.index();
        let h = self.grid().spacing()[normal];
        Some(if d == normal {
            let length = if p[d] == 0 {
                0.5 * h
            } else if p[d] < self.wet {
                h
            } else if p[d] == self.wet {
                self.height - (self.wet as f64 - 0.5) * h
            } else {
                0.0
            };
            self.rho_area * length
        } else {
            let length = self.dual_length(p[normal]).unwrap_or(0.0);
            self.rho_area
                * length
                * if p[d] == 0 || p[d] == self.grid().counts()[d] {
                    0.5
                } else {
                    1.0
                }
        })
    }
    /// Integrated divergence/gradient face measure. Outer sealed faces and
    /// inactive air have zero flux; the top pressure face is a virtual surface.
    pub fn flux_area(self, axis: Axis, p: [usize; 3]) -> Option<f64> {
        self.grid().face_index(axis, p)?;
        let d = axis.index();
        if p[d] == 0 || p[d] == self.grid().counts()[d] {
            return Some(0.0);
        }
        let normal = self.normal().index();
        Some(if d == normal {
            if p[d] <= self.wet { self.area } else { 0.0 }
        } else {
            let other = (0..3).find(|&x| x != d && x != normal).unwrap();
            self.grid().spacing()[other] * self.dual_length(p[normal]).unwrap_or(0.0)
        })
    }
    pub(crate) fn acceleration(self, axis: Axis, p: [usize; 3]) -> f64 {
        let q = self.flux_area(axis, p).unwrap();
        if q == 0.0 {
            0.0
        } else {
            q / self.face_mass(axis, p).unwrap()
        }
    }
    pub(crate) fn weight(self, axis: Axis, p: [usize; 3]) -> f64 {
        self.flux_area(axis, p).unwrap() * self.acceleration(axis, p)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnMacDirection {
    LiftToMac,
    RestrictTangents,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColumnMacTransferReport {
    pub geometry: VolumeStamp,
    pub direction: ColumnMacDirection,
    pub profile_mass: f64,
    pub mac_mass: [f64; 3],
    pub mass_error: [f64; 3],
    pub mass_budget: f64,
    pub momentum_before: [f64; 2],
    pub momentum_after: [f64; 2],
    pub wall_impulse: [f64; 2],
    pub rounding_momentum: [f64; 2],
    pub momentum_error: [f64; 2],
    pub momentum_budget: [f64; 2],
    pub kinetic_before: f64,
    pub kinetic_after: f64,
    pub wall_energy_removed: f64,
    pub mixing_loss: f64,
    pub rounding_work: f64,
    pub energy_error: f64,
    pub energy_budget: f64,
    pub omitted_normal_momentum: f64,
    pub omitted_normal_energy: f64,
    pub workspace_bytes: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnMacStateStamp {
    pub carrier: VolumeStamp,
    pub volume: VolumeStamp,
}
#[derive(Clone, Copy)]
pub struct ColumnMacPublishInputs<'a> {
    pub expected: ColumnMacStateStamp,
    pub profiles: [&'a [f32]; 2],
    /// Scales pressure in this instantaneous projection, not an advanced clock.
    pub projection_dt: f64,
}
#[derive(Debug, Clone, Copy)]
pub struct ColumnMacProjectionReport {
    pub pressure: crate::PressureReport,
    pub actual_divergence_max: f64,
    pub kinetic_before: f64,
    pub kinetic_after: f64,
    pub correction_energy: f64,
    pub residual_work: f64,
    pub rounding_work: f64,
    pub energy_error: f64,
    pub energy_budget: f64,
    pub momentum_before: [f64; 3],
    pub momentum_after: [f64; 3],
    pub pressure_impulse: [f64; 3],
    pub rounding_momentum: [f64; 3],
    pub momentum_error: [f64; 3],
    pub momentum_budget: [f64; 3],
}
#[derive(Debug, Clone, Copy)]
pub struct ColumnMacPublicationReport {
    pub before: ColumnMacStateStamp,
    pub after: ColumnMacStateStamp,
    pub transfer: ColumnMacTransferReport,
    pub projection: ColumnMacProjectionReport,
    pub prescribed_momentum_change: [f64; 3],
    pub prescribed_energy_change: f64,
    pub accepted_momentum_before: [f64; 3],
    pub accepted_energy_before: f64,
    pub represented_phase_mass: f64,
    pub phase_mass_error: f64,
    pub phase_mass_budget: f64,
    pub owned_array_bytes: usize,
    pub workspace_array_bytes: usize,
    pub total_array_bytes: usize,
}
#[derive(Debug, Clone, Copy)]
pub struct ColumnMacExportReport {
    pub source: ColumnMacStateStamp,
    pub transfer: ColumnMacTransferReport,
}
/// Three capped face-count f64 staging vectors (8F bytes nominal). Restriction
/// reuses their first cell-count entries. No per-call arrays or accepted owner.
pub struct ColumnMacWorkspace {
    grid: GridGeometry,
    candidate: [Vec<f64>; 3],
    allocated_bytes: usize,
}
#[derive(Default)]
struct Ledger {
    before: [Sum; 2],
    after: [Sum; 2],
    wall: [Sum; 2],
    round: [Sum; 2],
    absolute: [Sum; 2],
    energy_before: Sum,
    energy_after: Sum,
    wall_energy: Sum,
    mixing: Sum,
    work: Sum,
    absolute_work: Sum,
}
impl Ledger {
    fn parcel(&mut self, t: usize, m: f64, u: f64, before: bool) -> Result<(), ColumnMacError> {
        let p = mul(m, u)?;
        let e = mul(0.5, mul(m, mul(u, u)?)?)?;
        if before {
            self.before[t].add(p)?;
            self.energy_before.add(e)?;
        } else {
            self.after[t].add(p)?;
            self.energy_after.add(e)?;
        }
        self.absolute[t].add(p.abs())
    }
    fn candidate(&mut self, t: usize, m: f64, raw: f64, v: f64) -> Result<(), ColumnMacError> {
        self.parcel(t, m, v, false)?;
        let r = check(v - raw)?;
        let p = mul(m, r)?;
        self.round[t].add(p)?;
        self.absolute[t].add(p.abs())?;
        let work = mul(m, mul(r, check(raw + mul(0.5, r)?)?)?)?;
        self.work.add(work)?;
        self.absolute_work.add(work.abs())
    }
    fn mix(&mut self, m: f64, u: f64, raw: f64) -> Result<(), ColumnMacError> {
        let difference = check(u - raw)?;
        self.mixing
            .add(mul(0.5, mul(m, mul(difference, difference)?)?)?)
    }
}
impl ColumnMacWorkspace {
    pub fn new(grid: GridGeometry, limit: usize) -> Result<Self, ColumnMacError> {
        let f = Axis::ALL.into_iter().try_fold(0usize, |n, a| {
            n.checked_add(grid.face_len(a))
                .ok_or(ColumnMacError::AllocationFailed)
        })?;
        let required = f.checked_mul(8).ok_or(ColumnMacError::AllocationFailed)?;
        if required > limit {
            return Err(ColumnMacError::BufferLimit { required, limit });
        }
        let mut remaining = limit;
        let mut allocate = |axis: Axis| {
            let mut v = Vec::new();
            v.try_reserve_exact(grid.face_len(axis))
                .map_err(|_| ColumnMacError::AllocationFailed)?;
            let bytes = v
                .capacity()
                .checked_mul(8)
                .ok_or(ColumnMacError::AllocationFailed)?;
            if bytes > remaining {
                return Err(ColumnMacError::BufferLimit {
                    required: bytes,
                    limit: remaining,
                });
            }
            remaining -= bytes;
            v.resize(grid.face_len(axis), 0.0);
            Ok(v)
        };
        let candidate = [allocate(Axis::X)?, allocate(Axis::Y)?, allocate(Axis::Z)?];
        Ok(Self {
            grid,
            candidate,
            allocated_bytes: limit - remaining,
        })
    }
    pub fn grid(&self) -> &GridGeometry {
        &self.grid
    }
    pub fn allocated_bytes(&self) -> usize {
        self.allocated_bytes
    }
    fn validate_geometry(&self, geometry: FlatColumnMacGeometry<'_>) -> Result<(), ColumnMacError> {
        if geometry.grid() != &self.grid {
            Err(ColumnMacError::GeometryMismatch)
        } else {
            Ok(())
        }
    }
    fn validate_fields(
        &self,
        geometry: FlatColumnMacGeometry<'_>,
        v: [&[f32]; 3],
    ) -> Result<(), ColumnMacError> {
        for (d, field) in v.into_iter().enumerate() {
            let axis = Axis::ALL[d];
            if field.len() != self.grid.face_len(axis) {
                return Err(ColumnMacError::ShapeMismatch);
            }
            for (index, &u) in field.iter().enumerate() {
                if !(u == 0.0 || u.is_normal()) {
                    return Err(ColumnMacError::InvalidField);
                }
                let p = coordinate(index, self.grid.face_counts(axis));
                if (geometry.flux_area(axis, p).unwrap() == 0.0) && u != 0.0 {
                    return Err(ColumnMacError::InvalidField);
                }
            }
        }
        Ok(())
    }
    pub fn lift(
        &mut self,
        geometry: FlatColumnMacGeometry<'_>,
        profiles: [&[f32]; 2],
        output: [&mut [f32]; 3],
        mut cancel: impl FnMut(ColumnMacStage) -> bool,
    ) -> Result<ColumnMacTransferReport, ColumnMacError> {
        self.validate_geometry(geometry)?;
        let tangents = match geometry.normal() {
            Axis::X => [1, 2],
            Axis::Y => [0, 2],
            Axis::Z => [0, 1],
        };
        if profiles.iter().any(|v| v.len() != self.grid.cell_len())
            || output
                .iter()
                .enumerate()
                .any(|(d, v)| v.len() != self.grid.face_len(Axis::ALL[d]))
        {
            return Err(ColumnMacError::ShapeMismatch);
        }
        for v in profiles {
            for (index, &u) in v.iter().enumerate() {
                if !(u == 0.0 || u.is_normal()) || (!geometry.is_wet(index) && u != 0.0) {
                    return Err(ColumnMacError::InvalidField);
                }
            }
        }
        checkpoint(&mut cancel, ColumnMacStage::BeforeTransfer)?;
        for v in &mut self.candidate {
            v.fill(0.0);
        }
        let mut ledger = Ledger::default();
        for (t, &d) in tangents.iter().enumerate() {
            for (index, &u) in profiles[t].iter().enumerate() {
                let mass = geometry.cell_mass(index).unwrap();
                if mass == 0.0 {
                    continue;
                }
                let p = coordinate(index, self.grid.counts());
                let u = f64::from(u);
                ledger.parcel(t, mass, u, true)?;
                for wall in [p[d] == 0, p[d] + 1 == self.grid.counts()[d]] {
                    if wall {
                        let half = mul(0.5, mass)?;
                        let impulse = mul(-half, u)?;
                        ledger.wall[t].add(impulse)?;
                        ledger.absolute[t].add(impulse.abs())?;
                        ledger.wall_energy.add(mul(0.5, mul(half, mul(u, u)?)?)?)?;
                    }
                }
            }
            let axis = Axis::ALL[d];
            for index in 0..self.grid.face_len(axis) {
                checkpoint(&mut cancel, ColumnMacStage::TransferSlice)?;
                let p = coordinate(index, self.grid.face_counts(axis));
                if geometry.flux_area(axis, p).unwrap() == 0.0 {
                    continue;
                }
                let mut low = p;
                low[d] -= 1;
                let a = f64::from(profiles[t][self.grid.cell_index(low).unwrap()]);
                let b = f64::from(profiles[t][self.grid.cell_index(p).unwrap()]);
                let raw = if a == b { a } else { mul(0.5, check(a + b)?)? };
                let v = stored(raw)?;
                if !(a.min(b) <= raw && raw <= a.max(b) && a.min(b) <= v && v <= a.max(b)) {
                    return Err(ColumnMacError::AcceptanceFailure);
                }
                let m = geometry.face_mass(axis, p).unwrap();
                self.candidate[d][index] = v;
                ledger.candidate(t, m, raw, v)?;
                ledger.mix(mul(0.5, m)?, a, raw)?;
                ledger.mix(mul(0.5, m)?, b, raw)?;
            }
        }
        checkpoint(&mut cancel, ColumnMacStage::BeforeAcceptance)?;
        let report = self.finish(geometry, ColumnMacDirection::LiftToMac, ledger, 0.0, 0.0)?;
        checkpoint(&mut cancel, ColumnMacStage::BeforeTransferCommit)?;
        for (d, out) in output.into_iter().enumerate() {
            for (value, &u) in out.iter_mut().zip(&self.candidate[d]) {
                *value = u as f32;
            }
        }
        Ok(report)
    }
    pub fn restrict(
        &mut self,
        geometry: FlatColumnMacGeometry<'_>,
        velocity: [&[f32]; 3],
        output: [&mut [f32]; 2],
        mut cancel: impl FnMut(ColumnMacStage) -> bool,
    ) -> Result<ColumnMacTransferReport, ColumnMacError> {
        self.validate_geometry(geometry)?;
        self.validate_fields(geometry, velocity)?;
        if output.iter().any(|v| v.len() != self.grid.cell_len()) {
            return Err(ColumnMacError::ShapeMismatch);
        }
        checkpoint(&mut cancel, ColumnMacStage::BeforeTransfer)?;
        let tangents = match geometry.normal() {
            Axis::X => [1, 2],
            Axis::Y => [0, 2],
            Axis::Z => [0, 1],
        };
        let mut ledger = Ledger::default();
        let mut omitted_momentum = Sum::default();
        let mut omitted_energy = Sum::default();
        for (d, field) in velocity.into_iter().enumerate() {
            for (index, &u) in field.iter().enumerate() {
                let p = coordinate(index, self.grid.face_counts(Axis::ALL[d]));
                let m = geometry.face_mass(Axis::ALL[d], p).unwrap();
                let u = f64::from(u);
                if d == geometry.normal().index() {
                    omitted_momentum.add(mul(m, u)?)?;
                    omitted_energy.add(mul(0.5, mul(m, mul(u, u)?)?)?)?;
                } else {
                    let t = usize::from(d == tangents[1]);
                    ledger.parcel(t, m, u, true)?;
                }
            }
        }
        for (t, &d) in tangents.iter().enumerate() {
            self.candidate[d].fill(0.0);
            for index in 0..self.grid.cell_len() {
                checkpoint(&mut cancel, ColumnMacStage::TransferSlice)?;
                let m = geometry.cell_mass(index).unwrap();
                if m == 0.0 {
                    continue;
                }
                let p = coordinate(index, self.grid.counts());
                let mut high = p;
                high[d] += 1;
                let a = f64::from(velocity[d][self.grid.face_index(Axis::ALL[d], p).unwrap()]);
                let b = f64::from(velocity[d][self.grid.face_index(Axis::ALL[d], high).unwrap()]);
                let raw = if a == b { a } else { mul(0.5, check(a + b)?)? };
                let v = stored(raw)?;
                self.candidate[d][index] = v;
                ledger.candidate(t, m, raw, v)?;
                ledger.mix(mul(0.5, m)?, a, raw)?;
                ledger.mix(mul(0.5, m)?, b, raw)?;
            }
        }
        checkpoint(&mut cancel, ColumnMacStage::BeforeAcceptance)?;
        let report = self.finish(
            geometry,
            ColumnMacDirection::RestrictTangents,
            ledger,
            omitted_momentum.finish()?,
            omitted_energy.finish()?,
        )?;
        checkpoint(&mut cancel, ColumnMacStage::BeforeTransferCommit)?;
        for (t, out) in output.into_iter().enumerate() {
            for (value, &u) in out.iter_mut().zip(&self.candidate[tangents[t]]) {
                *value = u as f32;
            }
        }
        Ok(report)
    }
    fn finish(
        &self,
        geometry: FlatColumnMacGeometry<'_>,
        direction: ColumnMacDirection,
        ledger: Ledger,
        omitted_normal_momentum: f64,
        omitted_normal_energy: f64,
    ) -> Result<ColumnMacTransferReport, ColumnMacError> {
        let mut profile_mass = Sum::default();
        for index in 0..self.grid.cell_len() {
            profile_mass.add(geometry.cell_mass(index).unwrap())?;
        }
        let profile_mass = profile_mass.finish()?;
        let mass_budget = mul(64.0 * f64::EPSILON, mul(2.0, profile_mass)?)?;
        let mut mac_mass = [0.0; 3];
        let mut mass_error = [0.0; 3];
        for d in 0..3 {
            let mut sum = Sum::default();
            for index in 0..self.grid.face_len(Axis::ALL[d]) {
                sum.add(
                    geometry
                        .face_mass(
                            Axis::ALL[d],
                            coordinate(index, self.grid.face_counts(Axis::ALL[d])),
                        )
                        .unwrap(),
                )?;
            }
            mac_mass[d] = sum.finish()?;
            mass_error[d] = check(mac_mass[d] - profile_mass)?;
            if mass_error[d].abs() > mass_budget {
                return Err(ColumnMacError::AcceptanceFailure);
            }
        }
        let mut before = [0.0; 2];
        let mut after = [0.0; 2];
        let mut wall = [0.0; 2];
        let mut round = [0.0; 2];
        let mut error = [0.0; 2];
        let mut budget = [0.0; 2];
        let Ledger {
            before: pb,
            after: pa,
            wall: pw,
            round: pr,
            absolute: ab,
            energy_before,
            energy_after,
            wall_energy,
            mixing,
            work,
            absolute_work,
        } = ledger;
        for (t, ((((b, a), w), r), absolute)) in
            pb.into_iter().zip(pa).zip(pw).zip(pr).zip(ab).enumerate()
        {
            before[t] = b.finish()?;
            after[t] = a.finish()?;
            wall[t] = w.finish()?;
            round[t] = r.finish()?;
            error[t] = check(after[t] - before[t] - wall[t] - round[t])?;
            budget[t] = mul(64.0 * f64::EPSILON, absolute.finish()?)?;
            if error[t].abs() > budget[t] {
                return Err(ColumnMacError::AcceptanceFailure);
            }
        }
        let kinetic_before = energy_before.finish()?;
        let kinetic_after = energy_after.finish()?;
        let wall_energy_removed = wall_energy.finish()?;
        let mixing_loss = mixing.finish()?;
        let rounding_work = work.finish()?;
        let absolute_work = absolute_work.finish()?;
        let energy_error = check(
            kinetic_after - kinetic_before + wall_energy_removed + mixing_loss - rounding_work,
        )?;
        let floating = mul(
            64.0 * f64::EPSILON,
            check(
                kinetic_before + kinetic_after + wall_energy_removed + mixing_loss + absolute_work,
            )?,
        )?;
        let energy_budget = check(absolute_work + floating)?;
        if energy_error.abs() > floating
            || kinetic_after - kinetic_before + wall_energy_removed > energy_budget
        {
            return Err(ColumnMacError::AcceptanceFailure);
        }
        Ok(ColumnMacTransferReport {
            geometry: geometry.stamp(),
            direction,
            profile_mass,
            mac_mass,
            mass_error,
            mass_budget,
            momentum_before: before,
            momentum_after: after,
            wall_impulse: wall,
            rounding_momentum: round,
            momentum_error: error,
            momentum_budget: budget,
            kinetic_before,
            kinetic_after,
            wall_energy_removed,
            mixing_loss,
            rounding_work,
            energy_error,
            energy_budget,
            omitted_normal_momentum,
            omitted_normal_energy,
            workspace_bytes: self.allocated_bytes,
        })
    }
}
impl ColumnMacWorkspace {
    pub(crate) fn measure_velocity(
        &self,
        geometry: FlatColumnMacGeometry<'_>,
        velocity: [&[f32]; 3],
    ) -> Result<([f64; 3], f64), ColumnMacError> {
        self.validate_geometry(geometry)?;
        self.validate_fields(geometry, velocity)?;
        let mut momentum: [Sum; 3] = std::array::from_fn(|_| Sum::default());
        let mut energy = Sum::default();
        for (d, v) in velocity.into_iter().enumerate() {
            for (index, &u) in v.iter().enumerate() {
                let mass = geometry
                    .face_mass(
                        Axis::ALL[d],
                        coordinate(index, self.grid.face_counts(Axis::ALL[d])),
                    )
                    .unwrap();
                let u = f64::from(u);
                momentum[d].add(mul(mass, u)?)?;
                energy.add(mul(0.5, mul(mass, mul(u, u)?)?)?)?;
            }
        }
        let mut p = [0.0; 3];
        for (d, s) in momentum.into_iter().enumerate() {
            p[d] = s.finish()?;
        }
        Ok((p, energy.finish()?))
    }
    fn raw_face(
        &self,
        geometry: FlatColumnMacGeometry<'_>,
        axis: Axis,
        p: [usize; 3],
        pressure: &[f64],
        dt: f64,
    ) -> Result<(f64, f64), ColumnMacError> {
        let d = axis.index();
        let old = self.candidate[d][self.grid.face_index(axis, p).unwrap()];
        let acceleration = geometry.acceleration(axis, p);
        if acceleration == 0.0 {
            return Ok((old, 0.0));
        }
        let mut low = p;
        low[d] -= 1;
        let a = self.grid.cell_index(low).unwrap();
        let b = self.grid.cell_index(p).unwrap();
        let pa = if geometry.is_wet(a) { pressure[a] } else { 0.0 };
        let pb = if geometry.is_wet(b) { pressure[b] } else { 0.0 };
        let delta = mul(-dt, mul(acceleration, check(pb - pa)?)?)?;
        Ok((check(old + delta)?, delta))
    }
    pub(crate) fn qualify_projection(
        &self,
        geometry: FlatColumnMacGeometry<'_>,
        pressure: &[f64],
        dt: f64,
        velocity: [&[f32]; 3],
        pressure_report: crate::PressureReport,
        actual_divergence_max: f64,
    ) -> Result<ColumnMacProjectionReport, ColumnMacError> {
        self.validate_fields(geometry, velocity)?;
        if pressure.len() != self.grid.cell_len()
            || pressure.iter().any(|&p| !(p == 0.0 || p.is_normal()))
        {
            return Err(ColumnMacError::InvalidField);
        }
        if !dt.is_normal() || dt <= 0.0 {
            return Err(ColumnMacError::ArithmeticFailure);
        }
        let mut before: [Sum; 3] = std::array::from_fn(|_| Sum::default());
        let mut after: [Sum; 3] = std::array::from_fn(|_| Sum::default());
        let mut impulse: [Sum; 3] = std::array::from_fn(|_| Sum::default());
        let mut rounding: [Sum; 3] = std::array::from_fn(|_| Sum::default());
        let mut absolute: [Sum; 3] = std::array::from_fn(|_| Sum::default());
        let mut eb = Sum::default();
        let mut ea = Sum::default();
        let mut correction = Sum::default();
        let mut work = Sum::default();
        let mut absolute_work = Sum::default();
        for (d, values) in velocity.into_iter().enumerate() {
            let axis = Axis::ALL[d];
            for (index, &value) in values.iter().enumerate() {
                let p = coordinate(index, self.grid.face_counts(axis));
                let mass = geometry.face_mass(axis, p).unwrap();
                let old = self.candidate[d][index];
                let (raw, delta) = self.raw_face(geometry, axis, p, pressure, dt)?;
                let new = f64::from(value);
                if new != stored(raw)? {
                    return Err(ColumnMacError::AcceptanceFailure);
                }
                let r = check(new - raw)?;
                let po = mul(mass, old)?;
                let pn = mul(mass, new)?;
                let pi = mul(mass, delta)?;
                let pr = mul(mass, r)?;
                before[d].add(po)?;
                after[d].add(pn)?;
                impulse[d].add(pi)?;
                rounding[d].add(pr)?;
                absolute[d].add(check(po.abs() + pn.abs() + pi.abs() + pr.abs())?)?;
                eb.add(mul(0.5, mul(mass, mul(old, old)?)?)?)?;
                ea.add(mul(0.5, mul(mass, mul(new, new)?)?)?)?;
                correction.add(mul(0.5, mul(mass, mul(delta, delta)?)?)?)?;
                let wr = mul(mass, mul(r, check(raw + mul(0.5, r)?)?)?)?;
                work.add(wr)?;
                absolute_work.add(wr.abs())?;
            }
        }
        // Independent cell-side residual work using the same integrated face
        // measure. No raw-velocity or residual array is allocated.
        let mut residual = Sum::default();
        for (cell, &pvalue) in pressure.iter().enumerate() {
            if !geometry.is_wet(cell) {
                if pvalue != 0.0 {
                    return Err(ColumnMacError::InvalidField);
                }
                continue;
            }
            let p = coordinate(cell, self.grid.counts());
            let mut flux = Sum::default();
            for axis in Axis::ALL {
                let d = axis.index();
                let mut high = p;
                high[d] += 1;
                let lo = self.raw_face(geometry, axis, p, pressure, dt)?.0;
                let hi = self.raw_face(geometry, axis, high, pressure, dt)?.0;
                flux.add(check(
                    mul(geometry.flux_area(axis, p).unwrap(), lo)?
                        - mul(geometry.flux_area(axis, high).unwrap(), hi)?,
                )?)?;
            }
            residual.add(mul(-dt, mul(pvalue, flux.finish()?)?)?)?;
        }
        let kinetic_before = eb.finish()?;
        let kinetic_after = ea.finish()?;
        let correction_energy = correction.finish()?;
        let residual_work = residual.finish()?;
        let rounding_work = work.finish()?;
        let absolute_work = absolute_work.finish()?;
        let energy_error = check(
            kinetic_after - kinetic_before + correction_energy - residual_work - rounding_work,
        )?;
        let floating = mul(
            64.0 * f64::EPSILON,
            check(
                kinetic_before
                    + kinetic_after
                    + correction_energy
                    + residual_work.abs()
                    + absolute_work,
            )?,
        )?;
        let energy_budget = check(residual_work.max(0.0) + absolute_work + floating)?;
        if energy_error.abs() > floating || kinetic_after - kinetic_before > energy_budget {
            return Err(ColumnMacError::AcceptanceFailure);
        }
        let mut momentum_before = [0.0; 3];
        let mut momentum_after = [0.0; 3];
        let mut pressure_impulse = [0.0; 3];
        let mut rounding_momentum = [0.0; 3];
        let mut momentum_error = [0.0; 3];
        let mut momentum_budget = [0.0; 3];
        for (d, ((((b, a), i), r), ab)) in before
            .into_iter()
            .zip(after)
            .zip(impulse)
            .zip(rounding)
            .zip(absolute)
            .enumerate()
        {
            momentum_before[d] = b.finish()?;
            momentum_after[d] = a.finish()?;
            pressure_impulse[d] = i.finish()?;
            rounding_momentum[d] = r.finish()?;
            momentum_error[d] = check(
                momentum_after[d] - momentum_before[d] - pressure_impulse[d] - rounding_momentum[d],
            )?;
            momentum_budget[d] = mul(64.0 * f64::EPSILON, ab.finish()?)?;
            if momentum_error[d].abs() > momentum_budget[d] {
                return Err(ColumnMacError::AcceptanceFailure);
            }
        }
        Ok(ColumnMacProjectionReport {
            pressure: pressure_report,
            actual_divergence_max,
            kinetic_before,
            kinetic_after,
            correction_energy,
            residual_work,
            rounding_work,
            energy_error,
            energy_budget,
            momentum_before,
            momentum_after,
            pressure_impulse,
            rounding_momentum,
            momentum_error,
            momentum_budget,
        })
    }
}
