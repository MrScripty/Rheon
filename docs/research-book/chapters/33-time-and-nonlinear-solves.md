# 33 Time integration, Newton iteration and linear solvers

[Chapter 32](32-finite-elements-and-operators.md) gives \(M\ddot d+f^{\mathrm{int}}(d)=f^{\mathrm{ext}}\). Time integration defines a discrete problem before Newton or Krylov attempts to solve it. We derive several choices and select backward Euler for the complete reference algorithm. All timestep bounds and tolerances below describe the new exposition; no Rheon validation threshold is modified.

## Explicit central difference

With diagonal lumped mass \(M_L>0\), half-step velocity and a prescribed timestep \(h>0\) seconds,

\[
 v_{n+1/2}=v_{n-1/2}+hM_L^{-1}(f_n^{\mathrm{ext}}-f_n^{\mathrm{int}}),
 \qquad d_{n+1}=d_n+hv_{n+1/2}.
\]

Initialize \(v_{-1/2}=v_0-(h/2)a_0\) with acceleration from initial force balance. Impose prescribed degrees of freedom consistently and measure their reactions. Consistent mass instead requires a mass solve each stage; explicit refers to the force/time coupling, not necessarily a division per node.

For the linear undamped system \(M\ddot d+Kd=0\), \(M\) SPD and \(K\) positive semidefinite after appropriate constraints, diagonalize in generalized modes \(K\phi=\omega^2M\phi\). Each mode satisfies \(q_{n+1}-(2-h^2\omega^2)q_n+q_{n-1}=0\). Roots lie on the unit circle for \(0<h\omega<2\). At the equality endpoint a repeated root can produce growth, so use a strict bound with margin. This conditional linear stability result does not handle changing tangents, inverted elements, nonlinear contact, damping or fracture.

A wave-speed estimate uses \(c_p=\sqrt{(\lambda+2\mu)/\rho_0}\) and a short element length; its prefactor depends on element, mass, order, distortion and integrator. A mesh's original edge length is inadequate after large compression. Stiff penalties and near-incompressibility increase the fast frequency. Mesh feasibility and contact crossing can require a smaller step even when a linear frequency estimate passes.

## Backward Euler as a nonlinear residual

Select

\[
 d_{n+1}=d_n+hv_{n+1},\qquad
 M(v_{n+1}-v_n)/h+f^{\mathrm{int}}(d_{n+1})-f^{\mathrm{ext}}(d_{n+1},t_{n+1})=0.
\]

Eliminate velocity, define \(d_{\mathrm{pred}}=d_n+hv_n\), and solve the displacement residual

\[
 r(d)=M(d-d_{\mathrm{pred}})/h^2+f^{\mathrm{int}}(d)-f^{\mathrm{ext}}(d,t_{n+1}),
 \quad K_{\mathrm{eff}}=M/h^2+K_{\mathrm{int}}-K_{\mathrm{ext}}.
\]

Residual entries are newtons; tangent entries N/m. Afterwards \(v_{n+1}=(d-d_n)/h\), \(a_{n+1}=(v_{n+1}-v_n)/h\). Dead loads have zero \(K_{\mathrm{ext}}\); follower pressure and contact do not. If adding constant reference damping \(Cv\), add \(C(d-d_n)/h\) and \(C/h\). This is a declared linear damping model, not a general finite-strain Newtonian viscosity.

For conservative dead loads, the equation is stationarity of

\[
 \Phi(d)=\frac{1}{2h^2}(d-d_{\mathrm{pred}})^TM(d-d_{\mathrm{pred}})
          +\Psi(d)-f_{n+1}^{\mathrm{ext}}\cdot d.
\]

It may have multiple stationary points and an indefinite Hessian. The neo-Hookean energy is not globally convex in displacement. A converged Newton root is not automatically a global minimum or physically stable state.

**Conditional energy inequality.** If the discrete elastic potential \(\Psi\) is differentiable and convex along the segment from \(d_n\) to \(d_{n+1}\), exact backward Euler satisfies

\[
 E_{n+1}-E_n+\tfrac12\|v_{n+1}-v_n\|_M^2
 \le h f_{n+1}^{\mathrm{ext}}\cdot v_{n+1},
 \qquad E_n=\tfrac12v_n^TMv_n+\Psi(d_n).
\]

Proof: dot momentum with \(hv_{n+1}=d_{n+1}-d_n\), use
\(v_{n+1}^TM(v_{n+1}-v_n)=\tfrac12(\|v_{n+1}\|_M^2-\|v_n\|_M^2+\|v_{n+1}-v_n\|_M^2)\), and the endpoint convexity inequality for \(\Psi\). An unresolved residual adds its work to the right-hand side. Since global convexity is absent for general finite strain, this is not a universal energy theorem for the reference solid.

## Newmark and energy-aware alternatives

For acceleration-based structural dynamics the Newmark family specifies

\[
 d_{n+1}=d_n+hv_n+h^2[(1/2-\beta)a_n+\beta a_{n+1}],
 \quad v_{n+1}=v_n+h[(1-\gamma)a_n+\gamma a_{n+1}].
\]

