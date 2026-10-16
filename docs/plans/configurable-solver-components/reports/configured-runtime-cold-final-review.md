# PR110 final runtime cold binding independent review

Read-only review of exact `c230cf7a4b9a5e68ed2b7cb44add634a77a6327e`, tree `0fcd85d0333ccbb9e7e129c5ee13445a9f4cd8e5`, against `be0ed8efd74160d3b42b96b948d342288aa44b49`. Reconciled all three final paths with the prior complete fef77d2 review and independently inspected the full fef77d2-to-c230cf7 clock/fixture/document correction and owner call sites. No Cargo, numerical/model/runtime execution or repository changes by this reviewer. Exact base-to-head diff-check passed.

**The original medium blocker is resolved. No material source finding remains; approve bounded source admission of these exact three paths and fresh composed qualification.** This final assessment supersedes the pending assessment in `/tmp/rheon-configured-runtime-cold-independent-review.md`; the historical report is intentionally unchanged. Application worker/UI/numerical/model acceptance remains separately qualified.

## Clock correction

After full typed RCT1 audit and the frozen accepted-generation horizon check, runtime_custody accumulates from positive zero, adding the frozen requested_dt once per accepted generation. The iteration count is bounded by validated requested_steps (at most 10,000). The check is independent of observation presence, so None-observation/unrecorded tails cannot bypass it. Terminal accepted_time above that conservative upper clock is rejected. The core typed audit already validates finite/nonnegative coherent terminal clocks; no fabricated missing interval start, exact history or dispatch fact is added here.

This bound is conservative for the admitted profile: the actual owner starts at positive zero and advances by positive actual dt no greater than the frozen requested dt; floating-point addition is monotone for these nonnegative operands. Repeated addition preserves the owner's clock accumulation order. Multiplication, endpoint subtraction and arbitrary epsilon are correctly absent. A shorter actual clock remains allowed without being independently authenticated by the bound. The same helper runs before unpinned/member exposure and inside pin/publication validation, including already finalized member opens; pin presence does not skip this guard.

The new regression constructs actual accepted Session intervals after an actual one-record recorder cap failure, explicitly labels continued numerical advances as a fixture rather than a production worker behavior, and finishes a None-observation typed StorageFailure terminal. The negative mutates only its public terminal accepted_time to 4*DT for two actual steps. The public generic/core runtime audit first succeeds with UnrecordedAcceptedTail and retained constructor generation zero; the app then rejects the exact frozen-clock error, pin_closed fails without setting finalized and unpinned open returns Invalid. Thus it exercises the original hole rather than relying on an earlier unrelated refusal. The new test does not separately manufacture a forged finalized pin; the unconditional same runtime_custody call before finalized branching was independently checked in source, so this is a coverage limit rather than an unclosed source defect.

Positive cases retain an honest observation-free tail, actual host-shortened intervals and an actual 64-step 1/60 owner clock. The latter explicitly proves repeated accumulation is greater than generation*dt and remains cold-pinnable/Ready, preventing an accidentally overstrict multiplication fix. The prior dt negative is correctly retargeted only to the next unaccepted request's 2*DT and matching interval_end; accepted seed clock stays honest, preserving the distinct exact dt guard assertion. Original eight archive contracts remain unchanged; it would be false to claim all sixteen pre-fix test entries were unchanged.

## Retained custody boundaries

The three-path full change retains LegacyUnavailable separately from typed terminal audit; unknown reserved numeric RCT versions are Unsupported, malformed known payloads fail, generic prose and absence do not invent no-dispatch. RCP3 association and typed terminal audit are separate from physical candidate acceptance. Frozen request horizon, dt, exact coefficient, package identity, native-prefix phase and lower-bound native history checks remain in the existing owner. Available accepted row stage presence distinguishes Disabled versus applied zero viscosity; an unaccepted request's mu alone does not invent stage presence.

Header/constructor controls, exact expected source/run, strict typed profile, immutable archive hashes, path policy, complete/failure/cancelled recorded-prefix handling and Ready checks stay owned by the cold archive module. No deployment, Session/model process, alternate recorder/schema/numerical loop is constructed on cold inspection. The accessor alone does not grant complete member readiness or reconstruct missing RHS/forces/clock/history. SQLite's existing open/recovery contract remains inherited. Documentation accurately describes the new bound as consistency with declared intent, without authenticating missing execution.

## Execution evidence and limits

Read `/tmp/rheon-pr110-clock-final-public-receipt.md`, SHA-256 `99421b0305a90888b6cd946e1fd6f47389831d8578ab5729acf3da858c4c1619`, which reports an independent final exact-revision 88/88 result: archive17, intent9, carrier4, provider20, runtime21 and native17, zero failures/ignores/filters, plus strict affected desktop/test Clippy, fmt and diff checks. Public handoff is PR110 comment 6206029239. These are separately attributed external executions, not my reruns; embedded external raw artifact hashes were not independently replayed here. The receipt retains the initial guard run's failed obsolete fixture and explains its isolated repair rather than transferring that failure into a passing claim. Existing twelve-case producer-exit/PATH-empty cold-reader coverage remains source-reviewed. Authored operational metadata is not promoted to actual ModelProcess/IPC/model/GUI evidence. Root must admit final exact source pins and freshly qualify its composed consumer tree.

## Exact final pins

| Path | SHA-256 |
| --- | --- |
| `docs/api/configured-session-intent.md` | `0b3debf93ebacca8e45df814fd67694eac52f5bcac95a89c193b392b2e77b71d` |
| `src/bin/desktop/run_sets/configured/archive.rs` | `eb1d8e77019eb57afd9e5fd5f57af55f8ff320527ec8efc935945bd7ff477806` |
| `tests/configured_archive_binding_contract.rs` | `821ddd7d173c84453650d2cde938bf024d907e2f062a894ea2b182e44ac56932` |
