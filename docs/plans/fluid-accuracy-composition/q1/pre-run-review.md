# Independent pre-run review

Reviewer: existing independent `/root/p1_numerical_review`, 2026-10-20 UTC.
Read-only review; no candidate tests or numerical experiments performed before freeze.

Reviewed native periodic topology, signed donor/seam transfer and cancellation,
channel replay and discrete temporal reference, dyadic RT0 carrier, sphere
cell-average quadrature, fixed criteria/evaluator, uncertainty/refusal semantics.
Requested fixes, all verified before freeze:
- Signed seam impulse coverage on at least three cells.
- BeforeFlux cancellation precedes seam reads.
- Explicit sphere adaptive-depth exhaustion prevents positive admission.
- Every channel report/state and actual native geometry retained, not just maxima.
- Reverse fallback never exceeds nominal dt; operational time mismatch retained.

Final disposition: no remaining source/protocol/evaluator blocker to freeze and
execute ONE bounded Q1 comparator campaign. Post-run independent cellwise geometry,
field/ledger reconstruction and required regressions remain binding. No production
or Q2 accuracy admission follows. No criterion changed in response to selected
scientific execution, which has not begun.
