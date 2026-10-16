//! One frozen accepted interval through existing Session and CPU owners.
//! This component owns no outer loop, recorder, model history or supervisor.
use super::{
    accepted_provider::AcceptedCpuProvider,
    adapter::AdapterPackage,
    model_process::{ChildSlot, StartupFailure},
};
use crate::run_sets::configured::{
    DeploymentLocator, ExecutorDeclaration, ForceSpec, FrozenConfiguredSpec, PressureIntent,
    ViscosityIntent,
};
use rheon::{
    BodyForce, GridGeometry, LiquidInlet, LiquidStepInputs, LiquidStepReport,
    PressureImplementation, ProviderStepWorkspace, SessionError, SessionIdentity, SessionSnapshot,
    SimulationSession, ViscosityWorkspace, VolumeStamp,
    recording::diagnostic::NoCandidateReason,
    solver_provider::{ProviderRejection, configured::ContextPressureProvider},
};
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

/// Exact typed runtime observation for the latest call, separate from the
/// Session's coarse accepted pressure receipt. No previous-attempt queue exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConfiguredFailure {
    pub reason: NoCandidateReason,
    pub detail: String,
    /// Existing process request initiation; this does not prove child consumption.
    pub dispatched: bool,
    pub invalid_payload_sha256: Option<[u8; 32]>,
}

/// Numerical Vec payload, excluding recipe/identity metadata, child/protocol
/// allocations, allocator overhead and RSS. Caps apply per owner, never pooled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HostArrayBytes {
    pub session: usize,
    pub pressure_workspace: usize,
    pub viscosity_workspace: usize,
    pub cpu_adapter_scratch: usize,
}
impl HostArrayBytes {
    pub(crate) fn total(self) -> usize {
        // Frozen geometry is capped at 65,536 cells; each existing owner caps
        // its fixed field inventory independently before allocation.
        self.session + self.pressure_workspace + self.viscosity_workspace + self.cpu_adapter_scratch
    }
}

pub(crate) struct ConfiguredTrack {
    frozen: FrozenConfiguredSpec,
    session: SimulationSession,
    pressure: ProviderStepWorkspace,
    viscosity: Option<ViscosityWorkspace>,
    forces: Vec<BodyForce>,
    inlet: LiquidInlet,
    cancelled: Arc<AtomicBool>,
    provider: Option<AcceptedCpuProvider>,
    failure: Option<ConfiguredFailure>,
    runtime_observation: Option<rheon::recording::provider::runtime::RuntimeObservation>,
}
impl ConfiguredTrack {
    /// Validate exact executor custody before allocating state. No filesystem
    /// output, package resolution, interpreter or model code is involved.
    pub(crate) fn prepare(
        frozen: FrozenConfiguredSpec,
        fresh_identity: SessionIdentity,
        current_executor: &ExecutorDeclaration,
        cancelled: Arc<AtomicBool>,
    ) -> Result<Self, String> {
        frozen
            .validate_execution(current_executor)
            .map_err(|e| e.to_string())?;
        let session = frozen
            .create_session(fresh_identity)
            .map_err(|e| e.to_string())?;
        let grid = session.snapshot().grid().clone();
        // This cap funds the core workspace independently of the later CPU
        // bridge's two max_cells arrays. It is not a combined memory/RSS cap.
        let pressure = ProviderStepWorkspace::new(
            &grid,
            frozen.constructor.density_kg_m3,
            frozen.future.provider_workspace_limit_bytes,
            frozen.constructor.pressure_method.implementation(),
        )
        .map_err(|e| e.to_string())?;
        let viscosity = match frozen.future.viscosity {
            ViscosityIntent::Disabled => None,
            ViscosityIntent::NewtonianConstant { .. } => Some(
                ViscosityWorkspace::new(
                    grid,
                    frozen
                        .future
                        .viscosity_workspace_limit_bytes
                        .ok_or("Missing Newtonian workspace cap")?,
                )
                .map_err(|e| e.to_string())?,
            ),
        };
        let mut forces = Vec::new();
        forces
            .try_reserve_exact(frozen.future.forces.len())
            .map_err(|_| "Force recipe allocation failed")?;
        forces.extend(frozen.future.forces.iter().map(ForceSpec::body_force));
        // Deterministic sealed profile mapping, not additional content/history:
        // choose a zero-inlet identity distinct from the two constructor ids.
        let inlet_id = (0..=2)
            .find(|id| {
                *id != frozen.constructor.carrier_id && *id != frozen.constructor.volume_stamp.id
            })
            .ok_or("Sealed inlet identity unavailable")?;
        let inlet = LiquidInlet::new(
            VolumeStamp {
                id: inlet_id,
                version: 0,
            },
            [[0.0; 2]; 3],
        )
        .map_err(|e| e.to_string())?;
        Ok(Self {
            frozen,
            session,
            pressure,
            viscosity,
            forces,
            inlet,
            cancelled,
            provider: None,
            failure: None,
            runtime_observation: None,
        })
    }

