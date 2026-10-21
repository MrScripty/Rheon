# 40 Fluid portals and open boundaries

A box boundary is part of the physical model. It decides whether a wake returns, whether an exterior reservoir supplies pressure and incoming fluid, and whether walls carry the reaction to a propeller. A boundary that merely looks transparent can still impose a closed pressure operator or remove momentum invisibly.

This chapter develops two distinct models: **periodic transport**, which identifies paired faces and recirculates their contents, and **open reservoir exchange**, which approximates an exterior without returning the outgoing wake. Neither is automatically an unbounded ocean. The equations and conditional proofs below are original exposition; no portal implementation, machine-checked portal theorem, new numerical acceptance or executed portal simulation is claimed.

Read [the continuum equations](02-continuum-equations.md), [MAC geometry](03-staggered-grids.md), [projection](04-pressure-projection.md), [boundary work](08-boundary-geometry.md), [liquid surfaces](09-liquid-surfaces.md), and [particle methods](11-particle-methods.md) first. The [source/scope guide](../implementation/fluid-portals-sources-and-scope.md) owns literature and licensing provenance. The separate [implementation proposal](../../plans/fluid-portals-open-boundaries/plan.md) owns future engineering decisions and admission; this chapter does not authorize them.

## 40.1 Select the exterior before selecting a numerical boundary

A translation-periodic pair represents two traces of the same fluid region. Outflow on one trace is inflow on the other, retaining the transported velocity and material state. A pressure drop may drive this repeated domain, but it is an additional pump model.

A reservoir opening represents surrounding fluid with a specified equilibrium/reference state and a model for incoming disturbances. Outgoing fluid can leave the retained control volume without reappearing at another face. Incompressibility still constrains the total flux; opposite-face volumes do not need to match individually.

A fully filled constant-density domain is different from liquid with a free surface. A submerged top opening models water above the truncation, not an air interface. An open bottom models deeper water, not a seabed. For a liquid-air interface, wetness, surface motion and stress jump need their own consistent boundary conditions. For compressible or weakly compressible particles, pressure-density waves require the actual model's characteristics rather than an incompressible pressure shortcut.

## 40.2 Conservation identifies what a boundary may exchange


The following control-volume derivations are original to this report. Assumptions are stationary domain \(\Omega\), constant density \(\rho>0\), incompressible Newtonian fluid, and sufficiently regular fields. A free surface, moving solid, compressible gas, phase change or variable density requires additional terms. Write outward normal \(\mathbf n\), normal speed \(u_n=\mathbf u\cdot\mathbf n\), body acceleration \(\mathbf a=\mathbf g+\mathbf f\), and symmetric stress

\[
\boldsymbol\sigma=-p\mathbf I+2\mu\boldsymbol\varepsilon(\mathbf u),
\qquad \boldsymbol\varepsilon(\mathbf u)=\tfrac12(\nabla\mathbf u+(\nabla\mathbf u)^T).
\]

The equations are

\[
\nabla\cdot\mathbf u=0,\qquad
\rho\bigl(\partial_t\mathbf u+\mathbf u\cdot\nabla\mathbf u\bigr)
=\nabla\cdot\boldsymbol\sigma+\rho\mathbf a.
\tag{40.1}
\]

Integrating continuity gives \(\int_{\partial\Omega}u_n\,dA=0\) for a filled, fixed-volume domain. Fluid can cross many faces, but their **total volume flux must balance** at each accepted incompressible state. There need not be a one-to-one left/right replacement. A propeller may pull fluid in from side faces as well as upstream. With free surfaces, represented liquid volume can instead change as the interface moves; with compressibility, mass changes can alter density.

For a material fraction \(c\) without diffusion or sources,

\[
\frac{d}{dt}\int_\Omega \rho c\,dV
=-\int_{\partial\Omega}\rho c\,u_n\,dA.
\tag{40.2}
\]

An upwind boundary value is the interior value for \(u_n>0\) and specified exterior value for \(u_n<0\). Heat, dye, salinity, material fractions and particle mass need their own declared physical meaning and ledgers. A passive appearance concentration is not conserved water volume or thermodynamic energy.

Integrating (40.1) in conservative form gives the momentum balance

\[
\frac{d\mathbf P}{dt}=
-\int_{\partial\Omega}\rho\mathbf u u_n\,dA
+\int_{\partial\Omega}\boldsymbol\sigma\mathbf n\,dA
+\int_\Omega\rho\mathbf a\,dV,
\quad \mathbf P=\int_\Omega\rho\mathbf u\,dV.
\tag{40.3}
\]

Taking its inner product with velocity and integrating by parts yields

\[
\frac{dK}{dt}=
\int_\Omega\rho\mathbf a\cdot\mathbf u\,dV
-\int_\Omega2\mu\,\boldsymbol\varepsilon:\boldsymbol\varepsilon\,dV
+\int_{\partial\Omega}\left[
\mathbf u\cdot\boldsymbol\sigma\mathbf n
-\tfrac12\rho|\mathbf u|^2u_n\right]dA,
\quad K=\int_\Omega\tfrac12\rho|\mathbf u|^2dV.
\tag{40.4}
\]

Thus a good boundary is not necessarily zero-work or energy-preserving. A reservoir deliberately exchanges energy and momentum. Report advected mass/volume, advected momentum, pressure and viscous traction, kinetic-energy flux, gravity/actuator work, dissipation, stabilization work and numerical budget defects separately. A small divergence norm alone establishes none of the other balances. For the preview SPH model, a complete thermodynamic energy equation is not present; do not label its temperature or EOS pressure a validated total-energy budget.

## 40.3 Periodic topology, gravity, and momentum


### 40.3.1 Simple opposing faces

For \(x\in[0,L_x)\), translation periodicity imposes

\[
\mathbf u(0,y,z)=\mathbf u(L_x,y,z),\quad
c(0,y,z)=c(L_x,y,z),\quad
p(0,y,z)=p(L_x,y,z),
\tag{40.5}
\]

with consistent derivatives/fluxes. This is one identified physical face, not two independent velocity boundaries with similar values. Outward fluxes have opposite signs because normals differ. Pressure coupling joins the last pressure cell to the first, and diffusion, reconstruction and advection stencils cross the same seam.

Pairwise cancellation in (40.2)–(40.4) follows because fields/tractions match, areas match and normals oppose. It is structural at continuum level; numerical conservation still depends on transport and quadrature. On a MAC grid, the duplicated end samples of the normal component must represent one degree of freedom or be synchronized from one owner. Counting both in inertia, energy or reductions double-counts the seam. Semi-Lagrangian interpolation can still lose scalar integral, kinetic energy or momentum on a perfectly periodic topology. A conservative transport milestone must be distinguished from a periodic-sampling milestone.

