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

Run `python3 docs/education/verify_browser.py` from the repository while that server is running. It uses installed Playwright and Chromium, exercises all six labs, mobile navigation/search, math and console/network checks, and renders the illustrated PDF. Rebuild once after rendering to include the PDF download. `_site` and `node_modules` are disposable generated directories; the PDF in `downloads/` is the new edition artifact.

The Pages workflow builds and validates on pull requests; deploy runs only from main. The coordinator owns integration, repository Pages source configuration (`GitHub Actions`) and any deployment protections. A prepared site is not a claim of a live public URL.

## Proof and evidence boundaries

The original theorem statements are retained. Their broad `Mathlib` imports are replaced by specific imports, so the pinned project can be rebuilt from source when the official cache is unavailable. This is a source change with new inventory and qualification; historical receipts are preserved. Thirteen new exact-real contracts live in `proofs/Rheon/Physics.lean`. The source/data-bound qualification and explicit remaining gaps are in Appendix F and `expansion/`.

The production Rust, comparison harness, dependency files, historical measurements, PR2 retarget and review-only PRs are outside this lane. Rebase/merge decisions remain with the coordinator; the branch preserves the published comparison integration ancestry.
