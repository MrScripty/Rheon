# Executed coherent carrier / represented-liquid transport milestone

Qualified source commit fd71f48df1f726ae130ba9b18ea02a6ecec26937, tree
17e7506a426e112311183328175cdb9f12bd4a94, parent composition evidence milestone
8e3ea72a820fbbcfde40f14146f6b9536a109608. That parent freezes the real native
qualification of PR16 main plus accepted scale-guard source; its evidence and
ordered merge parents are retained unchanged.

The public LiquidTransportSimulation owns the existing carrier and liquid
owners and explicitly counted accepted pressure. Private preparation precedes
one final pair publication. The API covers closed carrier boxes and opt-in
prescribed box flow, borrowed forces/sources, explicit liquid inlet donors,
read-only accepted state, typed stages/errors, pause, actual dt and memory totals.
Velocity, pressure, appearance tracer, fractions, both clocks and version stamps
survive rejection and cancellation. The facade currently has no reset/import API.
See [equations, API, memory inventory and proof boundaries](../../docs/research-book/implementation/coupled-liquid-transport.md).

## Native and regression qualification

Final actual Rust counts are **147 default, 141 core-only, 153 desktop**.
All-target formatting and Clippy with warnings denied pass in every feature
configuration. Focused liquid tests total 27: 18 retained volume tests, eight
coupled methods (including feature-gated PNG integration), and one arithmetic
method executing four scale-division cases twice. The original 18 liquid
fixtures and pressure/operator/advection/sampling source bytes are unchanged.
The shared PR16 divergence-error conversion is retained.

The coupled tests cover both original PCG implementations, all signed axes,
hand-derived pressure and donor transfer, actual dt selection, closed rest,
capacity caps (including excess transferred fraction Vec capacity), pause,
early input/version failures, late raw-volume rejection after new pressure,
and every exercised carrier/volume/final callback stage at first and last
occurrence. Accepted pressure is initially nonzero in the late rejection test:
scratch's newer pressure never leaks into the accepted view. Failed/cancelled
retries match uninterrupted field bits. The cancellation fixture includes
already accepted nonzero appearance tracer as well as nonzero pressure.
Four native arithmetic cases separately reach the cell loop and reject
subnormal/rounded-to-zero candidate fractions, overflowing Courant division and
subnormal divergence division without publishing state.

commands-final.json and release-commands.json qualify this exact final source.
Earlier logs are retained as development history: focused-first.log predates
PNG export, clippy-first.log records an example usize/u64 compile failure,
fixed with checked conversion. The first complete checks precede extraction of
the PNG dimension helper; final checks/release were rerun afterwards. No earlier
draft logs are claimed as final-source qualification.

Rust/Cargo 1.92.0, LLVM 21.1.3, locked dependencies, one build job, Cargo default
test/debug/release profiles. External dependencies were downloaded normally;
no bypass or credential changes. Default/core/desktop includes compile/tests of
the optional GUI, not a native window demonstration. The existing Python
comparison/result lifecycle harness passed all **20** real CLI tests.

## Actual 3D transport, pressure and guidance evidence

The release example runs eight scenarios: each original pressure implementation
at 16/32/64 × 8 × 4 cells with outward Courant 0.25, plus 64 × 8 × 4 at 0.125.
A fraction-0.5 slab initially occupies x=[0.25,0.5] in the unit box, with
prescribed carrier through-flow 0.25 m/s. Every run ends at the common
carrier/liquid clock 0.5 s. Represented density is 800 kg/m³ and carrier density
1000 kg/m³; they are intentionally independent constants.

Each run exports accepted initial/first/final native PNGs, every-step numerical
budgets, first pressure, final pressure/fractions, all final face velocities and
a final completion manifest. The independent Python gate imports no solver code.
It recomputes layout/volume/mass/centroid, actual all-cell velocity divergence,
the manufactured first pressure, every-cell binomial donor fractions, exact
translated overlaps and every pixel's fraction-integral appearance equation.
Positive and tampered fraction/clock/pressure/pixel gates pass normally and under
Python -O; numerical requirements remain active in optimized execution.

| X cells | Courant | Steps | L1 volume shape error (m³) | Facade array bytes |
| ---: | ---: | ---: | ---: | ---: |
| 16 | 0.25 | 8 | 0.05837440490722656 | 69120 |
| 32 | 0.25 | 16 | 0.04222469791420731 | 137728 |
| 64 | 0.25 | 32 | 0.03019515172597401 | 274944 |
| 64 | 0.125 | 64 | 0.03263673847939313 | 274944 |

