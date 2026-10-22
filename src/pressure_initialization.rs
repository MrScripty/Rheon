//! Accepted pressure provenance. No numerical arrays or solver scratch live here.
use crate::{GridGeometry, PressureImplementation};
use std::sync::Mutex;

/// Numerical initialization is independent of pressure-provider selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PressureInitializationPolicy {
    #[default]
    Cold,
    /// Previous complete accepted native sealed-box liquid interval, in Pa.
    AcceptedPreviousV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressureSeedStage {
    AcceptedLiquidStep,
}

/// Process-local provenance, not a portable replay identifier. Numerical replay
/// requires the same accepted history and inputs; dense3D does not store it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PressureSeedStamp {
    pub owner_incarnation: u64,
    pub accepted_generation: u64,
    pub operator_revision: u64,
    pub stage: PressureSeedStage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressureColdReason {
    PolicyCold,
    Constructor,
    UnsupportedPath,
    Invalidated,
}

/// Initialization of an accepted complete step, separate from provider identity.
/// Failed attempts never replace the owner's last accepted report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PressureInitializationReport {
    pub requested: PressureInitializationPolicy,
    pub actual: PressureInitializationPolicy,
    /// The accepted predecessor actually borrowed, not the new publication.
    pub seed: Option<PressureSeedStamp>,
    pub cold_reason: Option<PressureColdReason>,
}

/// Exact key for the immutable constant-density sealed operator and zero gauge.
/// Unsupported domains never receive a key. RHS, forces and dt are not operator
/// coefficients; their changes must not rescale the pressure guess.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SealedOperatorKey {
    counts: [usize; 3],
    spacing: [u64; 3],
    origin: [u64; 3],
    density: u64,
    implementation: PressureImplementation,
}
impl SealedOperatorKey {
    pub(crate) fn new(
        grid: &GridGeometry,
        density: f64,
        implementation: PressureImplementation,
    ) -> Self {
        Self {
            counts: grid.counts(),
            spacing: grid.spacing().map(f64::to_bits),
            origin: grid.origin().map(f64::to_bits),
            density: density.to_bits(),
            implementation,
        }
    }
}

// Identity allocation only, once per constructor. Never used by numerical work
// or per-step publication. Checked nonwrapping exhaustion refuses construction.
static NEXT_INCARNATION: Mutex<u64> = Mutex::new(0);

#[derive(Clone, Copy)]
pub(crate) struct AcceptedPressureHistory {
    incarnation: u64,
    revision: u64,
    key: Option<SealedOperatorKey>,
    seed: Option<PressureSeedStamp>,
    published: bool,
}
impl AcceptedPressureHistory {
    pub(crate) fn new() -> Option<Self> {
        // No user code or fallible operation can panic while this lock is held.
        // A poisoned identity counter is still valid; do not reset/reuse IDs.
        let mut counter = NEXT_INCARNATION.lock().unwrap_or_else(|e| e.into_inner());
        *counter = counter.checked_add(1)?;
        Some(Self {
            incarnation: *counter,
            revision: 0,
            key: None,
            seed: None,
            published: false,
        })
    }

    pub(crate) fn seed(&self) -> Option<PressureSeedStamp> {
        self.seed
    }

    pub(crate) fn initialization(
        &self,
        requested: PressureInitializationPolicy,
        key: Option<SealedOperatorKey>,
        generation: u64,
    ) -> PressureInitializationReport {
        let reason = if requested == PressureInitializationPolicy::Cold {
            Some(PressureColdReason::PolicyCold)
        } else if key.is_none() {
            Some(PressureColdReason::UnsupportedPath)
        } else if !self.published {
            Some(PressureColdReason::Constructor)
        } else if key != self.key
            || self
                .seed
                .is_none_or(|s| s.accepted_generation != generation)
        {
            Some(PressureColdReason::Invalidated)
        } else {
            None
        };
        PressureInitializationReport {
            requested,
            actual: if reason.is_none() {
                PressureInitializationPolicy::AcceptedPreviousV1
            } else {
                PressureInitializationPolicy::Cold
            },
            seed: if reason.is_none() { self.seed } else { None },
            cold_reason: reason,
        }
    }

    /// Prepare metadata without mutating accepted history. The caller checks
    /// exhaustion before its last callback, then assigns this at infallible commit.
    pub(crate) fn publication(
        self,
        key: Option<SealedOperatorKey>,
        generation: u64,
    ) -> Option<Self> {
        let revision = if key != self.key {
            self.revision.checked_add(1)?
        } else {
            self.revision
        };
        Some(Self {
            incarnation: self.incarnation,
            revision,
            key,
            seed: key.map(|_| PressureSeedStamp {
                owner_incarnation: self.incarnation,
                accepted_generation: generation,
                operator_revision: revision,
                stage: PressureSeedStage::AcceptedLiquidStep,
            }),
            published: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn operator_identity_and_generation_are_required_and_publication_is_staged() {
        let grid = GridGeometry::new([3, 2, 1], [1.0; 3], [0.0; 3]).unwrap();
        let key = SealedOperatorKey::new(&grid, 1.0, PressureImplementation::JacobiPcgV1);
        let empty = AcceptedPressureHistory::new().unwrap();
        let accepted = empty.publication(Some(key), 7).unwrap();
        assert!(empty.seed().is_none());
        assert!(
            accepted
                .initialization(
                    PressureInitializationPolicy::AcceptedPreviousV1,
                    Some(key),
                    7
                )
                .seed
                .is_some()
        );
        assert!(
            accepted
                .initialization(
                    PressureInitializationPolicy::AcceptedPreviousV1,
                    Some(key),
                    8
                )
                .seed
                .is_none()
        );
        for changed in [
            SealedOperatorKey::new(
                &GridGeometry::new([2, 3, 1], [1.0; 3], [0.0; 3]).unwrap(),
                1.0,
                PressureImplementation::JacobiPcgV1,
            ),
            SealedOperatorKey::new(
                &GridGeometry::new([3, 2, 1], [2.0, 1.0, 1.0], [0.0; 3]).unwrap(),
                1.0,
                PressureImplementation::JacobiPcgV1,
            ),
            SealedOperatorKey::new(
                &GridGeometry::new([3, 2, 1], [1.0; 3], [1.0, 0.0, 0.0]).unwrap(),
                1.0,
                PressureImplementation::JacobiPcgV1,
            ),
            SealedOperatorKey::new(&grid, 2.0, PressureImplementation::JacobiPcgV1),
            SealedOperatorKey::new(
                &grid,
                1.0,
                PressureImplementation::SymmetricGaussSeidelPcgV1,
            ),
        ] {
            assert_eq!(
                accepted
                    .initialization(
                        PressureInitializationPolicy::AcceptedPreviousV1,
                        Some(changed),
                        7
                    )
                    .cold_reason,
                Some(PressureColdReason::Invalidated)
            );
            let next = accepted.publication(Some(changed), 8).unwrap();
            assert_eq!(next.seed().unwrap().operator_revision, 2);
        }
        let invalidated = accepted.publication(None, 8).unwrap();
        assert!(invalidated.seed().is_none());
        assert_eq!(accepted.seed().unwrap().accepted_generation, 7);
        let exhausted = AcceptedPressureHistory {
            revision: u64::MAX,
            ..accepted
        };
        assert!(exhausted.publication(None, 8).is_none());
        assert!(exhausted.publication(Some(key), 8).is_some());
    }
}
