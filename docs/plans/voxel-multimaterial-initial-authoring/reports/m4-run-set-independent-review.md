# Independent M4 run-set and cold archive review

Review date: 2026-10-22. Reviewer: separate agent `/root/voxel_implementation_review`, distinct from the implementer. Reviewed the new voxel run-set owner and its fixtures, shared saved-source/enum consumers, comparison refusals and owner-local fixture placement, checked recording-header getter, generic manifest/path/publication/SQLite owners, and the actual numerical retry implementation. Only this new report was written. No test/build/native/numerical/benchmark workload or external mutation was executed.

**Initial disposition: two required corrections below.** No further required issue was found in this complete round. This component does not accept a native worker, retained-material UI, actual saved-run rerun, whole M4 or A5/A6. The implementer owns serial correction, affected execution and admission; an existing passing fixture cannot establish a schedule the solver does not use.

## Complete required findings round

### R1 — Medium: the archive audit invents a timestep-halving schedule

Location: `publications`, `run_sets/voxel_material/run_set.rs:278–294`; negative fixture `false_schedule_or_captured_binding_cannot_be_pinned`.

The cold audit admits only requested dt divided by two zero through seven times. The real `VoxelMaterialSimulation::step` does something different: on Courant or viscous-stability failure it computes the corresponding `limit/actual` or `1/number` ratio, multiplies the current attempted dt by that ratio, and takes `next_down`. It makes at most eight attempts from the same accepted state. Those accepted values generally are neither powers-of-two subdivisions nor bit-identical to an exact half. Genuine supported shortened intervals can therefore be accepted by the solver, recorded exactly and then refused by this new pin/open owner. Conversely a half-sized value is not proof that the declared request or retry algorithm was actually used.

The current 0.075-versus-0.1 negative fixture codifies the invented schedule. A shorter dt cannot be called invalid solely because its ratio is non-dyadic. The existing numerical contract explicitly permits bounded shortening with actual dt, and its previously executed step fixture already exercises real retries. There is no approved numerical change to halve intervals here.

Required correction: align the declared archive audit with the actual method. At minimum, check a normal positive accepted dt no larger than the frozen request plus exact saved start/clock continuity, and state honestly which retry-cause/attempt facts are not authenticated by inspection. If a stronger attempt-specific audit is required, capture explicit versioned typed facts from the actual producer; do not parse Debug controls or instantiate/reexecute a numerical owner on cold open. Do not change the solver, freeze an invented schedule, add a rounding epsilon that merely admits some dyadic cases, or silently suppress an acceptance check without correcting its documented meaning.

Add an archive fixture driven by the actual solver from its frozen request that demonstrably shortens dt (`attempts>1`, actual dt below request) and can pin/reopen. Retain a genuinely outside-contract case such as dt greater than the frozen request, exact clock/start mismatch or invalid generation. Replace the old halving assertion because its oracle is wrong, with that reason retained in evidence; this is not tolerance tuning to make an observed failure pass.

### R2 — Low: constructor binding compares numeric zeros rather than exact frozen bits

Locations: `bound_run` fraction/velocity checks and constructor branch of `publications`.

The component promises exact constructor binding, but fraction checks use `f != expected` and zero-velocity checks use `v != 0.0`. IEEE −0.0 and +0.0 compare equal. A numerical constructor supplied gas fractions of −0.0 can therefore bind to a frozen Air document whose conversion produces +0.0, and a self-consistent archive with altered zero-sign constructor fields can pass the cold audit. Their physical occupancy/mass is the same, but the original constructor bytes and bitwise reproducibility claim are different. Geometry, coefficients and configuration already receive explicit bit comparisons, so the fraction/initial-velocity boundary should be equally precise.

Required correction: compare the native recipe's canonical constructor F and zero MAC fields by `to_bits` in producer binding and the cold constructor audit. The numerical/inspection APIs can retain their broader mathematical zero convention; this exact recipe owner is the correct place for the tighter binding. Add a targeted −0.0 gas-fraction producer mismatch and self-consistently recorded zero-sign tamper refusal, alongside a positive canonical constructor case. Do not reinterpret the saved IDs, normalize an accepted mismatching archive into a new identity or change the numerical kernel merely to enforce the recipe boundary.

