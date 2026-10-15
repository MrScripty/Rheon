//! Accepted pressure contexts over the existing host-qualified provider gate.
//! Packages own preparation and weights; the Session owns physical composition,
//! interval/history lineage and publication. This is a stateless candidate seam.
use super::{
    PressureProvider, PressureRequest, ProviderRejection, ProviderReport, SolverMetadata,
    TensorMetadata,
    context::{
        PressureContext, PressureContextInput, PressureContextMode, PressureHistory,
        PressureHistoryStatus, STAGE_CONTRACT,
    },
};
use sha2::{Digest, Sha256};
use std::{cell::Cell, sync::Arc};

/// Owned package custody declared by the runtime owner. The concrete runtime
/// must verify these pins before execution; metadata alone proves no authenticity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextProviderIdentity {
    pub id: String,
    pub version: String,
    pub manifest_sha256: [u8; 32],
    pub checkpoint_sha256: Option<[u8; 32]>,
    pub training_run_id: Option<String>,
    pub runtime: String,
    pub source_revision: String,
}
impl ContextProviderIdentity {
    pub fn validate(&self) -> Result<(), ProviderRejection> {
        let valid = |text: &str| {
            !text.is_empty() && text.len() <= 1000 && !text.chars().any(char::is_control)
        };
        if ![
            &self.id,
            &self.version,
            &self.runtime,
            &self.source_revision,
        ]
        .into_iter()
        .all(|s| valid(s))
            || self.training_run_id.as_deref().is_some_and(|s| !valid(s))
        {
            return Err(ProviderRejection::Unsupported(
                "invalid bounded configured package identity",
            ));
        }
        Ok(())
    }
}

pub struct ContextPressureRequest<'a> {
    pub context: PressureContext<'a>,
    /// Actual host Newtonian stage coefficient, Pa s; zero is inviscid.
    pub dynamic_viscosity: f64,
    /// Consecutive initial native accepted pressure intervals for this composition.
    pub native_prefix_steps: usize,
}

