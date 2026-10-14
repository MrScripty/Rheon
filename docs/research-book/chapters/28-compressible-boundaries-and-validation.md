# 28 Compressible boundaries and a verification ladder

Boundaries complete the conservative balance; they are not a display mask added after the fluid step. A production compressible branch would need compatible geometry, characteristic flow boundaries, wall stress and heat, coherent accepted states, and quantitative verification. The program below is a proposed qualification sequence. An executed Euler teaching fixture qualifies only its declared cells, flux, EOS and boundary model.

## Prescribe information on incoming characteristics

Let \(\mathbf n\) point out of the fluid domain and \(a=(\mathbf u-\mathbf v_b)\cdot\mathbf n\), where \(\mathbf v_b\) is the velocity of the boundary/control surface. The inviscid characteristic speeds relative to that surface are \(a-c,a,a,a,a+c\). A negative speed carries information into the domain. For a nonsonic ideal-gas Euler boundary in three dimensions the count is:

| Local regime | Incoming fields | Physical data to supply |
| --- | ---: | --- |
| Supersonic inflow, \(a<-c\) | 5 | A complete compatible state |
| Subsonic inflow, \(-c<a<0\) | 4 | Four independent data, with one acoustic field obtained from the interior |
| Subsonic outflow, \(0<a<c\) | 1 | One incoming acoustic datum, often associated with target pressure |
| Supersonic outflow, \(a>c\) | 0 | Interior extrapolation, with no independent downstream state |

This is a local inviscid count. Sonic points, stagnation, corners, recirculation and mixed regimes need explicit handling. Total pressure, total temperature and flow direction are one common subsonic-inlet parameterization; they must combine with the outgoing interior information and EOS to produce a valid state. Prescribing density, all velocity components and pressure independently at a subsonic inlet generally imposes too much information. A changing outflow can develop inflow, requiring an appropriate exterior entropy and tangential state rather than continuing an outflow-only rule.

NASA's Wind-US documentation uses characteristic theory for freestream boundaries and identifies one physical condition at a subsonic outflow. It also reports reflection from a constant-pressure outlet. A boundary described as “nonreflecting” therefore needs a measured reflection error for the incidence angles, frequencies and mean flow actually claimed. [NASA flow-boundary documentation](https://www.grc.nasa.gov/www/winddocs/user/bc.html)