## Confirmed manifest and archive ownership

**The new profile is explicit and bounded.** Revision/envelope version 3 and `voxel_material_set` kind select one member whose ID must equal the baseline. The optional parent digest is captured immutable revision metadata, not a mutable latest pointer or proof that a parent file was loaded. Nested FrozenInput keeps its own version/kind/digest/original payload; the custom serde adapter validates it rather than reopening a preset or substituting current Draft. Outer and nested structures deny unknown fields. A file open is bounded at 1 MiB and uses the existing original-byte strict header/typed parser, preserving duplicate-key refusal. One-member validation follows bounded deserialization; it does not add a generic run registry or numerical owner.

`save_new` reuses the no-replace publisher. Existing bytes are not overwritten; its `PublicationUncertain` result is propagated, and no recovery deletion/retry is introduced. The immutable set digest covers member/attempt/pin declarations and the nested original recipe. Declared executor source must agree with the attempt. This is content identity and declared provenance, not execution authentication.

**Producer/header binding is comprehensive apart from R2.** `bound_run` requires constructor generation/time, matching geometry and phase assignment, zero initial velocity and the declared source revision before creating the header. `require_metadata` compares exact geometry bits, initial-input digest, all four captured rho/mu values, gravity, pressure tolerances/iteration cap and numerical memory limit. Its strict JSON Binding carries the input digest and complete executor declaration, including build provenance, and must match the frozen value. Header audit additionally binds run ID/source and all archive limits. The new public metadata getter reuses the existing fixed checked codec and allocates no numerical or selected-field arrays; this permits meaningful inspection even for a failed archive with no publications.

**Pinning remains explicit.** A pin cannot be replaced in place. The owner checks declarations and protected relative path components, hashes before opening, checks run/status and full generic/typed publication integrity, then hashes after closing the reader before publishing the pin. Reopen compares the exact pin/header/status/count/tail, reaudits and hashes after reading before returning Ready. Ordinary mutation during inspection or after finalization is detected. This relies on the existing protected caller-owned local path/immutable closed-artifact precondition; it is not a hostile-filesystem race-proof or a general cryptographic authentication claim. No user data is deleted as a fallback.

**Readiness states retain different meanings.** No attempt is NotRun; an absent artifact is Missing. A valid matching header with an unpinned active writer is Incomplete, and an unpinned terminal artifact is Unpinned. Neither is promoted to Ready. Full generic and typed publication audits occur for pinning and pinned Ready admission; an unpinned/incomplete status is not a certificate that every stored field is valid. Failed/cancelled Ready outcomes remain failed/cancelled, never Completed. The close-only producer precondition remains necessary.

The typed audit requires constructor first and exact generation ordinal sequence; generic reader verification supplies contiguous records and timing validity. The new owner checks total count against requested steps and requires full coverage for Completed. A failed/cancelled recorded prefix can report at most one later accepted but unrecorded generation, matching stop-on-append-failure custody; a zero-publication terminal artifact can report only accepted generation zero. Accepted generation may not precede the committed tail or exceed the request. These checks keep durable count, accepted tail and terminal outcome separate. Their empty-failure/cancelled-tail cases would benefit from focused tests as this owner integrates with the worker, but no additional source defect was found in the inspected arithmetic.

Cold open calls FrozenInput validation/restoration and typed inspection only. It does not call `construct`, resolve a provider, run a step or recover restart state. Temporary original-document, raw-record and decoded vectors are bounded by admitted input/record counts and existing limits; combined worker/retained-preview allocation admission is still a later integration gate. R1 must be corrected without replacing cold inspection with numerical replay.

## Reachable consumer audit

I searched both source and integration-test populations for SavedRunSet/SavedMember and their native/configured accessors. Current changes cover:

- `run_sets/source.rs`: explicit Native/Configured/Voxel variants, version-routed file open, validation/digests/labels/member iteration, limits/attempts and owner-specific open. Only an unsupported envelope version selects the next file owner; malformed/integrity/I/O failures do not trigger a different profile parser. Nested enum decoding validates the selected revision.
- `run_set_view.rs`: voxel label and owner-mediated member state/recheck; it does not infer Ready from a pin alone. Inspect currently routes into the existing raw archive surface. New typed voxel retained views remain a stated later gate, and this review does not confer them.
- `comparison.rs`: explicit voxel capture refusal and display branch before legacy field-shape extraction. `comparison_archive.rs` refuses capture and loaded recipe validation before source selection. `comparison_field_view.rs` refuses before legacy f32 inventory/byte estimation. The owner-local comparison fixture uses a nonexistent source root to prove Unsupported occurs before file opening. No comparison algorithm is added.
- `rheon-desktop.rs`: the new exhaustive rerun arm returns explicit unavailable status; it starts no new worker or legacy fallback.
- `tests/comparison_field_view_contract.rs` and `tests/saved_source_contract.rs`: native/configured-only fixture helpers now explicitly reject Voxel rather than accidentally treating it as a legacy variant. These narrow exhaustive-match changes preserve their existing assertions.

The comparison page's general footer still uses “not configured” to show an informational Native-v1 diagnostic limitation. For a loaded voxel source, an explicit voxel-unsupported footer would be clearer than that legacy wording. The actual computation paths already refuse before estimation/open; this is optional wording cleanup, not another required archive finding.

The comparison refusal fixture was correctly moved from the run-set child to `comparison_archive`'s own cfg(test) module: external integration crates reuse `run_sets.rs` without importing desktop comparison modules. The alias for voxel pressure policy avoids a name collision without changing the policy. The exact write-set expansion includes these reachable test consumers. No common README/index or unrelated source owner was modified by this reviewer.

## Fixture and evidence limits

Inspected owner fixtures cover explicit version-3 dispatch; NotRun/Missing/Incomplete/Unpinned/Ready progression; exact pin/reopen, no-replace bytes and changed-file refusal; malformed top-level duplicate/unknown keys and digest; captured configuration mismatch; and a self-consistent wrong constructor F that generic SQLite integrity alone cannot detect. The comparison refusal is now owner-local. R1 requires a real shortened-step success case and correction of its wrong negative oracle; R2 requires signed-zero bit-boundary cases.

The implementer reports the four initial owner fixtures and five configured-comparison regressions passed, with final relocated comparison/all-target checks in progress. I inspected source without rerunning any workload and do not predeclare those final results. These tests do not establish append fault-stage durability, native saved-run execution, immutable retained UI or scientific/performance acceptance. All whole-M4/A5/A6 gates remain explicit.

## Initial reviewed identities

These identify the complete initial round after the compiler/fixture-placement corrections, before serial semantic repairs.

