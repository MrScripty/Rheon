# 43 Thermal expansion, buoyancy and convection

**Research exposition; book author: Puma.** Heat changes motion through density, pressure, viscosity and interfacial stresses. A prescribed upward force can imitate a plume's appearance while missing these couplings. This chapter derives the reduced models before comparing regimes. [Chapter 47](47-validation-and-research-conclusions.md) gives scoped reference cases.

## 43.1 Expansion follows the equation of state

For a differentiable single-phase EOS \(\rho(T,p,Y)\), define \(\beta=-\rho^{-1}(\partial\rho/\partial T)_{p,Y}\), \(\kappa_T=\rho^{-1}(\partial\rho/\partial p)_{T,Y}\), and \(\eta_a=\rho^{-1}(\partial\rho/\partial Y_a)_{T,p,Y_{b\ne a}}\) for independent composition coordinates. Mass conservation gives the exact smooth-phase identity

\[
\nabla\cdot\mathbf u=
\beta\frac{DT}{Dt}-\kappa_T\frac{Dp}{Dt}
-\sum_a\eta_a\frac{DY_a}{Dt}. \tag{43.1}
\]

The proof is \(D\rho/Dt=-\rho\nabla\cdot\mathbf u\) followed by the chain rule. It assumes barycentric velocity, no bulk total-mass source, and a well-defined EOS derivative. Gas heating generally produces divergence even without evaporation. A closed vessel can raise pressure instead of expanding globally; its pressure evolution is constrained by the volume and energy budgets.

## 43.2 Deriving the Boussinesq reduction

Take a reference density \(\rho_0\), temperature \(T_0\), fixed composition and small relative density variation, with \(\rho\approx\rho_0[1-\beta_0(T-T_0)]\). Let uniform gravity be \(\mathbf b\), and choose a reference hydrostatic pressure satisfying \(\nabla p_h=\rho_0\mathbf b\). Subtract that reference balance from momentum and retain density variation only in the gravity term:

\[
\nabla\cdot\mathbf u=0,
\qquad \frac{D\mathbf u}{Dt}
=-\frac{\nabla\pi}{\rho_0}+\nu\nabla^2\mathbf u
-\beta_0(T-T_0)\mathbf b,
\]
\[
\frac{DT}{Dt}=\alpha\nabla^2T+Q/(\rho_0c_p). \tag{43.2}
\]

With vertical coordinate upward and \(\mathbf b=-g\mathbf e_z\), a warm perturbation with \(\beta_0>0\) accelerates upward. The sign follows from subtracting hydrostatic support, not from declaring heat to be an upward force. Uniform heating in a perfectly filled Boussinesq domain changes the thermal state but does not change its modeled total volume.

The reduction is asymptotic modeling: it requires small \(|\beta_0\Delta T|\), small pressure-induced density changes and appropriate stratification, slowly varying properties over the range, low acoustic Mach number, and negligible compressional/dissipative heating for its chosen thermal equation. Small \(|\beta\Delta T|\) alone is insufficient if viscosity changes sharply or gas stratification spans many scale heights. The reference model deliberately drops density variations from inertia and continuity while retaining their leading gravity effect. It is not exact variable-density conservation.

It breaks down for substantial thermal dilation, evaporation with large density ratio, strongly heated gas, rapid pressure changes, near-critical anomalies, and EOS regions where linear expansion fails or changes sign. Water near its density maximum cannot be modeled with a single positive constant \(\beta\) across that maximum. A variable-property low-Mach model may be appropriate before a compressible model is needed; low Mach does not imply constant density.

## 43.3 Ratios tell us which terms compete

Choose characteristic length \(L\), speed \(U\), temperature difference \(\Delta T\), and reference \(\nu=\mu/\rho\), \(\alpha=k/(\rho c_p)\). The dimensionless groups are definitions for this scale choice:

| Group | Definition | Comparison |
|---|---|---|
| Reynolds | \(Re=UL/\nu\) | inertia / viscous diffusion |
| Prandtl | \(Pr=\nu/\alpha\) | momentum / thermal diffusivity |
| thermal Péclet | \(Pe=UL/\alpha=RePr\) | thermal advection / conduction |
| Grashof | \(Gr=g\beta\Delta T L^3/\nu^2\) | buoyancy / viscous scaling squared |
| Rayleigh | \(Ra=g\beta\Delta T L^3/(\nu\alpha)=GrPr\) | buoyancy with thermal diffusion |
| Richardson | \(Ri=Gr/Re^2=g\beta\Delta T L/U^2\) | buoyancy / forced-flow inertia |
| Mach | \(Ma=U/c_s\) | flow / acoustic speed |
| Nusselt | \(Nu=h_cL/k\) | heat transfer / chosen conductive reference |
| Biot | \(Bi=h_cL_s/k_s\) | solid internal / external thermal resistance scale |
| Fourier | \(Fo=\alpha t/L^2\) | elapsed / diffusion time |

