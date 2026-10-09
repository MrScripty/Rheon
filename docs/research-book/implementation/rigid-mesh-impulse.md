# Fixed-pose rigid response to mesh impulses

This successor to the qualified P1 triangle-load operator owns and updates a
body's linear and angular velocities. Geometry, center of mass, world-diagonal
inertia and body time stay fixed. The supplied center of mass and inertia are
physical input properties; they are not inferred from an open triangle surface.
This is an imposed instantaneous impulse response, not finite-duration torque
integration, contact resolution, or fluid coupling.

The original [Baraff rigid-body notes](https://www.cs.cmu.edu/~baraff/sigcourse/notesd1.pdf),
sections 2.9–2.11, give angular momentum L=I_world*omega and
I_world=R I_body Rᵀ. Their second-moment inertia formula also derives the
necessary principal-moment triangle inequalities. The original
[Batty, Bertails and Bridson variational coupling paper](https://www.cs.ubc.ca/~rbridson/docs/batty-siggraph2007-variationalcoupling.pdf),
section 3.1 equations 8 and 11, relates surface pressure traction to force and
torque about COM. These sources motivate the continuum identities. Our P1
quadrature and bounded fixed-pose API are new implementations, not stencils
claimed verbatim from those papers.

For the qualified mesh reduction (F,tau), and positive equivalent force duration
s seconds, impose J=s F (N s), K=s tau (N m s). With mass m>0 kg and diagonal
world moments I_i>0 kg m², set V'=V+J/m, omega'_i=omega_i+K_i/I_i. Exact reals
then give m(V'-V)=J and I_i(omega'_i-omega_i)=K_i. At a fixed orientation this is
the rigid momentum jump. During a finite interval anisotropic world inertia can
change; this API does not model that evolution or its gyroscopic effects.

Let T=1/2 m|V|²+1/2 sum I_i omega_i². The exact-real signed work identity is
T'-T=J dot (V+V')/2+K dot (omega+omega')/2. Negative work extracts kinetic
energy. For arbitrary actual endpoints, residuals eP=m(V'-V)-J and
eL_i=I_i(omega'_i-omega_i)-K_i instead give the conditional residual identity
T'-T=W_impulse+eP dot V_mid+eL dot omega_mid. Lean proves this algebra with
explicit premises; it does not prove geometry integration or IEEE conservation.

Mass and inertia must be finite and strictly positive. Principal moments also
obey triangle inequalities; planar equality is supported. Sorting a>=b>=c
reduces this to a<=b+c. Reject a>2b when 2b is finite, then compare a-b<=c.
Under ordinary IEEE round-to-nearest/gradual-underflow semantics Sterbenz makes
that subtraction exact; overflow of 2b cannot reject any finite a. The rational
laboratory checks represented values independently. This is not a Lean IEEE
proof or an error enclosure. Axis alignment of diagonal inertia is declared by
the caller; no inference of principal axes or closed-solid topology occurs.

Admission binds a body identity/generation, retained surface stamp and exactly
matching mesh moment reference/COM. A fixed-size local proposal computes all
loads, impulses, endpoint velocities and diagnostics. Generation overflow,
nonfinite arithmetic, detected nonzero product/quotient underflow, cancellation,
or mismatched ownership returns an error before the single publication. No
heap allocation is added by the body response. Absorbed rounded increments are
allowed and visible through momentum defects from the actual stored endpoints.
Every diagnostic is nearest rounded and unenclosed; none authorizes a fluid
step or integration of pose/time.

The frozen-pose rendered example must use the native stored before/after twist,
show unchanged geometry, label equivalent impulse duration, and distribute a
JPEG at quality 85. Generated outputs and toolchain caches stay outside Git.
