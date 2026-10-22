# CPU oracle refresh for GPU book publication

Source-only review, 21 October 2026. Original GPU research baseline:
`fd7c0f0aee63bc8710df99f9bb01154e7619b974`. Publication-base main:
`d67aec1a4575e0a61aa8cd931c8eeec33c8dc6a3`. This report inspected the actual
Git/source delta and proposes documentation corrections. It does not claim a
new numerical, runtime, device, benchmark or recording test result. No production
source, existing GPU document, science/build command or unrelated worker was
changed by this review. Root owns integration of the proposed text.

## Change provenance and unchanged oracle

PR145 merged at `7516a8e5d328c6dc8eda8ea577cb03f20b2e343a`. Its relevant source
commits are `f55cf595f8511fca9afc86cc572f26b017fb79ca` (validated authoring/evaluator),
`19e0145ce652fac565cd04b4e07ace3bbaa46954` (MAC/particle stage adapters), and
`8d2b4f8a6868b357df9e9be178c8fa290b7eda77` (accepted-force recording custody).
PR146 merged at `b9218038d138d1c73818cf4158757521853a5cd7`; it integrates native
desktop authoring, immutable request/run intent, dispatch and recording. The
selected core force/simulation/recording files have no additional delta between
those two merge results. PR147's separately owned FEM chapters and PR148's fluid
portal research are present at the publication base but confer no GPU or general
fluid implementation capability.

The inspected delta leaves `geometry.rs`, `sampling.rs`, `advection.rs`,
`operator.rs`, `pressure.rs`, `sgs.rs`, `viscosity.rs`, `liquid_step.rs`,
`solver_provider.rs`, `session.rs` and `material_session.rs` unchanged. Thus the
original filled-box MAC layout, six-f64-array pressure workspace, integrated
operator sign/units, zero pressure initialization, Jacobi/SGS PCG identities,
full true-residual plus predicted-divergence conjunction, stored-f32 divergence
and Courant acceptance remain the first pressure oracle. This is source identity
evidence, not a rerun of those contracts. The initial dense sealed-box GPU
recommendation remains appropriate.

The meaningful refresh is additional authored external-force inputs and their
accepted publication custody. The original inventory must retain its true
historical pin; a new dated refresh must distinguish publication-base source
observations from the original research and from historical execution receipts.

## Actual authored-field contract

[force_field.rs](../../../../src/force_field.rs) now owns version-1 `ForceScene`
authoring and validated `ForceSnapshot` evaluation. `freeze()` validates the
complete scene, including disabled objects, then clones it into an owned `Arc`.
It permits at most 256 objects with unique nonzero IDs. Objects carry name,
enabled state, world translation, unit quaternion, units, kind, support and time.
There is no scale transform or mutable evaluator access. Deserializing a scene
does not bypass the freeze boundary.

The admitted kinds are directional local vectors, signed radial constant
magnitude (positive outward, zero at the center), and signed vortex constant
tangential magnitude around local +Y (zero on the axis). Vortex positive
handedness sends +X toward -Z. These are explicit normalized fields, not inverse
square forces or a vorticity-confinement stencil. Supports are global, local
sphere with open outer boundary, or local half-open box
`[-half_extent,+half_extent)`. An optional edge fraction supplies the existing
smoothstep weight. Evaluation transforms the query to local space, evaluates
support and field, then rotates the weighted result to world space.

Activation is enabled and `start <= accepted_interval_start < end` (or no end).
Intervals are not split at activation endpoints. Consequently sampling at a
midpoint/end, integrating the on/off interval exactly, or advancing the field
time during particle stability retries would change the current CPU method.
Quaternion validation tolerates squared-norm error at most `1e-12`; evaluation
does not silently normalize that quaternion. Stable direction normalization
handles zero, tiny and large coordinates. The frozen object order defines
superposition order; a parallel object reduction would introduce a new rounding
order that needs an explicit policy.

`ForceUnits::Acceleration` is m/s². `ForceDensity` is N/m³ and is divided by the
positive density supplied by the numerical owner, never by particle mass. The
common authoring/evaluation definition therefore does not make the two solver
adapters' density or sampling rules interchangeable.

## MAC stage, timestep and stamp changes

