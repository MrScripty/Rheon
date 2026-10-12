# Preserved pressure approaches and native comparison milestone

## Scope and accepted design

The baseline is `6ba41c7cd05d1bf1851bef02048e7634245468e6`. The existing
fixed-box smoke solver remains the simulation owner. A stable
`PressureImplementation` registry now selects its pressure preconditioner:

- `jacobi-pcg-v1`: the original gauge-fixed Jacobi PCG implementation.
- `sgs-pcg-v1`: the same PCG solve with a symmetric Gauss–Seidel preconditioner.

The default remains Jacobi. `Simulation::with_implementation` selects a method
once for the lifetime of a simulation; `Simulation::new` retains the old
behavior. Changing a UI selection creates a new run from rest. It never
reinterprets an accepted state or modifies a completed result. IDs identify
real methods, not aliases for whichever method is newest. Preserve these
methods when adding future approaches; meaningful algorithm changes need a new
ID and equal-input evidence. This registry owns pressure approach identity only,
not the wider physics model, workload, hardware, render model or accuracy policy.

A bounded native desktop feature uses Rust egui/eframe directly over the existing
core. A Python/Tk prototype was rejected as a production runtime and is not part
of the repository. Python remains optional research/benchmark tooling. No new
cross-process production protocol, HTTP listener, interpreter or persistent
settings store is introduced. These decisions implement the requested selector
without committing to a complete interactive 3D editor.

Write set: pressure registry/preconditioner and tests; immutable simulation
selection; CLI selection and manifest; shared raw guidance projection; optional
native desktop and its worker lifecycle; comparison harness, evidence, CI and
this milestone documentation. Research book and original evidence stay intact.

## Algorithm boundary

On the gauge-eliminated symmetric positive-definite pressure matrix
A = D + L + Lᵀ, SGS applies M⁻¹ with
M = (D + L) D⁻¹ (D + L)ᵀ. One forward triangular solve followed by diagonal
scaling and one reverse solve uses the existing preconditioned-residual buffer.
The gauge value stays zero. The matrix weights and diagonal remain owned by
`PressureOperator`; no independently computed stencil is introduced.

Both methods share operator, RHS compatibility, gauge, true full residual
(including the eliminated row), predicted divergence, actual corrected binary32
divergence, Courant and transactional publication gates. Six pressure arrays and
the simulation payload remain unchanged. SGS is not multigrid and does not
change transport, boundaries, source physics or the concentration renderer.
Iteration reduction is not automatically a wall-time improvement because SGS
costs more work per iteration and introduces triangular ordering.

## Native UI

```
cargo run --locked --release --features desktop --bin rheon-desktop
```

The default desktop scene uses material water in a 1 m vacuum domain with Earth
gravity. See [material controls and model limits](material-fluids.md).
The following comparison controls apply to the original carrier mode.

The implementation dropdown lists the core registry. Grid edge, step count,
standard/tight pressure accuracy and CPU workers are explicit. Simulate executes
one method; Compare both solvers runs them sequentially from identical initial
conditions. The unit box and requested dt 0.02 s match the headless demo.
The default source is now a horizontal two-ended cylinder with independent,
nonuniform pressure-head profiles, rotating around Y at one revolution per
simulated second. Disabling that emitter restores the original smoke source,
which turns off halfway through each run.

The 3D viewer renders immutable previews of accepted concentration after every
step, while the next step calculates. The timeline supports scrubbing, playback,
single-step navigation, live following and separate solver tracks. Per-step
calculation time, accepted simulated time, pressure iterations and actual
divergence accompany each frame. Left or middle drag orbits, Shift+drag pans and
Ctrl+drag zooms. No Alt modifier is required on Linux. Wheel zoom, native
two-finger scroll/pinch and an override for touchpads reporting wheel events
are available. See [navigation and ownership](../src/bin/desktop/README.md).

