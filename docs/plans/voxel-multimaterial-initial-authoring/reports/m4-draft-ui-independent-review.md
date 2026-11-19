# Independent M4 draft file interaction review

Review date: 2026-10-22. Reviewer: separate agent `/root/voxel_implementation_review`, distinct from the implementer. Reviewed the complete dirty production write set (`initial_contents_ui.rs`, `scene_ui.rs`, `rheon-desktop.rs`, new `voxel_material_authoring.rs`), new file/widget/Worker fixtures, admission and reused DraftRevision/Scene/viewport input owners. Only this report was written. No build, test, simulation, native session, benchmark or external mutation was executed by this reviewer.

**Initial disposition: two bounded required corrections below.** R1 preserves another Scene owner's validation feedback. R2 closes input ownership at the queued Load→viewport boundary. No further required source correction was found in this complete round. This is a native draft-file interaction component, not implementation or acceptance of launch, retained-result adoption, cold native rerun, M4/A5/A6 or a general document system.

## Complete required findings round

### R1 — Low: contents adoption clears force-owned validation feedback

Location: `SceneUi::adopt_loaded_contents`, `scene_ui.rs:263–266`.

The new method correctly checks contents identity and installs a fresh editor, then assigns `self.error = None`. That Scene error is also populated by force validation and failed Scene operations. Loading a contents file does not repair an invalid force support/name/value or resolve an earlier force/model error. The new contents operation can therefore hide unrelated validation feedback while leaving its owning data unchanged. The implementer independently observed this same concern during the review; no correction has been applied in the reviewed source.

Required correction: preserve force/model-owned Scene feedback during contents adoption. Contents UI has its separate error/io_feedback owner and can reset its own brush feedback. Add a focused case retaining an existing invalid-force validation message/data through a successful noncolliding load. Do not clear/revalidate unrelated authoring state merely to make the error disappear; do not add a new diagnostic framework.

### R2 — Medium: queued successful Load does not own the remainder of its input frame

Locations: `voxel_material_authoring.rs::service_draft_file_action`, `initial_contents_ui.rs::install_loaded`, App update immediately after Scene controls, and existing `InitialContentsUi::canvas`.

App services the queued Load before it renders/processes the viewport. A successful install clears gesture/latch and replaces identity, cells and undo history, but the same egui input batch remains available. The viewport canvas later scans the full raw PointerButton event list rather than only unconsumed events belonging to the new document. There is no document-adoption/terminal-frame guard. A queued Load accompanied by earlier/coalesced primary input can consequently let that batch begin or commit a stroke against the just-installed cells, instead of retaining the exact loaded revision until a fresh gesture. Final primary-down tails can similarly inherit ownership after the replacement. Idle checks correctly protect an already active gesture, but cannot detect a press whose viewport handler has not run yet in this frame.

The current file-control fixture uses an isolated panel, calls service separately, and never composes service with the viewport canvas. It cannot establish this boundary. Prior M3 cancellation/release safeguards should not be bypassed simply because installation resets the contents UI state.

Required correction: treat successful Load as a terminal input handoff for its actual service frame, consume that entire frame before brush/undo handling and retain the existing primary-release latch when the discarded tail remains down. A local frame marker plus existing latch is sufficient; no generic input manager or global event clearing is needed. Direct restore/adoption APIs can continue to install fresh state; the actual App service must mark the real input frame. Do not prevent a genuinely fresh later press once the discarded tail has released. Failed loads must leave old input/editor ownership intact unless an explicitly defined cancellation boundary applies.

Add a composed real-egui file action→App service→canvas fixture covering coalesced press/release and a held primary tail. Require exact restored IDs/revision, fresh undo/redo, no camera/force mutation, tail suppression through release and successful fresh brushing afterward. It must exercise the owning service/input path rather than directly invoking `install_loaded` or assigning a synthetic latch. This is widget/input evidence; it does not substitute for native-window qualification.

## Confirmed atomic file and identity behavior

The App action first refuses a live Worker, non-Draft context, force edit, contents gesture/latch or active editor transaction. The service takes one queued action, executes it once, and stores feedback/status; a later frame does not automatically retry it. UI buttons also avoid replacing an already queued action. Empty/overlong path refusal precedes filesystem access, with the App checking bytes rather than relying only on TextEdit's character limit.

Save captures exactly once through the existing DraftRevision owner. It does not call undo/redo, rebuild the editor or change revision. No-replace publication protects existing files. PublicationUncertain stores the exact attempted path and pauses further session Save actions, including a different target. The existing durable owner supplies stage/error text; service preserves it as feedback. Load remains read-only with respect to that target and `install_loaded` does not clear the uncertainty latch. The fixture explicitly simulates the reaction to an existing uncertain state; it does not fabricate an actual filesystem failure, reconciliation or confirmed publication. Session-only uncertainty is not presented as a new crash journal/recovery platform.

