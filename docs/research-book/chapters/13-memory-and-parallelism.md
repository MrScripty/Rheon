# 13 Memory layout and parallel execution

A solver leaving no room for diffusion weights fails Rheon's intended role even if fast alone. The relevant quantity is peak simultaneous memory: persistent state, scratch, render resources, readback buffers and the host application. Persistent simulation arrays alone underestimate it.

## An explicit layout

For a cubic grid, face count is \(F=3n^2(n+1)\). Current and next velocity, eight cell-centered single-precision arrays and a one-byte mask require

\[
M(n)=4(2F+8n^3)+n^3\quad\text{bytes}.
\]

This calculated layout excludes allocator overhead, alignment, ghost layers, coefficients, meshes, particles, driver allocations and model weights. The eight scalars might be pressure, right-hand side, residual, search direction, matrix product, preconditioned residual and two tracer buffers. Assign actual lifetimes before reuse.

The formula gives 1.805 MiB at 32 cubed, 14.344 MiB at 64 cubed, 48.305 MiB at 96 cubed and 114.375 MiB at 128 cubed. Doubling every axis multiplies leading volume cost by eight. At 256 cubed this incomplete layout is about 913.5 MiB. Show estimates before allocation.

## Ownership and access

Separate velocity arrays naturally match staggered shapes. Particle structures of arrays aid per-field traversal, while tiled layouts may suit transfers reading several fields. Benchmark actual access patterns. Avoid abstractions that conceal shape or introduce unmeasured per-sample dispatch.

A shape owns counts, strides, spacing, origin and checked capacity. State owns persistent fields. Workspace owns scratch and alias rules. A solver borrows them during a step. This exposes reuse without a global pool whose lifecycle is unclear.

For a face joining a and b, the pressure operator contributes \(w(p_a-p_b)\) and its negative. Cell gathering writes one output without scatter races. Face scattering emphasizes paired conservation but requires atomics, coloring or local accumulation. Compare reproducibility and bandwidth.

## Parallel reductions

CG dot products are global reductions. Floating addition is nonassociative, so tree shape changes results. Fixed trees support repeatability on a pinned configuration; arbitrary atomics generally do not. Pairwise or compensated accumulation reduces some error at cost. Single-precision fields with double-precision reductions are a candidate to qualify, not a universally available GPU feature.

State whether replay is bitwise on one backend, tolerance-based across backends, or only configuration-reproducible. Save precision, solver tolerance, step policy and algorithm version when they affect the promise.

## Sparse grids

Sparse tiles save empty-space cost but add keys, occupancy maps, halos and fragmented work. Benefits depend on active fraction and stencil expansion. A moving plume may activate many tiles at once; reserve or reject before mutating accepted state. Disconnected sparse pressure regions need consistent component gauges.

Begin with dense bounded grids and honest peak reporting. Adopt sparsity when representative scenes show empty-space cost is the limiting resource and topology machinery has a clear owner.

## Three sizes that the desktop keeps separate

The current material scene has three independently meaningful sizes: its
physical domain in metres, its parcel sampling, and its stored preview grid.
Doubling the side of a filled cube at the same reference density multiplies
initial mass by eight. Raising preview resolution changes how accepted parcels
are displayed; it does not add numerical parcels or improve the fluid update.

The desktop stores an eight-bit preview and metadata for step zero and every
accepted step. A requested resolution may be reduced before the run so that
the whole planned history fits its budget. For orientation, one 128³ scalar
byte array is 2 MiB; 1,025 such arrays already occupy 2,050 MiB before metadata.
These calculated payload sizes exclude numerical state, meshes, thread stacks
and allocator overhead. The current sidebar reports the actual stored size
and planned bytes. Its history budget is not a process-memory cap.

The particle owner has a different budget. It reserves accepted and candidate
particles, per-particle density/pressure and force scratch, neighbor links and
bin heads. Capacity accounting uses the compiled element sizes and allocated
capacities, including padding. A full particle array must cause an explicit
failure rather than silently drop parcels or neighbors. See the
[material contract guide](../implementation/material-preview-contracts.md#memory-and-display-are-different-budgets)
and the [desktop ownership contract](../../../src/bin/desktop/README.md).

## What an adaptive CPU batch changes

Imagine 1,000 independent output entries divided into batches of 256. The four
jobs own intervals [0,256), [256,512), [512,768) and [768,1000). Every entry has
one writer; the final job simply has fewer entries. Four jobs do not mean four
workers will remain busy for the entire pass. Their costs may differ, and a
join waits for the slowest unfinished job.

Rheon's run-owned [`CpuScheduler`](../../../src/scheduling.rs) chooses bounded
batch sizes above Rayon's existing work-stealing scheduler. It times complete
joined passes and periodically compares a small set of candidates. It changes
work boundaries, while each item's arithmetic and the ordered reductions
retain their established order. Smaller jobs expose more parallel work but
also add scheduling overhead. The fastest observed candidate is evidence for
that workload and timing sample, not a universal best grain size.

The exploration counter also has a scope: for a stable workload with one
outstanding invocation and exactly one success record per pass, the first
eight successful calls visit four candidates twice, and the cycle repeats
after 128 calls. That conditional accounting does not guarantee exploration
under arbitrary overlapping calls, failures or changing workloads.

SGS sweeps also have dependencies. A tile may read completed predecessor tiles,
so only a ready wave can run together. In a 2 × 2 × 2 tile box, grouping by
\(b_x+b_y+b_z\) gives waves containing 1, 3, 3 and 1 tiles. Eight total jobs
therefore do not provide eight independent jobs at once. Rheon separately
calibrates its supported SGS tile sizes on scratch and keeps ordered wave
joins. The [scheduling contract guide](../implementation/cpu-scheduling-contracts.md)
separates interval coverage, dependency order and timing feedback.

To read the CPU inspector, compare complete-pass wall time before interpreting
worker boxes. Orange markers sample actual visits at the repaint rate, and
idle workers may retain their last position. Kernel shares include recorded
passes only; validation, preview capture and other unrecorded work are outside
that denominator. A busier display need not mean a faster accepted step.
The [scheduling evidence](../../../evidence/scheduling/README.md) retains exact
workloads, build identities and the limits of its historical measurements.

## Scheduling proof bridge

Finite-index and dependency statements can establish that a declared partition
covers its entries and that a predecessor lies in an earlier wave. They do not
prove that the OS runs every worker, that a timing choice is optimal, or that
floating reductions may be reordered safely. The current
[`CpuScheduling.lean`](../../../proofs/Rheon/CpuScheduling.lean) source and its
[implementation mapping](../implementation/cpu-scheduling-contracts.md) must be
read alongside actual cross-thread replay and scope-cleanup tests. A source
listing alone is not a fresh kernel qualification.
