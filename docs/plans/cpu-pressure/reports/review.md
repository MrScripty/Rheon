# Independent review

The delegated read-only reviewer `/root/review` inspected the pressure changes,
observer query, equivalence test and benchmark harness independently.

No correctness blocker: operand and compensated accumulation order are retained;
gauge row remains untouched but contributes its square to the norm; cancellation
and failure checks remain in their original order. The work-observation feature
already requires parallel support. Observer TLS borrow ends before probe binding.

Review conditions incorporated: narrow claim to observed single-worker execution;
include observer source in ownership; explain changed Reduction attribution;
rerun controls with final binaries; run feature-specific, independent numerical
and recovery tests, and exact-head CI. Added exceptional floating-point test cases.

Review also rejected workspace-only warm starting because solve success precedes
simulation acceptance, and operator compatibility cannot be inferred from size.

Performance acceptance is separate from source review. No claim of independent
hardware/device validation is made. See results.md and PR checks for gate status.

The final dispatch revision was reviewed again: Some(NaN) remains a completed
fused result, so it cannot cause fallback execution and a second update. The
original fallback arithmetic remains at its call site. The direct helper test's
multiworker runs do not prove dispatch, so the observer integration contract now
also runs real solves with 1/2/4 workers, in addition to full-solver measurements.

Final review approved the expanded observation contract and found no source
correctness blocker. It required explicit disclosure of the 4096-particle
single-worker slowdown as unresolved, rather than noise. This is now a draft
qualification blocker; the narrow pressure timing claim remains supported.
