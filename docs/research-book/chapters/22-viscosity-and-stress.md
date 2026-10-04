# 22 Viscosity, stress and dissipation

Density and viscosity have different roles. Density weights inertia; dynamic viscosity relates deformation rate to stress; kinematic viscosity controls a simple diffusion time scale. Treating a viscosity slider as an arbitrary velocity damping factor hides both the material units and boundary work. The first liquid research model is Newtonian, with positive density and nonnegative viscosity.

## Symmetric strain

For \(D=(\nabla u+\nabla u^T)/2\), Newtonian stress is \(\tau=2\mu D\). Its power dissipation is \(2\mu D:D\geq0\). The cross derivatives matter: rigid rotation has zero symmetric strain even though velocity gradients are nonzero. A scalar component Laplacian is equivalent to the incompressible constant-viscosity interior equation only under the matching assumptions and boundary terms.

Batty and Bridson's original viscous-free-surface paper separates normal and tangential traction and builds a compatible variational discretization [E4]. Their later variable-coefficient Stokes formulation supports irregular moving boundaries [S11]. Basilisk's original viscosity implementation is another useful code reference for coupled stress components [E5]. These sources motivate a choice; they do not qualify Rheon's future stencil.

## Finite dissipative update

Let face masses form positive diagonal \(M\), let E map face velocity to strain samples, and let nonnegative quadrature coefficients form W. Put \(K=E^TWE\). Homogeneous stationary boundaries and zero external force give backward Euler

\[
M(v-u)+\Delta t Kv=0.
\]

Taking the inner product with v and completing a square yields

\[
\|u\|_M^2-\|v\|_M^2
=\|u-v\|_M^2+2\Delta t\,v^TKv\geq0.
\]

`implicit_energy_identity` checks the finite identity from an explicit scalar work equation. `implicit_energy_nonincrease` adds nonnegative time and dissipation to obtain the inequality. Its finite sums use arbitrary positive masses, so variable density is represented without assuming uniform kinetic weights. It proves neither that an iterative solve reaches the work equation nor that E is the actual symmetric-gradient stencil.

Moving walls and body forces add work on the right. Energy need not decrease then. A diagnostic should compare measured energy change against viscous dissipation and declared external work, with residual and rounding budgets, rather than reject every energy increase.

## Shared shear-mode oracle

The reference code constructs a 32-cell periodic difference matrix with spacing 1/32 m. A sine mode has discrete eigenvalue \(\lambda_h=4\sin^2(\pi/32)/h^2\). With \(\Delta t=0.01\) s and \(\nu\) chosen from 0.01, 0.05 or 0.1 m²/s, backward Euler damps amplitude by \((1+\nu\Delta t\lambda_h)^{-n}\).

An independently assembled dense solve verifies the modal formula at each step. The browser displays the resulting velocity arrows in a three-dimensional slab and reports mass-weighted kinetic energy. The same profiles and energy records generate the book figure. A large implicit step remains damped but does not reproduce exact exponential decay; the error is shown rather than treated as a performance improvement.

## Boundary and material tests

Use affine shear to check the stress law and rigid rotation to check the strain nullspace. Add periodic decay, no-slip channel flow, prescribed-wall Couette flow and free-surface traction. The first two are local operator tests; a complete free surface additionally needs pressure and stress coupling. Explicitly record whether a split assumes zero normal viscous stress.

Generalized Newtonian and viscoelastic materials remain later research. A shear-dependent viscosity needs an admissible range and nonlinear residual; viscoelasticity needs history and its own stress evolution. The existing finite-budget nonlinear specification in Chapter 10 remains valid. This expansion does not quietly turn it into an implemented material law.

## A stable operator can damp the wrong mode

Positive semidefiniteness only says that a quadratic energy decreases. It does not imply that the stencil preserves rigid rotation. For Ω about the z axis, points at (−1,0,0) and (1,0,0) have velocities (0,−Ω,0) and (0,Ω,0). Pairwise vector-difference smoothing sees a difference of magnitude 2|Ω| and penalizes it, although the continuum symmetric strain of the rigid field vanishes. A future assembly theorem must show Br=0 for sampled rigid translation and rotation under explicit geometry assumptions. Only then does K=BᵀWB annihilate those modes.

Isolate viscosity for that fixture: a fully rotating drop still needs centripetal pressure, advection and capillary shape dynamics. Check an affine shear field separately, so a stencil that simply applies zero force everywhere cannot pass. At a free surface, normal and tangential traction belong to the stress condition; independent scalar diffusion with convenient component boundary conditions can violate it even when the interior formula matches μΔu.

For comparison with particle methods, the original Weiler et al. viscosity work evaluates conservation, spatial/temporal refinement and spurious viscous forces [R12]. Those criteria transfer as questions to ask of a grid method; its published performance and accuracy do not transfer as Rheon results. PBF positional density constraints, artificial pressure and postprocessed smoothing offer a different real-time model [R15]. Label their coefficients by their actual equations instead of silently assigning μ or σ units.

[R12]: https://animation.rwth-aachen.de/publication/0558/

[R15]: https://mmacklin.com/pbf_sig_preprint.pdf
