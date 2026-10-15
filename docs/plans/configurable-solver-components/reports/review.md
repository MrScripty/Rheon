# Independent documentation review

2026-10-15; read-only reviewer `/root/design_review`, GPT-6.1 Sol, separate from
the serial author. Reviewed research, proposed contracts, plan, standards review,
issues and ledger against current Rheon main `43ed26b` and adopted standards
`188beda1`. No source edits, numerical execution, inference or training performed.

**Outcome:** recommend accepting documentation design; no unresolved material
design finding. Q1/Q2 block implementation admission and do not block publication.
This is human-readable design/source review, not automated conformance or numerical
qualification. Supporting publication/link/scope checks belong to the serial writer.

| Finding | Evidence / correction | Disposition |
| --- | --- | --- |
| High implementation dependency: accepted recordings cannot reconstruct teacher-stage requests | [provider_step.rs](../../../../src/provider_step.rs) prepares RHS internally; [recording contract](../../../api/recording.md) omits residual/scratch. Contracts and M3 require bounded authoritative stage capture. | Addressed; Q3 remains future implementation gate |
| High implementation dependency: current fallback overwrites rejected pressure | [solver_provider.rs](../../../../src/solver_provider.rs) falls back after rejection. Stop-on-reject diagnostics need numerical-owned candidate-attempt outcome without fallback; observing completed production solve is insufficient. | Addressed in contracts/M3, no new loop authorized |
| Medium comparison requirement: adaptive runs may have no matched intervals | M3 now requires a nonempty matched-interval result and exact source publications alongside explicit unmatched coverage; no silent interpolation. | Addressed; Q8 retained |
| Lifecycle clarity | Future discovery/handoff does not become approved Planned implementation by being written. D1 concerns documentation only. | Addressed; future phases blocked/deferred |
| Low evidence wording | Local standards engine source exists; no application MCP tool exposed and application qualification not executed. | Clarified in standards review/Q9 |

The review confirms separate numerical acceptance, diagnostic recording, history
commit, runtime supervision and comparison authority. The scientific restrictions
include cubic cell spacing (not equal counts), origin support, actual dt and
viscosity-plan compatibility. New physics requires qualification. Remaining
implementation/platform/runtime questions have owners and revisit gates in issues.

## Review of integrated parent roadmap

After the parent supplied its draft, the serial author integrated its detailed
M0–M3 roadmap into the selected `configurable-solver-components` directory. The
same independent reviewer re-reviewed this material revision, including proposed
write sets, acceptance gates, issue/ledger reconciliation and documentation-only
publication authority; prior acceptance was not simply carried forward.

**Second outcome:** no unresolved material findings; recommend accepting revised
documentation design. M0 must reconcile actual source and exact write ownership;
M1–M3 remain proposed vertical workflow slices, not code authority. Shared public
contracts, schemas, fixtures, locks and workflows retain serial ownership. Cold
persistence, diagnostic isolation and nonempty matched-interval evidence remain
deciding gates. Q1/Q2 still block future implementation admission. No files edited
or numerical/runtime execution performed by the reviewer.

## M0 published-source reconciliation review

The earlier Q1 source-access and Q2 diagnostic-physics findings above are historical.
The same independent read-only GPT-6.1 Sol reviewer inspected the now-accessible
integration at `0fbadbe733b5b521e192dc8a2aa980d6d69a1e01`, full process/worker/UI
lifecycle, original and strengthened test, matching lab and retained receipts,
then reviewed [M0](m0-reconciliation.md) plus changed plan/contracts/issues/standards.
Original `66e50388...` ancestry is verified. Worker SHA256 and all five wrapper/
decoded log hashes independently match. No model/runtime/GUI execution was rerun.

Review confirms original cancellation step≥17/frames≥18 and restart16steps/17frames;
publication additionally asserts Linux immediate-PID absence and one restart AI
prediction. Normal Drop ordering supports reader teardown attempts before terminal
publication; no separate reader instrumentation, descendant custody, guaranteed
cleanup success or hard shutdown bound is established. Receipts bind worker/log
material, not authentication/full executable closure or absent deployment payloads.

One medium handoff inconsistency was fixed: diagnostic-only M1 does not satisfy
production A2 or M2 accepted-provider recording prerequisites. Unsupported viscosity
means values outside the diagnostic's actual mu=0.001 Pa·s; diagnostic retained
history and production acceptance rollback are separate policies. Existing Engine/
process/worker/JSON/UI are reused, not replaced by another loop or archive.

**Outcome:** no unresolved material findings after these corrections; source/design
M0 can close. Source access and diagnostic viscosity/no-fallback ownership are
resolved. Generic contracts, production composition, persistence and environment-
qualified implementation evidence remain pending and outside this task's authority.

## M1 implementation source review

After explicit implementation admission, the same independent read-only GPT-6.1
Sol High reviewer inspected the typed context, inert manifest, existing Engine/
ModelProcess/Worker integration and contract/lifecycle fixtures. Four findings
were fixed and re-reviewed: retain legacy frame-v2 for the actual lab reader;
bind prior grid/density; reject constructor pressure as solved prior; retain inert
preflight status after finished-worker rendering. Focused regressions cover each.

Final source outcome: no unresolved material M1 finding. The owned writer queue
is coherent for trusted immediate-child runtime execution. Descendant custody and
hard shutdown remain outside its guarantees. The reviewer ran no Cargo, model,
Torch or native-window execution; serial-writer validation is separate evidence.
