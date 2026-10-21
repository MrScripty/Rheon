# CPU source inventory and oracle boundaries for GPU research

This inventory pins the CPU reference to Rheon commit
`fd7c0f0aee63bc8710df99f9bb01154e7619b974` (`origin/main` at the start of this
research, 21 October 2026). It records source inspection, not newly executed
numerical evidence. No solver, scientific fixture, hardware benchmark, test suite,
book generator or proof checker was run for this inventory. It proposes no backend
implementation and changes no CPU algorithm, application worker or FEM code.

The useful initial GPU oracle is the existing constant-density Cartesian MAC
carrier, with explicit per-stage and publication checks. The particle material
preview, display voxels, conservative fraction transport, restricted surface
operators and experimental Stokes solvers are different models. Their presence
does not establish one general three-dimensional liquid solver. The
[library exports](../../../src/lib.rs), [current scope audit](../../audits/general-fluid-20261019/report.md)
and [requirements roadmap](requirements-roadmap.md) identify these distinctions.
The audit's numerical observations belong to its own recorded execution and source
lineage; citing them here does not rerun or extend their qualification.

## Publication refresh: authored forces and recording

Sections 1–10 below retain the original `fd7c0f0` source epoch. A separate
source-only refresh inspected publication-base main
`d67aec1a4575e0a61aa8cd931c8eeec33c8dc6a3` after PR145/146 merged.
The [publication source/license audit](gpu-sources-and-publication-scope.md)
keeps this refresh, historical execution and future GPU qualification separate.
Geometry, sampling/advection, pressure/operator, SGS, viscosity, liquid-step,
provider and session source are unchanged in that delta. No new numerical or
runtime pass is inferred from this comparison.

[`force_field.rs`](../../../src/force_field.rs) adds version-one `ForceScene`
authoring and validated immutable `ForceSnapshot` evaluation for up to 256
uniquely identified objects. Directional, signed constant-magnitude radial and
local-+Y vortex fields have world translation/quaternion orientation and global,
local open-sphere or local half-open-box support, with optional smoothstep edge
weight. These prescribed normalized fields are not inverse-square gravity or
solved vorticity confinement. Object order fixes superposition order. Activation
uses start-inclusive/end-exclusive bounds at the accepted interval start, with
no interval split at an activation endpoint. Disabled objects are validated;
empty/disabled/zero metadata is not silently converted to absent metadata.

The new closed-box `Simulation::step_with_force_fields` and
`step_with_emitter_and_force_fields` do not compose tracer barriers, prescribed
flux, surface modes, viscosity, providers or legacy `BodyForce` slices. The
[MAC adapter](../../../src/forces.rs) evaluates each component at its own world
face position after transport and legacy smoke/nozzle forcing, before pressure.
Acceleration is m/s²; force density is N/m³ divided by constant carrier density.
Normal outer faces remain unchanged. Preflight uses the active-object magnitude
triangle bound including its rotation/rounding allowance, rather than only a
maximum sampled on occupied faces. Work uses actual stored f32 velocity changes
and discrete face mass. An empty scene skips the numerical field stage but
retains force-aware publication custody.

The [particle adapter](../../../src/particle_fluid.rs) instead evaluates after
candidate emission and density/EOS, at pre-integration positions, before existing
pair-force accumulation. Force density divides by evaluated local SPH density,
not MAC density, reference density or parcel mass. All stability retries use the
accepted interval start. Diagnostics precede integration/contact while accepted
positions/velocities belong to the later publication time. The unchanged
`MaterialSession` wrapper does not expose every new direct API.

Both carrier interval-pressure stamps, legacy and force-aware, invalidate on a
later prepare attempt even if it fails; accepted velocity/tracer/time/generation
remain intact. Accepted particle force controls own independent diagnostic
storage and survive rejected/cancelled attempts. A successful legacy particle
step clears authored custody; a successful authored step replaces it. These
owners must not be conflated in a GPU publication policy.

[`recording/force_fields.rs`](../../../src/recording/force_fields.rs) adds the
typed little-endian `external_force_scene_v1` input document inside the unchanged
SQLite envelope. Capped at 256 KiB, it preserves order, IDs, names, disabled
objects and f64 bits including signed zero. It is immutable input metadata, not
sampled endpoint acceleration. Decoding bounds data and freezes the scene.
A run-header SHA256 binds the scene; append and selected read require matching
run/step metadata. Absence denotes the legacy profile, not proof that arbitrary
producers applied no force.

