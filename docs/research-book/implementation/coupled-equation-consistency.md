# Governing-equation limit and pressure interpretation

Research source/evidence `ef9c3dc3/598e9a06` and native source/evidence
`3f0271d6/a9428641` are frozen. This additive audit changes no native code,
acceptance allowance, Jacobi fixture or earlier failed counterexample. It checks
which governing equations the selected time discretization approaches. A temporal
self-convergence ratio alone cannot identify the physical equations being solved.

## The target is the declared semidiscrete ALE donor system

Let `z=J(q) eta`, `U=Rz`, `qdot=S eta`, with `S` selecting the three cap speeds.
The physical median-dual masses, relative fluid/PS-mesh face rates, pressure image
and symmetric-gradient strain are those in the fitted-height formulation. Put
`M=R^T diag(m_i I) R`, `c=R^T route(f,U)`, and `B=-Q^T W_area D`. The tangent
system previously derived and independently checked is

    M a + Mdot z + c + K z + B^T p = 0,
    D a + Ddot z = 0,     D z = 0,
    a = J alpha + Jdot eta,     qdot=S eta.

Here `p` is an **algebraic multiplier**, and physical atmospheric-relative
pressure is `Q(q)p`. It is determined from geometry/velocity and the differentiated
constraint. There is no independent pressure evolution law or equation of state.
The consequential constant pressure mode is retained; no gauge is pinned.

For the six-coordinate chart, define the square limiting matrix

    A=[M J, B^T],
    A [alpha0;p0] = -[M Jdot eta0 + Mdot z0+c0+K0 z0].

Under positive masses, a full-rank six-coordinate chart and a full-rank sixteen-mode
pressure image, `A` is nonsingular: `BJ=0`; multiplying a homogeneous system by
`J^T` gives the positive definite `J^T M J`, hence alpha is zero, then full rank
of `B^T` makes p zero. The independent rational assembly solves this 22-row
system and the full 38-row moving-constraint system consistently, on both fields.
It also differentiates the full 38-row system and independently checks the chart
form of its derivative and all 24 strong rows.

This target already contains lumped inertia and dissipative nodal donor transport.
It is a declared spatial discretization, not the exact continuum Navier–Stokes
PDE. Temporal consistency with it does not establish continuum spatial accuracy,
pointwise free-cap traction, mesh-family stability or material calibration.

## Small-step limit of the finite method

For unknowns `(alpha_h,Pi_h)` the method uses

    eta(s)=eta0+s alpha_h,
    q(s)=q0+s S eta0+s^2 S alpha_h/2,
    z(s)=J(q(s))eta(s),
    Fplus/Fminus = integral max(plus/minus f(s),0) ds,
    E_BE = (M z)_1-(M z)_0 + C_BE + h(K1 z1+B1^T Pi_h)=0.

`C_BE` routes actual integrated face masses with **endpoint** donor velocities.
For a bounded real solution branch with smooth nonsingular geometry/chart,

    E_BE/h = M0(J0 alpha_h+Jdot0 eta0)
             + Mdot0 z0+c0+K0 z0+B0^T Pi_h + O(h).

Thus the unique bounded limit is the previously derived tangent solution:
`alpha_h -> alpha0`, `Pi_h -> p0`. This conclusion compares explicit operators
and a nonsingular limiting system; it is not inferred from self-convergence.
The donor absolute value is Lipschitz. At zero initial face flux the leading
one-sided sign is used; higher-order smooth expansions are conditional on that
one-sided branch. Numerical sign discovery is not a sign-isolation theorem.
A real-equation limit does not guarantee that arbitrarily tiny IEEE steps pass
the unchanged Newton/arithmetic gates.

## Independent leading coefficients and the expected order

Write `u0=(alpha0,p0)`, and let `udot0=(alphadot0,pdot0)` be the derivative of
that algebraic tangent solution along its actual DAE trajectory. This derivative
is an observable; it is not a separately evolved pressure equation. On the
quadratic path with fixed `alpha0,p0`, let

    P2 = coefficient of s^2 in M(q(s))z(s),
    cdot0 = right derivative of actual instantaneous donor momentum,
    Fdot0 = derivative of K(q(s))z(s)+B(q(s))^T p0,
    L = R^T route(f0, Udot0)/2.

Exact rational Taylor jets rebuild PS intersections, gradients, masses, full
strain, pressure transpose and shared fluxes independently. They give

    A udot0 = -(2 P2+cdot0+Fdot0),
    A u1 = -(P2+cdot0/2+L+Fdot0),
    (alpha_h,Pi_h)=u0+h u1+O(h^2),
    u1-udot0/2 = -A^-1(L+Fdot0/2).

The independent differentiated full tangent system gives exactly the same
`udot0`. These coefficients predict local velocity error `O(h^2)` and local cap
position error `O(h^3)`; accumulated velocity/geometry errors are observed first
order. They do not constitute a global stability/convergence theorem.

The actual varying-donor momentum integral along the candidate curve with its
constant pressure coefficient has defect

    E_path = -h^2(L+Fdot0/2)+O(h^3),
    E_path/h = O(h).

