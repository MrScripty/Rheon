# Rheon Discrete Fluid Simulation

A research book credited Puma: mathematics, algorithm specifications, worked examples, reproducible numerical fixtures and checked Lean contracts for a low-memory 3D fluid framework.

The production Rust solver is not implemented by this research package. Liquid, coupled-stress, particle and GPU sections remain scoped research specifications. Exact proofs do not establish floating-point accuracy, geometry assembly, solver convergence, physical calibration or performance.

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