The [carrier adapter](../../../src/recording/carrier.rs) now consumes either
actual legacy or force-aware success custody. Authored publication adds one input
field to the legacy four-constructor/five-accepted-field profile; its wrapper
cannot relabel an unstamped generation. The [material adapter](../../../src/recording/material.rs)
checks requested-dt bits, emitter and report against actual accepted force
custody. Most other controls remain opaque text, but this scene has a typed
validated codec. It and frozen rerun intent still omit complete restart state.
Durable-prefix, unrecorded-tail, uncertain-append poisoning and keep-all limits
remain unchanged. Legacy `step_with_forces`, viscosity and GPU producers gain
no recording admission simply because authored adapters exist.

Current source oracles include [field](../../../tests/force_field_contract.rs),
[solver-adapter](../../../tests/force_field_solver_contract.rs) and
[recording](../../../tests/force_recording_integration_contract.rs) contracts.
Ignored integration cases are not fresh execution merely because they exist or
compile. Future parity identifies exact scene bytes/version/digest, evaluation
time/density, order/arithmetic and recording profile, or visibly refuses that
capability. No source review changes earlier physical-validation failures.

## 1. Source map and numerical ownership

| Owner | Actual state or operation | Reference boundary |
| --- | --- | --- |
| [geometry.rs](../../../src/geometry.rs) | Validated counts, origin, spacing, staggered indexing and managed-payload plan | Integer and physical-coordinate checks precede field allocation. |
| [simulation.rs](../../../src/simulation.rs) | Accepted/candidate velocity and tracer, physical clock, generation, step controls, final publication | Primary full sealed-box carrier oracle. Scratch and accepted state have separate ownership. |
| [sampling.rs](../../../src/sampling.rs), [advection.rs](../../../src/advection.rs) | Component-aware interpolation and midpoint backtracing of velocity/passive tracer | Bounded, clamped semi-Lagrangian transport; no conservative momentum or tracer-mass guarantee. |
| [operator.rs](../../../src/operator.rs) | Integrated pressure operator, flux RHS, correction, physical divergence | Coefficient/sign/boundary authority; changing only a pressure algorithm must preserve these equations. |
| [pressure.rs](../../../src/pressure.rs) | PCG recurrence, true residual qualification, Jacobi/SGS preconditioners | A pressure report qualifies pressure before stored f32 velocity correction. |
| [forces.rs](../../../src/forces.rs), [emitter.rs](../../../src/emitter.rs) | Explicit body force, prescribed source/nozzle controls | Acceleration and force density have distinct units; source heads do not create a solved nozzle interior. |
| [viscosity.rs](../../../src/viscosity.rs) | Explicit coupled symmetric strain on filled MAC domains | Separate opted-in workspace and stability contract; no general free-surface or imported-mesh stress boundary. |
| [parallel.rs](../../../src/parallel.rs), [scheduling.rs](../../../src/scheduling.rs), [sgs.rs](../../../src/sgs.rs) | Joined disjoint writes, adaptive CPU batching, dependent SGS sweeps | Execution decomposition; no ownership of timesteps or accepted physics. |
| [liquid_volume.rs](../../../src/liquid_volume.rs), [liquid_step.rs](../../../src/liquid_step.rs) | Cell fraction transfers and joint carrier/fraction publication | Conservative represented volume does not automatically change carrier inertia or pressure topology. |
| [free_surface.rs](../../../src/free_surface.rs), [column_surface.rs](../../../src/column_surface.rs), [column_mac.rs](../../../src/column_mac.rs) | Resolved slab/column domains and fixed flat liquid-mass bridge | Restricted surface representations, each with admission/refusal rules. |
| [static_obstacle.rs](../../../src/static_obstacle.rs), [obstacle_pressure.rs](../../../src/obstacle_pressure.rs), [obstacle_shear.rs](../../../src/obstacle_shear.rs) | Clipped single aligned box, component-aware pressure, separate reduced shear | Does not turn ordinary `Simulation` into an arbitrary triangle-mesh fluid solver. |
| [particle_fluid.rs](../../../src/particle_fluid.rs), [material.rs](../../../src/material.rs) | Particle SI state, reference-volume SPH, EOS and pair forces | Separate bounded preview with no call to the grid pressure provider. |
| [collision.rs](../../../src/collision.rs), [mesh_collision.rs](../../../src/mesh_collision.rs), [mesh_import.rs](../../../src/mesh_import.rs) | Surface queries versus particle-center static contact/import | Contact geometry and fluid pressure/shear boundary assembly are different capabilities. |
| [session.rs](../../../src/session.rs), [material_session.rs](../../../src/material_session.rs), [pressure_session.rs](../../../src/pressure_session.rs) | Run identity, immutable inspection, quotas/deadlines, synchronous step ownership | Cooperative cancellation; no hard realtime guarantee or implicit restart service. |
| [recording.rs](../../../src/recording.rs), [carrier adapter](../../../src/recording/carrier.rs), [material adapter](../../../src/recording/material.rs) | Native accepted publications in SQLite | Typed inspection/interchange with exact sample timing; incomplete integrator state. |

