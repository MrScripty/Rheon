# Independent M4 typed recording component review

Review date: 2026-10-22. Reviewer: separate agent `/root/voxel_implementation_review`, distinct from the implementer. Reviewed the new `src/recording/voxel_material.rs`, additive recording exports, `tests/voxel_material_recording_contract.rs`, component admission, existing SQLite field/payload/reader/writer contracts and the actual accepted-state/report owner. Only this report was written. No tests, builds, native app, simulations, benchmarks or external mutations were executed by this reviewer.

**Initial disposition: one required resource-contract correction below.** No other required correction was found in this complete source/fixture round. Source review and the implementer's three reported passing fixtures do not accept full M4, A5/A6, native worker integration, physical/scientific accuracy or performance.

## Complete required findings round

### R1 — Medium: the exported publication estimate cannot satisfy its own producer admission

Locations: `voxel_material_publication_bytes`, `src/recording/voxel_material.rs:167–180`; producer admission at `206–210`.

The public helper advertises conservative admission for one publication including metadata and Vec/String overhead, but returns only field bytes plus 4096. The producer then requires that result **plus** `controls.len()` before allocating its fields. For either constructor or accepted publication, controls is nonempty. Therefore using the exported estimate itself as the producer's limit always refuses at the early payload check, even on the ordinary 4³ fixture. A caller cannot use this advertised helper as the sufficient per-publication budget. Its name/docs do not tell a caller to add the private formatted controls separately, and that opaque string length is not a stable external contract.

Required correction: give the caller and producer one consistent definition of the required publication budget. Either provide a conservative complete bound (with an explicit bounded controls/metadata allowance) and use it consistently, or expose an accurately named partial estimate plus the missing explicit bounded input. Keep actual Vec/String capacity and serialized payload checks after allocation; those are valuable safeguards and must not be removed. Do not make a later native worker guess another magic allowance or parse Debug text to preflight a record.

Add a focused constructor and accepted-publication consistency fixture: the public admitted budget must suffice under the declared helper contract, and a lower true field/payload budget must refuse before copying large numerical fields. Existing one-byte refusal and cancellation fixtures should remain. This is a public resource API correction, not a request to benchmark or change numerical tolerances.

## Confirmed metadata and provenance

**The fixed metadata framing matches its typed decoder.** Solver identity fixes the filled, shared-velocity, closed free-slip, no-surface-tension matched-donor/strain/Jacobi method. `RVM1:` is followed by exactly 256 lowercase hex characters representing 128 bytes. Bytes 0–79 contain ten little-endian f64 values: liquid/gas rho/mu, three gravity components and three pressure tolerances. Bytes 80–95 contain iteration and numerical-memory limits as u64; the last 32 bytes capture the initial-input digest. Decode checks solver/prefix/length/hex before fixed-offset reads, reconstructs MaterialProperties with their checked constructor and converts integer words to the target index type. Grid count/config/policy checks occur before new dense decoded fields. This is explicit framing, not serde, Debug parsing or current-preset replay.

`voxel_material_run` captures the actual simulation's immutable materials/configuration/geometry and caller-supplied SessionIdentity/provenance/digest. It does not reconstruct a recipe from a name. The digest is a binding supplied by the caller; this library adapter cannot prove it corresponds to an external FrozenInput. Later M4 composition must compare/bind that real immutable recipe and constructor before publishing a run. An arbitrary digest is not itself cold-rerunnable input. Public inspection metadata supports checked actual coefficients; this component does not independently qualify arbitrary material pairs for native exposure.

The metadata bit codec preserves negative zero and exact finite coefficient/configuration bits. The fixture compares coefficient and gravity bits plus the digest/geometry. Its current oracle does not independently assert every literal byte offset or pressure/memory field; an additional small literal framing/configuration roundtrip case would strengthen this format if it changes. The present source mapping is explicit and consistent, so this is an optional test observation, not a second required finding.

## Accepted-state and pressure semantics

**Producer values come only from the accepted owner.** Constructor is generation/time zero and has no pressure field. Accepted publication requires a last report with exactly the accepted generation/time. It serializes authoritative f64 F and each f64 MAC velocity directly to little-endian bytes; no binary material IDs, f32 preview, SDF/mesh or derived density channel substitutes for F. Gas is occupied `1−F`; rho/mu are reconstructed from the captured endpoint coefficients. Field names, SI units, staggered offsets, dimensions, x-fast order and stages explicitly describe the data.

