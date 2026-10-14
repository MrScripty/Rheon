# From oriented mesh faces to pressure–velocity coupling

This completes the bounded reconstruction/coupling research slice built on the
[shared-flux controls](mesh-faces-to-collocated-flux.md) and
[component/gauge study](pressure-compatibility-and-gauges.md). It defines a path
for an engineering implementation; the browser mechanisms do not implement it.

## Mesh input contract before a pressure equation

A supported reader needs explicit length units, finite coordinates, an accepted
cell/face topology, oriented owner/neighbour incidence, positive cell volumes,
face centroids and area vectors, boundary labels and connectivity. Reject repeated
indices, inverted/degenerate cells, zero-distance reconstruction samples and
unsupported nonmanifold topology before publishing a mesh. A watertight collision
surface alone is not a valid volume mesh or a complete pressure boundary model.
Boundary patches need semantic conditions, not merely display colours.

One internal integrated flux must be shared by its two cells, including in
momentum transport. Reversing the orientation reverses its signed flux, preserving
the physical divergence. Constant-field and affine-field tests exercise distinct
properties: partition of unity alone cannot establish affine reproduction.
For geometric conservation under motion, swept face volumes and cell-volume
changes need a compatible time ledger; static incidence cancellation is not that
proof. Imported moving meshes, remeshing and geometric conservation remain later
production work. The book's current native geometry is deliberately restricted.

No mesher, reader or sparse-solver dependency is selected by this research slice.
CGNS/VTK format research and an Apache-compatible parser need independent
exact-version, transitive and data-licence review before implementation. Gmsh
binaries/SDKs/wrappers and GPL implementation copying remain excluded.

## A completely worked skew reconstruction

Use p(x,y)=a x+b y+c Pa, positions in metres, c=2, and samples
L=(-1/2,0), R=(1/2,s), T=(-1/2,t), with t>0. The face patch has centroid (0,0)
and unit normal n=(1,0). This is a sample/reconstruction configuration, not a
claim to have assembled a closed polyhedral mesh. The transverse sample gives
b_hat=(p_T-p_L)/t, followed by a_hat=p_R-p_L-s b_hat. Both are exact for this
specified affine field. The two difference equations have determinant t.

The centre vector is d=(1,s). A two-point contribution based on d/(d dot d)
gives g_two=(a+b s)/(1+s²). The missing nonorthogonal part is
(a_hat,b_hat) dot (n-d/(d dot d)). Adding it gives a exactly. This correction
uses a genuinely reconstructed gradient; two scalar cell samples alone would
not determine both planar components. General least-squares stencils require
rank/conditioning checks and are not qualified by this three-sample identity.

At s=1/2,a=b=t=1, samples are (3/2,3,5/2), reconstructed gradient is (1,1),
g_two=6/5 and correction=-1/5 Pa/m. The mean of the first two pressures is
9/4 Pa at their midpoint (0,1/4), not at the face centroid. Subtracting b_hat s/2
gives the exact face pressure 2 Pa. Nonorthogonality and face skewness therefore
have related but distinct corrections. The GUI compares actual sample locations,
normal-gradient arrows and face values with fixed scales; no velocity is implied.

## What a collocated algorithm must specify together

For momentum H u + Gp=f and continuity C phi=Q, define the diagonal response,
off-diagonal contribution, cell-to-face interpolation, pressure face difference
and nonorthogonal correction as one coupled construction. The PR60 checkerboard
probe explains why naive averaging of cell velocities is insufficient. Its compact
formula alone does not specify Rhie–Chow interpolation for variable coefficients,
relaxation, transient inertia, boundaries or strongly skew meshes.

A steady segregated iteration assembles and solves a momentum predictor, forms
a pressure-correction system from the same face response, fixes compatible
gauges, corrects conservative face fluxes and updates cell velocities and pressure
with a documented relaxation convention. Recompute continuity and momentum
residuals against the same accepted state. For a transient PISO/PIMPLE-style
sequence, also specify time discretization, inner pressure corrections, outer
coefficient updates and which flux defines transport. Neither residual history
nor algorithm name proves time accuracy or nonlinear convergence.

Greenshields and Weller's developer-authored [steady-state discussion, §5.12](https://doc.cfd.direct/notes/cfd-general-principles/steady-state-solution)
provides the momentum/pressure/flux iteration context; their [gradient discussion,
§3.15](https://doc.cfd.direct/notes/cfd-general-principles/gradient-discretisation)
explains reconstruction and geometric sampling issues. Read 2026-10-13. This
chapter's numerical networks and sample formulas are independently derived.
Original Rhie–Chow, Patankar–Spalding and Issa full texts were not accessible in
this workflow; no reading of those full papers or complete reproduction of their
algorithms is claimed. That limits a production design, not the finite examples.

## A reduced segregated iteration that actually executes

Two parallel paths connect the same two regions. Nondimensional velocities are
u=(u1,u2), pressure difference is p, and the deliberately specified linear model is

    H = [[2,-1],[-1,2]],    G = (1,1),    f = (1,0),
    H u + G p = f,         u1+u2=0.

H has positive eigenvalues 1 and 3. The exact constrained solution is
u=(1/6,-1/6), p=1/2. This network has no claimed geometric Navier–Stokes
consistency; its boxes and paths are a diagram of these unknowns.

Start from u=0,p=0. Form the diagonal momentum predictor
u_star=((1+u2-p)/2,(u1-p)/2). Set p_prime=u_star1+u_star2 and
u_hat=u_star-(p_prime/2)(1,1). Then update
u_next=(1-alpha)u+alpha u_hat and p_next=p+alpha p_prime.
This is the exact iteration in the browser, inspired by segregated correction;
it is not presented as the general SIMPLE algorithm. Since both old and corrected
u have zero sum, all iterates preserve continuity. Momentum need not be solved.

Writing u=(x,-x), the errors satisfy

    x_next-1/6 = (1-3 alpha/2)(x-1/6),
    p_next-1/2 = (1-alpha)(p-1/2).

Thus x_n=(1-(1-3alpha/2)^n)/6 and p_n=(1-(1-alpha)^n)/2. At alpha=1,n=4,
x=5/32 and p=1/2: continuity is zero but momentum residual is (1/32,-1/32).
At alpha=3/2 the velocity-error factor is -5/4 and its squared magnitude is
25/16>1, while the pressure-error factor is -1/2. A pressure trace or continuity
alone would hide the growing velocity error. Iterations are counted work, not
physical time or a speed comparison. General convergence requires a separate
operator/splitting analysis.

## Exact contracts and admission evidence

[CouplingStudy.lean](../../../proofs/Rheon/CouplingStudy.lean) checks affine sample
identities, the nonorthogonal and skew corrections, conservative correction,
error recurrences, a relaxation-factor bound, the exact network solution and
the overrelaxation counterexample. Division assumptions are explicit. It does
not prove general mesh conditioning, full Rhie–Chow consistency, SIMPLE/PISO
convergence, JavaScript/Rust equivalence, IEEE behavior or runtime performance.

All 48 reconstruction states and 15 network states have independent rational
closed-form checks. Browser checks separately inspect rendered sample locations,
face patches, actual gradient/velocity arrows, controls and scroll presets on
three viewport sizes. Source-bound kernel status and qualification results stay
separate from human visual review and from production/native admission.
