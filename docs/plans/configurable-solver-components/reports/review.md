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
