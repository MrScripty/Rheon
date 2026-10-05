//! Resolved, lower-side cell-aligned liquid slab with zero atmospheric pressure.
//! No interface reconstruction, air inertia, traction or mesh-solid mask.
use crate::{Axis, GridGeometry, VolumeStamp};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum FreeSurfaceError {
    InvalidSlab,
    UnsupportedColumn { column: usize },
    UnresolvedColumn { column: usize },
    SteepColumn { column: usize, neighbor: usize },
    ReconstructionCancelled { stage: crate::ReconstructionStage },
    ArithmeticFailure,
    AllocationFailed,
    BufferLimit { required: usize, limit: usize },
    GeometryMismatch,
    DensityMismatch,
    FractionMismatch { cell: usize },
    UnsupportedBoxFlux,
    UnsupportedSmokeSource,
}
impl fmt::Display for FreeSurfaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cell-aligned free surface rejected operation: {self:?}")
    }
}
impl std::error::Error for FreeSurfaceError {}

/// Pressure geometry held over one interval. Liquid occupies lower complete
/// layers; the wet/air interface is at their upper face. Both phases must exist.
/// After transport creates mixed cells, another interval requires reconstruction
/// and is rejected. The stamp is caller-owned, not a reconstructed-surface claim.
#[derive(Debug, Clone, PartialEq)]
pub struct SlabFreeSurface {
    grid: GridGeometry,
    axis: Axis,
    wet_layers: usize,
    stamp: VolumeStamp,
}
impl SlabFreeSurface {
    pub fn new(
        grid: GridGeometry,
        axis: Axis,
        wet_layers: usize,
        stamp: VolumeStamp,
    ) -> Result<Self, FreeSurfaceError> {
        if wet_layers == 0 || wet_layers >= grid.counts()[axis.index()] {
            return Err(FreeSurfaceError::InvalidSlab);
        }
        Ok(Self {
            grid,
            axis,
            wet_layers,
            stamp,
        })
    }
    pub fn grid(&self) -> &GridGeometry {
        &self.grid
    }
    pub fn axis(&self) -> Axis {
        self.axis
    }
    pub fn wet_layers(&self) -> usize {
        self.wet_layers
    }
    pub fn stamp(&self) -> VolumeStamp {
        self.stamp
    }
    pub fn position(&self) -> f64 {
        let d = self.axis.index();
        self.grid.origin()[d] + self.wet_layers as f64 * self.grid.spacing()[d]
    }
    pub fn is_wet_cell(&self, cell: usize) -> bool {
        let [nx, ny, _] = self.grid.counts();
        let strides = [1, nx, nx * ny];
        cell < self.grid.cell_len()
            && (cell / strides[self.axis.index()]) % self.grid.counts()[self.axis.index()]
                < self.wet_layers
    }
    pub fn wet_cells(&self) -> usize {
        self.grid.cell_len() / self.grid.counts()[self.axis.index()] * self.wet_layers
    }
    pub(crate) fn face_active(&self, axis: Axis, p: [usize; 3]) -> bool {
        let d = axis.index();
        if p[d] == 0 || p[d] == self.grid.counts()[d] {
            return false;
        }
        let mut low = p;
        low[d] -= 1;
        self.is_wet_cell(self.grid.cell_unchecked(low))
            || self.is_wet_cell(self.grid.cell_unchecked(p))
    }
    pub(crate) fn validate_fractions(
        &self,
        grid: &GridGeometry,
        fraction: &[f64],
    ) -> Result<(), FreeSurfaceError> {
        if grid != &self.grid || fraction.len() != grid.cell_len() {
            return Err(FreeSurfaceError::GeometryMismatch);
        }
        for (cell, &value) in fraction.iter().enumerate() {
            if value != if self.is_wet_cell(cell) { 1.0 } else { 0.0 } {
                return Err(FreeSurfaceError::FractionMismatch { cell });
            }
        }
        Ok(())
    }
}
