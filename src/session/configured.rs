//! Accepted configurable pressure publication through the existing Session owner.
use super::*;
use crate::solver_provider::{
    ProviderReport,
    configured::{
        AcceptedContextInput, AcceptedHistoryStamp, ConfiguredProviderReport,
        ContextPressureProvider,
    },
};
impl<'a> SessionSnapshot<'a> {
    /// Actual package custody and host request lineage for an accepted configured
    /// call. None after constructor or a successful legacy/boundary call.
    pub fn configured_provider_report(&self) -> Option<&'a ConfiguredProviderReport> {
        self.configured
    }
}

impl SimulationSession {
    /// Compose the existing sealed-box Newtonian stage (when supplied) with
    /// host-qualified pressure selection, followed by every existing transport
    /// gate and the same atomic publication point. No adapter owns history.
    /// Workspaces remain separately capped; no additional history array is kept.
    pub fn step_with_configured_pressure<F: FnMut(LiquidStepStage) -> bool>(
        &mut self,
        inputs: LiquidStepInputs<'_>,
        workspace: &mut crate::ProviderStepWorkspace,
        viscosity: Option<(&mut crate::ViscosityWorkspace, f64)>,
        preferred: Option<&mut dyn ContextPressureProvider>,
        deadline: Option<Instant>,
        cancel: F,
    ) -> Result<(LiquidStepReport, ConfiguredProviderReport), SessionError> {
        self.configured_step_with_clock(
            inputs,
            workspace,
            viscosity,
            preferred,
            deadline,
            cancel,
            Instant::now,
        )
    }
    #[allow(clippy::too_many_arguments)] // Explicit workspaces and injectable deadline clock.
    fn configured_step_with_clock<F: FnMut(LiquidStepStage) -> bool, N: FnMut() -> Instant>(
        &mut self,
        inputs: LiquidStepInputs<'_>,
        workspace: &mut crate::ProviderStepWorkspace,
        viscosity: Option<(&mut crate::ViscosityWorkspace, f64)>,
        preferred: Option<&mut dyn ContextPressureProvider>,
        deadline: Option<Instant>,
        mut cancel: F,
        mut now: N,
    ) -> Result<(LiquidStepReport, ConfiguredProviderReport), SessionError> {
        if self.stopped {
            return Err(SessionError::Stopped);
        }
        if self.steps >= self.max_steps {
            return Err(SessionError::StepLimit);
        }
        if deadline.is_some_and(|d| now() >= d) {
            return Err(SessionError::Deadline);
        }
        let generation = self
            .simulation
            .state()
            .carrier
            .generation
            .checked_add(1)
            .ok_or(SessionError::PressureBinding(
                crate::solver_provider::ProviderRejection::Unsupported(
                    "pressure generation exhausted",
                ),
            ))?;
        let run_id = self.identity.run_id.to_string();
        let configured = AcceptedContextInput {
            run_id,
            generation,
            interval_start: self.simulation.state().carrier.time,
            history: self.pressure_history,
            dynamic_viscosity: viscosity.as_ref().map_or(0.0, |(_, mu)| *mu),
            provider: preferred,
        };
        let mut expired = false;
        let result = self.simulation.step_with_configured_pressure(
            inputs,
            workspace,
            viscosity,
            configured,
            |stage| {
                expired = deadline.is_some_and(|d| now() >= d);
                if expired {
                    return true;
                }
                let cancelled = cancel(stage);
                expired = deadline.is_some_and(|d| now() >= d);
                expired || cancelled
            },
        );
        let (report, provider) = result.map_err(|e| {
            if expired {
                SessionError::Deadline
            } else {
                SessionError::Step(e)
            }
        })?;
        // All fallible/allocating work and callbacks ended before the physics
        // commit. Arc receipt clones are bounded ownership updates, not arrays.
        self.accept_pressure_history(report, &provider.pressure, provider.dynamic_viscosity);
        self.steps += 1;
        self.report = Some(report);
        self.provider = Some(provider.pressure.clone());
        self.configured = Some(provider.clone());
        Ok((report, provider))
    }
    pub(super) fn accept_pressure_history(
        &mut self,
        report: LiquidStepReport,
        provider: &ProviderReport,
        mu: f64,
    ) {
        let previous = self
            .pressure_history
            .filter(|previous| previous.dynamic_viscosity.to_bits() == mu.to_bits());
        let native = provider.attempted.is_none() || provider.rejection.is_some();
        let all_native = native && previous.is_none_or(|previous| previous.all_native);
        let native_prefix_steps = if all_native {
            previous
                .map_or(0, |previous| previous.native_prefix_steps)
                .saturating_add(1)
        } else {
            previous.map_or(0, |previous| previous.native_prefix_steps)
        };
        self.pressure_history = Some(AcceptedHistoryStamp {
            generation: report.carrier.step.generation,
            interval_end: report.carrier.step.time,
            dynamic_viscosity: mu,
            native_prefix_steps,
            all_native,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use std::{cell::Cell, time::Duration};

    #[test]
    fn configured_deadline_crossed_at_final_callback_preserves_history_and_quota() {
        let cap = 1024 * 1024;
        let grid = GridGeometry::new([2; 3], [0.5; 3], [0.0; 3]).unwrap();
        let config = LiquidTransportConfig {
            carrier: SimulationConfig {
                density: 1000.0,
                memory_limit: cap,
                pressure: PressureSettings {
                    relative_residual: 1e-12,
                    absolute_residual: 1e-12,
                    divergence_limit: 1e-9,
                    max_iterations: 1000,
                },
                actual_divergence_limit: 1e-5,
                max_courant: 1.0,
            },
            represented_density: 1000.0,
            volume_stamp: VolumeStamp { id: 1, version: 0 },
            carrier_id: 2,
        };
        let mut workspace =
            ProviderStepWorkspace::new(&grid, 1000.0, cap, PressureImplementation::JacobiPcgV1)
                .unwrap();
        let mut viscosity = ViscosityWorkspace::new(grid.clone(), cap).unwrap();
        let mut session = SimulationSession::new(
            SessionIdentity {
                run_id: 1,
                source_revision: "configured-clock-fixture".into(),
            },
            grid,
            config,
            PressureImplementation::JacobiPcgV1,
            vec![1.0; 8],
            cap,
            2,
        )
        .unwrap();
        let inputs = LiquidStepInputs {
            requested_dt: 0.0625,
            smoke_source: None,
            forces: &[],
            source: None,
            inlet: LiquidInlet::new(VolumeStamp { id: 3, version: 0 }, [[0.0; 2]; 3]).unwrap(),
            volume: LiquidVolumeSettings {
                max_outward_courant: 1.0,
                actual_divergence_limit: 1e-5,
            },
        };
        session
            .step_with_configured_pressure(
                inputs,
                &mut workspace,
                Some((&mut viscosity, 0.001)),
                None,
                None,
                |_| false,
            )
            .unwrap();
        let mut before = Vec::new();
        session.snapshot().write_dense3d(&mut before).unwrap();
        let receipt = session
            .snapshot()
            .configured_provider_report()
            .unwrap()
            .pressure
            .total;
        let history = session.pressure_history.unwrap();
        let start = Instant::now();
        let deadline = start + Duration::from_secs(1);
        let clock = Cell::new(start);
        let reached = Cell::new(false);
        let result = session.configured_step_with_clock(
            inputs,
            &mut workspace,
            Some((&mut viscosity, 0.001)),
            None,
            Some(deadline),
            |stage| {
                if stage == LiquidStepStage::BeforeCommit {
                    reached.set(true);
                    clock.set(deadline);
                }
                false
            },
            || clock.get(),
        );
        assert!(reached.get());
        assert!(matches!(result, Err(SessionError::Deadline)));
        let mut after = Vec::new();
        session.snapshot().write_dense3d(&mut after).unwrap();
        assert_eq!(before, after);
        assert_eq!(
            session
                .snapshot()
                .configured_provider_report()
                .unwrap()
                .pressure
                .total,
            receipt
        );
        let retained = session.pressure_history.unwrap();
        assert_eq!(retained.generation, history.generation);
        assert_eq!(
            retained.interval_end.to_bits(),
            history.interval_end.to_bits()
        );
        assert_eq!(retained.native_prefix_steps, history.native_prefix_steps);
        assert_eq!(retained.all_native, history.all_native);
        assert_eq!(
            retained.dynamic_viscosity.to_bits(),
            history.dynamic_viscosity.to_bits()
        );
        session
            .step_with_configured_pressure(
                inputs,
                &mut workspace,
                Some((&mut viscosity, 0.001)),
                None,
                None,
                |_| false,
            )
            .unwrap();
        assert_eq!(session.snapshot().frame(), 2);
        assert_eq!(session.pressure_history.unwrap().generation, 2);
        assert_eq!(session.pressure_history.unwrap().native_prefix_steps, 2);
        let limited =
            session
                .step_with_configured_pressure(inputs, &mut workspace, None, None, None, |_| false);
        assert!(matches!(limited, Err(SessionError::StepLimit)));
    }
}
