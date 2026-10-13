# Mesh faces to collocated flux: bounded research plan

Owner: Rheon book research lane. Date: 2026-10-13.
Branch: `research/book-engineering-flux`, based on PR51
`33094fb975753e35645ad807cf01015de041779e`, preserving PR43 → PR44 → PR51.
No production solver, native admission hashes, Cargo dependencies, main merge,
or deployment is authorized by this plan.

## Question and finite scope

How do oriented shared faces, prescribed boundary flux, pressure interpolation
and symmetric strain constrain an engineering discretization? Stage one is an
independent teaching reference, not an imported-mesh backend or CFD validation.
Use two unit cells, a four-cell periodic pressure strip, and affine planar
velocity fields embedded in a unit cube. State the reduced dimension explicitly.

## Milestones and acceptance

1. Read original/developer-authored sources; record equations, applicability,
   access limits and licences. Derive signs, units and complete numerical cases.
2. Implement bounded references and chapter-linked 3D controls for face work,
   checkerboarding and rotation/shear. Camera changes must not alter physical
   values. Check independent exact arithmetic, invalid inputs, actual glyph
   geometry, desktop and 390/320px controls, deep links and collapsed proofs.
3. Add exact-real Lean statements for shared incidence cancellation, two-cell
   boundary work, the periodic checkerboard witness and affine strain power.
   Review assumptions, extend the explicit audit and source inventory; require
   a fresh pinned kernel build. Prior proof receipts stay historical.
4. Update chapters 4/8/22, a research dossier and the coverage ledger. Run book,
   math/link/PDF, reference, source-policy and hosted UI/Lean checks. Review the
   final diff against independent equations before pushing a draft PR on PR51.
5. Only after that stage qualifies, select the next finite reconstruction or
   coupling study. General Rhie–Chow/SIMPLE, imported meshes, nonlinear/time
   convergence, engineering benchmarks, SST, AMG and learned acceleration are
   later stages, not deliverables silently claimed by the first lesson.

## Evidence and refusal policy

Distinguish authored derivation, exact theorem, computed teaching fixture,
native execution, IEEE correctness, physical validation and runtime measurement.
All generated builds/screenshots/logs stay outside Git; compact authored source
manifests may be tracked. A missing/denied tool or screenshot remains unverified;
do not bypass denials. No numerical acceptance from a bibliography or animation.

## Dependencies and provenance

Use the existing locked Three.js/viewer and pinned Lean/mathlib only; add no
runtime, parser, mesher or solver dependency. No Gmsh binary, SDK, wrapper,
bundled output corpus or GPL implementation is imported. Mathematical formulas
are derived independently and attributed; do not copy upstream source or figures.
Record exact existing lock identities and this stage's dependency delta in the
dossier. Future reader/AMG/model choices require separate exact-version and
transitive/data/checkpoint licence reviews.

## Status

Environment and pushed PR51/main identities checked. The dossier, three bounded
references, chapter controls, independent rational checks and 14 Lean statements
are implemented. All 126 reference states pass locally in normal/optimized
Python. The pinned kernel and 282-declaration axiom audit passed on `dd0d81c`
(workflow 38449564095; merge proof tree verified identical). Initial hosted
three-viewport glyph/control checks also passed. Strengthened control, proof
and true-scroll checks plus final full-book CI are pending on the continuation.
The supported screenshot download returned HTTP 403; human visual review remains
blocked. Final results must identify tested commits, not advance historical
receipts by rewriting a hash.
