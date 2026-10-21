# GPU research: publication sources, licenses and evidence scope

Book author: **Puma**. Publication preparation: 21 October 2026. This guide
accompanies Chapters [37](../chapters/37-gpu-grid-algorithms.md),
[38](../chapters/38-independent-cuda-and-vulkan.md) and
[39](../chapters/39-gpu-validation-and-roadmap.md). Their prose, equations and
derivations are original exposition. The publication replaces the earlier
research draft's nine code/diagram blocks with explanatory prose and explicit
recurrences; it contains no pseudocode, copied implementation code or new GPU
backend. CUDA and Vulkan remain independently usable proposed compute paths.

## Preserved research and refreshed CPU scope

Original research commit `295e9be5736a92425b4281047576d959bbc1d5b2` remains
preserved in the publication branch's ancestry. Its CPU oracle inventory pins
`fd7c0f0aee63bc8710df99f9bb01154e7619b974`, not an unspecified latest solver.
Publication preparation is based on main
`d67aec1a4575e0a61aa8cd931c8eeec33c8dc6a3`, after FEM PR147, force/backend
PR145, scene UI PR146 and portal PR148. No production physics, force eligibility,
recording schema, proof, tolerance or admission manifest changes are made here.

The [CPU inventory](gpu-cpu-oracle-inventory.md) retains the original numerical
baseline and adds a dated force/recording refresh. The pressure/operator/geometry
files are unchanged across these bases; authored immutable force scenes and
their MAC/particle/recording paths are additional admitted CPU capabilities.
Those additions do not establish a device-resident interface or a general
free-surface/coupled-fluid solver. The [reading/source map](gpu-research-plan.md)
connects stages, model limits and proposed implementation gates.

## License-compatible implementation references and scientific provenance

