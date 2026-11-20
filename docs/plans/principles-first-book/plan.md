# Plan: Rheon's book, principles first

**Status:** proposed 2026-11-19, waiting for the owner's approval. Nothing here is built.

**Owner, 2026-11-19:** "I want it to be a principles first book. So it describes from the beginning the base principles that it needs in order to do what it does, and then step through in logical order, introducing new concepts as they arise. [We] have an example of such a book already … the Kenoma repository."

## The model: Kenoma's book

- **Tooling.** mdBook 0.5.4 with `mdbook-katex` (vendored KaTeX). `create-missing = false`, so a missing chapter fails the build. A `build.sh` builds the WASM labs and checks the source registry, then runs `mdbook build`. CI builds the book, runs a headless lab check, and publishes to Pages.
- **Order.** It starts from a mathematics toolkit and from Newton's second law. Each concept arrives in the chapter that first needs it; linear solvers, for example, come when deformable tissue first needs them. Parts close with "Stage" chapters that leave a working model.
- **Chapter template** (`AUTHORING.md`):
  1. The question.
  2. The idea, in small steps, each formula restated in words.
  3. A worked example small enough to check by hand.
  4. Try it: a lab whose first step reproduces the example.
  5. Check against measurement.
  6. What this model leaves out, each gap pointing to the chapter that adds it.
  7. Summary.
  8. Next.

  Further rules:
  - Terms are defined in bold where first used and added to a glossary.
  - Every symbol is given with its unit.
  - Every derivation step is shown.
  - The text is about the subject only: no tests, branches, editions or open work, except in one dated "limits" chapter.
- **Sources.** Every fact links to a registry entry that records how much of the source was read, and a claim may not go beyond that.
- **Labs.** One idea per lab, driven by the simulator's own code compiled to WASM, never a re-implementation.

## Rheon's book today

The book has 47 chapters in `docs/research-book/chapters/` and 7 appendices. Pandoc builds it into a Pages site (`docs/education/build.py`), with about 30 qualification scripts, 10 separate lab pages, a PDF, and a second workflow that re-deploys a pinned older edition to the same site.

It departs from principles first in seven ways:

1. **It opens on process.** Chapter 1 begins with the product goal, a historical commit and a dated correction, and uses residuals, fallbacks and gauges before they are defined in chapters 4–5.
2. **Concepts are used before they are introduced.**
   - Projection and splitting error appear before any grid exists.
   - The Courant number is used in chapter 6 and defined in chapter 12.
   - The stress tensor first appears in chapters 22 and 31.
3. **Topics are scattered.**
   - Boundaries: chapters 8, 19, 20, 24, 29 and 40.
   - Viscosity: 10, 22 and 31.
   - Liquid surfaces: 9, 21, 23, 25 and 45.
   - Validation: 18, 25, 28, 39 and 47.
   - The FEM part re-derives continuum mechanics on its own.
4. **Evidence and history sit inside the explanation:** "checkpoint" sections, PR numbers, receipts and theorem names.
5. **Planned work is mixed with what exists.**
   - Most of chapter 11, the multigrid of chapter 5, and chapters 23, 25 and 30–47 have no production code.
6. **Foundations are missing:** no vector-calculus toolkit, no derivation of momentum from Newton's law, no material derivative before it is used, no glossary, no chapter template.
7. **Labs stand apart from the chapters,** on separate pages, many replaying recordings or Python oracles. They are bound to exact headings, which blocks restructuring.

## The new outline

**Tagging.** Only what exists on `main` goes in the main narrative; planned topics go in Part XV.

**Chapter template.** Each chapter follows Kenoma's template, at 100–250 lines, plus a closing **"In the code"** line naming the module it explains.

