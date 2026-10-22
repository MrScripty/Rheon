# M2 executed component evidence

Candidate follows M1 commit `638008d4f06e885b681428a9d67bd5c4c3e0b01e` on the isolated implementation branch. The whole native feature and A1–A6 remain pending. No benchmark, native user workflow or saved-run cold replay was executed.

## Outcomes

- Core private kernel invocation: `cargo test --locked --no-default-features --lib voxel_material` passed seven selected tests, including the independently referenced translated-box refinement and the new cancellation-sensitive strain fixture; 31 unrelated library tests were filtered deliberately.
- Public coupled invocation: `cargo test --locked --no-default-features --test voxel_material_step_contract` passed all five tests unfiltered. The Water/Air fixture actually took 20 requested steps with gravity, phase transport, unequal viscosity and pressure, requiring motion and lower liquid center of mass. This is a bounded numerical fixture, not calibrated physical validation.
- Default invocation: `cargo test --locked --test voxel_material_pressure_contract --test voxel_material_step_contract --test operator_contract --test pressure_contract` passed 27 tests (7 new pressure, 5 coupled, 5 old operator, 10 old pressure), none ignored/filtered.
- Core+parallel invoked the same seven private kernels, then the same four unfiltered contracts separately: seven +27 passed, no failures/ignores. Parallel is a build/path regression; the new transport/stress owner itself is serial.
- `cargo clippy --locked --all-targets -- -D warnings`, `cargo clippy --locked --no-default-features --features parallel --all-targets -- -D warnings`, and `cargo fmt --all -- --check` passed.

Pinned toolchain: Rust1.92.0 from unchanged `rust-toolchain.toml`. All Cargo invocations use the unchanged lockfile and isolated CARGO_HOME/RUSTUP_HOME under `/workspace`. No dependency or feature declaration changed.

## Diagnosis retained

Initial formatting found a test cast/comparison parse error, corrected before compile or numerical execution. Initial core check passed. A combined `--lib voxel_material --test voxel_material_step_contract` invocation passed the original six private kernels but selected zero public tests because of the shared filter; its public result was never counted. A subsequent separate public invocation passed five. Strict Clippy initially found needless range loops and test-module placement, corrected without numerical semantic or assertion changes.

Independent review then found R1, an unsupported signed final-work roundoff scale, and R2, a generic error assertion where typed cancellation was required. The corrected expression scale includes the absolute rank-one strain/force contributions and final absolute dot terms; the unchanged fixed operation envelope was not tuned to output. Direct energy arithmetic and dt-scaled work error remain distinct. The new predeclared two-entry near-rigid fixture demonstrates that its work cancellation exceeds the old budget and fits the corrected bound, then traverses the assembled anisotropic stress owner. The original six kernel/five public assertions were not weakened. R2 now requires reaching each target callback and receiving `Cancelled` after nonzero accepted history, before checking unchanged fields/time/report and exact retry reconstruction. The complete review and targeted disposition are in [m2-independent-review.md](m2-independent-review.md).

M1's unrelated broad particle-material command was already cancelled and diagnosed in its own report; it was not restarted. No legacy workload is currently running for M2. The bounded default/parallel pressure/operator regressions cover the unchanged shared M1 pressure path; all M2 changes are contained in its new module.

## Local raw evidence

Logs remain outside tracked outputs in `.generated/voxel-material/`:

| File | SHA-256 |
| --- | --- |
| `m2-corrected-kernels.log` | `666439e27c2203a9c046b5efb2eee05aad648a34a3f48c1f3aa2211fbeb9f8ce` |
| `m2-corrected-coupled.log` | `08fce79d937769808b4b30c8201caa490357478d65bef159f4f6ac066d1e5214` |
| `m2-default-contracts.log` | `f3ef8715987f038897b21c575980d5a9732d47e86ada7305110285becca2f401` |
| `m2-parallel-kernels.log` | `b8a36fad5adfb49cb62fcda5a7da1e69334a600434de5a69f665f0df0fedbdb8` |
| `m2-parallel-contracts.log` | `438e1b651652be06a58269173d19753d3ccdad46fef876c2e2a24f55d52f9538` |
| `m2-default-clippy.log` | `8bb2da507e43da2e87491f1d672af601728f66ecd539aefc802854371c50dd38` |
| `m2-parallel-clippy.log` | `604a2c2a8d9496bc8b759cde6c13d56ab7cb9e24143c41001bdb28828baf7498` |

## Limits and next boundary

No native method, authoring UI, new archive profile or retained-material view exists in this milestone. Numerical fresh reconstruction is not fresh-process saved-intent replay. The M2 memory figure accounts for managed Vec payload/metadata as documented, not RSS. First-order interface diffusion and explicit viscosity impose the declared limits. No arbitrary material pair, boundary, force or continuous parameter envelope is qualified by these fixtures.

Main remains frozen under the sole integration lead; parent reports PR59's fixed cohort failed and remains unmerged. This feature does not move that baseline or inherit its evidence. The PR59 coordination comment remains denied and unretried. The earlier Library write remains uncertain and untouched. Local component verification is not shared-source integration or publication approval.
