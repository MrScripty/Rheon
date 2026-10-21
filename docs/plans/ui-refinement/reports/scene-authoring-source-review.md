# Independent scene authoring source review

2026-10-20. Scope: uncommitted authoring increment on
`implementation/scene-authoring-ui-20261020` in `/workspace/Rheon-ui-authoring`,
parent `7dcbfa67578875dd266020be589e2fbea40467f8`, following canonical `43ec505`
and the eight separately preserved UI commits. This is source review, not native
interaction, scientific validation or a certificate for the earlier draft PR.
Reviewer executes no tests, numerical/model workloads, imports, recording/SQLite
operations, filesystem fixtures or cleanup. Only this report is authored.

## Source and test boundary

Reviewed the complete new `scene_ui.rs`, the main App composition/context/launch
diff, the quantity-label association and the subsequently admitted viewport
mode guard against `scene-authoring-admission.md`. Core/recording/runtime and
versioned run-set files are outside this slice. The viewport addition only
projects a pending-authored-draft flag, chooses Appearance and disables other
mode controls before their independent lab dispatch; it adds no force evaluator,
pick, outline or manipulator.

The four initially admitted exact tests are all `scene_ui::tests`:

- `scene_identity_context_removal_and_exhaustion_preserve_canonical_values`
- `scene_pointer_add_select_edit_collapse_and_retained_context`
- `scene_invalid_support_and_rotation_are_visible_without_snapshot_mutation`
- `scene_force_refusal_projects_disabled_run_without_dispatch`

Their entire frame/click/key/replace chain uses a local egui Context, literal
authoring values and direct SceneUi/run-controls painting. `ForceScene::freeze`
only validates and deep-copies immutable input; it never calls acceleration or
a solver. Euler/quaternion conversion here is ordinary UI coordinate arithmetic,
not a numerical simulation fixture. The App test calls `run_controls` but never
dispatches its return, calls App::update/start/Worker, or creates history. App
default retains absent worker/history/runtime objects; its added SceneUi default
constructs an empty canonical document and scalar IDs. RecordingLimits defaults
are constants, Inspector defaults retain no reader and ExportFile::new stores
path strings only. App Drop's join branch requires a worker, which the test
asserts absent. No model import, recording, file output, process or cleanup path
is reached. This source boundary admits only those individually enumerated
fully qualified exact probes, not a substring suite or uninspected new helper.

## Ownership, units and transactions

App owns one SceneUi containing the sole canonical ForceScene and selection.
Selected properties directly edit that document; transient Euler controls only
write a new quaternion after an explicit edit/reset. Untouched orientations are
not normalized or rewritten. Metres, seconds, m/s², N/m³ and degrees are labelled;
the supplied domain retains quaternion/SI storage, support boundary conventions,
activation and local-density semantics. Changing unit kind explicitly reinterprets
the authored number, without inventing a density conversion. Gravity remains
separate/additive. Properties validate through the backend freeze API, including
disabled objects; invalid draft values remain visible and cannot execute.

The checked monotonic allocator shares nonzero IDs across the one model slot and
forces. Remove does not recycle identity; exhaustion refuses instead of wrapping.
Model replacement requests a fresh ID before committing scene identity. The main
bridge restores the previous accepted mesh and avoids reset on refusal; removal
and successful transactional Apply retain the existing explicit history-clearing
behavior. CLI model admission is included. There is no model collection, import
schema or save/load interface added by this slice, so future import must still
enforce whole-scene collision and exhaustion checks.

Draft and Retained are explicit contexts independent of worker presence. The
intended retained projection uses immutable Request geometry and absent/unknown
legacy authored-force metadata, never reconstructing it from the current draft.
Editing and adding forces are disabled during an active worker or retained view.
Collapse retains selection; removal/replacement and context changes clear it.
The right list is a future common selection owner, not implemented viewport
picking. Accepted model properties remain read-only; existing setup Apply owns
collision admission. Runtime force propagation, persistence, outlines and handles
are not claimed.

Both new preview and configured draft launches refuse any nonempty authored
document, including disabled objects, before output creation/Worker entry. Legacy
frozen reruns intentionally retain their own recorded intent and do not consume
the current draft. The independent viewport lab guard is consistent with that
unsupported-workflow policy. Narrow composition reuses the existing run/status/
Cancel owner in a top strip while the scene drawer replaces setup; it does not
create another worker or action owner.