A rectangular box periodic in x with walls in y/z is a **recirculating tunnel**. Its wake returns after approximately \(L_x/U\) when a meaningful mean speed \(U\) exists. Wall type and cross-sectional area control confinement. Periodicity in all three directions models a repeated array on a torus, not an isolated propeller surrounded by infinite water. Pressure is elliptic: images and walls affect the near field before a visible wake has travelled to them. There is no purely advective waiting time that guarantees immunity to boundary effects.

### 40.3.2 Gravity and net momentum on a periodic domain

Integrate (40.1) over a fully periodic domain. Periodic transport, pressure and viscous surface terms cancel. For uniform density,

\[
\frac{d\langle\mathbf u\rangle}{dt}
=\mathbf g+\langle\mathbf f\rangle.
\tag{40.6}
\]

Pressure projection removes divergence-producing gradient components; it cannot remove a spatially uniform acceleration while retaining periodic pressure. Indeed \(\int_0^{L_x}\partial_xp\,dx=0\) for periodic \(p\), whereas hydrostatic balance along x requires \(\partial_xp=\rho g_x\), whose integral is \(\rho g_xL_x\). These are inconsistent unless \(g_x=0\) or a pressure jump is allowed. Horizontal periodicity plus vertical gravity and a supported vertical hydrostatic boundary is consistent. Full periodicity plus gravity can be defined as accelerating flow, but cannot be silently interpreted as a static ocean.

A useful alternative is a pressure-drop model \(p=\widetilde p-\mathbf G\cdot\mathbf x\), with periodic \(\widetilde p\). Then (40.6) gains \(\mathbf G/\rho\). This is an explicit pump/pressure-gradient forcing, not ordinary pressure-periodic boundaries. A maintained mean velocity instead requires a feedback force; its impulse and work must be reported. Subtracting mean velocity each step is an imposed momentum sink. Viscosity dissipates fluctuations but, in a fully periodic box, its integral cannot remove net momentum.

A propeller with nonzero mean thrust therefore accelerates the periodic fluid unless balanced by wall drag, an opposing body/hull force, imposed pressure gradient or declared momentum controller. A force applied to fluid alone represents an external actuator; a freely moving boat requires the equal-and-opposite rigid-body response and its own dynamics. A stationary mounted propeller can transfer reaction to its support. A torque-producing actuator also needs an angular-momentum account; a torus has no unique global position-based angular momentum, so local torque and mapped impulses are safer diagnostics.

### 40.3.3 Rotated or displaced portal patches

An orientation-preserving rigid map \(T(\mathbf x)=\mathbf R\mathbf x+\mathbf b\), \(\mathbf R^T\mathbf R=\mathbf I\), \(\det\mathbf R=1\), would require

\[
\mathbf n_B(T\mathbf x)=-\mathbf R\mathbf n_A(\mathbf x),\quad
\mathbf u_B(T\mathbf x)=\mathbf R\mathbf u_A(\mathbf x),\quad
c_B=c_A,\quad p_B=p_A.
\tag{40.7}
\]

Stresses and any tensor histories transform as \(\boldsymbol\sigma_B=\mathbf R\boldsymbol\sigma_A\mathbf R^T\). Area-preserving bijection, matching aperture geometry, reciprocal flux transfer and compatible fields are mandatory. A Cartesian signed-axis permutation is much simpler than arbitrary rotation, which generally needs nonmatching interpolation or conservative mortar-like coupling. Particle positions may map exactly while MAC face fluxes do not.

Rotation preserves speed and local kinetic energy, but changes world-space momentum by \(m(\mathbf R-\mathbf I)\mathbf u\) per particle; translation/rotation can change position-based angular momentum. This is a geometry/frame redirection or an external impulse, not global vector-momentum conservation in a single Euclidean room. Under a fixed gravity vector, force equivalence requires \(\mathbf R\mathbf g=\mathbf g\) and compatible gravitational potential. A height-changing connection needs an explicit pressure/potential jump or external work. Moving portals would add frame velocities and are outside the first scope.

At intersections, map composition must be unambiguous: two seam crossings must lead to the same identified location and field transformation. Translation maps commute; arbitrary rotations/translations need not. An ordinary seam cannot share a face degree of freedom with a wall. Partial periodic patches imply a different connected geometry and aperture-edge treatment, not a periodic texture mask. MFEM's periodic-mesh documentation and pinned vertex-identification implementation are useful evidence of the topology requirement; its rotational examples do not qualify arbitrary fluid portals in Rheon. [I2]

## 40.4 Weak conditions and the meaning of an open boundary

Assume the stationary boundary is partitioned into velocity-prescribed parts \(\Gamma_D\), open parts \(\Gamma_o\), and reciprocal periodic pairs. For viscosity \(\mu\ge0\), use vector test functions \(\mathbf v\) with zero trace on \(\Gamma_D\) and matching periodic traces; scalar pressure tests are \(q\). With sufficient regularity, multiply (40.1) by \(\mathbf v\), integrate stress divergence by parts, and use symmetry of stress. The weak momentum/continuity equations are

\[
(\rho\partial_t\mathbf u,\mathbf v)_\Omega
+(\rho(\mathbf u\cdot\nabla)\mathbf u,\mathbf v)_\Omega
+(2\mu\boldsymbol\varepsilon(\mathbf u),\boldsymbol\varepsilon(\mathbf v))_\Omega
-(p,\nabla\cdot\mathbf v)_\Omega
=(\rho\mathbf a,\mathbf v)_\Omega+\langle\mathbf t,\mathbf v\rangle_{\Gamma_o},
\]

\[
(q,\nabla\cdot\mathbf u)_\Omega=0,
\qquad \mathbf t=\boldsymbol\sigma\mathbf n.
\]

The notation \((\cdot,\cdot)_\Omega\) means volume integration; \(\langle\cdot,\cdot\rangle\) is boundary duality, ordinary area integration for smooth traces. Periodic traction contributions cancel because the test traces agree under the map and outward normals oppose. On prescribed-velocity boundaries, their reactions do not disappear physically merely because these test functions vanish there; moving/inlet boundary power must be retained when deriving an energy ledger by a lifting or directly from the strong balance.

