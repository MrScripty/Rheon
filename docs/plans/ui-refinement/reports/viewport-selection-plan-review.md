# Independent T3 viewport-selection plan review

Reviewed 2026-10-20 by `ui_plan_review` against
`6127cecbdcfe53270347e59ebe86ff3317528cfe` in
`/workspace/Rheon-ui-authoring`. Initial reviewed plan SHA256:
`799d783f20fbc9469970c9a61de003badefcc58909a52a2f53766ddd36e7d47e`;
viewport-selection admission SHA256:
`b9b59a222715cd902f15c0b1b2d6e4541bde2fe525de56c80e3386ed9cc290f5`.

Read the actual SceneUi selection/context API, viewport camera/input/render
composition, main draft/retained geometry bridge and accepted model topology
contract. Only this review report was authored. No build, test, GUI, solver,
model, evaluator, import or filesystem fixture was executed.

## Disposition

T3 is an appropriate bounded next slice. One pure projected-geometry module can
own authoring strokes, triangle picking and selected outlines while the existing
SceneUi remains the sole selection owner. The aggregate viewport input boundary
is proportionate to the existing composition. Neither force runtime nor its
recording propagation is needed to implement this authoring interaction.

Incorporate the three concrete conditions below before claiming the complete T3
selection/outline contract. They require no competing physics implementation,
new framework, broader worktree edits or optional user approval. Preserve the
exact 6127 source/native handoff; current native validation does not cover future
T3 source. Handles remain a subsequent separately admitted slice.

## Findings and design conditions

| ID | Severity | Condition |
| --- | --- | --- |
| VS1 | Medium | Distinguish unavailable force projection from an empty scene/ordinary miss. An invalid or unrepresentable draft must emit visible preview feedback, preserve canonical values and keep selection available for correction. Do not silently turn validation/projection failure into an empty force list that makes an ordinary empty click clear the selected invalid object. Model projection may have its own independently valid result. |
| VS2 | Medium | State and test the outline topology policy for accepted `TwoSidedSurface` models. That core policy permits open/disconnected surfaces, mixed winding and duplicate-position seams; only convex solids receive the stricter indexed-topology validation. Index-only boundary counting can expose coplanar triangulation seams as an outline. Support literal cases or explicitly refuse unsupported outline topology with feedback, without altering accepted collision geometry or silently drawing all triangle edges. |
| VS3 | Medium | Zero vector/strength is the default new-force state. Do not normalize zero or choose an invented direction. Use a neutral named anchor/type glyph for zero; nonzero schematic glyphs must preserve canonical directional orientation and radial/vortex sign plus rotation. Their fixed visual extent is not physical magnitude or bounded Global support. |

These conditions refine the already admitted validation, geometry-following
outline and truthful schematic-glyph behavior; they do not authorize force
evaluation or change any domain semantics.

## Camera, interaction and geometry contract

- Current `OrbitCamera` projects domain-normalized coordinates, with near depth
  0.015 and screen scale based on viewport height. Scene data is in metres.
  Normalize once at the projection boundary; match perspective/orthographic
  conventions and clip projected strokes/triangles consistently. Do not silently
  normalize malformed quaternions or accept nonfinite/narrowing-overflow data.
- Perspective model/stroke hit depth must be perspective-correct; screen-linear
  interpolation of camera depth is not generally correct. Orthographic depth is
  affine. Literal camera/triangle cases provide an independent expected nearest
  surface without constructing CollisionMesh or sampling a force field.
- Picking and painting use the same current camera, rect, scene context and
  canonical object IDs. Apply camera input before producing that frame's
  projection, or otherwise keep one consistent immutable view projection.
  Do not hit-test a stale cached projection against newly painted geometry.
- The named x-ray overlay policy is explicit: closest authoring stroke/marker
  takes precedence, then its depth and stable ID; absent an overlay hit, use the
  nearest actual model triangle surface. Boundary/tie thresholds need consistent
  screen-space meaning. Empty click clears only a supported, established miss.
- Use unmodified primary click only. A completed camera drag, modified primary,
  middle-button or wheel navigation must not also select/clear an object.
  T4 will add exclusive handle capture before these operations; this slice does
  not add a second gesture owner or modify recorded-probe identity.
- SceneUi.select validates the same object key used by tree/properties. Keep
  retained-run model identity/geometry read-only, and never render current draft
  forces as historical run facts. Current particle probes remain in their
  separate mode; pure T3 tests must not initialize Obstacle/Transport labs.

Projection work is bounded to the declared 256 forces and 12,000 model triangles.
Wire-support tessellation is render-only geometry, with explicit finite bounds
and display failure. Do not silently subsample pickable geometry or mutate a
canonical support to fit display limits. A thin screen-space selected stroke
must accompany the named selection in tree/properties, rather than using colour
as the only selection signal.

## Focused evidence and remaining gates

Inspect complete helper paths, enumerate list-only and execute one explicitly
allowlisted `--exact` pure geometry/UI test per command. Use literal arrays,
canonical force documents and local camera/Context/input only. Required cases
cover actual triangle versus bounding-box miss, front/back depth, perspective
and orthographic overlap, near-plane clipping, degenerate/offscreen geometry,
deterministic ties, valid two-sided seams or explicit topology refusal, zero and
signed glyphs, invalid preview, disabled-object selection and drag/modifier
non-selection. Real egui click tests connect the projection to SceneUi.select.

Do not invoke App.update, CollisionMesh construction/import, numerical lab
initialization, history/recording, Worker or the force evaluator. Compilation,
strict Clippy/formatting and independent source review are supporting gates.
These tests do not prove native hit-target usability, outline contrast/width,
high-DPI focus or complete assistive access. Exact T3 source needs its own native
render/pick/outline evidence; no runtime/physics acceptance or publication result
is implied by this plan review.
