# GPU grid-fluid research: reading plan and source map

Research addition to Puma's existing book, 21 October 2026. This file is the
reader/source map; the [publication source/license audit](gpu-sources-and-publication-scope.md)
owns the public source/access and publication-scope explanation. The proposed implementation stages are owned by
[Chapter 39](../chapters/39-gpu-validation-and-roadmap.md), and confer no production
implementation authority.

## Research question and source baseline

How can Rheon's actual CPU grid-fluid algorithms execute on a GPU while
preserving their numerical and accepted-state contracts, with CUDA and Vulkan
as independently usable compute backends? The goal is a route toward a general
fluid solver whose qualified workloads can remain resident on the device. It is
not a claim that GPU execution closes currently missing fluid physics.

The original checkout was a clean `work` branch at `6ba41c7`; it was preserved.
Remote main was fetched and inspected at
`fd7c0f0aee63bc8710df99f9bb01154e7619b974` (merged PR144). The research branch
`research/gpu-grid-algorithms-20261021` and isolated worktree
`/workspace/rheon-gpu-research` start there. Product PR145/146 were then active
and outside the original write set. Publication preparation now starts from main
`d67aec1a4575e0a61aa8cd931c8eeec33c8dc6a3`, after those product changes, FEM
PR147 and portal PR148 merged. The dated CPU inventory refresh records actual
authored-field and force-recording additions while preserving the original pin.
The publication changes no product source, proof, numerical gate, shared book
builder, workflow or existing bibliography; one additive book README paragraph
provides GPU discovery. Portal 40 and FEM 30–36 remain unchanged.

## Exact proposed book artifacts

| Artifact | Distinct purpose |
| --- | --- |
| [CPU oracle inventory](gpu-cpu-oracle-inventory.md) | Actual source stages, discretization, precision, boundaries, forces, validation, recording and capability limits |
| [37 GPU grid algorithms](../chapters/37-gpu-grid-algorithms.md) | Principles, mathematical dependencies and established grid-fluid GPU methods; algorithm assumptions and failures |
| [38 Independent CUDA and Vulkan](../chapters/38-independent-cuda-and-vulkan.md) | Two independent implementation paths, resource and capability contracts, synchronization and lifecycle mapping |
| [39 Validation and roadmap](../chapters/39-gpu-validation-and-roadmap.md) | CPU/independent oracles, numerical checks, benchmarks, integration and staged future implementation gates |
| [Algorithm bibliography](../sources/gpu-algorithm-bibliography.md) | Verified papers/whitepapers and pinned open-source solver source, provenance and license boundaries |
| [Backend bibliography](../sources/gpu-backend-bibliography.md) | Official NVIDIA/Khronos authority, exact consulted sections, version/access-date limits |

Parent coordination reserves Chapters 30–36 for the separate FEM draft at
`9489aa3c1bd6652f9ab1874fce904853b9063dc3` on
`research/lagrangian-fem-book-20261021`, also based on `fd7c0f0`.
Only the new GPU chapters were moved to 37–39; existing material was not
renumbered. Parent integration owns shared navigation/coverage/build changes
and consolidation into Appendix C. The FEM proposal's book README, Chapters
11/29 and fitted-height/fitted-ALE guide edits remain untouched here.

## Read principles before APIs

1. Refresh the CPU inventory and identify its admitted filled-box oracle. Separate
   the MAC fields from particle SPH and the appearance-density voxel renderer.
2. Read Chapter 37 after Chapters 3–5 and 13. Follow its derivation from compatible
   flux operators to parallel dependency graphs, pressure methods, memory and
   precision. Existing foundations are linked rather than rewritten.
3. Read Chapter 38 as two alternatives implementing those meanings. The common
   contract is field/stage/acceptance semantics, not a CUDA execution object
   hidden behind a Vulkan name or a Vulkan requirement on CUDA.
4. Read Chapter 39 to select future qualification evidence and implementation
   slices. A paper's benchmark is not a Rheon benchmark; a solved matrix is not
   a proof of PDE convergence, coupled traction or physical calibration.

## Claim-to-source map

