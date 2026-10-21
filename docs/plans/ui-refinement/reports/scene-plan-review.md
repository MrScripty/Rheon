# Independent scene and timeline plan review

Reviewed 2026-10-20 by `ui_plan_review`. Source: local HEAD
`7318023f04946171d7909119300af76e455c2926` plus preserved M2a and the reviewed
UI-5 overlay, whose frozen production checkpoint is reported as tree
`3e1fd57146e2f1a02abe22a09ac317497f39d738`. Reviewed the revised canonical
plan, scene integration contract and actual timeline, viewport, collision,
force, camera, history and recorded-probe ownership. Only this report was
authored. No runtime test, GUI, solver, model or filesystem fixture was run.

## Disposition

T1 is a suitable independent reversible slice: extract the existing timeline
painting/input boundary and remove its 106-pixel height floor while preserving
all controls, captions, ranges and transitions. It does not depend on the force
domain or authorize scene/physics changes. The lead incorporated all findings
below before production edits; no T1 plan/design blocker remains. No further
optional user approval or broad design investigation is needed for T1.

The scene/force direction is appropriately separated into later admissions.
The shared force API is explicitly pending, not agreed or inferred from existing
MAC forces/gravity. T2–T4 must resolve their context/identity/edit boundaries
before overlapping production edits. M1/native/global accessibility and the
previous delivery limitations remain unsatisfied.

## Findings

| ID | Severity / scope | Finding and required disposition |
| --- | --- | --- |
| SP1 | Medium / plan conformance | The revised simplicity review shortened the eight canonical probe labels and the T1 milestone uses `Admitted`, outside Planning's declared lifecycle. Restore the pinned source validator's exact probe labels and use `Planned` or `Active` as appropriate; run the already established structural check as supporting evidence. A structural pass is not design/engine certification. |
| SP2 | Low / T1 evidence | S1's height criterion needs an exact state and width boundary. Define the <=70 logical-pixel target for the empty idle strip, not every paused/selected state with all captions. Reserve the actual preceding 280-pixel setup sidebar when testing 900/1250 window widths, rather than giving the timeline the entire window. A simple inert SidePanel reservation supplies that layout without calling the real sidebar. Selected-frame/long-track captions may naturally grow; test no clipping or lost units without a new fixed cap. |
| SP3 | Medium / before T2 | `worker.is_some()` cannot define scene context. The current viewport continues using `running_request` geometry/domain/emitter after completion or cancellation while historical frames remain, but next-run setup is editable. Specify Draft versus frozen/retained-run viewing through paused/completed/cancelled history and explicit scene switches, plus selection disposition. Otherwise editable draft properties can represent different geometry from the highlighted historical object. This does not block T1. |
| SP4 | Medium / before T2 | The accepted-model key needs an import/replacement generation. Current collision state is one replaceable mesh slot; path, row index, Arc pointer or a reused singleton `Model` key does not establish stable identity. Reimport/replacement must invalidate or explicitly transition selection rather than silently retarget an edit. App can observe the existing accepted-change result without giving viewport/properties a second geometry owner. |
| SP5 | Medium / before T4 | Clarify release/Apply semantics. Handle release commits a validated draft placement or authored-force edit; it must not implicitly replace accepted collision geometry or mutate active physics. Accepted model geometry still changes through explicit transactional Apply, existing rejection/retention and history effects. A drag does not reread/import a file per frame. |

Resolution check: all five findings are incorporated in the revised plan/contract.
The exact eight probe labels and T1 `Active` lifecycle are restored. S1 now names
empty idle state and actual sidebar reservation. Draft versus Retained-run is
explicit independently of Worker presence; switching clears context-dependent
selection. Accepted-model generation and draft-only handle-release/explicit Apply
semantics are stated. The contract also identifies the existing configured-only
force form and assigns its future T3 migration rather than allowing a second
canonical force form or falsely claiming particle support.

Final reviewed plan SHA256:
`b79c5eb98eabebd5de8372cc404a2f328195fe4375280889677ed33631618fc1`.
Final reviewed integration contract SHA256:
`7016698534bb32983cc80bf22a663e0c4bcba233860acc339ca022b34620ba82`.
The timeline production source remained unchanged at this resolution check.
Structural-validator execution remains the lead's supporting gate; this reviewer
ran no tests and does not claim engine certification.

## T1 implementation and evidence boundary

The current block at `src/bin/rheon-desktop.rs:2144` owns Play/Pause, stepping,
Live, playback speed, track selection, scrubber and selected-frame captions.
Keep those transitions together in `App::timeline_ui` with explicit selected
frame/count arguments. The method must not query a new frame, history, Worker,
Inspector, numerical lab, filesystem or model. Remove only the reserved height;
moving scientific captions to details is outside this first slice.

Use directly constructed inert `PreviewFrame`/Request values and local egui
Context/input for the exact new tests. `App::default` does not initialize the
numerical lab, but `App::update` is outside this test boundary:
`selected_frame` can invoke `Inspector::ensure_timestep`, and viewport mode
initialization can compute independent numerical state. No captured simulation
frame, LiveHistory, archive, subprocess or disposable output is needed.

Inspect each complete test/helper path, enumerate list-only, then run one fully
qualified `--exact` allowlisted UI name per invocation. Preserve the previous
incident and failure logs; do not repeat a prefix suite or cleanup. Test actual
transport/Live/scrub input, initial/end stepping and track reset behavior using
inert state. Native geometry/focus evidence remains separate from headless
input/paint results.