The settings change captures the existing visible label and associates its ID
with the returned quantity response. It does not alter the precise formatter,
numeric parser, conversions, ranges, widget order, auto-ID allocation or canonical
values. It is an intentional delta from the prior precision source hash, not a
claim that the whole file remains byte-identical.

## Findings tracked during review

1. **Repaired:** Draft particle view initially received the retained Inspector.
   Main now filters that projection by retained context, preventing historical
   points from being interpreted with draft dimensions. Particle-view controls
   clear their display cache on absent Inspector; retained data is not deleted.
2. **Repaired:** Retained-to-Retained new runs could preserve stale model
   selection. `begin_run` now clears selection every time; a pure assertion was
   added without constructing a worker or run.
3. **Repaired:** An invalid future draft error was visible while inspecting a
   retained legacy run. Validation labels are now scoped to Draft. Name help and
   validation wording identify the backend's 256 UTF-8-byte limit; the TextEdit's
   character limit remains only an editor aid and does not replace validation.
4. **Open at this review checkpoint:** Draft paints the real timeline with zero
   count, so pinned egui Slider's default Always clamping writes step zero even
   while disabled. This loses the retained index. Selecting Retained after a
   precomputed Draft count can cause the same issue for one frame. Preserve the
   real retained playback state by omitting/discarding Draft timeline interaction
   and selecting the actual count/frame after scene context input settles.
5. **Open at this review checkpoint:** The independent “Compare Rheon / trained
   AI” entry checks only worker presence and can enter a different runtime owner
   without consuming/refusing an authored draft. Apply the same unsupported-draft
   refusal before entering it, or explicitly admit a different scoped policy.
   “Compare both pressure solvers” already routes through guarded `start(true)`.

No current test execution, visual geometry or native evidence is claimed in this
initial source-admission receipt. Native widths/scales, right-panel focus/order,
keyboard add/select/remove, full transform/support editing, active-worker input
locking, top run-strip geometry and model-import rollback interaction still need
their corresponding focused/native evidence. The complete App update must not be
used as an allegedly inert test harness: it can load Inspector fields or initialize
independent numerical labs. Pending source fixes and final hashes/results will be
recorded below before final source disposition.

## Follow-up source corrections and fifth exact probe

Both previously open findings above are repaired in the re-inspected source.
`App::scene_timeline_ui` returns None for Draft without painting/committing real
playback controls. The App selects retained count/frame only after scene context
input has settled, so a Draft-origin zero count cannot clamp the retained index
on a Retained transition. The AI comparison entry now requires the same draft
launch readiness; frozen archive rerun policy remains deliberately separate.

The fifth inspected exact test,
`scene_ui::tests::scene_draft_timeline_roundtrip_preserves_retained_index_without_history`,
uses App default, a literal PreviewFrame and local Context. It passes nonzero
step 7, track 1 and exact play-time bits through Draft/Retained/Draft/Retained
via `scene_timeline_ui` only. Retained count is a literal 10; it never calls
selected_frame, creates history, loads fields, dispatches actions or calls
App::update. The existing refusal test was strengthened to physically click
Simulate, but its projection captures only whether run_controls returned an
intent. Even a regressed enabled button cannot launch a worker in that helper.
These complete helper paths remain inert; no reviewer test execution occurred.

The admitted viewport guard's default is one false scalar. Main sets it only for
Draft plus nonempty canonical authoring objects. At viewport entry it chooses
Appearance before dispatch and disables Particles/Obstacle/Transport choices;
it does not initialize those independent workflows or claim force visualization.
The five scene tests never call Viewport::show. Guard behavior is therefore
source-inspected, with actual viewport/navigation coverage still pending.

No remaining blocking source finding has been identified after these repairs.
The focused probe admission is five fully qualified, list-enumerated exact names
only. Final source hashes, implementation-owner checks and actual evidence
limits must accompany the resulting checkpoint; this disposition is not native
or complete scene-feature acceptance.

## Final source disposition and checked evidence

No blocking source finding remains for this bounded authoring increment after
the corrections above. Source review supports the next implementation checkpoint
with the stated feature/evidence limits. It does not accept authored-force
execution, persistence, viewport picking/outlines/manipulators or native UI.

Read the implementation owner's final logs, without rerunning any test:

