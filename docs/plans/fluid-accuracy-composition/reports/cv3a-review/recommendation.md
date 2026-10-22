# CV3-A admission recommendation — static polynomial polygon MMS

**Recommend CV3-A as the next separately frozen numerical milestone, ahead of a literal paper-disk campaign.** It is a useful nonrepresentable, independently forced consistency/convergence test of the selected full-mass staggered-Q1/P0 formulation and CV2 solver. It is not a substitute for the canonical Larionov disk, a generic stability result, moving-surface method, production capability or current numerical pass. Admit protocol/source preparation subject to the explicit prerequisites below; execution and acceptance follow only after the independent oracle and resource freeze.

Review subject: PR139 report head `902f6092de6044419d028f92dabd8e59ae7a87ba`, numerical source `b0a9d52cc266cb651c5e67c07ec5f4f82419db55`. I read the CV1 decision/mathematics/protocol, CV2 method/protocol/qualification/source review, actual basis/trace/experiment implementation and canonical verification criteria. CV2's reported 24 cases, 15 debug/15 release contracts and45tool checks remain owner-attributed; no raw campaign, build or test was rerun here.

## Why this comes before the literal disk

CV1 makes one physical velocity reconstruction, support, full M/B/K and trace explicit; CV2 establishes a bounded accurate solve for its fixed corpus. An affine patch can remain exact even when approximation properties are inadequate. The proposed quadratic velocity and nonconstant pressure leave those finite spaces, exposing approximation, pressure response, physical stress trace and grid-phase sensitivity with exactly integrable data. Fixed polygons separate these issues from changing curved geometry and from the singular-looking radial expressions at the paper's disk centre. Passing this pilot would justify only its finite static cases and a subsequent method decision.

