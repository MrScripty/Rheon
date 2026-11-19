# Independent M4 existing-worker implementation review

Review date: 2026-10-22. Reviewer: separate agent `/root/voxel_implementation_review`, distinct from the implementer. Reviewed every dirty tracked production change, the new `voxel_material_execution.rs` and integration fixtures, corrected admission, prior prospective review and the reused numerical/recording/persistence/Inspector owners. Only this report was written. No tests, builds, native sessions, simulations, benchmarks or external mutations were executed by this reviewer.

**Initial disposition: two bounded required corrections/evidence requirements below.** R1 is a real aggregate-lifetime accounting defect at final pinning. R2 is a missing target-bound oracle for the new visual projection, not a claim that its current source formula is wrong. No additional required source correction was found in this complete round. The component remains pending implementer correction and affected execution. Whole M4, App adoption, actual native run/save/load and A5/A6 remain pending.

## Complete required findings round

### R1 — Medium: final pinning retains four publication groups, but admission charges three

Locations: `voxel_material_execution.rs:137–145`, `219–251`, `332–339`; reused `run_sets/voxel_material/run_set.rs::pin_closed` / `publications`.

The worker selects the constructor in `history.carrier_inspectors[0]`, which retains both its raw RecordedStep and decoded RecordedVoxelMaterial. UI selection can later replace that pair with a full accepted raw/decoded pair. Neither pair is released or excluded when the worker closes the recorder and invokes `pin_closed`. That audit separately selects each raw publication and decodes its F/MAC/pressure arrays. During `publications`, the Inspector's raw+decoded pair and the audit's raw+decoded pair therefore coexist: **four** groups, even in the isolated worker without an App. At native use the two pairs can both be full accepted publications. The numerical simulation also remains alive at this point, correctly charged by its declared limit.

The reserved sum contains only three accepted-publication estimates. Normal stepping respects that three-group arrangement: the producer publication is explicitly dropped before preview capture, while the retained selected raw/decoded pair occupies two groups. That correct live-step ordering does not cover final audit overlap. Per-record capacity checks do not repair an omitted fourth simultaneous owner. A request near the 384 MiB admission edge can be admitted against an undercounted sum.

Required correction: account the fourth accepted-publication group at the same aggregate cap and update the admission/lifetime explanation, or enforce a real mutual lifetime exclusion that remains correct under concurrent UI selection. Reserving four is the proportionate fix; clearing Inspector opportunistically would not exclude a simultaneous UI reselect and needlessly disrupt retained results. No new worker, lifecycle framework, decoder or numerical allocation is needed. Keep the producer drop and actual per-group bounds.

Add a targeted pre-output boundary case that would fit the old three-group sum but exceeds the corrected four-group sum, requiring typed/actionable refusal and no output. This can use a small input plus controlled retained-context bytes; it does not require allocating a large grid, measuring RSS or running a new numerical cohort. Re-execute affected worker/persistence fixtures after the serial correction. Preserve the cap and captured numerical policy rather than silently increasing either to avoid the new refusal.

### R2 — Low: positive fixtures do not verify the new fraction appearance values

Locations: `history.rs::PreviewFrame::capture_voxel`; `voxel_material_integration_tests.rs:138–165`, `243–259`.

The new producer implements its own f64-F-to-byte max-resampling path. Positive worker fixtures assert exact recorded fractions/velocities and frame report/time tags, while the history negative fixture asserts shape/capacity rejection. None asserts any `frame.density` value. An all-zero image, phase swap or wrong source-axis mapping in this new function could pass those checks while the numerical archive remains correct. The result thumbnail likewise has only a length assertion. Existing legacy tracer preview tests exercise another producer and do not close this gap.

Required focused evidence: use the actual new capture/publication path with a known original-index mask and independently stated expected byte image, including occupied gas mapping to transparent zero and liquid to visible values. Include one nondivisible or finer preview mapping if retaining the new resampling claim, and one known nonbinary F byte check where a small actual accepted state supplies mixed fractions. Avoid an oracle that duplicates the new nested loops. Check that projection leaves original f64 state/geometry unchanged and retains the explicit voxel tag; no native graphics or scientific validation is implied. A few fixed cells/planes suffice. This is a deciding test for the new display behavior, not a request for broad UI or renderer qualification.

## Confirmed numerical and Request boundary

The new VoxelRequest holds an Arc to one validated immutable recipe/revision and the retained-payload context. It contains no Simulation and Clone shares that immutable allocation. Validation is a dedicated early branch; execution branches at the beginning of the existing `run_simulations`, before legacy particle/carrier construction. The existing Worker owns cancellation, scheduler, join and its one pool/thread path. No second numerical runtime or worker manager was added.