**Conditional proof: energy identity.** For homogeneous stationary walls, admissible periodic traces, incompressibility and smooth fields, set \(\mathbf v=\mathbf u\). The pressure volume term vanishes. The advective term is \(\frac12\int_{\partial\Omega}\rho|\mathbf u|^2u_n\,dA\), by the divergence theorem. Symmetric stress gives the nonnegative volume dissipation \(\int 2\mu\boldsymbol\varepsilon:\boldsymbol\varepsilon\,dV\). Rearrangement gives (40.4). Nonhomogeneous inflow/outflow supplies power; the result is not an assertion that every open boundary dissipates energy.

A traction condition prescribes this weak boundary term. A pressure opening instead fixes a pressure trace with a compatible velocity/viscous treatment. Prescribing all normal velocities and arbitrary pressure independently overconstrains the same momentum/continuity problem. An incompressible characteristic construction cannot remove the elliptic pressure constraint. These distinctions persist in finite-volume and particle formulations even when their weak spaces are not implemented explicitly.

The symmetric Newtonian traction used here is not interchangeable with a Laplacian-based pseudo-traction without changing the viscous weak form and energy identity. The open-boundary papers [P2–P4] specify their conventions and splitting; their published stability results do not transfer by copying a boundary term into another discretization.


## 40.5 Hydrostatic reservoirs and lake-at-rest balance


### 40.5.1 Hydrostatic exterior, not force classification

Let \(\mathbf g=-\nabla\Phi\) for a stationary potential. For constant density, define

\[
p_h=C-\rho\Phi,\qquad \pi=p-p_h.
\tag{40.8}
\]

Because \(\nabla p_h=\rho\mathbf g\),

\[
-\frac1\rho\nabla p+\mathbf g=-\frac1\rho\nabla\pi.
\tag{40.9}
\]

For upward z and \(\mathbf g=-g\mathbf e_z\), an exterior lake at elevation \(\eta_{ext}\) has \(p_h(z)=p_{atm}+\rho g(\eta_{ext}-z)\) in the wetted region. All submerged open faces must see this same elevation-dependent pressure. Setting \(p=0\) everywhere on vertically separated or vertical submerged faces while applying gravity does **not** describe a stationary exterior lake. Setting **reduced** pressure \(\pi=0\) can, if (40.8)–(40.9) are applied consistently.

At rest, \(\mathbf u=0\), \(\pi\) constant is a solution. There is no need to test whether a force is “genuine”: gravity and pressure already cancel. A nonuniform conservative authored force can also be balanced by pressure; a boundary cannot infer user intent from a decomposition of forces. Suppressing gravity only near a portal changes the body-force field and commonly creates an artificial shear/pressure adjustment region.

A discrete well-balanced formulation requires, on the same face locations and distances used in projection,

\[
\mathbf g_f=\rho_f^{-1}(G p_h)_f.
\tag{40.10}
\]

If full pressure is used, a rest predictor \(\mathbf u^*=\Delta t\mathbf g\) must be corrected by exactly the same discrete gradient. If reduced pressure is used, the constant-density reference gravity term is removed algebraically from the predictor and \(p_h\) restored for physical pressure/traction inspection. Applying full gravity again would double-count it. At boundaries use the cell-center-to-face distance, not an interior full-cell spacing. Existing gates and stored-f32 divergence checks remain authoritative; residual cancellation is not proof of zero spurious current.

For variable density, subtracting a reference \(p_{h,0}\) leaves

\[
-\rho^{-1}\nabla p+\mathbf g
=-\rho^{-1}\nabla\pi+(1-\rho_0/\rho)\mathbf g
\tag{40.11}
\]

when \(\nabla p_{h,0}=\rho_0\mathbf g\). Buoyancy remains. A general density field cannot be removed by constant-reference pressure subtraction; equilibrium stratification, interface jumps and compatible density interpolation need separate analysis. Reduced pressure is also not permission to subtract pressure from an EOS-based SPH model without rederiving its forces and density equilibrium.

### 40.5.2 All-face openness and the propeller

A fully submerged computational box can exchange fluid through all six faces while using a common hydrostatic exterior. The bottom then represents deeper surrounding water, **not** a seabed. The top is an artificial submerged truncation, **not** automatically a free surface. At rest all six fluxes can vanish. Under propeller forcing, incoming and outgoing regions can coexist on each face; a fixed “left=inlet, right=outlet” rule can incorrectly suppress entrainment and transverse flow.

In an exterior-reservoir model, pressure and momentum fluxes in (40.3) can balance thrust without recirculating the wake. This requires neither exact left/right flux pairing nor copying the outgoing material upstream. Choose exterior velocity/current, pressure datum/water level, density, temperature and material fractions together. Matching all pressure values at rest does not imply a zero exterior dynamic traction under an imposed ambient current. A prescribed current is supplied momentum and energy, not an absence of forcing. Exterior current is a far-field, incoming-characteristic or traction-matching datum for the chosen model; it must not independently hard-prescribe the same normal velocity that a pressure opening is supposed to determine.

An open tunnel still has side-wall blockage. An all-open finite box still omits the external wake, return flow and pressure field. Domain-length, width, depth and boundary-model sensitivity are required before claiming an isolated propeller approximation. No universal safe number of propeller diameters is established here.

## 40.6 Discrete MAC pressure, flux compatibility, and gauges


These are original finite-volume derivations for a constant-density orthogonal MAC extension, not a specification already implemented in Rheon. Use each interior face once with oriented cell-face incidence \(C\), where \(Cq\) is integrated **outward** flux per cell. Let \(S\) contain positive face areas and \(L\) positive sample distances. With \(C\) positive on the lower-coordinate donor cell and negative on its neighbor,

\[
q=S u,\quad Gp=-L^{-1}C^Tp,\quad
W=S\rho^{-1}L^{-1},\quad A=CWC^T.
\tag{40.12}
\]

Pressure correction is \(q^{n+1}=q^*+\Delta t\,WC^Tp\). Known outward boundary fluxes form a cell vector \(Q\). Enforcing \(Cq^{n+1}+Q=0\) gives

\[
Ap=-\frac{Cq^*+Q}{\Delta t}.
\tag{40.13}
\]

Here \(q\) is m³/s, \(Q\) is m³/s, pressure is Pa and \(Ap\) is m³/s². The same \(S,L,\rho\) must govern operator assembly, correction and diagnostics. Summation by parts gives the matching pressure work plus boundary terms. The standard MAC projection ingredients and distinction between wall and air pressure conditions are described in Bridson and Müller-Fischer's course notes, Chapter 4; their code examples are not copied. [P1]

