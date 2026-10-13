# Initial-residual and warm-start assessment

## Decision

Reject the tested initial-zero-residual shortcut. No production change remains.
The 48 primary quiescent pairs had small improving median ratios, but mixed paired
outcomes and overlapping ranges; they did not demonstrate a gain beyond observed
variability under the predeclared gate. The 90 active/particle controls were mixed.
Do not call every adverse result a regression, or dismiss it as noise. Do not tune
binary layout or add samples selectively to obtain a favorable outcome.

All 138 pairs matched required field/report fingerprints, accepted horizons,
iteration counts where applicable, counts and managed memory. The candidate passed
12 targeted pressure tests including hand residual/gauge, signed zero, cancellation,
retry and scaled-divergence overflow. Independent review found no correctness blocker.
Full candidate regression and 16³/64³ acceptance expansion were not pursued after
the timing gate failed. This is a completed bounded assessment, not a speedup.

## Provenance and protocol

Baseline main: 2209012bc5f902693d0f695ec1c2d854126af317. Standards snapshot:
188beda1fa477d21d576c233dd8c7f4c4c267d23. Candidate source/test patch is candidate.patch;
frozen binaries/source hashes are in artifact-hashes.txt. Independent build roots
and target directories were used, with locked Rust 1.92 release builds and no
concurrent compilation/tests during measurements. Same cloud host as PR58: AMD
EPYC 9V74, five visible CPUs, four-CPU quota, 16 GiB memory. CPU evidence only.

The pressure harness is adapted from PR58/PR46 and adds explicit rest/emitter
workloads; particle harness is copied unchanged from PR58. Each pair has the same
settings, workload and physical horizon. Pressure dt = .02, tolerances 1e-9/1e-12,
predicted divergence 1e-7 and actual 1e-5; no settings change. Primary cases: 32³, 128
rest steps, 1/4 workers,observed/unobserved,12balanced shuffled AB/BA pairs per case
(seed 602). Controls:32³ emitter 8 steps, 1/2/4 workers, 6 pairs; dense particles 512/4096/32768,
20steps at dt = .001,1/2/4workers, 8 pairs (seed 603). Every trial is retained.

The fresh baseline.json has six sustained whole-step/phase snapshots at 16³/32³
and 16x32x8. These single observations locate work; they are not comparative claims.
rest-baseline.txt recorded zero PCG iterations but 128 applications of A*0 and 256
norms in 128 rest steps: 1.6974 s frame, .02384 s operator, .01576 s reductions. The admitted
opportunity was small (~2% from these stages). Phase counts change in the candidate;
whole-frame time, not phase averages, decides acceptance.

## Timings

Seconds per complete run. Positive delta is slower candidate. Paired median is
the median of per-pair percentage changes; it need not equal ratio of medians.

| Case | Pairs | Baseline median [min,max] | Candidate median [min,max] | Median ratio delta | Paired median delta | Slower pairs |
|---|---:|---:|---:|---:|---:|---:|
|rest32/1/observed=False|12|4.297629 [4.108704,4.526976]|4.160483 [3.957015,4.399099]|-3.19%|-1.52%|4|
|rest32/1/observed=True|12|4.184648 [3.985799,4.357022]|4.133922 [3.987156,4.343309]|-1.21%|-0.86%|5|
|rest32/4/observed=False|12|1.745812 [1.673908,1.895261]|1.688966 [1.584691,1.935762]|-3.26%|-1.03%|5|
|rest32/4/observed=True|12|1.696541 [1.610882,1.822786]|1.667386 [1.604183,1.741388]|-1.72%|-3.86%|3|
|emitter-32-1|6|2.140706 [2.045591,2.299920]|2.171425 [2.026102,2.296345]|+1.44%|+0.84%|3|
|particles-8-1|8|0.033521 [0.030747,0.039645]|0.034148 [0.031119,0.043265]|+1.87%|+6.13%|5|
|particles-16-1|8|0.349229 [0.333470,0.378082]|0.360329 [0.322620,0.392077]|+3.18%|+3.05%|5|
|particles-32-1|8|3.185595 [3.044903,3.449500]|3.107192 [3.003061,3.464148]|-2.46%|-1.58%|3|
|emitter-32-2|6|1.844216 [1.644090,1.983178]|1.742468 [1.683745,1.893308]|-5.52%|-3.19%|1|
|particles-8-2|8|0.019633 [0.018703,0.020720]|0.018877 [0.017861,0.022090]|-3.85%|-3.85%|2|
|particles-16-2|8|0.183774 [0.176104,0.207545]|0.189935 [0.177551,0.208458]|+3.35%|+0.34%|4|
|particles-32-2|8|1.741896 [1.636703,1.816281]|1.706704 [1.653674,1.752851]|-2.02%|-1.31%|3|
|emitter-32-4|6|1.646516 [1.505188,1.701782]|1.634144 [1.537425,1.750033]|-0.75%|-0.49%|2|
|particles-8-4|8|0.018300 [0.012351,0.036724]|0.014286 [0.012935,0.018570]|-21.94%|-14.85%|4|
|particles-16-4|8|0.114650 [0.109485,0.127945]|0.119383 [0.106877,0.147082]|+4.13%|+6.92%|5|
|particles-32-4|8|0.968320 [0.909066,1.019240]|0.938108 [0.924480,0.996772]|-3.12%|-3.43%|3|

