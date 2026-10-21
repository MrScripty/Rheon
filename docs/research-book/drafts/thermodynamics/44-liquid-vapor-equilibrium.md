# 44 Liquid–vapor equilibrium and its limits

**Research draft for proposed Chapter 44; book author: Puma.** A liquid does not change phase simply because its temperature crosses one fixed number. Equilibrium depends on pressure, composition and curvature; the actual rate depends on transport and possibly interface kinetics. This chapter separates equilibrium, metastability and dynamics.

## 44.1 Coexistence is equality of chemical potential

Assume a pure substance, a planar interface at rest, local equilibrium, negligible external-field differences across its molecular thickness and no surface excess contributions to the bulk state. Thermal and mechanical equilibrium give equal \(T\) and \(p\). Transfer a positive mass \(dm\) from liquid to vapor at fixed common \(T,p\). The Gibbs free-energy change is

\[
dG=(g_v-g_l)\,dm. \tag{44.1}
\]

An unconstrained interior minimum with both phases present requires \(g_l=g_v\), and stability requires comparison with competing phases and branches. Solving this equality with a consistent EOS defines \(p_{sat}(T)\), or its inverse \(T_{sat}(p)\), on the liquid–vapor coexistence curve. A high local temperature alone does not specify \(dm/dt\). A locally equilibrated interface can sustain finite transfer in a macroscopic transport model even though its leading chemical-potential difference vanishes; finite resistance requires a nonequilibrium closure.