| Question | Primary authority | How this research uses it |
| --- | --- | --- |
| What is already implemented? | Pinned Rheon `src/`, `tests/`, current audit and implementation notes | Source inventory, bounded domain and exact oracle; historical evidence labeled historical |
| Why do grid algorithms parallelize, and where do they synchronize? | GPU-fluid papers, GPU Gems authored technical chapters and reduction whitepaper in algorithm bibliography | Compare mechanisms and dependencies; do not transfer collocated discretization or paper timings into MAC claims |
| Which pressure strategies are candidates? | Original GPU sparse/multigrid papers, Bridson notes and pinned AMGX source | Compare Jacobi/PCG, fixed symmetric parallel preconditioners and multigrid under explicit operator assumptions |
| What does current CUDA permit? | NVIDIA programming/runtime documentation and compute-capability sections in backend bibliography | Kernel/stream/event/graph execution, synchronization, memory/resource limits and optional architecture features |
| What does current Vulkan permit? | Khronos specification, feature/property reference pages and official guide | Compute-only pipeline, SPIR-V interface, descriptor/queue/barrier/subgroup limits; no assumed FP64/atomic/subgroup size |
| How should accepted state and recordings behave? | Rheon `simulation`, `session`, `recording` and provider owners | Preserve candidate/accepted separation and durable-prefix semantics; no invented restart capability |
| Can CPU and GPU be compared meaningfully? | Actual pressure and step gates plus independently derived error metrics | Equal problem/tolerance/recording boundaries; distinguish parity, repeatability, physical convergence and performance |
| Is third-party code incorporated? | Each upstream source's pinned license, Rheon Apache-2.0 | No code, figures, binaries or datasets incorporated; future adoption needs a separate exact dependency/license audit |

The bibliographies distinguish immutable source revisions from moving official
documentation. Access date is 2026-10-21; a moving page is evidence of consulted
content, not a reproducible SDK build or an assertion about every installed
driver. A later implementation must pin its actual compiler, API environment,
SPIR-V target, loader/binding versions and device features. Source version/date
and license are source-review facts, not hardware verification.

## Existing material and the actual gaps

Chapter 13 already discusses SoA layouts, reductions, CPU scheduling and memory.
Chapters 3–5 already derive staggered operators and pressure, and Chapters 15/18
already distinguish floating arithmetic and validation. The new material adds
the missing GPU dependency/execution mapping, parallel preconditioner analysis,
capability-conditioned precision, independent CUDA/Vulkan architecture, resident
transaction/checkpoint/readback design and implementable differential benchmark
protocol. It does not duplicate a generic backend design as evidence of a solver.

Chapters 8–10 and 19–29 own physical boundary, surface, stress and coupling
assumptions. GPU chapters explain what additional data/dependencies their admitted
variants need. They do not promote a static query mesh, reduced column, bounded
cut reference or particle collision response into general evolving 3D grid
physics. Compressible and Lagrangian FEM owners remain separate.

## Build scope and safe review boundary

The book is source Markdown. `docs/education/build.py` discovers chapters and
appendices, plus implementation guides; it validates reference/source bindings
and uses Pandoc and npm assets. `docs/education/generate_references.py` executes
scientific companion/expansion generators. The Pages workflow also executes
native numerical qualification, fixture generation and browser/PDF freshness
checks. The manuscript builder calls Pandoc and writes generated Word/Markdown
artifacts. These commands were inspected as source and were not executed here.

Only nonnumerical document/link/format and source-scope checks were authorized
and executed in the original research phase. Later explicit publication
authorization admits the normal book/CI/PDF/browser gates, reported at their
actual source head separately from scientific GPU qualification. No GPU/hardware
workload, GPU backend implementation or benchmark is admitted. No generated
numerical artifact or copied paper is committed. Existing author credit remains Puma.

Parent integration owns the final merge/publication decision after exact-head
CI and source/PDF/browser evidence. Nine earlier code/diagram blocks have been
replaced with prose and equations; original formulas and algorithm conditions
remain. Markdown tables need real output-format review: the current Word builder does
not have a dedicated Markdown-table renderer. A passing local link check alone
does not demonstrate readable Word tables, correct equations, public site links
or a new published edition. The new source bibliographies are linked by the
chapters; their appearance in standalone downloads needs parent build review.
