# Focused desktop UI refinement

**Plan status:** `Active`
**Current phase:** M1 native verification (source checkpoint complete)
**Next slice:** M1 — native visual/interaction validation in the existing authorized environment
**Acceptance status:** `blocked`

**Execution ledger:** [execution-ledger.md](execution-ledger.md)

**Issues:** [issues.md](issues.md)

[Standards/design](reports/standards-and-design.md) · [Plan review](reports/plan-review.md) ·
[Implementation review](reports/implementation-review.md) · [Verification](reports/verification.md) ·
[Native handoff](reports/native-validation-handoff.md)

## Objective

Reduce visible text and competing controls while making Rheon's actual native
application easier to use. The user accepted audit
`libfile_825ba1b07a3c8191a340bb761eea1c89`, version `0`,
`rheon-ui-audit-2026-10-20.txt`, then authorized a standards-compliant plan and
implementation. Canonical admission is this path, operation `start` after the
independent plan review; subsequent implementation uses `continue`. No optional
approval is required for the reversible first slice.

Base remote main revalidated 2026-10-20:
`fd7c0f0aee63bc8710df99f9bb01154e7619b974`. Local branch
`implementation/ui-hierarchy-20261020`, worktree `/workspace/Rheon-ui-hierarchy`.
UI task owns implementation and serial integration; parent owns eventual main
integration. Branch is private, retained for review/validation. No push, PR,
merge, rebase, branch deletion or cleanup is authorized in this turn.

## Scope and non-goals

First: explicit preview versus Accepted Session setup, one primary launch at a
time, fixed run-action/status area, unchanged launch validation and worker
ownership. Later: collapse optional setup prose, shared control geometry and
accessible help, camera/playback consistency, D1 comparison/archive presentation.
Keep units, relevant limitations, blocking errors, absence/uncertainty, pressure
stage/time and integrity-versus-completeness distinctions accessible and explicit.

No numerical/tolerance/default-physics changes, solver/model/evaluator/corpus
execution, L1/MMS work, storage migration, new framework/dependency, web/Pages
redesign or wholesale D1 replacement. Do not modify README or local agent work.
No speculative integration of Inspector changes.

## Dependency strategy and ownership

M1 is based on main alone and does not depend on local Inspector slice
`241dfb5237db0fa064d1257f4baf12781e0a0d44` or dependency
`1424f2d55323825c8907cf9209914f29130ec7de511`. Their separate validator task
`01a15a9f-ffcc-75ee-bd76-02f1c277f027` owns validation and its test-only Clippy
repair. Preserve its branches; do not copy/rebase/merge their code. Parent relays
coordination because cross-thread messaging is unavailable in this environment.
M4 must revalidate that work's final source before admitting Inspector edits.

## Milestones

| Milestone | Goal and exact production write set | Dependency / gate | State |
| --- | --- | --- | --- |
| M1 | One selected setup workflow and persistent run controls. `src/bin/rheon-desktop.rs` (sidebar, presentation methods, UI state/tests only), `src/bin/desktop/configured_controls.rs` (form/action split and labels only), optional `src/bin/desktop/setup_ui_tests.rs` | Independent plan/design review; A1–A3 plus A6 supporting checks; A4 native evidence remains required | Verifying |
| M2 | Compact optional setup, aligned fields and keyboard-accessible help. `src/bin/desktop/{settings,collision_ui,emitter_controls}.rs`; optional new `src/bin/desktop/form_controls.rs` | M1 native feedback; preserve policy and unit editing; A4–A5 | Planned |
| M3 | Camera/transport consistency. `src/bin/desktop/{viewport,particle_view,engineering,transport_view}.rs`; optional `src/bin/desktop/playback_controls.rs` | M1/M2; separately re-admit exact main-file playback regions if needed; focus and session-ownership evidence | Planned |
| M4 | Promote existing D1 comparison/archive results. `src/bin/desktop/{comparison,comparison_field_view,archive_view,run_set_view,inspector}.rs` and `src/bin/desktop/ai_comparison/{mod,diagnostic_view}.rs` | M3 and final Inspector revision reconciled; re-admit exact composition-root regions; saved-fixture integrity/absence evidence | Planned |

Production write sets are disjoint across planned milestones. A later need to
touch earlier integration regions is an explicit serial admission, never
parallel ownership. Only the UI lead writes this plan directory; reviewers own
non-authorizing review reports. Authored plan, ledger, issues and review reports
may change throughout. Generated logs/renders remain outside Git per
`docs/REPOSITORY_OUTPUTS.md`.

## Binding Decisions

1. `App` owns the setup selection; it does not change `material_mode` or any
   numerical settings when switched. Preview retains material/carrier choice.
   Accepted Session exposes its own physics form plus the existing shared
   domain, grid, steps, maximum dt, pressure, preview, resource and CPU settings
   that its existing launch consumes. Hide irrelevant emitter/mesh/material
   controls for that workflow; do not hide consumed launch inputs.
2. Existing `start`, `start_configured`, validation, immutable intent publication,
   `Worker`, `poll`, cancellation and child cleanup remain authoritative. UI
   selection is disabled during active work. No new simulation loop or copied
   worker state. Settings/Inspector switching must retain run control access.
   Active work shows only frozen Request facts, not retained next-run form values.
