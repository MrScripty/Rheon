# Execution ledger

## 2026-10-13 — transport milestone

Fetched origin/main `2209012`, preserved the original clean saved checkout, and
created `/workspace/rheon-api` on `feat/public-simulation-api`. No local AGENTS or
skills existed. Parent later identified the external standards source; reviewed
public Coding-Standards at `188beda1fa477d21d576c233dd8c7f4c4c267d23` and added
canonical plan/design/issues artifacts. No full standards-engine approval claimed.
The branch/worktree remains retained-protected for parent draft review; its head
will be reachable from the published feature ref. Parent owns integration and
terminal disposition; no merge/deploy or history rewriting was performed.

Implemented existing full-box controls through SimulationSession, immutable
constructor/accepted snapshots, caller source/run identity, capabilities/units,
terminal stop, accepted-step quota and cooperative monotonic deadlines. No physics
algorithm or dense3D wire contract changed. Production clock remains Instant::now;
private clock injection gives deterministic final-callback cancellation coverage.

Environment: Linux cloud CPU, Rust/Cargo 1.92.0; installed pinned toolchain under
/workspace because cargo was initially absent. Commands below use
`CARGO_HOME=/workspace/.cargo RUSTUP_HOME=/workspace/.rustup` and the corresponding
cargo binary. No unsafe code, new dependency, external service or generated binding.

Passed:

- `cargo test --locked --no-default-features --lib --test session_contract`:
  14 library tests and 5 integration tests. New API evidence is one deterministic
  deadline unit test and five public integration tests. Independent reviewer reran
  this command (without --locked) and obtained the same counts; no blocking issues.
- `cargo clippy --locked --no-default-features --all-targets -- -D warnings`.
- `cargo clippy --locked --all-targets -- -D warnings` (default features).
- `cargo fmt --all -- --check` and `git diff --check`.
- `cargo test --locked --no-default-features --features parallel --test session_contract
  --test dense3d_controls_contract --test dense3d_sequence_contract` passed before
  the clock-test refactor; final focused verification above covers that refactor.
- `python3 -m unittest discover -s tools -p 'test_dense3d*.py'`: 39 tests,
  one existing environment-gated skip; no failures.
- `cargo run --locked --no-default-features --example headless_session`:
  nine JSON-parsed frames, final accepted time 0.5 s, array payload 3536 bytes.
- `cargo run --locked --no-default-features --example game_loop_session`:
  frame 2, accepted time 0.125 s, retained lag zero.

Broad regressions completed successfully: `cargo test --locked --no-default-features`
passed 329 tests across 41 test targets; `cargo test --locked` passed 337 tests across
42 test targets, zero failed/ignored. These runs began before the private clock-test
refactor; final focused verification and strict lint cover the changed clock path.
Parallel session/dense3D tests passed 20 tests before the refactor.

Published commit `3dffefd` on `feat/public-simulation-api`, draft
https://github.com/MrScripty/Rheon/pull/48 . M1/M2 accepted; broader objective remains
blocked on parent coordination of the peer backend contracts.
Inspected quantitative PR #47 at `1923d5bd3442a5369878a1a317b52f4a89026d5b`
read-only. Its ParticleDiagnostic and accessor are the selected future source;
recorded evaluation/publication stage distinction and run-local particle identity
in the plan. No branch merged, no capture type duplicated.

Independent review drove added checkpoint rollback, accepted-dt and deadline
coverage. The sleep-based deadline test was replaced after standards/review feedback.
No hardware performance measurements, Godot runtime, desktop rendering, GPU,
AI provider execution or downstream Tuldok consumer adoption are claimed.

Final clock-refactor matrix also passed:
`cargo test --locked --lib --test session_contract` (14 + 5), and
`cargo test --locked --no-default-features --features parallel --lib
--test session_contract --test dense3d_controls_contract --test dense3d_sequence_contract`
(19 + 5 + 6 + 8 = 38). Both default/core strict Clippy passed on final source.

## 2026-10-13 — material and qualified provider stack

User clarified that API owner designs integration. Superseded waiting on a
parent-authored combined contract. Created `feat/material-simulation-api` from PR #48
`f06a127`, merged full PR #47 `1923d5b` (merge `9754011`), implemented material API
in `177e800`, then merged full PR #49 `b8ca87c` (merge `4cb5a6e`). Resolved only the
additive `src/lib.rs` conflict by retaining both declarations. Main and the PR #48
branch were not changed; dependency history and other-owner code are preserved.

MaterialSession connects actual material config/emitter controls, transactional
particles and PR #47's canonical diagnostics. PressureSession connects actual
provider selection, model/provenance, explicit reference fallback, qualification
reports and retained pressure. It does not replace full-step physics acceptance.
CAD PR #50 remains owner of rendering and VTK encoding; API provides raw slices.

