# Mathlib manifest provenance repair

Candidate branch: `repair/pr1-manifest-provenance`, based on frozen PR1 head
`a97f1012f5570b4ed938a6ccead2f591313ecf67`.
[Review finding 4176232095](https://github.com/MrScripty/Rheon/pull/1#discussion_r4176232095)
is valid: changing only the Mathlib manifest URL while keeping the exact commit
and all other fields passed the source gate in both normal and optimized Python.

The gate now requires exactly one Mathlib entry. Its URL, name, revision and
inputRev match the already-validated configured Git dependency. The reviewed
entry is Git-based, rooted at the repository (subDir=null), unscoped and directly
required (inherited=false), with the existing root lake-manifest.json and
lakefile.lean routing. Those adjacent identity fields must match with their JSON
types. This checks the direct Mathlib source contract before dependency fetching;
it does not redesign transitive dependency validation or alter any lockfile.

The new tests alter one field at a time, including the URL-only case with every
other field unchanged. They also cover missing/duplicate Mathlib entries and a
missing URL. The original gate produces 22 failed rejection subcases (before.log).

## Checks

- All 4 source-gate tests pass under normal and optimized Python. Each test also
  exercises real normal and -O subprocesses. The reviewed manifest passes both.
- The source gate itself passes in both modes.
- Both actual-workflow-shell archive integrity tests pass.
- All 12 existing comparator/rejection tests pass in both Python modes.
- Git whitespace checks pass. Logs remove only trailing whitespace/surplus
  terminal blank lines; before-repair failures are preserved.

Only check_sources.py and its test file change, plus this evidence directory.
Mathematical definitions/proofs, auditor, inventory digests, Lean toolchain,
lakefile.toml, lake-manifest.json, workflows, experiment sources, fixtures,
scientific tolerances, reproduction profile and historical book/proof evidence
remain unchanged. No dependency is downloaded or changed for this Python gate
repair. No new kernel-build, performance or cross-platform qualification is
claimed; the previously green actual Lean/numerical CI applies to a97f1012.
The existing local Mathlib cache 403 is not retried or bypassed.

## Reproduction and review handoff

From repository root:

```sh
python3 proofs/scripts/check_sources.py
python3 -O proofs/scripts/check_sources.py
python3 -m unittest discover -s proofs/scripts -p test_check_sources.py -v
python3 -O -m unittest discover -s proofs/scripts -p test_check_sources.py -v
ELAN_TEST_ARCHIVE=PATH_TO_OFFICIAL_ELAN_ARCHIVE python3 -m unittest discover -s proofs/scripts -p test_bootstrap.py -v
python3 docs/research-book/companion/test_verify_results.py
python3 -O docs/research-book/companion/test_verify_results.py
```

The existing workflow discovers these source-gate tests in both modes, so no
workflow edit is needed. Candidate independent review is pending; parent must
accept before advancing PR1 and qualifying its actual hosted gates. PR1 remains
at a97f1012. No PR update, merge, review-thread resolution, bot review request,
credential/permission expansion or network-policy change is made.