## 2. Grid representation, arithmetic and memory

For counts `(Nx, Ny, Nz)`, `GridGeometry` stores cell-centered pressure/tracer
locations and three separate face lattices:

| Field | Shape | Physical offset from origin, in cell widths | Native scalar |
| --- | --- | --- | --- |
| x velocity | `(Nx+1, Ny, Nz)` | `(0, 1/2, 1/2)` | f32 |
| y velocity | `(Nx, Ny+1, Nz)` | `(1/2, 0, 1/2)` | f32 |
| z velocity | `(Nx, Ny, Nz+1)` | `(1/2, 1/2, 0)` | f32 |
| tracer | `(Nx, Ny, Nz)` | `(1/2, 1/2, 1/2)` | f32 |
| pressure and Krylov arrays | `(Nx, Ny, Nz)` | `(1/2, 1/2, 1/2)` | f64 |

Flat storage is x-fast: `i + Nx*(j + Ny*k)`, with each face array's own
dimensions substituted. Singleton dimensions are supported, including a single
cell; zero counts, integer/capacity overflow, invalid positive spacing, nonfinite
origin and unrepresentable half-cell coordinates are rejected. `at_cell_density`
changes spacing, not counts. UI axis/density maxima do not guarantee allocation.
These details are explicit in [geometry.rs](../../../src/geometry.rs) and
[geometry contracts](../../../tests/geometry_contract.rs).

`Simulation` owns two complete `Fields` sets. The candidate supplies advection
output and is later corrected in place; pressure correction needs no third
velocity set because each face reads its own prior value and immutable pressure.
`PressureWorkspace` owns six binary64 cell arrays: pressure, RHS, residual,
direction, operator product and preconditioned residual. Its SGS parallel variant
substitutes 64-bit atomic bit storage for the last array, preserving eight bytes
per value. These atomics carry stored bits, not floating-point atomic arithmetic.

With `N=Nx*Ny*Nz` and `F` the sum of the three face counts, the baseline planned
payload is `8*F + 8*N + 48*N` bytes: paired f32 velocities, paired f32 tracer and
six f64 pressure arrays. Actual retained `Vec` capacities are checked against the
budget. This excludes allocator metadata, stack, outputs, caller objects and
process RSS. Optional boundary, viscosity, provider, fraction, recording and
rendering owners add separate allocations. A GPU estimate must additionally
account for device scratch, transfers, staging, command/runtime allocations and
simultaneously retained host copies; the CPU payload is no total-memory promise.
See [BufferPlan](../../../src/geometry.rs), [pressure allocation](../../../src/pressure.rs)
and [simulation capacity checks](../../../tests/simulation_contract.rs).

Interpolation and physical coefficients use f64, then transport/correction
write f32 velocity/tracer. Pressure reductions use a fixed serial compensated
sum. A future change in reduction tree, fused arithmetic or storage precision can
change convergence and gate outcomes even if its mathematical expression matches.
The current CPU parallel source aims to preserve numerical order and tests exact
bits across thread counts; that is a useful CPU regression oracle, not an existing
GPU bitwise-equivalence guarantee.

## 3. Actual accepted carrier step

The authoritative sequence is `Simulation::prepare_impl`, followed by the
commit owner in [simulation.rs](../../../src/simulation.rs). Not every API enables
every optional stage.

1. Invalidate the prior interval-pressure inspection stamp. Validate pause state,
   positive finite requested dt, source/force/emitter inputs, workspace identity
   and supported combinations. Providers are refused with a surface or prescribed
   box-flux boundary on this path. Check generation/time representability.
2. Reduce dt using accepted component-speed bounds and source/external/emitter
   acceleration bounds. The velocity rate is `sum_d(max|u_d|/h_d)`; active bounds
   use half the configured maximum Courant allowance. Emitter motion/turbulence
   can shorten dt further. The accepted report stores actual dt, which may be
   shorter than the request.
