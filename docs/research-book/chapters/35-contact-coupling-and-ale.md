# 35 Contact, fluid–solid coupling and changing meshes

The core solvers in [Chapters 33–34](33-time-and-nonlinear-solves.md) evolve fixed connectivity. Real applications add collision, interface loads, large distortion and topology changes. These additions alter equations and state transfer; they are not postprocessing. This chapter derives their contracts and identifies the algorithms still needing separate implementation/qualification. It does not repeat the existing D1 product design.

## Contact is a constrained mechanics problem

For a frictionless deformable body against an obstacle, let signed gap \(g(x)\) be positive in the admissible separated region. Its gradient points toward increasing separation. A normal contact multiplier \(\lambda_c\ge0\), with units consistent with its discrete gap/area representation, gives the complementarity conditions

\[
 g\ge0,\qquad\lambda_c\ge0,\qquad\lambda_cg=0.
\]

For a discrete gap measured in metres and an integrated multiplier in newtons, the contact contribution to the equilibrium residual is \(-\lambda_c\nabla g\); the applied force is its opposite. An active-set solve chooses contact candidates, solves the coupled constrained system, and checks both gap feasibility and multiplier signs before accepting. Candidate activation/deactivation is nonsmooth; classical smooth Newton theory does not directly cover the switching point.

A penalty energy \(E_c=(\epsilon_c/2)\min(g,0)^2\), \(\epsilon_c\) in N/m for a single gap, has residual \(\epsilon_c\min(g,0)\nabla g\). It permits penetration and adds stiffness. In the active region its tangent is \(\epsilon_c[\nabla g\otimes\nabla g+g\nabla^2g]\); omitting the second term is an approximation on curved geometry. Increasing stiffness changes explicit frequency bounds and nonlinear conditioning. Augmented Lagrangian and mortar methods offer other discrete constraints; none follows merely from a bounding-volume hierarchy.

For Coulomb friction, the tangential traction opposes slip and obeys a stick/slip cone \(\|t_T\|\le\mu_c\lambda_c\), with \(\mu_c\) dimensionless under matching traction units. Sliding satisfies \(t_T\cdot v_{\mathrm{slip}}\le0\). A regularization needs its own velocity scale, dissipation and limiting behavior. Solid Coulomb friction, fluid Navier wall slip and wetting/contact-angle laws are different closures. The earlier [Chapter24](24-slip-and-wall-friction.md) owns Rheon's scoped fluid slip studies.