**Periodic/Neumann components.** Every periodic seam is another incidence column with two opposite entries. Then \(A\mathbf1=0\) and \(\mathbf1^Tb=0\) is necessary on each connected component without a pressure anchor. In the filled fixed box this is the net prescribed-volume-flux constraint. Gauge fixing removes nonuniqueness, not incompatible physical flux. Do not silently shift the RHS mean to hide a leak or inconsistent inlet request. Isolated fluid components behind obstacles each need their own compatibility check and pressure gauge; a single pin is insufficient in that extension. Very small periodic counts need explicit topology tests: a seam can join a cell to itself or create parallel edges.

**Pressure reservoirs.** For a boundary face at distance \(d_b\) from cell i, \(w_b=A_b/(\rho d_b)\). With physical or reduced boundary pressure \(p_b\) in the same convention as cell pressure,

\[
q_b^{n+1}=q_b^*+\Delta t\,w_b(p_i-p_b).
\tag{40.14}
\]

To make that extension explicit, let \(\mathbf e_i\) be cell i's coordinate vector, \(Q_{pres}\) contain only final prescribed wall/inlet fluxes, and \(Q_{open}^*=\sum_b\mathbf e_{i(b)}q_b^*\) contain the provisional reservoir-face fluxes. The full pressure system is

\[
\widehat A=A+\sum_b w_b\mathbf e_{i(b)}\mathbf e_{i(b)}^T,
\]

\[
\widehat A p=-\frac{Cq^*+Q_{pres}+Q_{open}^*}{\Delta t}
+\sum_b\mathbf e_{i(b)}w_b p_b.
\]

Every reservoir provisional flux appears once; it is not also a prescribed final flux in \(Q_{pres}\). Thus add \(w_b\) to diagonal \(A_{ii}\) and \(w_b p_b\) to that row's boundary-data contribution. For a regular boundary \(d_b=h/2\); the coefficient is twice the interior weight. Such a Dirichlet contact removes the constant nullspace on its connected component. There is no additional arbitrary gauge pin there. Net flux is obtained by the pressure solve rather than prescribing every normal velocity independently.

**Velocity boundaries.** From \(u_n^{n+1}=u_n^*-(\Delta t/\rho)\partial_np\), a specified final normal velocity \(v_b\) gives \(\partial_np=(\rho/\Delta t)(u_n^*-v_b)\) for this nonincremental projection. An incremental pressure-correction variable has its own condition; it is not automatically physical pressure. Do not impose arbitrary normal speed and arbitrary pressure simultaneously on the same face. Tangential viscous conditions, scalar inflow data and pressure anchoring are separate choices.

**Mixed/Robin conditions.** Traction and energy-stable conditions can make pressure data depend on viscous terms, backflow or boundary inertia. They require a derivation consistent with the chosen velocity/pressure splitting. A pressure-Dirichlet shortcut is not the complete traction condition for a viscous fluid. Symmetry/positive definiteness, boundary quadrature, solver support and preconditioner dependencies must be checked anew. Periodic cycles can invalidate traversal assumptions in an SGS triangular factorization. Manufactured boundary solutions and full stored-field divergence are necessary in addition to residual checks. Dong and Shen give a pressure-correction treatment of open boundaries; their spectral-element results are evidence for that formulation, not an established stability theorem for Rheon's MAC discretization. [P2]

### 40.6.1 Conditional finite-dimensional proofs

For each internal or translation-periodic face \(f=(i,j)\), take a positive weight \(w_f\), and let \(B\) be the set of pressure-reservoir faces. The matrix just derived satisfies

\[
p^T\widehat A p=
\sum_{f=(i,j)}w_f(p_i-p_j)^2+\sum_{b\in B}w_b p_{i(b)}^2\ge0.
\]

**Nullspace proof.** A vector has zero quadratic form precisely when pressures agree across every positive-weight edge and are zero at every anchored cell. Thus each unanchored connected component retains one constant-pressure mode; an anchored connected component has none. A gauge removes the unanchored constant freedom, while compatibility is a separate condition on the RHS. This assumes accurate connectivity and strictly positive coefficients; it is not a theorem about floating-point CG or a changing wet/solid topology.

**Pressure-work proof.** Consider unknown face velocities, each represented once, with mass metric \(M_f=\rho S_fL_f\), and write \(\overline C\) for incidence including pressure-opening faces in outward orientation. On an internal face \(L_f\) is the pressure-sample distance; on an opening it is the cell-to-boundary distance. Let \(\delta u=u^+-u^*\), and let \(p_{ext}\) be nonzero only on pressure-opening columns. The exact pressure correction is

\[
M\delta u=\Delta t\,S(\overline C^Tp-p_{ext}),
\qquad \overline C S u^++Q_{pres}=0.
\]

Using \(K(u)=\frac12u^TMu\) and
\(K(u^+)-K(u^*)=(u^+)^TM\delta u-\frac12\delta u^TM\delta u\) yields

\[
K(u^+)-K(u^*)=
-\Delta t\,p^TQ_{pres}
-\Delta t\sum_b p_b q_b^+
-\tfrac12\delta u^TM\delta u.
\]

For sealed/periodic homogeneous flux, both work terms vanish and this projection alone cannot increase the metric kinetic energy. Reservoir pressure deliberately supplies/removes work. For a nonzero divergence defect \(r=\overline C S u^++Q_{pres}\) and correction defect \(\eta=M\delta u-\Delta tS(\overline C^Tp-p_{ext})\), the right side gains \(\Delta t\,p^Tr+(u^+)^T\eta\). These terms explain why the actual stored velocity and correction must be checked after an inexact solve/cast. They are not authorization to loosen an acceptance threshold.

The proof assumes one shared pressure gradient/divergence pair, matching mass/area/distance, fixed geometry and exact arithmetic until the explicit defects are introduced. It covers a pressure substep, not velocity advection, forcing, viscosity splitting, backflow stabilization or free-surface reconstruction. A duplicated periodic face would invalidate its mass accounting. A reservoir normal face has half-cell dual mass on a regular box, whereas an identified periodic seam has one full dual cell.


## 40.7 Open-boundary methods and energy control