3. Advect each accepted velocity component into its own staggered candidate
   lattice with midpoint/RK2 backtracing and bounded trilinear sampling. Set
   normal outer-box faces exactly to zero. A surface variant clears inactive
   faces. The backtrace clamps both midpoint and departure to the physical box.
4. Apply legacy localized vertical smoke acceleration, then any prescribed
   emitter, then explicit external body forces. Empty external force slices
   omit that stage. Optional filled-domain viscosity follows force application
   and precedes pressure.
5. Build/project the candidate using the ordinary pressure operator, the selected
   qualified provider path, or a distinct prescribed-flux projection workspace.
   Store the corrected velocity as f32. Column variants may extend one air band.
6. Recompute actual physical divergence from stored corrected velocities and
   enforce `actual_divergence_limit`. Separately enforce the actual corrected
   Courant bound. Neither is replaced by the pressure solver's predicted value.
7. Advect old passive tracer with corrected candidate velocity, optionally with
   its explicit static tracer barrier. Apply admitted boundary tracer policy and
   source/emitter tracer. Check finite values and concentration in `[0,1]`.
8. Form finite reports (including tracer integral and kinetic energy), visit the
   final cancellation checkpoint, then swap candidate/accepted fields and update
   clock/generation. A pressure-success stamp is attached only by the admitted
   sealed-box recording entry points.

[Sampling](../../../src/sampling.rs) clamps interpolation to donor extrema;
[advection](../../../src/advection.rs) explicitly disclaims passive-tracer mass
conservation. Midpoint trajectory evaluation does not prove second-order accuracy
of the complete split method, especially when clamping is active. Tracer's
integral is a diagnostic, not an enforced conservation residual. Kinetic energy
reported by this carrier uses its discrete face weights, not a complete
continuum energy budget.

[Forces](../../../src/forces.rs) sample a constant vector on each component's
own face positions, skip fixed normal walls, and use half-open world-space
supports. Acceleration has units m/s²; force density N/m³ is divided by constant
rho. The work report uses the actual stored f32 velocity change and covers the
external-force stage. The legacy vertical source is prescribed acceleration,
not a coupled temperature/buoyancy equation. Source/nozzle pressure heads are
prescribed controls, not proof of a solved pipe or general traction boundary.

## 4. Pressure equation, gauge and preconditioning

The [operator](../../../src/operator.rs) applies the integrated symmetric matrix
`K = B diag(A/(rho*d)) B^T`. On the uniform sealed box, an interior face in
direction d has `w_d = face_area_d/(rho*h_d)`; a row sums
`w_d*(p_i-p_neighbor)` over available neighbors. Its RHS is negative outward
predicted volume flux divided by dt:

`b_i = sum_d A_d*(u_lower_d-u_upper_d)/dt`.

Pressure correction is
`u_new = u_star - dt*(p_positive-p_negative)/(rho*h_d)`.
Pressure is Pa; the integrated matrix product and RHS are m³/s². Coefficient
construction checks positive finite density, areas and weights. Ordinary sealed
walls require exactly zero normal velocity. Matrix application is ungauged;
PCG fixes cell zero and eliminates its search degree of freedom.

The [pressure workspace](../../../src/pressure.rs) checks sealed RHS compatibility
using compensated sums: `abs(sum b) <= 64*epsilon_f64*sum(abs(b))`. It does not
silently subtract the mean of incompatible forcing. Every solve starts pressure
from zero; the current baseline has no pressure warm-start history. The fixed
box has one gauge, not an arbitrary obstacle-component gauge service.

The two preserved implementations are `jacobi-pcg-v1` (default) and
`sgs-pcg-v1`. They use the same operator, pressure units, gauge, stopping criteria
and physical acceptance. Jacobi divides by the diagonal. SGS applies symmetric
forward/backward triangular solves corresponding to
`M=(D+L) D^-1 (D+L)^T` on the gauge-eliminated SPD system. It is not an
independently chosen one-way Gauss–Seidel pressure solve or multigrid method.

For each solve, the residual norm threshold is
`max(absolute_residual, relative_residual*||b||_2)`. A recursive residual can
trigger a candidate check, but acceptance requires a freshly evaluated full
`b-Kp`, including the gauge row. Both its L2 threshold and its predicted maximum
divergence must pass. For full uniform cells, predicted divergence is
`dt*max|b-Kp|/cell_volume`; flat-column masses use their row volumes. A rejected
recursive candidate with excessive true residual replaces the residual and
restarts direction. Physical-divergence rejection alone does not gratuitously
discard conjugate progress. Nonpositive/nonfinite curvature, invalid recurrence,
iteration exhaustion and cancellation return explicit errors.

