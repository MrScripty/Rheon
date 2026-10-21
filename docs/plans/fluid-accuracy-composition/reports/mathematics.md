# Mathematical repair decisions and research

Research date 2026-10-19. This is a selected roadmap plus explicitly unresolved method gates, not evidence that Rheon implements these equations. Independent research by `/root/math_research` informed this document. [Plan](../plan.md) owns status; [verification](verification.md) owns proposed acceptance. All discrete equations below are exact-real reference formulations until native floating-point/geometry correspondence is tested.

## Water, dimensions and scope

Freeze audit replay constants. At 293.15 K near atmospheric pressure, [IAPWS SR6-08(2011), eq. 7](https://iapws.org/technical-guidance/release/LiquidWater.download) evaluates dynamic viscosity to 0.001001567264603 Pa s, consistent with rounded 0.001002. Its release discusses generally 1% viscosity uncertainty and 0.17% at 293.15 K; stable-liquid density uncertainty is 0.0001%. Density and dynamic viscosity are independent inputs; kinematic viscosity is nu=mu/rho, never mu. Diffusion nu*dt/h² is dimensionless, rho*g*H is Pa, and mu*gradient(u) is Pa.

[IAPWS R1-76(2014)](https://iapws.org/technical-guidance/release/Surf-H2O.download) gives sigma=0.2358*tau^1.256*(1−0.625*tau) N/m, tau=1−T/647.096. At 293.15 K sigma=0.0727361404216 N/m, table uncertainty 0.00036 N/m. This is a water/vapour surface correlation, not a calibration of Rheon's cohesion coefficient. With audit rho/g its capillary length is about 2.726 mm. Surface tension cannot be dismissed for millimetre features merely because a metre-scale bulk example looks plausible.

Scope is Newtonian, approximately isothermal single water liquid with prescribed atmospheric stress and an explicitly admitted wall condition. Grid incompressibility omits acoustics. Particle weak compressibility introduces an artificial c and density error. Evaluate Re=UL/nu, Fr=U/sqrt(gL), We=rho*U²*L/sigma and Mach=U/c for each experiment. Laminar water shear tests verify viscosity without establishing metre-scale turbulent predictive ability. Turbulence, entrainment, contact-angle hysteresis and moving coupled bodies need separate claims.

## F1/F2/F5: coherent particle wall subgraph

Existing swept center collision is geometric contact; it does not establish density support, no-slip or pressure traction. Select the wall method by a bounded planar comparison, then freeze one version. Primary candidates are [Adami–Hu–Adams 2012, DOI 10.1016/j.jcp.2012.05.005](https://doi.org/10.1016/j.jcp.2012.05.005) dummy-wall pressure/velocity extrapolation and the semi-analytical support family examined by [Mayrhofer et al., sections 2–3](https://arxiv.org/pdf/1304.3692). The latter analyzes only approximate discrete skew-adjointness; neither paper supplies a blanket energy proof for Rheon's geometry.

Our required design: share boundary side classification and symmetric segment visibility across density/continuity, pressure and viscous interactions. Supply wall support, pressure derivative and reaction/work with the same quadrature. Density-only masking, viscosity-only retuning or a lattice normalization multiplier is insufficient. First support a resolved stationary planar volumetric wall and explicit slip/no-slip policy. Compare the candidates on hydrostatics, wall shear and opposite-side isolation before admitting corners, curved surfaces or two-sided thin triangle barriers. Unsupported wall topology/thickness is refused. Thin two-sided surfaces need side-labelled support; filled solid dummy particles alone do not hydraulically isolate them.

For a fixed symmetric visibility v_ij=v_ji and antisymmetric base forces F_ji=−F_ij, masking preserves action/reaction. For viscosity F_ij=a_ij(v_j−v_i), a_ij=a_ji≥0, pair power is −a_ij|v_i−v_j|². These algebraic facts require actual enumeration/coefficients to satisfy the hypotheses. They do not prove completeness, consistency or no leakage. Visibility depending on positions may jump at geometry events; ignoring that derivative/event invalidates a global potential-energy claim. Retain geometry-event and wall impulse/work defects; refuse an unsupported conservation claim instead of hiding the defect. Angular momentum additionally needs central forces.

[Dehnen–Aly 2012](https://arxiv.org/pdf/1204.2471) motivates studying support/neighbour growth and pairing-resistant kernels. Its astrophysical evidence does not validate water walls. For Rheon, native ordered and perturbed cloud tests must vary both spacing and h/spacing, measure partition/gradient/Laplacian moments and smooth shear decay, and separate sampling error from wall error. Fixed support ratio is not a general consistency limit. A new kernel/normalization must be a new compatible formulation, not a silent coefficient change.

### Equilibrium and timestep derivation

The existing liquid pressure clamps c²(rho−rho0) at zero. On the compressed wet branch, our continuum derivation from dp/dy=−rho*g, p(H)=0 and p=c²(rho−rho0) is:

    rho(y) = rho0 exp(g(H−y)/c²)
    p(y)   = rho0 c² [exp(g(H−y)/c²)−1].

The free surface is the branch endpoint; do not claim differentiability across the clamp or exact reproduction by an SPH stencil. A constant-density linear hydrostatic profile plus linear EOS is only an approximation. The audit began with uniform unpressurized fill, so its transient is not proof of instability of a properly initialized hydrostatic equilibrium. First measure a **discrete** equilibrium defect, then compare an admitted near-equilibrium rest run. For the compressed branch, integrating de/drho=p/rho² gives e(rho)=c²[log(rho/rho0)+rho0/rho−1] J/kg with e(rho0)=0; the clamped branch below rho0 can use e=0. The reference total is sum m_i(0.5|v_i|²+g*y_i+e(rho_i)). Density evolution and force quadrature must establish the corresponding discrete chain-rule/work identity or explicitly measure its defect, plus boundary/visibility-event/contact work. Kinetic-plus-gravity growth alone can be legitimate conversion of compressive energy and is not an energy-creation diagnostic.

For depth H and allowed relative compression epsilon, c² ≥ gH/log(1+epsilon). At H=1 m, epsilon=.01 requires c≥31.3936330073 m/s; c=20 gives 2.481962859% continuum equilibrium compression. Also impose declared anticipated speed Umax/c≤0.1 for the first low-Mach envelope. Re-evaluate when the state leaves it. Changing c is a stiffness/model change, not a free stability repair. A Tait EOS would have a different equilibrium and method version; it is not selected here.

Select explicit acoustic, acceleration, viscous and capillary dt bounds from the **actual** chosen operators and support convention. Scale candidates are h/(c+U), sqrt(h/|a|), h²/nu and sqrt(rho*h³/sigma), each with a derived/versioned safety coefficient and dimensional check. These scalings alone are not a stability theorem. Determine spectral/row bounds for the selected discrete explicit terms, check shortened dt, and validate the proposed state before commit. Avoid borrowing a numerical prefactor from a different kernel/grid method. Preserve failures and tighten admission rather than clipping velocities to make rest tests pass.

## F4: geometry-consistent initialization

Selected reference construction uses disjoint source cells C_i and clipped fluid regions Q_i=C_i∩Omega_liquid\Omega_solid:

    V_i = |Q_i|,   x_i = integral_Q_i x dV / V_i,
    m_i = integral_Q_i rho_equilibrium(x) dV.

For uniform incompressible water m_i=rho0*V_i. Empty cells create no particles. Split/refuse disconnected intersections or a representative lying in solid; a centroid of a nonconvex fluid region need not lie in fluid. Report clipping uncertainty and retained before/after volume. Do not globally renormalize masses to match the desired answer. Variable V_i and m_i must enter the selected density/force/wall quadrature coherently, requiring P1 compatibility; applying this seeding alone to equal-volume assumptions is not an admitted repair. Use independently known rotated-box volume and a separate clipping oracle, including translations relative to lattice phase.

## F3/F7: a shared grid geometry and unified Stokes reference

Select the unified pressure–viscosity formulation of [Larionov–Batty–Bridson 2017](https://elrnv.com/projects/variational-stokes-a-unified-pressure-viscosity-solver-for-accurate-viscous-liquids/) as the reference method family. Its [supplement, eqs. 5–15 and free-surface MMS](https://cs.uwaterloo.ca/~c2batty/papers/Larionov2017/supplemental.pdf) couples rho(u−u*)/dt=div(tau)−grad(p), div(u)=0 and tau=2mu*D(u), with natural traction (−pI+tau)n=0. Its MMS has nonzero surface pressure, discriminating against an incorrect independent p=0 boundary. [Batty–Bridson 2008, section 8](https://www.cs.ubc.ca/~rbridson/docs/batty-sca08-viscosity.pdf) identifies limitations of splitting this coupling. Keep the split preview as a named comparator, not the selected accurate free-surface stress model.

Our implementation obligations: define one integrated outward cell-flux B, positive velocity mass M, strain S and nonnegative stress quadrature W; K=SᵀWS. Match geometry and boundary lift everywhere, including source/transport apertures, actual fluid volume and pressure/stress traction observations. Grid sizes, associations, units and orientation signs must be explicit. Small-cell merging/redistribution or refusal must preserve declared flux/mass/work identities; arbitrarily flooring a mass/volume to improve conditioning is not geometry consistency.

For stationary homogeneous boundaries, exact backward Euler is

    M(u−u*) + dt K u − dt Bᵀ p = 0,    B u = 0.

Here B maps velocity to m³/s; Bᵀp has units N, M kg, K kg/s. Dot with u, use adjointness, and obtain

    E(u)−E(u*) + 1/2 ||u−u*||²_M + dt uᵀ K u = 0.

Pressure does no work because Bu=0. Nonhomogeneous wall/atmosphere/capillary terms require explicit boundary work; approximate momentum residual r adds r·u on the right. Pressure residual and local fluid volume give divergence tolerance in 1/s; one raw residual limit cannot serve all cut-cell volumes. Prove component compatibility/gauge handling and validate approximate-solve work, not only an ideal SPD claim.

[Batty et al. 2007, eqs. 4–7](https://www.cs.ubc.ca/~rbridson/docs/batty-siggraph2007-variationalcoupling.pdf) provides a variational pressure construction with dual-volume or approximate A*dx inertia. Therefore replacing rho*A*d by a different weight is not by itself a continuum/cut-cell accuracy repair. Test the assembled M/B/S, actual face flux, hydrostatic balance, smooth MMS and integrated traction on the same rotated geometry. Positive algebra and an exact Lean identity are necessary evidence, not evidence of geometry order.

Interface transport must share the geometry and preserve liquid volume with sources/fluxes. Choose conservative geometric flux reconstruction only after its bounded selection experiment; distinguish interface location/shape error from exact volume. Cap split fluxes with a method-specific multidimensional CFL. Grid/timestep refinement are independent studies; reduced dt at fixed dx may increase donor-cell smearing. Do not silently introduce a new advection method under the old version.

## F6/F8: accuracy envelopes, capillarity and wall work

Keep the original 2.139313% coarse-channel failure against 2%. Publish the measured convergence and resolution requirement rather than changing its threshold. New error studies separate space, time, solver tolerance, reconstruction and geometry. No monotonic accuracy claim for dt alone.

Set outward-liquid n and kappa=div_surface(n); a sphere has kappa=2/R. For atmosphere p_atm, selected surface traction is

    (−pI + 2mu D)n = −(p_atm + sigma*kappa)n.

Use a 3D drop delta-p=2sigma/R, not the 2D cylinder sigma/R. A balanced pressure/capillary discretization, curvature reconstruction and explicit capillary timestep need joint qualification. The official [Basilisk tension implementation](https://basilisk.fr/src/tension.h) and [capillary-wave test](https://basilisk.fr/src/test/capwave.c) are useful primary reproducible comparators, not evidence of Rheon equivalence. Existing cohesion remains uncalibrated preview until its own effective-sigma/drop/wave tests pass or it is superseded.

Wall force on fluid and negative body reaction use matching quadrature. Prescribed wall motion does work integral(t·u_wall)dA dt. Stationary no-slip walls have zero mechanical wall work; physical viscous dissipation remains in the fluid balance. Geometric collision projection may lose kinetic energy: record its impulse and loss separately, including discarded displacement/contact remainder. Before moving geometry admission, require swept-volume/geometric-conservation-law tests. Moving prescribed walls and genuinely coupled bodies are separate capabilities.

## Verification versus physical validation

The plan's MMS, exact algebra, dense operators, conservation and grid/time refinement are numerical verification. A water preset and plausible splash are not experimental validation. [Kleefsman et al. 2005, section 7](https://pure.rug.nl/ws/portalfiles/portal/2941055/2005JCompPhysKleefsman.pdf) supplies a primary dam-break/obstacle benchmark description. Quantitative Rheon validation still needs retained independent numeric observations and their measurement mapping/uncertainty. Plot images may support a documented digitization study with propagated error; they are not raw sensors. Pressure validation additionally needs matched wall pressure/traction support. Do not use surrogate outputs as observations.

## Proof scope

Existing `Discrete`, `ObstacleOperators`, `BoundedPhysics`, `AlignedStrain`, `MaterialPreview`, `WallFriction` and `NoSlip` modules already establish many finite algebraic conservation/work facts. Reuse them; do not duplicate them as a claimed boundary repair. New isolated follow-on proofs target checked publication logic and the selected coupled work identity only where feasible. Qualification records exact statements and axioms. Particle wall geometry, visibility events, common cut-cell assembly, capillary balance and EOS derivative proofs remain **planned gates tied to the selected formula**, not assumptions hidden behind an abstract theorem. Exact reals are not IEEE execution, mesh correctness, convergence or scientific validation.
