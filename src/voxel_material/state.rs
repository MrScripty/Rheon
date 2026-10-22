use super::{allocate, capacity, check_limit, faces, payload};
use crate::{Axis, FluidPreset, GridGeometry, OperatorError, PressureError};
use std::fmt;

/// Constant SI coefficients, without particle/EOS/thermal parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaterialProperties {
    density: f64,
    viscosity: f64,
}

impl MaterialProperties {
    /// Numerical coefficient construction is useful for independent fixtures;
    /// it does not qualify arbitrary material pairs for the native product.
    pub fn new(density: f64, viscosity: f64) -> Result<Self, VoxelMaterialError> {
        if !density.is_normal()
            || density <= 0.0
            || !viscosity.is_finite()
            || viscosity < 0.0
            || !(viscosity / density).is_finite()
        {
            return Err(VoxelMaterialError::InvalidMaterial);
        }
        Ok(Self { density, viscosity })
    }
    pub fn density(self) -> f64 {
        self.density
    }
    pub fn viscosity(self) -> f64 {
        self.viscosity
    }
}

/// Liquid and occupied gas remain distinct, even when coefficients coincide.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaterialPair {
    liquid: MaterialProperties,
    gas: MaterialProperties,
}

impl MaterialPair {
    pub fn new(liquid: MaterialProperties, gas: MaterialProperties) -> Self {
        Self { liquid, gas }
    }
    /// Captures the current reference rho/mu values. No preset is looked up
    /// again during stepping, and other particle properties are not applied.
    pub fn water_air() -> Self {
        let water = FluidPreset::Water.material();
        let air = FluidPreset::Air.material();
        Self {
            liquid: MaterialProperties {
                density: water.density,
                viscosity: water.viscosity,
            },
            gas: MaterialProperties {
                density: air.density,
                viscosity: air.viscosity,
            },
        }
    }
    pub fn liquid(self) -> MaterialProperties {
        self.liquid
    }
    pub fn gas(self) -> MaterialProperties {
        self.gas
    }
    pub(crate) fn density(self, f: f64) -> f64 {
        f * self.liquid.density + (1.0 - f) * self.gas.density
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VoxelMaterialError {
    InvalidMaterial,
    UnsupportedCounts,
    LengthMismatch,
    InvalidFraction { cell: usize },
    GeometryMismatch,
    CapacityOverflow,
    BufferLimit { required: usize, limit: usize },
    AllocationFailed,
    Operator(OperatorError),
    Pressure(PressureError),
    Divergence { actual: f64, limit: f64 },
    Cancelled,
    ArithmeticFailure,
}
impl fmt::Display for VoxelMaterialError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMaterial => write!(
                f,
                "material density must be positive and normal; viscosity finite and nonnegative"
            ),
            Self::UnsupportedCounts => write!(f, "two-material counts must be 4–64 per axis"),
            Self::LengthMismatch => write!(f, "two-material field length differs from the grid"),
            Self::InvalidFraction { cell } => write!(
                f,
                "liquid fraction at cell {cell} must be finite and in [0,1]"
            ),
            Self::GeometryMismatch => write!(f, "projection workspace geometry differs from state"),
            Self::CapacityOverflow => write!(f, "two-material capacity arithmetic overflowed"),
            Self::BufferLimit { required, limit } => write!(
                f,
                "two-material managed payload requires {required} bytes, limit {limit}"
            ),
            Self::AllocationFailed => write!(f, "two-material allocation failed"),
            Self::Operator(e) => e.fmt(f),
            Self::Pressure(e) => e.fmt(f),
            Self::Divergence { actual, limit } => {
                write!(f, "stored divergence {actual} exceeds {limit}")
            }
            Self::Cancelled => write!(f, "two-material operation cancelled before publication"),
            Self::ArithmeticFailure => write!(f, "two-material arithmetic is not representable"),
        }
    }
}
impl std::error::Error for VoxelMaterialError {}
impl From<OperatorError> for VoxelMaterialError {
    fn from(e: OperatorError) -> Self {
        Self::Operator(e)
    }
}
impl From<PressureError> for VoxelMaterialError {
    fn from(e: PressureError) -> Self {
        Self::Pressure(e)
    }
}

/// Authoritative numerical lattice at rest. F=0 is occupied gas, not vacuum.
/// Partial initial fractions are a numerical-fixture interface; the native
/// authoring boundary will admit only exact Water/Air binary conversion.
pub struct VoxelMaterialState {
    pub(crate) grid: GridGeometry,
    pub(crate) materials: MaterialPair,
    pub(crate) liquid_fraction: Vec<f64>,
    pub(crate) velocity: [Vec<f64>; 3],
    pub(crate) pressure: Vec<f64>,
    pub(crate) pressure_available: bool,
    pub(crate) generation: u64,
    pub(crate) time: f64,
    allocated_bytes: usize,
}
impl VoxelMaterialState {
    pub fn new(
        grid: GridGeometry,
        materials: MaterialPair,
        liquid_fraction: &[f64],
        limit: usize,
    ) -> Result<Self, VoxelMaterialError> {
        if grid.counts().iter().any(|n| !(4..=64).contains(n)) {
            return Err(VoxelMaterialError::UnsupportedCounts);
        }
        if liquid_fraction.len() != grid.cell_len() {
            return Err(VoxelMaterialError::LengthMismatch);
        }
        for (cell, &f) in liquid_fraction.iter().enumerate() {
            if !f.is_finite() || !(0.0..=1.0).contains(&f) {
                return Err(VoxelMaterialError::InvalidFraction { cell });
            }
            let rho = materials.density(f);
            if !rho.is_normal() || !(rho * grid.cell_volume()).is_normal() {
                return Err(VoxelMaterialError::ArithmeticFailure);
            }
        }
        check_limit(payload(2, 1, &grid)?, limit)?;
        let mut fractions = allocate(grid.cell_len())?;
        fractions.copy_from_slice(liquid_fraction);
        let pressure = allocate(grid.cell_len())?;
        let velocity = faces(&grid)?;
        let allocated_bytes = capacity([&fractions, &pressure].into_iter().chain(velocity.iter()))?;
        check_limit(allocated_bytes, limit)?;
        Ok(Self {
            grid,
            materials,
            liquid_fraction: fractions,
            velocity,
            pressure,
            pressure_available: false,
            generation: 0,
            time: 0.0,
            allocated_bytes,
        })
    }
    pub fn geometry(&self) -> &GridGeometry {
        &self.grid
    }
    pub fn materials(&self) -> MaterialPair {
        self.materials
    }
    pub fn liquid_fraction(&self) -> &[f64] {
        &self.liquid_fraction
    }
    pub fn velocity(&self, axis: Axis) -> &[f64] {
        &self.velocity[axis.index()]
    }
    pub fn pressure(&self) -> Option<&[f64]> {
        self.pressure_available.then_some(&self.pressure)
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn time(&self) -> f64 {
        self.time
    }
    pub fn allocated_bytes(&self) -> usize {
        self.allocated_bytes
    }
    pub fn density(&self, cell: usize) -> Option<f64> {
        self.liquid_fraction
            .get(cell)
            .map(|&f| self.materials.density(f))
    }
}
