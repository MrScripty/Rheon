# Independent review record

Read-only reviewer: existing delegated /root/review, explicitly authorized by the
user's independent-review requirement. No reviewer builds/tests/benchmarks ran
concurrently with measurements.

## Design and source

Owner contract was agreed before implementation. Reviewer found no design conflict
with reusing LiquidTransportSimulation's accepted pressure and publishing provenance
only at complete-step commit. It required distinct full-liquid stage identity,
constructor-only nonwrapping incarnation allocation, unsupported-path invalidation
(including separate column-MAC commit), and cold provider/fallback behavior.

Review of the implementation found no blocking source defect. Initial numerical
qualification still uses compensated compatibility, current RHS norms and full
b-Ap including the gauge row. Metadata preparation is pure; assignment occurs with
infallible owner swaps after final cancellation. Follow-up tests cover explicit
unsupported surface/column requests and preferred-provider/fallback transitions.
Saturated fraction1 failure is retained rather than weakening its transport gate.

## Measurement tooling

Reviewer found an ordering defect in the first four-permutation schedule: candidate
cold never occupied the final slot. No acceptance was based on that schedule.
Preserve those trials and prospectively add exactly the two missing permutations
for every case; all future cases use all six. Every variant then occupies every
position twice. Original and corrected manifests remain available.

Reviewer also required failed-horizon exclusion, finite checks, memory/conservation/
first-step equivalence, exact same-policy replay checks, and accurate capture labels.
Analyzer now applies these checks. A read-only audit of66 completed initial triplets
found complete horizons, exact baseline/cold nontiming fields, exact first-step
warm/cold fields, matching managed bytes and finite metrics. That partial audit is
not final performance acceptance. Tighter localized reference comparison and full
controls remain required by the measurement plan.

Resume now rejects mismatching binary hashes and preserves incomplete attempt files
instead of overwriting them. The grid lab's capture metric scans/hashes borrowed
fields and evaluates a hydrostatic oracle; it does not copy/export/render fields.

Final source/results review is recorded below. Exact-head CI results are attached
to the draft PR after the final evidence commit.

## Gauge-conjugacy correction review

Independent reviewer confirmed original64³ localized warm lost horizon is a
blocking failure, not a speedup. Diagnostic reduced true norm agrees with the
recursive residual below threshold while the full gauge residual remains large.
Reviewer supports opt-in reduced-system residual replacement: full acceptance,
default cold recurrence and iteration budget remain unchanged. No smaller graph
failure is required: independent chain/refinement provide oracles and the actual
64³ two-step case supplies the executable regression trigger. New binary must be
qualified afresh. The initial chain fixture passes old too; do not mislabel it.

## Evidence-helper follow-up

Reviewer confirmed archive destination containment, duplicate/empty confirmation
case rejection, and saving crash output before explicit failure. No new static
blocker found. Dataset validation remains distinct from performance admission;
full screening and the single bounded confirmation cohort remain required.

Reviewer confirmed before control results: repeatable switch-workload harm fails
this recorded admission gate; opt-in documentation does not override it. Baseline
harness disclosure is accurate, but source placement alone cannot prove absence
of binary-layout effects. Actual candidate-cold controls remain authoritative.


## Final admission review

Reviewer recommends qualified admission of corrected `3569d3b` for the explicit
opt-in sealed coupled-liquid path, pending exact-head CI. Source/tests still match
the reviewed frozen candidate. The reviewer independently recomputed all six cold
confirmations, three rest comparisons and two particle confirmations, and verified
balanced ordering, exact outputs/full horizons, unique archive members and hashes
for the 720 matrix runs and 228 final confirmation runs.

No blocking source or evidence defect was found. Main gains are repeatable; final
controls do not establish repeatable harm beyond observed variability. This is not
performance equivalence. The 1000-particle control remains positive in both cohorts
(+1.83% and +2.20%; final 7/12 adverse), leaving an unresolved modest runtime-regression
risk. All positive cold medians and the adverse short pilot remain disclosed. No
further sampling was recommended or authorized.
