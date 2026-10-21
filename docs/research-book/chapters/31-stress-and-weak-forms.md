# 31 Stress, constitutive laws and weak forms

[Chapter 30](30-material-motion-and-conservation.md) fixes coordinates, units and signs. This chapter turns conservation into an equation that finite elements can integrate. The independently derived formulas below use the conventions of that chapter; the [Sierra/SM 5.30 theory manual](https://www.sandia.gov/files/sierra/SM_Theory_5_30/index.html), chapters 3–9, supplies a primary technical reference. A paper using pressure positive in tension requires a sign conversion before comparison.

## One force, several stress measures

A stress tensor depends on the area and coordinate frame to which it refers. Nanson's formula maps an oriented reference area with outward normal \(N\):

\[
 n\,da=JF^{-T}N\,dA_0=\operatorname{Cof}(F)N\,dA_0.
\]

One way to derive it is to map two tangent vectors and use \((Fa)\times(Fb)=JF^{-T}(a\times b)\). Multiplying by \(\sigma\) gives force equality \(\sigma n\,da=PN\,dA_0\) and hence

\[
 P=J\sigma F^{-T},\qquad S=F^{-1}P,
 \qquad \sigma=J^{-1}PF^T=J^{-1}FSF^T.
\]

The first Piola tensor \(P\) is generally nonsymmetric; the second Piola tensor \(S\) is symmetric when \(\sigma\) is symmetric. Both have pressure units, but they refer to different area/force mappings. Using a Cauchy traction with reference area without the cofactor changes the force.

Green strain is \(E=(F^TF-I)/2\). If a solid energy is \(W(E)\) per reference volume, \(S=\partial W/\partial E\), \(P=FS\) and \(P:\dot F=S:\dot E\). Also \(P:\dot F=J\sigma:L\). These power identities explain the work-conjugate pairs rather than declaring tensors interchangeable.

## Objectivity and constitutive choice

A superposed rigid observer motion has \(F^*=QF\) and \(\sigma^*=Q\sigma Q^T\). Frame indifference requires \(W(QF)=W(F)\); an isotropic energy depending on invariants of \(C=F^TF\) satisfies it. Since \(Q^TQ=I\), \(C^*=C\), proving invariance for such a law. Isotropy further concerns rotation of material directions, \(F\mapsto FQ\); an objective fibre-reinforced law need not be isotropic.

For fluids, \(L^*=\dot Q Q^T+QLQ^T\). The first term is skew, so \(D^*=QDQ^T\). This establishes objectivity of isotropic Newtonian viscous stress. Integrating an ordinary componentwise stress rate through a finite rotation generally fails this test. Hypoelastic/plastic implementations need an appropriate objective rate or a finite-strain internal-variable formulation and a consistent algorithmic tangent. Those are distinct constitutive models, not automatic consequences of updating the mesh.

Select a three-dimensional compressible neo-Hookean teaching law, \(J>0\), \(\mu>0\), \(\lambda\ge0\), both moduli in Pa:

\[
 W(F)=\frac{\mu}{2}(F:F-3)-\mu\ln J
       +\frac{\lambda}{2}(\ln J)^2.
\]

This precise variant is also used in the [DOLFINx v0.9.0 hyperelasticity form](https://github.com/FEniCS/dolfinx/blob/v0.9.0/cpp/demo/hyperelasticity/hyperelasticity.py). Other models called neo-Hookean use different volumetric energies; their stresses and tangents differ. Our reference algorithm is derived from this specified energy, not copied from that implementation.

Let \(H=F^{-T}\), \(a=\lambda\ln J-\mu\). Differentiating \(\delta\ln J=H:\delta F\) and \(\delta H=-H\delta F^TH\) gives

\[
 P=\mu F+aH,
 \quad \mathcal A[\delta F]=\mu\delta F
       +\lambda(H:\delta F)H-aH\delta F^TH.
\]

In indices,

\[
 \mathcal A_{iAjB}=\mu\delta_{ij}\delta_{AB}
 +\lambda H_{iA}H_{jB}-aH_{iB}H_{jA}.
\]

This is \(\partial P/\partial F\) at fixed reference state. At \(F=I\), \(P=0\) and \(\mathcal A[H_0]=\mu(H_0+H_0^T)+\lambda\operatorname{tr}(H_0)I\), recovering linear elasticity. Here \(H_0\) denotes an arbitrary increment, not the inverse tensor \(H\). Major symmetry follows from a twice differentiable scalar energy; positive definiteness does not follow at arbitrary deformation. The law is illustrative, not calibrated for any Rheon material. For small-strain isotropic parameters, \(\mu=E_Y/[2(1+\nu)]\), \(\lambda=E_Y\nu/[(1+\nu)(1-2\nu)]\), with \(-1<\nu<1/2\); the chosen \(\lambda\ge0\) narrows this example to \(\nu\ge0\).

A Newtonian fluid instead uses \(\sigma=-pI+2\eta D\) under incompressibility. It has no persistent shear energy \(W(F)\); after flow stops its shear stress vanishes. A compressible viscous model can use \(2\eta\operatorname{dev}D+\zeta\operatorname{tr}(D)I\), \(\eta,\zeta\ge0\), plus an equation-of-state pressure and thermal energy. Replacing solid shear modulus with viscosity in a strain formula mixes Pa with Pa s and changes the physics.

## From strong to weak form

Partition the reference boundary into essential \(\Gamma_{D,0}\) and natural \(\Gamma_{N,0}\) portions, disjoint except at edges. Prescribe \(d=\bar d\) on the first and reference traction \(PN=\bar T\), N/m\(^2\) of reference area, on the second. Given \(d_0,V_0\), find motion satisfying material momentum. For an admissible test displacement \(w\), vanishing on \(\Gamma_{D,0}\), multiply momentum, integrate, and apply the divergence theorem:

\[
 \int_{\Omega_0}\rho_0w\cdot\ddot d\,dV_0
 +\int_{\Omega_0}\nabla_Xw:P\,dV_0
 =\int_{\Omega_0}\rho_0w\cdot b\,dV_0
 +\int_{\Gamma_{N,0}}w\cdot\bar T\,dA_0.
\]

The plus internal-force sign follows from integration by parts. The applied nodal internal force in an acceleration update is the negative of this positive residual term. Every term has work units if \(w\) is a displacement in metres. A dimensionless nodal test yields nodal force in newtons. In quasistatics omit inertia, but remove rigid modes through sufficient boundary data or explicitly handle a compatible nullspace. A free body's static equilibrium is not unique up to only a pressure gauge.

For dead loads the potential is \(\Pi=\int W-\int\rho_0b\cdot d-\int\bar T\cdot d\). Stationarity gives the static weak form. Hamilton's principle applies to conservative dynamics with fixed endpoint variations. Viscous forces enter virtual power or a dissipation functional; they do not follow from a conservative stored-energy potential alone.

The word Lagrangian has two related uses: following material coordinates, and using an action principle for mechanics. For conservative dead loads, define the action

\[
 \mathcal S[d]=\int_{t_0}^{t_1}
 [\tfrac12\dot d^TM\dot d-\Psi(d)+f(t)^Td]\,dt.
\]

With variations zero at the time endpoints, integration by parts gives
\(\delta\mathcal S=\int\delta d^T[-M\ddot d-\nabla\Psi+f]dt\).
Stationarity for all admissible variations produces \(M\ddot d+\nabla\Psi=f\), the semidiscrete Euler–Lagrange equation. Material-frame fluid equations remain Lagrangian descriptions even when viscous dissipation prevents this simple conservative action from being their complete model.

A pressure load on the current surface is a follower load: \(\bar T=-p_{\mathrm{ext}}\operatorname{Cof}(F)N\). Its derivative belongs in Newton's tangent. Holding its direction/area fixed defines a different load. Unspecified natural traction is zero traction, not zero pressure independently of viscous normal stress. Displacement, velocity and acceleration boundary values must be time-compatible under the selected integrator.

## Total and updated Lagrangian formulations

Total Lagrangian (TL) uses \(\Omega_0\), \(P\), reference gradients and fixed material mass. Updated Lagrangian (UL) uses the last accepted body \(\Omega_n\) as an incremental reference. If \(f=\partial x_{n+1}/\partial x_n\), then \(F_{n+1}=fF_n\), \(J_{n+1}=\det(f)J_n\), and \(\rho_n=\rho_0/J_n\). Incremental reference stress is \(P^{(n)}=\det(f)\sigma_{n+1}f^{-T}\). Transforming all terms consistently makes the continuum descriptions equivalent; discrete quadrature, constitutive integration and finite precision may produce different implementations.

One may also integrate the current weak form \(\int_{\Omega_t}\nabla_xw:\sigma\,dv\). Linearizing it requires derivatives of current volume, gradients, stress and traction. The resulting spatial tangent contains both constitutive and geometric terms. In TL, directly differentiating \(P(F)\) already contains the complete corresponding elastic derivative; adding another geometric stiffness to that derivative double counts. The [deal.II step-44 tutorial](https://dealii.org/current/doxygen/deal.II/step_44.html) is a primary implementation study of finite-strain mixed mechanics, not evidence that every updated-mesh code implements the same model.

## Two hand checks before assembly

For simple shear from Chapter 30 with constant \(k\), the teaching energy is \(W=\mu k^2/2\), and \(P_{12}=\mu k\). It stores elastic energy even when \(\dot k=0\), unlike the Newtonian stress \(\sigma_{12}=\eta\dot k\). For pure rigid rotation, \(F=Q\), \(J=1\), \(P=\mu Q-\mu Q=0\). Using linear strain \(\operatorname{sym}(F-I)\) in that finite rotation generally gives a nonzero result, exhibiting the domain limit of small-strain theory.
