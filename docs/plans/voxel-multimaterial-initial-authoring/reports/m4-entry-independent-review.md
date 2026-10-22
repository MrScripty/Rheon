# Independent M4 immutable capture/rerun entry review

Review date: 2026-10-22. Reviewer: separate agent `/root/voxel_implementation_review`, distinct from the implementer and source-capacity helper author. Reviewed the complete dirty diff, new setup owner/tests, admission, prior staged-adoption contracts and existing authoring/frozen-input/worker owners. Only this report was written. No test, build, native session, simulation, benchmark or external action was performed by the reviewer.

**Initial disposition: one bounded required correction below.** Fresh capture must apply the authoring density envelope separately from saved replay. No additional required source correction was found in this complete round. This does not reopen the physical model or accepted components. Normal UI exposure, actual native A6 and whole-feature acceptance remain pending.

## Complete required findings round

### R1 — Medium: fresh contents capture inherits the saved-spacing exception

Location: `voxel_material_setup.rs::capture_contents_request`, especially lines 74–101; existing `voxel_material_execution::validate` saved-spacing behavior and `voxel_material_authoring::native_authoring_geometry`.

Fresh capture freezes the generic InitialContents document directly, without applying the native authoring geometry/density gate. FrozenInput::capture deliberately supports more general inspected/saved geometry. Worker validation deliberately permits immutable saved inputs above the **new-authoring** 1000 cells/m cap, provided their 4–64 cubic native host extent is supported. Combining those two valid generic/replay boundaries is insufficient for a newly captured document.

A concrete input is a filled 4³ document with h=0.0003 m and zero origin: side=0.0012 m is inside the host extent, but its resolution is about 3333 cells/m. Scene's ordinary new-contents creation owner accepts that bounded grid and does not itself apply the authoring density cap. Current capture then creates a valid FrozenInput and start_contents can launch it through the saved-spacing worker exception. This bypasses the previously explicit distinction between new authoring and immutable replay. Normal launch is currently disabled, but the internal production entry being qualified must preserve that contract before exposure.

Required correction: apply the existing native authoring geometry check to the actual document before fresh capture/launch, preserving its original bits and refusing unsupported geometry with the existing actionable instruction. Reusing `voxel_material_authoring::native_authoring_geometry` through a narrowly recorded visibility/write-set addition is proportionate; do not reconstruct the grid from App.size/domain_side or add another copied policy. Saved rerun must retain its intentional spacing exception. Do not rewrite an accepted saved recipe or silently lower its resolution.

Add a focused boundary case proving the high-density current document refuses both fresh capture/start before output, with its document/history/old results intact. The same self-contained saved recipe must remain admissible through rerun under Adaptive preview and the existing host envelope; explicit finer preview policy remains independent. Retain the existing non-dyadic original-spacing fixture. No large numerical cohort or native session is required for this correction. The implementer acknowledged this finding and held serial edits for this complete round.

## Confirmed capture and rerun semantics

The entry owner builds one immutable VoxelRunSetRevision/FrozenInput from the actual document, not the scalar native preferences. Palette/IDs/geometry/revision remain exact, and binary material IDs convert through the existing frozen constructor boundary. It captures current dt, interval count, gravity and recording limits, with the admitted fixed qualified Water/Air method, Jacobi settings and 128 MiB numerical policy. The fixed policy can cause an honest aggregate refusal with large presentation/retained ownership; it is not a promise that every per-owner maximum fits simultaneously.

Capture refuses a live Worker/Pending value, wrong context/selection, force/contents gesture, configured Session workflow and current emitter/mesh/authored-force features. FrozenInput capture rejects an active editor transaction and Vacuum, including one started through the editor without UI gesture state. No fallback to carrier/particles, automatic feature deletion, temperature/phase field or initial-velocity invention is introduced. R1 is the missing native new-authoring envelope, not a failure to preserve the document or its physical coefficients.

The new Request uses the existing staged launch owner: one actual Jacobi track/thread, original frozen geometry for construction, immutable schedule/recording policy, fresh mutable attempt receipt, actual retained-capacity admission, typed constructor acknowledgment and existing cancellation/join/append owners. No new worker/runtime or numerical formula appears. Capture itself performs no output; unsupported launch refusal leaves the prior result installed.

Cold rerun consumes the supplied complete immutable revision, rather than current Draft, presets, gravity, native scalar counts or current recording defaults. Its copied prior attempt IDs feed the existing fresh-ID exclusion. Parent identity/finalized original remain unchanged; new attempt/path/child revision are fresh. The self-contained recipe supplies constructor zero: the runtime entry does not need old accepted fields, reopen/audit an old archive, or interpret a retained frame as a restart checkpoint. Tests may open old archives as their comparison oracle without making those reads runtime dependencies. Inspection state remains separate from execution.

