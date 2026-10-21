# Independent T5 runtime-force integration source review

## Current disposition

Suitable for a distinct gated source/pure-test checkpoint over `8148fa239a60502a1245d5a69d0eeb8ceb905c16` in `/workspace/Rheon-ui-authoring`. Production force execution remains refused. The additive custody/compatibility wiring is proportionate to the admitted desktop ownership. The two initial findings below are resolved in the final re-read source; no material source blocker remains. Source suitability and the focused passes do not qualify runtime/archive or native behavior.

The reviewer read source and wrote only this report. No build, test, numerical/model/evaluator workload, SQLite/archive operation, native application, cleanup, commit or publication was performed. This review does not refresh backend admission or confer numerical/runtime/archive qualification on the new desktop closure.

## Findings and required resolution

1. **Retained composition ordering — repaired:** the initial implementation painted the tree before resolving follow-live and rebound only the viewport after timeline input. `prepare_scene_presentation` now resolves the frame/active track once before tree composition and binds its retained document. The cached frame remains the viewport input for that render frame. Timeline scalar intents commit for the next render frame, preserving one frame/document projection across tree, properties and viewport. On Draft→Retained, the bound-track helper deliberately avoids a second active-track resolution; the next preparation follows live normally. The synthetic wrapper mirrors this ordering. Pinned egui 0.33.3 input handling requests immediate repaint for pointer/key events (with an outstanding follow-up repaint), so normal interactive deferred intents have a subsequent paint; native integration is still unverified.
2. **Independent v2 evidence — repaired and focused probe passed:** the v2 probe now pins literal metadata hex and a literal v2 SHA from independently constructed Python little-endian/tag bytes, rather than only Rust roundtrip equality. It explicitly checks None-versus-Some(empty) ForceMetadata declarations and order reversal, alongside name/disabled/units/signed-zero changes. The source format agrees with the canonical codec's field/tag order and the distinct v2 domain/length framing. Its individually exact result passed; no numerical execution is requested or inferred.

No other material source defect was found after re-reading the complete current integration diff, including production launch/output gates, recorder error/custody branches, v1 serialization and the narrow test fixture changes. The sealed source set and focused gate receipts below bind this disposition; future edits require a new review scope.

## Request and execution custody

Request owns a bounded `Vec<Option<ForceSnapshot>>`: empty represents all-legacy, otherwise validation requires exactly one option per method. `fields_for_track` is the sole resolver used by carrier stepping, carrier constructor/accepted recording, material stepping/recording and retained projection. Explicit None is not substituted with another track or the mutable draft. Every present immutable snapshot passes canonical metadata encoding; saved members are independently frozen and compared as exact encoded bytes, including absence, before worker admission.

All production literals use the private test-only admission flag false. Non-test compilation fixes admission false. A Some snapshot, including authored-empty metadata, is refused before Worker construction; configured/session/provider requests refuse before that flag can bypass the temporary gate. The flag does not bypass count, scene, saved binding, geometry, budget or supported-owner checks. No CLI/environment route exposes it. Fresh start also retains the existing draft-readiness refusal. `freeze_native_set` validates before output, and loaded reruns freeze stored per-member scenes and preflight before their first output path. Legacy reruns remain independent of the current draft. Test fixture changes only initialize the new fields as legacy/false; they do not authorize running those older fixtures.

The native carrier Some branch calls the canonical `step_with_force_fields` or emitter variant with the existing requested dt, source/emitter and cancellation callback. None retains the original branch. Configured execution has a defensive explicit unsupported branch. Particle material uses the canonical optional field adapter with the original emitter/cancel policy and records the same immutable track-zero snapshot. The patch introduces no alternate evaluator, density conversion, gravity, dt, retry or scheduler policy. This is source inspection of dispatch, not evidence that a step ran or a field produced correct physics.

## Recorder custody and compatibility

Legacy Inspector begin/record methods delegate through None. Optional constructor paths bind the run once. The already-bound native begin method checks the expected force metadata digest before `StepRecorder::create`; None rejects a bound header and Some requires the same digest. Reconstructed actual headers apply the same binding before the existing full equality check. Accepted publication helpers require the backend's actual matching force stamp; they cannot synthesize missing accepted custody from the supplied draft. Existing recorder ownership, identity, limits, finish/error/reader handling remain in place. No independent core recording source is changed.

FrozenSpec adds an omitted-when-None ForceScene field while NativeCarrierSpec is unchanged. Version 1 requires absence and retains the historical JSON spec hash/field order. Version 2 requires a valid present scene and hashes the fixed `rheon-desktop-native-force-spec-v2\0` domain followed by little-endian u64 byte lengths and the legacy JSON spec/canonical binary force bytes. Unknown or invalid version/payload pairs refuse. Explicit JSON null is rejected by the present-field deserializer; absence is the only legacy None spelling. Original input is bounded before strict typed decoding, preserving unknown/duplicate-key rejection.

