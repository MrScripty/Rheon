# 19 Forces and moving-boundary work

The accepted smoke demonstration supplies one localized acceleration in a fixed box. The separate [explicit body-force API](../implementation/explicit-forces.md) now distinguishes acceleration and force density in the fixed-box carrier. The [native wall/force progression](../implementation/native-wall-force-sequence.md) adds finite wall traction, compatible no-slip and uniform forcing in a separate fixed slab. This chapter's prescribed moving-solid and two-way coupling requirements remain research specifications. The first decision is whether motion is prescribed or dynamically coupled. A scripted paddle can supply arbitrarily large work without losing momentum. A dynamic paddle requires a body mass, inertia, reaction force and compatible time integration.

## Units before parameters

Use metres, seconds and kilograms internally. Density \(\rho\) is kg/m³, dynamic viscosity \(\mu\) is Pa s, and kinematic viscosity \(\nu=\mu/\rho\) is m²/s. An acceleration \(a\) changes velocity by \(\Delta t a\); a force density \(f\), in N/m³, changes it by \(\Delta t f/\rho\). A source of liquid volume is neither of these. Its addition must appear in the volume ledger and pressure compatibility condition.

For a face mass \(m>0\), an acceleration-only update satisfies

\[
\tfrac12m(u+\Delta t a)^2-\tfrac12mu^2
=m\Delta t au+\tfrac12m\Delta t^2a^2.
\]

The second term explains why logging old-velocity power alone does not reproduce a finite explicit step's energy change. `force_work_identity` checks this exact scalar identity. The reference fixture verifies it for a signed acceleration; it makes no general stability claim.

## One geometry interval

A rigid prescribed boundary has world velocity \(u_s(x)=v+\omega\times(x-x_c)\). Retain pose, translation, rotation and version for the interval consumed by transport, pressure, viscosity and output. Sampling only the final mesh pose can miss a swept collision. Sampling wall velocity from one pose and cut volumes from another breaks displaced-volume accounting even if pressure residual is tiny.

For an outward fluid normal, sealed moving-wall flux is positive when fluid-domain volume grows. Reynolds transport gives \(\sum_e Q_e=\dot V\), with \(Q_e=A_e u_s\cdot n_e\). Incompressibility on a moving control volume uses this geometric volume change, rather than imposing a zero sum of all wall fluxes blindly. A translating obstacle inside a fixed sealed container preserves total fluid-domain volume; a piston moving inward does not unless a compensating outlet or source exists.

For fixed-volume cell constraints choose an outward-positive internal incidence B (the negative of Chapter 16’s head-positive pressure incidence) and write \(Bu+Q=s\), where internal incidence columns sum to zero and \(s\) is an integrated volume source. Summing yields the necessary component condition \(\sum Q=\sum s\). `sealed_flux_compatibility` proves this implication under explicit balance and solved-constraint hypotheses. It does not prove a solution exists. Moving-cell schemes must additionally supply their geometric conservation law.

## Pressure and external work

Prescribed boundary data make divergence affine. The weighted adjoint relation includes pressure work \(p^TQ\), as Chapter 8 derives. Separate known boundary terms before testing a homogeneous transpose identity. For two-way coupling use the same normals, areas and pressure interpolation for wall flux, force and torque. Otherwise an apparently correct pressure solution can exchange inconsistent momentum. The variational solid-fluid paper provides the primary coupled formulation [E1].

The original three-cell amount ledger stores balanced and incompatible prescribed flux vectors. Their sums illustrate the necessary component condition; the separate 27-cell projection reference executes a dense pressure solve. No moving-wall pressure solve is claimed. The 3D mesh lab shows prescribed translation and the velocity at a selected collision; it is not a moving-solid fluid solve.

## Implementation contract and experiments

Proposed `ForceField` distinguishes acceleration from force density; `BoundaryInterval` owns immutable start/end geometry and motion. Reject nonfinite input, stale geometry, unclassified boundary and incompatible component source before publication. An accepted report retains force work, wall work, source volume, physical time and geometry version. Failure leaves the previously accepted state coherent.

Begin with gravity and a flat tank. Then use a translating closed obstacle, a rotating paddle and a piston with a specified outlet. Compare volume change against integrated boundary flux, include both signs of wall motion, and refine temporal sampling. Dynamic light-body added-mass tests belong to a later two-way implementation; prescribed-body examples do not establish them. Mesh query accuracy and liquid volume transport remain separate obligations even when the exact algebra passes.

## Work in a frozen-geometry operator

Let M be a positive mass matrix, K a nonnegative strain operator, C a tangential sample map, and Λ a nonnegative wall-friction matrix. With wall samples w, generalized force f and equation residual e, suppose

\[
M(v-u)+\Delta t\{Kv+C^T\Lambda(Cv-w)-f\}=e.
\]

For r=Cv-w and kinetic energy E, the finite work budget is

\[
E(v)-E(u)+\tfrac12\|v-u\|_M^2
+\Delta t\,v^TKv+\Delta t\,r^T\Lambda r
=\Delta t\,v^Tf-\Delta t\,w^T\Lambda r+v^Te.
\]

The minus sign in actuator work follows from wall force −Λr on the fluid. Laboratory-frame energy can increase while relative friction dissipates. The residual must be taken from this unscaled equation, with its norm and normalization declared. This richer matrix statement is a next proof target, numerically checked in small synthetic systems; it is not one of Physics.lean's thirteen compiled contracts. Prescribed normal-wall pressure work requires its own affine constraint term.

An affine projection M(v-u)+Aᵀλ=0, Av=b gives E(v)-E(u)+½‖v-u‖²M=−λᵀb. Here λ is impulse-scaled. A homogeneous stationary constraint has zero right side; a moving constraint supplies work. Feasibility is separate from pressure gauge uniqueness. For two-way solids, combine fluid and body masses and use one adjoint constraint J. Common translation conserves the corresponding generalized momentum only if J annihilates that mode; angular momentum additionally needs rotation modes and consistent interaction points. These are frozen-geometry specifications, not completed moving-solid proofs.
