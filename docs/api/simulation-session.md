# Embedded simulation session v1

Base: remote main `2209012`. Goal: expose existing generation controls and accepted
numerical data to Rust hosts, dataset producers and engine update loops. No new
physics, network server, renderer, provider implementation or wire-schema change.
The implementation plan, standards routing, acceptance and integration backlog live
in [the public API plan](../plans/public-simulation-api/plan.md). This document owns
the shipped contract; the plan owns sequencing. This first transport milestone
does not complete the broader multi-backend API objective.

## Contract for other workstreams

`SimulationSession::new` constructs the existing full-box transport path with an
owned boundary workspace; it deliberately does not accept arbitrary surface-mode
owners. `step` accepts unchanged `LiquidStepInputs` and `BoxFluxStepBoundary` and
returns `LiquidStepReport`. This includes requested dt, smoke source, body forces,
inlet fractions, distributed liquid source, volume settings and prescribed box
flux. Their native validators remain authoritative. A source/force does not gain
support beyond what the existing full-box step admits. Controls are borrowed for
one synchronous call, so they can be changed on each call without a queue.

`SessionSnapshot` privately binds a grid, publication number, immutable state and
last accepted report. `state()` returns the existing `LiquidTransportView`: f32
staggered x/y/z face velocity arrays and tracer, f64 pressure and liquid fractions,
accepted time, carrier generation, carrier and liquid stamps. See each native
field for exact types and the grid for x-fast indexing/face shapes. Constructor
pressure is zero; accepted pressure is the last interval's projection pressure
with the full-box gauge, not reconstructed end-surface pressure. Snapshots are
labelled `Constructor` or `Accepted`; no provisional solver stages escape.
The report retains existing boundary/inlet/source stamps and numerical diagnostics.

`SessionIdentity` is caller-supplied run ID and source revision (1..1000 UTF-8 bytes,
no control characters). It identifies a source declaration, not verified provenance,
a unique global run, or independent training samples. Owner state stamps remain
separate. API version 1 and crate version are exposed through capabilities, with
SI field units, transport-only and cooperative-cancellation flags. API version,
crate version, dense3D schema version and caller source revision are distinct.

Snapshots borrow the session; safe Rust prevents stepping while the view is still
used. Retention requires explicit consumer-owned copying or encoding. There are
no hidden history allocations, threads or queues. Stop is terminal and preserves
inspection; drop releases storage. Recreate the session to restart, using a new
run identity when appropriate. The accepted-step quota excludes rejected calls.
Caller input allocations, identity storage, allocator overhead, stack and RSS are
outside the reported simulation/workspace Vec-capacity payload.

Deadlines use host monotonic `Instant`, tested before work and at existing stage
checkpoints. Cancellation callbacks return true to reject the pending transaction.
A deadline or cancellation leaves the last accepted state unchanged and permits
retry. A kernel or caller callback can overrun the deadline; there is no preemption,
maximum wall time, asynchronous stop, frame-rate guarantee or real-time claim.
The host must bound callback work, requested resolution, solver iterations and
catch-up count. Advance time by the accepted report dt; CFL may shorten a request.

## Consumers

```sh
cargo run --locked --no-default-features --example headless_session > frames.jsonl
cargo run --locked --no-default-features --example game_loop_session
```

Both use an authored 4x2x2 fixture, not a production dataset or benchmark. The first
writes constructor plus eight accepted frames through existing dense3D encoding.
The second illustrates an engine-owned accumulator, capped catch-up work, cooperative
deadline and reading a published state for display. It does not sleep or own the
engine event loop. Output failure affects the caller's export, not session physics;
a partial JSONL frame must be discarded. Snapshot encoding does not create a
complete dataset manifest, guarantee durable publication, or capture controls.

For Tuldok use the existing `Dense3dSequence` and
`tools/export_dense3d_sequence.py --producer-controls-v2` publication workflow.
The fixed-case v2 accepted-control contract in
[producer controls](../implementation/dense3d-producer-controls.md) remains unchanged.
Do not relabel arbitrary session output as a completed v2 dataset: that workflow
requires exact supported controls, bounded capture, source/binary binding, hashes,
independent validation and atomic completion. Nine correlated frames are one run.

Godot has no binding crate, GDExtension ABI or Godot project in this repository.
The engine-neutral Rust game-loop sample is the supported integration seam for
this milestone. A native Godot binding would introduce dependencies and a separate
ABI/lifetime boundary; it is deferred rather than presenting an untested binding.
An eventual extension should own the session on one worker, copy accepted fields
into engine-owned buffers, and keep scene-tree access on the Godot main thread.
GPU, Godot runtime, desktop rendering and target-device timing are untested by this
API milestone. CAD, broader physics and AI capabilities must be added explicitly
when their implementations are accepted; they are not advertised by this facade.

## Compatibility and extension boundary

The Rust API is a source-level, public-versioned contract at API version 1;
there is no stable C ABI or remote protocol. Consumers pin the repository revision.
Breaking source or semantic changes require an explicit API-version decision and
consumer update; no runtime version-negotiation or overlapping old API is promised.
The existing dense3D wire formats have their separate compatibility policy.

Future material/SPH, quantitative inspection and AI-provider integration are tracked
in the plan. Do not interpret `transport_only` as a common denominator physics
claim: this facade cannot expose particle fields, collision semantics, model
accuracy or provider selection until those contracts have independent evidence.
