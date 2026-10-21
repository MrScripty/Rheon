# Chapter40 and plan verification

This records documentation/publication evidence only. No portal implementation, scientific qualification, machine-checked portal theorem, threshold change or expanded provider capability is accepted.

| Claim | Required observation | Current status |
| --- | --- | --- |
| D1 / source scope | New chapter/guide/audits/plan plus additive README; all 24 FEM integration paths except the announced README addition preserved; UI/GPU/runtime untouched. | Passed author scope check against f95cac0; repeat on staged diff. |
| D2 / manuscript | Pandoc AST and strict pinned KaTeX rendering; no code/pseudocode blocks; all local/transitive packaged links and references resolve. | Passed: seven authored Markdown files, 149 mathematical expressions, 42 links, zero code/raw blocks; strict KaTeX0.16.22. |
| D3 / mathematics and source | Author derivations/source hashes reviewed; separate independent reviewer reads actual manuscript and audit and dispositions findings. | Author review complete; independent reviewer pending. |
| D4 / build artifact | Normal existing reference prerequisites and source-only book/Markdown archive built with admitted immutable sources/assets. No fabricated reference data. | Passed: unchanged bounded reference pipeline and actual source-only book/archive build; final-source rebuild required after commit. |
| D5 / real publication rendering | Existing sandboxed Chromium checks, reading PDF rendering/source freshness and site verification; actual Chapter40 desktop/mobile math/links and PDF inspected. | Blocked locally: one canonical sandboxed Chromium launch aborted with SIGABRT because the SUID helper is not configured. No unsandboxed retry/security change. Normal runner required. |
| D6 / publication | Draft PR on verified base; normal remote gate outcomes and parent-owned integration/deployment. | Pending draft; no deployment claim. |

The feature acceptance A1–A8 in the plan remains entirely pending and is not satisfied by any D-row.

## Observations and limits

Main/FEM/UI metadata and 24-path source comparison are recorded in the ledger. Exact repository/third-party license and NOTICE hashes are in the new source inventories. Source-guided standards review is not a standards-engine certificate. Author analysis distinguishes exact continuum/finite algebra, inexact stored-field defects and the unproved full split timestep.

First `npm ci` attempt used the default npm cache and failed with ENOENT writing `/home/agent/.npm/_cacache`, including extraction/cleanup warnings. No source lock/version changed. A fresh task-owned asset directory and explicit writable cache are the proportionate environment repair. Further outcomes, exact checks/counts and any blockers are appended below when observed; pending rows are not passed by assertion.

The previous Library upload failure is outside this publication route: no retry, substitute upload, Library ID or finalized item is claimed. Existing historical receipts/admission manifests and all numerical acceptance thresholds are unchanged.

## Executed publication preparation

- The original locked asset attempt failed at the default cache as recorded above. A fresh asset directory with an explicit writable cache completed `npm ci` with the unchanged lockfile: three packages, install scripts disabled.
- `docs/education/generate_references.py` completed under a 60-second wall envelope. This runs existing bounded companion graph/projection, scalar transport/diffusion and local analytic/geometry generators needed for the book, not a portal solver, FEM campaign or broad propeller study. Original generators and historical comparison gates are unchanged.
- `docs/education/build.py` completed in a fresh external source-only edition. Build receipt:44 book sections (37 numbered chapters plus seven appendices),49 implementation guides,1720 rendered math expressions, historical proof status checked and current proof bytes matched to the reviewed inventory. These inventory states are not newly executed Lean checks. Optional native/obstacle/strain/compressible/WASM packets were absent as explicitly represented by the normal source-only build.
- The actual ZIP contains271 members. An independent Pandoc-AST walk checked652 relative links/images across its Markdown, including Chapter40 and the implementation plan. An HTML parser checked8102 local targets; no non-PDF broken target was found.105 links await the canonical newly rendered book PDF. No PDF, source-freshness receipt or browser pass was fabricated.
- Ten existing `test_pdf_freshness.py` cases and18 `test_publication_gates.py` cases passed. Their mocked browser fixtures are not a real-browser pass.
- The one actual sandbox-enabled Playwright/Chromium launch aborted at `setuid_sandbox_host.cc`: its SUID helper was found but not configured correctly. The process exited via SIGABRT before page qualification. No second launch, chmod/host-security edit, unsandboxed launch, full browser gate or PDF rendering occurred.
- All23 FEM integration paths other than the announced README addition remained byte-identical to main; its original FEM paragraph was preserved. Main still f95cac0 on the pre-commit fetch. No UI PR146, GPU, native code, proof, workflow, admission or numerical-threshold file is in the write set.

Disposable exact logs/manifests are under `/workspace/portal-book-review-20261021/`; the draft edition is `/workspace/portal-book-edition-draft-20261021/`. They are outside Git. Source updates to this report/audit require a fresh final edition before publication; these counts describe the observed draft build and its unchanged link/math graph.

## Required before parent-owned publication

A separate reviewer must inspect the actual chapter/proposal/source audit and disposition material mathematical/API/license findings. Normal source-specific CI must then build, run sandboxed browser/PDF/freshness/site checks, and retain a reviewable artifact. A successful draft PR is not merge approval. Parent coordinates current main/README compatibility, exact PR/head acceptance, merge and the standard main-triggered Pages deployment; its live Chapter40 URL and artifact lineage must be verified afterward. The historical PR36-reviewed artifact workflow is pinned to old content and cannot publish this chapter.
