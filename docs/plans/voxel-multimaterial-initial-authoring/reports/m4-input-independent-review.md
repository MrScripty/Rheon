# Independent M4 saved-input component review

Review date: 2026-10-22. Reviewer: separate agent `/root/voxel_implementation_review`, distinct from the implementer. Reviewed the checked `InitialContents::restore` addition, `run_sets/voxel_material.rs`, its child-module persistence fixtures, module export, admitted subset, actual numerical constructor/geometry/pressure boundaries, and the existing manifest read/publish owner. Only this report was written. No source, tests, admission, plan or ledger was changed; no test/build/native/simulation/benchmark workload or external mutation was performed.

**Initial disposition: three required corrections below; this input component is not yet accepted.** The implementer independently identified R1 before this round and has described its intended nonallocating correction, but the reviewed candidate still contains the gap. R2/R3 are additional findings. Full M4/A5/A6, SavedRunSet dispatch, worker/recording integration and native execution remain outside this component review regardless of its eventual disposition. M3's separate native evidence does not qualify persistence.

## Complete required findings round

### R1 — Medium: a validated frozen input can have deterministically invalid numerical constructor geometry

Locations: `ExecutionPolicy::validate` and `FrozenInput::validate`, `run_sets/voxel_material.rs:260–296,342–350` in this candidate.

Freeze validation currently checks the generic GridGeometry, exact supported palette/IDs, absence of Vacuum, policy scalars and a nominal numerical payload minimum. It does not check numerical representability of phase cell mass or phase totals. `GridGeometry` intentionally supports a broader physical range than the two-material arithmetic owner. For example a filled Water 4³ grid with spacing `[1e102;3]` and zero origin has finite positive cell volume and resolved coordinates, so it can be restored, frozen, captured, published and reopened successfully. Its Water cell mass overflows binary64. `VoxelMaterialState::new` rejects that product, so `FrozenInput::construct` then fails despite the earlier “validated constructor-zero recipe” claim. Small valid GridGeometry volumes can similarly produce subnormal phase mass, and finite per-cell quantities do not ensure finite domain totals.

Required correction: before admitting the frozen recipe, check the deterministic numerical geometry/material constraints used by the actual constructor and declared method. A bounded nonallocating preflight over the restored grid and captured coefficients is appropriate; do not instantiate a full numerical owner merely to inspect a manifest. It must cover normal finite per-cell/dual-relevant mass, phase-volume/mass totals and the declared pressure-coefficient representability across axes and permitted binary face density combinations, using the actual operation ordering/normal-versus-zero policy where relevant. Keep this tied to the real method rather than inventing a physical length band. The numerical constructor remains authoritative for actual allocations/capacities and state admission; a static preflight cannot guarantee future memory allocation or every timestep.

Add deterministic large/small/anisotropic geometry refusals on capture and self-consistently hashed cold input, plus an ordinary non-dyadic valid-grid case. Require the exact intended validation stage/category. Do not weaken the frozen palette, coerce the geometry, substitute current presets or move a constructor failure after a run is accepted. Root's proposed checks are a sensible direction but remain subject to targeted source/fixture rereview after the full correction round.

### R2 — Medium: captured palette colors are accepted but discarded on restore

Location: `InitialDocument::materials/restore`, `run_sets/voxel_material.rs:97–145`.

`color_srgb` is part of the strict typed document and its digest, but restore checks only palette version, ordered IDs/roles and coefficients. A file with a changed color and a recomputed valid digest is therefore accepted. The restored core editor has no color field and the UI uses fixed colors; recapturing it writes the fixed defaults rather than the accepted captured color. Thus a supposedly exact captured palette can be silently reinterpreted even though integrity succeeds. A digest detects accidental corruption, not unsupported self-consistent metadata.

Required correction for this fixed v1 scope: reject noncanonical captured Water/Air colors along with unsupported IDs/roles/palette version. Preserving arbitrary colors through a new core representation would enlarge the feature and is unnecessary. Add a self-consistently hashed color-tamper refusal; retain a positive capture→open→restore→recapture identity check for the supported palette. This applies to the shared document in draft and frozen input; do not fix only the UI path or only reject a stale digest.

### R3 — Low: all freeze failures are reported as Vacuum

Location: `FrozenInput::validate`, `run_sets/voxel_material.rs:346–349`.

