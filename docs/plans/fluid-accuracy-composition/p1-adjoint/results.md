# Static result: exact work is insufficient

Neither diagnostic is admitted as production water or given an acoustic timestep.
The pre-reviewed source/protocol was frozen at
`e457ac12e874029229c381d30db8481caaee5cf6` before tests or measurements. All 16
cases, 768 site records and 48 directional probes completed; no trajectory sweep
was run. Original PR121 source, campaign and failed rest criteria are unchanged.
The predeclared 1000Pa top pressure keeps centered differences wet but gives
about1.06105% exact bottom compression at c32, exceeding the production1%
free-surface envelope. This closed pressurized column is solely a derivative/
consistency diagnostic; it cannot replace or pass the original hydro campaign.

| Necessary property | Gamma offset adjoint | Clipped Voronoi reference |
|---|---:|---:|
| Maximum normalized instantaneous work defect | 7.25e-16 | 8.35e-16 |
| Maximum normalized momentum plus wall-reaction defect | 5.08e-15 | 0 |
| All three finite-difference scales pass fixed derivative criteria | 8/8 cases | 8/8 cases |
| Uniform-pressure max force / pressure, ordered | .459820 | 0 |
| Uniform-pressure max force / pressure, perturbed | .517953 | <=2.28e-14 |
| Uniform bottom load relative error | .349076 ordered, .373035 perturbed | <=2.59e-14 |
| Uniform top load relative error | .291232–.349076 | <=2.59e-14 |
| Perturbed hydro max residual / g, n32 / n64 | 18.4199 / 36.5907 | .494501 / .494464 |
| Perturbed hydro outside R boundary band, n32 / n64 | 4.88246 / 11.2308 | .0628724 / .0628722 |
| Manufactured linear-pressure boundary-band relative error | 18.5302–59.1217 | .5 ordered, .576797 perturbed |
| Alternating pressure has zero force at every nonend site | No such claim | Yes; unresolved pressure-operator defect |

The directional internal-energy, total-volume and density derivative normalized
errors are at most 5.88e-8, 1.96e-7 and 1.58e-7, respectively, over all 48
probes. These meet the frozen 1e-6 numerical derivative criterion. Every evaluated
state retained sorted interior sites, positive volumes and wet EOS density.
Base-state gamma minima: gamma>=.6927116, numerator>=692.2244 kg/m³; fixed offsets range
-49.2960 to +30.6611 kg/m³. These are measured finite-neighborhood bounds, not a
proof of global positivity. No offset was reset during a displacement.

## Causal interpretation

The two independent force constructions agree with the density Jacobian's
adjoint and centered internal-energy differences. Gamma's pressure/EOS work
mismatch is repaired for this declared map. The uniform-pressure force still
fails the fixed 1e-10 gate by billions of times. In those cases the independently
assembled force/p equals the analytic total-volume gradient to <=2.28e-14;
the independent volume directional difference also passes. Matching the initial
volume sum to 1 m is insufficient: its position derivative is nonzero.

Exact work and exact global action/reaction can also conceal wrong individual
wall loads. Gamma's uniform-load errors are about 29–37%. In the continuum
sampling limit its offset-calibrated volume coefficient gives an integral
involving gamma'/gamma, rather than an exact physical wall area. This is a
structural boundary/partition issue, not a timestep issue.

The frozen 'interior' classification means distance>=R, not completely wall-free:
neighbor gamma coefficients can transmit wall influence to distance2R. A clearly
labelled **post-run secondary causal analysis**, without changing any gate,
therefore examines only sites with distance>=2R. Gamma ordered uniform force/p
is <=6.67e-17 there, while the perturbed value is .188278 at both resolutions.
The corresponding acceleration doubles from 30.694 to 61.388 m/s² as dx halves.
Thus the perturbed E0 defect persists without wall coupling.

On ordered sites, the unchanged R=3dx kernel has exact discrete first-gradient
moment

 M=-sum_j dx*s_j*K'(s_j)
  =(42/27)[(2/3)^4*(7/3)+4*(1/3)^4*(11/3)]=728/729.

Its fixed deficit1/729=.0013717421 accounts for the measured >2R ordered hydro
residual .0013717109g / .0013717343g. Refining n at fixed support ratio leaves
this quadrature bias. This exact rational arithmetic identity is not a new Lean
kernel qualification. Multiplying a force kernel by729/728 in isolation would
break the matched density/work derivation and would not repair perturbed E0.

The Voronoi reference removes E0 through an actual volume partition. It also
reproduces manufactured linear *site pressure* in nonend interiors to <=1.61e-14
relative error. Nevertheless, its site positions differ from its cell centroids
on irregular samples. Source-integrated masses yield cell-mean EOS densities;
linear site-pressure consistency does not make those cell-mean hydro pressures
well balanced. Its perturbed interior residual stays about6.287% of gravity as
n doubles, and end residual stays about49.45%. The hydro wall reactions use
nearest-cell mean pressure, giving top-load errors15.20%/7.38% on perturbed
n32/n64. The retained alternating-pressure operator null is another separate
reason to withhold acoustic admission; it is not a full EOS/Hessian spectrum.

## Next defensible gate

Return to representation, shared volume quadrature and physical boundary degrees
of freedom. A next *candidate to derive and review*, not an admitted winner, is a
positive partition psi_i(x)=W_i(x)/sum_j W_j(x), with volumes integrated on ONE
shared quadrature V_i=sum_q w_q psi_i(x_q). With covered positive quadrature and
fixed total weights, sumV=sumw exactly, so its fully differentiated volume force
can satisfy the E0 identity. All denominator and wall-shape derivatives must be
retained. Derive shared first volume/mass moments, choose the representative
site/centroid and gravity/source quadrature, and differentiate that same total
potential; exact total volume alone does not resolve cell-mean hydrostatics.
The required discriminator is then linear-pressure/cell-mean hydro
consistency, per-wall load, translation/momentum aliasing, and the composed
acoustic spectrum before any trajectory. Boundary points outside the particle
convex hull require explicit boundary DOFs or a changed representation for
positive linear reproduction. Do not silently add an RK gradient or regularizer.

This route is motivated by the partition construction in
[Hopkins 2015, section2](https://arxiv.org/abs/1409.7395), but the proposed shared
quadrature Hamiltonian is a new diagnostic, not that paper's MFM solver. Its
practical volume approximation and astrophysical tests do not establish these
water or wall gates. [CRKSPH AppendixB](https://arxiv.org/abs/1605.00725) warns that
conservation alone can leave linear momentum inconsistency; the
[Voronoi paper section5.3](https://arxiv.org/abs/0912.0629) documents its Nyquist
mode. Every alternative must meet the unchanged plan rather than inherit a
source paper's claims. No positive-partition experiment is included in this PR.
