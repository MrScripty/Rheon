# Isolated Water/Air voxel implementation acceptance

The native app can now author binary Water/Air initial contents and run actual coupled two-material voxel dynamics through its existing Worker. M1–M5 and A1–A6 are **Accepted bounded** by [independent final review](m5-final-feature-review.md). This is a complete isolated candidate for draft review; remote CI, reconciliation with current main and sole-lead integration remain pending. No merge, main move or deployment is authorized.

## Implemented behavior and limits

- Dense f64 MAC state, variable-density pressure, conservative liquid fraction/phase mass with matched staggered momentum, variable symmetric-strain viscosity and frozen gravity. Water and occupied positive-mass Air share velocity/mechanical pressure in a filled closed free-slip box. First-order diffuse interface transport; no surface tension, EOS/compression, heat/phase change, free surface/vacuum evolution, runtime authored forces/emitters, GPU or arbitrary-fluid support.
- Exact binary initial lattice, one round brush, Add/Remove/Replace, material/radius, Layer/Volume and XYZ interior access, preview and numeric dab. Remove creates Vacuum and refuses both launch controls until explicitly filled; Replace Air changes occupied cells and Add Air fills Vacuum. Undo/redo commits deterministic atomic strokes with cancel/no-op/redo rules and bounded history.
- Strict saved drafts and immutable frozen inputs preserve IDs, geometry, captured coefficients and policy. Save preserves editing history; Load creates fresh session history. No crash-persistent undo/autosave is promised. Typed original-f64 recordings and Inspector expose both phases; saved rerun starts from the original recipe, unaffected by later Draft changes.
- Existing Worker/recorder own mutable simulation/publication. Pending launch retains old results until matching durable constructor and typed input acknowledgment. Failure/cancel preserve the appropriate confirmed prefix. Protected local no-replace publication distinguishes pre-link I/O failure from post-link uncertainty without blind retry.
- Numerical admission is4–64 cells/axis; fresh native authoring is cubic, zero-origin, equal-spacing, side0.001–100m, at most1000cells/m. Saved admitted geometry is preserved exactly.128MiB ordinary numerical policy and384MiB enumerated managed-host-payload admission are allocation policies, not measured process RSS. Convergence/stability/resource refusal remains explicit; these ranges are not exhaustive accuracy qualification.

## Source-bound evidence

| Boundary | Exact identity / evidence |
| --- | --- |
| Approved plan | `627847915eb97b80da81ec4544bb413bc5f806fa`, old production base `d67aec1a4575e0a61aa8cd931c8eeec33c8dc6a3` |
| M1 / M2 / M3 components | `638008d`, `d0e92b7`, `8a5fdcb`; [reconciliation](m5-acceptance-reconciliation.md) links the original checks and corrected independent reviews |
| Accepted M4 | `1e8c00fe32c88a2571ae583f26e0626c540d5845`; actual qualified non-test source `0059ca0`, distinct preserved binary `265de7f1039ffaaddc97862c5ae00bffdc4a05279d2e342439f491ad58bac367` |
| Final ordinary production/test source | `ca5922a5c167aa939a765dc9a0b13ab61bfd382a` |
| Ordinary tracked source-material declaration | SHA-256 `32906c3300a4dff33742486a49f2fd7a296901c6264628f0a47004387ca3e3df` |
| Preserved ordinary executable | `/workspace/.voxel-display/m5-rheon-desktop`, SHA-256 `94a6056c327f18854a5197a279e30d0107476f362b0305261dd032acb856074c` |
| Ordinary protocol / captures / typed result | [Executed native report](m5-normal-artifact-evidence.md): actual non-test debug build, no CLI args, real OS pointer/keyboard input, installed private authenticated Xorg dummy/Mesa software, one4³ zero-gravity0.001s interval |
| Final review | [Complete separate feature/evidence review](m5-final-feature-review.md): all19 ordinary captures,37 tabulated evidence hashes,28 reviewed current source identities, no blocking correction |

