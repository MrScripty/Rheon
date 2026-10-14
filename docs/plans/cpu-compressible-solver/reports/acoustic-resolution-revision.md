# Prospective S1 acoustic resolution revision

## Decision and evidence boundary

Parent approves replacing the **future** S1-E acoustic study population
8³/12³/20³ with 16³/24³/32³ and requiring at least eight cells per shortest
coordinate wavelength. This is a prospective change after observing a failure,
not the original preregistered study. Freeze this independently reviewed plan
revision and both execution subjects through parent before new numerical
acceptance. No native higher-resolution result is available in this report.

Evidence source: [PR73 diagnosis at b5e868d76aaacf3a538fd1004e0ace7f037edf1b](https://github.com/MrScripty/Rheon/blob/b5e868d76aaacf3a538fd1004e0ace7f037edf1b/docs/plans/cpu-cfd-validation/reports/native-s1-diagnosis.md).
The planning agent inspected that exact report/verifier and native example/API
at the native commit below, not raw capture directories in the independent
validation worker's environment. Receipt
locations/pins below are that owner's reported preserved evidence. The original
native source is `1b62ae378c2aaa01327055f121ebc0d869f2c4ac` in PR74.

## Preserve the original failed records

| Record / exact verifier | Retained original evidence | Recorded failure |
| --- | --- | --- |
| Original / `aea43a54d7c8e51b897b21f7e8119c2a22cd1605` | `/tmp/rheon-s1-1b62ae3-aea43a5-original`; origin.json SHA256 `7b53940fd168ae26e9cd0b29ffde0c191dab5b0d01a5be2953de21eae5c4dbec` | `assessment-failed.json`: `sod/initial-cell-average`; no accepted assessment |
| Fresh corrected oracle / `6454acf79279330eb01aaee568f1d347688c84f5` | `/tmp/rheon-s1-1b62ae3-6454acf-sod-fixed`; origin.json SHA256 `e7c098d13eb19fcc54abb37436c0f0e0ffc7e851cd30487e0a590a712c1866a7`; failed assessment SHA256 `3f732f84f3b34b5d0e0109fa149c6a041ce9c5a77574e738030dfefb55405279` | `acoustic/order-below-0.6`; externally pinned verification repeated failure; no accepted assessment |

Keep both directories, origins, failed assessments and subject identities
unchanged under validation ownership. Do not relabel, overwrite, regenerate or
attach a new verifier identity to them. A corrected diagnostic read of old fields
does not rewrite their origin. New studies use fresh owned output paths and
new exact source/binary/verifier/artifact pins. No old failure becomes a pass.

The original Sod t=0 oracle accumulated needless quadrature rounding on constant
states. The independent owner replaced it by direct analytic constant integration,
including a mixed odd-n cell, without widening the `16*epsilon` allowance.
Positive-time quadrature and Sod gates were unchanged. This is an oracle repair,
distinct from the prospective acoustic resolution change and from solver behavior.

## Causal resolution diagnosis

The corrected-verifier capture's native acoustic L1 errors were
0.3127430161, 0.2490710347 and 0.1712388221 on 8³/12³/20³. Errors decrease at both refinements,
but adjacent orders are 0.5614384037 and 0.7334771703: the first fails >0.6.
Independent linearized first-order LLF Fourier prediction gives
0.5614384981 and 0.7334772140. Its agreement is causal evidence for the declared
discretization/resolution limitation; it is not the continuum acceptance oracle
or proof that every native path is correct.

In the unit cube the two fixed modes are (1,1,1) and (1,-1,2). The shortest
coordinate wavelength is 1/2, from the second mode's z component. It contains
`n/2` cells on an n³ grid. A scalar central-gradient symbol has relative defect
`1-sin(theta)/theta`, where the largest `theta=4*pi/n` here:

| n | Cells per shortest coordinate wavelength | Largest modified-gradient defect |
| --- | --- | --- |
| 8 | 4 | 36.3380% |
| 12 | 6 | 17.3007% |
| 20 | 10 | 6.4511% |
| 16 | 8 | 9.9684% |
| 24 | 12 | 4.5070% |
| 32 | 16 | 2.5505% |

This count measures Cartesian-axis phase resolution. The oblique second mode's
wave-normal wavelength is `1/sqrt(6)`; on 16³ that is about 6.53 spacings, not
eight. No wave-normal eight-cell claim is made. The diagnosed largest directional
phase angle in this coordinate-wise LLF operator is the reason for the chosen
coordinate criterion.

The cell-count requirement applies to this coordinate-wise finite-volume
operator and these modes, not a universal acoustic mesh rule. Eight cells gives
a useful declared resolution floor: the worst central-gradient-symbol defect
is then below 10% in this fixture. That percentage is explanatory, not an added
accuracy gate, full LLF error bound or convergence theorem. Dissipative/time
errors and finite-amplitude/reference discrepancy still need observation.

## What changes and what remains fixed

Only acoustic mesh counts change. Same conservative cell averages means the
same analytic averaging rule applied to the same continuous field on each new
cell; it does not mean reuse of old mesh arrays or replacement by point samples.

| Property | Frozen value / interpretation |
| --- | --- |
| Physical case | Unit cube, periodic boundaries, ideal gas gamma=1.4, R=287, background rho=p=1, same five conservative channels |
| Modes | (1,1,1), weight 1, phase 0; (1,-1,2), weight 0.5, phase pi/3 |
| Density amplitude and final time | 1e-6 and t=0.08 |
| Initial/reference averaging | Existing `linear-conservative-cell-average-v1`, analytic per-axis sinc factors; no center-sample substitution |
| Method | Original first-order LLF, piecewise-constant reconstruction, unsplit forward Euler; no S2 method change |
| CFL | Same accepted incident-face bound/CFL policy and requested-final-time control; new h naturally changes actual accepted dt/step count, which must be recorded |
| Norm | Same volume-mean absolute density difference divided by 1e-6; no amplitude, background or error rescaling |
| Refinement gate | Strict error decrease and **both** adjacent `log(error_i/error_j)/log(n_j/n_i)>0.6`; new ratios are 24/16 and 32/24 |
| Speed/amplitude | Same finest-grid first-mode speed error <8%; both modes' speeds and attenuation reported; second-mode speed is not independently gated |
| Other gates | S1-A/B/C/D/F/G, including unchanged Sod grids/window/error allowance and five-component conservation/admissibility criteria |

The revised family has at least 8/12/16 cells per shortest coordinate wavelength.
Choosing it acknowledges where the original coarse study lies outside the
intended refinement regime. It does not lower the deciding order threshold,
choose a favorable norm, discard failed evidence, change physical time or tune
the timestep to a result. The family was selected after diagnosis; that fact and
the inspected Fourier forecasts remain disclosed. It must therefore be treated
as a new limited-domain qualification, not independent confirmation of the
original study. If any revised point/order fails, preserve the failure and
re-plan through parent; do not drop the point, try favorable grids or relax gates.

The diagnosis forecasts revised orders about 0.793 and 0.857 using a linearized
model. These are forecasts only. They are not a pass, native execution, general
first-order convergence claim or a replacement continuum solution. Even a
successful new three-grid study will demonstrate only its stated range/regime;
the original 8³/12³/20³ coarse-grid limitation must remain in capability evidence.

## Coordination and required fresh evidence

1. Obtain independent methodological review of this revision and record all
   findings. Parent pins the reviewed documentation commit and relays it to S1
   worker `01a13913-999b-7511-b9cd-f4677019e018` and independent validation worker
   `01a13906-b6ec-7398-820a-387f852c3015` before new numerical admission.
2. Native owner changes only the example's acoustic fixture population. No
   solver kernel/behavior, higher-order reconstruction, integrator, EOS,
   retry/CFL policy or S2 capability changes are authorized by this adjustment.
3. Validation owner prospectively updates its exact fixture-population admission
   and order denominators together. PR73 b5e868d still admits 8/12/20 and uses
   their explicit ratios; it cannot assess 16/24/32 unchanged. Update focused
   population/order negatives and retain the unchanged continuum oracle, norms,
   thresholds, source-origin protocol and exact failure diagnostics.
4. Parent pins both clean committed exporter and verifier subjects; validation
   owner runs a fresh actual tracked-only native capture and independently
   verifies its origin, actual controls/accepted increments and all S1 exported
   fields. Record full results for all three new meshes and unchanged cases,
   reference uncertainty, errors/orders, speeds/amplitudes, conserved totals,
   positivity, step counts and failures. Public API S1-A/C evidence remains a
   separate native-owner gate; no passing partial check closes S1.
5. Only after every required S1 gate and independent review passes may parent
   admit R1 numerical acceptance. Scoped hosted metadata CI, Fourier agreement,
   prior Sod diagnostics and plan-review approval cannot stand in for native
   execution or empirical vehicle validation. New evidence must cite this exact
   prospective revision; do not consume moving main as an accepted checkpoint.

Plan-only write set is the canonical plan/handoff/issues/ledger and authored
revision/verification reports. Numerical export/verifier edits and execution
belong to the separate owners. Composition, ownership and all non-resolution
decisions remain unchanged. Methodological review and delivery results are
recorded in [verification](verification.md); new numerical acceptance is blocked.