/// Implementations receive only authoritative accepted host history. They must
/// be stateless across calls, bound their resources, and honor cancellation.
/// New hidden-state or physical semantics require a separate contract.
pub trait ContextPressureProvider {
    fn identity(&self) -> Arc<ContextProviderIdentity>;
    fn supports(&self, request: &ContextPressureRequest<'_>) -> Result<(), ProviderRejection>;
    fn predict(
        &mut self,
        request: &ContextPressureRequest<'_>,
        pressure: &mut [f64],
        cancelled: &mut dyn FnMut() -> bool,
    ) -> Result<(), ProviderRejection>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcceptedPressureContextStamp {
    pub generation: u64,
    pub interval_start: f64,
    pub interval_end: f64,
    pub dt: f64,
    pub native_prefix_steps: usize,
    pub prior_generation: Option<u64>,
    pub prior_interval_end: Option<f64>,
    pub prior_pressure_sha256: Option<[u8; 32]>,
    pub rhs_sha256: [u8; 32],
}
/// Only returned/published with the complete accepted transport interval.
#[derive(Debug, Clone)]
pub struct ConfiguredProviderReport {
    pub binding: Option<Arc<ContextProviderIdentity>>,
    pub context: Option<AcceptedPressureContextStamp>,
    pub dynamic_viscosity: f64,
    /// The host called the provider's prediction method. This does not attest
    /// that an external runtime accepted or consumed a request.
    pub prediction_called: bool,
    pub pressure: ProviderReport,
}

#[derive(Clone, Copy)]
pub(crate) struct AcceptedHistoryStamp {
    pub generation: u64,
    pub interval_end: f64,
    pub dynamic_viscosity: f64,
    pub native_prefix_steps: usize,
    pub all_native: bool,
}
pub(crate) struct AcceptedContextInput<'a> {
    pub run_id: String,
    pub generation: u64,
    pub interval_start: f64,
    pub history: Option<AcceptedHistoryStamp>,
    pub dynamic_viscosity: f64,
    pub provider: Option<&'a mut dyn ContextPressureProvider>,
}
pub(crate) struct ContextAdapter<'a, 'h> {
    input: AcceptedContextInput<'a>,
    prior: &'h [f64],
    counts: [usize; 3],
    pub binding: Option<Arc<ContextProviderIdentity>>,
    pub stamp: Cell<Option<AcceptedPressureContextStamp>>,
    pub prediction_called: bool,
}
fn hash_values(values: &[f64]) -> [u8; 32] {
    let mut hash = Sha256::new();
    for value in values {
        hash.update(value.to_bits().to_le_bytes());
    }
    hash.finalize().into()
}
impl<'a, 'h> ContextAdapter<'a, 'h> {
    pub fn has_provider(&self) -> bool {
        self.input.provider.is_some()
    }
    pub fn new(input: AcceptedContextInput<'a>, prior: &'h [f64], counts: [usize; 3]) -> Self {
        let binding = input
            .provider
            .as_ref()
            .map(|provider| provider.identity())
            .filter(|identity| identity.validate().is_ok());
        Self {
            input,
            prior,
            counts,
            binding,
            stamp: Cell::new(None),
            prediction_called: false,
        }
    }
    fn request<'b>(
        &'b self,
        request: &'b PressureRequest<'b>,
    ) -> Result<ContextPressureRequest<'b>, ProviderRejection> {
        let history = self.input.history.filter(|history| {
            history.dynamic_viscosity.to_bits() == self.input.dynamic_viscosity.to_bits()
        });
        let prior = history.map(|history| PressureHistory {
            values: self.prior,
            grid: request.grid,
            density: request.density,
            run_id: &self.input.run_id,
            generation: history.generation,
            interval_end: history.interval_end,
            status: PressureHistoryStatus::Accepted,
        });
        let context = PressureContext::new(PressureContextInput {
            grid: request.grid,
            density: request.density,
            dt: request.dt,
            interval_start: self.input.interval_start,
            rhs: request.rhs,
            prior,
            run_id: &self.input.run_id,
            generation: self.input.generation,
            mode: PressureContextMode::AcceptedCandidate,
        })
        .map_err(|_| {
            ProviderRejection::Unsupported(
                "accepted pressure history unavailable for this composition",
            )
        })?;
        Ok(ContextPressureRequest {
            context,
            dynamic_viscosity: self.input.dynamic_viscosity,
            native_prefix_steps: history.map_or(0, |history| history.native_prefix_steps),
        })
    }
}
impl PressureProvider for ContextAdapter<'_, '_> {
    fn metadata(&self) -> SolverMetadata {
        // Stable role metadata is deliberately distinct from the actual owned
        // package identity carried by ConfiguredProviderReport.binding.
        SolverMetadata {
            id: "configured-pressure-context-v1",
            version: "1",
            stage: STAGE_CONTRACT,
            input: TensorMetadata {
                shape_xyz: self.counts,
                layout: "x_fast",
                scalar: "f64",
                units: "m3/s2",
            },
            output: TensorMetadata {
                shape_xyz: self.counts,
                layout: "x_fast",
                scalar: "f64",
                units: "Pa",
            },
            domain: "host-composed sealed constant-density MAC; package pins in binding receipt",
            provenance: None,
        }
    }
    fn supports(&self, request: &PressureRequest<'_>) -> Result<(), ProviderRejection> {
        self.binding
            .as_ref()
            .ok_or(ProviderRejection::Unsupported(
                "invalid configured provider identity",
            ))?
            .validate()?;
        let context = self.request(request)?;
        self.stamp.set(Some(AcceptedPressureContextStamp {
            generation: context.context.generation(),
            interval_start: context.context.interval_start(),
            interval_end: context.context.interval_end(),
            dt: context.context.dt(),
            native_prefix_steps: context.native_prefix_steps,
            prior_generation: context.context.prior().map(|p| p.generation),
            prior_interval_end: context.context.prior().map(|p| p.interval_end),
            prior_pressure_sha256: context.context.prior().map(|p| hash_values(p.values)),
            rhs_sha256: hash_values(context.context.rhs()),
        }));
        self.input
            .provider
            .as_ref()
            .ok_or(ProviderRejection::Unsupported("no configured provider"))?
            .supports(&context)
    }
    fn predict(
        &mut self,
        request: &PressureRequest<'_>,
        pressure: &mut [f64],
        cancelled: &mut dyn FnMut() -> bool,
    ) -> Result<(), ProviderRejection> {
        self.prediction_called = true;
        let provider = self
            .input
            .provider
            .take()
            .ok_or(ProviderRejection::Unsupported("no configured provider"))?;
        let result = self
            .request(request)
            .and_then(|context| provider.predict(&context, pressure, cancelled));
        self.input.provider = Some(provider);
        result
    }
}
