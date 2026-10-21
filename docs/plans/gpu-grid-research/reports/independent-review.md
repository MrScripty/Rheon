# Independent technical and primary-source review

Review date: 2026-10-21. CPU baseline: `fd7c0f0aee63bc8710df99f9bb01154e7619b974`.
Research branch: `research/gpu-grid-algorithms-20261021`.
Reviewer: separate technical/source review worker; original book authorship remains Puma.

**Final disposition: no unresolved deciding findings in the reviewed GPU
documentation after the repairs below were reread.** This is source/documentation
acceptance only; no implemented GPU method or numerical result is qualified.

This review covers the new CPU inventory, reader/source plan, Chapters 37–39,
and the algorithm/backend bibliographies. The initial drafts were numbered
30–32; parent coordination moved only the new GPU files to 37–39 to preserve
the separately owned FEM chapter range. This report records source inspection
and document checks, not numerical, device, performance or publication
qualification. The reviewer changed only this report; authors/parent made the
document repairs described below.

## Ranked deciding findings and repairs

No critical or high-severity source/API finding was identified. The following
medium-severity technical issues and low-severity navigation issue were found
in drafts and sent to their owners. Locations below use the final filenames;
line numbers identify the inspected repaired passage unless explicitly labeled
as original draft lines.

1. **Medium — preserve pressure acceptance before stored-field acceptance.**
   In [Chapter 37](../../../research-book/chapters/37-gpu-grid-algorithms.md),
   original draft line 181 made PCG acceptance require “stored correction
   checks.” That mixed the pressure candidate contract with the separate full
   step correction and could change solver iteration/stopping behavior or
   accidentally correct the candidate repeatedly. `src/pressure.rs:563` accepts
   the freshly recomputed full residual and its predicted divergence;
   `src/simulation.rs:944` then solves, corrects stored f32 velocity, and checks
   actual divergence at `src/simulation.rs:959`. Correction: PCG returns
   pressure success from true-residual/predicted-divergence gates; the step
   performs the one stored correction and independent gate afterward.
   **Resolved and reread:** Chapter 37 lines 169, 183–184 and 197 now state that
   separation. Chapter 39 lines 70–71 also show the correct sequence.

2. **Medium — reset the residual-replacement direction flag every iteration.**
   The Chapter 37 draft set `restart_direction = true` only on replacement,
   then read the flag unconditionally; it never established false on the
   ordinary path. An implementation of that sketch could read an undefined
   flag or retain true forever after one replacement, degenerating the
   intended PCG direction policy. `src/pressure.rs:614` establishes
   `replaced = false` inside each iteration before a candidate check.
   **Resolved and reread:** Chapter 37 line 173 initializes the flag inside the
   loop, before any branch.

3. **Medium — return iteration exhaustion before preparing a nonexistent next iteration.**
   Chapter 37 lines 185–189, as first rechecked, still preconditioned the
   residual and validated `gamma_next` after an unsuccessful final-budget true
   check. The CPU returns `IterationLimit` at `src/pressure.rs:625` before the
   next preconditioner application at line 631. The draft could mask exhaustion
   with a later breakdown and perform unnecessary work. Correction: return
   iteration exhaustion immediately after the unsuccessful final candidate
   check, before preconditioning. **Resolved and reread:** Chapter 37 line 187
   returns `IterationLimit` before the preconditioner at line 188.

4. **Medium — align two pressure fields with their own gauge values.**
   The [Chapter 39](../../../research-book/chapters/39-gpu-validation-and-roadmap.md)
   draft instructed subtracting the same pinned-cell value from both fields.
   Subtracting a common constant leaves their difference unchanged and does
   not remove a relative gauge offset. Subtract each field's own value at the
   same pin, or shift the candidate to the reference pin. The sealed CPU
   convention is pressure cell zero fixed to zero (`src/pressure.rs:409`,
   `src/pressure.rs:555`). **Resolved and reread:** Chapter 39 lines 112–116 now
   specify each field's own pinned value and retain the restriction against
   arbitrarily shifting pressure-anchored boundaries.

5. **Low — link the actual independent-backend chapter filename.**
   Chapter 37 original draft line 7 linked the nonexistent
   `31-cuda-vulkan-backends.md`. **Resolved and reread:** the link now targets
   `38-independent-cuda-and-vulkan.md`, including the final renumbering. The
   final scan also found no missing local relative-link destinations.

## Deciding CPU-source checks

The pressure/operator and publication claims are the deciding boundary for
this research. Direct source reads confirmed the following; no test was run.

