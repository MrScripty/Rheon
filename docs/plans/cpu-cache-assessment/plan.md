# CPU state/recomputation assessment

Status: Closed assessment; candidate rejected and reverted. Operation: assess, then admit only a justified bounded
implementation. Base main 2209012; branch perf/cpu-cache-assessment. PR58 and its
rejected fusion evidence remain untouched. No persistent-goal tool is available.

## Claim and scope

Reduce equal-accuracy complete CPU-step cost by eliminating justified repeated
work or using accepted-state pressure seeds. Never repeat fused-pass tuning,
change tolerances, shorten paired horizons or relabel a kernel result as a frame
result. Preserve the deterministic cold reference and all invalid-input,
compatibility, gauge, cancellation, retry, memory-budget and publication rules.

Initial write set: this plan directory and an isolated assessment example if
needed. Production edits require a recorded admission after source/profile review.
Potential warm starts cross src/pressure.rs and src/simulation.rs transactional
ownership; simulation/liquid files overlap active PR55 and need integration
coordination. No API/provider replacement, particle/boundary or book changes.

## Bounded discovery

1. Inspect existing allocation reuse and cached coefficients; identify repeated
   work actually represented in baseline stage/profile evidence.
2. Inspect accepted-state commit paths and competing provider/boundary/column
   routes. Determine whether a seed can be committed/inactivated safely without
   reaching into another owner's unfinished acceptance contract.
3. Capture fresh baseline whole-step/phase evidence at 16³/32³ and an anisotropic
   grid plus sustained execution. If a candidate is admitted, expand to 64³,
   1/2/4 workers, cold/long runs and particle/unaffected negative controls.

## Acceptance/rejection gates

Admit production work only with a concrete repeated cost and a bounded owner-safe
mechanism. Preallocated buffers or precomputed coefficients are not new targets.
Warm-start admission must specify explicit opt-in/reference behavior, accepted
cache commit, structural operator/domain invalidation, gauge/air handling,
compatibility checks and failure/retry semantics, with accounted memory.

For an implemented candidate: alternate balanced baseline/candidate runs with
independent build roots; fixed inputs, horizon and unchanged settings. Require
independent analytic/refinement/conservation and invalid/recovery tests; exact
outputs for arithmetic-preserving changes, or justified equal-accuracy oracles
for explicit numerical variants. Accept repeatable whole-step improvement beyond
observed variability only with non-adverse representative controls. Reject or
revert a candidate with unresolved control harm, as in PR58. Preserve all trials.
Run format, strict feature lints, regression tests, independent review and
exact-head CI before describing any implementation as qualified. Publish draft
only if supported; no merge or deployment.

If neither candidate is justified, report the source/profile finding and the
concrete next dependency. A discovery-only result is not a production speedup.

Standards: public MrScripty/coding-standards 188beda1, Core/router, implementation,
planning/discovery, performance, verification/oracles, Rust, commit and docs;
previously read in this task, relevant implementation/performance revisited.
No repository AGENTS.md or applicable application skills found.

## Admitted experiment: initial zero residual

Fresh observed 32³/128-step quiescent baseline: 1.6974 s CPU frame, zero PCG
iterations, 128 pressure operator applications (0.02384 s) and 256 reductions
(0.01576 s). One operator application and one repeated ordered norm per step
are algebraically redundant after pressure.fill(+0). The profiled opportunity
is small (roughly two percent from these stages); no speedup is assumed.

Admit one bounded stateless experiment in src/pressure.rs and focused pressure
contract tests. Initially narrow to sealed/gauged boxes, retaining the existing
initial true-report path for surfaces. Reuse already qualified rhs_l2, evaluate
the same maximum/scaled-divergence check, and copy RHS to scratch before existing
qualification/preconditioning. No fusion, new cache, sum reordering or public API.
Reviewer confirmed signed-zero behavior and necessity of the scaled-divergence
finite check. Active emitter and particle workloads are negative controls.
If whole-step benefit is ambiguous or controls adverse, reject and revert; do
not tune loop layout. Warm-start ownership remains a separate dependency.


Final disposition: 48 primary and 90 control pairs numerically matched. Small primary
median improvements did not establish a beyond-variability gain; controls mixed.
Reject the initial-residual experiment and restore production/tests to baseline.
No new draft PR or full implementation CI is warranted; retain patch, harness
inputs and all measurements on this separate assessment branch. Concrete next
warm-start dependency and qualification contract are in results.md. PR58 unchanged.
