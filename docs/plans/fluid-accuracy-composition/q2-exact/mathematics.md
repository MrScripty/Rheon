# Reference v3: exact face dimension for represented sources

Scope is the finite binary64 input polygon, not an ideal trigonometric disk or an
arbitrary continuum boundary. Map each coordinate to its exact dyadic rational.
For a full-dimensional, convex CCW polygon P, its intersection with a coordinate
line is empty, one point, or a closed interval. Every extremum is a source vertex
on the line or a strict edge crossing. Construct those crossings in rational
arithmetic and take minimum/maximum tangential coordinates; intersect this line
section with the exact represented face span. Strict ordering distinguishes
positive length from zero measure. No magnitude threshold is needed.

The native construction enumerates edge crossings; independent verification uses
all source halfplanes restricted to the line. These are equivalent for an
accepted convex CCW source, with different arithmetic paths. Rational endpoint
witnesses are exported without decimal rounding. Segment endpoints used for
floating integration are converted only after positive exact length is established.
If two endpoints collapse or cannot form a positive finite binary64 length, refuse
construction rather than discard the positive interval. This is a conservative
representation refusal, not a zero-measure classification. The old floating
source orientation/moment checks still precede exact convexity validation and may
conservatively refuse an otherwise valid represented source. No generic exact
source-admission or arbitrary topology claim follows.

For the independent region positivity check, let R=[lx,ux]x[ly,uy] be the actual
represented nondegenerate clipping box. A full-dimensional convex P intersects
its open horizontal slab with nonempty interior iff
max(ly,min_y(P)) < min(uy,max_y(P)). Clip P exactly by the two closed horizontal
halfplanes to obtain Y. Subject to the strict Y overlap, area(P intersect R)>0 iff
max(lx,min_x(Y)) < min(ux,max_x(Y)). For a convex set with interior, the projection
of its interior is the interior of its projection. Both strict tests are necessary;
a point or line contact has zero area. Cache the exact slab X bounds per actual Y
bounds. This certifies only positivity, not the numerical first/second moments.
Those retain the prior independent positive vertical-integration oracle and fixed
finite budgets. Export box bounds alongside their moments as an owned Region;
do not reconstruct rounded dual centers using ideal rational grid coordinates.

Positive interior-face intervals must have two positive adjacent primal regions
and a positive velocity dual in this contained, noncoincident fixed corpus.
Boundary faces must have no positive aperture. These are corpus consistency
checks; they do not select active DOFs, velocity extensions, nullspaces or coupled
operators. A zero aperture can still have a positive dual region.

Primary dependency sources: num-rational 0.4.2's official `from_float` implementation
uses integer mantissa/exponent decoding, and BigRational uses arbitrary precision
integers. https://github.com/rust-num/num-rational/blob/num-rational-0.4.2/src/lib.rs
https://docs.rs/num-rational/0.4.2/num_rational/type.BigRational.html
CGAL's exact-constructions kernel is an established alternative, but its C++/FFI
integration is outside this bounded private Rust reference.
https://doc.cgal.org/latest/Kernel_23/classCGAL_1_1Exact__predicates__exact__constructions__kernel.html
