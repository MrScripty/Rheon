# Shared material map reference: one coupled formulation

Version `material-interface-lumped-linear-eos-planar-reference-v1`. This is an
inviscid, 1D, fixed-connectivity material finite-element reference per unit area,
NOT a selected particle kernel or a production solver. It closes the causal
question left by PR121/123. No positive-kernel-partition solver is implemented.

Let 0=X_0<...<X_n=1 be fixed reference interfaces, L_e=X_{e+1}-X_e, and N_a(X)
the continuous linear nodal basis, including BOTH boundary nodes. Define
 x_h(X)=sum_a N_a x_a, J_e=(x_{e+1}-x_e)/L_e>0,
 rho(X)=rho_ref(X)/J_e, m_e=integral_e rho_ref dX,
 Q_e=integral_e rho_ref(X)(X-X_e)dX.
The same map gives physical volume V_e=J_e L_e and mass first moment
 B_e=m_e x_e+(x_{e+1}-x_e)Q_e/L_e. Total mass is sum m_e, total material mass
moment is sum B_e. There is no cellmean-to-site EOS reset or global mass scaling.
The nonnegative partition includes wall DOFs, reproduces x exactly on the whole
mapped interval, and avoids extrapolation beyond the interior-site convex hull.
It REPLACES particle DOFs by interface DOFs: this is a substantial method change.

Declare ONE inertia: row-lumped M_a=integral rho_ref N_a dX>0, assembled from
[m_e-Q_e/L_e,Q_e/L_e]. Kinetic energy T=.5 sum_a M_a v_a² is a declared lumped
approximation, not generally the exact integral .5 integral rho_ref u_h².
Nevertheless integral rho_ref u_h = sum M_a v_a, sum M_a=sum m_e, and
sum M_a x_a=sum B_e exactly in real arithmetic. Gravity potential is the SAME
mass moment U_g=g sum B_e=g sum M_a x_a; gravity force is -g M_a.

On the fully wet linear EOS p=c²(rho-rho0), e'=p/rho², define U_e by material
integration integral rho_ref e(rho_ref/J_e)dX. Its reference-shifted value is
 U_e(J)-U_e(1)=c²[-m_e logJ+rho0 L_e(J-1)].
Pressure force assembled from that potential is
 F^p_e,left=-P_e, F^p_e,right=P_e,
 P_e=c²[m_e/(J_e L_e)-rho0].
Total unconstrained force is F=F^p-gM. Density is evaluated at material points;
P_e is a cell pressure mean for the force integral, not a new cellwise density
reset. Differentiating the SAME potential gives sum v_a F_a+d(U+U_g)/dt=0.
Stationary endpoint constraints have zero work. Fluid reaction on the holding
wall is F_0 and F_n, respectively, at equilibrium; it includes total pressure+gravity endpoint force.
Using pressure-only endpoint force as wall load discards part of the source work.
Moving walls, contact and moving solid geometry are not admitted here.

At x=X choose rho_ref=rho0 A exp(g(1-X)/c²), A=1+p_top/(rho0 c²).
Since p'=-rho_ref g, exact integration gives
 F_a=integral p N'_a-g integral rho_ref N_a=[pN_a]_0^1.
Every interior residual is zero for ANY strictly ordered reference partition;
endpoint fluid reactions are -p(0),+p(1). First moments are essential: replacing
Q_e/L_e by m_e/2 changes gravity and breaks this weak equilibrium identity.

For manufactured affine PHYSICAL pressure p(x)=p0+s x, pressure assembly uses
its exact element average. With V_a=integral_physical N_a dV (half adjacent
physical lengths), F^p_a-[pN_a]_boundary=-s V_a including endpoints. This boundary
lift must be retained; raw endpoint force alone is not a body pressure gradient.

The exact wet-branch Hessian is
 H=sum_e c² m_e/(L_e² J_e²) d_e d_e^T,
where d_e has entries[-1,+1] on the interfaces. For fixed endpoints H_II is SPD:
a zero quadratic form requires every adjacent displacement equal, and zero ends
then force all displacements zero. M_II is positive diagonal. No extra even/odd
DISPLACEMENT null exists in this 1D system. This is stronger than PR123's pressure
operator check, but does not establish a 3D acoustic spectrum. An alternating
CELL pressure produces adjacent-pressure differences, not a zero central stencil.
A row bound for M^-1/2 H M^-1/2 is an upper linear frequency-squared bound;
no nonlinear step, positive-J update, viscosity, or production dt is admitted.

## Integration and independent oracles

For rho_ref=R exp(-a s), s in[0,L], store m and Q from
 I_k=R L^(k+1) sum_{r=0}^{12} (-aL)^r/[r!(r+k+1)], k=0,1.
For |aL|<=.01 the omitted exact tail is bounded by
 R L^(k+1) exp(|aL|)|aL|^13/[13!(k+14)]. This is a truncation-only real bound;
it does NOT bound IEEE rounding. Native shifted energy is evaluated stably with eta=J-1 from displacement
differences: c²[(rho0 L-m)eta+m(eta-log1p(eta))]. For |eta|<.01 the final
remainder uses its alternating series through power18; this is not an EOS change.
Constant source uses a=0 exactly. Independent
Decimal/high-precision or positive quadrature compares source integrals, forces,
mass moments, energy derivatives and wall reactions. Native positivity/wetness
checks use each source element's endpoint extrema, not just sampling points.

## Primary basis and limits

[Dobrev, Kolev and Rieben](https://doi.org/10.1137/120864672) and their
[LLNL framework presentation](https://computing.llnl.gov/sites/default/files/Multimat-11-PartI.pdf)
provide primary background for compatible material finite elements and shared
force/energy assembly. The low-order, linear-EOS, lumped-inertia and gravity
construction here is explicitly derived; it is not reproduction or validation
of their high-order gas-dynamics solver. Existing Rheon `Discrete.adjoint_identity`, `pressure_energy` and
`pressure_nonnegative` supply the finite adjoint/weighted-square algebra when B
is the node-edge incidence and weights are nonnegative. They do not prove the
strict fixed-end nullspace result or instantiate source integration. These
proofs apply only with their stated hypotheses; they do not verify
this Rust assembly, moving-mesh geometry or a new water capability. No new Lean
source, axiom or kernel qualification is claimed.
