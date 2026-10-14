# 26 Compressible state, thermodynamics and waves

Compressibility introduces an evolving density and a thermodynamic energy equation. Pressure now follows a constitutive law and propagates through acoustic waves. The incompressible projection in Chapter 4 does not supply these dynamics. These three chapters specify a research path for a compressible branch; they do not establish a production compressible capability in Rheon.

## A state that conserves the right quantities

For a single continuum in three dimensions, use the conservative state

\[
U=(\rho,\mathbf m,\mathcal E)^T,\qquad
\mathbf m=\rho\mathbf u,\qquad
\mathcal E=\rho e+\tfrac12\rho|\mathbf u|^2.
\]

Here \(e\) is specific internal energy in J/kg; \(\mathcal E\) is total energy **per volume**, in J/m³. Density has units kg/m³ and momentum density kg/(m² s). A reader comparing papers must check their notation: many authors write \(\rho E\) for what this chapter calls \(\mathcal E\). Storing a pressure or temperature tracer instead of total energy changes the conservation contract.

For an outward unit normal \(\mathbf n\), let \(u_n=\mathbf u\cdot\mathbf n\). The Euler normal flux is

\[
F_n(U)=\left(\rho u_n,\ \mathbf m u_n+p\mathbf n,\ (\mathcal E+p)u_n\right)^T,
\qquad \partial_tU+\nabla\cdot F(U)=0.
\]

Pressure transports momentum and performs work; the energy flux therefore contains \(p u_n\). In a finite volume, a face transports one mass, one vector momentum and one energy flux. Computing them from different advecting states can conserve each array globally while breaking the intended thermodynamic coupling. NASA's governing-equation overview identifies the coupled continuity, momentum and energy system; the ALE paper by Benoit and Nadarajah makes the conservative state and ideal-gas pressure explicit. [NASA equations](https://www.grc.nasa.gov/www/k-12/airplane/nseqs.html), [ALE formulation, §2.1](https://arxiv.org/html/1804.03731)

## Close the equations before choosing a solver

A calorically perfect ideal gas assumes constant \(c_v\), \(c_p\) and \(\gamma=c_p/c_v>1\), with \(R=c_p-c_v\). Its closure is

\[
p=(\gamma-1)\left(\mathcal E-\frac{|\mathbf m|^2}{2\rho}\right),\qquad
e=c_vT,\qquad p=\rho RT,\qquad c^2=\gamma p/\rho.
\]

