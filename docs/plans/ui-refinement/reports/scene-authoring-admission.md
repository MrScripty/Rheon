# Next bounded slice: canonical scene authoring

Owner: desktop UI task, isolated implementation/ui-scene-integration-20261020.
Precede this slice with distinct preservation/precision/timeline/backend commits;
no squash of original M2a or imported backend evidence. No numerical/model tests.

## Write set

- New src/bin/desktop/scene_ui.rs: one canonical editable ForceScene, checked
  SceneObjectId allocator shared with the one accepted model, one scoped selection,
  Draft/Retained-run context, right scene list and selected properties.
- src/bin/rheon-desktop.rs: App owns SceneUi; compose its right panel; synchronize
  transactional model replacement/removal (restore previous accepted mesh/ID and
  preserve history if fresh ID admission fails; include initial CLI admission) and successful run-context changes;
  select draft versus immutable retained geometry consistently; refuse new draft
  launches containing any force object before path/recording integration.
- Exact UI-only tests in scene_ui.rs or desktop/scene_ui_tests.rs. No viewport,
  solver/core recording/run_sets changes in this first authoring slice.

## Behavior and identities

Add force menu creates/selects Directional, Radial or Vortex canonical objects.
Use supplied IDs, units, metres/quaternion transforms, support/falloff and [start,end)
seconds. No second force schema/evaluator. Properties edit the canonical authoring
scene, report backend freeze errors, and never mutate a retained run. Invalid draft
is visible and cannot launch. Existing gravity stays additive; communicate that
without silently changing it. Default new strengths are zero to avoid unintended
extra gravity. Version1 excludes parenting/scale/animated transforms.

Selection is App-owned SceneObjectId scoped by Draft versus Retained-run context;
future viewport picks/handles call this same selection owner. Tree collapse keeps
selection. Remove/replacement clears the old selected model/force. IDs use checked
nonzero monotonic allocation across model+forces; exhaustion refuses, never wraps.
Only one accepted collision model exists. Its first properties are read-only
accepted geometry; existing transactional import/placement remains in setup.
Label the collection Authored forces; existing gravity/configured-only forms
remain separate real physics, not proof of all/no forces. No save/load interface is added here; its later admission must enforce the
whole-scene imported-ID collision/exhaustion policy already in the contract.

A retained run reads actual immutable Request geometry. Its old force metadata is
unavailable, not reconstructed from the current draft or declared absent physics.
Switching context clears selection and stops hidden playback, preserves retained
history/index, and never overlays draft geometry onto historical fluid. New forces
are disabled while retained context is selected or an active worker owns inputs.

At narrow width (<1100 logical px) opening Scene replaces the left setup drawer
with a top run/status/Cancel strip using the same existing owner/actions. Closing
Scene restores setup values. At wider sizes both side panels are available.
The right panel's collapse control stays named and keyboard accessible; its form
scrolls independently. Scientific properties have unit-labelled controls and
optional details. Essential validation/error/unsupported status stays visible.

## Runtime gate and next slices

Any nonempty draft document, including disabled objects, visibly refuses new
simulation/configured launch before output/Worker creation. Legacy frozen reruns
retain their actual recorded intent; they do not consume the current draft.
No authored Request snapshot or force-aware dispatch is claimed in this slice.

Next bounded slices wire actual viewport picking/thin geometry outline, then
transform handles/numeric alternatives and exclusive gesture capture on this
same selection owner. Runtime/archive propagation is separately admitted after
its versioned intent and qualified backend recording contract are integrated.

## Evidence

Inspect complete helper paths, list-only enumerate, then one fully qualified
--exact UI test per command. Tests may use literal force documents, mesh summaries,
local Context/input/shapes and freeze/validation only. No imports, simulation,
Worker, history capture, filesystem fixture/cleanup, model or external process.
Cover add/select/remove, canonical IDs and exhaustion, property units/validation,
context/collapse retention and launch-refusal policy. Focused locked compile,
strict Clippy/fmt and independent source review precede commit/publication.
Native validator later checks actual widths/scales/keyboard forms and exports
quality85 JPEGs of exact updated source; no native result is inferred here.
