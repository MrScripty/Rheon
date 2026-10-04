# 25 Surface tension and integrated qualification

Surface tension supplies an interface stress. It is not viscosity or a wall adhesion force. With constant \(\sigma\), outward liquid normal and curvature sum \(\kappa=\nabla\cdot n\), a spherical interface has \(\kappa=2/R\). Neglecting normal viscous stresses gives \(p_l-p_a=\sigma\kappa\). With viscous liquid stress and negligible air viscous stress, the normal jump instead includes \(n^T\tau_l n\). Chapter 9's sign convention remains authoritative.

## Pressure-force balance

A discrete surface force and pressure gradient should use compatible locations, masks and differences. Otherwise even an accurately solved stationary drop can develop parasitic velocity. Basilisk's original surface-tension implementation expresses an interfacial potential using curvature and applies an explicit capillary stability bound [E10]. Its code is a reference for examining balanced construction, not a borrowed Rheon backend.

The capillary lab holds the same cap volume and angle while varying tension through 0.03, 0.072 and 0.12 N/m. Shape remains the analytic zero-gravity equilibrium cap; pressure jump changes linearly with tension. For \(V=10^{-6}\) m³ and angle 90 degrees, R is approximately 0.007815926 m and the jump at 0.072 N/m is approximately 18.42392 Pa. The reference records use full precision, with these prose numbers rounded.

## Time scales and resolution

The inertial capillary scale is \(t_\sigma=\sqrt{\rho h^3/\sigma}\). The viscous diffusion scale is \(t_\nu=h^2/\nu\). The gravity-capillary length is \(\ell_c=\sqrt{\sigma/(\Delta\rho g)}\). These are dimensional scales, not universal accepted step sizes. Actual stability constants depend on discretization and time integration; implicit tension can change stability while retaining geometric error.

Dimensionless groups separate regimes: \(Re=\rho UL/\mu\), \(Ca=\mu U/\sigma\), \(We=\rho U^2L/\sigma\), and \(Bo=\Delta\rho gL^2/\sigma\). A spherical-cap reference assumes negligible Bo; a shear decay reference excludes an interface. Combining their pictures does not validate a coupled solver.

## Progressively richer teaching examples

The Pages edition presents six linked 3D laboratories: finite MAC projection; earliest triangle-mesh collision with prescribed motion; layered hydrostatic density; implicit viscous mode; Navier-slip Couette flow; and constant-volume wetting/capillarity. Each points to its chapter, original reference implementation, stored data and applicable proof. Parameters select or interpolate declared reference states, rather than pretending to run a general liquid simulator.

The original smoke render remains a separate accepted Rust demonstration with its original source identity. This edition does not relabel that smoke as a liquid calculation, or claim that the educational analytic caps establish production realism. Readers can compare the evidence levels in Appendix F.

## Production integration sequence

After research review, implement one-way collision geometry and gravity first. Add free-surface classification/transport with volume and spatial metrics, then constant and variable density using a consistent pressure correction. Add tensor viscosity and wall traction, then a declared contact-angle reconstruction and balanced capillarity. Dynamic contact lines and two-way solids come after their assumptions are selected. This is a research implementation sequence, not a merge gate for the existing bounded smoke milestone.

An accepted liquid step must publish interface, material fields, velocity, time and geometry version together. Failure in geometry, transport, nonlinear stress or pressure keeps the last coherent state. Capillary, viscous and Courant restrictions must be reported with the actual accepted interval; no hidden parameter changes should imitate a successful solve.

## Evidence that earns a capability claim

Use hydrostatic rest, variable-density layers, viscous decay/Couette flow, translating volume, a stationary droplet and a sessile cap before combined pouring scenes. Report pressure jump, parasitic speed, volume drift, contact angle, wall flux/work and boundary versions. Refine h and dt independently. Then add moving meshes, pinning/depinning and separation, with independent geometry references. Tests must name a physical regime and error budget before advertising calibrated material behavior.

The new Lean contracts remove ambiguity in finite identities and sign conditions. The reference programs qualify local mechanisms. Neither proves arbitrary mesh assembly, full three-dimensional liquid transport, contact-line convergence, floating-point production behavior or performance. Those are explicit research gaps, not reasons to reject the existing smoke demonstration.

## A future liquid qualification suite

The following fixtures are planned for a quantitative liquid implementation. Proposed error targets are starting engineering choices, not universal tolerances or merge gates for the accepted smoke milestone. The local references qualify only the finite/analytical mechanisms described earlier.

## Analytical fixtures

### F1 Planar Couette flow

Use a periodic streamwise channel with 0 ≤ y ≤ H, lower wall speed 0 and upper wall speed U. For stationary fully developed incompressible Newtonian flow with no slip,

\[
u_x(y)=Uy/H,\quad \tau=\mu U/H,\quad P_{wall}/A=\mu U^2/H.
\]

Use synthetic SI values H = 0.01 m, U = 0.1 m/s, μ = 0.1 Pa·s, ρ = 1000 kg/m³. Then ν = 10⁻⁴ m²/s, Re = 10 using gap width, tν = 1 s, τ = 1 Pa, and P/A = 0.1 W/m². Initialize the analytic steady profile for a discrete equilibrium test; start from rest separately to test transient relaxation. Do not claim steady state merely after an arbitrary number of frames.

