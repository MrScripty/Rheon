# 34 Incompressibility, locking and material-mesh fluids

A moving connected mesh does not choose the material law. Solids retain deformation-dependent shear stress; Newtonian fluids dissipate through velocity gradients. This chapter first explains mixed stability, then specifies a candidate incompressible moving-fluid timestep. The later isentropic-gas section selects a smaller compressible model for the complete algorithm in Chapter36. It is an independently derived reference formulation, **not a newly validated method or a theorem of nonlinear convergence**. Its derivative is specified so that [Chapter 36](36-reference-fem-solver.md) can give an implementable algorithm. No Rheon production capability is implied.

## Why displacement-only elements lock

In a nearly incompressible elastic law the bulk modulus \(\kappa\) is much greater than shear modulus. A displacement-only approximation minimizes a very large volumetric penalty while also trying to approximate shear. Low-order spaces may admit too few nearly volume-preserving modes, producing excessive stiffness that persists under practically affordable refinement. Increasing the penalty further can worsen conditioning and locking without improving shear accuracy. Reduced integration can relieve a constraint but introduce spurious modes. A mixed formulation makes pressure an independent unknown instead.

For example, in an exactly incompressible hyperelastic model choose an isochoric energy \(W_{\mathrm{iso}}(F)\) and the Lagrangian

\[
 \mathcal L(d,p)=\int_{\Omega_0}[W_{\mathrm{iso}}(F)-p(J-1)]\,dV_0
 -\mathcal W_{\mathrm{ext}}(d).
\]

Here pressure is positive in compression and \(P=\partial_FW_{\mathrm{iso}}-p\operatorname{Cof}F\). Variation in \(p\) imposes volume moments. A nearly incompressible version adds \(-p^2/(2\kappa)\), giving \(p=-\kappa(J-1)\) in this particular volumetric model. Compressing \(J<1\) produces positive pressure. This is a different volumetric law from the logarithmic model of Chapter31.

Adding a separate dilatation variable would produce a different three-field formulation. Its elimination, discrete spaces and stability would need their own derivation; the two-field equations above do not inherit another formulation’s guarantees.

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

## Selected complete material-mesh fluid: smooth isentropic gas

The mixed incompressible method above remains an advanced candidate whose moving-space stability and finite-step work require further research. To give the reader one closed fluid algorithm now, [Chapter36](36-reference-fem-solver.md) selects a **compressible inviscid barotropic gas** on P1 material tetrahedra. Its pressure is computed from density, not solved as an independent multiplier. It has no mixed inf-sup condition or pressure gauge. This is a deliberately smaller fluid model, not an implementation or validation claim.

Start with the perfect-gas relations from [Chapter26](26-compressible-state-and-waves.md), absolute pressure \(p=(\gamma-1)\rho e\), \(\gamma>1\). For smooth adiabatic inviscid material motion, \(de=p\,d\rho/\rho^2\). Substitution gives \(de/e=(\gamma-1)d\rho/\rho\), so \(e=e_0(\rho/\rho_0)^{\gamma-1}\) on an isentrope. With \(\rho=\rho_0/J\), reference pressure \(p_0=(\gamma-1)\rho_0e_0>0\), and fixed reference density,

\[
 \rho=\rho_0/J,\qquad e=e_0J^{1-\gamma},\qquad
 p=p_0J^{-\gamma},\qquad c^2=\gamma p/\rho.
\]

All powers act on dimensionless \(J\), so no dimensional reference is hidden in a fractional power. \(p\) is absolute thermodynamic pressure in Pa, \(e\) is J/kg and \(c\) is m/s. The material entropy parameter \(p_0/\rho_0^\gamma\) is constant along a trajectory; the selected core uses the same value in all elements. A frozen isentrope does not describe viscous/shock heating.

Define physical internal energy per reference volume

\[
 W_g(F)=\rho_0e=\frac{p_0}{\gamma-1}J^{1-\gamma},\quad J>0.
\]

This is volumetric internal energy, not elastic shear energy. Since \(dW_g/dJ=-p\), the variational pressure stress and Cauchy stress are

\[
 P_g=-p\operatorname{Cof}F=-pJ F^{-T},\qquad
 \sigma_g=J^{-1}P_gF^T=-pI.
\]

**Checked constitutive derivative.** Put \(H=F^{-T}\), \(Z=\delta F\), \(q=pJ\). Then \(\delta q=(1-\gamma)q(H:Z)\), \(\delta H=-HZ^TH\), and

\[
 \mathcal A_g[Z]=(\gamma-1)q(H:Z)H+qHZ^TH,
 \quad
 (\mathcal A_g)_{iAjB}=(\gamma-1)qH_{iA}H_{jB}+qH_{iB}H_{jA}.
\]

For constant element density/entropy and affine P1 motion, \(F,J,p,W_g,P_g,\mathcal A_g\) are constant within a tetrahedron. The element internal force is \(|K_0|P_g\nabla_XN_a\); the tangent is the same contraction as Chapter32 with \(\mathcal A_g\). Reference mass stays constant. No pressure interpolation, density advection, pressure pin, energy clipping or independent volume update is added to this selected model.

