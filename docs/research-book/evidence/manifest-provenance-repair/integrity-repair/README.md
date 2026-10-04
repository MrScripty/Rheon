# Archived evidence helper integrity repair

Successor candidate `repair/pr1-evidence-integrity` is based on accepted PR1
head `d72db7c1bd3fc10b27978c6dcc8ec10709195ace`. Functional source is
`e9cacf9e6cc3bb29b35a248df4f10fc1604595cb`, tree
`a480e84882e60b8eb08f73aef07e7f202f7922e8`.

The two evidence-helper findings in
[review 5404586267](https://github.com/MrScripty/Rheon/pull/1#pullrequestreview-5404586267)
are reproduced in `before.log`. Under normal and optimized Python the original
helper accepted untracked and ignored proof additions. After a later committed
source change made the actual source gate fail, it still accepted the old logs
and rewrote the original receipt with the later source identity.

The helper now rejects all untracked files under protected paths, including
ignored files, before reporting preservation. It checks the original logs and
receipt byte-for-byte against fixed archive commit d72db7c1, and checks the
receipt's source hashes against the exact original repair commit d4147b31.
It never writes a receipt and explicitly reports only an archived log/source
association. It does not run tests or qualify the current source. This narrow
scope is deliberate: later source qualification belongs in separate evidence.
The diagnostic nitpick is also addressed: manifest mutation tests require the
intended rejection message, including missing and duplicate entries.

`receipt.json` here binds **fresh** successor test logs to e9cacf9 and exact
source hashes. Python 3.12.14 was used with PYTHONDONTWRITEBYTECODE=1. Results:

- Source gate passes directly in normal and optimized Python; 4 gate tests pass
  per mode, including real normal/-O rejection subprocesses.
- 6 helper tests pass per mode. They cover unchanged historical receipt,
  untracked Lean files, ignored log/PNG additions, tracked proof changes,
  altered historical logs/receipts, and a later failing gate that is explicitly
  outside the archived helper's qualification claim. Each test checks actual
  normal/-O helper subprocesses and that the helper leaves the receipt intact.
- 12 comparator tests pass per mode. Both bootstrap tests pass against the
  existing official elan archive. No new network fetch or dependency change.
- Normal/-O archive verification passes; Git whitespace checks pass.

Commands are recorded beside each log's SHA-256 in this receipt. To reproduce
the new helper suite from repository root:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 docs/research-book/evidence/manifest-provenance-repair/test_verify.py -v
PYTHONDONTWRITEBYTECODE=1 python3 -O docs/research-book/evidence/manifest-provenance-repair/test_verify.py -v
PYTHONDONTWRITEBYTECODE=1 python3 docs/research-book/evidence/manifest-provenance-repair/verify.py
PYTHONDONTWRITEBYTECODE=1 python3 -O docs/research-book/evidence/manifest-provenance-repair/verify.py
```

The Git-clone fixtures are disposable local clones with no network access.
Logs normalize only trailing whitespace and terminal blank lines. The original
receipt, original logs, production Mathlib gate, proofs, dependencies, inventory,
auditor, workflows, experiments and book content are preserved. No Rust or
numerical implementation changes or performance measurements are involved.
No new native Lean qualification is claimed. Hosted CI reported green for d72
does not qualify this successor, and the known local Mathlib cache 403 was not
retried or bypassed. No rendered-book or new platform qualification is claimed.

Independent parent review remains pending. Only the isolated candidate branch
is published; PR1 is not advanced. No merge, external review request, thread
resolution, credentials/permissions/network expansion, or paid service use.
