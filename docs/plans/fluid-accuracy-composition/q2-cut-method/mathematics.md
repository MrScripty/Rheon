# Basis-integrated reference proposal and conditional properties

The selected2017continuum saddle functional, allowing prescribed liquid-outward
traction g and body force f, is
  .5rho∫Ω|u-u*|² + dt∫Ω[mu D(u):D(u)-p divu-f·u] -dt∫Γg·u.
Its variation in velocity gives mass/dt + viscous work - pressure divergence;
physical sigma=-pI+2muD has sigma*n=g. No separate pressure Dirichlet condition.
Use a fixed planar convex represented polygon and1mextrusion, rho/mu positive,
stationary geometry during this numerical Stokes substep. A free-boundary
kinematic update/transport and solid Dirichlet enforcement are separate gates.

CandidateC chooses continuous tensor-bilinear nodal basis on each of the two
existing staggered scalar lattices: u_x nodes at primalXfaces, u_y at primalYfaces.
Their knot rectangles differ; partition cuts at the union of their knot planes
(halfcell grid) so every basis is polynomial on each integration piece. Restrict
these basis functions toΩ. The source is contained within both lattice hulls,
so partition of unity and affine coordinate reproduction hold onΩ/Γ. Boundary
support truncation outside those hulls is explicitly refused, never zero-filled.
An active coefficient's basis has positive measure support inΩ. A node can lie
outsideΩ, or have zero physical aperture and zero old h-width dual volume, while
its wider basis support has positive actual mass. All active restrictions are
retained; no floor/deletion/merging/extrapolated-value guess.

Velocity u_h=Σ_i U_i φ_i, pressure p_h constant independently on each positive
primal cut cell Q_c. Each vector basis has only one component. Define
  M_ij=rho∫Ωφ_i·φ_j,
  B_ci=∫Q_c divφ_i,
  K_ij=∫Ω2mu Dφ_i:Dφ_j,
  F_i=∫Ωφ_i·f +∫Γφ_i·g.
Mass is CONSISTENT FULL SPD, not the prior diagonal dual-lumped mass. Integrate
normal strain weights2mu and engineering shear weightmu, retaining full symmetric
stress. A=M/dt+K; solve[A,-Bᵀ;-B,0](U,p)=(M U*/dt+F,0). This is a new discrete
reference variant of the same continuum variational problem, not literal published
weighted finite differences, a bitwise aligned operator replacement or production
provider. Existing GridGeometry/indexing, immutable represented sources and
coupled architecture are reused, without changing their qualified owners.

For every active restriction, some scalar lattice knot cell has an open subset
inΩ. Its four localQ1polynomials are linearly independent on any open subset. If
a global combination vanishes onΩ, the coefficients of all such local polynomials
vanish; consequently every active coefficient is zero. Thus exact integral M is
positive definite. K is positive semidefinite by its integral-of-square form.
This proof concerns actual basis-integrated mass, not clipped midpoint dual mass,
and does not bound floating conditioning as a cut shrinks. Positive quadrature
must be polynomially exact and full-column rank or refuse uncertain arithmetic.

Free-surface pressure is absolute: Bᵀ1 is generally the physical normal-pressure
boundary load, not zero. Do NOT remove one pressure per component. Exact rank(B)
for the chosen represented corpus must be checked before solve; if full row rank,
positive A makes KKT nonsingular. No inf-sup/mesh-uniform stability theorem is
assumed for this particular unstabilized staggeredQ1/P0pair. A rank deficiency or
failed conditioning/accuracy check is a method finding; do not delete pressure,
introduce a gauge or add stabilization after observation under this version.

One reconstructed H1velocity supplies all physical fluxes and virtual work:
  (B U)c=∫∂Q_c u_h·n,
including both actual grid-face apertures and the physical source boundary.
Shared interior-face integrals cancel with opposite normals. Basis continuity
ensures unique trace; use the same piecewise-polynomial basis, not RT0 mixed
with a different shear trace. The physical interface flux is retained separately;
a frozen instantaneous free surface need not have zero normal velocity.
Natural incompressibility requires total closed-cell flux0, not zero interface
flux or unchanged free-surface geometry in a later transport step.

With r=M(U-U*)/dt+KU-Bᵀp-F, define E=.5UᵀMU. Symmetry and polarization give
 E-E*+.5(U-U*)ᵀM(U-U*)+dt UᵀKU
 =dt UᵀF+dt pᵀBU+dt Uᵀr.
The same physical trace gives UᵀF_boundary=∫Γg·u_h. Translations and rotations
are represented exactly; their divergence/strain vanish, so total linear/angular
momentum change comes only from actual F and residual. Consistent mass identifies
these discrete moments with integrals of the represented velocity, subject to
native integration error. These facts do not prove a nonlinear/free-moving water
step or physical impact response. Existing diagonal-mass Lean StokesWork cannot
be relabelled as a full-mass proof; either derive the finite Gram-mass identity
with explicit assumptions or keep this a reviewed mathematical derivation.

Prospective discriminator: prescribe analytic affine divergence-free u=A*x+c,
constant p0 and sigma=-p0I+mu(A+Aᵀ), with u*=u and g=sigma*n on each original
physical edge. Then divsigma0 and the SAME analytic boundary load, integrated
independently from geometry/basis, should give KU-Bᵀp=F. Do not construct F from
matrix actions on the desired solution. Nonzero p0=2Pa and shear distinguish
wrong pressure gauge, load sign and a normal-only/pressure-only lifting. A rigid
field with zero natural traction distinguishes boundary drag/support truncation.
These patch tests plus exact rank/work/flux are prerequisite evidence, not full
MMS convergence or a pressure/stress accuracy envelope.

First implementation boundary: CV1 assembles operators and physical trace only. It does not execute the proposed coupled timestep/KKT solve. The KKT equations above define the eventual mathematical target, not a delivered native step. Exact B rank, mass positivity/conditioning and affine traction identities are deciding prerequisite evidence. A 2Pa coefficient check of the affine nodal load is not labelled PDE pressure recovery.

Planar boundary convention: this is the two-dimensional Stokes problem measured per Href=1m depth. GridGeometry countsZ=1 is indexing/unit bookkeeping; there is no dynamic uz and no physical free end-cap boundary or three-dimensional closed-volume claim. Γ contains only polygon edges times Href. Do not infer unprescribed 3D normal end-cap traction from these planar tests.