The desktop owns one joinable worker and one immutable request. A repeated Start
cannot replace active work; editing controls are disabled while running. Cancel
propagates through the solver checkpoints. Close requests cancellation and keeps
polling until the worker ends before closing. Accepted preview frames appear
before join and remain available after cancellation or failure. Completed
summary rows become visible only after joining success; cancellation requested
before final publication suppresses summary rows.
A final Drop cancellation/join backstop prevents detached simulation work.
Results remain in memory until the next accepted run or window close. Use the CLI
or benchmark harness for persisted PNG/CSV/JSON evidence. Calculation remains
CPU based and the desktop makes no real-time speed guarantee.

Single-run UI timing excludes projection/graphics and has no warmup or variability
qualification. It is explicitly exploratory. Standard accuracy sets relative
residual 1e-9 and predicted divergence 1e-7; tight uses 1e-11 and 1e-9. Both use
absolute residual 1e-12, actual divergence limit 1e-5 and 2000 iterations. These
are algebraic/incompressibility controls, not physical ground-truth errors.

The carrier GUI has a 192 MiB retained-array budget to admit the selectable
128³ domain.
The headless CLI keeps its existing configurable 64 MiB default. Preview history
has a separate, adjustable planned payload budget, defaulting to 96 MiB and
allowing up to 4,096 MiB. Material preview requests reach 512³; the sidebar
reports actual stored resolution and planned bytes before startup. Long runs
reduce preview resolution to retain every accepted step within that budget.
Renderer, textures, observer markers, process/runtime and allocator memory are
outside the numerical array budgets. The GUI releases each simulation before
constructing the next method. Orange worker boxes are sampled real kernel
coordinates, not an animation or historical replay; idle positions are dimmed.
Their optional atomic diagnostics add overhead and can be disabled before a run.

## Dependencies and supported evidence

### CPU execution

`desktop` includes the optional `parallel` feature. Its CPU threads control
detects the logical CPU threads available to this process, displays the detected
limit and defaults to Automatic using that count. Disable Automatic to select
1 through the detected limit. Availability is refreshed before each run;
automatic selection follows the new count and manual selection is clamped to
it. If detection is unavailable, the limit is one. Changing the control affects
the next run; results display the captured thread count.
The headless CLI defaults to one thread and accepts `--threads N`; counts greater
than one require `--features parallel`. Zero and counts above the available CPU
count are rejected before output creation. The schema-2 completion manifest
adds `cpu_threads` alongside the unchanged physics and solver fields.

Independent pressure-operator rows, Jacobi preconditioning, PCG vector updates
and velocity/tracer advection use disjoint mutable scratch slices. The original
solver IDs, equations, tolerances and acceptance gates remain unchanged. SGS's
forward/reverse triangular sweeps use dependency-preserving tile waves on
larger grids. Compensated floating-point sums and diagnostics retain their
serial evaluation order. The contract is exact replay across thread counts;
the real simulation tests compare all state bits, iterations, diagnostics and
retained payload with one, two and four threads.

SGS partitions the grid into 4³ or 8³ tiles. On the first threaded solve it
compares scratch-only sweeps at both sizes, alternating their order, and retains
the faster measured size for that workspace and worker count. Four trial sweeps
add startup work but preserve every value and reported solver iteration. It
retunes when the worker count changes. A forward tile wave at bx+by+bz=w reads
only previously completed neighboring tiles and earlier cells within its own
tile. Reverse waves and cells traverse the opposite order. Each wave joins
before the next starts, preserving the original lower/upper triangular systems
and each row's floating-point evaluation order. The SGS implementation ID stays
`sgs-pcg-v1`: this changes execution order only between independent rows.

At 32,768 or more cells, SGS replaces its existing preconditioned f64 buffer
with AtomicU64 storage of the exact binary64 bits, allowing race-free shared
reads and disjoint writes without unsafe code or an additional cell buffer.
The six-array 48N-byte pressure budget is unchanged. Relaxed bit loads/stores
are ordered by the wave joins; floating-point arithmetic itself stays local.
Targets without 64-bit atomics retain serial SGS sweeps. Smaller grids and
grids with no independent tile frontier stay sequential; a single-thread pool
uses contiguous lexicographic sweeps. Constant full-box coefficients come from
the pressure operator and skip unnecessary wet/air queries, preserving its
diagonal and neighbor summation order. Free-surface modes retain their original
operator queries.

