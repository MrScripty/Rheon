# Prescribed box-flux milestone

This separate branch descends from frozen tracer head
7a5184df20b3b438d8992c6f2f80372fe5344ac3 and shared PR12 repair
d5938e4a944534629881ffd223fbb3c52f8c69c5 through merge
5f8fd77396c71aae2d9dc5cebd5755e84e4d7c12. Ordered parents and exact source/tree
identities are recorded in receipt.json. Reviewed milestone branches remain
unchanged; coordination owns PR updates, independent review and main integration.

## Executed acceptance

Actual all-target tests pass 107 default, 102 core-only and 113 desktop. Strict
Clippy passes all three configurations; formatting passes. Eleven new fixture
methods cover signed through-flow in each axis/both methods, unequal side areas
against an independently assembled dense gauged system, affine adjoint and gauge
invariance, boundary work, zero-flux legacy pressure/output bits, independent net
flux rejection despite huge provisional values, actual stored divergence,
cancellation/partial scratch/retry, pressure iteration failure, singleton and
capacity bounds, invalid and extreme arithmetic. See the
[book addendum](../../docs/research-book/implementation/prescribed-box-flux.md).

A real red fixture exposed a nonzero prescribed speed whose area product
underflowed to zero, incorrectly accepting incompatible flux. The actual red
exit was 101, with net_flux=0 despite actual_divergence≈1e-130. The new guard
rejects unrepresentable nonzero side-flux products before solving or writing
output. Recreate the exact pre-guard module by removing the comment and if block
between `let terms` and `let net` in src/box_flux.rs; its SHA256 matches the red
receipt. Run the receipt's targeted Cargo command. Later dense-oracle iterator
cleanup changes fixture-source bytes only; the actual red hash is preserved.
The focused green log runs all eleven methods with the guard. Earlier example
private-Axis compilation and dense-oracle Clippy failures are retained; the
example uses explicit public axes and the oracle uses slice iteration. All final
feature logs are green.

The ordinary release example records six cases: both preserved methods and
speeds -0.25, 0, +0.25 on a 3×1×1 grid, rho=2, dt=0.5. Nonzero cases have
pressure endpoints 0 and ∓2, kinetic gain 0.125, correction energy 0.125 and
boundary pressure work 0.25. Measured divergence and work-budget error are zero
in this hand-solvable fixture. Workspace array capacity is 168 bytes; outer
prescribed samples are excluded from interior kinetic degrees of freedom.
Jacobi uses two iterations, SGS one on the nonzero cases. These tiny counts are
numerical evidence, not a useful performance benchmark or convergence study.

The freshly compiled isolated release CLI replays demo-16/demo-plume/demo-64
with original PNG/CSV byte identity and original non-timing manifest fields
matching. Both original pressure implementations, simulation, tracer and all
book/proof artifacts are preserved. Exactly 1243 base paths outside README and
lib exports retain their Git blobs; preserved-base-inventory.json records them.
The collision module and regression test retain the shared repair's exact blobs.

AffineProjection.lean compiles seven exact finite algebraic theorems and five
definitions. Fourteen declarations (including generated ones) pass an axiom audit
allowing only propext/Classical.choice/Quot.sound. Actual disposable admitted and
custom-axiom modules compile, then fail the unmodified auditor with exit 1.
The historical source inventory and accepted PDF/input-byte gates pass. No full
historical root rebuild, Rust/IEEE certification or continuum claim is made.

## Reproduction and environment

```sh
cargo test --locked --offline --all-targets
cargo test --locked --offline --all-targets --no-default-features
cargo test --locked --offline --all-targets --features desktop
cargo clippy --locked --offline --all-targets -- -D warnings
cargo clippy --locked --offline --all-targets --no-default-features -- -D warnings
cargo clippy --locked --offline --all-targets --features desktop -- -D warnings
cargo fmt --all --check
CARGO_TARGET_DIR=/tmp/rheon-box-release cargo build --locked --offline --release --bin rheon
CARGO_TARGET_DIR=/tmp/rheon-box-release cargo run --locked --offline --release --no-default-features --example box_flux
python3 evidence/cloud-qualification/replay.py /tmp/rheon-box-release/release/rheon NEW_DIRECTORY
python3 proofs/scripts/check_sources.py
```

With pinned proof dependencies already built, run from proofs/:

```sh
mkdir -p /tmp/rheon-box-proof
lake env lean --root=../evidence/box-flux -o /tmp/rheon-box-proof/AffineProjection.olean ../evidence/box-flux/AffineProjection.lean
lake env bash -c 'export LEAN_PATH="/tmp/rheon-box-proof:$LEAN_PATH"; lean --root=../evidence/box-flux ../evidence/box-flux/AuditAffineProjection.lean'
```

Negative probes replace the first two-space ring tactic by sorry, or append
`axiom Rheon.BoxFlux.unreviewedAssumption : False`. Compile into a disposable
AffineProjection.olean and prefix that directory to LEAN_PATH for the unchanged
auditor. Actual probe exit codes are in lean-negative-receipt.json.

Rust/Cargo 1.92.0, LLVM 21.1.3, Lean 4.19.0. Rust tests use the ordinary test
profile; CLI/example use ordinary optimized release, one build job, no custom
Rust flags. Five exposed Intel Xeon Platinum 8370C CPUs in a cloud VM. Builds,
proofs and replay overlap; no isolated timing benchmark or peak RSS was measured.
The final debug target is isolated per worktree, seeded only with external
dependency artifacts after removing local Rheon fingerprints/libraries. Release
uses a fresh isolated target, preserving earlier producer binaries. Lean uses
identical preserved Rheon.Physics and pinned source-built Mathlib dependencies
in /workspace/Rheon-bounded-physics/proofs, without full-root requalification.

Scope: fixed connected full box, uniform normal inlet/outlet speeds, no pressure
opening on those faces. No mesh wall/cut-cell classification, moving swept
volume, tangential no-slip rule, free surface, conservative tracer transport,
full Simulation-step integration, graphical run or performance guarantee.
Output/pressure are unaccepted scratch on failure and must be published only
with a successful report; immutable input and boundary identity are preserved.
