# 38 Independent CUDA and Vulkan compute backends

Research synthesis, 21 October 2026. This chapter proposes two **independent
compute methods** for Puma's Rheon solver: a CUDA backend and a Vulkan compute
backend. Neither backend wraps, loads, allocates through, compiles through, or
requires the other. Both implement a shared numerical and publication contract.
All proposed interfaces and schedules below are designs, not implemented GPU
support, hardware measurements, or a claim of scientific qualification.

Repository observations refer to main `fd7c0f0`. The
[CPU oracle inventory](../implementation/gpu-cpu-oracle-inventory.md) owns the
complete inventory; this chapter only extracts constraints that affect backend
design. [Chapter 39](39-gpu-validation-and-roadmap.md) owns prospective validation
and rollout. Official sources are identified by the `GPU-*` entries in the
[backend bibliography](../sources/gpu-backend-bibliography.md). Sources were
consulted online on 2026-10-21; moving-document revision limits are recorded
there. The live Vulkan specification identified itself as **1.4.365**, generated
2026-10-02 from commit `8c9361ba8180c1f4164c0bf79de2f6e817770b0d` [GPU-V00].
That reference version does not set the minimum runtime API requirement.

## A semantic interface, with two implementations

Separate **physics**, **algorithm**, **execution backend**, and **precision** in
method identity. For example, `sealed-mac / jacobi-pcg-v1 / cuda / f32-f64-v1`
and `sealed-mac / jacobi-pcg-v1 / vulkan / f32-f64-v1` are distinct execution
methods implementing the same declared discretization. A new multigrid
preconditioner or f32-only pressure solve changes a different axis. Sharing a
method label must never hide a changed operator or tolerance.

~~~text
                     numerical contract and test inputs
                          /                  \
       CUDA implementation                    Vulkan implementation
       CUDA capability discovery              Vulkan capability discovery
       CUDA allocations and modules           VkBuffer + VkDeviceMemory + SPIR-V
       kernels, streams, events, graphs        compute pipelines, queues, barriers
                          \                  /
                       qualified accepted-generation record
~~~

This is a dependency diagram, not an API adapter stack. An optional renderer
bridge attaches to either implementation after numerical acceptance. In
particular, a CUDA-to-Vulkan rendering bridge does not turn CUDA into a Vulkan
compute backend; the Vulkan solver must execute its own SPIR-V kernels without
CUDA installed.

The common host interface should describe inputs, ownership and completion,
without exposing either API's resource handles:

~~~text
probe(method_requirements) -> Supported(capability_record) | Unsupported(reason)
create_session(problem, numerical_policy, memory_budget) -> Session
submit_attempt(session, accepted_generation, controls) -> AttemptTicket
poll(ticket) -> Pending | Failed(reason) | Ready(bounded_report)
commit_ready(ticket, expected_generation) -> AcceptedGeneration
request_cancel(ticket) -> cancellation_requested
export_accepted(generation, export_budget) -> snapshot_ticket
~~~

These are proposed operations, not additions to the current Rust API.
`Ready` means device completion and all required acceptance checks have passed;
it does not itself publish candidate arrays. Tickets carry a run/session ID,
attempt ID, old generation, method ID and buffer epoch so a stale callback
cannot publish into a replacement session. Commit checks those identities and
the latest cancellation state under the same host-side synchronization used
for publication.

## The existing acceptance boundary constrains a GPU port

[`simulation.rs`](../../../src/simulation.rs) stores named-axis MAC velocity and
cell tracer arrays as f32; pressure settings, time and diagnostics use f64.
`Simulation::step` promises that error or cancellation preserves accepted
fields, time and generation. Candidate buffers and pressure scratch may change.
The public `StateView` borrows host slices, so retaining this exact interface
cannot automatically yield a GPU-resident simulation. A future GPU session
needs a distinct explicit snapshot/view contract rather than silently copying
every field whenever `state()` is called.

[`pressure.rs`](../../../src/pressure.rs) retains Jacobi-PCG and symmetric
Gauss-Seidel-PCG method identities, true residual checks and a separate physical
divergence criterion. The corrected **stored f32 velocity** has another actual
divergence gate in `simulation.rs`. A small recursive PCG residual does not
replace either true-residual qualification or the stored-velocity check.
Changing work scheduling does not authorize deleting those gates.