    /// Mutable package custody is checked only after prepare. The caller can
    /// persist frozen intent/header/constructor before this fallible startup.
    /// Prefix-zero packages requiring prior pressure are refused here; actual
    /// later request support and history remain the bridge/Session's authority.
    pub(crate) fn start_provider(
        &mut self,
        project_root: &Path,
        slot: &ChildSlot,
        progress: impl FnMut(&str) -> Result<(), String>,
    ) -> Result<(), StartupFailure> {
        let PressureIntent::NativePrefixThenConfigured {
            provider,
            deployment,
            requested_native_prefix_steps,
            minimum_native_prefix_steps,
        } = &self.frozen.future.pressure
        else {
            return Ok(());
        };
        if self.provider.is_some() {
            return Err("Configured CPU runtime already started".into());
        }
        if slot
            .lock()
            .map_err(|_| "Configured child slot lock poisoned")?
            .is_some()
        {
            return Err("Configured child slot already has an owner".into());
        }
        deployment.validate().map_err(|e| e.to_string())?;
        let path = match deployment {
            DeploymentLocator::ProjectRelative(relative) => project_root.join(relative),
            DeploymentLocator::Absolute(path) => path.into(),
        }
        .canonicalize()
        .map_err(|e| e.to_string())?;
        let package = AdapterPackage::load(&path).map_err(|e| e.to_string())?;
        if package.context().minimum_native_steps != *minimum_native_prefix_steps {
            return Err("CPU package minimum native prefix differs from frozen intent".into());
        }
        let mu = match self.frozen.future.viscosity {
            ViscosityIntent::Disabled => 0.0,
            ViscosityIntent::NewtonianConstant {
                dynamic_viscosity_pa_s,
            } => dynamic_viscosity_pa_s,
        };
        package
            .validate_support(
                self.geometry(),
                self.frozen.constructor.density_kg_m3,
                mu,
                self.frozen.future.requested_dt_s,
                *requested_native_prefix_steps,
                *requested_native_prefix_steps > 0,
            )
            .map_err(|e| e.to_string())?;
        let runtime = AcceptedCpuProvider::start(
            &path,
            Arc::new(provider.identity()),
            Arc::clone(slot),
            Arc::clone(&self.cancelled),
            self.frozen.future.provider_workspace_limit_bytes,
            progress,
        )?;
        self.provider = Some(runtime);
        Ok(())
    }