| Model | What it specifies | Value and limitation for Rheon |
|---|---|---|
| Prescribed balanced inflow/outflow velocity | Normal velocity/flux; compatible pressure correction; exterior scalars for inward flow | Closest to current `PrescribedBoxFlux`. Suitable for a controlled channel. Flow does not choose the boundary speed; not a passive ocean reservoir. |
| Static pressure / pressure reservoir | Pressure or reduced pressure; outgoing normal velocity from projection; compatible tangential condition and incoming state | Smallest useful pressure-opening extension. Hydrostatic rest is possible. Raw pressure with copied velocity has no general backflow-energy guarantee. |
| Traction / “do-nothing” | Total stress traction, often relative to exterior pressure | Natural for weak formulations and outgoing flow. Pressure and viscous stress are coupled. Reverse flow can import uncontrolled kinetic energy. |
| Convective / radiation | \(\partial_t\psi+U_c\partial_n\psi=0\), \(U_c>0\), for a chosen outgoing disturbance | Useful approximation for aligned, advected disturbances. One speed cannot absorb every vortex/wave/angle; reversing flow needs incoming data. Cannot replace global incompressible projection. |
| Characteristic | Incoming characteristic data supplied from exterior; outgoing characteristics from interior | Appropriate wave bookkeeping for WCSPH/compressible or shallow-water systems. Use the actual model's characteristics. Incompressible pressure is elliptic, not a finite-speed acoustic characteristic. |
| Energy-stable open boundary | Traction modified using the sign/size of backflow; some schemes include boundary inertia | Strong candidate for robust bidirectional outlets. A continuum energy estimate must be carried through discretization and splitting. Stability alone does not ensure a minimally distorted wake. |
| Sponge / relaxation layer | Explicit forcing/diffusion toward an exterior/reference state over a buffer | Can reduce reflected disturbances; costs cells and intentionally changes momentum/energy. Width, strength and overlap at corners need sensitivity studies and ledgers. |

### 40.7.1 Why backflow matters

The boundary contribution in (40.4) is

\[
B=\mathbf u\cdot\mathbf t-\tfrac12\rho|\mathbf u|^2u_n,
\qquad \mathbf t=\boldsymbol\sigma\mathbf n.
\tag{40.15}
\]

For a zero-traction outlet, \(B=-\rho|\mathbf u|^2u_n/2\): it removes energy for outgoing flow but imports energy when \(u_n<0\). A vortex crossing a face naturally has both signs; classifying an entire face as “outflow” is inadequate. A negative-normal-velocity clamp prevents this influx by introducing a one-way constraint, but can reflect vortices and prevent real entrainment. It is a valve approximation, not generic openness.

For an illustrative *continuum* rest-reservoir model, define \(u_n^- = \min(u_n,0)\) and take

\[
\mathbf t=-p_h\mathbf n+\tfrac12\rho u_n^-\mathbf u.
\tag{40.16}
\]

Substitution into (40.15), without a discretization assumption, gives

\[
B=-p_hu_n-\tfrac12\rho\max(u_n,0)|\mathbf u|^2.
\tag{40.17}
\]

This original calculation illustrates the negative-part stabilization family discussed by Dong and Shen; it is **not** a completed Rheon boundary algorithm. For gravity \(\mathbf g=-\nabla\Phi\), potential energy \(U=\int\rho\Phi\,dV\) obeys

\[
\frac{dU}{dt}=-\int_{\partial\Omega}\rho\Phi u_n\,dA
-\int_\Omega\rho\mathbf g\cdot\mathbf u\,dV.
\tag{40.18}
\]

Combining (40.4), (40.17) and (40.18), hydrostatic \(p_h+\rho\Phi=C\) contributes \(-C\int u_n\,dA=0\) for the compatible filled domain. The remaining open-boundary term is nonpositive. This explains how hydrostatic support and energy control can coexist without filtering gravity. A driven current, incoming turbulence, moving boundary or imposed reservoir pressure difference supplies additional power and needs its own estimate. Smoothing a sign switch, lagging it or applying it after projection does not inherit this exact bound automatically. For generic inviscid backflow, this full-vector traction may additionally restrict tangential velocity because inviscid stress has no tangential traction. Setting viscosity to zero therefore does not establish a general Euler boundary; a normal-only stabilizer or characteristic alternative needs its own analysis. Physical viscous traction here uses symmetric stress; some cited analyses use a Laplacian/pseudo-traction convention. Their boundary formulas must not be interchanged without the accompanying energy identity. [P2]

Dong's 2015 formulation adds boundary inertia to an energy-stable open condition and couples it to Robin pressure and velocity substeps. Its near-boundary convection scale is useful when vortices exit, but this is a larger change than copying an outlet velocity. Ni, Yang and Dong later show that dissipation choices can produce stable yet inaccurate flow. Rheon should compare wake distortion, reflection and budgets, not select the condition merely because it remains finite. [P3, P4]

### 40.7.2 Radiation, characteristics, and reflection

An outgoing linear disturbance \(\psi=f(s-ct)\) satisfies \(\partial_t\psi+c\partial_s\psi=0\). A boundary using \(U_c\ne c\) leaves a mismatch proportional to \((U_c-c)\partial_s\psi\); oblique waves also have different normal propagation. Convective boundaries are therefore approximations to an exterior impedance/transport problem. Orlanski's 1976 paper is the historical primary reference for estimating outgoing propagation; this dossier verified the institutional PDF endpoint but did not extract its scanned body, and does not claim an exact reproduction of its discrete scheme. [P5]

For a linearized barotropic WCSPH fluid about rest, in one normal coordinate, the acoustic subsystem is

\[
\partial_t u_n'+\rho_0^{-1}\partial_n p'=0,\qquad
\partial_t p'+\rho_0c_s^2\partial_n u_n'=0.
\]

Adding/subtracting these equations yields invariants \(J_\pm=u_n'\pm p'/(\rho_0c_s)\) travelling at \(\pm c_s\). With mean normal speed \(U_n\), characteristic speeds shift to \(U_n\pm c_s\). An incoming invariant comes from the exterior; specifying both pressure and velocity independently can overconstrain the same acoustic subsystem. These relations are a linear illustrative derivation, not a general nonlinear/multiphase SPH boundary specification. Rheon's reduced liquid sound speed changes acoustic impedance and reflection; imported boundary parameters must use that actual model, not water's physical acoustic speed.

For a free-surface long wave of depth H, speeds are \(U_n\pm\sqrt{gH}\), not \(U_n\pm c_s\). A shallow-water radiation boundary should not be applied unchanged to a fully submerged 3D incompressible solver. A useful verification diagnostic for rest-state 1D acoustics is the ratio of inward to outward invariant amplitudes, \(\mathcal R=\|J_-\|/\|J_+\|\), measured before the packet is contaminated by other reflections; free-surface wave gauges need the corresponding elevation/velocity decomposition. A claimed low-reflection result must state frequency, angle, amplitude, depth, resolution and measurement window.