Manifest version remains 1, permitting individually valid v1/v2 members without migration. Native declaration version follows its FrozenSpec. Bound run creation validates the frozen spec and constructor; cold binding checks declaration version/hash and expected run force digest. Existing reader verification and decoded selected publications enforce run/publication metadata agreement. `ForceMetadata` differences compare canonical bytes, retaining signed-zero, order and None/empty distinctions rather than relying on float PartialEq. Existing comparison/source presentation receives only its new exhaustive label. Nested FrozenSpec version failures are separate from the envelope's configured-owner fallback sentinel.

## Retained authoring boundary

SceneUi stores a separate read-only retained document and `(track, optional digest)` identity. Begin/clear resets that binding; changes clear retained selection even if IDs coincide. Legacy absence stays unknown with explicit UI copy; Some(empty) is a known empty authored document. Properties are painted from a disabled copy, and selected-force/edit/handle paths remain Draft-only. Main passes editable only while Draft with no worker, and handle painting independently checks Draft. Scene projection now uses the context-specific document. Stored scenes do not mutate the editable draft. The repaired composition uses a cached per-render frame/document; timeline selection changes appear together on the next frame. Nonempty retained authored documents also retain the lab-mode guard, so unrelated lab owners cannot silently substitute an unforced workflow.

## Pure-probe admission

The complete new `force_runtime_ui_tests.rs` file and helper paths were inspected before admission, then re-read with the added goldens and composition test. Seven individually enumerated exact names are source-safe:

- `force_runtime_ui_tests::force_intent_legacy_v1_bytes_and_digest_remain_exact`
- `force_runtime_ui_tests::force_intent_v2_freeze_roundtrip_and_mixed_member_differences_are_exact`
- `force_runtime_ui_tests::force_intent_rejects_mixed_versions_tamper_unknown_duplicate_and_oversize`
- `force_runtime_ui_tests::force_request_track_resolver_gates_before_work_and_binds_saved_members`
- `force_runtime_ui_tests::force_recording_header_preflight_rejects_missing_wrong_or_double_binding_without_io`
- `force_runtime_ui_tests::force_retained_track_projection_is_readonly_and_clears_same_id_selection`
- `force_runtime_ui_tests::force_retained_composition_resolves_once_before_tree_and_defers_track_intents`

Those helpers use literal serde metadata, NativeCarrierSpec/FrozenSpec validation, canonical codec/hash, scalar Request preflight, literal RecordingRun binding and SceneUi/default Context paint. Geometry/BufferPlan construction performs arithmetic only; ForceScene freeze validates and clones without sampling acceleration. The seventh helper adds bounded in-memory LiveHistory reservation and direct publication of two eight-byte literal PreviewFrames, plus App default/prepare calls. It never starts or polls the App. `prepare_scene_presentation` reaches the default readerless Inspector; selection fails before the reader closure and I/O. Worker stays None, so App drop has nothing to cancel/join. The tests never call `create_simulation`, capture, Worker/App update, Inspector begin/record/reload/readers, recording writers, filesystem fixtures or cleanup. Inspector binding helpers expose only the pure association operations to sibling tests. Generic JSON values are used to manufacture malformed test inputs; production decoding still receives original bytes. The seventh probe manually applies scalar track intent and checks projections; it is not native pointer/dropdown or complete App-update evidence.

No numerical, SQLite or native probe is admitted here. Prepared ignored integration tests, if later added, require a separate complete helper and authorization review. Existing backend and earlier UI/native evidence do not qualify this changed execution/recording closure. Final source/pure-test/build receipts must preserve that boundary, and the visible launch gate must remain until separately authorized bounded runtime/archive validation.

## Focused evidence and regression re-admission

Read-only receipt of `/tmp/rheon-ui-t5-list4.log`: 197 tests, zero benchmarks. `/tmp/rheon-ui-t5-exact1.log` through `exact7.log` each name one corresponding admitted test above and record one pass, zero failures, 196 filtered out. These are metadata/Context assertions, including the independent literal v2 golden; they provide no numerical or SQLite result. A final transfer manifest has not yet been received.

The following six existing regression names were independently re-inspected with their complete current helper paths and admitted only for individual exact execution after enumeration:

