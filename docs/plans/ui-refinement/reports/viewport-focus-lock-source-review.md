# Independent viewport focus-lock source review

## Disposition and scope

Suitable for a separate minimal fix checkpoint over `d6e23b36ab618f8873d32c89c67a62e95f775c5b` in `/workspace/Rheon-ui-authoring`, with three bounded exact passes and the static receipts below. The parent relayed native initial-frame stalls. The source establishes a nested context-lock defect, but this review has no native stack trace or qualified attribution proving it caused those native stalls. Actual native picking/handles remain unverified.

Only `src/bin/desktop/viewport.rs` changes production behavior: keyboard ownership is sampled before entering the input callback. The reviewer read source/dependency/inventory files and wrote only this report; no test, build, numerical/model workload, native window or executable was run. Runtime preparation remains separate and is not admitted by this review.

## Lock mechanism and repair

Pinned egui 0.33.3 `Context::input` invokes `Context::write`, holding the exclusive context lock for its callback. Its documentation explicitly warns against context use inside that callback. `Response::has_focus` calls its context's input and memory accessors; `Context::wants_keyboard_input` accesses memory. Calling either inside `Viewport::input`'s input closure therefore attempts another acquisition while the exclusive guard is held. This is a source-proven lock reentry defect on enabled hovered/dragged input, even without a Home/F key event: the keyboard-ownership condition is evaluated first. In debug builds the dependency's lock wrapper may eventually panic after its timeout; this is not normal progress.

The repair computes `response.has_focus() || !ui.ctx().wants_keyboard_input()` before `ui.input`, then uses that scalar inside. Evaluation order/shortcut policy is preserved: Home/F frames the viewport only when it owns keyboard or no widget owns keyboard. Drag, wheel, pinch, modifier, numeric view shortcuts, camera limits and gesture ownership are unchanged. `response.dragged()` remains in the callback, but pinned Response implementation reads a local flag and does not acquire the context; `dragged_by` would differ and is not used there. OrbitCamera orbit/pan/zoom operate only on values.

## Similar callback inventory

Read `/tmp/rheon-ui-context-access-inventory.txt` (SHA-256 `289faffc17a1ffbb6c59133032293478638926bea5674901162c734baf2c1dc4`) and inspected complete current bodies rather than relying on grep context. The inventory was captured before this fix and includes test-only focus queries; it is discovery evidence, not a complete semantic proof by itself.

The eleven production `input` read callbacks inspected are: main timeline speed keys, main close request, main playback delta; AI-comparison close request, playback delta and camera input; viewport camera input; scene-handle pointer/focus snapshot and modifier query; scene selection modifier query; synthetic-wrapper playback delta. The additional scene-handle `input_mut` callback only consumes Escape on the supplied InputState. All remaining closures read supplied input or update plain camera values; no further nested UI/Context/Response context accessor was found. Response focus checks in handle code occur after the tuple snapshot closure ends; speed focus checks precede its key closure. The AI camera helper calls only camera value methods inside input. This is a scoped source inspection, not a whole-dependency deadlock certification.

Settings' custom DragValue formatter queries context memory outside an input callback. The pinned DragValue calls that formatter directly after its separate input/memory reads have returned. Its existing precision/focus behavior is not changed here.

## Exact path admission

Only these three fully qualified exact names are admitted after the 199-name list-only enumeration:

- `viewport::tests::enabled_scene_input_hover_and_keyboard_focus_complete_without_nested_context_lock`
- `viewport::tests::primary_drag_orbits_pans_and_zooms_without_alt`
- `viewport::tests::native_touchpad_scroll_pans_orbits_and_pinches_through_the_viewport_input_path`

