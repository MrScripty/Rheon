# Proposed coupled temporal prototype on the same fitted two-column domain

The pressure/path counterexample source `eb7b354b7c20521357ca258728420c56ef96296a`
and evidence `00c502f1fe5be7f9c527ebb14ba5b78e450cb031` remain frozen. This
separate research prototype targets their missing space-time coupling, with the
same two-column periodic graph, density 3, viscosity 1/20, thickness 1, fixed
impermeable free-slip bottom, weak homogeneous cap traction, initial `(y,0,0)`
and zero external load. No public stepping API is changed.

## Proposed system and acceptance gates, before prototype implementation

Let `x` contain four independent cap coordinates, `R` the unchanged admissible
trace embedding, `z` all 22 potentially nonzero xy velocity coefficients, `D(x)`
the full 24 microtriangle divergence rows, `Q(x)` its 16-dimensional image basis,
`B(x)=-Q(x)ᵀ W_A(x) D(x)`, and `M_r=Rᵀ diag(m_i I_3) R`.
Physical PS points and their derivatives are rebuilt from this cap and its
material motion, including the corrected periodic neighbor-center intersections.
The current fixture has zero third momentum and a decoupled third strain block;
this is a full xy experiment, not omission of a component that becomes nonzero.

The coupled semidiscrete equations are

    xdot = C R z,
    M_r(x) zdot + B(x)ᵀ p = −g(x,z),
    D(x) z = 0,

    g = Rᵀ diag(mdot_i I_3) R z + Rᵀ c(Rz,f) + K_r(x) z.

`C` extracts the independent cap xy velocities. `f` is the actual physical dual
face flux of fluid minus differentiated PS mesh motion; `c` is its shared full
vector donor convection. `K_r` is the existing full symmetric strain. Neither
masses nor fluxes are fitted after the solve. Differentiating the constraint gives

    D zdot = −Ddot(x;xdot) z.

At each evaluation solve the pressure saddle system

    [ M_r  Bᵀ ] [ zdot ] = [ −g                    ],
    [ B     0 ] [ p    ]   [ Qᵀ W_A Ddot(x;xdot) z ].

Check all 24 strong differentiated constraint rows as well as the 16 projected
rows. This uses solved pressure, not pressure supplied from a separate force
mesh. Pressure coefficients are coordinates in the changing image basis; the
physical reconstructed pressure and its adjoint force must be checked.

To avoid drift from merely integrating the differentiated constraint, select a
locally nonsingular 16 by 16 block of `D` on the initial graph. With the six
remaining coefficients `eta`, define a geometry-dependent kernel chart `H(x)`:

    z=H(x) eta,  H_free=I,  H_pivot=−D_pivot⁻¹ D_free.

The ten integrated variables are `(x,eta)`. At every evaluated or interpolated
point reconstruct `z` from this chart and gate the **full** `D z` residual. Use
the same material `xdot=C R z` in `Ddot`, mass derivatives and physical fluxes.
The saddle solve determines `zdot`; its free rows determine `etadot`. The chart
chain rule is checked against `zdot`, rather than silently projecting momentum
onto a new velocity. Every point remains conditional on chart/rank/geometry
checks; no mesh-family inf-sup theorem is imported.

The first finite trajectory is a tightly controlled ODE research reference,
with a fixed upper bound on steps/evaluations. An adaptive Runge–Kutta dense
curve is an approximation, not an exactly constrained material trajectory by
itself. Its derivative must be evaluated analytically and tested separately.
Use the **actual derivative of this stored curve**, rather than replacing it by
the RHS, when rebuilding mesh motion and integrating the relative physical
flux. Reconstruct velocity through `H` even on dense points. Check its actual
chain-rule derivative and the pressure-dependent momentum residual there.

Acceptance gates are fixed before any result:

* full strong divergence, differentiated divergence, true xy momentum residual
  and material cap velocity mismatch at or below the existing `1e−11` scale;
* positive geometry/masses and the full 16-mode pressure image at every sampled
  stage/dense/quadrature point; measured conditioning reported, no uniform bound;
* actual positive/negative face integrals from the stored curved trajectory,
  with 16/32-point refinement measured per accepted dense interval and local GCL
  checked at the preceding native `128 eps (m_old,i+m_new,i)` scale;
* horizontal momentum and the complete integrated energy ledger, using shared
  vector donor loss, full strain, pressure work, measured actual-curve momentum
  residual and the measured GCL-defect work; no unexplained energy drop;
* two independently refined coupled trajectories, complete output replay and
  meaningful corruption controls, preserving any failed trial without changing
  these acceptance tolerances.

The reference and its bounded output arrays are research host allocations. It
adds no owner, clock, accepted state or simulation memory ledger. Public stepping
would still require a bounded native temporal solver, reusable workspaces,
rollback/cancellation from nonzero accepted states, repeated execution and native
qualification. A Python research pass cannot enable that API.

