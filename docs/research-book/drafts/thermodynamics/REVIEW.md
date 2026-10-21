# Thermodynamics and liquid–gas phase change: review guide

Book author: **Puma**. Research date: **21 October 2026**. Status: **local research manuscript; unpublished**. This work owns new documents only under `docs/research-book/drafts/thermodynamics/`. It makes no production, shared-index, README, workflow, proof/admission, publication, PR, merge or deployment change.

## Reading order and proposed chapter numbers

1. [41 Temperature, energy and entropy](41-temperature-energy-and-entropy.md): state variables, thermodynamic potentials, conservation, first/second laws and temperature equations.
2. [42 Heat transport and boundaries](42-heat-transport-and-boundaries.md): Fourier flux, variable properties, energy transport, boundary exchange and conditional discrete arguments.
3. [43 Thermal expansion and convection](43-thermal-expansion-and-convection.md): EOS dilation, Boussinesq reduction/breakdown, natural/forced/mixed convection, nondimensional regimes and thermocapillarity.
4. [44 Liquid–vapor equilibrium](44-liquid-vapor-equilibrium.md): chemical potential, saturation, Clapeyron, vapor–air mixtures, curvature, metastability, flash/cavitation and critical-region limits.
5. [45 Interfacial balances](45-interfacial-mass-momentum-and-energy.md): moving-interface mass, volume creation, pressure compatibility, recoil/capillarity, Stefan energy and species coupling.
6. [46 Numerical methods and coupling](46-numerical-methods-and-coupling.md): conservative sharp interfaces, level set/ghost fluid, phase field, enthalpy, low Mach/compressible and particle alternatives.
7. [47 Validation and conclusions](47-validation-and-research-conclusions.md): analytic derivations, published benchmark families, evidence limits and realistic staged conclusions.

Main was fetched successfully and inspected at `b9218038d138d1c73818cf4158757521853a5cd7`. Its chapter directory ends at 36 and includes the published FEM expansion. The task identifies GPU37–39 as a separate unmerged draft. Read-only GitHub/remote inspection confirmed portal PR148 open/unmerged at `4b0cec9265a968385dc1361de75023e696d8a927`, with Chapter40. Numbers **41–47 are therefore proposed**, not globally reserved by a shared edit. Recheck numbering with the integration owner before eventual publication.

The existing manuscript and Pages builders glob the canonical chapter directory. Drafts are intentionally outside that directory and all builder inputs/discovery files remain unchanged. Later integration must be a separately authorized operation: reconcile current main/numbering, move reviewed chapter sources, fix relative links, integrate source/scope material through the existing convention, then run the normal publication checks. No publication gate is claimed complete here.

## Reviewable supporting documents

- [Current-solver gap map](solver-gap-map.md) connects research needs to inspected CPU/MAC, liquid-volume, particle and ideal-gas source.
- [Primary bibliography and access scope](bibliography.md) states precisely what was read, what each source supports and what remains unread.
- [Source/license audit](source-license-audit.json) separates citation, permission to reuse expression/code, exclusions and access failures.
- [Repository audit](repository-audit.json) pins inspected source blobs and SHA-256 identities; these are provenance, not numerical qualification.
- [Review and verification notes](review-notes.md) records independent technical/source review, corrections and actual local checks.

## Scientific and rights boundaries

The text is original prose, equations, conditional derivations and hand references. There is no pseudocode, copied implementation, third-party figure/table, reproduced scientific proof, external dependency adoption or new measured solver result. Reference-state presets are not authoritative thermophysical tables. Analytical identities do not establish stability/convergence of an unspecified discretization; article-reported results do not become Rheon results.

Rheon is **Apache-2.0**, confirmed from current `LICENSE` and `Cargo.toml`. Scientific citation identifies facts and provenance; it does not import a paper's copyright license into Rheon or confer reuse permission. Restrictive/unknown code and figures are excluded. The inspected CC BY-NC-ND microlayer study is excluded as chapter authority under the conservative source-selection rule. CoolProp's pinned MIT root license supports a limited factual implementation study; no code is incorporated and no complete future dependency graph is approved.

Research/local commits are authorized. Production implementation, publication, pushing/opening a PR, merging and deployment are outside this task. No thermal scientific simulation, native build, Lean proof, GPU or hardware benchmark was run. Source/math/render checks have only their named documentary scope.

## Repository instructions and isolation

The original `/workspace/Rheon`, branch `work` at `6ba41c7`, was clean at inspection and was not switched or edited. A task-created worktree `/workspace/rheon-thermal-research` on `research/thermodynamics-phase-change` starts at fetched main. Fetching remote objects updates repository object/ref metadata; it does not integrate or publish another branch's changes.

No `AGENTS.md`, `SKILL.md`, or populated applicable `.agents/skills` was present in the inspected repository/ancestors; `/workspace/.agents` and `.codex` were empty. Existing accepted plans identify adopted `MrScripty/Coding-Standards` revision `188beda1fa477d21d576c233dd8c7f4c4c267d23`. Core, Router, documentation, verification, licensing, commit and its implementation prerequisite were read from that exact source. The research deliverable is the task itself; there is no admitted production plan to execute. Development proportionality was read for scope: completing requested research does not authorize replacing it with implementation. No language/UI/runtime/release profile is selected because those mechanisms do not change. This is source-guided application, not an automated standards-engine certificate.

The local branch/worktree is **retained-protected** for parent review: its head remains reachable from the named local research branch. Research task owns the new documents; parent owns acceptance/integration. No automatic rebase, shared-history rewrite, cleanup of other resources or synchronization with concurrently advancing publication work is permitted. Next disposition: parent reviews this draft, selects follow-up, and later arranges safe integration or retirement. Retaining the worktree is deliberate handoff, not abandonment.
