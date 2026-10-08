//! One immutable geometry source for an admitted closed axis-aligned box mesh.
//! Geometry/flux only: existing pressure and viscosity owners do not consume it.
use crate::{Axis, GridGeometry, SurfaceStamp, TriangleSurface};
use std::{fmt, mem::size_of};

/// Inactive cell label in the borrowed component array.
pub const NO_FLUID_COMPONENT: usize = usize::MAX;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObstacleStage {
    Cells,
    Faces(Axis),
    Connectivity,
}
#[derive(Debug, Clone, PartialEq)]
pub enum ObstacleError {
    UnsupportedBoxMesh,
    UnresolvedCellTopology { cell: usize },
    ArithmeticFailure,
    CapacityOverflow,
    AllocationFailure,
    BufferLimit { required: usize, limit: usize },
    Cancelled { stage: ObstacleStage, index: usize },
    LengthMismatch,
    InvalidCell,
    NonFiniteSpeed,
}
impl fmt::Display for ObstacleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "static obstacle geometry refused: {self:?}")
    }
}
impl std::error::Error for ObstacleError {}

/// Retained capacity payload plus constructor queue payload. This is not RSS,
/// allocator metadata, fixed stack, caller/callback storage or a whole-program cap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObstacleAllocation {
    pub retained_bytes: usize,
    pub constructor_peak_bytes: usize,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObstacleFace {
    pub area: f64,
    /// Flattened neighboring grid cell, even when that cell is dry.
    pub negative: Option<usize>,
    pub positive: Option<usize>,
}
#[derive(Debug)]
pub struct StaticObstacleGeometry {
    grid: GridGeometry,
    surface: TriangleSurface,
    lower: [f64; 3],
    upper: [f64; 3],
    volumes: Vec<f64>,
    areas: [Vec<f64>; 3],
    labels: Vec<usize>,
    components: usize,
    allocation: ObstacleAllocation,
}
impl StaticObstacleGeometry {
    /// Consumes the world-space surface. No snapping, implicit mesh repair,
    /// minimum-fraction clamp, moving-wall semantics or prior-owner mutation.
    /// `buffer_limit` gates actual managed Vec capacities before publication.
    /// Allocation can fail/refuse; an allocator may round a requested capacity
    /// up before its actual payload can be checked. No hard RSS/peak-cap claim.
    pub fn new(
        grid: GridGeometry,
        surface: TriangleSurface,
        buffer_limit: usize,
        mut cancel: impl FnMut(ObstacleStage, usize) -> bool,
    ) -> Result<Self, ObstacleError> {
        let (lower, upper) = admit_box(&surface)?;
        let n = grid.cell_len();
        let mut used = surface.allocated_bytes();
        gate(used, buffer_limit)?;
        // Plan every live payload before the first allocation, including BFS.
        let count = grid
            .face_len(Axis::X)
            .checked_add(grid.face_len(Axis::Y))
            .and_then(|v| v.checked_add(grid.face_len(Axis::Z)))
            .ok_or(ObstacleError::CapacityOverflow)?;
        let planned = count
            .checked_mul(size_of::<f64>())
            .and_then(|v| {
                n.checked_mul(size_of::<f64>() + 2 * size_of::<usize>())
                    .and_then(|w| v.checked_add(w))
            })
            .and_then(|v| used.checked_add(v))
            .ok_or(ObstacleError::CapacityOverflow)?;
        gate(planned, buffer_limit)?;
        let mut volumes = allocate(n, 0.0, &mut used, buffer_limit)?;
        let mut areas = [Vec::new(), Vec::new(), Vec::new()];
        for axis in Axis::ALL {
            areas[axis.index()] = allocate(grid.face_len(axis), 0.0, &mut used, buffer_limit)?;
        }
        let mut labels = allocate(n, NO_FLUID_COMPONENT, &mut used, buffer_limit)?;
        let mut queue = allocate(n, 0_usize, &mut used, buffer_limit)?;
        let peak = used;
        let dims = grid.counts();
        for k in 0..dims[2] {
            for j in 0..dims[1] {
                for i in 0..dims[0] {
                    let p = [i, j, k];
                    let index = grid.cell_unchecked(p);
                    cancelled(&mut cancel, ObstacleStage::Cells, index)?;
                    let (lo, hi) = cell_bounds(&grid, p);
                    let full = std::array::from_fn::<_, 3, _>(|d| hi[d] - lo[d]);
                    let cut = std::array::from_fn::<_, 3, _>(|d| {
                        overlap(lo[d], hi[d], lower[d], upper[d])
                    });
                    for d in 0..3 {
                        let others = [(d + 1) % 3, (d + 2) % 3];
                        if lower[others[0]] <= lo[others[0]]
                            && hi[others[0]] <= upper[others[0]]
                            && lower[others[1]] <= lo[others[1]]
                            && hi[others[1]] <= upper[others[1]]
                            && lo[d] < lower[d]
                            && upper[d] < hi[d]
                        {
                            return Err(ObstacleError::UnresolvedCellTopology { cell: index });
                        }
                    }
                    let total = product(&full)?;
                    let blocked = product(&cut)?;
                    let dry = (0..3).all(|d| lower[d] <= lo[d] && hi[d] <= upper[d]);
                    volumes[index] = measure(total, blocked, dry)?;
                }
            }
        }
        for axis in Axis::ALL {
            let d = axis.index();
            let tangents = [(d + 1) % 3, (d + 2) % 3];
            let fd = grid.face_counts(axis);
            for k in 0..fd[2] {
                for j in 0..fd[1] {
                    for i in 0..fd[0] {
                        let p = [i, j, k];
                        let index = grid.face_unchecked(axis, p);
                        cancelled(&mut cancel, ObstacleStage::Faces(axis), index)?;
                        let (lo, hi) = cell_bounds(&grid, p);
                        let a = tangents[0];
                        let b = tangents[1];
                        let total = product(&[hi[a] - lo[a], hi[b] - lo[b]])?;
                        // Solid is closed: a face on the box boundary is blocked too.
                        let on_solid = lower[d] <= lo[d] && lo[d] <= upper[d];
                        let blocked = if on_solid {
                            product(&[
                                overlap(lo[a], hi[a], lower[a], upper[a]),
                                overlap(lo[b], hi[b], lower[b], upper[b]),
                            ])?
                        } else {
                            0.0
                        };
                        let dry = on_solid
                            && lower[a] <= lo[a]
                            && hi[a] <= upper[a]
                            && lower[b] <= lo[b]
                            && hi[b] <= upper[b];
                        areas[d][index] = measure(total, blocked, dry)?;
                    }
                }
            }
        }
        let mut components = 0;
        for seed in 0..n {
            if volumes[seed] == 0.0 || labels[seed] != NO_FLUID_COMPONENT {
                continue;
            }
            labels[seed] = components;
            queue[0] = seed;
            let mut head = 0;
            let mut tail = 1;
            while head < tail {
                let cell = queue[head];
                head += 1;
                cancelled(&mut cancel, ObstacleStage::Connectivity, cell)?;
                let p = coordinate(dims, cell);
                for axis in Axis::ALL {
                    let d = axis.index();
                    for positive in [false, true] {
                        if (!positive && p[d] == 0) || (positive && p[d] + 1 == dims[d]) {
                            continue;
                        }
                        let mut face = p;
                        if positive {
                            face[d] += 1;
                        }
                        let mut neighbor = p;
                        if positive {
                            neighbor[d] += 1;
                        } else {
                            neighbor[d] -= 1;
                        }
                        let other = grid.cell_unchecked(neighbor);
                        if areas[d][grid.face_unchecked(axis, face)] == 0.0 {
                            continue;
                        }
                        if volumes[other] == 0.0 {
                            return Err(ObstacleError::ArithmeticFailure);
                        }
                        if labels[other] == NO_FLUID_COMPONENT {
                            labels[other] = components;
                            queue[tail] = other;
                            tail += 1;
                        }
                    }
                }
            }
            components += 1;
        }
        let queue_bytes = queue
            .capacity()
            .checked_mul(size_of::<usize>())
            .ok_or(ObstacleError::CapacityOverflow)?;
        let allocation = ObstacleAllocation {
            retained_bytes: used - queue_bytes,
            constructor_peak_bytes: peak,
        };
        Ok(Self {
            grid,
            surface,
            lower,
            upper,
            volumes,
            areas,
            labels,
            components,
            allocation,
        })
    }
    pub fn grid(&self) -> &GridGeometry {
        &self.grid
    }
    /// Collision uses the exact admitted source, not another derived mesh.
    pub fn surface(&self) -> &TriangleSurface {
        &self.surface
    }
    pub fn stamp(&self) -> SurfaceStamp {
        self.surface.stamp()
    }
    pub fn box_bounds(&self) -> ([f64; 3], [f64; 3]) {
        (self.lower, self.upper)
    }
    pub fn fluid_volumes(&self) -> &[f64] {
        &self.volumes
    }
    pub fn open_areas(&self, axis: Axis) -> &[f64] {
        &self.areas[axis.index()]
    }
    pub fn component_labels(&self) -> &[usize] {
        &self.labels
    }
    pub fn component_count(&self) -> usize {
        self.components
    }
    pub fn allocation(&self) -> ObstacleAllocation {
        self.allocation
    }
    pub fn face(&self, axis: Axis, p: [usize; 3]) -> Option<ObstacleFace> {
        let index = self.grid.face_index(axis, p)?;
        let d = axis.index();
        let negative = if p[d] > 0 {
            let mut c = p;
            c[d] -= 1;
            self.grid.cell_index(c)
        } else {
            None
        };
        let positive = self.grid.cell_index(p);
        Some(ObstacleFace {
            area: self.areas[d][index],
            negative,
            positive,
        })
    }
    /// Signed outward volume flux (m³/s). Outer faces remain geometric openings;
    /// callers explicitly supply prescribed wall/inlet speed. No pressure solve,
    /// boundary response, field mutation or allocation is performed here.
    pub fn outward_flux(&self, cell: [usize; 3], speed: [&[f64]; 3]) -> Result<f64, ObstacleError> {
        self.grid
            .cell_index(cell)
            .ok_or(ObstacleError::InvalidCell)?;
        for axis in Axis::ALL {
            if speed[axis.index()].len() != self.grid.face_len(axis) {
                return Err(ObstacleError::LengthMismatch);
            }
        }
        let mut sum = 0.0;
        for axis in Axis::ALL {
            let d = axis.index();
            let lo = self.grid.face_unchecked(axis, cell);
            let mut upper = cell;
            upper[d] += 1;
            let hi = self.grid.face_unchecked(axis, upper);
            for index in [lo, hi] {
                if !speed[d][index].is_finite() {
                    return Err(ObstacleError::NonFiniteSpeed);
                }
            }
            let flux = |index: usize| {
                let area = self.areas[d][index];
                let velocity = speed[d][index];
                let value = area * velocity;
                if !value.is_finite()
                    || (area != 0.0 && velocity != 0.0 && (!value.is_normal() || value == 0.0))
                {
                    Err(ObstacleError::ArithmeticFailure)
                } else {
                    Ok(value)
                }
            };
            sum += flux(hi)? - flux(lo)?;
        }
        if !sum.is_finite() {
            return Err(ObstacleError::ArithmeticFailure);
        }
        Ok(sum)
    }
}
fn gate(required: usize, limit: usize) -> Result<(), ObstacleError> {
    if required > limit {
        Err(ObstacleError::BufferLimit { required, limit })
    } else {
        Ok(())
    }
}
fn allocate<T: Clone>(
    n: usize,
    value: T,
    used: &mut usize,
    limit: usize,
) -> Result<Vec<T>, ObstacleError> {
    let min = n
        .checked_mul(size_of::<T>())
        .and_then(|v| used.checked_add(v))
        .ok_or(ObstacleError::CapacityOverflow)?;
    gate(min, limit)?;
    let mut buffer = Vec::new();
    buffer
        .try_reserve_exact(n)
        .map_err(|_| ObstacleError::AllocationFailure)?;
    *used = buffer
        .capacity()
        .checked_mul(size_of::<T>())
        .and_then(|v| used.checked_add(v))
        .ok_or(ObstacleError::CapacityOverflow)?;
    gate(*used, limit)?;
    buffer.resize(n, value);
    Ok(buffer)
}
fn cancelled(
    cancel: &mut impl FnMut(ObstacleStage, usize) -> bool,
    stage: ObstacleStage,
    index: usize,
) -> Result<(), ObstacleError> {
    if cancel(stage, index) {
        Err(ObstacleError::Cancelled { stage, index })
    } else {
        Ok(())
    }
}
fn coordinate(dims: [usize; 3], index: usize) -> [usize; 3] {
    [
        index % dims[0],
        index / dims[0] % dims[1],
        index / (dims[0] * dims[1]),
    ]
}
fn cell_bounds(grid: &GridGeometry, p: [usize; 3]) -> ([f64; 3], [f64; 3]) {
    let o = grid.origin();
    let h = grid.spacing();
    (
        std::array::from_fn(|d| o[d] + p[d] as f64 * h[d]),
        std::array::from_fn(|d| o[d] + (p[d] + 1) as f64 * h[d]),
    )
}
fn overlap(a: f64, b: f64, c: f64, d: f64) -> f64 {
    let lo = a.max(c);
    let hi = b.min(d);
    if hi > lo { hi - lo } else { 0.0 }
}
fn product(values: &[f64]) -> Result<f64, ObstacleError> {
    if values.iter().any(|v| !v.is_finite() || *v < 0.0) {
        return Err(ObstacleError::ArithmeticFailure);
    }
    if values.contains(&0.0) {
        return Ok(0.0);
    }
    let mut result = 1.0;
    for value in values {
        result *= value;
        if !result.is_normal() || result <= 0.0 {
            return Err(ObstacleError::ArithmeticFailure);
        }
    }
    Ok(result)
}
fn measure(total: f64, blocked: f64, dry: bool) -> Result<f64, ObstacleError> {
    let result = total - blocked;
    if !total.is_normal()
        || total <= 0.0
        || !blocked.is_finite()
        || blocked < 0.0
        || blocked > total
        || (blocked > 0.0 && (!blocked.is_normal() || result == total))
        || (dry && result != 0.0)
        || (!dry && (!result.is_normal() || result <= 0.0))
    {
        return Err(ObstacleError::ArithmeticFailure);
    }
    Ok(result)
}
fn admit_box(surface: &TriangleSurface) -> Result<([f64; 3], [f64; 3]), ObstacleError> {
    let vertices = surface.vertices();
    let triangles = surface.triangles();
    if vertices.len() != 8 || triangles.len() != 12 {
        return Err(ObstacleError::UnsupportedBoxMesh);
    }
    let lower =
        std::array::from_fn(|d| vertices.iter().map(|p| p[d]).fold(f64::INFINITY, f64::min));
    let upper = std::array::from_fn(|d| {
        vertices
            .iter()
            .map(|p| p[d])
            .fold(f64::NEG_INFINITY, f64::max)
    });
    if (0..3).any(|d| !((upper[d] - lower[d]).is_normal()) || upper[d] <= lower[d]) {
        return Err(ObstacleError::UnsupportedBoxMesh);
    }
    let mut corners = [0_u8; 8];
    let mut seen = 0_u16;
    for (i, p) in vertices.iter().enumerate() {
        let mut corner = 0_u8;
        for d in 0..3 {
            if p[d] == upper[d] {
                corner |= 1 << d;
            } else if p[d] != lower[d] {
                return Err(ObstacleError::UnsupportedBoxMesh);
            }
        }
        if seen & (1 << corner) != 0 {
            return Err(ObstacleError::UnsupportedBoxMesh);
        }
        seen |= 1 << corner;
        corners[i] = corner;
    }
    let mut faces = [[0_u16; 2]; 6];
    let mut counts = [0_usize; 6];
    for triangle in triangles {
        let c = triangle.map(|i| corners[i]);
        let d = (0..3)
            .find(|d| (c[0] >> d) & 1 == (c[1] >> d) & 1 && (c[0] >> d) & 1 == (c[2] >> d) & 1)
            .ok_or(ObstacleError::UnsupportedBoxMesh)?;
        let side = (c[0] >> d) & 1;
        let a = (d + 1) % 3;
        let b = (d + 2) % 3;
        let bit = |i: usize, axis: usize| i32::from((c[i] >> axis) & 1);
        let winding = (bit(1, a) - bit(0, a)) * (bit(2, b) - bit(0, b))
            - (bit(1, b) - bit(0, b)) * (bit(2, a) - bit(0, a));
        if winding != if side == 0 { -1 } else { 1 } {
            return Err(ObstacleError::UnsupportedBoxMesh);
        }
        let face = 2 * d + usize::from(side);
        if counts[face] == 2 {
            return Err(ObstacleError::UnsupportedBoxMesh);
        }
        faces[face][counts[face]] = c.iter().fold(0_u16, |mask, &corner| mask | (1 << corner));
        counts[face] += 1;
    }
    for face in 0..6 {
        if counts[face] != 2 {
            return Err(ObstacleError::UnsupportedBoxMesh);
        }
        let [a, b] = faces[face];
        let common = a & b;
        let expected = (0_u8..8)
            .filter(|c| (c >> (face / 2)) & 1 == (face % 2) as u8)
            .fold(0_u16, |mask, c| mask | (1 << c));
        if a | b != expected || common.count_ones() != 2 {
            return Err(ObstacleError::UnsupportedBoxMesh);
        }
        let first = common.trailing_zeros();
        let second = (common & !(1 << first)).trailing_zeros();
        if (first ^ second).count_ones() != 2 {
            return Err(ObstacleError::UnsupportedBoxMesh);
        }
    }
    Ok((lower, upper))
}
