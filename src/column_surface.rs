//! Bounded lower-wall column reconstruction. The column volume is authoritative;
//! unsupported thin, steep or disconnected initial columns are rejected.
use crate::{Axis, FreeSurfaceError, GridGeometry, VolumeStamp};
use std::mem::size_of;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ColumnHeight {
    full: usize,
    fraction: f64,
}
impl ColumnHeight {
    pub fn full_layers(self) -> usize {
        self.full
    }
    pub fn top_fraction(self) -> f64 {
        self.fraction
    }
    pub fn cell_fraction(self, layer: usize) -> f64 {
        if layer < self.full {
            1.0
        } else if layer == self.full {
            self.fraction
        } else {
            0.0
        }
    }
    fn phi(self, layer: usize) -> f64 {
        (layer as f64 - self.full as f64) + (0.5 - self.fraction)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconstructionStage {
    BeforeReconstruction,
    ColumnSlice,
    BeforeAcceptance,
}
#[derive(Debug, Clone, Copy)]
pub struct ColumnReconstructionReport {
    pub input_stamp: VolumeStamp,
    pub output_stamp: VolumeStamp,
    /// Number of columns whose noncanonical raw cells were collapsed to one
    /// bottom-attached region. This is a declared geometry closure, not a clamp.
    pub collapsed_columns: usize,
}
#[derive(Clone, Copy)]
pub struct ColumnSurfaceView<'a> {
    grid: &'a GridGeometry,
    axis: Axis,
    heights: &'a [ColumnHeight],
    stamp: VolumeStamp,
}
impl<'a> ColumnSurfaceView<'a> {
    pub fn grid(self) -> &'a GridGeometry {
        self.grid
    }
    pub fn axis(self) -> Axis {
        self.axis
    }
    pub fn stamp(self) -> VolumeStamp {
        self.stamp
    }
    pub fn heights(self) -> &'a [ColumnHeight] {
        self.heights
    }
    pub fn is_wet_cell(self, cell: usize) -> bool {
        if cell >= self.grid.cell_len() {
            return false;
        }
        let [nx, ny, _] = self.grid.counts();
        let p = [cell % nx, (cell / nx) % ny, cell / (nx * ny)];
        self.phi(p) < 0.0
    }
    pub(crate) fn phi(self, p: [usize; 3]) -> f64 {
        self.heights[column_index(self.grid, self.axis, p)].phi(p[self.axis.index()])
    }
    pub(crate) fn face_active(self, axis: Axis, p: [usize; 3]) -> bool {
        let d = axis.index();
        if p[d] == 0 || p[d] == self.grid.counts()[d] {
            return false;
        }
        let mut low = p;
        low[d] -= 1;
        self.phi(low) < 0.0 || self.phi(p) < 0.0
    }
    pub(crate) fn face_scale(self, axis: Axis, p: [usize; 3]) -> f64 {
        let mut low = p;
        low[axis.index()] -= 1;
        let a = self.phi(low);
        let b = self.phi(p);
        match (a < 0.0, b < 0.0) {
            (true, true) => 1.0,
            (false, false) => 0.0,
            (true, false) => (b - a) / (-a),
            (false, true) => (a - b) / (-b),
        }
    }
    pub(crate) fn matches(self, grid: &GridGeometry, fraction: &[f64], stamp: VolumeStamp) -> bool {
        grid == self.grid
            && stamp == self.stamp
            && fraction.len() == grid.cell_len()
            && fraction.iter().enumerate().all(|(cell, &f)| {
                let [nx, ny, _] = grid.counts();
                let p = [cell % nx, (cell / nx) % ny, cell / (nx * ny)];
                f == self.heights[column_index(grid, self.axis, p)]
                    .cell_fraction(p[self.axis.index()])
            })
    }
}

