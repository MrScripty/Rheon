# Parent handoff for a later draft PR

No remote push or PR creation was performed by this worker. Parent coordination
and review own the next publication decision. A push/PR can trigger the existing
research-book workflow's scientific/native qualification; creating a draft PR
does not itself prevent those jobs. The current task prohibits those workloads,
so parent must resolve that scope before using the normal remote workflow. This
note does not propose bypassing or disabling any gate.

Suggested title: `docs(research): expand GPU grid algorithms with independent CUDA and Vulkan`

Suggested description:

> Rheon's book has CPU memory/scheduling guidance but lacks a substantial GPU
> treatment grounded in its actual MAC solver. Add Chapters 37–39 covering
> compatible grid algorithms, independently usable CUDA/Vulkan compute paths,
> and source-identified CPU validation, residency, lifecycle and benchmark
> milestones. Include the current CPU inventory and verified primary-source
> bibliographies. Puma's author credit is preserved.
>
> The initial port target remains the bounded sealed constant-density MAC box.
> Particle appearance voxels, reduced surface/obstacle models and future general
> 3D coupled physics are explicitly distinguished. No backend, dependency, product
> source, proof or numerical threshold changes.
>
> Validation: independent technical/source review with deciding findings repaired;
> local source/link/fence/math-delimiter/whitespace checks and isolated Pandoc
> parsing only. No full book build, render, scientific/hardware workload or GPU
> qualification ran. All performance and device-support claims remain prospective.

## Exact integration boundaries

Use the [source map](../../../research-book/implementation/gpu-research-plan.md)
for every authored book path. Chapters 30–36 belong to the parallel FEM draft
`9489aa3c1bd6652f9ab1874fce904853b9063dc3` on
`research/lagrangian-fem-book-20261021`. Only the new GPU chapters were renamed
37–39 after this parent instruction. This branch does not include or modify FEM.

Shared book README, Chapters 11/29, fitted-height/fitted-ALE guides, TOC/build
scripts and Appendix C are untouched. Parent should integrate the two branches'
new material serially and decide shared navigation/coverage/reference placement.
The current builder discovers new chapter files, but reading/PDF/Word and source
bibliography distribution still require their normal scope/freshness/output review.

Review this research branch against its admitted base
`fd7c0f0aee63bc8710df99f9bb01154e7619b974`, then inspect newer main for relevant
CPU/source changes. Do not infer numerical or lifecycle compatibility after a
changed CPU oracle merely because Markdown applies cleanly. No rebase, merge,
deploy or publication is authorized by this handoff.
