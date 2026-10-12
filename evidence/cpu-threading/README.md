# CPU threading measurements

The desktop now parallelizes independent pressure and advection grid work using
a scoped CPU pool. These measurements compare the original release executable
with the threaded release executable on the user's Linux workstation (Intel
Core Ultra 9 275HX, 24 available CPUs, Rust 1.92.0). Other desktop and development
workloads were running; these are local measurements rather than an exclusive
hardware or real-time qualification.

Each sample advances two steps from rest with the Jacobi PCG implementation,
dt 0.02 s, standard pressure accuracy, and the smoke source enabled only for
the first step. Three repetitions rotate the order of the original, current
one-thread, four-thread and eight-thread executables. There is no separate
warmup run. Timing uses the existing CLI `measured_step_seconds`, which includes
stepping and buffered CSV writes and excludes pool startup and PNG export.

| Grid | Original, 1 thread | Current, 1 thread | 4 threads | 8 threads | Original / 8 threads |
| --- | ---: | ---: | ---: | ---: | ---: |
| 32³ | 0.941 s | 1.088 s | 0.379 s | 0.314 s | 3.00× |
| 64³ | 15.561 s | 15.808 s | 5.992 s | 4.672 s | 3.33× |

Values are medians. The raw local capture is
`/tmp/rheon-thread-benchmark-2ot37j8r/summary.json`; generated JSON observations
remain outside tracked repository inputs. The individual samples are recorded
below. Every sample's entire CSV and PNG matched the original bytes.
The retained numerical array payload remains 21,069,824 bytes at 64³. Thread
stacks and scheduler resources remain outside that existing payload measure.
One-thread timings varied, including a 22.463 s 64³ sample; a claim about serial
performance or exclusive hardware speed requires a more controlled environment.
This table captures the initial threading implementation, before parallel SGS
sweeps. The following measurements cover the SGS update.


| Grid | Executable / threads | Sample 1 | Sample 2 | Sample 3 |
| --- | --- | ---: | ---: | ---: |
| 32³ | original | 0.935475 s | 0.941161 s | 0.955186 s |
| 32³ | threads-1 | 1.087761 s | 0.958013 s | 1.103307 s |
| 32³ | threads-4 | 0.378840 s | 0.399389 s | 0.354679 s |
| 32³ | threads-8 | 0.325335 s | 0.313676 s | 0.312243 s |
| 64³ | original | 15.127434 s | 15.855555 s | 15.561006 s |
| 64³ | threads-1 | 15.808280 s | 22.462604 s | 14.910445 s |
| 64³ | threads-4 | 5.991982 s | 6.360790 s | 5.694100 s |
| 64³ | threads-8 | 5.155768 s | 4.672047 s | 4.204335 s |

Measured executable SHA-256:

- Original: `e2821c556d9b3d25f5b625578ce5950c5ddf13578be9e520b86431bbdb5f1f1f`
- Threaded: `396ccc19e2dbe98b44f89081cd1af55c198fe601f5f4dbd78b2411af14e85594`

The process CPU affinity allowed CPUs 0–23. Kernel: 7.0.0-31-generic.

To repeat, retain the original release `rheon` executable before rebuilding the
candidate and pass both executables to the captured measurement procedure:

```sh
cargo build --locked --release --features desktop --bins
python3 evidence/cpu-threading/measure.py --baseline /path/to/original/rheon --candidate target/release/rheon
```

The procedure creates a fresh temporary output directory, prints each timing,
asserts equal CSV/PNG bytes, and saves `summary.json` after each completed grid.
It requires at least eight available CPUs. A single representative CLI call is:

```sh
target/release/rheon --size 64 --steps 2 --source-off-at 1 --threads 8 --output /tmp/rheon-threaded-new-directory
```

Behavior and lifecycle verification use the real simulation and worker paths:

```sh
cargo test --locked --release --features desktop
cargo test --locked --release --test cli_contract --test implementation_contract --test advection_contract --test operator_contract --test pressure_contract --test simulation_contract
cargo check --locked --no-default-features
cargo clippy --locked --features desktop --all-targets -- -D warnings
```

