# Independent M1 cancellation-contract review

Reviewed 2026-10-20 on `implementation/ui-m1-cancel-contract-20261020`,
base `e1097581d751c061eaf8c2df45d83de62911a74d`. Scope: the external supervisor,
its unit tests, ignored genuine-worker contract, expanded inert status test,
and the corresponding contract/plan/ledger changes. Only this report was authored
by the reviewer; application production source was not changed.

## Disposition

No remaining material source finding in the reviewed diff. Suitable to retain
as a **conditional, unexecuted test contract**. This is not execution admission,
demonstrated genuine cancellation, kernel-containment integration evidence, or
a guarantee against uncontrolled work under arbitrary host failures.

The contract depends on a trusted pinned test, Linux kernel facilities and
externally reserved exclusive custody of the preconfigured cgroup leaf and
dedicated bounded tmpfs throughout execution. Empty-at-preflight is not a lease.
Unavailable prerequisites must block execution without fallback or provisioning.
The runtime environment marker is not a credential or substitute for separate
execution authority.

## Source findings resolved

- `tools/test_supervise_ui_cancel.py`: the owned-process fixture now retains its
  pidfd through final signaling/reaping. It no longer signals a numeric PID after
  successful reaping, when that PID could have been recycled.
- `supervise_ui_cancel.py::teardown`: normal SIGCHLD semantics are required before
  fork; reaping establishes ownership before numeric-PID fallback, and ECHILD
  suppresses that fallback. Read/reap/TERM errors remain conservative, preserve
  unresolved status and do not skip scoped KILL attempts. No broad process-group
  or PID-pattern termination is introduced.
- `planned_limits` retains stricter inherited soft/hard limits rather than
  raising them, and the receipt records effective values. Existing absolute
  TERM/KILL/wall deadlines are reused by exceptional cleanup; no second grace
  period or post-KILL observation window is silently started.
- `write_receipt` completes partial writes and checks forward progress before
  fsync. Descriptor ownership/finalization is explicit, signal handlers set a
  stop flag, and nested finalization releases remaining descriptors/restores
  handlers even when evidence writing fails. Existing close errors block GO and
  final acceptance; later cleanup failures can revoke acceptance.

The immutable memfd copy binds executed bytes to the declared SHA256/size;
growth while copying is rejected. Exact script/source/tree/lock checks precede
GO, and the child cannot exec the test before its leaf membership and admission
receipt are confirmed. Exit zero alone cannot pass: a matching complete result
is required. The source revision remains a caller-supplied build declaration,
explicitly distinct from hermetic build attestation.

## Numerical and evidence boundaries

`setup_ui_tests::bounded_genuine_cancel_contract` is ignored and excluded from
the `ui_setup_` filter. It validates admission and compiled identity before
`Worker::start`. Its request is one Jacobi carrier track, 4³ cells, two steps,
one thread, 0.02 s dt, edge-4 preview/16 MiB history and bounded fresh recording.
No model, configured/saved execution, corpus or scientific validation is added.

If separately admitted, it would hold the existing first-accepted-step gate,
click the real egui Cancel control, release the gate, observe completion while
painting, then poll/join only after `is_finished`. Integrity verification,
two retained publications, accepted generation one and cancelled terminal status
keep recording integrity distinct from requested-run completeness. These are
prospective worker/egui claims, not native rendered interaction evidence.

The documentation correctly separates one-CPU-equivalent bandwidth from the
monitored aggregate CPU trip and inherited per-process limits; supervisor/memfd
input memory from the child's memory cap; allocated tmpfs capacity from sparse
logical lengths/cumulative writes; and retained log limits from pipe-accepted
bytes. Catchable interrupts enter cleanup. Scheduling, uninterruptible kernel
tasks, supervisor SIGKILL/host failure and lost authority prevent an unconditional
hard-wall or no-leftover guarantee. Unresolved teardown is failure requiring
preserved resources/evidence and escalation to their authorized owner.

## Verification and remaining gaps

The reviewer independently ran all **13 supervisor unit tests**, normally and
with Python `-O`, against the final source: both exited zero. They exercised
actual immutable memfd/pipe behavior and an explicitly owned non-numerical
waiting child, plus fault-injected cleanup, identity, limits and receipt checks.
Mock cgroups and volume admission are not a real cgroup/tmpfs integration pass.
Fixture evidence remains under `/tmp/rheon-ui-supervisor-tests-*`; no cleanup
was performed.

The lead reports nine focused inert UI tests and Rust check/strict Clippy/fmt
passing. The reviewed status test uses fresh egui contexts for long-path,
16 KiB unbroken and multiline Unicode/combining text at both logical sizes;
pointer focus, exact full copy, end navigation, edit refusal and Shift+Tab exit
remain headless evidence. No numerical, model, solver, evaluator, corpus or
native workload was executed by this reviewer. The genuine case is compiled
but unexecuted. Native active-state/focus/accessibility checks, genuine
cancellation, word-count/usability measurements and whole M1 acceptance remain
open. M2–M4 are not admitted by this checkpoint.

## Reviewed content identity

| File | SHA256 |
| --- | --- |
| `tools/supervise_ui_cancel.py` | `23d4f18a97076d4908c11fedcd5c378568c9d8dd8bf5211b0116f5ebadc9b201` |
| `tools/test_supervise_ui_cancel.py` | `3d853281ee13e25a95375d73e4b7fc16652da93b6c27862c536fc09659234a98` |
| `src/bin/desktop/setup_ui_tests.rs` | `c1378336b2259a3f37fbe91f68872e8f9f4a2171555d32e752935b3dc819a018` |
| `reports/m1-cancellation-contract.md` | `28cc495dcf5c332bf9c90fe1c6f783509988304aac5e3cadb4453089fb4665b8` |

A later exact-commit no-run build and external seal can bind a concrete test
binary; this review neither performs that build nor permits executing it.
