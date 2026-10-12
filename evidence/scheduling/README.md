# Adaptive CPU batches and zero-tracer work

The requested outcome is less wasted calculation and better worker use in the
live cylinder demo. The change fixes an observed advection starvation defect,
adds run-owned batch tuning, and skips provably zero tracer interpolation while
preserving the current carrier physics and numerical replay contract.

## Reproduction and cause

The old advection path joined 16,384-value bands, then split those bands into
whole Z planes. At 128³, a plane itself has 16,384 cells, so each band contained
one task in a 24-worker pool. At 64³ it exposed only four plane tasks.
`advection::scheduling_tests::a_large_plane_exposes_parallel_work_instead_of_one_task_per_band`
failed on the old implementation with `large planes starve the worker pool`.
Its real band callback observes simultaneous work; the repaired path passes.

`examples/scheduling_lab.rs` also measured the old 128³ tracer kernel using only
one worker. The new kernel splits within planes. Pressure/Jacobi row kernels,
vector updates and emitter passes use the same bounded chunk policy.

## Owned decisions and contracts

`src/scheduling.rs` owns batch feedback and its diagnostics, layered over the
existing Rayon work-stealing scheduler. It owns no numerical fields, scene
state, simulation clock, thread creation or cancellation policy. Desktop and
CLI composition roots create one `CpuScheduler` per run and bind it around
every scoped thread's `run()`. Thread-local scopes restore prior bindings on
return or unwind; the scoped pool joins before the owner returns. Library
consumers may use the same binding, or get hardware/workload sizing without
timing feedback in an ordinary pool.

Kernel callers supply their stage, scratch length and preferred grain. The
module compares that grain against approximately one, two and four jobs per
worker, with 64–16,384 cells per batch. It measures the complete joined pass,
including scheduler overhead, once per invocation. Each shape samples the four
candidates twice, then uses the fastest observed candidate. Trials restart
every 128 successful calls or when length/worker count changes. Failed partial
passes do not train a successful plan. Short mutex sections hold only metadata;
callbacks execute outside them. Snapshots are independent diagnostics rather
than CPU-utilization measurements or a consistent whole-simulation snapshot.
The policy searches a bounded set and makes no global-optimum guarantee.

Advection owns the exact support proof: a convex interpolated velocity stays
within each component's donor magnitude. Expand exact nonzero tracer bounds by
the maximum RK2 displacement and two cells of interpolation/rounding support.
Outside that region, destination scratch becomes zero without backtracing or
sampling. Source data, tiny/subnormal concentrations and acceptance gates stay
intact. Unbounded departure arithmetic keeps dense evaluation and its errors.
No field-sized metadata or extra numerical array is allocated. A bounding box
may include gaps between separate occupied regions; this is not a sparse tree.

SGS owns its dependency graph and tile selection. It enumerates only ready
tiles rather than scheduling columns that do no work. At the first threaded
solve, four scratch-only trials compare 8³ and 4³ sweeps in alternating order;
the faster observed size is retained for that workspace/worker count. Forward
and reverse sweeps overwrite scratch in topological order, preserving the
original per-row arithmetic. The trial work contributes to step wall time,
but does not alter the reported PCG iteration count or accepted fields. A
worker-count change triggers calibration again. Wave joins and ordered scalar
reductions remain necessary, so some workers can be idle.

An initial attempt to force smaller tiles whenever the frontier was narrower
than the pool raised CPU use but slowed the 32³/24-worker cylinder workload by
about 12%. That policy was rejected. Recorded kernel timing, not busy-thread
count, now decides both linear batch and SGS tile choices.

The desktop retains batch diagnostics after completion and records exact
nonzero tracer cells before preview quantization. Clear concentration remains
carrier fluid: its velocity and pressure can influence the visible jets. Using
the display as a pressure-domain mask would change boundary conditions and
the physical model. A liquid-only sparse/free-surface solver is separate work.

## Follow-up: unchanged 32³ / 1,024-step timing

