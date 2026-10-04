# Optimization-safe historical cloud evidence gates

Candidate branch: `repair/cloud-evidence-optimization`. Accepted base is PR3
`c9b5f85e5bebb7ab46cafd7d385235cfe9ed8672`, tree
`e783b2ec5fceadae8a277919b6dc46c05d585123`, whose ordered parents are
`e3d84d2469923377a7bd0d0d419b896423ebbe1a` and
`2c5c526812ec2b9f8e9e0f967846f4d0545c8a32` (accepted PR2).
No new book publication, solver feature or numerical implementation is imported.

## Four actionable findings

The source was checked against CodeRabbit review
https://github.com/MrScripty/Rheon/pull/4#pullrequestreview-5407335527,
run `b3a0f009-b039-401d-829c-2ab3d9cc6a60` (85 included paths / 184 excluded artifacts).
Review-only PR4 and PR5 are unchanged. The findings remain valid in the accepted
PR3 base; review text is evidence, not authorization to invoke another bot review.

- [4178611352](https://github.com/MrScripty/Rheon/pull/4#discussion_r4178611352):
  an unequal accepted-time result now writes a failed case with an explicit error
  and returns exit code 1. The persisted and printed case status is authoritative.
- [4178611356](https://github.com/MrScripty/Rheon/pull/4#discussion_r4178611356):
  the qualification README identifies historical checkout
  `7e1a76dd487549a496e04b9305fa531fcc7ab2f9`, parent
  `0f2db35009624467a1f338700d8967a350443c93`. The repaired verifier's
  `--source-root` explicitly points at those original source bytes. Expected
  hashes in the original manifest are unchanged; a combined source checkout
  deliberately fails historical qualification in every Python mode.
- [4178611361](https://github.com/MrScripty/Rheon/pull/4#discussion_r4178611361):
  all former assertion conditions in benchmark.py, verify.py and replay.py
  now use explicit failures. Source hashes, controls, finite diagnostics,
  horizons/budgets, deterministic bytes and stable manifest equality survive
  `-O` and `PYTHONOPTIMIZE=1`.
- [4178611366](https://github.com/MrScripty/Rheon/pull/4#discussion_r4178611366):
  every embedded manifest field must equal its retained run.json field with
  matching type. Embedded iteration totals and maximum divergence are derived
  from retained CSV. For each method, only repeats 0/1/2 enter median/min/max
  seconds, sample count, first measured array bytes/iteration total and maximum
  measured divergence. Both comparison and receipt summaries must equal the
  independently reconstructed summaries. The existing deterministic iteration
  and equal array-budget checks justify their first-measured-run definitions.

The original six comparison files, all 48 run manifests/PNG/CSV files, receipts,
source manifest and historical timings remain unchanged. The historical residual
product gate is retained; this does not relabel it as the later repaired harness's
cell-volume-scaled gate. No replacement measurements were manufactured.

## Separate cancellation nit disposition

Source commit `649b7aaf42c0dd02e3b71eba5403a7fafaac93bb` follows the evidence fix
commit `291e64aae33e2aae3b4788dd5ed85b3a0a370da0`. The old cancellation test could
observe positive progress after completion and then exercise only suppression.
A `cfg(test)`-only channel gate pauses the real worker after its first accepted
step. The test verifies progress is exactly 1, below the total, and the handle is
unfinished before setting the real cancellation flag. It releases the gate,
checks unfinished workload progress, lets App::poll join, and requires the actual
simulation cancellation error, not the already-completed-result suppression
status. Empty rows/textures and successful SGS restart remain checked.

Channel waits and lifecycle polling have 10-second deadlines. Synchronization
uses no timing sleep; the bounded workload is four steps per method instead of
10,000. All hooks/fields are excluded from non-test builds. Existing production
cancellation, join, row publication and numerical code are unchanged.

## Actual checks and reproduction

Use the pinned Rust 1.92.0 toolchain and locked dependencies, as in the existing
setup environment. `receipt.json` binds exact source commit/tree, hashes, commands
and logs. From the candidate repository root:

```sh
python3 -m unittest discover -s evidence/cloud-qualification -p test_gates.py -v
python3 -O -m unittest discover -s evidence/cloud-qualification -p test_gates.py -v
PYTHONOPTIMIZE=1 python3 -m unittest discover -s evidence/cloud-qualification -p test_gates.py -v
python3 evidence/cloud-qualification/verify.py --source-root /path/to/rheon-at-7e1a76dd
python3 -O evidence/cloud-qualification/verify.py --source-root /path/to/rheon-at-7e1a76dd
PYTHONOPTIMIZE=1 python3 evidence/cloud-qualification/verify.py --source-root /path/to/rheon-at-7e1a76dd
cargo test --locked
cargo test --locked --no-default-features
cargo test --locked --features desktop
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --no-default-features --all-targets -- -D warnings
cargo clippy --locked --features desktop --all-targets -- -D warnings
cargo fmt --all --check
cargo build --locked --release --bin rheon
python3 -m unittest discover -s tools -p 'test_*.py' -v
python3 evidence/cloud-qualification/replay.py target/release/rheon FRESH_NORMAL_REPLAY
python3 -O evidence/cloud-qualification/replay.py target/release/rheon FRESH_OPTIMIZED_REPLAY
```

All 14 gate test methods pass in each of the three Python modes. Mutation
subcases use disposable copies of the actual archived evidence and historical
Git source, preserving the originals. Wrong finite medians and every other
summary field, altered embedded values, nonfinite/untyped measurements,
controls, diagnostics, source, manifest and PNG/CSV corruption are rejected.
An altered warmup duration in both temporary copies remains excluded from the
summary. CLI wrong-median corruption exits 1; a fake completed comparison with
unequal horizons tests failure persistence/return without claiming a solver run.

All six unchanged historical comparisons pass in all three modes; the current
source is rejected in all three modes. Before-repair logs separately demonstrate
that the original verifier rejected combined source normally but reported success
under `-O`. Those logs are defect reproduction, not passing qualification.

Actual Rust totals are 50 default / 45 core-only / 56 desktop; strict Clippy passes
all three feature configurations. The targeted cancellation test executes one
test and passes. The existing comparison suite passes 20 tests. Fresh current-CLI
replay passes all three original demo fixture PNG/CSV pairs and old non-timing
manifest fields under normal and optimized Python. Its incidental step timings
are not a benchmark and do not replace or qualify historical performance values.

The environment is headless: no native-window/graphics-driver interaction,
cross-platform, new performance, Lean or formal-Rust guarantee is claimed. No
whole-book gate was rerun. Immutable review branches, PR3, PR2, main and review
threads remain untouched; no external review request was sent. Independent review
and coordinated integration are pending.

Only surplus terminal blank lines were removed from four Rust test logs for Git
whitespace checks; the receipt records original and retained hashes.