Proposed gates: relative velocity L2 error below 1%, wall shear error below 2%, and steady wall-work minus dissipation residual below 1% of wall power on the finest test grid. Also verify that changing ρ while holding μ fixed changes transient timescale but not the steady profile.

### F2 Symmetric Navier slip in Couette flow

For equal slip lengths ℓ at both walls,

\[
u_x(y)=U\frac{y+\ell}{H+2\ell},\quad
\tau=\mu\frac{U}{H+2\ell}.
\]

Sweep ℓ/H = 0, 0.01, 0.1, 1. Use a constraint for the zero-slip limit, not division by zero in β = μ/ℓ. Check both fluid slip velocities and both traction signs. For finite ℓ, test

\[
\tau U=\mu\left(\frac{U}{H+2\ell}\right)^2H
+2\frac{\mu}{\ell}\left(\frac{U\ell}{H+2\ell}\right)^2.
\]

This partitions actuator power into bulk and wall dissipation. A solver matching the profile but omitting wall dissipation is still missing an energy term.

### F3 Pressure driven Poiseuille flow

For a planar channel, no slip, and G = −dp/dx > 0,

\[
u_x(y)=\frac{G}{2\mu}y(H-y),\qquad
Q'=\frac{GH^3}{12\mu}.
\]

Q′ has units m²/s and is flow rate per out-of-plane width. With equal Navier slip length ℓ, add GℓH/(2μ) to the velocity and multiply Q′ by 1 + 6ℓ/H. A uniform body acceleration reproducing the pressure gradient is a = G/ρ, not G itself. Check profile, flow rate, wall shear, and input pressure power per unit out-of-plane width GQ′Lx (W/m). Its time integral is work per unit width.

Use the Couette material values and G = 120 Pa/m for a no-slip mean speed 0.01 m/s. Proposed gates: below 1% relative flow-rate/profile error on the finest grid and decreasing errors under refinement. Repeat with a rotated or curved embedded channel only when an appropriate analytic or independently converged reference is supplied.

### F4 Viscous mode decay and rigid motion

For a periodic mode u = (U sin(ky), 0), the convective term vanishes and

\[
u_x(t)=U\sin(ky)e^{-\nu k^2t},\qquad
E(t)/E(0)=e^{-2\nu k^2t}.
\]

This distinguishes μ from ν and reveals excess numerical damping. With backward Euler and constant operator eigenvalue λ, the discrete amplitude ratio after N steps is U_N/U_0 = (1 + νλΔt)⁻ᴺ, where λ ≥ 0 is the eigenvalue of the positive discrete negative-Laplacian operator before multiplication by ν, with units m⁻². Compare against the discrete formula for the integrator test and the exponential for the PDE convergence test.

Independently apply only the viscous step to uniform translation and a rigid rotation sampled on a free domain. Check zero strain, zero viscous force, and unchanged velocity. Rotation tests boundary traction and symmetric-gradient consistency; decay of a nonrigid sine mode tests actual dissipation.

### F5 Static pressure jump and parasitic currents

Turn gravity off, place a circular drop away from walls, set zero velocity, and use constant γ. In a strictly two-dimensional planar cross-section with translational invariance, Δp = γ/R. For a three-dimensional sphere, Δp = 2γ/R. Do not use the sphere formula for a planar 2D disk.

Start with R = 0.001 m and synthetic γ = 0.072 N/m, giving 72 Pa in planar 2D and 144 Pa in 3D. These are declared synthetic parameters, not a measured fluid specification. Use ρl = 1000 kg/m³ and tγ = √(ρlR³/γ); select and report gas density/viscosity or the one-phase approximation. Sweep radii of 8, 16, 32, and 64 grid cells, drop offsets, and at least one density ratio beyond unity.

Measure bulk-phase mean pressure away from the interface band, maximum and RMS velocity, curvature bias/variance, volume drift, and interface error. Normalize parasitic velocity by Uγ = √(γ/(ρlR)); additionally report μUmax/γ when μ > 0. A proposed quantitative gate is pressure-jump error below 2%, relative volume drift below 0.1% over ten tγ, and Umax/Uγ below 10⁻³ on the finest fixture. These targets may need stricter tolerances for long capillary relaxation.

Basilisk's original spurious-current test supplies a reproducible scientific precedent with explicit diameter, Laplace number, viscous damping time, and resolution sweep. It is a reference design to reproduce, not a result already obtained here. [R24]

### F6 Sessile drops at several contact angles

Set gravity to zero, keep fixed liquid volume, and use a smooth flat rigid substrate. For a 2D circular segment with angle θ through the liquid, using radians in the formula,

\[
A=R^2(\theta-\sin\theta\cos\theta),\quad
a=R\sin\theta,\quad h=R(1-\cos\theta).
\]

For a 3D spherical cap,

\[
V=\frac{\pi R^3}{3}(2-3\cos\theta+\cos^3\theta),
\quad a=R\sin\theta,\quad h=R(1-\cos\theta).
\]