Complete helper paths were inspected. Each constructs only a local egui Context and default Viewport, which has default Appearance mode and absent engineering/transport/mesh/frame owners. The new test additionally uses an empty literal SceneUi, a text editor and enabled production `show_scene` composition. Pointer positions target the canvas, not lab tabs; no mode change is requested. Scene handles have no selected force and no drag; scene projection validates empty metadata. Appearance paints an empty display mesh/grid/captions with no emitter, workers, frame, model or Inspector. The other two tests use the test-only `show` delegate with Scene None, then the same enabled Appearance input path. No App, Worker, history, solver/model constructor, recorder/reader, SQLite, filesystem fixture or cleanup is reached. All ownership drops in memory.

The new test covers plain enabled hover, Home framing and F with a focused TextEdit preserving camera distance; it checks lab owners remain absent and the presentation-only guard is false. The earlier tests cover primary orbit/Shift-pan/Ctrl-zoom and point-scroll/Shift-pan/pinch. They are local Context/camera interaction probes, not native GPU/input or force-handle acceptance. No pre-fix local execution/reproduction is proposed; source analysis and final post-fix enabled-path coverage must be described accordingly.

## Scoped external timeout wrapper

Reviewed `/tmp/rheon_ui_exact_timeout.py`, SHA-256 `fac641390156cd82a7554253ca27f300d48473b7c9ab8e986f6a26eef8ae27c1`, before execution admission. It launches one exact child in a new session/process group; uses a 10 s wait, TERM, 1 s grace, then KILL and a final 1 s wait; imposes child CPU 5 s, address space 512 MiB, file size 1 MiB and core size zero; captures stdout/stderr to a fresh raw log and retains a JSON receipt. ProcessLookupError during signaling is tolerated as an exit race. Success requires exit zero, no timeout and an actual one-pass result. Invoke ordinary Python (not `-O`, which would disable its assertions) with one of the three inspected names and fresh output prefixes only.

This wrapper is admitted for these trusted no-spawn/no-fixture helpers, not as a general security/containment supervisor. Resource limits are per process; the group boundary relies on the admitted helper paths, and this does not sandbox unrelated binaries or provision cgroups. Final wait/receipt I/O failure can leave an unsuccessful or uncertain receipt and must not be treated as success. The wrapper does not authorize numerical tests, general descendant workloads, existing artifact deletion or retry after uncertain execution.

## Final source identity and evidence

Reviewed viewport source SHA-256 `ee7acacd33ae29af7ec7f592361861315b30891b1637cb2989ba7390fa8320d3`; wrapper SHA and inventory identity remain as above. No production edits followed the accepted checks. No material source blocker remains for this small repair and the three bounded exact probes.

Read-only receipt of `/tmp/rheon-ui-focus-lock-list1.log`: 199 tests, zero benchmarks. The three exact log/JSON pairs each name their corresponding admitted probe, record one pass with 198 filtered out, exit zero, no timeout and no TERM/KILL action. Captured logs are 225, 190 and 222 bytes; wrapper elapsed times are approximately 0.017 s each. All log SHA/size/result fields were independently checked against their JSON receipts. The current compiled test binary independently hashes to the receipts' `8935179a6738fd137fd2859bd865d8ddb0e1393586dbe366e9922f279902a008`. Readback confirms the three owned child PIDs are absent. This establishes post-fix enabled Context-path completion, Home/editor focus behavior and the existing camera regressions; no pre-fix execution is claimed.

Locked desktop check passed in 1.22 s and strict production Clippy in 2.81 s. The owner confirmed fmt/diff session 31169 completed with exit zero; those logs are empty. Final copied packet `evidence/viewport-focus-lock-20261021/manifest.json`, SHA-256 `568ab94d6fd093753b6f213a26b7850f7cdced18db133942cbc4f5c3364747a1`, was independently verified against current source and all thirteen packet/raw file hash/size pairs, including logs, receipts, wrapper and inventory. It marks native verification and numerical execution false. The reviewer ran no build/test/executable.

Native stall recovery and real picking/handles require the existing native validator; local completion must not be described as native attribution or acceptance. The three pure probes do not exercise a live run, recording/SQLite, model/runtime, numerical solver or worker cancellation, and provide no admission for the separate prepared runtime tests.