The validator requires exactly one actual Jacobi track and one Rayon thread. It refuses particle, saved-native, configured/provider, emitter, collision mesh, observer and authored-force combinations. None force slots do not supply forces. Captured schedule/dt bits, recording limits and the compiled executor declaration must match. The compiled identity owner remains explicitly a declaration with possible unspecified source/material values, not authentication or a reproducible-build certificate. Legacy validator/thread behavior is unchanged.

Native shape/side checks use restored captured geometry. The solver is constructed through FrozenInput rather than `domain_side / size`, preserving original spacing and signed-zero origin bits. The new 6³, h=0.1 fixture meaningfully exposes an actual scalar multiplication/division bit discrepancy and checks original recorded spacing. The saved-spacing/independent-preview policy is explicit, including actionable unsupported geometry. New authoring capture's 1000/m admission remains the later native capture owner's obligation; the generic frozen-input component is intentionally more general and this worker executes immutable saved intent. No cold inspection constructs a solver.

The worker calls only the already reviewed coupled `sim.step`. It does not reimplement pressure, transport, viscosity, density composition or retry arithmetic. It records each actual accepted report/dt and the original captured maximum remains unchanged. Actual ratio-shortened steps are exercised rather than filtered by the legacy fixed-dt guard. Completion refers to the captured accepted interval count and actual elapsed time, not an invented fixed-duration horizon. No thermal, phase-change, surface-tension, compressible-gas or vacuum evolution capability is introduced.

## Publication, cancellation and immutable attempt chain

The order is correct for constructor and accepted intervals: capture exact publication, append through StepRecorder, drop producer arrays, then capture/publish appearance. Rejected numerical candidates do not append. A preview failure after durable append returns explicitly recorded-but-not-presented; it does not retry append or erase a record. Existing StepRecorder poisons on append failure and attempts terminal status at most once. The new wrapper preserves that policy and reports accepted generation versus acknowledged publication count, including a possible unrecorded accepted tail.

Cancellation is checked before output preparation and before each step, and passed into the real step's cancellation callback. The explicit `cancelled` disposition is set by an observed cancellation boundary/error rather than merely relabelling an unrelated failure because the flag later changed. Timing is ended after the numerical call. A late flag after completed work does not change the recorder's completed outcome here. Later App polling/adoption still needs its own truthful terminal-status handling; current legacy App behavior is not accepted by this component.

Policy/executor/aggregate validation and actual numerical construction occur before the directory helper creates output. The helper's side effect is identified in source. Intent, started attempt and finalized child revisions use the existing no-replace owner; their parent digests form an immutable chain. Source/current prior run IDs are excluded for a fresh attempt. Exact spec remains unchanged across a rerun. The child process fixture opens the original saved profile and invokes the same Worker, rather than directly substituting a helper stepper.

The recorder is dropped before closed-archive hashing/verification/pinning. Pin failure leaves no finalized pin and is reported; terminal write failure returns the exact archive path and uncertainty without another append/terminal attempt. Publication errors retain the protected path and reconciliation instruction. A failed numerical/append attempt may still obtain a confirmed `failed` terminal record and a valid pin. `MemberState::Ready` then means ready for inspection of that failed attempt, not successful simulation completion. The record-limit fixture's comment saying never promote the poisoned attempt to Ready should be read/corrected with that distinction: its actual assertions now require a confirmed failed terminal and finalized pin, which are legitimate. Do not summarize that fixture as proving all failures remain unpinned.

## Capacity accounting and presentation scope

The new history owner reserves constructor plus every requested interval, counts actual outer/inner vector slots and inline owner/Inspector metadata, and checks actual density capacity/shape before installation. It retains a bounded explicit default metadata reserve and charges selected raw/decoded arrays separately. Checked arithmetic handles aggregate overflow. The new raw publication-capacity helper and decoded f64-capacity check refuse over-capacity selections without changing the typed recording schema or numerical formula. The raw-capacity negative targets the actual decode boundary.

The staging computation uses actual immutable recipe/member/string and Request-vector capacities, conservatively multiplied for the finite recipe/serialization owner set, plus the specified editor and bounded JSON/preparation reserves. It does not copy numerical state into Request. Inline capacity double-counting in the immutable recipe calculation is conservative. The 48 MiB figure remains a source-based staging admission reserve, not measured peak allocation or an allocator promise; a file-size bound alone must not be described as an actual-capacity measurement of every serialization temporary. The temporary restored IDs/F remain within the stated bounded preparation owner, rather than a new unbounded retained generation. No production old-result accounting is implemented or inferred from isolated zero-context fixtures.

The 80 MiB renderer reserve retains its narrow surface-array meaning. Egui CPU tessellation/staging, GPU/process/SQLite/allocator overhead and unrelated views remain explicitly outside the reserved sum. R1 concerns a concrete omitted in-scope owner, not those honest exclusions or a request to prove total RSS. Existing source caps are not executed performance evidence.

