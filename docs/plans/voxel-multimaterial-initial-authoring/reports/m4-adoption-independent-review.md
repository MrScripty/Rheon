# Independent M4 staged-adoption implementation review

Review date: 2026-10-22. Reviewer: separate agent `/root/voxel_implementation_review`, distinct from the implementer and capacity-helper author. Reviewed the full dirty tracked production diff, new launch owner/tests and Inspector payload child against admission and the targeted design check. Only this report was written. No tests, builds, native sessions, numerical workloads, benchmarks or external mutations were executed by this reviewer.

**Initial disposition: two bounded required corrections below.** R1 is a mutable receipt alias across retained and pending attempts. R2 is a queued Scene mutation that bypasses Pending's hold. No additional required source correction was found in this complete round. Component acceptance remains pending serial correction and affected checks. Normal launch/rerun exposure, actual native workflow, production saved-source caller accounting and full M4/A5/A6 remain pending.

## Complete required findings round

### R1 — Medium: a cloned next Request can retain the old mutable manifest receipt

Location: `voxel_material_launch.rs::launch_voxel_request`, particularly lines 142–145 in the reviewed snapshot.

The method creates a fresh Arc<Mutex<PathBuf>> only when the incoming manifest_receipt is None. Request::clone shares a supplied receipt Arc. A valid next request cloned from App.running_request can therefore carry the same receipt as the old retained result. The new worker's existing publish_set_revision then writes new intent/started/finished paths through that Arc while Pending still promises the old request/path are untouched. The old request object never moves, but its mutable receipt contents do change. A longer new path can also grow that counted old allocation after retained-capacity admission.

This does not require a data race or malformed recipe: cloning an otherwise valid old voxel Request, giving it a fresh output directory and submitting it uses the current types normally. A new attempt may reach its first progress gate before App polls/acknowledges, so the old request is still retained when the new started receipt is installed. On a no-ack failure the retained request can consequently keep the new attempt's path, contradicting pending_voxel_terminal's old-path preservation claim.

Required correction: make the launch owner establish a fresh per-attempt mutable receipt, or reject every supported alias with retained custody before starting the worker. Unconditionally assigning a new receipt at this private boundary is proportionate and avoids hidden alias assumptions in future callers. Immutable revision/force/mesh Arcs may continue to share immutable payload; this concern is specific to the mutable receipt. Do not create another publication owner or retry any uncertain write.

Add a focused real-Worker pending case submitting a request that initially shares the old receipt. Hold the new worker after publication but before App adoption; require distinct attempt receipt custody, unchanged old path/capacity and the correct new path in pending/terminal reporting. Existing no-ack failure preservation tests should still pass. A fresh output must remain independently no-replace; no fabricated publication uncertainty is needed.

### R2 — Medium: the normal update still services queued contents creation during Pending

Location: `rheon-desktop.rs` immediately after Scene preparation display, lines 2807–2819 in the reviewed snapshot; launch admission guards in `voxel_material_launch.rs`.

Pending replaces Scene controls with show_preparing, but the common update continues into service_draft_file_action and the unguarded create_requested handler. File actions are refused by worker presence, but creation has no such guard. A prequeued create_requested can allocate/install contents and change canonical Scene selection while the old result/Scene is supposed to be held. The launch validator checks active edits/gestures, not queued Scene commands, so it can accept this state. Existing ordinary UI ordering usually drains the flag earlier; that is not an invariant enforced by the new staged API. The implementer independently identified this same queued-command path during review and held corrections for this complete round.

Required correction: refuse incompatible queued authoring/file commands at launch, or explicitly hold their execution for Pending with an intentional later disposition. Also guard the production creation/service boundary so Pending cannot execute a queued authoring command. Do not silently perform it on another document or overwrite preparation/error status with unrelated action feedback. A small local guard/owning helper is sufficient; no general command bus or UI framework is required.

