# CAD and engineering visualization contracts

Research integration for Rheon · 12 October 2026

[Chapter 14](../chapters/14-guidance-rendering.md) is the short reading route. This guide retains the full algorithms, assumptions and proposed lesson designs. The native material inventory below is anchored to the stated main commit; it does not erase the separate browser carrier volume renderer or its instantaneous velocity curves. The chapter's existing sticky 24³ carrier example is not a material/SPH or CAD solver. **The seven lessons in Section 15 are specifications, not seven newly implemented interfaces.**

[Annotated bibliography](../sources/cad-visualization-bibliography.md) · [44 primary sources and 21 code anchors](../sources/cad-visualization-sources.json) · [implementation order](../sources/cad-visualization-implementation-plan.md) · [lesson specifications](../companion/cad-visualization-lessons.json) · [analytic fixture generator](../companion/cad_visualization_fixtures.py)

## Reader's route

Begin with Sections 1–3 if you know fluid animation but are new to CFD. Sections 4–10 explain what each view measures and how to implement it. Sections 11–14 cover scientific honesty, exchange formats, and a practical architecture. Section 15 contains seven interactive lessons with exact expected results. Section 16 is the release checklist. The linked implementation plan gives a short delivery order, and the annotated bibliography explains what each primary source actually establishes. CAD01–CAD44 are local source IDs, distinct from the book's existing citation labels.

This chapter distinguishes verified implementation inventory from proposed extensions. Rheon's current inventory is anchored to commit 2209012bc5f902693d0f695ec1c2d854126af317, as reviewed on 12 October 2026; [the particle implementation](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/particle_fluid.rs#L10) and the other commit-anchored links below provide the implementation citations. Prior discussion supplies the research question, not independent repository evidence. General formulas and visualization methods below do not establish that a particular Rheon mode currently supports them.

## 1. Start with the engineering question

A fluid image and an engineering result answer different questions. A volume image might reveal the location of a transported marker. An engineer may need to know whether a channel carries the requested flow, where a part experiences pressure loading, how much pumping power a constriction requires, or whether a result changes under refinement. All of those require quantities with definitions, coordinates, units, and repeatable measurements.

An effective first engineering workspace can be modest:

1. A geometry view showing the actual computational domain and named boundaries.
2. A movable scalar slice, initially colored by speed.
3. Optional velocity arrows, with a visible scale rule.
4. A pressure slice or pressure profile with an explicit reference convention.
5. A section measurement and a point probe.
6. A time-history chart and conservation diagnostics.
7. Export that allows the same fields to be inspected independently.

These features make a small CPU reference solver useful as a laboratory. They do not require a photorealistic liquid renderer, a general CAD kernel, or a promise of industrial certification. They also do not make an incomplete model complete. For example, thermal visualization requires a computed temperature field and energy model; a visually reconstructed interface does not supply surface tension or liquid–air coupling.

