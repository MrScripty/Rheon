# Proposed coupled implicit-donor temporal work experiment

This new research route preserves quadratic source `b2b85ee0226bf2f1ef61313b610a049bc587fb21`
and evidence `c6c88365c1f52ed9dc0644e2ed48e64b3e24d567`, including all rejected
cells. It changes the finite momentum discretization explicitly. It does not
retroactively validate their exact physical momentum-flux integral, change any
public step, raise a tolerance, or add a native/Lean claim.

## Two candidate routes and smallest selected experiment

A genuine space-time variational route would test conservative momentum with
constant component tests **and** the varying velocity test needed for work.
The old quadratic cell with six accelerations and sixteen constant pressure
coefficients supplies only 22 unknowns for 22 constant-test equations. An
additional independent work condition therefore needs richer temporal trial/test
spaces and a compatible pressure/divergence pairing; appending one scalar gate
alone gives no solvability or consistency theorem. A discrete-gradient route
likewise must account for changing mass in the kinetic-energy chain rule,
material-cap constraints and pressure's zero work on the relevant time test.
A scalar energy correction does not automatically preserve component momentum
or the actual material geometry. These are viable research routes, but neither
has been derived or implemented for this fitted moving space.

The smaller experiment reuses the qualified changing-mass implicit donor and
backward-Euler identities in the fitted-height formulation. Retain the same
reviewed six-coordinate chart and quadratic material/divergence-compatible path,
and integrate the **physical face MASS rates** on that path. Route those face
masses with endpoint donor velocities as an explicitly first-order finite
**MOMENTUM** discretization. Use endpoint full positive strain and its pressure
adjoint. This is not the integral of the varying donor momentum along the path.
The extra dissipation is separately named and measured, never called viscosity.

## Proposed finite system before implementation

Use the frozen two-column fixture, constant density 3, viscosity 1/20 and thickness
1, fixed impermeable free-slip bottom and natural weak cap traction. All 22 xy
coefficients are retained; the invariant third field is zero. With reviewed lift
`z=J(q) eta`, initial `(q0,eta0)`, six unknown accelerations `alpha`, and sixteen
endpoint pressure coordinates `pi`, declare

    eta(s)=eta0+s alpha,
    q(s)=q0+s eta0[0:3]+s² alpha[0:3]/2,
    z(s)=J(q(s)) eta(s),  0<=s<=h.

Rebuild each physical PS point and its true derivative. At every quadrature or
recorded point test all 24 `D z=0` and `D zdot+Ddot z=0` rows and cap material
motion. For each unordered face, oriented i to j, integrate separately

    Fplus_ij = integral max(f_ij(s),0) ds,
    Fminus_ij = integral max(-f_ij(s),0) ds,
    F_ij = Fplus_ij-Fminus_ij,
    f_ij(s) = physical relative fluid/mesh face MASS rate.

No marginal fit is allowed. Require `G_i=m1_i-m0_i+sum_j F_ij` within the
unchanged `128 eps (m0_i+m1_i)` gate. At the endpoint write `U1=R z1` and route

    cBE_i = sum_j [Fplus_ij U1_i - Fminus_ij U1_j].

Solve all 22 reduced conservative momentum equations

    rBE = M_r1 z1-M_r0 z0 + R^T cBE + h K_r1 z1 + h B1^T pi = 0,
    D1 z1 = 0,    B1=-Q1^T W_A1 D1.

The first equation is solved through the path's six `alpha` and sixteen `pi`.
The full divergence and cap chart remain independent checks. Atmospheric-relative
pressure is `Q1 pi`, with its consequential constant mode, no pressure pin.

## Exact discrete work identity and fixed gates

Define lumped energy `T(U,m)=sum_i m_i |U_i|²/2`, and

    D_BE  = sum_i m0_i |U1_i-U0_i|²/2,
    D_mix = sum_{unordered ij} (Fplus_ij+Fminus_ij)|U1_i-U1_j|²/2,
    D_mu  = h z1^T K_r1 z1,
    W_p   = h z1^T B1^T pi,
    W_G   = sum_i G_i |U1_i|²/2,
    W_r   = z1^T rBE.

Pairwise antisymmetry and the changing-mass polarization identity give exactly

    T1-T0 + D_BE + D_mix + D_mu + W_p + W_G - W_r = 0.

