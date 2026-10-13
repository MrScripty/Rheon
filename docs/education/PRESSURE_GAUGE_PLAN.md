# Pressure compatibility and gauges: bounded research plan

Owner: Rheon book research lane. Date: 2026-10-13.
Branch: `research/book-pressure-gauges`, stacked on PR60 `900e30b5807497c3c2bb20e3d7d0fed6d9e42cc1`.
PR60 stays unchanged and its run 38458741142 remains independently monitored.

## Question, model and limits

Which right-hand sides can an all-Neumann pressure operator balance, and what
can a pressure gauge change? Use two disconnected two-cell components with
positive face weights and volumes. Distinguish integrated equations from
volume-normalized divergence, and finite gauge choices from physical pressure
boundary conditions. No production solver, native admission hash, dependency,
mesh reader, main merge or deployment changes belong to this slice.

## Milestones and acceptance

1. Read primary developer/research sources and record exact applicability and
   access limits. Derive complete compatible and incompatible numerical cases,
   including global cancellation that hides opposite component imbalances.
2. Implement a recomputed chapter-5 3D lesson with component balances, actual
   pressure differences/face correction arrows, independent component shifts,
   fixed-cell/arithmetic-mean/volume-mean gauges, full residual and boundary-work
   diagnostics. Incompatible data must remain visibly rejected. A projected-RHS
   comparison must disclose its source change, never silently repair the input.
3. Prove finite exact-real component cancellation, gauge invariance, compatibility,
   fixed-cell residual counterexamples and pressure-work shift identities in Lean.
   State positive/nonzero assumptions and geometric limits. Extend the explicit
   axiom audit and proof inventory; preserve prior qualification as historical.
4. Validate all finite controls with an independent rational oracle, mutation
   counterexamples, desktop/390px/320px actual rendering, scroll/deep-link presets,
   controls and collapsed proof checks. Run book/PDF/links and pinned Lean CI.
5. Review signs, units, gauge/compatibility distinctions and final sources; commit,
   push and open a draft stacked PR. Report exact-head CI and screenshot-access
   limits. No broader CFD, IEEE, Rust refinement or performance claim follows.

## Evidence and provenance

Generated builds, screenshots and logs remain outside Git. Only authored source
and compact manifests are committed. Existing locked Three.js and Lean/mathlib
are reused. No new dependencies, external mesh/model data, GPL implementation
or upstream source/figures are copied. PETSc documentation is a cited research
source, not a linked or bundled solver. Denied downloads are not bypassed.

## Status

Plan recorded before implementation. PETSc primary nullspace notes and
Gebhardt–Romero v1 sections 1–2 have been read; the Bochev–Lehoucq full text
remains inaccessible (publisher abstract/metadata only). The complete derivation,
288-state independent rational oracle, seven heading presets and 15 scoped Lean
statements are implemented. Normal/optimized reference tests, source-policy,
metadata and publication-gate checks pass locally; a fresh HTML edition builds.
Fresh kernel and hosted browser/PDF/full-workflow qualification remain pending.
PR60 stays unchanged and its CI is monitored independently.