The admissible set has \(\rho>0\) and positive internal-energy density \(\mathcal E-|\mathbf m|^2/(2\rho)>0\). Absolute pressure enters the EOS. A plotting offset called gauge pressure must not replace it. Values such as \(\gamma=1.4\) define a teaching gas; constant heat capacities exclude temperature-dependent excitation, dissociation, ionization and chemical reaction. A thermally perfect gas can retain \(p=\rho RT\) while requiring temperature-dependent internal energy. NACA Report 1135 treats both perfect and imperfect gases, so its perfect-gas shock tables must be selected deliberately. [NACA Report 1135](https://ntrs.nasa.gov/citations/19930091059)

For a general single-material EOS \(p=p(\rho,e)\), the isentropic acoustic derivative follows from \(de=p\,d\rho/\rho^2\):

\[
c^2=\left.\frac{\partial p}{\partial\rho}\right|_e
+\left.\frac{\partial p}{\partial e}\right|_\rho\frac{p}{\rho^2}.
\]

The implementation must provide this derivative, a valid thermodynamic domain and consistent temperature/energy conversions. A negative \(c^2\), an EOS inversion outside its domain or an undefined heat capacity is a material-model failure, not a sound-speed value to clamp silently. Wang and Li derive EOS-dependent thermodynamics and a local stiffened-gas approximation; their approximation is a numerical strategy for a declared EOS, not a universal liquid calibration. [Wang and Li, §§1–3](https://arxiv.org/html/2108.13780)

One simple stiffened-gas convention is

\[
p=(\gamma_s-1)\rho e-\gamma_s p_\infty,\qquad
c^2=\gamma_s(p+p_\infty)/\rho.
\]

It reduces to an ideal gas when \(p_\infty=0\). A compatible constant-\(c_v\) caloric relation is \(e=c_vT+p_\infty/\rho\); other conventions include an energy offset and must be documented with their inversion. Admissibility requires \(\rho>0\), \(p+p_\infty>0\) and the chosen caloric domain. Reusing the ideal-gas condition \(p>0\) without reviewing this EOS changes its domain. Parameters must be fitted over the intended pressure and temperature range. A single-phase fit does not introduce cavitation, latent heat or a liquid-vapour interface model. The shock-compression experiments of Prasanna and colleagues illustrate why a liquid EOS fit needs a named material, data range and uncertainty. Their perfluorohexane parameters are not water parameters. [Liquid EOS experiment, §§II–IV](https://arxiv.org/html/2503.21306)

A Tait-type barotropic law instead prescribes pressure from density, for example

\[
p(\rho)=p_0+B\left[(\rho/\rho_0)^n-1\right],\qquad
c^2=\frac{nB}{\rho_0}(\rho/\rho_0)^{n-1},\qquad B>0,\ n>0.
\]

These equations define this chapter's convention; some sources absorb \(p_0\) into the pressure offset. It can represent a specified compressibility range. In weakly compressible numerical models, artificially reducing \(c\) to enlarge the time step changes acoustic travel times and compressibility. This is a modelling choice, not calibrated acoustics. A barotropic pressure law alone supplies no temperature dependence or shock heating. For smooth isentropic motion one may construct an energy with \(de/d\rho=p(\rho)/\rho^2\); that restricted mechanical energy does not close the general thermal Euler/Navier–Stokes system through shocks. Select a complete caloric EOS before claiming energy-resolved liquid shocks.

## Acoustic waves, contacts and shocks

In direction \(\mathbf n\), the three-dimensional Euler eigenvalues are

\[
u_n-c,\quad u_n,\quad u_n,\quad u_n,\quad u_n+c.
\]

The middle fields carry entropy and the two tangential velocity components. Acoustic fields travel relative to the fluid at \(\pm c\). A contact can have different density and temperature on either side while pressure and normal velocity agree. A pressure pulse and an advected density contact therefore exercise different mechanisms. The hyperbolic wave structure is set by the EOS; the ideal-gas formulas cannot be transferred unchanged to arbitrary real materials. [Wang and Li, §2](https://arxiv.org/html/2108.13780)

Linearize about uniform rest \((\rho_0,p_0,\mathbf u=0)\) with \(c_0^2=\gamma p_0/\rho_0\). Along \(x\), the acoustic subsystem is

\[
\partial_t p'+\rho_0c_0^2\partial_xu'=0,\qquad
\partial_tu'+\rho_0^{-1}\partial_xp'=0.
\]

Thus \(p'=f(x-c_0t)\), \(u'=p'/(\rho_0c_0)\), \(\rho'=p'/c_0^2\) is a right-travelling linear wave. Its impedance is \(Z=\rho_0c_0\). The nonlinear Euler system only approaches this reference as amplitude tends to zero; Chapter 28 specifies how to separate the linearization error from numerical error.

Across a planar discontinuity moving at normal speed \(s\), integrate conservation over a shrinking space-time control volume:

\[
s[U]=[F_n(U)],\qquad [a]=a_R-a_L.
\]

These Rankine–Hugoniot conditions constrain shock speed and both states. Conservation is necessary but does not select the physical branch. For an ideal gas use entropy relative to fixed reference values,

\[
\mathsf s-\mathsf s_{ref}
=c_v\log\left[(p/p_{ref})(\rho_{ref}/\rho)^\gamma\right].
\]

An admissible adiabatic shock increases specific entropy along the material crossing direction. Smooth inviscid motion transports it. The weak inequality is \(\partial_t(\rho\mathsf s)+\nabla\cdot(\rho\mathsf s\mathbf u)\ge0\). Mathematical entropy commonly uses the opposite sign, such as \(\eta=-\rho\mathsf s\), so its inequality is nonpositive. Always state the sign convention before plotting an entropy residual. NASA's normal-shock reference explains the increase of entropy and loss of stagnation pressure across a calorically perfect shock. [NASA normal shocks](https://www.grc.nasa.gov/www/k-12/airplane/normal.html)

For a stationary normal shock with upstream Mach \(M_1>1\), ideal-gas conservation gives

\[
\frac{\rho_2}{\rho_1}=\frac{(\gamma+1)M_1^2}{(\gamma-1)M_1^2+2},\qquad
\frac{p_2}{p_1}=1+\frac{2\gamma}{\gamma+1}(M_1^2-1).
\]

With \(\gamma=1.4\), \(M_1=2\), these ratios are \(8/3\) and \(4.5\). The temperature ratio is \(27/16\), and \(M_2^2=1/3\). These derived numbers are a local jump reference, not an executed Rheon shock calculation. Resolving a numerical shock over several cells does not reproduce its physical molecular thickness.

## Viscous stress and heat are part of energy accounting

For a compressible Newtonian fluid let \(D=(\nabla\mathbf u+\nabla\mathbf u^T)/2\), dynamic shear viscosity \(\mu\ge0\), bulk viscosity \(\zeta\ge0\), and conductivity \(k\ge0\). Use

\[
\tau=2\mu\left(D-\tfrac13(\nabla\cdot\mathbf u)I\right)
+\zeta(\nabla\cdot\mathbf u)I,\qquad \mathbf q=-k\nabla T.
\]

Stokes' hypothesis sets \(\zeta=0\); it is an assumption about bulk dissipation, not the condition \(\nabla\cdot\mathbf u=0\). With body acceleration \(\mathbf b\) and volumetric heat source \(Q\), the equations become

\[
\partial_t\rho+\nabla\cdot(\rho\mathbf u)=0,
\]
\[
\partial_t\mathbf m+\nabla\cdot(\mathbf m\otimes\mathbf u+pI)
=\nabla\cdot\tau+\rho\mathbf b,
\]
\[
\partial_t\mathcal E+\nabla\cdot[(\mathcal E+p)\mathbf u]
=\nabla\cdot(\tau\mathbf u-\mathbf q)+\rho\mathbf b\cdot\mathbf u+Q.
\]

Viscous work converts kinetic energy into internal energy while total energy changes through boundary work and heat. Adding a velocity diffusion step without its compatible energy term misses this conversion. For \(T>0\), the smooth entropy balance has production \(\tau:D/T+k|\nabla T|^2/T^2\ge0\), with heat entropy flux \(\mathbf q/T\). Numerical shock dissipation, physical viscosity and turbulent closure remain three distinct contributors. The original curvilinear Navier–Stokes study of Nolasco and colleagues explicitly constructs compatible viscous and entropy terms; its particular high-order discretization is not incorporated here. [Nolasco et al., §4](https://arxiv.org/html/1911.03682)

The next chapter constructs conservative numerical fluxes. Chapter 28 adds boundaries and a verification ladder. Any material or solver claim must name the EOS, transport coefficients, boundary regime and evidence actually obtained.
