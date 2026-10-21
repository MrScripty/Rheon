# Independent synthetic retained-presentation source review

## Disposition

Suitable as a separate test-only synthetic presentation harness, with the evidence limits below. The inspected construction and composed UI paths do not start a worker, numerical/model workload, recorder, child process or archive read. This is source acceptance and a receipt for the narrowly executed pure Context probe; native presentation and interaction acceptance remain pending.

Reviewed in `/workspace/Rheon-ui-authoring` over HEAD `425a9869fa3f36b7e3133ddbd213a493d7b5bab3`. The reviewer performed source/log reads and wrote only this report; no build, test, native launch, screenshot or workload was performed by the reviewer. No additional repository AGENTS.md or local SKILL.md was found in the inspected workspace locations. This is an independent scoped review, not a Standards Engine certification.

## Reviewed source identity

| File | SHA-256 |
| --- | --- |
| `src/bin/desktop/native_presentation_harness.rs` | `7ecb3487c24d57eb726e8212d1393bf855ca13125af53021bb0c6237dc056c6e` |
| `src/bin/desktop/viewport.rs` | `c79ef51daf9f0079f424f9d6d24343931d70d2f5d4fe233ee978931080aaa462` |
| `src/bin/rheon-desktop.rs` | `daf9b7e54d89decfbfbff8ae8dd7cb17cc4fbab3fd50f3323f442f02dd1c55b3` |

The production diff is confined to test configuration: the new module and alternate main are `cfg(test)`; the original main body is unchanged behind `cfg(not(test))`. The viewport field, default initializer, Appearance guard and inert-state query are also `cfg(test)`. No Cargo dependency, normal CLI/environment switch, runtime request, archive or physics implementation is added.

## Complete admitted helper boundary

The only admitted ordinary probe is `native_presentation_harness::synthetic_retained_history_timeline_and_disabled_modes_are_workerless`. Its complete construction, paint, input and drop path was inspected before execution admission.

`fixture` supplies literal Request metadata: size 2, one requested step, two distinct method identifiers, one thread, 1 m domain, 0.25 s timestamp spacing, edge-2 preview and 16 MiB planning budget. Material, emitter, mesh, configured/saved intent, receipts, gates and prior IDs are absent. The recording path is an unused sentinel string. `validate_request` checks limits, geometry and buffer/history arithmetic; it does not construct a Simulation. `BufferPlan::for_grid` computes checked storage sizes. The preview constructor reserves bounded in-memory frame slots and creates default readerless Inspectors.

Publication directly constructs four PreviewFrames (two tracks, two ordered frames each), with eight supplied density bytes per frame, steps 0/1, times 0/0.25, zero unmeasured calculation duration and no reports/material. It calls in-memory `LiveHistory::publish`, not capture, timing or numerical sampling helpers. The explicit paused track/index-zero and `follow_live=false` initialization avoids following the last-published track.

`App::default` and its default controls construct values, empty owners and strings, with an available-CPU query; worker and scheduler are absent. The harness never composes `App::update`, sidebar, run controls, polling, dispatch, archive/saved views or model loading. `selected_frame` only reads the in-memory history here. Its `ensure_timestep` reaches a default Inspector with no reader: `select_timestep` fails before the reader-selection closure and any archive I/O. That absent-recording feedback is not evidence of archive integrity or recording completeness.

The wrapper composes actual scene context and timeline helpers, then re-fetches the frame after timeline controls, matching production ordering. The Appearance renderer scans only the eight literal density cells. No collision mesh, emitter or worker voxel input exists. Draft force glyphs are presentation/validation only; `editable_scene=false` disables handles. No numerical lab owner is constructed. The entire viewport is disabled, and a test-only post-tab guard forces Appearance before any Particles/Obstacle/Transport controls or rendering branch, even if a tab interaction were unexpectedly reported. `presentation_is_inert` checks Appearance plus absent engineering/transport owners after every paint.

Pinned egui 0.33.3 response/UI source was readable from the workspace tool cache: disabled UI does not produce hovered/clicked interactions; the camera input helper exits before processing keys/wheel unless hovered or dragged. The earlier plan review's denied `/root/.cargo` read was not retried; this is a different accessible dependency location. The unconditional mode guard additionally removes reliance on disabled tab semantics for lab dispatch. Existing SceneUi/handle input paths are disabled or noneditable. No app recording/export/screenshot/persistence hook is composed. The enabled eframe dependency features omit persistence, and `persist_window=false` is explicit.

Normal ownership drops only in-memory data. `App::drop` cancels/joins only when a worker exists; this wrapper never creates one. Readerless Inspectors introduce no reader teardown or file cleanup. Native window/display initialization is outside this application-data I/O assertion.

## Presentation fidelity and lifecycle

The persistent banner says “Synthetic presentation data · no worker or physics”; a second visible note identifies supplied timestamps, unmeasured calculation duration and absence of archive/scientific results. These qualifications are essential because reused timeline captions include simulation/calculation wording. The banner must remain present in all native captures.

