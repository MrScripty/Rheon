# 20 From collision meshes to fluid boundaries

A rendered triangle soup is not automatically a closed collision solid. Rheon's proposed boundary pipeline accepts an oriented watertight mesh in a declared world transform, or a deliberately separate thin-shell model. It does not invent an inside for open edges. Static and prescribed rigid motion are the first research targets; deforming meshes require vertex velocities and swept geometry.

## Admission and representation

Validate finite vertices, valid triangle indices, nonzero triangle area at the declared tolerance, edge incidence, coherent orientation and supported topology. Self-intersections invalidate a simple signed-volume interpretation. Nonuniform scale affects distances and normals: transform a normal with the inverse transpose and normalize after checking its length. Negative determinant transforms require orientation handling. Record original topology and transformed geometry hashes.

An unsigned nearest distance comes from the closest point on triangles. Its sign requires an inside/outside policy, such as consistently oriented winding or parity with robust edge/vertex rules. A signed distance used for classification is a derived numerical field, not an authoritative replacement for the mesh. Thin features can disappear when sampled at cell centers; face openings and cell volumes need their own geometry queries.

Bridson's author-hosted mesh-query and geometric-predicate implementations provide relevant original implementation references [E2]. Their availability does not prove Rheon's independently written code. The present companion implements segment/triangle intersection and closed-cube tests directly, with explicit finite tolerances. It does not advertise a production robust-predicate package.

## Earliest path collision

For a segment \(x(t)=x_0+t(x_1-x_0)\), seek the smallest hit \(t\in[0,1]\). A triangle test solves the barycentric system with two in-plane coordinates and segment parameter; parallel or nearly degenerate configurations need a specified classification. Selecting the nearest endpoint distance is insufficient. An endpoint can be outside the solid while the segment crosses it twice.

The original fixture uses a triangulated closed cube, traces an outside-to-outside segment and finds its first hit at \(t=0.3125\). It also checks a miss and a trace beginning inside. The browser shows the complete segment, earliest hit and triangle mesh, so the evidence remains visible rather than hidden behind a texture effect. A boundary control translates the geometry and uses the corresponding regenerated hit records.

At a hit, collision response depends on what is transported. A passive tracer trace may stop at the surface; a liquid interface needs its own boundary/contact-angle extension; a velocity sample needs prescribed normal motion plus the chosen tangential law. Repeatedly clamping a point can pin a contact line unintentionally. No-slide and no-slip mean different operations.

## Checked planar first contact and clipping

The mesh-query fixture and the exact collision contract have different domains. The new `BoundedPhysics.lean` contract uses an infinite stationary plane in three dimensions. Its oriented plane value is

\[
g(x)=\sum_{i=0}^{2}n_i x_i-c,\qquad
x(t)=(1-t)a+tb.
\]

The permitted half-space is g(x)≥0. The normal points into that half-space and need not have unit length; g is a sign/plane-value function, not necessarily a signed distance. Assume strictly g(a)>0 and g(b)<0. These hypotheses exclude a zero denominator and imply

\[
t_* = \frac{g(a)}{g(a)-g(b)},\qquad 0<t_*<1.
\]

`wall_segment_affine` derives \(g(x(t))=(1-t)g(a)+tg(b)\) from the actual finite-vector definitions. `wall_hit_range` proves the range of \(t_*\). `wall_hit_on_surface` proves \(g(x(t_*))=0\). `wall_first_hit` proves every \(0\le t<t_*\) remains strictly in the permitted side. Finally, `clipped_segment_in_halfspace` proves every point on the segment from a to \(x(t_*)\), parameterized by \(0\le s\le1\), has nonnegative plane value, including contact. The hit and clipping conclusions are derived, not assumed.

The exact-rational witness uses plane values 1 and −1, giving \(t_*=1/2\). The unclipped continuation at \(t=3/4\) has value −1/2. If the proposed end instead has value 1/2, the formula gives \(t_*=2\) outside the segment; if the start/end values are −1 and −2, it gives \(t_*=-1\). The endpoint sign assumptions are therefore essential. Starting on the plane, grazing, same-side traces and moving planes require separate policies.

This proves a bounded planar geometric contract. It does not prove triangle containment, ordering among mesh facets, watertightness, cut-cell construction or floating-point robustness. A finite-facet query must establish both that its intersection lies inside the facet and that no earlier relevant facet was missed. The closed-cube browser fixture remains an independently executed numerical query, not an implementation refinement proved by these planar lemmas.

## Cut geometry and pressure

For a face opening \(A_e\), fluid control volume \(V_i\) and sample separation \(\ell_e\), a candidate pressure weight is \(w_e=A_e/(\rho_e\ell_e)\). Positive weights preserve the finite energy structure. Shared openings must be computed once for both adjacent cells. Independent face estimates can violate cancellation. The scalar volume and face metrics must agree with the chosen divergence/gradient pair, not merely look plausible.

Tiny volumes create stiffness and amplify residual-to-divergence scaling. A minimum fraction, cell merging or topology removal is a model policy requiring comparison against unclipped geometry. A binary staircase is a valid educational baseline with a disclosed geometry error; it is not continuous collision-mesh support. Basilisk's original embedded-boundary source exposes how geometry enters gradients, interpolation and small-cell handling [E3].

