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

## Checked bridge from strain to the implicit equations

The earlier Physics contracts require a scalar work identity. The new BoundedPhysics contracts derive that identity for a defined fixed finite-strain operator. For arbitrary finite dimensions n and m, let E map velocity degrees of freedom to strain samples, and let μ_e be fixed material/quadrature weights. Define

\[
(Ev)_e=\sum_iE_{ei}v_i,\quad
(Kv)_i=\sum_eE_{ei}\mu_e(Ev)_e,\quad
\mathcal D(v)=\sum_e\mu_e(Ev)_e^2.
\]

Thus K=Eᵀdiag(μ)E. `viscous_work` derives \(\sum_i(Kv)_iv_i=\mathcal D(v)\). `dissipation_nonnegative` derives \(\mathcal D(v)\geq0\) when every weight is nonnegative. The exact coordinate backward-Euler hypothesis is

\[
m_i v_i+\Delta t(Kv)_i=m_i u_i\quad\text{for every }i.
\]

`backward_euler_work` derives \(\sum_i m_i(u_i-v_i)v_i=\Delta t\mathcal D(v)\) directly from these equations. `backward_euler_energy_nonincrease` then applies the existing finite energy theorem: with m_i≥0, Δt≥0 and μ_e≥0, \(\sum_i m_i v_i^2\leq\sum_i m_i u_i^2\). The coordinate update and positive-weight hypotheses replace an assumed dissipative-work conclusion.

The theorem permits zero masses and weights, so it does not prove invertibility, existence or uniqueness. E is arbitrary and fixed during the step; no theorem here identifies it with an assembled symmetric-gradient stencil or proves a rigid-rotation null mode. This is an unforced homogeneous step. Moving-wall work, free-surface traction, nonlinear coefficients, approximate solves and IEEE rounding remain outside its statement.

For the exact witness E=[−1,1], μ=2, unit masses, Δt=1/4 and u=(1,−1), the computed update is v=(1/2,−1/2). The weighted squared norm falls from 2 to 1/2; kinetic energy with its 1/2 factor falls from 1 to 1/4. The derived work is 1/2=Δt·2. A negative weight μ=−1/2 instead gives squared norm 32/9, while an arbitrary candidate not satisfying the update can also increase it. These are exact rational witnesses, not liquid simulations or material calibration.

## Shared shear-mode oracle

The reference code constructs a 32-cell periodic difference matrix with spacing 1/32 m. A sine mode has discrete eigenvalue \(\lambda_h=4\sin^2(\pi/32)/h^2\). With \(\Delta t=0.01\) s and \(\nu\) chosen from 0.01, 0.05 or 0.1 m²/s, backward Euler damps amplitude by \((1+\nu\Delta t\lambda_h)^{-n}\).

An independently assembled dense solve verifies the modal formula at each step. The browser displays the resulting velocity arrows in a three-dimensional slab and reports mass-weighted kinetic energy. The same profiles and energy records generate the book figure. A large implicit step remains damped but does not reproduce exact exponential decay; the error is shown rather than treated as a performance improvement.

## Boundary and material tests

The [filled-box Newtonian implementation](../implementation/newtonian-box-viscosity.md)
uses symmetric strain under stationary free-slip box boundaries. The separate
[native wall/force progression](../implementation/native-wall-force-sequence.md)
qualifies fixed layer-uniform Navier/no-slip walls and external force, with an
explicit simultaneous step. Its explicit squared-increment work correction
differs in sign and interpretation from the backward-Euler decay identity above.
Neither implementation supplies arbitrary embedded/free-surface stress coupling.

Use affine shear to check the stress law and rigid rotation to check the strain nullspace. Add periodic decay, no-slip channel flow, prescribed-wall Couette flow and free-surface traction. The first two are local operator tests; a complete free surface additionally needs pressure and stress coupling. Explicitly record whether a split assumes zero normal viscous stress.

Generalized Newtonian and viscoelastic materials remain later research. A shear-dependent viscosity needs an admissible range and nonlinear residual; viscoelasticity needs history and its own stress evolution. The existing finite-budget nonlinear specification in Chapter 10 remains valid. This expansion does not quietly turn it into an implemented material law.

## A stable operator can damp the wrong mode

Positive semidefiniteness only says that a quadratic energy decreases. It does not imply that the stencil preserves rigid rotation. For Ω about the z axis, points at (−1,0,0) and (1,0,0) have velocities (0,−Ω,0) and (0,Ω,0). Pairwise vector-difference smoothing sees a difference of magnitude 2|Ω| and penalizes it, although the continuum symmetric strain of the rigid field vanishes. A future assembly theorem must show Br=0 for sampled rigid translation and rotation under explicit geometry assumptions. Only then does K=BᵀWB annihilate those modes.

Isolate viscosity for that fixture: a fully rotating drop still needs centripetal pressure, advection and capillary shape dynamics. Check an affine shear field separately, so a stencil that simply applies zero force everywhere cannot pass. At a free surface, normal and tangential traction belong to the stress condition; independent scalar diffusion with convenient component boundary conditions can violate it even when the interior formula matches μΔu.

