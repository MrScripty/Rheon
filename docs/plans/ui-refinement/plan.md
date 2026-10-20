# Desktop scene authoring and compact playback

**Plan status:** `Active`
**Current phase:** T4 bounded source checkpoint; native validation separate
**Next slice:** workerless native presentation harness, then T5 runtime/archive custody
**Acceptance status:** `blocked`

**Execution ledger:** [execution-ledger.md](execution-ledger.md)

**Issues:** [issues.md](issues.md)

[Scene integration contract](reports/scene-integration-contract.md) ·
[Scene plan review](reports/scene-plan-review.md) ·
[UI-5 precision repair](reports/ui5-precision-repair.md) ·
[T1 compact checkpoint](reports/timeline-compact-checkpoint.md) ·
[Backend receipt](reports/backend-integration-receipt.md) ·
[Current authoring admission](reports/scene-authoring-admission.md) ·
[Authoring source review](reports/scene-authoring-source-review.md) ·
[Native authoring handoff](reports/native-scene-authoring-handoff.md)

## Objective

Follow the user's new direction: a vertically compact timeline and a collapsible
right-side scene tree/properties inspector. Forces and existing models are scene
objects with shared tree/viewport selection, a thin selected-object outline and
manipulator handles. Keep the accepted audit's clean hierarchy, concise text,
clear labels and progressive disclosure. Preserve units, errors, limitations,
immutable run inputs and existing archives.

This direction supersedes continuing the former M2–M4 audit milestones unchanged.
The accepted audit remains libfile_825ba1b07a3c8191a340bb761eea1c89 v0; its40% word
reduction target is a matched-state secondary measure, never a reason to remove
needed object/force meaning or a substitute for this new feature's acceptance.
M1 remains incompletely accepted; old native evidence is scoped to426fd25 only.

Canonical admission is this path, operation `continue`. Sole desktop UI owner:
this task. A separate task owns the shared force domain and both solver adapters.
Only independently owned UI work can begin before its exact API is reconciled.
No optional approval is needed for admitted reversible UI slices after review.
Parent authorizes distinct commits, normal pushes and draft PR146 publication;
no merge, force push or branch deletion.

## Source, standards and preservation

Historical preserved baseline is7318023f04946171d7909119300af76e455c2926.
Active worktree is /workspace/Rheon-ui-authoring, branch
implementation/scene-authoring-ui-20261020, based on canonical backend43ec5052.
The five earlier UI commits and distinct M2a/precision/T1 commits were cherry-picked
with explicit -x provenance, preserving original branches and every checkpoint.
Draft PR146 is based on PR145; no merge or force push. The original uncommitted
M2a worktree `/workspace/Rheon-ui-hierarchy`, its exact patch and logs are untouched.
Detached `/workspace/Rheon-ui-precision` applies that M2a overlay then UI-5; the
reviewed precision checkpoint is frozen as tree3e1fd57146e2f1a02abe22a09ac317497f39d738
under `/workspace/scratch/rheon-ui-precision/checkpoint`. No commit/publication.
The original remote implementation branch is parent-reported426fd25; do not
fetch/reset/overwrite it or the separate Inspector/numerical branches.

Continue the inspected standards route pinned to Coding-Standards0fc92a24279f3a4574c80af0a2f31ea347a21597:
Planning/Implementation/Verification-GUI/Proportionality, Rust, Accessibility,
Diagnostics, Architecture/Code Design and ownership/contracts. No standards-engine
MCP/certificate is available or claimed. Original standards/design and reviews
remain historical evidence; current source and this revised admission govern.

## Scope and non-goals

T1 removes wasted timeline height and isolates its pure painting/input boundary.
T2 adds the right scene list, one shared selection and canonical force authoring
properties. T3 adds viewport picking/outlines; T4 adds manipulators on those same
IDs. T5 wires qualified immutable runtime/archive paths and native evidence.

No competing force physics, backend-specific UI approximations, solver/model/
scientific execution, unadmitted archive migration, worker/session ownership change, training,
L1/MMS work, dependency/framework rewrite, merge, force push or cleanup.
Live cancellation stays parked on missing existing containment; no provisioning.
Canonical domain source is now received and sealed. No fabricated runtime support
while desktop dispatch/frozen intent/load/rerun integration remains pending.

## Dependency strategy and ownership

See the binding [integration contract](reports/scene-integration-contract.md).
UI owns desktop composition/rendering/input/selection; UI allocates canonical SceneObjectId values across models/forces; force owner owns
the type, physical units/support/frame/time semantics, validation/freeze and both
solver adapters. The exact backend API/source at1d44c842 is received and sealed; desktop runtime
integration remains pending. See the source receipt and independent review. See exact proposed runtime/custody
file reservations in the contract; none is edited in T1. Force owner must not edit desktop UI files concurrently. UI must
not edit force/solver implementations. Exchange a pinned API/patch and serially
integrate only after source review. No speculative merges or duplicate physics.