The blanket `e.freeze().map_err(|_| Unsupported("Vacuum..."))` maps every core failure to a physical occupancy refusal. Freezing also allocates owned fraction/ID vectors and can return `ContentsError::Allocation` or an actual-capacity failure. A fully occupied input can consequently be rejected with “fill with Air or Water,” which is false and not actionable. This also performs extra dense copies just to validate occupancy before the real constructor path.

Required correction: produce the Vacuum category only for an actual Vacuum validation failure. Use the existing document occupancy validator without allocating frozen vectors when merely validating the recipe, or preserve distinct failures through an explicit match/helper. Keep allocation/resource refusal separate from unsupported physical state; no new general error framework or forced process-OOM experiment is needed. Retain the positive exact Vacuum refusal fixture and add a focused category check if a fallible-freeze mapping helper remains. The later `construct` can still allocate its owned input and return a constructor/resource failure without claiming Vacuum.

## Confirmed contracts and architecture

**Captured coefficients are used on restore and construction.** The core addition accepts supplied MaterialPair values only when their four rho/mu bit patterns equal the qualified version-one Water/Air profile. It restores identity/revision/geometry and known IDs into owned copied cells. It does not call `MaterialPair::water_air` on saved input. New authoring still captures current reference values, then passes the same profile check; a future preset change fails rather than silently rewriting old input. The JSON dependency already enables `float_roundtrip`; canonical serialization/digest retains finite numeric values, including signed zero framing, rather than relying on a display label as physics. The known color omission is R2.

**The binary/occupied model stays explicit.** Saved Draft may contain Vacuum, preserves its original revision/SceneObjectId and restores with empty session Undo/Redo. FrozenInput refuses Vacuum; Water becomes exact F=1 and Air exact F=0 through the core freeze, retaining positive captured gas density. Stored method, walls and initial velocity are closed enums: matched-donor/strain/Jacobi v1, closed free-slip and zero velocity. There is no particle/carrier fallback, mixed authoring reinterpretation, arbitrary material palette, thermal state, surface tension, external asset or mid-run restart promise. The constructor uses owned frozen data and the captured gravity/pressure/numerical limit, not later edits to the Draft.

**Decode output is preflighted before dense allocation.** A regular manifest read is capped at 1 MiB, including a second actual read-length check. Grid axis counts are checked at 4–64 before decoded-field allocation. Base64 encoded length must match the count-derived canonical size; output reserves `n+2`, then checks actual decoded length and every ID before constructing a u16 field. With n≤64³, the arithmetic is bounded. Invalid alphabet/padding/length/unknown IDs refuse. The parser still necessarily allocates the bounded encoded JSON/string before this decoded-array check; the claim is not zero allocation before validation. Temporary decoded bytes, u16 conversion and copied document remain bounded separate owners.

**Typed envelopes and integrity do not use parser fallback.** Draft v1 and input v3 have distinct kind/model enums and `deny_unknown_fields` on their nested structures. The existing `read_versioned` reads a small header and then decodes the original bytes, so duplicate required fields are not collapsed through a generic JSON map. Unknown version/kind/method, malformed numeric data, unknown keys and duplicate fields fail rather than trying another owner. The fixed two-entry palette must have the exact ordered unique IDs/roles. Digest checks compare canonical typed payload bytes before adoption. There is no broad compatibility exception for a damaged new profile. This module is not registered in the actual SavedRunSet source dispatcher yet, appropriately for this component-only admission.

**Immutability does not alias the editor or caller policy.** Both exposed saved types have private owned payload/digest fields. ExecutionPolicy is moved into capture and returned only by shared reference; the document is encoded into an owned string. Later caller edits cannot mutate that payload. `initial_editor` creates a fresh owner/history, and `construct` starts generation/time zero. Save/capture refuses a live candidate, so a last preview prefix cannot become an accepted input. Draft publish does not take a mutable editor or invalidate redo. R1 concerns whether the owned content is numerically admissible, not mutable aliasing.

**Publication reuses the existing failure/uncertainty owner.** The module passes bounded serialized envelopes directly through `publish_new`: private create-new staging, write/sync, no-replace hard-link publication, directory sync and cleanup. It propagates `PublicationUncertain` after publication may be visible; it neither deletes a possible target nor blindly retries or substitutes an overwrite route. Failed reads/publishes do not adopt or mutate Draft/history. The existing filesystem/path concurrency precondition remains applicable; this review does not claim a new hostile-filesystem defense or independently executed durability fault injection. Reopening a possible publication and checking its digest is the later UI owner's supported reconciliation step.

## Resource and policy limits

