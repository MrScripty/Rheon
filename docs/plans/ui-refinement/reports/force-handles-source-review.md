# Independent T4 force-handle source review

2026-10-20. Review in `/workspace/Rheon-ui-authoring`, preserving separately
committed responsive-header parent `8504b3e7c9e6a02a6e2521b54c1aefc9c0f58ec2`
and T3 evidence. Scope is `scene_handles.rs`, the canonical SceneUi transaction,
shared scene geometry/composition, viewport capture ownership and main editability.
Reviewed against force-handles admission and independent plan conditions. Only
this report is authored; no tests, builds, native UI, model/solver/evaluator,
runtime/history/recording, process, filesystem fixture or cleanup was executed
by the reviewer.

## Complete inert test boundary

Inspected these six full `scene_handles::tests` paths and all called helpers:

- `handles_geometry_ray_translation_units_and_nonidentity_local_rotation`
- `handles_transaction_invalid_final_position_never_commits_stale_preview`
- `handles_pointer_release_commits_once_and_escape_consumes_until_release`
- `handles_pointer_invalid_support_release_restores_base_and_snapshot`
- `handles_cancellation_context_focus_pointer_and_camera_loss_never_hands_off`
- `handles_named_controls_fit_narrow_without_touching_authoring_values`

The fixture constructs literal canonical ForceObjects/ForceScenes, a value-only
SceneUi test fixture and OrbitCamera. The harness creates a local Context and
canvas response, calls Handles.update/paint/controls directly and inspects its
state. It never constructs Viewport/App, calls their update/input/show, initializes
a lab, imports/constructs CollisionMesh, samples a field, creates a numerical
history, worker, recording or output file. Freeze is validation and immutable
deep-copy only. In-memory Serde JSON encoding for source comparison does not
perform IO. Ray/axis/plane/quaternion arithmetic is UI geometry, not numerical
simulation evidence. The narrow-controls helper paints only the toolbar and
asserts authoring values remain unchanged.

The five previously inspected scene_ui and five scene_view exact probes remain
within their inert boundaries. The extended scene-invalid-support test only
adds literal draft mutations, preview/release/freeze and signed-zero source
comparison. No broad suite/filter or uninspected helper is admitted. The intended
runtime selection is six handle plus those ten fully qualified names, each
list-enumerated and run individually with `--exact` by the implementation owner.

## Canonical ownership and geometry

SceneUi owns the draft, selected ID and optional transaction. The transaction
retains the exact source object, a validated last preview and separate current
validity; invalid current input cannot commit a stale valid preview. Canonical
values stay unchanged during drag. The display is a temporary Cow projection,
not a second mutable force owner or an updated run snapshot. Release checks
selection/context and source equality, including signed-zero distinction through
bounded in-memory JSON bytes. Final replacement revalidates the candidate in the
current containing document, preserving newer unrelated edits on refusal.

Translation uses world XYZ and metres; rotation/support use the base orientation's
local axes. Each candidate derives from the exact original object and original
ray parameter/vector, avoiding accumulated frame deltas. Ray inversion matches
the camera's orthographic/perspective scale. The closest-axis and ray-plane
formulae use normalized directions and reject parallel/central degeneracy.
World-axis delta quaternion multiplication against a nonidentity base preserves
the intended local-axis edit order; the literal rotated-box resize exercises the
same local support convention. Radius and selected box half-extent are edited
without clamping invalid sizes. Global has no size handle; its uniform-directional
anchor limitation remains in help. Models and retained runs are not manipulated.

Viewport gives a capture attempt first ownership, then ordinary camera input and
T3 picking only if the frame is unconsumed. Capture/latch forces Appearance and
disables independent view modes. Escape and lost focus/pointer/context/source/
camera/canvas/domain cancel without rolling an old base over newer data. Every
terminal/cancel frame stays consumed; explicit primary release clears the latch.
Wheel/Home/zoom and object picks cannot receive that owned frame. Numeric
properties remain the keyboard alternative and are disabled during an active edit;
add/remove is likewise disabled. Main passes idle-worker editability, while
selected_force limits candidates to the Draft context and canonical force IDs.

## Findings tracked during source review

1. **Repaired:** Hit admission originally ran before the same validation that
   suppressed painting, allowing invisible invalid-draft handles to capture.
   It now checks the validated display scene before a new hit.
2. **Repaired:** Coalesced primary press/release could create an orphaned capture.
   It now consumes that click frame, preserves exact base and finishes without
   leaving a drag/latch. A literal same-frame event assertion was added.
3. **Repaired:** Release originally trusted cached validity. It now freezes the
   preview inserted into the current full scene before canonical replacement;
   pure assertions cover an unrelated invalid object at release.
4. **Repaired:** Vector addition during world translation rewrote untouched signed
   zeros. Translation now edits only the selected component and returns the exact
   base for zero delta; zero-angle rotation likewise returns base. Literal bit
   assertions and a signed-zero source-conflict guard cover these cases.
5. **Repaired:** A tree/context change before viewport update could briefly project
   the old transaction under a different selected row. Cow and Properties now
   substitute the preview only while edit_matches is true; the new selected
   canonical object remains authoritative while Handles cancels/latches its
   captured gesture. A literal stale-selection display assertion was added.