| Path | SHA-256 |
| --- | --- |
| `src/bin/desktop/run_sets/voxel_material/run_set.rs` | `babd524669abb077d92bbb4ccc5fb989ca858330ab1fde33d997ed859d0c6fd6` |
| `src/bin/desktop/voxel_material_run_set_tests.rs` | `c3ac5fa73b5581f1641113a6a0b99dc2cc1e6d9558584e8c2bf8539d22b4296b` |
| `src/bin/desktop/run_sets/voxel_material.rs` | `cd43376b66c4f4cce454e31506bb0af8d5e9c003e82e438f93433c575e5a42d1` |
| `src/bin/desktop/run_sets/source.rs` | `dfb0ae266e9e0b5180225eb015a334013f31442815fd4496ac22a8c49098cc57` |
| `src/bin/desktop/run_set_view.rs` | `1c1720537c3f9ad001ace526b93164071d89d46b29ad209f67b3a762f1e0c47b` |
| `src/bin/desktop/comparison.rs` | `4daffd199ecb6ef877bed00ea26cf3b85a2788c70b02603acada146ce1444d5b` |
| `src/bin/desktop/comparison_field_view.rs` | `c979d62cdc768f42ca680ff809052048db3392edc0940336aa5f7fb9aa3f1360` |
| `src/bin/desktop/comparison_archive.rs` | `cb4e4656fa8c4a5f429321c563aeb26a5f14579d80cb429a8a8011d3040ac1e7` |
| `src/bin/rheon-desktop.rs` | `bf5d4b74e689404fd3e7a5486f91b77fd8fff87acbeda8a053e73d769414de5e` |
| `src/recording/voxel_material.rs` | `b0114efbcc8abc4d5ed3877c8210a853a2b101e0557fc79f0c28dc11fa600e49` |
| `src/recording.rs` | `fcf9213788e072bd422055e9aed76a0b24ba458c8a3951070d7c70eef954a995` |
| `tests/comparison_field_view_contract.rs` | `1e1a6e892b97d19db1cb3dec32b0d5e25837a1bf347ed024aaa78090424cc65b` |
| `tests/saved_source_contract.rs` | `87ca8779c708acaa9c80d1bfd51b8d044329da53296a235b6f51d683d77b9332` |
| `reports/m4-run-set-admission.md` | `37134ceaf7bc55e565de5fbea1e2b12af6daaca8d5050dc2ca6c013b202c962b` |

The complete required round is returned for implementer-owned serial corrections and targeted independent rereview. Only this report was written; no reviewer action was denied, pending or uncertain. Prior denied external operations and uncertain Library uploads were untouched. No shared-main or external publication is authorized by this source review.

## Targeted R1/R2 correction rereview

On 2026-10-22 the same separate reviewer inspected the frozen serial correction round and its added/adjusted owner fixtures. Only this report was appended. No test/build/native/numerical workload or external mutation was executed by the reviewer.

**Current independent source-review disposition: R1 and R2 are resolved. No further required source correction was found in this targeted scope.** The implementer was running six owner fixtures and final strict all-target checks at this review boundary; their outcome is not predeclared. Component admission remains implementer/parent-owned after affected execution. Whole M4/A5/A6 remain pending.

### R1 — resolved

The dyadic-candidate loop has been removed. Cold audit now requires actual accepted dt to be normal, positive and no greater than the frozen request, with exact bitwise accepted-clock addition and interval-start continuity. These conditions match the real owner's normal initial request and its Courant/stress ratio reductions followed by `next_down`. Its comment and updated admission explicitly state that cold data does not authenticate retry cause or attempt count. No Debug report is parsed and no numerical owner is created on cold open. The scope is a valid bounded stored schedule, not proof that a declared executor performed those calculations.

The old 0.075-vs-0.1 refusal case is replaced with 0.2-vs-0.1, a true over-requested interval. The reason for that oracle correction is preserved in this review: the initial halving policy was invented, not an intended scientific threshold relaxed after a failure. All metadata/binding/coverage and exact timing checks remain.

The added `actual_ratio_shortened_solver_interval_can_be_pinned_and_reopened` fixture starts from an owned 4³ binary Water/Air vertical interface, captures its real geometry/palette/IDs and declared gravity −9.81 m/s² with requested dt=1 s, and constructs from that FrozenInput. It records the canonical constructor, calls the actual solver with the frozen request, and requires a positive shortened dt and more than one, at most eight attempts. It records the accepted state, closes the archive, drops the numerical owner, pins and cold-opens the member, and compares the actual stored dt bits with the successful solver report. This exercises solver→record→audit compatibility rather than manually manufacturing a half timestep. It is a component interoperability fixture, not a new flow accuracy/cohort acceptance claim.

Completed coverage remains based on the declared number of accepted requested intervals, whose actual dt may be shortened. It does not imply the original requested-dt×step-count horizon was reached, nor add hidden substeps to reach it. Later native UI/worker integration must retain that distinction.

