# Embedded material session

`MaterialSession` owns the existing SPH preview solver, terminal stop, accepted-call
quota and borrowed immutable snapshots. It extends the transport API without
pretending that particles are MAC cells. Source/run identity and publication stages
reuse `SessionIdentity` and `SnapshotStage`. The quantitative dependency is PR #47,
`1923d5bd3442a5369878a1a317b52f4a89026d5b`; its `ParticleDiagnostic` remains the only
capture authority. This stack includes that complete branch, not a copied capture.

Construction accepts existing `GridGeometry` and `ParticleFluidConfig`. Material,
fill, ambient fluid, temperature, gravity, resolution, particle capacity and memory
limits are fixed for that session. Recreate with a new run ID to change them.
`MaterialStepControls` supplies requested dt and optional `MaterialEmitterControl`:
caller source ID/version plus the actual `RotatingCylinderEmitter` parameters.
Changing emitter controls needs no restart. Validity is enforced by the existing
solver; caller identities label provenance and do not authenticate it.

A successful step stores the actual request alongside `ParticleStepReport`.
Failed/cancelled requests never relabel the prior publication. Consumers advance
by report.dt, which may be shortened by the existing stability controller (at most
16 attempts per call). Snapshots expose particles, immutable config, grid, time,
generation, accepted controls and the report. Particle identity is (run_id,index);
indices are append-only within a session. Consumers own any retained copies.

`particle_diagnostics()` forwards PR #47's optional `(evaluation_time, slice)`.
Initial diagnostics are absent. Committed evaluation is **post-emission,
pre-integration**, including pair forces and gravity but excluding later wall
velocity/temperature response. Accepted positions have the later accepted time.
`diagnostic_stage()` describes evaluation separately from `stage()` publication.
Density is kg/m³, pressure Pa, acceleration m/s², mass kg, temperature K, velocity
m/s and positions m. These describe the implemented preview model, not validated
real-fluid measurements or complete wall-force ledgers.

`MaterialSessionError` distinguishes invalid identity, stop, quota, deadline and
underlying physics errors. Rejection is retryable without consuming quota.
Deadlines use monotonic time before work and around existing cancellation callbacks.
No kernel/provider/thread preemption, real-time budget or asynchronous shutdown is
promised. The solver memory cap includes canonical diagnostics; reported array bytes
exclude caller copies, source identity, allocator overhead, stack and process RSS.

Run `cargo run --locked --no-default-features --example material_session` for a
bounded fixed-update host producing full-precision inspection CSV. It uses actual
accepted dt, a cooperative deadline and an explicit catch-up count. CSV is an example
consumer projection, not a new Tuldok dataset schema. Existing dense3D v2 production
continues through its original owner. Godot bindings and GPU timing remain untested.

The material model does not use the grid pressure provider: capabilities explicitly
report `grid_pressure_provider=false`. Its EOS cannot be replaced by a frozen
sealed-box pressure model merely because both outputs are called pressure.
