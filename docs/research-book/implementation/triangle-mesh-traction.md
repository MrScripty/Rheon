# Bounded triangle-mesh pressure and traction loads

This successor starts at exact PR31 `ea70bd2729f1042f3812f27a12e7ad6a063d6c2d`.
It adds the roadmap's missing mesh-based force/torque primitive, independently
of the held PR25 case41 and geometry-band campaigns. It changes no fluid step,
pressure, transport, viscosity, collision predicate, geometry input, acceptance
threshold, or PR30/31/32 source. Local qualification is used; checkpoint commits
carry `[skip ci]` to avoid hosted Actions spending. No budget setting changes.

## Physical operator and derivation

The caller supplies world-space corner traction in N/m² on each retained
triangle, or signed corner pressure in Pa. The interpolated traction is P1:
`t(x)=sum_i lambda_i(x) t_i`. Geometry and load corners use the same indexed
triangle order; independent per-triangle values allow jumps between facets.
Pressure means `t_i=-p_i*n`, where `n` is the winding-derived unit normal. The
caller must establish outward solid winding and pressure side where required.
The API does not infer a solid from an open triangle surface.

For triangle area A and its barycentric functions, direct reference-triangle
integration gives `integral lambda_i=A/3`, `integral lambda_i²=A/6`, and
`integral lambda_i lambda_j=A/12` for distinct corners. The consistent forces
and resultant are therefore

```
f_i = integral lambda_i*t = A/12 * (sum_j t_j + t_i)
F   = sum_i f_i
tau = sum_i (x_i-c) cross f_i
```

Here c is the declared moment reference. Because both x and t are affine on a
facet, torque requires the quadratic moments. Applying F at the centroid loses
their covariance. The unit-area triangle `(0,0,0),(2,0,0),(0,1,0)` with corner
tractions `0,(0,0,3),0` has `F=(0,0,1)`, `tau=(1/4,-1,0)` about the origin;
the centroid shortcut would give `(1/3,-2/3,0)` instead.

For a virtual rigid velocity `u(x)=V+omega cross (x-c)`, the same nodal forces
satisfy the exact-real adjoint identity

```
integral t dot u = sum_i f_i dot u(x_i) = V dot F + omega dot tau.
```

Changing the moment reference from c to c+s gives `tau_new=tau-s cross F`;
the equivalent twist has `V_new=V+omega cross s`. These relations check both
the geometry and work sign. World traction is winding-independent after corner
remapping; pressure changes sign when the supplied winding is reversed.

The continuum force and torque targets are Batty, Bertails and Bridson,
[A Fast Variational Framework for Accurate Solid-Fluid Coupling (2007)](https://www.cs.ubc.ca/~rbridson/docs/batty-siggraph2007-variationalcoupling.pdf),
section 3.1, equations 8 and 11. Their coupled discrete operator uses volume weights
consistent with fluid pressure. This P1 surface rule is newly derived here;
it is not their discrete stencil or a matched reaction on Rheon's fluid.

## Borrowed owner, refusal and arithmetic scope

`TriangleMeshLoad` borrows an immutable `TriangleSurface` and an immutable load
slice with a matching caller-owned `SurfaceStamp`. Admission checks count,
triangle work cap, finite moment reference/load values, positive representable
physical area and nonzero A/12, and all intermediate nodal force and torque
operations. Existing ray-query admission alone does not guarantee a physically
representable area. Each admission/reduction pass polls cancellation per facet,
uses fixed-size scratch, allocates no heap payload, and returns no partial owner
or report on failure. Borrowed surface/load allocation is owned by the caller;
this is not a whole-process memory bound.

The reducer returns physical F in N and torque in N m. `triangle_load` exposes
the consistent corner forces so that a caller can consume actual load vectors.
Virtual velocities are caller-supplied test fields, not advanced body state.
Open, duplicate, disconnected or intersecting facets retain `TriangleSurface`
semantics and contribute once per supplied facet; duplicates therefore double
their load. Uniform pressure cancellation is only established for the explicitly
closed, coherently oriented box fixture, not arbitrary admitted surfaces.

Rust uses nearest-rounded binary64. Area is computed from scaled edges; zero
or nonfinite area/weight, detected zero products from nonzero factors, and
nonfinite intermediate products, arms, sums or powers refuse. Conservative
intermediate refusal can reject a mathematically representable final result.
Ordinary rounding/cancellation remains unenclosed. The reported difference
between nodal and rigid virtual power is a diagnostic, not an IEEE theorem,
numerical error bound or stepping authorization. No pressure solve, fluid force
spreading, swept geometry, body mass/inertia, body advancement, adhesion or
two-way impulse is supplied.

## Independent local qualification

The production algorithm uses consistent nodal loads. The independent rational
oracle instead integrates traction, position and rigid velocity at the three
degree-two triangle cubature points `(2/3,1/6,1/6)` and permutations, each with
weight A/3. Thus it tests the quadratic torque/work moments without duplicating
the production formula. All expected values are exact Fractions; native values
are compared at the fixed absolute `1e-12` allowance. Fixture inputs, actual
stdin/stdout, executable/source hashes and exact expected values are written
outside Git. Geometry-only cases include varying/negative traction, oblique
pressure, reflected and anisotropic geometry, translation/reference changes,
winding reversal, affine subdivision, duplicate facets, and a closed box with
affine pressure and a constant pressure offset. These are finite load tests,
not a PDE/refinement roster or mesh-fluid accuracy qualification.

The bounded example accepts at most 64 triangles, 192 vertices and 65536 input
bytes, calls the native owner, and emits the actual consistent loads. The
rendered example shows those native loads on one facet. Lean contracts separate
the exact barycentric-moment premise from the finite algebra/virtual-work proof;
they do not certify the rounded geometry or Rust implementation.
