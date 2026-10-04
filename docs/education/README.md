# Rheon GitHub Pages teaching edition

The complete 25-chapter book and six appendices, locally rendered equations, displayed Lean sources, six progressive interactive 3D reference laboratories, and PDF/Markdown downloads. Owner: Puma. GitHub Pages is the deployment destination; no Sites service is used.

The laboratories consume `../research-book/expansion/reference-data.json` directly. The original Python reference produces the book figures from those same records and checks them against independent analytic, dense-matrix, geometry and rational arithmetic oracles. It is not a production liquid simulator. The browser's motion is camera interaction; analytic caps do not pretend to simulate a dynamic contact line.

## Build and preview

Requires Python 3.12, Pandoc 3.1.11.1, Node 24, and the committed npm lock. Three.js 0.180.0 and KaTeX 0.16.22 are pinned, with licenses copied into the output. There are no runtime CDN requests.

```
cd docs/education
npm ci --ignore-scripts --no-audit --no-fund
python3 build.py
python3 -m http.server 8765 --bind 127.0.0.1 --directory _site
```

Install `docs/education/requirements.txt` and a Chromium browser (`python3 -m playwright install --with-deps chromium`, or use installed Chromium). Run `python3 docs/education/verify_browser.py --check` from the repository. It serves the built site on a private local port, checks retained browser source/book/PDF bindings, and exercises all six labs, mobile navigation/search, math and page/network errors. This publication mode never renders the PDF or writes qualification receipts. `--url` can select an existing preview server.

For an intentional new reading PDF, run `python3 docs/education/verify_browser.py --render-pdf`, visually inspect it, then rebuild once to include the PDF download. Only this explicit mode can render and write refreshed PDF/browser receipts. `_site` and `node_modules` are disposable generated directories; the PDF in `downloads/` is the new edition artifact.

The Pages workflow builds and validates on pull requests; deploy runs only from main. The coordinator owns integration, repository Pages source configuration (`GitHub Actions`) and any deployment protections. A prepared site is not a claim of a live public URL.

Publication also checks `pdf-inputs.json`: the retained PDF's exact hash must match, and its manuscript, figure, renderer, stylesheet and asset-lock inputs must be unchanged. Editing those inputs requires rebuilding HTML, running the browser/PDF renderer, inspecting the new PDF, and retaining its refreshed receipt before publication. `verify_browser.py` writes the receipt only after successful rendering and browser checks. Existing proof qualification remains a separate gate.

The illustrated Markdown download is `Rheon-expanded-markdown.zip`. Extract it with its `figures/` directory beside `Rheon-expanded-book.md`; all relative image paths resolve there. A separately offered Markdown text file requires those companion figures for offline illustrations. The publication check opens the actual ZIP and compares every referenced image with the built site asset.

## Proof and evidence boundaries

The original theorem statements are retained. Their broad `Mathlib` imports are replaced by specific imports, so the pinned project can be rebuilt from source when the official cache is unavailable. This is a source change with new inventory and qualification; historical receipts are preserved. Thirteen exact-real theorems live in `proofs/Rheon/Physics.lean`, and nine bounded planar/finite-strain theorems plus six definitions live in `proofs/Rheon/BoundedPhysics.lean`. Together the five modules have 42 public theorems; the integrated audit covers 60 declarations. The current proof receipt is `expansion/bounded-proof-qualification.json`, while the earlier receipts remain source-bound historical evidence. The source/data-bound qualification and explicit remaining gaps are in Appendix F and `expansion/`.

The production Rust, comparison harness, dependency files, historical measurements, PR2 retarget and review-only PRs are outside this lane. Rebase/merge decisions remain with the coordinator; the branch preserves the published comparison integration ancestry.
