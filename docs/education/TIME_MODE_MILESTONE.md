# Modal time integration continuation

Goal: make time-step stability, directional fidelity and accuracy separately
observable, with an independently checked physical reference and scoped proofs.

Branch: `research/book-time-mode-continuation`, based on PR44 `793b4d6`.
PR44 and PR43 are preserved. No persistent-goal API is exposed; this record and
the execution plan retain the goal. No local AGENTS.md or .agents skill appeared.

## Plan and acceptance

1. Monitor final PR44 CI; inspect AI/CAD handoffs: read exact PR49 `b8ca87c`
   and PR50 `3f49a15` implementations and scope records. Their native capabilities
   remain unmerged and separate from this lesson.
2. Implement chapter 12 periodic shear with actual modal updates, fixed final
   time refinement, fixed-scale 3D velocities and compact proof discussion.
3. Verify all 402 discrete states with rational powers/high-precision exponential;
   prove scalar amplification/energy/sign properties with fresh pinned Lean;
   check actual rendered glyphs and controls at desktop/390px/320px.
4. Inspect source-bound screenshots and final CI, push a dedicated stacked draft.
   No native solver edits, main merge or publication/deployment.

## Research and proof boundary

Primary source: Jos Stam, Stable Fluids (1999), author-hosted PDF, introduction
and sections 2.2–2.3, read 2026-10-13. Chapter 12 cites the exact URL and explains
why its split solver is broader than this one-mode time-integration reference.
The analytic shear ansatz removes advection and spatial discretization from the
comparison. Lean proves only exact scalar amplification contracts, not the PDE,
trigonometric integration, continuum convergence, execution or floating point.

Current source inventory status is in `native-sequence.json`. Updating hashes
never promotes a pending kernel check. PR44's exact-source receipt is historical.
The new source changes no Cargo.lock/native admission binding and uses no absent
native packet as substitute data. Local Lean bootstrap and sandboxed Chromium
restrictions from the first milestone remain; supported hosted verification is
required rather than changing those security boundaries.
