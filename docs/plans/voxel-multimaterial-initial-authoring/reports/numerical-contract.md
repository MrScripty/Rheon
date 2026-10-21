# Proposed numerical contract: filled two-material MAC box

This is original design mathematics and source-based planning, not an implemented solver or a qualification result. Method identifier proposal: `two-material-incompressible-mac-v1`. It does not extend the claims of legacy constant-density carrier, fraction transport, slab/column or compressible methods.

## 1. Model, state and physical meaning

The model is one-fluid incompressible, immiscible two-material flow with no capillary force. The interface is numerically diffuse under first-order transport. The phases share velocity and total stress; the method does not model two independent interpenetrating velocities, dissolved gas, diffusion/miscibility, gas compression or vacuum. A fully occupied gas cell has F=0 and positive mass, not zero occupancy.

Let F be liquid volume fraction, with gas fraction 1-F. For captured constants rho_l,rho_g>0 and mu_l,mu_g>=0,

\[
\rho(F)=F\rho_l+(1-F)\rho_g,\qquad
\mu(F)=F\mu_l+(1-F)\mu_g.
\]

The intended continuum equations are incompressible volume transport and conservative momentum with variable material properties:

\[
\nabla\cdot u=0,\quad \partial_tF+\nabla\cdot(Fu)=0,\quad
\partial_t(\rho u)+\nabla\cdot(\rho u\otimes u)
=-\nabla p+\nabla\cdot\{\mu(\nabla u+\nabla u^T)\}+\rho g.
\]

These are a scoped approximation, not a realistic compressible-air model. Pressure is a mechanical constraint in Pa with a gauge, not absolute EOS pressure. Reference material temperature is provenance of fixed input coefficients only; no temperature field, energy equation or phase change is present. Surface tension, wall drag and other particle-preset parameters do not enter this method. UI must display that exclusion rather than claim to apply every existing preset property.

First product offers only captured Water/Air rho and mu from the existing presets. Both entries may be displayed and named; physical editing/other pairs are not offered until separately qualified. Numerical coefficient tests include equal density and contrast up to 1000:1, covering the approximate 830:1 Water/Air reference ratio, but those tests do not certify arbitrary fluids or an entire continuous parameter envelope. Unsupported pairs return `unsupported_material_pair`, not a scalar-density approximation.

### Layout and authority

Use existing checked `GridGeometry` and x-fastest indexing. Cell volume V=h_x h_y h_z; F, rho and p are cell-centered. Velocity components are on their own MAC faces, including fixed boundary normal faces. Initial contents use the same origin, counts and spacing; preview resolution is unrelated. Cells are half-open in world space, with a checked upper-boundary picking rule.

New mode stores f64 F and f64 MAC velocity, accepted and candidate. Palette and geometry are immutable for a run. Rho/mu arrays and face masses may be cached but are derived exclusively from F and the captured constants; they are not separately editable state. Accepted pressure is the last accepted interval projection, stamped with its coefficient/candidate generation. Constructor pressure is unavailable until computed; a general zero-velocity draft is not declared hydrostatic equilibrium. Clock, generation, F, velocity, accepted pressure and report publish atomically. Existing f32 methods retain their exact paths and representation.

For interior face f between adjacent cells L,R, dual volume is V, and

\[
\rho_f=\tfrac12(\rho_L+\rho_R),\qquad m_f=V\rho_f>0.
\]

This is the mass of two half cells. Boundary normal velocity is fixed to zero and is not a dynamic momentum degree of freedom; do not divide a fictitious boundary-face momentum by an undefined mass. Distinguish mass in kg, density in kg/m³, volume in m³, viscosity in Pa s and velocity in m/s. Internal mixed cells contain both reference phase masses, not a blended material ID.

## 2. One complete proposed step