Recorded Inspector particle/cell probes retain their existing owner and identity.
They are not authoring-object selection. Reconcile the separately validated
Inspector slice before any overlapping changes; T1 excludes Inspector entirely.

## Milestones

| Milestone | Exact production write set and outcome | Dependency / gate | State |
| --- | --- | --- | --- |
| T1 | main timeline presentation and desktop/timeline_ui_tests.rs; bounded labelled speed editor, ordinary invisible local measurement, unchanged physical context | Four inspected exact UI probes + compile/Clippy/fmt + independent source review; native remains separate | Completed (bounded source) |
| T2 | new desktop/scene_ui.rs; main App scene composition/context/model-ID transactions and draft launch refusal; exact inert scene tests; viewport.rs mode-control guard only (no lab dispatch/picking changes) | Canonical domain received; reviewed [authoring admission](reports/scene-authoring-admission.md); one ForceScene/selection owner; viewport mode refusal only, no runtime dispatch | Completed (bounded source) |
| T3 | new scene_view.rs geometry/picking; viewport.rs aggregate-input composition and thin geometry outline; existing SceneUi selection and main routing | [Exact selection admission](reports/viewport-selection-admission.md), inert camera/geometry/UI tests; no lab initialization, then native | Bounded source complete; native pending |
| T4 | desktop/scene_handles.rs, viewport exclusive gesture routing, scene_ui validated draft transaction; force-only (model placement deferred) | [Force-only handle admission](reports/force-handles-admission.md); same IDs/domain types; no physics; exact Escape rollback and terminal-frame ownership | Bounded source complete; native pending |
| T5 | main Request/run_material, carrier_execution.rs, desktop run_sets versioned intent/load/rerun; exact backend recording bridge and native fixes | Qualified canonical custody source + reviewed immutable intent/version compatibility + separately bounded integration evidence. Unsupported owner paths refuse | Planned |


Stages execute serially under the UI owner; T2–T4 reuse some UI regions and are
not parallel write sets. Every next stage explicitly re-admits its exact regions
before editing. Force-domain write sets are disjoint throughout. Only the lead
writes canonical plan/ledger/issues; independent reviewers write their reports.
Generated logs/screenshots/transfer packets remain outside Git.

## Binding Decisions

1. App owns one ephemeral typed scene-object selection and current scene context, including import/replacement generation for the current singular model slot.
   Tree, viewport and properties emit Select/Edit intents against that owner; no
   duplicated canonical force/model properties. Draft and frozen-run contexts are
   explicit, and missing/deleted objects clear selection without retargeting.
2. Right scene panel collapses without losing selection/expanded rows/draft values.
   Properties belong to the selected object, not a disconnected force form.
   At narrow logical widths use one open setup/scene drawer at a time while an
   App-owned global run/status strip keeps the one launch action/Cancel reachable.
   Maintain usable viewport width; do not solve crowding by shrinking body fonts.
3. Force units, support, activation/time, coordinate frame, meaningful transforms,
   density/mass conversions and backend support come only from the domain API.
   Unsupported backend/object edits show blocking reasons. Do not map existing
   MAC-only BodyForce to particles or confuse visual anchor with physical support.
4. Active, paused, completed and cancelled retained-run scene geometry/properties come from immutable Request/snapshot
   facts and are read-only, independently of worker presence. An explicit Draft
   scene switch stops displaying retained fluid/run geometry without deleting
   history; selection clears across contexts. T2 must reconcile this before edits. Draft edits affect a future run through the existing
   validate/freeze/launch boundary; no mutation of an active session/archive.
5. Existing collision draft path/scale/translation and accepted Arc<Mesh> remain
   distinct. Keep transactional Apply/rejection and explicit history consequences.
   Accepted geometry is never silently replaced by an unvalidated drag preview.
6. Handle capture has priority over object pick, recorded probe and camera input.
   One gesture has one owner. Escape cancels preview, release validates/commits one
   draft authoring edit (models still require explicit transactional Apply), and numeric unit-labelled properties provide a keyboard alternative.
   A thin geometry-following selection outline accompanies tree/property identity;
   colour alone does not communicate selection or errors.
7. T1 retains transport/speed/Live/track/scrub semantics and physical time/step
   metadata. Replace the height floor with local content measurement; do not discard status/uncertainty or
   move scientific details until a later measured need justifies that exact change.
8. Every test path is inspected before list-only enumeration; execute one explicit
   fully qualified `--exact` UI-only name per command. No substring filters,
   fallback suites, simulations, model runtimes, archive fixtures or file cleanup.

## Objective Acceptance

