# 39 GPU integration, validation and an implementable roadmap

This chapter is prospective research for Puma's Rheon book, dated 21 October
2026. No GPU solver, supported-device claim, numerical qualification or timing
result is introduced. Read the [CPU source inventory](../implementation/gpu-cpu-oracle-inventory.md),
[GPU algorithms](37-gpu-grid-algorithms.md) and [independent backend paths](38-independent-cuda-and-vulkan.md)
first. The inventory freezes the inspected CPU baseline at
`fd7c0f0aee63bc8710df99f9bb01154e7619b974`; future implementation must refresh
that baseline rather than infer compatibility from this date.

## Start with the actual oracle

Rheon's first GPU target should be its sealed, constant-density, filled MAC box.
The initial comparison owner is `Simulation`, `PressureOperator` and
`PressureWorkspace`, including their actual f32 velocity writes and f64 pressure
arithmetic. Jacobi-preconditioned PCG is the first reference method because its
preconditioner has no triangular dependency. Symmetric Gauss-Seidel PCG remains
a second CPU cross-check. Its ordered sweeps must not become an unordered GPU
loop that is still labeled the same algorithm.

The CPU oracle is a tuple, not the name “CPU”: source commit, toolchain/features,
grid lengths and spacing, density, initial accepted fields, source/force sequence,
time increments, solver implementation, pressure settings, corrected-divergence
and Courant gates, boundary mode, recording profile and expected failure class.
Retain the exact input bytes and grid association of each field. A future run
receipt must identify their hashes and the backend's arithmetic policy. Existing
historical audit measurements are neither fresh CPU runs nor GPU baselines.

There are three independent comparisons. A differential check asks whether a GPU
implements the admitted discrete CPU problem. An independent analytic, rational,
dense or manufactured reference checks whether that problem and its assembly
are correct for a stated case. A refinement study checks behavior as spatial,
temporal and iterative errors are independently reduced. Agreement between two
backends can preserve a common bug; disagreement can reflect legal floating-point
order. Neither observation alone decides physical accuracy.

The existing [pressure provider](../../../src/solver_provider.rs) is a synchronous,
host-slice candidate API for sealed boxes. It can support a later diagnostic
pressure-only prototype if its transfer cost and provenance are explicit. It is
not already an interface for a resident whole-step GPU solver. In particular,
copying host RHS and pressure every invocation does not qualify the claim that
the fluid state remains on the GPU. Adding backend names to provider metadata
cannot admit obstacle, free-surface, variable-density or viscous equations.

## One accepted state, several execution mechanisms

The proposed semantic contract is shared mathematics and lifecycle. CUDA and
Vulkan independently implement it with their own allocations, execution objects,
capability checks and build dependencies. A CPU-only build, a CUDA-only build and
a Vulkan-only build must remain distinct testable configurations. Installing or
initializing CUDA must not be a prerequisite for Vulkan compute. Vulkan graphics
interop is optional to CUDA computation; it cannot become a requirement merely
because the desktop renderer uses a graphics API.

Each numerical owner continues to select its fields, associations, geometry,
boundary conditions and acceptance gates. A backend owns the representation and
execution of one explicitly admitted problem. In the current CPU path,
`Simulation` commits accepted time and generation; the session wraps that owner
with quotas and reports. A future resident numerical owner must preserve this
single commit authority; the recorder consumes accepted snapshots. The renderer consumes a
versioned projection of those snapshots. A display texture, pressure prediction
or completed dispatch is never an independent accepted-state authority.

The dependency view is conceptual pseudocode, not a new interface declaration:

~~~text
immutable accepted generation n + interval inputs + admitted capabilities
    -> backend-owned candidate allocation
    -> advection -> forces -> optional admitted viscosity
    -> RHS -> pressure solve with true-residual/predicted-divergence gates
    -> velocity correction -> stored-velocity divergence + interval gates
    -> tracer transport/source -> complete-step gates
    -> commit candidate generation n+1
    -> bounded immutable snapshot -> existing recording/render consumers
any invalid, cancelled or unsuccessful candidate -> retain generation n
~~~

An implementation may evaluate some reductions earlier or fuse independent work,
but it must preserve which field version a consumer reads. For example, tracer
transport using the predicted velocity instead of the accepted corrected velocity
changes the CPU algorithm. Recomputing a force at a different stage of a moving
field changes its time discretization. A faster graph with changed dependencies
is another method and needs a separate identity and qualification.