[`solver_provider.rs`](../../../src/solver_provider.rs) exposes a synchronous
host-slice `PressureProvider`: its request borrows an f64 RHS and its output
fills an f64 pressure slice. Adapting a GPU pressure candidate to this existing
boundary incurs RHS upload and pressure download, followed by host acceptance.
That can be a useful bounded experiment, but it is not a full resident solver.
It must be identified separately from a port that keeps advection, RHS assembly,
pressure iterations, correction and diagnostics on the device.

[`recording/provider/runtime.rs`](../../../src/recording/provider/runtime.rs)
explicitly states that request stamps and hashes are inspection evidence rather
than restart checkpoints or shutdown/preemption guarantees. Its accepted
context binds generation, interval, dt and prior pressure lineage. A GPU report
must preserve that distinction: completed kernel work, accepted simulation
state, recorded evidence and a durable restart file are different objects.

The proposed shared semantic contract therefore requires the following.

| Contract item | Required behavior in each independent backend |
| --- | --- |
| Geometry | Same named axes, staggered face counts, physical spacing, units, masks and boundary rules as the selected CPU problem |
| Operator | Same pressure coefficients, sign, volume weights, compatibility treatment and gauge; no implicit substitution of a different Laplacian |
| Precision | Explicit storage, arithmetic, accumulation and conversion policies; baseline follows f32 stored fields and f64 pressure arithmetic |
| Solver | Declared preconditioner, stopping criteria, maximum iterations and true-residual cadence; changing SGS ordering changes the method unless equivalence is established |
| Acceptance | Finite fields and scalars, valid pressure/gauge, true residual, physical divergence, stored-velocity divergence, Courant and applicable tracer/energy checks |
| Failure | Keep the old accepted generation, time and fields; invalidate candidate evidence and prevent publication |
| Publication | Advance fields, time, generation and associated report together after completion and qualification |
| Recording | Bind report and exported samples to the accepted generation and controls; do not call a pressure receipt a restart |

The table is a proposed cross-backend contract inferred from repository
semantics. It is not a list of implemented GPU guarantees.

## Resource residency and a budget that includes temporary storage

Let `C = nx*ny*nz`, and let `F = Fx+Fy+Fz` be the sum of the three MAC face
counts. Use checked host arithmetic for all products, padded extents, offsets,
dispatch sizes and byte counts before any allocation. A useful lower-bound
ledger for a sealed-box prototype is below; exact scratch depends on the
selected algorithm. The CPU pressure workspace has six f64 cell arrays, which
is a starting point for budgeting, not a requirement to copy its layout.

| Resource | Proposed residency and lifetime | Payload / additional accounting |
| --- | --- | --- |
| Accepted velocity and tracer `A` | Device; read-only throughout an attempt | `4*(F+C)` bytes |
| Candidate velocity and tracer `B` | Device; overwritten before each candidate use | Another `4*(F+C)` bytes |
| Advection/force/transport intermediates | Device; stage lifetime | Explicit method-dependent ledger; do not alias a still-read source |
| Pressure, RHS and Krylov/preconditioner scratch | Device; attempt/session lifetime | Six f64 `C` arrays would cost `48*C`; add any extra true-residual buffer |
| Geometry, masks, coefficients, optional hierarchy | Device; immutable or separately versioned | Account for materialized coefficients, halos and every multigrid level |
| Dot/norm partials and scalar state | Device; per attempt | All reduction levels, iteration counters, convergence/breakdown flags and report |
| Upload staging | Bounded host-visible storage | Initial fields, changed controls/geometry; retain until transfer completion |
| Diagnostic readback ring | Bounded host-visible storage | Fixed-size summaries plus separately requested bounded samples |
| Checkpoint/export staging | Bounded ring or tiled transfer | Reserve capacity before launch; include durable writer backlog |
| Rendering snapshot | Device; held by renderer until released | Separate buffer or explicit accepted-buffer lease; include images/mesh outputs |
| API/runtime overhead | Backend-owned host/device objects | Modules/pipelines, descriptors, command buffers/graphs, pools, alignment and allocator fragmentation |