A nonzero limit of `E_path/h^2` is its ordinary first-order truncation coefficient,
not a nonvanishing governing-equation defect. The correct momentum **rate**
defect is `E_path/h`, which tends to zero. Pointwise momentum residual likewise
is `O(h)` along the numerical path. Its old absolute gate is not relabeled zero.

## Pressure: multiplier, endpoint display and weighted impulse

The constructor stores the instantaneous algebraic tangent pressure. A finite
step solves `Pi_h` afresh; the previous stored pressure is not advanced or used
as a dynamical initial condition. The numerical step publishes `Q1 Pi_h` on the
accepted endpoint frame. This is a first-order algebraic pressure approximation
associated with that finite step, not exact instantaneous tangent pressure.
The force contribution is `h B1^T Pi_h`, an endpoint-weighted pressure **impulse**.
`Pi_h` itself is not an impulse: omitting h or dividing the displayed pressure
by h changes its dimensions and would produce a wrong or divergent limit.

Let `p_alg,end` be the tangent multiplier recomputed from the accepted endpoint,
and `p_avg` the average of tangent coefficients on the true DAE trajectory.
With coefficient coordinates kept explicit,

    Pi_h-p_avg = h(u1_p-pdot0/2)+O(h^2),
    Pi_h-p_alg,end = h(u1_p-pdot0)+O(h^2).

Both leading vectors are independently nonzero on both fixtures. Thus `Pi_h`
is generally neither a literal coefficient average nor the exact endpoint
multiplier, although both value errors vanish to first order. Physical pressure
averages also contain the changing basis Q and cannot be identified with a mean
coefficient vector. The actual pressure impulse has

    h B1^T Pi_h - integral B(q(t))^T p_alg(t) dt
      = h^2[B0^T(u1_p-pdot0/2)+Bdot0^T p0/2]+O(h^3).

Its rate error is again `O(h)`. The independent ODE/quadrature study checks this
coefficient, not merely an error ratio.

A deliberately invalid **pressure-as-evolved-state** diagnostic is retained:

    (Pi_h-p0)/h - pdot0 -> u1_p-pdot0 != 0.

For the first step after instantaneous initialization this derivative comparison
fails even though the algebraic pressure value converges. A first-order pressure
value is insufficient to validate its difference quotient. No pressure-rate
accuracy claim is made. Repeated native pressure values are separately compared
to the independent DAE at common physical time, using a declared labeled-mesh
ALE pullback and reference-area RMS; that is temporal pressure evidence only.

## Retained pressure-state coarse diagnostic and real counterexamples

The earlier pressure-state `C_BE-C_path` ratio `4.629` at `.05/.025` still fails
the original 15-percent band around four. Its exact leading expansion is
`h^2 L+O(h^3)` with nonzero L. The normalized rate difference is `h L+O(h^2)`
and vanishes; the nonzero `delta/h^2` limit is an error coefficient. Initial shear
has exactly zero f0 and L, hence donor difference `O(h^3)` and rate difference
`O(h^2)`. Endpoint-force truncation remains `O(h^2)` even there. Exact coefficients
and finer diagnostics explain the coarse failure without retrospectively passing
it or loosening any gate.

The independently retained **wrong-equation** variants genuinely fail as h tends
to zero: omitting `Mdot z` leaves a pressure-state momentum rate defect with
maximum component `0.00018051016017325`; freezing the constraint to `D a=0`
leaves strong differentiated defect `Ddot z`, maximum `0.00203121758941077`.
Both are invisible on the initial exact-shear tangent, so that fixture alone
cannot validate the moving equations. The selected method includes both terms.
All prior failed pointwise/integrated gates, native zero-seed refusal and
arithmetic/proof limitations remain unchanged. No new Lean theorem is asserted.

## Next bounded physics proposal after this audit

Complete the existing formulation's **nonzero third velocity component** on this
same two-column, constant-density/viscosity, z-invariant extruded graph before
introducing another material variable. The original full formulation retains
all three components and both third-component strain entries; the present native
coupled step qualifies their invariant zero slice only.

In this restricted extrusion, xy transport/geometry/pressure do not depend on w:
`partial_z=0`, pressure has no z force, and full symmetric strain splits into xy
and the two w shears. After the candidate xy solve, use the same physical face
integrals/masses for a bounded 12-coefficient implicit w momentum/strain solve.
The same accepted full nodal-velocity owner, candidate buffers, frame, pressure,
clock/stamp and final barrier must publish all components together. Account the
additional fixed matrix/scratch once; no parallel accepted phase or snapshots.

Qualify nonzero x/y/z transport with third momentum conservation, constant-w
preservation, both w strain components, exact total/separate work, true residual,
all cancellation/failure/retry states, budget exhaustion and actual refinement/
replay/render evidence. Retain constrained-embedding positivity limitations and
all continuum/contact/reconstruction gaps. This is a proposal, not implemented
physics in this checkpoint. Variable density, wetting/adhesion, capillarity and
new surface reconstruction remain later separately derived features.
