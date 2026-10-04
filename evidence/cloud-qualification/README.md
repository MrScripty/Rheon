# Independent cloud pressure qualification

Source under test: `0f2db35009624467a1f338700d8967a350443c93`, fetched from
`MrScripty/Rheon` branch `implementation/selectable-pressure-comparison`.
Verified ancestor: `6ba41c7cd05d1bf1851bef02048e7634245468e6`.
Additional qualification is isolated on `qualification/cloud-pressure-comparison`.
No merge, push, review request, credential change, or network-policy change was made.

The frozen head passed 48 default, 43 core-only, and 52 desktop tests, all three
strict Clippy configurations, formatting, and a locked release CLI build.
Three additional regression tests passed: final totals are 49 default, 44
core-only, and 55 desktop. Strict Clippy and formatting passed again. All three
real-executable Python comparison lifecycle tests passed. Explicit locked default,
core-only and desktop builds also passed (`final-build-*.log`). No production defect
was demonstrated and no production behavior or implementation ID was changed.

The added tests cover cancellation at every stage after a nonzero accepted step
for both implementations, preserving state bits/time/generation and exact retry;
native worker cancellation after progress followed by successful SGS restart;
and the App Drop cancellation/join backstop using an owned test worker. Existing
tests independently cover cancellation after worker completion but before result
publication, repeated Start exclusion, equal-input images and solver oracles.

The initial checkout was clean. `/workspace/.agents` and `/workspace/.codex`
were empty; no applicable AGENTS.md or SKILL.md files were present. Existing
repository Rust 1.92.0, formatting, feature, and strict-Clippy conventions apply.

## Reproduction

The environment's existing `/workspace/rheon-setup/env.sh` selects the official
pinned toolchain, an external Cargo target directory, and one build job.
Run these commands from the repository root:

```sh
source /workspace/rheon-setup/env.sh
cargo test --locked
cargo test --locked --no-default-features
cargo test --locked --features desktop
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --no-default-features --all-targets -- -D warnings
cargo clippy --locked --features desktop --all-targets -- -D warnings
cargo fmt --all --check
cargo build --locked --release --bin rheon
cargo build --locked
cargo build --locked --no-default-features
cargo build --locked --features desktop
```

The frozen-head results are in `test-{default,core,desktop}.log`,
`clippy-{default,core,desktop}.log`, `fmt.log`, and `release-build.log`.
Subsequent checks with additional regression tests use `final-*` logs.
The Python tests expect `target/release/rheon`; an ignored `target` symlink to
the configured external target directory supplies that path in this environment.

```sh
python3 -m unittest discover -s tools -p 'test_*.py' -v
python3 evidence/cloud-qualification/replay.py "$CARGO_TARGET_DIR/release/rheon" FRESH_REPLAY_DIR
python3 evidence/cloud-qualification/benchmark.py "$CARGO_TARGET_DIR/release/rheon" FRESH_BENCHMARK_DIR
python3 evidence/cloud-qualification/verify.py
```

The last command validates the committed evidence directories and source hashes;
it does not rerun timing measurements. It checks every retained run's requested
controls, finite diagnostics, residual/divergence/Courant gates, repeat bytes,
physical horizons and array budgets. `qualified-source-manifest.json` identifies
the source including the added tests; the benchmark receipt's Git HEAD identifies
the frozen parent because the evidence and test additions were not yet committed
when measurements ran. Production numerical code remained identical to that head.

Use fresh output directories. Builds and tests finish before benchmarking.
The original harness executes methods sequentially, retains warmups, and rotates
method order over three measured repeats. The wrapper records before/after
load, CPU statistics, CPU pressure, cgroup throttling and process snapshots.
Snapshots cannot exclude transient interference or contention on the physical host.

## Scope and limits

`frozen-source-check.json` verifies every frozen comparison source-manifest hash
and unchanged book, proof and original demo trees. Replay checks all three old
PNG/CSV fixture pairs byte-for-byte. Whole run manifests cannot match because
they include measured timings and the newer schema adds method/accuracy fields;
all old non-timing fields are checked for equality.

The compiler is official rustc 1.92.0, LLVM 21.1.3, x86_64 Linux. The release
profile is Cargo's standard optimized profile with no custom RUSTFLAGS or profile
overrides; `build-environment.json` and `environment.txt` record the environment.
Host-reported CPU is Intel Xeon Platinum 8370C, five visible logical CPUs,
cgroup quota four CPUs and memory limit 16 GiB. This is a shared VM, not an
exclusive hardware allocation. CLI timing includes stepping and CSV writes,
excludes startup and PNG export, and is not desktop rendering performance.
Retained numerical array bytes are not process RSS.

The environment has no configured display or Xvfb. Desktop tests exercise native
worker ownership and headless egui result publication, not actual window input,
graphics drivers, accessibility, or platform-specific close events. Existing
native-interaction claims are not repeated as new evidence. Book/Lean validation
is preserved but was not rerun; no new formal proof or cross-platform claim is made.

## Cloud measurements (2026-10-04 UTC)

All six comparisons completed. Each method has one retained warmup and three
measured samples; accepted times match within every comparison.

| Grid / steps / accuracy | Jacobi seconds median [min, max] | SGS seconds median [min, max] | Iterations Jacobi / SGS | Array bytes (both) |
| --- | --- | --- | --- | --- |
| 16³ / 12 / standard | 0.2449 [0.2199, 0.2558] | 0.1787 [0.1773, 0.1826] | 1644 / 654 | 333824 |
| 16³ / 12 / tight | 0.2187 [0.2183, 0.2194] | 0.1791 [0.1774, 0.1847] | 1701 / 678 | 333824 |
| 32³ / 6 / standard | 1.4684 [1.4432, 1.5200] | 1.2564 [1.1654, 1.3342] | 1689 / 645 | 2646016 |
| 32³ / 6 / tight | 1.5007 [1.4885, 1.5380] | 1.2043 [1.2026, 1.3164] | 1734 / 669 | 2646016 |
| 64³ / 3 / standard | 11.0235 [10.7211, 11.4777] | 7.6924 [7.6696, 8.5676] | 1645 / 596 | 21069824 |
| 64³ / 3 / tight | 13.3685 [11.2899, 13.4865] | 8.1569 [8.0022, 8.3577] | 1820 / 620 | 21069824 |

Largest recorded actual divergence across all cases: 1.1920929e-07, below the 1e-5 gate.
No cgroup throttled-time increase was recorded. Boundary snapshots show one-minute
load averages from 0.407 to 0.970. No concurrent build or test ran during timings.

The tight 16³ claim is independently reproduced: 1701 versus 678 iterations,
333824 bytes each. Tight tolerances do not imply better physical accuracy;
binary32 velocity correction remains an accuracy limit. The 64³ workloads use
three steps with source off after step one, as specified by the comparison harness,
whereas the historical 64³ replay uses source off after step two. Do not compare
their timings or iteration totals as if their forcing were identical.

All deterministic PNG/CSV bytes match across the four runs of each method/case.
All three historical demo fixture pairs also match byte-for-byte. Full run
manifests and per-step diagnostics are retained under `benchmarks/` and `replay/`.
No 128³, Windows/macOS, native-window or sustained real-time qualification is implied.

Test logs have only surplus terminal blank lines removed for Git whitespace checks.