For \(\beta>0\), the displacement tangent includes \(M/(\beta h^2)\); constant damping contributes \(\gamma C/(\beta h)\). Average acceleration \(\beta=1/4,\gamma=1/2\) is second order and unconditionally stable for the linear undamped positive system, but neither arbitrarily accurate at large \(h\) nor universally energy-conserving for nonlinear forces/contact. [Newmark (1959), 85(3), 67–94](https://ascelibrary.org/doi/10.1061/JMCEA3.0000098) is the original publication; this research verified its bibliographic record, not restricted full text. The time-integrator discussion is independently derived standard mathematics.

For clarity, the linear average-acceleration claim can be checked without the unread original paper. With constant symmetric positive-definite mass and symmetric positive-definite stiffness on the constrained displacement space, and initial acceleration satisfying the undamped force balance, write the equation as a first-order system \(\dot z=Az\). The average-acceleration update is trapezoidal, \((I-hA/2)z_{n+1}=(I+hA/2)z_n\). Each oscillatory mode has eigenvalue \(i\omega\), so its amplification factor is \((1+i h\omega/2)/(1-i h\omega/2)\), of modulus one for real \(h\omega\). Expanding that factor gives the exact exponential through second order. This is a conditional exact-real linear/modal argument, not a nonlinear stability theorem, original-source proof attribution or executed test.

Midpoint, generalized-alpha and discrete-gradient/energy-momentum methods make different choices about high-frequency damping, conservation and nonlinear work. One must specify both update and force evaluation; a method name is insufficient. [MFEM v4.7 example10](https://github.com/mfem/mfem/blob/v4.7/examples/ex10.cpp) separates its deformation/velocity operator from several selectable ODE integrators. Those examples motivate architecture comparison, not a claim of identical constitutive or damping equations.

## Consistent Newton, feasible line search and stopping

At iterate \(z_k\) in the free constrained coordinates, evaluate \(\widehat r=R^Tr(d_D+Rz_k)\), \(\widehat K=R^TK_{\mathrm{eff}}R\), then approximately solve \(\widehat K s=-\widehat r\). Recompute all quadrature states from the same trial and last accepted history. Local quadratic convergence requires a nonsingular derivative at the solution, adequate smoothness, sufficiently close initialization and sufficiently accurate linear solves; it is not a global guarantee.

A residual merit is \(m(z)=\tfrac12\|D_r\widehat r(z)\|_2^2\), with fixed positive row scaling \(D_r\) for this attempted step. For an exact Newton direction, \(m'(z)[s]=-\|D_r\widehat r\|_2^2\). Inexact solves require checking a negative directional derivative. Use bounded backtracking: test a positive \(\alpha\le1\), first check feasibility along the candidate trajectory, then enforce an Armijo decrease \(m(z+\alpha s)\le m(z)+c\alpha m'(z)[s]\), \(0<c<1\). If the direction is not descent or no step passes within the configured cap, report failure or use a separately specified globalization method; do not silently accept a worsening iterate.

Positive endpoint determinants do not certify the whole line-search segment. For affine tetrahedra \(\det(F+\alpha\delta F)\) is cubic in \(\alpha\); a robust implementation can bound the first positive zero and step strictly inside it. Global surface intersection still requires continuous collision detection. P2 geometry needs more than corner/quadrature sampling to certify positivity everywhere. The fluid branch therefore identifies such certification as a separate geometry requirement.

Use absolute-plus-relative residual tolerances in declared physical scales, a displacement-increment criterion, actual force balance, finite arithmetic and geometry checks. A tiny update caused by an ill-conditioned Jacobian is stagnation, not convergence. Cap Newton iterations, line-search trials, Krylov iterations, memory and total attempted timesteps. A reduced timestep retry is a new bounded attempt from unchanged accepted state; unlimited halving can hide a persistent model defect.

## Krylov and preconditioning

CG requires a symmetric positive-definite operator and an appropriate SPD preconditioner. A mass term can improve a tangent but does not prove positivity for arbitrarily deformed solids. MINRES handles symmetric indefinite systems with an SPD preconditioner. GMRES handles nonsymmetry; restarted GMRES needs a bounded restart space, while flexible GMRES accommodates changing preconditioners. A small Krylov estimate must be checked against the true assembled/operator residual, especially after preconditioning or restart.

For a solid, a sparse direct factorization is a useful small-problem reference; block Jacobi is cheap but ignores long-range elastic coupling. Algebraic or geometric multigrid needs elasticity-aware coarse modes, including near-rigid modes. Near incompressibility requires a mixed/block treatment rather than assuming a scalar Poisson preconditioner will suffice. Matrix-free high-order action can save global stiffness storage but shift cost to repeated quadrature and preconditioning. Pin solver/library versions when implementing; no performance claim follows from this exposition.

## A complete scalar implicit step by hand

Use one axial element with fixed left endpoint, reference length \(L_0=1\) m, area \(A_0=1\) m\(^2\), density \(\rho_0=1\) kg/m\(^3\). In this explicitly one-dimensional teaching reduction, choose \(\Psi(q)=kq^2/2\), \(k=4\) N/m, lumped free-node mass \(m=1/2\) kg. This is a linear axial model, not a general 3D plane-stress neo-Hookean reduction. At \(q_n=0\), \(v_n=1\) m/s, \(h=1/2\) s and zero external load,

\[
 r(q)=2(q-1/2)+4q=6q-1,\quad q_{n+1}=1/6\,\mathrm{m},
 \quad v_{n+1}=1/3\,\mathrm{m/s}.
\]

One exact Newton correction gives the solution from any initial guess. Initial energy is \(1/4\) J; final kinetic energy is \(1/36\) J and elastic energy \(1/18\) J, total \(1/12\) J. The decrease includes \(\tfrac12m(\Delta v)^2=1/9\) J plus elastic endpoint convexity gap \(1/18\) J. This checks coefficients and signs; it supplies no convergence measurements for nonlinear 3D FEM.
