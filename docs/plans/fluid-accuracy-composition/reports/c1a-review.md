# C1-A independent implementation review

Reviewed source: `ac6ed17003279e7078751a4390f2ee28200d4d5f`, including the final annotation and test-only refinements over `5c9fb96`. Reviewers did not author implementation or run duplicate builds/tests. The coordinating owner retains deciding logs under `/workspace/rheon-c1a-evidence`.

## Architecture — PASS

Independent architecture reviewer confirmed pressure-v1 delegates the existing dispatcher and returns its original report with a borrowed immutable pressure slice tied to the stage borrow. Unavailable replacement scopes do not touch workspace. NativeSession stores exclusive owner references only; family, caller-declared run/source identity and generation checks precede delegation, native callbacks and workspace use. Existing owner methods retain deadline, publication, fields and precision. Snapshots borrow original arrays, with no second clock, scheduler or history.

The focused differential tests substantiate 100 native MAC intervals, particle preview with nonempty/shortened intervals, adversarial pressure candidates/fallback on both implementations, and accepted-state preservation across native refusals/failures. Final test refinements strengthen the exact first cancellation checkpoint/error. A local `large_enum_variant` allowance retains by-value native reports and avoids adding heap allocation after the owner's publication; it establishes no performance claim. No substantive architecture finding or remaining blocker.

## Contracts — PASS

Independent contracts reviewer confirmed unchanged pressure-v1 delegation, protected result borrows and legacy path-specific validation/fallback behavior, including exact classical candidate acceptance with zero native iteration allowance. Metadata remains descriptive; no native-generation or hidden-state rollback authority is added to pressure v1. Native source/generation/family guards precede invocation and preserve native accepted history. Unavailable subgraph/whole-family and transfer paths have no provider or callback route.

Reviewer inspected coordinated evidence: 12 focused tests pass in each feature configuration, 33 existing contract tests pass, and two borrow compile-fail doctests pass. Three real-model tests remain ignored because their pinned artifact is absent; those are exclusions, not passing learned-model evidence. Final patch disposition at ac6ed17: PASS, no material source/test hole, no reviewer writes/builds or remaining blocker.

## Scope retained

These reviews accept only the bounded C1-A adapter/facade architecture and contracts. They do not accept C1-R, projection/whole-state substitution, transfers, arbitrary provider rollback, particle method selection, learned deployment, physical accuracy, GPU behavior, new native GUI behavior or performance. Existing G1 evidence remains separate. Hosted CI, dependency integration and merges belong to the repository lead.
