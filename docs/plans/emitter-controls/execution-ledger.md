# Execution ledger

2026-10-13: inspected existing pressure-head cylinder, carrier forcing, particle
birth/volume-credit/timestep paths, native controls and normalized viewport. Read
Core/Router and canonical planning workflow, retaining prior applicable standards.
No code changed before this plan. Declared write scope for parent relay to collision
and CPU owners. New branch starts at fixed PR55 head; no merge to main authorized.


## Implementation and review

Implemented shared velocity-vector, independent source motion, and stateless seeded
perturbation controls. Existing carrier and particle paths retain transactional
publication. Added SI native editing and direction overlays; MaterialSession records
the complete accepted controls and advertises emitter version2/API version2. Added
headless example and user semantics documentation. No collision, CPU scheduling,
AI inference or dense3D format changes.

Independent agent `api_review` reviewed numeric bounds, rollback, default path, UI
units and docs. Its initial focused run found a test incorrectly expecting only one
birth despite equal opposed jets; corrected the oracle to inspect both separated
births. Review independently passed 31 focused/library tests and two native-control
unit tests. Fixed recommended disabled-jet overlay and selected-mode disk colors.
Strict checks caught redundant closure/range-loop style and example Debug output;
fixed the example to report explicit snapshot fields without expanding the API.

## Pinned-base numerical comparison

The first 200-step particle fixture emitted no particles; review identified that
coverage gap. Replaced it with 4,000 default-emitter updates each for water and air,
including particle births. Both pinned base3525c97 and current source produced:

- 12 carrier accepted steps, serialized X/Y/Z/tracer bits and time at every step.
- Water: maximum3 particles, final time4.992398162071807s.
- Air: maximum7 particles, final time0.3327298454905428s.
- Every particle step serializes dt/time/count and each position/velocity.
- Byte-identical output SHA256
  `9de1cca0c75d98cb5a5986e2f25e358d75d8cbc518bbcc852953276930c9e457`.

Reproduce by copying [probe](reports/default-baseline-probe.rs) into
`examples/emitter_baseline_probe.rs` in separate base/current checkouts and running
`cargo run --release --locked --no-default-features --example emitter_baseline_probe -- output.bin`,
then `cmp` the files. Remove the temporary example afterwards. This is bounded
fixture evidence on Linux x86_64/Rust1.92, not universal or cross-platform proof.

## Verification commands (Rust 1.92.0, cloud Linux x86_64)

All cargo commands use the repository lockfile. Artifact-inclusive tests set
`RHEON_AI_EXPORT=/tmp/rheon-api-model/modal.bin` (the existing real Modal export).

- `cargo test --release --locked --no-default-features -- --include-ignored`:
  **362 passed, 0 failed, 0 ignored**.
- `cargo clippy --locked --no-default-features --all-targets -- -D warnings`: passed.
- `cargo clippy --locked --features desktop --all-targets -- -D warnings`: passed.
- `cargo run --release --locked --no-default-features --example emitter_controls`:
  40 accepted updates,10 particles, time0.03499920593063877s; printed retained controls,
  capabilities, accepted stage and first particle.
- `cargo test --release --locked --features desktop -- --include-ignored`: **403 passed, 0 failed, 0 ignored**. Includes PNG, parallel, work observation and native desktop unit tests.
- `cargo test --release --locked --no-default-features --test emitter_controls_contract`: **8 passed** after strengthening the birth test to subtract and verify inherited source motion.
- `cargo fmt --all -- --check` and `git diff --check`: passed.
- Final independent review confirmed the expanded baseline (3,795,024 bytes) and ran the corrected example; no remaining implementation blockers.

No interactive display/window, GPU, Godot runtime or hardware performance was tested.
No realtime claim, validated turbulence model or persistent dataset archive claimed.
Source motion remains a prescribed inlet, not a moving solid collider. Inherited
research-book qualification/source-pin CI issue is owned by the book task; do not
weaken that checker in this emitter branch.
