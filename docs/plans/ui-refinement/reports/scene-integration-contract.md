# Scene UI and shared force integration contract

The user requests compact playback, a collapsible right scene inspector, common
tree/viewport object selection, a thin selected-object outline and manipulators
for models/forces. Desktop UI has one implementation owner here. A separate task
owns shared force physics and solver adapters; parent coordinates its exact handoff.
The parent supplied the exact backend API/semantics from local commit
3e839cbf7c6aca56e8297ef740f7e48543de77b1 (authoring f55cf595, adapters19e0145c,
accepted custody8d2b4f8a),
based on fd7c0f0. Exact published source1d44c8429701f205f678b602040d4f62ea6457b3 is now fetched,
sealed and integrated in isolation; see backend-integration-receipt.md. Runtime
dispatch/desktop frozen intent are still unimplemented.
No UI implementation may invent an alternative domain or claim runtime support.

## Verified current source and boundaries

Source baseline:7318023 plus preserved M2a and reviewed UI-5 overlay, precision
tree3e1fd57146e2f1a02abe22a09ac317497f39d738. Main desktop timeline has an explicit
106px minimum and directly owns existing playback fields. Viewport shows one
accepted collision mesh and emitter; collision controls separately retain draft
path/scale/translation and transactional accepted Arc<Mesh>. Recorded particle
selection belongs to Inspector. Those facts are not a general scene graph.

Current library BodyForce is a stationary MAC-box vector: acceleration m/s² or
force density N/m³, optional world half-open box, sampled on component face
lattices with constant carrier density. Particle configuration has gravity;
that does not establish equivalent general force support. UI must not invent
particle N/m³ conversions, point-force semantics, support sampling or per-backend
approximations from these separate APIs.

ConfiguredControls also already retains a configured-session-only vector/units/
box force draft and publishes its existing ForceSpec. T3 must serially migrate
that form/bridge to the shared canonical authoring owner, preserving saved intent
and archive interpretation. Do not add another disconnected copy of those fields
or represent their current configured-only support as particle parity.

| Owner | Exclusive responsibilities/files | Integration boundary |
| --- | --- | --- |
| Desktop UI task | src/bin/rheon-desktop.rs scene composition, Request/freeze/launch and run_material dispatch; src/bin/desktop/carrier_execution.rs dispatch; new scene/property/manipulator modules, viewport.rs and timeline presentation | Allocate shared IDs, project canonical scenes, freeze once before worker/output creation. Parent confirmed ownership; runtime regions are reserved, not edited in T1. |
| Shared force task | Cargo.toml, src/lib.rs, src/force_field.rs, src/forces.rs, src/simulation.rs, src/particle_fluid.rs, force contract tests/docs; proposed src/material_session.rs and src/recording.rs / src/recording/{carrier,material,provider}.rs custody | Canonical domain/adapters and success-qualified accepted pressure/control custody. Coordinate each newly needed file before edits; no desktop UI ownership. |
| Desktop UI task: confirmed versioned custody | src/bin/desktop/run_sets.rs, run_sets/configured.rs, run_sets/source.rs, run_sets/configured/archive.rs; main Request/history bridge | Parent confirms UI ownership of desktop frozen intent/load/rerun. Backend owns core recording only. Re-admit exact source regions after isolated integration; no such edits in T1. |
| Inspector validation owner | Separately validated Inspector projection/archive work | Recorded probe identity stays separate; revalidate exact source before overlapping integration. T1 edits none of it. |

UI stages are serial; re-admit shared main/viewport/scene regions before each
stage. No speculative merge/rebase/publication. The exact force source handoff is received; source verification and
per-slice admission govern subsequent editing. T1 is independent and can proceed.

## Supplied canonical force domain and runtime gate

Canonical types are ForceScene(version=1, objects), SceneObjectId(u64), ForceObject,
ForceTransform, ForceUnits, ForceKind, ForceSupport, ForceTime and ForceSnapshot.
UI allocates nonzero IDs unique across models and forces. Rename/reorder/save/load
retain IDs; duplication gets a fresh ID. Particle IDs are a separate namespace.
Backend freeze checks force-list uniqueness; it does not provide UI allocation.
Before admitting a loaded scene, validate IDs across its model and force objects
against the whole retained draft. Reject collisions atomically with a visible
reason; never silently remap loaded IDs. The allocator uses checked u64 increment,
starts above the greatest admitted ID, never issues zero and rejects exhaustion
instead of wrapping/reusing an identity. Replacement/import/duplication allocation
must pass this same whole-scene check. Save/load preserves exact admitted IDs.
At most256 forces and256 UTF-8 bytes per name; disabled objects also validate.

