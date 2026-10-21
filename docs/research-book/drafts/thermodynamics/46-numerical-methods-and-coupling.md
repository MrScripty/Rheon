# 46 Numerical methods and thermal phase-change coupling

**Research draft for proposed Chapter 46; book author: Puma.** There is no single universally preferable phase-change solver. A method must match the intended pressure, density ratio, interface scale, gas composition and acoustic regime. This comparison is original synthesis; it specifies conservation obligations without pseudocode or copied implementation.

## 46.1 First choose the conserved state and approximation

A resolved sharp-interface model may carry liquid and gas masses, momentum, energy and gas species with one geometric interface. A low-Mach model commonly evolves sensible/total enthalpy with filtered acoustic pressure, whereas a compressible model evolves total energy including pressure work and waves. Define every neglected term before discussing a method's energy error. A numerically exact reduced enthalpy budget is not exact conservation of the full compressible/surface energy system.

For a cell with phase volume fractions \(\alpha_l+\alpha_v=1\), phase masses are \(m_k=\alpha_k\rho_kV\). A volume fraction is bounded geometry; density and energy are thermodynamic state. At equilibrium fixed pressure, vapor mass quality \(x=m_v/(m_l+m_v)\) satisfies

\[
h_{mix}=(1-x)h_l+xh_v,\qquad
\alpha_v=\frac{x/\rho_v}{(1-x)/\rho_l+x/\rho_v}. \tag{46.1}
\]

The second formula is just volume equals mass divided by density. A small vapor mass can occupy a large volume. It is incorrect to use \(x\) directly as a geometric fraction at high density ratio.

When total energy is evolved conservatively and its state inversion includes the phase-specific liquid/vapor reference offsets, latent storage is already represented. Adding another independent \(-jL\) sink then counts latent heat twice. Conservative mass exchange alone does not establish this energy constraint: phase conversion changes the internal-energy partition and the temperature/EOS inverse must satisfy the conserved energy. Conversely, a sensible-temperature equation without the reference offset needs the appropriate latent source. The chosen energy definition, phase fractions and heat source must be derived together, including the temperature dependence of enthalpy difference away from the reference saturation state.

## 46.2 Conservative sharp-interface VOF

VOF transports phase volume and reconstructs an interface inside mixed cells. Geometric PLIC uses a plane chosen to match the cell's represented volume with an estimated normal. Shared swept-face volumes can make phase advection conservative. With phase change, reconstruction displacement must also account for \(V_\Gamma=u_{l,n}-j/\rho_l\); simply transporting the fraction with a discontinuous single sampled velocity can create or lose mass.

Mass, momentum and energy fluxes should be consistent with the same phase face volumes and reconstructed states. On MAC grids momentum lives on dual control volumes: matching cell mass transport alone does not prove matching face-momentum transport. Temperature and gas species need one-sided interface values/gradients. A mixed-cell averaged temperature is generally not the correct trace for latent heat flux. Thin vapor films, near-empty phases, topology changes and the small-cut-cell timestep require explicit treatment.

