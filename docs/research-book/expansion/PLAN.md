# Solids, liquids and teaching edition

Owner: Puma. Research branch: `research/solids-liquids-education`, based on published comparison integration `a97a2dbce6b794456b634d87e2b111c059a9f696`. The coordinator owns subsequent production repairs and stack integration. This lane does not change production Rust, Cargo, comparison harnesses, historical receipts, or PR targets.

## Audited gaps

Chapters 8–10 already distinguish boundary work, free-surface traction and scalar diffusion. Their fixtures do not assemble arbitrary collision meshes, transport bounded liquid volume in three dimensions, or solve moving contact lines. The 20 historical Lean theorems prove finite real algebra, interpolation bounds, restricted 1D positivity and natural-number indexing. They are not a verified implementation.

## Path-scoped work

1. Append chapters 19–25 and appendix F. Derive physical and discrete contracts for forces, moving solids, mesh queries, density, interface volume, tensor viscosity, wetting, adhesion, slip and capillarity. Retain earlier chapters and identify their narrower scope.
2. Add `proofs/Rheon/Physics.lean`, import it into the existing pinned project, extend the explicit axiom inventory and review digests. Prove weighted conservation, prescribed-boundary compatibility, force work, implicit dissipative energy, slip dissipation and Young balance implications. State every hypothesis; do not claim geometry, solver or IEEE refinement.
3. Put original reference implementations, shared JSON, original plots, source registry and qualification under `expansion/`. Independent hand/analytic checks qualify each numerical fixture. This is educational reference code, not a production physics backend.
4. Build `docs/education/`: full book, source/proof display, downloads, local math rendering and progressive 3D labs sharing the JSON and calculations used by figures. Publish via GitHub Pages Actions after the coordinator integrates the research branch. No Sites hosting.
5. Deliver a complete Markdown edition, illustrated PDF and site/source archive through Library. Commit milestones without force pushes; preserve original evidence bindings.

## Acceptance

Exact pinned Lean build and transitive axiom audit, reference checks and reproducible data, math rendering without errors, complete navigation/source coverage, browser control and mobile checks, PDF page inspection, disjoint-path verification and artifact hashes. Missing external/tool qualification is reported precisely rather than relabeled as success.
