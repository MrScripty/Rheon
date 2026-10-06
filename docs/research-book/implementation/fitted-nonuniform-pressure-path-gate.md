# Nonuniform xy pressure solve and the material-path gate

This checkpoint continues the fitted-domain plan after native source
`23f93f9b72815766680e07a76ce2a6b23f8d7fce` and evidence
`6d57d1af5ca377f63cfeb90f8af4e71e11f6c280`. Both remain frozen. The next
question is whether a nonuniform xy pressure/strain solve also determines an
incompressible material-cap trajectory and conservative finite transfers. A
pressure solve on one geometry is insufficient to answer that question.

## Proposed smallest experiment and acceptance test, before implementation

Use the smallest supported periodic fitted graph: two columns, period one,
fixed bottom `(0,0),(1/2,0),(1,0)` and cap
`(0,1),(1/2,5/4),(1,1)`. Reuse the corrected periodic Powell–Sabin construction,
physical lumped mass, trace restrictions and full symmetric strain of the
existing formulation. Set density `3`, viscosity `1/20`, thickness `1`, time
interval `1/20`, initial velocity `(y,0,0)` and external load zero. This
initial shear is exactly divergence free and bottom impermeable. It need not
satisfy the natural viscous traction pointwise at the initial instant; the
variational force solve imposes the natural boundary condition weakly. There
is no pointwise traction or continuum accuracy claim.

Let `D` be the actual microtriangle divergence composed with admissible velocity
embedding `R`, `Q` a basis for its image, `W_A` the microtriangle areas,
`B=-Qᵀ W_A D`, `M=Rᵀ diag(m_i I_3) R`, and `K=Rᵀ Eᵀ W_mu E R`.
The first solve is the full xy saddle system

    (M + dt K) z + dt Bᵀ p = M z_old,
    B z = 0.

Pressure is solved, including the constant pressure mode admitted by the free
cap. No pressure gauge is removed. The zero third component remains zero
because its strain block decouples; all potentially nonzero xy components are
included. This is a force test on a declared geometry, not a transported step.

The mathematical gate for an advancing successor is one declared trajectory
`X(s), U(s), p(s)`, `0<=s<=dt`, satisfying all of:

* the actual cap-node material condition `Xdot_cap=U_cap` and fixed bottom;
* strong microtriangle divergence `D(X(s)) z(s)=0`, with `U(s)=R z(s)`,
  along that same path;
* physical shared transfers `F_ij=integral f_ij(X,U,Xdot) ds`, with separate
  positive/negative integrals when direction changes, and
  `m_i(new)-m_i(old)+sum_j F_ij=0`;
* all component momentum equations on the declared temporal quadrature, the
  pressure adjoint, full strain and measured transport/work residuals;
* positive geometry/masses, the existing arithmetic and rank/residual gates,
  bounded owner/workspace memory, and common-state publication after a final
  cancellation barrier. Failure/cancellation must preserve both accepted states.

Acceptance of a native general step requires these integrated gates, repeated
execution, temporal refinement, independent replay and rollback from nonzero
accepted states. Passing an endpoint saddle solve, a volume sum, or a fitted
flux marginal alone does not enable that step. No tolerance is relaxed.

The first deliberately falsifiable proposal is the endpoint force solve followed
by straight cap motion with its solved nodal velocity. Rebuild the physical PS
mesh at every path point and embed the same reduced coefficients there. The
cap then is material by construction. Recompute strong divergence and actual
dual relative flux using the differentiated rebuilt geometry. If these fail,
preserve an exact rational counterexample and derive the missing nonlinear
space-time equations; do not relabel the static force solve as advancement.

## Result: the force solve passes, its straight material path is refused

The exact-rational solve has 22 xy unknowns, 16 pressure modes, 19 geometric
nodes, 16 periodic masses and 24 microtriangles. Momentum and strong divergence
residuals, pressure work and the complete symmetric-strain backward-Euler work
identity are exactly zero. Reconstructed pressure reaches approximately
`0.0101967270162`; it is an unknown solved from zero load and the initial shear.
Vertical velocities become nonzero. Removing pressure or vertical momentum fails
the equations. Horizontal total momentum is exactly conserved. No conservation
of vertical momentum against the bottom reaction is asserted.

Now set

    X_cap(s)=X_cap(0)+s (R z)_cap,  X_bottom(s)=X_bottom(0),
    U(s)=R z,  0<=s<=1/20,

