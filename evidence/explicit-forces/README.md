# Explicit body-force milestone

Branch `implementation/explicit-body-forces`, based on preserved accepted
`a099cccaf71450958fa5f198dc6fc60397f711c2`. The dependency choice follows the
expanded book Chapters 17/19: externally prescribed forces precede moving
geometry, material/interface transport and two-way coupling. This milestone
adds production functionality rather than changing prior verification gates.

## Acceptance

`BodyForce` distinguishes acceleration from force density, supports three
components and optional world-space box support, and is borrowed per step.
`step_with_forces` applies it after advection/legacy smoke force and before
pressure, with accepted-only stage work diagnostics. The managed arrays and
both selectable pressure implementations are preserved. See the
[book addendum](../../docs/research-book/implementation/explicit-forces.md).

Actual default/core-only/desktop suites pass 61/56/67 tests. These include nine
new public-API fixtures plus two kernel fixtures: independent circulation/work
oracles, signed all-axis forcing, density conversion, half-open face support,
forcing travel budgets, mixed legacy/external stage work, negative/rounded work, cancellation after a partial
force update, failure/retry, and legacy empty-force stages/memory. Strict Clippy
passes in all three feature sets; formatting passes. The desktop result is a
compiled/tested feature build, without a newly exercised graphical window.

The exact candidate release CLI replays every original demo: opacity.png and
steps.csv match byte-for-byte for demo-16, demo-plume and demo-64; all original
non-timing manifest fields match. Its executable is built in a separate target
directory, preserving the prior stored replay producer. No old implementation,
fixture, source-bound receipt, accepted book/PDF or root proof inventory is removed.

Both release-mode force examples accept ten 16³ steps over 0.2 s, use 333824
managed-array bytes, and preserve the 1e-5 divergence gate. Maximum measured
divergence is below 3e-7; force-stage work/energy agreement is within 1.8e-15.
numerical-summary.json records actual metrics. These are finite numerical
acceptance results, not physical calibration or a performance guarantee.

The additive ForceWork.lean defines finite kinetic energy and stored work and
proves four exact-real contracts. The separate audit covers seven declarations,
including a generated proof, with only propext/Classical.choice/Quot.sound.
Disposable admitted/custom-axiom modules are actually rejected. These modules
are outside the historical root inventory and do not relabel its 42-theorem
qualification. The published PDF and all recorded rendering inputs still verify.

## Actual commands

```sh
cargo test --locked --offline --all-targets
cargo test --locked --offline --all-targets --no-default-features
cargo test --locked --offline --all-targets --features desktop
cargo clippy --locked --offline --all-targets -- -D warnings
cargo clippy --locked --offline --all-targets --no-default-features -- -D warnings
cargo clippy --locked --offline --all-targets --features desktop -- -D warnings
cargo fmt --all --check
CARGO_TARGET_DIR=/workspace/.rheon-tools/target-explicit-forces cargo build --locked --offline --release --bin rheon
python3 evidence/cloud-qualification/replay.py /workspace/.rheon-tools/target-explicit-forces/release/rheon NEW_REPLAY_DIRECTORY
cargo run --locked --offline --release --no-default-features --example forced_smoke
cargo run --locked --offline --release --no-default-features --example forced_smoke -- sgs-pcg-v1
python3 proofs/scripts/check_sources.py
```

With the existing pinned Lean/mathlib dependencies and Rheon.Physics built,
the additive proof can be reproduced from proofs/:

```sh
mkdir -p /tmp/rheon-force-proof
lake env lean --root=../evidence/explicit-forces -o /tmp/rheon-force-proof/ForceWork.olean ../evidence/explicit-forces/ForceWork.lean
lake env bash -c 'export LEAN_PATH="/tmp/rheon-force-proof:$LEAN_PATH"; lean --root=../evidence/explicit-forces ../evidence/explicit-forces/AuditForceWork.lean'
```

Local qualification used existing source-built pinned mathlib in the bounded
physics workspace, with identical Rheon.Physics source bytes and exact dependency
revision. Lean 4.19.0, Rust/Cargo 1.92.0 and LLVM 21.1.3 were used; Rust tests run
the dev profile, examples/replay the ordinary optimized release profile, without
custom Rust flags. The cloud VM exposes five Intel Xeon Platinum 8370C CPUs.
Desktop checking, replay and other read-only checks overlapped; this is not an
isolated benchmark. Hardware/build/binary/source identities are in receipt.json.

No moving wall, arbitrary mesh, viscosity, variable-density liquid, surface
tension, material fit, native-window or continuum validation is claimed. Lean
does not verify IEEE code or assembled face topology. Source review and
integration remain coordinator-owned; no PR edits/review requests/merge are made.
