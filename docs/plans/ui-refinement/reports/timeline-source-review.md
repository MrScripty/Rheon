# Independent T1 timeline source review

Reviewed 2026-10-20 against frozen UI-5 tree
`3e1fd57146e2f1a02abe22a09ac317497f39d738` in `/workspace/Rheon-ui-precision`.
Only this report was authored by the reviewer. No tests, numerical/model/native
workloads or filesystem cleanup were executed by the reviewer.

## Disposition

The source supports an **uncommitted, partial T1 checkpoint** for the tested
empty/simple-selected states. T1 acceptance remains open. The 106 px floor is
removed, existing playback controls/units are retained, and the reviewed exact
probes establish a 47 logical-pixel empty strip at both tested widths. This is
not a general all-state layout-safety result. Focused playback-speed editing,
track dropdowns, native interaction and screen-reader acceptance remain
unaccepted, for the concrete reason below.

## Implementation review

`TimelineControls` is a disposable six-scalar paint projection; App remains the
owner of step, track, Follow live, playing, playback time and speed. The shared
contents function preserves the existing transport, scrub, speed and track
handlers, physical-time/calculation captions, particle mass/temperature and
pressure/divergence details. No history/worker/recording/backend owner is copied
or dispatched. `App::update` calls the extracted helper at the original position,
after sidebar navigation and before viewport composition.

The initial floor-only change produced a real cold-frame regression: egui's
bottom panel reserved cached/default height before drawing larger content.
The repair locally measures the same frame/contents with
`UiBuilder::sizing_pass().invisible()` and a separate scalar copy, then reserves
the measured height for the real panel. Invisible also disables input in pinned
egui 0.33.3. Measurement uses a disjoint `timeline-sizing` ID; the real panel
retains `timeline` and `timeline-track`. There is no global discard/replay of
`App::update`, clock advancement, poll, sidebar, history or viewport work.
Only the real control projection is committed once after interactive painting.

The track caption is frozen across measurement and paint; a newly selected track
is projected on the next frame. This prevents measuring one caption and painting
another after input. Default frame/style and available width are shared, and
height is rounded upward rather than imposing a new permanent minimum.

Invisible disabled widgets still register AccessKit nodes. The measurement root
is explicitly marked hidden through the public local node API, whose pinned
contract excludes the root and descendants from assistive-technology presentation
and hit testing. No global AccessKit toggle is introduced in production.

## Complete inert-path inspection

Both fully qualified tests call only local Context/input/paint helpers and
`App::timeline_ui`, with a preceding empty 280 px SidePanel. The selected-frame
fixture directly constructs presentation data with empty density/no reports;
it does not capture or generate numerical evidence.

The full App default chain was inspected, including configured/saved comparison,
archive Inspector state, ExportFile string constructors, collision/material,
emitter/turbulence, camera/viewport and empty particle defaults. These create
values/empty owners. `available_threads` queries OS parallelism without starting
threads. Worker/history/request are None and asserted after every test frame;
App Drop therefore has no worker to cancel/join, nor a fixture cleanup path.
Tests never call update, selected_frame, history/view/capture, viewport rendering,
simulation/model/Worker startup, archive open/export, process execution or file
output. AccessKit is enabled only on the local headless Context, and its new
helper traverses finite output-tree nodes with a visited set.

## Reviewed evidence

The owner enumerated 170 tests list-only and selected only these individual
`--exact` names:

- `timeline_ui_tests::timeline_empty_idle_uses_content_height_after_sidebar`
- `timeline_ui_tests::timeline_controls_retain_transitions_and_physical_time`

The retained before log records 106 px idle height. The floor-only after log
records the first-paint panel-bounds failure; neither is relabeled as success.
Reviewed final `/tmp/rheon-ui-timeline-after-3-exact-{1,2}.log`: one test passes
per invocation. Windows are 900/1250 logical pixels wide, with 280 reserved for
the sidebar, leaving 620/970 for the strip; the headless height is 650. Empty
idle is 47 px tall at both widths. Cold/warm empty, cold simple-selected and
empty-to-selected text/panel/clip bounds are checked. Actual pointer input checks
Step, Play/Pause and Follow live state transitions, with synthetic physical-time
captions of 40 ms simulated and 2 ms calculation time. This does not exercise
continuous playback/history integration.

AccessKit output checks the hidden sizing subtree and exactly one unhidden real
Play/Pause control. This is tree-structure evidence, not a screen-reader session.
Final strict Clippy and affected check logs pass; the owner reports formatting
clean, and reviewer `git diff --check` passes. No additional tests were run by
the reviewer.

## Open source/evidence limits

Disabled sizing renders playback speed as a compact DragValue button. The real
focused control instead renders TextEdit from the real ID's cached edit buffer;
disjoint measurement IDs cannot inspect that buffer. Consequently identical
scalar values/shared contents do **not** prove identical focused-editor geometry.
Longer edit text or the carrier track selector near a wrapping boundary may need
more room than this disabled projection measured. The present exact tests do not
exercise speed focus/edit, long track captions, popup interaction or their
combined geometry. They cannot close that source assumption or establish
all-state measured-height correctness. Record it in the plan/handoff and validate
or repair it before claiming full T1 layout safety.

