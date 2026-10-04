# Comparison result qualification

Isolated branch: `implementation/comparison-result-qualification`, based on
reviewed comparison head `0de6a857adb17a6ee0312e889aff50e2cd7aa6dd`.
The checkout and repository instructions were inspected before edits. No issue
inventory exists; this repair closes the complete-success, matched-workload and
finite-diagnostic requirements already stated in `docs/COMPARISON.md`.

## Demonstrated failures and repair

The real unchanged release CLI produces each test packet; tests alter one field
or artifact after the child completes, before the harness accepts it. Baseline
metadata tests failed 16 subcases: changed/missing schema, forcing, absolute
residual, iteration and memory limits were accepted or not explicitly pinned.
The next diagnostic tests failed 49 subcases, including NaN/infinite/negative
diagnostics, step order/time/admission violations, inconsistent manifest
diagnostics/payloads and missing/truncated images. Separate summary write and
rename injections raised uncaught errors before the ownership repair. The
`*-before.log` files retain these failures rather than claiming the baseline
passed. `metadata-after.log` and `diagnostics-after.log` are intermediate checks.

The harness pins and checks the complete schema-2 workload. It qualifies every
step against existing numerical admission limits, using integrated residual
units (`residual * dt / cell_volume`), and checks final manifest agreement and
payload bounds. PNG checks cover container integrity and grayscale dimensions,
not pixel decoding or physical image accuracy. All rejection paths reap/close
the completed child, preserve diagnostics, stop further jobs and publish no
completed summary. Summary write/rename/completion-status failures clear owned
success and remove a published summary. Persistent filesystem failure while
saving failure status remains an I/O error; this is not crash-durable storage.

`final-tests.log`: all 18 Python tests pass. This includes the previous 8
lifecycle/deadline tests, new adversarial qualification/publication cases and
valid zero-source/single-cell and active-source runs for both methods at both
accuracy settings. The CI path filter now covers the new test file; its existing
test discovery runs it without installing dependencies.

## Reproduction and limits

From the repository root, after the locked release CLI is available:

```sh
python3 -m unittest discover -s tools -p 'test_*.py' -v
python3 evidence/cloud-qualification/benchmark.py target/release/rheon FRESH_OUTPUT
```

The release executable is reused, SHA256
`e10000f4fb61ba09765d7e9cbc76cac0463550cc7aecbcd72a66d458e0a1a584`.
Its compiler/profile receipt is the unchanged
`evidence/cloud-qualification/environment.txt` and `build-environment.json`:
official Rust 1.92.0, LLVM 21.1.3, default optimized Cargo release profile,
no RUSTFLAGS/profile overrides. No Rust/source/dependency change requires a new
binary. Rust tests, Clippy, native-window interaction and Lean/book builds are
not rerun or claimed as fresh evidence for this Python-only repair. Existing
49/44/55 Rust and hosted CI evidence remains unchanged on the reviewed branch.
No numerical implementation, stable ID, fixture, tolerance or book/proof artifact
is changed; no external review request or PR merge is made.

Serial standard/tight measurements and their source/hardware/contention receipt
are retained under `benchmarks/`. The unchanged archival benchmark wrapper loaded
the stricter harness from commit `d40bdd7a51637ac2005fd1b2a0e1c8232b945c7d`
(tree `9b3e31a90b89d34eb0205eabab4f0a97f0af170c`).
No exclusive-host, cross-device, desktop timing or real-time claim is implied.

Logs have trailing whitespace and surplus terminal blank lines removed only for Git whitespace checks.


## Current-harness measurements (2026-10-04 UTC)

One retained warmup and three measured samples per method, sequentially with
rotating order. Each case has an equal accepted physical horizon. Timing includes
accepted stepping and CSV writes, excluding startup and PNG export.

| Grid / steps / accuracy | Jacobi seconds median [min, max] | SGS seconds median [min, max] | Iterations Jacobi / SGS | Array bytes (both) |
| --- | --- | --- | --- | --- |
| 16³ / 12 / standard | 0.2109 [0.2109, 0.2111] | 0.1781 [0.1709, 0.1795] | 1644 / 654 | 333824 |
| 16³ / 12 / tight | 0.2142 [0.2126, 0.2157] | 0.1824 [0.1746, 0.1842] | 1701 / 678 | 333824 |
| 32³ / 6 / standard | 1.4746 [1.4709, 1.5405] | 1.1843 [1.1215, 1.2496] | 1689 / 645 | 2646016 |
| 32³ / 6 / tight | 1.4894 [1.4608, 1.5009] | 1.2015 [1.1366, 1.2196] | 1734 / 669 | 2646016 |
| 64³ / 3 / standard | 10.2275 [10.2161, 10.2371] | 7.6185 [7.5783, 7.6934] | 1645 / 596 | 21069824 |
| 64³ / 3 / tight | 11.3877 [11.2959, 11.4493] | 7.9481 [7.8273, 7.9558] | 1820 / 620 | 21069824 |

Host-reported Intel Xeon Platinum 8370C, Linux x86_64, five visible/affinity
CPUs, cgroup quota four CPUs and memory limit 16 GiB. No concurrent builds or
tests ran during timing. Normal control/editing work and shared physical-host
contention cannot be excluded. Before/after snapshots record load, CPU pressure,
CPU statistics, process CPU usage and cgroup throttling. There was no recorded
throttled-time increase; boundary one-minute load ranged 0.0205–0.9497.
This is descriptive single-host evidence, not a controlled speedup experiment.

`verification.log` and `verification.json` independently check all six cases,
all 48 runs' diagnostic gates and exact archived PNG/CSV bytes. Every non-timing
manifest field also matches the previous cloud measurements. Maximum predicted
divergence was 1.429882804e-09; maximum
actual divergence was 1.192092896e-07. No original
fixture or archived receipt was replaced. Preserved original demo replay evidence
remains valid because the executable hash and every production source/dependency
file are unchanged; those replay timings were not rerun.

```sh
python3 evidence/cloud-qualification/benchmark.py target/release/rheon evidence/result-qualification/benchmarks
python3 evidence/result-qualification/verify.py
```

The benchmark command above requires the retained output directory to be absent;
use another fresh directory for new measurements. The verification command reads
the committed evidence and performs no timing runs. It binds the benchmark's
exact source commit and source hashes while separately checking protected source,
book/proof and original evidence paths against the reviewed base.

## Next product decision

The documented fixed-box and selectable-pressure milestones are delivered, and
this demonstrated comparison qualification gap is closed. The open low-resolution
real-time objective needs a concrete target grid, accepted physical horizon,
accuracy setting and frame-time budget before performance profiling or a new
preconditioner can have a meaningful acceptance test. Native platform interaction
and accessibility also require an actual target platform; headless tests here do
not qualify them. No speculative solver/editor feature or repeated unchanged Rust
check was added to fill that decision gap.
