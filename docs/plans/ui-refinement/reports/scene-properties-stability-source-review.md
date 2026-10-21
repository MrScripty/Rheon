# Independent Scene properties stability source review

Disposition: the minimal source repair and focused local egui evidence are acceptable for this checkpoint; no material source blocker found. This review does not accept native repaired behavior.

## Scope and cause

The production change is limited to the two Draft property-editor call sites and one shared `draft_properties` helper in `src/bin/desktop/scene_ui.rs`. Normal editing and a disabled drag preview now render the same object under one explicit persistent per-object UI ID. The branches are mutually exclusive, so the shared ID does not introduce duplicate simultaneous editors. Retained properties keep their separate existing identity.

Before the repair, the normal editor used a `force-properties` per-object `push_id`, while the preview omitted that scope. This is a concrete source-level disclosure identity mismatch. Pinned egui 0.33.3 additionally seeds auto widget IDs from both the stable child ID and the parent's current auto-ID salt (`Ui::new_child`, ui.rs306–314). An ordinary `push_id` therefore does not by itself isolate numeric editor IDs from an inserted preceding error row. `UiBuilder::id` uses global scope and independently seeds the child IDs; its documented purpose is preserving widget state across UI-tree moves. The helper derives that explicit ID from the persistent parent/object identity and uses the same nested enabled-editor structure in both Draft branches.

The local before tests reproduced the disappearance of a visible `Position X` field and loss of the radius editor's focus after invalid-radius feedback at width 900. The disclosure failure's generic helper panic does not identify the exact phase by itself. The source mismatch and auto-ID mechanism explain the targeted repair; no native event trace proves the native focus symptom's precise cause.

## Invariants and helper admission

`force_properties`, the shared precise quantity editor, units, numeric parsing, domain validation/error copy, canonical selection, launch policy and transaction commit/cancel code are unchanged. Preview still edits only its temporary candidate; a valid release revalidates and replaces canonical data through the existing transaction. Invalid radius remains visible as an error and refuses freezing; the repair does not clamp invalid values or suppress feedback. Prior frozen snapshots retain their owned data.

Complete inspected paths: `sphere_scene`, `SceneUi::default/add/show`, `context/frame/text_rect/click/key/replace`, `draft_properties/force_properties/quantity`, `settings::quantity/precise_drag`, and `begin_edit/preview_edit/finish_edit/freeze_for_run` through `ForceScene::freeze` and `ForceObject::validate`. They only construct/validate/clone canonical metadata and process/paint egui UI. They do not construct App/Worker, call acceleration/evaluate, perform simulation/model work, recording, filesystem operations or cleanup.

Only these individually exact test paths were admitted after list enumeration:

- `scene_ui::tests::scene_properties_disclosure_survives_preview_apply_and_cancel`
- `scene_ui::tests::scene_invalid_radius_can_be_corrected_without_refocus`
- Existing affected regression: `scene_ui::tests::scene_pointer_add_select_edit_collapse_and_retained_context` (separately re-inspected and admitted).

The first checks both Transform and Support disclosures through multiple preview frames and cancel/apply, canonical-draft isolation, applied values and a previously frozen support snapshot. The second uses actual pointer/keyboard UI at widths 900 and 1250, invalid radius feedback, focus identity, correction without another click, restored validity and frozen-snapshot isolation. The existing regression also exercises Add/name/units/strength/enabled/hide/reopen/context inputs; its launch-readiness call only reads the Pending policy and never dispatches work.

## Evidence boundary

Read the preserved baseline logs `/tmp/rheon-ui-scene-stability-disclosure-before.log` and `/tmp/rheon-ui-scene-stability-focus-before.log`: each selected exactly one test, failed once, and filtered 205 others. No reviewer test/build/native execution occurred.

These egui tests use 820 logical pixels of height and do not establish the native 900×650 at 150% layout, actual viewport-handle pointer capture, screen-reader acceptance, or general focus behavior for every control/object/support. Parent's earlier native representative Radial/tree/picking/outline/X Move/Rotate/Sphere Size/Escape/camera isolation/close evidence is limited to those observed interactions on its exact earlier revision. The present repair still requires native verification; no broader runtime/scientific qualification is inferred. Production qualification remains Pending.

## Final receipt

Final reviewed source/test file SHA256: `46eeaeaf3cce71c7665cde4224ef2ed7e62100047796414c94d07e02dd4b7390`.

Read the before/after enumeration logs: each lists 206 tests and each of the three admitted names exactly once. Each accepted execution log selects one test, reports one pass and 205 filtered tests, with no ignored tests executed. The new disclosure and focus regressions pass in 0.04 s each; the existing authoring interaction regression passes in 0.10 s. Both baseline failures remain preserved and are distinguished above from the accepted repaired runs.

Owner's locked desktop check completes successfully in 1.21 s and strict desktop/test Clippy completes successfully in 6.84 s. Formatting and diff-check logs are empty; owner reports exit 0. These compile/static checks do not execute numerical cases. Reviewer read the source and evidence only and performed no test/build execution.

| Evidence `/tmp/rheon-ui-scene-stability-…` | SHA256 |
|---|---|
| `list-before.log` | `9918d22dadd297444ca45eb2b2fbab55ee7a0d8747b64129e1d6374b8db6c877` |
| `disclosure-before.log` | `d7782a15a521fbcfbe400e3538d0c1ab6c6d496f9fb8d1583937a28e5dc89d61` |
| `focus-before.log` | `59c843f1d1298f8f64c8036542d7d5e29d572899c8c6f23bee3cb4f6433af09d` |
| `list-after.log` | `37f111fa1e4e49623b13d3d31943161d2bd4fa15af48c381c47489ca9aeb27b1` |
| `disclosure-after.log` | `7920e6fcf08b502ba9a6d3ad8730101200a8384da47ed764fda2ab8a3f289f8e` |
| `focus-after.log` | `a025239c1e7e2565425f5603b5bd2620af8d9abc0e2193ffd2837b3a5293b2e2` |
| `authoring-regression.log` | `291acbf05100c55ed609432f4874ff88694e68c4eb62469e44653866895f67aa` |
| `check.log` | `5adfe47eb2a9b5b9ef51e4c77d25ade237255a8d7e514d584469a5c7619dee43` |
| `clippy.log` | `8c0e6e0907e6712eabdfcf565baacf868b526e4e3c55103788e4939398bc2251` |
| `fmt.log` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `diff.log` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

No production path beyond the property UI identity scope changed. Native repair acceptance remains pending; authored-force qualification remains Pending.
