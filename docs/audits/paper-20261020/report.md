# Research paper audit and publication handoff

Date: 20 October 2026. Author credit preserved: **Puma**.

The existing paper has been corrected at source level. Its new Chapter 29
researches liquid–solid boundaries and 3D collision meshes; Appendix G supplies
20 claim-to-equation-to-implementation/evidence findings and the derivation of
Rheon's actual mixed-material particle pressure force.

No demonstrated causal chain from a wrong original pressure formula to the
observed solver failures was found. Grid signs/units are consistent. Actual
repairs include CG recurrence state and exact geometric face classification;
particle wall/model consistency and unscaled cut-reference conditioning are
different findings. Original failed criteria are preserved, including particle
water-rest screens, failed P1 adaptations and all six unscaled CV2 square cases.
The 24-case CV2 success and PR142 software passes are not PDE/physical acceptance.

## Repository and publication identity

- Original checkout `/workspace/Rheon`, branch `work` at `6ba41c7`, was clean and
  remains untouched by edits. No `AGENTS.md` was found. The repository's
  `docs/REPOSITORY_OUTPUTS.md` policy was followed.
- Fetched/inspected main is `718469beedb9a949f4bb6f3299c7b880b4c47e47`.
  A separate worktree `/workspace/Rheon-paper` and local documentation branch
  `docs/research-paper-audit-20261020` isolate this work.
- Final book-source commit: `750352e3f56abc156d81799d32f94096e8884689`.
  First-pass source `4a78007` and its qualification remain separately retained.
  Proposed base/diff: `718469b...docs/research-paper-audit-20261020`.
  Later handoff-report commits do not alter book or renderer inputs.
- GitHub Pages API and successful deployment `7116715012` both identify
  `https://mrscripty.github.io/Rheon/`; deployment source is `d8771ad` on
  16 October 2026. Direct endpoint content retrieval was unavailable, so no
  fresh live-byte comparison is claimed.
- Canonical paper: `docs/research-book/chapters/*.md`, `appendices/*.md`.
  Existing Pages builder: `docs/education/build.py`; publication workflow:
  `.github/workflows/research-book-pages.yml`. The separately pinned PR36
  deployment workflow was inspected and not invoked.
- Open PRs and branch heads were read before edits. Current docs/README work
  and active implementation/plan/evidence owners were preserved. PR141 admits
  the prospective MMS contract; PR143 is its preparation PR at `bc82e92`, with
  no numerical execution claimed. No merge, push, deploy or branch deletion
  occurred.

## Exact changed files

The final diff contains **16 Markdown files**: 14 book sources and two authored
audit records. The book sources are:

1. `docs/research-book/chapters/01-purpose-and-model.md`
2. `docs/research-book/chapters/04-pressure-projection.md`
3. `docs/research-book/chapters/09-liquid-surfaces.md`
4. `docs/research-book/chapters/10-forces-and-viscosity.md`
5. `docs/research-book/chapters/11-particle-methods.md`
6. `docs/research-book/chapters/17-rust-implementation.md`
7. `docs/research-book/chapters/18-validation-program.md`
8. `docs/research-book/chapters/19-forces-and-moving-boundaries.md`
9. `docs/research-book/chapters/20-collision-mesh-pipeline.md`
10. `docs/research-book/chapters/22-viscosity-and-stress.md`
11. `docs/research-book/chapters/25-capillarity-and-integration.md`
12. `docs/research-book/chapters/29-liquid-solid-interaction.md` (new)
13. `docs/research-book/appendices/C-bibliography.md`
14. `docs/research-book/appendices/G-paper-audit-and-corrections.md` (new)

This report and `independent-review.md` are two additional authored audit files.
No solver, lockfile, proof, renderer, README, active plan or protected numerical
evidence changed. Existing source discovery includes the new chapter/appendix
without a new book or workflow. Chapters 19/20/22/23/24/25 were reconciled with
the lead's handoff; the six base chapters and relevant native collision sources
are unchanged between main `718469b` and PR142. Chapter 29 now cross-links their
existing coverage. Chapter 25's stale PR45 status and renamed section link are
corrected. The supplement derives the separate pressure-work/volume-partition
condition and distinguishes absolute open-traction pressure and Larionov's
zero-total-traction disk from the prospective CV3-A polygon.

## Review, tests and artifact

The [independent review](independent-review.md) accepts the substantive
mathematics/source corrections and records exact reviewed hashes and citation
limits. Main-source admission still matches all 124 selected paths; this check
is source identity only, not new numerical qualification.

First-pass completed checks (retained at their original source identity):

- Existing reference generators complete; strict existing Pandoc/KaTeX builder
  produces 36 sections and 917 rendered mathematics expressions.
- 134 existing documentation/reference tests pass across 22 normal/optimized
  invocations: practical, completion, gauge, engineering, decay, volume, PDF
  freshness, publication gates, transport, native sequence and expansion.
- 12 companion comparator tests and four historical-baseline tests pass.
  Unchanged Lean source/pin inventory check passes; the Lean kernel was not rerun.
