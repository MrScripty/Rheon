# Rheon Discrete Fluid Simulation

A research book credited Puma: mathematics, algorithm specifications, worked examples, reproducible numerical fixtures and checked Lean contracts for a low-memory 3D fluid framework.

The dedicated [Lagrangian FEM research expansion](implementation/lagrangian-fem-sources-and-scope.md) adds chapters [30](chapters/30-material-motion-and-conservation.md)–[36](chapters/36-reference-fem-solver.md): material motion and conservation, stress/weak forms, element operators, time/nonlinear solution, compressible and mixed fluids, contact/ALE and two fully specified minimal reference algorithms. The [publication source and scope audit](implementation/lagrangian-fem-sources-and-scope.md) records the solid/gas choices, license exclusions and remaining qualification. Chapters30–36 use prose, formulas, conditional proofs and worked examples without pseudocode. This is a reviewed draft exposition, not a general FEM implementation or new scientific qualification. The audit separates manuscript review from future numerical evidence. Puma's author credit and earlier research remain intact.

The GPU research expansion adds Chapters [37](chapters/37-gpu-grid-algorithms.md)–[39](chapters/39-gpu-validation-and-roadmap.md): compatible grid algorithms, independent CUDA and Vulkan compute approaches, and CPU-reference validation, device residency and staged implementation requirements. The [source/license and publication-scope audit](implementation/gpu-sources-and-publication-scope.md) preserves inspected-source/access limits and distinguishes original mathematical exposition from software reuse. This is prose-and-equation research without pseudocode, GPU implementation, hardware qualification or measured speedup; Puma's credit and existing numerical boundaries remain intact.

The [fluid portals and open boundaries chapter](chapters/40-fluid-portals-and-open-boundaries.md) adds Chapter 40, reserving 37–39 for separate GPU research. It develops periodic topology, hydrostatic reservoirs, weak/MAC conditions, conditional conservation and energy proofs, backflow and particle requirements. The [source/scope guide](implementation/fluid-portals-sources-and-scope.md) and [implementation proposal](../plans/fluid-portals-open-boundaries/plan.md) separate reviewed mathematics, publication evidence and future numerical acceptance. No portal implementation or new scientific qualification is claimed.

The repository contains bounded Rust box/liquid components and separate research prototypes. General moving mesh/interface composition, coupled physical boundaries, particles and GPU sections remain incomplete or scoped research specifications. Exact proofs do not establish floating-point accuracy, geometry assembly, solver convergence, physical calibration or performance.

## Read and reproduce

Chapters and appendices are editable Markdown. The reading PDF and native-equation Word book are delivered through Library. The source archive preserves all inputs needed to rebuild them.

From this book directory, using Python 3.12:

    python -m pip install -r companion/requirements.txt
    python companion/experiments.py
    python companion/depth_experiments.py
    python companion/make_figures.py

For Word generation, install python-docx 1.2.0 and lxml 6.1.1 and provide Pandoc 3.1.11.1 on PATH. Run:

    python build_manuscript.py

Render the resulting DOCX with LibreOffice and inspect every page before delivery. The qualified render used LibreOfficeDev 26.8.0.0.alpha0, Liberation Serif, DejaVu Sans and DejaVu Math TeX Gyre. The builder normalizes OMML delimiter-property order to prevent closing-delimiter misrendering in that renderer. Equations remain native, not rasterized.

## Evidence

The companion README states the limits of each executed mechanism fixture. Proof qualification and source hashes are in evidence/proof-qualification.json. The separate repository-root proofs directory owns the pinned Lean project; in the standalone source archive it is included beside this README.

All generated cover artwork is original to this edition, with prompts and hashes in artwork. Figures are derived from the companion scripts. The bibliography links primary sources rather than redistributing their illustrations.

## Expanded solids and liquids teaching edition

Chapters 19–25 and Appendix F add forces, moving collision geometry, density/volume, tensor viscosity, separate wetting/adhesion/slip, capillarity and a future fixture matrix. The historical expansion has 42 source-bound public Lean theorems, including bounded planar clipping and a finite-strain backward-Euler work bridge, six interactive 3D local-reference labs and original figures. The connected unmerged PR22–24 progression adds finite Navier traction, compatible no-slip and uniform body forcing, with their actual distinct fixed-slab models and original recorded playback. Its accepted kernel proof inventory has 57 public theorems / 77 audited declarations; the older 42/60 receipts remain historical. This documentation integration changes no production source. Build/download instructions are in `../education/README.md`; the expanded Markdown/PDF preserve the original 18 chapters. New evidence lives in expansion, separate from historical receipts.

The [native wall/force progression](implementation/native-wall-force-sequence.md) connects the equations, units, proof assumptions and recorded demos. The [requirements roadmap](implementation/requirements-roadmap.md) maps actual collision, force, viscosity, adhesion and density coverage and distinguishes current static collision/import capability from the remaining conservative fluid-topology/stress assembly. Generated site, PDF, assets and qualification receipts stay outside Git. The source-only CI hub explicitly identifies unavailable recorded bundles.

The [practical mechanisms](engineering/practical-methods.md) add wall/work, sparse iteration, memory, arithmetic, transfer, constitutive, refinement and proposal-gate examples. The [coverage ledger](../education/COVERAGE.md) records partial placements in all 25 chapters and six appendices, exact checked proof source and pending publication/UI evidence.

## Compressible research extension

Read Chapters 26–28 in order after the continuum/transport foundations. They
specify conservative thermodynamic state, gas/liquid EOS, waves, fluxes,
admissibility, compressible boundaries and a staged validation program. See the
[coverage audit and implementation roadmap](implementation/compressible-flow-roadmap.md),
[computed Euler reference](compressible/README.md), and the separately generated
3D teaching edition described in [education](../education/README.md). This
research extension does not add a production compressible solver.
