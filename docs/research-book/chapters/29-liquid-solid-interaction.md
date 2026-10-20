# 29 Liquid–solid interaction and three-dimensional collision meshes

This chapter is a researched synthesis added on 20 October 2026. Chapters 8,
19 and 20 establish the earlier geometry and work contracts; this chapter
connects them to the implemented solver and subsequent negative results.
[Appendix G](../appendices/G-paper-audit-and-corrections.md) provides the
claim-to-source audit. All implementation statements refer to main `718469b`
unless a separate, pinned research branch is named. No new solver is introduced.

## Contact, boundary conditions and coupling

A collision query asks whether geometry intersects and where contact first
occurs. A fluid boundary asks which volume is available, which samples can
communicate, what normal flux and tangential stress are allowed, and which body
receives the reaction. Passing the first test answers only part of the second.

Let \(n_f\) point out of the fluid into the solid, and let
\(\sigma_f=-pI+2\mu D(u)\) be fluid Cauchy stress. The kinematic condition is
\((u-u_s)\cdot n_f=0\). No-slip sets the full velocity equal to \(u_s\);
free-slip instead imposes zero tangential traction. A Navier law relates
tangential relative speed to traction and needs a slip length or friction
coefficient with declared units. Rheon's mesh point-contact response implements
none of these calibrated tangential stress laws.

The following signs follow directly from traction and action/reaction. The force
on the fluid is \(\int_\Gamma\sigma_f n_f\,dA\). With solid-outward normal
\(n_s=-n_f\), the load on the body is

\[
 F_s=\int_\Gamma\sigma_f n_s\,dA,
 \qquad T_s=\int_\Gamma(x-c)\times(\sigma_f n_s)\,dA.
\]

Thus pressure contributes \(-pn_s\), while viscosity supplies the remaining
traction. Matching normals, quadrature points and moment arms is essential.
The same interface transfer must appear with opposite signs in the fluid and
body momentum ledgers. These continuum identities are requirements, not measured
Rheon mesh loads.

## Admit a solid or admit a surface

A volumetric solid needs consistent inside/outside semantics. Check finite
positions, nondegenerate triangles, index validity, closure, manifold edges and
vertex neighborhoods, orientation, duplicate faces, self-intersections and
component nesting. Watertight edge pairing alone does not establish geometric
validity. CGAL's developer manual explicitly separates manifold combinatorics,
self-intersection tests and closed-mesh side predicates [A1]. Repair can move
the boundary; retain its geometry change rather than silently welding a gap.

Rheon's [`CollisionMesh`](../../../src/mesh_collision.rs) uses a narrower
`ConvexSolid` contract: connected indexed closure, coherent paired-edge winding
and convex halfspaces. Globally reversed winding is accepted. Its separate
`TwoSidedSurface` policy allows open or concave surfaces without inventing an
enclosed volume. Neither is a generic self-intersection repair algorithm.
[`mesh_import.rs`](../../../src/mesh_import.rs) applies OBJ scale or glTF
hierarchy/placement transforms, including reflection handling, before collision
admission. Mesh units, axes, pose and tolerances belong to the physical problem.

A signed distance field stores distance magnitude plus a sign policy. Closest
triangle distance alone is unsigned. Sampling a valid signed distance at cell
centers still need not detect a thin sheet, narrow gap or two disconnected fluid
pieces inside one cell. SDF interpolation, reinitialization and voxel resolution
add errors independent of the original triangles. Fractions and connectivity
therefore need their own intersection construction; a pretty isosurface cannot
serve as their oracle.

## Broad phase, narrow phase and tunneling

A broad phase cheaply rejects disjoint bounds. Rheon builds an AABB BVH over
static triangles; traversal selects candidate facets for narrow-phase
segment/triangle queries. Convex containment uses separate halfspace tests.
A BVH accelerates queries without
specifying robust predicates, shell thickness or temporal accuracy. The CGAL
AABB documentation distinguishes a traversal's arbitrary encountered hit from
the nearest hit along a ray [A2]. Contact must select the earliest relevant
parameter and handle edge/vertex ties, grazing and degenerate configurations.
Rheon's finite-tolerance triangle predicate uses a plane band and edge
halfspaces; this is not an exact zero-thickness geometric certificate.

Endpoint-only containment misses an outside-to-outside path through a wall.
Rheon's particle update sweeps the old-to-proposed **center segment** against
actual static triangles. At the first contact it retreats by a numerical
clearance, removes normal velocity and discards unused travel; tied nonparallel
contacts stop velocity. Kernel support and display radius can still overlap
the wall. This is continuous intersection of that declared linear point path,
not a swept sphere, a curved trajectory proof, or collision against a rotating
triangle during the interval.

Brochu, Edwards and Bridson's primary CCD work treats geometric degeneracies and
robust moving-primitive intersection [A3]. Its guarantees belong to its stated
trajectory model and predicates. A future rotating/deforming Rheon collider
would need swept bounds and relative motion in narrow phase; checking only the
final object pose is insufficient. Substepping reduces travel but needs an
error/contact bound before it can establish absence of tunneling.