## Why not the other mechanisms?

PressureWorkspace already allocates its six arrays once and reports their capacities.
Simulation owns accepted/candidate field buffers and swaps at commit. Fixed-box
diagonal combinations are already precomputed in PressureOperator. No new measured
application-buffer allocation target was found; this is not an assertion that Rayon
or the allocator performs zero allocations. Column geometry and SGS still have
recomputation possibilities, but these Jacobi profiles do not justify changing them.

Warm-starting addresses iteration count and therefore remains a plausible larger
opportunity. It is not admitted as a workspace-only cache: solve success precedes
f32 divergence, Courant, tracer, liquid and final cancellation acceptance. The last
workspace pressure can belong to a rejected attempt or a different domain/route.

## Concrete next dependency

Coordinate a frozen integration base and accepted-pressure seed ownership with the
PR55 owner (head 3525c979e16629b295a6865e3abbb2a36803cc93 when inspected). Its changes
in simulation.rs and liquid_step.rs add provider pressure selection at the exact
prepare/commit seam. Main already also supports boundary/column pressure routes.
PR55 is a coordination dependency, not a mathematical prerequisite or an AI-replacement
request. Do not silently stack or independently change shared commit semantics.

The next warm-start slice needs an owner-agreed contract:

- Explicit opt-in seeded CPU solve; default cold deterministic reference retained.
- Cache pressure only after whole-step/whole-liquid acceptance, never after PCG alone.
- Identify native/provider/boundary/column pressure source in commit outcome; invalidate
  on route/operator/domain/reset changes. Same cell count is insufficient identity.
- Preserve closed Neumann compatibility and pinned gauge; do not reuse closed seeds
  for changing free surfaces without their own qualification.
- Retry/cancellation leaves the last accepted seed intact; reset clears validity.
- Account for retained seed memory (8 bytes/cell if a new array is needed), within
  existing memory limits. Reuse an already-owned accepted pressure only with its
  owner’s lifetime/provenance contract.
- Then measure cold/steady whole steps with unchanged tolerances, independent physical
  oracles, full rollback/invalidation tests and representative negative controls.

No source/API changes, new draft PR, merge or book modifications are justified by
this assessment. PR58 remains intact. A future implementation requires the above
integration decision and fresh seeded-trajectory evidence.

## Reproduction

Archive main 2209012 into separate baseline/candidate roots. Copy these two .rs
harnesses into each root’s examples directory. Apply candidate.patch only to the
candidate root. Build both examples with --release --locked --features work-observation
and distinct CARGO_TARGET_DIRs. Never build the rejected candidate from this final
unmodified production tree without applying its patch. Then run:

```sh
python3 docs/plans/cpu-cache-assessment/measure.py BASELINE_PRESSURE CANDIDATE_PRESSURE /tmp/initial.json
python3 docs/plans/cpu-cache-assessment/controls.py BASELINE_PRESSURE CANDIDATE_PRESSURE BASELINE_PARTICLE CANDIDATE_PARTICLE /tmp/controls.json
```

The .rs files here are reproduction inputs, not new public application entry points.
