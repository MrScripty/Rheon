# Particle regression follow-up

## Protocol and provenance

Frozen binaries are the original final baseline/candidate pair in artifact-hashes.txt.
Candidate source: d03ac6e30de5a52839878b1da53905e9009966b8; baseline: 2209012.
No rebuild, tolerance, particle input, capture or horizon changes between pair members.
No concurrent builds/tests during timing. Deterministic shuffled balanced AB/BA order
uses seeds 581/582. CPU affinity is explicit in every command; unbound means the
same allowed CPUs 0–4 as the original run. All samples, including adverse trials,
are retained in particle-causality.json and particle-controls.json.

First campaign: 20 pairs each of 5/50 steps, 4096 particles, one worker pinned
to CPU0. Second: 20 pairs each of unbound short/long, CPU2 long, and identical
baseline-binary long controls; eight 20-step pairs for other particle/worker cases.
Each pair must match every-frame state fingerprint, particle count, accepted time
and managed memory. All 184 pairs passed these checks, including same-binary controls.

## Results

Times are run medians in seconds. Positive delta is slower candidate. Same-binary
labels both execute the baseline and are a timing negative control, not a candidate
comparison. No trials were discarded.

| Case | Pairs | Baseline | Candidate | Delta | Slower pairs |
|---|---:|---:|---:|---:|---:|
| cpu0-5steps | 20 | 0.082636 | 0.083578 | +1.14% | 10 |
| cpu0-50steps | 20 | 0.845205 | 0.831107 | -1.67% | 8 |
| unbound-short | 20 | 0.086060 | 0.085213 | -0.98% | 10 |
| unbound-long | 20 | 0.832884 | 0.834814 | +0.23% | 12 |
| cpu2-long | 20 | 0.856311 | 0.844097 | -1.43% | 10 |
| same-binary-long | 20 | 0.846326 | 0.863612 | +2.04% | 11 |
| matrix-8-1 | 8 | 0.031349 | 0.032119 | +2.46% | 5 |
| matrix-8-2 | 8 | 0.021154 | 0.019250 | -9.00% | 3 |
| matrix-8-4 | 8 | 0.014543 | 0.014100 | -3.05% | 3 |
| matrix-16-2 | 8 | 0.180533 | 0.183321 | +1.54% | 5 |
| matrix-16-4 | 8 | 0.115214 | 0.124139 | +7.75% | 7 |
| matrix-32-1 | 8 | 3.056183 | 3.096099 | +1.31% | 3 |
| matrix-32-2 | 8 | 1.683609 | 1.665972 | -1.05% | 3 |
| matrix-32-4 | 8 | 0.920451 | 0.946720 | +2.85% | 5 |

## Causality and decision

The original +9.7% single-worker slowdown did not reproduce as a stable effect.
Its sign changed across affinity and duration; the same-binary control itself
showed an apparent +2.04% median slowdown. These observations establish substantial
timing variability, not proof that every adverse result is noise.

Independent static review found no pressure symbols in either particle executable
and no live path from this harness to the changed solver. Particle step, preparation
closures and neighbor instructions match after normalizing relocations/LLVM suffixes.
Addresses differ (step 0x29cf0 versus 0x29d20; Probe::new 0x3ce80 versus 0x3aba0).
Binary-layout effects remain plausible but their causal contribution was not isolated.
Do not attribute the adverse timings conclusively to layout or environmental noise.

The fresh 4096-particle/four-worker workload is adverse in 7/8 pairs (+7.75% median).
That is sufficient unresolved representative risk to reject the candidate under
the original gate. Independent review recommends reverting rather than tuning
linker alignment for this host. Production fusion, observer query and bespoke tests
are restored to baseline. This resolves shipping risk by removing the candidate;
it does not claim to have explained the timing mechanism or recovered a speedup.

The pressure gain (4.6% sustained observed single-worker) is forfeited. No production
optimization is accepted; this is a completed investigation/tooling milestone.

## Reproduction

Build frozen source d03ac6e and baseline as described in results.md using separate
source roots and target directories; the current reverted source is not the rejected
candidate. Run without concurrent builds/tests:

```sh
python3 docs/plans/cpu-pressure/reports/particle_causality.py BASELINE_BINARY CANDIDATE_BINARY /tmp/particle-causality.json
python3 docs/plans/cpu-pressure/reports/particle_controls.py BASELINE_BINARY CANDIDATE_BINARY /tmp/particle-controls.json
```

Scripts require Linux taskset and the recorded CPU IDs for pinned cases.
Do not silently substitute affinity if reproducing on a different host.

## Final local qualification

After revert: release work-observation/default/core suites passed 341/331/324
Rust tests; strict all-target Clippy passed for all three configurations.
Formatting, Python script compilation, release CLI build and both retained
example smoke runs passed. Python oracle/executable suite passed 98 tests
with one existing skip. Commands and outputs: revert-checks.json. Independent
review confirmed the empty src/tests diff and recomputed all follow-up statistics.
Exact-head CI is recorded in PR58 after push; no book gate change is authorized.