## Related source ownership

The production launch calculator now additionally charges a still-retained RunSetView source revision when its complete immutable value matches the submitted source. It counts the **stored** owner's capacities, including spare capacity, plus retained root/inline storage; it does not substitute the smaller clone's capacity or serialize to estimate bytes. Complete equality can conservatively charge representationally equal values; it does not drop a required normal UI source, which is cloned from that same stored revision. Other independent page data stay outside the previously declared scope.

The related Archive Inspector query reads existing header custody under one lock. A current voxel header whose initial-input digest matches is charged even if no typed timestep is selected. A malformed/unclassifiable current voxel header is conservatively charged, rather than trusted as zero; an established different digest or unrelated profile can remain outside the required-source group. No numerical decode, solver construction or full archive audit is used merely to estimate live allocations. A poisoned owner lock refuses rather than reporting empty ownership.

The Inspector refactor moves the already reviewed retained calculation into one helper; its raw/typed/legacy/configured arrays and metadata arithmetic are unchanged. The related-header fixture explicitly covers matching, malformed and unrelated header-only readers, spare path capacity and different digest. The RunSetView fixture proves value equality survives inflated stored String/member capacities and charges those actual capacities. The App entry fixture checks its admitted retained sum includes both stored source and matching Inspector. Related controls remain inactive during Pending because existing worker/page gates and the staged hold remain in effect.

This closes the previously deferred related-source obligation for the current App entry owner without expanding comparisons or importing a source runtime. The existing 384 MiB/four-publication/reserve policy and its explicit exclusions remain unchanged. None of this is RSS, allocator-peak or performance evidence.

## Disabled normal UI and queued execution

Normal availability is a constant false in the new setup owner. The contents button and saved-voxel rerun button both use that guard; the old native/configured rerun paths stay enabled under their prior policy. There is no qualification build/runtime flag or normal user bypass in this slice. Internal App methods and fixtures can call the entry owner while the user-facing action stays disabled; their success is not a native UI receipt.

Queued Simulate intent is consumed once by the existing initial-contents command owner and discarded with local refusal while a worker/Pending exists. It reaches the same capture/staged-launch path when explicitly invoked by internal fixtures. The root saved-variant match similarly routes to the internal rerun owner; its ordinary UI cannot produce that action while availability is false. This review does not admit merely flipping that flag: actual native/readiness/input/whole-feature gates still precede exposure, including final enabled-control behavior.

## Executed-evidence limits

Read-only inspection of `m4-entry-tests.log` shows four new parent App fixtures passed and the deliberately ignored fresh child entrypoint was explicitly executed and passed separately. They exercise actual local Save/Load, a real brush one-cell binary mask, exact constructor F/zero MAC/no constructor pressure, original-spacing bits despite changed scalar preferences, actual existing Worker/SQLite App rerun after changed Draft/defaults, source/child immutability and full original f64 field/time/dt equality. The child opens the original recipe in a new process and reruns through App's same worker path. These are integration/replay observations, not independent numerical accuracy or native graphical acceptance.

The refusal fixture covers emitter/configured mode, Vacuum and active transaction before output. It does not cover the density counterexample in R1 yet. Strict desktop all-target Clippy log completed successfully before correction. Selected affected logs inspected at this boundary show voxel 45 plus explicit child runs, Inspector 9, source-payload 1, contents input 10, Scene 10, viewport 11, timeline 5 and setup 9 plus its pre-existing ignored harness passing. A further comparison check was still running at the log snapshot; its outcome is not predeclared. These were implementer executions; the reviewer ran none.

No actual native paint→save/load→launch→both-phase inspection→cold rerun, physical GPU, timing/RSS result or whole M4/A5/A6 acceptance follows. No production source outside the admitted entry/related-capacity set, persistence schema, numerical formula, main or unrelated task changed in this review. Parent owns serial correction, affected verification and admission, followed by targeted rereview.

## Reviewed identities