/// Three explicitly capped derived geometry arrays: accepted end geometry,
/// candidate geometry, and the held geometry of the last published projection.
/// No independent phase authority and no step-time allocation.
pub struct ColumnSurfaceWorkspace {
    grid: GridGeometry,
    axis: Axis,
    accepted: Vec<ColumnHeight>,
    candidate: Vec<ColumnHeight>,
    pressure: Vec<ColumnHeight>,
    stamp: VolumeStamp,
    pressure_stamp: VolumeStamp,
    allocated_bytes: usize,
}
impl ColumnSurfaceWorkspace {
    pub fn new(
        grid: GridGeometry,
        axis: Axis,
        fraction: &[f64],
        stamp: VolumeStamp,
        limit: usize,
    ) -> Result<Self, FreeSurfaceError> {
        if fraction.len() != grid.cell_len() {
            return Err(FreeSurfaceError::GeometryMismatch);
        }
        let columns = grid.cell_len() / grid.counts()[axis.index()];
        let required = columns
            .checked_mul(3)
            .and_then(|n| n.checked_mul(size_of::<ColumnHeight>()))
            .ok_or(FreeSurfaceError::ArithmeticFailure)?;
        if required > limit {
            return Err(FreeSurfaceError::BufferLimit { required, limit });
        }
        let mut remaining = limit;
        let mut accepted = allocate(columns, &mut remaining)?;
        let candidate = allocate(columns, &mut remaining)?;
        let pressure = allocate(columns, &mut remaining)?;
        for (column, height) in accepted.iter_mut().enumerate() {
            *height = canonical_height(&grid, axis, column, fraction)?
                .ok_or(FreeSurfaceError::UnsupportedColumn { column })?;
            validate_height(*height, grid.counts()[axis.index()], column)?;
        }
        validate_slopes(&grid, axis, &accepted)?;
        let mut result = Self {
            grid,
            axis,
            accepted,
            candidate,
            pressure,
            stamp,
            pressure_stamp: stamp,
            allocated_bytes: limit - remaining,
        };
        result.pressure.copy_from_slice(&result.accepted);
        Ok(result)
    }
    pub fn allocated_bytes(&self) -> usize {
        self.allocated_bytes
    }
    pub fn state(&self) -> ColumnSurfaceView<'_> {
        ColumnSurfaceView {
            grid: &self.grid,
            axis: self.axis,
            heights: &self.accepted,
            stamp: self.stamp,
        }
    }
    pub fn pressure_geometry(&self) -> ColumnSurfaceView<'_> {
        ColumnSurfaceView {
            grid: &self.grid,
            axis: self.axis,
            heights: &self.pressure,
            stamp: self.pressure_stamp,
        }
    }
    pub(crate) fn candidate(&self, stamp: VolumeStamp) -> ColumnSurfaceView<'_> {
        ColumnSurfaceView {
            grid: &self.grid,
            axis: self.axis,
            heights: &self.candidate,
            stamp,
        }
    }
    pub(crate) fn prepare(
        &mut self,
        fraction: &mut [f64],
        stamp: VolumeStamp,
        mut cancel: impl FnMut(ReconstructionStage) -> bool,
    ) -> Result<ColumnReconstructionReport, FreeSurfaceError> {
        if fraction.len() != self.grid.cell_len() {
            return Err(FreeSurfaceError::GeometryMismatch);
        }
        check(&mut cancel, ReconstructionStage::BeforeReconstruction)?;
        let mut collapsed = 0;
        for column in 0..self.accepted.len() {
            check(&mut cancel, ReconstructionStage::ColumnSlice)?;
            let height =
                if let Some(height) = canonical_height(&self.grid, self.axis, column, fraction)? {
                    height
                } else {
                    // Integral bottom-fill closure of unresolved vertical donor
                    // distribution. Raw cell bounds have already passed. Preserve
                    // the column amount, then requalify the original volume ledger.
                    let mut sum = 0.0_f64;
                    let mut correction = 0.0_f64;
                    let mut gap = false;
                    for layer in 0..self.grid.counts()[self.axis.index()] {
                        let f = fraction[column_cell(&self.grid, self.axis, column, layer)];
                        if gap && f > 0.0 {
                            return Err(FreeSurfaceError::UnsupportedColumn { column });
                        }
                        gap |= f == 0.0;
                        let next = sum + f;
                        correction += if sum.abs() >= f.abs() {
                            (sum - next) + f
                        } else {
                            (f - next) + sum
                        };
                        sum = next;
                    }
                    let total = sum + correction;
                    if !total.is_finite() || total < 0.0 {
                        return Err(FreeSurfaceError::ArithmeticFailure);
                    }
                    collapsed += 1;
                    ColumnHeight {
                        full: total.floor() as usize,
                        fraction: total - total.floor(),
                    }
                };
            validate_height(height, self.grid.counts()[self.axis.index()], column)?;
            self.candidate[column] = height;
            for layer in 0..self.grid.counts()[self.axis.index()] {
                fraction[column_cell(&self.grid, self.axis, column, layer)] =
                    height.cell_fraction(layer);
            }
        }
        validate_slopes(&self.grid, self.axis, &self.candidate)?;
        check(&mut cancel, ReconstructionStage::BeforeAcceptance)?;
        Ok(ColumnReconstructionReport {
            input_stamp: self.stamp,
            output_stamp: stamp,
            collapsed_columns: collapsed,
        })
    }
    pub(crate) fn commit_column_mac_revision(&mut self, stamp: VolumeStamp) {
        self.pressure.copy_from_slice(&self.accepted);
        self.stamp = stamp;
        self.pressure_stamp = stamp;
    }
    pub(crate) fn commit(&mut self, report: ColumnReconstructionReport) {
        self.pressure.copy_from_slice(&self.accepted);
        self.pressure_stamp = self.stamp;
        std::mem::swap(&mut self.accepted, &mut self.candidate);
        self.stamp = report.output_stamp;
    }
}
fn check(
    cancel: &mut impl FnMut(ReconstructionStage) -> bool,
    stage: ReconstructionStage,
) -> Result<(), FreeSurfaceError> {
    if cancel(stage) {
        Err(FreeSurfaceError::ReconstructionCancelled { stage })
    } else {
        Ok(())
    }
}
fn allocate(n: usize, remaining: &mut usize) -> Result<Vec<ColumnHeight>, FreeSurfaceError> {
    let required = n
        .checked_mul(size_of::<ColumnHeight>())
        .ok_or(FreeSurfaceError::ArithmeticFailure)?;
    if required > *remaining {
        return Err(FreeSurfaceError::BufferLimit {
            required,
            limit: *remaining,
        });
    }
    let mut values = Vec::new();
    values
        .try_reserve_exact(n)
        .map_err(|_| FreeSurfaceError::AllocationFailed)?;
    let bytes = values
        .capacity()
        .checked_mul(size_of::<ColumnHeight>())
        .ok_or(FreeSurfaceError::ArithmeticFailure)?;
    if bytes > *remaining {
        return Err(FreeSurfaceError::BufferLimit {
            required: bytes,
            limit: *remaining,
        });
    }
    values.resize(n, ColumnHeight::default());
    *remaining -= bytes;
    Ok(values)
}
fn canonical_height(
    grid: &GridGeometry,
    axis: Axis,
    column: usize,
    fraction: &[f64],
) -> Result<Option<ColumnHeight>, FreeSurfaceError> {
    let mut height = ColumnHeight::default();
    let mut air = false;
    for layer in 0..grid.counts()[axis.index()] {
        let f = fraction[column_cell(grid, axis, column, layer)];
        if !f.is_finite() || f.is_subnormal() || !(0.0..=1.0).contains(&f) {
            return Err(FreeSurfaceError::UnsupportedColumn { column });
        }
        if air && f != 0.0 {
            return Ok(None);
        }
        if f == 1.0 {
            height.full += 1;
        } else {
            if !air {
                height.fraction = f;
            }
            air = true;
        }
    }
    Ok(Some(height))
}
fn validate_height(
    height: ColumnHeight,
    layers: usize,
    column: usize,
) -> Result<(), FreeSurfaceError> {
    if (height.full == 0 && height.fraction <= 0.5)
        || height.full >= layers
        || (height.full == layers - 1 && height.fraction > 0.5)
    {
        return Err(FreeSurfaceError::UnresolvedColumn { column });
    }
    if !height.fraction.is_finite()
        || height.fraction.is_subnormal()
        || !(0.0..1.0).contains(&height.fraction)
    {
        return Err(FreeSurfaceError::ArithmeticFailure);
    }
    Ok(())
}
fn validate_slopes(
    grid: &GridGeometry,
    axis: Axis,
    heights: &[ColumnHeight],
) -> Result<(), FreeSurfaceError> {
    for (column, &a) in heights.iter().enumerate() {
        let p = column_coordinate(grid, axis, column);
        for d in 0..3 {
            if d == axis.index() || p[d] + 1 >= grid.counts()[d] {
                continue;
            }
            let mut q = p;
            q[d] += 1;
            let other = column_index(grid, axis, q);
            let b = heights[other];
            let difference = (a.full as f64 - b.full as f64) + (a.fraction - b.fraction);
            if difference.abs() > 1.0 {
                return Err(FreeSurfaceError::SteepColumn {
                    column,
                    neighbor: other,
                });
            }
        }
    }
    Ok(())
}
fn column_index(grid: &GridGeometry, axis: Axis, p: [usize; 3]) -> usize {
    let dims = grid.counts();
    let mut index = 0;
    let mut stride = 1;
    for d in 0..3 {
        if d != axis.index() {
            index += stride * p[d];
            stride *= dims[d];
        }
    }
    index
}
fn column_coordinate(grid: &GridGeometry, axis: Axis, column: usize) -> [usize; 3] {
    let mut remainder = column;
    let mut p = [0; 3];
    for (d, value) in p.iter_mut().enumerate() {
        if d != axis.index() {
            *value = remainder % grid.counts()[d];
            remainder /= grid.counts()[d];
        }
    }
    p
}
fn column_cell(grid: &GridGeometry, axis: Axis, column: usize, layer: usize) -> usize {
    let mut p = column_coordinate(grid, axis, column);
    p[axis.index()] = layer;
    grid.cell_unchecked(p)
}

