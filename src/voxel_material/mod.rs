//! Components for the filled, incompressible two-material MAC model.
//!
//! These numerical components are not a complete native simulation mode. The
//! coupled transport/stress step and authoring/recording gates are tracked in
//! `docs/plans/voxel-multimaterial-initial-authoring/plan.md`. No vacuum, EOS,
//! thermal evolution, phase change or surface tension is represented.
mod pressure;
mod state;
mod step;
mod transport;
mod viscosity;

pub use pressure::{MaterialProjectionWorkspace, ProjectedVelocity, ProjectionReport};
pub use state::{MaterialPair, MaterialProperties, VoxelMaterialError, VoxelMaterialState};
pub use step::{
    VoxelMaterialConfig, VoxelMaterialSimulation, VoxelMaterialStage, VoxelMaterialStepReport,
};
pub use transport::TransportReport;
pub use viscosity::ViscosityReport;

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

// These kernels reject subnormal intermediates instead of applying a relative
// error bound where gradual underflow invalidates that bound.
pub(crate) fn checked(x: f64) -> Result<f64, VoxelMaterialError> {
    if x == 0.0 || x.is_normal() {
        Ok(x)
    } else {
        Err(VoxelMaterialError::ArithmeticFailure)
    }
}
pub(crate) fn multiply(a: f64, b: f64) -> Result<f64, VoxelMaterialError> {
    let value = checked(a * b)?;
    if value == 0.0 && a != 0.0 && b != 0.0 {
        Err(VoxelMaterialError::ArithmeticFailure)
    } else {
        Ok(value)
    }
}
pub(crate) fn divide(a: f64, b: f64) -> Result<f64, VoxelMaterialError> {
    if !b.is_normal() || b == 0.0 {
        return Err(VoxelMaterialError::ArithmeticFailure);
    }
    let value = checked(a / b)?;
    if value == 0.0 && a != 0.0 {
        Err(VoxelMaterialError::ArithmeticFailure)
    } else {
        Ok(value)
    }
}
pub(crate) fn upper(x: f64) -> Result<f64, VoxelMaterialError> {
    checked(if x == 0.0 { 0.0 } else { x.next_up() })
}
pub(crate) fn gamma(operations: usize) -> Result<f64, VoxelMaterialError> {
    // All admitted grids/counts keep the integer exactly representable.
    if operations as u64 > (1u64 << 52) {
        return Err(VoxelMaterialError::CapacityOverflow);
    }
    let k = upper(operations as f64 * f64::EPSILON)?;
    if k >= 1.0 {
        return Err(VoxelMaterialError::ArithmeticFailure);
    }
    upper(k / (1.0 - k).next_down())
}
pub(crate) fn roundoff(operations: usize, scale: f64) -> Result<f64, VoxelMaterialError> {
    upper(multiply(gamma(operations)?, scale)?)
}
pub(crate) fn sum(values: impl Iterator<Item = f64>) -> Result<f64, VoxelMaterialError> {
    let mut total = 0.0_f64;
    let mut correction = 0.0_f64;
    for value in values {
        checked(value)?;
        let next = checked(total + value)?;
        let error = if total.abs() >= value.abs() {
            checked((total - next) + value)?
        } else {
            checked((value - next) + total)?
        };
        correction = checked(correction + error)?;
        total = next;
    }
    checked(total + correction)
}
pub(crate) fn coordinate(grid: &GridGeometry, cell: usize) -> [usize; 3] {
    let [nx, ny, _] = grid.counts();
    [cell % nx, (cell / nx) % ny, cell / (nx * ny)]
}
pub(crate) fn face_mass(
    grid: &GridGeometry,
    materials: MaterialPair,
    f: &[f64],
    axis: Axis,
    p: [usize; 3],
) -> Result<f64, VoxelMaterialError> {
    let d = axis.index();
    if p[d] == 0 || p[d] == grid.counts()[d] {
        return Ok(0.0);
    }
    let mut lo = p;
    lo[d] -= 1;
    let rho = checked(
        0.5 * materials.density(f[grid.cell_unchecked(lo)])
            + 0.5 * materials.density(f[grid.cell_unchecked(p)]),
    )?;
    let m = multiply(grid.cell_volume(), rho)?;
    if m <= 0.0 {
        return Err(VoxelMaterialError::ArithmeticFailure);
    }
    Ok(m)
}
