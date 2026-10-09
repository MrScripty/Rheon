# Rheon Discrete Fluid Simulation

A research book credited Puma: mathematics, algorithm specifications, worked examples, reproducible numerical fixtures and checked Lean contracts for a low-memory 3D fluid framework.

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

The [native wall/force progression](implementation/native-wall-force-sequence.md) connects the equations, units, proof assumptions and recorded demos. The [requirements roadmap](implementation/requirements-roadmap.md) maps actual collision, force, viscosity, adhesion and density coverage and recommends shared static solid-fluid geometry as the next concrete feature. Generated site, PDF, assets and qualification receipts stay outside Git. The source-only CI hub explicitly identifies unavailable recorded bundles.

The [wall-observable bias and cancellation note](implementation/flat-wall-observable-bias-20261009.md) explains the retained flat-wall Ritz findings, the optional reconstruction from two face averages, and why a force near the manufactured reference does not establish physical accuracy. It adds research-book source only; the original refused campaign is preserved.