For a sponge, a transparent accounting model is \(\mathbf f_s=-\chi(\mathbf x)(\mathbf u-\mathbf u_{ext})\), with \(\chi\ge0\) in s⁻¹. It damps perturbation energy relative to a stationary uniform reference, but total work \(\int\rho\mathbf u\cdot\mathbf f_s\,dV\) need not be negative for nonzero exterior current. Blending forces or their overlap at corners must be defined once; abrupt ramps can themselves reflect waves. Relaxing the total hydrostatic pressure or water level to the wrong target creates artificial currents. MITgcm's official OBCS documentation provides an operational example of separate radiation, sponge, inflow tracer and transport-balancing controls, including implementation limitations. It is a method reference here, not a copied or audited implementation dependency. [N1]

## 40.8 Faces, patches, material, and particle support


A future domain description should partition every outer face area into nonoverlapping patches. Each patch has exactly one kinematic/pressure category: wall, prescribed flux, pressure reservoir, traction/radiation opening, or member of a reciprocal periodic pair. Tangential stress/slip and incoming material state are additional declared attributes, not competing normal constraints. Opposite-face periodic selection must be reciprocal and matching; a wall can coexist on different faces and, later, on disjoint portions of one face.

For the first version, whole faces on a fixed rectangular grid remove ambiguity. Unselected faces remain explicitly specified walls. An all-open preset means six reservoir/open faces using a common exterior; an all-periodic preset means three paired axes and very different physics. Open patches may meet walls at edges and produce stress/pressure singularities; a coarse grid will smear them. Do not claim grid-independent corner accuracy from a converged pressure residual.

Pressure rows accumulate contributions from each incident face, with no “last boundary wins” override. Ghost intersections for advection, diffusion and interpolation need a deterministic extension compatible with all incident conditions. At a periodic-wall intersection, periodic wrapping preserves the wall classification; at two periodic axes, diagonal neighbor access must wrap both. Reservoir corners sample a single coherent exterior state. Conflicting prescribed states should be rejected or modeled with an explicit transition, not averaged invisibly. At exact edge/corner crossings, particles need a unique event and ownership rule so they are neither spawned/deleted twice nor missed. Multi-face leaps need bounded time/trajectory treatment; endpoint clamping is insufficient.

For a conservative volume/scalar update, compute each interior or periodic transfer once and apply opposite signed amounts to its two recipient cells. At open boundaries use the projected signed volume flux, and take advected properties from the upstream side. All material fractions on inflow must be admissible and collectively consistent; mixed densities change mass and momentum flux. Temperature transport needs heat capacity to become a thermal-energy ledger. Viscosity/diffusion also requires scalar boundary data if retained.

A periodic transfer returns the same material, heat and dye; a fresh reservoir replenishes the exterior state. Recycling wake velocity while resetting dye, temperature or turbulence is a hybrid forced model. It can be useful artistically but should disclose exactly which quantities are erased and count their source/sink budgets. Matching inflow and outflow **volume** alone does not match momentum, energy or material mass.


### 40.8.1 Periodic SPH and particle transfer

For translation periodicity, particle positions wrap modulo the periodic length while mass, velocity, temperature and material identity are retained. Neighbor search and force/density sums must include the periodic image interactions *before* a particle center crosses the seam. Otherwise density and pressure collapse near a boundary even if trajectories look continuous. Minimum-image distances require a support/domain-size restriction or explicit multiple images when support reaches farther; corner images and self-image/double-count exclusions need specification. Pair interactions should be evaluated with equal-and-opposite force accounting. Particle identity stays stable when wrapping; images are computational support, not extra physical mass.

For rotated portals, vectors and tensor histories need the transforms in (40.7), and the ledger must count the momentum redirection. For FLIP/PIC/APIC, a future hybrid must also map particle-grid transfer support, affine velocity state and grid topology consistently; wrapping particles without the carrier grid would not be periodic fluid. Rheon's present material preview is SPH, not FLIP/PIC.

### 40.8.2 Open SPH is more than deleting particles

Near an opening, density summation and pressure/viscous forces still need kernel support. Removing neighbors immediately can create an artificial free surface or pressure deficit. A buffer/reservoir zone should supply sufficient support for the actual kernel radius, with a separately defined state update. Ghost/mirror positions can reconstruct fields, but this mathematical mirroring is not a physical test that admits only non-gravitational force. Buffer membership must not be counted twice in physical mass.

Outgoing particles should cross a defined control surface, remain available as required for kernel support, and retire once their support no longer affects the interior. Incoming particles require prescribed exterior velocity, material, temperature and a pressure/density state compatible with the EOS and hydrostatic column. Accumulate volume demand from boundary flux to avoid rate quantization bias; insertion spacing and interaction support matter. Bidirectional boundaries must support reversal without overlapping emit/retire regions or resetting returning particles spuriously. Capacity rejection must remain transactional, not delete accepted fluid to make room.

The current `(run_id,index)` identity is append-only. Recycling a storage slot for a new incoming particle would alias recorded histories unless a generation/stable particle ID is introduced. Every birth/exit should record mass, represented volume, momentum and available energy terms. Pressure/density diagnostics are currently pre-integration while accepted positions are later; a boundary ledger must retain these distinct times.

Negi, Ramachandran and Haftu compare frozen-particle, mirror, characteristic and hybrid outlets, including pressure pulses and advecting vortices; their pinned PySPH hybrid module explicitly cites the study. The paper's EDAC/WCSPH scheme differs from Rheon's preview EOS/density algorithm, so it supplies test ideas and design evidence rather than a drop-in guarantee. No implementation text is copied. [P6, I3]

### 40.8.3 Fully submerged fluid versus liquid-air flow

A filled incompressible carrier has fixed total liquid volume. A free surface has a kinematic material interface and a dynamic stress condition; neglecting air and surface tension reduces the latter to atmospheric pressure, but does not prescribe zero velocity. If waves cross an open face, boundary data must carry interface position/fraction, wet/dry classification, pressure and momentum together. External water level must align with the hydrostatic reference. Simply filling every inflow voxel with water can flood a domain that includes air.

Variable-density two-phase flow also needs phase-consistent mass and momentum flux and interfacial energy/stress. Dong and Wang's primary two-phase open-boundary study treats this explicitly in a phase-field model. It demonstrates that interface crossings need extra conditions; it does not establish compatibility with Rheon's donor-cell/slab/column representations. Breaking waves, spray, entrained air and capillary exit events remain outside a constant-density first version. [P7]

