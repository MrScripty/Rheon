# Independent T4 force-handle plan review

## Disposition

Conditionally suitable for the next sequential production slice. The canonical
`ForceObject` transaction, force-only scope, explicit unsupported model handling
and independent gesture ownership are proportionate. Resolve the conditions
below in the admission/canonical plan before implementation; verify them in the
source review and exact UI tests before calling T4 implemented. No new runtime,
solver, recording or model-placement work is required by this review.

## Reviewed evidence and limits

- Worktree: `/workspace/Rheon-ui-authoring`; HEAD
  `6127cecbdcfe53270347e59ebe86ff3317528cfe`.
- T3 source was locally modified and uncommitted at review. T4 depends on its
  distinct reviewed checkpoint, rather than treating the current tree as a
  committed T3 base.
- Admission: `reports/force-handles-admission.md`, SHA-256
  `f15cbb62e1886d4d2dc04ff2a4321b9cf700a1dc2a034fb06a86ad7e9a260d4b`.
- Canonical plan: `plan.md`, SHA-256
  `ba9fddff4fee2982af4098ad4fcee07f8081f69746f48f87fcd552068dcbf962`.
- Read the canonical force schema/validation, `scene_ui.rs` ownership,
  `scene_view.rs` selection/projection, `viewport.rs` input routing and
  `camera.rs` projection. This is a source/design review, with no builds, tests,
  application interaction, screenshots or scientific execution.
- Existing standards source review applies; this report is not Standards Engine
  certification. Native focus, high-DPI hit targets and contrast remain for the
  exact-revision native validator.

## Required design conditions

| Priority | Finding | Required disposition and evidence |
| --- | --- | --- |
| High | A last-valid preview alone cannot represent an invalid current pointer position. A valid size move followed by crossing zero or ray degeneracy could otherwise commit an obsolete preview on release. | Keep current-position validity separately from the candidate. Invalid release rejects/discards the transaction; canonical draft retains its exact base. Test valid move → invalid move → release, including support and degenerate geometry. |
| High | Gesture cancellation/commit clears transaction state, but the same frame can still reach T3 click selection or camera input. Escape alone is not the only cancellation route. | Return explicit per-frame input consumption. Every cancellation route retains ownership until primary release, and terminal commit/cancel/release consumes that frame. Test Escape, focus/pointer loss, context/identity/rect/camera/domain changes and no fall-through selection or camera movement. |
| Medium | The canonical milestone still names `scene_manipulator.rs` and a `collision_ui` model-placement bridge; the new admission is narrower. | After the separate T3 checkpoint, make the canonical next slice the exact force-only write set: `scene_handles.rs`, canonical transaction in `scene_ui.rs`, projection/composition helpers, viewport routing and main editability. Defer model movement/Apply explicitly; do not touch the bridge in this slice. |
| Medium | A visible handle can be hit even when its axis/plane is geometrically unsolvable. Falling through to orbit would contradict the visible manipulation affordance. | Distinguish handle hit from solvable capture. A hit with near-parallel/central-ray refusal reports concise feedback and consumes/latches that attempt. A genuine miss and initially modified/nonprimary press retain ordinary camera behavior. |

These conditions were sent to the implementation owner before production T4
edits. They are design requirements, not observed defects in an implementation
that has not yet been written.

## Transaction and geometry contract

`SceneUi` remains the single owner of draft, selection and edit transaction. Keep
the original canonical object and a candidate, without writing the draft during
drag. On release recheck Draft context, idle worker, selected identity, unchanged
source object and captured geometry. Freeze the candidate in the containing
scene before the one canonical replacement. On conflicts, discard the preview;
never restore an old base over a newer external edit or selection. Unrelated
objects, IDs, kind, units, activation times, enabled state and snapshot remain
unchanged. Numeric properties, add/remove and mode changes cannot mutate a
captured transaction behind the handle.

Derive every update from the original object and initial ray parameter/vector,
rather than accumulating frame deltas. Invert the actual perspective and
orthographic camera projection; convert normalized domain coordinates to metres
exactly once. Require finite values, valid domain/rect/camera and forward/near
intersection. Translation uses world axes. Rotation and support use the base
orientation's local axes; apply the world-axis delta quaternion to the original
orientation in the correct order. Do not repair malformed stored orientations or
silently clamp invalid dimensions. Global support has no size handle; sphere
radius and box half-extents retain their canonical meanings and SI units.

Capture starts only on a plain primary press over a selected editable force
handle, before camera input and object picking. Keep it while held outside the
canvas. Focus/pointer loss is cancellation, never synthetic release acceptance.
Do not steal focus without that initiating click. Block wheel, zoom, rotation
events and camera keys while owned; mid-drag modifiers do not change mode/axis.
Models, Retained context and active workers do not offer editable handles. Named
modes/axes and numeric properties remain available for discoverability and a
keyboard alternative. The Global Directional placement limitation must remain
in contextual help.

## Proportionate acceptance evidence

Inspect test bodies/helper calls before list-only enumeration, then invoke only
individual fully qualified exact tests. Use literal geometry and local egui
contexts; no Worker, solver/evaluator, runtime fixtures or imported model files.

- Independent ray/axis and ray/plane cases in perspective and orthographic
  projection, with finite/near/parallel rejection and positive SI sizes.
- A nonidentity original quaternion and nontrivial local axis/angle to distinguish
  quaternion multiplication order; original-based repeat updates to expose drift.
- Real press/move/release showing canonical draft unchanged through preview and
  changed once on valid release; invalid-current release leaves base bits intact.
- Terminal-frame consumption and until-release cancellation latch, including
  modified/nonprimary admission, outside-canvas continuation, focus/pointer loss,
  stale identity/context/source and camera/rect/domain change. Ordinary camera
  navigation resumes only for a later independent gesture.
- Draft/Retained/worker/model guard cases; immutable snapshot and unrelated field
  preservation. These checks exercise metadata validation, not field evaluation.

Compile/lint/format and an independent source review remain implementation gates.
Headless evidence cannot close native usability acceptance. Roll back only the
distinct T4 increment; preserve T3, `6127` native evidence and unrelated branches.
