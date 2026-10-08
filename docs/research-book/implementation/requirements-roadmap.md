# Requirements map and the next solid-fluid geometry contract

This map reads the immutable PR36 snapshot `d31cf735645579357f9f58dcc55958e23f77af59`, including the earlier wall/force and bounded obstacle components. The original API links in the table retain their historical PR24 identities. The new
native wall/force sequence is qualified within its fixed-column model; the
broader liquid capability is incomplete. Source links below freeze the inspected
PR24 source, so a future implementation cannot silently change these claims.

| User requirement | Actual implementation | Research/teaching coverage | Remaining capability |
| --- | --- | --- | --- |
| Collision meshes | [TriangleSurface](https://github.com/MrScripty/Rheon/blob/50d5e527b46745a9ee9a11cdecca41a32f32b9f3/src/collision.rs) provides immutable two-sided earliest segment queries/clipping and stamps; [translation](translating-surfaces.md) has its own prescribed-motion query scope. Open/disconnected/intersecting surfaces are admitted. | Chapter20 explains shared cut volumes, face openings and connectivity. BoundedPhysics proves infinite-plane crossing/clipping, not arbitrary triangle-solid assembly. | [StaticObstacleGeometry](static-obstacle-geometry.md) now admits a single closed axis-aligned box and derives shared volumes, openings, labels and conservative flux from the retained surface. General mesh cuts remain absent. A [bounded follow-on](static-obstacle-flow.md) now adds sealed obstacle pressure and a separate reduced extruded-shear response from the same owner; general 3D coupling remains absent. A hit normal alone is not fluid topology. |
| Forces | [BodyForce / ForceUnits](https://github.com/MrScripty/Rheon/blob/50d5e527b46745a9ee9a11cdecca41a32f32b9f3/src/forces.rs) supports explicit acceleration/force-density vectors and optional regions in the fixed box; [LiquidStepInputs](https://github.com/MrScripty/Rheon/blob/50d5e527b46745a9ee9a11cdecca41a32f32b9f3/src/liquid_step.rs#L26) borrows them. PR24 adds uniform tangential forcing to the separate constrained slab. | Chapter19 explains external and moving-boundary work; the Poiseuille lab separates fixed-q and fixed-a density controls. | Mesh-based traction, moving-solid displaced volume/work and two-way solid impulse/torque are not supplied by these force APIs. |
| Viscosity | [ViscosityWorkspace](https://github.com/MrScripty/Rheon/blob/50d5e527b46745a9ee9a11cdecca41a32f32b9f3/src/viscosity.rs) assembles symmetric Newtonian strain on a fully filled sealed MAC box with stationary free-slip walls. [step_viscous](https://github.com/MrScripty/Rheon/blob/50d5e527b46745a9ee9a11cdecca41a32f32b9f3/src/liquid_step.rs#L494) retains that domain restriction. PR22–24 qualify a separate layer-uniform fixed slab with Navier/no-slip. | Chapter22 and finite-strain Lean identities distinguish dissipation from stencil correctness; fitted/coupled prototypes retain separate research scopes. | The same static owner now supplies a strictly admitted [reduced shear channel](static-obstacle-flow.md), with actual cut-layer inertia, traction, forced response and work/impulse ledgers. No arbitrary embedded-wall stress, general evolving-free-surface traction, spatially varying/nonlinear viscosity or viscosity/mesh/interface composition in the general carrier. Passing an isolated slab does not remove this gap. |
| Stickiness / adhesion | Finite beta is mechanical slip friction. It has units Pa s/m and is not an adhesive material law. No implemented contact-angle/adhesion evolution is exposed by the slab. | Chapter23 defines wall/interface energies, Young angle and adhesion work; the cap lab selects analytic equilibrium records at fixed volume. | A declared wall energy/contact-angle reconstruction, balanced capillarity and moving contact-line regularization are needed. An attractive force or larger beta cannot substitute for these. |
| Density | [LiquidTransportConfig](https://github.com/MrScripty/Rheon/blob/50d5e527b46745a9ee9a11cdecca41a32f32b9f3/src/liquid_step.rs#L13) uses a constant carrier density and represented density, with equality required in surface modes. Box/slab inertia and nu=mu/rho have explicit units. | Chapter21 includes hydrostatic layers and weighted pressure identities; the forced lab checks density scaling within a single constant material. | No general advected variable-density/two-phase inertia and matching variable pressure/stress coefficients, conservative interphase momentum or density-ratio liquid qualification. |

The [liquid composition](coupled-liquid-transport.md) and fixed-box viscosity
milestones already exist; they are not the remaining requirement or a new
completion claim. Volume/pressure classifications, interval geometry and output
publication are real bounded components. General three-dimensional fluid
topology and coupled physical boundaries remain separate prerequisites. The
unresolved research qualifications and failed numerical evidence are retained;
this book integration executes no new E2 roster or reference integrations.

## Geometry foundation and bounded operator follow-on

The [bounded geometry owner](static-obstacle-geometry.md) retains an admitted
closed box surface and supplies physical cell volumes, one shared opening per
MAC face, component labels and a conservative flux helper. It refuses unsupported
meshes and disconnected fluid pieces within one cell. The [operator follow-on](static-obstacle-flow.md) borrows this owner for sealed pressure and a separately bounded reduced shear mode. It does not add those operators to the existing carrier or compose their different boundary models. World-space vertices express
translation and axis-preserving scaling; no moving-body work is implied.

Nine recorded native controls and independent rational checks cover wet/dry and
partial cuts, three separator orientations, translations/scaling, container
contact, collision source agreement and shared flux incidence. Three thin baffles
are recorded topology refusals. The non-dyadic fixture has a declared binary64
assembly allowance; exact-real Lean identities do not prove implementation
refinement. Managed buffer capacities and constructor queue lifetimes are
reported with an explicit scope, rather than a whole-program cap claim.

The pressure follow-on checks component compatibility/gauges, full integrated and cellwise residuals, actual corrected fields and a face-area kinetic ledger. The reduced shear follow-on checks owner-derived inertia/traction and a forced momentum/work ledger. The next gate is general embedded symmetric-strain interaction geometry and a composed step under consistent boundaries. Material variation and balanced capillarity/contact-angle reconstruction
come after their coefficient/interface contracts exist. Moving/two-way solids
add swept geometry and reaction/torque obligations. This sequence connects the
remaining mesh, force, viscosity, density and adhesion requirements without
claiming capstone completeness.

## Isolated aligned-box integration experiment

The [newly reconstructed aligned strain](reconstructed-aligned-strain.md) supplies
three normal and three engineering-shear row families on a retained, padded,
exactly grid-aligned internal box. Its [live laboratory](aligned-strain-laboratory.md)
shows finite actions and force work without advancing time. The next smallest
integration is the [experimental aligned Stokes substep](experimental-aligned-stokes.md):
an explicit unforced viscosity proposal followed by pressure on that same
constant-density, sealed/free-slip outer and stationary/no-slip obstacle domain.
It lives outside the production library and requires explicit opt-in.

An outward bound, both actual energy changes, per-face momentum defects and
all wet-cell fluxes govern acceptance. The rounded pressure coefficient is
qualified against the stored-mass reference; no rounded product identity is
assumed. Sound interval arithmetic and the documented default floating-point
environment are explicit premises. Independent rational comparisons and
conditional Lean statements qualify this bounded transaction, not general
liquid transport or PDE convergence. Unsupported arithmetic or unresolved
inequalities refuse the complete caller update.

The separate [triangle-mesh traction reducer](triangle-mesh-traction.md) now
integrates caller-supplied P1 surface pressure/traction into physical force,
torque and consistent corner loads, with rigid virtual-work diagnostics.
It uses the retained triangle geometry, including oblique facets; it supplies
neither a fluid pressure interpolation nor a matched fluid reaction or body step.

General embedded geometry, moving/free interfaces, forcing, variable material,
capillarity and two-way solid coupling still require their own contracts.


## Bounded body progression at PR36

The [mesh load](triangle-mesh-traction.md) now feeds a separate
[linear/angular impulse transaction](rigid-mesh-impulse.md). The next
[spherical-inertia motion owner](spherical-rigid-motion.md) advances an actual
COM clock and quaternion pose, regenerating the retained render/traction mesh
from immutable reference vertices. Mass and spherical inertia remain declared
inputs, not inferred from the mesh. These APIs have no fluid reaction.

The [contact successor](static-sphere-contact.md) uses a separately declared
COM-centered sphere collider against retained finite static triangles. Its
unique earliest transversal event applies one frictionless normal restitution
impulse and stops; misses coast the interval. Initial contact/overlap, ambiguous
features, grazing and near-simultaneous facets refuse atomically. Repeated or
resting contact, friction, gravity, arbitrary moving-mesh collision and fluid
coupling remain absent. These bounded body steps do not discharge the moving
fluid-volume, work and two-way reaction obligations in the table above.
