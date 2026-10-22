# Independent Q1 post-run disposition

Frozen scientific source/protocol: `b9a9d4ca3ff02c9eae38eb20dced49e87a923380`.
Only `independent-new/` was written by this reviewer. No native candidate,
campaign, branch file, criterion, or existing evidence was edited or rerun.

**The channel verification is supported. The interface campaign is incomplete
and cannot receive its requested accuracy admission.** Native execution stopped
on the declared 64-grid sphere quadrature exhaustion gate. Independent agreement
with that retained array does not turn the failed native gate into a pass.

Retained cardinality is exactly 6 channel summaries, 116 channel step reports,
3064 channel layer states and 56 channel geometry rows; 12/18 completed transport
trajectories, 7/9 geometry fields and 12/18 constant controls. There is no 64-grid
transport result or finest-pair spatial order. The full frozen evaluator requires
the complete matrix; these independent partial reconstructions do not replace it.

## Channel reconstruction

`reconstruct.py` uses 70-digit Decimal arithmetic to reconstruct every retained
step's kinetic energy, increment energy, bulk plus boundary dissipation, body
work, momentum, body and wall impulses, true equation residual and rounding
budgets from the actual adjacent layer states, masses and conductances. It
also checks geometry/centroids, source-force units, summary maxima, physical
flux and traction. Geometry matches the source channel: maximum cell-volume
relative deviation is 6.00e-15, centroid deviation 2.17e-19 m, and the exported
mass/conductance coefficients agree with their independently reconstructed
physical definitions.

A separate generalized symmetric eigendecomposition of the stored mass and
stiffness gives the closed-form backward-Euler modal solution at every saved
time, without advancing another reference trajectory. Its maximum velocity
discrepancy is 2.60e-14 m/s. The independently reconstructed instantaneous
ledger defects remain at rounding scale. Steady-case actual reported maxima,
normalized by the frozen physical scales, are 6.263e-14 energy, 3.744e-14
momentum and 1.445e-15 equation residual, all below 1e-10.

| Wet levels | Relative velocity RMS | Relative volume-flux error |
|---|---:|---:|
| 8 | 2.139312749% | 3.125% |
| 16 | 0.534881739% | 0.78125% |
| 32 | 0.133721272% | 0.1953125% |

Spatial orders are 1.99985555 and 1.99999097. The 8-level velocity result still
fails its original 2% criterion. Both resolved levels pass the new resolved
velocity envelope. Native wall-traction magnitudes reconstruct as 0.004991 Pa
on each stationary wall, up to rounding; this reduced-channel agreement does
not admit arbitrary 3D stress or free-surface traction.

The separate fixed-spatial-operator temporal study has RMS errors
0.006386725, 0.003245675 and 0.001636295 m/s at 8/16/32 steps, with orders
0.97655800 and 0.98808557. These support the declared first-order backward-Euler
time claim against its semidiscrete decay reference. The continuum spatial
error is not substituted for this temporal error.

## Transport, carriers and constant controls

All supplied f32 carrier arrays have matching periodic seams and exactly zero
native discrete divergence when independently decoded. Outward and reversed
outward rates reconstruct exactly. Every retained trajectory reaches T=1 s,
with the ledger's timestep, volume difference, zero external flux, divergence
and summed outward Courant reconstructed. Maximum actual forward/reverse
sum(dt) mismatch is 4.219e-15 s; this is a retained finite-time representation
error, not a claim of exact operational reversal.

For the six constant-velocity trajectories, an independent product of periodic
upwind Fourier symbols reproduces each full saved final field within 1.111e-15
in fraction. Saved L1 and discrete Fourier diagnostics reconstruct within
1.10e-14 and 1.54e-14 respectively. The maximum native-reported relative mass
drift is 1.2271e-14; stable independent summation gives at most 1.228e-16 for
the same stored fields. Both are far below 1e-10, but conservation alone does
not establish shape accuracy.