Toolchain and host are the same Linux CPU/Rust 1.92.0 environment. Downloaded the
actual frozen checkpoint from lab source `e8a90bfef6b2955c502008ccc62443e29dd9f2ec`.
`tools/export_ai_pressure.py` verified checkpoint SHA256
`25533a2fbaa5ab0f5fb574dc6307a2b8d6a686deb7db867d0d03338d4d70d5a9` and produced export
SHA256 `902ad8d77b16ae42f22f90bf71c22c19ae2e3cbe0cc0f4cc3d770930d4049aea`.
Weights and generated numerical outputs remain outside Git under /tmp.

Executed exact configuration commands (CARGO_HOME/RUSTUP_HOME as above,
RHEON_AI_EXPORT=/tmp/rheon-api-model/modal.bin):

- `cargo test --release --locked --no-default-features -- --include-ignored`:
  **347 passed, 0 failed, 0 ignored**, 45 test targets including empty targets.
- `cargo test --release --locked -- --include-ignored`:
  **354 passed, 0 failed, 0 ignored**, 46 test targets including empty targets.
- `cargo test --release --locked --no-default-features --features parallel --lib
  --test material_session_contract --test pressure_session_contract --test session_contract
  --test particle_inspection_contract --test solver_provider_contract -- --include-ignored`:
  **41 passed, 0 failed, 0 ignored** across six selected targets.
- `cargo clippy --locked --no-default-features --all-targets -- -D warnings` and
  `cargo clippy --locked --all-targets -- -D warnings`: passed.
- `cargo fmt --all -- --check`, `git diff --check`: passed.
- `cargo run --release --locked --no-default-features --example material_session`:
  64 CSV particle rows; parsed and checked evaluation time precedes accepted time.
- `cargo run --release --locked --no-default-features --example pressure_session`:
  numerical Jacobi selection; qualified residual L2 2.75275252640750624e-10.
- Same pressure example with `/tmp/rheon-api-model/modal.bin`: real model selected,
  provenance exposed, qualified residual L2 6.10153409474846344e-12, no fallback.
- Existing `ai_pressure` example with that export and a fresh /tmp comparison path:
  36 real method/case/repetition invocations; independent `verify_ai_pressure.py`
  checkpoint replay passed. `RHEON_AI_CHECKPOINT` and `RHEON_AI_RUN` supplied to
  `python3 -m unittest discover -s tools -p 'test_ai_pressure_tools.py'`: all 5 passed,
  none skipped (an earlier input-free invocation correctly skipped four replay tests).

Counts are separate overlapping feature configurations, not additive unique-test
counts. The earlier transport milestone's **329 core-only / 337 default** counts
also meant passes in two separate configurations, each zero failed/ignored. Final
stack numbers above supersede them for this composed source revision.

Independent reviewer reran real-artifact release library/material/pressure tests:
**24 passed, 0 failed, 0 ignored** (17 unit + 3 material + 4 pressure contracts).
No blocking findings. Added deterministic final-callback deadline coverage at the
reviewer's suggestion. An initial high-frequency real-model request failed its
quality gate then exhausted the 3000-iteration reference budget. Retained it as a
negative fixture proving typed failure and prior-snapshot preservation; used a
nonzero smooth mode for positive real-model qualification. No thresholds changed.

No Godot binding/runtime, GPU, desktop native-window/clipboard, ARM/WebAssembly,
full trajectories using AI pressure, or target-device timing is claimed. The
provider replay is correctness evidence, not a general speedup or accuracy claim.
The task-created worktree remains retained-protected for draft review, reachable
from the published feature branch; parent owns its later integration/disposition.

## 2026-10-13 — moving-plate contract review and hosted verification

Read PR #52 `fd281ebf51d63b5406f86a74cdc67873ea9a0174` contract and actual experiment
source without merging it. Added M5 as a separately gated session-exposure milestone:
layered fluid/wall speeds, existing interval reports, fixed plate modes/axes, atomic
preflighted clock/generation and admitted Linux x86_64 arithmetic. Current API
capabilities remain unchanged. Normal pressure, torque, rotation/contact and arbitrary
geometry are not inferred from this tangential planar model.

Monitoring hosted PR #53 checks to terminal. Initial source-head runs confirm native
AI checkpoint qualification, executable contracts and dense3D scoped checks passed;
feature matrix and remaining jobs were still running when this follow-up began.
Terminal evidence will be reported with exact run IDs and final head status.

Hosted research-book run `38383894838` failed after the fresh aligned-strain
qualification passed: the educational packet refused `changed qualified native
source Cargo.lock`. The packet pins historical `f75cd66`; this branch's Cargo.lock
is identical to starting main `2209012`. This is a pre-existing publication-source
pin mismatch, not an API numerical failure. Do not relax its qualification gate
or replace historical evidence to make an unrelated API change pass. The current
crate-root additions also exceed that packet's explicitly pinned dense3D-only
extension. A separate education/source-qualification update is required.