The first batch changes were insufficient for the reported long smoke run.
For its first 64 steps, whole-kernel wall timers attributed about 24% of total
step time to pressure matrix application, 23% to Jacobi preconditioning,
26% to ordered reductions and 20% to pressure-vector updates. Advection was
only about 2%. Disabling worker markers changed the prefix from 15.88 s to
14.27 s; marker overhead did not explain most of the runtime.

The fixed-box operator now uses constant neighbor weights and flat neighbor
strides directly. Arbitrary batches traverse contiguous X-row fragments,
computing coordinates once per fragment rather than dividing for every cell.
Jacobi reuses a 64-entry boundary-neighbor diagonal descriptor (512 bytes),
computed in the original lower/upper X/Y/Z addition order. Free-surface and
column operators retain their existing face-dependent coefficients. The
fourteen numerical arrays, arithmetic order, pressure tolerances, stopping
gates, solver IDs and transactional behavior are preserved.

The batch sidebar now shows cumulative kernel wall time and shares, including
ordered scalar reductions. Shares exclude validation, preview capture and
other unrecorded work; they are not CPU-utilization readings. A batch is a
sequence of voxels on one worker, while other workers execute other batches.
Candidate sizes test complete-pass throughput, including join overhead;
they do not evaluate several voxels simultaneously inside a worker.

This smoke model starts with incompressible carrier fluid throughout the box.
The source adds momentum and appearance concentration, not liquid mass. Its
pressure projection couples the connected domain, including clear cells;
it does not accumulate an absolute uniform pressure from added liquid volume.
Exact-zero tracer skipping can save early transport work, but tiny nonzero
donors count as occupied and transport spreads them over the long run.

The same 32³ / 1,024-step smoke inputs were then run from rest on both frozen
release builds, with 24 workers, worker markers enabled, the source disabled
at step 512, requested dt 0.02 s, and tight Jacobi PCG tolerances (relative
1e-11, absolute 1e-12, predicted divergence 1e-9, actual divergence 1e-5).
Pool startup, rendering and history capture are excluded; whole calculation
includes all steps and scheduler calibration. One full run per build on this
non-exclusive workstation is evidence for this case, not a general guarantee.
No builds or tests ran concurrently with these measurements.

| Entire 1,024-step run / cumulative kernel | Before pressure changes | After |
| --- | ---: | ---: |
| Total simulation calculation | 232.908241 s | 170.238748 s |
| Pressure matrix application | 53.530309 s | 20.784397 s |
| Jacobi preconditioning | 52.017379 s | 24.707763 s |
| Pressure-vector updates | 45.692385 s | 45.142847 s |
| Ordered reductions | 63.718176 s | 61.180635 s |

Total calculation time decreased 26.91% (1.368× throughput). Both numerical
field fingerprints were `1e7747174fd861e5`; all pressure-related invocation
counts matched. The remaining major costs are ordered reductions and small
vector passes, so full pool utilization is not a throughput guarantee.
Raw inputs, profiles and progress remain in the local generated file
`evidence/scheduling/smoke-tight-pressure.json`, which is not tracked. The
measured executable identities are retained here:

- Before: `338603d871fb389288db671799cf588c88a771b1d359a760dd26914f8a53d80b`
- After: `e4b0b1061fa0ebcd05bde51a002a4a99cce4ac5b6c0796c4fd9c55947c12ff7b`

Reproduce using the retained before/after lab executables with arguments
`32 24 1024 smoke-tight`.

## Measured effect

Linux workstation, Intel Core Ultra 9 275HX, CPU affinity 0–23, Rust 1.92.0,
locked release builds. Other desktop work was running; this is non-exclusive
local evidence. Three repetitions alternate baseline/candidate order. Both
builds enable actual worker observation, matching the desktop's default. Pool
startup, allocation, rendering and history capture are outside reported kernel
time. Tracer kernels get one warmup; velocity kernels do not. Whole-step
cylinder cases start from rest and include scheduler/SGS calibration.