ParaView offers the established reference workflow: scalar slices, contours, glyphs, vector integration, sampling, and charts belong to a processing pipeline whose output is separate from presentation. Use it to check a first exporter and to learn which outputs are useful before rebuilding every operation. [CAD01](https://docs.paraview.org/en/latest/UsersGuide/filteringData.html)

### What “accurate” needs to mean

There are at least four questions:

- **Implementation verification:** does the code implement the stated mathematical method correctly?
- **Solution verification:** is this particular run sufficiently converged in iteration, space, and time for the quantity of interest?
- **Physical validation:** does the model predict relevant experimental observations, within measured uncertainty, for the intended class of problems?
- **Visualization verification:** do the displayed and exported values faithfully represent the accepted solver state?

NASA's verification guidance distinguishes code checks and calculation checks; its validation guidance restricts validation claims to supported applications and experimental comparisons. These are useful distinctions for product wording as well as technical practice. [CAD02](https://www.grc.nasa.gov/www/wind/valid/tutorial/verassess.html), [CAD03](https://www.grc.nasa.gov/www/wind/valid/tutorial/valassess.html)

An analytical Poiseuille profile verifies a laminar-flow calculation. It does not validate every imported valve, a turbulent wake, or a free-surface impact. A low linear-system residual is evidence about that algebraic solve, not a percentage error on drag. A convincing picture provides neither kind of evidence on its own.

### Rheon's present starting point: two distinct data paths {#rheons-present-starting-point-two-distinct-data-paths}

The reviewed implementation has a new bounded **reference-volume particle/SPH preview** and a separate **staggered carrier projection/transport path**. Do not combine their fields as if they describe one coupled state.

The particle preview uses a poly6 density/pressure neighborhood calculation, clamped liquid compression pressure, ideal-gas rho-R-T pressure, symmetric viscosity, same-material liquid cohesion, symplectic Euler, and inelastic box-wall contact. Its public particle view includes position in meters, velocity in m/s, mass in kg, temperature in K, and material ID. Per-particle density, pressure, acceleration, and damping data are presently private. Public report pressure maxima are evaluated before motion and wall-temperature updates, while reported speed refers to the accepted state. Therefore even an apparently simple pressure-versus-speed comparison needs evaluation-stage metadata or an additional synchronized diagnostic evaluation. See [public accessors](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/particle_fluid.rs#L274), [pressure assembly](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/particle_fluid.rs#L483), [motion/contact](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/particle_fluid.rs#L627), and [report assembly](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/particle_fluid.rs#L686).

The rotating emitter is a prescribed source with pressure-head-based speed, midpoint orientation, fixed-volume parcels, and transactional volume credits. See [axis orientation](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/emitter.rs#L34), [prescribed profile](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/emitter.rs#L78), [particle admission](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/particle_fluid.rs#L387), and [accepted publication](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/particle_fluid.rs#L698). The particle source evaluates orientation at the interval midpoint and its own disk profile; the separate carrier source applies its declared forcing profile. It is not a collision solid or a simulated nozzle wall. The liquid compression response and gas absolute-pressure response do not yet establish one common ambient gauge or a CAD wall-pressure map. Particle temperature exists, but that alone is not evidence of a resolved heat-transfer or conjugate-thermal solver. See [material equations of state](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/material.rs#L42).

The separate carrier path has face velocities and a pressure correction on a fixed box. Its pressure correction must not be relabeled as particle pressure, and its native face flux must not be reported as SPH flux. Existing carrier output includes opacity images and step/run records; dimensional flux/wall-gradient export support still needs checking. See [declared carrier exports](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/bin/rheon.rs#L68) and [their implementation](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/bin/rheon.rs#L146).

The native material UI at the inventory commit uses eframe/egui Glow with CPU-projected, depth-sorted voxel faces. Particles are quantized into an appearance view, with ambient material hidden. It has no verified CAD/BREP/NURBS pipeline, liquid reconstructed surface, volume ray marcher, or wall-gradient view. Its immutable history stores preview bytes, report, time, and step, rather than full particle checkpoints. There is no verified public particle CSV, mesh, or surface export in the material UI; that is an inventory finding rather than an explicit prohibition in its API. See [desktop architecture](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/bin/desktop/README.md), [preview payload](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/bin/desktop/history.rs#L15), [appearance filtering](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/bin/desktop/history.rs#L71), [history frames](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/bin/desktop/history.rs#L134), and [projection/depth sorting](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/bin/desktop/viewport.rs#L143).

**Immediate consequence:** first expose quantitative particle inspection and a synchronized, unquantized snapshot/export contract. Speed/material/temperature particle coloring and inventory/source diagnostics are available earlier than pressure slices, complete particle histories, or wall loads. Grid-based methods remain useful for the carrier branch and as independent teaching fixtures, but they are not a shortcut around the particle data prerequisites.

## 2. What is actually stored?

### 2.1 The field dictionary

The following is a proposed field dictionary, not a claim that every field is available.

| Quantity | Meaning and SI units | First useful view | Important qualification |
|---|---|---|---|
| Velocity **u** | Fluid velocity, m/s | Speed slice and arrows | Record lab/body frame; components may be staggered |
| Speed | \|**u**\|, m/s | Sequential scalar map | Magnitude after vector interpolation differs from interpolated magnitude |
| Physical pressure p | Normal isotropic stress variable, Pa | Slice, profile, surface load | State gauge and whether hydrostatic terms are included |
| Kinematic pressure π | p/ρ, m²/s², for the stated reference density | Slice with its real units | Do not label as Pa before conversion |
| Mass density ρ | Mass per physical fluid volume, kg/m³ | Slice if variable | Constant density can conceal all interesting motion |
| Liquid fraction α | Fraction occupied by a phase, dimensionless | Cell slice; interface display | Specify denominator: full cell or available fluid volume |
| Tracer c | Passive marker or species variable | Slice, outlet history | Dimensionless appearance marker is not automatically mass fraction |
| Solid/open fraction β | Available volume or face opening, dimensionless | Geometry diagnostic | Distinct from liquid fraction |
| Vorticity **ω** | ∇×**u**, s⁻¹ | Magnitude slice or component map | Nonzero in simple shear; not a turbulence detector |
| Divergence d | ∇·**u**, s⁻¹ | Diagnostic slice and norms | Use the solver's discrete flux divergence for conservation checks |
| Q, λ₂ | Velocity-gradient invariants, s⁻² | Thresholded isosurfaces | Definition, threshold, and gradient scheme matter |
| Wall shear **τw** | Tangential traction, Pa | Wall map | Requires a valid viscous/wall model and wall derivative |
| Temperature T | Thermodynamic temperature, K | Thermal slice | Requires an energy equation and thermal boundary conditions |

Pressure naming is particularly treacherous. OpenFOAM explicitly distinguishes kinematic and physical pressure, and supports adding or subtracting hydrostatic contributions. Treat that as a reason to carry units and semantics, not as a reason to copy another solver's variable names blindly. [CAD04](https://doc.openfoam.com/2312/tools/post-processing/function-objects/field/pressure/)

Do not use “density” as a collective label for mass density, fraction, tracer, and opacity. Volume-of-fluid methods represent occupancy through a fractional volume; the historical VOF papers describe that geometric role separately from a complete physical model. [CAD05](https://www.sciencedirect.com/science/article/pii/0021999181901455)

### 2.2 Four states a field can have

A field selector needs more than a list of array names. Distinguish:

1. **Native:** directly stored by the accepted simulation state.
2. **Derived:** computed from native data with an identified operation.
3. **Prescribed:** supplied by a boundary condition or analytic example, not predicted.
4. **Unavailable:** not produced or not meaningful in this model.

A pressure array full of zeros in a transport-only mode is not a measured zero-pressure solution. An unsupported wall-shear field should say “not available for this model,” rather than silently show blue everywhere. A stale derived field should not survive a timestep or geometry change under its old label.

### 2.3 A minimal data contract

Every exported snapshot should answer these questions without reading its source code:

- What solver mode, version/commit, configuration, geometry, and run produced it?
- What accepted simulation time and timestep does it represent?
- Which coordinate system, origin, orientation, and length unit are used?
- Where does each component live: cell center, node, face, boundary quadrature point, or particle?
- Does an array include halos, inactive cells, air cells, or other nonphysical values?
- Which material and boundary models are active?
- What do pressure zero, connected-component IDs, and phase fractions mean?
- What diagnostic tolerances were used, and did this state pass its acceptance checks?

Recommended field metadata includes a stable ID, human label, unit dimensions, components, association, spatial offset, validity mask, native/derived/prescribed status, reference frame, value semantics, and source revision. Derived metadata should identify input fields, sampling/gradient method, parameters, and a reproducible cache key.

An analysis result additionally needs the measurement geometry, orientation, weighting convention, normalizing scales, selected boundary IDs, numerical precision, invalid-sample count, and integration method. Save these definitions with the run. A screenshot alone cannot reproduce a pressure-drop value.

## 3. Sampling without lying about the grid

Rheon's particle preview should use the particle-specific reconstruction in Section 3.5. Sections 3.1–3.4 describe the separate staggered carrier route and generic structured-grid analysis. A regular visualization lattice reconstructed from particles is a derived display grid, not evidence that the particle solver has native face fluxes or a pressure-projection lattice.

### 3.1 Cell centers and staggered velocities

In a common uniform MAC arrangement, pressure lies at cell centers and velocity components lie at faces. For a domain origin **o**, spacings (hx,hy,hz), and cell indices (i,j,k):

- Cell center: **o** + ((i+½)hx,(j+½)hy,(k+½)hz)
- x-face speed u: **o** + (i hx,(j+½)hy,(k+½)hz)
- y-face speed v: **o** + ((i+½)hx,j hy,(k+½)hz)
- z-face speed w: **o** + ((i+½)hx,(j+½)hy,k hz)

Their logical dimensions differ: u has (Nx+1,Ny,Nz), v has (Nx,Ny+1,Nz), and w has (Nx,Ny,Nz+1). The MAC layout is designed around discrete pressure gradients and flux divergence; it is not a set of colocated vector samples. Bridson and Müller-Fischer's course notes provide a practical explanation of that arrangement. [CAD06](https://www.cs.ubc.ca/~rbridson/fluidsimulation/fluids_notes.pdf)

For interior visualization, a cell-centered vector can be reconstructed as:

u_c(i,j,k) = ½[u(i,j,k)+u(i+1,j,k)]

with analogous formulas for v and w. This average is suitable only under the stated uniform/interior assumptions. Cut cells, irregular faces, and boundary conditions require a reconstruction consistent with their geometry. Label the output “reconstructed cell-centered velocity.” Preserve the face arrays separately whenever they are needed for conservation-sensitive analysis.

### 3.2 The sampler is shared infrastructure

Implement one tested sampler for slices, probes, seed integration, and export previews. For an array with location offset **δ** in grid units, transform a physical point into its array coordinates:

**q** = (**x**−**o**) / **h** − **δ**.

For each axis, take the floor index and fractional coordinate. Trilinear interpolation combines the eight valid neighboring samples using the product of linear weights. Sample each staggered component at its own offset. This avoids an extra center-resampling stage and retains a clearer relationship to the native representation.

The sampler should return a value plus validity/provenance, not merely a float. Possible failures include outside the domain, inside a solid, absent timestep, unavailable field, and insufficient boundary stencil. Never convert these failures into zero. Handle periodic wrapping deliberately and separately from wall clamping. Clamp only when that behavior is the stated numerical extension; blind clamping creates plausible but fabricated near-wall values.

For a speed display, choose and record whether to compute |interpolate(**u**)| or interpolate(|**u**|). They are different operations. Opposing vectors can average to zero while their magnitudes remain positive. The first is normally the appropriate speed of the reconstructed local velocity; the second describes a smoothed sampled-speed scalar.

### 3.3 Cell-to-point conversion is a transformation

VTK's cell-to-point filter averages contributing cell data onto mesh points. That is a useful display operation, but it changes the represented field. [CAD07](https://vtk.org/doc/nightly/html/classvtkCellDataToPointData.html) On an obstacle boundary, ordinary averaging may blend values across regions that should not communicate. Keep a piecewise-constant “native cells” view beside a smooth reconstructed view for diagnosis.

Use native face fluxes for flux totals. Use native cell-volume weights for integral quantities. Use raw valid data, not a display-decimated volume, for min/max reports. The renderer may upload Float32 while the CPU retains Float64 measurements, but this conversion must be explicit and tested.

### 3.4 A practical sampler test suite

- Constant fields remain constant at every valid sample.
- Linear fields interpolate exactly in the interior of a uniform grid, up to floating-point error.
- Each staggered component samples correctly under anisotropic spacing and a nonzero origin.
- Outside/solid samples are invalid, not zero or extrapolated unless specified.
- Sampling does not bridge two disconnected fluid regions.
- A pressure gauge shift changes displayed p but preserves valid within-component pressure differences.
- UI probes, exported probes, and headless probes return the same values and validity states.

### 3.5 Reconstructing a field from SPH particles

Start by displaying and probing actual particles. A point cloud colored by accepted-state speed or material is more honest than inventing a smooth field before its sampling rules exist. Give selected particles stable IDs, show their unquantized positions and velocities, and distinguish hidden ambient material from material absent in the simulation. A filtered-out particle must not disappear from a physical inventory merely because it is invisible.

A common SPH density estimate is rho(x)=Σ_j m_j W(x−x_j,h). A common volume-weighted scalar interpolant is A(x)≈Σ_j (m_j/rho_j) A_j W(x−x_j,h). A normalized display interpolant divides that sum by Σ_j (m_j/rho_j)W. These are different estimators: the normalized version can preserve a constant sampled attribute but does not by itself provide a conservative mass field or a consistent gradient. Price's SPH review explains the distinction between interpolation and the discretization of the physical equations. [CAD41](https://arxiv.org/abs/1012.1885)

For a conventional three-dimensional poly6 kernel with support radius h,

W(r,h) = 315/(64πh⁹) (h²−r²)³ for 0≤r≤h, and zero otherwise.

The kernel has units m⁻³. This formula is a reference from the particle-fluid literature, not a substitute for inspecting Rheon's exact implementation, radius convention, self contribution, material selection, and normalization. Müller, Charypar, and Gross describe these particle-fluid kernel ideas and a related interactive formulation. [CAD42](https://matthias-research.github.io/pages/publications/sca03.pdf)

For Rheon's first scalar slice, the required new data include each particle's synchronized density and pressure, kernel/support metadata, material rules, and a valid support/coverage indicator. Evaluate the interpolant at slice samples using a spatial hash or cell-linked list. The bin width and neighbor stencil must cover the entire support; boundary particles must not be missed because they lie in adjacent bins. Brute-force sums over tiny fixtures provide an independent reference for a neighbor-accelerated implementation.

Reject nonfinite/nonpositive density where the interpolant divides by density, and report a zero or insufficient support denominator as invalid rather than silently dividing by a convenient epsilon. A spatial index changes candidate search cost, not the kernel definition: it must return the same contributing neighbors and weights as the reference calculation. For a bounded small preview, a CPU-generated slice may be sufficient before any GPU renderer change.

Separate the following fields in both API and UI:

- Native particle density at the solver's evaluation stage.
- Native particle pressure and its material-specific equation-of-state semantics.
- Reconstructed mass density on a display lattice.
- Reconstructed velocity/temperature/pressure attributes, with their weighting rule.
- Kernel support/normalization measure and neighbor count.
- Particle count or occupancy used for appearance.

They can look similar and have very different meanings. In particular, a particle-count image is not mass density when particle masses or parcel volumes differ. A pressure maximum computed before motion should not be placed at a particle's post-motion position without disclosure. Prefer a pure diagnostic re-evaluation at accepted positions/temperatures, with no effect on the accepted simulation, or keep the original stage's positions and arrays together in a distinct snapshot.

Near a free surface or solid boundary, kernel support can be incomplete. Normalizing a display field may hide that deficiency without fixing the physics. Boundary treatments such as Ghost SPH explicitly address this numerical issue; they are additional methods, not effects obtained by drawing a smoother surface. [CAD43](https://www.cs.ubc.ca/~rbridson/docs/schechter_phd.pdf) Do not average across an impermeable wall or blend different material pressure conventions by default. Use material/region masks and explain samples whose support crosses a phase boundary.

For particle-derived vorticity or Q, first decide whether to use a native SPH gradient, a corrected particle gradient, or differences of a reconstructed grid. These have different errors. A coarse display grid adds kernel reconstruction, resampling, and finite-difference errors before the invariant calculation. Label the method and record h, output-grid spacing, neighborhood counts, correction policy, and invalid regions. Do not apply the MAC derivative recipe to a particle occupancy image.

**Particle-based measurements need their own definitions.** Mass inventory is Σm_j over the selected material/region. A sum of reference parcel volumes is a reference-volume inventory; Σm_j/rho_j is a density-based volume estimate and need not equal it. Report these separately. The emitter's transactional volume credits can be checked against admitted parcel volume and pending remainder, but a credit balance is not a measured downstream flow rate.

An approximate Eulerian section flux can be obtained by integrating reconstructed rho*u over a plane. A different estimate counts signed particle mass crossings over a time interval and divides by its duration. Crossing counts require stable particle IDs, complete positions/times, births/deletions, and sufficiently resolved trajectories; multiple crossings between sparse snapshots can be missed. Label the two methods and compare them rather than call either a native finite-volume face flux.

For box-wall reaction, logging the actual contact momentum change can produce an interval-averaged contact-force diagnostic: reaction on wall = −ΣΔ(momentum_particle,contact)/Δt. That is a contact-impulse quantity, not a resolved hydrodynamic wall-shear or pressure distribution. Log pressure/viscosity/body/source/contact contributions separately where the integrator allows it. Excluding emitter momentum or counting gravity as wall contact can make a superficially plausible force balance wrong.

Finally, SPH resolution studies need particle spacing, smoothing support/neighbor behavior, timestep, and boundary treatment to be specified together. Merely making an isosurface finer, or increasing neighbor count at fixed particles, is not a general convergence guarantee. Dehnen and Aly analyze particle disorder, kernel choice, neighbor counts, and pairing stability in SPH; use their findings as a warning against simplistic resolution sliders, not as proof of Rheon's accuracy. [CAD44](https://arxiv.org/abs/1204.2471)

## 4. Scalar slices, contours, and clipping

### 4.1 A slice is a measurement surface

Start with XY, XZ, and YZ slices. Add an arbitrary plane when basic indexing and masking are correct. An arbitrary plane can be defined by a point **x₀** and an orthonormal basis (**e₁**,**e₂**,**n**). Sample at

**x**(a,b) = **x₀** + a**e₁** + b**e₂**.

The resolution of this display grid is independent of the simulation resolution. A 2048-pixel slice through a 32-cell domain does not reveal 2048 independent samples. Offer “show source cells” and retain a tooltip showing the physical coordinate, interpolated value, and contributing cell/location.

**Fast implementation:** upload a scalar 3D texture and sample it on a plane in a shader, with explicit coordinate transforms and an appropriate validity mask. **Reference implementation:** evaluate the same plane on the CPU using the shared sampler. Compare their sampled values before judging image similarity. For very small CPU runs, producing a 2D texture on demand can be simpler and sufficiently fast.

An unlit slice is a good quantitative default. Lighting modulates the very brightness used to read values. If geometry shading is helpful, use it on the surrounding solid, keep the quantitative plane unlit, and make any alternative shading switch obvious.

### 4.2 Slice versus clip

A slice extracts a lower-dimensional cross-section. A clip removes part of a dataset and exposes what remains. They answer different questions and have different memory costs. ParaView's filtering guide explicitly distinguishes them and warns that some volume-subsetting operations convert compact structured grids into larger unstructured representations. [CAD01](https://docs.paraview.org/en/latest/UsersGuide/filteringData.html)

In a native viewer, visual clipping can initially be a shader-plane test rather than a new volumetric mesh. Do not use the clipped display's triangle count or silhouette as a geometric measurement unless actual intersection geometry has been generated and validated.

### 4.3 Contour lines and isosurfaces

A contour line on a 2D slice satisfies f(x,y)=c. An isosurface in 3D satisfies f(x,y,z)=c. They are level sets of a chosen scalar, not automatically material boundaries. A pressure isosurface is not a solid wall; a Q isosurface is not a material tube; an α=0.5 surface is a display interpretation of a phase-fraction field.

Marching-cubes-style extraction classifies a grid cell's corner values relative to the level, intersects crossing edges, and emits triangles. Interpolation on an edge from values f₀ to f₁ uses t=(c−f₀)/(f₁−f₀). Specify tie handling when a value equals the level, degenerate cases, consistent shared-edge indexing, and a topology policy for ambiguous configurations. The source samples must be meaningfully located at the extraction lattice vertices; cell-centered data needs a declared reconstruction or an equivalent shifted lattice.

For a mature CPU implementation, VTK's Flying Edges is a useful reference. Its edge-oriented multi-pass structure counts output, allocates once, and avoids repeated edge intersections; its documentation also warns that degenerate triangles can occur. Do not promise published speedups on Rheon's data without benchmarking them. [CAD08](https://vtk.org/doc/nightly/html/classvtkFlyingEdges3D.html), [CAD09](https://www.kitware.com/really-fast-isocontouring/)

### 4.4 Liquid surfaces deserve a separate label

If the numerical method has a native geometric interface reconstruction, exporting it can preserve more of the method's meaning than contouring a smoothed fraction. If it has only fractions, an isosurface is still useful, provided it is labeled as a display surface and its threshold is shown.

Compute liquid volume from native fractions and their documented volume denominator. Do not infer it from the volume enclosed by a display surface. For example, if α is the fraction of the available fluid portion of a cut cell, V_liquid=Σ αᵢ βᵢ V_cell,ᵢ. If α already measures liquid relative to the entire cell, multiplying by β again is wrong. This is a data-contract decision, not a rendering preference.

Smoothing, closing holes, decimation, and normal smoothing can improve appearance while changing shape. Preserve the unmodified extracted mesh for measurement, and describe processed meshes as display derivatives. A reconstructed free surface cannot compensate for absent gas pressure, surface tension, contact-angle, gravity, or interface-advection physics.

## 5. Direction: arrows, streamlines, pathlines, and streaklines

### 5.1 Velocity glyphs

Place arrows on a sparse lattice, a slice, or selected points. Use stable deterministic sampling so arrows do not flicker randomly as time changes. Store whether length is proportional to speed, normalized, capped, or logarithmic. A normalized arrow plot is a direction plot, not a speed plot. If arrows use fixed length, color by speed and make that convention visible.

On a slice, distinguish the full 3D velocity from its in-plane projection:

**u_parallel** = **u** − (**u**·**n**)**n**.

A nearly zero projected arrow may indicate strong out-of-plane flow. Show the normal component separately or offer full 3D glyphs. A streamline of the projected slice field is not necessarily the intersection of a 3D streamline with that slice.

Glyph instancing is preferable to creating a separate object for every arrow. Display limits should be configurable, but decimation should not affect numerical statistics. At zero speed, omit orientation or show a neutral marker; normalizing a zero vector creates NaNs and arbitrary directions.

### 5.2 Three curves, three meanings

The classical distinctions are essential in unsteady flow. MIT's fluid-mechanics notes and NASA's unsteady-visualization work explicitly separate them. [CAD10](https://live.ocw.mit.edu/courses/16-01-unified-engineering-i-ii-iii-iv-fall-2005-spring-2006/c37f038e7f899163071e1b9914a66f8d_f08_new_fall.pdf), [CAD11](https://www.nas.nasa.gov/assets/nas/pdf/techreports/1995/nas-95-017.pdf)

- **Streamline:** a curve tangent to **u**(**x**,t₀) with time frozen at one instant. An arc-length parameterization is d**x**/ds=**u**/|**u**|.
- **Pathline:** the trajectory of one ideal passive particle through changing fields, d**X**/dt=**u**(**X**,t), from a specified release time and point.
- **Streakline:** at observation time t, the locus of particles released from a specified source at different earlier times.

For a steady velocity field these curves can coincide. Recomputing streamlines for successive snapshots and playing them as a movie still does not make them particle histories. Likewise, a “travel time” computed along a frozen streamline is a frozen-field estimate, not a prediction through an unsteady run.

### 5.3 A trustworthy streamline integrator

For a small CPU reference implementation, begin with RK2 midpoint integration and then add RK4 or adaptive RK45 if needed. Keep a testable fixed-step implementation as a baseline. VTK's stream tracer offers these integrator choices and records termination reasons; it uses arc-length-oriented stepping and configurable limits. [CAD12](https://vtk.org/doc/nightly/html/classvtkStreamTracer.html)

An arc-length RK2 step is conceptually:

1. Sample a valid velocity at x and reject speeds below ε_speed.
2. Let k₁=u(x)/|u(x)|.
3. Sample at x+½h k₁; normalize to k₂.
4. Propose x_new=x+h k₂.
5. Check the segment against fluid-domain boundaries, not only its endpoint.
6. Append a point and continue, subject to length, step, error, and validity limits.

Set h relative to local cell dimensions and geometry features, not camera zoom. A step that starts and ends in fluid can still cross a thin solid wall. A robust implementation uses segment traversal/intersection or a conservative step bound, and rejects or shortens a crossing step. Do not reflect a particle off a wall unless that is the intended model.

Store termination reasons such as left domain, hit wall, stagnant, invalid stencil, maximum arc length, maximum steps, or tolerance failure. A truncated curve with an unexplained endpoint can be misread as flow stagnation.

For adaptive methods, tighter integration tolerance cannot repair a coarse or inconsistent interpolated velocity field. Refine both the streamline integration and the underlying field separately. Record seed coordinates, direction, field snapshot, integrator, step units, tolerance, and termination limits with a saved view.

### 5.4 Time-dependent integration

Pathlines require at least two bracketing snapshots for simple time interpolation, actual timestamps, and consistent geometry/frame conventions. A basic interpolation is

**u**(**x**,t)=(1−θ)**u**(**x**,tₙ)+θ**u**(**x**,tₙ₊₁), θ=(t−tₙ)/(tₙ₊₁−tₙ).

Use enough snapshots to resolve relevant motion; temporal interpolation cannot recover events that were never saved. Do not continue past a missing time interval by pretending the field is steady. If mesh geometry changes, transform or locate samples on the appropriate geometry at each time. VTK's particle tracer explicitly handles unsteady vector fields and distinguishes mesh-over-time behaviors. [CAD13](https://vtk.org/doc/nightly/html/classvtkParticleTracer.html)

For streaklines, release particles at defined physical-time intervals and integrate each to the common observation time. Seed density represents your visualization choice unless weighted injection rates are explicitly modeled. Ideal tracer paths neglect inertia, buoyancy, finite-size collisions, and diffusion. They should not be labeled droplets, sand, or bubbles without the necessary dynamics.

### 5.5 Dense direction textures

Line integral convolution (LIC) can reveal a dense 2D direction pattern by filtering noise along the vector field. It is a useful optional slice mode when arrows become cluttered. It does not inherently show direction sign or speed, and an in-plane LIC inherits the projection caveat. Cabral and Leedom's original method is the primary reference. [CAD14](https://www.osti.gov/servlets/purl/10185520) Keep LIC after the simpler, testable arrow and streamline tools; it adds visual density rather than a new engineering measurement.

## 6. Pressure, flow rate, and losses

### 6.1 Pressure needs a reference

In an incompressible pressure solve, an additive constant may be undetermined. The implementation usually fixes a reference pressure, constrains a mean, or anchors a degree of freedom. If multiple sealed disconnected regions have independent gauges, subtracting their pressures does not yield a physical pressure difference without an additional convention.

Show pressure kind, units, reference, connected component, and any hydrostatic transformation near the legend or in the field details. Negative gauge pressure is not by itself a vacuum or cavitation prediction. Cavitation needs an appropriate absolute-pressure interpretation and phase-change model. A projection multiplier may also differ from the pressure one intends to use for mechanical loads; verify its temporal and physical meaning in the solver.

For constant density, physical pressure p and kinematic pressure π are related by p=ρπ. If the solver nondimensionalizes, first use its complete reference scales. Multiplying a dimensionless multiplier by a convenient density is insufficient. OpenFOAM's documented pressure conversions are useful examples of explicit rather than implicit semantics. [CAD04](https://doc.openfoam.com/2312/tools/post-processing/function-objects/field/pressure/)

### 6.2 Section flux

For a stationary oriented section A:

Q = ∫A **u**·**n** dA, with units m³/s.

Mass flow is ṁ=∫A ρ**u**·**n** dA. Only for a known constant density does ṁ=ρQ. For a moving measurement surface, use the relative normal velocity (**u**−**v_surface**)·**n** for flux through that surface. Phase flux requires the phase-specific numerical flux or a stated reconstruction, not an undocumented multiplication of tracer by total flux.

On a face-aligned section of a finite-volume/staggered method:

Q_h = Σf s_f A_open,f u_n,f,

where s_f accounts for the chosen section orientation and A_open is the actual fluid aperture. If the solver already stores integrated volume flux q_f, sum q_f; do not multiply by area again. If it stores area-weighted momentum or a mass flux, do not relabel it as velocity.

This aligned measurement is the best first feature because it can share the solver's discrete conservation definition. OpenFOAM's surface analysis distinguishes sums, area integrals, weighted averages, and normal integrals; copying the right reduction is more important than copying its interface labels. [CAD15](https://doc.openfoam.com/2312/tools/post-processing/function-objects/field/surfaceFieldValue/)

For an arbitrary plane, intersect each active cell or fluid subcell with the plane; form nonoverlapping polygons; triangulate consistently; and integrate the reconstructed normal velocity with an identified quadrature. Clip against the real fluid region and report total sampled/open area. Use analytic uniform/linear fields to verify quadrature and an axis-aligned limit test to compare against native flux. An arbitrary-plane estimate can differ from native discrete flux even when both algorithms are implemented correctly.

### 6.3 Averages and pressure drop

Area mean:

p̄_A = (∫A p dA)/(∫A dA).

Forward-flow-weighted mean:

p̄_Q+ = (∫A p max(**u**·**n**,0)dA)/(∫A max(**u**·**n**,0)dA).

A signed flux-weighted mean uses **u**·**n** without max, and can become unstable when inflow and outflow nearly cancel. Report forward, reverse, and net flow separately in recirculating sections. Avoid silently switching between signed and absolute weights. A denominator near zero should produce an unavailable measurement with a reason.

Define pressure drop as Δp=p̄_in−p̄_out using explicitly stored sections and a common gauge. State whether p is static pressure or a total-pressure quantity. A speed change in an ideal flow can change static pressure without an irreversible loss. Under an appropriate incompressible interpretation, p_total=p+½ρ|u|²; a loss coefficient requires a defined total-pressure difference and reference dynamic pressure. Treat these as model-qualified measurements, not universal labels.

Hydraulic power is Δp Q under the stated steady incompressible device interpretation and consistent section convention. It is not automatically motor electrical power; efficiency and other losses are separate. A nondimensional loss coefficient K=Δp_loss/(½ρU_ref²) must retain its U_ref, density, and pressure-loss definition.

### 6.4 Conservation views

For a closed control volume, compare change of stored mass with boundary flux and sources. For an incompressible fixed geometry, show native flux divergence and net boundary balance. In cut cells, the divergence denominator is the active volume appropriate to the method. An extreme value in a tiny active volume deserves investigation; hiding tiny cells in a display must not erase their contribution from totals.

Present absolute and normalized discrepancies. A percentage normalized by a nearly zero net flow is meaningless. Use a declared scale such as total throughflow, initial inventory, or reference flow, with a small-denominator policy. Conservation is necessary evidence, but a conservative wrong model can still give a wrong engineering answer.

## 7. Loads on parts: pressure, shear, force, and torque

### 7.1 Fix the normal convention first

Let **n_b** point outward from a solid body into the surrounding fluid, and let fluid stress be σ=−pI+τ. The force exerted by fluid on the body is

**F** = ∫S [−p**n_b** + τ**n_b**] dA.

Moment about **x₀** is

**M**(**x₀**) = ∫S (**x**−**x₀**)×[−p**n_b**+τ**n_b**] dA.

If using normals outward from the fluid domain instead, reverse the appropriate sign. A closed body's outward normals should satisfy Σ A_f **n_b,f**≈0; uniform pressure should yield near-zero net force and moment on that closed surface. This is a powerful geometry and sign test, not a universal zero-load rule for an open patch.

OpenFOAM's force utility separates pressure, viscous, and optional porous contributions and records coordinate/moment conventions. That separation is an appropriate model for a trustworthy UI. [CAD16](https://doc.openfoam.com/2306/tools/post-processing/function-objects/forces/forces/)

### 7.2 Viscous stress and wall shear

For a Newtonian incompressible fluid of dynamic viscosity μ:

τ = μ(∇**u** + (∇**u**)ᵀ).

More general compressible or modeled stresses need their own constitutive definition. At a wall, tangential traction is

**τw** = (I−**n_b****n_b**ᵀ)(τ**n_b**).

Do not estimate this simply by coloring the nearest interior speed. Wall gradients depend strongly on spatial resolution, wall position, slip/no-slip conditions, ghost reconstruction, and any turbulence wall model. A correct-looking bulk velocity profile can still yield poor wall stress.

Some solvers expose stress divided by density. OpenFOAM's wallShearStress documentation is a concrete warning: the listed field can have m²/s² units rather than Pa. Inspect the actual model/field dimensions before converting or comparing. [CAD17](https://doc.openfoam.com/2312/tools/post-processing/function-objects/field/wallShearStress/) Its wall-function documentation also makes near-wall modeling an explicit numerical choice. [CAD18](https://doc.openfoam.com/2312/tools/processing/models/turbulence/ras/wall-functions/)

If Rheon's verified model supports pressure projection but not a viscous wall treatment, offer pressure inspection and perhaps a carefully qualified pressure-only load experiment. Do not call it total drag. If the geometry is a voxel approximation, integrate on the method's actual boundary representation, or clearly label a transferred CAD-surface load as approximate.

### 7.3 CAD surface coloring versus load integration

There are three different objects:

1. The original design surface.
2. The boundary actually used by the numerical method.
3. A display mesh used for rendering.

They may differ. A transfer map can color the display CAD mesh from nearby fluid samples, but every sample needs a valid source region, distance, and extrapolation policy. Such a map is not automatically a conservative force transfer. For integrated loads, prefer solver boundary faces/quadrature and preserve patch IDs. Avoid duplicate faces, hidden backfaces, and inconsistent winding.

Constant gauge shifts cancel over a truly closed consistently oriented surface, but generally do not cancel over an open patch. Therefore a patch pressure force requires an explicit pressure reference or opposing-side pressure. Reporting the load on one side of a plate without the other pressure may not answer the structural question.

### 7.4 Drag, lift, and coefficients

Choose orthonormal directions **e_D**, **e_L**, and a moment axis. Then D=**F**·**e_D**, L=**F**·**e_L**, and

C_D = D/(½ρ_ref U_ref² A_ref),

C_L = L/(½ρ_ref U_ref² A_ref),

C_M = (**M**·**e_M**)/(½ρ_ref U_ref² A_ref L_ref).

No coefficient is meaningful without its reference density, speed, area, length, axes, and moment origin. Reject zero reference scales. OpenFOAM's forceCoeffs implementation makes those inputs explicit and checks them; it is a useful implementation precedent. [CAD19](https://api.openfoam.com/2506/forceCoeffs_8C_source.html)

When a result is two-dimensional or extruded, state whether force is per unit span (N/m) or for a finite span (N). Do not compare them directly. For unsteady flows, show both history and a clearly defined averaging interval. Averaging after transients and estimating sampling uncertainty are additional tasks, not rendering options.

## 8. Rotation, deformation, vorticity, Q, and λ₂

### 8.1 Begin with the velocity gradient

Define A_ij=∂u_i/∂x_j. Its symmetric and antisymmetric parts are

S=(A+Aᵀ)/2, Ω=(A−Aᵀ)/2.

S describes local strain rate; Ω describes local rigid-rotation content. Vorticity is ∇×**u**. All entries of A, S, Ω, and **ω** have s⁻¹ units. A uniform translational velocity has zero gradient. A rigid rotation **u**=(−a y,a x,0) has **ω**=(0,0,2a).

On a regular interior grid, centered differences are a reasonable reference derivative. Near walls and cut cells, use the method's boundary-aware reconstruction or an explicitly separate diagnostic approximation. Never use screen-space differences of a rendered image. A derivative amplifies noise and interpolation artifacts; smoothing before differentiation changes the analysis and must be recorded.

### 8.2 Why vorticity is insufficient

Simple shear **u**=(γy,0,0) has nonzero vorticity (0,0,−γ), although local shear can dominate the interpretation. A vorticity-magnitude isosurface highlights rotational/shear regions but does not prove a coherent vortex or turbulence. Prefer a beginner explanation with these simple fields to a gallery of dramatic but unexplained vortex tubes.

### 8.3 There are two commonly conflated Q formulas

The rotation-minus-strain quantity is

Q_rot = ½(‖Ω‖_F²−‖S‖_F²).

The second invariant of A is

Q_inv = ½[(tr A)²−tr(A²)]
      = Q_rot + ½(∇·**u**)².

For divergence-free flow they coincide. With compressibility or numerical divergence they do not. OpenFOAM documents its Q as the second invariant. [CAD20](https://doc.openfoam.com/2212/tools/post-processing/function-objects/field/Q/) A Rheon field should name its convention rather than assume every “Q” from another program matches it.

Positive Q_rot marks regions where the local rotation norm exceeds the strain norm under this definition. It is a local criterion, not a unique or universal definition of a physical vortex. Choose a nonzero threshold based on the problem, and display it with units. For cross-case comparison one can also define Q*=Q L_ref²/U_ref², provided those scales are explicit and fixed.

### 8.4 λ₂

Form the symmetric matrix B=S²+Ω² and order its real eigenvalues λ₁≥λ₂≥λ₃. The λ₂ criterion identifies a region with λ₂<0; visualizations often choose a particular negative isovalue. Jeong and Hussain's original paper defines this approach; OpenFOAM's implementation description confirms the middle-eigenvalue operation. [CAD21](https://www.cambridge.org/core/journals/journal-of-fluid-mechanics/article/abs/on-the-identification-of-a-vortex/D26006DDB95FB28DA80E28A581182DF1), [CAD22](https://api.openfoam.com/2606/classFoam_1_1functionObjects_1_1Lambda2.html)

Use a tested symmetric 3×3 eigenvalue algorithm. Sort explicitly, handle near-degenerate eigenvalues, and check against analytically known tensors. In general, Q and λ₂ need not select identical regions. Thresholds have s⁻² dimensions. Their invariance to uniform translation does not make them invariant to arbitrary rotating observer frames; document the velocity reference frame.

### 8.5 Exact gradient fixtures

These values are analytical consequences of the definitions above, intended as implementation tests:

| Velocity field | Divergence | Vorticity | Q_rot | Q_inv | λ₂ |
|---|---:|---|---:|---:|---:|
| (U,0,0) | 0 | (0,0,0) | 0 | 0 | 0 |
| (−a y,a x,0) | 0 | (0,0,2a) | a² | a² | −a² |
| (γ y,0,0) | 0 | (0,0,−γ) | 0 | 0 | 0 |
| (a x,−a y,0) | 0 | (0,0,0) | −a² | −a² | a² |
| (a x,a y,a z) | 3a | (0,0,0) | −1.5a² | 3a² | a² |

The last row deliberately violates incompressibility and catches the Q-convention bug. Add a constant vector to any row: the gradients and invariants should not change. Rotate both coordinates and vectors consistently: scalar invariants should remain the same within tolerance.

## 9. Geometry and mesh boundaries are part of the result

A CAD viewer is not yet a CFD geometry pipeline. A rendered triangle surface may contain leaks, self-intersections, duplicate faces, wrong normals, tiny slivers, or disconnected pieces. A topologically valid mesh can still have geometric defects. CGAL's polygon-mesh documentation makes this distinction and provides separate operations for intersection tests, connected components, normals, repair, and remeshing. [CAD23](https://doc.cgal.org/latest/Polygon_mesh_processing/index.html)

For a voxel/cut-cell reference solver, show:

- Domain box, coordinate triad, and physical dimensions.
- Cell resolution and nonuniform/anisotropic spacing where present.
- Solid mask, open volume, and face apertures.
- Inlet, outlet, wall, symmetry, periodic, and prescribed-velocity regions.
- Connected fluid components and pressure-reference components.
- Geometry approximation relative to the imported/display surface.
- Excluded or invalid cells and the reason for exclusion.

Preserve stable boundary IDs through processing. Names like inlet_A and wall_blade should connect boundary conditions, saved measurements, selected loads, and reports. A geometry edit may invalidate a section or surface selection; require remapping or mark it stale instead of silently attaching it to a different face.

A diagnostic cut plane showing cell occupancy and the numerical wall beside the smooth CAD outline is often more useful than transparent glass geometry. It reveals a passage that disappeared under voxelization, a wall one cell too thick, or an unintended connection. Those can dominate results even when the rendering looks excellent.

Do not automatically “repair” engineering geometry by closing every hole: an opening may be an inlet. Any topology-changing preprocessing belongs in a visible provenance record with before/after dimensions, region counts, and user-reviewable changes.

## 10. Probes, profiles, histories, and design comparison

### 10.1 Point and line probes

A point probe should report position, time, field, units, association, sampling method, and validity. A line probe should save endpoints, physical arc length, sample count, and sampling policy. Gaps outside fluid should remain gaps. Connecting a chart line across a wall or missing interval creates false continuity.

Probe count is a sampling control, not a simulation-resolution control. Sampling 10,000 points from a 32-cell profile produces a smoother chart, not more resolved flow. Offer a native-sample overlay where practical. Point samples exactly at a discontinuity or wall need a specified side/limit; “the value at the wall” may be a boundary quantity rather than an interpolated interior quantity.

### 10.2 Time histories

Use physical time on the horizontal axis. Store timestep and accepted-state IDs, because output frames may be unevenly spaced. Separate solver iteration histories from physical-time histories. Pressure residual versus Krylov iteration is not pressure at a sensor versus seconds.

Recommended first histories are section flow, section pressure difference, total liquid volume, native conservation error, probe pressure, and a chosen velocity component. Later, add forces and statistical quantities only when supported.

For irregular times, a time mean needs time weighting. For example, use trapezoidal integration of a scalar history divided by the duration, or a stated piecewise-constant convention. An arithmetic mean of output frames overweights densely saved intervals. Report averaging start/end times, sample count, missing intervals, and whether the startup transient is excluded.

An RMS fluctuation, standard deviation, and uncertainty of the mean are different quantities. A long correlated time series does not provide as many independent samples as timestamps. Do not call min/max envelopes or a resolution spread a confidence interval without a defined statistical procedure.

### 10.3 A measurement definition is reusable state

Make section and probe definitions first-class records. A design comparison should reuse the same physical interpretation, even if a surface's numerical representation changes. Save:

- Common operating conditions and material parameters.
- Reference length, speed, area, pressure, and coordinate axes.
- Probe/section geometry and correspondence rules.
- Field and averaging definitions.
- Time window and transient treatment.
- Mesh/timestep/convergence evidence.
- Common color ranges and camera settings where visual comparison matters.

The essential output is a defensible change in a quantity of interest, together with its uncertainty. “The second plot is redder” is not a design conclusion if the legends differ.

## 11. Honest color and numerical uncertainty

### 11.1 Color policy

Use a sequential map for ordered nonnegative quantities such as speed or fraction. Use a diverging map when a physically meaningful central value matters, such as a signed velocity component or pressure deviation from a defined reference. Use qualitative colors for boundary IDs and connected components. Avoid a rainbow default whose unequal perceptual transitions invent apparent bands. Moreland's color-map research explains this failure mode and offers carefully designed alternatives. [CAD24](https://www.kennethmoreland.com/color-maps/ColorMapsExpanded.pdf)

Lock ranges across time and compared runs by default. Provide explicit alternatives: current frame, selected time window, whole run, or manual. Show the selected policy. Mark under/over-range values rather than silently making them indistinguishable from endpoints. Give NaN/invalid data a distinct treatment. Log mapping needs an appropriate strictly positive domain; symmetric-log transforms must show their definition and linear threshold. ParaView's transfer-function controls demonstrate that color, opacity, and range-rescaling are separate choices. [CAD25](https://docs.paraview.org/en/latest/ReferenceManual/colorMapping.html)

Every exported figure should include field, component, units, physical time, range, orientation/section location, and whether data are native or reconstructed. Do not let the camera crop the only scale information. Provide a numerical readout so color is never the only accessible channel. Moreland's current advice also notes the tension between scalar luminance and 3D shape shading. [CAD26](https://www.kennethmoreland.com/color-advice/)

### 11.2 What to display about reliability

Put numerical diagnostics near the main result rather than in an obscure debugging console:

- Solver acceptance/rejection and residual definition.
- Native divergence norms and boundary flux balance.
- Fraction/tracer bounds and any corrections/clamping.
- Minimum active cell volume and geometry-resolution warnings.
- Spatial and temporal resolution.
- The latest available refinement comparison for the selected quantity.

Do not use a single green “accurate” badge. Prefer statements such as “flow measurement agreed within X under this tested refinement” once such evidence actually exists. A result without a refinement study can still be explored, but it should not inherit an unrelated benchmark's error estimate.

### 11.3 A compact refinement derivation

For a smooth quantity f(h)=f*+C h^p in an asymptotic regime, use three systematically refined grids h₁<h₂<h₃ with common ratio r=h₂/h₁=h₃/h₂. If differences have a consistent sign and the assumptions hold:

p_obs = ln| (f₃−f₂)/(f₂−f₁) | / ln r,

f_extrap = f₁ + (f₁−f₂)/(r^p_obs−1).

A fine-grid GCI-style estimate is F_s |(f₁−f₂)/f₁|/(r^p_obs−1), often expressed as a percentage. NASA's tutorial describes the asymptotic assumptions, recommends three levels, and gives a safety factor of 1.25 for studies using three or more grids. [CAD27](https://www.grc.nasa.gov/www/wind/valid/tutorial/spatconv.html)

Do not apply this formula mechanically to oscillatory/nonmonotone convergence, changes of physical regime, discontinuities, unaligned geometry changes, a near-zero f₁, or a chaotic instantaneous field at mismatched phase. Report the raw refinement values first. Compare suitable statistics for genuinely unsteady/turbulent cases. Refine timestep separately from grid where feasible; otherwise their errors can mask one another. NASA's space-time assessment work is a useful primary example of coupled refinement planning. [CAD28](https://www.nas.nasa.gov/assets/nas/pdf/papers/housman_j_space_time_accuracy_2011.pdf)

Experimental validation needs experimental uncertainty and matched conditions. Keep numerical, experimental, input, and model-form uncertainty distinct. A small discretization estimate is not a bound on all physical error. [CAD03](https://www.grc.nasa.gov/www/wind/valid/tutorial/valassess.html)

## 12. Volume rendering and surface rendering

### 12.1 Two fundamentally different representations

**Surface rendering** draws explicit geometry: a CAD boundary, section plane, isosurface, or streamline tube. It is well suited to geometry context and specific level sets.

**Direct volume rendering** accumulates color and opacity through a 3D scalar field without first constructing an explicit surface. Its appearance depends on the transfer function, ray sampling, and compositing. NVIDIA's volume-rendering chapter provides the emission–absorption interpretation and practical sampling considerations. [CAD29](https://developer.nvidia.com/gpugems/gpugems/part-vi-beyond-triangles/chapter-39-volume-rendering-techniques)

Neither representation reconstructs missing data. A dense 3D volume can contain little engineering information if its scalar is just appearance tracer. A transparent volume may hide a small high-pressure region behind a broad opaque foreground. Pair volume views with quantitative slices and probes.

### 12.2 A minimal ray marcher

For each pixel, intersect the viewing ray with the volume bounds. Restrict the ray interval using clipping planes and opaque-scene depth. At sample points, reconstruct scalar f, map it to color **c** and opacity α, then composite front to back:

**C** ← **C** + (1−A) α **c**,

A ← A + (1−A) α.

Here **C** is accumulated premultiplied color. Stop early when remaining transmittance is negligible. If the reference opacity α₀ was defined for distance Δs₀, preserve the corresponding absorption under a new step length using

α(Δs)=1−(1−α₀)^(Δs/Δs₀).

This prevents a resolution/step-size change from arbitrarily changing the apparent opacity. The formula is the discretized exponential attenuation relationship. Keep transfer-function units and reference sample length in the saved view. NVIDIA's implementation discussion stresses opacity correction when sampling rate changes. [CAD29](https://developer.nvidia.com/gpugems/gpugems/part-vi-beyond-triangles/chapter-39-volume-rendering-techniques)

Depth clipping must use the same camera transforms and depth convention as the opaque geometry. Otherwise fluid can appear through a wall or disappear in front of it. NVIDIA's 3D-fluids chapter gives a concrete implementation of limiting ray traversal by scene depth. [CAD30](https://developer.nvidia.com/gpugems/gpugems3/part-v-physics-simulation/chapter-30-real-time-simulation-and-rendering-3d-fluids)

### 12.3 Performance without scientific ambiguity

Useful optimizations include early termination, empty-space skipping, coarse preview resolution during interaction, brick min/max ranges, and cached transfer functions. They need correctness boundaries:

- A brick is not empty merely because its current transfer function happens to hide it; changing the transfer function invalidates that skip decision.
- A coarse mipmap can erase thin jets, walls, and peaks. Label interactive previews and recompute final views at the requested quality.
- Lighting gradients are rendering derivatives; do not reuse them as certified physical gradients.
- Color/opacity edits should not mutate solver arrays.
- A camera change should not change a numerical measurement.

Scientific volume rendering can be valuable for tracer distribution and spatial context. Place it after slices, measurements, and basic validation in the implementation order. Surface reflections, refraction, foam, caustics, and cinematic lighting are optional presentation work with no direct bearing on pressure-drop accuracy.

## 13. Export and scalable web/Rust architecture

### 13.1 First export: VTK XML plus a manifest

**For the current particle preview, precede the volume export with a particle export.** A .vtp dataset with explicit vertex cells can hold positions and point arrays for velocity, mass, temperature, material ID, and newly exposed stage-aligned density/pressure. Include stable particle IDs and release/source metadata when implemented. Export the actual unquantized arrays, not the appearance voxelization. For early debugging, a small CSV plus JSON manifest is also useful; do not imply the existing UI already provides this export. Reconstructed .vti fields should be separate derived products with kernel and sampling metadata.

For a uniform Cartesian dataset, VTK ImageData (.vti) can represent three-dimensional scalar/vector fields in physical coordinates. PolyData (.vtp) is suitable for surfaces and curves; UnstructuredGrid (.vtu) is suitable for explicit arbitrary cells. VTK's XML specification defines dimensions/extents, origin/spacing, field association, and binary/appended representations. [CAD31](https://docs.vtk.org/en/latest/vtk_file_formats/vtkxml_file_format.html)

For Nx×Ny×Nz cells, an ImageData point lattice normally has (Nx+1)×(Ny+1)×(Nz+1) points and cell arrays have NxNyNz tuples. A different valid choice is a point lattice of cell-center samples, but then its geometry describes a sample lattice rather than the original cell volumes. Name that choice explicitly. Test physical locations, not merely tuple counts.

A single cell/point association cannot faithfully encode all three native staggered components as one colocated vector without transformation. Recommended first export:

- Main cell grid: native cell scalars, masks, component IDs, and a clearly labeled reconstructed vector for standard viewers.
- Optional separate component grids: native u/v/w with their own extents and physical offsets.
- Surface dataset: numerical boundary geometry, patch IDs, available native boundary fields.
- CSV/JSON: measurements and diagnostics with real timestamps and definitions.
- Manifest: units, solver/model provenance, pressure gauges, fraction semantics, transforms, array layout, and checksums.

Use an actual-time collection/series mechanism supported by the target ParaView version. Do not equate frame number with time. Before adding compression or large parallel files, round-trip a tiny asymmetric fixture and verify coordinates, values, components, time, and association in an independent reader. An asymmetric field catches transposed axes that a sphere or symmetric vortex can conceal.

### 13.2 Later formats

VTKHDF is an evolving HDF5-based option with explicit temporal layouts and an implementation-status matrix. Check the exact reader/writer versions and dataset types rather than assuming the specification implies support in every installed ParaView or VTK build. Current documentation describes temporal ImageData and its time-first storage layout. [CAD32](https://docs.vtk.org/en/v9.7.0/vtk_file_formats/vtkhdf_file_format/vtkhdf_specifications.html), [CAD33](https://docs.vtk.org/en/v9.7.0/vtk_file_formats/vtkhdf_file_format/vtkhdf_status.html)

Large transient datasets benefit from chunked arrays, selective field loading, and separate static geometry. A mesh format for presentation is not automatically an analysis archive: a colored GLB, STL, or screenshot generally does not preserve the original volume field, staggered association, pressure reference, or measurement definitions. Keep scientific data and presentation exports separate even when both are convenient.

### 13.3 Three implementation routes

**Route A: external ParaView first.** The lowest-risk reference route. Write a small correct exporter, reproduce the intended analysis pipeline in ParaView, and use analytical fixtures and numerical comparisons to isolate solver/export/viewer issues. It supplies an independent implementation, not automatic validation of the solver.

**Route B: web viewer using vtk.js or VTK.wasm.** vtk.js provides browser-oriented visualization components and remote-geometry/rendering integration; VTK.wasm makes C++ VTK pipelines available through WebAssembly. Feature sets, bundle size, and browser/backend support should be tested for the chosen version. [CAD34](https://kitware.github.io/vtk-js/docs/), [CAD35](https://kitware.github.io/vtk-wasm/) Do not assume vtk.js contains every desktop VTK filter. Its documented ImageStreamline is a specific RK2 ImageData algorithm and does not currently interpolate input arrays onto output streamline points, so speed coloring and richer termination/validity semantics may need additional work. [CAD36](https://kitware.github.io/vtk-js/api/Filters_General_ImageStreamline.html)

**Route C: shared Rust analysis plus wgpu presentation.** Keep a pure analysis library for metadata, sampling, reductions, derivatives, integration, and export. Use wgpu for slice/volume textures, geometry, glyph instances, and curve rendering. wgpu supports native backends and browser WebGPU/WebGL paths, but support levels and available features differ. [CAD37](https://wgpu.rs/), [CAD38](https://wgpu.rs/doc/wgpu/struct.Backends.html) Query limits and formats on the actual adapter. Do not hard-code a 3D texture size or assume compute shaders and float filtering are universally available. [CAD39](https://docs.rs/wgpu/latest/wgpu/struct.Limits.html)

For Rheon, choose based on existing UI architecture and deployment constraints, after code review. A small custom native inspector can be appropriate; a browser prototype can use vtk.js; a desktop reference can remain ParaView. The scientific contract should be common to all three.

### 13.4 Memory budgets that matter

These are arithmetic estimates for uncompressed Float32 cell scalars, excluding halos, geometry, masks, GPU padding, and temporary arrays:

| Grid | Cells | One scalar | Three-component vector | Six scalars |
|---|---:|---:|---:|---:|
| 64³ | 262,144 | 1 MiB | 3 MiB | 6 MiB |
| 128³ | 2,097,152 | 8 MiB | 24 MiB | 48 MiB |
| 256³ | 16,777,216 | 64 MiB | 192 MiB | 384 MiB |
| 512³ | 134,217,728 | 512 MiB | 1.5 GiB | 3 GiB |

Two time snapshots, CPU and GPU copies, gradients, staging buffers, and an extracted surface can multiply these totals. A full 3×3 gradient alone is nine scalars per cell. Compute derived fields on demand or over regions, cache with bounds, and avoid retaining every temporary array.

A 256³ scalar at 30 uploads/s is roughly 1.875 GiB/s of raw upload data, before overhead. Upload once per accepted displayed frame, not once per camera movement. Changing a color map should normally update a small lookup texture, not resend the volume.

Run heavy CPU analysis in a worker/task with cancellation. Coalesce rapid slice-slider updates and never show a stale result under a new timestamp. Publish immutable accepted snapshots, or use a ownership/synchronization scheme that prevents the renderer from reading a partially updated state. For truly large data, server-side extraction/rendering can reduce client load; ParaView distinguishes transmitting geometry from transmitting rendered images. Its documented server mode also requires proper access/network security rather than public exposure of an unprotected server. [CAD40](https://docs.paraview.org/en/latest/ReferenceManual/parallelDataVisualization.html)

## 14. A proposed analysis API

The important design choice is not a particular class name. It is that the numerical meaning lives outside the renderer.

### Snapshot

Contains immutable run/state IDs, physical time, grid/domain/boundary definitions, field descriptors, native arrays, masks, materials, reference conventions, and diagnostics.

### Analysis operations

- sample(field, positions, policy) → values + validity + provenance
- reconstruct_velocity(location, policy) → vector field + provenance
- gradient(field, boundary_policy) → derivatives + quality/validity
- section_flux(section, phase, method) → signed/forward/reverse flux + area
- section_average(field, section, weight) → value + denominator + validity
- surface_load(boundaries, origin, stress_model) → pressure/viscous/total force and moment
- trace_streamlines(snapshot, seeds, integrator) → polylines + sample arrays + termination reasons
- trace_pathlines(time_source, releases, integrator) → trajectories + time/validity
- extract_isosurface(field, level, reconstruction) → mesh + provenance
- history(measurement, time_range) → series + missing intervals

### Presentation state

Contains camera, visible representations, selected scalar/component, units display, range policy, transfer functions, clipping, glyph size, and display quality. It refers to immutable analysis outputs. It does not own pressure-drop formulas or silently rewrite native arrays.

### Common error policy

Return typed unavailable/invalid results. Examples: “pressure reference differs between sections,” “no wall stress model,” “section has zero valid area,” “flux-weighted denominator is near zero,” and “pathline crosses an unsaved time interval.” The UI can explain these plainly; headless tests can assert them exactly.

### Suggested acceptance hierarchy

1. Mathematical unit tests on exact fields.
2. Data-layout and physical-coordinate tests.
3. Boundary/mask and invalid-data tests.
4. Independent export/readback comparisons.
5. Qualified solver examples and refinement checks.
6. Rendering regression tests, including legends and invalid masks.
7. Usability/performance tests on representative hardware.

An image-regression test cannot replace the first five. Two wrong samplers can produce identical images.

## 15. Seven interactive worked lessons {#seven-interactive-worked-lessons}

These are proposed controls and acceptance criteria. The companion computes analytic/synthetic numbers only; it launches no UI or native simulation. Run it from the repository root with `python3 docs/research-book/companion/cad_visualization_fixtures.py --output .generated/cad-visualization/numerical-fixtures.json`. The output is a generated artifact, not a committed numerical baseline.

These lessons are deliberately designed so a reader can reason about the answer before trusting the picture. The first, third, fourth, fifth, sixth, and seventh can use analytic or synthetic fields without claiming they were produced by Rheon. Label data origin conspicuously. The second becomes a solver demonstration only after the specified model and comparison are implemented and tested.

### Lab 1. A bright volume can hide a moving fluid

**Question:** why is a density image insufficient?

**Field:** on a unit box, set ρ=1000 kg/m³ and α=1 everywhere. Compare u=(0,0,0), u=(1,0,0) m/s, and a prescribed linear shear u=(y,0,0) with y in meters and shear rate 1 s⁻¹.

**Controls:** field selector, slice location, glyph toggle, normalization toggle, and a fixed color range.

**Expected observation:** density/fraction views are identical across all three fields; speed and arrows distinguish them immediately. A normalized arrow view cannot distinguish speeds at different y without color or labels.

**Measurement:** through the unit x-normal face, the three volume fluxes are 0, 1, and 0.5 m³/s. Reverse the normal and every sign reverses. This demonstrates an engineering measurement that opacity alone cannot provide.

**Tests:** zero-vector glyph handling; units; orientation; constant/linear sampling; flux integration. No claim is made that the shear field satisfies the box's chosen wall conditions.

### Lab 2. Pressure-driven planar channel

**Question:** can a solver reproduce a known profile, flow, and wall shear simultaneously?

**Model:** steady, incompressible, Newtonian, fully developed laminar flow between parallel plates y=0 and y=H; no slip; no transverse variation in the ideal model. Let G=−dp/dx>0.

From 0=G+μ d²u/dy² and u(0)=u(H)=0:

u(y)=G y(H−y)/(2μ), U_mean=G H²/(12μ), U_max=1.5 U_mean.

For a represented width W, Q=W G H³/(12μ), Δp=GL, and |τw|=GH/2. If sidewalls are physically present and influential, this planar model is no longer the exact reference; use a periodic spanwise direction or explicitly state the infinite-width assumption.

**Numerical example:** H=0.01 m, W=0.02 m, L=0.1 m, μ=0.001 Pa·s, ρ=1000 kg/m³, G=1 Pa/m. Then U_mean=0.0083333333 m/s, U_max=0.0125 m/s, Q=1.6666667×10⁻⁶ m³/s, Δp=0.1 Pa, and |τw|=0.005 Pa. Reynolds number based on H and mean speed is 83.3333; based on 2H it is 166.6667. State which definition is used.

**Controls:** grid resolution, driving gradient, viscosity, slice position, profile overlay, native/smooth display, and pressure reference shift.

**Linked views:** channel geometry, speed slice, cross-channel profile against the analytical curve, pressure along x, Q history, wall-shear values, and refinement plot.

**Expected lessons:** doubling G doubles speed, flux, drop, and shear in this model; doubling μ halves speed/flux while the prescribed pressure gradient and shear remain unchanged; pressure gauge shifts do not alter within-component Δp or velocity. A refined display does not substitute for a refined solver.

**Failure probes:** if the speed profile passes but shear fails, inspect wall placement/gradients; if Q differs from integrated profile, inspect native flux areas, staggering, or quadrature; if a closed domain is used accidentally, the setup is not this channel problem.

### Lab 3. Vorticity is not a vortex detector

**Question:** what do vorticity, Q, and λ₂ distinguish?

**Fields:** rigid rotation (−a y,a x,0), simple shear (γy,0,0), planar strain (a x,−a y,0), and isotropic expansion (a x,a y,a z).

**Controls:** field type, a or γ, translation velocity, derivative resolution, and the selected invariant.

**Expected numbers:** use the exact table in Section 8.5. For a=2 s⁻¹, rigid rotation has |ω|=4 s⁻¹, Q=4 s⁻², and λ₂=−4 s⁻². For γ=2 s⁻¹, simple shear has |ω|=2 s⁻¹ but Q=λ₂=0. For isotropic expansion with a=2, divergence is 6 s⁻¹, Q_rot=−6 s⁻², and Q_inv=12 s⁻².

**Lesson:** an attractive nonzero-vorticity region is not enough to characterize a vortex, and a divergence error can change the meaning of an unlabeled Q implementation. The expansion fixture is deliberately not an incompressible flow.

### Lab 4. Streamline, pathline, and streakline disagree

**Question:** what is lost by freezing a time-dependent velocity field?

Use the prescribed field u=(U,a t,0), with U>0 and a a constant acceleration. A streamline at t=T is a straight line with slope aT/U. A particle released from the origin at time τ has

X(t)=U(t−τ), Y(t)=½a(t²−τ²).

At observation T, the streakline of particles released continuously from the origin is

Y=(aT/U)X−aX²/(2U²), for 0≤X≤U(T−τ_min).

A pathline of the particle released at τ=0 follows Y=aX²/(2U²). For U=1 m/s, a=1 m/s², and T=2 s: the current streamline through the origin is Y=2X; the τ=0 pathline is Y=X²/2 and ends at (2,2); the streakline is Y=2X−X²/2, also ending at that oldest particle.

**Controls:** observation time, release interval, saved-snapshot spacing, integration step, and curve type. Show a shared seed and use distinct labels rather than only color.

**Lesson:** the three curves answer different questions even in a very simple field. Coarsening integration or saved time samples demonstrates numerical error without invoking complicated turbulence. This is an analytic visualization test, not a boundary-conditioned Rheon solution.

### Lab 5. The color legend can conceal a tenfold change

**Field:** f(x,t)=a(t)x on x∈[0,1] m, where f is an illustrative speed scalar and a changes from 1 to 10 s⁻¹.

**Controls:** animate amplitude; switch current-frame autoscale versus locked 0–10 m/s; hover a fixed x; display min/max history.

**Expected observation:** autoscaled images can look identical while the value at x=0.5 m rises from 0.5 to 5 m/s. Locked range and the probe expose the difference. Add an invalid region and an above-range region to test whether the display falsely maps either to an ordinary endpoint color.

**Lesson:** a legend is part of the data, not decorative furniture. This lab is especially effective before asking readers to compare design screenshots.

### Lab 6. Does the analysis converge, or only look smoother?

Use a synthetic quantity f(h)=1+2h² for h=0.25,0.125,0.0625. The values are 1.125,1.03125,1.0078125. With fine/medium/coarse ordering, the observed order is 2 and Richardson extrapolation gives exactly 1 under this constructed model. A three-grid fine GCI formula with F_s=1.25 gives about 0.9690%.

**Controls:** add an O(h³) term; add a fixed model bias; add small roundoff-like perturbations; switch to nonmonotone values; change display interpolation independently.

**Expected lesson:** display smoothing does not change the quantity; a constant model bias survives grid refinement; a near-zero reference value or nonmonotone sequence can make a rote relative estimate misleading. Clearly label this a manufactured convergence lesson rather than evidence about Rheon's current error.

### Lab 7. Particle density, coverage, and appearance are different

**Question:** why can the edge of a particle cloud show lower reconstructed density even though every particle has the same mass and reference volume?

**Fixture:** place 125 particles at all combinations of x,y,z∈{−0.2,−0.1,0,0.1,0.2} m. Give each particle mass 1 kg and reference volume 0.001 m³. Use the conventional poly6 kernel from Section 3.5 with support radius 0.2 m. This is a static mathematical fixture, not a Rheon fluid state or equilibrium claim.

**Controls:** probe position, kernel support radius, display-grid resolution, mass-density versus normalized constant-attribute reconstruction, and point/appearance/slice representations.

**Expected values:** total particle mass is always 125 kg and reference parcel-volume inventory is always 0.125 m³. With the stated kernel, the density sum at the origin is approximately 1009.775167 kg/m³; at the outer-face particle position (−0.2,0,0) m it is approximately 816.999908 kg/m³. The radially integrated kernel is one, while the finite particle lattice does not exactly reproduce a continuous uniform medium and has truncated support at its boundary.

**Lesson:** a normalized display interpolant can reproduce a constant attribute despite missing support; it does not correct the physical density calculation. More display pixels do not add particles. Changing opacity or hiding a material does not change mass. Record the kernel, support radius, and data origin beside the legend.

**Rheon connection:** once quantitative snapshots exist, use the same layout to show particle-native density, reconstructed density, support, and pressure evaluation stage side by side. Do not feed the synthetic fixture's expected values into a Rheon test until its exact kernel/material/self-contribution conventions have been matched.

## 16. What to ship, and what to defer

### Rheon-specific prerequisite milestone

For the reviewed particle preview, ship the following before advertising a general engineering field inspector:

1. Explicit model/branch labels, with particle SPH separated from carrier projection/transport.
2. Public read-only, stage-aligned particle diagnostics and unquantized export; never pair a pre-motion pressure with a post-motion location without its stage label.
3. Stable particle IDs and source/birth metadata; selectable particle values and material-aware speed/temperature views.
4. Histories for accepted particle mass, reference parcel volume, emitted/admitted volume, pending emitter credits, and clearly named contact/source diagnostics.
5. Full quantitative snapshots at a controlled cadence if replay, pathlines, or mass-crossing histories are required. Existing preview-byte history cannot supply missing particle states retroactively.
6. Kernel reconstruction and support diagnostics before smooth particle-field slices, volume rendering, or isosurfaces.

The carrier branch can independently gain structured-grid export, slices, and native flux measurements after its field units and retained boundary data are verified. Keep its outputs, examples, and evidence visibly separate.

### First usable engineering inspector

Ship only after the field contract, physical coordinates, and validity masks are tested:

- Geometry/domain/boundary overlay.
- Field selector with units and availability status.
- Axis slices with speed, velocity component, pressure if meaningful, tracer, fraction, and diagnostics.
- Sparse velocity glyphs and a numerical probe.
- Fixed/manual color ranges and invalid-data handling.
- One face-aligned native section flux and its time history.
- VTK export plus metadata and a tiny independent round-trip test.
- At least one analytic/prescribed fixture clearly separated from a solver run.

### Next quantitative layer

Add arbitrary slices, line profiles, saved section definitions, area/flux-weighted averages, pressure-drop histories, conservation charts, and a qualified channel case. Add gradients, vorticity, and streamlines with explicit reconstruction and integration tests. The renderer, headless path, and exported results must agree numerically.

### Later, conditional features

Add Q/λ₂ after the gradient/boundary implementation is verified. Add pathlines/streaklines after time sampling and state retention are adequate. Add wall shear and integrated loads after wall geometry, constitutive stress, pressure meaning, and boundary treatment are established. Add thermal, multiphase, or turbulence outputs only when the solver really supplies those models and has supporting evidence.

Volume rendering, advanced interface shading, LIC, and high-density glyphs can follow as presentation work. They should not block the first useful measurements. Scalable formats, bricking, in-situ analysis, and remote rendering should be driven by measured data sizes and bottlenecks rather than speculative architecture.

### Release review questions

1. Can a reader tell which fields were solved, prescribed, reconstructed, or unavailable?
2. Are all physical quantities labeled with units, frames, references, and time?
3. Does the same probe/measurement agree across UI, headless analysis, and export?
4. Do invalid cells, walls, disconnected regions, and missing time intervals remain explicit?
5. Are pressure differences, flux signs, surface normals, and moment origins tested?
6. Is the reported engineering quantity checked under relevant spatial/time refinement?
7. Is a rendered interface clearly separated from interface physics and conservation?
8. Are comparisons reproducible with saved operating conditions and measurement definitions?
9. Do fast-preview settings alter only presentation, never authoritative measurements?
10. Do capability claims match the currently verified solver mode?

The strongest first result is a small, reproducible laboratory in which a user can inspect a field, ask a quantitative question, see the corresponding measurement, and understand the evidence and limitations. That is the foundation on which a CAD-oriented CFD workflow can grow.

## References

The linked evidence bibliography supplies access notes and scopes for these sources. Links here are direct primary-source destinations.

- **CAD01.** ParaView, [Filtering Data](https://docs.paraview.org/en/latest/UsersGuide/filteringData.html).
- **CAD02.** NASA NPARC Alliance, [Verification Assessment](https://www.grc.nasa.gov/www/wind/valid/tutorial/verassess.html).
- **CAD03.** NASA NPARC Alliance, [Validation Assessment](https://www.grc.nasa.gov/www/wind/valid/tutorial/valassess.html).
- **CAD04.** OpenCFD, [pressure function object](https://doc.openfoam.com/2312/tools/post-processing/function-objects/field/pressure/).
- **CAD05.** Hirt and Nichols (1981), [Volume of fluid method for the dynamics of free boundaries](https://www.sciencedirect.com/science/article/pii/0021999181901455), DOI 10.1016/0021-9991(81)90145-5.
- **CAD06.** Bridson and Müller-Fischer (2007), [Fluid Simulation course notes](https://www.cs.ubc.ca/~rbridson/fluidsimulation/fluids_notes.pdf), especially grid and pressure sections.
- **CAD07.** VTK, [vtkCellDataToPointData](https://vtk.org/doc/nightly/html/classvtkCellDataToPointData.html).
- **CAD08.** VTK, [vtkFlyingEdges3D](https://vtk.org/doc/nightly/html/classvtkFlyingEdges3D.html).
- **CAD09.** Schroeder and Tsalikis, Kitware (2023), [Really Fast Isocontouring](https://www.kitware.com/really-fast-isocontouring/), discussing Flying Edges and the Schroeder–Maynard–Geveci LDAV 2015 paper.
- **CAD10.** MIT OpenCourseWare, [Fluids Lecture 8: Streamlines, Pathlines, Streaklines](https://live.ocw.mit.edu/courses/16-01-unified-engineering-i-ii-iii-iv-fall-2005-spring-2006/c37f038e7f899163071e1b9914a66f8d_f08_new_fall.pdf).
- **CAD11.** NASA Ames, [Visualization of Numerical Unsteady Fluid Flows](https://www.nas.nasa.gov/assets/nas/pdf/techreports/1995/nas-95-017.pdf).
- **CAD12.** VTK, [vtkStreamTracer](https://vtk.org/doc/nightly/html/classvtkStreamTracer.html).
- **CAD13.** VTK, [vtkParticleTracer](https://vtk.org/doc/nightly/html/classvtkParticleTracer.html).
- **CAD14.** Cabral and Leedom (1993), [Imaging vector fields using line integral convolution](https://www.osti.gov/servlets/purl/10185520), SIGGRAPH '93, DOI 10.1145/166117.166151.
- **CAD15.** OpenCFD, [surfaceFieldValue](https://doc.openfoam.com/2312/tools/post-processing/function-objects/field/surfaceFieldValue/).
- **CAD16.** OpenCFD, [forces](https://doc.openfoam.com/2306/tools/post-processing/function-objects/forces/forces/).
- **CAD17.** OpenCFD, [wallShearStress](https://doc.openfoam.com/2312/tools/post-processing/function-objects/field/wallShearStress/).
- **CAD18.** OpenCFD, [Wall functions](https://doc.openfoam.com/2312/tools/processing/models/turbulence/ras/wall-functions/).
- **CAD19.** OpenCFD, [forceCoeffs implementation](https://api.openfoam.com/2506/forceCoeffs_8C_source.html).
- **CAD20.** OpenCFD, [Q](https://doc.openfoam.com/2212/tools/post-processing/function-objects/field/Q/).
- **CAD21.** Jeong and Hussain (1995), [On the identification of a vortex](https://www.cambridge.org/core/journals/journal-of-fluid-mechanics/article/abs/on-the-identification-of-a-vortex/D26006DDB95FB28DA80E28A581182DF1), Journal of Fluid Mechanics 285, 69–94, DOI 10.1017/S0022112095000462.
- **CAD22.** OpenCFD, [Lambda2](https://api.openfoam.com/2606/classFoam_1_1functionObjects_1_1Lambda2.html).
- **CAD23.** CGAL, [Polygon Mesh Processing user manual](https://doc.cgal.org/latest/Polygon_mesh_processing/index.html).
- **CAD24.** Moreland (2009), [Diverging Color Maps for Scientific Visualization, expanded paper](https://www.kennethmoreland.com/color-maps/ColorMapsExpanded.pdf), DOI 10.1007/978-3-642-10520-3_9.
- **CAD25.** ParaView, [Color maps and transfer functions](https://docs.paraview.org/en/latest/ReferenceManual/colorMapping.html).
- **CAD26.** Moreland, [Color Map Advice for Scientific Visualization](https://www.kennethmoreland.com/color-advice/).
- **CAD27.** NASA NPARC Alliance, [Examining Spatial (Grid) Convergence](https://www.grc.nasa.gov/www/wind/valid/tutorial/spatconv.html).
- **CAD28.** Housman et al. (2011), NASA, [Space Time Accuracy Assessment of CFD Simulations for the Launch Environment](https://www.nas.nasa.gov/assets/nas/pdf/papers/housman_j_space_time_accuracy_2011.pdf).
- **CAD29.** NVIDIA GPU Gems, [Chapter 39: Volume Rendering Techniques](https://developer.nvidia.com/gpugems/gpugems/part-vi-beyond-triangles/chapter-39-volume-rendering-techniques).
- **CAD30.** NVIDIA GPU Gems 3, [Chapter 30: Real-Time Simulation and Rendering of 3D Fluids](https://developer.nvidia.com/gpugems/gpugems3/part-v-physics-simulation/chapter-30-real-time-simulation-and-rendering-3d-fluids).
- **CAD31.** VTK, [XML File Formats](https://docs.vtk.org/en/latest/vtk_file_formats/vtkxml_file_format.html).
- **CAD32.** VTK, [VTKHDF format specification](https://docs.vtk.org/en/v9.7.0/vtk_file_formats/vtkhdf_file_format/vtkhdf_specifications.html).
- **CAD33.** VTK, [VTKHDF implementation status](https://docs.vtk.org/en/v9.7.0/vtk_file_formats/vtkhdf_file_format/vtkhdf_status.html).
- **CAD34.** Kitware, [vtk.js overview](https://kitware.github.io/vtk-js/docs/).
- **CAD35.** Kitware, [VTK.wasm](https://kitware.github.io/vtk-wasm/).
- **CAD36.** vtk.js, [ImageStreamline API](https://kitware.github.io/vtk-js/api/Filters_General_ImageStreamline.html).
- **CAD37.** wgpu, [project overview](https://wgpu.rs/).
- **CAD38.** wgpu, [Backends](https://wgpu.rs/doc/wgpu/struct.Backends.html).
- **CAD39.** wgpu, [Limits](https://docs.rs/wgpu/latest/wgpu/struct.Limits.html).
- **CAD40.** ParaView, [Remote and parallel visualization](https://docs.paraview.org/en/latest/ReferenceManual/parallelDataVisualization.html).
- **CAD41.** Price (2012), [Smoothed Particle Hydrodynamics and Magnetohydrodynamics](https://arxiv.org/abs/1012.1885), Journal of Computational Physics 231, 759–794; [author manuscript](https://users.monash.edu.au/~dprice/pubs/spmhd/price-spmhd-jcp.pdf).
- **CAD42.** Müller, Charypar, and Gross (2003), [Particle-Based Fluid Simulation for Interactive Applications](https://matthias-research.github.io/pages/publications/sca03.pdf).
- **CAD43.** Schechter (2012), [Enhancing Particle Methods for Fluid Simulation in Computer Graphics](https://www.cs.ubc.ca/~rbridson/docs/schechter_phd.pdf), especially the Ghost SPH boundary-condition work.
- **CAD44.** Dehnen and Aly (2012), [Improving convergence in smoothed particle hydrodynamics simulations without pairing instability](https://arxiv.org/abs/1204.2471).
