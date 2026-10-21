# PR59 independent default-controls analyzer review

2026-10-21. Reviewed the uncommitted `analyze_default_controls.py` against the frozen runner and already reviewed protocol. AST parsing and in-memory syntax compilation passed; the analyzer was not imported or executed, no synthetic/real numerical data was processed, and no cohort/harness/build/test/cloud/GitHub operation or binary mutation occurred. Only this review report was written.

**Decision: no blocking defect found for its stated role as a post-cohort summary analyzer.** The strict per-case statistical decision is preserved, with no construction-only exception or hidden pooling.

- Exactly two cases in the frozen `(1000,1)`, `(4096,1)` order are required. Each must contain exactly twelve pairs with the exact seed-20261013 shuffled balanced AB/BA sequence used by the confirmation runner, not merely six of each in any order. There is no run, retry, extension, filtering, resampling or outlier-removal code.
- Each row requires finite positive frame time, 64 finite nonnegative step samples and the existing accepted-time/count checks. Each pair must match accepted-time text, fingerprint, managed-byte and particle-count fields exactly. Deltas are recomputed using the runner's original full-precision arithmetic, checked against stored deltas, and retained without rounding. The median is checked against the runner's median.
- Ascending endpoints `ordered[2]` and `ordered[9]` correctly implement one-based order statistics 3 and 10. Median/minimum/maximum, all paired deltas, and negative/zero/positive counts are reported separately per case. The 96.142578125% coverage and its pointwise independence/host-drift limits are stated; no simultaneous coverage or equivalence claim is introduced.
- `median > 0` is adverse. Otherwise `upper > 0` is inconclusive. Only the remaining nonpositive-median/nonpositive-upper case meets the frozen rule. Top-level admission is `blocked` unless **both** cases meet it. There is no tolerance/epsilon, positive-median override, pooled statistic or layout/hash-based waiver.
- Input is only read. Output uses exclusive `open('x')`, so an existing output (including the input path) cannot be overwritten. Analysis and finite JSON serialization occur before output creation. Input-summary and analyzer SHA256 hashes bind the reported computation. It does not change input summaries, manifests, logs, runner or protocol.

## Limits callers must retain

This is a summary analyzer, not a standalone custody/experiment validator. It copies supplied source/binary hash strings and does not authenticate the manifest, executable bytes, per-row commands, stdout/stderr, runner hash, host/environment, accepted physical source, or whether an earlier/larger cohort existed. The exact twelve records/order checks enforce the presented summary shape; provenance and no-extension certification still require retained original runner outputs, frozen manifest and independent final review. `cohort_extended: false` correctly describes this analyzer's behavior and must not be treated as proof of unobserved collection history.

An adverse/inconclusive cohort writes a report containing `performance_admission: blocked` and prints `blocked`, while normal completion returns process exit status zero. That is acceptable for a reporting-only program; **successful script exit or output-file creation is not performance admission**. Any later gate must inspect the explicit decision, and final human/lead review remains required. Invalid input raises before report creation; a later filesystem write failure can leave a new incomplete file but cannot overwrite the source or an existing report. These limits do not weaken the frozen classification.

No actual timing result was assessed or accepted here. Original historical **+2.20%** adverse confirmation remains preserved. Final actual ordered composition, source admission, CI, fixed cohort and lead authorization remain outstanding. No protocol waiver or cohort extension is recommended.

## Exact material binding

- Analyzer (uncommitted): `676c7166e693c99d75c3a2add120c9f2ef0022e9bebadae18bba72678e69413d`.
- Frozen protocol: `c4041b4db994980d8da93329a1ed47a764d3c6324d22c54c2f59691683533732`.
- Frozen runner: `7b90ef578f96624e5b0cef4d0583ed0f304dc1f0d0e0a90f7214abbfccafeb1f`.

Protocol and runner were independently verified byte-identical to commit `ca9e640695d25fa3336b63bf28cc483ca6f11409`. This report binds the analyzer bytes above; any later code change needs delta review and refreshed binding. No denied or uncertain external action arose.
