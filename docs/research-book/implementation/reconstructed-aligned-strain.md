# Reconstructed aligned-box symmetric strain

This is a new derivation for one exactly grid-aligned retained internal box,
with at least one complete fluid cell between every box plane and the sealed
outer domain. The obstacle is stationary and no-slip; outer walls are sealed
and free-slip. The only unknowns are the existing pressure-active binary64 MAC
faces, with the pressure operator's stored inertia `m_f = rho*A_f*d_f`.
No recovered implementation or historical test receipt qualifies this work.
PR25's held geometry-band case is outside this construction.

## Primary sources and the new choices

[Batty and Bridson (2008), original author PDF](https://www.cs.ubc.ca/~rbridson/docs/batty-sca08-viscosity.pdf),
p. 5, equations (9)–(11), defines the symmetric deformation tensor and its
viscous dissipation. Section 5.1 and Figure 5, pp. 5–6, place normal stress at
cell centers and shear stress at MAC edges, with fluid-volume quadrature.
Section 5.2 discusses combining prescribed traces and variational boundaries.

[Batty, Bertails and Bridson (2007), original author PDF](https://www.cs.ubc.ca/~rbridson/docs/batty-siggraph2007-variationalcoupling.pdf),
p. 3, equations (1) and (3), formulates pressure correction and kinetic energy;
p. 4, equations (4)–(6), gives the mass-weighted discrete projection. Section
2.1 permits a face-area estimate for mass. Here the fixed pressure mass is
`rho*A*d`; no exact clipped dual-volume equality is inferred.

Those papers motivate the strain energy and quadrature locations. The
sector-resolved aligned-corner reconstruction below, its reflected anisotropic
formula, the explicit row-sum bound, and this restricted admission contract
are newly derived choices. They are not attributed verbatim to either paper.

## Represented geometry and enumeration

Let `X_a[i]` be stored grid-plane coordinates and `C_a[i]` the stored fluid-cell
centers. All widths, adjacent-center distances and center-to-wall distances use
these represented coordinates; a translated binary64 center need not be an
exact midpoint. Require finite positive used widths, sector volumes, face
areas, pressure distances and masses. The six obstacle planes must equal their
retained stored grid planes exactly, with integer index ranges strictly inside
the domain and at least one cell of padding. Cells are fully fluid or fully
solid. Moving walls, partial cells, contact with the domain and several boxes
are excluded.

For a cell with retained stored volume `V`, include the three normal rows

    E_aa(u) = (u_a at X_a[i+1] - u_a at X_a[i]) / (X_a[i+1]-X_a[i]),
    w_aa = 2 V,                       a = x,y,z.

A normal face on a stationary obstacle or outer wall is a prescribed zero and
is eliminated from the row coefficients. Keep a row whose resulting
coefficient list is empty: it is still one fluid quadrature sample.

For each of the three unordered axis pairs `(a,b)`, enumerate only strictly
internal `a,b` grid planes and each cell-center layer in the remaining axis
`c`. The engineering shear is `gamma_ab = partial_b u_a + partial_a u_b`.
The outer-wall engineering shear is analytically zero: the sealed normal
trace is constant zero along the wall, and free slip gives zero cross-wall
tangential derivative by even continuation. Hence outer-edge shear samples
are eliminated analytically before enumeration. This convention must not be
confused with dropping zero rows on the included internal edges.

An included edge has four adjacent cell quadrants. Every actual fluid
quadrant contributes a row, including a row that becomes zero. Solid quadrants
contribute no row. For quadrant `q`, its weight is the stored product

    v_q = |C_a[q_a]-X_a[i]| * |C_b[q_b]-X_b[j]|
          * (X_c[k+1]-X_c[k]).

In exact-real algebra the stored coefficients and weights are fixed input
numbers. Their binary64 computation is outside the proof. The shortcut
`v_q = V/4` needs exact midpoint and exact product-volume hypotheses.

## Internal edge cases

Four fluid quadrants use the same row in each quadrant:

    gamma_ab = (u_a,b+ - u_a,b-) / (C_b[j]-C_b[j-1])
             + (u_b,a+ - u_b,a-) / (C_a[i]-C_a[i-1]).

Two adjacent fluid quadrants describe a flat no-slip wall. The wall-normal
component is a prescribed zero at both tangential locations. The tangential
component is continued affinely from its fluid face sample `U` to the zero
wall trace at the represented wall position. Each fluid quadrant has the row
`s U/delta`, where `delta` is the actual stored center-to-wall distance and
`s` is the signed derivative direction. The diagonal conductance without
viscosity is

    k = sum_q(v_q) / delta^2 = A_eff / delta,
    A_eff = sum_q(v_q) / delta.

The geometric face-area formula `k=A/delta` additionally requires the explicit
exact-product-volume identity `sum_q(v_q)=A*delta`. Rounded products do not
establish it automatically.

Three fluid quadrants describe a convex obstacle edge. Reflect local
coordinates and vector components so the missing solid quadrant is
`x<0,y<0`. Let `ell_b,ell_a>0` be the actual positive fluid center-to-wall
lengths, and let `U,V` be the two surviving active samples. Choose

    u_a(x,y) = U max(y,0)/ell_b,
    u_b(x,y) = V max(x,0)/ell_a.

The traces vanish on both solid half-walls and interpolate the samples. The
three piecewise derivatives produce the sector rows

    northeast: U/ell_b + V/ell_a,
    northwest: U/ell_b,
    southeast: V/ell_a.

The local quadratic form is their weighted sum of squares. Its matrix is

    [ (v_NE+v_NW)/ell_b^2       v_NE/(ell_a*ell_b)       ]
    [ v_NE/(ell_a*ell_b)        (v_NE+v_SE)/ell_a^2       ].

For a unit centered grid, `ell_a=ell_b=1/2` and every sector volume is `1/4`,
giving `[[2,1],[1,2]]`. Reflection reverses the corresponding derivative signs.
More explicitly, if the missing signs are `(s_a,s_b)`, a fluid sector
`(t_a,t_b)` includes `(-s_b/ell_b) U` when `t_b=-s_b` and includes
`(-s_a/ell_a) V` when `t_a=-s_a`. This includes all orientations without
changing the physical vector-component convention.

Zero fluid quadrants contribute no sample. One fluid quadrant or two diagonal
fluid quadrants on an internal edge cannot arise from the admitted single
padded box; reject such a classification. Do not invent a fallback stencil.

## Gather, transpose and exact work

Write all retained rows as a finite real matrix `E`, with fixed weights
`w_r>=0`. Define

    s_r = (Eu)_r = sum_f E_rf u_f,
    (Ku)_f = sum_r E_rf w_r s_r,          K=E^T W E,
    D(u) = sum_r w_r s_r^2.

Gather each row from active faces and scatter its weighted strain through
exactly the same coefficients. Finite sum exchange gives
`u^T Ku=D(u)`, symmetry follows by the same exchange, and `D(u)>=0` follows
from nonnegative weights. These conclusions hold with zero rows and do not
identify this matrix with arbitrary embedded Newtonian traction.

For mass `m_f>0`, put `L_r=sum_g |E_rg|` and use any `B>=0` satisfying

    (1/m_f) sum_r w_r |E_rf| L_r <= B       for every f.

The implementation uses the maximum of these face bounds, with maximum zero
for an empty active space. Two weighted Cauchy inequalities establish

    sum_f (Ku)_f^2/m_f <= B D(u).

Indeed, for `L_r>0`, each face square is bounded by
`[sum_r w_r |E_rf| L_r] [sum_r w_r |E_rf| s_r^2/L_r]`.
After division by `m_f`, apply the face bound and exchange sums; the second
factor sums over faces to `D(u)`. A row with `L_r=0` has every coefficient and
strain zero, so contributes zero and requires no division. Equivalently the
symmetric scaled operator `M^(-1/2) K M^(-1/2)` has spectrum in `[0,B]`.
No iterative-solver convergence is used.

## Conditional Euler and matched pressure composition

With constant scalar viscosity `mu>=0`, no force and exact arithmetic, set
`t=dt*mu>=0` and take the coordinate step

    v_f = u_f - t (Ku)_f/m_f.

For `H(u)=sum_f m_f u_f^2/2`, direct expansion and the work identity give

    H(v)-H(u) = -t D(u) + (t^2/2) sum_f (Ku)_f^2/m_f
             <= -t (1-t B/2) D(u) <= 0       if t B<=2.

The Lean statement exposes positive masses, nonnegative fixed weights,
nonnegative time and viscosity, the exact coordinate step, the per-face
coefficient bound, and `dt*mu*B<=2`. Its `coefficient_force_bound` theorem
derives the quadratic operator bound directly from those coefficients using
weighted finite Cauchy and sum exchange, including zero rows. No abstract
operator-norm premise is assumed in the energy or composition theorem.

For pressure incidence `P`, the matching correction is
`z_f=v_f-dt*(P^T p)_f/(rho*d_f)`, with the same face space and
`m_f=rho*A_f*d_f`. Let outward integrated flux be `Q=-P(A*v)` and require an
exact solve of the defined full pressure residual, positive `rho,A_f,d_f`, and
`dt!=0`. The existing exact-pressure theorem then gives `H(z)<=H(v)`, so the
new composition gives `H(z)<=H(u)`. This is an exact matched projection;
finite PCG tolerance, binary64 dot products, gauge implementation and force
work are separate obligations. The formal statement does not claim a global
solver-convergence, IEEE refinement or continuum-convergence result.