Later delivery commits contain documentation only. They do not relabel the binary as rebuilt from a later documentation HEAD. Artifact hashes are observed local custody, not authentication or reproducible-build certification. Pixels, logs, binaries and SQLite are local evidence outside Git; reviewed reports contain identities and findings.

## Selected verification and corrections

[Final focused verification](m5-verification.md) records actual commands/results. Three missing numerical observations each passed a focused check: a manufactured conditional dual-mass defect, analytic piecewise shear traction and a second full-step hydrostatic geometry. No production numerical formula, budget or tolerance changed. Independent numerical R1/R2 rereview resolved the findings. Water/Air8/16/32 phase refinement combines with separate exact five-contrast mass/momentum/pressure oracles under the reviewed fixed-Q decomposition; five unexecuted refinement cohorts are not claimed.

Two publisher fixtures passed six test-only failure boundaries and normal/no-overwrite cases. A real fresh child process reopened complete and Vacuum-containing drafts without numerical work. Five ordinary controls checks and final affected desktop filters passed:52 voxel parent tests (plus explicitly invoked children), Scene10, contents10, setup9 with one pre-existing ignored, forceUI7, saved payload1, configured Worker6, configured comparison5 and legacy saved Worker1. Overlapping filters are not summed as unique tests. Strict core/desktop all-target Clippy, formatting and diff checks passed. The final explicit closed ordinary-artifact audit passed1; its earlier compile-only ignore is not counted as success.

Actual M4 native evidence already covers8³ moving Water/Air, paint/undo/redo/save/alter/load/run/typed both-phase inspection, later Draft isolation, fresh native cold rerun with exact raw fields and genuine cancellation. It was not repeated. The changed ordinary artifact separately covers startup, count4/Create, explicit emitter disable, Water replacement, Remove→Vacuum/refusal→Undo, Save/alter/Load, actual global launch and typed result/open-saved readiness. Its exact four Water IDs are37/38/41/42, with60 occupied Air. This one-interval smoke did not click rerun/cancel or claim moving-interface behavior; those contributions remain the accepted prior evidence. Both owned App/display processes exited0.

One nonblocking final-review finding remains: the global startup last-status may still mention the legacy rotating emitter until a later contents action updates it. Actual readiness, emitter-disable remedy, frozen intent and backend are correct. [I20](../issues.md) assigns later copy cleanup without claiming changed code inherits this binary evidence. No native physical-GPU, packaged-release, performance, calibrated scientific realism, arbitrary-grid or cross-build-replay validation is claimed.

## Delivery, ownership and remaining gates

The full branch history `d67aec1..ca5922a5` was inspected before delivery:16 linear topic commits, no rewrite/merge topology, changes restricted to this topic plan/reports, new numerical/authoring/recording owners and reviewed targeted existing boundaries/tests. No common README/index/chapter, workflow, Cargo/lockfile, thermal, portal or GPU task file changed. Temporary qualification transport existed only in historical qualified commits and is removed in the final ordinary source. Dependencies are unchanged.

Branch `implementation/voxel-multimaterial-authoring-20261022` and `/workspace/Rheon-voxel-material-implementation` are `retained-protected`, reachable through the named branch. Root retains them for parent/draft review; sole integration lead `01a16096-15e8-706f-8479-1f318a3c601d` owns integration. No automatic rebase/synchronization/cleanup. Retirement follows explicit disposition and protected-OID checks. Original research/planning worktrees and all unrelated work remain preserved.

The latest read-only PR59 observation at preparation is open/draft/unmerged, base `20a0e1e20538b859193c08eaf25a3b8cd26135a5`; parent reports its fixed performance cohort failed. No receipt transfers to this feature. This branch is intentionally older than that frozen main. A draft PR can carry the complete candidate, but cannot admit current-main compatibility, mergeability or remote CI before actual results and lead review. Reconciliation/reverification must occur only at the lead-approved boundary. Historical denied metadata/comment actions and uncertain Library delivery remain stopped; no replacement upload/comment or benchmark was attempted.