Rheon's [LICENSE](https://github.com/MrScripty/Rheon/blob/d67aec1a4575e0a61aa8cd931c8eeec33c8dc6a3/LICENSE),
blob `261eeb9e9f8b2b4b0d119366dda99c6fd7d35c64`, contains Apache License 2.0.
[Cargo.toml](https://github.com/MrScripty/Rheon/blob/d67aec1a4575e0a61aa8cd931c8eeec33c8dc6a3/Cargo.toml),
blob `7d6846d80bb3d98eace08eb6d6c042642592dbdc`, declares `Apache-2.0`.
Neither file is changed by this publication.

The user's publication rule excludes incompatible or unestablished software
reuse. The retained implementation comparisons are the exact AmgX and modern
Flow/PhysX source snapshots in the [algorithm bibliography](../sources/gpu-algorithm-bibliography.md).
Their inspected license authorities and selected source headers are BSD-3-Clause:

- AmgX commit `91a8413ef267b1c32aff4014c02820e1c5897ac2`, version-file value
  2.5.0: [license authority](https://github.com/NVIDIA/AMGX/blob/91a8413ef267b1c32aff4014c02820e1c5897ac2/LICENSES/BSD-3-Clause.txt).
  Selected PCG, polynomial smoother and configuration source supports factual
  algorithm-composition comparisons. It is a CUDA library, not a Vulkan backend
  or a complete Rheon fluid solver.
- Flow within PhysX commit `b97365c6ddb32fda8469bc788ec18024862b0368`:
  [root license](https://github.com/NVIDIA-Omniverse/PhysX/blob/b97365c6ddb32fda8469bc788ec18024862b0368/LICENSE.md)
  and selected pressure-source SPDX headers. These support sparse pressure-pass
  orchestration and collocated-shader observations. This is not the older Flow
  repository's unestablished license and not a MAC-compatible drop-in.

No external code, substantial prose, figures, binaries, libraries, SDKs or
datasets are incorporated. No GPL/copy-left implementation dependency is selected
or imported. BSD notice, disclaimer and nonendorsement obligations would matter
to actual redistribution; this source comparison does not admit an entire
dependency graph or certify every bundled component. Future code adoption needs
the exact component/version, notices, transitives and distribution audit.

Original papers, course notes, NVIDIA technical chapters and official CUDA and
Vulkan documents supply factual and scientific provenance. They are not being
redistributed or relicensed under Apache-2.0. Reading a mathematical identity or
API requirement is distinct from copying copyrighted expression or incorporating
licensed software. No vendor code sample, paper illustration, proof body or
substantial quotation is reproduced. Local recurrences, error metrics, resource
models, schedules and architecture recommendations are the book's original
synthesis under their stated assumptions.

## Exact supporting claims and access limits

The [algorithm bibliography](../sources/gpu-algorithm-bibliography.md) owns exact
paper identities, versions, dates, selected source paths and retrieval limits.
The [backend bibliography](../sources/gpu-backend-bibliography.md) owns official
CUDA/Khronos sections and moving-document revision limits. Access was recorded
on 2026-10-21; it is not an installed SDK or tested driver identity.

| Authority | Admitted use and limit |
| --- | --- |
| Harris GPU Gems 38 (2004), G1 | Historical rectangular 2D grid-pass decomposition; no free-surface or current Rheon speed claim |
| Crane/Llamas/Tariq GPU Gems 3 chapter 30 (2007), G2 | Historical 3D pass/advection exposition and its collocated/MAC distinction; relevant sections, not a claim of whole-method parity |
| Bolz et al. (2003), G3 | Primary GPU sparse-CG/regular-multigrid formulation and reduction dependencies; historical hardware and stopping conditions |
| McAdams/Sifakis/Teran (2010), G4 | Boundary-aware multigrid-PCG ideas; the inspected implementation/timings are shared-memory CPU evidence |
| Harris reduction presentation, G5 | Hierarchical reduction construction; no invented publication date or modern API synchronization authority |
| Bridson/Müller-Fischer notes (2007), G6 | Pressure, MIC and boundary derivations in inspected sections; not a GPU triangular-solve implementation |
| Weber et al. (2015), G7 | Inspected author-manuscript motivation for geometry-specific cut-cell hierarchy; publisher full text unavailable, no imported rate/timing or general-geometry guarantee |
| Aanjaneya et al. (2019), G9 | Author-abstract-level coupled-viscous/box-smoother point only; full paper unavailable within retrieval limits, no detailed smoother reconstruction or timing claim |
| Official NVIDIA guide/runtime sources | Selected execution, memory, graph, arithmetic and capability requirements; no entire SDK audit, hardware query or supported-model certification |
| Official Khronos specification/reference pages | Selected compute, resource, feature, subgroup and synchronization requirements; not a whole-specification read or a requirement that every Vulkan device supports f64 |

Relevant sections of G2, G5, G6 and vendor documents were consulted, rather than
asserted cover-to-cover readings. G7 uses author-manuscript text, not a successful
publisher full-text retrieval. Exact AmgX/Flow commit timestamps were checked by
the bibliography author; the independent reviewer reopened pinned source and
licenses but could not reproduce that metadata request. These limits are retained
in the public ledgers rather than silently promoted to full verification.

The consulted Vulkan documentation identifies build 1.4.365, generated
2026-10-02 from `8c9361ba8180c1f4164c0bf79de2f6e817770b0d`. Checked CUDA Guide
footers identify 2026-09-10; the separately consulted Runtime API page identifies
13.4. These moving references do not identify an installed toolkit or set a
minimum runtime profile. Optional FP64, atomic and subgroup behavior must be
queried and enabled for each later implementation/device configuration.

## Publication evidence is separate from solver qualification

The original independent source review repaired pressure/stored-field acceptance
ordering, per-iteration replacement state, final-budget failure classification,
gauge comparison and a chapter link. The publication continuation independently
reviews the prose-only recurrences, updated CPU/source scope and license/access
limits. Final commit, exact-head CI, actual PDF visual inspection, source/link
and browser evidence belong to the publication completion report; this guide
does not predeclare those results.

The existing builder discovers all three chapters and this implementation guide.
Chapter bodies enter the normal reading/print/PDF edition. This audit is a
separate HTML implementation page and packaged Markdown source; its full body
is not automatically appended to print/PDF because that sequence uses the
existing explicit implementation-guide list. No fabricated interactive GPU lab
or benchmark is added. Existing FEM 30–36, portal 40 and thermal-owned paths are
outside this task's write scope.

A successful book pipeline qualifies its actual source/artifact and publication
checks. It cannot establish an implemented GPU solver, numerical CPU/GPU parity,
physical convergence, hardware support, restart equivalence or measured speedup.
Those remain prospective, separately authorized gates in Chapter 39. The earlier
Library preview and uncertain upload state are retained unchanged; repository
publication is a distinct operation and does not retry that upload.
