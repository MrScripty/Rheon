# Prescribed translating-surface queries

Chapter 20's qualification ladder calls for relative-coordinate translating
traces before rotation and swept cut geometry. `TranslationInterval` implements
that geometry prerequisite over one immutable interval. It borrows an admitted
`TriangleSurface`, retaining its original object/version stamp, and records a
caller-owned interval identity, increasing finite physical times and two finite
world-space translation offsets. Units are seconds and metres; the reported
wall velocity is in metres per second. No geometry arrays are copied or allocated.

The point and surface both move linearly over the same parameter s in [0,1]:

\[
x(s)=(1-s)x_0+sx_1,\qquad
q(s)=(1-s)q_0+sq_1,\qquad
r(s)=x(s)-q(s)=(1-s)(x_0-q_0)+s(x_1-q_1).
\]

Query the unchanged reference surface with endpoints r(0), r(1). The smallest
computed static hit parameter is the earliest computed translating contact.
Report world position x(s), physical time (1-s)t0+s t1 and constant prescribed
wall velocity (q1-q0)/(t1-t0). Pure translation preserves geometric normal and
barycentric weights. The `reference_hit` remains explicitly in the unshifted
surface's coordinates; `world_position` is separate. Facing is relative motion
against/along that geometric normal and does not classify solid entry.

This detects a wall sweeping across a stationary world point and a crossing
missed by testing only the final wall pose. It does not approximate a rotating or
deforming trajectory with a linear translation, extrapolate outside the interval,
or change accepted geometry in place. Caller-owned interval-id uniqueness is
separate from the reference geometry version. A new interval describes new motion.

## Numerical and ownership contract

Construction checks finite times, positive finite duration, finite translations,
representable displacement/velocity, and finite world vertex coordinates at both
endpoint poses. A nonzero displacement whose velocity underflows to zero is
rejected, as is overflow. These checks do not certify preservation of small
features after large-offset floating-point rounding. Query validation rejects
nonfinite world/reference endpoints or endpoint differences. A zero relative
segment returns `NoRelativeMotion`: persistent contact and stationary-distance
classification have no policy here. A stationary world point with nonzero relative
motion is supported. Static conditioning and ambiguity errors propagate intact.

Queries borrow the immutable surface and interval, allocate no arrays, and scan
all triangles. Cancellation polls once per reference facet, exposes no partial
hit result, and leaves both reusable. An error after a previously found candidate
also returns no partial result. `clip_segment` stops a point's linear trajectory
at first space-time contact, or retains the endpoint on a definite miss. It does
not prescribe the continuation of a point as a moving wall advances afterward.

Subtractive reference-coordinate rounding, static predicates, near-tie selection
and world reconstruction are numerical operations, not exact/adaptive predicates.
No bounded error or arbitrary-scale guarantee is claimed. This linear scan does
not assemble cut volumes, face openings, connectivity, prescribed wall flux,
pressure work, friction or two-way solid reactions. Original fixed-box transport,
pressure and force stages remain unchanged.

## Exact finite contracts and executed fixtures

`evidence/translating-surfaces/TranslationFrame.lean` defines reference/world
coordinate maps, the physical clock and wall velocity. Seven checked exact-real
lemmas derive the affine relative segment, inverse world reconstruction,
conditional reference-to-world contact, translated plane value, monotonic clock,
clock range and velocity/displacement identity. Increasing times and supplied
parameter/contact hypotheses are explicit. These statements justify the coordinate
reduction conditional on the real model and correct static candidates; they do
not prove the IEEE implementation, candidate completeness, manifoldness, rotational
motion or any fluid update. The separate namespace audit includes all four
specified definitions and all seven theorems. Historical proof inventory and book
PDF inputs are preserved; this addendum is not incorporated into that frozen PDF.

Ten Rust contract methods execute 216 signed, axis-permuted hand-oracle queries,
stationary-point sweeps, final-pose false negatives, shared translating frames,
zero-motion static equivalence, normals/winding, endpoints, clipping, cancellation
and retry, invalid intervals/trajectories and pose overflow. Release example:

```sh
cargo run --locked --release --no-default-features --example translating_surface
```

This is prescribed geometric motion, not a moving-wall fluid or liquid simulation.