| Claim reviewed | Deciding source evidence | Disposition |
| --- | --- | --- |
| Named-axis staggered f32 velocities/tracer, x-fast layout, six f64 pressure vectors and baseline payload | `src/geometry.rs:81`, `src/geometry.rs:235`, `src/geometry.rs:281`; `src/simulation.rs:158`; `src/pressure.rs:235` | Consistent. Payload is managed array storage, not total process/device memory. |
| Integrated pressure sign/coefficient and physical correction | `src/operator.rs:57`, `src/operator.rs:318`, `src/operator.rs:399`, `src/operator.rs:568` | Consistent with `K=B diag(area/(rho*distance)) B^T`, negative outward-flux RHS and subtraction of physical pressure gradient. |
| Zero guess, sealed compatibility, eliminated gauge, full ungauged true residual and conjunction of residual/predicted divergence | `src/pressure.rs:374`, `src/pressure.rs:545`, `src/pressure.rs:555`, `src/pressure.rs:563`, `src/pressure.rs:586` | Consistent. No mean-shift compatibility repair or pressure warm start is invented. |
| Actual candidate stage order and stored-velocity checks | `src/simulation.rs:655`, `src/simulation.rs:792`, `src/simulation.rs:818`, `src/simulation.rs:860`, `src/simulation.rs:944`, `src/simulation.rs:959`, `src/simulation.rs:984`, `src/simulation.rs:1006`, `src/simulation.rs:1080` | Consistent after acceptance-order repair. Tracer uses corrected candidate velocity; its integral is diagnostic. |
| Ordered compensated pressure reductions and SGS cross-row dependencies | `src/pressure.rs:127`, `src/pressure.rs:142`, `src/pressure.rs:406`; `src/sgs.rs:1`, `src/sgs.rs:20`, `src/sgs.rs:56` | Consistent. Atomic storage contains f64 bits; joined waves establish visibility. Parallel Jacobi is a different preconditioner from SGS. |
| Candidate/accepted ownership and cooperative cancellation | `src/simulation.rs:649`, `src/simulation.rs:663`, `src/simulation.rs:1080`; `src/session.rs:234` | Consistent. `Simulation` commits numerical state; session quotas wrap it. No hard wall-time preemption guarantee follows from callbacks. |
| Pressure provider is a synchronous host-slice candidate boundary | `src/solver_provider.rs`; `src/provider_step.rs`; `src/recording/provider/runtime.rs` | Consistent. Pressure-only transfers and selected/fallback provenance must be disclosed; this API is not already a resident whole-step backend. |
| Particle preview is a separate physical owner and display representation | `src/particle_fluid.rs:353`; `src/material_session.rs:49`; `src/bin/desktop/history.rs` | Consistent. Diagnostic particle pressure is pre-integration; material capability says `grid_pressure_provider=false`; appearance voxels are not MAC samples. |
| Accepted interval pressure versus restart and archive failure behavior | `src/recording/carrier.rs:1`, `src/recording/carrier.rs:46`; `src/simulation.rs:312`; `src/recording.rs:320`; `docs/api/recording.md:129` | Consistent. Recording profiles have bounded admission, pressure stamps invalidate on later attempts, append failure poisons the writer, and inspection is not complete continuation state. |

The limited slab/column, fraction and aligned-obstacle owners remain explicitly
separate from the filled-box oracle. The chapters do not promote collision
queries or particle contact to arbitrary embedded-grid pressure/stress
boundaries. Existing physical qualification failures remain historical evidence,
not new execution or a GPU pass.

## Independently reopened primary sources

The reviewer reopened the following sources through read-only web retrieval on
2026-10-21 instead of relying solely on the bibliography's assertion of access.
Each supports the limited claim listed; none establishes installed capabilities.