For nonnegative face transfers and positive old mass, `D_BE,D_mix>=0`. `D_mu>=0`
uses the full endpoint symmetric strain. Exact endpoint divergence makes `W_p=0`.
The work residual and GCL work remain explicit. Gate both `|W_r|` and the ledger
error at the preceding `128 eps` energy scale including the old/new energies,
BE increment, mixing, strain, pressure, momentum-residual and GCL work magnitudes.
Gate every component of finite momentum and its rate-normalized norm at the
existing `1e-11` scale; retain full strong constraints/material mismatch there.
Do not hide pointwise continuous momentum or exact integrated physical momentum:
measure and report both against their original gates as **different equations**.
A discrete-work pass does not label those ODE residuals zero.

The loss uses `Fplus+Fminus`, not `abs(Fplus-Fminus)`. Sign reversals can transport
both ways; net-zero transport can still mix. Corruption tests must reject
omitted BE/mixing losses, false claims about physical momentum, wrong pressure
adjoint, independently edited masses and zero-balance face circulation.

## Consistency and qualification plan

Conditional on a smooth nonsingular chart and smooth spatial operators,
`z1=z0+h zdot0+O(h²)`, `m1=m0+h mdot0+O(h²)`, and
`Fplus/Fminus=h max(plus/minus f0,0)+O(h²)` (Lipschitz also at sign changes).
Endpoint donor momentum differs from its actual path integral by `O(h²)`;
endpoint strain and pressure forces have the same first-order quadrature error.
The finite equations divided by h thus approach the previously qualified
semidiscrete coupled momentum equation. The quadratic cap position is compatible
with the same limiting material velocity. This is a conditional local
consistency argument, not a nonlinear existence, stability or global accuracy
proof. `D_BE` is an explicit temporal dissipation of order h² per smooth step;
`D_mix/h` approaches the semidiscrete donor loss.

First qualify a single cell on the old initial shear and nonzero-pressure field,
with physical mass 16/32-point quadrature refinement, full path checks, independent
exact-rational/high-precision work assembly and meaningful corruption controls.
Then qualify bounded repeated host execution and temporal refinement against an
independently refined coupled DAE/ODE reference on the same spatial discretization.
Research cancellation/failure must preserve the complete nonzero accepted state
at every barrier, using one accepted state and one candidate, no accepted-state
snapshot history. Bound nonlinear evaluations, steps, quadrature and recorded
outputs. Host research arrays are not a production memory-accounting proof.

No public step follows until native bounded solver/workspace/owner publication,
actual cancellation/failure, arithmetic guards and repeated accuracy are qualified.
Retain all continuum, pointwise traction, constrained positivity, density,
adhesion/capillarity and surface-reconstruction limitations.

## Bounded numerical quadrature and source context

Fixed 16/32 quadrature over an entire repeated cell failed at a physical face
sign reversal; that trial is retained. The candidate now discovers sign brackets
on sixteen subintervals and performs exactly twenty bisections per crossing,
with at most 64 roots and 4,096 face evaluations. Each resulting subinterval
uses the same 16/32 rules. This is a bounded numerical subdivision strategy,
not an exact sign-isolation theorem. Missed roots or inaccurate subdivisions
still have to pass the unchanged `1e-15` measured refinement and local GCL gates.
Both positive and negative mass transfers are always integrated; no small flux
is clipped. The first relative-iterate root finder failed near zero time; its
log/source is retained. A 52-bisection trial is also retained, with the same
physical gates. Bisection depth is a resource/quad partition choice, not a
physical residual allowance.

The initial SciPy nonlinear iterate-status criterion stalled despite tiny
measured residuals on a repeated cell. A bounded Newton loop now terminates on
the actual rate-normalized residual at `1e-13`, **stricter** than the unchanged
`1e-11` acceptance gate. Each correction uses a bounded 22-column finite
difference Jacobian; at most 200 equation evaluations are allowed. No solver
status replaces work, momentum, path or mass checks.