At rest \(F=I\), \(P_g=-p_0I\), so gas is **not** stress-free. Rigid rotation keeps \(J,p,W_g\) unchanged and Cauchy stress isotropic, but does not make stress vanish. A closed box supplies normal wall reactions. On the fixed coordinate walls the admissible tests satisfy \(w\cdot n=0\); a uniform pressure gives zero free residual because \(-p_0\int\nabla\cdot w=-p_0\int_{\partial\Omega}w\cdot n=0\). Discarding constrained force components would hide the force on the container.

Although \(W_g''(J)=\gamma p/J>0\), convexity in scalar \(J\) does not imply convexity in nodal displacement because determinant is nonlinear. The earlier conditional backward-Euler energy result is not upgraded to a universal theorem. Near uniform rest on a fixed box, the second variation of total volume vanishes for admissible variations, so the assembled gas stiffness reduces to \(\int\gamma p_0(\nabla\cdot w)(\nabla\cdot z)\,dV_0\). It penalizes compression and has solenoidal zero-stiffness modes, consistent with the absence of elastic shear modulus. Inertia makes the linearized backward-Euler operator positive definite on this finite velocity space; this local observation is not a nonlinear coercivity or convergence theorem.

For \(\rho_0=p_0=1\) in SI and \(\gamma=5/3\), reference specific energy is \(e_0=3/2\) J/kg and sound speed squared is \(5/3\) m\(^2\)/s\(^2\). A material constitutive check at isotropic compression \(F=I/2\) gives \(J=1/8\), \(\rho=8\), \(p=32\), \(e=6\), \(W_g=6\), and \(P_g=-8I\), with corresponding SI units. That compression violates Chapter36's restrictive mesh guard and fixed-wall boundary data; it is an element-law check, not an admissible whole-box step. For an infinitesimal breathing increment \(Z=\epsilon I\) at rest, the tangent gives \(\delta P_g=(3\gamma-2)p_0\epsilon I=3p_0\epsilon I\), agreeing with direct differentiation of \(P_g(sI)=-p_0s^{2-3\gamma}I\).

## From the selected isentrope to a full thermal fluid

The barotropic core closes a smooth material momentum problem with internal energy prescribed by density. It does not independently evolve a thermal degree of freedom, enforce an entropy inequality through shocks or convert numerical time-stepping losses into heat. Pressure/density jumps at shocks, cavitation, heat conduction and viscosity require a larger model. Mesh/admissibility checks cannot certify that a smooth PDE solution still exists; a passing algebraic step is not a shock-detection certificate.

A concrete next semidiscrete thermal choice is continuous P1 velocity and cellwise constant specific energy \(e_e\) on each material tetrahedron, with \(m_e=\rho_{0,e}|K_{0,e}|\). Let \(V_e=J_e|K_{0,e}|\), \(L_e=(\nabla_Xv_e)F_e^{-1}\), \(\vartheta_e=\mathrm{tr}L_e\), \(p_e=(\gamma-1)(m_e/V_e)e_e\). For adiabatic inviscid flow with no external power (including stationary normal walls with v dot n zero), define

\[
 f^{\mathrm{int}}_{e,a}=|K_{0,e}|(-p_eJ_eF_e^{-T})\nabla_XN_a,
 \quad M_0\dot v=-f^{\mathrm{int}}+f_{\mathrm{support}},
 \quad m_e\dot e_e=-p_eV_e\vartheta_e.
\]

For stationary essential constraints write \(v=Rz\), \(R^Tf_{\mathrm{support}}=0\) and solve \(R^TM_0R\dot z=-R^Tf^{\mathrm{int}}\); the full balance retains the wall reactions rather than setting them to zero. Thus \(v^Tf_{\mathrm{support}}=0\). Dotting momentum with nodal velocity gives \(\dot T=\sum_ep_eV_e\vartheta_e\); summing the cell internal-energy equations cancels this power exactly in the semidiscrete, exact-arithmetic model. Compression \(\vartheta_e<0\) raises internal energy. A fully discrete integrator still needs a compatible work update and positive-energy acceptance; ordinary independent endpoint updates do not inherit exact total-energy conservation.

For a compressible Newtonian stress \(\sigma=-pI+\tau\), with \(\tau=2\eta\operatorname{dev}D+\zeta\vartheta I\), \(\eta,\zeta\ge0\), add the same viscous stress to nodal momentum and \(V_e\tau:D\) to internal heating. The latter equals \(V_e[2\eta|\operatorname{dev}D|^2+\zeta\vartheta^2]\ge0\). Boundary heat flux and intercell heat exchange need their own conservative pairing. Artificial shock viscosity must be separately declared and shown to produce admissible heating; an arbitrary velocity smoother is not a physical thermodynamic closure.

The inspected [Laghos v3.1 operator source](https://github.com/CEED/Laghos/blob/v3.1/laghos_solver.cpp), blob `2a04477bca658a5fa7731691a505fc6df25dc3f8`, contains an explicit position/velocity/specific-energy state and the ideal-gas pressure/sound-speed calculation near lines1005–1008. Its shared force operation connects momentum and energy. This is accessible primary source evidence; no unread Dobrev article theorem is needed for the algebra above. The inspected kernel also floors a sampled energy before evaluating pressure; that implementation choice is **not** imported. Our core rejects nonpositive/invalid constitutive states instead. High-order spaces, artificial viscosity, device kernels and Laghos's own time-integration details remain distinct from this book's low-order implicit algorithm.