This separation also bounds maintenance. Changing pressure preconditioning touches
the numerical backend and its method/qualification identity, not archive time
ownership. Changing an archive codec belongs to recording, not the stencil. A
Vulkan barrier repair changes execution ordering without changing the pressure
equation. Adding physical free-surface traction changes the numerical contract
and must reach every implementing backend and its independent oracles. A field
layout or arithmetic change invalidates relevant kernel and restart evidence.

## Acceptance in physical units

Use the same full integrated operator as the admitted CPU problem. With cell
volumes \(V_i>0\), pressure RHS \(b\), matrix \(K\), and residual
\(r=b-Kp\), the compatible pressure/correction identity is

\[
 D u^{n+1}=-\Delta t\,V^{-1}r.
\]

This is an exact-arithmetic identity under the operator/boundary assumptions in
Chapters 3–4. The actual code separately checks the divergence of rounded stored
velocities. In the sealed graph, \(b\) and \(Kp\) have units m³/s² and divergence
has units s⁻¹. A small Euclidean integrated residual can hide a large cellwise
divergence when a volume is small. GPU qualification must retain the full
ungauged residual, compatibility of every sealed connected component, its gauge,
and the final stored-field gate. Checking only the reduced/gauged rows misses a
physical equation.

Define a pressure comparison only after aligning an allowed gauge. For a sealed
component one may subtract each field's own pinned-cell value, so both use the
same zero-at-pin convention. An
atmospheric or prescribed-traction boundary can fix absolute pressure; subtracting
an arbitrary mean would erase a real error. Likewise, a pressure relative norm
at a zero-pressure solution needs a stated absolute scale.

For any scalar field \(q\), useful proposed metrics are

\[
 E_{2,V}(q)=\sqrt{\frac{\sum_i V_i(q_i-q_i^{ref})^2}{\sum_i V_i}},
 \qquad E_\infty(q)=\max_i|q_i-q_i^{ref}|.
\]

For face velocity, replace cell weights by the admitted face inertia or dual
volume weights. Report components and locations separately; flattening dissimilar
units into one norm has no useful physical interpretation. Compare conservative
tracer/volume updates through their boundary/source ledger; ordinary sampled
appearance tracer is not a liquid-mass conservation claim.

An elementary mixed comparison uses

\[
 |q_i-q_i^{ref}|\le a_q+r_q\max(|q_i|,|q_i^{ref}|).
\]

Here \(a_q\) has q's units and \(r_q\) is dimensionless. The constants are
future, predeclared case requirements, not universal values supplied by this
chapter. They must account for discretization, iterative residual, storage and
reduction error while preserving the physical acceptance threshold. Derive them
from independent case scales and a frozen protocol before observing candidate
results. A weak CPU iteration budget is not a license to loosen a GPU gate.
When the CPU lies near a gate, compare error bounds and higher-quality oracles;
do not declare either implementation correct because it happened to pass.

Tolerance-based parity is the normal cross-device goal. Bitwise identity is a
separate, narrower promise requiring pinned compiler/shader versions, arithmetic
options, dispatch/reduction topology and device behavior. A fixed tree can support
repeatability on a qualified configuration; it does not prove identical results
across architectures, FMA policies, subgroups or compilers. If repeated runs
produce different acceptance outcomes, record the instability and reject that
configuration until the numerical margin and ordering are understood.

## Differential and independent checks

The following is a future validation matrix. No row is marked executed by this
documentation task. Tests should compare intermediate stages before long-run
differences amplify and obscure the first incorrect dependency.

