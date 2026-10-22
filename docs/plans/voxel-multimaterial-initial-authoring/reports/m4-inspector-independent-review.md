# Independent M4 retained voxel inspection review

Review date: 2026-10-22. Reviewer: separate agent `/root/voxel_implementation_review`, distinct from the implementer. Reviewed `inspector.rs`, the new retained slice/probe child and SQLite/egui fixture, admission, typed recording metadata/decoder and the existing Inspector clearing/raw-field/CSV owners. Only this new report was written. No test/build/native/numerical/benchmark workload or external mutation was performed.

**Initial disposition: three bounded corrections/evidence requirements below.** R1 affects a new adapter with no production caller; deletion is a proportionate fix. R2 concerns material names at standalone archive inspection. R3 is target-bound UI evidence for the new view. No additional required correction was found in this complete round. Full M4/A5/A6, native worker/run/retained workflow and performance remain pending.

## Complete required findings round

### R1 — Low: the new recording adapter does not preserve exact geometry binding

Location: `Inspector::record_voxel`, `inspector.rs:70–97`.

The new helper reconstructs a RecordingRun and compares derived `PartialEq`. Geometry contains f64 arrays, so opposite signed-zero origins compare equal even though their recorded bits/digests differ. The actual configuration string preserves its bits, but that does not close geometry equality. A different passed simulation with a +0.0 versus −0.0 origin can therefore evade the advertised configuration/header mismatch before append. This repeats the exact-identity issue already corrected in the native recipe owner.

Required correction: either remove this unused early producer adapter and have the future worker reuse the actual immutable owner/publication/StepRecorder boundary when it is implemented, or compare exact geometry bits in addition to the existing header comparison. The implementer's proposed deletion is proportionate: the current slice is read-only and should not retain an otherwise unused production adapter merely to make a fixture convenient. If retained, add a signed-zero geometry refusal requiring unchanged recorder prefix and an unpoisoned writer before a subsequent correct append. No legacy adapter or numerical owner change is needed. A test-only typed-selection accessor likewise need not become a new production API absent a caller.

### R2 — Low: raw typed records do not always justify Water/Air endpoint names

Location: `voxel_material_view::show`, especially unconditional heading/model/legend/totals/probe labels.

The immutable native input profile fixes qualified Water/Air coefficients, and a pinned voxel member checks that exact binding. Standalone Inspector reload, however, accepts the generic typed recording profile without a FrozenInput. Its numerical/recording APIs admit any checked MaterialPair for numerical fixtures and capture the actual coefficient values. The new view unconditionally names those two endpoints Water and Air. A valid nonreference pair can consequently be inspected under names that were neither captured nor proved by the record. Displaying its actual rho/mu afterward does not establish those material identities.

Required correction: keep the view honest at this boundary. Generic Liquid/occupied Gas endpoint names throughout this recorded view are sufficient and avoid inferring chemical identity from coefficient equality; the separate authoring UI retains its qualified Water/Air palette. Alternatively, an explicitly bound named-palette view could require that captured identity. Do not look up current presets, silently replace recorded values, add arbitrary new editable material infrastructure or make raw inspection depend on constructing a solver. Add a valid nonreference-pair archive/view case proving generic naming or explicit refusal while captured coefficients remain unchanged.

### R3 — Low: the new fixture does not check the view's new derivations or probe interaction

Location: `typed_voxel_selection_survives_later_evolution_and_failed_selection_clears_it`, `voxel_material_inspector_tests.rs`.

The fixture meaningfully checks real SQLite dispatch, old typed generation retention and missing-step invalidation. Its egui check only requires some shapes, which would also pass if the entire new slice/probe/totals body were omitted and the existing Inspector heading remained. The callback checks decoded F and positive gas density, not the newly displayed phase volumes/masses, occupied gas complement, numeric probe coordinates or probe-to-original-field mapping. Those are the deciding new UI/data claims of this component.

Required focused evidence: inspect actual egui output for known phase totals/units and `1−F`, and drive a named numeric cell probe or slice selection through the rendered control path. Require the correct physical center and matching x-fast original `liquid_fraction` scalar index/value. Include at least one mixed-cell value so a binary-ID approximation cannot satisfy the gas/display oracle. Verify material naming for R2 and retain the existing immutability/error checks. This is a small read-only fixture, not a new scientific cohort or native-window qualification. No broad screenshot or GUI framework is required.

## Confirmed selection, clearing and ownership

The typed selection is an owned Arc separate from the raw RecordedStep and reader. Exact solver identity selects RecordedVoxelMaterial decoding; f64 shape/range/wall/gauge/interval checks remain with that owner. A failed select clears the previous raw and typed values before reading/decoding the new record, so no latest-step substitution survives. Successful selection installs matching raw/typed objects together. Reload clears reader/path/selection/raw and every typed variant before I/O. Observed count/audit/metadata errors likewise clear the new voxel Arc alongside legacy variants. Resetting transient slice/probe controls is separate from retaining stale values; those controls do not own numerical fields.

