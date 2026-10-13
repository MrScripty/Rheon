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