Wavefront width changes through the sweep, so it cannot keep every hardware
thread busy at every instant. Barriers, sequential compensated reductions,
memory bandwidth and other applications also limit scaling. Automatic chooses
the available count rather than benchmarking an optimal count; smaller manual
counts can be faster for small grids or a busy machine.

Carrier arrays below 16,384 values stay sequential. Larger pressure, vector, advection
and emitter passes use contiguous chunks that can split within a Z plane.
This fixes large-plane advection exposing only one task despite a large pool.
Each desktop/CLI run owns `CpuScheduler` feedback, layered over Rayon's work
stealing. It compares preferred chunks with approximately one, two and four
tasks per worker, bounded to 64–16,384 cells per chunk. Eight successful calls
sample these candidates twice; subsequent calls reuse the fastest joined
kernel wall time. It repeats the trials every 128 calls and resets them when
length or worker count changes. Timings include task scheduling overhead;
partial failures do not train a successful plan. This is runtime tuning over
a bounded set, not a promise of a global optimum or maximum CPU utilization.

Tracer transport computes exact nonzero donor bounds and expands them by the
maximum possible RK2 displacement plus interpolation/rounding support. Outside
those bounds it clears destination scratch without backtracing or interpolating.
It uses no display threshold or additional dense array. Invalid inputs still
fail validation, and unbounded departure arithmetic retains dense evaluation
and its original errors. The carrier velocity and pressure still cover the box:
clear concentration is not an empty fluid cell or an atmospheric pressure boundary.
Advection joins bounded bands of at least 16,384 values (or one larger Z slice)
before checking cancellation again. Cancellation callbacks run only on the
calling thread and still accept non-Send closures. Parallel bands check their
slice callbacks before computing the band; disposable scratch can differ on
failure, while accepted fields, time and generation remain intact.

Each desktop comparison worker and threaded CLI invocation owns a scoped Rayon
pool. Kernels join before their consumer continues; `build_scoped` also joins
the pool threads before the invocation returns, including cancellation and
failure. The UI continues polling the owning comparison worker during close.
Library consumers enabling `parallel` can call the existing APIs inside their
own Rayon pool's `install`; outside a pool, Rayon owns its process-wide default
pool. Those consumers get hardware/workload chunk sizing automatically; they
may bind `CpuScheduler::scope` around each scoped pool thread's `run()` to enable
run-local timing feedback. Core-only/default library builds retain sequential
execution and the exact zero-tracer optimization.

Rayon 1.11.0 (MIT/Apache-2.0, Rust 1.80 minimum) owns reusable work scheduling;
the Rust 1.92 build consumes the graph pinned in Cargo.lock. Spawning fresh
standard-library threads for every pressure iteration would repeatedly pay
thread creation costs, while a custom reusable scheduler would add lifecycle
and scheduling machinery. Rayon fits borrowed disjoint slices and scoped
ownership without adding a UI or asynchronous runtime dependency to the core.
The numerical Vec payload is unchanged; pool stacks, scheduler allocations
and process overhead remain outside the retained-array memory budget.

Verification lives in `tests/parallel_contract.rs`, `tests/cli_contract.rs`, the
SGS preconditioner differential test (unequal/partial tiles, gauge, all slab
orientations, degenerate grids and repeated reuse) and the desktop hardware
selection and worker lifecycle tests. See [local timing evidence](../evidence/cpu-threading/README.md)
for the measured effect and reproduction settings.
See [adaptive scheduling measurements](../evidence/scheduling/README.md) for
the large-plane regression, current measurements and tuning limits.

