# Shared external force contract, version 1

Backend owner scope: the library domain, pure evaluator and the closed-box MAC
and particle adapters. Desktop owns controls, inspector, scene handles, outlines,
timeline and run/archive wiring. No desktop source is changed here.

## Repository and implementation plan

On 2026-10-20, remote main was verified as
`fd7c0f0aee63bc8710df99f9bb01154e7619b974`; remote
`implementation/ui-hierarchy-20261020` was
`426fd25d22975e1259f5543c77dc771a6ac646ff`. The pre-existing local `work` branch
at `6ba41c7` and its worktree were preserved. Implementation uses separate branch
`implementation/shared-force-domain-20261020` based on the verified main.
No AGENTS.md or .agents/skills files were present in the repository/workspace
instruction directories. Existing Rust 2024/1.92, unsafe-forbid, checked error
and transactional candidate/commit conventions apply.

The compact contract and disjoint write set were announced before implementation:

1. Add validated authoring objects, one pure semantic evaluator, owned immutable
   run snapshots, optional portable serialization and exact pure tests.
2. Add opt-in MAC/particle entry points; preserve BodyForce/gravity paths and
   stage boundaries. Share the MAC lattice/work kernel.
3. Obtain independent equation/source review. Prepare bounded integration tests,
   compile/list them, and leave them unexecuted.
4. Record validation and UI requirements, commit on the isolated branch. No push,
   merge, deployment, branch removal, credentials or network configuration changes.

Write set: Cargo.toml; src/{lib,force_field,forces,simulation,particle_fluid}.rs;
tests/force_field{,_solver}_contract.rs; this plan/contract and validation record.
Cargo.lock is unchanged. No scientific L1/MMS, model workload or native simulation
execution is part of this change.

## Authoring objects and identity

`ForceScene { version: FORCE_SCENE_VERSION, objects: Vec<ForceObject> }` is the
editable document. `SceneObjectId(u64)` is a caller-assigned nonzero stable ID.
The host allocates IDs in one scene namespace for models and forces, retaining
IDs across rename/reorder/save/load and assigning fresh IDs to duplicates.
Freeze rejects zero IDs and duplicates within the force list. Model IDs and
cross-kind uniqueness remain the scene host's responsibility. Particle IDs are
a separate run-local namespace and are not scene-object IDs.

`ForceObject` carries ID, name, enabled, transform, units, kind, support and time.
Names have a 256 UTF-8 byte limit; a scene has at most 256 force objects. Disabled
objects are still validated. Snapshot metadata/allocation is caller-owned and
outside the solvers' retained-array budgets. Hosts should bound serialized input
bytes before deserialization; freeze's object count limit is not a parser limit.

`ForceTransform` is translation in metres and quaternion `[x,y,z,w]` mapping
local to world. Squared quaternion norm must be within 1e-12 of one; the backend
does not silently normalize malformed authoring input. The UI must produce a
unit quaternion. There is no scale, parenting, animated pose or mesh attachment.
Support dimensions are edited explicitly in metres.

## Equations and units

Let `R` be the quaternion rotation and `q = R^T (world - translation)`.
The raw local vector `v(q)` is:

| Kind | Raw local vector |
| --- | --- |
| Directional | Authored local `vector` |
| Radial | `strength * q/length(q)`; zero at the centre |
| Vortex | `strength * (q.z,0,-q.x)/hypot(q.x,q.z)`; zero on local Y axis |

Positive radial strength repels; negative attracts. Positive vortex strength
follows the right-hand rule about local +Y (at local +X it points toward -Z).
Strength is constant magnitude before support weighting, not an inverse-square
law or a calibrated pressure/torque. Direction normalization scales by the
largest absolute component first, avoiding norm underflow/overflow. Coincident
centres and axes are defined as zero; there is no hidden softening radius.

Support is global, sphere, or local axis-aligned box (rotated with the object).
Sphere uses `r=length(q/radius)` and excludes `r>=1`. Box uses
`r=max(abs(q[d])/half_extent[d])` and hard support
`-half_extent <= q < +half_extent` componentwise. Thus a hard box includes its
lower plane and excludes its upper plane. Box/sphere dimensions are positive.
Support can extend beyond the simulation domain; adapters sample only their
normal domain and preserve their boundary policy. Legacy BodyForce retains its
stricter requirement that its world region fit inside the grid.

With edge fraction `e` in [0,1], bounded support has weight 1 when `e=0`.
For `e>0`, set `s=clamp((1-r)/e,0,1)`, then weight is `s*s*(3-2*s)`.
It equals one in the inner region and smoothly reaches zero at all outer planes.
Box falloff uses max-norm shells, not Euclidean distance to the nearest surface.
The sphere centre remains fully weighted even with `e=1`.