At obtuse angles the spherical cap can overhang its footprint, so a height-field-only visualization cannot represent every shape. Compare θ = 30°, 60°, 90°, 120°, and 150° first. Extreme angles are a separate capability claim and require appropriate curvature stencils and film resolution. Report angle measured at a stated distance from the wall, fitted radius, footprint, height, and remaining kinetic energy. Proposed gates are angle error below 2° and dimensions within 2% on the finest baseline grid, with converging errors and stable volume.

The original Basilisk 2D and 3D sessile-drop tests provide implementation-level references. Preserve the distinction between a circular segment and a spherical cap; the published 2D page uses an area-like volume convention. [R22], [R23]

### F7 Dynamic capillary oscillation

Perturb a drop slightly and measure frequency and damping. Use the correct dimensionality and phase model. For an inviscid 3D spherical drop in a dynamically negligible exterior, the degree-l mode has ωl² = l(l − 1)(l + 2)γ/(ρR³), l ≥ 2. With a dynamically important exterior or significant viscosity, use the corresponding two-fluid/damped theory rather than that formula. [R27]

Keep perturbation amplitude small and verify amplitude independence over a decreasing-amplitude sequence. Report oscillation frequency separately from amplitude decay. A method can produce the correct equilibrium radius while being badly overdamped. The original Basilisk oscillation fixture explicitly analyzes frequency error and equivalent numerical viscosity; use it as an additional reproducible baseline for its own dimensional setup. [R25]

### F8 Moving contact line and capillary penetration

For a selected slip/diffuse-interface closure, compare spreading radius and apparent angle versus time while refining the grid at fixed physical regularization length. If ℓs or ε is tied to Δx, report that as a changing-model sequence. Do not label it ordinary convergence to a fixed physical problem.

For a long horizontal circular capillary of radius r, with negligible inertia/gravity/gas resistance, quasi-steady Poiseuille flow, no slip outside the regularized contact region, and constant θ, the Lucas–Washburn reference is

\[
x^2(t)-x^2(0)=\frac{r\gamma\cos\theta}{2\mu}t.
\]

Use θ < 90° and a finite initial filled length to avoid the unphysical infinite startup speed of the inertia-free approximation. Fit only a time window where its hypotheses hold; entrance losses, inertia, gravity, dynamic angle, and trapped gas can all invalidate it. The original Washburn publication supplies the square-root penetration law and its assumptions. [R26]

### F9 Moving walls and two way solids

Translate a wall through initially stationary fluid and rotate a paddle with prescribed angular speed. Plot accumulated actuator work against changes in kinetic/potential/surface energy and dissipation. Then move both fluid and obstacle at the same constant velocity; a Galilean-consistent relative-velocity treatment should not generate drag from their common translation. Repeat as the obstacle crosses grid cells.

For a freely moving solid in a closed fluid system without external forces, check combined linear momentum. Check total angular momentum only with compatible rotational geometry and transfer. During a fixed-geometry implicit projection, T4 predicts a particular nonnegative numerical energy loss, not exact kinetic-energy conservation.

## Geometry stress fixtures

| Fixture | Sweep | Future capability observation |
|---|---|---|
| Thin closed plate | Thickness/Δx from 4 to 0.1; sub-cell offsets and orientations | No unreported disappearance or leakage; displaced volume error disclosed |
| Zero-thickness two-sided shell | Crossings of every interpolation/advection stencil | Barrier respected or explicit unsupported status |
| Narrow slot | Width/Δx = 0.25, 0.5, 1, 2, 4, 8 | Conductance convergence when resolved; no invented sub-grid accuracy |
| Sliver cut cell | Fluid fraction approaching zero | Stability, flux conservation, redistribution locality, no negative mass |
| Moving cut surface | Swept cell fractions and common translation | Uniform-field preservation and mass budget under covering/uncovering |
| Rotating paddle | Angular speed and interaction-point offsets | Correct local wall speed, torque, and work signs |
| Dirty mesh | Open seam, self-intersection, reversed patch, degenerate triangles | Deterministic rejection or documented repair; never silently plausible output |
| Stale deformation map | Bending or scaling obstacle after preprocessing | Update/rebuild or explicit unsupported status |

Collision leakage should be measured as fluid mass crossing a labeled impermeable barrier, not merely particles visibly inside a solid at the end of a frame. For a closed static tank, declare a finite reference observation time T = Lref/Uref with Uref > 0, or select tγ or tν when there is no reference flow. Proposed regression gates are relative mass drift below 0.1% over ten such T and zero topological barrier crossings in exact-geometry tests; engineering fixtures should additionally report the numerical tolerance used to classify a crossing.


[R22]: https://basilisk.fr/src/test/sessile.c

[R23]: https://basilisk.fr/src/test/sessile3D.c

[R24]: https://www.basilisk.fr/src/test/spurious.c

[R25]: https://basilisk.fr/src/test/oscillation.c

[R26]: https://journals.aps.org/pr/abstract/10.1103/PhysRev.17.273

[R27]: https://pmc.ncbi.nlm.nih.gov/articles/PMC11470527/
