# One-shot flat-wall campaign execution guards

This is source for a separately approved diagnostic campaign, not authorization
or evidence of a physical solve. The numerical provider and finite model remain
unchanged. The previous source/unit approval at
`53d0adf8e4e026326f499da7b8453dd7eee4d509` supplies the retained numerical
prerequisite; guard changes require new independent source review. No physical
N6/N9/N12, PR25, pressure, stepping or refinement campaign runs during guard
verification. Every process test is synthetic Python code.

## Deadline and containment

The public launcher starts a supervisor outside a fresh controller session and
process group. Its monotonic aggregate budget is **1080 seconds**, measured from
launcher entry, including argument preparation, controller creation, Git checks,
hashing, source generation, child runs, postprocessing, cleanup and receipt
writing. Work stops at 1077 seconds, leaving 3 seconds for cleanup. An outer
`ITIMER_REAL` alarm at 1080 seconds sends SIGKILL to the known controller group
and pidfds of observed descendants, then exits 124 without blocking waits or
receipt writes. A controller hang cannot disable that supervisor alarm. No
unbounded `communicate`/pipe drain/wait is used. Native and comparison children
have 180 second deadlines; each Git check has 10 seconds. Each child deadline is
also truncated by the remaining aggregate work deadline. There are no retries.
Before each spawn the controller publishes a bounded launch-deadline message to
the supervisor. The supervisor times that phase independently, including a
blocked spawn/exec wait; completion messages must arrive with a timestamp inside
the phase deadline. SIGTERM and SIGINT also trigger owned cleanup and refusal,
with the outer alarm retained throughout bounded cleanup.
The initial controller uses fork/exec without waiting for exec readiness. Its
PID and pidfd are recorded while cancellation signals are briefly masked, then
signals are restored before any readiness wait. The pidfd covers cancellation
even before the child has created its private session. A delayed controller
exec therefore remains a known cleanup target.
The raw unreaped fork PID is retained before constructing any process handle or
allocating a pidfd, so allocation/setup failures also have an owned kill/reap
fallback. Startup errors use the reserved controller log pipe. Entry refuses
preexisting alarms, blocked guard signals and nondefault SIGCHLD handling; it
does not change those conditions or silently run without its safeguards.

Linux signal delivery and scheduling are not real-time guarantees. SIGKILL may
remain pending for a task in uninterruptible kernel wait. Normal receipts report
surviving live and unreaped zombie members rather than promise universal cleanup.
The alarm path may exit without a receipt; that is a refused campaign.

All fixed controller children inherit its group. No worker calls setsid or
setpgid. Sampled descendant discovery includes session/group membership and
parent lineage; an observed escape or extra live group member refuses. pidfds
bind cleanup to observed process identities, and the supervisor/root group is
never a cleanup target. An unobserved short-lived or escaped descendant is not
claimed to be covered by sampling. This is supervision of fixed reviewed
commands, not a security sandbox for hostile executables.

The successful prescribed roster is 16 launched processes: one supervisor, one
controller, three native children, three comparison workers and eight Git
checks, with one controller child active at a time. This is a launch ledger,
not a universal OS process-creation bound. Receipts separately report sampled
member/identity counts. Optional Git fsmonitor/submodule helpers and index
writes are disabled for these read-only checks; repositories with `.gitmodules`
refuse. No global Git, security or cgroup setting changes.

## Whole-process memory and managed payload

The supervisor samples `/proc` RSS at nominal **50 ms** intervals. It sums its
own RSS and every observed controller-group/descendant RSS. The threshold is
**268435456 bytes (256 MiB)**. A sample above the threshold refuses and triggers
cleanup. Shared pages can be counted more than once, making the sum conservative
relative to deduplicated physical pages. Python interpreter, native code, stack
and allocator resident pages participate in RSS; kernel memory and nonresident
virtual address space do not.

This is a **sampled enforcement threshold, not a continuous hard RSS bound**.
Fast peaks or growth between samples may be missed; OS delays may lengthen the
sampling interval. No per-process RSS/address-space limit or campaign-specific
cgroup limit is installed. The environment's shared cgroup is not a campaign
allowance. The independent managed-payload gates remain native 10,000,000 B and
comparison 16,000,000 B, with unchanged accounting exclusions. The planned native
physical payload bound is 732,112 B, as recorded in the pre-launch source
qualification. The retained failed-run N12 record reports 719,501 B, below
that bound; this observation does not qualify physical accuracy. RSS supervision
does not turn it
into total-process-memory evidence.

## Output before writes

The producer reservations sum exactly **4194304 bytes (4 MiB)**:

| Producer | Reserved bytes |
| --- | ---: |
| Forcing file | 65536 |
| Three native artifact sets | 3 × 1048576 |
| Three comparison JSON reports | 3 ×65536 |
| Six native/comparison logs | 6 ×65536 |
| Campaign summary | 262144 |
| Controller stdout/stderr log | 65536 |
| Supervisor guard receipt | 65536 |

There is no unreserved temporary copy or refusal log. Forcing/comparison/summary
serialization checks its slot before opening or writing. `BoundedWriter` refuses
an entire chunk that would exceed its remaining slot. stdout and stderr share
bounded pipes; Git output is captured in at most 8192 bytes per check rather than
written to an extra file. Native artifact totals retain the reviewed Rust
shared `LimitedWriter` 1 MiB cap. Comparison report cap is narrowed via the
campaign's explicit 64 KiB argument; ordinary standalone comparison retains its
existing 1 MiB maximum. Python bytecode writes are disabled. A controller-local
inherited `RLIMIT_FSIZE=1 MiB` backs up regular-file writes, without changing
supervisor/global limits. This per-file backstop is not the aggregate guard.

The final directory-size check is accounting verification, not the mechanism
that prevents quota excess. The aggregate claim applies to these fixed reviewed
producers and exact source-bound binary, not arbitrary new filesystem writers.
Generated output stays outside Git, uses a new directory and never overwrites.

## Scientific limits remain unchanged

The solver consumes only fixed body-force coefficients. Reference force and
torque are consulted after native exit. Both P1 and normal-P2 schemes retain
their predicted coarse failures. Decreasing force/torque errors and adjacent
orders are observations. Physical acceptance additionally requires uncertainties
small enough to explain the improvement; source-to-field/goal/trace/spatial-error
propagation and absolute application force/torque budgets remain unspecified or
unqualified. No absolute physical tolerances are invented here. Both
`physical_qualified` and `convergence_claim` remain false after this diagnostic
roster. Guard test success changes no physical claim.