| Layer | Input/problem and independent check | Required observations and negative cases |
| --- | --- | --- |
| Layout and bounds | Non-cubic grids with distinguishable x/y/z face values; analytic constant/linear sampling | Each face extent, x-fastest mapping, halo and boundary rule; tail workgroups, overflow and insufficient resource refusal |
| Operator | Tiny explicit incidence/dense matrix independent of kernel indexing | Full `Kp`, symmetry/work, nullspace, positive quadratic form on gauged subspace; closed face and disconnected-component cases only when admitted |
| Reductions | Signed dynamic-range vectors and independent high-quality accumulation | Dot, sum, max, finite flag and compatibility budget; NaN/Inf, negative curvature and reduction scratch exhaustion |
| Pressure | Fixed RHS and initial guess, analytic smooth modes and dense small solve | True full residual, gauge, iteration/failure class, pressure error and divergence identity; incompatible RHS, zero RHS and iteration limit |
| Correction | Predicted f32 MAC fields plus known pressure | Stored f32 corrected fields, exact boundary prescription, true divergence, pressure-work/kinetic ledger under its assumptions |
| Transport | Constant/linear samples, translation, midpoint backtrace and bounded source | Footprint and stage, extrema, trace error and domain escape; tracer integral is diagnostic unless flux-conservative transport is selected |
| Forces | Uniform acceleration, force density at two densities and restricted regions | Units, force sampling locations, source saturation and no write to forbidden boundary DOFs; invalid interval parameters |
| Viscosity | Existing admitted filled-box strain operator plus analytic decay/refinement | Tensor action, mass-weighted dissipation, correct dynamic/kinematic conversion and explicit stability guard; unsupported stress boundaries refused |
| Whole step | Frozen source/time sequence and actual CPU acceptance settings | Accepted time/generation, divergence, Courant, fields and diagnostics at each accepted step; every staged cancellation and rejected candidate |
| Persistence and presentation | Accepted snapshots and controlled writer failures | Exact field stages/units/identity, immutable render view, durable prefix and unrecorded tail; no failed candidate labeled accepted |
| Future geometry | Independently assembled apertures, volumes, connectivity and swept ledgers | Shared fluxes, traction, compatibility and force/work under stated model; no arbitrary mesh admission through collision-only metadata |

The independent references in Rheon's existing
[validation material](18-validation-program.md) and
[paper audit](../appendices/G-paper-audit-and-corrections.md) have bounded scopes.
For example, the fixed-column Poiseuille solution does not validate general
three-dimensional embedded-wall viscosity. Historical failed particle rest screens
and measured cross-wall-neighborhood limitations are retained and cannot be
“fixed” by a faster grid solver. The latter were documented model limitations,
not failed preregistered barrier screens. GPU validation adds a method comparison, not a new interpretation of
old receipts or proof hypotheses.

Refinement needs independent axes. Tighten linear tolerances until the measured
spatial/time error is insensitive to further solver tightening; vary time at
fixed sufficiently fine grid and grid at controlled time error. For an error
\(E(h)\), report the observed rate
\(p_{obs}=\log(E(h)/E(h/2))/\log 2\), the norm, reference, domain and regime.
No rate is defined by zero or noise-dominated errors. Mesh conditioning, interface
topology changes and small cut volumes require separate analysis. A bounded
field-comparison pass cannot establish general stability or convergence.

## GPU-resident state without losing transactional publication

The resident design keeps accepted and candidate state distinct. At the start of
a step, accepted buffers are read-only to candidate computation. On successful
completion, the session swaps or rebinds a complete candidate only after all
device writes and gates are visible. Diagnostic flags are cleared for each
attempt; device-side “converged” from an old generation is not success.

Device loss, allocation failure, illegal execution, nonfinite arithmetic,
iteration exhaustion and user cancellation all need closed outcomes. A cancelled
kernel is generally not a transaction rollback mechanism. Stop scheduling new
batches, await or otherwise safely retire in-flight work, discard candidate
publication, and preserve accepted state. Long kernels prevent prompt cooperative
cancellation. Use bounded dispatch batches and check the cancel/generation token
between them. A device-visible flag needs the backend's valid memory and
synchronization protocol; an ordinary host store is not a portable cancellation
channel. Benchmark the resulting latency overhead as a separate tradeoff.

Cancellation races have one owner. An observed cancellation at the final
decision checkpoint rejects the candidate; the subsequent commit has a declared
linearization point. The current CPU polls `BeforeCommit` and then commits
without another poll. A request racing after that observation may therefore
apply to the next interval; it cannot retroactively erase an accepted generation.
A future backend must declare its observation rule as precisely. A late readback from generation n must not overwrite a
display or recording result for n+1. Resource reuse requires completion tokens,
not just matching buffer dimensions. Graph/command-buffer reuse must invalidate
on geometry, descriptor, allocation or numerical-contract changes that affect
its bindings.