The numerical owner copies projection pressure into its candidate and swaps candidate/state/report only after all acceptance/cancellation gates. A failed or cancelled next attempt leaves the old state and matching last report intact. This recording adapter correctly republishes that same old accepted publication, rather than treating rejected scratch as a newer result. This is the two-material owner's actual contract; it need not copy the legacy carrier's different pressure-invalidation policy.

Pressure is present only for an accepted interval, with cell-0 zero gauge, Pa units and explicit `not endpoint pressure` stage. Its recorded start is reconstructed as accepted time minus report dt. This is a floating-point interval association checked to the existing timing tolerance, not an independent preservation of the previous time's exact bits or an EOS/end-state pressure claim. The consumer requires a finite nonnegative start strictly before publication time and a consistent dt/interval. Generic publication validation also rejects impossible constructor/accepted generation/time/dt metadata. The actual pressure values are the accepted state array.

The tests include a real BeforeCommit cancellation with a reached-stage flag and exact `Cancelled` category, then compare old versus new publication fields, controls and time. They do not relabel a generic early failure as this late cancellation. They also check constructor pressure absence, accepted pressure values and retained decoded data independent of later evolution.

## Decoder shape, resource and malformed-state checks

The typed decoder first invokes generic run/publication validation. That checks field scalar widths, dimension products, exact byte lengths, finite scalar data, finite field evaluation times, duplicate field names and valid constructor/accepted metadata. Method/configuration framing and 4–64 axis counts are then checked before decoding vectors. Exact expected field count together with required unique names admits precisely four constructor fields or five accepted fields. Missing, extra, duplicate or renamed inventory cannot silently drop a field.

Each required field must match f64 type, units, Cartesian association/offset/order, stage, evaluation convention, shape and one component. Thus `chunks_exact(8)` follows a proven exact byte shape; it does not discard a truncated trailing value. Fractions must remain in [0,1]. Constructor velocity must be zero, and every closed outer normal face velocity must be zero. Pressure interval and gauge checks run before returning a successful owned view. Unknown methods/schema/range/wall/time/gauge/inventory cases in the fixtures target those owning checks. No legacy codec fallback or method-name coercion is added.

The output owns new f64 vectors and never aliases raw SQLite blobs or live simulation buffers. Counts bound decoded dense fields to the admitted lattice. `decode` has no caller-specific memory-limit parameter; it adds these bounded output vectors while the input RecordedStep remains live. A consumer must separately preflight the input plus decoded arrays, previews/cached geometry and process overhead, as the helper's docs state. The SQLite reader already checks selected raw payload/count aggregates and file limits before copying blobs; this new decoder does not replace that owner. Arbitrary already-owned oversized caller inputs are not proof of an unbounded new decoded allocation, because the typed dimensions/counts still have to match the capped grid before values are copied.

The producer checks a nominal bound before numerical field copies and checks actual capacities afterward: RecordedStep/field-container metadata, each field's data and string capacities, and controls capacity, plus the generic serialized logical payload count. These are managed owner bytes, not allocator/RSS/GPU measurements. Field multiplication and public estimator arithmetic are bounded/checked for the capped numerical grid; the actual capacity sum's bounded cardinality does not admit caller-controlled unbounded field strings. R1 concerns agreement of the exported estimate and early admission, rather than omission of these final actual-capacity checks.

## SQLite, diagnostics and remaining integration gates

The new adapter reuses StepRecorder/RecordingReader without modifying schema, legacy codecs or lifecycle. Existing owner checks adjacent generation/time, ordinal, record count/payload/database limits, field/run digests and accepted append transactions. Append uncertainty remains that owner's result; a new read reconciles actual committed prefix before a completion claim. The new component neither retries an uncertain append nor rolls physics back to match disk. Its actual SQLite fixture writes constructor plus three accepted 4³ gravity intervals, finalizes, reopens and decodes the final publication. This is useful component persistence evidence; it is not new whole-feature scientific validation or an actual native worker/cold saved-run workflow.

