# T3 viewport selection admission

Base6127cecbdcfe53270347e59ebe86ff3317528cfe on PR146. Preserve that exact commit,
evidence and native handoff. Same sole UI owner and normal-push branch.

Write set: new desktop/scene_view.rs pure authoring geometry/projected hit tests;
viewport.rs composition of scene interaction/paint plus small aggregate view-input
API; main composition passes the existing SceneUi by mutable reference. SceneUi
selection API remains canonical; optional force type label/help only. No core
force/evaluator/solver/recording/runtime or model import change.

Force markers display canonical translation/rotation/support, with fixed-size
visual directional/radial/vortex glyphs. Marker size does not claim physical force
magnitude; Global anchors do not imply bounded physical support. Sphere/Box wire
support uses actual metres and local transform. Disabled objects remain visible,
muted and selectable. Invalid drafts refuse force preview with visible feedback;
never silently normalize malformed transforms or infer historical forces.

The one accepted model uses actual triangle geometry, not a bounding-box pick.
Selected outline is thin screen-space silhouette/boundary edges, not every mesh
triangulation edge. Force wire/glyph selection likewise follows rendered geometry.
Bound projection to256 forces and12000 model triangles; reject unsupported display
instead of silently reducing scene selection. Display coordinates outside ±1e12 domain lengths explicitly refuse (no physical
value changes). Check finite inputs, near-plane
clipping, degenerate projected triangles/segments, perspective/orthographic depth,
viewport bounds and deterministic overlap/tie behavior.

Plain primary click chooses closest authoring stroke/marker (then depth/stable ID),
otherwise nearest model surface. Markers are explicitly x-ray authoring overlays,
selectable through fluid/model; models are surface-picked. Empty click clears
selection. Camera drags and modified/middle gestures do not select. Never replay
App.update or initialize a numerical lab to test it. Tree and viewport both call
SceneUi.select; missing IDs clear and context changes keep their established rules.
Retained view may select actual frozen model geometry read-only; it never paints
current draft forces. Existing particle probes keep their separate identity/mode.

T4 follows with force translation/rotation/support handles. Its captured gesture
must precede picking/camera and commit one validated draft edit on release, with
Escape/focus/context/identity loss cancelling exactly. This slice does not claim
handles or authored runtime. New force runs remain refused.

Evidence: inspect full geometry/UI helper paths, list-only enumerate, then individually
selected --exact geometry/UI tests using literal arrays/force documents, camera and
local Context only. No CollisionMesh construction/import, numerical/model/Worker,
history/recording or filesystem fixture/cleanup. Compile/Clippy/fmt and independent
source review before commit/normal push. Native current6127 validation remains
separate; new revision needs its own render/pick/outline evidence and quality85JPEGs.

Independent review conditions resolved in source: invalid/nonrepresentable projection
has a distinct unavailable miss and keeps selection; valid hits still select.
Exact-position welding removes duplicate-index seams, and canonical edge/opposite
vertex projection makes two-sided outlines independent of triangle winding.
Non-manifold edges explicitly refuse the outline (surface picking remains), and
these contours are x-ray silhouette/boundary overlays, including self-occluded edges;
this is not a depth-buffer outer contour claim. Zero strengths have only neutral
anchors/support. Signed radial arrows and vortex direction follow canonical sign.
