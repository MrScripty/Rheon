# 47 Validation ladder and staged research conclusions

**Research draft for proposed Chapter 47; book author: Puma.** This is a proposed validation program, not a record of executed thermal, boiling or hardware tests. Analytic references test equations under their assumptions; published numerical benchmarks test reproducibility; experimental comparisons test physical closure within uncertainty. These roles must remain separate.

## 47.1 What to measure before looking at pictures

For a fixed domain report signed balance residuals after subtracting boundary/source exchange. Total mass uses both phases and all species. Momentum includes gravity, pressure/viscous tractions, surface forces, inlet/outlet flux and actuator force/impulse. Energy includes the chosen internal or total-energy definition, latent offsets, conductive and diffusive enthalpy flux, pressure and actuator work where retained, wall/reservoir heat and surface energy where modeled. A disappearing drop is not mass loss if its vapor remains in the domain.

Record minimum density/absolute temperature, phase-fraction bounds, species bounds and sum, EOS domain failures, divergence relative to required \(S\), phase inventories, interface area/location, one-sided flux jump, and pressure compatibility. Conservation error should be normalized by a physically relevant energy/mass scale; relative error divided by a nearly zero net source can be meaningless. Record absolute residual as well.

Refine space and time independently, compare at a common physical time, and distinguish iterative tolerance from truncation error. At least three asymptotic resolutions can support an observed order only when the error behaves regularly and the reference is more accurate. Topology events, startup singularities and changing physical diffuse width can spoil simple Richardson estimates. No target tolerance is admitted by this manuscript.

## 47.2 Exact thermal references derived here

On a periodic interval with constant diffusivity, uniform speed \(U\), and no source, the function

\[
T(x,t)=T_0+A\exp(-\alpha k^2t)\cos(k(x-Ut)),
\qquad k=2\pi n/L \tag{47.1}
\]

solves \(T_t+UT_x=\alpha T_{xx}\). Direct differentiation proves the identity; periodicity requires integer \(n\). Choose \(T_0>|A|\) to guarantee positive absolute temperature. Setting \(U=0\) isolates diffusion; setting \(\alpha=0\) isolates phase-speed/advection error. This smooth, constant-property reference does not validate interfaces, discontinuous conductivity, wall heat transfer or buoyancy.

For steady conduction through layers \(d_a,k_a\), with prescribed end temperatures and no volumetric heat, constant flux satisfies

\[
q=(T_{hot}-T_{cold})/\sum_a(d_a/k_a). \tag{47.2}
\]

Temperature is piecewise linear and continuous at perfect contacts. This derives by integrating Fourier's law through each layer. It tests property jumps, harmonic resistance and interface flux continuity. Add an independently prescribed contact resistance to test temperature jumps. It does not test advection or latent heat.

A uniform lumped body exchanging heat with a fixed reservoir has
\(T(t)=T_\infty+(T_i-T_\infty)\exp[-h_cAt/(mc)]\).
This is exact for the lumped ODE; small Biot number is a physical justification for treating a spatial solid this way. It is not an exact solution for a spatial body at arbitrary Biot number. This distinction also prevents Rheon's wall-relaxation formula from being mistaken for a conjugate solid heat solver.

## 47.3 Buoyant convection before evaporation

Start with uniform-temperature hydrostatic rest. A discretization should not generate a thermal plume when \(T=T_0\) and hydrostatic pressure balances gravity. In a Boussinesq perturbation formulation this means zero residual buoyancy and a consistent pressure reference. Then prescribe heating and verify the predicted buoyancy sign without conflating it with arbitrary authored force fields.