[Hairer's local-coordinate/index-two notes](https://www.unige.ch/~hairer/poly-sde-mani.pdf)
motivate constraint charts but do not prove this temporal discretization.
The publisher identifies [McLachlan, Quispel and Robidoux's discrete-gradient
paper](https://royalsocietypublishing.org/rsta/article/357/1754/1021/52506/Geometric-integration-using-discrete-gradients);
its full text was not accessible here, so no theorem from it is imported.
The changing-mass work identity and method comparison above are derived locally.
The reference follows [SciPy's documented DOP853 interface](https://docs.scipy.org/doc/scipy/reference/generated/scipy.integrate.solve_ivp.html),
using installed SciPy 1.17.0; web documentation currently describes 1.18.0.
The two reference runs change both tolerances and maximum step and report their
actual end-state difference. Their agreement is numerical evidence, not an ODE
error certificate or continuum solution.

## Observed outcome: finite work passes, finer repeated solve rejected

Six cells (two initial states, intervals `.05,.025,.0125`) pass the actual
rate-normalized 22-row momentum, full moving constraint, physical finite GCL,
endpoint pressure work and unchanged discrete-work gates. At `.05` on the
initial shear, momentum-rate norm is `4.31e-14`, residual work `1.45e-15` and
ledger error `2.18e-16`, below allowance `4.5653e-14`. BE increment, donor mixing
and physical strain losses remain distinct. Independent rational reconstruction
checks the same changing-mass identity, not the old integrated physical momentum
claim. That old equation still fails here: its `.05` residual norm is about
`1.14149e-4`. Its pointwise continuous momentum gate also remains false.

Tangent acceleration/pressure error ratios are `2.087,2.043` on initial shear and
`2.080,2.040` on the nonzero-pressure field. Endpoint versus true donor momentum
convection differences refine at `8.118,8.059` for initial shear (vanishing initial
relative flux) and `4.629,4.335` on the pressure field. The latter coarse ratio
fails the experiment's originally assumed 15-percent band around the leading
factor four. The failed assertion log/source and a **false** order-diagnostic
flag are retained. This asymptotic-rate diagnostic is distinct from the physical
momentum/work acceptance equations; no failed diagnostic is relabeled passing.

Repeated host cases to `.1` complete at `.05,.025,.0125`, with 2,4,8 steps and
lumped velocity errors `3.44682e-5,1.74329e-5,8.76669e-6` against the refined
coupled ODE reference. Geometry errors are `1.58577e-6,7.68853e-7,3.77818e-7`.
These three cases pass every recorded work, mass and finite momentum gate and
show first-order behavior on this **same spatial model**. The `.00625` case
fails the declared bounded Newton `1e-13` convergence check. The existing
`1e-11` physical gate and `128 eps` work/mass scales are not loosened. Its failed
log, empty unfinished output and partial completed-case records are preserved.
Four-case repeated accuracy qualification is therefore **rejected/incomplete**.
The complete-trajectory replay helper is not executed without a completed payload.

A separate actual rollback run cancels and injects failure at each of four
barriers after a nonzero accepted host step, checks every accepted geometry,
velocity coordinate, pressure coordinate, clock and stamp unchanged, rejects
four invalid intervals, and successfully continues that same owner. This is
research transaction evidence; native owner/memory/rollback is still pending.
The replay GIF depicts recorded single-cell **candidate** snapshots, not a
new native advancing simulation. The plot includes the three completed cases
and explicitly marks the fourth rejection.

## Next research work, saved without implementation

Investigate the finer-cell Newton floor while retaining its convergence check:
first compare cancellation in `M1 z1-M0 z0` with the algebraically identical
nodal form `R^T[M1 R(z1-z0)+(M1-M0)R z0]`, and independently recheck true residual
and work. That numerical repair is **not implemented or qualified here**.
Then repeat the failed fourth case, preserve complete step output incrementally,
and qualify complete repeated replay, additional references and real-path
rank/sign bounds. Only after that consider bounded native workspaces and the
existing production owner/publication protocol. Native arithmetic/scale guards,
publication rollback, memory accounting and continuum/traction accuracy remain
separate obligations. A genuine space-time/discrete-gradient alternative remains
research, with no demonstrated fitted-space solvability or energy theorem.

The parent requested a clean pushed checkpoint and stop under the strict usage
ceiling. This packet freezes the current finite-work success and repeated-solve
rejection; it makes no public simulation change and starts no further experiment.