The policy's nominal numerical minimum `8*(14*Ncells+9*Nfaces)` matches the M2 owner's admitted array composition, and counts bound its usize arithmetic. The 192 MiB ceiling is an explicit method policy, not proof of allocation. Actual Vec capacities and aggregate numerical admission remain checked by the constructor. File/input parsing, editor/history, frozen copies, numerical arrays, recorder, retained frames and process/graphics overhead are separate owners; this component does not yet preflight their simultaneously live native composition. Repeated restore/freeze passes incur additional bounded allocations; R3 can remove the unnecessary freeze during validation without introducing a cache or new authority.

Requested dt/step count, pressure settings, finite gravity, iteration cap and declared provenance are captured and checked. The requested horizon is positive/normal; it is not a promise that adaptive accepted dt equals that horizon or that every configured future step succeeds. Recording limits get the existing generic limits check and enough record slots for constructor plus requested steps. The implementer explicitly confirms this is **not** record-publication capacity admission: a syntactically valid very small record byte limit can still be insufficient for a future voxel record, and database/field payload requirements are not defined by this component. Full M4 launch must bind that real record shape and combined allocation/work/retention admission before native exposure; do not describe `FrozenInput::validate` as complete run readiness. This is a deferred integration obligation, not a request to invent a recorder format in this input slice.

Similarly, collision meshes, authored forces, emitters and other unsupported selected features are not inputs here. The native composition must explicitly refuse them rather than silently omitting them when it later captures a recipe. Loaded Scene identity collision/adoption and the existing worker/retained-result state machine remain later owners. No current native readiness control was loosened by this slice.

## Fixture assessment and execution limits

The inspected fixtures meaningfully check incomplete Draft roundtrip/revision/identity, redo preservation during save, new history on load, typed Vacuum refusal, non-overwrite bytes, owned frozen contents after a later removal, constructor-zero fraction count, resource/retention policy refusal and active-stroke refusal. Malformed draft fixtures recompute hashes for bad payload length/unknown IDs, unsupported captured coefficient and excessive counts; separate cases target duplicate/unknown top-level keys, version, stale digest and file-byte cap. R1/R2 need their targeted negative and positive extensions.

The cold fixture launches the same test executable with the exact ignored child entrypoint in a new process, passes only the owned saved path, opens it, reconstructs generation zero and executes a declared single 4³ rest step. It checks the original identity/revision/phase population and reports a child marker; the parent also reopens/equality-checks the frozen recipe after editing the original Draft. This is a real subprocess constructor fixture as written, not an actual native worker, saved-run dispatcher, recording or nontrivial moving-flow equivalence test. Its current expectation compares coefficients with the current reference pair; source inspection, the exact profile admission and stored typed bytes are the evidence that no lookup happens in the restoration path. Do not turn this one rest step into Water/Air scientific validation or whole-A6 cold replay acceptance.

The implementer reports the first component execution passed four fixtures and deliberately ignored the child entrypoint in the parent invocation; the cold parent explicitly invokes that child with `--ignored --exact`. I did not execute or independently repeat those results. No tolerance/assertion change is endorsed to address the required findings.

Further bounded test additions can include a representative malformed frozen-input method/policy envelope and exact non-dyadic/signed-zero captured values. Shared strict decoding already supplies the reviewed rejection structure; no exhaustive generic serde/framework test suite is required. The admission text currently mentions root test registration, whereas actual registration is the new module's child `#[path]`; the implementer should keep the exact write-set description aligned without adding an unnecessary root registration.

## Initial reviewed identities

These identify the complete first round before serial corrections. Only this report is reviewer-owned output.

| Path | SHA-256 |
| --- | --- |
| `src/initial_contents.rs` | `e9bf1cd1d2bfe5396c6a5fb161d1f1dfea883a00ccfd8db8c6b0547ace28948a` |
| `src/bin/desktop/run_sets/voxel_material.rs` | `9e61a966116df6440e3ad026dfd96ddeb3113eb89fd344334a502b21cffb28a4` |
| `src/bin/desktop/voxel_material_persistence_tests.rs` | `5a31237e9ab73bf93b655853932b1cce125069764ad17b6975cd38870e872288` |
| `src/bin/desktop/run_sets.rs` | `3f2b51a87a82e194b95a202d3a9e21dd6dbdcc365ca648ff6bc07117dcaffd35` |
| `reports/m4-input-admission.md` | `1317e0771dac4f55128149e55ece47f55d0855e15dd118a78fc132d7963cbc34` |

