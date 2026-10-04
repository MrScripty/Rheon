# PR1 CG evidence reproduction diagnosis and bounded repair

Investigated source: `1588be9e9f31a9baee11748aca38d150dfd5fe9c` on draft
[PR1](https://github.com/MrScripty/Rheon/pull/1). Failed
[run 37151494668](https://github.com/MrScripty/Rheon/actions/runs/37151494668),
job `111286150095`, checked merge commit
`b547f9a09470cfe202eb1bba77a7eeafcc9190ab`. Both trees are exactly
`5b431254414768cc0f41ac369afb0823e7483529`, so merge content does not explain
the failure. A retained CI excerpt and review-state receipt identify the evidence.

## Cause isolation

The original programs, seed, matrix and RHS were unchanged. On the cloud Xeon
8370C, NumPy 2.3.5 / SciPy 1.17.0 load OpenBLAS 0.3.30. Selecting only a different
OpenBLAS CPU kernel produces these zero-based CG-history element 53 values:

| Kernel | Value | Entries outside existing tolerance |
| --- | --- | --- |
| SkylakeX | 0.028793851420700668 | 0 |
| Haswell | 0.02879384969967465 | 18 |
| Nehalem | 0.028793850442315784 | 17 |

Haswell exactly reproduces the reported CI value and printed final projection
diagnostics. The absolute element-53 discrepancy is 1.7210260161437674e-9;
the maximum Haswell history discrepancy is 3.428668496373614e-5 at element 62.
Raising tolerance just enough for the first failure would miss the larger issue.
Every kernel gives the same output with one and four threads in these runs.
Fresh independent Haswell replay also repeats exactly. No stochastic variation
was observed in these repetitions. This is backend-sensitive finite-precision
CG trajectory behavior, not demonstrated corruption of the historical fixture.

All six runs have byte-identical input velocity, RHS and reduced matrix storage
(digests in each `oracle.json`), all converge in 78 iterations, and all
non-CG-history fields in both JSON files pass the original 1e-8 relative / 1e-12
absolute comparator. SciPy CG uses NumPy dot products and norms in its recurrence;
backend-dependent reduction rounding can propagate through that recurrence.
CPU kernel selection is an explicit OpenBLAS facility, documented in
[OpenBLAS usage](https://github.com/OpenMathLib/OpenBLAS/blob/develop/USAGE.md)
and [NumPy troubleshooting](https://numpy.org/doc/stable/user/troubleshooting-importerror.html).

The final reduced pressure differs by at most 3.126388037344441e-13 from the
Haswell result across the tested kernels. Against a separate sparse direct
factorization of the same reduced system, the largest pressure infinity-norm
error is 7.390639211735106e-10 and relative L2 error is about 3.459e-12. Final
reduced relative residuals are below 8.902e-12 and corrected divergence L2 is
about 4.443e-10, below the unchanged experiment assertion of 1e-7. These are
same-matrix numerical checks, not an independent proof of geometric assembly.

The old CI log does not expose actual BLAS dispatch, so Haswell is a reproduced
causal explanation, not a directly observed backend identity for that runner.
The local Python is 3.12.14; that CI used 3.12.3. No general claim of
cross-platform bitwise reproducibility follows from these measurements.

## Repair boundary

No historical output, experiment implementation, proof, book figure or solver
identity is rewritten. No numerical tolerance or fixture acceptance assertion
is loosened. The default comparator remains unchanged.

The optional `haswell-openblas-0.3.30` profile records a **separate** reviewed
CG trajectory for a named execution backend. It replaces only the expected
`projection.histories.cg` in memory during explicit profile comparison. The
original iteration count and every other field still use the historical
baseline. Both original JSON files and both experiment programs are hash-locked
before applying the profile. A source/fixture change fails closed and requires
deliberate profile review and requalification. There is no profile fallback,
automatic baseline update, or alternate solver under an old implementation ID.

CI sets Haswell dispatch and one thread locally for this research job, logs the
loaded numerical libraries, and explicitly chooses the profile. The verifier
requires Linux x86_64 / Python 3.12, the pinned NumPy and SciPy versions, and both
loaded OpenBLAS libraries at version 0.3.30, Haswell, one thread. The new pinned
threadpoolctl dependency inspects those facts. This supports the specified
AVX2-capable x86_64 reproduction environment; it is not a portable reference
arithmetic implementation or a requirement imposed on the Rust solver.

## Reproduction and tests

`diagnose.py FRESH_OUTPUT` runs the unchanged fixture scripts under three CPU
kernels and two thread counts, and retains outputs plus a separate direct-solve
check. Run that diagnostic only on hardware supporting all selected kernels,
including SkylakeX AVX-512. The ordinary CI reproduction needs only Haswell:

```sh
export OPENBLAS_CORETYPE=Haswell OPENBLAS_NUM_THREADS=1
python3 docs/research-book/companion/test_verify_results.py
# Run copies of experiments.py and depth_experiments.py in a fresh companion
# directory to keep the checked-out historical fixtures intact, then:
python3 docs/research-book/companion/verify_results.py \
  --baseline-dir docs/research-book/companion --actual-dir FRESH_COMPANION \
  --cg-profile haswell-openblas-0.3.30
```

`tests.log` records 12 passing tests. New tests reject altered original fixture
bytes, changed experiment sources, changed profile iteration counts, changed CG
entries, changed unrelated numerical fields and unqualified BLAS runtimes.
`negative-checks.log` retains real CLI failures for the original comparator on
Haswell, the explicit profile on the old SkylakeX trajectory, a wrong kernel,
and a wrong thread count. `repaired-comparison.log` records full replay success.
`runs/` retains all six diagnoses, not only the passing backend. Original source
and fixture hashes are in `receipt.json` and the profile. Test-log terminal blank
lines and trailing whitespace are normalized for Git checks.

`isolated-*` logs record a second qualification in a fresh virtual environment
installed from the exact requirements, including the loaded library identities.
`workflow-replay.log` executes the workflow order in a disposable copy: preserve
historical files, run 12 comparator tests against those committed fixtures,
regenerate both outputs, then compare using the explicit profile. Every command
exits zero. Tests deliberately run before regeneration because their fixture
integrity checks require the historical files, not the newly generated ones.
`verify_diagnosis.py` independently checks the retained six-run evidence; its
output is in `diagnosis-verification.log`.

## Review disposition and limits

All three PR1 threads were unresolved when inspected. At exact `1588be9`:

- The workflow already preserves committed evidence and invokes its comparator;
  the still-active thread needs this backend qualification to make the gate usable.
- The `next_rz` breakdown check is already corrected, including the restart check;
  its thread is outdated. No additional pseudocode edit is needed.
- Every manuscript builder text read/write explicitly uses UTF-8; that thread is
  outdated. No additional builder edit is needed.

No review thread is resolved and no PR head is updated by this repair. The PR1
description still identifies the older `4deee396` source, so the parent should
update its qualification description when integrating a reviewed repair.
Hosted exact-head CI, independent scientific review of the new profile, book
rendering, and Lean requalification are not claimed here. Existing proof and
published book artifacts remain untouched. The profile adds a narrowly qualified
execution identity; it does not prove convergence or physical accuracy in general.
