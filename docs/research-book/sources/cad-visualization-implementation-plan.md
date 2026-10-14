# Rheon engineering-visualization implementation plan

Based on the reviewed inventory at commit 2209012bc5f902693d0f695ec1c2d854126af317. Research checked 12 October 2026. This is a proposed implementation sequence; it does not claim that the work below is already present.

This is a native material-inspection roadmap, not a statement that the existing browser carrier volume and instantaneous streamlines are absent. The [technical guide](../implementation/cad-visualization-contracts.md) gives the detailed contracts and [source ledger](cad-visualization-sources.json) fixes the evidence identities.

## The key decision

Treat the reference-volume SPH preview and the staggered carrier projection/transport path as different numerical products with different data contracts. Build the current particle inspector first. Preserve the carrier path as a separate structured-grid analysis/reference route. Do not borrow the carrier pressure correction or face flux to fill missing particle diagnostics.

## Stage 0: make the current state inspectable

### Particle branch

- Define an immutable quantitative snapshot containing accepted particle positions, velocities, masses, temperatures, material IDs, physical time, solver mode, source revision, and boundary/model settings.
- Expose density, pressure, acceleration/force components, and damping diagnostics read-only, with evaluation-stage identifiers. The current report's pressure maximum is evaluated before motion/wall-temperature updates; accepted speed uses a later state. Either keep these as distinct stage records or add a pure diagnostic evaluation at accepted positions/temperatures.
- Introduce stable particle IDs and birth/source records before trajectory/history features. Verify whether generation IDs currently identify the entire preview or individual particles; do not infer the latter.
- Preserve liquid compression-pressure semantics separately from gas rho-R-T absolute-pressure semantics. No combined pressure legend until a coherent reference has been defined.
- Keep unquantized particle data separate from appearance voxels and hidden-ambient display filtering.

### Carrier branch

- Verify native array dimensions, physical coordinates, face offsets, pressure-correction units/reference, active volumes, and open face areas.
- Determine which values/diagnostics are retained after accepted steps and which are transient scratch.
- Keep native flux definitions separate from SPH reconstructed/crossing estimates.

### Acceptance

Every exposed array has unit/association/stage/meaning/validity metadata. A selected particle's value agrees with the accepted snapshot. Stage-mismatched pressure/speed is impossible to present as a synchronized field by accident. Unsupported fields report unavailable, not zero.

## Stage 1: useful inspection and export

### Particle branch

- Render quantitative particle points/glyphs colored by speed, velocity component, material, and temperature, with an explicit temperature-model qualification.
- Add selected-particle readouts and hidden-material counts.
- Export a tiny particle CSV plus manifest for debugging, and .vtp vertex data for ParaView. Include all supported native arrays, physical units, IDs, timestamps, stage, source, and model metadata.
- Add accepted mass and reference-volume inventory, admitted/emitted parcel history, source momentum where available, and pending emitter-volume credits. Name bookkeeping quantities precisely.
- If useful and supported by the integrator, log box-contact impulses separately from pressure/viscosity/body/source contributions. Do not label them wall shear or CAD pressure loads.

### Carrier branch

- Export .vti cell data plus a clearly reconstructed vector; optionally native staggered components in separate grids.
- Add XY/XZ/YZ scalar slices, sparse velocity arrows, probe, fixed legend, and boundary/mask overlay.
- Implement one aligned section flux using actual native open-face flux semantics, if the data contract supports it.

### Acceptance

Round-trip an asymmetric fixture through an independent VTK reader. Check physical coordinates, axes, time, association, arrays, and exact small-data measurements. UI, headless output, and export agree within declared numeric tolerances. No public pressure map is synthesized from a report maximum.

## Stage 2: particle reconstruction and measurements

- Add a spatial-hash neighbor query with brute-force tiny-fixture comparisons.
- Implement explicitly named kernel mass-density and normalized attribute reconstruction. Preserve material/region boundaries, support radius, kernel normalization, and invalid-coverage flags.
- Add scalar slices and profiles with source-particle/coverage overlays. Pressure becomes eligible only after synchronized per-particle pressure is exposed.
- Keep native reference parcel volume, density-derived volume, and display isosurface volume separate.
- Add saved probe/section definitions and actual-time charts.
- Offer reconstructed section flux as an approximation with its quadrature and kernel settings. Add particle-crossing mass flow only after stable IDs and complete sufficiently frequent trajectories exist.

### Acceptance

