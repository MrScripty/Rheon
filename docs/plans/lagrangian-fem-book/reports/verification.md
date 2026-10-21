# Nonnumerical manuscript verification

Status: passed for the final corrected manuscript. This report records text/render/scope evidence only; it is not scientific qualification. No production source, admission manifest, validation threshold or prior evidence is changed.

Initial direct Pandoc MathML conversion found unsupported legacy `rm` switches; replaced those in the new chapters with `mathrm`. Link checks initially observed this report and the independent review report before they existed; both are now authored. Final checks and exact counts follow below.

## Executed checks

- Direct `pandoc 3.1.11.1` parse to JSON for authored/linked Markdown, direct HTML5/MathML conversion of new chapters and source/review documents; no warnings remain. This invokes Pandoc only, not Rheon teaching modules or numerical code.
- Local Markdown targets and referenced anchors checked from Pandoc AST; sequential chapter numbering1–36, balanced code fences and inline/display equation delimiters checked.
- `git diff --check`, explicit staged write-set and cumulative `origin/main` scope review; only the declared Markdown files changed.
- Independent mathematical/source/license review passed after disclosed corrections; see [review report](independent-review.md).

The reusable check procedure is retained at `/tmp/check-rheon-fem-text.py`; output counters and a self-contained MathML reading preview are in `/tmp/rheon-fem-review-20261021/`. Those disposable local artifacts are not a qualified full-book build, PDF edition, deployment or scientific receipt. The preview has no JavaScript solver or remote math-library dependency. Final counters/head are recorded by the completion response and local `text-checks.json`; source scope is reproducible from the research branch versus `fd7c0f0aee63bc8710df99f9bb01154e7619b974`.

## Preserved scope

No `src/`, `tests/`, Cargo, proofs, existing evidence, admission/qualification manifest, threshold or build-script edit. Earlier chapter text and fitted studies are preserved with additive cross-links only. Puma credit and original source authors remain. Existing worktree `/workspace/Rheon` is still clean on `work` at `6ba41c7`; PR145 and PR146 were read-only metadata lookups only. AppendixC remains unchanged to avoid shared bibliography ownership; FEM source inventory owns all new citations. Separate GPU research owns CUDA/Vulkan chapters; no GPU algorithm is added here.

## Unexecuted and remaining

No companion/scientific numerical experiment, Rust build/example, Lean execution, native application, GPU/hardware run, model download, full teaching build, security change, publication or merge. Current build scope was inspected: reference verification reads source/data hashes; optional packet validators may replay scientific oracles (e.g. compressible validation launches a Python numerical verification subprocess). Nothing here upgrades their historical source bindings. Parent must coordinate any later current-book qualification and source-only/full publication promise.

Direct source fetches returned HTTP403 for the exact ACM DOI, EUDML record, Oxford/UPC records and three ScienceDirect article paths listed in [source retrieval limits](../../../research-book/lagrangian/source-map.md#retrieval-limits-and-exclusions). Those actions stopped; outcome is denied/unavailable full text, with completed metadata/source research retained. No repository mutation was denied and no write has an uncertain outcome. These access limits remain a review-fidelity limitation, not evidence of full-text verification.

## M2 report-only rendering and audit checks

The new ordinary-text status report was independently reviewed for source attribution and coverage omissions. Direct Pandoc/XeLaTeX generated a seven-page A4 PDF; `pdfinfo` and extracted text confirm its structure/content, and all pages were visually inspected. Long code identifiers/URLs initially overflowed; a document-only Lua filter renders them through `nolinkurl` with xurl line breaking. The PDF includes no JavaScript, executable solver or external math renderer. Fontconfig emitted87 nonfatal unavailable-cache warnings, with exit0 and no other stderr diagnostics; no cache/security setting was changed.

Markdown/PDF, rendering log, extracted text, page previews and helper inputs are retained under `/workspace/Rheon-lagrangian-deliverables/`. No scientific modules were imported or full book builder run. The checkpoint9489aa3 is preserved as an ancestor; the follow-up uses an ordinary documentation commit. M2 acceptance is the reviewed audit/document artifact, while full research closure remains pending in A6.


## M3 minimal solver completion and readable previews

Final direct checks cover21 Markdown files,191 links,722 inline/display math nodes, chapters1–36 and13,453 words in new chapters30–36. Pandoc parse/MathML, local file/anchor checks, fences/delimiters and whitespace pass without warnings. The only executed algorithms are document tooling; no proposed FEM numerical arithmetic or solver was run.

Direct Pandoc/XeLaTeX produced a three-page completion report and an eleven-page equation-bearing solver extract. All pages were visually inspected. Initial solver code/long display math clipped; document-only fvextra/adjustbox wrapping corrected it before delivery. Final logs have only87 nonfatal Fontconfig unavailable-cache warnings per render and exit0; no cache/security setting was changed. Extracted text, render commands/helpers and page previews remain in `/workspace/Rheon-lagrangian-deliverables/`. The full-chapter MathML HTML preview remains self-contained and has no executable solver.

Supported Library batch create succeeded for the current PDFs, followed in the same tool invocation by the required identity-metadata helper, both exit0. Completion report: `libfile_0bafb1712c2081919bdf0d3d5be24660`, backing `file_000000007d8881f8af2be39d3c80edc9`. Solver extract: `libfile_05b9cb85b5c881919c681b4b715e725c`, current backing `file_000000004d9481fbb766a67db8f572bf`, version1. The M2 report's earlier Library identity remains separate. Local extracted PDF text was unchanged after the saves; both PDFs have no JavaScript. Save outcomes are confirmed, with no denial, workaround, pending or uncertain write.

Final local PDF SHA256 values: completion `2764b6d7a431367f3255e47da39d05277e12633c5c9af4a9cb3b4aa7b099ccc9`; solver extract `f42290ac656580d3b8c61e7cc9a098d20fa029105cd9ced5dc195908ee24b24f`. These identify local rendered bytes; Library IDs identify the retained artifacts. No full builder, numerical/scientific/native/hardware/machine-proof execution, push, merge or deployment.

Final cross-reference review restored the actual conditional gradient/injectivity/determinant/path proof beside its use and added a checked projection-contraction same-box proof. Independent review confirmed both and removal of the stale pointer. The current solver PDF was rerendered, all11 pages re-inspected and replaced successfully under its same Library ID with optimistic version0→1; identity helper exit0. No duplicate Library file was created.


## M4 publication revision checks

Focused publication checks pass for chapters30–36 and the public audit guide:647 unchanged TeX expressions rendered with strict pinned KaTeX0.16.22, zero CodeBlock AST nodes,26 local file/anchor links, and60 transitively linked Markdown files with no historical FEM audit reachability. Original formulas/proofs remain; excluded implementation studies are absent from chapter authorities. No normal threshold, admission file or builder/workflow changed relative to current main7516a8e.

An actual document-only Markdown bundle probe could not complete because preexisting generated figure output is absent, first observed at chapter24 `figures/expansion/couette-slip-and-power.svg`. No placeholder was created. The coordinated publisher must run the normal generated-asset/book/PDF/browser checks before publication; focused math rendering is not their substitute. Exact checks/artifacts and the scoped publication handoff are recorded in [publication review](publication-review.md). No FEM numerical/native/hardware/machine-proof execution or deployment occurred in this revision.
