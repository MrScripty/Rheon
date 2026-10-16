//! Exact configured-selection facts beside the shared native field comparison.
//! No model, Session, path resolver, field store or alternate reducer is owned
//! here. A selected publication cannot establish missing history or terminal
//! coverage; whole-archive verification and runtime termination audit are separate.
use super::{
    PROFILE, RecordedConfiguredReceipt, RecordedConfiguredSession, RecordedPressureSelection,
    runtime::RuntimeObservation,
};
use crate::recording::{
    RecordingError,
    comparison::{Selection, SelectionIdentity, Unavailable, validate_profile},
};
use std::time::Duration;

/// Maximum sampled publications retained by one operational summary.
/// Reader limits bound each record; this is not a peak-memory or I/O guarantee.
pub const MAX_SELECTIONS: usize = 128;

/// Actual owner-supplied accepted projection interval, in seconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PressureInterval {
    pub start: f64,
    pub end: f64,
    pub dt: f64,
}

/// Typed facts for one exact configured publication. Constructors have neither
/// interval nor receipt. Runtime absence means unavailable, not no dispatch.
/// Rejected candidate diagnostics retain their original IEEE bits in `receipt`.
#[derive(Debug, Clone)]
pub struct ConfiguredSelectionFacts {
    pub identity: SelectionIdentity,
    pub interval: Option<PressureInterval>,
    pub receipt: Option<RecordedConfiguredReceipt>,
    pub runtime: Option<RuntimeObservation>,
}

/// Validate the full configured row, including RCP3 association, and retain only
/// metadata. This synchronous extraction temporarily decodes bounded native
/// arrays; callers own their aggregate selections and memory budget.
pub fn inspect(selection: &Selection) -> Result<ConfiguredSelectionFacts, RecordingError> {
    let observed = RecordedConfiguredSession::decode(selection.run(), selection.publication())?;
    let interval = match observed.interval_pressure {
        Some((start, _)) => Some(PressureInterval {
            start,
            end: observed.time,
            dt: observed
                .dt
                .ok_or(RecordingError::Invalid("configured comparison interval"))?,
        }),
        None => None,
    };
    Ok(ConfiguredSelectionFacts {
        identity: selection.identity().clone(),
        interval,
        receipt: observed.receipt,
        runtime: observed.runtime,
    })
}

/// Relation of retained authoritative RHS payload hashes only. Equal hashes do
/// not establish an equal operator, request, trajectory or authenticated origin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RhsRelation {
    EqualHashes,
    DifferentHashes,
    Unavailable,
}

/// Native carrier-v1 records have no authoritative RHS stamp. Configured rows
/// are fully typed-decoded even if the other side lacks a stamp. This does not
/// grant physical alignment; use the shared `recording::comparison` functions.
pub fn rhs_relation(
    reference: &Selection,
    candidate: &Selection,
) -> Result<RhsRelation, RecordingError> {
    let reference = rhs_hash(reference)?;
    let candidate = rhs_hash(candidate)?;
    Ok(match (reference, candidate) {
        (Some(a), Some(b)) if a == b => RhsRelation::EqualHashes,
        (Some(_), Some(_)) => RhsRelation::DifferentHashes,
        _ => RhsRelation::Unavailable,
    })
}

fn rhs_hash(selection: &Selection) -> Result<Option<[u8; 32]>, RecordingError> {
    if selection.run().solver == PROFILE {
        return Ok(inspect(selection)?
            .receipt
            .and_then(|receipt| receipt.context.map(|context| context.rhs_sha256)));
    }
    validate_profile(selection).map_err(|reason| match reason {
        Unavailable::UnsupportedProfile => {
            RecordingError::Unsupported("configured comparison source profile")
        }
        _ => RecordingError::Invalid("configured comparison source schema"),
    })?;
    Ok(None)
}

/// Existing host pressure-call scopes only. These are not full-step wall time,
/// isolated model compute, startup/warmup, throughput or a speedup measurement.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PressureDurations {
    pub total: Duration,
    pub inference: Duration,
    pub qualification: Duration,
    pub fallback: Duration,
}
impl PressureDurations {
    fn add(&mut self, receipt: &RecordedConfiguredReceipt) -> Result<(), RecordingError> {
        let checked = |a: Duration, b| {
            a.checked_add(b)
                .ok_or(RecordingError::Limit("configured comparison duration sum"))
        };
        self.total = checked(self.total, receipt.total)?;
        self.inference = checked(self.inference, receipt.inference)?;
        self.qualification = checked(self.qualification, receipt.qualification)?;
        self.fallback = checked(self.fallback, receipt.fallback)?;
        Ok(())
    }
}

/// Facts about exactly the selected subset, not an attempted horizon or complete
/// run. Gaps are allowed. Core host rejection and runtime outcomes remain separate.
#[derive(Debug, Clone, Default)]
pub struct ConfiguredAggregate {
    pub selections: Vec<SelectionIdentity>,
    pub constructor_count: usize,
    pub accepted_count: usize,
    pub attempted_count: usize,
    pub prediction_called_count: usize,
    pub configured_selected_count: usize,
    pub native_selected_count: usize,
    pub rejected_count: usize,
    pub runtime_observed_count: usize,
    pub durations: PressureDurations,
}

/// Sum existing pressure-call durations with checked arithmetic. All selections
/// must have identical run digests and strictly increasing ordinals; declaration
/// IDs alone cannot bind constructor/profile metadata. No field arrays are kept.
pub fn summarize(selections: &[Selection]) -> Result<ConfiguredAggregate, RecordingError> {
    if selections.len() > MAX_SELECTIONS {
        return Err(RecordingError::Limit(
            "configured comparison selection count",
        ));
    }
    let mut summary = ConfiguredAggregate::default();
    let mut previous_ordinal = None;
    let run = selections.first().map(Selection::run_digest);
    for selection in selections {
        if Some(selection.run_digest()) != run
            || previous_ordinal.is_some_and(|ordinal| selection.identity().ordinal <= ordinal)
        {
            return Err(RecordingError::Invalid(
                "configured comparison selection order/run",
            ));
        }
        let facts = inspect(selection)?;
        previous_ordinal = Some(facts.identity.ordinal);
        summary.selections.push(facts.identity);
        summary.runtime_observed_count += usize::from(facts.runtime.is_some());
        if let Some(receipt) = facts.receipt {
            summary.accepted_count += 1;
            summary.attempted_count += usize::from(receipt.attempted.is_some());
            summary.prediction_called_count += usize::from(receipt.prediction_called);
            summary.configured_selected_count +=
                usize::from(receipt.selected == RecordedPressureSelection::Configured);
            summary.native_selected_count += usize::from(matches!(
                receipt.selected,
                RecordedPressureSelection::Native(_)
            ));
            summary.rejected_count += usize::from(receipt.rejection.is_some());
            summary.durations.add(&receipt)?;
        } else {
            summary.constructor_count += 1;
        }
    }
    Ok(summary)
}
