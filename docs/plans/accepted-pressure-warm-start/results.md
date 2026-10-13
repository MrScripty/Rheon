# CPU qualification results

Status: candidate `ec535bc` rejected; corrected candidate `3569d3b` admitted
for the scoped opt-in policy after independent review, pending exact-head CI. The baseline is PR 55 commit `3525c979`, not an unqualified
comparison with older kernel-only benchmarks. This is cloud CPU evidence and
supports no device or GPU performance guarantee.

## Rejected first candidate

The lossless `rejected-ec535bc.jsonl` contains 119 complete triplets across 20 cases
(357 runs), manifests, receipts, all orderings and per-child RSS. Unpack with
`archive_measurements.py` and inspect with `analyze_matrix.py --progress`. The
analyzer rejects five completed triplets for repeated acceptance mismatches and
lost horizons at 64³/w4/localized. Partial interrupted attempts are retained
separately and excluded from ratios.

The warm variant repeatedly accepts only step 0, then reaches 3000 iterations at
step 1 (full maximum residual 2.323e-9); both cold controls finish 16 steps. These
costs are failures, never speedups. Large steady gains do not override this gate.

Original observations cannot qualify a rejected binary: 64³/w1/steady had a median
warm/base change of −84.93%; 16³/w4/steady candidate cold had a median +5.35%,
adverse in five of six short pairs. The abrupt-force-removal pilot also showed
warm could be slower. No automatic cold retry or tolerance relaxation concealed
these outcomes.

## Diagnostic decision

At iteration 400 of the failing solve, reduced true residual 7.885e-12 agrees
with the recursive residual below the 3.141e-11 threshold, while the gauge/full
residual remains 2.330e-9. The old full-norm restart rule discards conjugacy despite
no reduced recurrence drift. Explicit seeded initialization now uses reduced true
residual for replacement only; success still checks the original full residual
and divergence. Cold reference recurrence and iteration budget are unchanged.

Independent review supports this distinction. The instrumented correction completes
16 of 16 steps, but diagnostic timings are not qualification results.
`check_gauge_regression.py` exercises the actual failure and corrected horizon
against frozen uninstrumented binaries. Independent hand-assembled chain and
continuum refinement tests verify numerical correctness. The chain also passes
the old rule and is not claimed as a reproducer; the actual 64³ case is the
regression trigger.

## Separate long cold-control confirmations

The original 16³/w4/steady short cohort had five of six adverse pairs, median
+5.35%. The predeclared 128-step confirmation retained twelve pairs per candidate:
original median −1.46%, range [−6.47%, +7.56%], six adverse; corrected median
−1.35%, range [−6.73%, +2.23%], four adverse. All 24 pairs retain exact cold
outputs and two-second horizons.

This supports limited non-reproduction, not equivalence or dismissal of the short
cohort. Corrected short-grid results remain separate. Independent review requested
one final bounded cohort at the original 16-step horizon for other adverse cells,
with no extension after its results.

## Completed corrected screening

Frozen candidate `3569d3b` completed the complete 40-case matrix: 240 balanced
triplets, 720 runs. Final-mode analysis reports zero violations. All 648 runs in
successful fixtures accepted their full 16-step horizons. All 72 saturated runs
rejected the first step identically under the existing fraction-bounds gate; these
are retained failures and contribute no speedup ratios. Dataset qualification does
not itself establish performance admission.

Paired median native whole-step plus inspection changes against PR 55 baseline:

- Localized Jacobi: 8.46–15.08% lower across all grids and worker counts. At 64³,
  reductions are 9.24%, 11.99%, and 9.09% for 1, 2, and 4 workers respectively.
- Hydrostatic steady Jacobi: 63.47–85.84% lower. These nearly stationary cases are
  especially favorable to accepted pressure reuse and do not represent arbitrary
  dynamic flow.
- Changing-force Jacobi: 41.28–55.42% lower over the recorded 16-step horizon.
  The slower four-step pilot and individual force-removal costs remain adverse
  evidence; no automatic retry, clipping or tolerance change hides them.
- SGS: localized medians 9.50% and 10.73% lower at 16³ and 32³. One 16³ localized
  warm trial is slower; all six trials remain in the distributions. Steady SGS
  medians are 64.23% and 75.75% lower.
- Rest is mixed. In particular, 32³ warm/base has median +2.54%, with four of six
  adverse pairs. Candidate cold also has four adverse pairs there. The frozen
  three-variant confirmation addresses this without presuming a warm-specific cause.