Load reads, validates digest/schema/palette/IDs and restores a fresh editor before changing App or Scene. The native-authoring gate requires cubic equal-spacing 4–64 counts, physically zero origin, side 0.001–100 m and the existing 1000 cells/m limit. This is deliberately narrower than generic saved inspection and immutable worker replay. No resampling/rebinarization or scalar reconstruction of the restored document occurs. The App's scalar side is only the presentation product `h*n`; original spacing/origin bits remain in the editor. Vacuum remains legal draft data and does not acquire solver-launch permission by loading.

Identity admission checks current forces, retained Scene forces, current/retained model IDs and all retained Request force tracks before installation. The Request check matters for a force in a track other than the currently bound retained Scene. A same-ID replacement of the previous contents document is allowed; unrelated force/model identity reuse is refused. The allocator advances to at least loaded ID+1 without reviving an exhausted allocator; loading MAX exhausts it deliberately. Those are coherent ownership semantics, not a new global identity service.

After checks, Scene installs the fresh contents editor and selection, then App assigns scalar counts/side and clears only the draft's numerical-density selector. No fallible operation follows that adoption sequence. A failed read/geometry/collision/busy admission leaves the original editor, revision, undo/redo, geometry and selection installed. Successful load intentionally resets contents-only history/brush state and selects loaded contents. R1 is the unrelated-feedback exception to preserve; R2 is the subsequent input-frame exception to close.

Unrelated force/model values are retained rather than transformed to the loaded domain. Retained Request, history, rows/textures, playback fields and archive owners are not assigned by this action. Loading a draft does not alter any old numerical state or retroactively mutate its accepted intent. A later supported launch needs its own readiness checks for the changed draft. No launch is introduced here; the contents run button and saved-voxel rerun remain unavailable.

## UI and evidence assessment

The panel uses a concise labelled local-path field and Save draft/Load draft controls. Its text states that Save needs a new file and Load replaces contents/domain and clears contents undo. Load is available without an existing editor, while Save requires one; busy/editable and uncertain-save states disable the appropriate controls and are revalidated in App. The path TextEdit participates in existing text-focus undo priority. Global shortcut/force-selection routing was not replaced. A default collapsed file section keeps the brush panel proportionate.

The implementer's first six fixtures passed after a test-only accessibility lookup correction. The original widget fixture tried to find an unchanged label in a sparse AccessKit release update; the corrected fixture uses the actual painted existing path rectangle, clicks it, sends Ctrl+A/Text and clicks the real Save/Load controls. It does not programmatically request focus. Source still associates the path label through `labelled_by`. The earlier failed attempt is evidence history, not a product/native failure or a reason to weaken the intended interaction assertion.

The fixtures meaningfully exercise actual App file actions/local filesystem, redo-preserving save and no-replace bytes, exact non-dyadic/signed-zero loaded data, fresh history, retained history Arc/playback preservation, collision/refusal, maximum-ID exhaustion and persistent session uncertainty. Two further fixtures now on disk exercise an actual existing Worker held at its first-progress gate and all retained Request force tracks. The current log snapshot ends with a test-only `unwrap_err` Debug-bound compile error for the new Worker fixture; the implementer reports replacing it with `err().unwrap()` without changing production semantics. Their final execution is pending at this report boundary. No success is predeclared for that corrected snapshot or for R1/R2 fixtures not yet written.

The new retained-Request fixture creates a retained binding for validation, without claiming it was produced by that request's solver. The live-Worker fixture uses the real bounded 4³ existing worker and local SQLite, not a new numerical/performance cohort. Neither establishes actual native Save/Load, an uncertain publication fault, cold rerun or whole-feature result adoption. Required affected formatting/lint and existing contents/Scene/setup/input regressions remain implementer-owned after serial corrections; this reviewer ran none.

## Scope and reviewed identities

The production changes stay within the admitted file/UI/Scene/App-service boundaries and reuse DraftRevision's unchanged format. No worker/solver/recording numerical method, dependency, README/index, unrelated task or main change is introduced. The remaining actual native file/run/inspect/cold workflow and production retained-resource/adoption gates remain separate. This review grants no push, PR, merge, deployment or external publication permission.

| Path | SHA-256 at complete findings round |
| --- | --- |
| `reports/m4-draft-ui-admission.md` | `c2c92d78705d1ccc737a92086e9c9efe728f0e90cc75bae46f40b17654e1a030` |
| `src/bin/desktop/voxel_material_authoring.rs` | `4e0f281bb5e682b7f13214ade0bffb84bce96d5967ad88b555abf1c9f1a803ff` |
| `src/bin/desktop/voxel_material_authoring_tests.rs` | `06879b3aaf82b1d1f58aafc85c98b06d67663dd84122d49f3d0d57a395bec049` |
| `src/bin/desktop/initial_contents_ui.rs` | `1701e5356ded88acfbccdc77ce768c71bd0160f5d174b503ff3b69ad886a8642` |
| `src/bin/desktop/scene_ui.rs` | `7cc02b4164b540df83a5611468173829c1c222a35ba10b2f1e9719640ddfb12d` |
| `src/bin/rheon-desktop.rs` | `bb154f31ff6cf1d629179d3293a2e2c698dd4f173a01daf34a7e687cdba277d7` |

