//! Process-local identity shared by desktop consumers; not global provenance.
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT_RUN: AtomicU64 = AtomicU64::new(1);
pub fn new_identity(backend: &str) -> rheon::SessionIdentity {
    rheon::SessionIdentity {
        run_id: NEXT_RUN.fetch_add(1, Ordering::Relaxed),
        source_revision: super::build_identity::executor_revision(&format!("desktop-{backend}")),
    }
}
