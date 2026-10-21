# T4 force manipulation admission

Sequential continuation of T3 on PR146; same sole desktop source owner. No core,
solver, recording, Request or run-set edits. Files: scene_ui.rs transactional draft
edit, new scene_handles.rs geometry/capture; scene_view.rs helpers/composition;
viewport.rs ownership before camera; main supplies current editability. Literal
UI/geometry tests only, inspected then list-only then individual exact names.

Use the canonical ForceObject/ForceTransform/ForceSupport. One SceneUi selection.
The canonical draft is unchanged while dragging: keep one validated candidate
and exact base object in a SceneUi edit transaction. Display candidate as an
explicit preview; release commits once only if base/selection/context are still
current. Escape, focus loss, pointer loss, identity/context/geometry change cancel.
Freeze before accepting candidate; never normalize a malformed stored quaternion,
clamp support dimensions silently, or change run snapshots. Properties/add/remove
are disabled during capture. Numeric properties remain the keyboard alternative.

Named Translate / Rotate / Support controls apply to the selected Draft force
only, idle worker only. Models remain read-only until an accepted model-placement
transaction bridge exists. Retained context shows no current draft glyph/handles.
Translation axes are world XYZ; rotation and support axes are local XYZ. Colour
is accompanied by X/Y/Z labels and hover instructions. Global has no size handle.
Dragging a Global Directional anchor only changes authoring placement, not the
uniform field; explain this in contextual help. All physical distances remain m.

Projection shares the existing camera/near/viewport finite checks. Begin on an
unmodified primary press over a visible selected handle before picking/camera.
Capture exclusively owns pointer until release, including outside canvas. Escape
cancels and latches ownership until release; do not hand an ongoing drag to camera.
Other buttons/modifiers begin camera normally; changing modifiers during captured
gesture does not retarget its axes. Wheel/Home/zoom ignored while captured. Focus
canvas on begin; never steal keyboard focus without a click. Cancel when window
focus, canvas rect/camera/domain, selection/context or source object changes.

Translation uses closest ray/axis parameters; near-parallel rays refuse admission.
Rotation uses ray/plane vectors about the chosen local axis, atan2 signed angle,
then world-axis unit quaternion multiplication against exact original orientation.
Near-parallel/central-ray degeneracy refuses. Support-size uses local-axis ray
displacement on original positive radius/half-extent; invalid candidates remain
uncommitted and visibly refused. Sphere uses its radius; Box uses each half-extent.
No arbitrary strength, density or force evaluation is performed by handles.

Acceptance: pure independent axis/ray/quaternion cases plus actual local egui
pointer press/drag/release, Escape latch, modified-button admission, camera-owner
return, focus/pointer/context/identity/rect loss, invalid support rejection and
snapshot/base bit preservation. Compile/Clippy/fmt, independent source review;
then native exact-revision high-DPI hit/contrast/focus tests by existing validator.
No native/assistive usability, model manipulation, runtime/archive or scientific
acceptance is implied by headless Context tests. Rollback only this distinct
increment, preserving T3/6127 evidence and unrelated M2a branches.

Independent review conditions: current-pointer validity is separate from last
validated preview; valid move then invalid move then release cancels, never commits
stale preview. Every cancellation reason latches ownership until button release.
The terminal release/cancel frame remains consumed even after transaction removal.
A hit but geometrically unsolvable handle visibly refuses and captures/latches,
never silently begins camera orbit. Quaternion evidence uses nonidentity base and
local axis to distinguish multiplication order.