## 40.9 Physical controls and artistic approximations


| Control | Meaning and constraint |
|---|---|
| Boundary mode per face, later per patch | Periodic identification, wall, controlled flux or exterior reservoir. Preserve mode-specific limits and reject overlaps. |
| Exterior water level / pressure datum | Sets hydrostatic \(p_h\) and pressure work. Absolute pressure matters for gas EOS/cavitation models; a constant pressure gauge is otherwise arbitrary in a closed incompressible component. |
| Exterior current / inflow material and temperature | Defines incoming state and supplied momentum/material/heat. Does not dictate outgoing wake state. |
| Pressure difference / pump / mean-flow controller | A real drive or an explicit constrained model, with work and impulse. A periodic pressure drop is separate from periodic pressure. |
| Wall slip / roughness / drag | Changes tunnel resistance and momentum transfer. Keep existing wall-model scope explicit. |
| Radiation speed / sponge width and timescale / backflow stabilization | Numerical/exterior-model parameters. Expose units and diagnostics; choose through validation and sensitivity, not by claiming a perfect absorber. |
| Gravity and reference frame | Physical acceleration plus the matching hydrostatic reference; arbitrary gravity filtering at seams is not a physical control. |
| Wake reset, force gating, momentum retention slider, automatic mean subtraction | Artistic source/sink operations. If offered, label separately and report removed/added material, momentum and energy. Do not call them passive open water. |
| All faces open versus all axes periodic | Different domain models, not two strength settings for the same effect. Free-surface inclusion is an independent physical choice. |

## 40.10 What has been proved, and under which assumptions?

| Statement | Mathematical support in this chapter | What remains unproved for Rheon |
| --- | --- | --- |
| Continuum volume/material/momentum/energy budgets | Integrate (40.1), transport and stress identities for a stationary domain and the stated regularity/density assumptions. | Numerical assembly, advection, storage rounding and complete timestep budgets. |
| Periodic cancellation and mean acceleration | Opposed paired normals and matching traces cancel surface integrals; constant-density mean momentum gives (40.6). | Correct seam topology, scalar/momentum transfer, duplicate-face removal and supported forces in actual code. |
| Hydrostatic balance and reduced pressure | Algebraic cancellation (40.8)–(40.11); discrete rest requires the same pressure gradient and gravity samples. | Lake-at-rest behavior for actual f32 fields, all boundary configurations and SPH EOS equilibrium. |
| Pressure nullspace and correction work | Positive weighted-graph quadratic form and mass-metric identities above. | A new operator's SPD/gauge implementation, SGS ordering, PCG convergence and finite-precision diagnostics. |
| Negative-part backflow energy control | Continuum substitution (40.16)–(40.18), compatible total flux and a rest hydrostatic exterior. | A stable, consistent discrete boundary with Rheon's actual advection/force/pressure/viscous split. |
| Arbitrarily rotated portal | Necessary transformation, reciprocity and area conditions only. | Conservative nonmatching interpolation, global closure, corner consistency and external impulse/work. |

These are hand-derived conditional mathematical results, not new machine-checked statements. Existing [formal-contract guidance](16-formal-contracts.md) and [pressure-gauge research](../engineering/pressure-compatibility-and-gauges.md) explain narrower source-bound algebra. No existing theorem count is enlarged, no Lean receipt is refreshed, and no historical numerical result qualifies a portal. Independent manuscript review, strict math rendering and link checks concern the exposition; future native qualification is separate.


## 40.11 Verification before implementation claims


The cases below are proposed experiments and analysis obligations. No row is an observed Rheon pass. Before execution, fix domain, units, discrete equations, output quantities, cost bounds and acceptance criteria through the repository's normal authorization/review process. Preserve existing pressure and divergence thresholds. Use analytic balances, refinement trends and independently assembled tiny operators; avoid tests that merely call the same boundary routine twice.

| Case | Setup and independent expected behavior | Measurements / failure signatures |
|---|---|---|
| **Lake at rest under gravity** | Filled box with selected side/bottom/top submerged reservoirs using one \(p_h\); compare full and reduced pressure formulations. Also use a horizontal periodic axis with vertical walls/free surface in supported scope. Analytic \(\mathbf u=0\), \(\nabla p_h=\rho\mathbf g\). | Spurious max/RMS speed, stored divergence, net and per-face volume flux, pressure-gradient mismatch and work. Vary depth, origin and aspect ratio; rest should not depend on arbitrary pressure datum. Include inconsistent raw-zero-pressure data as an explicit negative case. |
| **Fully periodic gravity counterexample** | Periodic constant-density uniform fluid and uniform gravity; analytic mean acceleration (40.6). A separate declared pressure-gradient cancellation gives zero mean acceleration. | Mean momentum and work versus time. Reject a “static ocean” configuration or document acceleration; never treat a gauge pin as hydrostatic support. |
| **Periodic scalar and momentum transport** | Uniform translation with smooth dye/material packet crossing the seam, then multiple axes/diagonal crossings. Exact periodic translated field and constant global volume/momentum in the unforced conservative model. | Material integral, center phase, extrema, seam continuity, momentum and kinetic energy defect. Compare topology/transport effects; current semi-Lagrangian method may dissipate, which must remain visible. |
| **Tiny pressure matrices and nullspaces** | Independently assemble one-/two-/three-cell periodic graphs, Neumann component, one pressure anchor, disconnected components and parallel/self seam edges. | Symmetry, row sums, compatible RHS, nullity, energy quadratic form, corrected flux. Test half-cell reservoir distance and no extra pin. Deliberately incompatible prescribed flux must fail without state mutation. |
| **Pressure-driven / forced channel** | Laminar planar channel between no-slip walls, periodic streamwise pressure drop or reservoir pressure difference. For height H, \(u_x(y)=F y(H-y)/(2\mu)\), \(Q/W=F H^3/(12\mu)\) with \(F=-dp/dx+\rho f_x\). Free-slip requires a different analytic case. | Profile, flux, pressure drop, wall shear, actuator work versus viscous loss. Separate entrance effects for finite reservoir channels. Never compare no-slip analytic drag against a free-slip implementation. |
| **Exiting vortex and pulse** | Advect a weak vortex through downstream opening; for SPH add linear acoustic pulse/ramp; for surface model add a low-amplitude wave packet. Compare with a larger-domain reference in the retained interior. | Reflected invariant/wave amplitude, circulation/wake shape, near-body pressure, energy/volume defect and sensitivity to outlet speed/sponge. Multiple incidence angles and wavelengths; “no visible return” is insufficient. |
| **Backflow and flow reversal** | A vortex straddles an outlet, or reverse a prescribed exterior flow. Adjacent faces can simultaneously contain inflow and outflow; begin with bounded low-amplitude cases. | No uncontrolled boundary-energy injection; resolved work from incoming state, stabilization, pressure and advection. Scalar inflow switches to reservoir state locally, no negative material fraction or one-way clamping. |
| **Propeller wake / domain-size sensitivity** | Fixed localized actuator with specified thrust/torque history. Compare periodic tunnel and open reservoir; vary upstream/downstream and transverse dimensions independently at fixed near-actuator resolution, then refine resolution separately. | Thrust/work, mean flow, swirl, axial wake profiles and decay, return-flow fraction, near-actuator pressure, budgets, and differences within a fixed region of interest. Steady statistics only after authorized duration/sampling design. No universal domain-size rule assumed. |
| **Mass/material conservation and reservoirs** | Uniform-material throughflow, dye slug, unequal density/material inflow where supported, and a reservoir that should remain stationary. Particle cases vary flux relative to one-particle mass. | Equation (40.2) including sources/exits and phase volumes; no mass from ghost images, birth-count quantization drift or recycling-ID aliasing. Count interior inventory and buffer inventory separately. |
| **Mixed boundaries and corners** | Periodic x/wall y-z; open x/wall y-z; all-open hydrostatic box; later disjoint wall/open patches. Particle paths cross seams/edges/corners and potentially more than one face per interval. | One boundary owner per face, reciprocal topology, unambiguous ghost fills, compatible pressure, per-patch flux/work, no duplicate transfers and correct cancellation/rollback on invalid configurations. |
| **Free-surface extension** | Supported still-water elevation followed by a small wave leaving a wetted opening into same-level exterior. Dry part admits air rather than full water. | Water-volume balance, surface height/phase, spurious current, pressure jump and reflected wave. Defer breaking/spray/capillarity claims until those models exist. |

