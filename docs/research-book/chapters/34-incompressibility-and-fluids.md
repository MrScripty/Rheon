# 34 Incompressibility, locking and material-mesh fluids

A moving connected mesh does not choose the material law. Solids retain deformation-dependent shear stress; Newtonian fluids dissipate through velocity gradients. This chapter first explains mixed stability, then specifies a complete finite-dimensional moving-fluid timestep. It is an independently derived reference formulation, **not a newly validated method or a theorem of nonlinear convergence**. Its derivative is specified so that [Chapter 36](36-reference-fem-solver.md) can give an implementable algorithm. No Rheon production capability is implied.

## Why displacement-only elements lock

In a nearly incompressible elastic law the bulk modulus \(\kappa\) is much greater than shear modulus. A displacement-only approximation minimizes a very large volumetric penalty while also trying to approximate shear. Low-order spaces may admit too few nearly volume-preserving modes, producing excessive stiffness that persists under practically affordable refinement. Increasing the penalty further can worsen conditioning and locking without improving shear accuracy. Reduced integration can relieve a constraint but introduce spurious modes. A mixed formulation makes pressure an independent unknown instead.

For example, in an exactly incompressible hyperelastic model choose an isochoric energy \(W_{\mathrm{iso}}(F)\) and the Lagrangian

\[
 \mathcal L(d,p)=\int_{\Omega_0}[W_{\mathrm{iso}}(F)-p(J-1)]\,dV_0
 -\mathcal W_{\mathrm{ext}}(d).
\]

Here pressure is positive in compression and \(P=\partial_FW_{\mathrm{iso}}-p\operatorname{Cof}F\). Variation in \(p\) imposes volume moments. A nearly incompressible version adds \(-p^2/(2\kappa)\), giving \(p=-\kappa(J-1)\) in this particular volumetric model. Compressing \(J<1\) produces positive pressure. This is a different volumetric law from the logarithmic model of Chapter31.

