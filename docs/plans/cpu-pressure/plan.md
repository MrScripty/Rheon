# CPU pressure optimization

Status: Verifying. Phase: rejected candidate, revert qualification and draft update.
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

1. Baseline and ordered reduction experiment — Rejected and reverted; write set above; gate:
   fresh paired performance and reference correctness.
2. Other approved opportunities — Deferred after bounded investigation; investigate accepted-state warm start,
   stronger preconditioning, particle locality, worker/grain and preview overhead;
   production changes require evidence and shared-file coordination. Defer rather
   than force regressions or expand ownership without coordination.
3. Review/publication — Verifying; clean scoped diff, regression/lint/format,
   commit/push, draft PR, exact-head CI. Preserve existing book gate.

Acceptance: rejected. The observed pressure gain did not satisfy representative
control gates. After 184 additional balanced pairs, the original 4096/1 slowdown
was not stable, but 4096/4 remained adverse (+7.75%, 7/8 pairs). The reviewer
recommended a production revert. Restore all src/tests changes to main2209012;
retain harnesses and all evidence. No accepted production optimization remains.
Next slice: final regression/lint/format, independently verify the revert, push
and monitor exact-head CI. Keep PR58 draft. Do not change the separately owned
book binding gate. No source-level causal claim for particle timing is established.

This completes an investigation/tooling milestone only. The broader CPU speedup
goal remains unmet; the 4.6% pressure benefit is forfeited to reject control risk.
No link-layout tuning, tolerance relaxation or selective trial exclusion is used.

Standards: public MrScripty/coding-standards Core and Router, implementation,
verification (including oracle boundaries), proportionality, planning/discovery,
commit, documentation, performance and Rust/library profiles. Application policy
is read from public canonical Markdown; the available authoring-only skill does
not apply. No standards-engine connection exists. No repository AGENTS.md or
application skills were present. Existing code conventions remain authoritative.

See [ledger](execution-ledger.md), [issues](issues.md), and reports/.