Per-step `controls` currently stores a Debug-rendered report as **opaque provenance**. The fixed metadata and typed physical fields do not parse it or use it for replay, which is appropriate. Phase totals/masses and acceptance diagnostics in that text are not yet a typed diagnostic API. Later retained Inspector/recording integration must either expose them honestly as opaque provenance or add an explicitly versioned typed diagnostic projection for the canonical plan's phase-volume/mass/closure/continuity/budget views. It must not reconstruct numerical intent or populate quantitative fields by parsing Debug names. This is a stated later integration obligation, not a demand for a general diagnostics framework in this component.

The admitted subset contains no worker, native launch, SavedRunSet/source/comparison dispatch, preview/Inspector mapping or immutable saved recipe publication. Those consumers still have to bind the real input digest, metadata/configuration, retained step identity and limits and explicitly refuse unsupported features/comparisons. Recording/header fields cannot authorize exposing a new runnable native method. Full M4 and A5/A6 remain pending; comparisons, restart state, thermal fields, performance and physical GPU validation are out of scope.

## Execution evidence and initial identities

The implementer reports all three scoped tests passed after correcting initial harness API signatures, without numerical/tolerance changes. I inspected their source but executed no workloads. No actual append-fault stage or native/system acceptance is inferred from their presence or from compilation.

| Path | SHA-256 |
| --- | --- |
| `src/recording/voxel_material.rs` | `af3e8091596423ed0c4a71e0bf8b9509e3cfb604b71cb351f75fc87923748f8a` |
| `src/recording.rs` | `3d93cba0f4bdb1ea88fec4865a56dcdfc6b328d497b350457845bc33c079037e` |
| `tests/voxel_material_recording_contract.rs` | `457201d303366897a4c63bb3528015aa749964d804531e1179864843a0c690f0` |
| `reports/m4-recording-admission.md` | `5706ed4f1c0e2902ffd89406fb6cddfab53d1df14ea2d6d7faa79af8252e98d0` |

The complete required finding is returned for implementer-owned serial correction, affected execution and targeted independent rereview. Only this new report was written. No reviewer action was denied, pending or uncertain; prior denied external operations and uncertain Library writes were untouched.

## Targeted R1 correction rereview

On 2026-10-22 the same separate reviewer inspected the frozen serial R1 correction and its additional fixture. Only this existing report was appended. I executed no test/build/native/numerical workload or external action.

**R1 — resolved.** The exported helper now computes checked raw field bytes plus `4096 + MAX_TEXT`, where the existing owner bounds valid controls text to MAX_TEXT. The producer validates that actual controls text and compares the single exported estimate with the caller's limit; it no longer charges the same privately formatted controls as an additional unknown requirement. Actual field-container/string/data capacities and generic validated payload remain checked afterward. The public estimate and producer preflight consequently have one consistent definition, while allocator failure or unexpected actual capacities can still cause a typed refusal. No opaque report parsing, second worker allowance, new codec or numerical change was introduced.

The added `exported_publication_budget_admits_constructor_and_accepted_payload` fixture calls the public helper for both constructor and actual accepted states and requires production to succeed using exactly that returned budget. It also requires a one-byte-below estimate to fail with the typed preflight payload category. The original one-byte total refusal, exact SQLite field roundtrip, malformed-state checks and reached-BeforeCommit cancellation assertions remain unchanged. The conservative estimate is a supported admission bound, not a claim of the smallest possible allocation or measured memory consumption.

**Current independent source-review disposition: all required recording-component findings are resolved; no further required correction was found in this targeted scope.** The implementer had begun final default/parallel four-fixture execution and strict desktop checks at this review boundary; their outcome is not predeclared here. The earlier reported three-fixture results and legacy recording regressions do not substitute for the affected final execution. Component admission remains implementer/parent-owned after those checks.

The decoder/output-budget distinction, opaque diagnostic provenance and complete immutable-input binding obligations recorded above remain explicit later integration gates. Nothing in this source disposition accepts full M4/A5/A6, cold native saved-run execution, restart state, physical GPU behavior, scientific validity or performance. The existing SQLite append uncertainty owner and all legacy codecs are unchanged.

### Corrected identities

| Path | SHA-256 |
| --- | --- |
| `src/recording/voxel_material.rs` | `729019c418a52f21e66200a0a2726b1f71ab6da11c8d3ac2876747f416cc3129` |
| `src/recording.rs` | `3d93cba0f4bdb1ea88fec4865a56dcdfc6b328d497b350457845bc33c079037e` |
| `tests/voxel_material_recording_contract.rs` | `87d2d4dc04dd86b86027d434bee43adf81e8a8e5b3aa8c40491685aa7b3c7b9e` |
| `reports/m4-recording-admission.md` | `5706ed4f1c0e2902ffd89406fb6cddfab53d1df14ea2d6d7faa79af8252e98d0` |