and rebuild the corrected PS geometry, including its true split-node derivative,
at each path point. The cap is exactly material, all checked areas/masses are
positive and total liquid mass is exactly `27/8` at the five rational snapshots.
Yet microtriangle 6, on geometric nodes `(0,12,7)`, satisfies

    D_6(X(0)) z = 0,
    d/ds [D_6(X(s)) z] at s=0 ≈ −0.00203121758941077 ≠ 0.

The full exact fraction is in `research-qualification/normal-geometry.json`.
This is an exact nonzero derivative, not a fitted finite difference. It proves
failure on sufficiently small positive intervals by continuity of the rational
geometry formulas at the positive initial mesh. At `s=1/20`, maximum actual
divergence is about `1.02675160113e−4` and maximum local continuity defect is
`1.62767785196e−5`. The global sum of that defect remains exactly zero.
The physical Reynolds identity passes **including** the measured divergence
term; setting that term to zero is what fails.

For this trace-constrained material cap, the exact nodal identity is

    m'_i + sum_j f_ij = (rho b / 3) sum_{T containing i} A_T div_T U.

At `s=0` its right side vanishes, but its exact derivative has nonzero entries.
Consequently the finite local GCL defect has leading term

    m_i(h)−m_i(0)+integral_0^h sum_j f_ij ds
      = h²/2 (rho b / 3) sum_{T containing i} A_T(0) Ddot_T(0) z
        + O(h³).

The exact nodal coefficients are retained in the geometry receipt. Independent
16/32-point quadrature of the actual physical relative faces measures maximum
finite defects `4.06927623524e−7`, `1.01733944226e−7` and
`2.54337405270e−8` for intervals `.05,.025,.0125`. Refinement ratios near four
are the quadratic **failure** of local GCL, not convergence of an accepted
liquid simulation. Face quadrature differences are below `1e−15`; they do not
justify a universal quadrature bound. No flux is adjusted to force its marginals.

A native bounded diagnostic solves the same 38 by 38 saddle system from existing
public `FittedHeightWorkspace` operators, then independently rebuilds/inspects
these five candidate meshes. It publishes no advancing state. Its frame charges
23,584 heap bytes; assembly and solver use fixed stack arrays of 26,304 and 24,016
bytes. Input/output/render allocations are diagnostic host allocations, outside
the simulation memory ledger. There are no accepted-state snapshots or new
simulation authority. The production source and existing tests remain byte
identical to the frozen ALE milestone. An initial unrefined native solve missed
the `1e−11` pressure-coordinate comparison. The failed payload and traceback
remain in `trials`; fused multiply-add elimination reduces the error below the
same bound. A separately retained decimal/float path-bound failure was repaired
by representing the requested decimal interval as an exact fraction, without
extending the accepted interval.

## Required next equations and concrete blocker

This counterexample rejects the stated force-then-straight-cap proposal. It does
not prove that the fitted space cannot support general incompressible motion,
or that every temporal integrator must use this proposal. Moving the force solve
to the candidate endpoint alone still does not supply intermediate divergence
or physical face integrals; it needs a coupled temporal derivation and its gates.

For a differentiable exact constrained trajectory with constant trace embedding
`R` and changing geometry, differentiating `D(X) z=0` gives

    D(X) zdot = −Ddot(X; Xdot) z,
    Xdot_cap=(R z)_cap,  Xdot_bottom=0.

The current static saddle solve enforces the first equation only with its
geometry frozen; the counterexample leaves `Ddot z` nonzero. The conservative
semidiscrete formulation instead requires

    M_r(X) zdot + B_r(X)ᵀ p
      = −Rᵀ diag(mdot(X)) R z −Rᵀ c(R z,f(X,z)) −K_r(X) z,
    B_r(X) zdot = Q(X)ᵀ W_A(X) Ddot(X; Xdot) z.

The second row is the pressure-image projection of the differentiated strong
divergence equation, using `B_r=−Qᵀ W_A D`. It is sufficient only while the
chosen image has the required rank and the full strong residual is independently
checked. It introduces pressure and mesh dependence into the actual time
integration, alongside the same physical momentum transport and changing masses.
A finite successor may use another consistent temporal formulation, but must
qualify its full cap/divergence/flux/work equations together. No such nonlinear
finite integrator or native solver has been selected or qualified here. That
space-time coupling is the concrete blocker to enabling the general public step.

The render and replay depict a **rejected candidate path**, not accepted motion.
No new Lean theorem, native general pressure stepping mode, continuum spatial
accuracy, pointwise free-surface traction, positivity, density variation,
adhesion, capillarity or surface reconstruction claim is made. Original Jacobi
fixtures, frozen arithmetic rejections and all previous numerical/proof failures
remain intact. The restricted ALE API and the general public refusals are unchanged.