Translation/support dimensions are metres. Rotation is local-to-world quaternion
[x,y,z,w], squared norm within1e-12 of one; malformed values are rejected, never
silently normalized. No scale, parenting, animated transform/strength or mesh
attachment in v1. Local q=Rᵀ(p-c); vector rotates back by R. Directional uses its
local vector; radial is signed constant magnitude (positive repels, centre zero);
vortex follows right-hand local +Y (+X points toward -Z, axis zero). No inverse
square approximation. Support Global, Sphere (outer surface excluded), or Box
(local half-extents, -h<=q<h). Sphere/box edge_fraction0..1 uses the supplied
smoothstep falloff; box falloff is max-norm, not spherical. Support rotates.

Acceleration is m/s². ForceDensity is N/m³ divided exactly once by positive local
mass density kg/m³, never particle mass. Existing gravity remains additive; UI
must avoid accidental duplicate Earth gravity. Activation is [start_seconds,
end_seconds), sampled at accepted interval start, no endpoint step splitting;
particle retries share that time. Sum stored order. Preserve all these meanings
in concise property labels and accessible help rather than inventing shortcuts.

Authoring Serde uses exact external enum variants, bare integer IDs and v1, rejects
unknown fields/variants; deserialize is not validation. Bound imported bytes before
parse. ForceScene::freeze validates/deep-copies; ForceSnapshot::scene is immutable,
snapshot has Clone/Debug but no Serde. Serialize scene(), not a mutable run draft.

**Minimal proposed runtime interface (not implemented):** Request retains one
Option<Arc<ForceSnapshot>>, produced from the canonical ForceScene before worker
or output creation and reused for the entire run. History/rerun/provenance retain
that exact scene under an agreed versioned specification. Empty scenes continue
the existing legacy solver calls. Nonempty authored scenes explicitly refuse
launch until the selected workflow has snapshot dispatch AND qualified recording
custody; no silently ignored forces and no direct solver bypass of session owners.

Initial d7bffdd5 metadata disclosed a missing MAC force-pressure stamp, so no UI
wiring was attempted. Parent now reports8d2b4f8a/3e839cbf supplies separate
accepted_force_interval_pressure(), accepted_force_controls(), force-aware run
metadata/publication wrappers and binding validation. Exact source has now been fetched
and sealed in isolation; review the receipt before using these APIs. Do not attach a misleading
legacy stamp or bypass a session/provider owner. Historical glyphs use recorded
scenes, never current drafts; preserve old hashes and explicit absence metadata.

Reported host sequence: freeze one snapshot; build RecordingRun then call
with_force_fields before recorder creation; use carrier/material publication
wrapper for constructor; step through authored-force adapter; publish through
existing qualified recording owners using the expected snapshot. Wrapper and
writer/reader checks must reject missing/mismatched scenes. Metadata v1 is bounded
to262144 bytes and has a digest-bound configuration suffix and named u8 field;
verify exact source/schema instead of duplicating its codec in desktop.

The parent reports five exact bounded adapter tests passed once from the sealed
PRE-CUSTODY binary0042e68e..., not the latest source. Seven pure in-memory codec
tests and static checks cover the new custody source. No post-custody solver,
SQLite or desktop integration execution; do not apply the older solver results
to the custody changes. Session/configured/provider/transport/free-surface force
propagation remains unsupported and must refuse rather than silently ignore forces.

Parent has confirmed desktop runtime and versioned frozen-intent/load/rerun file
ownership above. Source transfer/verification is complete. Precise
per-slice admission and runtime integration remain dependencies; no such runtime changes occur in T1.
Pure scene authoring can now consume the transferred canonical ForceScene;
unsupported run dispatch remains visibly refused until its selected owner path is integrated and qualified.

## Scene selection and properties contract

App owns one typed object key plus explicit Draft/Frozen-run context. Current
supported objects can use distinct Domain/Emitter/accepted-model keys; future
forces and the model use the canonical SceneObjectId allocated by the UI. There is currently one collision model, not
an invented multi-model collection. Tree, viewport and properties emit selection
or edit intent for that same key; neither stores a second canonical property set.
Missing/deleted objects clear selection. Panel collapse retains it.

