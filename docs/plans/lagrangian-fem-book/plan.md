# Plan: Principles-first Lagrangian finite-element book expansion

**Plan status:** Active
**Current phase:** M2 audit milestone complete; full research closure pending
**Next slice:** Parent-coordinated research on the compressible material-mesh fluid derivation gap.
**Acceptance status:** partial

[Execution ledger](execution-ledger.md) · [Issues](issues.md) · [Source map](../../research-book/lagrangian/source-map.md)

## Objective and admission

Extend Puma's Rheon research book from material motion to an implementable reference FEM algorithm for solids and fluids. The user explicitly requested this new dedicated research task and separate branch. Canonical plan: `docs/plans/lagrangian-fem-book/plan.md`; operation `start`, admitted 2026-10-21. This admits research and manuscript work, not production implementation, deployment or merging. Completed product D1 design is outside this task.

## Objective acceptance

A1–A4 are M1 manuscript acceptance only. A5 accepts the follow-up audit artifact; A6 keeps the original full-research-closure scope visible.

| ID | Observable criterion | Kind | Environment | Mode | Status | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| A1 | Primary papers, exact project versions, source locations and authoritative license boundaries recorded | source review | public primary sources | manual | satisfied | [source map](../../research-book/lagrangian/source-map.md) |
| A2 | Principles-first derivations, conditional proofs, worked examples and complete bounded solid/fluid solver dataflow | manuscript | Markdown source | manual independent review | satisfied | [independent review](reports/independent-review.md) |
| A3 | Book cross-links, chapter ordering, equation syntax and whitespace checked without executing science | artifact | local text/render tools | automated/manual | satisfied | [verification](reports/verification.md) |
| A4 | Existing research, Puma credit, production source, thresholds/manifests and other branches preserved | scope | Git diff | automated/manual | satisfied | [verification](reports/verification.md) |
| A5 | Ordinary-text source/claim/coverage audit and local Markdown/PDF report reviewed and rendered without science | artifact/source review | local document tools | independent/manual | satisfied | [status report](../../research-book/lagrangian/research-status-report.md) |
| A6 | Full research closure, including missing compressible-fluid/ALE/coupling derivations and original-literature review | research | manuscript/primary sources | manual | pending | [omissions and next work](../../research-book/lagrangian/research-status-report.md#coverage-audit-fluids-and-solids-are-present-research-closure-is-incomplete) |

## Scope, constraints and assumptions

Write set: this plan directory; `docs/research-book/lagrangian/{source-map,review-guide,research-status-report}.md`; new chapters `30-*.md` through `36-*.md`; additive cross-links in book README, chapters 11 and 29, and existing fitted-height/fitted-ALE implementation guides. Local PDF/report previews may be generated outside Git using document tools only. No solver, admission manifest, qualification script, threshold, evidence fixture or historical proof edits. No third-party code, figures or substantial prose imported. Citations are research references, not dependency adoption.

Reference scope: 3D orientation-preserving finite-strain hyperelastic dynamics on a connected tetrahedral mesh; incompressible Newtonian material-mesh fluid branch with mixed velocity/pressure and explicitly differentiated geometry. Contact, coupling and remeshing are derived extensions with their own acceptance conditions; no universal solver is asserted. Mathematical assumptions and failure domains belong beside equations. No numerical experiment, Lean build, native/hardware run, model download or security change is admitted by this plan.

## Binding decisions

| Decision | Owner | Evidence | Supersedes |
| --- | --- | --- | --- |
| Append 30–36; preserve 1–29 and original attribution | research task | current main book order | none |
| Derive total-Lagrangian solid first, then mixed moving-fluid branch | research task | distinct constitutive/state requirements | none |
| Separate standard derivation, conditional exact-real proof, original exposition and executed evidence | research task | user request and book proof boundaries | none |
| Nonnumerical checks precede any full build | research task | teaching build calls packet validators before rendering | none |

## Standards and design observations

Source-guided standards review follows Rheon's adopted [Coding-Standards revision 188beda1](https://github.com/MrScripty/Coding-Standards/tree/188beda1fa477d21d576c233dd8c7f4c4c267d23): Core/Router, planning/discovery, implementation, development proportionality, verification, documentation, commit, licensing and terminology/ownership. No application standards-engine tool is exposed, so no automated conformance certificate is claimed. The authoring skill was inspected and excludes ordinary application agents. No Rheon/ancestor AGENTS.md or repository `.agents/skills` exists in inspected main/workspace. No language, UI, persistence, release or concurrency implementation profile applies to these authored Markdown changes.

Intent and constraints are explicit; no new D1 discovery exercise is necessary. Research is the requested deliverable, rather than an investigation delaying production. Stop source discovery after each requested topic has primary support and implementation-specific license/version evidence; unresolved claims must be qualified or removed.

**Composed-design review applicability:** not-applicable. This changes explanatory text and introduces no runtime composition, state owner or public/persisted program contract.
**Simplicity and ownership review applicability:** applicable to document structure. Existing chapters own particle/collision history; new chapters own FEM pedagogy; source map owns citation/version/license facts; ledger owns execution; review guide owns remaining qualification. Shared derivations are linked, not duplicated as competing production contracts.

## Milestones

### M0: Research plan and reading progression

Goal: make scope, sources and teaching order reviewable before substantive drafting. Write set: plan directory and source map. Gate: explicit acceptance claims, preserved-scope baseline and ordered chapter map. Status: Accepted.

### M1: Substantive manuscript

Goal: derive a connected reference solver with hand-worked examples. Write set: full declared Markdown set. Gate: A1–A4 and an independent read-only review with findings dispositioned. Status: Accepted.

### M2: Source-access and coverage audit

Goal: distinguish a useful draft from full research closure, map unread references to claims and report all remaining fluid/solid omissions. Write set: existing declared Markdown and new research status report; disposable Markdown/PDF outputs outside Git. Gate: independent review of attribution/evidence categories, source-access claim dispositions and readable nonnumerically rendered report. Status: Accepted as an audit/report milestone; full research closure A6 remains pending.

## Repository isolation and concurrent work

Main verified by successful `git fetch origin main`: `fd7c0f0aee63bc8710df99f9bb01154e7619b974` (2026-10-20 merge PR144). Original `/workspace/Rheon`, branch `work`, clean at `6ba41c7`, remains untouched. Task branch `research/lagrangian-fem-book-20261021` in task-created `/workspace/Rheon-lagrangian-research` starts from current main. PR145 force backend and PR146 UI remain independent open drafts. This task owns only its branch and worktree; parent owns integration. Retain worktree/branch for parent review; no merge, deployment or deletion. Synchronize only at a later parent-coordinated integration; no automatic rebase. Retirement condition: parent accepts/rejects and arranges safe disposition. One read-only review worker may be created after the draft; no duplicate active implementation worker.

## Blockers and re-plan triggers

Blockers: none at admission. A denied action stops at its target and is reported without alternate routing. Replan for production implementation, changed equation domain, mandatory numerical qualification, unavailable essential primary evidence or overlapping writes. Full teaching build and scientific qualification remain separately owned follow-ups, not silently passed exclusions.

## Final acceptance

A1–A4 satisfied for the M1 research manuscript draft; independent mathematical/source review and initial Pandoc/link/order/whitespace/scope checks passed. Full research closure is not accepted. Parent explicitly requested the M2 audit; its report tracks compressible-fluid, ALE, coupling, stabilization and source-access omissions. This does not accept a production FEM solver or scientific qualification. The branch/worktree is retained-protected for parent review under the isolation contract above. No publication/merge/deployment was performed. Full-text access limitations and future bounded qualification remain explicit in the source map/review guide; parent owns whether to commission those before publishing a comprehensive literature review.