For comparison with particle methods, the original Weiler et al. viscosity work evaluates conservation, spatial/temporal refinement and spurious viscous forces [R12]. Those criteria transfer as questions to ask of a grid method; its published performance and accuracy do not transfer as Rheon results. PBF positional density constraints, artificial pressure and postprocessed smoothing offer a different real-time model [R15]. Label their coefficients by their actual equations instead of silently assigning μ or σ units.

[R12]: https://animation.rwth-aachen.de/publication/0558/

[R15]: https://mmacklin.com/pbf_sig_preprint.pdf

## Draft checkpoint: stationary-box Stokes acceptance

Unmerged [PR45](https://github.com/MrScripty/Rheon/pull/45), reviewed at
`bbc6089888e257962b968e70ed84ec61b7668d5b`, implements an opt-in
[aligned Stokes experiment](https://github.com/MrScripty/Rheon/blob/bbc6089888e257962b968e70ed84ec61b7668d5b/experiments/aligned_stokes.rs).
It combines bounded explicit symmetric-strain decay with a pressure proposal
on a fixed aligned-box domain. The obstacle is stationary and no-slip; outer
walls are sealed and free-slip. It is not the production liquid timestep or a
moving-body solver, and this book branch does not import that implementation.

The [independent rational controls](https://github.com/MrScripty/Rheon/blob/bbc6089888e257962b968e70ed84ec61b7668d5b/tools/test_aligned_stokes_oracle.py)
construct a cell cycle around all four sides of the central obstacle in a 3³
grid with 26 wet cells. With unit density and viscosity, initial energy 1 J and
dt = 1/32 s, the ideal discrete step ends at **731/1024 J** with exactly zero
cell flux. Pressure is zero by symmetry in this case. Predict the energy loss
(293/1024 J), then compare with the separate asymmetric curl: it develops
nonzero intermediate flux and needs a nonzero pressure correction. The ring
alone does not demonstrate the ability to solve a nontrivial pressure field.

The qualification driver enumerates 13 one-step cases (eight accepted specimens
and five refusal specimens) plus two three-step sequences. These are a finite
acceptance roster, not 15 successful runs or a convergence study. During book
coordination, ten exact mathematical controls were independently rerun in normal
and optimized Python; native execution and the draft owner's source-bound native
receipts remain separate evidence. Exact oracle flux is not a claim that every
accepted binary64 field has identically zero divergence.

The [experimental Lean statements](https://github.com/MrScripty/Rheon/blob/bbc6089888e257962b968e70ed84ec61b7668d5b/experiments/proofs/AlignedStepAcceptance.lean)
assume sound enclosures of energy changes, flux and momentum defects. They derive
conditional acceptance implications; they do not establish Rust/IEEE enclosure
soundness, compiler correspondence or continuum convergence. They are outside
this branch's root Lean inventory and theorem counts. The native interval path
has explicit Linux x86_64/default floating-environment premises and can refuse
unsupported or unresolved arithmetic. An unenclosed scalar stability estimate
cannot authorize an actual step.

The experiment has no advection, source forcing, inflow/outflow, free surface,
variable material, capillarity or moving geometry. It exposes no production
clock/generation owner and is not integrated into PR48's transport facade.
Calibrated wall loads, arbitrary meshes and two-way body/contact dynamics still
need their own geometry, work, impulse and rollback contracts. A stationary
viscous acceptance case does not complete those broader physics topics.

## Affine rotation and shear

Use a centred unit cube as a measurement region, not a no-slip container.
Prescribe U=(-s y,s x,0), constant viscosity mu, and pressure zero. This is rigid
rotation: the velocity gradient is nonzero but D=(grad U+grad U^T)/2 vanishes.
Consequently tau=2 mu D, traction and constitutive dissipation vanish.
Teal arrows show actual sampled velocity at a fixed scale; the field is planar
and independent of z. Camera motion does not advance the field.

## Boundary work supplies shear

Select U=(s y,0,0). Its two off-diagonal strain entries are s/2. The unit-cube
dissipation is mu s² W, which is 1 W at unit rate and viscosity. Orange arrows
show tau n at the six face centres, using a separate labelled traction scale.
The integral of traction dot velocity over those faces is also 1 W.

Constant stress has zero divergence here. Dissipation is supplied by boundary
work, not an evolving bulk-force calculation. All affine component Laplacians
are zero. Comparing mu grad U:grad U with 2 mu D:D while omitting the different
boundary terms would be misleading. These imposed fields do not satisfy
stationary no-slip conditions on the cube faces.

## Extension control

For U=(s x,-s y,0), dissipation and supplied boundary power are 4 mu s² W.
Changing the sign of s reverses the velocity and traction but leaves power
unchanged. Setting viscosity to zero removes traction and dissipation while
leaving the prescribed velocity intact. Read the complete matrices and
six-face quadrature in the [research dossier](../engineering/mesh-faces-to-collocated-flux.md).
Batty and Bridson's [section 3, equations (3.1)–(3.5)](https://arxiv.org/pdf/1010.2832v2)
provides the constitutive/traction context; this lesson does not implement their
embedded-boundary solver or inherit its convergence results.

<details><summary>Planar stress proof scope</summary>

[EngineeringFlux.lean](../../../proofs/Rheon/EngineeringFlux.lean) proves the
planar power formula, nonnegativity when mu>=0, and rotation/shear/extension
values. The boundary theorem checks the stated finite contraction. Geometric
quadrature, PDE evolution and code/IEEE refinement remain separate claims.

</details>