Search finds possible contacting primitives; mechanics determines gap enforcement and reaction work. Broad-phase bounds must cover the actual attempted motion. Continuous collision detection checks the path, not just endpoint overlap; spatial self-contact can occur despite positive element determinants. [Sierra/SM 5.30 contact chapter](https://www.sandia.gov/files/sierra/SM_Theory_5_30/main/contact.html) explicitly separates search from enforcement. The [Li et al. (2020) IPC author page](https://ipc-sim.github.io/) reports intersection- and inversion-free elastodynamics. The full paper and supplements were not reviewed here, so those are author-reported claims rather than independently checked guarantees. The generic contact equations above are not an IPC implementation, and an arbitrary logarithmic penalty does not inherit IPC claims.

A barrier's domain excludes contact/intersection rather than tolerating penalty penetration. It therefore needs a feasible initial configuration, collision-safe step selection, finite nonlinear tolerances and treatment of imposed boundary trajectories. The new reference solver makes no IPC implementation or formal guarantee claim.

## Interface kinematics, action/reaction and power

At a fitted interface \(\Gamma\), let normals \(n_f\) and \(n_s=-n_f\) point outward from fluid and solid. Perfect no-slip coupling imposes \(v_f=v_s\). Traction equilibrium is

\[
 \sigma_fn_f+\sigma_sn_s=0.
\]

Adding fluid and solid weak forms cancels interface power when tests and velocities match. With slip, only normal velocities match; tangential traction contributes a declared nonnegative loss. A rigid solid's force and torque require integrated tractions and consistent moment arms, as derived in [Chapter29](29-liquid-solid-interaction.md). Point-collision impulses alone do not establish resolved hydrodynamic loading.

For nonmatching interface nodes, suppose a fixed linear map \(H\) transfers solid velocity to fluid interface velocity, \(v_f=Hv_s\). If \(f_f\) denotes the interface load applied to the fluid in integrated nodal-force coordinates, choose the solid reaction \(f_s=-H^Tf_f\). Then

\[
 v_f^Tf_f+v_s^Tf_s=(Hv_s)^Tf_f-v_s^TH^Tf_f=0.
\]

This is a conditional discrete work proof. If the stored quantities are traction samples instead of integrated nodal forces, quadrature/mass metrics enter the adjoint map. Conservation of torque further requires reproduction of rigid rotations/coordinates; a partition-of-unity transfer gives translation consistency alone. Interpolating displacement and force independently can inject energy even if their pictures match.

Monolithic coupling solves fluid, solid and interface equations together, requiring block preconditioning and consistent interface derivatives. Partitioned coupling reuses separate solvers but needs iterative exchange, relaxation or quasi-Newton acceleration and a common convergence/work criterion. A toy added-mass iteration illustrates risk: if incompressible fluid reaction is \(-m_a a^{(k)}\), a partitioned solid solve gives \(m_s a^{(k+1)}=f-m_aa^{(k)}\). Acceleration errors satisfy \(e^{(k+1)}=-(m_a/m_s)e^{(k)}\). Under these scalar assumptions, iteration diverges for \(m_a>m_s\). This is our elementary model, not a substitute for a full PDE analysis. [Causin, Gerbeau and Nobile (2005)](https://www.sciencedirect.com/science/article/pii/S0045782504005328) is the original added-mass study; only its primary bibliographic/search record was available in this session, not its restricted full text.

## Mesh distortion, free boundaries and topology

Track minimum determinant, scaled Jacobian, aspect ratio, short altitude, boundary intersections and conditioning. A positive volume can coexist with a nearly singular long thin element, destroying accuracy and timestep estimates. Smooth volume-preserving shear from Chapter30 is an explicit example. Curved high-order elements need domain-wide positivity control; checking a few samples is only a diagnostic.

A material free surface follows the solved velocity and represents a sharp boundary until the topology fails. Overturning is allowed by a general 3D mesh but not by the existing fitted-height graph. Breakup and merging need a domain reconstruction rule, new boundary conditions, and conservative reassignment of state. Alpha shapes and Delaunay remeshing in PFEM are choices that can add/remove volume; they must be assessed rather than treated as exact geometry facts. Solid remeshing must transfer deformation/history relative to material, not declare the current shape stress-free. Deleting material or reconnecting through a wall changes the physical problem.

## ALE separates computational and material motion

Let \(x=\mathcal A(\xi,t)\) be a computational map, \(a=\partial_t\mathcal A|_\xi\) its mesh velocity. For any spatial field \(s\),

\[
 \frac{Ds}{Dt}=\partial_t s|_\xi+(v-a)\cdot\nabla_xs.
\]

Lagrangian motion has \(a=v\); a fixed Eulerian mesh has \(a=0\). Choosing a smooth interior mesh extension while matching boundary normal motion generally makes interior relative flux nonzero. The original [Hirt, Amsden and Cook (1974) bibliographic record](https://www.sciencedirect.com/science/article/pii/0021999174900515) concerns a finite-difference ALE algorithm, not automatic evidence for an FEM transfer. Donea, Huerta, Ponthot and Rodríguez-Ferran's 2004 encyclopedia chapter is a primary ALE survey; the [source map](../implementation/lagrangian-fem-sources-and-scope.md) records its metadata and current access limitations.

For a moving computational cell \(K(t)\), Reynolds' theorem gives

\[
 \frac{d}{dt}\int_{K(t)}\rho\,dv
 +\int_{\partial K(t)}\rho(v-a)\cdot n\,da=0,
 \qquad
 \frac{d|K(t)|}{dt}=\int_{\partial K(t)}a\cdot n\,da.
\]

The second equation is the geometric conservation law (GCL). Momentum has relative advective flux \(\rho v\otimes(v-a)\), stress traction and body force. Energy needs matching pressure/viscous work and transport. Geometry volumes and face-time fluxes must describe the same trajectory. Matching only endpoint mass marginals does not prove that face fluxes are physical; divergence-free face circulations can preserve all marginals while moving momentum incorrectly.

A simple one-dimensional GCL witness is a cell \([0,\ell(t)]\): \(\dot{|K|}=\dot\ell\). If constant density is stored and physical \(v=0\), the outward relative mass flux at the moving right face is \(-\rho\dot\ell\); it cancels the cell mass increase. Omitting this mesh flux creates mass from moving geometry. For a truly material right boundary \(v=\dot\ell\), relative flux vanishes and density must change unless length remains constant.

Rheon's [fitted-height formulation](../implementation/fitted-height-liquid-formulation.md) and [native fitted ALE step](../implementation/fitted-ale-native-step.md) already derive a specific compatible moving-mass/work ledger. They remain restricted non-overturning, periodic-extruded studies; the native family \(U=(a,0,w)\) has analytic zero relative pressure. They are valuable precedents for custody and GCL provenance, not implementations of this chapter's general three-dimensional fluid/solid equations.

## Conservative remap and what it can lose

A Lagrange–remap algorithm first advances material state, then rezones a mesh and transfers state at the same physical time. A remesh is not an additional physical timestep. For piecewise constant old density and velocity on old cells \(K_i\), and a new tessellation \(K'_j\) of the **same** physical domain, define exact overlap volumes \(V_{ji}=|K'_j\cap K_i|\):

\[
 m'_j=\sum_i\rho_iV_{ji},\qquad
 q'_j=\sum_i\rho_iV_{ji}v_i,\qquad v'_j=q'_j/m'_j\quad(m'_j>0).
\]

If overlaps partition every old cell exactly, summing over \(j\) proves total mass and momentum preservation. Nonnegative overlaps preserve density positivity. New domains with gaps, overlap, material mixing or changed boundaries invalidate the premise. Floor masses or ad hoc redistribution change the transfer and require their own balance.

For continuous FEM, a density-weighted Galerkin projection on a common domain solves

\[
 M^{\mathrm{new}}v^{\mathrm{new}}=C v^{\mathrm{old}},\quad
 M^{\mathrm{new}}_{jk}=\int\rho N'_jN'_k\,dv,\quad
 C_{ji}=\int\rho N'_jN_i\,dv.
\]

Mixed products require integration across old/new intersections, not just point samples on one mesh. If both bases reproduce constants and integration is exact, testing with constant velocity proves momentum conservation. Orthogonality in this same positive density-weighted inner product proves \(\|v^{\mathrm{new}}\|_\rho\le\|v^{\mathrm{old}}\|_\rho\): write the old field as its projection plus an orthogonal error. It may damp kinetic energy and does not generally preserve pointwise positivity, extrema, constitutive admissibility or divergence constraints. It also preserves angular momentum if the unconstrained new space contains every rigid-rotation test field, the domain and density metric are unchanged, and integration is exact: test projection orthogonality with each field x crossed with a Cartesian unit vector. Standard affine P1/P2 spaces reproduce those linear fields, but constrained or inconsistent transfers can lose this property. Density transfer and subsequent pressure projection must be included in the combined ledger. [Farrell and Maddison (2011), 200:89–100](https://www.sciencedirect.com/science/article/pii/S0045782510002276) is an original conservative Galerkin-transfer reference; full-text access was unavailable here, so the proof just given is our conditional derivation.

Two equal one-kilogram cells with velocities \(+1\) and \(-1\) m/s have zero momentum and one joule kinetic energy. Combining them into one two-kilogram cell by conservative averaging gives velocity zero and zero kinetic energy. Conservation of mass/momentum has not conserved kinetic energy. An energy-conserving compressible transfer could send the lost kinetic energy to internal energy, but that is a deliberate thermodynamic update. For a viscous incompressible fluid it is numerical dissipation to measure, not physical viscosity to claim.

## Distinguish neighboring methods

| Method | Where equations/forces live | What represents material | Main distinction from connected material-mesh FEM |
| --- | --- | --- | --- |
| Connected Lagrangian FEM | Integrals on persistent moving elements | Element material map and nodal fields | Basis connectivity moves and distorts with matter |
| PFEM | FEM on repeatedly generated moving meshes | Moving nodes plus reconstructed domain/history | Remeshing and boundary reconstruction are essential stages |
| SPH | Kernel sums on neighborhoods | Mass-carrying particles | No conforming element partition or standard FEM stiffness |
| PBF | Iterative particle density constraints | Positions and reconstructed velocities | Approximate positional correction is not a mixed FEM weak form |
| PIC/FLIP/APIC | Grid evolution with particle/grid transfers | Particles and selected moments | Grid is not a persistent connected material mesh; transfer changes energy/noise |
| MPM (schematic terminology; original algorithm not source-audited here) | Background-grid weak form with material-point quadrature/history | Material points carrying deformation/history | Uses FE-like grid bases but resets/changes particle-grid support; no persistent material element connectivity |
| ALE FEM | Weak form on independently moving mesh plus relative transport | Field/state plus a computational map | Mesh motion differs from material motion and needs GCL/transport |

[Chapter11](11-particle-methods.md) contains primary PIC/FLIP/APIC/PBF references and transfer formulas. [Sulsky, Chen and Schreyer (1994)](https://www.sciencedirect.com/science/article/pii/0045782594901120) is an original material-point reference, verified here at bibliographic level only. These method families can share conservation principles and linear algebra while retaining different geometry, quadrature, state and failure modes. Calling every moving particle system a Lagrangian FEM solver would erase precisely the choices a solver implementer must make.

The [publication source and scope audit](../implementation/lagrangian-fem-sources-and-scope.md) distinguishes inspected text from bibliographic provenance. In particular, the MPM row is a schematic distinction rather than a verified account of the 1994 Sulsky algorithm; no conservation or stability result is attributed to that unread paper.

## Extension choices after the two minimal algorithms

[Chapter36](36-reference-fem-solver.md) now fixes complete P1 solid and inviscid isentropic-gas paths. The following original derivations expose the extra equations needed to extend them; they are not flags in those algorithms and have not been executed or machine-checked. The existing incompressible viscous candidate and a full thermal fluid require their own state/evidence, as explained in Chapter34.

### ALE weak transport must use the actual mesh trajectory

Let scalar test \(N_a(x,t)\) move with computational velocity a, so \(\partial_tN_a|_x=-a\cdot\nabla N_a\). Put \(c=v-a\), nodal tested mass \(m_a=\int_{\Omega(t)}\rho N_a\,dv\), and tested momentum \(q_a=\int\rho vN_a\,dv\). Reynolds' theorem and integration by parts give

\[
 \dot m_a-\int_{\Omega(t)}\rho c\cdot\nabla N_a\,dv
 +\int_{\partial\Omega(t)}\rho N_ac\cdot n\,da=0,
\]

\[
 \dot q_a-\int\rho v(c\cdot\nabla N_a)\,dv
 +\int\sigma\nabla N_a\,dv
 +\int_{\partial\Omega}N_a[\rho v(c\cdot n)-\sigma n]\,da
 -\int\rho N_ab\,dv=0.
\]

For total energy per current volume \(\mathcal E=\rho e+\rho|v|^2/2\), heat flux \(q_h\) and volumetric heat source Q, define \(E_a=\int\mathcal E N_a\,dv\). Its relative flux is

\[
 \mathcal F_E=\mathcal E(v-a)-\sigma v+q_h,
 \quad
 \dot E_a-\int\mathcal F_E\cdot\nabla N_a\,dv
 +\int_{\partial\Omega}N_a\mathcal F_E\cdot n\,da
 =\int N_a(\rho b\cdot v+Q)\,dv.
\]

These equations distinguish physical stress power from mesh transport. Setting a=v removes only relative advective transport; it does not remove pressure/viscous work from thermal energy. In a fixed mesh a=0 they recover the conservative spatial weak form. They are continuous weak identities, not a chosen upwind, stabilized or fully discrete ALE method. A future scheme must select fluxes, moving-basis quadrature, time integration, boundary states and a discrete GCL on the same trajectory. Choosing new node positions from smoothing alone does not determine those fluxes.

A conservative rezone at one physical time offers another path. On a common old/new intersection tessellation, transfer mass, momentum and (for a thermal model) total energy using their actual old fields. With P1 old velocity and cellwise density/energy, kinetic-energy density is quadratic on old tetrahedra, so even exact overlap volumes alone are insufficient: momentum/energy need corresponding moments or adequate intersection quadrature. Then compute new velocity from a declared mass metric and internal energy from total minus kinetic energy. Require positive mass and internal energy; reject an inadmissible transfer rather than floor it.

The selected isentropic model has no independent thermal energy or entropy degree of freedom. A dissipative remap may preserve mass/momentum while losing kinetic energy, but it cannot both convert that loss to heat and retain the same frozen isentrope without changing the model. Either keep a declared barotropic mechanical-loss ledger, or move to the thermal extension and transfer its entropy/energy consistently. This choice is part of physics/state ownership, not a cosmetic interpolation option.

### Capillarity has a concrete surface residual and tangent

For a current surface triangle with positions \(x_a,x_b,x_c\), define \(u=x_b-x_a\), \(w=x_c-x_a\), \(c_s=u\times w\), area \(A_s=|c_s|/2\), and unit normal \(n=c_s/|c_s|\). Constant surface tension \(\gamma_s\), N/m, has energy \(\gamma_sA_s\). Its **internal residual**, whose negative is the applied capillary force, is

\[
 f_b^\gamma=\tfrac12\gamma_s(w\times n),\qquad
 f_c^\gamma=\tfrac12\gamma_s(n\times u),\qquad
 f_a^\gamma=-f_b^\gamma-f_c^\gamma.
\]

The derivative in an arbitrary nodal direction is implementable without choosing a separate pointwise curvature sign:

\[
 \delta c_s=\delta u\times w+u\times\delta w,\quad
 \delta n=(I-nn^T)\delta c_s/|c_s|,
\]
\[
 \delta f_b^\gamma=\tfrac12\gamma_s(\delta w\times n+w\times\delta n),
 \quad
 \delta f_c^\gamma=\tfrac12\gamma_s(\delta n\times u+n\times\delta u),
 \quad\delta f_a^\gamma=-\delta f_b^\gamma-\delta f_c^\gamma.
\]

The gradient has force units because \(\nabla_xA_s\) has length units. A degenerate triangle has undefined n and must be rejected, not repaired by an arbitrary area floor. Translation invariance gives zero resultant, and rotation invariance gives zero net internal torque in exact consistent assembly. For the unit right triangle in the xy plane, area1/2 and \(n=e_3\), the gradients are \(f_b^\gamma=\gamma_se_1/2\), \(f_c^\gamma=\gamma_se_2/2\), \(f_a^\gamma=-\gamma_s(e_1+e_2)/2\), a hand sign check.

For a closed interface against constant ambient absolute pressure \(p_{\mathrm{ext}}\), include \(p_{\mathrm{ext}}V+\gamma_sA\) in the potential in addition to the interior energy. On a smooth sphere of radius R, stationarity gives \(-p4\pi R^2+p_{\mathrm{ext}}4\pi R^2+8\pi\gamma_sR=0\), hence \(p-p_{\mathrm{ext}}=2\gamma_s/R\). This conditional continuum calculation fixes the outward-normal/pressure sign; it is not an executed mesh droplet. A contact line adds solid/interface energies and an explicit wall-slip/contact-angle model. Neither the fixed-box gas nor clamped-solid core currently contains these boundaries.

### Contact and FSI require a selected kinematic interface law

A simple fixed-plane node contact could use \(g=n\cdot x-c_0\), \(|n|=1\), g≥0. The active penalty residual \(\epsilon_cg n\) and tangent \(\epsilon_cnn^T\), for g<0, follow from the earlier penalty energy. This closes a point-plane penalty law but deliberately permits penetration. It does not discretize distributed surface contact, Coulomb slip, moving finite facets or global self-contact. A multiplier alternative must solve complementarity, and an IPC alternative needs its own inspected barrier/collision algorithm. Those are separate numerical choices; the source-access caveats remain.

For a fixed discrete FSI interface map, select matrices \(C_s,C_f\) acting on solid/fluid velocities and a multiplier λ. **Normal-only** matching is the natural inviscid impermeability choice for selected gas G; full vector no-slip requires a viscous interface model or an explicitly different welded model. Write semidiscrete balances

\[
 M_s\dot v_s+f_s^{\mathrm{int}}+C_s^T\lambda=f_s^{\mathrm{ext}},\quad
 M_f\dot v_f+f_f^{\mathrm{int}}-C_f^T\lambda=f_f^{\mathrm{ext}},\quad
 C_sv_s-C_fv_f=0.
\]

The applied interface powers cancel because their sum is \(-\lambda^T(C_sv_s-C_fv_f)\). Matrices must include the actual interpolation, normals and integration metrics. A visual match of interface nodes is not that work proof. With fixed maps and initially compatible positions, backward-Euler displacement coupling uses \(c=C_s\Delta d_s-C_f\Delta d_f=0\). Its Newton blocks have the form

\[
 \begin{bmatrix}K_s&0&C_s^T\\0&K_f&-C_f^T\\C_s&-C_f&0\end{bmatrix}
 \begin{bmatrix}\delta d_s\\\delta d_f\\\delta\lambda\end{bmatrix}
 =-\begin{bmatrix}r_s\\r_f\\c\end{bmatrix}.
\]

Force and displacement rows have different units and require explicit scaling. For moving normals or changing nonmatching interpolation, differentiate the **actual nonlinear** interface residual including \(\delta C^T\lambda\) and kinematic derivatives; the fixed-map block alone is not a general FSI tangent. The coupling-force multiplier λ is independent of the gas thermodynamic pressure p. That p is EOS-derived in gas G; the mixed incompressible candidate instead has its own pressure multiplier. The FSI λ, EOS pressure and incompressibility multiplier must not be silently interchanged.

A monolithic implementation would require interface constraint rank/gauge checks, compatible initial conditions, a saddle solve/preconditioner and coupled nonlinear feasibility. A partitioned implementation would additionally require a specified iteration, acceleration/relaxation, common convergence/work gate and bounded failure. The earlier scalar added-mass recurrence explains a possible failure mechanism but selects none of those algorithms. Thus the extension equations are more explicit than a survey, while advanced general contact/FSI remains distinct from the two complete minimal core paths.
