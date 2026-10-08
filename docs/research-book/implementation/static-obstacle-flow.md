# Stationary obstacle pressure and reduced wall shear

The same [immutable static box owner](static-obstacle-geometry.md) now supplies
geometry to two bounded field operators. Both borrow its retained triangle
surface, physical cell volumes and unique shared face openings. Neither creates
another collision mesh or substitutes a binary solid mask. The original geometry
lab remains a construction-only record. The new [field-response lab](../../education/obstacle-flow-lab.html)
plays native pressure and reduced-shear results with independent rational checks.

These operators have **different declared boundary models**. Pressure uses a
sealed stationary container. Shear is a fully developed, tangentially
invariant/periodic extrusion above one stationary obstacle wall. They are not a
composed three-dimensional fluid step. Arbitrary embedded tensor viscosity,
evolving liquid interfaces, moving solids, general material variation and
adhesion/contact-angle evolution remain open.

## Pressure: geometry, incidence, units and inertia

For each internal open face, let \(a\) be its negative-side cell, \(b\) its
positive-side cell, \(A_f>0\) the owner's opening in m², and \(d_f>0\) the
separation in metres of its represented nominal grid pressure centers. The
unknowns are the positive-volume cells only. A pressure point can lie inside the
cut-away part of a cell: this face-area method does not relocate general cut-cell
pressure centers or establish a continuum consistency order there.

With constant density \(\rho>0\) in kg/m³, define

\[
w_f=\frac{A_f}{\rho d_f},\qquad
m_f=\rho A_f d_f,\qquad
(Lp)_i=\sum_{f\sim i}w_f(p_i-p_j).
\]

\(m_f\) is a declared face-area lumped mass. It is **not** the exact clipped
staggered dual-cell mass. The operator is symmetric in its integrated form;
\(p^T Lp=\sum_f w_f(p_b-p_a)^2\ge0\) in exact arithmetic. No minimum volume,
area or distance clamp changes the admitted geometry. Invalid, subnormal,
nonfinite or lost-positive coefficients refuse construction.

The owner's outward flux uses \(+A_fu_f\) at \(a\) and \(-A_fu_f\) at \(b\).
All outer container faces and fully blocked faces must have exactly zero speed.
For an explicit positive step \(\Delta t\) in seconds,

\[
b_i=-\frac{Q_i^*}{\Delta t},\qquad Lp=b,\qquad
u_f=u_f^*-\frac{\Delta t}{\rho d_f}(p_b-p_a).
\]

Pressure has units Pa. \(Q\) has units m³/s; the integrated right-hand side,
operator result and residual have units m³/s². Pressure correction changes the
actual stored binary64 field, rather than merely reporting a residual.

## Components, gauges and acceptance

The retained geometry labels active cells by positive-opening connectivity.
Each sealed component has its own constant-pressure mode and compatibility
condition \(\sum_{i\in C}b_i=0\). The workspace pins the first cell of each
component to zero, including isolated singleton components. Dry cells are not
unknowns and must have zero right-hand side. An all-dry owner is a valid zero
operation. Compatibility is checked independently for every component with a
compensated sum and a declared \(64\epsilon\sum |b_i|\) allowance. Incompatible
data are refused; no global mean subtraction quietly transfers volume between
isolated regions.

Jacobi-preconditioned conjugate gradients operate on the gauge-eliminated
system. Every candidate acceptance recomputes the **full** integrated residual,
including gauge rows. With \(r=b-Lp\), the exact identities have the signs

\[
Q_i^{\rm new}=-\Delta t\,r_i,\qquad
D_i^{\rm new}=-\frac{\Delta t\,r_i}{V_i}.
\]

Both the full residual norm and \(\max_i\Delta t|r_i|/V_i\) must satisfy the
caller's tolerances. After correction, the owner independently differentiates
the actual stored face speeds and checks \(\max_i|Q_i^{\rm new}|/V_i\) again.
A tiny positive cell cannot hide behind a small integrated residual. A contract
with volume \(2^{-30}\) deliberately fails a loose integrated-residual gate
because its local divergence is too large. This is a numerical acceptance
control, not a lower bound on all future cut-cell conditioning.

The face-area kinetic energy is \(E(u)=\tfrac12\sum_fm_fu_f^2\). In exact
arithmetic the correction satisfies

\[
E(u^{\rm new})-E(u^*)+E(u^{\rm new}-u^*)
=\Delta t\sum_i p_i Q_i^{\rm new}.
\]

