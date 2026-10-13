//! Caller-owned scratch for opt-in production sealed-box provider stepping.
use crate::solver_provider::{
    PressureProvider, PressureRequest, ProviderReport, ProviderWorkspace,
};
use crate::{
    ForcedStepReport, GridGeometry, PressureImplementation, PressureOperator, PressureSettings,
    SimulationError,
};

/// Published only after the complete production simulation step is accepted.
#[derive(Debug, Clone)]
pub struct ProviderStepReport {
    pub step: ForcedStepReport,
    pub provider: ProviderReport,
    pub simulation_array_bytes: usize,
    pub provider_workspace_array_bytes: usize,
}

pub(crate) struct ProviderStepInput<'a, 'p> {
    pub(crate) workspace: &'a mut ProviderStepWorkspace,
    pub(crate) preferred: Option<&'p mut dyn PressureProvider>,
}

/// Nine cell arrays: eight in the existing dispatcher plus the assembled RHS.
/// The simulation retains its original pressure scratch. Both payloads count;
/// model/caller allocations, metadata, stack and allocator overhead are excluded.
/// Scratch is never accepted state and is overwritten on the next call.
pub struct ProviderStepWorkspace {
    grid: GridGeometry,
    density: f64,
    implementation: PressureImplementation,
    provider: ProviderWorkspace,
    rhs: Vec<f64>,
}
impl ProviderStepWorkspace {
    pub fn new(
        grid: &GridGeometry,
        density: f64,
        limit: usize,
        implementation: PressureImplementation,
    ) -> Result<Self, SimulationError> {
        PressureOperator::new(grid, density)?;
        let required = grid
            .cell_len()
            .checked_mul(8)
            .ok_or(SimulationError::AllocationFailed)?;
        if required > limit {
            return Err(SimulationError::BufferLimit { required, limit });
        }
        let mut rhs = Vec::new();
        rhs.try_reserve_exact(grid.cell_len())
            .map_err(|_| SimulationError::AllocationFailed)?;
        let required = rhs
            .capacity()
            .checked_mul(8)
            .ok_or(SimulationError::AllocationFailed)?;
        let remaining = limit
            .checked_sub(required)
            .ok_or(SimulationError::BufferLimit { required, limit })?;
        rhs.resize(grid.cell_len(), 0.0);
        let provider = ProviderWorkspace::new(grid, remaining, implementation)?;
        Ok(Self {
            grid: grid.clone(),
            density,
            implementation,
            provider,
            rhs,
        })
    }
    pub fn allocated_bytes(&self) -> usize {
        self.provider.allocated_bytes() + self.rhs.capacity() * 8
    }
    pub(crate) fn matches(
        &self,
        grid: &GridGeometry,
        density: f64,
        implementation: PressureImplementation,
    ) -> bool {
        self.grid == *grid
            && self.density.to_bits() == density.to_bits()
            && self.implementation == implementation
    }
    pub(crate) fn pressure(&self) -> &[f64] {
        self.provider.pressure()
    }
    pub(crate) fn solve(
        &mut self,
        operator: &PressureOperator<'_>,
        velocity: [&[f32]; 3],
        dt: f64,
        settings: PressureSettings,
        preferred: Option<&mut dyn PressureProvider>,
        cancel: impl FnMut() -> bool,
    ) -> Result<ProviderReport, SimulationError> {
        operator.build_rhs(velocity, dt, &mut self.rhs)?;
        Ok(self.provider.solve(
            PressureRequest {
                grid: &self.grid,
                density: self.density,
                dt,
                rhs: &self.rhs,
            },
            settings,
            preferred,
            cancel,
        )?)
    }
}