## Independent review and selected first finite-cell experiment

The independent review reproduced exact tangent compatibility on both the initial
shear and the preceding nonzero-pressure field. The local model here independently
reproduces reconstructed pressures `0.0103045276861` and `0.0101166010162`,
respectively, with exact strong acceleration/momentum/work/GCL and horizontal
momentum-rate residuals zero. Thus the frozen straight-path failure is a temporal
obstruction, not a spatial impossibility.

Following the review's more specific finite-cell construction, reduce cap geometry
to `q=(x0,x1,h0)`, with `h1=9/4−h0` and periodic endpoint `(x0+1,h0)`. Its area
is exactly `9/8`. Use divergence rows
`[0,2,3,4,5,6,8,10,11,12,14,16,17,18,20,22]`, cap velocities
`(ux0,ux1,uy0)`, and selectors `(z[0],z[1],z[4])` to define a six-coordinate
lift `z=J(q) eta`. Its first three coordinates are `qdot`; the other cap vertical
velocity is `−uy0`. The remaining three coordinates describe interior motion.

The selected bounded candidate cell is

    eta(s)=eta0+s alpha,
    q(s)=q0+s eta0[0:3]+(s²/2) alpha[0:3],
    z(s)=J(q(s)) eta(s),  0<=s<=h.

Cap motion is therefore **quadratic**, and cap velocity follows the actual path
derivative by construction. Rebuild all PS split-node derivatives from that
motion. Solve for six `alpha` and sixteen pressure coefficients `pi` using all
22 integrated conservative momentum equations,

    M_r(q(h)) z(h)−M_r(q(0)) z(0)
      + integral_0^h [c_r(s)+K_r(s) z(s)+B(q(s))ᵀ pi] ds = 0.

Physical pressure is `Q(q(s)) pi`, with no constant-pressure gauge removal.
Integrate the actual vector momentum flux at each quadrature point; endpoint
velocity times aggregate mass transfer is not this equation. No extra cap
projection equations are appended to the momentum system.

The original ODE research proposal above remains a possible accuracy reference;
this finite cell is a different temporal discretization. Its integrated momentum
residual uses the same fixed `1e−11` gate. Pointwise momentum residuals are measured
and retain their original gate result; an integrated pass is not relabeled as a
pointwise ODE pass. Most critically, even a zero integrated vector residual does
not imply a zero work defect. With

    r(s)=(M_r z)' + c_r + K_r z + Bᵀ pi,

measure `integral z(s)ᵀ r(s) ds` independently. Require its magnitude and the
complete unexplained energy-ledger error at the preceding `128 eps` energy scale.
If that work gate fails, this candidate is rejected; no tolerance changes and
no public advancement follow from its divergence or momentum successes. This
separate work gate is part of the independent review's construction, not an
energy theorem supplied by the coordinate chart.

