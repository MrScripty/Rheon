# Independent T3 viewport selection source review

2026-10-20. Reviewed the uncommitted T3 increment in
`/workspace/Rheon-ui-authoring` against `viewport-selection-admission.md` and its
independent plan review, preserving parent
`6127cecbdcfe53270347e59ebe86ff3317528cfe`. This report is the reviewer's only
write. No tests, builds, native interaction, model import/construction, numerical
or evaluator workload, runtime/history/recording, filesystem fixture or cleanup
was executed by the reviewer.

## Complete inert boundary and exact admission

Read complete `scene_view.rs`, SceneUi's test-only `geometry_fixture`, the
Viewport/ViewInputs composition, main bridge and shared OrbitCamera paths.
The following five fully qualified names are the inspected exact probe boundary:

- `scene_view::tests::geometry_depth_near_clip_and_degenerate_hits`
- `scene_view::tests::geometry_two_sided_outline_welds_exact_seams_and_refuses_nonmanifold`
- `scene_view::tests::geometry_force_glyphs_preserve_zero_direction_sign_and_support`
- `scene_view::tests::scene_pick_identity_overlap_retained_and_unavailable_preview`
- `scene_view::tests::scene_pointer_selects_same_identity_ignores_drag_and_modified_click`

Tests construct literal vertex/index arrays, force documents, projection points,
camera fields and a local egui Context. `geometry_fixture` stores those values
with a test-only model identity; it neither constructs CollisionMesh nor imports
geometry, allocates a worker, loads fields or performs production ID admission.
Projection calls ForceScene::freeze for validation/deep-copy only, never force
sampling/acceleration. Glyph construction is schematic UI geometry, not field
evaluation. Camera trigonometry, clipping, interpolation, BTreeMap topology,
shape painting and hit tests are local geometry arithmetic. The pointer test
allocates one egui response and calls only scene_view interact/paint. No App
default/update, viewport show, lab initialization, solver, history, Inspector,
recording, process, file or cleanup path is reached. Later added assertions on
these same literal paths do not expand the workload boundary. No broad test
filter is admitted by this source review.

## Canonical selection and rendering semantics

SceneUi remains the one selection/context owner. The main bridge supplies that
same mutable owner through ViewInputs, along with already selected frame/emitter/
worker/Inspector projections. It does not copy canonical force properties or
change Request/solver/recording/run-set ownership. Appearance picking invokes
SceneUi.select; retained context excludes current draft forces and uses the
existing actual frozen model identity/geometry. Particle probes stay in their
separate mode. A supported authoring hit opens the existing properties drawer.

The helper converts metres to domain-normalized display coordinates once per
projected point, applies the canonical quaternion without normalization and
validates the complete draft before force projection. Sphere/box wires use actual
support dimensions and transforms. Global forces receive an anchor/glyph without
inventing bounded support. Glyph extent is a fixed authoring scale, unrelated to
strength or density; its projection changes with camera. Zero directional/radial/
vortex values have only neutral anchor/support. Nonzero directional orientation,
radial attraction/repulsion and right-handed local-Y vortex sign are preserved;
disabled objects remain muted/selectable. No authored force is executed.

Projected support/glyph strokes are explicit x-ray overlays. Picks prioritize
closest stroke/anchor, then perspective-correct depth and stable ID; only without
an overlay hit do actual model triangle surfaces compete by nearest depth.
Unmodified primary clicks select or clear established misses. Dragged and modified
primary responses do not select. Middle/wheel navigation retains its camera
owner; manipulator capture/transactions are a later slice, not implemented here.
Camera input is now applied before picking and painting, so both use the same
current camera rather than picking old geometry then painting a changed view.

## Geometry and topology review

Near-plane clipping uses the existing camera convention with a small positive
margin. Screen-line clipping bounds strokes before tessellation. Stroke/triangle
depth interpolation is reciprocal in perspective and affine in orthographic
projection. Degenerate projected triangles do not produce surface hits. The
model uses its triangle inventory, not a bounding-box proxy; there is no silent
subsampling of selectable surfaces.

Exact-position keys weld duplicate-index seams, including equivalent signed
zeros, without welding nearby physical vertices. Each shared edge is given one
canonical endpoint order; the opposite vertex's view-side sign is independent
of triangle winding. Opposite signs remove internal coplanar diagonals; boundary
and same-side contour edges remain. This supports open/disconnected/two-sided
input without treating its original winding as an outward-solid convention.
More than two incident faces explicitly refuses the outline while retaining
complete surface picking. The resulting thin contour is a silhouette/boundary
x-ray overlay and may include self-occluded edges; it is not a depth-buffer-only
outer-contour claim. Named tree/properties selection remains the complementary
non-colour signal.

Work is bounded by backend validation to 256 forces and by display admission to
12,000 model triangles. A display-only normalized-coordinate limit of 1e12 now
refuses extreme projection explicitly; it never clamps or edits physical values.
Malformed/absent/over-budget geometry marks picking incomplete. Valid hits can
still select independently supported objects; uncertain misses preserve the
selection needed for correction.

