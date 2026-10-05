# Selected fitted-height liquid formulation

This research successor selects one geometry and derives transport, pressure and
viscosity together. It does not enable the refused advancing-viscosity API or
change any frozen Rust source, Jacobi fixture, receipt or negative result. The
reference assembly and conditional Lean statements live in
`evidence/fitted-height-formulation/`. They qualify a semidiscrete formulation and
finite algebraic contracts, not an advancing liquid simulator.

## Model and primary sources

Select a two-dimensional, non-overturning polygonal height field extruded through
a periodic width b. In physical coordinates the liquid is

    Ω(t) = {(x,y,z): x periodic over L, 0 < y < H(x,t), z periodic over b}.

H is continuous and piecewise affine, positive, with ordered top abscissae.
Retain all three components U=(u,v,w); every field is independent of z. Density
ρ>0 is constant and viscosity μ≥0 is constant. The bottom is impermeable with
natural zero tangential traction. The cap is material and its atmospheric
traction is zero for the relative pressure π=p−p_atm. There is no surface tension,
contact line, adhesion, density jump, air solve, overturning, breakup or adaptive
remeshing in this slice. Horizontal periodicity is an explicit model choice.

Batty and Bridson's [variational Stokes formulation](https://arxiv.org/abs/1010.2832)
provides the reference for coupled pressure/strain work and natural free-surface
traction, rather than independent pressure and diffusion boundary rules. Its
weighted MAC matrices are not the collapsed-slab matrices used in Rheon's static
bridge. Its section 10 uses semi-Lagrangian transport, so it supplies neither our
conservative transport nor a full-vector moving-volume proof. Its table 9.2 also
does not establish pointwise pressure/stress convergence. The
[original implementation](https://github.com/elrnv/stokes-houdini/blob/master/SIM/SIM_Stokes.C)
uses sampled geometry and minimum weights; those floors and density clamps are
not imported. The earlier
[variational viscosity paper](https://www.cs.ubc.ca/~rbridson/docs/batty-sca08-viscosity.pdf)
supports the full symmetric strain and traction pairing. Basilisk's primary
[conserving implementation](https://basilisk.fr/src/navier-stokes/conserving.h)
is an example of momentum carried by actual phase transport; it does not supply
our fitted discretization.

For the fitted velocity space use a Powell–Sabin split of a triangular macro
mesh. [Guzmán, Lischke and Neilan](https://arxiv.org/abs/1904.05466) give the exact
sequence and continuous piecewise-linear velocity/divergence-image construction.
This research adds boundary trace constraints and nodal mass lumping. It does not
inherit the paper's uniform inf-sup result for those modifications. The transport
and moving-mass work identities below are derived here, not attributed to that
paper or to Batty–Bridson.

## One geometry, one accepted physical state

The macro mesh fits the graph and the bottom. Each macrotriangle has an interior
point. Neighboring interior points determine the shared edge split by their
line intersection; boundary edge splits are midpoints. Connecting the interior
point to the vertices and edge splits gives six microtriangles. Reject exterior
intersections, nonpositive areas, loss of graph ordering, inadequate thickness or
mesh quality. No positive weight is manufactured by a floor.

Continuous P1 nodal basis functions N_i on the microtriangles are nonnegative and
sum to one. All three velocities use them. Periodic nodes are identified. A cap
or bottom edge midpoint velocity is the average of its macro endpoints, so its
physical trace stays affine on that macro edge. Bottom normal velocity is zero.
Write the fixed-topology constraints as U=R Z. R is a constant linear map while
that topology and its identifications persist. The reduced mass Rᵀ M R is sparse
and generally not diagonal. These constraints are not a second carrier owner.

The geometry moves with a continuous piecewise-linear mesh velocity a_h. On the
cap a_h,xy=u_h,xy. On the bottom a_h·n=u_h·n=0; bottom tangential mesh speed may be
zero. Interior macro positions follow a declared bounded extension; split points
are recomputed from the macro positions, and their velocities are the actual
derivatives of that construction. Arbitrary independently supplied new heights
are not an allowed transport input. All operators use this same physical mesh.

The median dual C_i partitions each incident microtriangle into three equal-area
parts, using its barycenter and edge midpoints. Thus

    m_i = ρ b |C_i| = ρ b Σ_{T incident i} |T|/3 = ρ b ∫ N_i dA,
    M = diag(m_i) ⊗ I₃,
    T(U,m) = ½ Σ_i m_i |U_i|².

This is a declared lumped kinetic energy, not the exact consistent-FE integral of
|u_h|². Each component's mass partition sums to the same liquid mass; the third
component and the free-cap normal velocity are never omitted.

A future opt-in implementation must make this fitted graph/mesh the sole
accepted phase geometry for this model. Cartesian fractions and MAC samples are
bounded derived exports. It must replace the legacy geometry authority, rather
than keep two independently publishable accepted liquid states. Candidate mesh,
velocity, pressure and derived exports commit together after cancellation and
all numerical gates. Until that implementation exists, the current refused API
continues to refuse.

## Conservative semidiscrete motion and full momentum

Let u_h=Σ_i N_i U_i. Define the pressure space as the exact divergence image of
the admissible horizontal velocity space:

    Q_h = div(V_h,xy),
    B_{q,i,d} = −b ∫ q ∂_d N_i dA  (d=x,y),    B_{q,i,z}=0.

Use an independent basis of this image, including its permitted constant mode.
The reference fixture constructs that basis by exact elimination; an actual
bounded implementation needs a topology-aware basis and rank/conditioning
qualification. An unconstrained P0 pressure per microtriangle would include
spurious modes. Do not pin an arbitrary pressure gauge: the free normal cap
makes a constant pressure physically consequential. In this fixture the
admissible field (0,y,0) has divergence one, so a constant is in Q_h.

BU=0 implies div u_h=0 on every microtriangle: choose q=div u_h in the constraint
and obtain its weighted squared norm zero. This inference is exact-real and
exact-assembly; a finite solver residual needs a quantified gate. It makes the
actual fluid flux through every dual cell sum to zero.

For each oriented internal dual face store once

    f_ij = ρ b ∫_{∂C_i∩∂C_j} (u_h−a_h)·n_i ds,    f_ji=−f_ij.

Periodic boundary faces pair; relative flux is zero at cap and bottom. Reynolds'
identity for the actual moving dual geometry gives the geometric conservation law

    m'_i + Σ_j f_ij = 0.                                      (GCL)

These are physical face integrals from the same velocity and mesh motion. Neither
node mass marginals nor a valid geometry stamp establishes that provenance.
There is no unrelated donor-cap velocity API.

Carry all three momentum components on that same face:

    g_ij = f_ij (U_i+U_j)/2 + |f_ij| (U_i−U_j)/2,
    c_i = Σ_j g_ij,
    D_adv = ½ Σ_{unordered ij} |f_ij| |U_i−U_j|² ≥ 0.

This donor flux is antisymmetric and first-order dissipative. Removing its second
term gives the central semidiscrete energy-preserving flux, with a separate
stability problem; this milestone selects donor flux. It does not prove spatial
convergence for distorted dual meshes. Shared fluxes, rather than independent
component remaps, conserve all vector momentum internally. For admissible
horizontal/third translations, internal strain, pressure and trace reactions
have zero resultant. Bottom normal reactions and external body loads remain
explicit exchanges; total normal momentum is not claimed constant with a wall.

## Full strain, pressure and traction on that geometry

With ∂_z=0 the five nonzero independent engineering strain entries are

    E U = (∂_x u, ∂_y v, ∂_y u+∂_x v, ∂_x w, ∂_y w).

On each microtriangle assemble

    W_T = μ b |T| diag(2,2,1,1,1),    K=Eᵀ W E.

Hence Uᵀ K U=∫_Ω 2μ D(u):D(u) dV, including both third-component shears and
both in-plane normal strains. The full symmetric stress is σ=−π I+2μD. Use
exactly the transpose of the integrated negative divergence for pressure force:

    Rᵀ [ (M U)' + c(U,f) + K U + Bᵀ π − F ] = 0,    B U=0, U=R Z.  (1)

F contains the assembled body load and any prescribed boundary traction. The
weak integration-by-parts identity makes σn=0 the natural cap condition. There
is no separately imposed π=0 or zero shear derivative at that cap. The bottom
normal test is removed and its reaction is recorded; bottom tangential traction
is naturally zero. This is a projected weak traction condition. The restricted
cap trace does not certify pointwise stress accuracy or import table 9.2's
convergence results.

Dot (1) with U, using the constant R, pressure adjoint and GCL. Pairwise flux work
is

    Σ_i U_i·c_i = −½ Σ_i |U_i|² m'_i + D_adv.

Together with T'=Σ_i U_i·(M U)'_i−½Σ_i |U_i|²m'_i, this gives

    T' = UᵀF − D_adv − UᵀK U.                                (2)

A nonzero reduced equation residual contributes its velocity work. For a full
nodal residual r_mom, divergence residual r_div=BU, and
r_mass=m'+Σf, the corresponding identity has the additional terms
Uᵀr_mom−πᵀr_div−½Σ_i|U_i|² r_mass,i. A reaction orthogonal to R does no work.
Roundoff, quadrature and true linear residuals must be measured separately.
Positive semidefiniteness alone establishes none of these residual bounds.

## Finite algebra contract, with temporal provenance still open

For a prospective finite step require integrated face masses
F_ij=∫_{t_n}^{t_{n+1}} f_ij dt from one declared physical space-time mesh path,
and require

    m_i^{n+1}−m_i^n + Σ_j F_ij=0,  F_ji=−F_ij.

This is an additional temporal contract, not provided by an endpoint quadrature
or by fitting fluxes to new masses. The material cap, strong divergence during
the chosen path, and its endpoint force solve must be time-discretized together.
This milestone does not select or qualify that nonlinear temporal integrator.

Given physically qualified F and positive old/new masses, implicit donor
transport of each component is

    A U_adv = M_old U_old,
    A_ii=m_new,i+Σ_j max(F_ij,0),    A_ij=min(F_ij,0) (j≠i).

Then A1=m_old and 1ᵀA=m_newᵀ. Its strict row diagonal dominance margin is m_old,i,
and its off-diagonals are nonpositive; under the stated positivity conditions
its inverse is nonnegative. To see the sign, if Ax≥0 had a negative minimum x_i,
then (Ax)_i=m_old,i x_i+Σ_{j≠i} A_ij(x_j−x_i)<0, a contradiction.
W=diag(m_new)A⁻¹diag(m_old) therefore has nonnegative
entries and old/new mass marginals. This same map carries all components because
they share this fitted dual partition. A production solve must not materialize
that dense map. Constants, momentum and convex bounds follow for pure transport;
viscous/pressure motion has no componentwise convex-bound claim.

The exact changing-mass transport identity is

    T(U_adv,m_new)−T(U_old,m_old)
      +½Σ_i m_old,i |U_adv,i−U_old,i|²
      +½Σ_{ij} |F_ij| |U_adv,i−U_adv,j|² = 0.                  (3)

On a given endpoint mesh the variational backward-Euler force solve is

    Rᵀ[(M_new+Δt K)V + Δt Bᵀπ − M_new U_adv − Δt F]=0,
    BV=0, V=RZ.

It satisfies

    T(V,m_new)−T(U_adv,m_new)+½||V−U_adv||²_M
      +Δt Vᵀ K V = Δt VᵀF,                                 (4)

conditional on zero work residual. Equations (3) and (4) are consistent algebra
on their declared geometry; they do not establish that an arbitrary composition
advances the same physical cap or that a particular time discretization is
accurate. The finite reference witness explicitly uses m_new=m+Δt m' and frozen
instantaneous face fluxes. It tests (3), not a finite physical mesh trajectory.

## Actual consistency evidence and proof limits

The bounded exact-rational reference uses 8 macrotriangles and 48 microtriangles
under the periodic varying graph H=[1,5/4,1,3/2,1], with bottom nodes fixed and
cap nodes translating horizontally at 1/4. This is a genuine instantaneous
material-surface motion, H_t+u H_x=0, with a differentiated Powell–Sabin mesh.
Liquid area is 19/16 and each component's total mass is 57/16 at ρ=3, b=1. There
are 32 periodic nodal masses, 22 nonzero mass rates and 76 nonzero shared fluxes.
All GCL residuals are exactly zero. The pressure image has rank 33 with 44
horizontal velocity unknowns; its assembled integrated B also has row rank 33.
This is one fixture's rank, not a mesh-family inf-sup bound.

Constant full-vector velocity has zero divergence and strain and is carried
without changing velocity. A nonconstant third component with the actual
boundary trace restriction uses the reduced mass solve for its instantaneous
rate: third momentum rate is zero and (2) holds exactly with positive advection
and strain dissipation. An unrestricted affine incompressible full-vector patch
has strain power 475/512 and exactly balances assembled K U+Bᵀπ against its
prescribed boundary traction. That affine patch is not a periodic homogeneous
free-surface flow. Rigid rotation has zero strain. Pressure adjoint work is exact.
A wrong-flux circulation preserves every mass marginal but fails direct physical
face-flux recomputation; this witnesses why provenance cannot be replaced by
marginal checks.

The separate smooth instantaneous flat-cap test uses
ψ=a sin(kx) f(y), f(y)=y+c y³, c=−k²/(6+k²H²),
(u,v,w)=(ψ_y,−ψ_x,constant). It is incompressible, impermeable/free-slip at the
bottom and side walls, and has zero top tangential traction because
f''(H)+k²f(H)=0. Its nonuniform top normal velocity requires
π(x)=−2μ a k cos(kx) f'(H). At H=1,k=π,a=.1,μ=.5 the pressure amplitude is
0.27198533969818417; setting pressure to zero leaves that normal-traction error.
The predictor is derived as u*=u−Δt(μΔu−∇π)/ρ, not set equal to the target field.
The sampled backward-Euler residual is below 3.3e−14. This is an analytic
instantaneous benchmark with a flat cap, not a repeated moving-surface exact
solution or a numerical pressure convergence study.

Lean checks conditional exact-real finite identities corresponding to the
triangle mass partition, shared flux work, changing-mass kinetic derivative,
residual work, full-strain shear factors, adjoint work, donor transport and BE
force work, plus the cleared-denominator analytic shear relation. It does not
prove Reynolds' theorem, finite-element assembly, matrix inverse positivity,
uniform stability, existence of a moving solution, IEEE arithmetic or Rust.
The exact Python assembly independently checks selected geometry/operator
contracts; it is research code with bounded dense elimination, not production
memory accounting. A labelled figure shows the analytic graph translation and
instantaneous manufactured traction only, not native time evolution.

## Replacement, reuse and the next implementation slice

Replace the collapsed slab's mass/divergence, ghost-pressure rule, static-box
strain stencil, tangential-only remap and canonical bottom-fill reconstruction
as physical authority for this opt-in formulation. Their frozen contracts remain
valid in their own scope. Reuse the existing candidate/accepted transaction,
cancellation pattern, geometry stamps, checked normal arithmetic, explicit
residual/work ledgers and bounded solver kernels where their contracts apply.
Do not reuse their physical quadrature by naming it compatible.

The next slice is a fixed-topology fitted workspace and exact-contract assembly:
constructor-declared macro/micro/node/dual-face/pressure-basis and sparse nonzero
caps; one candidate mesh; one accepted mesh; owned scratch included in the public
budget. Assemble masses, derivatives, shared dual fluxes, Q/B and full E/W/K from
that candidate mesh. Qualify affine traction, rigid rotation, mass partition,
strong-divergence image, graph-translation GCL, residual-work and corruption
rejection natively. First publish it as assembly/instantaneous diagnostics with
no physical-time advancement. Only after this qualification select a compatible
space-time geometry integrator and qualify equations (3)/(4) with its actual
trajectory, rejection and paired publication. A rollback snapshot of arbitrary
history, an externally accepted new cap, or a parallel phase owner is not part of
that plan.

Failures include thin/sliver elements, split-intersection failure, graph loss,
pressure-rank/conditioning loss, unresolved solve or GCL residuals, nonfinite or
nonzero subnormal checked intermediates, workspace-cap exhaustion and
cancellation. Preserve the accepted mesh and all accepted flow fields on any of
these. Mesh-family error estimates, pointwise traction, free-surface temporal
accuracy, production budget numbers and native advancing/render evidence remain
unqualified. The existing advancing-viscosity refusal therefore stays intact.
