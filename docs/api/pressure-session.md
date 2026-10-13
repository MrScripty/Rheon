# Qualified pressure session

`PressureSession` composes the real `solver_provider::ProviderWorkspace` from PR #49
(`b8ca87cab7f74f64c1686714a33e843fbb178c3d`) with publication, source identity,
quota, stop and monotonic-deadline ownership. This pressure-only adapter does not
advance a simulation. The separate [full-step API](full-step-provider.md) integrates
providers into sealed-box `SimulationSession` transactions. `MaterialSession` remains
unsupported because its SPH EOS is not a sealed-box grid projection.
No numerical algorithm or acceptance tolerance is changed by this adapter.

Construct with a grid, shared `SessionIdentity`, combined array-memory cap,
`PressureImplementation` fallback and maximum accepted solve count. Invoke `solve`
with the actual full integrated sealed-box RHS and `PressureSolveControls` (source
ID/version, density, dt and existing `PressureSettings`). `None` selects the declared
numerical reference. `Some(&mut provider)` explicitly selects provider-with-reference-
fallback; there is no implicit backend policy. The caller owns provider lifetime,
model storage and external side effects. This supports the published frozen
`LabModalPressure` and the existing typed provider interface without a registry or
speculative universal physics trait.

A successful call returns the unchanged `ProviderReport`: selected implementation,
attempted metadata/provenance, rejection reason, recomputed candidate/accepted
residuals and inference/qualification/fallback/total durations. Report timings are
those of the provider workspace; they exclude session publication copy, final
callback, caller input preparation, model construction and velocity correction.
They are measurements of that call, not a performance guarantee or speedup claim.

`snapshot()` is None initially. On success it borrows an immutable
`QualifiedPressureSnapshot`, stage `qualified-sealed-box-pressure`, with pressure
in Pa, x-fastest cell-centered f64 layout, input controls/identity, solve generation
and report. The RHS has units m³/s². Density is kg/m³, dt seconds. The source ID and
version are caller declarations; input arrays are not retained. This is inspection,
not cold replay or a durable provenance attestation. API version 1 is source-level;
there is no stable ABI or wire-schema promise.

The workspace's pressure is invalid after errors, so the session owns a separate
published vector and only replaces it after successful qualification plus a final
cancellation/deadline checkpoint. Errors preserve the previous controls, pressure,
report and generation, and do not consume quota. Cancellation aborts without fallback.
Stop is terminal while preserving inspection. Deadlines are cooperative, checked
around callbacks and before publication; kernels and arbitrary providers cannot be
preempted and panics/hung calls are not recovered. A source/provider can change
its own external state on rejection; only session publication is transactional.

The combined Vec-capacity cap covers workspace and retained pressure. Provider
models, caller RHS, identity, stack, allocator overhead and process RSS are outside
it. Actual source validation, residual recomputation, supported-domain checks and
fallback remain exclusively owned by `solver_provider`.

A qualified pressure is **not an accepted full simulation step**. A full pipeline
must apply velocity correction, recompute actual f32 divergence, and pass its
physical boundary and transactional publication gates. Capability metadata says
`full_step_acceptance=false`. No generic hook is added to bypass those gates.

## Consumer and evidence

```
cargo run --release --locked --no-default-features --example pressure_session
cargo run --release --locked --no-default-features --example pressure_session -- VERIFIED_MODEL_EXPORT
```

Obtain the export using the [provider's SHA256-verified workflow](../implementation/ai-provider/README.md).
The example constructs an authored nonzero smooth pressure mode and its exact
operator RHS. Tests execute the frozen model through this session, preserve model
identity/provenance, and observe explicit SGS fallback for an unsupported grid.
An authored high-frequency pressure request did not qualify and exhausted the
3000-iteration fallback budget at the original tolerances; the test retains that
failure and verifies prior-publication preservation. No tolerance was loosened.
This evidence does not establish unrestricted model accuracy or full trajectories.

The API's raw transport/particle views are the integration input for CAD's VTK and
live-field work in PR #50 (`3f49a15`). CAD owns encoding and export UI; this module
adds neither. The [production provider route](full-step-provider.md) separately
qualifies full-step actual divergence and rollback through the real candidate/commit
path; this module retains its pressure-only capability. Godot binding/runtime,
GPU, ARM/WebAssembly and target-device timing remain unqualified.
