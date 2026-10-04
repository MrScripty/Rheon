# Fixed-box smoke and tracer milestone

Historical baseline at 6ba41c7. See [the later comparison milestone](COMPARISON.md)
for selectable approaches and the optional native desktop.

Rheon now has a working headless Rust executable and reusable CPU simulation
core. It implements a fixed orthogonal Cartesian MAC box, impermeable normal
wall velocities, midpoint semi-Lagrangian velocity/tracer transport, localized
prescribed Y acceleration, saturating passive concentration injection, and
matrix-free gauge-fixed Jacobi PCG pressure projection. The source acceleration
is not a temperature-derived buoyancy law. Density is a positive constant.

No viscosity, moving solids, cut cells, liquid interfaces, particles, GUI,
GPU path, multigrid preconditioner, Zig or Odin port is implemented. This is a
bounded first solver milestone, not the complete Rheon project.

## Build and run

Rust 1.92.0 and the committed Cargo.lock are required. The numerical kernels use
the standard library. PNG encoding uses image-rs `png` pinned to 0.18.1 with
locked transitive dependencies (https://docs.rs/png/0.18.1/png/). The default
`png-export` feature owns both image export and the CLI. Library consumers can
use `default-features = false` for a standard-library-only simulation dependency
graph; no export function or CLI is then exposed. Both configurations are tested.
The established PNG codec was selected rather than implementing a compressed
image format locally. It is Apache-2.0/MIT licensed, compatible with the existing
Apache-2.0 repository. Full advisory auditing has not yet been qualified.

```
cargo test --locked -j 1
cargo fmt --check
cargo clippy --locked --all-targets -j 1 -- -D warnings
cargo run --release --locked -j 1 --bin rheon -- --help
cargo run --release --locked -j 1 --bin rheon -- \
  --size 16 --steps 100 --source-off-at 50 --output rheon-demo-16
```

The output directory must not exist. The CLI never overwrites an existing run.
`steps.csv` records every accepted step, actual time increment, pressure
iterations, full residual, actual binary32 velocity divergence, Courant number,
concentration integral and discrete kinetic energy. `opacity.png` is an
orthographic +Z concentration-integral guidance map, with +Y upward, black
background and white opacity. Its fixed coefficient is 8 inverse concentration
length units: opacity = 1 - exp(-8 integral concentration dz). It is not physical
scattering, depth, normals or a liquid renderer. `run.json` is written last as
the completion manifest. Failed runs may leave partial diagnostics without that
manifest. No timestamps are embedded in the PNG, making replay bytes testable.

Defaults are size 64 cubed, 30 steps, requested dt 0.02 seconds and a 64 MiB
managed simulation array limit. These defaults are configurable and are not
validated real-time performance promises. The source occupies the stated
normalized unit-box region in the CLI; use the library for explicit geometry
and source configuration. A one-cell grid is valid but the demo source may
contain no sample locations.

## Numerical and transaction contracts

`Simulation` owns named x/y/z/tracer fields in an accepted state and a separate
candidate. Read-only `StateView` exposes accepted slices. Low-level operator,
pressure and advection functions still take positional arrays in exactly
[X, Y, Z] order: length checks cannot detect swapping equally shaped components
on a cubic grid. The state facade constructs these views centrally and offers
no mutable accepted-field access.

Each step validates configuration/source/time, selects a conservative dt from
old velocity and prescribed acceleration bounds, advects candidate velocity,
applies the source force, solves pressure, corrects candidate velocity in place,
checks full actual divergence and Courant number, transports/injects tracer,
checks finite/range diagnostics, and swaps only after the final cancellation
checkpoint. dt can be smaller than requested and is always reported. The
post-projection Courant check may reject a step; no hidden retry or tolerance
relaxation occurs. The dt policy is not a general stability/convergence proof.

All errors and all seven cancellation stages preserve accepted array bits,
time and generation. Retry after cancellation matches the uninterrupted run.
Pause rejects steps without advancing. Reset zeroes both field sets and time,
then advances generation; it reuses allocations. Scratch is disposable and may
be partially modified on rejection. Source concentration is saturated at one;
semi-Lagrangian interpolation is bounded but does not conserve total mass.

Pressure compatibility checks the unshifted full RHS. The solver fixes one gauge
cell, uses an on-the-fly Jacobi diagonal, and accepts only a recomputed full
ungauged residual including the gauge row. A rejected candidate residual check
restarts directions rather than silently trusting recursive residuals. The
actual corrected binary32 field is independently differentiated on every cell
and checked separately before publication. Zero RHS and single-cell boxes exit
without dividing by zero. Breakdown, nonfinite arithmetic, incompatible RHS,
iteration exhaustion and cancellation are typed failures.

The book's checked Lean algebraic statements motivate invariants. They do not
verify this Rust program, finite indexing, floating-point evaluation, pressure
convergence, boundary sampling, or performance. Geometry uses a conservative
implementation admission policy: half-spacing must be normal, half-offset
indices representable, and h/axis_scale at least 32 binary64 epsilons. Some
usable grids are deliberately rejected rather than admitting collapsed adjacent
samples. The exact 2^52+3 duplicate-center counterexample and large-origin
regressions require no field allocation. This policy is not a new Lean theorem.

## Complete retained simulation allocation inventory

There are exactly fourteen owned numerical Vec buffers:

- Accepted x/y/z face components and cell tracer: four f32 arrays.
- Candidate x/y/z face components and cell tracer: four f32 arrays.
- Pressure, RHS, residual, direction, operator product, preconditioned residual:
  six f64 cell arrays.

The product array is reused for post-projection divergence. No stored diagonal,
extra pressure RHS, hidden diagnostic cell array, or third velocity buffer is
allocated. Constructors use fallible allocation, inspect actual capacities and
check the remaining payload budget before retaining each field/workspace.
Stepping, interpolation, transport and diagnostics allocate no heap buffers in
this implementation; user-supplied cancellation callbacks are outside that
claim. Descriptors, iterator state, settings and scalar reports are stack/struct
values. The retained array payload is 8F + 56N, where F counts all component
faces and N is the number of cells.

At 64 cubed: N=262144, F=798720, retained payload=21069824 bytes (20.09375 MiB).
The limit covers retained Vec capacities, not allocator metadata, transient
allocation behavior, executable/runtime memory, stacks or process RSS. Export
has a separately checked raw-pixel limit of 16 MiB; PNG compression workspace,
CLI strings, CSV/JSON/file buffering and output encoding are outside the
simulation limit. No whole-process hard memory cap is claimed.

## Executed acceptance and measured demonstration

42 tests cover geometry/spacing/overflow rejection, exact layout, independently
assembled rectangular pressure operators, hand-calculated chains, units, full
gauge-row stopping, manufactured 3D pressure, actual f32 divergence and energy,
zero-RHS circulation, cancellation, component offsets, convex donor bounds,
translation/rotation tracing, pulse transport, rest, transactional failures,
replay, pause/reset, source toggling, allocation limits and real CLI export/readback.
Formatting and Clippy with warnings denied also pass on Rust 1.92.

Local release runs on 2026-10-03:

- 16 cubed, 20 steps, source disabled after step 12: 0.248 seconds measured
  stepping time; 333824 retained simulation bytes; final maximum divergence
  1.49e-7 inverse seconds.
- 16 cubed, 100 steps, source disabled after step 50: 1.260 seconds measured
  stepping time; nonzero bounded opacity image independently decoded with Pillow
  and visually inspected.
- 64 cubed, three steps, source disabled after step 2: 7.403 seconds measured
  stepping time, 21069824 retained bytes, Linux child peak RSS 22464 KiB.
  Pressure iterations were 585, 585 and 550; final maximum divergence 1.42e-7.

These are single-host demonstration measurements, not broad benchmarks. The
64-cubed run is substantially slower than real time. Stronger preconditioning,
operator optimization, profiling, GUI execution and sustained performance
qualification remain future milestones. The original low-resolution real-time
product objective remains open.
