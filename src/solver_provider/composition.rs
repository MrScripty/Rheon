//! Closed replacement selection over the existing pressure-v1 dispatcher.
//!
//! Only the pressure stage is callable here. Native subgraph and whole-family
//! replacements require separate request and qualification contracts.
use super::{PressureProvider, PressureRequest, ProviderReport, ProviderWorkspace};
use crate::{PressureError, PressureSettings};
use std::fmt;

/// Finite replacement scopes. Naming a scope does not make it available.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplacementContract {
    PressureV1,
    SealedMacProjectionV1,
    WholeSealedMacV1,
    WholeParticlePreviewV1,
}

/// The selected scope has no callable replacement contract in C1-A.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplacementUnavailable {
    pub requested: ReplacementContract,
}
impl fmt::Display for ReplacementUnavailable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "replacement unavailable: {:?}", self.requested)
    }
}
impl std::error::Error for ReplacementUnavailable {}

/// Borrowed entry point to the existing sealed-MAC pressure-v1 dispatcher.
/// Selection and construction do not run a provider or modify its workspace.
pub struct PressureStageV1<'workspace> {
    workspace: &'workspace mut ProviderWorkspace,
}
impl<'workspace> PressureStageV1<'workspace> {
    pub fn select(
        contract: ReplacementContract,
        workspace: &'workspace mut ProviderWorkspace,
    ) -> Result<Self, ReplacementUnavailable> {
        match contract {
            ReplacementContract::PressureV1 => Ok(Self { workspace }),
            requested => Err(ReplacementUnavailable { requested }),
        }
    }

    /// Preserve the original dispatcher's input, cancellation, provider and
    /// fallback semantics. Metadata remains descriptive under pressure v1.
    pub fn solve(
        &mut self,
        request: PressureRequest<'_>,
        settings: PressureSettings,
        preferred: Option<&mut dyn PressureProvider>,
        cancelled: impl FnMut() -> bool,
    ) -> Result<QualifiedPressureV1<'_>, PressureError> {
        let report = self
            .workspace
            .solve(request, settings, preferred, cancelled)?;
        Ok(QualifiedPressureV1 {
            pressure: self.workspace.pressure(),
            report,
        })
    }
}

/// Host-qualified pressure with its original report. This is not a corrected
/// velocity or accepted Session state. The borrow prevents overwriting pressure
/// while the qualified view is used. Failed calls return no qualified view;
/// the original workspace's failure semantics still apply.
///
/// ```compile_fail,E0499
/// use rheon::solver_provider::{PressureRequest, composition::PressureStageV1};
/// use rheon::PressureSettings;
/// fn cannot_overwrite(stage: &mut PressureStageV1<'_>, request: PressureRequest<'_>,
///                     settings: PressureSettings) {
///     let accepted = stage.solve(request, settings, None, || false).unwrap();
///     let _next = stage.solve(request, settings, None, || false);
///     println!("{:?}", accepted.pressure());
/// }
/// ```
pub struct QualifiedPressureV1<'workspace> {
    pressure: &'workspace [f64],
    report: ProviderReport,
}
impl QualifiedPressureV1<'_> {
    pub fn pressure(&self) -> &[f64] {
        self.pressure
    }
    pub fn report(&self) -> &ProviderReport {
        &self.report
    }
    /// Release the pressure borrow and retain the dispatcher report.
    pub fn into_report(self) -> ProviderReport {
        self.report
    }
}

#[cfg(test)]
#[path = "composition_tests.rs"]
mod tests;