An explicit voxel preview tag carries an optional actual VoxelMaterialStepReport; legacy StepReport and particle report remain absent. Constructor report is absent and accepted report retains its stamp. Fraction appearance is quantized/max-resampled and cannot become an authoritative mixed-cell or volume measure. The full f64 typed archive/Inspector remains the both-phase boundary. Gas transparency is explicitly labelled occupied; generic retained Inspector role names remain honest, while this qualified native recipe identifies Water/Air.

The tracked presentation changes reach running setup, method/track labels, timeline/report caption, selected-Inspector routing, nonzero-liquid explanation and result tooltip. The numerical Inspector sidebar already routes material=None requests to the per-track Inspector, so the new backend reaches the intended owner. Viewport tags force Appearance and disable inappropriate particle/lab/session modes. R2 is the remaining image-value evidence gap, not an observed misrouting. Existing direct Request/PreviewFrame fixtures changed only by None fields; the separate AI Request, old recording schemas, numerical formulas and unrelated work were untouched.

## Executed-evidence limits and remaining gates

Read-only inspection of the implementer's latest `.generated/voxel-material/m4-worker-tests.log` shows 9 parent fixtures passed, 0 failed and 1 deliberately ignored child entrypoint; that child was explicitly launched and passed separately. The log includes the actual generation-6 DiskFull append refusal, confirmed failed terminal/pin and retained prefix. The assertions avoid relying on an exact SQLite fill generation. Strict desktop lint's log finishes successfully. Affected history, viewport, timeline, force UI, configured Worker and setup logs also show passing selected results; the setup suite retains its separately ignored genuine-cancel harness. Recording's affected log shows 5 passing fixtures and the retained Inspector's selected logs show passing results. These are implementer-executed checks read from logs, not executions performed by this reviewer or acceptance of untested paths.

The new tests use actual SQLite and existing Worker start/join, exact original fields, child-process rerun, ratio-shortened steps, mutation of later Draft state, pre-output validation, cancellation and a real recording limit. The cold equality oracle reuses the same numerical method in another process; it establishes exercised replay, not independent mathematical accuracy. The previously accepted numerical oracles remain the numerical component evidence. No claims of native-window behavior, practical speed, total memory/RSS, physical GPU or broad scientific validity are made.

Normal App launch from Initial contents and saved voxel rerun remain explicitly unavailable. `App::launch_request` still adopts old-style workers eagerly; this review does not accept that behavior for the new mode. Production retained-context calculation, staged constructor acknowledgment/adoption, preserved old results on failed start, identity/load/reset ownership, native save/load/run/inspect and actual A5/A6 remain mandatory later gates. Main stays frozen under the separate integration lead. This report grants no push, PR, merge or release permission.

## Source identities at complete findings round

| Path | SHA-256 |
| --- | --- |
| `reports/m4-worker-integration-admission.md` | `f40d2c681f22eea721f3c7b7baf9b773d61f51e58e9049ab2eff1c3d16864c12` |
| `src/bin/desktop/voxel_material_execution.rs` | `1536198ef9640d62a5757aedc48520afcb34af74397fe33de23f4dfd9c9a251b` |
| `src/bin/desktop/voxel_material_integration_tests.rs` | `e81ff8bd985777d1e24c80b0a879a85139bdefc7ce5c01b9104aedb9cbde84d6` |
| `src/bin/rheon-desktop.rs` | `c1e76932e517e439424013c1778578fb053d5b76e46cf2c92a0e68dfafd28611` |
| `src/bin/desktop/history.rs` | `543175ebae4ac8b951d857077b21bf1aef9ccaa0bbd675a54e4eb6a2b88029c9` |
| `src/bin/desktop/viewport.rs` | `5ffcfe2ec6a6b1aff087d5c17c37eecb82e4f724e351471225d62a09a4fbd635` |
| `src/bin/desktop/run_sets/voxel_material.rs` | `6a5b1f0cffea4992476c9ae6b9c8a88061fe89ddec8f4fde833a356479ccb8e0` |
| `src/bin/desktop/run_sets/voxel_material/run_set.rs` | `6165917638619e9f1f250bae017e0381780b2838c0522384ecc3c859ac128958` |
| `src/recording/voxel_material.rs` | `35148ff3972a7e22e4dea75d6c665d6a8437103e6dec1d818e56a479c0588e05` |

These identify reviewed bytes before semantic corrections, not executed certification. Only this report was written. No reviewer action was denied, pending or uncertain. Historical denied external operations and uncertain Library uploads were untouched. Parent/implementer owns serial corrections, affected execution and component admission, followed by targeted independent rereview.

## Targeted R1/R2 correction rereview

On 2026-10-22 the same separate reviewer inspected the serial corrections, added fixtures and final implementer logs. Only this report was appended. No tests, build, native session, numerical workload or external mutation was executed by this reviewer.

