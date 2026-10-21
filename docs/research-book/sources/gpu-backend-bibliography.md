# Independent GPU backends: official-source bibliography

Accessed online **2026-10-21**. This ledger supports
[Chapter 38](../chapters/38-independent-cuda-and-vulkan.md). Primary NVIDIA and
Khronos documentation only; repository observations are separately linked at
the end. API facts support the design, not a claim that Rheon implements them
or that a particular GPU was queried or measured. Proposed schedules,
interfaces, memory ledgers and admission policies are engineering synthesis.

## Revision policy

NVIDIA's current **CUDA Programming Guide** pages display **Last updated on
Sep 10, 2026** where individually verified below. The guide's landing page does
not print a toolkit version in the retrieved text; this ledger does not infer
one from another document. The separately retrieved CUDA Runtime API reference
does explicitly identify **13.4** in its page title. Moving URLs are preserved
as accessed snapshots, not mislabeled immutable version URLs.

Khronos's [documentation index](https://docs.vulkan.org/spec/latest/index.html)
explicitly identifies the consulted Vulkan Specification build as **1.4.365**,
including all registered extensions, generated **2026-10-02 02:01:57Z**, branch
`main`, commit **`8c9361ba8180c1f4164c0bf79de2f6e817770b0d`**. Specification
chapter URLs below are that moving documentation build as observed on the
access date. Standalone reference pages are also moving documents; their
retrieved pages do not separately print a version/build identifier, so no
independent revision identity is asserted for them. A future implementation
must pin and recheck its actual headers, loader, driver, compiler and target
environment rather than assuming the latest documentation is its runtime.

## NVIDIA CUDA

### GPU-C00. CUDA Programming Guide: identity and scope

