# Independent prospective M4 worker design review

Review date: 2026-10-22. Reviewer: separate agent `/root/voxel_implementation_review`, distinct from the implementer. Reviewed the proposed worker admission against the canonical plan and actual Request, Worker, dispatcher, history, renderer, recording, frozen-input, saved-profile and compiled-identity owners. Only this report was written. No build, test, native session, simulation, benchmark or external mutation was performed.

**Disposition: the proposed integration is a coherent minimum, subject to the two admission clarifications below before implementation.** These are prospective contract findings, not defects in a completed worker implementation. The existing numerical/persistence components do not establish this integration's acceptance. M4, A5/A6, App adoption, release exposure and main integration remain pending. No new numerical scheme, worker framework, renderer platform or scientific cohort is required by this review.

## Complete required findings round

### R1 — Medium: define the reserve's actual scope and the enforceable allocation obligations

The proposed checked 384 MiB sum is a useful admission policy. It is not yet an enforced bound on all managed host memory. Two existing source facts matter:

- `history.rs::plan_resolution` budgets nominal cell bytes, frame/Arc overhead and frame slots. `with_resolution` reserves vectors, but does not reconcile actual capacities with that nominal plan. `publish` checks edge/sequence/count, without checking the density vector's length or capacity. These existing methods alone cannot support the new actual-capacity assertion.
- The renderer's documented approximately 78 MiB limit covers two face buffers and two projected meshes, including the previous cache. Its source explicitly excludes **egui**, GPU, allocator and other views. The proposed admission lists GPU/allocator/process/SQLite/unrelated-view exclusions, but omits egui's CPU shape, tessellation and paint staging. The 80 MiB reserve must not be presented as encompassing those allocations. `try_reserve_exact` also does not promise exact capacity.

Required admission correction: call 384 MiB a checked **reserved admission budget for enumerated payload groups**, name egui CPU allocations as outside the inherited surface-array estimate, and specify which newly owned capacities will be checked before installation/publication. Alternatively, actually include and enforce additional groups; that broader work is unnecessary for this slice. Existing renderer caps may be reused honestly as a source-based reserve rather than advertised as a new hard measured total. Allocation refusal after checking an actual returned capacity remains possible; this is not an allocator/RSS guarantee.

For the new history path, charge actual vector capacities and metadata/slot storage, validate `density.len() == edge³`, and reject overflow, capacity excess or shape mismatch before publishing a frame. Preserve constructor-plus-all-requested-interval planning. A frame rejected after its durable record was appended must produce an explicit recorded-but-not-presented condition, rather than erase the record or silently retry append. It need not invalidate the accepted numerical state or existing acknowledged preview prefix.

The remaining reserve groups need a short lifetime/accounting statement in admission or the owner implementation:

- The 48 MiB editor/staging reserve is plausible, not already proved. Existing authoring bounds give 32 MiB command history, 8 MiB candidate/preparation and at most 0.5 MiB dense u16 document cells. Enumerate the remaining simultaneous frozen ID/F copies, immutable Request/recipe clones, JSON/base64 buffers and manifest staging; release temporary copies at their owning boundary. Refuse live gestures at capture. A repeated unbounded clone chain cannot be excused by the reserve.
- Use the **accepted-record** publication estimate as the maximum, including pressure, rather than only the smaller constructor estimate. Three simultaneous groups can cover producer publication, selected raw record and selected decoded f64 fields if those lifetimes are enforced. Clear old selection before installing a new one, avoid additional retained decoded generations or producer copies, and check each group's actual capacity using checked arithmetic. Arc references alone do not duplicate array payload. Do not make the previously removed unused Inspector adapter reappear merely to obtain access.
- Production retained-host accounting is still missing by design. Include retained old history/frame capacities, raw/decoded Inspector selections, result arrays and immutable retained requests/inputs, plus other overlapping owned arrays within the declared scope. Avoid forgetting old arrays because a new Worker owns a different Arc. Conservatively counting shared payload twice is safe; silently omitting an owner is not. A zero old-context fixture is valid only when those owners do not exist. Its success cannot qualify a real App caller-supplied zero.