| Workload | Workers | Previous median | Current median | Previous/current |
| --- | ---: | ---: | ---: | ---: |
| 128³ dense tracer, 2 passes | 24 | 0.962538 s | 0.206394 s | 4.66× |
| 128³ dense-case velocity, 2 passes | 24 | 2.939826 s | 0.541466 s | 5.43× |
| 128³ localized tracer, 2 passes | 24 | 0.969330 s | 0.021787 s | 44.49× |
| 32³ cylinder SGS, 5 accepted steps | 8 | 0.490484 s | 0.456053 s | 1.08× |
| 32³ cylinder SGS, 5 accepted steps | 24 | 0.677720 s | 0.640027 s | 1.06× |
| 64³ cylinder SGS, 2 accepted steps | 24 | 2.476962 s | 2.480867 s | 1.00× |

All measured numerical-field fingerprints matched. Dense tracer observed all
24 workers instead of one. Across the dense tracer/velocity process workload,
CPU time divided by whole-process wall time rose from approximately 1.00 to
14.15 average cores. That process metric includes startup and warmup and is
not a stage-specific utilization figure. The large advection gains do not
translate directly into whole-step speedups: pressure still dominates these
short cylinder cases. The 64³ SGS result shows no meaningful improvement in
this sample. Small speedups require more controlled qualification before any
general performance promise.

Raw samples, ranges, fingerprints, CPU accounting and machine/build identities:
`/tmp/rheon-scheduling-measure-f71waozr/measurements.json`. Earlier rejected
policy measurements are at `/tmp/rheon-scheduling-measure-vx0bi957/measurements.json`
and `/tmp/rheon-scheduling-measure-kdqgnfhb/measurements.json`.

Frozen release lab executable SHA-256:

- Baseline: `ab4e81fa42191df3f7ff1a9cfa6a0244e568555e8308cc25fb5f19591dca2448`
- Current: `32fefadd566b0d651894b95917374bafccb2b534d5c7d215b32d6786f27535c1`

To reproduce, retain the old lab binary before rebuilding:

```sh
cargo build --locked --release --features work-observation --example scheduling_lab
python3 evidence/scheduling/measure.py --baseline /path/to/old/scheduling_lab --candidate target/release/examples/scheduling_lab
```

The lab runs actual advection APIs on constant carrier velocity with either
uniform tracer or a 4³ localized pulse, and actual `step_with_emitter` calls
for the SGS cylinder cases. Fingerprints establish agreement with the frozen
baseline; contract tests separately compare individual field/report bits.

## Verification and standards

340 full release desktop tests and 26 focused core-only tests passed after the
pressure follow-up, including
partial tiles, anisotropic/nonzero-origin grids, gauge/free surfaces, repeated
reuse, thread-count replay, scope cleanup, invalid inputs, cancellation and
retry. Dense RK2 reference evaluation matches skipped-region output bit for
bit, including negative and subnormal donors and boundary clamping. The
occupancy test distinguishes tiny numerical values from quantized previews.
Adaptive feedback tests verify choosing the fastest trial and resetting after
a workload change. Worker ownership/lifecycle and Linux navigation tests remain
applicable. No new dependency or numerical array was introduced. Anisotropic
boundary and singleton-axis fixtures verify the bounded diagonal cache's
original floating-point addition order. Partial-row fixtures cover arbitrary
flat batches; the full 1,024-step comparison retains the baseline fingerprint.

Guidance was retrieved through the Standards MCP's complete route, with no
unresolved routing facts. The selected owners included Implementation,
Verification, Performance, Concurrency, Architecture, Diagnostics, Library and
Rust API/Tooling. Repository standards Markdown was not used as authority after
the user's instruction to use the MCP.

Rayon remains responsible for scheduling mechanics; see its
[parallel slice API](https://docs.rs/rayon/1.11.0/rayon/slice/trait.ParallelSliceMut.html).
The physical distinction between carrier flow and a liquid/air pressure domain
is documented in the existing [simulation milestone](../../docs/MILESTONE.md)
and [live viewer contract](../../src/bin/desktop/README.md).
