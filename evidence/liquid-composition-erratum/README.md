# Liquid-composition PDF command erratum

This separate erratum addresses [PR 17 review comment 4187689906](https://github.com/MrScripty/Rheon/pull/17#discussion_r4187689906).
The frozen `evidence/liquid-composition/` packet, including its recorded exit
status, log, receipt, and hashes, is unchanged.

## Historical record and limitation

The `pdf-gate` entry in
[historical supporting-commands.json](https://github.com/MrScripty/Rheon/blob/9cd4587a54befa61bdfddc8e35014bd3c34f02fb/evidence/liquid-composition/supporting-commands.json)
records:

```text
python3 import docs/education/pdf_freshness.py; verify_pdf(Path("."))
```

That text is not an executable shell invocation of the PDF verifier. Its
historical `exit: 0` cannot be reproduced by running the recorded string. The
retained log says that the PDF input and byte checks passed, but does not recover
the exact shell command that produced that observation. This erratum does not
assert that the corrected command below was the historical invocation, or
retroactively validate the original exit record.

## Executable reconstruction

From the repository root of the source checkout being checked, run exactly:

```sh
python3 -B -c 'from pathlib import Path; from docs.education.pdf_freshness import verify_pdf; verify_pdf(Path(".")); print("PASS reconstructed PDF gate: input hashes and PDF bytes match receipt")'
```

This imports the retained verifier and calls `verify_pdf` with the repository
root. `-B` prevents writing Python bytecode. Python 3 and its standard library
are sufficient; no Rust build, dependencies, browser, or PDF renderer are used.
An exception yields a nonzero exit; the success message prints only after the
gate returns.

The inputs are:

- `docs/education/pdf_freshness.py`, the verifier being executed
- `docs/education/pdf-inputs.json`, the existing qualified-input receipt
- Every manuscript, figure, renderer, stylesheet, and lock-file input selected
  by `input_hashes` and enumerated in that receipt (53 inputs in both runs)
- `docs/education/downloads/Rheon-expanded-book.pdf`, the existing PDF

The gate compares the complete selected input path/hash set with the receipt,
then checks the retained PDF's SHA-256. It does not render or visually review
the PDF and does not replace either the PDF or its receipt.

## Newly observed verification

[historical verification.json](https://github.com/MrScripty/Rheon/blob/9cd4587a54befa61bdfddc8e35014bd3c34f02fb/evidence/liquid-composition-erratum/verification.json) records fresh runs on 2026-10-05 using
Python 3.12.14. Each run executed the exact command above through `/bin/sh -c`
from the root of a local materialization of its identified Git tree:

- Original qualified composition source:
  `f4af9b47223f4f6928f40f7f3b5283e6023bf620`, tree
  `a209d4b8739f3bd353c4c685ac408c70aeae184c` (materialized by `git archive`)
- Reviewed PR 17 source:
  `13b38639b9b0c64926b59eed500d1fd8b7cb6b23`, tree
  `de71240c547f654167c356e9d9864caa44dbdb6b` (materialized from the exact tree
  identified by the remote commit metadata)

Both newly observed executions exited 0 and printed the reconstructed-gate
success message. All 56 files read or executed by each gate were checked against
the specified Git tree before execution. The JSON retains the source identities,
exact command, timestamps, observed exits, stdout/stderr, and verifier/receipt/PDF
blob and SHA-256 bindings. The bound `pdf-inputs.json` supplies the full 53-input
hash list. These are new observations, not backdated historical receipts.

Two additional lightweight checks ran from the reviewed PR source root:

```sh
python3 -B docs/education/test_pdf_freshness.py
python3 -B evidence/liquid-composition/verify.py
```

All seven PDF freshness regression tests passed. The unchanged packet verifier
passed its source, preservation, hash, and recorded-result checks; it did not
rerun the historical native tests. No new Rust, solver, Lean, browser, rendering,
or broader publication qualification is claimed.
