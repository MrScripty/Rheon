use crate::geometry::{Axis, GridGeometry};
use std::fmt;

/// A rejected numerical operation. Output scratch may be partially overwritten
/// after ArithmeticFailure; an accepted simulation state must never alias it.
#[derive(Debug, Clone, PartialEq)]
pub enum OperatorError {
    InvalidDensity,
    InvalidTimeStep,
    InvalidCoefficient,
    InvalidPressureDomain,
    NonZeroAirRhs { cell: usize },
    LengthMismatch,
    NonFiniteInput,
    NonZeroWallVelocity { axis: Axis, index: usize },
    ArithmeticFailure,
}

impl fmt::Display for OperatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDensity => write!(f, "density must be finite and positive"),
            Self::InvalidTimeStep => write!(f, "time step must be finite and positive"),
            Self::InvalidCoefficient => {
                write!(f, "pressure coefficient is not finite and positive")
            }
            Self::InvalidPressureDomain => write!(f, "invalid cell-aligned pressure domain"),
            Self::NonZeroAirRhs { cell } => write!(f, "inactive air RHS is nonzero at {cell}"),
            Self::LengthMismatch => {
                write!(f, "field length does not match validated grid geometry")
            }
            Self::NonFiniteInput => write!(f, "field contains a nonfinite value"),
            Self::NonZeroWallVelocity { axis, index } => {
                write!(f, "fixed wall velocity is nonzero on {axis:?} face {index}")
            }
            Self::ArithmeticFailure => write!(f, "numerical operation produced a nonfinite value"),
        }
    }
}

impl std::error::Error for OperatorError {}

/// Symmetric integrated pressure operator for a single connected rectangular
/// domain with fixed impermeable walls. Pressure is cell centered, speed is
/// face centered, and units are metres, seconds, kilograms.
pub struct PressureOperator<'a> {
    grid: &'a GridGeometry,
    face_area: [f64; 3],
    weight: [f64; 3],
    acceleration_factor: [f64; 3],
    surface: Option<crate::column_surface::PressureSurface<'a>>,
    flat_columns: Option<crate::FlatColumnMacGeometry<'a>>,
}

impl<'a> PressureOperator<'a> {
    pub fn new(grid: &'a GridGeometry, density: f64) -> Result<Self, OperatorError> {
        if !density.is_finite() || density <= 0.0 {
            return Err(OperatorError::InvalidDensity);
        }
        let h = grid.spacing();
        let face_area = [h[1] * h[2], h[0] * h[2], h[0] * h[1]];
        let acceleration_factor = std::array::from_fn(|d| 1.0 / (density * h[d]));
        let weight = std::array::from_fn(|d| face_area[d] * acceleration_factor[d]);
        for d in 0..3 {
            if !face_area[d].is_finite()
                || face_area[d] <= 0.0
                || !acceleration_factor[d].is_finite()
                || acceleration_factor[d] <= 0.0
                || !weight[d].is_finite()
                || weight[d] <= 0.0
            {
                return Err(OperatorError::InvalidCoefficient);
            }
        }
        Ok(Self {
            grid,
            face_area,
            weight,
            acceleration_factor,
            surface: None,
            flat_columns: None,
        })
    }

