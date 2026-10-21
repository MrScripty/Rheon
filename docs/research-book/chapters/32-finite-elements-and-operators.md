# 32 Finite-element spaces, quadrature and element operators

[Chapter 31](31-stress-and-weak-forms.md) supplies the weak equation. We now choose a finite-dimensional motion and derive each array needed to evaluate it. The default solid is a conforming, straight-sided P1 tetrahedral mesh with positive reference volumes, constant density per element, and the specified hyperelastic law. Higher-order and mixed alternatives are stated separately. The [Sierra/SM 5.30 manual](https://www.sandia.gov/files/sierra/SM_Theory_5_30/index.html), provides primary weak-form and mechanics context in the selected chapters identified by the publication audit. Element calculations and practical explanations below are independently authored.

## Three maps that must not be confused

Let \(\widehat K\) be the unit tetrahedron with local coordinate \(\xi\), vertices \(0,e_1,e_2,e_3\) and volume \(1/6\). Its geometric map to a reference element is \(X_e(\xi)=\sum_a\widehat N_a(\xi)X_a\), with geometric Jacobian \(G_e=\partial X_e/\partial\xi\). The physical motion is \(x_e=\sum_a\widehat N_a x_a\), with \(g_e=\partial x_e/\partial\xi\). The mechanical deformation gradient is

\[
 F=g_eG_e^{-1},\qquad J=\det(g_e)/\det(G_e).
\]

Thus \(\det(G_e)\) is a reference integration scale, not the deformation volume ratio. On affine tetrahedra require \(\det(G_e)>0\) and use \(dV_0=\det(G_e)d\xi\); taking an absolute determinant can hide an orientation error.

P1 shape functions are \(\widehat N_1=1-\xi_1-\xi_2-\xi_3\), \(\widehat N_2=\xi_1\), \(\widehat N_3=\xi_2\), \(\widehat N_4=\xi_3\). They satisfy \(\sum N_a=1\), \(\sum X_aN_a=X\), and \(\sum\nabla_XN_a=0\). Store gradients as column vectors

\[
 G_a=\nabla_XN_a=G_e^{-T}\nabla_\xi\widehat N_a,
 \qquad F=\sum_ax_a\otimes G_a=I+\sum_ad_a\otimes G_a.
\]

P1 has four scalar basis functions per tetrahedron and twelve local displacement components. A shared vertex has one global displacement, not one per incident element. Connectivity plus the local-to-global map defines assembly. P2 uses ten nodes (vertices and edge midpoints): barycentric vertex bases are \(\ell_a(2\ell_a-1)\), and edge bases \(4\ell_a\ell_b\). These reproduce quadratic fields but are not all nonnegative. Their mass-lumping behavior differs from P1.

## Quadrature is part of the method

For weights \(\widehat w_q\) integrating on \(\widehat K\), approximate \(\int_{K_0}h\,dV_0\) by \(\sum_qw_qh_q\), \(w_q=\widehat w_q\det G_e(\xi_q)\). State whether tabulated weights sum to \(1/6\) or one; both conventions exist and require different scaling.

For affine P1 with homogeneous material, \(F\) and \(P\) are constant on each element. One barycentre sample with physical weight \(|K_0|\) exactly integrates internal force and energy for that field. Mass contains products \(N_aN_b\) and requires degree-two integration. The one-point consistent mass \(\rho_0|K_0|N_a(\bar\xi)N_b(\bar\xi)\) has rank one per component, so it is not a valid substitute for either exact mass or deliberate positive row-sum lumping. Boundary loads and variable coefficients may require more quadrature even for P1.

For P2 or curved geometry, determinants, inverses and logarithms produce nonpolynomial integrands. Polynomial exactness of a quadrature rule does not establish exact nonlinear integration. Specify a baseline rule and an independently refined quadrature comparison in later qualification. Reduced integration can introduce zero-energy modes; hourglass stabilization changes the discrete operator and needs its own consistency and work analysis.

## Mass, loads and internal force

With scalar basis functions and vector nodal unknowns, the consistent block mass and nodal internal residual are

\[
 M_{ai,bj}=\delta_{ij}\int_{\Omega_0}\rho_0N_aN_b\,dV_0,
 \qquad f^{\mathrm{int}}_{ai}=\sum_{e\ni a}\int_{K_{0,e}}P_{iA}(G_a)_A\,dV_0.
\]

For material meshes this mass is constant in time because both \(\rho_0\) and material bases are fixed. Current density and volume must transform together to recover the same integral. The body load is \(f^b_a=\int\rho_0N_ab\,dV_0\); a dead surface load is \(f^T_a=\int N_a\bar T\,dA_0\). Their sum is \(f^{\mathrm{ext}}\). The semidiscrete equation is \(M\ddot d+f^{\mathrm{int}}=f^{\mathrm{ext}}\), before damping or contact.

For affine P1 and constant \(\rho_0\), exact scalar element mass is

\[
 M_e^{s}=\frac{\rho_0|K_0|}{20}
 \begin{bmatrix}2&1&1&1\\1&2&1&1\\1&1&2&1\\1&1&1&2\end{bmatrix}.
\]

Each row sums to \(\rho_0|K_0|/4\). P1 row-sum lumping places this value on the diagonal; it preserves total mass but changes kinetic energy and modal frequencies. Blind P2 row sums can be nonpositive: on a tetrahedron \(\int\ell_a(2\ell_a-1)dV=-|K_0|/20\). The mixed fluid branch therefore uses consistent P2 mass.

## Consistent elastic tangent and constraints

Differentiating the actual quadrature residual gives

\[
 K_{ai,bj}=\sum_{e\ni a,b}\sum_q w_q
 (G_a)_A\mathcal A_{iAjB}(F_q)(G_b)_B.
\]

The derivative uses the same points, weights and constitutive evaluation as the residual. For a direction \(z\), compute \(\delta F=\sum_bz_b\otimes G_b\), apply \(\mathcal A[\delta F]\), then scatter \(w_q\mathcal A[\delta F]G_a\). This defines a matrix-free tangent action without storing all element matrices. Matrix-free evaluation still needs a preconditioner, geometry, basis tables, workspace and communication.

For a fixed linear constraint map write \(d=d_D+Rz\). The admissible residual and tangent are \(R^Tr\) and \(R^TKR\); the consistent reduced mass is \(R^TMR\). A general hanging-node or periodic constraint does not preserve diagonal mass even if the full-space mass was lumped. Moving/nonlinear constraints require derivatives of the constraint itself. Reactions are the eliminated full residual components, with signs stated by the applied-force convention; they are not zeroed data to be discarded.

**Conditional proof, zero internal resultant.** With partition of unity, identical element stress for all local nodes and consistent quadrature,
\(\sum_af^{\mathrm{int}}_a=\sum_qw_qP\sum_aG_a=0\).
If coordinates are reproduced and \(PF^T=J\sigma\) is symmetric, \(\sum_ax_a\times f^{\mathrm{int}}_a=0\): its components contract the antisymmetric cross-product tensor with symmetric \(PF^T\). These statements concern unconstrained internal contributions in exact arithmetic. External loads, support reactions, inconsistent boundary integration and floating-point scatter alter the observed balance.

## Worked tetrahedron, force and mass

Use nodes \(X_1=(0,0,0)\), \(X_2=(1,0,0)\), \(X_3=(0,1,0)\), \(X_4=(0,0,1)\), metres. Then \(|K_0|=1/6\) m\(^3\), \(G_1=(-1,-1,-1)\), \(G_2=e_1\), \(G_3=e_2\), \(G_4=e_3\), in m\(^{-1}\). At \(\rho_0=6\) kg/m\(^3\), mass is one kilogram, consistent diagonal entries are \(1/10\) kg, off-diagonal \(1/20\) kg, and lumped entries \(1/4\) kg per component. Summing scalar entries gives one kilogram; summing all three component blocks gives three copies of the same mass metric, not three kilograms of matter.

Set affine shear \(d_a=(kX_{a2},0,0)\), \(k=1/2\), \(\mu=2\) Pa. Then

\[
 F=\begin{bmatrix}1&k&0\\0&1&0\\0&0&1\end{bmatrix},
 \quad P=\begin{bmatrix}0&\mu k&0\\\mu k&0&0\\0&0&0\end{bmatrix}.
\]

For the Chapter31 energy \(J=1\), so the value of \(\lambda\ge0\) does not affect this stress. Internal residuals are \(f_1=(-1/6,-1/6,0)\), \(f_2=(0,1/6,0)\), \(f_3=(1/6,0,0)\), \(f_4=0\), newtons. Their sum vanishes. Stored energy is \(|K_0|\mu k^2/2=1/24\) joule. Boundary nodal loads equal to these residuals hold the affine state in equilibrium. A single element is a derivative/sign oracle, not a convergence study or an incompressible stable mesh.

## Element dataflow

An element evaluation starts from its reference coordinates, nodal displacement and chosen quadrature rule. The reference coordinate map supplies the determinant and physical basis gradients; positive reference orientation is a construction requirement. The current displacement determines F, whose determinant must lie in the selected material domain before stress or tangent evaluation.

The same quadrature weights and reference measures multiply the energy, stress–gradient force and derivative contraction written above. Their local nodal contributions are accumulated into the shared global degrees of freedom. Shared nodes receive the sum from every incident element; a later element does not replace an earlier contribution. This consistency is what makes the assembled tangent differentiate the assembled residual.

Constitutive history in nonlinear trial evaluations is candidate scratch derived from the last accepted history, never repeatedly committed during Newton. The selected hyperelastic law needs no evolving history. [Chapter 33](33-time-and-nonlinear-solves.md) adds inertia and nonlinear solution; [Chapter 34](34-incompressibility-and-fluids.md) adds pressure and the fluid geometry derivative.