Pressure qualification happens before f32 correction. `actual_divergence_max`
then evaluates the stored field independently. A GPU pressure comparison that
checks only pressure L2 or recursive residual misses the core acceptance contract.
Useful source oracles include the independently assembled rectangular operator,
hand chain, eliminated-gauge residual, three-dimensional manufactured pressure,
random f32 projection and retained circulation cases in
[operator contracts](../../../tests/operator_contract.rs) and
[pressure contracts](../../../tests/pressure_contract.rs). The manufactured
discrete tests are not automatically independent continuum-refinement evidence.

The optional [provider API](../../../src/solver_provider.rs) is a candidate
generator for the sealed constant-density operator. The host reconstructs its
quality checks and can invoke reference fallback. A provider-backed accepted run
does not prove the candidate itself succeeded. Future GPU measurements must
identify attempted, rejected, selected and fallback execution and include the
appropriate transfer/qualification/correction scopes. Existing model-provider
metadata does not authorize arbitrary operator or backend substitution.

## 5. Dependency order, cancellation and publication

[CPU parallel helpers](../../../src/parallel.rs) split disjoint mutable scratch,
join each operation before its consumer and retain caller-thread cancellation
callbacks. Larger grid vector kernels use Rayon; small work stays serial.
Particle density/force kernels have a lower threshold because each item performs
neighbor work. [Scheduling](cpu-scheduling-contracts.md) adjusts batch sizes from
joined invocation timings. It changes decomposition, not physical timestep or
equations, and its feedback is not a repeatable speedup result.

[SGS](../../../src/sgs.rs) has actual cross-row dependencies. Eligible large
workspaces store the preconditioned values through atomic f64 bits and traverse
tiles by `bx+by+bz`, joining every wave; the reverse triangular solve reverses
the dependency order. It calibrates 8³ and 4³ tile sweeps with scratch-only passes
and falls back for one worker or thin shapes. A GPU implementation that launches
all rows independently would change this preconditioner. Serial compensated
scalar reductions and the local per-cell arithmetic order remain separate
constraints. CPU tests compare tiled and original serial SGS bits; no GPU
execution is implied by those tests.

`StepStage` exposes pre-advection, velocity slices, force, viscosity, pressure
iteration, projection-related, tracer and precommit stages. Availability differs
by path: ordinary in-place correction has no per-slice cancel callback, whereas
the box-flux projection has correction/acceptance callbacks. A long synchronous
kernel or one SGS preconditioner invocation cannot be asynchronously preempted
through these callbacks. Deadline checking is cooperative and has no maximum
wall latency guarantee. See [session](../../../src/session.rs),
[pressure session](../../../src/pressure_session.rs),
[simulation cancellation contracts](../../../tests/simulation_contract.rs) and
[parallel contracts](../../../tests/parallel_contract.rs).

An error may partially overwrite scratch. It must leave accepted fields, time
and generation intact; the next attempt overwrites candidates. That guarantee
does not promise unchanged scratch, scheduler calibration, observation counters
or the previous interval-pressure stamp. The latter is invalidated on a later
attempt even if the later attempt fails. `reset` zeros fields/time and increments
generation; it is not a newly constructed run. A future GPU boundary must join
or otherwise establish completion before reads and must prevent late writes or
device errors from reaching accepted state after cancellation/publication.

## 6. Boundaries, liquid components and collision distinctions