    /// Atmospheric zero at a resolved wet/air face, half a cell from the
    /// wet pressure center. Outer box faces remain impermeable. No gauge is
    /// imposed: every wet component of this slab touches the pressure surface.
    pub fn with_free_surface(
        grid: &'a GridGeometry,
        density: f64,
        surface: &'a crate::SlabFreeSurface,
    ) -> Result<Self, OperatorError> {
        if surface.grid() != grid {
            return Err(OperatorError::InvalidPressureDomain);
        }
        let mut operator = Self::new(grid, density)?;
        if operator
            .weight
            .iter()
            .chain(&operator.acceleration_factor)
            .any(|v| !(2.0 * v).is_finite())
        {
            return Err(OperatorError::InvalidCoefficient);
        }
        operator.surface = Some(crate::column_surface::PressureSurface::Slab(surface));
        Ok(operator)
    }
    /// Ghost-fluid pressure distances from the reconstructed column height
    /// field. No minimum-distance clamp; unsupported scales are rejected.
    pub fn with_columns(
        grid: &'a GridGeometry,
        density: f64,
        surface: crate::ColumnSurfaceView<'a>,
    ) -> Result<Self, OperatorError> {
        if surface.grid() != grid {
            return Err(OperatorError::InvalidPressureDomain);
        }
        let mut operator = Self::new(grid, density)?;
        operator.surface = Some(crate::column_surface::PressureSurface::Columns(surface));
        for axis in Axis::ALL {
            let d = axis.index();
            let dims = grid.face_counts(axis);
            for k in 0..dims[2] {
                for j in 0..dims[1] {
                    for i in 0..dims[0] {
                        let p = [i, j, k];
                        if p[d] == 0 || p[d] >= grid.counts()[d] {
                            continue;
                        }
                        let scale = surface.face_scale(axis, p);
                        for base in [operator.weight[d], operator.acceleration_factor[d]] {
                            let value = base * scale;
                            if !value.is_finite()
                                || value.is_subnormal()
                                || (value == 0.0 && scale != 0.0)
                            {
                                return Err(OperatorError::InvalidCoefficient);
                            }
                        }
                    }
                }
            }
        }
        Ok(operator)
    }
    /// Matched liquid dual-face mass and integrated flux geometry for a fixed
    /// flat column domain. Distinct from the frozen full-inertia column mode.
    pub fn with_flat_column_masses(
        grid: &'a GridGeometry,
        density: f64,
        surface: crate::ColumnSurfaceView<'a>,
    ) -> Result<Self, OperatorError> {
        if surface.grid() != grid {
            return Err(OperatorError::InvalidPressureDomain);
        }
        let geometry = crate::FlatColumnMacGeometry::new(surface, density)
            .map_err(|_| OperatorError::InvalidPressureDomain)?;
        let mut operator = Self::new(grid, density)?;
        operator.surface = Some(crate::column_surface::PressureSurface::Columns(surface));
        operator.flat_columns = Some(geometry);
        Ok(operator)
    }
    pub(crate) fn row_volume(&self, cell: usize) -> f64 {
        self.flat_columns
            .map_or(self.grid.cell_volume(), |g| g.cell_volume(cell).unwrap())
    }
    pub(crate) fn scaled_residual_divergence(&self, residual: &[f64], dt: f64) -> f64 {
        if self.flat_columns.is_none() {
            residual.iter().fold(0.0_f64, |a, b| a.max(b.abs())) * dt / self.grid.cell_volume()
        } else {
            residual
                .iter()
                .enumerate()
                .filter(|(cell, _)| self.is_wet(*cell))
                .fold(0.0_f64, |a, (cell, b)| {
                    a.max(b.abs() * dt / self.row_volume(cell))
                })
        }
    }
    pub(crate) fn neighbor_weight(&self, axis: Axis, p: [usize; 3]) -> f64 {
        self.flat_columns
            .map_or(self.weight[axis.index()], |g| g.weight(axis, p))
    }
    fn face_weight(&self, axis: Axis, p: [usize; 3]) -> f64 {
        if let Some(geometry) = self.flat_columns {
            return geometry.weight(axis, p);
        }

        let mut low = p;
        low[axis.index()] -= 1;
        let a = self.is_wet(self.grid.cell_unchecked(low));
        let b = self.is_wet(self.grid.cell_unchecked(p));
        if a && b {
            self.weight[axis.index()]
        } else if a || b {
            self.weight[axis.index()] * self.surface.map_or(1.0, |s| s.scale(axis, p))
        } else {
            0.0
        }
    }
    pub(crate) fn has_gauge(&self) -> bool {
        self.surface.is_none()
    }
    pub(crate) fn is_wet(&self, cell: usize) -> bool {
        self.surface.is_none_or(|s| s.is_wet(cell))
    }
    pub(crate) fn clear_inactive(&self, candidate: [&mut [f32]; 3]) {
        if let Some(surface) = self.surface {
            for (d, field) in candidate.into_iter().enumerate() {
                let axis = Axis::ALL[d];
                let [nx, ny, nz] = self.grid.face_counts(axis);
                for k in 0..nz {
                    for j in 0..ny {
                        for i in 0..nx {
                            let p = [i, j, k];
                            if !surface.face_active(axis, p) {
                                field[self.grid.face_unchecked(axis, p)] = 0.0;
                            }
                        }
                    }
                }
            }
        }
    }
    pub(crate) fn extend_column_band(&self, candidate: [&mut [f32]; 3]) {
        let Some(crate::column_surface::PressureSurface::Columns(surface)) = self.surface else {
            return;
        };
        let up = surface.axis().index();
        for (d, field) in candidate.into_iter().enumerate() {
            let axis = Axis::ALL[d];
            let dims = self.grid.face_counts(axis);
            // Read only an active lower face: a copied air value cannot become
            // the donor for another layer. Outer impermeable faces stay zero.
            for k in 0..dims[2] {
                for j in 0..dims[1] {
                    for i in 0..dims[0] {
                        let p = [i, j, k];
                        if p[d] == 0
                            || p[d] == self.grid.counts()[d]
                            || p[up] == 0
                            || surface.face_active(axis, p)
                        {
                            continue;
                        }
                        let mut below = p;
                        below[up] -= 1;
                        if surface.face_active(axis, below) {
                            field[self.grid.face_unchecked(axis, p)] =
                                field[self.grid.face_unchecked(axis, below)];
                        }
                    }
                }
            }
        }
    }
    fn correction(&self, axis: Axis, p: [usize; 3], pressure: &[f64]) -> Option<f64> {
        let d = axis.index();
        let mut low = p;
        low[d] -= 1;
        let left = self.grid.cell_unchecked(low);
        let right = self.grid.cell_unchecked(p);
        if let Some(geometry) = self.flat_columns {
            let acceleration = geometry.acceleration(axis, p);
            if acceleration == 0.0 {
                return None;
            }
            let a = if self.is_wet(left) {
                pressure[left]
            } else {
                0.0
            };
            let b = if self.is_wet(right) {
                pressure[right]
            } else {
                0.0
            };
            return Some(acceleration * (b - a));
        }
        match (self.is_wet(left), self.is_wet(right)) {
            (true, true) => Some(self.acceleration_factor[d] * (pressure[right] - pressure[left])),
            (true, false) => Some(
                self.surface.map_or(1.0, |s| s.scale(axis, p))
                    * self.acceleration_factor[d]
                    * (-pressure[left]),
            ),
            (false, true) => Some(
                self.surface.map_or(1.0, |s| s.scale(axis, p))
                    * self.acceleration_factor[d]
                    * pressure[right],
            ),
            (false, false) => None,
        }
    }

