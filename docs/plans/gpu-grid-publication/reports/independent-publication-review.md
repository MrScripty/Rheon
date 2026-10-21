# Independent GPU publication source review

Review date: 2026-10-21. Publication base:
`d67aec1a4575e0a61aa8cd931c8eeec33c8dc6a3`. Imported research snapshot:
`295e9be5736a92425b4281047576d959bbc1d5b2`.
Reviewer ownership: this report only. Chapter/shared-file repairs belong to the
publication owner. Book authorship remains **Puma**.

**Final source disposition: no unresolved deciding technical, citation, license
or source-scope blocker in the frozen publication content reviewed below.**
The prose conversion, public audit and dated CPU refresh were reread. Committed
head identity, actual CI, PDF/browser results and deployment remain separate
subsequent evidence; this report does not predeclare them.
This is source/documentation review, not publication deployment or numerical,
device or performance qualification.

## Applied publication rule

The reviewer read the existing
[FEM publication source audit](../../../research-book/implementation/lagrangian-fem-sources-and-scope.md)
and [publication review](../../lagrangian-fem-book/reports/publication-review.md).
The applied rule preserves original mathematical exposition and scientific
provenance while excluding copyleft or uncertain implementation incorporation.
Scientific citation is distinct from reproducing copyrighted expression or
incorporating software. No third-party paper or proprietary API documentation
is thereby relicensed under Apache-2.0. No source, figure, substantial prose,
proof body, binary or dependency from the reviewed projects is imported.

At the publication base, `LICENSE` blob
`261eeb9e9f8b2b4b0d119366dda99c6fd7d35c64` contains Apache License 2.0 and
`Cargo.toml` blob `7d6846d80bb3d98eace08eb6d6c042642592dbdc` declares
`Apache-2.0`. These match the earlier FEM audit; this publication changes neither.
The only implementation studies retained in the GPU chapters are the inspected
AmgX and modern Flow/PhysX pins, whose relevant license/source authorities are
BSD-3-Clause. This does not approve an entire dependency graph for future reuse.

## Citation and license checks

The original independent GPU source review remains historical evidence at its
declared CPU/research snapshot. For this publication review, the reviewer again
opened the following decisive primary authorities on 2026-10-21.

