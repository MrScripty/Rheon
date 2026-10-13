# Constitutive and validation research: decisions before implementation

Read on 2026-10-13. This is a research record for the remaining teaching work,
not an implemented turbulence solver or a report of completed CFD benchmarks.
No source code, figures, meshes or datasets are imported from these sources.

## Pin the turbulence variant

The proposed teaching target is incompressible **SST-2003**, retaining the
isotropic turbulent stress. The current [Turbulence Modeling Resource](https://tmbwg.github.io/turbmodels/sst.html)
specifies strain magnitude in the viscosity limiter, limited production in
both transport equations, and the corrected omega-production expression.
Its specification gives

\[
 S=\sqrt{2S_{ij}S_{ij}},\quad
 \mu_t=\frac{0.31\rho k}{\max(0.31\omega,SF_2)},\quad
 \widetilde P=\min(P,0.9\rho\omega k).
\]

The omega production is \(\gamma\widetilde P/\nu_t\), with
\(\nu_t=\mu_t/\rho\), not an unlimited strain source. Inner/outer coefficient
pairs are \(\sigma_k=(0.85,1)\), \(\sigma_\omega=(0.5,0.856)\),
\(\beta=(0.075,0.0828)\), \(\gamma=(5/9,0.44)\); \(\beta^*=0.09\).
The cross-diffusion floor is \(10^{-10}\). The cited resource defines the
complete transport/blending functions; those must be adopted together rather
than assembled from different variants. Its “m” suffix identifies omission
of the isotropic turbulent stress. Its wall prescription uses \(k=0\) and
\(\omega=60\nu/(\beta_1\Delta d_1^2)\), with wall-adjacent distance
\(\Delta d_1\). The source also documents the original 2003 production typo.

[Primary report: Menter, NASA TM 103975 (1992)](https://ntrs.nasa.gov/api/citations/19930013620/downloads/19930013620.pdf),
printed pages 3–9, was read for the BSL blending construction, shear-stress
limiter motivation and wall prescription. It transforms the outer model into
k–omega variables, blends its coefficients and cross-diffusion with the inner
model, and limits eddy viscosity to account for boundary-layer shear-stress
transport. Its SST viscosity uses vorticity; its near-wall coefficients and
blending formulas differ from the later specification. Therefore this report
supports the physical motivation, **not an assertion that the 1992 and 2003
formulas are identical**. The paper reports particular computations; it does
not validate a Rheon implementation. The original 2003 proceedings article
has not been read here; its correction is sourced to the maintained resource.

The following implementation decisions are ours and still require explicit
validation. Use supplied positive omega and nonnegative k in the first
constitutive probe. Display the raw and limited production separately. For a
prescribed simple shear, the displayed stress arrow is a constitutive response,
not a resolved eddy or a timestep. Do not animate invented turbulence. A complete
transport solver additionally needs positivity treatment, wall distance,
face diffusion, inlet/backflow data, coupled residuals and consistent pressure
and stress conventions. It must define the laminar k=0 limit without evaluating
an undefined 0/0 omega source. Arbitrary epsilon clamping is a different numerical
method and needs its own stated limit and convergence study.

For incompressible flow, absorbing the isotropic term into a modified pressure
changes the pressure variable reported to a user. Recover the declared physical
pressure before comparing surface loads. Rotation/curvature corrections,
transition modeling, sustaining terms and compressible energy closure are
excluded unless separately specified. A theorem about a nonnegative algebraic
viscosity or clipped production cannot establish any of those transport,
separation or load predictions.

## A benchmark is a complete boundary-value problem

The official [DFG 2D-1 benchmark definition](https://wwwold.mathematik.tu-dortmund.de/~featflow/en/benchmarks/cfdbenchmarking/flow/dfg_benchmark1_re20.html)
sets a stationary channel \([0,2.2]\times[0,0.41]\), excluding the circle
centered at \((0.2,0.2)\) with radius 0.05. Density is 1 and kinematic
viscosity 0.001. The parabolic inlet has peak speed 0.3, mean 0.2 and diameter
Reynolds number 20. Walls and cylinder are no-slip. The outlet is specifically
\(\nu\partial_nu-pn=0\). The reported force convention uses
\(\nu\nabla u-pI\) and the cylinder's outward normal. Reference coefficients
are drag 5.57953523384, lift 0.010618948146 and pressure difference
0.11752016697 between \((0.15,0.2)\) and \((0.25,0.2)\).

Our consequence: replacing that outlet by zero full symmetric traction changes
the test. A general engineering stress chapter can correctly use symmetric
Cauchy stress while a benchmark reproduction must still honor its specified
operator, traction and normal conventions. Document the equivalence, if any,
for the exact boundary and quantity; never assume it globally. This benchmark
is laminar and cannot by itself validate SST. No Rheon values are available in
this research record, and the references above are not Rheon measurements.

A reproducible future run must retain mesh/unit/topology identity, inlet and
outlet definitions, force quadrature and signs, pressure sample interpolation,
nonlinear and linear stopping histories, and spatial refinement. Iterating to a
small continuity residual alone is insufficient, as the chapter 4 two-path
counterexample demonstrates. A load comparison must include momentum residual
and mesh error before attributing disagreement to turbulence modeling.

## Verification and uncertainty remain separate

[NASA's spatial convergence tutorial](https://www.grc.nasa.gov/www/wind/valid/tutorial/spatconv.html)
describes Richardson extrapolation and Grid Convergence Index assessment, with
safety factor 1.25 for studies using at least three grids and 3 for two grids.
Its asymptotic-range check relates the two GCIs through the refinement ratio
and observed order. A GCI report therefore needs evidence that the refinement
sequence is appropriate, not just three plotted values.

Our teaching fixture should calculate an actual discretized quantity with a
known exact answer before introducing unknown-reference CFD. Reject an observed
order when successive differences vanish, change sign under a formula assuming
monotonic convergence, or cannot be resolved above roundoff. Spatial, temporal,
iteration and measurement/model discrepancy are separate entries in an error
budget. A safety-factor estimate is not a probability distribution or a proof
of a universal confidence bound. A learned pressure proposal must pass the same
source/domain/gauge/full-residual gate as any candidate; inference latency and
physical acceptance need separate measurements. No real-model inference or
speedup is established by this proposed teaching gate.

## Completion ledger for this record

Read: maintained SST equations/variants, the primary 1992 model and wall
sections, official Re20 geometry/BC/force definitions, NASA spatial-convergence
procedure. Decided: exact variant, pressure convention, laminar benchmark scope,
and required failure cases. Still to implement and qualify: wall-work probe,
actual sparse iteration/memory/roundoff lessons, constitutive probe,
refinement/error-budget explorer, their bounded Lean statements, and remaining
chapter/appendix placements. This record does not close those ledger items.