Viscous Navier–Stokes additionally requires compatible velocity/stress and temperature/heat conditions. The hyperbolic table alone does not specify those parabolic data. An impermeable stationary Euler wall sets \(\mathbf u\cdot\mathbf n=0\) and allows slip. A moving Euler wall sets \((\mathbf u-\mathbf v_w)\cdot\mathbf n=0\). A no-slip Newtonian wall sets \(\mathbf u=\mathbf v_w\), with either an adiabatic condition \(\mathbf q\cdot\mathbf n=0\), a prescribed wall temperature, or another explicitly modelled thermal relation. These are different wall models. [NASA wall conditions](https://www.grc.nasa.gov/www/winddocs/user/bc.html)

## Shared solid geometry owns volumes and fluxes

For a stationary solid intersecting a background grid, the fluid geometry should publish one versioned description: fluid volumes, open fluid-fluid areas, oriented wall patches, connectivity and face quadrature. A triangle hit distance alone does not provide this description. A velocity collision response followed by an unchanged full-cell flux update cannot claim conservative arbitrary-solid compressible flow.

The boundary of each valid cut control volume must close geometrically. For planar area quadrature this includes

\[
\sum_{f\in\partial K_i}A_f\mathbf n_{if}=0.
\]

Neighbouring fluid cells share the same open face geometry with opposite normals. The same wall normals and areas supply pressure forces, viscous traction, heat flux and solid reaction. A uniform static pressure then has zero resultant on a closed surface when its quadrature closes. An imbalance can produce spurious momentum even though all fluid-fluid fluxes cancel. The original metric study of Nolasco and colleagues connects geometry identities with freestream preservation; its smooth mapped-grid analysis does not prove arbitrary polyhedral clipping or topology assembly. [Geometry identities and freestream preservation](https://arxiv.org/html/1911.03682)

For an inviscid impermeable stationary wall, the outward conservative flux is

\[
F_{wall}=(0,\ p_w\mathbf n,\ 0)^T.
\]

The pressure \(p_w\) is the boundary/Riemann pressure, with a declared reconstruction. In a ghost-state implementation, reflect the velocity component relative to wall motion while keeping the selected density and internal energy compatible with the EOS. Recompute total energy for the reflected velocity. A generic primitive-state copy does not enforce wall impermeability.

Small cut cells introduce a real stability cost: a flux acts on the fluid volume \(V_i\), potentially far below the background cell volume. The local explicit time-scale estimate is \(V_i/\sum_fA_f\alpha_f\). Holding the full-cell step while shrinking the fluid volume can violate the selected CFL or positivity condition. Cell merging, conservative redistribution or local stepping can address this, but each needs a specified exchange, admissibility and accuracy argument. Clamping the volume fraction changes geometry and content. Report the smallest fluid fraction, the controlling cell and any stabilization transfers.

## Moving solids require swept geometry and wall work

For a moving control volume, Reynolds transport gives

\[
\frac{d}{dt}\int_{K(t)}U\,dV
+\int_{\partial K(t)}[F_n(U)-v_{b,n}U]\,dA=0,
\qquad v_{b,n}=\mathbf v_b\cdot\mathbf n.
\]

The ALE flux is therefore \(F_n-v_{b,n}U\). Its acoustic speeds use fluid velocity relative to the face. At an inviscid impermeable solid wall \(u_n=v_{w,n}\), it reduces to

\[
F^{ALE}_{wall}=(0,\ p_w\mathbf n,\ p_wv_{w,n})^T.
\]

The outward energy flux removes energy at rate \(p_wv_{w,n}\) per area. With \(\mathbf n\) outward from the fluid, an inward-moving piston has \(v_{w,n}<0\), adding energy to the fluid. Omitting this term loses pressure work even when mass leakage is zero. For a no-slip viscous wall, the total outward energy flux is \(p_wv_{w,n}-(\tau\mathbf n)\cdot\mathbf v_w+\mathbf q\cdot\mathbf n\). Prescribed wall motion uses an external actuator; a two-way solid calculation needs the opposite force and work on the solid. The ALE derivation in Benoit and Nadarajah provides the moving-face subtraction from which these wall expressions follow. [ALE Euler balance, §2.1](https://arxiv.org/html/1804.03731)

One necessary discrete geometric conservation law is

\[
V_i^{n+1}-V_i^n
=\int_{t_n}^{t_{n+1}}\sum_f A_f(t)v_{b,n,f}(t)\,dt.
\]

Use the same time-integrated swept geometry in the volume change and the ALE content update. A freestream on compatible moving control volumes should remain a freestream; stale areas or a different velocity quadrature can manufacture density and pressure. The moving-grid study explicitly analyses integrated face velocities and tests freestream preservation. That evidence is necessary for moving geometry, but does not by itself qualify newly uncovered cells, disappearing cells or cut-cell redistribution. [Moving geometry and GCL, §§3–4](https://arxiv.org/html/1804.03731)

A moving physical piston need not preserve a uniform pressure: it compresses the fluid. Use a compatible translating fluid/wall configuration or free moving-mesh problem for a freestream test, and a separate piston work test for compression. Topology changes must transfer conservative contents over swept regions without duplicating or discarding fluid. Publish geometry, conservative state, material parameters and accepted time together. If assembly, EOS inversion or a stage fails, retain the previous coherent state and report the failure.

## Turbulence is a separate modelling choice

Euler shock capture does not establish a turbulence model. Compressible Navier–Stokes with molecular stress specifies a continuum; a simulation only qualifies as DNS when its resolution and time sampling resolve the relevant dissipative motion for the claimed regime. LES evolves filtered quantities and needs a stated subgrid stress and heat-flux treatment, explicit or implicit, with a filter/grid interpretation. RANS evolves mean quantities and closes Reynolds/Favre stresses, turbulent heat transport and any additional model equations. LES, RANS and hybrid methods introduce model-form uncertainty beyond discretization error. NASA's primary Wind-US model documentation exposes separate laminar, RANS, LES and hybrid options and their coefficients. [NASA turbulence-model documentation](https://www.grc.nasa.gov/www/winddocs/user/keywords/turbulence.html)

State the closure variant, coefficients, turbulent Prandtl treatment, inflow turbulence, transition assumptions and calibration data. Matching one drag value after retuning coefficients is calibration, not independent validation. For a wall model report \(y^+=\rho u_\tau y/\mu\), with \(u_\tau=\sqrt{|\tau_w|/\rho}\), its evaluation convention in variable-density flow, and the full first-cell distribution. Also report streamwise/spanwise spacing and wall-normal layer counts. A universal \(y^+\) target does not cover wall-resolved LES, wall-function RANS and DNS equally. Near-wall shear and heat flux can remain wrong while a bulk velocity plot looks smooth.

## A reproducible verification ladder

Use analytical solutions and conservation identities for verification of equations and code; reserve physical validation for comparisons with independent experiments and their uncertainty. NASA distinguishes verification of a code from verification of a calculation and requires convergence and consistency checks. [NASA verification](https://www.grc.nasa.gov/www/wind/valid/tutorial/verassess.html), [NASA validation](https://www.grc.nasa.gov/www/wind/valid/tutorial/valassess.html)

Every case should retain the EOS and nondimensionalization, initial conservative cell averages, physical boundary data, geometry version, reconstruction/flux/time integrator, all grid sizes and actual accepted steps, final time, reference method and its error, norms, conservation residuals, minimum density/pressure/internal energy and limiter/fallback counts. Save machine-readable values and original plotting scripts. Thresholds should follow a declared error budget; the case names below supply no universal pass percentage.

The separate [computed reference and reproduction instructions](../compressible/README.md) document the bounded first-order periodic ideal-gas experiments used by the [three-dimensional lab](../../education/compressible-lab.html). Its planar Sod data end at \(t=0.08\); periodicity introduces a second jump, so the central exact-reference comparison is restricted to \(0.2<x<0.8\). Its initial smooth fields are sampled at cell centres, and its acoustic viewer uses two nonparallel finite-amplitude modes. These are distinct from the proposed open-tube \(t=0.2\), pulse and smooth-vortex cases below. Periodic momentum conservation and open-tube pressure-force momentum change are both correct within their respective boundary models. The README records numerical checks and limitations; the lab displays their saved fields.

### V0 Local algebra and negative cases

Before a flow animation, check conservative/primitive round trips, units, uniform-state flux, orientation reversal and shared-face cancellation. Check the normal-shock ratios in Chapter 26 and their jump residuals. Present inadmissible input and excessive-CFL cases as explicit rejected operations. For solids, exercise closed area-normal sums, zero mass leakage, wall work and a GCL freestream reference separately. Exact finite proofs establish their assumptions and algebra only; they do not prove floating-point EOS evaluation or arbitrary-mesh assembly.

### V1 Sod shock tube

Use ideal gas \(\gamma=1.4\), \(0\le x\le1\), discontinuity at \(x_0=0.5\), left primitive state \((\rho,u,p)=(1,0,1)\), right state \((0.125,0,0.1)\), and final time \(t=0.2\). The remaining two velocities are zero. Choose grids with even cell counts, for example 100, 200, 400 and 800, so the initial jump lies on a face. Keep endpoint data at their undisturbed states through this time; the wave fan has not reached the ends. Use unit cross-section, or report the actual extruded volume factor.

Derive the exact ideal-gas Riemann reference by solving \(f_L(p_*)+f_R(p_*)+u_R-u_L=0\), where a rarefaction branch uses

\[
f_k(p)=\frac{2c_k}{\gamma-1}
[(p/p_k)^{(\gamma-1)/(2\gamma)}-1],
\]

and a shock branch uses \(f_k(p)=(p-p_k)\sqrt{A_k/(p+B_k)}\), \(A_k=2/[(\gamma+1)\rho_k]\), \(B_k=(\gamma-1)p_k/(\gamma+1)\). Bracket the positive root and retain the root tolerance. Approximate checkpoints are \(p_*=0.3031301781\), \(u_*=0.9274526200\), \(\rho_{*,L}=0.4263194282\), \(\rho_{*,R}=0.2655737117\); independently recompute them rather than treating rounded prose as the oracle. Wang and Li give the Riemann matching construction, including its ideal-gas limit. [Riemann reference, §2](https://arxiv.org/html/2108.13780)

Compare **conservative cell averages** of the exact solution, integrating across wave breaks, with numerical cell averages. For primitive-field comparisons, derive the reference primitives from those conservative averages, or explicitly report a different primitive-average convention: nonlinear conversion and averaging do not commute. Record density, velocity and pressure L1 errors, shock/contact locations and widths, and overshoots. Initial integrated mass is 0.5625 and total energy 1.375 in these nondimensional units. Momentum changes by the net endpoint pressure flux: its integral should be \((1-0.1)t=0.18\) at \(t=0.2\), not zero. Do not demand conservation of momentum in a tube with unequal boundary pressure forces. Smooth-region order and discontinuity-dominated norms need different interpretations. This case verifies one-dimensional wave structure; it says little about turbulence or multidimensional shock instability.

### V2 Small acoustic pulse and outlet reflection

Set \(\rho_0=p_0=1\), \(\gamma=1.4\), \(c_0=\sqrt{1.4}\), and a Gaussian \(f(x)=\epsilon\exp[-((x-0.25)/0.025)^2]\) on \([0,1]\). Initialize \(p=1+f\), \(u=f/(\rho_0c_0)\), \(\rho=1+f/c_0^2\), with other velocities zero and total energy recovered from the EOS. For \(t=0.2/c_0\) compare with the right-travelling linear reference \(f(x-c_0t)\), away from boundary contamination. Sweep \(\epsilon=10^{-3},10^{-4},10^{-5}\) and at least three grids resolving the pulse, such as 256, 512 and 1024 cells.

Measure propagation-speed/phase error, amplitude loss and the acoustic combinations \(p'\pm\rho_0c_0u'\). The unwanted left-travelling component measures a different error from damping. The linear reference has nonlinear Euler discrepancy of order \(\epsilon^2\) over fixed pre-steepening time; refinement at fixed amplitude can plateau for that reason. Repeat with the pulse leaving a characteristic outlet, then compare with a longer-domain computation at the same resolution and time to estimate reflected amplitude. State probe location, time window and amplitude normalization. A planar pulse cannot qualify oblique-wave absorption.

### V3 Smooth isentropic vortex convergence

Use \(\gamma=1.4\), background \(\rho=p=1\), convection velocity \((1,1,0.2)\), and vortex strength \(\beta=1\). With \(X=x-t\), \(Y=y-t\), \(r^2=X^2+Y^2\), define

\[
\delta u=-\frac{\beta}{2\pi}Y\exp[(1-r^2)/2],\qquad
\delta v=\frac{\beta}{2\pi}X\exp[(1-r^2)/2],
\]
\[
\Theta=1-\frac{(\gamma-1)\beta^2}{8\gamma\pi^2}\exp(1-r^2),\qquad
\rho=\Theta^{1/(\gamma-1)},\quad p=\rho^\gamma,
\quad\mathbf u=(1+\delta u,1+\delta v,0.2).
\]

These equations specify this fixture independently. Use \([-10,10]^2\times[0,1]\), periodic \(z\), analytic exterior states/compatible incoming data in \(x,y\), and \(t=1\). A truncated Gaussian vortex is not exactly periodic in \(x,y\); do not call a wrapped seam an exact smooth periodic solution. Compare 64², 128² and 256² grids with a declared extruded \(z\) resolution. Integrate initial and reference conservative cell averages with quadrature more accurate than the expected numerical error. Nolasco and colleagues use an analytical three-dimensional isentropic vortex to assess metric accuracy, providing a primary reference for this type of smooth test. [Vortex study, §5.1](https://arxiv.org/html/1911.03682)

For \(q=\rho,p,u,v,w\), report volume-weighted L1/L2/L∞ errors and the observed refinement rate \(\log(e_h/e_{h/2})/\log2\). Refine time independently so spatial and temporal orders can be distinguished. First-order LLF should not be advertised as a second-order method because the reference is smooth. A z-invariant extrusion exercises three momentum components but does not verify fully three-dimensional vortex interaction or turbulence.

### V4 Oblique shock and inclined wall

Use a calorically perfect gas with \(\gamma=1.4\), \(M_1=2\), \(\rho_1=p_1=1\), zero attack angle and a stationary slip wedge turning the flow through \(\theta=10^\circ\). Then \(u_1=M_1\sqrt{\gamma p_1/\rho_1}\). Solve the weak attached-shock root \(\beta_s\) of

\[
\tan\theta=2\cot\beta_s
\frac{M_1^2\sin^2\beta_s-1}
{M_1^2(\gamma+\cos2\beta_s)+2}.
\]

Use upstream normal Mach \(M_{n1}=M_1\sin\beta_s\), apply the normal-shock ratios, and obtain \(M_2=M_{n2}/\sin(\beta_s-\theta)\). The weak root is approximately \(39.31393^\circ\). Retain branch, bracket and angular units in the reference. NASA's oblique-shock equations state the turning relation and normal/tangential decomposition and link to the original NACA report. [NASA oblique shocks](https://www.grc.nasa.gov/www/k-12/airplane/oblique.html), [NACA 1135](https://ntrs.nasa.gov/citations/19930091059)

Publish domain extents, wedge location, wall representation, farfield/outlet data and convergence criteria. Compare shock angle, pressure and density ratios, wall impermeability and global boundary-flux budgets under grid and boundary-distance refinement. An embedded wedge needs the same shared-geometry checks as other cut cells. Add a perturbed grid-aligned strong-shock case before claiming immunity to carbuncle instability. This inviscid attached-shock reference excludes separation, viscous wall heating and detached shocks beyond the attachment limit.

### V5 Laminar boundary layer, then heat and turbulence

A first viscous check can use the NASA low-Mach flat-plate study: \(M_\infty=0.1\), static pressure 6.0 psia, temperature 700 Rankine, plate length 1 ft and reported \(Re_L\approx200000\). Its documented grid, inlet/outlet and wall choices make the case reproducible. Record the selected viscosity law and exact resulting Reynolds number; do not obtain a desired Reynolds number by silently changing material parameters. No archive code or figures are included here. [NASA laminar flat plate, Study 1](https://www.grc.nasa.gov/www/wind/valid/fplam/fplam01/fplam01.html)

For the incompressible, constant-property, zero-pressure-gradient approximation, independently solve Blasius' boundary-value problem

\[
f'''+\tfrac12ff''=0,\qquad f(0)=f'(0)=0,\quad f'(\infty)=1,
\quad\eta=y\sqrt{U_\infty/(\nu x)}.
\]

Its reference values give local \(C_f=0.664/\sqrt{Re_x}\) and \(\delta^*/x=1.7208/\sqrt{Re_x}\). Compare velocity profiles, displacement thickness and wall shear at declared downstream stations, excluding the leading-edge singular region. Refine streamwise and wall-normal resolution separately and demonstrate steady convergence. At finite Mach this incompressible comparison is an approximation; for a tighter compressible claim use a compressible similarity/reference solution with the same thermal boundary and viscosity law. Adiabatic low-Mach agreement cannot qualify high-Mach heat transfer. Add heat-conduction/thermal-wall tests before claiming conductivity, then independent transition and turbulent-wall evidence before claiming a calibrated turbulence closure.

## What the ladder establishes

Passing V1–V3 can support a bounded inviscid algorithm claim for the documented domains and regimes. V4 adds inclined-wall and shock evidence; V5 adds molecular-stress evidence under a declared thermal model. Arbitrary moving solids, EOS mixtures, shock–boundary-layer interaction, high-temperature gas and turbulence remain additional questions. Original reference data, checked finite identities and a three-dimensional teaching visualization are useful evidence with distinct scopes; none alone establishes a general production compressible solver.
