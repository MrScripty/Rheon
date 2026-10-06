# Nonzero third velocity on the selected periodic extrusion

The oracle-repair source/evidence `5dc5d8c3/ec26e48d` and equation-audit
source/evidence `946bebfa/d1617802` remain frozen and are composed by a normal
merge on the new implementation branch. This successor adds the third velocity
to their existing spatial and temporal model; it does not substitute a static
geometry or a different boundary problem.

## Exact domain, boundaries and admissible fields

The domain is

    Omega(t) = {(x,y,z): x in R/Z, 0 < y < H(x,t), z in R/Z}.

Both x and z have period one. There are no extrusion end walls. All fields are
independent of z, but U=(u,v,w) has three components. In particular w may cross
the periodic z seam; impermeable end walls would instead exclude that motion.
This is a two-dimensional, three-component family, not general 3D motion.

The same two-column polygonal cap, Powell–Sabin microtriangles, periodic x
identifications, bottom abscissae [0,.5,1], density 3 and viscosity .05 are used.
The two independent cap heights sum to 2.25; liquid mass is 3.375. The bottom is
impermeable (v=0), with natural zero tangential traction for both u and w.
The cap is material in x/y, with zero atmospheric-relative traction in the
restricted weak formulation. Its normal has no z component, so w does not
change the graph's kinematic equation. The corresponding third traction is
mu grad(w) dot n_xy=0, weakly on the cap and bottom. There are no imposed w=0
boundary values, body forces, adhesion, density variation or capillary forces.
Pointwise traction accuracy is still not established by this constrained space.

The continuous P1 third field uses the existing periodic nodal embedding
w=Rw xi. Its cap/bottom edge midpoints are the averages of their macro endpoints.
It has twelve independent coefficients, the same scalar trace constraints as
the x component. Rw is constant and full column rank on this topology; Rw 1=1.
The example's nonconstant dyadic coefficient vector is admissible weak initial
data. It excites both shears; it is not an exact classical traction benchmark.

## Conservative third momentum and symmetric strain

Use exactly the accepted/candidate liquid masses and the already qualified xy
mesh path. A z mesh speed of zero is permitted since all fields are z invariant.
Flux through paired z faces cancels cell by cell. The in-plane physical relative
face rate f_ij, its sign partition and integrated Fplus/Fminus are unchanged.
The third donor momentum flux is

    g_ij,w = f_ij (w_i+w_j)/2 + abs(f_ij) (w_i-w_j)/2,
    c_i,w = sum_j g_ij,w.

It is antisymmetric on each shared face. Pressure has no third force because
partial_z p=0. The engineering strain remains

    E U = (u_x, v_y, u_y+v_x, w_x, w_y),
    W_T = mu |T| diag(2,2,1,1,1).

The third stiffness is therefore K_w=Rw^T G^T diag(mu |T| I_2) G Rw, not twice
that matrix and not a scalar diffusion model with different boundary rules.
The existing full element symmetric-strain assembly supplies that block.
In the semidiscrete model,

    Rw^T [ (m w)' + c_w ] + K_w xi = 0,
    Mw xi' = -Rw^T [m' w+c_w] - K_w xi,
    Mw = Rw^T diag(m) Rw.

The xy geometry, momentum, divergence and pressure solve are independent of w
on this restricted family. Dotting this equation with xi and using the same
physical GCL gives

    T_w' = -D_adv,w - xi^T K_w xi,
    T_w = sum_i m_i w_i^2/2,
    D_adv,w = sum_{unordered ij} abs(f_ij)(w_i-w_j)^2/2.

Both third shears are required. Third momentum is conserved because constant
third translations are admissible, the shared donor flux is antisymmetric and
the natural strain block annihilates constants. Normal wall momentum has the
existing explicit reaction and is not claimed conserved.

## Finite step and work gate

After solving the unchanged nonlinear xy path, solve only this twelve-row block:

    Aw xi1 = Rw^T diag(m0) w0,
    Aw = Rw^T diag(m1) Rw + C_F + h K_w,1,
    C_F = sum_{ij} (Ri-Rj)^T (Fplus_ij Ri-Fminus_ij Rj).

Here Ri is row i of Rw and K_w,1 is the endpoint full-strain block. All fluxes
are the actual integrated xy face rates, not fitted mass marginals. Positive
masses, injective Rw and exact GCL imply coercivity of this generally
nonsymmetric matrix:

    xi^T Aw xi = sum_i (m0_i+m1_i) w_i^2/2
                 + sum_{ij} (Fplus+Fminus)(w_i-w_j)^2/2
                 + h xi^T K_w,1 xi > 0  for xi != 0.

No inverse-positivity or componentwise maximum principle is inherited through
the constrained Rw projection. With actual finite residual r and GCL defect
g_i=m1_i-m0_i+sum_j(Fplus-Fminus)_ij, the exact finite identity is

    T_w,1-T_w,0 + D_BE,w+D_mix,w+D_mu,w
      + sum_i g_i w1_i^2/2 - xi1^T r = 0,
    D_BE,w = sum_i m0_i(w1_i-w0_i)^2/2,
    D_mix,w = sum_{ij}(Fplus+Fminus)(w1_i-w1_j)^2/2,
    D_mu,w = h sum_T mu |T| [(w_x)^2+(w_y)^2].

The last loss is reported as two separate shear contributions as well as their
sum. Full kinetic energy and each loss are the sums of the xy and w terms;
pressure work remains the existing xy term. Separate third and total residual/
ledger gates prevent a large xy energy from hiding a bad third solve. Direct
and stable changing-mass momentum rates, total third momentum and full accepted
embedding are checked. Existing physical limits and 128-epsilon work/GCL factors
are retained. The method is first-order endpoint donor transport and backward
Euler strain on the same moving path; it is not exact path momentum integration.

## Owner, publication and evidence scope

The implementation plan reuses the existing CoupledDiscreteFlow accepted
frame, full three-component velocity, candidate velocity and pressure owner.
The opt-in constructor/step will reuse its twelve-entry scalar buffers. Additional dense factors
and embedding scratch are fixed arrays with a conservative declared reservation;
there is no new accepted phase, heap allocation during a step or rollback history.
Every candidate field and all reports/gates finish before the common final
publication barrier. The legacy zero-third constructor and step retain their
restricted contract and allocation budget.

Evidence must export actual owner velocity, geometry, masses, physical pressure,
clock and stamp after publication. The successor replay keeps mandatory physical
fields, canonical distinct common-endpoint cases, independently regenerated
related references, and recomputed errors/ratios. Full-vector work is reassembled
from those actual publications. A render shows their actual w on the moving
microtriangles. Independent DAE references include the twelve third coefficients
and both shears; temporal self-convergence alone is not model validation.

All earlier packets, invalid pressure-rate interpretation and the initial
.00078125 bounded-Newton refusal remain retained. The existing conditional
full-vector algebra motivates these identities; no new Lean or IEEE, sign-
isolation, arbitrary-mesh, global stability, positivity, continuum spatial,
pointwise cap traction, variable-material or reconstruction theorem is claimed.
