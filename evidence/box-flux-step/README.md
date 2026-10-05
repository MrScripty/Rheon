# Prescribed box flux in transactional simulation steps

This separate milestone extends the independently accepted standalone primitive
without changing original pressure, advection, sampler, force or geometry code.
The base is the refreshed repaired stack; its exact head/tree and ordered merge
ancestry are recorded in receipt.json. Existing frozen heads remain unchanged.
See the [book addendum](../../docs/research-book/implementation/box-flux-simulation.md)
for units, boundary assignment, work and conservation assumptions.

## Scope and qualification

Actual final all-target suites pass 119 default, 114 core-only and 125 desktop
tests. Strict Clippy passes all three configurations, and formatting passes.
The final release CLI replays all three original Jacobi PNG/CSV fixtures byte
identically, with original non-timing manifest fields equal. Source inventory
and accepted PDF/input byte gates pass. All 1431 base paths except README,
lib exports and simulation dispatch/ledger keep exact Git blobs.

The release example runs five accepted steps per solver on a 3×2×1 grid. The
maximum full-step work-budget error is 2.42861286636753e-17 and maximum actual
stored divergence is 2.2351741790771484e-8. Net volume imbalance is zero in this
fixture. Simulation arrays remain 568 bytes, caller workspace arrays are 452,
combined 1020. The first nonzero step records 0.5 boundary pressure work; final
interior energy is 0.30700000262683536. These are bounded numerical interaction
results, without timing, refinement, physical calibration or peak-RSS claims.

`step_with_box_flux` advects previous accepted velocity, applies the original
smoke/external forcing, projects with copied end boundary data, gates actual
stored divergence and Courant, transports tracer under an explicitly selected
ClampedAppearance policy, then adds sources and publishes all state/reports
at the existing final commit. An independently capped caller workspace owns
affine-pressure scratch and provisional face copies. The simulation's existing
allocation remains retained and separately counted. The report uses interior
unknown-face energy and records advection, smoke/external force and boundary
pressure work, explicit residual terms and the complete step budget error.

Twelve integration fixture methods cover repeated signed flow along all axes and
both solver identities, requested boundary speed in dt selection, version changes
and failed publication, smoke/external work, source saturation and clamped tracer
integrals, all eleven checkpoint kinds (including partial correction/tracer
updates), bit-preserved failure/retry, independent incompatible flux, pressure
exhaustion, late source arithmetic error, workspace geometry/density/method
matching, exact capacity/singletons, pause/reset, zero-flux legacy fields and
callbacks, and stored divergence rejection after pressure convergence.

The nonlinear 3×2×1 fixture has balanced fluid volume and bounded tracer, yet
transport integral growth is 0.0625246912240982 rather than the old endpoint
upwind boundary reference 0.0625. This documents the absence of a conservative
scalar boundary-flux contract; it does not relabel appearance concentration as
physical smoke mass. No specified reservoir concentration is implemented.

StepBudget.lean compiles seven exact finite theorems and two definitions. Ten
declarations including a generated equation pass the namespace axiom allowlist.
Actual disposable sorry/custom-axiom modules compile then fail the unchanged
auditor. Stage work composes with the preserved affine projection theorem;
conditional energy nonincrease needs nonpositive total work and nonnegative
masses. Row normalization preserves constants; column balance preserves scalar
sum for uniform cell volume. A two-cell normalized counterexample separates the
assumptions. No Rust/IEEE/continuum proof or full historical-root rebuild is
claimed. Initial wrong-workdir proof invocation and missing shorthand lemma logs
are retained; final proof uses explicit decidable Fin-2 enumeration.

The final run removes local package artifacts and disables incremental reuse
after a seeded-cache core build linked old exports; that failure log is retained.
A nested workspace-match condition was simplified after strict Clippy rejected
it. Final test/build counts, strict Clippy results, example metrics, fresh original
replays, source/PDF gates, producer and artifact hashes are bound in receipt.json.
Original runtime APIs retain their closed-box callbacks and capacities. Historical
composition receipts retain original source identities and byte-preserved path
mappings from the refreshed stack; they are not rebound to this feature.

## Reproduction

```sh
cargo test --locked --offline --all-targets
cargo test --locked --offline --all-targets --no-default-features
cargo test --locked --offline --all-targets --features desktop
cargo clippy --locked --offline --all-targets -- -D warnings
cargo clippy --locked --offline --all-targets --no-default-features -- -D warnings
cargo clippy --locked --offline --all-targets --features desktop -- -D warnings
cargo fmt --all --check
CARGO_TARGET_DIR=/tmp/rheon-step-release cargo build --locked --offline --release --bin rheon
CARGO_TARGET_DIR=/tmp/rheon-step-release cargo run --locked --offline --release --no-default-features --example box_flux_step
python3 evidence/cloud-qualification/replay.py /tmp/rheon-step-release/release/rheon NEW_DIRECTORY
python3 proofs/scripts/check_sources.py
```

Using the pinned built dependencies, from proofs/:

```sh
mkdir -p /tmp/rheon-step-proof
lake env lean --root=../evidence/box-flux -o /tmp/rheon-step-proof/AffineProjection.olean ../evidence/box-flux/AffineProjection.lean
lake env bash -c 'export LEAN_PATH="/tmp/rheon-step-proof:$LEAN_PATH"; lean --root=../evidence/box-flux-step -o /tmp/rheon-step-proof/StepBudget.olean ../evidence/box-flux-step/StepBudget.lean'
lake env bash -c 'export LEAN_PATH="/tmp/rheon-step-proof:$LEAN_PATH"; lean --root=../evidence/box-flux-step ../evidence/box-flux-step/AuditStepBudget.lean'
```

Negative probes replace the first two-space ring tactic with sorry or append
`axiom Rheon.BoxFluxStep.unreviewedAssumption : False`. Compile disposable
StepBudget.olean and prefix that directory to LEAN_PATH before running the
unmodified auditor, retaining the preserved AffineProjection dependency.

Rust/Cargo 1.92.0, LLVM 21.1.3, Lean 4.19.0 and pinned Mathlib. Ordinary test profile
and ordinary optimized release, one Cargo build job, no custom Rust flags. Five
exposed Intel Xeon Platinum 8370C CPUs in a cloud VM. Builds/proofs/replays overlap;
no isolated timing benchmark, peak RSS or native graphical run is claimed.
Final debug qualification uses CARGO_INCREMENTAL=0. Targets isolate local package
artifacts; imported source bytes and previous
producer binaries are preserved. Work uses exact finite algebra in Lean and
measured stored f32 fields in Rust; the two scopes are explicitly separate.

Limits: fixed connected full rectangular box, prescribed uniform normal side
speeds and fixed positive density. No mesh/cut-cell/moving-wall swept volume,
pressure openings, free surfaces, tangential no-slip, conservative tracer,
reservoir concentration, combined tracer-barrier/inlet step or continuum
calibration. Parent owns PRs, external review and integration; author publishes
only separate milestone branches.