Add a target-bound case with a queued creation command and no existing contents (so creation would actually succeed). Require pre-output refusal or demonstrated Pending hold with unchanged Scene identity/selection/editor, old request/history, playback and admitted payload. Exercise the production update boundary or a narrow production helper used there. A composed Pending Scene/viewport/timeline interaction is useful here: the current held-selection fixture calls sidebar, prepare_scene_presentation and timeline with no events, but never invokes show_preparing plus the disabled viewport path. It cannot by itself qualify those new input guards. Preserve the ability to create/edit normally once Pending has ended, according to the chosen queued-action policy.

## Confirmed staged custody and acknowledgment

The new private launch owner reuses App.worker and the actual existing Worker, with one local pending value containing immutable Request and captured old view. It refuses a live worker, closing, active force/contents gesture and active independent pages. The Appearance query also refuses alternate labs or active viewport handles rather than mutating them to make the request fit. It calculates actual old payload and validates the new request before Worker::start/output; it does not trust the caller's retained_payload_bytes value.

Old App.history, running_request, rows, textures, Scene binding and playback values remain in place during Pending. The captured view is obtained without ensure_timestep, decode or clamp. Sidebar shows pending method/preparation information instead of exposing the old Inspector. Selected-frame and preparation functions return the held view; timeline presents a held caption rather than applying scalar controls. Playback advancement is suppressed while preserving its original values. Scene controls are replaced by a read-only preparation panel, and the viewport CentralPanel is disabled. The picking path now explicitly checks ui.is_enabled before changing Scene selection. R2 is the remaining queued-handler exception to this otherwise coherent freeze.

Poll invokes constructor acknowledgment before finished/join handling, including a fast already-terminal worker. It requires exact frame0, zero time and constructor voxel tag with no legacy/particle/accepted report. Inspector's helper reads already verified raw and typed constructor custody under one lock, checking ordinal/generation/stage/time/dt, absence of interval pressure and the captured initial-input digest. Frame-only, missing typed selection, wrong digest and accepted selection do not acknowledge. This is a bound producer-custody check, not archive authentication or a reconstruction of the numerical state from pixels.

On acknowledgment, the UI thread adopts the matching new history/request/scheduler and known-force binding together, then clears old rows/textures and resets playback/track for the new result. The real worker remains the sole numerical owner. If no acknowledgment exists at join, pending terminal handling retains old fields and reports the new pending target/receipt rather than substituting old run-set path. R1 must ensure those receipt owners are actually independent, not merely differently named fields.

After acknowledgment, ordinary new-prefix semantics apply. Fast completion adopts before join and installs rows using the new request geometry. The voxel branch avoids the legacy late-cancel shortcut, so a flag set after completed work does not relabel its completed archive or suppress its rows. Cancellation/failure before verified construction retains old results; after construction it can adopt the new acknowledged prefix even if the first poll sees an already finished error. Close still cancels and joins through the existing owner. There is no detached worker or append/terminal retry.

The explicit RetainedForceBinding distinguishes KnownNoForces, Unavailable and Authored digest. prepare_scene_presentation recognizes the voxel request and rebinds known absence on later frames, so legacy None cannot downgrade it to unknown. An empty read-only scene is appropriate for this fixed physics; it is not inferred from the current draft. Existing force/model/contents data and validation owners remain separate. Solver formulas, phase transport, pressure, material coefficients and persistence formats are unchanged.

## Actual capacity ownership audit

The new App calculator counts current/retained Scene force vectors/names; viewport cached density even outside history; each relevant viewport/current-draft/retained-Request mesh; history slots/frames; every selected Inspector; retained Request vectors/force snapshots/profiles/receipt path; and result row/pixel/label capacities. Shared aliases may be conservatively counted twice. Unsupported configured runtime handles refuse instead of being treated as empty. Checked addition/multiplication handles aggregate overflow; the latest viewport cached-density and receipt-path additions use checked arithmetic too.

