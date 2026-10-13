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

Broad `cargo test --locked --no-default-features` and `cargo test --locked` remain
Verifying at draft creation: existing long material-scene tests are still running.
They are not reported as passes until complete. They began before the final private
clock-test refactor; final focused tests and strict lint cover the changed code.

Independent review drove added checkpoint rollback, accepted-dt and deadline
coverage. The sleep-based deadline test was replaced after standards/review feedback.
No hardware performance measurements, Godot runtime, desktop rendering, GPU,
AI provider execution or downstream Tuldok consumer adoption are claimed.
