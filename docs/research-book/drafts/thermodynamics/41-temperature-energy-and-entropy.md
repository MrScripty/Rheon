# 41 Temperature, energy and entropy

**Research draft; proposed numbering, not published. Book author: Puma.** Read [the review guide](REVIEW.md) for scope and [the bibliography](bibliography.md) for inspected source limits. This chapter extends Chapters 2 and 26 without changing their published text. Every derivation below is original exposition under its stated hypotheses; none is a machine-checked theorem or evidence of an implemented thermal solver.

## 41.1 What temperature measures

Temperature is an intensive state variable. Internal energy is extensive: twice as much material at the same state has twice the energy. Specific internal energy \(e\) has units J/kg; internal-energy density \(\rho e\) has units J/m³. Heat is energy crossing a boundary because of a temperature difference, not a substance stored inside a particle. A fluid may warm through compression or viscous dissipation without receiving boundary heat.

Assume a locally equilibrated, simple compressible pure material, with fixed composition and no independent elastic, electrical or magnetic work. For specific entropy \(s\), specific volume \(v=1/\rho\), and differentiable equilibrium energy \(e(s,v)\), define

\[
T=\left(\frac{\partial e}{\partial s}\right)_v,
\qquad p=-\left(\frac{\partial e}{\partial v}\right)_s,
\qquad de=T\,ds-p\,dv. \tag{41.1}
\]

Here \(T>0\) is absolute temperature, in kelvin, \(s\) is J/(kg K), and \(p\) is absolute thermodynamic pressure, in Pa. A pressure multiplier in an incompressible calculation can be defined only up to an additive gauge; an equation of state cannot. The formula is a local equilibrium constitutive statement, not a description of a molecular interface far from equilibrium. Mixtures require composition terms.

Define enthalpy \(h=e+pv\), Helmholtz free energy \(a=e-Ts\), and Gibbs free energy \(g=h-Ts\). Direct differentiation of (41.1) gives

\[
dh=T\,ds+v\,dp,\qquad da=-s\,dT-p\,dv,
\qquad dg=-s\,dT+v\,dp. \tag{41.2}
\]

The same letter \(g\) often denotes gravitational acceleration; here gravity is \(\mathbf b\). For a pure substance, the mass-specific chemical potential is \(g\); molar chemical potential is \(M g\), with molar mass \(M\). Never mix molar and mass-specific latent heat in an exponential containing a gas constant.

An EOS is more than a pressure formula. A consistent \(a(T,\rho)\) provides

\[
s=-a_T,\quad e=a-Ta_T,\quad p=\rho^2a_\rho,
\quad c_v=-T a_{TT}. \tag{41.3}
\]