- Separate actual-browser review passes 20 corrected-page desktop/mobile
  inspections in sandboxed Chromium 151.0.7922.173: no page errors, failed
  requests, missing rendered math, lost Puma credit or viewport overflow.
- Existing full `verify_browser.py --render-pdf` completes in sandboxed Chromium
  with all six reference labs, teaching/transport/source-only controls and no
  page errors or HTTP failures. It generates the source-bound 211-page PDF.
- `verify_site.py` passes: 36 sections, 917 equations, 6,637 local targets,
  530 packaged Markdown links/images, 18 packaged figure paths and 397 portable
  PDF URI links with no localhost destinations. Current proof/linked-source
  hashes match their unchanged inventories.
- All 211 PDF pages contain extractable text with no replacement character.
  Thirteen raster pages were visually inspected, including all new Chapter 29
  pages 120–124, Appendix G pages 146–150 and citation/context pages. Equations,
  table continuation and Puma footer are readable; desktop/mobile screenshots
  of the new sections were also inspected.
- `git diff --check` passes. Independent chapter/appendix hashes match.
- The final `verify_browser.py --check` replay passes in the approved context;
  all six labs re-execute, with no page errors or HTTP failures. The verifier
  confirms unchanged PDF and receipt bytes plus current source/book bindings.

An initial pre-final-source browser run was deliberately interrupted and is not
counted as passing; a fresh
final-source run uses the committed book. A premature `verify_site` invocation
correctly refused absent PDF receipts and is not counted as passing. The first
read-only browser replay was also launched in the restricted namespace and
failed at Chromium startup; the successful approved-context replay is separate.

The workspace namespace presents Chromium's setuid helper with non-root
ownership, causing its sandboxed launch to abort. The same existing verifier was
approved for execution outside that namespace, **with Chromium sandbox enabled**.
No host security setting or renderer gate was changed. No action was denied by
automatic approval review in this task.

External artifacts, logs and screenshot review live in
`/workspace/rheon-paper-review/`; the final local site is
`/workspace/rheon-paper-preview-final/`. The first-pass preview is preserved.
Generated assets, PDFs, logs, simulation data
and archives stay outside Git. No native solver packet/recorded bundles were
provided to this source-only preview, and it labels their absence. This does
not substitute for native/hosted CI qualification of another solver branch.

## Final supplemental qualification at 750352e

The lead handoff was corroborated against pinned source and the primary
Larionov–Batty–Bridson supplement, then independently reviewed. All final
substantive source hashes match the supplemental review. Fresh qualification
uses `/workspace/rheon-paper-review/final/`, separate from first-pass logs:

- All 150 selected tests pass again; all 26 commands exit zero, including proof
  source identity and 124-path native admission checks.
- The existing builder renders 36 sections and **924 equations**. Full sandboxed
  browser/PDF qualification passes with zero page errors/HTTP failures and
  produces the **213-page** final reading PDF.
- All 20 corrected-page desktop/mobile inspections pass again. The added local
  fragment check verifies **452 targets with zero failures**, including the
  corrected Chapter 25 link.
- `verify_site.py` passes **6,645 local targets**, **546 packaged Markdown
  links/images**, 18 figure paths and **401 portable PDF URI links**, with no
  localhost PDF destination. Source/PDF/linked-file freshness hashes match.
- Every PDF page contains extractable text with no replacement character.
  Thirteen final raster pages were inspected, covering the revised chapter,
  partition/pressure derivations, audit tables and new primary citation.
- Final read-only browser replay passes all six labs with no page errors or
  HTTP failures, and confirms unchanged PDF/receipt bytes and final source
  bindings. No final local documentation blocker remains.

## Publication disposition

**Ready for documentation review and source-only publication packaging:** local
build, full browser/PDF generation, read-only replay, site/link checks and
independent substantive review pass. No local documentation blocker remains.
This is not complete native-packet qualification or exact-head hosted CI.
The task authorizes local source edits/tests, not live Pages deployment.
The repository lead owns final approval,
integration, remote branch/PR creation if desired, exact-head CI and publication.
Known PR134 desktop contract and PR131 VTK-installation CI diagnoses remain
separate; no paper assertion was altered to turn those failures green.

The local link check establishes packaged/local path integrity and portable PDF
URIs, not HTTP availability of every external research site. GitHub source links
to the new local commit become publicly reachable only after the lead elects to
publish that branch. The self-contained preview and Markdown ZIP contain the
local source copies; no remote write is needed to review them.

To inspect the final local site, serve `/workspace/rheon-paper-preview-final/` with a local
HTTP server. The portable reading copy is
`/workspace/rheon-paper-preview-final/downloads/Rheon-expanded-book.pdf`.
`/workspace/rheon-paper-review/research-paper-site-final.zip` packages the site;
`/workspace/rheon-paper-review/research-paper-audit.patch` is the proposed diff.
`source-manifest.json`, `artifact-receipts.json` and `handoff-manifest.json`
bind source and external artifact identities. These are local handoff artifacts,
not remote publication receipts.