## Triangle geometry to conservative fluid operators

Define a cut fluid region \(\Omega_i\), volume \(V_i\), open Cartesian face
area \(A_f\), and physical solid boundary patch. Integrating divergence gives
the sum of open-face flux and wall flux. For stationary walls, a single shared
interior-face integral enters its neighboring cells with opposite signs. A cut
cell containing two disconnected pockets cannot in general use one pressure
unknown. Reject, refine, or represent both regions explicitly.

The derivation in Chapter 3 uses head-positive incidence \(B\), diagonal areas
\(S\), pressure distances \(L\), densities \(R\), cell volumes \(V\), and
known outward boundary flux \(Q\):

\[
 D(u)=V^{-1}(Q-BSu),\quad G=L^{-1}B^T,
 \quad K=BSR^{-1}L^{-1}B^T.
\]

The two-point gradient requires the declared normal sample separation; arbitrary
cut centroids are not automatically orthogonal. Positive coefficients prove a
quadratic energy property, not geometric consistency. Batty, Bertails and
Bridson explicitly discuss alternative staggered-volume approximations in
their variational pressure construction [S7, Section 2.1]. Rheon's area-lumped
\(\rho A_f\ell_f\) is a model choice needing qualification, not evidence that
their research contains a dimensional typo.

Rheon's [`StaticObstacleGeometry`](../../../src/static_obstacle.rs) computes
shared volumes/openings/components for one admitted axis-aligned box.
[`StaticObstaclePressure`](../../../src/obstacle_pressure.rs) consumes them.
This is distinct from the arbitrary static triangle collider. The reduced
stationary shear channel and opt-in aligned Stokes experiment have separate
boundary and symmetry assumptions; neither turns imported triangles into a
general liquid free-surface/stress domain.

## Small geometry and small cells

For explicit donor transport, an available-volume condition has the local scale
\(\Delta t\sum_f A_f\max(u_f\cdot n_f,0)/V_i\leq1\).
As \(V_i/A_f\) shrinks, this can become much stricter than a nominal-grid CFL.
The physical projection error is likewise amplified:
\(D(u^+)=-\Delta t V^{-1}(b-Kp)\). A tiny integrated residual can leave large
divergence in a tiny cell. These are derived scaling facts, not universal
timestep constants for arbitrary cut methods.

Basilisk's original embedded-boundary implementation documents and handles
tracer overflow with redistribution [A4]. Importing that idea into Rheon would
still require conservation, positivity, barrier-connectivity and refinement
checks. Merging cells, dropping support, inflating volumes and flooring apertures
alter different parts of the method; none is a consequence of merely using a
BVH or SDF.

The private CV1/CV2 progression illustrates the distinction. CV1 uses integrated
restricted velocity bases, full Gram mass and symmetric strain; it is not the
old diagonal cut-pressure operator. CV2 improves the numerical solve of those
unchanged blocks. On 24 fixed planar cases it passes its unchanged accuracy
gate, while six unscaled square failures and mass conditioning about
\(6.53\times10^{11}\) remain retained. Conditioning treatment does not establish
a cut-uniform stability theorem, PDE convergence or dynamic geometry support.

## Particle boundary support

An SPH particle near a wall loses fluid neighbors. A position pushout does not
restore density support or pressure traction. Possible methods include sampled
boundary/ghost particles with derived volumes and velocities, semi-analytical
kernel boundary integrals, and precomputed density/volume maps. Each must match
the density/volume estimate, its derivative, wall reaction and viscosity. A
solid wall is not a liquid free surface: supplying atmospheric pressure at both
would impose different physics.

Mayrhofer and collaborators analyze the distinction between continuous and
discrete skew-adjointness for semi-analytical SPH walls [A5]. Their result does
not give arbitrary particle discretizations exact energy conservation. Koschier
and Bender's author abstract describes precomputed narrow-band density
contributions for static/rigid boundaries [A6]; that abstract supports a method
comparison, not a verified drop-in formula for Rheon. The full density-map PDF
was unavailable to this audit's retrieval tool, so no detailed equation is
attributed to it here.

Rheon's current density and pair-force loops do **not** occlude neighbors across
mesh triangles. The retained thin-wall audit measures actual cross-barrier
kernel contributions even though center trajectories do not penetrate. Removing
only density neighbors while keeping the old pressure force would also change
the potential/derivative relationship. PR121's wall adaptations fail the fixed
hydrostatic/shear gates; PR123's work-consistent repair still fails consistency
tests. They are research counterexamples, not enabled boundary methods.

