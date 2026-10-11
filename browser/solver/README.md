# Bounded browser solver adapter

This crate calls the existing `Simulation::step_with_forces` unchanged. It has
two separately bounded fixtures. The first is the independently derived four-edge circulation in
`tests/forces_contract.rs`: a sealed 2×2×1 unit grid, density 1, rest initial
velocity, dt 0.25, and acceleration (2,0,0) supported on [0,2)×[0,1)×[0,1).
There is no tracer source, free surface, geometry change, or pressure setting
mutation. The timestep executes rest advection, force, native pressure solve,
projection, tracer transport, acceptance checks, and transactional commit.

For the four interior faces, the force gives `(1/2,0,0,0)` and the divergence-free
cycle is `c=(1,-1,-1,1)`. Orthogonal projection gives `c/8`. Expected work is
1/8, final kinetic energy 1/32, maximum Courant number 1/16, and divergence zero.
Force work describes its own stage before projection, not the final energy gain.
The two existing pressure implementations may take different iteration counts.
This numerical fixture does not extend a Lean checked claim to compiled code.

The wasm-bindgen ES module exports `CycleSimulation(methodId)` with the existing
IDs `jacobi-pcg-v1` and `sgs-pcg-v1`. It owns native memory and exports JSON copies:

- `snapshot()` returns a snapshot JSON string.
- `step(dt, cancelBeforeCommit)` returns an outcome JSON string. Only dt 0.25 is
  accepted. One successful step is allowed per reset. A cancellation probe can
  run before or after that step and reaches the native `BeforeCommit` checkpoint.
- `reset()` returns a snapshot JSON string. Native reset increments generation,
  zeroes arrays and time, and permits the next accepted step.
- `free()` is the generated wasm-bindgen destructor.

The predetermined cancellation callback is a transaction demonstration; it does
not interrupt synchronous WASM from another worker message. Multiple accepted
timesteps and long-time stability are outside this fixture's evidence.

Snapshots include full `x`, `y`, `z`, `tracer` float32 values and corresponding
`bits` arrays of uint32 values; `time`, hexadecimal `time_bits`, numeric
`generation`, exact decimal `generation_u64`, fixed `grid`, `method`, and
`accepted_since_reset`. Array indexing is x-fastest on each staggered lattice.
Outcomes contain `status`, typed `code`, optional `detail`, optional accepted
`report`, and the unchanged or committed `state`. The report copies the native
pressure, force, divergence, Courant, tracer integral, and energy diagnostics.

The native exporter runs both methods through baseline, invalid dt, cancellation
at rest, acceptance, cancellation with nonzero accepted fields, a blocked second
acceptance, reset, and acceptance after reset. It uses this same Rust adapter.
Contract tests additionally derive the circulation, divergence, and energies
independently and check bit-preserving rollback and retry.

## Bounded three-dimensional source pulse

`FlowSimulation(methodId)` uses the same existing pressure IDs and owns a separate
24³ unit-box simulation at density 1.2 kg/m³. `snapshot()` and `reset()` return
JSON snapshots; `step(cancelBeforeCommit)` attempts one native step with requested
dt 0.02 s. The actual native dt may be smaller. At most 80 steps can be accepted
per reset. The injection source is active for the first 30 accepted steps. A
localized force density (0.6, 1.2, −0.3) N/m³ continues throughout the run; exact
support bounds and acceptance settings are exported in every snapshot.

The full x/y/z staggered velocity arrays and cell tracer are computed in three
dimensions. The snapshot also contains accepted-step count, actual last dt, source
cutoff integral and independently checkable field diagnostics. It is a sealed,
constant-density appearance-tracer model. The transport is not conservative
liquid volume: the qualified bounded run loses roughly 16% of its tracer amount
after injection stops. There is no physical buoyancy, viscosity, free surface or
internal obstacle coupled to this timestep. The source wireframe is not a solid.

`flow-reference <fresh-output-directory>` writes `native-flow-reference.json`
and `native-flow-fields.bin`. Both methods have baseline, 80 accepted snapshots,
rest/nonzero cancellation, limit rejection, reset and retry records. The binary
stores exact little-endian f32 x/y/z/tracer tensors; the JSON binds every byte
range, report and state identity. These files are qualification references only;
the interactive scene never loads them as recorded playback. Browser qualification
compares actual worker responses and independently recomputes full-field metrics.

Three release-mode contracts check initialization, rollback/retry and both complete
trajectories. The two trajectory tests are skipped in debug mode to keep ordinary
adapter checks bounded; the packet builder runs all tests in release mode.

Build outside the repository with Rust 1.92.0, the wasm32-unknown-unknown target,
and `wasm-bindgen-cli` **0.2.105** installed:

```sh
python3 docs/education/build_solver.py --out /tmp/rheon-solver-packet
```

The build uses locked Cargo dependencies, runs nine release contract tests (six cycle and three flow), and records
source and artifact SHA-256 hashes. No generated binaries belong in Git.
`validate_packet(repo, packet)` checks source/artifact freshness and reruns the
retained native exporters. `publish(repo, out, packet)` copies the admitted web
module, WASM, type declarations, native JSON/binary references and build receipt into `out/solver`;
it leaves the host executables in the external packet. Chromium qualification is
separate from this build receipt.
