# M1 native validation handoff

Owner: existing authorized native validation environment, coordinated by the
parent. Do not create a duplicate Inspector/validation effort. This source is
main-based UI work; do not merge/rebase the Inspector slice to obtain a display.
Its separate validator task is `01a15a9f-ffcc-75ee-bd76-02f1c277f027`.

## Candidate and build custody

Use the local checkpoint on `implementation/ui-hierarchy-20261020` at
`/workspace/Rheon-ui-hierarchy`; resolve and record its full commit before transfer.
Its base is `fd7c0f0aee63bc8710df99f9bb01154e7619b974`. Confirm the working tree
matches the checkpoint and record source/tree, dependency lock and actual binary
identity. Do not relabel another binary or the separate Inspector binary as this
candidate. Production source/test hashes are in the independent review.

For a source-bound build use the existing `tools/build_qualified_desktop.py`
recipe documented by `docs/api/qualified-desktop-build.md`: exact clean committed
source, a fresh output outside all checkouts, declared existing trusted Cargo and
Rustup homes/PATH, Rust 1.92.0 and locked desktop features. Select actual paths in
the validation environment; do not copy this environment's credentials/settings.
The wrapper builds but does not execute either binary. Its sidecar is bounded
build-material evidence, not numerical or hermetic qualification. A developer
build is acceptable for exploratory UI feedback if identified as such, not for
the stronger source-custody claim.

Launch only the desktop UI with no workload-triggering flags. Do not pass
`--auto-run`, `--run`, model/deployment options or any numerical CLI command.
Do not select Obstacle lab (initialization solves a projection) or run/advance
Transport. If the selected environment cannot display the app normally, report
that exact block; do not change sandbox/graphics/session settings to bypass it.

## Procedure requiring no simulation/model workload

1. Record OS, display/session, graphics backend, UI scale, actual window size,
   binary/source identity and state isolation. Compare this candidate against a
   separately identified main-base UI at 1250×820 and 900×650, 100% and 150% scale.
2. Capture default Simulation preview, Accepted Session, expanded Simulation,
   Domain, recording/resource and CPU sections. Verify one primary launch,
   reachable shared inputs/units, preserved profile limitations and no clipping.
   Switch back and confirm edited next-run settings persist without executing.
3. Scroll each long form and open Inspector with no run selected. Confirm the
   action/status strip remains visible and the viewport is not overlapped.
   Open/close Status details; if long feedback is needed, use a rejected open of
   a known nonexistent archive or an approved UI-only fixture, never a real run.
   Confirm long feedback remains bounded, readable and exactly copyable.
4. Use keyboard Tab/Shift+Tab to reach Setup, action and Status details. Open the
   disclosure and acquire its read-only text control without direct test focus
   assignment. Navigate to the tail, select/copy and attempt text input; verify
   immutable content, visible focus and a usable exit path. Test pointer
   equivalence and ensure hiding a form does not strand focus.
5. Select Compare with CPU adapter without inspecting a package. Confirm the
   primary paired action is disabled and its reason visible. Do not inspect or
   execute a model package for this UI pass. Do not click any enabled run action.
6. Retain matched images with annotations for hierarchy/spacing/affordances and
   record observed failures. Count visible explanatory words only for identical
   states/window/scale; separate values/units/warnings from helper prose. The
   40% target belongs to full staged refinement, not an automatic M1 pass gate.

## Active-worker evidence still required

Actual progress, frozen-rerun summary, disabled setup selection and Cancel use
the unchanged real worker owner. They have not been exercised natively here.
Validate them only with an existing approved UI-only fixture or separate explicit
authority for an actual run; this handoff grants no simulation/model authority.
If neither is available, keep that claim blocked. Do not fabricate a worker,
relabel recorded images or treat the inert summary test as a live workflow.

No completed/failed/cancelled run, persisted-source integrity/completeness or
model accuracy claim follows from startup, paint or the above idle-state checks.
Those states retain their existing owners and are separate from source/visual
evidence. Report each covered state/modality and every uncovered required claim.

## Return evidence

Return candidate/build identity, annotated current native captures, exact
pointer/keyboard operations and outcomes, size/scale coverage, focus/overflow
findings, permitted state coverage and blockers. Keep generated images/logs out
of repository commits. Parent can accept M1 only after the named native evidence
is satisfied, or keep it Verifying and revise the layout. Later M2–M4 remain
planned; reconcile the final Inspector source before admitting M4 edits.
