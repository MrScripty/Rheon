# Learning path: from strain to a single sphere impact

This reading edition starts from PR36 source
`d31cf735645579357f9f58dcc55958e23f77af59` (tree
`68bfbefca7f83ead7468ce870b57353e2753e27d`). Its newer mechanisms are separate,
bounded contracts. A mesh load, a fluid pressure correction and a sphere impact
do not yet form a coupled solid–liquid solver. Read the following guides in order;
the HTML pager, PDF contents and combined Markdown use this same sequence.

## 1. Establish the wall and geometry models

Begin with the [native wall/force sequence](native-wall-force-sequence.md),
then [finite Navier walls](column-wall-friction.md), [no-slip](column-no-slip.md)
and [forced Poiseuille flow](column-poiseuille.md). Case/time controls select
saved native values. Missing original packets are explicitly marked absent;
these controls cannot recompute material parameters.

Continue with [retained obstacle geometry](static-obstacle-geometry.md) and
[pressure and reduced shear](static-obstacle-flow.md). A sealed pressure
projection and fully developed extruded shear have different admissible flows.
Neither demonstration establishes arbitrary three-dimensional viscous flow.

## 2. Derive strain before attempting a step

The [reconstructed strain](reconstructed-aligned-strain.md) defines finite
rows E and weights W. For face velocity u, set K=EᵀWE. Expanding the transpose
gives u·Ku=(Eu)·W(Eu), which is nonnegative for nonnegative weights. With
positive face-mass matrix M, the exact unforced reference update is

    v = u - dt*mu*M^-1*K*u.

Expanding its kinetic-energy difference gives

    H(v)-H(u) = -dt*mu*(Eu)·W(Eu)
                 + (dt*mu)^2*(Ku)·M^-1*(Ku)/2.

The finite coefficient bound makes this nonpositive when dt*mu*B≤2.
The [finite-strain laboratory](aligned-strain-laboratory.md) changes face
samples and viscosity to expose those terms; it advances no time. Ask which
normal/shear rows become nonzero, and why zero rows must remain in the inventory.

The [experimental viscous/pressure step](experimental-aligned-stokes.md)
adds acceptance checks on actual rounded proposals: outward energy, divergence
and relative equation-defect enclosures. The theorem's exact-solve premises
cannot be replaced by a small diagnostic residual. The experiment is an opt-in
sealed grid-aligned box with a stationary no-slip internal obstacle; no moving
boundary, free surface or fluid–body coupling follows from it. Its
[Lean strain contracts](../../../proofs/Rheon/AlignedStrain.lean) and separate
[acceptance audit](../../../experiments/proofs/AlignedStepAcceptanceAudit.lean)
make those premises inspectable.

## 3. Integrate a load, then apply an impulse

For a triangle of area A with affine corner tractions tᵢ,
integrating barycentric products gives

    f_i = A/12*(sum_j t_j + t_i),
    F = sum_i f_i,    tau = sum_i (x_i-c) cross f_i.

The [mesh traction guide](triangle-mesh-traction.md) derives the moments and
virtual work. On its unit-area example, changing only the second corner to
traction (0,0,3) yields F=(0,0,1), tau=(1/4,-1,0). Applying the resultant
at the centroid gives the wrong torque. This is a useful first numerical lab:
check the independent degree-two cubature against the actual native loads,
then shift the moment reference and verify tau_new=tau-s×F.
The continuum load target is discussed by
[Batty, Bertails and Bridson (2007), §3.1](https://www.cs.ubc.ca/~rbridson/docs/batty-siggraph2007-variationalcoupling.pdf);
Rheon's P1 surface integration is derived separately and supplies no matched
reaction on a fluid. See [MeshTraction.lean](../../../proofs/Rheon/MeshTraction.lean).

The [rigid impulse guide](rigid-mesh-impulse.md) connects force/torque to
velocity and angular momentum while holding geometry fixed. Compare loads,
impulses and velocities with their units: N, N·m, N·s, kg·m²/s and m/s.
For the same native varying-traction fixture, the equivalent duration is 0.5 s,
so the recorded linear impulse is (0,0,0.5) N·s and angular impulse is
(0.125,-0.5,0) N·m·s. With mass 2 kg, the recorded velocity changes from
(1,2,3) to (1,2,3.25) m/s. These are velocity updates at fixed geometry,
not a clock or pose advance.
Its [Lean source](../../../proofs/Rheon/RigidImpulse.lean) is finite exact algebra,
not a proof of rounded inertia inversion or an advancing pose.

## 4. Advance a declared spherical-inertia body

The [spherical motion guide](spherical-rigid-motion.md) adds an actual COM clock,
quaternion drift and native regenerated vertices. Inspect stored frames at
their recorded times: moving the camera or drawing a mesh at new coordinates
does not establish accepted time advancement. Spherical inertia is a declared
mass model, not inferred from the rendering mesh. The first-order sampled-load
path has no general-inertia, contact or fluid accuracy claim. Inspect
[RigidMotion.lean](../../../proofs/Rheon/RigidMotion.lean) beside the native
pose and clock defects.

## 5. Locate a finite-feature event and stop

The [sphere contact guide](static-sphere-contact.md) sweeps a declared COM-centered
sphere against actual closed finite facets. A face, edge and vertex are distinct
closest-distance regions; an infinite-plane hit can miss the finite triangle.
[Fauerby (2003), §§3.3–3.4](https://www.peroxide.dk/papers/collision/collision.pdf)
describes that feature separation. Rheon's root/refusal policy is derived
independently and does not adopt the paper's sliding response.

For a unit normal n, incoming vn=V·n<0 and restitution 0≤e≤1,

    J = -(1+e)*m*vn*n,    V_plus = V + J/m,
    deltaT = -(m/2)*(1-e^2)*vn^2.

The sphere lever arm is -R*n, so its angular impulse is zero. Compare the
native face, edge and vertex events against the independent 80-digit
closest-distance root oracle, rather than against a second copy of the
production seven-feature formula. The impulse/restitution background is
[Baraff (1997), §8](https://www.cs.cmu.edu/~baraff/sigcourse/notesd2.pdf);
[SphereContact.lean](../../../proofs/Rheon/SphereContact.lean) proves conditional
exact-real algebra, not floating-point feature coverage.

A unique hit ends immediately after **one frictionless sphere impact**.
The unused interval remains unused. There is no repeated or resting contact,
friction, rolling, gravity, general moving-mesh CCD, or fluid coupling. Initial
touch/overlap, ambiguous features, grazing and near-simultaneous facets refuse
without publishing a partial state. Stored endpoint pictures show the declared
sphere collider separately from the retained render/traction mesh. The
[requirements roadmap](requirements-roadmap.md) records the remaining work.

## What qualifies this edition

Source identity, theorem qualification, native numerical evidence and browser
behavior are separate gates. The build must refuse stale proof bindings; a hash
edit cannot substitute for compiling and auditing the changed modules. A
source-only lab page communicates absence rather than displaying invented
results. Local PDF/browser receipts bind the exact edition bytes. Publication
still requires parent review of the source changes, the PDF and its receipts.
No held PR25 case41 or geometry-band campaign is needed for this learning path.
