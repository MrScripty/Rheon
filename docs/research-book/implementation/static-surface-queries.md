# Implementation addendum: static triangle-surface queries

Chapter 20 distinguishes a rendered surface from a collision solid and requires
earliest-path queries before cut geometry, moving boundaries and liquids. This
milestone supplies a reusable Rust query primitive, independent of the force
milestone under review. It branches from accepted main a431d81a. The published
book/PDF, chapters, research references and root proof inventory are retained.

## Surface ownership and scope

`TriangleSurface::new` takes caller-owned world-coordinate vertices and indexed
triangles, a `SurfaceStamp`, and `SurfaceSettings`. Inputs are consumed even on
rejection. It checks finite vertices, index validity, nonempty geometry, retained
buffer capacity and numerically conditioned nonzero facet area. Admitted
geometry is immutable; accessors expose only borrowed slices. Queries carry the
same caller-supplied identity/version. The caller controls identity uniqueness
and supplies a new object/version when geometry changes.

Open sheets, disconnected meshes, duplicate facets and intersecting surfaces are
accepted as two-sided surfaces. Admission does not certify watertightness,
manifoldness, coherent global orientation, self-intersection freedom or a solid
volume. No inside/outside sign is inferred. Imported transforms must already be
applied to the vertices; normals are computed from those world vertices and
winding, not copied from a rendering mesh. Coordinates and endpoints use the
same world frame and length units.

## Segment query and clipping

`first_hit(start, end, cancel)` scans each triangle for a nonzero finite segment.
It returns the smallest computed intersection parameter, triangle index,
position, nonnegative barycentric weights, winding-derived normalized normal,
geometric front/back facing, and surface stamp. Front/back is not a claim that
the path entered/exited a closed solid. Exact parameter ties retain the lowest
triangle index. Cancellation polls per facet and returns no partial result.
`clip_segment` ends the proposed trace at that contact, or preserves its end
when the query finds no hit. It does not impose no-slip, restitution or any
liquid/contact-line law.

For edges e1=b−a and e2=c−a, admission scales by their largest component before
forming e1×e2. This avoids directly squaring/cubing extreme world scales. The
normalized cross-product magnitude must exceed the declared relative tolerance.
The query intersects the segment with this geometric plane, then solves the
two-dimensional barycentric equations after dropping the dominant normal axis.
This avoids cancellation in a nearly singular Gram determinant. Segment/facet
axis bounds provide a simple rejection phase. There is no BVH; query work is
linear in triangle count and adds no heap buffers.

The default dimensionless tolerance is 1e-12; supported settings range from
32 binary64 epsilons to 1e-3. Overlapping coplanar/near-parallel candidates are
ambiguous. Values just outside or inside barycentric/segment boundaries within
the uncertainty band are also ambiguous; exact represented zero/one boundary
values are accepted. One ambiguous candidate rejects the whole query even if
another facet has a hit, since publishing that hit could skip earlier contact.
The policy can conservatively reject queries that stronger predicates resolve.

These are binary64 computed intersections, not exact/adaptive predicates. The
bands do not certify every rounded sign, near-tie ordering, or complete exact
geometric coverage. Position and barycentric reconstruction can differ by
roundoff. Later volumetric/barrier assembly needs its own supported-geometry
and predicate qualification; this API does not claim that obligation is solved.

## Numerical and discrete evidence

Twelve production fixtures cover the book's translated cube, outside-to-outside
crossings, a miss, an inside-start trace, oblique facets, edge/vertex and endpoint
contacts, reversed winding, ordering/ties, open surfaces, ambiguity, cancellation,
capacity limits and rejection. Independent hand arithmetic gives t=0.3125 for
the unshifted cube crossing. Scaled triangle fixtures from 1e-100 to 1e100 and
translated world coordinates exercise the normalized arithmetic. They do not
establish arbitrary-scale robust geometry or physical accuracy.

The separate exact-real `FacetSelection.lean` defines a barycentric point and
valid facet coordinates. It proves nonnegative weights summing to one and
plane membership when all three vertices lie on the plane. Its actual minimum
fold selects an initial/list candidate, is no greater than any input candidate,
and remains in [0,1] when those candidates do. Six public theorems and three
definitions are checked with an eleven-declaration transitive axiom audit,
including generated equations. Complete/correct candidate enumeration, cross
product assembly, winding, tolerances and IEEE execution remain unproved.

The module is additive evidence outside the retained root proof inventory. Its
six statements are not relabeled as part of the book's historical 42 theorems.
The existing fixed-box solvers, simulation stages and comparison formats are
unchanged. Queries are not yet coupled to transport or pressure; coupling only
transport while leaving pressure connectivity unchanged would need a separate
boundary model and acceptance.

```sh
cargo test --locked --no-default-features --test collision_contract
cargo run --locked --release --no-default-features --example surface_queries
```

The example emits nine query records from three immutable shifted snapshots of
the book cube. It reports surface versions, first-hit/clipped positions and
geometric normals. These are static queries, not a moving-wall simulation.
