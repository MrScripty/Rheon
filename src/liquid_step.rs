//! Transactional fixed-box carrier and represented-volume transport. Fractions
//! classify pressure only on opt-in surface modes; inertia remains one constant.
use crate::{
    Axis, BodyForce, BoxFluxStepBoundary, BoxFluxStepReport, BoxFluxStepWorkspace, BufferPlan,
    ForcedStepReport, GridGeometry, LiquidFlowInterval, LiquidInlet, LiquidVolumeError,
    LiquidVolumeReport, LiquidVolumeSettings, LiquidVolumeSource, LiquidVolumeState,
    LiquidVolumeView, PressureImplementation, Simulation, SimulationConfig, SimulationError,
    SmokeSource, StateView, StepStage, VolumeStage, VolumeStamp,
};
use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct LiquidTransportConfig {
    /// memory_limit caps all facade-owned arrays, including accepted pressure
    /// and transferred initial fraction capacity. Boundary scratch is separate.
    pub carrier: SimulationConfig,
    /// Independent constant in full-box mode; surface modes require equality
    /// with carrier density. No two-phase inertia.
    pub represented_density: f64,
    pub volume_stamp: VolumeStamp,
    /// Caller-owned identity for this carrier's published generations.
    pub carrier_id: u64,
}

#[derive(Clone, Copy)]
pub struct LiquidStepInputs<'a> {
    pub requested_dt: f64,
    pub smoke_source: Option<SmokeSource>,
    pub forces: &'a [BodyForce],
    pub inlet: LiquidInlet,
    pub source: Option<LiquidVolumeSource<'a>>,
    pub volume: LiquidVolumeSettings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiquidStepStage {
    Carrier(StepStage),
    Volume(VolumeStage),
    Reconstruction(crate::ReconstructionStage),
    /// Last callback; neither owner has published yet. No callbacks or fallible
    /// operations run after this checkpoint accepts the candidate pair.
    BeforeCommit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiquidStepError {
    Carrier(SimulationError),
    Volume(LiquidVolumeError),
    Cancelled,
    AllocationFailed,
    FreeSurface(crate::FreeSurfaceError),
    ColumnMac(crate::ColumnMacError),
    BufferLimit { required: usize, limit: usize },
}
impl fmt::Display for LiquidStepError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "coupled liquid transport rejected operation: {self:?}")
    }
}
impl std::error::Error for LiquidStepError {}
impl From<SimulationError> for LiquidStepError {
    fn from(e: SimulationError) -> Self {
        Self::Carrier(e)
    }
}
impl From<LiquidVolumeError> for LiquidStepError {
    fn from(e: LiquidVolumeError) -> Self {
        Self::Volume(e)
    }
}

#[derive(Clone, Copy)]
pub struct LiquidTransportView<'a> {
    pub carrier: StateView<'a>,
    pub liquid: LiquidVolumeView<'a>,
    /// Last accepted interval projection pressure, in Pa. Closed mode fixes a
    /// gauge; surface modes use their declared atmospheric geometry. Initial
    /// rest state uses zero pressure. Failed preparation cannot change this view.
    pub pressure: &'a [f64],
    pub carrier_stamp: VolumeStamp,
    /// Immutable pressure geometry used during this interval, not reconstructed
    /// geometry inferred from transported end fractions.
    pub pressure_surface: Option<&'a crate::SlabFreeSurface>,
    pub reconstructed_surface: Option<crate::ColumnSurfaceView<'a>>,
    /// Geometry of the held interval pressure, distinct from end geometry.
    pub pressure_columns: Option<crate::ColumnSurfaceView<'a>>,
    /// Static materialization mode: MAC liquid masses and pressure share end
    /// geometry. Dynamics remain refused until a compatible step is supplied.
    pub flat_column_mac: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct LiquidStepReport {
    pub carrier: ForcedStepReport,
    pub boundary: Option<BoxFluxStepReport>,
    pub liquid: LiquidVolumeReport,
    pub owned_array_bytes: usize,
    pub boundary_workspace_array_bytes: usize,
    pub total_array_bytes: usize,
    pub pressure_surface: Option<VolumeStamp>,
    pub column_reconstruction: Option<crate::ColumnReconstructionReport>,
    pub viscosity: Option<crate::ViscosityReport>,
    pub viscosity_workspace_array_bytes: usize,
}