History retains its existing actual slot/frame counting and adds every Inspector's payload. Inspector's owner-local child accounts reader metadata, raw fields, typed voxel, legacy carrier fields/pressure/forces, particle arrays/diagnostics/forces, and configured fields/fractions/pressure plus receipt/provider/runtime text. I inspected the actual variant fields: force objects contain only their name as heap text; provider/configured attempt/executor/deployment strings and boxed provider ownership are included by the corresponding helpers; SpecDifference vectors have no hidden nested dynamic payload. No archive decode, solver constructor, freeze serialization or preset lookup is used to estimate the old arrays. UI-only selector/export/formatting and work-observation metadata remain outside the explicit policy, rather than being silently reported as counted numerical payload.

Native/configured immutable profiles charge actual member/difference/force capacities and relevant strings/boxes. Their spare-capacity fixtures deliberately use incomplete nonrunnable metadata owners only to check accounting, not as simulation qualification. Existing voxel immutable capacity remains conservative. Arc/header and inline double-counting is safe. The canonical RecordedStep::allocated_bytes factors the previous voxel raw arithmetic without changing it; decoded voxel accounting factors the same f64 capacity sum and is still enforced at decode. No record schema or physics formula changed.

The 384 MiB/four-new-publication/128 MiB reserve policy is preserved. It includes old density separately from the inherited face/mesh reserve. GPU/egui/process/SQLite/allocator internals, inactive independent pages and stated UI-only metadata remain outside it. This is source/capacity admission, not total memory/RSS, allocator-peak or measured performance proof. The 240 MiB spare pixel-capacity fixture exercises actual Vec reservation and production refusal; it does not establish physical resident usage. A future saved-voxel launch UI must still account its related source revision/reader rather than calling it unrelated; no such normal caller is introduced here.

The test-only pre-constructor failure gate supplies deterministic ordering for the existing Worker fixtures. It adds no production branch or numerical algorithm. The numerical constructor may still be allocated after a cancellation flag is raised at that gate, before the existing pre-output check; the evidence is pre-output cancellation/old-result retention, not a claim of allocation-free instant cancellation.

## Executed evidence and limits

Read-only inspection of the implementer's logs shows six new staged App/Worker fixtures passed and four Inspector payload/acknowledgment fixtures passed. Those include held old selection, actual constructor/prefix adoption, fast completion with late cancel, preconstructor cancellation/output failure, real spare-capacity refusal, selected-history accounting and after-constructor cancellation. The owner fixtures exercise raw/typed/digest/accepted acknowledgment negatives, native/configured spare metadata, legacy raw/particle capacities and poisoned-lock refusal. New Scene known-no-forces and cached-viewport tests are present. These are implementer executions, not reviewer workloads.

The parent reports affected regressions passed: core recording 24; desktop voxel 39 plus explicit child 1; Inspector 8; History 6; Scene 10; viewport 11; timeline 5; setup 9 plus 1 pre-existing ignored harness; force UI 7; configured Worker 6. Initial source compilation exposed a mistaken constructor material_color check (voxel previews legitimately use Some too), which was corrected before these deciding runs. Strict Clippy found a routine boolean simplification in poll, still pending serial cleanup at this report boundary. No success is predeclared for the upcoming correction snapshot or strict checks.

The current passive pending fixture does not demonstrate hostile pointer/key input through the new Scene/disabled-viewport composition; R2 asks the targeted owning regression rather than treating the old native idle or widget suites as that evidence. No actual native launch/save/load/inspect/cold rerun, whole-feature acceptance, physical GPU, numerical accuracy or performance result follows from this source review. Normal contents launch and saved voxel rerun remain unavailable. Main and unrelated work remain untouched.

## Source identities at complete findings round

