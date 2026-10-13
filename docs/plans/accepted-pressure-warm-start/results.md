# CPU qualification results

Status: candidateec535bc rejected; corrected candidate awaiting fresh qualification.
Base is PR55 commit3525c979, not an unqualified comparison with older kernel-only
benchmarks. No device/GPU performance conclusion follows from this CPU evidence.

## Rejected first candidate

The lossless `rejected-ec535bc.jsonl` contains119 complete triplets across20cases
(357runs), manifests, receipts, all orderings and per-child RSS. Unpack with
`archive_measurements.py`, then run `analyze_matrix.py`. Analyzer reports ten
violations: five repeated acceptance mismatches plus five lost-horizon checks
for64³/w4/localized. Partial interrupted attempts are separate and excluded.

Warm64³/w4/localized repeatedly accepts onlystep0 then reaches3000iterations at
step1 (fullmaximum2.322999417e-9); both cold controls finish16steps. These costs
are failures, never speedups. The large steady gains do not override this gate.

Other original observations are provisional and cannot qualify a rejected binary:
64³/w1/steady median warm versus baseline-84.93% (six trials);16³/w4/steady default
cold median+5.35%, adverse in5/6 short pairs, requiring bounded separate confirmation.
The abrupt-force-removal pilot also shows warm can be slower. No automatic cold
retry or tolerance relaxation was introduced to conceal either outcome.

## Diagnostic decision

At iteration400 of the failing solve the reduced true residual7.885e-12 agrees
with recursive residual below3.141e-11 threshold, while the gauge/full residual
is2.3299e-9. The old full-norm restart rule discards conjugacy despite no reduced
recurrence drift. Explicit seeded initialization now uses reduced true residual
for replacement only; success still checks the original full residual/divergence.
The cold reference recurrence is unchanged. Independent review supports this
criterion; instrumented correction completes16/16. Diagnostic timings are not
qualification results. `check_gauge_regression.py` exercises the actual failure
and corrected horizon against frozen uninstrumented binaries.

The hand-assembled chain and continuum refinement tests independently verify
numerical correctness. The chain also passes the old rule and is not claimed as
a reproducer. The actual64³ case is the regression trigger.