The harness mirrors dark style, 8×7 spacing, 280 px sidebar reservation and the narrow/open-Scene decision, without composing setup actions. It uses one canonical App/SceneUi/timeline owner. Draft hides playback, pauses it and preserves the retained index; Retained uses the existing timeline controls and timestamp selection. It is a composed presentation fixture, not normal production startup/sidebar or runtime parity evidence. The disabled canvas deliberately cannot validate ordinary camera, picking or handle gestures.

The fixed initial native window is 1250×820, with minimum 600×430; operators may resize it normally. There is no size environment selector. The Close control requests ordinary window closure. The single 300-second monotonic deadline requests Close from repaint; it is cooperative, not a hard process/GPU/teardown bound. Native validator process supervision remains required. Interaction does not extend that deadline.

## Build gate and evidence receipt

The ordinary compiler-generated cargo test main enumerates the one pure harness probe and no native test entry. The standalone recipe is `cargo rustc --locked --offline --features desktop --bin rheon-desktop -- --cfg test`, without `--test`; it compiles the separate main-thread eframe entry. Other test-only modules may compile, but no test runner or unrelated test is invoked by that main. No native binary execution is admitted in this environment.

Read-only log receipt: `/tmp/rheon-ui-presentation-list4.log` enumerates 190 tests and zero benchmarks. `/tmp/rheon-ui-presentation-exact2.log` names only the fully qualified admitted probe and records one pass, zero failures, 189 filtered out, 0.06 s. This is the final post-guard probe; the earlier exact1 pass preceded that guard and is historical evidence only. The final source hashes above match the inspected post-guard source.

The probe checks banner presence at 1250×820, 900×650 and 600×433.33334 logical sizes; attempts pointer press/release on all three disabled lab tabs; verifies inert owners, selected literal timestamps, Draft pause/index retention, unchanged deadline and relevant frozen Request metadata. It does not run a native entry. It does not establish full label clipping, real DPI/font/GPU behavior, screen-reader acceptance or native timeline interaction.

The final standalone build2 log records success in 7.80 s. Its 258 unused-code/import warnings arise from the standalone `cfg(test)` compilation and are not a strict test-configuration Clippy pass; no lint suppression was added. The ordinary production build2 restored the normal target in 15.73 s; strict production Clippy2 passed in 2.81 s and locked check2 in 1.28 s. Final fmt2/diff2 logs are empty and the parent reports successful exits. No executable was run by the reviewer or reported run in these gates.

The independently hashed final standalone artifact is `/workspace/scratch/rheon-ui-presentation/rheon-ui-synthetic-presentation-testcfg-guarded-425a986-increment.bin`: 235,185,520 bytes, SHA-256 `040715719f4e34e04951c8ad428024cbdbd219d7b65ac95d64d54e3e6e8d5cbd`. Its `guarded-standalone-binary-seal.json` has SHA-256 `fd84034c3e0ed1e8895aa0dbbafaaf16571ec1d375d7a2840d218bac5c679d6c`, identifies base `425a9869fa3f36b7e3133ddbd213a493d7b5bab3`, `build_cfg=test`, `rustc_test_harness=false`, `executed=false`, and agrees with the actual file's hash/size. The uniquely named copy survives the normal target restoration. The earlier successful standalone artifact preceded the final guard and is superseded for transfer. The existing native validator must execute only the final sealed standalone artifact, under its own authorization. Source hashes above bind the reviewed local increment, not an invented committed revision.

Final packet receipt: `evidence/synthetic-presentation-20261020/manifest.json`, SHA-256 `72df6b803271253cdaede05edfcf48cf9b6318a2fc61946a61abdca18b1e09c8`, was independently checked against all three current source files, all nine copied/raw log pairs (hash and size), and the guarded standalone artifact and seal JSON. Eight logs cover the current accepted list/exact/build/restore/lint/check/format/diff gates; the ninth preserves the initial compilation failure. That failure records unavailable `egui_winit`/`with_any_thread` and forbidden App struct-update moves under Drop. The final code removes the cross-thread native-test design and uses default-then-field assignment; the accepted compile/probe logs supersede that failed attempt without deleting it. The list contains exactly one harness test entry, once, and the exact result names only that probe. The manifest marks native execution false; no native execution is claimed by this receipt.

The read-only handoff review of `reports/native-synthetic-presentation-handoff.md` agrees with the inspected entry point and final artifact. It distinguishes standalone compilation from exact test execution, immediate unique sealing from ordinary-target restoration, and separately authorized native execution/process supervision. It retains synthetic annotations, cooperative closure and the viewport/physics/archive limitations. Native builds in another environment must seal their own executable and source identity; this local binary hash is not a reproducibility guarantee.

## Remaining acceptance limits

- The plan requested attempted canvas/key inputs; the current probe attempts disabled tab pointers only. Camera/key containment is source-reviewed, not empirically exercised here. Do not report canvas/wheel/key interaction coverage.
- Native Play/Pause, speed, track popup, slider, Draft↔Retained, resize/scale, banner visibility and Close behavior await the existing validator. Quality-85 JPEG delivery and native process closure are not established by this report.
- Normal production startup/sidebar integration, viewport gestures, force provenance, actual runtime/recording/archive parity, worker progress/cancellation and scientific correctness remain outside this harness.

No material source blocker remains for the scoped synthetic harness and the one admitted pure probe. These remaining gates limit evidence claims; they do not authorize broader tests or workloads.
