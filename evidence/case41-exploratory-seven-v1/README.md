# Case41 one-shot exploratory diagnostic

This is a distinct, explicitly authorized resource-policy protocol for the
unchanged frozen paired numerical ELF. It makes no 67,584-byte qualification
claim. The historical 66,368-byte subtotal and complete-memory blocker remain.

The isolated ELF process has a hard 256 MiB address-space limit, 8 MiB main
and Rust test-thread stacks, 60 seconds of CPU time, and 120 seconds of wall
time. It has 8 MiB stdout and 1 MiB stderr budgets, zero core dump allowance,
and 32 open descriptors. Limits use child-only rlimits plus a parent wall/output
supervisor. RSS is measured by wait4 rusage, not advertised as a Linux RLIMIT_RSS
guarantee. Address space is a hard process limit; measured peak RSS is a separate
observation and cannot certify an additional-memory difference.

Current ELF load payload is about 3 MiB; one geometry requests 23,584 bytes,
one chart is 62,096 bytes, and the previous conservative crate/sort fixed-frame
envelopes are below 2 MiB together. The 8 MiB thread stack leaves substantial
room for unresolved library stack. The 256 MiB process allowance also covers
libc thread arenas, runtime mappings and unmeasured allocator overhead. The
host snapshot records roughly 16 GiB of cgroup capacity with several GiB free;
256 MiB is a small fraction of it. These are pragmatic isolated-process limits,
not a complete static proof of successful execution.

The older four-equation plus point/partition observation capture completed in
0.2203 seconds on its recorded executor. A 60-second CPU / 120-second wall
budget is deliberately conservative for seven unchanged fixed-fixture equations.
The existing observation bound is under 4 MiB for the roster; stdout is capped
at 8 MiB, stderr at 1 MiB. A limit hit is terminal incomplete evidence, not a
reason to rerun with larger limits.

`preflight.py` validates frozen source, original scalar/layout evidence and the
exact original six perturbations without executing the numerical ELF. `run_once.py`
creates an exclusive before marker before launch and has no retry path. It runs
only `research_public_call::case41_paired_candidate_roster`, with one test thread.
The reference roster, old runner, Newton controller, owner construction, extra
iterations, order32, integrations and production APIs are not executed.

The native baseline assertion runs before perturbations. The reader independently
checks its bits, the exact roster and perturbation graph, nominal denominators,
native stored-field rate replay, stable/direct embedding identity, frozen start
inputs, pressure-column provenance and unchanged physical thresholds. Endpoint
physical gates are observed at their original values; no missing coarse/fine
quadrature or full step qualifier is fabricated. Any failure is incomplete and
permanently ends this protocol's numerical attempt.