The budget is more than logical array payload. For Vulkan, use memory
requirements and compatible memory types, not an assumption that a buffer's
requested byte size equals its backing allocation. Host visibility, device
locality and coherence are separate properties [GPU-V06]. For CUDA, explicit
device allocations and bounded pinned staging make ownership clear; a future
allocator pool must report reserved capacity as well as live payload. Start
with allocation outside the stepping loop rather than making a pooled allocator
a correctness prerequisite.

The normal step should upload only changed controls, leave intermediate arrays
resident, and read back a small report. Rendering and recording cadence must
not force a full field download at every pressure iteration. Unified physical
memory on some systems does not remove synchronization or ownership rules.
Oversubscription, migration, and heap exhaustion remain admission concerns;
neither backend may promise a fitting problem from grid dimensions alone.

## CUDA: a complete native execution path

CUDA kernels execute a grid of thread blocks and expose global/shared storage
with different lifetimes. A block barrier is local to a block [GPU-C01]. The
proposed implementation assigns contiguous x indices to neighboring threads,
uses separate component kernels when face extents differ, and bounds every
global index. First implement a straightforward global-memory stencil; shared
tiles and halo loads are later optimizations whose extra barriers must be
auditable.

Use an explicitly created nonblocking stream for the numerical sequence. CUDA
stream order supplies dependencies within that stream. Events and
`cudaStreamWaitEvent` express dependencies across streams; host polling must
distinguish pending completion from an error. Pinned host staging supports the
intended asynchronous copies [GPU-C02]. Avoid relying on default-stream
behavior, accidental host blocking, or a device-wide synchronize per kernel.

### CUDA kernel and dependency schedule

~~~text
initial upload -> event(initial_ready)
compute_stream waits(initial_ready)
  advect A.velocity -> velocity_tmp
  add forces/boundary treatment -> B.velocity
  assemble_rhs(B.velocity, geometry) -> rhs
  initialize_pressure_and_krylov(rhs) -> p, r, z, d, scalar_state
  repeat bounded chunks:
    apply_operator(d) -> q
    reduce(d*q) -> sigma; finalize alpha/breakdown
    update(p,r,alpha); precondition(r) -> z
    reduce(r*z and required norms) -> scalars; update d
    scheduled true_residual(p,rhs) -> qualification state
    all later chunk kernels honor done/breakdown state
  correct_velocity(p,B.velocity) -> stored f32 B.velocity
  compute actual divergence and Courant from B.velocity
  transport A.tracer using the declared carrier -> B.tracer
  reduce remaining acceptance diagnostics -> device_report
  record event(report_ready)
readback_stream waits(report_ready)
  copy fixed-size device_report -> pinned report_slot
  record event(report_copied)
host observes report_copied, checks status/identity/cancellation, commits or rejects
~~~

The arrows denote real buffer dependencies. A reduction is at least a partial
kernel plus a final reduction kernel when more than one block participates;
`__syncthreads()` cannot finalize a grid-wide dot product. Each producer and
consumer remains in stream order or gains an explicit event dependency. A
render or checkpoint stream must similarly wait for the accepted generation
and hold a lease until its last read completes.

A CUDA Graph can encode a stable chunk with explicit producer/consumer edges,
then be instantiated and replayed. NVIDIA separates graph definition,
instantiation and execution; graph edges constrain scheduling [GPU-C03]. This
is a launch-overhead optimization, not a new linear solver. Keep the ordinary
stream implementation as a reference path. Graph identity includes the buffer
epoch, launch extents and numerical policy; do not replay old pointers after
resize or reuse a stale captured scalar. Device graph launch and conditional
graph nodes are optional later designs, not baseline requirements.

Chunk `B_iter` iterations at a time to limit submitted work without downloading
all dot products. Keep PCG scalars and a monotonic `done`/`breakdown` state on
the device; kernels after convergence leave solver state unchanged. A host
status copy at a chunk boundary is permitted. Choosing the chunk length trades
launch/notification overhead against cancellation latency and unnecessary
masked dispatches; it requires future measurement, not an invented constant.
True residual and maximum-iteration semantics must remain correct even when
convergence happens in the middle of a replayed chunk.

### CUDA discovery and build independence

