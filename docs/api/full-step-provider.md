# Production full-step pressure selection

`Simulation::step_with_pressure_provider` and
`LiquidTransportSimulation::step_with_pressure_provider` compose the existing
`solver_provider::PressureProvider` into the production sealed-box step. The public
`SimulationSession` exposes the same selection with its lifecycle, quota and
cooperative deadline. This is distinct from `PressureSession`, which still publishes
only a qualified pressure stage. The provider owner contract is unchanged.

`ProviderStepWorkspace::new(grid, density, limit, implementation)` owns one RHS array
plus the existing eight-array dispatcher. Grid, density and numerical fallback must
match the simulation. The preferred provider is borrowed per call. `None` selects the
numerical reference; `Some` uses the existing provider qualification and explicit
reference fallback. No network service, asynchronous queue or automatic model loading
is introduced. The example `production_provider_session` executes 24 bounded host
updates, optionally loading the SHA256-verified export produced by the existing tools.

## Numerical and publication boundary

The existing owner selects actual dt, advects accepted velocity and applies forces.
The adapter assembles the existing sealed operator RHS from that candidate velocity
and invokes the provider. Correcting the stored f32 candidate is followed by the
original actual-divergence, Courant, tracer finite/range, time/generation and final
commit gates. Liquid transport additionally qualifies the represented volume before
pressure, velocity, tracer, fractions, stamps and clocks commit together. No numerical
threshold is relaxed. Successful pressure qualification cannot bypass later gates.

A post-pressure failure rejects the entire interval. It does not silently retry the
whole step with another solver; only pressure-stage fallback inside the unchanged
dispatcher is permitted. Model support is evaluated against the **actual**, possibly
shortened dt. An unsupported dt can therefore select the explicit numerical fallback.

`SessionSnapshot::provider_report()` is present only on an accepted full step from
this path. It preserves selected/attempted identity, model provenance, rejection,
residuals and pressure-stage timings. Those timings exclude the rest of the step.
The snapshot stage is `Accepted`; its contained provider metadata still names the
pressure stage. The snapshot's run/source identity, frame, time, carrier/liquid stamps
and pressure all belong to that same completed interval. Constructor and successfully
accepted legacy boundary steps have no provider report. Any rejection preserves the
previous report and all published state. Caller controls/model scratch may change
between calls; a failed call does not roll back a caller-owned provider's internal
state, side effects or profiling counters.

Cancellation/deadlines remain synchronous and cooperative, including a check after
the final user callback before commit. Kernels and callbacks cannot be preempted;
panics/hung providers are not isolated. Rejected calls consume no step quota.

## Scope and ownership

Only a sealed, constant-density full-box carrier is admitted. Slab/column/free-surface
and static column-MAC modes return `UnsupportedProviderDomain`; this path has no
prescribed box-flux or viscosity parameter. It is not an SPH EOS, obstacle/moving CAD,
variable-density solver, GPU backend or engine binding. Existing liquid fractions
remain represented transport, not two-phase inertia. Provider admission and retained
source guarantees are those of PR49 through `89997c5`.

`ProviderStepWorkspace::allocated_bytes()` counts actual array capacities. Its limit
excludes caller/model storage and process/allocator overhead. Original simulation
pressure scratch stays allocated. `LiquidStepReport` exposes provider, boundary,
viscosity and owner components; `total_array_bytes` is their sum for the active call.
`SimulationSession::allocated_array_bytes()` also counts its retained legacy boundary
workspace even during provider calls; add the caller-owned provider workspace to that
value for combined session/scratch array residency. Do not add the liquid report total
again or call any of these values RSS.

API capability version 2 adds `sealed_box_pressure_provider`. Existing dense3D wire
versions and v2 accepted-control validation are unchanged. Dense3D frame writing uses
accepted fields and does not serialize the provider report: retain that report
separately if provenance is needed; a frame is not a complete model-training manifest.

## Acceptance evidence

`provider_step_contract` exercises original numerical step equivalence, every observed
cancellation stage, explicit fallback, a pressure-qualified candidate rejected by
stored-velocity divergence, post-pressure liquid failure, unsupported surface modes,
workspace identity/caps, and a real frozen-model 24-interval production trajectory.
The actual model must be selected, not merely attempted. `session` unit tests inject a
clock crossing the deadline inside the final callback, without sleeps.

The existing PR49 trajectory harness/oracle remains an independent dependency with
its final verifier fixes; it is not copied or relabelled as production-session
validation. Source-specific test results are recorded in the implementation ledger.
No general physics accuracy, real-time deadline or speedup claim follows.
