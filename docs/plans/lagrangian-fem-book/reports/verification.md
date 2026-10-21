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
