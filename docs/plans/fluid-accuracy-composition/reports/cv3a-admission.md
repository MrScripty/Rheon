# CV3-A admission: static polygon manufactured-solution pilot

**Admitted next numerical milestone; no evaluation or acceptance yet.** The
parent's 2026-10-20 continuation asks the lead to select the next grid-first slice
after bounded CV2. Resume the existing numerical owner; do not create a duplicate
solver worker. First commit the discovered source/write set, resource plan,
prospective protocol and independent oracle/review, then perform one bounded
private-reference campaign. This admission changes no production capability.

Dependency: final [PR139](https://github.com/MrScripty/Rheon/pull/139), head
`902f6092de6044419d028f92dabd8e59ae7a87ba`, numerical source
`b0a9d52cc266cb651c5e67c07ec5f4f82419db55`. Its
[sealed custody manifest](https://github.com/MrScripty/Rheon/blob/902f6092de6044419d028f92dabd8e59ae7a87ba/docs/plans/fluid-accuracy-composition/q2-cut-solve/evidence-manifest.md)
retains all 24 static cases, independent references, six unscaled square
velocity-accuracy failures and unchanged original mass conditioning. CV2 resolves
that fixed solve question; it establishes neither approximation order nor generic
cut stability. Its actual source and evidence remain immutable.

## Decision and alternatives

| Route | Disposition and reason |
| --- | --- |
| Repeat CV2 conditioning fixtures | Retain existing results; another same-corpus solve does not test approximation of a field outside the discrete spaces. |
| Full published free-surface disk now | Retain as the canonical later gate. Exact curved geometry, nonpolynomial forcing/integration and feasible 32/64/128 solution/reference machinery need separate preparation. A fixed polygon must not be called the zero-traction circular problem. |
| Static, exact-source polygon MMS with nonrepresentable fields | **Selected CV3-A.** It isolates field/pressure approximation and physical trace errors using existing static geometry and solve owners before expanding geometry or production state. |
| Add guessed stabilization, delete small supports or change pressure rows | Not admitted. Such changes require a distinct method decision and unchanged physical gates; passing residuals or a few positive spectra is insufficient. |

The [Larionov–Batty–Bridson supplement, section 1 and equations 5–15](https://cs.uwaterloo.ca/~c2batty/papers/Larionov2017/supplemental.pdf)
uses analytic velocity/pressure and total traction to test coupled Stokes. Its
free-surface disk has nonzero interface pressure and zero **total** traction.
Here the exact polygon receives its own analytic prescribed traction; this is
not that circular free-surface test. [Burman–Claus–Massing's abstract](https://arxiv.org/abs/1408.5165v2)
attributes cut-independent stability to their particular stabilized formulation.
It supplies no theorem for Rheon's unstabilized staggered-Q1/P0 reference.

## Fixed pilot problem

Use the exact represented-source rectangle and square-17 polygons from CV1,
preserving their physical coordinates, source bytes, orientation and extrusion.
Set rho=1 kg/m³, mu=the represented binary64(.1) Pa s, dt=1 s,
extrusion=1 m, L_ref=1 m and U_ref=1 m/s. Export exact primitive bit identities;
R2 reconstructs those represented primitives before arithmetic rounding. Derive
coefficients symbolically from them rather than substituting rational 1/10 or
independently rounded decimal coefficients. R1 solves actual exported rounded data.
Define X=x/L_ref, Y=y/L_ref and
tau_ref=max(mu*U_ref/L_ref, rho*U_ref*L_ref/dt)=1 Pa. The chosen continuum fields
are deliberately outside the staggered bilinear/P0 spaces:

```text
psi = U_ref * L_ref * (X^2*Y + X*Y^2)
u   = (d_y psi, -d_x psi)
p   = tau_ref * (1 + X + 2*Y + X*Y)
sigma = -p I + mu (grad u + grad u^T)
u_star = 0
f = (rho/dt) u - div sigma
g = sigma n_outward
```

Independently derive and dimension-check f, g and div(u)=0 before source freeze.
Integrate these **continuum** body/traction loads. Do not construct a manufactured
discrete RHS from A*u_h and B-transpose*p_h: that would test the solver again.
Include both body and boundary loads in momentum, torque and work/resultant
checks, with each contribution reported separately. The forced problem does not
inherit an unforced nonincreasing-energy inequality. Pressure remains absolute
under the prescribed natural-traction problem; do not
remove a constant pressure row or introduce a gauge by analogy with closed walls.
This is a single time-discrete static verification problem, not temporal-order
or evolving-interface acceptance.

Freeze nine scenarios before execution: rectangle n=4/8/16 at its existing grid
origin, and square-17 n=4/8/16 at that origin and a grid-origin shift
(.25h,.25h). Source polygons stay fixed in world coordinates. Freeze bounding-box
coverage and all staggered knots/support rules explicitly. Each scenario identity
includes source geometry hash, grid/phase, units/material/dt, field/load version,
basis/reconstruction version and exact command. No phase, case or finest level
may be dropped after seeing outputs.

## Bounded implementation and ownership

Use a fresh branch on the exact PR139 dependency, suggested name
`implementation/q2-static-mms-pilot-20261020`. The numerical owner owns only the
private experiment, test adapter, oracle and new q2-static-mms protocol/results
directory. Reuse CV1 M/B/K/physical basis and the CV2 selected solver. No Session,
PressureProvider, worker, SQLite, desktop, source-admission, Cargo or production
owner change is admitted. Composition work remains independently owned.

Two explicit private-reference prerequisites are admitted for prospective review:

- CV1 Layout currently refuses n outside 2..=8. Extend only its declared private
  ceiling to 16, with allocation/overflow and above-16 refusal controls. Preserve
  formulas, support admission, geometry semantics and all old-case behavior.
  Record the exact helper/test write set before editing; do not silently coarsen
  the pilot to avoid the ceiling.
- The old constant-stress trace rule uses two Gauss points. Quadratic pressure
  times a bilinear basis trace can have degree four on an oblique segment.
  Freeze a separately reviewed degree-at-least-four-exact load/traction rule
  (for example three-point Gauss after **all** staggered-knot and P0 splits),
  checked against independent exact polynomial moments. Verify volume-load and
  squared-error degree requirements too. This is a new reference integration
  contract, not retroactive qualification of old quadrature for new loads.

Any broader helper, discretization, boundary policy, sparse backend, stabilization
or support/pressure elimination need requires a recorded bounded replan before
expansion. Preserve CV1/CV2 frozen sources, campaign receipts and negative data;
old scientific campaigns are not repeated or relabelled as CV3 evidence.

## Deciding verification and resource gate

Keep the canonical [grid verification criteria](verification.md#grid-stokes-transport-and-traction)
unchanged. The pilot uses the same 5% finest relative velocity/pressure and
characteristic-stress-scaled physical traction budgets, and L1/L2 observed order
at least .9 on the declared finest refinement pairs. Freeze component/vector
norms, pressure weighting, cross-phase comparisons and pairwise order formulas
before evaluation. Report L-infinity without assuming second order at cut faces.
Use actual field reconstruction on the exact polygons, not coefficient errors,
cell counts or nominal full-cell weights as substitutes.

Retain original-coordinate normalized momentum/continuity and work/residual-work
budgets at 1e-10. Distinguish algebraic weak boundary satisfaction from independent
physical traction error; the former cannot establish the latter. Prescribed g
must be subtracted in the physical traction comparison. The separate canonical
zero-traction enforcement target remains 1e-8 of the same positive stress scale.

The independent reference must integrate continuum polynomial body/traction
loads and reconstructed L2 **squared** errors independently of native assembly.
L1 absolute norms are not globally polynomial; freeze separate bounded integration
and propagate its uncertainty. Independent exported-target solution checks must
quantify original-coordinate algebraic uncertainty below a predeclared numerical
error budget, propagated into every deciding physical norm and refinement
difference. Require at most 1% of each reported discretization error and
refinement difference; apply 5%/.9 decisions conservatively to the resulting
error intervals. Agreement of 80/120-digit references alone is a numerical estimate,
not a certified forward-error bound. Before launch, freeze either a validated
solve/residual-plus-verified-inverse enclosure or an explicitly estimated budget;
an estimated budget cannot satisfy a claimed guaranteed-bound requirement. A
resource or uncertainty failure stops the campaign; a small native residual alone cannot do this, as CV2's retained
unscaled failures demonstrate. Exact-integral reference and rounded exported
target errors stay separate. Unresolved reference/quadrature uncertainty cannot
be labelled pass or used to infer order.

Record exact B rank, pressure modes and a dimensionally declared velocity-H1 /
pressure-L2 scaled spectral diagnostic across grids/phases, including the fixed
alternating-cell pressure pattern. Keep physical constant-pressure semantics.
Freeze the full consistent velocity H1 Riesz matrix
Gv=integral(grad(phi_i):grad(phi_j)+L_ref^-2 phi_i*phi_j), pressure L2 mass
Qp=diag(positive cell volumes), and the spectrum of Qp^-1/2 B Gv^-1 B^T Qp^-1/2.
Keep every pressure row, including the constant mode. Freeze scaling, exact
rank algorithm, reference precision and spectral uncertainty first. CV2 A/Schur
equilibration spectra are different diagnostics and do not replace these norms. These are finite-instance diagnostics with no invented
beta cutoff, generic inf-sup theorem or automatic stabilization policy.

Before any selected solve, derive matrix sizes, dense storage/factor/reference
costs and available resources for **all nine** cases. At n16, conservative
full-grid bounds are 544 velocity coefficients, 256 pressure rows and 800 KKT
rows; exact rank/high-precision cost can dominate binary64 matrix storage. Freeze hard RAM, scratch,
CPU-worker and wall-time caps, along with source/oracle hashes and actual commands.
If n16 or an independent reference cannot fit, stop and report the feasibility
blocker for a separately reviewed backend/protocol decision. Do not begin a
partial campaign and call whichever finest level completed the deciding result.
After independent protocol/source review, run one serialized source-bound campaign
into new external output, retain every failure and validate evaluator guards in
normal/-O/-OO modes with malformed and residual-consistent wrong-solution controls.

## Handoff and remaining boundaries

The existing numerical worker may resume discovery/protocol/oracle preparation
under this admission, then implement and evaluate only after the prospective
independent/resource gates above are satisfied. No further user confirmation is
needed for that bounded authorized scope. Root retains shared plan/ledger,
complete-source admission, dependency order, hosted checks and publication.
Fresh source qualification and exact-head hosted gates remain separate from
the numerical experiment and from any public merge.

CV3-A has no result yet. Even a passing pilot would be a static prescribed-traction
polygon envelope only. The full Larionov disk 32/64/128, annulus/no-slip lifting,
water hydrostatics/oblique obstacles, generic cut stability, evolving geometry/
transport/GCL, 3D and production acceptance remain unqualified. The grid-first,
then particle/output-surface, then later Lagrangian roadmap remains unchanged.
