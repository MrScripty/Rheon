# UI-5 focus precision repair

Original M2a is preserved byte-for-byte in `/workspace/Rheon-ui-hierarchy`, its
65,576-byte patch SHA256
`0c8b8d4e4eb87f761ff94f998f58548888ba23aa6b40414605652044816c2a6b`
and `/workspace/scratch/rheon-ui-m2a`. This continuation uses a separate detached
worktree `/workspace/Rheon-ui-precision`, HEAD7318023 plus that exact M2a patch
(tree8740bf155b3db1f3c44e109277605b9411f459fe). No commit/publication.

## Observed fact versus inferred cause

UI-5 was first observed after the M2a diff in
`/tmp/rheon-ui-m2a-tests-4.log`. No untouched-baseline UI test was run. Source
inspection found the same formatting/conversion path in prerequisite7318023 and
pinned egui0.33.3's focus-loss reparse. A prior mechanism is inferred, not an
empirically demonstrated baseline defect. This report supersedes the stronger
"preexisting defect" wording in the preserved historical M2a reports.

Observed Air density changed without typing from1.2041288413018594 to
1.2041300000000001 kg/m³; molar mass from0.02896546 to
0.028965499999999998 kg/mol. Full canonical bits must survive focusing and
forward/reverse Tab, not merely remain within a numerical tolerance.

## Bounded repair and verification procedure

Only production file: settings.rs. Add one private adapter immediately around
DragValue addition, used by density and the existing quantity helper. Its focused
text must roundtrip the actual display value; retain the ordinary idle formatter,
IDs, parser, speeds, ranges, clamping and SI update guards. No model/solver/worker,
recording or dependency change. The shared quantity helper also serves main
domain/maximum-step/stiffness and emitter properties; the formatting reach is
broader than material fields, although those caller sources are untouched.

The new tests and selected existing tests were inspected through their complete
helper paths: local context/frame/text/pointer/key/click helpers, MaterialControls
fluid/domain forms, quantity and pure material/config defaults. They do not
construct simulations, Workers, archives or files, spawn processes, or clean up
filesystem outputs. Egui Context/input/paint runs headlessly. Runtime selection
must first be list-only, then one fully qualified `--exact` name per invocation;
stop if any helper/selection is uncertain. No broad filter/fallback suite.

New regression covers Air plus precise liquid properties, all three density units,
240/280px widths, actual forward/reverse Tab and exact-bit material preservation.
Explicit-edit checks cover density SI conversions, molar-mass g/mol conversion,
out-of-range clamping and invalid text. Existing selected thermal/selection/menu
tests use the same verified inert paths. Focused editor width must be checked,
not only idle captions. These checks cannot accept native appearance, complete
application traversal/accessibility or scientific correctness.

## Incident evidence remains preserved

The earlier broad `ui_` test-selection incident is not repeated or reclassified.
**Automatic successful-test teardown removed disposable numerical outputs before
detection.** Their removed bytes and resource use cannot now be reconstructed.
There were no CPU/RSS/disk resource measurements; the19-pass/1-ignored log's5.85s
is harness elapsed time, not scientific acceptance or a containment result.
Log SHA256 remains
`9d1b7245761f461a86dc436f21911f4a5525af3d2b8f7692d6ff8b5e6f2bbba1`.
No manual cleanup, numerical/model test, genuine cancellation, cgroup/mount
provisioning or network change is admitted for this precision continuation.

## Results and remaining gaps

Five individually enumerated/selected exact UI-only tests pass, one per command:

1. `settings::tests::ui_precision_tab_and_focus_preserve_material_bits`
2. `settings::tests::ui_precision_explicit_edits_keep_si_units_and_ranges`
3. `settings::tests::ui_material_thermal_disclosures_preserve_values_and_si_editing`
4. `settings::tests::ui_material_selection_updates_only_its_existing_owner`
5. `settings::tests::ui_material_presets_and_open_menus_fit_narrow_forms`

List-only log `/tmp/rheon-ui-precision-final-test-list-5.log` enumerated168 names;
the five are an explicit allowlist, not a runtime prefix filter. Final runtime
logs `/tmp/rheon-ui-precision-final-v3-exact-{1,2,3,4,5}.log` each record one pass.
Locked desktop check, strict desktop/test Clippy, formatting and diff whitespace
checks pass (`/tmp/rheon-ui-precision-final-{check,clippy,fmt,diff-check}.log`).
Reviewed source SHA256:
`099fa897a53feee4cb1de483655893f50e6c713c39e66c1a25c8f0549ee2c121`.

The production adapter uses round-trip Debug text only while focused (scientific
notation for tiny values); ordinary idle formatting remains unchanged. Pointer
entry adjusts initial select-all to the complete edit text through public
TextEditState, preserving subsequent caret placement. Density's unit selector
moves to the next row only when the focused value leaves insufficient room.
No persistent canonical/dirty state, replacement parser, or tolerance is added.

The deciding pre-repair exact test failed material bits against byte-identical
M2a production (`before.log`). Initial full-text geometry overflow is preserved
in `after-1.log`; the initial click-selection regression and test label-lookup
failures remain in separate earlier exact logs. Full precision lengthening exposed
a real click-to-replace regression: the compact button selected fewer characters
than the new editor. It was repaired and the original no-Ctrl+A thermal regression
passes unchanged; the new density edit test also uses ordinary click replacement.
Exact-bit forward/reverse/pointer focus checks include tiny positive viscosity,
both supported narrow form widths and all three density units. Explicit edits
retain SI conversion, ArrowUp, out-of-range clamps and invalid-text refusal.

Independent [source review](ui5-precision-review.md) supplies its own scoped
disposition; no reviewer runtime tests are needed. Preserve every deciding failure
and final log separately; no log was overwritten to hide a failure. Original
M2a report/review identities remain historical, not rewritten to imply acceptance
of the precision repair. Native focused-field appearance, full application focus,
assistive technology, representative usability and M1/A4/A5 remain unaccepted.