3. Split configured form painting from its launch button so one fixed action
   area above the scrolling form can return intent. Paint the current form and
   process readiness invalidation before executing intent through existing launch
   validation. Keep package inspection/readiness and
   invalidation in `ConfiguredControls`; do not expose its private representation.
4. Preserve full status/error text in keyboard-reachable, height-capped scrolling
   Status details; keep a concise single-line projection visible outside it so
   long paths/errors cannot displace Cancel or the form. Shorten routine labels
   and status copy only.
   Existing string status is not parsed to invent severity/completion semantics.
   Compact progress uses the existing worker's actual numerator/denominator;
   cancellation remains the existing operation, not inferred success.
5. Keep advanced secondary actions available under task-named disclosures.
   Preserve relevant limitations and clearing/retention consequences. No
   unexplained icons or reduced body font/hit targets to meet text targets.
6. No browser renderer is substituted for actual native validation. The audit's
   historical `a552d0a` capture is not a current baseline.

## Objective Acceptance

| ID | Observable criterion | Kind / environment / mode | Status | Evidence |
| --- | --- | --- | --- | --- |
| A1 | Actual egui pointer selection presents only the selected setup form and one primary launch; switching preserves settings and never launches work | integration / simulated egui input / automated | satisfied | [Verification](reports/verification.md) |
| A2 | Primary action/status remain outside scroll content, including Inspector mode; shared consumed inputs remain reachable; disabled package launch remains explicit | integration / simulated egui input / automated | satisfied | [Verification](reports/verification.md) |
| A3 | Launch/request/worker/cancel/recording semantics unchanged; no Inspector/D1/numerical edits; unit and limitation copy retained | focused / not-applicable / manual source review | satisfied | [Verification](reports/verification.md) |
| A4 | Native 1250×820 and 900×650 at 100% and 150% UI scale: no clipped labels/overlap; mode/action/status visible; scroll/keyboard/focus/help and in-progress cancellation work | user-workflow / required-real supported native display / manual | blocked | No display/session here |
| A5 | Matched current before/after states show materially less text with unchanged values/warnings; target 40% fewer visible explanatory words across full planned refinement, not M1 alone; 4/5 users identify workflow/edit material/find launch within 30s | user-workflow / representative native / manual | pending | No counts/usability results yet |
| A6 | Rust 1.92 formatting, affected desktop build, strict affected-target Clippy and focused UI tests pass | focused / Linux pinned Rust / automated | satisfied | [Verification](reports/verification.md) |

Before expanding beyond M1, native owner observes real default/switched/Inspector
screens and overflow; that observation can revise panel placement/width, not
scientific semantics. Run ongoing-worker/cancel UI coverage only with an approved
UI-only fixture or independently authorized run; this turn authorizes no solver
or model execution. Headless paint establishes emitted layout/input behavior,
not human perception, native scaling, accessibility or complete workflows.

## Simplicity And Ownership Review

**Applicability:** `applicable`

The desktop composition root changes.

- Independent concepts and dimensions: App selects displayed setup and routes intent; ConfiguredControls
   owns package/form readiness; worker owns execution; recording owns durable
   truth. Only presentation timing/placement changes for desktop users.
- State, identity, value, time, policy, and mechanism: required interleaving is selected intent plus actual worker busy state.
   Avoid coupling UI selection to physics mode, timeline selection or archives.
- Caller and composition-root knowledge: App knows the selected workflow and asks ConfiguredControls to paint its
   form/action; it does not know package pin details or invent execution status.
- Representative change paths and forced owners: new workflow label changes only presentation. New configured readiness rule
   stays in ConfiguredControls. Cancellation still changes only worker owner.
- Stable Interfaces versus hidden knowledge: the form/action interface carries enabled state and action intent; it does
   not leak provider layout, source pins or shutdown ordering.
- Independent evolution, testing, failure, and replacement: forms are paintable/testable without execution. Existing worker failures
   remain full status feedback; a failed package check cannot enable launch.
- Deletion and cumulative machinery result: no permanent registry, adapter or generic UI framework is introduced. Removing
   selection/placement would restore competing launch forms, so that mechanism
   contains the user-facing hierarchy requirement. Ordinary functions suffice.
- Necessary complexity and containment: inherent complexity is preserving two existing run profiles and shared
   inputs; contain it in App's form composition. No new persisted UI schema,
   parallel state model or solver abstraction is retained.

## Rollback and re-plan

M1 is presentation-only, reversible by reverting its isolated local diff/commit;
no migration, archive deletion or saved-data rewrite is needed. Preserve branch
and worktree until parent accepts a terminal disposition. Rollback must not
reset another worktree or alter the Inspector/numerical branches.

## Re-Plan Triggers

Re-plan for missing consumed launch inputs, changed package/worker ownership,
native evidence contradicting placement, overlap with Inspector integration,
unintended numerical changes or an inadequate UI-test oracle. Fix ordinary
lint/test failures inside the admitted boundary without restarting design.

## Blockers

Native display and representative user evidence are unavailable, so objective
acceptance remains incomplete. No standards application MCP is exposed; direct
source standards review is used and no engine conformance certificate claimed.
Build/tool availability and any baseline lint failures are recorded in the ledger.

## Final Acceptance

Native visual/interaction and matched-state usability claims remain unsatisfied.
No delivery, standards-engine certification or whole-plan acceptance is claimed.