Constant-attribute reconstruction passes; kernel/support units pass; masks do not bridge walls/material semantics; neighbor queries match brute force; integrated kernel mass agrees on a sufficiently large integration domain under refinement. Kernel clipping at boundaries is visible. Display resolution changes do not change native inventory.

## Stage 3: flow structure and time

- Implement gradients with an explicitly chosen branch-specific method. Add vorticity, Q_rot, Q_invariant, and λ₂ only after exact tensor fixtures pass.
- Add instantaneous streamlines with shared sampling, clear seed/integrator units, segment-wall checks, and termination reasons.
- Add quantitative snapshot retention at a bounded cadence/storage budget. Existing preview-byte history is not replayable solver state and cannot be upgraded retroactively by the viewer.
- Add actual particle tracks from saved IDs; separately add passive pathlines/streaklines from a reconstructed time-dependent field if useful.

### Acceptance

Rigid rotation, shear, strain, and expansion have the expected invariants. Uniform translation leaves gradients/invariants unchanged. Streamline integration converges as step decreases. Paths stop at missing time intervals or invalid geometry. The changing-uniform-flow lesson produces distinct streamline/pathline/streakline curves.

## Stage 4: qualify one engineering case

- Choose a case the verified physical model can represent, rather than starting with complex imported CAD. A planar channel is a candidate for a suitable viscous/grid branch; it is not automatically a valid example for the current box-contact SPH preview.
- Match geometry, material properties, source/boundary conditions, and analytical assumptions.
- Compare velocity profile, flow, pressure drop, and wall shear only where those quantities are supported. Record spatial/particle and timestep studies and the actual quantity of interest.
- For SPH, vary spacing/support/neighbor behavior, timestep, and boundary treatment deliberately. Visual mesh refinement alone is irrelevant.

### Acceptance

Publish raw comparison values, stated tolerances, and qualified claims. A green residual or a convincing image is not the acceptance criterion. No universal engineering-accuracy badge.

## Stage 5: advanced surfaces and CAD loads

- Add display isosurfaces and volume rendering from verified fields. Preserve thresholds, transfer functions, sampling distances, and reconstructed-versus-native status.
- Introduce CAD processing only with explicit geometry/mesh/boundary-region contracts and stable selections. The current emitter is a prescribed source, not a nozzle collision solid.
- Add native boundary traction or conservative load transfer only after wall geometry, pressure convention, stress model, and wall gradients are established.
- Keep pressure/viscous/contact/source forces separate; record moment origin and drag/lift reference scales.

### Acceptance

Uniform pressure on a closed surface gives zero net load within geometric tolerance. Reversing normals reverses traction signs. Patch loads expose reference pressure. Pressure-only load is not labeled total drag. A smooth CAD surface does not hide voxel/particle boundary approximations.

## Architecture and performance

- Shared analysis owns sampling/reconstruction, measurement, diagnostics, trajectories, and export. Presentation owns camera, clipping, color, opacity, and display quality.
- Use immutable snapshots and explicit cache keys based on state + field + algorithm + parameters.
- Existing Rayon work organization is not proof that a new postprocessor is race-free. Reuse the accepted-state boundary and avoid reading mutable integration scratch.
- Keep a small deterministic CPU reference for every accelerated algorithm. Preserve fixed-order or documented reductions where reproducibility matters.
- Query actual device limits before a wgpu/WebGPU path. A basic particle/slice inspector need not wait for a renderer migration.
- Cancel/coalesce stale interaction jobs; never relabel old results with a new time.
- Profile actual arrays and frame costs. A 256³ Float32 scalar is 64 MiB; gradients, temporal pairs, and CPU/GPU copies multiply memory quickly.

## Teaching-book integration

Use the research chapter as the depth reference for the reserved Chapter 14/CAD guide. Lead the chapter with the verified two-branch inventory and the meaning of a measurement. Build the interactive lessons from analytic/synthetic fixtures first, marking their data origin. Link a later verified solver fixture only when its actual acceptance evidence exists. Reuse the same field metadata, legend rules, and measurement API in the teaching view and product inspector.

## Definition of done for the first milestone

A user can select a real particle and read its accepted physical values, export a small quantitative snapshot, inspect speed/material/temperature honestly, distinguish pressure stages and solver branches, and verify mass/source bookkeeping. In the carrier branch, a user can inspect numerical fields and native aligned flux without relying on opacity. Every result has a reproducible data definition, and unsupported engineering quantities remain explicitly unavailable.
