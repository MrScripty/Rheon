# Comparison result qualification

Isolated branch: `implementation/comparison-result-qualification`, based on
reviewed comparison head `0de6a857adb17a6ee0312e889aff50e2cd7aa6dd`.
The checkout and repository instructions were inspected before edits. No issue
inventory exists; this repair closes the complete-success, matched-workload and
finite-diagnostic requirements already stated in `docs/COMPARISON.md`.

## Demonstrated failures and repair

The real unchanged release CLI produces each test packet; tests alter one field
or artifact after the child completes, before the harness accepts it. Baseline
metadata tests failed 16 subcases: changed/missing schema, forcing, absolute
residual, iteration and memory limits were accepted or not explicitly pinned.
The next diagnostic tests failed 49 subcases, including NaN/infinite/negative
diagnostics, step order/time/admission violations, inconsistent manifest
diagnostics/payloads and missing/truncated images. Separate summary write and
rename injections raised uncaught errors before the ownership repair. The
`*-before.log` files retain these failures rather than claiming the baseline
passed. `metadata-after.log` and `diagnostics-after.log` are intermediate checks.

The harness pins and checks the complete schema-2 workload. It qualifies every
step against existing numerical admission limits, using integrated residual
units (`residual * dt / cell_volume`), and checks final manifest agreement and
payload bounds. PNG checks cover container integrity and grayscale dimensions,
not pixel decoding or physical image accuracy. All rejection paths reap/close
the completed child, preserve diagnostics, stop further jobs and publish no
completed summary. Summary write/rename/completion-status failures clear owned
success and remove a published summary. Persistent filesystem failure while
saving failure status remains an I/O error; this is not crash-durable storage.

`final-tests.log`: all 18 Python tests pass. This includes the previous 8
lifecycle/deadline tests, new adversarial qualification/publication cases and
valid zero-source/single-cell and active-source runs for both methods at both
accuracy settings. The CI path filter now covers the new test file; its existing
test discovery runs it without installing dependencies.

## Reproduction and limits

From the repository root, after the locked release CLI is available:

```sh
python3 -m unittest discover -s tools -p 'test_*.py' -v
python3 evidence/cloud-qualification/benchmark.py target/release/rheon FRESH_OUTPUT
```

The release executable is reused, SHA256
`e10000f4fb61ba09765d7e9cbc76cac0463550cc7aecbcd72a66d458e0a1a584`.
Its compiler/profile receipt is the unchanged
`evidence/cloud-qualification/environment.txt` and `build-environment.json`:
official Rust 1.92.0, LLVM 21.1.3, default optimized Cargo release profile,
no RUSTFLAGS/profile overrides. No Rust/source/dependency change requires a new
binary. Rust tests, Clippy, native-window interaction and Lean/book builds are
not rerun or claimed as fresh evidence for this Python-only repair. Existing
49/44/55 Rust and hosted CI evidence remains unchanged on the reviewed branch.
No numerical implementation, stable ID, fixture, tolerance or book/proof artifact
is changed; no external review request or PR merge is made.

Serial standard/tight measurements and their source/hardware/contention receipt
are retained under `benchmarks/` when the follow-up qualification is complete.
No exclusive-host, cross-device, desktop timing or real-time claim is implied.

Logs have surplus terminal blank lines removed only for Git whitespace checks.