These follow by changing \(v\) to \(\rho\) in (41.2). The [IAPWS-95 release, §4 and Table 3](https://iapws.org/technical-guidance/release/IAPWS-95.download) is an authoritative pure-water example built from a Helmholtz potential; its saturation properties are calculated consistently with that potential. This is a model with a specified validity region and uncertainty, not a universal EOS for every Rheon preset.

## 41.2 The first law in a moving fluid

Work in a single smooth phase with density \(\rho>0\), velocity \(\mathbf u\), Cauchy stress \(\boldsymbol\sigma=-pI+\boldsymbol\tau\), body acceleration \(\mathbf b\), conductive heat flux \(\mathbf q\), and volumetric heating \(Q\). Adopt outward heat flux as positive. Let \(E=e+|\mathbf u|^2/2\), excluding gravitational potential for now. Local conservation is

\[
\partial_t\rho+\nabla\cdot(\rho\mathbf u)=0, \tag{41.4}
\]
\[
\partial_t(\rho\mathbf u)+\nabla\cdot(\rho\mathbf u\otimes\mathbf u)
=\nabla\cdot\boldsymbol\sigma+\rho\mathbf b, \tag{41.5}
\]
\[
\partial_t(\rho E)+\nabla\cdot(\rho E\mathbf u)
=\nabla\cdot(\boldsymbol\sigma\mathbf u-\mathbf q)
+\rho\mathbf b\cdot\mathbf u+Q. \tag{41.6}
\]

Stress is symmetric in this classical continuum. Its mechanical energy flux is therefore \(\boldsymbol\sigma\mathbf u\); equivalently total-energy advection includes \(p\mathbf u\), and viscous work is on the right. Integrating over a fixed volume makes each sign testable: outward \(\mathbf q\cdot\mathbf n>0\) cools the contents. If gravity derives from a stationary potential \(\Phi\), moving \(\rho\Phi\) into total energy removes the gravity work source, by mass conservation. Time-dependent external fields still do work.

**Conditional identity.** Dot (41.5) with velocity and use (41.4). The product rule gives the kinetic-energy equation with stress power \(-\boldsymbol\sigma:\nabla\mathbf u\) after separating its flux. Subtracting this from (41.6) gives

\[
\rho\frac{De}{Dt}=-p\nabla\cdot\mathbf u
+\boldsymbol\tau:\nabla\mathbf u-\nabla\cdot\mathbf q+Q. \tag{41.7}
\]

This proof requires differentiability. Across shocks or phase boundaries the conservative integral equations and jump relations replace this manipulation. A discretization that separately damps velocity and transports temperature does not inherit (41.7): it must account for the lost kinetic energy, boundary work and any intended heat reservoir.

For a three-dimensional isotropic Newtonian fluid, let \(D=(\nabla\mathbf u+\nabla\mathbf u^T)/2\), \(\theta=\nabla\cdot\mathbf u\), and \(D'=D-\theta I/3\). The closure

\[
\boldsymbol\tau=2\mu D'+\zeta\theta I,
\qquad \boldsymbol\tau:\nabla\mathbf u=2\mu|D'|^2+\zeta\theta^2\ge0 \tag{41.8}
\]

requires \(\mu,\zeta\ge0\); the skew part of the velocity gradient contracts to zero with symmetric stress. Nonnegative continuum dissipation does not prove a positive or stable explicit finite step. Artificial SPH viscosity, turbulence closures and physical viscosity also have different calibration meanings.

## 41.3 The second law is a constraint on closure

Under the hypotheses of (41.1), mass conservation gives \(Dv/Dt=v\theta\). Substitute (41.7) into the Gibbs relation, divide by positive temperature and use the product rule for \(\mathbf q/T\):

\[
\partial_t(\rho s)+\nabla\cdot\left(\rho s\mathbf u+\frac{\mathbf q}{T}\right)
=\frac Q T+\frac{\boldsymbol\tau:\nabla\mathbf u}{T}
-\frac{\mathbf q\cdot\nabla T}{T^2}. \tag{41.9}
\]

Fourier conduction \(\mathbf q=-k\nabla T\), \(k\ge0\), makes the last two terms nonnegative. Entropy can leave an open domain, so total entropy inside need not increase. The inequality concerns internal production after entropy flux and heat supply are included. Setting a temperature floor adds energy unless its exchange is explicitly recorded. An energy-conserving method can still violate positive temperature or the entropy inequality.

Mixtures add species diffusion, chemical potential driving forces and enthalpy transported by diffusing species. The entropy flux need not be just \(\mathbf q/T\). Heat and mass diffusion may have cross effects; an Onsager formulation constrains the transport matrix to yield nonnegative production. This draft's later binary Fick model deliberately omits Soret and Dufour effects and does not claim a complete multicomponent entropy theorem.

## 41.4 Temperature equations have hypotheses

Define \(c_p=(\partial h/\partial T)_p\), \(c_v=(\partial e/\partial T)_v\), thermal expansion \(\beta=(1/v)(\partial v/\partial T)_p\), and isothermal compressibility \(\kappa_T=-(1/v)(\partial v/\partial p)_T\). Maxwell symmetry applied to a smooth Gibbs potential yields

\[
dh=c_p\,dT+v(1-\beta T)\,dp,
\qquad c_p-c_v=\frac{T\beta^2}{\rho\kappa_T}. \tag{41.10}
\]

For example, \(ds=(c_p/T)dT-(\partial v/\partial T)_p dp\), obtained by differentiating \(s=-g_T\); substituting in \(dh=Tds+vdp\) proves the first identity. Combining that entropy differential with its constant-volume counterpart proves the second. These are equilibrium identities where derivatives exist and \(\kappa_T>0\); a constant-density idealization is a constrained limit, not permission to divide by zero compressibility.

Adding \(D(p/\rho)/Dt\) to (41.7) gives

\[
\rho\frac{Dh}{Dt}=\frac{Dp}{Dt}+\boldsymbol\tau:\nabla\mathbf u-\nabla\cdot\mathbf q+Q,
\]
\[
\rho c_p\frac{DT}{Dt}=\beta T\frac{Dp}{Dt}
+\boldsymbol\tau:\nabla\mathbf u+\nabla\cdot(k\nabla T)+Q. \tag{41.11}
\]

Only for negligible pressure heating and viscous heating, no species/phase transfer, and the specified heat capacity does this reduce to an advection–diffusion equation. Even then variable \(k\) belongs inside the divergence. An ideal gas has \(\beta T=1\); a liquid under strong compression can also require pressure work. Low Mach alone does not justify discarding the thermodynamic pressure derivative in a sealed heated vessel.

## 41.5 What must be stored and measured

A thermal state needs a consistent energy reference, composition, EOS branch and energy-to-temperature inverse. For constant heat capacity in one phase, \(e-e_* = c_v(T-T_*)\) is convenient. Using \(\rho c_pT\) as a conserved variable across arbitrary pressure changes confuses enthalpy and internal energy. Across phase change, separate phase offsets carry latent energy; they cannot be chosen independently after the fact.

Rheon's particle preview stores temperature and has wall-contact relaxation, while its ideal-gas Euler prototype stores conservative total energy. Neither fact supplies a coupled two-phase energy balance. [The gap map](solver-gap-map.md) identifies the exact inspected source. Before implementation, decide whether a proposed stage claims conserved total energy, an approximate low-Mach enthalpy budget, or merely prescribed temperature. Their acceptance tests are different.