| Primary source independently read | Claim checked and result |
| --- | --- |
| [NVIDIA asynchronous execution](https://docs.nvidia.com/cuda/cuda-programming-guide/02-basics/asynchronous-execution.html) | Stream sequencing, event dependencies, asynchronous host completion and hardware-dependent concurrency support the proposed schedule. Footer independently displayed 2026-09-10. |
| [NVIDIA graphs](https://docs.nvidia.com/cuda/cuda-programming-guide/04-special-topics/cuda-graphs.html) and [compute capabilities](https://docs.nvidia.com/cuda/cuda-programming-guide/05-appendices/compute-capabilities.html) | Definition/instantiation/execution and target-dependent capabilities support explicit graph/resource and admission identities. They supply no Rheon timing. |
| [NVIDIA CUDA platform](https://docs.nvidia.com/cuda/cuda-programming-guide/01-introduction/cuda-platform.html) | NVIDIA hardware, driver/toolkit, PTX and cubin/fatbin distinctions support Chapter 38's supported-device discussion. No specific model/OS certification is inferred. |
| [Khronos documentation index](https://docs.vulkan.org/spec/latest/index.html) | Independently matched Vulkan 1.4.365, generation time 2026-10-02 02:01:57Z and commit `8c9361ba8180c1f4164c0bf79de2f6e817770b0d`. This is a moving document identity, not the chosen minimum API. |
| [Khronos features](https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceFeatures.html), [floating atomics](https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceShaderAtomicFloatFeaturesEXT.html) and [shaders](https://docs.vulkan.org/spec/latest/chapters/shaders.html) | `shaderFloat64`, distinct buffer/shared f32/f64 atomic-add features, workgroup scope and implementation-dependent subgroup arithmetic order support capability-conditioned arithmetic and hierarchical reductions. |
| [Khronos synchronization](https://docs.vulkan.org/spec/latest/chapters/synchronization.html) | Execution dependencies alone do not supply visibility; explicit memory dependencies, host completion, semaphore and ownership rules support the resource ledger. |
| [Harris GPU Gems chapter](https://developer.nvidia.com/gpugems/gpugems/part-vi-beyond-triangles/chapter-38-fast-fluid-dynamics-simulation-gpu) and [Crane et al. GPU Gems 3 chapter](https://developer.nvidia.com/gpugems/gpugems3/part-v-physics-simulation/chapter-30-real-time-simulation-and-rendering-3d-fluids) | Confirmed historical 2D rectangular/no-free-surface scope and the 3D chapter's deliberately collocated exposition/MAC caution. Their examples are not Rheon's discretization. |
| [Bolz primary PDF](https://www.cs.columbia.edu/cg/pdfs/28_GPUSim.pdf) and [McAdams primary PDF](https://www.math.ucdavis.edu/~jteran/papers/MST10.pdf) | Confirmed GPU sparse CG/regular multigrid versus shared-memory CPU multigrid-PCG. The drafts correctly avoid transferring published timings to Rheon. |
| [Harris reduction PDF](https://developer.download.nvidia.com/assets/cuda/files/reduction.pdf) and [Bridson/Müller-Fischer notes](https://www.cs.ubc.ca/~rbridson/fluidsimulation/fluids_notes.pdf) | Confirmed hierarchical reduction motivation, historical hardware limits and notes' title/date/preconditioner scope. Modern synchronization is separately sourced. |
| [Weber author manuscript](https://www.academia.edu/14777152/A_Cut_Cell_Geometric_Multigrid_Poisson_Solver_for_Fluid_Simulation) and [Aanjaneya author abstract](https://orionquest.github.io/papers/EGMVL/paper.html) | Confirmed geometry-specific staggered cut-cell hierarchy motivation and coupled-viscosity box smoothing. Their general suitability and detailed implementation are not inferred. |
| [Pinned AmgX PCG source](https://raw.githubusercontent.com/NVIDIA/AMGX/91a8413ef267b1c32aff4014c02820e1c5897ac2/src/solvers/pcg_solver.cu), [version file](https://raw.githubusercontent.com/NVIDIA/AMGX/91a8413ef267b1c32aff4014c02820e1c5897ac2/ReleaseVersion.txt) and [license](https://raw.githubusercontent.com/NVIDIA/AMGX/91a8413ef267b1c32aff4014c02820e1c5897ac2/LICENSES/BSD-3-Clause.txt) | Confirmed apply/dot/update/precondition structure, version 2.5.0 and BSD-3-Clause scope at the cited hash. No claim about bundled dependencies' licenses or Vulkan support follows. |
| [Pinned Flow pressure orchestration](https://raw.githubusercontent.com/NVIDIA-Omniverse/PhysX/b97365c6ddb32fda8469bc788ec18024862b0368/flow/source/nvflow/Pressure.cpp), [divergence shader](https://raw.githubusercontent.com/NVIDIA-Omniverse/PhysX/b97365c6ddb32fda8469bc788ec18024862b0368/flow/source/nvflow/shaders/PressureDivergenceCS.hlsl) and [license](https://raw.githubusercontent.com/NVIDIA-Omniverse/PhysX/b97365c6ddb32fda8469bc788ec18024862b0368/LICENSE.md) | Confirmed separate pressure passes/resources, centered collocated float4 velocity differences and BSD-3-Clause at the cited hash. No integrated MAC parity claim or code incorporation is made. |

Independent retrieval limits are explicit: direct ACM opening returned 403;
the Bolz institutional PDF supplied an alternative primary copy. Initial
Wisconsin PDF guesses were unavailable; the cited UCDavis author PDF succeeded.
The Wiley full-text fetch timed out; the author manuscript supported only the
limited cut-cell claim. GitHub API commit metadata was inaccessible in this
reviewer session, and the PhysX commit page exceeded the retrieval size limit;
pinned raw source/license content succeeded. Exact upstream commit timestamps
remain the bibliography author's metadata check rather than an independently
reproduced reviewer check. No denied operation was retried through an access
workaround, and no source-fetch failure was treated as numerical evidence.

## Document checks and remaining qualification

After all repairs and renumbering, the local relative-link existence scan
covered the seven new GPU book documents and **159 local relative links, with
zero missing destinations**. No old GPU chapter paths remained in those files.
It does not validate fragment anchors, rendered URLs, equations, tables, public
downloads or PDF/Word/browser delivery. Parent-owned formatting checks and
standards review are separate evidence, not execution by this reviewer.

No solver, numerical fixture, device enumeration, scientific workload, GPU
kernel, test suite, full book build, installation, security change or publication
was run by this reviewer. No production/shared navigation/TOC source was edited.
Passing this source review can support a documentation handoff only. Numerical
parity, device/error/cancellation qualification, end-to-end speed, memory peaks,
physical convergence and restart equivalence remain future gates in Chapter 39.
