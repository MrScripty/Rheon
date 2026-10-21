# Q1 native result — incomplete accuracy qualification

Scientific source freeze `b9a9d4ca3ff02c9eae38eb20dced49e87a923380`.
One native Linux CPU campaign; native exit1 retained. No criterion/tolerance changed,
no refused sphere/transport run restarted. A frozen evaluator additionally exits1
on missing required row counts; it cannot produce a positive result from this run.

## Channel and original diffusion comparator

| Wet levels | Relative profile RMS | Relative flux error |
|---|---:|---:|
| 8 | 2.1393127492% (FAIL fixed2%) | 3.125% |
| 16 | .5348817394% | .78125% |
| 32 | .1337212717% | .1953125% |

Spatial orders1.99985555,1.99999097. Original temporal sine-mode RMS at
8/16/32 steps: .006386724998/.003245674510/.001636294871; observed temporal
orders .97655800/.98808557. Energy did not increase. Both wall tractions
.004991Pa match rho*a*H/2 to displayed precision. Native momentum/energy/force
residuals are retained with every step/profile and native geometry, independently
reviewed before disposition. This reduced aligned shear is not general3D water.

Unchanged auxiliary `liquid_volume` example retains all four slab comparators:
L1 volume errors .09375/.068359375/.04909515381 at n16/32/64,C.5, while n64,C.25
has .06039030345. Represented volume .25m³ in each. Smaller dt at fixed grid
increased donor diffusion; this distinct example does not replace audit119's
frozen3D export evidence.

## Interface and source-geometry failures

The required18 trajectories /9 geometry fields /18 Full/Dry controls are
incomplete:12 /7 /12 were produced before the first n64 sphere-source refusal.
The `translation-initial/64` paired sphere quadrature exhausted its fixed depth22.
Its empirical uncertainty8.0853e-12 is small, but is not a rigorous error bound
and cannot override the explicit no-exhaustion gate. The candidate geometry is
retained, and no n64 transport run or finest-pair order claim exists.

All twelve n16/32 mixed trajectories reachedT1 and respected native mass/bounds
checks. Relative represented-volume drift max1.2271e-14. Their relative shape
L1 errors are large: translation .9506–1.0331 at n16 and .7611–.8391 at n32;
adapted vortex1.2713–1.3363 and1.1057–1.1807 respectively. These are available
coarse diagnostics, not the missing finest accuracy criterion.

Full controls additionally refused native fraction1.0000000000000002 at
(n16,C.5,cell56), (n32,C.1,cell373), (n32,C.5,cell180). Accepted state remains
unchanged on refusal. No clamp, mass reset, retry or silent safety relaxation was
introduced. Full/Dry target completeness and support are not admitted.

Vortex scope is only the documented dyadic2D curl extruded through a sphere,
T1 abrupt autonomous reversal. It is not the full Enright3D/T3 benchmark. Actual
forward/reverse dt sums and float carriers are retained in raw ledgers/arrays.

## Disposition

No Q1 transport-accuracy admission; no production water repair, general mesh-water,
physical validation, GPU/native UI, or Q2 coupled-Stokes accuracy claim.
The aligned channel resolved envelope is numerical verification of its existing
owner. Q1 failures justify only a separately frozen Q2 geometry/reference
prerequisite: common volumes/open faces/boundary moments and actual CPU polynomial
flux/stress closure and geometry-refinement checks. Production transport selection
and full coupled MMS/hydrostatics remain gates with unchanged plan criteria.
Independent raw-data review and regression disposition are recorded separately.