Local-coordinate DAE methods motivate constraint-preserving representations, but
their smoothness/conditioning assumptions require separate checks here.
[Ascher and Petzold, Sections 2.1–2.3](https://www.cs.ubc.ca/sites/default/files/tr/1991/TR-91-09.pdf)
derive a constrained underlying ODE and discuss projection to retain constraints.
The finite chart and momentum/work equations above are our fixture derivation;
that paper does not validate them. Likewise the space-time conservation approach
in [Busto, Dumbser and Río-Martín's abstract](https://arxiv.org/abs/2301.09601)
uses a different staggered FV/FE method. Its numerical claims do not transfer to
this fitted velocity/pressure space or donor operator. The adaptive reference,
if used, follows [SciPy's DOP853 documentation](https://docs.scipy.org/doc/scipy/reference/generated/scipy.integrate.DOP853.html);
its algorithmic description supplies no liquid acceptance or energy proof.

## Observed coupled-cell result and work rejection

The reviewed quadratic construction closes the moving constraints and the actual
physical mass flux on the `.05` cell. The nonlinear solve uses 62 evaluations;
all 22 integrated momentum equations have maximum component residual about
`1.51e−16`. Strong velocity divergence, strong acceleration compatibility and
material cap mismatch stay below `1e−11` at all recorded quadrature points.
The finite local GCL defect is about `1.29e−16`, within the preceding native
`128 eps (m_old+m_new)` scale. The 16/32-point flux/work difference is about
`1.06e−17`. These are numerical checks of the published curve, not a universal
quadrature theorem or a native advancing solver.

The complete energy ledger passes only when its actual residual work is included:

    T_new−T_old + integral(D_adv+D_strain+pressure_work) ds
      −integral zᵀr ds + integral [sum_i G_i |U_i|²/2] ds = error,
    G_i=mdot_i+sum_j f_ij.

For `.05`, energy decreases from approximately `0.803142280736` to
`0.800306896842`, yet the residual work is `−2.86083482087e−8`, versus the
fixed allowance `4.56533530687e−14`. The energy-ledger error is approximately
`−6.86e−16`. Thus monotone energy and tiny unweighted momentum residual do not
establish the required work balance. This candidate is rejected. Its pointwise
momentum residual also fails the original `1e−11` gate, rather than being hidden
by its integrated pass.

An independent rational reconstruction solves the 22-row chart at each point,
rebuilds all physical fluxes and symmetric strains, and sums with 80-digit
arithmetic. It gives residual work `−2.86083482105e−8`, independently agreeing
with the energy/loss defect. Its integrated momentum maximum is about
`7.15e−16`; local GCL is about `3.78e−22`. No NumPy momentum/strain/flux arrays
or nonlinear-solver work metrics enter that reconstruction. Repeated cells at
`.025` and `.0125` have work defects `−3.54614759230e−9` and
`−4.41416889514e−10`. Ratios near eight describe the cubic **work failure** of
this rejected time approximation, not temporal convergence of an accepted liquid
step. All three work gates fail with the same allowance.

The reason is visible directly in the equations. A zero `integral r ds` implies

    integral zᵀr ds = integral (z−z_bar)ᵀ r ds,

for any constant `z_bar`, but does not make that weighted integral zero. If a
small-cell residual has leading form `r(s)=(s−h/2)b+O(h²)` and
`z(s)=z0+s a+O(h²)`, then

    integral_0^h zᵀr ds = h³ (aᵀb)/12 + O(h⁴).

This conditional expansion explains the observed failure order without assuming
that this quadratic integrator is stable or that its pressure is a continuum
solution. The missing condition is temporal momentum testing that controls its
velocity-weighted work, while retaining conservative component momentum and
physical cap/face coupling. It cannot be repaired by fitting face masses,
relaxing residual tolerances or calling the extra energy loss viscosity.

## Whole real-path chart and geometry certificate

Eight constant microtriangle row relations are verified symbolically for general
`(x0,x1,h0)` with the declared volume constraint. Four are equal-divergence pairs
on trace-constrained physical boundary splits; four are alternating relations at
PS interior/periodic singular vertices. Their 176 entries against the full
`D(q)` are identically zero. The selected 16 row selectors together with these
eight relations form a nonsingular 24 by 24 row transformation. Hence solving the
selected zero-divergence rows in the chart implies all 24 equations on this
physical construction, rather than silently projecting an incompatible component.
The symbolic area-weighted divergence identity also establishes the fourth cap
velocity `uy1=−uy0`. Independent incompatible-component tests still retain a full
strong-residual gate.

For the published `.05` candidate, a 512-box exact rational interval certificate
covers the entire actual binary-coefficient polynomial path. Every PS intersection
and positive microarea is enclosed, and minimum shape quality stays above the
unchanged `1e−6` gate. The chart and pressure-image matrices are nonsingular on
each box: an approximate inverse is used only as an **exact binary rational
preconditioner**, and `||I−C A(q)||_infinity<1` is proved by exact interval bounds.
Maximum chart/pressure Neumann bounds are approximately `.06172/.55261`.
Microarea and shape-quality lower bounds exceed `.03814/.07506`.

Interval endpoints are rounded outward to 200-bit dyadic rationals using integer
floor/ceiling operations; the final Neumann midpoint/radius bound uses exact
integer arithmetic. It does not trust floating inverse accuracy. The first
256-box resource budget was insufficient and its failed log is retained; the
published host certificate has an explicit 1,024-box bound. Physical tolerances
are unchanged. This is a certificate of this **real rational path**, not IEEE
execution, an arbitrary-geometry or mesh-family bound, or a uniform inf-sup
constant. The two shorter refinement cells have recorded numerical path checks,
not this separate whole-interval certificate.

The production Rust/Cargo/test/example inputs are byte identical to the frozen
pressure-path milestone; its actual Rust matrix and native replay evidence remain
preserved. This packet runs new Python research, not a new Rust temporal build.
No new Lean theorem or native public step is claimed. The remaining blocker is
an energy-qualified temporal formulation, followed by bounded native solver,
workspace/owner integration and rollback/repeated-accuracy qualification. Existing
Jacobi fixtures, scale refusals, negative positivity evidence, pointwise traction,
continuum accuracy, density/adhesion/capillarity/reconstruction limits remain intact.

[Hairer, Sections III.2 and V.3](https://www.unige.ch/~hairer/poly-sde-mani.pdf)
explains local coordinate partitioning and index-two constraint treatment. The
specific chart, physical donor operator and failed integrated work above remain
our independently checked fixture construction, not a theorem from those notes.
