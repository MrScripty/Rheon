use std::fmt;

/// Cartesian velocity component and its corresponding face orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    pub(crate) const ALL: [Self; 3] = [Self::X, Self::Y, Self::Z];

    pub(crate) const fn index(self) -> usize {
        match self {
            Self::X => 0,
            Self::Y => 1,
            Self::Z => 2,
        }
    }
}

/// Rejected geometry or buffer-payload request.
#[derive(Debug, Clone, PartialEq)]
pub enum GeometryError {
    ZeroDimension { axis: usize },
    IntegerRange,
    ArithmeticOverflow,
    InvalidSpacing { axis: usize },
    InvalidOrigin { axis: usize },
    UnrepresentableCoordinates { axis: usize },
    InvalidCellVolume,
    BufferLimit { required: usize, limit: usize },
}

impl fmt::Display for GeometryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroDimension { axis } => write!(f, "cell count is zero on axis {axis}"),
            Self::IntegerRange => write!(f, "cell count does not fit the target index domain"),
            Self::ArithmeticOverflow => write!(f, "grid capacity arithmetic overflowed"),
            Self::InvalidSpacing { axis } => {
                write!(f, "spacing must be finite and positive on axis {axis}")
            }
            Self::InvalidOrigin { axis } => write!(f, "origin must be finite on axis {axis}"),
            Self::UnrepresentableCoordinates { axis } => {
                write!(f, "world coordinates lose cell resolution on axis {axis}")
            }
            Self::InvalidCellVolume => write!(f, "cell volume must be finite and positive"),
            Self::BufferLimit { required, limit } => write!(
                f,
                "managed buffer payload requires {required} bytes, limit is {limit}"
            ),
        }
    }
}

impl std::error::Error for GeometryError {}

fn product(values: [usize; 3]) -> Result<usize, GeometryError> {
    values[0]
        .checked_mul(values[1])
        .and_then(|v| v.checked_mul(values[2]))
        .ok_or(GeometryError::ArithmeticOverflow)
}

/// Immutable validated MAC geometry. Single-cell and degenerate (one-cell-wide)
/// dimensions are supported. Zero counts, nonfinite coordinates and capacity
/// overflow are rejected before allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct GridGeometry {
    counts: [usize; 3],
    face_counts: [[usize; 3]; 3],
    cell_len: usize,
    face_lens: [usize; 3],
    origin: [f64; 3],
    spacing: [f64; 3],
    upper: [f64; 3],
    cell_volume: f64,
}

impl GridGeometry {
    /// Validate source-domain counts before target conversion, arithmetic or
    /// allocation. The source integer type cannot represent negative counts;
    /// CLI adapters must reject a leading minus sign rather than cast it.
    pub fn new(
        counts: [u64; 3],
        spacing: [f64; 3],
        origin: [f64; 3],
    ) -> Result<Self, GeometryError> {
        let mut dims = [0; 3];
        for axis in 0..3 {
            if counts[axis] == 0 {
                return Err(GeometryError::ZeroDimension { axis });
            }
            dims[axis] = usize::try_from(counts[axis]).map_err(|_| GeometryError::IntegerRange)?;
            if !spacing[axis].is_finite() || spacing[axis] <= 0.0 {
                return Err(GeometryError::InvalidSpacing { axis });
            }
            if !origin[axis].is_finite() {
                return Err(GeometryError::InvalidOrigin { axis });
            }
        }
        let cell_len = product(dims)?;
        let mut face_counts = [dims; 3];
        let mut face_lens = [0; 3];
        for axis in 0..3 {
            face_counts[axis][axis] = dims[axis]
                .checked_add(1)
                .ok_or(GeometryError::ArithmeticOverflow)?;
            face_lens[axis] = product(face_counts[axis])?;
        }
        let cell_volume = spacing[0] * spacing[1] * spacing[2];
        if !cell_volume.is_finite() || cell_volume <= 0.0 {
            return Err(GeometryError::InvalidCellVolume);
        }
        let mut upper = [0.0; 3];
        for axis in 0..3 {
            // Integer endpoints alone are insufficient: k + 1/2 loses
            // resolution earlier. Require exactly representable half-offset
            // indices and a deliberately conservative physical-scale margin.
            if counts[axis] > (1_u64 << 52) {
                return Err(GeometryError::UnrepresentableCoordinates { axis });
            }
            let n = counts[axis] as f64;
            let extent = n * spacing[axis];
            upper[axis] = origin[axis] + extent;
            let scale = origin[axis].abs().max(upper[axis].abs()).max(extent);
            // Coordinate evaluation includes a multiply and an add. At this
            // margin their rounding cannot consume a half-cell separation;
            // bounding the whole axis also covers interior adjacent samples.
            // Excluding subnormal half-spacing avoids relative-error arguments
            // outside the normal binary64 range. Some usable grids are
            // intentionally rejected rather than silently collapsing samples.
            if !extent.is_finite()
                || !scale.is_finite()
                || !(0.5 * spacing[axis]).is_normal()
                || spacing[axis] / scale < 32.0 * f64::EPSILON
            {
                return Err(GeometryError::UnrepresentableCoordinates { axis });
            }
            let first_center = origin[axis] + 0.5 * spacing[axis];
            let last_center = origin[axis] + (n - 0.5) * spacing[axis];
            if !upper[axis].is_finite()
                || upper[axis] <= origin[axis]
                || first_center <= origin[axis]
                || last_center >= upper[axis]
                || (dims[axis] > 1 && origin[axis] + spacing[axis] <= origin[axis])
            {
                return Err(GeometryError::UnrepresentableCoordinates { axis });
            }
        }
        Ok(Self {
            counts: dims,
            face_counts,
            cell_len,
            face_lens,
            origin,
            spacing,
            upper,
            cell_volume,
        })
    }