The singular accepted-model slot needs an explicit import/replacement generation;
neither a row index, path string nor Arc pointer is stable authoring identity.
Reimport/replacement clears a selection for the old generation instead of silently
retargeting edits. The UI allocates the canonical domain SceneObjectId across models and forces;
no competing ID type or row-index surrogate. A model replacement gets a new ID. T2 must settle and test these identities before changing scene files.

During work, project actual frozen Request geometry/forces and disable edits;
retained next-run draft values cannot masquerade as active objects. Recorded
cell/particle probes retain Inspector's separate identity and do not select an
authoring force/model. New model/force authoring is not archive field editing.

This rule also covers paused/completed/cancelled retained runs: current viewport
code keeps reading running_request after worker completion. Worker presence is
not the scene-context discriminator. Use explicit Draft versus Retained-run
context tied to that immutable request/revision. Switching to Draft shows draft
geometry without retained fluid/run geometry, preserves saved history/index and
clears cross-context object selection. Switching back restores the retained view;
do not overlay editable next-run transforms on historical fluid. This is a T2
admission prerequisite, not a claim that the switch is implemented today.

Properties group Transform, supported object-specific values and named optional
advanced/provenance details. Preserve units and concise visible blocking errors.
Named controls and numeric keyboard alternatives accompany axis handles; no
unexplained icon-only scientific controls. Right panel has a labelled collapse
affordance. At narrow widths switch between setup/scene drawers while keeping
the App-owned launch/status/Cancel strip and a usable viewport visible.

## Picking, outline and manipulation contract

Gesture priority: captured manipulator, scene-object pick, recorded probe, camera.
Consume one gesture once. Empty-space selection/clear policy and visibility
through fluid must be explicit and tested. Draw a thin screen-space outline that
follows actual selected geometry/support; retain matching selected name/type in
tree/properties so colour is not the only affordance.

Preview transforms use world metres and the current scene context; normalize only
for viewport projection. Escape discards the preview, release validates and commits
one draft authoring edit, and validation failure retains prior draft/accepted
values with a reason. Model handle release commits draft placement only; accepted
collision geometry still changes through explicit transactional Apply.
Existing model import only supports scale/translation: do not expose working
rotation handles without implementing/reviewing that geometry capability.
Draft model placement remains separate from accepted collision geometry. Keep
transactional Apply/rejection/history consequences; do not reread arbitrary source
files on every drag or silently replace accepted physics with a render preview.

## First independent slice and evidence

T1 extracts only existing timeline painting/input into App::timeline_ui with
explicit selected frame/count arguments. It must not call selected_frame,
Inspector.ensure_timestep, History capture/view, Worker, viewport show or numerical
lab initialization. Remove the106px floor; preserve all playback/track/step/time/
calculation labels, ranges and transitions. Pinned egui bottom panels reserve height
before painting; floor removal alone demonstrably clips cold frames. Admit a local
invisible/disabled sizing Ui with disjoint IDs and a disposable six-scalar playback
projection. Measure the same frozen display context and Frame, then reserve exact
measured height and commit interactive controls once. No App.update replay/global
discard, history reads or persistent parallel playback state. Natural wrapping may
grow when content needs space; idle must not reserve empty height.

Reserve the actual preceding280px sidebar using an inert SidePanel in layout
tests (900/1250 window widths leave about620/970px for the strip). The <=70px
criterion applies to empty idle state; selected captions can naturally wrap/grow.
Use only inspected, list-enumerated, individually named --exact UI tests. Construct
an inert PreviewFrame directly when needed; never capture it from a simulation.
App default is inspected and does not initialize a numerical lab; never call
App.update in these tests. Test actual controls, narrow/wide panel geometry and
visible units/context. No filesystem fixtures/cleanup, model/numerical workloads
or genuine cancellation. Independent review precedes further UI integration.

Native validation remains separate: actual updated idle screens at1250×820 and
900×650,100%/150%, with retained quality85 JPEGs labelled by exact source/tree and
state. No historical screenshot or headless layout is native proof; image delivery
must have confirmed attachment identities. This idle procedure requires no solver
run/cancel workload. Full new scene/manipulator acceptance waits for T2–T4 and the
shared domain handoff; T1 supplies neither force parity nor scientific acceptance.
