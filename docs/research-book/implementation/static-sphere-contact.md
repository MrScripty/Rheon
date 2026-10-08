# One spherical collider against a retained static triangle surface

This opt-in successor to PR35 connects its moving spherical-inertia owner to
actual finite static triangles. The collision shape is a caller-declared sphere
of radius R centered at COM. The retained moving mesh still supplies rendering
and traction geometry; it is not silently reclassified as a sphere or claimed
to be the collision boundary. A collision envelope is a different physical
model from contact between arbitrary moving triangle meshes.

## Research and newly derived contract

Kasper Fauerby's original [2003 collision paper](https://www.peroxide.dk/papers/collision/collision.pdf),
§§3.3–3.4, separates a swept sphere's contacts with the interior of a finite
triangle, its finite edges and its vertices. We independently derive that
geometry below. We do not adopt its sliding response. David Baraff's original
[1997 rigid-body notes](https://www.cs.cmu.edu/~baraff/sigcourse/notesd2.pdf),
§6, distinguishes geometric collision accuracy from integration accuracy;
§8 Eqs8–5–8–7,8–10,8–18 gives impulse, angular impulse and restitution.
The event query, numerical refusals and atomic owner connection here are new
derived implementation choices, not paper algorithms asserted verbatim.

There are no applied loads during the interval. COM follows c(t)=c0+t V.
Sphere orientation may coast under PR35's spherical-inertia method and does
not affect its collision shape. Static triangles form a two-sided union of
closed facets, including their finite edges and vertices. Open or disconnected
surfaces are meaningful barriers; no enclosed-solid inside/outside is inferred.
Initial overlap or touching any facet is refused. The scan is capped and
cancellable. Ambiguous feature transitions, grazing roots, and near-simultaneous
earliest facets are refused rather than solved as a contact manifold.

For normalized interval parameter s, d=h V and w=c0-a. A face has unit
normal n: solve n·(w+s d)=±R and test the projected point's three barycentric
coordinates. For an edge e=b-a, subtract the components parallel to e from w
and d. Its squared perpendicular distance gives

    A s² + 2 B s + C = 0,
    A=d_perp·d_perp, B=w_perp·d_perp, C=w_perp·w_perp-R².

Retain only roots whose projected edge parameter is in (0,1) and whose
closest point on the entire triangle is that edge. At a vertex use the same
quadratic with w=c0-a and d unchanged, and test its vertex Voronoi region.
The seven feature regions cover the exact-real closest-point partition of a
nondegenerate triangle. The runtime conservatively refuses conditioned
partition boundaries. It evaluates both roots with a cancellation-resistant
quadratic formula, selects approaching roots, then the unique earliest facet.
This finite-feature derivation is not an IEEE completeness proof.

At the event, n points from the static closest point to the sphere center.
For an edge or vertex it need not equal a triangle's winding normal. With
unit n, vn=V·n<0, mass m>0 and restitution 0≤e≤1,

    j=-(1+e)m vn, J=j n, V_plus=V+J/m.

The sphere's contact lever arm is -R n, so its angular impulse is exactly
zero and angular velocity is unchanged. Thus vn_plus=-e vn and
ΔT=-(m/2)(1-e²)vn²≤0. Static support receives -J but is not a second evolving
body. No friction, rolling, gravity, resting-contact solver, multiple-contact
response, or continuation through the unused portion of the requested interval
is supplied. A hit stops immediately after one impact; a definite miss coasts
the whole interval. A second call starting in contact is refused.

Caller-supplied mass/inertia are not inferred from either mesh. The caller
declares an isotropic physical mass distribution contained within R. A
conservative arithmetic admission check enforces its necessary condition
3I≤2mR²; this condition alone does not reconstruct that distribution.

## Numerical interpretation and evidence

Physical distances are metres, time seconds, velocity m/s, impulse N s and
energy joules. TriangleSurface's admitted relative tolerance conditions feature
classification and quadratic roots. The caller also supplies a maximum allowed
contact gap residual in metres and a near-simultaneous event window in seconds.
These are explicit numerical policies, not global geometric/time error bounds.
Reported gap, requested versus represented event time, normal norm, momentum,
restitution and energy defects are nearest-rounded and unenclosed. Passing
an algebraic comparison at 1e-12 does not establish 1e-12 physical accuracy.
Qualification separately compares event time, geometry and response with an
80-digit independent closest-distance minimization/root oracle, then declares
the measured fixture errors and the limited tested domain.

Before publication, all query, body, clock, pose, geometry, payload and late
cancellation checks complete on a local proposal. Any refusal preserves the
entire owner. PR35's free coast and immutable reference-vertex regeneration are
reused; previous pressure, viscosity, boundary geometry and data remain intact.
The Lean source proves finite exact-real quadratic and frictionless impulse
algebra under explicit premises. It does not prove the floating-point query,
libm, feature coverage, physical mass distribution or contact continuation.