The right side is numerical residual work. Stationary impermeable walls do no
mechanical work. Rust reports all terms, checks the identity with its declared
binary64 allowance, and permits no unexplained energy increase. This ledger
does not change the declared lumped-mass approximation into an exact geometric
mass theorem.

## Reduced shear: when scalar diffusion really is Newtonian strain

`StaticObstacleShear` admits only a box covering the entire two-dimensional
cross-section and extending from below the container to one lower wall. The
fluid above it is a connected channel. The normal direction may be X, Y or Z;
the laboratory uses Y. The upper wall is stationary no-slip. Tangential
directions are invariant/periodic, so no sealed end-wall response is implied.

For the exact reduced continuum mode \(\mathbf u=(u(y),0,0)\), constant
Newtonian symmetric strain gives shear stress \(\tau=\mu u_y\) and dissipation
\(\mu u_y^2\). Thus this scalar reduction is physically justified **within
that mode**. It is not a license to smooth each component independently near
arbitrary embedded or free-surface boundaries.

Each fluid layer has volume \(V_i\) obtained by summing the owner's cell
volumes, centroid \(y_i\) from its actual clipped interval, and mass
\(M_i=\rho V_i\). Interlayer area \(A_{i+1/2}\) is the sum of the owner's
shared open normal faces. The conductance and equal-opposite fluid forces are

\[
k_{i+1/2}=\frac{\mu A_{i+1/2}}{y_{i+1}-y_i},\qquad
F_{i\leftarrow i+1}=k_{i+1/2}(u_{i+1}-u_i).
\]

For wall-to-centroid distance \(d_w>0\), stationary no-slip gives
\(k_w=\mu A_w/d_w\). A finite Navier friction coefficient
\(\beta\ge0\), in Pa s/m, gives the series fluid/wall closure

\[
k_w=\frac{A_w\mu\beta}{\mu+\beta d_w}.
\]

At \(\beta=0\) there is zero tangential wall traction. `NoSlip` is its own
boundary mode, not an enormous finite beta. The effective boundary dissipation
combines the fluid half-layer loss and wall friction. Increasing beta changes
friction; it does not introduce adhesion, wall/interface energy or wetting.

## Forcing, one update and its work ledger

Dynamic viscosity \(\mu\) is Pa s; kinematic viscosity is \(\mu/\rho\) in
m²/s. Force input is explicit: `Density(q)` means N/m³, while
`Acceleration(a)` means m/s² and converts to \(q=\rho a\). With the symmetric
traction matrix \(K\), including stationary-wall diagonal terms, the update is

\[
(M+\Delta tK)v=Mu+\Delta t\,qV.
\]

A tridiagonal direct solve performs one backward-Euler step. There are no hidden
substeps. The exact discrete energy and momentum identities are

\[
E(v)-E(u)+E(v-u)+\Delta t\,v^TKv
=\Delta t\sum_iqV_iv_i,
\]
\[
\sum_i M_i(v_i-u_i)
=\Delta t\sum_iqV_i-\Delta t(k_{\rm lower}v_0+k_{\rm upper}v_{n-1}).
\]

The source checks both identities and the true momentum equation residual
before publishing the field. Unforced kinetic energy cannot increase beyond the
declared arithmetic allowance. This is a discrete forced/dissipative response,
not a transient continuum-reference or temporal-convergence qualification.

A one-layer hand control has area 1 m², fluid thickness 0.5 m, density 2 kg/m³,
viscosity 1 Pa s, and stationary no-slip walls. Its mass is 1 kg and total
conductance is 8 kg/s. At \(\Delta t=0.25\) s, \(q=2\) N/m³, starting from
1 m/s, the exact discrete answer is
\(v=(1+0.25)/(1+0.25\cdot8)=5/12\) m/s. Changing the lower closure to finite
Navier friction or free slip changes the actual velocity, force and energy
ledgers.

## Native records and independent controls

The [exporter](../../../examples/obstacle_flow.rs) records four pressure cases:
partial-box gradient removal, an outside-box control, a disconnected separator,
and partial-box rest. Six shear cases record initial rest and eight updates,
comparing no-slip/Navier/free-slip and fixed-q/fixed-a density responses. These
are 4 pressure projections and 48 bounded shear updates. No historical
seven-equation campaign is run or replaced.

