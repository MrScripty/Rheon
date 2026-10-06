# Finite relative transport on the existing fitted physical flow

This research checkpoint preserves restricted flow source `96bb0efb5674d5a9505dcfae1d4ee5d2b73df001` and evidence `d8133c302fa61644062d347c3b387e7f6f5d4e8b`. It changes no production Rust or public stepping refusal. It closes a missing finite physical face-flux contract on a bounded deformation of that same flow, and records two failures that constrain the next implementation.

## Precise obstruction and smallest connected example

Retain the prior domain, density rho=3, width b=1, dynamic viscosity mu=1/20 and full velocity U=(a,0,w), a=1/4. Its varying graph H(x,t)=H0(x-at), H0=[1,5/4,1,3/2,1] on periodic quarter-spaced vertices, follows the accepted carrier velocity. Relative pressure is analytically zero; this is not a newly claimed pressure solver.

Fix the bottom mesh abscissae, move every cap vertex by q(t)=at, and recompute macro barycenters and the actual periodic Powell–Sabin intersections at each q. Boundary split points remain midpoints. The fitted mesh now deforms, whereas the physical domain still translates. This gives nonzero fluid-minus-mesh xy flux, changing nodal liquid masses and conservative third-momentum transport on the existing flow. It is not an unrelated prescribed height sequence or a new analytic class selected to avoid the preceding transport gate.

The supported research interval is 0<=q<=1/8 (time 0 through 1/2). Exact rational symbolic root isolation certifies positive 48 microareas and 32 nodal masses and absence of face-flux reversals and poles on this interval. The total liquid mass is identically 57/16. Twenty-two nodal masses change and 76 oriented shared face pairs have nonzero relative flux. No topology replacement, support creation, general time interval or uniform pressure stability is inferred.

The actual split points are rational functions of q. On each median-dual piece, fluid velocity is (a,0) and mesh velocity is a times the derivative of its recomputed nodal position with respect to q. Integrating the P1 velocity on the segment from a primal-edge midpoint to its triangle barycenter gives mesh weights 5/12,5/12,1/6 on the three primal nodes. The old corrected instantaneous reference supplies the independent exact Dual check; its bottom-fixed motion is now precisely the declared finite trajectory rather than being reused for an incompatible material translation.

Let f_ij(q) denote the physical oriented mass flux and g_ij(q)=f_ij(q)/a. The finite transfer is

    F_ij = integral(q_old,q_new) g_ij(q) dq.

The symbolic reference derives these functions from geometry, computes antiderivatives, verifies their derivatives, and evaluates the definite transfers at high precision. Fifty-six antiderivatives contain logarithms. Positive/negative transfer must be integrated separately if a flux changes sign. Exact root isolation shows no such reversal on this qualified interval; it does not license dropping that gate on arbitrary meshes.

## GCL alone does not certify the physical transfer

All instantaneous GCL functions m'_i(q)+sum_j g_ij(q) vanish identically, and the antiderivatives integrate the actual finite GCL. In this particular fixture each nodal mass is affine in q, despite nonlinear individual split-point motion and logarithmic face primitives.

Consequently frozen endpoint flux has the correct mass marginals while giving the wrong face transfers. For q=0 to 1/8, endpoint-frozen GCL error is approximately 2.08e-17, but a face transfer differs from the true integral by 0.001636186128. The difference is a circulation invisible to node balances. The initial attempt to distinguish this wrong flux using GCL alone failed; its actual traceback is retained. Final tests reject it by physical face provenance, not by changing mass tolerances or fitting a correction to new masses.

