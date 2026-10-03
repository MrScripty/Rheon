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