The [primary supplement](https://cs.uwaterloo.ca/~c2batty/papers/Larionov2017/supplemental.pdf), section1 and equations5/10, describes a different disk test with zero total traction and nonzero interface pressure. Its continuum equations motivate the load signs here; they do not make prescribed polygon traction equivalent to that test. The canonical r=.75 disk,32/64/128 and rescaled-water/annulus gates remain unchanged and unmet. A polygon approximation, a nine-case pilot or a dense backend is not their replacement.

## Freeze the actual continuum problem

Use physical coordinates X=x/Lref and Y=y/Lref with a fixed global origin, Lref=Uref=Href=1, rho=1, mu=the represented binary64(.1), dt=1, and fixed nonzero stress scale

`tau_ref=max(mu Uref/Lref, rho Uref Lref/dt)=1 Pa`.

Set **u*=0** explicitly. With psi=Uref Lref(X²Y+XY²),

- u=Uref(X²+2XY, -2XY-Y²), so div u=0 and u is not in the componentwise bilinear space;
- p=tau_ref(1+X+2Y+XY), an **absolute** pressure not in the primal P0 space;
- sigma=-p I+mu(grad u+grad uᵀ), g=sigma n on every original polygon edge;
- f=(rho/dt)u-div sigma in the body, not a matrix action on desired coefficients.

In the stated unit scales, the stress components are `sigma_xx=-p+4mu(x+y)`, `sigma_yy=-p-4mu(x+y)`, `sigma_xy=2mu(x-y)`. The body force is

`f=(x²+2xy-2mu+1+y, -2xy-y²+2mu+2+x)`.

Keep these as symbolic expressions evaluated from the frozen primitive constants; do not silently replace exact represented mu with rational1/10 or independently rounded .2/.8 literals. R1 uses actual exported native rounded data; R2 reconstructs exact represented primitive data before arithmetic rounding, as in CV2. Export all primitives and actual loads.

The saddle equations remain `A U-BᵀP=F`, `B U=0`, A=M/dt+K, now with `F_i=integral_Omega phi_i·f + integral_Gamma phi_i·g`. Never form F as A times an interpolated exact velocity minus Bᵀ times sampled pressure. This is one time-discretized planar Stokes problem per case, per1m depth. Only polygon side edges form Gamma; no physical 3D end caps or interface motion is implied. There is no pressure gauge or prescribed pressure boundary value in addition to g.

The energy and momentum checks must include **both body and boundary** load/work/resultant/torque. Zero u* does not imply nonincreasing kinetic energy in this forced problem. Reuse the full residual/work identity, not CV2's unforced energy inequality or boundary-only momentum formula.

## Nine fixed cases and two source prerequisites

| Physical polygon | Grid origins | n |
| --- | --- | --- |
| CV1 rectangle (.17,.23)–(.83,.79) | (0,0) |4,8,16|
| Exact represented CV1 square, side.3, centre(.537,.479),17degrees | (0,0) |4,8,16|
| The **same** square bytes | (h/4,h/4), h=1/n |4,8,16|

Use extent1 and record the shifted computational domain explicitly. Pin the represented polygon vertices once, including square vertex bits; do not regenerate or move the physical polygon/MMS with the grid, scale geometry with h, rotate the field, or reconstruct vertices from different trigonometric evaluations per level. Keep the source within both staggered lattice hulls. Phase changes the grid, not the physical problem. All positivity, support, coincidence and representability refusals remain exact, with no floor/deletion/merging/stabilization/gauge.

1. **n16 is not currently supported by the unchanged CV1 constructor.** `basis.rs::Layout::new` explicitly accepts only n2..8. Admit an allocated, independently reviewed bounded ceiling extension for this new source subject, with unchanged mathematical basis/support/M/B/K formulas, retained old-corpus byte controls and an above-ceiling refusal. Do not describe the implementation as wholly unchanged source, or quietly coarsen the protocol to n8. CV2 solver/fixed refinement can remain byte-identical.
2. **The new load cannot use CV1's two-point constant-stress edge rule unchanged.** On an oblique edge the Q1 trace has degree2 and this pressure/traction degree2, so their product can have degree4; two-point Gauss is only degree3-exact. Use a separately frozen degree>=4-exact rule, e.g. three-point Gauss after all common half-cell knot and pressure-cell splits, with independent exact polynomial segment moments. Preserve the existing CV1 trace maps/evidence. The current positive3x3Duffy volume rule is sufficient in real arithmetic for Q1 times this degree2 body force; independently verify rather than assuming native floating integration is exact.

These are prospective source obligations, not observed failures of the unrun pilot. Keep their allocation small and versioned; no production pressure/geometry/Session integration follows.

## Measurements, oracle and numerical-error budget

Freeze complete velocity reconstruction and its derivatives on every common integration piece, actual P0 pressure in every positive cut cell, and the liquid-side physical stress trace on each source-edge piece. Pressure and gradient traces must split at their actual discontinuity/knot locations; averaging, extrapolation, nodal-only error or fitting a new stress reconstruction would change the test.

Use physical-domain norms, not an unweighted list of nodal coefficients:

- velocity relative L1/L2 errors against the nonzero analytic field;
- absolute-pressure relative L1/L2 errors against the nonzero analytic p, with **no mean subtraction**;
- boundary traction L1/L2 errors for sigma_h n-g, normalized by tau_ref and the corresponding boundary measure; report normal and tangential parts and edgewise maxima separately;
- L-infinity diagnostics with a declared search/evaluation definition, without claiming cut-boundary second order.

State whether L1 uses Euclidean vector magnitude or a componentwise norm, and use it consistently across levels/phases. L2 squared field and traction errors are piecewise polynomials and admit independent exact moment integration. **Absolute-value L1 integrands are not globally polynomials.** Use an independent bounded integration/root-partition procedure, retaining its numerical uncertainty and refusal when the budget cannot resolve it; do not call Gauss sampling an exact polynomial L1 oracle.

Retain the pilot's fixed finest5% velocity/pressure/physical-traction targets and at least.9 L1/L2 finest-pair orders for each three-level family, with the precise metrics/denominators frozen before data. Report both4→8 and8→16 orders; specify explicitly which pairs are deciding. Apply thresholds and orders conservatively to norm-error intervals, not rounded central values. For example, a conservative finest-pair order uses log2((E8-delta8)/(E16+delta16)) when its numerator is positive. Do not claim canonical disk acceptance from passing these pilot metrics.

Keep independent physical traction error separate from algebraic boundary-load assembly, momentum residual, continuity and work checks. Here g is prescribed and nonzero: there is **no zero-traction residual to celebrate**. Preserve full original-coordinate residual/work1e-10 and fixed CV2 R1/R2 solution-comparison gates, alongside the new physical metrics. Also report physical body/boundary power and momentum separately so cancellation cannot hide a missing load term.

Independently assemble exact polynomial loads/norms and M/B/K from the represented physical source. Reuse neither native Gauss implementation nor native exported operators as the R2 oracle. Retain independent highprecision LU references to both R1 and R2, all selected native refinement states and the R1–R2 gap. Do not change or select the CV2 solver/refinement count after observing errors.

**Freeze a quantitative original-coordinate numerical-error budget before launch.** Propagate native solve/assembly/reference/integration uncertainty into the physical velocity/pressure/traction norms and keep it well below the measured discretization error and refinement difference, e.g. a predeclared1% budget for each. Use worst-case intervals for5%/.9 decisions. Small transformed residuals and80/120digit agreement alone do not establish a guaranteed forward-error enclosure. If the protocol requires a literal certified bound, select a validated solve or residual plus verified inverse bound before execution; otherwise explicitly call the CV2-style highprecision comparisons numerical-reference estimates, and do not claim that the guaranteed-bound clause has passed. Resource inability to establish the selected error budget is a stopped/blocked result, not permission to accept residual-only accuracy.

## Stability diagnostics and stop rules

Require exact rational full row rank of B for each of the nine actual geometries/meshes. Retain all pressure rows, including the constant mode: prescribed total traction makes absolute pressure observable. No rank-tolerance deletion or added gauge.

For a comparable finite diagnostic, declare the velocity Riesz matrix

`Gv_ij=integral_Omega (grad phi_i:grad phi_j + Lref^-2 phi_i·phi_j)`

and pressure L2 mass `Qp_cc=volume(Qc)`. Report the spectrum of `Qp^-1/2 B Gv^-1 Bᵀ Qp^-1/2` (or the equivalent generalized eigenproblem), its minimum singular-value square root, eigenvector/pressure-cell pattern and precision uncertainty. Use full consistent vector H1 norm: the mass term avoids rigid-mode kernels, and the gradient is not silently replaced by symmetric strain. CV2's A/Schur equilibration spectrum has a different norm/material/dt meaning and cannot substitute for this diagnostic. Report conditioning/near-checkerboard evolution and original/scaled matrices separately. Do not choose an arbitrary beta cutoff, suppress a small mode or claim a finite observed spectrum proves mesh/cut-uniform inf-sup stability. Exact rank failure is a method failure; a small positive mode still must meet the independent solution-error budget.

Before the one campaign freeze numeric native/reference wall-time, RSS, output and total budgets plus supported precision/rank algorithms. At n16 the absolute full-grid upper bounds are544velocity coefficients,256pressure cells and800KKT rows; dense/BigRational/highprecision work is not automatically feasible just because binary64 matrices fit memory. Keep per-case source/attempt/refusal logs and completed outputs on a later failure. If n16 rank, independent highprecision/reference integration or dense solve exceeds the frozen resources, stop/replan with the failed case retained. No silent coarsening, extra precision/backend fallback, support pruning or posthoc tolerance increase under the same version.

Before raw evaluation, freeze normal/-O/-OO always-on oracle guards and negative controls: incorrect/omitted/reversed pressure traction, wrong body force, wrong phase/source/pressure trace, insufficient oblique-edge quadrature, and a deliberately wrong final field with self-consistent rewritten residual metadata. The intended accuracy/load guard must reject each control; parser failure alone is insufficient. Source/binary/commands/raw outputs and all oracle reports remain hash-bound. Any selected failure is reported rather than tuning the corpus until it passes.

## Decision boundary

This is a coherent, dependency-ready **prospective CV3-A development admission**, conditional on the resource/oracle/source freezes above. It is preferable to making the first nonlinear-field test simultaneously depend on curved-boundary representation,32/64/128 dense feasibility and the literal paper solution. It leaves that harder target visible. No experiment has run, no result has passed, no generic PDE/stability/production gate is closed, and no user/model asset or training is needed for these synthetic fixtures.

Source links (exact reviewed head): [CV1 decision](https://github.com/MrScripty/Rheon/blob/902f6092de6044419d028f92dabd8e59ae7a87ba/docs/plans/fluid-accuracy-composition/q2-cut-method/decision.md), [CV1 mathematics](https://github.com/MrScripty/Rheon/blob/902f6092de6044419d028f92dabd8e59ae7a87ba/docs/plans/fluid-accuracy-composition/q2-cut-method/mathematics.md), [CV1 protocol](https://github.com/MrScripty/Rheon/blob/902f6092de6044419d028f92dabd8e59ae7a87ba/docs/plans/fluid-accuracy-composition/q2-cut-method/protocol.md), [basis bounds](https://github.com/MrScripty/Rheon/blob/902f6092de6044419d028f92dabd8e59ae7a87ba/examples/grid_cut_variational_reference/basis.rs), [physical trace rule](https://github.com/MrScripty/Rheon/blob/902f6092de6044419d028f92dabd8e59ae7a87ba/examples/grid_cut_variational_reference/trace.rs), [CV2 method](https://github.com/MrScripty/Rheon/blob/902f6092de6044419d028f92dabd8e59ae7a87ba/docs/plans/fluid-accuracy-composition/q2-cut-solve/method.md), [CV2 protocol](https://github.com/MrScripty/Rheon/blob/902f6092de6044419d028f92dabd8e59ae7a87ba/docs/plans/fluid-accuracy-composition/q2-cut-solve/protocol.md), [CV2 qualification](https://github.com/MrScripty/Rheon/blob/902f6092de6044419d028f92dabd8e59ae7a87ba/docs/plans/fluid-accuracy-composition/q2-cut-solve/qualification.md). Canonical criteria were read at main `718469beedb9a949f4bb6f3299c7b880b4c47e47`, [verification lines61–72](https://github.com/MrScripty/Rheon/blob/718469beedb9a949f4bb6f3299c7b880b4c47e47/docs/plans/fluid-accuracy-composition/reports/verification.md#grid-stokes-transport-and-traction).

Read-only review; only this external recommendation written. No source/experiment/build/campaign/ref/PR mutation or duplicate implementation. Neither mathematical reasoning nor digest custody is authentication, reproducibility or physical acceptance.