1. Validate immutable geometry/palette, requested positive finite dt, accepted finite/bounded state, boundary zeros and all allocation/work budgets. The owner retains accepted state unchanged on every failure.
2. Build a closed, checked old-time transport volume flux Q from accepted velocity using the closure below. It is scratch, not a mutation of accepted velocity. Select dt within both phase and dual-mass Courant limits; bounded retry may shorten the request from the same accepted state and must report actual dt. No hidden unbounded substeps.
3. Advect F using the shared Q, form material mass flux J from the same phase flux, and derive end-candidate rho, mu and dual face masses. Check phase volumes, mass, bounds and continuity defects before further stages.
4. Advect staggered momentum with the derived dual mass flux; divide by the same end-candidate face mass, not a stale scalar density. Apply variable viscous stress to that advected state using those same masses, subject to its stability/energy gate.
5. After viscosity, apply captured gravity as momentum impulse dt m_f g, then immediately solve the variable-density projection with end-candidate coefficients and correct candidate f64 faces. Do not apply viscosity to the gravity predictor: its artificial near-wall strain would alter the hydrostatic pressure gradient even if projection returned zero velocity. Check true linear residual, independently differentiated stored divergence, boundaries, Courant, phase/mass and finite arithmetic. Reject on incompatible RHS, nonconvergence or any invalid intermediate/final state; no alternate backend or scalar-density fallback.
6. At a final cancellation checkpoint, swap/publish the entire candidate and its report. No fallible callbacks or allocations occur after this commit point. Failed/cancelled attempts preserve accepted fields, time and generation, including prior accepted pressure metadata.

The initial version uses one serial CPU numerical owner, existing joinable native worker and bounded reusable scratch. New parallel scheduling, worker-observation instrumentation, trained providers and SGS qualification are unnecessary dependencies.

## 3. Phase transport and the finite-divergence trap

Orient each interior face once from its lower to upper cell. Q_f=A_f u_f has units m³/s; outer Q is zero. With upwind old F at each face, H_f=Q_f F_donor is liquid volume flux and Q_f-H_f is gas volume flux. In exact arithmetic the conservative update is

\[
F_i^{cons}=F_i^n-\frac{\Delta t}{V}\operatorname{div}_{int}H_i.
\]

Shared face fluxes cancel in the global phase-volume sum. For a discretely divergence-free Q and admitted Courant numbers this donor update preserves bounds, while producing a diffuse rather than reconstructed sharp interface. No threshold/rebinarization follows transport.

**A pressure residual is not sufficient for bounded fractions.** A fully liquid cell with F=1 can exceed 1 under a tiny convergent volume-flux residual. The existing fraction operator's source/limit gates do not solve new two-material consistency merely by accepting a pressure tolerance. No “clamp small overshoots” or unreported renormalization is permitted.

### Explicit transport-flux closure

Before phase/momentum advection, reconcile the volume flux on a deterministic tree of all cells: a cell's parent lowers X when possible, otherwise Y, otherwise Z; root is (0,0,0). Compute each cell's integrated outward residual d_i. Process children before parents; alter only the shared child-parent flux to cancel the child's accumulated residual, propagating that residual into the parent. In exact arithmetic all cells close, and the root receives the global boundary flux, zero for this sealed box. This adds no wall flux, material source or arbitrary pressure mean subtraction.

In binary64 the closure is not exact by declaration. Recompute residuals from actual stored corrected Q, using the same orientation and independent summation. Record max/sum flux corrections, induced velocity corrections and residual-volume defects. Exact-arithmetic bounds from this tree include max face correction <=sum_i |d_i| and summed corrections <=L sum_i |d_i|, with L<=n_x+n_y+n_z-3. Runtime budgets add rounding bounds derived from actual operation counts/magnitudes and the admitted original divergence limit. Reject incompatible root residuals, excessive correction or failed finite arithmetic. The closure is a small certified consistency correction, not permission to replace a poorly converged velocity field.

The stored F update uses the algebraically equivalent constant-preserving evaluation

\[
F_i^{adv}=F_i^n+\frac{\Delta t}{V}\sum_{f\;incoming}|Q_f|(F_{donor(f)}^n-F_i^n).
\]