    pub fn geometry(&self) -> &GridGeometry {
        self.grid
    }

    /// Apply K = B diag(area/(density*distance)) B^T without imposing a gauge.
    pub fn apply_full(&self, pressure: &[f64], out: &mut [f64]) -> Result<(), OperatorError> {
        self.validate_cells(pressure)?;
        if out.len() != self.grid.cell_len() {
            return Err(OperatorError::LengthMismatch);
        }
        let [nx, ny, nz] = self.grid.counts();
        for k in 0..nz {
            for j in 0..ny {
                for i in 0..nx {
                    let coordinate = [i, j, k];
                    let row = self.grid.cell_unchecked(coordinate);
                    if !self.is_wet(row) {
                        out[row] = pressure[row];
                        continue;
                    }
                    let mut value = 0.0;
                    for d in 0..3 {
                        if coordinate[d] > 0 {
                            let mut neighbor = coordinate;
                            neighbor[d] -= 1;
                            let other = self.grid.cell_unchecked(neighbor);
                            value += if self.is_wet(other) {
                                self.neighbor_weight(Axis::ALL[d], coordinate)
                                    * (pressure[row] - pressure[other])
                            } else {
                                self.face_weight(Axis::ALL[d], coordinate) * pressure[row]
                            };
                        }
                        if coordinate[d] + 1 < self.grid.counts()[d] {
                            let mut neighbor = coordinate;
                            neighbor[d] += 1;
                            let other = self.grid.cell_unchecked(neighbor);
                            value += if self.is_wet(other) {
                                self.neighbor_weight(Axis::ALL[d], neighbor)
                                    * (pressure[row] - pressure[other])
                            } else {
                                self.face_weight(Axis::ALL[d], neighbor) * pressure[row]
                            };
                        }
                    }
                    if !value.is_finite() {
                        return Err(OperatorError::ArithmeticFailure);
                    }
                    out[row] = value;
                }
            }
        }
        Ok(())
    }

