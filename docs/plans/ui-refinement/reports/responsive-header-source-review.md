# Independent responsive viewport header review

2026-10-20. Reviewed only the responsive mode-tab repair relative to
`60f3498f19403fc449aa2d868a4fdf4238fa786e` in
`/workspace/Rheon-ui-authoring`. The parent relayed a native 900×650/150% case
where the Transport session label wrapped internally and consumed excessive
height. This reviewer did not perform that native observation. In-progress T4
changes in the same working file are outside this review and must remain separate.
Only this report was authored; no tests, builds, native UI or workloads were run.

The small `view_mode_tabs` helper retains all four full labels and selectable
button affordances. `TextWrapMode::Extend` prevents word wrapping within one
button, while the existing outer `horizontal_wrapped` layout moves whole buttons
between rows. It preserves the authored-force disabled policy and explanatory
disabled-hover text. No font shrinking, icon substitution, units, camera/lab
dispatch or scientific semantics change. Pinned egui's existing selectable-value
helper uses the same selectable-button implementation, so keyboard/focus and
named button behavior remain on that standard widget path. One minor response
semantics recommendation was sent: preserve its clicked-and-value-changed
`mark_changed()` behavior when returning the helper's responses.

Completely inspected the one exact probe:
`viewport::header_ui_tests::responsive_mode_tabs_keep_whole_labels_at_900_650_150_percent`.
It creates a local Context, sets pixels-per-point, paints two inert SidePanels and
the mode-tab helper in a CentralPanel. It uses only a literal ViewMode and local
UI responses, iterating 1×/1.5× scale and blocked/unblocked states for two frames.
There is no Viewport/default/show, App, T4 handle, camera, lab, model constructor,
solver/evaluator, history, worker, runtime, recording, filesystem or cleanup path.
Only this individually enumerated exact test is admitted by this inspection.

The geometry checks model a 900×650 physical-size canvas with logical dimensions
divided by scale, reserve 280px setup and 64px collapsed-scene panels, and assert
whole-button horizontal containment, bounded per-button/row height, disabled
policy and remaining canvas height. They establish local egui layout conditions,
not an actual native window, OS scaling, complete viewport header with every
additional control, pointer activation or screen-reader acceptance.

The generated header-only patch contains only the tab helper, replacement of the
old mode-loop and this test module. It excludes T4 fields, handle logic and
ViewInputs changes. Read-only `git apply --check --cached` passed against the
clean HEAD index. Checking against the working file fails because that file
already contains the repair; that outcome is not a base-patch failure and no
patch was applied by the reviewer. Final packet identity and retained probe/static
evidence will be recorded below. Do not attribute a whole working-file hash to
the isolated repair while T4 changes are also present.

No blocking source finding was identified for this bounded layout fix. Native
verification on its actual committed source remains required; no T4 interaction
or force/runtime acceptance is implied.

## Final source isolation receipt; evidence pending

The response-semantics recommendation is implemented: only a click that changes
the mode assigns it and marks the response changed, matching pinned egui's
selectable-value behavior. This adds no new helper/test path.

The final header-only patch is
`/tmp/rheon-ui-header-only-reviewed3.patch`, SHA256
`901622bd12cbb09fc5729547ba3c108c85f0dd6abcfc026b9d6bc9139a998a44`.
Independently inspected the staged diff: it contains only the helper, mode-loop
replacement and test module, with no T4 field, ViewInputs or handle changes.
The staged viewport blob, `/tmp/rheon-ui-header-only.rs` and isolated
`/workspace/Rheon-ui-header-verify/src/bin/desktop/viewport.rs` have identical
SHA256 `abbaf658447dcc946fa8c51fe48d6c9597af8f80da4592b220152820042f1eb8`.
The isolated worktree differs from parent only in that file's header repair;
its whitespace check is clean. Reviewer did not stage/apply a patch or mutate
that worktree.

The earlier shared-worktree exact log reports one passing responsive-layout test,
but is not being used as isolated final gate evidence while T4 is present there.
Final isolated list/exact/static results remain pending in this receipt. Native
and subsequent synthetic-presentation harness work are separate evidence tasks.

## Final isolated evidence

Verified `../evidence/responsive-header-20261020/manifest.json`, SHA256
`dfbe770f6d49ad576b1c73910472530bdda4696e17741ddc66da9f7702757ee8`.
All six retained logs match declared byte counts/SHA256 and their original `/tmp`
bytes. The exact named probe occurs once in the isolated 183-test enumeration;
its isolated result reports one pass, zero failures/ignored and 182 filtered out.
The isolated source, temporary repair source and staged blob remain identical
at the source seal above; the reviewed patch seal also matches. This final
receipt supersedes the evidence-pending status and does not use T4 as gate evidence.

Read isolated locked-check and strict-Clippy completion logs (7.12 and 10.95
seconds). Format/whitespace logs are empty, consistent with the implementation
owner's successful-exit receipt. Reviewer performed read/hash checks only and
did not rerun any gate. No blocking source/evidence finding remains for this
separate repair checkpoint. Native verification remains pending on its actual
committed source; no synthetic harness, complete header navigation, T4 interaction,
scientific runtime or publication outcome is inferred.
