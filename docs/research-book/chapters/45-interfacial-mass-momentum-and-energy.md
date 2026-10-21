# 45 Interfacial mass, momentum and energy

**Research exposition; book author: Puma.** Fix signs before selecting a discretization. This chapter uses a sharp liquid–vapor interface, negligible interfacial mass, and no chemical reaction. Bulk properties have well-defined traces. Surface energy and surface stresses are retained where stated; the reduced Stefan balance is identified separately from the full energy problem.

## 45.1 Orientation and kinematics

Let \(\mathbf n\) point **from liquid to vapor**, \([\![a]\!]=a_v-a_l\), and \(V_\Gamma\) be interface normal speed in that direction. Define positive evaporative mass flux \(j\), in kg/(m² s), by

\[
j=\rho_l(\mathbf u_l\cdot\mathbf n-V_\Gamma)
=\rho_v(\mathbf u_v\cdot\mathbf n-V_\Gamma). \tag{45.1}
\]

**Pillbox proof:** integrate mass conservation over a control volume moving with the interface. Its two normal mass fluxes are equal as thickness tends to zero if the interface has no mass storage. Therefore

\[
[\![\mathbf u\cdot\mathbf n]\!]
=j(1/\rho_v-1/\rho_l),\qquad
V_\Gamma=\mathbf u_l\cdot\mathbf n-j/\rho_l. \tag{45.2}
\]

An evaporating planar surface with stationary liquid recedes into the liquid. A bubble has a different geometric normal: for liquid outside a vapor sphere, \(\mathbf n\) points inward and \(V_\Gamma=-\dot R\). Never reuse a droplet radius sign for a vapor bubble without this conversion.

If the two bulk densities are constant and each bulk velocity is divergence free, the piecewise physical velocity has the distributional divergence

\[
\nabla\cdot\mathbf u=j(1/\rho_v-1/\rho_l)\delta_\Gamma. \tag{45.3}
\]

The surface delta is normalized by \(\int_\Omega f\delta_\Gamma dV=\int_\Gamma f\,dA\), hence has units 1/m. One derivation integrates divergence over a thin interface-crossing column; the outgoing normal-velocity jump is (45.2). Thermal expansion and composition add the smooth bulk terms of (43.1). Equation (45.3) does not assign a unique interpolated velocity inside a finite-thickness numerical band.

Let \(\chi_l\) be the liquid indicator. In the distributional sense,

\[
\partial_t(\rho_l\chi_l)+\nabla\cdot(\rho_l\chi_l\mathbf u_l)
=-j\delta_\Gamma,
\]
\[
\partial_t(\rho_v\chi_v)+\nabla\cdot(\rho_v\chi_v\mathbf u_v)
=+j\delta_\Gamma. \tag{45.4}
\]

Adding proves conservation of total mass. Dividing each equation by its constant phase density and adding proves (45.3). Liquid mass loss and vapor mass gain are equal, but their volume changes are unequal. A geometric volume fraction is not a vapor mass quality.