For each numerical case, separate spatial, temporal and domain-truncation error. Record actual accepted dt, stored-field divergence, full residual, solver identity, boundary snapshot, source commit, allocations and rejected candidates. Parameter comparisons must not obtain apparent improvement by loosening existing gates, increasing unreported damping or moving the region of interest.

## 40.12 Primary sources and implementation-license boundary

Each entry states what was actually used. Source summaries are deliberately brief; the report's independent derivations are not reproductions of any source's algorithms.

**P1.** Robert Bridson and Matthias Müller-Fischer, *Fluid Simulation*, SIGGRAPH 2007 course notes, especially Chapters 1, 4 and 6. [Author-hosted PDF](https://www.cs.ubc.ca/~rbridson/fluidsimulation/fluids_notes.pdf). Consulted the MAC gradient/divergence/projection and wall/free-surface distinction. No code or illustrations reused.

**P2.** Suchuan Dong and Jie Shen, *A Pressure Correction Scheme for Generalized Form of Energy-Stable Open Boundary Conditions for Incompressible Flows*, JCP 291 (2015), 254–278; [author manuscript, arXiv:1409.6676](https://arxiv.org/html/1409.6676). Consulted §2.1 energy-boundary family and §2.2 splitting. The paper uses its specified normalized stress/gradient convention; Rheon's discrete counterpart remains to be derived.

**P3.** Suchuan Dong, *A Convective-like Energy-Stable Open Boundary Condition for Simulations of Incompressible Flows*, JCP 302 (2015), 300–328; [author manuscript, arXiv:1506.01320](https://arxiv.org/html/1506.01320). Consulted §2, particularly boundary inertia and Robin substeps. Published test outcomes belong to that paper, not this task.

**P4.** Naxian Ni, Zhiguo Yang and Suchuan Dong, *Energy-Stable Boundary Conditions Based on a Quadratic Form: Applications to Outflow/Open-Boundary Problems in Incompressible Flows*, [author manuscript, arXiv:1807.07056](https://arxiv.org/html/1807.07056). Consulted introduction and §2 on energy-dissipation scale and stability versus physical accuracy; no blanket “stable means accurate” inference.

**P5.** Isidoro Orlanski, *A Simple Boundary Condition for Unbounded Hyperbolic Flows*, JCP 21 (1976), 251–269, DOI [10.1016/0021-9991(76)90023-1](https://doi.org/10.1016/0021-9991(76)90023-1). [GFDL institutional PDF](https://www.gfdl.noaa.gov/bibliography/related_files/io7601.pdf) resolves to a 19-page scanned PDF. Full body text was not extracted here; retained as historical primary follow-up, not the basis of a claimed exact discrete formula.

**P6.** Pawan Negi, Prabhu Ramachandran and Asmelash Haftu, *An Improved Non-reflecting Outlet Boundary Condition for Weakly-compressible SPH*, CMAME (2020), article 113119, DOI [10.1016/j.cma.2020.113119](https://doi.org/10.1016/j.cma.2020.113119); [author manuscript v2](https://arxiv.org/html/1907.04034v2). Consulted §§3–4, outlet families and simple pulse/vortex tests. Its EDAC formulation and reported efficiencies are not Rheon qualifications.

**P7.** Suchuan Dong and X. Wang, *A Rotational Pressure-Correction Scheme for Incompressible Two-Phase Flows with Open Boundaries*, [author manuscript, arXiv:1412.8735](https://arxiv.org/html/1412.8735), especially §2. Consulted for extra interface/stress conditions at two-phase openings; phase-field scheme not adopted.

**N1.** MITgcm development team, [OBCS: Open boundary conditions for regional modeling](https://mitgcm.readthedocs.io/en/latest/phys_pkgs/obcs.html), §§8.3.1.4–8.3.1.5, and [Discretization and Algorithm](https://mitgcm.readthedocs.io/en/latest/algorithm/algorithm.html). Official numerical-method documentation consulted for pressure/free-surface distinctions, radiation, transport compatibility and sponge/control limitations. These `latest` pages are mutable; no implementation code or dependency is licensed into Rheon by this citation.


The [public source/scope audit](../implementation/fluid-portals-sources-and-scope.md) distinguishes papers used for mathematical evidence from exact implementation files inspected for possible reuse. No third-party code, pseudocode, figures or protected paper prose is incorporated. Scientific citation and independent derivation are distinct from copied or translated implementation. Excluded or uncertain code candidates are not recommended for copying; their licenses do not prohibit citing the associated mathematics.