Native sizes/scales, full application focus order, assistive technology,
representative usability and matched-state word reduction remain unverified.
Force physics/scene selection/property integration and parked live cancellation
are outside this checkpoint. The earlier broad-filter incident retains its
original scope and supplies no evidence here.

## Reviewed identity

| File | SHA256 |
| --- | --- |
| `src/bin/rheon-desktop.rs` | `c0f7b12a2b75b95bb4c70da920b343da36658fc6c766b4f156bf51af88e57695` |
| `src/bin/desktop/timeline_ui_tests.rs` | `12bee2a70ab3b6cb40a73e183e0786d220ed516761c416bafa7ca097c64352fa` |
| Exact allowlist | `9db83063d55f5a5b8db77b30ce2b49f31d0d8235f84b9b231a3e2bc965096fd2` |

The earlier precision source remains unchanged at SHA256
`099fa897a53feee4cb1de483655893f50e6c713c39e66c1a25c8f0549ee2c121`.

## Follow-up: bounded speed editor and four-test source admission

2026-10-20, `/workspace/Rheon-ui-scene`. This follow-up supersedes the focused
DragValue source limitation above for the implementation identified below;
the earlier checkpoint/failure receipts remain historical evidence. Reviewer
performed source inspection only, including the complete four test/helper paths,
and did not execute tests, numerical/model workloads or filesystem cleanup.

The speed control now uses a clipped single-line TextEdit with a 44px desired
inner width, named `Speed` and associated through `Response::labelled_by` with
its visible label; the multiplier unit remains visible. It retains one canonical
App-owned f64 plus transient edit text/invalid state. The local measurement
projection clones those values for its disposable disabled pass; only the real
pass commits App input. Unlike the removed DragValue branch, pinned egui's
TextEdit builder uses the same bounded width with `clip_text(true)` while
focused or disabled, independently of the full edit galley's width. Scrolling
the actual text within that field is deliberate. This is a visual width bound,
not a hard memory limit on the input string.

Only a changed buffer parses into the canonical speed. Finite values retain
the 0.1–4 range through clamping; invalid/nonfinite input retains the previous
canonical value with concise feedback. Up/Down changes the canonical value by
0.1 within the same range. Valid nonfocused text is refreshed from that value;
mere focus traversal does not parse/round the f64. The invalid-feedback visibility
is read before input changes, so both passes paint the same prior-state rows;
new feedback appears on the next paint. The playback speed no longer supports
pointer dragging; editing and the documented keyboard increment are the intended
replacement affordances and need native usability validation.

Two review findings were corrected before the admitted probe run: the initially
unnamed editor received the explicit visible-label association, and the test's
pointer target now intersects the long galley with its clip rectangle. The speed
test verifies the actual focused AccessKit node is TextInput with a label
association before exercising the input. AccessKit tree inspection supports
source/input checks; it does not establish screen-reader acceptance.

All four full test/helper paths are inert:

- `timeline_empty_idle_uses_content_height_after_sidebar`
- `timeline_controls_retain_transitions_and_physical_time`
- `timeline_speed_edit_focus_bounds_validation_and_keyboard`
- `timeline_track_selection_and_report_rows_fit_cold_and_changed_states`

Each is in `timeline_ui_tests`. Context/run helpers reserve only an empty 280px
sidebar and call `App::timeline_ui` directly. App::default's complete owner chain
remains scalar/string/empty-collection construction and absent runtime objects;
its sole OS query is available thread count. Viewport defaults keep numerical
views and the optional scene absent. App Drop joins only `worker: Some`, while
each frame asserts worker/history absent. Nested defaults do not open paths or
perform output cleanup. `RecordingLimits::default` consists of three scalar
limits and allocates no database or recording buffers.

The new Request is a literal native-method vector, scalars and empty strings;
collision mesh, emitter, material simulation configuration, saved/configured
revisions, receipts and synchronization gates are absent. `track_label` reads
only its native method IDs. Literal StepReport/ParticleStepReport/PreviewFrame
values do not construct a solver, capture a numerical frame, initialize history
or render a viewport. Their deliberately combined carrier/particle captions are
synthetic presentation stress, not proof of a reachable physical run state.
Text helpers, shape/AccessKit traversal, keyboard/pointer events and Ctrl+A use
only the local egui context and retained form/presentation values. They never
call App::update, selected_frame, Worker, recording/archive APIs, a model or a
numerical fixture. No uninspected indirect workload or cleanup path was found.

The admitted additions check exact speed bits on focus/Tab, finite edits,
nonfinite/4096-character invalid text, recovery, clamping and ArrowUp; they also
open/select the native SGS track and exercise literal report captions in cold
and changed states at 900/1250px window widths. Separate popup layers are
intentionally outside timeline-panel containment assertions. Configured long
member/provider captions, popup window bounds/keyboard navigation, arbitrary
format/report combinations and continuous playback/history remain unaccepted.
Four exact probes cannot establish general all-state measured-height correctness.
Native focus, actual screen readers and both scales remain separate requirements.