| Part | Chapters (the question each answers) |
| --- | --- |
| I Mathematics toolkit | Quantities at every point · How fields change in space · Adding up flux through a surface |
| II The continuum | A fluid as a continuum (material derivative) · Mass is conserved · Newton's law for a parcel · Viscous stress · The Navier–Stokes equations · Fluid that keeps its volume · Energy in a moving fluid |
| III Space on a grid | Cutting a box into cells (MAC grid) · Discrete divergence and gradient · Why the grid is staggered · Values between samples |
| IV Stepping through time | One step in stages (splitting) · How big a step? (Courant) · How wrong is a step? |
| V Advection | Carrying a quantity along the flow · The price of stability (numerical diffusion) · Transport that conserves |
| VI Pressure projection | Making velocity divergence-free · The pressure equation · Compatibility and gauge · Solving by iteration (Jacobi, CG) · Preconditioning in parallel (SGS) · When is a solve good enough? (tolerances, warm starts) |
| VII Boundaries | Walls of a box · Slip and wall friction · Letting fluid in and out |
| VIII Forces and viscosity | Body forces and gravity · Work done by a force · Viscosity on a grid |
| IX Materials and two fluids | Material properties · Water and air on one grid |
| X Free surfaces | A surface open to air · Where the liquid is (volume fraction, level set, PLIC) · Surfaces that follow the liquid |
| XI Obstacles | Shapes as triangle meshes · An obstacle in the grid · Pressure and shear round an obstacle |
| XII Particles | Fluid as particles (SPH) · Particles meeting walls |
| XIII Compressible flow | When density changes · Fluxes between cells |
| XIV How good, how fast | Exact maths, finite arithmetic · How Rheon is checked · What a grid costs · Many cores · Writing out what was simulated · What the model leaves out (living, dated) |
| XV Research directions | Short overviews linking to the preserved research text: FEM, GPU, portals and open boundaries, thermodynamics, surface tension and wetting, multigrid, turbulence, higher-order transport |

**Appendices:**
- sources registry, in Kenoma's format;
- proved properties: the Lean results in plain words;
- glossaries;
- data and reference fixtures;
- building the book and the solver;
- algorithm reference.

**Moved out of the book into repository docs:** the paper audit, history and correction notes, `implementation/`, `engineering/`, `lagrangian/`, `evidence/`, and the receipts.

Each chapter's sources of existing text are mapped in the pull request that adds this file. The model chapter would be "Making velocity divergence-free", from chapter 04 and appendix E.

## Tooling

- **Adopt Kenoma's stack** in a new `docs/book/`: mdBook and KaTeX, `AUTHORING.md` adapted from Kenoma's, and a source registry check.
- **Labs.** Add a `<rheon-lab>` element backed by Rheon's own code compiled to WASM (`browser/solver` already holds a four-edge projection fixture to start from).
- **Pages.** A new `book.yml` replaces `research-book-pages.yml`. It builds the book, checks sources, links and math, runs a headless lab check, and deploys.
- **Retire** the pinned-edition workflow, the heading bindings in `teaching.py`, the per-lab browser qualification scripts and the DOCX path.
- **Solver checks stay** in `rust-rheon.yml` and `lean-rheon.yml`.

## Steps

| Step | Size | Work |
| --- | --- | --- |
| 1 | S | `docs/book/` skeleton, `AUTHORING.md`, `SUMMARY.md` with Parts I–XIV, local build. The old site is untouched |
| 2 | S | Write the model chapter ("Making velocity divergence-free") in the full template, for the owner to review and settle the style |
| 3 | M | Source registry and its check, converted from the bibliography, with read levels verified |
| 4 | L | Parts I–IV (foundations; rewritten 02, 03, 12) |
| 5 | L | Parts V–VIII |
| 6 | M | Lab crate, `<rheon-lab>`, the first lab and the headless lab check |
| 7 | L | Parts IX–XIV, each checked against the module it names |
| 8 | M | Part XV overviews; research text moved to `docs/research-notes/`; appendices |
| 9 | M | `book.yml`; Pages switches to the new book; the old site's build and workflows are removed |
| 10 | M, ongoing | Port the remaining labs to WASM labs driven by Rheon's code |

**Risk:** steps 4, 5 and 7 are rewriting, not moving text: the current prose is in a dense research register. Step 2 settles the style before that work starts.

## Decisions needed

1. This outline and Kenoma's tooling for Rheon.
2. Step 2 first: one model chapter for the owner to read before the rest is rewritten.
3. Whether the new book replaces the current Pages site when it reaches step 9 (recommended), or both are published for a time.
