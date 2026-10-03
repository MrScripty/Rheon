# 10 Forces viscosity and material behavior

Gravity is acceleration: \(\mathbf u^*=\mathbf u+\Delta t\mathbf g\) needs no density multiplier. Pressure and stress use density according to the selected formulation. Configuration and types should distinguish world lengths, grid coordinates and dimensionless coefficients.

## Hydrostatic balance

With upward \(z\) and gravity \(-g\mathbf e_z\), rest requires \(\partial_zp=-\rho g\). A linear pressure field with face differences \(-\rho gh\) cancels gravity in ideal discrete arithmetic when boundaries agree. Measure speed, divergence and surface drift over many steps instead of judging whether a tank looks still.

## Diffusion

One-dimensional explicit diffusion is

\[
q_j^{+}=q_j+r(q_{j-1}-2q_j+q_{j+1}),
\qquad r=\nu\Delta t/h^2.
\]

Nonnegative coefficients require \(0\leq r\leq1/2\), established by explicit_diffusion_positive. In isotropic three-dimensional seven-point diffusion, the center coefficient is \(1-6r\), giving the analogous restriction \(r\leq1/6\). That extension is derived here, not in the current Lean inventory.

Implicit Euler solves

\[
(I+\nu\Delta t L)q^{+}=q
\]

for a positive semidefinite negative Laplacian \(L\). An eigenmode of eigenvalue \(\lambda\geq0\) is multiplied by \(1/(1+\nu\Delta t\lambda)\), between zero and one. Large steps still change decay accuracy; boundaries remain essential.

The companion's 64-cell periodic sine mode uses \(\nu=0.01\), time step 0.001 and 100 steps. Final amplitude 0.9613286500 differs from continuum 0.9612907007 by about \(3.79\times10^{-5}\). This combines spatial and temporal errors in a scalar oracle, not a complete viscous-fluid implementation.

## Stress and constitutive choices

Newtonian stress is

\[
\tau=2\mu D(\mathbf u),\qquad
D(\mathbf u)=\tfrac12(\nabla\mathbf u+\nabla\mathbf u^T).
\]

Replacing its divergence by \(\mu\nabla^2\mathbf u\) uses constant viscosity and incompressibility with compatible boundary terms. Variable viscosity and free surfaces can need coupled stresses. Batty and Bridson's variable-coefficient Stokes work is a primary reference [S11]. State the validity of a simpler componentwise approximation.

Smoke buoyancy may depend on temperature deviation and a density-like tracer. This is not compressible thermodynamics. Vorticity \(\boldsymbol\omega=\nabla\times\mathbf u\) can drive confinement using a normalized gradient of its magnitude crossed with vorticity. Fedkiw, Stam and Jensen use this to compensate for visually lost rotation [S12]. It injects energy; record work and guard near-zero normalization. Resolution changes its effective behavior.

A shear-dependent viscosity may use \(\dot\gamma=\sqrt{2D:D}\) and regularized power law \(\mu=K(\dot\gamma^2+\epsilon^2)^{(n-1)/2}\). Regularization changes zero-shear behavior. Lagged coefficients, Picard iteration and Newton methods have different convergence requirements. Viscoelasticity additionally needs stress history and cannot be modeled by viscosity alone. Delay extensions until their constitutive tests and memory costs are justified.

## A discrete variable viscosity energy

Use a discrete strain operator E mapping face velocities to symmetric strain components at appropriate sample locations. Let M contain quadrature volumes and positive viscosity coefficients, with shear factors chosen consistently with tensor contraction. A viscous dissipation quadratic has the form \(u^TE^TMEu\). Its nonnegativity follows from positive coefficients, independent of a particular stencil layout.

A backward-Euler viscous update with face mass matrix \(M_u\) is

\[
(M_u+\Delta t E^TME)u^*
=M_uu^n+\Delta t f.
\]

This construction exposes symmetry and dissipation. It also shows why independently smoothing each component can miss cross derivatives when viscosity varies or free-surface traction matters. The pressure constraint can be coupled through a saddle-point system or handled by a specifically justified splitting; those alternatives have different boundary errors.

At an impermeable no-slip solid, prescribed velocity contributes known terms when eliminating boundary degrees of freedom. At a stress-free liquid surface, traction conditions belong to the variational boundary treatment. Reusing a closed-box Laplacian at the free surface silently imposes a different stress model.

The expanded companion assembles a small weighted difference energy \(E^TME\) and checks symmetry, nonnegative spectrum and implicit decay. This validates the algebraic construction only. It does not implement the complete three-dimensional symmetric-gradient stencil or free-surface traction quadrature.

## Nonlinear viscosity iteration

For generalized Newtonian viscosity, coefficients depend on strain. A lagged-coefficient iteration takes the current velocity guess, evaluates shear rate and viscosity, builds the linear viscous system, solves it, and repeats until both velocity change and nonlinear residual meet their criteria.

~~~
u_guess = old_velocity
repeat:
    strain = E(u_guess)
    viscosity = material_law(strain)
    require positive finite viscosity
    solve the frozen-coefficient system for u_candidate
    evaluate the original nonlinear residual at u_candidate
    if accepted: return u_candidate
    apply declared damping if needed
    u_guess = updated guess
return NonlinearIterationLimit
~~~

A small iterate change alone can mean stagnation, not convergence. Viscosity clipping can make the linear system manageable but changes the material law. Record the range and active clipping fraction. A yield-stress model needs a stated regularization or complementarity treatment; a high-viscosity threshold is only an approximation.

A simple shear test is the first constitutive oracle. Prescribe a linear velocity profile, calculate constant shear rate analytically, and compare stress against the selected law. Then use a channel-flow case with known or independently computed profile. A visually thick fluid is not sufficient calibration.

## Prescribed and dynamic solid coupling

A prescribed obstacle has externally given motion. It can inject work into the fluid without receiving a matching reaction in the simulation. This is a valid one-way model if stated. A dynamic rigid body needs pressure forces and torques integrated over the same geometric interface used by the fluid operator.

For pressure traction \(-p\mathbf n\), force and torque are

\[
\mathbf F=-\int_{\partial S}p\mathbf n\,dA,\qquad
\boldsymbol\tau=-\int_{\partial S}
(\mathbf x-\mathbf x_c)\times p\mathbf n\,dA.
\]

Discretize these with the same face areas and normals used for flux coupling. Unrelated force quadrature can violate momentum exchange even if fluid divergence is small. The rigid body update also changes wall velocity, so explicit one-way sequencing can have added-mass instability for light bodies. A coupled solve or carefully qualified iteration is a separate capability.

Rheon's initial scope should support prescribed obstacles with explicit wall-work diagnostics. Dynamic two-way coupling should wait for momentum, rest-state and light-body tests. The variational coupling literature [S7] motivates that extension, but the current proof package does not verify it.