Initial identities remain historical. No reviewer action was denied, pending or uncertain.

## Exact interval-start metadata extension

The implementer subsequently identified the precision limit noted in this report's initial pressure discussion: recovering the previous clock by inverse subtraction need not preserve its bits. The admitted write set was narrowly expanded to `src/voxel_material/step.rs` for one success-report field, and this separate reviewer inspected the complete extension and added fixture. Only this review report is appended; no workload or external action was executed.

**Disposition: no required source correction found for this extension.** `VoxelMaterialStepReport::interval_start` is assigned from `self.state.time`, the still-authoritative old accepted clock, when the disposable report is formed. The numerical time update remains the same checked `self.state.time + dt`; generation, transport, stress, gravity, projection, acceptance arithmetic, allocation and retry formulas are unchanged. The recording producer now uses this captured field as the pressure evaluation time rather than subtracting dt from the new clock. This supersedes the initial review's statement that the producer stores a reconstructed interval start; the returned/stored timestamp now preserves the original accepted clock bits.

The capture is correctly inside the success boundary. `self.state` is unchanged throughout a failed attempt and throughout shortening retries, so every retry sees the same original interval-start clock. Its report uses the dt of that actual attempt. The BeforeCommit callback still occurs before the state/report swap. If it cancels, the new local report and its timestamp are discarded and the previously accepted report remains intact. An earlier arithmetic/pressure/retry failure likewise cannot replace `self.report`. On success, report, accepted state, time and generation are published together through the existing infallible final assignments. The new field does not convert a rejected candidate timestamp into accepted evidence.

The new `interval_timestamp_preserves_original_clock_instead_of_inverse_subtraction` fixture creates a zero-gravity rest owner from the same captured grid/material/F/configuration, performs three actual requested dt=0.1 steps, and on each interval compares report and produced pressure-evaluation timestamp bits with the pre-step authoritative clock. It additionally requires at least one inverse-subtraction mismatch, so a future fixture that accidentally stops reaching the floating-point counterexample cannot pass as coverage. At the third such interval, rounded addition to approximately 0.30000000000000004 followed by subtraction of 0.1 need not recover the stored 0.2 clock. This is a focused timestamp oracle, not a new moving-flow/scientific validation claim.

The existing reached-BeforeCommit cancellation fixture also compares publication controls and fields after cancellation; the unchanged old accepted report now includes its captured start. Existing retry/step contracts remain the implementer's affected checks. No new restart state, metadata parser, physical field, 128-byte run-header framing or archive schema is introduced. Generic pressure-interval adjacency validation still uses the existing arithmetic tolerance, while stored scalar bits remain exact. The stack/value report grows by one f64; the numerical budget explicitly measures managed array payload rather than claiming report-struct bytes or process RSS.

The implementer was running default/parallel five-recording-fixture, existing step-contract and strict desktop checks at this review boundary. Their final outcome is not predeclared here. Source-review disposition for this recording component remains clear of required findings, conditional on implementer-owned affected execution before admission. Full M4/A5/A6, native execution and all prior scope limits remain pending.

### Interval-start extension identities

| Path | SHA-256 |
| --- | --- |
| `src/voxel_material/step.rs` | `132222b19f6750da9081762e2c841e9e11c39cabca8ac10e7d67dfd9fd0819e8` |
| `src/recording/voxel_material.rs` | `4d349abe6a20a4300f8b2559cb23fd157c231f9315c5a7fb5b11e8f8d691eca3` |
| `src/recording.rs` | `3d93cba0f4bdb1ea88fec4865a56dcdfc6b328d497b350457845bc33c079037e` |
| `tests/voxel_material_recording_contract.rs` | `f5a302e4dba2c9f6f471fe176a56db7b5954a6b9d2fa7682e1e5a3f68a8a8b37` |
| `reports/m4-recording-admission.md` | `dacba4ec8d29346bb6b5577e9c09fbed2eef9fd12ae7eceb11c1bb161cce173c` |

Only this existing report was appended. No reviewer action was denied, pending or uncertain.