| Path | Implemented contract | Limit that remains relevant to a GPU oracle |
| --- | --- | --- |
| Ordinary `Simulation` | Fixed Cartesian box, impermeable zero normal faces, constant rho | General imported-mesh fluid pressure, viscous no-slip and moving-body load coupling are absent from this step. |
| [Box flux](../../../src/box_flux.rs), [step integration](../../../src/box_flux_step.rs) | Explicit prescribed normal boundary speeds, compatibility, pressure-work and tracer-policy ledgers | A separate boundary mode, not arbitrary inlet/outlet stress selection. |
| [Slab free surface](../../../src/free_surface.rs) | Resolved wet/air interface, atmospheric zero pressure at half-cell distance | Restricted cell-aligned wet slab; not arbitrary surface topology, air inertia or capillary stress. |
| [Reconstructed columns](../../../src/column_surface.rs) | Lower-wall-connected column heights and ghost-fluid distances; unsupported scales reject | No general thin/disconnected interface, overhang or arbitrary interface momentum formulation. |
| [Liquid volume](../../../src/liquid_volume.rs) | Shared signed donor face transfers, source/inlet/outlet ledgers, outward Courant and fraction bounds | Conserves represented volume; numerical interface diffusion remains. Ordinary all-fluid carrier inertia persists even in zero-fraction cells. |
| [Filled viscosity](../../../src/viscosity.rs) | Explicit symmetric Newtonian strain, normal/shear cross-component derivatives, stationary free-slip outer box | Dynamic mu becomes nu=mu/rho; explicit bound `nu*dt*sum(1/h_d²)<=1/4`. Partial/surface/mismatched-density domains refuse on this integration path. |
| [Static obstacle](../../../src/static_obstacle.rs) | One admitted closed axis-aligned box, exact clipped cell/face measures and connected-fluid labels | Refuses unresolved cell topology. Separate pressure/reduced-shear owners do not compose a general imported mesh/liquid step. |
| [Surface query](../../../src/collision.rs), [tracer barrier](../../../src/tracer_barrier.rs) | Triangle earliest-hit/clipping and explicit tracer sampling barrier | Barrier tracer transport leaves ordinary grid velocity/pressure fixed-box physics unchanged. |
| [Particle mesh](../../../src/mesh_collision.rs) | BVH static center sweeps; two-sided open surface or connected watertight consistently wound convex solid | Contact is not an SPH pressure/shear boundary or body solver; no normal mesh motion, torque or two-way rigid-body dynamics is created. |

Column shear/momentum and the [flat MAC bridge](../../../src/column_mac.rs),
[fitted-height geometry](../../../src/fitted_height.rs),
[translated viscous flow](../../../src/translated_viscous.rs),
[fixed-bottom ALE](../../../src/fixed_bottom_ale.rs) and
[coupled discrete flow](../../../src/coupled_discrete.rs) supply bounded research
mechanisms. Their topology, velocity families and parameter restrictions must be
carried into any comparison. Adding an extruded third velocity component does
not establish arbitrary three-dimensional geometry or flow.
The opt-in [aligned Stokes](../../../experiments/aligned_stokes.rs),
[submarine Stokes](../../../experiments/submarine_stokes.rs) and
[plate coupling](../../../experiments/plate_coupling.rs) experiments solve different
restricted problems. They are not the default carrier or a general FEM backend.

## 7. Particle preview and display voxels

[ParticleFluidSimulation](../../../src/particle_fluid.rs) owns accepted/candidate
`FluidParticle` arrays with f64 position, velocity, mass, temperature and material
identity, plus thermodynamic/force/diagnostic arrays and deterministic linked-cell
neighbor bins. `GridGeometry` supplies the domain and observation coordinates;
its existence does not turn the particles into a MAC pressure grid.

The actual stages are admission and dt bounds; replayable emission into a
candidate; neighbor-bin construction; reference-volume density and EOS;
pressure/viscosity/cohesion plus gravity; stability screening; symplectic Euler
velocity then position; box-wall and optional static mesh response; diagnostic
capture and final swap. Liquid pressure uses reduced-stiffness positive
compression, gases an ideal-gas EOS; vacuum owns no particles. Density uses the
poly6 kernel and reference volumes, with an explicit 0.05 volume-density floor.
The pressure force differentiates that same poly6 compression construction;
viscosity/cohesion have their own bounded pair formulas. These details are model
choices that any particle GPU comparison must retain or explicitly requalify.

Up to sixteen stability retries replay from accepted state with shortened dt;
failed attempts retain source credits, emission sequence, accepted particles and
clock. Density/force evaluations are disjoint per particle and read the fixed
neighbor structure, so CPU parallelism does not require unordered atomic pair
accumulation. Imported contact resolves particle-center sweeps after integration;
pair neighbor interactions are not partitioned by a thin barrier. Box-wall
clamping/normal stopping, tangential drag and heat exchange do not establish
general wall stress or resolved wetting physics. See
[material independent reference tests](../../../tests/material_reference_contract.rs)
and [mesh contracts](../../../tests/mesh_collision_contract.rs).

The committed density, pressure and acceleration describe post-emission,
pre-integration evaluation at the old interval time. Accepted position/velocity
are at the new time, after wall response. Acceleration excludes that response.
The [material session](../../../src/material_session.rs) exposes the distinction
and explicitly reports `grid_pressure_provider=false`. Treating diagnostic
pressure as synchronized endpoint particle pressure would compare different
states. The [general-fluid audit](../../audits/general-fluid-20261019/report.md)
records failures of its particle physical-rest screens; pair/transactional
correctness does not establish continuum or experimental water accuracy.

