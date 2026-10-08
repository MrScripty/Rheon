# Single-face stationary sphere support

New local feature from preserved `446c68416755d8d38bff313d8a3d62c4d9a9e050`.
PR36 and all impact APIs remain unchanged. No public publication/CI is requested.

## Force, constraint and interval contract chosen before implementation

Declare the same fixed-radius COM sphere and positive exactly isotropic stored
mass/moment. A separate explicit stationary-support API admits only V=omega=0,
exactly one fixed triangle, and a supplied represented contact-point witness p
strictly inside its face. This is an equilibrium subset, not a settling rule.
No impact refusal is redirected to this API. Contact edges/vertices, multiple
facets, moving supports and approximate gap/velocity bands remain unsupported.

Reuse StaticSphereSweep admission for stamps, collider radius/moment and bounded
geometry, without invoking its impact-only initial-contact query. Reuse the
existing TriangleSurface segment query from COM c to p as a geometry probe.
Then independently check the proposed witness: exact represented |c-p|^2=R^2,
(c-p) dot (vertex-p)=0 at all three vertices, and strict projected interior
orientation signs. Use the existing bounded ProductSign kernel for these signs;
expand coordinate differences before multiplication, without rounded sign tests.
The segment probe's rounded point is diagnostic and does not replace p. A failed
probe or exact witness predicate refuses without adjusting pose, radius or point.
No formal IEEE refinement of the sign kernel is claimed.

External loading is an immutable retained moving-mesh P1 load, reduced with the
existing TriangleMeshLoad primitive about c, plus caller-declared constant
gravity acceleration g applied at COM. F=Fmesh+m*g and tau=taumesh are the stored
nearest-rounded reduced wrench. Correspondence to exact physical integrals and
the continuous gravity force is an explicit proof premise, not an IEEE claim.
Require tau=0 and exact represented collinearity (c-p) cross F=0. Its inward
sign is (c-p) dot F<=0. Outward force refuses LiftOff; a tangential component or
torque refuses rather than adding friction, adhesion or an orientation lock.

The support vector is Fs=-F applied at actual p. Check and report actual lever
r=p-c, its evaluated torque r cross Fs, net force and net torque. A nonzero
rounded residual torque refuses before publication even if the exact radial
premise holds. Thus the admitted *represented reduced wrench* balances exactly.
The diagnostic normal (c-p)/R and |Fs| are nearest-rounded, unenclosed values;
their approximate normalization does not authorize equilibrium. Zero load is
a neutral zero-support equilibrium, with no positive force invented.

The force proposal is immutable and independent of elapsed time. A finite h>0
specifies its equivalent impulse/work diagnostics. An optional held-interval
successor uses the existing PR35 zero-net-load atomic motion primitive only
after all equilibrium predicates and diagnostics succeed. No external kick,
normal-only state, pushout, clipped force or half-step is published. Net force
and torque are zero, so it is the exact-real stationary special case of that
primitive. The first held implementation requires stored identity orientation:
this is a representation restriction so the inherited quaternion regeneration
cannot renormalize a nonidentity resting pose. The immutable force proposal
does not need this restriction. General stored-pose preservation is deferred.

The held interval inherits the owner's max_dt, positive representable clock,
stamps/generation, geometry and payload bounds. All failure/cancellation paths
preserve owner pose, mesh, time and stamps. A report distinguishes support force
in N, equivalent component impulses in N s, zero support/external work in J,
nearest-rounded impulse-ledger defects, and the actual held-motion report.
Separate component impulse products may have rounding residuals although the
stored net wrench is zero. Diagnostics never certify a general timestep.
Caller loading/gravity and static geometry are declared constant over each h.
Changing loads is a new admission; upward load does not retain stale support.

## Original research and the scalar reduction

[Baraff's original nonpenetration notes](https://www.cs.cmu.edu/~baraff/sigcourse/notesd2.pdf),
section 9, equations 9-5--9-9 (PDF pages 22--23), formulate repulsive force,
nonnegative normal acceleration and zero force during separation. For a resting
radial sphere on a fixed face, n is unit, torque from N*n is zero, and the normal
constraint has scalar compliance 1/m:

    a_n=(F dot n+N)/m, N>=0, a_n>=0, N*a_n=0.
    N=max(0,-F dot n).

For the chosen equilibrium subset F=-(N*n), tau=0, V=omega=0, both acceleration
and power vanish and pose/gap stay constant for a constant-load interval.
There is no matrix/QP solver choice in this one-constraint closed form. A
general interacting contact set would need a separate explicit solver choice;
rolling/sliding would additionally need a frictional evolution contract.

[Mirtich's original thesis](https://people.eecs.berkeley.edu/~jfc/mirtich/thesis/mirtichThesis.pdf),
section 4.6 (PDF pages 101--103), discusses static contact, ordinary-impulse
limitations and special microcollision treatment. We do not implement that
collision-envelope/restitution mechanism or claim it produces exact rest.
The existing isolated/repeated impact APIs are unforced and refuse initial
touch; e=0 alone cannot balance a continuing gravity force or prove equilibrium.

## Conditional proof and qualification boundaries

Lean will prove the scalar complementarity solution/uniqueness, radial support
torque, conditional matched force balance, zero-power/work and stationary
energy/gap preservation under explicit physical/load/geometry premises. It will
not prove settling, global contact completeness, PDE convergence, generic
resting/friction dynamics or IEEE soundness.

Native/reference labs will cover gravity support over repeated held intervals,
nonzero consistent P1 force with matched zero torque, neutral load, reflected
and exactly representable oblique faces, winding/translation/scale changes,
unsupported tangential/torque/lift-off/multiple/boundary/gap/twist cases,
nonidentity force proposal versus held-pose refusal, all cancellation stages,
clock/arithmetic/payload refusals and distinct old impact initial-touch behavior.
Fresh proof/audit compilation, immutable source/evidence checkpoints and
independent reference review are required. PR25 and broad physics campaigns
remain held. No arbitrary mesh collider, fluid coupling or new global engine.