Probe through CUDA's own device-management APIs: enumeration, properties,
attributes, runtime/driver compatibility and module availability [GPU-C04].
Record device UUID, compute capability, relevant limits, driver/runtime versions
and the selected binary/PTX target. Feature support depends on the target
architecture; the official compute-capability tables are a source for mapping
requirements, not evidence about an unqueried device [GPU-C05]. FP64 support
does not imply useful FP64 throughput, and supported atomic types do not imply
deterministic accumulation.

Proposed CUDA builds compile `.cu` kernels with the CUDA toolchain and expose a
narrow host boundary to Rust; alternatively a separately qualified driver-API
module path could load a shipped device image. NVIDIA documents the host/device
split, PTX, Cubin and fatbinary compilation [GPU-C06]. Pin compiler version,
architecture targets and arithmetic options in artifacts. A CUDA-only build
must neither invoke a SPIR-V compiler nor link a Vulkan loader. Do not claim
every installed NVIDIA device is supported by every toolkit binary.

## Vulkan: a complete SPIR-V compute path

Vulkan compute uses compute pipelines and workgroup invocations [GPU-V01]. A
proposed backend owns its instance, selected physical/logical device,
compute-capable queue, memory, descriptors, shader modules, pipelines and
command pools. It may share an appropriately managed Vulkan device with a
renderer, but it must also operate headlessly. Device enumeration and queue
family selection are Vulkan operations [GPU-V02]; neither calls CUDA.

Represent numerical fields as storage buffers with fixed, documented scalar
layouts. Create buffers with storage and required transfer usage, inspect
memory requirements, choose compatible memory types, bind memory, and create
descriptor sets for the exact ranges read/written by each pipeline. Descriptor
layouts declare binding types and shader access [GPU-V03]. Use per-attempt or
per-buffer-epoch descriptor sets; do not update a set still in use unless a
separately enabled feature and its rules explicitly permit that usage.

SPIR-V is the device program, not CUDA PTX translated at runtime. The proposed
build compiles its own shader sources for a declared Vulkan/SPIR-V target and
validates the resulting modules for that target. Record shader and compiler
hashes, specialization constants and layouts. A Vulkan-only build must neither
invoke `nvcc` nor link/load the CUDA runtime. No build scripts or dependencies
are introduced by this research chapter.

### Supported-device differences

The native CUDA path targets compatible NVIDIA CUDA devices and their driver,
toolkit and executable-image requirements [GPU-C09]. Vulkan selects an
implementation through its own physical-device enumeration and advertised
properties, including vendor/device identity and device type [GPU-V02]. The
Vulkan design is therefore vendor-independent at its API boundary; actual
eligibility still depends on the exact compute, memory and arithmetic profile.
An integrated GPU, discrete GPU or software implementation must be identified
as such in evidence. Merely enumerating a Vulkan device does not establish
acceleration, usable f64 throughput or numerical eligibility.

An NVIDIA device can expose both APIs, but each backend must query its own
requirements and run its own qualification. A system with a qualifying Vulkan
implementation and no CUDA installation remains a valid Vulkan-only target.
Conversely, a headless CUDA solver need not create a Vulkan device. No specific
GPU model or operating-system support matrix is certified by this research.

### A conservative Vulkan capability profile

Choose a minimum API profile explicitly before implementation. One reasonable
research baseline is Vulkan 1.1 storage-buffer compute with ordinary pipeline
barriers and fences; synchronization2 is an optional, separately checked path.
This is an engineering proposal, not a claim that all Vulkan 1.1 devices run
the numerical baseline. In particular the baseline's f64 pressure arithmetic
requires `shaderFloat64`, queried and enabled explicitly [GPU-V04].