[simulation.rs:420](../../../../src/simulation.rs#L420) adds
`step_with_force_fields(requested_dt, source, fields, cancel)` and
`step_with_emitter_and_force_fields(requested_dt, emitter, fields, cancel)`.
Their internal construction explicitly supplies no tracer barrier, prescribed
boundary, free surface, viscosity or provider, and no legacy `BodyForce` slice.
They are closed-box carrier entry points, not arbitrary combinations of every
optional `Simulation` capability. The emitter remains its existing separate
source stage.

[forces.rs:128](../../../../src/forces.rs#L128) evaluates the immutable scene at
each velocity component's own world-space MAC face position, using constant
`SimulationConfig.density` and accepted interval start. The stage follows
advection and legacy smoke/nozzle forcing and precedes pressure. Normal outer
faces remain unchanged. Writes use existing f32 storage and the force work
report uses the actual stored velocity change with the existing discrete face
mass. `BeforeForces` and per-z-slice `ForceSlice` cancellation remain cooperative.

[simulation.rs:793](../../../../src/simulation.rs#L793) adds an authored-field
acceleration rate to timestep preflight. The snapshot bound sums active-object
magnitude bounds by triangle inequality, ignores cancellation/support overlap,
and includes a small rotation/rounding allowance. It is multiplied by
`sum(1/h_d)` before entering the existing acceleration displacement budget. A
GPU maximum sampled only on occupied faces is not the same preflight policy.
No new pressure equation or pressure tolerance results from this force addition.

The new `AcceptedForceIntervalPressure` exposes pressure, actual interval start,
step report, source/emitter controls, frozen scene and optional external-stage
work. On authored success the legacy `accepted_interval_pressure()` stays absent
to avoid presenting incomplete controls. Both legacy and force-aware carrier
stamps are cleared before a later prepare attempt, reset or commit; even a
rejected next attempt invalidates pressure inspection while accepted velocity,
tracer, time and generation remain intact. An empty authored scene skips field
work in the MAC numerical path but still receives force-aware custody: explicit
empty metadata is distinct from absence of authored metadata.

## Particle adapter and different rejected-attempt semantics

[particle_fluid.rs:403](../../../../src/particle_fluid.rs#L403) adds
`ParticleFluidSimulation::step_with_force_fields`. After candidate emission and
the existing density/EOS evaluation, it adds scene acceleration to gravity,
then accumulates existing pair forces. Query positions are pre-integration
particle positions, including newly emitted particles. Force density divides by
the evaluated local SPH density used by that pressure stage, not the material's
reference density, constant MAC density or parcel mass. The field time remains
the accepted interval start on all up-to-sixteen stability retries. The existing
post-force acceleration/damping screen includes the resulting total force;
there is no new MAC-style field-bound preflight in this particle adapter.

Committed particle diagnostics still refer to post-emission/pre-integration
evaluation, now including gravity, authored fields and pair forces. They exclude
subsequent wall/contact response. Accepted positions/velocities remain at the
new publication time. Pair kernels, EOS, neighbor visibility and contact geometry
are not replaced by authored forces and gain no continuum-water qualification.

`AcceptedParticleForceControls` retains actual interval start, original requested
dt, emitter, actual accepted report and frozen scene only after success. Unlike
carrier pressure scratch, its diagnostic storage has independent ownership:
rejected/cancelled later attempts preserve the prior particle force publication.
A successful legacy particle step clears the authored stamp; a successful
authored step replaces it. Documentation must not generalize carrier pressure
stamp invalidation into a rule that all accepted particle metadata disappears
on any next attempt. The synchronous `MaterialSession` wrapper was not extended
to expose these new authored APIs in this delta.

## Recording additions and preserved limits

[recording/force_fields.rs](../../../../src/recording/force_fields.rs) adds a
versioned binary input document within the existing SQLite envelope. The SQL
schema version and legacy field encodings/hash algorithms are unchanged.
The new field is `external_force_scene_v1`, scalar u8, one component, dimensions
`[encoded_byte_count,1,1]`, units `1`, association
`immutable_run_input; world_m; activation_at_interval_start_s`, stage
`external_force_document_v1`, with no evaluation timestamp. It represents
immutable input metadata, not sampled acceleration or an endpoint force field.
Encoding is little-endian, versioned, capped at 256 KiB and preserves f64 bits,
including signed zero, full IDs/order, disabled objects and activation controls.
Decoding bounds bytes/count/name length, rejects invalid/trailing data and
freezes the reconstructed scene.

`RecordingRun::with_force_fields` binds the complete document's SHA256 into the
existing configuration header. `validate_run_force_fields` requires either
legacy absence on both sides or matching run/step force metadata. Append and
selected read both apply it. Therefore the current force-aware archive binds
one immutable scene for the run: changing even a disabled object's name or
bits cannot be silently treated as the same run input. Absence means legacy
metadata, not proof that an arbitrary producer applied no force. This binding
checks integrity/association, not authenticated physical execution.

[carrier_publication](../../../../src/recording/carrier.rs#L46) now accepts
either the legacy success stamp or the actual force-aware pressure stamp. The
legacy canonical inventory remains four constructor fields/five accepted fields;
an authored publication adds one metadata field. The adapter automatically
retains the actual accepted scene plus external-stage work/control provenance.
`carrier_publication_with_force_fields` attaches constructor metadata or checks
that an accepted publication already owns exactly the supplied scene. It cannot
relabel an unstamped accepted publication or substitute later draft force data.
The existing error string mentioning only `step/step_with_emitter` is narrower
than the new control flow; source branches and tests, not that string alone,
establish force-aware admission.

[material_publication](../../../../src/recording/material.rs#L90) similarly
checks supplied requested-dt bits, emitter and report against actual accepted
particle-force custody, then retains the actual scene. Its explicit force-aware
wrapper attaches constructor metadata or requires actual accepted custody.
Both decoded carrier and material values now expose optional force metadata.
No derived per-particle acceleration is recomputed from the scene during readback.

This is a new typed force-document codec, so the broad statement that *all*
recorded controls are opaque Debug text needs qualification. Most configuration,
report and source-control text remains opaque; the force document itself is now
bit-preserving validated typed input. It still is not a complete restart codec:
mesh bytes, source/emission/integrator state and all other required state remain
absent. SQLite acceptance-after-physics, durable-prefix/unrecorded-tail,
uncertain-append poisoning, keep-all limits and cold-selection semantics are
unchanged. Legacy `step_with_forces`, viscosity and arbitrary GPU producers do
not gain a recording stamp merely because authored-field adapters exist.

PR146's [desktop dispatch](../../../../src/bin/desktop/carrier_execution.rs#L131)
uses these authored APIs for the native carrier and the direct particle owner.
Its [force eligibility policy](../../../../src/bin/desktop/force_run_policy.rs)
admits native carrier/particle material while refusing configured/session/
provider/free-surface/AI workflows. Presence includes empty/disabled/zero scenes.
[FrozenSpec](../../../../src/bin/desktop/run_sets.rs#L248) preserves legacy
native intent v1 without fields and adds v2 with the validated scene and a
force-inclusive digest. This is frozen rerun intent, not accepted-state restart.
No native interaction or worker runtime was exercised by this refresh.

## Proposed text and placement for root integration

These locations refer to the merged original research text before publication
edits; headings remain stable if root changes line numbers.

| Artifact/location | Required correction |
| --- | --- |
| CPU inventory introduction, currently lines 3–9 | Keep `fd7c0f0...` as original inspection provenance. Add a separate dated publication refresh at full `d67aec1...` and identify this report/current source. Do not globally replace the old pin and imply the old review executed on new source. |
| CPU inventory source map, section 1 | Add `force_field.rs`, its frozen scene/evaluator and force-aware recording owner. Scope the old constant-vector `BodyForce` paragraph as the legacy API, then add authored-field timing/support/density distinctions. |
| CPU inventory step order, sections 3 and 5 | Add authored bound and face force stage; name its closed-box-only combinations. Distinguish both invalidated carrier stamps from persistent accepted particle controls on rejection. |
| CPU inventory particle section 7 | Acceleration may include authored fields. Include local evaluated-density conversion and retry time semantics, while retaining pre-integration versus accepted-position timing. Keep MaterialSession's unchanged narrower API explicit. |
| CPU inventory recording section 8 | Qualify four/five fields as legacy, add one input document for authored profiles and run digest custody. Explain force metadata is typed while other text remains opaque. Preserve the no-restart conclusion. |
| Chapter 37 introduction and section 37.8 | Preserve original baseline, link dated refresh. Add actual field-evaluator semantics so radial/vortex are not confused with vorticity confinement or future buoyancy. Frozen scene/order/time/density are part of the admitted force kernel and dt reduction policy. |
| Chapter 38 source baseline and existing acceptance section | Identify the two source epochs. Add immutable force document/version/digest to device input lifetimes and readback admission; changing authoring draft must not mutate an in-flight device scene or relabel accepted records. These inputs do not alter CUDA/Vulkan independence. |
| Chapter 39 introduction and oracle tuple | Retain original pin and append current refresh; add exact force-scene bytes/version/digest, activation stage, density convention and recording profile to future oracle identity. |
| Chapter 39 validation matrix, Forces/Persistence rows | Add rotated radial/vortex/support edges, activation endpoints/straddles, empty-vs-absent, summed-object order, force-aware stamps and run/step scene mismatch. Core integration tests are prepared/ignored and were not executed here. |
| Chapter 39 G3 and following paragraph, currently lines 319–324 | Replace blanket exclusion of force recording with the exact legacy/authored/viscosity/GPU distinction below. |
| Book source plan, source baseline paragraph | Record PR145/146 as merged at the publication refresh, retaining that they were active/outside the original write set. Separate original research authorization/checks from later authorized publication; do not rewrite history as if navigation/build/merge had been authorized originally. |

Suggested replacement for Chapter 39's recording-profile paragraph:

> At the original research baseline, canonical carrier recording admitted the
> stamped `step` and `step_with_emitter` lineage. At the publication refresh
> `d67aec1a4575e0a61aa8cd931c8eeec33c8dc6a3`, it also consumes the force-aware
> success stamp from `step_with_force_fields` and
> `step_with_emitter_and_force_fields`, retaining the actual immutable scene in
> `external_force_scene_v1` and checking its run binding. Legacy `BodyForce`
> `step_with_forces`, viscosity, configured/provider/free-surface and GPU-resident
> producers are not thereby admitted. A new GPU adapter must preserve field
> stages, actual source/force custody, profile inventory and run/step association;
> passing existing recording tests does not admit a new producer.

Suggested publication refresh text for the inventory and chapter/source-plan
introductions (abbreviate and link to the single detailed owner):

> The original inventory is pinned to `fd7c0f0aee63bc8710df99f9bb01154e7619b974`.
> A source-only publication refresh inspected main
> `d67aec1a4575e0a61aa8cd931c8eeec33c8dc6a3` after PR145/146 merged. Pressure,
> layout and acceptance remain unchanged in that delta; immutable authored
> force scenes, their MAC/particle adapters and force-aware recording custody
> are additional current capabilities. This refresh adds no execution result.

## GPU recommendation consequences and available source oracles

Keep G0 plus a small isolated G1 dense operator lab as the first future slice.
An optional first no-authored-field profile must explicitly say that it excludes
the new force capability; a later full-step parity claim must either implement
that admitted profile or visibly refuse it. Do not silently drop nonzero fields,
map radial/vortex to a constant vector, normalize away authored bits or treat
empty metadata as absent. The GPU does not create new physics by evaluating
these existing formulas in parallel.

For an authored MAC extension, retain one immutable, versioned/digested device
scene per admitted input epoch, evaluate on each own-axis face lattice at interval
start using constant rho, preserve the conservative preflight acceleration bound,
and preserve the f32 force work calculation. Define precision/order policy for
rotation, normalization, support smoothstep and object superposition. The
snapshot input and staging copies add memory beyond the old field-payload plan.
For particle work, use the separate local-density/pre-integration contract and
retain its retry/custody distinctions; it remains a later separate model.

The relevant source oracles now include
[force-field contracts](../../../../tests/force_field_contract.rs),
[solver adapter contracts](../../../../tests/force_field_solver_contract.rs),
[force recording integration contracts](../../../../tests/force_recording_integration_contract.rs)
and in-source codec/schema adversarial tests. The two integration suites contain
explicit ignored execution admission; their presence or compilation cannot be
reported as numerical/SQLite acceptance. This refresh did not run, reinterpret
or authenticate their upstream receipts. Root's separately owned publication
checks must report their own exact scope and outcomes.