It is equivalent to the conservative formula only when d_i=0. Their difference is dt F_i^n d_i/V, explicitly measured and budgeted. Incoming and outgoing volume Courant numbers must each be <=1/2; this is the chosen method safety margin, not a measured performance result. Pure constant phases remain constant algebraically; the binary64 implementation must still check strict stored 0<=F<=1, including nonfinite/underflow policy. Do not claim a universal floating-point proof.

Require direct per-phase totals and the sum of induced residual-volume defects to fit predeclared **cumulative arithmetic budgets**, not merely the pressure tolerance. Derive gamma_k=k epsilon/(1-k epsilon) bounds from the arithmetic reduction/update paths and magnitudes, with k and any conservative safety factor reviewed before outputs exist. A larger arbitrary user epsilon must not authorize mass drift. Track run-to-date allowed rounding contributions and actual mass differences, not an independent fresh tolerance every step that hides long-run drift.

M2 has a hard stop: if pure phases, ordinary mixed interfaces or target Water/Air cases repeatedly cannot satisfy strict bounds and these budgets, revise the bounded conservative flux method with independent review before native exposure. A permanently rejecting supported mode is not completion. Do not add clipping, silently enlarge budgets or suppress the residual report to finish the milestone.

## 4. Mass-consistent staggered momentum

Use the same corrected Q and H to construct mass flux

\[
J_f=\rho_l H_f+\rho_g(Q_f-H_f)
=\rho_gQ_f+(\rho_l-\rho_g)H_f\quad [kg/s].
\]

Old/new cell mass derives from rho(F)V. With the stored constant-preserving evaluation, its continuity defect relative to the conservative J update is

\[
m_i^{adv}-[m_i^n-\Delta t\operatorname{div}_{int}J_i]
=\Delta t\rho_i^n d_i+\text{arithmetic error}.
\]

The multiplier is **old mixture density**, not just gas density. Budget defects using actual densities and mass scales, crucial at Water/Air contrast.

Each a-component velocity has a dual control volume made from two adjacent half cells. Its dual mass flux through a b-normal side is the half-sum of the corresponding adjacent primary J fluxes: for b=a, interpolate the two neighboring a-face fluxes to the intervening cell-center plane; for b!=a, average the b-face flux in the two a-adjacent cells. The same area/orientation convention is used throughout. Interior dual mass continuity is therefore the matching half-sum of primary cell continuity, including its measured floating defect. Boundary constraints/reactions are explicitly included rather than inventing external material flux.

Transport each component momentum P_f=m_f u_f conservatively using dual J and donor old component velocity on that dual side; the sign of the dual mass flux chooses its donor. Require dual outgoing mass Courant <=1/2 as well as the volume Courant bound. Divide candidate momentum by end-candidate m_f. Transport flux, end masses and projection coefficients must come from the same F update/generation.

For uniform component velocity U in an isolated compatible transport fixture, the velocity error induced by dual continuity defect R_f is -U R_f/m_f^{adv}, plus arithmetic. Verify that relation and its small bound; do not promise exact free-stream preservation after nonexact closure. Product closed-box velocities obey impermeable walls, so a globally uniform nonzero vector is not a valid full-box fixture. A local/periodic **test oracle only** can decide the interior transport invariant; it does not add portal/periodic product support.

Closed walls exchange momentum through pressure/viscous reactions. The invariant is phase and total mass conservation within arithmetic budgets, plus momentum update/force/reaction accounting—not unconditional global momentum conservation in a closed box. No energy-conservation claim is made for first-order donor transport.

## 5. Variable-density projection and equilibria

With D the finite-volume divergence, G the MAC pressure gradient and beta_f=1/rho_f from the end-candidate dual masses, solve

\[
A p=-\frac{V}{\Delta t}D u^*,\qquad A=-V D\beta G,\qquad
u^{n+1}=u^*-\Delta t\beta Gp.
\]

The neighbor weight is A_f/(rho_f h_d), positive and shared by both rows. The complete filled pressure graph is connected. Use the existing compatible sealed-box gauge convention, with a pre-gauge compatibility check; pressure is relative. Check true residual independently of the recurrence and differentiate actual stored f64 faces afterward. Projection is a mass-weighted orthogonal velocity correction in exact arithmetic, not proof of physical interface calibration.