The [IAPWS-95 release, §4, Table 3 and saturation discussion](https://iapws.org/technical-guidance/release/IAPWS-95.download) supplies a consistent water phase-equilibrium authority. [IF97, §8](https://iapws.org/technical-guidance/release/IF97-Rev.download) supplies its industrial saturation-region formulation. Neither implies that Rheon's reference-state water preset already has saturation, latent heat or a two-phase caloric EOS.

## 44.2 Deriving Clapeyron and its approximation

Along a differentiable planar pure-fluid coexistence curve, differentiate \(g_v=g_l\) using \(dg=-s\,dT+v\,dp\). Thus

\[
(v_v-v_l)\,dp=(s_v-s_l)\,dT,
\qquad \frac{dp_{sat}}{dT}=\frac{L(T)}{T(v_v-v_l)},
\quad L=h_v-h_l=T(s_v-s_l). \tag{44.2}
\]

This conditional proof assumes two distinct equilibrium phases, common \(T,p\), nonzero volume difference and differentiability. For ordinary subcritical vaporization \(L>0\), \(v_v>v_l\), so saturation pressure rises with temperature. Units check: \((J/kg)/(K\,m^3/kg)=Pa/K\). Latent heat is an enthalpy difference; replacing it with \(e_v-e_l\) drops the pressure–volume work \(p(v_v-v_l)\).

If vapor is ideal, \(v_v=R_vT/p\); if \(v_l\ll v_v\), and if \(L\) can be regarded as constant over the interval, (44.2) reduces to

\[
\frac{d\log p_{sat}}{dT}=\frac{L}{R_vT^2},
\qquad \log\frac{p_{sat}(T)}{p_*}
=-\frac L{R_v}\left(\frac1T-\frac1{T_*}\right). \tag{44.3}
\]

The reference pair must be an actual coexistence state. Here \(R_v=R_u/M_v\), in J/(kg K). Molar latent heat requires \(R_u\). Variable latent heat instead requires an integral. Large pressure, nonideal vapor, significant liquid volume and the critical region invalidate this simple approximation. It is not an arbitrary empirical boiling curve to extrapolate everywhere.

For an ideal gas and nearly incompressible liquid with finite capacities, the latent enthalpy is not exactly constant with temperature. Along saturation,
\(dL/dT=\Delta c_p+\Delta[v(1-\beta T)]\,dp_{sat}/dT\).
This follows by subtracting each phase's (41.10). A constant-\(L\), constant-\(c_p\) approximation must therefore declare its restricted range rather than claiming global thermodynamic consistency.

## 44.3 Vapor mixed with a noncondensable gas

Evaporation into air occurs below the pure-fluid boiling temperature at the total gas pressure. In an ideal gas mixture, the vapor chemical potential depends on its partial pressure \(p_v=x_vp_g\), not just total pressure. For a planar pure liquid with negligible pressure correction, equilibrium gives \(p_v=p_{sat}(T_\Gamma)\). Thus

\[
x_{v,\Gamma}=p_{sat}(T_\Gamma)/p_g,
\qquad Y_{v,\Gamma}=
\frac{M_v x_{v,\Gamma}}{M_vx_{v,\Gamma}+M_a(1-x_{v,\Gamma})}. \tag{44.4}
\]

Mole fraction \(x_v\) and mass fraction \(Y_v\) differ unless molecular weights match. This closure assumes \(0\le p_{sat}\le p_g\), ideal gas mixing, pure liquid and no dissolved inert gas. If the ratio exceeds one, clipping it conceals a regime failure: the assumed gas mixture/interface state cannot satisfy these hypotheses. Condensation occurs when transport and thermal conditions supply vapor to a colder interface, not by negating a temperature switch.

At elevated pressure use fugacity equality. Liquid pressure changes introduce a Poynting correction; dissolved solute introduces activity; multicomponent liquids require chemical-potential equality for each species and an appropriate vapor–liquid equilibrium model. Henry's law for a dilute dissolved gas describes a different species limit from pure-liquid saturation. Relative humidity is a partial-pressure ratio under its defined reference, not a liquid volume fraction.

The [Scapin–Costa–Brandt primary preprint, §2, equations (7)–(10)](https://arxiv.org/pdf/1907.00890v2) explicitly separates gas vapor fraction, equilibrium and thermal coupling. Its model is a single-component liquid in an inert gas with declared approximations. It is evidence that these variables belong in a resolved evaporation model, not an endorsement of constant properties or Clausius–Clapeyron at every pressure.

## 44.4 Curvature, pressure and metastability

For a liquid sphere of radius \(R\), negligible motion and constant surface tension, \(p_l-p_v=2\sigma/R\). Chemical equilibrium is then \(g_l(T,p_l)=g_v(T,p_v)\) at unequal bulk pressures. Using an incompressible liquid, ideal vapor and a planar reference at the same temperature yields, to leading order in liquid pressure correction,

\[
\log\frac{p_v(R)}{p_{sat}(T)}\approx\frac{2\sigma v_l}{R R_vT}. \tag{44.5}
\]

**Derivation:** integrate \(dg_l=v_l dp_l\), integrate \(dg_v=R_vT\,d\log p_v\), equate changes, and approximate \(p_l-p_{sat}\) by \(2\sigma/R\). The neglected vapor-pressure shift in that last approximation must be small relative to the capillary liquid-pressure change. This Kelvin relation concerns a convex liquid drop; a vapor bubble has the opposite liquid-to-vapor curvature and must be derived with its own pressure traces. Molecular-size drops require surface physics beyond constant \(\sigma\) and a sharp continuum interface.

A superheated liquid may remain metastable because creating an interface costs free energy. In a classical spherical homogeneous nucleation illustration at prescribed driving pressure \(\Delta p>0\), neglecting elastic, thermal and molecular corrections,

\[
\Delta G(R)=4\pi R^2\sigma-\frac{4\pi}3R^3\Delta p,
\quad R_* = 2\sigma/\Delta p,
\quad \Delta G_* = 16\pi\sigma^3/(3\Delta p^2). \tag{44.6}
\]

Differentiation proves the stationary radius and barrier for this assumed functional. It does not prove a nucleation rate, waiting time, active-site density or boiling onset. Heterogeneous cavities, wetting, dissolved gas, wall roughness and history alter nucleation. A mesh cell that exceeds \(T_{sat}\) supplies none of those closures. Resolving an already seeded bubble tests growth, not spontaneous nucleation.

## 44.5 Distinct physical questions

**Evaporation** is interfacial liquid-to-vapor transfer, often limited by gas diffusion and heat supply. **Condensation** reverses that transfer and releases latent enthalpy; noncondensable gas can add substantial mass-transfer resistance. **Boiling** forms/grows vapor inside superheated liquid or at a heated wall; nucleation, contact-line physics and wall heat partition matter in addition to the interface energy jump.

**Flash evaporation** follows a pressure drop that makes the initial liquid state nonequilibrium with the new surroundings. A steady adiabatic throttling idealization conserves enthalpy, so at prescribed final pressure a homogeneous equilibrium mass quality can satisfy \(x=(h_{in}-h_l)/L\) if the state lies inside the two-phase region. This is an equilibrium endpoint with negligible kinetic/potential changes, not a predicted transient rate or bubble population. A sealed rigid adiabatic vessel instead conserves total internal energy and volume; using the same isenthalpic formula would be wrong.

**Cavitation** concerns vapor formation/growth under reduced liquid pressure, often with noncondensable gas and strong inertia. Collapse may involve shocks and large local heating. A barotropic pressure clamp or reduced particle sound speed does not establish a cavitation model. A thermal evaporation closure alone does not establish collapse dynamics.

## 44.6 The critical region is a different limit

As the critical point is approached, phase density contrast and latent heat vanish while response functions and transport behavior become strongly state dependent. A sharp interface with constant thickness negligible against the grid, constant \(L\), incompressible liquid and ideal vapor becomes a poor description. The ratio in (44.2) becomes a limiting thermodynamic problem; substituting zero numerator and denominator is not an evaluation rule.

Above critical coexistence there is no liquid–gas interface to move with a Stefan law. Numerical mixtures with a discontinuous phase label do not repair that missing physical distinction. IAPWS includes a critical-region treatment and uncertainty limits; even a reliable EOS does not by itself supply critical dynamics, molecular fluctuations or a well-resolved diffuse interface. The first Rheon phase-change research should explicitly remain away from that region.