## Qualification ladder

Compare cube intersections against hand arithmetic, then oblique planes and spheres against analytic distances. Add grazing rays, edge hits, reversed winding, repeated triangles, very small and large scales, and subcell wall offsets. Use refinement to distinguish geometry error from linear-solver error. Validate moving traces in relative coordinates for rigid translation, then rotation against an independent swept reference.

The research admission contract returns `UnsupportedOpenMesh`, `DegenerateTriangle`, `AmbiguousSign` or `UnresolvedThinFeature` where appropriate. It does not silently substitute a box. The finite Lean conservation contracts assume a valid assembled incidence and shared fluxes; they do not verify these geometry predicates. A complete mesh-to-cut-cell implementation remains future production work.

## Mesh validity is part of the numerical method

A render mesh is not automatically a valid collision domain. The conversion should produce an explicit report containing:

- Unit scale, coordinate transform, winding convention, and sign convention.
- Watertightness and manifold checks for volumetric obstacles.
- Degenerate triangles, self-intersections, duplicate faces, and open boundaries.
- Minimum feature/gap scale relative to Δx, particle spacing, kernel support, and collision thickness.
- A declaration that a surface is a finite-volume solid, a zero-thickness shell, or a intentionally thickened approximation.

For a closed solid, a signed distance function can describe inside and outside if its sign construction is valid. A zero-thickness open sheet has no enclosed solid volume. A sub-grid sheet can disappear entirely if an algorithm samples only whether cell centers lie inside it. Inflating it avoids that disappearance but changes displaced volume, passage width, wetted area, and pressure forces.

Guendelman, Selle, Losasso, and Fedkiw's thin-shell method uses ray-based visibility information to prevent interpolation and finite-difference stencils from crossing an infinitesimally thin triangulated barrier. It also treats incompressibility near that barrier. This is the important conceptual precedent: detect which samples may communicate across the shell, rather than treating every nearby sample as a neighbor. [R18]

For cut cells, store fluid cell volumes and open face areas. Also represent connectivity: an internal baffle can split one Cartesian cell into disconnected fluid pieces that cannot share a single pressure unknown without introducing a false connection. Scalar volume fractions alone are not enough for all sub-cell topologies. The correct fallback may be refinement, multi-region cut cells, or rejecting the feature as unresolved.

Small cut cells introduce their own explicit transport restriction. When fluid volume tends to zero faster than an open face area, the local transit time tends to zero. Basilisk's embedded-boundary code documents this issue and implements a redistribution approach for tracer overflow. Any adopted merge/redistribution rule needs separate conservation, positivity, accuracy, and barrier-connectivity checks. Clamping a tiny volume to a larger value is a model change unless the resulting discrepancy is accounted. [R19]

## Moving and deforming walls

For a rigid wall, evaluate local velocity at the actual interaction point:

\[
U_w(x)=V+\Omega\times(x-c).
\]

A single object translation velocity is incorrect for a rotating paddle. Use the same point and frame for wall traction, solid force, and solid torque. For a two-way solid, accumulate equal-and-opposite impulses and the moment arm. For a prescribed wall, accumulate the actuator work; do not report fluid energy growth as a solver defect without including it.

When geometry moves, a cut cell can be uncovered or swallowed. Update swept volumes and associated mass/momentum consistently. For a space-time control volume, a discrete geometric conservation law is the prerequisite for transporting a uniform field without generating a fictitious disturbance. Reinitializing newly uncovered cells by arbitrary extrapolation may be robust, but it is not automatically conservative or work-consistent.

Collision detection for advected particles should consider the segment or swept trajectory, not only its endpoint, whenever a particle can traverse a thin wall in one substep. Surface-only pushout can repair position while leaving pressure fluxes, density estimates, and wall work wrong. Do not use a smoothed rendering normal as the geometric contact normal without analyzing the resulting gap and traction error. Nonuniform scaling also requires correct normal transformation and a recomputed distance metric.

Precomputed rigid-boundary maps may be transformed with a rigid body; arbitrary deformation changes the precomputed geometry. A deforming mesh needs an updated representation and checks for new self-intersections or inverted elements. A narrow band must cover the actual query reach, including kernel support and swept motion, not merely the visible surface.

[R18]: https://graphics.stanford.edu/papers/thin_shells_fluid_coupling-sig05/

[R19]: https://basilisk.fr/src/embed.h


## Implemented bounded static owner

The [static-obstacle geometry feature](../implementation/static-obstacle-geometry.md)
now retains one admitted closed axis-aligned triangle box for collision, cell
volumes, uniquely shared face openings, connectivity and conservative flux.
Its exact-real overlap and complement formulas are checked in
[StaticObstacle.lean](../../../proofs/Rheon/StaticObstacle.lean); Rust evaluates
represented world endpoints with explicit arithmetic refusal. A subcell box
spanning two full axes can split one cell into two fluid pieces, so that topology
is refused. Existing pressure and viscosity owners do not consume this geometry.
The matching lab inspects recorded native controls; it advances no fluid.
