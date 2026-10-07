# Source and generated output policy

Rheon tracks source, mathematical proofs, authored configuration/locks, licenses,
compact audit manifests and curated documentation illustrations.
Simulation frames, CSV fields, qualification logs, replay copies, renders, archives
and PDFs are build/test artifacts. They do not belong in commits.

Use `.generated/` for local test exports, `rheon-demo/` for the CLI default, and
`rheon-comparisons/` for comparisons. Pass a fresh output directory to Rust examples.
CI uses disposable runner storage. Legacy evidence generators retain their
interfaces; output extensions in evidence trees are ignored. Small source-bound
manifests remain tracked inputs. New authored inputs/configs should live outside
output trees, rather than requiring `git add -f` for generated data.

The Python verifier tests regenerate needed fixtures by executing the actual Rust
examples via `tools/generated_fixtures.py`. Example selection uses Cargo's reported
executable. Generated fixture directories are source-bound, ignored and reused by
adversarial tests; failures retain partial output for diagnosis instead of rerunning
or deleting unrelated files. No handwritten fixture replaces a native simulation.
To regenerate one suite explicitly:

```
python3 tools/generated_fixtures.py free_surface
python3 tools/verify_free_surface.py
```

The seven existing suites include refinement/long-interval cases. Full fresh test
regeneration may take several minutes and requires the pinned Rust toolchain,
Pillow and NumPy. The root source-only Rust contracts do not require those exports.
Historical evidence scripts outside current CI may require their original replay
inputs; use the documented source commit from the audit manifest or reproduce the
original command in ignored storage. A historical receipt is not a new test result.

`docs/generated-output-audit.json` records the removal base, original paths, byte
counts and SHA256 identities, grouped by equal content to avoid copying outputs.
Git history is unchanged. This inventory is not a numerical acceptance baseline.
No saved simulation fields, numerical experiment results or CG histories remain
tracked. Educational scenarios and independent analytic/dense/rational checks live
in `expansion/reference.py` and `expansion/test_reference.py`. Run the reference
generator before building Pages; its lab JSON, numerical receipt and PNGs are
ignored artifacts, copied into the generated site for deployment.

The companion experiments write ignored `results.json` and `depth-results.json`.
Their strict historical regression gate retrieves three immutable, hash-locked
files into `.generated/research-baselines/` using `companion/historical_baselines.py`.
It prefers existing Git objects; shallow checkouts download only those small files
from the exact archived public commit. `companion/baseline-sources.json` contains
only repository/commit/path identities, byte counts and hashes. Expected results
never come from the current experiment. Source/runtime gates, numerical tolerances
and adversarial comparisons are unchanged; unavailable/corrupt evidence fails.

Pages builds its reading PDF and source/PDF/browser freshness receipts afresh in
ignored `docs/education/downloads/` and receipt paths: build HTML, explicitly render
and qualify the PDF, rebuild HTML to include it, then run publication verification.
A source change still rejects stale receipts; cleanup does not bypass those gates.
See `docs/education/README.md`. PDFs are CI artifacts/separate downloads and must
not be committed. Full Pages verification needs Pandoc, locked npm assets and
Chromium. Lean proof sources/pins and their small qualification manifests remain;
rerunning kernel qualification requires the pinned mathematical-library cache.

Committed raster illustrations use JPEG quality 85; lossless simulation PNGs remain
valid generated test outputs because their pixel contracts are independently tested.
Curated SVG diagrams stay vector. Original cover PNG hashes are retained with the
JPEG display hashes in `docs/research-book/artwork/sha256.json`.

This cleanup reduces a fresh checkout, not historical Git object storage. There is
no force push, history rewrite, removal of personal/untracked files, or solver change.
