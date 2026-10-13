# CPU pressure pass measurements — rejected candidate

## Final disposition

The production optimization and its bespoke tests were reverted after expanded
controls exposed another adverse particle workload. The final `src/` and `tests/`
trees equal baseline 2209012. **No production speedup is accepted.** This PR now
retains the benchmark tools, standards plan and complete rejected-experiment
evidence. See [follow-up](particle-follow-up.md). All candidate timings below
refer to the frozen, rejected d03ac6e binaries, not the final reverted tree.
The historical verification status below also refers to d03ac6e; final checks
are recorded separately in revert-checks.json and the PR exact-head CI.

## Scope and provenance

Baseline: Rheon main `2209012bc5f902693d0f695ec1c2d854126af317`.
Standards snapshot: MrScripty/coding-standards
`188beda1fa477d21d576c233dd8c7f4c4c267d23`.
The candidate fuses pressure/residual updates and the ordered compensated norm
only with an active work observer and one Rayon worker. Other paths retain the
original loop at its call site. No sum reorder, tolerance change, warm start,
extra buffer, pressure implementation ID or particle/boundary change.

See environment.txt and artifact-hashes.txt. This Linux cloud container exposes
AMD EPYC 9V74 CPUs with a four-CPU quota and 16 GiB memory limit; Rust 1.92.0,
locked dependencies, release optimization, default target CPU, no fast math.
CPU measurements cannot establish device, renderer or GPU performance.

The pressure harness extends PR46's CPU step/capture example at `e245bcbd` with
anisotropic dimensions, iterations/residual reports, per-step times, observer
mode and phase output. It measures complete CPU simulation steps plus owned
field copying and consumption. It does not measure desktop density downsampling,
PNG encoding, rendering, worker startup or initial allocation. First-step timing
is cold solver execution within an initialized worker pool. Remaining samples
measure sustained steps in the same simulation; no reset between steps.

Each pair uses identical dimensions, emitter, dt=0.02, step count, pressure
settings and physical horizon. Every frame and pressure/physical report is
fingerprinted, not only the terminal state. Relative/absolute residual limits
remain 1e-9/1e-12, predicted divergence 1e-7, actual divergence 1e-5, maximum
iterations 2000. Grids use 8 steps except 64³'s 3 steps in the short matrix;
longer trajectories are reported separately. These are equal-horizon pairs,
not comparisons with the older 1024-step profile or kernel-only figures.

## Measurement and acceptance method

Run baseline/candidate in alternating AB/BA/AB order without concurrent builds
or CPU workloads. Retain all samples, median and min/max, plus individual step
samples. Three trials are bounded experimental evidence, not a population
confidence interval. A single pair cannot establish a timing improvement.
Require exact report/field fingerprints, iterations, accepted time and managed
memory for every pair. Existing independent analytic/refinement and recovery
tests supply physical/failure conformance; agreement alone is not that oracle.

Accept a narrow timing gain when consistent pair improvements and distributions
support it. Do not generalize noisy controls or claim every workload improves.
The fused Reduction phase includes the vector update; baseline serial vector
updates had no stage timer. Phase totals therefore cannot be compared as if
only reduction arithmetic changed. Full CPU-frame timing decides acceptance.

The particle control uses dense water fill with 512/4096/32768 particles, five
steps at dt=0.001, workers 1/2/4, and identical capture consumption. Particle code
is unchanged. Sparse emitted-particle, collision-mesh and new turbulence-control
performance is outside this measurement; no particle optimization is claimed.

## Reproduction

Use two independent source roots and target directories. Sharing a Cargo target
between same-name package checkouts produced stale baseline linkage during an
experiment; that set was detected by symbol inspection and discarded.

