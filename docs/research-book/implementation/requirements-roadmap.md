# Requirements map and the next solid-fluid geometry contract

This map describes accepted main plus the unmerged PR22–24 stack. The new
native wall/force sequence is qualified within its fixed-column model; the
broader liquid capability is incomplete. Source links below freeze the inspected
PR24 source, so a future implementation cannot silently change these claims.

| User requirement | Actual implementation | Research/teaching coverage | Remaining capability |
| --- | --- | --- | --- |
| Collision meshes | [TriangleSurface](https://github.com/MrScripty/Rheon/blob/50d5e527b46745a9ee9a11cdecca41a32f32b9f3/src/collision.rs) provides immutable two-sided earliest segment queries/clipping and stamps; [translation](translating-surfaces.md) has its own prescribed-motion query scope. Open/disconnected/intersecting surfaces are admitted. | Chapter20 explains shared cut volumes, face openings and connectivity. BoundedPhysics proves infinite-plane crossing/clipping, not arbitrary triangle-solid assembly. | No certified closed-solid admission, inside/outside fluid domain, mesh-derived cut-cell volumes/apertures or consistent pressure/viscous wall coupling. A hit normal is not a solid-fluid topology. |
| Forces | [BodyForce / ForceUnits](https://github.com/MrScripty/Rheon/blob/50d5e527b46745a9ee9a11cdecca41a32f32b9f3/src/forces.rs) supports explicit acceleration/force-density vectors and optional regions in the fixed box; [LiquidStepInputs](https://github.com/MrScripty/Rheon/blob/50d5e527b46745a9ee9a11cdecca41a32f32b9f3/src/liquid_step.rs#L26) borrows them. PR24 adds uniform tangential forcing to the separate constrained slab. | Chapter19 explains external and moving-boundary work; the Poiseuille lab separates fixed-q and fixed-a density controls. | Mesh-based traction, moving-solid displaced volume/work and two-way solid impulse/torque are not supplied by these force APIs. |
| Viscosity | [ViscosityWorkspace](https://github.com/MrScripty/Rheon/blob/50d5e527b46745a9ee9a11cdecca41a32f32b9f3/src/viscosity.rs) assembles symmetric Newtonian strain on a fully filled sealed MAC box with stationary free-slip walls. [step_viscous](https://github.com/MrScripty/Rheon/blob/50d5e527b46745a9ee9a11cdecca41a32f32b9f3/src/liquid_step.rs#L494) retains that domain restriction. PR22–24 qualify a separate layer-uniform fixed slab with Navier/no-slip. | Chapter22 and finite-strain Lean identities distinguish dissipation from stencil correctness; fitted/coupled prototypes retain separate research scopes. | No arbitrary embedded-wall stress, general evolving-free-surface traction, spatially varying/nonlinear viscosity or viscosity/mesh/interface composition in the general carrier. Passing an isolated slab does not remove this gap. |
| Stickiness / adhesion | Finite beta is mechanical slip friction. It has units Pa s/m and is not an adhesive material law. No implemented contact-angle/adhesion evolution is exposed by the slab. | Chapter23 defines wall/interface energies, Young angle and adhesion work; the cap lab selects analytic equilibrium records at fixed volume. | A declared wall energy/contact-angle reconstruction, balanced capillarity and moving contact-line regularization are needed. An attractive force or larger beta cannot substitute for these. |
| Density | [LiquidTransportConfig](https://github.com/MrScripty/Rheon/blob/50d5e527b46745a9ee9a11cdecca41a32f32b9f3/src/liquid_step.rs#L13) uses a constant carrier density and represented density, with equality required in surface modes. Box/slab inertia and nu=mu/rho have explicit units. | Chapter21 includes hydrostatic layers and weighted pressure identities; the forced lab checks density scaling within a single constant material. | No general advected variable-density/two-phase inertia and matching variable pressure/stress coefficients, conservative interphase momentum or density-ratio liquid qualification. |

The [liquid composition](coupled-liquid-transport.md) and fixed-box viscosity
milestones already exist; they are not the remaining requirement or a new
completion claim. Volume/pressure classifications, interval geometry and output
publication are real bounded components. General three-dimensional fluid
topology and coupled physical boundaries remain separate prerequisites. The
unresolved research qualifications and failed numerical evidence are retained;
this book integration executes no new E2 roster or reference integrations.

## Next concrete feature: static closed obstacle geometry for the carrier

Introduce a reusable solid-fluid geometry owner that supplies one immutable
versioned set of fluid cell volumes, uniquely shared face-open areas and fluid
connectivity to the existing MAC carrier. Begin with an admitted axis-aligned
closed box represented as triangles. This crosses the current gap between
collision queries and fluid geometry; it is not another slab parameter sweep.

The first bounded API should explicitly admit a closed oriented box, apply its
world transform that preserves axis alignment and length units, and refuse open, nonmanifold, degenerate,
ambiguous or unsupported non-box geometry before publication. Retain the
existing two-sided surface API as a separate query contract; do not silently
reinterpret every admitted TriangleSurface as a solid. General curved-mesh
cut reconstruction requires a later declared policy and its own qualification.

For this admitted box, compute fluid volume as cell volume minus exact box
intersection volume, and each shared open face area as face area minus box
intersection area. Neighboring cells reference the same face value and opposite
incidence signs. Derive connectivity from positive openings, so sealed pockets
have separate pressure compatibility/gauge components. Geometry, mask, normals
and stamp must describe one coherent owner; output rejection/cancellation leaves
the previous geometry untouched. Report actual allocations and lifetimes.

Acceptance should use independently constructed analytic cube intersections:
fully dry/wet cells, partial cuts, translations across a face/edge/corner,
obstacles touching the container, closed pockets, disconnected fluid regions,
and malformed/ambiguous triangle input. Use exact rational fixture volumes to
check volume sums with a declared floating-point assembly allowance, uniquely shared
area symmetry, component connectivity, units and version/publication behavior.
This first feature qualifies geometry only. It must not advertise a fluid solve
until the pressure assembly/correction consumes those same volumes/openings and
passes rest/flux and component-compatibility gates.

Then qualify stationary impermeable-obstacle pressure on that owner, followed by
viscous wall traction using the same interaction geometry and momentum/work
ledger. Material variation and balanced capillarity/contact-angle reconstruction
come after their coefficient/interface contracts exist. Moving/two-way solids
add swept geometry and reaction/torque obligations. This sequence connects the
remaining mesh, force, viscosity, density and adhesion requirements without
claiming capstone completeness.