    /// Exactly one requested interval. All numerical gates, native fallback,
    /// accepted history and the single commit remain inside the existing Session.
    pub(crate) fn advance(
        &mut self,
        deadline: Option<Instant>,
    ) -> Result<LiquidStepReport, SessionError> {
        self.failure = None;
        self.runtime_observation = None;
        if let Some(provider) = &mut self.provider {
            provider.begin_interval_observation();
        }
        let frame = self.session.snapshot().frame();
        if frame >= self.frozen.future.requested_steps {
            return Err(SessionError::StepLimit);
        }
        let use_provider = match &self.frozen.future.pressure {
            PressureIntent::NativeOnly => false,
            PressureIntent::NativePrefixThenConfigured {
                requested_native_prefix_steps,
                ..
            } => frame >= *requested_native_prefix_steps,
        };
        if use_provider && self.provider.is_none() {
            return Err(SessionError::PressureBinding(
                ProviderRejection::Unsupported("Configured runtime unavailable or retired"),
            ));
        }
        let preferred = if use_provider {
            self.provider
                .as_mut()
                .map(|p| p as &mut dyn ContextPressureProvider)
        } else {
            None
        };
        let viscosity = match self.frozen.future.viscosity {
            ViscosityIntent::Disabled => None,
            ViscosityIntent::NewtonianConstant {
                dynamic_viscosity_pa_s,
            } => self.viscosity.as_mut().map(|w| (w, dynamic_viscosity_pa_s)),
        };
        let result = self
            .session
            .step_with_configured_pressure(
                LiquidStepInputs {
                    requested_dt: self.frozen.future.requested_dt_s,
                    smoke_source: None,
                    forces: &self.forces,
                    inlet: self.inlet,
                    source: None,
                    volume: self.frozen.future.volume.settings(),
                },
                &mut self.pressure,
                viscosity,
                preferred,
                deadline,
                |_| self.cancelled.load(Ordering::Relaxed),
            )
            .map(|(report, _)| report);
        self.runtime_observation = self
            .provider
            .as_ref()
            .and_then(AcceptedCpuProvider::last_observation)
            .cloned();
        self.failure = self
            .provider
            .as_ref()
            .and_then(AcceptedCpuProvider::last_failure)
            .map(|f| ConfiguredFailure {
                reason: f.reason,
                detail: f.detail.clone(),
                dispatched: f.dispatched,
                invalid_payload_sha256: f.invalid_payload_sha256,
            });
        if self.failure.as_ref().is_some_and(|f| {
            matches!(
                f.reason,
                NoCandidateReason::RuntimeFailure
                    | NoCandidateReason::InvalidOutput
                    | NoCandidateReason::Cancelled
            )
        }) {
            self.retire_runtime();
        }
        result
    }
    /// Drop through the existing owner, joining I/O and reaping its child.
    /// This preserves the latest observation and every accepted Session field.
    pub(crate) fn retire_runtime(&mut self) {
        drop(self.provider.take());
    }
    pub(crate) fn current_failure(&self) -> Option<&ConfiguredFailure> {
        self.failure.as_ref()
    }
    pub(crate) fn runtime_observation(
        &self,
    ) -> Option<&rheon::recording::provider::runtime::RuntimeObservation> {
        self.runtime_observation.as_ref()
    }
    #[cfg(test)]
    pub(crate) fn runtime_ready(&self) -> bool {
        self.provider.is_some()
    }
    pub(crate) fn snapshot(&self) -> SessionSnapshot<'_> {
        self.session.snapshot()
    }
    pub(crate) fn frozen(&self) -> &FrozenConfiguredSpec {
        &self.frozen
    }
    pub(crate) fn geometry(&self) -> &GridGeometry {
        self.session.snapshot().grid()
    }
    pub(crate) fn native_implementation(&self) -> PressureImplementation {
        self.frozen.constructor.pressure_method.implementation()
    }
    #[cfg(test)]
    pub(crate) fn inlet(&self) -> LiquidInlet {
        self.inlet
    }
    pub(crate) fn host_array_bytes(&self) -> HostArrayBytes {
        HostArrayBytes {
            session: self.session.allocated_array_bytes(),
            pressure_workspace: self.pressure.allocated_bytes(),
            viscosity_workspace: self
                .viscosity
                .as_ref()
                .map_or(0, ViscosityWorkspace::allocated_bytes),
            cpu_adapter_scratch: self
                .provider
                .as_ref()
                .map_or(0, AcceptedCpuProvider::scratch_bytes),
        }
    }
}

#[cfg(test)]
#[path = "configured_track_tests.rs"]
mod tests;