```sh
mkdir -p /tmp/rheon-cpu-baseline /tmp/rheon-cpu-rejected
git archive 2209012bc5f902693d0f695ec1c2d854126af317 | tar -x -C /tmp/rheon-cpu-baseline
git archive d03ac6e30de5a52839878b1da53905e9009966b8 | tar -x -C /tmp/rheon-cpu-rejected
cp /tmp/rheon-cpu-rejected/examples/cpu_pressure_lab.rs /tmp/rheon-cpu-rejected/examples/cpu_particle_lab.rs /tmp/rheon-cpu-baseline/examples/
CARGO_TARGET_DIR=/tmp/rheon-cpu-baseline-target cargo build --manifest-path /tmp/rheon-cpu-baseline/Cargo.toml --locked --release --features work-observation --example cpu_pressure_lab --example cpu_particle_lab
CARGO_TARGET_DIR=/tmp/rheon-cpu-candidate-target cargo build --manifest-path /tmp/rheon-cpu-rejected/Cargo.toml --locked --release --features work-observation --example cpu_pressure_lab --example cpu_particle_lab
python3 docs/plans/cpu-pressure/reports/measure.py /tmp/rheon-cpu-baseline-target/release/examples/cpu_pressure_lab /tmp/rheon-cpu-candidate-target/release/examples/cpu_pressure_lab /tmp/observed.json observed
python3 docs/plans/cpu-pressure/reports/measure.py /tmp/rheon-cpu-baseline-target/release/examples/cpu_pressure_lab /tmp/rheon-cpu-candidate-target/release/examples/cpu_pressure_lab /tmp/unobserved.json --controls
python3 docs/plans/cpu-pressure/reports/measure.py /tmp/rheon-cpu-baseline-target/release/examples/cpu_pressure_lab /tmp/rheon-cpu-candidate-target/release/examples/cpu_pressure_lab /tmp/sustained.json observed --sustained
python3 docs/plans/cpu-pressure/reports/measure.py /tmp/rheon-cpu-baseline-target/release/examples/cpu_pressure_lab /tmp/rheon-cpu-candidate-target/release/examples/cpu_pressure_lab /tmp/two-worker.json observed --two-worker-control
python3 docs/plans/cpu-pressure/reports/measure_particles.py /tmp/rheon-cpu-baseline-target/release/examples/cpu_particle_lab /tmp/rheon-cpu-candidate-target/release/examples/cpu_particle_lab /tmp/particles.json
```

## Rejected and deferred decisions

- Compensation operand-selection rewrite: no stable win; removed. See
  compensation-experiment.json.
- Fusion for unobserved execution: ambiguous/slightly regressive whole-frame
  results; removed. See fusion-experiment.json and frame-samples.json. The
  preliminary observed benefit in observed-fusion-experiment.json did not
  authorize retaining the unobserved path.
- Extracting both fused and original loops into one helper: isolated 64³
  two-worker regression despite single-worker gain; rejected. See
  observed-wrapper-experiment.json. The final fallback remains at its original
  call site.
- Warm start: a successful pressure solve is not accepted simulation state.
  Requires transactional commit ownership, structural operator invalidation,
  gauge and retry tests across simulation/domain owners. No unsafe cache added.
- Stronger preconditioning: benchmark existing SGS before introducing another
  implementation. Iteration reduction alone is insufficient; no new
  preconditioner justified in this slice.
- Particle locality: existing linked bins retained. Identity/summation changes
  require coordinated particle/boundary ownership and their own measurements.
- Worker/grain policy: preserve adaptive batching; noisy workload-dependent
  multiworker results do not justify a new global threshold.
- Preview/export: separate CPU capture measurement; no quality reduction and no
  unsupported rendering/PNG speedup claim. No measured priority for a new
  export pipeline in this pressure slice.

## Final measurements and gates

Detailed raw files retain all samples. Timings below are seconds per complete
run, not seconds per frame. Negative changes are slower candidate medians.
Ranges expose substantial scheduling variability, particularly with two workers.

### Observed short matrix

