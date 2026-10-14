# CAD visualization: evidence bibliography

Checked 12 October 2026. Primary scientific papers, official documentation, and commit-anchored repository evidence only. Source versions are deliberately preserved where a fixed version was inspected; latest/nightly pages can change. Recheck implementation support when choosing a dependency version.

The 44 primary-source identities and original 17 repository citations are preserved. IDs use the CAD prefix to avoid collision with the book's existing source labels; four emitter anchors complete the supplied inventory. See the [machine-readable ledger](cad-visualization-sources.json) and [technical guide](../implementation/cad-visualization-contracts.md).

## Scientific and implementation references

### CAD01. ParaView, [Filtering Data](https://docs.paraview.org/en/latest/UsersGuide/filteringData.html).

Supports: Established slice/clip, glyph, contour, sampling, and time-processing workflow; structured-to-unstructured memory caveats.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD02. NASA NPARC Alliance, [Verification Assessment](https://www.grc.nasa.gov/www/wind/valid/tutorial/verassess.html).

Supports: Code verification versus calculation verification; consistency, convergence, and accurate-solution comparisons.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD03. NASA NPARC Alliance, [Validation Assessment](https://www.grc.nasa.gov/www/wind/valid/tutorial/valassess.html).

Supports: Application-specific physical validation, experimental uncertainty, and building-block comparisons.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD04. OpenCFD, [pressure function object](https://doc.openfoam.com/2312/tools/post-processing/function-objects/field/pressure/).

Supports: Physical versus kinematic pressure, total-pressure conversion, coefficient reference inputs, and hydrostatic transformation.

Access: Primary OpenCFD documentation read through indexed page content; some direct page opens returned HTTP 403. Relevant API/source pages were also used where available. No login or access restriction was bypassed.

### CAD05. Hirt and Nichols (1981), [Volume of fluid method for the dynamics of free boundaries](https://www.sciencedirect.com/science/article/pii/0021999181901455), DOI 10.1016/0021-9991(81)90145-5.

Supports: Fractional-volume representation of free boundaries; historical VOF method, not proof of Rheon phase physics.

Access: Primary publisher/author/preprint material retrieved. Research conclusions are scoped to the paper and are not evidence about Rheon.

### CAD06. Bridson and Müller-Fischer (2007), [Fluid Simulation course notes](https://www.cs.ubc.ca/~rbridson/fluidsimulation/fluids_notes.pdf), especially grid and pressure sections.

Supports: Staggered MAC locations, discrete pressure gradients/divergence, and practical fluid discretization background.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD07. VTK, [vtkCellDataToPointData](https://vtk.org/doc/nightly/html/classvtkCellDataToPointData.html).

Supports: Cell-to-point transfer is an averaging transformation, not preservation of original cell values.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD08. VTK, [vtkFlyingEdges3D](https://vtk.org/doc/nightly/html/classvtkFlyingEdges3D.html).

Supports: Flying Edges extraction strategy, output allocation, threading, and degenerate-triangle warning.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD09. Schroeder and Tsalikis, Kitware (2023), [Really Fast Isocontouring](https://www.kitware.com/really-fast-isocontouring/), discussing Flying Edges and the Schroeder–Maynard–Geveci LDAV 2015 paper.

Supports: Algorithm-author explanation of edge-based isocontouring and data-dependent performance improvements; no Rheon benchmark.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD10. MIT OpenCourseWare, [Fluids Lecture 8: Streamlines, Pathlines, Streaklines](https://live.ocw.mit.edu/courses/16-01-unified-engineering-i-ii-iii-iv-fall-2005-spring-2006/c37f038e7f899163071e1b9914a66f8d_f08_new_fall.pdf).

Supports: Formal distinctions among streamlines, pathlines, and streaklines.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD11. NASA Ames, [Visualization of Numerical Unsteady Fluid Flows](https://www.nas.nasa.gov/assets/nas/pdf/techreports/1995/nas-95-017.pdf).

Supports: Unsteady CFD visualization and why instantaneous and time-dependent traces differ.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD12. VTK, [vtkStreamTracer](https://vtk.org/doc/nightly/html/classvtkStreamTracer.html).

Supports: Streamline integrators, arc-length stepping, limits, interpolation, and termination-reason output.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD13. VTK, [vtkParticleTracer](https://vtk.org/doc/nightly/html/classvtkParticleTracer.html).

Supports: Unsteady particle tracing, temporal interpolation, and mesh-over-time behavior.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD14. Cabral and Leedom (1993), [Imaging vector fields using line integral convolution](https://www.osti.gov/servlets/purl/10185520), SIGGRAPH '93, DOI 10.1145/166117.166151.

Supports: Original line integral convolution method: noise/texture filtering along vector fields.

Access: Original paper retrieved from the US DOE OSTI primary repository. DOI confirmed separately; direct DOI open was unavailable.

### CAD15. OpenCFD, [surfaceFieldValue](https://doc.openfoam.com/2312/tools/post-processing/function-objects/field/surfaceFieldValue/).

Supports: Surface sums, area/weighted/normal operations, and sampling/integration caveats.

Access: Primary OpenCFD documentation read through indexed page content; some direct page opens returned HTTP 403. Relevant API/source pages were also used where available. No login or access restriction was bypassed.

### CAD16. OpenCFD, [forces](https://doc.openfoam.com/2306/tools/post-processing/function-objects/forces/forces/).

Supports: Pressure/viscous/porous force and moment decomposition and pressure-reference configuration.

Access: Primary OpenCFD documentation read through indexed page content; some direct page opens returned HTTP 403. Relevant API/source pages were also used where available. No login or access restriction was bypassed.

### CAD17. OpenCFD, [wallShearStress](https://doc.openfoam.com/2312/tools/post-processing/function-objects/field/wallShearStress/).

Supports: Wall-shear field and its documented dimensions; warns against assuming all stress outputs are Pa.

Access: Primary OpenCFD documentation read through indexed page content; some direct page opens returned HTTP 403. Relevant API/source pages were also used where available. No login or access restriction was bypassed.

### CAD18. OpenCFD, [Wall functions](https://doc.openfoam.com/2312/tools/processing/models/turbulence/ras/wall-functions/).

Supports: Near-wall modeling and wall-function context.

Access: Primary OpenCFD documentation read through indexed page content; some direct page opens returned HTTP 403. Relevant API/source pages were also used where available. No login or access restriction was bypassed.

### CAD19. OpenCFD, [forceCoeffs implementation](https://api.openfoam.com/2506/forceCoeffs_8C_source.html).

Supports: Reference scales and axes for force/moment coefficients; nonzero-scale checks in implementation.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD20. OpenCFD, [Q](https://doc.openfoam.com/2212/tools/post-processing/function-objects/field/Q/).

Supports: OpenFOAM Q is the full second invariant of velocity gradient, with inverse-time-squared units.

Access: Primary OpenCFD documentation read through indexed page content; some direct page opens returned HTTP 403. Relevant API/source pages were also used where available. No login or access restriction was bypassed.

### CAD21. Jeong and Hussain (1995), [On the identification of a vortex](https://www.cambridge.org/core/journals/journal-of-fluid-mechanics/article/abs/on-the-identification-of-a-vortex/D26006DDB95FB28DA80E28A581182DF1), Journal of Fluid Mechanics 285, 69–94, DOI 10.1017/S0022112095000462.

Supports: Original lambda2 vortex-identification paper; use with the direct implementation definition in CAD22.

Access: Publisher page and indexed paper material inspected; detailed implementation definition corroborated by CAD22. No claim of a new reproduction of the paper experiments.

### CAD22. OpenCFD, [Lambda2](https://api.openfoam.com/2606/classFoam_1_1functionObjects_1_1Lambda2.html).

Supports: Middle eigenvalue of squared symmetric plus antisymmetric velocity-gradient parts.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD23. CGAL, [Polygon Mesh Processing user manual](https://doc.cgal.org/latest/Polygon_mesh_processing/index.html).

Supports: Mesh topology versus geometric defects; intersections, connected components, normals, and repair organization.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD24. Moreland (2009), [Diverging Color Maps for Scientific Visualization, expanded paper](https://www.kennethmoreland.com/color-maps/ColorMapsExpanded.pdf), DOI 10.1007/978-3-642-10520-3_9.

Supports: Perceptual problems of rainbow color maps and diverging-map design.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD25. ParaView, [Color maps and transfer functions](https://docs.paraview.org/en/latest/ReferenceManual/colorMapping.html).

Supports: Independent color/opacity transfer functions and explicit automatic range-rescaling policies.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD26. Moreland, [Color Map Advice for Scientific Visualization](https://www.kennethmoreland.com/color-advice/).

Supports: Current author guidance on scalar color, luminance, and shape-shading interactions.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD27. NASA NPARC Alliance, [Examining Spatial (Grid) Convergence](https://www.grc.nasa.gov/www/wind/valid/tutorial/spatconv.html).

Supports: Richardson extrapolation/GCI assumptions, three-grid comparison, safety factor, and failure caveats.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD28. Housman et al. (2011), NASA, [Space Time Accuracy Assessment of CFD Simulations for the Launch Environment](https://www.nas.nasa.gov/assets/nas/pdf/papers/housman_j_space_time_accuracy_2011.pdf).

Supports: A primary space-time convergence assessment example; not a universal error guarantee.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD29. NVIDIA GPU Gems, [Chapter 39: Volume Rendering Techniques](https://developer.nvidia.com/gpugems/gpugems/part-vi-beyond-triangles/chapter-39-volume-rendering-techniques).

Supports: Direct volume-rendering optical model, sampling, compositing, and opacity-rate correction.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD30. NVIDIA GPU Gems 3, [Chapter 30: Real-Time Simulation and Rendering of 3D Fluids](https://developer.nvidia.com/gpugems/gpugems3/part-v-physics-simulation/chapter-30-real-time-simulation-and-rendering-3d-fluids).

Supports: Ray marching through a simulated volume with opaque-scene depth clipping.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD31. VTK, [XML File Formats](https://docs.vtk.org/en/latest/vtk_file_formats/vtkxml_file_format.html).

Supports: VTK XML dataset types, extents, coordinates, association, and binary/appended representations.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD32. VTK, [VTKHDF format specification](https://docs.vtk.org/en/v9.7.0/vtk_file_formats/vtkhdf_file_format/vtkhdf_specifications.html).

Supports: VTKHDF dataset and temporal layout specification, including temporal ImageData dimension order.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD33. VTK, [VTKHDF implementation status](https://docs.vtk.org/en/v9.7.0/vtk_file_formats/vtkhdf_file_format/vtkhdf_status.html).

Supports: Reader/writer support differs from the format specification; version-specific status and limitations.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD34. Kitware, [vtk.js overview](https://kitware.github.io/vtk-js/docs/).

Supports: vtk.js browser visualization and remote data/rendering architecture, not parity with all desktop VTK filters.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD35. Kitware, [VTK.wasm](https://kitware.github.io/vtk-wasm/).

Supports: VTK C++ pipelines compiled to WebAssembly; browser processing/rendering route.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD36. vtk.js, [ImageStreamline API](https://kitware.github.io/vtk-js/api/Filters_General_ImageStreamline.html).

Supports: vtk.js ImageStreamline uses RK2 on ImageData and has a narrower feature contract than desktop StreamTracer.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD37. wgpu, [project overview](https://wgpu.rs/).

Supports: wgpu native/browser graphics API scope.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD38. wgpu, [Backends](https://wgpu.rs/doc/wgpu/struct.Backends.html).

Supports: Backend support and first-tier versus secondary support distinctions.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD39. wgpu, [Limits](https://docs.rs/wgpu/latest/wgpu/struct.Limits.html).

Supports: Device/adapter resource limits must be queried and requested appropriately.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD40. ParaView, [Remote and parallel visualization](https://docs.paraview.org/en/latest/ReferenceManual/parallelDataVisualization.html).

Supports: Remote versus parallel processing, geometry versus image delivery, and unprotected-server security warning.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD41. Price (2012), [Smoothed Particle Hydrodynamics and Magnetohydrodynamics](https://arxiv.org/abs/1012.1885), Journal of Computational Physics 231, 759–794; [author manuscript](https://users.monash.edu.au/~dprice/pubs/spmhd/price-spmhd-jcp.pdf).

Supports: SPH density estimate/interpolation versus a consistent equation discretization; numerical caveats.

Access: Primary publisher/author/preprint material retrieved. Research conclusions are scoped to the paper and are not evidence about Rheon.

### CAD42. Müller, Charypar, and Gross (2003), [Particle-Based Fluid Simulation for Interactive Applications](https://matthias-research.github.io/pages/publications/sca03.pdf).

Supports: Particle-fluid interpolation and poly6-style kernel reference; not confirmation of Rheon exact coefficients.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD43. Schechter (2012), [Enhancing Particle Methods for Fluid Simulation in Computer Graphics](https://www.cs.ubc.ca/~rbridson/docs/schechter_phd.pdf), especially the Ghost SPH boundary-condition work.

Supports: Ghost SPH/free-surface/solid-boundary numerical issues and boundary-support treatment.

Access: Primary document/documentation read via public web retrieval on 2026-10-12.

### CAD44. Dehnen and Aly (2012), [Improving convergence in smoothed particle hydrodynamics simulations without pairing instability](https://arxiv.org/abs/1204.2471).

Supports: SPH particle disorder, neighbor/kernel choices, convergence, and pairing-stability caveats.

Access: Primary publisher/author/preprint material retrieved. Research conclusions are scoped to the paper and are not evidence about Rheon.

## Rheon repository evidence

These exact links were supplied by the fresh engineering review at commit 2209012bc5f902693d0f695ec1c2d854126af317. They anchor the capability inventory, not a physical validation claim. Absence of material export is an inventory finding rather than an explicit API guarantee.

- R01: [Particle types, reports, and private scratch](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/particle_fluid.rs#L10)
- R02: [Public physical particle accessors](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/particle_fluid.rs#L274)
- R03: [Particle pressure assembly](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/particle_fluid.rs#L483)
- R04: [Motion and box contact](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/particle_fluid.rs#L627)
- R05: [Report evaluation stages](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/particle_fluid.rs#L686)
- R06: [Material equations of state](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/material.rs#L42)
- R07: [Adaptive scheduling policy](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/scheduling.rs#L1)
- R08: [Joined parallel chunks](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/parallel.rs#L34)
- R09: [SGS wave structure](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/sgs.rs#L1)
- R10: [Preview history payload](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/bin/desktop/history.rs#L15)
- R11: [Voxel appearance and filtering](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/bin/desktop/history.rs#L71)
- R12: [Immutable preview history frames](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/bin/desktop/history.rs#L134)
- R13: [CPU projection and depth sorting](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/bin/desktop/viewport.rs#L143)
- R14: [Viewport shading](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/bin/desktop/viewport.rs#L395)
- R15: [Desktop architecture](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/bin/desktop/README.md)
- R16: [Declared carrier exports](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/bin/rheon.rs#L68)
- R17: [Carrier export implementation](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/bin/rheon.rs#L146)

## Evidence boundaries

- Prior discussion supplied the research question, not independent implementation evidence.
- No Rheon simulation, industrial CAD case, or experimental validation was run for this research.
- The fixture verifier checks analytical/synthetic calculations only. It is not a test suite against Rheon implementation.
- Current pressure-stage, export, and history limitations materially affect what can be visualized honestly.
- Proposed architecture, prioritization, failure checks, and lesson controls are engineering recommendations, distinguished from source-established facts.
- No published performance number is asserted for Rheon; memory/bandwidth values in the chapter are elementary arithmetic estimates.

## Completed emitter citations

The integration review supplied and inspected four precise anchors at the same commit. They describe a prescribed source, not collision geometry or a simulated nozzle wall.

- R18: [Rotating source axis](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/emitter.rs#L34)
- R19: [Prescribed nozzle profile](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/emitter.rs#L78)
- R20: [Particle midpoint source admission and volume credits](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/particle_fluid.rs#L387)
- R21: [Accepted particle/source-credit publication](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/particle_fluid.rs#L698)