### R2 — resolved

Producer binding now compares each constructor fraction with the canonical F=1 bit pattern for Water and positive-zero bits for Air, and requires positive-zero bits for all initial MAC velocities. Cold constructor audit makes the same exact fraction comparison and checks every decoded constructor velocity vector for canonical zero bits. These stricter rules belong to the immutable native recipe binding; the generic numerical/recorded-value APIs retain mathematical zero semantics.

The new `signed_zero_cannot_change_exact_constructor_fraction_or_velocity_bits` fixture first requires an actual numerical constructor with gas F=−0.0 to fail `bound_run`. It then obtains a valid canonical header/publication, changes one encoded constructor F or velocity value to −0.0, and appends/finalizes through the ordinary SQLite owner. Generic integrity is therefore self-consistent. Pinning must refuse at the exact constructor fraction or velocity owner and leave the attempt unpinned. This is a target-bound bit-identity check, not a stale-digest negative. The previous all-Water-versus-Air constructor mismatch and positive canonical path remain intact.

### Scope and final identities

All header/geometry/coefficient/configuration/limit bindings, terminal coverage, hash-before/after checks, no-replace publication, explicit enum dispatch and comparison refusal paths remain unchanged. The corrected admission documents the bounded-dt and exact-bit semantics. No numerical formula, retry policy, tolerance, archive codec, native worker or retained-view source was changed by these repairs. No whole-feature/system acceptance is inferred from source review.

| Path | SHA-256 |
| --- | --- |
| `src/bin/desktop/run_sets/voxel_material/run_set.rs` | `3018decddb681f23f53c4fbc35fbfb1a8e6a28c66d894e2db35947a4e904a8f2` |
| `src/bin/desktop/voxel_material_run_set_tests.rs` | `939d49bcc4b5fac346d0f259bdc7ba1939b026b9134b2057bcdc699c29f6e2fb` |
| `src/bin/desktop/run_sets/voxel_material.rs` | `cd43376b66c4f4cce454e31506bb0af8d5e9c003e82e438f93433c575e5a42d1` |
| `src/bin/desktop/run_sets/source.rs` | `dfb0ae266e9e0b5180225eb015a334013f31442815fd4496ac22a8c49098cc57` |
| `src/bin/desktop/run_set_view.rs` | `1c1720537c3f9ad001ace526b93164071d89d46b29ad209f67b3a762f1e0c47b` |
| `src/bin/desktop/comparison.rs` | `4daffd199ecb6ef877bed00ea26cf3b85a2788c70b02603acada146ce1444d5b` |
| `src/bin/desktop/comparison_field_view.rs` | `c979d62cdc768f42ca680ff809052048db3392edc0940336aa5f7fb9aa3f1360` |
| `src/bin/desktop/comparison_archive.rs` | `cb4e4656fa8c4a5f429321c563aeb26a5f14579d80cb429a8a8011d3040ac1e7` |
| `src/bin/rheon-desktop.rs` | `bf5d4b74e689404fd3e7a5486f91b77fd8fff87acbeda8a053e73d769414de5e` |
| `src/recording/voxel_material.rs` | `b0114efbcc8abc4d5ed3877c8210a853a2b101e0557fc79f0c28dc11fa600e49` |
| `src/recording.rs` | `fcf9213788e072bd422055e9aed76a0b24ba458c8a3951070d7c70eef954a995` |
| `tests/comparison_field_view_contract.rs` | `1e1a6e892b97d19db1cb3dec32b0d5e25837a1bf347ed024aaa78090424cc65b` |
| `tests/saved_source_contract.rs` | `87ca8779c708acaa9c80d1bfd51b8d044329da53296a235b6f51d683d77b9332` |
| `reports/m4-run-set-admission.md` | `9d2f2a84e61a7396eebcebcb72df444232d597b5654ad08840ff6f6508a8eac2` |

Initial identities remain historical. No reviewer action was denied, pending or uncertain; prior denied external actions and uncertain Library writes were untouched.
