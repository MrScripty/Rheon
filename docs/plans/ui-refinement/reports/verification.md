# M1 implementation checkpoint and verification

Base: remote main `fd7c0f0aee63bc8710df99f9bb01154e7619b974`, revalidated
before planning and after implementation on 2026-10-20. Branch:
`implementation/ui-hierarchy-20261020`; worktree `/workspace/Rheon-ui-hierarchy`.
The accepted audit remains Library file `libfile_825ba1b07a3c8191a340bb761eea1c89`
version 0. [Independent plan review](plan-review.md) preceded production edits;
[independent implementation review](implementation-review.md) assessed the
actual source and evidence, including the repairs below.

## Implemented behavior

- A named Setup selector chooses Simulation preview or Accepted Session. Only
  the selected form and its primary launch action appear. Switching retains
  preview material mode and current numerical settings; it does not launch.
- Shared domain, physical resolution, steps, maximum dt, pressure, display,
  recording/resource and CPU inputs remain available to Accepted Session using
  carrier semantics even when retained preview mode is material.
- Action, actual progress, Cancel and status live in a persistent top strip
  above the scrolling form, including the Inspector tab. Buttons return intent;
  dispatch follows current form edits and existing package/launch validation.
- Full status is retained in a height-capped read-only text control with
  navigation/selection/copy, behind Status details. A concise projection stays
  visible. No string parsing invents error, success or completeness states.
- During an actual worker, the setup area shows frozen Request facts rather
  than unrelated next-run form values. The selected preference is not overwritten.
- Secondary comparison/reset actions are under Comparisons & reset; saved-file
  and mesh retention is visible when that group is open. Sealed-domain and
  model-scope limitations remain explicit for Accepted Session.

No numerical implementation/default, request construction, start/validation,
worker/cancellation, immutable-intent publication, recording codec, D1 or
Inspector implementation was changed. Added Request::setup_workflow is a UI
projection of its existing configured-set presence. No new dependency/framework.

## Executed checks

Toolchain: rustc 1.92.0 (`ded5c06cf`), Cargo 1.92.0 (`344c4567c`), Clippy 0.1.92.
All Cargo commands use the existing isolated Rheon toolchain/cache configuration
from `/workspace/rheon-setup/env.sh`, `--locked`, desktop features and one build job.

| Check | Actual outcome / proof boundary |
| --- | --- |
| `cargo check --locked --features desktop --bin rheon-desktop` | Passed for initial baseline; final source is also compiled by strict Clippy and test gates |
| `cargo fmt --all --check` | Passed after final source/test edits |
| `cargo clippy --locked --features desktop --bin rheon-desktop --tests -- -D warnings` | Passed; compiles/lints tests but does not execute numerical tests |
| `cargo test --locked --features desktop --bin rheon-desktop ui_setup_ -- --nocapture` | 5 passed, 0 failed; 152 filtered out |
| Same target, filter `sidebar_sections_collapse_and_preserve_their_state_through_real_egui_input` | Existing test passed unchanged |
| Same target, filter `docked_inspector_cannot_overlap_viewport_at_supported_window_sizes` | Existing first-frame bounds test passed unchanged after placement repair |
| Same target, filter `archive_opening_frame_suppresses_the_previous_live_inspector` | Existing transition test passed unchanged; absent archive path only |
| Pinned source `standards_verifier.checks.plan_contract.validate_plan` | Canonical plan structurally valid; source helper with explicit Python paths, not application-engine qualification |
| `git diff --check` | Passed |

Generated command logs are outside Git at `/tmp/rheon-ui-*.log`. Focused tests
exercise real egui pointer/paint events, not screenshots: mode selection,
preserved settings/no worker launch, carrier controls, disabled uninspected
adapter action, independent form scrolling while the action remains fixed,
Inspector composition, bounded long-status details, focused Ctrl+End navigation,
select-all/exact copy and attempted edit rejection, plus an inert Request summary.
No new test constructs a Worker or dispatches a launch. Existing selected tests
do not advance simulation. No model, corpus, evaluator or scientific suite ran.

## Failures found and repaired

1. Initial new tests used Rust struct-update syntax on Drop-owning App; compilation
   rejected that move. Tests now use ordinary App construction/field assignment.
2. Review found a saved-rerun projection could mix an active profile with retained
   next-run form settings. Active work now paints only its frozen Request summary.
3. Review found a plain Label inside ScrollArea did not provide keyboard access
   to the tail of long feedback. Immutable multiline TextEdit plus focused
   navigation/copy/immutability assertions replaced it.
4. Existing first-frame dock test exposed nested bottom-panel height-cache
   overflow. Bottom placement was superseded by a normal top-down strip before
   scrolling content, eliminating height estimation, extra sizing passes and
   blank reserved space. The unchanged test then passed. No test weakened.

These were ordinary M1 repairs. The placement change is recorded in the ledger;
it preserves the required persistent action behavior, authority and write scope.

## Acceptance limits

This is an implemented and source-reviewed M1 checkpoint, not whole-plan/native
acceptance. No native display/session exists here: DISPLAY, WAYLAND_DISPLAY and
XDG_RUNTIME_DIR are absent, and `/tmp/.X11-unix` does not exist. No native GUI was
started or captured. Headless input/paint tests do not prove native Tab order,
focus visibility, contrast, scaling, actual active-worker cancellation or usability.
Direct test focus assignment proves only behavior after focus is acquired.

No 40% visible-text reduction, user timing, scientific validation or performance
gain is claimed. Contemporary matched native captures and user observations
remain required. The historical audit capture is not a contemporary baseline.
No standards application tool was exposed, and the source structural helper is
not a standards conformance certificate. Inspector slice/dependency remain
untouched and unvalidated by this task. [Native handoff](native-validation-handoff.md)
defines the next evidence boundary without authorizing numerical work.