Required findings are returned for implementer-owned serial correction, focused execution and targeted independent rereview. No reviewer action was denied, pending or uncertain. Prior package/comment denials and uncertain Library writes were untouched; no production exposure or external publication is authorized by this component review.

## First targeted correction rereview

On 2026-10-22 the same separate reviewer inspected the frozen serial correction round and additional fixtures. Only this report is appended. No tests, builds, native app, numerical cohort or external action were executed. The implementer reports five focused fixtures passed with one deliberately ignored child entrypoint; its parent fixture explicitly launches the child in a fresh process. This is implementer execution evidence, not reviewer repetition or full M4 qualification.

**Current disposition: R2 and R3 are resolved. R1 is substantially corrected but retains the arithmetic-order correction below.** No additional required issue was found in this targeted round. Initial findings and identities above remain historical.

### R1 — partially resolved; match the pressure row's actual floating-point expression

The new nonallocating `numerical_geometry` correctly runs after strict document/occupancy restoration. It checks positive normal cell, half-cell and domain volumes; endpoint cell/half-cell/domain masses; actual nonempty phase volume/mass totals; the three binary face densities; axis areas/accelerations/weights; and a positive six-face row bound. It does not construct numerical fields or promise future allocation/timestep success. The four deliberately extreme but generically valid grids now target large cell mass, small volume/mass, domain-total overflow and anisotropic pressure geometry. Both capture and a self-consistently hashed cold envelope are required to refuse at the numerical representability owner. The ordinary non-dyadic spacing/origin case preserves signed-zero origin bits through publication/open and constructs generation zero. These are useful, focused corrections to the originally accepted-but-unconstructible recipe.

One expression still differs from the actual operator. At `run_sets/voxel_material.rs:187–202`, the preflight computes

\[
a_d=\operatorname{fl}(1/(\rho h_d)),\qquad
w_d=\operatorname{fl}(A_d a_d),
\]

and uses those product-form weights to bound the interior diagonal, with a comment saying this is the same order as the pressure owner. It matches `PressureOperator::with_face_densities`' coefficient-admission product, but the actual `PressureOperator::neighbor_weight` (`src/operator.rs:234–238`) computes

\[
\widehat w_d=\operatorname{fl}(A_d/\operatorname{fl}(\rho h_d))
\]

directly. The two rounded values need not be identical. The maximum of product-form values has not therefore been established as an upper bound on the actual direct-division weights. Near normal/finite thresholds, using their sum as the representability guard does not justify the claimed bound on the actual row. This is a source/arithmetic finding; I did not run a threshold search or claim a reproduced failing grid.

Required small remaining correction: retain the reciprocal/product checks required by `with_face_densities`, separately validate the actual direct-division weight, and form the maximum/diagonal guard from that actual expression (or document and implement a justified outward bound). Preserve the operator's X-low/X-high, Y, Z accumulation order. With the actual finite positive binary weights and matching positive-sum order, the max-per-axis row bound can then bound any initial binary row by monotonicity. Do not change the production pressure operator or numerical tolerance to make the preflight agree. This is a narrow completion of R1, not a request for a new solver or a universal future mixed-state/timestep proof. A targeted fixture can make the two expression paths observably distinct or directly check the intended preflight admission; do not relax the four existing negative cases or valid non-dyadic case.

### R2 — resolved

`InitialDocument::materials` now requires the canonical Water and Air colors in the same shared owner that checks ordered IDs, roles and palette version. The self-consistently hashed color mutation reaches `Unsupported("initial voxel palette")`, rather than passing restore or merely failing digest comparison. The positive Draft roundtrip now recaptures the restored editor and requires equality with the saved DraftRevision. Thus this fixed v1 metadata can no longer be accepted and silently replaced on a subsequent save. No arbitrary-color core/API expansion was introduced.

### R3 — resolved

FrozenInput validation now invokes `InitialContents::validate` on the strictly restored document instead of allocating a FrozenContents fraction/ID pair. That current validator only counts/locates Vacuum and has no fallible allocation path. Restore allocation errors propagate before this occupancy mapping, so a resource failure cannot become the “fill with Air or Water” category. Actual construction still owns its fallible frozen copies and numerical buffers. The exact Vacuum refusal fixture remains in place. If the document validator gains other failure kinds later, the mapping must evolve with that contract; the current correction is sufficient without a new injected-OOM facility.

### Other targeted observations and limits