[PreviewFrame::capture_particles](../../../src/bin/desktop/history.rs) separately
paints emitted particles into u8 appearance voxels with a radius and max-composite
weight. Surrounding material still participates in physics even though this
preview highlights emitted material only. Carrier appearance can also be display
resampled. The [viewport](../../../src/bin/desktop/viewport.rs) extracts visible
voxel surfaces with rendering caps, and [particle view](../../../src/bin/desktop/particle_view.rs)
uses a bounded point cache. Neither creates numerical samples. Changing preview
edge, point stride, camera or surface count does not refine particles or the MAC
solver. Orange worker boxes are sampled work diagnostics, not a solver mesh or
complete utilization trace. GPU rendering speed and GPU solver speed require
separate evidence.

## 8. Recording, inspection, restart and checkpoints

The [accepted-publication contract](../../api/recording.md) and its source adapters
are the authority for what survives producer exit. `StepRecorder` stores one
publication and its contiguous native field BLOBs in one SQLite transaction,
with x-fast dimensions, association, scalar type, units, stage and optional
evaluation time. Canonical carrier construction has four f32 fields; an admitted
accepted sealed-box step has those plus f64 interval pressure with cell-zero
gauge. That pressure spans the actual previous time to publication time. It is
neither absolute atmospheric nor endpoint pressure.

Canonical material recording includes accepted particle fields and their earlier
pre-integration diagnostics. Recorded source/config/control text and revisions
are provenance, not an executable stable replay codec. Mesh bytes, evaluation
positions, per-contact impulses, all source/integrator internals, pressure/Krylov
scratch and complete backend state are absent. `RecordingReader` can verify and
select a recorded publication; it cannot reconstruct complete continuation from
it. Selection must not substitute the live latest state or fabricate missing
fields. Native field bit preservation is an inspection contract, not restart
equivalence or authenticated source provenance.

Recording follows numerical acceptance and precedes its corresponding desktop
appearance publication. Cancelled/rejected candidates add no ordinal. If physics
has accepted before cancellation, synchronous recording may finish before the
terminal cancellation report. A recording error cannot roll back accepted
physics; an unrecorded accepted tail can remain. Append failure poisons further
append admission because commit outcome may be uncertain. A fresh reader must
reconcile durable contents; acknowledged counts do not prove a failed append's
outcome. An `active` cold archive is incomplete/unknown, and destruction does
not mark a run complete. Capacity is keep-all/stop-at-limit, with no automatic
eviction, precision reduction or writer resume. See
[recording source](../../../src/recording.rs),
[carrier contracts](../../../tests/recording_carrier_contract.rs) and
[generic recording contracts](../../../tests/recording_contract.rs).

The name `CompressibleCheckpoint` in [compressible.rs](../../../src/compressible.rs)
means a cooperative cancellation location (stage, attempt and optional cell),
not a serialized restart checkpoint. The separate compressible implementation
is a serial periodic cell-centered ideal-gas Euler/LLF solver with five conserved
averages, positivity handling and retries. Its conservative state, EOS and
boundary conditions are different from the incompressible MAC carrier and SPH
preview. GPU work must not infer a shared checkpoint/backend contract from a
type name or from all three models having three spatial coordinates.

## 9. CPU oracle hierarchy and future validation gates

The following are proposed gates for future authorized backend work. Existing
test source supplies reference cases; no row claims a fresh pass here. A higher
row does not make a lower row unnecessary.

