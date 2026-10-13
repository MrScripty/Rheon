# Execution ledger

- 2026-10-13: fetched main 2209012, clean work tree; created dedicated branch.
  Read standards and current solver. Existing PCG/SGS, wavefront, buffers and
  scheduling are retained. PR46 e245bcbd fetched read-only for harness reuse.
  Built unmodified scheduling_lab release with work-observation; saved binary
  outside repository for paired comparisons before optimization.
- Ordered-compensation operand-selection rewrite rejected: three paired 32³,
  one-worker runs had no stable gain; arithmetic preserved but optimization not
  justified. Source restored; compensation-experiment.json retains all samples.
- Fused ordered update/norm preliminary work-observation-build runs improved
  median 32³ timing, but parallel-only full-frame runs are ambiguous/regressive.
  Do not accept fusion globally. Investigate active observation independently,
  where one traversal avoids duplicate diagnostic stores. Gate remains equal
  outputs and full CPU frame timing, not renamed phase totals.
- Independent review found numerical operation/gauge/failure order preserved;
  noted changed phase attribution (Reduction includes fused vector updates),
  and confirmed warm start requires simulation acceptance integration.
- Final-harness setup found shared Cargo target contamination: an archived-main
  baseline binary unexpectedly exported `PressureWorkspace::update_residual`.
  The source archive did not contain that method. That measurement set was
  stopped/discarded and supplies no acceptance evidence. Rebuild both revisions
  with separate CARGO_TARGET_DIRs; retain binary/source hashes. Original
  scheduling and parallel-only baseline binaries lacked the candidate symbol.
- Added dense particle control workload (512/4096/32768 particles) without editing
  particle/emitter/boundary code, to cover requested particle counts and worker
  scaling while preserving concurrent ownership. No particle speedup claimed.
- Isolated wrapper experiment: 64³ observed single-worker median 14.711 ->
  14.052 seconds, but two-worker median 10.449 -> 11.283 seconds regressed.
  Rejected that implementation despite its narrow win. Restore fallback loop at
  the original solve call site and extract only the fused body; repeat final
  matrix. Raw rejected evidence: observed-wrapper-experiment.json.
- Independent review of revised dispatch confirms each path updates once and
  takes one sqrt; Some(NaN) cannot fall through and update twice. Direct helper
  tests do not prove production dispatch; full-solver controls supply that claim.
- Final short-matrix 32³/two-worker timings overlapped broadly but had an adverse
  median. Add six longer 32-step paired controls before accepting the unchanged
  fallback path; do not dismiss a possible regression based on source similarity.
- 1024-step 16³ trajectory pair completed with identical every-frame fingerprints,
  pressure reports, accepted horizon and managed bytes. This single pair proves
  trajectory equivalence for that workload, not a performance confidence interval.
- The full local debug work-observation suite passed: 342 tests in 465.9 s.
  Its result is retained in debug-attempt.json. Stopped the following duplicate
  local default/core debug sequence during compilation and selected the same
  full tests in release mode locally; debug default/core/desktop remain required
  on exact-head CI. No test or assertion is removed or skipped.

- Final independent review found no source correctness blocker and supported the
  narrow 4.6% sustained pressure claim. It identified a 4096-particle/one-worker
  adverse control: all three pairs slower, 9.7% median, nonoverlapping ranges.
  Recorded as unresolved and non-attributed; no regression-free acceptance.
  Close the bounded milestone as a draft with this qualification blocker.
- Release work-observation/default/core suites and all three strict Clippy
  configurations passed. Release CLI build and Python oracle/executable suite
  passed (98 tests, one existing skip). Exact-head CI follows publication.