Optional `desktop` enables exact eframe 0.33.3 (MIT/Apache-2.0), default fonts,
Glow, X11, Wayland and AccessKit. eframe owns native windowing, input, rendering
and accessibility integration; the solver does not depend on it. Glow avoids an
unneeded second graphics backend. No persistence or inspection listener feature
is enabled. The selected version builds under the project's official Rust1.92
pin; actual graph versions and checksums are retained in Cargo.lock. The current
0.36.2 package declares Rust1.95 (verified with cargo info), so it is not silently
adopted under the project's Rust1.92 toolchain contract.

Candidates considered: native egui (direct Rust core integration, chosen), Godot
shell (additional framework/process ownership), and Python/Tk (new interpreter
in production, rejected). The desktop feature adds substantial build/runtime
platform dependencies, which is why it is opt-in. Default and core-only builds
are independently checked. No Windows/macOS executable qualification is implied
by Linux compilation. Platform screen-reader behavior and a complete advisory
audit remain unqualified; AccessKit being enabled is not proof of conformance.

References: https://docs.rs/eframe/0.33.3/eframe/ and
https://docs.rs/eframe/0.33.3/eframe/trait.App.html.

## Reproducible comparisons

```
cargo build --locked --release --bin rheon
python3 tools/rheon_compare.py --output comparison-16-standard --size 16 --steps 12 --repeats 3
python3 tools/rheon_compare.py --output comparison-16-tight --size 16 --steps 12 --repeats 3 --accuracy tight
```

The executable registry is authoritative. The harness requires a fresh output
directory, runs a retained warmup for each method, rotates sequential method
order across repeats, and writes `comparison.json` only on complete success.
Cancel/failure retains per-run diagnostics and `status.json`, never a completed
comparison. Reports preserve resolution, dt, source-off point, tolerance values,
iteration budget, executable SHA256, CPU/OS/affinity/cgroup facts and all individual
run manifests. Median/min/max reflect measured samples only. CPU details describe
the host, not an independently verified exclusive CPU allocation.

The harness explicitly passes dt 0.02, source-off step, method, selected pressure
and actual-divergence limits, 2000 iterations and 64 MiB to every child. It checks
schema 2, model identity and every workload/accuracy/budget setting, including the
CLI's fixed absolute residual 1e-12. Missing fields, changed settings and wrong
JSON types fail qualification. The summary records these controls.

Every accepted step must be present in order with finite nonnegative diagnostics,
positive dt no greater than requested, cumulative accepted time, bounded pressure
iterations, actual divergence and Courant number. Predicted divergence uses the
CLI's integrated full residual multiplied by dt and divided by cell volume;
there is no new or relaxed tolerance. Final time/divergence must agree with the
manifest. Retained payloads must fit the declared array and export budgets.
Opacity PNG qualification checks grayscale dimensions, chunk checksums and a
complete container; it does not decode pixels or certify image/state agreement.

Malformed or incomplete packets become terminal failures after the child is
reaped and its log closed. No later job starts and no completed summary is
published. Summary write/rename or completion-status write failure enters terminal
failed state and clears the owned success result before attempting summary cleanup
or failure-status persistence. Primary, cleanup and persistence errors remain in
the job error; repeated polls stay failed and perform no further publication work.
Run diagnostics remain intact. If cleanup fails, comparison.json can remain; if
failure-status persistence also fails, status.json can be stale. The caller must
honor the job's terminal status/error. This is recoverable I/O failure handling,
not a durable transaction across two files or protection against filesystem loss
or process interruption. Other lifecycle status writes retain their existing
I/O error behavior.

An optional `--run-timeout SECONDS` sets a positive finite wall-clock deadline
for each measurement process, including each warmup. The default is disabled;
choose a limit appropriate to the workload, for example `--run-timeout 120`.
The Python API exposes the same option as `run_timeout`, default `None`.
The limit includes startup, stepping, CSV and PNG export; it is separate from the
reported stepping timer and never changes a solver stopping tolerance. It applies
only to this benchmark harness, not the production CLI or native desktop.

