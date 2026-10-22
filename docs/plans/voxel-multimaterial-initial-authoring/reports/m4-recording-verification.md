# M4 typed voxel recording verification

The independently reviewed typed recording component is Accepted. It adds exact f64 cell fraction and staggered velocity, plus success-only interval pressure, to the existing SQLite owner. Full M4, SavedRunSet/worker/retained-view integration and A5/A6 remain pending. No native launch was enabled.

The fixed128-byte RVM1 header retains actual density/viscosity, gravity/pressure/numerical limits and initial-input digest. Filled gas is `1-F`; no binary accepted material IDs, pressure at construction, checkpoint, thermal or particle state is invented. Debug report controls are opaque provenance; future consumers may display them but must not parse them as typed quantitative diagnostics or replay instructions. Field association/stage/units/finite values and supported geometry/configuration are checked. Raw archive syntax inspection alone is not scientific validity or proof of agreement with an external recipe; the next archive owner must bind the exact validated FrozenInput before Ready.

[Independent review and rereviews](m4-recording-independent-review.md) resolved the single complete-round budget finding. The exported conservative preflight now includes bounded control text once, and the producer retains actual Vec/String capacity and validated-payload checks. A self-identified timestamp precision extension captures the original interval-start clock in the accepted success report. Separate source review confirms success/cancel/retry semantics; the numerical formulas/order/limits were not changed.

Executed with pinned Rust1.92.0 and locked dependencies:

| Check | Result | Local ignored log |
| --- | --- | --- |
| `cargo test --locked --test voxel_material_recording_contract` final metadata candidate | 5 passed | `.generated/voxel-material/m4-recording-timestamp-tests.log` |
| Same focused test with `--features parallel` | 5 passed | `.generated/voxel-material/m4-recording-timestamp-parallel-tests.log` |
| `cargo test --locked --test voxel_material_step_contract` after report extension | 5 passed | `.generated/voxel-material/m4-recording-step-report-regression.log` |
| `cargo test --locked --test recording_contract` unchanged SQLite owner regression | 15 passed | `.generated/voxel-material/m4-recording-legacy-regression.log` |
| Strict desktop all-target Clippy after final extension | Passed | `.generated/voxel-material/m4-recording-timestamp-clippy.log` |
| Strict default all-target Clippy before metadata-only extension | Passed; its timing is explicit | `.generated/voxel-material/m4-recording-default-clippy.log` |
| Formatting/whitespace | Passed | `cargo fmt --all`, `git diff --check` |

The five focused fixtures exercise actual SQLite constructor plus3 accepted4³ gravity intervals, exact decoded solver fields and old retained copy independence; malformed method/configuration/schema/inventory/fraction/wall/time/gauge refusal; bounded payload and cancellation before commit preserving previous accepted publication; exact exported-budget admission for constructor/accepted and budget-minus-one refusal; and3 rest intervals at dt0.1 requiring the inverse-subtraction rounding counterexample while preserving original start-clock bits. These are serialization/staging contracts. They do not constitute native worker evidence, new calibrated physical validation, a benchmark campaign or end-to-end A6. Initial harness API mistakes (`select`, terminal-generation argument) were corrected before successful execution; no physics assertions/tolerances were loosened.

Final source hashes match the independent report. Generated logs/databases remain outside tracked deliverables. The native evidence wording also clarifies that root made no manual DBus/package retry although Xorg itself repeated automatic DBus discovery. Previously denied operations, uncertain Library upload, main and external publication remain untouched. Next is the explicit immutable voxel run-set/cold archive boundary and exhaustive unsupported comparison handling, followed by real worker/UI integration.