## Ownership conclusions

- App owns one ephemeral typed selection within an explicit scene context.
  Tree, viewport and properties use that key and emit intent; they do not keep
  competing canonical object properties. Panel collapse retains selection.
- Recorded particle/cell IDs remain Inspector/probe state, distinct from scene
  authoring IDs. The right authoring inspector does not replace numerical
  archive inspection or create missing archived object provenance.
- The force owner supplies canonical identity, units, support/boundary convention,
  activation/time, transforms, validation, freeze and both-backend semantics.
  Existing `BodyForce` is MAC-specific; UI cannot infer particle N/m³ conversion,
  point-force behavior or support sampling. A whole-box uniform-force visual
  anchor must remain distinct from physical localized support.
- The UI task exclusively owns desktop composition/input. The force task must
  supply a pinned reviewed domain handoff without concurrently editing desktop
  files. Serial shared-region re-admission is appropriate; no speculative merge
  or duplicated force adapter is justified.
- Existing collision draft path/scale/translation and accepted immutable mesh
  are distinct truths. The current one-model capability is stated honestly;
  no placeholder multi-model collection or unsupported rotation is advertised.
- Captured handle gestures precede object picking, recorded probes and camera
  input. A future outline follows actual selected geometry/support and has a
  matching named tree/property selection; colour alone is insufficient.
- The narrow-screen single-drawer/global-run-strip proposal belongs to T2 and
  needs actual native space/focus evidence. It must not expand T1 or weaken
  existing status/Cancel visibility.

The replacement design has the required proportionate direction and concrete
ownership. Source-only standards review and an honest missing domain/display
contract support the independent T1 work; they do not accept the larger scene
authoring feature, force parity, scientific behavior, publication or cleanup.

## Follow-up: supplied force metadata and bounded T1 sizing

The updated plan/contract now attributes canonical force metadata to parent
handoff `d7bffdd5d6e80e022f15c59d3574739e3b39b352`, including its authoring/adapter
lineage. Exact source transfer remains pending. This review cannot verify those
new types or adapter semantics from the current main-derived checkout and does
not promote the metadata or unexecuted integration cases to runtime parity.

No source-independent T1 admission blocker remains. The observed floor-only
cold-frame clipping supports a narrowly scoped local sizing pass: one disposable
copy of the six existing playback scalars, a disjoint-ID invisible/disabled Ui,
the same frozen display context/Frame, then one real interactive paint and one
commit back to App. Pinned egui 0.33.3 `UiBuilder::invisible()` explicitly disables
the Ui. This does not justify an App replay/discard loop, persistent duplicate
playback state, numerical history reads or Worker access. Exact cold, changing-
caption and actual-input regressions remain required; this source/admission
review ran none and does not claim their results or native appearance.

The minimal proposed `Option<Arc<ForceSnapshot>>` retains a single immutable
snapshot rather than copying mutable authoring state per step. Validation/freeze
must precede Worker/output creation. Keeping empty scenes on the old solver path
is important because the reported new MAC path lacks the pressure stamp consumed
by current `carrier_publication`; current source explicitly rejects a non-
constructor state without success-qualified interval pressure. Do not route even
an empty scene through the unqualified new step method. Nonempty authored scenes
must visibly refuse a selected workflow until both dispatch and accepted
pressure/control/recording custody are qualified. Preserve configured-session,
particle/session and transport owners rather than bypassing them to make a new
step callable.

Runtime/custody ownership is correctly marked proposed/reserved, not permission
to edit. Parent confirmation of exact file ownership and a reviewed source
handoff remain required before runtime or persisted-spec/digest changes. In
particular, configured execution currently passes through `ConfiguredTrack` and
its accepted-session/provider owner; it must remain unsupported for new authored
forces until that path has an explicit agreed disposition. The current T1
production changes do not edit any of these boundaries.

The additional scene-stage ID requirement is incorporated in the latest contract:
import checks cover model and force IDs against the whole retained draft, reject
collisions atomically without remapping, and use checked u64 allocation above
the admitted maximum with no zero/wrap/reuse and explicit exhaustion rejection.
Save/load preserves exact IDs. The reported backend freeze checks uniqueness
inside the force list only; the UI retains the combined-namespace obligation.
This supplies a coherent admission contract, not proof of its future implementation.

Updated admission reviewed at plan SHA256
`a28d43ea72b6ddb3f5735f3f7062ae6d34301c729e7a82a80c39e37f78fe18da`
and integration-contract SHA256
`9d14082474e368a8dd71e38615d19f4526c07e37cc41ba472921ee063b9a4bcd`.
Earlier reviewed hashes above retain their original scope. No test, numerical,
model, GUI, filesystem fixture, cleanup or canonical-plan mutation was performed
by this reviewer; only this follow-up report changed.

Final metadata/sizing admission disposition: no remaining source-independent
blocker to T1. Latest contract SHA256:
`2a51b777e23b104548cbeb4e42eb38ecb7a4c52f4228aa46ff653a4291f0cc35`.
The lead's `/tmp/rheon-ui-scene-plan-structure-3.log` records the pinned structural
check passing; this reviewer inspected that result without executing it. Exact
domain source transfer, parent-agreed custody ownership, workflow-qualified
dispatch/recording and native evidence remain gates on their own later claims.
Nothing in this disposition admits runtime/custody edits or a force workload.