Each active object's world value is `R*v(q)*weight`. `ForceUnits::Acceleration`
means m/s²; `ForceUnits::ForceDensity` means N/m³. The latter divides the world
value by positive local mass density in kg/m³. Ordered vector addition produces
one acceleration. Total force in N, velocity targets/drag, inverse-square
singularities, potential-energy claims, material stress, turbulence closure and
source emission are not implemented as force types.

All authored numeric values must be finite. Sampling requires finite position,
nonnegative finite time and positive finite density. Arithmetic overflow returns
an error; it never clips strength or returns nonfinite accelerations. Subnormal
direction normalization and magnitude bounds have pure regression coverage.

## Time, snapshot and serialization

`ForceTime { start_seconds, end_seconds }` activates on `[start,end)`; `None`
means no end. Start is nonnegative, and a supplied end is finite and after start.
Both adapters sample force activation at the accepted interval's start. A step
is not split at activation endpoints, so a force starting inside a step begins
on the next step. Every particle retry samples the same accepted start time.
The first schema supports static strength/pose and these activation intervals.

`ForceScene::freeze()` validates and independently owns a cloned `ForceScene`
inside `ForceSnapshot`. Only an immutable view is exposed; no Deserialize or
mutable accessor can bypass validation. Draft edits after freeze cannot change
a run. The host must retain that snapshot with run inputs and use it on every
step; per-call borrowing alone does not enforce host-level run consistency.

Serde is available under `force-serde` without desktop dependencies, and under
`desktop`. The version 1 document uses derived enum variant names and explicit
version. Structs/enum payloads reject unknown fields, unknown enum variants fail,
and freeze rejects unsupported versions. There is no silent migration. Save the
entire versioned scene, including disabled objects, IDs and names; do not save
only enabled vectors. Deserializing gives authoring data requiring freeze again.

## Solver adapters and legacy compatibility

- `Simulation::step_with_force_fields(dt, smoke_source, snapshot, cancel)`
  samples each component at its own MAC face position before projection.
- `Simulation::step_with_emitter_and_force_fields(dt, emitter, snapshot, cancel)`
  combines the existing nozzle source kernel with the same authored force stage.
- `ParticleFluidSimulation::step_with_force_fields(dt, emitter, snapshot, cancel)`
  samples post-emission/pre-integration particle positions and the accepted
  interval start time. It adds fields to existing gravity, then pair forces.

MAC uses configured constant density. Its force-density division happens once
inside the common semantic evaluator. The existing closed normal wall faces
remain fixed; pressure may balance/project out applied acceleration. The shared
MAC update computes stored-f32 finite-step work with the existing face mass
`rho*cell_volume`; this measures only the force stage, not full-step energy.
A conservative scalar magnitude bound is applied to every world component for
the existing displacement budget. It sums object magnitudes, with a rounding
allowance, ignores support/cancellation, and can shorten dt more than legacy
componentwise BodyForce budgeting. It is a policy, not a stability theorem.

Particle force density divides by the evaluated local thermodynamic/SPH density
at the force stage, **not particle mass or material reference density**. This is
the model's reference-volume density with its existing 0.05 reference-density
floor. The adapter introduces no extra density clamp. Existing combined-force
retry limits, domain walls and static-mesh response still apply. Diagnostics
include authored acceleration at their existing pre-integration stage.

Legacy `BodyForce`, regions, smoke, gravity and old entry points remain supported.
Empty snapshots route through legacy paths. Existing public calls pass no
snapshot. Failure/cancellation retains accepted fields/particles/time/generation;
scratch is overwritten on retry. Authored sampling errors become each adapter's
existing ArithmeticFailure error. Sources and material forces remain separate.

## UI integration requirements and explicit limits

Use one stable scene selection identity for model and force rows. Properties map
to ForceObject fields; transform handles update translation/quaternion, while
bounds handles update support sizes. Global force extents need a UI glyph rather
than a physical finite boundary. Vortex arrows follow the signed +Y convention.
The UI may disable edits during a run, but it must still retain the frozen run
snapshot rather than re-read mutable drafts. Keep run fields in history/archive
metadata so historical viewport glyphs cannot be relabeled by draft edits.

At Simulate: freeze the draft, surface validation failures, put the immutable
snapshot in the run request, and dispatch the relevant new adapter. Capture the
same versioned document with saved run specifications. Old records can map an
absent force document to an empty scene only through an explicit host migration.
Gravity remains additive: a gravity authoring object must not accidentally apply
Earth gravity twice. No desktop or recording schema is changed by this branch.

Authored fields are initially unsupported in LiquidTransportSimulation,
MaterialSession/SimulationSession, provider/configured-comparison adapters,
free-surface/column, box-flux and viscosity combinations. Do not silently display
fields as applied in those paths; retain their existing APIs or reject unsupported
requests. New field calls also do not publish the legacy accepted-interval
pressure inspection controls, which cannot describe authored snapshots yet.
Combining an authored snapshot with a separate BodyForce list is not exposed.
No solver retains authoring IDs or scene geometry internally. Archive/session
custody and desktop visuals require the coordinated UI owner/parent follow-up.
