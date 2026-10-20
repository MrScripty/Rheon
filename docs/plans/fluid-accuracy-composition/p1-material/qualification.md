# Reference qualification and limits

Frozen numerical source/protocol:3b609ee443a1a704f6aeeb4b836fcf30f7f4d248.
Independent derivation/source review preceded tests and computation. Native CPU
Linux x86_64 Rust/cargo1.92.0 binary64 release, no-default-features. Actual binary
SHA2565abf51717b6d6fc3b893201272079d08c845d908441233b2b6a252026de86418.
Toolchain, full frozen source, execution/status logs and raw CSVs are retained.

| Claim / check | Result | Scope |
|---|---|---|
| A1 source/refusal contracts | 5/5 debug and5/5 release pass | Includes unchanged EOS checks |
| Strict affected Clippy | example and adapter pass, -D warnings | Private CPU research targets |
| Formatting/diff | Pass | Source hygiene |
| A2 one frozen native n8 gate | exit0;8 states,72nodes,64elements,48probes | Static material-FE reference only |
| Native necessary criteria | All8 states/16 probe groups pass | 8 smallest-step pointwise Hessian failures retained |
| Wrong controls | Both discriminate missing moment/load | Not admitted candidate methods |
| A3 independent m/Q integral and field review | Pass; source integrals, stored force/geometry fields and diagnostics checked | Must pass frozen percell thresholds before positive reference disposition |
| Earlier campaigns/production/G1 preservation | Source diff empty; both earlier archive hashes unchanged | No merges, API/dependency/proof/workflow changes |

The lumped kinetic energy is a declared approximation, distinct from the
consistent FE kinetic integral. A truncation-tail bound is not a native rounding
bound. Source integral comparisons use the precommitted percell tolerances;
finite differences use two consecutive scales, with every failure retained.
The n8 result establishes neither mesh refinement nor nonlinear stability.
No full CPU regression is repeated for isolated private example/test additions;
PR121's original582-test release result remains historical supporting evidence,
not a new full-suite claim. Hosted CI is separate evidence.

Existing `Discrete` weighted-incidence proofs are reused conceptually with their
stated assumptions. No Lean source, axiom, historical receipt, fresh kernel or
Rust-equivalence qualification is claimed. Native3D viscosity, obstacle geometry,
free-surface/transport, nonlinear dt and all physical/GPU/UI claims remain absent.
No original P-W production gate is discharged by this reference.

Reproduce with the committed protocol and fresh output path:

```
cargo test --locked --no-default-features --test material_partition_gate_contract
cargo test --locked --release --no-default-features --test material_partition_gate_contract
cargo clippy --locked --no-default-features --example material_partition_gate -- -D warnings
cargo clippy --locked --no-default-features --test material_partition_gate_contract -- -D warnings
cargo run --locked --release --no-default-features --example material_partition_gate -- FRESH_OUTPUT
python3 docs/plans/fluid-accuracy-composition/p1-material/analyze.py FRESH_OUTPUT
```

This operation intentionally ends with a concrete grid Q1→Q2 recommendation,
not further material/particle production promotion. Canonical plan and later
admission belong to the repository lead.

Evidence archive `rheon-p1-material-gate-3b609ee.zip` is retained in Library.
[Manifest](manifest.json) binds its successful receipt,6,976,553-byte size,
SHA25652d1f4a718f70eba807195561d016eafbadc5c8ad297b06cd1071607988fd174,
actual executable, full frozen source, raw failures and independent review/replay.
The archive CRC/inventory passed. Its custody receipt remains external to avoid
a self-hash cycle. Earlier archives were rehashed unchanged; original, PR121 and
PR123 worktrees stayed clean. No source/protocol/evaluator/route derivation changed
after numerical freeze; only reports and custody metadata were added.