The cap need not admit the Cartesian product of every individual maximum: 192 MiB numerics + 96 MiB history + 128 MiB reserves already exceeds 384 MiB before publication groups or old results. An explicit refusal is coherent. Persist a tightly chosen new numerical limit only at original capture; never lower a saved recipe's declared policy on rerun. A large retained old result may legitimately prevent a new launch. Do not silently drop it to satisfy the new budget. The proposed later production context calculator and atomic adoption gate are therefore necessary, not optional evidence to infer from the headless worker.

### R2 — Medium: make the serial executor declaration an executable request constraint

The admission promises serial execution and cold equality under a captured serial declaration, but existing Request validation permits any available thread count and `run_owned` builds that Rayon pool. The reused pressure operator's `apply_full` enters the shared parallel helper; its pointwise chunks can execute concurrently on a multi-thread pool. Serial reductions do not make that whole execution serial.

Required admission correction: require `request.threads == 1` for this new baseline, with actual Jacobi pressure and the matching voxel executor identity. Keep legacy thread policy unchanged. An alternative declared parallel policy would require honest identity/qualification rather than the current serial promise; it is unnecessary here. The immutable recipe, requested duration/count, recording policy, actual coefficient/configuration bits and compiled executor declaration must all match before any output directory, intent or archive is created.

`build_identity.rs` explicitly describes compile-time source/build values as declarations, not authentication or complete reproducible-build proof. An ordinary build can return `build revision unspecified` and no tracked-material hash. Preserve that fact in capture/display/refusal policy; do not fabricate a revision or treat matching unspecified strings as verified binary provenance. A fresh-process same-build fixture establishes the exercised numerical replay scope, not equality for other compilers, hosts or arbitrary binaries bearing the same declaration.

No other required design correction was found in this complete round. The following are concrete implementation and deciding-evidence obligations already consistent with the proposed direction, rather than requests for a wider architecture.

## Owner and dispatch assessment

One owned immutable VoxelRequest containing the validated single-member recipe/revision and resource context is sufficient. Request's existing Clone must clone immutable values/Arcs, never a VoxelMaterialSimulation. Construct exactly one numerical owner inside the existing worker thread. Reuse `Worker::start`, cancellation, scheduler, join and progress; branch at the start of `run_simulations`, before particle or legacy carrier construction. The numerical owner already implements bounded success-only steps; the desktop must not reproduce its transport, projection, retries or state transaction.

Validation needs a dedicated voxel branch. The current common path treats `material == None` as carrier, performs a legacy BufferPlan on reconstructed scalar geometry and classifies force policy from existing workflow fields. It must not accidentally admit force snapshots, a second backend or a wrong observer/provider path. Reject the listed particle, configured/native saved, emitter, mesh, authored-force, observer/provider and multi-track combinations explicitly, before output. Preserve the old validators and their fixture behavior for old modes.

The proposed 4–64 cubic equal-spacing zero-origin native envelope is a reasonable host/picking/presentation limit. The components remain more general. Construct from original FrozenInput geometry and coefficient bits; never round-trip spacing through `side = h*n` and `h = side/n` to make the solver grid. The legacy scalar side can drive presentation only after checked finite host compatibility. Physically zero signed origins must not silently be rewritten to positive-zero in frozen data. Decide and test the new host's existing `[0.001,100] m` side and 1000 cells/m policy explicitly: current new-request validation applies the sampling cap unless a legacy saved/configured field is set. Do not bypass it by masquerading as one of those profiles. Either preserve frozen-spacing inspection/native refusal consistently or explicitly admit the host-compatible saved case without changing its intent. Unsupported general grids remain inspectable with an actionable run refusal.

Keep the true voxel method/configuration label. A PressureImplementation field retained for existing track structure is not authority to call the backend a carrier solver. No legacy StepReport should be manufactured from a voxel report. The display-only tag must distinguish constructor/accepted frames and the actual method; typed records retain full f64 F, occupied gas `1−F`, MAC velocities, pressure and original geometry.

