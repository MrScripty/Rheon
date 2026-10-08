# Isolated Coulomb sphere impact: next bounded collision contract

This local successor starts from unchanged PR37
`c2286e7b8ee73997ca3df2243730b6ac8d376f49`. Its release agent remains pinned to
PR36; this branch is not automatically part of the reading edition. Public
publishing is held for parent coordination.

## Selection and scope

The documented collision boundary lacks tangential response and actual contact
torque. The smallest useful production extension is a single isolated frictional
impact of the already declared COM sphere against a stationary finite triangle
union, followed by an explicit stop. It can reduce slip, transfer energy between
translation and spin, and report the static reaction impulse. The existing
frictionless interval driver remains unchanged. There is no automatic frictional
remaining-time continuation, resting-force solver, gravity, adhesion, rolling
resistance, moving support, two-body motion, simultaneous-contact law or fluid
coupling. Persistent contact would need an acceleration/constraint-force contract;
repeated zero-time collision impulses are not introduced as a substitute.

Use the retained moving triangle mesh for load/render geometry and the analytic
sphere for collision geometry. Reuse PR36's unique finite face/edge/vertex event
and PR35's free-coast proposal; update both velocity and spin before its single
atomic publication. A miss coasts the whole requested interval. A hit reports
unused time and stops. Initial touch, ambiguity, near-simultaneous facets, motion
limits, arithmetic failure, stamps and cancellation remain explicit refusals.

[Mirtich's original thesis](https://people.eecs.berkeley.edu/~jfc/mirtich/thesis/mirtichThesis.pdf),
Collision Response chapter, printed pp.45–53 (PDF pp.66–74), describes contact
velocity, Coulomb friction, momentum/torque impulse equations and the collision
matrix. It also warns that general frictional collisions can change sliding
direction and that Newton restitution can produce energy increases with coupled
contact inertia. This contract does not implement his full Stronge/Routh
collision integration or microcollision resting model. The fixed-point isotropic
sphere law below is newly derived here from those kinematics and a declared
constitutive choice. [Baraff's original rigid notes](https://www.cs.cmu.edu/~baraff/sigcourse/notesd2.pdf)
remain the normal-event/impulse reference, and the preceding
[finite sphere contact guide](static-sphere-contact.md) supplies geometric scope.

## Constitutive model and actual contact lever

At the actual stored free-coast endpoint c, let q be the detector's finite static
closest point. Set r=q-c, d=|r| and n=-r/d, pointing toward the sphere COM.
The measured gap d-R must satisfy the existing caller gap policy. The response
uses this actual q-c lever; it does not silently replace d by nominal radius R.
Thus the detected static point is also the applied impulse point. The ideal
physical sphere correspondence d=R is an additional exact-real premise. Stored
r, n and d are nearest-rounded; collinearity, unit-normal and geometry/momentum
residuals are diagnostics, not a floating-point refinement proof.

Caller-supplied mass m>0 and exactly isotropic moment I>0 retain the existing
physical-inertia admission. Material coefficients are dimensionless normal
restitution 0<=e<=1 and Coulomb coefficient mu>=0. mu is not the dimensional
Navier beta or a liquid adhesion parameter. The instantaneous contact point,
normal, lever and mass properties are held fixed during the impulse.

Define relative point velocity and normal/tangent split against the fixed support:

    g = V + omega cross r,  vn = V dot n,
    vt = g - (g dot n)n,
    Jn = -(1+e)m vn > 0,
    k = 1/m + d^2/I,
    q_stop = |vt|/k,  q_cap = mu Jn,
    qt = min(q_stop,q_cap),
    Jt = -qt vt/|vt| if |vt|>0; otherwise Jt=0,
    J = Jn n + Jt,
    V_plus = V + J/m,
    omega_plus = omega + (r cross J)/I.

The implementation may stage the existing normal response and then tangential
response; their rounded residuals remain explicit. No normal-only intermediate
state is published. Numerically zero slip produces no tangential impulse; it
is not an assertion of exact sticking or a resting-contact solution. Reported
regimes describe the computed candidate (uncapped slip cancellation versus
Coulomb-capped impulse), not an IEEE-certified material classification. Near
threshold decisions and all arithmetic restrictions must be stated in the final
API. Negative/nonfinite mu and nonrepresentable calculations refuse atomically.

## Derivation and energy boundary

For an exact unit normal with r=-d n and Jt dot n=0, the contact inverse inertia is

    K J = (Jn/m)n + k Jt.

Normal and tangent directions decouple for this isotropic radial lever; this is
why the general Newton/friction energy warning does not invalidate this subset.
The normal law gives vn_plus=-e vn. Tangential point velocity becomes
vt_plus=(1-k qt/|vt|)vt, with multiplier in [0,1], so slip is not reversed by the
exact law. For positive slip, minimizing
vt dot Jt + (k/2)|Jt|^2 on the impulse disk |Jt|<=mu Jn gives the declared qt.
The disk constrains impulse, not a finite-duration force model. Sticking here
means only cancellation of the instantaneous post-impact tangential velocity;
it does not establish a continuing no-slip constraint.

Exact total kinetic energy includes both translation and spin:

    T = m|V|^2/2 + I|omega|^2/2,
    DeltaT = g dot J + |J|^2/(2m) + |r cross J|^2/(2I),
    DeltaT = -(m/2)(1-e^2)vn^2 - qt|vt| + (k/2)qt^2 <= 0.

Translation alone can gain energy when friction draws from spin. The static
support receives -J at q; in exact matched geometry the world-origin angular
increment equals q cross J. No evolving support body or finite force history is
claimed. Units: V and g m/s, omega rad/s, m kg, I kg m^2, r/d/R m, J N s,
angular impulse N m s, k 1/kg, T/work J.

Focused Lean contracts will prove the contact inertia, linear/angular impulse,
point virtual-work/kinetic increment, conditional restitution, tangent decay and
nonincrease from explicit unit-normal/radial/mass/constitutive premises. They
will not prove floating geometry, libm, mode decisions, Rust assembly or general
frictional collision completeness. Numerical reports separately measure the
cone, tangent law, normal restitution, linear/spin/world angular momentum,
actual total energy, point work, and geometry residuals. All are unenclosed;
no tolerance or nearest-rounded estimate authorizes a general dynamics step.

## Qualification before local review

Rebuild actual native single-event specimens with sliding, slip cancellation,
zero mu, zero slip, spin-driven slip, energy transfer into translation, finite
edge/vertex normals, oblique/reflected and nonmidpoint geometry. Independently
solve the contact impulse disk with exact rational axis fixtures and high
precision geometric events; compare actual stored poses/meshes and both velocity
components. Test mode boundaries, initial/nonapproaching/simultaneous refusals,
arithmetic restrictions and cancellation at every prepublication stage. Rerun
PR37/PR36/PR35 contracts and old native oracles against protected prior source.
Build fresh Lean outputs outside Git and audit axioms with clean negative probes.
Independent review is required. Keep raw evidence outside Git and persist local
source/evidence checkpoints privately; no paid CI, held campaign, main merge,
deployment, or public publication in this task.