| Log under `/tmp/rheon-ui-scene-authoring-` | Checked result |
| --- | --- |
| `final-ids.log` | One exact identity/removal/exhaustion test passed |
| `exact-pointer-2.log` | One exact pointer add/select/edit/units/collapse/context test passed at 900/1250px widths |
| `final-validation.log` | One exact invalid-draft/frozen-snapshot/rotation-values test passed |
| `final-refusal.log` | One exact disabled-Simulate intent-only projection test passed |
| `final-context.log` | One exact history-free Draft/Retained index roundtrip test passed |
| `final-precision.log` | One exact existing material focus/Tab bit-preservation test passed after label association |
| `clippy-1.log` | Desktop/test strict Clippy completed successfully in 52.99 seconds |

Each runtime log reports one selected pass and 176 filtered out; list-only
enumeration reports 177 tests. The retained `exact-pointer-1.log` failed at a
literal `0 N/m³` caption lookup. Its test-only repair locates the actually shaped
caption by exact unit suffix and numeric prefix equal to zero, then clicks through
the same clipped-pointer helper. This preserves the canonical edit/unit assertion
and adds no workload. It is not evidence that an unexecuted native model or
scientific state was inspected. Current locked-check/format outcomes and any
additional individually selected precision gates remain the implementation
owner's final verification receipt; this reviewer independently ran only
`git diff --check`, which passed.

| Final reviewed source | SHA256 |
| --- | --- |
| `src/bin/desktop/scene_ui.rs` (including five tests) | `68d3eed0bdfd1ee424854325349d3e27391cbdf85b352f4c7acb5a0c7d5a11ec` |
| `src/bin/rheon-desktop.rs` | `32be43dd2ce112e30017b6b9bf706607f942ca574b2415f74237c296f79614c5` |
| `src/bin/desktop/settings.rs` | `72a2e42de38100fa7cb517bfa7c1b9377975d23c3c4cd887615c296549c093c4` |
| `src/bin/desktop/viewport.rs` | `a2ae713e3a06f2370e72300593d208c1991aa201b41e0924cd87268a48a296e1` |

The tests establish the inspected pure-state and egui input paths, not full App
update/worker behavior. Full mesh-import failure rollback, actual active-worker
locking and viewport mode refusal are source-inspected here. The pointer test
reserves at least 300px viewport width at 820px height; it does not assert every
expanded property/control's bounds at 900×650/150% or screen-reader usability.
Native validation must cover those states and the persistent top run strip.
Retained archive integrity, recording completeness and force provenance remain
their existing distinct contracts; the draft scene is not evidence of any of
them and is not persisted by this increment.

## Final retained evidence receipt

Independently read and verified
`../evidence/scene-authoring-20261020/manifest.json`, SHA256
`5b586883c0ecb31124ae8dc4154a4d2302aaa69525bf9a27652b512fb04b6ec6`.
All five source seals, including unchanged timeline test source
`653848ce12986065b966c6ab2d97b23c2e7da4cdf5ec7f5a6b99c1e5a9ef2c35`,
match the working files. All 20 retained logs match their declared byte counts,
SHA256 values and original `/tmp` bytes. Git attributes mark the retained logs
binary with text conversion disabled. This verifies the checked evidence packet's
identity; it is not a signed binary provenance certificate.

The manifest's 14 distinct exact names each occur once in the 177-test list-only
enumeration and each match one retained passing log with one selected pass,
zero failures/ignored and 176 filtered out. Those are five scene tests, four
timeline tests and five material/precision tests, all on the previously inspected
inert helper paths. The raw earlier pointer-caption failure is retained alongside
its corrected pass. No failed log was overwritten or promoted to a pass.

The final retained gate logs show completed Clippy in 52.99 seconds and locked
check in 1.37 seconds. Format and whitespace logs are empty, consistent with the
implementation owner's reported successful exits; empty output by itself does
not independently encode a process exit status. Reviewer reran only read/hash
checks and `git diff --check`, which was clean. No tests/builds/workloads were
executed for this evidence review, and no source seal changed.

The native handoff's five seals match this packet and it explicitly requires
idle-only native inspection; existing separately approved retained/model fixtures
are prerequisites for those interactions, not permission to generate numerical
evidence. Native screenshots and screen-reader behavior remain absent. Actual
model-import rollback, active-worker/progress/Cancel, viewport mode-guard execution,
picking/outlines/handles, and force runtime/archive acceptance remain unverified.
The completed focused gates support this bounded source checkpoint with those
gaps; they do not change the final source disposition or authorize additional
workloads. The review itself performed no commit, push or PR operation.
