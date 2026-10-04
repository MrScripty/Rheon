# Terminal publication failure repair

Successor branch `repair/comparison-publication-terminal` starts at exact reviewed
candidate `416744700a45a8e74de27ca4e25cd00b0931926c`. PR1 repair `a97f1012` is a
separate frozen source and is not changed by this repair.

The independent review found a concrete ordering defect: after completion-status
write failed, summary unlink could raise before the job entered failed state.
The API then retained completed status with result None and comparison.json on
disk, and subsequent polls returned completed. The new real-child regression
reproduces this before repair (`before.log`: two errored subcases).

The publication exception handler now sets terminal failed, clears result and
records the primary error before any cleanup or status persistence. It collects
unlink and failure-status write errors without losing the primary error. Repeated
polls stay failed and perform no more cleanup, publication or child work.
No solver, workload, tolerance, cancellation or timeout behavior changes.

`tests.log`: all 20 Python harness tests pass against the unchanged release CLI.
The combined-failure regression covers recoverable and persistent failure-status
writes, asserts failed state during cleanup, retains all errors, checks three
polls, verifies child/log ownership is released, and compares PNG/CSV/manifest
diagnostic bytes before and after failure. Existing cancellation, TERM/KILL/reap,
normal publication and single-failure tests also pass.

If unlink fails, comparison.json remains. If failure-status persistence also
fails, status.json remains stale. The API's status/error is authoritative in
those cases; this repair cannot guarantee deletion or durable failure status on
a failing filesystem. The tests explicitly assert those limits rather than
claiming successful cleanup. Other lifecycle status writes keep their existing
I/O behavior.

Reproduce from the repository root with the existing locked release binary:

```sh
python3 -m unittest discover -s tools -p 'test_*.py' -v
```

Binary SHA256 is unchanged:
`e10000f4fb61ba09765d7e9cbc76cac0463550cc7aecbcd72a66d458e0a1a584`.
Rust/proof/book/fixture sources and archived benchmark evidence are unchanged.
No Rust/Clippy/Lean or timing runs were repeated or claimed as fresh checks for
this exception-handler repair. The existing 48 benchmark packets are requalified
with the final validator without rewriting their receipts or rerunning timings.
Git whitespace checks pass. Log normalization removes only trailing whitespace
and surplus terminal blank lines.

Parent independent review of this successor candidate is required before
integration. No PR3 update, merge, external review request, permission/credential
change or network-policy adjustment is made by this repair.