| Case | C | E_L1 at 16 | E_L1 at 32 | Single available pair order |
|---|---:|---:|---:|---:|
| Translation | .1 | 1.033111 | .839138 | .30001 |
| Translation | .25 | 1.005394 | .812178 | .30789 |
| Translation | .5 | .950567 | .761136 | .32063 |
| Adapted vortex | .1 | 1.336294 | 1.180684 | .17861 |
| Adapted vortex | .25 | 1.314800 | 1.155071 | .18686 |
| Adapted vortex | .5 | 1.271279 | 1.105685 | .20134 |

E_L1 is normalized symmetric volume difference against source cell averages,
not a percentage in this table; it may exceed one. These available-grid errors
are much larger than .05. There is no measured 64-grid error or finest pair,
so no finest-grid threshold or three-level convergence result is asserted.
The lower-C comparator is more diffusive here; no universal improvement from
reducing dt is inferred. The vortex remains the explicitly adapted 2D RT0
carrier extruded through the 3D sphere, not the full 3D deformation benchmark.

The six dry controls pass. Three full controls pass with changes of at most
1.111e-16; three full controls refuse. Independent reconstruction of the declared
f64 face-product arithmetic using the retained f32 fields reproduces the exact
first offending cells: 56 at n16/C.5, 373 at n32/C.1, and 180 at n32/C.5.
Each offending fraction is 1.0000000000000002. That computation is a diagnostic
of the saved inputs, not an additional native attempt. Accepted constant state
is preserved on refusal. The raw bounds policy rejects these results; no
clamp, tolerance relaxation, or post-run corrected comparator is introduced.

## Separate per-cell source geometry oracle

`geometry_oracle.py` constructs a different positive clipped-chord integration:
it partitions an inner coordinate at chord/box contacts and integrates its
positive interval length, followed by SciPy QUADPACK Gauss-Kronrod in the outer
coordinate. It does not import the candidate, use its four-corner disk-area
formula, or use adaptive Simpson. Exact sphere reflection/permutation symmetries
cache repeated integrals without approximate key matching. Independent nearest
box-point and all-eight-corner predicates classify every retained field; contacts
within an explicit roundoff band receive Decimal checks. No native source
fraction is clipped or renormalized.

Every native FULL/EMPTY cell agrees with its independent classification.
Every one of the seven stored fields passes the frozen independent source gate:
normalized sum of cellwise absolute discrepancy plus the oracle's aggregate
integration estimate is below 1e-4. Maximum cellwise normalized discrepancy is
1.399e-14; maximum aggregate oracle integration estimate is 4.983e-12. There are
no SciPy integration warnings. All independent per-cell arrays and classification
maps are retained. Integration estimates are empirical, not rigorous enclosures.

For the refused translation-initial/64 field specifically, normalized cellwise
discrepancy is 6.776e-16 and aggregate estimate is 2.183e-13. This is evidence
that the saved geometric field agrees closely with an independent integral; it
does not discharge the native termination/exhaustion criterion. `exhausted=true`
remains a failed native admission, the executable's exit1 remains retained, and
missing final/initial fields and all 64-grid trajectories remain absent.

## Dependency disposition

Preserve this single incomplete campaign, the original coarse channel failure,
all three P1 campaigns and the current donor version. The observed interface
diffusion and raw Full-cell refusals block positive interface promotion. Do not
claim an n64 accuracy/order result, full coupled water acceptance, physical
validation or native UI/GPU capability.

A smallest Q2 common-geometry prerequisite is justified for implementation and
review, while its coupled Stokes/MMS, true solid-exterior geometry, absolute
free-surface pressure, boundary traction and transport method-selection gates
remain pending. A convex-fluid polygon integration reference can support common
moments and oriented flux assembly; it cannot be represented as completion of
the original coupled Q2 capability.

Reconstruct with `python3 reconstruct.py RAW_DIR OUTPUT_DIR` and
`python3 geometry_oracle.py RAW_DIR OUTPUT_DIR`. Python/NumPy/SciPy versions,
all raw input hashes, numerical comparisons and per-step/per-field results are
retained in `field-summary.json` and `geometry-summary.json`.