| Authority | Inspected evidence and publication limit |
| --- | --- |
| [G7 Weber author manuscript](https://www.academia.edu/14777152/A_Cut_Cell_Geometric_Multigrid_Poisson_Solver_for_Fluid_Simulation) | Its author-copy header identifies the authors and volume 34(2), 2015; its abstract/body describes staggered finite-volume cut-cell hierarchy construction. Chapter 37 uses the narrow geometry-specific motivation. No rate, timing, detailed transfer formula or implementation guarantee is imported from it. Publisher/Fraunhofer failed access is retained as failed access, not full-text verification. |
| [G9 Aanjaneya author project abstract](https://orionquest.github.io/papers/EGMVL/paper.html) | Confirms authors/venue/year and coupled viscous stencils requiring box smoothing. Chapter 37's limited box-smoother statement is supported. The original strain-form equation and discussion are general exposition; no detailed paper algorithm, box-system size, Cholesky recipe, reported speedup or capacity is attributed to the unread article body. The bibliography explicitly limits evidence to the abstract. |
| [Pinned AmgX BSD license](https://raw.githubusercontent.com/NVIDIA/AMGX/91a8413ef267b1c32aff4014c02820e1c5897ac2/LICENSES/BSD-3-Clause.txt) | Reopened three-clause text, including notice/disclaimer and nonendorsement requirements. The separately inspected PCG/polynomial source SPDX headers and exact `91a8413...` pin bound the architectural comparison. No GPL implementation or dependency import is made. Bundled third-party licensing remains outside the narrow project-file conclusion. |
| [Pinned Flow/PhysX BSD license](https://raw.githubusercontent.com/NVIDIA-Omniverse/PhysX/b97365c6ddb32fda8469bc788ec18024862b0368/LICENSE.md) | Reopened BSD-3-Clause text at `b97365c...`. The inspected pressure sources carry matching SPDX headers. This is the modern PhysX Flow tree, not a license conclusion about older GameWorks Flow or every bundled component. The chapter correctly labels its centered collocated float4 divergence as different from Rheon's MAC pair. |

The earlier independent review directly reopened the other deciding literature
and official API authorities: Harris/Crane GPU Gems, Bolz institutional PDF,
McAdams author PDF, Harris reduction presentation, Bridson/Müller-Fischer notes,
NVIDIA streams/graphs/platform/capability pages, and Khronos
features/atomics/shader/synchronization pages. Their provenance and use in this
publication remain subject to the same scoped facts: historical algorithms and
API rules support original design synthesis, without becoming Rheon benchmark,
installed-device, implementation, convergence or physical-validation evidence.

The bibliography accurately distinguishes source retrieval from execution,
metadata from inspected bodies, exact repository hashes from moving vendor
documentation and source licensing from future combined-work licensing. In the
reviewer's initial session exact upstream GitHub commit timestamps were not
independently retrievable; the authors' recorded metadata check supplies those
timestamps. Pinned raw content/license identity was independently checked.

## Deciding observations and final disposition

No high- or medium-severity citation/license defect was found in the initial
publication drafts. The reviewer sent the owner these publication requirements
and boundaries for the final pass:

1. The final public source audit should expressly separate factual paper/API
   provenance from permissive implementation comparisons and state that no
   copyrighted source expression or GPL software is imported. This follows the
   applied FEM publication rule; it is not a blanket license judgment about
   scientific citations or all potential future uses. **Resolved:** the new
   [public audit](../../../research-book/implementation/gpu-sources-and-publication-scope.md),
   lines 30–68, states these distinctions, exact BSD pins, no copied software or
   substantial source expression and no GPL/copy-left dependency selection.
2. The retained CPU inventory freezes `fd7c0f0`; it must not be presented as a
   complete current-main audit. The publication base has later force-field,
   recording, particle and desktop changes. The pressure/operator/geometry
   source owners are unchanged between those commits. The original frozen
   oracle remains a sound declared comparison boundary. **Resolved:** the
   inventory's dated publication refresh at lines 20–95 explicitly retains
   sections 1–10 at the old source epoch, while describing current authored
   force evaluation, adapter density/time, custody and typed recording scope.
   Chapters 37–39 and the source plan link that distinction.
3. Root is removing the research snapshot's pseudocode/code-like diagrams.
   Final review must verify that the pressure acceptance, restart direction,
   final-budget classification, stored-velocity gate and publication ownership
   survive as prose/equations, and that no fenced code block remains in the
   three publication chapters. **Resolved:** all nine research code/diagram
   blocks are replaced. Chapter 37 lines 138–163 retains the checked PCG
   recurrence as prose and equations, including zero-problem acceptance,
   positive finite products, fresh true residual, immediate final-budget
   exhaustion and a per-iteration replacement decision. Pressure acceptance
   still precedes f32 correction and actual stored-field divergence. Chapter
   38 retains independent API custody and generation/lease-based publication.
   Chapter 39 retains allowed gauge alignment, acceptance and failure gates.

## Final source and document checks

The owner froze content and requested this final pass. The reviewer inspected
the changes relative to research commit `295e9be...` and publication base
`d67aec1...`: Chapters 37–39, the CPU inventory's dated refresh, the updated
reading/source plan, the new public publication audit, and the single additive
book README paragraph. The two GPU bibliographies remain unchanged from the
reviewed research snapshot. Retained historical plan/review files remain
historical and are not substituted as the current public source audit. No
production, dependency manifest, proof, admission, tolerance, workflow or book
builder change is present in this source scope.

Independent Pandoc JSON parsing used `markdown+tex_math_single_backslash` and
compared each original Math node as a multiset, preserving exact expression
text and display/inline association. All **25 original expressions** remain;
there are **42 current expressions** after explanatory additions. The three
chapter ASTs contain **zero CodeBlock nodes**, compared with nine in the
research snapshot. Counts by chapter are:

| Chapter | Original math | Current math | Missing original math | Current code blocks |
| --- | ---: | ---: | ---: | ---: |
| 37 | 10 | 24 | 0 | 0 |
| 38 | 0 | 0 | 0 | 0 |
| 39 | 15 | 18 | 0 | 0 |

A separate local relative-link existence scan covered the three chapters,
three GPU implementation guides and two GPU bibliographies: **181 links, zero
missing destinations**. This check does not qualify fragment anchors, public
URLs, mathematical rendering, table pagination or browser/PDF results.
Staged and unstaged Git whitespace checks also passed. Root owns its separately
reported strict KaTeX, anchor, document-fixture and eventual pipeline checks.

The reviewed content hashes below permit a later committed-head identity check
without a self-referential report hash. They identify source content, not an
artifact/deployment receipt.

| Reviewed path relative to repository | SHA256 |
| --- | --- |
| `docs/research-book/chapters/37-gpu-grid-algorithms.md` | `21475f9a70436a9faa6a3497b26b87ab447928be61331219661afa6671e81120` |
| `docs/research-book/chapters/38-independent-cuda-and-vulkan.md` | `ad4139a764e35477b6b81450abccbec1b48a70eb59df37b3f14a9a41bbc3e275` |
| `docs/research-book/chapters/39-gpu-validation-and-roadmap.md` | `03b619ba8f8dab0e194d59794137599fdefe4a3883c7e423b4cb86cba4d06ec8` |
| `docs/research-book/implementation/gpu-cpu-oracle-inventory.md` | `72c9c9a2d39a63a75ec5e25b5ac549740cc32fc610078623819b1c641a8c104f` |
| `docs/research-book/implementation/gpu-research-plan.md` | `e95f2b1c4505317616de096747562215da4f4c6d2170c5aa43fb79c62ad8d384` |
| `docs/research-book/implementation/gpu-sources-and-publication-scope.md` | `c536c28145503698c5d3545ede1d4621fbde5c245707256de6ff3d251344aa8c` |
| `docs/research-book/sources/gpu-algorithm-bibliography.md` | `ce9ca2e928f30ef2e5e0c5cdf266cb0778d5ee6f0080e243625b08026c6bf509` |
| `docs/research-book/sources/gpu-backend-bibliography.md` | `8f9c10e6be93f19d41c3b2d5a63d6817c6860ca563bd74441efe95c50ea2f491` |
| `docs/research-book/README.md` | `d9581edf7218857166ea99842d9c3621b356e7049754601a91726127aab6ae6b` |

## Evidence boundary and remaining gates

No source-review action was denied or left with an uncertain mutation outcome.
The metadata access limit above remains a disclosed reviewer limitation, not a
claim of independently checked timestamps. There is no remaining source-content
blocker. A later exact-head identity review and the parent-owned normal
CI/PDF/browser gates are still required before merge/publication acceptance.
No solver/scientific fixture, GPU workload, hardware enumeration, workflow,
installation, security change, full book build or deployment was run by this
reviewer. The reviewer has not edited any chapter, bibliography, production
source, shared navigation, build or workflow file. Any later normal pipeline or
deployment evidence belongs to its actual runner and exact revision.