#[derive(Clone, Copy)]
pub(crate) enum PressureSurface<'a> {
    Slab(&'a crate::SlabFreeSurface),
    Columns(ColumnSurfaceView<'a>),
}
impl PressureSurface<'_> {
    pub(crate) fn face_active(self, axis: Axis, p: [usize; 3]) -> bool {
        match self {
            Self::Slab(s) => s.face_active(axis, p),
            Self::Columns(s) => s.face_active(axis, p),
        }
    }
    pub(crate) fn is_wet(self, cell: usize) -> bool {
        match self {
            Self::Slab(s) => s.is_wet_cell(cell),
            Self::Columns(s) => s.is_wet_cell(cell),
        }
    }
    pub(crate) fn scale(self, axis: Axis, p: [usize; 3]) -> f64 {
        match self {
            Self::Slab(_) => 2.0,
            Self::Columns(s) => s.face_scale(axis, p),
        }
    }
}

#[derive(Clone, Copy)]
pub struct ColumnVolumeInputs<'a> {
    pub flow: crate::LiquidFlowInterval<'a>,
    pub inlet: crate::LiquidInlet,
    pub source: Option<crate::LiquidVolumeSource<'a>>,
    pub settings: crate::LiquidVolumeSettings,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnVolumeStage {
    Volume(crate::VolumeStage),
    Reconstruction(ReconstructionStage),
    BeforeCommit,
}
#[derive(Debug, Clone, Copy)]
pub struct ColumnVolumeReport {
    pub volume: crate::LiquidVolumeReport,
    pub geometry: ColumnReconstructionReport,
}
