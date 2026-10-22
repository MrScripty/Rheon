# M1 component verification — 2026-10-22

M1's bounded state/coefficient/projection contract passes its named component
gates on the isolated plan-base branch. This is not acceptance of A1–A6 for the
complete feature. Coupled transport/stress, interface refinement, native editing,
persistence and current-main integration remain pending.

Environment: native x86_64 Linux, repository-pinned Rust1.92.0
(`rustc 1.92.0 (ded5c06cf 2025-12-08)`), standard Cargo test runner, debug test
profile. Toolchain/cache provisioning is in the ledger. No dependencies,
toolchain declaration, CI policy or ignored tests were changed.

| Actual invocation | Observed result / claim |
| --- | --- |
| `cargo test --locked --no-default-features --test voxel_material_pressure_contract --test operator_contract --test pressure_contract` | Initial six new +15 legacy component tests passed |
| `cargo test --locked --test voxel_material_pressure_contract --test operator_contract --test pressure_contract` | Final seven new +15 legacy tests passed under default features |
| Same three contracts with `--no-default-features --features parallel` | Final seven new +15 legacy tests passed. No heterogeneous SGS numerical-parity claim |
| `cargo test --locked --no-default-features --features parallel --test voxel_material_pressure_contract --test operator_contract --test pressure_contract --lib operator::fixed_box_tests` | Two selected operator unit tests passed, including explicit refusal of uniform SGS weights. The filter selected **zero** integration tests; that invocation is not their pass evidence. The separate unfiltered invocation above supplies it |
| `cargo test --locked --no-default-features --test simulation_contract --test liquid_step_contract --test pressure_session_contract --test pressure_context_contract --test provider_step_contract --test force_field_solver_contract --test forces_contract` | 36 tests passed across seven suite invocations; seven pre-existing ignored tests remained ignored. These are affected legacy behavior contracts, not new force/provider support |
| Strict Clippy `--locked --all-targets -- -D warnings`, default and core-only+parallel; initial core-only all-target check | Passed. Default/parallel checks include the final seven-test fixture; the initial core-only check preceded its targeted addition |
| `cargo fmt --all --check`, exact diff whitespace, independent seven-file identity comparison | Passed; no unrelated source/manifest/README change |

The independent graph oracle assembles shared edges using explicit Cartesian
indices and `V/(h² rho_face)`. It checks the operator, symmetry/nullspace/positive
energy and manufactured pressure recovery at density ratios1/2/10/100/1000.
Half-column hydrostatic oracles check pressure differences as well as rest
velocity on X/Y/Z, including actual captured Water/Air. A separate targeted
heterogeneous analytic-gradient fixture executes both f32 correction APIs and
fixed walls; T1 was independently rereviewed and closed. No test tolerance or
assertion was adjusted in response to candidate outputs.

Invalid fields/materials, aggregate caps, boundary velocity, incompatible RHS,
iteration failure and cancellation retain typed failures. Projection's immutable
state borrow and success-only scratch view prevent scratch from becoming an
accepted timestep. The tests do not exercise a complete viscous/gravity step.

## Retained cancelled broader invocation

`cargo test --locked --no-default-features` was initially broader than needed.
It completed 40 suite summaries with330 passed and seven ignored tests, then
entered `material_contract`, which uses the separate particle solver and has no
`PressureOperator`/`PressureWorkspace` dependency. After that unrelated suite's
long workload, the owner stopped only its exact verified test-process PID.
Cargo returned101 with SIGTERM15. Seven material tests had individually reported
success, but that suite and the overall invocation are **cancelled, not passed**.
No failure/ignore/cancellation is relabelled as acceptance. The source-informed
affected-consumer invocation above closed the remaining M1 regression population.
No benchmark, ignored scientific cohort, native window or simulated graphical
qualification was run.

Raw local logs are retained under `.generated/voxel-material/m1/`, outside Git.
These compact identities bind those local files; they are not remotely saved
archives or current-main/PR59 results.

| Local log | SHA-256 |
| --- | --- |
| `default-contracts.log` | `0ffa2b6dd270387807fb2c7ddf7a0d566f7d7c004785e2929f832cd501611d4e` |
| `parallel-contracts.log` | `175edce6f3307e35312bede4fc32317d81ee0c95e3c31c258c2325430b5ef5d8` |
| `parallel-focused.log` | `f6db647907aedeace56ac93cc57ea09f6902d74eaa5331dc41bfd6cffabea882` |
| `affected-consumers.log` | `ff6eb4b5d53a1d61f78f52a8c9d0f82f7e3293efe58f543fddaeadd36661624b` |
| `default-clippy.log` | `6c8ff01a17f1123b75eccb286d2b2a44db5e32d313d65ebaa0dd076f0f7a790d` |
| `parallel-clippy.log` | `946034cd25d899186893f4c9e4b9a98a7249a52fb8d0c3ee1c9d80a9816fbba8` |
| `core-tests.log` — cancelled | `585db7416328972ba209189767090263c6a44ad1ac52e5b1598fc1762a9038a3` |

The [separate source review](m1-independent-review.md) owns reviewed source
identities and dispositions. All seven recorded hashes were checked against the
final candidate. Remaining integration blocker: denied PR59 coordination comment,
pending supported lead coordination; no main movement, push, PR or merge occurred.