- `native_presentation_harness::synthetic_retained_history_timeline_and_disabled_modes_are_workerless`
- `scene_ui::tests::scene_pointer_add_select_edit_collapse_and_retained_context`
- `scene_ui::tests::scene_draft_timeline_roundtrip_preserves_retained_index_without_history`
- `scene_view::tests::scene_pick_identity_overlap_retained_and_unavailable_preview`
- `scene_handles::tests::handles_cancellation_context_focus_pointer_and_camera_loss_never_hands_off`
- `timeline_ui_tests::timeline_track_selection_and_report_rows_fit_cold_and_changed_states`

The synthetic presentation helper retains the previously inspected workerless/readerless construction and disabled Appearance guard; it now mirrors cached preparation rather than refetching the active track mid-frame. The SceneUi pointer helper composes only SceneUi and literal input; its retained roundtrip helper uses App default and a direct PreviewFrame with no history or worker. The scene-pick helper builds projection from literal vertices/triangles/force objects and a camera, never CollisionMesh/model constructors. The handles helper directly invokes handle update/paint with literal SceneUi/camera/Context, never Viewport or App update; its cancellation assertion does not establish full camera fallthrough behavior. The timeline helper paints direct PreviewFrame/report structs and a literal Request, with no history; supplied report values are not calculated results. None of these helper paths starts a Worker, constructs/steps a solver, opens SQLite, attaches an Inspector reader, creates a recorder, or performs filesystem cleanup. App-containing helpers keep worker None through drop.

Read-only receipt of `/tmp/rheon-ui-t5-regression1.log` through `regression6.log`: all six requested names passed individually, one pass and 196 filtered out per log. Together with the seven new names this is thirteen distinct exact pure/Context passes, not a broad suite. Locked desktop check3 passed in 1.43 s and strict production Clippy1 in 4.02 s. The parent confirms no production edits after these checks; hash reads agree with the final inspected source. Format/diff and a sealed transfer packet, if later supplied, remain separate receipts. No native entry or numerical/SQLite integration probe ran as part of these thirteen logs.

## Final reviewed source seals

All paths below are relative to the repository. The unrelated source/numerical owners are outside this write set; plan/docs are not included as production source seals.

| File | SHA-256 |
| --- | --- |
| `src/bin/rheon-desktop.rs` | `0cc6bd651a60b2d09abc77bd49515a114d993faa842acebf19e3955ad9b7a62e` |
| `src/bin/desktop/carrier_execution.rs` | `c4455879373fc71eaaf398fdaa230c4d0b115dc7ff0a9da6356afeb24cba23b2` |
| `src/bin/desktop/inspector.rs` | `aad6760e94582d6512c32387609c3c2874e92728c93c3c7d2089cbf3bc6ee62c` |
| `src/bin/desktop/run_sets.rs` | `b25f07c5d9f4a27572922956043224c35dde35d9a0d987ab8af4f3f3151c91f0` |
| `src/bin/desktop/run_sets/source.rs` | `39e8982fffaf4e9adcd0ca0c60f49eabc96b829957bc6d9c05479c80eee99daf` |
| `src/bin/desktop/comparison.rs` | `27ad4b88db848430a68f5f9fae796fa10eab1df26af9dbae73386e55c9b5c74f` |
| `src/bin/desktop/scene_ui.rs` | `cc866e716876dfee8c9245c2473426c7c3253b2b0787b49b907202a6e1402557` |
| `src/bin/desktop/scene_view.rs` | `d16851804c3aa727c7fdf658a821ef8c8cd2c167e853b72d02604f2a14dfa02f` |
| `src/bin/desktop/scene_handles.rs` | `4818b85644a2891de562adc00fc203eb59c674bbf46aa88cd440bcab6477eb22` |
| `src/bin/desktop/force_runtime_ui_tests.rs` | `0f8b88db0553cf079c0f78a2e41766fcb6160f6dda1a3d117f1ce93fe17b91f5` |
| `src/bin/desktop/native_presentation_harness.rs` | `38b8b68ebc8a54769bcf8e65b2cf80890c03eac2251b4f838fac29ecbf63ecb6` |
| `src/bin/desktop/configured_worker_tests.rs` | `dabe1c3cf4214ee30ca42102c4c5387564b661f81b810d2c61035b8691669016` |
| `src/bin/desktop/setup_ui_tests.rs` | `db3d1328aa159e79ea4f8a15cd59ebb6cb7c9352854760bd8164587014e35900` |
| `src/bin/desktop/timeline_ui_tests.rs` | `915c565a2be34618a4be8ad801e215a2abe15a7eaf801fb2e5353bf632bdad54` |

The last three fixture files only initialize legacy empty/false Request fields, as do unrelated existing Request literals in main. They were not executed merely because they compile. Changed synthetic harness source does not retroactively change the older sealed native binary or its evidence; a later native build needs a fresh source/binary seal.