The cold-child coefficient oracle now compares literal qualified four-value bit patterns, rather than looking up the current preset in its expectation. The ordinary frozen roundtrip keeps the original digest/payload and explicitly checks negative-zero origin bits. The admission now correctly describes the persistence fixtures as a child of the new module, with no unnecessary root registration. These address the initial review's associated traceability/test observations.

Publication/digest/decoded-count ownership is unchanged. No recorder/profile dispatcher/native worker is introduced in this correction. Actual record byte/database admission, combined simultaneously live native allocations, unsupported-feature routing and whole-M4/A5/A6 remain later gates. Passing these five focused fixtures cannot replace them. There is no source-review or scientific-validation claim beyond this bounded component boundary.

### First corrected identities

These identify the first targeted correction candidate, before the remaining pressure-expression alignment. The core restore and module export are unchanged from the first round.

| Path | SHA-256 |
| --- | --- |
| `src/initial_contents.rs` | `e9bf1cd1d2bfe5396c6a5fb161d1f1dfea883a00ccfd8db8c6b0547ace28948a` |
| `src/bin/desktop/run_sets/voxel_material.rs` | `d8e7de638cb483a6278072cef7a0a77eeb1f24509d87454c317ecc2538f54973` |
| `src/bin/desktop/voxel_material_persistence_tests.rs` | `3a5ca6edc0f073525e0d74b46b7429ed7d2ec63ecfa6a5a0e33876d6aec5f288` |
| `src/bin/desktop/run_sets.rs` | `3f2b51a87a82e194b95a202d3a9e21dd6dbdcc365ca648ff6bc07117dcaffd35` |
| `reports/m4-input-admission.md` | `d57f96f1ff891397f13cccf04f5ed92e14ee0b2ebdd47367d6ffba43b00e4bd6` |

The complete remaining finding is returned before implementer-owned serial correction and targeted rereview. No reviewer action was denied, pending or uncertain.

## Final targeted source disposition

After returning the complete arithmetic-order finding, the implementer serially corrected only that pressure preflight path and requested a further targeted review. I inspected the resulting source without executing any workload.

**R1 — resolved.** `numerical_geometry` now names and checks both rounded expressions: `constructor_weight = area * acceleration` for the operator-constructor admission, and `weight = area / (rho * h)` for its actual neighbor weights. `largest_weight` uses the latter. The diagonal bound adds that maximum twice per axis in the operator's X/Y/Z low/high order. For the initial binary states, every actual face density is one of the enumerated endpoints/midpoint, each actual face weight is bounded by the corresponding maximum, and matching-order addition of finite positive terms is monotone. Missing wall neighbors can be treated as zero contributions. Thus the checked finite six-face sum now bounds the actual initial row under the intended binary profile; the earlier mismatch between rounded formulas is removed. The production operator, physical model and numerical tolerances are unchanged.

**Current independent source-review disposition: all required input-component findings R1–R3 are resolved; no further required correction was found in this targeted scope.** The previously reported five passing fixtures precede this final expression alignment. The implementer owns affected final checks and evidence before component admission; this review does not predeclare a rerun result. No additional threshold-search workload or generic arithmetic mirror test is required by this source disposition. The original four extreme geometry checks and non-dyadic positive case remain unchanged.

This source disposition is limited to the independent saved-input component. Actual no-replace publication fault stages, record-capacity admission, SavedRunSet/recording/worker integration, immutable native accepted intent and A5/A6 remain the stated later gates. It does not accept full M4 or authorize external publication, main integration or a new native runnable method. Only this review report was appended; no reviewer action was denied, pending or uncertain.

### Final reviewed identities

| Path | SHA-256 |
| --- | --- |
| `src/initial_contents.rs` | `e9bf1cd1d2bfe5396c6a5fb161d1f1dfea883a00ccfd8db8c6b0547ace28948a` |
| `src/bin/desktop/run_sets/voxel_material.rs` | `64d155e81a74b386e3078d079c88458ef78013490518a15fa5f5501c4d4b8822` |
| `src/bin/desktop/voxel_material_persistence_tests.rs` | `3a5ca6edc0f073525e0d74b46b7429ed7d2ec63ecfa6a5a0e33876d6aec5f288` |
| `src/bin/desktop/run_sets.rs` | `3f2b51a87a82e194b95a202d3a9e21dd6dbdcc365ca648ff6bc07117dcaffd35` |
| `reports/m4-input-admission.md` | `d57f96f1ff891397f13cccf04f5ed92e14ee0b2ebdd47367d6ffba43b00e4bd6` |