## Findings and repairs

1. **Repaired:** Original viewport composition picked before camera input but
   painted after it. Appearance input now precedes interaction and painting.
2. **Repaired:** An anchor outside the viewport could still be picked within
   seven pixels although painting suppressed it. Picking now uses the same
   centre-containment policy as painting; a literal edge case is included.
3. **Repaired:** Any warning originally made a miss unavailable, so an outline-only
   non-manifold warning prevented clearing selection despite complete surface
   projection. Dedicated `pick_incomplete` distinguishes that case from actual
   invalid/unavailable projection, with an established-miss assertion.
4. **Repaired:** A finite but enormous coordinate could overflow screen projection
   without warning and clear selection. The explicit display-coordinate refusal
   covers the reported orthographic zero-Global example without touching its
   canonical draft; an added literal assertion covers the extreme point.
5. **Repaired:** A hardcoded warning y=70 could overlap the width-wrapped viewport
   and mesh captions. PaintArea now carries the measured bottom of that stack;
   standalone tests use a Rect-derived area and do not claim integration layout
   evidence for that stack.
6. **Repaired:** Nonfinite camera fields originally resembled ordinary offscreen
   geometry and could clear selection on a miss. Projection now validates finite,
   positive bounded canvas dimensions, finite camera distance/basis and unit basis
   axes before any geometry. Failure produces visible unavailable feedback and
   incomplete picking; legitimate behind-near/offscreen misses remain distinct.
   A NaN-yaw literal assertion extends the existing inspected exact test.

No native evidence exists for this new increment. Initial/final implementation
test logs, source seals and any final correction are to be recorded below before
final source disposition. Native hit-target usability, overlap/contrast, high-DPI
focus, model contour appearance, retained geometry and assistive access remain
separate requirements. Core force/runtime/scientific, archive provenance and
future handles are outside T3 acceptance.

## Final disposition and sealed evidence

No blocking source finding remains for the bounded T3 increment after the six
corrections above. This supports a source/focused-geometry checkpoint, not native
or full application interaction acceptance. The late camera/canvas assertion
remains on the same literal Projection helper path; no additional workload path
was introduced. The property drawer and SceneUi selection owner are reused;
there is no competing authoring state or new runtime dispatch.

Independently read and hash-verified
`../evidence/viewport-selection-20261020/manifest.json`, SHA256
`7dbed249419d616ae97017b3db272d2e1d10a0a3023f46ba1e463b5b536799cd`.
All four declared source seals match the current files. All ten retained logs
match declared sizes/SHA256 and their original `/tmp` bytes. The retained log
attribute disables text conversion. The five distinct exact test identities
match the list-only 182-test enumeration once each and each final result reports
one pass, zero failures/ignored and 181 filtered out. These logs were read, not
rerun by the reviewer.

The final five probes cover depth/near clipping and degeneracy, winding/seam/
non-manifold policy, zero and signed/transformed glyphs/support, triangle-vs-box
misses/overlap/disabled/retained/unavailable identity, off-canvas anchor refusal,
and real egui click/modified-click/drag-to-selection behavior. The malformed
camera and extreme orthographic coordinate assertions are included in the final
accepted identity probe. They are literal display geometry, not scientific
fixtures or force evaluator tests.

The retained accepted check and Clippy logs complete successfully in 1.28 and
2.76 seconds respectively. Format/diff logs are empty, consistent with the
implementation owner's successful-exit receipt; empty output alone does not
encode an exit status. Reviewer independently ran read/hash verification and
`git diff --check`, which was clean, and executed no test or build.

| Final source | SHA256 |
| --- | --- |
| `src/bin/desktop/scene_view.rs` | `d1ea804559495fe869b143383734afb953b0462e06b12e176840632ac003cb05` |
| `src/bin/desktop/scene_ui.rs` | `0dd573b5bb2d7d9af3249a15ee768d5a39ce3381db010d04bccb7f6cee954d40` |
| `src/bin/desktop/viewport.rs` | `c4fbdf394accc0b831460a3a1e75d78ee0dea900043bc82b2543619de4aa3a57` |
| `src/bin/rheon-desktop.rs` | `af4c6ed921b2fde0a78d5bc2ebeb45f1a8f939b4947de82067e5f4278a537bbb` |

The pure pointer fixture calls interact/paint directly; it does not exercise
actual Viewport camera-input composition, particle-probe switching, real mesh
ownership or main's retained-request bridge. Those paths are source-inspected
only. Native current-revision evidence is still needed for target usability,
thin-outline contrast, warning-stack geometry, mixed camera gestures, both
window sizes/scales and accessibility. The topology probe is literal and does not
constitute actual imported-model interaction. No T4 handles/capture transaction,
authored runtime, archive custody, numerical acceptance or publication result is
implied. No production source or canonical plan was edited by this reviewer.
