# Bounded PR7 verifier repair

Accepted parent: `3b71007bfcc25f5c752dd2911ae7ed3f698dbc61`, tree
`c050177168e9b99bf0246e22341ce5548ca93201`. Branch:
`repair/pr7-verifier-invariants`. Only the three affected verifiers change;
this directory adds focused source/test evidence. Numerical/CG/desktop, research,
book/proof, prior repairs, measurements, receipts and fixture bytes are preserved.

## Demonstrated findings

The seven before probes execute accepted-source verifier code retrieved directly
from Git at 3b71007, against disposable real historical checkouts/data. before.log
confirms the following; its successful probe assertions confirm gaps, not successful
qualification. Seven finding probes execute; five after-only tests are skipped.

1. [4178998720](https://github.com/MrScripty/Rheon/pull/7#discussion_r4178998720):
   the same negative first-step Courant, divergence norm, pressure iteration count,
   residual norm, tracer integral or kinetic energy in all four method repeats was
   accepted. Where needed, only temporary embedded/receipt iteration totals were
   made consistent; no original data or timing changed. Explicit lower bounds now
   precede existing upper bounds. Step/count fields are nonnegative integers.
   The fixed smoke model bounds tracer in [0,1], norms are maxima of magnitudes,
   and kinetic energy is a positive-density sum of squares. Zero magnitudes remain
   admissible. Signed velocity/acceleration values receive no unsigned rule; no
   physics scope or new diagnostic quantity is introduced.
2. [4178998726](https://github.com/MrScripty/Rheon/pull/7#discussion_r4178998726):
   empty replay inventory and empty artifact-hash maps produced success. The
   verifier now requires exactly one named demo-16/demo-plume/demo-64 entry and
   exactly opacity.png/steps.csv hash keys. Receipt order is immaterial. Absent,
   duplicate, unexpected and incomplete inventories fail. Cloud PASS output is
   deferred until the whole check succeeds.
3. [4178998729](https://github.com/MrScripty/Rheon/pull/7#discussion_r4178998729):
   staging changed Cargo.toml, then restoring its working bytes to BASE, passed
   the inventory gate. Cached/index diffs are now checked alongside existing
   working-tree diffs in both affected inventory-claiming verifiers.
4. [4178998731](https://github.com/MrScripty/Rheon/pull/7#discussion_r4178998731):
   a different committed terminal repair source passed all source-phase checks.
   The probe stops at the unavailable binary phase; it does not fabricate a full
   binary-qualified PASS. The terminal verifier now requires the documented exact
   historical checkout 8bc0f54604d98d838b3a1ab2f6fcdfff9af2ac51 before qualification.
   Its original last-source-changing commit derivation is retained only within
   that pinned checkout, preserving original source identities.
5. [4178998732](https://github.com/MrScripty/Rheon/pull/7#discussion_r4178998732):
   publication into proofs/ created a protected addition while reporting unchanged
   paths. This is a full actual result-verifier case; the terminal publisher probe
   uses its real archived receipt with qualification isolated, not a fake binary
   pass. Both affected publishers reject lexical and symlink-resolved protected
   destinations before verify(), and recheck before publication.
6. [4178998736](https://github.com/MrScripty/Rheon/pull/7#discussion_r4178998736):
   an untracked protected result-source addition qualified. Both affected
   inventory claims now reject ignored/untracked additions and preserve tracked
   checks. Ignored generated protected files require a clean qualification tree.
7. [4178998738](https://github.com/MrScripty/Rheon/pull/7#discussion_r4178998738):
   an extra NaN summary field failed serialization after creating an empty output,
   preventing retry. This is an availability defect, not false acceptance. Both
   affected writers serialize before exclusive creation, remove their own partial
   destination after a write failure, and preserve existing operator files and
   symlinks. Cleanup uses the created inode identity; cleanup errors retain the
   original error. Failing filesystems are not promised successful cleanup.

These are coherent local verifier changes, not a new framework. No finding is
intentionally unresolved. No immutable PR4/5/7 branch, main, PR3 or held PR2 retarget
is modified, and no external/bot review is requested.

## Actual validation and reproduction

```sh
RHEON_PROBE_BEFORE=1 python3 -m unittest discover -s evidence/pr7-verifier-repair -p test_invariants.py -v
python3 -m unittest discover -s evidence/pr7-verifier-repair -p test_invariants.py -v
python3 -O -m unittest discover -s evidence/pr7-verifier-repair -p test_invariants.py -v
PYTHONOPTIMIZE=1 python3 -m unittest discover -s evidence/pr7-verifier-repair -p test_invariants.py -v
python3 -m unittest discover -s evidence/cloud-qualification -p test_gates.py -v
python3 -m unittest discover -s evidence/pr5-evidence-repair -p test_integrity.py -v
python3 -m unittest discover -s tools -p 'test_*.py' -v
```

All 12 new regression methods pass in all three Python modes. They include real
staged-only/ignored/untracked Git changes, consistent-repeat negative diagnostics,
missing/duplicate/unexpected fixture/hash inventories, pinned checkout identity,
protected and symlinked output paths, nonfinite serialization, partial-write
cleanup, retry, and preservation of pre-existing files/dangling operator symlinks.
Existing 14 PR4, 13 PR5 and 20 harness regressions also pass. PR5 regressions use the
unchanged stored fresh replay and its actual producing executable; its normal
RHEON_BINARY/RHEON_FRESH_REPLAY overrides remain available after a different build.

Actual historical cloud and result CLI checks pass in normal/-O/PYTHONOPTIMIZE,
using unchanged 7e1a76dd and 41674470 source checkouts respectively, with new result
receipts in this directory. Historical terminal source/inventory/publisher phases
are tested separately: its e100 executable is unavailable, and no full fresh pass
of that fixed binary gate is claimed or weakened. No benchmark, solver, Rust,
Clippy, Lean, native-window, performance or physics-validation run is newly claimed.

## Coverage handoff

review-scope.json was fetched from actual immutable PR7 review 5407804411, run
bfce51be-d51d-444d-b7ff-8744f8f8d9f9: 33 eligible paths, 54 excluded artifacts, head
3b71007/tree c050177. The three updated verifier files invalidate prior review
coverage for those bytes; unchanged eligible paths retain their original identities.
New tests/notes/receipts also require coordinator disposition. The exact final
head/tree/path delta is provided in the external coverage handoff, for reconciliation
with the parent's coverage ledger. Prior PR7 coverage is not relabeled as a review
of this successor; independent review and integration remain pending.