    /// Build the full (ungauged) RHS from signed integrated face flux.
    /// Walls must already satisfy the fixed-box contract.
    pub fn build_rhs(
        &self,
        velocity: [&[f32]; 3],
        dt: f64,
        out: &mut [f64],
    ) -> Result<(), OperatorError> {
        if !dt.is_finite() || dt <= 0.0 {
            return Err(OperatorError::InvalidTimeStep);
        }
        self.validate_velocity(velocity)?;
        if out.len() != self.grid.cell_len() {
            return Err(OperatorError::LengthMismatch);
        }
        let [nx, ny, nz] = self.grid.counts();
        for k in 0..nz {
            for j in 0..ny {
                for i in 0..nx {
                    let p = [i, j, k];
                    let cell = self.grid.cell_unchecked(p);
                    if !self.is_wet(cell) {
                        out[cell] = 0.0;
                        continue;
                    }
                    let mut flux = 0.0;
                    for axis in Axis::ALL {
                        let d = axis.index();
                        let mut upper = p;
                        upper[d] += 1;
                        let lo = self.grid.face_unchecked(axis, p);
                        let hi = self.grid.face_unchecked(axis, upper);
                        if let Some(geometry) = self.flat_columns {
                            flux += geometry.flux_area(axis, p).unwrap()
                                * f64::from(velocity[d][lo])
                                - geometry.flux_area(axis, upper).unwrap()
                                    * f64::from(velocity[d][hi]);
                        } else {
                            flux += self.face_area[d]
                                * (f64::from(velocity[d][lo]) - f64::from(velocity[d][hi]));
                        }
                    }
                    let value = flux / dt;
                    if !value.is_finite() {
                        return Err(OperatorError::ArithmeticFailure);
                    }
                    out[self.grid.cell_unchecked(p)] = value;
                }
            }
        }
        Ok(())
    }

    /// Direct physical divergence; independently scaled from the integrated
    /// RHS. This checks every cell, including a pressure gauge cell.
    pub fn divergence(&self, velocity: [&[f32]; 3], out: &mut [f64]) -> Result<(), OperatorError> {
        self.validate_velocity(velocity)?;
        if out.len() != self.grid.cell_len() {
            return Err(OperatorError::LengthMismatch);
        }
        if let Some(geometry) = self.flat_columns {
            for (cell, value) in out.iter_mut().enumerate() {
                if !self.is_wet(cell) {
                    *value = 0.0;
                    continue;
                }
                let [nx, ny, _] = self.grid.counts();
                let p = [cell % nx, (cell / nx) % ny, cell / (nx * ny)];
                let mut flux = 0.0;
                for axis in Axis::ALL {
                    let d = axis.index();
                    let mut hi = p;
                    hi[d] += 1;
                    flux += geometry.flux_area(axis, hi).unwrap()
                        * f64::from(velocity[d][self.grid.face_unchecked(axis, hi)])
                        - geometry.flux_area(axis, p).unwrap()
                            * f64::from(velocity[d][self.grid.face_unchecked(axis, p)]);
                }
                *value = flux / self.row_volume(cell);
                if !value.is_finite() {
                    return Err(OperatorError::ArithmeticFailure);
                }
            }
            return Ok(());
        }
        let [nx, ny, nz] = self.grid.counts();
        let h = self.grid.spacing();
        for k in 0..nz {
            for j in 0..ny {
                for i in 0..nx {
                    let x = velocity[0];
                    let y = velocity[1];
                    let z = velocity[2];
                    let value = (f64::from(x[self.grid.face_unchecked(Axis::X, [i + 1, j, k])])
                        - f64::from(x[self.grid.face_unchecked(Axis::X, [i, j, k])]))
                        / h[0]
                        + (f64::from(y[self.grid.face_unchecked(Axis::Y, [i, j + 1, k])])
                            - f64::from(y[self.grid.face_unchecked(Axis::Y, [i, j, k])]))
                            / h[1]
                        + (f64::from(z[self.grid.face_unchecked(Axis::Z, [i, j, k + 1])])
                            - f64::from(z[self.grid.face_unchecked(Axis::Z, [i, j, k])]))
                            / h[2];
                    if !value.is_finite() {
                        return Err(OperatorError::ArithmeticFailure);
                    }
                    out[self.grid.cell_unchecked([i, j, k])] = value;
                }
            }
        }
        Ok(())
    }

