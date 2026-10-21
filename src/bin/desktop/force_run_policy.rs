//! Desktop force eligibility only. No execution, persistence or field evaluation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    NativeCarrier,
    ParticleMaterial,
    Unsupported,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Qualification {
    Pending,
    Validated,
}
// Remains closed until the exact native acceptance is returned and reviewed.
pub const PRODUCTION_QUALIFICATION: Qualification = Qualification::Pending;
pub const PENDING: &str =
    "Authored forces cannot run yet: native interaction acceptance is pending";
pub const UNSUPPORTED: &str = "Authored forces are unsupported for this workflow (configured/session/provider/free-surface/AI)";

/// Presence means authored metadata, including an empty, disabled or zero scene.
/// Qualification cannot grant capability to an unsupported execution owner.
pub fn check(
    target: Target,
    authored: bool,
    qualification: Qualification,
) -> Result<(), &'static str> {
    if !authored {
        return Ok(());
    }
    if target == Target::Unsupported {
        return Err(UNSUPPORTED);
    }
    if qualification != Qualification::Validated {
        return Err(PENDING);
    }
    Ok(())
}