| Requirement / optimization | CUDA admission or selection | Vulkan admission or selection |
| --- | --- | --- |
| Compute execution | CUDA device and compatible kernel image | Physical device with `VK_QUEUE_COMPUTE_BIT`, compatible pipeline/module |
| Launch shape | Device and kernel block/shared-memory limits | `maxComputeWorkGroupCount`, `maxComputeWorkGroupSize`, `maxComputeWorkGroupInvocations`, `maxComputeSharedMemorySize` |
| Large fields | Checked allocation sizes and device budget | Storage descriptor range, allocation limits, alignment, heap/type compatibility; partition if the design supports it |
| f64 pressure | Supported target arithmetic and explicit policy; throughput measured later | Query and enable `shaderFloat64`; reject f64 method if unavailable |
| Reduction primitive | Shared-memory tree baseline; warp operations only with valid participation | Shared-memory tree baseline; subgroup arithmetic only if compute stage and operation/type support are present |
| Subgroup/warp width | Do not copy a CUDA-specific assumption into shared semantics | Query subgroup properties; never assume width 32 or 64; size control is optional |
| Floating atomic add | Exact type, address space, scope and architecture must support operation | `VK_EXT_shader_atomic_float` and the exact `shaderBufferFloat32AtomicAdd`, `shaderBufferFloat64AtomicAdd`, or shared-memory feature as appropriate |
| Float behavior | Compiler options, operation selection and target policy | Query float-control properties for bit widths and requested rounding/denormal behavior |
| Modern submission | Graph path optional | synchronization2, timeline semaphore or subgroup-size-control paths are separately gated |
| Render sharing | Optional API/device/handle compatibility | Optional renderer device/queue compatibility; external handles are not numerical prerequisites |

Vulkan feature support must be queried and then enabled at logical-device
creation; an API version alone is not permission to use an optional feature
[GPU-V04]. Limits and subgroup properties come from properties queries
[GPU-V05]. Query extension availability before chaining extension-specific
requirements, and record both advertised and enabled values. A device lacking
f64 can report `Unsupported` for `f32-f64-v1`; a prospective mixed-precision or
f32-only method needs a distinct identity and its own qualification. It must
not silently lower arithmetic precision.

Neither global nor shared floating atomic addition belongs in the conservative
Vulkan baseline. Their features distinguish address space, type and operation
[GPU-V07]. Use a hierarchical reduction: one workgroup writes one partial;
another dispatch reduces partials until a scalar remains. For a tail tile,
out-of-range lanes contribute zero and still reach every required workgroup
barrier. A subgroup optimization must account for active lanes and supported
types as well as `supportedOperations`; `shaderSubgroupExtendedTypes` is an
additional requirement where extended subgroup operand types need it. Vulkan
subgroup arithmetic reduction order is implementation-dependent [GPU-V08].

### Explicit resource dependencies and barriers

Vulkan execution order alone does not establish memory visibility. Its
synchronization chapter distinguishes execution dependencies from availability
and visibility, and distinguishes read-after-write/write-after-write hazards
from write-after-read hazards [GPU-V09]. The following is the proposed minimum
resource dependency ledger for a single compute queue. Rows apply to actual
buffer ranges; include read/write access when the consumer modifies the range.

| Producer -> consumer | Affected resources | Required dependency |
| --- | --- | --- |
| Upload transfer -> first compute | Initial fields, geometry, changed controls | Transfer write -> compute shader read |
| RHS assembly -> solver initialization | RHS | Compute shader write -> compute shader read |
| Operator application -> dot partials | Matrix product `q` | Compute shader write -> compute shader read |
| Reduction partials -> next reduction level | Partial buffer | Compute shader write -> compute shader read |
| Scalar finalize -> vector update | Alpha/beta, flags | Compute shader write -> compute shader read |
| Vector update -> next stencil/reduction | Pressure, residual, direction | Compute shader write -> compute shader read/write |
| True residual -> qualification/update | Residual diagnostics, status | Compute shader write -> compute shader read/write |
| Pressure -> velocity correction | Pressure | Compute shader write -> compute shader read |
| Correction -> stored-velocity diagnostics | f32 candidate velocity | Compute shader write -> compute shader read |
| Final report -> readback copy | Device report | Compute shader write -> transfer read |
| Readback transfer -> host read | Host-visible report staging | Transfer write -> host read; completion wait, then invalidate if noncoherent |
| Renderer/export reader -> next overwrite | Leased accepted/snapshot range | Reader completion before buffer reuse; not merely numerical completion |

For the synchronization2 path, a representative same-queue buffer dependency
is the following conceptual API sequence. It omits initialization boilerplate,
not the dependency. The backend must enable `synchronization2` and use a core
or extension entry point appropriate to the selected runtime profile.