    /// Subtract the physical pressure gradient into separate f32 face buffers.
    /// Boundary faces remain zero. The rounding error is measured separately
    /// by the all-cell divergence acceptance gate.
    pub fn correct_velocity(
        &self,
        old: [&[f32]; 3],
        pressure: &[f64],
        dt: f64,
        output: [&mut [f32]; 3],
    ) -> Result<(), OperatorError> {
        if !dt.is_finite() || dt <= 0.0 {
            return Err(OperatorError::InvalidTimeStep);
        }
        self.validate_velocity(old)?;
        self.validate_cells(pressure)?;
        for axis in Axis::ALL {
            let d = axis.index();
            if output[d].len() != self.grid.face_len(axis) {
                return Err(OperatorError::LengthMismatch);
            }
        }
        for (d, output_axis) in output.into_iter().enumerate() {
            let axis = Axis::ALL[d];
            let dims = self.grid.face_counts(axis);
            for k in 0..dims[2] {
                for j in 0..dims[1] {
                    for i in 0..dims[0] {
                        let p = [i, j, k];
                        let index = self.grid.face_unchecked(axis, p);
                        if p[d] == 0 || p[d] == self.grid.counts()[d] {
                            output_axis[index] = 0.0;
                            continue;
                        }
                        let mut left = p;
                        left[d] -= 1;
                        let difference = pressure[self.grid.cell_unchecked(p)]
                            - pressure[self.grid.cell_unchecked(left)];
                        let value = if self.surface.is_some() {
                            self.correction(axis, p, pressure)
                                .map_or(0.0, |gradient| f64::from(old[d][index]) - dt * gradient)
                        } else {
                            f64::from(old[d][index]) - dt * self.acceleration_factor[d] * difference
                        };
                        if !value.is_finite() || value.abs() > f64::from(f32::MAX) {
                            return Err(OperatorError::ArithmeticFailure);
                        }
                        output_axis[index] = value as f32;
                    }
                }
            }
        }
        Ok(())
    }

    /// Correct only a disposable candidate in place. Each face depends on its
    /// own old value and immutable pressure, so no third face buffer is needed.
    /// Error can leave candidate partially written; never pass accepted state.
    pub fn correct_candidate_in_place(
        &self,
        pressure: &[f64],
        dt: f64,
        candidate: [&mut [f32]; 3],
    ) -> Result<(), OperatorError> {
        if !dt.is_finite() || dt <= 0.0 {
            return Err(OperatorError::InvalidTimeStep);
        }
        self.validate_velocity([candidate[0], candidate[1], candidate[2]])?;
        self.validate_cells(pressure)?;
        for (d, values) in candidate.into_iter().enumerate() {
            let axis = Axis::ALL[d];
            let dims = self.grid.face_counts(axis);
            for k in 0..dims[2] {
                for j in 0..dims[1] {
                    for i in 0..dims[0] {
                        let p = [i, j, k];
                        if p[d] == 0 || p[d] == self.grid.counts()[d] {
                            continue;
                        }
                        let index = self.grid.face_unchecked(axis, p);
                        let mut left = p;
                        left[d] -= 1;
                        let difference = pressure[self.grid.cell_unchecked(p)]
                            - pressure[self.grid.cell_unchecked(left)];
                        let value = if self.surface.is_some() {
                            self.correction(axis, p, pressure)
                                .map_or(0.0, |gradient| f64::from(values[index]) - dt * gradient)
                        } else {
                            f64::from(values[index]) - dt * self.acceleration_factor[d] * difference
                        };
                        if !value.is_finite() || value.abs() > f64::from(f32::MAX) {
                            return Err(OperatorError::ArithmeticFailure);
                        }
                        values[index] = value as f32;
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn diagonal(&self, coordinate: [usize; 3]) -> f64 {
        if !self.is_wet(self.grid.cell_unchecked(coordinate)) {
            return 1.0;
        }
        let mut value = 0.0;
        for d in 0..3 {
            if coordinate[d] > 0 {
                value += self.face_weight(Axis::ALL[d], coordinate);
            }
            if coordinate[d] + 1 < self.grid.counts()[d] {
                let mut hi = coordinate;
                hi[d] += 1;
                value += self.face_weight(Axis::ALL[d], hi);
            }
        }
        value
    }

    pub(crate) fn validate_cells(&self, values: &[f64]) -> Result<(), OperatorError> {
        if values.len() != self.grid.cell_len() {
            return Err(OperatorError::LengthMismatch);
        }
        if values.iter().any(|x| !x.is_finite()) {
            return Err(OperatorError::NonFiniteInput);
        }
        Ok(())
    }

    pub(crate) fn validate_velocity(&self, velocity: [&[f32]; 3]) -> Result<(), OperatorError> {
        for axis in Axis::ALL {
            let d = axis.index();
            if velocity[d].len() != self.grid.face_len(axis) {
                return Err(OperatorError::LengthMismatch);
            }
            if velocity[d].iter().any(|v| !v.is_finite()) {
                return Err(OperatorError::NonFiniteInput);
            }
            let dims = self.grid.face_counts(axis);
            for k in 0..dims[2] {
                for j in 0..dims[1] {
                    for i in 0..dims[0] {
                        let p = [i, j, k];
                        if p[d] == 0 || p[d] == self.grid.counts()[d] {
                            let index = self.grid.face_unchecked(axis, p);
                            if velocity[d][index] != 0.0 {
                                return Err(OperatorError::NonZeroWallVelocity { axis, index });
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