## Accepted intervals, persistence and cold rerun

An interval count is not a guarantee of `count * requested_dt` elapsed time. The actual numerical owner can ratio-shorten the requested interval and records its original start plus actual accepted dt. Reuse that report verbatim; do not infer a halving sequence, reject a valid shortened step using the old carrier fixed-dt guard, recompute its original clock by subtraction, or infer retry cause/count from an archive. Repeat the captured maximum requested dt for each requested interval according to the existing numerical API. Completion, progress, result time and cold audit must describe the actual accepted count/time.

The proposed order is correct: freeze immutable intent, construct numerical state, append constructor, then publish constructor appearance; for each success, capture the accepted publication, append through StepRecorder, then publish immutable appearance. A failed numerical candidate does not publish. A failed/uncertain append stops execution and preserves the acknowledged recorded prefix plus the explicitly unrecorded accepted tail. A poisoned writer is not made healthy by a desktop retry. Terminal publication/close/pin failures must retain their real action, path and uncertainty; neither generic `Cancelled` nor a Ready child revision may conceal them.

Use existing protected local path/identity/no-replace publication owners. Beware that `new_recording_path` creates its directory: aggregate/configuration/executor refusal must precede that helper or use the non-writing candidate-path stage first. Fresh attempts exclude source run IDs and use distinct archive/revision paths. Original accepted input/parent identity stays exact; a final child records the new attempt only after confirmed terminal close and pin. Cold rerun opens original immutable bytes, never current Draft/presets or retained preview bytes. Inspection alone must not construct a numerical owner.

Same-process identity counters are not globally authenticated IDs; explicit source-ID exclusion and protected fresh paths are the relevant local contract. Preserve failed preparation/attempt artifacts honestly. A failed final manifest write must not be retried blindly after an uncertain outcome or reported as no simulation having occurred.

## Actual presentation and fixture consumers

The proposed production write set is appropriately narrow. The listed Request/PreviewFrame fixture literals cover the current desktop owners; the separate AI diagnostic Request is unrelated. Enumerate any further compiler-discovered literal before editing, rather than changing broad files. None-field compile repairs are not new voxel behavior evidence.

The new explicit preview/backend tag must reach the current consumers that infer semantics from `material_color` or the absence of particle reports:

- Track labels, running setup, timeline captions and result-row/report fallbacks must show the real voxel method rather than “Filled carrier-fluid,” “Material fluid” or a Native baseline.
- Retained Inspector selection and side-panel routing currently divide particle versus carrier ownership using `material_color`/`Request.material`. Choose the real per-track typed Inspector deliberately; constructor and selected frame identity must agree with the archive.
- Current metadata displays “Nonzero tracer” and “Pressure includes clear carrier fluid” for every non-particle frame. The voxel path needs liquid-appearance wording and an explicit occupied-gas transparency explanation. Max-resampling/quantization is an appearance reduction, not conserved volume or authoritative occupancy.
- Existing alternate material/laboratory viewport modes must not accidentally expose another model's data under this request. Keep inappropriate modes unavailable or route the explicit new mode correctly while preserving camera/Scene/force custody.

The typed retained slice remains the direct both-phase/f64 inspection boundary. No mesh, byte preview, transparency threshold, current palette lookup or constructor-only material ID may substitute for evolved F. Add focused semantic presentation assertions alongside literal fixture fixes; otherwise wrong legacy labels can compile and pass worker-only checks.

## Deciding fixtures and deferred App gate

The proposed real existing-Worker/SQLite and fresh-process fixtures are the right next evidence. They should cover constructor plus accepted fields/intervals, positive occupied gas mass, exact captured coefficients/configuration, actual ratio-shortened flow, immutable input after later Draft edits, source-ID exclusion and original-intent cold rerun. Drive cancellation/failure at an observable real boundary, assert the retained prefix/tail status and terminal disposition, and exercise pre-output aggregate/feature/executor refusal. Capacity negatives must target the actual new admission/publication owners; nominal arithmetic-only tests do not establish actual-capacity enforcement. No additional benchmark or broad accuracy cohort is needed here.