Signs of \(Ra\) and \(Ri\) depend on the signed temperature difference and geometry; for magnitude comparisons use absolute values and state the stratification. A stably stratified layer can suppress motion despite a large magnitude of buoyancy forcing. There is no universal critical Reynolds or Rayleigh number for every container, wall condition or free surface.

For naturally driven motion choose a free-fall speed \(U_f=\sqrt{g\beta\Delta T L}\) for destabilizing positive scales. Then \(Re_f=\sqrt{Ra/Pr}\) and \(Pe_f=\sqrt{RaPr}\). These follow algebraically and do not predict a turbulent heat-transfer coefficient. In viscous-dominated natural convection a different speed scale may be more useful. Thin thermal layers at large \(Pr\), thin velocity layers at small \(Pr\), and strong advection at large \(Pe\) place different demands on the mesh.

Forced convection has imposed pressure, wall motion or inlet momentum. Natural convection generates flow through buoyancy. Mixed convection requires both; a sign and geometry must be attached to \(Ri\) to distinguish aiding and opposing flow. A measured wall flux should be normalized by a clearly stated \(\Delta T,L,k\) before reporting \(Nu\). Reporting a color field as a heat-transfer result omits the essential flux.

The [de Vahl Davis 1983 primary record](https://doi.org/10.1002/fld.1650030305) identifies the two-dimensional differentially heated square cavity and \(10^3\le Ra\le10^6\). Its published benchmark is a scoped laminar natural-convection reference, not a proof of arbitrary three-dimensional turbulent convection. This audit read the publisher abstract/metadata; numeric tables are not copied or used as verified targets here.

## 43.4 Surface tension can drive flow without gravity

Let a clean interface have surface tension \(\sigma(T,Y_s)\). Define \(\nabla_s=(I-\mathbf n\mathbf n^T)\nabla\). Surface stress has a tangential contribution

\[
\nabla_s\sigma=\sigma_T\nabla_sT+
\sum_a\sigma_{Y_a}\nabla_sY_a. \tag{43.3}
\]

When \(\sigma_T<0\), the surface is pulled toward colder, higher-tension regions. The adjacent bulk recirculation depends on viscosity ratio, confinement and thermal transport; a universal translation direction for all drops does not follow from this sentence. Trace surfactant, dissolved material and contamination can oppose a thermal stress. A pairwise constant cohesion approximation is not a thermocapillary stress discretization.

Define \(Ma_T=|\sigma_T|\Delta T L/(\mu\alpha)\), the thermal Marangoni number, separately from acoustic Mach \(Ma\). Its scale estimates surface driving against viscous/thermal diffusion. The capillary number \(Ca=\mu U/\sigma\), Weber number \(We=\rho U^2L/\sigma\), and Bond number \(Bo=\Delta\rho gL^2/\sigma\) compare viscous, inertial and gravitational effects with capillarity. They do not describe nucleation barriers. The [Young–Goldstein–Block paper](https://doi.org/10.1017/S0022112059000684) is the primary provenance for classical thermocapillary bubble motion; only its bibliographic record/abstract was inspected here, so no unverified speed formula is imported.

## 43.5 Thermal and solutal effects interact

For a dilute independent concentration \(C\), write a local density approximation \(\rho/\rho_0\approx1-\beta_T(T-T_0)+\beta_C(C-C_0)\). Even if thermal and compositional buoyancy cancel initially, unequal diffusivities can destroy the cancellation. A solutal Rayleigh number, Schmidt number \(Sc=\nu/D\), and Lewis number \(Le=\alpha/D\) distinguish these effects. Sign conventions for \(\beta_C\) must follow the chosen concentration units.

Evaporation cools a surface and enriches the surrounding gas in vapor. Both can alter gas density; whether the plume rises or sinks depends on molecular weights, heat, pressure and boundary conditions. Vapor cannot generally be treated as a passive appearance tracer. Multicomponent liquids also change composition as the more volatile component leaves, changing saturation and surface tension. The [2024 multicomponent VOF paper's primary institutional record](https://www.sintef.no/en/publications/publication/0198cc4292bb-664f2bd2-b563-4c80-bf57-17b7ee1b1cad/) identifies a Robin species-coupling approach; this draft does not adopt its unread detailed closure.

## 43.6 A sensible research boundary

The first motion study can use a single fluid, constant positive transport properties and explicitly bounded Boussinesq temperature range. Its outputs should include wall heat flux, thermal energy exchange, velocity, hydrostatic residual and refinement. A later variable-density low-Mach study must use (43.1), consistent material properties and absolute background pressure. A phase-change study must additionally satisfy the interfacial divergence and energy conditions of Chapter 45. Passing the first study does not qualify the latter two.
