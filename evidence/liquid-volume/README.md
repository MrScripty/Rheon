# Executed conservative liquid-volume milestone

The branch adds a separately owned represented liquid-volume transport state to
frozen box-flux simulation head `6c10349a828f6074490375e604fb5c3fc372fe9f`
(tree `8f92173578522da167ee67f646bcba41155733ee`). The master receipt binds the
qualified source commit/tree/ordered parent and every packet file, including
nested receipts. Historical receipts and their archive mappings retain their
original identities. Parent owns integration and review; no PR or review request
is created by this worker.

The early [scope](SCOPE.md) names fixed Cartesian cells, held carrier flow,
first-order shared-face transfers, signed volume sources and constant represented
liquid density. [Implementation notes](../../docs/research-book/implementation/conservative-liquid-volume.md)
give units, assumptions and limits. This state conserves represented volume with
an explicit reduction budget and rejects raw out-of-range candidates. It does
not clamp fractions or implement free-surface pressure or general mesh walls.

## Executed Rust checks

Actual all-target counts are **133 default, 128 core-only, 139 desktop**. All
passed; strict Clippy (`-D warnings`) passed in all three configurations, and
`cargo fmt --all --check` passed. The 14 new contract methods cover signed/all-axis
donors, inlet/outlet/source amounts, anisotropic closed circulation, oblique
constants, unsplit 3D shape error, independent slab refinement, thin merging
supports, raw bounds/CFL/divergence failures, partial cancellation and retry,
identities, interval/geometry, version overflow, capacities and underflow. Both
PCG identities supply readonly accepted carrier fields in one fixture.

Commands executed from the checkout, with `CARGO_BUILD_JOBS=1`,
`CARGO_INCREMENTAL=0`, locked dependencies and offline resolution:

```sh
cargo test --locked --offline --all-targets
cargo test --locked --offline --all-targets --no-default-features
cargo test --locked --offline --all-targets --features desktop
cargo clippy --locked --offline --all-targets -- -D warnings
cargo clippy --locked --offline --all-targets --no-default-features -- -D warnings
cargo clippy --locked --offline --all-targets --features desktop -- -D warnings
cargo fmt --all --check
cargo test --locked --offline --no-default-features --test liquid_volume_contract -- --nocapture
```

The debug target retained cached external dependencies after
`cargo clean -p rheon --profile dev`; local incremental compilation was disabled.
The fresh release target was `target-liquid-volume-release` under the cloud tool
directory. Rust/Cargo 1.92.0, LLVM 21.1.3; Cargo default debug/test and release
profiles. [Environment](environment.json) records Xeon Platinum 8370C, five
exposed affinity CPUs, a four-CPU cgroup quota and 16 GiB cgroup memory limit.
Rust checks, release compilation/replay and Lean overlapped. These are correctness
and spatial checks, with no isolated speed, peak-RSS or native-window claim.

The initial fixture compile failed on a borrowed grid moved into its state
constructor (`E0505`); the fixture now clones that immutable geometry. Logs retain
the failed build. The new module also initially rejected valid signed zero by a
redundant sign-bit guard. The targeted Rust test actually returned exit 101
([red log](red-signed-zero.log)); removing the redundant guard made it pass.
[Reconstruction](red-signed-zero-reconstruction.json) specifies exact accepted
and pre-fix blocks. Replacing only that block in a disposable checkout recreates
the recorded pre-fix module SHA. The historical fixture SHA predates the later
fixture additions/style cleanup; the signed-zero test remains in the final suite.
Neither fix changes an old implementation.

## Numerical and original-output evidence

```sh
cargo run --release --locked --offline --no-default-features --example liquid_volume -- FRESH_SPATIAL_DIRECTORY
python3 evidence/liquid-volume/verify_numerics.py
cargo build --release --locked --offline --bin rheon
python3 evidence/cloud-qualification/replay.py FRESH_RELEASE_BINARY FRESH_REPLAY_DIRECTORY
```

The checked-in summary CSV and full per-cell CSVs came from the executed release
example. The Python check independently recomputes geometric overlap, volume,
centroid and shape error, and compares every exported cell with a binomial oracle.