The `with_voxel` callback clones its Arc and drops the temporary lock guard before invoking caller code. It does not expose a mutable simulation alias or hold the Inspector mutex across a reentrant callback. Source inspection and the current fixture support old selected generation remaining unchanged after later simulation evolution/append. They do not promise automatic detection of an externally modified archive without explicit selection/reload/audit; existing repaint reuse is expressly an immutable verified selection under protected-path assumptions.

The current voxel `verify` path invokes the generic RecordingReader integrity/continuity audit, while typed semantic decoding happens on selection. A pinned saved-member audit separately validates all typed publications against its recipe. These are different scopes; the generic Inspector “integrity audit passed” is not full numerical/recipe authentication. No pressure report Debug text is parsed to infer missing acceptance facts.

## Display semantics and bounds

Fractions are read from the immutable f64 record. Liquid volume is `Vcell * sum(F)`; occupied gas volume is `Vcell * sum(1−F)`; phase mass uses the corresponding captured density. Mixture color is an explicitly visual interpolation between endpoint colors; it does not edit/rebin F, change resolution, infer Vacuum from transparency or make a blended material ID authoritative. Gas remains occupied and positive-density. The color legend includes text, with a mixed-cell explanation. R2 is the remaining naming issue for unbound generic records, not a finding that the arithmetic swaps phases.

Totals currently use straightforward f64 display reductions, and nonfinite results show unavailable. They are approximate displayed initial/accepted accounting, not the numerical owner's compensated reduction or a conservation certificate. The finite-only guard does not by itself certify every underflow/subnormal edge for arbitrary fabricated records; the view must not promote this into a broader representability/physics claim. Qualified accepted publications and frozen native input have separate numerical checks. Raw original f64 sample/CSV remains the precision/provenance boundary.

Axis/layer choices remain within the typed grid's 4–64 counts, and probe coordinates are clamped before indexing. At most 64² rectangles are emitted for a slice. The canvas intentionally uses index-space sampling, labelled as a cell-index slice, rather than pretending its fixed aspect is a physical metric projection. The numeric probe reports the physical cell center from captured origin/spacing. Pointer mapping reverses the vertical index and uses x-fast Cartesian grid indexing. It changes transient probe/raw-field selection only. Numeric probing can choose a 3D cell separately from the displayed plane; the cell label identifies that selection and no highlighted surface point or solver edit is implied.

The three numeric controls use associated `Cell i/j/k` labels and bounded integer ranges. A changed probe/slice returns its original cell index, and Inspector selects `liquid_fraction` at that index in the existing raw-field owner. Raw field type, sample association, interval/evaluation time, dimensions and units remain shown; unavailable fields/indices have explicit absence rather than substituted values. No f32 preview or mesh extraction is used. R3 asks the actual controls/derived-label checks needed to substantiate this otherwise straightforward source mapping.

Decoded record plus raw field arrays remain simultaneously owned; the view Arc clone does not recopy them each frame. Totals scan the bounded cell array and drawing visits one bounded plane. This is bounded source behavior, not measured latency/RSS. Native worker/retained preview combined allocation admission, large history retention and actual compact/native rendering remain later gates. The slice does not change any numerical, worker, launch, force/Scene, saved-input or recording-lifecycle owner.

## Scope, executed-evidence limits and initial identities

The implementer reports three selected Inspector fixtures passed, including the one new voxel fixture, and strict lint identified the unused premature production helpers plus routine style issues. I inspected the actual new fixture rather than describing all three as new voxel UI qualification. Final source cleanup and targeted R1–R3 evidence remain implementer-owned; this reviewer ran no workload and predeclares no final result.

The fixture uses actual SQLite constructor/accepted publications and calls the real Inspector/egui body. It is not a native window, moving-flow accuracy test, cold saved-run rerun or durable append-fault qualification. No actual native both-material inspector interaction, clipboard/export operation or A5/A6 workflow is accepted here. The parent remains responsible for component admission and complete integration gates.

| Path | SHA-256 |
| --- | --- |
| `src/bin/desktop/inspector.rs` | `4d62a6feeac61f31515c5ce80001a88da8c7e1fa60a1ab24c6077eb323a868a2` |
| `src/bin/desktop/voxel_material_view.rs` | `073a055187db905d6f296f05aaafc969b53cd9a876a5f6c80f1c03b173fe8387` |
| `src/bin/desktop/voxel_material_inspector_tests.rs` | `6f22d9e21e62f99de8457f232270c7ba12233e1985ab1dbb6b6b7c751c12f429` |
| `reports/m4-inspector-admission.md` | `db184ba6846d13ef40aebb99f2157a3491044c449af04782cfd34ffcbe3ed50a` |