The [Scapin–Costa–Brandt preprint, §2, equations (3)–(4)](https://arxiv.org/pdf/1907.00890v2) and [Malan et al., §2, equations (1)–(7)](https://arxiv.org/pdf/2001.03477) independently provide useful primary comparisons for these kinematic relations. Their curvature signs and numerical velocities must still be translated into this chapter's convention.

## 45.2 Projection must target the required divergence

For a candidate velocity \(\mathbf u^*\), consider the modeled pressure correction

\[
\mathbf u^{n+1}=\mathbf u^*-\Delta t\,\rho^{-1}\nabla\pi,
\qquad \nabla\cdot(\Delta t\,\rho^{-1}\nabla\pi)
=\nabla\cdot\mathbf u^*-S, \tag{45.5}
\]

where \(S\) contains the chosen bulk dilation and interfacial volume production. This is obtained simply by imposing \(\nabla\cdot\mathbf u^{n+1}=S\). It is not a derivation of an entire low-Mach time integrator. Coefficients, interface pressure jumps, velocity traces and pressure boundary conditions remain part of that integrator.

Integrating the target constraint gives

\[
\int_{\partial\Omega}\mathbf u\cdot\mathbf n_{out}\,dA
=\int_\Omega S\,dV. \tag{45.6}
\]

In a sealed rigid domain with two exactly incompressible constant-density phases, net evaporation generally violates this compatibility if no compensating mechanism exists. A pressure gauge cannot fix a nonzero net source. The physical resolution may be gas compression and evolving background pressure, compliant boundaries, condensation elsewhere, or external volume flux. Subtracting the mean of \(S\) silently removes physical volume creation. Rheon's existing constant-density zero-divergence gates must not simply be relaxed to admit this new physics.

## 45.3 Momentum, capillarity and recoil

Let \(\boldsymbol T=-pI+\boldsymbol\tau\). Choose \(\kappa=\nabla_s\cdot\mathbf n\), positive \(2/R\) for a spherical liquid drop. The surface force per area is
\(\mathbf f_s=\nabla_s\sigma-\sigma\kappa\mathbf n\).
Under negligible surface inertia and no other surface traction,

\[
j[\![\mathbf u]\!]=[\![\boldsymbol T]\!]\mathbf n
+\nabla_s\sigma-\sigma\kappa\mathbf n. \tag{45.7}
\]

**Derivation:** apply momentum conservation to the same moving pillbox. Net advected momentum is \(j\mathbf u_v-j\mathbf u_l\); bulk tractions sum to \([\![\boldsymbol T]\!]\mathbf n\), with surface traction \(\mathbf f_s\). The assumed zero surface momentum storage equates them. This checks the sign independently of a chosen continuum-surface-force formula.

Its normal component and (45.2) give

\[
p_l-p_v=\sigma\kappa+j^2(1/\rho_v-1/\rho_l)
-[\![\tau_{nn}]\!]. \tag{45.8}
\]

The \(j^2\) term is vapor recoil, with pressure units. Static evaporation-free drops recover \(p_l-p_v=2\sigma/R\). The tangential balance is
\(j[\![\mathbf u_t]\!]=[\![\boldsymbol\tau\mathbf n]\!]_t+\nabla_s\sigma\).
For continuous tangential velocity it sets the viscous traction jump to \(-\nabla_s\sigma\). The physical surface force still points toward increasing tension. Reversing normal or defining curvature with the opposite sign changes the written formula, not the physics.

An inviscid gas does not generally permit ignoring liquid tangential stress if \(\sigma\) varies. A constant pair-cohesion parameter, constant pressure offset, or density-only capillary pressure cannot represent all three terms. Additional surfactant surface transport or surface viscosity requires surface constitutive laws beyond (45.7).

## 45.4 What the Stefan balance omits

For two phases of a pure fluid without species diffusion and a planar interface translating at constant normal speed, use its inertial rest frame and relative velocity \(\mathbf w=\mathbf u-V_\Gamma\mathbf n\). Assume no surface energy storage, no change of surface area, no tangential surface-energy transport or external surface heating. The total-energy jump is

\[
\left[\!\left[
j\left(e+\tfrac12|\mathbf w|^2\right)
-(\boldsymbol T\mathbf n)\cdot\mathbf w+\mathbf q\cdot\mathbf n
\right]\!\right]=0. \tag{45.9}
\]

Pressure work has not disappeared. Using \(w_n=j/\rho\) transforms this to

\[
j\left[\!\left[h+\tfrac12|\mathbf w|^2\right]\!\right]
-[\![(\boldsymbol\tau\mathbf n)\cdot\mathbf w]\!]
+[\![\mathbf q\cdot\mathbf n]\!]=0. \tag{45.10}
\]

For low relative kinetic-energy flux and negligible viscous work, local equilibrium at \(T_\Gamma\) gives the reduced Stefan condition

\[
jL(T_\Gamma)=\mathbf q_l\cdot\mathbf n-\mathbf q_v\cdot\mathbf n
=k_v\partial_nT_v-k_l\partial_nT_l. \tag{45.11}
\]

This is an energy-limited interface relation, not an independent nucleation or kinetic rate law. It needs a saturation/chemical-potential condition, or an explicitly selected finite-resistance interface closure, to determine the interface temperature. A vapor–air mixture also needs species transport. A single fixed saturation temperature is only a local approximation at a named pressure, composition and curvature regime.

For a curved moving interface the surface has energy and does mechanical work. With constant \(\sigma\), the closed-system energy ledger includes \(\sigma A\); changing area exchanges capillary and bulk mechanical energy. The planar no-storage derivation cannot be promoted to an exact curved-interface total-energy theorem. For temperature-dependent \(\sigma\), surface free energy differs from surface internal energy: at fixed surface composition a simple surface thermodynamics has surface entropy \(-\partial\sigma/\partial T\) and internal-energy density \(\sigma-T\partial\sigma/\partial T\). Surface composition/work conventions must be declared. A complete exact thermal capillary budget requires those surface balances; (45.11) is a reduced model.

## 45.5 Species transport and Stefan flow

Consider a pure volatile liquid against an ideal binary mixture of its vapor and an insoluble noncondensable gas. Use barycentric gas velocity and Fick vapor diffusion \(\mathbf J_v=-\rho_gD\nabla Y_v\), \(D\ge0\), with \(\mathbf J_a=-\mathbf J_v\). No noncondensable mass crosses the interface. The relative noncondensable flux therefore satisfies

\[
j(1-Y_{v,\Gamma})+\rho_gD\partial_nY_v=0,
\qquad j=-\frac{\rho_gD\partial_nY_v}{1-Y_{v,\Gamma}}. \tag{45.12}
\]

An outward-decreasing vapor fraction has \(\partial_nY_v<0\), hence positive evaporation. Gas diffusion induces a bulk motion, usually called Stefan flow. The denominator is not optional except in an explicitly dilute approximation. As \(Y_{v,\Gamma}\to1\), this binary diffusion-limited formula degenerates; pure-vapor boiling/condensation is a different limit and cannot be computed by inserting one and dividing by zero.

For variable gas state use conservative species transport,

\[
\partial_t(\rho_gY_v)+\nabla\cdot(\rho_gY_v\mathbf u_g+\mathbf J_v)=0 \tag{45.13}
\]

in the gas bulk, with interface exchange and exterior conditions. Diffusing species transport enthalpy \(\sum_a h_a\mathbf J_a\); the energy equation must either include that flux explicitly or define its nonadvective heat flux to include it. Omitting it while claiming general multicomponent energy conservation is inconsistent. This draft's simple formulas exclude reactions, gas dissolution, multicomponent diffusion matrices and Soret/Dufour coupling.

## 45.6 Solving the interface state and rate together

At each physical interface location, temperature, phase chemical potentials, heat fluxes and species fluxes form one coupled problem. An equilibrium vapor fraction evaluated at an old temperature can overpredict transfer, which cools the interface, which reduces saturation pressure. Condensation gives the reverse latent heating feedback. Interface kinetic resistance may introduce a relation between flux and a chemical-potential/temperature departure; an accommodation coefficient must be measured or justified. It must not be adjusted only to hide timestep dependence.

The [Palmore–Desjardins preprint, §2.1 and numerical coupling sections](https://arxiv.org/pdf/1906.04565) treats equilibrium, species and temperature as coupled conditions in a low-Mach VOF framework. Its reported stability is specific to its constructed coupling; it does not prove that an arbitrary split saturation update is stable. At very small length scale, rarefaction or strong nonequilibrium, a continuum equilibrium jump needs explicit reconsideration.

## 45.7 A reference accounting calculation

As an invented arithmetic witness, not material data, take \(j=0.01\) kg/(m² s), \(\rho_l=1000\) kg/m³, \(\rho_v=1\) kg/m³ and \(L=2\times10^6\) J/kg at a flat interface. Then the liquid-relative interface speed magnitude is \(10^{-5}\) m/s; the vapor–liquid normal velocity jump is \(0.00999\) m/s; the required heat-flux difference is \(20{,}000\) W/m²; and recoil is \(9.99\times10^{-5}\) Pa. Over one square metre for one second, 0.01 kg changes phase, liquid volume decreases by \(10^{-5}\) m³ and vapor volume increases by 0.01 m³.

All four consequences use the same mass transfer. Advancing interface recession from one \(j\), pressure dilation from another and latent energy from a third destroys closure even if each individual equation looks plausible. Condensation changes the signed mass/heat/volume transfer; recoil remains quadratic in \(j\). This witness checks units and orientation, not accuracy at physical water properties.
