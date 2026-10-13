# 2 From conservation to a tractable model

Consider a fluid region \(\Omega\), velocity \(\mathbf u\), mass density \(\rho\), pressure \(p\), and body acceleration \(\mathbf f\). Local mass conservation is

\[
\partial_t\rho+\nabla\cdot(\rho\mathbf u)=0.
\]

If density is constant along trajectories, then \(D\rho/Dt=0\), and the product rule gives \(\nabla\cdot\mathbf u=0\). This is a model assumption about volume preservation, not a statement that pressure is zero. Pressure is the constraint force that makes velocity compatible with incompressibility and boundaries.

For a Newtonian fluid of constant density and viscosity, momentum becomes

\[
\partial_t\mathbf u+(\mathbf u\cdot\nabla)\mathbf u
=-\rho^{-1}\nabla p+\nu\nabla^2\mathbf u+\mathbf f,
\qquad \nabla\cdot\mathbf u=0.
\]

Here \(\nu=\mu/\rho\) is kinematic viscosity, with units square metres per second. Pressure has units kilograms per metre per second squared. Confusing dynamic viscosity \(\mu\) with \(\nu\) introduces a factor of density. Chorin's original projection work provides a foundational velocity-pressure splitting construction [S3]. Rheon's finite operators are specified independently in later chapters.

## Nondimensionalization

Choose a length \(L\), speed \(U\), time \(L/U\), and pressure scale \(\rho U^2\). Substitution gives

\[
\partial_{\hat t}\hat{\mathbf u}
+(\hat{\mathbf u}\cdot\hat\nabla)\hat{\mathbf u}
=-\hat\nabla\hat p+\mathrm{Re}^{-1}\hat\nabla^2\hat{\mathbf u}
+\hat{\mathbf f},\qquad \mathrm{Re}=UL/\nu.
\]

Changing scene scale while retaining numerical settings changes the apparent material. A two-metre scene moving at one metre per second with \(\nu=0.01\) has \(\mathrm{Re}=200\). Reducing length by ten at the same speed and viscosity gives 20. Configuration should retain world units or explicitly declare nondimensional parameters, never silently interpret one value as both.

Gravity introduces \(\mathrm{Fr}=U/\sqrt{gL}\); surface tension introduces \(\mathrm{We}=\rho U^2L/\sigma\), where \(\sigma\) is force per unit length. Changing surface tension to obtain a desired shape can be intentional but should not be called calibrated water. Features with radii comparable to grid spacing have poorly resolved curvature and pressure jumps.

## Energy and boundaries

Multiply momentum by \(\rho\mathbf u\), integrate, and assume sufficient smoothness, incompressibility and vanishing boundary terms, such as periodic boundaries:

\[
\frac{d}{dt}\frac{\rho}{2}\int_\Omega |\mathbf u|^2\,dV
=-\mu\int_\Omega |\nabla\mathbf u|^2\,dV
+\rho\int_\Omega\mathbf f\cdot\mathbf u\,dV.
\]

Advection becomes a boundary flux; pressure vanishes against divergence-free velocity; integration by parts makes viscosity dissipative. Moving walls, inlets and outlets add work and transport terms. Energy gain is not automatically wrong, but it needs an identified source. Vorticity confinement is an added force and belongs in this accounting.

## What the model excludes

A constant-density incompressible solver omits acoustic waves, compressibility-driven shocks and phase-transition density changes. A buoyant smoke tracer under a Boussinesq approximation is not combustion. A single-phase liquid with prescribed atmospheric pressure does not model trapped compressible bubbles. An inviscid baseline cannot predict calibrated viscous folding. No finite collection of discrete lemmas proves global regularity of three-dimensional Navier-Stokes flow.

These exclusions guide the API. A material enum should describe implemented constitutive behavior. If the only stress is constant Newtonian viscosity, expose that. If foam means a thresholded visual tracer, say so rather than imply a multiphase physical model.

## Splitting errors

Let \(A\) denote transport and \(B\) another evolution operator. One step \(e^{\Delta tB}e^{\Delta tA}\) differs from \(e^{\Delta t(A+B)}\) by commutator terms of order \(\Delta t^2\) locally under suitable smoothness. Over fixed duration, the usual global order is one. Symmetric splitting can improve order under compatible assumptions, but changing fluid regions and boundary handling can invalidate that improvement. Test accuracy on smooth manufactured problems before advertising it for obstacles or free surfaces.

## Wall forces and body work

Consider constant viscosity μ > 0 in a one-metre plate gap with unit area.
The lower wall is stationary, the upper wall moves at U, and a uniform body
force G acts along x. The steady solution is

\[
 u(y)=Uy+\frac{G}{2\mu}y(1-y),\qquad
 \tau(y)=\mu U+\frac G2(1-2y).
\]

Traction on the fluid uses its outward normal: bottom traction is
−μU−G/2 and top traction is μU−G/2. Their sum plus the body force is zero.
The browser recomputes nine velocity/shear samples, wall traction and work.
With μ = U = 1 and G = 2, the top traction is zero and the bottom is −2 Pa.
Body work supplies the dissipation, even though the upper wall moves.

The volume-average speed is U/2 + G/(12μ). Wall power is U(μU−G/2),
body power is G times that average, and dissipation is μU² + G²/(12μ).
Their exact balance follows by integrating the polynomial profile and its
squared derivative. Density does not enter this already specified steady
unidirectional balance. Arbitrary geometry, unsteady inertia, thermal work
and solid response require additional equations and boundary data.

<details><summary>Practical study proof scope</summary>

[PracticalStudy.lean](../../../proofs/Rheon/PracticalStudy.lean) states the
exact real-arithmetic premises. Kernel qualification for these new statements
is pending. The independent finite oracle checks the browser formulas; it does
not prove a Rust refinement, IEEE solver, continuum prediction or runtime speed.

</details>