| Path | SHA-256 at complete findings round |
| --- | --- |
| `reports/m4-entry-admission.md` | `0ae49ff05a97f3bc02c389b2672e784e41d4f74e1a5538e5be1cbfccfb1ec22c` |
| `src/bin/desktop/voxel_material_setup.rs` | `6da1e805d3385fd11ee631ead5672a466c2540fdf9312dd47749489377268f47` |
| `src/bin/desktop/voxel_material_setup_tests.rs` | `fdd1358ea4ce5ce968f7ddd7af4bbbda557e5335fb00c19417c8fab6a8ddd312` |
| `src/bin/desktop/voxel_material_launch.rs` | `3e73ec2c1e263310a12d2b1d15b5aa527ccc6c534acee99f6c8ef0ff15d89d82` |
| `src/bin/rheon-desktop.rs` | `f69b320fa2bb8d830d020f1a8345a7a42a94a1c36cd29c62d91cfdd22d8e539d` |
| `src/bin/desktop/initial_contents_ui.rs` | `78cc5f51d083f8c218562983333d901f1472553647497dcb935493b1bea302be` |
| `src/bin/desktop/run_set_view.rs` | `c4ec6367bd2f3bc160a955648fbc02d45579512a90e2a5a3fdc5d8645d5cd900` |
| `src/bin/desktop/inspector_payload.rs` | `84f5cb765823fa0074523a403000a5891507ed8ffc826fe969e24b60a175efa3` |

Only this report was written. No reviewer action was denied, pending or uncertain. Historical denied operations and uncertain Library uploads were untouched. This review grants no normal exposure, external publication or main integration permission.

## Targeted R1 correction rereview

On 2026-10-22 the same separate reviewer inspected the serial correction, distinguishing boundary fixture and corrected execution logs. Only this report was appended. No tests, builds, native sessions, simulations, benchmarks or external actions were performed by the reviewer.

**Current independent source-review disposition: R1 is resolved. No further required correction was found in this targeted scope.** Parent/implementer owns bounded component admission after affected checks. Normal availability remains false; original actual native/whole-feature M4/A5/A6 and integration gates remain pending.

Fresh capture now calls the existing `voxel_material_authoring::native_authoring_geometry` on the actual document before FrozenInput capture. That existing owner is exposed only within the desktop boundary through the explicitly appended narrow write-set addition; its retained-geometry error wording applies to both load and current capture. No duplicate density policy, scalar reconstruction or resampling was introduced. Rerun does not call this new-authoring gate, retaining its intentional immutable saved-spacing exception and independent preview policy.

The added fixture first retains an actual completed old result, then installs a filled current 4³, h=0.0003 document. Fresh capture and start both require the 1000 cells/m refusal before output/worker, and the fixture checks exact draft digest, selection, retained history Arc, old Request input digest and old row retention. It then creates a self-contained generic saved recipe for that same grid and replays it with Adaptive preview through App's actual staged Worker, requiring recorded spacing bits and counts to stay exact. The test does not rewrite any original on-disk intent or weaken the worker's saved-input policy. Existing non-dyadic-spacing and original-state rerun fixtures remain intact.

Read-only inspection of `m4-entry-corrected-tests.log` shows 5 parent fixtures passed, 0 failed, with the deliberately ignored child entrypoint explicitly executed and passing separately. Final strict desktop all-target Clippy log completes successfully. The earlier configured comparison check also completed with 5 passing fixtures. Other affected corrected checks were still being collected at this review boundary; their results are not predeclared here. These are implementer executions, not reviewer workloads or native/scientific/performance acceptance.

| Path | Final SHA-256 |
| --- | --- |
| `reports/m4-entry-admission.md` | `490ffcb8a8a28c30b25c0c5371e764ef1a7427be53c6638c199ae864ea96f2d6` |
| `src/bin/desktop/voxel_material_setup.rs` | `b6072698edd5e11f8f6b76e0e749aad2981a763ed5a1ec871bc2536406c20660` |
| `src/bin/desktop/voxel_material_setup_tests.rs` | `79b4d5d60debe0dff3601a30591deb507580eccd0e6947c1df228490702e8286` |
| `src/bin/desktop/voxel_material_authoring.rs` | `93a37d870d2992751f8a245d7b7da003b3994409bf7509c15b82470de7c3442a` |
| `src/bin/desktop/voxel_material_launch.rs` | `3e73ec2c1e263310a12d2b1d15b5aa527ccc6c534acee99f6c8ef0ff15d89d82` |
| `src/bin/rheon-desktop.rs` | `f69b320fa2bb8d830d020f1a8345a7a42a94a1c36cd29c62d91cfdd22d8e539d` |
| `src/bin/desktop/initial_contents_ui.rs` | `78cc5f51d083f8c218562983333d901f1472553647497dcb935493b1bea302be` |
| `src/bin/desktop/run_set_view.rs` | `c4ec6367bd2f3bc160a955648fbc02d45579512a90e2a5a3fdc5d8645d5cd900` |
| `src/bin/desktop/inspector_payload.rs` | `84f5cb765823fa0074523a403000a5891507ed8ffc826fe969e24b60a175efa3` |

Only this report changed through reviewer action. No action was denied, pending or uncertain. Prior external denials and uncertain Library uploads remained untouched; no normal exposure, publication or main integration permission follows.