For a proposed boundary method, first test a resolved planar wall: constant and
affine pressure response, static hydrostatics with matched EOS initialization,
Couette/Poiseuille traction, and equal/opposite reactions. Repeat with perturbed
particles and truncated support. Verify one coherent operator before increasing
mesh complexity. [Appendix G](../appendices/G-paper-audit-and-corrections.md)
records why the later connected material-interface reference is a different
discretization rather than an SPH boundary patch.

## Moving boundaries and rigid response

For a rigid body, \(u_s(x)=V_s+\omega_s\times(x-c)\). On a fixed Cartesian
cell with moving solid cut and impermeability, Reynolds transport and integrated
incompressibility give

\[
 \dot V_i=\int_{\Gamma_{s,i}}u_s\cdot n_f\,dA,
 \qquad \dot V_i+\sum_{\mathrm{fixed\ open\ faces}}
 A_fu_f\cdot n_f=0.
\]

These independently derived identities expose the swept-volume term. A uniform
field transported on a moving mesh must also satisfy the corresponding discrete
geometric conservation law. Newly uncovered regions require a conservative
mass/momentum policy. A closed source-free incompressible domain must preserve
total fluid volume; a sealed inward piston is incompatible without another
flux, a source term, or a changed compressible model.

With prescribed motion, \(\int(\sigma_f n_f)\cdot u_s\,dA\) is actuator power
into the fluid when velocity matches the wall. With a dynamic body, its load
power is the negative of that interface term. Accumulate impulse and torque at
the same interaction point; then test linear and angular momentum including
external forces. Pressure and viscosity must both be included where modeled.

A useful finite specification combines fluid and body velocity into \(z\),
positive mass/inertia into \(M\), and a shared constraint into \(Jz=q\).
From \(M(z^+-z^*)+J^T\lambda=0\), polarization gives

\[
 E^+-E^*+\tfrac12\|z^+-z^*\|_M^2=-\lambda^Tq.
\]

This derivation assumes frozen geometry, exact equations and a feasible
constraint. It proves neither geometric assembly nor floating-point accuracy.
Batty et al.'s coupled pressure/body formulation [S7, Section 3] motivates the
adjoint load/velocity pairing. Light-body stability needs method-specific
qualification; simultaneous coupling or qualified iteration are candidate
remedies. An explicit reaction computed after a fluid-only solve
does not inherit that formulation's stability.

Rheon has separate translating-surface queries, rigid traction/impulse helpers
and a reduced two-way tangential plate experiment. It does not yet compose these
into a moving/rotating triangle-mesh liquid boundary with swept volumes and a
shared pressure/viscous wrench. Static collision tests cannot qualify that step.

## Practical method choice and verification

| Representation | Useful role | Main obligation for Rheon |
| --- | --- | --- |
| Triangle mesh + BVH | Precise static contact queries and retained source geometry | Earliest-hit robustness; current point response has no hydrodynamic wall support |
| Binary voxels | Simple pressure topology | Staircase/offset error; unresolved thin sheets or gaps |
| SDF/narrow band | Fast distance/normal queries and classification | Verified sign, bandwidth, interpolation, feature resolution; derive fractions separately |
| Embedded cut geometry | Shared fluid volume, flux, stress and load assembly | Connectivity, sample consistency, small cells, conditioning and convergence |
| Ghost/sampled particles | Boundary contribution near SPH support | Quadrature, wall pressure/velocity and paired reaction; qualify perturbed sampling |
| Semi-analytical integrals/maps | Continuous boundary support | Matching derivatives, map resolution, motion assumptions and work consistency |
| Conforming material mesh | Connected Lagrangian volume and stress | New deformation/Jacobian/remap/solver family; not particle surface extraction |

Guendelman et al.'s author record specifically addresses infinitesimally thin
shells using visibility-aware fluid operations and incompressibility [A7]. It
supports treating cross-wall communication as a boundary problem, rather than
making the collider thicker until leakage becomes invisible.

Verification should proceed through independent geometric distances/intersections,
closed-cell flux and adjoint tests, constant/affine/rigid patches, hydrostatic
rest, shear loads, and spatial/temporal manufactured solutions. Add subcell
offsets, reversed orientation, thin oblique baffles, narrow gaps and disconnected
pockets. Moving tests must check uniform-field/GCL preservation, translated
displacement, rotating-wall work, neutral buoyancy and light-body stability.
Keep solver residual, actual field error, geometric error and physical
observations separate. These are proposed benchmarks, not new passing results.

Kleefsman et al.'s original dam-break/obstacle experiment provides a later
physical benchmark [A8]. A quantitative comparison needs matching dimensions,
initial state, boundary operators, probe locations and measured time series with
uncertainty. Rheon's retained release specimen is a feasibility/conservation
exercise, not that experiment or impact-pressure validation.

Particle-to-surface reconstruction estimates a visible liquid/air boundary.
Collision geometry represents solids. A material mesh carries connected
Lagrangian degrees of freedom. Keeping those three representations explicit
prevents a new renderer, a swept contact test or a successful 1D material
reference from being reported as a validated general liquid solver.