    pub fn counts(&self) -> [usize; 3] {
        self.counts
    }

    pub fn cell_len(&self) -> usize {
        self.cell_len
    }

    pub fn face_counts(&self, axis: Axis) -> [usize; 3] {
        self.face_counts[axis.index()]
    }

    pub fn face_len(&self, axis: Axis) -> usize {
        self.face_lens[axis.index()]
    }

    pub fn spacing(&self) -> [f64; 3] {
        self.spacing
    }

    pub fn origin(&self) -> [f64; 3] {
        self.origin
    }

    pub fn upper(&self) -> [f64; 3] {
        self.upper
    }

    pub fn cell_volume(&self) -> f64 {
        self.cell_volume
    }

    /// Checked public cell indexing. The validated shape proves arithmetic
    /// representability; coordinate checks establish the remaining precondition.
    pub fn cell_index(&self, coordinate: [usize; 3]) -> Option<usize> {
        index(self.counts, coordinate)
    }

    pub fn face_index(&self, axis: Axis, coordinate: [usize; 3]) -> Option<usize> {
        index(self.face_counts[axis.index()], coordinate)
    }

    pub fn cell_position(&self, coordinate: [usize; 3]) -> Option<[f64; 3]> {
        self.cell_index(coordinate)?;
        Some(std::array::from_fn(|d| {
            self.origin[d] + (coordinate[d] as f64 + 0.5) * self.spacing[d]
        }))
    }

    pub fn face_position(&self, axis: Axis, coordinate: [usize; 3]) -> Option<[f64; 3]> {
        self.face_index(axis, coordinate)?;
        Some(std::array::from_fn(|d| {
            let offset = if d == axis.index() { 0.0 } else { 0.5 };
            self.origin[d] + (coordinate[d] as f64 + offset) * self.spacing[d]
        }))
    }

    pub(crate) fn cell_unchecked(&self, coordinate: [usize; 3]) -> usize {
        // Only called by loops whose bounds are this immutable validated shape.
        coordinate[0] + self.counts[0] * (coordinate[1] + self.counts[1] * coordinate[2])
    }

    pub(crate) fn face_unchecked(&self, axis: Axis, coordinate: [usize; 3]) -> usize {
        let dims = self.face_counts[axis.index()];
        coordinate[0] + dims[0] * (coordinate[1] + dims[1] * coordinate[2])
    }
}

fn index(dims: [usize; 3], p: [usize; 3]) -> Option<usize> {
    if (0..3).any(|d| p[d] >= dims[d]) {
        None
    } else {
        Some(p[0] + dims[0] * (p[1] + dims[1] * p[2]))
    }
}

/// Named planned managed-buffer payloads. This is not process RSS and excludes
/// allocator metadata, stack values, image export and GUI resources. Actual
/// Vec capacities must be checked against this plan when allocation is added.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferPlan {
    pub velocity_pair_bytes: usize,
    pub tracer_pair_bytes: usize,
    pub pressure_workspace_bytes: usize,
    pub total_bytes: usize,
}

impl BufferPlan {
    /// Two f32 MAC velocity buffers, two f32 cell tracer buffers, and six f64
    /// pressure vectors (p, rhs, r, direction, A-direction, preconditioned r).
    pub fn for_grid(grid: &GridGeometry, limit: usize) -> Result<Self, GeometryError> {
        let faces = grid.face_lens.iter().try_fold(0_usize, |sum, &n| {
            sum.checked_add(n).ok_or(GeometryError::ArithmeticOverflow)
        })?;
        let velocity_pair_bytes = faces
            .checked_mul(8)
            .ok_or(GeometryError::ArithmeticOverflow)?;
        let tracer_pair_bytes = grid
            .cell_len
            .checked_mul(8)
            .ok_or(GeometryError::ArithmeticOverflow)?;
        let pressure_workspace_bytes = grid
            .cell_len
            .checked_mul(48)
            .ok_or(GeometryError::ArithmeticOverflow)?;
        let total_bytes = velocity_pair_bytes
            .checked_add(tracer_pair_bytes)
            .and_then(|n| n.checked_add(pressure_workspace_bytes))
            .ok_or(GeometryError::ArithmeticOverflow)?;
        if total_bytes > limit {
            return Err(GeometryError::BufferLimit {
                required: total_bytes,
                limit,
            });
        }
        Ok(Self {
            velocity_pair_bytes,
            tracer_pair_bytes,
            pressure_workspace_bytes,
            total_bytes,
        })
    }
}