When polling observes a live child past its deadline, the harness enters
`timing_out`, sends TERM, and sends KILL after a two-second grace period if the
child still runs. It waits for child exit, closes the log, and saves terminal
`timed_out` status with an error and the configured deadline. No subsequent job
starts and no `comparison.json` is published; partial diagnostics remain.
Explicit user cancellation remains `cancelling` then `cancelled`; the first stop
reason is retained. A timeout makes the CLI return exit code 1. API callers must
continue polling through `running`, `cancelling`, or `timing_out` to completion.
Enforcement depends on polling (the CLI polls every 50 ms) and process termination
by the OS; the deadline is not a hard bound on an unresponsive kernel wait.

CLI `--build-info` exposes version/platform/debug-assertion facts; the harness
rejects a debug-assertion-enabled executable and records those build facts. This
does not prove arbitrary custom optimization flags; retain the release build log.
CLI `--implementation`, `--relative-residual`, `--pressure-divergence-limit`,
`--actual-divergence-limit` and `--max-iterations` make the tested controls explicit.
An unknown implementation fails before creating output. `run.json` schema2 adds
method and accuracy metadata while retaining original fields. CLI stepping time
includes CSV writes; GUI time does not. Do not mix those timing columns. The
harness marks unequal accepted physical horizons and does not equate divergence
with full simulation error. Multiple resolutions are separate workloads, not a
convergence study unless their physical horizon and reference solution are also
controlled. Run this same locked binary/source contract on another hardware
host before making cross-device claims.

## Verification claims

- Original-v1 16³/20-step PNG and every CSV diagnostic reproduce the committed
  baseline byte-for-byte.
- An independently hand-assembled 2×2×1 triangular-system oracle qualifies the
  SGS action; anisotropic manufactured pressure recovers modulo gauge.
- Both methods pass full-residual, incompatible RHS, zero/single-cell,
  cancellation and equal-input state agreement checks.
- Every cancellation stage preserves accepted bits/time/generation; retry replays
  uninterrupted output for both implementations.
- Real CLI selection, unknown-ID rejection and original default replay are tested.
- Native worker tests cover equal-input result images, pre-cancellation and
  repeated Start exclusion. Pixel/window interaction requires separate actual
  native evidence and is not inferred from those tests.
- Python harness tests exercise real executable success, output preservation,
  cancellation before/during subprocess work and invalid selection.

Final local verification: 48 default tests, 43 core-only tests, 52 desktop-feature
tests, three real-executable Python harness tests, formatting, all three strict
Clippy feature configurations and the release build pass. The original final
release reproduces baseline PNG and every CSV diagnostic byte-for-byte.

Actual cloud Linux desktop interaction verified selection, selected-method
execution, completed two-method comparison with image/table output, repeated
Start exclusion, cancellation without partial rows, restart and close during
active work. Process exit was independently checked. Those interactions used
the debug binary; their timing is excluded from performance claims.

See `evidence/comparison/checkpoint.json` and its linked raw run manifests and
per-step diagnostics. Final 16³/12-step tests use one retained warmup plus three
measured samples per method for standard and tight tolerances. The tight run
uses the actual 1e-11/1e-9 controls: Jacobi totals 1701 pressure iterations versus
SGS 678, both retain 333824 numerical-array bytes and both have maximum actual
divergence approximately 1.04e-7. Binary32 correction limits that divergence even
when the algebraic pressure tolerance is tightened; do not report it as a
physical accuracy improvement.

Larger preliminary 32³ and 64³ measurements retain their inputs, hardware and
variability in the receipt. On 64³/3 steps, pressure iterations are 1645 versus
596, with identical 21069824-byte retained payload. Those measurements may
include concurrent cloud build load. The final 16³ pair was serialized after
local compilation drained, but the shared host is not exclusive hardware. No
controlled speedup, cross-device support or real-time acceptance is claimed.
Hosted exact-head CI remains a publication-stage gate.
