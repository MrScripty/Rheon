# Aligned coupled algebra and proof correspondence

Existing AlignedStrain supplies active stationary-box face velocities and strain
rows, not a timestep. The new reference assembles their mass, integrated outward
incidence and viscosity into a single backward-Euler system. It does not invoke
the existing split aligned_stokes update/projection experiment. SI dimensions:
velocity m/s, pressure Pa, M kg, B m² (Bu m³/s), S m⁻¹, W=mu*quadratureVolume
kg*m²/s, K=SᵀWS kg/s, A=M/dt+K kg/s. Positive M and nonnegative W imply A is
positive definite for dt>0. Normal rows carry2V and engineering shear rows carry
V, so the quadratic viscous work is mu times the discrete integral of2D:D.

For each positive open interior face the incidence column is +A_face at the
negative cell, -A_face at the positive cell. Componentwise column sums vanish.
B equals an oriented graph incidence times a positive diagonal edge-area matrix;
its exact-real rank is nc-components. Remove one pressure column and its paired
constraint row per connected component. The resulting B has full row rank.
[A,-Bᵀ;-B,0] is then nonsingular: a homogeneous solution satisfies Bu=0, and
uᵀAu=0 forces u=0; full row rank then forces the gauge-fixed pressure to0.
All eliminated velocities have homogeneous stationary boundary values, so no
unaccounted boundary lift/work enters. This argument does not admit nonhomogeneous
compatibility, disconnected cut policies, arbitrary mesh geometry or an iterative
backend. Independent graph reconstruction checks the actual assembly hypotheses.

Native residual r=M(u-u*)/dt+Ku-Bᵀp has units N. Polarization gives exactly
E(u)-E(u*)+.5||u-u*||²_M+dt*uᵀKu = dt*pᵀBu+dt*rᵀu.
The reported a posteriori residual-work allowance is
  dt*(sum_c |p_c(Bu)c| + sum_i |r_i*u_i|)/Echar,
with Echar=.5sum_i M_i*Uref², Uref=1m/s. Pressure work is zero only when actual
continuity is satisfied; report its numerical value and the full residual.
The original arbitrary analytic u* can have divergence; pressure projection loss
is included in the positive .5||u-u*||²_M term. No forcing/desired final solution
is manufactured from the assembled matrix. Enew<=Eold follows for an exact
unforced constrained solve; floating residual allowance is measured separately.

Reuse plan120's kernel-checked exact-real StokesWork.pressure_work, viscous_work,
coupled_work_with_defect and exact_constrained_work: set proof mass=M, proof
weight=W, proof strain=S, and defect=dt*r. Their assumptions are fixed rows/weights,
homogeneous stationary boundaries and the actual step equation/constraint. That
prior proof does not check this native assembly, binary64 errors, linear solver,
continuum traction or convergence. No duplicate/new Lean theorem or axiom claim.
Parent integration may bring the already merged plan proof into the final source;
this bounded reference's evidence remains source-bound to its own frozen commit.
