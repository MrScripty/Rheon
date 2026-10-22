//! Components for the filled, incompressible two-material MAC model.
//!
//! These numerical components are not a complete native simulation mode. The
//! coupled transport/stress step and authoring/recording gates are tracked in
//! `docs/plans/voxel-multimaterial-initial-authoring/plan.md`. No vacuum, EOS,
//! thermal evolution, phase change or surface tension is represented.
mod pressure;
mod state;

pub use pressure::{MaterialProjectionWorkspace, ProjectedVelocity, ProjectionReport};
pub use state::{MaterialPair, MaterialProperties, VoxelMaterialError, VoxelMaterialState};

use crate::{Axis, GridGeometry};

pub(crate) fn allocate(n: usize) -> Result<Vec<f64>, VoxelMaterialError> {
    let mut v = Vec::new();
    v.try_reserve_exact(n)
        .map_err(|_| VoxelMaterialError::AllocationFailed)?;
    v.resize(n, 0.0);
    Ok(v)
}

pub(crate) fn faces(grid: &GridGeometry) -> Result<[Vec<f64>; 3], VoxelMaterialError> {
    Ok([
        allocate(grid.face_len(Axis::X))?,
        allocate(grid.face_len(Axis::Y))?,
        allocate(grid.face_len(Axis::Z))?,
    ])
}

pub(crate) fn payload(
    cell_arrays: usize,
    face_arrays: usize,
    grid: &GridGeometry,
) -> Result<usize, VoxelMaterialError> {
    let face_len = Axis::ALL
        .iter()
        .try_fold(0usize, |a, &axis| a.checked_add(grid.face_len(axis)))
        .ok_or(VoxelMaterialError::CapacityOverflow)?;
    grid.cell_len()
        .checked_mul(cell_arrays)
        .and_then(|n| {
            face_len
                .checked_mul(face_arrays)
                .and_then(|m| n.checked_add(m))
        })
        .and_then(|n| n.checked_mul(std::mem::size_of::<f64>()))
        .ok_or(VoxelMaterialError::CapacityOverflow)
}

pub(crate) fn capacity<'a>(
    mut fields: impl Iterator<Item = &'a Vec<f64>>,
) -> Result<usize, VoxelMaterialError> {
    fields.try_fold(0usize, |sum, v| {
        v.capacity()
            .checked_mul(std::mem::size_of::<f64>())
            .and_then(|bytes| sum.checked_add(bytes))
            .ok_or(VoxelMaterialError::CapacityOverflow)
    })
}

pub(crate) fn check_limit(required: usize, limit: usize) -> Result<(), VoxelMaterialError> {
    if required > limit {
        Err(VoxelMaterialError::BufferLimit { required, limit })
    } else {
        Ok(())
    }
}