| Path | SHA-256 |
| --- | --- |
| `reports/m4-adoption-admission.md` | `0de1aa42b82389c77b7f9ebf2167e34bbe37dd0a1aff240e3cb06b3aa55b74df` |
| `src/bin/desktop/voxel_material_launch.rs` | `692ab12b865ed2e693e3990fb0a8043d3a328096a81355f4cd9977b02b715091` |
| `src/bin/desktop/voxel_material_launch_tests.rs` | `7839cc3d421b9fd70b7de09d9a886ff9913493806176611709b445536cadf413` |
| `src/bin/desktop/inspector_payload.rs` | `fce0efe715f9f7ca4686edcf544bc1dad775978826d5364697f96eea9aef0990` |
| `src/bin/rheon-desktop.rs` | `191c12ecbd5b36e2fedfd1ff077080d1f02c49ce74d907cb23b08303c79497cb` |
| `src/bin/desktop/history.rs` | `45c3525b62ecee6725c47bdc62625ff6dcb05c60e062f278f17a54b295e83d34` |
| `src/bin/desktop/inspector.rs` | `cdadece77dc351fb3fbf6011427366d37ac333a02ae5ac1f155619df6f0cdded` |
| `src/bin/desktop/scene_ui.rs` | `a3443d7484d7437f8dc6dfc6d4648266485fb03c2bc7c7bafcc5da782e038c2f` |
| `src/bin/desktop/viewport.rs` | `0b802cd14ad8800c37789c595f660e0d00a5009e9832ace751d61c271fab51d1` |
| `src/bin/desktop/run_sets.rs` | `140ac4f4def798cd4ce2bb03c96e4b85534cc5adc64c3fb825c5e7b939ae61ff` |
| `src/bin/desktop/run_sets/configured.rs` | `7ea676927f5c84543b2f4448ea29b69b39cc45ce0770962478d531185389fff3` |
| `src/recording.rs` | `d2965dbc52042cb2df2f02c45baa8e895fa8b26908ad1e8c468603953c0e2d92` |
| `src/recording/voxel_material.rs` | `281873f0f6ed550fa8afcda76ecb25725c6a658836c29f7f98d00532add64a24` |

These identify reviewed bytes before serial corrections, not executed certification. Only this report was written. No reviewer action was denied, pending or uncertain. Previously denied external actions and uncertain Library uploads were untouched. Parent/implementer owns the complete correction round, affected verification and admission, followed by targeted independent rereview. This report grants no external publication or shared-main integration permission.

## Targeted R1/R2 correction rereview

On 2026-10-22 the same separate reviewer inspected the serial correction round, new actual Worker/queued-input fixtures and corrected execution logs. Only this report was appended. No test/build/native/numerical/benchmark workload or external action was performed by this reviewer.

**Current independent source-review disposition: R1 and R2 are resolved. No further required correction was found in this targeted scope.** Parent/implementer retains component admission ownership after affected verification. This does not accept normal launch exposure, a future saved-source caller, actual native workflow or full M4/A5/A6.

### R1 — resolved

The launch owner now assigns a fresh per-attempt receipt unconditionally before Worker::start. The worker and pending Request share that new receipt, while a supplied incoming alias is discarded from new attempt custody. Immutable revision/mesh/force Arcs remain shared appropriately. This preserves old receipt contents/capacity during Pending and prevents a new publication from changing the old request's path through a hidden mutable Arc alias. No publication format, append/retry owner or captured numerical policy changed.

The added real-Worker fixture initially clones the old running Request and proves its incoming receipt aliases the old Arc. It launches with a fresh output target, holds the worker after the first accepted publication while App is still pending, and requires distinct new receipt custody, unchanged old path/capacity and the actual new started.json path. After cancellation/final poll it requires the new finished path and the old Arc still unchanged. This targets the alias rather than merely comparing App's top-level fields. Existing preconstructor old-path preservation tests remain intact.

### R2 — resolved

Launch refuses queued contents creation or file actions before Worker/output. The production update now calls one narrow `service_initial_contents_commands` owner instead of separately servicing files and unconditionally creating contents. During Pending or a live Worker it consumes stale commands, records a local io_feedback refusal and returns without executing, creating a document or replacing run status. Idle behavior reuses the original file action and creation path. The policy explicitly requires the person to issue a fresh command in idle Draft; it does not silently replay an old command against another document.