Checkpoint and inspection are different capabilities. A faithful restart needs
all state that affects the next admitted step: accepted velocity/tracer or
conservative fields, time/generation, physical/configuration identity, boundary
and geometry versions, source/random state, and any history variables. If warm
pressure is restored, identify its stage and arithmetic format; if it is omitted,
declare the resulting cold start and possible convergence/bitwise differences.
PCG scratch and in-flight commands need not be restart state if checkpoints are
allowed only between complete steps. A saved image or pressure-only profile
cannot be advertised as restart. This research does not change existing recording
schemas or upgrade their capabilities.

Readback should be scheduled by a bounded consumer policy. Small final acceptance
reports can cross each step; complete fields cross only for selected recording,
restart or inspection requests. An asynchronous staging ring overlaps copy and
later work but must retain each source generation until its copy completes. A
full ring needs an explicit choice: slow simulation, skip optional visualization,
or refuse a recording request. Silently dropping required accepted records is
not a performance optimization. GPU-resident execution cannot promise zero
transfer while a host archive requires every complete field.

The existing accepted-state and recording failure semantics in the
[inventory](../implementation/gpu-cpu-oracle-inventory.md) remain authoritative.
An archive failure after numerical commit creates an unrecorded accepted tail;
it does not roll physics back. A future recorder adapter must reconcile uncertain
durable commits through its existing owner. Backend failure must not fabricate a
recorded acceptance, nor should a renderer fence become a physics gate unless
the admitted product contract explicitly requires it.

## Benchmark what the user actually waits for

GPU parallelism is useful only if useful work amortizes dispatch, synchronization,
transfers and initialization. Measure the same admitted equations to the same
physical tolerance. A low-iteration Jacobi frame and a converged CPU PCG solve
are different workloads. Iteration counts are diagnostics; a multigrid cycle,
PCG iteration and Jacobi sweep are unequal units of work.

Predeclare workload families: small boxes exposing launch overhead; increasing
dense boxes approaching memory capacity; anisotropic dimensions; smooth and
high-frequency pressure RHS; warm and cold starts; force-rich intervals; and
only later, qualified sparse/cut geometry with controlled occupancy and topology.
Suggested size progressions are experiment inputs, not supported capacities.
Reject a configuration cleanly when its budget or descriptor/dispatch limits
cannot represent the workload. Never make an unsupported-device run part of a
geometric correctness claim.

| Measurement | Included boundary | Interpretation |
| --- | --- | --- |
| Construction/cold start | Device discovery, allocations, shader/kernel load or compilation, pipeline/graph construction, initial upload | User startup cost, reported separately from warmed steps |
| Device mechanism | Explicit event/timestamp interval around admitted work, with dependencies complete | Stencil, reduction, solve or full-step device time; not host latency |
| Submission and synchronization | Host preparation, submissions, waits, scalar convergence/readback | Exposes roundtrip or launch limits; includes rejected attempts |
| Accepted-step latency | Input preparation through final session commit and required acceptance data | Comparable numerical service time at equal acceptance conditions |
| Recorded-step latency | Above plus required copy, serialization and durable recorder completion | Application cost when recording is mandatory |
| Presentation latency | Accepted snapshot selection through renderer-visible completion | Includes interop/staging; separate from physical step rate |
| Memory | Accepted/candidate fields, solver/preconditioner, geometry, scratch, staging and optional render views | Peak allocated/resident budgets and host mirrors reported separately |

Use CUDA events or Vulkan timestamp queries only in their documented valid
queues and scopes; availability, timestamp precision and wrap must be checked.
A CPU wall timer around asynchronous dispatch measures submission unless it
waits for the relevant completion. Likewise, adding a global wait after every
kernel changes the performance being measured. Report both natural application
latency and scoped device timings without conflating them.

Record exact CPU/GPU models, driver/runtime/API versions, shader/compiler flags,
available memory, power/clock policy if controlled, numerical contract, output
cadence and other material environment facts. Disclose whether host/device copies,
pipeline creation, recorder work and rendering are included. Qualify each
independent backend on its own devices; a CUDA result is not Vulkan evidence and
an NVIDIA Vulkan result is not cross-vendor evidence.

