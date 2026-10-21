# FEM manuscript review and qualification boundary

This is an authored research draft for Puma's book, starting from verified Rheon main `fd7c0f0aee63bc8710df99f9bb01154e7619b974`. The [source map](source-map.md) owns bibliography, versions and licenses. The [plan](../../plans/lagrangian-fem-book/plan.md) owns scope/status. Chapters30–36 contain approximately ten thousand words, exact-real derivations, hand examples and implementable pseudocode; no new scientific/native/hardware/machine-proof execution is claimed.

## Mathematical and source review questions

- Does every motion/gradient/stress formula use its stated reference/current domain and traction sign?
- Do residuals and tangents differentiate the same geometry, quadrature, constraints and boundary loads?
- Are conditional energy, volume, injectivity and mixed-stability statements limited to their actual hypotheses?
- Can the solid and fluid pseudocode be implemented from the preceding equations without inventing a missing pressure, mass or geometric derivative?
- Are endpoint volume constraints distinguished from endpoint divergence and local density errors?
- Do source versions and per-file license authorities support the architecture descriptions, without copying third-party code?
- Are existing Rheon fitted-height/native ALE results described at their restricted scope, with Puma and original source authors credited?

Independent review and dispositions are recorded in [the review report](../../plans/lagrangian-fem-book/reports/independent-review.md); checks in [verification](../../plans/lagrangian-fem-book/reports/verification.md). Neither report is a standards-engine or mathematical machine-proof certificate.

## Build execution scope inspected before checking

`docs/education/build.py:build()` invokes `verify_reference(BOOK/'expansion')` before rendering. That function reads the stored qualification and checks source/data hashes; it does not itself execute the reference. Build also invokes native-sequence, obstacle, obstacle-flow, aligned-strain, solver and compressible packet validators. Some validators invoke verification subprocesses, exact-source qualification or scientific replay; supplied packet/source bindings matter. Imports and helper calls must be inspected as part of any future execution plan. A `--help` or source-only-looking argument is not assumed harmless without tracing the entrypoint.

The historical `docs/research-book/build_manuscript.py` does Pandoc/python-docx assembly but includes historical front matter/evidence counts and cannot establish current complete teaching-edition qualification. Neither full builder is run for this research task. Existing qualification manifests and thresholds are unchanged.

Initial checks are limited to `git diff --check`, Markdown links/order/fence/equation delimiters, and direct Pandoc parsing/rendering of the authored Markdown. They execute no companion experiment, FE computation, Rust example, Lean build, model download, browser worker or hardware benchmark. Generated local review HTML is an equation/text preview only, with no solver and no deployment.

## Pending bounded scientific qualification

The exposition's solid P1 backward-Euler and fluid P2/P1 endpoint-volume equations require actual implementations before numerical claims can exist. Parent or a future implementation owner must separately admit an exact small-case execution set, independent oracle, finite DOF/step/Newton/Krylov/backtracking budgets and wall/CPU/memory/output bounds. Do not infer permission for a parameter sweep or existing full-book qualification from this document.

Recommended first evidence: reference tetra mass/shear force, energy/stress/tangent derivatives, scalar implicit step and pressure block hand oracles; then a small constrained 3D solid and fixed-domain mixed Stokes case. Separately qualify nonlinear fluid geometry, pressure rank/stability, local/total volume error and finite-step pressure work. Derivative checks must include geometry and follower loads. Spatial/time/quadrature refinements need independently derived smooth solutions and distinct error budgets.

P2 geometry safety needs a real bounded implementation of a sufficient certificate, such as chapter36's restrictive convex-domain gradient bound, or a stronger reviewed determinant/path/intersection certificate. The exact-real argument is not a floating-point certificate. Contact, interface transfer, remeshing, fracture/topology change and compressible shock/thermal branches require separate state, equation, oracle and failure qualification. No completed-run receipts or fabricated proof output are supplied.

## Integration coordination

The FEM branch owns new chapters30–36, this directory and its plan. Shared edits are additive links in book README, chapters11/29 and fitted-height/fitted-ALE guides. AppendixC, teaching/build scripts, solver source and scientific admission files remain unchanged. A separate CUDA/Vulkan research task owns GPU treatment; it should select chapter numbers with parent to avoid collisions and add cross-links only after both drafts are reviewed. Parent owns publication, merging, deployment and final book-wide qualification.

The follow-up [research status report](research-status-report.md) audits source access and coverage and keeps full research closure open. M1 acceptance records a manuscript milestone only; it must not be presented as complete original-literature review, a fully specified compressible/ALE solver or run-validated implementation.
