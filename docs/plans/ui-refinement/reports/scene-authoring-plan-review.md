# Independent canonical scene-authoring admission review

Reviewed 2026-10-20 by `ui_plan_review` in `/workspace/Rheon-ui-scene`, branch
`implementation/ui-scene-integration-20261020`, HEAD
`7318023f04946171d7909119300af76e455c2926` plus the preserved UI and canonical
backend overlays. Subject: `scene-authoring-admission.md`, initial SHA256
`7618709846988cd208e5fd3497b6efbc75b5ebca33fb8afaf1abb70a7a07fac3`, the
canonical plan/integration contract, and directly affected App/collision/domain
source. The actual `force_field.rs` read here hashes to
`adecbc5f92b89179a2e38885be2acbcddd42df40061f00ffa5d758224ed045a5`.

Only this review report was authored. No tests, solver/model workloads, GUI,
filesystem fixtures, compilation, commits or publication were performed by the
reviewer. The backend's newer reported remote tip changes evidence/docs only;
this review relies on the transferred canonical source and its reviewed receipt,
not on a new runtime claim for that tip.

## Disposition

The authoring-only slice is proportionate and independently implementable:
one canonical ForceScene, one shared checked model/force ID allocation owner,
one context-scoped selection, a right scene list/properties panel, and Add
Directional/Radial/Vortex. No viewport, solver, recording or run-set change is
needed. Existing snapshots remain authoritative and the new authored scene
cannot launch until runtime/custody integration is independently admitted.

The lead accepted the concrete transaction/scope conditions below before scene
production edits. No unresolved source-independent design blocker remains once
the canonical plan selects this exact next slice and records the preceding T1
focused-gate disposition. That is a lifecycle/scope update, not another optional
user approval or a reason to broaden design investigation.

## Conditions and agreed dispositions

| ID | Concern | Reviewed disposition |
| --- | --- | --- |
| SA1 | Collision import currently commits `collision.mesh` before `show` returns its accepted-change flag. Exhausted shared IDs must not leave replacement geometry unidentifiable or clear history on a failed scene admission. | Lead will retain prior accepted Arc/status before painting the existing collision form. On successful replacement, admit a fresh canonical ID before resetting history; if allocation fails, restore accepted geometry, report refusal and preserve prior ID/selection/history. Removal needs no new ID. Initial CLI import must also perform transactional ID admission. No collision evaluator/importer rewrite is needed. |
| SA2 | The empty authored list is not proof that no physical forces exist: gravity and the existing configured-session-only force form remain separate legacy sources in this slice. | Label the collection `Authored forces`, keep a visible gravity-additive note, and avoid total/no-force physics claims. Legacy retained force metadata stays unavailable; do not reconstruct new force objects from gravity or the current configured draft. |
| SA3 | Canonical plan still selected T1 at initial review, whereas this report supplies a narrower next authoring write set than the former T2/T3 table. | Before editing scene production, record T1's actual focused checks/disposition and set exactly one canonical next slice with the new `scene_ui.rs`/main/test boundaries. Reports remain non-authorizing detail/evidence, not competing plan authority. |

The transaction described in SA1 preserves old accepted geometry/identity and
history on refusal, while leaving the user's new import settings as draft input
for correction. Test its pure admission/rollback helper with in-memory summaries
or literal geometry if needed; do not exercise filesystem imports in this UI pass.

## Canonical source and ownership review

- `ForceScene::freeze` checks version, at most 256 objects, nonzero and force-list
  unique IDs, then validates all objects including disabled ones and deep-copies
  the document. The UI must additionally enforce its combined model/force
  namespace with checked monotonic allocation and explicit exhaustion refusal.
- Names are bounded in UTF-8 bytes. Canonical units are acceleration or force
  density; translation/support use metres and activation uses seconds. Quaternion
  validation rejects malformed values; the UI must not silently normalize or
  impose unrelated physical limits. Use the shared domain directly, not another
  evaluator, force schema or backend-specific approximation.
- Zero-strength additions are safe explicit authoring defaults. Invalid draft
  values remain visible and cannot launch; a disabled object still validates.
  Support, local vector/orientation, radial/vortex sign and time semantics need
  concise named help. Numeric field editing supplies the initial authoring path;
  spatial picking, outlines and handles are explicitly subsequent slices.
- App's collision controls own mutable import placement plus accepted mesh;
  Request retains the immutable run mesh. `SceneUi` may retain IDs/selection and
  receive read-only accepted summaries, but must not clone a second canonical
  model-property store. Context changes clear selection and stop hidden playback
  while preserving history/index. Draft geometry must not overlay retained fluid.
- New authoring must be disabled in retained/active contexts. Retained scene
  properties come from the actual Request, with old force provenance explicitly
  unavailable. Current draft forces must never appear as recorded run facts.
- Every nonempty authored draft, including disabled/zero-strength objects, must
  refuse both preview and configured new launches before path/output/Worker
  creation. Preserve legacy frozen reruns as their own immutable intent; they do
  not consume the current authoring draft. Keep the blocking reason visible when
  the scene panel is collapsed, rather than only inside object properties.
- At narrow widths, compose one setup/scene drawer and call the existing
  App-owned run/status/Cancel projection once. Do not create a second action or
  worker owner. Panel collapse retains selection and draft values; native focus,
  clipping and scaling evidence remains required.

## Proportionate evidence boundary

Inspect each complete helper path, enumerate list-only, then execute one fully
qualified `--exact` UI-only test per command. Literal force documents, model
summaries, canonical freeze/validation and egui input/paint are sufficient for
add/select/remove, property editing, ID exhaustion/rollback, context and collapse.
Do not call field evaluation, simulations, Worker, App::update, history capture,
file import, archive publication, external processes or cleanup from these tests.

Test launch refusal through the pure policy/actual inert control projection and
review the early guards in real launch methods. Do not make a regression test
that might start a workload if its refusal guard fails. Compile/lint and source
review complement those tests; none proves native rendering or force dispatch.

The separate backend's core recording/accepted-pressure work and upstream
evidence retain their own source/coverage limits. This admission does not accept
post-custody solver/SQLite integration, configured/provider propagation, saved
force replay, native scene interaction or scientific parity. Distinct reviewed
commits and any later draft publication follow the parent's actual gates; no
publication result is implied by this review.
