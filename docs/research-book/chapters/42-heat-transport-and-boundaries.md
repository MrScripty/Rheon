# 42 Heat transport, properties and boundary exchange

**Research exposition; book author: Puma.** This chapter uses the energy and sign conventions of [Chapter 41](41-temperature-energy-and-entropy.md). It develops a single-phase thermal foundation before adding phase change. Relations derived here are mathematical checks under named assumptions, not newly executed Rheon heat tests.

## 42.1 Fourier conduction is a flux law

In an isotropic continuum near local equilibrium, \(\mathbf q=-k(T,p)\nabla T\), where conductivity has units W/(m K). In an anisotropic material replace \(k\) by a symmetric positive semidefinite tensor. Positive conduction transfers heat down a temperature gradient and produces entropy \(\nabla T\cdot k\nabla T/T^2\). Fourier's parabolic law has no finite signal speed; very short molecular times, rarefaction, ballistic transport and strongly nonequilibrium interfaces require a different closure.

For fixed constant density, negligible pressure/dissipative heating and a specified sensible-energy function \(H(T)\) with \(H'(T)=c>0\), use

\[
\partial_t(\rho H)+\nabla\cdot(\rho H\mathbf u)
=\nabla\cdot(k\nabla T)+Q,\qquad \nabla\cdot\mathbf u=0. \tag{42.1}
\]

Here \(c\) denotes the heat capacity selected by the reduced thermodynamic model; it is not an assertion that physical \(c_p=c_v\) for every liquid. Constant-pressure low-Mach enthalpy commonly uses \(c_p\). Only when \(\rho,c,k\) are constant does (42.1) become

\[
\partial_t T+\mathbf u\cdot\nabla T=\alpha\nabla^2T+Q/(\rho c),
\qquad \alpha=k/(\rho c). \tag{42.2}
\]

Diffusivity \(\alpha\) has units m²/s. Diffusion crosses a length \(L\) over order \(L^2/\alpha\). Doubling a physical length therefore multiplies this time by four. Increasing preview resolution without changing physical dimensions does not change this physical scale.

## 42.2 The boundary is an energy source or sink

Use the outward normal \(\mathbf n\) of the fluid domain. A prescribed temperature is Dirichlet data, \(T=T_w\); it implies whatever heat is necessary to maintain that temperature. A prescribed outward heat flux is Neumann data, \(\mathbf q\cdot\mathbf n=q_{out}\), equivalently \(-k\partial_nT=q_{out}\). An insulated boundary has \(q_{out}=0\), not \(T=0\).

Newton cooling is a Robin condition,

\[
\mathbf q\cdot\mathbf n=h_c(T-T_\infty),\qquad h_c\ge0, \tag{42.3}
\]

with \(h_c\) in W/(m² K). A lumped body's relaxation rate is \(h_cA/(mc)\), in s⁻¹, when its internal temperature can be treated as uniform. Rheon's particle wall relaxation rate is already in s⁻¹; it is not a measured heat-transfer coefficient without identifying parcel contact area and heat capacity. Near a corner the current axis-wise relaxation can apply more than once; a future wall-area discretization must declare its geometric meaning.

For two perfect-contact materials, the temperature traces agree and the normal heat flux is continuous when both use the same normal. With an interfacial thermal resistance \(R_K\) and common flux directed from material 1 to 2, \(T_1-T_2=R_Kq_{1\to2}\), \(R_K\ge0\). Its entropy production is \(q_{1\to2}(1/T_2-1/T_1)\ge0\). A heat-generating interface instead has a flux jump. A phase-changing boundary has the latent-heat balance in Chapter 45; continuity of heat flux alone would remove the heat consumed by vaporization.

A radiating surface may exchange \(\epsilon\sigma_{SB}(T^4-T_{sur}^4)\) W/m² in a gray diffuse model with its view-factor assumptions. This is an additional boundary model, not Fourier conduction through empty space. Strong radiative, reactive or plasma conditions are outside the first thermal milestone.

## 42.3 A face-flux conservation proof

Partition a fixed domain into cells. Let \(U_i=\rho_iH_iV_i\). If each internal face has a single energy transfer \(F_{ij}=-F_{ji}\) in W, the update \((U_i^+-U_i)/\Delta t=-\sum_jF_{ij}+Q_iV_i\) satisfies

\[
\sum_i(U_i^+-U_i)
=\Delta t\left(\sum_iQ_iV_i-\sum_{f\subset\partial\Omega}F_f\right). \tag{42.4}
\]

**Proof:** sum all cells; every internal face appears twice with opposite signs. This is exact-real algebra independent of how accurate the face approximation is. In floating point, the measured defect depends on arithmetic and reductions. Cancellation does not prove that fluxes are physically correct, that temperatures stay positive, or that a curved boundary area is resolved.

For a stationary layered conductor, integrate steady \(q=-k\,dT/dx\). A face separating distances \(d_i,d_j>0\), area \(A\), and positive constant conductivities has conductance

\[
G_{ij}=\frac{A}{d_i/k_i+d_j/k_j},\qquad
F_{i\to j}=G_{ij}(T_i-T_j). \tag{42.5}
\]

This is a resistance-in-series derivation, giving a harmonic conductivity for equal distances. An arithmetic average of \(k\) incorrectly conducts through a poorly conducting layer. Cut cells, contact resistance, anisotropy and nonorthogonal grids need their own geometric derivation. An arithmetic volume average may be appropriate for cell energy capacity; that does not justify arithmetic averaging for every face coefficient.

For frozen positive capacities \(C_i=m_ic_i\) and conductances \(G_{ij}=G_{ji}\ge0\), isolated conduction obeys \(C_i\dot T_i=\sum_jG_{ij}(T_j-T_i)\). Summing proves conservation of \(\sum_i C_iT_i\). Multiplying by \(T_i\) gives

\[
\frac{d}{dt}\frac12\sum_i C_iT_i^2
=-\sum_{i<j}G_{ij}(T_i-T_j)^2\le0. \tag{42.6}
\]

This quadratic functional is not thermodynamic entropy. For positive temperatures and a fixed reference \(T_{ref}>0\), multiplying by \(1/T_i\) instead shows
\(d\sum_i C_i\log(T_i/T_{ref})/dt=\sum_{i<j}G_{ij}(T_i-T_j)^2/(T_iT_j)\ge0\).
Both results assume fixed capacities, no boundary exchange and the stated graph. They do not automatically cover variable properties, explicit finite steps or latent energy.

## 42.4 Advection: a temperature interpolant is insufficient

For constant \(\rho c\), bounded semi-Lagrangian interpolation may keep temperature within donor extrema. It need not conserve \(\int\rho H\,dV\). Rheon's `advect_tracer` explicitly declares this distinction. Conservative thermal transport should share a face mass flux with the density update and transport the matching specific energy or enthalpy. If hot and cold phases have unequal capacities, averaging temperatures conserves neither internal energy nor enthalpy in general.

For a simple two-mass mixing calculation with no work or loss, identical composition, no phase change and constant heat capacities, conservation gives

\[
T_f=\frac{m_1c_1T_1+m_2c_2T_2}{m_1c_1+m_2c_2}. \tag{42.7}
\]

This is a hand reference for energy weighting, not a mixture EOS. With variable capacity solve \(\sum m_iH_i(T_f)=\sum m_iH_i(T_i)\); strict monotonicity of each \(H_i\) provides uniqueness between the extreme initial temperatures. If equilibrium crosses saturation, this inverse must instead solve the phase-energy problem.

## 42.5 Stability, positivity and stiffness are separate

Forward Euler applied to the frozen conduction graph is a convex combination if

\[
\Delta t\sum_jG_{ij}/C_i\le1. \tag{42.8}
\]

This proves a discrete maximum principle with appropriate bounded boundary forcing. On an orthogonal uniform d-dimensional grid it yields \(\Delta t\le[2\alpha\sum_{a=1}^d1/\Delta x_a^2]^{-1}\). Heating sources and sinks impose additional bounds. The bound is sufficient for this update, not a universal necessary condition.

Backward Euler with positive capacities and a connected nonnegative-conductance graph forms a matrix with positive diagonal, nonpositive off-diagonal entries and a positive row margin; its inverse is nonnegative under the usual finite graph boundary conditions. It preserves bounds for the isolated linear problem at arbitrary step size. A large step can still be inaccurate. Crank–Nicolson can be linearly stable yet oscillate below zero for stiff modes. Nonlinear EOS inversion, source splitting, approximate solves and phase-change coupling require renewed admissibility checks.

## 42.6 Property models and their provenance

Water conductivity and viscosity are separate functions of state. The [IAPWS 2011 conductivity release, §2](https://iapws.org/technical-guidance/release/ThCond.download) and [2008 viscosity release, §2](https://iapws.org/technical-guidance/release/viscosity.download) include near-critical contributions and bounded domains; neither grants a metal or polymer closure. A temperature-dependent \(k\) cannot be pulled outside \(\nabla\cdot(k\nabla T)\). Likewise a variable \(\mu\) must remain inside stress divergence.

Record units, validity region, phase branch, energy reference and uncertainty for every property set. A table must preserve positive capacities/compressibility where required, monotone energy inversion, and consistent saturation states. Arbitrarily interpolating \(p,e,h,c_p,\rho\) independently may violate thermodynamic identities even when every table entry is positive. [CoolProp's high-level interface](https://coolprop.org/coolprop/HighLevelAPI.html) documents saturation and phase queries; its permissively licensed implementation is a possible reference oracle, not an imported dependency in this research.

Incoming reservoir fluid supplies composition and specific enthalpy as well as temperature. Outgoing fluid carries the interior state. A periodic seam must use one matched conductive and advective flux on its two traces. These thermal dependencies supplement the separate Chapter 40 portal research without editing it. A constant wall temperature, a finite wall with conjugate heat capacity, and an infinite reservoir are different physical experiments.