These identify the complete first findings round before serial corrections. Only this new report was written. No reviewer action was denied, pending or uncertain. Previously denied external actions and uncertain Library uploads were untouched; no external publication or main integration is authorized by this source review.

## Targeted R1–R3 correction rereview

On 2026-10-22 the same separate reviewer inspected the frozen serial correction round and added mixed-cell keyboard fixture. Only this existing report was appended. No test/build/native/numerical workload or external mutation was executed.

**Current independent source-review disposition: R1–R3 are resolved; no further required source correction was found in this targeted scope.** The implementer reports the two corrected voxel fixtures passed before final wording polish, with all four Inspector fixtures and strict checks running at this review boundary. Their final outcome is not predeclared here. Component admission remains implementer/parent-owned after affected execution; full M4/A5/A6 and actual native retained/worker workflows remain pending.

### R1 — resolved by deletion

The unused `record_voxel` adapter is removed entirely, so its PartialEq-based geometry binding cannot become a production recording contract. There is no replacement producer API or dead-code allowance. The fixture now appends through the real core publication function and existing StepRecorder, preserving the separation between read-only selection and the future worker's immutable input/recording owner. `with_voxel` is explicitly cfg(test), retaining its useful assertion access without inventing a production callback consumer. Production typed selection and the actual slice/raw-field consumers remain intact. Future worker integration must use the reviewed exact recipe/header boundaries rather than reintroducing the deleted comparison.

### R2 — resolved

The retained view now consistently uses Liquid and Occupied gas roles in its model explanation, legend, totals, captured-coefficient display and probe fractions. It does not infer chemical identity from coefficients, consult presets or change stored values. Colors remain a display-only endpoint blend. The separate initial authoring palette/names are unchanged. This works for both the qualified native pair and a generic valid numerical record without falsely naming the latter Water/Air.

The new fixture writes an actual typed SQLite record with captured liquid/gas densities 3 and 2 kg/m³, nonreference viscosities and one F=0.5 cell. It requires rendered Liquid/Occupied gas labels and absence of Water/Air naming in the actual panel output. The literal displayed masses additionally check use of those captured densities. No broad material editor or new recording metadata is introduced.

### R3 — resolved

`actual_named_keyboard_probe_and_rendered_mixed_phase_totals_use_captured_fields` now renders the actual Inspector over that selected SQLite record, enables AccessKit, resolves the associated Cell j/k numeric inputs and checks their SpinButton/TextInput roles. It reaches them by real Tab events, selects/replaces their text with keyboard events, commits with Enter and does not assign Slice control state directly. Starting from the default i=0, editing j=k=1 must select x-fast index 20.

The fixture checks the actual rendered physical center `[0.125,0.375,0.375] m`, Liquid F=0.5 and Occupied gas F=0.5. On a 4³ unit box, its independent literal expected volumes are 0.0078125 and 0.9921875 m³; expected masses are 0.0234375 and 1.984375 kg. It requires those formatted values and units, then checks that the existing raw-field owner selected `liquid_fraction` scalar index 20 and that its CSV row contains the original value 0.5. This directly exercises the new view derivations and numeric-control→raw-selection link that the earlier nonempty-shape assertion did not establish. It is a meaningful mixed-fraction/occupied-gas oracle rather than a binary material-count stand-in.

The original fixture still proves typed selection survives later evolution/append and clears on an unavailable selected step. Clamp/collapsible-if style corrections do not change view semantics or suppress lint. No native display, physical keyboard layout, compact-scale readability, clipboard delivery or end-to-end retained run acceptance is inferred from the egui fixture. Naive f64 totals remain approximate display reductions, not a solver conservation certificate or a guarantee over every fabricated extreme record; raw f64 values retain their original owner.

### Corrected identities

| Path | SHA-256 |
| --- | --- |
| `src/bin/desktop/inspector.rs` | `6a7a9ce81db62e610ef0c9478bd27726a724a478f4976a7b5152ac5b1cf2f231` |
| `src/bin/desktop/voxel_material_view.rs` | `1dcc44cb0318aa71366f7a8b8a9b019be4d0a6293d4264c453f5b93ca570d775` |
| `src/bin/desktop/voxel_material_inspector_tests.rs` | `5581864decaa3c6e22d31b7cb38577b18f4ae53e3820773887b26faf3d25939c` |
| `reports/m4-inspector-admission.md` | `763462ab07aa8b813d2f64eb3fa0a549f8c57a14245999c01d8b18bd4a2d333d` |

Initial findings/identities remain historical. Only this report was appended. No reviewer action was denied, pending or uncertain; prior denied external operations and uncertain Library uploads were untouched.