6. **Repaired:** Solve admission originally rejected only intersections behind
   the ray origin. Ray now carries the camera-forward near-plane parameter;
   axis and plane solves reject forward intersections before that threshold.
   Positive-but-before-near literal cases extend the existing geometry test.
7. **Repaired:** Axis labels at endpoint+(7,-7) could disappear at top/right
   canvas boundaries. Labels are now measured and clamped within the canvas,
   preserving named axes alongside their colours.

The final harness extends the same cancellation test with domain-change
cancellation, continuation/release outside the canvas and Middle non-admission,
in addition to rect, focus/pointer/context/camera/editability loss. These additions
remain local geometry/Context paths. The changed-domain case checks cancellation;
final harness painting also receives the fixture's current side, but no
changed-domain visual geometry is asserted. Native full-viewport gesture suppression
is not established by the direct Handles harness, which records a consumed flag
rather than invoking a camera. Actual native focus recovery, contrast, high-DPI
targets and assistive usability remain separate gates. Model movement and force
runtime/scientific acceptance remain outside this slice.

## Source disposition before final evidence

No blocking source finding remains after the seven repairs above. The complete
six handle paths and previously inspected five SceneUi/five SceneView paths are
admitted only as individually enumerated exact UI/geometry probes. Current
source inspection does not promote the earlier intermediate pass logs to final
sealed evidence. No test or build was executed by this reviewer.

| Reviewed source | SHA256 |
| --- | --- |
| `src/bin/desktop/scene_handles.rs` | `8c2b0a78283aa8948daaeac5accc1d1c642c2af689cc5d85731df602192556db` |
| `src/bin/desktop/scene_ui.rs` | `5df230c1da9b0c976ea8839e564e6db0ac95bbc6c965ad20cbf6ae13ec069cc2` |
| `src/bin/desktop/scene_view.rs` | `ab55f9a7e5d439211b1c4a4145d9c43adfda15748301055739df5c9251348572` |
| `src/bin/desktop/viewport.rs` | `ccdedbdfcd49740a11c5855bdf48267e3645e60c9d6e4475b9c854359db7a2c6` |
| `src/bin/rheon-desktop.rs` | `6388514fb5a44f43c0c699534e76487f9f7ee527159b701e72265504d1784fab` |

Pending final evidence must retain the exact enumerated names/source seals and
attribute actual checks separately from native usability. The source paths do
not mutate core force semantics, numerical arrays, Request, frozen run intent,
recording/archive or model Apply. Reviewer changed only this report and the
read-only whitespace check is clean.

## Final retained evidence and disposition

Independently verified
`../evidence/force-handles-20261020/manifest.json`, SHA256
`ca49f7b8eee55b8ba2e5ecf5745c6f6ad8d763e6893860934254c3a207ccd261`.
All five source seals match the current files above. All **23** retained logs
match declared sizes/SHA256 and original `/tmp` bytes: 21 accepted logs
(16 exact probes plus enumeration/check/Clippy/fmt/diff), and two preserved
compile/style failures. Log attributes disable text conversion. These checks
verify the packet's identity, not a signed binary provenance certificate.

The final accepted enumeration contains 189 tests. Each of the 16 distinct
inspected names occurs once in it and matches one retained passing exact log;
each reports one selected pass, zero failures/ignored and 188 filtered out.
No test was run by the reviewer. This is focused local UI/geometry evidence,
not evidence of a model, solver, evaluator, worker or recorded run.

The compile-only list4 attempt failed because the newly added near-plane Ray
initializer initially omitted its new field. That initializer was repaired before
the final accepted compilation/execution. The first strict-Clippy attempt rejected
late initialization of `endpoint`. Re-read the final change: both branch bodies
and geometry are unchanged, with the result now expressed as
`let endpoint = if ... { ... } else { ... };`. The final source identity in the
table includes that style repair. Both earlier failures are retained and neither
is represented as a passing gate.

Accepted locked check and strict Clippy complete in 1.08 and 5.29 seconds.
Format/diff logs are empty, consistent with the implementation owner's successful
exit receipt; empty text alone is not an independent exit-status record. Reviewer
performed read/hash checks and `git diff --check`, which was clean, and authored
only this report. No build/test/publication operation was performed by reviewer.

No blocking source or retained-evidence finding remains for this distinct T4
checkpoint. It is suitable for the existing scoped review/publication workflow,
while preserving the separate header/T3 history and the explicit limitations
below. This is not complete native or scientific feature acceptance.

The direct harness demonstrates consumed flags and canonical transactions; it
does not execute full Viewport camera/picking fall-through or an actual active
worker. Native focus recovery when the OS loses a release event, mixed gestures,
high-DPI handle targets/contrast, canvas-edge labels, full property focus order
and assistive usability still require current-revision inspection. Actual model
movement/Apply, immutable-run interaction and authored force runtime/archive
propagation remain unavailable or separately unverified. Existing numerical,
archive-integrity and recording-completeness claims are not expanded by these
authoring tests. The separately authorized synthetic native-presentation harness
is a later task, not evidence supplied by this receipt.