Freeze warmup, repeated-sample and variability policy before measurement. Report
sample distributions, median and tail, observed iteration counts, accuracy and
failure rate. For successful comparable work, speedup would be
\(S=T_{CPU}/T_{GPU}\) with both timing boundaries named. No numerical value for S
is claimed here. Preserve rejected/failed attempts and capacity limits; measuring
only selected passing frames hides robustness and resource costs. Kernel tuning
must rerun relevant correctness and lifecycle gates at its new workgroup/tile
sizes, because a timing change can also expose races.

## Proposed stages, with explicit exit gates

These are candidate implementation milestones, not authorization to implement.
Each later stage needs a refreshed source audit, separately approved scope and
its own numerical/device evidence. CUDA and Vulkan may progress at different
rates while preserving the same contract IDs and explicit capability results.

| Stage | Deliverable and bounded target | Gate before claiming completion |
| --- | --- | --- |
| R0 — research | This source inventory, algorithms, independent backend mappings and validation protocol | Primary-source/technical review and nonnumerical document checks; no hardware claim |
| G0 — capability/resource skeleton | Independent optional build and device discovery for each backend; no production selection | CPU-only/CUDA-only/Vulkan-only configurations, deterministic unsupported outcomes, checked dimensions and full resource budget |
| G1 — operators | Dense sealed-box gather stencils, staggered sampling, reductions and f32 correction in isolated lab | Tiny independent matrix/index references, device memory/race diagnostics when authorized, negative arithmetic/resource cases |
| G2 — pressure | Resident Jacobi PCG, full true-residual/gauge/compatibility gates | Differential pressure and corrected-field checks, independent exact/dense cases, failure/cancel gates; no full-step claim |
| G3 — bounded full step | Existing fixed-box stage order, explicit force/source, tracer and existing admitted viscosity as separately qualified choices | One-step stage comparison, long-run/refinement and transaction tests; recording adapter/profile admission separately; equal-tolerance accepted-step benchmarks |
| G4 — scaling alternatives | Symmetric parallel preconditioner, multigrid, precision and tiling candidates | Original operator preserved, convergence/accuracy/resource comparison against G2; rejected alternatives retained |
| G5 — admitted geometry families | Existing bounded static aperture/volume owner, then separately approved liquid/interface/stress families | Geometry, per-component compatibility, work/flux, traction and continuum/temporal gates for each named family |
| G6 — general solver research | Moving cut cells, coupled free-surface stress, variable density, sparse topology, capillarity/contact and later coupling | Independent mathematical/discrete validation before GPU port; swept-volume, conservation, reaction/work and interface qualifications |

The existing canonical carrier recording profile is narrower than all of
`Simulation`'s numerical entry points: it admits its stamped `step` and
`step_with_emitter` lineage and rejects an unstamped accepted generation. A
force/viscous or GPU-resident path needs separately designed adapter/profile
qualification before claiming that recording capability. Passing existing
recording tests alone cannot admit a new numerical producer.

G5 must not import the particle preview collision model as hydrodynamic grid
topology. G6 is not a promise that arbitrary three-dimensional liquid/solid
physics exists on CPU or GPU. Compressible conservation laws use a different
state and wave/flux contract from this incompressible projection; Chapters 26–28
remain their research owner. Lagrangian FEM remains separately owned work.

The first actionable future slice is G0 plus a deliberately small G1 operator
lab after implementation authorization. It must not replace runtime selection,
archive or desktop behavior. Decisions to adopt an external kernel library,
driver binding, shader compiler, sparse grid or pressure package require exact
version/license/transitive and integration review. Reading open-source examples
does not admit their code into Rheon.

## When to stop and re-plan

Stop the affected method when a capability is absent, an admitted operator cannot
be represented, independent tests disagree beyond their declared uncertainty,
physical acceptance changes under reduction order, or cancellation/readback
cannot preserve state ownership within its resource limit. Report exact backend,
contract, action, device and known outcome. A CPU fallback is allowed only when
the session contract explicitly authorizes it and records actual selection,
rejection and transfer cost; it cannot silently make a failed GPU benchmark pass.

Re-plan the numerical problem when geometry changes adjointness or compatibility,
the coupled traction model needs an indefinite system, or a precision choice
cannot reach the fixed physical gate. Re-plan the execution strategy when
recording or scalar host roundtrips dominate, sparse metadata exceeds dense
storage, or the coarse multigrid solve serializes the workload. These are
observable decisions for later experiments, not reasons to promise a speedup
before one is measured.