/// Owns the existing Simulation and LiquidVolumeState and one explicitly
/// counted accepted-pressure array. No mutable access to either accepted owner
/// is exposed. Stepping reuses candidates and scratch; no rollback snapshots.
/// The optional borrowed boundary workspace retains its own capacity cap.
pub struct LiquidTransportSimulation {
    carrier: Simulation,
    liquid: LiquidVolumeState,
    pressure: Vec<f64>,
    carrier_id: u64,
    allocated_bytes: usize,
    surface: Option<crate::SlabFreeSurface>,
    columns: Option<crate::ColumnSurfaceWorkspace>,
    flat_column_mac: bool,
}
impl LiquidTransportSimulation {
    pub fn new(
        grid: GridGeometry,
        config: LiquidTransportConfig,
        implementation: PressureImplementation,
        fraction: Vec<f64>,
    ) -> Result<Self, LiquidStepError> {
        let limit = config.carrier.memory_limit;
        let faces = Axis::ALL.into_iter().try_fold(0_usize, |n, a| {
            n.checked_add(grid.face_len(a))
                .ok_or(LiquidStepError::AllocationFailed)
        })?;
        let volume_min = fraction
            .capacity()
            .checked_add(grid.cell_len())
            .and_then(|n| n.checked_add(faces))
            .and_then(|n| n.checked_mul(8))
            .ok_or(LiquidStepError::AllocationFailed)?;
        let pressure_min = grid
            .cell_len()
            .checked_mul(8)
            .ok_or(LiquidStepError::AllocationFailed)?;
        let carrier_min = BufferPlan::for_grid(&grid, usize::MAX)
            .map_err(SimulationError::from)?
            .total_bytes;
        let required = carrier_min
            .checked_add(volume_min)
            .and_then(|n| n.checked_add(pressure_min))
            .ok_or(LiquidStepError::AllocationFailed)?;
        if required > limit {
            return Err(LiquidStepError::BufferLimit { required, limit });
        }
        let mut carrier_config = config.carrier;
        carrier_config.memory_limit = limit - volume_min - pressure_min;
        let carrier =
            Simulation::with_implementation(grid.clone(), carrier_config, implementation)?;
        let liquid = LiquidVolumeState::new(
            grid,
            config.represented_density,
            config.volume_stamp,
            fraction,
            limit - carrier.allocated_bytes() - pressure_min,
        )?;
        let mut remaining = limit - carrier.allocated_bytes() - liquid.allocated_bytes();
        let pressure = crate::liquid_volume::allocate(carrier.grid().cell_len(), &mut remaining)?;
        Ok(Self {
            carrier,
            liquid,
            pressure,
            carrier_id: config.carrier_id,
            allocated_bytes: limit - remaining,
            surface: None,
            columns: None,
            flat_column_mac: false,
        })
    }
    /// One-liquid resolved slab pressure mode. Mixed interface geometry is not
    /// reconstructed; subsequent steps reject when binary admission is lost.
    pub fn with_free_surface(
        grid: GridGeometry,
        config: LiquidTransportConfig,
        implementation: PressureImplementation,
        fraction: Vec<f64>,
        surface: crate::SlabFreeSurface,
    ) -> Result<Self, LiquidStepError> {
        if config.carrier.density.to_bits() != config.represented_density.to_bits() {
            return Err(LiquidStepError::FreeSurface(
                crate::FreeSurfaceError::DensityMismatch,
            ));
        }
        surface
            .validate_fractions(&grid, &fraction)
            .map_err(LiquidStepError::FreeSurface)?;
        crate::PressureOperator::with_free_surface(&grid, config.carrier.density, &surface)
            .map_err(SimulationError::from)?;
        let mut state = Self::new(grid, config, implementation, fraction)?;
        state.surface = Some(surface);
        Ok(state)
    }
    /// Opt-in lower-wall column-height geometry with geometric axial transfer
    /// and staged reconstruction. Does not alter the frozen binary-slab mode.
    pub fn with_reconstructed_surface(
        grid: GridGeometry,
        config: LiquidTransportConfig,
        implementation: PressureImplementation,
        fraction: Vec<f64>,
        axis: Axis,
    ) -> Result<Self, LiquidStepError> {
        if config.carrier.density.to_bits() != config.represented_density.to_bits() {
            return Err(LiquidStepError::FreeSurface(
                crate::FreeSurfaceError::DensityMismatch,
            ));
        }
        // Preflight transferred fraction capacity together with both owners
        // and nominal geometry before allocating any reconstruction arrays.
        let faces = Axis::ALL.into_iter().try_fold(0_usize, |n, a| {
            n.checked_add(grid.face_len(a))
                .ok_or(LiquidStepError::AllocationFailed)
        })?;
        let volume_min = fraction
            .capacity()
            .checked_add(grid.cell_len())
            .and_then(|n| n.checked_add(faces))
            .and_then(|n| n.checked_mul(8))
            .ok_or(LiquidStepError::AllocationFailed)?;
        let pressure_min = grid
            .cell_len()
            .checked_mul(8)
            .ok_or(LiquidStepError::AllocationFailed)?;
        let carrier_min = BufferPlan::for_grid(&grid, usize::MAX)
            .map_err(SimulationError::from)?
            .total_bytes;
        let owners_min = carrier_min
            .checked_add(volume_min)
            .and_then(|n| n.checked_add(pressure_min))
            .ok_or(LiquidStepError::AllocationFailed)?;
        let geometry_min = (grid.cell_len() / grid.counts()[axis.index()])
            .checked_mul(3)
            .and_then(|n| n.checked_mul(std::mem::size_of::<crate::ColumnHeight>()))
            .ok_or(LiquidStepError::AllocationFailed)?;
        let required = owners_min
            .checked_add(geometry_min)
            .ok_or(LiquidStepError::AllocationFailed)?;
        if required > config.carrier.memory_limit {
            return Err(LiquidStepError::BufferLimit {
                required,
                limit: config.carrier.memory_limit,
            });
        }
        let columns = crate::ColumnSurfaceWorkspace::new(
            grid.clone(),
            axis,
            &fraction,
            config.volume_stamp,
            config.carrier.memory_limit - owners_min,
        )
        .map_err(LiquidStepError::FreeSurface)?;
        crate::PressureOperator::with_columns(&grid, config.carrier.density, columns.state())
            .map_err(SimulationError::from)?;
        let bytes = columns.allocated_bytes();
        let limit = config.carrier.memory_limit;
        let mut reduced = config;
        reduced.carrier.memory_limit = limit - bytes;
        let mut owner =
            Self::new(grid, reduced, implementation, fraction).map_err(|error| match error {
                LiquidStepError::BufferLimit { required, .. } => {
                    match required.checked_add(bytes) {
                        Some(required) => LiquidStepError::BufferLimit { required, limit },
                        None => LiquidStepError::AllocationFailed,
                    }
                }
                other => other,
            })?;
        owner.allocated_bytes += bytes;
        owner.columns = Some(columns);
        Ok(owner)
    }
    pub fn grid(&self) -> &GridGeometry {
        self.carrier.grid()
    }
    pub fn implementation(&self) -> PressureImplementation {
        self.carrier.implementation()
    }
    pub fn allocated_bytes(&self) -> usize {
        self.allocated_bytes
    }
    pub fn state(&self) -> LiquidTransportView<'_> {
        let carrier = self.carrier.state();
        LiquidTransportView {
            carrier,
            liquid: self.liquid.state(),
            pressure: &self.pressure,
            pressure_surface: self.surface.as_ref(),
            reconstructed_surface: self.columns.as_ref().map(|s| s.state()),
            pressure_columns: self.columns.as_ref().map(|s| s.pressure_geometry()),
            flat_column_mac: self.flat_column_mac,
            carrier_stamp: VolumeStamp {
                id: self.carrier_id,
                version: carrier.generation,
            },
        }
    }
    /// Export tangential parcel averages from this accepted static MAC state.
    /// The report retains the actual carrier/volume revision pair and explicitly
    /// accounts for omitted normal momentum/energy. This is a lossy restriction.
    pub fn export_flat_column_profiles(
        &self,
        workspace: &mut crate::ColumnMacWorkspace,
        expected: crate::ColumnMacStateStamp,
        output: [&mut [f32]; 2],
        cancel: impl FnMut(crate::ColumnMacStage) -> bool,
    ) -> Result<crate::ColumnMacExportReport, LiquidStepError> {
        let state = self.state();
        let source = crate::ColumnMacStateStamp {
            carrier: state.carrier_stamp,
            volume: state.liquid.stamp,
        };
        if source != expected {
            return Err(LiquidStepError::ColumnMac(
                crate::ColumnMacError::StampMismatch,
            ));
        }
        if !self.flat_column_mac {
            return Err(LiquidStepError::ColumnMac(
                crate::ColumnMacError::UnsupportedState,
            ));
        }
        let geometry = crate::FlatColumnMacGeometry::new(
            state.reconstructed_surface.unwrap(),
            state.liquid.density,
        )
        .map_err(LiquidStepError::ColumnMac)?;
        let transfer = workspace
            .restrict(
                geometry,
                [state.carrier.x, state.carrier.y, state.carrier.z],
                output,
                cancel,
            )
            .map_err(LiquidStepError::ColumnMac)?;
        Ok(crate::ColumnMacExportReport { source, transfer })
    }
    /// Materialize prescribed tangential parcel profiles into the existing
    /// accepted MAC/pressure owners at unchanged physical time and geometry.
    /// Requires a fresh rest carrier, or this same static materialization mode.
    /// Both owner revisions and pressure/momentum geometry publish together.
    pub fn materialize_flat_column_profiles(
        &mut self,
        workspace: &mut crate::ColumnMacWorkspace,
        inputs: crate::ColumnMacPublishInputs<'_>,
        mut cancel: impl FnMut(crate::ColumnMacStage) -> bool,
    ) -> Result<crate::ColumnMacPublicationReport, LiquidStepError> {
        let failure = LiquidStepError::ColumnMac;
        let before = crate::ColumnMacStateStamp {
            carrier: VolumeStamp {
                id: self.carrier_id,
                version: self.carrier.state().generation,
            },
            volume: self.liquid.state().stamp,
        };
        if inputs.expected != before {
            return Err(failure(crate::ColumnMacError::StampMismatch));
        }
        if !self.flat_column_mac && self.carrier.state().generation != 0 {
            return Err(failure(crate::ColumnMacError::UnsupportedState));
        }
        let volume = VolumeStamp {
            id: before.volume.id,
            version: before
                .volume
                .version
                .checked_add(1)
                .ok_or_else(|| failure(crate::ColumnMacError::VersionOverflow))?,
        };
        let columns = self
            .columns
            .as_ref()
            .ok_or_else(|| failure(crate::ColumnMacError::UnsupportedState))?;
        if self.carrier.density().to_bits() != self.liquid.state().density.to_bits()
            || !columns.state().matches(
                self.carrier.grid(),
                self.liquid.state().fraction,
                before.volume,
            )
        {
            return Err(failure(crate::ColumnMacError::GeometryMismatch));
        }
        let geometry = crate::FlatColumnMacGeometry::new(columns.state(), self.carrier.density())
            .map_err(failure)?;
        let old = self.carrier.state();
        let (accepted_momentum_before, accepted_energy_before) = workspace
            .measure_velocity(geometry, [old.x, old.y, old.z])
            .map_err(failure)?;
        let total_array_bytes = self
            .allocated_bytes
            .checked_add(workspace.allocated_bytes())
            .ok_or(LiquidStepError::AllocationFailed)?;
        let (transfer, projection, generation) = self
            .carrier
            .prepare_column_mac(
                geometry,
                workspace,
                inputs.profiles,
                inputs.projection_dt,
                &mut cancel,
            )
            .map_err(failure)?;
        let represented_phase_mass = self.liquid.column_mac_represented_mass()?;
        let phase_mass_error =
            crate::column_mac::check(transfer.profile_mass - represented_phase_mass)
                .map_err(failure)?;
        let phase_mass_budget = crate::column_mac::check(
            64.0 * f64::EPSILON * (transfer.profile_mass + represented_phase_mass),
        )
        .map_err(failure)?;
        if phase_mass_error.abs() > phase_mass_budget {
            return Err(failure(crate::ColumnMacError::AcceptanceFailure));
        }
        let mut prescribed_momentum_change = accepted_momentum_before.map(|p| -p);
        let tangents = match geometry.normal() {
            Axis::X => [1, 2],
            Axis::Y => [0, 2],
            Axis::Z => [0, 1],
        };
        for (t, d) in tangents.into_iter().enumerate() {
            prescribed_momentum_change[d] += transfer.momentum_before[t];
        }
        let prescribed_energy_change = transfer.kinetic_before - accepted_energy_before;
        for &value in &prescribed_momentum_change {
            crate::column_mac::check(value).map_err(failure)?;
        }
        crate::column_mac::check(prescribed_energy_change).map_err(failure)?;
        if cancel(crate::ColumnMacStage::BeforePublish) {
            return Err(failure(crate::ColumnMacError::Cancelled {
                stage: crate::ColumnMacStage::BeforePublish,
            }));
        }
        self.pressure
            .copy_from_slice(self.carrier.candidate_pressure());
        self.carrier.commit_column_mac(generation);
        self.liquid.commit_column_mac_revision(volume);
        self.columns
            .as_mut()
            .unwrap()
            .commit_column_mac_revision(volume);
        self.flat_column_mac = true;
        Ok(crate::ColumnMacPublicationReport {
            before,
            after: crate::ColumnMacStateStamp {
                carrier: VolumeStamp {
                    id: self.carrier_id,
                    version: generation,
                },
                volume,
            },
            transfer,
            projection,
            prescribed_momentum_change,
            prescribed_energy_change,
            accepted_momentum_before,
            accepted_energy_before,
            represented_phase_mass,
            phase_mass_error,
            phase_mass_budget,
            owned_array_bytes: self.allocated_bytes,
            workspace_array_bytes: workspace.allocated_bytes(),
            total_array_bytes,
        })
    }
    pub fn set_paused(&mut self, paused: bool) {
        self.carrier.set_paused(paused);
    }
    /// Closed carrier box. Inlet fractions have no effect on zero outer flux.
    pub fn step(
        &mut self,
        inputs: LiquidStepInputs<'_>,
        cancel: impl FnMut(LiquidStepStage) -> bool,
    ) -> Result<LiquidStepReport, LiquidStepError> {
        self.step_impl(inputs, None, None, cancel)
    }
    /// End-of-step prescribed carrier box flux and explicit liquid inflow
    /// donors. Appearance tracer follows the existing selected boundary policy.
    pub fn step_with_box_flux(
        &mut self,
        inputs: LiquidStepInputs<'_>,
        workspace: &mut BoxFluxStepWorkspace,
        boundary: BoxFluxStepBoundary,
        cancel: impl FnMut(LiquidStepStage) -> bool,
    ) -> Result<LiquidStepReport, LiquidStepError> {
        self.step_impl(inputs, Some((workspace, boundary)), None, cancel)
    }
    /// Fully filled Newtonian liquid in a sealed box with free-slip stationary
    /// walls. mu is dynamic viscosity (Pa s). Surface geometry, box flux,
    /// unequal material density and phase sources require another model.
    pub fn step_viscous(
        &mut self,
        inputs: LiquidStepInputs<'_>,
        workspace: &mut crate::ViscosityWorkspace,
        dynamic_viscosity: f64,
        cancel: impl FnMut(LiquidStepStage) -> bool,
    ) -> Result<LiquidStepReport, LiquidStepError> {
        if self.surface.is_some()
            || self.columns.is_some()
            || inputs.source.is_some()
            || self.liquid.state().density.to_bits() != self.carrier.density().to_bits()
            || self.liquid.state().fraction.iter().any(|&f| f != 1.0)
            || !workspace.matches(self.carrier.grid())
        {
            return Err(
                SimulationError::Viscosity(crate::ViscosityError::UnsupportedDomain).into(),
            );
        }
        self.step_impl(inputs, None, Some((workspace, dynamic_viscosity)), cancel)
    }
    fn step_impl(
        &mut self,
        inputs: LiquidStepInputs<'_>,
        mut boundary: Option<(&mut BoxFluxStepWorkspace, BoxFluxStepBoundary)>,
        viscosity: Option<(&mut crate::ViscosityWorkspace, f64)>,
        mut cancel: impl FnMut(LiquidStepStage) -> bool,
    ) -> Result<LiquidStepReport, LiquidStepError> {
        if self.flat_column_mac {
            return Err(LiquidStepError::ColumnMac(
                crate::ColumnMacError::UnsupportedState,
            ));
        }
        if self.surface.is_some() || self.columns.is_some() {
            if boundary.is_some() {
                return Err(LiquidStepError::FreeSurface(
                    crate::FreeSurfaceError::UnsupportedBoxFlux,
                ));
            }
            if inputs.smoke_source.is_some() {
                return Err(LiquidStepError::FreeSurface(
                    crate::FreeSurfaceError::UnsupportedSmokeSource,
                ));
            }
            if self.liquid.state().density.to_bits() != self.carrier.density().to_bits() {
                return Err(LiquidStepError::FreeSurface(
                    crate::FreeSurfaceError::DensityMismatch,
                ));
            }
            if let Some(surface) = &self.surface {
                surface
                    .validate_fractions(self.carrier.grid(), self.liquid.state().fraction)
                    .map_err(LiquidStepError::FreeSurface)?;
            }
            if self.columns.as_ref().is_some_and(|surface| {
                !surface.state().matches(
                    self.carrier.grid(),
                    self.liquid.state().fraction,
                    self.liquid.state().stamp,
                )
            }) {
                return Err(LiquidStepError::FreeSurface(
                    crate::FreeSurfaceError::GeometryMismatch,
                ));
            }
        }
        self.liquid.validate_advance(inputs.source, inputs.volume)?;
        let boundary_bytes = boundary.as_ref().map_or(0, |(w, _)| w.allocated_bytes());
        let viscosity_bytes = viscosity.as_ref().map_or(0, |(w, _)| w.allocated_bytes());
        let total_bytes = self
            .allocated_bytes
            .checked_add(boundary_bytes)
            .and_then(|n| n.checked_add(viscosity_bytes))
            .ok_or(LiquidStepError::AllocationFailed)?;
        let candidate = self.carrier.prepare_liquid_carrier(
            inputs.requested_dt,
            inputs.smoke_source,
            inputs.forces,
            boundary.as_mut().map(|(w, b)| (&mut **w, *b)),
            self.surface
                .as_ref()
                .map(crate::column_surface::PressureSurface::Slab)
                .or_else(|| {
                    self.columns
                        .as_ref()
                        .map(|s| crate::column_surface::PressureSurface::Columns(s.state()))
                }),
            viscosity,
            |stage| cancel(LiquidStepStage::Carrier(stage)),
        )?;
        let step = candidate.legacy.step.step;
        let mut flow = LiquidFlowInterval::new(
            self.carrier.grid(),
            VolumeStamp {
                id: self.carrier_id,
                version: step.generation,
            },
            self.carrier.candidate_velocity(),
            self.carrier.state().time,
            step.dt,
        )?;
        if let Some(surface) = &self.surface {
            flow = flow.on_slab(surface);
        }
        if let Some(surface) = &self.columns {
            flow = flow.on_columns(surface.state());
        }
        let mut liquid = self.liquid.prepare_advance(
            flow,
            inputs.inlet,
            inputs.source,
            inputs.volume,
            |stage| cancel(LiquidStepStage::Volume(stage)),
        )?;
        let column_reconstruction = if let Some(surface) = &mut self.columns {
            let report = surface
                .prepare(
                    self.liquid.candidate_fraction_mut(),
                    liquid.stamp,
                    |stage| cancel(LiquidStepStage::Reconstruction(stage)),
                )
                .map_err(LiquidStepError::FreeSurface)?;
            self.liquid
                .requalify_reconstruction(&mut liquid, inputs.source)?;
            crate::PressureOperator::with_columns(
                self.carrier.grid(),
                self.carrier.density(),
                surface.candidate(report.output_stamp),
            )
            .map_err(SimulationError::from)?;
            Some(report)
        } else {
            None
        };
        if cancel(LiquidStepStage::BeforeCommit) {
            return Err(LiquidStepError::Cancelled);
        }
        // All sizes, times, versions and numeric gates have already passed.
        // Pressure copy and owner swaps are infallible and invoke no callbacks.
        let pressure = match &boundary {
            Some((w, _)) => w.projection.pressure(),
            None => self.carrier.candidate_pressure(),
        };
        self.pressure.copy_from_slice(pressure);
        self.carrier.commit_prepared(&candidate);
        if let (Some(surface), Some(report)) = (&mut self.columns, column_reconstruction) {
            surface.commit(report);
        }
        self.liquid.commit_prepared(&liquid);
        Ok(LiquidStepReport {
            carrier: candidate.legacy.step,
            boundary: candidate.boundary,
            liquid,
            owned_array_bytes: self.allocated_bytes,
            boundary_workspace_array_bytes: boundary_bytes,
            total_array_bytes: total_bytes,
            pressure_surface: self.surface.as_ref().map(|s| s.stamp()),
            column_reconstruction,
            viscosity: candidate.viscosity,
            viscosity_workspace_array_bytes: viscosity_bytes,
        })
    }
}
