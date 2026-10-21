# Thermodynamics research book publication

**Plan status:** Verifying
**Current phase:** Exact-head normal publication verification and draft PR handoff
**Next slice:** Obtain exact-head normal CI/PDF/browser evidence and return the reviewed draft PR to the parent integration owner.
**Acceptance status:** partial

Canonical plan: `docs/plans/thermodynamics-book/plan.md`; operation `start`, admitted by the explicit 21 October 2026 user publication follow-up. [Ledger](execution-ledger.md), [issues](issues.md), [verification](reports/verification.md), [independent review](reports/independent-review.md).

## Objective, authority and constraints

Publish the reviewed thermodynamics research as Chapters 41–47 on Rheon's existing GitHub Pages book, with primary-source/license/scope material and preserved Puma credit. User authorized publication, PR and eventual merge/deployment after independent review and exact-head CI/PDF/browser checks. Parent retains the final repository-lead merge decision. This worker prepares the additive branch and draft PR and returns evidence before any merge. No production thermal solver, numerical retuning, proof/admission change or GPU publication is admitted.

Preserve research commit `0f9652572b55419424e51dfd4431acd6f19b9321` and its thirteen documents exactly. It descends from fetched main `b9218038d138d1c73818cf4158757521853a5cd7`. Publication branch `documentation/thermodynamics-chapters41-47-20261021`, worktree `/workspace/Rheon-thermal-publication`, includes that commit by fast-forward and adds the publication projection. Original research and initial work checkouts remain untouched. No force-push, branch deletion or history rewrite.

Portal PR148 was merged on main as `d67aec1a4575e0a61aa8cd931c8eeec33c8dc6a3`. Incorporate that main through an ordinary merge while preserving Chapter40 and its source/audit files; do not modify its branch. GPU37–39 remain separate publication work; FEM30–36 is published. The one additive README conflict retains both complete entries. Numbering41–47 leaves those slots intact.

## Write set and binding decisions

Write set: this plan directory; seven new `docs/research-book/chapters/41-*.md` through `47-*.md`; new `docs/research-book/implementation/thermodynamics-sources-and-scope.md`; new `docs/research-book/sources/thermodynamics-{license,repository}-audit.json`; one additive paragraph in `docs/research-book/README.md`. The thirteen historical draft files are inherited unchanged, not edited. No root README, shared bibliography, publication workflow, builder, index generator, existing chapter, production source, lockfile or numerical fixture changes.

The current builders discover chapter and implementation Markdown automatically. Use that mechanism; do not add an interactive thermal lab or new publication plumbing. Chapter mathematical expressions/hypotheses remain identical to the reviewed draft. Only status and relative links change. The public guide owns bibliography, source/license boundary and the gap map. Audit copies retain historical identities and the original draft purpose, with that provenance explained in the guide.

## Objective acceptance

| ID | Observable claim | Kind / environment / mode | Status | Evidence |
|---|---|---|---|---|
| T1 | Seven principles-first chapters, no pseudocode/copied material, exact reviewed equations/hypotheses | manuscript / text / independent | satisfied | Historical [review](../../research-book/drafts/thermodynamics/review-notes.md); publication retention independently passed |
| T2 | Primary identities, actual access limits, Apache license, compatible selected use and exclusions preserved | source / primary authorities / independent | satisfied | Historical review and new scope guide; integration independently reviewed |
| T3 | Chapters/guides discovered, relative links/anchors and equations render, old work preserved | artifact / supported document tools / automated + independent | pending | [Verification](reports/verification.md) |
| T4 | Exact publication head passes normal required CI including PDF/browser/site checks; actual artifacts reviewed | release artifact / normal runner / automated + manual | pending | [Verification](reports/verification.md) |
| T5 | Draft PR, exact head, independent review and evidence returned before parent-controlled merge | handoff / GitHub / manual | pending | Completion receipt |
| T6 | Actual main integration and Pages deployment verified by integration owner after dependency reconciliation | publication / GitHub Pages / normal pipeline | pending | Parent-owned follow-up |

No source parser, local reading export or successful math render substitutes for T4/T6. No thermal scientific evidence is claimed.

## Milestones, routing and review ownership

M1 integrates one coherent documentation slice with the write set above; gate: original draft hashes, all chapter expressions, scoped local links and independent source/math/integration review. Status: Accepted. M2 runs the unchanged normal exact-head pipeline, renders/inspects reading artifacts and hands off the draft PR; gate: actual scoped CI/PDF/browser evidence, with unavailable artifact access reported. Status: Verifying. M3 is parent-controlled merge/deploy after portal coordination; worker does not perform it. Status: Planned.

Source-guided standards: adopted Coding-Standards188beda1 Core/Router, Planning, Implementation, Verification, Development Proportionality, Documentation, Commit and Licensing; release guidance applies to eventual published artifact verification. No AGENTS.md or applicable local .agents skill was present. No standards-engine certificate is claimed. Intent and source design are already explicit; no new discovery questionnaire is needed. Composed-design review: **not-applicable** because this changes explanatory text and its existing build projection, with no runtime composition or program contract. Documentation ownership: public guide for sources/scope, chapters for pedagogy, historical draft for preserved research record, plan/reports for publication lifecycle/evidence.

One existing independent reviewer is assigned read-only review of the complete publication delta; no shared writes, code execution, external mutation or cleanup. It checks exact draft/equation retention, paths, public claims and license/source boundaries and returns findings for serial integration. Reports do not grant admission or mutate plan authority. Parent owns final integration; this worker owns its new documents and branch. No concurrent-plan profile is needed for read-only report work; overlapping shared README integration remains parent-serial.

## Blockers, re-plan and retained resources

Normal exact-head publication evidence remains pending. Portal main integration is complete locally. Draft PR creation was rejected once by automatic approval review, which cited the original research-only prohibition despite the later parent-relayed user publication authorization; no PR was created and no alternate route or retry was attempted. Further PR creation requires resolution with materially new relevant authority. If artifact access is denied or stale in this executor, stop that operation and report exact target/action/outcome; do not recreate a worker, change networking, bypass access or retry uncertain Library uploads. Local PDF review and other independent work may continue within the existing publication rules. A source or equation defect, unanticipated production change, incompatible retained material or unsupported publication path triggers scoped re-plan.

Both task-created worktrees are retained-protected for parent review with heads reachable from their named branches; no cleanup is attempted. Research commit remains a protected ancestor. Retirement requires parent acceptance/rejection and safe recorded resource disposition. No generic prune or deletion of other worktrees is authorized.