~~~text
dispatch apply_operator(d, q)
vkCmdPipelineBarrier2(
  buffer = q, offset = q_offset, size = q_bytes,
  srcStage = COMPUTE_SHADER, srcAccess = SHADER_WRITE,
  dstStage = COMPUTE_SHADER, dstAccess = SHADER_READ,
  srcQueueFamily = IGNORED, dstQueueFamily = IGNORED)
dispatch dot_partials(d, q, partials)
barrier(partials, COMPUTE/SHADER_WRITE -> COMPUTE/SHADER_READ)
dispatch reduce_partials(partials, scalar_state)
barrier(scalar_state, COMPUTE/SHADER_WRITE -> COMPUTE/SHADER_READ)
dispatch update_vectors(...)
~~~

The legacy baseline expresses equivalent stages/accesses through
`vkCmdPipelineBarrier` and `VkBufferMemoryBarrier`. This is not permission to
substitute a shader `barrier()` for the cross-dispatch dependency. Workgroup
barriers only coordinate the workgroup; a shader that spins until all
workgroups arrive can deadlock when not all workgroups are resident. Use
separate dispatches for global phases.

For separate compute and transfer queues, add semaphore dependencies with
correct wait stages. If an exclusively shared buffer crosses queue **families**,
pair release/acquire ownership transfers with the queue synchronization;
different queues in one family need synchronization but not a family transfer.
The simple first path should use one queue and avoid this extra ownership
state. Sharing mode and queue-family identities belong in every resource's
ledger. Host access must await completion, and noncoherent mapped ranges require
the specified flush/invalidate handling and atom-size alignment [GPU-V06].

### Vulkan submission without per-iteration host round trips

Record a bounded command buffer containing the same numerical phases as the
CUDA schedule, with explicit barriers at the dependencies above. Ordinary
Vulkan compute does not supply a CUDA-Graph-style arbitrary device-side loop
over dispatches. A conservative design records a fixed chunk of PCG iterations
and uses device scalar flags to mask numerical updates after convergence or
breakdown. Each recorded reduction writes valid output even when masked, and
all participating invocations still obey shader barrier rules.

Submit that chunk, copy only bounded status at its boundary, and poll a fence
or a separately supported timeline mechanism. Reuse a command buffer only
after its previous use has completed, with stable descriptor/resource epochs.
Reserve solver buffers across chunks. Re-record or select a separately recorded
variant when extents or bindings change. Dispatch-indirect control, persistent
kernels and device-generated commands are possible separate research topics;
none is needed to make Vulkan an independent backend.

## Precision and reproducibility are part of method identity

The CPU's compensated sums have a particular evaluation order. A GPU reduction
tree can be accurate without matching that order or its bits. Keep f64 PCG
vectors and accumulation in the first comparison profile, recompute the true
residual using the declared operator, then separately check divergence after
the f32 velocity store. An f64 pressure report alone cannot prove that the
stored field passes the physical gate.

CUDA's floating-point appendix discusses rounding, subnormals, fused
operations and mathematical function behavior [GPU-C07]. Vulkan exposes
bit-width-specific float-control properties [GPU-V10]. A proposed method record
must state contraction/FMA, reassociation, denormal policy, rounding, division
and square-root policy, and compiler options. A compiler setting described as
fast math needs numerical qualification, not an assumption of equivalence.

Distinguish four future claims: agreement within declared numerical bounds;
repeatability on one device/software configuration; stability across launch
shapes; and bitwise equality across backends or CPU. Evidence for one does not
establish the others. A fixed two-level reduction tree can reduce variability,
but hardware/compiler changes and subgroup reductions still need study.
Floating atomic sums introduce scheduling-dependent addition order and are a
poor default for qualification metrics. Explicitly detect nonfinite values
rather than letting a max reduction's NaN behavior hide an invalid field.

SGS needs particular care: parallel Jacobi is pointwise, while a forward/backward
SGS sweep consumes previously updated values. Replacing lexicographic SGS with
red-black, block-Jacobi or an unsynchronized in-place sweep changes the
preconditioner. A prospective wavefront implementation must identify diagonal
dependencies and barrier phases and preserve a symmetric positive-definite
preconditioner before claiming the existing PCG method name. This is a design
constraint, not a GPU SGS equivalence result.