The classical differentially heated square cavity has no-slip walls, hot/cold opposite vertical walls, insulated horizontal walls, a declared gravity direction and property scales. The [de Vahl Davis 1983 primary record](https://doi.org/10.1002/fld.1650030305) verifies the two-dimensional benchmark identity and its \(Ra\) range. A future fixture must obtain and independently check the actual table, Prandtl number, normalizations and sample coordinates before adopting numerical targets; this audit inspected no table. Compare average/local wall Nusselt number and velocity extrema, not just streamline shape.

A horizontal heated layer offers Rayleigh–Bénard onset and conduction references, but the threshold depends on mechanical/thermal boundary conditions and horizontal domain. Do not quote one critical \(Ra\) without those conditions. Thermal cavity success establishes neither nucleation nor high-density-ratio gas motion. For Marangoni flow, a flat-interface imposed-gradient stress check precedes a drop-migration comparison; the [Young–Goldstein–Block original record](https://doi.org/10.1017/S0022112059000684) is provenance, with its detailed speed formula still requiring body verification.

## 47.4 Stefan problems: useful but scoped

An exact one-phase latent-heat surrogate has a motionless melted layer \(0<x<s(t)\), equal solid/liquid density \(\rho\), constant properties, initial solid at melting temperature \(T_m\), and fixed wall \(T_h>T_m\). Assume zero heat flux in the solid, no fluid motion and no surface energy. For \(t>0\),

\[
T(x,t)=T_h-(T_h-T_m)
\frac{\operatorname{erf}(x/(2\sqrt{\alpha t}))}{\operatorname{erf}(\lambda)},
\qquad s(t)=2\lambda\sqrt{\alpha t}, \tag{47.3}
\]
\[
\lambda e^{\lambda^2}\operatorname{erf}\lambda
=\frac{Ste}{\sqrt\pi},\qquad Ste=c(T_h-T_m)/L. \tag{47.4}
\]

**Conditional derivation:** the similarity variable reduces heat diffusion to \(F''+2\eta F'=0\). Its bounded integral is proportional to \(\operatorname{erf}\eta\). The two temperatures determine the constants; substituting \(\rho L\dot s=-kT_x(s^-)\) gives (47.4). The left side rises from zero to infinity for \(\lambda>0\), so each \(Ste>0\) has a unique positive root. The initial wall discontinuity gives an unbounded initial gradient; begin at a positive analytic time or treat the singular startup explicitly.

This is a **solid–liquid, equal-density mathematical surrogate**. It isolates energy/latent-front coupling and enthalpy inversion. It does not validate liquid–vapor volume production, gas pressure, species diffusion, capillarity or nucleation. Calling it a complete boiling test would be misleading.

Next use a planar two-phase vaporization/condensation problem with the actual density ratio, interface velocity and thermal advection. The [Palmore–Desjardins preprint's verification section](https://arxiv.org/pdf/1906.04565) and [Malan et al., §4](https://arxiv.org/pdf/2001.03477) provide primary benchmark families. Reproduce the selected paper's exact boundary conditions and property normalization before comparing. Report interface position, both phase masses, volume source, heat flux, energy closure and pressure/outlet response. Reverse heat driving for a separately constructed condensation case; do not claim the cited authors ran that new reversal.

## 47.5 Evaporating and condensing drops

For an isolated spherical liquid drop of radius \(R\), a classical quasi-steady diffusion model in a quiescent unbounded binary gas gives a useful \(d^2\) reference. Assume constant gas density/diffusivity, fixed uniform interface temperature and composition, fixed far-field vapor fraction, no forced/buoyant flow, negligible droplet recession effects in the gas (large liquid/gas density ratio), no curvature shift and no heat limitation. With Stefan flow and zero noncondensable flux,

\[
\dot m_{out}=4\pi R\rho_gD\log(1+B_M),\qquad
B_M=\frac{Y_s-Y_\infty}{1-Y_s},
\]
\[
\frac{d(d^2)}{dt}=-\frac{8\rho_gD}{\rho_l}\log(1+B_M),
\qquad d=2R. \tag{47.5}
\]

**Derivation:** steady radial total mass flow is constant; zero inert-species flux gives \(\rho_gD\,dY/dr=-(\dot m_{out}/4\pi r^2)(1-Y)\). Integrate from \(R,Y_s\) to infinity, then differentiate liquid mass \(4\pi\rho_lR^3/3\). The stated recession approximation is essential when interpreting a shrinking boundary against a stationary-gas quasi-steady field. Condensing \(Y_\infty>Y_s\) gives negative outward transfer and positive diameter-squared growth within this limited model.

This reference verifies diffusion, Stefan flow and inventory geometry. It does not validate droplet heat-up, strong vapor-density variation, natural/forced convection, nucleation, multicomponent activity or finite-domain vapor buildup. A 2D circle has different diffusion geometry; the spherical formula cannot be used by relabeling its radius. Subsequent nonisothermal drop cases should solve interface heat supply and equilibrium together and include latent cooling/heating. Shrinking-domain/interface area errors can mimic a mass-transfer error.

For finite reservoirs, vapor accumulation should eventually suppress evaporation or alter pressure. A permanently imposed dry far field removes vapor and carries enthalpy; it is an open experiment. Droplet condensation at high noncondensable fraction can be species-limited even when the wall is very cold. These cases probe exterior and mixture assumptions missing from a pure-vapor Stefan test.

## 47.6 Bubble growth, boiling and pressure transients

The [Plesset–Zwick 1954 primary record](https://doi.org/10.1063/1.1721668) identifies classical vapor-bubble growth in superheated liquid. Its full body was not inspected here; no unverified coefficient or universal \(R(t)\) formula is copied. The inspected [Malan benchmark section](https://arxiv.org/pdf/2001.03477) gives an accessible published numerical route for a seeded spherical thermal bubble and named comparison assumptions. Extract the exact nondimensional superheat definition before using it: Jakob number conventions can include a density ratio, while a Stefan number typically does not.

A spherical seeded bubble can test phase mass, thermal boundary layer, interface heat flux, symmetry and approximately diffusion-controlled growth. It does not test wall nucleation, active-site statistics, microlayer/contact-line models, bubble coalescence or critical heat flux. Radius agreement alone can conceal compensating heat-flux and interface-area errors. Report thermal profiles and energy transfer as well.

For inertial bubble tests, pressure, viscosity, capillarity and liquid inertia introduce separate scales. A Rayleigh–Plesset-type reference assumes spherical symmetry and an incompressible surrounding liquid; its thermal and gas law must be declared. Near collapse those assumptions can fail, making a compressible benchmark necessary. A reduced-stiffness particle liquid cannot be claimed to reproduce acoustic collapse without separate qualification.

Wall boiling requires conjugate wall heat capacity/conduction, a nucleation rule or specified initial seed, wetting/contact-line treatment, and resolution/modeling of thin films. The present manuscript does not select a universal wall-boiling correlation. An adiabatic flash endpoint test validates mass/energy/EOS equilibrium constraints; transient flashing additionally needs nucleation and relaxation evidence. A near-critical case is explicitly outside the first proposed implementation regime.

## 47.7 A concrete ladder with limits

| Level | Proposed evidence | Pass would support | Would still not support |
|---|---|---|---|
| A | property units, energy inverse, saturation equality, Clapeyron derivative vs consistent EOS | thermodynamic closure within sampled domain | transport accuracy, extrapolation |
| B | sinusoidal diffusion/advection, layered conductor, wall heat exchange | thermal operators, boundary signs, conserved inventory | buoyancy or phase change |
| C | hydrostatic rest and laminar thermal cavity with verified reference data | bounded single-phase buoyancy coupling | arbitrary variable-density/boiling behavior |
| D | equilibrium latent-energy cell, equal-density Stefan surrogate | latent storage/inversion and front energy | liquid–gas expansion or gas pressure |
| E | planar unequal-density phase change, both heating directions | jump signs, dilation, phase-energy coupling | curved geometry, nucleation |
| F | isothermal spherical diffusion drop, then nonisothermal drop | species/Stefan flow and latent heat feedback | forced turbulent sprays, wall boiling |
| G | seeded spherical bubble, capillary rest and thermocapillary stress | curved interface, capillarity, thermal growth in selected regime | wall nucleation or collapse shocks |
| H | sealed heated two-phase volume and vented reservoir comparison | global pressure/energy/volume compatibility | universal low-Mach adequacy |
| I | later carefully specified experiments with uncertainty and resolution studies | validation of selected physical closures | untested materials/pressure regimes |

Cancellation/rollback, source-stage identity and reproducible restart must be checked alongside numerical cases. These are custody tests, not scientific validation. Memory, timing, parallel scaling and hardware data require actual authorized measurements; no such results are supplied by this research.

## 47.8 Research conclusions and remaining decisions

The present CPU source supports conservative liquid-volume bookkeeping, MAC pressure infrastructure, bounded transactional stepping, and a distinct energy-conserving single-gas prototype. These are useful foundations. It has no complete coupled thermal phase-change solver, and the particle temperature field must not be represented as one. [The gap map](solver-gap-map.md) connects these conclusions to pinned source.

A realistic first thermal scope is a single-fluid energy inventory, conservative advection/conduction, explicit wall/reservoir exchange and a bounded Boussinesq study. The first phase-change scope should specify pure-fluid versus vapor–air, absolute pressure, density/property range, permitted boundary exchange and an already resolved interface. A sharp-interface low-speed grid comparison fits the existing inventory/MAC concepts, but still requires new variable-density/jump-aware pressure, energy/species and geometry work. Particle thermal evolution is a separate candidate rather than an interchangeable UI mode.

Strong pressure transients, wall nucleation, cavitation collapse, multicomponent chemistry and near-critical dynamics are separate research branches with different closure and validation needs. The central unresolved decisions are the target physical regime, exact energy definition, interface model, gas composition/pressure treatment, and error tolerances tied to intended outputs. These drafts inform future planning; they authorize no production change, publication, merge or deployment.