`tests/parallel_contract.rs` checks unequal grid dimensions and partial tasks,
exact state and diagnostic parity for both solvers across one/two/four threads,
cancellation after work has begun, unchanged accepted time/generation, recovery
after cancellation, and non-Send cancellation closures confined to their caller.
The desktop's existing worker lifecycle tests cover cancellation, suppression
of unfinished results, restart and close/drop ownership.

## Parallel SGS sweeps and automatic hardware selection

The desktop now detects the available logical CPU threads, defaults to
Automatic using that count (24 on this workstation), and refreshes the limit
before each run. Manual selection stays available within the detected limit.

SGS now executes its forward/reverse sweeps as waves of independent 8³ tiles.
Its constant fixed-box coefficients skip wet/air queries. The solver ID,
per-row arithmetic, compensated reductions, acceptance gates and retained
array payload are unchanged. Every measured CSV and PNG matched the previous
SGS executable byte for byte.

The following samples use the same two-step standard-accuracy workload as
above, three repetitions with rotating order, and the same non-exclusive
workstation. Baseline counts refer to the previous implementation with serial
SGS sweeps; candidate counts refer to the completed SGS tile implementation.
Times are medians. Average CPU cores use child process user+system CPU time
divided by whole-process wall time, including startup/export; this is distinct
from the stepping-only timing metric and does not imply useful work on every
busy thread.

| Grid | CPU threads | Previous SGS | Parallel SGS | Speedup | Previous / new average CPU cores |
| --- | ---: | ---: | ---: | ---: | ---: |
| 32³ | 8 | 1.063 s | 0.335 s | 3.18× | 1.41 / 4.27 |
| 32³ | 24 | 1.201 s | 0.517 s | 2.32× | 2.06 / 7.96 |
| 64³ | 8 | 15.117 s | 3.627 s | 4.17× | 1.33 / 3.88 |
| 64³ | 24 | 15.500 s | 4.594 s | 3.37× | 1.52 / 5.99 |

Eight threads were faster than 24 for these workloads. Automatic honors the
detected CPU availability; it does not benchmark an optimal pool size. Narrow
wavefronts, barriers, serial reductions, memory traffic and other applications
still limit SGS scaling.

| Grid | Executable / threads | Sample 1 | Sample 2 | Sample 3 |
| --- | --- | ---: | ---: | ---: | ---: |
| 32³ | baseline-8 | 1.063464 s | 1.060280 s | 1.074572 s |
| 32³ | baseline-24 | 1.200996 s | 1.253115 s | 1.100402 s |
| 32³ | threads-8 | 0.323144 s | 0.372835 s | 0.334947 s |
| 32³ | threads-24 | 0.521197 s | 0.516624 s | 0.453286 s |
| 64³ | baseline-8 | 15.116535 s | 15.360636 s | 14.170188 s |
| 64³ | baseline-24 | 15.499908 s | 15.683423 s | 14.560056 s |
| 64³ | threads-8 | 4.090369 s | 3.616485 s | 3.627110 s |
| 64³ | threads-24 | 4.593735 s | 4.785224 s | 4.381886 s |

Measured executable SHA-256:

- Previous SGS: `f52a4643f36d80de24baeeabb3be9318aed0d1d282f425c905fd27c7e70b54a2`
- Parallel SGS: `9ea78a1ee8b0e0f090419953c36bbdcd5d034f76e0b0c8bc79542a78027a8b05`

Raw observations: `/tmp/rheon-thread-benchmark-fbdhca0w/summary.json`.
Reproduce with the executable retained before the SGS update:

```sh
python3 evidence/cpu-threading/measure.py --baseline /path/to/previous/rheon --candidate target/release/rheon --implementation sgs-pcg-v1 --baseline-threads 8 24 --threads 8 24
```

The SGS differential test also covers anisotropic/partial tiles, gauge and
wet/air boundaries on all three axes, one-dimensional and two-dimensional
degenerate grids, and reuse with different residuals. Whole-step tests cross
the SGS activation threshold, compare all state/diagnostic bits across thread
counts, and verify cancellation/retry. Hardware-selection tests exercise
automatic selection, manual bounds and changing detected limits.