[Malan et al., §§2–3](https://arxiv.org/pdf/2001.03477) provides a primary example of geometric VOF, a sharp velocity jump and consistent thermal-capacity fluxes on a MAC grid. This draft uses that paper only for this architectural comparison and its benchmark identities; it imports no PARIS code, coefficients, performance or numerical results. The article's reduced thermal model must not be described as a general full-energy scheme.

VOF's benefits are conservative phase inventory and natural topology change. Its costs are geometric reconstruction, flux bookkeeping, curvature accuracy and complicated coupled stencils at solids. Boundedness requires a proved or tested update under actual directional splitting and sources. Unconditionally clipping \(\alpha\) back into \([0,1]\) conceals a mass defect. Redistributing excess can preserve total volume but still move the interface and alter energy unless all affected fields follow the same redistribution.

## 46.3 Level set, conservative level set and ghost fluid

A signed distance \(\phi\) represents the interface through \(\phi=0\); normals and curvature follow its derivatives where regular. The kinematic equation uses interface velocity, not necessarily the liquid or vapor velocity. Reinitialization restores distance-like structure but can move the zero set or change enclosed mass. A conservative level-set/color formulation changes the transported variable and may use interface compression; the name alone does not prove sharpness, phase mass conservation or energy consistency.

Ghost-fluid methods impose discontinuous traces or jump corrections in stencils across an interface. They can preserve sharp temperature, species or pressure conditions without making every mixed cell a homogeneous material. They are an interface-condition treatment and can be paired with VOF, level set or front tracking. Ghost values do not themselves conserve phase mass. Curvature, interface placement, jump signs and compatibility of the pressure gradient with its divergence control spurious currents.

Hybrid coupled level-set/VOF methods use a conservative fraction for inventory and a distance field for geometry. They must reconcile the two descriptions after transfer and phase change; otherwise heat flux sees one interface and mass loss sees another. A geometric front-tracking mesh gives explicit interface area and movement but requires remeshing/topology operations and conservative transfer. None of these names eliminates the need for the same interfacial mass, momentum and energy budget.

## 46.4 Diffuse-interface and phase-field models

A phase field gives the interface a finite thickness and derives capillarity from a free-energy functional. For a restricted isothermal illustration on a fixed domain, use

\[
\mathcal F[\phi]=\int_\Omega f_0\left(W(\phi)+\tfrac12\epsilon^2|\nabla\phi|^2\right)dV,
\quad \mu_\phi=f_0\left(W'(\phi)-\epsilon^2\nabla^2\phi\right),
\quad \partial_t\phi=\nabla\cdot(M\nabla\mu_\phi). \tag{46.2}
\]

Here \(\phi,W\) are dimensionless, \(\epsilon\) is a length, and the constant \(f_0>0\) has units J/m³. The variational potential \(\mu_\phi\) has units J/m³; it is not the mass-specific chemical potential in Chapter 44. Mobility \(M\) has units m⁵/(J s). With smooth fields, constant \(\epsilon,f_0\), nonnegative mobility, and zero normal flux of \(\phi\)'s gradient and chemical potential, integration by parts proves

\[
\frac{d\mathcal F}{dt}=-\int_\Omega M|\nabla\mu_\phi|^2dV\le0,
\qquad \frac d{dt}\int_\Omega\phi\,dV=0. \tag{46.3}
\]

This is an independent conditional proof for the stated stationary isothermal Cahn–Hilliard system. It does not prove a maximum principle, liquid–vapor latent-heat model, unequal-density momentum balance, or a fully discrete method. Allen–Cahn relaxation has a different conservation contract. Mobility-driven order-parameter diffusion is not automatically the physical evaporation flux.

The [Abels–Garcke–Grün primary preprint, §§2–3 and sharp-interface analysis](https://arxiv.org/pdf/1104.1336) shows why unmatched-density thermodynamics and diffusive momentum flux need a consistent formulation. It concerns incompressible constituent mixtures; it is not automatically a thermal liquid–vapor boiling law. [Guo–Lin's primary abstract](https://doi.org/10.1017/jfm.2014.696) identifies a thermodynamically constructed nonisothermal thermocapillary phase-field model; its body was not inspected, and no detailed scheme or proof is adopted here.

Diffuse models make topology changes convenient and can be useful near complex interfaces. They require calibration of thickness, mobility, surface tension and thermal free energy. The Cahn number \(Cn=\epsilon/L\) and mobility scaling are part of the modeled limit. Holding interface width at a few grid cells while refining changes the physical/numerical model unless the sharp-interface limit is controlled. Explicit fourth-order Cahn–Hilliard terms can impose a timestep scaling of order \(\Delta x^4/(M f_0\epsilon^2)\) in the simple constant-coefficient model. Energy-stable implicit discretization need not preserve positive temperature or bounded fractions.

## 46.5 Enthalpy and apparent-capacity methods: proper scope

An enthalpy formulation stores sensible plus latent energy and inverts an equilibrium state relation. At a prescribed pressure, a pure substance in a locally equilibrated two-phase interval can have fixed saturation temperature while quality changes. An apparent heat capacity spreads the latent jump over a finite temperature interval; that interval is regularization or a physical mixture range that must be identified.

These methods are particularly natural for conduction-controlled melting/solidification with small density change. The [Voller–Prakash 1987 primary record](https://doi.org/10.1016/0017-9310(87)90317-6) explicitly concerns convection–diffusion and a mushy region. Its porous resistance interpretation should not be transplanted to a liquid–vapor interface. Liquid–gas phase change additionally needs gas displacement/compression, phase momentum, species exchange where present, interfacial area and pressure-dependent equilibrium. An enthalpy variable is useful there, but a smoothed heat-capacity equation alone is incomplete.

At fixed pressure an equilibrium enthalpy inverse gives \(x=(h-h_l)/L\) only inside the coexistence interval. Outside it, use the appropriate single-phase branch; do not clip quality while discarding excess energy. At fixed volume, solve the coupled internal-energy, mass and volume constraints, not the fixed-pressure inverse. An equilibrium cell mixture also assumes subcell relaxation is fast; it does not resolve a nucleation barrier or two different phase temperatures.

## 46.6 Low Mach and compressible strategies

For a weakly stratified low-Mach gas write \(p=p_0(t)+\pi\), with dynamic \(\pi/p_0=O(Ma^2)\). EOS and saturation use the appropriate absolute thermodynamic pressure, including modeled hydrostatic/capillary effects when significant, while \(\pi\) enforces a velocity constraint. This filtering removes fast acoustics. With ideal mixture \(\rho=p_0/[R(Y)T]\), smooth continuity gives

\[
\nabla\cdot\mathbf u=\frac1T\frac{DT}{Dt}
-\frac1{p_0}\frac{dp_0}{dt}+\frac1R\frac{DR}{Dt}. \tag{46.4}
\]

This is a particular EOS derivative, not the entire low-Mach closure. A sealed domain determines \(p_0(t)\) through global compatibility and energy; an open prescribed-pressure reservoir supplies different constraints. Strong hydrostatic stratification needs a suitable spatial base state, not this spatially uniform ansatz. Interfacial dilation is added consistently with the two-phase equations, not inserted twice into an already averaged mixture derivative.

Constant-density phase-change projection is a narrower low-speed approximation than variable-density low Mach. It can be useful for vented heat-driven bubble growth away from acoustics but cannot generally represent net boiling inside a sealed rigid volume. At high Mach, strong flash transients, collapse shocks or acoustically important pressure feedback, compressible multiphase methods may be required. They need a caloric EOS for each phase and compatible interface relaxation or jump conditions, along with conservative total energy, positivity and acoustic CFL control.

A mixed cell containing unlike EOSs cannot be closed by arbitrarily averaging pressure or ratio of specific heats. Conservation of cell energy does not alone preserve a uniform-pressure material contact. Mixture-equilibrium, non-equilibrium multi-pressure or sharp-interface Riemann strategies make different assumptions about pressure, temperature and relaxation. Relaxation source terms can be stiff and must preserve admissible states and the correct equilibrium limit. Rheon's existing periodic ideal-gas LLF Euler prototype is useful evidence for single-gas conservative state handling; it supplies none of this multiphase closure.

The [Palmore–Desjardins primary preprint](https://arxiv.org/pdf/1906.04565) is a pertinent low-Mach interface-resolved comparison. A newer [Dodd–Trefftz-Posada–Ferrante 2025 paper, §§2–3](https://doddm.com/publications/2025-jcp-md-pt-af.pdf) studies constant-density liquid/bicomponent-gas phase change with VOF and pressure correction. Its phase-velocity extrapolation illustrates the continuing distinction between interface-advection velocity and physical divergent velocity. This is a bounded architectural comparison, not adopted code or independently reproduced performance. The publisher body returned HTTP 403; the authors' separate public copy was readable, with that access history retained in the audit.

## 46.7 Particle approaches

SPH can carry parcel mass, internal energy/enthalpy, temperature and composition. Pairwise conductive transfers with one nonnegative conductance and equal/opposite energy exchange give the graph conservation and entropy arguments in Chapter 42 for fixed masses/capacities. Consistent pressure work and viscous heating are still required. Boundary-truncated kernels, discontinuous conductivities, disorder and changing neighbor graphs require their own analysis. The [Cleary–Monaghan primary paper record](https://doi.org/10.1006/jcph.1998.6118) is conduction provenance; its body was not read, so no kernel or numerical result is copied.

Changing a particle's phase label without reconciling energy, density, volume and neighbor support creates an inconsistent thermodynamic state. Fixed parcel mass with changing density changes represented volume; particle deletion or splitting requires mass, momentum, energy, composition and identity ledgers. A parcel in vacuum is not a continuum vapor mixture with finite pressure. Weakly compressible SPH with artificially small liquid sound speed changes pressure/expansion dynamics; boiling and cavitation claims cannot use that speed as measured acoustics.

PIC/FLIP/APIC can transport energy with particles and perform conduction/pressure on a grid. Thermal transfers need consistent mass weighting, partition of unity, admissible energy inversion, and source accounting during reseeding. Velocity FLIP does not automatically conserve thermal energy. MPM/PFEM add material or mesh transfer choices with similar obligations. Rheon currently implements a bounded SPH preview, not these full alternatives.

## 46.8 Coupling and timestep restrictions

| Constraint | Representative scale or obligation | Qualification limit |
|---|---|---|
| advection | \(\Delta t\sum_d|u_d|/\Delta x_d\lesssim C\) | method/stencil-specific; interface speed also matters |
| thermal diffusion | \(\Delta t\lesssim[2\alpha\sum_d1/\Delta x_d^2]^{-1}\) for simple explicit centered diffusion | implicit stability does not guarantee accuracy |
| compressible acoustics | \(\Delta t\sum_d(|u_d|+c_s)/\Delta x_d\lesssim C\) | EOS/flux-specific admissibility proof needed |
| explicit capillarity | order \(\sqrt{\rho_{eff}\Delta x^3/\sigma}\) | coefficient, density choice and mode analysis depend on scheme |
| interfacial depletion | transferred mass \(|j|A_\Gamma\Delta t\) cannot exceed available donor-phase mass | flux limiting must adjust latent/momentum exchange consistently |
| interface displacement | \(|V_\Gamma|\Delta t\) small enough for chosen reconstruction/swept geometry | tiny cut volumes can be more restrictive |
| phase relaxation | resolve kinetic/thermal relaxation or use a consistent implicit/asymptotic-preserving treatment | saturation projection alone is not such a proof |

The symbols \(\lesssim C\) describe scaling, not established Rheon acceptance constants. Implicit heat diffusion can leave an explicitly coupled latent source unstable. Subcycling phase transfer requires matching accumulated mass, momentum, energy and pressure volume sources. A failed EOS inverse or negative donor mass should reject an uncommitted candidate; silently flooring it changes physics. Transactional state ownership in Rheon is a useful existing pattern, but a successful transaction does not prove physical conservation.

## 46.9 Comparison for the likely research progression

| Candidate | Strong reason to study it | Main unresolved cost for Rheon |
|---|---|---|
| conservative VOF + one-sided heat/species + jump-aware MAC pressure | closest conceptual fit to voxel inventory and MAC velocities | genuine two-phase inertia, geometry, energy fluxes and dilation all remain new |
| level set + ghost fluid, optionally VOF inventory | sharp traces and accessible geometry derivatives | phase mass/reinitialization reconciliation and conservative energy |
| thermodynamic phase field | topology and a common free-energy framework | thickness/mobility calibration, unequal density, thermal closure, stiffness |
| equilibrium enthalpy | transparent latent-energy accounting and conduction benchmarks | insufficient alone for moving liquid–gas volume and gas/species dynamics |
| variable-density low Mach | heat/diffusion-driven flows without acoustic cost | background pressure, compatibility, interface jumps and mixture energy |
| compressible multiphase | acoustics, strong pressure transients and collapse | multiple EOS/contact consistency, relaxation and much stricter timesteps |
| thermal particle method | material transport and sparse occupied regions | energy equation, phase geometry, conservative transfer and pressure calibration |

The evidence currently favors studying a conservative grid thermal foundation and then a deliberately narrow sharp-interface low-speed problem. That is a research conclusion about architectural fit, not an implementation authorization or a claim that VOF wins every regime.
