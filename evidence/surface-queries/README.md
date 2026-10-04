# Static surface-query milestone

Branch `implementation/static-surface-queries`, based on accepted main
`a431d81a8ea815acfd12649aa279695381b4a78a`. The explicit-force candidate
6fbeebd1 remains separate and unchanged. This milestone implements a Chapter 20
geometry prerequisite, without depending on that candidate or modifying fluid
solver behavior. See the [book addendum](../../docs/research-book/implementation/static-surface-queries.md).

## Scope and actual acceptance

Owned indexed triangle surfaces support two-sided first-segment hits and clipped
endpoints, with normals, barycentric coordinates, object/version identity and
per-facet cancellation. Open sheets are admitted as surfaces without inventing
solid volume or inside/outside. Conditioning and detected ambiguous candidates
have explicit errors. Arrays are immutable after admission; retained capacity
is capped and queries allocate no heap buffers. The scan is linear, without BVH.

Twelve production fixtures pass: the translated book cube and hand-derived
t=0.3125, outside-to-outside crossings, misses, inside-start exit, oblique facets,
edge/vertex/endpoint contacts, reversed winding, ordering/ties, small/large scales,
near-parallel/coplanar/boundary ambiguity, mixed ambiguous/definite candidates,
cancellation after a found hit, capacity accounting and invalid-input admission.
Actual default/core-only/desktop totals are 62/57/68 tests; strict Clippy passes
all three configurations and formatting passes. No new graphical-window run is
claimed by the desktop build.

The release example executes nine queries against three immutable shifted cube
snapshots: six hits and three misses. Maximum parameter error against hand
arithmetic is 1.12e-16. Geometry spans 1e-100 to 1e100 in the separate scale
fixtures; this is numerical fixture evidence, not an arbitrary-range robustness
or physical-validation claim.

The exact final release CLI replays demo-16/demo-plume/demo-64 with byte-identical
original opacity.png/steps.csv and equal original non-timing manifest fields.
The CLI is built in an isolated target directory, preserving earlier producers.
No pressure method, original fixture, simulation stage, historical receipt,
published book/PDF, or historical root proof source/inventory is changed.

FacetSelection.lean adds six public exact-real theorems and three definitions.
Its audit checks eleven declarations including two generated equations, allowing
only propext/Classical.choice/Quot.sound. Actual disposable admitted/custom-axiom
fixtures are rejected. The module is outside the historical root inventory:
neither its statements nor its compilation relabel the earlier 42-theorem
qualification. Complete/correct candidate generation and IEEE predicates remain
unproved. The PDF/input-byte gate and historical proof source gate still pass.

## Commands and identities

```sh
cargo test --locked --offline --all-targets
cargo test --locked --offline --all-targets --no-default-features
cargo test --locked --offline --all-targets --features desktop
cargo clippy --locked --offline --all-targets -- -D warnings
cargo clippy --locked --offline --all-targets --no-default-features -- -D warnings
cargo clippy --locked --offline --all-targets --features desktop -- -D warnings
cargo fmt --all --check
cargo run --locked --offline --release --no-default-features --example surface_queries
CARGO_TARGET_DIR=/workspace/.rheon-tools/target-surface-queries cargo build --locked --offline --release --bin rheon
python3 evidence/cloud-qualification/replay.py /workspace/.rheon-tools/target-surface-queries/release/rheon NEW_OUTPUT_DIRECTORY
python3 proofs/scripts/check_sources.py
```

With the pinned dependencies and Rheon.BoundedPhysics built, run from proofs/:

```sh
mkdir -p /tmp/rheon-surface-proof
lake env lean --root=../evidence/surface-queries -o /tmp/rheon-surface-proof/FacetSelection.olean ../evidence/surface-queries/FacetSelection.lean
lake env bash -c 'export LEAN_PATH="/tmp/rheon-surface-proof:$LEAN_PATH"; lean --root=../evidence/surface-queries ../evidence/surface-queries/AuditFacetSelection.lean'
```

Local additive Lean qualification used the existing source-built pinned project
in the bounded-physics workspace, with identical BoundedPhysics bytes and exact
mathlib revision. Rust/Cargo 1.92.0, LLVM 21.1.3 and Lean 4.19.0 are used. Tests
use the dev profile; example/replay use normal optimized release, with one Cargo
build job and no custom Rust flags. The VM exposes five Intel Xeon Platinum
8370C CPUs. Builds, proof checks and replay overlap; this is not an isolated
benchmark. Exact source/binary/artifact hashes and limits are in receipt.json.

No exact/adaptive geometric predicate, signed-distance/solid classification,
self-intersection/manifold certification, moving wall, cut-cell assembly, fluid
coupling, performance guarantee or continuum truth is claimed. Coordinator owns
review and integration; no PR edits, external review requests or merges are made.