The [deal.II v9.6.2 step44 source](https://github.com/dealii/dealii/blob/v9.6.2/examples/step-44/step-44.cc) implements a three-field displacement/pressure/dilatation approach with element-level condensation. Its particular spaces, pressure convention and energy should not be replaced by our two-field equations while retaining its stability claims.

## Inf-sup stability and pressure nullspaces

On a fixed geometry, incompressible Stokes uses velocity \(v\), test \(w\), pressure \(p\), test \(q\), \(v\) prescribed on essential boundaries and

\[
 a(v,w)=\int 2\eta D(v):D(w)\,dv,
 \qquad b(w,p)=-\int p\nabla\cdot w\,dv.
\]

The saddle system is \(Av+B^Tp=f\), \(Bv=0\) for a consistently chosen sign of \(B\). A pressure mode in \(\ker B^T\) exerts no discrete force. Unphysical modes beyond the legitimate gauge make pressure indeterminate. A full numerical rank check on one mesh is not a mesh-independent stability proof.

For appropriate norms a discrete inf-sup bound requires

\[
 \inf_{q_h\ne0}\sup_{w_h\ne0}
 \frac{|\int q_h\nabla\cdot w_h\,dv|}
 {\|w_h\|_{H^1}\|q_h\|_{L^2}}\ge\beta_0>0,
\]

uniformly over the admitted mesh family after removing actual pressure gauges. Coercivity on the divergence-free kernel and this bound support mixed well-posedness/error estimates under the other hypotheses. [Brezzi (1974), sections 1–2](https://www.numdam.org/article/M2AN_1974__8_2_129_0.pdf) is the original abstract analysis; applying it requires verifying its hypotheses for the chosen spaces and boundaries.

Continuous quadratic velocity/linear pressure (Taylor–Hood type) is a common mixed choice; uniform stability depends on dimension, domain, boundary conditions and mesh family. Equal-order P1/P1 generally needs a derived stabilization; simply adding a pressure diagonal changes the problem. The 2013 PFEM source uses a particular FIC stabilization and a tensile-positive pressure convention. Our choice below is an unstabilized P2/P1 reference branch, not that PFEM algorithm or a claim about all tetrahedral meshes.

Closed fully prescribed velocity boundaries usually leave pressure up to a constant, subject to compatible net flux. A free surface with prescribed absolute traction generally fixes the pressure level; subtracting an arbitrary mean then changes the load. Determine nullspaces from admissible tests and actual traction conditions. Time-dependent domains, constrained nodes and degenerating elements can destroy a previously valid bound.

## A defined moving-fluid timestep

Select a homogeneous incompressible Newtonian fluid, no thermal equation, \(\rho_0>0\), \(\eta\ge0\), three components, no remeshing during the attempted step. The initial reference geometry has \(J=1\). Use continuous P2 material bases \(N_a(X)\) for both motion and velocity, continuous P1 pressure bases \(Q_\alpha(X)\), and consistent reference mass \(M_{ai,bj}=\delta_{ij}\int\rho_0N_aN_b\,dV_0\). Store all P2 geometry nodes; a P2 velocity cannot silently move only vertices while edge nodes follow an unrelated rule.

Given accepted \(x_n,v_n\), seek nodal \(v,p\), with

\[
 x=x_n+hv,\quad F=\nabla_Xx,\quad J=\det F>0,
 \quad L=(\nabla_Xv)F^{-1},\quad D=\operatorname{sym}L,
 \quad p_h=\sum_\alpha Q_\alpha p_\alpha.
\]

Integrate Newtonian stress pulled back from this endpoint geometry:

\[
 P_f=J(-p_hI+2\eta D)F^{-T}.
\]

Choose the finite-step volume constraint explicitly, rather than alternating unspecified projection and mesh movement:

\[
 r^v_a=\sum_bM_{ab}(v_b-v_{n,b})/h+
          \int_{\Omega_0}P_f\nabla_XN_a\,dV_0-f_a^{\mathrm{ext}}=0,
 \qquad
 g_\alpha=-\frac1h\int_{\Omega_0}Q_\alpha(J-1)\,dV_0=0.
\]

This is a backward-Euler-like discretization of the position/velocity incompressible DAE with a nonlinear endpoint volume constraint. \(r^v\) has units N; \(g\) has units m\(^3\)/s. Positive row scaling must account for their different units. Body load \(\int\rho_0N_ab\,dV_0\) remains material mass times acceleration; external current traction requires a cofactor surface transformation and its derivative. Prescribed velocity determines prescribed endpoint positions through the same update. The branch admits only fixed topology and feasible boundary data.

These moment constraints do not imply \(J=1\) pointwise, nor \(\nabla_x\cdot v=0\) pointwise at the endpoint. Treat \(\rho_h=\rho_0/J\) as the density implied by discrete material mass and report its deviations from \(\rho_0\). Do not independently force current density to \(\rho_0\) while using deformed element volumes. If the constant function belongs to the complete tested pressure space, exact constraints and exact integration imply \(\int J\,dV_0=|\Omega_0|\), proving **total** volume preservation. Removing a gauge equation requires showing its volume condition is redundant for the boundary data, not silently deleting volume control.

A weak endpoint divergence constraint instead defines a different timestep. Even exact \(\nabla_x\cdot v_{n+1}=0\) does not preserve volume under a finite backward-Euler motion update. Set \(F_n=I\) and the constant endpoint spatial gradient \(L_{n+1}=\mathrm{diag}(a,-a,0)\). Differentiating \(x_{n+1}=X+h v_{n+1}(x_{n+1})\) gives \((I-hL_{n+1})F_{n+1}=I\), hence \(J_{n+1}=1/(1-h^2a^2)\). At \(ha=1/2\) it is \(4/3\), although the endpoint divergence is zero. A forward update using that gradient on the old geometry instead gives \(J_{n+1}=1-h^2a^2=3/4\); these are different time discretizations. Both hand counterexamples explain why geometry integration must be part of the incompressibility contract.

## Every Jacobian block is specified

At fixed material quadrature define \(H=F^{-T}\), \(C=JH\), and for a trial direction \((\delta v,\delta p)\),

\[
 \delta F=h\nabla_X\delta v,\quad
 \delta J=JH:\delta F,\quad
 \delta H=-H\delta F^TH,
 \quad \delta C=\delta JH+J\delta H.
\]

Differentiate the velocity gradient, including geometric dependence:

\[
 \delta L=(\nabla_X\delta v)F^{-1}
   -L\delta FF^{-1},\qquad \delta D=\operatorname{sym}\delta L.
\]

With \(\tau=-p_hI+2\eta D\),

\[
 \delta P_f=\delta J\tau H
        +J(-\delta p_hI+2\eta\delta D)H+J\tau\delta H,
\]

\[
 \delta r^v_a=\sum_bM_{ab}\delta v_b/h+
       \int\delta P_f\nabla_XN_a\,dV_0-\delta f_a^{\mathrm{ext}},
 \qquad
 \delta g_\alpha=-\frac1h\int Q_\alpha\,\delta J\,dV_0.
\]

These formulas directly implement all four tangent blocks as an action. The pressure-pressure block is zero; pressure enters \(\delta P_f\) through \(-\delta p_hC\). Define \(B_{\alpha,ai}=\int Q_\alpha C_{iA}\partial_AN_a\,dV_0\). The off-diagonal blocks are \(-B^T\) and \(-B\) with the chosen constraint scaling. The velocity block includes geometry/stress derivatives and is not generally symmetric. Freezing \(F\), dropping \(\delta D\)'s second term or using a lagged surface load produces an approximate tangent, not this consistent derivative. Pressure/motion coupling is not replaced with the zero analytic pressure of the existing restricted Rheon ALE study.

At fixed geometry, the Schur complement of the symmetric idealization is \(-BA^{-1}B^T\). A block preconditioner approximates the velocity solve and pressure Schur solve separately, handles true pressure nullspaces, and respects inertia-dominated versus viscosity-dominated scaling. Suitable pressure mass/Laplacian approximations depend on these limits and boundary conditions. For the fully differentiated nonsymmetric system use GMRES/flexible GMRES as appropriate. A finite-dimensional rank check and reported conditioning diagnose the current problem; they do not establish a uniform inf-sup constant for curved moving meshes.

## Free surfaces and fluid-specific limits

On a material fluid boundary \(\dot x=v\). With no surface tension and relative pressure measured from constant atmosphere, the natural free-surface condition is \(\sigma n=0\). This includes both pressure and viscous traction. Imposing \(p=0\) alone while ignoring \(2\eta Dn\) is generally different. Curvature force follows variation of surface energy \(\gamma A\), \(\gamma\) in N/m, with signs tied to the chosen outward normal. Its nodal residual and surface Hessian, if included, must use the same geometry. Wall contact lines need separate wetting/slip laws, as in existing chapters 23–25.

The new fluid residual is fully specified but has no unconditional energy theorem: endpoint pressure work need not vanish from the endpoint volume moments. Weak volume control permits local compression, and a P2 determinant can fail between samples. Mesh distortion persists even when volume is constant. Before adopting this branch, qualify temporal/spatial consistency, mixed stability on the actual family, pressure/traction convergence, geometric feasibility and finite-step work. None of these is established by a successful nonlinear solve alone.

A compressible extension requires a thermodynamic state, equation of state, internal-energy update and compatible momentum/work assembly. For an abstract force matrix \(F_c\), a pairing \(M_v\dot v=-F_c\mathbf1\), \(M_e\dot e=F_c^Tv\) cancels kinetic/internal power in exact semidiscrete arithmetic. Shock viscosity adds entropy and must be paired consistently; an arbitrary explicit RK integrator need not preserve fully discrete total energy. [Laghos v3.1 operator declaration](https://github.com/CEED/Laghos/blob/v3.1/laghos_solver.hpp) illustrates separate kinematic/thermodynamic spaces and force operators. It does not validate our incompressible equations.
