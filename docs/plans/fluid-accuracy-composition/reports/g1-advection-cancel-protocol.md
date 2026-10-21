# G1 advection cancellation receipt repair — frozen protocol

Admitted 2026-10-20 by the delegated implementation owner following the parent's explicit repair direction. Base PR136 e56bc7bd31ea8e8121f09cba15eb8a2bc5e514e4; fetched main remains 718469beedb9a949f4bb6f3299c7b880b4c47e47. Preserve existing checkpoints, negative evidence and independently owned numerical work. Lead owns integration, canonical status and source admission.

## Boundary

Fix only the configured Session's classification of `Step(Carrier(Advection(Cancelled)))` as `AttemptOutcome::Cancelled`. Retain the typed error and existing no-publication behavior. Do not alter numerical formulas, ordering, domains, pressure v1, serialized versions or the meaning of historical receipts. Write set: `src/session/configured.rs` (one classification arm), adjacent actual-receipt tests in `src/recording/workflow.rs`, focused native consumer tests in `src/bin/desktop/workflow_inspector_tests.rs`, and repair-prefixed protocol/qualification/review/manifest/readiness reports here. No product UI changes. Any other production owner requires a new bounded decision.

## Fixed deciding evidence

1. Retain tests-only red evidence for both velocity- and tracer-advection cancellation at the actual configured Session checkpoints. Exercise generation zero and a prior accepted generation. Assert the exact nested cancellation error, unchanged accepted state/generation, no publication, correct partial trace, and Cancelled live receipt.
2. Persist real receipts in RCT2 through the existing recorder, drop owners and cold reopen: no invented accepted row; accepted tail and attempt linkage unchanged. Historical explicitly Failed receipts retain Failed; old RCP2/RCP3/RCT1 compatibility remains passing. Include an actual non-cancellation numerical failure control that remains Failed and unpublished.
3. Exercise the native egui receipt consumer for actual live and cold-archive cancellation, retaining existing pointer/keyboard/focus/close-reopen coverage. These headless native tests do not claim a new native-window run.
4. Run focused core workflow/recording and native inspector suites, affected integration/compatibility checks, format and strict all-target Clippy at exact source. Record failures without retuning criteria. Independent architecture/persistence and native consumer review must dispose findings before qualification.

Publish a focused draft dependent PR. Local results do not satisfy hosted or complete new source-admission gates; relay changed hashes to lead. After qualification, assess the next dependency-ready composition slice using existing C1 contracts without repeating their design or implementing another scope implicitly.
