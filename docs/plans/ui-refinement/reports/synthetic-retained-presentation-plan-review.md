# Independent synthetic retained-presentation plan review

## Disposition

Conditionally suitable as a separately reviewed, test-only presentation harness.
No source evidence requires a worker, archive reconstruction or numerical state
to exercise the proposed timeline. Keep the layout/evidence and containment
conditions below in the implementation. This admits construction and source
review; it does not accept a harness that has not yet been written or executed.

## Reviewed source and limits

Worktree `/workspace/Rheon-ui-authoring`, HEAD
`8504b3e7c9e6a02a6e2521b54c1aefc9c0f58ec2`, with T4 production changes locally
modified. Reviewed `reports/synthetic-retained-presentation-contract.md`, SHA-256
`7983ab9a9a5953aa5ae3a8a6b02c6099e03941764f7a65f22e29cb5087c2451d`,
against `Request`, `validate_request`, `App::default`, `selected_frame`,
`scene_timeline_ui`, `timeline_ui`, `App::drop`, history construction/publication,
default Inspector selection, scene composition, Appearance routing and existing
isolated timeline test composition. No harness implementation existed at the
initial review; the entry-point refinement below records the later source check.

No build, test, native window, screenshot, workload or archive operation was run.
An attempted read of dependency `egui` response/UI source beneath
`/root/.cargo/registry/src` was denied by filesystem permissions. It was not
retried. The review therefore does not certify dependency-level disabled-widget
input semantics; exact local Context probes and the existing native validator
remain required. This does not block review of the application's entry points.

## Conditions and evidence fidelity

| Priority | Condition | Required evidence |
| --- | --- | --- |
| Medium | Match available production layout width and style. Omitting the production sidebar creates different timeline wrapping. | Reserve an inert 280 px left panel where production would show setup, without calling `sidebar`; use the actual open-Scene/narrow layout decision and panel order. Apply production dark style and 8×7 item spacing. If an alternate composition is retained, label its screenshots harness-only and exclude production wrapping/compactness claims. |
| Medium | Literal metadata must remain visibly synthetic. Existing timeline captions still say “Simulated” and “Step calculation” with absent reports. | Keep the banner persistent in every captured state; explain that timestamps/densities are supplied literals and zero calculation duration is unmeasured. Do not interpret those captions as execution or timing evidence. No production copy change is necessary for this harness. |
| Medium | Disabled presentation UI must be a containment boundary, not merely an instruction to the operator. | Construct a fresh default Appearance viewport, keep its entire composition disabled every frame, with no user-accessible mode setter or Inspector UI. A narrow `cfg(test)` Appearance reset/inert-state query is allowed. Inspect complete rendering/helper paths and probe attempted tab/canvas/key input in local Context. No `App::update`, sidebar, polling, run helpers or archive/saved view may be composed. |
| Low | Publication and default App state otherwise choose live/latest history rather than the intended first frame. | Explicitly set `follow_live=false`, track/index/play time and paused initial state. Publishing track 1 last updates `active_track`; do not rely on defaults. Check each frame's method, ordered step/time, exact edge/density length and distinguishable literals. |

## Ownership and containment

The only new application file is the proposed test module, plus a root
`cfg(test)` module declaration/main and narrow test-only viewport guard/query.
These additions must stay test-only. The original proposed ignored native test
is superseded by the standalone test-configured main below; the ordinary probe
uses only construction and local egui Context frames. No production launch flag,
default startup change, runtime adapter, force integration or archive
reconstruction belongs here.

The proposed request is legal metadata: two distinct pressure method identifiers,
size 2, one requested step, one thread, 1 m domain, 0.25 s duration, exact preview
edge 2 and 16 MiB planning budget. In the specified absent-material/emitter/model/
configured/saved case, `validate_request` performs limit/geometry/history/buffer
arithmetic without constructing a simulation. `LiveHistory::with_resolution`
reserves bounded preview metadata storage; four directly constructed eight-byte
densities are sufficient. Do not use `capture`, `capture_state`,
`capture_particles`, timing helpers, scheduler construction or any run helper.

