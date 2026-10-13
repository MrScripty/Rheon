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

Final source/results review and exact-head CI are pending until the complete matrix
and final evidence are recorded.

## Gauge-conjugacy correction review

Independent reviewer confirmed original64³ localized warm lost horizon is a
blocking failure, not a speedup. Diagnostic reduced true norm agrees with the
recursive residual below threshold while the full gauge residual remains large.
Reviewer supports opt-in reduced-system residual replacement: full acceptance,
default cold recurrence and iteration budget remain unchanged. No smaller graph
failure is required: independent chain/refinement provide oracles and the actual
64³ two-step case supplies the executable regression trigger. New binary must be
qualified afresh. The initial chain fixture passes old too; do not mislabel it.
