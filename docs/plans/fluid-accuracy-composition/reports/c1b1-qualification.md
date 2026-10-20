# C1-B1 supplied projection qualification

Status: **locally qualified foundation / integration pending**. Exact source f161d1ad9f8a4acfb1e1d86c63c6a875fca3182a. Frozen admission ea37b6040b8fb2c5973170e4379565e47357727d, fixed-gate amendment aa3c435; [protocol](c1b1-protocol.md). [Draft PR128](https://github.com/MrScripty/Rheon/pull/128) targets the preserved PR125 branch at91b2038. Local qualification permits dependent development; it is not hosted-CI, merged, release or full C1-B/C1-R acceptance.

## Delivered boundary

ProjectionRequest immutably binds actual f32 provisional faces, grid, density, actual dt/interval start, declared source/generation, pressure settings, actual divergence limit and exact version/operator/gauge/representation units/precision/layout. Host builds RHS. Unknown support, mismatched source/input fingerprint or representation and invalid identity are typed refusals before callbacks. Source tags prove consistency of supplied data, not authenticity of an accepted Session state.

ProjectionWorkspace qualifies immutable supplied pressure/f32 staggered-velocity arrays with the original pressure dispatcher, production in-place correction witness and actual divergence. Exact witness comparison preserves signed-zero wall bits and rejects wrong dynamics even when divergence and pressure residual are zero. Numerical candidate failures explicitly select compatible native fallback from identical original input; no partially corrected candidate is reused. The two-pressure-call path retains the discarded accepted candidate-pressure report and final native report separately. Cancellation/native failure returns no qualified view. Success borrows disposable capped scratch; it adopts no Session state/history.

Only composition.rs declaration/scope documentation and two new projection source/test files changed production/test source. Numerical/geometry owners, existing physical fixtures, particle, Session, recording, process, UI, Cargo and lockfile remain untouched. C1-A PressureStageV1 still refuses its projection variant. This API executes no candidate provider or external runtime and has no stateful lifecycle; supplied identity is declared provenance only.

## Exact-source results

Rust1.92.0, unchanged locked dependencies, representative native CPU. Fixed criteria/inputs are in the protocol. Every result below uses f161d1a Rust source; no repair/reinterpretation after evaluation. Logs and captured executable hashes are retained under /workspace/rheon-c1b1-evidence, bound by [manifest](c1b1-manifest.json).

| Check | Result | Evidence |
| --- | --- | --- |
| Focused C1-B1 no-default | 11 passed, 0 failed, 0 ignored; .14s | c1b1-no-default.log |
| Focused C1-B1 default | 11 passed, 0 failed, 0 ignored; .06s | c1b1-default.log |
| Seven existing operator/pressure/context/provider/Session targets | 41 passed, 0 failed, 2 real-model tests ignored | existing-contracts.log |
| Preserved C1-A selection/v1 behavior | 1 passed | c1a-compatibility.log |
| Borrow compile-fail Rustdoc | 3 passed, including new projection output borrow | rustdoc.log |
| Strict all-target Clippy, no-default and desktop | PASS / PASS, -D warnings | clippy-core.log / clippy-desktop-initial.log |
| Formatting / diff check | PASS / PASS | fmt-initial.log / coordinating check |
| Independent architecture / contracts review | PASS / PASS at f161d1a | [review dispositions](c1b1-review.md) |

All eleven focused tests cover both native pressure implementations: exact native/supplied fields, 100 independent fixed-input requests per method, wrong divergence-free dynamics, signed-zero wall/gauge behavior, every request binding and representation field, unsupported domains/header identity, field/gauge/pressure rejection, actual-divergence-triggered fallback with both reports, native actual-divergence failure, iteration exhaustion, cancellation around qualification and inside iteration1, retry/immutability and exact aggregate array capacity. The 100 requests do not advance a Session or establish integrated trajectory accuracy. Array equality is asserted by the source-bound native test executables; the logs are not standalone scientific field exports.

Two existing tests remain ignored: real checkpoint replay/corruption and real-model 24-interval Session advance require RHEON_AI_EXPORT from the pinned checkpoint. No learned assets were supplied or synthesized. Platform allocator failure was not injected; checked try_reserve paths and explicit array-cap rejection are the selected evidence. No hard preemption/deadline or arbitrary external-effect rollback is claimed.

## Lifecycle, integration and exact next gate

C1-B1 Active → Implemented at f161d1a → Verifying → locally qualified for the frozen stateless foundation only. No canonical plan, ledger or claim-register edit: serial reconciliation belongs to repository lead01a13f94-5c63-7243-abfc-f5474da1902d. Parent/lead must relay interface notices to numerical owner01a1569b-5cbb-711f-935f-0ccef3cae00c; direct collaboration attempts returned agent not found because those threads are outside this agent tree. This did not require numerical-owner edits or block isolated implementation.

The next C1-B integration slice must first freeze its native invocation/publication write set and additive immutable-intent/attempt/custody versions. It must bind authority to actual accepted Session input, retain observed candidate/fallback facts, execute applicable failure/transaction/runtime tests and the real 100-interval integrated substitution oracle, and receive persistence/native consumer review where those owners change. B1 does not satisfy those gates. Stateful candidates additionally require tested prepare/abort, uncertainty invalidation and reconstruction/replay before reuse; C1-C retains full native-state/history and whole-interval gates. No missing learned assets block classical integration work. No numerical repair depends on this composition sequence.

Branch implement/c1b1-projection-qualification-20261020 is separate and dependent on retained PR125/PR122/PR120. No merge, rebase, canonical status overwrite, protected-source edit or branch cleanup. Lead owns dependency freshness, hosted CI, all merges and cleanup.