| ID | Observable criterion | Kind / environment / mode | Status | Evidence |
| --- | --- | --- | --- | --- |
| S1 | Empty idle timeline fits <=70 logical px in900/1250px windows with the preceding280px sidebar reserved; selected-frame captions remain present and may naturally wrap/grow; actual Play/Pause/step/Live input keeps prior transitions | integration / isolated egui painting/input / automated | pending | Four exact probes pass: idle48px versus106px, focused speed editing and native-track/literal-report layouts. Native/configured/popup-keyboard coverage remains open |
| S2 | Right panel collapses/reopens retaining selection; tree/viewport choose the same existing object; deleted/replaced/missing identities never edit another object | integration + user-workflow / inert UI then native / automated + manual | pending | Canonical authoring source and five exact UI probes pass; viewport picking/native remain pending |
| S3 | A force is one canonical scene object in tree, properties, outline and handles; UI exposes only shared supported units/support/transform; unsupported backend blocks with a reason | integration / shared domain contract + inert UI / reviewed + automated | blocked | Canonical authoring source uses one ForceScene and checked IDs; dispatch/frozen intent still pending |
| S4 | Handle capture prevents simultaneous camera/probe motion; Escape restores exact draft; release commits one validated unit-correct edit; active snapshots stay immutable | integration + user-workflow / inert UI then native / automated + manual | pending | Domain/manipulator stages not admitted yet |
| S5 |1250×820 and900×650 at100%/150%: usable viewport, no overlap/clipping, visible status/Cancel/errors, thin outline, keyboard alternatives and predictable drawer focus | user-workflow / actual native display / manual | blocked | No local display; prior426fd25 images do not cover this source |
| S6 | Locked affected check, strict Clippy, formatting and individually selected exact UI tests pass; existing archives/source ownership unchanged | focused / pinned Rust1.92 / automated + source review | pending | T1 focused gates reported in checkpoint; no scientific certificate |

The former M1 A4/A5 and live cancellation remain unsatisfied. UI-5 has a reviewed
isolated-form exact-bit repair, not native/global keyboard acceptance. Earlier
accidental numerical-fixture results earn no acceptance credit; automatic test
teardown removed their disposable outputs before detection and no resource
measurements/scientific result are claimed. Preserve every incident/failure log.

## Simplicity And Ownership Review

**Applicability:** `applicable`

- Independent concepts and dimensions: App composes scene context/selection and intent; UI paints
  canonical projections; force domain validates/freezes physical meaning; solver
  adapters execute it. Inspector owns recorded numerical probes separately.
- State, identity, value, time, policy, and mechanism: one selection keyed by stable
  domain/object ID within draft/frozen context; no parallel force-value registry.
- Caller and composition-root knowledge: desktop knows capabilities and routes explicit
  intents, not force integration formulas or backend density/mass policy.
- Representative change paths and forced owners: new domain force type supplies capabilities/properties;
  new renderer changes drawing only; new model import retains geometry ownership.
- Stable Interfaces versus hidden knowledge: typed selection/edit intents and pinned validated freeze API,
  not string-parsed status or copied provider/worker internals.
- Independent evolution, testing, failure, and replacement: T1 paints/tests without history access; force
  validation failure retains draft/accepted values and a visible blocking reason.
- Deletion and cumulative machinery result: ordinary modules and enums suffice. No ECS,
  scene framework, duplicate physics schema or universal widget abstraction.
- Necessary complexity and containment: draft-versus-frozen identity and exclusive gesture capture
  are inherent; contain them at App/scene boundary rather than inside solvers.

## Rollback and re-plan

Preserve all checkpoints. Roll back only a slice's exact incremental working patch,
never reset the original M2a/Inspector/numerical worktree or delete saved output.
T1 is presentation-only; no migration. T2–T4 require explicit source/identity
review before rollback because scene selection may refer to removed draft objects.
No persistent-data rewrite is admitted.

## Re-Plan Triggers

Re-plan for force API/capability mismatch, UI/force overlapping ownership, silent
backend approximations, broken active/draft identity, native drawer crowding,
changed collision admission, lost units/errors, or any uncertain test helper path.
Ordinary lint/layout failures stay within the current exact slice and retain logs.

## Blockers

Canonical force source is received/sealed in isolated branch; its arrival does not
implement scene UI or desktop runtime. Desktop dispatch/frozen intent/load/rerun
and unsupported owner paths still need staged integration and qualified evidence.
Right-scene
integration must reconcile exact Inspector work before overlap. Local native
display unavailable; JPEG delivery of older screenshots remains separately blocked.
Live cancellation lacks required existing containment and stays parked. No mount,
cgroup, permission, credential or network changes are authorized.

## Final Acceptance

T1 focused speed edit-buffer geometry and native carrier track/literal-report probes
pass after bounded editor and ordinary-layout measurement repairs. Configured
long captions, popup keyboard/window bounds and native behavior remain unaccepted.
No complete UI, force parity, native/accessibility, M1 or scientific acceptance is
claimed. Accept only exact staged source/test/native evidence once its objective
criteria are met. Publish/merge only after separate coordination/authorization.