[Guide landing page](https://docs.nvidia.com/cuda/cuda-programming-guide/) and
[§1.1 Introduction](https://docs.nvidia.com/cuda/cuda-programming-guide/01-introduction/introduction.html).

Verified: official guide identity; §1.1.1 describes CUDA's independence from
graphics APIs. Introduction footer explicitly dated 2026-09-10. No toolkit
version inferred. Access: public web retrieval, 2026-10-21.

### GPU-C01. §2.3 Writing SIMT Kernels

[Official chapter](https://docs.nvidia.com/cuda/cuda-programming-guide/02-basics/writing-cuda-kernels.html).

Consulted: §2.3.2 Thread Hierarchy; §2.3.2.1 Thread Block Synchronization;
§2.3.3 GPU Device Memory Spaces, especially global/shared memory; §2.3.5
Atomics. Supports block-local synchronization, memory scope distinctions and
kernel decomposition. Individually verified footer: 2026-09-10. Access:
2026-10-21. Does not establish a device's supported block size or occupancy.

### GPU-C02. §2.5 Asynchronous Execution

[Official chapter](https://docs.nvidia.com/cuda/cuda-programming-guide/02-basics/asynchronous-execution.html).

Consulted: §2.5.1 asynchronous completion; §2.5.2 CUDA Streams, including
§2.5.2.3 transfers and §2.5.2.4 synchronization; events and cross-stream
dependencies; §2.5.6 default/blocking/nonblocking stream behavior. Supports the
independent CUDA scheduling design and pinned host staging. Individually
verified footer: 2026-09-10. Access: 2026-10-21. An asynchronous API is not a
guarantee of hardware concurrency or a timing bound.

### GPU-C03. §4.2 CUDA Graphs

[Official chapter](https://docs.nvidia.com/cuda/cuda-programming-guide/04-special-topics/cuda-graphs.html).

Consulted: §4.2.1 Graph Structure; §4.2.2 Building and Running Graphs;
§4.2.3 Updating Instantiated Graphs; §4.2.6 Device Graph Launch;
§4.2.7 Using Graph APIs. Supports graph dependency edges, separate definition,
instantiation and execution, and why advanced graph paths need distinct
requirements. Individually verified footer: 2026-09-10. Access: 2026-10-21.
Chapter 38's bounded PCG chunk is a proposed design, not vendor sample code.

### GPU-C04. CUDA Runtime API Reference Manual 13.4, §6.3 Device Management

[Official device-management reference](https://docs.nvidia.com/cuda/cuda-runtime-api/cuda_runtime_api/group__CUDART__DEVICE.html).

Verified page title: **CUDA Runtime API Reference Manual 13.4 documentation**.
Consulted function inventory and descriptions for device enumeration,
`cudaGetDeviceProperties` and `cudaDeviceGetAttribute`. Supports CUDA-owned
capability discovery. No last-updated date was displayed in retrieved text;
access date is 2026-10-21. This version applies to this reference, not an
inferred installed toolkit or Programming Guide version.

### GPU-C05. §5.1 Compute Capabilities

[Official appendix](https://docs.nvidia.com/cuda/cuda-programming-guide/05-appendices/compute-capabilities.html).

Consulted: architecture-dependent feature and technical-limit tables. Supports
target-specific admission rather than universal device assumptions.
Individually verified footer: 2026-09-10. Access: 2026-10-21. No hardware
support matrix for Rheon was tested; FP64 throughput is not measured here.

### GPU-C06. §2.7 NVCC: The NVIDIA CUDA Compiler

[Official chapter](https://docs.nvidia.com/cuda/cuda-programming-guide/02-basics/nvcc.html).

Consulted: §2.7.1 CUDA Source Files and Headers; §2.7.2 NVCC Compilation
Workflow. Supports separate host/device compilation, PTX, Cubin and fatbinary
artifacts. Accessed moving guide chapter on 2026-10-21; no individually checked
footer date is asserted for this page. Rust FFI choices in Chapter 38 are
proposed project decisions, not an NVIDIA-prescribed Rust integration.

### GPU-C07. §5.5 Floating-Point Computation

[Official appendix](https://docs.nvidia.com/cuda/cuda-programming-guide/05-appendices/mathematical-functions.html).

Consulted: §5.5.1 Floating-Point Introduction, normal/subnormal and rounding
discussion, and mathematical-function behavior. Supports declaring arithmetic
policy separately from storage types and checking numerical differences.
Individually verified footer: 2026-09-10. Access: 2026-10-21. The URL's filename
differs from the visible chapter title; the visible title above is intentional.

### GPU-C08. §4.21 CUDA Interoperability with APIs

[Official chapter](https://docs.nvidia.com/cuda/cuda-programming-guide/04-special-topics/graphics-interop.html).

Consulted: §4.21.2 External resource interoperability; §4.21.2.1 Vulkan
interoperability, including device matching, imported memory and
§4.21.2.1.9 Signaling/Waiting on Imported Synchronization Objects. Supports only
the optional render-interop discussion. Accessed moving guide chapter on
2026-10-21; no individually checked footer date asserted here. Interop is not
a dependency of either numerical backend.

### GPU-C09. §1.3 The CUDA platform

[Official chapter](https://docs.nvidia.com/cuda/cuda-programming-guide/01-introduction/cuda-platform.html).

Consulted: §1.3.1 NVIDIA GPU compute capability; §1.3.2 CUDA Toolkit and NVIDIA
Driver; §1.3.3 PTX and §1.3.4 Cubins/Fatbins. Supports the CUDA platform/device
scope and distinction between driver, toolkit and image compatibility. Footer
individually verified 2026-09-10; accessed 2026-10-21. No GPU was enumerated,
driver installed or toolkit compatibility tested here.

## Khronos Vulkan

### GPU-V00. Documentation build identity

[Official Vulkan documentation index](https://docs.vulkan.org/spec/latest/index.html),
“About,” build identity quoted by value in the revision policy above.

Supports the exact consulted specification revision and generation date.
Access: 2026-10-21. “latest” is a moving URL. Specification 1.4.365 does not
imply a requirement for a Vulkan 1.4 device.

### GPU-V01. Compute pipelines and dispatch

[Shaders](https://docs.vulkan.org/spec/latest/chapters/shaders.html), introduction
and compute shader execution; [Dispatching Commands](https://docs.vulkan.org/spec/latest/chapters/dispatch.html),
`vkCmdDispatch`.

Specification build: GPU-V00. Access: 2026-10-21. Supports compute workgroup
execution and dispatch semantics. Chapter 38's shader compilation and numerical
pipeline are proposed; no SPIR-V modules were compiled or executed.

### GPU-V02. Devices and Queues

[Official specification chapter](https://docs.vulkan.org/spec/latest/chapters/devsandqueues.html),
“Physical Devices,” physical-device properties and queue-family discovery.

Specification build: GPU-V00. Access: 2026-10-21. Supports independent Vulkan
device enumeration and compute queue selection. Does not establish which
devices or queue families exist in the execution environment.

### GPU-V03. Descriptor Sets

[Official specification chapter](https://docs.vulkan.org/spec/latest/chapters/descriptorsets.html),
“Descriptor Set Layout,” descriptor bindings and descriptor-set use/update
requirements.

Specification build: GPU-V00. Access: 2026-10-21. Supports explicit shader
resource bindings and lifetime-conscious descriptor use. Storage buffer layout
and per-attempt descriptors in Chapter 38 are proposed design choices.

### GPU-V04. Features: query, enable, and shaderFloat64

[Official specification chapter](https://docs.vulkan.org/spec/latest/chapters/features.html),
opening feature-query/enable rules; `VkPhysicalDeviceFeatures`,
`shaderFloat64`; subgroup extended-type and synchronization2 feature entries.

Specification build: GPU-V00. Access: 2026-10-21. Supports explicit
`vkGetPhysicalDeviceFeatures2` querying and logical-device enablement, and
optional f64 shader arithmetic. API version does not itself establish that an
optional arithmetic/atomic feature is enabled.

### GPU-V05. Limits and subgroup properties

[Official specification chapter](https://docs.vulkan.org/spec/latest/chapters/limits.html),
`VkPhysicalDeviceLimits`, `VkPhysicalDeviceSubgroupProperties`,
`VkSubgroupFeatureFlagBits` and subgroup-size-control properties.

Specification build: GPU-V00. Access: 2026-10-21. Supports workgroup, storage
buffer and allocation-limit admission, subgroup size/stages/operations queries,
and avoidance of fixed-width subgroup assumptions. Extended operand-type
requirements also need GPU-V04; arithmetic-bit support alone is insufficient.

### GPU-V06. Memory Allocation and mapped-memory visibility

[Official specification chapter](https://docs.vulkan.org/spec/latest/chapters/memory.html),
device memory, allocation/memory types, mapped memory,
`vkFlushMappedMemoryRanges` and `vkInvalidateMappedMemoryRanges`.

Specification build: GPU-V00. Access: 2026-10-21. Supports compatible memory
selection, explicit host coherence handling and the need for device completion
before invalidation/readback. Mapping alone does not invalidate noncoherent
memory. Chapter 38's byte budget and staging rings are engineering proposals.

### GPU-V07. Floating-point atomics are separate features

[VkPhysicalDeviceShaderAtomicFloatFeaturesEXT](https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceShaderAtomicFloatFeaturesEXT.html),
“Description,” and the matching feature definitions in
[Features](https://docs.vulkan.org/spec/latest/chapters/features.html).

Access: 2026-10-21. Standalone reference-page version is not separately printed;
specification chapter revision is GPU-V00. Supports separate buffer/shared
f32/f64 atomic and atomic-add feature bits, query and enable rules. Baseline
hierarchical reductions avoid requiring these optional additions.

### GPU-V08. Shader scopes and arithmetic group operations

[Shaders](https://docs.vulkan.org/spec/latest/chapters/shaders.html),
“Workgroup,” “Subgroup,” “Group Operations,” and “Arithmetic Group Operations.”

Specification build: GPU-V00. Access: 2026-10-21. Supports scope limits and
implementation-dependent subgroup arithmetic evaluation order. A proposed
deterministic reduction must state its own tree and arithmetic policy; a
subgroup reduction is not a universal bitwise-reproducibility primitive.

### GPU-V09. Synchronization and Cache Control

[Official specification chapter](https://docs.vulkan.org/spec/latest/chapters/synchronization.html),
“Execution and Memory Dependencies,” “Pipeline Barriers,” fences, semaphores,
buffer barriers and queue-family ownership transfers.

Specification build: GPU-V00. Access: 2026-10-21. Supports execution versus
availability/visibility distinctions, resource-hazard dependencies and the
legacy/synchronization2 design discussion. Chapter 38's dependency table is a
proposed resource schedule, not a validated command buffer.

### GPU-V10. Floating-point controls

[VkPhysicalDeviceFloatControlsProperties](https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceFloatControlsProperties.html),
“Description,” including per-width signed-zero/Inf/NaN, denormal and rounding
properties.

Access: 2026-10-21. Standalone reference-page version is not separately printed.
Supports querying admissible float-control behavior; availability of f64
arithmetic is a distinct GPU-V04 requirement. No cross-backend bitwise
equivalence follows from the existence of these properties.

## Repository evidence inspected without execution

Main baseline: `fd7c0f0`. Existing project authorship is Puma's; this research
does not replace it. The
[CPU oracle inventory](../implementation/gpu-cpu-oracle-inventory.md) provides
the full source inventory and qualification limits.

| Source | Observed constraint used in Chapter 38 |
| --- | --- |
| [`src/simulation.rs`](../../../src/simulation.rs) | f32 named-axis fields; candidate/accepted separation; host-slice `StateView`; transactional step semantics; actual stored-velocity divergence gate |
| [`src/pressure.rs`](../../../src/pressure.rs) | Jacobi/SGS PCG identities; f64 pressure/reduction arithmetic; true residual and physical qualification |
| [`src/solver_provider.rs`](../../../src/solver_provider.rs) | Synchronous host-slice pressure provider; provider produces a candidate that Rheon qualifies; bounded allocation and cancellation responsibilities |
| [`src/provider_step.rs`](../../../src/provider_step.rs) | Whole-step acceptance boundary and separate provider workspace accounting |
| [`src/recording/provider/runtime.rs`](../../../src/recording/provider/runtime.rs) | Accepted-context lineage; bounded runtime custody; explicit exclusion of restart, authentication and preemption guarantees |

No hardware discovery, scientific workload, GPU workload, dependency install or
full build was performed for this bibliography. A direct shell HTTP retrieval
attempt was unavailable because the environment's network proxy returned 403;
official pages were successfully read through the available web retrieval tool.
No restriction was bypassed. The unresolved evidence is implementation and
hardware qualification, not source access.