## Cancellation, checkpointing and readback

Cancellation means **stop accepting new chunks and reject the attempt**, then
retire already-submitted work before reusing its buffers. Neither an event nor
a fence is a portable command to abort an arbitrary running kernel. Avoid long
persistent kernels in the initial path. Set chunk/dispatch work bounds and
check cancellation before submission, at chunk boundaries and immediately
before commit. A user-visible latency bound additionally needs hardware/runtime
evidence; no such bound is established here. Device loss is a separate failure
class: accepted device memory may be unavailable, requiring a previously
completed durable checkpoint for recovery.

~~~text
attempt(A_generation):
  reserve B, scratch, report_slot; mark ticket unpublished
  while solver not finished:
    if cancel_requested: stop submitting; retire outstanding work; reject
    submit one bounded chunk with explicit dependencies
    poll completion/status without exposing B
  submit correction, transport and all acceptance checks
  await fixed-size report; reject any API error, nonfinite value or failed gate
  acquire publication lock
  recheck cancellation, session ID, buffer epoch and A_generation
  publish B + time + generation + report atomically as one logical transaction
  release publication lock
  recycle old A only after all renderer/export readers release it
~~~

The publication operation is a host ownership change, not a giant atomic GPU
copy. Keep accepted buffers immutable while an attempt runs. Double buffering
alone is insufficient when a renderer is still reading the old accepted slot
that the next step wants as scratch; use leases/backpressure or a separately
budgeted snapshot slot. Bounded rings need an explicit policy when consumers
fall behind: skip optional visual samples, throttle exports or pause stepping
according to the requested workflow, while never publishing a partial state.

A restart checkpoint should export a single accepted generation with geometry,
units, boundary/source controls, method and precision IDs, time/generation,
required physical state and any necessary continuation history. Solver scratch
may be omitted only when restart semantics permit reconstructing it. Record
whether pressure history is required by the selected method, rather than
treating the current interval-pressure view as a restart payload. Copy in
bounded tiles from a leased generation, record completion for every tile,
validate the manifest/checksums, then publish a complete durable file. A hash
does not prove physical correctness or successful restart.

Readback categories should be explicit: fixed-size step report; requested
diagnostic sample; visual snapshot; and full checkpoint. Each has a byte cap,
frequency, generation stamp, completion token and backlog cap. A CPU pressure
provider adapter is an intentionally different transfer profile and must report
its full RHS/pressure transfers when evaluating performance.

## Optional rendering interoperation

A Vulkan solver and Vulkan renderer on one compatible device can consume an
accepted buffer or derived image using proper compute-to-graphics stage/access
dependencies, queue ownership and image layout transitions where applicable.
Rendering cannot read a candidate that may still be rejected. Copying a scalar
field into a display texture changes storage and sampling, so that texture is
not the scientific state or an acceptance oracle.

A CUDA solver may optionally use external memory/semaphore interoperation with
a renderer. NVIDIA documents Vulkan device matching, imported memory and
signal/wait objects in its interoperability section [GPU-C08]. Matching physical
devices, supported handle types, resource lifetime and synchronization must be
qualified separately. This optional bridge neither supplies Vulkan numerical
kernels nor becomes a dependency of headless CUDA stepping. A bounded host
snapshot remains an alternative output path when no bridge is available.

## Evidence boundary and open implementation decisions

This chapter establishes a proposed resource/semantic design from repository
inspection and official API documents. It establishes no installed driver,
physical device, FP64 throughput, memory capacity, shader compilation, solver
accuracy, cancellation timing or speedup. No GPU workload or build was run.

Before implementation, decide the minimum Vulkan API/SPIR-V profile, precise
f64 eligibility policy, CUDA architecture/compiler targets, shader toolchain,
Rust ownership boundary, report schema, allocation headroom and chunking policy.
Each backend needs its own failure handling and conformance evidence. Shared
acceptance fixtures and mathematical operators are desirable; shared CUDA or
Vulkan execution objects are not required by the architecture. Preserve Puma's
authorship and the existing CPU methods while qualifying each proposed method
under its own explicit identity.
