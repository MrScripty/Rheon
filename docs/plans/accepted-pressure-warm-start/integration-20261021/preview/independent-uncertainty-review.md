# PR59 independent pre-timing uncertainty review

2026-10-21. Read-only review of the appended protocol and existing particle-control runner. No build, repository test, timing, cloud/GitHub action or source/binary mutation performed. No actual timing data was assessed.

**Decision: the proposed indices, coverage arithmetic and strict classification are correct. No statistical-rule blocker found.** It respects the supplied lead decision retaining positive/inconclusive blocking and rejecting construction-only substitution. No waiver or cohort extension is recommended.

## Derivation and implementation precision

For twelve independent identically distributed deltas from a continuous distribution with fixed population median m, B = number of deltas below m is Binomial(12, 1/2). With one-based ascending order statistics, m lies in `[delta_(3), delta_(10)]` exactly when 3 ≤ B ≤ 9. Each excluded tail is `(C(12,0)+C(12,1)+C(12,2))/2^12 = (1+12+66)/4096 = 79/4096`. Thus coverage is `1−158/4096 = 1969/2048 = 0.96142578125`, or **96.142578125%**, as written. Python sorted-list endpoints are **indices 2 and 9**, not 3 and 10. The ordinary even-sample median is the mean of the sixth and seventh ordered deltas, consistent with the runner's `statistics.median`.

Use the twelve stored full-precision finite `delta_pct = 100*(candidate_frame_s/base_frame_s−1)` values for each case. Apply endpoints, median and sign comparisons before presentation rounding. Report all values, min/max and negative/zero/positive counts. Missing/invalid pairs or unequal accepted horizon/fingerprint/managed bytes fail qualification rather than silently shrinking the sample. The runner's confirmation mode fixes twelve balanced shuffled pairs, 1000/4096 particles, one worker, 64 steps at dt 0.001, retains stdout/stderr, and computes the paired deltas/median separately per case. No change to that runner is necessary to define this additional summary.

## Decision rule

- Positive sample median: adverse and blocking, regardless of other summaries.
- Nonpositive sample median with upper endpoint > 0: inconclusive and blocking.
- Nonpositive sample median with upper endpoint ≤ 0: meets the stated strict non-adverse control rule for that case.

Both cases must meet the rule independently; no pooling or favorable-case compensation. Upper endpoint ≤ 0 entails at least ten nonpositive deltas, so the extra median condition is logically redundant for valid sorted inputs but explicit and consistent. An upper endpoint exactly zero qualifies under the stated non-adverse boundary; it does not establish strict improvement or equivalence. Construction/codegen evidence cannot override an adverse or inconclusive cohort.

## Assumptions and scope

The 96.142578125% value is **pointwise per-case**, conditional on independent deltas sampling a common representative distribution. Balanced randomized AB/BA order reduces some order confounding but does not prove independence/stationarity under host-load drift or autocorrelation. State those limitations with results; do not describe the number as an unconditional empirical guarantee or as simultaneous 96.14% coverage for both cases. Ties/quantized observations can make the continuous-distribution equality inapplicable; for a fixed median and independent sampling the closed interval is generally conservative, but host dependence remains a separate concern. This does not create an equivalence margin or authorize an alternative acceptance rule.

The clarification is prospective and consistent with the original requirement for paired uncertainty. The twelve-pair stopping point, no extension, no outlier removal, no optional stopping, no percent threshold, historical +2.20% adverse evidence and final actual-main composition requirements remain unchanged. No new-main/default-universal performance claim is admitted by this review. Lead-comment contents were supplied by the parent and reflected in the local protocol; no GitHub retrieval was performed.

Exact reviewed material:

- Protocol SHA256: `c4041b4db994980d8da93329a1ed47a764d3c6324d22c54c2f59691683533732`.
- Particle-control runner SHA256: `7b90ef578f96624e5b0cef4d0583ed0f304dc1f0d0e0a90f7214abbfccafeb1f`.

All requested review work completed. No denied action arose; actual timing, admission and final integration remain pending.