| Cells | Courant | Steps | dt (s) | L1 volume shape error (m³) | Owned array bytes |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 16 | 0.5 | 4 | 0.125 | 0.09375 | 904 |
| 32 | 0.5 | 8 | 0.0625 | 0.068359375 | 1800 |
| 64 | 0.5 | 16 | 0.03125 | 0.04909515380859375 | 3592 |
| 64 | 0.25 | 32 | 0.015625 | 0.06039030345194791 | 3592 |

Every case retains exactly 0.25 m³ and 250 kg at t=0.5 s with zero reported balance
error. Centroid error is at most 1.12e-16 m. Refining h at matched Courant improves
shape; reducing dt at fixed h increases this first-order scheme's diffusion.
The separately passed 3D cube fixture retains unit volume but has L1 shape error
0.65625 m³ after one step. Conservation alone does not imply interface accuracy.
The payload formula is 16*Ncell+8*Nface, with actual transferred Vec capacities
capped and no heap buffers allocated during advance. Inputs, allocator overhead
and RSS are excluded.

All three original Jacobi demos were freshly replayed: PNG and CSV bytes are
identical to the committed fixtures; all original non-timing manifest fields
match. The [nested replay receipt](legacy-replay/receipt.json) records commands
and hashes. Whole manifests contain elapsed timings and are not byte-identical.

## Lean and preservation

Lean 4.19.0 and pinned Mathlib
`c44e0c8ee63ca166450922a373c7409c5d26b00b` compiled
[VolumeLedger.lean](VolumeLedger.lean). Eight finite exact theorems and three
definitions establish shared-face/source/boundary balances, constant-density mass
scaling, conditional donor bounds/constants, and explicit CFL/clamp
counterexamples. The audit checks 12 declarations, including a generated equation,
and permits only `propext`, `Classical.choice`, `Quot.sound`. Real disposable
variants replacing the first `ring` with `sorry`, or adding
`axiom Rheon.LiquidVolume.unreviewedAssumption : False`, each compile at exit 0
and fail the unmodified audit at exit 1. Actual build/audit/negative logs are kept.

Reproduction uses the existing source-built dependency closure in
`/workspace/Rheon-bounded-physics/proofs`, the unchanged `Rheon.Physics` import
(SHA256 `26cbb4f75076c7e70a90da126f96622b75044befce26a4c17a50066c76e96050`),
and a fresh temporary olean directory on `LEAN_PATH`:

```sh
lake env lean --root=ABSOLUTE_PACKET -o FRESH_OLEAN/VolumeLedger.olean ABSOLUTE_PACKET/VolumeLedger.lean
lake env bash -c 'LEAN_PATH=FRESH_OLEAN:$LEAN_PATH lean --root=ABSOLUTE_PACKET ABSOLUTE_PACKET/AuditVolumeLedger.lean'
python3 proofs/scripts/check_sources.py
```

The Lean statements assume supplied incidence and donor weights; they do not
certify Rust stencil assembly, IEEE arithmetic, interface geometry or continuum
accuracy. The full historical root was not rebuilt. The original reviewed source
inventory/dependency gate and accepted book PDF/input-byte gate passed.

The base inventory verifies **1482 existing files unchanged**, excluding only
the added README section and crate exports. Simulation, pressure, smoke tracer,
geometry, forces, collision and all earlier proof/book/evidence files retain exact
base Git blobs. Fourteen frozen feature/integration worktrees remain at their
recorded heads and clean. Six prior producer hashes match their historical
receipt; the accepted box-step producer hash is additionally recorded here.

After the master receipt is present:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 evidence/liquid-volume/verify_packet.py
```

Cancellation or rejection preserves accepted fraction bits, state version and
clock; retry overwrites scratch and matches a fresh call. Separate already
accepted carrier state remains accepted if volume later rejects. This milestone
therefore makes a volume-only transaction claim. Unified phase/pressure ownership,
free-surface coupling, geometric reconstruction, mesh walls, viscosity, wetting,
surface tension and physical calibration remain outside this slice.
