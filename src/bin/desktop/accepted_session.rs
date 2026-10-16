//! Desktop accepted execution composes the existing comparison process owner.
//! Keep this graph in the executable that consumes it; the standalone lab
//! retains its existing diagnostic lifecycle and test-only Driver.
#[cfg(test)]
use crate::ai_comparison::cpu_contract_tests;
pub(crate) use crate::ai_comparison::model_process;
use crate::ai_comparison::{adapter, engine};

#[path = "ai_comparison/accepted_provider.rs"]
mod accepted_provider;
#[path = "ai_comparison/configured_track.rs"]
pub(crate) mod configured_track;

/// Runtime/terminal text has the codec's 1,000-byte budget. Preserve exact
/// dispatch/digest facts separately and visibly mark any human-detail truncation.
pub(crate) fn recording_detail(value: &str) -> String {
    const LIMIT: usize = 1000;
    if value.len() <= LIMIT {
        return value.to_owned();
    }
    let mut end = LIMIT - 16;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    format!("{} [truncated]", &value[..end])
}
