# Pre-recorded measurement matrix

Frozen baseline3525c979 (PR55), candidate milestone to be committed before trials.
Same Rust1.92 locked release build, work-observation feature, CPU scheduler scoped
on each 1/2/4-worker pool without voxel observer. Source/target roots are isolated.
Baseline harness differs only by replacing unavailable warm API dispatch with old
cold step; baseline is invoked cold only. Record source and executable hashes.

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
