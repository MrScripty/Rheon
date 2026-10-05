# Prescribed translating-surface milestone

Based on separately qualified integration candidate
`6a03fcfd8b90ec4121f184573346e08a086a4f78`, which preserves complete accepted
forces and surface histories. Original surface head `777a7339`, force head
`6fbeebd1`, main `6ba1cbe4` and their evidence remain frozen. The book Chapter 20
qualification ladder motivates relative-coordinate translation before rotation,
cut geometry and a moving-wall fluid update. See the
[implementation addendum](../../docs/research-book/implementation/translating-surfaces.md).

`TranslationInterval` borrows immutable reference geometry, validates physical
times and endpoint translations, and maps a linear world point trajectory into
one reference segment. It reports an explicit reference hit, separate world
contact, physical time, prescribed wall velocity and caller-owned interval ID.
Queries allocate no geometry arrays and propagate cancellation/ambiguity without
partial publication. A stationary world point is supported when relative motion
is nonzero; a zero relative segment is explicitly unsupported. Pose overflow,
invalid durations, velocity overflow/underflow-to-zero and invalid trajectories
reject before a result. Pure translation preserves the reference normals and
barycentric weights. No wall continuation/response is supplied after clipping.

Fresh tests pass 83 default, 78 core-only and 89 desktop tests; strict Clippy
passes all three feature configurations and formatting passes. Ten new fixtures
include 216 hand-oracle queries across all axis permutations, both motion signs
and tangential translation, plus stationary-point sweeps, end-pose misses,
shared-frame invariance, zero-motion static equivalence, reversed winding,
endpoint contact, clipping, cancellation after a found hit, retry and admission
failures. Desktop evidence is compilation/testing, not a new graphical-window run.

The release example executes three trajectories: two independently checkable
contacts (parameters 1/2 and 2/3) and one miss. The isolated final release CLI
replays every original Jacobi demo with byte-identical PNG/CSV fixtures and equal
non-timing manifest fields. Earlier producer binaries and all original pressure,
force, transport, static geometry, proof/book and receipt bytes are preserved.

TranslationFrame.lean compiles seven conditional exact-real theorems and four
definitions. Its namespace audit covers twelve declarations including a generated
clock equation, with only propext/Classical.choice/Quot.sound allowed. Actual
admitted/custom-axiom modules are rejected. The imported BoundedPhysics/Physics
source bytes and pinned mathlib revision match this checkout. Historical root
proof inventory is unchanged and its source gate passes; no full root rebuild
or extra historical theorem qualification is claimed. The preserved PDF and all
its recorded input hashes pass the freshness gate.

## Actual commands and environment

```sh
cargo test --locked --offline --all-targets
cargo test --locked --offline --all-targets --no-default-features
cargo test --locked --offline --all-targets --features desktop
cargo clippy --locked --offline --all-targets -- -D warnings
cargo clippy --locked --offline --all-targets --no-default-features -- -D warnings
cargo clippy --locked --offline --all-targets --features desktop -- -D warnings
cargo fmt --all --check
CARGO_TARGET_DIR=/workspace/.rheon-tools/target-translating-surfaces cargo build --locked --offline --release --bin rheon
CARGO_TARGET_DIR=/workspace/.rheon-tools/target-translating-surfaces cargo run --locked --offline --release --no-default-features --example translating_surface
python3 evidence/cloud-qualification/replay.py /workspace/.rheon-tools/target-translating-surfaces/release/rheon NEW_DIRECTORY
python3 proofs/scripts/check_sources.py
```

With pinned dependencies and BoundedPhysics built, reproduce from proofs/:

```sh
mkdir -p /tmp/rheon-translation-proof
lake env lean --root=../evidence/translating-surfaces -o /tmp/rheon-translation-proof/TranslationFrame.olean ../evidence/translating-surfaces/TranslationFrame.lean
lake env bash -c 'export LEAN_PATH="/tmp/rheon-translation-proof:$LEAN_PATH"; lean --root=../evidence/translating-surfaces ../evidence/translating-surfaces/AuditTranslationFrame.lean'
```

Local Lean qualification uses the existing source-built pinned dependencies in
`/workspace/Rheon-bounded-physics/proofs`, with identical imported source bytes.
Rust/Cargo 1.92.0, LLVM 21.1.3, Lean 4.19.0; dev tests and normal optimized release
example/CLI, one Cargo build job, no custom Rust flags. The cloud VM exposes five
Intel Xeon Platinum 8370C CPUs. Builds/proof/example/replay overlap: no isolated
benchmark, performance/RSS guarantee or continuum calibration is claimed.

Limits: constant translation and linear point trajectories only; numerical
subtractive/static-predicate/reconstruction rounding is not exact/adaptive
certification. No rotation, persistent-contact classification, solid sign,
cut-cell/pressure geometry, swept-volume conservation, fluid coupling, friction,
wall-work or two-way reaction is implemented. Caller owns interval identity.
No PR edit, external review request or main integration is performed by the author.
