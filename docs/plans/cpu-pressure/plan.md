# CPU pressure optimization

Status: Verifying. Phase: draft publication and exact-head CI.
Operation: verify, following implementation authorized by the delegated CPU optimization task.
Owner: CPU optimization agent; integration owner: parent thread.
Base: main 2209012. Branch: perf/cpu-pressure-passes; target: main.
No stacking on API PR55 or validation PR46. Preserve their independent work.

## Objective and constraints

Reduce CPU solve/frame cost at identical physical horizon and accuracy. Keep
ordered compensated sums, gauge compatibility, invalid-input and cancellation
semantics, allocation accounting and public APIs. No device/GPU claim.
Write set: src/pressure.rs, src/observation.rs, tests/observation_contract.rs, focused pressure tests, this plan directory, and
an isolated benchmark example if existing tools cannot cover the selected claim.
Particle/boundary, API, UI and book files belong to concurrent owners.
No persistent-goal tool is exposed; this plan is the durable task record.

## Admission and gates

User's profile identifies ordered reductions and vector passes as dominant.
First compare arithmetic-preserving simplification of compensated accumulation
and redundant passes against unmodified main. Keep only reproducible measured
wins, with equal iterations, residuals, accepted time and output bits. Use
release builds; alternate baseline/candidate; report all samples and median,
range, phase times, full CPU step/capture times and managed bytes. Cover 16/32/64
and anisotropic grids, 1/2/4 workers, cold and sustained calls. Container has a
four-CPU quota; measurement variability is reported, not hidden in a threshold.
A consistent timing improvement beyond observed run variability admits a gain;
ambiguous results do not support a speedup claim.

Required correctness: existing analytic/manufactured pressure, divergence,
conservation/refinement, invalid-input and recovery tests; targeted bitwise
reference reduction checks if arithmetic changes. Supporting gates: format,
strict clippy and default/core/parallel tests plus exact-head CI. Independent
review before publication. No merge or deployment.

## Milestones

1. Baseline and ordered reduction experiment — Implemented; write set above; gate:
   fresh paired performance and reference correctness.
2. Other approved opportunities — Deferred after bounded investigation; investigate accepted-state warm start,
   stronger preconditioning, particle locality, worker/grain and preview overhead;
   production changes require evidence and shared-file coordination. Defer rather
   than force regressions or expand ownership without coordination.
3. Review/publication — Verifying; clean scoped diff, regression/lint/format,
   commit/push, draft PR, exact-head CI. Preserve existing book gate.

Acceptance: narrow pressure timing gain supported; overall regression acceptance pending.
Next slice: commit, publish draft and inspect exact-head CI.
Narrowed decision: fuse pressure/residual update with ordered norm only when
work observation is active and the Rayon pool has one worker. Broad fusion was
rejected after parallel-only measurements. No tolerance, sum ordering, field
bits, public setting, allocation or physical horizon changes. Reduction timing
now includes the fused vector update; compare whole frames, not that phase alone.
Work markers describe the combined pass as Reduction. Other execution modes
keep the existing separate passes. Final short matrix observed single-worker medians improved 32³ by 8.4% and
64³ by 3.1%; sustained 32³/64-step median improves 4.6%. Local regression
and lint gates passed; CI remains pending.
The earlier extracted fallback wrapper was rejected for a two-worker regression;
the original fallback loop is retained at its original call site.
Blockers: unresolved adverse 4096-particle/one-worker timing control prevents
merge readiness; disclose in draft, without attributing it to noise. Replan for accuracy/output changes, uncertain
wins, shared-file requirements or changed base. Composed-design review:
not-applicable to private arithmetic/pass simplification; no new state, boundary,
public contract or retained compatibility mechanism.

Standards: public MrScripty/coding-standards Core and Router, implementation,
verification (including oracle boundaries), proportionality, planning/discovery,
commit, documentation, performance and Rust/library profiles. Application policy
is read from public canonical Markdown; the available authoring-only skill does
not apply. No standards-engine connection exists. No repository AGENTS.md or
application skills were present. Existing code conventions remain authoritative.

See [ledger](execution-ledger.md), [issues](issues.md), and reports/.