**Current independent source-review disposition: R1 and R2 are resolved. No further required correction was found in this targeted scope.** Parent/implementer retains component-admission ownership. Full M4, production retained-context/adoption, actual native workflow, A5/A6 and release/main integration remain pending; these bounded worker checks do not satisfy those gates.

### R1 — resolved

The checked reserved sum now charges four copies of the accepted-stage publication estimate while keeping the 384 MiB cap, numerical limit and remaining groups unchanged. Admission explains that live stepping uses at most three groups, whereas final pinning can retain the Inspector raw/decoded pair alongside the independent audit raw/decoded pair. The producer is already dropped before preview and before pinning. No Inspector clearing race, new worker/lifecycle authority or policy rewrite was introduced to avoid the overlap.

The new small-input boundary fixture sets retained-context bytes so the old three-group formula would equal the cap. Actual Worker::start now refuses that request before creating its output directory. Reducing the declared context by exactly one publication estimate reaches the corrected four-group boundary, where validation succeeds and still performs no output. That distinguishes the old and corrected formulas without allocating fictitious old payload or pretending it qualifies production context calculation. Checked accumulation still handles overflow and other capacities remain independently bounded.

### R2 — resolved

The actual new capture path now has explicit original-index byte oracles rather than only shape/report assertions. For the captured 4³ binary mask, fixed planes require the correct zero/255 phase appearance at edges 4, 5 and 3. The 5³ and 3³ oracles state the expected max-coverage replication for finer/nondivisible sampling; they do not duplicate the implementation's nested source-coverage loops. Source geometry/F are compared after all captures, and constructor tag/report absence is checked.

A separate real numerical owner starts with known mixed fractions 0.25 and 0.5 at fixed cells, executes an actual zero-gravity rest step, then captures the accepted state. Its exact output requires bytes 64 and 128 at those cells and zero elsewhere, positive-liquid cell count two and an accepted generation-1 voxel report. The test checks the original mixed F and captured geometry remain unchanged. This catches all-zero, binary-only or phase-inverted appearance while preserving the distinction between lossy display bytes and original f64 state. It is a bounded projection oracle, not transport accuracy or native rendering qualification.

The record-limit fixture comment now distinguishes confirmed failed-terminal inspection readiness from completed simulation. Its assertions still require the truthful failed terminal/pin and unrecorded-tail disclosure; no assertion or tolerance was weakened to satisfy the source correction.

### Executed evidence and final identities

Read-only inspection of final `.generated/voxel-material/m4-worker-tests.log` shows 11 parent fixtures passed, 0 failed and the one deliberately ignored child entrypoint. The parent explicitly executes that child in a fresh process, and its separate result is 1 passed. The same log still contains the real generation-6 DiskFull append refusal with preserved failed-terminal/pinned outcome. Final strict desktop all-target Clippy log ends successfully. These are implementer-executed results, not reviewer workloads. Earlier affected suite results remain as described above; no new numerical/native/performance claim is inferred.

| Path | Final SHA-256 |
| --- | --- |
| `reports/m4-worker-integration-admission.md` | `002980a41deb15852280f30a3d89132b8ca3044acbb45ca56719cd34eec475e4` |
| `src/bin/desktop/voxel_material_execution.rs` | `c2d04c805afb919ce7383da01818d64e6f8e8cffc11812f2fe7e4c87fd15f35b` |
| `src/bin/desktop/voxel_material_integration_tests.rs` | `af408b8b2611f0f56f0ff0455218093ab3a276deac59294e6dcfc14905915fc9` |
| `src/bin/rheon-desktop.rs` | `c1e76932e517e439424013c1778578fb053d5b76e46cf2c92a0e68dfafd28611` |
| `src/bin/desktop/history.rs` | `543175ebae4ac8b951d857077b21bf1aef9ccaa0bbd675a54e4eb6a2b88029c9` |
| `src/bin/desktop/viewport.rs` | `5ffcfe2ec6a6b1aff087d5c17c37eecb82e4f724e351471225d62a09a4fbd635` |
| `src/bin/desktop/run_sets/voxel_material.rs` | `6a5b1f0cffea4992476c9ae6b9c8a88061fe89ddec8f4fde833a356479ccb8e0` |
| `src/bin/desktop/run_sets/voxel_material/run_set.rs` | `6165917638619e9f1f250bae017e0381780b2838c0522384ecc3c859ac128958` |
| `src/recording/voxel_material.rs` | `35148ff3972a7e22e4dea75d6c665d6a8437103e6dec1d818e56a479c0588e05` |

No source outside the serial correction scope changed in these reviewed identities. No reviewer action was denied, pending or uncertain. Previously denied operations and uncertain Library uploads remained untouched.
