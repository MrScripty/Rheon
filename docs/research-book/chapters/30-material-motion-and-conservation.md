# 30 Material motion, connected meshes and conservation

This begins a principles-first FEM sequence ending in [Chapter 36](36-reference-fem-solver.md). It is research exposition, authored for Puma's book, not a claim that Rheon implements general Lagrangian FEM. [Chapter 11](11-particle-methods.md) owns particle transfers and [Chapter 29](29-liquid-solid-interaction.md) distinguishes material meshes from collision geometry. The [source map](../implementation/lagrangian-fem-sources-and-scope.md) records primary references, versions and evidence limits.

## What moves, and what is connected?

A material label identifies the same portion of matter over time. A spatial coordinate identifies a place that different matter may occupy. Following material can remove an explicit convection term, but it does not remove deformation, stress, incompressibility or numerical instability.

A connected material mesh partitions a body into elements. Shared nodes and bases define a continuous approximate motion; connectivity determines which elements exchange force. A surface triangle used only for ray intersection supplies neither interior volume nor a mechanical approximation space. A cloud of moving particles likewise does not specify FEM integration domains. PFEM periodically builds a connected finite-element mesh on moving nodes; the word particle describes those nodes' motion, not an SPH kernel discretization. The authors' [2013 PFEM account, sections 2–3](https://www.scipedia.com/public/Onate_et_al_2013a) makes this distinction explicit.

We use a three-dimensional reference body \(\Omega_0\subset\mathbb R^3\), material coordinate \(X\) in metres and physical time \(t\) in seconds. Until remeshing or fracture, assume a differentiable, globally one-to-one motion \(x=\chi(X,t)\) with positive determinant. Its image is \(\Omega_t\). Local positivity alone does not establish global injectivity: two distant elements can overlap without either inverting. Boundary and self-contact therefore remain separate checks.

## Motion map and gradients

Define displacement \(d=\chi-X\), material velocity \(V=\partial_t\chi|_X\), and spatial velocity \(v(x,t)=V(\chi^{-1}(x,t),t)\). Then

\[
 F=\nabla_X\chi=I+\nabla_Xd,\qquad J=\det F>0,
 \qquad \nabla_x v=(\nabla_X V)F^{-1}=L.
\]

Here \((\nabla_XV)_{iA}=\partial V_i/\partial X_A\); lower-case indices label spatial components, capitals material components, with repeated indices summed. \(F,J\) are dimensionless, \(L\) has units s\(^{-1}\). A differential material line maps as \(dx=F\,dX\). For a positively oriented reference volume, \(dv=J\,dV_0\).

For any sufficiently smooth spatial scalar \(a\), the chain rule gives

\[
 \frac{D a}{Dt}=\partial_t a|_x+v\cdot\nabla_xa
 =\partial_t[a(\chi(X,t),t)]|_X.
\]

Thus acceleration is \(\partial_t V|_X\), even though the same physical field has spatial form \(\partial_t v+v\cdot\nabla v\). A material derivative is not the derivative holding an arbitrary moving computational coordinate fixed; [Chapter 35](35-contact-coupling-and-ale.md) derives the ALE correction.

**Conditional proof, determinant evolution.** If \(F\) is differentiable and invertible, Jacobi's determinant identity and \(\dot F=\nabla_XV=LF\) yield

\[
 \dot J=J\,\mathrm{tr}(F^{-1}\dot F)
 =J\,\mathrm{tr}(L)=J\,\nabla_x\cdot v.
\]

This is an exact-real identity. Evaluating a determinant at two discrete timesteps does not prove the continuous identity, nor does an endpoint check prevent inversion between endpoints.

## Mass is attached to material

Let \(\rho_0(X)>0\) denote reference mass density, kg/m\(^3\), and \(\rho(x,t)\) current density. With no mass sources or phase exchange, conservation on every material subbody \(B_0\) states

\[
 \int_{\chi(B_0,t)}\rho\,dv=\int_{B_0}\rho_0\,dV_0.
\]

Changing variables and using arbitrary subbodies gives \(\rho(\chi,t)J=\rho_0\). Differentiation yields \(D\rho/Dt+\rho\,\nabla\cdot v=0\). An incompressible homogeneous fluid has constant material density, so \(J=1\) relative to its initial configuration and \(\nabla\cdot v=0\). A discretization imposing only selected moments of either condition does not impose both pointwise at finite timestep.

Reference element mass \(m_e=\int_{\Omega_{0,e}}\rho_0\,dV_0\) stays fixed while its volume changes. Recomputing density from \(\rho_0/J\) preserves this model; independently overwriting density and volume can violate it. Node masses are integration-dependent weights, not little physical balls. A consistent mass matrix represents kinetic energy using overlapping basis functions.

## Momentum, stress power and energy

Use tensile-positive Cauchy stress \(\sigma\), Pa, outward current normal \(n\), traction on the body \(t=\sigma n\), and body acceleration \(b\), m/s\(^2\). Conservation of momentum on a material body is

\[
 \frac{d}{dt}\int_{\Omega_t}\rho v\,dv
 =\int_{\partial\Omega_t}\sigma n\,da+\int_{\Omega_t}\rho b\,dv.
\]

With sufficient regularity this gives \(\rho Dv/Dt=\nabla_x\cdot\sigma+\rho b\). In the absence of couple stresses, angular momentum balance makes \(\sigma\) symmetric. Define first Piola stress \(P=J\sigma F^{-T}\). Then material momentum is

\[
 \rho_0\ddot\chi=\mathrm{Div}_X P+\rho_0b.
\]

[Chapter 31](31-stress-and-weak-forms.md) derives the traction transformation and variational form. Dotting momentum with velocity and integrating by parts gives the kinetic-energy rate

\[
 \frac{d}{dt}\frac12\int_{\Omega_0}\rho_0|V|^2dV_0
 =\mathcal P_{\mathrm{ext}}-\int_{\Omega_0}P:\dot F\,dV_0.
\]

The double contraction is \(A:B=\sum_{iA}A_{iA}B_{iA}\). For a hyperelastic solid \(P=\partial W/\partial F\), the last integral is the stored-energy rate, so kinetic plus elastic energy changes by external work. For an incompressible Newtonian fluid \(\sigma=-pI+2\eta D\), \(D=(L+L^T)/2\), dynamic viscosity \(\eta\) in Pa s, stress power is \(-p\nabla\cdot v+2\eta D:D\). Exact incompressibility removes pressure work and leaves nonnegative dissipation. These statements assume the stated constitutive law and compatible boundary work; they prove neither stability of every time integrator nor a discrete energy inequality.

For a compressible adiabatic fluid, specific internal energy \(e\), J/kg, obeys \(\rho_0\dot e=P:\dot F\), absent heat flow or sources. An equation of state closes pressure. [Dobrev, Kolev and Rieben (2012)](https://epubs.siam.org/doi/10.1137/120864672) and [Laghos v3.1](https://github.com/CEED/Laghos/blob/v3.1/README.md) are primary references for high-order compressible hydrodynamics, a different model from the incompressible branch developed here.

## Worked affine motions

Take \(\chi=(sX_1,X_2,X_3)\), \(s(t)>0\). Then \(F=\mathrm{diag}(s,1,1)\), \(J=s\), \(V=(\dot s X_1,0,0)\), \(\nabla\cdot v=\dot s/s\), and \(\rho=\rho_0/s\). A reference litre becomes \(s\) litres with unchanged mass. At \(s=2\) this is not incompressible merely because the mesh follows matter.

For simple shear \(\chi=(X_1+k(t)X_2,X_2,X_3)\), \(J=1\) exactly and \(L_{12}=\dot k\). Volume remains constant while elements become arbitrarily skew as \(|k|\) grows. Volume preservation therefore does not bound mesh quality. The symmetric rate has \(D_{12}=D_{21}=\dot k/2\), and fluid viscous power density is \(\eta\dot k^2\).

Finally let \(\chi=Q(t)X+c(t)\), \(Q^TQ=I\), \(\det Q=1\). It has \(J=1\) and \(D=0\). Rigid rotation must not create elastic strain or Newtonian viscous stress. Objectivity in the next chapter formalizes this requirement. These examples are symbolic hand calculations, not executed solver measurements.
