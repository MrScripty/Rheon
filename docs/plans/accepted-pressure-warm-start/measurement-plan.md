# Pre-recorded measurement matrix

Frozen baseline3525c979 (PR55), candidate milestone to be committed before trials.
Same Rust1.92 locked release build, work-observation feature, CPU scheduler scoped
on each 1/2/4-worker pool without voxel observer. Source/target roots are isolated.
Baseline is invoked cold only. Its frozen harness predates the optional tighter
reference and terminal field-dump support, as well as using the legacy cold
dispatch. Normal tolerances and the measured loop match. The extra argument parsing,
configuration selection and optional dump are outside timed steps; performance runs
use eight arguments and never dump. Retain the exact baseline harness and diff,
plus source and executable hashes.

Pilot (not acceptance): candidate32³,1worker,4steps,steady/localized/switch/rest/
saturated,cold/warm,one run each; establish runtime and find invalid fixtures.
No tuning of solver/physical tolerances based on speed results.

If pilot permits: primary grids16³,32³,64³,[48,24,12], workers1/2/4,
steady and localized forcing,16steps at1/64s (target0.25s),4 balanced repetitions
of baseline cold/candidate cold/candidate warm. All 3 variants run per block,
order randomized with recorded fixed seed. Report step0/startup and sustained
steps1..15 separately, pressure phase and full step+capture totals, iterations,
managed array bytes and process RSS. Keep every attempt and error.

Controls: same grids,1worker,switch/rest/saturated,16steps,4 repetitions of
all3variants; switch alternates dt1/64 and1/128, reverses acceleration at half
horizon and removes it at3/4. Saturated fraction1 keeps strict transport gate;
failed horizons are failures, not speedups. SGS first16³/32³1worker steady and
localized16steps,4 repetitions if numerical gates pass. No automatic retries.
Particle controls (native warm does not touch SPH) use prior representative
particle harness with1024/4096particles and1/4workers against frozen base;
4 balanced paired repetitions, matching hashes/outcomes required.

Independent hydrostatic pressure/zero-velocity oracle for uniform forcing across
refined grids; localized forcing uses unchanged production gates plus a tighter
reference-run comparison before admission. Target accuracy thresholds are inherited
from existing PR55 provider fixtures (relative1e-8, absolute1e-12,predicteddiv1e-7,
actualdiv1e-5), not loosened. Half-filled successful fixtures permit examining
pressure behavior without hiding strict saturated-liquid failures. Same physical
horizon and accepted dt required; same-policy replay deterministic, cross-policy
bit equality not required. Candidate cold must match baseline output bits.

Admission requires repeatable full-step improvement beyond run variability,
unchanged gates and no repeatable adverse representative workload. Report all
failures and regressions. Cloud CPU observations are not GPU/device guarantees.

## Prospective protocol correction before acceptance

Independent review found that the initial four permutation choices did not put
candidate-cold last. Retain all four trials and add exactly the two missing
permutations for every primary-small case. All remaining groups use all six
permutations, each variant exactly twice in each position. This correction is
uniform across outcomes, not selective extension for significance. Original
manifest and four receipts remain; v2 manifests record six-repetition protocol.

The CSV capture_s metric is borrowed-field hashing/scanning plus hydrostatic
oracle evaluation, not field copying, export or rendering. startup_s times owner
allocation inside an already-created pool; worker-pool/process startup is excluded.
Error-row iterations=0 is a placeholder for unavailable attempt iterations, never
a claim that a failed downstream step used zero PCG iterations. Exclude failed
horizons from speed ratios; retain their costs and acceptance outcomes separately.
Particle sampling is cubic: use1000(10³),4096(16³), not nonexistent1024 cubic fill.

Particle controls use six alternating base/candidate pairs per case, each64steps
at dt0.001s, with1000/4096particles and1/4workers. Exact trajectory fingerprints,
accepted horizon and managed bytes must match. Capture here actually copies
particles into a retained vector and hashes them; it is distinct from the grid
lab's borrowed-field inspection and still excludes rendering/export.

## Prospective bounded default-cold confirmation

Before admission, independently examine the16³/w4/steady default-cold control:
the original six short pairs were adverse in5/6, median+5.35%. Run exactly12
balanced shuffled AB/BA pairs (seed20261013),128steps at unchanged dt1/64,
frozen base/candidate binaries, cold policy, steady Jacobi. Each pair compares
the same2s physical horizon. Retain every trial; separately report allocation,
step0 and sustained timing, exact numerical fingerprints and accepted horizons.
Do not pool with the original cohort or extend sampling until favorable. A
repeatable slowdown blocks admission; mixed results support only limited
non-reproduction. Run serially after the existing pipeline, without builds.

## Corrected-candidate qualification after rejected horizon

Candidateec535bc fails the64³ localized horizon and is not admissible. Retain its
completed/partial attempts in the rejected archive. A separately frozen correction
preserves seeded conjugacy on gauge-only rejection while retaining full acceptance.
Run the same six-permutation matrix from scratch in a new directory, with original
baseline; do not pool versions. Also run the predeclared128step cold confirmation
for each frozen candidate separately (12pairs each); no optional extension.
Diagnostic timings are excluded from performance claims. The hand-computed chain
is acceptance coverage, not a reproducer (old and corrected pass); retained actual
64³ localized logs reproduce old failure and corrected complete horizon.

## One final bounded short-horizon cold confirmation

Independent review of corrected small-grid screening requests a single separate
confirmation cohort after all remaining screening groups finish. Already included:
16³/w1/localized,16³/w2/steady,48x24x12/w4/localized (Jacobi). Preserve the original
16-step horizon and settings: a longer horizon alone cannot resolve short-run harm.
Freeze the complete list, including any additional independently reviewed adverse
64/control cases, before running exactly12cold pairs per case,6AB/6BA shuffled.
Use frozen baseline/corrected binaries, retain all trials, exact outputs, phase
costs and accepted horizons; no pooling with screening or postconfirmation sample
extension. Coherent repeated harm blocks admission.16³/w4/steady has mixed short
and long evidence and will not receive another cohort merely for uncertainty.


## Rest partition decided before confirmation

After the complete grid screening and before any confirmation execution, independent
review selected six cold-only cases: 16³/w1/localized Jacobi, 16³/w2/steady Jacobi,
48×24×12/w4/localized Jacobi, 64³/w2/steady Jacobi, 16³/w1/switch Jacobi, and
16³/w1/steady SGS. Each receives exactly twelve balanced cold pairs.

The 32³/w1/rest screening has mixed adverse comparisons against baseline for both
candidate policies, but only three of six warm/cold comparisons are adverse. Add
one separate twelve-triplet rest partition, each baseline/cold/warm permutation
twice, at the same 16-step horizon. This tests both concerns without attributing
shared timing variation to warm initialization. `confirm_rest.py` records all
orders, commands, costs and exact outputs. The complete cohort, including any
particle concerns, is frozen once particle screening finishes; no postconfirmation
extension or pooling is permitted. Smaller mixed screening cells remain reported.


The complete cohort is now frozen in `confirmation-plan.json` before execution.
Particle screening found adverse patterns in both one-worker cases; include exactly
12 balanced pairs each for 1000 and 4096 particles, at unchanged 64-step horizons.
The full cohort contains 228 runs. Independent review confirms this final scope.
There will be no additional samples after its outcomes. The exact original particle
screening driver remains in `particle-screening-driver.py.txt`.
