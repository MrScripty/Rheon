# 8 Boundaries and obstacles

On a wall moving at \(\mathbf u_s\), impermeability requires \((\mathbf u-\mathbf u_s)\cdot\mathbf n=0\). No-slip also matches tangential velocity; free-slip does not. These are distinct models. Prescribed normal wall velocity enters the divergence right-hand side. Atmospheric-pressure openings instead fix pressure and allow flow. Combining both conditions on one face can overconstrain the system.

## Binary geometry

A first obstacle model can classify cells and block faces. The resulting staircase approximates a curved boundary but changes effective volume and creates grid-aligned features. Refinement can move the represented wall discontinuously as classifications change. An accurate linear solve cannot remove geometric error.

A boundary snapshot should have one version consumed by advection, pressure and rendering for a step. If geometry changes during pressure solution, discard or restart the stale candidate rather than publish velocity corrected for a different domain.

## Fractional faces

An open area fraction \(a_e\) and fluid cell-volume fraction \(\theta_i\) produce divergence terms proportional to \(a_eh^2/(\theta_ih^3)\). Tiny \(\theta_i\) creates small-cell stiffness. Clamping coefficients changes the model; removing cells changes topology. Either can be an explicit approximation, not a free fix.

Batty, Bertails and Bridson develop variational solid-fluid coupling on coarse Cartesian grids [S7]. The structural lesson is to choose gradient, divergence and energy together. Rheon's arbitrary-matrix theorem does not establish cut-cell quadrature.

For the linear homogeneous operator \(D_0\), with cell metric \(M_c\) and face metric \(M_f\), require

\[
\langle p,D_0\mathbf u\rangle_{M_c}
=-\langle Gp,\mathbf u\rangle_{M_f},
\qquad D_0=-M_c^{-1}G^TM_f.
\]

An attractive divergence formula paired with an unrelated gradient can yield an inconsistent pressure operator. Test this linear weighted adjoint identity with random vectors on every supported geometry class after prescribed boundary flux is separated. For the metric construction in Chapter 3, \(M_c=V\), \(M_f=SL\), and \(D_0=-V^{-1}BS\). The full physical divergence is affine: \(D(u)=D_0u+V^{-1}Q\). Therefore its identity includes boundary work:

\[
\langle p,D(u)\rangle_V=-\langle Gp,u\rangle_{SL}+p^TQ.
\]

Omitting the final term is valid only when its value is zero, for example homogeneous prescribed flux. The companion includes nonzero Q to reject that omission. The existing Lean arbitrary-B theorem proves the linear algebraic identity, not moving-boundary assembly.

## Trajectory collision

A departure point outside solid material does not prove the path avoided it. A long trace can cross a thin wall and emerge on the far side. Intersect straight traces with geometry and apply the declared rule at the earliest hit. Multi-stage curved traces need compatible approximation. Limiting travel helps but is not collision proof.

Boundary interpolation must define its extension. Constant extrapolation, mirrored normal velocity and no-slip ghost values differ. Do not bury them in a texture sampler mode. Check samples on both sides of walls and at corners.

## Diagnostic sequence

Test a sealed box with uniform translation, then stationary fluid under gravity with hydrostatic pressure, then moving obstacles with displaced-volume and wall-work accounting. A wall thinner than one cell must be resolved or explicitly unsupported rather than silently leak.

Report flux by boundary category and closed component. Long-term leaks often originate in one inconsistent face update. More pressure iterations help only if pressure residual is responsible; topology, geometry and buffer timing remain separate suspects.