Only this report was written. No reviewer action was denied, pending or uncertain. Historical denied operations and uncertain Library uploads remained untouched. Parent/implementer owns the complete serial correction round, affected execution and component admission, followed by targeted independent rereview.

## Targeted R1/R2 correction rereview

On 2026-10-22 the same separate reviewer inspected the serial corrections, composed input fixture, force-owner fixture and final implementer logs. Only this report was appended. No build, test, simulation, native session, benchmark or external mutation was executed by this reviewer.

**Current independent source-review disposition: R1 and R2 are resolved. No further required correction was found in this targeted scope.** Component admission remains parent/implementer-owned. This closes the bounded draft-file review; actual native file/run/save/load/retained-result workflow, production launch adoption and full M4/A5/A6 remain pending.

### R1 — resolved

`SceneUi::adopt_loaded_contents` no longer clears Scene.error. The editor installation resets its own brush error while force/model diagnostic ownership stays intact. The added Scene-owned fixture creates a force, sets its vector nonfinite, invokes the actual Scene validation/freeze path, records the resulting feedback and data, then loads a noncolliding saved contents file through the real App action. It requires unchanged force values and the same nonempty error alongside the new contents ID. The test's earlier assumption that an empty force name was invalid was corrected to a genuinely invalid nonfinite vector; the validator was not changed to satisfy the test. This is the appropriate owner-level evidence.

### R2 — resolved

Actual App service now receives the real egui Context and stamps a successful Load with the current cumulative frame number and final primary-down state. It stamps only success; failed load and Save do not alter input ownership. Direct loaded-editor installation still resets to a fresh editor/UI transaction state. The canvas checks the load-frame marker before any brush, shortcut/undo or gesture logic, consumes that invocation, paints only the restored view and retains the existing release latch if the discarded tail remains down. Later held movement/release therefore cannot become a new stroke or fall through to camera input. A genuinely fresh later press remains usable.

The composed fixture renders the real Scene controls and viewport, queues Load by clicking the actual button, then deliberately delays servicing to exercise a pending action alongside a raw coalesced input batch. It invokes the actual App service and viewport canvas in their owning order. Both a full press/release batch and a held-down tail require exact loaded revision/cell counts, empty undo/redo, unchanged camera and untouched forces. The held case also checks movement and release preserve the loaded revision, followed by a fresh brush that succeeds. It neither directly installs a latch nor programmatically assigns focus. Querying the current painted button/canvas rectangles on each loop corrected a stale-layout test coordinate after feedback appeared; the restoration/gesture assertions were retained.

The implementation uses a local frame marker and existing latch, without removing global events or adding another input manager. I also inspected the installed egui 0.33.3 Context.run/cumulative-frame implementation while checking the marker boundary; no additional required correction was found for this App's one-canvas path. The composed test deliberately stresses the queued service boundary rather than claiming a native-window gesture receipt.

### Executed evidence and final identities

Final read-only log inspection and implementer completion report agree: draft authoring 9 passed; Scene 9 passed; contents input 10 passed; viewport 9 passed; setup 9 passed with 1 pre-existing ignored harness; force UI 7 passed; strict desktop all-target Clippy completed successfully. Those are implementer-executed results, not reviewer workloads. The draft fixtures now include the actual bounded Worker refusal and all-track retained Request conflict/preservation cases whose earlier test-only compile failure was recorded above. No numerical/performance/native acceptance transfers from those small component fixtures.

| Path | Final SHA-256 |
| --- | --- |
| `reports/m4-draft-ui-admission.md` | `c2c92d78705d1ccc737a92086e9c9efe728f0e90cc75bae46f40b17654e1a030` |
| `src/bin/desktop/voxel_material_authoring.rs` | `eab322f20b7f17033dea43acd422a799fce6cfb99cd53a90c434a05e7aac5daf` |
| `src/bin/desktop/voxel_material_authoring_tests.rs` | `9a4adc0dcd4a0ec2280cf80eb2064b8f93c4c9fc0d9df54dafa77a179cf1d858` |
| `src/bin/desktop/initial_contents_ui.rs` | `0829ee860fae6116b0d49c7e013529f6db52e2ff274703faf666a02085668ac4` |
| `src/bin/desktop/scene_ui.rs` | `863b410df4d99230dc65f7bf6a89227a1bc32e7137ad5830ed175aca55c8f089` |
| `src/bin/rheon-desktop.rs` | `3bab0198375310e9fd328ed2af138efa4b2392b1c246fca6d066d41307799fe5` |

Only this report was changed by the reviewer. A read-only cache-location lookup included `/root/.cargo/registry/src/*/egui-*/src/context.rs`, which was unreadable due to filesystem permissions; that path was not retried. The existing dependency file identified the readable installed source at `/workspace/.voxel-cargo/registry/src/index.crates.io-1949cf8c6b5b557f/egui-0.33.3/src/context.rs`, so review completed without an outstanding blocker or permission escalation. No action is pending or uncertain. Previously denied external operations and uncertain Library uploads remained untouched.