The [checker](../../education/obstacle_flow.py) recomputes box measures and
components with exact rational arithmetic. It checks analytic affine pressures,
actual corrected speeds, cellwise divergence and pressure work. A tiny dense
rational Gaussian-elimination oracle checks each shear field independently of
the Rust tridiagonal algorithm, along with energy and impulse balances. Its
48 rational linear solves are fixture-level discrete controls, not continuum
reference integrations. Separately, the checker substitutes the **actual recorded
old and new fields**, plus the independently checked represented coefficients,
into the momentum equations and energy/impulse ledgers. The exact reference
trajectory never substitutes for those stored fields.

The actual shear equation gate is the native
\(4096\epsilon\max_i(|\mathrm{inertia}_i|+|\mathrm{stress}_i|+|\mathrm{load}_i|)\).
Pressure energy uses \(1024\epsilon\) times the actual ledger-term scale;
shear energy and momentum use their documented \(2048\epsilon\) scales.
Exported maxima and budgets must be finite and nonnegative. Every exported
budget is checked against its formula from the reported terms, with a separate
\(16\epsilon\) relative allowance for that short arithmetic expression.
Reported ledger terms/residuals are compared to exact rational recomputation
using \(64\epsilon\) times their contributing-term scales. These checks have
**no unit-sized absolute floor**. They are explicit fixture arithmetic
allowances, not new operator tolerances or universal IEEE error theorems.
The independent ideal-trajectory field allowance remains
\(4096\epsilon\max(1,|\text{exact control}|)\); pressure additionally requires
independent actual and predicted divergence at most \(10^{-10}\) s⁻¹.

Records, executable hashes, qualification receipts, browser captures, generated
book/PDF editions and logs stay outside Git. Publication requires a frozen clean
source, exact record bytes and matching source hashes. Missing packets produce
an explicit source-only page. Browser controls select saved native fields; they
do not solve fluid equations in JavaScript. Corruption tests must reject changed
geometry, gauges, fields, force semantics and ledgers in normal and optimized
Python.

## Proof boundary and original research

The existing `Discrete` and `StaticObstacle` Lean modules establish exact-real
graph symmetry/energy, incidence cancellation, component-constant jumps and
conditional positive-volume scaling. [ObstacleOperators.lean](../../../proofs/Rheon/ObstacleOperators.lean) adds 24
public theorems and 15 definitions. With pinned Lean 4.19.0 and the unchanged
axiom allowlist, all ten modules and the root compile; 88 public theorems,
118 explicitly expected declarations and 135 total declarations are audited.
The three real negative audit probes and four source/pin tests pass in normal
and optimized Python. This local qualification invoked the pinned compiler
directly with private dependency artifacts; it is not a new `lake build` or
hosted-CI receipt.

The new contracts derive signed/absolute residual-divergence scaling, per-label
compatibility, matched-mass pressure work, forced shear energy and two-wall
momentum, the Navier traction/loss split, and the reduced 3-by-3 symmetric-strain
identity. Exact shear coordinate equations and physical coefficient assumptions
remain explicit hypotheses. No theorem here proves Rust mesh admission, IEEE
arithmetic, PCG/Thomas convergence, complete graph-nullspace identification or
arbitrary-domain continuum accuracy.
All historical proof receipts and failed numerical evidence retain their own
source identities.

Batty, Bertails and Bridson's original pressure work formulates projection as a
kinetic-energy minimization and discusses subgrid geometry and face-area
approximations. It motivates the matched coefficient/mass relation; this bounded
stationary implementation does not reproduce their moving rigid/deformable
coupling. [Original paper, 2007](https://www.cs.ubc.ca/labs/imager/tr/2007/Batty_VariationalFluids/variationalFluids.pdf).
The author's [reference implementation](https://github.com/christopherbatty/FluidRigidCoupling2D)
explicitly distinguishes face-area weights from fluid-volume weights.

Batty and Bridson's viscosity work derives an implicit variational update from
symmetric-strain dissipation and emphasizes the coupled traction condition at
free surfaces. The present exact shear reduction has neither general
free-surface traction nor arbitrary embedded tensor assembly.
[Original paper, 2008](https://www.cs.ubc.ca/labs/imager/tr/2008/Batty_ViscousFluids/viscosity.pdf).

The next physical gate is shared staggered interaction geometry for general
embedded symmetric strain and an integrated pressure/viscosity step under one
consistent boundary model. Variable-density transport, moving/two-way solids,
capillarity and contact-angle dynamics need their own contracts and evidence.
