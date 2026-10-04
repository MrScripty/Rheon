# Static passive-tracer barrier milestone

Branch `implementation/static-tracer-barriers` is based on independently reviewed
translation head `28fe4682e743a74e8f57eec98ad94475f9e5a564`. Reviewed force, static
surface, composition and translation heads remain unchanged. This production
slice follows Chapters 6/17/20: explicit scalar communication rules precede a
complete compatible fluid-boundary operator. See the
[book addendum](../../docs/research-book/implementation/static-tracer-barriers.md).

## Scope and executed acceptance

The opt-in owned simulation step checks both clamped midpoint trace legs against
one static two-sided triangle surface. Contact reverts to the old arrival-cell
value. Clear traces sample only directly visible positive-weight donors,
renormalizing their weights and clamping to their range. The original sampler,
advection kernels and pressure implementations are retained. Velocity/pressure
continue to use the original fixed-box model: this is passive-concentration
appearance transport, not an impermeable fluid wall. Sources follow the barrier
stage as before. Units, no-donor/contact rules and numerical limits are explicit
in the addendum. No moving interval is silently converted to a static barrier.

Actual tests pass 95 default, 90 core-only and 101 desktop tests; strict Clippy
passes all three configurations and formatting passes. Twelve new contract
methods exercise all sheet orientations, visible range/constants, full 3D
renormalization, first/second trace contacts, stencil leakage despite a clear
trace, unobstructed legacy bits, no-visible-donor/ambiguity/outside input failures,
partial scratch cancellation and actual simulation publication/error/retry,
optional-path callback/state equivalence, force/pressure/memory preservation and
both solver IDs. No new graphical-window run is claimed.

A real red fixture caught the new sampler's pre-clamp variant returning
3.2500000000000004 instead of the sole positive-donor value 3.25 when zero-weight
corners were 100. The positive visible-range clamp fixes this demonstrated
violation without editing ScalarSampler. The actual red and focused green logs
are retained. To recreate the red variant, remove only the final visible-range
clamp in VisibleTracerSample construction and run
`cargo test --locked --offline --no-default-features --test tracer_barrier_contract zero_weight_extremes_cannot_relax_the_positive_donor_range`.

The release example executes eight accepted updates per implementation on a
2×2×1 circulation fixture, starting from a source-seeded left concentration.
Legacy concentration at the selected right cell reaches 0.012619733810424805;
barrier concentration remains exactly zero in all sixteen records. Each barrier
update records two blocked donor corners and zero reverted traces, so the example
isolates stencil communication. Managed simulation arrays remain 384 bytes, plus
96 independently owned surface-array bytes; actual divergence is zero in this
small fixture. These are bounded numerical results, not physical calibration,
wall pressure evidence, refinement validation or a performance guarantee.

The exact final release CLI replays demo-16/demo-plume/demo-64 with byte-identical
original PNG/CSV fixtures and matching original non-timing manifest fields.
Its isolated target preserves previous producer binaries. No old implementation,
source-bound receipt, proof inventory, accepted PDF or book input is removed.

VisibleWeights.lean compiles eight conditional exact-real theorems and four
definitions. Fourteen declarations, including two generated equations, pass the
namespace audit with only propext/Classical.choice/Quot.sound. Actual admitted and
custom-axiom disposable modules are rejected. Imported Rheon.Transport was built
from identical preserved source bytes using the existing pinned dependencies.
The historical source-inventory gate and PDF/input-byte gate pass. No complete
historical root rebuild or relabeling of its 42-theorem qualification is claimed.

## Actual commands and environment

```sh
cargo test --locked --offline --all-targets
cargo test --locked --offline --all-targets --no-default-features
cargo test --locked --offline --all-targets --features desktop
cargo clippy --locked --offline --all-targets -- -D warnings
cargo clippy --locked --offline --all-targets --no-default-features -- -D warnings
cargo clippy --locked --offline --all-targets --features desktop -- -D warnings
cargo fmt --all --check
CARGO_TARGET_DIR=/workspace/.rheon-tools/target-tracer-barriers cargo build --locked --offline --release --bin rheon
CARGO_TARGET_DIR=/workspace/.rheon-tools/target-tracer-barriers cargo run --locked --offline --release --no-default-features --example tracer_barrier
python3 evidence/cloud-qualification/replay.py /workspace/.rheon-tools/target-tracer-barriers/release/rheon NEW_DIRECTORY
python3 proofs/scripts/check_sources.py
```

With pinned dependencies and Rheon.Transport built, reproduce from proofs/:

```sh
mkdir -p /tmp/rheon-tracer-proof
lake env lean --root=../evidence/tracer-barriers -o /tmp/rheon-tracer-proof/VisibleWeights.olean ../evidence/tracer-barriers/VisibleWeights.lean
lake env bash -c 'export LEAN_PATH="/tmp/rheon-tracer-proof:$LEAN_PATH"; lean --root=../evidence/tracer-barriers ../evidence/tracer-barriers/AuditVisibleWeights.lean'
```

Local Lean qualification uses existing source-built pinned dependencies in
`/workspace/Rheon-bounded-physics/proofs`, with identical imported source bytes.
Rust/Cargo 1.92.0, LLVM 21.1.3, Lean 4.19.0; dev tests and ordinary optimized release
example/CLI, one Cargo build job, no custom Rust flags. Cloud VM: five exposed
Intel Xeon Platinum 8370C CPUs. Builds/proofs/example/replay overlap; no isolated
benchmark, peak-RSS or performance guarantee is claimed.

Limits: static direct visibility and piecewise linear trace model; numerical
predicates/rounding without exact certification. No fluid obstacle classification,
internal wall velocity/pressure flux, cut volumes/connectivity, conservative mass
transport, no-slip/wetting, swept moving-wall volume, actuator work or continuum
truth. The original sampler is preserved; None uses the exact legacy kernel.
Failure leaves scratch partial but accepted state coherent; caller owns surface
identity/version. Coordinator owns PR, independent review and main integration.