Default history Inspectors contain no reader. `selected_frame` reaches
`ensure_timestep` and then returns absent-recording feedback before any read. Keep
that state: never reload an Inspector, attach a reader, compose export controls or
interpret absence as verified recording completeness. Request recording path is
an unused sentinel string, not an artifact destination. No app-owned recording,
export, screenshot, trace or persistence output hook may be added. External native
validator screenshots remain separately owned evidence; normal display-system
initialization is outside this application-data I/O assertion.

## Presentation and lifecycle contract

Use the existing selection/timeline methods with one canonical App/SceneUi owner.
Scene context changes pause playback; while Draft, neither hidden selection nor
play time advances. Returning to Retained preserves the retained index. Apply
bounded stable delta only while Retained and playing, and let existing end-of-track
behavior stop playback. Track changes, slider/step actions, invalid speed editing
and Play/Pause are actual timeline interactions; disabled viewport gestures,
force handles and worker cancellation are outside the evidence.

Render the persistent synthetic label and Close control outside the disabled
canvas. Initialize one monotonic 300-second deadline; interaction must not extend
it. Request repaint while idle so self-close is reached, and keep external native
process supervision. No custom teardown should call run, join or I/O helpers;
worker remains absent, so existing `App::drop` has nothing to cancel/join. App
ownership ends normally when the wrapper closes. Do not add unconditional process
exit or cleanup/deletion logic.

## Acceptance and remaining gates

Review the complete new construction/render/helper path before list-only
enumeration and a single fully qualified exact ordinary probe. Check literal
request/history shape, no worker/scheduler/reader/runtime intent, intended initial
selection, Draft pause/index retention, track/time display and disabled lab-mode
attempts. Compile the standalone native entry without running it here. Source review,
format/lint/build evidence and exact invocation/patch seals precede transfer.

Only the existing validator may run the sealed standalone entry under its authorized
display session. Capture exact source revision, window size/scale, synthetic
banner, Play/Pause/speed/track/slider and Draft↔Retained results, plus Close/deadline
outcome. Such evidence can support synthetic timeline presentation and its
composed layout; it cannot establish normal App startup/sidebar integration,
viewport interaction, runtime/recording/archive parity, force provenance or any
scientific behavior. Rollback is deletion/reversion of this separate test-only
increment, preserving T3/T4 production work and existing native evidence.

## Standalone main-thread entry refinement

Later read-only source check: HEAD
`425a9869fa3f36b7e3133ddbd213a493d7b5bab3`, with the harness changes still local.
The source places the unchanged normal main body behind `cfg(not(test))` and adds
a `cfg(test)` main that calls `run_native_presentation`. The new native function
has no test attribute. This is a proportionate way to enter eframe on the process
main thread without a winit dependency or cross-thread extension trait.

Standard `cargo test` uses its generated test runner; it must never launch this
native main. The new module should enumerate exactly one ordinary pure Context
test and no native test entry. Existing unrelated tests still exist in the
binary's test suite: enumeration is not permission to execute them. Keep the
individual fully qualified exact-test workflow.

The standalone compile gate is `cargo rustc --locked --offline --features desktop
--bin rheon-desktop -- --cfg test`, without `--test`; success has not been checked
by this reviewer. Use a dedicated target directory for this recipe, or immediately
seal a uniquely named copy before any normal build can replace it. A standalone
test-configured artifact shares the ordinary binary name and must not be
mistaken for a production build. Record source revision/patch seal, full recipe,
toolchain and executable hash. The existing validator executes only that sealed
path, under separate native authorization; no execution is admitted here.

The build gate must prove this test configuration compiles without adding Cargo
dependencies/features or broad lint suppression, and an ordinary non-test build
must retain the original main behavior. `cfg(test)` also includes other test-only
modules/helpers; their presence is not a workload, but the new main must call
only the inspected presentation construction/update path. Do not use a test-suite
invocation or application `--run` flag to enter native presentation.

At this check the contract still described the superseded ignored test. Align
that contract before transfer. The implementation also read the test-only
`RHEON_PRESENTATION_SIZE` environment variable, whereas the latest delegated
refinement specified no environment switch. Remove that selector and resize the
native window normally, or explicitly admit that narrow size selector in the
contract before transfer. This is an isolation/documentation condition, not
evidence of a runtime or physics entry. No new native/build/test result is claimed.