At this source-admission checkpoint, list-only output reports 172 tests; no
runtime result for the new source is claimed here. The implementation owner
will run only each inspected fully qualified name individually with `--exact`
and retain its logs. This review does not authorize a broad filter or additional
numerical/model execution. Earlier exact results apply only to the earlier
checkpoint until current evidence is recorded.

| Follow-up reviewed file | SHA256 |
| --- | --- |
| `src/bin/rheon-desktop.rs` | `89cb033b05499a4504abfb6e6f797ffd8d62fb32f945de43b574545ecab2f928` |
| `src/bin/desktop/timeline_ui_tests.rs` | `c2b2a2f7b5be44963d1495914007b16cdae9df961b17a135c793e63ff1aae824` |
| Preserved UI5 `settings.rs` | `099fa897a53feee4cb1de483655893f50e6c713c39e66c1a25c8f0549ee2c121` |

No remaining blocking source finding for these four bounded probes. This is
source admission with explicit evidence gaps, not full T1/native acceptance.

## Final follow-up: ordinary invisible measurement and exact evidence

The final reviewed projection removes `UiBuilder::sizing_pass()` and retains
`invisible()`. This is a justified correction to the earlier measurement recipe,
not removal of the disabled interaction guard. Pinned egui 0.33.3 makes an
invisible root disabled (`disabled || invisible`) and propagates that restriction
to children. The measurement still uses a disjoint root ID, explicit hidden
AccessKit root, disposable cloned form values, matching frame/width and one real
App commit. It does not replay App::update or introduce solver/history reads.

Inspection of pinned `Ui::new_child` confirms that sizing-pass mode changes
cross justification/alignment. `Separator::ui` separately substitutes zero
available space while sizing. Those differences affect the actual wrapped
timeline's row geometry, so the earlier sizing-pass measurement was not fully
equivalent to ordinary painting. The newly added cold synthetic-report probe
exposed a panel-out-of-window failure with that mode enabled. Ordinary invisible
layout preserves the normal layout rules for these contents and removes that
specific mismatch. It remains an offscreen projection, not a general guarantee
that all disabled and active widget states have identical geometry.

Read retained `/tmp/rheon-ui-timeline-complete-exact-track-1.log`: one selected
test failed its unchanged `panel.bottom() <= 650.5` boundary assertion before
this correction. Read each final exact-result log below: each selects exactly
one named test, reports one pass and 171 filtered out. No test was run by the
reviewer.

| Final log under `/tmp/` | Checked result |
| --- | --- |
| `rheon-ui-timeline-complete-exact-idle-2.log` | Empty cold/warm geometry and hidden measurement tree pass; idle height **48 logical px** at 620/970px available strip widths, in 900/1250px windows with 280px sidebar reserved |
| `rheon-ui-timeline-complete-exact-controls-2.log` | Cold/changed simple selection, physical-time captions, pointer Step/Play/Pause/Follow-live transitions pass |
| `rheon-ui-timeline-complete-exact-speed-2.log` | Focused labelled editor, unchanged speed bits on focus/Tab, edits/ArrowUp, invalid text including 4096 characters, clamping/recovery and checked geometry pass |
| `rheon-ui-timeline-complete-exact-track-2.log` | Native track selection/reset plus cold/changed literal carrier/particle report caption geometry pass at both widths |

The tests were re-read in full after the change. No assertion was weakened to
accept the measurement failure: the panel-bound assertion gained diagnostic
text; only the exact editor payload has the intentional clipping exception.
The fixture/helper and App default/Drop boundary remains inert as described
above. The preserved precision source remains unchanged. Reviewer independently
ran only `git diff --check`, which passed; strict Clippy and remaining static
gate outcomes belong to the implementation owner's final receipt.

| Final reviewed file | SHA256 |
| --- | --- |
| `src/bin/rheon-desktop.rs` | `86f2d65ed296a5dbdf35887a7f1ae4c03eadfb55c58fc8b77ba3ae9e9ca62a7b` |
| `src/bin/desktop/timeline_ui_tests.rs` | `653848ce12986065b966c6ab2d97b23c2e7da4cdf5ec7f5a6b99c1e5a9ef2c35` |
| Preserved UI5 `settings.rs` | `099fa897a53feee4cb1de483655893f50e6c713c39e66c1a25c8f0549ee2c121` |

No blocking source finding remains for this bounded timeline increment. The
four current exact probes close the earlier untested focused-speed/native-track
and literal-report cases within their stated conditions. Native screenshots,
OS-level focus, screen-reader behavior, configured long captions, popup bounds/
keyboard navigation and broader report/state combinations remain unaccepted.
This supports a source-and-focused-test checkpoint; full T1/native acceptance
remains separate. No scene, runtime force, numerical/model or live-cancellation
acceptance is implied.