Both methods agree in these errors to rounding. Independently reduced volume
is 0.125 m³ and represented mass is 100 kg; final centroid is 0.5 m to rounding.
Maximum binomial fraction disagreement is below 3e-13, first pressure error below
1e-8 Pa, and final direct divergence below 2e-12 /s. See the full
[numerical summary](https://github.com/MrScripty/Rheon/blob/9cd4587a54befa61bdfddc8e35014bd3c34f02fb/evidence/liquid-step/numerical-summary.json), per-cell source data under demo/
and [visually inspected figure](https://github.com/MrScripty/Rheon/blob/9cd4587a54befa61bdfddc8e35014bd3c34f02fb/evidence/liquid-step/guidance-comparison.png).
Spatial refinement improves the shape; halving dt at fixed h increases
first-order donor diffusion. No pouring, droplet, film or material calibration
is established. The fraction-0.5 initial support intentionally avoids full-cell
raw-bound sensitivity to small carrier residuals, which remains a rejection
condition rather than a silently relaxed gate.

A second final release execution produced **65 byte-identical output files**,
including all eight scenario CSVs/PNGs/manifests and the completion manifest.
The [replay receipt](replay-receipt.json) lists their hashes. All original Jacobi
demo PNG/CSV bytes and stable manifest fields also replay exactly; timing fields
in those smoke manifests remain excluded from whole-file equality.

The PNG is +Z fraction-integral guidance, coefficient 8/m and +Y upward. It is
not a reconstructed liquid surface, normal/depth map or physical scattering.
Its raw pixels have an explicit separate cap. The figure arranges the actual
native images alongside accepted fraction and independently computed error data.
Python verifier/render arrays and example IO lie outside solver memory accounting.

## Storage and preserved proof scope

There are 20 facade-owned arrays with nominal payload 16 F + 80 N bytes,
including one accepted f64 pressure array. Optional borrowed boundary scratch
retains its existing independent cap and 4 F + 56 N inventory. At n=64 in this
3D example: 274944 facade bytes plus 142464 boundary bytes = 417408 combined.
Actual Vec capacities, including transferred initial fraction capacity, are
counted; no hidden rollback snapshot is created. Kernels reuse candidates and
scratch without heap allocations. Caller callbacks, source/force fields,
allocator overhead, stacks, process RSS and export/encoder workspaces are excluded.
No whole-process cap or isolated real-time/performance measurement is claimed.

preservation.json checks **1348** frozen evidence/proof/original-chapter/PDF
Git blobs from the composition parent, and the composition verifier still passes.
The reviewed proof-source inventory and PDF input/byte freshness gates pass.
Existing Lean sources, axiom inventory and receipts are unchanged. No new Lean
build or theorem is required or claimed; retained qualification belongs to its
original exact source/evidence identities. Finite ledger and affine projection
theorems assume supplied incidence/coefficients/solutions. They do not verify
the unified Rust publication, indexing/assembly, IEEE arithmetic, convergence,
geometry, continuum accuracy or performance.

This is a **transport-only coupled step**, not a fully validated liquid simulator.
Free-surface pressure/air extension, density feedback, geometric surface
reconstruction, mesh-solid/cut-cell/moving swept-volume physics, viscosity,
traction, adhesion/wetting and capillarity remain separate future work.
Parent owns PRs, external reviews, merges and PR16 postmerge CI; this branch makes
no such request and does not alter main.

## Reproduce

    cargo fmt --all --check
    cargo test --locked --all-targets
    cargo test --locked --all-targets --no-default-features
    cargo test --locked --all-targets --features desktop
    cargo clippy --locked --all-targets -- -D warnings
    cargo clippy --locked --all-targets --no-default-features -- -D warnings
    cargo clippy --locked --all-targets --features desktop -- -D warnings
    cargo run --release --locked --example liquid_step -- FRESH_OUTPUT
    python3 evidence/liquid-step/verify_numerics.py FRESH_OUTPUT
    python3 -O evidence/liquid-step/test_verify_numerics.py
    python3 evidence/liquid-step/verify.py
    python3 -O evidence/liquid-step/verify.py

Independent image checks use Pillow 12.3.0; figure rendering uses Matplotlib
3.10.8. No extra dependency is introduced into the Rust simulation or core-only
feature graph. Receipt verification reads the frozen source commit, so future
source development does not rebind this packet.
