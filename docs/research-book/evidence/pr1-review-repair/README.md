# PR1 external review repair

Reviewed head: `50ff0b2218cd62e3023fa57eecd2a2411486e039`.
[External review 5404234097](https://github.com/MrScripty/Rheon/pull/1#pullrequestreview-5404234097)
processed 79 files. Repairs are isolated on `repair/pr1-external-review`, separate
from the CPU comparison branch. No PR merge, thread resolution or external review
request is made. Parent independent review and full project Lean qualification
are pending.

## Finding dispositions

| Finding | Validation and repair |
| --- | --- |
| Downloaded elan runs without integrity verification | Valid. Pin SHA256 and check with strict sha256sum between download and extraction. The actual workflow shell is exercised with offline routing stubs: correct official bytes reach extraction/installer; one-byte corruption reaches neither. Before-fix test fails. |
| Nonlinear-viscosity repeat never reaches its limit return | Valid specification defect. Require a finite nonnegative integer budget and iterate over its bounded range. Exhaustion, including zero, returns NonlinearIterationLimit without publishing a candidate. No numerical implementation or material law changes. |
| Source/pin gate uses removable asserts | Valid. Explicit ValueError checks preserve every existing gate under optimization. Changed, extra and missing sources, toolchain, direct/manifest mathlib pins and transitive revision format are rejected in normal and optimized subprocesses. Original gate fails eight optimized negative subcases. |
| Reproduction appendix omits qualified profile commands | Valid. Preserve baseline JSON, explicitly select process-local Haswell/one-thread controls, run both scripts and verify the named profile. Explain that this reproduces separate profile evidence rather than replacing historical CG observations. Commands pass in a disposable copy. |
| Checkout version annotation absent | Valid minor consistency issue. Add the existing v4.2.2 annotation; action SHA and permissions remain unchanged. |
| Audit completeness uses a count | Current loop visits every expected name, so no current theorem omission was demonstrated. Nevertheless the count does not establish membership. Track audited names and explicitly require every expected name. A core-only fixture with additional proofs demonstrates the old count guard accepting a skipped existing expected declaration. |

Only AxiomAudit.lean's inventory digest changes. Its allowed assumptions, expected
production declarations and collectAxioms traversal remain intact. All Rheon
theorem statements/proofs, Lean/mathlib/transitive pins, experiment sources,
historical JSON, reproduction profile, scientific tolerances, artwork, figures
and prior evidence remain unchanged.

## Elan provenance

SHA256 `f81c2e48c1588d4612cd2c8851947898a45ac8d72748a07dff3a5694f1cf589b`
was computed from the official HTTPS
[elan 4.1.2 Linux x86_64 release asset](https://github.com/leanprover/elan/releases/download/v4.1.2/elan-x86_64-unknown-linux-gnu.tar.gz).
Official GitHub release metadata identifies asset 258187011, 4985879 bytes, but
provides no digest or detached signature (`elan-provenance.json`). This is a
reviewable pin of trusted official-download bytes, not an upstream signed digest
or independent publisher authentication. It rejects later byte changes; changing
the version or pin requires review. No installer ran before the local checksum
check. No credentials, permissions or network settings were broadened.

## Checks and honest limits

- Source gates: 2 tests, both modes; each test also launches normal and optimized
  subprocesses. All pass after repair. `source-gates-before.log` records eight
  optimized rejection failures before repair.
- Bootstrap integrity: 2 actual-workflow-shell tests pass. `bootstrap-before.log`
  records the corrupted-archive failure before verification was added. Download,
  extraction and installer are stubbed; SHA256 is the real system command.
- Existing research comparator: all 12 tests pass under normal and optimized
  Python. Comparison tolerances and scientific experiment assertions are unchanged.
- Documented Haswell reproduction: both fixture scripts and profile comparison
  pass in a disposable copy; committed JSON is not overwritten. Scientific
  experiments use normal Python, because their assertions remain disabled by -O.
- Disposable figure/manuscript assembly passes: 18 chapters, 5 appendices,
  325 distinct native equations and 11 figures. DOCX contains 347 native equation
  nodes and the repaired loop/profile instructions. No LibreOffice renderer is
  available; no visual pagination qualification or new delivered book is claimed.
  The initial builder attempt lacked generated PNGs; the complete disposable
  pipeline generates them first. Both attempt logs are retained.
- Lean 4.19.0 official compiler is available. Exact auditor compiles against a
  Lean-core fixture and its real negative Python driver passes both modes.
  `audit_smoke.py` is reproducible; it routes lake env lean to the actual compiler
  in a temporary fixture workspace. These fixture definitions are not Rheon
  mathematical proofs. Full actual-project lake build/audit remains unqualified.

The automatic elan toolchain fetch encountered a proxy 403. The official pinned
Lean archive was downloaded directly without changing network settings. Mathlib
sources match the pinned commit, but 6641 cache downloads returned HTTP 403.
Default cache storage was outside the writable workspace; the supported
MATHLIB_CACHE_DIR option moved cache storage into /workspace, after which the
download 403 remained. No permission expansion or alternate untrusted cache was
used. The GitHub CLI read-only workflow lookup also returns Forbidden; available
connector tools cannot dispatch a new workflow. The parent must run the full
Lean workflow at the published repair head before integration.

## Reproduction

From the repository root:

```sh
python3 -m unittest discover -s proofs/scripts -p test_check_sources.py -v
python3 -O -m unittest discover -s proofs/scripts -p test_check_sources.py -v
ELAN_TEST_ARCHIVE=PATH_TO_OFFICIAL_ELAN_ARCHIVE python3 -m unittest discover -s proofs/scripts -p test_bootstrap.py -v
python3 docs/research-book/companion/test_verify_results.py
python3 -O docs/research-book/companion/test_verify_results.py
python3 docs/research-book/evidence/pr1-review-repair/audit_smoke.py PATH_TO_PINNED_LEAN_BINARY
```

With the pinned mathlib cache available, from proofs:

```sh
python3 scripts/check_sources.py
python3 -O scripts/check_sources.py
lake exe cache get
lake build
lake env lean AxiomAudit.lean
python3 scripts/test_audit.py
python3 -O scripts/test_audit.py
```

The updated workflow adds normal/optimized source-gate and audit-negative checks,
plus actual-bootstrap-shell integrity tests. Inventory digests are never
regenerated in CI. Logs remove trailing whitespace/surplus terminal blank lines
only; failed attempts and native blockers are preserved rather than relabeled.