[Farhat, Geuzaine and Grandmont's primary publication abstract](https://www.sciencedirect.com/science/article/abs/pii/S0021999101969323) connects discrete geometric conservation to constant preservation and, for its sample schemes, stability. We do not import a stability theorem for this modified constrained donor scheme. The mass-marginal counterexample and all identities below are derived and tested for the declared Rheon construction.

## One constrained transport and viscosity equation

Fixed topology retains the existing constant third-component embedding R, including cap/bottom midpoint traces. Its reduced mass is generally not diagonal. With physically integrated shared transfers, define the donor operator A by

    A_ii=m_new,i+sum_j max(F_ij,0),  A_ij=min(F_ij,0), i!=j.

The candidate endpoint third velocity V=R z solves

    R^T (A+dt K_new) R z = R^T M_old U_old,z.

The same shared transfers carry all three components. Constant xy carrier velocity is maintained through the same constant-preserving row balance; its divergence and strain vanish in exact arithmetic. The cap trajectory therefore remains driven by that accepted velocity and pressure remains analytically zero for the same physical family. This research has not implemented a public native step or generic pressure coupling.

For an exact finite GCL and positive masses,

    x^T A x = 1/2 sum_i (m_old,i+m_new,i) x_i^2
              +1/2 sum_unordered_ij |F_ij| (x_i-x_j)^2.

Thus the symmetric part is positive, and so is its restriction by the injective R. A+dt K is generally nonsymmetric: the preceding CG viscosity solve must not be reused without changing the solver. The next native implementation needs bounded nonsymmetric iteration and a true composed residual gate.

For full kinetic energy, define the increment loss with old masses and donor loss with unordered faces. If r is the reduced composed residual and d_i=m_new,i-m_old,i+sum_j F_ij is the measured GCL defect, then

    E_new-E_old + 1/2 sum_i m_old,i |V_i-U_old,i|^2
      +1/2 sum_unordered_ij |F_ij| |V_i-V_j|^2 +dt V_z^T K_new V_z
      = z^T r -1/2 sum_i d_i |V_i|^2.

A separate exact deliberately inconsistent algebra test fixes the defect sign. Neither defect work nor true solver work is silently counted as viscosity. Admissible translations conserve resultant momentum because R contains the constant mode and the shared flux/strain forces cancel. The native successor must test all components, geometry/volume/mass, actual divergence, true residual, physical face integration, work and cancellation before a common final publication.

## Preserved convex-bound failure

The unconstrained donor M-matrix theorem from the earlier formulation does not transfer automatically through R^T A R. On the same finite path, the constrained pure-transport coefficient map has 93 negative entries, minimum -0.0120824975. A nonnegative admissible nodal velocity fixture with range [0,1] produces range [-0.0120824975,0.5107670997]. This is a recorded **failed componentwise convex-bound claim**, not an accepted positivity theorem. Velocity components may have either sign; the measured momentum and positive work identity still hold. Liquid masses themselves remain positive from the actual geometry. No previous unconstrained positivity/Lean claim is weakened or extended to this constrained solve.

## Actual numerical evidence and next implementation gate

An independently assembled dense endpoint solve with actual integrated flux evolves the y^2 third-component fixture on this time-dependent ALE mesh. Its reference is the independently integrated nonautonomous semidiscrete ODE, including actual Mdot, donor flux and K(q), not the previous fixed-mesh exponential. A DOP853 tolerance refinement checks the time-reference error. At dt 0.05,0.025,0.0125 the mass-L2 temporal errors are approximately 0.00326518,0.00165314,0.000831893, with ratios approximately 1.975 and 1.987. The momentum and changing-mass work ledgers pass; every interval has nonzero donor and strain dissipation. A separate 16-point Gauss physical integration agrees with exact face primitives, including the whole interval and smaller initial/final steps.

Exact pressure-image elimination measures rank 32 at q=0,1/16,1/8 only. These are three real snapshots, not a claim of a manufactured pressure degree count, a whole-interval rank theorem or an inf-sup constant. No new Lean or Rust/IEEE proof, continuum spatial accuracy or general free-surface pressure validation is claimed. All frozen proofs, Jacobi fixtures, failed accuracy records and public refusals retain their bytes.

The finite transport contract is therefore usable for a native successor of the same flow. The remaining implementation gate is one explicitly bounded candidate geometry workspace plus physical quadrature and a nonsymmetric composed solve, with common publication and failure/cancellation rollback. It must independently reject the face-circulation counterexample and preserve the constrained convex-bound failure. General pressure coupling, deformation outside this bounded qualification, nonuniform xy momentum, adhesion, variable density and surface reconstruction remain separate unfinished criteria.