| Grid / solver | Workers | Baseline median [min, max] | Candidate median [min, max] | Median change |
|---|---:|---:|---:|---:|
| 16 / jacobi-pcg-v1 | 1 | 0.1728 [0.1723, 0.1761] | 0.1767 [0.1694, 0.1780] | -2.2% |
| 16 / jacobi-pcg-v1 | 2 | 0.1696 [0.1696, 0.1742] | 0.1764 [0.1757, 0.1906] | -4.0% |
| 16 / jacobi-pcg-v1 | 4 | 0.1751 [0.1701, 0.1753] | 0.1715 [0.1686, 0.1808] | +2.1% |
| 32 / jacobi-pcg-v1 | 1 | 2.5794 [2.5114, 2.6055] | 2.3635 [2.2842, 2.5266] | +8.4% |
| 32 / jacobi-pcg-v1 | 2 | 2.0726 [2.0197, 2.1452] | 2.2785 [1.9115, 2.6231] | -9.9% |
| 32 / jacobi-pcg-v1 | 4 | 1.9362 [1.8883, 1.9967] | 1.8734 [1.7281, 1.9133] | +3.2% |
| 64 / jacobi-pcg-v1 | 1 | 14.7632 [14.5026, 15.0660] | 14.3127 [13.5934, 14.3329] | +3.1% |
| 64 / jacobi-pcg-v1 | 2 | 10.8908 [10.7504, 10.8991] | 10.4623 [10.1012, 11.1387] | +3.9% |
| 64 / jacobi-pcg-v1 | 4 | 7.8630 [7.6264, 7.8917] | 8.0399 [7.7790, 8.0707] | -2.2% |
| 16x32x8 / jacobi-pcg-v1 | 1 | 0.2611 [0.2412, 0.2646] | 0.2559 [0.2334, 0.2699] | +2.0% |
| 16x32x8 / jacobi-pcg-v1 | 2 | 0.2389 [0.2356, 0.2646] | 0.2465 [0.2461, 0.3197] | -3.2% |
| 16x32x8 / jacobi-pcg-v1 | 4 | 0.2465 [0.2381, 0.2636] | 0.2534 [0.2390, 0.2624] | -2.8% |
| 32 / sgs-pcg-v1 | 1 | 1.8572 [1.8344, 1.9178] | 1.8623 [1.7723, 1.8662] | -0.3% |
| 32 / sgs-pcg-v1 | 4 | 1.1317 [1.0743, 1.5609] | 1.1870 [1.1043, 1.2400] | -4.9% |

Raw evidence: [observed-final.json](observed-final.json).

### Unobserved controls

| Grid / solver | Workers | Baseline median [min, max] | Candidate median [min, max] | Median change |
|---|---:|---:|---:|---:|
| 16 / jacobi-pcg-v1 | 1 | 0.1387 [0.1376, 0.1553] | 0.1426 [0.1355, 0.1431] | -2.8% |
| 32 / jacobi-pcg-v1 | 1 | 2.0311 [1.9926, 2.1528] | 2.0527 [2.0062, 2.1393] | -1.1% |
| 64 / jacobi-pcg-v1 | 1 | 11.6766 [11.5588, 11.9336] | 11.5167 [11.0674, 11.6465] | +1.4% |
| 16x32x8 / jacobi-pcg-v1 | 1 | 0.1945 [0.1933, 0.2112] | 0.2108 [0.1950, 0.2153] | -8.4% |
| 32 / jacobi-pcg-v1 | 4 | 2.0225 [1.6431, 2.0501] | 1.9102 [1.7686, 2.1637] | +5.6% |

Raw evidence: [unobserved-final.json](unobserved-final.json).

### Sustained observed single-worker runs

| Grid / solver | Workers | Baseline median [min, max] | Candidate median [min, max] | Median change |
|---|---:|---:|---:|---:|
| 16 / jacobi-pcg-v1 | 1 | 23.0551 [23.0551, 23.0551] | 21.7028 [21.7028, 21.7028] | +5.9% |
| 32 / jacobi-pcg-v1 | 1 | 20.3907 [20.3868, 20.5522] | 19.4496 [19.3382, 19.7097] | +4.6% |