Current `App::launch_request` replaces history, running request, rows and textures immediately after Worker::start, before a constructor can be acknowledged. Reusing that adoption behavior would violate the plan. The proposed deferral is honest: this component may qualify the real worker behind an unavailable normal App action, but cannot claim old-result retention on failed launch. Later App work needs a staged pending worker/request and a defined constructor acknowledgment, with atomic adoption of matching history/request/Scene bindings. Durable constructor append followed by successful constructor-frame publication is a useful observable acknowledgment; the exact owner must be explicit. Before that event, cancellation/start failure retains old results and bindings. Afterward, normal accepted-prefix behavior applies. There is no requirement for a second worker manager or a mutable solver in App.

Production old-context calculation, identity collision/load/reset handling, save/load, actual native execution and A5/A6 remain mandatory later gates. A qualification session must be labelled and use the actual application/solver. Source inspection, component tests or the earlier M3 idle software-rendered session cannot stand in for them. No exposure, whole-feature acceptance, performance/RSS claim, physical-GPU claim or integration permission follows from this design review.

## Reviewed identities and limits

The report applies to the following source snapshot, before worker source edits. Hashes identify reviewed bytes, not executed qualification.

| Path | SHA-256 |
| --- | --- |
| `reports/m4-worker-integration-admission.md` | `b28a303bd9ec2638903aee2a2a79013f4cdfdfb2255adc5eee580dd4cbff51d6` |
| `plan.md` | `c74b5255cd07c8545f5dcdb96b41af804bd78a6a27d66ef3b807875fd1b2317b` |
| `src/bin/rheon-desktop.rs` | `bf5d4b74e689404fd3e7a5486f91b77fd8fff87acbeda8a053e73d769414de5e` |
| `src/bin/desktop/history.rs` | `e4be154bbbf40a36f370b68ec9e15fea663fb589d476a43eb298c0e8c0f0d821` |
| `src/bin/desktop/viewport.rs` | `34e7c9e072a7e11572945a82a57186563da411bb4c39a2713202c59c5527c874` |
| `src/bin/desktop/build_identity.rs` | `a94fea1f039a49f8d0eb79b019b3fbd0f2c9da1762ba854d10cba1e5fd87a25b` |
| `src/bin/desktop/initial_contents_ui.rs` | `c95fae88aff4ecfc00e46fd2896d57f64e0db81b2d94f5dd6eb22b9ac7c40d18` |
| `src/initial_contents.rs` | `e9bf1cd1d2bfe5396c6a5fb161d1f1dfea883a00ccfd8db8c6b0547ace28948a` |
| `src/bin/desktop/run_sets/voxel_material.rs` | `cd43376b66c4f4cce454e31506bb0af8d5e9c003e82e438f93433c575e5a42d1` |
| `src/bin/desktop/run_sets/voxel_material/run_set.rs` | `3018decddb681f23f53c4fbc35fbfb1a8e6a28c66d894e2db35947a4e904a8f2` |
| `src/recording/voxel_material.rs` | `b0114efbcc8abc4d5ed3877c8210a853a2b101e0557fc79f0c28dc11fa600e49` |
| `src/recording.rs` | `fcf9213788e072bd422055e9aed76a0b24ba458c8a3951070d7c70eef954a995` |
| `src/voxel_material/step.rs` | `132222b19f6750da9081762e2c841e9e11c39cabca8ac10e7d67dfd9fd0819e8` |
| `src/operator.rs` | `6fdc9de97d7900a59421f63e7c6123e7f43ae5afc4dced7e36ecb1fbbe23cba8` |
| `src/parallel.rs` | `ee5420673a59ae4fb247b3542878e74902f7a87056744d4303fb400be074677e` |

No reviewer action was denied, pending or uncertain. Prior denied external actions and uncertain Library uploads were not retried. Main and unrelated work were untouched. The parent owns correction/admission and subsequent implemented-source review; this prospective report does not change plan status.