| Gate | CPU/independent oracle and required comparison | What passing would establish |
| --- | --- | --- |
| 0. Input and layout | [Geometry](../../../tests/geometry_contract.rs), [sampling](../../../tests/sampling_contract.rs): rectangular and singleton shapes, own-axis offsets, checked bounds/overflow, finite inputs | Correct indexing/representation and refusal behavior for the declared domain. |
| 1. Local operators | [Operator](../../../tests/operator_contract.rs), in-source SGS triangular hand assembly, [force](../../../tests/forces_contract.rs), [strain](../../../tests/viscosity_contract.rs) references | Signs, units, coefficients and stage equations agree with independently constructed small systems. |
| 2. Transport | [Advection](../../../tests/advection_contract.rs), clamping/donor-range and sparse-versus-dense cases | The selected transport algorithm is reproduced, including boundaries; no conservation claim follows from bounded interpolation. |
| 3. Pressure | [Pressure](../../../tests/pressure_contract.rs), both preconditioners, true full residual, gauge, compatibility, breakdown/exhaustion, actual corrected f32 divergence | Numerical pressure acceptance and its stored-field consequence match declared tolerances. Iteration count alone is insufficient. |
| 4. Stage and full step | [Simulation](../../../tests/simulation_contract.rs), [force cycle](../../../tests/forces_contract.rs), [parallel](../../../tests/parallel_contract.rs): identical accepted inputs/controls/dt with all enabled stage fields/reports | Correspondence of the implemented split carrier and acceptance order. Record CPU/GPU arithmetic differences explicitly. |
| 5. Failure lifecycle | Every reachable cancellation stage, invalid input, solver refusal and retry; [session](../../../tests/session_contract.rs) and [material session](../../../tests/material_session_contract.rs) quotas/deadlines | Accepted-state/clock/generation integrity and no delayed write after rejection. No hard realtime guarantee. |
| 6. Model-specific composition | Separate [volume](../../../tests/liquid_volume_contract.rs), [liquid step](../../../tests/liquid_step_contract.rs), [surface](../../../tests/free_surface_contract.rs), [obstacle](../../../tests/obstacle_flow_contract.rs), [material](../../../tests/material_reference_contract.rs), [mesh](../../../tests/mesh_collision_contract.rs) matrices | Only the specifically enabled physical/domain model is qualified; unsupported combinations must remain refused. |
| 7. Publication | [Carrier archive](../../../tests/recording_carrier_contract.rs), [generic archive](../../../tests/recording_contract.rs), field timing/type/association and cold selection | Inspection keeps actual accepted values with correct lineage and uncertainty semantics; no restart claim. |
| 8. Numerical/physical validation | Independent continuum manufactured solutions, spatial/temporal refinement, conservation/work ledgers and relevant experimental comparisons with uncertainty | Accuracy for the tested equations/regime, separately from CPU agreement. Existing audit findings and failures must remain visible. |
| 9. Hardware/performance | Future declared device/runtime/compiler/precision, memory peaks and complete stage/full-step timings including transfers, validation and fallback | Qualified measurements for that environment and workload, separately from correctness. No performance result currently exists in this inventory. |

For gate 8, avoid constructing every expected value through the production
operator: CPU/GPU agreement can preserve the same assembly defect. Distinguish
an exact discrete manufactured field from a continuum source assembled
independently and refined with geometry. Exact Lean identities, regression
contracts, recorded field integrity, graphical plausibility and experimental
validation answer different questions. The existing
[audit protocol](../../audits/general-fluid-20261019/protocol.md) exemplifies
predeclared acceptance criteria and independent continuum sources; its bounded
results do not grant a general-fluid qualification.

## 10. Book entry points and evidence limits

The source-only review also inspected the publication entry points. The
[book README](../README.md) identifies companion experiments/depth experiments
and figure generation before the native-equation Word builder. Those scripts
are scientific execution, not passive Markdown formatting. The
[Word builder](../build_manuscript.py) explicitly selects chapter and appendix
Markdown, builds equations through Pandoc and emits DOCX/manuscript outputs;
implementation notes are not automatically in that selection.

The [education builder](../../education/build.py) renders every chapter, appendix
and `implementation/*.md` page, but constructs its print/reading sequence using
the explicit `GUIDES` list from [native_sequence.py](../../education/native_sequence.py).
A new implementation file alone therefore supplies a web page while needing a
separate reading-order integration to enter that print sequence. The builder
validates source-bound scientific packets/reference receipts and creates portable
Markdown/source downloads. [PDF freshness](../../education/pdf_freshness.py)
binds source inputs, and [browser qualification](../../education/verify_browser.py)
renders/qualifies the PDF and interactive behavior.

The [full research-book workflow](../../../.github/workflows/research-book-pages.yml)
executes numerical references, native contracts/examples, scientific packet
qualification, browser/Wasm work and render/publication checks; it is not a
safe substitute for a source-only documentation check under this task's scope.
The [reviewed publication workflow](../../../.github/workflows/research-book-reviewed-pages.yml)
is another publication entry point and likewise is not evidence of a local run.
No full build was attempted here. Inclusion of a note must not be described as
successful PDF/Word/browser delivery until those actual outputs and freshness
checks exist. Generated outputs remain outside Git under the
[repository output policy](../../REPOSITORY_OUTPUTS.md).

Remaining limits are concrete: no implemented GPU oracle comparison, no device
error/cancellation qualification, no GPU memory/performance evidence, no unified
general mesh/free-surface stress integrator, no restart codec for the recorded
carrier/material path and no new physical validation from this research. These
are boundaries for subsequent authorized work, not failures concealed by a
successful source inventory.