Raw evidence: [sustained-final.json](sustained-final.json).

### Dense particle controls

| Particles | Workers | Baseline median [min, max] | Candidate median [min, max] |
|---|---:|---:|---:|
| 512 | 1 | 0.0082 [0.0078, 0.0084] | 0.0081 [0.0079, 0.0083] |
| 512 | 2 | 0.0047 [0.0046, 0.0047] | 0.0045 [0.0045, 0.0048] |
| 512 | 4 | 0.0034 [0.0031, 0.0036] | 0.0039 [0.0034, 0.0039] |
| 4096 | 1 | 0.0834 [0.0810, 0.0871] | 0.0916 [0.0900, 0.0974] |
| 4096 | 2 | 0.0474 [0.0471, 0.0535] | 0.0473 [0.0459, 0.0488] |
| 4096 | 4 | 0.0319 [0.0268, 0.0326] | 0.0272 [0.0269, 0.0287] |
| 32768 | 1 | 0.7760 [0.7383, 0.7796] | 0.7874 [0.7173, 0.7940] |
| 32768 | 2 | 0.4183 [0.4110, 0.4305] | 0.4210 [0.4106, 0.4297] |
| 32768 | 4 | 0.2296 [0.2209, 0.2301] | 0.2359 [0.2283, 0.2502] |

Raw evidence: [particle-final.json](particle-final.json). No particle speedup
claim: the production particle implementation is unchanged.

### Follow-up two-worker control

Six longer 32³ / 32-step observed Jacobi pairs: baseline median
8.3749 s [7.7562, 8.5519], candidate
8.2946 s [7.5109, 8.6573]. Four pairs improved and two
regressed; this longer control did not reproduce a consistent slowdown.
It does not support a multiworker speedup claim. This follow-up was selected because
the short matrix's two-worker median was adverse. All observations are retained
in [two-worker-control.json](two-worker-control.json).

## Historical pressure gain — candidate subsequently rejected

The rejected d03ac6e candidate fused observed single-worker execution. In the three 32³ / 64-step
pairs, complete CPU-frame medians fell **20.3907 -> 19.4496 seconds (4.6%)**;
every pair improved and the ranges did not overlap. The 64³ short matrix fell
14.7632 -> 14.3127 seconds (3.1%), again with all pairs improved. The one 16³ /
1024-step pair is trajectory evidence and a timing observation, not a replicated
performance claim. No global speedup, real-time budget or device guarantee.

Every final paired run matched fields/report fingerprints, iteration counts,
accepted time, divergence/residual metrics and managed bytes. Particle controls
matched their state fingerprints, particle counts, horizon and managed bytes.
The core solver, unobserved path and multiworker fallback retain ordered
arithmetic and original tolerances. The 4096-particle single-worker control is
an **unresolved adverse control**: all three candidate runs were slower,
with a 9.7% median slowdown and nonoverlapping ranges. Unchanged particle source
does not establish unchanged binary performance. Attribution is unresolved;
this prevents a regression-free or general performance acceptance claim and
was a draft blocker subsequently resolved by rejecting/reverting the candidate. Other overlapping
control distributions do not establish improvements or equivalence in timing.

Managed simulation / capture bytes by grid:

- 16: 333824 / 68608.
- 32: 2646016 / 536576.
- 64: 21069824 / 4243456.
- 16x32x8: 334848 / 69120.

Per-process peak RSS is recorded in each final pressure sample using Linux
wait4; it is separate from managed allocation accounting. CPU field capture is
small in these runs; this does not characterize rendering or PNG export.

## Verification status

Independent source review: passed (review.md). Formatting: passed. The complete
local debug work-observation suite passed 342 tests (debug-attempt.json). Full
release suites passed with work-observation, default and no-default features.
Strict all-target Clippy passed for all three configurations. The release CLI
built and Python executable/oracle tests passed (98 tests, one existing skip).
See checks.json for commands and results. Exact-head
CI is required after draft PR publication; the book binding gate remains intact.