Add a mutually exclusive full-box face-density/coefficient variant to existing `PressureOperator`; do not combine it with slab/column/obstacle geometry. Reuse `PressureWorkspace::solve_rhs` and its existing f64 PCG, initially Jacobi. Add only the f64 RHS/divergence/correction paths this mode needs. `neighbor_weight`, `fixed_box_weights`, `apply_full`, `diagonal` and correction must all distinguish variable coefficients. The current “surface is None” tests must no longer imply constant weights. `sgs.rs` consumes `fixed_box_weights`; it must fall back correctly or explicitly refuse this method, without claiming untested SGS parity. Old f32/scalar paths keep their semantics and focused regression evidence.

Gravity uses the same end-candidate m_f as pressure. For aligned horizontal material layers at rest, the discrete hydrostatic condition is

\[
p_R-p_L=\rho_f g_d h_d.
\]

Half-cell density averaging reproduces the two half-column weights at a cell-aligned interface. That is a deciding rest test at different density ratios, interface heights and vertical origins, including nonzero unequal viscosities. Viscosity of the advected rest state is zero; gravity immediately before projection is then balanced by the stated pressure differences. Check the near-wall pressure as well as velocity so the wrong gravity-before-viscosity ordering fails. With g=0, any supported F field and uniform pressure should remain at rest. With g!=0, arbitrary painted blobs are generally not equilibria: buoyancy-driven motion is expected, not evidence of a pressure bug by itself. A low-pressure gas pocket is not modeled, and projection pressure is never passed to an ideal-gas EOS.

## 6. Variable viscosity and boundaries

All outer faces are stationary impermeable, free slip (zero tangential traction), as in the existing filled-box viscosity scope. No no-slip claim, obstacle mask, contact angle or moving boundary is imported. Initial normal wall velocity is exactly zero; reject a loaded incompatible velocity instead of silently projecting unknown author intent. UI initial state is rest only.

Use the symmetric strain layout of the existing Cartesian viscous formulation: normal strain at cells, engineering shear at edges. Cell mu is volume weighted as above; shear-site mu is the harmonic mean of its surrounding cells (zero if any contributing mu is zero), with the declared boundary free-slip quadrature. This gives a positive weighted strain form K=S^T W_mu S, not per-component smoothing with one averaged nu. Store this coefficient/averaging choice in the method version; do not switch it based on visual appearance. Its interface accuracy is a numerical claim to test, especially for layered shear.

Update with the same candidate mass matrix M_f. For the explicit stage, derive a conservative spectral upper bound lambda_bar for M^{-1/2} K M^{-1/2} from actual stencil row absolute sums; require dt lambda_bar<=1 (stronger than the exact energy limit 2). Verify finite coefficients, kinetic-energy decrease and the discrete strain/work identity within arithmetic budgets. Zero mu is the exact no-stress limit. Distinct mu must change stress/damping measurably in an independent analytic/dense fixture. No unconditional continuum traction accuracy follows from positive energy alone.

## 7. Budgets and acceptance envelope

Start with dense counts 4–64 per axis, finite physical spacing, existing native physical-density limits, and a separately charged owned-array cap. The caller may request less; no legal count promises allocation. Account both F/velocity buffers, pressure and PCG scratch, derived cell/face coefficients, primary/dual flux work, closure tree/residual scratch and reports by actual Vec capacities. Reuse work arrays where lifetimes do not overlap, but keep accounting explicit. The 192 MiB legacy cap is a reference default, not proof the new aggregate fits. Editor, immutable input, recorder, retained preview and process/GPU overhead are separate.

The step has bounded iteration/retry counts and typed failure with stage, cell/face where applicable and retained accepted generation. Numerical owners choose reviewed tolerances from analytic/error budgets **before** candidate results; freeze them in intent. Existing pressure tolerances are not automatically valid at 830:1 contrast. Independent small dense, hydrostatic, transport and stress checks decide the envelope; refinement evidence reports first-order diffusion separately from solver residual and roundoff. No simulation, test cohort or benchmark has run in this planning task.