## Targeted R1/R2 admission rereview

On 2026-10-22 the same separate reviewer read the corrected prospective admission and checked it against the complete findings above and the unchanged source boundaries. Only this report was appended. No build, test, native, simulation, benchmark or external action was performed.

**Current prospective design disposition: R1 and R2 are resolved. No remaining admission-design correction was found in this targeted scope.** This permits the parent to decide admission of the proposed worker slice; it does not accept an implementation, change plan status or establish M4/A5/A6/native exposure. The specified capacity gates, lifetime accounting and execution constraints still have to be implemented and reviewed against their deciding fixtures.

### R1 — resolved as a prospective contract

The admission now names the 384 MiB value as a reserved admission budget for enumerated payload groups. It explicitly excludes egui CPU shapes/tessellation/paint staging alongside the other stated exclusions, distinguishes the inherited surface-array reserve from a measured renderer total, and acknowledges that checking returned capacities cannot bound allocator peaks. It requires actual numerical/preview/raw/decoded sub-bound checks before owner installation/publication, including outer/inner history capacities, Inspector slots, frame/Arc metadata, density capacity and exact `edge³` shape. Constructor plus all requested intervals remain reserved. A durable append followed by preview refusal is explicitly recorded-but-not-presented, with no append retry or record erasure.

The 48 MiB enumeration is honest **as an implementation obligation**, not a proof that every current serialization path already stays within it. The fixed authoring groups are the existing 32 MiB history, 8 MiB candidate/preparation and at most 0.5 MiB dense u16 document; the remaining 7.5 MiB must contain actual simultaneous frozen ID/F, recipe/Request and JSON/base64/staging copies. The input/profile owners do have 1 MiB encoded-file bounds, but those do not imply vector capacity equals length or that serialization creates only one buffer. The corrected admission expressly requires the final implementation to enumerate real simultaneous copies, release temporary owners at their boundary and refuse if the envelope does not cover them. That qualification closes the prospective finding without inventing a capacity or peak-allocation result. Do not treat a file-length check alone as satisfying it during source review.

The accepted-stage estimate including pressure is now the maximum for exactly three concurrent publication/raw/decoded array groups. Retaining no extra decoded generation/producer copy and clearing old selection are explicit lifetime conditions. Retained old App context must be calculated from production owners before exposure; a zero-context isolated Worker fixture cannot replace that calculation. Saved numerical policy remains unchanged, and admission may refuse combinations whose sum exceeds the cap. These are proportionate obligations for the new worker mode rather than a request to rewrite all legacy allocation owners.

### R2 — resolved as a prospective contract

The new mode explicitly requires `Request.threads == 1`, actual Jacobi pressure and the matching compiled voxel executor declaration. Legacy thread policy is unchanged. Unspecified build revision/hash remains an explicit provenance limitation, and all captured method/executor/numerical/time/record limits must match before an output-directory helper is called. This closes the serial-declaration ambiguity without claiming authentication or cross-build reproducibility.

The related host-policy choice is also now explicit: native side `[0.001,100] m`, 4–64 cubic equal-spacing zero-origin cells, the existing 1000 cells/m restriction for new capture, and an immutable saved-spacing exception within that native host envelope. Unsupported general saved grids remain inspectable with an actionable native-run refusal. The exception neither rewrites geometry nor impersonates a legacy saved variant. Explicit preview requests retain their own 1000/m policy, while Adaptive is available. This is a defensible host/presentation boundary; source must still preserve original frozen spacing bits rather than reconstruct them through scalar side arithmetic.

Corrected admission SHA-256: `fc88d7a26b4c7e9513d38556a7e0a688b78d8aa1322469905a245ccc8646afb5`. Rechecked Request/Worker, history, viewport, compiled identity, frozen-input and recording sources retain the hashes in the initial table. No production source was changed by this rereview. No action was denied, pending or uncertain; previous external denials and uncertain Library delivery remained untouched.