The added fixture uses an actual retained legacy carrier result with no existing contents, so unguarded creation would succeed. Queued create and Load each refuse before output with old Scene/history/payload unchanged. A real new Worker is then held before constructor, and stale create/Save are injected at the actual production command boundary. The composed egui path paints show_preparing, the held timeline and a disabled real viewport; pointer press/held/release and Ctrl+Z are sent, including targeting the actually painted Particles tab. The Appearance query remains valid, no contents or selection appears, commands are consumed with local refusal, run status stays intact and old request/history/playback remain paired. After cancellation, a newly issued idle-Draft creation succeeds through the same production owner. This closes the earlier passive-fixture gap without introducing a generic command platform or native-window claim.

The negated boolean in legacy/voxel cancellation selection was simplified equivalently for Clippy. Moving the force-capacity helper before the test module is an item-order change. Neither changes physical evolution, persistence semantics or the reviewed payload sums.

### Executed evidence and final identities

Read-only inspection of `.generated/voxel-material/m4-adoption-corrected-tests.log` shows all 8 staged-adoption fixtures passed, 0 failed. The final strict desktop all-target Clippy log completes successfully. The broader affected baseline results remain recorded above; the parent was executing affected corrected-snapshot regressions at this boundary, so their outcome is not predeclared here. These are implementer executions, not reviewer workloads or native/scientific/performance acceptance.

| Path | Final SHA-256 |
| --- | --- |
| `reports/m4-adoption-admission.md` | `0de1aa42b82389c77b7f9ebf2167e34bbe37dd0a1aff240e3cb06b3aa55b74df` |
| `src/bin/desktop/voxel_material_launch.rs` | `6142e9e7b655c44e31ea3ed85b00ace82073d47d314a3e2b37a97c90b2078eed` |
| `src/bin/desktop/voxel_material_launch_tests.rs` | `9f42a7f12ffb54d022df82237b5ebfd084d393eedcce066828c1236ecbe599c5` |
| `src/bin/desktop/inspector_payload.rs` | `fce0efe715f9f7ca4686edcf544bc1dad775978826d5364697f96eea9aef0990` |
| `src/bin/rheon-desktop.rs` | `21cb7fa689cb1a381230bc9d2d023faf79e6bebd79a9ec01ab5e2e7d01db3591` |
| `src/bin/desktop/history.rs` | `45c3525b62ecee6725c47bdc62625ff6dcb05c60e062f278f17a54b295e83d34` |
| `src/bin/desktop/inspector.rs` | `cdadece77dc351fb3fbf6011427366d37ac333a02ae5ac1f155619df6f0cdded` |
| `src/bin/desktop/scene_ui.rs` | `196f24054564a00d391bb6b1cf569e8753416d39d10ded00e67f433c78131b51` |
| `src/bin/desktop/viewport.rs` | `0b802cd14ad8800c37789c595f660e0d00a5009e9832ace751d61c271fab51d1` |
| `src/bin/desktop/run_sets.rs` | `140ac4f4def798cd4ce2bb03c96e4b85534cc5adc64c3fb825c5e7b939ae61ff` |
| `src/bin/desktop/run_sets/configured.rs` | `7ea676927f5c84543b2f4448ea29b69b39cc45ce0770962478d531185389fff3` |
| `src/recording.rs` | `d2965dbc52042cb2df2f02c45baa8e895fa8b26908ad1e8c468603953c0e2d92` |
| `src/recording/voxel_material.rs` | `281873f0f6ed550fa8afcda76ecb25725c6a658836c29f7f98d00532add64a24` |

No reviewer source change or workload was performed. No action was denied, pending or uncertain; historical external denials and uncertain Library uploads remained untouched. Corrected component admission and all original exposure/integration gates remain parent-owned.

At the final report boundary the corrected affected checks also completed: desktop voxel 41 passed with the explicit fresh child passing separately and two child-only ignored entrypoints retained; Inspector 8, Scene 10, viewport 11, timeline 5, setup 9 plus its pre-existing ignored harness, and contents input 10 passed. Final formatting and strict all-target desktop Clippy passed. The unchanged baseline recording 24, History 6, force UI 7 and configured Worker 6 results remain the earlier affected evidence; they were not unnecessarily repeated. The final logs were inspected read-only. This closes the pending corrected-check statement above without adding a native/performance/whole-feature claim.