See [all measurement tables](measurement-tables.md) for each case, paired ranges,
allocation, initial/sustained steps, pressure and inspection phases, iterations,
managed bytes and peak RSS. Raw records retain every per-step sample. Candidate
cold outputs match baseline exactly; first warm steps match cold, same-policy
replays match, and managed array bytes are unchanged. Measured cost excludes
process/pool startup, rendering and export; pressure phase includes correction.

## Tighter-reference diagnostic and particle screening

Normal cold, normal warm and explicitly tighter cold reference trajectories all
complete 16 localized steps at 16³, 32³, 64³ and 48×24×12. Across these comparisons,
terminal maximum velocity discrepancy is 1.118e-8, pressure discrepancy 7.065e-9,
and fraction discrepancy 1.023e-8; tracer discrepancy is zero. These are descriptive
errors against the same solver at tighter tolerances, not an independent oracle or
a new post-hoc acceptance threshold. Independent hand-system, Neumann refinement,
flux/mass and hydrostatic tests supply separate numerical evidence.

Particle controls retain exact fingerprints, managed bytes, particle counts and
0.064-second accepted horizons. One-worker screening is adverse in five of six
pairs for both 1000 and 4096 particles; four-worker results are mostly favorable
but include large positive outliers. All are retained. The two one-worker cases
are included in the single bounded final confirmation cohort, without modifying
SPH implementation, worker scheduling, inputs or physical horizon.

## Fixed final confirmation cohort

All 228 predeclared runs completed. All numerical outputs and requested horizons
matched; no samples were added, removed or pooled with screening. Positive changes
mean candidate cost increased. These distributions do not establish equivalence.

| Cold case | Median change | Range | Adverse / 12 |
|---|---:|---|---:|
| 16×16×16 / w1 / localized / jacobi | +0.60% | [-18.17%, +10.48%] | 7 |
| 16×16×16 / w2 / steady / jacobi | -1.54% | [-9.08%, +8.37%] | 4 |
| 48×24×12 / w4 / localized / jacobi | +0.22% | [-4.74%, +9.25%] | 6 |
| 64×64×64 / w2 / steady / jacobi | +0.73% | [-4.26%, +8.51%] | 7 |
| 16×16×16 / w1 / switch / jacobi | +1.24% | [-6.74%, +14.49%] | 6 |
| 16×16×16 / w1 / steady / sgs | -2.54% | [-11.47%, +1.61%] | 1 |

| 32³ rest comparison | Median change | Range | Adverse / 12 |
|---|---:|---|---:|
| Cold / baseline | +0.88% | [-10.09%, +5.22%] | 7 |
| Warm / baseline | -0.76% | [-9.18%, +13.80%] | 4 |
| Warm / cold | -1.69% | [-4.17%, +11.84%] | 5 |

| One-worker particle control | Screening median | Confirmation median | Confirmation range | Adverse / 12 |
|---|---:|---:|---|---:|
| 1000 particles | +1.83% | +2.20% | [-14.24%, +9.33%] | 7 |
| 4096 particles | +6.17% | -1.05% | [-6.30%, +6.02%] | 3 |

The 1000-particle control remains mildly positive in both cohorts, with mixed
confirmation signs and a broad range. Retain this uncertainty explicitly; unchanged
SPH source and exact outputs do not prove identical runtime. Independent review recommends scoped admission while explicitly retaining this
unresolved modest runtime-regression risk. No further sampling is authorized by
this protocol.


## Scoped decision and limits

Admit the corrected implementation for explicit opt-in native sealed coupled-liquid
initialization. Repeatable localized, steady and changing-force full-horizon gains
support this decision with unchanged tolerances, physical horizons, full residual
and divergence gates, accepted-state publication, deterministic cold behavior and
array budgets. Independent review found no blocking source or evidence defect.
Final controls do not establish repeatable harm beyond observed variability, but
they do not prove performance equivalence. In particular, the positive particle
medians remain an unresolved modest runtime-regression risk.

Existing APIs remain cold by default. Previous accepted pressure can be a poor
guess after abrupt input changes; individual frames and the short switch pilot
can cost more. Unsupported routes remain outside warm-start eligibility. There is
no hidden cold retry, provider replacement, tolerance relaxation or horizon cut.
Cloud CPU measurements do not justify device/GPU or GUI-frame claims. Exact-head
CI is a publication gate; links and results belong on the draft PR for that commit.

Earlier pressure-pass fusion and initial-zero-residual shortcuts were rejected and
reverted on their separate preserved branches. This change does not force new
preconditioners, SPH layout, worker/grain or preview/export changes without measured
justification. Existing SGS, bins, adaptive batching and buffers are retained.
