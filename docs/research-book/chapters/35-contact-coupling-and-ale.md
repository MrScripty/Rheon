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

Lagrangian motion has \(a=v\); a fixed Eulerian mesh has \(a=0\). Choosing a smooth interior mesh extension while matching boundary normal motion generally makes interior relative flux nonzero. The original [Hirt, Amsden and Cook (1974) bibliographic record](https://www.sciencedirect.com/science/article/pii/0021999174900515) concerns a finite-difference ALE algorithm, not automatic evidence for an FEM transfer. Donea, Huerta, Ponthot and Rodríguez-Ferran's 2004 encyclopedia chapter is a primary ALE survey; the [source map](../lagrangian/source-map.md) records its metadata and current access limitations.

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

The [research status report](../lagrangian/research-status-report.md) maps every unread reference to its actual manuscript use. In particular, the MPM row is a schematic distinction rather than a verified account of the 1994 Sulsky algorithm; no conservation or stability result is attributed to that unread paper.
